//! HIR — the semantic representation, and name resolution.
//!
//! ```text
//! .nudo → Lexer → Parser → SyntaxTree → AST → HIR (+ name resolution)
//! ```
//!
//! # What this layer is for
//!
//! HIR is not "the AST with different structs". It exists to stop representing
//! *text* and start representing *meaning*: after lowering, a name is no longer
//! a string in a tree, it is a reference to a definition.
//!
//! ```text
//! AST:  PathExpression("value")      a name, in a place
//!                 ↓ lower + resolve
//! HIR:  ExprKind::Path { target: Resolved(DefId(3)) }
//! ```
//!
//! Three things follow, and they are the reasons this crate exists:
//!
//! * **Nothing syntactic survives that carries no meaning.** Parentheses are
//!   gone: `(a)` and `a` are the same HIR, because they mean the same thing.
//!   Trivia was already gone, one layer down.
//! * **Definitions are first-class.** A [`DefId`] is stable for the
//!   compilation, so a diagnostic — and later a type, a trace, a provenance
//!   record — can point at a definition instead of at the third occurrence of a
//!   name.
//! * **Scopes are explicit.** Every definition records the scope it lives in, so
//!   "why can this name not see that one" has an answer.
//!
//! # The rules it implements
//!
//! * **Two namespaces**: `type` (structs, enums, built-in types, type
//!   parameters) and `value` (functions, bindings, constants, parameters,
//!   agents, tasks, tools, models). `struct User` and `let user = …` coexist,
//!   and using one where the other belongs is `NDO2003` rather than "not found".
//! * **Shadowing in a nested scope is allowed; a duplicate in one scope is
//!   `NDO2002`.** The scope is what makes the difference.
//! * **An initialiser is lowered before its own name is in scope**, so
//!   `let x = x;` refers to the outer `x`, as a reader expects.
//! * **Items are hoisted**: a function may call one declared below it.
//! * **An unresolved name stays in the HIR**, marked, so that one mistake does
//!   not turn into a cascade and a consumer can still ask about the rest.
//!
//! # What it deliberately does not do
//!
//! * **No types.** `Int` resolves to a built-in definition and is not checked;
//!   nothing is inferred. That is M3.2.
//! * **No pattern resolution.** A pattern head (`Pending` in `match x { Pending
//!   => … }`) cannot be resolved without the scrutinee's type — a bare name
//!   there is either a variant or a new binding. Sub-patterns are lowered as
//!   bindings; heads are left for M3.2, and the rule is recorded in
//!   [`docs/internals/hir-design.md`][design].
//! * **No modules.** A path written with `::` is reported as unresolved with a
//!   note, because `::` reaches into a module and modules are M10.
//!
//! [design]: https://github.com/smouj/nudo/blob/main/docs/internals/hir-design.md

use nudo_diagnostics::{Diagnostic, Diagnostics, codes};
use nudo_source::{SourceFile, SourceId};
use nudo_span::Span;
use nudo_syntax::{SyntaxKind, SyntaxNode, SyntaxToken, SyntaxTree};

/// Identifies a definition: a function, a type, a binding, a parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DefId(u32);

impl DefId {
    /// The index of this definition.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }

    /// The definition at `index`.
    ///
    /// For consumers that walk the arena in order — the type checker does — and
    /// that therefore have an index rather than an id. It is not a way to invent
    /// a definition: an index that was never declared has no entry in the arena,
    /// and every accessor here returns `None` for it.
    #[must_use]
    pub const fn from_index(index: u32) -> Self {
        DefId(index)
    }
}

/// Identifies an expression in the HIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExprId(u32);

impl ExprId {
    /// The index of this expression.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// Identifies a statement in the HIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StmtId(u32);

impl StmtId {
    /// The index of this statement.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// Identifies a scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScopeId(u32);

impl ScopeId {
    /// The index of this scope.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// The namespace a name lives in.
///
/// Two, not one: a struct and a binding may share a name without ambiguity,
/// because nothing is both a type and a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Namespace {
    /// Types: structs, enums, built-in types, type parameters.
    Type,
    /// Values: functions, bindings, constants, parameters, agents, tasks,
    /// tools, models.
    Value,
}

impl Namespace {
    /// The namespace as lowercase text, for diagnostics and dumps.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Namespace::Type => "type",
            Namespace::Value => "value",
        }
    }
}

/// What a definition is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefKind {
    /// `fn`.
    Function,
    /// `struct`.
    Struct,
    /// `enum`.
    Enum,
    /// `agent`.
    Agent,
    /// `task`.
    Task,
    /// `tool`.
    Tool,
    /// `model`.
    Model,
    /// `const`.
    Const,
    /// A `let` binding.
    Binding,
    /// A function, task or tool parameter.
    Parameter,
    /// A pattern binding inside a `match` arm.
    Local,
    /// A type parameter of a declaration: the `T` of `fn identity<T>`.
    TypeParameter,
    /// A type the language provides: `Int`, `Result`, `Verified`, …
    BuiltinType,
}

impl DefKind {
    /// The kind as lowercase text, for diagnostics and dumps.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            DefKind::Function => "function",
            DefKind::Struct => "struct",
            DefKind::Enum => "enum",
            DefKind::Agent => "agent",
            DefKind::Task => "task",
            DefKind::Tool => "tool",
            DefKind::Model => "model",
            DefKind::Const => "const",
            DefKind::Binding => "binding",
            DefKind::Parameter => "parameter",
            DefKind::Local => "local",
            DefKind::TypeParameter => "type-parameter",
            DefKind::BuiltinType => "builtin-type",
        }
    }

    /// The namespace this kind lives in.
    #[must_use]
    pub const fn namespace(self) -> Namespace {
        match self {
            DefKind::Struct | DefKind::Enum | DefKind::TypeParameter | DefKind::BuiltinType => {
                Namespace::Type
            }
            _ => Namespace::Value,
        }
    }
}

/// A named field of a struct, or one field of an enum variant's payload.
///
/// Fields are **not** names in a scope: a field is reached through a value
/// (`article.title`), never on its own, so they are recorded beside their owner
/// rather than declared as definitions. That is why this is not a `Def`.
#[derive(Debug, Clone)]
pub struct Field {
    /// The field's name.
    pub name: String,
    /// Its type, when one is written.
    pub ty: Option<TypeRef>,
    /// Where it is written.
    pub span: Span,
}

/// One arm of a `match`.
///
/// The head is recorded as **written**, not as decided. A bare name in a pattern
/// is either a variant of the scrutinee's enum or a new binding, and which one it
/// is needs the scrutinee's type — which exists now, one layer up. Guessing from
/// capitalisation here would invent a language rule in the wrong layer.
#[derive(Debug, Clone)]
pub struct Arm {
    /// The name written at the pattern's head, if the pattern has one.
    pub head: Option<String>,
    /// Where the pattern is written.
    pub span: Span,
    /// The definitions the pattern introduces, in source order.
    pub bindings: Vec<DefId>,
    /// The arm's body.
    pub body: ExprId,
}

/// One variant of an enum, with the payload it carries.
#[derive(Debug, Clone)]
pub struct Variant {
    /// The variant's name.
    pub name: String,
    /// The payload's fields, in source order. Empty for a variant that carries
    /// nothing.
    pub fields: Vec<Field>,
    /// Where it is written.
    pub span: Span,
}

/// A definition: the thing a name resolves to.
#[derive(Debug, Clone)]
pub struct Def {
    /// The name, as written.
    pub name: String,
    /// What kind of definition it is.
    pub kind: DefKind,
    /// Where it is written.
    pub span: Span,
    /// The scope it lives in.
    pub scope: ScopeId,
    /// Its declared type, if one is written down.
    pub ty: Option<TypeRef>,
    /// Its value or body, if it has one.
    pub value: Option<ExprId>,
    /// Its parameters, in source order, for anything callable.
    pub parameters: Vec<DefId>,
    /// Its fields, for a `struct`.
    pub fields: Vec<Field>,
    /// Its variants, for an `enum`.
    pub variants: Vec<Variant>,
}

impl Def {
    /// The namespace this definition lives in.
    #[must_use]
    pub const fn namespace(&self) -> Namespace {
        self.kind.namespace()
    }

    /// Whether this definition is the program's entry point.
    #[must_use]
    pub fn is_main(&self) -> bool {
        self.kind == DefKind::Function && self.name == "main"
    }
}

/// What a name resolved to.
///
/// Unresolved is a *state*, not an absence: the name stays in the HIR, marked,
/// so a later stage can keep working and a diagnostic has something to point at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// Resolved to a definition.
    Resolved(DefId),
    /// Not resolved; a diagnostic was reported at its span.
    Unresolved,
}

impl Resolution {
    /// The definition, if the name resolved.
    #[must_use]
    pub const fn resolved(self) -> Option<DefId> {
        match self {
            Resolution::Resolved(id) => Some(id),
            Resolution::Unresolved => None,
        }
    }

    /// Whether the name resolved.
    #[must_use]
    pub const fn is_resolved(self) -> bool {
        matches!(self, Resolution::Resolved(_))
    }
}

/// Every name reference resolution saw, kept for dumps, tests and later tools.
#[derive(Debug, Clone)]
pub struct Reference {
    /// Where the name is written.
    pub span: Span,
    /// The name as written, joined with `::` where it has segments.
    pub name: String,
    /// What it resolved to.
    pub target: Resolution,
    /// The namespace it was looked up in.
    pub namespace: Namespace,
}

/// A literal, as far as resolution is concerned.
///
/// The value is not computed here: that is the type checker's and the
/// interpreter's job, and computing it in this layer would put arithmetic in the
/// wrong place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralKind {
    /// An integer literal.
    Int,
    /// A floating-point literal.
    Float,
    /// A text literal.
    Text,
    /// `true` or `false`.
    Bool,
}

/// An expression, with the syntax that carries no meaning removed.
#[derive(Debug, Clone)]
pub enum ExprKind {
    /// A literal.
    Literal(LiteralKind),
    /// A name, and what it resolved to.
    Path {
        /// The name as written.
        name: String,
        /// What it resolved to.
        target: Resolution,
    },
    /// `f(x)`.
    Call {
        /// The expression being called.
        callee: ExprId,
        /// The arguments, in source order.
        arguments: Vec<ExprId>,
    },
    /// `a.b`.
    Field {
        /// The value being read from.
        receiver: ExprId,
        /// The field's name.
        name: String,
    },
    /// `a[i]`.
    Index {
        /// The value being indexed.
        base: ExprId,
        /// The index expression.
        index: ExprId,
    },
    /// `-a`, `!a`.
    Unary {
        /// The operator, as written.
        operator: String,
        /// The operand.
        operand: ExprId,
    },
    /// `a + b`, and every other binary operator.
    Binary {
        /// The operator, as written.
        operator: String,
        /// The left operand.
        left: ExprId,
        /// The right operand.
        right: ExprId,
    },
    /// A block, whose last expression is its value.
    Block {
        /// The statements, in source order.
        statements: Vec<StmtId>,
        /// The block's value, if it has one.
        value: Option<ExprId>,
    },
    /// `if … { … } else { … }`.
    If {
        /// The condition.
        condition: ExprId,
        /// The `then` block.
        then_block: ExprId,
        /// The `else` branch, which may be another `if`.
        else_branch: Option<ExprId>,
    },
    /// `match … { … }`.
    Match {
        /// The value being matched.
        scrutinee: ExprId,
        /// The arms, in source order.
        arms: Vec<Arm>,
    },
    /// `ask Name { … }`.
    Ask {
        /// The model or agent being asked.
        agent: Resolution,
        /// The prompt block.
        prompt: Option<ExprId>,
    },
    /// `verify name with Verifier`.
    Verify {
        /// The value being verified.
        value: ExprId,
        /// The verifier.
        verifier: Resolution,
    },
    /// `delegate Name { … } with budget: …`.
    Delegate {
        /// The agent receiving the work.
        agent: Resolution,
        /// The request block.
        request: Option<ExprId>,
    },
}

/// An expression and where it is.
#[derive(Debug, Clone)]
pub struct Expr {
    /// What the expression is.
    pub kind: ExprKind,
    /// Where it is written.
    pub span: Span,
}

/// A statement.
#[derive(Debug, Clone)]
pub enum StmtKind {
    /// `let name = value;`, with or without an annotation.
    Let {
        /// The definition the statement introduces; `None` when the parser
        /// could not read a name, in which case it already reported why.
        def: Option<DefId>,
    },
    /// An expression used for its effect.
    Expression(ExprId),
}

/// A statement and where it is.
#[derive(Debug, Clone)]
pub struct Stmt {
    /// What the statement is.
    pub kind: StmtKind,
    /// Where it is written.
    pub span: Span,
}

/// A type reference: resolved as far as names go, and no further.
#[derive(Debug, Clone)]
pub struct TypeRef {
    /// The name as written.
    pub name: String,
    /// What it resolved to.
    pub target: Resolution,
    /// Its type arguments, in source order.
    pub arguments: Vec<TypeRef>,
    /// Where it is written.
    pub span: Span,
}

/// One scope: the names visible at one place, and the scope around it.
#[derive(Debug, Clone)]
pub struct Scope {
    /// The enclosing scope, if there is one.
    pub parent: Option<ScopeId>,
    /// What this scope is for, for diagnostics and dumps.
    pub description: &'static str,
    /// The names declared here, in declaration order.
    pub entries: Vec<(String, Namespace, DefId)>,
}

/// The lowering result: the HIR, and everything resolution found wrong with it.
#[derive(Debug)]
pub struct HirResult {
    /// The resolved representation.
    pub hir: Hir,
    /// Diagnostics from resolution.
    pub diagnostics: Diagnostics,
}

/// The semantic representation of one file.
#[derive(Debug)]
pub struct Hir {
    defs: Vec<Def>,
    exprs: Vec<Expr>,
    stmts: Vec<Stmt>,
    scopes: Vec<Scope>,
    references: Vec<Reference>,
    root: ScopeId,
}

impl Hir {
    /// Every definition, in creation order: the file's items first, then the
    /// definitions inside them.
    #[must_use]
    pub fn defs(&self) -> &[Def] {
        &self.defs
    }

    /// Looks a definition up.
    #[must_use]
    pub fn def(&self, id: DefId) -> Option<&Def> {
        self.defs.get(id.index() as usize)
    }

    /// Looks an expression up.
    #[must_use]
    pub fn expr(&self, id: ExprId) -> Option<&Expr> {
        self.exprs.get(id.index() as usize)
    }

    /// Looks a statement up.
    #[must_use]
    pub fn statement(&self, id: StmtId) -> Option<&Stmt> {
        self.stmts.get(id.index() as usize)
    }

    /// Looks a scope up.
    #[must_use]
    pub fn scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id.index() as usize)
    }

    /// The file's scope.
    #[must_use]
    pub const fn root_scope(&self) -> ScopeId {
        self.root
    }

    /// Every name reference, in lowering order.
    #[must_use]
    pub fn references(&self) -> &[Reference] {
        &self.references
    }

    /// The file's top-level definitions, in source order.
    #[must_use]
    pub fn items(&self) -> Vec<&Def> {
        let mut items: Vec<&Def> = self
            .defs
            .iter()
            .filter(|def| def.scope == self.root && def.kind != DefKind::BuiltinType)
            .collect();
        items.sort_by_key(|def| def.span.start().get());
        items
    }

    /// The definition `name` resolves to from `scope`, if any.
    #[must_use]
    pub fn visible_from(&self, scope: ScopeId, name: &str, namespace: Namespace) -> Option<DefId> {
        let mut current = Some(scope);
        while let Some(id) = current {
            let scope = &self.scopes[id.index() as usize];
            if let Some((_, _, def)) = scope
                .entries
                .iter()
                .rev()
                .find(|(entry, entry_namespace, _)| entry == name && *entry_namespace == namespace)
            {
                return Some(*def);
            }
            current = scope.parent;
        }
        None
    }
}

/// Lowers a tree and resolves every name in it.
///
/// The HIR is produced even when resolution failed: an unresolved name is a
/// marked state, not a hole, so a consumer can keep asking questions about the
/// rest of the file.
#[must_use]
pub fn lower(tree: &SyntaxTree, source: SourceId) -> HirResult {
    let mut lowerer = Lowerer::new(source);
    lowerer.run(tree);
    HirResult {
        hir: Hir {
            defs: lowerer.defs,
            exprs: lowerer.exprs,
            stmts: lowerer.stmts,
            scopes: lowerer.scopes,
            references: lowerer.references,
            root: lowerer.root,
        },
        diagnostics: lowerer.diagnostics,
    }
}

/// Renders the resolutions of a compilation in a stable, diffable form.
///
/// The format is documented in `tests/conformance/README.md`:
///
/// ```text
/// # nudo-hir v1
/// def 0005 function `main` 1:1
/// ref 2:9 value `main` -> def 0005 function `main`
/// ```
///
/// Built-in types are left out: they have no place in the source, and listing
/// nine of them in every dump would bury what the case is about.
#[must_use]
pub fn dump_resolutions(hir: &Hir, source: &SourceFile) -> String {
    let mut out = String::from("# nudo-hir v1\n");
    for (index, def) in hir.defs.iter().enumerate() {
        if def.kind == DefKind::BuiltinType {
            continue;
        }
        let start = source.line_col(def.span.start());
        out.push_str(&format!(
            "def {index:04} {} `{}` {start}\n",
            def.kind.as_str(),
            def.name,
        ));
    }
    let mut references: Vec<&Reference> = hir.references.iter().collect();
    references.sort_by_key(|reference| reference.span.start().get());
    for reference in references {
        let start = source.line_col(reference.span.start());
        let target = match reference.target {
            Resolution::Resolved(id) => hir.def(id).map_or_else(
                || "unresolved".to_string(),
                |def| {
                    format!(
                        "def {:04} {} `{}`",
                        index_of(hir, def),
                        def.kind.as_str(),
                        def.name
                    )
                },
            ),
            Resolution::Unresolved => "unresolved".to_string(),
        };
        out.push_str(&format!(
            "ref {start} {} `{}` -> {target}\n",
            reference.namespace.as_str(),
            reference.name,
        ));
    }
    out
}

fn index_of(hir: &Hir, def: &Def) -> usize {
    hir.defs
        .iter()
        .position(|candidate| candidate.span == def.span && candidate.name == def.name)
        .unwrap_or(0)
}

struct Lowerer {
    source: SourceId,
    defs: Vec<Def>,
    exprs: Vec<Expr>,
    stmts: Vec<Stmt>,
    scopes: Vec<Scope>,
    references: Vec<Reference>,
    diagnostics: Diagnostics,
    root: ScopeId,
    scope: ScopeId,
}

impl Lowerer {
    fn new(source: SourceId) -> Self {
        Lowerer {
            source,
            defs: Vec::new(),
            exprs: Vec::new(),
            stmts: Vec::new(),
            scopes: vec![Scope {
                parent: None,
                description: "file",
                entries: Vec::new(),
            }],
            references: Vec::new(),
            diagnostics: Diagnostics::new(),
            root: ScopeId(0),
            scope: ScopeId(0),
        }
    }

    fn run(&mut self, tree: &SyntaxTree) {
        // Built-in types exist before anything else, so a program can name
        // `Int` without declaring it. They are declared in the file scope and
        // a program cannot redefine them: the duplicate rule catches it.
        for builtin in BUILTIN_TYPES {
            self.declare(
                (*builtin).to_string(),
                DefKind::BuiltinType,
                Span::new(nudo_span::BytePos::new(0), nudo_span::BytePos::new(0)),
                None,
            );
        }

        let root = tree.root();
        let items: Vec<SyntaxNode<'_>> = root.child_nodes();

        // Pass 1 hoists every item, so a function may call one declared below
        // it. This is the whole reason lowering has two passes.
        for item in &items {
            self.declare_item(*item);
        }

        // Pass 2 lowers the bodies, now that every name in the file is known.
        for item in &items {
            self.lower_item_body(*item);
        }
    }

    // ---------------------------------------------------------------- items --

    fn declare_item(&mut self, node: SyntaxNode<'_>) {
        let Some(kind) = item_kind(node.kind()) else {
            return;
        };
        let Some(name) = item_name_token(node) else {
            // The parser already reported the missing name.
            return;
        };
        let ty = match kind {
            DefKind::Const | DefKind::Binding => self.lower_annotation(node),
            _ => None,
        };
        self.declare(name.text().to_string(), kind, name.span(), ty);
    }

    fn lower_item_body(&mut self, node: SyntaxNode<'_>) {
        let Some(kind) = item_kind(node.kind()) else {
            return;
        };
        let Some(name) = item_name_token(node) else {
            return;
        };
        let Some(def_id) = self.find_def(name.text(), name.span()) else {
            return;
        };

        let outer = self.scope;
        self.scope = self.push_scope(outer, "declaration");
        // A declaration's own type parameters and parameters are visible inside
        // it, and nowhere else.
        for list in node.children_of_kind(SyntaxKind::GenericParameterList) {
            for token in list
                .child_tokens()
                .into_iter()
                .filter(|token| token.kind() == SyntaxKind::Ident)
            {
                self.declare(
                    token.text().to_string(),
                    DefKind::TypeParameter,
                    token.span(),
                    None,
                );
            }
        }
        // A declaration's parameters belong to the declaration, not to the
        // file: they are declared here, in its own scope, with their
        // annotations lowered where its type parameters are already visible.
        let mut parameters = Vec::new();
        if let Some(list) = node.child_of_kind(SyntaxKind::ParameterList) {
            for parameter in list.children_of_kind(SyntaxKind::Parameter) {
                let Some(token) = name_token(parameter) else {
                    continue;
                };
                let annotation = self.lower_annotation(parameter);
                parameters.push(self.declare(
                    token.text().to_string(),
                    DefKind::Parameter,
                    token.span(),
                    annotation,
                ));
            }
        }
        self.defs[def_id.index() as usize].parameters = parameters;
        self.lower_nominal_structure(node, def_id);

        // Return types, annotations and bodies are lowered after the parameters
        // are in scope, because they may name them.
        let return_type = node
            .child_nodes()
            .into_iter()
            .find(|child| is_type(child.kind()))
            .map(|child| self.lower_type(child));
        if let Some(return_type) = return_type {
            self.defs[def_id.index() as usize].ty = Some(return_type);
        }

        if let Some(block) = node.child_of_kind(SyntaxKind::Block) {
            let body = self.lower_block(block);
            self.defs[def_id.index() as usize].value = body;
        } else if matches!(kind, DefKind::Const | DefKind::Binding) {
            let value = node
                .child_nodes()
                .into_iter()
                .find(|child| is_expression(child.kind()))
                .and_then(|child| self.lower_expression(child));
            self.defs[def_id.index() as usize].value = value;
        }
        self.scope = outer;
    }

    /// Records what a `struct` or an `enum` is made of.
    ///
    /// Fields and variants are lowered in the declaration's own scope, so a type
    /// parameter of the declaration is visible inside them — which is what makes
    /// `struct Pair<A> { first: A }` ready for instantiation later, without this
    /// pass having to know anything about instantiation.
    fn lower_nominal_structure(&mut self, node: SyntaxNode<'_>, def_id: DefId) {
        match node.kind() {
            SyntaxKind::StructDecl => {
                let fields = self.lower_fields(node);
                self.defs[def_id.index() as usize].fields = fields;
            }
            SyntaxKind::EnumDecl => {
                let mut variants = Vec::new();
                for variant in node.children_of_kind(SyntaxKind::Variant) {
                    let Some(name) = name_token(variant) else {
                        continue;
                    };
                    variants.push(Variant {
                        name: name.text().to_string(),
                        fields: self.lower_fields(variant),
                        span: name.span(),
                    });
                }
                self.defs[def_id.index() as usize].variants = variants;
            }
            _ => {}
        }
    }

    /// The fields written directly inside `node`.
    fn lower_fields(&mut self, node: SyntaxNode<'_>) -> Vec<Field> {
        let mut fields = Vec::new();
        for field in node.children_of_kind(SyntaxKind::Field) {
            let Some(name) = name_token(field) else {
                continue;
            };
            fields.push(Field {
                name: name.text().to_string(),
                ty: self.lower_annotation(field),
                span: name.span(),
            });
        }
        fields
    }

    fn find_def(&self, name: &str, span: Span) -> Option<DefId> {
        self.defs
            .iter()
            .position(|def| def.name == name && def.span == span)
            .map(|index| DefId(index as u32))
    }

    // ---------------------------------------------------------------- types --

    fn lower_annotation(&mut self, node: SyntaxNode<'_>) -> Option<TypeRef> {
        node.child_nodes()
            .into_iter()
            .find(|child| is_type(child.kind()))
            .map(|child| self.lower_type(child))
    }

    fn lower_type(&mut self, node: SyntaxNode<'_>) -> TypeRef {
        let span = node.span();
        match node.kind() {
            SyntaxKind::PathType | SyntaxKind::GenericType => {
                let arguments = node
                    .child_nodes()
                    .into_iter()
                    .filter(|child| is_type(child.kind()))
                    .map(|child| self.lower_type(child))
                    .collect();
                let name = node
                    .child_of_kind(SyntaxKind::Path)
                    .map_or_else(String::new, |path| path_text(&path));
                let target = self.resolve(&name, Namespace::Type, span);
                TypeRef {
                    name,
                    target,
                    arguments,
                    span,
                }
            }
            SyntaxKind::SequenceType => {
                let inner = node
                    .child_nodes()
                    .into_iter()
                    .find(|child| is_type(child.kind()));
                match inner {
                    Some(inner) => {
                        let inner = self.lower_type(inner);
                        TypeRef {
                            name: format!("[{}]", inner.name),
                            target: Resolution::Unresolved,
                            arguments: vec![inner],
                            span,
                        }
                    }
                    None => TypeRef {
                        name: "[]".to_string(),
                        target: Resolution::Unresolved,
                        arguments: Vec::new(),
                        span,
                    },
                }
            }
            SyntaxKind::OptionalType => {
                let inner = node
                    .child_nodes()
                    .into_iter()
                    .find(|child| is_type(child.kind()))
                    .map(|child| self.lower_type(child));
                match inner {
                    Some(inner) => TypeRef {
                        name: format!("{}?", inner.name),
                        target: Resolution::Unresolved,
                        arguments: vec![inner],
                        span,
                    },
                    None => TypeRef {
                        name: "?".to_string(),
                        target: Resolution::Unresolved,
                        arguments: Vec::new(),
                        span,
                    },
                }
            }
            SyntaxKind::FunctionType => {
                let mut types: Vec<TypeRef> = node
                    .child_nodes()
                    .into_iter()
                    .filter(|child| is_type(child.kind()))
                    .map(|child| self.lower_type(child))
                    .collect();
                let result = types.pop();
                let parameters = types;
                let result_name = result
                    .as_ref()
                    .map_or_else(String::new, |it| it.name.clone());
                let mut arguments = parameters;
                if let Some(result) = result {
                    arguments.push(result);
                }
                TypeRef {
                    name: format!("Fn(…) -> {result_name}"),
                    target: Resolution::Unresolved,
                    arguments,
                    span,
                }
            }
            _ => TypeRef {
                name: node.text().trim().to_string(),
                target: Resolution::Unresolved,
                arguments: Vec::new(),
                span,
            },
        }
    }

    // ---------------------------------------------------------- expressions --

    fn lower_block(&mut self, node: SyntaxNode<'_>) -> Option<ExprId> {
        let outer = self.scope;
        self.scope = self.push_scope(outer, "block");
        let mut statements: Vec<StmtId> = Vec::new();
        let mut value = None;
        for child in node.children() {
            let Some(child) = child.into_node() else {
                continue;
            };
            match child.kind() {
                SyntaxKind::Binding => {
                    let def = self.lower_binding(child);
                    let id = StmtId(self.stmts.len() as u32);
                    self.stmts.push(Stmt {
                        kind: StmtKind::Let { def },
                        span: child.span(),
                    });
                    statements.push(id);
                }
                SyntaxKind::ExpressionStatement => {
                    if let Some(expression) = child
                        .child_nodes()
                        .into_iter()
                        .find(|node| is_expression(node.kind()))
                        .and_then(|node| self.lower_expression(node))
                    {
                        let id = StmtId(self.stmts.len() as u32);
                        self.stmts.push(Stmt {
                            kind: StmtKind::Expression(expression),
                            span: child.span(),
                        });
                        statements.push(id);
                    }
                }
                SyntaxKind::Error => {}
                _ => {
                    if is_expression(child.kind()) {
                        value = self.lower_expression(child);
                    }
                }
            }
        }
        self.scope = outer;
        Some(self.push_expr(ExprKind::Block { statements, value }, node.span()))
    }

    /// Lowers a `let`. The initialiser is lowered *before* the new name is in
    /// scope, so `let x = x;` names the outer `x`.
    fn lower_binding(&mut self, node: SyntaxNode<'_>) -> Option<DefId> {
        let ty = self.lower_annotation(node);
        let value = node
            .child_nodes()
            .into_iter()
            .find(|child| is_expression(child.kind()))
            .and_then(|child| self.lower_expression(child));
        let name = name_token(node)?;
        let def = self.declare(name.text().to_string(), DefKind::Binding, name.span(), ty);
        self.defs[def.index() as usize].value = value;
        Some(def)
    }

    fn lower_expression(&mut self, node: SyntaxNode<'_>) -> Option<ExprId> {
        let span = node.span();
        let kind = match node.kind() {
            SyntaxKind::LiteralExpression => {
                let token = node
                    .child_tokens()
                    .into_iter()
                    .find(|token| !token.kind().is_trivia() && token.kind() != SyntaxKind::Eof)?;
                ExprKind::Literal(literal_kind(token.kind())?)
            }
            SyntaxKind::PathExpression => {
                let path = node.child_of_kind(SyntaxKind::Path)?;
                let name = path_text(&path);
                let target = self.resolve(&name, Namespace::Value, path.span());
                ExprKind::Path { name, target }
            }
            SyntaxKind::CallExpression => {
                let callee = node
                    .child_nodes()
                    .into_iter()
                    .find(|child| is_expression(child.kind()))
                    .and_then(|child| self.lower_expression(child))?;
                let arguments = node
                    .child_of_kind(SyntaxKind::ArgumentList)
                    .map(|list| {
                        list.child_nodes()
                            .into_iter()
                            .filter(|child| is_expression(child.kind()))
                            .filter_map(|child| self.lower_expression(child))
                            .collect()
                    })
                    .unwrap_or_default();
                ExprKind::Call { callee, arguments }
            }
            SyntaxKind::FieldExpression => {
                let receiver = node
                    .child_nodes()
                    .into_iter()
                    .find(|child| is_expression(child.kind()))
                    .and_then(|child| self.lower_expression(child))?;
                let name = node
                    .child_tokens()
                    .into_iter()
                    .rfind(|token| token.kind() == SyntaxKind::Ident)
                    .map_or_else(String::new, |token| token.text().to_string());
                ExprKind::Field { receiver, name }
            }
            SyntaxKind::IndexExpression => {
                let mut parts = node
                    .child_nodes()
                    .into_iter()
                    .filter(|child| is_expression(child.kind()))
                    .filter_map(|child| self.lower_expression(child));
                let base = parts.next()?;
                let index = parts.next()?;
                ExprKind::Index { base, index }
            }
            SyntaxKind::UnaryExpression => {
                let operator = node
                    .child_tokens()
                    .into_iter()
                    .find(|token| !token.kind().is_trivia())
                    .map_or_else(String::new, |token| token.text().to_string());
                let operand = node
                    .child_nodes()
                    .into_iter()
                    .find(|child| is_expression(child.kind()))
                    .and_then(|child| self.lower_expression(child))?;
                ExprKind::Unary { operator, operand }
            }
            SyntaxKind::BinaryExpression => {
                let mut parts = node
                    .child_nodes()
                    .into_iter()
                    .filter(|child| is_expression(child.kind()))
                    .filter_map(|child| self.lower_expression(child));
                let left = parts.next()?;
                let right = parts.next()?;
                let operator = node
                    .child_tokens()
                    .into_iter()
                    .find(|token| !token.kind().is_trivia())
                    .map_or_else(String::new, |token| token.text().to_string());
                ExprKind::Binary {
                    operator,
                    left,
                    right,
                }
            }
            // Parentheses are syntax, not meaning: `(a)` and `a` are the same
            // HIR. This is the clearest example of what lowering removes.
            SyntaxKind::ParenthesizedExpression => {
                let inner = node
                    .child_nodes()
                    .into_iter()
                    .find(|child| is_expression(child.kind()))?;
                return self.lower_expression(inner);
            }
            SyntaxKind::Block => return self.lower_block(node),
            SyntaxKind::IfExpression => {
                let condition = node
                    .child_nodes()
                    .into_iter()
                    .find(|child| is_expression(child.kind()))
                    .and_then(|child| self.lower_expression(child))?;
                let branches: Vec<ExprId> = node
                    .child_nodes()
                    .into_iter()
                    .filter(|child| {
                        matches!(child.kind(), SyntaxKind::Block | SyntaxKind::IfExpression)
                    })
                    .filter_map(|child| self.lower_expression(child))
                    .collect();
                let then_block = *branches.first()?;
                let else_branch = branches.get(1).copied();
                ExprKind::If {
                    condition,
                    then_block,
                    else_branch,
                }
            }
            SyntaxKind::MatchExpression => {
                let scrutinee = node
                    .child_nodes()
                    .into_iter()
                    .find(|child| is_expression(child.kind()))
                    .and_then(|child| self.lower_expression(child))?;
                // An arm whose body did not lower is dropped, and that is safe
                // for the wrong reason to be careless about: a missing body is a
                // parse error, and no later stage runs on a file the parser
                // rejected. The checker's exhaustiveness rule therefore never
                // sees a silently missing arm.
                let arms: Vec<Arm> = node
                    .children_of_kind(SyntaxKind::MatchArm)
                    .into_iter()
                    .filter_map(|arm| self.lower_arm(arm))
                    .collect();
                ExprKind::Match { scrutinee, arms }
            }
            SyntaxKind::AskExpression => {
                let agent = self.resolve_line_agent(node);
                let prompt = self.lower_child_block(node);
                ExprKind::Ask { agent, prompt }
            }
            SyntaxKind::VerifyExpression => {
                let value = node
                    .child_nodes()
                    .into_iter()
                    .find(|child| is_expression(child.kind()))
                    .and_then(|child| self.lower_expression(child))?;
                let verifier = node
                    .child_nodes()
                    .into_iter()
                    .find(|child| child.kind() == SyntaxKind::Path)
                    .map_or(Resolution::Unresolved, |path| {
                        let name = path_text(&path);
                        self.resolve(&name, Namespace::Value, path.span())
                    });
                ExprKind::Verify { value, verifier }
            }
            SyntaxKind::DelegateExpression => {
                let agent = self.resolve_line_agent(node);
                let request = self.lower_child_block(node);
                ExprKind::Delegate { agent, request }
            }
            _ => return None,
        };
        Some(self.push_expr(kind, span))
    }

    fn resolve_line_agent(&mut self, node: SyntaxNode<'_>) -> Resolution {
        node.child_nodes()
            .into_iter()
            .find(|child| child.kind() == SyntaxKind::Path)
            .map_or(Resolution::Unresolved, |path| {
                let name = path_text(&path);
                self.resolve(&name, Namespace::Value, path.span())
            })
    }

    fn lower_child_block(&mut self, node: SyntaxNode<'_>) -> Option<ExprId> {
        node.child_nodes()
            .into_iter()
            .find(|child| child.kind() == SyntaxKind::Block)
            .and_then(|block| self.lower_block(block))
    }

    /// Lowers one `match` arm.
    ///
    /// Only sub-patterns become bindings. A bare head name is a variant or a new
    /// binding, and which one it is needs the scrutinee's type, so the head is
    /// left for M3.2 rather than guessed from its spelling.
    fn lower_arm(&mut self, node: SyntaxNode<'_>) -> Option<Arm> {
        let outer = self.scope;
        self.scope = self.push_scope(outer, "match-arm");

        let mut head = None;
        let mut span = node
            .child_tokens()
            .first()
            .map_or_else(zero_span, |token| token.span());
        let mut bindings = Vec::new();

        if let Some(pattern) = node.child_of_kind(SyntaxKind::Pattern) {
            if let Some(token) = item_name_token(pattern) {
                head = Some(token.text().to_string());
                span = token.span();
            }
            for nested in pattern.children_of_kind(SyntaxKind::Pattern) {
                if let Some(token) = item_name_token(nested) {
                    bindings.push(self.declare(
                        token.text().to_string(),
                        DefKind::Local,
                        token.span(),
                        None,
                    ));
                }
            }
        }

        let body = node
            .child_nodes()
            .into_iter()
            .find(|child| is_expression(child.kind()))
            .and_then(|child| self.lower_expression(child));
        self.scope = outer;
        Some(Arm {
            head,
            span,
            bindings,
            body: body?,
        })
    }

    fn push_expr(&mut self, kind: ExprKind, span: Span) -> ExprId {
        let id = ExprId(self.exprs.len() as u32);
        self.exprs.push(Expr { kind, span });
        id
    }

    // ------------------------------------------------------------- scopes ---

    fn push_scope(&mut self, parent: ScopeId, description: &'static str) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);
        self.scopes.push(Scope {
            parent: Some(parent),
            description,
            entries: Vec::new(),
        });
        id
    }

    fn declare(&mut self, name: String, kind: DefKind, span: Span, ty: Option<TypeRef>) -> DefId {
        let namespace = kind.namespace();
        let id = DefId(self.defs.len() as u32);
        self.defs.push(Def {
            name: name.clone(),
            kind,
            span,
            scope: self.scope,
            ty,
            value: None,
            parameters: Vec::new(),
            fields: Vec::new(),
            variants: Vec::new(),
        });
        self.insert(name, namespace, id);
        id
    }

    fn insert(&mut self, name: String, namespace: Namespace, def: DefId) {
        let scope = self.scope;
        let previous = self.scopes[scope.index() as usize]
            .entries
            .iter()
            .rev()
            .find(|(entry, entry_namespace, _)| *entry == name && *entry_namespace == namespace)
            .map(|(_, _, def)| *def);
        if let Some(previous) = previous {
            let span = self.defs[def.index() as usize].span;
            let previous_span = self.defs[previous.index() as usize].span;
            let previous_kind = self.defs[previous.index() as usize].kind;
            let kind = self.defs[def.index() as usize].kind;
            self.diagnostics.push(
                Diagnostic::error(
                    codes::DUPLICATE_NAME,
                    format!("`{name}` is already defined in this scope"),
                )
                .with_location(self.source, span)
                .with_note(format!(
                    "the first definition is the {} at byte {}",
                    previous_kind.as_str(),
                    previous_span.start().get()
                ))
                .with_note(format!(
                    "this one is a {}; a nested scope may shadow, the same scope may not",
                    kind.as_str()
                )),
            );
        }
        self.scopes[scope.index() as usize]
            .entries
            .push((name, namespace, def));
    }

    fn resolve(&mut self, name: &str, namespace: Namespace, span: Span) -> Resolution {
        if let Some(def) = self.lookup(name, namespace) {
            self.references.push(Reference {
                span,
                name: name.to_string(),
                target: Resolution::Resolved(def),
                namespace,
            });
            return Resolution::Resolved(def);
        }

        let other = match namespace {
            Namespace::Type => Namespace::Value,
            Namespace::Value => Namespace::Type,
        };
        if let Some(def) = self.lookup(name, other) {
            let kind = self.defs[def.index() as usize].kind;
            let diagnostic = Diagnostic::error(
                codes::WRONG_NAMESPACE,
                format!(
                    "`{name}` is a {}, not a {}",
                    kind.as_str(),
                    namespace.as_str()
                ),
            )
            .with_location(self.source, span)
            .with_help(format!("use it where a {} is expected", other.as_str()));
            self.diagnostics.push(diagnostic);
            self.references.push(Reference {
                span,
                name: name.to_string(),
                target: Resolution::Unresolved,
                namespace,
            });
            return Resolution::Unresolved;
        }

        let mut diagnostic =
            Diagnostic::error(codes::UNRESOLVED_NAME, format!("unresolved name `{name}`"))
                .with_location(self.source, span);
        if name.contains("::") {
            diagnostic = diagnostic
                .with_note("a path with `::` reaches into a module, and modules are planned (M10)");
        }
        self.diagnostics.push(diagnostic);
        self.references.push(Reference {
            span,
            name: name.to_string(),
            target: Resolution::Unresolved,
            namespace,
        });
        Resolution::Unresolved
    }

    fn lookup(&self, name: &str, namespace: Namespace) -> Option<DefId> {
        let mut current = Some(self.scope);
        while let Some(id) = current {
            let scope = &self.scopes[id.index() as usize];
            if let Some((_, _, def)) = scope
                .entries
                .iter()
                .rev()
                .find(|(entry, entry_namespace, _)| entry == name && *entry_namespace == namespace)
            {
                return Some(*def);
            }
            current = scope.parent;
        }
        None
    }
}

/// The built-in types, in a fixed order.
const BUILTIN_TYPES: &[&str] = &[
    "Int",
    "Float",
    "Bool",
    "Text",
    "Unit",
    "Result",
    "Generated",
    "Verified",
    "Fn",
];

fn item_kind(kind: SyntaxKind) -> Option<DefKind> {
    match kind {
        SyntaxKind::FunctionDecl => Some(DefKind::Function),
        SyntaxKind::StructDecl => Some(DefKind::Struct),
        SyntaxKind::EnumDecl => Some(DefKind::Enum),
        SyntaxKind::AgentDecl => Some(DefKind::Agent),
        SyntaxKind::TaskDecl => Some(DefKind::Task),
        SyntaxKind::ToolDecl => Some(DefKind::Tool),
        SyntaxKind::ModelDecl => Some(DefKind::Model),
        SyntaxKind::ConstDecl => Some(DefKind::Const),
        SyntaxKind::Binding => Some(DefKind::Binding),
        _ => None,
    }
}

/// The token an item is named by.
///
/// A tool is named by a path, so its name is the path's first segment — and a
/// pattern binding has the same shape: `Failed(reason)` binds `reason`, whose
/// token lives inside a `Path`, not as a token of the pattern itself.
fn zero_span() -> Span {
    Span::new(nudo_span::BytePos::new(0), nudo_span::BytePos::new(0))
}

fn item_name_token(node: SyntaxNode<'_>) -> Option<SyntaxToken<'_>> {
    name_token(node).or_else(|| {
        node.child_of_kind(SyntaxKind::Path)
            .and_then(|path| path.child_tokens().into_iter().next())
    })
}

fn name_token(node: SyntaxNode<'_>) -> Option<SyntaxToken<'_>> {
    node.child_tokens()
        .into_iter()
        .find(|token| token.kind() == SyntaxKind::Ident)
}

/// A path's text, with `::` between its segments.
fn path_text(path: &SyntaxNode<'_>) -> String {
    let mut text = String::new();
    for token in path.child_tokens() {
        if token.kind() == SyntaxKind::Ident {
            if !text.is_empty() {
                text.push_str("::");
            }
            text.push_str(token.text());
        }
    }
    text
}

fn literal_kind(kind: SyntaxKind) -> Option<LiteralKind> {
    match kind {
        SyntaxKind::IntLiteral => Some(LiteralKind::Int),
        SyntaxKind::FloatLiteral => Some(LiteralKind::Float),
        SyntaxKind::TextLiteral => Some(LiteralKind::Text),
        SyntaxKind::KeywordTrue | SyntaxKind::KeywordFalse => Some(LiteralKind::Bool),
        _ => None,
    }
}

fn is_type(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::PathType
            | SyntaxKind::GenericType
            | SyntaxKind::SequenceType
            | SyntaxKind::FunctionType
            | SyntaxKind::ParenthesizedType
            | SyntaxKind::OptionalType
    )
}

fn is_expression(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LiteralExpression
            | SyntaxKind::PathExpression
            | SyntaxKind::CallExpression
            | SyntaxKind::FieldExpression
            | SyntaxKind::IndexExpression
            | SyntaxKind::UnaryExpression
            | SyntaxKind::BinaryExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::Block
            | SyntaxKind::IfExpression
            | SyntaxKind::MatchExpression
            | SyntaxKind::AskExpression
            | SyntaxKind::VerifyExpression
            | SyntaxKind::DelegateExpression
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use nudo_source::SourceMap;

    struct Compilation {
        sources: SourceMap,
        id: SourceId,
    }

    impl Compilation {
        fn new(text: &str) -> Self {
            let mut sources = SourceMap::new();
            let id = sources.add("main.nudo", text);
            Compilation { sources, id }
        }

        fn run(&self) -> (nudo_parser::Parse, HirResult) {
            let file = self.sources.get(self.id).expect("just added");
            let parsed = nudo_parser::parse(file);
            assert!(!parsed.has_errors(), "the test input must parse cleanly");
            let hir = lower(parsed.tree(), self.id);
            (parsed, hir)
        }

        fn dump(&self, hir: &Hir) -> String {
            let file = self.sources.get(self.id).expect("just added");
            dump_resolutions(hir, file)
        }
    }

    fn codes(result: &HirResult) -> Vec<String> {
        result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code().id().to_string())
            .collect()
    }

    #[test]
    fn resolves_a_function_body_end_to_end() {
        let compilation = Compilation::new(
            "fn calculate(value: Int) -> Int {\n    let doubled = value + value;\n    doubled\n}\n",
        );
        let (_, result) = compilation.run();
        assert_eq!(codes(&result), Vec::<String>::new());
        let hir = &result.hir;

        let calculate = hir
            .items()
            .into_iter()
            .find(|def| def.name == "calculate")
            .expect("a function")
            .clone();
        assert_eq!(calculate.kind, DefKind::Function);
        assert_eq!(calculate.namespace(), Namespace::Value);
        assert_eq!(calculate.parameters.len(), 1);
        let value = hir.def(calculate.parameters[0]).expect("a parameter");
        assert_eq!(value.name, "value");
        assert_eq!(value.kind, DefKind::Parameter);

        let doubled = hir
            .defs()
            .iter()
            .find(|def| def.name == "doubled")
            .expect("a binding");
        assert_eq!(doubled.kind, DefKind::Binding);
        assert_eq!(
            hir.scope(doubled.scope).and_then(|scope| scope.parent),
            Some(value.scope),
            "the block scope is nested inside the declaration scope"
        );

        // The annotation and the return type both resolved to `Int`.
        let annotation = value.ty.as_ref().expect("an annotation");
        assert_eq!(annotation.name, "Int");
        assert!(annotation.target.is_resolved());

        // Every use of `value` points at the parameter, not at a string.
        let targets: Vec<Resolution> = hir
            .references()
            .iter()
            .filter(|reference| reference.name == "value")
            .map(|reference| reference.target)
            .collect();
        assert_eq!(targets.len(), 2);
        assert!(
            targets
                .iter()
                .all(|target| *target == Resolution::Resolved(calculate.parameters[0]))
        );
    }

    #[test]
    fn reports_an_unresolved_name() {
        let compilation = Compilation::new("fn main() {\n    let value = missing;\n}\n");
        let (_, result) = compilation.run();
        assert_eq!(codes(&result), vec!["NDO2001".to_string()]);
        let diagnostic = &result.diagnostics.as_slice()[0];
        assert!(diagnostic.message().contains("missing"));
        assert_eq!(result.hir.references().len(), 1);
        assert!(!result.hir.references()[0].target.is_resolved());
    }

    #[test]
    fn a_duplicate_in_one_scope_is_an_error_and_shadowing_is_not() {
        let compilation = Compilation::new("fn twice() {}\nfn twice() {}\n");
        let (_, result) = compilation.run();
        assert_eq!(codes(&result), vec!["NDO2002".to_string()]);

        let shadowing =
            Compilation::new("fn f(value: Int) -> Int {\n    let value = value;\n    value\n}\n");
        let (_, result) = shadowing.run();
        assert_eq!(codes(&result), Vec::<String>::new());
        // The initialiser names the outer `value`; the new binding is a second
        // definition, in a nested scope.
        let bindings: Vec<&Def> = result
            .hir
            .defs()
            .iter()
            .filter(|def| def.kind == DefKind::Binding)
            .collect();
        assert_eq!(bindings.len(), 1);
        let initialiser = &result.hir.references()[0];
        assert_ne!(
            initialiser.target,
            Resolution::Resolved(
                result
                    .hir
                    .defs()
                    .iter()
                    .position(|def| def.kind == DefKind::Binding)
                    .map(|index| DefId(index as u32))
                    .expect("a binding")
            )
        );
    }

    #[test]
    fn a_type_and_a_value_may_share_a_name() {
        let compilation = Compilation::new(
            "struct User {\n    name: Text\n}\n\nfn user() -> User {\n    user\n}\n",
        );
        let (_, result) = compilation.run();
        assert_eq!(codes(&result), Vec::<String>::new());
        let names: Vec<(&str, Namespace)> = result
            .hir
            .items()
            .into_iter()
            .map(|def| (def.name.as_str(), def.namespace()))
            .collect();
        assert!(names.contains(&("User", Namespace::Type)));
        assert!(names.contains(&("user", Namespace::Value)));
    }

    #[test]
    fn using_a_type_where_a_value_belongs_is_reported_as_such() {
        let compilation =
            Compilation::new("struct User {\n    name: Text\n}\n\nfn f() {\n    User\n}\n");
        let (_, result) = compilation.run();
        assert_eq!(codes(&result), vec!["NDO2003".to_string()]);
        let diagnostic = &result.diagnostics.as_slice()[0];
        assert!(diagnostic.message().contains("not a value"));
    }

    #[test]
    fn type_parameters_are_visible_inside_their_declaration_only() {
        let compilation = Compilation::new(
            "fn identity<T>(value: T) -> T {\n    value\n}\n\nfn other() {\n    T\n}\n",
        );
        let parsed_then_hir = compilation.run();
        // `T` resolves inside `identity`, and not inside `other`.
        assert_eq!(codes(&parsed_then_hir.1), vec!["NDO2001".to_string()]);
        assert!(
            parsed_then_hir
                .1
                .hir
                .defs()
                .iter()
                .any(|def| def.name == "T" && def.kind == DefKind::TypeParameter)
        );
    }

    #[test]
    fn parentheses_are_syntax_and_do_not_survive() {
        let compilation = Compilation::new("let plain = 1;\nlet wrapped = ((1));\n");
        let (_, result) = compilation.run();
        assert_eq!(codes(&result), Vec::<String>::new());
        // Two initialisers, both literals: `((1))` is one expression, not three.
        let literals = result
            .hir
            .defs()
            .iter()
            .filter(|def| def.kind == DefKind::Binding)
            .filter(|def| {
                def.value
                    .and_then(|value| result.hir.expr(value))
                    .is_some_and(|expr| matches!(expr.kind, ExprKind::Literal(LiteralKind::Int)))
            })
            .count();
        assert_eq!(literals, 2);
    }

    #[test]
    fn a_function_may_call_one_declared_below_it() {
        let compilation = Compilation::new(
            "fn first() -> Int {\n    second()\n}\n\nfn second() -> Int {\n    1\n}\n",
        );
        let (_, result) = compilation.run();
        assert_eq!(codes(&result), Vec::<String>::new());
        let reference = result
            .hir
            .references()
            .iter()
            .find(|reference| reference.name == "second")
            .expect("a reference");
        let target = reference.target.resolved().expect("resolved");
        assert_eq!(
            result.hir.def(target).expect("a definiiton").kind,
            DefKind::Function
        );
    }

    #[test]
    fn a_match_arm_records_what_it_matches() {
        let source = "enum Status {\n    Failed(reason: Text)\n    Ready\n}\n\nfn describe(status: Status) -> Text {\n    match status {\n        Failed(reason) => reason\n        Ready => \"ready\"\n    }\n}\n";
        let compilation = Compilation::new(source);
        let (_, resolved) = compilation.run();

        // The function's body is a block whose final expression is the match.
        let function = resolved
            .hir
            .defs()
            .iter()
            .find(|definition| definition.name == "describe")
            .expect("the function");
        let body = resolved
            .hir
            .expr(function.value.expect("a body"))
            .expect("a body");
        let ExprKind::Block {
            value: Some(match_id),
            ..
        } = body.kind
        else {
            panic!("the body is a block with a value");
        };
        let matched = resolved.hir.expr(match_id).expect("the match");
        let ExprKind::Match { arms, .. } = &matched.kind else {
            panic!("the value is a match");
        };

        assert_eq!(arms.len(), 2, "{arms:?}");
        assert_eq!(arms[0].head.as_deref(), Some("Failed"));
        assert_eq!(arms[0].bindings.len(), 1, "the payload is bound");
        assert_eq!(arms[1].head.as_deref(), Some("Ready"));
        assert!(
            arms[1].bindings.is_empty(),
            "a variant without payload binds nothing"
        );
    }

    #[test]
    fn the_dump_is_stable_and_names_what_resolved() {
        let compilation = Compilation::new("fn main() {\n    let value = 1;\n    value\n}\n");
        let (_, result) = compilation.run();
        let dump = compilation.dump(&result.hir);
        assert!(dump.starts_with("# nudo-hir v1\n"));
        assert!(dump.contains("def 0009 function `main` 1:4"), "{dump}");
        assert!(dump.contains("def 0010 binding `value` 2:9"), "{dump}");
        assert!(
            dump.contains("ref 3:5 value `value` -> def 0010 binding `value`"),
            "{dump}"
        );
    }

    #[test]
    fn a_match_arm_binds_its_sub_pattern() {
        let compilation = Compilation::new(
            "enum Status {\n    Failed(reason: Text)\n}\n\nfn f(status: Status) -> Text {\n    match status {\n        Failed(reason) => reason\n    }\n}\n",
        );
        let (_, result) = compilation.run();
        assert_eq!(codes(&result), Vec::<String>::new());
        let local = result
            .hir
            .defs()
            .iter()
            .find(|def| def.kind == DefKind::Local)
            .expect("a pattern binding");
        assert_eq!(local.name, "reason");
        // The arm's head is a variant, and resolving it needs a type: it is not
        // reported as unresolved, because that would be the wrong answer.
        assert!(
            !result
                .hir
                .references()
                .iter()
                .any(|reference| reference.name == "Failed")
        );
    }

    #[test]
    fn visible_from_walks_outwards() {
        let compilation = Compilation::new("fn main() {\n    let value = 1;\n}\n");
        let (_, result) = compilation.run();
        let hir = &result.hir;
        let def = hir
            .defs()
            .iter()
            .find(|def| def.kind == DefKind::Binding)
            .expect("a binding");
        let found = hir
            .visible_from(def.scope, "value", Namespace::Value)
            .expect("visible in its own scope");
        assert_eq!(hir.def(found).expect("a definition").name, "value");
        // The file scope holds the function, and the block is inside it.
        assert!(
            hir.visible_from(hir.root_scope(), "main", Namespace::Value)
                .is_some()
        );
    }
}
