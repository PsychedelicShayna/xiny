use super::SearchEngine;

use regex::{self, Regex, RegexBuilder};

#[derive(Default, Clone, Debug)]
pub struct RegexSearch;

impl SearchEngine for RegexSearch {
    fn search(&mut self, lines: &Vec<(usize, String)>, query: &str) -> Vec<(usize, usize)> {
        let mut results = Vec::new();

        let Ok(pattern) = RegexBuilder::new(query)
            .case_insensitive(true)
            .unicode(true)
            .build()
        else {
            return results;
        };

        // let Ok(mut pattern) = Regex::new(query) else {
        //     return results;
        // };

        for (line_number, line) in lines {
            if pattern.is_match(line) {
                results.push((*line_number, 0));
            }
        }

        results
    }
}
