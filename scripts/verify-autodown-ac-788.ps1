# PLAN-788 verification gate (T-08).
#   -Scope acceptance : every V-matrix family + corpus + turnaround, refuses skips
#   -Scope closeout   : merge-owned full surfaces on the frozen commit (t, tv/tt
#                       per touched faces, worker/probe/783/787 independent suites)
# Usage: powershell -NoProfile -File scripts\verify-autodown-ac-788.ps1 [-Scope acceptance|closeout]
param(
    [ValidateSet("acceptance", "closeout")]
    [string]$Scope = "acceptance"
)

$ErrorActionPreference = "Continue"
$env:RUSTC_WRAPPER = ""
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

# Counted expectations: each family must actually RUN its tests (0-selected
# families abort the gate — a green banner over nothing is refused).
$fails = 0

function Run-Cargo([string[]]$argv) {
    Write-Host ("== cargo " + ($argv -join " ")) -ForegroundColor Cyan
    $text = (& cargo @argv 2>&1 | Out-String)
    if ($LASTEXITCODE -ne 0) {
        $script:fails++
        Write-Host ("FAMILY FAIL exit=" + $LASTEXITCODE) -ForegroundColor Red
        Write-Host $text -ForegroundColor Red
        return
    }
    if ($text -notmatch "[1-9][0-9]* passed") {
        $script:fails++
        Write-Host "ZERO-TEST family refused (no tests actually ran):" $text -ForegroundColor Red
        return
    }
    Write-Host "   family ok" -ForegroundColor Green
}

if ($Scope -eq "acceptance") {
    # V-01/07/15 contract + stale face
    Run-Cargo @("test", "-p", "auto-test-case")
    # V-02/03/05/06 executor + worker protocol
    Run-Cargo @("test", "--manifest-path", "experimental/ac-core/Cargo.toml", "--test", "link_capture_")
    Run-Cargo @("test", "--manifest-path", "experimental/ac-case-worker/Cargo.toml", "--test", "feasibility_t01_", "--test", "protocol_t03_", "--test", "cache_t05_", "--test", "review_r1_", "--test", "syntax_t06_")
    # V-04/08/09/10 store faces (783 regressions + 788 R1 controls)
    Run-Cargo @("test", "--manifest-path", "experimental/ac-artifact-probe/Cargo.toml", "--release", "--test", "plan788_r1_", "--test", "native_resume_", "--test", "invalidation_", "--test", "identity_", "--test", "concurrency_", "--test", "store_boundary_")
    # V-11/13/14 runner integration: dispatch, corpus, ledger, v1 regression
    Run-Cargo @("test", "-p", "auto-lang", "--features", "test-cases", "--lib", "ac_case_tests")
    Run-Cargo @("test", "-p", "auto-lang", "--features", "test-cases", "--lib", "test_cases_tests")
    # V-12 turnaround + P95 hard gates (release, single instance)
    Run-Cargo @("test", "--manifest-path", "experimental/ac-case-worker/Cargo.toml", "--release", "--test", "turnaround_t08_", "--", "--nocapture")

    if ($fails -gt 0) { throw "acceptance: $fails family/ies failed or refused" }
    Write-Host "PLAN-788 acceptance: ALL PASS" -ForegroundColor Green
    exit 0
}

# closeout (merge-owned): frozen-commit full surfaces.
Run-Cargo @("test", "-p", "auto-lang", "--features", "test-cases")
Run-Cargo @("t")
Run-Cargo @("tv")
Run-Cargo @("tt")
Run-Cargo @("test", "--manifest-path", "experimental/ac-case-worker/Cargo.toml", "--release")
Run-Cargo @("test", "--manifest-path", "experimental/ac-artifact-probe/Cargo.toml", "--release")
Run-Cargo @("test", "--manifest-path", "experimental/ac-syntax/runtime/Cargo.toml", "--release")

if ($fails -gt 0) { throw "closeout: $fails surface(s) failed" }
Write-Host "PLAN-788 closeout: ALL PASS" -ForegroundColor Green
