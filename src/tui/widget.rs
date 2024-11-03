use std::fmt::{Display, Formatter};
use std::rc::Rc;

use super::event_loop::TuiState;

use anyhow as ah;
use crossterm::event::Event;

#[derive(Debug, Clone)]
pub enum DrawError {
    NoSpace { need: (u16, u16), have: (u16, u16) },
    DrawingDisabled
}

impl Display for DrawError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::NoSpace { need, have } => {
                write!(
                    f,
                    "Require {}x{} to draw, but terminal only has {}x{}",
                    need.0, need.1, have.0, have.1
                )
            }
            DrawError::DrawingDisabled => todo!(),

        }
    }
}

pub trait Widget {
    fn new(parent: Rc<TuiState>) -> Self;

    /// Must queue! the commands needed to draw itself using crossterm.
    fn queue_draws(&self) -> ah::Result<()>;

    /// Must queue! the commands needed to clear itself using crossterm.
    fn queue_clear(&self) -> ah::Result<()>;

    /// Must be able to handle events thrown its way, and update itself.
    fn handle_event(&mut self, event: Event) -> ah::Result<()>;

    /// Enable this widget; if disabled, does not render, clear, or respond.
    fn set_enabled(&mut self, enabled: bool);

    /// Return the rows and columns strictly necessary to draw this widget.
    /// If there isn't at least this much available, then it can't draw.
    fn get_min_size(&self) -> (u16, u16);

    /// Return the actual amount of rows and columns this widget would like
    /// to occupy, based on its internal state, and the state of the parent.
    fn get_size(&self) -> (u16, u16);
}
