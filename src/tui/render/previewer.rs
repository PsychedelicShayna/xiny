use super::*;

// Renders the search input field.

use crate::tui::event_loop::{TuiState, ViMode};

use crossterm::{
    cursor::MoveToNextLine,
    queue,
    style::{Color, Colors, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

pub fn render_previewer(state: &TuiState, anchor: &(usize, usize)) -> ah::Result<()> {



    Ok(())
}
