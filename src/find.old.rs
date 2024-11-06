use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{self, AtomicBool, AtomicI64};
use std::sync::Arc;

use crossterm::{self as ct, queue};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::style::{ContentStyle, PrintStyledContent, StyledContent};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, Clear, ClearType, DisableLineWrap,
    EnableLineWrap, EnterAlternateScreen, ScrollDown, ScrollUp, SetSize, WindowSize,
};
use std::collections::HashSet;

use crate::xiny::*;
use crate::futile::*;

use anyhow::{self as ah, Context};

pub enum FindMethod {
    Fuzzy(String),
    Regex(Vec<String>),
    Terms(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum LineLengthLimit {
    // Positive values will limit the line length to that number of characters,
    // negative values will subtract from the terminal's width, and a value of
    // 0 will use the whole available width without risking line wrap.
    TermOffset(isize),

    /// Line cannot exceed N% of the terminal's width.
    TermPercent(usize),
}

impl LineLengthLimit {
    pub fn calculate_limit(&self) -> ah::Result<usize> {
        match self {
            LineLengthLimit::TermOffset(offset) => {
                let (width, _): (u16, u16) = ct::terminal::size()?;
                let width: isize = width as isize;
                let offset: isize = *offset;

                if offset < 0 {
                    return Ok((width - offset) as usize);
                }

                Ok(offset as usize)
            }

            LineLengthLimit::TermPercent(percentage) => {
                if *percentage > 100 {
                    ah::bail!("Percentage cannot exceed 100");
                }

                let (width, _): (u16, u16) = ct::terminal::size()?;

                let width = width as f64;
                let percentage = *percentage as f64;

                // It seems redundant, but it's better to be explicit about
                // important constraints that would cause problems if violated.
                // Another developer might not realize that implicit behavior
                // is being relied upon for the program to function correctly.
                let limit: f64 = (width * (percentage.floor() / 100.0)).round();

                // Implicit behavior of casting to usize is not as universal
                // as floor or round; floor always means floor, round always
                // means round. You do no have to guess if the developer is
                // intentionally making use of implicit behavior or not.
                Ok(limit as usize)
            }
        }
    }
}


#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MatchedLine {
    pub file: PathBuf,
    pub line_num: usize,
    pub content: String,
    pub length: usize,
}

impl MatchedLine {
    pub fn new(file: PathBuf) -> Self {
        Self {
            file,
            ..Default::default()
        }
    }
}
