# PLAN-865 verification gate for source closures.
#
# Scopes:
#   targeted   - closure frontend and MIR lowering contracts.
#   compat     - frozen PLAN-823 closures-run-v1 corpus replay.
#   closures   - source-to-session probes and real ACE M1 K01-K06 suite,
#                including restart/body-edit cache and 20-sample P90 gates.
#   acceptance - all three scopes above.
#
# Run: powershell -File scripts/verify-av-865.ps1 -Scope targeted
param(
    [ValidateSet("targeted", "compat", "closures", "acceptance")]
    [string]$Scope = "acceptance"
)

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo

function Run-CargoTest([string]$Label, [string[]]$CargoArgs, [int]$Min = 1) {
    Write-Host "== PLAN-865 $Label =="
    $previous = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $output = & cargo @CargoArgs 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    $ErrorActionPreference = $previous
    if ($exitCode -ne 0) {
        $tail = ($output -split "`r?`n" | Select-Object -Last 80) -join [Environment]::NewLine
        Write-Host $tail
        throw "$Label failed (exit $exitCode)"
    }
    $resultLine = ($output -split "`r?`n" | Where-Object { $_ -match "test result:" } | Select-Object -Last 1)
    if ($resultLine -notmatch "test result: ok\.\s+(\d+) passed") {
        throw "$Label emitted no successful test result"
    }
    $passed = [int]$Matches[1]
    if ($passed -lt $Min) { throw "$Label matched $passed tests (< $Min)" }
    if ($resultLine -match ";\s*[1-9]\d* ignored") { throw "$Label ignored a selected test" }
    Write-Host "   $passed passed"
}

function Run-Targeted {
    Run-CargoTest "source closure surface" @(
        "test", "--manifest-path", "experimental/ac-core/Cargo.toml",
        "--test", "closures_865_surface"
    ) 8
    Run-CargoTest "closure MIR lowering" @(
        "test", "--manifest-path", "experimental/auto-mir-lower/Cargo.toml",
        "--test", "closures_865_lowering"
    ) 2
    Run-CargoTest "parent-side source closure oracle" @(
        "test", "--manifest-path", "crates/auto-lang-core/Cargo.toml",
        "--features", "test-cases", "--lib", "closures_source_judge_tests"
    ) 1
}

function Run-Compat {
    Run-CargoTest "frozen closures-run-v1 corpus" @(
        "test", "--manifest-path", "crates/auto-lang/Cargo.toml",
        "--features", "test-cases", "--lib", "av_closures_823_corpus_green", "--", "--nocapture"
    ) 1
}

function Run-Closures {
    Run-CargoTest "source-to-session and capture regressions" @(
        "test", "--manifest-path", "experimental/ac-av-probe/probe/Cargo.toml",
        "--test", "av_closures_865_source"
    ) 5
    Run-CargoTest "ACE M1 K01-K06 file-backed source suite" @(
        "test", "--manifest-path", "crates/auto-lang/Cargo.toml",
        "--features", "test-cases", "--lib", "ace_m1_closures_865_source_suite_green", "--", "--nocapture"
    ) 1
}

switch ($Scope) {
    "targeted" { Run-Targeted }
    "compat" { Run-Compat }
    "closures" { Run-Closures }
    "acceptance" {
        Run-Targeted
        Run-Compat
        Run-Closures
    }
}

Write-Host "verify-av-865 [$Scope]: GREEN"
