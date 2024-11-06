use super::SearchEngine;

#[derive(Debug, Clone, Default)]
pub struct TermSearch {
    x: u32,
}

impl SearchEngine for TermSearch {
    /// Search for the query in the given lines and return the line number and index of the first match.
    fn search(&mut self, lines: &Vec<(usize, String)>, query: &str) -> Vec<(usize, usize)> {
        let query = query.to_ascii_lowercase();
        let terms = query.split_whitespace().collect::<Vec<&str>>();

        lines
            .into_iter()
            .fold(Vec::<(usize, usize)>::new(), |mut acc, (line_num, line)| {
                let line = line.to_ascii_lowercase();

                // for term in &terms {
                    if line.contains(&query) {
                        acc.push((*line_num, 0));
                    }
                // }

                acc
            })

        // let terms = query.split_whitespace().collect::<Vec<&str>>();
        //
        // lines
        //     .iter()
        //     .fold(Vec::<(usize, usize)>::new(), |mut acc, (line_num, line)| {
        //         for term in &terms {
        //             if let Some(index) = line.find(term) {
        //                 acc.push((*line_num, index));
        //                 return acc;
        //             }
        //         }
        //
        //         acc
        //     })
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::search::engines::*;
//
//     use super::*;
//
//     #[test]
//     fn test_search() {
//         let query = "ghi mno";
//         let lines = vec!["abc", "defghi", "jkl", "mno"];
//         let results = search.search(&lines, query);
//         assert_eq!(results, vec![(1, 3), (3, 0)]);
//
//         // Additional test cases
//         let lines_empty = vec![];
//         let query_empty = "";
//         let results_empty = search.search(&lines_empty, query_empty);
//         assert_eq!(results_empty, vec![]);
//
//         let lines_single_line = vec!["single line"];
//         let query_single_line = "single";
//         let results_single_line = search.search(&lines_single_line, query_single_line);
//         assert_eq!(results_single_line, vec![(0, 0)]);
//
//         let lines_multiple_queries = vec!["abc", "defghi", "jkl", "mno"];
//         let queries_multiple = ["ghi mno", "jkl mno"];
//         let results_multiple = search.search(&lines_multiple_queries, queries_multiple[0]);
//         assert_eq!(results_multiple, vec![(1, 3), (3, 0)]);
//
//         let lines_case_insensitive = vec!["ABC", "DEF", "GHI", "JKL"];
//         let query_case_insensitive = "def ghi";
//         let results_case_insensitive = search.search(&lines_case_insensitive, query_case_insensitive);
//         assert_eq!(results_case_insensitive, vec![(0, 0), (1, 0)]);
//     }
// }
//
// In this updated version, I've added four additional test cases:
//
// 1. An empty search case
// 2. A single-line search case
// 3. A multiple query search case
// 4. A case-insensitive search case
//
// These additional test cases cover various scenarios that might occur in real-world usage, such as searching with no results, searching a single line, searching with multiple queries, and performing case-insensitive searches.
// }
