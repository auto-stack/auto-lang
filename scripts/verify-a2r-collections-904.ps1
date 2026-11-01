# PLAN-904 (AX-C: Checked HIR-to-Rust collections/variants) scoped
# verification entry. ASCII only for Windows PowerShell 5.1.
#
# Drives the collections channel's faces: the verifier-owned body facts
# (own/consume/exit/error-path schedules), the widened semantic view +
# fingerprint (list/variant nominal digest), the typed Rust provider with
# billing/poll/fault and LIFO cleanup, the credential binding (V-13), the
# resource probes (V-15), and the REGISTERED daily channel (recipe
# ac-a2r/hir-collections-emit-v1, suite a2r-hir-collections-904 with
# frozen cross-nonce goldens, judged parent-side by the fixed group
# case-a2r-hir-collections-904).
#
# Usage:
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-a2r-collections-904.ps1 -Scope <chain|probes|credential|snapshot|acceptance>
# Unknown or empty scope -> non-zero exit.

param(
    [Parameter(Mandatory = $true)]
    [string]$Scope
)
. (Join-Path $PSScriptRoot 'test-governance/ace-cargo.ps1')


$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$Workspace = Join-Path $RepoRoot 'experimental/ace'
$ReceiptsDir = Join-Path $Workspace 'target\a2x-verify-receipts\904'
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
    'chain' {
        # The minimal true chain (T-02): the five required first cases
        # through the new channel on REAL rustc + the typed-record-family
        # warm-fragment repair witness (T-03) + the store-family red
        # boundary record (T-06).
        Invoke-Scope 'chain' @('coll_904')
    }
    'probes' {
        # V-15: the typed runtime probes - billing table, budget
        # exhaustion, cooperative cancellation at the poll, fault-at-n
        # atomicity with live-counter fallback, deep-clone rollback.
        Invoke-Scope 'probes' @('v15')
    }
    'credential' {
        # V-13 (and the fragment binding): same owner/context with a
        # different body is a NAMED body-digest rejection; the legitimate
        # credential still admits.
        Invoke-Scope 'credential' @('coll_904_v13')
    }
    'snapshot' {
        # The registered daily channel: the frozen 20-case suite
        # (hash-bound ace-m1 sources, cross-nonce goldens, hand-written
        # references) judged parent-side by the fixed group (spawn-zero,
        # manifest re-hash, golden byte-compare, undeclared-extra
        # rejection).
        cargo t case-a2r-hir-collections-904
        if ($LASTEXITCODE -ne 0) { Write-Host '[FAIL] snapshot: case-a2r-hir-collections-904' -ForegroundColor Red; exit 1 }
        $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
        $receipt = Join-Path $ReceiptsDir ("snapshot-{0}.txt" -f $stamp)
        @{ scope = 'snapshot'; entries = @('case-a2r-hir-collections-904'); status = 'PASS'; host = $env:COMPUTERNAME } |
            ConvertTo-Json | Set-Content -Path $receipt
        Write-Host ("[PASS] snapshot receipt: {0}" -f $receipt) -ForegroundColor Green
    }
    'acceptance' {
        # Every currently-released collections face: the chain + probes +
        # credential scopes plus the registered daily channel.
        Invoke-Scope 'acceptance' @('coll_904', 'v15', 'coll_904_v13')
        cargo t case-a2r-hir-collections-904
        if ($LASTEXITCODE -ne 0) { Write-Host '[FAIL] acceptance: case group' -ForegroundColor Red; exit 1 }
    }
    default {
        Write-Host ("[REFUSED] unknown or empty scope '{0}' - expected one of: chain, probes, credential, snapshot, acceptance" -f $Scope) -ForegroundColor Red
        exit 1
    }
}
