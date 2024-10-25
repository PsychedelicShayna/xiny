use std::io;
use std::process::exit;

use anyhow::{self as ah, Context};
use clap::{CommandFactory, Parser};

pub mod ansi;
pub mod args;
pub mod conf;
pub mod data;
pub mod iana;
pub mod lang;
pub mod render;
pub mod repo;
pub mod shell;
pub mod table;
pub mod xiny;

use crate::{args::*, conf::*, lang::*, repo::*, shell::*, xiny::*};

fn get_terminal_size() -> (usize, usize) {
    let (width, height) = term_size::dimensions().unwrap_or((80, 24));
    (width, height)
}

/// Handles the --set-conf argument, which sets a configuration value.
fn handle_set_conf(set_conf: &Vec<String>, conf: &mut ConfigFile) -> ah::Result<()> {
    if !set_conf.len() == 2 {
        ah::bail!(
            "Invalid number of key-value pairs provided, expected 2, got {}",
            set_conf.len()
        );
    }

    let key = set_conf[0].as_str();
    let value = set_conf[1].as_str();

    if !Config::is_valid_key(key) {
        eprintln!("Config key '{}' does not exist.", key);
        exit(1);
    }

    conf.values
        .set_value(key, value)
        .context("ConfigFile set_value")?;

    conf.write_changes().context("ConfigFile write_changes")?;
    println!("Config key '{}' set to '{}'", key, value);

    Ok(())
}

/// Handles the --get-conf argument, which retrieves configuration values.
fn handle_get_conf(get_conf: &Vec<String>, conf: &mut ConfigFile) -> ah::Result<()> {
    if get_conf.is_empty() {
        println!("{}", conf.values.dump());
    } else if get_conf.len() == 1 {
        let get_conf = get_conf[0].as_str();

        let value = conf.values.get_value(get_conf).unwrap_or_else(|| {
            eprintln!("Config get_conf not found: {}", get_conf);
            exit(1);
        });

        println!("{}: {}", get_conf, value);
    }

    Ok(())
}

fn main() -> ah::Result<()> {
    let mut config = ConfigFile::new().unwrap();
    let repo = Repo::new(&config.values.repo, &config.values.branch).unwrap();
    let xiny = XinY::new(&repo.repo_dir).context("XinY::new")?;

    let cli = CliArgs::parse();

    // set-conf ---------------------------------------------------------------
    {
        if let Some(cfg) = &cli.set_conf {
            handle_set_conf(&cfg, &mut config)?;
            exit(0);
        }
    }

    // get-conf ---------------------------------------------------------------
    {
        if let Some(cfg) = &cli.get_conf {
            handle_get_conf(&cfg, &mut config)?;
            exit(0);
        }
    }

    // gencompletions ---------------------------------------------------------
    {
        if let Some(shell) = cli.gen_completions {
            clap_complete::aot::generate(shell, &mut CliArgs::command(), "xiny", &mut io::stdout());
            exit(0);
        }
    }

    // sync & reclone ---------------------------------------------------------
    {
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
    }

    // check-remote -----------------------------------------------------------
    {
        if cli.check_remote {
            let changed: bool = repo.is_remote_ahead().context("Repo::is_remote_ahead")?;

            if changed {
                println!("Local repository is out-of-date; remote repository is ahead.");
            } else {
                println!("Local repository is up-to-date with the remote repository.");
            }
            exit(0);
        }
    }

    // list -------------------------------------------------------------------
    {
        if cli.list {
            let mut subjects = xiny.available_subjects();

            if let Some(lang) = cli.lang {
                let language = Language::from_tag(&lang).unwrap_or_else(|e| {
                    eprintln!("Invalid language tag: {}, err: {:?}", lang, e);
                    exit(1);
                });

                subjects = subjects
                    .into_iter()
                    .filter(|s| xiny.get_subject_in(s, &language).is_some())
                    .collect();
            }

            let longest = subjects.iter().map(|s| s.len()).max().unwrap_or(0);
            let padding = longest + 2;

            let (w, _) = get_terminal_size();
            let wrap_limit = w / padding;

            let mut wrap_counter = 1;

            for subject in &subjects {
                print!("{:<padding$}", subject);

                if wrap_counter % wrap_limit == 0 {
                    println!();
                    wrap_counter = 0;
                }

                wrap_counter += 1;
            }

            exit(0);
        }
    }

    // langs ------------------------------------------------------------------
    {
        if cli.langs {
            let langs = xiny.get_available_languages();

            let longest = langs.iter().map(|l| l.tag.len()).max().unwrap_or(0);
            let padding = longest + 2;

            let (w, _) = get_terminal_size();
            let wrap_limit = w / padding;

            let mut wrap_counter = 1;

            for lang in langs {
                print!("{:<padding$}", lang.tag);

                if wrap_counter % wrap_limit == 0 {
                    println!();
                    wrap_counter = 0;
                }

                wrap_counter += 1;
            }

            exit(0);
        }
    }

    // subject printing -------------------------------------------------------
    let mut subject_name: Option<String> = None;

    if let Some(subject) = cli.explicit_subject {
        subject_name = Some(subject);
    } else if let Some(subject) = cli.implicit_subject {
        subject_name = Some(subject);
    }

    if let Some(subject) = subject_name {
        let subject = xiny.get_subject(&subject).unwrap_or_else(|| {
            eprintln!("Subject not found: {}", subject);
            exit(1);
        });

        let lang = match cli.lang {
            Some(lang) => Language::from_tag(&lang).unwrap_or_else(|e| {
                eprintln!("Invalid language tag: {}, err: {:?}", lang, e);
                exit(1);
            }),

            None => Language::from_tag("en-us").unwrap(),
        };

        let document_path = subject.get_in_language(&lang).unwrap_or_else(|| {
            eprintln!("Subject not available in language: {:?}", lang);
            exit(1);
        });

        let renderer = (!config.values.renderer.is_empty()).then_some(config.values.renderer);

        if let Err(e) = render::print_document(document_path, renderer.as_deref()) {
            eprintln!("Error rendering document: {:?}", e);
            exit(1);
        }

        exit(0);
    }

    exit(0);
}
