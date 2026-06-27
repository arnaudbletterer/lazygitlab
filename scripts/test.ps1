#!/usr/bin/env pwsh
# Mirrors the Makefile targets for Windows / PowerShell.
# Usage: pwsh scripts/test.ps1 <target>
# Targets: test | test-live-smoke | test-live-full | test-live-massive | seed | seed-massive | seed-reset | lint | format | check | ci | coverage | clean

param(
    [Parameter(Position = 0)]
    [ValidateSet("test", "test-live-smoke", "test-live-full", "test-live-massive", "seed", "seed-massive", "seed-reset", "lint", "format", "check", "ci", "coverage", "clean", "help")]
    [string]$Target = "help"
)

function Invoke-Checked {
    param([string[]]$Args)
    & $Args[0] @($Args[1..($Args.Length - 1)])
    if ($LASTEXITCODE -ne 0) { throw "Command failed: $($Args -join ' ')" }
}

switch ($Target) {
    "help" {
        Write-Host @"
lazygitlab test ground targets (PowerShell):
  test              - wiremock + unit (fast, hermetic)
  test-live-smoke   - quick live tests (--ignored live_smoke)
  test-live-full    - exhaustive live tests (--ignored live_full)
  test-live-massive - massive-MR diff test (requires --with-massive-mr seed)
  seed              - populate test ground (idempotent)
  seed-massive      - populate test ground + seed a 120-file MR
  seed-reset        - DELETE test ground project (DESTRUCTIVE)
  lint              - cargo clippy --all-targets -- -D warnings
  format            - cargo fmt --all
  check             - fmt --check + lint + test (pre-merge gate)
  ci                - check + test-live-smoke
  coverage          - cargo tarpaulin (Linux only)
  clean             - cargo clean + remove test_ground.json
"@
    }
    "test"              { Invoke-Checked @("cargo", "test") }
    "test-live-smoke"   { Invoke-Checked @("cargo", "test", "--", "--ignored", "live_smoke") }
    "test-live-full"    { Invoke-Checked @("cargo", "test", "--", "--ignored", "live_full") }
    "test-live-massive" {
        & python scripts/populate_test_ground.py --with-massive-mr
        if ($LASTEXITCODE -ne 0) { throw "Seed failed" }
        Invoke-Checked @("cargo", "test", "--test", "test_massive_mr_diffs", "--", "--ignored")
    }
    "seed"              { Invoke-Checked @("python", "scripts/populate_test_ground.py") }
    "seed-massive"      { Invoke-Checked @("python", "scripts/populate_test_ground.py", "--with-massive-mr") }
    "seed-reset"        { Invoke-Checked @("python", "scripts/populate_test_ground.py", "--reset") }
    "lint"              { Invoke-Checked @("cargo", "clippy", "--all-targets", "--", "-D", "warnings") }
    "format"            { Invoke-Checked @("cargo", "fmt", "--all") }
    "check" {
        & cargo fmt --all -- --check
        if ($LASTEXITCODE -ne 0) { throw "Format check failed" }
        & cargo clippy --all-targets -- -D warnings
        if ($LASTEXITCODE -ne 0) { throw "Clippy failed" }
        & cargo test
        if ($LASTEXITCODE -ne 0) { throw "Tests failed" }
    }
    "ci" {
        & $PSCommandPath check
        if ($LASTEXITCODE -ne 0) { throw "Check failed" }
        & $PSCommandPath test-live-smoke
        if ($LASTEXITCODE -ne 0) { throw "Live smoke failed" }
    }
    "coverage"          { Invoke-Checked @("cargo", "tarpaulin", "--out", "Html", "--output-dir", "target/coverage", "--timeout", "180") }
    "clean" {
        Invoke-Checked @("cargo", "clean")
        Remove-Item -Force -ErrorAction SilentlyContinue data/test_ground.json
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue target/coverage
    }
    default { throw "Unknown target: $Target" }
}
