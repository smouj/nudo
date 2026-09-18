# Installing the toolchain

NUDO is pre-alpha. What you install today is a compiler front end that reads
`.nudo` files, lexes them, parses them and reports lexical and syntax
diagnostics. Nothing else works yet — see
[`../../ROADMAP.md`](../../ROADMAP.md).

## Requirements

* [Rust](https://rustup.rs), stable channel. The declared MSRV is in
  [`../../Cargo.toml`](../../Cargo.toml).
* Git.
* No third-party Rust dependencies are needed: the workspace has none.

## Build from source

```sh
git clone https://github.com/smouj/nudo
cd nudo

# Linux, macOS, WSL
scripts/bootstrap.sh

# Windows
scripts\bootstrap.ps1
```

The bootstrap script installs the toolchain user-locally through rustup when it
is missing, adds `rustfmt` and `clippy`, and builds the workspace. Nothing is
installed system-wide and no elevation is requested.

Then either run the CLI through cargo:

```sh
cargo run --package nudo-cli -- --version
cargo run --package nudo-cli -- check examples/00-hello-world/main.nudo
```

or put it on your `PATH`:

```sh
cargo build --release --package nudo-cli
export PATH="$PWD/target/release:$PATH"    # Windows: target\release
nudo --version
```

## Verifying the installation

```sh
nudo --version        # nudo 0.0.1 (pre-alpha)
nudo --help           # which commands work, and which are planned
```

`nudo --help` is the honest inventory: it lists the one implemented command and
the twelve that are planned.

## Running the full check

If you are contributing, or you want to know whether your checkout is sound:

```sh
scripts/check.sh        # Linux, macOS, WSL
scripts\check.ps1       # Windows
```

That runs formatting, lints, build, tests, the conformance corpus and the
documentation checks — the same pipeline as CI.

## There are no releases to download yet

There is no installer, no package on crates.io, and no prebuilt binary. Release
workflows exist and are prepared
([`../../.github/workflows/release.yml`](../../.github/workflows/release.yml)),
but publishing a pre-alpha toolchain that cannot compile a program would be a
promise the project cannot keep. Build from source, or wait.
