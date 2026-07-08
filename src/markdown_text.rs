use iced::advanced::text::Highlighter;
use iced::widget::text_editor;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::ops::Range;

pub struct RenderedBlock {
    pub kind: RenderedBlockKind,
    pub text: String,
    pub content: text_editor::Content,
    pub links: Vec<RenderedLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedLink {
    pub range: Range<usize>,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderedBlockKind {
    Paragraph,
    Heading(u8),
    ListItem,
    Quote,
    Code { language: Option<String> },
    Rule,
    Table,
}

impl RenderedBlock {
    fn new(kind: RenderedBlockKind, text: String, links: Vec<RenderedLink>) -> Self {
        Self {
            content: text_editor::Content::with_text(&text),
            kind,
            text,
            links,
        }
    }

    pub fn link_at_cursor(&self) -> Option<&str> {
        let cursor = self.content.cursor().position;
        let line_offset = self
            .text
            .split_inclusive('\n')
            .take(cursor.line)
            .map(str::len)
            .sum::<usize>();
        let offset = line_offset + cursor.column;

        self.links
            .iter()
            .find(|link| link.range.contains(&offset))
            .map(|link| link.url.as_str())
    }

    pub fn link_highlights(&self) -> LinkHighlightSettings {
        let mut lines = vec![Vec::new(); self.text.lines().count().max(1)];
        let mut line_start = 0;

        for (line_index, line) in self.text.split('\n').enumerate() {
            let line_end = line_start + line.len();
            for link in &self.links {
                let start = link.range.start.max(line_start);
                let end = link.range.end.min(line_end);
                if start < end {
                    lines[line_index].push((start - line_start)..(end - line_start));
                }
            }
            line_start = line_end + 1;
        }

        LinkHighlightSettings { lines }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkHighlightSettings {
    lines: Vec<Vec<Range<usize>>>,
}

pub struct LinkHighlighter {
    settings: LinkHighlightSettings,
    line: usize,
}

impl Highlighter for LinkHighlighter {
    type Settings = LinkHighlightSettings;
    type Highlight = ();
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, ())>;

    fn new(settings: &Self::Settings) -> Self {
        Self {
            settings: settings.clone(),
            line: 0,
        }
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        self.settings = new_settings.clone();
        self.line = 0;
    }

    fn change_line(&mut self, line: usize) {
        self.line = line;
    }

    fn highlight_line(&mut self, _line: &str) -> Self::Iterator<'_> {
        let ranges = self
            .settings
            .lines
            .get(self.line)
            .cloned()
            .unwrap_or_default();
        self.line += 1;
        ranges
            .into_iter()
            .map(|range| (range, ()))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn current_line(&self) -> usize {
        self.line
    }
}

struct ListState {
    next: Option<u64>,
}

pub fn selectable_text(markdown: &str) -> String {
    let blocks = rendered_blocks(markdown);
    let mut text = String::new();
    let mut previous_kind: Option<&RenderedBlockKind> = None;

    for block in &blocks {
        if let Some(kind) = previous_kind {
            text.push_str(block_gap(kind, &block.kind));
        }
        text.push_str(&block.text);
        previous_kind = Some(&block.kind);
    }

    text
}

pub fn rendered_blocks(markdown: &str) -> Vec<RenderedBlock> {
    let parser = Parser::new_ext(markdown, Options::all());
    let mut blocks = Vec::new();
    let mut text = String::new();
    let mut lists: Vec<ListState> = Vec::new();
    let mut current = CurrentBlock::None;
    let mut quote_depth = 0usize;
    let mut table_cell_open = false;
    let mut links = Vec::new();
    let mut open_links: Vec<(usize, String)> = Vec::new();

    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {
                if !matches!(current, CurrentBlock::ListItem | CurrentBlock::Table) {
                    current = if quote_depth > 0 {
                        CurrentBlock::Quote
                    } else {
                        CurrentBlock::Paragraph
                    };
                    text.clear();
                    links.clear();
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if matches!(current, CurrentBlock::Paragraph | CurrentBlock::Quote) {
                    push_text_block(&mut blocks, current.kind(), &mut text, &mut links);
                    current = CurrentBlock::None;
                }
            }
            Event::Start(Tag::Heading { level, .. }) => {
                current = CurrentBlock::Heading(heading_level(level));
                text.clear();
                links.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                push_text_block(&mut blocks, current.kind(), &mut text, &mut links);
                current = CurrentBlock::None;
            }
            Event::Start(Tag::BlockQuote(_)) => quote_depth += 1,
            Event::End(TagEnd::BlockQuote(_)) => quote_depth = quote_depth.saturating_sub(1),
            Event::Start(Tag::CodeBlock(kind)) => {
                current = CurrentBlock::Code {
                    language: code_language(kind),
                };
                text.clear();
                links.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                push_text_block(&mut blocks, current.kind(), &mut text, &mut links);
                current = CurrentBlock::None;
            }
            Event::Start(Tag::List(start)) => lists.push(ListState { next: start }),
            Event::End(TagEnd::List(_)) => {
                lists.pop();
            }
            Event::Start(Tag::Item) => {
                current = CurrentBlock::ListItem;
                text.clear();
                links.clear();
                text.push_str(&"  ".repeat(lists.len().saturating_sub(1)));
                match lists.last_mut().and_then(|list| list.next.as_mut()) {
                    Some(next) => {
                        text.push_str(&format!("{next}. "));
                        *next += 1;
                    }
                    None => text.push_str("- "),
                }
            }
            Event::End(TagEnd::Item) => {
                push_text_block(
                    &mut blocks,
                    RenderedBlockKind::ListItem,
                    &mut text,
                    &mut links,
                );
                current = CurrentBlock::None;
            }
            Event::Start(Tag::Table(_)) => {
                current = CurrentBlock::Table;
                text.clear();
                links.clear();
            }
            Event::End(TagEnd::Table) => {
                push_text_block(&mut blocks, RenderedBlockKind::Table, &mut text, &mut links);
                current = CurrentBlock::None;
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                open_links.push((text.len(), dest_url.into_string()));
            }
            Event::End(TagEnd::Link) => {
                if let Some((start, url)) = open_links.pop()
                    && start < text.len()
                {
                    links.push(RenderedLink {
                        range: start..text.len(),
                        url,
                    });
                }
            }
            Event::Start(Tag::TableRow) => ensure_line_break(&mut text),
            Event::End(TagEnd::TableRow) => {
                table_cell_open = false;
                ensure_line_break(&mut text);
            }
            Event::Start(Tag::TableCell) => {
                if table_cell_open {
                    text.push('\t');
                }
                table_cell_open = true;
            }
            Event::End(TagEnd::TableCell) => {}
            Event::Text(value)
            | Event::Code(value)
            | Event::InlineMath(value)
            | Event::DisplayMath(value) => text.push_str(&value),
            Event::SoftBreak if matches!(current, CurrentBlock::Code { .. }) => text.push('\n'),
            Event::SoftBreak if matches!(current, CurrentBlock::Table) => text.push(' '),
            Event::SoftBreak => text.push(' '),
            Event::HardBreak => text.push('\n'),
            Event::Rule => {
                blocks.push(RenderedBlock::new(
                    RenderedBlockKind::Rule,
                    "---".to_string(),
                    Vec::new(),
                ));
            }
            Event::TaskListMarker(done) => {
                text.push_str(if done { "[x] " } else { "[ ] " });
            }
            Event::FootnoteReference(label) => {
                text.push('[');
                text.push_str(&label);
                text.push(']');
            }
            Event::Html(_) | Event::InlineHtml(_) => {}
            Event::Start(_) | Event::End(_) => {}
        }
    }

    push_text_block(&mut blocks, current.kind(), &mut text, &mut links);

    blocks
}

enum CurrentBlock {
    None,
    Paragraph,
    Heading(u8),
    ListItem,
    Quote,
    Code { language: Option<String> },
    Table,
}

impl CurrentBlock {
    fn kind(&self) -> RenderedBlockKind {
        match self {
            Self::None => RenderedBlockKind::Paragraph,
            Self::Paragraph => RenderedBlockKind::Paragraph,
            Self::Heading(level) => RenderedBlockKind::Heading(*level),
            Self::ListItem => RenderedBlockKind::ListItem,
            Self::Quote => RenderedBlockKind::Quote,
            Self::Code { language } => RenderedBlockKind::Code {
                language: language.clone(),
            },
            Self::Table => RenderedBlockKind::Table,
        }
    }
}

fn push_text_block(
    blocks: &mut Vec<RenderedBlock>,
    kind: RenderedBlockKind,
    text: &mut String,
    links: &mut Vec<RenderedLink>,
) {
    let trim_start = text.len() - text.trim_start().len();
    let value = text.trim().to_string();
    if !value.is_empty() {
        let value_len = value.len();
        let adjusted_links = std::mem::take(links)
            .into_iter()
            .filter_map(|link| {
                let start = link.range.start.saturating_sub(trim_start).min(value_len);
                let end = link.range.end.saturating_sub(trim_start).min(value_len);
                (start < end).then_some(RenderedLink {
                    range: start..end,
                    url: link.url,
                })
            })
            .collect();
        blocks.push(RenderedBlock::new(kind, value, adjusted_links));
    }
    text.clear();
    links.clear();
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn code_language(kind: pulldown_cmark::CodeBlockKind<'_>) -> Option<String> {
    match kind {
        pulldown_cmark::CodeBlockKind::Fenced(info) => info
            .split_whitespace()
            .next()
            .filter(|language| !language.is_empty())
            .map(ToOwned::to_owned),
        pulldown_cmark::CodeBlockKind::Indented => None,
    }
}

fn ensure_line_break(text: &mut String) {
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
}

fn block_gap(previous: &RenderedBlockKind, next: &RenderedBlockKind) -> &'static str {
    match (previous, next) {
        (RenderedBlockKind::ListItem, RenderedBlockKind::ListItem) => "\n",
        (RenderedBlockKind::Table, RenderedBlockKind::Table) => "\n",
        _ => "\n\n",
    }
}

#[cfg(test)]
mod tests {
    use super::{RenderedBlockKind, rendered_blocks, selectable_text};

    #[test]
    fn extracts_readable_text_from_common_markdown() {
        let text =
            selectable_text("# Title\n\nA **bold** [link](https://example.com).\n\n- One\n- Two");

        assert_eq!(text, "Title\n\nA bold link.\n\n- One\n- Two");
    }

    #[test]
    fn creates_selectable_rendered_blocks() {
        let blocks = rendered_blocks("# Title\n\nBody\n\n```rust\nfn main() {}\n```");

        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].kind, RenderedBlockKind::Heading(1));
        assert_eq!(blocks[0].text, "Title");
        assert_eq!(blocks[1].kind, RenderedBlockKind::Paragraph);
        assert_eq!(
            blocks[2].kind,
            RenderedBlockKind::Code {
                language: Some("rust".to_string()),
            }
        );
    }

    #[test]
    fn tracks_link_ranges_in_rendered_text() {
        let blocks = rendered_blocks("Read [the guide](https://example.com/guide) today.");

        assert_eq!(
            &blocks[0].text[blocks[0].links[0].range.clone()],
            "the guide"
        );
        assert_eq!(blocks[0].links[0].url, "https://example.com/guide");
    }
}
