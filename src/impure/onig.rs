use std::io::{self, BufRead};

use onig;
pub type Regex = onig::Regex;
pub type RegexOptions = onig::RegexOptions;
pub type Syntax = onig::Syntax;

use super::super::{error_exit, errors, msg_error, stringutils, PipeIntercepter, DEFAULT_CAP};

pub fn new_regex() -> Regex {
    Regex::new("").unwrap()
}

pub fn new_option_multiline_regex(s: &str) -> Regex {
    Regex::with_options(s, RegexOptions::REGEX_OPTION_MULTILINE, Syntax::default())
        .unwrap_or_else(|e| error_exit(&e.to_string()))
}

pub fn new_option_none_regex(s: &str) -> Regex {
    Regex::with_options(s, RegexOptions::REGEX_OPTION_NONE, Syntax::default())
        .unwrap_or_else(|e| error_exit(&e.to_string()))
}

/// Bypassing multiple strings in a line based on Oniguruma Regular Expression ( -g -G -o )
pub fn regex_onig_proc(
    ch: &mut PipeIntercepter,
    line: &Vec<u8>,
    re: &Regex,
    invert: bool,
) -> Result<(), errors::TokenSendError> {
    let line = String::from_utf8_lossy(&line).to_string();
    let mut left_index = 0;
    let mut right_index;
    for cap in re.find_iter(&line) {
        right_index = cap.0;
        let unmatched = &line[left_index..right_index];
        let matched = &line[cap.0..cap.1];
        // Ignore empty string.
        // Regex "*" matches empty, but , in most situations,
        // handling empty string is not helpful for users.
        if !unmatched.is_empty() {
            if !invert {
                ch.send_asis(unmatched.to_string())?;
            } else {
                ch.send_byps(unmatched.to_string())?;
            }
        }
        if !invert {
            ch.send_byps(matched.to_string())?;
        } else {
            ch.send_asis(matched.to_string())?;
        }
        left_index = cap.1;
    }
    if left_index < line.len() {
        let unmatched = &line[left_index..line.len()];
        if !invert {
            ch.send_asis(unmatched.to_string())?;
        } else {
            ch.send_byps(unmatched.to_string())?;
        }
    }
    Ok(())
}

/// Bypassing particular lines based on Oniguruma Regular Expression ( -g -G )
pub fn regex_onig_line_proc(
    ch: &mut PipeIntercepter,
    re: &Regex,
    invert: bool,
    line_end: u8,
) -> Result<(), errors::TokenSendError> {
    let stdin = io::stdin();
    loop {
        let mut buf = Vec::with_capacity(DEFAULT_CAP);
        match stdin.lock().read_until(line_end, &mut buf) {
            Ok(n) => {
                let eol = stringutils::trim_eol(&mut buf);
                if n == 0 {
                    ch.send_eof()?;
                    break;
                }
                let line = String::from_utf8_lossy(&buf).to_string();
                match re.find(&line) {
                    Some(_) => {
                        if invert {
                            ch.send_asis(line.to_string())?;
                        } else {
                            ch.send_byps(line.to_string())?;
                        }
                    }
                    None => {
                        if invert {
                            ch.send_byps(line.to_string())?;
                        } else {
                            ch.send_asis(line.to_string())?;
                        }
                    }
                };
                ch.send_asis(eol)?;
            }
            Err(e) => msg_error(&e.to_string()),
        }
    }
    Ok(())
}
