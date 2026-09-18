//! The fixture corpus: `valid/` must lex cleanly, `invalid/` must produce
//! exactly the diagnostics it declares.
//!
//! The declaration format is documented in `fixtures/README.md`. This test is
//! deliberately strict in both directions: an undeclared diagnostic fails, and
//! a declared diagnostic that never appears also fails.
//!
//! Since M2 a fixture may also fail the *parser*, and a declared `NDO1001` is
//! not the lexer's business. This test therefore speaks only about the codes
//! the lexer owns; `compiler/nudo-parser/tests/fixtures.rs` checks the whole
//! declared set, so nothing declared can hide between the two.

use std::fs;
use std::path::{Path, PathBuf};

use nudo_lexer::tokenize;
use nudo_source::SourceMap;
use nudo_span::LineCol;

/// The `NDO` codes the lexer can emit. Anything else declared by a fixture is
/// another stage's business.
const LEXICAL_CODES: &[&str] = &["NDO1002", "NDO1003", "NDO1004", "NDO1005", "NDO1006"];

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

/// One `// EXPECT:` declaration.
#[derive(Debug, PartialEq, Eq)]
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
                let code = code.trim().to_string();
                let (line, column) = position
                    .trim()
                    .split_once(':')
                    .unwrap_or_else(|| panic!("`{rest}` is not `CODE @ line:column`"));
                let line: u32 = line.trim().parse().expect("line number");
                let column: u32 = column.trim().parse().expect("column number");
                (code, Some((line, column)))
            }
            None => (rest.to_string(), None),
        };
        found.push(Expectation { code, position });
    }
    found
}

fn lex_file(path: &Path) -> (SourceMap, nudo_lexer::Lexed) {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let mut sources = SourceMap::new();
    let id = sources.add(
        path.file_name().map_or_else(
            || path.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        ),
        text,
    );
    let file = sources.get(id).expect("just added");
    let lexed = tokenize(file);
    (sources, lexed)
}

#[test]
fn valid_fixtures_lex_without_diagnostics() {
    for path in fixture_files("valid") {
        let text = fs::read_to_string(&path).expect("fixture is readable");
        assert!(
            expectations(&text).is_empty(),
            "{} is in valid/ but declares expectations",
            path.display()
        );
        if path.extension().is_some_and(|ext| ext != "nudo") {
            // Inputs that deliberately use another extension are checked by the
            // CLI tests, not here.
            continue;
        }
        let (sources, lexed) = lex_file(&path);
        let diagnostics = lexed.diagnostics();
        if !diagnostics.is_empty() {
            let rendered = nudo_diagnostics::Renderer::new(&sources, false).render_all(diagnostics);
            panic!(
                "{} should lex cleanly, but reported:\n{rendered}",
                path.display()
            );
        }
        assert_eq!(
            lexed.tokens().last().map(|token| token.is_eof()),
            Some(true),
            "{} must end with an Eof token",
            path.display()
        );
    }
}

#[test]
fn invalid_fixtures_report_exactly_what_they_declare() {
    for path in fixture_files("invalid") {
        let text = fs::read_to_string(&path).expect("fixture is readable");
        let declared = expectations(&text);
        assert!(
            !declared.is_empty(),
            "{} is in invalid/ but declares no expectations",
            path.display()
        );
        let declared: Vec<Expectation> = declared
            .into_iter()
            .filter(|item| LEXICAL_CODES.contains(&item.code.as_str()))
            .collect();
        if declared.is_empty() {
            continue;
        }

        let (sources, lexed) = lex_file(&path);
        let diagnostics = lexed.diagnostics();

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
fn fixtures_use_the_documented_naming() {
    for kind in ["valid", "invalid"] {
        for path in fixture_files(kind) {
            let name = path
                .file_name()
                .expect("a file name")
                .to_string_lossy()
                .into_owned();
            assert!(
                name.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.'),
                "{name} must be kebab-case ASCII"
            );
        }
    }
}
