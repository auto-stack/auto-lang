# PLAN-788 verification gate (T-08; R3 F06 rewrite: exact executed-ok ID
# parsing, per-binary binding + counts, nextest-aware closeout — a required
# test can no longer be satisfied by prose text, a name prefix, another
# binary's totals, or an assumed summary format).
#   -Scope acceptance : every V-matrix family with REQUIRED test IDs (exact,
#                       bound to their test binary), per-binary minimums,
#                       ignored=0 (named allowlist only)
#   -Scope selftest   : parser unit tests over canned outputs, including the
#                       prose-substitution and prefix-substitution negatives
#   -Scope closeout   : merge-owned full surfaces on the frozen commit; the
#                       root cargo t/tv/tt families run under NEXTTEST and are
#                       verified against nextest's actual output format
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

# ---- libtest parsing --------------------------------------------------------
# Splits cargo test output into per-binary segments ("     Running tests\x.rs
# (target\...)" headers), collecting each segment's executed-ok test names,
# ignored names and its "test result:" summary. Returns an array of
# @{ Binary; OkNames; IgnoredNames; Passed; Failed; Ignored }.
function Split-LibTestSegments {
    param([string]$Text)
    $segments = @()
    $cur = $null
    foreach ($line in ($Text -split "`r?`n")) {
        # -cmatch (CASE-SENSITIVE): libtest also prints "running N tests"
        # which must NOT be taken for cargo's "Running <binary>" header.
        if ($line -cmatch '^\s*Running\s+(?:unittests\s+|benchmarks\s+|doctests\s+|tests\\)?(\S+)') {
            if ($null -ne $cur) { $segments += ,$cur }
            $cur = @{ Binary = $Matches[1]; OkNames = @(); IgnoredNames = @(); Passed = 0; Failed = 0; Ignored = 0; HasSummary = $false }
        } elseif ($null -eq $cur) {
            # output before any Running header still belongs to a (single)
            # implicit target — cargo always prints the header, but be robust
            $cur = @{ Binary = "<implicit>"; OkNames = @(); IgnoredNames = @(); Passed = 0; Failed = 0; Ignored = 0; HasSummary = $false }
        }
        if ($line -match '^test (\S+) \.\.\. ok') {
            $cur.OkNames += $Matches[1]
        } elseif ($line -match '^test (\S+) \.\.\. ignored') {
            $cur.IgnoredNames += $Matches[1]
        } elseif ($line -match '(\d+) passed; (\d+) failed; (\d+) ignored') {
            $cur.Passed = [int]$Matches[1]; $cur.Failed = [int]$Matches[2]; $cur.Ignored = [int]$Matches[3]
            $cur.HasSummary = $true
        }
    }
    if ($null -ne $cur) { $segments += ,$cur }
    return $segments
}

# libtest family contract: exit 0; >=1 segment with a summary; failed==0 in
# every summary; ignored tests are an explicit named allowlist and match the
# summary counts; every RequiredId is an EXACT executed-ok name in the
# RequiredInBinary segment (if bound) or in exactly one segment (if not);
# per-binary minimums hold (a fat sibling binary cannot substitute a missing
# family), and the family total >= MinPassed.
function Test-LibTestFamily {
    param(
        [string]$Text, [int]$ExitCode,
        [string[]]$RequiredIds, [hashtable]$RequiredInBinary = @{},
        [int]$MinPassed, [hashtable]$MinPerBinary = @{},
        [string[]]$AllowedIgnored = @()
    )
    if ($ExitCode -ne 0) { return "exit=$ExitCode" }
    $segments = @(Split-LibTestSegments -Text $Text)
    $summarized = @($segments | Where-Object { $_.HasSummary })
    if ($summarized.Count -eq 0) { return "no test summaries parsed" }
    foreach ($s in $summarized) {
        if ($s.Failed -ne 0) { return "failed=$($s.Failed) in a summary ($($s.Binary))" }
        if ($s.Ignored -ne $s.IgnoredNames.Count) {
            return "summary ignored=$($s.Ignored) does not match named ignored tests ($($s.IgnoredNames.Count)) ($($s.Binary))"
        }
    }
    $allIgnored = @()
    foreach ($s in $summarized) { $allIgnored += $s.IgnoredNames }
    foreach ($n in $allIgnored) {
        if ($AllowedIgnored -notcontains $n) { return "ignored test not allowlisted: $n" }
    }
    $totalPassed = 0
    foreach ($s in $summarized) { $totalPassed += $s.Passed }
    if ($totalPassed -lt $MinPassed) { return "totalPassed=$totalPassed < required $MinPassed" }
    foreach ($bin in $MinPerBinary.Keys) {
        $seg = @($summarized | Where-Object { $_.Binary -like "*$bin*" })
        if ($seg.Count -eq 0) { return "required binary '$bin' produced no summary" }
        $p = 0
        foreach ($s in $seg) { $p += $s.Passed }
        if ($p -lt [int]$MinPerBinary[$bin]) {
            return "binary '$bin' passed=$p < required $($MinPerBinary[$bin]) — a sibling binary cannot substitute it"
        }
    }
    $okAny = @{}
    foreach ($s in $summarized) { foreach ($n in $s.OkNames) { $okAny[$n] = $true } }
    # Exact executed-ok matching, tolerant of the libtest MODULE PREFIX:
    # a required id matches a name only on FULL equality or equality of the
    # final `::` segment ("tests::ac_case_tests::x" satisfies "x"; it does
    # NOT satisfy "x_extra" or prose text).
    foreach ($id in $RequiredIds) {
        $idLast = ($id -split '::')[-1]
        if ($RequiredInBinary.ContainsKey($id)) {
            $bin = $RequiredInBinary[$id]
            $seg = @($summarized | Where-Object { $_.Binary -like "*$bin*" })
            $hit = $false
            foreach ($s in $seg) {
                foreach ($n in $s.OkNames) {
                    if ($n -eq $id -or (($n -split '::')[-1]) -eq $idLast) { $hit = $true }
                }
            }
            if (-not $hit) { return "required id missing from binary '$bin' executed-ok set: $id" }
        } else {
            $hit = $false
            foreach ($k in $okAny.Keys) {
                if ($k -eq $id -or (($k -split '::')[-1]) -eq $idLast) { $hit = $true; break }
            }
            if (-not $hit) { return "required id never executed-ok: $id" }
        }
    }
    return $null
}

# ---- nextest parsing (closeout: root cargo t/tv/tt) -------------------------
# nextest prints "PASS [ 0.012s] binary::test_name" lines plus a final
# "Summary [ 12.3s] N tests run: M passed, K failed, J skipped."
function Test-NextestFamily {
    param([string]$Text, [int]$ExitCode, [string[]]$RequiredIds = @())
    if ($ExitCode -ne 0) { return "exit=$ExitCode" }
    $summaries = [regex]::Matches($Text, '(\d+) tests run: (\d+) passed, (\d+) failed, (\d+) skipped')
    if ($summaries.Count -eq 0) { return "no nextest summaries parsed" }
    foreach ($s in $summaries) {
        if ([int]$s.Groups[3].Value -ne 0) { return "failed=$($s.Groups[3].Value) in a nextest summary" }
        if ([int]$s.Groups[4].Value -ne 0) { return "skipped=$($s.Groups[4].Value) in a nextest summary (no skips are allowlisted)" }
    }
    $passedNames = @{}
    foreach ($m in [regex]::Matches($Text, '(?:PASS|DONE)\s+\[\s*[\d.]+s\]\s+(\S+)')) {
        $passedNames[$m.Groups[1].Value] = $true
    }
    foreach ($id in $RequiredIds) {
        $short = ($id -split '::')[-1]
        $hit = $false
        foreach ($k in $passedNames.Keys) { if ($k -eq $id -or ($k -split '::')[-1] -eq $short) { $hit = $true } }
        if (-not $hit) { return "required id never reported PASS by nextest: $id" }
    }
    return $null
}

function Run-CargoFamily {
    # Positional call shape (as used below): Name, Argv, RequiredIds,
    # MinPassed, AllowedIgnored — Format/RequiredInBinary/MinPerBinary are
    # named-only.
    param(
        [string]$Name, [string[]]$Argv,
        [string[]]$RequiredIds = @(), [int]$MinPassed = 1,
        [string[]]$AllowedIgnored = @(),
        [hashtable]$RequiredInBinary = @{},
        [hashtable]$MinPerBinary = @{},
        [string]$Format = "libtest"
    )
    Write-Host "== $Name" -ForegroundColor Cyan
    $text = (& cargo @Argv 2>&1 | Out-String)
    $code = $LASTEXITCODE
    $why = if ($Format -eq "nextest") {
        Test-NextestFamily -Text $text -ExitCode $code -RequiredIds $RequiredIds
    } else {
        Test-LibTestFamily -Text $text -ExitCode $code -RequiredIds $RequiredIds `
            -RequiredInBinary $RequiredInBinary -MinPassed $MinPassed -MinPerBinary $MinPerBinary `
            -AllowedIgnored $AllowedIgnored
    }
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
        # legal controls
        @{ name = "fully-legal"; text = "running 1 test`ntest a ... ok`ntest result: ok. 5 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @("a"); min = 1; want = $null },
        # R3 F06 negatives (permanent): prose text and name PREFIXES can no
        # longer satisfy a required ID.
        @{ name = "id-only-in-prose"; text = "note: needed something`ntest unrelated ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @("needed"); min = 1; want = "required id never executed-ok: needed" },
        @{ name = "id-prefix-substitution"; text = "running 1 test`ntest needed_extra ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @("needed"); min = 1; want = "required id never executed-ok: needed" },
        # binary binding: the required id exists but in the WRONG binary
        @{ name = "wrong-binary"; text = "     Running tests\alpha.rs (a.exe)`ntest result: ok. 1 passed; 0 failed; 0 ignored`n     Running tests\beta.rs (b.exe)`ntest needed ... ok`ntest result: ok. 9 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @("needed"); bind = @{ "needed" = "alpha" }; min = 1; want = "required id missing from binary 'alpha' executed-ok set: needed" },
        # the lowercase "running N tests" libtest banner must NOT be taken
        # for cargo's "Running <binary>" header (case-sensitive match) —
        # no binary segment exists, so the bound id cannot be found there
        @{ name = "lowercase-running-not-header"; text = "running 7 tests`ntest needed ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @("needed"); bind = @{ "needed" = "alpha" }; min = 1; want = "required id missing from binary 'alpha' executed-ok set: needed" },
        # module prefixes satisfy a bare required id (exact last segment)
        @{ name = "module-prefix-legal"; text = "running 1 test`ntest tests::ac_case_tests::needed ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @("needed"); min = 1; want = $null },
        # per-binary minimum: a fat sibling cannot substitute a missing family
        @{ name = "sibling-substitution"; text = "     Running tests\alpha.rs (a.exe)`ntest result: ok. 0 passed; 0 failed; 0 ignored`n     Running tests\beta.rs (b.exe)`ntest result: ok. 9 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @(); min = 1; perbin = @{ "alpha" = 3 }; want = "binary 'alpha' passed=0 < required 3 — a sibling binary cannot substitute it" },
        # legacy shapes still refused
        @{ name = "zero-selected"; text = "running 0 tests`ntest result: ok. 0 passed; 0 failed; 0 ignored"; exit = 0; ids = @(); min = 1; want = "totalPassed=0 < required 1" },
        @{ name = "partially-skipped"; text = "test result: ok. 4 passed; 0 failed; 999 ignored"; exit = 0; ids = @(); min = 1; want = "summary ignored=999 does not match named ignored tests (0) (<implicit>)" },
        @{ name = "unnamed-skip"; text = "test mystery_tool ... ignored`ntest result: ok. 4 passed; 0 failed; 1 ignored"; exit = 0; ids = @(); min = 1; want = "ignored test not allowlisted: mystery_tool" },
        @{ name = "allowlisted-skip"; text = "test pilot_oracle_bootstrap ... ignored`ntest result: ok. 4 passed; 0 failed; 1 ignored"; exit = 0; ids = @(); min = 1; want = $null },
        @{ name = "nonzero-exit"; text = "test result: FAILED. 1 passed; 1 failed; 0 ignored"; exit = 101; ids = @(); min = 1; want = "exit=101" },
        @{ name = "failed-in-summary"; text = "test x ... ok`ntest result: FAILED. 1 passed; 1 failed; 0 ignored"; exit = 0; ids = @(); min = 1; want = "failed=1 in a summary (<implicit>)" },
        # nextest parser
        @{ name = "nextest-ok"; fmt = "nextest"; text = "PASS [ 0.01s] alpha::needed`nSummary [ 1.0s] 5 tests run: 5 passed, 0 failed, 0 skipped"; exit = 0; ids = @("alpha::needed"); want = $null },
        @{ name = "nextest-skipped"; fmt = "nextest"; text = "Summary [ 1.0s] 5 tests run: 4 passed, 0 failed, 1 skipped"; exit = 0; ids = @(); want = "skipped=1 in a nextest summary (no skips are allowlisted)" },
        @{ name = "nextest-missing-id"; fmt = "nextest"; text = "PASS [ 0.01s] alpha::other`nSummary [ 1.0s] 1 tests run: 1 passed, 0 failed, 0 skipped"; exit = 0; ids = @("alpha::needed"); want = "required id never reported PASS by nextest: alpha::needed" }
    )
    foreach ($c in $cases) {
        $allow = if ($c.name -eq "allowlisted-skip") { @("pilot_oracle_bootstrap") } else { @() }
        $bind = if ($c.bind) { $c.bind } else { @{} }
        $perbin = if ($c.perbin) { $c.perbin } else { @{} }
        $min = if ($c.min) { [int]$c.min } else { 1 }
        $fmt = if ($c.fmt) { $c.fmt } else { "libtest" }
        if ($fmt -eq "nextest") {
            $why = Test-NextestFamily -Text $c.text -ExitCode $c.exit -RequiredIds $c.ids
        } else {
            $why = Test-LibTestFamily -Text $c.text -ExitCode $c.exit -RequiredIds $c.ids `
                -RequiredInBinary $bind -MinPassed $min -MinPerBinary $perbin -AllowedIgnored $allow
        }
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
    # V-02/04/07/08/09/10 worker faces — every required ID bound to ITS
    # binary, with per-binary minimums (R3 F06).
    $workerBind = @{
        "ac_stage_parse_boundary_attributed" = "protocol_t03_"
        "native_cache_fresh_then_object_hit" = "cache_t05_"
        "native_invalidation_body_edit_full_miss" = "cache_t05_"
        "r1_budget_one_ms_total_is_enforced_not_ignored" = "review_r1_"
        "r1_product_run_invalid_exe_is_infra_error" = "review_r1_"
        "r1_backend_missing_entry_typed_through_store_path" = "review_r1_"
        "syntax_case_edit_equivalence_and_counters" = "syntax_t06_"
        "revision_trace_sequential_edits_advance" = "syntax_t06_"
    }
    $workerPerBin = @{ "feasibility_t01_" = 3; "protocol_t03_" = 10; "cache_t05_" = 7; "review_r1_" = 8; "syntax_t06_" = 7 }
    Run-CargoFamily "worker protocol+cache+r1+syntax" @("test", "--manifest-path", "experimental/ac-case-worker/Cargo.toml", "--test", "feasibility_t01_", "--test", "protocol_t03_", "--test", "cache_t05_", "--test", "review_r1_", "--test", "syntax_t06_") `
        @("ac_stage_parse_boundary_attributed", "native_cache_fresh_then_object_hit", "native_invalidation_body_edit_full_miss", "r1_budget_one_ms_total_is_enforced_not_ignored", "r1_product_run_invalid_exe_is_infra_error", "r1_backend_missing_entry_typed_through_store_path", "syntax_case_edit_equivalence_and_counters", "revision_trace_sequential_edits_advance") 35 `
        -RequiredInBinary $workerBind -MinPerBinary $workerPerBin
    # V-08/09/10 store faces
    Run-CargoFamily "probe plan788_r1_ + 783 regressions" @("test", "--manifest-path", "experimental/ac-artifact-probe/Cargo.toml", "--release", "--test", "plan788_r1_", "--test", "native_resume_", "--test", "invalidation_", "--test", "identity_", "--test", "concurrency_", "--test", "store_boundary_") `
        @("r1_compiler_recipe_change_invalidates_hir_and_object", "r1_observed_spans_carry_real_frontend_boundaries", "native_resume_four_fixtures_fresh_restart_hirhit_and_source_absent") 20
    # V-03/04/11/13/14 runner integration
    Run-CargoFamily "auto-lang ac_case_tests (V-03/04/11/14)" @("test", "-p", "auto-lang", "--features", "test-cases", "--lib", "ac_case_tests") `
        @("ac_native_run_exit_oracle_and_red_sibling", "ac_native_typed_negative_stage_attribution", "ac_product_handoff_run_consumes_real_build", "r1_consumer_invalid_exe_is_error_not_pass", "r1_response_admission_table", "syntax_parse_and_edit_dispatch_with_goldens", "ac_native_suite_full_run_and_ledger_reconcile") 13
    # pilot_oracle_bootstrap is a PRE-EXISTING deliberate #[ignore] manual
    # regeneration tool from Plan 768 (documented in the suite), not a
    # coverage skip — allowlisted by name, nothing else.
    Run-CargoFamily "auto-lang v1 regression" @("test", "-p", "auto-lang", "--features", "test-cases", "--lib", "test_cases_tests") `
        @("pilot_suite_full_run", "stale_check_detects_oracle_drift") 26 @("tests::test_cases_tests::pilot_oracle_bootstrap")
    # V-05/06 + F01/F04/F05 R3 closures (single serial process: the perf
    # scenarios and subprocess stale paths must not fight each other for
    # the machine; no --nocapture — libtest splits "test NAME ... ok" when
    # output interleaves, and the receipts live in the raw JSONL files).
    $r3Bind = @{
        "r3_parent_io_lifecycle_overcap_and_deadline" = "plan788_r3_"
        "r3_hir_backend_negative_keeps_legal_partial_hir" = "plan788_r3_"
        "r3_stale_production_paths_per_adapter_action" = "plan788_r3_"
        "r3_native_bytes_real_worker_path_producers" = "plan788_r3_"
        "turnaround_native_six_scenarios_p95_gates" = "plan788_r3_"
        "turnaround_syntax_gate_and_revision_stream" = "plan788_r3_"
    }
    Run-CargoFamily "auto-lang plan788_r3_ R3 closures + turnaround (V-12/13 gates)" @("test", "-p", "auto-lang", "--features", "test-cases", "--test", "plan788_r3_", "--", "--test-threads=1") `
        @("r3_parent_io_lifecycle_overcap_and_deadline", "r3_hir_backend_negative_keeps_legal_partial_hir", "r3_stale_production_paths_per_adapter_action", "r3_native_bytes_real_worker_path_producers", "turnaround_native_six_scenarios_p95_gates", "turnaround_syntax_gate_and_revision_stream") 6 `
        -RequiredInBinary $r3Bind

    if ($fails -gt 0) { throw "acceptance: $fails family/ies failed or refused" }
    Write-Host "PLAN-788 acceptance: ALL PASS" -ForegroundColor Green
    exit 0
}

# closeout (merge-owned): frozen-commit full surfaces. The root aliases
# (t/tv/tt) run under NEXTEST — verified per its real output format; the
# remaining families are libtest multi-binary runs.
Run-CargoFamily "auto-lang full (test-cases)" @("test", "-p", "auto-lang", "--features", "test-cases") @() 1
Run-CargoFamily "root cargo t (nextest)" @("t") @() 1 -Format "nextest"
Run-CargoFamily "root cargo tv (nextest)" @("tv") @() 1 -Format "nextest"
Run-CargoFamily "root cargo tt (nextest)" @("tt") @() 1 -Format "nextest"
Run-CargoFamily "worker full (release)" @("test", "--manifest-path", "experimental/ac-case-worker/Cargo.toml", "--release") @() 1
Run-CargoFamily "probe full (release)" @("test", "--manifest-path", "experimental/ac-artifact-probe/Cargo.toml", "--release") @() 1
Run-CargoFamily "ac-syntax runtime (release)" @("test", "--manifest-path", "experimental/ac-syntax/runtime/Cargo.toml", "--release") @() 1

if ($fails -gt 0) { throw "closeout: $fails surface(s) failed" }
Write-Host "PLAN-788 closeout: ALL PASS" -ForegroundColor Green
