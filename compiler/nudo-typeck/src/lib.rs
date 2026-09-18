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
//! # What this first slice checks
//!
//! Deliberately small, and complete within itself:
//!
//! * **Literal typing.** An integer literal is `Int`, a float is `Float`, `true`
//!   is `Bool`, text is `Text`.
//! * **Annotations.** `let x: Int = 1;` types the annotation and the value and
//!   compares them.
//! * **Blocks.** A block's type is its final expression's, or `Unit` when it has
//!   none — which is why a function with no written return type takes the type
//!   of its body, the same statement the specification makes read from the other
//!   end.
//! * **Names.** A path takes the type of the definition it resolved to, which is
//!   why resolution had to exist first.
//! * **One diagnostic.** `NDO2004`, with `expected` and `found` as structure
//!   rather than as a sentence to re-parse.
//!
//! # What it deliberately does not check yet
//!
//! Everything else, and each omission is a named next step rather than a hole:
//! calls beyond their type (arity and argument compatibility), generics and
//! instantiation, structs and enums, `Result<T, E>` exhaustiveness, effects, and
//! the trust rules of M3.3. Until then those expressions type as [`Type::Error`],
//! which is compatible with everything and therefore reports nothing: silence,
//! not a wrong answer.

pub mod types;

use nudo_diagnostics::{Diagnostic, Diagnostics, codes};
use nudo_hir::{Def, DefId, DefKind, ExprId, ExprKind, Hir, LiteralKind, StmtKind, TypeRef};
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

    let mut checker = Checker {
        hir,
        source,
        definition_types: vec![None; hir.defs().len()],
        expression_types: vec![None; last_expression.map_or(0, |last| last as usize + 1)],
        diagnostics: Diagnostics::new(),
    };

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
    definition_types: Vec<Option<Type>>,
    expression_types: Vec<Option<Type>>,
    diagnostics: Diagnostics,
}

impl Checker<'_> {
    /// The type a definition declares, if it declares one.
    fn signature_of(&self, definition: &Def) -> Option<Type> {
        match definition.kind {
            DefKind::Function => {
                let parameters = definition
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
                    parameters,
                    result: Box::new(result),
                })
            }
            DefKind::Parameter | DefKind::Binding | DefKind::Const => definition
                .ty
                .as_ref()
                .map(|annotation| self.type_of_annotation(annotation)),
            _ => None,
        }
    }

    /// The type an annotation names.
    ///
    /// A built-in name is the type it names; anything else is a named type whose
    /// arguments are lowered the same way. Nothing is looked up structurally:
    /// identity is what this slice compares, and identity is already in HIR.
    fn type_of_annotation(&self, annotation: &TypeRef) -> Type {
        if let Some(builtin) = builtin(&annotation.name) {
            return builtin;
        }
        Type::Named {
            name: annotation.name.clone(),
            definition: annotation.target,
            arguments: annotation
                .arguments
                .iter()
                .map(|argument| self.type_of_annotation(argument))
                .collect(),
        }
    }

    fn check_definition(&mut self, definition: &Def, id: DefId) {
        if matches!(definition.kind, DefKind::Parameter | DefKind::BuiltinType) {
            return;
        }
        let Some(value) = definition.value else {
            return;
        };
        let actual = self.type_of_expression(value);
        let span = self.hir.expr(value).map_or(definition.span, |it| it.span);

        match definition.kind {
            DefKind::Function => {
                let expected = match &definition.ty {
                    Some(annotation) => self.type_of_annotation(annotation),
                    None => actual.clone(),
                };
                self.expect(&expected, &actual, span);
                let parameters = definition
                    .parameters
                    .iter()
                    .map(|parameter| {
                        self.definition_types[parameter.index() as usize]
                            .clone()
                            .unwrap_or(Type::Error)
                    })
                    .collect();
                self.definition_types[id.index() as usize] = Some(Type::Function {
                    parameters,
                    result: Box::new(expected),
                });
            }
            _ => match &definition.ty {
                Some(annotation) => {
                    let expected = self.type_of_annotation(annotation);
                    // Point at the value, not at the name: the value is what
                    // breaks the promise, and that is the place a reader has to
                    // change.
                    self.expect(&expected, &actual, span);
                    self.definition_types[id.index() as usize] = Some(expected);
                }
                None => self.definition_types[id.index() as usize] = Some(actual),
            },
        }
    }

    /// Types an expression, writing the answer down as it goes.
    fn type_of_expression(&mut self, id: ExprId) -> Type {
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
                    Some(value) => self.type_of_expression(value),
                    None => Type::Unit,
                }
            }
            ExprKind::Call { callee, arguments } => self.type_of_call(callee, &arguments),
            // Deferred, and silent about it: these produce `Error`, which is
            // compatible with everything, so nothing is reported about work this
            // slice does not do yet.
            ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Unary { .. }
            | ExprKind::Binary { .. }
            | ExprKind::If { .. }
            | ExprKind::Match { .. }
            | ExprKind::Ask { .. }
            | ExprKind::Verify { .. }
            | ExprKind::Delegate { .. } => Type::Error,
        };
        if let Some(slot) = self.expression_types.get_mut(id.index() as usize) {
            *slot = Some(typed.clone());
        }
        typed
    }

    /// Types a call: the callee's signature decides everything.
    ///
    /// Three things can go wrong, and each one says which it is: the callee is
    /// not a function (`NDO2006`), the count is wrong (`NDO2005`), or an argument
    /// is the wrong type (`NDO2004`, reported at the argument, because that is
    /// the thing to change).
    ///
    /// A signature mentioning a type parameter is **not checked**: instantiation
    /// is the next slice (NEP-0013), and comparing an argument against `T` would
    /// invent an error about a rule that does not exist yet. The call is typed as
    /// unknown instead, which reports nothing.
    fn type_of_call(&mut self, callee: ExprId, arguments: &[ExprId]) -> Type {
        let callee_type = self.type_of_expression(callee);
        let Type::Function { parameters, result } = callee_type.clone() else {
            if !callee_type.is_error() {
                let span = self.hir.expr(callee).map_or_else(zero_span, |it| it.span);
                self.diagnostics.push(
                    Diagnostic::error(codes::NOT_CALLABLE, "this value is not a function")
                        .with_location(self.source, span)
                        .with_note(format!("found: `{callee_type}`"))
                        .with_help("only a function type can be called"),
                );
            }
            return Type::Error;
        };

        if parameters
            .iter()
            .any(|parameter| self.mentions_type_parameter(parameter))
            || self.mentions_type_parameter(&result)
        {
            return Type::Error;
        }

        if arguments.len() != parameters.len() {
            let expected = if parameters.len() == 1 {
                "argument"
            } else {
                "arguments"
            };
            let span = self.hir.expr(callee).map_or_else(zero_span, |it| it.span);
            self.diagnostics.push(
                Diagnostic::error(
                    codes::WRONG_ARGUMENT_COUNT,
                    format!(
                        "this call passes {} and the function takes {} {expected}",
                        arguments.len(),
                        parameters.len()
                    ),
                )
                .with_location(self.source, span)
                .with_note(format!("expected: {}", parameters.len()))
                .with_note(format!("found: {}", arguments.len()))
                .with_help("every call passes exactly what the signature takes"),
            );
            return *result;
        }

        for (index, argument) in arguments.iter().enumerate() {
            let actual = self.type_of_expression(*argument);
            let expected = parameters[index].clone();
            let span = self
                .hir
                .expr(*argument)
                .map_or_else(zero_span, |it| it.span);
            self.expect(&expected, &actual, span);
        }
        *result
    }

    /// Whether a type mentions a type parameter of the declaration it came from.
    ///
    /// This is what tells the checker that a signature is not instantiable yet,
    /// and therefore that comparing against it would be inventing a rule.
    fn mentions_type_parameter(&self, ty: &Type) -> bool {
        match ty {
            Type::Named {
                definition,
                arguments,
                ..
            } => {
                if let Some(id) = definition.resolved() {
                    if self
                        .hir
                        .def(id)
                        .is_some_and(|definition| definition.kind == DefKind::TypeParameter)
                    {
                        return true;
                    }
                }
                arguments
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

    /// Reports a mismatch, with the two types as structure rather than prose.
    fn expect(&mut self, expected: &Type, found: &Type, span: Span) {
        if found.is_compatible_with(expected) {
            return;
        }
        self.diagnostics.push(
            Diagnostic::error(codes::TYPE_MISMATCH, "type mismatch")
                .with_location(self.source, span)
                .with_note(format!("expected: `{expected}`"))
                .with_note(format!("found: `{found}`"))
                .with_help("an annotation is a promise, and this value does not keep it"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nudo_hir::lower;
    use nudo_parser::parse;
    use nudo_source::SourceMap;

    fn checked(text: &str) -> (SourceMap, SourceId, TypeckResult) {
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
        (sources, id, result)
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
    fn there_is_no_implicit_numeric_conversion() {
        assert_eq!(
            codes_for("let x: Float = 1;\n"),
            vec!["NDO2004".to_string()]
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
    fn this_slice_does_not_report_about_work_it_does_not_do() {
        // Calls are the next slice. They must be silent, not wrong: `Error` is
        // compatible with everything, so nothing is reported about them.
        let (_, _, result) = checked(
            "fn add(a: Int, b: Int) -> Int {\n    a\n}\n\nfn main() {\n    let total = add(1, 2);\n}\n",
        );
        assert_eq!(
            result.diagnostics.len(),
            0,
            "a later slice's work must not produce a diagnostic now: {:?}",
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
        assert_eq!(codes_for(program), vec!["NDO2006".to_string()]);
    }

    #[test]
    fn a_call_to_a_generic_function_is_not_checked_yet() {
        // Instantiation is the next slice (NEP-0013). Comparing an argument
        // against `T` would invent an error about a rule that does not exist, so
        // the call is typed as unknown and says nothing.
        let program = "fn identity<T>(value: T) -> T {\n    value\n}\n\nfn main() {\n    let one = identity(1);\n}\n";
        assert_eq!(codes_for(program), Vec::<String>::new());
    }

    #[test]
    fn a_call_to_a_function_declared_later_works() {
        let program = "fn main() {\n    later();\n}\n\nfn later() {\n}\n";
        assert_eq!(codes_for(program), Vec::<String>::new());
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
