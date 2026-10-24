# PLAN-827 verify script — scoped gates for the av-source baseline.
# Usage: powershell -File scripts/verify-av-source-827.ps1 -Scope <scope>
#   first-case (T-03) | semantics | admission | cache | resources |
#   compatibility | autodown | inventory | acceptance
# Scopes land with their tasks (T-03..T-06); an unimplemented scope is a
# NON-ZERO exit with an explicit message, never green. Every implemented
# scope enforces selected>0 (0 selected / skipped required cases = red),
# and cargo exit codes are passed through.
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("first-case", "semantics", "admission", "cache", "resources", "compatibility", "autodown", "inventory", "incremental", "acceptance", "selftest", "fail-shim")]
    [string]$Scope
)

$ErrorActionPreference = "Continue"  # cargo writes progress to stderr; exit codes are the gate
$repo = Split-Path -Parent $PSScriptRoot
$probeWs = Join-Path $repo "experimental/ac-av-probe"
$reportDir = Join-Path $repo "docs/reports/plan827"
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null

if ($env:SCCACHE_SERVER_PORT -eq $null) { $env:SCCACHE_SERVER_PORT = "28101" }

function Test-Selected([string]$Out, [string]$Label) {
    # Returns $true iff the run output proves a NON-EMPTY selection with
    # ZERO skipped/ignored cases. nextest summary: "Summary [...] N tests
    # run: X passed, Y failed, Z skipped"; libtest: "test result: ok. N
    # passed; M failed; K ignored". A required case that is skipped or
    # ignored is NOT a pass (R1-F01).
    $selected = 0
    if ($Out -match '(\d+) tests run') { $selected = [int]$Matches[1] }
    elseif ($Out -match '(\d+) passed') { $selected = [int]$Matches[1] }
    if ($selected -le 0) {
        Write-Host "FAILED: $Label selected 0 tests (0 selected is never a pass)"
        return $false
    }
    $skipped = 0
    if ($Out -match '(\d+) skipped') { $skipped = [int]$Matches[1] }
    elseif ($Out -match '(\d+) ignored') { $skipped = [int]$Matches[1] }
    if ($skipped -gt 0) {
        Write-Host "FAILED: $Label has $skipped skipped/ignored cases (skips are never green)"
        return $false
    }
    Write-Host "   selected: $selected (0 skipped)"
    return $true
}

function Assert-Selected([string]$Out, [string]$Label) {
    if (!(Test-Selected $Out $Label)) { exit 1 }
}

function Run-Probe-Tests([string]$TestBin, [string]$Label) {
    Write-Host "== $Label (ac-av-probe --test $TestBin)"
    Push-Location $probeWs
    & cargo test -p auto-av-probe --test $TestBin 2>&1 | Tee-Object -Variable out | Write-Host
    $code = $LASTEXITCODE
    Pop-Location
    $out | Out-File (Join-Path $reportDir "$Label.log") -Encoding utf8
    if ($code -ne 0) {
        Write-Host "FAILED: $Label (exit $code)"
        exit 1
    }
    Assert-Selected ($out -join "`n") $Label
}

function Run-Runner-Tests([string]$Filter, [string]$Label) {
    Write-Host "== $Label (main workspace, filter: $Filter)"
    Push-Location $repo
    & cargo t $Filter 2>&1 | Tee-Object -Variable out | Write-Host
    $code = $LASTEXITCODE
    Pop-Location
    $out | Out-File (Join-Path $reportDir "$Label.log") -Encoding utf8
    if ($code -ne 0) {
        Write-Host "FAILED: $Label (exit $code)"
        exit 1
    }
    Assert-Selected ($out -join "`n") $Label
}

function Not-Wired([string]$Scope, [string]$Owner) {
    Write-Host "NOT WIRED: scope '$Scope' lands with $Owner (explicit non-green, never silent)"
    exit 1
}

function Invoke-Scope([string]$Scope) {
    # Run one scope as a REAL child script and return $true iff it exits 0.
    # The acceptance aggregate and the selftest failure-propagation control
    # share this one path (R1-F01). Child output is captured and shown via
    # Write-Host so the function's return value is ONLY the boolean.
    $out = & powershell -File $PSCommandPath -Scope $Scope 2>&1
    $code = $LASTEXITCODE
    $out | Write-Host
    if ($code -ne 0) {
        Write-Host "scope '$Scope' red (exit $code)"
        return $false
    }
    return $true
}

switch ($Scope) {
    "first-case" {
        # V-01 first case, full path: real AP-01 frozen bytes (identity
        # receipts) -> ephemeral -> store put/hit -> NEW-worker restore, plus
        # the formal-recipe corpus gate judged parent-side.
        Run-Probe-Tests "av_source_827_first_case" "first-case"
        Run-Runner-Tests "av_source_827" "first-case-corpus"
    }
    "semantics" {
        # V-02/03 (+ targeted V-04/05/09 axes): AP-22 argument-value change,
        # AP-23 precise trap + false-only control + native exit70 channel,
        # trace authorization, same-path/frozen-bytes identity, front-door
        # rejections with zero dispatch, budget/cancel axes.
        Run-Probe-Tests "av_source_827_semantics" "semantics"
        Run-Runner-Tests "av_source_827_corpus" "semantics-corpus"
    }
    "admission" {
        # V-04 runner-level admission negatives: invalid UTF-8 = infra (not
        # a trap), missing file = resolution error, path escapes rejected
        # at decode. Plus the unsupported front-door corpus faces.
        Run-Runner-Tests "av_source_827_admission" "admission"
        Run-Runner-Tests "av_source_827_corpus" "admission-corpus"
    }
    "resources" {
        # V-09 heavy tier (single instance, serial):
        # 1. gate-0 counter-control: bench-turnaround with --gate-ms 0 MUST
        #    fail (a threshold of 0 can never be green);
        # 2. the real P95 gate: 30 samples, gate 1000ms, raw JSONL kept;
        # 3. Line-A frontend attribution + Line-B residual via the frozen
        #    828 formal-session campaign (V-08 dual gate, >=30 revisions).
        # Small-file cold data is reported separately by the first-case log.
        $worker = Join-Path $probeWs "target/release/av_probe_worker.exe"
        # Every bench run keeps its OWN raw JSONL (failed runs included —
        # failure samples are never deleted or overwritten).
        $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
        Write-Host "== resources: bench gate-0 counter-control (must FAIL)"
        Push-Location $probeWs
        & $worker bench-turnaround --samples 3 --gate-ms 0 --out "$reportDir/bench-gate0-$stamp.jsonl" 2>&1 | Tee-Object -Variable g0 | Write-Host
        $g0code = $LASTEXITCODE
        Pop-Location
        if ($g0code -eq 0) {
            Write-Host "FAILED: gate-0 counter-control was green (a 0ms gate must be red)"
            exit 1
        }
        Write-Host "   gate-0 red as required (exit $g0code)"
        Write-Host "== resources: bench-turnaround 30 samples gate 1000ms"
        Push-Location $probeWs
        & $worker bench-turnaround --samples 30 --gate-ms 1000 --out "$reportDir/bench-30-$stamp.jsonl" 2>&1 | Tee-Object -Variable b30 | Write-Host
        $b30code = $LASTEXITCODE
        Pop-Location
        if ($b30code -ne 0) {
            Write-Host "FAILED: bench-turnaround gate (exit $b30code)"
            exit 1
        }
        Write-Host "== resources: 828 formal-session dual-gate campaign (V-08)"
        Push-Location $probeWs
        & cargo test -p auto-av-probe --test function_incremental_828_t08 t08_formal_campaign_v08_dual_gate 2>&1 | Tee-Object -Variable f828 | Write-Host
        $f828code = $LASTEXITCODE
        Pop-Location
        if ($f828code -ne 0) {
            Write-Host "FAILED: formal campaign (exit $f828code)"
            exit 1
        }
        Assert-Selected ($f828 -join "`n") "resources-formal-828"
    }
    "autodown" {
        # V-10: the formal recipe positives + every parent-side injection:
        # corpus green (with required population), negative oracle, worker
        # faults (exit9/extra-frame/wrong-hash/delay = infra).
        Run-Runner-Tests "av_source_827" "autodown"
    }
    "inventory" {
        # V-11: the G0 denominators re-derived from the real discovery
        # rules; the JSON summary is the inventory artifact.
        Run-Runner-Tests "av_source_827_inventory" "inventory"
    }
    "acceptance" {
        # V-01..12 ALL required sub-cases across EVERY implemented scope —
        # including resources (R1-F01: a resource gate failure must break
        # the aggregate) — selected>0 with 0 skips everywhere, plus the
        # SEM decision material (SEM-01..08) presence check.
        $sem = Join-Path $repo "docs/plans/attachments/827/sem-decisions.md"
        if (!(Test-Path $sem)) {
            Write-Host "FAILED: SEM decision material missing ($sem)"
            exit 1
        }
        $semRows = (Select-String -Path $sem -Pattern '^\| SEM-\d\d').Count
        if ($semRows -lt 8) {
            Write-Host "FAILED: SEM table must carry SEM-01..08 (saw $semRows rows)"
            exit 1
        }
        Write-Host "== acceptance: replaying every implemented scope (resources included)"
        foreach ($scope in @(
            "first-case", "semantics", "admission", "cache", "autodown",
            "inventory", "compatibility", "incremental", "resources"
        )) {
            if (!(Invoke-Scope $scope)) {
                Write-Host "FAILED: acceptance — scope '$scope' red"
                exit 1
            }
        }
        Write-Host "PASS: acceptance (all scopes incl. resources + SEM material)"
    }
    "selftest" {
        # R1-F01 repair evidence: negative controls replayed against the
        # REAL validator/aggregator pieces. Each bad shape must be RED.
        $fails = 0
        # 1. valid nextest summary, zero skipped -> green.
        if (!(Test-Selected "Summary [ 1.2s] 5 tests run: 5 passed, 0 failed, 0 skipped" "valid")) { $fails++ }
        # 2. nextest with skipped -> red.
        if (Test-Selected "Summary [ 1.2s] 5 tests run: 4 passed, 0 failed, 1 skipped" "skipped") { Write-Host "CONTROL FAILED: skipped must be red"; $fails++ }
        # 3. libtest with ignored -> red.
        if (Test-Selected "test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured" "ignored") { Write-Host "CONTROL FAILED: ignored must be red"; $fails++ }
        # 4. missing/zero selection -> red.
        if (Test-Selected "Summary [ 0.1s] 0 tests run: 0 passed" "missing") { Write-Host "CONTROL FAILED: zero-selected must be red"; $fails++ }
        if (Test-Selected "build finished, no summary matched" "no-summary") { Write-Host "CONTROL FAILED: no-summary must be red"; $fails++ }
        # 5. a REAL failing child scope propagated through the same
        #    Invoke-Scope path acceptance uses -> red.
        if (Invoke-Scope "fail-shim") { Write-Host "CONTROL FAILED: failing scope must be red"; $fails++ }
        if ($fails -gt 0) {
            Write-Host "FAILED: selftest ($fails red-path controls mis-behaved)"
            exit 1
        }
        Write-Host "PASS: selftest (validator + aggregator negative controls all red as required)"
    }
    "fail-shim" {
        # Selftest-only scope: a deliberately red child.
        Write-Host "fail-shim: intentional failure"
        exit 5
    }
    "incremental" {
        # V-13 (T-09): the 828 session reuse over this plan's source entry.
        Run-Probe-Tests "av_source_827_incremental" "incremental"
    }
    "cache" {
        # V-07/V-08: overlay misses only the changed unit; caps re-key;
        # corrupt same-key republish conflicts (good survives); cross-
        # project content-addressed reuse; position shift = new identity.
        # Formal face: the session-overlay corpus case (new worker per
        # check, shared warm store, parent-side judgement).
        Run-Probe-Tests "av_source_827_cache" "cache"
        Run-Runner-Tests "av_source_827_corpus" "cache-corpus"
    }
    "compatibility" {
        # V-11: the ORIGINAL consumers keep their support faces — the ac-av
        # corpus and the delivered 817/818 recipe gates run unchanged.
        Run-Runner-Tests "ac_av_probe" "compatibility-ac-av"
        Run-Runner-Tests "ac_cte_probe" "compatibility-ac-cte"
        Run-Runner-Tests "ac_mn_probe" "compatibility-ac-mn"
    }
}
Write-Host "PASS: scope '$Scope' green"
exit 0
