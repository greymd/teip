// Many part of this module is based on https://github.com/BurntSushi/rust-csv/tree/master/csv-core
// which is dual-licensed under MIT and Unlicense as of 2023-02-21

use super::terminator::Terminator;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum NfaState {
    // These states aren't used in the DFA, so we
    // assign them meaningless numbers.
    EndFieldTerm = 200,
    InRecordTerm = 201,
    End = 202,

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
    /// The delimiter that separates fields.
    delimiter: u8,
    /// The terminator that separates records.
    term: Terminator,
    /// The quotation byte.
    quote: u8,
    /// Whether to recognize escaped quotes.
    escape: Option<u8>,
    /// Whether to recognized doubled quotes.
    double_quote: bool,
    /// If enabled, lines beginning with this byte are ignored.
    comment: Option<u8>,
    /// If enabled (the default), then quotes are respected. When disabled,
    /// quotes are not treated specially.
    quoting: bool,
    /// The current line.
    line: u64,
    /// The current field.
    field: u64,
    /// Whether this parser has ever read anything.
    has_read: bool,
    /// The current position in the output buffer when reading a record.
    output_pos: usize,
}


impl Default for Parser {
    fn default() -> Parser {
        Parser {
            nfa_state: NfaState::StartRecord,
            delimiter: b',',
            term: Terminator::default(),
            quote: b'"',
            escape: None,
            double_quote: true,
            comment: None,
            quoting: true,
            line: 0,
            field: 0,
            has_read: false,
            output_pos: 0,
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
    pub fn reset(&mut self) {
        self.nfa_state = NfaState::StartRecord;
        self.line = 1;
        self.has_read = false;
    }

    /// Return the current line number as measured by the number of occurrences
    /// of `\n`.
    ///
    /// Line numbers starts at `1` and are reset when `reset` is called.
    pub fn line(&self) -> u64 {
        self.line
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
        self.nfa_state == NfaState::InField
    }

    /// Set the line number.
    ///
    /// This is useful after a call to `reset` where the caller knows the
    /// line number from some additional context.
    pub fn set_line(&mut self, line: u64) {
        self.line = line;
    }

    /// Strip off a possible UTF-8 BOM at the start of a file. Quick note that
    /// this method will fail to strip off the BOM if only part of the BOM is
    /// buffered. Hopefully that won't happen very often.
    fn strip_utf8_bom<'a>(&self, input: &'a [u8]) -> (&'a [u8], usize) {
        let (input, nin) = if {
            !self.has_read
                && input.len() >= 3
                && &input[0..3] == b"\xef\xbb\xbf"
        } {
            (&input[3..], 3)
        } else {
            (input, 0)
        };
        (input, nin)
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
        c: u8,
    ) -> (NfaState, NfaInputAction) {
        use self::NfaState::*;
        match state {
            End => (End, NfaInputAction::Epsilon),
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
                if b'\n' == c {
                    (StartRecord, NfaInputAction::Discard)
                } else {
                    (InComment, NfaInputAction::Discard)
                }
            }
            InRecordTerm => {
                if self.term.is_crlf() && b'\r' == c {
                    (CRLF, NfaInputAction::Discard)
                } else {
                    (EndRecord, NfaInputAction::Discard)
                }
            }
            CRLF => {
                if b'\n' == c {
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
    pub fn interpret(&mut self, c: u8) -> Option<std::io::Error> {
        loop {
            let (state, action) = self.transition_nfa(self.nfa_state, c);
            self.nfa_state = state;
            match self.nfa_state {
                NfaState::StartRecord => {
                    self.line += 1;
                    self.field = 0;
                }
                NfaState::StartField => {
                    self.field += 1;
                }
                _ => {}
            }
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
        let mut par = self.par.clone();
        // rdr.build_dfa();
        par
    }

    /// The field delimiter to use when parsing CSV.
    ///
    /// The default is `b','`.
    pub fn delimiter(&mut self, delimiter: u8) -> &mut ParserBuilder {
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
    pub fn quote(&mut self, quote: u8) -> &mut ParserBuilder {
        self.par.quote = quote;
        self
    }

    /// The escape character to use when parsing CSV.
    ///
    /// In some variants of CSV, quotes are escaped using a special escape
    /// character like `\` (instead of escaping quotes by doubling them).
    ///
    /// By default, recognizing these idiosyncratic escapes is disabled.
    pub fn escape(&mut self, escape: Option<u8>) -> &mut ParserBuilder {
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
    pub fn comment(&mut self, comment: Option<u8>) -> &mut ParserBuilder {
        self.par.comment = comment;
        self
    }

    /// A convenience method for specifying a configuration to read ASCII
    /// delimited text.
    ///
    /// This sets the delimiter and record terminator to the ASCII unit
    /// separator (`\x1F`) and record separator (`\x1E`), respectively.
    pub fn ascii(&mut self) -> &mut ParserBuilder {
        self.delimiter(b'\x1F').terminator(Terminator::Any(b'\x1E'))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    fn test_parse_csv() {
        let data = "
foo,bar,baz
a,b,c
e,ff,ggg
あい,うえ,お
xxx,yyy,zzz
";
        let mut parser = Parser::new();
        let bytes = data.as_bytes();
        // Check each one byte
        for b in bytes {
            let res = parser.interpret(*b);
            match res {
                Some(err) => {
                    println!("Error: {}", err);
                }
                None => {}
            }
        }
        assert_eq!(3, parser.field);
        assert_eq!(5, parser.line);
    }
}
