# verify-ac-artifact-probe.ps1 — PLAN-783 acceptance / closeout aggregator.
#
# Scopes:
#   -Scope acceptance (default)
#       Full V-01..V-12 matrix at work-scope loads: every test family must
#       run and pass; a missing family, a skipped required case, or any
#       sub-step failure fails the WHOLE run (plan §6: missing/skipped/sub-step
#       failure must fail as a whole).
#   -Scope closeout
#       Acceptance plus the merge-owned full loads: the 512-iteration churn
#       and the ≥30-sample perf report (generated, then contract-validated).
#
# No root-workspace cargo targets are invoked (probe-only change; AGENTS
# Category A).

param(
    [ValidateSet("acceptance", "closeout")]
    [string]$Scope = "acceptance",
    # Extra args forwarded to cargo test (e.g. --offline).
    [string[]]$CargoArgs = @("--offline"),
    # Skip cargo --release? Default release: layout/perf loads are sized
    # for the release build.
    [switch]$DebugProfile
)

$ErrorActionPreference = "Continue"
# scripts/ lives at the repo root: probe = <repo>/experimental/ac-artifact-probe.
$repo = Split-Path -Parent $PSScriptRoot
$probe = Join-Path $repo "experimental\ac-artifact-probe"
$manifest = Join-Path $probe "Cargo.toml"
if (-not (Test-Path $manifest)) { Write-Error "probe manifest not found at $manifest"; exit 2 }

$profile = if ($DebugProfile) { "" } else { "--release" }

# V-family -> test target filter (plan §6 table). All must run & pass.
$families = [ordered]@{
    "lib-unit"     = "--lib"
    "V-01 identity"     = "--test identity_"
    "V-02 admission"    = "--test admission_"
    "V-03 layout"       = "--test layout_"
    "V-04 native-resume"= "--test native_resume_"
    "V-05 invalidation" = "--test invalidation_"
    "V-06 reachability" = "--test reachability_"
    "V-07 concurrency"  = "--test concurrency_"
    "V-08 recovery"     = "--test recovery_"
    "V-09 gc-budget"    = "--test gc_budget_"
    "V-10 store-boundary" = "--test store_boundary_"
    "V-11 perf"         = "--test perf_"
    "V-12 dep-audit"    = "--test dep_audit_"
}

$failures = @()
$ranAny = $false

foreach ($entry in $families.GetEnumerator()) {
    $name = $entry.Key
    $target = $entry.Value
    Write-Host "== $name ($target)" -ForegroundColor Cyan
    $ranAny = $true
    # Through cmd.exe: PS 5.1 turns native stderr (cargo warnings) into
    # terminating errors when merged inside PowerShell.
    & cmd /c "cargo test --manifest-path `"$manifest`" $CargoArgs $profile $target 2>&1" | Tee-Object -Variable out | Write-Host
    if ($LASTEXITCODE -ne 0) {
        $failures += "$name"
    }
    # A family with zero executed tests is a missing case -> fail.
    $executed = ($out | Select-String "test result:").ToString()
    if ($executed -match "(\d+) passed; (\d+) failed") {
        if ([int]$Matches[1] -eq 0 -and [int]$Matches[2] -eq 0) {
            $failures += "$name (no tests executed — missing case)"
        }
    }
}

if ($Scope -eq "closeout") {
    Write-Host "== full 512-iteration churn (merge-owned)" -ForegroundColor Cyan
    & cmd /c "cargo test --manifest-path `"$manifest`" $CargoArgs $profile --test gc_budget_ gc_budget_churn_512 -- --ignored --nocapture 2>&1" | Tee-Object -Variable churnOut | Write-Host
    if ($LASTEXITCODE -ne 0) { $failures += "512-churn" }

    Write-Host "== full perf pass (>=30 samples/scenario)" -ForegroundColor Cyan
    $reportDir = Join-Path $repo "docs\reports\plan783"
    New-Item -ItemType Directory -Force -Path $reportDir | Out-Null
    $report = Join-Path $reportDir "perf.json"
    $node = Join-Path $probe "target\release\node.exe"
    if (-not (Test-Path $node)) { $node = Join-Path $probe "target\debug\node.exe" }
    & $node perf --out $report --samples 30 --root "$env:TEMP\ac783-perf-closeout" --fixture p04-while-retest.at 2>&1 | Write-Host
    if ($LASTEXITCODE -ne 0) { $failures += "perf-generation" }
    $env:AC_PROBE_PERF_REPORT = $report
    $env:AC_PROBE_PERF_MIN_SAMPLES = "30"
    & cmd /c "cargo test --manifest-path `"$manifest`" $CargoArgs $profile --test perf_ perf_full_report_contract 2>&1" | Tee-Object -Variable perfOut | Write-Host
    if ($LASTEXITCODE -ne 0) { $failures += "perf-contract" }
    Remove-Item Env:AC_PROBE_PERF_REPORT -ErrorAction SilentlyContinue
    Remove-Item Env:AC_PROBE_PERF_MIN_SAMPLES -ErrorAction SilentlyContinue
}

Write-Host ""
if ($failures.Count -gt 0 -or -not $ranAny) {
    Write-Host "VERIFY-AC-ARTIFACT-PROBE: FAIL" -ForegroundColor Red
    $failures | ForEach-Object { Write-Host "  failed: $_" -ForegroundColor Red }
    exit 1
}
Write-Host "VERIFY-AC-ARTIFACT-PROBE [$Scope]: PASS (all families ran and passed)" -ForegroundColor Green
exit 0
