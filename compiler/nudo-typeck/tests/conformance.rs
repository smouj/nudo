//! Language-level conformance cases for the type checker.
//!
//! Every directory under `tests/conformance/typeck/` is one case:
//!
//! | File | Meaning |
//! | ---- | ------- |
//! | `main.nudo`        | the input, which must parse and resolve cleanly |
//! | `types.txt`        | the expected `nudo-typeck v1` dump |
//! | `diagnostics.txt`  | the expected diagnostics (optional) |
//!
//! The format is specified in `tests/conformance/README.md`. Unlike the lexer and
//! parser corpora, the CLI exposes no type dump yet — the checker is deliberately
//! not wired into `nudo check` until calls and structural rules exist — so these
//! expectations are written by this driver with `NUDO_BLESS=1` and **reviewed as
//! a diff**, like every other expectation in the repository. Blessing is not a
//! way to make a failing case pass: it is the same command, minus the comparison.

use std::fs;
use std::path::{Path, PathBuf};

use nudo_diagnostics::Diagnostics;
use nudo_hir::lower;
use nudo_parser::parse;
use nudo_source::{SourceFile, SourceMap};
use nudo_typeck::{check, dump_types};

/// The number of cases the corpus must contain, so that deleting one cannot make
/// this test quietly pass.
const MINIMUM_CASES: usize = 2;

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance/typeck")
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
        .replace("\r\n", "\n")
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
fn cases_match_their_expected_types_and_diagnostics() {
    let bless = std::env::var_os("NUDO_BLESS").is_some();
    for case in case_dirs() {
        let name = case
            .file_name()
            .map_or_else(String::new, |it| it.to_string_lossy().into_owned());
        let text = read(&case.join("main.nudo"));
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", text);
        let file = sources.get(id).expect("just added");

        let parsed = parse(file);
        assert!(
            !parsed.has_errors(),
            "case {name}: a type case must parse; the parser owns syntax errors"
        );
        let resolved = lower(parsed.tree(), id);
        assert!(
            resolved.diagnostics.is_empty(),
            "case {name}: a type case must resolve; the resolver owns name errors"
        );

        let checked = check(&resolved.hir, id);
        let actual_types = dump_types(&checked, &resolved.hir, file);
        let actual_diagnostics = normalized(file, &checked.diagnostics);

        if bless {
            fs::write(case.join("types.txt"), &actual_types).expect("write types.txt");
            if actual_diagnostics == "# nudo-diagnostics v1\n" {
                let _ = fs::remove_file(case.join("diagnostics.txt"));
            } else {
                fs::write(case.join("diagnostics.txt"), &actual_diagnostics)
                    .expect("write diagnostics.txt");
            }
            continue;
        }

        assert_eq!(
            actual_types,
            read(&case.join("types.txt")),
            "case {name}: types differ from types.txt"
        );

        let expected = case.join("diagnostics.txt");
        if expected.exists() {
            assert_eq!(
                actual_diagnostics,
                read(&expected),
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

/// Renders diagnostics in the normalized form the corpus compares.
fn normalized(file: &SourceFile, diagnostics: &Diagnostics) -> String {
    let mut out = String::from("# nudo-diagnostics v1\n");
    for diagnostic in diagnostics.as_slice() {
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
