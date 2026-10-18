# PLAN-796 unified verification entry (T-07 deliverable).
#
# Usage:
#   powershell -File scripts/verify-ac-semantics-796.ps1 -Scope acceptance
#   powershell -File scripts/verify-ac-semantics-796.ps1 -Scope closeout
#
# Scopes:
#   acceptance — the V-01..V-09 gate set over the sema workspace
#     (targeted suites + boundary audit + report gate). V-08 (formal
#     AutoDown execution) is HONESTLY BLOCKED until D-788 + T-06:
#     acceptance therefore FAILS by design while the blocker stands —
#     it must never be made green by skipping the check.
#   closeout  — merge-owned gate inventory (root cargo t, surface
#     tiers, affected 788 suite, tf due-check receipt). These run on
#     the frozen commit at merge time, NOT from this script's test
#     pass; the scope verifies their INVENTORY and receipts, and fails
#     until each has an explicit receipt.
#
# Environment note: sccache's default port can collide with the
# Hyper-V reserved range on this machine; a fallback port is set when
# unset (same workaround as the T-01 receipt).

param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('acceptance', 'closeout')]
    [string]$Scope
)

$ErrorActionPreference = 'Stop'
if (-not $env:SCCACHE_SERVER_PORT) { $env:SCCACHE_SERVER_PORT = '44001' }

$repoRoot = Split-Path -Parent $PSScriptRoot
$ws = Join-Path $repoRoot 'experimental/ac-semantics'
Set-Location $ws

$script:Results = [System.Collections.Generic.List[string]]::new()
$script:Failed = 0

function Add-Result([string]$Id, [string]$Status, [string]$Note) {
    $script:Results.Add(('{0,-6}  {1,-7}  {2}' -f $Id, $Status, $Note))
    if ($Status -ne 'PASS') { $script:Failed++ }
}

function Run-Suite([string]$Id, [string[]]$CargoArgs, [string]$Note, [string]$Cwd = $ws, [int]$MinPassed = 1) {
    Write-Host ">> cargo $($CargoArgs -join ' ') (in $Cwd)" -ForegroundColor DarkGray
    # run through cmd so native stderr never becomes a PS error record
    # (EAP=Stop + 2>&1 misfires on Windows PowerShell 5.1)
    $argline = ($CargoArgs | ForEach-Object { if ($_ -match '\s') { '"$_"' } else { $_ } }) -join ' '
    $out = (& cmd /c "cd /d `"$Cwd`" && cargo $argline 2>&1") -join "`n"
    $code = $LASTEXITCODE
    $summary = ($out -split "`n" | Select-String 'test result:') -join '; '
    # F-796-R4-11: a zero-selection run is NOT a pass - count the
    # tests that actually EXECUTED across every result line and bind a
    # per-row minimum.
    $executed = 0
    $joined = $out -split "`n" -join ' '
    foreach ($m in [regex]::Matches($joined, 'test result: [a-z]+\. (\d+) passed')) {
        $executed += [int]$m.Groups[1].Value
    }
    if ($code -eq 0 -and $summary -match '0 failed' -and $executed -ge $MinPassed) {
        Add-Result $Id 'PASS' "$Note ($executed executed; $summary)"
    }
    else {
        Add-Result $Id 'FAIL' "$Note :: executed=$executed (<$MinPassed bound) :: $summary"
    }
}

if ($Scope -eq 'acceptance') {
    # --- V-01 / AC-01: identity, recursive groups, matrix ------------
    Run-Suite 'V-01' @('test', '-p', 'ac-sema-contract') 'sema_identity_ + unit (contract)' $ws 14

    # --- V-02 / AC-02: real inference vs the old pipeline ------------
    Run-Suite 'V-02a' @('test', '-p', 'ac-sema-core') 'infer_core_ unit suite' $ws 13
    Run-Suite 'V-02b' @('test', '-p', 'ac-sema-probe', '--test', 'infer_core_parity') 'differential parity corpus' $ws 4

    # --- V-03 / AC-03: queries, deps, cache lifecycle ----------------
    Run-Suite 'V-03' @('test', '-p', 'ac-sema-probe', '--test', 'meta_query_cache_deps') 'meta_query_ dependency faces' $ws 7
    Run-Suite 'V-03b' @('test', '-p', 'ac-sema-probe', '--test', 'sema_cache_first_case', '--test', 'sema_cache_more', '--test', 'sema_cache_r4_repairs') 'sema_cache_ first case + lifecycle + R4 repairs' $ws 21

    # --- V-06 / AC-03: budgets and pressure ---------------------------
    Run-Suite 'V-06' @('test', '-p', 'ac-sema-probe', '--test', 'sema_budget_pressure') 'sema_budget_ 32MiB/deep/churn' $ws 5

    # --- V-04 / V-07 / AC-04: turnaround stages + the hard gate ------
    Run-Suite 'V-04' @('test', '-p', 'ac-sema-probe', '--test', 'sema_turnaround_scenarios') 'sema_turnaround_ six scenarios + gate' $ws 5

    # report gate: the committed report must exist and carry a PASS verdict
    $report = Join-Path $repoRoot 'docs/reports/plan796/turnaround.json'
    $ipcReport = Join-Path $repoRoot 'docs/reports/plan796/turnaround-ipc.json'
    if (-not (Test-Path $report)) {
        Add-Result 'V-07' 'FAIL' 'docs/reports/plan796/turnaround.json missing'
    }
    elseif (-not (Test-Path $ipcReport)) {
        Add-Result 'V-07' 'FAIL' 'turnaround-ipc.json missing (real adapter/IPC path)'
    }
    else {
        $json = Get-Content $report -Raw | ConvertFrom-Json
        $ipc = Get-Content $ipcReport -Raw | ConvertFrom-Json
        $gate = ($json.stats | Where-Object { $_.scenario -eq 'local-body-edit' })
        $libOk = ([long]$gate.p95_ns -lt 1000000000) -and ([int]$gate.samples -ge 30)
        $ipcOk = ($ipc.verdict -eq 'PASS') -and (@($ipc.samples).Count -ge 30)
        $genOk = $json.machine.generator -eq 'gen796-1mi/v1'
        if ($libOk -and $ipcOk -and $genOk) {
            Add-Result 'V-07' 'PASS' ("library P95 {0:N2}ms ({1} samples) + REAL IPC P95 {2:N2}ms ({3} samples); generator {4}" -f ($gate.p95_ns / 1e6), $gate.samples, [double]$ipc.p95_ms, @($ipc.samples).Count, $json.machine.generator)
        }
        else {
            Add-Result 'V-07' 'FAIL' "libOk=$libOk ipcOk=$ipcOk genOk=$genOk (need >=30 samples each, gate under 1000ms, frozen generator)"
        }
    }

    # --- V-05 / AC-01: CT/generic seam (model-only, schema-frozen) ----
    Run-Suite 'V-05' @('test', '-p', 'ac-sema-probe', '--test', 'meta_query_cache_deps', '--', 'meta_query_ct_model_seam_admission_and_cycles') 'ct seam admission/cycles (model)' $ws 1

    # --- V-09 / AC-06: boundary + parallel audit ----------------------
    Run-Suite 'V-09' @('test', '-p', 'ac-sema-probe', '--test', 'sema_boundary_audit') 'sema_boundary_ tree/diff/provenance' $ws 6

    # --- V-08 / AC-05: formal AutoDown execution (T-06 landed) --------
    # D-788 was satisfied (PLAN-788 merged, receipts on master); the
    # probe adapter corpus now EXECUTES through the unified runner:
    # real sema worker processes, real restart boundary, parent-side
    # judgement. The corpus+admission suite is the V-08 gate.
    Run-Suite 'V-08' @('test', '-p', 'auto-lang', '--features', 'test-cases', '--', 'probe_adapter') 'probe adapter through the unified runner (V-08)' $repoRoot 11
    # the REAL adapter/IPC turnaround hard gate (F-796-R4-10)
    Run-Suite 'V-08c' @('test', '-p', 'auto-lang', '--features', 'test-cases', '--lib', '--', 'tests::probe_adapter_turnaround_ipc') 'real adapter/IPC path P95 gate (30 samples)' $repoRoot 1
    $caseFiles = Get-ChildItem (Join-Path $repoRoot 'crates/auto-lang/test/cases/ac-semantics') -Filter '*.case.ad'
    $pending = ($caseFiles | Where-Object { (Get-Content $_.FullName -Raw) -match 'not-executed' }).Count
    if ($pending -gt 0) {
        Add-Result 'V-08b' 'FAIL' ("{0}/{1} case files still carry not-executed markers" -f $pending, $caseFiles.Count)
    }
}
else {
    # --- closeout: merge-owned gate INVENTORY -------------------------
    Add-Result 'M-01' 'PENDING' 'root bare cargo t on the frozen commit (merge window, single instance)'
    Add-Result 'M-02' 'PENDING' 'surface tiers per final diff (tv/tt as applicable; T-06 protocol changes trigger the 788-affected suite)'
    Add-Result 'M-03' 'PENDING' 'independent sema full suite on the frozen commit'
    Add-Result 'M-04' 'PENDING' 'tf batch regression due-check (docs/plans/.last-batch-regression.json; /auto-plan:regress on main checkout)'
    Add-Result 'M-05' 'PENDING' 'spec deposit (SD-01..08), ledger refresh, archive, wt-guard cleanup receipts'
    $tf = Join-Path $repoRoot 'docs/plans/.last-batch-regression.json'
    if (Test-Path $tf) {
        Add-Result 'M-04b' 'PASS' ("tf receipt present: {0}" -f (Get-Content $tf -Raw).Trim())
    }
    else {
        Add-Result 'M-04b' 'FAIL' 'no tf due-check receipt file — batch regression not yet adjudicated'
    }
}

Write-Host ''
Write-Host ('==== PLAN-796 verify ({0}) ====' -f $Scope) -ForegroundColor Cyan
$script:Results | ForEach-Object {
    $color = if ($_ -match '\sPASS\s') { 'Green' } elseif ($_ -match '\sBLOCKED\s') { 'Yellow' } else { 'Red' }
    Write-Host $_ -ForegroundColor $color
}
Write-Host ''
if ($Scope -eq 'acceptance') {
    if ($script:Failed -gt 0) {
        Write-Host "acceptance: FAIL ($script:Failed item(s) not PASS — V-08 stays BLOCKED until D-788 + T-06 by design)" -ForegroundColor Red
        exit 1
    }
    Write-Host 'acceptance: PASS' -ForegroundColor Green
    exit 0
}
else {
    # closeout lists the merge-owned inventory; it cannot pass from a
    # test run — passing requires every PENDING replaced by a receipt
    Write-Host 'closeout: FAIL (merge-owned gates outstanding — run them in the merge window)' -ForegroundColor Red
    exit 1
}
