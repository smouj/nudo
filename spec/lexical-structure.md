# Lexical structure

**State: implemented** (milestone M1). This chapter describes what
`compiler/nudo-lexer` does today, and is backed by cases in
[`../tests/conformance/lexer`](../tests/conformance/lexer).

## Source files

NUDO source files MUST be UTF-8. A file that is not valid UTF-8 is a tool error,
not a language error: the compiler cannot report a position inside it reliably.

The conventional source file extension is `.nudo`, and `main.nudo` is the
conventional entry point of a package. A file that uses another extension is
accepted with a warning (`NDO8001`); the extension is a convention, not a
syntactic rule.

`.nu` is **not** used. It is already associated with another ecosystem, and a
new language should not collide with it.

A byte-order mark (`U+FEFF`) at the very beginning of a file is ignored.

## Whitespace

Space, tab, carriage return, line feed and any other Unicode whitespace
character separate tokens. Line endings may be `\n` or `\r\n`; both are
equivalent, and neither is a token.

## Comments

Two forms, both of which are trivia and produce no token:

```nudo
// a line comment, ending at the line terminator

/* a block comment,
   which may span lines */
```

Block comments **nest**:

```nudo
/* outer /* inner */ still inside the outer comment */
```

Nesting is not a stylistic choice. A block comment is how code is temporarily
disabled, and a commented-out region that contains a comment must not end early.
An unterminated block comment is `NDO1004`.

A line comment cannot be continued with a backslash. A block comment opened
inside a text literal does not open a comment; the literal wins, because
literal scanning consumes everything up to its closing quote.

## Identifiers

```text
identifier-start    := ASCII letter | "_"
identifier-continue := identifier-start | ASCII digit
identifier          := identifier-start, { identifier-continue }
```

Identifiers are ASCII in the current implementation. Non-ASCII letters are
reported as `NDO1002` rather than guessed at.

**Open question (needs a NEP):** whether NUDO has Unicode identifiers at all.
The argument for is that a language meant for people should not reject their
names. The argument against is normalisation and homoglyph attacks in a language
that grants capabilities. Until it is decided, ASCII only, and the lexer says so
instead of pretending.

## Reserved words

The current toolchain reserves exactly two words:

```text
fn    let
```

Everything else that the language may eventually reserve — `struct`, `enum`,
`agent`, `task`, `tool`, `model`, `policy`, `verify`, `ask`, `allow` and so on —
is an ordinary identifier today, and appears in the grammar as provisional.

This is deliberate. Reserving a word is a breaking change for every program that
uses it as a name, so it happens when the feature lands, not before. The current
state is reported by `nudo check`, which never claims to understand syntax it
does not implement.

## Numbers

```text
digit        := "0" … "9"
integer      := digit, { digit | "_" digit }
float        := integer, ".", digit, { digit | "_" digit }
```

* `_` may separate digits, and MUST NOT start or end a literal, or appear twice
  in a row: `1_000_000` is valid, `1_`, `_1` and `1__0` are not.
* `_1` is an identifier, not a number: identifiers win at the start of a token.
* A `.` is part of a float only when a digit follows it. Otherwise the number
  ends, and the `.` is lexed on its own.
* A digit sequence immediately followed by an identifier character is
  `NDO1005`: `123abc` is one malformed literal, not a number followed by a name.
* The current implementation lexes the token and does not compute its value.
  Value computation, radix literals and suffixes arrive with the parser (M2) and
  require a NEP.

## Text literals

```text
text := '"', { character | escape }, '"'
```

A text literal opens and closes with `"` on the **same line**. A newline inside
a literal is `NDO1003`, not a continuation: a silently multi-line literal is a
common way to swallow a whole file by accident.

Defined escapes:

| Escape | Meaning |
| ------ | ------- |
| `\n` | line feed |
| `\r` | carriage return |
| `\t` | tab |
| `\\` | backslash |
| `\"` | double quote |
| `\0` | null |

Anything else is `NDO1006`. There is no `\u{…}` escape and no raw-string or
multi-line form yet; both are open questions and need a NEP, because both change
what "this literal ends here" means.

## Punctuation

The token set of the pre-alpha lexer:

| Token | Lexeme | Token | Lexeme |
| ----- | ------ | ----- | ------ |
| `LParen` | `(` | `Plus` | `+` |
| `RParen` | `)` | `Minus` | `-` |
| `LBrace` | `{` | `Star` | `*` |
| `RBrace` | `}` | `Slash` | `/` |
| `Colon` | `:` | `Semi` | `;` |
| `Comma` | `,` | `Arrow` | `->` |
| `Equals` | `=` | | |

`->` is one token, not `-` followed by `>`. `/` is a comment only when followed
by `/` or `*`; otherwise it is `Slash`.

Any other character is `NDO1002`, and the lexer skips it and continues. Adding a
token is a specification change: the token set grows with the grammar, not ahead
of it.

## End of file

Lexing always produces exactly one `Eof` token, last, whose position is the end
of the file. A parser never needs to check for "no more tokens"; it checks for
`Eof`.

## Errors and recovery

Lexing does not stop at the first error. An unknown character is reported and
skipped, and a malformed literal still produces a token so that later stages see
the program's shape rather than a truncated stream.

The guarantees, which are tested adversarially in
`compiler/nudo-lexer/tests/robustness.rs`:

1. lexing terminates on any input;
2. it never panics;
3. every token span lies inside the source and on character boundaries;
4. tokens are ordered and do not overlap;
5. the result is deterministic.
