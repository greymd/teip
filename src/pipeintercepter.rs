use super::token::Chunk;
use super::spawnutils;
use super::stringutils::trim_eol;
use super::{errors,errors::*};
use super::{HL,DEFAULT_CAP};

use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};
use log::debug;

/// Bypassing system and its interface set between stdin and stdout
pub struct PipeIntercepter {
    tx: Sender<Chunk>,
    pipe_writer: BufWriter<Box<dyn Write + Send + 'static>>, // Not used when -s
    handler: Option<JoinHandle<()>>,                         // "option dance"
    line_end: u8,
    solid: bool,
    dryrun: bool,
}

impl PipeIntercepter {
    /// Spawn an external which receive from bypassed data and modify it
    ///            Example:
    ///            `````````````````````````````````````````````````````````````
    ///            $ echo -e "AAA\nBBB\nCCC\nDDD" | teip -l 3,4 -- sed 's/./@/g'
    ///            AAA
    ///            BBB
    ///            @@@
    ///            @@@
    ///            `````````````````````````````````````````````````````````````
    ///            ┌─────────────────────────┐      ┌────────────────────────────────────────────────────────────────────────────┐
    ///            │ Main thread             │      │ PipeIntercepter                                                            │
    ///            ├─────────────────────────┤      ├────────────────────────────────────────────────────────────────────────────┤
    ///            │  ┌────────────────────┐ │      │                                                    ┌─────────────────────┐ │
    ///            │  │ line_line_proc     │ │      │                                                    │ start_output thread │ │
    ///            │  ├────────────────────┤ │   ┌──┴────────┐   Keep("AAA")   ┌─────────────────┐       ├─────────────────────┤ │
    /// ┌───────┐  │  │                    │ │   │           │   Keep("BBB")   │ tx queue        │       │                     │ │
    /// │ stdin ├──┼──►  "AAA" <= Unmatch──┼─┼───►           │                 │ (std:sync:mpsc) │       │  Keep("AAA")        │ │
    /// └───────┘  │  │                    │ │   │ send_keep ├─────────────────►                 ├─(rx)──►                     │ │
    ///            │  │  "BBB" <= Unmatch──┼─┼───►           │                 │                 │       │  Keep("BBB")        │ │
    ///            │  │                    │ │   │           │       ┌─────────►                 │       │ ┌────────────────┐  │ │   ┌────────┐
    ///            │  │                    │ │   └──┬────────┘       │         └─────────────────┘       │ │Hole = "@@@"    │  ├─┼───► stdout │
    ///            │  │                    │ │      │                │                                   │ ├────────────────┤  │ │   └────────┘
    ///            │  │                    │ │   ┌──┴─────────┐      │ Hole                              │ │Hole = "@@@"    │  │ │
    ///            │  │  "CCC" <= Match────┼─┼───►            │      │ Hole    ┌───────────────────┐     │ └──▲─────────────┘  │ │    AAA
    ///            │  │                    │ │   │            ├──────┘         │ Pipe queue        │     │    │                │ │    BBB
    ///            │  │  "DDD" <= Match────┼─┼───► send_byps  │                │ (Targeted Command)├─────┼──(pipe_reader)      │ │    @@@
    ///            │  │                    │ │   │            ├──(pipe_writer)─►                   │     │                     │ │    @@@
    ///            │  │                    │ │   │            │                │  sed 's/./@/g'    │     └─────────────────────┘ │
    ///            │  └────────────────────┘ │   └──┬─────────┘    "CCC"       └───────────────────┘                             │
    ///            │                         │      │              "DDD"                                                         │
    ///            └─────────────────────────┘      └────────────────────────────────────────────────────────────────────────────┘
    pub fn start_output(
        cmds: Vec<String>,
        line_end: u8,
        dryrun: bool,
    ) -> Result<PipeIntercepter, errors::SpawnError> {
        let (tx, rx) = mpsc::channel();
        let (child_stdin, child_stdout, _) = spawnutils::exec_cmd(&cmds)?;
        let pipe_writer = BufWriter::new(child_stdin);
        let handler = thread::spawn(move || {
            debug!("thread: spawn");
            let mut pipe_reader = BufReader::new(child_stdout);
            let mut result_writer = BufWriter::new(io::stdout());
            loop {
                let token = match rx.recv() {
                    Ok(t) => t,
                    Err(e) => {
                        msg_error(&e.to_string());
                        break;
                    }
                };
                match token {
                    Chunk::Keep(msg) => {
                        debug!("thread: rx.recv <= Keep:[{}]", msg);
                        result_writer
                            .write(msg.as_bytes())
                            .unwrap_or_else(|e| exit_silently(&e.to_string()));
                    }
                    Chunk::Hole => {
                        debug!("thread: rx.recv <= Hole");
                        match PipeIntercepter::read_pipe(&mut pipe_reader, line_end) {
                            Ok(msg) => {
                                result_writer
                                    .write(msg.as_bytes())
                                    .unwrap_or_else(|e| exit_silently(&e.to_string()));
                            }
                            Err(e) => {
                                // pipe may be exhausted
                                result_writer.flush().unwrap();
                                error_exit(&e.to_string())
                            }
                        }
                    }
                    Chunk::EOF => {
                        debug!("thread: rx.recv <= EOF");
                        break;
                    }
                    _ => {
                        error_exit("Exit with bug.");
                    }
                };
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

    /// Spawn an external process for solid mode
    ///            Example:
    ///            `````````````````````````````````````````````````````````````
    ///            $ echo -e "AAA\nBBB\nCCC\nDDD" | teip -s -l 3,4 -- sed 's/./@/g'
    ///            AAA
    ///            BBB
    ///            @@@
    ///            @@@
    ///            `````````````````````````````````````````````````````````````
    ///            ┌─────────────────────────┐      ┌───────────────────────────────────────────────────────────────────────────────────────┐
    ///            │ Main thread             │      │ PipeIntercepter                                                                       │
    ///            ├─────────────────────────┤      ├───────────────────────────────────────────────────────────────────────────────────────┤
    ///            │                         │      │                                                    ┌───────────────────────────────┐  │
    ///            │  ┌────────────────────┐ │      │                                                    │ start_solid_output            │  │
    ///            │  │ line_line_proc     │ │      │                                                    │ thread                        │  │
    ///            │  ├────────────────────┤ │   ┌──┴────────┐   Keep("AAA")   ┌─────────────────┐       ├───────────────────────────────┤  │
    /// ┌───────┐  │  │                    │ │   │           │   Keep("BBB")   │ tx queue        │       │                               │  │
    /// │ stdin ├──┼──►  "AAA" <= Unmatch──┼─┼───►           │                 │ (std:sync:mpsc) │       │ Keep("AAA")                   │  │
    /// └───────┘  │  │                    │ │   │ send_keep ├─────────────────►                 ├─(rx)──►                               │  │
    ///            │  │  "BBB" <= Unmatch──┼─┼───►           │                 │                 │       │ Keep("BBB")                   │  │
    ///            │  │                    │ │   │           │       ┌─────────►                 │       │                               │  │  ┌────────┐
    ///            │  │                    │ │   └──┬────────┘       │         └─────────────────┘       │                               ├──┼──► stdout │
    ///            │  │                    │ │      │                │                                   │                               │  │  └────────┘
    ///            │  │                    │ │   ┌──┴─────────┐      │                                   │ SHole("CCC")                  │  │
    ///            │  │  "CCC" <= Match────┼─┼───►            │      │ SHole("CCC")                      │   │ ┌───────────────┐         │  │   AAA
    ///            │  │                    │ │   │ send_byps  ├──────┘ SHole("DDD")                      │   │ │exec_cmd_sync  ├─► "@@@" │  │   BBB
    ///            │  │  "DDD" <= Match────┼─┼───►            │                                          │   └─►  sed 's/./@/g'│         │  │   @@@
    ///            │  │                    │ │   │ solid=true │                                          │     └───────────────┘         │  │   @@@
    ///            │  │                    │ │   │            │                                          │                               │  │
    ///            │  └────────────────────┘ │   └──┬─────────┘                                          │ SHole("DDD")                  │  │
    ///            │                         │      │                                                    │   │ ┌───────────────┐         │  │
    ///            │                         │      │                                                    │   │ │exec_cmd_sync  ├─► "@@@" │  │
    ///            └─────────────────────────┘      │                                                    │   └─►  sed 's/./@/g'│         │  │
    ///                                             │                                                    │     └───────────────┘         │  │
    ///                                             │                                                    └───────────────────────────────┘  │
    ///                                             └───────────────────────────────────────────────────────────────────────────────────────┘
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
                let token = match rx.recv() {
                    Ok(t) => t,
                    Err(e) => {
                        msg_error(&e.to_string());
                        break;
                    }
                };
                match token {
                    Chunk::Keep(msg) => {
                        debug!("thread: rx.recv <= Keep:[{}]", msg);
                        writer
                            .write(msg.as_bytes())
                            .unwrap_or_else(|e| exit_silently(&e.to_string()));
                    }
                    Chunk::SHole(msg) => {
                        debug!("thread: rx.recv <= SHole:[{}]", msg);
                        let result = spawnutils::exec_cmd_sync(msg, &cmds, line_end);
                        writer
                            .write(result.as_bytes())
                            .unwrap_or_else(|e| exit_silently(&e.to_string()));
                    }
                    Chunk::EOF => {
                        debug!("thread: rx.recv <= EOF");
                        break;
                    }
                    _ => {
                        error_exit("Exit with bug.");
                    }
                };
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

    /// Print string as is, that means it outputs to stdout without any modifications.
    /// This is data "under the masking tape".
    pub fn send_keep(&self, msg: String) -> Result<(), errors::TokenSendError> {
        debug!("tx.send => Channle({})", msg);
        self.tx
            .send(Chunk::Keep(msg))
            .map_err(|e| errors::TokenSendError::Channel(e))?;
        Ok(())
    }

    /// Bypassing strings to the pipe and will be modified by the targeted command.
    /// This is data is in the hole on the masking tape".
    pub fn send_byps(&mut self, msg: String) -> Result<(), errors::TokenSendError> {
        if self.dryrun {
            // Highlight the string instead of bypassing
            let msg_highlighted: String;
            msg_highlighted = HL[0].to_string() + &msg + HL[1];
            debug!("tx.send => Channle({})", msg_highlighted);
            self.tx
                .send(Chunk::Keep(msg_highlighted))
                .map_err(|e| errors::TokenSendError::Channel(e))?;
            return Ok(());
        }
        if self.solid {
            debug!("tx.send => Solid({})", msg);
            self.tx
                .send(Chunk::SHole(msg))
                .map_err(|e| errors::TokenSendError::Channel(e))?;
            Ok(())
        } else {
            debug!("tx.send => Piped");
            self.tx
                .send(Chunk::Hole)
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

    /// Notify PipeIntercepter the end of file to exit process
    pub fn send_eof(&self) -> Result<(), errors::TokenSendError> {
        debug!("tx.send => EOF");
        self.tx
            .send(Chunk::EOF)
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
