/// A record terminator.
///
/// Use this to specify the record terminator while parsing CSV. The default is
/// CRLF, which treats `\r`, `\n` or `\r\n` as a single record terminator.
#[derive(Clone, Copy, Debug)]
pub enum Terminator {
    /// Parses `\r`, `\n` or `\r\n` as a single record terminator.
    CRLF,
    /// Parses the byte given as a record terminator.
    Any(char),
    /// Hints that destructuring should not be exhaustive.
    ///
    /// This enum may grow additional variants, so this makes sure clients
    /// don't count on exhaustive matching. (Otherwise, adding a new variant
    /// could break existing code.)
    #[doc(hidden)]
    __Nonexhaustive,
}

impl Terminator {
    /// Checks whether the terminator is set to CRLF.
    pub fn is_crlf(&self) -> bool {
        match *self {
            Terminator::CRLF => true,
            Terminator::Any(_) => false,
            _ => unreachable!(),
        }
    }

    pub fn equals(&self, other: char) -> bool {
        match *self {
            Terminator::CRLF => other == '\r' || other == '\n',
            Terminator::Any(b) => other == b,
            _ => unreachable!(),
        }
    }
}

impl Default for Terminator {
    fn default() -> Terminator {
        Terminator::CRLF
    }
}


