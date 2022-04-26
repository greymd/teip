mod list {
    pub mod converter;
    pub mod ranges;
}
mod impure {
    #[cfg(feature = "oniguruma")]
    pub mod onig;
}
mod pure {
    #[cfg(not(feature = "oniguruma"))]
    pub mod onig;
}
mod token;
mod errors;
mod procspawn;
use errors::*;
mod pipeintercepter;
use pipeintercepter::PipeIntercepter;
mod stringutils;

#[macro_use]
extern crate lazy_static;

use log::debug; // Enable with RUST_LOG=debug
use regex::Regex;
use std::env;
use std::io::{self, BufRead};
use structopt::StructOpt;

#[cfg(feature = "oniguruma")]
use impure::onig;

#[cfg(not(feature = "oniguruma"))]
use pure::onig;

const CMD: &'static str = env!("CARGO_PKG_NAME"); // "teip"
pub const DEFAULT_CAP: usize = 1024;

lazy_static! {
    static ref REGEX_WS: Regex = Regex::new("\\s+").unwrap();
    static ref DEFAULT_HIGHLIGHT: String = match env::var("TEIP_HIGHLIGHT") {
        Ok(v) => v,
        Err(_) => "\x1b[36m[\x1b[0m\x1b[01;31m{}\x1b[0m\x1b[36m]\x1b[0m".to_string(),
    };
    static ref HL: Vec<&'static str> = DEFAULT_HIGHLIGHT.split("{}").collect();
}

#[derive(StructOpt, Debug)]
#[structopt(
    about = "Bypassing a partial range of standard input to an arbitrary command",
    usage = "teip [OPTIONS] [FLAGS] [--] [<command>...]",
    help = "USAGE:
  teip -g <pattern> [-oGsvz] [--] [<command>...]
  teip -f <list> [-d <delimiter> | -D <pattern>] [-svz] [--] [<command>...]
  teip -c <list> [-svz] [--] [<command>...]
  teip -l <list> [-svz] [--] [<command>...]
  teip -e <string> [-svz] [--] [<command>...]

OPTIONS:
    -c <list>        Bypassing these characters
    -d <delimiter>   Use <delimiter> for field delimiter of -f
    -D <pattern>     Use regular expression <pattern> for field delimiter of -f
    -e <string>      Execute <string> on another process that will receive identical
                     standard input as the teip, and numbers given by the result
                     are used as line numbers for bypassing
    -l <list>        Bypassing these lines
    -f <list>        Bypassing these white-space separated fields
    -g <pattern>     Bypassing lines that match the regular expression <pattern>

FLAGS:
    -h, --help       Prints help information
    -v               Invert the sense of selecting
    -G               -g adopts Oniguruma regular expressions
    -o               -g selects only matched parts
    -s               Execute command for each selected part
    -V, --version    Prints version information
    -z               Line delimiter is NUL instead of a newline

EXAMPLES:
  Edit 2nd, 3rd, and 4th columns in the CSV file
    $ cat file.csv | teip -f 2-4 -d , -- sed 's/./@/g'
  Convert timestamps in /var/log/secure to UNIX time
    $ cat /var/log/secure | teip -c 1-15 -- date -f- +%s
  Edit the line containing 'hello' and the three lines before and after it
    $ cat access.log | teip -e 'grep -n -C 3 hello' -- sed 's/./@/g'

Full documentation at:<https://github.com/greymd/teip>",
)]

struct Args {
    #[structopt(short = "g")]
    regex: Option<String>,
    #[structopt(short = "o")]
    only_matched: bool,
    #[structopt(short = "G")]
    onig_enabled: bool,
    #[structopt(short = "f")]
    list: Option<String>,
    #[structopt(short = "d")]
    delimiter: Option<String>,
    #[structopt(short = "D",)]
    regexp_delimiter: Option<String>,
    #[structopt(short = "c")]
    char: Option<String>,
    #[structopt(short = "l")]
    line: Option<String>,
    #[structopt(short = "s")]
    solid: bool,
    #[structopt(short = "v")]
    invert: bool,
    #[structopt(short = "z")]
    zero: bool,
    #[structopt(short = "e")]
    exoffload_pipeline: Option<String>,
    #[structopt(name = "command")]
    commands: Vec<String>,
}

fn main() {
    env_logger::init();

    // ***** Parse options and prepare configures *****
    let args: Args = Args::from_args();

    debug!("{:?}", args);

    if HL.len() < 2 {
        error_exit("Invalid format in TEIP_HIGHLIGHT variable")
    }

    let flag_zero = args.zero;
    let cmds: Vec<&str> = args.commands.iter().map(|s| s.as_str()).collect();
    let flag_only = args.only_matched;
    let flag_regex = args.regex.is_some();
    let flag_onig = args.onig_enabled;
    let flag_solid = args.solid;
    let flag_invert = args.invert;
    let flag_char = args.char.is_some();
    let flag_lines = args.line.is_some();
    let flag_field = args.list.is_some();
    let flag_delimiter = args.delimiter.is_some();
    let delimiter = args.delimiter.as_ref().map(|s| s.as_str()).unwrap_or("");
    let flag_regex_delimiter = args.regexp_delimiter.is_some();
    let flag_exoffload = args.exoffload_pipeline.is_some();
    let exoffload_pipeline = args.exoffload_pipeline.as_ref().map(|s| s.as_str()).unwrap_or("");

    let mut regex_mode = String::new();
    let mut regex = Regex::new("").unwrap();
    let mut regex_onig = onig::new_regex();
    let mut line_end = b'\n';
    let mut single_token_per_line = false; // true if single hole is always coveres entire line
    let mut ch: PipeIntercepter;
    let mut flag_dryrun = true;
    let regex_delimiter;

    // If any necessary flags is not enabled, show help and exit.
    if !( flag_exoffload || flag_regex || flag_field || flag_char || flag_lines) {
        Args::clap().print_help().unwrap();
        std::process::exit(1);
    }

    let char_list = args
        .char
        .as_ref()
        .and_then(|s| {
            list::converter::to_ranges(s.as_str(), flag_invert)
                .map_err(|e| error_exit(&e.to_string()))
                .ok()
        })
        .unwrap_or_else(|| list::converter::to_ranges("1", true).unwrap());

    let field_list = args
        .list
        .as_ref()
        .and_then(|s| {
            list::converter::to_ranges(s.as_str(), flag_invert)
                .map_err(|e| error_exit(&e.to_string()))
                .ok()
        })
        .unwrap_or_else(|| list::converter::to_ranges("1", true).unwrap());

    let line_list = args
        .line
        .as_ref()
        .and_then(|s| {
            list::converter::to_ranges(s.as_str(), flag_invert)
                .map_err(|e| error_exit(&e.to_string()))
                .ok()
        })
        .unwrap_or_else(|| list::converter::to_ranges("1", true).unwrap());

    if flag_zero {
        regex_mode = "(?ms)".to_string();
        line_end = b'\0';
    }

    if !flag_onig {
        regex =
            Regex::new(&(regex_mode.to_owned() + args.regex.as_ref().unwrap_or(&"".to_owned())))
                .unwrap_or_else(|e| error_exit(&e.to_string()));
    } else {
        if flag_zero {
            regex_onig =
                onig::new_option_multiline_regex(args.regex.as_ref().unwrap_or(&"".to_owned()));
        } else {
            regex_onig = onig::new_option_none_regex(args.regex.as_ref().unwrap_or(&"".to_owned()));
        }
    }

    if flag_regex_delimiter {
        regex_delimiter =
            Regex::new(&(regex_mode.to_string() + args.regexp_delimiter.as_ref().unwrap()))
                .unwrap_or_else(|e| error_exit(&e.to_string()));
    } else {
        regex_delimiter = REGEX_WS.clone();
    }

    if cmds.len() > 0 {
        flag_dryrun = false;
    }

    if (!flag_only && flag_regex) || flag_lines || flag_exoffload {
        single_token_per_line = true;
    }

    if flag_solid {
        ch =
            PipeIntercepter::start_solid_output(stringutils::vecstr_rm_references(&cmds), line_end, flag_dryrun)
                .unwrap_or_else(|e| error_exit(&e.to_string()));
    } else {
        ch = PipeIntercepter::start_output(stringutils::vecstr_rm_references(&cmds), line_end, flag_dryrun)
            .unwrap_or_else(|e| error_exit(&e.to_string()));
    }

    // ***** Start processing *****
    if single_token_per_line {
        if flag_lines {
            line_line_proc(&mut ch, &line_list, line_end)
                .unwrap_or_else(|e| error_exit(&e.to_string()));
        } else if flag_regex {
            if flag_onig {
                onig::regex_onig_line_proc(&mut ch, &regex_onig, flag_invert, line_end)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
            } else {
                regex_line_proc(&mut ch, &regex, flag_invert, line_end)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
            }
        } else if flag_exoffload {
            exoffload_proc(&mut ch, exoffload_pipeline, flag_invert, line_end)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
        }
    } else {
        let stdin = io::stdin();
        loop {
            let mut buf = Vec::with_capacity(DEFAULT_CAP);
            match stdin.lock().read_until(line_end, &mut buf) {
                Ok(0) => {
                    ch.send_eof().unwrap_or_else(|e| msg_error(&e.to_string()));
                    break;
                }
                Ok(_) => {},
                Err(e) => msg_error(&e.to_string()),
            };
            let eol = stringutils::trim_eol(&mut buf);
            if flag_regex {
                if flag_onig {
                    onig::regex_onig_proc(&mut ch, &buf, &regex_onig, flag_invert)
                        .unwrap_or_else(|e| error_exit(&e.to_string()));
                } else {
                    regex_proc(&mut ch, &buf, &regex, flag_invert)
                        .unwrap_or_else(|e| error_exit(&e.to_string()));
                }
            } else if flag_char {
                char_proc(&mut ch, &buf, &char_list)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
            } else if flag_field && flag_delimiter {
                field_proc(&mut ch, &buf, delimiter, &field_list)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
            } else if flag_field {
                field_regex_proc(&mut ch, &buf, &regex_delimiter, &field_list)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
            }
            ch.send_msg(eol)
                .unwrap_or_else(|e| msg_error(&e.to_string()));
        }
    }
}

/// External execution for match offloading ( -e )
/// TODO: Add overview of this function
fn exoffload_proc(
    ch: &mut PipeIntercepter,
    exoffload_pipeline: &str,
    invert: bool,
    line_end: u8,
) -> Result<(), errors::TokenSendError> {
    let (rx_stdin1, rx_stdin2, _thread1) = procspawn::tee(line_end)
            .unwrap_or_else(|e| error_exit(&e.to_string()));
    let (rx_messy_numbers, _thread2) = procspawn::run_pipeline_generating_numbers(exoffload_pipeline, rx_stdin1)
            .unwrap_or_else(|e| error_exit(&e.to_string()));
    let (rx_numbers, _thread3) = procspawn::clean_numbers(rx_messy_numbers, line_end);
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
                ch.send_msg(line.to_string())?;
            } else {
                ch.send_pipe(line.to_string())?;
            }
        } else {
            if invert {
                ch.send_pipe(line.to_string())?;
            } else {
                ch.send_msg(line.to_string())?;
            }
        }
        ch.send_msg(eol)?;
    }
    Ok(())
}

/// Bypassing particular lines based on given list ( -l )
fn line_line_proc(
    ch: &mut PipeIntercepter,
    ranges: &Vec<list::ranges::Range>,
    line_end: u8,
) -> Result<(), errors::TokenSendError> {
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
                    ch.send_pipe(line.to_string())?;
                } else {
                    ch.send_msg(line.to_string())?;
                }
                ch.send_msg(eol)?;
            }
            Err(e) => msg_error(&e.to_string()),
        }
        i += 1;
    }
    Ok(())
}

/// Bypassing particular lines based on Regular Expression ( -g )
fn regex_line_proc(
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
                if re.is_match(&line) {
                    if invert {
                        ch.send_msg(line.to_string())?;
                    } else {
                        ch.send_pipe(line.to_string())?;
                    }
                } else {
                    if invert {
                        ch.send_pipe(line.to_string())?;
                    } else {
                        ch.send_msg(line.to_string())?;
                    }
                }
                ch.send_msg(eol)?;
            }
            Err(e) => msg_error(&e.to_string()),
        }
    }
    Ok(())
}

/// Bypassing particular strings based on Regular Expression ( -o -g )
fn regex_proc(
    ch: &mut PipeIntercepter,
    line: &Vec<u8>,
    re: &Regex,
    invert: bool,
) -> Result<(), errors::TokenSendError> {
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
        left_index = cap.end();
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

/// Bypassing character range ( -c )
fn char_proc(
    ch: &mut PipeIntercepter,
    line: &Vec<u8>,
    ranges: &Vec<list::ranges::Range>,
) -> Result<(), errors::TokenSendError> {
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
            ch.send_msg(str_out.to_string())?;
            str_out.clear();
        } else if !is_in && last_is_in {
            ch.send_pipe(str_in.to_string())?;
            str_in.clear();
        }
        last_is_in = is_in;
    }
    if last_is_in && !str_in.is_empty() {
        ch.send_pipe(str_in)?;
    } else {
        ch.send_msg(str_out)?;
    }
    Ok(())
}

/// Bypassing white space separation ( -f )
fn field_regex_proc(
    ch: &mut PipeIntercepter,
    line: &Vec<u8>,
    re: &Regex,
    ranges: &Vec<list::ranges::Range>,
) -> Result<(), errors::TokenSendError> {
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
            ch.send_pipe(field.to_string())?;
        } else {
            ch.send_msg(field.to_string())?;
        }
        ch.send_msg(spaces.to_string())?;
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
            ch.send_pipe(field.to_string())?;
        } else {
            ch.send_msg(field.to_string())?;
        }
    }
    Ok(())
}

/// Bypassing field separation ( -f -d )
fn field_proc(
    ch: &mut PipeIntercepter,
    line: &Vec<u8>,
    delim: &str,
    ranges: &Vec<list::ranges::Range>,
) -> Result<(), errors::TokenSendError> {
    let line = String::from_utf8_lossy(&line).to_string();
    let tokens = line.split(delim);
    let mut ri = 0;
    for (i, token) in tokens.enumerate() {
        if i > 0 {
            ch.send_msg(delim.to_string())?;
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
            ch.send_pipe(token.to_string())?;
        } else {
            ch.send_msg(token.to_string())?;
        }
    }
    Ok(())
}
