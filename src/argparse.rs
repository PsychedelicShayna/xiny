use clap::{command, ArgGroup, Parser};
use clap_complete::Shell;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)] // Populates author/version info from Cargo.toml
#[command(group(ArgGroup::new("AlternateOperatingModes")
        .args(&["list", "langs", "set_conf", "get_conf",  "gen_completions", "check_remote", "sync", "reclone"])
        .multiple(false)))]
#[clap(
    name = "xiny",
    version = "0.1.0",
    about = "A CLI for the LearnXinYMinutes repository."
)]
#[command(group(ArgGroup::new("any_subject").args(&["explicit_subject", "implicit_subject"]).multiple(false).conflicts_with("AlternateOperatingModes")))]
#[command(group(ArgGroup::new("LangRequirements").args(&["explicit_subject", "implicit_subject"]).multiple(false)))]
pub struct CliArgs {
    // IMPLICIT SUBJECT
    // ================================================================================================================
    #[arg(
        required_unless_present_any(&["AlternateOperatingModes", "explicit_subject"]),
        help = "The subject to view (e.g. bash, python, etc.)",
        value_parser = SUBJECTS,
        hide_possible_values(true),
        index(1),
    )]
    pub implicit_subject: Option<String>,

    // EXPLICIT SUBJECT
    // ================================================================================================================
    #[arg(
        long = "subject",
        short = 's',
        value_parser = SUBJECTS,
        hide_possible_values(true),
        value_name = "SUBJECT",
        help = "The subject to view (e.g. bash, python, etc.). Explicit form of the positional argument."
    )]
    pub explicit_subject: Option<String>,

    // LIST
    // ================================================================================================================
    #[arg(
        short,
        long,
        help = "Display a table of every subject that is available in the current language."
    )]
    pub list: bool,

    // LANG
    // ================================================================================================================
    #[arg(
        long,
        short = 'L',
        help = "Sets the desired language in IANA LST format (see --help)",
        long_help = "Sets the desired language. If a translation exists, the corresponding document will be
used. Also filters the output of --list to only include subjects available in the set
language. Available languages viewable with --langs, and IANA tags are treated case
insensitively. I recommend sourcing --gencompletions so you can just tab complete the
languages.",
        value_parser = LANGUAGES, 
        hide_possible_values = true,
        value_name("LANGUAGE-REGION"),
        default_value = "en-us"
    )]
    pub lang: Option<String>,

    // LANGS
    // ================================================================================================================
    #[arg(long, help = "List all available translated language names and tags.")]
    pub langs: bool,

    // FIND
    // ================================================================================================================
    #[arg(
        short,
        long,
        requires_if("find", "explicit_subject"),
        requires_if("find", "implicit_subject"),
        num_args(1..),
        value_name = "TERMS",
        help = "Searches the subject document for the provivded terms and displays surrounding lines (see --help)",
       long_help = "Searches the subject document for the provivded terms and displays lines surrounding the
match. The default behavior is quite basic; the document is searched line by line, and
the line which contains the most search terms is selected as the match. The behavior can
be configured using:

     Flags               Options                    
                                                          
    --interactive        --context (default: 6)            
    --vimgrep            --matches (default: 1)            
    --regex              --case    (default: insensitive)  
    --fuzzy"
    )]
    pub find: Option<Vec<String>>,

    // CONTEXT
    // ================================================================================================================
    #[arg(
        long,
        short = 'C',
        requires("find"),
        conflicts_with("interactive"),
        default_value = "6",
        num_args(1),
        value_name = "LINES",
        help = "The number of lines to display before and after a --find match.",
        long_help = "The number of lines to display before and after a --find match. To clarify, a value of 6
means you'll see 13 lines. That's 6 before the match, 6 after the match, plus the match
itself; 6+6+1 = 13. This does not mean the total line count but how many lines before and
ater the match. That means that the line number must be divisible by 2. You cannot have 5
lines before and 6 lines after. The context is always centered around the match. Passing
3 would not work, but 4 would. If you pass 0, then the entire document is displayed, the
only difference being the match is highlighted."
    )]
    pub context: Option<usize>,

    // MATCHES
    // ================================================================================================================
    #[arg(
        long,
        short,
        requires("find"),
        conflicts_with("interactive"),
        default_value = "1",
        num_args(1),
        value_name = "AMOUNT",
        help = "The number of --find matches to display. If set to 0, all matches are displayed."
    )]
    pub matches: Option<usize>,

    // INTERACTIVE
    // ================================================================================================================
    #[arg(
        short,
        long,
        requires_if("interactive", "explicit_subject"),
        requires_if("interactive", "implicit_subject"),
        conflicts_with("find"),
        help = "An interactive version of find; type, see and select matches interactively in a TUI
popup (see --help)"
    )]
    pub interactive: bool,

    // REGEX
    // ================================================================================================================
    #[arg(
        short,
        long,
        requires_if("regex", "find"),
        requires_if("regex", "interactive"),
        conflicts_with("fuzzy"),
        help = "Enable Regex mode for --find; all terms will be treated as regular expressions (logical AND chain) --help for more info.",
        long_help = "Enable Regex mode for --find; all terms will be treated as regular expressions. Each
expression will be attempted on each line, and if all expressions pass, then the line is
added to the list of matches; a logical AND across all expressions. When several lines
pass the same expression(s), then the relevancy is determined by the length of the
matched content. Shorter content is treated as more relevant. The logic being that the
excess content does not need to exist for any match to be able to exist. The shorter
content matched the same expression more tightly. If fuzzy imprecise matching is what you
need, then --fuzzy may be a better fit."
    )]
    pub regex: bool,

    // FUZZY
    // ================================================================================================================
    #[arg(
        short = 'z',
        long,
        requires_if("fuzzy", "find"),
        requires_if("fuzzy", "interactive"),
        conflicts_with("regex"),
        help = "Enable FuzzyFind mode for --find; terms will be concatenated into one single query, and
matches are derived from similarity."
    )]
    pub fuzzy: bool,

    // FUZZY
    // ================================================================================================================
    #[arg(
        long,
        conflicts_with("any_subject"),
        help = "Check if the local repository is up-to-date without making any changes."
    )]
    pub check_remote: bool,

    // SYNC
    // ================================================================================================================
    #[arg(
        long,
        short = 'p',
        conflicts_with("any_subject"),
        help = "Pulls changes from the remote repository if the local repository is behind."
    )]
    pub sync: bool,

    // RECLONE
    // ================================================================================================================
    #[arg(
        long = "reclone",
        short = 'P',
        conflicts_with("any_subject"),
        help = "Purges the local repository and re-clones the remote repository."
    )]
    pub reclone: bool,

    // GENCOMPLETIONS
    // ================================================================================================================
    #[arg(
        long = "gencompletions",
        num_args(1),
        conflicts_with("any_subject"),
        value_name("SHELL"),
        help = "Generate shell completions for the specified shell and output to stdout."
    )]
    pub gen_completions: Option<Shell>,

    // WHERE
    // ================================================================================================================
    #[arg(
        short,
        long,
        requires_if("where", "explicit_subject"),
        requires_if("where", "implicit_subject"),
        help = "Only output the file path of the Markdown document."
    )]
    pub r#where: bool,

    // GET-CONF
    // ================================================================================================================
    #[arg(
        long = "get-conf",
        num_args(0..2),
        value_name("KEY"),
        help = "List the config values, or a specific config key."
    )]
    pub get_conf: Option<Vec<String>>,

    // SET-CONF
    // ================================================================================================================
    #[arg(
        short = 'c',
        long = "set-conf",
        num_args(2),
        value_names(&["KEY", "VALUE"]),
        help = "Set a configuration key to a new value."
    )]
    pub set_conf: Option<Vec<String>>,
}

const LANGUAGES: [&str; 38] = [
    "ar-ar", "be-by", "bg-bg", "ca-es", "cs-cz", "de-de", "el-gr", "en-us", "es-es", "fa-ir",
    "fi-fi", "fr-fr", "he-he", "hi-in", "hu-hu", "id-id", "it-it", "ja-jp", "ko-kr", "lt-lt",
    "ms-my", "nl-nl", "no-nb", "pl-pl", "pt-br", "pt-pt", "ro-ro", "ru-ru", "sk-sk", "sl-si",
    "sv-se", "ta-in", "th-th", "tr-tr", "uk-ua", "vi-vn", "zh-cn", "zh-tw",
];

const SUBJECTS: [&str; 187] = [
    "ada",
    "amd",
    "angularjs",
    "ansible",
    "apl",
    "arturo",
    "asciidoc",
    "assemblyscript",
    "asymptotic-notation",
    "ats",
    "awk",
    "ballerina",
    "bash",
    "bc",
    "bf",
    "bqn",
    "c++",
    "c",
    "chapel",
    "chicken",
    "citron",
    "clojure-macros",
    "clojure",
    "cmake",
    "cobol",
    "coffeescript",
    "coldfusion",
    "common-lisp",
    "compojure",
    "coq",
    "crystal",
    "csharp",
    "css",
    "cue",
    "cypher",
    "d",
    "dart",
    "dhall",
    "directx9",
    "docker",
    "dynamic-programming",
    "easylang",
    "edn",
    "elisp",
    "elixir",
    "elm",
    "emacs",
    "erlang",
    "factor",
    "fish",
    "forth",
    "fortran",
    "fsharp",
    "gdscript",
    "git",
    "gleam",
    "go",
    "golfscript",
    "groovy",
    "hack",
    "haml",
    "haskell",
    "haxe",
    "hcl",
    "hdl",
    "hjson",
    "hocon",
    "hq9+",
    "html",
    "httpie",
    "hy",
    "inform7",
    "janet",
    "java",
    "javascript",
    "jinja",
    "jq",
    "jquery",
    "json",
    "jsonnet",
    "julia",
    "kdb+",
    "kotlin",
    "lambda-calculus",
    "latex",
    "lbstanza",
    "ldpl",
    "lean4",
    "less",
    "lfe",
    "linker",
    "livescript",
    "logtalk",
    "lolcode",
    "lua",
    "m",
    "make",
    "markdown",
    "matlab",
    "mercurial",
    "mercury",
    "messagepack",
    "miniscript",
    "mips",
    "mongodb",
    "moonscript",
    "nim",
    "nix",
    "objective-c",
    "ocaml",
    "opencv",
    "opengl",
    "openmp",
    "openscad",
    "osl",
    "p5",
    "paren",
    "pascal",
    "pcre",
    "perl",
    "phel",
    "phix",
    "php-composer",
    "php",
    "pogo",
    "powershell",
    "processing",
    "prolog",
    "protocol-buffer-3",
    "pug",
    "purescript",
    "pyqt",
    "python",
    "pythonlegacy",
    "pythonstatcomp",
    "qml",
    "qsharp",
    "qt",
    "r",
    "racket",
    "raku-pod",
    "raku",
    "raylib",
    "rdf",
    "reason",
    "red",
    "rescript",
    "rst",
    "ruby-ecosystem",
    "ruby",
    "rust",
    "sass",
    "scala",
    "sed",
    "self",
    "set-theory",
    "shutit",
    "sing",
    "smallbasic",
    "smalltalk",
    "solidity",
    "sorbet",
    "sql",
    "standard-ml",
    "stylus",
    "swift",
    "tailspin",
    "tcl",
    "tcsh",
    "texinfo",
    "textile",
    "tmux",
    "toml",
    "typescript",
    "uxntal",
    "v",
    "vala",
    "vim",
    "vimscript",
    "visualbasic",
    "wasm",
    "wikitext",
    "wolfram",
    "xml",
    "yaml",
    "zfs",
    "zig",
];
