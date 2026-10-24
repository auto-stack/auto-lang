# PLAN-818 verify script — scoped gates for the ac-mir-native workspace.
# Usage: powershell -File scripts/verify-ac-mir-native-818.ps1 -Scope <scope>
#   first-case | admission | semantics | cache | resources | autodown |
#   turnaround | acceptance | closeout
# The turnaround scope is a SERIAL measurement scope (nextest
# mn-turnaround group); the resources scope is the heavy tier (~7 min).
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("first-case", "admission", "semantics", "cache", "resources", "autodown", "turnaround", "acceptance", "closeout")]
    [string]$Scope,
    [int]$Samples = 30,
    [double]$GateMs = 1000.0
)

$ErrorActionPreference = "Continue"  # cargo writes progress to stderr; exit codes are the gate
$repo = Split-Path -Parent $PSScriptRoot
$ws = Join-Path $repo "experimental/ac-mir-native"
$reportDir = Join-Path $repo "docs/reports/plan818"
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null

if ($env:SCCACHE_SERVER_PORT -eq $null) { $env:SCCACHE_SERVER_PORT = "28101" }

function Run-Workspace-Tests([string]$TestBin, [string]$Label) {
    Write-Host "== $Label (ac-mir-native --test $TestBin)"
    Push-Location $ws
    & cargo test --test $TestBin 2>&1 | Tee-Object -Variable out | Write-Host
    $code = $LASTEXITCODE
    Pop-Location
    if ($code -ne 0) {
        Write-Host "FAILED: $Label"
        exit 1
    }
    $out | Out-File (Join-Path $reportDir "$Label.log") -Encoding utf8
}

function Run-Runner-Tests([string]$Filter, [string]$Label) {
    Write-Host "== $Label (main workspace nextest filter: $Filter)"
    Push-Location $repo
    # `cargo t` carries the workspace alias's features/config; -E passes
    # the nextest filter expression through.
    & cargo t -E $Filter 2>&1 | Tee-Object -Variable out | Write-Host
    $code = $LASTEXITCODE
    Pop-Location
    if ($code -ne 0) {
        Write-Host "FAILED: $Label"
        exit 1
    }
    $out | Out-File (Join-Path $reportDir "$Label.log") -Encoding utf8
}

switch ($Scope) {
    "first-case" {
        Run-Workspace-Tests "first_case" "first-case"
    }
    "admission" {
        # Codegen-door admission negatives (mn.unsupported.* with codegen=0)
        # live in the codegen tier.
        Run-Workspace-Tests "codegen_tier" "admission"
    }
    "semantics" {
        Run-Workspace-Tests "codegen_tier" "semantics-codegen"
        Run-Workspace-Tests "parity_tier" "semantics-parity"
    }
    "cache" {
        Run-Workspace-Tests "cache_tier" "cache-units"
        Run-Workspace-Tests "transaction_tier" "cache-transactions"
    }
    "resources" {
        Run-Workspace-Tests "resources_tier" "resources-scale"
    }
    "autodown" {
        # The formal runner faces: corpus + negative oracle + decode
        # negatives (T-02) and the full adapter suite (T-06).
        Run-Runner-Tests 'test(/ac_mn_probe_corpus|ac_mn_probe_negative|ac_mn_probe_decode|mir_native_adapter/)' "autodown"
    }
    "turnaround" {
        # Serial wall-clock gates (nextest mn-turnaround group) + the
        # driver-tier scenarios.
        Run-Runner-Tests "test(mir_native_turnaround)" "turnaround-formal"
        Run-Workspace-Tests "turnaround_tier" "turnaround-driver"
    }
    "acceptance" {
        # Wall-clock gates FIRST (quiet machine): the ~10-minute resources
        # tier's IO aftermath (AV queue / cache writeback) inflates the
        # turnaround tails by 3-12x if the gates run right after it
        # (measured). Heavy tiers go last.
        Run-Workspace-Tests "first_case" "first-case"
        Run-Runner-Tests "test(mir_native_turnaround)" "turnaround-formal"
        Run-Workspace-Tests "codegen_tier" "semantics-codegen"
        Run-Workspace-Tests "parity_tier" "semantics-parity"
        Run-Workspace-Tests "cache_tier" "cache-units"
        Run-Workspace-Tests "transaction_tier" "cache-transactions"
        Run-Runner-Tests 'test(/ac_mn_probe_corpus|ac_mn_probe_negative|ac_mn_probe_decode|mir_native_adapter/)' "autodown"
        Run-Workspace-Tests "turnaround_tier" "turnaround-driver"
        Run-Workspace-Tests "incremental_tier" "incremental-d828"
        Run-Workspace-Tests "resources_tier" "resources-scale"
    }
    "closeout" {
        # The full frozen face, replayed on the frozen merge commit
        # (same ordering rationale as acceptance).
        Run-Workspace-Tests "first_case" "first-case"
        Run-Runner-Tests "test(mir_native_turnaround)" "turnaround-formal"
        Run-Workspace-Tests "codegen_tier" "semantics-codegen"
        Run-Workspace-Tests "parity_tier" "semantics-parity"
        Run-Workspace-Tests "cache_tier" "cache-units"
        Run-Workspace-Tests "transaction_tier" "cache-transactions"
        Run-Runner-Tests 'test(/ac_mn_probe_corpus|ac_mn_probe_negative|ac_mn_probe_decode|mir_native_adapter/)' "autodown"
        Run-Workspace-Tests "turnaround_tier" "turnaround-driver"
        Run-Workspace-Tests "incremental_tier" "incremental-d828"
        Run-Workspace-Tests "resources_tier" "resources-scale"
    }
}
Write-Host "OK: $Scope"
