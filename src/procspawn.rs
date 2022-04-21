use super::errors;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use log::debug;

pub fn exec_cmd(
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
