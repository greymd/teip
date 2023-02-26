// Many part of this module is based on https://github.com/BurntSushi/rust-csv/tree/master/csv-core
// which is dual-licensed under MIT and Unlicense as of 2023-02-21

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NfaState {
    // These states aren't used in the DFA, so we
    // assign them meaningless numbers.
    EndFieldTerm = 200,
    InRecordTerm = 201,
    // All states below can be used in DFA.
    // assign them meaningful numbers for future use.
    StartRecord = 0,
    StartField = 1,
    InField = 2,
    InQuotedField = 3,
    InEscapedQuote = 4,
    InDoubleEscapedQuote = 5,
    InComment = 6,
    EndFieldDelim = 7,
    EndRecord = 8,
    CRLF = 9,
}

/// What should be done with input bytes during an NFA transition
#[derive(Clone, Debug, Eq, PartialEq)]
enum NfaType {
    // Epsilon state of the NFA
    // Keep interpretation of the input byte until the next state
    Epsilon,
    // Actual information that the CSV delivers
    Content,
    // Meta information that constructs the CSV.
    // (for example, seeing a field
    // delimiter is information of CSV but
    // should not categorized as a actual content)
    Meta,
}

#[derive(Clone, Debug)]
pub struct Parser {
    /// The current NFA state
    nfa_state: NfaState,
    /// The last NFA state
    last_nfa_state: Option<NfaState>,
    /// The delimiter that separates fields.
    delimiter: char,
    /// The delimiter that separates records.
    lf: char, // line field
    cr: char, // carriage return
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
    /// The number of current record.
    record: u64,
    /// The number of current field.
    field: u64,
}


impl Default for Parser {
    fn default() -> Parser {
        Parser {
            nfa_state: NfaState::StartRecord,
            last_nfa_state: None,
            delimiter: ',',
            lf: '\n',
            cr: '\r',
            quote: '"',
            escape: None,
            double_quote: true,
            comment: None,
            quoting: true,
            record: 0,
            field: 0,
        }
    }
}

impl Parser {
    /// Create a new CSV reader with a default parser configuration.
    pub fn new() -> Parser {
        ParserBuilder::new().build()
    }

    #[allow(dead_code)]
    pub fn state(&self) -> NfaState {
        self.nfa_state
    }

    /// Return the current record number as measured by the number of occurrences
    /// of `\n`.
    #[allow(dead_code)]
    pub fn record(&self) -> u64 {
        self.record
    }

    /// Return the current field number as measured by the number of occurrences
    /// of delimiter on the current line.
    pub fn field(&self) -> u64 {
        self.field
    }

    /// Return if the current position is on a field or not.
    pub fn is_in_field(&self) -> bool {
        self.nfa_state == NfaState::InField ||
            self.nfa_state == NfaState::InQuotedField ||
            self.nfa_state == NfaState::InEscapedQuote ||
            self.nfa_state == NfaState::InDoubleEscapedQuote
    }

    fn is_term(&self, b: char) -> bool {
        self.cr == b || self.lf == b
    }

    /// Compute the next NFA state given the current NFA state and the current
    /// input byte.
    ///
    /// This returns the next NFA state along with an NfaType that
    /// indicates what should be done with the input byte.
    #[inline(always)]
    fn transition_nfa(
        &self,
        state: NfaState,
        c: char,
    ) -> (NfaState, NfaType) {
        use self::NfaState::*;
        match state {
            StartRecord => {
                if self.is_term(c) {
                    (StartRecord, NfaType::Meta)
                } else if self.comment == Some(c) {
                    (InComment, NfaType::Meta)
                } else {
                    (StartField, NfaType::Epsilon)
                }
            }
            EndRecord => (StartRecord, NfaType::Epsilon),
            StartField => {
                if self.quoting && self.quote == c {
                    (InQuotedField, NfaType::Meta)
                } else if self.delimiter == c {
                    (EndFieldDelim, NfaType::Meta)
                } else if self.is_term(c) {
                    (EndFieldTerm, NfaType::Epsilon)
                } else {
                    (InField, NfaType::Content)
                }
            }
            EndFieldDelim => (StartField, NfaType::Epsilon),
            EndFieldTerm => (InRecordTerm, NfaType::Epsilon),
            InField => {
                if self.delimiter == c {
                    (EndFieldDelim, NfaType::Meta)
                } else if self.is_term(c) {
                    (EndFieldTerm, NfaType::Epsilon)
                } else {
                    (InField, NfaType::Content)
                }
            }
            InQuotedField => {
                if self.quoting && self.quote == c {
                    (InDoubleEscapedQuote, NfaType::Meta)
                } else if self.quoting && self.escape == Some(c) {
                    (InEscapedQuote, NfaType::Meta)
                } else {
                    (InQuotedField, NfaType::Content)
                }
            }
            InEscapedQuote => (InQuotedField, NfaType::Content),
            InDoubleEscapedQuote => {
                if self.quoting && self.double_quote && self.quote == c {
                    (InQuotedField, NfaType::Content)
                } else if self.delimiter == c {
                    (EndFieldDelim, NfaType::Meta)
                } else if self.is_term(c) {
                    (EndFieldTerm, NfaType::Epsilon)
                } else {
                    (InField, NfaType::Content)
                }
            }
            InComment => {
                if self.lf == c {
                    (StartRecord, NfaType::Meta)
                } else {
                    (InComment, NfaType::Meta)
                }
            }
            InRecordTerm => {
                if self.cr == c {
                    (CRLF, NfaType::Meta)
                } else {
                    (EndRecord, NfaType::Meta)
                }
            }
            CRLF => {
                if self.lf == c {
                    (StartRecord, NfaType::Meta)
                } else {
                    (StartRecord, NfaType::Epsilon)
                }
            }
        }
    }

    /// Interpret single char as a part of CSV
    ///
    /// This method is used to interpret single char as a part of CSV.
    /// It returns None if the char is a part of CSV and returns error otherwise.
    /// Parser is updated its own state according to the input.
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
                NfaType::Epsilon => {},
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
