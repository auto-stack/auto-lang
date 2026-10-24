# PLAN-820 verification script — av.data/1 profile gates.
# Scopes (one per invocation):
#   first-case    — T-02 first-case family (whole path + bits + restore).
#   semantics     — T-03 semantics + type-equivalence grid (V-01/02/05/06/11).
#   admission     — T-03/T-04 rejection families (shape/resource/kind).
#   resources     — T-04 V-07 boundaries (budget/fault/cancel + B-line).
#   cache         — T-05 V-08 (dependency-partitioned restores, tamper).
#   compatibility — T-05 V-09 (old-consumer replay/refusals).
#   autodown      — T-02/T-06 formal AutoDown (corpus + V-10 negatives).
#   acceptance    — every family above + core + fmt/check (aggregate gate).
#
# F820-R2-02/R3 guards: each tier's REQUIRED id set is DERIVED from the
# same cargo selection via `--list` — the actual complete id set,
# fully-qualified on the root workspace (`tests::<module>::<name>`),
# auto-adapting to renames and additions. A tier is green only when every
# listed id executed and passed (`test <id> ... ok`), NOTHING was ignored,
# no test failed, no id is missing/duplicated/unexpected, the ok-line count
# equals the summed `passed` totals, and the total is > 0.
# `-GuardSelfTest` runs the SAME judgment over synthetic outputs (positive
# + negative controls) and exits non-zero if any guard is blind.
param(
    [Parameter(Mandatory = $true, ParameterSetName = "scope")]
    [ValidateSet(
        "first-case", "semantics", "admission", "resources",
        "cache", "compatibility", "autodown", "acceptance"
    )]
    [string]$Scope,
    [Parameter(Mandatory = $true, ParameterSetName = "selftest")]
    [switch]$GuardSelfTest
)

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $repo "experimental/ac-av-probe/Cargo.toml"

function Fail([string]$msg) {
    Write-Host "[verify-av-data-820] FAIL: $msg" -ForegroundColor Red
    exit 1
}

# The tier-output judgment (F820-R2-02/R3): returns the violation list
# (empty = clean) so the production path and the self-test share ONE
# implementation. Clean requires: every required id executed and ok, zero
# ignored, zero failed, ok ids are a SET (no duplicates) matching the
# required selection (nothing missing, nothing unexpected), ok-line count
# == summed passed totals, total > 0.
function Get-TierViolations(
    [string]$text,
    [string[]]$required
) {
    $v = @()
    if ($text -match "[1-9]\d* failed") { $v += "failed tests present" }
    foreach ($m in [regex]::Matches($text, "(\d+) ignored")) {
        if ([int]$m.Groups[1].Value -gt 0) {
            $v += "$($m.Groups[1].Value) ignored test(s) — ignored is never green"
        }
    }
    $total = 0
    foreach ($m in [regex]::Matches($text, "(\d+) passed;")) {
        $total += [int]$m.Groups[1].Value
    }
    if ($total -eq 0) { $v += "ZERO tests selected — a 0-run can never be green" }
    # The executed ok ids (set semantics): duplicates are padded accounting,
    # ids outside the selection are unexpected execution.
    $okIds = @(
        [regex]::Matches($text, "(?m)^test\s+(\S+)\s+\.\.\.\s+ok\s*$") |
            ForEach-Object { $_.Groups[1].Value }
    )
    if ($okIds.Count -ne $total) {
        $v += "accounting mismatch: $total passed vs $($okIds.Count) ok-lines"
    }
    $dupes = $okIds | Group-Object | Where-Object { $_.Count -gt 1 }
    foreach ($d in $dupes) { $v += "duplicate ok id '$($d.Name)' x$($d.Count)" }
    $okSet = @($okIds | Sort-Object -Unique)
    foreach ($id in $required) {
        if ($okSet -notcontains $id) { $v += ("missing required test {0}" -f $id) }
    }
    foreach ($id in $okSet) {
        if ($required -notcontains $id) { $v += ("unexpected ok test {0}" -f $id) }
    }
    return $v
}

function Assert-TierOutput(
    [string]$what,
    [string]$text,
    [string[]]$required
) {
    $v = Get-TierViolations $text @($required)
    if ($v.Count -gt 0) {
        Fail ("{0}: {1}:`n{2}" -f $what, ($v -join "; "), $text)
    }
}

# The CONTRACT case manifest (F820-R2-02/R4 close): an INDEPENDENT list of
# the plan's required case ids per tier — NOT generated from the verified
# object. The `--list` derivation alone cannot detect a required case that
# was deleted or cfg-excluded (listing and execution would both miss it
# and the tier would stay green); the contract anchors what MUST exist.
# A contract name matches a listed id exactly or as its trailing
# `::<name>` segment, so fully-qualified root ids bind without hardcoding
# prefixes, and renames stay an explicit manifest edit.
$TIER_CONTRACT = @{
    "first-case" = @(
        "av_data_820_first_case_whole_path",
        "av_data_820_first_case_native_independent_comparison",
        "av_data_820_first_case_wire_round_trip",
        "av_data_820_record_copy_keeps_original",
        "av_data_820_wide_scalar_bit_exact_roundtrip",
        "av_data_820_entry_rejects_wrong_kind_and_record",
        "av_data_820_same_width_owner_mismatch_rejected",
        "av_data_820_wide_scalar_arith_rejected",
        "av_data_820_type_cycle_and_empty_record_rejected",
        "av_data_820_wire_family_isolation",
        "av_data_820_store_restore_new_process",
        "av_data_820_worker_memory_envelope_b_line",
        "av_data_820_worker_peak_sampler_catches_transient"
    )
    "semantics" = @(
        "av_data_820_ctor_evaluates_in_source_order_not_declaration_order",
        "av_data_820_ctor_source_order_trace_is_ba",
        "av_data_820_unexecuted_branch_trap_stays_latent",
        "av_data_820_swap_assignment_reads_before_writes",
        "av_data_820_nested_field_write_hits_original_root",
        "av_data_820_ctor_missing_and_duplicate_fields_rejected",
        "av_data_820_record_slot_uninitialized_read_rejected",
        "av_data_820_bits_survive_call_chain",
        "av_data_820_type_equivalence_grid_all_consumption_points",
        "av_data_820_duplicate_primitive_entries_normalize",
        "av_data_820_record_field_count_and_path_depth_are_resource",
        "av_data_820_unit_kind_never_executes",
        "av_data_820_instruction_contract_round_trip_all_opcodes"
    )
    "admission" = @(
        "av_data_820_type_equivalence_grid_all_consumption_points",
        "av_data_820_ctor_missing_and_duplicate_fields_rejected",
        "av_data_820_record_slot_uninitialized_read_rejected",
        "av_data_820_record_field_count_and_path_depth_are_resource",
        "av_data_820_unit_kind_never_executes",
        "av_data_820_wide_scalar_arith_rejected",
        "av_data_820_type_cycle_and_empty_record_rejected",
        "av_data_820_entry_rejects_wrong_kind_and_record",
        "av_data_820_wire_family_isolation"
    )
    "resources" = @(
        "av_data_820_slot_budget_within_cap_control_succeeds",
        "av_data_820_slot_budget_over_cap_rejects_mid_run",
        "av_data_820_slot_budget_one_unit_under_rejects",
        "av_data_820_fault_at_nth_allocation_preserves_prior_state",
        "av_data_820_cancellation_keeps_peak_accounting",
        "av_data_820_layout_overflow_real_input_rejected",
        "av_data_820_layout_work_shares_the_verify_budget"
    )
    "memory-envelope" = @(
        "av_data_820_worker_memory_envelope_b_line",
        "av_data_820_worker_peak_sampler_catches_transient"
    )
    "cache" = @(
        "av_data_820_cache_changed_types_is_clean_miss_not_stale_reuse",
        "av_data_820_cache_good_hash_bad_payload_rejected",
        "av_data_820_data_payload_under_core_envelope_identity_rejected"
    )
    "store-restore" = @("av_data_820_store_restore_new_process")
    "compatibility" = @(
        "av_data_820_old_consumer_replays_scalar_and_rejects_record_source",
        "av_data_820_old_manifest_restore_refuses_data_store"
    )
    "corpus" = @("ac_av_probe_corpus_executes_green")
    "v10-negatives" = @(
        "ac_av_data_probe_wrong_result_oracle_fails",
        "ac_av_data_probe_wrong_diag_oracle_fails",
        "ac_av_data_probe_skip_shape_is_not_green",
        "ac_av_data_probe_worker_faults_are_infra",
        "ac_av_data_probe_wrong_bits_oracle_fails",
        "ac_av_data_probe_wrong_type_tag_oracle_fails",
        "ac_av_data_probe_unknown_typed_fixture_is_error"
    )
    "cache-compat" = @(
        "av_data_820_cache_changed_types_is_clean_miss_not_stale_reuse",
        "av_data_820_cache_good_hash_bad_payload_rejected",
        "av_data_820_data_payload_under_core_envelope_identity_rejected",
        "av_data_820_old_consumer_replays_scalar_and_rejects_record_source",
        "av_data_820_old_manifest_restore_refuses_data_store"
    )
    "incremental-v13" = @(
        "av_data_820_v13_body_edit_is_function_level",
        "av_data_820_v13_dependency_changes_invalidate_precisely",
        "av_data_820_v13_forced_degradations_redden_the_gate",
        "av_data_820_v13_old_snapshot_survives_for_comparison",
        "av_data_820_v13_multi_function_edit_is_explicit_non_fast",
        "av_data_820_v13_failed_edit_installs_nothing",
        "av_data_820_v13_metering_injection_controls_redden_the_gate",
        "av_data_820_v13_caps_grant_syncs_the_hot_frontend",
        "av_data_820_v13_caps_revoke_is_controlled_and_gates_hot_edits"
    )
    # "core" is a support tier (the shared core suite) — guarded by the
    # listing/execution rules only, no plan contract of its own.
}

# Contract coverage: every contract case must be present in the DERIVED
# listing (exact id or trailing ::<name> segment). Returns the misses.
function Get-ContractMisses(
    [string[]]$listedIds,
    [string[]]$contract
) {
    $misses = @()
    foreach ($name in $contract) {
        $covered = $false
        foreach ($id in $listedIds) {
            if ($id -eq $name -or $id.EndsWith("::$name")) { $covered = $true; break }
        }
        if (-not $covered) { $misses += ("contract case {0} missing from the tier listing (deleted or excluded)" -f $name) }
    }
    return $misses
}

# Run one cargo test selection under the tier guard. The REQUIRED id set
# is DERIVED from the same selection via `--list` (F820-R2-02/R3): the
# actual complete id set — fully-qualified on the root workspace, and
# self-maintaining as cases are renamed or added. Tiers that target
# workspace members OUTSIDE the probe workspace pass their own manifest.
function Run-Tier(
    [string]$what,
    [string[]]$cargoArgs,
    [string[]]$filters,
    [string]$mft
) {
    $useManifest = if ($mft) { $mft } else { $manifest }
    $sel = @()
    if ($filters -and $filters.Count -gt 0) { $sel += "--"; $sel += $filters }
    # `--list` must reach LIBTEST: after the filter positionals when a
    # selection exists, behind an explicit `--` separator otherwise (a
    # bare `--list` would be parsed by cargo itself).
    $listTail = if ($sel.Count -gt 0) { $sel + @("--list") } else { @("--", "--list") }
    $listArgs = @("test", "--manifest-path", $useManifest) + $cargoArgs + $listTail
    Write-Host "[verify-av-data-820] $what : cargo $($listArgs -join ' ')"
    $listOut = & cargo @listArgs 2>&1
    if ($LASTEXITCODE -ne 0) {
        Fail "$what listing red (exit $LASTEXITCODE):`n$(($listOut | Out-String))"
    }
    $listText = ($listOut | Out-String)
    $required = @(
        [regex]::Matches($listText, "(?m)^(.+): test\r?$") |
            ForEach-Object { $_.Groups[1].Value }
    )
    if ($required.Count -eq 0) {
        Fail "$what listed ZERO tests — a 0-selection tier can never be green"
    }
    # Independent contract oracle: the plan's required cases must ALL be
    # present in the listing — a case deleted or excluded from the binary
    # disappears from listing AND execution together; only this manifest
    # anchors it (F820-R2-02/R4).
    $contract = $TIER_CONTRACT[$what]
    if ($contract) {
        $misses = Get-ContractMisses $required $contract
        if ($misses.Count -gt 0) {
            Fail ("{0}: {1}" -f $what, ($misses -join "; "))
        }
    }
    $all = @("test", "--manifest-path", $useManifest) + $cargoArgs + $sel
    Write-Host ("[verify-av-data-820] {0} : cargo {1} [required ids: {2}]" -f `
            $what, ($all -join ' '), $required.Count)
    $out = & cargo @all 2>&1
    $code = $LASTEXITCODE
    $text = ($out | Out-String)
    if ($code -ne 0) { Fail "$what red (exit $code):`n$text" }
    Assert-TierOutput $what $text $required
}

if ($GuardSelfTest) {
    # Guard self-test (F820-R2-02/R3): the SAME judgment the tiers use,
    # exercised over synthetic outputs. Clean inputs must yield ZERO
    # violations; every degraded shape MUST be caught.
    $ok2 = @"
running 2 tests
test av_data_820_worker_memory_envelope_b_line ... ok
test av_data_820_worker_peak_sampler_catches_transient ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.0s
"@
    $req2 = @(
        "av_data_820_worker_memory_envelope_b_line",
        "av_data_820_worker_peak_sampler_catches_transient"
    )

    # Positive controls.
    if ((Get-TierViolations $ok2 $req2).Count -ne 0) {
        Fail "selftest: a clean run was flagged"
    }
    # Multi-target aggregate: a sibling 0-selection target line is legal
    # when the SUM still matches the ok-line count.
    $multi = $ok2 + "`ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.0s"
    if ((Get-TierViolations $multi $req2).Count -ne 0) {
        Fail "selftest: a clean multi-target run was flagged"
    }
    # Contract coverage: fully-qualified listing covers the manifest...
    $listedRoot = @(
        "tests::ac_av_probe_tests::ac_av_data_probe_wrong_result_oracle_fails",
        "tests::ac_av_probe_tests::ac_av_data_probe_wrong_diag_oracle_fails"
    )
    if ((Get-ContractMisses $listedRoot @("ac_av_data_probe_wrong_result_oracle_fails", "ac_av_data_probe_wrong_diag_oracle_fails")).Count -ne 0) {
        Fail "selftest: a fully-covered contract listing was flagged"
    }
    # ...and a listing that LOST a contract case is caught even though the
    # execution of what remains is perfectly green (the R4 deletion shape).
    if ((Get-ContractMisses $listedRoot @("ac_av_data_probe_wrong_result_oracle_fails", "ac_av_data_probe_skip_shape_is_not_green")).Count -eq 0) {
        Fail "selftest: contract coverage is BLIND to a deleted required case"
    }
    # Fully-qualified root ids parse identically (the R3 false-miss shape).
    $root = @"
running 1 test
test tests::ac_av_probe_tests::ac_av_probe_corpus_executes_green ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 5.0s
"@
    if ((Get-TierViolations $root @("tests::ac_av_probe_tests::ac_av_probe_corpus_executes_green")).Count -ne 0) {
        Fail "selftest: a fully-qualified root id run was flagged"
    }

    # Negative controls — each shape MUST produce a violation.
    $cases = @(
        @{ name = "ignored"; text = "test result: ok. 1 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out"; req = @("only_one") },
        @{ name = "failed"; text = "test av_data_820_x ... FAILED`ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"; req = @() },
        @{ name = "zero"; text = "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.0s"; req = @() },
        @{ name = "missing-family"; text = $ok2; req = @("av_data_820_v13_body_edit_is_function_level") },
        @{ name = "accounting"; text = "test av_data_820_a ... ok`ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"; req = @("av_data_820_a") },
        @{ name = "duplicate-ok"; text = "test av_data_820_a ... ok`ntest av_data_820_a ... ok`ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"; req = @("av_data_820_a") },
        @{ name = "unexpected-ok"; text = "test av_data_820_a ... ok`ntest unrelated_control ... ok`ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"; req = @("av_data_820_a") }
    )
    foreach ($c in $cases) {
        $v = Get-TierViolations $c.text @($c.req)
        if ($v.Count -eq 0) { Fail "selftest: guard is BLIND to shape '$($c.name)'" }
        Write-Host ("[verify-av-data-820] selftest '{0}' caught: {1}" -f $c.name, ($v -join '; '))
    }
    Write-Host "[verify-av-data-820] GuardSelfTest PASS (5 positive + 7 negative controls)" -ForegroundColor Green
    exit 0
}

switch ($Scope) {
    "first-case" {
        Run-Tier "first-case" @("-p", "auto-av-probe", "--test", "av_data_820") @() ""
        Write-Host "[verify-av-data-820] first-case PASS" -ForegroundColor Green
        exit 0
    }
    "semantics" {
        Run-Tier "semantics" @("-p", "auto-av-probe", "--test", "av_data_820_semantics") @() ""
        Write-Host "[verify-av-data-820] semantics PASS" -ForegroundColor Green
        exit 0
    }
    "admission" {
        Run-Tier "admission" @("-p", "auto-av-probe", "--test", "av_data_820_semantics", "--test", "av_data_820_resources", "--test", "av_data_820") @(
            "av_data_820_type_equivalence_grid",
            "av_data_820_ctor_missing_and_duplicate_fields_rejected",
            "av_data_820_record_slot_uninitialized_read_rejected",
            "av_data_820_record_field_count_and_path_depth_are_resource",
            "av_data_820_unit_kind_never_executes",
            "av_data_820_wide_scalar_arith_rejected",
            "av_data_820_type_cycle_and_empty_record_rejected",
            "av_data_820_entry_rejects_wrong_kind_and_record",
            "av_data_820_wire_family_isolation"
        ) ""
        Write-Host "[verify-av-data-820] admission PASS" -ForegroundColor Green
        exit 0
    }
    "resources" {
        Run-Tier "resources" @("-p", "auto-av-probe", "--test", "av_data_820_resources") @() ""
        Run-Tier "memory-envelope" @("-p", "auto-av-probe", "--test", "av_data_820") @(
            "av_data_820_worker_memory_envelope_b_line",
            "av_data_820_worker_peak_sampler_catches_transient"
        ) ""
        Write-Host "[verify-av-data-820] resources PASS" -ForegroundColor Green
        exit 0
    }
    "cache" {
        Run-Tier "cache" @("-p", "auto-av-probe", "--test", "av_data_820_cache_compat") @(
            "av_data_820_cache_changed_types",
            "av_data_820_cache_good_hash_bad_payload",
            "av_data_820_data_payload_under_core_envelope"
        ) ""
        Run-Tier "store-restore" @("-p", "auto-av-probe", "--test", "av_data_820") @("av_data_820_store_restore_new_process") ""
        Write-Host "[verify-av-data-820] cache PASS" -ForegroundColor Green
        exit 0
    }
    "compatibility" {
        Run-Tier "compatibility" @("-p", "auto-av-probe", "--test", "av_data_820_cache_compat") @(
            "av_data_820_old_consumer",
            "av_data_820_old_manifest_restore"
        ) ""
        Write-Host "[verify-av-data-820] compatibility PASS" -ForegroundColor Green
        exit 0
    }
    "autodown" {
        # Formal AutoDown: the committed corpus (builds the release worker
        # and judges parent-side) plus the V-10 negative/infra controls.
        Run-Tier "corpus" @("-p", "auto-lang", "--features", "test-cases", "--lib", "ac_av_probe_corpus_executes_green") @() (Join-Path $repo "Cargo.toml")
        Run-Tier "v10-negatives" @("-p", "auto-lang", "--features", "test-cases", "--lib") @("ac_av_data_probe") (Join-Path $repo "Cargo.toml")
        Write-Host "[verify-av-data-820] autodown PASS" -ForegroundColor Green
        exit 0
    }
    "acceptance" {
        # Aggregate: every family + core + corpus + negatives + check.
        Run-Tier "first-case" @("-p", "auto-av-probe", "--test", "av_data_820") @() ""
        Run-Tier "semantics" @("-p", "auto-av-probe", "--test", "av_data_820_semantics") @() ""
        Run-Tier "resources" @("-p", "auto-av-probe", "--test", "av_data_820_resources") @() ""
        Run-Tier "cache-compat" @("-p", "auto-av-probe", "--test", "av_data_820_cache_compat") @() ""
        Run-Tier "core" @("-p", "auto-av-core") @() ""
        Run-Tier "incremental-v13" @("-p", "auto-av-probe", "--test", "av_data_820_incremental") @() ""
        Run-Tier "corpus" @("-p", "auto-lang", "--features", "test-cases", "--lib", "ac_av_probe_corpus_executes_green") @() (Join-Path $repo "Cargo.toml")
        Run-Tier "v10-negatives" @("-p", "auto-lang", "--features", "test-cases", "--lib") @("ac_av_data_probe") (Join-Path $repo "Cargo.toml")
        & cargo check --manifest-path $manifest --workspace 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0) { Fail "probe workspace check red" }
        Write-Host "[verify-av-data-820] acceptance PASS" -ForegroundColor Green
        exit 0
    }
}
