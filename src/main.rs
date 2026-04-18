use std::io::{self, Write};
use std::process::exit;

use anyhow::{self as ah, Context};
use argparse::CliArgs;
use clap::{CommandFactory, Parser};
use config::parser::*;

pub mod argparse;
pub mod config;
pub mod database;
pub mod language;
pub mod render;
pub mod search;
pub mod tui;
pub mod utils;

use database::database::XinY;
use database::repository::Repo;
use language::language::Language;
use search::engines::terms::TermSearch;
use tui::event_loop::{self};

fn handle_set_conf(set_conf: &Vec<String>, conf: &mut ConfigFile) -> ah::Result<()> {
    if set_conf.len() != 2 {
        ah::bail!(
            "Expected 2 arguments (KEY VALUE), got {}",
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

fn handle_get_conf(get_conf: &Vec<String>, conf: &mut ConfigFile) -> ah::Result<()> {
    if get_conf.is_empty() {
        println!("{}", conf.values.dump());
    } else if get_conf.len() == 1 {
        let key = get_conf[0].as_str();
        let value = conf.values.get_value(key).unwrap_or_else(|| {
            eprintln!("Config key not found: {}", key);
            exit(1);
        });
        println!("{}: {}", key, value);
    }

    Ok(())
}

fn main() -> ah::Result<()> {
    let mut config = ConfigFile::new().unwrap();
    let cli = CliArgs::parse();

    if let Some(shell) = cli.gen_completions {
        clap_complete::aot::generate(shell, &mut CliArgs::command(), "xiny", &mut io::stdout());
        exit(0);
    }

    if let Some(cfg) = &cli.set_conf {
        handle_set_conf(cfg, &mut config)?;
        exit(0);
    }

    if let Some(cfg) = &cli.get_conf {
        handle_get_conf(cfg, &mut config)?;
        exit(0);
    }

    let repo = Repo::new(&config.values.repo, &config.values.branch).unwrap();

    if cli.sync || cli.reclone {
        println!("Comparing commit hashes..");
        let changed = repo.sync(cli.reclone).unwrap();

        if cli.reclone {
            println!("Local repository has been purged and recloned.");
        } else if changed {
            println!("Repository was out of date, synced successfully.");
        } else {
            println!("Repository is up to date, no changes made.");
        }

        exit(0);
    }

    if cli.check_remote {
        if !repo.git_dir.exists() {
            eprintln!("Database has not been cloned yet. Run `xiny --sync` to clone it.");
            exit(1);
        }

        let changed = repo.is_remote_ahead().context("Repo::is_remote_ahead")?;

        if changed {
            println!("Local repository is out-of-date; remote repository is ahead.");
        } else {
            println!("Local repository is up-to-date with the remote repository.");
        }

        exit(0);
    }

    if !repo.git_dir.exists() {
        print!(
            "Documentation database not found at {}.\nClone it now? [Y/n] ",
            repo.repo_dir.display()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let answer = input.trim().to_lowercase();

        if answer.is_empty() || answer == "y" || answer == "yes" {
            repo.sync(false)?;
        } else {
            println!("Skipping. Run `xiny --sync` when you're ready to clone.");
            exit(0);
        }
    }

    let xiny = XinY::new(&repo.repo_dir).context("XinY::new")?;

    if cli.list {
        let mut subjects = xiny.available_subjects();

        if let Some(ref lang) = cli.lang {
            let language = Language::from_tag(lang).unwrap_or_else(|e| {
                eprintln!("Invalid language tag: {}, err: {:?}", lang, e);
                exit(1);
            });
            subjects.retain(|s| xiny.get_subject_in(s, &language).is_some());
        }

        if subjects.is_empty() {
            eprintln!("No subjects found. The database may be empty try `xiny --sync`.");
            exit(1);
        }

        let longest = subjects.iter().map(|s| s.len()).max().unwrap_or(0);
        let padding = longest + 2;
        let w = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
        let wrap_limit = (w / padding).max(1);
        let mut wrap_counter = 1;

        for subject in &subjects {
            print!("{:<padding$}", subject);
            if wrap_counter % wrap_limit == 0 {
                println!();
                wrap_counter = 0;
            }
            wrap_counter += 1;
        }
        println!();
        exit(0);
    }

    if cli.langs {
        let langs = xiny.get_available_languages();

        if langs.is_empty() {
            eprintln!("No languages found. The database may be empty try `xiny --sync`.");
            exit(1);
        }

        let longest = langs.iter().map(|l| l.tag.len()).max().unwrap_or(0);
        let padding = longest + 2;
        let w = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);
        let wrap_limit = (w / padding).max(1);
        let mut wrap_counter = 1;

        for lang in langs {
            print!("{:<padding$}", lang.tag);
            if wrap_counter % wrap_limit == 0 {
                println!();
                wrap_counter = 0;
            }
            wrap_counter += 1;
        }
        println!();
        exit(0);
    }

    let subject_name: Option<String> = cli.explicit_subject.or(cli.implicit_subject);

    if let Some(subject) = subject_name {
        let subject = xiny.get_subject(&subject).unwrap_or_else(|| {
            eprintln!(
                "Subject not found: {}. Try `xiny --list` to see available subjects.",
                subject
            );
            exit(1);
        });

        let lang = match cli.lang.as_deref() {
            Some(lang) => Language::from_tag(lang).unwrap_or_else(|e| {
                eprintln!("Invalid language tag: {}, err: {:?}", lang, e);
                exit(1);
            }),
            None => Language::from_tag("en-us").unwrap(),
        };

        let document_path = subject.get_in_language(&lang).unwrap_or_else(|| {
            eprintln!("Subject not available in language: {:?}", lang);
            exit(1);
        });

        if cli.r#where {
            println!("{}", document_path.display());
            exit(0);
        }

        if cli.find.is_some() {
            eprintln!("--find is not yet implemented.");
            exit(1);
        }

        let renderer = (!config.values.renderer.is_empty()).then_some(config.values.renderer);

        if cli.interactive {
            event_loop::event_loop::<TermSearch>(document_path.to_path_buf())?;
        } else if let Err(e) = render::print_document(document_path, renderer.as_deref()) {
            eprintln!("Error rendering document: {:?}", e);
            exit(1);
        }

        exit(0);
    }

    exit(0);
}
