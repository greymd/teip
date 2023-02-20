use super::pipeintercepter::PipeIntercepter;
use super::spawnutils;
use super::list;
use super::{errors,errors::*};
use super::stringutils;
use regex::Regex;
use super::DEFAULT_CAP;
use std::io::{self, BufRead};

/// Bypassing particular lines based on given list ( -l )
pub fn line_line_proc(
    ch: &mut PipeIntercepter,
    ranges: &Vec<list::ranges::Range>,
    line_end: u8,
) -> Result<(), errors::ChunkSendError> {
    let mut i: usize = 0;
    let mut ri: usize = 0;
    let stdin = io::stdin();
    loop {
        let mut buf = Vec::with_capacity(DEFAULT_CAP);
        match stdin.lock().read_until(line_end, &mut buf) {
            Ok(n) => {
                let eol = stringutils::trim_eol(&mut buf);
                let line = String::from_utf8_lossy(&buf).to_string();
                if n == 0 {
                    ch.send_eof()?;
                    break;
                }
                if ranges[ri].high < (i + 1) && (ri + 1) < ranges.len() {
                    ri += 1;
                }
                if ranges[ri].low <= (i + 1) && (i + 1) <= ranges[ri].high {
                    ch.send_byps(line.to_string())?;
                } else {
                    ch.send_keep(line.to_string())?;
                }
                ch.send_keep(eol)?;
            }
            Err(e) => msg_error(&e.to_string()),
        }
        i += 1;
    }
    Ok(())
}

/// Bypassing particular lines based on Regular Expression ( -g )
pub fn regex_line_proc(
    ch: &mut PipeIntercepter,
    re: &Regex,
    invert: bool,
    line_end: u8,
) -> Result<(), errors::ChunkSendError> {
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
                if re.is_match(&line) {
                    if invert {
                        ch.send_keep(line.to_string())?;
                    } else {
                        ch.send_byps(line.to_string())?;
                    }
                } else {
                    if invert {
                        ch.send_byps(line.to_string())?;
                    } else {
                        ch.send_keep(line.to_string())?;
                    }
                }
                ch.send_keep(eol)?;
            }
            Err(e) => msg_error(&e.to_string()),
        }
    }
    Ok(())
}

/// Bypassing particular strings based on Regular Expression ( -o -g )
pub fn regex_proc(
    ch: &mut PipeIntercepter,
    line: &Vec<u8>,
    re: &Regex,
    invert: bool,
) -> Result<(), errors::ChunkSendError> {
    let line = String::from_utf8_lossy(&line).to_string();
    let mut left_index = 0;
    let mut right_index;
    for cap in re.find_iter(&line) {
        right_index = cap.start();
        let unmatched = &line[left_index..right_index];
        let matched = &line[cap.start()..cap.end()];
        // Ignore empty string.
        // Regex "*" matches empty, but , in most situations,
        // handling empty string is not helpful for users.
        if !unmatched.is_empty() {
            if !invert {
                ch.send_keep(unmatched.to_string())?;
            } else {
                ch.send_byps(unmatched.to_string())?;
            }
        }
        if !invert {
            ch.send_byps(matched.to_string())?;
        } else {
            ch.send_keep(matched.to_string())?;
        }
        left_index = cap.end();
    }
    if left_index < line.len() {
        let unmatched = &line[left_index..line.len()];
        if !invert {
            ch.send_keep(unmatched.to_string())?;
        } else {
            ch.send_byps(unmatched.to_string())?;
        }
    }
    Ok(())
}

/// Bypassing character range ( -c )
pub fn char_proc(
    ch: &mut PipeIntercepter,
    line: &Vec<u8>,
    ranges: &Vec<list::ranges::Range>,
) -> Result<(), errors::ChunkSendError> {
    let line = String::from_utf8_lossy(&line).to_string();
    let cs = line.chars();
    let mut str_in = String::new();
    let mut str_out = String::new();
    let mut ri = 0;
    let mut is_in;
    let mut last_is_in = false;
    // Merge consequent characters' range to execute commands as few times as possible.
    for (i, c) in cs.enumerate() {
        if ranges[ri].high < (i + 1) && (ri + 1) < ranges.len() {
            ri += 1;
        }
        if ranges[ri].low <= (i + 1) && (i + 1) <= ranges[ri].high {
            is_in = true;
            str_in.push(c);
        } else {
            is_in = false;
            str_out.push(c);
        }
        if is_in && !last_is_in {
            ch.send_keep(str_out.to_string())?;
            str_out.clear();
        } else if !is_in && last_is_in {
            ch.send_byps(str_in.to_string())?;
            str_in.clear();
        }
        last_is_in = is_in;
    }
    if last_is_in && !str_in.is_empty() {
        ch.send_byps(str_in)?;
    } else {
        ch.send_keep(str_out)?;
    }
    Ok(())
}

/// Bypassing white space separation ( -f )
pub fn field_regex_proc(
    ch: &mut PipeIntercepter,
    line: &Vec<u8>,
    re: &Regex,
    ranges: &Vec<list::ranges::Range>,
) -> Result<(), errors::ChunkSendError> {
    let line = String::from_utf8_lossy(&line).to_string();
    let mut i = 1; // current field index
    let mut ri = 0;
    let mut left_index = 0;
    let mut right_index;
    for cap in re.find_iter(&line) {
        right_index = cap.start();
        let field = &line[left_index..right_index]; // This can be empty string
        let spaces = &line[cap.start()..cap.end()];
        left_index = cap.end();
        if ranges[ri].high < i && (ri + 1) < ranges.len() {
            ri += 1;
        }
        if ranges[ri].low <= i && i <= ranges[ri].high {
            ch.send_byps(field.to_string())?;
        } else {
            ch.send_keep(field.to_string())?;
        }
        ch.send_keep(spaces.to_string())?;
        i += 1;
    }
    // If line ends with delimiter, empty fields must be handled.
    if left_index <= line.len() {
        if ranges[ri].high < i && (ri + 1) < ranges.len() {
            ri += 1;
        }
        // filed is empty if line ends with delimiter
        let field = &line[left_index..line.len()];
        if ranges[ri].low <= i && i <= ranges[ri].high {
            ch.send_byps(field.to_string())?;
        } else {
            ch.send_keep(field.to_string())?;
        }
    }
    Ok(())
}

/// Bypassing field separation ( -f -d )
pub fn field_proc(
    ch: &mut PipeIntercepter,
    line: &Vec<u8>,
    delim: &str,
    ranges: &Vec<list::ranges::Range>,
) -> Result<(), errors::ChunkSendError> {
    let line = String::from_utf8_lossy(&line).to_string();
    let chunks = line.split(delim);
    let mut ri = 0;
    for (i, chunk) in chunks.enumerate() {
        if i > 0 {
            ch.send_keep(delim.to_string())?;
        }
        if ranges[ri].high < (i + 1) && (ri + 1) < ranges.len() {
            ri += 1;
        }
        if ranges[ri].low <= (i + 1) && (i + 1) <= ranges[ri].high {
            // Should empty filed sent as empty string ? Discussion is needed.
            // But author(@greymd) believes empty string is good to be sent.
            // Because teip can be used as simple CSV file editor if it is allowed!
            // ```
            // $ printf ',,,\n,,,\n,,,\n' | teip -d, -f1- -- seq 12
            // 1,2,3,4
            // 5,6,7,8
            // 9,10,11,12
            // ```
            ch.send_byps(chunk.to_string())?;
        } else {
            ch.send_keep(chunk.to_string())?;
        }
    }
    Ok(())
}

/// External execution for match offloading ( -e )
///  Example:
///  ``````````````````````````````````````````````````````````````````
///  $ echo -e "AAA\nBBB\nCCC\nDDD\nEEE\n" | teip -e 'grep -n "[ACE]"'
///  [AAA]
///  BBB
///  [CCC]
///  DDD
///  [EEE]
///  ``````````````````````````````````````````````````````````````````
///
///                          [ stdin ] "AAA\nBBB\nCCC\n..."
///                              │
///                              │
///                         ┌────▼──────────┐
///                         │  _tee_thread  ├──────┐
///                         └────┬──────────┘      │
///                              │                 │
///                          (rx_stdin1)           │
///                          "AAA\nBBB\nCCC\n..."  │
///                              │                 │
///                              │           (rx_stdin2)
///                              │            "AAA\nBBB\nCCC\n..."
///                         ┌────▼──────────┐      │
/// (exoffload_pipeline)────►   _ex_thread  │      │
/// "grep -n '[ACE]'"       └────┬──────────┘      │
///                              │                 │
///                         (rx_messy_numbers)     │
///                         "1:AAA\n3:CCC\n..."    │
///                              │                 │
///                         ┌────▼──────────┐      │
///                         │  _num_thread  │      │
///                         └────┬──────────┘      │
///                              │                 │
///                            (rx_numbers)        │
///                            1,│3, 5 ...as u64   │
///                              │                 │
///                         ┌────┴─────────────────┴──────────┐  ┌─────────────────────┐
///                         │ Main thread > exoffload_proc    │  │ PipeIntercepter     │
///                         ├────┬─────────────────┬──────────┤  ├─────────────────────┤
///                         │    │                 │          │  │                     │
///                         │    │              ┌──▼────────┐ │  │                     │
///                         │    │              │ NR LINE   │ │  │                     │
///                         │  ┌─▼─┐            │           │ │  │    ┌───────────┐    │
///                         │  │ 1 │  ==MATCH== │  1 "AAA"──┼─┼──┼────┤►send_byps │    │
///                         │  │   │            │           │ │  │    │───────────┤    │
///                         │  │   │            │  2 "BBB"──┼─┼──┼────┤►send_keep │    │
///                         │  │   │            │           │ │  │    │───────────┤ ───┼──► (See pipeintercepter.rs)
///                         │  │ 3 │  ==MATCH== │  3 "CCC"──┼─┼──┼────┤►send_byps │    │
///                         │  │   │            │           │ │  │    │───────────┤    │
///                         │  │   │            │  4 "DDD"──┼─┼──┼────┤►send_keep │    │
///                         │  │   │            │           │ │  │    │───────────┤    │
///                         │  │ 5 │  ==MATCH== │  5 "EEE"──┼─┼──┼────┤►send_byps │    │
///                         │  │   │            │           │ │  │    └───────────┘    │
///                         │  └───┘            └───────────┘ │  │                     │
///                         └─────────────────────────────────┘  └─────────────────────┘
pub fn exoffload_proc(
    ch: &mut PipeIntercepter,
    exoffload_pipeline: &str,
    invert: bool,
    line_end: u8,
) -> Result<(), errors::ChunkSendError> {
    let stdin = io::stdin();
    let (rx_stdin1, rx_stdin2, _tee_thread) = spawnutils::tee(stdin, line_end)
            .unwrap_or_else(|e| error_exit(&e.to_string()));
    let (rx_messy_numbers, _ex_thread) = spawnutils::exec_pipeline_mpsc_input(exoffload_pipeline, rx_stdin1)
            .unwrap_or_else(|e| error_exit(&e.to_string()));
    let (rx_numbers, _num_thread) = spawnutils::clean_numbers(rx_messy_numbers, line_end);
    let mut nr: u64 = 0;     // number of read
    let mut pos: u64 = 0;    // position of printable numbers
    let mut last_pos: u64 = pos;
    let mut expect_new_numbers: bool = true;
    loop {
        nr += 1;
        // Load line from stdin
        let mut buf = match rx_stdin2.recv() {
            Ok(b) => b,
            Err(_) => {
                ch.send_eof()?;
                break;
            },
        };
        let eol = stringutils::trim_eol(&mut buf);
        let line = String::from_utf8_lossy(&buf).to_string();
        // Try to detect printable line numbers which is bigger than current read line
        while expect_new_numbers && pos < nr {
            pos = match rx_numbers.recv() {
                Ok(n) => n,
                Err(_) => {
                    // Once queue got disconnected, new numbers is no longer expected.
                    expect_new_numbers = false;
                    break;
                },
            };
            if pos < last_pos {
                msg_error(format!("WARN: pipeline must print numbers in ascending order: order {} -> {} found", last_pos, pos).as_ref());
            }
            last_pos = pos;
        }
        if pos == nr {
            if invert {
                ch.send_keep(line.to_string())?;
            } else {
                ch.send_byps(line.to_string())?;
            }
        } else {
            if invert {
                ch.send_byps(line.to_string())?;
            } else {
                ch.send_keep(line.to_string())?;
            }
        }
        ch.send_keep(eol)?;
    }
    Ok(())
}
