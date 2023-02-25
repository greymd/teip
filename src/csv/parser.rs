// Many part of this module is based on https://github.com/BurntSushi/rust-csv/tree/master/csv-core
// which is dual-licensed under MIT and Unlicense as of 2023-02-21

use super::terminator::Terminator;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NfaState {
    // These states aren't used in the DFA, so we
    // assign them meaningless numbers.
    EndFieldTerm = 200,
    InRecordTerm = 201,
    // End = 202,

    // All states below are DFA states.
    StartRecord = 0,
    StartField = 1,
    InField = 2,
    InQuotedField = 3,
    InEscapedQuote = 4,
    InDoubleEscapedQuote = 5,
    InComment = 6,
    // All states below are "final field" states.
    // Namely, they indicate that a field has been parsed.
    EndFieldDelim = 7,
    // All states below are "final record" states.
    // Namely, they indicate that a record has been parsed.
    EndRecord = 8,
    CRLF = 9,
}

/// What should be done with input bytes during an NFA transition
#[derive(Clone, Debug, Eq, PartialEq)]
enum NfaInputAction {
    // Do not consume an input byte
    Epsilon,
    // Copy input byte to a caller-provided output buffer
    CopyToOutput,
    // Consume but do not copy input byte (for example, seeing a field
    // delimiter will consume an input byte but should not copy it to the
    // output buffer.
    Discard,
}

#[derive(Clone, Debug)]
pub struct Parser {
    /// The current NFA state
    nfa_state: NfaState,
    /// The last NFA state
    last_nfa_state: Option<NfaState>,
    /// The delimiter that separates fields.
    delimiter: char,
    /// The terminator that separates records.
    term: Terminator,
    /// The quotation byte.
    quote: char,
    /// Whether to recognize escaped quotes.
    escape: Option<char>,
    /// Whether to recognized doubled quotes.
    double_quote: bool,
    /// If enabled, lines beginning with this byte are ignored.
    comment: Option<char>,
    /// If enabled (the default), then quotes are respected. When disabled,
    /// quotes are not treated specially.
    quoting: bool,
    /// The current line.
    record: u64,
    /// The current field.
    field: u64,
    // /// Whether this parser has ever read anything.
    // has_read: bool,
    // /// The current position in the output buffer when reading a record.
    // output_pos: usize,
}


impl Default for Parser {
    fn default() -> Parser {
        Parser {
            nfa_state: NfaState::StartRecord,
            last_nfa_state: None,
            delimiter: ',',
            term: Terminator::default(),
            quote: '"',
            escape: None,
            double_quote: true,
            comment: None,
            quoting: true,
            record: 0,
            field: 0,
            // has_read: false,
            // output_pos: 0,
        }
    }
}

impl Parser {
    /// Create a new CSV reader with a default parser configuration.
    pub fn new() -> Parser {
        ParserBuilder::new().build()
    }

    /// Reset the parser such that it behaves as if it had never been used.
    ///
    /// This may be useful when reading CSV data in a random access pattern.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.nfa_state = NfaState::StartRecord;
        self.last_nfa_state = None;
        self.record = 1;
        // self.has_read = false;
    }

    pub fn state(&self) -> NfaState {
        self.nfa_state
    }

    /// Return the current record number as measured by the number of occurrences
    /// of `\n`.
    ///
    /// Line numbers starts at `1` and are reset when `reset` is called.
    #[allow(dead_code)]
    pub fn record(&self) -> u64 {
        self.record
    }

    /// Return the current field number as measured by the number of occurrences
    /// of `,` on the current line.
    ///
    /// field numbers starts at `0` and are reset when `reset` is called.
    pub fn field(&self) -> u64 {
        self.field
    }

    /// Return if the current position is in a field or not.
    pub fn is_in_field(&self) -> bool {
        self.nfa_state == NfaState::InField ||
            self.nfa_state == NfaState::InQuotedField ||
            self.nfa_state == NfaState::InEscapedQuote ||
            self.nfa_state == NfaState::InDoubleEscapedQuote
    }

    /// Set the record number.
    ///
    /// This is useful after a call to `reset` where the caller knows the
    /// line number from some additional context.
    #[allow(dead_code)]
    pub fn set_record(&mut self, record: u64) {
        self.record = record;
    }

    /// Compute the next NFA state given the current NFA state and the current
    /// input byte.
    ///
    /// This returns the next NFA state along with an NfaInputAction that
    /// indicates what should be done with the input byte (nothing for an epsilon
    /// transition, copied to a caller provided output buffer, or discarded).
    #[inline(always)]
    fn transition_nfa(
        &self,
        state: NfaState,
        c: char,
    ) -> (NfaState, NfaInputAction) {
        use self::NfaState::*;
        match state {
            // End => (End, NfaInputAction::Epsilon),
            StartRecord => {
                if self.term.equals(c) {
                    (StartRecord, NfaInputAction::Discard)
                } else if self.comment == Some(c) {
                    (InComment, NfaInputAction::Discard)
                } else {
                    (StartField, NfaInputAction::Epsilon)
                }
            }
            EndRecord => (StartRecord, NfaInputAction::Epsilon),
            StartField => {
                if self.quoting && self.quote == c {
                    (InQuotedField, NfaInputAction::Discard)
                } else if self.delimiter == c {
                    (EndFieldDelim, NfaInputAction::Discard)
                } else if self.term.equals(c) {
                    (EndFieldTerm, NfaInputAction::Epsilon)
                } else {
                    (InField, NfaInputAction::CopyToOutput)
                }
            }
            EndFieldDelim => (StartField, NfaInputAction::Epsilon),
            EndFieldTerm => (InRecordTerm, NfaInputAction::Epsilon),
            InField => {
                if self.delimiter == c {
                    (EndFieldDelim, NfaInputAction::Discard)
                } else if self.term.equals(c) {
                    (EndFieldTerm, NfaInputAction::Epsilon)
                } else {
                    (InField, NfaInputAction::CopyToOutput)
                }
            }
            InQuotedField => {
                if self.quoting && self.quote == c {
                    (InDoubleEscapedQuote, NfaInputAction::Discard)
                } else if self.quoting && self.escape == Some(c) {
                    (InEscapedQuote, NfaInputAction::Discard)
                } else {
                    (InQuotedField, NfaInputAction::CopyToOutput)
                }
            }
            InEscapedQuote => (InQuotedField, NfaInputAction::CopyToOutput),
            InDoubleEscapedQuote => {
                if self.quoting && self.double_quote && self.quote == c {
                    (InQuotedField, NfaInputAction::CopyToOutput)
                } else if self.delimiter == c {
                    (EndFieldDelim, NfaInputAction::Discard)
                } else if self.term.equals(c) {
                    (EndFieldTerm, NfaInputAction::Epsilon)
                } else {
                    (InField, NfaInputAction::CopyToOutput)
                }
            }
            InComment => {
                if '\n' == c {
                    (StartRecord, NfaInputAction::Discard)
                } else {
                    (InComment, NfaInputAction::Discard)
                }
            }
            InRecordTerm => {
                if self.term.is_crlf() && '\r' == c {
                    (CRLF, NfaInputAction::Discard)
                } else {
                    (EndRecord, NfaInputAction::Discard)
                }
            }
            CRLF => {
                if '\n' == c {
                    (StartRecord, NfaInputAction::Discard)
                } else {
                    (StartRecord, NfaInputAction::Epsilon)
                }
            }
        }
    }

    /// Interpret single char as a part of CSV
    ///
    /// This method is used to interpret single char as a part of CSV.
    /// It returns None if the char is a part of CSV and error otherwise.
    /// Parser is updated its own state according to the char.
    pub fn interpret(&mut self, c: char) -> Option<std::io::Error> {
        loop {
            let (state, action) = self.transition_nfa(self.nfa_state, c);
            self.nfa_state = state;
            // println!("c = {:?}, record = {}, state = {:?}, last = {:?}", c, self.record, self.nfa_state, self.last_nfa_state);
            match self.nfa_state {
                NfaState::StartRecord => {
                    self.field = 0;
                    match self.last_nfa_state {
                        Some(NfaState::EndRecord) => {
                            self.record += 1;
                        }
                        _ => {}
                    }
                }
                NfaState::StartField => {
                    self.field += 1;
                    // Forcefully set record 1.
                    // If the input starts with a field,
                    // StartRecord is not emitted.
                    if self.record == 0 {
                        self.record = 1
                    }
                }
                _ => {}
            }
            self.last_nfa_state = Some(self.nfa_state.clone());
            match action {
                NfaInputAction::Epsilon => {},
                _ => {
                    break;
                }
            }
        }
        None
    }
}

/// Builds a CSV reader with various configuration knobs.
///
/// This builder can be used to tweak the field delimiter, record terminator
/// and more for parsing CSV. Once a CSV `Reader` is built, its configuration
/// cannot be changed.
#[derive(Debug, Default)]
pub struct ParserBuilder {
    par: Parser,
}

impl ParserBuilder {
    /// Create a new builder.
    pub fn new() -> ParserBuilder {
        ParserBuilder::default()
    }

    /// Build a CSV parser from this configuration.
    pub fn build(&self) -> Parser {
        let par = self.par.clone();
        // rdr.build_dfa();
        par
    }

/*
    /// The field delimiter to use when parsing CSV.
    ///
    /// The default is `b','`.
    pub fn delimiter(&mut self, delimiter: char) -> &mut ParserBuilder {
        self.par.delimiter = delimiter;
        self
    }

    /// The record terminator to use when parsing CSV.
    ///
    /// A record terminator can be any single byte. The default is a special
    /// value, `Terminator::CRLF`, which treats any occurrence of `\r`, `\n`
    /// or `\r\n` as a single record terminator.
    pub fn terminator(&mut self, term: Terminator) -> &mut ParserBuilder {
        self.par.term = term;
        self
    }

    /// The quote character to use when parsing CSV.
    ///
    /// The default is `b'"'`.
    pub fn quote(&mut self, quote: char) -> &mut ParserBuilder {
        self.par.quote = quote;
        self
    }

    /// The escape character to use when parsing CSV.
    ///
    /// In some variants of CSV, quotes are escaped using a special escape
    /// character like `\` (instead of escaping quotes by doubling them).
    ///
    /// By default, recognizing these idiosyncratic escapes is disabled.
    pub fn escape(&mut self, escape: Option<char>) -> &mut ParserBuilder {
        self.par.escape = escape;
        self
    }

    /// Enable double quote escapes.
    ///
    /// This is enabled by default, but it may be disabled. When disabled,
    /// doubled quotes are not interpreted as escapes.
    pub fn double_quote(&mut self, yes: bool) -> &mut ParserBuilder {
        self.par.double_quote = yes;
        self
    }

    /// Enable or disable quoting.
    ///
    /// This is enabled by default, but it may be disabled. When disabled,
    /// quotes are not treated specially.
    pub fn quoting(&mut self, yes: bool) -> &mut ParserBuilder {
        self.par.quoting = yes;
        self
    }

    /// The comment character to use when parsing CSV.
    ///
    /// If the start of a record begins with the byte given here, then that
    /// line is ignored by the CSV parser.
    ///
    /// This is disabled by default.
    pub fn comment(&mut self, comment: Option<char>) -> &mut ParserBuilder {
        self.par.comment = comment;
        self
    }

    /// A convenience method for specifying a configuration to read ASCII
    /// delimited text.
    ///
    /// This sets the delimiter and record terminator to the ASCII unit
    /// separator (`\x1F`) and record separator (`\x1E`), respectively.
    pub fn ascii(&mut self) -> &mut ParserBuilder {
        self.delimiter('\x1F').terminator(Terminator::Any('\x1E'))
    }
*/
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_csv() {
        let data = "
foo,bar,baz
a,b,c
e,ff,ggg
あい,うえ,お
xxx,yyy,zzz
";
        let mut parser = Parser::new();
        let bytes = data.chars();
        // Check each one byte
        for b in bytes {
            let res = parser.interpret(b);
            println!("{:?} record = {}, {:?}", b, parser.record(), parser.nfa_state);
            match res {
                Some(err) => {
                    println!("Error: {}", err);
                }
                None => {}
            }
        }
        assert_eq!(5, parser.record());
    }

    #[test]
    fn test_parse_csv_utf8() {
        let data = "いち,に,さん
１rec,\"あいう
えお\",かきく
２rec,\"さしす
せそ\",\"たちつ
てと\"
";
        let mut parser = Parser::new();
        let c = data.chars().collect::<Vec<char>>();
        // Check each one byte
        let mut i = 0;
        parser.interpret(c[i]); assert_eq!('い', c[i]); assert_eq!(1, parser.record()); assert_eq!(1, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('ち', c[i]); assert_eq!(1, parser.record()); assert_eq!(1, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!(',' , c[i]); assert_eq!(1, parser.record()); assert_eq!(1, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('に', c[i]); assert_eq!(1, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!(',' , c[i]); assert_eq!(1, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('さ', c[i]); assert_eq!(1, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('ん', c[i]); assert_eq!(1, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('\n', c[i]); assert_eq!(1, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('１', c[i]); assert_eq!(2, parser.record()); assert_eq!(1, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('r' , c[i]); assert_eq!(2, parser.record()); assert_eq!(1, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('e' , c[i]); assert_eq!(2, parser.record()); assert_eq!(1, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('c' , c[i]); assert_eq!(2, parser.record()); assert_eq!(1, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!(',' , c[i]); assert_eq!(2, parser.record()); assert_eq!(1, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('"' , c[i]); assert_eq!(2, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('あ', c[i]); assert_eq!(2, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('い', c[i]); assert_eq!(2, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('う', c[i]); assert_eq!(2, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('\n', c[i]); assert_eq!(2, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('え', c[i]); assert_eq!(2, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('お', c[i]); assert_eq!(2, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('"' , c[i]); assert_eq!(2, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!(',' , c[i]); assert_eq!(2, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('か', c[i]); assert_eq!(2, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('き', c[i]); assert_eq!(2, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('く', c[i]); assert_eq!(2, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('\n', c[i]); assert_eq!(2, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('２', c[i]); assert_eq!(3, parser.record()); assert_eq!(1, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('r' , c[i]); assert_eq!(3, parser.record()); assert_eq!(1, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('e' , c[i]); assert_eq!(3, parser.record()); assert_eq!(1, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('c' , c[i]); assert_eq!(3, parser.record()); assert_eq!(1, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!(',' , c[i]); assert_eq!(3, parser.record()); assert_eq!(1, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('"' , c[i]); assert_eq!(3, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('さ', c[i]); assert_eq!(3, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('し', c[i]); assert_eq!(3, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('す', c[i]); assert_eq!(3, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('\n', c[i]); assert_eq!(3, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('せ', c[i]); assert_eq!(3, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('そ', c[i]); assert_eq!(3, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('"' , c[i]); assert_eq!(3, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!(',' , c[i]); assert_eq!(3, parser.record()); assert_eq!(2, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('"' , c[i]); assert_eq!(3, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('た', c[i]); assert_eq!(3, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('ち', c[i]); assert_eq!(3, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('つ', c[i]); assert_eq!(3, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('\n', c[i]); assert_eq!(3, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('て', c[i]); assert_eq!(3, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('と', c[i]); assert_eq!(3, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('"' , c[i]); assert_eq!(3, parser.record()); assert_eq!(3, parser.field()); i += 1;
        parser.interpret(c[i]); assert_eq!('\n', c[i]); assert_eq!(3, parser.record()); assert_eq!(3, parser.field());
    }
}


