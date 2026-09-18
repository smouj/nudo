//! The `check` report, in the three modes.
//!
//! The command calls these in order, with the diagnostics rendered between them,
//! so that the existing diagnostic presentation is used rather than duplicated:
//!
//! ```text
//! header()        NUDO CHECK
//! file_header()   the file, and in interactive mode its pipeline
//! stages_block()  what ran, and how it went
//! <diagnostics>   the existing renderer, untouched
//! summary()       the counts, and the contract line scripts already parse
//! ```
//!
//! The contract line — `checked 1 file: 0 errors and 0 warnings` — is passed in
//! by the caller and printed in **every** mode. Structure is added around it, so
//! that scripts, documentation and the console checks keep working while the
//! presentation around them changes.

use super::Capabilities;
use super::{Mode, Role, Section, Stage, render_rule, render_stages};

/// What one file's check produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileReport {
    /// The path as the user wrote it.
    pub(crate) path: String,
    /// The stages that actually ran, in order.
    pub(crate) stages: Vec<Stage>,
}

impl FileReport {
    /// A report for one file.
    #[must_use]
    pub(crate) fn new(path: impl Into<String>, stages: Vec<Stage>) -> Self {
        FileReport {
            path: path.into(),
            stages,
        }
    }
}

/// The report's title, in the modes that have one.
#[must_use]
pub(crate) fn header(capabilities: &Capabilities) -> String {
    match capabilities.mode {
        Mode::Interactive | Mode::Ci => {
            let theme = capabilities.theme();
            format!(
                "{}\n\n",
                theme.paint(Role::Plain, "NUDO CHECK").to_uppercase()
            )
        }
        Mode::Plain => String::new(),
    }
}

/// The file this report is about.
#[must_use]
pub(crate) fn file_header(capabilities: &Capabilities, path: &str) -> String {
    let theme = capabilities.theme();
    match capabilities.mode {
        Mode::Interactive => format!(
            "{}\n{}\n\n",
            theme.paint(Role::Plain, path),
            render_rule(capabilities.width, theme)
        ),
        Mode::Plain => format!("checking {path}\n"),
        Mode::Ci => String::new(),
    }
}

/// What ran, and how it went.
///
/// The stages are whatever the caller actually did. A file the parser rejected
/// has no `hir` stage, because no resolution happened — the pipeline is a record
/// of work, not a checklist of features.
#[must_use]
pub(crate) fn stages_block(capabilities: &Capabilities, stages: &[Stage]) -> String {
    let theme = capabilities.theme();
    let symbols = capabilities.symbols();
    match capabilities.mode {
        Mode::Interactive => {
            let mut out = render_stages(stages, capabilities.width, symbols, theme);
            out.push('\n');
            out
        }
        Mode::Plain => {
            let mut out = String::new();
            for stage in stages {
                out.push_str(&format!("{}: {}\n", stage.label, stage.status.word()));
            }
            out
        }
        Mode::Ci => {
            let total = stages.len();
            let widest = stages
                .iter()
                .map(|stage| stage.label.chars().count())
                .max()
                .unwrap_or(0);
            let mut out = String::new();
            for (index, stage) in stages.iter().enumerate() {
                let padding = " ".repeat(widest.saturating_sub(stage.label.chars().count()));
                out.push_str(&format!(
                    "[{}/{total}] {}{padding} {}\n",
                    index + 1,
                    stage.label,
                    stage.status.word(),
                ));
            }
            out.push('\n');
            out
        }
    }
}

/// The counts, and the line scripts parse.
#[must_use]
pub(crate) fn summary(
    capabilities: &Capabilities,
    files: usize,
    errors: usize,
    warnings: usize,
    contract: Option<&str>,
) -> String {
    let theme = capabilities.theme();
    match capabilities.mode {
        Mode::Interactive => {
            let section = Section::new(
                "SUMMARY",
                vec![
                    ("files".to_string(), files.to_string()),
                    ("errors".to_string(), errors.to_string()),
                    ("warnings".to_string(), warnings.to_string()),
                ],
            );
            let mut out = super::render_sections(&[section], capabilities.width, theme);
            out.push_str(&render_rule(capabilities.width, theme));
            if let Some(contract) = contract {
                out.push('\n');
                out.push_str(&theme.paint(Role::Plain, contract));
            }
            out.push('\n');
            out
        }
        // A failure prints no contract line, exactly as before: a script that
        // greps for "checked" must not find one for a run that did not check.
        Mode::Plain => contract.map_or_else(String::new, |contract| format!("{contract}\n")),
        Mode::Ci => {
            let mut out = String::new();
            out.push_str(&format!("files       {files}\n"));
            out.push_str(&format!("errors      {errors}\n"));
            out.push_str(&format!("warnings    {warnings}\n\n"));
            if let Some(contract) = contract {
                out.push_str(&format!("{contract}\n\n"));
            }
            let verdict = if errors == 0 {
                theme.paint(Role::Ok, "PASS")
            } else {
                theme.paint(Role::Err, "FAIL")
            };
            out.push_str(&format!("{verdict}\n"));
            out
        }
    }
}

/// The whole report, given already-rendered diagnostics.
///
/// This is what the snapshot tests compare: a single string, in one mode, with no
/// stream and no timing in it. `nudo check` composes the same pieces itself,
/// because its diagnostics go to another stream; a command that keeps everything
/// on one stream calls this.
#[must_use]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn render(
    capabilities: &Capabilities,
    files: &[FileReport],
    diagnostics: &str,
    errors: usize,
    warnings: usize,
    contract: &str,
) -> String {
    let mut out = header(capabilities);
    for file in files {
        out.push_str(&file_header(capabilities, &file.path));
        out.push_str(&stages_block(capabilities, &file.stages));
    }
    if !diagnostics.is_empty() {
        out.push_str(diagnostics);
        if !diagnostics.ends_with('\n') {
            out.push('\n');
        }
    }
    out.push_str(&summary(
        capabilities,
        files.len(),
        errors,
        warnings,
        Some(contract),
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::{Environment, Status};

    fn capabilities(environment: Environment) -> Capabilities {
        Capabilities::from(&environment)
    }

    fn pipeline() -> Vec<Stage> {
        vec![
            Stage::new("SOURCE", "source", Status::Success),
            Stage::new("LEX", "lex", Status::Success),
            Stage::new("PARSE", "parse", Status::Success),
            Stage::new("HIR", "hir", Status::Success),
        ]
    }

    fn interactive() -> Capabilities {
        capabilities(Environment {
            is_tty: true,
            unicode: true,
            width: Some(60),
            // Snapshots are taken without colour: the escapes are the same
            // bytes in every test, and a reader of the source is reading the
            // layout, not the sequence numbers.
            color_choice: crate::ColorChoice::Never,
            ..Environment::default()
        })
    }

    fn plain() -> Capabilities {
        capabilities(Environment {
            unicode: true,
            width: Some(60),
            ..Environment::default()
        })
    }

    fn ci() -> Capabilities {
        capabilities(Environment {
            is_tty: true,
            ci: true,
            unicode: true,
            width: Some(60),
            color_choice: crate::ColorChoice::Never,
            ..Environment::default()
        })
    }

    const CONTRACT: &str = "checked 1 file: 0 errors and 0 warnings";

    #[test]
    fn a_clean_check_reads_well_in_a_terminal() {
        let files = [FileReport::new("src/main.nudo", pipeline())];
        let rendered = render(&interactive(), &files, "", 0, 0, CONTRACT);
        assert_eq!(
            rendered,
            "NUDO CHECK\n\n\
             src/main.nudo\n\
             ────────────────────────────────────────────────────────────\n\n\
             ✓ SOURCE ── ✓ LEX ── ✓ PARSE ── ✓ HIR\n\n\
             SUMMARY\n  files     1\n  errors    0\n  warnings  0\n\
             ────────────────────────────────────────────────────────────\n\
             checked 1 file: 0 errors and 0 warnings\n",
            "{rendered}"
        );
    }

    #[test]
    fn a_pipe_gets_one_line_per_event_and_no_decoration() {
        let files = [FileReport::new("src/main.nudo", pipeline())];
        let rendered = render(&plain(), &files, "", 0, 0, CONTRACT);
        assert_eq!(
            rendered,
            "checking src/main.nudo\n\
             source: ok\nlex: ok\nparse: ok\nhir: ok\n\
             checked 1 file: 0 errors and 0 warnings\n",
            "{rendered}"
        );
        assert!(!rendered.contains('\x1b'));
    }

    #[test]
    fn ci_gets_a_deterministic_block_that_ends_in_a_verdict() {
        let files = [FileReport::new("src/main.nudo", pipeline())];
        let rendered = render(&ci(), &files, "", 0, 0, CONTRACT);
        assert_eq!(
            rendered,
            "NUDO CHECK\n\n\
             [1/4] source ok\n[2/4] lex    ok\n[3/4] parse  ok\n[4/4] hir    ok\n\n\
             files       1\nerrors      0\nwarnings    0\n\n\
             checked 1 file: 0 errors and 0 warnings\n\nPASS\n",
            "{rendered}"
        );
    }

    #[test]
    fn a_failed_check_says_fail_in_ci() {
        let files = [FileReport::new("src/main.nudo", pipeline())];
        let rendered = render(
            &ci(),
            &files,
            "error[NDO2001]: unresolved name\n",
            1,
            0,
            "checked 1 file: 1 error and 0 warnings",
        );
        assert!(rendered.ends_with("FAIL\n"), "{rendered}");
        assert!(rendered.contains("error[NDO2001]"), "{rendered}");
    }

    #[test]
    fn a_stage_that_did_not_run_is_not_shown() {
        // A file the parser rejected: no resolution happened, so there is no
        // `hir` row to show. The pipeline records work, it does not advertise.
        let stages = vec![
            Stage::new("SOURCE", "source", Status::Success),
            Stage::new("LEX", "lex", Status::Success),
            Stage::new("PARSE", "parse", Status::Error),
        ];
        let rendered = stages_block(&interactive(), &stages);
        assert!(!rendered.contains("HIR"), "{rendered}");
        assert!(rendered.contains("PARSE"), "{rendered}");
    }

    #[test]
    fn a_narrow_terminal_stacks_the_pipeline() {
        // The width is clamped to the 40 columns the specification promises, and
        // the details are what a real run puts on the line — so this is what a
        // narrow terminal would actually see.
        let narrow = capabilities(Environment {
            is_tty: true,
            unicode: true,
            width: Some(30),
            color_choice: crate::ColorChoice::Never,
            ..Environment::default()
        });
        assert_eq!(narrow.width, crate::terminal::MINIMUM_WIDTH);
        let stages = vec![
            Stage::new("SOURCE", "source", Status::Success),
            Stage::new("LEX", "lex", Status::Success).with_detail("12 tokens"),
            Stage::new("PARSE", "parse", Status::Success),
            Stage::new("HIR", "hir", Status::Success).with_detail("4 definitions"),
        ];
        let rendered = stages_block(&narrow, &stages);
        assert_eq!(
            rendered, "  ✓ source\n  ✓ lex    12 tokens\n  ✓ parse\n  ✓ hir    4 definitions\n\n",
            "{rendered}"
        );
    }

    #[test]
    fn every_mode_keeps_the_contract_line() {
        let files = [FileReport::new("src/main.nudo", pipeline())];
        for capabilities in [interactive(), plain(), ci()] {
            let rendered = render(&capabilities, &files, "", 0, 0, CONTRACT);
            assert!(
                rendered.contains(CONTRACT),
                "{:?} lost the contract line:\n{rendered}",
                capabilities.mode
            );
        }
    }

    #[test]
    fn a_colourful_terminal_gets_painted_states() {
        let colorful = capabilities(Environment {
            is_tty: true,
            unicode: true,
            color_choice: crate::ColorChoice::Always,
            ..Environment::default()
        });
        let rendered = stages_block(&colorful, &pipeline());
        assert!(rendered.contains("\x1b[32m"), "{rendered}");
        assert!(rendered.contains("\x1b[0m"), "{rendered}");
        // The word is still there, so the state survives with no colour at all.
        assert!(rendered.contains("SOURCE"), "{rendered}");
    }

    #[test]
    fn no_mode_emits_escape_bytes_without_a_terminal_that_can_show_them() {
        let files = [FileReport::new("src/main.nudo", pipeline())];
        for capabilities in [plain(), ci()] {
            let rendered = render(&capabilities, &files, "", 0, 0, CONTRACT);
            assert!(!rendered.contains('\x1b'), "{rendered}");
        }
    }

    #[test]
    fn ascii_symbols_are_used_when_the_locale_cannot_show_unicode() {
        let ascii = capabilities(Environment {
            is_tty: true,
            width: Some(60),
            color_choice: crate::ColorChoice::Never,
            ..Environment::default()
        });
        let rendered = stages_block(&ascii, &pipeline());
        assert_eq!(
            rendered, "+ SOURCE -- + LEX -- + PARSE -- + HIR\n\n",
            "{rendered}"
        );
    }
}
