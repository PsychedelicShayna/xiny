use std::sync::atomic::Ordering;

use crate::tui::components::input_field::ViMode;

use std::time::Duration;

use crossterm as ct;
use crossterm::event::{self as cte, Event};

use anyhow as ah;

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
pub fn motion_word(str: &String, idx: usize, backward: bool, endwise: bool) -> usize {
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

pub fn handle_inputs(state: &mut TuiState) -> ah::Result<()> {
    let Ok(event_available) = cte::poll(Duration::from_millis(50)) else {
        return Ok(());
    };

    if !event_available {
        return Ok(());
    }

    let Ok(Event::Key(kevent)) = cte::read() else {
        return Ok(());
    };

    // Alias the crossterm key event types for easier access.
    use ct::event::KeyCode as KCode;

    use ct::event::KeyModifiers as KMods;

    // Same with the event's fields.
    let kmods = kevent.modifiers;
    let kcode = kevent.code;
    let kkind = kevent.kind;
    let kstate = kevent.state;

    match (&state.input_field.vi_mode, kcode, kmods) {
        (ViMode::Insert, KCode::Esc | KCode::Enter, KMods::NONE) => {
            state.input_field.vi_mode = ViMode::Normal;
        }

        (ViMode::Insert, KCode::Char(c), _) => {
            // Ensure the cursor is within bounds before inserting, as
            // String::insert panics if the index is out of bounds (..lol)
            if state.search_cursor_index > state.search_buffer.len() {
                state.search_cursor_index = state.search_buffer.len();
            }

            // Insert and increment the cursor index.
            state.search_buffer.insert(state.search_cursor_index, c);
            state.search_cursor_index += 1;
        }

        // Chords: key sequences that require multiple key presses.
        // --------------------------------------------------------------------
        (ViMode::Normal, KCode::Char('d'), KMods::NONE) => match state.input_field.vi_kseq.first() {
            Some('d') => {
                state.search_buffer.clear();
                state.input_field.vi_kseq.clear();
                state.search_cursor_index = 0;
            }
            None => {
                state.input_field.vi_kseq.push('d');
            }

            _ => state.input_field.vi_kseq.clear(),
        },

        (ViMode::Normal, KCode::Char(c), _) if matches!(state.input_field.vi_kseq.first(), Some('r')) => {
            state.input_field.vi_kseq.clear();

            if state.search_cursor_index < state.search_buffer.len() {
                state.search_buffer.remove(state.search_cursor_index);
                state.search_buffer.insert(state.search_cursor_index, c);
            }
        }

        (ViMode::Normal, KCode::Char('r'), KMods::NONE) => {
            if state.input_field.vi_kseq.is_empty() {
                state.input_field.vi_kseq.push('r');
            }
        }

        (ViMode::Normal, KCode::Char('c'), KMods::NONE) => match state.input_field.vi_kseq.first() {
            Some('c') => {
                state.search_buffer.clear();
                state.input_field.vi_kseq.clear();
                state.search_cursor_index = 0;
            }
            None => {
                state.input_field.vi_kseq.push('c');
            }

            _ => state.input_field.vi_kseq.clear(),
        },

        (ViMode::Normal, KCode::Char('w'), KMods::NONE)
            if matches!(state.input_field.vi_kseq.first(), Some('d') | Some('c')) =>
        {
            let change = state.input_field.vi_kseq.first().is_some_and(|&c| c == 'c');

            let buf = &mut state.search_buffer;
            let idx = state.search_cursor_index;

            let words = find_words(buf);

            match words.last() {
                Some(last) if last.0 == idx => {
                    buf.truncate(idx);
                }
                _ => {
                    let new_idx = motion_word(buf, idx, false, false);
                    buf.drain(idx..new_idx);

                    if change {
                        buf.insert(idx, ' ');
                    }
                }
            }

            if change {
                state.input_field.vi_mode = ViMode::Insert;
            }

            state.input_field.vi_kseq.clear();
        }

        (ViMode::Normal, KCode::Char('b'), KMods::NONE)
            if matches!(state.input_field.vi_kseq.first(), Some('d') | Some('c')) =>
        {
            let buf = &mut state.search_buffer;
            let idx = state.search_cursor_index;
            // one two

            let mut new_idx = motion_word(buf, idx, true, false);

            if idx == new_idx {
                new_idx = 0;
            }

            buf.drain(new_idx..idx);

            state.search_cursor_index = new_idx;

            if matches!(state.input_field.vi_kseq.first(), Some('c')) {
                state.input_field.vi_mode = ViMode::Insert;
            }

            state.input_field.vi_kseq.clear();
        }

        (ViMode::Normal, KCode::Char('e'), KMods::NONE)
            if matches!(state.input_field.vi_kseq.first(), Some('g')) =>
        {
            state.input_field.vi_kseq.clear();

            state.search_cursor_index =
                motion_word(&state.search_buffer, state.search_cursor_index, true, true);
        }

        (ViMode::Normal, KCode::Char('g'), KMods::NONE) if state.input_field.vi_kseq.is_empty() => {
            state.input_field.vi_kseq.push('g');
        }

        (ViMode::Normal, KCode::Char('c'), KMods::NONE) => match state.input_field.vi_kseq.first() {
            Some('c') => {
                state.search_buffer.clear();
                state.input_field.vi_kseq.clear();
                state.search_cursor_index = 0;
                state.input_field.vi_mode = ViMode::Insert;
            }
            Some(_) => state.input_field.vi_kseq.clear(),
            None => {
                state.input_field.vi_kseq.push('c');
            }
        },

        // If there's a pending chord that did not progress, clear it.
        // --------------------------------------------------------------------
        (ViMode::Normal, _, KMods::NONE) if !state.input_field.vi_kseq.is_empty() => {
            state.input_field.vi_kseq.clear();
        }

        // --------------------------------------------------------------------
        (ViMode::Normal, KCode::Char('k'), KMods::NONE) => {
            state.preview_jump = false;
            let start_line = &mut state.preview_viewport.start_line;

            if *start_line != 0 {
                *start_line -= 1;
            }
        }

        // Scroll up in the preview by decreasing offset.
        (ViMode::Normal, KCode::Char('j'), KMods::NONE) => {
            state.preview_jump = false;
            let start_line = &mut state.preview_viewport.start_line;
            let end_line = (*start_line + 1) + (state.preview_viewport.context * 2);

            if end_line < state.markdown_lines.len()+1 {
                *start_line += 1;
            }
        }

        (ViMode::Normal, KCode::Char('n'), KMods::NONE) => {
            if (state.search_result_index + 1) < state.search_results.len() {
                state.search_result_index += 1
            } else {
                state.search_result_index = 0
            }

            state.preview_jump = true;
        }

        // Go to the bottom of 
        (ViMode::Normal, KCode::Char('G'), KMods::NONE) => {
            state.preview_jump = false;
            let start_line = &mut state.preview_viewport.start_line;
            let end_line = (*start_line + 1) + (state.preview_viewport.context * 2);

            if end_line < state.markdown_lines.len()+1 {
                *start_line += 1;
            }
        }



        (ViMode::Normal, KCode::Char('N'), KMods::SHIFT) => {
            if state.search_result_index > 0 {
                state.search_result_index -= 1;
            } else {
                state.search_result_index = state.search_results.len() - 1;
            }

            state.preview_jump = true;
        }

        (ViMode::Normal, KCode::Char('q'), KMods::NONE) => {
            state.kill_event_loop = true;
            state.kill_se_thread.store(true, Ordering::SeqCst);
        }

        (ViMode::Normal, KCode::Char('i'), KMods::NONE) => {
            state.input_field.vi_mode = ViMode::Insert;
        }

        (ViMode::Normal, KCode::Char('h'), KMods::NONE) => {
            if state.search_cursor_index > 0 {
                state.search_cursor_index -= 1;
            }
        }

        (ViMode::Insert, KCode::Backspace, KMods::NONE) => {
            if state.search_cursor_index > 0 {
                state.search_buffer.remove(state.search_cursor_index - 1);
                state.search_cursor_index -= 1;
            }
        }

        (ViMode::Normal, KCode::Char('x'), KMods::NONE) => {
            if state.search_cursor_index < state.search_buffer.len() {
                state.search_buffer.remove(state.search_cursor_index);
            }
        }

        // this is a sentence
        (ViMode::Normal, KCode::Char('w'), KMods::NONE) => {
            let buf = &mut state.search_buffer;
            let idx = &mut state.search_cursor_index;

            let new_idx = motion_word(buf, *idx, false, false);

            if new_idx == *idx && !buf.is_empty() {
                *idx = buf.len() - 1;
            } else {
                *idx = new_idx;
            }
        }

        (ViMode::Normal, KCode::Char('e'), KMods::NONE) => {
            let buf = &state.search_buffer;
            let idx = &mut state.search_cursor_index;

            *idx = motion_word(buf, *idx, false, true);
        }

        (ViMode::Normal, KCode::Char('C'), KMods::SHIFT) => {
            state.search_buffer.truncate(state.search_cursor_index);
            state.input_field.vi_mode = ViMode::Insert;
        }

        (ViMode::Normal, KCode::Char('D'), KMods::SHIFT) => {
            state.search_buffer.truncate(state.search_cursor_index);
        }

        (ViMode::Normal, KCode::Char('0'), KMods::NONE) => {
            state.search_cursor_index = 0;
        }

        (ViMode::Normal, KCode::Char('$'), KMods::NONE) => {
            state.search_cursor_index = state.search_buffer.len() - 1;
        }

        (ViMode::Normal, KCode::Char('A'), KMods::SHIFT) => {
            state.search_cursor_index = state.search_buffer.len();
            state.input_field.vi_mode = ViMode::Insert;
        }

        (ViMode::Normal, KCode::Char('a'), KMods::NONE) => {
            state.search_cursor_index += 1;

            if state.search_cursor_index >= state.search_buffer.len() {
                state.search_buffer.push(' ');
            }

            state.input_field.vi_mode = ViMode::Insert;
        }

        (ViMode::Normal, KCode::Char('b'), KMods::NONE) => {
            let buf = &state.search_buffer;
            let idx = &mut state.search_cursor_index;

            let new_idx = motion_word(buf, *idx, true, false);

            if *idx == new_idx {
                *idx = 0;
            } else {
                *idx = new_idx;
            }
        }

        (ViMode::Normal, KCode::Char('l'), KMods::NONE) => {
            if state.search_cursor_index < state.search_buffer.len() {
                state.search_cursor_index += 1;
            }
        }

        _ => (),
    }

    Ok(())
}
