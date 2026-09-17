//! Byte positions, spans and line/column mapping.
//!
//! Every position in the NUDO front end is a **byte offset** into a source
//! file's UTF-8 text. Offsets are cheap to move around and never require the
//! source text, while line/column pairs are computed on demand for humans, who
//! think in 1-based characters.

use std::fmt;

/// A byte offset into a source file's UTF-8 text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BytePos(u32);

impl BytePos {
    /// The offset of the first byte of a source file.
    pub const ZERO: BytePos = BytePos(0);

    /// Creates a position from a raw byte offset.
    #[must_use]
    pub const fn new(offset: u32) -> Self {
        BytePos(offset)
    }

    /// The raw byte offset.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    /// The offset as a `usize`, for slicing source text.
    #[must_use]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for BytePos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A half-open byte range `[start, end)` within a single source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    start: BytePos,
    end: BytePos,
}

impl Span {
    /// Creates a span from a start and an end offset.
    ///
    /// If `end` precedes `start` the span is empty at `start`.
    #[must_use]
    pub const fn new(start: BytePos, end: BytePos) -> Self {
        if end.0 < start.0 {
            Span { start, end: start }
        } else {
            Span { start, end }
        }
    }

    /// Creates a span of `len` bytes starting at `start`.
    #[must_use]
    pub const fn at(start: BytePos, len: u32) -> Self {
        Span::new(start, BytePos::new(start.0.saturating_add(len)))
    }

    /// The first byte of the span.
    #[must_use]
    pub const fn start(self) -> BytePos {
        self.start
    }

    /// One past the last byte of the span.
    #[must_use]
    pub const fn end(self) -> BytePos {
        self.end
    }

    /// The length of the span in bytes.
    #[must_use]
    pub const fn len(self) -> u32 {
        self.end.get() - self.start.get()
    }

    /// Whether the span covers no bytes.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start.get() == self.end.get()
    }

    /// The span that covers both `self` and `other`.
    #[must_use]
    pub const fn join(self, other: Span) -> Span {
        let start = if self.start.0 < other.start.0 {
            self.start
        } else {
            other.start
        };
        let end = if self.end.0 > other.end.0 {
            self.end
        } else {
            other.end
        };
        Span { start, end }
    }

    /// Whether `pos` lies inside the span.
    #[must_use]
    pub const fn contains(self, pos: BytePos) -> bool {
        pos.0 >= self.start.0 && pos.0 < self.end.0
    }

    /// The source text the span covers, if it is inside `text` and lands on
    /// character boundaries.
    #[must_use]
    pub fn text(self, text: &str) -> Option<&str> {
        let start = self.start.as_usize();
        let end = self.end.as_usize();
        if end > text.len() || !text.is_char_boundary(start) || !text.is_char_boundary(end) {
            return None;
        }
        Some(&text[start..end])
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A 1-based line and column pair, as shown to humans.
///
/// Columns count characters, not bytes, so that the caret of a diagnostic
/// lines up with what an editor displays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineCol {
    line: u32,
    column: u32,
}

impl LineCol {
    /// Creates a line/column pair. Both values are 1-based.
    #[must_use]
    pub const fn new(line: u32, column: u32) -> Self {
        LineCol { line, column }
    }

    /// The 1-based line.
    #[must_use]
    pub const fn line(self) -> u32 {
        self.line
    }

    /// The 1-based character column.
    #[must_use]
    pub const fn column(self) -> u32 {
        self.column
    }
}

impl fmt::Display for LineCol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// The byte offset at which every line of a text starts.
///
/// The index is built once per file and then answers line/column queries in
/// logarithmic time.
#[derive(Debug, Clone)]
pub struct LineIndex {
    line_starts: Vec<BytePos>,
}

impl LineIndex {
    /// Builds the line index for `text`.
    ///
    /// A trailing newline does not start a new line; an empty text has exactly
    /// one line. `\r\n` and `\n` are both accepted as line terminators.
    #[must_use]
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![BytePos::ZERO];
        for (offset, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(BytePos::new((offset + 1) as u32));
            }
        }
        LineIndex { line_starts }
    }

    /// The number of lines in the indexed text. Always at least 1.
    #[must_use]
    pub fn line_count(&self) -> u32 {
        self.line_starts.len() as u32
    }

    /// The offset at which the 1-based `line` starts.
    #[must_use]
    pub fn line_start(&self, line: u32) -> Option<BytePos> {
        if line == 0 {
            return None;
        }
        self.line_starts.get((line - 1) as usize).copied()
    }

    /// The 1-based line containing `pos`.
    #[must_use]
    pub fn line_of(&self, pos: BytePos) -> u32 {
        self.line_starts
            .partition_point(|start| start.get() <= pos.get()) as u32
    }

    /// The line/column pair of `pos` in `text`.
    ///
    /// `pos` is clamped to the end of `text` and moved back to the nearest
    /// character boundary, so this never panics on a degenerate position.
    #[must_use]
    pub fn line_col(&self, text: &str, pos: BytePos) -> LineCol {
        let offset = floor_char_boundary(text, pos.as_usize());
        let line = self.line_of(BytePos::new(offset as u32));
        let start = self
            .line_start(line)
            .map_or(0, |start| start.as_usize().min(offset));
        let column = text[start..offset].chars().count() as u32 + 1;
        LineCol::new(line, column)
    }

    /// The byte range of the 1-based `line`, excluding its terminator.
    #[must_use]
    pub fn line_bounds(&self, text: &str, line: u32) -> Option<(BytePos, BytePos)> {
        let start = self.line_start(line)?;
        let next = self
            .line_starts
            .get(line as usize)
            .copied()
            .unwrap_or_else(|| BytePos::new(text.len() as u32));
        let mut end = next.as_usize().min(text.len());
        let bytes = text.as_bytes();
        if end > start.as_usize() && bytes[end - 1] == b'\n' {
            end -= 1;
        }
        if end > start.as_usize() && bytes[end - 1] == b'\r' {
            end -= 1;
        }
        Some((start, BytePos::new(end as u32)))
    }
}

/// Rounds `offset` down to the nearest UTF-8 character boundary of `text`.
///
/// Offsets past the end of `text` clamp to `text.len()`.
#[must_use]
pub fn floor_char_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_len_and_emptiness() {
        let span = Span::new(BytePos::new(2), BytePos::new(7));
        assert_eq!(span.len(), 5);
        assert!(!span.is_empty());
        assert!(Span::new(BytePos::new(3), BytePos::new(3)).is_empty());
    }

    #[test]
    fn inverted_span_is_empty() {
        let span = Span::new(BytePos::new(9), BytePos::new(4));
        assert!(span.is_empty());
        assert_eq!(span.start(), BytePos::new(9));
    }

    #[test]
    fn join_covers_both() {
        let a = Span::new(BytePos::new(1), BytePos::new(4));
        let b = Span::new(BytePos::new(3), BytePos::new(9));
        assert_eq!(a.join(b), Span::new(BytePos::new(1), BytePos::new(9)));
    }

    #[test]
    fn contains_uses_a_half_open_range() {
        let span = Span::new(BytePos::new(1), BytePos::new(4));
        assert!(span.contains(BytePos::new(1)));
        assert!(span.contains(BytePos::new(3)));
        assert!(!span.contains(BytePos::new(4)));
    }

    #[test]
    fn span_text_rejects_bad_ranges() {
        let text = "fn add() {}";
        let ok = Span::new(BytePos::new(0), BytePos::new(2));
        assert_eq!(ok.text(text), Some("fn"));
        let past_end = Span::new(BytePos::new(0), BytePos::new(99));
        assert_eq!(past_end.text(text), None);
    }

    #[test]
    fn span_text_rejects_partial_characters() {
        let text = "«nudo»";
        let split = Span::new(BytePos::new(1), BytePos::new(3));
        assert_eq!(split.text(text), None);
    }

    #[test]
    fn line_index_counts_lines() {
        assert_eq!(LineIndex::new("").line_count(), 1);
        assert_eq!(LineIndex::new("a").line_count(), 1);
        assert_eq!(LineIndex::new("a\n").line_count(), 2);
        assert_eq!(LineIndex::new("a\nb\nc").line_count(), 3);
        assert_eq!(LineIndex::new("a\r\nb").line_count(), 2);
    }

    #[test]
    fn line_of_maps_offsets() {
        let text = "abc\ndef\nghi";
        let index = LineIndex::new(text);
        assert_eq!(index.line_of(BytePos::new(0)), 1);
        assert_eq!(index.line_of(BytePos::new(3)), 1);
        assert_eq!(index.line_of(BytePos::new(4)), 2);
        assert_eq!(index.line_of(BytePos::new(9)), 3);
    }

    #[test]
    fn line_col_is_one_based() {
        let text = "abc\ndef";
        let index = LineIndex::new(text);
        assert_eq!(index.line_col(text, BytePos::new(0)), LineCol::new(1, 1));
        assert_eq!(index.line_col(text, BytePos::new(4)), LineCol::new(2, 1));
        assert_eq!(index.line_col(text, BytePos::new(6)), LineCol::new(2, 3));
    }

    #[test]
    fn line_col_counts_characters_not_bytes() {
        let text = "áéí";
        let index = LineIndex::new(text);
        assert_eq!(index.line_col(text, BytePos::new(2)), LineCol::new(1, 2));
        assert_eq!(index.line_col(text, BytePos::new(4)), LineCol::new(1, 3));
    }

    #[test]
    fn line_col_clamps_out_of_range_positions() {
        let text = "abc";
        let index = LineIndex::new(text);
        assert_eq!(index.line_col(text, BytePos::new(99)), LineCol::new(1, 4));
    }

    #[test]
    fn line_bounds_excludes_terminators() {
        let text = "one\r\ntwo\nthree";
        let index = LineIndex::new(text);
        let (start, end) = index.line_bounds(text, 1).unwrap();
        assert_eq!(Span::new(start, end).text(text), Some("one"));
        let (start, end) = index.line_bounds(text, 2).unwrap();
        assert_eq!(Span::new(start, end).text(text), Some("two"));
        let (start, end) = index.line_bounds(text, 3).unwrap();
        assert_eq!(Span::new(start, end).text(text), Some("three"));
        assert!(index.line_bounds(text, 4).is_none());
    }

    #[test]
    fn floor_char_boundary_walks_back() {
        let text = "á";
        assert_eq!(floor_char_boundary(text, 1), 0);
        assert_eq!(floor_char_boundary(text, 2), 2);
        assert_eq!(floor_char_boundary(text, 50), 2);
    }
}
