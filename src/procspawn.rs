use super::DEFAULT_CAP;
use super::errors;
use super::stringutils;
use std::thread::JoinHandle;
use std::io::{self, BufRead, BufWriter, BufReader, Read, Write};
use std::thread;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self,Receiver};
use log::debug;

/// Execute command and return two pipes, stdin and stdout of the new process.
pub fn exec_cmd(
    cmds: &Vec<String>,
) -> std::result::Result<
    (
        Box<dyn Write + Send + 'static>,
        Box<dyn Read + Send + 'static>,
        String,
    ),
    errors::SpawnError,
> {
    debug!("thread: exec_cmd: {:?}", cmds);
    if cmds.len() == 0 {
        // In the case of dryrun, return dummy objects.
        return Ok((Box::new(io::sink()), Box::new(io::empty()), "".to_string()));
    }
    let child = Command::new(&cmds[0])
        .args(&cmds[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| errors::SpawnError::Io(e))?;
    let first = &cmds[0];
    let child_stdin = child.stdin.ok_or(errors::SpawnError::StdinOpenFailed)?;
    let child_stdout = child.stdout.ok_or(errors::SpawnError::StdoutOpenFailed)?;
    Ok((
        Box::new(child_stdin),
        Box::new(child_stdout),
        first.to_string(),
    ))
}

/// Execute single command and return the stdout of the command as String synchronously
pub fn exec_cmd_sync(input: String, cmds: &Vec<String>, line_end: u8) -> String {
    debug!("thread: exec_cmd_sync: {:?}", &cmds);
    let mut child = Command::new(&cmds[0])
        .args(&cmds[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn child process");
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        let mut vec = Vec::new();
        vec.extend_from_slice(input.as_bytes());
        vec.extend_from_slice(&[line_end]);
        stdin
            .write_all(vec.as_slice())
            .expect("Failed to write to stdin");
    }
    let mut output = child
        .wait_with_output()
        .expect("Failed to read stdout")
        .stdout;
    if output.ends_with(&[line_end]) {
        output.pop();
    }
    String::from_utf8_lossy(&output).to_string()
}

/// Generate two readers which prints identical standard input.
/// The behavior is similar to `tee' command but mpsc:channel queues data as much as they can.
pub fn tee(line_end: u8) -> std::result::Result<(Receiver<Vec<u8>>, Receiver<Vec<u8>>, JoinHandle<()>), errors::SpawnError> {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let handler = thread::spawn(move || {
            let stdin = io::stdin();
            loop {
                let mut buf = Vec::with_capacity(DEFAULT_CAP);
                match stdin.lock().read_until(line_end, &mut buf) {
                    Ok(0) => {
                        // Finish to read entire input, discard channels
                        drop(tx1);
                        drop(tx2);
                        break;
                    },
                    Ok(_) => {
                        // Suppress errors
                        let _ = tx1.send(buf.clone());
                        let _ = tx2.send(buf.clone());
                    },
                    Err(_) => {
                        debug!("tee_chain: Got error while loading from stdin");
                    },
                };
            }
    });
    return Ok((rx1, rx2, handler))
}

/// Spawn process with given pipeline and keep sending strings from channel as stdin.
/// Return value is BufReader of stdout.
/// Spawn process is supposed to be generating integer numbers but each line may contain some
/// noise (see clean_numbers function).
pub fn run_pipeline_generating_numbers (
    command: &str,
    input: Receiver<Vec<u8>>,
    ) -> std::result::Result<
            (BufReader<Box<dyn Read + Send>>, JoinHandle<()>),
            errors::SpawnError
    > {
    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            let cmds: Vec<String> = vec!["cmd","/C", command].into_iter().map(|s| s.to_owned()).collect();
        } else {
            let cmds: Vec<String> = vec!["sh","-c", command].into_iter().map(|s| s.to_owned()).collect();
        }
    }
    let (fd_in, fd_out, _) = self::exec_cmd(&cmds)?;
    let mut n_writer = BufWriter::new(fd_in);
    let n_reader = BufReader::new(fd_out);
    let handler = thread::spawn(move || {
        loop {
            let buf = match input.recv() {
                Ok(buf) => buf,
                Err(_) => break, // Generally, entering here means queue just got emptied. Do nothing.
            };
            match n_writer.write(&buf) {
                Ok(0) => break,
                _ => {},         // Ignore error because the command may not accept standard input (i.e seq command).
            };
        }
        n_writer = BufWriter::new(Box::new(io::sink()));
        drop(n_writer);
    });
    Ok((n_reader, handler))
}

/// Extract numbers from noisey strings.
/// Example of noisey numbers (Read from BufReader):
/// ```
/// 1: test test"
/// 2-
///     3@@@the line has spaces at beginning
///     4!!!TAB character is also acceptable
/// 5
/// ```
/// Example of result (Receiver will get u64 numbers):
/// ```
/// 1
/// 2
/// 3
/// 4
/// 5
/// ```
pub fn clean_numbers (
    mut input: BufReader<Box<dyn Read + Send>>,
    line_end: u8
) -> (Receiver<u64>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let handler = thread::spawn(move || {
        debug!("clean_numbers: thread: start");
        loop {
            let mut buf = Vec::with_capacity(DEFAULT_CAP);
            match input.read_until(line_end, &mut buf) {
                Ok(0) => break,
                Ok(_) => {},
                Err(_) => break,
            };
            let line = String::from_utf8_lossy(&buf).to_string();
            match stringutils::extract_number(line) {
                Some(i) => {
                    match tx.send(i) {
                        Ok(_) => (),
                        Err(_) => {
                            break;
                        },
                    }
                },
                None => {},
            };
        }
        drop(tx);
        debug!("clean_numbers: thread: end");
    });
    return (rx, handler)
}
