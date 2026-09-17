#!/usr/bin/env python3
"""Checks that the EBNF grammar is usable by the parser NUDO says it will have.

Three classes of defect are caught, all of which the repository has shipped at
least one of:

1. **Left recursion.** `nudo-parser` will be a recursive-descent parser, which
   cannot handle a production that derives a form starting with itself. The
   previous revision of `grammar/nudo.ebnf` had four such productions
   (`call-expression`, `field-expression`, `binary-expression`,
   `optional-type`), so the grammar was not parseable by the strategy the
   project specified. Direct and indirect recursion are both detected.

2. **Unstated precedence.** Operator precedence written in prose drifts from the
   productions, silently. The `PRECEDENCE:` marker in the EBNF is the single
   definition; this checker verifies that the productions actually implement it,
   and that the identical marker appears in `grammar/syntax-reference.md` and
   `spec/expressions.md`.

3. **Broken references.** A production that names something undefined, or a
   production nothing can reach from the entry points.

Run by `scripts/check.sh` and by the `CI` workflow. Exit code 0 when the grammar
is sound, 1 otherwise.
"""

from __future__ import annotations

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EBNF = os.path.join(ROOT, "grammar", "nudo.ebnf")
ENTRY_POINTS = ("program", "source-file")
# Files that must carry a byte-identical copy of the precedence marker.
MARKER_FILES = ("grammar/syntax-reference.md", "spec/expressions.md")

IDENT = re.compile(r"[A-Za-z][A-Za-z0-9_-]*")
SYMBOLS = set("=|,;()[]{}<>")


# --------------------------------------------------------------------------- #
# Scanning
# --------------------------------------------------------------------------- #

class Token:
    __slots__ = ("kind", "value")

    def __init__(self, kind: str, value: str) -> None:
        self.kind = kind  # ident | string | special | symbol
        self.value = value

    def __repr__(self) -> str:  # pragma: no cover - debugging aid
        return f"Token({self.kind}, {self.value!r})"


def scan(text: str, origin: str) -> list[Token]:
    """Tokenises the EBNF subset this repository uses.

    Comments, string literals and special sequences are recognised in one pass,
    so a `?` inside a string cannot start a special sequence and vice versa.
    """
    tokens: list[Token] = []
    index = 0
    length = len(text)
    while index < length:
        char = text[index]
        if text.startswith("(*", index):
            end = text.find("*)", index + 2)
            if end == -1:
                raise SystemExit(f"{origin}: unterminated comment at offset {index}")
            index = end + 2
        elif char == "?":
            end = text.find("?", index + 1)
            if end == -1:
                raise SystemExit(f"{origin}: unterminated special sequence at offset {index}")
            tokens.append(Token("special", text[index : end + 1]))
            index = end + 1
        elif char in {'"', "'"}:
            # EBNF terminals are written with either delimiter, and the grammar
            # uses both: '"' for the quote character itself, "..." elsewhere.
            end = text.find(char, index + 1)
            if end == -1:
                raise SystemExit(f"{origin}: unterminated terminal at offset {index}")
            tokens.append(Token("string", text[index : end + 1]))
            index = end + 1
        elif char in SYMBOLS:
            tokens.append(Token("symbol", char))
            index += 1
        elif char.isspace():
            index += 1
        else:
            match = IDENT.match(text, index)
            if match is None:
                raise SystemExit(f"{origin}: unexpected character {char!r} at offset {index}")
            tokens.append(Token("ident", match.group(0)))
            index = match.end()
    return tokens


# --------------------------------------------------------------------------- #
# Parsing the EBNF subset
# --------------------------------------------------------------------------- #

class Node:
    """A parsed right-hand side: an alternation of sequences of factors."""

    def __init__(self, alternatives: list[list["Factor"]]) -> None:
        self.alternatives = alternatives


class Factor:
    def __init__(self, kind: str, value=None) -> None:
        # kind: ident | terminal | optional | repeated | grouped
        self.kind = kind
        self.value = value


class Parser:
    def __init__(self, tokens: list[Token], origin: str) -> None:
        self.tokens = tokens
        self.position = 0
        self.origin = origin

    def peek(self) -> Token | None:
        return self.tokens[self.position] if self.position < len(self.tokens) else None

    def next(self) -> Token:
        token = self.peek()
        if token is None:
            raise SystemExit(f"{self.origin}: unexpected end of file")
        self.position += 1
        return token

    def expect(self, value: str) -> None:
        token = self.next()
        if token.kind != "symbol" or token.value != value:
            raise SystemExit(f"{self.origin}: expected {value!r}, found {token.value!r}")

    def parse_rule(self) -> tuple[str, Node]:
        name = self.next()
        if name.kind != "ident":
            raise SystemExit(f"{self.origin}: expected a production name, found {name.value!r}")
        self.expect("=")
        body = self.parse_alternation()
        self.expect(";")
        return name.value, body

    def parse_alternation(self) -> Node:
        alternatives = [self.parse_sequence()]
        while self.peek() is not None and self.peek().value == "|":
            self.next()
            alternatives.append(self.parse_sequence())
        return Node(alternatives)

    def parse_sequence(self) -> list[Factor]:
        factors = [self.parse_factor()]
        while self.peek() is not None and self.peek().value == ",":
            self.next()
            factors.append(self.parse_factor())
        return factors

    def parse_factor(self) -> Factor:
        token = self.next()
        if token.kind == "ident":
            return Factor("ident", token.value)
        if token.kind in {"string", "special"}:
            return Factor("terminal", token.value)
        if token.kind == "symbol":
            if token.value == "[":
                inner = self.parse_alternation()
                self.expect("]")
                return Factor("optional", inner)
            if token.value == "{":
                inner = self.parse_alternation()
                self.expect("}")
                return Factor("repeated", inner)
            if token.value == "(":
                inner = self.parse_alternation()
                self.expect(")")
                return Factor("grouped", inner)
        raise SystemExit(f"{self.origin}: unexpected token {token.value!r} in a production")


def parse_grammar(path: str) -> dict[str, Node]:
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    tokens = scan(text, os.path.relpath(path, ROOT))
    parser = Parser(tokens, os.path.relpath(path, ROOT))
    productions: dict[str, Node] = {}
    while parser.peek() is not None:
        name, body = parser.parse_rule()
        if name in productions:
            raise SystemExit(f"{os.path.relpath(path, ROOT)}: {name} is defined twice")
        productions[name] = body
    return productions


# --------------------------------------------------------------------------- #
# Analysis
# --------------------------------------------------------------------------- #

def compute(productions: dict[str, Node]):
    """Returns (nullable, first_symbols) to a fixed point.

    `first_symbols` is the set of non-terminals that can appear leftmost in a
    derivation without consuming a terminal first. A production that contains
    itself there is left-recursive.
    """
    nullable: dict[str, bool] = dict.fromkeys(productions, False)
    firsts: dict[str, set[str]] = {name: set() for name in productions}

    changed = True
    while changed:
        changed = False

        def node_nullable(node: Node) -> bool:
            return any(all(factor_nullable(factor) for factor in alternative)
                       for alternative in node.alternatives)

        def factor_nullable(factor: Factor) -> bool:
            if factor.kind == "ident":
                return nullable.get(factor.value, False)
            if factor.kind in {"optional", "repeated"}:
                return True
            if factor.kind == "grouped":
                return node_nullable(factor.value)
            return False

        def node_firsts(node: Node) -> set[str]:
            result: set[str] = set()
            for alternative in node.alternatives:
                for factor in alternative:
                    result |= factor_firsts(factor)
                    if not factor_nullable(factor):
                        break
            return result

        def factor_firsts(factor: Factor) -> set[str]:
            if factor.kind == "ident":
                return {factor.value} | firsts.get(factor.value, set())
            if factor.kind in {"optional", "repeated", "grouped"}:
                return node_firsts(factor.value)
            return set()

        for name, body in productions.items():
            now_nullable = node_nullable(body)
            if now_nullable != nullable[name]:
                nullable[name] = now_nullable
                changed = True
            now_firsts = node_firsts(body)
            if now_firsts != firsts[name]:
                firsts[name] = now_firsts
                changed = True

    # `firsts` is transitive, which is what left-recursion detection needs:
    # `expression` does start with `primary-expression`, through the chain.
    # The precedence check needs the opposite: only the nonterminal each level
    # names directly, so that "level A is defined in terms of level B" can be
    # tested without the whole chain collapsing into one set.
    direct: dict[str, set[str]] = {}

    def node_direct(node: Node) -> set[str]:
        result: set[str] = set()
        for alternative in node.alternatives:
            for factor in alternative:
                result |= factor_direct(factor)
                if not factor_nullable(factor):
                    break
        return result

    def factor_direct(factor: Factor) -> set[str]:
        if factor.kind == "ident":
            return {factor.value}
        if factor.kind in {"optional", "repeated", "grouped"}:
            return node_direct(factor.value)
        return set()

    for name, body in productions.items():
        direct[name] = node_direct(body)

    return nullable, firsts, direct


def reachable(productions: dict[str, Node]) -> set[str]:
    def walk(node: Node, seen: set[str]) -> None:
        for alternative in node.alternatives:
            for factor in alternative:
                if factor.kind == "ident":
                    if factor.value not in seen and factor.value in productions:
                        seen.add(factor.value)
                        walk(productions[factor.value], seen)
                elif factor.kind in {"optional", "repeated", "grouped"}:
                    walk(factor.value, seen)

    seen: set[str] = set()
    for entry in ENTRY_POINTS:
        if entry in productions:
            seen.add(entry)
            walk(productions[entry], seen)
    return seen


def marker_chain() -> list[str]:
    with open(EBNF, encoding="utf-8") as handle:
        text = handle.read()
    match = re.search(r"\(\*\s*PRECEDENCE:(.*?)\*\)", text, re.S)
    if not match:
        raise SystemExit("grammar/nudo.ebnf: no PRECEDENCE marker")
    return [level.strip() for level in match.group(1).split(">") if level.strip()]


def marker_text() -> str:
    with open(EBNF, encoding="utf-8") as handle:
        text = handle.read()
    match = re.search(r"\(\*\s*PRECEDENCE:(.*?)\*\)", text, re.S)
    assert match is not None
    return " ".join(match.group(1).split())


def main() -> int:
    productions = parse_grammar(EBNF)
    nullable, firsts, direct = compute(productions)
    problems: list[str] = []
    notes: list[str] = []

    # 1. Left recursion.
    recursive = sorted(name for name in productions if name in firsts[name])
    for name in recursive:
        problems.append(
            f"{name} is left-recursive: it can derive a form starting with itself, "
            "which a recursive-descent parser cannot handle"
        )

    # 2. Broken and unreachable references.
    for name, body in productions.items():
        for alternative in body.alternatives:
            for factor in alternative:
                if factor.kind == "ident" and factor.value not in productions:
                    problems.append(f"{name} references undefined production {factor.value}")
    seen = reachable(productions)
    unreachable = sorted(set(productions) - seen)
    if unreachable:
        notes.append(f"unreachable from the entry points: {', '.join(unreachable)}")

    # 3. The declared precedence chain against the productions.
    chain = marker_chain()
    if not chain:
        problems.append("the PRECEDENCE marker is empty")
    else:
        if chain[0] != "expression":
            problems.append(f"the precedence chain starts at {chain[0]}, expected expression")
        for level in chain:
            if level not in productions:
                problems.append(f"the precedence chain names undefined production {level}")
        for current, following in zip(chain, chain[1:]):
            if current not in productions or following not in productions:
                continue
            leftmost = direct[current]
            if following not in leftmost:
                problems.append(
                    f"{current} is not defined in terms of {following}; the declared "
                    "precedence chain and the productions disagree"
                )
            for other in sorted(leftmost - {following}):
                # An extra leftmost nonterminal is acceptable only when it is a
                # pure operator production, whose own body is terminals: that is
                # what makes `-x` and `!x` one token of lookahead away from an
                # operand, and `==` and `!=` one token away from a comparison.
                if direct.get(other):
                    problems.append(
                        f"{current} can also start with {other}, whose own body is not a pure "
                        "operator production, so its precedence is ambiguous"
                    )

    # 4. The same marker must appear in the documents that state precedence.
    expected = marker_text()
    for relative in MARKER_FILES:
        path = os.path.join(ROOT, relative)
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        match = re.search(r"<!--\s*PRECEDENCE:(.*?)-->", text, re.S)
        if not match:
            problems.append(f"{relative} has no PRECEDENCE marker")
            continue
        found = " ".join(match.group(1).split())
        if found != expected:
            problems.append(
                f"{relative} states a different precedence chain than grammar/nudo.ebnf"
            )

    print(f"grammar: {len(productions)} productions, {len(recursive)} left-recursive, "
          f"{len(chain)} precedence levels")

    for note in notes:
        print(f"info: {note}")

    if problems:
        print("\ngrammar problems:")
        for problem in problems:
            print(f"  - {problem}")
        return 1

    print("grammar: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
