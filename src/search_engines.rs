use anyhow::{self as ah, Context};

#[derive(Debug)]
pub struct IndexRange {
    start: usize,
    end: usize,
}

#[derive(Debug)]
pub enum SearchEngine {
    Regex,
    Fuzzy,
    Terms,
}

#[derive(Debug, Default)]
pub struct FuzzyMatches {
    pub score: i64,
    pub indicies: Vec<usize>,
}

#[derive(Debug, Default)]
pub struct TermsMatches {
    pub matches: Vec<IndexRange>,
}

#[derive(Debug, Default)]
pub struct RegexMatches {
    pub matches: Vec<IndexRange>,
}

#[derive(Debug)]
pub enum SearchResults {
    Regex(Vec<(usize, RegexMatches)>),
    Fuzzy(Vec<(usize, FuzzyMatches)>),
    Terms(Vec<(usize, TermsMatches)>),
    Nothing,
}

impl SearchEngine {
    pub fn search(&self, lines: &Vec<String>, query: &String) -> ah::Result<SearchResults> {
        let results: SearchResults = match self {
            SearchEngine::Regex => {
                use regex::RegexBuilder;

                let rbuilder = RegexBuilder::new(query);
                let rpattern = rbuilder.build().context("Invalid Regex pattern")?;

                let mut results = Vec::<(usize, RegexMatches)>::new();

                for (line_number, line) in lines.iter().enumerate() {
                    let matches: Vec<IndexRange> = rpattern
                        .find_iter(line)
                        .map(|m| IndexRange {
                            start: m.start(),
                            end: m.end(),
                        })
                        .collect();

                    results.push((line_number, RegexMatches { matches }));
                }

                SearchResults::Regex(results)
            }

            SearchEngine::Fuzzy => {
                use fuzzy_matcher::skim::SkimMatcherV2;
                use fuzzy_matcher::FuzzyMatcher;

                let matcher = SkimMatcherV2::default();
                let mut results = Vec::<(usize, FuzzyMatches)>::new();

                for (line_number, line) in lines.iter().enumerate() {
                    if let Some((score, indicies)) = matcher.fuzzy_indices(line, query) {
                        results.push((line_number, FuzzyMatches { score, indicies }));
                    }
                }

                SearchResults::Fuzzy(results)
            }

            SearchEngine::Terms => {
                let mut results = Vec::<(usize, TermsMatches)>::new();
                let terms: Vec<&str> = query.split_whitespace().collect();

                for (line_number, line) in lines.iter().enumerate() {
                    let result: Vec<IndexRange> = terms
                        .iter()
                        .map(|&term| {
                            line.match_indices(term)
                                .map(|(start, term)| IndexRange {
                                    end: start + term.len(),
                                    start,
                                })
                                .collect::<Vec<IndexRange>>()
                        })
                        .flatten()
                        .collect();

                    results.push((line_number, TermsMatches { matches: result }));
                }

                SearchResults::Terms(results)
            }
        };

        Ok(results)
    }
}
