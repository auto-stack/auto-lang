# PLAN-822 verify script: the av.collections/1 acceptance driver.
# Template: verify-av-owned-821.ps1 + verify-av-data-820.ps1's guard ideas.
# F822-R2-04: the guard REJECTS ignored/skipped cases and verifies every
# REQUIRED case id appears EXACTLY ONCE in the run output (missing or
# duplicated mandatory subcases are failures, not silence). -GuardSelfTest
# replays the validator controls on synthetic outputs (valid/skip/missing/
# duplicate) and must exit non-zero on every bad shape.
[CmdletBinding()]
param(
    [Parameter(ParameterSetName = 'Scope', Mandatory = $true)]
    [ValidateSet("first-case", "semantics", "admission", "resources", "cache", "compatibility", "autodown", "incremental", "acceptance")]
    [string]$Scope,

    [Parameter(ParameterSetName = 'Guard')]
    [switch]$GuardSelfTest
)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot | Split-Path -Parent
$probe = Join-Path $root 'experimental/ac-av-probe'

# The output completeness guard (F822-R2-04):
#  - a result line must exist;
#  - zero selected tests is a FAIL (a green banner with no cases);
#  - ANY ignored/skipped case is a FAIL (mandatory subcases may not skip);
#  - every required id must appear EXACTLY ONCE among the executed test
#    lines (missing = dropped mandatory case; 2+ = duplicated).
function Assert-Selected {
    param(
        [string]$Output, [string]$Bin, [string]$Filter, [string]$Label,
        [string[]]$RequiredIds = @()
    )
    if ($Output -notmatch 'test result:') {
        throw "$Label : no test-result line captured ($Bin $Filter)"
    }
    if ($Output -match 'test result: ok\. 0 passed; 0 failed') {
        throw "$Label : zero tests selected ($Bin $Filter) — a green banner with no cases is a FAIL"
    }
    if ($Output -match ';\s*(\d+) ignored') {
        $ignored = [int]$Matches[1]
        if ($ignored -gt 0) {
            throw "$Label : $ignored ignored/skipped cases ($Bin $Filter) — mandatory subcases may not skip"
        }
    }
    foreach ($id in $RequiredIds) {
        $count = ([regex]::Matches($Output, "(?m)^test $([regex]::Escape($id)) .*$")).Count
        if ($count -ne 1) {
            throw "$Label : required case '$id' matched $count times (expected exactly 1) — missing or duplicated mandatory subcase ($Bin $Filter)"
        }
    }
}

function Run-Probe-Tests {
    param([string]$Bin, [string]$Filter = '', [string]$Label, [string[]]$RequiredIds = @())
    Push-Location $probe
    try {
        $args = @('test', '-p', 'auto-av-probe', '--test', $Bin)
        if ($Filter) { $args += $Filter }
        $out = & cargo @args 2>&1 | Out-String
        Write-Host $out
        Assert-Selected -Output $out -Bin $Bin -Filter $Filter -Label $Label -RequiredIds $RequiredIds
        if ($LASTEXITCODE -ne 0) { throw "$Label : cargo test failed ($Bin $Filter)" }
    } finally { Pop-Location }
}

function Run-Runner-Tests {
    param([string]$Filter, [string]$Label, [string[]]$RequiredIds = @())
    Push-Location $root
    try {
        $out = & cargo test -p auto-lang --lib --features auto-lang/test-cases $Filter '--' '--test-threads=1' 2>&1 | Out-String
        Write-Host $out
        Assert-Selected -Output $out -Bin 'auto-lang' -Filter $Filter -Label $Label -RequiredIds $RequiredIds
        if ($LASTEXITCODE -ne 0) { throw "$Label : runner tests failed ($Filter)" }
    } finally { Pop-Location }
}

if ($GuardSelfTest) {
    # F822-R2-04 validator controls on synthetic outputs. Every bad shape
    # MUST throw (exit non-zero); the valid shape MUST pass.
    $ok = 0; $bad = 0
    $valid = "running 2 tests`ntest alpha ... ok`ntest beta ... ok`n`ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s`n"
    $skip  = "running 2 tests`ntest alpha ... ok`ntest beta ... ignored, reason`n`ntest result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out`n"
    $missing = $valid
    $dup = "running 3 tests`ntest alpha ... ok`ntest alpha ... ok`ntest beta ... ok`n`ntest result: ok. 3 passed; 0 failed; 0 ignored; 0 measured`n"

    Assert-Selected -Output $valid -Bin 'ctrl' -Filter '' -Label 'valid control' -RequiredIds @('alpha', 'beta')
    $ok++

    foreach ($case in @(
        @{ Name = 'skip control'; Out = $skip;  Req = @('alpha', 'beta') },
        @{ Name = 'missing control'; Out = $missing; Req = @('alpha', 'gamma') },
        @{ Name = 'zero-selected control'; Out = "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out"; Req = @() },
        @{ Name = 'duplicate control'; Out = $dup; Req = @('alpha', 'beta') }
    )) {
        try {
            Assert-Selected -Output $case.Out -Bin 'ctrl' -Filter '' -Label $case.Name -RequiredIds $case.Req
            throw "$($case.Name): the guard ACCEPTED a bad shape"
        } catch {
            if ($_.Exception.Message -like '*the guard ACCEPTED*') { throw }
            $bad++
            Write-Host "guard control ok: $($case.Name) rejected ($($_.Exception.Message))"
        }
    }
    if ($ok -ne 1 -or $bad -ne 4) { throw "guard self-test counts wrong: ok=$ok bad=$bad" }
    Write-Host 'GuardSelfTest: all validator controls behave' -ForegroundColor Green
    exit 0
}

# The frozen per-scope mandatory case lists (F822-R2-04: exactly once).
$REQ = @{
    'first-case' = @(
        'av_collections_822_first_case_executes_success',
        'av_collections_822_first_case_requires_real_gates',
        'av_collections_822_first_case_restore_reverifies',
        'av_collections_822_first_case_admission_negatives'
    )
    'semantics' = @(
        'av_collections_822_semantics_list_i32_roundtrip',
        'av_collections_822_semantics_empty_list_len_cap',
        'av_collections_822_semantics_index_traps',
        'av_collections_822_semantics_remove_shifts',
        'av_collections_822_semantics_insert_positions',
        'av_collections_822_semantics_list_note_field_ops',
        'av_collections_822_semantics_variant_disc_guard',
        'av_collections_822_semantics_variant_project_owned_and_trap',
        'av_collections_822_semantics_variant_switch_lifecycle',
        'av_collections_822_semantics_payload_moved_rejection',
        'av_collections_822_semantics_nested_list_outcome',
        'av_collections_822_semantics_domain_rejections',
        'av_collections_822_semantics_set_keeps_neighbors',
        'av_collections_822_semantics_list_drop_reverse_elements',
        'av_collections_822_semantics_list_drop_reverse_on_trap',
        'av_collections_822_semantics_nested_variant_list_drop_reverse',
        'av_collections_822_semantics_place_positive_reads',
        'av_collections_822_semantics_place_stale_after_push',
        'av_collections_822_semantics_place_stale_after_remove_fresh_reads_shift',
        'av_collections_822_semantics_place_retired_by_element_replacement',
        'av_collections_822_semantics_place_mint_out_of_range_traps'
        'av_collections_822_semantics_cross_frame_place_write_no_panic',
        'av_collections_822_semantics_cross_frame_place_read_control',
        'av_collections_822_semantics_callee_local_place_escape_refused',
        'av_collections_822_semantics_record_place_token_refused',
        'av_collections_822_semantics_place_param_roundtrip_supported'
        'av_collections_822_semantics_local_place_escape_across_jump_refused',
        'av_collections_822_semantics_place_param_return_across_jump_supported',
        'av_collections_822_semantics_merge_shapes_local_vs_external',
        'av_collections_822_semantics_mutate_after_place_copy_supported',
        'av_collections_822_semantics_helper_roundtrip_then_update_supported'
        'av_collections_822_semantics_place_provenance_memory_budget'
        'av_collections_822_semantics_clone_list_is_independent',
        'av_collections_822_semantics_clone_owned_elements_independent',
        'av_collections_822_semantics_clone_variant_payload_independent',
        'av_collections_822_semantics_clone_nth_item_failure_rolls_back',
        'av_collections_822_semantics_clone_budget_both_sides'
        'av_collections_822_semantics_variant_self_cycle_rejected',
        'av_collections_822_semantics_variant_indirect_cycle_rejected'
        'av_collections_822_semantics_clone_visitor_cancel_seam',
        'av_collections_822_semantics_clone_visitor_fuel_seam',
        'av_collections_822_semantics_clone_visitor_full_control',
        'av_collections_822_semantics_clone_visitor_wall_seam_zero_remaining',
        'av_collections_822_semantics_clone_visitor_byte_scaled_fuel',
        'av_collections_822_semantics_clone_byte_fuel_production_witness',
        'av_collections_822_semantics_nested_variant_cleanup_domain_control',
        'av_collections_822_semantics_deep_chain_cleanup_small_stack',
        'av_collections_822_semantics_deep_chain_trap_exit_cleanup',
        'av_collections_822_semantics_variant_switch_deep_chain_release',
        'av_collections_822_semantics_partial_clone_rollback_small_stack',
        'av_collections_822_semantics_partial_clone_rollback_controls',
        'av_collections_822_semantics_nested_variant_clone_control',
        # F822-R11/T-06: the two R9 two-phase ids that stood here
        # (clone_cancel_mid_traversal / clone_wall_mid_traversal) were
        # deleted by the R10 oracle rewrite but the list kept requiring
        # them — the frozen semantics scope was red before R11 (stale
        # mandatory ids, no such tests exist in the bin).
        'av_collections_822_semantics_place_get_wrong_type_rejected',
        'av_collections_822_semantics_place_set_wrong_type_rejected',
        'av_collections_822_semantics_place_mint_binds_referent',
        'av_collections_822_semantics_move_preserves_list',
        'av_collections_822_semantics_move_preserves_owned_elements',
        'av_collections_822_semantics_copy_share_drop_source_preserves_dst',
        'av_collections_822_semantics_call_move_in_preserves_list'
    )
    'admission' = @(
        'av_collections_822_first_case_admission_negatives'
    )
    'resources' = @(
        'av_collections_822_resources_list_cap_both_sides',
        'av_collections_822_resources_slot_bytes_rejects_before_write',
        'av_collections_822_resources_fault_injection_attempt_semantics',
        'av_collections_822_resources_budget_is_the_only_delta',
        'av_collections_822_resources_peak_and_drain',
        'av_collections_822_resources_index_arithmetic_shape',
        'av_collections_822_worker_memory_hard_gate_parent_sampled',
        'av_collections_822_resources_listset_transaction_both_sides',
        'av_collections_822_resources_listfieldset_transaction_both_sides',
        'av_collections_822_resources_listremove_transaction_both_sides',
        'av_collections_822_resources_variantproject_transaction_both_sides'
    )
    'cache' = @(
        'av_collections_822_cache_restore_identity_and_miss',
        'av_collections_822_cache_good_envelope_bad_payload_rejected',
        'av_collections_822_compat_wire_generation_isolation',
        'av_collections_822_compat_layer_separation',
        'av_collections_822_cache_deterministic_replay_steps',
        'av_collections_822_cache_restore_payload_types_fp_mismatch_rejected',
        'av_collections_822_cache_restore_payload_entry_identity_rejected',
        'av_collections_822_cache_restore_payload_profile_rejected'
    )
    'incremental' = @(
        'av_collections_822_incremental_body_edit_identity',
        'av_collections_822_incremental_rejected_revision_preserves_session',
        'av_collections_822_incremental_execution_failure_preserves_session',
        'av_collections_822_incremental_signature_change_precise',
        'av_collections_822_incremental_forced_degradation_gate_nonzero',
        'av_collections_822_incremental_baseline_campaign_replay',
        'av_collections_822_incremental_place_referent_out_of_range_refused',
        'av_collections_822_incremental_place_referent_boundary_and_domain',
        'av_collections_822_incremental_layout_internal_revision_precise',
        'av_collections_822_incremental_layout_interface_change_rejected',
        'av_collections_822_incremental_host_grant_lifecycle',
        'av_collections_822_worker_memory_hard_gate_parent_sampled'
    )
}

switch ($Scope) {
    'first-case' {
        Run-Probe-Tests -Bin 'av_collections_822_first_case' -Label 'first-case' -RequiredIds $REQ['first-case']
    }
    'semantics' {
        Run-Probe-Tests -Bin 'av_collections_822_semantics' -Label 'semantics' -RequiredIds $REQ['semantics']
    }
    'admission' {
        Run-Probe-Tests -Bin 'av_collections_822_first_case' -Filter 'av_collections_822_first_case_admission_negatives' -Label 'admission' -RequiredIds $REQ['admission']
    }
    'resources' {
        Run-Probe-Tests -Bin 'av_collections_822_resources' -Label 'resources' -RequiredIds (($REQ['resources'] | Where-Object { $_ -notlike '*memory_hard_gate*' }))
    }
    'compatibility' {
        # V-09: the PRIOR profiles replay unchanged through their own
        # frozen corpora (798/817/818/820/821); the collections wire is a
        # new AVAD generation the old consumers reject by design.
        Run-Probe-Tests -Bin 'av_owned_821_first_case' -Label 'compatibility: 821 first-case'
        Run-Probe-Tests -Bin 'av_owned_821_cache_compat' -Label 'compatibility: 821 cache'
        Run-Runner-Tests -Filter 'av_owned_821' -Label 'compatibility: 821 corpus' -RequiredIds @(
            'tests::av_owned_821_tests::av_owned_821_corpus_green',
            'tests::av_owned_821_tests::av_owned_821_negative_oracle_fails',
            'tests::av_owned_821_tests::av_owned_821_worker_faults_are_infra'
        )
        Run-Runner-Tests -Filter 'ac_av_probe' -Label 'compatibility: 798/820 corpus'
        Run-Runner-Tests -Filter 'cte_probe' -Label 'compatibility: 817 corpus'
        Run-Runner-Tests -Filter 'ac_mn_probe' -Label 'compatibility: 818 corpus' -RequiredIds @(
            'tests::ac_mn_probe_tests::ac_mn_probe_corpus_executes_green',
            'tests::ac_mn_probe_tests::ac_mn_probe_negative_oracle_fails',
            'tests::ac_mn_probe_tests::mir_native_adapter_bad_actual_oracles_are_red',
            'tests::ac_mn_probe_tests::mir_native_adapter_diagnosed_rejection_faces',
            'tests::ac_mn_probe_tests::mir_native_adapter_edit_source_materializes_and_judges',
            'tests::ac_mn_probe_tests::mir_native_adapter_restart_session_reuses_cache_across_processes'
        )
    }
    'autodown' {
        Run-Runner-Tests -Filter 'av_collections_822' -Label 'autodown: recipe corpus + counter-controls' -RequiredIds @(
            'tests::av_collections_822_tests::av_collections_822_corpus_green',
            'tests::av_collections_822_tests::av_collections_822_negative_oracle_fails',
            'tests::av_collections_822_tests::av_collections_822_worker_faults_are_infra'
        )
    }
    'incremental' {
        Run-Probe-Tests -Bin 'av_collections_822_incremental' -Label 'incremental: V-13 identity/baseline/gate' -RequiredIds (($REQ['incremental'] | Where-Object { $_ -notlike '*memory_hard_gate*' }))
        Run-Probe-Tests -Bin 'av_collections_822_resources' -Filter 'av_collections_822_worker_memory_hard_gate_parent_sampled' -Label 'incremental: worker B-line memory gate' -RequiredIds @('av_collections_822_worker_memory_hard_gate_parent_sampled')
    }
    'acceptance' {
        # Replay EVERY scope in a CHILD process (env seams cannot leak).
        foreach ($s in @('first-case', 'semantics', 'admission', 'resources', 'cache', 'compatibility', 'autodown', 'incremental')) {
            Write-Host "=== acceptance: scope $s ===" -ForegroundColor Cyan
            & pwsh -NoProfile -File $PSCommandPath -Scope $s
            if ($LASTEXITCODE -ne 0) { throw "acceptance: scope $s failed" }
        }
        Write-Host 'PLAN-822 acceptance: all scopes green' -ForegroundColor Green
    }
}
