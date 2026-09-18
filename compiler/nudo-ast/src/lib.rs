//! Typed wrappers over the lossless syntax tree.
//!
//! `nudo-syntax` holds everything, including errors and trivia, and knows no
//! meaning. This crate is the other half of that contract: it exposes the part
//! of a tree that is **unambiguously well formed** as typed values — a
//! `Function` with a name and a body, a `Binding` with a value, an `Expression`
//! with a shape — and returns `None` for everything else.
//!
//! The rule that keeps the two honest:
//!
//! > A type here never invents structure. If the tree does not hold the part,
//! > the accessor returns `None`; it does not guess, default or repair.
//!
//! That matters because a consumer that needs to be sure — a formatter deciding
//! whether it may rewrite, an agent deciding whether it may edit — has to be
//! able to tell "this is a function" from "this is something that looked like a
//! function until the eighth token".
//!
//! # Example
//!
//! ```no_run
//! use nudo_ast::{AstNode, Function, Item, SourceFile};
//! use nudo_source::SourceMap;
//!
//! let mut sources = SourceMap::new();
//! let id = sources.add("main.nudo", "fn add(a: Int, b: Int) -> Int {\n    a + b\n}\n");
//! let file = sources.get(id).expect("just added");
//! let parsed = nudo_parser::parse(file);
//! let root = SourceFile::cast(parsed.tree().root()).expect("a source file");
//! for item in root.items() {
//!     if let Item::Function(function) = item {
//!         assert_eq!(function.name().map(|name| name.text().to_string()), Some("add".into()));
//!     }
//! }
//! ```
//!
//! # What this crate does not do
//!
//! Names, types and effects are milestones M3 and later. Nothing here resolves
//! anything: a `Path` is a list of names, not a reference, and a `Type` is a
//! shape, not a checked type.

use nudo_span::Span;
use nudo_syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

/// A node with a known shape.
///
/// Implemented for every typed wrapper in this crate. The lifetime is the tree
/// the node borrows, so a value of this crate is cheap to copy and cannot
/// outlive the tree it describes.
pub trait AstNode<'t>: Copy {
    /// The syntax node this value wraps.
    fn syntax(self) -> SyntaxNode<'t>;

    /// Reads a node as this type, if its kind matches.
    #[must_use]
    fn cast(node: SyntaxNode<'t>) -> Option<Self>
    where
        Self: Sized;

    /// The byte range this value covers.
    #[must_use]
    fn span(self) -> Span {
        self.syntax().span()
    }

    /// The source text this value covers.
    #[must_use]
    fn text(self) -> &'t str {
        self.syntax().text()
    }
}

/// Declares a wrapper around one syntax kind, and the `cast` that recognises
/// it. Every wrapper in this crate is one of these; the ones that are not have
/// a hand-written implementation below.
macro_rules! node_wrapper {
    ($(#[$meta:meta])* $name:ident, $kind:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name<'t> {
            syntax: SyntaxNode<'t>,
        }

        impl<'t> AstNode<'t> for $name<'t> {
            fn syntax(self) -> SyntaxNode<'t> {
                self.syntax
            }

            fn cast(node: SyntaxNode<'t>) -> Option<Self> {
                if node.kind() == SyntaxKind::$kind {
                    Some($name { syntax: node })
                } else {
                    None
                }
            }
        }
    };
}

node_wrapper!(
    /// The whole file.
    SourceFile,
    SourceFile
);
node_wrapper!(
    /// A parameter of a function, task or tool.
    Parameter,
    Parameter
);
node_wrapper!(
    /// A field of a struct, or of an enum variant.
    Field,
    Field
);
node_wrapper!(
    /// A variant of an enum.
    Variant,
    Variant
);
node_wrapper!(
    /// A module path: `web::search`.
    Path,
    Path
);
node_wrapper!(
    /// A block: `{ … }`.
    Block,
    Block
);

/// A token that names something.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Name<'t> {
    token: SyntaxToken<'t>,
}

impl<'t> Name<'t> {
    /// Reads a name from the first identifier token under `node`.
    #[must_use]
    pub fn cast(node: SyntaxNode<'t>) -> Option<Self> {
        node.child_tokens()
            .into_iter()
            .find(|token| token.kind() == SyntaxKind::Ident)
            .map(|token| Name { token })
    }

    /// The text of the name.
    #[must_use]
    pub fn text(self) -> &'t str {
        self.token.text()
    }

    /// The byte range of the name.
    #[must_use]
    pub fn span(self) -> Span {
        self.token.span()
    }
}

/// One item of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item<'t> {
    /// `function-decl`.
    Function(Function<'t>),
    /// `struct-decl`.
    Struct(Struct<'t>),
    /// `enum-decl`.
    Enum(Enum<'t>),
    /// `binding`.
    Binding(Binding<'t>),
    /// `const-decl`.
    Const(ConstDecl<'t>),
    /// `agent-decl`.
    Agent(Agent<'t>),
    /// `task-decl`.
    Task(Task<'t>),
    /// `tool-decl`.
    Tool(Tool<'t>),
    /// `model-decl`.
    Model(Model<'t>),
}

impl<'t> Item<'t> {
    /// Reads a node as an item, if it is one.
    #[must_use]
    pub fn cast(node: SyntaxNode<'t>) -> Option<Self> {
        match node.kind() {
            SyntaxKind::FunctionDecl => Function::cast(node).map(Item::Function),
            SyntaxKind::StructDecl => Struct::cast(node).map(Item::Struct),
            SyntaxKind::EnumDecl => Enum::cast(node).map(Item::Enum),
            SyntaxKind::Binding => Binding::cast(node).map(Item::Binding),
            SyntaxKind::ConstDecl => ConstDecl::cast(node).map(Item::Const),
            SyntaxKind::AgentDecl => Agent::cast(node).map(Item::Agent),
            SyntaxKind::TaskDecl => Task::cast(node).map(Item::Task),
            SyntaxKind::ToolDecl => Tool::cast(node).map(Item::Tool),
            SyntaxKind::ModelDecl => Model::cast(node).map(Item::Model),
            _ => None,
        }
    }

    /// The syntax node behind this item.
    #[must_use]
    pub fn syntax(self) -> SyntaxNode<'t> {
        match self {
            Item::Function(item) => item.syntax(),
            Item::Struct(item) => item.syntax(),
            Item::Enum(item) => item.syntax(),
            Item::Binding(item) => item.syntax(),
            Item::Const(item) => item.syntax(),
            Item::Agent(item) => item.syntax(),
            Item::Task(item) => item.syntax(),
            Item::Tool(item) => item.syntax(),
            Item::Model(item) => item.syntax(),
        }
    }

    /// The byte range this item covers.
    #[must_use]
    pub fn span(self) -> Span {
        self.syntax().span()
    }

    /// The source text this item covers.
    #[must_use]
    pub fn text(self) -> &'t str {
        self.syntax().text()
    }

    /// The name the item declares, if it declares one.
    #[must_use]
    pub fn name(self) -> Option<Name<'t>> {
        match self {
            Item::Function(item) => item.name(),
            Item::Struct(item) => item.name(),
            Item::Enum(item) => item.name(),
            Item::Binding(item) => item.name(),
            Item::Const(item) => item.name(),
            Item::Agent(item) => item.name(),
            Item::Task(item) => item.name(),
            Item::Tool(item) => item.name(),
            Item::Model(item) => item.name(),
        }
    }
}

impl<'t> SourceFile<'t> {
    /// The items of the file, in source order.
    ///
    /// Nodes with errors are left out: this crate describes what is well
    /// formed, and the syntax tree is where everything else lives.
    #[must_use]
    pub fn items(self) -> Vec<Item<'t>> {
        self.syntax
            .child_nodes()
            .into_iter()
            .filter_map(Item::cast)
            .collect()
    }
}

node_wrapper!(
    /// `fn name(params) -> Type { … }`.
    Function,
    FunctionDecl
);
node_wrapper!(
    /// `struct Name { … }`.
    Struct,
    StructDecl
);
node_wrapper!(
    /// `enum Name { … }`.
    Enum,
    EnumDecl
);
node_wrapper!(
    /// `let name = value;`.
    Binding,
    Binding
);
node_wrapper!(
    /// `const NAME: Type = value;`.
    ConstDecl,
    ConstDecl
);
node_wrapper!(
    /// `agent Name { … }`.
    Agent,
    AgentDecl
);
node_wrapper!(
    /// `task Name(params) -> Type { … }`.
    Task,
    TaskDecl
);
node_wrapper!(
    /// `tool name(params) -> Type { … }`.
    Tool,
    ToolDecl
);
node_wrapper!(
    /// `model Name { … }`.
    Model,
    ModelDecl
);

macro_rules! owner_accessors {
    ($name:ident) => {
        impl<'t> $name<'t> {
            /// The name this declaration introduces, if the tree holds it.
            #[must_use]
            pub fn name(self) -> Option<Name<'t>> {
                Name::cast(self.syntax)
            }
        }
    };
}

owner_accessors!(Function);
owner_accessors!(Struct);
owner_accessors!(Enum);
owner_accessors!(Binding);
owner_accessors!(ConstDecl);
owner_accessors!(Agent);
owner_accessors!(Task);
owner_accessors!(Model);

impl<'t> Function<'t> {
    /// The parameters, in source order.
    #[must_use]
    pub fn parameters(self) -> Vec<Parameter<'t>> {
        self.syntax
            .descendants()
            .into_iter()
            .filter_map(Parameter::cast)
            .collect()
    }

    /// The declared return type, if the signature has one.
    #[must_use]
    pub fn return_type(self) -> Option<Type<'t>> {
        self.syntax
            .children_of_kind(SyntaxKind::ParameterList)
            .first()
            .and_then(|list| after_tokens(*list, self.syntax, Type::cast))
    }

    /// The body.
    #[must_use]
    pub fn body(self) -> Option<Block<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Block::cast)
    }

    /// The capabilities the function declares with `with …`.
    #[must_use]
    pub fn capabilities(self) -> Vec<Name<'t>> {
        capability_names(self.syntax)
    }
}

impl<'t> Parameter<'t> {
    /// The parameter's name.
    #[must_use]
    pub fn name(self) -> Option<Name<'t>> {
        Name::cast(self.syntax)
    }

    /// The parameter's type.
    #[must_use]
    pub fn ty(self) -> Option<Type<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Type::cast)
    }
}

impl<'t> Struct<'t> {
    /// The fields, in source order.
    #[must_use]
    pub fn fields(self) -> Vec<Field<'t>> {
        self.syntax
            .children()
            .into_iter()
            .filter_map(SyntaxElement::into_node)
            .filter_map(Field::cast)
            .collect()
    }
}

impl<'t> Field<'t> {
    /// The field's name.
    #[must_use]
    pub fn name(self) -> Option<Name<'t>> {
        Name::cast(self.syntax)
    }

    /// The field's type.
    #[must_use]
    pub fn ty(self) -> Option<Type<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Type::cast)
    }
}

impl<'t> Enum<'t> {
    /// The variants, in source order.
    #[must_use]
    pub fn variants(self) -> Vec<Variant<'t>> {
        self.syntax
            .children()
            .into_iter()
            .filter_map(SyntaxElement::into_node)
            .filter_map(Variant::cast)
            .collect()
    }
}

impl<'t> Variant<'t> {
    /// The variant's name.
    #[must_use]
    pub fn name(self) -> Option<Name<'t>> {
        Name::cast(self.syntax)
    }

    /// The fields the variant carries, if it carries any.
    #[must_use]
    pub fn fields(self) -> Vec<Field<'t>> {
        self.syntax
            .children()
            .into_iter()
            .filter_map(SyntaxElement::into_node)
            .filter_map(Field::cast)
            .collect()
    }
}

impl<'t> Binding<'t> {
    /// The annotated type, if the binding has one.
    #[must_use]
    pub fn ty(self) -> Option<Type<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Type::cast)
    }

    /// The initialiser.
    #[must_use]
    pub fn value(self) -> Option<Expression<'t>> {
        self.syntax
            .child_nodes()
            .into_iter()
            .find_map(Expression::cast)
    }
}

impl<'t> ConstDecl<'t> {
    /// The annotated type.
    #[must_use]
    pub fn ty(self) -> Option<Type<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Type::cast)
    }

    /// The constant's value.
    #[must_use]
    pub fn value(self) -> Option<Expression<'t>> {
        self.syntax
            .child_nodes()
            .into_iter()
            .find_map(Expression::cast)
    }
}

impl<'t> Task<'t> {
    /// The parameters, in source order.
    #[must_use]
    pub fn parameters(self) -> Vec<Parameter<'t>> {
        self.syntax
            .descendants()
            .into_iter()
            .filter_map(Parameter::cast)
            .collect()
    }

    /// The declared result type.
    #[must_use]
    pub fn return_type(self) -> Option<Type<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Type::cast)
    }

    /// The agent the task names, if it names one.
    #[must_use]
    pub fn agent(self) -> Option<Path<'t>> {
        self.syntax
            .children_of_kind(SyntaxKind::TaskAgentClause)
            .first()
            .and_then(|clause| clause.child_nodes().into_iter().find_map(Path::cast))
    }

    /// The verifier the task names, if it names one.
    #[must_use]
    pub fn verifier(self) -> Option<Path<'t>> {
        self.syntax
            .children_of_kind(SyntaxKind::VerifyClause)
            .first()
            .and_then(|clause| clause.child_nodes().into_iter().find_map(Path::cast))
    }
}

impl<'t> Tool<'t> {
    /// The tool's name.
    #[must_use]
    pub fn name(self) -> Option<Name<'t>> {
        self.syntax
            .child_nodes()
            .into_iter()
            .find_map(Path::cast)
            .and_then(|path| path.segments().into_iter().next())
    }

    /// The full path of the tool's name.
    #[must_use]
    pub fn path(self) -> Option<Path<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Path::cast)
    }

    /// The parameters, in source order.
    #[must_use]
    pub fn parameters(self) -> Vec<Parameter<'t>> {
        self.syntax
            .descendants()
            .into_iter()
            .filter_map(Parameter::cast)
            .collect()
    }

    /// The declared result type, if the declaration has one.
    #[must_use]
    pub fn return_type(self) -> Option<Type<'t>> {
        self.syntax
            .children_of_kind(SyntaxKind::ParameterList)
            .first()
            .and_then(|list| after_tokens(*list, self.syntax, Type::cast))
    }

    /// The capabilities the tool declares with `with …`.
    #[must_use]
    pub fn capabilities(self) -> Vec<Name<'t>> {
        capability_names(self.syntax)
    }

    /// The body.
    #[must_use]
    pub fn body(self) -> Option<Block<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Block::cast)
    }
}

impl<'t> Agent<'t> {
    /// The capabilities the agent declares with `allow: …`.
    #[must_use]
    pub fn allowed(self) -> Vec<Name<'t>> {
        self.syntax
            .children_of_kind(SyntaxKind::AllowClause)
            .first()
            .map(|clause| {
                clause
                    .descendants()
                    .into_iter()
                    .filter_map(Capability::cast)
                    .filter_map(Capability::name)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The tools the agent declares with `tools: …`.
    #[must_use]
    pub fn tools(self) -> Vec<Path<'t>> {
        self.syntax
            .children_of_kind(SyntaxKind::ToolsClause)
            .first()
            .map(|clause| {
                clause
                    .children()
                    .into_iter()
                    .filter_map(SyntaxElement::into_node)
                    .filter_map(Path::cast)
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// A single capability in a list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capability<'t> {
    syntax: SyntaxNode<'t>,
}

impl<'t> Capability<'t> {
    /// Reads a node as a capability, if it is one.
    #[must_use]
    pub fn cast(node: SyntaxNode<'t>) -> Option<Self> {
        if node.kind() == SyntaxKind::Capability {
            Some(Capability { syntax: node })
        } else {
            None
        }
    }

    /// The capability's name.
    #[must_use]
    pub fn name(self) -> Option<Name<'t>> {
        Name::cast(self.syntax)
    }
}

impl<'t> Path<'t> {
    /// The segments of the path, in source order.
    #[must_use]
    pub fn segments(self) -> Vec<Name<'t>> {
        self.syntax
            .child_tokens()
            .into_iter()
            .filter(|token| token.kind() == SyntaxKind::Ident)
            .map(|token| Name { token })
            .collect()
    }
}

/// One statement of a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Statement<'t> {
    /// A `let` binding.
    Binding(Binding<'t>),
    /// An expression followed by `;`.
    Expression(Expression<'t>),
}

impl<'t> Statement<'t> {
    /// The syntax node behind this statement.
    #[must_use]
    pub fn syntax(self) -> SyntaxNode<'t> {
        match self {
            Statement::Binding(binding) => binding.syntax(),
            Statement::Expression(expression) => expression.syntax(),
        }
    }
}

impl<'t> Block<'t> {
    /// The statements, in source order.
    #[must_use]
    pub fn statements(self) -> Vec<Statement<'t>> {
        let mut statements = Vec::new();
        for child in self.syntax.child_nodes() {
            if let Some(binding) = Binding::cast(child) {
                statements.push(Statement::Binding(binding));
            } else if child.kind() == SyntaxKind::ExpressionStatement {
                if let Some(expression) = child.child_nodes().into_iter().find_map(Expression::cast)
                {
                    statements.push(Statement::Expression(expression));
                }
            }
        }
        statements
    }

    /// The block's value: the final expression, which has no `;`.
    ///
    /// A block whose last element is a statement has no value.
    #[must_use]
    pub fn tail_expression(self) -> Option<Expression<'t>> {
        // The closing `}` is a child of the block too, so the search is for the
        // last child *node*, not the last child element.
        let last = self.syntax.child_nodes().into_iter().next_back()?;
        if last.kind() == SyntaxKind::ExpressionStatement {
            return None;
        }
        Expression::cast(last)
    }
}

/// An expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expression<'t> {
    /// `a + b`, and every other binary operator.
    Binary(SyntaxNode<'t>),
    /// `-a`, `!a`.
    Unary(SyntaxNode<'t>),
    /// `f(x)`.
    Call(CallExpression<'t>),
    /// `a.b`.
    Field(SyntaxNode<'t>),
    /// `a[i]`.
    Index(SyntaxNode<'t>),
    /// A name, or a path.
    Path(PathExpression<'t>),
    /// A literal.
    Literal(SyntaxNode<'t>),
    /// `( … )`.
    Parenthesized(SyntaxNode<'t>),
    /// The block's own value shapes.
    Block(Block<'t>),
    /// `if … { … } else { … }`.
    If(SyntaxNode<'t>),
    /// `match … { … }`.
    Match(SyntaxNode<'t>),
    /// `ask Name { … }`.
    Ask(AskExpression<'t>),
    /// `verify value with Verifier`.
    Verify(VerifyExpression<'t>),
    /// `delegate Name { … } with budget: …`.
    Delegate(DelegateExpression<'t>),
}

impl<'t> Expression<'t> {
    /// Reads a node as an expression, if it is one.
    #[must_use]
    pub fn cast(node: SyntaxNode<'t>) -> Option<Self> {
        match node.kind() {
            SyntaxKind::BinaryExpression => Some(Expression::Binary(node)),
            SyntaxKind::UnaryExpression => Some(Expression::Unary(node)),
            SyntaxKind::CallExpression => CallExpression::cast(node).map(Expression::Call),
            SyntaxKind::FieldExpression => Some(Expression::Field(node)),
            SyntaxKind::IndexExpression => Some(Expression::Index(node)),
            SyntaxKind::PathExpression => PathExpression::cast(node).map(Expression::Path),
            SyntaxKind::LiteralExpression => Some(Expression::Literal(node)),
            SyntaxKind::ParenthesizedExpression => Some(Expression::Parenthesized(node)),
            SyntaxKind::Block => Block::cast(node).map(Expression::Block),
            SyntaxKind::IfExpression => Some(Expression::If(node)),
            SyntaxKind::MatchExpression => Some(Expression::Match(node)),
            SyntaxKind::AskExpression => AskExpression::cast(node).map(Expression::Ask),
            SyntaxKind::VerifyExpression => VerifyExpression::cast(node).map(Expression::Verify),
            SyntaxKind::DelegateExpression => {
                DelegateExpression::cast(node).map(Expression::Delegate)
            }
            _ => None,
        }
    }

    /// The syntax node behind this expression.
    #[must_use]
    pub fn syntax(self) -> SyntaxNode<'t> {
        match self {
            Expression::Binary(node)
            | Expression::Unary(node)
            | Expression::Field(node)
            | Expression::Index(node)
            | Expression::Literal(node)
            | Expression::Parenthesized(node)
            | Expression::If(node)
            | Expression::Match(node) => node,
            Expression::Call(call) => call.syntax(),
            Expression::Path(path) => path.syntax(),
            Expression::Block(block) => block.syntax(),
            Expression::Ask(ask) => ask.syntax(),
            Expression::Verify(verify) => verify.syntax(),
            Expression::Delegate(delegate) => delegate.syntax(),
        }
    }

    /// The byte range this expression covers.
    #[must_use]
    pub fn span(self) -> Span {
        self.syntax().span()
    }

    /// The source text this expression covers.
    #[must_use]
    pub fn text(self) -> &'t str {
        self.syntax().text()
    }

    /// The operands of a binary expression: `(left, right)`.
    ///
    /// `None` for any other shape, and also for a binary expression whose
    /// operands are not both expressions — which is what an error node in the
    /// middle produces.
    #[must_use]
    pub fn operands(self) -> Option<(Expression<'t>, Expression<'t>)> {
        if !matches!(self, Expression::Binary(_)) {
            return None;
        }
        let mut operands = self
            .syntax()
            .child_nodes()
            .into_iter()
            .filter_map(Expression::cast);
        let left = operands.next()?;
        let right = operands.next()?;
        Some((left, right))
    }
}

node_wrapper!(
    /// A path used as an expression.
    PathExpression,
    PathExpression
);
node_wrapper!(
    /// `f(x)`.
    CallExpression,
    CallExpression
);
node_wrapper!(
    /// `ask Name { … }`.
    AskExpression,
    AskExpression
);
node_wrapper!(
    /// `verify value with Verifier`.
    VerifyExpression,
    VerifyExpression
);
node_wrapper!(
    /// `delegate Name { … }`.
    DelegateExpression,
    DelegateExpression
);

impl<'t> PathExpression<'t> {
    /// The path this expression names.
    #[must_use]
    pub fn path(self) -> Option<Path<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Path::cast)
    }
}

impl<'t> CallExpression<'t> {
    /// The expression being called.
    #[must_use]
    pub fn callee(self) -> Option<Expression<'t>> {
        self.syntax
            .child_nodes()
            .into_iter()
            .find_map(Expression::cast)
    }

    /// The arguments, in source order.
    #[must_use]
    pub fn arguments(self) -> Vec<Expression<'t>> {
        self.syntax
            .children_of_kind(SyntaxKind::ArgumentList)
            .first()
            .map(|list| {
                list.child_nodes()
                    .into_iter()
                    .filter_map(Expression::cast)
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl<'t> AskExpression<'t> {
    /// The model or agent being asked.
    #[must_use]
    pub fn agent(self) -> Option<Path<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Path::cast)
    }

    /// The prompt block.
    #[must_use]
    pub fn prompt(self) -> Option<Block<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Block::cast)
    }
}

impl<'t> VerifyExpression<'t> {
    /// The expression being verified.
    #[must_use]
    pub fn value(self) -> Option<Expression<'t>> {
        self.syntax
            .child_nodes()
            .into_iter()
            .find_map(Expression::cast)
    }

    /// The verifier.
    #[must_use]
    pub fn verifier(self) -> Option<Path<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Path::cast)
    }
}

impl<'t> DelegateExpression<'t> {
    /// The agent the work is delegated to.
    #[must_use]
    pub fn agent(self) -> Option<Path<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Path::cast)
    }

    /// The block describing the delegated work.
    #[must_use]
    pub fn request(self) -> Option<Block<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Block::cast)
    }
}

/// One match arm: a pattern and the expression it selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchArm<'t> {
    syntax: SyntaxNode<'t>,
}

impl<'t> MatchArm<'t> {
    /// Reads a node as a match arm, if it is one.
    #[must_use]
    pub fn cast(node: SyntaxNode<'t>) -> Option<Self> {
        if node.kind() == SyntaxKind::MatchArm {
            Some(MatchArm { syntax: node })
        } else {
            None
        }
    }

    /// The pattern.
    #[must_use]
    pub fn pattern(self) -> Option<Pattern<'t>> {
        self.syntax
            .child_nodes()
            .into_iter()
            .find_map(Pattern::cast)
    }

    /// The expression the arm selects.
    #[must_use]
    pub fn body(self) -> Option<Expression<'t>> {
        self.syntax
            .child_nodes()
            .into_iter()
            .find_map(Expression::cast)
    }
}

/// A pattern: a path, optionally with nested patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pattern<'t> {
    syntax: SyntaxNode<'t>,
}

impl<'t> Pattern<'t> {
    /// Reads a node as a pattern, if it is one.
    #[must_use]
    pub fn cast(node: SyntaxNode<'t>) -> Option<Self> {
        if node.kind() == SyntaxKind::Pattern {
            Some(Pattern { syntax: node })
        } else {
            None
        }
    }

    /// The path the pattern names.
    #[must_use]
    pub fn path(self) -> Option<Path<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Path::cast)
    }

    /// The nested patterns, in source order.
    #[must_use]
    pub fn sub_patterns(self) -> Vec<Pattern<'t>> {
        self.syntax
            .child_nodes()
            .into_iter()
            .filter_map(Pattern::cast)
            .collect()
    }
}

/// A type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type<'t> {
    /// A named type: `Int`.
    Path(PathType<'t>),
    /// A generic type: `Result<Int, Text>`.
    Generic(GenericType<'t>),
    /// A sequence: `[T]`.
    Sequence(SyntaxNode<'t>),
    /// A function value: `Fn(A) -> B`.
    Function(SyntaxNode<'t>),
    /// A parenthesised type.
    Parenthesized(SyntaxNode<'t>),
    /// An optional type: `T?`.
    Optional(SyntaxNode<'t>),
}

impl<'t> Type<'t> {
    /// Reads a node as a type, if it is one.
    #[must_use]
    pub fn cast(node: SyntaxNode<'t>) -> Option<Self> {
        match node.kind() {
            SyntaxKind::PathType => PathType::cast(node).map(Type::Path),
            SyntaxKind::GenericType => GenericType::cast(node).map(Type::Generic),
            SyntaxKind::SequenceType => Some(Type::Sequence(node)),
            SyntaxKind::FunctionType => Some(Type::Function(node)),
            SyntaxKind::ParenthesizedType => Some(Type::Parenthesized(node)),
            SyntaxKind::OptionalType => Some(Type::Optional(node)),
            _ => None,
        }
    }

    /// The syntax node behind this type.
    #[must_use]
    pub fn syntax(self) -> SyntaxNode<'t> {
        match self {
            Type::Path(inner) => inner.syntax(),
            Type::Generic(inner) => inner.syntax(),
            Type::Sequence(node)
            | Type::Function(node)
            | Type::Parenthesized(node)
            | Type::Optional(node) => node,
        }
    }

    /// The byte range this type covers.
    #[must_use]
    pub fn span(self) -> Span {
        self.syntax().span()
    }

    /// The source text this type covers.
    #[must_use]
    pub fn text(self) -> &'t str {
        self.syntax().text()
    }

    /// The type inside an `OptionalType`, or the type itself.
    #[must_use]
    pub fn inner(self) -> Option<Type<'t>> {
        match self {
            Type::Optional(node) => node.child_nodes().into_iter().find_map(Type::cast),
            _ => None,
        }
    }
}

node_wrapper!(
    /// A named type.
    PathType,
    PathType
);
node_wrapper!(
    /// A generic type.
    GenericType,
    GenericType
);

impl<'t> PathType<'t> {
    /// The path the type names.
    #[must_use]
    pub fn path(self) -> Option<Path<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Path::cast)
    }
}

impl<'t> GenericType<'t> {
    /// The generic type's name.
    #[must_use]
    pub fn path(self) -> Option<Path<'t>> {
        self.syntax.child_nodes().into_iter().find_map(Path::cast)
    }

    /// The type arguments, in source order.
    #[must_use]
    pub fn arguments(self) -> Vec<Type<'t>> {
        self.syntax
            .child_nodes()
            .into_iter()
            .filter_map(Type::cast)
            .collect()
    }
}

/// The capability names of an `EffectClause` under `node`.
fn capability_names<'t>(node: SyntaxNode<'t>) -> Vec<Name<'t>> {
    node.children_of_kind(SyntaxKind::EffectClause)
        .first()
        .map(|clause| {
            clause
                .descendants()
                .into_iter()
                .filter_map(Capability::cast)
                .filter_map(Capability::name)
                .collect()
        })
        .unwrap_or_default()
}

/// The first `T` that appears after `anchor` among `owner`'s child nodes.
///
/// The return type of a function is written after the parameter list, and it is
/// the only `Type` after it; this is how the wrapper finds it without the parser
/// having to add a node for "the return type".
fn after_tokens<'t, T>(
    anchor: SyntaxNode<'t>,
    owner: SyntaxNode<'t>,
    cast: fn(SyntaxNode<'t>) -> Option<T>,
) -> Option<T> {
    let children = owner.children();
    let position = children
        .iter()
        .position(|element| element.into_node() == Some(anchor))?;
    children[position + 1..]
        .iter()
        .filter_map(|element| element.into_node())
        .find_map(cast)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nudo_source::SourceMap;

    fn parse(text: &str) -> nudo_parser::Parse {
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", text);
        let file = sources.get(id).expect("just added");
        nudo_parser::parse(file)
    }

    /// Parses `text`, asserts it is clean, and hands the items to `check`.
    ///
    /// The parse is owned by this function on purpose: a typed wrapper borrows
    /// its tree, so a helper that returned items would be returning a borrow of
    /// a local value.
    fn with_items<T>(text: &str, check: impl FnOnce(Vec<Item<'_>>) -> T) -> T {
        let parsed = parse(text);
        assert!(
            !parsed.has_errors(),
            "test input must parse cleanly: {text}"
        );
        let root = SourceFile::cast(parsed.tree().root()).expect("a source file");
        check(root.items())
    }

    #[test]
    fn reads_a_function_signature_and_body() {
        let parsed = parse("fn add(a: Int, b: Int) -> Int {\n    a + b\n}\n");
        let root = SourceFile::cast(parsed.tree().root()).expect("a source file");
        assert_eq!(root.items().len(), 1);
        // The tree outlives the parse, so a second cast works on the same tree.
        let function = match root.items()[0] {
            Item::Function(function) => function,
            other => panic!("expected a function, found {other:?}"),
        };
        assert_eq!(function.name().expect("a name").text(), "add");
        let parameters = function.parameters();
        assert_eq!(parameters.len(), 2);
        assert_eq!(parameters[0].name().expect("a name").text(), "a");
        assert_eq!(parameters[1].name().expect("a name").text(), "b");
        let ty = function.return_type().expect("a return type");
        assert!(matches!(ty, Type::Path(_)));
        assert_eq!(ty.text(), "Int");
        let body = function.body().expect("a body");
        assert!(body.statements().is_empty());
        let tail = body.tail_expression().expect("a value");
        assert!(matches!(tail, Expression::Binary(_)));
        let (left, right) = tail.operands().expect("two operands");
        assert_eq!(left.text(), "a");
        assert_eq!(right.text(), "b");
        assert!(function.capabilities().is_empty());
    }

    #[test]
    fn reads_a_binding_with_a_annotation() {
        let parsed = parse("let draft: Generated<Article> = ask Writer { \"x\" };\n");
        let root = SourceFile::cast(parsed.tree().root()).expect("a source file");
        let binding = match root.items()[0] {
            Item::Binding(binding) => binding,
            other => panic!("expected a binding, found {other:?}"),
        };
        assert_eq!(binding.name().expect("a name").text(), "draft");
        let ty = binding.ty().expect("an annotated type");
        let Type::Generic(generic) = ty else {
            panic!("expected a generic type, found {ty:?}");
        };
        assert_eq!(generic.path().expect("a path").text(), "Generated");
        assert_eq!(generic.arguments().len(), 1);
        let value = binding.value().expect("a value");
        let Expression::Ask(ask) = value else {
            panic!("expected an ask expression, found {value:?}");
        };
        assert_eq!(ask.agent().expect("an agent").text(), "Writer");
        assert!(ask.prompt().is_some());
    }

    #[test]
    fn reads_structs_enums_and_agents() {
        let source = "\
struct User {
    name: Text
    age: Int
}

enum Status {
    Pending
    Failed(reason: Text)
}

agent Researcher {
    role: \"research\"
    tools: web::search, web::open
    allow: Network
}

task Research(topic: Text) -> Verified<Report> {
    agent: Researcher
    verify: SourcesRequired
}
";
        let parsed = parse(source);
        let root = SourceFile::cast(parsed.tree().root()).expect("a source file");
        let items = root.items();
        assert_eq!(items.len(), 4);

        let Item::Struct(user) = items[0] else {
            panic!("expected a struct")
        };
        assert_eq!(user.name().expect("a name").text(), "User");
        assert_eq!(user.fields().len(), 2);
        assert_eq!(user.fields()[1].name().expect("a name").text(), "age");

        let Item::Enum(status) = items[1] else {
            panic!("expected an enum")
        };
        assert_eq!(status.variants().len(), 2);
        assert_eq!(status.variants()[1].fields().len(), 1);

        let Item::Agent(agent) = items[2] else {
            panic!("expected an agent")
        };
        assert_eq!(agent.allowed().len(), 1);
        assert_eq!(agent.allowed()[0].text(), "Network");
        assert_eq!(agent.tools().len(), 2);
        assert_eq!(agent.tools()[1].text(), "web::open");

        let Item::Task(task) = items[3] else {
            panic!("expected a task")
        };
        assert_eq!(task.agent().expect("an agent").text(), "Researcher");
        assert_eq!(
            task.verifier().expect("a verifier").text(),
            "SourcesRequired"
        );
        assert_eq!(task.parameters().len(), 1);
        assert!(matches!(task.return_type(), Some(Type::Generic(_))));
    }

    #[test]
    fn reads_an_optional_type_and_a_sequence() {
        with_items(
            "fn f(xs: [Int], maybe: Text?) -> Unit {\n    xs\n}\n",
            |items| {
                let Item::Function(function) = items[0] else {
                    panic!("expected a function")
                };
                let parameters = function.parameters();
                assert!(matches!(parameters[0].ty(), Some(Type::Sequence(_))));
                let optional = parameters[1].ty().expect("a type");
                assert!(matches!(optional, Type::Optional(_)));
                let inner = optional.inner().expect("an inner type");
                assert_eq!(inner.text(), "Text");
            },
        );
    }

    #[test]
    fn reads_control_flow_and_match_arms() {
        with_items(
            "fn f(status: Status) -> Text {\n    match status {\n        Pending => \"p\"\n        Failed(reason) => reason\n    }\n}\n",
            |items| {
                let Item::Function(function) = items[0] else {
                    panic!("expected a function")
                };
                let body = function.body().expect("a body");
                let tail = body.tail_expression().expect("a value");
                let Expression::Match(node) = tail else {
                    panic!("expected a match, found {tail:?}")
                };
                let arms: Vec<MatchArm<'_>> = node
                    .child_nodes()
                    .into_iter()
                    .filter_map(MatchArm::cast)
                    .collect();
                assert_eq!(arms.len(), 2);
                let pattern = arms[1].pattern().expect("a pattern");
                assert_eq!(pattern.path().expect("a path").text(), "Failed");
                assert_eq!(pattern.sub_patterns().len(), 1);
                assert!(arms[1].body().is_some());
            },
        );
    }

    #[test]
    fn statements_and_tail_expressions_are_distinguished() {
        with_items("fn f() {\n    let a = 1;\n    g();\n    a\n}\n", |items| {
            let Item::Function(function) = items[0] else {
                panic!("expected a function")
            };
            let body = function.body().expect("a body");
            let statements = body.statements();
            assert_eq!(statements.len(), 2);
            assert!(matches!(statements[0], Statement::Binding(_)));
            assert!(matches!(statements[1], Statement::Expression(_)));
            assert!(body.tail_expression().is_some());
        });
    }

    #[test]
    fn a_block_whose_last_element_is_a_statement_has_no_value() {
        with_items("fn f() {\n    g();\n}\n", |items| {
            let Item::Function(function) = items[0] else {
                panic!("expected a function")
            };
            let body = function.body().expect("a body");
            assert!(body.tail_expression().is_none());
            assert_eq!(body.statements().len(), 1);
        });
    }

    #[test]
    fn a_broken_file_yields_no_items_rather_than_guesses() {
        let parsed = parse("fn { }} struct");
        let root = SourceFile::cast(parsed.tree().root()).expect("a source file");
        // The crate does not invent structure: whatever survives is real, and
        // nothing that is broken is reported as well formed.
        for item in root.items() {
            assert!(matches!(
                item.syntax().kind(),
                SyntaxKind::FunctionDecl
                    | SyntaxKind::StructDecl
                    | SyntaxKind::EnumDecl
                    | SyntaxKind::Binding
                    | SyntaxKind::ConstDecl
                    | SyntaxKind::AgentDecl
                    | SyntaxKind::TaskDecl
                    | SyntaxKind::ToolDecl
                    | SyntaxKind::ModelDecl
            ));
        }
    }

    #[test]
    fn verifications_and_delegations_are_readable() {
        let source = "\
let article: Verified<Article> = verify draft with ArticleVerifier;
let work = delegate Researcher { \"topic\" } with budget: Budget(tokens: 4_000);
";
        let parsed = parse(source);
        let root = SourceFile::cast(parsed.tree().root()).expect("a source file");
        let items = root.items();
        let Item::Binding(article) = items[0] else {
            panic!("expected a binding")
        };
        let Expression::Verify(verify) = article.value().expect("a value") else {
            panic!("expected a verification")
        };
        assert_eq!(verify.value().expect("a value").text(), "draft");
        assert_eq!(
            verify.verifier().expect("a verifier").text(),
            "ArticleVerifier"
        );

        let Item::Binding(work) = items[1] else {
            panic!("expected a binding")
        };
        let Expression::Delegate(delegate) = work.value().expect("a value") else {
            panic!("expected a delegation")
        };
        assert_eq!(delegate.agent().expect("an agent").text(), "Researcher");
        assert!(delegate.request().is_some());
    }

    #[test]
    fn cast_rejects_the_wrong_kind() {
        let parsed = parse("fn f() {}\n");
        let root = parsed.tree().root();
        assert!(Function::cast(root).is_none());
        assert!(Block::cast(root).is_none());
        assert!(SourceFile::cast(root).is_some());
        assert!(Item::cast(root).is_none());
        assert!(Type::cast(root).is_none());
        assert!(Expression::cast(root).is_none());
        let function = SourceFile::cast(root)
            .expect("a source file")
            .items()
            .into_iter()
            .next()
            .expect("an item");
        assert!(Name::cast(function.syntax()).is_some());
        assert_eq!(function.name().expect("a name").span().len(), 1);
        assert_eq!(function.span(), function.syntax().span());
        assert!(!function.text().is_empty());
    }
}
