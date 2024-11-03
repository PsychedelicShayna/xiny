use std::ops::Deref;

use super::*;

// Renders the search input field.

use crate::tui::event_loop::{TuiState};

use crossterm::{
    cursor::MoveToNextLine, event::KeyCode, queue, style::{Color, Colors, Print, ResetColor, SetBackgroundColor, SetForegroundColor}, terminal::{Clear, ClearType}
};

#[derive(Clone, Debug, Default)]
pub struct ViewPort {
    pub start_line: usize,
    pub context: usize,
}


pub fn render_previewer(state: &mut TuiState) -> ah::Result<()> {
    let dimensions = &state.terminal_size;
    let separator = BHCL.to_string().repeat(dimensions.1 as usize);
    let no_search_results = state.search_results.is_empty();

    let matched_row: Option<usize> = state
        .search_results
        .get(state.search_result_index)
        .map(|(row, _)| *row);

    let line_range: &[(usize, String)] = match state.search_results.get(state.search_result_index) {
        Some((row, _)) if state.preview_jump => {
            let context = state.preview_viewport.context;

            let mut start: isize;
            let mut end: isize;

            start = *row as isize - context as isize;
            end = *row as isize + context as isize;

            

            if start < 0 {
                end += start.abs();
                start = 0
            }

            if end >= state.doc_lines.len() as isize {
                let delta = end - (state.doc_lines.len()) as isize;
                start -= delta;

                end = (state.doc_lines.len() - 1) as isize;
            }

            state.preview_viewport.start_line = start as usize;

            &state.doc_lines[start as usize..end as usize]
        }
        _ => {
            let mut start: usize = state.preview_viewport.start_line;
            let mut end: usize = start + (state.preview_viewport.context * 2);

            // start = start.saturating_sub(state.preview_context);
            // end = end.saturating_add(state.preview_context);
            //
            // if end > (state.document_lines.len()) {
            //     let delta = end - (state.document_lines.len());
            //     start = start.saturating_sub(delta);
            //     end = state.document_lines.len();
            // }

            &state.doc_lines[start..end]
        }
    };

    queue!(
        std::io::stdout(),
        MoveToNextLine(1),
        Clear(ClearType::CurrentLine),
        Print(format!(
            "[{}/{}]",
            state.search_result_index,
            state.search_results.len(),
            // (state.preview_viewport.start_line / state.document_lines.len()) * 100
        )),
        MoveToNextLine(1),
        Print(&separator),
        MoveToNextLine(1),
    )?;

    for (num, line) in line_range {
        let mut formatted = format!("{:0pad$}: {}", num, line, pad = state.linum_digits);

        if formatted.len() > dimensions.1 as usize {
            formatted.truncate(dimensions.1 as usize - 2);
            formatted.push_str("..");
        }

        if matched_row.is_some_and(|row| row == *num) && !no_search_results {
            queue!(
                std::io::stdout(),
                SetBackgroundColor(Color::Green),
                SetForegroundColor(Color::Black),
                Clear(ClearType::CurrentLine),
                Print(&formatted),
                ResetColor,
                MoveToNextLine(1),
            )?;
        } 

        else if state.search_results.iter().any(|(row, _)| row == num) {
            queue!(
                std::io::stdout(),
                // SetBackgroundColor(Color::DarkGrey),
                SetForegroundColor(Color::Green),
                Clear(ClearType::CurrentLine),
                Print(&formatted),
                ResetColor,
                MoveToNextLine(1),
            )?;
        }

        else {
            queue!(
                std::io::stdout(),
                Clear(ClearType::CurrentLine),
                Print(&formatted),
                MoveToNextLine(1),
            )?;
        }
    }

    queue!(
        std::io::stdout(),
        Clear(ClearType::CurrentLine),
        Print(&separator),
    )?;

    // for (num, line) in state
    //     .document_lines
    //     .iter()
    //     .skip(if srow == 0 {
    //         0
    //     } else {
    //         let mut base = srow.saturating_sub(context);
    //
    //         if offset.is_negative() {
    //             base = base.saturating_sub(offset.abs() as usize);
    //         } else {
    //             base = base.saturating_add(*offset as usize);
    //         }
    //
    //         base
    //     })
    //     .take(lines_to_show)
    // {
    //     let line = format!("{:0pad$}: {}", num, line, pad = state.preview_linum_pad);
    //
    //     if *num == srow && !no_results {
    //         queue!(
    //             std::io::stdout(),
    //             SetBackgroundColor(Color::Green),
    //             SetForegroundColor(Color::Black),
    //             Clear(ClearType::CurrentLine),
    //             Print(&line),
    //             ResetColor,
    //             MoveToNextLine(1),
    //         )?;
    //     } else {
    //         queue!(
    //             std::io::stdout(),
    //             Clear(ClearType::CurrentLine),
    //             Print(&line),
    //             MoveToNextLine(1),
    //         )?;
    //     }
    // }

    Ok(())
}
