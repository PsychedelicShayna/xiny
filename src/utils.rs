use std::process::{ChildStderr, ChildStdout, Command, Stdio};
use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Read},
    path::PathBuf,
};

use anyhow::{self as ah, Context};
use crossterm::{
    self as ct,
    event::{Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled},
};

use term_size;

#[derive(Debug, Clone, Default)]
pub struct Dimensions {
    width: usize,
    height: usize,
}

pub fn crc32(data: &Vec<u8>) -> String {
    let result = data.iter().fold(0, |acc, b| (acc << 8) ^ *b as u32);
    format!("{:08x}", result)
}

impl Dimensions {
    pub fn from_terminal() -> ah::Result<Self> {
        let (width, height) = term_size::dimensions()
            .ok_or_else(|| ah::anyhow!("Failed to get terminal dimensions"))?;

        Ok(Self { width, height })
    }

    pub fn unpack(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn rows(&self) -> usize {
        self.height
    }

    pub fn cols(&self) -> usize {
        self.width
    }
}

/// Count the amount of lines in a file.
pub fn count_file_lines(path: &PathBuf) -> ah::Result<usize> {
    let file = OpenOptions::new()
        .read(true)
        .open(path)
        .context("count_file_lines opening file")?;

    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}

/// Gets a single key press from the terminal.
pub fn get_input() -> ah::Result<KeyCode> {
    if !is_raw_mode_enabled()? {
        enable_raw_mode()?
    }

    let key_event: KeyCode = match ct::event::read() {
        Ok(Event::Key(ke)) => ke.code,
        _ => KeyCode::Null,
    };

    if is_raw_mode_enabled()? {
        disable_raw_mode()?;
    }

    Ok(key_event)
}

/// Quality of life when invoking shell commmands.

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
