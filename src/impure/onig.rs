use std::io::{self, BufRead};

use onig::{self};
pub type Regex = onig::Regex;
pub type RegexOptions = onig::RegexOptions;
pub type Syntax = onig::Syntax;

use super::super::{errors, PipeIntercepter, DEFAULT_CAP, trim_eol, msg_error};

/// Handles regex onig ( -g -G )
pub fn regex_onig_proc(
    ch: &mut PipeIntercepter,
    line: &Vec<u8>,
    re: &onig::Regex,
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
                ch.send_msg(unmatched.to_string())?;
            } else {
                ch.send_pipe(unmatched.to_string())?;
            }
        }
        if !invert {
            ch.send_pipe(matched.to_string())?;
        } else {
            ch.send_msg(matched.to_string())?;
        }
        left_index = cap.1;
    }
    if left_index < line.len() {
        let unmatched = &line[left_index..line.len()];
        if !invert {
            ch.send_msg(unmatched.to_string())?;
        } else {
            ch.send_pipe(unmatched.to_string())?;
        }
    }
    Ok(())
}

pub fn regex_onig_line_proc(
    ch: &mut PipeIntercepter,
    re: &onig::Regex,
    invert: bool,
    line_end: u8,
) -> Result<(), errors::TokenSendError> {
    let stdin = io::stdin();
    loop {
        let mut buf = Vec::with_capacity(DEFAULT_CAP);
        match stdin.lock().read_until(line_end, &mut buf) {
            Ok(n) => {
                let eol = trim_eol(&mut buf);
                if n == 0 {
                    ch.send_eof()?;
                    break;
                }
                let line = String::from_utf8_lossy(&buf).to_string();
                match re.find(&line) {
                    Some(_) => {
                        if invert {
                            ch.send_msg(line.to_string())?;
                        } else {
                            ch.send_pipe(line.to_string())?;
                        }
                    },
                    None => {
                        if invert {
                            ch.send_pipe(line.to_string())?;
                        } else {
                            ch.send_msg(line.to_string())?;
                        }
                    }
                };
                ch.send_msg(eol)?;
            }
            Err(e) => msg_error(&e.to_string()),
        }
    }
    Ok(())
}

