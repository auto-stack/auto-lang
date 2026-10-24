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
    Write-Host "[verify-av-data-820] GuardSelfTest PASS (3 positive + 7 negative controls)" -ForegroundColor Green
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
