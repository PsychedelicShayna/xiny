use std::fmt::Debug;

use crate::tui::point::{self, Point};

pub mod fuzzy;
pub mod regex;
pub mod terms;

pub enum SearchEngine {
    Regex,
    Fuzzy,
    Terms,
}

pub enum SearchResult {
    Regex {
        line: usize,
        matches: Vec<Point>,
    },
    Fuzzy {
        line: usize,
        score: i64,
        indicies: Vec<usize>,
    },
    Terms {
        line: usize,
        matches: Vec<Point>,
    },
}

