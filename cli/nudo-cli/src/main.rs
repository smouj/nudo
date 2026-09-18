//! The `nudo` command-line interface.
//!
//! # What this binary does today
//!
//! Exactly one command is implemented: `nudo check`, which reads `.nudo`
//! sources, lexes them, parses them and reports diagnostics. That is milestones
//! M1 and M2 of the roadmap. Every other command in the toolchain is listed by
//! `nudo --help` and exits with code [`EXIT_USAGE`] after saying so plainly.
//!
//! Nothing here pretends to work: a command that is not implemented never
//! exits successfully.
//!
//! # Exit codes
//!
//! | Code | Meaning                                              |
//! | ---- | ---------------------------------------------------- |
//! | 0    | success, no error diagnostics                        |
//! | 1    | at least one error diagnostic was reported           |
//! | 2    | usage error, unreadable input, or unimplemented      |

use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use nudo_common::{CHANNEL, TOOLCHAIN_NAME, VERSION, has_source_extension};
use nudo_diagnostics::{ColorChoice, Diagnostic, Diagnostics, Renderer, codes};
use nudo_source::SourceMap;

/// Success.
const EXIT_OK: u8 = 0;
/// Error diagnostics were reported.
const EXIT_DIAGNOSTICS: u8 = 1;
/// The invocation was wrong, an input could not be read, or the command is
/// not implemented yet.
const EXIT_USAGE: u8 = 2;

/// Commands the toolchain declares but does not implement in this pre-alpha.
const PLANNED_COMMANDS: &[(&str, &str)] = &[
    ("new", "Create a new NUDO package"),
    ("init", "Add a nudo.toml manifest to an existing directory"),
    ("run", "Compile and run a package"),
    ("build", "Compile a package"),
    ("test", "Run a package's tests"),
    ("fmt", "Format NUDO sources"),
    ("doc", "Generate documentation"),
    ("repl", "Start an interactive session"),
    ("trace", "Inspect the execution trace of a run"),
    ("eval", "Evaluate agent and task behaviour"),
    ("doctor", "Report toolchain and environment health"),
    ("audit", "Audit capabilities, policies and provenance"),
];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    ExitCode::from(run(&args))
}

fn run(args: &[String]) -> u8 {
    let Some(command) = args.first() else {
        eprintln!("{TOOLCHAIN_NAME}: no command given");
        eprintln!("run `{TOOLCHAIN_NAME} --help` for usage");
        return EXIT_USAGE;
    };
    match command.as_str() {
        "-h" | "--help" | "help" => {
            print_help();
            EXIT_OK
        }
        "-V" | "--version" => {
            println!("{TOOLCHAIN_NAME} {VERSION} ({CHANNEL})");
            EXIT_OK
        }
        "check" => cmd_check(&args[1..]),
        other => {
            if let Some((name, _)) = PLANNED_COMMANDS.iter().find(|(name, _)| *name == other) {
                eprintln!(
                    "{TOOLCHAIN_NAME}: `{name}` is planned and is not implemented in {VERSION} ({CHANNEL})"
                );
                eprintln!("run `{TOOLCHAIN_NAME} --help` to see the commands that work today");
            } else {
                eprintln!("{TOOLCHAIN_NAME}: unknown command `{other}`");
                eprintln!("run `{TOOLCHAIN_NAME} --help` for usage");
            }
            EXIT_USAGE
        }
    }
}

fn print_help() {
    let planned: String = PLANNED_COMMANDS
        .iter()
        .map(|(name, about)| format!("    {name:<10} {about}\n"))
        .collect();
    println!(
        "\
{TOOLCHAIN_NAME} {VERSION} ({CHANNEL}) — the NUDO language toolchain

USAGE:
    {TOOLCHAIN_NAME} <COMMAND> [OPTIONS] [FILE]...
    {TOOLCHAIN_NAME} --version
    {TOOLCHAIN_NAME} --help

IMPLEMENTED
    check      Read .nudo sources and report lexical, syntax and name
               diagnostics (milestones M1-M3: lexing, parsing and name
               resolution; nothing is executed)

PLANNED (declared, not implemented)
{planned}
OPTIONS
    -h, --help        Print this help
    -V, --version     Print version information
    --color <WHEN>    Colour diagnostics: auto (default), always, never
    --dump-tokens     With `check`: print the token stream instead of nothing
    --dump-tree       With `check`: print the syntax tree instead of nothing
    --dump-resolutions
                      With `check`: print what each name resolved to

EXIT CODES
    0    success, no error diagnostics
    1    at least one error diagnostic was reported
    2    usage error, unreadable input, or an unimplemented command

EXAMPLES
    {TOOLCHAIN_NAME} check examples/00-hello-world/main.nudo
    {TOOLCHAIN_NAME} check --dump-tokens examples/00-hello-world/main.nudo
    {TOOLCHAIN_NAME} check --dump-tree examples/00-hello-world/main.nudo

STATUS
    Pre-alpha. The language is not stable, the toolchain is incomplete and
    nothing here is fit for production use. See README.md and ROADMAP.md."
    );
}

fn print_check_help() {
    println!(
        "\
{TOOLCHAIN_NAME} check — read .nudo sources and report diagnostics

USAGE:
    {TOOLCHAIN_NAME} check [OPTIONS] <FILE>...

OPTIONS:
    --dump-tokens     Print the token stream in the stable `nudo-tokens v1` format
    --dump-tree       Print the syntax tree in the stable `nudo-tree v1` format
    --dump-resolutions
                      Print what each name resolved to, `nudo-hir v1` format
    --color <WHEN>    Colour diagnostics: auto (default), always, never
    -h, --help        Print this help

SCOPE:
    Milestones M1-M3. `check` reads each file, lexes it, parses it, resolves
    its names and reports lexical, syntax and resolution diagnostics. Type
    checking, effect checking and execution are planned and are not
    implemented, so a clean run means \"no lexical, syntax or name
    diagnostics\", not \"this program is correct\"."
    );
}

fn cmd_check(args: &[String]) -> u8 {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut color = ColorChoice::Auto;
    let mut dump_tokens = false;
    let mut dump_tree = false;
    let mut dump_resolutions = false;

    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        match arg {
            "-h" | "--help" => {
                print_check_help();
                return EXIT_OK;
            }
            "--dump-tokens" => dump_tokens = true,
            "--dump-tree" => dump_tree = true,
            "--dump-resolutions" => dump_resolutions = true,
            "--color" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    eprintln!("{TOOLCHAIN_NAME}: `--color` needs a value: auto, always or never");
                    return EXIT_USAGE;
                };
                let Some(choice) = ColorChoice::parse(value) else {
                    eprintln!(
                        "{TOOLCHAIN_NAME}: `{value}` is not a colour choice; use auto, always or never"
                    );
                    return EXIT_USAGE;
                };
                color = choice;
            }
            _ if arg.starts_with("--color=") => {
                let value = &arg["--color=".len()..];
                let Some(choice) = ColorChoice::parse(value) else {
                    eprintln!(
                        "{TOOLCHAIN_NAME}: `{value}` is not a colour choice; use auto, always or never"
                    );
                    return EXIT_USAGE;
                };
                color = choice;
            }
            _ if arg.starts_with('-') && arg.len() > 1 => {
                eprintln!("{TOOLCHAIN_NAME}: unknown option `{arg}` for `check`");
                eprintln!("run `{TOOLCHAIN_NAME} check --help` for usage");
                return EXIT_USAGE;
            }
            _ => files.push(PathBuf::from(arg)),
        }
        index += 1;
    }

    if files.is_empty() {
        eprintln!("{TOOLCHAIN_NAME}: `check` needs at least one .nudo file");
        eprintln!("run `{TOOLCHAIN_NAME} check --help` for usage");
        return EXIT_USAGE;
    }

    let use_color = color.resolve(std::io::stderr().is_terminal());

    // Load everything first: a diagnostic renderer borrows the source map.
    let mut sources = SourceMap::new();
    let mut loaded: Vec<(PathBuf, nudo_source::SourceId)> = Vec::new();
    let mut unreadable = 0usize;
    for path in &files {
        match sources.add_file(path) {
            Ok(id) => loaded.push((path.clone(), id)),
            Err(error) => {
                eprintln!(
                    "{TOOLCHAIN_NAME}: cannot read `{}`: {error}",
                    path.display()
                );
                unreadable += 1;
            }
        }
    }

    let renderer = Renderer::new(&sources, use_color);
    let mut diagnostics = Diagnostics::new();
    let mut dump = String::new();

    for (path, id) in &loaded {
        let Some(file) = sources.get(*id) else {
            continue;
        };
        if !has_source_extension(&path.display().to_string()) {
            diagnostics.push(
                Diagnostic::warning(
                    codes::UNEXPECTED_FILE_EXTENSION,
                    format!(
                        "`{}` does not use the .{} extension",
                        path.display(),
                        nudo_common::SOURCE_EXTENSION
                    ),
                )
                .with_location(*id, file.full_span())
                .with_help("NUDO sources are named `*.nudo`"),
            );
        }
        let lexed = nudo_lexer::tokenize(file);
        if dump_tokens {
            // The token dump is the lexer's, so that its format stays exactly
            // what the conformance corpus pins: no trivia, indices over the
            // token stream, `nudo-tokens v1`.
            dump.push_str(&nudo_lexer::dump_tokens(file, lexed.tokens()));
        }
        let parsed = nudo_parser::parse(file);
        diagnostics.extend_from(parsed.diagnostics().clone());
        if dump_tree {
            dump.push_str(&nudo_syntax::dump_tree(parsed.tree()));
        }
        // Resolution runs only on a file the parser accepted: a file that is not
        // a program has no names to resolve, and resolving one would bury the
        // parser's diagnostics under a cascade about a tree nobody agreed to.
        if !parsed.has_errors() {
            let resolved = nudo_hir::lower(parsed.tree(), *id);
            diagnostics.extend_from(resolved.diagnostics.clone());
            if dump_resolutions {
                dump.push_str(&nudo_hir::dump_resolutions(&resolved.hir, file));
            }
        }
    }

    if (dump_tokens || dump_tree || dump_resolutions) && !dump.is_empty() {
        print!("{dump}");
        let _ = std::io::stdout().flush();
    }

    let errors = diagnostics.error_count();
    let warnings = diagnostics.warning_count();
    if !diagnostics.is_empty() {
        eprintln!("{}", renderer.render_all(&diagnostics));
    }

    if errors > 0 {
        eprintln!(
            "\n{TOOLCHAIN_NAME}: {} and {}",
            count(errors, "error"),
            count(warnings, "warning")
        );
        return EXIT_DIAGNOSTICS;
    }

    if unreadable > 0 {
        eprintln!(
            "{TOOLCHAIN_NAME}: could not read {}",
            count(unreadable, "file")
        );
        return EXIT_USAGE;
    }

    println!(
        "checked {}: {} and {}",
        count(loaded.len(), "file"),
        count(errors, "error"),
        count(warnings, "warning")
    );
    EXIT_OK
}

/// Formats a count with an English plural, for example `1 file`, `2 files`.
fn count(number: usize, noun: &str) -> String {
    if number == 1 {
        format!("{number} {noun}")
    } else {
        format!("{number} {noun}s")
    }
}

/// Reports whether `path` looks like a NUDO source file.
///
/// Unused today, but kept next to [`has_source_extension`] usage so that the
/// dependency on `nudo-common` stays explicit in one place.
#[allow(dead_code)]
fn is_nudo_source(path: &Path) -> bool {
    has_source_extension(&path.display().to_string())
}
