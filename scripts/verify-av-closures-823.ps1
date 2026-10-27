# PLAN-823 verification script (T-02: first-case scope live; further
# scopes arrive with T-03..T-06 and fail loudly here until then).
# Usage: pwsh -File scripts/verify-av-closures-823.ps1 -Scope first-case
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('first-case', 'semantics', 'admission', 'resources', 'cache', 'compatibility', 'autodown', 'baseline', 'turnaround', 'acceptance')]
    [string]$Scope
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$probe = Join-Path $root 'experimental/ac-av-probe'

function Invoke-CargoFilter {
    param([string]$TestTarget, [string]$Filter, [string]$Label, [string[]]$Required = @())
    Write-Host "== av.closures/1 $Label ==" -ForegroundColor Cyan
    Push-Location $probe
    try {
        # R1-F02: 0 selected is a FAILURE, never a pass — parse the count.
        $out = cargo test --test $TestTarget $Filter 2>&1 | Tee-Object -Variable lines
        $resultLine = $lines | Where-Object { $_ -match 'test result:' } | Select-Object -First 1
        if ($LASTEXITCODE -ne 0) {
            Write-Host "FAIL: $Label (exit $LASTEXITCODE)" -ForegroundColor Red
            exit $LASTEXITCODE
        }
        if ($resultLine -notmatch '(\d+) passed') {
            Write-Host "FAIL: $Label (no result line)" -ForegroundColor Red
            exit 5
        }
        if ([int]$Matches[1] -eq 0) {
            Write-Host "FAIL: $Label (0 tests selected — never a pass)" -ForegroundColor Red
            exit 4
        }
        if ($resultLine -match ';\s*[1-9]\d* ignored') {
            throw "FAIL: $Label (required tests were ignored)"
        }
        $outputText = $lines | Out-String
        foreach ($name in $Required) {
            $pattern = "(?m)^test $([regex]::Escape($name)) \.\.\. ok\s*$"
            if (([regex]::Matches($outputText, $pattern)).Count -ne 1) {
                throw "FAIL: $Label (missing or duplicated required test: $name)"
            }
        }
    }
    finally { Pop-Location }
}

function Invoke-RootTest {
    param([string]$Name)
    Push-Location $root
    try {
        $output = & cargo test -p auto-lang --lib --features auto-lang/test-cases $Name -- --test-threads=1 2>&1 | Out-String
        Write-Host $output
        if ($LASTEXITCODE -ne 0) { throw "compatibility test $Name failed" }
        if (([regex]::Matches($output, "(?m)^test .*::$([regex]::Escape($Name)) \.\.\. ok\s*$")).Count -ne 1 -or
            $output -notmatch 'test result: ok\.' -or $output -match ';\s*[1-9]\d* ignored') {
            throw "compatibility test missing, duplicated, or ignored: $Name"
        }
    } finally { Pop-Location }
}

function Invoke-ModuleSubset {
    Write-Host "== av.closures/1 module subset (parallel/823) ==" -ForegroundColor Cyan
    $output = & cargo test --manifest-path (Join-Path $probe 'parallel/823/Cargo.toml') av_closures_823 2>&1 | Out-String
    Write-Host $output
    if ($LASTEXITCODE -ne 0) { throw "module subset failed" }
    if ($output -notmatch 'test result: ok\.\s+[1-9]\d* passed' -or $output -match ';\s*[1-9]\d* ignored') {
        throw "module subset missing, empty, or skipped"
    }
    foreach ($name in @('prepare_commit_mixed', 'env_drop_reverse_once', 'faulted_clone_sources_readable',
        'move_source_preserved_on_failure', 'borrow_kinds_rejected', 'commit_drift_rejected', 'budget_boundaries',
        'unknown_profile_rejected', 'module_subset_identity', 'deep_clone_independent_buffers',
        'list_variant_place_kinds', 'borrow_mode_staging')) {
        if (([regex]::Matches($output, "(?m)^test .*::av_closures_823_t10_$name \.\.\. ok\s*$")).Count -ne 1) {
            throw "module subset missing or duplicated required test: $name"
        }
    }
}

switch ($Scope) {
    'first-case' {
        # The §5.2 whole path: creator-returns closure, new-frame call,
        # mutable env field, drop; real gates + code-cache restore +
        # admission negatives (V-01/V-08 positive controls).
        Invoke-CargoFilter 'av_closures_823_first_case' 'av_closures_823' 'first-case'
        # The same-source module subset (V-14 module axes) stays green.
        Invoke-ModuleSubset
    }
    'semantics' {
        # T-03: the full-support-surface fixtures and negatives — borrow
        # captures (V-02), nested envs/frame reuse (V-03), call typing
        # (V-04), trap/cancel release + closure clone independence
        # (V-06/V-11/V-12), over the REAL pipeline.
        Invoke-CargoFilter 'av_closures_823_semantics' 'av_closures_823' 'semantics'
        Invoke-CargoFilter 'av_closures_823_first_case' 'av_closures_823' 'first-case (regression)'
        Invoke-ModuleSubset
    }
    'admission' {
        # T-02/T-03 front-door negatives: untrusted-builder rejections the
        # REAL gate catches before execution (dispatch=0 equivalent).
        Invoke-CargoFilter 'av_closures_823_first_case' 'av_closures_823_first_case_admission_negatives' 'admission negatives'
        Invoke-CargoFilter 'av_closures_823_semantics' 'rejected' 'semantics negatives'
    }
    'resources' {
        # T-04: executor-side resource/fault states and shared-store
        # terminal-metadata cleanup, including its recovery-state control.
        Invoke-CargoFilter 'av_closures_823_resources' 'av_closures_823' 'resources'
        Push-Location $root
        try {
            $output = & cargo test --manifest-path experimental/ac-artifact-probe/Cargo.toml --test plan823_gc_metadata -- --test-threads=1 2>&1 | Out-String
            Write-Host $output
            $required = @('gc_prunes_only_terminal_metadata_and_preserves_user_data')
            if ([Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT) { $required += 'gc_retains_busy_gate_and_retries_after_reader_release' }
            if ($LASTEXITCODE -ne 0 -or $output -notmatch 'test result: ok\.' -or $output -match ';\s*[1-9]\d* ignored') {
                throw 'store GC metadata controls skipped or failed'
            }
            foreach ($name in $required) {
                if (([regex]::Matches($output, "(?m)^test $([regex]::Escape($name)) \.\.\. ok\s*$")).Count -ne 1) {
                    throw "store GC control missing or duplicated: $name"
                }
            }
        } finally { Pop-Location }
    }
    'cache' {
        # T-05: same-store restore identity axes and old-consumer
        # isolation (V-08/V-09).
        Invoke-CargoFilter 'av_closures_823_cache_compat' 'av_closures_823' 'cache compatibility'
        Invoke-CargoFilter 'av_closures_823_first_case' 'restore' 'first-case restore (regression)'
    }
    'compatibility' {
        # T-05/T-06: the delivered-consumer replay — the earlier profiles'
        # families must stay green with the closures surface present.
        Invoke-CargoFilter 'av_data_820_semantics' 'av_data' '820 replay'
        Invoke-CargoFilter 'av_owned_821_first_case' 'av_owned_821' '821 replay (first)'
        Invoke-CargoFilter 'av_owned_821_semantics' 'av_owned_821' '821 replay (semantics)'
        Invoke-CargoFilter 'av_collections_822_first_case' 'av_collections_822' '822 replay (first)'
        Invoke-CargoFilter 'av_collections_822_semantics' 'av_collections_822' '822 replay (semantics)'
        Invoke-CargoFilter 'av_closures_823_cache_compat' 'closures_wire_isolated' 'V-09 isolation'
        foreach ($name in @('ac_av_probe_corpus_executes_green', 'cte_probe_corpus_executes_green', 'ac_mn_probe_corpus_executes_green')) {
            Invoke-RootTest $name
        }
        Push-Location $root
        try {
            $output = & cargo test --manifest-path experimental/ac-mir-native/Cargo.toml --test closures_823_compat closures_units_are_rejected_before_native_codegen 2>&1 | Out-String
            Write-Host $output
            if ($LASTEXITCODE -ne 0 -or $output -notmatch '(?m)^test closures_units_are_rejected_before_native_codegen \.\.\. ok\s*$' -or
                $output -notmatch 'test result: ok\.' -or $output -match ';\s*[1-9]\d* ignored') {
                throw 'native closures rejection missing, skipped, or failed'
            }
        } finally { Pop-Location }
    }
    'autodown' {
        # The formal suite must exercise the actual parent and worker.
        Push-Location $root
        try {
            $output = & cargo test -p auto-lang --lib --features auto-lang/test-cases av_closures_823 -- --test-threads=1 2>&1 | Out-String
            Write-Host $output
            if ($LASTEXITCODE -ne 0) { throw "formal closures tests failed" }
            foreach ($required in @('av_closures_823_corpus_green', 'av_closures_823_negative_oracle_fails', 'av_closures_823_worker_faults_are_infra', 'av_closures_823_missing_case_is_not_green', 'av_closures_823_missing_worker_is_not_green')) {
                if (([regex]::Matches($output, "(?m)^test .*::$required \.\.\. ok\s*$")).Count -ne 1) { throw "missing or duplicated formal test: $required" }
            }
            if ($output -notmatch 'test result: ok\.' -or $output -match ';\s*[1-9]\d* ignored') { throw "formal result absent or skipped" }
        } finally { Pop-Location }
    }
    'baseline' {
        # Validate typed-start labels, required binding rows and the recorded
        # input identity before running the actual tests.
        & python (Join-Path $PSScriptRoot 'verify-av-closures-823-bindings.py') --root $root --self-test
        if ($LASTEXITCODE -ne 0) { throw '823 binding or stale-evidence controls failed' }
        # Explicit required cases make missing/ignored matrix entries fail.
        Invoke-CargoFilter 'av_closures_823_first_case' 'av_closures_823' 'K01/K02/K06/R05' @(
            'av_closures_823_first_case_executes_success', 'av_closures_823_first_case_restore_reverifies')
        Invoke-CargoFilter 'av_closures_823_semantics' 'borrow' 'K03/K04' @(
            'av_closures_823_sem_borrow_legal_sync_call', 'av_closures_823_sem_borrow_escape_rejected',
            'av_closures_823_sem_borrow_source_pinned', 'av_closures_823_sem_borrow_domain_rejects_non_str')
        Invoke-CargoFilter 'av_closures_823_semantics' 'clone' 'K05' @('av_closures_823_sem_closure_clone_independent_env')
        Invoke-CargoFilter 'av_closures_823_cache_compat' 'restore' 'I05/I06 restore axes' @('av_closures_823_cache_restore_reproduces_and_reverifies')
        Invoke-CargoFilter 'av_closures_823_cache_compat' 'cross_project' 'I06 cross-project identity' @('av_closures_823_cache_cross_project_content_identity')
        Invoke-CargoFilter 'av_closures_823_incremental' 'av_closures_823' 'I01/I02/I04/I05/I10/I11/I12' @(
            'av_closures_823_incremental_body_edit_identity', 'av_closures_823_incremental_env_contract_drift_refused',
            'av_closures_823_incremental_forced_degradation_gate_nonzero', 'av_closures_823_incremental_churn_512', 'av_closures_823_incremental_warm_edit_p90', 'av_closures_823_incremental_p90_threshold_controls',
            'av_closures_823_incremental_baseline_campaign_replay', 'av_closures_823_recipe_new_process_worker_restore',
            'av_closures_823_worker_memory_hard_gate_parent_sampled', 'av_closures_823_incremental_paused_call_pins_old_body', 'av_closures_823_incremental_history_revisit_hits', 'av_closures_823_incremental_history_bound_and_failed_publish', 'av_closures_823_incremental_store_history_revisit', 'av_closures_823_incremental_history_hot_budget_evicts', 'av_closures_823_incremental_store_failed_publish_does_not_root_candidate', 'av_closures_823_incremental_store_512_revisions_gc_bounded', 'av_closures_823_incremental_shared_store_session_roots_are_independent', 'av_closures_823_incremental_dropped_session_releases_roots')
        Invoke-CargoFilter 'av_closures_823_recipe' 'judge_detects_lies' 'I12 oracle sensitivity' @('av_closures_823_recipe_judge_detects_lies')
    }
    'turnaround' {
        # Plan 823 r9: retain the exact 798 parent-side window while using
        # 10 samples and the explicitly selected P90 rule. Restore caller env.
        Invoke-RootTest 'plan823_798_p90_controls'
        $keys = @('AV798_FORMAL_SAMPLES', 'AV798_FORMAL_GATE_MS', 'AV798_FORMAL_PLAN823_P90', 'AV798_FAULT')
        $previous = @{}
        foreach ($key in $keys) { $previous[$key] = [Environment]::GetEnvironmentVariable($key, 'Process') }
        try {
            $env:AV798_FORMAL_SAMPLES = '10'
            $env:AV798_FORMAL_GATE_MS = '2000'
            $env:AV798_FORMAL_PLAN823_P90 = '1'
            Remove-Item Env:AV798_FAULT -ErrorAction SilentlyContinue
            Push-Location $root
            try {
                $output = & cargo test -p auto-lang --lib --features auto-lang/test-cases ac_av_probe_formal_turnaround_gates -- --test-threads=1 --nocapture 2>&1 | Out-String
                Write-Host $output
                if ($LASTEXITCODE -ne 0 -or $output -notmatch '(?m)^test result: ok\. 1 passed; 0 failed; 0 ignored' -or
                    $output -notmatch 'plan823 798 P90:.*limit=2000ms n=10' -or $output -match ';\s*[1-9]\d* ignored') {
                    throw 'plan 823 798 P90 gate missing, skipped, or failed'
                }
            } finally { Pop-Location }
        } finally {
            foreach ($key in $keys) { [Environment]::SetEnvironmentVariable($key, $previous[$key], 'Process') }
        }
    }
    'acceptance' {
        # Each published scope is mandatory, including formal parent-side
        # adjudication and the 852 baseline. A failing child fails the gate.
        foreach ($child in @('first-case', 'semantics', 'admission', 'resources', 'cache', 'compatibility', 'autodown', 'baseline', 'turnaround')) {
            & (Get-Process -Id $PID).Path -NoProfile -File $PSCommandPath -Scope $child
            if ($LASTEXITCODE -ne 0) { throw "acceptance scope '$child' failed (exit $LASTEXITCODE)" }
        }
        Write-Host "acceptance: all closures scopes green" -ForegroundColor Green
    }
    default {
        Write-Host "Scope '$Scope' is not implemented yet. Refusing to pass." -ForegroundColor Red
        exit 2
    }
}
Write-Host "PASS: $Scope" -ForegroundColor Green
