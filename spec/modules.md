# Modules and packages

**State: proposed, and not implemented.** No command reads a manifest yet.

## Modules

A module is a source file. Its declarations are visible inside it; visibility
across modules is explicit.

```nudo
// a file is a module
pub fn exported() -> Int { 1 }

fn private() -> Int { 2 }
```

* Import syntax is provisional.
* There are no implicit re-exports and no prelude that changes meaning.
* Name resolution is a compiler stage (`nudo-hir`), not a runtime lookup. What a
  name refers to must be answerable without executing the program.

## Packages

A package is a directory with a manifest:

```text
my-package/
  nudo.toml
  nudo.lock        # produced by the package manager, not hand-written
  src/
    main.nudo
```

### `nudo.toml`

The manifest schema is **provisional and unimplemented**. The smallest proposed
form:

```toml
[package]
name = "my-package"
version = "0.1.0"
description = "One line about the package."
license = "MIT OR Apache-2.0"
repository = "https://github.com/example/my-package"
authors = ["Someone"]

[dependencies]
```

Rules for the schema, which matter more than the fields:

* **Unknown fields are an error.** A manifest that ignores what it does not
  understand turns a typo into a build that appears to work.
* **A dependency is not a permission.** Declaring a dependency grants nothing;
  capabilities are declared separately and explicitly. This is deliberate —
  dependency graphs are how authority leaks in a system that does not intend it
  to.
* The manifest may not contain credentials. Secrets are referenced, never
  embedded.

### `nudo.lock`

A lockfile pins resolved versions and their content hashes so that a build is
reproducible. It is generated, committed, and never edited by hand. It does not
exist yet, because nothing resolves dependencies yet, and a checked-in lockfile
that no tool can produce would be a lie about the project's state.

## Versioning

Packages follow [Semantic Versioning](https://semver.org/). The toolchain's own
version and the language's version are the same number today; whether they
diverge is an open question.

## Compatibility

* Before 1.0, a package's dependencies may break it on any update. This is
  documented rather than softened.
* After 1.0, the intent is editions, which is a design that has to be written
  against a language that runs.

## What this chapter may not do

* Describe a resolver that does not exist as though it did.
* Define a manifest field whose meaning is not enforced, because a field that
  means nothing is a field people will rely on.
