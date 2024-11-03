
/// This struct will hold the state of every match, and the currently selected
/// match. An interactive prompt that takes character input will be used to
/// re-draw the screen with the next match upon hitting the next/prev key.
/// We'll be using crossterm to handle positioning the cursor and clearing the
/// lines. We won't be using a TUI library, since we just need to move the
/// cursor and clear lines and print the next match, and since every match
/// is always the same length, we can easily use the same calculated position
/// to clear the line and print the next match.
#[derive(Debug, Clone, Default)]
pub struct Cycler {
    matches: Vec<(String, MatchedLine)>,
    matches_index: usize,
    preview_xy: Dimensions,
    options: CyclerOptions,
    halt: Arc<AtomicBool>,
}

impl Cycler {
    pub fn new(matches: Vec<(String, MatchedLine)>, options: CyclerOptions) -> ah::Result<Self> {
        let previewer_dimensions = Dimensions {
            height: options.context * 2 + 1,
            width: options.line_length_limit.calculate_limit()?,
        };

        Ok(Self {
            matches,
            matches_index: 0,
            options,
            preview_xy: previewer_dimensions,
            halt: Arc::new(AtomicBool::new(false)),
        })
    }

    fn render_separator(&self, separator: &Separator) -> String {
        match separator {
            Separator::LineLengthLimit(c) => {
                format!("{}", c.to_string().repeat(self.preview_xy.width))
            }

            Separator::MatchLongestLine(c) => {
                let longest_line = self
                    .matches
                    .iter()
                    .map(|m| m.1.length)
                    .max()
                    .unwrap_or(self.preview_xy.width);

                format!("{}", c.to_string().repeat(longest_line))
            }

            Separator::Repeat(c, n) => {
                format!("{}", c.to_string().repeat(*n))
            }

            Separator::String(s) => {
                format!("{}", s)
            }
        }
    }


    fn print_match(&self, matched_line: &MatchedLine) -> ah::Result<()> {
        let mut stdout = std::io::stdout();

        let file = OpenOptions::new()
            .read(true)
            .open(&matched_line.file)
            .unwrap();

        let total_lines: Option<usize> = self
            .options
            .show_line_numbers
            .then(|| count_file_lines(&matched_line.file).ok())
            .flatten();

        let reader = BufReader::new(file);

        #[rustfmt::skip]
        let context_lines = 
               matched_line.line_num - self.options.context
            ..=matched_line.line_num + self.options.context;

        for (line_num, mut line) in reader.lines().filter_map(Result::ok).enumerate() {
            if !context_lines.contains(&line_num) {
                continue;
            }

            // If line numbers are enabled, prepped the line number to the line.
            if let Some(total) = total_lines {
                let padding = ((total as f64).log10().floor() + 1.0) as usize;
                line = format!("{:0padding$}: {}", line_num, line, padding = padding);
            }

            // It's the exact line that contains the match.
            if line_num == matched_line.line_num {
                if let Some(hstr) = &self.options.highlight_match_str {
                    line = format!("{}{}", hstr, line);
                }

                // Switch color to highlight match color
                if let Some(color) = self.options.match_highlight_color {
                    execute!(stdout, SetForegroundColor(color))?;
                }
            }

            // Truncate the line if it exceeds the line length limit.
            if line.len() > self.preview_xy.width {
                if let Some((c, n)) = self.options.line_cutoff {
                    line.truncate(self.preview_xy.width - n);
                    line.push_str(&c.to_string().repeat(n));
                } else {
                    line = line[..self.preview_xy.width].to_string();
                }
            }

            // print the line
            execute!(stdout, Print(line))?;

            // reset color
            execute!(stdout, ct::style::ResetColor)?;

            // move to next line
            execute!(stdout, ct::cursor::MoveToNextLine(1))?;
        }

        Ok(())
    }

    pub fn render(&mut self) -> ah::Result<()> {

        // if let Some(sep) = &self.options.separator_top {
        //     let separator = self.render_separator(sep);
        //     // execute!(std::io::stdout(), print(separator), ct::cursor::movetonextline(1))?;
        //     println!("{}", separator);
        // } else {
        //     println!("");
        // }

        // for _ in 0..self.preview_xy.height {
        //     // execute!(std::io::stdout(), Print(""), ct::cursor::MoveToNextLine(1))?;
        // }
        self.print_blanks();

        let pos2 = ct::cursor::position()?;

        self.print_blanks();

        while !self.halt.load(atomic::Ordering::SeqCst) {
            if let Some(sep) = &self.options.separator_top {
                let separator = self.render_separator(sep);
                // execute!(std::io::stdout(), print(separator), ct::cursor::movetonextline(1))?;
                println!("{}", separator);
            } else {
                println!("");
            }


            // Clear the lines where the matches would be printed.
            // ----------------------------------------------------------------
            execute!(
                std::io::stdout(),
                ct::cursor::MoveToRow(pos2.1),
                ct::cursor::MoveToColumn(0)
            )?;

            for _ in 0..self.preview_xy.height {
                execute!(
                    std::io::stdout(),
                    ct::cursor::MoveToPreviousLine(1),
                    ct::terminal::Clear(ClearType::CurrentLine),
                )?;
            }



        if let Some(sep) = &self.options.separator_bottom {
            let separator = self.render_separator(sep);
            println!("{}", separator);
            // execute!(std::io::stdout(), Print(separator), ct::cursor::MoveToPreviousLine(1))?;
        } else {
            println!();
        }


            if let Some(matched_line) = self.matches.get(self.matches_index) {
                self.print_match(&matched_line.1)?;
            } else {
                execute!(std::io::stdout(), ct::style::SetForegroundColor(Color::Red), Print("No matches to display."), ct::style::ResetColor)?;
            }

            let key = get_input()?;

            match key {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.halt.store(true, atomic::Ordering::Relaxed);
                    break;
                }

                KeyCode::Char('l') => {
                    if self.options.carousel {
                        self.matches_index = (self.matches_index + 1) % self.matches.len();
                    } else if self.matches_index < self.matches.len() - 1 {
                        self.matches_index += 1;
                    }
                }

                KeyCode::Char('h') => {
                    if self.options.carousel {
                        if self.matches_index > 0 {
                            self.matches_index -= 1;
                        } else {
                            self.matches_index = self.matches.len() - 1;
                        }
                    } else if self.matches_index > 0 {
                        self.matches_index -= 1;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn print_blanks(&self) {
        (0..self.preview_xy.height).for_each(|_| println!());
    }

    /// just a test function to fill the screen with hashes to see if
    /// we can clear the lines properly.
    pub fn print_test_fill(&self, i: usize, pos: (u16, u16)) -> ah::Result<()> {
        // let opos = ct::cursor::position()?;
        // let mut stdout = std::io::stdout();
        // let chars: [char; 7] = ['#', '*', '@', '!', '+', '-', '='];
        // let content = chars[i % 7].to_string().repeat(self.max_line_length);
        //
        // for i in 0..self.context {
        //     execute!(stdout, Print(&content), ct::cursor::MoveToNextLine(1))?;
        // }
        //
        // Ok(())
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
        println!("{}", "-".repeat(20));

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
impl Default for CyclerOptions {
    fn default() -> Self {
        Self {
            line_length_limit: LineLengthLimit::TermPercent(36),
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
            buffer_position: ct::cursor::position().ok(),
            syntax_highlight: true,
            carousel: true,
            highlight_match_str: Some("-> ".into()),
            match_highlight_color: Some(Color::Green),
            show_line_numbers: true,

            // '─' < box char, but len == 3, TODO: write unicode friendly len function.
            separator_top: Some(Separator::LineLengthLimit('-')),
            separator_bottom: Some(Separator::LineLengthLimit('-')),
            context: 3,
        }
    }
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
    pub separator_top: Option<Separator>,

    /// The bottom separator, printed after the match body.
    pub separator_bottom: Option<Separator>,

    /// How many lines above and below the match to print alongside the match.
    /// the match. If the context is 3, then 7 lines would be printed, 1 for
    /// the match, and then 3 above & below the matched line. Can be set to
    /// 0 in order to only print the matched line.
    pub context: usize,

    pub key_next: HashSet<KeyCode>,
    pub key_prev: HashSet<KeyCode>,
}
