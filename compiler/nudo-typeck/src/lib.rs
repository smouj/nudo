//! The type checker.
//!
//! ```text
//! .nudo → Lexer → Parser → SyntaxTree → AST → HIR → Typeck
//!                                                  compiler/nudo-typeck
//! ```
//!
//! # It consumes HIR, and only HIR
//!
//! This crate never looks at a syntax tree, at an AST, or at a byte of source.
//! That is not a style preference: it is what makes "is HIR enough?" a question
//! with a test. If a rule here needed to know where a parenthesis was, or what a
//! token looked like, then **HIR is missing a fact** and the fix belongs in
//! [`nudo_hir`], not in a second reader of the same file.
//!
//! What comes out is the type of every expression, the type of every definition,
//! and the diagnostics. Spans come from HIR, so a diagnostic points where the
//! resolver would have pointed.
//!
//! # What this checker checks today
//!
//! Deliberately bounded, with each supported rule exercised independently:
//!
//! * **Literal typing.** An integer literal is `Int`, a float is `Float`, `true`
//!   is `Bool`, text is `Text`.
//! * **Annotations.** `let x: Int = 1;` types the annotation and the value and
//!   compares them.
//! * **Named type arity.** Structs, enums, type parameters and intrinsic types
//!   require exactly their declared number of type arguments (`NDO2017`).
//! * **Blocks.** A block's type is its final expression's, or `Unit` when it has
//!   none — which is why a function with no written return type takes the type
//!   of its body, the same statement the specification makes read from the other
//!   end.
//! * **Names.** A path takes the type of the definition it resolved to, which is
//!   why resolution had to exist first.
//! * **Calls.** Arity, callability, argument compatibility, and generic type
//!   arguments inferred from those arguments in source order.
//! * **Function types.** An annotation such as `Fn(Int) -> Text` retains its
//!   function shape, so function-valued parameters are callable and participate
//!   in structural type comparison and bounded generic inference.
//! * **Fields.** A field access takes its declared field type and reports a
//!   missing field on a known struct.
//! * **Numeric arithmetic.** When both operands are assignable to the same
//!   primitive numeric type, `+`, `-`, `*` and `/` return that type; `%` requires
//!   `Int`; unary `-` returns its operand's numeric type.
//! * **Trust assignment.** The resolved built-in `Verified<T>` can be used
//!   where `T` is expected. `Generated<T>` cannot, and generic arguments remain
//!   invariant.
//! * **Conditions.** An `if` condition is checked as `Bool`, with that expected
//!   type available to generic inference.
//! * **One diagnostic.** `NDO2004`, with `expected` and `found` as structure
//!   rather than as a sentence to re-parse.
//!
//! # What it deliberately does not check yet
//!
//! Each omission is a named next step rather than a hole: the condition of an
//! `if` is checked as `Bool`, but its branch/result typing remains deferred;
//! logical, equality and comparison operators, indexing, diagnostics for
//! unresolved generic parameters, `verify`, provenance tracking,
//! `Result<T, E>` exhaustiveness, and effects remain. Unsupported expression
//! results type as [`Type::Error`], which is compatible with everything and
//! therefore reports nothing: silence, not a wrong answer.

pub mod types;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use nudo_diagnostics::{Diagnostic, Diagnostics, codes};
use nudo_hir::{
    Def, DefId, DefKind, ExprId, ExprKind, Hir, LiteralKind, StmtKind, TypeRef, TypeRefKind,
};
use nudo_source::SourceId;
use nudo_span::Span;

pub use types::{Type, builtin};

/// What the checker produced.
#[derive(Debug)]
pub struct TypeckResult {
    /// Every expression's type, indexed by `ExprId`. `None` means the expression
    /// was never typed, which is not the same as being typed as an error.
    pub expression_types: Vec<Option<Type>>,
    /// Every definition's type, indexed by `DefId`.
    pub definition_types: Vec<Option<Type>>,
    /// What the checker found wrong.
    pub diagnostics: Diagnostics,
}

impl TypeckResult {
    /// The type of a definition, if it has one.
    #[must_use]
    pub fn type_of_def(&self, id: DefId) -> Option<&Type> {
        self.definition_types.get(id.index() as usize)?.as_ref()
    }

    /// The type of an expression, if it has one.
    #[must_use]
    pub fn type_of_expr(&self, id: ExprId) -> Option<&Type> {
        self.expression_types.get(id.index() as usize)?.as_ref()
    }
}

/// Types a resolved program.
#[must_use]
pub fn check(hir: &Hir, source: SourceId) -> TypeckResult {
    let last_expression = hir
        .defs()
        .iter()
        .filter_map(|definition| definition.value)
        .map(|value| value.index())
        .max();
    let builtin_verified = hir
        .defs()
        .iter()
        .take_while(|definition| definition.kind == DefKind::BuiltinType)
        .position(|definition| definition.name == "Verified")
        .map(|index| DefId::from_index(index as u32));

    let mut checker = Checker {
        hir,
        source,
        builtin_verified,
        definition_types: vec![None; hir.defs().len()],
        expression_types: vec![None; last_expression.map_or(0, |last| last as usize + 1)],
        invalid_type_refs: HashSet::new(),
        diagnostics: Diagnostics::new(),
    };

    // Reject malformed type applications before signatures are built, so their
    // recovery type is Error and does not create follow-up mismatches.
    checker.check_type_argument_arities();

    // Signatures first, so that a name's type is known whatever the order in the
    // file. HIR hoists declarations, and this pass respects that.
    for index in 0..hir.defs().len() {
        let id = DefId::from_index(index as u32);
        let signature = checker.signature_of(&hir.defs()[index]);
        checker.definition_types[id.index() as usize] = signature;
    }

    // Then the bodies and the values.
    for index in 0..hir.defs().len() {
        let id = DefId::from_index(index as u32);
        let definition = hir.defs()[index].clone();
        checker.check_definition(&definition, id);
    }

    TypeckResult {
        expression_types: checker.expression_types,
        definition_types: checker.definition_types,
        diagnostics: checker.diagnostics,
    }
}

/// Renders the types a compilation produced, in a stable, diffable form.
///
/// The format is documented in `tests/conformance/README.md`; the `typeck`
/// corpus compares against it.
///
/// ```text
/// # nudo-typeck v1
/// def 0009 binding `value`: Int
/// def 0010 function `answer`: Fn() -> Int
/// ```
///
/// Definitions are listed in declaration order, and built-in types are left out
/// for the same reason they are left out of the resolution dump: they are not in
/// the source, and nine lines of them would bury what a case is about. A
/// definition with no type is printed as `<unknown>`, so that "not typed yet" is
/// visible rather than absent.
#[must_use]
pub fn dump_types(result: &TypeckResult, hir: &Hir, source: &nudo_source::SourceFile) -> String {
    let mut out = String::from("# nudo-typeck v1\n");
    for (index, definition) in hir.defs().iter().enumerate() {
        if definition.kind == DefKind::BuiltinType {
            continue;
        }
        let ty = result
            .definition_types
            .get(index)
            .and_then(Clone::clone)
            .map_or_else(|| "<unknown>".to_string(), |ty| ty.to_string());
        out.push_str(&format!(
            "def {index:04} {} `{}` {}: {ty}\n",
            definition.kind.as_str(),
            definition.name,
            source.line_col(definition.span.start()),
        ));
    }
    out
}

/// A span for the cases where there is nothing in the source to point at.
///
/// It exists so that a fallback is explicit rather than a panic: a diagnostic
/// with a zero span is still a diagnostic, and the alternative — unwrapping —
/// would make a reporting path able to crash the compiler.
fn zero_span() -> Span {
    Span::new(nudo_span::BytePos::new(0), nudo_span::BytePos::new(0))
}

struct Checker<'a> {
    hir: &'a Hir,
    source: SourceId,
    builtin_verified: Option<DefId>,
    definition_types: Vec<Option<Type>>,
    expression_types: Vec<Option<Type>>,
    invalid_type_refs: HashSet<Span>,
    diagnostics: Diagnostics,
}

impl Checker<'_> {
    /// The type a definition declares, if it declares one.
    fn signature_of(&self, definition: &Def) -> Option<Type> {
        match definition.kind {
            DefKind::Function => {
                let parameters: Vec<_> = definition
                    .parameters
                    .iter()
                    .map(|parameter| {
                        self.hir
                            .def(*parameter)
                            .and_then(|parameter| parameter.ty.as_ref())
                            .map_or(Type::Error, |annotation| {
                                self.type_of_annotation(annotation)
                            })
                    })
                    .collect();
                let result = definition
                    .ty
                    .as_ref()
                    .map_or(Type::Unit, |annotation| self.type_of_annotation(annotation));
                Some(Type::Function {
                    parameters: parameters.into(),
                    result: Arc::new(result),
                })
            }
            DefKind::Parameter | DefKind::Binding | DefKind::Const => definition
                .ty
                .as_ref()
                .map(|annotation| self.type_of_annotation(annotation)),
            _ => None,
        }
    }

    /// Checks every type reference, including types nested in fields and enum
    /// payloads. References are traversed in place to avoid cloning complete
    /// nested type trees before the signature pass.
    fn check_type_argument_arities(&mut self) {
        let hir = self.hir;
        let source = self.source;
        let invalid_type_refs = &mut self.invalid_type_refs;
        let diagnostics = &mut self.diagnostics;

        for definition in hir.defs() {
            if let Some(annotation) = &definition.ty {
                Self::check_type_argument_arity(
                    hir,
                    source,
                    annotation,
                    invalid_type_refs,
                    diagnostics,
                );
            }
            for field in &definition.fields {
                if let Some(annotation) = &field.ty {
                    Self::check_type_argument_arity(
                        hir,
                        source,
                        annotation,
                        invalid_type_refs,
                        diagnostics,
                    );
                }
            }
            for variant in &definition.variants {
                for field in &variant.fields {
                    if let Some(annotation) = &field.ty {
                        Self::check_type_argument_arity(
                            hir,
                            source,
                            annotation,
                            invalid_type_refs,
                            diagnostics,
                        );
                    }
                }
            }
        }
    }

    fn check_type_argument_arity(
        hir: &Hir,
        source: SourceId,
        annotation: &TypeRef,
        invalid_type_refs: &mut HashSet<Span>,
        diagnostics: &mut Diagnostics,
    ) {
        if annotation.kind == TypeRefKind::Named {
            if let Some(expected) = Self::expected_type_argument_count(hir, annotation) {
                let found = annotation.arguments.len();
                if expected != found {
                    invalid_type_refs.insert(annotation.span);
                    let span = if found > expected {
                        annotation
                            .arguments
                            .get(expected)
                            .map_or(annotation.span, |argument| argument.span)
                    } else {
                        annotation
                            .arguments
                            .last()
                            .map_or(annotation.span, |argument| argument.span)
                    };
                    let expected_label = if expected == 1 {
                        "argument"
                    } else {
                        "arguments"
                    };
                    diagnostics.push(
                        Diagnostic::error(
                            codes::WRONG_TYPE_ARGUMENT_COUNT,
                            format!("wrong number of type arguments for `{}`", annotation.name),
                        )
                        .with_location(source, span)
                        .with_note(format!("expected: {expected}"))
                        .with_note(format!("found: {found}"))
                        .with_help(format!("provide exactly {expected} type {expected_label}")),
                    );
                }
            }
        }

        for argument in &annotation.arguments {
            Self::check_type_argument_arity(hir, source, argument, invalid_type_refs, diagnostics);
        }
    }

    fn expected_type_argument_count(hir: &Hir, annotation: &TypeRef) -> Option<usize> {
        let definition = hir.def(annotation.target.resolved()?)?;
        match definition.kind {
            DefKind::Struct | DefKind::Enum | DefKind::TypeParameter => {
                Some(definition.type_parameters.len())
            }
            DefKind::BuiltinType => Some(match definition.name.as_str() {
                "Result" => 2,
                "Generated" | "Verified" => 1,
                _ => 0,
            }),
            _ => None,
        }
    }

    /// The type an annotation names.
    ///
    /// A resolved built-in primitive is the type it names; anything else is a
    /// named type whose arguments are lowered the same way. Use the resolved
    /// definition rather than the spelling, because a type parameter can shadow
    /// a built-in in a nested scope.
    fn type_of_annotation(&self, annotation: &TypeRef) -> Type {
        if self.invalid_type_refs.contains(&annotation.span) {
            return Type::Error;
        }
        if annotation.kind == TypeRefKind::Function {
            let Some((result, parameters)) = annotation.arguments.split_last() else {
                return Type::Error;
            };
            return Type::Function {
                parameters: parameters
                    .iter()
                    .map(|parameter| self.type_of_annotation(parameter))
                    .collect::<Vec<_>>()
                    .into(),
                result: Arc::new(self.type_of_annotation(result)),
            };
        }

        let builtin_type = annotation
            .target
            .resolved()
            .and_then(|id| self.hir.def(id))
            .filter(|definition| definition.kind == DefKind::BuiltinType)
            .and_then(|definition| builtin(&definition.name));
        if let Some(builtin) = builtin_type {
            return builtin;
        }
        Type::Named {
            name: annotation.name.clone(),
            definition: annotation.target,
            arguments: annotation
                .arguments
                .iter()
                .map(|argument| self.type_of_annotation(argument))
                .collect::<Vec<_>>()
                .into(),
        }
    }

    fn check_definition(&mut self, definition: &Def, id: DefId) {
        if matches!(definition.kind, DefKind::Parameter | DefKind::BuiltinType) {
            return;
        }
        let Some(value) = definition.value else {
            return;
        };
        let span = self.hir.expr(value).map_or(definition.span, |it| it.span);

        match definition.kind {
            DefKind::Function => {
                let declared_result = definition
                    .ty
                    .as_ref()
                    .map(|annotation| self.type_of_annotation(annotation));
                let actual = self.type_of_expression_with_expected(value, declared_result.as_ref());
                let expected = declared_result.unwrap_or_else(|| actual.clone());
                self.expect(&expected, &actual, span);
                let parameters: Vec<_> = definition
                    .parameters
                    .iter()
                    .map(|parameter| {
                        self.definition_types[parameter.index() as usize]
                            .clone()
                            .unwrap_or(Type::Error)
                    })
                    .collect();
                self.definition_types[id.index() as usize] = Some(Type::Function {
                    parameters: parameters.into(),
                    result: Arc::new(expected),
                });
            }
            _ => match &definition.ty {
                Some(annotation) => {
                    let expected = self.type_of_annotation(annotation);
                    let actual = self.type_of_expression_with_expected(value, Some(&expected));
                    // Point at the value, not at the name: the value is what
                    // breaks the promise, and that is the place a reader has to
                    // change.
                    self.expect(&expected, &actual, span);
                    self.definition_types[id.index() as usize] = Some(expected);
                }
                None => {
                    self.definition_types[id.index() as usize] =
                        Some(self.type_of_expression(value));
                }
            },
        }
    }

    /// Types an expression, writing the answer down as it goes.
    fn type_of_expression(&mut self, id: ExprId) -> Type {
        self.type_of_expression_with_expected(id, None)
    }

    /// Types an expression with a type already required by its surrounding
    /// annotation, return type, or call parameter.
    fn type_of_expression_with_expected(&mut self, id: ExprId, expected: Option<&Type>) -> Type {
        if let Some(known) = self
            .expression_types
            .get(id.index() as usize)
            .cloned()
            .flatten()
        {
            return known;
        }
        let Some(expression) = self.hir.expr(id) else {
            return Type::Error;
        };
        let kind = expression.kind.clone();
        let typed = match kind {
            ExprKind::Literal(literal) => match literal {
                LiteralKind::Int => Type::Int,
                LiteralKind::Float => Type::Float,
                LiteralKind::Text => Type::Text,
                LiteralKind::Bool => Type::Bool,
            },
            ExprKind::Path { target, .. } => target
                .resolved()
                .and_then(|definition| {
                    self.definition_types
                        .get(definition.index() as usize)
                        .cloned()
                        .flatten()
                })
                .unwrap_or(Type::Error),
            ExprKind::Block { statements, value } => {
                for statement in statements {
                    if let Some(statement) = self.hir.statement(statement) {
                        if let StmtKind::Expression(expression) = statement.kind {
                            self.type_of_expression(expression);
                        }
                    }
                }
                match value {
                    Some(value) => self.type_of_expression_with_expected(value, expected),
                    None => Type::Unit,
                }
            }
            ExprKind::Call {
                callee,
                arguments,
                arguments_span,
            } => self.type_of_call(callee, &arguments, arguments_span, expected),
            ExprKind::Field {
                receiver,
                name,
                name_span,
            } => self.type_of_field(receiver, &name, name_span),
            ExprKind::If {
                condition,
                then_block,
                else_branch,
            } => self.type_of_if(condition, then_block, else_branch),
            ExprKind::Index { base, index } => {
                self.type_of_expression(base);
                self.type_of_expression(index);
                Type::Error
            }
            ExprKind::Unary { operator, operand } => self.type_of_unary(&operator, operand),
            ExprKind::Binary {
                operator,
                left,
                right,
            } => self.type_of_binary(&operator, left, right),
            ExprKind::Match { scrutinee, arms } => {
                self.type_of_expression(scrutinee);
                for arm in arms {
                    self.type_of_expression(arm.body);
                }
                Type::Error
            }
            ExprKind::Ask { prompt, .. } => {
                if let Some(prompt) = prompt {
                    self.type_of_expression(prompt);
                }
                Type::Error
            }
            ExprKind::Verify { value, .. } => {
                self.type_of_expression(value);
                Type::Error
            }
            ExprKind::Delegate { request, .. } => {
                if let Some(request) = request {
                    self.type_of_expression(request);
                }
                Type::Error
            }
        };
        if let Some(slot) = self.expression_types.get_mut(id.index() as usize) {
            *slot = Some(typed.clone());
        }
        typed
    }

    /// Checks the condition rule for `if`. The expression's result remains
    /// unknown until the specification settles how its branch values agree.
    fn type_of_if(
        &mut self,
        condition: ExprId,
        then_block: ExprId,
        else_branch: Option<ExprId>,
    ) -> Type {
        let expected = Type::Bool;
        let actual = self.type_of_expression_with_expected(condition, Some(&expected));
        let span = self
            .hir
            .expr(condition)
            .map_or_else(zero_span, |expression| expression.span);
        self.expect(&expected, &actual, span);

        // Branch agreement is not specified yet, but expressions inside either
        // branch still get their independent checks (including nested `if`s).
        self.type_of_expression(then_block);
        if let Some(else_branch) = else_branch {
            self.type_of_expression(else_branch);
        }

        Type::Error
    }

    /// Types unary numeric negation, whose primitive operand type determines
    /// the result type.
    fn type_of_unary(&mut self, operator: &str, operand: ExprId) -> Type {
        let operand_type = self.type_of_expression(operand);
        if operand_type.is_error() {
            return Type::Error;
        }
        if operator != "-" {
            // The type rule for logical negation has not been specified yet.
            return Type::Error;
        }
        if let Some(numeric_type) = self.numeric_type(&operand_type) {
            return numeric_type;
        }
        // Other operand domains remain deferred until their rules are
        // specified; do not reject a possible future built-in operation here.
        Type::Error
    }

    /// Types the arithmetic operators whose numeric behavior is fixed by
    /// NEP-0007. Other operators remain deferred until their rules are decided.
    fn type_of_binary(&mut self, operator: &str, left: ExprId, right: ExprId) -> Type {
        let left_type = self.type_of_expression(left);
        let right_type = self.type_of_expression(right);
        if left_type.is_error() || right_type.is_error() {
            return Type::Error;
        }
        if operator == "%" {
            let expected = Type::Int;
            let left_is_int = self.is_assignable(&left_type, &expected);
            let right_is_int = self.is_assignable(&right_type, &expected);
            if !left_is_int {
                let span = self.hir.expr(left).map_or_else(zero_span, |it| it.span);
                self.report_type_mismatch(
                    &expected.to_string(),
                    &left_type.to_string(),
                    span,
                    "remainder requires `Int` operands",
                );
            }
            if !right_is_int {
                let span = self.hir.expr(right).map_or_else(zero_span, |it| it.span);
                self.report_type_mismatch(
                    &expected.to_string(),
                    &right_type.to_string(),
                    span,
                    "remainder requires `Int` operands",
                );
            }
            return if left_is_int && right_is_int {
                Type::Int
            } else {
                Type::Error
            };
        }
        if !matches!(operator, "+" | "-" | "*" | "/") {
            return Type::Error;
        }

        let left_numeric = self.numeric_type(&left_type);
        let right_numeric = self.numeric_type(&right_type);
        let (Some(left_numeric), Some(right_numeric)) = (left_numeric, right_numeric) else {
            // The operator domains beyond numeric operands are not specified
            // yet, so keep those results deferred rather than inventing rules.
            return Type::Error;
        };
        if left_numeric != right_numeric {
            let span = self.hir.expr(right).map_or_else(zero_span, |it| it.span);
            self.report_type_mismatch(
                &left_numeric.to_string(),
                &right_numeric.to_string(),
                span,
                "arithmetic operands must have the same numeric type",
            );
            return Type::Error;
        }
        left_numeric
    }

    /// Types a call: the callee's signature decides everything.
    ///
    /// Three things can go wrong, and each one says which it is: the callee is
    /// not a function (`NDO2006`), the count is wrong (`NDO2005`), or an argument
    /// is the wrong type (`NDO2004`, reported at the argument, because that is
    /// the thing to change).
    ///
    /// Type parameters are inferred from arguments, left to right, and then from
    /// an already-known expected result type, as NEP-0013 specifies. A diagnostic
    /// for parameters that stay unresolved is still pending, so a result that
    /// contains one is typed as unknown rather than guessed.
    fn type_of_call(
        &mut self,
        callee: ExprId,
        arguments: &[ExprId],
        arguments_span: Span,
        expected_result: Option<&Type>,
    ) -> Type {
        let callee_type = self.type_of_expression(callee);
        let callable_type = self.assignable_view(&callee_type).clone();
        let Type::Function { parameters, result } = callable_type.clone() else {
            if !callable_type.is_error() {
                let span = self.hir.expr(callee).map_or_else(zero_span, |it| it.span);
                self.diagnostics.push(
                    Diagnostic::error(codes::NOT_CALLABLE, "this value is not a function")
                        .with_location(self.source, span)
                        .with_note(format!("found: `{callee_type}`"))
                        .with_help("only a function type can be called"),
                );
            }
            self.check_arguments_without_expectations(arguments);
            return Type::Error;
        };

        let arity_mismatch = arguments.len() != parameters.len();
        if arity_mismatch {
            let expected = if parameters.len() == 1 {
                "argument"
            } else {
                "arguments"
            };
            self.diagnostics.push(
                Diagnostic::error(
                    codes::WRONG_ARGUMENT_COUNT,
                    format!(
                        "this call passes {} and the function takes {} {expected}",
                        arguments.len(),
                        parameters.len()
                    ),
                )
                .with_location(self.source, arguments_span)
                .with_note(format!("expected: {}", parameters.len()))
                .with_note(format!("found: {}", arguments.len()))
                .with_help("every call passes exactly what the signature takes"),
            );
        }

        let mut generic_parameters = HashSet::new();
        for parameter in parameters.iter() {
            self.collect_type_parameters(parameter, &mut generic_parameters);
        }
        self.collect_type_parameters(&result, &mut generic_parameters);

        if generic_parameters.is_empty() {
            for (expected, argument) in parameters.iter().zip(arguments) {
                let actual = self.type_of_expression_with_expected(*argument, Some(expected));
                let span = self
                    .hir
                    .expr(*argument)
                    .map_or_else(zero_span, |it| it.span);
                self.expect(expected, &actual, span);
            }
            self.check_arguments_without_expectations(
                arguments.get(parameters.len()..).unwrap_or(&[]),
            );
            return result.as_ref().clone();
        }

        let mut substitutions = HashMap::new();
        for (parameter, argument) in parameters.iter().zip(arguments) {
            let actual = self.type_of_expression_with_expected(*argument, Some(parameter));
            self.infer_type_arguments(parameter, &actual, &generic_parameters, &mut substitutions);
        }
        if let Some(expected_result) = expected_result {
            self.infer_type_arguments(
                &result,
                expected_result,
                &generic_parameters,
                &mut substitutions,
            );
        }

        let unresolved_parameters = generic_parameters
            .iter()
            .filter(|parameter| !substitutions.contains_key(parameter))
            .copied()
            .collect::<HashSet<_>>();
        for (index, argument) in arguments.iter().enumerate() {
            let Some(parameter) = parameters.get(index) else {
                self.type_of_expression(*argument);
                continue;
            };
            let actual = self.type_of_expression(*argument);
            let actual = self.substitute_type(&actual, &substitutions);
            let expected = self.substitute_type(parameter, &substitutions);
            if let Some(slot) = self.expression_types.get_mut(argument.index() as usize) {
                *slot = Some(actual.clone());
            }
            let span = self
                .hir
                .expr(*argument)
                .map_or_else(zero_span, |it| it.span);
            if !self.compatible_with_unresolved_parameters(
                &actual,
                &expected,
                &unresolved_parameters,
            ) {
                self.expect(&expected, &actual, span);
            }
        }
        if arity_mismatch {
            // Keep the established error-recovery result for invalid calls:
            // do not use a partial argument list to invent their result type.
            return if self.mentions_type_parameter(&result) {
                Type::Error
            } else {
                result.as_ref().clone()
            };
        }
        let result = self.substitute_type(&result, &substitutions);
        if self.contains_type_parameters(&result, &unresolved_parameters) {
            Type::Error
        } else {
            result
        }
    }

    /// Checks supplied call arguments when no parameter context exists, such as
    /// arguments to a non-callable value or arguments beyond the arity. This
    /// keeps independent errors in nested expressions visible.
    fn check_arguments_without_expectations(&mut self, arguments: &[ExprId]) {
        for argument in arguments {
            self.type_of_expression(*argument);
        }
    }

    /// Types a field access: `article.title`.
    ///
    /// A field is reached through a value, so the receiver's type decides
    /// everything. A struct that has the field gives back the field's type; a
    /// struct that does not is `NDO2007`, naming the field and listing what the
    /// type does have; a value of any other type is the same code with a
    /// different sentence, because "`Int` has no fields" is the honest answer.
    ///
    /// Two receivers stay silent. An unknown one (`Error`) has nothing to say. An
    /// `enum` one is read through a `match`, and typing a `match` is the next
    /// slice: saying nothing beats inventing a rule for reaching into a variant.
    fn type_of_field(&mut self, receiver: ExprId, name: &str, name_span: Span) -> Type {
        let receiver_type = self.type_of_expression(receiver);
        let receiver_type = self.assignable_view(&receiver_type).clone();
        if receiver_type.is_error() {
            return Type::Error;
        }

        let Type::Named {
            definition,
            arguments,
            ..
        } = &receiver_type
        else {
            self.report_unknown_field(name, &receiver_type, name_span, &[]);
            return Type::Error;
        };
        let Some(id) = definition.resolved() else {
            return Type::Error;
        };

        // Everything read out of the definition is copied first: reporting a
        // diagnostic needs `&mut self`, and a borrow of the HIR would outlive it.
        let (is_struct, type_parameters, found, available) = {
            let Some(definition) = self.hir.def(id) else {
                return Type::Error;
            };
            match definition.kind {
                DefKind::Struct => (
                    true,
                    definition.type_parameters.clone(),
                    definition
                        .fields
                        .iter()
                        .find(|field| field.name == name)
                        .and_then(|field| field.ty.clone()),
                    definition
                        .fields
                        .iter()
                        .map(|field| field.name.clone())
                        .collect::<Vec<String>>(),
                ),
                _ => (false, Vec::new(), None, Vec::new()),
            }
        };

        if !is_struct {
            return Type::Error;
        }
        if let Some(annotation) = found {
            if type_parameters.len() != arguments.len() {
                // Type-argument arity is checked in its own slice. Until then,
                // do not invent a field type from an incomplete instantiation.
                return Type::Error;
            }
            let substitutions = type_parameters
                .iter()
                .copied()
                .zip(arguments.iter().cloned())
                .collect::<HashMap<_, _>>();
            let field_type = self.type_of_annotation(&annotation);
            return self.substitute_type(&field_type, &substitutions);
        }
        self.report_unknown_field(name, &receiver_type, name_span, &available);
        Type::Error
    }

    /// Reports a field a value does not have.
    fn report_unknown_field(
        &mut self,
        name: &str,
        receiver: &Type,
        span: Span,
        available: &[String],
    ) {
        let mut diagnostic = Diagnostic::error(
            codes::UNKNOWN_FIELD,
            format!("`{receiver}` has no field `{name}`"),
        )
        .with_location(self.source, span)
        .with_note(format!("found: `{receiver}`"));
        if !available.is_empty() {
            diagnostic = diagnostic.with_note(format!("fields: {}", available.join(", ")));
        }
        self.diagnostics.push(
            diagnostic
                .with_help("a field is read through a value, so the value's type has to have it"),
        );
    }

    /// Whether a type mentions any type parameter.
    fn mentions_type_parameter(&self, ty: &Type) -> bool {
        match ty {
            Type::Named { arguments, .. } => {
                self.type_parameter_id(ty).is_some()
                    || arguments
                        .iter()
                        .any(|argument| self.mentions_type_parameter(argument))
            }
            Type::Function { parameters, result } => {
                parameters
                    .iter()
                    .any(|parameter| self.mentions_type_parameter(parameter))
                    || self.mentions_type_parameter(result)
            }
            _ => false,
        }
    }

    /// Collects the resolved type parameter definitions present in a type.
    fn collect_type_parameters(&self, ty: &Type, found: &mut HashSet<DefId>) {
        match ty {
            Type::Named { arguments, .. } => {
                if let Some(id) = self.type_parameter_id(ty) {
                    found.insert(id);
                    return;
                }
                for argument in arguments.iter() {
                    self.collect_type_parameters(argument, found);
                }
            }
            Type::Function { parameters, result } => {
                for parameter in parameters.iter() {
                    self.collect_type_parameters(parameter, found);
                }
                self.collect_type_parameters(result, found);
            }
            _ => {}
        }
    }

    /// Adds the type arguments that a formal parameter determines from an
    /// actual argument. An existing binding wins, so later arguments are checked
    /// against it in the ordinary compatibility pass.
    fn infer_type_arguments(
        &self,
        formal: &Type,
        actual: &Type,
        generic_parameters: &HashSet<DefId>,
        substitutions: &mut HashMap<DefId, Type>,
    ) {
        if actual.is_error() {
            return;
        }
        if let Some(id) = self.type_parameter_id(formal) {
            if generic_parameters.contains(&id) {
                substitutions.entry(id).or_insert_with(|| actual.clone());
            }
            return;
        }
        match (formal, actual) {
            (
                Type::Named {
                    definition: formal_definition,
                    arguments: formal_arguments,
                    ..
                },
                Type::Named {
                    definition: actual_definition,
                    arguments: actual_arguments,
                    ..
                },
            ) if formal_definition == actual_definition
                && formal_arguments.len() == actual_arguments.len() =>
            {
                for (formal, actual) in formal_arguments.iter().zip(actual_arguments.iter()) {
                    self.infer_type_arguments(formal, actual, generic_parameters, substitutions);
                }
            }
            (
                Type::Function {
                    parameters: formal_parameters,
                    result: formal_result,
                },
                Type::Function {
                    parameters: actual_parameters,
                    result: actual_result,
                },
            ) if formal_parameters.len() == actual_parameters.len() => {
                for (formal, actual) in formal_parameters.iter().zip(actual_parameters.iter()) {
                    self.infer_type_arguments(formal, actual, generic_parameters, substitutions);
                }
                self.infer_type_arguments(
                    formal_result,
                    actual_result,
                    generic_parameters,
                    substitutions,
                );
            }
            _ => {}
        }
    }

    /// The definition id when `ty` is a type parameter.
    fn type_parameter_id(&self, ty: &Type) -> Option<DefId> {
        let Type::Named { definition, .. } = ty else {
            return None;
        };
        let id = definition.resolved()?;
        self.hir
            .def(id)
            .is_some_and(|definition| definition.kind == DefKind::TypeParameter)
            .then_some(id)
    }

    /// Replaces each type parameter that this call inferred.
    fn substitute_type(&self, ty: &Type, substitutions: &HashMap<DefId, Type>) -> Type {
        if let Some(id) = self.type_parameter_id(ty) {
            return substitutions
                .get(&id)
                .cloned()
                .unwrap_or_else(|| ty.clone());
        }
        match ty {
            Type::Named {
                name,
                definition,
                arguments,
            } => Type::Named {
                name: name.clone(),
                definition: *definition,
                arguments: arguments
                    .iter()
                    .map(|argument| self.substitute_type(argument, substitutions))
                    .collect::<Vec<_>>()
                    .into(),
            },
            Type::Function { parameters, result } => Type::Function {
                parameters: parameters
                    .iter()
                    .map(|parameter| self.substitute_type(parameter, substitutions))
                    .collect::<Vec<_>>()
                    .into(),
                result: Arc::new(self.substitute_type(result, substitutions)),
            },
            _ => ty.clone(),
        }
    }

    /// Whether a type still contains one of this call's unresolved parameters.
    fn contains_type_parameters(&self, ty: &Type, unresolved: &HashSet<DefId>) -> bool {
        if self
            .type_parameter_id(ty)
            .is_some_and(|id| unresolved.contains(&id))
        {
            return true;
        }
        match ty {
            Type::Named { arguments, .. } => arguments
                .iter()
                .any(|argument| self.contains_type_parameters(argument, unresolved)),
            Type::Function { parameters, result } => {
                parameters
                    .iter()
                    .any(|parameter| self.contains_type_parameters(parameter, unresolved))
                    || self.contains_type_parameters(result, unresolved)
            }
            _ => false,
        }
    }

    /// Compares types while treating this call's unresolved parameters as
    /// wildcards. This still catches known mismatches around an uninferred part
    /// of a nested generic type without claiming to know that parameter itself.
    fn compatible_with_unresolved_parameters(
        &self,
        actual: &Type,
        expected: &Type,
        unresolved: &HashSet<DefId>,
    ) -> bool {
        if actual.is_error() || expected.is_error() {
            return true;
        }
        if self
            .type_parameter_id(expected)
            .is_some_and(|id| unresolved.contains(&id))
        {
            return true;
        }
        match (actual, expected) {
            (
                Type::Named {
                    definition: actual_definition,
                    arguments: actual_arguments,
                    ..
                },
                Type::Named {
                    definition: expected_definition,
                    arguments: expected_arguments,
                    ..
                },
            ) => {
                actual_definition == expected_definition
                    && actual_arguments.len() == expected_arguments.len()
                    && actual_arguments.iter().zip(expected_arguments.iter()).all(
                        |(actual, expected)| {
                            self.compatible_with_unresolved_parameters(actual, expected, unresolved)
                        },
                    )
            }
            (
                Type::Function {
                    parameters: actual_parameters,
                    result: actual_result,
                },
                Type::Function {
                    parameters: expected_parameters,
                    result: expected_result,
                },
            ) => {
                actual_parameters.len() == expected_parameters.len()
                    && actual_parameters
                        .iter()
                        .zip(expected_parameters.iter())
                        .all(|(actual, expected)| {
                            self.compatible_with_unresolved_parameters(actual, expected, unresolved)
                        })
                    && self.compatible_with_unresolved_parameters(
                        actual_result,
                        expected_result,
                        unresolved,
                    )
            }
            // Assignment compatibility is intentionally not recursive here:
            // generic arguments and function components remain invariant even
            // though a top-level Verified<T> can be used as T.
            _ => actual.is_compatible_with(expected),
        }
    }

    /// Whether a value can be used where `expected` is required.
    ///
    /// Most values must have the identical resolved type. The one accepted
    /// subtype relation is the built-in `Verified<T>` to `T` rule. Comparing
    /// resolved definition identities prevents a type parameter with the same
    /// spelling from acquiring this privilege.
    fn is_assignable(&self, found: &Type, expected: &Type) -> bool {
        found.is_compatible_with(expected)
            || self.assignable_view(found).is_compatible_with(expected)
    }

    /// Returns the type a value can be used as after applying the accepted
    /// `Verified<T>`-to-`T` relation. The relation is transparent in supported
    /// use positions such as numeric operations, field access and calls; it does
    /// not recurse into generic arguments or function components.
    fn assignable_view<'a>(&self, ty: &'a Type) -> &'a Type {
        let mut candidate = ty;
        loop {
            let Type::Named {
                definition,
                arguments,
                ..
            } = candidate
            else {
                return candidate;
            };
            let is_builtin_verified = self
                .builtin_verified
                .is_some_and(|builtin| definition.resolved() == Some(builtin));
            if !is_builtin_verified || arguments.len() != 1 {
                return candidate;
            }
            candidate = &arguments[0];
        }
    }

    /// The primitive numeric type a value is assignable to, if any.
    fn numeric_type(&self, ty: &Type) -> Option<Type> {
        if self.is_assignable(ty, &Type::Int) {
            Some(Type::Int)
        } else if self.is_assignable(ty, &Type::Float) {
            Some(Type::Float)
        } else {
            None
        }
    }

    /// Reports a mismatch, with the two types as structure rather than prose.
    fn expect(&mut self, expected: &Type, found: &Type, span: Span) {
        if self.is_assignable(found, expected) {
            return;
        }
        self.report_type_mismatch(
            &expected.to_string(),
            &found.to_string(),
            span,
            "an annotation is a promise, and this value does not keep it",
        );
    }

    /// Reports a mismatch with context-specific expected and found details.
    fn report_type_mismatch(&mut self, expected: &str, found: &str, span: Span, help: &str) {
        self.diagnostics.push(
            Diagnostic::error(codes::TYPE_MISMATCH, "type mismatch")
                .with_location(self.source, span)
                .with_note(format!("expected: `{expected}`"))
                .with_note(format!("found: `{found}`"))
                .with_help(help),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nudo_hir::{Hir, lower};
    use nudo_parser::parse;
    use nudo_source::SourceMap;

    fn checked(text: &str) -> (SourceMap, SourceId, TypeckResult) {
        let (sources, id, _hir, result) = checked_with_hir(text);
        (sources, id, result)
    }

    fn checked_with_hir(text: &str) -> (SourceMap, SourceId, Hir, TypeckResult) {
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", text);
        let file = sources.get(id).expect("just added");
        let parsed = parse(file);
        assert!(!parsed.has_errors(), "the input must parse: {text}");
        let resolved = lower(parsed.tree(), id);
        assert!(
            resolved.diagnostics.is_empty(),
            "the input must resolve: {text}"
        );
        let result = check(&resolved.hir, id);
        (sources, id, resolved.hir, result)
    }

    fn codes_for(text: &str) -> Vec<String> {
        let (_, _, result) = checked(text);
        result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code().id().to_string())
            .collect()
    }

    fn named_types(result: &TypeckResult) -> Vec<String> {
        result
            .definition_types
            .iter()
            .flatten()
            .map(std::string::ToString::to_string)
            .collect()
    }

    #[test]
    fn literals_have_their_own_types() {
        let (_, _, result) =
            checked("let a = 1;\nlet b = 1.5;\nlet c = true;\nlet d = \"nudo\";\n");
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
        let types = named_types(&result);
        for expected in ["Int", "Float", "Bool", "Text"] {
            assert!(
                types.contains(&expected.to_string()),
                "{expected} is missing from {types:?}"
            );
        }
    }

    #[test]
    fn an_annotation_that_matches_is_accepted() {
        assert_eq!(codes_for("let x: Int = 1;\n"), Vec::<String>::new());
        assert_eq!(codes_for("let x: Bool = true;\n"), Vec::<String>::new());
        assert_eq!(codes_for("let x: Text = \"nudo\";\n"), Vec::<String>::new());
    }

    #[test]
    fn an_annotation_that_does_not_match_is_ndo2004() {
        assert_eq!(
            codes_for("let x: Int = true;\n"),
            vec!["NDO2004".to_string()]
        );
        assert_eq!(codes_for("let x: Text = 1;\n"), vec!["NDO2004".to_string()]);
    }

    #[test]
    fn the_diagnostic_carries_expected_and_found_as_structure() {
        let (sources, id, result) = checked("let x: Int = true;\n");
        let diagnostic = &result.diagnostics.as_slice()[0];
        assert_eq!(diagnostic.code().id(), "NDO2004");
        let notes = diagnostic.notes().join(" | ");
        assert!(notes.contains("expected: `Int`"), "{notes}");
        assert!(notes.contains("found: `Bool`"), "{notes}");
        let file = sources.get(id).expect("just added");
        let span = diagnostic.span().expect("a span");
        // The diagnostic points at the value that breaks the promise, not at the
        // name the promise was made for.
        let expected_column = "let x: Int = true;".find("true").expect("the value") + 1;
        assert_eq!(
            file.line_col(span.start()).to_string(),
            format!("1:{expected_column}")
        );
    }

    #[test]
    fn a_nominal_mismatch_names_both_types_and_points_at_the_value() {
        let program = "struct Article {\n    title: Text\n}\n\
                       struct Book {\n    title: Text\n}\n\
                       fn publish(article: Article) -> Text {\n    article.title\n}\n\
                       fn main(book: Book) -> Text {\n    publish(book)\n}\n";
        let (sources, id, result) = checked(program);
        let diagnostic = &result.diagnostics.as_slice()[0];
        assert_eq!(diagnostic.code().id(), "NDO2004");
        let notes = diagnostic.notes().join(" | ");
        assert!(notes.contains("expected: `Article`"), "{notes}");
        assert!(notes.contains("found: `Book`"), "{notes}");
        let file = sources.get(id).expect("just added");
        let span = diagnostic.span().expect("the argument has a span");
        assert_eq!(file.span_text(span), Some("book"));
    }

    #[test]
    fn there_is_no_implicit_numeric_conversion() {
        assert_eq!(
            codes_for("let x: Float = 1;\n"),
            vec!["NDO2004".to_string()]
        );
    }

    #[test]
    fn numeric_arithmetic_preserves_its_operand_type() {
        let (_, _, hir, result) = checked_with_hir(
            "fn add(left: Int, right: Int) -> Int { left + right }\n\
             let sum = 1 + 2;\n\
             let difference = 6 - 2;\n\
             let quotient = 4.0 / 2.0;\n\
             let scaled = 1.5 * 2.0;\n\
             let remainder = -7 % 2;\n\
             let negative = -3;\n",
        );
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let add = hir
            .defs()
            .iter()
            .find(|definition| definition.name == "add")
            .expect("function exists");
        assert_eq!(
            result.type_of_expr(add.value.expect("function has a body")),
            Some(&Type::Int),
            "function body takes the arithmetic result type"
        );
        for (name, expected) in [
            ("sum", Type::Int),
            ("difference", Type::Int),
            ("quotient", Type::Float),
            ("scaled", Type::Float),
            ("remainder", Type::Int),
            ("negative", Type::Int),
        ] {
            let definition = hir
                .defs()
                .iter()
                .find(|definition| definition.name == name)
                .unwrap_or_else(|| panic!("missing binding `{name}`"));
            let value = definition.value.expect("binding has an initializer");
            assert_eq!(result.type_of_expr(value), Some(&expected), "{name}");
        }
    }

    #[test]
    fn mixed_numeric_arithmetic_is_a_type_mismatch_at_the_right_operand() {
        let (sources, id, result) = checked("let value = 1 + 2.0;\n");
        assert_eq!(
            result
                .diagnostics
                .as_slice()
                .iter()
                .map(|diagnostic| diagnostic.code().id())
                .collect::<Vec<_>>(),
            vec!["NDO2004"]
        );
        let diagnostic = &result.diagnostics.as_slice()[0];
        let notes = diagnostic.notes().join(" | ");
        assert!(notes.contains("expected: `Int`"), "{notes}");
        assert!(notes.contains("found: `Float`"), "{notes}");
        let file = sources.get(id).expect("just added");
        let span = diagnostic.span().expect("a span");
        let expected_column = "let value = 1 + 2.0;".find("2.0").expect("right operand") + 1;
        assert_eq!(
            file.line_col(span.start()).to_string(),
            format!("1:{expected_column}")
        );
    }

    #[test]
    fn a_function_with_no_written_return_type_takes_its_body_type() {
        let (_, _, result) = checked("fn answer() {\n    42\n}\n");
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
        assert!(
            named_types(&result).contains(&"Fn() -> Int".to_string()),
            "{:?}",
            named_types(&result)
        );
    }

    #[test]
    fn a_function_whose_body_contradicts_its_return_type_is_reported() {
        assert_eq!(
            codes_for("fn answer() -> Int {\n    true\n}\n"),
            vec!["NDO2004".to_string()]
        );
    }

    #[test]
    fn if_conditions_must_be_bool_including_nested_else_if_conditions() {
        let program = "fn main() {\n    if 1 {} else if \"ready\" {} else {};\n}\n";
        assert_eq!(
            codes_for(program),
            vec!["NDO2004".to_string(), "NDO2004".to_string()]
        );
    }

    #[test]
    fn an_if_condition_provides_context_for_generic_result_inference() {
        let program =
            "fn empty<T>() -> T {\n    empty()\n}\n\nfn main() {\n    if empty() {};\n}\n";
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", program);
        let file = sources.get(id).expect("just added");
        let parsed = parse(file);
        assert!(!parsed.has_errors(), "the input must parse");
        let resolved = lower(parsed.tree(), id);
        assert!(
            resolved.diagnostics.is_empty(),
            "the input must resolve: {:?}",
            resolved.diagnostics
        );
        let result = check(&resolved.hir, id);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);

        let main = resolved
            .hir
            .defs()
            .iter()
            .find(|definition| definition.is_main())
            .expect("main function");
        let body = main.value.expect("main body");
        let ExprKind::Block {
            statements,
            value: None,
        } = &resolved.hir.expr(body).expect("main body expression").kind
        else {
            panic!("main should have one expression statement and no tail value");
        };
        let statement = resolved
            .hir
            .statement(statements[0])
            .expect("if expression statement");
        let StmtKind::Expression(if_expression) = statement.kind else {
            panic!("main's expression statement should contain the if expression");
        };
        let ExprKind::If { condition, .. } = &resolved
            .hir
            .expr(if_expression)
            .expect("if expression")
            .kind
        else {
            panic!("main's expression statement should contain an if expression");
        };
        assert_eq!(result.type_of_expr(*condition), Some(&Type::Bool));
    }

    #[test]
    fn expressions_inside_if_branches_are_checked() {
        let program = "fn main() {\n    if true {\n        let first: Int = true;\n    } else if false {\n        let second: Text = 2;\n    };\n}\n";
        assert_eq!(
            codes_for(program),
            vec!["NDO2004".to_string(), "NDO2004".to_string()]
        );
    }

    #[test]
    fn supported_calls_inside_deferred_binary_expressions_are_checked() {
        let program = "fn add(a: Int, b: Int) -> Int {\n    a\n}\n\nfn main() {\n    let value = add(1, true) + 2;\n}\n";
        assert_eq!(codes_for(program), vec!["NDO2004".to_string()]);
    }

    #[test]
    fn a_path_takes_the_type_of_what_it_resolved_to() {
        let (_, _, result) = checked("let value: Int = 1;\nlet copy = value;\n");
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
        let types = named_types(&result);
        assert_eq!(
            types.iter().filter(|ty| *ty == "Int").count(),
            2,
            "{types:?}"
        );
    }

    #[test]
    fn a_block_without_a_value_is_unit() {
        let (_, _, result) = checked("fn nothing() {\n    let x = 1;\n}\n");
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
        assert!(
            named_types(&result).contains(&"Fn() -> Unit".to_string()),
            "{:?}",
            named_types(&result)
        );
    }

    #[test]
    fn a_named_type_survives_with_its_arguments() {
        let (_, _, result) =
            checked("struct Article {\n    title: Text\n}\n\nlet draft: Generated<Article> = 1;\n");
        assert!(
            named_types(&result).contains(&"Generated<Article>".to_string()),
            "a named type is represented, even though no structural rule exists yet: {:?}",
            named_types(&result)
        );
    }

    #[test]
    fn a_valid_call_does_not_produce_diagnostics() {
        let (_, _, result) = checked(
            "fn add(a: Int, b: Int) -> Int {\n    a\n}\n\nfn main() {\n    let total = add(1, 2);\n}\n",
        );
        assert_eq!(
            result.diagnostics.len(),
            0,
            "a valid call must not produce a diagnostic: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn a_call_is_checked_against_the_signature() {
        let program = "fn add(a: Int, b: Int) -> Int {\n    a\n}\n\nfn main() {\n    let one = add(1, 2);\n}\n";
        assert_eq!(codes_for(program), Vec::<String>::new());

        let wrong_type = "fn add(a: Int, b: Int) -> Int {\n    a\n}\n\nfn main() {\n    let one = add(1, true);\n}\n";
        assert_eq!(codes_for(wrong_type), vec!["NDO2004".to_string()]);

        let wrong_count =
            "fn add(a: Int, b: Int) -> Int {\n    a\n}\n\nfn main() {\n    let one = add(1);\n}\n";
        assert_eq!(codes_for(wrong_count), vec!["NDO2005".to_string()]);
    }

    #[test]
    fn an_arity_diagnostic_points_at_the_argument_list() {
        let program = "fn add(a: Int, b: Int) -> Int { a }\n\
                       fn main() {\n\
                           let one = add(1);\n\
                       }\n";
        let (sources, id, result) = checked(program);
        let diagnostic = result
            .diagnostics
            .as_slice()
            .iter()
            .find(|diagnostic| diagnostic.code().id() == "NDO2005")
            .expect("the call has the wrong number of arguments");
        let file = sources.get(id).expect("just added");
        let span = diagnostic.span().expect("the argument list has a span");
        assert_eq!(file.span_text(span), Some("(1)"));
    }

    #[test]
    fn an_arity_error_does_not_hide_errors_inside_supplied_arguments() {
        let program = "fn takes(value: Int) {}\n\
                       let number = 1;\n\
                       fn main() {\n\
                           let result = takes(0, number());\n\
                       }\n";
        assert_eq!(
            codes_for(program),
            vec!["NDO2005".to_string(), "NDO2006".to_string()]
        );
    }

    #[test]
    fn an_arity_error_still_checks_arguments_with_known_parameter_positions() {
        let program = "fn add(left: Int, right: Int) -> Int { left }\n\
                       fn main() {\n\
                           let missing = add(true);\n\
                           let extra = add(1, false, 2);\n\
                       }\n";
        assert_eq!(
            codes_for(program),
            vec![
                "NDO2005".to_string(),
                "NDO2004".to_string(),
                "NDO2005".to_string(),
                "NDO2004".to_string(),
            ]
        );
    }

    #[test]
    fn a_non_callable_call_does_not_hide_errors_inside_arguments() {
        let program = "let number = 1;\n\
                       fn main() {\n\
                           let result = number(number());\n\
                           let tainted = number()(number());\n\
                       }\n";
        assert_eq!(
            codes_for(program),
            vec![
                "NDO2006".to_string(),
                "NDO2006".to_string(),
                "NDO2006".to_string(),
                "NDO2006".to_string(),
            ]
        );
    }

    #[test]
    fn a_call_produces_a_type_and_not_only_a_complaint() {
        // The result type travels: it is what makes the checker useful to the
        // next stage rather than only to a reader.
        let consistent = "fn add(a: Int, b: Int) -> Int {\n    a\n}\n\nfn main() {\n    let total: Int = add(1, 2);\n}\n";
        assert_eq!(codes_for(consistent), Vec::<String>::new());

        let inconsistent = "fn add(a: Int, b: Int) -> Int {\n    a\n}\n\nfn main() {\n    let total: Bool = add(1, 2);\n}\n";
        assert_eq!(codes_for(inconsistent), vec!["NDO2004".to_string()]);
    }

    #[test]
    fn calling_something_that_is_not_a_function_is_reported() {
        let program = "let number = 1;\n\nfn main() {\n    number();\n}\n";
        let (sources, id, result) = checked(program);
        let diagnostic = &result.diagnostics.as_slice()[0];
        assert_eq!(diagnostic.code().id(), "NDO2006");
        assert_eq!(diagnostic.message(), "this value is not a function");
        assert!(
            diagnostic.notes().join(" | ").contains("found: `Int`"),
            "{:?}",
            diagnostic.notes()
        );
        assert_eq!(
            diagnostic.help(),
            Some("only a function type can be called")
        );
        let file = sources.get(id).expect("just added");
        let span = diagnostic.span().expect("the callee has a span");
        assert_eq!(file.span_text(span), Some("number"));
    }

    #[test]
    fn a_verified_non_callable_value_keeps_its_source_type_in_the_diagnostic() {
        let program = "fn main(value: Verified<Int>) {\n    value();\n}\n";
        let (sources, id, result) = checked(program);
        let diagnostic = &result.diagnostics.as_slice()[0];
        assert_eq!(diagnostic.code().id(), "NDO2006");
        assert!(
            diagnostic
                .notes()
                .join(" | ")
                .contains("found: `Verified<Int>`"),
            "{:?}",
            diagnostic.notes()
        );
        let file = sources.get(id).expect("just added");
        let span = diagnostic.span().expect("the callee has a span");
        assert_eq!(file.span_text(span), Some("value"));
    }

    #[test]
    fn a_generic_call_infers_its_result_from_the_argument() {
        let program = "fn identity<T>(value: T) -> T {\n    value\n}\n\nfn main() {\n    let one = identity(1);\n}\n";
        let (_, _, result) = checked(program);
        assert_eq!(result.diagnostics.len(), 0, "{:?}", result.diagnostics);
        assert!(
            named_types(&result).contains(&"Int".to_string()),
            "the inferred binding should have type Int: {:?}",
            named_types(&result)
        );
    }

    #[test]
    fn repeated_generic_arguments_must_agree() {
        let program = "fn same<T>(first: T, second: T) -> T {\n    first\n}\n\nfn main() {\n    let value = same(1, true);\n}\n";
        assert_eq!(codes_for(program), vec!["NDO2004".to_string()]);
    }

    #[test]
    fn generic_inference_walks_nested_named_types() {
        let program = "struct Box<T> {\n    value: T\n}\n\nfn nested<T>(value: Box<T>) -> Box<T> {\n    value\n}\n\nfn forward<U>(value: Box<U>) -> Box<U> {\n    nested(value)\n}\n";
        assert_eq!(codes_for(program), Vec::<String>::new());
    }

    #[test]
    fn an_expected_result_type_infers_generic_arguments() {
        let program = "struct Box<T> {\n    value: T\n}\n\nfn empty<T>() -> Box<T> {\n    empty()\n}\n\nfn make_text() -> Box<Text> {\n    empty()\n}\n\nfn main() {\n    let answer: Box<Text> = empty();\n}\n";
        assert_eq!(codes_for(program), Vec::<String>::new());
    }

    #[test]
    fn a_call_parameter_provides_context_to_a_nested_generic_call() {
        let program = "fn empty<T>() -> T {\n    empty()\n}\n\nfn take_text(value: Text) -> Text {\n    value\n}\n\nfn forward() -> Text {\n    take_text(empty())\n}\n";
        assert_eq!(codes_for(program), Vec::<String>::new());
    }

    #[test]
    fn a_known_argument_type_wins_over_a_conflicting_expected_result() {
        let program = "fn identity<T>(value: T) -> T {\n    value\n}\n\nfn main() {\n    let answer: Text = identity(1);\n}\n";
        assert_eq!(codes_for(program), vec!["NDO2004".to_string()]);
    }

    #[test]
    fn a_call_to_a_function_declared_later_works() {
        let program = "fn main() {\n    later();\n}\n\nfn later() {\n}\n";
        assert_eq!(codes_for(program), Vec::<String>::new());
    }

    #[test]
    fn a_field_access_takes_the_field_type() {
        let program = "struct Article {\n    title: Text\n}\n\nfn headline(article: Article) -> Text {\n    article.title\n}\n";
        assert_eq!(codes_for(program), Vec::<String>::new());
        assert!(
            named_types(&checked(program).2).contains(&"Fn(Article) -> Text".to_string()),
            "{:?}",
            named_types(&checked(program).2)
        );
    }

    #[test]
    fn a_field_that_does_not_exist_is_ndo2007() {
        let program = "struct Article {\n    title: Text\n}\n\nfn headline(article: Article) -> Text {\n    article.titel\n}\n";
        assert_eq!(codes_for(program), vec!["NDO2007".to_string()]);
    }

    #[test]
    fn the_field_diagnostic_names_the_field_and_what_the_type_has() {
        let program = "struct Article {\n    title: Text\n    body: Text\n}\n\nfn headline(article: Article) -> Text {\n    article.titel\n}\n";
        let (sources, id, result) = checked(program);
        let diagnostic = &result.diagnostics.as_slice()[0];
        assert_eq!(diagnostic.message(), "`Article` has no field `titel`");
        let file = sources.get(id).expect("just added");
        let span = diagnostic.span().expect("the unknown field has a span");
        assert_eq!(file.span_text(span), Some("titel"));
        let notes = diagnostic.notes().join(" | ");
        assert!(notes.contains("found: `Article`"), "{notes}");
        assert!(notes.contains("fields: title, body"), "{notes}");
    }

    #[test]
    fn reading_a_field_of_something_that_has_none_is_reported() {
        let program = "let number = 1;\n\nfn main() -> Int {\n    number.foo\n}\n";
        assert_eq!(codes_for(program), vec!["NDO2007".to_string()]);
    }

    #[test]
    fn a_field_access_chains_through_struct_fields() {
        let program = "struct Author {\n    name: Text\n}\n\nstruct Article {\n    author: Author\n}\n\nfn author_name(article: Article) -> Text {\n    article.author.name\n}\n";
        assert_eq!(codes_for(program), Vec::<String>::new());
    }

    #[test]
    fn generic_struct_fields_use_the_receiver_type_arguments() {
        let program = "struct Box<T> {\n    value: T\n}\n\nstruct Wrap<U> {\n    inner: Box<U>\n}\n\nfn get<T>(wrap: Wrap<T>) -> T {\n    wrap.inner.value\n}\n\nfn get_int(box: Box<Int>) -> Int {\n    box.value\n}\n";
        assert_eq!(codes_for(program), Vec::<String>::new());
    }

    #[test]
    fn verified_values_use_their_inner_type_for_fields_and_calls() {
        let program = "struct Article {\n    title: Text\n}\n\
                       fn headline(article: Verified<Article>) -> Text {\n\
                           article.title\n\
                       }\n\
                       fn apply(operation: Verified<Fn(Int) -> Int>, value: Int) -> Int {\n\
                           operation(value)\n\
                       }\n";
        let (_, _, hir, result) = checked_with_hir(program);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);

        for (name, expected) in [("headline", Type::Text), ("apply", Type::Int)] {
            let definition = hir
                .defs()
                .iter()
                .find(|definition| definition.name == name)
                .unwrap_or_else(|| panic!("missing function `{name}`"));
            assert_eq!(
                result.type_of_expr(definition.value.expect("function has a body")),
                Some(&expected),
                "the body of `{name}` should have its declared result type"
            );
        }
    }

    #[test]
    fn a_field_access_on_an_inferred_primitive_reports_the_missing_field() {
        let program = "fn identity<T>(value: T) -> T {\n    value\n}\n\nfn main() {\n    let anything = identity(1);\n    let field = anything.name;\n}\n";
        assert_eq!(codes_for(program), vec!["NDO2007".to_string()]);
    }

    #[test]
    fn an_expression_is_typed_once() {
        let (_, _, result) = checked("let a = 1;\nlet b = a;\n");
        let typed = result
            .expression_types
            .iter()
            .filter(|ty| ty.is_some())
            .count();
        assert!(typed >= 2, "each literal and each path is typed: {typed}");
    }
}
