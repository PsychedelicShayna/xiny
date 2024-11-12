use std::process::exit;
use std::{io, time::Instant};

use anyhow::{self as ah, Context};
use argparse::CliArgs;
use clap::{CommandFactory, Parser};
use config::parser::*;
use crossterm as ct;

pub mod argparse;
pub mod config;
pub mod database;
pub mod language;
pub mod render;
// pub mod search;
pub mod search_engines;

pub mod utils;
// pub mod tui;
pub mod debug;
pub mod tui;


use database::database::XinY;
use database::repository::Repo;
use fuzzy_matcher::FuzzyMatcher;
use language::language::Language;
use search_engines::SearchEngine;
use tui::point::Point;
use tui::tui::{Tui, TuiOptions};
use utils::percentage_of_columns;
// use search::engines::{fuzzy::FuzzySearch, regex::RegexSearch, terms::TermSearch};
// use tui::event_loop::{self, TuiState};

// fn get_terminal_size() -> (usize, usize) {
//     let (width, height) = term_size::dimensions().unwrap_or((80, 24));
//     (width, height)
// }

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
    #[cfg(debug_assertions)]
    unsafe {
        debug::START_TIME = Some(Instant::now());
    }

    let mut config = ConfigFile::new().unwrap();

    let repo = Repo::new(&config.values.repo, &config.values.branch).unwrap();

    let xiny = XinY::new(&repo.repo_dir).context("XinY::new")?;
    let cli = CliArgs::parse();

    // set-conf ---------------------------------------------------------------
    {
        if let Some(cfg) = &cli.set_conf {
            handle_set_conf(cfg, &mut config)?;
            exit(0);
        }
    }

    // get-conf ---------------------------------------------------------------
    {
        if let Some(cfg) = &cli.get_conf {
            handle_get_conf(cfg, &mut config)?;
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

                subjects.retain(|s| xiny.get_subject_in(s, &language).is_some());
            }

            let longest = subjects.iter().map(|s| s.len()).max().unwrap_or(0);
            let padding = longest + 2;

            let Ok((cols, _)) = ct::terminal::size() else {
                exit(1)
            };

            let wrap_limit = cols as usize / padding;

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

            let (rows, _) = ct::terminal::size()?;
            let wrap_limit = (rows as usize / padding) % 8;

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

    // Subject Related -------------------------------------------------------
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

        // What to do with the document now ----------------------------------

        // Just print the path
        if cli.r#where {
            println!("{}", document_path.display());
            exit(0);
        }

        let renderer = (!config.values.renderer.is_empty()).then_some(config.values.renderer);

        if cli.interactive {
            let mut opts = TuiOptions::default();
            opts.search_engine = SearchEngine::Fuzzy;
            opts.rsi_mode = cli.rsi;
            let mut tui = Tui::new(opts)?;
            tui.start()?;

            // if cli.fuzzy {
            //     event_loop::event_loop::<FuzzySearch>(document_path.to_path_buf())?;
            // } else if cli.regex {
            //     event_loop::event_loop::<RegexSearch>(document_path.to_path_buf())?;
            // } else {
            //     event_loop::event_loop::<TermSearch>(document_path.to_path_buf())?;
            // }
        } else if let Err(e) = render::print_document(document_path, renderer.as_deref()) {
            eprintln!("Error rendering document: {:?}", e);
            exit(1);
        }

        exit(0);
    }

    exit(0);
}
