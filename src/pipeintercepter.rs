use super::token::Token;
use super::stringutils::trim_eol;
use super::{errors,errors::*};
use super::{HL,DEFAULT_CAP};

use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};
use log::debug;

pub struct PipeIntercepter {
    tx: Sender<Token>,
    pipe_writer: BufWriter<Box<dyn Write + Send + 'static>>, // Not used when -s
    handler: Option<JoinHandle<()>>,                         // "option dance"
    line_end: u8,
    solid: bool,
    dryrun: bool,
}

impl PipeIntercepter {
    // Spawn another process which continuously prints results
    pub fn start_output(
        cmds: Vec<String>,
        line_end: u8,
        dryrun: bool,
    ) -> Result<PipeIntercepter, errors::SpawnError> {
        let (tx, rx) = mpsc::channel();
        let (child_stdin, child_stdout, _) = PipeIntercepter::exec_cmd(&cmds)?;
        let pipe_writer = BufWriter::new(child_stdin);
        let handler = thread::spawn(move || {
            debug!("thread: spawn");
            let mut reader = BufReader::new(child_stdout);
            let mut writer = BufWriter::new(io::stdout());
            loop {
                match rx.recv() {
                    Ok(token) => match token {
                        Token::Channel(msg) => {
                            debug!("thread: rx.recv <= Channle:[{}]", msg);
                            writer
                                .write(msg.as_bytes())
                                .unwrap_or_else(|e| exit_silently(&e.to_string()));
                        }
                        Token::Piped => {
                            debug!("thread: rx.recv <= Piped");
                            match PipeIntercepter::read_pipe(&mut reader, line_end) {
                                Ok(msg) => {
                                    writer
                                        .write(msg.as_bytes())
                                        .unwrap_or_else(|e| exit_silently(&e.to_string()));
                                }
                                Err(e) => {
                                    // pipe may be exhausted
                                    writer.flush().unwrap();
                                    error_exit(&e.to_string())
                                }
                            }
                        }
                        Token::EOF => {
                            debug!("thread: rx.recv <= EOF");
                            break;
                        }
                        _ => {
                            error_exit("Exit with bug.");
                        }
                    },
                    Err(e) => {
                        msg_error(&e.to_string());
                        break;
                    }
                }
            }
        });
        Ok(PipeIntercepter {
            tx,
            pipe_writer,
            handler: Some(handler),
            line_end,
            solid: false,
            dryrun,
        })
    }

    // Spawn another process for solid mode
    pub fn start_solid_output(
        cmds: Vec<String>,
        line_end: u8,
        dryrun: bool,
    ) -> Result<PipeIntercepter, errors::SpawnError> {
        let (tx, rx) = mpsc::channel();
        let handler = thread::spawn(move || {
            debug!("thread: spawn");
            let mut writer = BufWriter::new(io::stdout());
            loop {
                match rx.recv() {
                    Ok(token) => match token {
                        Token::Channel(msg) => {
                            debug!("thread: rx.recv <= Channle:[{}]", msg);
                            writer
                                .write(msg.as_bytes())
                                .unwrap_or_else(|e| exit_silently(&e.to_string()));
                        }
                        Token::Solid(msg) => {
                            debug!("thread: rx.recv <= Solid:[{}]", msg);
                            let result = PipeIntercepter::exec_cmd_sync(msg, &cmds, line_end);
                            writer
                                .write(result.as_bytes())
                                .unwrap_or_else(|e| exit_silently(&e.to_string()));
                        }
                        Token::EOF => {
                            debug!("thread: rx.recv <= EOF");
                            break;
                        }
                        _ => {
                            error_exit("Exit with bug.");
                        }
                    },
                    Err(e) => {
                        msg_error(&e.to_string());
                        break;
                    }
                }
            }
        });
        let dummy = Box::new(io::sink());
        Ok(PipeIntercepter {
            tx,
            pipe_writer: BufWriter::new(dummy),
            handler: Some(handler),
            line_end,
            solid: true,
            dryrun,
        })
    }

    fn read_pipe<R: BufRead + ?Sized>(
        reader: &mut R,
        line_end: u8,
    ) -> Result<String, errors::PipeReceiveError> {
        debug!("thread: read_pipe");
        let mut buf = Vec::with_capacity(DEFAULT_CAP);
        let n = reader
            .read_until(line_end, &mut buf)
            .map_err(|e| errors::PipeReceiveError::Io(e))?;
        if n == 0 {
            // If pipe is exhausted, throw error.
            return Err(errors::PipeReceiveError::EndOfFd);
        }
        trim_eol(&mut buf);
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    fn exec_cmd(
        cmds: &Vec<String>,
    ) -> Result<
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

    fn exec_cmd_sync(input: String, cmds: &Vec<String>, line_end: u8) -> String {
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

    pub fn send_msg(&self, msg: String) -> Result<(), errors::TokenSendError> {
        debug!("tx.send => Channle({})", msg);
        self.tx
            .send(Token::Channel(msg))
            .map_err(|e| errors::TokenSendError::Channel(e))?;
        Ok(())
    }

    pub fn send_pipe(&mut self, msg: String) -> Result<(), errors::TokenSendError> {
        if self.dryrun {
            let msg_annotated: String;
            msg_annotated = HL[0].to_string() + &msg + HL[1];
            debug!("tx.send => Channle({})", msg_annotated);
            self.tx
                .send(Token::Channel(msg_annotated))
                .map_err(|e| errors::TokenSendError::Channel(e))?;
            return Ok(());
        }
        if self.solid {
            debug!("tx.send => Solid({})", msg);
            self.tx
                .send(Token::Solid(msg))
                .map_err(|e| errors::TokenSendError::Channel(e))?;
            Ok(())
        } else {
            debug!("tx.send => Piped");
            self.tx
                .send(Token::Piped)
                .map_err(|e| errors::TokenSendError::Channel(e))?;
            debug!("stdin => {}[line_end]", msg);
            self.pipe_writer
                .write(msg.as_bytes())
                .map_err(|e| errors::TokenSendError::Pipe(e))?;
            self.pipe_writer
                .write(&[self.line_end])
                .map_err(|e| errors::TokenSendError::Pipe(e))?;
            Ok(())
        }
    }

    pub fn send_eof(&self) -> Result<(), errors::TokenSendError> {
        debug!("tx.send => EOF");
        self.tx
            .send(Token::EOF)
            .map_err(|e| errors::TokenSendError::Channel(e))?;
        Ok(())
    }
}

impl Drop for PipeIntercepter {
    fn drop(&mut self) {
        debug!("close pipe");
        // Replace the writer with a dummy object to close the pipe.
        self.pipe_writer = BufWriter::new(Box::new(io::sink()));
        self.handler.take().unwrap().join().unwrap();
    }
}
