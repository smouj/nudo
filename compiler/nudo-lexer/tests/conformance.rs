//! Language-level conformance cases.
//!
//! Every directory under `tests/conformance/lexer/` is one case:
//!
//! | File | Meaning |
//! | ---- | ------- |
//! | `main.nudo`         | the input |
//! | `tokens.txt`        | the expected token dump, `nudo-tokens v1` format |
//! | `diagnostics.txt`   | the expected diagnostics, one per line |
//!
//! The format is specified in `tests/conformance/README.md` and is meant to be
//! reproducible by an implementation that is not this one. This test is only
//! the reference driver for the reference implementation.

use std::fs;
use std::path::{Path, PathBuf};

use nudo_lexer::{dump_tokens, tokenize};
use nudo_source::SourceMap;

/// The minimum number of cases the corpus must contain, so that deleting a
/// case cannot make this test quietly pass.
const MINIMUM_CASES: usize = 5;

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance/lexer")
}

fn case_dirs() -> Vec<PathBuf> {
    let dir = corpus_dir();
    let mut cases: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    cases.sort();
    cases
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
}

#[test]
fn corpus_is_not_empty() {
    let cases = case_dirs();
    assert!(
        cases.len() >= MINIMUM_CASES,
        "{} holds {} cases, expected at least {MINIMUM_CASES}",
        corpus_dir().display(),
        cases.len()
    );
}

#[test]
fn cases_match_their_expected_tokens_and_diagnostics() {
    for case in case_dirs() {
        let name = case
            .file_name()
            .map_or_else(String::new, |it| it.to_string_lossy().into_owned());
        let input = case.join("main.nudo");
        let expected_tokens = case.join("tokens.txt");
        let expected_diagnostics = case.join("diagnostics.txt");

        let text = read(&input);
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", text);
        let file = sources.get(id).expect("just added");
        let lexed = tokenize(file);

        let actual_tokens = dump_tokens(file, lexed.tokens());
        assert_eq!(
            actual_tokens,
            read(&expected_tokens),
            "case {name}: token dump differs from tokens.txt"
        );

        let actual_diagnostics = normalized_diagnostics(file, &lexed);
        if expected_diagnostics.exists() {
            assert_eq!(
                actual_diagnostics,
                read(&expected_diagnostics),
                "case {name}: diagnostics differ from diagnostics.txt"
            );
        } else {
            assert!(
                actual_diagnostics == "# nudo-diagnostics v1\n",
                "case {name}: produced diagnostics but has no diagnostics.txt:\n{actual_diagnostics}"
            );
        }
    }
}

/// Renders diagnostics in the normalized, implementation-neutral form used by
/// the corpus: severity, code and 1-based start position.
fn normalized_diagnostics(file: &nudo_source::SourceFile, lexed: &nudo_lexer::Lexed) -> String {
    let mut out = String::from("# nudo-diagnostics v1\n");
    for diagnostic in lexed.diagnostics() {
        let position = diagnostic
            .span()
            .map(|span| file.line_col(span.start()))
            .map_or_else(|| "?:?".to_string(), |line_col| line_col.to_string());
        out.push_str(&format!(
            "{} {} {}\n",
            diagnostic.severity().as_str(),
            diagnostic.code().id(),
            position
        ));
    }
    out
}
