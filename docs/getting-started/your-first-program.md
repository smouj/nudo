# Your first program

This page is short, because the toolchain can do one thing.

## Write a file

`hello.nudo`:

```nudo
// The smallest thing the pre-alpha toolchain can read.

fn main() {
    let greeting = "hello, nudo";
}

fn add(a: Int, b: Int) -> Int {
    a + b
}

let answer = add(1, 2);
```

## Check it

```console
$ nudo check hello.nudo
checked 1 file: 0 errors and 0 warnings
```

**What that actually means.** `nudo check` reads the file, lexes it, and reports
lexical diagnostics. A clean run means "no lexical diagnostics" — not "this
program is correct", and not "this program runs". There is no parser, no type
checker and no interpreter yet
([`../../ROADMAP.md`](../../ROADMAP.md), M2–M4).

## See what the compiler sees

```console
$ nudo check --dump-tokens hello.nudo
# nudo-tokens v1
0000 5:1-5:3 KeywordFn "fn"
0001 5:4-5:8 Ident "main"
0002 5:8-5:9 LParen "("
...
```

The format is stable and documented in
[`../../tests/conformance/README.md`](../../tests/conformance/README.md). It is
also the format other implementations are checked against, which is why it does
not change casually.

## Break it on purpose

`broken.nudo`:

```nudo
fn main() {
    let greeting = "never closed
}
```

```console
$ nudo check broken.nudo
error[NDO1003]: unterminated text literal
  --> broken.nudo:2:20
  |
2 |     let greeting = "never closed
  |                    ^^^^^^^^^^^^^
   = note: a text literal must be closed on the same line it starts on
   = help: add a closing `"`

nudo: 1 error and 0 warnings
```

Exit code `1` means diagnostics were reported. Exit code `2` means the tool could
not run at all — a usage error, an unreadable file, or an unimplemented command.

## Where to go next

* [`../language/tour.md`](../language/tour.md) — the language NUDO is being
  designed towards. Most of it does not run yet, and the page says so.
* [`../../examples/README.md`](../../examples/README.md) — examples, marked by
  whether the current toolchain can read them.
* [`../../fixtures/README.md`](../../fixtures/README.md) — how to write your own
  test inputs with declared expectations.
