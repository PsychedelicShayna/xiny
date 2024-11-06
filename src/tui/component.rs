use std::fmt::{Debug, Display, Formatter};
use std::io::stdout;
use std::rc::Rc;

use super::point::Point;

use anyhow as ah;
use crossterm::event::Event;

use crossterm::{self as ct, execute};

#[derive(Debug, Clone)]
pub enum DrawError {
    NoSpace { need: Point, have: Point },
    DrawingDisabled,
}

impl Display for DrawError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::NoSpace { need, have } => {
                write!(
                    f,
                    "Require {} to draw, but terminal only has {}",
                    need, have
                )
            }
            DrawError::DrawingDisabled => todo!(),
        }
    }
}

/// Checks if the cursor is out of bounds, and resets to the top left, and if
/// that works, then se it to the bottom left.
pub fn cursor_ok() -> bool {
    ct::cursor::position().is_ok()
}

pub fn cursor_reset() -> ah::Result<()> {
    execute!(stdout(), ct::cursor::MoveTo(0, 0))?;
    Ok(())
}

pub trait Component: Debug {
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
    fn get_min_size(&self) -> Point;

    /// Return the actual amount of rows and columns this widget would like
    /// to occupy, based on its internal state, and the state of the parent.
    fn get_size(&self) -> Point;
}
