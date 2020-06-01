use super::token::Token;
use std::error;
use std::fmt;
use std::sync::mpsc;

const PIPE_ERROR_MSG: &'static str = "Output of given command is exhausted";

pub enum PipeReceiveError {
    EndOfFd,
    Io(std::io::Error),
}

impl fmt::Display for PipeReceiveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PipeReceiveError::EndOfFd => write!(f, "{}", PIPE_ERROR_MSG),
            PipeReceiveError::Io(ref e) => write!(f, "IO error: {}", e),
        }
    }
}

impl error::Error for PipeReceiveError {
    fn description(&self) -> &str {
        match *self {
            PipeReceiveError::EndOfFd => PIPE_ERROR_MSG,
            PipeReceiveError::Io(_) => "IO error",
        }
    }
}

impl fmt::Debug for PipeReceiveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PipeReceiveError::EndOfFd => write!(f, "{}", PIPE_ERROR_MSG),
            PipeReceiveError::Io(ref e) => write!(f, "IO error: {}", e),
        }
    }
}

pub enum TokenSendError {
    Channel(mpsc::SendError<Token>),
    Pipe(std::io::Error),
}

impl fmt::Display for TokenSendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TokenSendError::Channel(ref err) => write!(f, "Channel error: {}", err),
            TokenSendError::Pipe(ref err) => write!(f, "IO error: {}", err),
        }
    }
}

impl error::Error for TokenSendError {
    fn description(&self) -> &str {
        match *self {
            TokenSendError::Channel(_) => "Channel error",
            TokenSendError::Pipe(_) => "IO error",
        }
    }
}

impl fmt::Debug for TokenSendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TokenSendError::Channel(ref e) => write!(f, "Channel error: {}", e),
            TokenSendError::Pipe(ref e) => write!(f, "IO error: {}", e),
        }
    }
}

const STDIN_ERROR_MSG: &'static str = "Failed to get FD of stdin for given command";
const STDOUT_ERROR_MSG: &'static str = "Failed to get FD of stdout for given command";

pub enum SpawnError {
    StdinOpenFailed,
    StdoutOpenFailed,
    Io(std::io::Error),
}

impl fmt::Display for SpawnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SpawnError::StdinOpenFailed => write!(f, "{}", STDIN_ERROR_MSG),
            SpawnError::StdoutOpenFailed => write!(f, "{}", STDOUT_ERROR_MSG),
            SpawnError::Io(ref err) => write!(f, "IO error: {}", err),
        }
    }
}

impl error::Error for SpawnError {
    fn description(&self) -> &str {
        match *self {
            SpawnError::StdinOpenFailed => STDIN_ERROR_MSG,
            SpawnError::StdoutOpenFailed => STDOUT_ERROR_MSG,
            SpawnError::Io(_) => "IO error",
        }
    }
}

impl fmt::Debug for SpawnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SpawnError::StdinOpenFailed => write!(f, "{}", STDIN_ERROR_MSG),
            SpawnError::StdoutOpenFailed => write!(f, "{}", STDOUT_ERROR_MSG),
            SpawnError::Io(ref err) => write!(f, "IO error: {}", err),
        }
    }
}
