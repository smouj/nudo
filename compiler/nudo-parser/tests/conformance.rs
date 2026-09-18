//! Language-level conformance cases for the parser.
//!
//! Every directory under `tests/conformance/parser/` is one case:
//!
//! | File | Meaning |
//! | ---- | ------- |
//! | `main.nudo`         | the input |
//! | `tree.txt`          | the expected syntax tree, `nudo-tree v1` format |
//! | `diagnostics.txt`   | the expected diagnostics, one per line (optional) |
//!
//! The format is specified in `tests/conformance/README.md` and is meant to be
//! reproducible by an implementation that is not this one. This test is only
//! the reference driver for the reference implementation.
//!
//! One property is checked for every case, whether or not it has expectations:
//! the tree reprints its source byte for byte. A lossless tree is what makes a
//! formatter, an editor and a safe automated rewrite possible, so a case that
//! breaks it is a case that breaks the reason for the tree's shape.

use std::fs;
use std::path::{Path, PathBuf};

use nudo_parser::parse;
use nudo_source::SourceMap;
use nudo_syntax::dump_tree;

/// The minimum number of cases the corpus must contain, so that deleting a
/// case cannot make this test quietly pass.
const MINIMUM_CASES: usize = 12;

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance/parser")
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

/// Reads a corpus file with line endings normalised to `\n`.
///
/// Expectations are compared line by line, so a carriage return is a checkout
/// artefact, not content. The token corpus learned this the hard way: with CRLF
/// the comparison failed on Windows and nowhere else.
fn read(path: &Path) -> String {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    raw.replace("\r\n", "\n")
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
fn cases_match_their_expected_trees_and_diagnostics() {
    for case in case_dirs() {
        let name = case
            .file_name()
            .map_or_else(String::new, |it| it.to_string_lossy().into_owned());
        let input = case.join("main.nudo");
        let expected_tree = case.join("tree.txt");
        let expected_diagnostics = case.join("diagnostics.txt");

        let text = read(&input);
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", text.clone());
        let file = sources.get(id).expect("just added");
        let parsed = parse(file);

        assert!(
            parsed.tree().is_lossless(),
            "case {name}: the tree does not reprint its source"
        );
        assert_eq!(
            parsed.tree().validate(),
            Ok(()),
            "case {name}: the tree is not well formed"
        );

        assert_eq!(
            dump_tree(parsed.tree()),
            read(&expected_tree),
            "case {name}: the tree differs from tree.txt"
        );

        let actual_diagnostics = normalized_diagnostics(file, &parsed);
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
fn normalized_diagnostics(file: &nudo_source::SourceFile, parsed: &nudo_parser::Parse) -> String {
    let mut out = String::from("# nudo-diagnostics v1\n");
    for diagnostic in parsed.diagnostics() {
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
