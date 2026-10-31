# PLAN-902 (AX-B: Checked HIR-to-Rust record/text/text-record) scoped
# verification entry. ASCII only for Windows PowerShell 5.1.
#
# Drives the record-text channel's faces: the widened semantic view, the
# mode/resource-legalized Rust provider with verifier-planned cleanup, the
# AXS2/AXR2 layered session increment, the certification controls, and the
# REGISTERED daily channel (recipe ac-a2r/hir-record-text-emit-v1, suite
# a2r-hir-record-text-902 with frozen goldens, judged parent-side by the
# fixed group case-a2r-hir-record-text-902).
#
# Usage:
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-a2r-record-text-902.ps1 -Scope <modes|surface|incremental|cache|certify|snapshot|acceptance>
# Unknown or empty scope -> non-zero exit.
#
# The formal sampling window (four-state 30-sample tiers, P90 budgets) is
# T-08's explicit single-instance window - this script refuses it instead
# of faking a timing run.

param(
    [Parameter(Mandatory = $true)]
    [string]$Scope
)
. (Join-Path $PSScriptRoot 'test-governance/ace-cargo.ps1')


$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$Workspace = Join-Path $RepoRoot 'experimental/ace'
$ReceiptsDir = Join-Path $Workspace 'target\a2x-verify-receipts\902'
New-Item -ItemType Directory -Force -Path $ReceiptsDir | Out-Null

function Invoke-Scope {
    param([string]$Name, [string[]]$TestEntries)
    Write-Host ("== scope {0}: {1}" -f $Name, ($TestEntries -join ', '))
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    foreach ($entry in $TestEntries) {
        Write-Host ("-- Invoke-AceCargo test -p ace-a2x-worker --test conformance {0}" -f $entry)
        & Invoke-AceCargo test --manifest-path (Join-Path $Workspace 'Cargo.toml') -p ace-a2x-worker --test conformance $entry
        if ($LASTEXITCODE -ne 0) {
            Write-Host ("[FAIL] {0} ({1}s): filter {2}" -f $Name, [math]::Round($sw.Elapsed.TotalSeconds, 1), $entry) -ForegroundColor Red
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
    'modes' {
        # V-01/V-03: single-body credentials (record methods, text bodies
        # with their verifier resource plans), credential binding refusals,
        # Copy snapshots, multi-mut exclusivity, receiver-once order.
        Invoke-Scope 'modes' @('record_text_modes')
    }
    'surface' {
        # V-02/V-04/V-05: the full target surface - mut text views,
        # resource-record field cleanup and deep clone, Unicode/NUL edges,
        # borrowed-return provenance, branch/loop cleanup schedules.
        Invoke-Scope 'surface' @('record_text_t04')
    }
    'incremental' {
        # V-06/V-07: true function-level increment over AXS2/AXR2 - A-B-A
        # historical byte reuse, callee-edit isolation, declaration
        # invalidation, restart re-admission; plus the V-07 fingerprint
        # matrix (a2x-core conformance).
        Invoke-Scope 'incremental' @('store_rt_902')
        Write-Host "-- Invoke-AceCargo test -p ace-a2x-core --test conformance"
        & Invoke-AceCargo test --manifest-path (Join-Path $Workspace 'Cargo.toml') -p ace-a2x-core --test conformance
        if ($LASTEXITCODE -ne 0) { Write-Host '[FAIL] incremental: a2x-core conformance' -ForegroundColor Red; exit 1 }
    }
    'cache' {
        # V-08: read-only zero-write, quota preservation, the close
        # contract, family isolation from the scalar channel.
        Invoke-Scope 'cache' @('store_rt_902')
    }
    'certify' {
        # V-09/V-10: the explicit certify matrix - PASS leg with observed
        # toolchain binding, wrong oracle/reference/late-failure refusals,
        # cross-channel impersonation, protocol legs via the real worker.
        Invoke-Scope 'certify' @('cert_rt_902')
    }
    'snapshot' {
        # The registered daily channel: the frozen suite's four cases
        # (hash-bound ace-m1 sources, cross-nonce goldens) judged
        # parent-side by the fixed group (spawn-zero, manifest re-hash,
        # golden byte-compare, undeclared-extra rejection).
        cargo t case-a2r-hir-record-text-902
        if ($LASTEXITCODE -ne 0) { Write-Host '[FAIL] snapshot: case-a2r-hir-record-text-902' -ForegroundColor Red; exit 1 }
        $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
        $receipt = Join-Path $ReceiptsDir ("snapshot-{0}.txt" -f $stamp)
        @{ scope = 'snapshot'; entries = @('case-a2r-hir-record-text-902'); status = 'PASS'; host = $env:COMPUTERNAME } |
            ConvertTo-Json | Set-Content -Path $receipt
        Write-Host ("[PASS] snapshot receipt: {0}" -f $receipt) -ForegroundColor Green
    }
    'acceptance' {
        # Every currently-released record-text face (the full T-02..T-07
        # probe set + the registered daily channel). NOT the final V-01..12
        # contract: T-08 owns the formal sampling window.
        Invoke-Scope 'acceptance' @('record_text_902', 'record_text_modes', 'record_text_t04', 'store_rt_902', 'cert_rt_902')
        cargo t case-a2r-hir-record-text-902
        if ($LASTEXITCODE -ne 0) { Write-Host '[FAIL] acceptance: case group' -ForegroundColor Red; exit 1 }
    }
    'formal' {
        # T-08's explicit single-instance window (four-state 30-sample
        # tiers, P90 <= 2000ms, budget ledger) - never faked here.
        Write-Host '[REFUSED] formal: the formal sampling window is T-08-owned (single instance, explicit window).' -ForegroundColor Yellow
        exit 2
    }
    default {
        Write-Host ("[REFUSED] unknown or empty scope '{0}' - expected one of: modes, surface, incremental, cache, certify, snapshot, acceptance, formal" -f $Scope) -ForegroundColor Red
        exit 3
    }
}
