//! Source files, source identifiers and the source map.
//!
//! The compiler never reads files itself: callers load them into a
//! [`SourceMap`], and every later stage refers to a file by its [`SourceId`].
//! That keeps diagnostics, traces and conformance output addressable without
//! threading paths through the pipeline.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use nudo_span::{BytePos, LineCol, LineIndex, Span};

/// Identifies one source file inside a [`SourceMap`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(u32);

impl SourceId {
    /// Creates an identifier from its index.
    #[must_use]
    pub const fn new(index: u32) -> Self {
        SourceId(index)
    }

    /// The index of the file in its source map.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source#{}", self.0)
    }
}

/// A single source file: its identity, its name and its text.
///
/// The line index is built once, when the file is created, so diagnostic
/// rendering never rescans the text.
#[derive(Debug, Clone)]
pub struct SourceFile {
    id: SourceId,
    name: String,
    text: String,
    line_index: LineIndex,
}

impl SourceFile {
    /// Creates a source file from its identity, its display name and its text.
    ///
    /// The name is a human-facing label: a path, a module name or something
    /// like `<repl>`. NUDO source must be valid UTF-8.
    #[must_use]
    pub fn new(id: SourceId, name: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let line_index = LineIndex::new(&text);
        SourceFile {
            id,
            name: name.into(),
            text,
            line_index,
        }
    }

    /// The identity of this file.
    #[must_use]
    pub const fn id(&self) -> SourceId {
        self.id
    }

    /// The display name of this file.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The text of this file.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The precomputed line index of this file.
    #[must_use]
    pub fn line_index(&self) -> &LineIndex {
        &self.line_index
    }

    /// The length of the file in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Whether the file contains no bytes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// The span covering the whole file.
    #[must_use]
    pub fn full_span(&self) -> Span {
        Span::new(BytePos::ZERO, BytePos::new(self.text.len() as u32))
    }

    /// The line/column pair of `pos`, clamped to this file.
    #[must_use]
    pub fn line_col(&self, pos: BytePos) -> LineCol {
        self.line_index.line_col(&self.text, pos)
    }

    /// The byte range of the 1-based `line`, excluding its terminator.
    #[must_use]
    pub fn line_bounds(&self, line: u32) -> Option<(BytePos, BytePos)> {
        self.line_index.line_bounds(&self.text, line)
    }

    /// The text covered by `span`, if the span is inside this file.
    #[must_use]
    pub fn span_text(&self, span: Span) -> Option<&str> {
        span.text(&self.text)
    }
}

/// Owns every source file the compiler is working with.
///
/// A source map is append-only while a compilation runs: diagnostics keep
/// referring to the same [`SourceId`] values from start to finish.
#[derive(Debug, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    /// Creates an empty source map.
    #[must_use]
    pub fn new() -> Self {
        SourceMap { files: Vec::new() }
    }

    /// Adds a file under `name` and returns its identity.
    pub fn add(&mut self, name: impl Into<String>, text: impl Into<String>) -> SourceId {
        let id = SourceId::new(self.files.len() as u32);
        self.files.push(SourceFile::new(id, name, text));
        id
    }

    /// Reads a UTF-8 source file from disk and adds it under its path.
    ///
    /// # Errors
    ///
    /// Returns the underlying [`io::Error`] when the file cannot be read, and
    /// [`io::ErrorKind::InvalidData`] when it is not valid UTF-8.
    pub fn add_file(&mut self, path: &Path) -> io::Result<SourceId> {
        let bytes = std::fs::read(path)?;
        let text = String::from_utf8(bytes).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} is not valid UTF-8", path.display()),
            )
        })?;
        Ok(self.add(path.display().to_string(), text))
    }

    /// Looks a file up by identity.
    #[must_use]
    pub fn get(&self, id: SourceId) -> Option<&SourceFile> {
        self.files.get(id.index() as usize)
    }

    /// Iterates over the files in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &SourceFile> {
        self.files.iter()
    }

    /// The number of files in the map.
    #[must_use]
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether the map holds no files.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl<'a> IntoIterator for &'a SourceMap {
    type Item = &'a SourceFile;
    type IntoIter = std::slice::Iter<'a, SourceFile>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.iter()
    }
}

/// Reads a UTF-8 source file without adding it to a source map.
///
/// # Errors
///
/// Returns the underlying [`io::Error`] when the file cannot be read, and
/// [`io::ErrorKind::InvalidData`] when it is not valid UTF-8.
pub fn read_to_string(path: &Path) -> io::Result<String> {
    let bytes = std::fs::read(path)?;
    String::from_utf8(bytes).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{} is not valid UTF-8", path.display()),
        )
    })
}

/// The conventional file stem for a NUDO program: `main.nudo`.
#[must_use]
pub fn main_source_path(directory: impl AsRef<Path>) -> PathBuf {
    directory
        .as_ref()
        .join(format!("main.{}", nudo_common::SOURCE_EXTENSION))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_files_with_sequential_ids() {
        let mut map = SourceMap::new();
        let first = map.add("a.nudo", "fn a() {}");
        let second = map.add("b.nudo", "fn b() {}");
        assert_eq!(first.index(), 0);
        assert_eq!(second.index(), 1);
        assert_eq!(map.len(), 2);
        assert!(!map.is_empty());
    }

    #[test]
    fn looks_up_files_by_id() {
        let mut map = SourceMap::new();
        let id = map.add("a.nudo", "fn a() {}");
        let file = map.get(id).expect("file is present");
        assert_eq!(file.name(), "a.nudo");
        assert_eq!(file.text(), "fn a() {}");
        assert_eq!(file.id(), id);
    }

    #[test]
    fn unknown_id_is_none() {
        let map = SourceMap::new();
        assert!(map.get(SourceId::new(7)).is_none());
        assert!(map.is_empty());
    }

    #[test]
    fn reports_line_and_column() {
        let mut map = SourceMap::new();
        let id = map.add("a.nudo", "fn a() {\n    let x = 1\n}\n");
        let file = map.get(id).expect("file is present");
        assert_eq!(file.line_col(BytePos::new(9)), LineCol::new(2, 1));
        assert_eq!(file.len(), file.text().len());
        assert_eq!(file.full_span().end().get() as usize, file.len());
    }

    #[test]
    fn iterates_in_insertion_order() {
        let mut map = SourceMap::new();
        map.add("a.nudo", "");
        map.add("b.nudo", "");
        let names: Vec<&str> = map.iter().map(SourceFile::name).collect();
        assert_eq!(names, vec!["a.nudo", "b.nudo"]);
        let names: Vec<&str> = (&map).into_iter().map(SourceFile::name).collect();
        assert_eq!(names, vec!["a.nudo", "b.nudo"]);
    }

    #[test]
    fn reads_utf8_files_and_rejects_other_bytes() {
        let dir = std::env::temp_dir().join("nudo-source-tests");
        std::fs::create_dir_all(&dir).expect("temp dir");
        let good = dir.join("good.nudo");
        std::fs::write(&good, "let café = 1\n").expect("write");
        let bad = dir.join("bad.nudo");
        std::fs::write(&bad, [0x66, 0x6e, 0xff, 0xfe]).expect("write");

        let mut map = SourceMap::new();
        let id = map.add_file(&good).expect("valid utf-8");
        assert_eq!(map.get(id).expect("file").text(), "let café = 1\n");

        let err = map.add_file(&bad).expect_err("invalid utf-8");
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn main_path_uses_the_nudo_extension() {
        assert!(main_source_path("examples").ends_with("main.nudo"));
    }
}
