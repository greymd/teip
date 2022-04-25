use super::DEFAULT_CAP;
use super::errors;
use super::stringutils;
use std::thread::JoinHandle;
use std::sync::Mutex;
use std::sync::Arc;
use std::io::{self, BufRead, BufWriter, BufReader, Read, Write};
use std::thread;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self,Receiver};
use log::debug;
use filedescriptor::*;

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

// Generate two readers which prints identical standard input.
// Even if either reader outputs a line, another one will keep the identical line.
// The behavior is similar to `tee' command.
pub fn tee(line_end: u8) -> std::result::Result<(BufReader<FileDescriptor>, BufReader<FileDescriptor>, JoinHandle<()>, Arc<Mutex<u64>>, Arc<Mutex<u64>>), errors::SpawnError> {
    let fd1 = Pipe::new().map_err(|e| errors::SpawnError::Fd(e))?;
    let fd2 = Pipe::new().map_err(|e| errors::SpawnError::Fd(e))?;
    let mut writer1 = BufWriter::new(fd1.write);
    let mut writer2 = BufWriter::new(fd2.write);
    let reader1 = BufReader::new(fd1.read);
    let reader2 = BufReader::new(fd2.read);
    let processed_bytes = Arc::new(Mutex::new(0));
    let nr = Arc::new(Mutex::new(0)); // number of read
    let byte_incrementer = Arc::clone(&processed_bytes);
    let nr_incrementer = Arc::clone(&nr);
    let handler = thread::spawn(move || {
        debug!("tee: thread: start");
        let stdin = io::stdin();
        loop {
            let mut buf = Vec::with_capacity(DEFAULT_CAP);
            // TODO: Use read method not read_until
            match stdin.lock().read_until(line_end, &mut buf) {
                Ok(n) => {
                    // debug!("tee: thread: Read from parent stdin: FINISH");
                    // debug!("tee: thread: Deliver to stdin1: START");
                    match writer1.write(&buf) {
                        Ok(n) => {
                            let mut c = byte_incrementer.lock().unwrap();
                            *c += n as u64;
                            let mut c = nr_incrementer.lock().unwrap();
                            *c += 1;
                            // debug!("tee: thread: Deliver to stdin1: FINISH");
                        },
                        Err(_) => {
                            // Writer can easily got error if the command which doesn't accept
                            // standard input is specified to -e, like teip -e 'cat file'.
                            // Therefore, do nothing explicitly here.
                            // debug!("tee: thread: writer1 error");
                        },
                    };
                    // debug!("tee: thread: Deliver to stdin2: START");
                    match writer2.write(&buf) {
                        Ok(_) => {
                            // debug!("tee: thread: Deliver to stdin2: FINISH");
                        },
                        Err(_e) => {
                            // debug!("tee: thread: writer2 error: {}", e.to_string());
                        },
                    };
                    if n == 0 {
                        break;
                    }
                },
                Err(_e) => {
                    // debug!("tee: thread: Error: {}", e.to_string());
                    drop(writer1);
                    drop(writer2);
                    break;
                }
            }
        }
        debug!("tee: thread: end");
    });
    Ok((reader1, reader2, handler, processed_bytes, nr))
}

pub fn spawn_exoffload_command (
    command: &str,
    mut input: BufReader<FileDescriptor>,
    line_end: u8
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
        // debug!("spawn_exoffload_command: thread: start");
        loop {
            let mut buf = Vec::with_capacity(DEFAULT_CAP);
            // debug!("spawn_exoffload_command: thread: Read from stdin1: START");
            match input.read_until(line_end, &mut buf) {
                Ok(n) => {
                    // debug!("spawn_exoffload_command: thread: Read from stdin1: FINISH");
                    let line = String::from_utf8_lossy(&buf).to_string();
                    match n_writer.write(line.as_bytes()) {
                        Ok(_) => (),
                        Err(e) => {
                            // debug!("spawn_exoffload_command: thread: Error: {}", e.to_string());
                        },
                    };
                    if n == 0 {
                        break;
                    }
                },
                Err(_) => {
                    break;
                },
            }
        }
        n_writer = BufWriter::new(Box::new(io::sink()));
        drop(n_writer);
    });
    debug!("spawn_exoffload_command: thread: end");
    Ok((n_reader, handler))
}

pub fn clean_numbers (mut input: BufReader<Box<dyn Read + Send>>, line_end: u8) -> (Receiver<u64>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let handler = thread::spawn(move || {
        debug!("clean_numbers: thread: start");
        loop {
            let mut buf = Vec::with_capacity(DEFAULT_CAP);
            // debug!("clean_numbers: thread: START: Read from external offloaded pipeline");
            match input.read_until(line_end, &mut buf) {
                Ok(n) => {
                    // debug!("clean_numbers: thread: FINISH: to read from external offloaded pipeline");
                    let line = String::from_utf8_lossy(&buf).to_string();
                    match stringutils::extract_number(line) {
                        Some(i) => {
                            // debug!("clean_numbers: thread: tx => {}", i);
                            match tx.send(i) {
                                Ok(_) => (),
                                Err(_) => {
                                    // debug!("clean_numbers: thread: receiver thread may be closed earler than this thread.");
                                    break;
                                },
                            }
                        },
                        None => {},
                    }
                    if n == 0 {
                        break;
                    }
                },
                Err(_) => {
                    break;
                },
            }
        }
        drop(tx);
        debug!("clean_numbers: thread: end");
    });
    return (rx, handler)
}
