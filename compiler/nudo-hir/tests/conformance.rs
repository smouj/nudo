//! Language-level conformance cases for name resolution.
//!
//! Every directory under `tests/conformance/resolve/` is one case:
//!
//! | File | Meaning |
//! | ---- | ------- |
//! | `main.nudo`        | the input |
//! | `resolutions.txt`  | the expected `nudo-hir v1` dump: definitions and what each name resolved to |
//! | `diagnostics.txt`  | the expected diagnostics (optional) |
//!
//! The format is specified in `tests/conformance/README.md` and is meant to be
//! reproducible by an implementation that is not this one. This test is only
//! the reference driver for the reference implementation.
//!
//! The dump is deliberately about *names*, not about the shape of the HIR: a
//! second implementation is free to lay its arenas out differently, and still
//! has to agree about which definition a name refers to.

use std::fs;
use std::path::{Path, PathBuf};

use nudo_hir::{dump_resolutions, lower};
use nudo_parser::parse;
use nudo_source::SourceMap;

/// The minimum number of cases the corpus must contain, so that deleting a case
/// cannot make this test quietly pass.
const MINIMUM_CASES: usize = 4;

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance/resolve")
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
fn cases_match_their_expected_resolutions_and_diagnostics() {
    for case in case_dirs() {
        let name = case
            .file_name()
            .map_or_else(String::new, |it| it.to_string_lossy().into_owned());
        let input = case.join("main.nudo");
        let expected_resolutions = case.join("resolutions.txt");
        let expected_diagnostics = case.join("diagnostics.txt");

        let text = read(&input);
        let mut sources = SourceMap::new();
        let id = sources.add("main.nudo", text);
        let file = sources.get(id).expect("just added");
        let parsed = parse(file);
        assert!(
            !parsed.has_errors(),
            "case {name}: a resolution case must parse; the parser owns syntax errors"
        );
        assert!(parsed.tree().is_lossless(), "case {name}");

        let resolved = lower(parsed.tree(), id);
        assert_eq!(
            dump_resolutions(&resolved.hir, file),
            read(&expected_resolutions),
            "case {name}: resolutions differ from resolutions.txt"
        );

        let actual = normalized_diagnostics(file, &resolved);
        if expected_diagnostics.exists() {
            assert_eq!(
                actual,
                read(&expected_diagnostics),
                "case {name}: diagnostics differ from diagnostics.txt"
            );
        } else {
            assert!(
                actual == "# nudo-diagnostics v1\n",
                "case {name}: produced diagnostics but has no diagnostics.txt:\n{actual}"
            );
        }
    }
}

/// Renders diagnostics in the normalized, implementation-neutral form used by
/// the corpus: severity, code and 1-based start position.
fn normalized_diagnostics(
    file: &nudo_source::SourceFile,
    resolved: &nudo_hir::HirResult,
) -> String {
    let mut out = String::from("# nudo-diagnostics v1\n");
    for diagnostic in &resolved.diagnostics {
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
