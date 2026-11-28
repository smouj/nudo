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
//! * **Blocks and function results.** A block's type is its final expression's,
//!   or `Unit` when it has none. A function without a result annotation has the
//!   declared result `Unit`; its body must satisfy that type.
//! * **Names.** A path takes the type of the definition it resolved to, which is
//!   why resolution had to exist first.
//! * **Calls.** Arity, callability, argument compatibility, and generic type
//!   arguments inferred from those arguments in source order.
//! * **Function types.** An annotation such as `Fn(Int) -> Text` retains its
//!   function shape, so function-valued parameters are callable and participate
//!   in structural type comparison and bounded generic inference.
//! * **Fields.** A field access takes its declared field type and reports a
//!   missing field on a known struct.
//! * **Operators.** Arithmetic uses matching numeric operands, `%` uses `Int`,
//!   Boolean operators use `Bool`, equality is limited to matching primitive
//!   types, and ordering is limited to matching `Int` or `Float` operands. An
//!   invalid operator domain produces one `NDO2018` at the operator.
//! * **Trust assignment.** The resolved built-in `Verified<T>` can be used
//!   where `T` is expected. `Generated<T>` cannot, and generic arguments remain
//!   invariant.
//! * **Conditionals.** Conditions are `Bool`; branch values use an expected
//!   type when one is available and otherwise must agree exactly. An `if`
//!   without `else` has type `Unit` and requires a `Unit` then-branch.
//! * **Enum matching.** `match` accepts user enums and `Result<T, E>`, checks
//!   flat payload bindings, wildcard coverage and arm result types.
//! * **One diagnostic.** `NDO2004`, with `expected` and `found` as structure
//!   rather than as a sentence to re-parse.
//!
//! # What it deliberately does not check yet
//!
//! Each omission is a named next step rather than a hole: indexing, `verify`,
//! provenance tracking, and effects remain. Unsupported expression results
//! type as [`Type::Error`], which is compatible with everything and therefore
//! reports nothing: silence, not a wrong answer.

pub mod types;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use nudo_diagnostics::{Diagnostic, Diagnostics, codes};
use nudo_hir::{
    Arm, Def, DefId, DefKind, ExprId, ExprKind, Hir, IntegerMagnitude, LiteralKind, MatchPattern,
    PayloadPattern, StmtKind, TypeRef, TypeRefKind,
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

    // Function parameters that do not occur in the callable signature cannot
    // be inferred at any call site. Report them once at the declaration.
    checker.check_uninferable_function_parameters();

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

struct EnumVariantType {
    name: String,
    payload_start: usize,
    payload_len: usize,
}

struct EnumVariants {
    variants: Vec<EnumVariantType>,
    payload_types: Vec<Type>,
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

    /// Rejects function type parameters absent from all input and result types.
    ///
    /// Traverse the HIR type references once per function. This also counts a
    /// parameter inside a malformed type application, whose recovery `Type`
    /// may be `Error` and therefore no longer retain the reference.
    fn check_uninferable_function_parameters(&mut self) {
        for definition in self.hir.defs() {
            if definition.kind != DefKind::Function || definition.type_parameters.is_empty() {
                continue;
            }

            let mut used = HashSet::new();
            for parameter in &definition.parameters {
                if let Some(annotation) = self.hir.def(*parameter).and_then(|it| it.ty.as_ref()) {
                    self.collect_type_parameters_from_ref(annotation, &mut used);
                }
            }
            if let Some(result) = &definition.ty {
                self.collect_type_parameters_from_ref(result, &mut used);
            }

            for parameter in &definition.type_parameters {
                if used.contains(parameter) {
                    continue;
                }
                let Some(parameter_definition) = self.hir.def(*parameter) else {
                    continue;
                };
                self.diagnostics.push(
                    Diagnostic::error(
                        codes::TYPE_PARAMETER_NOT_IN_SIGNATURE,
                        format!(
                            "type parameter `{}` is not used in this function's signature",
                            parameter_definition.name
                        ),
                    )
                    .with_location(self.source, parameter_definition.span)
                    .with_help("use the type parameter in an input or result type, or remove it"),
                );
            }
        }
    }

    /// Collects type-parameter references directly from HIR type syntax.
    fn collect_type_parameters_from_ref(&self, reference: &TypeRef, found: &mut HashSet<DefId>) {
        if let Some(id) = reference.target.resolved() {
            if self
                .hir
                .def(id)
                .is_some_and(|definition| definition.kind == DefKind::TypeParameter)
            {
                found.insert(id);
            }
        }
        for argument in &reference.arguments {
            self.collect_type_parameters_from_ref(argument, found);
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
                let expected = definition
                    .ty
                    .as_ref()
                    .map(|annotation| self.type_of_annotation(annotation))
                    .unwrap_or(Type::Unit);
                let actual = self.type_of_expression_with_expected(value, Some(&expected));
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
                LiteralKind::Int(magnitude) => {
                    self.type_of_integer_literal(magnitude, expression.span)
                }
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
            } => self.type_of_call(
                callee,
                &arguments,
                arguments_span,
                expression.span,
                expected,
            ),
            ExprKind::Field {
                receiver,
                name,
                name_span,
            } => self.type_of_field(receiver, &name, name_span),
            ExprKind::If {
                condition,
                then_block,
                else_branch,
            } => self.type_of_if(condition, then_block, else_branch, expected),
            ExprKind::Index { base, index } => {
                self.type_of_expression(base);
                self.type_of_expression(index);
                Type::Error
            }
            ExprKind::Unary {
                operator,
                operator_span,
                operand,
            } => self.type_of_unary(&operator, operator_span, operand),
            ExprKind::Binary {
                operator,
                operator_span,
                left,
                right,
            } => self.type_of_binary(&operator, operator_span, left, right),
            ExprKind::Match { scrutinee, arms } => {
                self.type_of_match(scrutinee, &arms, expected, expression.span)
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

    /// Checks `if` conditions and branches, preserving a known expected type.
    fn type_of_if(
        &mut self,
        condition: ExprId,
        then_block: ExprId,
        else_branch: Option<ExprId>,
        expected: Option<&Type>,
    ) -> Type {
        let expected_condition = Type::Bool;
        let actual_condition =
            self.type_of_expression_with_expected(condition, Some(&expected_condition));
        let condition_span = self
            .hir
            .expr(condition)
            .map_or_else(zero_span, |expression| expression.span);
        self.expect(&expected_condition, &actual_condition, condition_span);

        let then_span = self
            .hir
            .expr(then_block)
            .map_or_else(zero_span, |expression| expression.span);
        let Some(else_branch) = else_branch else {
            let unit = Type::Unit;
            let then_type = self.type_of_expression_with_expected(then_block, Some(&unit));
            let matches_unit = self.is_assignable(&then_type, &unit);
            self.expect(&unit, &then_type, then_span);
            return if matches_unit { unit } else { Type::Error };
        };

        let else_span = self
            .hir
            .expr(else_branch)
            .map_or_else(zero_span, |expression| expression.span);
        if let Some(expected) = expected {
            let then_type = self.type_of_expression_with_expected(then_block, Some(expected));
            let else_type = self.type_of_expression_with_expected(else_branch, Some(expected));
            let then_matches = self.is_assignable(&then_type, expected);
            let else_matches = self.is_assignable(&else_type, expected);
            self.expect(expected, &then_type, then_span);
            self.expect(expected, &else_type, else_span);
            if then_matches && else_matches {
                return expected.clone();
            }
            return Type::Error;
        }

        let then_type = self.type_of_expression(then_block);
        let else_type = self.type_of_expression(else_branch);
        if self.contains_error_type(&then_type) || self.contains_error_type(&else_type) {
            return Type::Error;
        }
        if then_type == else_type {
            return then_type;
        }

        self.diagnostics.push(
            Diagnostic::error(
                codes::IF_BRANCH_TYPE_MISMATCH,
                "the branches of this `if` have different types",
            )
            .with_location(self.source, else_span)
            .with_note(format!("then branch: `{then_type}`"))
            .with_note(format!("else branch: `{else_type}`"))
            .with_help("make both branches produce the same type, or add an expected type"),
        );
        Type::Error
    }

    /// Types an enum match without choosing a result type from arm order.
    fn type_of_match(
        &mut self,
        scrutinee: ExprId,
        arms: &[Arm],
        expected: Option<&Type>,
        match_span: Span,
    ) -> Type {
        let scrutinee_type = self.type_of_expression(scrutinee);
        let scrutinee_span = self
            .hir
            .expr(scrutinee)
            .map_or_else(zero_span, |expression| expression.span);
        let scrutinee_has_error = self.contains_error_type(&scrutinee_type);
        let variants = if scrutinee_has_error {
            None
        } else {
            self.enum_variants(&scrutinee_type)
        };
        let mut has_error =
            scrutinee_has_error || expected.is_some_and(|ty| self.contains_error_type(ty));

        if !scrutinee_has_error && variants.is_none() {
            self.diagnostics.push(
                Diagnostic::error(codes::MATCH_REQUIRES_ENUM, "match requires an enum")
                    .with_location(self.source, scrutinee_span)
                    .with_note(format!("found: `{scrutinee_type}`"))
                    .with_help("match a value whose type is an enum or `Result<T, E>`"),
            );
            has_error = true;
        }

        let variant_count = variants
            .as_ref()
            .map_or(0, |variants| variants.variants.len());
        let variant_indices = variants
            .as_ref()
            .map(|variants| {
                variants
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(index, variant)| (variant.name.as_str(), index))
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default();
        let mut covered = vec![0_u64; variant_count.div_ceil(64)];
        let mut covered_count = 0;
        let mut wildcard_seen = false;
        let mut inferred_arm_type: Option<Type> = None;

        for arm in arms {
            let after_wildcard = wildcard_seen && variants.is_some();
            if after_wildcard {
                let span = match &arm.pattern {
                    MatchPattern::Wildcard { span } => *span,
                    MatchPattern::Variant { name_span, .. } => *name_span,
                };
                self.report_unreachable_match_arm(
                    span,
                    "a preceding `_` matches every remaining variant",
                );
                has_error = true;
            }

            match &arm.pattern {
                MatchPattern::Wildcard { span } => {
                    if variants.is_some() && !after_wildcard && covered_count == variant_count {
                        self.report_unreachable_match_arm(
                            *span,
                            "every enum variant is already covered",
                        );
                        has_error = true;
                    }
                    wildcard_seen = true;
                    covered.fill(u64::MAX);
                    covered_count = variant_count;
                }
                MatchPattern::Variant {
                    name,
                    name_span,
                    span,
                    payloads,
                } => {
                    let variant = variant_indices.get(name.as_str()).copied();
                    let payload_types = match (variants.as_ref(), variant) {
                        (Some(variants), Some(index)) => {
                            let bit = 1_u64 << (index % 64);
                            let word = &mut covered[index / 64];
                            let previously_covered = *word & bit != 0;
                            if !after_wildcard && previously_covered {
                                self.report_unreachable_match_arm(
                                    *name_span,
                                    "this variant is already covered by an earlier arm",
                                );
                                has_error = true;
                            } else if !after_wildcard {
                                *word |= bit;
                                covered_count += 1;
                            }
                            let variant = &variants.variants[index];
                            let payload_end = variant.payload_start + variant.payload_len;
                            Some(&variants.payload_types[variant.payload_start..payload_end])
                        }
                        (Some(_), None) => {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    codes::INVALID_MATCH_PATTERN,
                                    "unknown variant in match",
                                )
                                .with_location(self.source, *name_span)
                                .with_note(format!("found enum: `{scrutinee_type}`"))
                                .with_help("use a variant declared by the matched enum"),
                            );
                            has_error = true;
                            None
                        }
                        (None, _) => None,
                    };

                    if let Some(payload_types) = payload_types {
                        if payloads.len() != payload_types.len() {
                            let error_span = if payloads.len() > payload_types.len() {
                                match &payloads[payload_types.len()] {
                                    PayloadPattern::Binding { span, .. }
                                    | PayloadPattern::Wildcard { span } => *span,
                                }
                            } else {
                                *span
                            };
                            self.diagnostics.push(
                                Diagnostic::error(
                                    codes::WRONG_VARIANT_PATTERN_ARITY,
                                    "variant pattern has the wrong payload count",
                                )
                                .with_location(self.source, error_span)
                                .with_note(format!(
                                    "expected payload patterns: {}",
                                    payload_types.len()
                                ))
                                .with_note(format!("found payload patterns: {}", payloads.len()))
                                .with_help(
                                    "bind or discard exactly one pattern for each payload field",
                                ),
                            );
                            has_error = true;
                        }
                        for (index, payload) in payloads.iter().enumerate() {
                            let ty = payload_types.get(index).cloned().unwrap_or(Type::Error);
                            if self.contains_error_type(&ty) {
                                has_error = true;
                            }
                            if let PayloadPattern::Binding { definition, .. } = payload {
                                if let Some(slot) =
                                    self.definition_types.get_mut(definition.index() as usize)
                                {
                                    *slot = Some(ty);
                                }
                            }
                        }
                    } else {
                        for payload in payloads {
                            if let PayloadPattern::Binding { definition, .. } = payload {
                                if let Some(slot) =
                                    self.definition_types.get_mut(definition.index() as usize)
                                {
                                    *slot = Some(Type::Error);
                                }
                            }
                        }
                    }
                }
            }

            let body_span = self
                .hir
                .expr(arm.body)
                .map_or_else(zero_span, |expression| expression.span);
            let body_type = if let Some(expected) = expected {
                let body_type = self.type_of_expression_with_expected(arm.body, Some(expected));
                let assignable = self.is_assignable(&body_type, expected);
                self.expect(expected, &body_type, body_span);
                if !assignable || self.contains_error_type(&body_type) {
                    has_error = true;
                }
                body_type
            } else {
                self.type_of_expression(arm.body)
            };

            if self.contains_error_type(&body_type) {
                has_error = true;
            } else if expected.is_none() {
                if let Some(first_type) = &inferred_arm_type {
                    if first_type != &body_type {
                        self.diagnostics.push(
                            Diagnostic::error(
                                codes::MATCH_ARM_TYPE_MISMATCH,
                                "match arms have different types",
                            )
                            .with_location(self.source, body_span)
                            .with_note(format!("expected: `{first_type}`"))
                            .with_note(format!("found: `{body_type}`"))
                            .with_help("make every arm produce the same type, or give the match an expected type"),
                        );
                        has_error = true;
                    }
                } else {
                    inferred_arm_type = Some(body_type);
                }
            }
        }

        if let Some(variants) = &variants {
            if covered_count < variant_count {
                let missing = variants
                    .variants
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| covered[*index / 64] & (1_u64 << (*index % 64)) == 0)
                    .map(|(_, variant)| variant.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                self.diagnostics.push(
                    Diagnostic::error(
                        codes::NON_EXHAUSTIVE_MATCH,
                        "this match does not cover every variant",
                    )
                    .with_location(self.source, match_span)
                    .with_note(format!("missing variants: {missing}"))
                    .with_help("add a pattern for each missing variant, or use a final `_` for the remaining variants"),
                );
                has_error = true;
            }
        }

        if has_error {
            Type::Error
        } else if let Some(expected) = expected {
            expected.clone()
        } else {
            inferred_arm_type.unwrap_or(Type::Error)
        }
    }

    /// Returns declaration-order variants and their payload types for a match.
    fn enum_variants(&self, ty: &Type) -> Option<EnumVariants> {
        let Type::Named {
            definition,
            arguments,
            ..
        } = ty
        else {
            return None;
        };
        let id = definition.resolved()?;
        let definition = self.hir.def(id)?;

        if definition.kind == DefKind::BuiltinType && definition.name == "Result" {
            if arguments.len() != 2 {
                return None;
            }
            return Some(EnumVariants {
                variants: vec![
                    EnumVariantType {
                        name: "Ok".to_string(),
                        payload_start: 0,
                        payload_len: 1,
                    },
                    EnumVariantType {
                        name: "Err".to_string(),
                        payload_start: 1,
                        payload_len: 1,
                    },
                ],
                payload_types: vec![arguments[0].clone(), arguments[1].clone()],
            });
        }
        if definition.kind != DefKind::Enum || definition.type_parameters.len() != arguments.len() {
            return None;
        }

        let substitutions = definition
            .type_parameters
            .iter()
            .copied()
            .zip(arguments.iter().cloned())
            .collect::<HashMap<_, _>>();
        let payload_count = definition
            .variants
            .iter()
            .map(|variant| variant.fields.len())
            .sum();
        let mut variants = Vec::with_capacity(definition.variants.len());
        let mut payload_types = Vec::with_capacity(payload_count);
        for variant in &definition.variants {
            let payload_start = payload_types.len();
            for field in &variant.fields {
                let ty = field.ty.as_ref().map_or(Type::Error, |annotation| {
                    let ty = self.type_of_annotation(annotation);
                    self.substitute_type(&ty, &substitutions)
                });
                payload_types.push(ty);
            }
            variants.push(EnumVariantType {
                name: variant.name.clone(),
                payload_start,
                payload_len: payload_types.len() - payload_start,
            });
        }
        Some(EnumVariants {
            variants,
            payload_types,
        })
    }

    fn report_unreachable_match_arm(&mut self, span: Span, reason: &str) {
        self.diagnostics.push(
            Diagnostic::error(
                codes::UNREACHABLE_MATCH_ARM,
                "this match arm is unreachable",
            )
            .with_location(self.source, span)
            .with_note(reason)
            .with_help("remove the arm or change earlier patterns so this arm can be reached"),
        );
    }

    /// Types unary operators and reports invalid domains at the operator token.
    fn type_of_unary(&mut self, operator: &str, operator_span: Span, operand: ExprId) -> Type {
        if operator == "-" && self.is_minimum_integer_magnitude(operand) {
            self.set_expression_type(operand, Type::Int);
            return Type::Int;
        }

        let operand_type = self.type_of_expression(operand);
        if self.contains_error_type(&operand_type) {
            return Type::Error;
        }
        match operator {
            "-" => {
                if let Some(numeric_type) = self.numeric_type(&operand_type) {
                    return numeric_type;
                }
            }
            "!" if self.is_assignable(&operand_type, &Type::Bool) => return Type::Bool,
            "!" => {}
            _ => return Type::Error,
        }
        self.report_invalid_operator_types(operator, operator_span, &[operand_type]);
        Type::Error
    }

    /// Types an integer magnitude, allowing `2^63` only in the dedicated unary
    /// minus rule above.
    fn type_of_integer_literal(&mut self, magnitude: IntegerMagnitude, span: Span) -> Type {
        match magnitude {
            IntegerMagnitude::Value(value) if value < (1_u64 << 63) => Type::Int,
            IntegerMagnitude::Invalid => Type::Error,
            IntegerMagnitude::Value(value) if value == (1_u64 << 63) => {
                self.report_integer_literal_out_of_range(span, true);
                Type::Error
            }
            IntegerMagnitude::Value(_) | IntegerMagnitude::OutOfRange => {
                self.report_integer_literal_out_of_range(span, false);
                Type::Error
            }
        }
    }

    /// Whether this expression is the `2^63` magnitude used directly under
    /// unary minus. HIR has already erased parentheses around the literal.
    fn is_minimum_integer_magnitude(&self, expression: ExprId) -> bool {
        self.hir.expr(expression).is_some_and(|expression| {
            matches!(
                expression.kind,
                ExprKind::Literal(LiteralKind::Int(IntegerMagnitude::Value(value)))
                    if value == (1_u64 << 63)
            )
        })
    }

    fn set_expression_type(&mut self, expression: ExprId, ty: Type) {
        if let Some(slot) = self.expression_types.get_mut(expression.index() as usize) {
            *slot = Some(ty);
        }
    }

    fn report_integer_literal_out_of_range(&mut self, span: Span, is_minimum_magnitude: bool) {
        let diagnostic = Diagnostic::error(
            codes::INTEGER_LITERAL_OUT_OF_RANGE,
            "integer literal is outside the range of `Int`",
        )
        .with_location(self.source, span);
        let help = if is_minimum_magnitude {
            "write this magnitude as the direct operand of unary `-` to denote `Int::MIN`"
        } else {
            "use a magnitude no greater than `9223372036854775807`; exactly `9223372036854775808` is valid only directly under unary `-` to denote `Int::MIN`"
        };
        self.diagnostics.push(diagnostic.with_help(help));
    }

    /// Types unary and binary operators without coercion or overload search.
    fn type_of_binary(
        &mut self,
        operator: &str,
        operator_span: Span,
        left: ExprId,
        right: ExprId,
    ) -> Type {
        let left_type = self.type_of_expression(left);
        let right_type = self.type_of_expression(right);
        if self.contains_error_type(&left_type) || self.contains_error_type(&right_type) {
            return Type::Error;
        }
        let result = match operator {
            "+" | "-" | "*" | "/" => {
                let left_numeric = self.numeric_type(&left_type);
                let right_numeric = self.numeric_type(&right_type);
                match (left_numeric, right_numeric) {
                    (Some(left), Some(right)) if left == right => Some(left),
                    _ => None,
                }
            }
            "%" => {
                if self.numeric_type(&left_type) == Some(Type::Int)
                    && self.numeric_type(&right_type) == Some(Type::Int)
                {
                    Some(Type::Int)
                } else {
                    None
                }
            }
            "&&" | "||" => {
                if self.is_assignable(&left_type, &Type::Bool)
                    && self.is_assignable(&right_type, &Type::Bool)
                {
                    Some(Type::Bool)
                } else {
                    None
                }
            }
            "==" | "!=" => match (
                self.equality_type(&left_type),
                self.equality_type(&right_type),
            ) {
                (Some(left), Some(right)) if left == right => Some(Type::Bool),
                _ => None,
            },
            "<" | ">" | "<=" | ">=" => {
                let left_numeric = self.numeric_type(&left_type);
                let right_numeric = self.numeric_type(&right_type);
                match (left_numeric, right_numeric) {
                    (Some(left), Some(right)) if left == right => Some(Type::Bool),
                    _ => None,
                }
            }
            _ => return Type::Error,
        };
        if let Some(result) = result {
            return result;
        }

        self.report_invalid_operator_types(operator, operator_span, &[left_type, right_type]);
        Type::Error
    }

    fn report_invalid_operator_types(&mut self, operator: &str, span: Span, operands: &[Type]) {
        let mut diagnostic = Diagnostic::error(
            codes::INVALID_OPERATOR_TYPES,
            format!("operator `{operator}` cannot be applied to these operand types"),
        )
        .with_location(self.source, span);
        for (index, operand) in operands.iter().enumerate() {
            let label = match (operands.len(), index) {
                (1, _) => "operand",
                (_, 0) => "left operand",
                _ => "right operand",
            };
            diagnostic = diagnostic.with_note(format!("{label}: `{operand}`"));
        }
        self.diagnostics
            .push(diagnostic.with_help("use operand types supported by this operator"));
    }

    /// The primitive equality domain after applying the ordinary supported
    /// `Verified<T>`-to-`T` assignment view.
    fn equality_type(&self, ty: &Type) -> Option<Type> {
        [Type::Int, Type::Float, Type::Bool, Type::Text]
            .into_iter()
            .find(|primitive| self.is_assignable(ty, primitive))
    }

    /// Types a call: the callee's signature decides everything.
    ///
    /// Three things can go wrong, and each one says which it is: the callee is
    /// not a function (`NDO2006`), the count is wrong (`NDO2005`), or an argument
    /// is the wrong type (`NDO2004`, reported at the argument, because that is
    /// the thing to change).
    ///
    /// Type parameters are inferred from arguments, left to right, and then from
    /// an already-known expected result type, as NEP-0013 specifies. Unresolved
    /// parameters are diagnosed at the call site, and their result is `Error` to
    /// prevent a cascade.
    fn type_of_call(
        &mut self,
        callee: ExprId,
        arguments: &[ExprId],
        arguments_span: Span,
        call_span: Span,
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
            let expected_noun = if parameters.len() == 1 {
                "argument"
            } else {
                "arguments"
            };
            let found_noun = if arguments.len() == 1 {
                "argument"
            } else {
                "arguments"
            };
            self.diagnostics.push(
                Diagnostic::error(
                    codes::WRONG_ARGUMENT_COUNT,
                    format!(
                        "this call passes {} {found_noun}, but the function takes {} {expected_noun}",
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

        // Only a direct function item owns call-site inference variables.
        // Type parameters appearing in a function-valued parameter or local
        // value belong to their enclosing declaration and stay fixed here.
        let ordered_generic_parameters = self
            .hir
            .expr(callee)
            .and_then(|expression| match &expression.kind {
                ExprKind::Path { target, .. } => target.resolved(),
                _ => None,
            })
            .and_then(|definition| self.hir.def(definition))
            .filter(|definition| definition.kind == DefKind::Function)
            .map(|definition| definition.type_parameters.to_vec())
            .unwrap_or_default();
        let generic_parameters = ordered_generic_parameters
            .iter()
            .copied()
            .collect::<HashSet<_>>();

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
        let mut blocked_parameters = HashSet::new();
        for (parameter, argument) in parameters.iter().zip(arguments) {
            let actual = self.type_of_expression_with_expected(*argument, Some(parameter));
            if self.contains_error_type(&actual) {
                self.collect_type_parameters(parameter, &mut blocked_parameters);
            }
            self.infer_type_arguments(parameter, &actual, &generic_parameters, &mut substitutions);
        }
        if let Some(expected_result) = expected_result {
            if self.contains_error_type(expected_result) {
                self.collect_type_parameters(&result, &mut blocked_parameters);
            }
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
                self.collect_type_parameters(&expected, &mut blocked_parameters);
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
        let mut reported_uninferable_parameter = false;
        for parameter in &ordered_generic_parameters {
            if !unresolved_parameters.contains(parameter) || blocked_parameters.contains(parameter)
            {
                continue;
            }
            let Some(definition) = self.hir.def(*parameter) else {
                continue;
            };
            self.diagnostics.push(
                Diagnostic::error(
                    codes::TYPE_PARAMETER_NOT_INFERRED,
                    format!(
                        "type parameter `{}` cannot be inferred here",
                        definition.name
                    ),
                )
                .with_location(self.source, call_span)
                .with_help("add a type annotation to the value that uses this call"),
            );
            reported_uninferable_parameter = true;
        }
        if reported_uninferable_parameter {
            return Type::Error;
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

    /// Whether an error type appears anywhere inside a recovered type.
    fn contains_error_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Error => true,
            Type::Named { arguments, .. } => arguments
                .iter()
                .any(|argument| self.contains_error_type(argument)),
            Type::Function { parameters, result } => {
                parameters
                    .iter()
                    .any(|parameter| self.contains_error_type(parameter))
                    || self.contains_error_type(result)
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
    /// `Verified<T>`-to-`T` relation. Supported expression positions such as
    /// field access and function calls use this same view; it never descends
    /// into generic arguments or function components.
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
    fn generic_enum_payloads_are_substituted_and_wildcards_cover_remaining_variants() {
        let program = "enum Maybe<T> {\n    Some(value: T)\n    None\n}\n\n\
             fn unwrap(value: Maybe<Int>) -> Int {\n    match value {\n        Some(number) => number\n        None => 0\n    }\n}\n\n\
             fn with_wildcard(value: Maybe<Int>) -> Int {\n    match value {\n        None => 0\n        _ => 1\n    }\n}\n";
        let (_, _, hir, result) = checked_with_hir(program);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let number = hir
            .defs()
            .iter()
            .enumerate()
            .find(|(_, definition)| definition.name == "number")
            .expect("payload binding exists");
        assert_eq!(
            result.type_of_def(DefId::from_index(number.0 as u32)),
            Some(&Type::Int),
            "the enum's `T` payload is substituted with `Int`"
        );
    }

    #[test]
    fn result_patterns_bind_their_instantiated_payload_types() {
        let program = "fn describe(value: Result<Int, Text>) -> Text {\n\
             match value {\n    Ok(number) => \"success\"\n    Err(reason) => reason\n}\n}\n";
        let (_, _, hir, result) = checked_with_hir(program);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        let reason = hir
            .defs()
            .iter()
            .enumerate()
            .find(|(_, definition)| definition.name == "reason")
            .expect("error payload binding exists");
        assert_eq!(
            result.type_of_def(DefId::from_index(reason.0 as u32)),
            Some(&Type::Text)
        );
    }

    #[test]
    fn non_enum_matches_report_one_type_error_without_pattern_cascades() {
        let program = "fn inspect(value: Int) -> Int {\n\
             match value {\n    Missing => 1\n}\n}\n";
        let (sources, id, result) = checked(program);
        assert_eq!(
            result
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code().id())
                .collect::<Vec<_>>(),
            ["NDO2011"]
        );
        let diagnostic = &result.diagnostics.as_slice()[0];
        assert_eq!(
            diagnostic.help(),
            Some("match a value whose type is an enum or `Result<T, E>`")
        );
        let file = sources.get(id).expect("source exists");
        assert_eq!(
            file.span_text(diagnostic.span().expect("scrutinee span")),
            Some("value")
        );
    }

    #[test]
    fn missing_match_variants_follow_declaration_order() {
        let program = "enum Direction {\n    North\n    East\n    South\n    West\n}\n\n\
             fn rank(value: Direction) -> Int {\n    match value {\n        East => 1\n        West => 3\n    }\n}\n";
        let (_, _, result) = checked(program);
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
        assert_eq!(result.diagnostics.as_slice()[0].code().id(), "NDO2014");
        assert_eq!(
            result.diagnostics.as_slice()[0].notes(),
            &["missing variants: North, South".to_string()]
        );
    }

    #[test]
    fn match_pattern_and_body_errors_are_reported_and_recover_as_error() {
        let program = "enum Flag {\n    On\n    Off\n}\n\n\
             fn inspect(value: Flag) -> Int {\n    match value {\n        Missing => true + 1\n        On => 1\n    }\n}\n";
        let (_, _, hir, result) = checked_with_hir(program);
        assert_eq!(
            result
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code().id())
                .collect::<Vec<_>>(),
            ["NDO2012", "NDO2018", "NDO2014"]
        );
        let function = hir
            .defs()
            .iter()
            .find(|definition| definition.name == "inspect")
            .expect("function exists");
        let body = hir
            .expr(function.value.expect("function has a body"))
            .unwrap();
        let ExprKind::Block {
            value: Some(match_id),
            ..
        } = body.kind
        else {
            panic!("function body is a block ending in the match");
        };
        assert_eq!(result.type_of_expr(match_id), Some(&Type::Error));
    }

    #[test]
    fn match_payload_arity_and_missing_variants_have_precise_diagnostics() {
        let program = "enum Pair {\n    Both(left: Int, right: Bool)\n    Other\n}\n\n\
             fn too_few(value: Pair) -> Int {\n    match value {\n        Both(left) => left\n        Other => 0\n    }\n}\n\n\
             fn too_many(value: Pair) -> Int {\n    match value {\n        Both(left, right, _) => left\n        Other => 0\n    }\n}\n\n\
             fn incomplete(value: Pair) -> Int {\n    match value {\n        Other => 0\n    }\n}\n";
        let (sources, id, result) = checked(program);
        let diagnostics = result.diagnostics.as_slice();
        assert_eq!(diagnostics.len(), 3, "{diagnostics:?}");
        assert_eq!(diagnostics[0].code().id(), "NDO2013");
        assert_eq!(diagnostics[1].code().id(), "NDO2013");
        assert_eq!(diagnostics[2].code().id(), "NDO2014");
        assert!(
            diagnostics[2]
                .notes()
                .iter()
                .any(|note| note == "missing variants: Both"),
            "{:?}",
            diagnostics[2].notes()
        );
        let file = sources.get(id).expect("source exists");
        assert_eq!(
            file.span_text(
                diagnostics[0]
                    .span()
                    .expect("excess or missing payload span")
            ),
            Some("Both(left)")
        );
        assert_eq!(
            file.span_text(diagnostics[1].span().expect("first excess payload span")),
            Some("_")
        );
        assert_eq!(
            diagnostics[1].notes(),
            &[
                "expected payload patterns: 2".to_string(),
                "found payload patterns: 3".to_string()
            ]
        );
    }

    #[test]
    fn duplicate_and_unreachable_match_arms_are_reported() {
        let program = "enum One {\n    Only\n}\n\n\
             enum Two {\n    First\n    Second\n}\n\n\
             fn redundant(value: One) -> Int {\n    match value {\n        Only => 1\n        _ => 2\n    }\n}\n\n\
             fn after_wildcard(value: Two) -> Int {\n    match value {\n        First => 1\n        _ => 2\n        Second => 3\n    }\n}\n\n\
             fn duplicate(value: Two) -> Int {\n    match value {\n        First => 1\n        First => 2\n        _ => 3\n    }\n}\n";
        let (_, _, result) = checked(program);
        assert_eq!(
            result
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code().id())
                .collect::<Vec<_>>(),
            ["NDO2015", "NDO2015", "NDO2015"]
        );
    }

    #[test]
    fn match_arm_types_must_agree_without_context_and_use_context_when_available() {
        let program = "enum Flag {\n    Yes\n    No\n}\n\n\
             fn inferred(value: Flag) {\n    let result = match value {\n        Yes => 1\n        No => true\n    };\n}\n\n\
             fn contextual(value: Flag, checked: Verified<Int>) -> Int {\n    match value {\n        Yes => checked\n        No => 0\n    }\n}\n\n\
             fn expected_mismatch(value: Flag) -> Int {\n    match value {\n        Yes => 1\n        No => true\n    }\n}\n";
        let (_, _, result) = checked(program);
        assert_eq!(
            result
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code().id())
                .collect::<Vec<_>>(),
            ["NDO2004", "NDO2016"]
        );
        let diagnostic = &result.diagnostics.as_slice()[1];
        assert_eq!(
            diagnostic.notes(),
            &["expected: `Int`".to_string(), "found: `Bool`".to_string()]
        );
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
    fn integer_literal_boundaries_follow_signed_int() {
        let program = "let maximum: Int = 9223372036854775807;\n\
             let minimum: Int = -9223372036854775808;\n\
             let parenthesized: Int = -((9_223_372_036_854_775_808));\n\
             let twice_negated: Int = -(-9223372036854775808);\n";
        let (_, _, hir, result) = checked_with_hir(program);
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        for (index, definition) in hir.defs().iter().enumerate() {
            if definition.kind == DefKind::Binding {
                assert_eq!(
                    result.type_of_def(DefId::from_index(index as u32)),
                    Some(&Type::Int),
                    "{} must have type Int",
                    definition.name
                );
            }
        }
    }

    #[test]
    fn invalid_integer_magnitudes_report_at_the_literal_without_cascades() {
        let program = "let positive = 9223372036854775808;\n\
             let subtraction = 0 - 9223372036854775808;\n\
             let too_large = -9223372036854775809;\n";
        let (sources, id, result) = checked(program);
        assert_eq!(
            result
                .diagnostics
                .as_slice()
                .iter()
                .map(|diagnostic| diagnostic.code().id())
                .collect::<Vec<_>>(),
            ["NDO2010", "NDO2010", "NDO2010"]
        );
        let file = sources.get(id).expect("just added");
        let literal_spans = result
            .diagnostics
            .as_slice()
            .iter()
            .map(|diagnostic| {
                file.span_text(diagnostic.span().expect("literal diagnostic has a span"))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            literal_spans,
            [
                Some("9223372036854775808"),
                Some("9223372036854775808"),
                Some("9223372036854775809"),
            ]
        );
        assert_eq!(
            result.diagnostics.as_slice()[0].message(),
            "integer literal is outside the range of `Int`"
        );
        assert_eq!(
            result.diagnostics.as_slice()[0].help(),
            Some("write this magnitude as the direct operand of unary `-` to denote `Int::MIN`")
        );
        assert_eq!(
            result.diagnostics.as_slice()[2].help(),
            Some(
                "use a magnitude no greater than `9223372036854775807`; exactly `9223372036854775808` is valid only directly under unary `-` to denote `Int::MIN`"
            )
        );
    }

    #[test]
    fn the_minimum_integer_magnitude_does_not_widen_to_float() {
        let (_, _, result) = checked("let value: Float = -9223372036854775808;\n");
        assert_eq!(
            result
                .diagnostics
                .as_slice()
                .iter()
                .map(|diagnostic| diagnostic.code().id())
                .collect::<Vec<_>>(),
            ["NDO2004"]
        );
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
    fn mixed_numeric_arithmetic_reports_at_the_operator_with_both_types() {
        let (sources, id, result) = checked("let value = 1 + 2.0;\n");
        assert_eq!(
            result
                .diagnostics
                .as_slice()
                .iter()
                .map(|diagnostic| diagnostic.code().id())
                .collect::<Vec<_>>(),
            vec!["NDO2018"]
        );
        let diagnostic = &result.diagnostics.as_slice()[0];
        let notes = diagnostic.notes().join(" | ");
        assert!(notes.contains("left operand: `Int`"), "{notes}");
        assert!(notes.contains("right operand: `Float`"), "{notes}");
        let file = sources.get(id).expect("just added");
        let span = diagnostic.span().expect("a span");
        assert_eq!(file.span_text(span), Some("+"));
        let expected_column = "let value = 1 + 2.0;".find('+').expect("operator") + 1;
        assert_eq!(
            file.line_col(span.start()).to_string(),
            format!("1:{expected_column}")
        );
    }

    #[test]
    fn an_uncontextualised_if_branch_mismatch_is_reported_once_at_else() {
        let program = "fn main() {\n    let value = if true { 1 } else { true };\n}\n";
        let (sources, id, result) = checked(program);
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
        let diagnostic = &result.diagnostics.as_slice()[0];
        assert_eq!(diagnostic.code().id(), "NDO2019");
        let notes = diagnostic.notes().join(" | ");
        assert!(notes.contains("then branch: `Int`"), "{notes}");
        assert!(notes.contains("else branch: `Bool`"), "{notes}");
        let file = sources.get(id).expect("just added");
        assert_eq!(
            file.span_text(diagnostic.span().expect("else branch span")),
            Some("{ true }")
        );
    }

    #[test]
    fn an_omitted_function_result_is_unit_and_rejects_a_value_body() {
        let program = "fn prepare() {\n    let ready = true;\n}\n\nfn answer() {\n    42\n}\n";
        let (sources, id, hir, result) = checked_with_hir(program);
        assert_eq!(
            result
                .diagnostics
                .as_slice()
                .iter()
                .map(|diagnostic| diagnostic.code().id())
                .collect::<Vec<_>>(),
            vec!["NDO2004"]
        );
        for name in ["prepare", "answer"] {
            let (index, _definition) = hir
                .defs()
                .iter()
                .enumerate()
                .find(|(_, definition)| definition.name == name)
                .unwrap_or_else(|| panic!("missing function `{name}`"));
            assert_eq!(
                result.type_of_def(DefId::from_index(index as u32)),
                Some(&Type::Function {
                    parameters: Arc::from([]),
                    result: Arc::new(Type::Unit),
                }),
                "{name} keeps its declared Unit result"
            );
        }
        let diagnostic = &result.diagnostics.as_slice()[0];
        let notes = diagnostic.notes().join(" | ");
        assert!(notes.contains("expected: `Unit`"), "{notes}");
        assert!(notes.contains("found: `Int`"), "{notes}");
        let source = sources.get(id).expect("just added");
        assert_eq!(
            source.span_text(diagnostic.span().expect("function body span")),
            Some("{\n    42\n}")
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
    fn a_call_argument_mismatch_points_at_the_complete_argument_expression() {
        let program = "fn takes(value: Int) {}\n\
                       fn main() {\n    takes(true);\n}\n";
        let (sources, id, result) = checked(program);
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
        let diagnostic = &result.diagnostics.as_slice()[0];
        assert_eq!(diagnostic.code().id(), "NDO2004");
        let file = sources.get(id).expect("just added");
        let span = diagnostic.span().expect("the argument has a source span");
        assert_eq!(file.span_text(span), Some("true"));
    }

    #[test]
    fn an_arity_diagnostic_points_at_the_argument_list() {
        for (call, expected_span) in [("add(1)", "(1)"), ("add(1, 2, 3)", "(1, 2, 3)")] {
            let program = format!(
                "fn add(left: Int, right: Int) -> Int {{ left }}\n\
                 fn main() {{ let value = {call}; }}\n"
            );
            let (sources, id, result) = checked(&program);
            let diagnostic = result
                .diagnostics
                .as_slice()
                .iter()
                .find(|diagnostic| diagnostic.code().id() == "NDO2005")
                .expect("the call has the wrong number of arguments");
            let file = sources.get(id).expect("just added");
            let span = diagnostic.span().expect("the argument list has a span");
            assert_eq!(file.span_text(span), Some(expected_span));
        }
    }

    #[test]
    fn an_arity_diagnostic_pluralizes_expected_and_found_counts() {
        let program = "fn takes_two(left: Int, right: Int) {}\n\
                       fn takes_none() {}\n\
                       fn takes_one(value: Int) {}\n\
                       fn main() {\n\
                           takes_two(1);\n\
                           takes_two(1, 2, 3);\n\
                           takes_none(1);\n\
                           takes_one();\n\
                       }\n";
        let (_, _, result) = checked(program);
        assert_eq!(result.diagnostics.len(), 4, "{:?}", result.diagnostics);
        let messages = result
            .diagnostics
            .as_slice()
            .iter()
            .filter(|diagnostic| diagnostic.code().id() == "NDO2005")
            .map(|diagnostic| diagnostic.message())
            .collect::<Vec<_>>();

        assert_eq!(
            messages,
            vec![
                "this call passes 1 argument, but the function takes 2 arguments",
                "this call passes 3 arguments, but the function takes 2 arguments",
                "this call passes 1 argument, but the function takes 0 arguments",
                "this call passes 0 arguments, but the function takes 1 argument",
            ]
        );
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
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
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
    fn uninferable_parameters_are_reported_in_declaration_order_at_the_call() {
        let program = "struct Pair<A, B> {\n    first: A\n    second: B\n}\n\nfn make<A, B>() -> Pair<A, B> {\n    make()\n}\n\nfn main() {\n    let value = make();\n}\n";
        let (sources, id, result) = checked(program);
        let diagnostics = result.diagnostics.as_slice();
        assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");
        assert_eq!(diagnostics[0].code().id(), "NDO2008");
        assert_eq!(
            diagnostics[0].message(),
            "type parameter `A` cannot be inferred here"
        );
        assert_eq!(diagnostics[1].code().id(), "NDO2008");
        assert_eq!(
            diagnostics[1].message(),
            "type parameter `B` cannot be inferred here"
        );
        let file = sources.get(id).expect("just added");
        for diagnostic in diagnostics {
            assert_eq!(
                diagnostic.help(),
                Some("add a type annotation to the value that uses this call")
            );
            let span = diagnostic.span().expect("call span");
            assert_eq!(file.span_text(span), Some("make()"));
        }
    }

    #[test]
    fn a_parameter_absent_from_the_function_signature_is_reported_once() {
        let program = "fn answer<T>() -> Int {\n    42\n}\n";
        let (sources, id, result) = checked(program);
        let diagnostics = result.diagnostics.as_slice();
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        let diagnostic = &diagnostics[0];
        assert_eq!(diagnostic.code().id(), "NDO2009");
        assert_eq!(
            diagnostic.message(),
            "type parameter `T` is not used in this function's signature"
        );
        assert_eq!(
            diagnostic.help(),
            Some("use the type parameter in an input or result type, or remove it")
        );
        let file = sources.get(id).expect("just added");
        assert_eq!(
            file.span_text(diagnostic.span().expect("declaration span")),
            Some("T")
        );
    }

    #[test]
    fn a_function_value_call_does_not_infer_its_enclosing_type_parameters() {
        let program = "fn apply<T>(operation: (Fn(T) -> T)) -> T {\n    operation(42)\n}\n";
        assert_eq!(codes_for(program), vec!["NDO2004".to_string()]);
    }

    #[test]
    fn repeated_generic_arguments_must_agree() {
        let program = "fn same<T>(first: T, second: T) -> T {\n    first\n}\n\nfn main() {\n    let value = same(1, true);\n}\n";
        let (sources, id, result) = checked(program);
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);

        let diagnostic = &result.diagnostics.as_slice()[0];
        assert_eq!(diagnostic.code().id(), "NDO2004");
        assert_eq!(diagnostic.message(), "type mismatch");
        assert_eq!(
            diagnostic.notes(),
            &["expected: `Int`".to_string(), "found: `Bool`".to_string()]
        );
        assert_eq!(
            diagnostic.help(),
            Some("an annotation is a promise, and this value does not keep it")
        );
        assert_eq!(diagnostic.source(), Some(id));

        let file = sources.get(id).expect("just added");
        let span = diagnostic
            .span()
            .expect("the conflicting argument has a span");
        assert_eq!(file.span_text(span), Some("true"));
        let line_text = program.lines().nth(5).expect("call line");
        let column = line_text.find("true").expect("conflicting argument") + 1;
        assert_eq!(
            file.line_col(span.start()).to_string(),
            format!("6:{column}")
        );
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
        let (sources, id, result) = checked(program);
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);

        let diagnostic = &result.diagnostics.as_slice()[0];
        assert_eq!(diagnostic.code().id(), "NDO2004");
        assert_eq!(diagnostic.message(), "type mismatch");
        assert_eq!(
            diagnostic.notes(),
            &["expected: `Text`".to_string(), "found: `Int`".to_string()]
        );
        assert_eq!(
            diagnostic.help(),
            Some("an annotation is a promise, and this value does not keep it")
        );
        assert_eq!(diagnostic.source(), Some(id));

        let file = sources.get(id).expect("just added");
        let span = diagnostic.span().expect("the inferred call has a span");
        assert_eq!(file.span_text(span), Some("identity(1)"));
        let line_text = program.lines().nth(5).expect("call line");
        let column = line_text.find("identity(1)").expect("inferred call") + 1;
        assert_eq!(
            file.line_col(span.start()).to_string(),
            format!("6:{column}")
        );
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
    fn a_type_parameter_named_verified_does_not_get_the_builtin_call_view() {
        let program = "fn inspect<Verified>(value: Verified) {\n    value();\n}\n";
        let (_, _, result) = checked(program);
        let diagnostic = &result.diagnostics.as_slice()[0];
        assert_eq!(diagnostic.code().id(), "NDO2006");
        assert_eq!(diagnostic.notes(), &["found: `Verified`".to_string()]);
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
        let file = sources.get(id).expect("just added");
        let notes = result.diagnostics.as_slice()[0].notes().join(" | ");
        let span = result.diagnostics.as_slice()[0]
            .span()
            .expect("the unknown field has a span");
        assert_eq!(file.span_text(span), Some("titel"));
        assert!(
            notes.contains("no field `titel`") || notes.contains("found: `Article`"),
            "{notes}"
        );
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
