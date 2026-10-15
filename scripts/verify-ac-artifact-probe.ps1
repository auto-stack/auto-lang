# verify-ac-artifact-probe.ps1 — PLAN-783 acceptance / closeout aggregator.
#
# Scopes:
#   -Scope acceptance (default)
#       Full V-01..V-12 matrix at work-scope loads. A family FAILS unless
#       its cargo summary is PRESENT, failed==0, passed >= the inventory
#       minimum, and ignored == the explicit allowlist (only gc_budget_'s
#       merge-owned 512 load). The COMMITTED perf report contract is
#       validated for real (>=30 samples, six unique scenarios, zero
#       semantic failures). Missing summaries or skipped cases can NOT
#       produce PASS (review R1 F-09).
#   -Scope closeout
#       Acceptance plus the merge-owned full loads: the 512-iteration
#       churn (frozen default watermarks) and a freshly regenerated
#       >=30-sample perf report.
#
# No root-workspace cargo targets are invoked (probe-only change; AGENTS
# Category A).

param(
    [ValidateSet("acceptance", "closeout")]
    [string]$Scope = "acceptance",
    [string[]]$CargoArgs = @("--offline"),
    [switch]$DebugProfile
)

$ErrorActionPreference = "Continue"

# scripts/ lives at the repo root: probe = <repo>/experimental/ac-artifact-probe.
$repo = Split-Path -Parent $PSScriptRoot
$probe = Join-Path $repo "experimental\ac-artifact-probe"
$manifest = Join-Path $probe "Cargo.toml"
if (-not (Test-Path $manifest)) { Write-Error "probe manifest not found at $manifest"; exit 2 }

$profile = if ($DebugProfile) { "" } else { "--release" }

# Family inventory (R1 F-09): minimum PASSING cases required, and the only
# allowed IGNORED counts (gc_budget_: the merge-owned 512 churn, run by
# closeout explicitly). Update the mins when ADDING tests; lowering a min
# requires a plan-level justification.
$families = [ordered]@{
    "lib-unit"           = @{ target = "--lib";                 minPassed = 14; allowedIgnored = 0 }
    "V-01 identity"      = @{ target = "--test identity_";      minPassed = 4;  allowedIgnored = 0 }
    "V-02 admission"     = @{ target = "--test admission_";     minPassed = 7;  allowedIgnored = 0 }
    "V-03 layout"        = @{ target = "--test layout_";        minPassed = 6;  allowedIgnored = 0 }
    "V-04 native-resume" = @{ target = "--test native_resume_"; minPassed = 3;  allowedIgnored = 0 }
    "V-05 invalidation"  = @{ target = "--test invalidation_";  minPassed = 5;  allowedIgnored = 0 }
    "V-06 reachability"  = @{ target = "--test reachability_";  minPassed = 4;  allowedIgnored = 0 }
    "V-07 concurrency"   = @{ target = "--test concurrency_";   minPassed = 3;  allowedIgnored = 0 }
    "V-08 recovery"      = @{ target = "--test recovery_";      minPassed = 4;  allowedIgnored = 0 }
    "V-09 gc-budget"     = @{ target = "--test gc_budget_";     minPassed = 4;  allowedIgnored = 1 }
    "V-10 store-boundary"= @{ target = "--test store_boundary_";minPassed = 5;  allowedIgnored = 0 }
    "V-11 perf"          = @{ target = "--test perf_";          minPassed = 2;  allowedIgnored = 0 }
    "V-12 dep-audit"     = @{ target = "--test dep_audit_";     minPassed = 5;  allowedIgnored = 0 }
    "R1 regressions"     = @{ target = "--test review_r1_";     minPassed = 12; allowedIgnored = 0 }
    "R2 regressions"     = @{ target = "--test review_r2_";     minPassed = 6;  allowedIgnored = 0 }
    "R3 regressions"     = @{ target = "--test review_r3_";     minPassed = 4;  allowedIgnored = 0 }
}

$failures = @()

function Invoke-CargoFamily {
    param([string]$Name, [string]$Target, [int]$MinPassed, [int]$AllowedIgnored)
    Write-Host "== $Name ($Target)" -ForegroundColor Cyan
    # Through cmd.exe: PS 5.1 turns native stderr (cargo warnings) into
    # terminating errors when merged inside PowerShell.
    $out = & cmd /c "cargo test --manifest-path `"$manifest`" $CargoArgs $profile $Target 2>&1"
    $out | Write-Host
    if ($LASTEXITCODE -ne 0) { $script:failures += "$Name (cargo exit $LASTEXITCODE)" }
    $summary = $out | Select-String "test result:" | Select-Object -First 1
    if (-not $summary) {
        $script:failures += "$Name (NO test summary — output missing/incomplete)"
        return
    }
    if ($summary -match "(\d+) passed; (\d+) failed; (\d+) ignored") {
        $passed = [int]$Matches[1]; $failed = [int]$Matches[2]; $ignored = [int]$Matches[3]
        if ($failed -ne 0) { $script:failures += "$Name ($failed failed)" }
        if ($passed -lt $MinPassed) {
            $script:failures += "$Name (passed $passed < required $MinPassed — missing cases)"
        }
        if ($ignored -ne $AllowedIgnored) {
            $script:failures += "$Name (ignored $ignored != allowed $AllowedIgnored)"
        }
    } else {
        $script:failures += "$Name (unparsable summary: $summary)"
    }
}

foreach ($entry in $families.GetEnumerator()) {
    Invoke-CargoFamily -Name $entry.Key -Target $entry.Value.target `
        -MinPassed $entry.Value.minPassed -AllowedIgnored $entry.Value.allowedIgnored
}

function Test-PerfReport {
    param([string]$Path, [int]$MinSamples, [string]$Tag)
    if (-not (Test-Path $Path)) { $script:failures += "$Tag (report missing at $Path)"; return }
    try {
        $report = Get-Content -Raw $Path | ConvertFrom-Json
    } catch {
        $script:failures += "$Tag (report not valid JSON: $_)"; return
    }
    $expected = @("fresh","same-process-hot","restart-disk-hot","cross-project","body-modified","gc-concurrent-query")
    $names = @($report.scenarios | ForEach-Object { $_.name })
    if ($names.Count -ne 6) { $script:failures += "$Tag (scenario count $($names.Count) != 6)" }
    if (($names | Select-Object -Unique).Count -ne $names.Count) {
        $script:failures += "$Tag (duplicated scenario identities)"
    }
    foreach ($e in $expected) {
        if ($names -notcontains $e) { $script:failures += "$Tag (missing scenario $e)" }
    }
    foreach ($s in $report.scenarios) {
        if ([int]$s.stats.n -lt $MinSamples) {
            $script:failures += "$Tag (scenario $($s.name): $($s.stats.n) samples < $MinSamples)"
        }
        if ([int]$s.semantic_failures -ne 0) {
            $script:failures += "$Tag (scenario $($s.name): semantic failures)"
        }
        if ([int]$s.ok -lt $MinSamples) {
            $script:failures += "$Tag (scenario $($s.name): ok $($s.ok) < $MinSamples)"
        }
    }
    if ([int]$report.samples_per_scenario -lt $MinSamples) {
        $script:failures += "$Tag (samples_per_scenario $($report.samples_per_scenario) < $MinSamples)"
    }
}

# Acceptance validates the COMMITTED report for real (R1 F-09: the
# contract check must never silently no-op).
$committedReport = Join-Path $repo "docs\reports\plan783\perf.json"
Write-Host "== committed perf report contract (>=30 samples/scenario)" -ForegroundColor Cyan
Test-PerfReport -Path $committedReport -MinSamples 30 -Tag "perf-report"

if ($Scope -eq "closeout") {
    # --- full 512 churn: identity + summary enforced, not just exit code
    # (R2-06: a filtered-out test or stubbed cargo exits 0 with zero runs).
    Write-Host "== full 512-iteration churn (merge-owned, frozen watermarks)" -ForegroundColor Cyan
    $out = & cmd /c "cargo test --manifest-path `"$manifest`" $CargoArgs $profile --test gc_budget_ gc_budget_churn_512 -- --ignored --nocapture 2>&1"
    $out | Write-Host
    if ($LASTEXITCODE -ne 0) { $failures += "512-churn (cargo exit $LASTEXITCODE)" }
    $churnSummary = $out | Select-String "test result:" | Select-Object -First 1
    if (-not $churnSummary) {
        $failures += "512-churn (NO test summary — zero cases executed)"
    } elseif ($churnSummary -match "(\d+) passed; (\d+) failed; (\d+) ignored") {
        if ([int]$Matches[1] -ne 1 -or [int]$Matches[2] -ne 0) {
            $failures += "512-churn (expected exactly 1 passed/0 failed, got $($Matches[1])/$($Matches[2]))"
        }
    } else {
        $failures += "512-churn (unparsable summary: $churnSummary)"
    }
    if (-not ($out | Select-String "gc_budget_churn_512_converges_between_watermarks \.\.\. ok")) {
        $failures += "512-churn (test identity line '. ok' absent — wrong or skipped case)"
    }

    # --- perf regeneration: NEW temp output, validate, THEN promote.
    # The frozen commit is bound via AC_PROBE_SOURCE_COMMIT (R2-06): a
    # stale/unbound report can never pass as new evidence, and a failed
    # regeneration leaves the old report intact but fails the gate.
    Write-Host "== regenerate full perf report (>=30 samples/scenario)" -ForegroundColor Cyan
    $node = Join-Path $probe "target\release\node.exe"
    if (-not (Test-Path $node)) { $node = Join-Path $probe "target\debug\node.exe" }
    $head = & git -C $repo rev-parse HEAD 2>$null
    if (-not $head) { $head = "unknown" }
    $tmpReport = Join-Path $env:TEMP ("ac783-perf-closeout-" + [guid]::NewGuid().ToString("N") + ".json")
    $env:AC_PROBE_SOURCE_COMMIT = "$head".Trim()
    & $node perf --out $tmpReport --samples 30 --root "$env:TEMP\ac783-perf-closeout" --fixture p04-while-retest.at 2>&1 | Write-Host
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path $tmpReport)) {
        $failures += "perf-generation (node exit $LASTEXITCODE, artifact present: $(Test-Path $tmpReport))"
    } else {
        Write-Host "== regenerated perf report contract (temp output)" -ForegroundColor Cyan
        # ATOMIC PROMOTION (R3-04): Test-PerfReport and the binding/overlap
        # checks only APPEND to $failures — an invalid report must never be
        # published over the old evidence. Snapshot the failure count and
        # promote ONLY when this validation added zero new failures.
        $failuresBefore = $failures.Count
        Test-PerfReport -Path $tmpReport -MinSamples 30 -Tag "perf-report-regen"
        # Binding: the regenerated report must carry the frozen commit.
        try {
            $regen = Get-Content -Raw $tmpReport | ConvertFrom-Json
            if ([string]$regen.source_commit -ne "$head".Trim()) {
                $failures += "perf-report-regen (source_commit '$($regen.source_commit)' != frozen '$head')"
            }
            $gc = $regen.scenarios | Where-Object { $_.name -eq "gc-concurrent-query" }
            if ([long]$gc.gc_deleted_during_queries -le 0) {
                $failures += "perf-report-regen (gc deletions did not overlap the timed queries)"
            }
        } catch {
            $failures += "perf-report-regen (post-run validation: $_)"
        }
        if ($failures.Count -eq $failuresBefore) {
            # Every artifact check succeeded — publish.
            Copy-Item $tmpReport $committedReport -Force
        } else {
            # Invalid new report: keep the OLD report bytes untouched; the
            # failures recorded above already fail the gate.
            Write-Host "perf regeneration INVALID — old report preserved, new evidence NOT published" -ForegroundColor Yellow
        }
    }
    Remove-Item Env:AC_PROBE_SOURCE_COMMIT -ErrorAction SilentlyContinue
    if (Test-Path $tmpReport) { Remove-Item $tmpReport -Force -ErrorAction SilentlyContinue }
}

Write-Host ""
if ($failures.Count -gt 0) {
    Write-Host "VERIFY-AC-ARTIFACT-PROBE [$Scope]: FAIL" -ForegroundColor Red
    $failures | Select-Object -Unique | ForEach-Object { Write-Host "  failed: $_" -ForegroundColor Red }
    exit 1
}
Write-Host "VERIFY-AC-ARTIFACT-PROBE [$Scope]: PASS (all families ran with required case counts)" -ForegroundColor Green
exit 0
