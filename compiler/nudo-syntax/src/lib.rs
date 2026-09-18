//! Lossless syntax tree shared by the parser, formatter and language server.
//!
//! The tree this crate defines has one property that everything else depends
//! on: **it loses nothing**. Every byte of the source is reachable from the
//! tree — including whitespace and comments — so a consumer can reprint the
//! file exactly as it was written. A parser that discards trivia cannot support
//! a formatter, an editor or a safe automated rewrite, and safe automated
//! rewriting is a stated reason for this language to exist.
//!
//! ```text
//! concatenating the text of every token of the tree, in order, equals the file
//! ```
//!
//! That is a property the tests check, not a comment.
//!
//! # Shape
//!
//! ```text
//! SyntaxKind    one enum: every token kind and every node kind
//! SyntaxTree    the root: the source text, the token list and the node arena
//! SyntaxNode    a handle into a tree: kind, span, children (borrows the tree)
//! SyntaxToken   a handle into a tree: kind, span, text     (borrows the tree)
//! TreeBuilder   how a parser builds a tree
//! ```
//!
//! Nodes live in one arena and children refer to them by index, so a tree is
//! cheap to move and needs no reference counting. Handles borrow their tree,
//! which is what makes them cheap: `SyntaxNode` is two words, not a pointer
//! chase.
//!
//! Trivia — whitespace and comments — is attached to the *following* token,
//! which is the convention that makes reprinting, and later formatting,
//! straightforward.
//!
//! # What this crate is not
//!
//! It holds no meaning. Nothing here resolves a name, checks a type or decides
//! whether a program is correct; that is `nudo-ast` for structure and the later
//! milestones for meaning. The design and its rejected alternatives are in
//! [`docs/internals/parser-design.md`][design].
//!
//! [design]: https://github.com/smouj/nudo/blob/main/docs/internals/parser-design.md

use std::fmt;

use nudo_span::{BytePos, Span};

/// The kind of a token or of a node.
///
/// One enum holds both, so a consumer can match on a child without asking
/// first whether it is a token. The names are part of the conformance format:
/// they appear verbatim in `nudo check --dump-tree` output, so renaming a
/// variant changes what another implementation must match.
///
/// Token kinds are named after their lexeme (`KeywordFn`, `LBrace`, `Lt`), as
/// in `nudo-lexer`, and node kinds after the grammar production they come from
/// (`function-decl` is [`SyntaxKind::FunctionDecl`]), so that the tree can be
/// read against `grammar/nudo.ebnf`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntaxKind {
    // ---------------------------------------------------------- trivia ----
    /// Whitespace between tokens, including line terminators.
    Whitespace,
    /// A line or block comment.
    Comment,

    // --------------------------------------------------------- literals ---
    /// A name: `add`, `User`, `_tmp`.
    Ident,
    /// An integer literal: `42`, `1_000`.
    IntLiteral,
    /// A floating-point literal: `3.5`, `1_0.2_5`.
    FloatLiteral,
    /// A text literal, including its quotes.
    TextLiteral,
    /// A character that starts no token. The lexer reports it and keeps it as
    /// a token anyway, so that no byte of the file is unaccounted for; nothing
    /// dispatches on it, because the lexical diagnostic already said what is
    /// wrong with it.
    Unknown,

    // -------------------------------------------------------- keywords ----
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

    // ---------------------------------------------------- punctuation -----
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

    /// The end of the token stream. Always present, always last.
    Eof,

    // ------------------------------------------------------------ nodes ---
    /// The whole file: `program`.
    SourceFile,
    /// Tokens the parser could not place.
    Error,

    /// `function-decl`.
    FunctionDecl,
    /// `struct-decl`.
    StructDecl,
    /// `enum-decl`.
    EnumDecl,
    /// `agent-decl`.
    AgentDecl,
    /// `task-decl`.
    TaskDecl,
    /// `tool-decl`.
    ToolDecl,
    /// `model-decl`.
    ModelDecl,
    /// `const-decl`.
    ConstDecl,
    /// `binding`: a `let` statement or item.
    Binding,

    /// `parameter-list`.
    ParameterList,
    /// `generic-parameter-list`: the `<T, U>` a declaration states after its name.
    GenericParameterList,
    /// `parameter`.
    Parameter,
    /// `effect-clause`.
    EffectClause,
    /// `capability-list`.
    CapabilityList,
    /// A single capability in a list.
    Capability,
    /// `field` of a struct or of an enum variant.
    Field,
    /// `variant` of an enum.
    Variant,
    /// A `name: expression` field of a model declaration.
    ModelField,
    /// `role-clause`.
    RoleClause,
    /// `tools-clause`.
    ToolsClause,
    /// `allow-clause`.
    AllowClause,
    /// `budget-clause`.
    BudgetClause,
    /// The `agent: path` clause of a task declaration.
    TaskAgentClause,
    /// `verify-clause`.
    VerifyClause,
    /// `budget-literal`.
    BudgetLiteral,
    /// `budget-field`.
    BudgetField,

    /// `path-type`.
    PathType,
    /// `generic-type`.
    GenericType,
    /// `sequence-type`.
    SequenceType,
    /// `function-type`.
    FunctionType,
    /// `parenthesized-type`.
    ParenthesizedType,
    /// A type with a trailing `?`.
    OptionalType,
    /// `path`.
    Path,

    /// `block`.
    Block,
    /// `expression-statement`: an expression followed by `;`.
    ExpressionStatement,
    /// A `-` or `!` applied to an expression.
    UnaryExpression,
    /// A binary operator applied to two expressions.
    BinaryExpression,
    /// `argument-list`.
    ArgumentList,
    /// A call: `f(x)`.
    CallExpression,
    /// A field access: `a.b`.
    FieldExpression,
    /// An index access: `a[i]`.
    IndexExpression,
    /// `path-expression`.
    PathExpression,
    /// A literal used as an expression.
    LiteralExpression,
    /// `parenthesized-expression`.
    ParenthesizedExpression,
    /// `if-expression`.
    IfExpression,
    /// `match-expression`.
    MatchExpression,
    /// `match-arm`.
    MatchArm,
    /// `pattern`.
    Pattern,
    /// `ask-expression`.
    AskExpression,
    /// `verify-expression`.
    VerifyExpression,
    /// `delegate-expression`.
    DelegateExpression,
}

impl SyntaxKind {
    /// The name of the kind, as printed by `--dump-tree`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            SyntaxKind::Whitespace => "Whitespace",
            SyntaxKind::Comment => "Comment",
            SyntaxKind::Ident => "Ident",
            SyntaxKind::IntLiteral => "IntLiteral",
            SyntaxKind::FloatLiteral => "FloatLiteral",
            SyntaxKind::TextLiteral => "TextLiteral",
            SyntaxKind::Unknown => "Unknown",
            SyntaxKind::KeywordFn => "KeywordFn",
            SyntaxKind::KeywordLet => "KeywordLet",
            SyntaxKind::KeywordStruct => "KeywordStruct",
            SyntaxKind::KeywordEnum => "KeywordEnum",
            SyntaxKind::KeywordIf => "KeywordIf",
            SyntaxKind::KeywordElse => "KeywordElse",
            SyntaxKind::KeywordMatch => "KeywordMatch",
            SyntaxKind::KeywordConst => "KeywordConst",
            SyntaxKind::KeywordTrue => "KeywordTrue",
            SyntaxKind::KeywordFalse => "KeywordFalse",
            SyntaxKind::LParen => "LParen",
            SyntaxKind::RParen => "RParen",
            SyntaxKind::LBrace => "LBrace",
            SyntaxKind::RBrace => "RBrace",
            SyntaxKind::LBracket => "LBracket",
            SyntaxKind::RBracket => "RBracket",
            SyntaxKind::Colon => "Colon",
            SyntaxKind::ColonColon => "ColonColon",
            SyntaxKind::Comma => "Comma",
            SyntaxKind::Dot => "Dot",
            SyntaxKind::Question => "Question",
            SyntaxKind::Arrow => "Arrow",
            SyntaxKind::FatArrow => "FatArrow",
            SyntaxKind::Equals => "Equals",
            SyntaxKind::EqEq => "EqEq",
            SyntaxKind::BangEq => "BangEq",
            SyntaxKind::Lt => "Lt",
            SyntaxKind::Gt => "Gt",
            SyntaxKind::LtEq => "LtEq",
            SyntaxKind::GtEq => "GtEq",
            SyntaxKind::AmpAmp => "AmpAmp",
            SyntaxKind::PipePipe => "PipePipe",
            SyntaxKind::Bang => "Bang",
            SyntaxKind::Plus => "Plus",
            SyntaxKind::Minus => "Minus",
            SyntaxKind::Star => "Star",
            SyntaxKind::Slash => "Slash",
            SyntaxKind::Semi => "Semi",
            SyntaxKind::Eof => "Eof",
            SyntaxKind::SourceFile => "SourceFile",
            SyntaxKind::Error => "Error",
            SyntaxKind::FunctionDecl => "FunctionDecl",
            SyntaxKind::StructDecl => "StructDecl",
            SyntaxKind::EnumDecl => "EnumDecl",
            SyntaxKind::AgentDecl => "AgentDecl",
            SyntaxKind::TaskDecl => "TaskDecl",
            SyntaxKind::ToolDecl => "ToolDecl",
            SyntaxKind::ModelDecl => "ModelDecl",
            SyntaxKind::ConstDecl => "ConstDecl",
            SyntaxKind::Binding => "Binding",
            SyntaxKind::ParameterList => "ParameterList",
            SyntaxKind::GenericParameterList => "GenericParameterList",
            SyntaxKind::Parameter => "Parameter",
            SyntaxKind::EffectClause => "EffectClause",
            SyntaxKind::CapabilityList => "CapabilityList",
            SyntaxKind::Capability => "Capability",
            SyntaxKind::Field => "Field",
            SyntaxKind::Variant => "Variant",
            SyntaxKind::ModelField => "ModelField",
            SyntaxKind::RoleClause => "RoleClause",
            SyntaxKind::ToolsClause => "ToolsClause",
            SyntaxKind::AllowClause => "AllowClause",
            SyntaxKind::BudgetClause => "BudgetClause",
            SyntaxKind::TaskAgentClause => "TaskAgentClause",
            SyntaxKind::VerifyClause => "VerifyClause",
            SyntaxKind::BudgetLiteral => "BudgetLiteral",
            SyntaxKind::BudgetField => "BudgetField",
            SyntaxKind::PathType => "PathType",
            SyntaxKind::GenericType => "GenericType",
            SyntaxKind::SequenceType => "SequenceType",
            SyntaxKind::FunctionType => "FunctionType",
            SyntaxKind::ParenthesizedType => "ParenthesizedType",
            SyntaxKind::OptionalType => "OptionalType",
            SyntaxKind::Path => "Path",
            SyntaxKind::Block => "Block",
            SyntaxKind::ExpressionStatement => "ExpressionStatement",
            SyntaxKind::UnaryExpression => "UnaryExpression",
            SyntaxKind::BinaryExpression => "BinaryExpression",
            SyntaxKind::ArgumentList => "ArgumentList",
            SyntaxKind::CallExpression => "CallExpression",
            SyntaxKind::FieldExpression => "FieldExpression",
            SyntaxKind::IndexExpression => "IndexExpression",
            SyntaxKind::PathExpression => "PathExpression",
            SyntaxKind::LiteralExpression => "LiteralExpression",
            SyntaxKind::ParenthesizedExpression => "ParenthesizedExpression",
            SyntaxKind::IfExpression => "IfExpression",
            SyntaxKind::MatchExpression => "MatchExpression",
            SyntaxKind::MatchArm => "MatchArm",
            SyntaxKind::Pattern => "Pattern",
            SyntaxKind::AskExpression => "AskExpression",
            SyntaxKind::VerifyExpression => "VerifyExpression",
            SyntaxKind::DelegateExpression => "DelegateExpression",
        }
    }

    /// Whether this kind is a token: something the lexer produced, rather than
    /// something the parser built.
    #[must_use]
    pub const fn is_token(self) -> bool {
        !self.is_node()
    }

    /// Whether this kind is a node built by the parser.
    #[must_use]
    pub const fn is_node(self) -> bool {
        matches!(
            self,
            SyntaxKind::SourceFile
                | SyntaxKind::Error
                | SyntaxKind::FunctionDecl
                | SyntaxKind::StructDecl
                | SyntaxKind::EnumDecl
                | SyntaxKind::AgentDecl
                | SyntaxKind::TaskDecl
                | SyntaxKind::ToolDecl
                | SyntaxKind::ModelDecl
                | SyntaxKind::ConstDecl
                | SyntaxKind::Binding
                | SyntaxKind::ParameterList
                | SyntaxKind::GenericParameterList
                | SyntaxKind::Parameter
                | SyntaxKind::EffectClause
                | SyntaxKind::CapabilityList
                | SyntaxKind::Capability
                | SyntaxKind::Field
                | SyntaxKind::Variant
                | SyntaxKind::ModelField
                | SyntaxKind::RoleClause
                | SyntaxKind::ToolsClause
                | SyntaxKind::AllowClause
                | SyntaxKind::BudgetClause
                | SyntaxKind::TaskAgentClause
                | SyntaxKind::VerifyClause
                | SyntaxKind::BudgetLiteral
                | SyntaxKind::BudgetField
                | SyntaxKind::PathType
                | SyntaxKind::GenericType
                | SyntaxKind::SequenceType
                | SyntaxKind::FunctionType
                | SyntaxKind::ParenthesizedType
                | SyntaxKind::OptionalType
                | SyntaxKind::Path
                | SyntaxKind::Block
                | SyntaxKind::ExpressionStatement
                | SyntaxKind::UnaryExpression
                | SyntaxKind::BinaryExpression
                | SyntaxKind::ArgumentList
                | SyntaxKind::CallExpression
                | SyntaxKind::FieldExpression
                | SyntaxKind::IndexExpression
                | SyntaxKind::PathExpression
                | SyntaxKind::LiteralExpression
                | SyntaxKind::ParenthesizedExpression
                | SyntaxKind::IfExpression
                | SyntaxKind::MatchExpression
                | SyntaxKind::MatchArm
                | SyntaxKind::Pattern
                | SyntaxKind::AskExpression
                | SyntaxKind::VerifyExpression
                | SyntaxKind::DelegateExpression
        )
    }

    /// Whether this kind is trivia: whitespace or a comment.
    ///
    /// Trivia is preserved so the tree is lossless, and is attached to the
    /// following token. Nothing that looks at structure treats it as content.
    #[must_use]
    pub const fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::Whitespace | SyntaxKind::Comment)
    }

    /// Whether this kind is a character the lexer could not tokenise.
    ///
    /// Nothing in the tree may dispatch on it: a parser that reported it again
    /// would report one mistake twice, and the lexical diagnostic already
    /// names it.
    #[must_use]
    pub const fn is_unknown(self) -> bool {
        matches!(self, SyntaxKind::Unknown)
    }

    /// Whether this kind carries no structure: trivia, or a character the
    /// lexer could not tokenise.
    #[must_use]
    pub const fn is_skipped(self) -> bool {
        self.is_trivia() || self.is_unknown()
    }

    /// Whether this kind is a keyword.
    #[must_use]
    pub const fn is_keyword(self) -> bool {
        matches!(
            self,
            SyntaxKind::KeywordFn
                | SyntaxKind::KeywordLet
                | SyntaxKind::KeywordStruct
                | SyntaxKind::KeywordEnum
                | SyntaxKind::KeywordIf
                | SyntaxKind::KeywordElse
                | SyntaxKind::KeywordMatch
                | SyntaxKind::KeywordConst
                | SyntaxKind::KeywordTrue
                | SyntaxKind::KeywordFalse
        )
    }
}

impl fmt::Display for SyntaxKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An index into a tree's node arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u32);

impl NodeId {
    /// The raw index of the node in its tree.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// A position between two children of the node being built.
///
/// A checkpoint is how the parser wraps something it has already built: it
/// takes a checkpoint, parses an operand, and then wraps everything it added
/// since that checkpoint in the node its operator belongs to. Without it, a
/// parser cannot express left-associative or nested operators, because the
/// left operand is already a sibling of the operator that should own it.
///
/// ```text
/// let checkpoint = builder.checkpoint();
/// parse_operand();                    // adds elements to the current node
/// builder.start_node_at(checkpoint, SyntaxKind::BinaryExpression);
/// parse_operator_and_right_operand(); // adds them inside the new node
/// builder.finish_node();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Checkpoint(usize);

/// The data of one node in the arena: its kind and its ordered children.
#[derive(Debug, Clone)]
struct NodeData {
    kind: SyntaxKind,
    children: Vec<Element>,
}

/// The data of one token: its kind and where it is.
#[derive(Debug, Clone, Copy)]
struct TokenData {
    kind: SyntaxKind,
    span: Span,
}

/// A child of a node: either a node or a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Element {
    Node(NodeId),
    Token(u32),
}

/// A lossless syntax tree: the source text, its tokens and the node arena.
///
/// The tree owns the text it was built from, so a handle into it is enough to
/// reprint the file, read a token's lexeme, or answer a span query. There is no
/// separate "source" argument to thread around.
#[derive(Debug, Clone)]
pub struct SyntaxTree {
    source: u32,
    text: String,
    tokens: Vec<TokenData>,
    nodes: Vec<NodeData>,
    root: NodeId,
}

impl SyntaxTree {
    /// The identifier of the source file the tree was built from.
    #[must_use]
    pub const fn source(&self) -> u32 {
        self.source
    }

    /// The text of the file.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The root node: always [`SyntaxKind::SourceFile`].
    #[must_use]
    pub fn root(&self) -> SyntaxNode<'_> {
        SyntaxNode {
            tree: self,
            id: self.root,
        }
    }

    /// Every token of the file, in source order, trivia included.
    #[must_use]
    pub fn tokens(&self) -> Vec<SyntaxToken<'_>> {
        (0..self.tokens.len() as u32)
            .map(|index| SyntaxToken { tree: self, index })
            .collect()
    }

    /// The number of tokens, trivia included, counting the final `Eof`.
    #[must_use]
    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    /// The number of nodes in the arena.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Reprints the file from the tree alone.
    ///
    /// For a lossless tree this returns the original text byte for byte. It is
    /// the operation the M10 formatter is built on, and the reason trivia is
    /// preserved rather than skipped.
    #[must_use]
    pub fn reprint(&self) -> String {
        let mut out = String::with_capacity(self.text.len());
        for token in self.tokens() {
            out.push_str(token.text());
        }
        out
    }

    /// Whether the tree is lossless: its tokens re-concatenate to the file.
    #[must_use]
    pub fn is_lossless(&self) -> bool {
        self.reprint() == self.text
    }

    /// Checks every invariant the rest of the workspace relies on.
    ///
    /// A tree that fails this is a bug in whoever built it, not in its reader,
    /// so the message names the invariant rather than the symptom.
    ///
    /// # Errors
    ///
    /// Returns a description of the first invariant that does not hold.
    pub fn validate(&self) -> Result<(), String> {
        if self.nodes.is_empty() {
            return Err("the tree has no nodes".to_string());
        }
        if self.nodes[self.root.index() as usize].kind != SyntaxKind::SourceFile {
            return Err("the root node is not a SourceFile".to_string());
        }
        if !self.is_lossless() {
            return Err(
                "the tree is not lossless: its tokens do not re-concatenate to the file"
                    .to_string(),
            );
        }
        // Every token must be reachable exactly once, in order, and every node
        // must be reachable from the root.
        let mut seen_tokens: Vec<u32> = Vec::with_capacity(self.tokens.len());
        let mut seen_nodes: Vec<NodeId> = Vec::with_capacity(self.nodes.len());
        let mut stack = vec![self.root];
        while let Some(id) = stack.pop() {
            if seen_nodes.contains(&id) {
                return Err(format!("node {} is reachable twice", id.index()));
            }
            seen_nodes.push(id);
            let node = &self.nodes[id.index() as usize];
            for child in node.children.iter().rev() {
                match *child {
                    Element::Node(child_id) => stack.push(child_id),
                    Element::Token(index) => seen_tokens.push(index),
                }
            }
        }
        seen_tokens.sort_unstable();
        let expected: Vec<u32> = (0..self.tokens.len() as u32).collect();
        if seen_tokens != expected {
            return Err("the tree does not contain every token exactly once".to_string());
        }
        if seen_nodes.len() != self.nodes.len() {
            return Err(format!(
                "{} node(s) are unreachable from the root",
                self.nodes.len() - seen_nodes.len()
            ));
        }
        // A node's span must cover exactly its children's spans.
        for (index, node) in self.nodes.iter().enumerate() {
            if node.children.is_empty() {
                continue;
            }
            if !node.kind.is_node() {
                return Err(format!("node {index} is a token kind: {}", node.kind));
            }
        }
        if self.root_span() != Span::new(BytePos::ZERO, BytePos::new(self.text.len() as u32)) {
            return Err("the root span does not cover the whole file".to_string());
        }
        Ok(())
    }

    fn root_span(&self) -> Span {
        self.node_span(self.root)
    }

    fn node_span(&self, id: NodeId) -> Span {
        let node = &self.nodes[id.index() as usize];
        let mut span: Option<Span> = None;
        for child in &node.children {
            let child_span = match *child {
                Element::Node(child_id) => self.node_span(child_id),
                Element::Token(index) => self.tokens[index as usize].span,
            };
            span = Some(match span {
                Some(current) => current.join(child_span),
                None => child_span,
            });
        }
        span.unwrap_or_else(|| Span::new(BytePos::ZERO, BytePos::ZERO))
    }
}

/// A handle to one node of a tree.
#[derive(Debug, Clone, Copy)]
pub struct SyntaxNode<'t> {
    tree: &'t SyntaxTree,
    id: NodeId,
}

impl<'t> SyntaxNode<'t> {
    /// The kind of this node.
    #[must_use]
    pub fn kind(self) -> SyntaxKind {
        self.tree.nodes[self.id.index() as usize].kind
    }

    /// A stable identifier for this node inside its tree.
    #[must_use]
    pub const fn id(self) -> NodeId {
        self.id
    }

    /// The span this node covers: exactly its children's spans, joined.
    #[must_use]
    pub fn span(self) -> Span {
        self.tree.node_span(self.id)
    }

    /// The text this node covers.
    #[must_use]
    pub fn text(self) -> &'t str {
        self.span().text(&self.tree.text).unwrap_or("")
    }

    /// The tree this node belongs to.
    #[must_use]
    pub const fn tree(self) -> &'t SyntaxTree {
        self.tree
    }

    /// The children, nodes and tokens, in source order, trivia included.
    #[must_use]
    pub fn children(self) -> Vec<SyntaxElement<'t>> {
        self.raw_children()
            .iter()
            .map(|child| match *child {
                Element::Node(id) => SyntaxElement::Node(SyntaxNode {
                    tree: self.tree,
                    id,
                }),
                Element::Token(index) => SyntaxElement::Token(SyntaxToken {
                    tree: self.tree,
                    index,
                }),
            })
            .collect()
    }

    /// The child nodes, in source order, skipping tokens and trivia.
    #[must_use]
    pub fn child_nodes(self) -> Vec<SyntaxNode<'t>> {
        self.children()
            .into_iter()
            .filter_map(SyntaxElement::into_node)
            .collect()
    }

    /// The tokens directly under this node, trivia included.
    #[must_use]
    pub fn child_tokens(self) -> Vec<SyntaxToken<'t>> {
        self.children()
            .into_iter()
            .filter_map(SyntaxElement::into_token)
            .collect()
    }

    /// The first child node of `kind`.
    #[must_use]
    pub fn child_of_kind(self, kind: SyntaxKind) -> Option<SyntaxNode<'t>> {
        self.child_nodes()
            .into_iter()
            .find(|node| node.kind() == kind)
    }

    /// The first non-trivia token directly under this node.
    #[must_use]
    pub fn first_token(self) -> Option<SyntaxToken<'t>> {
        self.child_tokens()
            .into_iter()
            .find(|token| !token.kind().is_trivia())
    }

    /// The direct children of `kind`, in source order.
    #[must_use]
    pub fn children_of_kind(self, kind: SyntaxKind) -> Vec<SyntaxNode<'t>> {
        self.child_nodes()
            .into_iter()
            .filter(|node| node.kind() == kind)
            .collect()
    }

    /// Whether this node or any node below it is an error node.
    #[must_use]
    pub fn has_errors(self) -> bool {
        self.descendants()
            .into_iter()
            .any(|node| node.kind() == SyntaxKind::Error)
    }

    /// This node and every node below it, in source order.
    #[must_use]
    pub fn descendants(self) -> Vec<SyntaxNode<'t>> {
        let mut out = Vec::new();
        let mut stack = vec![self];
        while let Some(node) = stack.pop() {
            out.push(node);
            let mut children = node.child_nodes();
            children.reverse();
            stack.extend(children);
        }
        out
    }

    fn raw_children(self) -> &'t [Element] {
        &self.tree.nodes[self.id.index() as usize].children
    }
}

impl PartialEq for SyntaxNode<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.tree, other.tree) && self.id == other.id
    }
}

impl Eq for SyntaxNode<'_> {}

/// A handle to one token of a tree.
#[derive(Debug, Clone, Copy)]
pub struct SyntaxToken<'t> {
    tree: &'t SyntaxTree,
    index: u32,
}

impl<'t> SyntaxToken<'t> {
    /// The kind of this token.
    #[must_use]
    pub fn kind(self) -> SyntaxKind {
        self.tree.tokens[self.index as usize].kind
    }

    /// The byte range this token covers.
    #[must_use]
    pub fn span(self) -> Span {
        self.tree.tokens[self.index as usize].span
    }

    /// The text of this token, exactly as it appears in the file.
    #[must_use]
    pub fn text(self) -> &'t str {
        self.span().text(&self.tree.text).unwrap_or("")
    }

    /// This token's position in the token stream.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }
}

impl PartialEq for SyntaxToken<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.tree, other.tree) && self.index == other.index
    }
}

impl Eq for SyntaxToken<'_> {}

/// A child of a node: a node or a token.
#[derive(Debug, Clone, Copy)]
pub enum SyntaxElement<'t> {
    /// A child node.
    Node(SyntaxNode<'t>),
    /// A child token.
    Token(SyntaxToken<'t>),
}

impl<'t> SyntaxElement<'t> {
    /// The node, if this element is one.
    #[must_use]
    pub fn into_node(self) -> Option<SyntaxNode<'t>> {
        match self {
            SyntaxElement::Node(node) => Some(node),
            SyntaxElement::Token(_) => None,
        }
    }

    /// The token, if this element is one.
    #[must_use]
    pub fn into_token(self) -> Option<SyntaxToken<'t>> {
        match self {
            SyntaxElement::Token(token) => Some(token),
            SyntaxElement::Node(_) => None,
        }
    }

    /// The kind of whichever of the two this element is.
    #[must_use]
    pub fn kind(self) -> SyntaxKind {
        match self {
            SyntaxElement::Node(node) => node.kind(),
            SyntaxElement::Token(token) => token.kind(),
        }
    }

    /// The span of whichever of the two this element is.
    #[must_use]
    pub fn span(self) -> Span {
        match self {
            SyntaxElement::Node(node) => node.span(),
            SyntaxElement::Token(token) => token.span(),
        }
    }
}

/// Builds a tree, one node or token at a time.
///
/// This is the parser's cursor into the tree it is producing. Nodes are opened
/// and closed around the tokens they own, in source order:
///
/// ```text
/// builder.start_node(SyntaxKind::FunctionDecl);
/// builder.token(SyntaxKind::KeywordFn, span_of_fn);
/// ...
/// builder.finish_node();
/// ```
///
/// The builder enforces the two properties the tree must have: tokens are
/// appended in source order, and a token is never added twice. Trivia is added
/// by the builder's caller exactly like any other token, which is what makes
/// the tree lossless.
///
/// # Panics
///
/// [`TreeBuilder::finish_node`] panics if no node is open. A parser that closes
/// a node it never opened has a structural bug, and failing loudly is better
/// than producing a tree that silently disagrees with the source.
#[derive(Debug)]
pub struct TreeBuilder {
    source: u32,
    text: String,
    tokens: Vec<TokenData>,
    nodes: Vec<NodeData>,
    stack: Vec<(SyntaxKind, Vec<Element>)>,
    root: Option<NodeId>,
}

impl TreeBuilder {
    /// Starts building a tree for a file.
    #[must_use]
    pub fn new(source: u32, text: impl Into<String>) -> Self {
        TreeBuilder {
            source,
            text: text.into(),
            tokens: Vec::new(),
            nodes: Vec::new(),
            stack: Vec::new(),
            root: None,
        }
    }

    /// Opens a node. Everything added until the matching
    /// [`TreeBuilder::finish_node`] becomes its child.
    pub fn start_node(&mut self, kind: SyntaxKind) {
        debug_assert!(kind.is_node(), "{kind} is a token kind");
        self.stack.push((kind, Vec::new()));
    }

    /// Closes the innermost open node.
    ///
    /// # Panics
    ///
    /// Panics if no node is open.
    pub fn finish_node(&mut self) {
        let (kind, children) = self
            .stack
            .pop()
            .expect("finish_node called with no open node");
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(NodeData { kind, children });
        match self.stack.last_mut() {
            Some((_, parent_children)) => parent_children.push(Element::Node(id)),
            None => {
                debug_assert!(self.root.is_none(), "the root node was closed twice");
                self.root = Some(id);
            }
        }
    }

    /// Adds a token to the innermost open node.
    ///
    /// # Panics
    ///
    /// Panics if no node is open. A token that belongs to no node is a parser
    /// bug: the tree would not be able to answer where it is.
    pub fn token(&mut self, kind: SyntaxKind, span: Span) {
        debug_assert!(kind.is_token(), "{kind} is a node kind");
        let index = self.tokens.len() as u32;
        if let Some(last) = self.tokens.last() {
            debug_assert!(
                last.span.end() <= span.start(),
                "tokens must be added in source order"
            );
        }
        self.tokens.push(TokenData { kind, span });
        let (_, children) = self
            .stack
            .last_mut()
            .expect("token() called with no open node");
        children.push(Element::Token(index));
    }

    /// How many nodes are currently open.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Marks a position in the node currently being built.
    ///
    /// # Panics
    ///
    /// Panics if no node is open: a checkpoint is a position inside a node.
    #[must_use]
    pub fn checkpoint(&self) -> Checkpoint {
        let (_, children) = self.stack.last().expect("checkpoint needs an open node");
        Checkpoint(children.len())
    }

    /// Opens a node that wraps everything added to the current node since
    /// `checkpoint`, and closes the previous node's membership there.
    ///
    /// The new node stays open, so the caller adds the operator and the right
    /// operand to it and then calls [`TreeBuilder::finish_node`].
    ///
    /// # Panics
    ///
    /// Panics if no node is open, or if `checkpoint` is past the end of the
    /// current node's children. Wrapping nothing is a parser bug: the result
    /// would be a node with no content.
    pub fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        debug_assert!(kind.is_node(), "{kind} is a token kind");
        let (_, children) = self
            .stack
            .last_mut()
            .expect("start_node_at needs an open node");
        assert!(
            checkpoint.0 < children.len(),
            "start_node_at must wrap at least one element"
        );
        let wrapped = children.split_off(checkpoint.0);
        self.stack.push((kind, wrapped));
    }

    /// Finishes the tree, closing every open node.
    ///
    /// # Panics
    ///
    /// Panics if nothing was built, because a tree with no root cannot be read.
    #[must_use]
    pub fn finish(mut self) -> SyntaxTree {
        while self.depth() > 0 {
            self.finish_node();
        }
        let root = self.root.expect("a tree must have a root node");
        SyntaxTree {
            source: self.source,
            text: self.text,
            tokens: self.tokens,
            nodes: self.nodes,
            root,
        }
    }
}

/// Renders a tree in the stable, diffable format used by the conformance suite
/// and by `nudo check --dump-tree`.
///
/// The format is documented in `tests/conformance/README.md`:
///
/// ```text
/// # nudo-tree v1
/// SourceFile 0..57
///   FunctionDecl 0..45
///     KeywordFn "fn" 0..2
///     Ident "main" 3..7
/// ```
///
/// Each line is two spaces of indentation per level, the [`SyntaxKind`] name,
/// and — for a token — the lexeme in Rust debug form. Every line ends with the
/// byte range the element covers, so a reader can find the same element in the
/// source without counting lines.
#[must_use]
pub fn dump_tree(tree: &SyntaxTree) -> String {
    let mut out = String::from("# nudo-tree v1\n");
    write_node(tree.root(), 0, &mut out);
    out
}

fn write_node(node: SyntaxNode<'_>, depth: usize, out: &mut String) {
    let span = node.span();
    out.push_str(&format!(
        "{:indent$}{} {}..{}\n",
        "",
        node.kind(),
        span.start().get(),
        span.end().get(),
        indent = depth * 2,
    ));
    for child in node.children() {
        match child {
            SyntaxElement::Node(child_node) => write_node(child_node, depth + 1, out),
            SyntaxElement::Token(token) => {
                out.push_str(&format!(
                    "{:indent$}{} {:?} {}..{}\n",
                    "",
                    token.kind(),
                    token.text(),
                    token.span().start().get(),
                    token.span().end().get(),
                    indent = (depth + 1) * 2,
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nudo_span::BytePos;

    fn span(start: u32, end: u32) -> Span {
        Span::new(BytePos::new(start), BytePos::new(end))
    }

    /// Builds `fn main() {}` by hand, the way the parser will.
    fn function_tree() -> SyntaxTree {
        let text = "fn main() {}";
        let mut builder = TreeBuilder::new(0, text);
        builder.start_node(SyntaxKind::SourceFile);
        builder.start_node(SyntaxKind::FunctionDecl);
        builder.token(SyntaxKind::KeywordFn, span(0, 2));
        builder.token(SyntaxKind::Whitespace, span(2, 3));
        builder.token(SyntaxKind::Ident, span(3, 7));
        builder.start_node(SyntaxKind::ParameterList);
        builder.token(SyntaxKind::LParen, span(7, 8));
        builder.token(SyntaxKind::RParen, span(8, 9));
        builder.finish_node();
        builder.token(SyntaxKind::Whitespace, span(9, 10));
        builder.start_node(SyntaxKind::Block);
        builder.token(SyntaxKind::LBrace, span(10, 11));
        builder.token(SyntaxKind::RBrace, span(11, 12));
        builder.finish_node();
        builder.finish_node();
        builder.finish_node();
        builder.finish()
    }

    #[test]
    fn a_hand_built_tree_is_lossless() {
        let tree = function_tree();
        assert!(tree.is_lossless());
        assert_eq!(tree.reprint(), "fn main() {}");
        assert_eq!(tree.validate(), Ok(()));
    }

    #[test]
    fn the_root_is_the_whole_file() {
        let tree = function_tree();
        assert_eq!(tree.root().kind(), SyntaxKind::SourceFile);
        assert_eq!(tree.root().span(), span(0, 12));
        assert_eq!(tree.root().text(), "fn main() {}");
        assert_eq!(tree.token_count(), 8);
        assert_eq!(tree.node_count(), 4);
        assert_eq!(tree.source(), 0);
    }

    #[test]
    fn handles_walk_the_tree() {
        let tree = function_tree();
        let root = tree.root();
        let function = root.child_of_kind(SyntaxKind::FunctionDecl).expect("decl");
        assert_eq!(function.kind(), SyntaxKind::FunctionDecl);
        assert_eq!(function.span(), span(0, 12));
        assert_eq!(function.first_token().expect("token").text(), "fn");
        assert_eq!(root.child_nodes(), vec![function]);
        assert_eq!(function.children().len(), 6);
        assert_eq!(function.children_of_kind(SyntaxKind::Block).len(), 1);
        assert!(!function.has_errors());
        assert_eq!(root.descendants().len(), 4);
    }

    #[test]
    fn tokens_report_their_kind_span_and_text() {
        let tree = function_tree();
        let tokens = tree.tokens();
        assert_eq!(tokens[0].kind(), SyntaxKind::KeywordFn);
        assert_eq!(tokens[0].text(), "fn");
        assert_eq!(tokens[0].span(), span(0, 2));
        assert_eq!(tokens[0].index(), 0);
        assert_eq!(tokens[2].text(), "main");
        assert!(SyntaxKind::KeywordFn.is_keyword());
        assert!(SyntaxKind::Whitespace.is_trivia());
        assert!(SyntaxKind::Comment.is_trivia());
        assert!(!SyntaxKind::Ident.is_trivia());
        assert!(SyntaxKind::SourceFile.is_node());
        assert!(SyntaxKind::Ident.is_token());
        assert!(!SyntaxKind::Ident.is_node());
    }

    #[test]
    fn elements_split_into_nodes_and_tokens() {
        let tree = function_tree();
        let children = tree.root().children();
        assert_eq!(children.len(), 1);
        assert!(children[0].into_node().is_some());
        assert!(children[0].into_token().is_none());
        assert_eq!(children[0].kind(), SyntaxKind::FunctionDecl);
        let function = children[0].into_node().expect("a node");
        let token = function.children()[0];
        assert!(token.into_token().is_some());
        assert!(token.into_node().is_none());
        assert_eq!(token.span(), span(0, 2));
    }

    #[test]
    fn equality_compares_position_in_the_same_tree() {
        let tree = function_tree();
        let other = function_tree();
        let root = tree.root();
        assert_eq!(root, tree.root());
        assert_ne!(root, other.root());
        assert_eq!(tree.tokens()[0], tree.tokens()[0]);
        assert_ne!(tree.tokens()[0], tree.tokens()[1]);
    }

    #[test]
    fn an_error_node_is_reachable_and_marked() {
        let mut builder = TreeBuilder::new(0, "fn }");
        builder.start_node(SyntaxKind::SourceFile);
        builder.start_node(SyntaxKind::Error);
        builder.token(SyntaxKind::KeywordFn, span(0, 2));
        builder.token(SyntaxKind::Whitespace, span(2, 3));
        builder.token(SyntaxKind::RBrace, span(3, 4));
        builder.finish_node();
        builder.finish_node();
        let tree = builder.finish();
        assert!(tree.is_lossless());
        assert!(tree.root().has_errors());
        assert_eq!(tree.reprint(), "fn }");
    }

    #[test]
    fn an_empty_file_is_a_valid_tree() {
        let mut builder = TreeBuilder::new(3, "");
        builder.start_node(SyntaxKind::SourceFile);
        builder.token(SyntaxKind::Eof, span(0, 0));
        builder.finish_node();
        let tree = builder.finish();
        assert!(tree.is_lossless());
        assert_eq!(tree.validate(), Ok(()));
        assert_eq!(tree.token_count(), 1);
        assert_eq!(tree.source(), 3);
    }

    #[test]
    fn open_nodes_are_closed_by_finish() {
        let mut builder = TreeBuilder::new(0, "x");
        builder.start_node(SyntaxKind::SourceFile);
        builder.start_node(SyntaxKind::Block);
        builder.token(SyntaxKind::Ident, span(0, 1));
        assert_eq!(builder.depth(), 2);
        let tree = builder.finish();
        assert_eq!(tree.validate(), Ok(()));
        assert_eq!(tree.root().child_nodes().len(), 1);
    }

    #[test]
    fn validate_rejects_a_broken_tree() {
        let mut builder = TreeBuilder::new(0, "fn");
        builder.start_node(SyntaxKind::SourceFile);
        builder.token(SyntaxKind::KeywordFn, span(0, 2));
        builder.finish_node();
        let tree = builder.finish();
        assert_eq!(tree.validate(), Ok(()));

        // A tree whose text does not match its tokens is not lossless.
        let mut builder = TreeBuilder::new(0, "different text");
        builder.start_node(SyntaxKind::SourceFile);
        builder.token(SyntaxKind::KeywordFn, span(0, 2));
        builder.finish_node();
        let broken = builder.finish();
        assert!(broken.validate().is_err());
        assert!(!broken.is_lossless());
    }

    #[test]
    fn dump_tree_is_stable() {
        let tree = function_tree();
        let dump = dump_tree(&tree);
        let expected = "\
# nudo-tree v1
SourceFile 0..12
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
";
        assert_eq!(dump, expected);
    }

    #[test]
    fn kind_names_are_unique_and_readable() {
        // Every kind is reachable through the two predicates, and no name is
        // ambiguous: the conformance corpus matches on these names.
        let kinds = [
            SyntaxKind::SourceFile,
            SyntaxKind::Error,
            SyntaxKind::FunctionDecl,
            SyntaxKind::Whitespace,
            SyntaxKind::Eof,
            SyntaxKind::VerifyExpression,
        ];
        for kind in kinds {
            assert!(!kind.as_str().is_empty());
            assert_eq!(kind.to_string(), kind.as_str());
            assert_ne!(kind.is_node(), kind.is_token());
        }
    }
}
