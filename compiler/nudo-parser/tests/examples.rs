//! The example corpus, from the parser's side.
//!
//! `examples/README.md` promises a reader two things:
//!
//! 1. the examples it lists as readable are read by the toolchain today;
//! 2. every other one is a *preview*, and not accepted.
//!
//! Before M2 the second promise was checked by the lexer, which was a weak
//! statement: a file can use only known tokens and still not be a program. Now
//! it is checked properly — a preview must be **rejected by the parser**, and a
//! readable example must produce no diagnostics at all.

use std::fs;
use std::path::PathBuf;

use nudo_parser::parse;
use nudo_source::SourceMap;

/// Examples the toolchain can read: they must lex and parse cleanly.
const READABLE: &[&str] = &[
    "00-hello-world",
    "01-variables",
    "02-functions",
    "03-types",
    "04-results",
];

/// Examples that illustrate proposed syntax: they must be rejected.
const PREVIEWS: &[&str] = &[
    "05-agent",
    "06-tools",
    "07-generated-verified",
    "08-policy",
    "09-multi-agent",
];

fn examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn example_source(name: &str) -> String {
    let path = examples_dir().join(name).join("main.nudo");
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
}

fn parse_source(name: &str, text: String) -> nudo_parser::Parse {
    let mut sources = SourceMap::new();
    let id = sources.add(format!("{name}/main.nudo"), text);
    let file = sources.get(id).expect("just added");
    parse(file)
}

#[test]
fn readable_examples_produce_no_diagnostics() {
    for name in READABLE {
        let parsed = parse_source(name, example_source(name));
        assert!(
            !parsed.has_errors(),
            "examples/{name}/main.nudo is documented as readable, but reported {}. \
             Update the example and examples/README.md if the language changed.",
            parsed
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.code().id())
                .collect::<Vec<_>>()
                .join(", ")
        );
        assert_eq!(parsed.tree().validate(), Ok(()), "examples/{name}");
    }
}

#[test]
fn previews_are_rejected_by_the_parser() {
    for name in PREVIEWS {
        let parsed = parse_source(name, example_source(name));
        assert!(
            parsed.has_errors(),
            "examples/{name}/main.nudo is marked as a preview but the toolchain accepted it; \
             promote it to the readable table in examples/README.md if that is intended"
        );
    }
}

#[test]
fn every_preview_declares_why_it_is_not_accepted() {
    for name in PREVIEWS {
        let text = example_source(name);
        let header: String = text.lines().take(20).collect::<Vec<_>>().join("\n");
        assert!(
            header.contains("PREVIEW") && header.contains("NOT ACCEPTED"),
            "examples/{name}/main.nudo must declare itself a preview in its header"
        );
        assert!(
            header.contains("Why not:"),
            "examples/{name}/main.nudo must say which syntax stops it from being accepted"
        );
    }
}

#[test]
fn previews_are_named_in_the_index_with_capitalised_keys() {
    // The index is the only description of the corpus as a whole, so it has to
    // mention every example the toolchain cannot read.
    let index = fs::read_to_string(examples_dir().join("README.md")).expect("examples README");
    for name in PREVIEWS {
        assert!(
            index.contains(name),
            "{name} must appear in examples/README.md"
        );
    }
}
