//! Case-insensitive search across the open document.

use crate::markdown_text::{RenderedBlock, RenderedBlockKind};
use std::ops::Range;

/// One hit of the current query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindMatch {
    /// Index of the rendered block holding the match; always 0 in the
    /// source view, where the whole file is one text.
    pub block: usize,
    /// The table cell holding the match as `(row, column)`, when the block
    /// is a table.
    pub cell: Option<(usize, usize)>,
    /// Byte range of the match within the block or cell text.
    pub range: Range<usize>,
}

#[derive(Debug, Default)]
pub struct FindState {
    pub open: bool,
    pub query: String,
    pub matches: Vec<FindMatch>,
    pub current: usize,
}

impl FindState {
    /// The match the search bar and highlights point at.
    pub fn current_match(&self) -> Option<&FindMatch> {
        self.matches.get(self.current)
    }
}

/// Byte ranges of every non-overlapping, case-insensitive occurrence of
/// `query` in `text`.
pub fn matches_in(text: &str, query: &str) -> Vec<Range<usize>> {
    let needle: Vec<char> = query.chars().collect();
    if needle.is_empty() {
        return Vec::new();
    }
    let haystack: Vec<(usize, char)> = text.char_indices().collect();

    let mut matches = Vec::new();
    let mut index = 0;
    while index + needle.len() <= haystack.len() {
        let hit = needle
            .iter()
            .enumerate()
            .all(|(offset, needle_char)| chars_match(haystack[index + offset].1, *needle_char));
        if hit {
            let (start, _) = haystack[index];
            let (last_index, last_char) = haystack[index + needle.len() - 1];
            matches.push(start..last_index + last_char.len_utf8());
            index += needle.len();
        } else {
            index += 1;
        }
    }
    matches
}

fn chars_match(a: char, b: char) -> bool {
    a == b || a.to_lowercase().eq(b.to_lowercase())
}

/// Every match of `query` across the rendered blocks, in document order.
/// Rules and images carry no readable text and are skipped.
pub fn rendered_matches(blocks: &[RenderedBlock], query: &str) -> Vec<FindMatch> {
    let mut matches = Vec::new();
    for (block_index, block) in blocks.iter().enumerate() {
        match &block.kind {
            RenderedBlockKind::Rule | RenderedBlockKind::Image { .. } => {}
            RenderedBlockKind::Table(table) => {
                for (row_index, row) in table.rows.iter().enumerate() {
                    for (column_index, cell) in row.iter().enumerate() {
                        matches.extend(matches_in(&cell.text, query).into_iter().map(|range| {
                            FindMatch {
                                block: block_index,
                                cell: Some((row_index, column_index)),
                                range,
                            }
                        }));
                    }
                }
            }
            _ => {
                matches.extend(
                    matches_in(&block.text, query)
                        .into_iter()
                        .map(|range| FindMatch {
                            block: block_index,
                            cell: None,
                            range,
                        }),
                );
            }
        }
    }
    matches
}

/// Every match of `query` in the raw source, as single-block matches.
pub fn source_matches(content: &str, query: &str) -> Vec<FindMatch> {
    matches_in(content, query)
        .into_iter()
        .map(|range| FindMatch {
            block: 0,
            cell: None,
            range,
        })
        .collect()
}

/// The zero-based line and byte column of a byte offset in `text`, in the
/// form a text editor cursor expects.
pub fn line_column(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset.min(text.len())];
    let line = before.matches('\n').count();
    let line_start = before.rfind('\n').map(|index| index + 1).unwrap_or(0);
    (line, offset - line_start)
}

#[cfg(test)]
mod tests {
    use super::{line_column, matches_in, rendered_matches};
    use crate::markdown_text::rendered_blocks;
    use std::path::Path;

    #[test]
    fn finds_case_insensitive_matches() {
        assert_eq!(
            matches_in("Hello hello HELLO", "hello"),
            vec![0..5, 6..11, 12..17]
        );
        assert_eq!(
            matches_in("straße", "SS"),
            Vec::<std::ops::Range<usize>>::new()
        );
        assert_eq!(matches_in("Ünïcode ünïcode", "ünï"), vec![0..5, 10..15]);
        assert!(matches_in("anything", "").is_empty());
        assert!(matches_in("short", "much longer").is_empty());
    }

    #[test]
    fn matches_do_not_overlap() {
        assert_eq!(matches_in("aaaa", "aa"), vec![0..2, 2..4]);
    }

    #[test]
    fn locates_lines_and_columns() {
        let text = "first\nsecond line\nthird";
        assert_eq!(line_column(text, 0), (0, 0));
        assert_eq!(line_column(text, 6), (1, 0));
        assert_eq!(line_column(text, 13), (1, 7));
        assert_eq!(line_column(text, text.len()), (2, 5));
    }

    #[test]
    fn searches_blocks_and_table_cells() {
        let blocks = rendered_blocks(
            "# Alpha\n\nalpha beta\n\n| Name | Value |\n| --- | --- |\n| alpha | gamma |\n\n---",
            Path::new("/docs"),
        );
        let matches = rendered_matches(&blocks, "alpha");

        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].block, 0);
        assert_eq!(matches[1].block, 1);
        assert_eq!(matches[2].block, 2);
        assert_eq!(matches[2].cell, Some((1, 0)));
        assert_eq!(matches[2].range, 0..5);
    }
}
