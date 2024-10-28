use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

use crossterm as ct;
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::style::{ContentStyle, PrintStyledContent, StyledContent};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, Clear, ClearType, DisableLineWrap,
    EnableLineWrap, EnterAlternateScreen, ScrollDown, ScrollUp, SetSize, WindowSize,
};
use std::collections::HashSet;

use crate::xiny::*;

use anyhow::{self as ah, Context};

pub enum FindMethod {
    Fuzzy(String),
    Regex(Vec<String>),
    Terms(Vec<String>),
}

/// This struct will hold the state of every match, and the currently selected
/// match. An interactive prompt that takes character input will be used to
/// re-draw the screen with the next match upon hitting the next/prev key.
/// We'll be using crossterm to handle positioning the cursor and clearing the
/// lines. We won't be using a TUI library, since we just need to move the
/// cursor and clear lines and print the next match, and since every match
/// is always the same length, we can easily use the same calculated position
/// to clear the line and print the next match.
#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Cycler {
    matches: Vec<MatchedLine>,
    matches_index: usize,

    options: CyclerOptions,
    rows_needed: usize,
}

pub fn test_entrypoint() {
    let mut i: usize = 0;
    let mut cy = Cycler::new(
        vec![MatchedLine {
            content: "1234567890".to_string(),
            length: 10,
            ..Default::default()
        }],
        6,
    )
    .unwrap();

    let pos1 = ct::cursor::position().unwrap();

    // for i in 0..cy.context {
    //     let blank = "0".repeat(cy.max_line_length);
    //     println!("{}", blank);
    // }

    // let pos2 = ct::cursor::position().unwrap();
    let pos2 = (0u16, cy.context as u16 - 1);

    loop {
        let kc = get_input().unwrap();
        match kc {
            KeyCode::Esc => break,
            KeyCode::Char('q') => break,
            KeyCode::Char('l') => {
                i += 1;
            }
            KeyCode::Char('h') => {
                if (i > 0) {
                    i -= 1;
                }
            }
            _ => {}
        }

        cy.clear(pos2.clone()).unwrap();

        execute!(
            std::io::stdout(),
            ct::cursor::MoveToRow(pos1.1),
            ct::cursor::MoveToColumn(0)
        );

        cy.print_test_fill(i, pos1).unwrap();
    }
}

/// Gets a single key press from the terminal, without needing NL/CR.
fn get_input() -> ah::Result<KeyCode> {
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

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Separator {
    /// Repeat a character a fixed number of times.
    Repeat(char, usize),

    /// Follow the line length limit.
    LineLengthLimit(char),

    /// Repeat a character up to the length of the longest line.
    /// If a line length limit is set, then it will stop there.
    MatchLongestLine(char),

    /// Just use a fixed string.
    String(String),

    /// Don't display a separator.
    Nothing,
}

impl Default for Separator {
    fn default() -> Self {
        Separator::Nothing
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum LineLengthLimit {
    // Positive values will limit the line length to that number of characters,
    // negative values will subtract from the terminal's width, and a value of
    // 0 will use the whole available width without risking line wrap.
    TermOffset(isize),

    /// Line cannot exceed N% of the terminal's width.
    TermPercent(usize),

    /// No limit on the line length, the terminal may wrap the line.
    Unlimited,
}

#[derive(Debug, Clone)]
pub struct CyclerOptions {
    /// The limit on the length of a line in the output. If the line exceeds
    /// the limit, then either it simply won't continue, printing past, or if
    /// line_cutoff isn't None, then the last N characters will be replaced.
    pub line_length_limit: LineLengthLimit,

    /// When a line is longer than max_line_length, how should it be cut off?
    /// Example: (char: '.', usize: 3) would display "incomplete sent..." if
    /// the line was "incomplete sentence", and the max_line_length was 15.
    pub line_cutoff: Option<(char, usize)>,

    /// Where the cycler should be displayed in the terminal. Defaults to being
    /// directly below the cursor when the cycler is opened and this is None.
    pub buffer_position: Option<(u16, u16)>,

    /// Return to the first match when attempting to go beyond the last match.
    pub carousel: bool,

    /// TODO: Add syntax highlighting to Markdown code blocks.
    pub syntax_highlight: bool,

    /// The string, if any, to prepend to the line containing the match.
    pub highlight_match_str: Option<String>,

    /// The color, if any, to highlight the match with.
    pub match_highlight_color: Option<Color>,

    /// Include line numbers in the output.
    pub show_line_numbers: bool,

    /// The top separator separating the title from the match body.
    pub separator_top: Separator,

    /// The bottom separator, printed after the match body.
    pub separator_bottom: Separator,

    /// How many lines above and below the match to print alongside the match.
    /// the match. If the context is 3, then 7 lines would be printed, 1 for
    /// the match, and then 3 above & below the matched line. Can be set to
    /// 0 in order to only print the matched line.
    pub context: usize,

    pub key_next: HashSet<KeyCode>,
    pub key_prev: HashSet<KeyCode>,
}
impl Default for CyclerOptions {
    fn default() -> Self {
        Self {
            line_length_limit: LineLengthLimit::TermPercent(37),
            line_cutoff: Some(('.', 2)),
            key_next: [
                KeyCode::Char('j'),
                KeyCode::Char('l'),
                KeyCode::Char('s'),
                KeyCode::Char('d'),
                KeyCode::Down,
                KeyCode::Tab,
            ]
            .iter()
            .cloned()
            .collect(),
            key_prev: [
                KeyCode::Char('k'),
                KeyCode::Char('h'),
                KeyCode::Char('w'),
                KeyCode::Char('a'),
                KeyCode::Up,
                KeyCode::BackTab,
            ]
            .iter()
            .cloned()
            .collect(),
            buffer_position: None,
            syntax_highlight: true,
            carousel: true,
            highlight_match_str: Some("-> ".into()),
            match_highlight_color: Some(Color::Green),
            show_line_numbers: true,

            // '─' < box char, but len == 3, TODO: write unicode friendly len function.
            separator_top: Separator::LineLengthLimit('-'),
            separator_bottom: Separator::LineLengthLimit('-'),
            context: 3,
        }
    }
}

impl Cycler {
    pub fn new(matches: Vec<MatchedLine>, options: CyclerOptions) -> ah::Result<Self> {
        let longest_line = matches.iter().map(|ml| ml.length).max().unwrap_or(0);

        #[rustfmt::skip]
        let additional = (
            1 + // The title line.
            1 + // The top separator line.
            1 + // The matched line itself.
            1   // The bottom separator line.
        );

        Ok(Self {
            matches,
            options,
            ..Default::default()
        })
    }

    pub fn configure(&mut self, options: CyclerOptions) {}

    fn print_blanks(&self) -> ah::Result<()> {
        Ok(())
    }

    /// Clear the lines where the matches woud be printed.
    pub fn clear(&self, pos: (u16, u16)) -> ah::Result<()> {
        let rows_to_clear = self.context;
        let mut stdout = std::io::stdout();

        execute!(
            stdout,
            ct::cursor::MoveToRow(pos.1),
            ct::cursor::MoveToColumn(0)
        )?;

        // for i in 0..=rows_to_clear {
        //     execute!(stdout, Print(format!("{:?}", opos)))?;
        //     execute!(
        //         stdout,
        //         ct::cursor::MoveToRow(opos.1 + i as u16),
        //     )?;
        // }
        //
        for i in 0..self.context {
            execute!(
                stdout,
                Clear(ClearType::CurrentLine),
                ct::cursor::MoveToPreviousLine(1)
            )?;
        }

        Ok(())
    }

    /// just a test function to fill the screen with hashes to see if
    /// we can clear the lines properly.
    pub fn print_test_fill(&self, i: usize, pos: (u16, u16)) -> ah::Result<()> {
        // let opos = ct::cursor::position()?;
        let mut stdout = std::io::stdout();
        let chars: [char; 7] = ['#', '*', '@', '!', '+', '-', '='];
        let content = chars[i % 7].to_string().repeat(self.max_line_length);

        for i in 0..self.context {
            execute!(stdout, Print(&content), ct::cursor::MoveToNextLine(1))?;
        }

        Ok(())
    }

    pub fn print_nth(&self, index: usize) -> ah::Result<()> {
        todo!()
    }

    /// Print the current match to the screen.
    pub fn print_curr(&self) -> ah::Result<()> {
        todo!()
    }

    pub fn print_next(&mut self) -> ah::Result<()> {
        todo!()
    }

    pub fn print_prev(&mut self) -> ah::Result<()> {
        todo!()
    }
}

pub fn find_terms(path: &PathBuf, terms: Vec<String>) -> ah::Result<Vec<(String, MatchedLine)>> {
    let mut matches = Vec::<(String, MatchedLine)>::new();

    let file = OpenOptions::new()
        .read(true)
        .open(path)
        .context("find_terms opening file")?;

    let reader = BufReader::new(file);

    for (line_num, ref line) in reader.lines().filter_map(Result::ok).enumerate() {
        for term in &terms {
            if line.to_lowercase().contains(&term.to_lowercase()) {
                let matched_line = MatchedLine {
                    file: path.clone(),
                    line_num,
                    content: line.clone(),
                    length: line.len(),
                };

                matches.push((term.to_string(), matched_line));
            }
        }
    }

    Ok(matches)
}

pub fn match_printer(
    file_path: &PathBuf,
    matches: Vec<(String, MatchedLine)>,
    context: usize,
    printmax: usize,
) -> ah::Result<()> {
    if context % 2 != 0 {
        ah::bail!("Context must be divisible by 2; an even number.");
    }

    let file = OpenOptions::new()
        .read(true)
        .open(&file_path)
        .context("match_printer opening file")?;

    let lines: Vec<String> = BufReader::new(file)
        .lines()
        .filter_map(Result::ok)
        .collect();

    let mut matches_printed: usize = 0;

    for (term, matched_line) in matches.iter() {
        if printmax > 0 && matches_printed >= printmax {
            break;
        }

        let start_line: usize = if matched_line.line_num < context {
            0
        } else {
            matched_line.line_num - context
        };

        let end_line: usize = matched_line.line_num + context;

        println!("Match for term '{}':", term);
        println!("{}", "-".repeat(80));

        for (line_num, ref line) in lines.iter().enumerate() {
            if line_num >= start_line && line_num <= end_line {
                if line_num == matched_line.line_num {
                    println!("> {}: {}", line_num, line);
                } else {
                    println!("{}: {}", line_num, line);
                }
            }
        }

        println!();

        matches_printed += 1;
    }

    Ok(())
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MatchedLine {
    pub file: PathBuf,
    pub line_num: usize,
    pub content: String,
    pub length: usize,
}

impl MatchedLine {
    pub fn new(file: PathBuf) -> Self {
        Self {
            file,
            ..Default::default()
        }
    }
}
