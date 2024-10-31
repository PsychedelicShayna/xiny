use std::fmt::Debug;

pub mod fuzzy;
pub mod regex;
pub mod terms;

pub trait SearchEngine: Debug + Clone + Default {
    /// Search through every line for query, return (row,col) for every match.
    fn search(&mut self, lines: &Vec<String>, query: &str) -> Vec<(usize, usize)>;
}
