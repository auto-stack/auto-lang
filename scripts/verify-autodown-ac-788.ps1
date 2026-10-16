# PLAN-788 verification gate (T-08, R2 rewrite: per-family required IDs,
# zero-ignored, no cross-family substitution — review-r2 F08).
#   -Scope acceptance : every V-matrix family with REQUIRED test IDs, ignored=0
#   -Scope selftest   : parser unit tests over canned outputs (the parsing layer
#                       itself is tested, including the shim counterexample)
#   -Scope closeout   : merge-owned full surfaces on the frozen commit
# Usage: powershell -NoProfile -File scripts\verify-autodown-ac-788.ps1 [-Scope acceptance|selftest|closeout]
param(
    [ValidateSet("acceptance", "selftest", "closeout")]
    [string]$Scope = "acceptance"
)

$ErrorActionPreference = "Continue"
$env:RUSTC_WRAPPER = ""
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$fails = 0

# ---- the parsing layer (pure, testable) -------------------------------------
# Returns $null when the cargo output satisfies the family contract:
#   exit 0, at least one "N passed; M failed; K ignored" summary, every
#   summary has failed==0 AND ignored==0, total passed >= MinPassed, and
#   every required test ID appears in the executed test lines.
function Test-CargoFamilyOutput {
    param([string]$Text, [int]$ExitCode, [string[]]$RequiredIds, [int]$MinPassed)
    if ($ExitCode -ne 0) { return "exit=$ExitCode" }
    $summaries = [regex]::Matches($Text, "(\d+) passed; (\d+) failed; (\d+) ignored")
    if ($summaries.Count -eq 0) { return "no test summaries parsed" }
    $totalPassed = 0
    foreach ($s in $summaries) {
        $p = [int]$s.Groups[1].Value; $f = [int]$s.Groups[2].Value; $ig = [int]$s.Groups[3].Value
        if ($f -ne 0) { return "failed=$f in a summary" }
        if ($ig -ne 0) { return "ignored=$ig in a summary (skips are not accepted)" }
        $totalPassed += $p
    }
    if ($totalPassed -lt $MinPassed) { return "totalPassed=$totalPassed < required $MinPassed" }
    foreach ($id in $RequiredIds) {
        if ($Text -notmatch [regex]::Escape($id)) { return "required id missing: $id" }
    }
    return $null
}

function Run-CargoFamily {
    param([string]$Name, [string[]]$Argv, [string[]]$RequiredIds, [int]$MinPassed)
    Write-Host "== $Name" -ForegroundColor Cyan
    $text = (& cargo @Argv 2>&1 | Out-String)
    $code = $LASTEXITCODE
    $why = Test-CargoFamilyOutput -Text $text -ExitCode $code -RequiredIds $RequiredIds -MinPassed $MinPassed
    if ($why) {
        $script:fails++
        Write-Host "   FAMILY FAIL ($why)" -ForegroundColor Red
        Write-Host $text -ForegroundColor Red
        return
    }
    Write-Host "   family ok" -ForegroundColor Green
}

if ($Scope -eq "selftest") {
    $cases = @(
        @{ name = "fully-legal"; text = "test a ... ok`ntest result: ok. 5 passed; 0 failed; 0 ignored; 0 measured`n"; exit = 0; ids = @("test a"); min = 1; want = $null },
        @{ name = "zero-selected"; text = "running 0 tests`ntest result: ok. 0 passed; 0 failed; 0 ignored"; exit = 0; ids = @(); min = 1; want = "totalPassed=0 < required 1" },
        @{ name = "partially-skipped"; text = "test result: ok. 4 passed; 0 failed; 999 ignored"; exit = 0; ids = @(); min = 1; want = "ignored=999 in a summary (skips are not accepted)" },
        @{ name = "nonzero-exit"; text = "test result: FAILED. 1 passed; 1 failed; 0 ignored"; exit = 101; ids = @(); min = 1; want = "exit=101" },
        @{ name = "family-missing-id"; text = "test unrelated ... ok`ntest result: ok. 3 passed; 0 failed; 0 ignored"; exit = 0; ids = @("test needed"); min = 1; want = "required id missing: test needed" },
        @{ name = "count-substitution"; text = "test result: ok. 1 passed; 0 failed; 0 ignored"; exit = 0; ids = @(); min = 5; want = "totalPassed=1 < required 5" }
    )
    foreach ($c in $cases) {
        $why = Test-CargoFamilyOutput -Text $c.text -ExitCode $c.exit -RequiredIds $c.ids -MinPassed $c.min
        $ok = if ($null -eq $why) { $null -eq $c.want } else { $why -eq $c.want }
        if ($ok) {
            Write-Host "selftest $($c.name): ok" -ForegroundColor Green
        } else {
            $script:fails++
            Write-Host "selftest $($c.name): FAIL (got '$why', want '$($c.want)')" -ForegroundColor Red
        }
    }
    if ($fails -gt 0) { throw "selftest: $fails case(s) failed" }
    Write-Host "PLAN-788 selftest: ALL PASS" -ForegroundColor Green
    exit 0
}

if ($Scope -eq "acceptance") {
    # V-01/07/15 contract face
    Run-CargoFamily "auto-test-case (V-01)" @("test", "-p", "auto-test-case") `
        @("contract_v2_report_json_round_trips_through_standard_parser", "contract_v2_cache_declaration_requires_participation", "wrong_schema_is_rejected", "minimal_case_decodes") 86
    # V-02/05/06 executor level
    Run-CargoFamily "ac-core link_capture_ (V-05/06)" @("test", "--manifest-path", "experimental/ac-core/Cargo.toml", "--test", "link_capture_") `
        @("process_budget_deadline_kills_the_tree", "process_budget_output_cap_terminates_immediately", "native_bytes_invalid_utf8_round_trips_exactly", "native_bytes_nul_and_crlf_preserved", "containment_failure_replays_through_captured_path") 7
    # V-02/04/07/08/09/10 worker faces
    Run-CargoFamily "worker protocol+cache+r1+syntax" @("test", "--manifest-path", "experimental/ac-case-worker/Cargo.toml", "--test", "feasibility_t01_", "--test", "protocol_t03_", "--test", "cache_t05_", "--test", "review_r1_", "--test", "syntax_t06_") `
        @("ac_stage_parse_boundary_attributed", "native_cache_fresh_then_object_hit", "native_invalidation_body_edit_full_miss", "r1_budget_one_ms_total_is_enforced_not_ignored", "r1_product_run_invalid_exe_is_infra_error", "r1_backend_missing_entry_typed_through_store_path", "syntax_case_edit_equivalence_and_counters", "revision_trace_sequential_edits_advance") 28
    # V-08/09/10 store faces
    Run-CargoFamily "probe plan788_r1_ + 783 regressions" @("test", "--manifest-path", "experimental/ac-artifact-probe/Cargo.toml", "--release", "--test", "plan788_r1_", "--test", "native_resume_", "--test", "invalidation_", "--test", "identity_", "--test", "concurrency_", "--test", "store_boundary_") `
        @("r1_compiler_recipe_change_invalidates_hir_and_object", "r1_observed_spans_carry_real_frontend_boundaries", "native_resume_four_fixtures_fresh_restart_hirhit_and_source_absent") 20
    # V-03/04/11/13/14 runner integration
    Run-CargoFamily "auto-lang ac_case_tests (V-03/04/11/14)" @("test", "-p", "auto-lang", "--features", "test-cases", "--lib", "ac_case_tests") `
        @("ac_native_run_exit_oracle_and_red_sibling", "ac_native_typed_negative_stage_attribution", "ac_product_handoff_run_consumes_real_build", "r1_consumer_invalid_exe_is_error_not_pass", "r1_response_admission_table", "syntax_parse_and_edit_dispatch_with_goldens", "ac_native_suite_full_run_and_ledger_reconcile") 13
    Run-CargoFamily "auto-lang v1 regression" @("test", "-p", "auto-lang", "--features", "test-cases", "--lib", "test_cases_tests") `
        @("pilot_suite_full_run", "stale_check_detects_oracle_drift") 26
    # V-12/V-13 turnaround + hard gates
    Run-CargoFamily "worker turnaround (V-12/13 + gates)" @("test", "--manifest-path", "experimental/ac-case-worker/Cargo.toml", "--release", "--test", "turnaround_t08_", "--", "--nocapture") `
        @("turnaround_native_six_scenarios_p95_gates", "turnaround_syntax_gate_and_revision_stream") 2

    if ($fails -gt 0) { throw "acceptance: $fails family/ies failed or refused" }
    Write-Host "PLAN-788 acceptance: ALL PASS" -ForegroundColor Green
    exit 0
}

# closeout (merge-owned): frozen-commit full surfaces.
Run-CargoFamily "auto-lang full (test-cases)" @("test", "-p", "auto-lang", "--features", "test-cases") @() 1
Run-CargoFamily "root cargo t" @("t") @() 1
Run-CargoFamily "root cargo tv" @("tv") @() 1
Run-CargoFamily "root cargo tt" @("tt") @() 1
Run-CargoFamily "worker full (release)" @("test", "--manifest-path", "experimental/ac-case-worker/Cargo.toml", "--release") @() 1
Run-CargoFamily "probe full (release)" @("test", "--manifest-path", "experimental/ac-artifact-probe/Cargo.toml", "--release") @() 1
Run-CargoFamily "ac-syntax runtime (release)" @("test", "--manifest-path", "experimental/ac-syntax/runtime/Cargo.toml", "--release") @() 1

if ($fails -gt 0) { throw "closeout: $fails surface(s) failed" }
Write-Host "PLAN-788 closeout: ALL PASS" -ForegroundColor Green
