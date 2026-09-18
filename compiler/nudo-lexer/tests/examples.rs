//! The example corpus.
//!
//! `examples/README.md` makes two promises to a reader, and this test is what
//! keeps them honest:
//!
//! 1. the examples marked "read by the toolchain today" lex with zero
//!    diagnostics;
//! 2. every other example says, in its own header, that it is a preview.
//!
//! A preview that stops declaring itself a preview is worse than a broken
//! example: it turns an intention into an apparent feature.
//!
//! The lexer can only answer the lexical half of the first promise. The half
//! that matters after M2 — a preview is not silently *accepted* — needs a
//! parser, and lives in `compiler/nudo-parser/tests/examples.rs`.
//!
//! M2 extended the token set to the one the grammar declares
//! ([`grammar/nudo.ebnf`](../../../grammar/nudo.ebnf)), so a preview no longer
//! has to contain an unknown character to be unimplemented. Most of them now
//! lex cleanly and are rejected by the parser instead, which is a stronger
//! statement than the one this file used to make.

use std::fs;
use std::path::PathBuf;

use nudo_lexer::tokenize;
use nudo_source::SourceMap;

/// Examples the toolchain can read: they must lex cleanly.
const READABLE: &[&str] = &["00-hello-world", "01-variables", "02-functions", "03-types"];

/// Examples that illustrate proposed syntax: they must say so.
const PREVIEWS: &[&str] = &[
    "04-results",
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

fn lex(name: &str, text: String) -> nudo_lexer::Lexed {
    let mut sources = SourceMap::new();
    let id = sources.add(format!("{name}/main.nudo"), text);
    let file = sources.get(id).expect("just added");
    tokenize(file)
}

#[test]
fn every_example_directory_has_a_main_file() {
    for name in READABLE.iter().chain(PREVIEWS) {
        let path = examples_dir().join(name).join("main.nudo");
        assert!(path.is_file(), "{} is missing", path.display());
    }
}

#[test]
fn readable_examples_lex_without_diagnostics() {
    for name in READABLE {
        let text = example_source(name);
        let lexed = lex(name, text.clone());
        assert!(
            lexed.diagnostics().is_empty(),
            "examples/{name}/main.nudo is documented as readable, but reported {} diagnostic(s): {:?}",
            lexed.diagnostics().len(),
            lexed
                .diagnostics()
                .iter()
                .map(|it| it.code().id())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn readable_examples_are_documented_as_readable() {
    let index = fs::read_to_string(examples_dir().join("README.md")).expect("examples README");
    for name in READABLE {
        assert!(
            index.contains(name),
            "{name} must appear in examples/README.md"
        );
    }
}

#[test]
fn previews_declare_themselves_as_previews() {
    for name in PREVIEWS {
        let text = example_source(name);
        let header: String = text.lines().take(12).collect::<Vec<_>>().join("\n");
        assert!(
            header.contains("PREVIEW") && header.contains("NOT ACCEPTED"),
            "examples/{name}/main.nudo must declare itself a preview in its header"
        );
    }
}

#[test]
fn example_corpus_is_classified_completely() {
    let mut on_disk: Vec<String> = fs::read_dir(examples_dir())
        .expect("examples/ is readable")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    on_disk.sort();
    let mut classified: Vec<String> = READABLE
        .iter()
        .chain(PREVIEWS)
        .map(|name| (*name).to_string())
        .collect();
    classified.sort();
    assert_eq!(
        on_disk, classified,
        "every example directory must be classified as readable or as a preview"
    );
}
