//! The type the checker assigns to something.
//!
//! One enum, and the whole point of it is what it is *ready* for. This first
//! slice types primitives and nothing else, but the shape already carries the
//! things M3.2 and M3.3 will need — a named type with arguments, a function
//! type, and an error type that keeps a bad definition from becoming a cascade —
//! so that adding `Struct`, `Enum`, `Result<T, E>`, `Generated<T>` and
//! `Verified<T>` is filling in cases rather than redesigning the centre.
//!
//! Two deliberate omissions, both visible rather than hidden:
//!
//! * **`Named` compares definitions, not structure.** Two named types are
//!   compatible when they resolve to the same definition. Field and variant
//!   checking is the next slice; identity is already the right answer for the
//!   cases that exist today, and it is the answer that cannot be wrong.
//! * **There is no type variable.** Inference (NEP-0013) is bounded and local,
//!   and until generic instantiation exists there is nothing to solve, so a type
//!   variable here would be a placeholder pretending to be a decision.

use std::fmt;

use nudo_hir::Resolution;

/// A type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// `Int`, a signed 64-bit integer (NEP-0007).
    Int,
    /// `Float`, an IEEE-754 binary floating-point number.
    Float,
    /// `Bool`.
    Bool,
    /// `Text`, UTF-8.
    Text,
    /// `Unit`: nothing to see, and not the same as absent.
    Unit,
    /// A named type, and what that name resolved to.
    Named {
        /// The name as written.
        name: String,
        /// Its definition, when the name resolved.
        definition: Resolution,
        /// Its type arguments, in source order.
        arguments: Vec<Type>,
    },
    /// `Fn(A, B) -> C`.
    Function {
        /// The parameter types, in order.
        parameters: Vec<Type>,
        /// The result type.
        result: Box<Type>,
    },
    /// A type error already reported. Compatible with everything, so that one
    /// mistake is one diagnostic instead of a cascade.
    Error,
}

impl Type {
    /// Whether two types are compatible in the sense this slice checks: the same
    /// type, or a type error on either side.
    ///
    /// `Error` is compatible on purpose. Once a diagnostic has been reported,
    /// every later comparison involving that value would report the same mistake
    /// again, in a different place.
    #[must_use]
    pub fn is_compatible_with(&self, expected: &Type) -> bool {
        if matches!(self, Type::Error) || matches!(expected, Type::Error) {
            return true;
        }
        match (self, expected) {
            (Type::Int, Type::Int)
            | (Type::Float, Type::Float)
            | (Type::Bool, Type::Bool)
            | (Type::Text, Type::Text)
            | (Type::Unit, Type::Unit) => true,
            (
                Type::Named {
                    definition: actual,
                    arguments: actual_arguments,
                    ..
                },
                Type::Named {
                    definition: expected,
                    arguments: expected_arguments,
                    ..
                },
            ) => {
                // Identity, not structure: two names are the same type when they
                // mean the same definition. Arguments are compared the same way,
                // and invariantly, because NEP-0006 says so.
                actual == expected
                    && actual_arguments.len() == expected_arguments.len()
                    && actual_arguments
                        .iter()
                        .zip(expected_arguments)
                        .all(|(actual, expected)| actual.is_compatible_with(expected))
            }
            (
                Type::Function {
                    parameters: actual,
                    result: actual_result,
                },
                Type::Function {
                    parameters: expected,
                    result: expected_result,
                },
            ) => {
                actual.len() == expected.len()
                    && actual
                        .iter()
                        .zip(expected)
                        .all(|(actual, expected)| actual.is_compatible_with(expected))
                    && actual_result.is_compatible_with(expected_result)
            }
            _ => false,
        }
    }

    /// Whether this is the error type.
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Type::Error)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => formatter.write_str("Int"),
            Type::Float => formatter.write_str("Float"),
            Type::Bool => formatter.write_str("Bool"),
            Type::Text => formatter.write_str("Text"),
            Type::Unit => formatter.write_str("Unit"),
            Type::Named {
                name, arguments, ..
            } => {
                formatter.write_str(name)?;
                if !arguments.is_empty() {
                    formatter.write_str("<")?;
                    for (index, argument) in arguments.iter().enumerate() {
                        if index > 0 {
                            formatter.write_str(", ")?;
                        }
                        write!(formatter, "{argument}")?;
                    }
                    formatter.write_str(">")?;
                }
                Ok(())
            }
            Type::Function { parameters, result } => {
                formatter.write_str("Fn(")?;
                for (index, parameter) in parameters.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "{parameter}")?;
                }
                write!(formatter, ") -> {result}")
            }
            Type::Error => formatter.write_str("<error>"),
        }
    }
}

/// The type a built-in name stands for, if it is one.
#[must_use]
pub fn builtin(name: &str) -> Option<Type> {
    match name {
        "Int" => Some(Type::Int),
        "Float" => Some(Type::Float),
        "Bool" => Some(Type::Bool),
        "Text" => Some(Type::Text),
        "Unit" => Some(Type::Unit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitives_are_only_compatible_with_themselves() {
        assert!(Type::Int.is_compatible_with(&Type::Int));
        assert!(!Type::Int.is_compatible_with(&Type::Bool));
        assert!(!Type::Text.is_compatible_with(&Type::Int));
    }

    #[test]
    fn there_is_no_implicit_numeric_conversion() {
        assert!(!Type::Int.is_compatible_with(&Type::Float));
        assert!(!Type::Float.is_compatible_with(&Type::Int));
    }

    #[test]
    fn unit_is_a_type_of_its_own() {
        assert!(Type::Unit.is_compatible_with(&Type::Unit));
        assert!(!Type::Unit.is_compatible_with(&Type::Int));
    }

    #[test]
    fn an_error_is_compatible_with_everything() {
        assert!(Type::Error.is_compatible_with(&Type::Int));
        assert!(Type::Int.is_compatible_with(&Type::Error));
        assert!(Type::Error.is_compatible_with(&Type::Error));
    }

    #[test]
    fn a_named_type_is_compatible_with_the_same_definition() {
        let named = |name: &str| Type::Named {
            name: name.to_string(),
            definition: Resolution::Unresolved,
            arguments: Vec::new(),
        };
        assert!(named("Article").is_compatible_with(&named("Article")));
    }

    #[test]
    fn named_types_are_invariant_in_their_arguments() {
        let generated = |argument: Type| Type::Named {
            name: "Generated".to_string(),
            definition: Resolution::Unresolved,
            arguments: vec![argument],
        };
        assert!(generated(Type::Int).is_compatible_with(&generated(Type::Int)));
        assert!(!generated(Type::Int).is_compatible_with(&generated(Type::Bool)));
    }

    #[test]
    fn a_function_type_compares_its_parts() {
        let function = |parameters: Vec<Type>, result: Type| Type::Function {
            parameters,
            result: Box::new(result),
        };
        assert!(
            function(vec![Type::Int], Type::Bool)
                .is_compatible_with(&function(vec![Type::Int], Type::Bool))
        );
        assert!(
            !function(vec![Type::Int], Type::Bool)
                .is_compatible_with(&function(vec![Type::Text], Type::Bool))
        );
        assert!(
            !function(vec![Type::Int, Type::Int], Type::Bool)
                .is_compatible_with(&function(vec![Type::Int], Type::Bool))
        );
    }

    #[test]
    fn types_print_the_way_the_specification_writes_them() {
        assert_eq!(Type::Int.to_string(), "Int");
        assert_eq!(
            Type::Named {
                name: "Result".to_string(),
                definition: Resolution::Unresolved,
                arguments: vec![Type::Int],
            }
            .to_string(),
            "Result<Int>"
        );
        assert_eq!(
            Type::Function {
                parameters: vec![Type::Int],
                result: Box::new(Type::Text),
            }
            .to_string(),
            "Fn(Int) -> Text"
        );
        assert_eq!(
            Type::Named {
                name: "Verified".to_string(),
                definition: Resolution::Unresolved,
                arguments: vec![Type::Named {
                    name: "Article".to_string(),
                    definition: Resolution::Unresolved,
                    arguments: Vec::new(),
                }],
            }
            .to_string(),
            "Verified<Article>"
        );
    }

    #[test]
    fn builtins_map_to_their_type_and_nothing_else_does() {
        assert_eq!(builtin("Int"), Some(Type::Int));
        assert_eq!(builtin("Unit"), Some(Type::Unit));
        assert_eq!(builtin("Generated"), None, "an intrinsic, not a primitive");
        assert_eq!(builtin("Article"), None);
    }
}
