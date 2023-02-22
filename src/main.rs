mod list {
    pub mod converter;
    pub mod ranges;
}
mod csv {
    pub mod parser;
    pub mod terminator;
}
mod impure {
    #[cfg(feature = "oniguruma")]
    pub mod onig;
}
mod pure {
    #[cfg(not(feature = "oniguruma"))]
    pub mod onig;
}
mod chunk;
mod procs;
mod errors;
mod spawnutils;
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
  teip -f <list> [-d <delimiter> | -D <pattern> | --csv] [-svz] [--] [<command>...]
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
    -v               Invert the range of bypassing
    -G               -g adopts Oniguruma regular expressions
    -o               -g bypasses only matched parts
    -s               Execute new command for each bypassed part
    -V, --version    Prints version information
    -z               Line delimiter is NUL instead of a newline
    --csv            -f uses CSV parser instead of white-space separated fields

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
    #[structopt(long = "csv",)]
    csv: bool,
    #[structopt(long = "\x75\x6E\x6B\x6F")]
    u: bool,
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
    let cmds = args.commands;
    let flag_only = args.only_matched;
    let flag_regex = args.regex.is_some();
    let flag_onig = args.onig_enabled;
    let flag_solid = args.solid;
    let flag_invert = args.invert;
    let flag_char = args.char.is_some();
    let flag_lines = args.line.is_some();
    let flag_field = args.list.is_some();
    let flag_delimiter = args.delimiter.is_some();
    let flag_csv = args.csv;
    let delimiter = args.delimiter.as_ref().map(|s| s.as_str()).unwrap_or("");
    let flag_regex_delimiter = args.regexp_delimiter.is_some();
    let flag_exoffload = args.exoffload_pipeline.is_some();
    let exoffload_pipeline = args.exoffload_pipeline.as_ref().map(|s| s.as_str()).unwrap_or("");

    let mut regex_mode = String::new();
    let mut regex = Regex::new("").unwrap();
    let mut regex_onig = onig::new_regex();
    let mut line_end = b'\n';
    let mut process_each_line = true; // true if single hole is always coveres entire line
    let mut ch: PipeIntercepter;
    let mut flag_dryrun = true;
    let regex_delimiter;
    if args.u {
        u();
    }

    // If any mandatory flags is not enabled, show help and exit.
    if !( flag_exoffload ||
          flag_regex     ||
          flag_field     ||
          flag_char      ||
          flag_lines )
        // Even though --csv is specified, -f is not specified, show help and exit.
        || ( flag_csv && !flag_field)
    {
        Args::clap().print_help().unwrap();
        std::process::exit(1);
    }

    // Parse argument of -c option if specified
    let char_list = args
        .char
        .as_ref()
        .and_then(|s| {
            list::converter::to_ranges(s.as_str(), flag_invert)
                .map_err(|e| error_exit(&e.to_string()))
                .ok()
        })
        .unwrap_or_else(|| list::converter::to_ranges("1", true).unwrap());

    // Parse argument of -f option if specified
    let field_list = args
        .list
        .as_ref()
        .and_then(|s| {
            list::converter::to_ranges(s.as_str(), flag_invert)
                .map_err(|e| error_exit(&e.to_string()))
                .ok()
        })
        .unwrap_or_else(|| list::converter::to_ranges("1", true).unwrap());

    // Parse argument of -l option if specified
    let line_list = args
        .line
        .as_ref()
        .and_then(|s| {
            list::converter::to_ranges(s.as_str(), flag_invert)
                .map_err(|e| error_exit(&e.to_string()))
                .ok()
        })
        .unwrap_or_else(|| list::converter::to_ranges("1", true).unwrap());

    // If -z option is specified, change regex mode and line end
    if flag_zero {
        regex_mode = "(?ms)".to_string();
        line_end = b'\0';
    }

    if !flag_onig {
        // Use default regex engine
        regex =
            Regex::new(&(regex_mode.to_owned() + args.regex.as_ref().unwrap_or(&"".to_owned())))
                .unwrap_or_else(|e| error_exit(&e.to_string()));
    } else {
        // If -G option is specified, change regex engine
        if flag_zero {
            regex_onig =
                onig::new_option_multiline_regex(args.regex.as_ref().unwrap_or(&"".to_owned()));
        } else {
            regex_onig = onig::new_option_none_regex(args.regex.as_ref().unwrap_or(&"".to_owned()));
        }
    }

    // If -D option is specified, compile regex delimiter
    if flag_regex_delimiter {
        regex_delimiter =
            Regex::new(&(regex_mode.to_string() + args.regexp_delimiter.as_ref().unwrap()))
                .unwrap_or_else(|e| error_exit(&e.to_string()));
    } else {
        regex_delimiter = REGEX_WS.clone();
    }

    // If no command is specified, set dryrun mode
    if cmds.len() > 0 {
        flag_dryrun = false;
    }

    if (!flag_only && flag_regex) || flag_lines || flag_exoffload || flag_csv {
        // The process requires to process whole stdin, not line by line
        process_each_line = false;
    }

    if flag_solid {
        ch =
            PipeIntercepter::start_solid_output(cmds, line_end, flag_dryrun)
                .unwrap_or_else(|e| error_exit(&e.to_string()));
    } else {
        ch = PipeIntercepter::start_output(cmds, line_end, flag_dryrun)
            .unwrap_or_else(|e| error_exit(&e.to_string()));
    }

    // ***** Start processing *****
    if process_each_line {
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
                    procs::regex_proc(&mut ch, &buf, &regex, flag_invert)
                        .unwrap_or_else(|e| error_exit(&e.to_string()));
                }
            } else if flag_char {
                procs::char_proc(&mut ch, &buf, &char_list)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
            } else if flag_field && flag_delimiter {
                procs::field_proc(&mut ch, &buf, delimiter, &field_list)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
            } else if flag_field {
                procs::field_regex_proc(&mut ch, &buf, &regex_delimiter, &field_list)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
            }
            ch.send_keep(eol)
                .unwrap_or_else(|e| msg_error(&e.to_string()));
        }
    } else {
        if flag_lines {
            procs::line_line_proc(&mut ch, &line_list, line_end)
                .unwrap_or_else(|e| error_exit(&e.to_string()));
        } else if flag_regex {
            if flag_onig {
                onig::regex_onig_line_proc(&mut ch, &regex_onig, flag_invert, line_end)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
            } else {
                procs::regex_line_proc(&mut ch, &regex, flag_invert, line_end)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
            }
        } else if flag_exoffload {
            procs::exoffload_proc(&mut ch, exoffload_pipeline, flag_invert, line_end)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
        } else if flag_csv {
            procs::csv_proc(&mut ch, &field_list)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
        }
    }
}
