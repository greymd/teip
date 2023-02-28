mod list {
    pub mod converter;
    pub mod ranges;
}
mod csv {
    pub mod parser;
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
    static ref GREP_PATH: String = match env::var("TEIP_GREP_PATH") {
        Ok(v) => v,
        Err(_) => "grep".to_string(),
    };
    static ref SED_PATH: String = match env::var("TEIP_SED_PATH") {
        Ok(v) => v,
        Err(_) => "sed".to_string(),
    };
    static ref AWK_PATH: String = match env::var("TEIP_AWK_PATH") {
        Ok(v) => v,
        Err(_) => "awk".to_string(),
    };
    static ref HL: Vec<&'static str> = DEFAULT_HIGHLIGHT.split("{}").collect();
}

#[derive(StructOpt, Debug)]
#[structopt(
    about = "Bypassing a partial range of standard input to an arbitrary command",
    usage = "teip [OPTIONS] [FLAGS] [--] [<command>...]",
    help = "USAGE:
  teip -g <pattern> [-osvz] [--] [<command>...]
  teip -E <pattern> [-osvz] [--] [<command>...]
  teip -c <list> [-svz] [--] [<command>...]
  teip -l <list> [-svz] [--] [<command>...]
  teip -f <list> [-d <delimiter> | -D <pattern> | --csv] [-svz] [--] [<command>...]
  teip -e <string> [-svz] [--] [<command>...]

OPTIONS:
    -g <pattern>        Bypassing lines that match the regular expression <pattern>
        -o              -g bypasses only matched parts
        -G              [Deprecated] Use -E instead. -g interprets Oniguruma
                        regular expressions.
    -E <pattern>        Behaves like -g but interprets <pattern> as Oniguruma regular
                        expression
    -c <list>           Bypassing these characters
    -l <list>           Bypassing these lines
    -f <list>           Bypassing these white-space separated fields
        -d <delimiter>  Use <delimiter> for field delimiter of -f
        -D <pattern>    Use regular expression <pattern> for field delimiter of -f
        --csv           -f interprets <list> as field number of a CSV according to
                        RFC 4180, instead of white-space separated fields
    -e <string>         Execute <string> on another process that will receive identical
                        standard input as the teip, and numbers given by the result
                        are used as line numbers for bypassing

FLAGS:
    -h, --help          Prints help information
    -V, --version       Prints version information
    -s                  Execute new command for each bypassed chunk
      --chomp           Command spawned by -s receives standard input without trailing
                        newlines
    -v                  Invert the range of bypassing
    -z                  Line delimiter is NUL instead of a newline

EXPERIMENTAL OPTIONS:
    -g <pattern>
        -A <number>     Alias of -e 'grep -n -A <number> <pattern>'
        -B <number>     Alias of -e 'grep -n -B <number> <pattern>'
        -C <number>     Alias of -e 'grep -n -C <number> <pattern>'
    --sed <pattern>     Alias of -e 'sed -n \"<pattern>=\"'
    --awk <pattern>     Alias of -e 'awk \"<pattern>{print NR}\"'

EXAMPLES:
  Replace 'WORLD' to 'EARTH' on line including 'HELLO' in input:
    $ cat file | teip -g HELLO -- sed 's/WORLD/EARTH/'
  Edit '|' separated fields of input:
    $ cat file.csv | teip -d '|' -f 2 -- sed 's/./@/g'
  Edit 2nd and 3rd columns in the CSV file:
    $ cat file.csv | teip --csv -f 2,3 -- sed 's/./@/g'
  Convert timestamps in /var/log/secure to UNIX time:
    $ cat /var/log/secure | teip -c 1-15 -- date -f- +%s
  Edit the line containing 'HELLO' and the three lines before and after it:
    $ cat access.log | teip -e 'grep -n -C 3 HELLO' -- sed 's/./@/g'

Full documentation at:<https://github.com/greymd/teip>",
)]

struct Args {
    #[structopt(short = "g")]
    regex: Option<String>,
    #[structopt(short = "o")]
    only_matched: bool,
    #[structopt(short = "G")]
    onig_enabled: bool,
    #[structopt(short = "E")]
    regex_onig: Option<String>,
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
    #[structopt(long = "chomp")]
    solid_chomp: bool,
    #[structopt(short = "v")]
    invert: bool,
    #[structopt(short = "z")]
    zero: bool,
    #[structopt(short = "e")]
    exoffload_pipeline: Option<String>,
    #[structopt(short = "A")]
    after: Option<usize>,
    #[structopt(short = "B")]
    before: Option<usize>,
    #[structopt(short = "C")]
    center: Option<usize>,
    #[structopt(long = "sed")]
    sed: Option<String>,
    #[structopt(long = "awk")]
    awk: Option<String>,
    #[structopt(name = "command")]
    commands: Vec<String>,
}

fn main() {
    env_logger::init();

    // ***** Parse options and prepare configures *****
    let mut args: Args = Args::from_args();

    debug!("{:?}", args);

    if HL.len() < 2 {
        error_exit("Invalid format in TEIP_HIGHLIGHT variable")
    }

    let flag_zero = args.zero;
    let cmds = args.commands;
    let flag_only = args.only_matched;
    let mut flag_regex = args.regex.is_some();
    let mut flag_onig = args.regex_onig.is_some();
    let flag_solid = args.solid;
    let flag_solid_chomp = args.solid_chomp;
    let flag_invert = args.invert;
    let flag_char = args.char.is_some();
    let flag_lines = args.line.is_some();
    let flag_field = args.list.is_some();
    let flag_delimiter = args.delimiter.is_some();
    let flag_csv = args.csv;
    let delimiter = args.delimiter.as_ref().map(|s| s.as_str()).unwrap_or("");
    let flag_regex_delimiter = args.regexp_delimiter.is_some();
    let mut flag_exoffload = args.exoffload_pipeline.is_some();
    let mut exoffload_pipeline = args.exoffload_pipeline.as_ref().map(|s| s.as_str()).unwrap_or("");

    let mut regex_mode = String::new();
    let mut regex_compiled = Regex::new("").unwrap();
    let mut regex_onig_compiled = onig::new_regex();
    let mut line_end = b'\n';
    let mut process_each_line = true; // true if single hole is always coveres entire line
    let mut ch: PipeIntercepter;
    let mut flag_dryrun = true;
    let regex_delimiter;
    if args.u {
        u();
    }

    // If any of -A, -B, -C is specified, set -e option and set regex flag off
    //   "-A 1 -g pattern" => "-e 'grep -A 1 pattern'"
    //   "-B 1 -g pattern" => "-e 'grep -B 1 pattern'"
    //   "-C 1 -g pattern" => "-e 'grep -C 1 pattern'"
    let mut grep_args = vec![GREP_PATH.to_string(), "-n".to_string()];
    let pipeline;
    if ( args.after.is_some() || args.before.is_some() || args.center.is_some() ) && flag_regex {
        if let Some(n) = args.after {
            grep_args.push("-A".to_string());
            grep_args.push(n.to_string());
        } else if let Some(n) = args.before {
            grep_args.push("-B".to_string());
            grep_args.push(n.to_string());
        } else if let Some(n) = args.center {
            grep_args.push("-C".to_string());
            grep_args.push(n.to_string());
        }
        if let Some(ref pattern) = args.regex {
            grep_args.push(pattern.to_string());
        }
        flag_exoffload = true;
        flag_regex = false;
        pipeline = grep_args.join(" ");
        exoffload_pipeline = &pipeline;
    } else if let Some(ref pattern) = args.sed {
        // --sed option
        flag_exoffload = true;
        pipeline = format!("{} -n '{}='", SED_PATH.as_str(), pattern);
        exoffload_pipeline = &pipeline;
    } else if let Some(ref pattern) = args.awk {
        // --awk option
        flag_exoffload = true;
        pipeline = format!("{} '{}{{print NR}}'", AWK_PATH.as_str(), pattern);
        exoffload_pipeline = &pipeline;
    }

    // Make -G works to keep backword compatibility
    // -G -g pattern => -E pattern
    if args.onig_enabled {
        flag_onig = true;
        flag_regex = false;
        args.regex_onig = args.regex.clone();
    }

    // If any mandatory flags is not enabled, show help and exit.
    if !( flag_exoffload ||
          flag_regex     ||
          flag_onig      ||
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

    if flag_regex {
        // Use default regex engine
        regex_compiled =
            Regex::new(&(regex_mode.to_owned() + args.regex.as_ref().unwrap_or(&"".to_owned())))
                .unwrap_or_else(|e| error_exit(&e.to_string()));
    }

    if flag_onig {
        // If -G option is specified, change regex engine
        if flag_zero {
            regex_onig_compiled =
                onig::new_option_multiline_regex(args.regex_onig.as_ref().unwrap_or(&"".to_owned()));
        } else {
            regex_onig_compiled = onig::new_option_none_regex(args.regex_onig.as_ref().unwrap_or(&"".to_owned()));
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
            PipeIntercepter::start_solid_output(cmds, line_end, flag_dryrun, flag_solid_chomp)
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
                procs::regex_proc(&mut ch, &buf, &regex_compiled, flag_invert)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
            } else if flag_onig {
                onig::regex_onig_proc(&mut ch, &buf, &regex_onig_compiled, flag_invert)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
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
                onig::regex_onig_line_proc(&mut ch, &regex_onig_compiled, flag_invert, line_end)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
            } else {
                procs::regex_line_proc(&mut ch, &regex_compiled, flag_invert, line_end)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
            }
        } else if flag_exoffload {
            procs::exoffload_proc(&mut ch, exoffload_pipeline, flag_invert, line_end)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
        } else if flag_csv {
            procs::csv_proc(&mut ch, &field_list, line_end, flag_solid)
                    .unwrap_or_else(|e| error_exit(&e.to_string()));
        }
    }
}
