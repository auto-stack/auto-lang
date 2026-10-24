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
    [ValidateSet("first-case", "semantics", "admission", "cache", "resources", "compatibility", "autodown", "inventory", "acceptance")]
    [string]$Scope
)

$ErrorActionPreference = "Continue"  # cargo writes progress to stderr; exit codes are the gate
$repo = Split-Path -Parent $PSScriptRoot
$probeWs = Join-Path $repo "experimental/ac-av-probe"
$reportDir = Join-Path $repo "docs/reports/plan827"
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null

if ($env:SCCACHE_SERVER_PORT -eq $null) { $env:SCCACHE_SERVER_PORT = "28101" }

function Assert-Selected([string]$Out, [string]$Label) {
    # nextest summary: "Summary [...] N tests run: ..."; libtest: "N passed".
    $selected = 0
    if ($Out -match '(\d+) tests run') { $selected = [int]$Matches[1] }
    elseif ($Out -match '(\d+) passed') { $selected = [int]$Matches[1] }
    if ($selected -le 0) {
        Write-Host "FAILED: $Label selected 0 tests (0 selected is never a pass)"
        exit 1
    }
    Write-Host "   selected: $selected"
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
    "resources" { Not-Wired $Scope "T-04/T-06" }
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
    "autodown" { Not-Wired $Scope "T-06" }
    "inventory" { Not-Wired $Scope "T-06" }
    "acceptance" { Not-Wired $Scope "T-06" }
}
Write-Host "PASS: scope '$Scope' green"
exit 0
