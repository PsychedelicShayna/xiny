/// Quality of life when invoking shell commmands.
use std::{
    io::{BufReader, Read},
    process::{ChildStderr, ChildStdout, Command, Stdio},
};

use anyhow::{self as ah, Context};

pub fn shell(command: &str, arguments: Vec<&str>) -> ah::Result<(String, String)> {
    let mut cmd = Command::new(command);

    for arg in arguments {
        cmd.arg(arg);
    }

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("shell spawning command")?;

    let mut buffer = String::new();

    let child_stdout: ChildStdout = child
        .stdout
        .take()
        .ok_or(ah::anyhow!("Failed to take stdout from child process"))?;

    let mut reader = BufReader::new(child_stdout);
    reader
        .read_to_string(&mut buffer)
        .context("shell reading stdout")?;

    let stdout: String = buffer.clone();
    buffer.clear();

    let child_stderr: ChildStderr = child
        .stderr
        .take()
        .ok_or(ah::anyhow!("Failed to take stderr from child process"))?;

    let mut reader = BufReader::new(child_stderr);
    reader
        .read_to_string(&mut buffer)
        .context("shell reading stderr")?;

    let stderr: String = buffer.clone();
    buffer.clear();

    child.kill().context("shell killing child process")?;

    Ok((stdout, stderr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell() {
        let (stdout, stderr) = shell("echo", vec!["Hello,", "world!"].into()).unwrap();
        assert_eq!(stdout, "Hello, world!\n");
        assert_eq!(stderr, "");

        let (stdout, stderr) = shell("git", vec!["--git-dir", "/dev/null", "status"]).unwrap();
        assert_eq!(stdout, "");
        assert_eq!(stderr, "fatal: not a git repository: '/dev/null'\n");
    }
}
