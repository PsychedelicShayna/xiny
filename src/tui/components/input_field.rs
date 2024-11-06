use point::Point;

use super::*;

#[derive(Debug, Clone)]
pub enum ViMode {
    Normal,
    Insert,
}

impl Display for ViMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Normal => write!(f, "N"),
            Self::Insert => write!(f, "I"),
        }
    }
}

impl Default for ViMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Clone, Debug)]
pub struct InputField {
    pub vi_mode: ViMode,
    pub vi_kseq: Vec<char>,
    vcursor_color: Colors,
    vcursor_idx: usize,
    buffer: String,
    prompt: String,
    enabled: bool,
}

impl Default for InputField {
    fn default() -> Self {
        Self {
            vi_mode: ViMode::default(),
            vi_kseq: Vec::default(),
            prompt: "> ".into(),
            buffer: String::new(),
            vcursor_color: Colors {
                foreground: Some(Color::Black),
                background: Some(Color::White),
            },
            vcursor_idx: 0,
            enabled: true,
        }
    }
}

impl InputField {
    pub fn format_text(&self) -> String {
        format!("[{}] {}{}", self.vi_mode, self.prompt, self.buffer)
    }
}

impl Component for InputField {
    fn queue_draws(&self) -> ah::Result<()> {
        if !self.enabled {
            ah::bail!(DrawError::DrawingDisabled);
        }

        queue!(
            stdout(),
            Clear(ClearType::CurrentLine),
            Print(boxchars::BCTL),
            Print(boxchars::BHCL.to_string().repeat(pref_size.col as usize)),
            MoveToNextLine(1),
        )?;

        queue!(
            stdout(),
            Clear(ClearType::CurrentLine),
            Print(format!("[{}] {}{}", self.vi_mode, self.prompt, self.buffer,)),
            MoveToNextLine(1)
        )?;

        queue!(
            stdout(),
            Clear(ClearType::CurrentLine),
            Print(boxchars::BCBL),
            Print(boxchars::BHCL.to_string().repeat(pref_size.col as usize)),
            MoveToNextLine(1),
        )?;

        Ok(())
    }

    fn queue_clear(&self) -> anyhow::Result<()> {
        if !self.enabled {
            ah::bail!(DrawError::DrawingDisabled);
        }

        let term_size = self.parent.terminal_size;
        let min_size = self.get_min_size();
        let pref_size = self.get_size();

        for _ in 0..pref_size.row.max(term_size.row) {
            queue!(stdout(), Clear(ClearType::CurrentLine), MoveToNextLine(1),)?;
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        match event {
            Event::Key(key_event) => {}

            Event::Resize(tcols, trows) => {
                let min_size = self.get_min_size();
                let term_size = self.parent.terminal_size;

                if term_size.row < min_size.row || term_size.col < min_size.col {
                    self.set_enabled(false);
                } else {
                    self.set_enabled(true);
                }
            }

            _ => {}
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
        let minimum_cols = self.format_text().len() as u16;

        Point {
            col: minimum_cols,
            row: minimum_rows,
        }
    }
}

// pub fn render(state: &TuiState, anchor: &(usize, usize)) -> ah::Result<()> {
//     let sep = BHCL.to_string().repeat(state.terminal_dimensions.width);
//     let buf = &state.search_buffer;
//
//     let prefix = format!(
//         "[{}] {}",
//         if matches!(state.vi_mode, ViMode::Insert) {
//             "I"
//         } else {
//             "N"
//         },
//         pref
//     );
//
//     // The search input field should look like this:
//
//     /*
//       -----------------------------------------
//       > typed input▕
//       -----------------------------------------
//     */
//
//     let current_char = &buf.chars().nth(state.search_cursor_index);
//
//     let post_idx_str = match current_char {
//         Some(_) if state.search_cursor_index + 1 < buf.len() => {
//             &buf[state.search_cursor_index + 1..]
//         }
//         _ => "",
//     };
//
//     queue!(
//         std::io::stdout(),
//         // MoveToNextLine(1),
//         // Clear(ClearType::CurrentLine),
//         // Print(&sep),
//         MoveToNextLine(1),
//         Clear(ClearType::CurrentLine),
//         Print(&prefix),
//         Print(&buf[..state.search_cursor_index]),
//         SetBackgroundColor(Color::White),
//         SetForegroundColor(Color::Black),
//         Print(current_char.unwrap_or(' ')),
//         ResetColor,
//         Print(post_idx_str),
//         MoveToNextLine(1),
//         Clear(ClearType::CurrentLine),
//         Print(&sep),
//     )?;
//
//     Ok(())
// }
