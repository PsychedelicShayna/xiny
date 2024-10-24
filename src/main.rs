use std::fs::{self, DirEntry, File, OpenOptions};

use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::{self, Path, PathBuf};
use std::process::{self, abort, exit, Child, ChildStderr, ChildStdin, ChildStdout, Output, Stdio};
use std::sync::Arc;

use anyhow::{self as ah, Context};
use clap::builder::PossibleValuesParser;
use clap::{CommandFactory, Parser};
use lazy_static::lazy_static;

use clap_complete::*;

pub mod args;
pub mod conf;
pub mod lang;
pub mod repo;
pub mod shell;
pub mod xiny;
pub mod data;

use crate::{args::*, conf::*, lang::*, repo::*, shell::*, xiny::*};

fn main() -> ah::Result<()> {
    let mut config = ConfigFile::new().unwrap();
    let mut repo = Repo::new(config.values.repo, config.values.branch).unwrap();

    let cli = CliArgs::parse();

    if let Some(shell) = cli.gen_completions {
        clap_complete::aot::generate(shell, &mut CliArgs::command(), "xiny", &mut io::stdout());
        exit(0);
    }

    if cli.sync || cli.reclone {
        println!("Comparing commit hashes..");
        let changed = repo.sync(cli.reclone).unwrap();

        if cli.reclone {
            println!("Local repository has been purged, and the remote repository recloned.");
        } else if changed {
            println!("Repository was out of date, synced successfully.");
        } else {
            println!("Repository was up to date, no changes made.");
        }

        exit(0);
    }

    exit(0);
}
