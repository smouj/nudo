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
//! The reserved words are the ones NEP-0005 decided: a small core reserved
//! everywhere (`fn`, `let`, `struct`, `enum`, `if`, `else`, `match`, `const`,
//! `true`, `false`). Every other word the grammar mentions (`agent`, `task`,
//! `tool`, `model`, `role`, `tools`, `allow`, `budget`, `with`, `verify`,
//! `ask`, `delegate`) is recognised in context by the parser and stays
//! usable as an identifier, so the pre-alpha toolchain never claims to
//! understand syntax it does not implement.
//!
//! # Losslessness
//!
//! [`tokenize`] skips trivia, because nothing after the lexer needs it.
//! The parser does: the tree it builds must be able to reprint the file byte
//! for byte, so [`tokenize_with_trivia`] returns the same tokens with
//! whitespace and comments kept as [`TokenKind::Whitespace`] and
//! [`TokenKind::Comment`].
//!
//! A character that starts no token is reported as `NDO1002` and still produces
//! one [`TokenKind::Unknown`] token covering it. Reporting it is not enough:
//! a byte that belongs to no token is a byte the tree cannot account for, and
//! "every byte of the file is reachable from the tree" would stop being true.
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
    /// A character that starts no token. Always accompanied by an `NDO1002`
    /// diagnostic, and always produces a token anyway, so that no byte of the
    /// source is unaccounted for.
    Unknown,
    /// The reserved word `fn`.
    KeywordFn,
    /// The reserved word `let`.
    KeywordLet,
    /// The reserved word `struct`.
    KeywordStruct,
    /// The reserved word `enum`.
    KeywordEnum,
    /// The reserved word `if`.
    KeywordIf,
    /// The reserved word `else`.
    KeywordElse,
    /// The reserved word `match`.
    KeywordMatch,
    /// The reserved word `const`.
    KeywordConst,
    /// The reserved word `true`.
    KeywordTrue,
    /// The reserved word `false`.
    KeywordFalse,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `:`
    Colon,
    /// `::`
    ColonColon,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `?`
    Question,
    /// `->`
    Arrow,
    /// `=>`
    FatArrow,
    /// `=`
    Equals,
    /// `==`
    EqEq,
    /// `!=`
    BangEq,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `<=`
    LtEq,
    /// `>=`
    GtEq,
    /// `&&`
    AmpAmp,
    /// `||`
    PipePipe,
    /// `!`
    Bang,
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
    /// Whitespace, produced only by [`tokenize_with_trivia`].
    Whitespace,
    /// A line or block comment, produced only by [`tokenize_with_trivia`].
    Comment,
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
            TokenKind::Unknown => "Unknown",
            TokenKind::KeywordFn => "KeywordFn",
            TokenKind::KeywordLet => "KeywordLet",
            TokenKind::KeywordStruct => "KeywordStruct",
            TokenKind::KeywordEnum => "KeywordEnum",
            TokenKind::KeywordIf => "KeywordIf",
            TokenKind::KeywordElse => "KeywordElse",
            TokenKind::KeywordMatch => "KeywordMatch",
            TokenKind::KeywordConst => "KeywordConst",
            TokenKind::KeywordTrue => "KeywordTrue",
            TokenKind::KeywordFalse => "KeywordFalse",
            TokenKind::LParen => "LParen",
            TokenKind::RParen => "RParen",
            TokenKind::LBrace => "LBrace",
            TokenKind::RBrace => "RBrace",
            TokenKind::LBracket => "LBracket",
            TokenKind::RBracket => "RBracket",
            TokenKind::Colon => "Colon",
            TokenKind::ColonColon => "ColonColon",
            TokenKind::Comma => "Comma",
            TokenKind::Dot => "Dot",
            TokenKind::Question => "Question",
            TokenKind::Arrow => "Arrow",
            TokenKind::FatArrow => "FatArrow",
            TokenKind::Equals => "Equals",
            TokenKind::EqEq => "EqEq",
            TokenKind::BangEq => "BangEq",
            TokenKind::Lt => "Lt",
            TokenKind::Gt => "Gt",
            TokenKind::LtEq => "LtEq",
            TokenKind::GtEq => "GtEq",
            TokenKind::AmpAmp => "AmpAmp",
            TokenKind::PipePipe => "PipePipe",
            TokenKind::Bang => "Bang",
            TokenKind::Plus => "Plus",
            TokenKind::Minus => "Minus",
            TokenKind::Star => "Star",
            TokenKind::Slash => "Slash",
            TokenKind::Semi => "Semi",
            TokenKind::Whitespace => "Whitespace",
            TokenKind::Comment => "Comment",
            TokenKind::Eof => "Eof",
        }
    }

    /// Whether this kind is a reserved word.
    #[must_use]
    pub const fn is_keyword(self) -> bool {
        matches!(
            self,
            TokenKind::KeywordFn
                | TokenKind::KeywordLet
                | TokenKind::KeywordStruct
                | TokenKind::KeywordEnum
                | TokenKind::KeywordIf
                | TokenKind::KeywordElse
                | TokenKind::KeywordMatch
                | TokenKind::KeywordConst
                | TokenKind::KeywordTrue
                | TokenKind::KeywordFalse
        )
    }

    /// Whether this kind is trivia: whitespace or a comment.
    ///
    /// Trivia is produced only when the lexer is asked to preserve it, and it
    /// carries no meaning.
    #[must_use]
    pub const fn is_trivia(self) -> bool {
        matches!(self, TokenKind::Whitespace | TokenKind::Comment)
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
/// This is the reserved core [NEP-0005] decided: words that introduce a
/// construct no program can spell another way. Adding a word here is a
/// language change and needs a NEP; the words that stay contextual
/// (`agent`, `task`, `tool`, `model`, `role`, `tools`, `allow`, `budget`,
/// `with`, `verify`, `ask`, `delegate`) are deliberately *not* in this list,
/// because a program may still use them as names. See
/// `spec/lexical-structure.md`.
///
/// [NEP-0005]: https://github.com/smouj/nudo/blob/main/neps/0005-keyword-policy.md
pub const KEYWORDS: &[(&str, TokenKind)] = &[
    ("fn", TokenKind::KeywordFn),
    ("let", TokenKind::KeywordLet),
    ("struct", TokenKind::KeywordStruct),
    ("enum", TokenKind::KeywordEnum),
    ("if", TokenKind::KeywordIf),
    ("else", TokenKind::KeywordElse),
    ("match", TokenKind::KeywordMatch),
    ("const", TokenKind::KeywordConst),
    ("true", TokenKind::KeywordTrue),
    ("false", TokenKind::KeywordFalse),
];

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
/// This is the whole entry point of the crate for consumers that do not need
/// trivia: the lexer is stateless from the caller's point of view.
#[must_use]
pub fn tokenize(source: &SourceFile) -> Lexed {
    Lexer::new(source).run()
}

/// Lexes `source`, keeping whitespace and comments.
///
/// Every byte of the file belongs to exactly one token of the result, which is
/// what lets `nudo-syntax` build a tree that can reprint the file. Trivia is
/// otherwise identical to what [`tokenize`] returns, token for token.
#[must_use]
pub fn tokenize_with_trivia(source: &SourceFile) -> Lexed {
    Lexer::with_source(source.text(), source.id())
        .preserving_trivia(true)
        .run()
}

/// The lexer: a cursor over one source file's bytes.
#[derive(Debug)]
pub struct Lexer<'a> {
    text: &'a str,
    source: SourceId,
    offset: usize,
    trivia: bool,
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
            trivia: false,
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
            trivia: false,
            diagnostics: Diagnostics::new(),
        }
    }

    /// Chooses whether whitespace and comments are skipped (the default) or
    /// returned as tokens.
    #[must_use]
    pub const fn preserving_trivia(mut self, trivia: bool) -> Self {
        self.trivia = trivia;
        self
    }

    /// Lexes the whole input.
    #[must_use]
    pub fn run(mut self) -> Lexed {
        let mut tokens = Vec::new();
        loop {
            if self.trivia {
                if let Some(token) = self.next_trivia() {
                    tokens.push(token);
                    continue;
                }
            } else {
                self.skip_trivia();
            }
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
                    if self.current() == Some(':') {
                        self.bump();
                        Some(TokenKind::ColonColon)
                    } else {
                        Some(TokenKind::Colon)
                    }
                }
                ',' => {
                    self.bump();
                    Some(TokenKind::Comma)
                }
                '.' => {
                    self.bump();
                    Some(TokenKind::Dot)
                }
                '?' => {
                    self.bump();
                    Some(TokenKind::Question)
                }
                '[' => {
                    self.bump();
                    Some(TokenKind::LBracket)
                }
                ']' => {
                    self.bump();
                    Some(TokenKind::RBracket)
                }
                '=' => {
                    self.bump();
                    match self.current() {
                        Some('=') => {
                            self.bump();
                            Some(TokenKind::EqEq)
                        }
                        Some('>') => {
                            self.bump();
                            Some(TokenKind::FatArrow)
                        }
                        _ => Some(TokenKind::Equals),
                    }
                }
                '!' => {
                    self.bump();
                    if self.current() == Some('=') {
                        self.bump();
                        Some(TokenKind::BangEq)
                    } else {
                        Some(TokenKind::Bang)
                    }
                }
                '<' => {
                    self.bump();
                    if self.current() == Some('=') {
                        self.bump();
                        Some(TokenKind::LtEq)
                    } else {
                        Some(TokenKind::Lt)
                    }
                }
                '>' => {
                    self.bump();
                    if self.current() == Some('=') {
                        self.bump();
                        Some(TokenKind::GtEq)
                    } else {
                        Some(TokenKind::Gt)
                    }
                }
                '&' if self.peek() == Some('&') => {
                    self.bump();
                    self.bump();
                    Some(TokenKind::AmpAmp)
                }
                '|' if self.peek() == Some('|') => {
                    self.bump();
                    self.bump();
                    Some(TokenKind::PipePipe)
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
                        Some("the pre-alpha lexer recognises only the token set of `spec/lexical-structure.md`"),
                    );
                    // The character is reported and still produces a token: a
                    // byte that belongs to no token is a byte the syntax tree
                    // could not account for.
                    Some(TokenKind::Unknown)
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
                Some('/') if self.peek() == Some('*') => self.scan_block_comment(),
                _ => return,
            }
        }
    }

    /// Returns the trivia token at the cursor, if there is one.
    ///
    /// This is the preserving counterpart of [`Lexer::skip_trivia`]: the same
    /// bytes, returned as [`TokenKind::Whitespace`] or [`TokenKind::Comment`]
    /// instead of being dropped.
    fn next_trivia(&mut self) -> Option<Token> {
        let start = self.offset;
        let current = self.current()?;
        let kind = match current {
            '\u{feff}' if self.offset == 0 => {
                self.bump();
                TokenKind::Whitespace
            }
            c if c.is_whitespace() => {
                while self.current().is_some_and(char::is_whitespace) {
                    self.bump();
                }
                TokenKind::Whitespace
            }
            '/' if self.peek() == Some('/') => {
                while let Some(c) = self.current() {
                    if c == '\n' {
                        break;
                    }
                    self.bump();
                }
                TokenKind::Comment
            }
            '/' if self.peek() == Some('*') => {
                self.scan_block_comment();
                TokenKind::Comment
            }
            _ => return None,
        };
        Some(Token::new(
            kind,
            Span::new(BytePos::new(start as u32), BytePos::new(self.offset as u32)),
        ))
    }

    /// Scans a block comment from the cursor, reporting it if it never closes.
    ///
    /// Nested block comments are supported, so a commented-out region
    /// containing a comment stays commented out.
    fn scan_block_comment(&mut self) {
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
                // `café` is `caf` followed by a character that starts no token.
                TokenKind::Ident,
                TokenKind::Unknown,
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
        // NEP-0005: the reserved core.
        for (word, kind) in KEYWORDS {
            assert_eq!(keyword_kind(word), Some(*kind), "{word} must be reserved");
        }
        // Contextual words stay identifiers so that a program may name a
        // binding `agent` or `verify`. The parser recognises them in position.
        for word in [
            "agent", "task", "tool", "model", "role", "tools", "allow", "budget", "with", "verify",
            "ask", "delegate",
        ] {
            assert_eq!(keyword_kind(word), None, "{word} must stay contextual");
        }
    }

    #[test]
    fn lexes_the_extended_punctuation() {
        assert_eq!(
            kinds("[ ] :: . ? => != < > <= >= && || !"),
            vec![
                TokenKind::LBracket,
                TokenKind::RBracket,
                TokenKind::ColonColon,
                TokenKind::Dot,
                TokenKind::Question,
                TokenKind::FatArrow,
                TokenKind::BangEq,
                TokenKind::Lt,
                TokenKind::Gt,
                TokenKind::LtEq,
                TokenKind::GtEq,
                TokenKind::AmpAmp,
                TokenKind::PipePipe,
                TokenKind::Bang,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn prefers_the_longest_operator() {
        assert_eq!(
            kinds("a == b = c; x <= y < z; p >= q > r; m != n ! o;"),
            vec![
                TokenKind::Ident,
                TokenKind::EqEq,
                TokenKind::Ident,
                TokenKind::Equals,
                TokenKind::Ident,
                TokenKind::Semi,
                TokenKind::Ident,
                TokenKind::LtEq,
                TokenKind::Ident,
                TokenKind::Lt,
                TokenKind::Ident,
                TokenKind::Semi,
                TokenKind::Ident,
                TokenKind::GtEq,
                TokenKind::Ident,
                TokenKind::Gt,
                TokenKind::Ident,
                TokenKind::Semi,
                TokenKind::Ident,
                TokenKind::BangEq,
                TokenKind::Ident,
                TokenKind::Bang,
                TokenKind::Ident,
                TokenKind::Semi,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn a_single_ampersand_or_pipe_is_not_a_token() {
        // `&&` and `||` are tokens; `&` and `|` are not, and are reported
        // rather than guessed at. There is no bitwise operator in NUDO.
        let (kinds, diagnostics) = lex("a & b | c");
        assert_eq!(diagnostics.error_count(), 2);
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident,
                TokenKind::Unknown,
                TokenKind::Ident,
                TokenKind::Unknown,
                TokenKind::Ident,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn closing_angle_brackets_are_two_tokens() {
        // There is no shift operator, so `Result<Result<Int, Text>, Text>`
        // closes with two `Gt` tokens rather than one `>>`.
        assert_eq!(
            kinds("Result<Result<Int, Text>, Text>>"),
            vec![
                TokenKind::Ident,
                TokenKind::Lt,
                TokenKind::Ident,
                TokenKind::Lt,
                TokenKind::Ident,
                TokenKind::Comma,
                TokenKind::Ident,
                TokenKind::Gt,
                TokenKind::Comma,
                TokenKind::Ident,
                TokenKind::Gt,
                TokenKind::Gt,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn boolean_literals_are_reserved_words() {
        assert_eq!(
            kinds("true false"),
            vec![
                TokenKind::KeywordTrue,
                TokenKind::KeywordFalse,
                TokenKind::Eof,
            ]
        );
        assert!(TokenKind::KeywordTrue.is_keyword());
        assert!(!TokenKind::KeywordTrue.is_literal());
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
        // Two characters start no token (`@` and `#`); everything else
        // recovers, and both characters keep a token of their own.
        assert_eq!(diagnostics.error_count(), 2);
        assert_eq!(
            kinds,
            vec![
                TokenKind::KeywordLet,
                TokenKind::Unknown,
                TokenKind::Equals,
                TokenKind::Unknown,
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
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind(), TokenKind::Unknown);
        assert_eq!(tokens[0].text("@"), Some("@"));
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn preserving_trivia_accounts_for_every_byte() {
        let mut sources = SourceMap::new();
        let source = "// c\nfn main() {\n    let x = 1;\n}\n";
        let id = sources.add("test.nudo", source);
        let file = sources.get(id).expect("file");
        let with_trivia = tokenize_with_trivia(file);
        let joined: String = with_trivia
            .tokens()
            .iter()
            .map(|token| token.text(source).unwrap_or(""))
            .collect();
        assert_eq!(joined, source);
        assert!(
            with_trivia
                .tokens()
                .iter()
                .any(|token| token.kind() == TokenKind::Comment)
        );
        assert!(
            with_trivia
                .tokens()
                .iter()
                .any(|token| token.kind() == TokenKind::Whitespace)
        );
        // Skipping trivia and preserving it agree on every non-trivia token.
        let skipped = tokenize(file);
        let kept: Vec<TokenKind> = with_trivia
            .tokens()
            .iter()
            .map(|token| token.kind())
            .filter(|kind| !kind.is_trivia())
            .collect();
        let expected: Vec<TokenKind> = skipped.tokens().iter().map(|token| token.kind()).collect();
        assert_eq!(kept, expected);
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
