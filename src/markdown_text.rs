use iced::advanced::text::Highlighter;
use iced::widget::text_editor;
use pulldown_cmark::{
    Alignment, BlockQuoteKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};
use std::ops::Range;
use std::path::{Path, PathBuf};
use unicode_width::UnicodeWidthStr;

pub struct RenderedBlock {
    pub kind: RenderedBlockKind,
    pub text: String,
    pub content: text_editor::Content,
    pub spans: Vec<RenderedSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedSpan {
    pub range: Range<usize>,
    pub kind: SpanKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanKind {
    Link(String),
    Strong,
    Emphasis,
    Code,
    Strike,
    Dim,
    /// Structural glyphs (list bullets, numbers, task boxes, alert labels)
    /// rendered in the theme accent to guide the eye.
    Marker,
    /// A hit of the find-in-document query.
    Match,
    /// The find hit the search bar currently points at.
    CurrentMatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderedBlockKind {
    Paragraph,
    Heading(u8),
    ListItem,
    Quote,
    Code { language: Option<String> },
    Rule,
    Table(TableData),
    Image { source: ImageSource, alt: String },
}

/// The cells of a rendered table; the first row is the header. The block's
/// `text` keeps an aligned monospace rendering of the same cells for copying.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableData {
    pub alignments: Vec<CellAlign>,
    pub rows: Vec<Vec<TableCell>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableCell {
    pub text: String,
    pub spans: Vec<RenderedSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageSource {
    Local(PathBuf),
    Remote(String),
}

fn resolve_image_source(url: &str, base_dir: &Path) -> ImageSource {
    if url.starts_with("http://") || url.starts_with("https://") {
        ImageSource::Remote(url.to_string())
    } else {
        let path = Path::new(url.strip_prefix("file://").unwrap_or(url));
        ImageSource::Local(if path.is_absolute() {
            path.to_path_buf()
        } else {
            base_dir.join(path)
        })
    }
}

impl RenderedBlock {
    fn new(kind: RenderedBlockKind, text: String, spans: Vec<RenderedSpan>) -> Self {
        Self {
            content: text_editor::Content::with_text(&text),
            kind,
            text,
            spans,
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

        self.spans.iter().find_map(|span| match &span.kind {
            SpanKind::Link(url) if span.range.contains(&offset) => Some(url.as_str()),
            _ => None,
        })
    }

    /// Highlight settings for this block's inline styling plus any extra
    /// `marks` (search hits) laid over it.
    pub fn span_highlights(&self, marks: &[RenderedSpan]) -> SpanHighlightSettings {
        let spans: Vec<&RenderedSpan> = self.spans.iter().chain(marks).collect();
        highlight_settings(&self.text, &spans)
    }
}

/// A run of text sharing one combined style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment<'a> {
    pub range: Range<usize>,
    pub style: SpanStyle,
    pub link: Option<&'a str>,
}

/// Splits `text` into contiguous runs, each carrying the combined style of
/// every span covering it. Plain runs are included, so the runs tile the
/// whole text.
pub fn styled_segments<'a>(
    text: &str,
    spans: &'a [RenderedSpan],
    marks: &'a [RenderedSpan],
) -> Vec<Segment<'a>> {
    let all: Vec<&RenderedSpan> = spans.iter().chain(marks).collect();
    segments_between(0, text.len(), &all, true)
}

/// Per-line highlight ranges for a text editor, one entry per line of
/// `text`. Editors highlight line by line, so spans are clipped to lines.
pub fn highlight_settings(text: &str, spans: &[&RenderedSpan]) -> SpanHighlightSettings {
    let mut lines = Vec::new();
    let mut line_start = 0;

    for line in text.split('\n') {
        let line_end = line_start + line.len();
        let segments = segments_between(line_start, line_end, spans, false)
            .into_iter()
            .map(|segment| {
                (
                    (segment.range.start - line_start)..(segment.range.end - line_start),
                    segment.style,
                )
            })
            .collect();
        lines.push(segments);
        line_start = line_end + 1;
    }

    SpanHighlightSettings { lines }
}

/// Highlight ranges must not overlap, so the spans touching `start..end`
/// are flattened into disjoint runs carrying the combined style of every
/// span covering them.
fn segments_between<'a>(
    start: usize,
    end: usize,
    spans: &[&'a RenderedSpan],
    include_plain: bool,
) -> Vec<Segment<'a>> {
    let overlapping: Vec<&RenderedSpan> = spans
        .iter()
        .copied()
        .filter(|span| span.range.start < end && span.range.end > start)
        .collect();

    let mut bounds: Vec<usize> = vec![start, end];
    bounds.extend(overlapping.iter().flat_map(|span| {
        [
            span.range.start.clamp(start, end),
            span.range.end.clamp(start, end),
        ]
    }));
    bounds.sort_unstable();
    bounds.dedup();

    let mut segments = Vec::new();
    for pair in bounds.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let mut style = SpanStyle::default();
        let mut link = None;
        for span in &overlapping {
            if span.range.start <= from && span.range.end >= to {
                style.apply(&span.kind);
                if let SpanKind::Link(url) = &span.kind {
                    link = Some(url.as_str());
                }
            }
        }
        if include_plain || !style.is_plain() {
            segments.push(Segment {
                range: from..to,
                style,
                link,
            });
        }
    }
    segments
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpanStyle {
    pub link: bool,
    pub strong: bool,
    pub emphasis: bool,
    pub code: bool,
    pub strike: bool,
    pub dim: bool,
    pub marker: bool,
    /// Part of a find-in-document hit.
    pub search: bool,
    /// Part of the find hit the search bar points at.
    pub current: bool,
}

impl SpanStyle {
    fn apply(&mut self, kind: &SpanKind) {
        match kind {
            SpanKind::Link(_) => self.link = true,
            SpanKind::Strong => self.strong = true,
            SpanKind::Emphasis => self.emphasis = true,
            SpanKind::Code => self.code = true,
            SpanKind::Strike => self.strike = true,
            SpanKind::Dim => self.dim = true,
            SpanKind::Marker => self.marker = true,
            SpanKind::Match => self.search = true,
            SpanKind::CurrentMatch => {
                self.search = true;
                self.current = true;
            }
        }
    }

    fn is_plain(self) -> bool {
        self == Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanHighlightSettings {
    lines: Vec<Vec<(Range<usize>, SpanStyle)>>,
}

pub struct SpanHighlighter {
    settings: SpanHighlightSettings,
    line: usize,
}

impl Highlighter for SpanHighlighter {
    type Settings = SpanHighlightSettings;
    type Highlight = SpanStyle;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, SpanStyle)>;

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
        ranges.into_iter().collect::<Vec<_>>().into_iter()
    }

    fn current_line(&self) -> usize {
        self.line
    }
}

struct ListState {
    next: Option<u64>,
}

struct TableBuilder {
    alignments: Vec<Alignment>,
    rows: Vec<Vec<TableCell>>,
}

pub fn selectable_text(blocks: &[RenderedBlock]) -> String {
    let mut text = String::new();
    let mut previous_kind: Option<&RenderedBlockKind> = None;

    for block in blocks {
        if let Some(kind) = previous_kind {
            text.push_str(copy_gap(kind, &block.kind));
        }
        text.push_str(&block.text);
        previous_kind = Some(&block.kind);
    }

    text
}

pub fn rendered_blocks(markdown: &str, base_dir: &Path) -> Vec<RenderedBlock> {
    let parser = Parser::new_ext(markdown, Options::all());
    let mut blocks = Vec::new();
    let mut text = String::new();
    let mut spans: Vec<RenderedSpan> = Vec::new();
    let mut open_spans: Vec<(usize, SpanKind)> = Vec::new();
    let mut lists: Vec<ListState> = Vec::new();
    let mut current = CurrentBlock::None;
    let mut quote_depth = 0usize;
    let mut table: Option<TableBuilder> = None;
    let mut in_metadata = false;
    let mut open_images: Vec<(usize, String)> = Vec::new();

    for event in parser {
        match event {
            Event::Start(Tag::MetadataBlock(_)) => in_metadata = true,
            Event::End(TagEnd::MetadataBlock(_)) => in_metadata = false,
            _ if in_metadata => {}
            Event::Start(Tag::Paragraph) => {
                if !matches!(current, CurrentBlock::ListItem | CurrentBlock::Table) {
                    current = if quote_depth > 0 {
                        CurrentBlock::Quote
                    } else {
                        CurrentBlock::Paragraph
                    };
                    text.clear();
                    spans.clear();
                    open_spans.clear();
                }
            }
            Event::End(TagEnd::Paragraph) => match current {
                CurrentBlock::Paragraph | CurrentBlock::Quote => {
                    push_text_block(&mut blocks, current.kind(), &mut text, &mut spans);
                    current = CurrentBlock::None;
                }
                // Separate multiple paragraphs inside one list item, aligning
                // continuations under the item text rather than the marker.
                CurrentBlock::ListItem => {
                    text.push('\n');
                    text.push_str(&"    ".repeat(lists.len().saturating_sub(1)));
                    text.push_str("  ");
                }
                _ => {}
            },
            Event::Start(Tag::Heading { level, .. }) => {
                current = CurrentBlock::Heading(heading_level(level));
                text.clear();
                spans.clear();
                open_spans.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                push_text_block(&mut blocks, current.kind(), &mut text, &mut spans);
                current = CurrentBlock::None;
            }
            Event::Start(Tag::BlockQuote(kind)) => {
                quote_depth += 1;
                // GitHub-style alerts (> [!NOTE] etc.) get a bold label line.
                if let Some(kind) = kind {
                    let label = match kind {
                        BlockQuoteKind::Note => "NOTE",
                        BlockQuoteKind::Tip => "TIP",
                        BlockQuoteKind::Important => "IMPORTANT",
                        BlockQuoteKind::Warning => "WARNING",
                        BlockQuoteKind::Caution => "CAUTION",
                    };
                    blocks.push(RenderedBlock::new(
                        RenderedBlockKind::Quote,
                        label.to_string(),
                        vec![
                            RenderedSpan {
                                range: 0..label.len(),
                                kind: SpanKind::Strong,
                            },
                            RenderedSpan {
                                range: 0..label.len(),
                                kind: SpanKind::Marker,
                            },
                        ],
                    ));
                }
            }
            Event::End(TagEnd::BlockQuote(_)) => quote_depth = quote_depth.saturating_sub(1),
            Event::Start(Tag::CodeBlock(kind)) => {
                current = CurrentBlock::Code {
                    language: code_language(kind),
                };
                text.clear();
                spans.clear();
                open_spans.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                push_text_block(&mut blocks, current.kind(), &mut text, &mut spans);
                current = CurrentBlock::None;
            }
            Event::Start(Tag::List(start)) => {
                // A nested list starting inside an item: flush the item's own
                // text first so it isn't lost when the nested items begin.
                if matches!(current, CurrentBlock::ListItem) && !text.trim().is_empty() {
                    push_text_block(
                        &mut blocks,
                        RenderedBlockKind::ListItem,
                        &mut text,
                        &mut spans,
                    );
                }
                lists.push(ListState { next: start });
            }
            Event::End(TagEnd::List(_)) => {
                lists.pop();
            }
            Event::Start(Tag::Item) => {
                current = CurrentBlock::ListItem;
                text.clear();
                spans.clear();
                open_spans.clear();
                let depth = lists.len().saturating_sub(1);
                text.push_str(&"    ".repeat(depth));
                let marker_start = text.len();
                match lists.last_mut().and_then(|list| list.next.as_mut()) {
                    Some(next) => {
                        text.push_str(&format!("{next}."));
                        *next += 1;
                    }
                    None => text.push_str(unordered_marker(depth)),
                }
                spans.push(RenderedSpan {
                    range: marker_start..text.len(),
                    kind: SpanKind::Marker,
                });
                text.push(' ');
            }
            Event::End(TagEnd::Item) => {
                push_text_block(
                    &mut blocks,
                    RenderedBlockKind::ListItem,
                    &mut text,
                    &mut spans,
                );
                current = CurrentBlock::None;
            }
            Event::Start(Tag::Table(alignments)) => {
                current = CurrentBlock::Table;
                table = Some(TableBuilder {
                    alignments,
                    rows: Vec::new(),
                });
                text.clear();
                spans.clear();
                open_spans.clear();
            }
            Event::End(TagEnd::Table) => {
                if let Some(builder) = table.take() {
                    let (table_text, table_spans) = assemble_table(&builder);
                    if !table_text.is_empty() {
                        blocks.push(RenderedBlock::new(
                            RenderedBlockKind::Table(table_data(builder)),
                            table_text,
                            table_spans,
                        ));
                    }
                }
                text.clear();
                spans.clear();
                current = CurrentBlock::None;
            }
            Event::Start(Tag::TableHead | Tag::TableRow) => {
                if let Some(builder) = table.as_mut() {
                    builder.rows.push(Vec::new());
                }
            }
            Event::Start(Tag::TableCell) => {
                text.clear();
                spans.clear();
                open_spans.clear();
            }
            Event::End(TagEnd::TableCell) => {
                if let Some(row) = table.as_mut().and_then(|builder| builder.rows.last_mut()) {
                    row.push(TableCell {
                        text: std::mem::take(&mut text),
                        spans: std::mem::take(&mut spans),
                    });
                }
            }
            Event::Start(Tag::DefinitionListTitle) => {
                current = CurrentBlock::Paragraph;
                text.clear();
                spans.clear();
                open_spans.clear();
            }
            Event::End(TagEnd::DefinitionListTitle) => {
                if !text.is_empty() {
                    spans.push(RenderedSpan {
                        range: 0..text.len(),
                        kind: SpanKind::Strong,
                    });
                }
                push_text_block(
                    &mut blocks,
                    RenderedBlockKind::Paragraph,
                    &mut text,
                    &mut spans,
                );
                current = CurrentBlock::None;
            }
            Event::Start(Tag::DefinitionListDefinition) => {
                current = CurrentBlock::ListItem;
                text.clear();
                spans.clear();
                open_spans.clear();
                text.push_str("    ");
            }
            Event::End(TagEnd::DefinitionListDefinition) => {
                push_text_block(
                    &mut blocks,
                    RenderedBlockKind::ListItem,
                    &mut text,
                    &mut spans,
                );
                current = CurrentBlock::None;
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                open_spans.push((text.len(), SpanKind::Link(dest_url.into_string())));
            }
            Event::End(TagEnd::Link) => {
                close_span(&mut open_spans, &mut spans, text.len(), |kind| {
                    matches!(kind, SpanKind::Link(_))
                });
            }
            Event::Start(Tag::Strong) => open_spans.push((text.len(), SpanKind::Strong)),
            Event::End(TagEnd::Strong) => {
                close_span(&mut open_spans, &mut spans, text.len(), |kind| {
                    matches!(kind, SpanKind::Strong)
                });
            }
            Event::Start(Tag::Emphasis) => open_spans.push((text.len(), SpanKind::Emphasis)),
            Event::End(TagEnd::Emphasis) => {
                close_span(&mut open_spans, &mut spans, text.len(), |kind| {
                    matches!(kind, SpanKind::Emphasis)
                });
            }
            Event::Start(Tag::Strikethrough) => open_spans.push((text.len(), SpanKind::Strike)),
            Event::End(TagEnd::Strikethrough) => {
                close_span(&mut open_spans, &mut spans, text.len(), |kind| {
                    matches!(kind, SpanKind::Strike)
                });
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                open_images.push((text.len(), dest_url.into_string()));
            }
            Event::End(TagEnd::Image) => {
                let Some((start, url)) = open_images.pop() else {
                    continue;
                };
                // The alt text accumulated as regular inline content; take it
                // back out of the buffer.
                let alt = text[start.min(text.len())..].trim().to_string();
                text.truncate(start);
                spans.retain(|span| span.range.end <= start);
                let label = if alt.is_empty() {
                    "[image]".to_string()
                } else {
                    format!("[{alt}]")
                };

                if matches!(current, CurrentBlock::Table | CurrentBlock::Heading(_)) {
                    // No widgets inside tables or headings; keep the dim
                    // inline placeholder there.
                    let placeholder_start = text.len();
                    text.push_str(&label);
                    for kind in [SpanKind::Dim, SpanKind::Emphasis] {
                        spans.push(RenderedSpan {
                            range: placeholder_start..text.len(),
                            kind,
                        });
                    }
                } else {
                    // Emit the image as its own block, flushing any text
                    // already collected for the surrounding block first.
                    if !text.trim().is_empty() {
                        push_text_block(&mut blocks, current.kind(), &mut text, &mut spans);
                    } else {
                        text.clear();
                        spans.clear();
                    }
                    for (open_start, _) in &mut open_spans {
                        *open_start = 0;
                    }
                    blocks.push(RenderedBlock::new(
                        RenderedBlockKind::Image {
                            source: resolve_image_source(&url, base_dir),
                            alt,
                        },
                        label,
                        Vec::new(),
                    ));
                }
            }
            Event::Text(value) => text.push_str(&value),
            Event::Code(value) | Event::InlineMath(value) | Event::DisplayMath(value) => {
                let start = text.len();
                text.push_str(&value);
                spans.push(RenderedSpan {
                    range: start..text.len(),
                    kind: SpanKind::Code,
                });
            }
            Event::SoftBreak => text.push(if matches!(current, CurrentBlock::Code { .. }) {
                '\n'
            } else {
                ' '
            }),
            Event::HardBreak => text.push(if matches!(current, CurrentBlock::Table) {
                ' '
            } else {
                '\n'
            }),
            Event::Rule => {
                blocks.push(RenderedBlock::new(
                    RenderedBlockKind::Rule,
                    "---".to_string(),
                    Vec::new(),
                ));
            }
            Event::TaskListMarker(done) => {
                // The task box replaces the list bullet, like GitHub does.
                for marker in ["• ", "◦ ", "▪ "] {
                    if text.ends_with(marker) {
                        text.truncate(text.len() - marker.len());
                        break;
                    }
                }
                spans.retain(|span| span.range.end <= text.len());
                let start = text.len();
                text.push(if done { '☑' } else { '☐' });
                spans.push(RenderedSpan {
                    range: start..text.len(),
                    kind: SpanKind::Marker,
                });
                text.push(' ');
            }
            Event::FootnoteReference(label) => {
                text.push('[');
                text.push_str(&label);
                text.push(']');
            }
            Event::InlineHtml(html) => {
                let tag = html.trim();
                if tag.eq_ignore_ascii_case("<br>")
                    || tag.eq_ignore_ascii_case("<br/>")
                    || tag.eq_ignore_ascii_case("<br />")
                {
                    text.push(if matches!(current, CurrentBlock::Table) {
                        ' '
                    } else {
                        '\n'
                    });
                }
            }
            Event::Html(_) => {}
            Event::Start(_) | Event::End(_) => {}
        }
    }

    push_text_block(&mut blocks, current.kind(), &mut text, &mut spans);

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
            // Tables are assembled from their cells when the table ends;
            // stray text can only be a paragraph.
            Self::Table => RenderedBlockKind::Paragraph,
        }
    }
}

fn unordered_marker(depth: usize) -> &'static str {
    match depth {
        0 => "•",
        1 => "◦",
        _ => "▪",
    }
}

fn close_span(
    open_spans: &mut Vec<(usize, SpanKind)>,
    spans: &mut Vec<RenderedSpan>,
    end: usize,
    matches_kind: impl Fn(&SpanKind) -> bool,
) {
    if let Some(position) = open_spans.iter().rposition(|(_, kind)| matches_kind(kind)) {
        let (start, kind) = open_spans.remove(position);
        if start < end {
            spans.push(RenderedSpan {
                range: start..end,
                kind,
            });
        }
    }
}

/// Pads every row to the same number of cells and maps the column
/// alignments, ready for the cell grid.
fn table_data(builder: TableBuilder) -> TableData {
    let columns = builder.rows.iter().map(Vec::len).max().unwrap_or(0);
    let alignments = (0..columns)
        .map(|column| match builder.alignments.get(column) {
            Some(Alignment::Center) => CellAlign::Center,
            Some(Alignment::Right) => CellAlign::Right,
            _ => CellAlign::Left,
        })
        .collect();
    let rows = builder
        .rows
        .into_iter()
        .map(|mut row| {
            row.resize_with(columns, || TableCell {
                text: String::new(),
                spans: Vec::new(),
            });
            row
        })
        .collect();
    TableData { alignments, rows }
}

fn assemble_table(builder: &TableBuilder) -> (String, Vec<RenderedSpan>) {
    let columns = builder.rows.iter().map(Vec::len).max().unwrap_or(0);
    if columns == 0 {
        return (String::new(), Vec::new());
    }

    let mut widths = vec![0usize; columns];
    for row in &builder.rows {
        for (column, cell) in row.iter().enumerate() {
            widths[column] = widths[column].max(cell.text.width());
        }
    }

    let mut text = String::new();
    let mut spans = Vec::new();

    for (row_index, row) in builder.rows.iter().enumerate() {
        // The first row comes from the table head; underline it. The frame
        // glyphs are dimmed so the cell content stays in the foreground.
        if row_index == 1 {
            let separator_start = text.len();
            for (column, width) in widths.iter().enumerate() {
                if column > 0 {
                    text.push_str("─┼─");
                }
                text.push_str(&"─".repeat(*width));
            }
            spans.push(RenderedSpan {
                range: separator_start..text.len(),
                kind: SpanKind::Dim,
            });
            text.push('\n');
        }

        for (column, width) in widths.iter().enumerate() {
            if column > 0 {
                let gutter_start = text.len();
                text.push_str(" │ ");
                spans.push(RenderedSpan {
                    range: gutter_start..text.len(),
                    kind: SpanKind::Dim,
                });
            }

            let cell = row.get(column);
            let cell_text = cell.map(|cell| cell.text.as_str()).unwrap_or("");
            let pad = width.saturating_sub(cell_text.width());
            let (left_pad, right_pad) = match builder.alignments.get(column) {
                Some(Alignment::Right) => (pad, 0),
                Some(Alignment::Center) => (pad / 2, pad - pad / 2),
                _ => (0, pad),
            };

            text.push_str(&" ".repeat(left_pad));
            let cell_start = text.len();
            text.push_str(cell_text);

            if let Some(cell) = cell {
                for span in &cell.spans {
                    spans.push(RenderedSpan {
                        range: (cell_start + span.range.start)..(cell_start + span.range.end),
                        kind: span.kind.clone(),
                    });
                }
            }
            if row_index == 0 && !cell_text.is_empty() {
                spans.push(RenderedSpan {
                    range: cell_start..text.len(),
                    kind: SpanKind::Strong,
                });
            }

            if column + 1 < columns {
                text.push_str(&" ".repeat(right_pad));
            }
        }
        text.push('\n');
    }

    let text = text.trim_end().to_string();
    // Tables render in a monospace font; a whole-table code span keeps
    // styled segments (bold headers, links) in the monospace family so
    // column alignment survives.
    spans.push(RenderedSpan {
        range: 0..text.len(),
        kind: SpanKind::Code,
    });

    (text, spans)
}

fn push_text_block(
    blocks: &mut Vec<RenderedBlock>,
    kind: RenderedBlockKind,
    text: &mut String,
    spans: &mut Vec<RenderedSpan>,
) {
    // List items keep their leading indentation; everything else is trimmed.
    let keep_indent = matches!(kind, RenderedBlockKind::ListItem);
    let trim_start = if keep_indent {
        0
    } else {
        text.len() - text.trim_start().len()
    };
    let value = if keep_indent {
        text.trim_end().to_string()
    } else {
        text.trim().to_string()
    };

    if !value.is_empty() {
        let value_len = value.len();
        let adjusted_spans = std::mem::take(spans)
            .into_iter()
            .filter_map(|span| {
                let start = span.range.start.saturating_sub(trim_start).min(value_len);
                let end = span.range.end.saturating_sub(trim_start).min(value_len);
                (start < end).then_some(RenderedSpan {
                    range: start..end,
                    kind: span.kind,
                })
            })
            .collect();
        blocks.push(RenderedBlock::new(kind, value, adjusted_spans));
    }
    text.clear();
    spans.clear();
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

/// The text that separates two adjacent blocks when they are copied: list
/// items and quote lines stay on consecutive lines, everything else gets a
/// blank line between.
pub fn copy_gap(previous: &RenderedBlockKind, next: &RenderedBlockKind) -> &'static str {
    match (previous, next) {
        (RenderedBlockKind::ListItem, RenderedBlockKind::ListItem)
        | (RenderedBlockKind::Quote, RenderedBlockKind::Quote) => "\n",
        _ => "\n\n",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CellAlign, ImageSource, RenderedBlock, RenderedBlockKind, SpanKind, SpanStyle,
        selectable_text, styled_segments,
    };
    use std::path::{Path, PathBuf};

    fn rendered_blocks(markdown: &str) -> Vec<RenderedBlock> {
        super::rendered_blocks(markdown, Path::new("/docs"))
    }

    #[test]
    fn extracts_readable_text_from_common_markdown() {
        let blocks =
            rendered_blocks("# Title\n\nA **bold** [link](https://example.com).\n\n- One\n- Two");

        assert_eq!(
            selectable_text(&blocks),
            "Title\n\nA bold link.\n\n• One\n• Two"
        );
    }

    #[test]
    fn renders_images_as_blocks() {
        let blocks = rendered_blocks("Intro text.\n\n![Screenshot](assets/shot.png)");

        assert_eq!(blocks.len(), 2);
        assert_eq!(
            blocks[1].kind,
            RenderedBlockKind::Image {
                source: ImageSource::Local(PathBuf::from("/docs/assets/shot.png")),
                alt: "Screenshot".to_string(),
            }
        );
        assert_eq!(blocks[1].text, "[Screenshot]");
    }

    #[test]
    fn splits_paragraph_around_inline_image() {
        let blocks = rendered_blocks("Before ![badge](https://example.com/b.svg) after.");

        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].text, "Before");
        assert!(matches!(
            &blocks[1].kind,
            RenderedBlockKind::Image {
                source: ImageSource::Remote(url),
                ..
            } if url == "https://example.com/b.svg"
        ));
        assert_eq!(blocks[2].text, "after.");
    }

    #[test]
    fn keeps_inline_placeholder_for_images_in_tables() {
        let blocks = rendered_blocks("| A |\n| --- |\n| ![icon](i.png) |");

        assert_eq!(blocks.len(), 1);
        assert!(matches!(blocks[0].kind, RenderedBlockKind::Table(_)));
        assert!(blocks[0].text.contains("[icon]"));
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

        let link = blocks[0]
            .spans
            .iter()
            .find(|span| matches!(span.kind, SpanKind::Link(_)))
            .expect("link span");
        assert_eq!(&blocks[0].text[link.range.clone()], "the guide");
        assert_eq!(
            link.kind,
            SpanKind::Link("https://example.com/guide".to_string())
        );
    }

    #[test]
    fn tracks_inline_style_spans() {
        let blocks = rendered_blocks("Some **bold**, *italic*, and `code` text.");

        let strong = blocks[0]
            .spans
            .iter()
            .find(|span| span.kind == SpanKind::Strong)
            .expect("strong span");
        assert_eq!(&blocks[0].text[strong.range.clone()], "bold");

        let emphasis = blocks[0]
            .spans
            .iter()
            .find(|span| span.kind == SpanKind::Emphasis)
            .expect("emphasis span");
        assert_eq!(&blocks[0].text[emphasis.range.clone()], "italic");

        let code = blocks[0]
            .spans
            .iter()
            .find(|span| span.kind == SpanKind::Code)
            .expect("code span");
        assert_eq!(&blocks[0].text[code.range.clone()], "code");
    }

    #[test]
    fn aligns_table_columns() {
        let blocks = rendered_blocks("| Key | Value |\n| --- | --- |\n| a | bb |\n| ccc | d |");

        assert_eq!(blocks.len(), 1);
        let RenderedBlockKind::Table(table) = &blocks[0].kind else {
            panic!("expected a table block");
        };
        assert_eq!(table.rows.len(), 3);
        assert_eq!(table.rows[0][0].text, "Key");
        assert_eq!(table.rows[2][1].text, "d");
        assert_eq!(table.alignments, vec![CellAlign::Left, CellAlign::Left]);
        let lines: Vec<&str> = blocks[0].text.lines().collect();
        assert_eq!(lines[0], "Key │ Value");
        assert_eq!(lines[1], "────┼──────");
        assert_eq!(lines[2], "a   │ bb");
        assert_eq!(lines[3], "ccc │ d");
        assert!(!blocks[0].text.contains('\t'));

        let header = blocks[0]
            .spans
            .iter()
            .find(|span| span.kind == SpanKind::Strong)
            .expect("bold header span");
        assert_eq!(&blocks[0].text[header.range.clone()], "Key");
    }

    #[test]
    fn pads_ragged_table_rows_and_keeps_cell_spans() {
        let blocks =
            rendered_blocks("| A | B | C |\n| :-: | --: | --- |\n| **x** | [y](https://y.io) |");

        let RenderedBlockKind::Table(table) = &blocks[0].kind else {
            panic!("expected a table block");
        };
        assert_eq!(
            table.alignments,
            vec![CellAlign::Center, CellAlign::Right, CellAlign::Left]
        );
        assert_eq!(table.rows[1].len(), 3);
        assert_eq!(table.rows[1][2].text, "");
        assert_eq!(table.rows[1][0].spans[0].kind, SpanKind::Strong);
        assert_eq!(
            table.rows[1][1].spans[0].kind,
            SpanKind::Link("https://y.io".to_string())
        );
    }

    #[test]
    fn styled_segments_tile_the_text() {
        let blocks = rendered_blocks("plain **bold** [link](https://l.io) end");
        let segments = styled_segments(&blocks[0].text, &blocks[0].spans, &[]);

        let text: String = segments
            .iter()
            .map(|segment| &blocks[0].text[segment.range.clone()])
            .collect();
        assert_eq!(text, blocks[0].text);
        assert_eq!(segments[0].style, SpanStyle::default());
        assert!(segments[1].style.strong);
        assert_eq!(segments[3].link, Some("https://l.io"));
    }

    #[test]
    fn quote_lines_copy_without_blank_lines() {
        let blocks = rendered_blocks("> one\n>\n> two\n\n- a\n- b");

        assert_eq!(selectable_text(&blocks), "one\ntwo\n\n• a\n• b");
    }

    #[test]
    fn nested_lists_keep_parent_item_text() {
        let blocks = rendered_blocks("- Outer\n  - Inner");

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].text, "• Outer");
        assert_eq!(blocks[1].text, "    ◦ Inner");
    }

    #[test]
    fn renders_task_list_markers() {
        let blocks = rendered_blocks("- [x] Done\n- [ ] Pending");

        assert_eq!(blocks[0].text, "☑ Done");
        assert_eq!(blocks[1].text, "☐ Pending");
    }
}
