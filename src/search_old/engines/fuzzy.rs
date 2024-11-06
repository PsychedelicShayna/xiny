use super::{SearchEngine, SearchResult};

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

impl SearchEngine for FuzzySearch {
    fn search(&mut self, lines: &Vec<String>, query: &str) -> Vec<(usize, usize)> {
        let mut results = Vec::<(usize, usize)>::new();
        let matcher = SkimMatcherV2::default();

        for line in lines {
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
