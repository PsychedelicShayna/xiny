use std::fs::{self, DirEntry, File, OpenOptions};

use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::{self, Path, PathBuf};
use std::process::{self, abort, exit, Child, ChildStderr, ChildStdin, ChildStdout, Output, Stdio};

use anyhow::{self as ah, Context};
use clap::{CommandFactory, Parser};
use lazy_static::lazy_static;

use crate::{args::*, conf::*, lang::*, repo::*, shell::*, xiny::*};

pub fn testing(arg: &str) -> Result<String, ()> {
    Ok(String::new())
}

lazy_static! {
    pub static ref VALID_SUBJECT: Vec<SubjectName> = {
        let v: Vec<SubjectName> = vec![];
        v
    };
}

pub fn handle_cli_args() -> ah::Result<()> {
    let mut config = ConfigFile::new().unwrap();
    let mut repo = Repo::new(config.values.repo, config.values.branch).unwrap();
    repo.sync(true).unwrap();

    let mut xiny = XinY::new(&repo.repo_dir).context("XinY::new")?;

    let subjects = xiny.available_subjects();

    let bash_english = xiny.get_subject_in("bash", &Language::from_tag("en-us").unwrap());

    println!("{:?}", bash_english);

    // println!("Available subjects #{}", subjects.len());
    //
    // for subject in subjects {
    //     let available_in = xiny.subject_available_in(subject);
    //     let lang_str = available_in
    //         .iter()
    //         .map(|lang| format!("{} ({})", lang.language, lang.region))
    //         .collect::<Vec<String>>()
    //         .join(", ");
    //
    //     println!("{}, available in {}", subject, lang_str);
    // }

    exit(0);

    let cli = CliArgs::parse();

    let mut command = CliArgs::command();

    let mut config = ConfigFile::new().unwrap();
    let mut repo = Repo::new(config.values.repo, config.values.branch).unwrap();

    if let Some(shell) = cli.completions {
        clap_complete::generate(shell, &mut CliArgs::command(), "xiny", &mut io::stdout());
        exit(0);
    }

    if cli.sync || cli.fsync {
        println!("Comparing commit hashes..");
        let changed = repo.sync(cli.fsync).unwrap();

        if changed {
            println!("Repository was out of date, synced successfully.");
        } else {
            println!("Repository was up to date, no changes made.");
        }

        exit(0);
    }

    exit(0);
}
