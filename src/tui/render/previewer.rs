use std::ops::Deref;

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
    let dims = &state.preview_dimensions;

    let rows = dims.rows() as usize;
    let cols = dims.cols() as usize;

    let separator = BHCL.to_string().repeat(36);

    let mut no_results = state.search_results.is_empty();

    let context = state.preview_context;
    let selected = state.search_result_index;

    let (srow, scol) = match state.search_results.get(selected) {
        Some(s) => *s,
        None => (context, 0),
    };

    let offset = &state.preview_offset;
    let lines_to_show = context * 2 + 1;

    let percentage = if no_results {
        0
    } else {
        let total = state.document_lines.len() as f64;
        let current = srow as f64;
        let percent = (current / total) * 100.0;
        percent as usize
    };

    queue!(
        std::io::stdout(),
        MoveToNextLine(1),
        Clear(ClearType::CurrentLine),
        Print(format!(
            "[{}/{}] {}%",
            state.search_result_index,
            state.search_results.len(),
            percentage
        )),
        MoveToNextLine(1),
        Print(&separator),
        MoveToNextLine(1),
    )?;


    for (num, line) in state
        .document_lines
        .iter()
        .skip(if srow == 0 {
            0
        } else {
            srow.saturating_sub(context)
        })
        .take(lines_to_show)
    {
        // let line = format!("{:0width$}", line, width = 6);

        if *num == srow && !no_results {
            queue!(
                std::io::stdout(),
                SetBackgroundColor(Color::Green),
                SetForegroundColor(Color::Black),
                Clear(ClearType::CurrentLine),
                Print(&line),
                ResetColor,
                MoveToNextLine(1),
            )?;
        } else {
            queue!(
                std::io::stdout(),
                Clear(ClearType::CurrentLine),
                Print(&line),
                MoveToNextLine(1),
            )?;
        }
    }

    queue!(
        std::io::stdout(),
        Clear(ClearType::CurrentLine),
        Print(&separator),
    )?;

    Ok(())
}
