//! Reproducible micro-benchmarks for the NUDO compiler front end.
//!
//! The point of this binary is not optimisation; it is *measurement*. Before
//! changing the lexer we want a number that can be reproduced on any machine,
//! with no third-party harness and no nightly toolchain:
//!
//! ```text
//! cargo run --release --package nudo-bench
//! ```
//!
//! Each case builds a deterministic synthetic source, then measures
//! `SourceMap` construction (which includes the line index) and lexing.
//! Reported figures are the best and the mean of the timed iterations, plus
//! throughput in megabytes per second. Parsing, type checking and runtime
//! startup are **PLANNED**: they cannot be measured before they exist.

use std::hint::black_box;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use nudo_source::SourceMap;

/// Timed iterations per case unless overridden on the command line.
const DEFAULT_ITERATIONS: usize = 200;
/// Untimed iterations run before timing, to warm caches and the allocator.
const WARMUP_ITERATIONS: usize = 20;

fn main() -> ExitCode {
    let mut iterations = DEFAULT_ITERATIONS;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!(
                    "\
nudo-bench — reproducible micro-benchmarks for the NUDO front end

USAGE:
    nudo-bench [--iterations <N>]

OPTIONS:
    --iterations <N>    Timed iterations per case (default {DEFAULT_ITERATIONS})
    -h, --help          Print this help

Measured today: source registration (line index) and lexing.
Planned, not measurable yet: parsing, type checking, runtime startup."
                );
                return ExitCode::SUCCESS;
            }
            "--iterations" => {
                let Some(value) = args.next() else {
                    eprintln!("nudo-bench: `--iterations` needs a value");
                    return ExitCode::from(2);
                };
                match value.parse::<usize>() {
                    Ok(parsed) if parsed > 0 => iterations = parsed,
                    _ => {
                        eprintln!("nudo-bench: `{value}` is not a positive integer");
                        return ExitCode::from(2);
                    }
                }
            }
            other => {
                eprintln!("nudo-bench: unknown argument `{other}`");
                eprintln!("run `nudo-bench --help` for usage");
                return ExitCode::from(2);
            }
        }
    }

    println!("nudo-bench — lexer micro-benchmarks");
    println!("timed iterations: {iterations}, warm-up: {WARMUP_ITERATIONS}\n");
    println!(
        "{:<24} {:>9} {:>9} {:>11} {:>11} {:>12}",
        "case", "bytes", "tokens", "best", "mean", "throughput"
    );

    for case in cases() {
        let measurement = run_case(&case, iterations);
        println!(
            "{:<24} {:>9} {:>9} {:>11} {:>11} {:>12}",
            case.name,
            case.source.len(),
            measurement.tokens,
            format_duration(measurement.best),
            format_duration(measurement.mean),
            format_throughput(case.source.len(), measurement.best),
        );
    }

    println!(
        "\nMeasurements are indicative, not a promise: NUDO is pre-alpha and no\n\
         performance target has been accepted. See `benchmarks/README.md`."
    );
    ExitCode::SUCCESS
}

/// One benchmark case: a name and the source text to measure.
struct Case {
    name: &'static str,
    source: String,
}

/// What a case measured.
struct Measurement {
    best: Duration,
    mean: Duration,
    tokens: usize,
}

fn run_case(case: &Case, iterations: usize) -> Measurement {
    for _ in 0..WARMUP_ITERATIONS {
        black_box(lex_once(&case.source));
    }

    let mut best = Duration::MAX;
    let mut total = Duration::ZERO;
    let mut tokens = 0;
    for _ in 0..iterations {
        let start = Instant::now();
        let lexed = lex_once(&case.source);
        let elapsed = start.elapsed();
        tokens = lexed;
        total += elapsed;
        if elapsed < best {
            best = elapsed;
        }
    }
    let mean = total / u32::try_from(iterations).unwrap_or(u32::MAX);
    Measurement { best, mean, tokens }
}

/// Registers the source (building its line index) and lexes it.
fn lex_once(source: &str) -> usize {
    let mut sources = SourceMap::new();
    let id = sources.add("nudo-bench.nudo", source);
    let file = sources.get(id).expect("just added");
    let lexed = nudo_lexer::tokenize(file);
    lexed.tokens().len()
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "functions",
            source: synthetic_functions(2_000),
        },
        Case {
            name: "bindings",
            source: synthetic_bindings(5_000),
        },
        Case {
            name: "text-literals",
            source: synthetic_text(2_000),
        },
        Case {
            name: "comments",
            source: synthetic_comments(2_000),
        },
    ]
}

fn synthetic_functions(count: usize) -> String {
    let mut source = String::with_capacity(count * 48);
    for index in 0..count {
        source.push_str(&format!(
            "fn f{index}(a: Int, b: Int) -> Int {{\n    a + b * {index}\n}}\n\n"
        ));
    }
    source
}

fn synthetic_bindings(count: usize) -> String {
    let mut source = String::with_capacity(count * 24);
    for index in 0..count {
        source.push_str(&format!("let value_{index} = {index} + 1_000;\n"));
    }
    source
}

fn synthetic_text(count: usize) -> String {
    let mut source = String::with_capacity(count * 56);
    for index in 0..count {
        source.push_str(&format!(
            "let message_{index} = \"line {index}\\ttab\\nnewline\\\\slash\";\n"
        ));
    }
    source
}

fn synthetic_comments(count: usize) -> String {
    let mut source = String::with_capacity(count * 72);
    for index in 0..count {
        source.push_str(&format!(
            "// comment {index} explaining nothing in particular\nlet c{index} = {index};\n/* block {index} */\n"
        ));
    }
    source
}

fn format_duration(duration: Duration) -> String {
    let nanos = duration.as_nanos();
    if nanos < 1_000 {
        format!("{nanos}ns")
    } else if nanos < 1_000_000 {
        format!("{:.2}µs", nanos as f64 / 1_000.0)
    } else {
        format!("{:.2}ms", nanos as f64 / 1_000_000.0)
    }
}

fn format_throughput(bytes: usize, best: Duration) -> String {
    let seconds = best.as_secs_f64();
    if seconds <= 0.0 {
        return "n/a".to_string();
    }
    let megabytes = bytes as f64 / (1024.0 * 1024.0);
    format!("{:.1} MiB/s", megabytes / seconds)
}
