use super::DEFAULT_CAP;
use super::errors;
use super::stringutils;
use std::thread::JoinHandle;
use std::io::{self, BufRead, BufWriter, BufReader, Read, Write};
use std::thread;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self,Receiver};
use log::debug;

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
// The behavior is similar to `tee' command but mpsc:channel queues inputs as much as they can.
// There is no kernel bufferes
pub fn tee_chan(line_end: u8) -> std::result::Result<(Receiver<Vec<u8>>, Receiver<Vec<u8>>, JoinHandle<()>), errors::SpawnError> {
    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let handler = thread::spawn(move || {
            let stdin = io::stdin();
            loop {
                let mut buf = Vec::with_capacity(DEFAULT_CAP);
                match stdin.lock().read_until(line_end, &mut buf) {
                    Ok(0) => {
                        break
                    },
                    Ok(_) => {
                        match tx1.send(buf.clone()) {
                            Ok(_) => {},
                            Err(_) => {},
                        };
                        match tx2.send(buf.clone()) {
                            Ok(_) => {},
                            Err(_) => {},
                        };
                    },
                    Err(_) => {
                        debug!("tee_chain: Got error while loading from stdin");
                    },
                };
            }
    });
    return Ok((rx1, rx2, handler))
}

pub fn spawn_exoffload_command_chan (
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
            match input.recv() {
                Ok(buf) => {
                    match n_writer.write(&buf) {
                        Ok(0) => break,
                        Ok(_) => {},
                        Err(_) => {},
                    }
                },
                Err(_) => {
                    // Generally, entering here means queue got emptied.
                    break;
                },
            }
        }
        n_writer = BufWriter::new(Box::new(io::sink()));
        drop(n_writer);
    });
    Ok((n_reader, handler))
}

pub fn clean_numbers (mut input: BufReader<Box<dyn Read + Send>>, line_end: u8) -> (Receiver<u64>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let handler = thread::spawn(move || {
        debug!("clean_numbers: thread: start");
        loop {
            let mut buf = Vec::with_capacity(DEFAULT_CAP);
            match input.read_until(line_end, &mut buf) {
                Ok(n) => {
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
