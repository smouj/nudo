//! NUDO lexer: turns source text into a token stream.
//!
//! This is the first real stage of the compiler pipeline:
//!
//! ```text
//! SourceFile -> Lexer -> Tokens (+ Diagnostics)
//! ```
//!
//! # Scope of the pre-alpha lexer
//!
//! The lexer implements the lexical structure described in
//! `spec/lexical-structure.md` and is the first stage to be tested by the
//! conformance suite. Everything after it — parser, AST, types, effects — is
//! **PLANNED** and does not exist yet.
//!
//! Only `fn` and `let` are reserved words today. Every other word that the
//! language may reserve later (such as `agent`, `task` or `verify`) lexes as
//! an identifier, so that the pre-alpha toolchain never claims to understand
//! syntax it does not implement.
//!
//! # Recovery
//!
//! Lexing never stops at the first error: an unknown character is reported and
//! skipped, and a malformed literal still produces a token. The lexer always
//! terminates, always emits exactly one [`TokenKind::Eof`] token, and never
//! panics — including on arbitrary input, which the robustness tests exercise.

use nudo_diagnostics::{Diagnostic, DiagnosticCode, Diagnostics, codes};
use nudo_source::{SourceFile, SourceId};
use nudo_span::{BytePos, Span};

/// The kinds of token the pre-alpha lexer produces.
///
/// Variant names are part of the conformance format: they appear verbatim in
/// `nudo check --dump-tokens` output, so renaming a variant changes what other
/// implementations must match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// A name: `add`, `User`, `_tmp`.
    Ident,
    /// An integer literal, possibly with `_` separators: `42`, `1_000`.
    IntLiteral,
    /// A floating-point literal: `3.5`, `1_0.2_5`.
    FloatLiteral,
    /// A text literal, including its quotes: `"hello"`.
    TextLiteral,
    /// The reserved word `fn`.
    KeywordFn,
    /// The reserved word `let`.
    KeywordLet,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `->`
    Arrow,
    /// `=`
    Equals,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `;`
    Semi,
    /// The end of the token stream. Always present, always last.
    Eof,
}

impl TokenKind {
    /// The name of the token kind, as printed by `--dump-tokens`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            TokenKind::Ident => "Ident",
            TokenKind::IntLiteral => "IntLiteral",
            TokenKind::FloatLiteral => "FloatLiteral",
            TokenKind::TextLiteral => "TextLiteral",
            TokenKind::KeywordFn => "KeywordFn",
            TokenKind::KeywordLet => "KeywordLet",
            TokenKind::LParen => "LParen",
            TokenKind::RParen => "RParen",
            TokenKind::LBrace => "LBrace",
            TokenKind::RBrace => "RBrace",
            TokenKind::Colon => "Colon",
            TokenKind::Comma => "Comma",
            TokenKind::Arrow => "Arrow",
            TokenKind::Equals => "Equals",
            TokenKind::Plus => "Plus",
            TokenKind::Minus => "Minus",
            TokenKind::Star => "Star",
            TokenKind::Slash => "Slash",
            TokenKind::Semi => "Semi",
            TokenKind::Eof => "Eof",
        }
    }

    /// Whether this kind is a reserved word.
    #[must_use]
    pub const fn is_keyword(self) -> bool {
        matches!(self, TokenKind::KeywordFn | TokenKind::KeywordLet)
    }

    /// Whether this kind is a literal.
    #[must_use]
    pub const fn is_literal(self) -> bool {
        matches!(
            self,
            TokenKind::IntLiteral | TokenKind::FloatLiteral | TokenKind::TextLiteral
        )
    }
}

/// The words the pre-alpha toolchain reserves.
///
/// Adding a reserved word is a language change and needs a NEP; the list is
/// deliberately short. See `spec/lexical-structure.md`.
pub const KEYWORDS: &[(&str, TokenKind)] =
    &[("fn", TokenKind::KeywordFn), ("let", TokenKind::KeywordLet)];

/// Looks up a reserved word.
#[must_use]
pub fn keyword_kind(text: &str) -> Option<TokenKind> {
    KEYWORDS
        .iter()
        .find(|(keyword, _)| *keyword == text)
        .map(|(_, kind)| *kind)
}

/// A lexed token: a kind plus the byte range it covers in its source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token {
    kind: TokenKind,
    span: Span,
}

impl Token {
    /// Creates a token.
    #[must_use]
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Token { kind, span }
    }

    /// The kind of the token.
    #[must_use]
    pub const fn kind(self) -> TokenKind {
        self.kind
    }

    /// The byte range the token covers.
    #[must_use]
    pub const fn span(self) -> Span {
        self.span
    }

    /// Whether this is the end-of-input token.
    #[must_use]
    pub const fn is_eof(self) -> bool {
        matches!(self.kind, TokenKind::Eof)
    }

    /// The text of the token, if `text` is the source it came from.
    #[must_use]
    pub fn text(self, text: &str) -> Option<&str> {
        self.span.text(text)
    }
}

/// The result of lexing one source file.
#[derive(Debug, Clone)]
pub struct Lexed {
    tokens: Vec<Token>,
    diagnostics: Diagnostics,
}

impl Lexed {
    /// The tokens, ending with [`TokenKind::Eof`].
    #[must_use]
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// The diagnostics produced while lexing.
    #[must_use]
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// Whether lexing produced at least one error.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// Splits the result into its tokens and its diagnostics.
    #[must_use]
    pub fn into_parts(self) -> (Vec<Token>, Diagnostics) {
        (self.tokens, self.diagnostics)
    }
}

/// Lexes `source`.
///
/// This is the whole entry point of the crate: the lexer is stateless from the
/// caller's point of view.
#[must_use]
pub fn tokenize(source: &SourceFile) -> Lexed {
    Lexer::new(source).run()
}

/// The lexer: a cursor over one source file's bytes.
#[derive(Debug)]
pub struct Lexer<'a> {
    text: &'a str,
    source: SourceId,
    offset: usize,
    diagnostics: Diagnostics,
}

impl<'a> Lexer<'a> {
    /// Creates a lexer over `source`.
    #[must_use]
    pub fn new(source: &'a SourceFile) -> Self {
        Lexer {
            text: source.text(),
            source: source.id(),
            offset: 0,
            diagnostics: Diagnostics::new(),
        }
    }

    /// Creates a lexer over raw text that belongs to `source`.
    ///
    /// Useful for tests and for tools that hold text without a
    /// [`SourceFile`].
    #[must_use]
    pub fn with_source(text: &'a str, source: SourceId) -> Self {
        Lexer {
            text,
            source,
            offset: 0,
            diagnostics: Diagnostics::new(),
        }
    }

    /// Lexes the whole input.
    #[must_use]
    pub fn run(mut self) -> Lexed {
        let mut tokens = Vec::new();
        loop {
            self.skip_trivia();
            if self.offset >= self.text.len() {
                let end = BytePos::new(self.text.len() as u32);
                tokens.push(Token::new(TokenKind::Eof, Span::new(end, end)));
                break;
            }
            if let Some(token) = self.next_token() {
                tokens.push(token);
            }
        }
        Lexed {
            tokens,
            diagnostics: self.diagnostics,
        }
    }

    // ----------------------------------------------------------- scanning --

    fn next_token(&mut self) -> Option<Token> {
        let start = self.offset;
        let current = self.current()?;
        let kind = if is_ident_start(current) {
            self.lex_ident()
        } else if current.is_ascii_digit() {
            self.lex_number(start)
        } else if current == '"' {
            self.lex_text(start)
        } else {
            match current {
                '(' => {
                    self.bump();
                    Some(TokenKind::LParen)
                }
                ')' => {
                    self.bump();
                    Some(TokenKind::RParen)
                }
                '{' => {
                    self.bump();
                    Some(TokenKind::LBrace)
                }
                '}' => {
                    self.bump();
                    Some(TokenKind::RBrace)
                }
                ':' => {
                    self.bump();
                    Some(TokenKind::Colon)
                }
                ',' => {
                    self.bump();
                    Some(TokenKind::Comma)
                }
                '=' => {
                    self.bump();
                    Some(TokenKind::Equals)
                }
                '+' => {
                    self.bump();
                    Some(TokenKind::Plus)
                }
                '*' => {
                    self.bump();
                    Some(TokenKind::Star)
                }
                '/' => {
                    self.bump();
                    Some(TokenKind::Slash)
                }
                ';' => {
                    self.bump();
                    Some(TokenKind::Semi)
                }
                '-' => {
                    self.bump();
                    if self.current() == Some('>') {
                        self.bump();
                        Some(TokenKind::Arrow)
                    } else {
                        Some(TokenKind::Minus)
                    }
                }
                _ => {
                    self.bump();
                    let span =
                        Span::new(BytePos::new(start as u32), BytePos::new(self.offset as u32));
                    let message = format!("unknown character `{}`", current.escape_debug());
                    self.report(
                        codes::UNKNOWN_CHARACTER,
                        message,
                        span,
                        Some("the pre-alpha lexer recognises only the minimal token set; see `spec/lexical-structure.md`"),
                    );
                    None
                }
            }
        };
        kind.map(|kind| {
            Token::new(
                kind,
                Span::new(BytePos::new(start as u32), BytePos::new(self.offset as u32)),
            )
        })
    }

    fn lex_ident(&mut self) -> Option<TokenKind> {
        let start = self.offset;
        while self.current().is_some_and(is_ident_continue) {
            self.bump();
        }
        let text = &self.text[start..self.offset];
        Some(keyword_kind(text).unwrap_or(TokenKind::Ident))
    }

    fn lex_number(&mut self, start: usize) -> Option<TokenKind> {
        let mut malformed = false;
        let mut kind = TokenKind::IntLiteral;
        self.eat_digits(&mut malformed);
        if self.current() == Some('.') && self.peek().is_some_and(|c| c.is_ascii_digit()) {
            kind = TokenKind::FloatLiteral;
            self.bump();
            self.eat_digits(&mut malformed);
        }
        if self.current().is_some_and(is_ident_continue) {
            while self.current().is_some_and(is_ident_continue) {
                self.bump();
            }
            malformed = true;
        }
        if malformed {
            let span = Span::new(BytePos::new(start as u32), BytePos::new(self.offset as u32));
            let literal = self.text[start..self.offset].to_string();
            let message = format!("invalid number literal `{literal}`");
            self.report(
                codes::INVALID_NUMBER_LITERAL,
                message,
                span,
                Some("`_` may separate digits but cannot start or end a literal, and a literal cannot run into an identifier"),
            );
        }
        Some(kind)
    }

    /// Consumes digits, allowing `_` only between them.
    fn eat_digits(&mut self, malformed: &mut bool) {
        loop {
            match self.current() {
                Some(c) if c.is_ascii_digit() => {
                    self.bump();
                }
                Some('_') => {
                    self.bump();
                    if !self.current().is_some_and(|c| c.is_ascii_digit()) {
                        *malformed = true;
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn lex_text(&mut self, start: usize) -> Option<TokenKind> {
        self.bump(); // opening quote
        loop {
            let Some(current) = self.current() else {
                self.report_unterminated_text(start, self.offset);
                break;
            };
            if current == '\n' {
                self.report_unterminated_text(start, self.offset);
                break;
            }
            if current == '"' {
                self.bump();
                break;
            }
            if current == '\\' {
                let escape_start = self.offset;
                self.bump();
                match self.current() {
                    Some('n' | 'r' | 't' | '\\' | '"' | '0') => {
                        self.bump();
                    }
                    None | Some('\n') => {
                        self.report_unterminated_text(start, escape_start);
                        break;
                    }
                    Some(other) => {
                        let end = self.offset + other.len_utf8();
                        self.bump();
                        let message =
                            format!("unknown escape sequence `\\{}`", other.escape_debug());
                        self.report(
                            codes::INVALID_ESCAPE_SEQUENCE,
                            message,
                            Span::new(BytePos::new(escape_start as u32), BytePos::new(end as u32)),
                            Some("supported escapes are `\\n`, `\\r`, `\\t`, `\\\\`, `\\\"` and `\\0`"),
                        );
                    }
                }
                continue;
            }
            self.bump();
        }
        Some(TokenKind::TextLiteral)
    }

    fn skip_trivia(&mut self) {
        loop {
            match self.current() {
                // A leading byte-order mark is not part of the program.
                Some('\u{feff}') if self.offset == 0 => {
                    self.bump();
                }
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('/') if self.peek() == Some('/') => {
                    while let Some(c) = self.current() {
                        if c == '\n' {
                            break;
                        }
                        self.bump();
                    }
                }
                Some('/') if self.peek() == Some('*') => self.skip_block_comment(),
                _ => return,
            }
        }
    }

    /// Skips a block comment. Nested block comments are supported, so a
    /// commented-out region containing a comment stays commented out.
    fn skip_block_comment(&mut self) {
        let start = self.offset;
        self.bump();
        self.bump();
        let mut depth = 1u32;
        loop {
            match (self.current(), self.peek()) {
                (None, _) => {
                    let span =
                        Span::new(BytePos::new(start as u32), BytePos::new(self.offset as u32));
                    self.report(
                        codes::UNTERMINATED_BLOCK_COMMENT,
                        "unterminated block comment".to_string(),
                        span,
                        Some("close the comment with `*/`"),
                    );
                    return;
                }
                (Some('*'), Some('/')) => {
                    self.bump();
                    self.bump();
                    depth -= 1;
                    if depth == 0 {
                        return;
                    }
                }
                (Some('/'), Some('*')) => {
                    self.bump();
                    self.bump();
                    depth += 1;
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    // ------------------------------------------------------------ cursor ---

    fn current(&self) -> Option<char> {
        self.text[self.offset..].chars().next()
    }

    fn peek(&self) -> Option<char> {
        let mut chars = self.text[self.offset..].chars();
        chars.next();
        chars.next()
    }

    fn bump(&mut self) -> Option<char> {
        let current = self.current()?;
        self.offset += current.len_utf8();
        Some(current)
    }

    // ------------------------------------------------------- diagnostics ---

    fn report(&mut self, code: DiagnosticCode, message: String, span: Span, help: Option<&str>) {
        let mut diagnostic = Diagnostic::error(code, message).with_location(self.source, span);
        if let Some(help) = help {
            diagnostic = diagnostic.with_help(help);
        }
        self.diagnostics.push(diagnostic);
    }

    fn report_unterminated_text(&mut self, start: usize, end: usize) {
        let diagnostic = Diagnostic::error(codes::UNTERMINATED_STRING, "unterminated text literal")
            .with_location(
                self.source,
                Span::new(BytePos::new(start as u32), BytePos::new(end as u32)),
            )
            .with_note("a text literal must be closed on the same line it starts on")
            .with_help("add a closing `\"`");
        self.diagnostics.push(diagnostic);
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Renders a token stream in the stable, diffable format used by the
/// conformance suite.
///
/// The format is versioned and documented in `tests/conformance/README.md`:
///
/// ```text
/// # nudo-tokens v1
/// 0000 1:1-1:3 KeywordFn "fn"
/// 0001 1:4-1:7 Ident "add"
/// ```
///
/// Each line holds the token index, the 1-based start and end position
/// (exclusive), the [`TokenKind`] name and the lexeme in Rust debug form.
#[must_use]
pub fn dump_tokens(source: &SourceFile, tokens: &[Token]) -> String {
    let mut out = String::from("# nudo-tokens v1\n");
    for (index, token) in tokens.iter().enumerate() {
        let start = source.line_col(token.span().start());
        let end = source.line_col(token.span().end());
        let lexeme = token.text(source.text()).unwrap_or("");
        out.push_str(&format!(
            "{index:04} {start}-{end} {} {lexeme:?}\n",
            token.kind().as_str(),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use nudo_source::SourceMap;

    fn lex(text: &str) -> (Vec<TokenKind>, Diagnostics) {
        let mut sources = SourceMap::new();
        let id = sources.add("test.nudo", text);
        let file = sources.get(id).expect("file");
        let lexed = tokenize(file);
        let kinds = lexed.tokens().iter().map(|token| token.kind()).collect();
        (kinds, lexed.diagnostics().clone())
    }

    fn kinds(text: &str) -> Vec<TokenKind> {
        let (kinds, diagnostics) = lex(text);
        assert!(
            !diagnostics.has_errors(),
            "unexpected diagnostics: {:?}",
            diagnostics
        );
        kinds
    }

    #[test]
    fn empty_input_yields_only_eof() {
        assert_eq!(kinds(""), vec![TokenKind::Eof]);
        assert_eq!(kinds("   \n\t"), vec![TokenKind::Eof]);
    }

    #[test]
    fn lexes_a_function_declaration() {
        assert_eq!(
            kinds("fn add(a: Int, b: Int) -> Int { a + b }"),
            vec![
                TokenKind::KeywordFn,
                TokenKind::Ident,
                TokenKind::LParen,
                TokenKind::Ident,
                TokenKind::Colon,
                TokenKind::Ident,
                TokenKind::Comma,
                TokenKind::Ident,
                TokenKind::Colon,
                TokenKind::Ident,
                TokenKind::RParen,
                TokenKind::Arrow,
                TokenKind::Ident,
                TokenKind::LBrace,
                TokenKind::Ident,
                TokenKind::Plus,
                TokenKind::Ident,
                TokenKind::RBrace,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_let_binding() {
        assert_eq!(
            kinds("let x = 1;"),
            vec![
                TokenKind::KeywordLet,
                TokenKind::Ident,
                TokenKind::Equals,
                TokenKind::IntLiteral,
                TokenKind::Semi,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn distinguishes_minus_from_arrow() {
        assert_eq!(
            kinds("a - b -> c"),
            vec![
                TokenKind::Ident,
                TokenKind::Minus,
                TokenKind::Ident,
                TokenKind::Arrow,
                TokenKind::Ident,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_numbers() {
        assert_eq!(
            kinds("1 1_000 3.5 1_0.2_5"),
            vec![
                TokenKind::IntLiteral,
                TokenKind::IntLiteral,
                TokenKind::FloatLiteral,
                TokenKind::FloatLiteral,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_text_literals_with_escapes() {
        assert_eq!(
            kinds(r#""hello" "a\nb" "\\" "\"""#),
            vec![
                TokenKind::TextLiteral,
                TokenKind::TextLiteral,
                TokenKind::TextLiteral,
                TokenKind::TextLiteral,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_underscore_and_digit_identifiers() {
        assert_eq!(
            kinds("_tmp x1"),
            vec![TokenKind::Ident, TokenKind::Ident, TokenKind::Eof]
        );
    }

    #[test]
    fn non_ascii_letters_are_reported_not_guessed() {
        let (kinds, diagnostics) = lex("let café = 1");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics.as_slice()[0].code(), codes::UNKNOWN_CHARACTER);
        assert_eq!(
            kinds,
            vec![
                TokenKind::KeywordLet,
                TokenKind::Ident,
                TokenKind::Equals,
                TokenKind::IntLiteral,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_comments_and_nested_block_comments() {
        assert_eq!(
            kinds("// line\nlet /* a /* nested */ b */ x = 1"),
            vec![
                TokenKind::KeywordLet,
                TokenKind::Ident,
                TokenKind::Equals,
                TokenKind::IntLiteral,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn slash_is_division_outside_comments() {
        assert_eq!(
            kinds("a / b"),
            vec![
                TokenKind::Ident,
                TokenKind::Slash,
                TokenKind::Ident,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_a_leading_byte_order_mark() {
        assert_eq!(
            kinds("\u{feff}let x = 1"),
            vec![
                TokenKind::KeywordLet,
                TokenKind::Ident,
                TokenKind::Equals,
                TokenKind::IntLiteral,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn reserved_words_are_recognised_case_sensitively() {
        assert_eq!(keyword_kind("fn"), Some(TokenKind::KeywordFn));
        assert_eq!(keyword_kind("let"), Some(TokenKind::KeywordLet));
        assert_eq!(keyword_kind("Fn"), None);
        // Provisional language words are not reserved yet.
        assert_eq!(keyword_kind("agent"), None);
        assert_eq!(keyword_kind("verify"), None);
    }

    #[test]
    fn reports_unterminated_text() {
        let (kinds, diagnostics) = lex("let x = \"open\n");
        assert_eq!(diagnostics.error_count(), 1);
        assert_eq!(diagnostics.as_slice()[0].code(), codes::UNTERMINATED_STRING);
        assert_eq!(
            kinds,
            vec![
                TokenKind::KeywordLet,
                TokenKind::Ident,
                TokenKind::Equals,
                TokenKind::TextLiteral,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn reports_unterminated_text_at_end_of_file() {
        let (_, diagnostics) = lex("\"");
        assert_eq!(diagnostics.error_count(), 1);
        assert_eq!(diagnostics.as_slice()[0].code(), codes::UNTERMINATED_STRING);
    }

    #[test]
    fn reports_unknown_escape_sequence() {
        let (_, diagnostics) = lex(r#""a\qb""#);
        assert_eq!(diagnostics.error_count(), 1);
        let diagnostic = &diagnostics.as_slice()[0];
        assert_eq!(diagnostic.code(), codes::INVALID_ESCAPE_SEQUENCE);
        assert!(diagnostic.message().contains(r"\q"));
    }

    #[test]
    fn reports_unterminated_block_comment() {
        let (_, diagnostics) = lex("let /* open");
        assert_eq!(diagnostics.error_count(), 1);
        assert_eq!(
            diagnostics.as_slice()[0].code(),
            codes::UNTERMINATED_BLOCK_COMMENT
        );
    }

    #[test]
    fn reports_invalid_number_literals() {
        for text in ["1abc", "1_", "1__0"] {
            let (_, diagnostics) = lex(text);
            assert!(
                diagnostics
                    .iter()
                    .any(|it| it.code() == codes::INVALID_NUMBER_LITERAL),
                "{text} should be rejected"
            );
        }
    }

    #[test]
    fn recovers_after_unknown_characters() {
        let (kinds, diagnostics) = lex("let @ = #1;");
        // Two characters start no token (`@` and `#`); everything else recovers.
        assert_eq!(diagnostics.error_count(), 2);
        assert_eq!(
            kinds,
            vec![
                TokenKind::KeywordLet,
                TokenKind::Equals,
                TokenKind::IntLiteral,
                TokenKind::Semi,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn spans_cover_the_token_text() {
        let mut sources = SourceMap::new();
        let id = sources.add("test.nudo", "fn add() {}");
        let file = sources.get(id).expect("file");
        let lexed = tokenize(file);
        assert_eq!(lexed.tokens()[0].text(file.text()), Some("fn"));
        assert_eq!(lexed.tokens()[1].text(file.text()), Some("add"));
        assert!(!lexed.tokens()[0].is_eof());
        assert!(lexed.tokens().last().expect("eof").is_eof());
    }

    #[test]
    fn token_kind_helpers() {
        assert!(TokenKind::KeywordFn.is_keyword());
        assert!(!TokenKind::Ident.is_keyword());
        assert!(TokenKind::IntLiteral.is_literal());
        assert!(!TokenKind::Eof.is_literal());
        assert_eq!(TokenKind::Arrow.as_str(), "Arrow");
    }

    #[test]
    fn lexed_reports_errors_and_parts() {
        let mut sources = SourceMap::new();
        let id = sources.add("test.nudo", "@");
        let file = sources.get(id).expect("file");
        let lexed = tokenize(file);
        assert!(lexed.has_errors());
        let (tokens, diagnostics) = lexed.into_parts();
        assert_eq!(tokens.len(), 1);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn lexer_can_run_on_raw_text() {
        let lexer = Lexer::with_source("let x = 1", SourceId::new(0));
        let lexed = lexer.run();
        assert_eq!(lexed.tokens().len(), 5);
    }

    #[test]
    fn dump_tokens_is_stable() {
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", "fn add(a: Int) -> Int {\n  a\n}\n");
        let file = sources.get(id).expect("file");
        let lexed = tokenize(file);
        let dump = dump_tokens(file, lexed.tokens());
        assert!(dump.starts_with("# nudo-tokens v1\n"));
        assert!(dump.contains("0000 1:1-1:3 KeywordFn \"fn\""));
        assert!(dump.contains("0010 2:3-2:4 Ident \"a\""));
        assert!(dump.ends_with(&format!(
            "{:04} 4:1-4:1 Eof \"\"\n",
            lexed.tokens().len() - 1
        )));
    }
}
