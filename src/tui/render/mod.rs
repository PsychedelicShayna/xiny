use std::io::{stdout, Write};

use super::event_loop::TuiState;

pub mod input_field;
pub mod previewer;

use anyhow::{self as ah, Context};

use crossterm::{
    cursor::{self},
    queue,
};

pub const BHCL: char = '─';
pub const BHJL: char = '┤';
pub const BHJO: char = '┼';
pub const BHJR: char = '├';
pub const BVCL: char = '│';
pub const BVJD: char = '┬';
pub const BVJU: char = '┴';
pub const BCTL: char = '┐';
pub const BCBL: char = '└';
pub const BCTR: char = '┌';
pub const BCBR: char = '┘';

pub fn components(state: &TuiState, anchor: &(usize, usize)) -> ah::Result<()> {
    queue!(stdout(), cursor::MoveTo(anchor.0 as u16, anchor.1 as u16))?;

    stdout().flush()?;

    previewer::render_previewer(state, anchor).context("Attempted to render previewer component.")?;
    input_field::render(state, anchor).context("Attempted to render input_field component.")?;

    stdout().flush()?;
    Ok(())
}

pub fn cleanup() -> ah::Result<()> {
    eprintln!("Cleanup not implemented yet");
    Ok(())
}
