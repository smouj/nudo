//! The fixture corpus, from the parser's side.
//!
//! `fixtures/README.md` defines the contract: `valid/` must produce no
//! diagnostics at all, `invalid/` must produce exactly the diagnostics it
//! declares. This test checks the whole set — lexical and syntactic — and the
//! tree, which is what M2 added.
//!
//! It also checks the property the whole pipeline rests on: **the tree is
//! lossless**, for every fixture, including the ones that are wrong. A parser
//! that drops bytes on malformed input cannot be trusted to reprint a file it
//! half understood.

use std::fs;
use std::path::{Path, PathBuf};

use nudo_parser::{MAX_ERRORS, parse};
use nudo_source::SourceMap;
use nudo_span::LineCol;

fn fixture_dir(kind: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(kind)
}

fn fixture_files(kind: &str) -> Vec<PathBuf> {
    let dir = fixture_dir(kind);
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", dir.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "no fixtures in {}; the corpus is not allowed to be empty",
        dir.display()
    );
    files
}

fn is_source(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "nudo")
}

/// One `// EXPECT:` declaration. The format is documented in
/// `fixtures/README.md`.
#[derive(Debug)]
struct Expectation {
    code: String,
    position: Option<(u32, u32)>,
}

fn expectations(text: &str) -> Vec<Expectation> {
    let mut found = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("// EXPECT:") else {
            continue;
        };
        let rest = rest.trim();
        let (code, position) = match rest.split_once('@') {
            Some((code, position)) => {
                let (line, column) = position
                    .trim()
                    .split_once(':')
                    .unwrap_or_else(|| panic!("`{rest}` is not `CODE @ line:column`"));
                (
                    code.trim().to_string(),
                    Some((
                        line.trim().parse().expect("line number"),
                        column.trim().parse().expect("column number"),
                    )),
                )
            }
            None => (rest.to_string(), None),
        };
        found.push(Expectation { code, position });
    }
    found
}

fn parse_file(path: &Path) -> (SourceMap, nudo_parser::Parse) {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    );
    let mut sources = SourceMap::new();
    let id = sources.add(name, text);
    let file = sources.get(id).expect("just added");
    let parsed = parse(file);
    (sources, parsed)
}

#[test]
fn valid_fixtures_parse_without_diagnostics() {
    for path in fixture_files("valid") {
        if !is_source(&path) {
            continue;
        }
        let text = fs::read_to_string(&path).expect("fixture is readable");
        assert!(
            expectations(&text).is_empty(),
            "{} is in valid/ but declares expectations",
            path.display()
        );
        let (sources, parsed) = parse_file(&path);
        if parsed.has_errors() {
            let rendered =
                nudo_diagnostics::Renderer::new(&sources, false).render_all(parsed.diagnostics());
            panic!(
                "{} should lex and parse cleanly, but reported:\n{rendered}",
                path.display()
            );
        }
        assert!(
            !parsed.tree().root().has_errors(),
            "{} parsed cleanly but the tree contains an error node",
            path.display()
        );
    }
}

#[test]
fn invalid_fixtures_produce_exactly_what_they_declare() {
    for path in fixture_files("invalid") {
        let text = fs::read_to_string(&path).expect("fixture is readable");
        let declared = expectations(&text);
        assert!(
            !declared.is_empty(),
            "{} is in invalid/ but declares no expectations",
            path.display()
        );

        let (sources, parsed) = parse_file(&path);
        let diagnostics = parsed.diagnostics();

        let mut expected_codes: Vec<String> =
            declared.iter().map(|item| item.code.clone()).collect();
        let mut actual_codes: Vec<String> = diagnostics
            .iter()
            .map(|item| item.code().id().to_string())
            .collect();
        expected_codes.sort();
        actual_codes.sort();
        let rendered = nudo_diagnostics::Renderer::new(&sources, false).render_all(diagnostics);
        assert_eq!(
            actual_codes,
            expected_codes,
            "{} declares different diagnostics than it produces:\n{rendered}",
            path.display()
        );

        for expectation in &declared {
            let Some((line, column)) = expectation.position else {
                continue;
            };
            let file = sources
                .get(diagnostics.as_slice()[0].source().expect("a source id"))
                .expect("the file is in the map");
            let matched = diagnostics.iter().any(|diagnostic| {
                diagnostic.code().id() == expectation.code
                    && diagnostic.span().is_some_and(|span| {
                        file.line_col(span.start()) == LineCol::new(line, column)
                    })
            });
            assert!(
                matched,
                "{} declares {} at {line}:{column}, which was not reported:\n{rendered}",
                path.display(),
                expectation.code
            );
        }
    }
}

#[test]
fn every_fixture_produces_a_lossless_tree() {
    for kind in ["valid", "invalid"] {
        for path in fixture_files(kind) {
            if !is_source(&path) {
                continue;
            }
            let (_, parsed) = parse_file(&path);
            let tree = parsed.tree();
            assert!(
                tree.is_lossless(),
                "{}: the tree does not reprint the file",
                path.display()
            );
            assert_eq!(tree.validate(), Ok(()), "{}", path.display());
            assert!(tree.token_count() > 0, "{}", path.display());
        }
    }
}

#[test]
fn no_fixture_produces_more_than_the_diagnostic_cap() {
    for kind in ["valid", "invalid"] {
        for path in fixture_files(kind) {
            if !is_source(&path) {
                continue;
            }
            let (_, parsed) = parse_file(&path);
            assert!(
                parsed.diagnostics().len() <= MAX_ERRORS + 16,
                "{} produced {} diagnostics",
                path.display(),
                parsed.diagnostics().len()
            );
        }
    }
}
