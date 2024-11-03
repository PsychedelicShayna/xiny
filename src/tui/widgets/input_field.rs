use std::fmt::{Display, Formatter};

use super::*;
use crate::tui::event_loop::TuiState;

use super::widget::Widget;

use crossterm::{
    cursor::MoveToNextLine,
    event::KeyCode,
    queue,
    style::{Color, Print},
    terminal::{Clear, ClearType},
};
use widget::DrawError;

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

    parent: Rc<TuiState>,

    /// The fg, bg colors of the virtual cursor.
    vcursor_color: (Color, Color),

    /// The index of the virtual cursor in the buffer. This is not the actual
    /// terminal cursor, the character at this position is just colorized.
    vcursor_idx: usize,

    /// The buffer storing the typed input.
    buffer: String,

    /// The prefix string to display before the user's input, e.g. "> "
    prompt: String,

    /// If set to false, the entire widget is deactivated / will not draw.
    enabled: bool,
}

impl Default for InputField {
    fn default() -> Self {
        Self {
            vi_mode: ViMode::default(),
            vi_kseq: Vec::new(),
            parent: Rc::default(),
            prompt: "> ".into(),
            buffer: String::new(),
            vcursor_color: (Color::Black, Color::White),
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

impl Widget for InputField {
    fn new(parent: Rc<TuiState>) -> Self {
        Self {
            parent,
            ..Default::default()
        }
    }

    fn queue_draws(&self) -> ah::Result<()> {
        if !self.enabled {
            ah::bail!(DrawError::DrawingDisabled);
        }

        let tsize @ (trows, tcols) = self.parent.terminal_size;
        let msize @ (mrows, mcols) = self.get_min_size();
        let (_, pcols) = self.get_size();

        if trows <= mrows || tcols <= mcols {
            ah::bail!(DrawError::NoSpace {
                have: tsize,
                need: msize,
            });
        }

        queue!(
            stdout(),
            Clear(ClearType::CurrentLine),
            Print(BCTL),
            Print(BHCL.to_string().repeat(pcols as usize)),
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
            Print(BCBL),
            Print(BHCL.to_string().repeat(pcols as usize)),
            MoveToNextLine(1),
        )?;

        Ok(())
    }


    fn queue_clear(&self) -> anyhow::Result<()> {
        if !self.enabled {
            ah::bail!(DrawError::DrawingDisabled);
        }

        let tsize @ (trows, tcols) = self.parent.terminal_size;
        let msize @ (mrows, mcols) = self.get_min_size();
        let (prows, _) = self.get_size();

        for _ in 0..prows.max(trows) {
            queue!(
                stdout(),
                Clear(ClearType::CurrentLine),
                MoveToNextLine(1),
            )?;
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        match event {
            Event::Key(key_event) => {

            }

            Event::Resize(tcols, trows) => {
                let (mrows, mcols) = self.get_min_size();

                if trows < mrows || tcols < mcols {
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

    fn get_size(&self) -> (u16, u16) {
        (3u16, 36u16)
    }

    fn get_min_size(&self) -> (u16, u16) {
        let (_, tcols) = self.parent.terminal_size;

        // Cannot concede on needing 3 rows.
        let mrows = 3u16;
        let mcols = self.format_text().len() as u16;

        (mrows, mcols)
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
