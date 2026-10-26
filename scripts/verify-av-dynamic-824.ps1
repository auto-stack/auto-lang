# PLAN-824 verify script: the av.dynamic/1 acceptance driver.
# Template: verify-av-collections-822.ps1's guard ideas. T-02 scope:
# first-case + autodown (+ acceptance = both); the remaining scopes
# (semantics/admission/resources/cache/compatibility) land with T-03..T-06.
# The output completeness guard REJECTS ignored/skipped cases and verifies
# every REQUIRED case id appears EXACTLY ONCE in the run output.
[CmdletBinding()]
param(
    [Parameter(ParameterSetName = 'Scope', Mandatory = $true)]
    [ValidateSet("first-case", "semantics", "admission", "resources", "cache", "compatibility", "autodown", "acceptance")]
    [string]$Scope,

    [Parameter(ParameterSetName = 'Guard')]
    [switch]$GuardSelfTest
)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot | Split-Path -Parent
$probe = Join-Path $root 'experimental/ac-av-probe'

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
        $count = ([regex]::Matches($Output, "(?m)^test .*\b$([regex]::Escape($id))\b.*$")).Count
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
        if ($LASTEXITCODE -ne 0) {
            throw "$Label : cargo exited $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }
}

function Run-Lang-Tests {
    param([string]$Filter, [string]$Label, [string[]]$RequiredIds = @())
    $out = & cargo test -p auto-lang --lib --features auto-lang/test-cases $Filter -- --test-threads=1 2>&1 | Out-String
    Write-Host $out
    Assert-Selected -Output $out -Bin 'auto-lang/lib' -Filter $Filter -Label $Label -RequiredIds $RequiredIds
    if ($LASTEXITCODE -ne 0) {
        throw "$Label : cargo exited $LASTEXITCODE"
    }
}

function Run-Harness-Tests {
    # The parallel/824 same-source harness: the SAME production files
    # (core/src/{dynamic,portable}.rs) compiled through the harness's
    # #[path] seam — the V-14 dual-compilation axis (module tests).
    param([string]$Label)
    $manifest = Join-Path $probe 'parallel/824/Cargo.toml'
    # --lib: exactly one result line (the doc-test target's `0 passed`
    # banner must not trip the zero-selected guard).
    $out = & cargo test --manifest-path $manifest --lib 2>&1 | Out-String
    Write-Host $out
    Assert-Selected -Output $out -Bin 'parallel/824' -Filter '' -Label $Label
    if ($LASTEXITCODE -ne 0) {
        throw "$Label : cargo exited $LASTEXITCODE"
    }
}

$REQ = @{
    'first-case' = @(
        'av_dynamic_824_first_case_executes_success',
        'av_dynamic_824_requires_real_gates',
        'av_dynamic_824_restore_reverifies',
        'av_dynamic_824_admission_negatives'
    )
    'autodown' = @(
        'av_dynamic_824_corpus_green',
        'av_dynamic_824_negative_oracle_fails',
        'av_dynamic_824_worker_faults_are_infra'
    )
    'admission' = @(
        'av_dynamic_824_admission_negatives'
    )
    'resources' = @(
        'av_dynamic_824_resources_slot_bytes_rejects_before_write',
        'av_dynamic_824_resources_value_alloc_fault_no_half_commit',
        'av_dynamic_824_resources_arena_fault_attempt_semantics',
        'av_dynamic_824_resources_arena_fault_preserves_prior_legal_events',
        'av_dynamic_824_resources_budget_is_the_only_delta',
        'av_dynamic_824_resources_boundary_both_sides',
        'av_dynamic_824_resources_deep_chain_roundtrip_within_bound',
        'av_dynamic_824_resources_deep_chain_beyond_bound_rejected',
        'av_dynamic_824_resources_take_trap_drains_ledger',
        'av_dynamic_824_resources_cancel_is_its_own_state',
        'av_dynamic_824_resources_worker_memory_hard_gate_parent_sampled'
    )
    'cache' = @(
        'av_dynamic_824_cache_restore_identity_and_miss',
        'av_dynamic_824_cache_portable_restore_identity_and_miss',
        'av_dynamic_824_cache_good_envelope_bad_payload_rejected',
        'av_dynamic_824_cache_portable_good_checksum_bad_structure_rejected',
        'av_dynamic_824_cache_restore_payload_types_fp_mismatch_rejected',
        'av_dynamic_824_cache_restore_payload_entry_identity_rejected',
        'av_dynamic_824_cache_restore_payload_profile_rejected',
        'av_dynamic_824_compat_wire_generation_isolation',
        'av_dynamic_824_compat_layer_separation'
    )
    'semantics' = @(
        'av_dynamic_824_semantics_pack_source_stays_readable',
        'av_dynamic_824_semantics_two_packs_two_dyns',
        'av_dynamic_824_semantics_take_share_dyn_still_takeable',
        'av_dynamic_824_semantics_is_false_never_traps_dyn_usable',
        'av_dynamic_824_semantics_same_width_different_name_unequal',
        'av_dynamic_824_semantics_unknown_type_operand_rejected',
        'av_dynamic_824_semantics_frozen_payload_carries_no_authority',
        'av_dynamic_824_semantics_f64_bits_preserved_through_dyn',
        'av_dynamic_824_semantics_f32_bits_preserved_through_dyn',
        'av_dynamic_824_semantics_int_extremes_and_id_collision_stay_integers',
        'av_dynamic_824_semantics_restored_handles_detached',
        'av_dynamic_824_semantics_shared_dag_pack_ok_freeze_rejects',
        'av_dynamic_824_semantics_list_of_fn_not_declarable',
        'av_dynamic_824_semantics_record_with_fn_not_declarable',
        'av_dynamic_824_semantics_record_with_place_not_declarable',
        'av_dynamic_824_semantics_place_slot_pack_rejected',
        'av_dynamic_824_semantics_nested_dyn_rejected',
        'av_dynamic_824_semantics_portable_admission_two_domains_and_refusals',
        'av_dynamic_824_semantics_unfreeze_fault_retry_clean',
        'av_dynamic_824_semantics_production_same_source_byte_equivalence'
    )
}

if ($GuardSelfTest) {
    # Replay the validator controls on synthetic outputs: every bad shape
    # must throw (and the good shape must not).
    $good = "test x ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored"
    Assert-Selected -Output $good -Bin 's' -Filter '' -Label 'guard-good' -RequiredIds @('x')
    foreach ($bad in @(
        @{ o = 'no result line'; ids = @() },
        @{ o = "test result: ok. 0 passed; 0 failed; 0 ignored"; ids = @() },
        @{ o = "t`ntest result: ok. 1 passed; 0 failed; 1 ignored"; ids = @() },
        @{ o = "test x ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored"; ids = @('missing') },
        @{ o = "test x ... ok`ntest x ... ok`ntest result: ok. 2 passed; 0 failed; 0 ignored"; ids = @('x') }
    )) {
        $threw = $false
        try {
            Assert-Selected -Output $bad.o -Bin 's' -Filter '' -Label 'guard-bad' -RequiredIds $bad.ids
        }
        catch { $threw = $true }
        if (-not $threw) {
            throw "GuardSelfTest: a bad output shape passed the validator: $($bad.o)"
        }
    }
    Write-Host 'guard self-test: all controls behave'
    exit 0
}

switch ($Scope) {
    'first-case' {
        Run-Probe-Tests -Bin 'av_dynamic_824_first_case' -Label 'first-case' -RequiredIds $REQ['first-case']
    }
    'semantics' {
        # T-03: the full §5 support surface (V-02/03/04/05/11/12/14 axes)
        # through the real gates, plus the same-source harness module axis.
        Run-Probe-Tests -Bin 'av_dynamic_824_semantics' -Label 'semantics' -RequiredIds $REQ['semantics']
        Run-Harness-Tests -Label 'semantics-harness'
    }
    'admission' {
        # T-04: the untrusted-builder admission negatives (verify-gate
        # rejections; nothing executes).
        Run-Probe-Tests -Bin 'av_dynamic_824_first_case' -Filter 'av_dynamic_824_admission_negatives' -Label 'admission' -RequiredIds $REQ['admission']
    }
    'resources' {
        # T-04: budgets/faults/cleanup states + the parent-sampled worker
        # memory hard gate (V-06/V-07 axes).
        Run-Probe-Tests -Bin 'av_dynamic_824_resources' -Label 'resources' -RequiredIds $REQ['resources']
    }
    'cache' {
        # T-05: same-store restore identity (A23/A24), the R14/R15/R16
        # identity bindings, tampered-payload refusals, wire-generation
        # isolation and layer separation (V-08 axes).
        Run-Probe-Tests -Bin 'av_dynamic_824_cache_compat' -Label 'cache' -RequiredIds $REQ['cache']
    }
    'compatibility' {
        # V-09: the PRIOR profiles replay unchanged through their own
        # frozen corpora (798/817/818/820/821/822); the dynamic wire is a
        # new AVAD generation the old consumers reject by design.
        Run-Probe-Tests -Bin 'av_owned_821_first_case' -Label 'compatibility: 821 first-case'
        Run-Probe-Tests -Bin 'av_owned_821_cache_compat' -Label 'compatibility: 821 cache'
        Run-Probe-Tests -Bin 'av_collections_822_first_case' -Label 'compatibility: 822 first-case'
        Run-Lang-Tests -Filter 'av_owned_821' -Label 'compatibility: 821 corpus' -RequiredIds @(
            'tests::av_owned_821_tests::av_owned_821_corpus_green',
            'tests::av_owned_821_tests::av_owned_821_negative_oracle_fails',
            'tests::av_owned_821_tests::av_owned_821_worker_faults_are_infra'
        )
        Run-Lang-Tests -Filter 'av_collections_822' -Label 'compatibility: 822 corpus' -RequiredIds @(
            'tests::av_collections_822_tests::av_collections_822_corpus_green',
            'tests::av_collections_822_tests::av_collections_822_negative_oracle_fails',
            'tests::av_collections_822_tests::av_collections_822_worker_faults_are_infra'
        )
        Run-Lang-Tests -Filter 'ac_av_probe' -Label 'compatibility: 798/820 corpus'
        Run-Lang-Tests -Filter 'cte_probe' -Label 'compatibility: 817 corpus'
        Run-Lang-Tests -Filter 'ac_mn_probe' -Label 'compatibility: 818 corpus' -RequiredIds @(
            'tests::ac_mn_probe_tests::ac_mn_probe_corpus_executes_green',
            'tests::ac_mn_probe_tests::ac_mn_probe_negative_oracle_fails',
            'tests::ac_mn_probe_tests::mir_native_adapter_bad_actual_oracles_are_red',
            'tests::ac_mn_probe_tests::mir_native_adapter_diagnosed_rejection_faces',
            'tests::ac_mn_probe_tests::mir_native_adapter_edit_source_materializes_and_judges',
            'tests::ac_mn_probe_tests::mir_native_adapter_restart_session_reuses_cache_across_processes'
        )
    }
    'autodown' {
        Run-Lang-Tests -Filter 'av_dynamic_824' -Label 'autodown' -RequiredIds $REQ['autodown']
    }
    'acceptance' {
        # Replay every implemented scope in child processes (isolated
        # working directories, same shapes the docs promise).
        foreach ($s in @('first-case', 'semantics', 'admission', 'resources', 'cache', 'compatibility', 'autodown')) {
            & pwsh -File $PSCommandPath -Scope $s
            if ($LASTEXITCODE -ne 0) {
                throw "acceptance: scope $s failed"
            }
        }
    }
}
Write-Host "PLAN-824 verify [$Scope]: PASS"
