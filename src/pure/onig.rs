pub type Regex = i64;
use super::super::{errors, PipeIntercepter};
use super::super::CMD;

pub fn new_regex() -> Regex {
    1
}

pub fn new_option_multiline_regex(_s: &str) -> Regex {
    1
}

pub fn new_option_none_regex(_s: &str) -> Regex {
    1
}

/// Handles regex onig ( -g -G )
pub fn regex_onig_proc(
    _ch: &mut PipeIntercepter,
    _line: &Vec<u8>,
    _re: &Regex,
    _invert: bool,
) -> Result<(), errors::TokenSendError> {
    eprintln!("{}: This build is not enabled 'oniguruma'", CMD);
    Ok(())
}

pub fn regex_onig_line_proc(
    _ch: &mut PipeIntercepter,
    _re: &Regex,
    _invert: bool,
    _line_end: u8,
) -> Result<(), errors::TokenSendError> {
    eprintln!("{}: This build is not enabled 'oniguruma'", CMD);
    Ok(())
}
