use super::chunk::{Chunk, ChunkGroup};
use super::spawnutils;
use super::stringutils::trim_eol;
use super::{errors,errors::*};
use super::{HL,DEFAULT_CAP};

use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::sync::mpsc::{self, Sender};
use std::thread::{self, JoinHandle};
use log::debug;

pub struct ChunkBuf {
    keep_buf: String,
    byps_buf: String,
    last_target: Option<ChunkGroup>,
}

impl ChunkBuf {
    fn new() -> ChunkBuf {
        ChunkBuf {
            keep_buf: String::with_capacity(DEFAULT_CAP),
            byps_buf: String::with_capacity(DEFAULT_CAP),
            last_target: None,
        }
    }
}

/// struct for bypassing input and its interface
pub struct PipeIntercepter {
    tx: Sender<Chunk>,
    pipe_writer: BufWriter<Box<dyn Write + Send + 'static>>, // Not used when -s
    handler: Option<JoinHandle<()>>,                         // "option dance"
    line_end: u8,
    solid: bool,
    dryrun: bool,
    chunk_buf: ChunkBuf,
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
    ///            │  │                    │ │   │           │   Keep("BBB")   │ tx queue        │       │                     │ │
    ///  [stdin] ──┼──►  "AAA" <= Unmatch──┼─┼───►           │                 │ (std:sync:mpsc) │       │  Keep("AAA")        │ │
    ///            │  │                    │ │   │ send_keep ├─────────────────►                 ├─(rx)──►                     │ │
    ///            │  │  "BBB" <= Unmatch──┼─┼───►           │                 │                 │       │  Keep("BBB")        │ │
    ///            │  │                    │ │   │           │       ┌─────────►                 │       │ ┌────────────────┐  │ │
    ///            │  │                    │ │   └──┬────────┘       │         └─────────────────┘       │ │Hole = "@@@"    │  ├─┼───► [stdout]
    ///            │  │                    │ │      │                │                                   │ ├────────────────┤  │ │
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
                let chunk = match rx.recv() {
                    Ok(t) => t,
                    Err(e) => {
                        msg_error(&e.to_string());
                        break;
                    }
                };
                match chunk {
                    Chunk::Keep(msg) => {
                        debug!("thread: rx.recv <= Keep:[{:?}]", msg);
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
            chunk_buf: ChunkBuf::new(),
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
    ///            │  │                    │ │   │           │   Keep("BBB")   │ tx queue        │       │                               │  │
    ///  [stdin] ──┼──►  "AAA" <= Unmatch──┼─┼───►           │                 │ (std:sync:mpsc) │       │ Keep("AAA")                   │  │
    ///            │  │                    │ │   │ send_keep ├─────────────────►                 ├─(rx)──►                               │  │
    ///            │  │  "BBB" <= Unmatch──┼─┼───►           │                 │                 │       │ Keep("BBB")                   │  │
    ///            │  │                    │ │   │           │       ┌─────────►                 │       │                               │  │
    ///            │  │                    │ │   └──┬────────┘       │         └─────────────────┘       │                               ├──┼──► [stdout]
    ///            │  │                    │ │      │                │                                   │                               │  │
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
        chomp: bool,
        replace_str: Option<String>,
    ) -> Result<PipeIntercepter, errors::SpawnError> {
        let (tx, rx) = mpsc::channel();
        let is_replace = replace_str.is_some();
        let replace_str = replace_str.unwrap_or_else(|| "".to_string());
        let handler = thread::spawn(move || {
            debug!("thread: spawn");
            let mut writer = BufWriter::new(io::stdout());
            loop {
                let chunk = match rx.recv() {
                    Ok(t) => t,
                    Err(e) => {
                        msg_error(&e.to_string());
                        break;
                    }
                };
                match chunk {
                    Chunk::Keep(msg) => {
                        debug!("thread: rx.recv <= Keep:[{:?}]", msg);
                        writer
                            .write(msg.as_bytes())
                            .unwrap_or_else(|e| exit_silently(&e.to_string()));
                    }
                    Chunk::SHole(msg) => {
                        debug!("thread: rx.recv <= SHole:[{:?}]", msg);
                        // -I option
                        if is_replace {
                            let result = spawnutils::exec_cmd_sync_replace(msg, &cmds, line_end, chomp, replace_str.as_ref());
                            writer
                                .write(result.as_bytes())
                                .unwrap_or_else(|e| exit_silently(&e.to_string()));
                        } else {
                            let result = spawnutils::exec_cmd_sync(msg, &cmds, line_end, chomp);
                            writer
                                .write(result.as_bytes())
                                .unwrap_or_else(|e| exit_silently(&e.to_string()));
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
        let dummy = Box::new(io::sink());
        Ok(PipeIntercepter {
            tx,
            pipe_writer: BufWriter::new(dummy),
            handler: Some(handler),
            line_end,
            solid: true,
            dryrun,
            chunk_buf: ChunkBuf::new(),
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
        // Remove line_end from buf.
        trim_eol(&mut buf);
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    /// Print string as is, that means it outputs to stdout without any modifications.
    /// This is data "under the masking tape".
    pub fn send_keep(&self, msg: String) -> Result<(), errors::ChunkSendError> {
        debug!("tx.send => Channle({:?})", msg);
        self.tx
            .send(Chunk::Keep(msg))
            .map_err(|e| errors::ChunkSendError::Channel(e))?;
        Ok(())
    }

    /// Bypassing strings to the pipe and will be modified by the targeted command.
    /// This is data is in the hole on the masking tape".
    pub fn send_byps(&mut self, msg: String) -> Result<(), errors::ChunkSendError> {
        if self.dryrun {
            // Highlight the string instead of bypassing
            let msg_highlighted: String;
            msg_highlighted = HL[0].to_string() + &msg + HL[1];
            debug!("tx.send => Channle({:?})", msg_highlighted);
            self.tx
                .send(Chunk::Keep(msg_highlighted))
                .map_err(|e| errors::ChunkSendError::Channel(e))?;
            return Ok(());
        }
        if self.solid {
            debug!("tx.send => Solid({:?})", msg);
            self.tx
                .send(Chunk::SHole(msg))
                .map_err(|e| errors::ChunkSendError::Channel(e))?;
            Ok(())
        } else {
            debug!("tx.send => Hole");
            self.tx
                .send(Chunk::Hole)
                .map_err(|e| errors::ChunkSendError::Channel(e))?;
            debug!("stdin => {}[line_end]", msg);
            // FIXME: Marging line_end to the end of the string may improve the performance.
            //        Need benchmarking.
            self.pipe_writer
                .write(msg.as_bytes())
                .map_err(|e| errors::ChunkSendError::Pipe(e))?;
            self.pipe_writer
                .write(&[self.line_end])
                .map_err(|e| errors::ChunkSendError::Pipe(e))?;
            Ok(())
        }
    }

    /// Used as send_keep() but it prevents sending chunk immediately.
    /// Keep the chunk in the buffer until the next Hole is found.
    /// Actually, send_keep() is not called directly but called by buf_send_byps().
    pub fn buf_send_keep(&mut self, msg: String) -> Result<(), errors::ChunkSendError> {
        // append msg to keep_buf
        self.chunk_buf.keep_buf.push_str(&msg);
        match self.chunk_buf.last_target {
            Some(ChunkGroup::Keep) => {
                return Ok(());
            }
            Some(ChunkGroup::Hole) => {
                match self.send_byps(self.chunk_buf.byps_buf.clone()) {
                    Ok(_) => {
                        // clear keep_buf
                        self.chunk_buf.byps_buf.clear();
                        self.chunk_buf.last_target = Some(ChunkGroup::Keep);
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                }
            }
            // Initialize
            None => {
                self.chunk_buf.last_target = Some(ChunkGroup::Keep);
                return Ok(());
            }
        }
    }

    /// Used as send_byps() but it prevents sending chunk immediately.
    /// Keep the chunk in the buffer until the next Keep is found.
    /// Actually, send_byps() is not called directly but called by buf_send_keep().
    pub fn buf_send_byps(&mut self, msg: String) -> Result<(), errors::ChunkSendError> {
        // append msg to byps_buf
        self.chunk_buf.byps_buf.push_str(&msg);
        match self.chunk_buf.last_target {
            Some(ChunkGroup::Keep) => {
                match self.send_keep(self.chunk_buf.keep_buf.clone()) {
                    Ok(_) => {
                        // clear keep_buf
                        self.chunk_buf.keep_buf.clear();
                        self.chunk_buf.last_target = Some(ChunkGroup::Hole);
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                }
            }
            Some(ChunkGroup::Hole) => {
                return Ok(());
            }
            // Initialize
            None => {
                self.chunk_buf.last_target = Some(ChunkGroup::Hole);
                return Ok(());
            }
        }
    }

    // send remaining chunks to be bypassed in the buffer
    pub fn flush_byps(&mut self) -> Result<(), errors::ChunkSendError> {
        if !self.chunk_buf.byps_buf.is_empty() {
            match self.send_byps(self.chunk_buf.byps_buf.clone()) {
                Ok(_) => {
                    // clear keep_buf
                    self.chunk_buf.byps_buf.clear();
                    self.chunk_buf.last_target = None;
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }
        return Ok(());
    }

    pub fn flush_keep(&mut self) -> Result<(), errors::ChunkSendError> {
        if !self.chunk_buf.keep_buf.is_empty() {
            match self.send_keep(self.chunk_buf.keep_buf.clone()) {
                Ok(_) => {
                    // clear keep_buf
                    self.chunk_buf.keep_buf.clear();
                    self.chunk_buf.last_target = None;
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }
        return Ok(());
    }

    /// Notify PipeIntercepter the end of file to exit process
    pub fn send_eof(&self) -> Result<(), errors::ChunkSendError> {
        debug!("tx.send => EOF");
        self.tx
            .send(Chunk::EOF)
            .map_err(|e| errors::ChunkSendError::Channel(e))?;
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
