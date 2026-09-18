//! Diagnostic model, stable `NDO` error codes and human-readable renderers.
//!
//! Diagnostics are the compiler's user interface: they are what a person reads
//! when something is wrong, and what a machine reads when an agent needs to
//! repair a program. Both audiences impose constraints:
//!
//! * codes are **stable** and allocated by family (see [`codes`]), so a tool
//!   can match on `NDO1002` without parsing English;
//! * every diagnostic carries a span into a source file whenever one exists,
//!   so it can be rendered with a caret or turned into an editor range.
//!
//! Formatting rules for codes and their meanings are specified in
//! `spec/errors.md`. Adding a code is a specification change, not an
//! implementation detail.

use std::fmt;
use std::fmt::Write as _;

use nudo_source::{SourceFile, SourceId, SourceMap};
use nudo_span::Span;

/// How serious a diagnostic is.
///
/// Only [`Severity::Error`] makes a compilation fail; warnings and notes are
/// informational.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// The program is rejected.
    Error,
    /// The program is accepted, but something is probably unintentional.
    Warning,
    /// Extra context attached to another diagnostic.
    Note,
}

impl Severity {
    /// The severity as lowercase text, as printed by the renderer.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        }
    }

    /// Whether this severity rejects the program.
    #[must_use]
    pub const fn is_error(self) -> bool {
        matches!(self, Severity::Error)
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A stable diagnostic identifier: an `NDO` code plus a symbolic name.
///
/// The code is the machine-facing contract (`NDO1002`); the name is the
/// human-facing one (`UNKNOWN_CHARACTER`). Codes are never reused for a
/// different meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticCode {
    id: &'static str,
    name: &'static str,
}

impl DiagnosticCode {
    /// Creates a code. Both parts are `'static` because the registry is fixed
    /// at compile time.
    #[must_use]
    pub const fn new(id: &'static str, name: &'static str) -> Self {
        DiagnosticCode { id, name }
    }

    /// The machine-facing code, for example `NDO1002`.
    #[must_use]
    pub const fn id(self) -> &'static str {
        self.id
    }

    /// The symbolic name, for example `UNKNOWN_CHARACTER`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// The family of the code, taken from its first digit after the `NDO`
    /// prefix, for example `1` for `NDO1xxx`.
    #[must_use]
    pub const fn family(self) -> char {
        let bytes = self.id.as_bytes();
        if bytes.len() < 7 {
            return '?';
        }
        bytes[3] as char
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id)
    }
}

/// The registry of stable `NDO` diagnostic codes.
///
/// Family layout, fixed by `spec/errors.md`:
///
/// | Family | Area                          |
/// | ------ | ----------------------------- |
/// | `1xxx` | lexical structure and syntax  |
/// | `2xxx` | types                         |
/// | `3xxx` | effects                       |
/// | `4xxx` | agents, tasks and tools       |
/// | `5xxx` | capabilities and policies     |
/// | `6xxx` | runtime                       |
/// | `7xxx` | interoperability              |
/// | `8xxx` | packages and files            |
/// | `9xxx` | internal compiler invariants  |
pub mod codes {
    use super::DiagnosticCode;

    // ---------------------------------------------------------------- 1xxx --
    /// A token appeared in a position the grammar does not allow.
    pub const UNEXPECTED_TOKEN: DiagnosticCode = DiagnosticCode::new("NDO1001", "UNEXPECTED_TOKEN");

    /// The lexer met a character that starts no token.
    pub const UNKNOWN_CHARACTER: DiagnosticCode =
        DiagnosticCode::new("NDO1002", "UNKNOWN_CHARACTER");

    /// A text literal was not closed before the end of its line.
    pub const UNTERMINATED_STRING: DiagnosticCode =
        DiagnosticCode::new("NDO1003", "UNTERMINATED_STRING");

    /// A block comment was not closed before the end of the file.
    pub const UNTERMINATED_BLOCK_COMMENT: DiagnosticCode =
        DiagnosticCode::new("NDO1004", "UNTERMINATED_BLOCK_COMMENT");

    /// A numeric literal is malformed, for example `1_000abc`.
    pub const INVALID_NUMBER_LITERAL: DiagnosticCode =
        DiagnosticCode::new("NDO1005", "INVALID_NUMBER_LITERAL");

    /// A text literal contains an escape sequence NUDO does not define.
    pub const INVALID_ESCAPE_SEQUENCE: DiagnosticCode =
        DiagnosticCode::new("NDO1006", "INVALID_ESCAPE_SEQUENCE");

    // ---------------------------------------------------------------- 2xxx --
    /// A name that resolution could not find.
    pub const UNRESOLVED_NAME: DiagnosticCode = DiagnosticCode::new("NDO2001", "UNRESOLVED_NAME");

    /// Two definitions of the same name in one scope and namespace.
    pub const DUPLICATE_NAME: DiagnosticCode = DiagnosticCode::new("NDO2002", "DUPLICATE_NAME");

    /// A name used as the wrong kind of thing: a type where a value is
    /// expected, or the other way round.
    pub const WRONG_NAMESPACE: DiagnosticCode = DiagnosticCode::new("NDO2003", "WRONG_NAMESPACE");

    // ---------------------------------------------------------------- 8xxx --
    /// The file being compiled does not use the `.nudo` extension.
    pub const UNEXPECTED_FILE_EXTENSION: DiagnosticCode =
        DiagnosticCode::new("NDO8001", "UNEXPECTED_FILE_EXTENSION");

    /// Every code reserved by the specification, whether or not the current
    /// toolchain can emit it yet.
    pub const ALL: &[DiagnosticCode] = &[
        UNEXPECTED_TOKEN,
        UNKNOWN_CHARACTER,
        UNTERMINATED_STRING,
        UNTERMINATED_BLOCK_COMMENT,
        INVALID_NUMBER_LITERAL,
        INVALID_ESCAPE_SEQUENCE,
        UNRESOLVED_NAME,
        DUPLICATE_NAME,
        WRONG_NAMESPACE,
        UNEXPECTED_FILE_EXTENSION,
    ];

    /// The codes the pre-alpha toolchain can actually emit today.
    ///
    /// Everything in [`ALL`] but not here is reserved and documented as
    /// planned; the toolchain never claims to detect a class of error it
    /// cannot detect.
    pub const IMPLEMENTED: &[DiagnosticCode] = &[
        UNEXPECTED_TOKEN,
        UNKNOWN_CHARACTER,
        UNTERMINATED_STRING,
        UNTERMINATED_BLOCK_COMMENT,
        INVALID_NUMBER_LITERAL,
        INVALID_ESCAPE_SEQUENCE,
        UNRESOLVED_NAME,
        DUPLICATE_NAME,
        WRONG_NAMESPACE,
        UNEXPECTED_FILE_EXTENSION,
    ];

    /// Looks a code up by its identifier, for example `NDO1002`.
    #[must_use]
    pub fn lookup(id: &str) -> Option<DiagnosticCode> {
        ALL.iter().copied().find(|code| code.id == id)
    }
}

/// A single diagnostic: a code, a severity, a message and optional context.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    code: DiagnosticCode,
    severity: Severity,
    message: String,
    source: Option<SourceId>,
    span: Option<Span>,
    notes: Vec<String>,
    help: Option<String>,
}

impl Diagnostic {
    /// Creates an error diagnostic.
    pub fn error(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Diagnostic::new(code, Severity::Error, message)
    }

    /// Creates a warning diagnostic.
    pub fn warning(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Diagnostic::new(code, Severity::Warning, message)
    }

    /// Creates a note diagnostic.
    pub fn note(code: DiagnosticCode, message: impl Into<String>) -> Self {
        Diagnostic::new(code, Severity::Note, message)
    }

    fn new(code: DiagnosticCode, severity: Severity, message: impl Into<String>) -> Self {
        Diagnostic {
            code,
            severity,
            message: message.into(),
            source: None,
            span: None,
            notes: Vec::new(),
            help: None,
        }
    }

    /// Attaches the source file the diagnostic belongs to.
    #[must_use]
    pub fn with_source(mut self, source: SourceId) -> Self {
        self.source = Some(source);
        self
    }

    /// Attaches the span the diagnostic points at.
    #[must_use]
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Attaches the source file and span in one step.
    #[must_use]
    pub fn with_location(mut self, source: SourceId, span: Span) -> Self {
        self.source = Some(source);
        self.span = Some(span);
        self
    }

    /// Appends a note line.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Appends a help line. There is at most one help line per diagnostic.
    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// The diagnostic code.
    #[must_use]
    pub const fn code(&self) -> DiagnosticCode {
        self.code
    }

    /// The severity.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// The one-line message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The source file this diagnostic points into, if any.
    #[must_use]
    pub const fn source(&self) -> Option<SourceId> {
        self.source
    }

    /// The span this diagnostic points at, if any.
    #[must_use]
    pub const fn span(&self) -> Option<Span> {
        self.span
    }

    /// The attached notes.
    #[must_use]
    pub fn notes(&self) -> &[String] {
        &self.notes
    }

    /// The attached help line, if any.
    #[must_use]
    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }

    /// Whether this diagnostic rejects the program.
    #[must_use]
    pub const fn is_error(&self) -> bool {
        self.severity.is_error()
    }
}

/// An ordered collection of diagnostics.
#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
}

impl Diagnostics {
    /// Creates an empty collection.
    #[must_use]
    pub fn new() -> Self {
        Diagnostics { items: Vec::new() }
    }

    /// Appends a diagnostic.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.items.push(diagnostic);
    }

    /// Appends every diagnostic of `other`.
    pub fn extend_from(&mut self, other: Diagnostics) {
        self.items.extend(other.items);
    }

    /// The diagnostics in emission order.
    #[must_use]
    pub fn as_slice(&self) -> &[Diagnostic] {
        &self.items
    }

    /// Iterates over the diagnostics.
    pub fn iter(&self) -> std::slice::Iter<'_, Diagnostic> {
        self.items.iter()
    }

    /// The number of diagnostics.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether there are no diagnostics.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The number of error diagnostics.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.items.iter().filter(|item| item.is_error()).count()
    }

    /// The number of warning diagnostics.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| item.severity() == Severity::Warning)
            .count()
    }

    /// Whether any diagnostic rejects the program.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.items.iter().any(Diagnostic::is_error)
    }
}

impl<'a> IntoIterator for &'a Diagnostics {
    type Item = &'a Diagnostic;
    type IntoIter = std::slice::Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

/// Whether the renderer paints its output with ANSI colours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorChoice {
    /// Colour only when the output is a terminal.
    #[default]
    Auto,
    /// Always colour.
    Always,
    /// Never colour.
    Never,
}

impl ColorChoice {
    /// Parses `auto`, `always` or `never`.
    #[must_use]
    pub fn parse(spec: &str) -> Option<ColorChoice> {
        match spec {
            "auto" => Some(ColorChoice::Auto),
            "always" => Some(ColorChoice::Always),
            "never" => Some(ColorChoice::Never),
            _ => None,
        }
    }

    /// Decides whether to colour, given whether the stream is a terminal.
    #[must_use]
    pub const fn resolve(self, is_terminal: bool) -> bool {
        match self {
            ColorChoice::Auto => is_terminal,
            ColorChoice::Always => true,
            ColorChoice::Never => false,
        }
    }
}

/// ANSI escape sequences used by the renderer.
mod style {
    pub(crate) const RESET: &str = "\u{1b}[0m";
    pub(crate) const BOLD: &str = "\u{1b}[1m";
    pub(crate) const RED: &str = "\u{1b}[31m";
    pub(crate) const YELLOW: &str = "\u{1b}[33m";
    pub(crate) const BLUE: &str = "\u{1b}[34m";
    pub(crate) const CYAN: &str = "\u{1b}[36m";
}

/// Renders diagnostics the way a compiler should: location first, then the
/// source line, then a caret under the offending bytes.
///
/// ```text
/// error[NDO1002]: unknown character `@`
///   --> examples/00-hello-world/main.nudo:3:5
///    |
///  3 |     @foo
///    |     ^
/// ```
#[derive(Debug)]
pub struct Renderer<'a> {
    sources: &'a SourceMap,
    color: bool,
}

impl<'a> Renderer<'a> {
    /// Creates a renderer over `sources`, colouring or not according to
    /// `color`.
    #[must_use]
    pub fn new(sources: &'a SourceMap, color: bool) -> Self {
        Renderer { sources, color }
    }

    /// Renders one diagnostic. The result has no trailing newline.
    #[must_use]
    pub fn render(&self, diagnostic: &Diagnostic) -> String {
        let mut out = String::new();
        let (severity_color, severity_text) = match diagnostic.severity() {
            Severity::Error => (style::RED, "error"),
            Severity::Warning => (style::YELLOW, "warning"),
            Severity::Note => (style::CYAN, "note"),
        };
        if self.color {
            let _ = write!(
                out,
                "{}{}{}[{}{}{}{}]: {}",
                style::BOLD,
                severity_color,
                severity_text,
                style::RESET,
                style::BOLD,
                diagnostic.code(),
                style::RESET,
                diagnostic.message(),
            );
        } else {
            let _ = write!(
                out,
                "{severity_text}[{}]: {}",
                diagnostic.code(),
                diagnostic.message()
            );
        }

        let file = diagnostic.source().and_then(|id| self.sources.get(id));
        let span = diagnostic.span();
        if let (Some(file), Some(span)) = (file, span) {
            if !self.render_snippet(&mut out, file, span) {
                self.render_location_line(&mut out, file);
            }
        } else if let Some(file) = file {
            self.render_location_line(&mut out, file);
        }

        for note in diagnostic.notes() {
            let _ = write!(out, "\n   = note: {note}");
        }
        if let Some(help) = diagnostic.help() {
            let _ = write!(out, "\n   = help: {help}");
        }
        out
    }

    /// Renders every diagnostic, separated by a blank line.
    #[must_use]
    pub fn render_all(&self, diagnostics: &Diagnostics) -> String {
        let rendered: Vec<String> = diagnostics.iter().map(|it| self.render(it)).collect();
        rendered.join("\n\n")
    }

    fn render_location_line(&self, out: &mut String, file: &SourceFile) {
        if self.color {
            let _ = write!(
                out,
                "\n  {}-->{}{} {}",
                style::BOLD,
                style::BLUE,
                style::RESET,
                file.name()
            );
        } else {
            let _ = write!(out, "\n  --> {}", file.name());
        }
    }

    /// Writes the location, the source line and the caret. Returns `false`
    /// when the span cannot be placed in the file, leaving `out` untouched.
    fn render_snippet(&self, out: &mut String, file: &SourceFile, span: Span) -> bool {
        let text = file.text();
        let line_number = file.line_index().line_of(span.start());
        let Some((line_start, line_end)) = file.line_bounds(line_number) else {
            return false;
        };
        let start = span.start().get().max(line_start.get());
        let end = span.end().get().min(line_end.get());
        if start > end || start as usize > text.len() {
            return false;
        }
        let prefix = &text[line_start.as_usize()..start as usize];
        let marked = &text[start as usize..end as usize];
        let column = prefix.chars().count() + 1;
        let carets = marked.chars().count().max(1);
        let line_text = &text[line_start.as_usize()..line_end.as_usize()];
        let display_line = line_text.replace('\t', " ");
        let gutter_width = line_number.to_string().len().max(1);
        let pad = " ".repeat(gutter_width);

        self.render_location_line(out, file);
        let location = file.line_col(span.start());
        let _ = write!(out, ":{}:{}", location.line(), location.column());
        let _ = write!(out, "\n{pad} |");
        let _ = write!(out, "\n{line_number:>gutter_width$} | {display_line}");
        let marker = if self.color {
            format!("{}{}{}", style::BOLD, style::RED, "^".repeat(carets))
        } else {
            "^".repeat(carets)
        };
        let _ = write!(out, "\n{pad} | {}{marker}", " ".repeat(column - 1));
        if self.color {
            let _ = write!(out, "{}", style::RESET);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nudo_span::BytePos;

    #[test]
    fn severities_render_as_text() {
        assert_eq!(Severity::Error.as_str(), "error");
        assert_eq!(Severity::Warning.as_str(), "warning");
        assert_eq!(Severity::Note.as_str(), "note");
        assert!(Severity::Error.is_error());
        assert!(!Severity::Warning.is_error());
    }

    #[test]
    fn code_families_follow_the_id() {
        assert_eq!(codes::UNKNOWN_CHARACTER.family(), '1');
        assert_eq!(codes::UNEXPECTED_FILE_EXTENSION.family(), '8');
    }

    #[test]
    fn registry_ids_are_unique_and_well_formed() {
        let mut ids: Vec<&str> = codes::ALL.iter().map(|code| code.id()).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "diagnostic ids must be unique");

        for code in codes::ALL {
            assert_eq!(code.id().len(), 7, "ids are NDO plus four digits");
            assert!(code.id().starts_with("NDO"));
            assert!(
                code.id()[3..].bytes().all(|byte| byte.is_ascii_digit()),
                "{} must end in digits",
                code.id()
            );
            assert!(
                code.name()
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte == b'_'),
                "{} must be SCREAMING_SNAKE_CASE",
                code.name()
            );
        }
    }

    #[test]
    fn implemented_codes_are_registered() {
        for code in codes::IMPLEMENTED {
            assert!(
                codes::ALL.contains(code),
                "{} is implemented but not registered",
                code.id()
            );
        }
        assert!(codes::IMPLEMENTED.len() <= codes::ALL.len());
    }

    #[test]
    fn lookup_finds_and_misses() {
        assert_eq!(codes::lookup("NDO1002"), Some(codes::UNKNOWN_CHARACTER));
        assert_eq!(codes::lookup("NDO9999"), None);
    }

    #[test]
    fn color_choice_parses_and_resolves() {
        assert_eq!(ColorChoice::parse("auto"), Some(ColorChoice::Auto));
        assert_eq!(ColorChoice::parse("always"), Some(ColorChoice::Always));
        assert_eq!(ColorChoice::parse("never"), Some(ColorChoice::Never));
        assert_eq!(ColorChoice::parse("rainbow"), None);
        assert!(!ColorChoice::Auto.resolve(false));
        assert!(ColorChoice::Auto.resolve(true));
        assert!(ColorChoice::Always.resolve(false));
        assert!(!ColorChoice::Never.resolve(true));
    }

    #[test]
    fn builder_collects_context() {
        let diagnostic = Diagnostic::error(codes::UNKNOWN_CHARACTER, "unknown character `@`")
            .with_span(Span::new(BytePos::new(1), BytePos::new(2)))
            .with_note("note one")
            .with_help("help one");
        assert_eq!(diagnostic.severity(), Severity::Error);
        assert_eq!(diagnostic.message(), "unknown character `@`");
        assert_eq!(diagnostic.notes(), ["note one".to_string()]);
        assert_eq!(diagnostic.help(), Some("help one"));
        assert!(diagnostic.is_error());
        assert_eq!(diagnostic.source(), None);
    }

    #[test]
    fn collection_counts_errors_and_warnings() {
        let mut diagnostics = Diagnostics::new();
        assert!(diagnostics.is_empty());
        diagnostics.push(Diagnostic::error(codes::UNKNOWN_CHARACTER, "one"));
        diagnostics.push(Diagnostic::warning(codes::UNEXPECTED_FILE_EXTENSION, "two"));
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics.error_count(), 1);
        assert_eq!(diagnostics.warning_count(), 1);
        assert!(diagnostics.has_errors());
        assert_eq!((&diagnostics).into_iter().count(), 2);
    }

    #[test]
    fn extends_from_another_collection() {
        let mut first = Diagnostics::new();
        first.push(Diagnostic::error(codes::UNKNOWN_CHARACTER, "one"));
        let mut second = Diagnostics::new();
        second.push(Diagnostic::error(codes::UNKNOWN_CHARACTER, "two"));
        first.extend_from(second);
        assert_eq!(first.len(), 2);
    }

    #[test]
    fn renders_header_without_source() {
        let sources = SourceMap::new();
        let renderer = Renderer::new(&sources, false);
        let diagnostic = Diagnostic::error(codes::UNKNOWN_CHARACTER, "unknown character `@`");
        assert_eq!(
            renderer.render(&diagnostic),
            "error[NDO1002]: unknown character `@`"
        );
    }

    #[test]
    fn renders_snippet_with_caret() {
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", "fn main() {\n    @bad\n}\n");
        let renderer = Renderer::new(&sources, false);
        let diagnostic = Diagnostic::error(codes::UNKNOWN_CHARACTER, "unknown character `@`")
            .with_location(id, Span::new(BytePos::new(16), BytePos::new(17)));
        // Gutter convention follows rustc: the `-->` line is indented by the
        // width of the line number, and every `|` column lines up with it.
        let expected = "\
error[NDO1002]: unknown character `@`
  --> main.nudo:2:5
  |
2 |     @bad
  |     ^";
        assert_eq!(renderer.render(&diagnostic), expected);
    }

    #[test]
    fn renders_notes_and_help() {
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", "@\n");
        let renderer = Renderer::new(&sources, false);
        let diagnostic = Diagnostic::error(codes::UNKNOWN_CHARACTER, "unknown character `@`")
            .with_location(id, Span::new(BytePos::new(0), BytePos::new(1)))
            .with_note("only the minimal token set is recognised")
            .with_help("remove the character");
        let rendered = renderer.render(&diagnostic);
        assert!(rendered.ends_with(
            "   = note: only the minimal token set is recognised\n   = help: remove the character"
        ));
    }

    #[test]
    fn rendering_handles_multi_line_spans_and_tabs() {
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", "fn main() {\n\tlet x = 1\n}\n");
        let renderer = Renderer::new(&sources, false);
        let diagnostic =
            Diagnostic::error(codes::UNTERMINATED_STRING, "unterminated string literal")
                .with_location(id, Span::new(BytePos::new(12), BytePos::new(22)));
        let rendered = renderer.render(&diagnostic);
        // The leading tab is rendered as one space, so the caret still lines up
        // with the column the diagnostic reports.
        assert!(rendered.contains("2 |  let x = 1"), "{rendered}");
        assert!(rendered.contains("  | ^^^^^^^^^^"), "{rendered}");
    }

    #[test]
    fn colored_output_carries_escape_sequences() {
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", "@\n");
        let renderer = Renderer::new(&sources, true);
        let diagnostic = Diagnostic::error(codes::UNKNOWN_CHARACTER, "unknown character `@`")
            .with_location(id, Span::new(BytePos::new(0), BytePos::new(1)));
        let rendered = renderer.render(&diagnostic);
        assert!(rendered.contains(style::RESET));
        assert!(rendered.contains("[31m"));
    }

    #[test]
    fn render_all_separates_diagnostics() {
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", "@\n@\n");
        let renderer = Renderer::new(&sources, false);
        let mut diagnostics = Diagnostics::new();
        diagnostics.push(
            Diagnostic::error(codes::UNKNOWN_CHARACTER, "first")
                .with_location(id, Span::new(BytePos::new(0), BytePos::new(1))),
        );
        diagnostics.push(
            Diagnostic::error(codes::UNKNOWN_CHARACTER, "second")
                .with_location(id, Span::new(BytePos::new(2), BytePos::new(3))),
        );
        let rendered = renderer.render_all(&diagnostics);
        assert_eq!(rendered.matches("error[NDO1002]").count(), 2);
        assert!(rendered.contains("\n\nerror[NDO1002]: second"));
    }
}
