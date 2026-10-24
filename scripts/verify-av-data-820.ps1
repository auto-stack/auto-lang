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
# F820-R2-02 guards: a tier is green only when its REQUIRED test ids all
# executed AND passed (`test <id> ... ok`), NOTHING was ignored, no test
# failed, the ok-line count equals the summed `passed` totals (no padded
# or duplicated accounting), and the total is > 0. `-GuardSelfTest` runs
# the SAME judgment over synthetic outputs (positive + negative controls)
# and exits non-zero if any guard is blind.
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

# The tier-output judgment (F820-R2-02): returns the violation list
# (empty = clean) so the production path and the self-test share ONE
# implementation. Clean requires: required ids executed and ok, zero
# ignored, zero failed, ok-line count == summed passed totals, total > 0.
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
    $okLines = [regex]::Matches($text, "(?m)^test\s+\S+\s+\.\.\.\s+ok\s*$").Count
    if ($okLines -ne $total) { $v += "accounting mismatch: $total passed vs $okLines ok-lines" }
    foreach ($id in $required) {
        if ($text -notmatch "(?m)^test\s+$([regex]::Escape($id))\s+\.\.\.\s+ok\s*$") {
            $v += ("missing required test {0}" -f $id)
        }
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

# Run one cargo test selection under the tier guard; fails on non-zero
# exit, any red/ignored/missing case, or accounting mismatch. Tiers that
# target workspace members OUTSIDE the probe workspace pass their own
# manifest.
function Run-Tier(
    [string]$what,
    [string[]]$cargoArgs,
    [string[]]$filters,
    [string]$mft,
    [string[]]$required
) {
    $useManifest = if ($mft) { $mft } else { $manifest }
    $all = @("test", "--manifest-path", $useManifest) + $cargoArgs
    if ($filters -and $filters.Count -gt 0) { $all += "--"; $all += $filters }
    Write-Host "[verify-av-data-820] $what : cargo $($all -join ' ')"
    $out = & cargo @all 2>&1
    $code = $LASTEXITCODE
    $text = ($out | Out-String)
    if ($code -ne 0) { Fail "$what red (exit $code):`n$text" }
    Assert-TierOutput $what $text @($required)
}

# The plan's REQUIRED case ids per family (F820-R2-02: the id sets are the
# contract — a tier is green only when every id executed and passed).
$REQ_FIRST_CASE = @(
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
$REQ_SEMANTICS = @(
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
$REQ_ADMISSION = @(
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
$REQ_RESOURCES = @(
    "av_data_820_slot_budget_within_cap_control_succeeds",
    "av_data_820_slot_budget_over_cap_rejects_mid_run",
    "av_data_820_slot_budget_one_unit_under_rejects",
    "av_data_820_fault_at_nth_allocation_preserves_prior_state",
    "av_data_820_cancellation_keeps_peak_accounting",
    "av_data_820_layout_overflow_arm_is_defense_only"
)
$REQ_MEMORY = @(
    "av_data_820_worker_memory_envelope_b_line",
    "av_data_820_worker_peak_sampler_catches_transient"
)
$REQ_CACHE = @(
    "av_data_820_cache_changed_types_is_clean_miss_not_stale_reuse",
    "av_data_820_cache_good_hash_bad_payload_rejected",
    "av_data_820_data_payload_under_core_envelope_identity_rejected"
)
$REQ_COMPATIBILITY = @(
    "av_data_820_old_consumer_replays_scalar_and_rejects_record_source",
    "av_data_820_old_manifest_restore_refuses_data_store"
)
$REQ_CORPUS = @("ac_av_probe_corpus_executes_green")
$REQ_V10 = @(
    "ac_av_data_probe_wrong_result_oracle_fails",
    "ac_av_data_probe_wrong_diag_oracle_fails",
    "ac_av_data_probe_skip_shape_is_not_green",
    "ac_av_data_probe_worker_faults_are_infra"
)

if ($GuardSelfTest) {
    # Guard self-test (F820-R2-02): the SAME judgment the tiers use,
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

    # Negative controls — each shape MUST produce a violation.
    $cases = @(
        @{ name = "ignored"; text = "test result: ok. 1 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out"; req = @() },
        @{ name = "failed"; text = "test av_data_820_x ... FAILED`ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out"; req = @() },
        @{ name = "zero"; text = "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.0s"; req = @() },
        @{ name = "missing-required"; text = $ok2; req = @("av_data_820_v13_body_edit_is_function_level") },
        @{ name = "accounting"; text = "test av_data_820_a ... ok`ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"; req = @() }
    )
    foreach ($c in $cases) {
        $v = Get-TierViolations $c.text @($c.req)
        if ($v.Count -eq 0) { Fail "selftest: guard is BLIND to shape '$($c.name)'" }
        Write-Host ("[verify-av-data-820] selftest '{0}' caught: {1}" -f $c.name, ($v -join '; '))
    }
    Write-Host "[verify-av-data-820] GuardSelfTest PASS (2 positive + 5 negative controls)" -ForegroundColor Green
    exit 0
}

switch ($Scope) {
    "first-case" {
        Run-Tier "first-case" @("-p", "auto-av-probe", "--test", "av_data_820") @() "" $REQ_FIRST_CASE
        Write-Host "[verify-av-data-820] first-case PASS" -ForegroundColor Green
        exit 0
    }
    "semantics" {
        Run-Tier "semantics" @("-p", "auto-av-probe", "--test", "av_data_820_semantics") @() "" $REQ_SEMANTICS
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
        ) "" $REQ_ADMISSION
        Write-Host "[verify-av-data-820] admission PASS" -ForegroundColor Green
        exit 0
    }
    "resources" {
        Run-Tier "resources" @("-p", "auto-av-probe", "--test", "av_data_820_resources") @() "" $REQ_RESOURCES
        Run-Tier "memory-envelope" @("-p", "auto-av-probe", "--test", "av_data_820") @(
            "av_data_820_worker_memory_envelope_b_line",
            "av_data_820_worker_peak_sampler_catches_transient"
        ) "" $REQ_MEMORY
        Write-Host "[verify-av-data-820] resources PASS" -ForegroundColor Green
        exit 0
    }
    "cache" {
        Run-Tier "cache" @("-p", "auto-av-probe", "--test", "av_data_820_cache_compat") @(
            "av_data_820_cache_changed_types",
            "av_data_820_cache_good_hash_bad_payload",
            "av_data_820_data_payload_under_core_envelope"
        ) "" $REQ_CACHE
        Run-Tier "store-restore" @("-p", "auto-av-probe", "--test", "av_data_820") @("av_data_820_store_restore_new_process") "" @("av_data_820_store_restore_new_process")
        Write-Host "[verify-av-data-820] cache PASS" -ForegroundColor Green
        exit 0
    }
    "compatibility" {
        Run-Tier "compatibility" @("-p", "auto-av-probe", "--test", "av_data_820_cache_compat") @(
            "av_data_820_old_consumer",
            "av_data_820_old_manifest_restore"
        ) "" $REQ_COMPATIBILITY
        Write-Host "[verify-av-data-820] compatibility PASS" -ForegroundColor Green
        exit 0
    }
    "autodown" {
        # Formal AutoDown: the committed corpus (builds the release worker
        # and judges parent-side) plus the V-10 negative/infra controls.
        Run-Tier "corpus" @("-p", "auto-lang", "--features", "test-cases", "--lib", "ac_av_probe_corpus_executes_green") @() (Join-Path $repo "Cargo.toml") $REQ_CORPUS
        Run-Tier "v10-negatives" @("-p", "auto-lang", "--features", "test-cases", "--lib") @("ac_av_data_probe") (Join-Path $repo "Cargo.toml") $REQ_V10
        Write-Host "[verify-av-data-820] autodown PASS" -ForegroundColor Green
        exit 0
    }
    "acceptance" {
        # Aggregate: every family + core + corpus + negatives + check.
        Run-Tier "first-case" @("-p", "auto-av-probe", "--test", "av_data_820") @() "" $REQ_FIRST_CASE
        Run-Tier "semantics" @("-p", "auto-av-probe", "--test", "av_data_820_semantics") @() "" $REQ_SEMANTICS
        Run-Tier "resources" @("-p", "auto-av-probe", "--test", "av_data_820_resources") @() "" $REQ_RESOURCES
        Run-Tier "cache-compat" @("-p", "auto-av-probe", "--test", "av_data_820_cache_compat") @() "" @()
        Run-Tier "core" @("-p", "auto-av-core") @() "" @()
        Run-Tier "incremental-v13" @("-p", "auto-av-probe", "--test", "av_data_820_incremental") @() "" @()
        Run-Tier "corpus" @("-p", "auto-lang", "--features", "test-cases", "--lib", "ac_av_probe_corpus_executes_green") @() (Join-Path $repo "Cargo.toml") $REQ_CORPUS
        Run-Tier "v10-negatives" @("-p", "auto-lang", "--features", "test-cases", "--lib") @("ac_av_data_probe") (Join-Path $repo "Cargo.toml") $REQ_V10
        & cargo check --manifest-path $manifest --workspace 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0) { Fail "probe workspace check red" }
        Write-Host "[verify-av-data-820] acceptance PASS" -ForegroundColor Green
        exit 0
    }
}
