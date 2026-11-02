//! Deterministic robustness properties for the type-checking stage.
//!
//! Typeck consumes HIR, including HIR lowered from syntax trees that recovered
//! from malformed source. These cases make sure that recovery remains safe,
//! diagnostic spans stay inside the source, and repeated checks are stable.

use nudo_hir::lower;
use nudo_parser::parse;
use nudo_source::SourceMap;
use nudo_typeck::{TypeckResult, check, dump_types};

const ALPHABET: &[char] = &[
    'f', 'n', 'l', 'e', 't', 'x', 'A', '_', '0', '1', '9', ' ', '\t', '\n', '\r', '(', ')', '{',
    '}', ':', ',', '-', '>', '=', '+', '*', '/', '%', ';', '"', '\\', '.', '@', '#', '€', 'é',
    '\u{feff}',
];

const STRUCTURED_CHUNKS: &[&str] = &[
    "fn identity<T>(value: T) -> T { value }\n",
    "fn main() { let value = identity(1); }\n",
    "struct Box<T> { value: T }\nfn get<T>(box: Box<T>) -> T { box.value }\n",
    "fn wrong(value: Bool) { let number: Int = value; }\n",
    "fn call(value: Int) { value(1); }\n",
    "fn field(value: Int) { value.missing; }\n",
    "fn arithmetic(left: Int, right: Float) { let sum = left + right; let remainder = right % right; }\n",
    "fn conditional(condition: Bool) { if condition { let value: Int = false; } }\n",
    "fn broken(\n",
];

/// Small deterministic generator shared by the random and structured cases.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut state = self.0;
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        self.0 = state;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next_u64() % bound as u64) as usize
    }
}

fn random_source(rng: &mut Rng, length: usize) -> String {
    let mut source = String::with_capacity(length);
    for _ in 0..length {
        source.push(ALPHABET[rng.below(ALPHABET.len())]);
    }
    source
}

fn structured_source(rng: &mut Rng) -> String {
    let mut source = String::new();
    for _ in 0..rng.below(8) + 1 {
        source.push_str(STRUCTURED_CHUNKS[rng.below(STRUCTURED_CHUNKS.len())]);
    }
    source
}

#[derive(Debug, PartialEq, Eq)]
struct DiagnosticFingerprint {
    code: String,
    severity: String,
    message: String,
    notes: Vec<String>,
    help: Option<String>,
    span: Option<(usize, usize)>,
}

fn diagnostic_fingerprint(result: &TypeckResult) -> Vec<DiagnosticFingerprint> {
    result
        .diagnostics
        .as_slice()
        .iter()
        .map(|diagnostic| DiagnosticFingerprint {
            code: diagnostic.code().id().to_string(),
            severity: diagnostic.severity().as_str().to_string(),
            message: diagnostic.message().to_string(),
            notes: diagnostic.notes().to_vec(),
            help: diagnostic.help().map(str::to_string),
            span: diagnostic
                .span()
                .map(|span| (span.start().as_usize(), span.end().as_usize())),
        })
        .collect()
}

fn check_recovered_input(source: &str, case: usize) -> (bool, bool, bool) {
    let mut sources = SourceMap::new();
    let id = sources.add("robustness.nudo", source);
    let file = sources.get(id).expect("just added");
    let parsed = parse(file);
    let parser_recovered = parsed.has_errors();
    assert!(
        parsed.tree().is_lossless(),
        "case {case}: parser recovery dropped source bytes from {source:?}"
    );
    assert_eq!(
        parsed.tree().validate(),
        Ok(()),
        "case {case}: parser recovery produced a broken tree for {source:?}"
    );

    // Typeck is deliberately exercised even if parsing or name resolution had
    // to recover: the pipeline must not panic on an incomplete HIR.
    let resolved = lower(parsed.tree(), id);
    let resolver_reported = !resolved.diagnostics.is_empty();
    let first = check(&resolved.hir, id);
    let second = check(&resolved.hir, id);
    let typeck_reported = !first.diagnostics.is_empty();

    assert_eq!(
        first.definition_types.len(),
        resolved.hir.defs().len(),
        "case {case}: one type slot is required per definition"
    );
    assert_eq!(
        dump_types(&first, &resolved.hir, file),
        dump_types(&second, &resolved.hir, file),
        "case {case}: type dumps differ between repeated checks for {source:?}"
    );
    assert_eq!(
        diagnostic_fingerprint(&first),
        diagnostic_fingerprint(&second),
        "case {case}: diagnostics differ between repeated checks for {source:?}"
    );

    for diagnostic in first.diagnostics.as_slice() {
        let span = diagnostic.span().unwrap_or_else(|| {
            panic!("case {case}: {} has no source span", diagnostic.code().id())
        });
        let start = span.start().as_usize();
        let end = span.end().as_usize();
        assert!(
            start <= end && end <= source.len(),
            "case {case}: {} span {span} escapes {source:?}",
            diagnostic.code().id()
        );
        assert!(
            source.is_char_boundary(start) && source.is_char_boundary(end),
            "case {case}: {} span {span} splits a UTF-8 character in {source:?}",
            diagnostic.code().id()
        );
    }

    (parser_recovered, resolver_reported, typeck_reported)
}

#[test]
fn random_sources_keep_typeck_results_safe_and_stable() {
    let mut rng = Rng::new(0x7A11_CE55_1234_ABCD);
    for case in 0..400 {
        let length = rng.below(192);
        let source = random_source(&mut rng, length);
        let _ = check_recovered_input(&source, case);
    }
}

#[test]
fn structured_sources_keep_typeck_results_safe_and_stable() {
    let mut rng = Rng::new(0xC0FF_EE12_3456_789B);
    let (parser_recovered, _, _) = check_recovered_input("fn broken(", 9_999);
    assert!(
        parser_recovered,
        "the malformed seed must exercise parser recovery"
    );

    let mut resolver_recoveries = 0;
    let mut typeck_diagnostics = 0;
    for case in 0..160 {
        let source = structured_source(&mut rng);
        let (_, resolver_reported, typeck_reported) = check_recovered_input(&source, 10_000 + case);
        resolver_recoveries += usize::from(resolver_reported);
        typeck_diagnostics += usize::from(typeck_reported);
    }
    assert!(
        resolver_recoveries > 0,
        "structured cases must exercise HIR errors"
    );
    assert!(
        typeck_diagnostics > 0,
        "structured cases must exercise typeck errors"
    );
}
