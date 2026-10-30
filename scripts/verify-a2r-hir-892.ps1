# PLAN-892 (A2X / Checked-HIR-to-Rust, AX-A) scoped verification entry.
# ASCII only for Windows PowerShell 5.1.
#
# Drives the FROZEN private probe entries of the experimental/ace
# workspace (true chain: real source -> checked frontend -> semantic view ->
# Rust provider -> real rustc -> run -> oracle). The formal/acceptance H-side
# registration (recipe routing in test_cases, .suite.ad AutoDown assets) is
# deliberately NOT wired yet: PLAN-888 still holds that write surface, so
# this script covers exactly the plan-owned released faces and refuses the
# scopes that need the window instead of faking them.
#
# Usage:
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-a2r-hir-892.ps1 -Scope <source|snapshot|certify|incremental|resources|formal|acceptance>
# Unknown or empty scope -> non-zero exit (plan section 6 requirement).

param(
    [Parameter(Mandatory = $true)]
    [string]$Scope
)
. (Join-Path $PSScriptRoot 'test-governance/ace-cargo.ps1')


$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$Workspace = Join-Path $RepoRoot 'experimental/ace'
$ReceiptsDir = Join-Path $Workspace 'target\a2x-verify-receipts\892'
New-Item -ItemType Directory -Force -Path $ReceiptsDir | Out-Null

function Invoke-Scope {
    param([string]$Name, [string[]]$TestEntries)
    Write-Host ("== scope {0}: {1}" -f $Name, ($TestEntries -join ', '))
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    foreach ($entry in $TestEntries) {
        Write-Host ("-- Invoke-AceCargo test -p ace-a2x-worker --test {0}" -f $entry)
        # NOTE: no 2>&1 pipeline capture - PS 5.1 turns native stderr
        # progress into ErrorRecords under Stop preference.
        & Invoke-AceCargo test --manifest-path (Join-Path $Workspace 'Cargo.toml') -p ace-a2x-worker --test $entry
        if ($LASTEXITCODE -ne 0) {
            Write-Host ("[FAIL] {0} ({1}s): test entry {2}" -f $Name, [math]::Round($sw.Elapsed.TotalSeconds, 1), $entry) -ForegroundColor Red
            exit 1
        }
    }
    $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
    $receipt = Join-Path $ReceiptsDir ("{0}-{1}.txt" -f $Name, $stamp)
    @{ scope = $Name; entries = $TestEntries; seconds = [math]::Round($sw.Elapsed.TotalSeconds, 1); status = 'PASS'; host = $env:COMPUTERNAME } |
        ConvertTo-Json | Set-Content -Path $receipt
    Write-Host ("[PASS] {0} ({1}s) receipt: {2}" -f $Name, [math]::Round($sw.Elapsed.TotalSeconds, 1), $receipt) -ForegroundColor Green
}

switch ($Scope) {
    'source' {
        # Full frozen 745/767 corpus + ACE-M1 B-cases through the true chain
        # (real rustc compile+run, frozen oracles; V-01..V-04 shapes).
        Invoke-Scope 'source' @('scalar_semantics')
    }
    'snapshot' {
        # File-set determinism + golden byte-compare (emit must be stable).
        Invoke-Scope 'snapshot' @('contract_certification')
    }
    'certify' {
        # Certification-shaped dry run: actual vs independent i32-form
        # reference vs closed-form oracle + strict cert draft validation.
        Invoke-Scope 'certify' @('contract_certification')
    }
    'incremental' {
        # Single-body credentials, candidate transactions, layered cache,
        # resident reuse, A-B-A history, restart re-admission.
        Invoke-Scope 'incremental' @('incremental_cache', 'store_cache')
    }
    'resources' {
        # 48MiB warm management budget at load256 scale + dependency
        # direction controls.
        Invoke-Scope 'resources' @('resource_budget', 'boundaries')
    }
    'acceptance' {
        # The full currently-available acceptance face (every frozen probe
        # entry). NOT the final V-01..14 contract: the H-side registration
        # and formal sampling window are still pending (PLAN-888 handoff).
        Invoke-Scope 'acceptance' @('hir_shape', 'scalar_semantics', 'incremental_cache', 'contract_certification', 'store_cache', 'resource_budget', 'boundaries')
    }
    'formal' {
        # Formal sampling (fresh/hit/restart 30-sample tiers, P90 gates) is
        # explicitly window-separated from 888/889 formal runs (plan T-06);
        # it also needs the registered daily channel. Controlled refusal.
        Write-Host '[REFUSED] formal: the formal sampling window is not released (plan T-06; window-separated from PLAN-888/889 and needs the H-side registration).' -ForegroundColor Yellow
        exit 2
    }
    default {
        Write-Host ("[REFUSED] unknown or empty scope '{0}' - expected one of: source, snapshot, certify, incremental, resources, formal, acceptance" -f $Scope) -ForegroundColor Red
        exit 3
    }
}
