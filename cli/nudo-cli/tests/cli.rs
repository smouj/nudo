//! End-to-end tests for the `nudo` binary.
//!
//! These run the real executable through `CARGO_BIN_EXE_nudo`, so they cover
//! argument handling, exit codes and stream separation exactly as a user or a
//! CI job sees them. Inputs are the checked-in fixtures, so the tests never
//! write to disk.

use std::path::PathBuf;
use std::process::{Command, Output};

fn repo_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn nudo(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nudo"))
        .args(args)
        .output()
        .expect("the nudo binary runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn version_prints_name_and_channel() {
    let output = nudo(&["--version"]);
    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.starts_with("nudo 0.0.1"), "got {text:?}");
    assert!(text.contains("pre-alpha"));
}

#[test]
fn help_lists_the_implemented_command_and_the_planned_ones() {
    let output = nudo(&["--help"]);
    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("check"));
    assert!(text.contains("PLANNED (declared, not implemented)"));
    assert!(text.contains("nudo <COMMAND> [OPTIONS] [FILE]..."));
}

#[test]
fn no_arguments_is_a_usage_error() {
    let output = nudo(&[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("no command given"));
}

#[test]
fn unknown_command_is_a_usage_error() {
    let output = nudo(&["frobnicate"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("unknown command `frobnicate`"));
}

#[test]
fn planned_commands_never_succeed() {
    for command in [
        "new", "init", "run", "build", "test", "fmt", "doc", "repl", "trace", "eval", "doctor",
        "audit",
    ] {
        let output = nudo(&[command]);
        assert_eq!(output.status.code(), Some(2), "{command} must not succeed");
        assert!(
            stderr(&output).contains("is planned and is not implemented"),
            "{command} must say it is not implemented"
        );
    }
}

#[test]
fn check_accepts_a_valid_program() {
    let path = repo_path("fixtures/valid/hello.nudo");
    let output = nudo(&["check", path.to_str().expect("utf-8 path")]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(stdout(&output).contains("0 errors"));
    assert!(stderr(&output).is_empty());
}

#[test]
fn check_reports_errors_for_invalid_programs() {
    let path = repo_path("fixtures/invalid/unterminated-text.nudo");
    let output = nudo(&["check", path.to_str().expect("utf-8 path")]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("NDO1003"));
    assert!(stderr(&output).contains("1 error"));
}

#[test]
fn check_dumps_tokens_on_stdout() {
    let path = repo_path("fixtures/valid/hello.nudo");
    let output = nudo(&["check", "--dump-tokens", path.to_str().expect("utf-8 path")]);
    assert_eq!(output.status.code(), Some(0));
    let text = stdout(&output);
    assert!(text.starts_with("# nudo-tokens v1\n"));
    // The summary line still goes to stdout after the dump.
    assert!(text.contains("checked 1 file"));
}

#[test]
fn check_dumps_the_syntax_tree_on_stdout() {
    let path = repo_path("fixtures/valid/hello.nudo");
    let output = nudo(&["check", "--dump-tree", path.to_str().expect("utf-8 path")]);
    assert_eq!(output.status.code(), Some(0));
    let text = stdout(&output);
    assert!(text.starts_with("# nudo-tree v1\n"));
    assert!(text.contains("SourceFile"));
    assert!(text.contains("FunctionDecl"));
    assert!(text.contains("checked 1 file"));
}

#[test]
fn check_reports_syntax_errors_that_lex_cleanly() {
    // A file can use only known tokens and still not be a program. Before M2
    // this exited 0, which is exactly the false confidence the roadmap set out
    // to remove.
    let path = repo_path("fixtures/invalid/syntax.nudo");
    let output = nudo(&["check", path.to_str().expect("utf-8 path")]);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert!(stderr(&output).contains("NDO1001"));
}

#[test]
fn check_reports_every_preview_example_as_not_accepted() {
    for name in [
        "05-agent",
        "06-tools",
        "07-generated-verified",
        "08-policy",
        "09-multi-agent",
    ] {
        let path = repo_path(&format!("examples/{name}/main.nudo"));
        let output = nudo(&["check", path.to_str().expect("utf-8 path")]);
        assert_eq!(
            output.status.code(),
            Some(1),
            "examples/{name} is a preview and must not be accepted"
        );
    }
}

#[test]
fn check_accepts_the_examples_the_readme_calls_readable() {
    for name in [
        "00-hello-world",
        "01-variables",
        "02-functions",
        "03-types",
        "04-results",
    ] {
        let path = repo_path(&format!("examples/{name}/main.nudo"));
        let output = nudo(&["check", path.to_str().expect("utf-8 path")]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "examples/{name} is documented as readable:\n{}",
            stderr(&output)
        );
    }
}

#[test]
fn check_warns_about_the_wrong_extension() {
    let path = repo_path("fixtures/valid/not-a-nudo-file.txt");
    let output = nudo(&["check", path.to_str().expect("utf-8 path")]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(stderr(&output).contains("NDO8001"));
}

#[test]
fn check_accepts_an_explicit_colour_choice() {
    let path = repo_path("fixtures/invalid/unterminated-text.nudo");
    let output = nudo(&[
        "check",
        "--color",
        "always",
        path.to_str().expect("utf-8 path"),
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("\u{1b}["));
}

#[test]
fn check_rejects_an_unknown_colour_choice() {
    let output = nudo(&["check", "--color", "rainbow", "x.nudo"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("not a colour choice"));
}

#[test]
fn check_needs_a_file() {
    let output = nudo(&["check"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("needs at least one"));
}

#[test]
fn check_reports_unreadable_files() {
    let output = nudo(&["check", "this-file-does-not-exist.nudo"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("cannot read"));
}

#[test]
fn check_help_documents_the_scope() {
    let output = nudo(&["check", "--help"]);
    assert!(output.status.success());
    let text = stdout(&output);
    assert!(text.contains("Milestones M1-M3"));
    assert!(text.contains("--dump-tree"));
}

#[test]
fn check_dumps_resolutions_on_stdout() {
    let path = repo_path("fixtures/valid/hello.nudo");
    let output = nudo(&[
        "check",
        "--dump-resolutions",
        path.to_str().expect("utf-8 path"),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.starts_with("# nudo-hir v1\n"), "{text}");
    assert!(text.contains("function `main`"), "{text}");
}

#[test]
fn check_reports_an_unresolved_name() {
    let path = repo_path("fixtures/invalid/unresolved-name.nudo");
    let output = nudo(&["check", path.to_str().expect("utf-8 path")]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("NDO2001"), "{}", stderr(&output));
    assert!(stderr(&output).contains("missing_name"));
}
