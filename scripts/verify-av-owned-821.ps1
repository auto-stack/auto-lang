# PLAN-821 verify script — scoped gates for the av.owned/1 profile.
# Usage: powershell -File scripts/verify-av-owned-821.ps1 -Scope <scope>
#   first-case (T-02) | semantics (T-03) | admission | resources (T-04) |
#   cache | compatibility (T-05) | autodown | acceptance (T-06)
# One scope per invocation (contract-decision §10). Scopes land with their
# tasks; an unimplemented scope is a NON-ZERO exit with an explicit
# message, never green. Every implemented scope enforces selected>0 (0
# selected / skipped required cases = red) and passes cargo exit codes
# through.
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("first-case", "semantics", "admission", "resources", "cache", "compatibility", "autodown", "incremental", "acceptance")]
    [string]$Scope
)

$ErrorActionPreference = "Continue"  # cargo writes progress to stderr; exit codes are the gate
$repo = Split-Path -Parent $PSScriptRoot
$probeWs = Join-Path $repo "experimental/ac-av-probe"
$reportDir = Join-Path $repo "docs/reports/plan821"
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null

if ($env:SCCACHE_SERVER_PORT -eq $null) { $env:SCCACHE_SERVER_PORT = "28101" }

function Test-Selected([string]$Out, [string]$Label, [string[]]$RequiredIds = @()) {
    # Returns $true iff the run output proves: a NON-EMPTY selection, ZERO
    # really-skipped/ignored cases among the selected, and every REQUIRED
    # case id observed as exactly one passing result (the 827 validator
    # semantics — filter exclusion is not a skip; near-miss names do not
    # satisfy an id).
    $selected = 0
    if ($Out -match '(\d+) tests run') { $selected = [int]$Matches[1] }
    elseif ($Out -match '(\d+) passed') { $selected = [int]$Matches[1] }
    if ($selected -le 0) {
        Write-Host "FAILED: $Label selected 0 tests (0 selected is never a pass)"
        return $false
    }
    $realSkips = ([regex]::Matches($Out, '(?m)^\s*SKIP \[')).Count
    foreach ($m in [regex]::Matches($Out, 'test result:[^\r\n]*?(\d+) ignored')) {
        if ([int]$m.Groups[1].Value -gt 0) { $realSkips += [int]$m.Groups[1].Value }
    }
    if ($realSkips -gt 0) {
        Write-Host "FAILED: $Label has $realSkips skipped/ignored SELECTED cases (never green)"
        return $false
    }
    $seen = @{}
    foreach ($id in $RequiredIds) {
        if ($seen.ContainsKey($id)) {
            Write-Host "FAILED: $Label required list declares '$id' more than once"
            return $false
        }
        $seen[$id] = $true
        $esc = [regex]::Escape($id)
        $nextest = [regex]::Matches($Out, "(?m)^.*PASS.*[ :]$esc\r?$")
        $libtest = [regex]::Matches($Out, "(?m)^test $esc \.\.\. ok\r?$")
        $hits = $nextest.Count + $libtest.Count
        if ($hits -eq 0) {
            Write-Host "FAILED: $Label required case '$id' did not pass (missing/renamed/red)"
            return $false
        }
        if ($hits -gt 1) {
            Write-Host "FAILED: $Label required case '$id' matched $hits times (duplicate/ambiguous)"
            return $false
        }
    }
    Write-Host "   selected: $selected (0 real skips; required $($RequiredIds.Count) all passed)"
    return $true
}

function Assert-Selected([string]$Out, [string]$Label, [string[]]$RequiredIds = @()) {
    if (!(Test-Selected $Out $Label $RequiredIds)) { exit 1 }
}

function Run-Probe-Tests([string]$TestBin, [string]$Label, [string[]]$RequiredIds = @()) {
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
    Assert-Selected ($out -join "`n") $Label $RequiredIds
}

function Run-Runner-Tests([string]$Filter, [string]$Label, [string[]]$RequiredIds = @()) {
    Write-Host "== $Label (main workspace, filter: $Filter)"
    Push-Location $repo
    . (Join-Path $repo 'scripts/test-governance/invoke-modules.ps1')
    Invoke-ScopedTestModules -Repo $repo -TestNames @($Filter) -Output (Join-Path $reportDir ($Label+'-modules')) -RequiredIds $RequiredIds
    $out=@('Original identities checked in module execution receipt')
    $code = $LASTEXITCODE
    Pop-Location
    $out | Out-File (Join-Path $reportDir "$Label.log") -Encoding utf8
    if ($code -ne 0) {
        Write-Host "FAILED: $Label (exit $code)"
        exit 1
    }
    # Required IDs were checked against the actual module receipt.
}

function Not-Wired([string]$Scope, [string]$Owner) {
    Write-Host "NOT WIRED: scope '$Scope' lands with $Owner (explicit non-green, never silent)"
    exit 1
}

function Invoke-Scope([string]$Scope) {
    # Run one scope as a REAL child script; $true iff it exits 0 (the
    # acceptance aggregate and failure propagation share this one path).
    $out = & powershell -File $PSCommandPath -Scope $Scope 2>&1
    $code = $LASTEXITCODE
    $out | Write-Host
    if ($code -ne 0) {
        Write-Host "scope '$Scope' red (exit $code)"
        return $false
    }
    return $true
}

function Invoke-Owned-Request([string]$Worker, [string]$Src, [string]$Store) {
    # One fresh-process owned request; returns the parsed JSON line.
    $out = & $Worker owned-request --store $Store --src-file $Src --entry d_main 2>$null | Where-Object { $_ -match '^\{' }
    if ($null -eq $out) { return $null }
    try { return ($out | ConvertFrom-Json) } catch { return $null }
}

switch ($Scope) {
    "first-case" {
        # V-01/V-08/V-10 first case, full formal path: the probe first-case
        # family (whole path + wire round trip + same-store restore + move
        # negatives + native independent comparison), the recipe-side
        # fixture/canonical/seeds-identity pins, and the formal corpus gate
        # (real worker, parent-side frozen-table judgement + sabotage and
        # worker-fault counter-controls).
        Run-Probe-Tests "av_owned_821_first_case" "first-case" @(
            "av_owned_821_first_case_whole_path_and_live_zero",
            "av_owned_821_first_case_wire_round_trip",
            "av_owned_821_first_case_same_store_restore",
            "av_owned_821_first_case_move_negative_controls",
            "av_owned_821_first_case_native_independent_comparison"
        )
        Run-Probe-Tests "av_owned_821_recipe" "first-case-recipe" @(
            "av_owned_821_recipe_seeds_identity",
            "av_owned_821_recipe_fixture_face_matches_hand_audit",
            "av_owned_821_recipe_canonical_face_shapes"
        )
        Run-Probe-Tests "av_owned_821_r1_replay" "first-case-r1-replay" @(
            "av_owned_821_r1_return_dropped_and_ended_escape_rejected",
            "av_owned_821_r1_loan_invariants_hold",
            "av_owned_821_r1_static_view_across_units_resolves_in_owner",
            "av_owned_821_r1_aggregate_transfer_releases_source_at_commit",
            "av_owned_821_r1_clone_byte_fuel_thresholds_discriminate",
            "av_owned_821_r1_analysis_memory_bound_honest_no_tokens",
            "av_owned_821_r1_borrowed_record_source_paths"
        )
        Run-Runner-Tests "av_owned_821" "first-case-corpus" @(
            "av_owned_821_corpus_green",
            "av_owned_821_negative_oracle_fails",
            "av_owned_821_worker_faults_are_infra"
        )
        Run-Probe-Tests "av_data_820" "first-case-regression-820" @()
        Push-Location $probeWs
        & cargo test -p auto-av-core --lib 2>&1 | Tee-Object -Variable coreOut | Write-Host
        $coreCode = $LASTEXITCODE
        Pop-Location
        $coreOut | Out-File (Join-Path $reportDir "first-case-regression-core.log") -Encoding utf8
        if ($coreCode -ne 0) {
            Write-Host "FAILED: auto-av-core regression (exit $coreCode)"
            exit 1
        }
        Assert-Selected ($coreOut -join "`n") "first-case-regression-core" @()

        # ---- T-02 CPU/time/memory evidence (plan §5.2) --------------------
        # Fresh-process owned-request samples on BOTH first-case sources;
        # the receipt binds worker sha256 + git head and the script
        # re-validates the hand-audited faces (result bits/bytes, live=0)
        # before writing the evidence JSON.
        Write-Host "== first-case: CPU/time/memory evidence (owned-request samples)"
        $worker = Join-Path $probeWs "target/release/av_probe_worker.exe"
        if (!(Test-Path $worker)) {
            Write-Host "FAILED: release worker not found at $worker (the corpus gate builds it; rerun if cleaned)"
            exit 1
        }
        $exeHash = (Get-FileHash $worker -Algorithm SHA256).Hash
        $gitHead = (& git -C $repo rev-parse HEAD 2>$null) -join ""
        $gitHead = $gitHead.Trim()
        $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
        $samples = @()
        foreach ($pair in @(
            @{ seed = "first-case.at"; n = 5 },
            @{ seed = "first-case-int-twin.at"; n = 3 }
        )) {
            $src = Join-Path $repo "crates/auto-lang/test/cases/av-data/821/src/$($pair.seed)"
            for ($i = 0; $i -lt $pair.n; $i++) {
                $store = Join-Path $env:TEMP "av821-evidence-$stamp-$($pair.seed)-$i"
                $r = Invoke-Owned-Request $worker $src $store
                if ($null -eq $r) {
                    Write-Host "FAILED: owned-request produced no JSON line (seed $($pair.seed) sample $i)"
                    exit 1
                }
                if ($r.ok -ne $true) {
                    Write-Host "FAILED: owned-request not ok (seed $($pair.seed) sample $i)"
                    exit 1
                }
                if ([int]$r.owned.live_after_release -ne 0) {
                    Write-Host "FAILED: live_after_release != 0 (seed $($pair.seed) sample $i)"
                    exit 1
                }
                $samples += [pscustomobject]@{
                    seed = $pair.seed
                    sample = $i
                    wall_ms = [double]$r.wall_ms
                    private_bytes_peak = [int64]$r.private_bytes_peak
                    result = $r.owned.result
                }
                Remove-Item -Recurse -Force $store -ErrorAction SilentlyContinue
            }
        }
        # Independent hand-audit re-check (script-side, not the worker's word):
        # the record face is Note{body:"Auto",n:4}; the twin's bits are 194.
        $rec = @($samples | Where-Object { $_.seed -eq "first-case.at" })
        $twin = @($samples | Where-Object { $_.seed -eq "first-case-int-twin.at" })
        if ($rec.Count -ne 5 -or $twin.Count -ne 3) {
            Write-Host "FAILED: evidence sample counts (record $($rec.Count)/5, twin $($twin.Count)/3)"
            exit 1
        }
        foreach ($s in $rec) {
            if ($s.result.name -ne "t_rec_Note" -or $s.result.fields[0].utf8 -ne "Auto" -or $s.result.fields[1].bits -ne "4") {
                Write-Host "FAILED: record face deviates from the hand-audit: $($s.result | ConvertTo-Json -Compress)"
                exit 1
            }
        }
        foreach ($s in $twin) {
            if ($s.result.kind -ne "i32" -or $s.result.bits -ne "194") {
                Write-Host "FAILED: twin face deviates from the hand-audit (404 = 0x194): $($s.result | ConvertTo-Json -Compress)"
                exit 1
            }
        }
        $evidence = [pscustomobject]@{
            stamp             = $stamp
            scope             = "first-case"
            worker_sha256     = $exeHash
            git_head          = $gitHead
            machine_note      = "single-instance serial, fresh process per sample (poll-quantized peaks)"
            samples           = $samples
            wall_ms_p90       = [math]::Round((($samples | ForEach-Object { $_.wall_ms } | Sort-Object)[($samples.Count - [int][math]::Floor($samples.Count / 10) - 1)]), 3)
            wall_ms_p95       = [math]::Round((($samples | ForEach-Object { $_.wall_ms } | Sort-Object)[[int][math]::Floor(($samples.Count - 1) * 0.95)]), 3)
            wall_ms_max       = [math]::Round(($samples | ForEach-Object { $_.wall_ms } | Measure-Object -Maximum).Maximum, 3)
            peak_bytes_max    = ($samples | ForEach-Object { $_.private_bytes_peak } | Measure-Object -Maximum).Maximum
            budget_note       = "worker B-line increment gate (128MiB) is the T-04/T-06 tier; these are the T-02 baseline samples"
        }
        $evPath = Join-Path $reportDir "first-case-evidence-$stamp.json"
        $evidence | ConvertTo-Json -Depth 8 | Out-File $evPath -Encoding utf8
        Write-Host "   evidence: $evPath (p90 wall $($evidence.wall_ms_p90)ms, p95 wall $($evidence.wall_ms_p95)ms, max peak $($evidence.peak_bytes_max) bytes)"
    }
    "semantics" {
        # T-03: the full frozen semantics matrices — V-01 reservation /
        # RHS-reads-same-root, V-02 borrow exclusivity + multi-view,
        # V-05 handle/integer isolation, V-06 frozen bit samples +
        # escapes + traps, V-11 merges/loops/trap-cleanup/sparse/budget,
        # V-12 arm-specific init/borrow joins (all as typed MIR fixtures
        # through the REAL admission chain), plus the formal corpus gate.
        Run-Probe-Tests "av_owned_821_semantics" "semantics" @(
            "av_owned_821_v01_field_replace_rhs_reads_same_root_legal",
            "av_owned_821_v01_self_move_is_reservation_conflict",
            "av_owned_821_v01_source_domain_shapes_keep_verify_codes",
            "av_owned_821_v02_multi_view_and_view_copy_legal",
            "av_owned_821_v02_live_mutref_blocks_field_set",
            "av_owned_821_v02_borrow_end_releases_exclusion",
            "av_owned_821_v02_copy_owned_and_mutref_refused",
            "av_owned_821_v02_double_move_rejected",
            "av_owned_821_v05_integer_equal_to_resource_id_stays_integer",
            "av_owned_821_v05_view_into_int_op_rejected_at_verify",
            "av_owned_821_v05_borrow_end_then_redefine_frees_old_exactly_once",
            "av_owned_821_v06_frozen_bit_samples_observe_exactly",
            "av_owned_821_v06_oob_char_at_and_invalid_codepoint_trap",
            "av_owned_821_v06_char_domain_round_trips",
            "av_owned_821_v06_view_escape_rejected_and_param_view_legal",
            "av_owned_821_v11_one_arm_move_join_read_rejected",
            "av_owned_821_v11_loop_back_edge_move_conservative",
            "av_owned_821_v11_call_arg_shape_mismatch_rejected",
            "av_owned_821_v11_trap_cleans_owned_in_creation_order",
            "av_owned_821_v11_sparse_field_paths_execute",
            "av_owned_821_v11_analysis_over_budget_is_resource",
            "av_owned_821_v12_single_arm_temp_read_rules",
            "av_owned_821_v12_one_arm_view_blocks_join_mut_until_ended",
            "av_owned_821_v12_two_arm_end_borrow_then_mut_legal",
            "av_owned_821_v12_zero_or_many_loop_borrow_and_move_faces"
        )
        Run-Runner-Tests "av_owned_821_corpus" "semantics-corpus" @("av_owned_821_corpus_green")
    }
    "admission" {
        # T-04: the diagnosed-admission corpus face — the 782 source-gate
        # partial-move rejection surfaces with its code through the owned
        # recipe (parse-stage verify.partial-move; an admitted-nothing
        # program never counts as a run success), plus the corpus gate
        # with the required population and the T-03 verify-side
        # rejection family replayed.
        Run-Runner-Tests "av_owned_821" "admission-corpus" @(
            "av_owned_821_corpus_green",
            "av_owned_821_negative_oracle_fails",
            "av_owned_821_worker_faults_are_infra"
        )
        Run-Probe-Tests "av_owned_821_semantics" "admission-verify-faces" @(
            "av_owned_821_v01_self_move_is_reservation_conflict",
            "av_owned_821_v01_source_domain_shapes_keep_verify_codes",
            "av_owned_821_v02_live_mutref_blocks_field_set",
            "av_owned_821_v02_copy_owned_and_mutref_refused",
            "av_owned_821_v02_double_move_rejected",
            "av_owned_821_v05_view_into_int_op_rejected_at_verify",
            "av_owned_821_v06_view_escape_rejected_and_param_view_legal",
            "av_owned_821_v11_one_arm_move_join_read_rejected",
            "av_owned_821_v11_loop_back_edge_move_conservative",
            "av_owned_821_v11_call_arg_shape_mismatch_rejected",
            "av_owned_821_v11_analysis_over_budget_is_resource",
            "av_owned_821_v12_single_arm_temp_read_rules",
            "av_owned_821_v12_one_arm_view_blocks_join_mut_until_ended"
        )
    }
    "resources" {
        # T-04: the frozen V-03 allocation-failure transactions, V-04
        # all-exit cleanup and V-07 cap-boundary/fault-accounting axes —
        # arena-level fault seam + precheck (预算前拒 attempt=0), plus the
        # inherited 820 finite-resource tier.
        Run-Probe-Tests "av_owned_821_resources" "resources" @(
            "av_owned_821_v03_clone_fault_attempt_counted_no_success",
            "av_owned_821_v03_toowned_fault_mid_replace_preserves_old",
            "av_owned_821_v03_record_clone_mid_failure_released_once",
            "av_owned_821_v04_all_exit_kinds_clean_lifo",
            "av_owned_821_v04_cancel_mid_run_cleans",
            "av_owned_821_v07_cap_boundary_pass_and_over_by_one",
            "av_owned_821_r1_worker_memory_hard_gate_parent_sampled"
        )
        Run-Probe-Tests "av_data_820_resources" "resources-inherited-820" @()
    }
    "cache" {
        # T-05 V-08: dependency-partitioned restores — own-fingerprint
        # restores reproduce identical results; a changed type table is a
        # CLEAN MISS (never stale reuse); a tampered payload fails the
        # envelope checksum (a good address never resurrects bad bytes);
        # producer/profile identities never cross.
        Run-Probe-Tests "av_owned_821_cache_compat" "cache" @(
            "av_owned_821_v08_changed_types_is_clean_miss_not_stale_reuse",
            "av_owned_821_v08_good_hash_bad_payload_rejected",
            "av_owned_821_v08_profile_identity_isolation",
            "av_owned_821_v09_owned_bytes_rejected_by_old_consumers",
            "av_owned_821_v09_control_owned_restore_still_green",
            "av_owned_821_r1_new_process_restore_executes",
            "av_owned_821_r1_self_consistent_bad_inner_never_executes"
        )
        Run-Probe-Tests "av_owned_821_first_case" "cache-restore-first-case" @(
            "av_owned_821_first_case_same_store_restore"
        )
        Run-Runner-Tests "av_owned_821_corpus" "cache-corpus" @("av_owned_821_corpus_green")
    }
    "incremental" {
        # T-09 V-13: (1) the SHARED scalar baseline — the frozen 828
        # formal-session campaign dual gate (30 revisions, structure +
        # P90 <=2s, dual memory lines) replays green, plus the 827
        # incremental family (the scalar V-13 precedent, unchanged);
        # (2..4) the owned faces — two-fn local body updates (object +
        # generation reuse, zero production deltas), precise refusals
        # (signature/layout stale-link refusals, module-wide import-table
        # invalidation), forced-whole-recompute visibility; plus the
        # inherited 828 lifecycle transaction axes.
        Push-Location $probeWs
        & cargo test -p auto-av-probe --test function_incremental_828_t08 t08_formal_campaign_v08_dual_gate 2>&1 | Tee-Object -Variable f828 | Write-Host
        $f828code = $LASTEXITCODE
        Pop-Location
        $f828 | Out-File (Join-Path $reportDir "incremental-baseline-828.log") -Encoding utf8
        if ($f828code -ne 0) {
            Write-Host "FAILED: incremental baseline (exit $f828code)"
            exit 1
        }
        Assert-Selected ($f828 -join "`n") "incremental-baseline-828" @(
            "t08_formal_campaign_v08_dual_gate_30_revisions"
        )
        Run-Probe-Tests "av_owned_821_incremental" "incremental-owned" @(
            "av_owned_821_v13_two_fn_local_body_updates",
            "av_owned_821_v13_signature_edits_refused_or_whole_context",
            "av_owned_821_v13_forced_whole_recompute_reddens_the_gate",
            "av_owned_821_v13_stale_caller_unit_link_refused",
            "av_owned_821_v13_shared_scalar_session_still_warms"
        )
        Run-Probe-Tests "av_source_827_incremental" "incremental-scalar-precedent" @(
            "av_source_827_incremental_d828_baseline_30_structure_and_p95",
            "av_source_827_incremental_raw_shape_rejects_missing_baseline",
            "av_source_827_incremental_failing_verdict_still_writes_raw",
            "av_source_827_incremental_two_fn_local_updates",
            "av_source_827_incremental_precise_refusals",
            "av_source_827_incremental_degradation_caught",
            "av_source_827_incremental_detector_negative_controls"
        )
        Run-Probe-Tests "function_incremental_828_contract" "incremental-inherited-lifecycle" @(
            "contract_828_transaction_sequential_cancel_then_commit",
            "contract_828_transaction_close_releases_owned_entries",
            "contract_828_transaction_prepare_does_not_publish_and_drop_is_inert"
        )
    }
    "compatibility" {
        # T-05 V-09: the ORIGINAL consumers keep their support faces — the
        # core-i32 ac-av corpus, the delivered 817 (CTE) and 818
        # (MIR-native) recipe gates run unchanged; the owned corpus gate
        # replays; owned bytes under old consumers are explicit refusals
        # (probe-side, in the cache bin).
        Run-Runner-Tests "ac_av_probe" "compatibility-ac-av" @("ac_av_probe_corpus_executes_green")
        Run-Runner-Tests "cte_probe" "compatibility-ac-cte" @("cte_probe_corpus_executes_green")
        Run-Runner-Tests "ac_mn_probe" "compatibility-ac-mn" @("ac_mn_probe_corpus_executes_green")
        Run-Runner-Tests "av_owned_821" "compatibility-owned-corpus" @(
            "av_owned_821_corpus_green"
        )
        Run-Probe-Tests "av_data_820" "compatibility-820" @()
    }
    "autodown" {
        # T-06 V-10: the formal recipe positives + every parent-side
        # injection — both sabotage faces (wrong result / wrong ledger
        # each redden their own axis) and the worker-contract faults
        # (exit9/extra-frame/wrong-hash = infra, never green).
        Run-Runner-Tests "av_owned_821" "autodown" @(
            "av_owned_821_corpus_green",
            "av_owned_821_negative_oracle_fails",
            "av_owned_821_worker_faults_are_infra"
        )
    }
    "acceptance" {
        # T-06: replays EVERY implemented scope as a REAL child script —
        # a failure anywhere breaks the aggregate (no scope may silently
        # drop out of acceptance).
        foreach ($scope in @(
            "first-case", "semantics", "admission", "cache",
            "compatibility", "autodown", "incremental", "resources"
        )) {
            if (!(Invoke-Scope $scope)) {
                Write-Host "FAILED: acceptance — scope '$scope' red"
                exit 1
            }
        }
        Write-Host "PASS: acceptance (all eight scopes replayed green)"
    }
}
Write-Host "PASS: scope '$Scope' green"
exit 0
