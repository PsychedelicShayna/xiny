
use clap::{command, ArgGroup, Parser};
use clap_complete::Shell;

use crate::data;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Populates author/version info from Cargo.toml
#[command(group(ArgGroup::new("AlternateOperatingModes")
        .args(&["list", "langs", "set_conf", "get_conf",  "gen_completions", "seek", "check_remote", "sync", "reclone"])
        .multiple(false)))]
#[clap(
    name = "xiny",
    version = "0.1.0",
    about = "A CLI for the LearnXinYMinutes repository."
)]
#[command(group(ArgGroup::new("SubjectGroup").args(&["explicit_subject", "implicit_subject"]).multiple(false).conflicts_with("AlternateOperatingModes")))]
#[command(group(ArgGroup::new("LangRequirements").args(&["explicit_subject", "implicit_subject"]).multiple(false)))]

pub struct CliArgs {
    #[arg(required_unless_present_any(&["AlternateOperatingModes", "explicit_subject"]))]
    #[arg(help = "The subject to view (e.g. bash, python, etc.)")]
    #[arg(value_parser = data::SUBJECTS)]
    #[arg(hide_possible_values(true), value_name = "SUBJECT")]
    #[arg(index(1))]
    pub implicit_subject: Option<String>,

    /// The subject to view (e.g. bash, python, etc.)
    #[arg(value_parser = data::SUBJECTS)]
    #[arg(hide_possible_values(true), value_name = "SUBJECT-EXPLICIT")]
    #[arg(
        long = "subject",
        short = 's',
        help = "The subject to view (e.g. bash, python, etc.). Explicit form of the positional argument."
    )]
    pub explicit_subject: Option<String>,

    /// List all available subjects.
    #[arg(
        short,
        long,
        help = "List all available subjects. Output is filtered by the language set with -L (defaults to English)"
    )]
    pub list: bool,

    /// List available languages.
    #[arg(long, help = "List all available translated language names and tags.")]
    pub langs: bool,

    /// Specify a language translation.
    #[arg(
        short = 'L',
        long,
        help = "Specify a language translation in either tag or name format. Also filters the output of --list"
    )]
    #[arg(requires_if("refind", "explicit_subject"))]
    #[arg(requires_if("refind", "implicit_subject"))]
    #[arg(requires_if("refind", "list"))]
    #[arg(value_parser = data::LANGUAGES)]
    #[arg(value_name("LANGUAGE"))]
    pub lang: Option<String>,

    /// Use regex to filter content within the subject.
    #[arg(short = 'r', long)]
    #[arg(requires = "SubjectGroup")]
    #[arg(value_name = "PATTERN")]
    pub regex: Option<String>,

    #[arg(
        short = 'R',
        long,
        help = "Search every single Markdown file, using the subject string as the expression."
    )]
    #[arg(requires_if("rip", "explicit_subject"))]
    #[arg(requires_if("rip", "implicit_subject"))]
    pub rip: bool,

    #[arg(long, short, help = "Output RegEx matches in Vimgrep format.")]
    #[arg(requires = "regex")]
    pub vimgrep: bool,

    #[arg(
        long,
        help = "Check if the local repository is up-to-date without making any changes."
    )]
    #[arg(conflicts_with("SubjectGroup"))]
    pub check_remote: bool,

    /// Sync with the remote repository if it's behind.
    #[arg(
        long,
        short = 'p',
        help = "Pulls changes from the remote repository if the local repository is behind."
    )]
    #[arg(conflicts_with("SubjectGroup"))]
    pub sync: bool,

    #[arg(
        long = "reclone",
        short = 'P',
        help = "Purges the local repository and re-clones the remote repository."
    )]
    #[arg(conflicts_with("SubjectGroup"))]
    pub reclone: bool,

    /// Generate shell completions and output to stdout.
    #[arg(
        long = "gencompletions",
        help = "Generate shell completions for the specified shell and output to stdout."
    )]
    #[arg(num_args(1))]
    #[arg(conflicts_with("SubjectGroup"))]
    #[arg(value_name("SHELL"))]
    pub gen_completions: Option<Shell>,

    /// Seek through the document interactively (using fuzzy finder).
    #[arg(
        long,
        short = 'i',
        help = "Interactively fuzzy find the subject document."
    )]
    #[arg(requires_if("seek", "explicit_subject"))]
    #[arg(requires_if("seek", "implicit_subject"))]
    pub seek: bool,

    /// Only output the file path of the subject document.
    #[arg(
        short,
        long,
        help = "Only output the file path of the Markdown document."
    )]
    #[arg(requires_if("seek", "explicit_subject"))]
    #[arg(requires_if("seek", "implicit_subject"))]
    pub r#where: bool,

    // Change a config key with a specific value.
    #[arg(
        long = "get-conf",
        help = "List the config values, or a specific config key."
    )]
    #[arg(num_args(0..2))]
    #[arg(value_name("KEY"))]
    pub get_conf: Option<Vec<String>>,

    // Change a config key with a specific value.
    #[arg(
        short = 'c',
        long = "set-conf",
        help = "Set a configuration key to a new value."
    )]
    #[arg(num_args(2))]
    #[arg(value_names(&["KEY", "VALUE"]))]
    pub set_conf: Option<Vec<String>>,
}
