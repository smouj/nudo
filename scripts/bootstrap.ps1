# Prepares a Windows machine to build NUDO from source.
#
# Mirrors scripts/bootstrap.sh: install the Rust toolchain when missing, add
# the components the pipeline needs, then build the workspace.
#
# Usage:  powershell -ExecutionPolicy Bypass -File scripts/bootstrap.ps1

$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
        Write-Host "installing rustup (user-local, https://rustup.rs)"
        $init = Join-Path $env:TEMP "nudo-rustup-init.exe"
        Invoke-WebRequest -Uri "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile $init
        & $init -y --profile minimal --default-toolchain stable --no-modify-path
        Remove-Item $init -ErrorAction SilentlyContinue
        $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
    }
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "cargo is still unavailable; add %USERPROFILE%\.cargo\bin to PATH and re-run"
}

Write-Host "toolchain: $(cargo --version), $(rustc --version)"

if (Get-Command rustup -ErrorAction SilentlyContinue) {
    rustup component add rustfmt clippy
}

Write-Host "building the workspace"
cargo build --workspace

Write-Host ""
Write-Host "ready. Next:"
Write-Host "  cargo run --package nudo-cli -- check examples/00-hello-world/main.nudo"
Write-Host "  scripts\check.ps1"
