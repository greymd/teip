use super::DEFAULT_CAP;
use super::{errors,errors::*};
use super::stringutils;
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
pub fn tee(line_end: u8) -> std::result::Result<(BufReader<FileDescriptor>, BufReader<FileDescriptor>), errors::SpawnError> {
    let fd1 = Pipe::new().map_err(|e| errors::SpawnError::Fd(e))?;
    let fd2 = Pipe::new().map_err(|e| errors::SpawnError::Fd(e))?;
    let mut writer1 = BufWriter::new(fd1.write);
    let mut writer2 = BufWriter::new(fd2.write);
    let reader1 = BufReader::new(fd1.read);
    let reader2 = BufReader::new(fd2.read);
    thread::spawn(move || {
        debug!("tee: thread: start");
        let stdin = io::stdin();
        loop {
            let mut buf = Vec::with_capacity(DEFAULT_CAP);
            match stdin.lock().read_until(line_end, &mut buf) {
                Ok(n) => {
                    let line = String::from_utf8_lossy(&buf).to_string();
                    writer1.write(line.as_bytes()).unwrap();
                    writer2.write(line.as_bytes()).unwrap();
                    if n == 0 {
                        break;
                    }
                },
                Err(e) => {
                    error_exit(&e.to_string())
                }
            }
        }
        debug!("tee: thread: end");
    });
    Ok((reader1, reader2))
}

pub fn start_moffload_filter (
    command: &str,
    mut input: BufReader<FileDescriptor>,
    line_end: u8
    ) -> std::result::Result<
            BufReader<Box<dyn Read + Send>>,
            errors::SpawnError
    > {
    debug!("start_moffload_filter: start");
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
    thread::spawn(move || {
        debug!("start_moffload_filter: thread: start");
        loop {
            let mut buf = Vec::with_capacity(DEFAULT_CAP);
            match input.read_until(line_end, &mut buf) {
                Ok(n) => {
                    let line = String::from_utf8_lossy(&buf).to_string();
                    n_writer.write(line.as_bytes()).unwrap();
                    if n == 0 {
                        break;
                    }
                },
                Err(_) => {
                    break;
                },
            }
        }
    });
    debug!("start_moffload_filter: end");
    Ok(n_reader)
}

pub fn clean_numbers (mut input: BufReader<Box<dyn Read + Send>>, line_end: u8) -> Receiver<u64> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        debug!("clean_numbers: thread: start");
        loop {
            let mut buf = Vec::with_capacity(DEFAULT_CAP);
            match input.read_until(line_end, &mut buf) {
                Ok(n) => {
                    let line = String::from_utf8_lossy(&buf).to_string();
                    match stringutils::extract_number(line) {
                        Some(i) => {
                            debug!("clean_numbers: thread: tx => {}", i);
                            tx.send(i).unwrap();
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
    return rx
}
