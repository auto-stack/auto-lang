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
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet(
        "first-case", "semantics", "admission", "resources",
        "cache", "compatibility", "autodown", "acceptance"
    )]
    [string]$Scope
)

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $repo "experimental/ac-av-probe/Cargo.toml"

function Fail([string]$msg) {
    Write-Host "[verify-av-data-820] FAIL: $msg" -ForegroundColor Red
    exit 1
}

# Run one cargo test selection; fails on non-zero exit, any red test, or a
# zero-selection run (a 0-run can never be green). Tiers that target
# workspace members OUTSIDE the probe workspace pass their own manifest.
function Run-Tier([string]$what, [string[]]$cargoArgs, [string[]]$filters, [string]$mft) {
    $useManifest = if ($mft) { $mft } else { $manifest }
    $all = @("test", "--manifest-path", $useManifest) + $cargoArgs
    if ($filters -and $filters.Count -gt 0) { $all += "--"; $all += $filters }
    Write-Host "[verify-av-data-820] $what : cargo $($all -join ' ')"
    $out = & cargo @all 2>&1
    $code = $LASTEXITCODE
    $text = ($out | Out-String)
    if ($code -ne 0) { Fail "$what red (exit $code):`n$text" }
    # Zero-selection guard over ALL targets in the run (a multi-target
    # invocation legitimately shows 0-line targets; the TOTAL must be >0).
    $total = 0
    foreach ($m in [regex]::Matches($text, "(\d+) passed;")) { $total += [int]$m.Groups[1].Value }
    if ($total -eq 0) { Fail "$what selected ZERO tests — a 0-run can never be green" }
    if ($text -match "[1-9]\d* failed") { Fail "$what has failing tests:`n$text" }
}

switch ($Scope) {
    "first-case" {
        Run-Tier "first-case" @("-p", "auto-av-probe", "--test", "av_data_820") @()
        Write-Host "[verify-av-data-820] first-case PASS" -ForegroundColor Green
        exit 0
    }
    "semantics" {
        Run-Tier "semantics" @("-p", "auto-av-probe", "--test", "av_data_820_semantics") @()
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
        )
        Write-Host "[verify-av-data-820] admission PASS" -ForegroundColor Green
        exit 0
    }
    "resources" {
        Run-Tier "resources" @("-p", "auto-av-probe", "--test", "av_data_820_resources") @()
        Run-Tier "memory-envelope" @("-p", "auto-av-probe", "--test", "av_data_820") @("av_data_820_worker_memory_envelope_b_line")
        Write-Host "[verify-av-data-820] resources PASS" -ForegroundColor Green
        exit 0
    }
    "cache" {
        Run-Tier "cache" @("-p", "auto-av-probe", "--test", "av_data_820_cache_compat") @(
            "av_data_820_cache_changed_types",
            "av_data_820_cache_good_hash_bad_payload",
            "av_data_820_data_payload_under_core_envelope"
        )
        Run-Tier "store-restore" @("-p", "auto-av-probe", "--test", "av_data_820") @("av_data_820_store_restore_new_process")
        Write-Host "[verify-av-data-820] cache PASS" -ForegroundColor Green
        exit 0
    }
    "compatibility" {
        Run-Tier "compatibility" @("-p", "auto-av-probe", "--test", "av_data_820_cache_compat") @(
            "av_data_820_old_consumer",
            "av_data_820_old_manifest_restore"
        )
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
        Run-Tier "first-case" @("-p", "auto-av-probe", "--test", "av_data_820") @()
        Run-Tier "semantics" @("-p", "auto-av-probe", "--test", "av_data_820_semantics") @()
        Run-Tier "resources" @("-p", "auto-av-probe", "--test", "av_data_820_resources") @()
        Run-Tier "cache-compat" @("-p", "auto-av-probe", "--test", "av_data_820_cache_compat") @()
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
