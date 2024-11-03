use super::SearchEngine;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

#[derive(Default, Clone, Debug)]
pub struct FuzzySearch;

impl SearchEngine for FuzzySearch {
    fn search(&mut self, lines: &Vec<(usize, String)>, query: &str) -> Vec<(usize, usize)> {
        let mut results = Vec::<(usize, usize)>::new();

        let matcher = SkimMatcherV2::default();

        for (line_number, line) in lines {
            if let Some((_, indicies)) = matcher.fuzzy_indices(line, query) {
                let Some(index) = indicies.first() else {
                    continue;
                };

                results.push((*line_number, *index));
            }
        }

        results
    }
}
