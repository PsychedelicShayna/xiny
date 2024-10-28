// This is a utils.rs file in disguise. I just couldn't resist the pun.

use std::{fs::OpenOptions, io::{BufRead, BufReader}, path::PathBuf};

use anyhow::{self as ah, Context};

pub fn count_file_lines(path: &PathBuf) -> ah::Result<usize> {
    let file = OpenOptions::new()
        .read(true)
        .open(&path)
        .context("count_file_lines opening file")?;

    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}
