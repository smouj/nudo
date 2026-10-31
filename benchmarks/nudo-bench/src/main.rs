//! Reproducible measurements of the NUDO compiler front end.
//!
//! The point of this binary is not optimisation; it is *measurement*. A number
//! is only useful if another person can reproduce it, so every result carries
//! the commit, the toolchain and the machine that produced it, and the input is
//! generated deterministically before the timed region starts.
//!
//! ```text
//! cargo run --release --package nudo-bench
//! cargo run --release --package nudo-bench -- --format json --output results.json
//! cargo run --release --package nudo-bench -- --compare baseline.json
//! ```
//!
//! # What is measured
//!
//! | Stage | Timed region | Isolated? |
//! | ----- | ------------ | --------- |
//! | `lex` | `tokenize` on a registered source | yes |
//! | `parse` | `tokenize` + `parse` | no: parsing needs a token stream |
//! | `hir` | `lower` on a pre-built syntax tree | yes |
//! | `typeck` | `check` on a pre-built HIR | yes |
//!
//! `parse` is cumulative on purpose and says so. Splitting a lexer from its
//! parser would mean timing a private entry point with a hand-built token
//! vector, which measures the harness rather than the parser.
//!
//! Nothing is claimed about a stage the toolchain does not have. A benchmark for
//! the interpreter is not "pending": it cannot exist before `nudo run` does.

use std::collections::BTreeMap;
use std::hint::black_box;
use std::process::ExitCode;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use nudo_hir::lower;
use nudo_parser::parse;
use nudo_source::SourceMap;
use nudo_typeck::check;

/// Timed iterations per case unless overridden on the command line.
const DEFAULT_ITERATIONS: usize = 200;
/// Untimed iterations run before timing, to warm caches and the allocator.
const DEFAULT_WARMUP: usize = 20;
/// A median slower than the baseline by this percentage is a regression.
const DEFAULT_THRESHOLD_PERCENT: f64 = 10.0;

fn main() -> ExitCode {
    let options = match Options::parse(std::env::args().skip(1)) {
        Ok(Some(options)) => options,
        Ok(None) => return ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nudo-bench: {error}");
            eprintln!("run `nudo-bench --help` for usage");
            return ExitCode::from(2);
        }
    };

    let cases = build_cases();
    let mut results = Vec::with_capacity(cases.len());
    for case in &cases {
        results.push(run_case(case, options.iterations, options.warmup));
    }

    let report = Report {
        metadata: Metadata::collect(options.iterations, options.warmup, options.label.clone()),
        cases: results,
    };

    match options.format {
        Format::Text => print_text(&report),
        Format::Json => {
            let json = report.to_json();
            match &options.output {
                Some(path) => {
                    if let Err(error) = std::fs::write(path, json.as_bytes()) {
                        eprintln!("nudo-bench: cannot write `{path}`: {error}");
                        return ExitCode::from(1);
                    }
                    eprintln!("wrote {path}");
                }
                None => println!("{json}"),
            }
        }
    }

    if let Some(path) = &options.compare {
        let baseline = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                eprintln!("nudo-bench: cannot read baseline `{path}`: {error}");
                return ExitCode::from(1);
            }
        };
        let baseline = match Baseline::parse(&baseline) {
            Ok(baseline) => baseline,
            Err(error) => {
                eprintln!("nudo-bench: cannot parse baseline `{path}`: {error}");
                return ExitCode::from(1);
            }
        };
        let comparison = compare(&report, &baseline, options.threshold_percent);
        println!("{}", comparison.to_markdown());
    }

    ExitCode::SUCCESS
}

// ---------------------------------------------------------------- options ---

#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    Text,
    Json,
}

struct Options {
    iterations: usize,
    warmup: usize,
    format: Format,
    output: Option<String>,
    compare: Option<String>,
    threshold_percent: f64,
    label: Option<String>,
}

impl Options {
    fn parse(args: impl Iterator<Item = String>) -> Result<Option<Self>, String> {
        let mut options = Self {
            iterations: DEFAULT_ITERATIONS,
            warmup: DEFAULT_WARMUP,
            format: Format::Text,
            output: None,
            compare: None,
            threshold_percent: DEFAULT_THRESHOLD_PERCENT,
            label: None,
        };
        let mut args = args.peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    return Ok(None);
                }
                "--iterations" => {
                    options.iterations = positive(&mut args, "--iterations")?;
                }
                "--warmup" => {
                    options.warmup = positive(&mut args, "--warmup")?;
                }
                "--format" => {
                    let value = value(&mut args, "--format")?;
                    options.format = match value.as_str() {
                        "text" => Format::Text,
                        "json" => Format::Json,
                        other => return Err(format!("`{other}` is not `text` or `json`")),
                    };
                }
                "--output" => options.output = Some(value(&mut args, "--output")?),
                "--compare" => options.compare = Some(value(&mut args, "--compare")?),
                "--threshold" => {
                    let raw = value(&mut args, "--threshold")?;
                    let parsed = raw
                        .parse::<f64>()
                        .map_err(|_| format!("`{raw}` is not a number"))?;
                    if parsed <= 0.0 {
                        return Err("`--threshold` must be positive".to_string());
                    }
                    options.threshold_percent = parsed;
                }
                "--label" => options.label = Some(value(&mut args, "--label")?),
                other => return Err(format!("unknown argument `{other}`")),
            }
        }
        Ok(Some(options))
    }
}

fn value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next().ok_or_else(|| format!("`{flag}` needs a value"))
}

fn positive(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<usize, String> {
    let value = value(args, flag)?;
    match value.parse::<usize>() {
        Ok(parsed) if parsed > 0 => Ok(parsed),
        _ => Err(format!("`{flag}` needs a positive integer, got `{value}`")),
    }
}

fn print_help() {
    println!(
        "\n\
nudo-bench — reproducible measurements of the NUDO front end

USAGE:
    nudo-bench [OPTIONS]

OPTIONS:
    --iterations <N>    Timed iterations per case (default {DEFAULT_ITERATIONS})
    --warmup <N>        Untimed warm-up iterations per case (default {DEFAULT_WARMUP})
    --format <F>        `text` (default) or `json`
    --output <PATH>     Write JSON results to a file instead of stdout
    --compare <PATH>    Compare this run against a JSON result and print Markdown
    --threshold <P>     Regression threshold in percent (default {DEFAULT_THRESHOLD_PERCENT})
    --label <TEXT>      Free-form label stored in the metadata
    -h, --help          Print this help

Stages measured: lex, parse, hir, typeck.
Stages that cannot exist yet: nothing runs and nothing executes a file, so there
is no interpreter benchmark.
"
    );
}

// ----------------------------------------------------------------- cases ---

struct Case {
    case: &'static str,
    stage: Stage,
    source: String,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    Lex,
    Parse,
    Hir,
    Typeck,
}

impl Stage {
    const fn as_str(self) -> &'static str {
        match self {
            Stage::Lex => "lex",
            Stage::Parse => "parse",
            Stage::Hir => "hir",
            Stage::Typeck => "typeck",
        }
    }
}

fn build_cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for (case, source) in [
        ("functions", synthetic_functions(2_000)),
        ("bindings", synthetic_bindings(5_000)),
        ("text-literals", synthetic_text(2_000)),
        ("comments", synthetic_comments(2_000)),
    ] {
        cases.push(Case {
            case,
            stage: Stage::Lex,
            source: source.clone(),
        });
        cases.push(Case {
            case,
            stage: Stage::Parse,
            source: source.clone(),
        });
        cases.push(Case {
            case,
            stage: Stage::Hir,
            source: source.clone(),
        });
        cases.push(Case {
            case,
            stage: Stage::Typeck,
            source,
        });
    }
    cases
}

// ------------------------------------------------------------ measurement ---

struct CaseResult {
    case: &'static str,
    stage: Stage,
    bytes: usize,
    units: usize,
    unit: &'static str,
    min_ns: u128,
    median_ns: u128,
    p95_ns: u128,
    mean_ns: u128,
    throughput_mib_s: f64,
    iterations: usize,
}

fn run_case(case: &Case, iterations: usize, warmup: usize) -> CaseResult {
    let prepared = Prepared::new(case);

    for _ in 0..warmup {
        black_box(prepared.run());
    }

    let mut samples = Vec::with_capacity(iterations);
    let mut units = 0;
    for _ in 0..iterations {
        let started = Instant::now();
        let measured = prepared.run();
        samples.push(started.elapsed().as_nanos());
        units = measured;
    }
    samples.sort_unstable();

    let min_ns = *samples.first().unwrap_or(&0);
    let median_ns = percentile(&samples, 50.0);
    let p95_ns = percentile(&samples, 95.0);
    let total: u128 = samples.iter().sum();
    let divisor = u128::try_from(samples.len()).unwrap_or(1).max(1);
    let mean_ns = total / divisor;

    let seconds = median_ns as f64 / 1_000_000_000.0;
    let throughput_mib_s = if seconds > 0.0 {
        (case.source.len() as f64 / (1024.0 * 1024.0)) / seconds
    } else {
        0.0
    };

    CaseResult {
        case: case.case,
        stage: case.stage,
        bytes: case.source.len(),
        units,
        unit: prepared.unit(),
        min_ns,
        median_ns,
        p95_ns,
        mean_ns,
        throughput_mib_s,
        iterations,
    }
}

/// Nearest-rank percentile. `sorted` must be ascending and non-empty.
fn percentile(sorted: &[u128], percentile: f64) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = ((percentile / 100.0) * sorted.len() as f64).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted.len() - 1);
    sorted[index]
}

/// The per-case setup that must happen *outside* the timed region.
enum Prepared {
    Lex {
        source: String,
    },
    Parse {
        source: String,
    },
    Hir {
        source_id: nudo_source::SourceId,
        tree: nudo_syntax::SyntaxTree,
    },
    Typeck {
        source_id: nudo_source::SourceId,
        hir: nudo_hir::Hir,
    },
}

impl Prepared {
    fn new(case: &Case) -> Self {
        match case.stage {
            Stage::Lex => Self::Lex {
                source: case.source.clone(),
            },
            Stage::Parse => Self::Parse {
                source: case.source.clone(),
            },
            Stage::Hir => {
                let mut sources = SourceMap::new();
                let source_id = sources.add("nudo-bench.nudo", case.source.as_str());
                let file = sources.get(source_id).expect("just added");
                let tree = parse(file).into_parts().0;
                Self::Hir { source_id, tree }
            }
            Stage::Typeck => {
                let mut sources = SourceMap::new();
                let source_id = sources.add("nudo-bench.nudo", case.source.as_str());
                let file = sources.get(source_id).expect("just added");
                let tree = parse(file).into_parts().0;
                let hir = lower(&tree, source_id).hir;
                Self::Typeck { source_id, hir }
            }
        }
    }

    const fn unit(&self) -> &'static str {
        match self {
            Self::Lex { .. } | Self::Parse { .. } => "tokens",
            Self::Hir { .. } | Self::Typeck { .. } => "definitions",
        }
    }

    /// Runs the timed region and returns the unit count it observed.
    fn run(&self) -> usize {
        match self {
            Self::Lex { source } => {
                let mut sources = SourceMap::new();
                let id = sources.add("nudo-bench.nudo", source.as_str());
                let file = sources.get(id).expect("just added");
                black_box(nudo_lexer::tokenize(file).tokens().len())
            }
            Self::Parse { source } => {
                let mut sources = SourceMap::new();
                let id = sources.add("nudo-bench.nudo", source.as_str());
                let file = sources.get(id).expect("just added");
                let parsed = parse(file);
                black_box(parsed.tree().node_count())
            }
            Self::Hir { source_id, tree } => {
                let lowered = lower(tree, *source_id);
                black_box(lowered.hir.defs().len())
            }
            Self::Typeck { source_id, hir } => {
                let checked = check(hir, *source_id);
                black_box(checked.expression_types.len())
            }
        }
    }
}

// ------------------------------------------------------------- generators ---

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
            "let message_{index} = \"line {index}\\ttab\\\\nnewline\\\\\\\\slash\";\n"
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

// ----------------------------------------------------------------- report ---

struct Report {
    metadata: Metadata,
    cases: Vec<CaseResult>,
}

struct Metadata {
    commit: String,
    dirty: bool,
    rustc: String,
    os: String,
    arch: String,
    cpu: String,
    iterations: usize,
    warmup: usize,
    timestamp_unix: u64,
    label: Option<String>,
}

impl Metadata {
    fn collect(iterations: usize, warmup: usize, label: Option<String>) -> Self {
        Self {
            commit: command("git", &["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".into()),
            dirty: command("git", &["status", "--porcelain"])
                .is_some_and(|status| !status.is_empty()),
            rustc: command("rustc", &["--version"]).unwrap_or_else(|| "unknown".into()),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu: cpu_description(),
            iterations,
            warmup,
            timestamp_unix: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| duration.as_secs()),
            label,
        }
    }
}

fn command(program: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text.lines().next().unwrap_or_default().to_string())
    }
}

fn cpu_description() -> String {
    if let Ok(text) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in text.lines() {
            if let Some(model) = line.strip_prefix("model name") {
                return model
                    .trim_start_matches(|c: char| c == ':' || c.is_whitespace())
                    .to_string();
            }
        }
    }
    std::env::consts::ARCH.to_string()
}

impl Report {
    fn to_json(&self) -> String {
        let mut out = String::new();
        out.push_str("{\n");
        out.push_str("  \"schema\": \"nudo-bench/v1\",\n");
        out.push_str(&format!(
            "  \"label\": {},\n",
            match &self.metadata.label {
                Some(label) => json_string(label),
                None => "null".to_string(),
            }
        ));
        out.push_str("  \"metadata\": {\n");
        out.push_str(&format!(
            "    \"commit\": {},\n",
            json_string(&self.metadata.commit)
        ));
        out.push_str(&format!("    \"dirty\": {},\n", self.metadata.dirty));
        out.push_str(&format!(
            "    \"rustc\": {},\n",
            json_string(&self.metadata.rustc)
        ));
        out.push_str(&format!(
            "    \"os\": {},\n",
            json_string(&self.metadata.os)
        ));
        out.push_str(&format!(
            "    \"arch\": {},\n",
            json_string(&self.metadata.arch)
        ));
        out.push_str(&format!(
            "    \"cpu\": {},\n",
            json_string(&self.metadata.cpu)
        ));
        out.push_str(&format!(
            "    \"iterations\": {},\n",
            self.metadata.iterations
        ));
        out.push_str(&format!("    \"warmup\": {},\n", self.metadata.warmup));
        out.push_str(&format!(
            "    \"timestamp_unix\": {}\n",
            self.metadata.timestamp_unix
        ));
        out.push_str("  },\n");
        out.push_str("  \"cases\": [\n");
        for (index, case) in self.cases.iter().enumerate() {
            out.push_str("    {\n");
            out.push_str(&format!("      \"case\": {},\n", json_string(case.case)));
            out.push_str(&format!(
                "      \"stage\": {},\n",
                json_string(case.stage.as_str())
            ));
            out.push_str(&format!("      \"bytes\": {},\n", case.bytes));
            out.push_str(&format!("      \"units\": {},\n", case.units));
            out.push_str(&format!("      \"unit\": {},\n", json_string(case.unit)));
            out.push_str(&format!("      \"min_ns\": {},\n", case.min_ns));
            out.push_str(&format!("      \"median_ns\": {},\n", case.median_ns));
            out.push_str(&format!("      \"p95_ns\": {},\n", case.p95_ns));
            out.push_str(&format!("      \"mean_ns\": {},\n", case.mean_ns));
            out.push_str(&format!(
                "      \"throughput_mib_s\": {:.3},\n",
                case.throughput_mib_s
            ));
            out.push_str(&format!("      \"iterations\": {}\n", case.iterations));
            out.push_str(if index + 1 == self.cases.len() {
                "    }\n"
            } else {
                "    },\n"
            });
        }
        out.push_str("  ]\n}\n");
        out
    }
}

fn json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn print_text(report: &Report) {
    println!("nudo-bench — NUDO front-end measurements");
    println!(
        "commit {} ({}), {}, {}",
        report.metadata.commit,
        if report.metadata.dirty {
            "dirty"
        } else {
            "clean"
        },
        report.metadata.rustc,
        report.metadata.cpu
    );
    println!(
        "timed iterations: {}, warm-up: {}\n",
        report.metadata.iterations, report.metadata.warmup
    );
    println!(
        "{:<6} {:<16} {:>9} {:>9} {:>10} {:>10} {:>10} {:>12}",
        "stage", "case", "bytes", "units", "min", "median", "p95", "throughput"
    );
    for case in &report.cases {
        println!(
            "{:<6} {:<16} {:>9} {:>9} {:>10} {:>10} {:>10} {:>12}",
            case.stage.as_str(),
            case.case,
            case.bytes,
            case.units,
            format_duration(case.min_ns),
            format_duration(case.median_ns),
            format_duration(case.p95_ns),
            format!("{:.1} MiB/s", case.throughput_mib_s),
        );
    }
    println!(
        "\nMeasurements are indicative, not a promise: NUDO is pre-alpha and no\n\
         performance target has been accepted. See `benchmarks/README.md`."
    );
}

fn format_duration(nanos: u128) -> String {
    if nanos < 1_000 {
        format!("{nanos}ns")
    } else if nanos < 1_000_000 {
        format!("{:.2}µs", nanos as f64 / 1_000.0)
    } else {
        format!("{:.2}ms", nanos as f64 / 1_000_000.0)
    }
}

// ------------------------------------------------------------- comparison ---

struct Baseline {
    metadata_commit: String,
    medians: BTreeMap<(String, String), u128>,
}

impl Baseline {
    fn parse(text: &str) -> Result<Self, String> {
        let value = Json::parse(text)?;
        let root = value.as_object().ok_or("baseline is not a JSON object")?;
        let metadata = root
            .get("metadata")
            .and_then(Json::as_object)
            .ok_or("baseline has no `metadata` object")?;
        let commit = metadata
            .get("commit")
            .and_then(Json::as_str)
            .unwrap_or("unknown")
            .to_string();
        let cases = root
            .get("cases")
            .and_then(Json::as_array)
            .ok_or("baseline has no `cases` array")?;
        let mut medians = BTreeMap::new();
        for case in cases {
            let object = case.as_object().ok_or("a case is not an object")?;
            let name = object
                .get("case")
                .and_then(Json::as_str)
                .ok_or("a case has no name")?;
            let stage = object
                .get("stage")
                .and_then(Json::as_str)
                .ok_or("a case has no stage")?;
            let median = object
                .get("median_ns")
                .and_then(Json::as_number)
                .ok_or("a case has no median")?;
            if median < 0.0 {
                return Err("a median is negative".into());
            }
            medians.insert(
                (stage.to_string(), name.to_string()),
                median.round() as u128,
            );
        }
        Ok(Self {
            metadata_commit: commit,
            medians,
        })
    }
}

struct ComparisonRow {
    stage: String,
    case: String,
    baseline_ns: u128,
    current_ns: u128,
    delta_percent: f64,
}

struct Comparison {
    baseline_commit: String,
    current_commit: String,
    threshold_percent: f64,
    rows: Vec<ComparisonRow>,
    missing: Vec<(String, String)>,
}

impl Comparison {
    fn regressions(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| row.delta_percent > self.threshold_percent)
            .count()
    }

    fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("## nudo-bench comparison\n\n");
        out.push_str(&format!(
            "* baseline: `{}`\n* current: `{}`\n* regression threshold: {:.1}%\n\n",
            self.baseline_commit, self.current_commit, self.threshold_percent
        ));
        out.push_str("| Stage | Case | Baseline | Current | Delta |\n");
        out.push_str("| ----- | ---- | -------- | ------- | ----- |\n");
        for row in &self.rows {
            let verdict = if row.delta_percent > self.threshold_percent {
                " **regression**"
            } else if row.delta_percent < -self.threshold_percent {
                " improvement"
            } else {
                ""
            };
            out.push_str(&format!(
                "| {} | {} | {} | {} | {:+.1}%{} |\n",
                row.stage,
                row.case,
                format_duration(row.baseline_ns),
                format_duration(row.current_ns),
                row.delta_percent,
                verdict
            ));
        }
        if !self.missing.is_empty() {
            out.push_str("\nCases with no baseline measurement:\n\n");
            for (stage, case) in &self.missing {
                out.push_str(&format!("* `{stage}` / `{case}`\n"));
            }
        }
        out.push_str(&format!(
            "\n{} regression(s) above the {:.1}% threshold.\n",
            self.regressions(),
            self.threshold_percent
        ));
        out.push_str(
            "\nA single comparison is not proof of a regression on shared hardware.\n\
             Re-run both sides on the same machine before acting on a number near the\n\
             threshold; this report does not fail CI by itself.\n",
        );
        out
    }
}

fn compare(report: &Report, baseline: &Baseline, threshold_percent: f64) -> Comparison {
    let mut rows = Vec::new();
    let mut missing = Vec::new();
    for case in &report.cases {
        let key = (case.stage.as_str().to_string(), case.case.to_string());
        match baseline.medians.get(&key) {
            Some(base) if *base > 0 => {
                let current = case.median_ns;
                let delta_percent = ((current as f64 - *base as f64) / *base as f64) * 100.0;
                rows.push(ComparisonRow {
                    stage: key.0,
                    case: key.1,
                    baseline_ns: *base,
                    current_ns: current,
                    delta_percent,
                });
            }
            _ => missing.push(key),
        }
    }
    Comparison {
        baseline_commit: baseline.metadata_commit.clone(),
        current_commit: report.metadata.commit.clone(),
        threshold_percent,
        rows,
        missing,
    }
}

// ------------------------------------------------------------ mini JSON ---

/// A minimal JSON reader, enough for the schema this binary writes.
///
/// It exists instead of a dependency because the workspace has zero third-party
/// dependencies on purpose (`AGENTS.md`), and reading back our own output does
/// not justify the first one. It rejects malformed input rather than guessing.
enum Json {
    #[allow(dead_code)]
    Null,
    #[allow(dead_code)]
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn parse(text: &str) -> Result<Self, String> {
        let mut parser = JsonParser {
            bytes: text.as_bytes(),
            position: 0,
        };
        let value = parser.value()?;
        parser.skip_whitespace();
        if parser.position != parser.bytes.len() {
            return Err(format!("trailing content at byte {}", parser.position));
        }
        Ok(value)
    }

    fn as_object(&self) -> Option<&Vec<(String, Json)>> {
        match self {
            Json::Object(entries) => Some(entries),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&Vec<Json>> {
        match self {
            Json::Array(items) => Some(items),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Json::String(text) => Some(text),
            _ => None,
        }
    }

    fn as_number(&self) -> Option<f64> {
        match self {
            Json::Number(number) => Some(*number),
            _ => None,
        }
    }
}

trait JsonObjectExt {
    fn get(&self, key: &str) -> Option<&Json>;
}

impl JsonObjectExt for Vec<(String, Json)> {
    fn get(&self, key: &str) -> Option<&Json> {
        self.iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }
}

struct JsonParser<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl JsonParser<'_> {
    fn value(&mut self) -> Result<Json, String> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Json::String(self.string()?)),
            Some(b't') => self.literal("true", Json::Bool(true)),
            Some(b'f') => self.literal("false", Json::Bool(false)),
            Some(b'n') => self.literal("null", Json::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(other) => Err(format!(
                "unexpected byte `{}` at {}",
                other as char, self.position
            )),
            None => Err("unexpected end of input".into()),
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.expect(b'{')?;
        let mut entries = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(Json::Object(entries));
        }
        loop {
            self.skip_whitespace();
            let key = self.string()?;
            self.skip_whitespace();
            self.expect(b':')?;
            let value = self.value()?;
            entries.push((key, value));
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b'}') => {
                    self.position += 1;
                    return Ok(Json::Object(entries));
                }
                _ => return Err(format!("expected `,` or `}}` at {}", self.position)),
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(Json::Array(items));
        }
        loop {
            items.push(self.value()?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b']') => {
                    self.position += 1;
                    return Ok(Json::Array(items));
                }
                _ => return Err(format!("expected `,` or `]` at {}", self.position)),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let byte = self
                .bytes
                .get(self.position)
                .copied()
                .ok_or("unterminated string")?;
            self.position += 1;
            match byte {
                b'"' => return Ok(out),
                b'\\' => {
                    let escape = self
                        .bytes
                        .get(self.position)
                        .copied()
                        .ok_or("unterminated escape")?;
                    self.position += 1;
                    match escape {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let hex = self
                                .bytes
                                .get(self.position..self.position + 4)
                                .ok_or("truncated \\u escape")?;
                            let hex = std::str::from_utf8(hex)
                                .map_err(|_| "invalid \\u escape".to_string())?;
                            let code = u32::from_str_radix(hex, 16)
                                .map_err(|_| "invalid \\u escape".to_string())?;
                            self.position += 4;
                            out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                        }
                        other => return Err(format!("unknown escape `\\{}`", other as char)),
                    }
                }
                byte if byte < 0x20 => return Err("control character in string".into()),
                byte => {
                    let start = self.position - 1;
                    let width = utf8_width(byte);
                    let end = start + width;
                    let slice = self
                        .bytes
                        .get(start..end)
                        .ok_or("truncated UTF-8 sequence")?;
                    let text = std::str::from_utf8(slice)
                        .map_err(|_| "invalid UTF-8 in string".to_string())?;
                    out.push_str(text);
                    self.position = end;
                }
            }
        }
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.position;
        if self.peek() == Some(b'-') {
            self.position += 1;
        }
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.position += 1;
        }
        if self.peek() == Some(b'.') {
            self.position += 1;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.position += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.position += 1;
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.position])
            .map_err(|_| "invalid number".to_string())?;
        text.parse::<f64>()
            .map(Json::Number)
            .map_err(|_| format!("invalid number `{text}`"))
    }

    fn literal(&mut self, text: &str, value: Json) -> Result<Json, String> {
        if self.bytes[self.position..].starts_with(text.as_bytes()) {
            self.position += text.len();
            Ok(value)
        } else {
            Err(format!("expected `{text}` at {}", self.position))
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        if self.peek() == Some(byte) {
            self.position += 1;
            Ok(())
        } else {
            Err(format!("expected `{}` at {}", byte as char, self.position))
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.position += 1;
        }
    }
}

const fn utf8_width(first: u8) -> usize {
    if first < 0x80 {
        1
    } else if first >> 5 == 0b110 {
        2
    } else if first >> 4 == 0b1110 {
        3
    } else {
        4
    }
}
