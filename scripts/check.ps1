# The canonical check for the NUDO repository, on Windows.
#
# Runs the same steps as scripts/check.sh. Any divergence between the two
# scripts is a bug in one of them.
#
# Usage:  powershell -ExecutionPolicy Bypass -File scripts/check.ps1

$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

$locked = @()
if ($env:NUDO_CI -eq "1") {
    $locked = @("--locked")
    Write-Host "NUDO_CI=1: cargo will refuse to change Cargo.lock"
}

function Invoke-Step {
    param([string]$Name, [scriptblock]$Body)
    Write-Host ""
    Write-Host "=== $Name ==="
    & $Body
    if ($LASTEXITCODE -ne 0) { throw "$Name failed" }
}

Invoke-Step "cargo fmt --all --check" { cargo fmt --all --check }
Invoke-Step "cargo clippy (warnings denied)" { cargo clippy --workspace --all-targets --all-features @locked -- -D warnings }
Invoke-Step "cargo build --workspace --all-targets" { cargo build --workspace --all-targets @locked }
Invoke-Step "cargo test --workspace" { cargo test --workspace @locked }
Invoke-Step "conformance corpus" { cargo test --package nudo-lexer --test conformance }
Invoke-Step "rustdoc (broken links denied)" {
    $env:RUSTDOCFLAGS = "-D warnings"
    cargo doc --workspace --no-deps --quiet @locked
}
Invoke-Step "documentation checks" { python scripts/check-docs.py }
Invoke-Step "documentation checker tests" { python scripts/test-check-docs.py }
Invoke-Step "workflow policy" { python scripts/check-workflows.py }
Invoke-Step "documented console output" { python scripts/check-console.py }
Invoke-Step "grammar" { python scripts/check-grammar.py }

Write-Host ""
Write-Host "all checks passed"
