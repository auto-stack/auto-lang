# PLAN-796 unified verification entry (T-07 deliverable).
#
# Usage:
#   powershell -File scripts/verify-ac-semantics-796.ps1 -Scope acceptance
#   powershell -File scripts/verify-ac-semantics-796.ps1 -Scope closeout
#   powershell -File scripts/verify-ac-semantics-796.ps1 -Scope selftest
#
# Scopes:
#   acceptance — the V-01..V-09 gate set over the sema workspace
#     (targeted suites + boundary audit + report gates). V-08 (formal
#     AutoDown execution) is HONESTLY BLOCKED until D-788 + T-06:
#     acceptance therefore FAILS by design while the blocker stands —
#     it must never be made green by skipping the check.
#   closeout  — merge-owned gate inventory (root cargo t, surface
#     tiers, affected 788 suite, tf due-check receipt). These run on
#     the frozen commit at merge time, NOT from this script's test
#     pass; the scope verifies their INVENTORY and receipts, and fails
#     until each has an explicit receipt.
#   selftest  — F-796-R6-08 negative controls for the acceptance helper
#     itself: zero selection, missing REQUIRED test identity, missing
#     target and a real red must all FAIL; a genuine green control must
#     PASS. F-796-R10-02 adds the V-07 raw-report admission controls:
#     missing/null/zero/negative edit-session times (all or one sample)
#     must FAIL the report gate; a legal report and a slow (1001ms)
#     report are the positive/high controls. Exit 0 only when every
#     control behaves as specified.
#
# Environment note: sccache's default port can collide with the
# Hyper-V reserved range on this machine; a fallback port is set when
# unset (same workaround as the T-01 receipt).

param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('acceptance', 'closeout', 'selftest')]
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

# PURE result computation (no console/result side effects) so the
# selftest scope can exercise the exact acceptance logic.
function Get-SuiteOutcome([int]$Code, [string]$Out, [int]$MinPassed, [string[]]$Required) {
    $summary = ($Out -split "`n" | Select-String 'test result:') -join '; '
    $executed = 0
    foreach ($m in [regex]::Matches($Out, 'test result: [a-z]+\. (\d+) passed')) {
        $executed += [int]$m.Groups[1].Value
    }
    $okNames = @([regex]::Matches($Out, '(?m)^test ([A-Za-z0-9_:]+) \.\.\. ok') |
        ForEach-Object { $_.Groups[1].Value })
    # required identity: exact, or the module-qualified suffix (unit
    # tests report `module::tests::name`, integration tests report the
    # bare name) — never a mere count
    $missing = @($Required | Where-Object {
            $req = $_
            -not ($okNames | Where-Object { $_ -eq $req -or $_.EndsWith("::$req") })
        })
    $ok = ($Code -eq 0) -and ($summary -match '0 failed') -and ($executed -ge $MinPassed) -and ($missing.Count -eq 0)
    [pscustomobject]@{
        Ok       = $ok
        Executed = $executed
        Missing  = $missing
        Summary  = $summary
    }
}

function Run-Suite([string]$Id, [string[]]$CargoArgs, [string]$Note, [string]$Cwd = $ws, [int]$MinPassed = 1, [string[]]$Required = @()) {
    Write-Host ">> cargo $($CargoArgs -join ' ') (in $Cwd)" -ForegroundColor DarkGray
    # run through cmd so native stderr never becomes a PS error record
    # (EAP=Stop + 2>&1 misfires on Windows PowerShell 5.1)
    $argline = ($CargoArgs | ForEach-Object { if ($_ -match '\s') { '"$_"' } else { $_ } }) -join ' '
    $out = (& cmd /c "cd /d `"$Cwd`" && cargo $argline 2>&1") -join "`n"
    $code = $LASTEXITCODE
    $r = Get-SuiteOutcome $code $out $MinPassed $Required
    if ($r.Ok) {
        Add-Result $Id 'PASS' "$Note ($($r.Executed) executed; $($r.Summary))"
    }
    else {
        $why = if ($r.Missing.Count -gt 0) { "missing required: $($r.Missing -join ', ')" } else { "executed=$($r.Executed) (<$MinPassed bound)" }
        Add-Result $Id 'FAIL' "$Note :: $why :: $($r.Summary)"
    }
}

function Run-Command([string]$Id, [string[]]$CargoArgs, [string]$Note, [string]$Cwd = $ws) {
    Write-Host ">> cargo $($CargoArgs -join ' ') (in $Cwd)" -ForegroundColor DarkGray
    $argline = ($CargoArgs | ForEach-Object { if ($_ -match '\s') { '"$_"' } else { $_ } }) -join ' '
    $out = (& cmd /c "cd /d `"$Cwd`" && cargo $argline 2>&1") -join "`n"
    $code = $LASTEXITCODE
    if ($code -eq 0) {
        Add-Result $Id 'PASS' "$Note (exit 0)"
    }
    else {
        $tail = ($out -split "`n" | Select-Object -Last 8) -join ' | '
        Add-Result $Id 'FAIL' "$Note (exit $code) :: $tail"
    }
}

function Get-P95Ms([double[]]$Values) {
    if ($Values.Count -eq 0) { return [double]::MaxValue }
    $sorted = @($Values | Sort-Object)
    $idx = [math]::Round(($sorted.Count - 1) * 0.95)
    return [double]$sorted[[int]$idx]
}

function Test-HasStages($Stages) {
    $names = @($Stages | ForEach-Object { $_[0] })
    foreach ($want in @('request', 'cache-read-validate', 'infer', 'publish')) {
        if ($names -notcontains $want) { return $false }
    }
    return $true
}

# F-796-R10-02: raw per-sample admission for report time fields. A
# missing field, null, non-numeric, non-finite or NON-POSITIVE value is
# a gate failure on that sample — it must never default to 0ms (which
# would silently speed up the percentiles). Returns the admitted times
# (already divided by $Divisor) and the rejection reasons.
function Get-AdmittedTimes($Samples, [string]$Field, [double]$Divisor) {
    $items = @($Samples)
    $times = [System.Collections.Generic.List[double]]::new()
    $bad = [System.Collections.Generic.List[string]]::new()
    for ($i = 0; $i -lt $items.Count; $i++) {
        $s = $items[$i]
        $prop = $s.PSObject.Properties[$Field]
        if ($null -eq $prop) { $bad.Add("sample ${i}: $Field missing"); continue }
        $raw = $prop.Value
        if ($null -eq $raw) { $bad.Add("sample ${i}: $Field is null"); continue }
        $v = 0.0
        if (-not [double]::TryParse([string]$raw, [System.Globalization.NumberStyles]::Float, [System.Globalization.CultureInfo]::InvariantCulture, [ref]$v)) {
            $bad.Add("sample ${i}: $Field '$raw' is not numeric"); continue
        }
        if ([double]::IsNaN($v) -or [double]::IsInfinity($v)) { $bad.Add("sample ${i}: $Field '$raw' is not finite"); continue }
        if ($v -le 0) { $bad.Add("sample ${i}: $Field must be positive, got '$raw'"); continue }
        $times.Add($v / $Divisor)
    }
    [pscustomobject]@{ Times = @($times); Bad = @($bad) }
}

# F-796-R10-02: PURE recomputation of the V-07 IPC report gate over a
# parsed turnaround-ipc.json object — the exact acceptance logic, kept
# side-effect free so the selftest scope can exercise it on mutated
# reports.
function Get-IpcReportOutcome($Ipc) {
    $samples = @($Ipc.samples)
    $adm = Get-AdmittedTimes $samples 'edit_session_ms' 1.0
    $p95 = Get-P95Ms $adm.Times
    $allPass = @($samples | Where-Object { $_.status -ne 'Pass' }).Count -eq 0
    $stages = @($samples | Where-Object { -not (Test-HasStages $_.edit_query_stages) }).Count -eq 0
    $ok = ($adm.Bad.Count -eq 0) -and ($samples.Count -ge 30) -and ($p95 -lt 1000.0) -and $allPass -and $stages `
        -and ($Ipc.verdict -eq 'PASS') -and ($Ipc.generator -eq 'gen796-1mi/v1') `
        -and ([int]$Ipc.reference_bytes -ge 1048576) -and ($null -ne $Ipc.total_p95_ms)
    [pscustomobject]@{ Ok = $ok; P95 = $p95; N = $samples.Count; Bad = $adm.Bad; AllPass = $allPass; Stages = $stages }
}

# Selftest fixture for Get-IpcReportOutcome: 30 samples whose
# edit_session_ms values are numbers, or the markers 'NULL' (property
# present with a null value) / 'MISSING' (property absent).
function New-IpcControlReport([object[]]$EditMs) {
    $samples = foreach ($v in $EditMs) {
        $s = [pscustomobject]@{
            total_ms           = 123.0
            status             = 'Pass'
            edit_query_stages  = @(
                @('request', 1), @('cache-read-validate', 1),
                @('infer', 1), @('publish', 1)
            )
        }
        if ($v -is [double] -or $v -is [int]) {
            $s | Add-Member -NotePropertyName edit_session_ms -NotePropertyValue $v
        }
        elseif ($v -eq 'NULL') {
            $s | Add-Member -NotePropertyName edit_session_ms -NotePropertyValue $null
        }
        # 'MISSING' keeps the property deliberately absent
        $s
    }
    [pscustomobject]@{
        samples         = @($samples)
        verdict         = 'PASS'
        generator       = 'gen796-1mi/v1'
        reference_bytes = 1048577
        total_p95_ms    = 2055.39
    }
}

if ($Scope -eq 'selftest') {
    # F-796-R6-08: the acceptance helper's own negative controls.
    $cases = @(
        @{ Name = 'green-control'; Code = 0; Out = "test result: ok. 2 passed; 0 failed; 0 ignored`ntest a_x ... ok`ntest b_y ... ok"; Min = 2; Req = @('a_x', 'b_y'); Expect = $true },
        @{ Name = 'zero-selection'; Code = 0; Out = 'test result: ok. 0 passed; 0 failed; 0 ignored'; Min = 1; Req = @(); Expect = $false },
        @{ Name = 'missing-required'; Code = 0; Out = "test result: ok. 1 passed; 0 failed; 0 ignored`ntest present ... ok"; Min = 1; Req = @('meta_query_ct_nested_unknown_rejected_everywhere'); Expect = $false },
        @{ Name = 'missing-target'; Code = 101; Out = 'error: no test target named `nope`'; Min = 1; Req = @(); Expect = $false },
        @{ Name = 'real-red'; Code = 101; Out = "test failed`ntest red_one ... FAILED`ntest result: FAILED. 0 passed; 1 failed; 0 ignored"; Min = 1; Req = @(); Expect = $false }
    )
    $bad = 0
    foreach ($c in $cases) {
        $r = Get-SuiteOutcome $c.Code $c.Out $c.Min $c.Req
        $want = if ($c.Expect) { 'PASS' } else { 'FAIL' }
        if ($r.Ok -eq $c.Expect) {
            Add-Result "S-$($c.Name)" 'PASS' "helper verdict $($r.Ok) as expected"
        }
        else {
            $bad++
            Add-Result "S-$($c.Name)" 'FAIL' "helper verdict $($r.Ok), expected $($c.Expect)"
        }
    }

    # F-796-R10-02: V-07 raw-report admission controls — the report gate
    # must reject missing/null/zero/negative edit times (all-or-one) and
    # recompute the threshold; a legal report passes, a 1001ms report
    # fails on the SLOW side (high control).
    $v07Cases = @(
        @{ Name = 'v07-legit-report'; Report = New-IpcControlReport @(1..30 | ForEach-Object { 274.0 }); Expect = $true },
        @{ Name = 'v07-missing-all'; Report = New-IpcControlReport @(1..30 | ForEach-Object { 'MISSING' }); Expect = $false },
        @{ Name = 'v07-null-all'; Report = New-IpcControlReport @(1..30 | ForEach-Object { 'NULL' }); Expect = $false },
        @{ Name = 'v07-zero-all'; Report = New-IpcControlReport @(1..30 | ForEach-Object { 0.0 }); Expect = $false },
        @{ Name = 'v07-negative-all'; Report = New-IpcControlReport @(1..30 | ForEach-Object { -1.0 }); Expect = $false },
        @{ Name = 'v07-one-missing'; Report = New-IpcControlReport (@(1..29 | ForEach-Object { 274.0 }) + @('MISSING')); Expect = $false },
        @{ Name = 'v07-slow-1001ms'; Report = New-IpcControlReport @(1..30 | ForEach-Object { 1001.0 }); Expect = $false }
    )
    foreach ($c in $v07Cases) {
        $r = Get-IpcReportOutcome $c.Report
        if ($r.Ok -eq $c.Expect) {
            Add-Result "S-$($c.Name)" 'PASS' ("gate verdict {0} as expected (n={1}, p95={2}, bad={3})" -f $r.Ok, $r.N, $r.P95, $r.Bad.Count)
        }
        else {
            $bad++
            $firstBad = ($r.Bad | Select-Object -First 1)
            Add-Result "S-$($c.Name)" 'FAIL' ("gate verdict {0}, expected {1} (n={2}, p95={3}, bad={4}: {5})" -f $r.Ok, $c.Expect, $r.N, $r.P95, $r.Bad.Count, $firstBad)
        }
    }
    Write-Host ''
    Write-Host '==== PLAN-796 verify (selftest) ====' -ForegroundColor Cyan
    $script:Results | ForEach-Object { Write-Host $_ }
    if ($bad -gt 0) { exit 1 }
    Write-Host 'selftest: PASS' -ForegroundColor Green
    exit 0
}

if ($Scope -eq 'acceptance') {
    # --- V-01 / AC-01: identity, recursive groups, matrix ------------
    Run-Suite 'V-01' @('test', '-p', 'ac-sema-contract') 'sema_identity_ + unit (contract)' $ws 16 @(
        'sema_identity_r11_method_owner_is_part_of_the_nominal_identity',
        'sema_identity_r11_method_declref_codec_roundtrip'
    )

    # --- V-02 / AC-02: real inference vs the old pipeline ------------
    # F-796-R6-08: the CT admission controls are REQUIRED by NAME, not
    # merely counted — filtering either out now fails the row.
    Run-Suite 'V-02a' @('test', '-p', 'ac-sema-core') 'infer_core_ unit suite' $ws 13 @('meta_query_ct_nested_unknown_rejected_everywhere', 'meta_query_ct_model_result_admission')
    Run-Suite 'V-02b' @('test', '-p', 'ac-sema-probe', '--test', 'infer_core_parity') 'differential parity corpus' $ws 4

    # --- V-03 / AC-03: queries, deps, cache lifecycle ----------------
    Run-Suite 'V-03' @('test', '-p', 'ac-sema-probe', '--test', 'meta_query_cache_deps') 'meta_query_ dependency faces' $ws 7
    Run-Suite 'V-03b' @('test', '-p', 'ac-sema-probe', '--test', 'sema_cache_first_case', '--test', 'sema_cache_more', '--test', 'sema_cache_r4_repairs') 'sema_cache_ first case + lifecycle + repairs' $ws 34 @(
        'sema_cache_r11_method_owner_identity_distinguishes_a_get_b_get_and_top_get',
        'sema_cache_r11_foreign_owner_method_page_rejected_on_read',
        'sema_cache_r9_method_record_return_is_part_of_the_version',
        'sema_cache_r8_method_signature_is_a_freshness_input',
        'sema_cache_r7_three_level_leaf_edit_republishes_current_closure',
        'sema_cache_r7_scc_formation_and_split_change_the_carrier',
        'sema_cache_r7_external_group_dependency_tracks_effective_version',
        'sema_cache_r6_group_member_edit_new_key_no_stale_page_no_conflict',
        'sema_cache_r6_dag_dependency_version_change_invalidates_reader',
        'sema_cache_r6_stale_ref_version_page_rejected_on_read',
        'sema_cache_r6_reuse_roots_current_project_and_survives_gc',
        'sema_cache_r5_recursive_group_publishes_recovers_survives_gc',
        'sema_cache_r5_group_with_external_dag_required_edges'
    )

    # --- V-06 / AC-03: budgets and pressure ---------------------------
    Run-Suite 'V-06' @('test', '-p', 'ac-sema-probe', '--test', 'sema_budget_pressure') 'sema_budget_ 32MiB/deep/churn/prealloc' $ws 12 @('sema_budget_deep_chain_4096_full_path', 'sema_budget_streamed_32mib_under_managed_budget', 'sema_budget_large_inference_rejects_before_publish', 'sema_budget_long_names_are_accounted', 'sema_budget_cache_readback_reserves_before_decode', 'sema_budget_long_names_reject_before_encode', 'sema_budget_oversized_page_rejected_as_resource_before_encode', 'sema_budget_long_name_readback_is_charged', 'sema_budget_target_name_is_part_of_the_page_budget')

    # --- V-04 / V-07 / AC-04: turnaround stages + the hard gate ------
    Run-Suite 'V-04' @('test', '-p', 'ac-sema-probe', '--test', 'sema_turnaround_scenarios') 'sema_turnaround_ six scenarios + gate' $ws 5
    # F-796-R6-08: REGENERATE the committed library report from the
    # current code — V-07 must never adjudicate a stale committed file.
    $benchStore = Join-Path ([System.IO.Path]::GetTempPath()) ("ac796-bench-" + [System.Diagnostics.Process]::GetCurrentProcess().Id)
    Run-Command 'V-04b' @('run', '--quiet', '-p', 'ac-sema-probe', '--', 'bench-turnaround', (Join-Path $repoRoot 'docs/reports/plan796'), $benchStore) 'regenerate docs/reports/plan796/turnaround.{json,md}' $ws

    # --- V-05 / AC-01: CT/generic seam (model-only, schema-frozen) ----
    Run-Suite 'V-05' @('test', '-p', 'ac-sema-probe', '--test', 'meta_query_cache_deps', '--', 'meta_query_ct_model_seam_admission_and_cycles') 'ct seam admission/cycles (model)' $ws 1 @('meta_query_ct_model_seam_admission_and_cycles')

    # --- V-09 / AC-06: boundary + parallel audit ----------------------
    Run-Suite 'V-09' @('test', '-p', 'ac-sema-probe', '--test', 'sema_boundary_audit') 'sema_boundary_ tree/diff/provenance' $ws 6

    # --- V-08 / AC-05: formal AutoDown execution (T-06 landed) --------
    # D-788 was satisfied (PLAN-788 merged, receipts on master); the
    # probe adapter corpus now EXECUTES through the unified runner:
    # real sema worker processes, real restart boundary, parent-side
    # judgement. The corpus+admission suite is the V-08 gate.
    Run-Suite 'V-08' @('test', '-p', 'auto-lang', '--features', 'test-cases', '--', 'probe_adapter', '--skip', 'tests::probe_adapter_turnaround_ipc') 'probe adapter through the unified runner (V-08)' $repoRoot 16 @(
        'tests::probe_adapter_tests::probe_adapter_corpus_executes_green',
        'tests::probe_adapter_tests::probe_adapter_parse_error_is_language_negative',
        'tests::probe_adapter_tests::probe_adapter_descendant_held_pipes_bounded_by_parent_containment',
        'tests::probe_adapter_tests::probe_adapter_stale_unchanged_report_fresh_and_swap_stale',
        'tests::probe_adapter_tests::probe_adapter_mixed_report_uses_per_family_toolchain_identity'
    )
    # the REAL adapter/IPC turnaround hard gate (F-796-R4-10/R6-06);
    # F-796-R10-01 binds the window-tiling control by name as well
    Run-Suite 'V-08c' @('test', '-p', 'auto-lang', '--features', 'test-cases', '--lib', '--', 'tests::probe_adapter_turnaround_ipc') 'real adapter/IPC path on the 1MiB module (30 samples)' $repoRoot 2 @(
        'tests::probe_adapter_turnaround_ipc::probe_adapter_turnaround_ipc_hard_gate_p95',
        'tests::probe_adapter_turnaround_ipc::probe_adapter_edit_session_window_tiles_prep_worker_admit'
    )
    $caseFiles = Get-ChildItem (Join-Path $repoRoot 'crates/auto-lang/test/cases/ac-semantics') -Filter '*.case.ad'
    $pending = ($caseFiles | Where-Object { (Get-Content $_.FullName -Raw) -match 'not-executed' }).Count
    if ($pending -gt 0) {
        Add-Result 'V-08b' 'FAIL' ("{0}/{1} case files still carry not-executed markers" -f $pending, $caseFiles.Count)
    }

    # --- V-07 / AC-04: report gate AFTER the writing tests ------------
    # F-796-R6-08: the previous order read the reports BEFORE V-08c
    # rewrote them, and trusted the verdict/sample count. Now the gate
    # runs last and RECOMPUTES the thresholds from the raw samples.
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
        $entry = @($json.raw | Where-Object { $_[0] -eq 'local-body-edit' })
        $libSamples = if ($entry.Count -gt 0) { @($entry[0][1]) } else { @() }
        # F-796-R10-02: per-sample raw admission on BOTH time sources —
        # a missing/null/non-finite/non-positive value fails the gate
        # instead of defaulting to 0ms.
        $libAdm = Get-AdmittedTimes $libSamples 'total_ns' 1e6
        $libP95 = Get-P95Ms $libAdm.Times
        $libPublished = @($libSamples | Where-Object { $_.status -ne 'published' -or [int]$_.compute_calls -ne 1 }).Count -eq 0
        $genOk = $json.machine.generator -eq 'gen796-1mi/v1'
        $libOk = ($libSamples.Count -ge 30) -and ($libAdm.Bad.Count -eq 0) -and ($libP95 -lt 1000.0) -and $libPublished -and $genOk

        $ipcOutcome = Get-IpcReportOutcome $ipc
        if ($libOk -and $ipcOutcome.Ok) {
            Add-Result 'V-07' 'PASS' ("recomputed library P95 {0:N2}ms ({1} raw) + REAL IPC edit-session P95 {2:N2}ms ({3} raw, full request-to-admission window, 1MiB local edit); generator {4}" -f $libP95, $libSamples.Count, $ipcOutcome.P95, $ipcOutcome.N, $json.machine.generator)
        }
        else {
            $libBad = ($libAdm.Bad | Select-Object -First 3) -join '; '
            $ipcBad = ($ipcOutcome.Bad | Select-Object -First 3) -join '; '
            Add-Result 'V-07' 'FAIL' "libOk=$libOk (n=$($libSamples.Count) p95=$libP95 published=$libPublished) ipcOk=$($ipcOutcome.Ok) (n=$($ipcOutcome.N) p95=$($ipcOutcome.P95) allPass=$($ipcOutcome.AllPass) stages=$($ipcOutcome.Stages) gen=$($ipc.generator) bytes=$($ipc.reference_bytes)) raw-admission: lib[$libBad] ipc[$ipcBad]"
        }
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
        Write-Host "acceptance: FAIL ($($script:Failed) item(s) not PASS — V-08 stays BLOCKED until D-788 + T-06 by design)" -ForegroundColor Red
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
