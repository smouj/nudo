//! Recursive-descent parser that builds the lossless syntax tree.
//!
//! # Shape
//!
//! ```text
//! .nudo source
//!      ↓  nudo-lexer            tokens, trivia included
//!      ↓  nudo-parser           SyntaxTree + Diagnostics
//! ```
//!
//! One function per grammar production, one function per precedence level, and
//! each level calls the next one down — exactly the chain
//! [`grammar/nudo.ebnf`][ebnf] declares and `scripts/check-grammar.py` enforces.
//! The strategy and its rejected alternatives are argued in
//! [`docs/internals/parser-design.md`][design].
//!
//! [ebnf]: https://github.com/smouj/nudo/blob/main/grammar/nudo.ebnf
//! [design]: https://github.com/smouj/nudo/blob/main/docs/internals/parser-design.md
//!
//! # Lookahead
//!
//! The parser needs one token of lookahead, in the places the design document
//! lists: `path-type` versus `generic-type` (`Foo` and `Foo<…>` share a prefix),
//! item dispatch, and statement dispatch. It never backtracks, so a program it
//! rejects is reported at the token that could not be placed rather than at
//! wherever a backtracking parser happened to give up.
//!
//! # Contextual words
//!
//! [`NEP-0005`](https://github.com/smouj/nudo/blob/main/neps/0005-keyword-policy.md)
//! reserves a small core (`fn`, `let`, `struct`, `enum`, `if`, `else`, `match`,
//! `const`, `true`, `false`) and keeps everything else contextual. A contextual
//! word starts its construct only where nothing else could appear:
//!
//! * `agent`, `task`, `tool`, `model` — at item position;
//! * `role`, `tools`, `allow`, `budget`, `agent`, `verify` — as a clause head,
//!   where the `:` that follows is what confirms it;
//! * `ask` and `delegate` — in an expression, and only when a path follows,
//!   because two names in a row are not an expression;
//! * `with` — where the grammar expects an effect, verify or budget clause.
//!
//! `verify` is the one case the rule cannot settle on its own: it is followed
//! by an *expression*, so `verify (draft) with V` would need to look past the
//! `(` to tell a verification from a call. The parser therefore reads `verify`
//! as a verification when the next token starts an operand and is not `(`,
//! `.` or `[`. See the known limitation in
//! [`docs/internals/parser-design.md`](https://github.com/smouj/nudo/blob/main/docs/internals/parser-design.md#known-limitations).
//!
//! # Recovery
//!
//! Recovery is a contract, not a best effort:
//!
//! * every recovery step consumes at least one token, so parsing never hangs;
//! * a token the parser cannot place goes into an `Error` node, which stays in
//!   the tree, so the file can still be reprinted;
//! * one mistake produces one diagnostic: the innermost construct that cannot
//!   proceed reports, and its callers stay quiet;
//! * a file of nonsense produces a bounded number of diagnostics
//!   ([`MAX_ERRORS`]), not one per token.

use nudo_diagnostics::{Diagnostic, DiagnosticCode, Diagnostics, codes};
use nudo_lexer::{Token, TokenKind, tokenize_with_trivia};
use nudo_source::{SourceFile, SourceId};
use nudo_span::Span;
use nudo_syntax::{SyntaxKind, SyntaxTree, TreeBuilder};

/// The largest number of syntactic diagnostics reported for one file.
///
/// A file of garbage would otherwise produce one diagnostic per token, which
/// buries the first real mistake and costs an agent that reads diagnostics more
/// than it tells it. The lexical diagnostics are never capped: they come from
/// a stage that already recovers.
pub const MAX_ERRORS: usize = 24;

/// How deeply expressions and blocks may nest before the parser stops
/// descending.
///
/// The parser is recursive, so without a limit a file of open delimiters would
/// exhaust the stack instead of producing a diagnostic. The limit is far above
/// anything a person writes and far below what a thread can hold.
pub const MAX_DEPTH: usize = 96;

/// The result of parsing one file: a tree, and everything wrong with it.
#[derive(Debug)]
pub struct Parse {
    tree: SyntaxTree,
    diagnostics: Diagnostics,
}

impl Parse {
    /// The syntax tree. It is complete and lossless even when the file has
    /// errors; a caller that wants structure without errors should look at
    /// [`SyntaxTree::validate`] and [`Parse::has_errors`].
    #[must_use]
    pub fn tree(&self) -> &SyntaxTree {
        &self.tree
    }

    /// The lexical and syntactic diagnostics, in source order.
    #[must_use]
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// Whether anything rejected the file.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// Splits the result into its tree and its diagnostics.
    #[must_use]
    pub fn into_parts(self) -> (SyntaxTree, Diagnostics) {
        (self.tree, self.diagnostics)
    }
}

/// Parses one file.
///
/// The result carries the lexical diagnostics too, because a caller wants one
/// answer to "what is wrong with this file": the parser sees a token stream the
/// lexer already repaired, and reporting them separately would ask every caller
/// to merge them in the same order.
#[must_use]
pub fn parse(source: &SourceFile) -> Parse {
    let lexed = tokenize_with_trivia(source);
    let (tokens, lexical) = lexed.into_parts();
    let already_reported: Vec<u32> = lexical
        .iter()
        .filter_map(|diagnostic| diagnostic.span().map(|span| span.start().get()))
        .collect();
    let mut parser = Parser::new(source.id(), source.text(), &tokens, already_reported);
    parser.parse_source_file();
    let (tree, diagnostics) = parser.finish();
    let mut all = lexical;
    all.extend_from(diagnostics);
    Parse {
        tree,
        diagnostics: all,
    }
}

/// Maps a lexical token kind to the syntax kind of the same token.
///
/// The two enums are parallel on purpose: the lexer's job is to say what bytes
/// are there, and the tree's job is to hold them. This function is exhaustive,
/// so a token added to the lexer cannot be forgotten here.
#[must_use]
pub fn syntax_kind(kind: TokenKind) -> SyntaxKind {
    match kind {
        TokenKind::Whitespace => SyntaxKind::Whitespace,
        TokenKind::Comment => SyntaxKind::Comment,
        TokenKind::Ident => SyntaxKind::Ident,
        TokenKind::IntLiteral => SyntaxKind::IntLiteral,
        TokenKind::FloatLiteral => SyntaxKind::FloatLiteral,
        TokenKind::TextLiteral => SyntaxKind::TextLiteral,
        TokenKind::Unknown => SyntaxKind::Unknown,
        TokenKind::KeywordFn => SyntaxKind::KeywordFn,
        TokenKind::KeywordLet => SyntaxKind::KeywordLet,
        TokenKind::KeywordStruct => SyntaxKind::KeywordStruct,
        TokenKind::KeywordEnum => SyntaxKind::KeywordEnum,
        TokenKind::KeywordIf => SyntaxKind::KeywordIf,
        TokenKind::KeywordElse => SyntaxKind::KeywordElse,
        TokenKind::KeywordMatch => SyntaxKind::KeywordMatch,
        TokenKind::KeywordConst => SyntaxKind::KeywordConst,
        TokenKind::KeywordTrue => SyntaxKind::KeywordTrue,
        TokenKind::KeywordFalse => SyntaxKind::KeywordFalse,
        TokenKind::LParen => SyntaxKind::LParen,
        TokenKind::RParen => SyntaxKind::RParen,
        TokenKind::LBrace => SyntaxKind::LBrace,
        TokenKind::RBrace => SyntaxKind::RBrace,
        TokenKind::LBracket => SyntaxKind::LBracket,
        TokenKind::RBracket => SyntaxKind::RBracket,
        TokenKind::Colon => SyntaxKind::Colon,
        TokenKind::ColonColon => SyntaxKind::ColonColon,
        TokenKind::Comma => SyntaxKind::Comma,
        TokenKind::Dot => SyntaxKind::Dot,
        TokenKind::Question => SyntaxKind::Question,
        TokenKind::Arrow => SyntaxKind::Arrow,
        TokenKind::FatArrow => SyntaxKind::FatArrow,
        TokenKind::Equals => SyntaxKind::Equals,
        TokenKind::EqEq => SyntaxKind::EqEq,
        TokenKind::BangEq => SyntaxKind::BangEq,
        TokenKind::Lt => SyntaxKind::Lt,
        TokenKind::Gt => SyntaxKind::Gt,
        TokenKind::LtEq => SyntaxKind::LtEq,
        TokenKind::GtEq => SyntaxKind::GtEq,
        TokenKind::AmpAmp => SyntaxKind::AmpAmp,
        TokenKind::PipePipe => SyntaxKind::PipePipe,
        TokenKind::Bang => SyntaxKind::Bang,
        TokenKind::Plus => SyntaxKind::Plus,
        TokenKind::Minus => SyntaxKind::Minus,
        TokenKind::Star => SyntaxKind::Star,
        TokenKind::Slash => SyntaxKind::Slash,
        TokenKind::Semi => SyntaxKind::Semi,
        TokenKind::Eof => SyntaxKind::Eof,
    }
}

/// The words that start an item. Used by recovery, not by dispatch.
const ITEM_WORDS: &[&str] = &["agent", "task", "tool", "model"];

/// The words that start a clause inside an `agent` or `task` declaration.
const AGENT_CLAUSE_WORDS: &[&str] = &["role", "tools", "allow", "budget"];

struct Parser<'a> {
    source: SourceId,
    text: &'a str,
    tokens: &'a [Token],
    /// The next token to place in the tree, trivia included.
    pos: usize,
    builder: TreeBuilder,
    diagnostics: Diagnostics,
    errors: usize,
    depth: usize,
    /// Byte offsets the lexer already rejected, so that the parser does not
    /// report the same position a second time.
    already_reported: Vec<u32>,
}

impl<'a> Parser<'a> {
    fn new(
        source: SourceId,
        text: &'a str,
        tokens: &'a [Token],
        already_reported: Vec<u32>,
    ) -> Self {
        Parser {
            source,
            text,
            tokens,
            pos: 0,
            builder: TreeBuilder::new(source.index(), text),
            diagnostics: Diagnostics::new(),
            errors: 0,
            depth: 0,
            already_reported,
        }
    }

    fn finish(self) -> (SyntaxTree, Diagnostics) {
        (self.builder.finish(), self.diagnostics)
    }

    // ------------------------------------------------------------ cursor ---

    /// The index of the next token that is content: trivia and characters the
    /// lexer could not tokenise are skipped, because nothing dispatches on them.
    fn significant_index(&self) -> usize {
        let mut index = self.pos;
        while index + 1 < self.tokens.len() && syntax_kind(self.tokens[index].kind()).is_skipped() {
            index += 1;
        }
        index
    }

    fn nth_index(&self, n: usize) -> usize {
        let mut index = self.significant_index();
        for _ in 0..n {
            if index + 1 >= self.tokens.len() {
                break;
            }
            index += 1;
            while index + 1 < self.tokens.len()
                && syntax_kind(self.tokens[index].kind()).is_skipped()
            {
                index += 1;
            }
        }
        index
    }

    fn kind(&self) -> SyntaxKind {
        syntax_kind(self.tokens[self.significant_index()].kind())
    }

    fn nth_kind(&self, n: usize) -> SyntaxKind {
        syntax_kind(self.tokens[self.nth_index(n)].kind())
    }

    fn span(&self) -> Span {
        self.tokens[self.significant_index()].span()
    }

    fn at(&self, kind: SyntaxKind) -> bool {
        self.kind() == kind
    }

    fn at_eof(&self) -> bool {
        self.at(SyntaxKind::Eof)
    }

    /// Whether the current token is an identifier spelled `word`.
    fn at_word(&self, word: &str) -> bool {
        self.at(SyntaxKind::Ident) && self.token_text(self.significant_index()) == Some(word)
    }

    fn token_text(&self, index: usize) -> Option<&'a str> {
        self.tokens[index].span().text(self.text)
    }

    /// How the current token is named in a diagnostic.
    fn found(&self) -> String {
        if self.at_eof() {
            return "end of file".to_string();
        }
        self.token_text(self.significant_index())
            .unwrap_or("?")
            .to_string()
    }

    /// Places the current token and its leading trivia in the tree.
    ///
    /// Every token of the file passes through here exactly once, in order,
    /// which is what makes the tree lossless.
    fn bump(&mut self) {
        let index = self.significant_index();
        for position in self.pos..=index {
            let token = self.tokens[position];
            self.builder.token(syntax_kind(token.kind()), token.span());
        }
        self.pos = index + 1;
    }

    /// Consumes the current token when it is `kind`.
    fn eat(&mut self, kind: SyntaxKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Consumes a token the parser cannot place, without reporting.
    ///
    /// Used by recovery, where the diagnostic for the mistake has already been
    /// emitted and a second one would report the same mistake twice.
    fn skip_token(&mut self) {
        if !self.at_eof() {
            self.bump();
        }
    }

    /// Opens a node whose first token is the next content token.
    ///
    /// Trivia that sits before that token is placed in the *enclosing* node
    /// first. Without this, a node's span — and so its `text()` — would begin
    /// in the middle of the whitespace in front of it, which makes every span a
    /// consumer reads slightly wrong. The trivia is still in the tree, one
    /// level up, so nothing is lost.
    fn start_node(&mut self, kind: SyntaxKind) {
        self.flush_trivia();
        self.builder.start_node(kind);
    }

    /// Places the trivia up to the next content token in the current node.
    fn flush_trivia(&mut self) {
        let index = self.significant_index();
        while self.pos < index {
            let token = self.tokens[self.pos];
            self.builder.token(syntax_kind(token.kind()), token.span());
            self.pos += 1;
        }
    }

    /// Marks the current position, after the preceding trivia.
    ///
    /// A checkpoint taken before the trivia would make the node built from it
    /// start in the whitespace in front of its first token.
    fn checkpoint(&mut self) -> nudo_syntax::Checkpoint {
        self.flush_trivia();
        self.builder.checkpoint()
    }

    // ------------------------------------------------------- diagnostics ---

    fn report(&mut self, code: DiagnosticCode, message: String, span: Span, note: Option<String>) {
        if self.errors >= MAX_ERRORS {
            return;
        }
        // One position, one diagnostic. If the lexer already rejected a byte
        // in the gap before this token, the parser has nothing to add: the
        // reader would see a cascade of two messages for one mistake, and an
        // agent repairing the file would fix the same byte twice.
        if self.suppressed(span) {
            return;
        }
        self.errors += 1;
        let mut diagnostic = Diagnostic::error(code, message).with_location(self.source, span);
        if let Some(note) = note {
            diagnostic = diagnostic.with_note(note);
        }
        self.diagnostics.push(diagnostic);
    }

    /// Whether the lexer already reported a byte in the gap that ends at
    /// `span`.
    ///
    /// The gap is what the parser did not place: trivia, and the characters the
    /// lexer could not tokenise. A diagnostic about a construct that could not
    /// be read *because* of such a byte is the same mistake, reported twice.
    fn suppressed(&self, span: Span) -> bool {
        if self.already_reported.is_empty() {
            return false;
        }
        let index = self.significant_index();
        // The gap reaches back to the start of the last token the parser read,
        // because the lexer's complaint can be *inside* that token: an
        // unterminated text literal is reported at its opening quote, and the
        // parser then finds the statement around it unreadable.
        let mut previous_start = 0;
        let mut position = index;
        while position > 0 {
            position -= 1;
            if !syntax_kind(self.tokens[position].kind()).is_skipped() {
                previous_start = self.tokens[position].span().start().get();
                break;
            }
        }
        let start = span.start().get();
        self.already_reported
            .iter()
            .any(|position| *position >= previous_start && *position < start)
    }

    /// Reports that the current token cannot appear here.
    fn unexpected(&mut self, expected: &str) {
        let span = self.span();
        let message = if self.at_eof() {
            "unexpected end of file".to_string()
        } else {
            format!("unexpected `{}`", self.found())
        };
        self.report(
            codes::UNEXPECTED_TOKEN,
            message,
            span,
            Some(format!("expected {expected}")),
        );
    }

    /// Reports that the current token could not be read as an item.
    fn error_item(&mut self) {
        let span = self.span();
        let message = if self.at_eof() {
            "unexpected end of file".to_string()
        } else {
            format!("unexpected `{}`", self.found())
        };
        self.report(
            codes::UNEXPECTED_TOKEN,
            message,
            span,
            Some(
                "expected an item: `fn`, `let`, `const`, `struct`, `enum`, `agent`, `task`, \
                 `tool` or `model`"
                    .to_string(),
            ),
        );
    }

    /// Reports a contextual word that is in the right place but has the wrong
    /// shape after it.
    ///
    /// NEP-0005 requires this to say what the construct needs, rather than
    /// leaving the reader with an unexplained identifier.
    fn error_clause(&mut self, word: &str, expected: &str) {
        let span = self.span();
        let message = if self.at_eof() {
            "unexpected end of file".to_string()
        } else {
            format!("unexpected `{}`", self.found())
        };
        self.report(
            codes::UNEXPECTED_TOKEN,
            message,
            span,
            Some(format!("`{word}` needs {expected}")),
        );
    }

    /// Reports an expression or a block that nests deeper than the parser will
    /// follow.
    ///
    /// The limit exists because the parser is recursive: without it, a file of
    /// ten thousand open parentheses would abort the process instead of
    /// reporting a diagnostic.
    fn error_too_deep(&mut self, what: &str) {
        let span = self.span();
        self.report(
            codes::UNEXPECTED_TOKEN,
            format!("this {what} nests too deeply"),
            span,
            Some(format!(
                "the parser follows at most {MAX_DEPTH} nested {what}s, so that a malformed file cannot exhaust the stack"
            )),
        );
    }

    // ------------------------------------------------------------ items ---

    fn parse_source_file(&mut self) {
        // The root node is opened directly: flushing trivia would need a node
        // to put it in, and there is none yet. Trivia at the very start of the
        // file therefore belongs to the file, which is where it should be.
        self.builder.start_node(SyntaxKind::SourceFile);
        while !self.at_eof() {
            let before = self.pos;
            self.parse_item();
            if self.pos == before {
                self.recover_item();
            }
        }
        // The trailing trivia and the `Eof` token belong to the file: dropping
        // them would make the tree unable to reprint the source.
        self.bump();
        self.builder.finish_node();
    }

    fn parse_item(&mut self) {
        match self.kind() {
            SyntaxKind::KeywordFn => self.parse_function_decl(),
            SyntaxKind::KeywordLet => self.parse_binding(),
            SyntaxKind::KeywordConst => self.parse_const_decl(),
            SyntaxKind::KeywordStruct => self.parse_struct_decl(),
            SyntaxKind::KeywordEnum => self.parse_enum_decl(),
            SyntaxKind::Ident if self.at_word("agent") => self.parse_agent_decl(),
            SyntaxKind::Ident if self.at_word("task") => self.parse_task_decl(),
            SyntaxKind::Ident if self.at_word("tool") => self.parse_tool_decl(),
            SyntaxKind::Ident if self.at_word("model") => self.parse_model_decl(),
            _ => self.error_item(),
        }
    }

    /// Whether the current token can start an item. Used only to decide where
    /// recovery stops.
    fn at_item_start(&self) -> bool {
        match self.kind() {
            SyntaxKind::KeywordFn
            | SyntaxKind::KeywordLet
            | SyntaxKind::KeywordConst
            | SyntaxKind::KeywordStruct
            | SyntaxKind::KeywordEnum => true,
            SyntaxKind::Ident => ITEM_WORDS.iter().any(|word| self.at_word(word)),
            _ => false,
        }
    }

    /// Consumes what cannot start an item, up to the next item.
    fn recover_item(&mut self) {
        self.start_node(SyntaxKind::Error);
        while !self.at_eof() && !self.at_item_start() {
            self.skip_token();
        }
        self.builder.finish_node();
    }

    // ------------------------------------------------------- declarations ---

    fn parse_function_decl(&mut self) {
        self.start_node(SyntaxKind::FunctionDecl);
        self.bump(); // `fn`
        if !self.expect_ident("a function name") {
            self.builder.finish_node();
            return;
        }
        self.parse_parameter_list();
        if self.eat(SyntaxKind::Arrow) {
            self.parse_type();
        }
        if self.at_word("with") {
            self.parse_effect_clause();
        }
        self.parse_block();
        self.builder.finish_node();
    }

    fn parse_struct_decl(&mut self) {
        self.start_node(SyntaxKind::StructDecl);
        self.bump(); // `struct`
        if !self.expect_ident("a struct name") {
            self.builder.finish_node();
            return;
        }
        if !self.expect(SyntaxKind::LBrace, "`{`") {
            self.builder.finish_node();
            return;
        }
        while !self.at(SyntaxKind::RBrace) && !self.at_eof() {
            if self.at(SyntaxKind::Ident) {
                self.start_node(SyntaxKind::Field);
                self.bump();
                if self.eat(SyntaxKind::Colon) {
                    self.parse_type();
                } else {
                    self.unexpected("`:` and the field's type");
                }
                self.builder.finish_node();
            } else {
                self.error_and_recover("a field of the form `name: Type`", &[SyntaxKind::RBrace]);
            }
        }
        self.expect(SyntaxKind::RBrace, "`}`");
        self.builder.finish_node();
    }

    fn parse_enum_decl(&mut self) {
        self.start_node(SyntaxKind::EnumDecl);
        self.bump(); // `enum`
        if !self.expect_ident("an enum name") {
            self.builder.finish_node();
            return;
        }
        if !self.expect(SyntaxKind::LBrace, "`{`") {
            self.builder.finish_node();
            return;
        }
        while !self.at(SyntaxKind::RBrace) && !self.at_eof() {
            if self.at(SyntaxKind::Ident) {
                self.start_node(SyntaxKind::Variant);
                self.bump();
                if self.eat(SyntaxKind::LParen) {
                    if !self.at(SyntaxKind::RParen) {
                        loop {
                            self.parse_field();
                            if !self.eat(SyntaxKind::Comma) {
                                break;
                            }
                        }
                    }
                    self.expect(SyntaxKind::RParen, "`)`");
                }
                self.builder.finish_node();
            } else {
                self.error_and_recover("an enum variant", &[SyntaxKind::RBrace]);
            }
        }
        self.expect(SyntaxKind::RBrace, "`}`");
        self.builder.finish_node();
    }

    fn parse_field(&mut self) {
        self.start_node(SyntaxKind::Field);
        if self.expect_ident("a field name") && self.eat(SyntaxKind::Colon) {
            self.parse_type();
        } else if !self.at(SyntaxKind::Comma) && !self.at(SyntaxKind::RParen) {
            self.unexpected("`:` and the field's type");
        }
        self.builder.finish_node();
    }

    fn parse_const_decl(&mut self) {
        self.start_node(SyntaxKind::ConstDecl);
        self.bump(); // `const`
        self.expect_ident("a constant name");
        if self.eat(SyntaxKind::Colon) {
            self.parse_type();
        } else {
            self.unexpected("`:` and a type");
        }
        if self.eat(SyntaxKind::Equals) {
            self.parse_expression();
        } else {
            self.unexpected("`=` and a value");
        }
        self.expect(SyntaxKind::Semi, "`;`");
        self.builder.finish_node();
    }

    fn parse_binding(&mut self) {
        self.start_node(SyntaxKind::Binding);
        self.bump(); // `let`
        self.expect_ident("a binding name");
        if self.eat(SyntaxKind::Colon) {
            self.parse_type();
        }
        if self.eat(SyntaxKind::Equals) {
            self.parse_expression();
        } else {
            self.unexpected("`=` after a binding name");
            // Skip what follows, so that one mistake does not produce a
            // diagnostic for the missing `=`, another for the `;` that is two
            // tokens away, and a third for the token the item loop then finds.
            self.recover_in_node(&[SyntaxKind::Semi, SyntaxKind::RBrace]);
        }
        self.expect(SyntaxKind::Semi, "`;`");
        self.builder.finish_node();
    }

    fn parse_agent_decl(&mut self) {
        self.start_node(SyntaxKind::AgentDecl);
        self.bump(); // `agent`
        if !self.expect_ident("an agent name") {
            self.builder.finish_node();
            return;
        }
        if !self.expect(SyntaxKind::LBrace, "`{`") {
            self.builder.finish_node();
            return;
        }
        while !self.at(SyntaxKind::RBrace) && !self.at_eof() {
            if self.at_agent_clause() {
                self.parse_agent_clause();
            } else {
                self.error_and_recover(
                    "a clause: `role`, `tools`, `allow` or `budget`",
                    &[SyntaxKind::RBrace],
                );
            }
        }
        self.expect(SyntaxKind::RBrace, "`}`");
        self.builder.finish_node();
    }

    fn at_agent_clause(&self) -> bool {
        SyntaxKind::Ident == self.kind() && AGENT_CLAUSE_WORDS.iter().any(|word| self.at_word(word))
    }

    fn parse_agent_clause(&mut self) {
        if self.at_word("role") {
            self.parse_role_clause();
        } else if self.at_word("tools") {
            self.parse_tools_clause();
        } else if self.at_word("allow") {
            self.parse_allow_clause();
        } else {
            self.parse_budget_clause();
        }
    }

    fn parse_role_clause(&mut self) {
        self.start_node(SyntaxKind::RoleClause);
        self.bump(); // `role`
        if self.eat(SyntaxKind::Colon) {
            self.expect(SyntaxKind::TextLiteral, "a text literal");
        } else {
            self.error_clause("role", "a role clause of the form `role: \"…\"`");
        }
        self.builder.finish_node();
    }

    fn parse_tools_clause(&mut self) {
        self.start_node(SyntaxKind::ToolsClause);
        self.bump(); // `tools`
        if self.eat(SyntaxKind::Colon) {
            self.parse_path();
            while self.eat(SyntaxKind::Comma) {
                self.parse_path();
            }
        } else {
            self.error_clause("tools", "a tools clause of the form `tools: a.b, c.d`");
        }
        self.builder.finish_node();
    }

    fn parse_allow_clause(&mut self) {
        self.start_node(SyntaxKind::AllowClause);
        self.bump(); // `allow`
        if self.eat(SyntaxKind::Colon) {
            self.parse_capability_list();
        } else {
            self.error_clause("allow", "an allow clause of the form `allow: Network`");
        }
        self.builder.finish_node();
    }

    fn parse_budget_clause(&mut self) {
        self.start_node(SyntaxKind::BudgetClause);
        self.bump(); // `budget`
        if self.eat(SyntaxKind::Colon) {
            self.parse_budget_literal();
        } else {
            self.error_clause(
                "budget",
                "a budget clause of the form `budget: Budget(tokens: 1_000)`",
            );
        }
        self.builder.finish_node();
    }

    fn parse_budget_literal(&mut self) {
        if !self.at_word("Budget") {
            self.unexpected("`Budget(…)`");
            return;
        }
        self.start_node(SyntaxKind::BudgetLiteral);
        self.bump(); // `Budget`
        if !self.expect(SyntaxKind::LParen, "`(`") {
            self.builder.finish_node();
            return;
        }
        if !self.at(SyntaxKind::RParen) {
            loop {
                self.start_node(SyntaxKind::BudgetField);
                if self.expect_ident("a budget field name") {
                    if self.eat(SyntaxKind::Colon) {
                        self.parse_expression();
                    } else {
                        self.unexpected("`:` and the field's value");
                    }
                }
                self.builder.finish_node();
                if !self.eat(SyntaxKind::Comma) {
                    break;
                }
            }
        }
        self.expect(SyntaxKind::RParen, "`)`");
        self.builder.finish_node();
    }

    fn parse_task_decl(&mut self) {
        self.start_node(SyntaxKind::TaskDecl);
        self.bump(); // `task`
        if !self.expect_ident("a task name") {
            self.builder.finish_node();
            return;
        }
        self.parse_parameter_list();
        if self.eat(SyntaxKind::Arrow) {
            self.parse_type();
        } else {
            self.unexpected("`->` and the task's result type");
        }
        if !self.expect(SyntaxKind::LBrace, "`{`") {
            self.builder.finish_node();
            return;
        }
        while !self.at(SyntaxKind::RBrace) && !self.at_eof() {
            if self.at_word("agent") {
                self.start_node(SyntaxKind::TaskAgentClause);
                self.bump();
                if self.eat(SyntaxKind::Colon) {
                    self.parse_path();
                } else {
                    self.error_clause("agent", "an agent clause of the form `agent: Name`");
                }
                self.builder.finish_node();
            } else if self.at_word("verify") {
                self.start_node(SyntaxKind::VerifyClause);
                self.bump();
                if self.eat(SyntaxKind::Colon) {
                    self.parse_path();
                } else {
                    self.error_clause("verify", "a verify clause of the form `verify: Name`");
                }
                self.builder.finish_node();
            } else if self.at_agent_clause() {
                self.parse_agent_clause();
            } else {
                self.error_and_recover(
                    "a clause: `agent`, `verify`, `role`, `tools`, `allow` or `budget`",
                    &[SyntaxKind::RBrace],
                );
            }
        }
        self.expect(SyntaxKind::RBrace, "`}`");
        self.builder.finish_node();
    }

    fn parse_tool_decl(&mut self) {
        self.start_node(SyntaxKind::ToolDecl);
        self.bump(); // `tool`
        self.parse_path();
        self.parse_parameter_list();
        if self.eat(SyntaxKind::Arrow) {
            self.parse_type();
        }
        if self.at_word("with") {
            self.parse_effect_clause();
        }
        self.parse_block();
        self.builder.finish_node();
    }

    fn parse_model_decl(&mut self) {
        self.start_node(SyntaxKind::ModelDecl);
        self.bump(); // `model`
        if !self.expect_ident("a model name") {
            self.builder.finish_node();
            return;
        }
        if !self.expect(SyntaxKind::LBrace, "`{`") {
            self.builder.finish_node();
            return;
        }
        while !self.at(SyntaxKind::RBrace) && !self.at_eof() {
            if self.at(SyntaxKind::Ident) {
                self.start_node(SyntaxKind::ModelField);
                self.bump();
                if self.eat(SyntaxKind::Colon) {
                    self.parse_expression();
                } else {
                    self.unexpected("`:` and the field's value");
                }
                self.builder.finish_node();
            } else {
                self.error_and_recover("a field of the form `name: value`", &[SyntaxKind::RBrace]);
            }
        }
        self.expect(SyntaxKind::RBrace, "`}`");
        self.builder.finish_node();
    }

    // ----------------------------------------------------------- clauses ---

    fn parse_parameter_list(&mut self) {
        if !self.at(SyntaxKind::LParen) {
            self.unexpected("`(`");
            return;
        }
        self.start_node(SyntaxKind::ParameterList);
        self.bump(); // `(`
        if !self.at(SyntaxKind::RParen) {
            loop {
                if self.at_eof() || self.at(SyntaxKind::LBrace) {
                    break;
                }
                let before = self.pos;
                self.start_node(SyntaxKind::Parameter);
                if self.expect_ident("a parameter name") {
                    if self.eat(SyntaxKind::Colon) {
                        self.parse_type();
                    } else {
                        self.unexpected("`:` and the parameter's type");
                    }
                }
                self.builder.finish_node();
                if self.pos == before {
                    self.error_and_recover(
                        "a parameter of the form `name: Type`",
                        &[SyntaxKind::RParen, SyntaxKind::LBrace],
                    );
                    continue;
                }
                if !self.eat(SyntaxKind::Comma) {
                    break;
                }
            }
        }
        self.expect(SyntaxKind::RParen, "`)`");
        self.builder.finish_node();
    }

    fn parse_effect_clause(&mut self) {
        self.start_node(SyntaxKind::EffectClause);
        self.bump(); // `with`
        self.parse_capability_list();
        self.builder.finish_node();
    }

    fn parse_capability_list(&mut self) {
        self.start_node(SyntaxKind::CapabilityList);
        loop {
            let before = self.pos;
            self.start_node(SyntaxKind::Capability);
            self.expect_ident("a capability name");
            self.builder.finish_node();
            if self.pos == before {
                break;
            }
            if !self.eat(SyntaxKind::Comma) {
                break;
            }
        }
        self.builder.finish_node();
    }

    // ------------------------------------------------------------- types ---

    fn parse_type(&mut self) {
        let checkpoint = self.checkpoint();
        let before = self.pos;
        self.parse_primary_type();
        if self.pos != before && self.at(SyntaxKind::Question) {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::OptionalType);
            self.bump(); // `?`
            self.builder.finish_node();
        }
    }

    fn parse_primary_type(&mut self) {
        match self.kind() {
            SyntaxKind::LBracket => {
                self.start_node(SyntaxKind::SequenceType);
                self.bump();
                self.parse_type();
                self.expect(SyntaxKind::RBracket, "`]`");
                self.builder.finish_node();
            }
            SyntaxKind::LParen => {
                self.start_node(SyntaxKind::ParenthesizedType);
                self.bump();
                self.parse_type();
                self.expect(SyntaxKind::RParen, "`)`");
                self.builder.finish_node();
            }
            SyntaxKind::Ident if self.at_word("Fn") && self.nth_kind(1) == SyntaxKind::LParen => {
                self.parse_function_type();
            }
            SyntaxKind::Ident => self.parse_path_type(),
            _ => self.unexpected("a type"),
        }
    }

    fn parse_function_type(&mut self) {
        self.start_node(SyntaxKind::FunctionType);
        self.bump(); // `Fn`
        self.bump(); // `(`
        if !self.at(SyntaxKind::RParen) {
            loop {
                let before = self.pos;
                self.parse_type();
                if self.pos == before || !self.eat(SyntaxKind::Comma) {
                    break;
                }
            }
        }
        self.expect(SyntaxKind::RParen, "`)`");
        if self.eat(SyntaxKind::Arrow) {
            self.parse_type();
        } else {
            self.unexpected("`->`");
        }
        self.builder.finish_node();
    }

    fn parse_path_type(&mut self) {
        let checkpoint = self.checkpoint();
        let before = self.pos;
        self.parse_path();
        if self.pos == before {
            return;
        }
        if self.at(SyntaxKind::Lt) {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::GenericType);
            self.bump(); // `<`
            loop {
                let type_start = self.pos;
                self.parse_type();
                if self.pos == type_start || !self.eat(SyntaxKind::Comma) {
                    break;
                }
            }
            self.expect(SyntaxKind::Gt, "`>`");
            self.builder.finish_node();
        } else {
            self.builder.start_node_at(checkpoint, SyntaxKind::PathType);
            self.builder.finish_node();
        }
    }

    // ------------------------------------------------------- expressions ---

    fn parse_block(&mut self) {
        if self.depth >= MAX_DEPTH {
            self.error_too_deep("block");
            return;
        }
        self.depth += 1;
        self.parse_block_inner();
        self.depth -= 1;
    }

    fn parse_block_inner(&mut self) {
        if !self.at(SyntaxKind::LBrace) {
            self.unexpected("`{`");
            return;
        }
        self.start_node(SyntaxKind::Block);
        self.bump(); // `{`
        loop {
            if self.at(SyntaxKind::RBrace) || self.at_eof() {
                break;
            }
            if self.at(SyntaxKind::KeywordLet) {
                let before = self.pos;
                self.parse_binding();
                if self.pos == before {
                    self.recover_in_node(&[SyntaxKind::Semi, SyntaxKind::RBrace]);
                }
                continue;
            }
            let checkpoint = self.checkpoint();
            let before = self.pos;
            self.parse_expression();
            if self.pos == before {
                self.recover_in_node(&[SyntaxKind::Semi, SyntaxKind::RBrace]);
                continue;
            }
            if self.eat(SyntaxKind::Semi) {
                self.builder
                    .start_node_at(checkpoint, SyntaxKind::ExpressionStatement);
                self.builder.finish_node();
                continue;
            }
            // An expression with no `;` is the block's value, and the block
            // ends here.
            break;
        }
        self.expect(SyntaxKind::RBrace, "`}`");
        self.builder.finish_node();
    }

    fn parse_expression(&mut self) {
        self.parse_logical_or();
    }

    fn parse_logical_or(&mut self) {
        let checkpoint = self.checkpoint();
        let before = self.pos;
        self.parse_logical_and();
        if self.pos == before {
            return;
        }
        while self.at(SyntaxKind::PipePipe) {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::BinaryExpression);
            self.bump();
            self.parse_logical_and();
            self.builder.finish_node();
        }
    }

    fn parse_logical_and(&mut self) {
        let checkpoint = self.checkpoint();
        let before = self.pos;
        self.parse_equality();
        if self.pos == before {
            return;
        }
        while self.at(SyntaxKind::AmpAmp) {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::BinaryExpression);
            self.bump();
            self.parse_equality();
            self.builder.finish_node();
        }
    }

    fn parse_equality(&mut self) {
        let checkpoint = self.checkpoint();
        let before = self.pos;
        self.parse_comparison();
        if self.pos == before {
            return;
        }
        while self.at(SyntaxKind::EqEq) || self.at(SyntaxKind::BangEq) {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::BinaryExpression);
            self.bump();
            self.parse_comparison();
            self.builder.finish_node();
        }
    }

    /// Comparison does not chain: `a < b < c` is rejected, not read as
    /// `(a < b) < c`.
    ///
    /// The extra operators are still parsed into the tree, so that one mistake
    /// produces one diagnostic and the file keeps a shape a formatter can
    /// print.
    fn parse_comparison(&mut self) {
        let checkpoint = self.checkpoint();
        let before = self.pos;
        self.parse_additive();
        if self.pos == before || !self.at_comparison_operator() {
            return;
        }
        self.builder
            .start_node_at(checkpoint, SyntaxKind::BinaryExpression);
        self.bump();
        self.parse_additive();
        self.builder.finish_node();
        while self.at_comparison_operator() {
            let span = self.span();
            self.report(
                codes::UNEXPECTED_TOKEN,
                "comparison operators do not chain".to_string(),
                span,
                Some("write `a < b && b < c` instead of `a < b < c`".to_string()),
            );
            self.builder
                .start_node_at(checkpoint, SyntaxKind::BinaryExpression);
            self.bump();
            self.parse_additive();
            self.builder.finish_node();
        }
    }

    fn at_comparison_operator(&self) -> bool {
        matches!(
            self.kind(),
            SyntaxKind::Lt | SyntaxKind::Gt | SyntaxKind::LtEq | SyntaxKind::GtEq
        )
    }

    fn parse_additive(&mut self) {
        let checkpoint = self.checkpoint();
        let before = self.pos;
        self.parse_multiplicative();
        if self.pos == before {
            return;
        }
        while self.at(SyntaxKind::Plus) || self.at(SyntaxKind::Minus) {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::BinaryExpression);
            self.bump();
            self.parse_multiplicative();
            self.builder.finish_node();
        }
    }

    fn parse_multiplicative(&mut self) {
        let checkpoint = self.checkpoint();
        let before = self.pos;
        self.parse_unary();
        if self.pos == before {
            return;
        }
        while self.at(SyntaxKind::Star) || self.at(SyntaxKind::Slash) {
            self.builder
                .start_node_at(checkpoint, SyntaxKind::BinaryExpression);
            self.bump();
            self.parse_unary();
            self.builder.finish_node();
        }
    }

    fn parse_unary(&mut self) {
        if self.at(SyntaxKind::Minus) || self.at(SyntaxKind::Bang) {
            self.start_node(SyntaxKind::UnaryExpression);
            self.bump();
            self.parse_unary();
            self.builder.finish_node();
            return;
        }
        self.parse_postfix();
    }

    fn parse_postfix(&mut self) {
        let checkpoint = self.checkpoint();
        let before = self.pos;
        self.parse_primary();
        if self.pos == before {
            return;
        }
        loop {
            match self.kind() {
                SyntaxKind::LParen => {
                    self.builder
                        .start_node_at(checkpoint, SyntaxKind::CallExpression);
                    self.parse_argument_list();
                    self.builder.finish_node();
                }
                SyntaxKind::Dot => {
                    self.builder
                        .start_node_at(checkpoint, SyntaxKind::FieldExpression);
                    self.bump();
                    self.expect_ident("a field name after `.`");
                    self.builder.finish_node();
                }
                SyntaxKind::LBracket => {
                    self.builder
                        .start_node_at(checkpoint, SyntaxKind::IndexExpression);
                    self.bump();
                    self.parse_expression();
                    self.expect(SyntaxKind::RBracket, "`]`");
                    self.builder.finish_node();
                }
                _ => break,
            }
        }
    }

    fn parse_argument_list(&mut self) {
        self.start_node(SyntaxKind::ArgumentList);
        self.bump(); // `(`
        if !self.at(SyntaxKind::RParen) {
            loop {
                let before = self.pos;
                self.parse_expression();
                if self.pos == before || !self.eat(SyntaxKind::Comma) {
                    break;
                }
            }
        }
        self.expect(SyntaxKind::RParen, "`)`");
        self.builder.finish_node();
    }

    fn parse_primary(&mut self) {
        if self.depth >= MAX_DEPTH {
            self.error_too_deep("expression");
            return;
        }
        self.depth += 1;
        self.parse_primary_inner();
        self.depth -= 1;
    }

    fn parse_primary_inner(&mut self) {
        match self.kind() {
            SyntaxKind::IntLiteral
            | SyntaxKind::FloatLiteral
            | SyntaxKind::TextLiteral
            | SyntaxKind::KeywordTrue
            | SyntaxKind::KeywordFalse => {
                self.start_node(SyntaxKind::LiteralExpression);
                self.bump();
                self.builder.finish_node();
            }
            SyntaxKind::LParen => {
                self.start_node(SyntaxKind::ParenthesizedExpression);
                self.bump();
                self.parse_expression();
                self.expect(SyntaxKind::RParen, "`)`");
                self.builder.finish_node();
            }
            SyntaxKind::LBrace => self.parse_block(),
            SyntaxKind::KeywordIf => self.parse_if_expression(),
            SyntaxKind::KeywordMatch => self.parse_match_expression(),
            SyntaxKind::Ident if self.at_word("ask") && self.nth_kind(1) == SyntaxKind::Ident => {
                self.parse_ask_expression();
            }
            SyntaxKind::Ident
                if self.at_word("delegate") && self.nth_kind(1) == SyntaxKind::Ident =>
            {
                self.parse_delegate_expression();
            }
            SyntaxKind::Ident
                if self.at_word("verify") && self.starts_operand(self.nth_kind(1)) =>
            {
                self.parse_verify_expression();
            }
            SyntaxKind::Ident => {
                self.start_node(SyntaxKind::PathExpression);
                self.parse_path();
                self.builder.finish_node();
            }
            _ => self.unexpected("an expression"),
        }
    }

    /// Whether a token can start an operand.
    ///
    /// `(` is excluded on purpose: after `verify`, a `(` is read as a call on
    /// something named `verify`. That is the one place where a contextual word
    /// is followed by something that could also continue a path, and the
    /// limitation is recorded in the parser design document.
    fn starts_operand(&self, kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SyntaxKind::Ident
                | SyntaxKind::IntLiteral
                | SyntaxKind::FloatLiteral
                | SyntaxKind::TextLiteral
                | SyntaxKind::KeywordTrue
                | SyntaxKind::KeywordFalse
                | SyntaxKind::KeywordIf
                | SyntaxKind::KeywordMatch
                | SyntaxKind::LBrace
        )
    }

    fn parse_if_expression(&mut self) {
        self.start_node(SyntaxKind::IfExpression);
        self.bump(); // `if`
        self.parse_expression();
        self.parse_block();
        if self.at(SyntaxKind::KeywordElse) {
            self.bump();
            if self.at(SyntaxKind::KeywordIf) {
                self.parse_if_expression();
            } else {
                self.parse_block();
            }
        }
        self.builder.finish_node();
    }

    fn parse_match_expression(&mut self) {
        self.start_node(SyntaxKind::MatchExpression);
        self.bump(); // `match`
        self.parse_expression();
        if !self.expect(SyntaxKind::LBrace, "`{`") {
            self.builder.finish_node();
            return;
        }
        while !self.at(SyntaxKind::RBrace) && !self.at_eof() {
            if !self.at(SyntaxKind::Ident) {
                self.error_and_recover(
                    "a match arm of the form `pattern => expression`",
                    &[SyntaxKind::RBrace],
                );
                continue;
            }
            self.start_node(SyntaxKind::MatchArm);
            self.parse_pattern();
            if self.eat(SyntaxKind::FatArrow) {
                self.parse_expression();
            } else {
                self.unexpected("`=>`");
            }
            self.builder.finish_node();
        }
        self.expect(SyntaxKind::RBrace, "`}`");
        self.builder.finish_node();
    }

    fn parse_pattern(&mut self) {
        self.start_node(SyntaxKind::Pattern);
        self.parse_path();
        if self.eat(SyntaxKind::LParen) {
            if !self.at(SyntaxKind::RParen) {
                loop {
                    let before = self.pos;
                    self.parse_pattern();
                    if self.pos == before || !self.eat(SyntaxKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(SyntaxKind::RParen, "`)`");
        }
        self.builder.finish_node();
    }

    /// `ask Name { … }` — a model call, which always yields a `Generated<T>`.
    fn parse_ask_expression(&mut self) {
        self.start_node(SyntaxKind::AskExpression);
        self.bump(); // `ask`
        self.parse_path();
        self.parse_block();
        self.builder.finish_node();
    }

    /// `verify value with Verifier` — an explicit, fallible step.
    fn parse_verify_expression(&mut self) {
        self.start_node(SyntaxKind::VerifyExpression);
        self.bump(); // `verify`
        self.parse_expression();
        if self.at_word("with") {
            self.bump();
            self.parse_path();
        } else {
            self.error_clause(
                "verify",
                "a verification of the form `verify value with Verifier`",
            );
        }
        self.builder.finish_node();
    }

    /// `delegate Name { … } with budget: Budget(…)` — work handed to an agent,
    /// with authority that can only narrow.
    fn parse_delegate_expression(&mut self) {
        self.start_node(SyntaxKind::DelegateExpression);
        self.bump(); // `delegate`
        self.parse_path();
        self.parse_block();
        if self.at_word("with") {
            self.bump();
            if self.at_word("budget") {
                self.bump();
                if self.eat(SyntaxKind::Colon) {
                    self.parse_budget_literal();
                } else {
                    self.error_clause(
                        "budget",
                        "a budget clause of the form `with budget: Budget(…)`",
                    );
                }
            } else {
                self.error_clause(
                    "with",
                    "a budget clause of the form `with budget: Budget(…)`",
                );
            }
        }
        self.builder.finish_node();
    }

    fn parse_path(&mut self) {
        if !self.at(SyntaxKind::Ident) {
            self.unexpected("a name");
            return;
        }
        self.start_node(SyntaxKind::Path);
        self.bump();
        while self.at(SyntaxKind::ColonColon) {
            self.bump();
            self.expect_ident("a name after `::`");
        }
        self.builder.finish_node();
    }

    // ---------------------------------------------------------- recovery ---

    /// Consumes tokens into an `Error` node until one of `sync` appears.
    ///
    /// The token that stops the skip is left alone, so the construct that
    /// expects it can report the mistake once.
    fn recover_in_node(&mut self, sync: &[SyntaxKind]) {
        self.start_node(SyntaxKind::Error);
        while !self.at_eof() && !sync.iter().any(|kind| self.at(*kind)) {
            self.skip_token();
        }
        self.builder.finish_node();
    }

    /// Reports a token that cannot start the construct being read, then skips
    /// it and its neighbours up to `sync`.
    ///
    /// Skipping without reporting is how a parser silently accepts a file it
    /// did not understand, which is worse than rejecting it: the caller sees
    /// exit code 0 and believes the file was read. Every recovery that follows
    /// a token the loop could not start goes through here.
    fn error_and_recover(&mut self, expected: &str, sync: &[SyntaxKind]) {
        if sync.iter().any(|kind| self.at(*kind)) {
            return;
        }
        let before = self.pos;
        self.unexpected(expected);
        self.recover_in_node(sync);
        if self.pos == before && !self.at_eof() {
            // Nothing was consumed, and the caller's loop would spin.
            self.skip_token();
        }
    }

    /// Consumes the current token when it is `kind`, or reports what was
    /// expected.
    fn expect(&mut self, kind: SyntaxKind, expected: &str) -> bool {
        if self.eat(kind) {
            return true;
        }
        self.unexpected(expected);
        false
    }

    /// Consumes the current token when it is an identifier, or reports what was
    /// expected.
    fn expect_ident(&mut self, expected: &str) -> bool {
        if self.eat(SyntaxKind::Ident) {
            return true;
        }
        self.unexpected(expected);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nudo_source::SourceMap;
    use nudo_syntax::{SyntaxNode, dump_tree};

    fn parse_text(text: &str) -> (String, Vec<String>, Vec<u32>) {
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", text);
        let file = sources.get(id).expect("just added");
        let parsed = parse(file);
        let tree = parsed.tree();
        assert!(
            tree.is_lossless(),
            "the tree dropped bytes:\n{text}\n---\n{}",
            tree.reprint()
        );
        let codes = parsed
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().id().to_string())
            .collect();
        let kinds: Vec<u32> = parsed
            .diagnostics()
            .iter()
            .filter_map(|diagnostic| diagnostic.span().map(|span| span.start().get()))
            .collect();
        (dump_tree(tree), codes, kinds)
    }

    fn error_codes(text: &str) -> Vec<String> {
        let (_, codes, _) = parse_text(text);
        codes
    }

    fn tree_of(text: &str) -> (String, Vec<u32>) {
        let (dump, _, starts) = parse_text(text);
        (dump, starts)
    }

    #[test]
    fn parses_a_function_with_a_body() {
        let (dump, errors, _) = parse_text("fn add(a: Int, b: Int) -> Int {\n    a + b\n}\n");
        assert_eq!(errors, Vec::<String>::new(), "{dump}");
        assert!(dump.contains("FunctionDecl"), "{dump}");
        assert!(dump.contains("ParameterList"), "{dump}");
        assert!(dump.contains("BinaryExpression"), "{dump}");
        assert!(!dump.contains("Error"), "{dump}");
    }

    #[test]
    fn parses_items_of_every_kind() {
        for source in [
            "fn f() {}",
            "let answer = 42;",
            "const LIMIT: Int = 1_000;",
            "struct User {\n    name: Text\n}",
            "enum Status {\n    Pending\n    Failed(reason: Text)\n}",
            // The grammar's `tool-name` is a `path`, and a path uses `::`.
            // The dotted spelling in `examples/05-agent` is one of the
            // grammar-versus-example conflicts recorded in the pull request
            // that added M2; the grammar is normative here.
            "agent R {\n    role: \"research\"\n    tools: web::search, web::open\n    allow: Network\n    budget: Budget(tokens: 20_000)\n}",
            "task T(topic: Text) -> Verified<Report> {\n    agent: R\n    verify: Sources\n    budget: Budget(tokens: 1)\n}",
            "tool web::search(query: Text) -> [Result] with Network, Budget {}",
            "model Local {\n    provider: \"local\"\n}",
        ] {
            let codes = error_codes(source);
            assert_eq!(
                codes,
                Vec::<String>::new(),
                "`{source}` should parse without diagnostics"
            );
        }
    }

    #[test]
    fn precedence_is_the_shape_of_the_tree() {
        // `1 + 2 * 3` is `1 + (2 * 3)`: the additive node's right child is a
        // binary node, and the whole thing is one additive at the top.
        let (dump, errors, _) = parse_text("let x = 1 + 2 * 3;");
        assert!(errors.is_empty());
        let root_index = dump.find("BinaryExpression").expect("a binary node");
        let plus = dump.find("\"+\"").expect("the plus operator");
        let star = dump.find("\"*\"").expect("the star operator");
        assert!(root_index < plus && plus < star, "{dump}");
        assert_eq!(dump.matches("BinaryExpression").count(), 2, "{dump}");
    }

    #[test]
    fn postfix_forms_chain_left() {
        let (dump, errors, _) = parse_text("let x = a.b(c)[d];");
        assert!(errors.is_empty(), "{dump}");
        assert_eq!(dump.matches("CallExpression").count(), 1, "{dump}");
        assert_eq!(dump.matches("FieldExpression").count(), 1, "{dump}");
        assert_eq!(dump.matches("IndexExpression").count(), 1, "{dump}");
    }

    #[test]
    fn comparison_does_not_chain() {
        let codes = error_codes("fn f() { a < b < c }");
        assert_eq!(codes, vec!["NDO1001".to_string()]);
    }

    #[test]
    fn an_optional_type_wraps_once() {
        let (dump, errors, _) = parse_text("fn f() -> Text? { }");
        assert!(errors.is_empty());
        assert!(dump.contains("OptionalType"), "{dump}");
        let codes = error_codes("let x: Text?? = 1;");
        assert_eq!(codes, vec!["NDO1001".to_string()]);
    }

    #[test]
    fn generic_and_plain_types_are_distinguished() {
        let (dump, errors, _) = parse_text("let a: Result<Int, Text> = x; let b: Int = y;");
        assert!(errors.is_empty(), "{dump}");
        assert_eq!(dump.matches("GenericType").count(), 1, "{dump}");
        // `Result<Int, Text>` holds two plain path types, and the second
        // binding adds a third.
        assert_eq!(dump.matches("PathType").count(), 3, "{dump}");
    }

    #[test]
    fn contextual_words_are_still_names() {
        // `agent`, `verify` and friends stay usable as names where their
        // construct cannot appear. At item position `agent` *does* start a
        // declaration, which is the whole point of the NEP-0005 rule.
        let inside = "fn f() {\n    let agent = 1;\n    let verify = 2;\n    let delegate = 3;\n    agent + verify\n}\n";
        assert_eq!(error_codes(inside), Vec::<String>::new());
        assert_eq!(
            error_codes("fn f(agent: Int) -> Int { agent }"),
            Vec::<String>::new()
        );
        assert_eq!(
            error_codes("let agent = 1;\nlet verify = 2;\nlet delegate = 3;"),
            Vec::<String>::new()
        );
        // At item position the same word starts a declaration, and a
        // declaration with no name is reported as one.
        let at_item_position = error_codes("agent + verify;");
        assert!(!at_item_position.is_empty());
    }

    #[test]
    fn the_agentic_expressions_parse() {
        let (dump, errors, _) = parse_text(
            "let draft: Generated<Article> = ask Writer { \"create\" };\n\
             let article: Verified<Article> = verify draft with ArticleVerifier;\n\
             let work = delegate Researcher { \"topic\" } with budget: Budget(tokens: 4_000);\n",
        );
        assert!(errors.is_empty(), "{dump}");
        assert!(dump.contains("AskExpression"), "{dump}");
        assert!(dump.contains("VerifyExpression"), "{dump}");
        assert!(dump.contains("DelegateExpression"), "{dump}");
    }

    #[test]
    fn a_verify_call_on_a_name_called_verify_is_a_call() {
        // Documented limitation: `verify(x)` is a call, not a verification.
        let (dump, errors, _) = parse_text("fn f() { verify(1) }");
        assert!(errors.is_empty(), "{dump}");
        assert!(dump.contains("CallExpression"), "{dump}");
        assert!(!dump.contains("VerifyExpression"), "{dump}");
    }

    #[test]
    fn statements_and_final_expressions_are_distinguished() {
        let (dump, errors, _) = parse_text("fn f() {\n    let a = 1;\n    g();\n    a\n}\n");
        assert!(errors.is_empty(), "{dump}");
        assert_eq!(dump.matches("ExpressionStatement").count(), 1, "{dump}");
        assert!(dump.contains("Block"), "{dump}");
    }

    #[test]
    fn if_else_and_match_parse() {
        let (dump, errors, _) = parse_text(
            "fn f(xs: Int) -> Text {\n    match xs {\n        Pending => \"p\"\n        Failed(reason) => reason\n    }\n}\n",
        );
        assert!(errors.is_empty(), "{dump}");
        assert_eq!(dump.matches("MatchArm").count(), 2, "{dump}");
        let codes = error_codes("fn f() { if a { 1 } else if b { 2 } else { 3 } }");
        assert_eq!(codes, Vec::<String>::new());
    }

    #[test]
    fn the_tree_can_reprint_the_file() {
        let source = "// a comment\nfn main() {\n    let x = 1;   /* trailing */\n}\n";
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", source);
        let file = sources.get(id).expect("just added");
        let parsed = parse(file);
        assert_eq!(parsed.tree().reprint(), source);
        assert_eq!(parsed.tree().validate(), Ok(()));
    }

    #[test]
    fn a_character_the_lexer_cannot_tokenise_is_still_in_the_tree() {
        let source = "fn main() {\n    let x = @;\n}\n";
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", source);
        let file = sources.get(id).expect("just added");
        let parsed = parse(file);
        assert_eq!(parsed.tree().reprint(), source);
        let codes: Vec<&str> = parsed
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code().id())
            .collect();
        // The lexer reports the character. The parser does not report it a
        // second time: one byte, one mistake, one diagnostic.
        assert_eq!(codes, vec!["NDO1002"]);
    }

    #[test]
    fn parsing_never_hangs_and_always_produces_a_tree() {
        for source in [
            "",
            "{",
            "}",
            ")))(((",
            "fn",
            "fn (",
            "let = ;",
            "struct",
            "match",
            "verify",
            "delegate",
            "a < b < c < d < e",
            "let x = (((((1)))));",
            "fn f() { @ }",
            "agent",
            "task T() -> { }",
            "tool",
            "model",
            "const",
            "fn f() -> { }",
            "let x = 1 + + + ;",
        ] {
            let mut sources = SourceMap::new();
            let id = sources.add("main.nudo", source);
            let file = sources.get(id).expect("just added");
            let parsed = parse(file);
            assert!(parsed.tree().is_lossless(), "dropped bytes for `{source}`");
            assert!(parsed.tree().token_count() > 0, "no tokens for `{source}`");
            assert!(
                parsed.diagnostics().len() <= MAX_ERRORS + 8,
                "unbounded diagnostics for `{source}`: {}",
                parsed.diagnostics().len()
            );
        }
    }

    #[test]
    fn diagnostics_are_bounded_on_nonsense() {
        let source = "} ".repeat(200);
        let codes = error_codes(&source);
        assert!(
            codes.len() <= MAX_ERRORS,
            "expected at most {MAX_ERRORS} diagnostics, got {}",
            codes.len()
        );
        assert!(!codes.is_empty());
    }

    #[test]
    fn one_mistake_produces_one_diagnostic() {
        // A missing `}` at the end of a file is one mistake, not one per token.
        for source in [
            "fn f() {\n    let x = 1;\n",
            "let x = ;",
            "fn f( { }",
            "struct S {",
            "match x {",
        ] {
            let codes = error_codes(source);
            assert!(
                codes.len() <= 2,
                "`{source}` produced {} diagnostics: {codes:?}",
                codes.len()
            );
        }
    }

    #[test]
    fn a_missing_token_is_never_synthesised() {
        // The tree only ever holds tokens the file contains.
        let (dump, _) = tree_of("fn f() { let x = 1 }");
        assert!(!dump.contains("Error"), "{dump}");
        let source = "fn f() { let x = 1 }";
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", source);
        let file = sources.get(id).expect("just added");
        let parsed = parse(file);
        assert_eq!(parsed.tree().reprint(), source);
    }

    #[test]
    fn tokens_the_parser_cannot_place_land_in_an_error_node() {
        let source = "fn f() {} ) ) ) fn g() {}";
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", source);
        let file = sources.get(id).expect("just added");
        let parsed = parse(file);
        assert_eq!(parsed.tree().reprint(), source);
        assert!(parsed.tree().root().has_errors());
        assert!(parsed.has_errors());
    }

    #[test]
    fn syntax_kinds_cover_every_lexical_token() {
        // The mapping is exhaustive, and trivia maps to trivia.
        assert_eq!(syntax_kind(TokenKind::Whitespace), SyntaxKind::Whitespace);
        assert_eq!(syntax_kind(TokenKind::Comment), SyntaxKind::Comment);
        assert_eq!(syntax_kind(TokenKind::Eof), SyntaxKind::Eof);
        assert_eq!(syntax_kind(TokenKind::Unknown), SyntaxKind::Unknown);
        assert_eq!(syntax_kind(TokenKind::FatArrow), SyntaxKind::FatArrow);
    }

    #[test]
    fn parse_exposes_its_parts() {
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", "fn f() {}");
        let file = sources.get(id).expect("just added");
        let parsed = parse(file);
        assert!(!parsed.has_errors());
        let (tree, diagnostics) = parsed.into_parts();
        assert_eq!(tree.root().kind(), SyntaxKind::SourceFile);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn dump_of_a_real_program_is_stable() {
        let (dump, errors, _) = parse_text("fn main() {}\n");
        assert!(errors.is_empty());
        let expected = "\
# nudo-tree v1
SourceFile 0..13
  FunctionDecl 0..12
    KeywordFn \"fn\" 0..2
    Whitespace \" \" 2..3
    Ident \"main\" 3..7
    ParameterList 7..9
      LParen \"(\" 7..8
      RParen \")\" 8..9
    Whitespace \" \" 9..10
    Block 10..12
      LBrace \"{\" 10..11
      RBrace \"}\" 11..12
  Whitespace \"\\n\" 12..13
  Eof \"\" 13..13
";
        assert_eq!(dump, expected);
    }

    #[test]
    fn helper_predicates_are_coherent() {
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", "agent R { role: \"x\" }");
        let file = sources.get(id).expect("just added");
        let parsed = parse(file);
        let agent: Option<SyntaxNode<'_>> = parsed
            .tree()
            .root()
            .child_nodes()
            .into_iter()
            .find(|node| node.kind() == SyntaxKind::AgentDecl);
        let agent = agent.expect("an agent declaration");
        assert!(agent.child_of_kind(SyntaxKind::RoleClause).is_some());
        assert!(agent.first_token().is_some());
    }
}
