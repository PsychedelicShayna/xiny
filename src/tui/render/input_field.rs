use super::*;

// Renders the search input field.

use crate::tui::event_loop::{TuiState, ViMode};

use crossterm::{
    cursor::MoveToNextLine,
    queue,
    style::{Color, Colors, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

pub fn render(state: &TuiState, anchor: &(usize, usize)) -> ah::Result<()> {
    let sep1: char = '-';
    let pref: &str = "> ";
    let sep = BHCL.to_string().repeat(36);
    let buf = &state.search_buffer;

    let prefix = format!(
        "[{}] {}",
        if matches!(state.vi_mode, ViMode::Insert) {
            "I"
        } else {
            "N"
        },
        pref
    );

    // The search input field should look like this:

    /*
      -----------------------------------------
      > typed input▕
      -----------------------------------------
    */

    let current_char = &buf.chars().nth(state.search_cursor_index);

    let post_idx_str = match current_char {
        Some(_) if state.search_cursor_index + 1 < buf.len() => {
            &buf[state.search_cursor_index + 1..]
        }
        _ => "",
    };

    queue!(
        std::io::stdout(),
        MoveToNextLine(1),
        Clear(ClearType::CurrentLine),
        Print(&sep),
        MoveToNextLine(1),
        Clear(ClearType::CurrentLine),
        Print(&prefix),
        Print(&buf[..state.search_cursor_index]),
        SetBackgroundColor(Color::White),
        SetForegroundColor(Color::Black),
        Print(current_char.unwrap_or(' ')),
        ResetColor,
        Print(post_idx_str),
        MoveToNextLine(1),
        Clear(ClearType::CurrentLine),
        Print(&sep),
    )?;

    Ok(())
}
