use std::iter::successors;
use std::process::exit;

use crossterm::ExecutableCommand;
use cursor::{MoveToColumn, Show};
use style::ResetColor;
use terminal::{disable_raw_mode, enable_raw_mode};

use crate::sat;
use crate::tui::tui::Tui;

use super::*;
use std::sync::mpsc;

#[derive(Debug, Clone)]
pub enum VimMode {
    Normal,
    Insert,
    // Unused for the moment.
    // Visual,
    // Command
}

impl Display for VimMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Normal => write!(f, "N"),
            Self::Insert => write!(f, "I"),
        }
    }
}

impl Default for VimMode {
    fn default() -> Self {
        Self::Normal
    }
}

pub enum InputFieldMessage {
    TuiStopSignal,
    None,
}

#[derive(Debug)]
pub struct InputField {
    ///  The target rendering size. Input field should try fit or fill this.
    pub size: Point,

    /// The channel to send messages to the TUI.
    pub tuitx: mpsc::Sender<InputFieldMessage>,

    /// The minimum size of the input field, where anything lower will cause
    /// it not to render properly. Note: this will not prevent rendering, but
    /// it's to be cheked by the TUI to deice whether to render or not.
    pub minimum_size: Point,

    /// As the cursor is disabled, we're using a virtual cursor that changes
    /// the color of the character at the index in order to create a "cursor"
    pub cursor_color: Colors,
    pub cursor_index: usize,

    /// The full contents, of which parts are rendered  from.
    pub buffer: String,

    /// Where characters will be inserted into the buffer.
    /// This is indepdent of the cursor positon.
    pub cursor_position_in_window: usize,

    /// What prompt to show before the input field.
    pub prompt: String,

    /// Disable the whoel component if set to false.
    pub enabled: bool,

    /// Repetitive Strain Injury Mode (Disables Vim Motions)
    pub rsi: bool,

    pub vi_mode: VimMode,
    pub vi_cseq: String,

    /// The offset from the start of the buffer contents to where the visible
    /// text should be printed after the cursor scrolls out of bounds.
    window_start: usize,
}

impl Component for InputField {
    fn queue_draws(&self) -> ah::Result<()> {
        if !self.enabled {
            ah::bail!(DrawError::DrawingDisabled);
        }

        match self.rsi {
            true => self.queue_draws_rsi()?,
            false => self.queue_draws_vim()?,
        };

        Ok(())
    }

    fn queue_clear(&self) -> anyhow::Result<()> {
        if !self.enabled {
            ah::bail!(DrawError::DrawingDisabled);
        }

        queue!(stdout(), Clear(ClearType::CurrentLine))?;
        queue!(stdout(), MoveToPreviousLine(1))?;
        queue!(stdout(), Clear(ClearType::CurrentLine))?;
        queue!(stdout(), MoveToPreviousLine(1))?;
        queue!(stdout(), Clear(ClearType::CurrentLine))?;
        queue!(stdout(), MoveToPreviousLine(1))?;
        queue!(stdout(), Clear(ClearType::CurrentLine))?;

        #[cfg(debug_assertions)]
        {
            for _ in 0..=11 {
                queue!(stdout(), MoveToPreviousLine(1))?;
                queue!(stdout(), Clear(ClearType::CurrentLine))?;
            }
        }

        Ok(())
    }

    fn handle_input(&mut self, input_event: KeyEvent) -> anyhow::Result<()> {
        if self.rsi {
            self.handle_input_rsi(input_event)?;
        } else {
            self.handle_input_vim(input_event)?;
        }

        Ok(())
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_size(&self) -> Point {
        // (3u16, 36u16)
        todo!()
    }

    fn get_min_size(&self) -> Point {
        let minimum_rows = 3u16;
        let minimum_cols = self.format_buffer().len() as u16 + self.buffer.len() as u16;

        Point {
            col: minimum_cols,
            row: minimum_rows,
        }
    }
}

/// Returns the indices of the start and end of each word.
fn find_words(string: &str) -> Vec<(usize, usize)> {
    let mut word_indicies = Vec::<(usize, usize)>::new();
    let mut start = 0;
    let mut in_word = false;

    for (i, c) in string.chars().enumerate() {
        if c.is_whitespace() && in_word {
            word_indicies.push((start, i - 1));
            in_word = false;
        } else if !c.is_whitespace() && !in_word {
            start = i;
            in_word = true;
        }
    }

    if in_word {
        word_indicies.push((start, string.len() - 1));
    }

    word_indicies
}

/// Calculate the new index of the cursor after the word motion.
fn motion_word(str: &String, idx: usize, backward: bool, endwise: bool) -> usize {
    let words = find_words(str);

    let mut new_idx = idx;

    for (start, end) in words {
        if !backward {
            if endwise && start == idx && end != idx {
                new_idx = end;
                break;
            }

            if start <= idx || end <= idx {
                continue;
            }

            new_idx = if endwise { end } else { start };
            break;
        } else {
            if start < idx && end >= idx {
                new_idx = start;
                break;
            }

            if end >= idx || start >= idx {
                continue;
            }

            new_idx = if endwise { end } else { start };
        }
    }

    new_idx
}

impl InputField {
    pub fn format_buffer(&self) -> String {
        let visible = self
            .buffer
            .chars()
            .into_iter()
            .skip(self.window_start)
            .take(self.size.col.into())
            .collect::<String>();

        match &self.rsi {
            false => format!(
                "{} {} {} {}{}",
                boxchars::BVCL,
                self.vi_mode,
                boxchars::BVCL,
                self.prompt,
                visible
            ),
            true => format!("{} {}{}", boxchars::BVCL, self.prompt, visible),
        }
    }

    pub fn new() -> (Self, mpsc::Receiver<InputFieldMessage>) {
        let (tuitx, tuirx) = mpsc::channel::<InputFieldMessage>();

        let mut input_field = InputField {
            rsi: false,
            tuitx,
            size: Point::default(),
            minimum_size: Point { col: 3, row: 3 },
            cursor_color: Colors {
                foreground: Some(Color::Black),
                background: Some(Color::White),
            },
            cursor_index: 0,
            buffer: String::new(),
            prompt: "".into(),
            enabled: true,
            vi_mode: VimMode::Normal,
            vi_cseq: String::new(),
            window_start: 0,
            cursor_position_in_window: 0,
        };

        input_field.minimum_size.row = 3;

        (input_field, tuirx)
    }

    /// Handle input events when Vim motions are enabled.
    fn handle_input_vim(&mut self, input_event: KeyEvent) -> ah::Result<()> {
        let mut cursor_position_in_window = self.cursor_index - self.window_start;

        match (&self.vi_mode, input_event.code, input_event.modifiers) {
            (VimMode::Normal, KeyCode::Char('i'), KeyModifiers::NONE) => {
                self.vi_mode = VimMode::Insert;
            }
            (VimMode::Insert, KeyCode::Esc | KeyCode::Enter, KeyModifiers::NONE) => {
                self.vi_mode = VimMode::Normal;
            }
            (VimMode::Normal, KeyCode::Char('q'), KeyModifiers::NONE) => {
                self.tuitx.send(InputFieldMessage::TuiStopSignal)?;
            }
            (VimMode::Normal, KeyCode::Char('h'), KeyModifiers::NONE) => {
                if self.cursor_index > 0 {
                    self.cursor_index -= 1;
                    if cursor_position_in_window == 0 {
                        self.window_start = self.window_start.saturating_sub(1);
                    }
                }
            }
            (VimMode::Normal, KeyCode::Char('l'), KeyModifiers::NONE) => {
                if self.cursor_index < self.buffer.len() {
                    self.cursor_index += 1;

                    if cursor_position_in_window >= (self.size.col as usize - 1) {
                        self.window_start += 1;
                    }
                }
            }
            (VimMode::Normal, KeyCode::Char('w'), KeyModifiers::NONE) => {
                let index = motion_word(&self.buffer, self.cursor_index, false, false);

                if index < self.buffer.len() {
                    self.cursor_index = index;

                    loop {
                        cursor_position_in_window = self.cursor_index - self.window_start;

                        if cursor_position_in_window < (self.size.col as usize - 1) {
                            break;
                        }

                        self.window_start += 1;
                    }
                }
            }
            (VimMode::Normal, KeyCode::Char('b'), KeyModifiers::NONE) => {
                let index = motion_word(&self.buffer, self.cursor_index, true, false);
                self.cursor_index = index;

                if index > 0 {
                    loop {
                        cursor_position_in_window =
                            self.cursor_index.saturating_sub(self.window_start);

                        if cursor_position_in_window != 0 {
                            break;
                        }

                        self.window_start = self.window_start.saturating_sub(1);
                    }
                } else {
                    self.window_start = 0;
                }
            }
            (VimMode::Normal, KeyCode::Char('0'), KeyModifiers::NONE) => {
                self.cursor_index = 0;
                self.window_start = 0;
            }
            (VimMode::Normal, KeyCode::Char('$'), KeyModifiers::NONE) => {
                self.cursor_index = self.buffer.len() - 1;

                loop {
                    cursor_position_in_window = self.cursor_index - self.window_start;

                    if cursor_position_in_window < (self.size.col as usize - 1) {
                        break;
                    }

                    self.window_start += 1;
                }
            }

            (VimMode::Normal, KeyCode::Char('C'), KeyModifiers::SHIFT) => {
                self.buffer.truncate(self.cursor_index);
                self.vi_mode = VimMode::Insert;
            }
            (VimMode::Normal, KeyCode::Char('D'), KeyModifiers::SHIFT) => {
                self.buffer.truncate(self.cursor_index);
            }

            (VimMode::Normal, KeyCode::Char('A'), KeyModifiers::SHIFT) => {
                self.cursor_index = self.buffer.len() - 1;
                self.vi_mode = VimMode::Insert;

                loop {
                    cursor_position_in_window = self.cursor_index - self.window_start;

                    if cursor_position_in_window < (self.size.col as usize - 1) {
                        break;
                    }

                    self.window_start += 1;
                }
            }
            (VimMode::Normal, KeyCode::Char('e'), KeyModifiers::NONE) => {
                let index = motion_word(&self.buffer, self.cursor_index, false, true);

                if index < self.buffer.len() {
                    self.cursor_index = index;

                    loop {
                        cursor_position_in_window = self.cursor_index - self.window_start;
                        if cursor_position_in_window < (self.size.col as usize - 1) {
                            break;
                        }
                        self.window_start += 1;
                    }
                }
            }
            //
            (VimMode::Normal, KeyCode::Char('a'), KeyModifiers::NONE) => {
                self.cursor_index += 1;

                if self.cursor_index >= self.buffer.len() {
                    self.buffer.push(' ');
                }

                if cursor_position_in_window >= (self.size.col as usize - 1) {
                    self.window_start += 1;
                }

                self.vi_mode = VimMode::Insert;
            }

            (VimMode::Normal, KeyCode::Char('x'), KeyModifiers::NONE) => {
                if self.cursor_index < self.buffer.len() {
                    self.buffer.remove(self.cursor_index);
                }
            }

            (VimMode::Insert, KeyCode::Backspace, KeyModifiers::NONE) => {
                if self.cursor_index != 0 {
                    self.buffer.remove(self.cursor_index - 1);
                    self.cursor_index = self.cursor_index.saturating_sub(1);

                    if cursor_position_in_window.saturating_sub(1) == 0 {
                        self.window_start =
                            self.window_start.saturating_sub(self.size.col as usize);
                    }
                }
            }
            (VimMode::Insert, KeyCode::Char(c), _) => {
                self.buffer.insert(self.cursor_index, c);
                self.cursor_index += 1;

                if cursor_position_in_window >= (self.size.col as usize - 1) {
                    self.window_start += 1;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle inputs when Vim motions are disabled.
    fn handle_input_rsi(&mut self, input_event: KeyEvent) -> ah::Result<()> {
        let mut cursor_position_in_window = self.cursor_index - self.window_start;

        match (input_event.code, input_event.modifiers) {
            (KeyCode::Delete, KeyModifiers::NONE) => {
                if self.cursor_index < self.buffer.len() {
                    self.buffer.remove(self.cursor_index);
                }
            }

            (KeyCode::Right, KeyModifiers::NONE) => {
                if self.cursor_index < self.buffer.len() {
                    self.cursor_index += 1;

                    if cursor_position_in_window >= (self.size.col as usize - 1) {
                        self.window_start += 1;
                    }
                }
            }

            (KeyCode::Left, KeyModifiers::NONE) => {
                if self.cursor_index > 0 {
                    self.cursor_index -= 1;
                    if cursor_position_in_window == 0 {
                        self.window_start = self.window_start.saturating_sub(1);
                    }
                }
            }

            (KeyCode::Right, KeyModifiers::CONTROL) => {
                let idx = &mut self.cursor_index;
                let buf = &self.buffer;

                let new_idx = motion_word(buf, *idx, false, false);

                if new_idx == *idx && !buf.is_empty() {
                    *idx = buf.len() - 1;
                } else {
                    *idx = new_idx;
                }
            }

            (KeyCode::Left, KeyModifiers::CONTROL) => {
                let index = motion_word(&self.buffer, self.cursor_index, true, false);
                self.cursor_index = index;

                if index > 0 {
                    loop {
                        cursor_position_in_window =
                            self.cursor_index.saturating_sub(self.window_start);

                        if cursor_position_in_window != 0 {
                            break;
                        }

                        self.window_start = self.window_start.saturating_sub(1);
                    }
                } else {
                    self.window_start = 0;
                }
            }

            (KeyCode::Backspace, KeyModifiers::NONE) => {
                if self.cursor_index != 0 {
                    self.buffer.remove(self.cursor_index - 1);
                    self.cursor_index = self.cursor_index.saturating_sub(1);

                    if cursor_position_in_window.saturating_sub(1) == 0 {
                        self.window_start =
                            self.window_start.saturating_sub(self.size.col as usize);
                    }
                }
            }

            (KeyCode::Esc, KeyModifiers::NONE) | (KeyCode::F(4), KeyModifiers::ALT) => {
                self.tuitx.send(InputFieldMessage::TuiStopSignal)?;
            }

            (KeyCode::Char(c), _) => {
                self.buffer.insert(self.cursor_index, c);
                self.cursor_index += 1;

                if cursor_position_in_window >= (self.size.col as usize - 1) {
                    self.window_start += 1;
                }
            }

            _ => {}
        }

        Ok(())
    }

    fn queue_draws_vim(&self) -> ah::Result<()> {
        let mut stdout = stdout();

        let (formatted, visible, boxdelta) = {
            let visible = self
                .buffer
                .chars()
                .skip(self.window_start)
                .take(self.size.col.into())
                .collect::<String>();

            let formatted = match &self.rsi {
                false => format!(
                    "{} {} {} {} {}",
                    boxchars::BVCL,
                    self.vi_mode,
                    boxchars::BVCL,
                    self.prompt,
                    visible
                ),
                true => format!("{} {}{}", boxchars::BVCL, self.prompt, visible),
            };

            let delta = (formatted.len() - visible.len()) / 2;
            (formatted, visible, delta)
        };

        let header = format!("{} {} {} ", boxchars::BVCL, self.vi_mode, boxchars::BVCL);
        let cursor_position_in_window = self.cursor_index - self.window_start;

        // The top half of the input field.
        queue!(
            stdout,
            Clear(ClearType::CurrentLine),
            Print(boxchars::BCTL),
            Print(boxchars::BHCL),
            Print(boxchars::BHCL),
            Print(boxchars::BHCL),
            Print(boxchars::BVJD),
            Print(boxchars::BHCL.to_string().repeat(self.size.col as usize)),
            MoveToNextLine(1),
        )?;

        queue!(
            stdout,
            Clear(ClearType::CurrentLine),
            MoveToColumn(0),
            Print(&header),
            Print(&visible),
            MoveToColumn((cursor_position_in_window).saturating_add(header.chars().count()) as u16),
            SetBackgroundColor(Color::Green),
            SetForegroundColor(Color::Black),
            Print(self.buffer.chars().nth(self.cursor_index).unwrap_or(' ')), // Highlight the character under cursor
            ResetColor,
            MoveToNextLine(1)
        )?;

        // The bottom half; flipped version of the top.
        queue!(
            stdout,
            Clear(ClearType::CurrentLine),
            Print(boxchars::BCBL),
            Print(boxchars::BHCL),
            Print(boxchars::BHCL),
            Print(boxchars::BHCL),
            Print(boxchars::BVJU),
            Print(boxchars::BHCL.to_string().repeat(self.size.col as usize)),
            MoveToNextLine(1),
        )?;

        #[cfg(debug_assertions)]
        queue!(
            stdout,
            MoveToNextLine(1),
            Print(format!("cursor_col: {}", self.cursor_index)),
            MoveToNextLine(1),
            Print(format!("buffer_index: {}", self.cursor_position_in_window)),
            MoveToNextLine(1),
            Print(format!("buffer_range_start: {}", self.window_start)),
            MoveToNextLine(1),
            Print(format!("buffer: {}", self.buffer)),
            MoveToNextLine(1),
            Print(format!("delta: {}", boxdelta)),
            MoveToNextLine(1),
            Print(format!("vi_mode: {}", self.vi_mode)),
            MoveToNextLine(1),
            Print(format!("vi_cseq: {}", self.vi_cseq)),
            MoveToNextLine(1),
            Print(format!("buffer_view: {}", &formatted)),
            MoveToNextLine(1),
            Print(format!("size: {:?}", self.size)),
            MoveToNextLine(1),
            Print(format!("minimum_size: {:?}", self.minimum_size)),
            MoveToNextLine(1),
        )?;
        Ok(())
    }

    fn queue_draws_rsi(&self) -> ah::Result<()> {
        let mut stdout = stdout();

        let visible = self
            .buffer
            .chars()
            .skip(self.window_start)
            .take(self.size.col.into())
            .collect::<String>();

        let header = format!("{} ", boxchars::BVCL);
        let cursor_position_in_window = self.cursor_index - self.window_start;

        // The top half of the input field.
        queue!(
            stdout,
            Clear(ClearType::CurrentLine),
            Print(boxchars::BCTL),
            Print(boxchars::BHCL.to_string().repeat(self.size.col as usize)),
            MoveToNextLine(1),
        )?;

        queue!(
            stdout,
            Clear(ClearType::CurrentLine),
            MoveToColumn(0),
            Print(&header),
            Print(&visible),
            MoveToColumn((cursor_position_in_window).saturating_add(header.chars().count()) as u16),
            SetBackgroundColor(Color::Green),
            SetForegroundColor(Color::Black),
            Print(self.buffer.chars().nth(self.cursor_index).unwrap_or(' ')), // Highlight the character under cursor
            ResetColor,
            MoveToNextLine(1)
        )?;

        // The bottom half; flipped version of the top.
        queue!(
            stdout,
            Clear(ClearType::CurrentLine),
            Print(boxchars::BCBL),
            Print(boxchars::BHCL.to_string().repeat(self.size.col as usize)),
            MoveToNextLine(1),
        )?;

        #[cfg(debug_assertions)]
        queue!(
            stdout,
            MoveToNextLine(1),
            Print(format!("cursor_col: {}", self.cursor_index)),
            MoveToNextLine(1),
            Print(format!("buffer_index: {}", self.cursor_position_in_window)),
            MoveToNextLine(1),
            Print(format!("buffer_range_start: {}", self.window_start)),
            MoveToNextLine(1),
            Print(format!("buffer: {}", self.buffer)),
            MoveToNextLine(1),
            Print(format!("vi_mode: {}", self.vi_mode)),
            MoveToNextLine(1),
            Print(format!("vi_cseq: {}", self.vi_cseq)),
            MoveToNextLine(1),
            Print(format!("size: {:?}", self.size)),
            MoveToNextLine(1),
            Print(format!("minimum_size: {:?}", self.minimum_size)),
            MoveToNextLine(1),
        )?;
        Ok(())
    }
}
