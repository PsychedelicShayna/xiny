pub(super) mod input_field;
// pub mod previewer;

// // // // // // // // // // // // // // // // // // // // // // // // // //  
// Every component necessarily requires certain modules, or generally requires
// other modules, so rather than repeatedly importing the same common modules
// they can just use super::* and anything specific to their functionality is
// something they can just import themselves. This is to reduce boilerplate.
// // // // // // // // // // // // // // // // // // // // // // // // // //  

/* ---- Standard library -------------------------------------------------- */
pub(in self) use std::fmt::{self, Display, Formatter};
pub(in self) use std::io::{self, stdout, Read, Write};
pub(in self) use std::rc::{self, Rc};

/* ---- Project Specific -------------------------------------------------- */
use super::{
    super::debug,
    boxchars,
    component::{self, Component, DrawError},
    point,
};

/* ---- External Dependencies --------------------------------------------- */
use anyhow::{self as ah, Context};

pub use crossterm::{
    cursor::{self, MoveTo, MoveToNextLine, MoveToPreviousLine},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{self, Color, Colors, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{
        self, Clear,
        ClearType::{self, CurrentLine},
    },
};
