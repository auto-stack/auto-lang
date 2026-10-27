# PLAN-826 compact AVAC wire scoped acceptance driver. Unimplemented scopes
# fail explicitly; a zero-selected, ignored, or missing mandatory case is
# never PASS.
[CmdletBinding()]
param(
    [Parameter(ParameterSetName = 'Scope', Mandatory = $true)]
    [ValidateSet("first", "targeted", "compat", "incremental", "bench", "baseline", "acceptance")]
    [string]$Scope,
    [Parameter(ParameterSetName = 'Guard')]
    [switch]$GuardSelfTest
)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot | Split-Path -Parent
$probe = Join-Path $root 'experimental/ac-av-probe'

function Assert-Selected {
    param([string]$Output, [string]$Label, [string[]]$RequiredIds)
    if ($Output -notmatch 'test result:') { throw "$Label : no test-result line" }
    if ($Output -match 'test result: ok\. 0 passed; 0 failed') {
        throw "$Label : zero tests selected"
    }
    if ($Output -match ';\s*(\d+) ignored') {
        if ([int]$Matches[1] -gt 0) { throw "$Label : ignored/skipped mandatory case" }
    }
    foreach ($id in $RequiredIds) {
        $count = ([regex]::Matches($Output, "(?m)^test $([regex]::Escape($id)) .* (?:ok|FAILED|ignored).*$")).Count
        if ($count -ne 1) { throw "$Label : required case '$id' occurred $count times" }
    }
}

function Invoke-Cargo {
    param([string[]]$CargoArgs)
    $previous = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        $script:out = & cargo @CargoArgs 2>&1 | Out-String
        $script:exitCode = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $previous
    }
    Write-Host $script:out
    if ($script:exitCode -ne 0) { throw "cargo $($CargoArgs -join ' ') failed with exit $script:exitCode" }
    return $script:out
}

function Invoke-Native {
    param([string]$Exe, [string[]]$NativeArgs, [string]$Log)
    $previous = $ErrorActionPreference
    try {
        $ErrorActionPreference = 'Continue'
        $out = & $Exe @NativeArgs 2>&1 | Tee-Object -FilePath $Log | Out-String
        $code = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $previous
    }
    Write-Host $out
    if ($code -ne 0) { throw "$Exe failed with exit $code" }
}

function Run-Probe {
    param([string]$Bin, [string]$Filter = '', [string[]]$RequiredIds)
    Push-Location $probe
    try {
        $args = @('test', '-p', 'auto-av-probe', '--test', $Bin)
        if ($Filter) { $args += $Filter }
        $out = Invoke-Cargo -CargoArgs $args
        Assert-Selected -Output $out -Label $Bin -RequiredIds $RequiredIds
    } finally { Pop-Location }
}

function Run-Runner {
    param([string]$Filter, [string[]]$RequiredIds)
    Push-Location $root
    try {
        $out = Invoke-Cargo -CargoArgs @('test', '-p', 'auto-lang', '--lib', '--features', 'auto-lang/test-cases', $Filter, '--', '--test-threads=1')
        Assert-Selected -Output $out -Label $Filter -RequiredIds $RequiredIds
    } finally { Pop-Location }
}

function Run-ParallelHarness {
    # PLAN-826 V-13 module axis: the SAME production source compiled by the
    # independent parallel/826 harness — the formal script aggregates it so
    # a green integration never silently diverges from the frozen module
    # contract (T-10).
    Push-Location $root
    try {
        $out = Invoke-Cargo -CargoArgs @('test', '--manifest-path', 'experimental/ac-av-probe/parallel/826/Cargo.toml', '--lib')
        Assert-Selected -Output $out -Label 'parallel-826' -RequiredIds @()
    } finally { Pop-Location }
}

function Invoke-Benchmark {
    $raw = Join-Path $root 'docs/plans/attachments/826/bench-r1'
    New-Item -ItemType Directory -Force -Path $raw | Out-Null
    foreach ($name in @('av826.jsonl', 'js.jsonl', 'liftoff.jsonl', 'turbofan.jsonl', 'summary.json', 'js-tier.log', 'liftoff-tier.log', 'turbofan-tier.log', 'turnaround.jsonl', 'turnaround-summary.json')) {
        $p = Join-Path $raw $name
        if (Test-Path $p) { Remove-Item -LiteralPath $p -Force }
    }
    Get-ChildItem -LiteralPath $raw -Filter '*.bin' -File | Remove-Item -Force
    Invoke-Cargo -CargoArgs @('build', '--release', '--manifest-path', 'experimental/ac-av-probe/Cargo.toml', '--bin', 'av_wire_826_bench', '--bin', 'av_probe_worker') | Out-Host
    Invoke-Native -Exe (Join-Path $probe 'target/release/av_wire_826_bench.exe') -NativeArgs @('--out', (Join-Path $raw 'av826.jsonl'), '--payload-dir', $raw) -Log (Join-Path $raw 'av-tier.log')
    # V8/Wasm comparator entries over the frozen common load (the PLAN-825
    # bench driver, engine-parameterized; tier flags pin the tiers).
    Invoke-Native -Exe 'node' -NativeArgs @('--jitless', '--print-bytecode', (Join-Path $root 'scripts/av-825-bench.mjs'), '--engine', 'v8-js-ignition', '--out', (Join-Path $raw 'js.jsonl')) -Log (Join-Path $raw 'js-tier.log')
    Invoke-Native -Exe 'node' -NativeArgs @('--liftoff-only', '--no-wasm-tier-up', '--no-wasm-dynamic-tiering', '--trace-wasm-compilation-times', (Join-Path $root 'scripts/av-825-bench.mjs'), '--engine', 'v8-wasm-liftoff', '--out', (Join-Path $raw 'liftoff.jsonl')) -Log (Join-Path $raw 'liftoff-tier.log')
    Invoke-Native -Exe 'node' -NativeArgs @('--no-liftoff', '--trace-wasm-compilation-times', (Join-Path $root 'scripts/av-825-bench.mjs'), '--engine', 'v8-wasm-turbofan', '--out', (Join-Path $raw 'turbofan.jsonl')) -Log (Join-Path $raw 'turbofan-tier.log')
    & powershell -NoProfile -File (Join-Path $root 'scripts/av-826-bench-report.ps1') -RawDir $raw -OutFile (Join-Path $raw 'summary.json')
    if ($LASTEXITCODE -ne 0) { throw 'benchmark report failed' }
    $summary = Get-Content -LiteralPath (Join-Path $raw 'summary.json') -Raw -Encoding UTF8 | ConvertFrom-Json
    if ($summary.total_rows -ne 2176 -or $summary.groups.Count -ne 50 -or $summary.wire_sizes.Count -ne 13) { throw 'benchmark completeness mismatch' }
    # Tier evidence: the flags must have pinned the claimed tiers.
    $js = Get-Content -LiteralPath (Join-Path $raw 'js-tier.log') -Raw -Encoding UTF8
    $liftoff = Get-Content -LiteralPath (Join-Path $raw 'liftoff-tier.log') -Raw -Encoding UTF8
    $turbo = Get-Content -LiteralPath (Join-Path $raw 'turbofan-tier.log') -Raw -Encoding UTF8
    if ($js -notmatch '\[generated bytecode for function:') { throw 'V8 JS Ignition evidence missing' }
    if ([regex]::Matches($liftoff, 'using Liftoff').Count -lt 8) { throw 'Wasm Liftoff evidence missing' }
    if ([regex]::Matches($turbo, 'using TurboFan').Count -lt 8) { throw 'Wasm optimizing-tier evidence missing' }
    # V-09: the original D798 edit load must hold its P95/memory gates on
    # this tree (legacy semantics unchanged behind the compact policy).
    Invoke-Native -Exe (Join-Path $probe 'target/release/av_probe_worker.exe') -NativeArgs @('bench-turnaround', '--samples', '30', '--gate-ms', '1000', '--out', (Join-Path $raw 'turnaround.jsonl')) -Log (Join-Path $raw 'turnaround-summary.json')
    $turnaround = Get-Content -LiteralPath (Join-Path $raw 'turnaround-summary.json') -Raw -Encoding UTF8 | ConvertFrom-Json
    if ($turnaround.hard_gate_failures.Count -ne 0 -or $turnaround.single_body_edit.p95_ms -ge 1000.0) { throw 'original editing P95 gate failed' }
    if ($turnaround.frontend_transient_target_met -ne $true -or $turnaround.worker_private_bytes_target_met -ne $true) { throw 'original editing memory gate failed' }
    # V-10 negative control: fabricated data (a stripped process-peak
    # counter) must redden the analyzer — never a silent PASS.
    $guard = Join-Path $raw 'guard'
    if (Test-Path $guard) { Remove-Item -LiteralPath $guard -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $guard | Out-Null
    Get-ChildItem -LiteralPath $raw -Filter '*.jsonl' -File | ForEach-Object { Copy-Item $_.FullName -Destination $guard }
    Copy-Item (Join-Path $raw '*.bin') -Destination $guard
    $peakFile = Join-Path $guard 'js.jsonl'
    $withoutPeak = @(Get-Content -LiteralPath $peakFile -Encoding UTF8 | Where-Object { $_ -notmatch 'process-peak' })
    Set-Content -LiteralPath $peakFile -Value $withoutPeak -Encoding UTF8
    $failed = $false
    try {
        & powershell -NoProfile -File (Join-Path $root 'scripts/av-826-bench-report.ps1') -RawDir $guard -OutFile (Join-Path $guard 'bad.json') 2>$null
        if ($LASTEXITCODE -ne 0) { $failed = $true }
    } catch { $failed = $true }
    if (-not $failed) { throw 'missing process-peak counter-control was accepted' }
    Remove-Item -LiteralPath $guard -Recurse -Force
    # The negative control's expected non-zero exit must not leak into the
    # caller's $LASTEXITCODE: this scope completed all gates successfully.
    $global:LASTEXITCODE = 0
}

if ($GuardSelfTest) {
    $valid = "running 2 tests`ntest alpha ... ok`ntest beta ... ok`n`ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
    Assert-Selected -Output $valid -Label 'valid' -RequiredIds @('alpha', 'beta')
    $long = "running 1 test`ntest alpha has been running for over 60 seconds`ntest alpha ... ok`n`ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
    Assert-Selected -Output $long -Label 'long-result' -RequiredIds @('alpha')
    foreach ($case in @(
        @{ Name = 'skip'; Out = "running 1 test`ntest alpha ... ignored`n`ntest result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out"; Req = @('alpha') },
        @{ Name = 'missing'; Out = $valid; Req = @('alpha', 'gamma') },
        @{ Name = 'zero'; Out = 'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out'; Req = @() },
        @{ Name = 'duplicate'; Out = "running 3 tests`ntest alpha ... ok`ntest alpha ... ok`ntest beta ... ok`n`ntest result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"; Req = @('alpha', 'beta') }
    )) {
        $failed = $false
        try { Assert-Selected -Output $case.Out -Label $case.Name -RequiredIds $case.Req } catch { $failed = $true }
        if (-not $failed) { throw "guard case '$($case.Name)' must be rejected" }
    }
    Write-Host 'guard self-test ok'
    exit 0
}

$FIRST_PROBE = @(
    'av_wire_826_first_case_compact_roundtrip_admission_and_execution',
    'av_wire_826_first_case_wire_generations_never_cross_admit',
    'av_wire_826_first_case_persist_cache_hit_and_new_worker_restore',
    'av_wire_826_first_case_legacy_driver_and_codec_namespaces',
    'av_wire_826_first_case_unit_never_dispatches'
)
$FIRST_RUNNER = @('tests::av_wire_826_tests::av_wire_826_first_case_corpus_green')
$SENSITIVITY = @('tests::av_wire_826_tests::av_wire_826_parent_sensitivity_controls_non_green')
$TARGETED = @(
    'av_wire_826_targeted_instruction_density_and_uleb_boundaries',
    'av_wire_826_targeted_numeric_domain_roundtrip_and_ref_separation',
    'av_wire_826_targeted_section_directory_and_mutation_rejections'
)
$SECURITY = @(
    'av_wire_826_security_duplicate_missing_and_unknown_required_sections',
    'av_wire_826_security_strip_equivalence_and_inner_invariants',
    'av_wire_826_security_execution_budgets_unchanged'
)
$COMPACT_COMPAT = @(
    'av_wire_826_compat_edit_session_over_compact_cache',
    'av_wire_826_compat_restore_requires_current_authorization',
    'av_wire_826_compat_codec_routing_gate_and_delivered_wire_untouched'
)
$BASELINE = @(
    'av_wire_826_baseline_sources_match_independent_oracles_over_both_wires',
    'av_wire_826_baseline_resume_old_call_after_new_version'
)
$INCREMENTAL = @(
    'av_wire_826_incremental_two_function_edit_identity_and_zero_work',
    'av_wire_826_incremental_content_hit_reuses_every_index',
    'av_wire_826_incremental_signature_reject_then_exact_invalidation',
    'av_wire_826_incremental_forced_copy_cannot_claim_reuse',
    'av_wire_826_incremental_persisted_rows_are_codec_namespaced'
)

switch ($Scope) {
    'first' {
        Run-ParallelHarness
        Run-Probe -Bin 'av_wire_826_first_case' -RequiredIds $FIRST_PROBE
        Run-Runner -Filter 'av_wire_826_first_case_corpus_green' -RequiredIds $FIRST_RUNNER
        Run-Runner -Filter 'av_wire_826_parent_sensitivity_controls_non_green' -RequiredIds $SENSITIVITY
    }
    'targeted' {
        Run-ParallelHarness
        Run-Probe -Bin 'av_wire_826_first_case' -RequiredIds $FIRST_PROBE
        Run-Probe -Bin 'av_wire_826_targeted' -RequiredIds $TARGETED
        Run-Probe -Bin 'av_wire_826_security' -RequiredIds $SECURITY
        Run-Runner -Filter 'av_wire_826_first_case_corpus_green' -RequiredIds $FIRST_RUNNER
    }
    'baseline' {
        Run-ParallelHarness
        Run-Probe -Bin 'av_wire_826_baseline' -RequiredIds $BASELINE
        # I01/I11: the shared D828 30-revision dual-gate campaign.
        Run-Probe -Bin 'function_incremental_828_t08' -Filter 't08_formal_campaign_v08_dual_gate_30_revisions' -RequiredIds @('t08_formal_campaign_v08_dual_gate_30_revisions')
        # I10: the executor-level resource churn tier (wire-independent).
        Run-Probe -Bin 'resources_tier' -Filter 'av_resources_32mib_product_random_read_and_churn' -RequiredIds @('av_resources_32mib_product_random_read_and_churn')
        # Delivered data/owned/collections consumer faces over their own
        # AVAD wires must not regress behind the compact integration.
        Run-Probe -Bin 'av_data_820_cache_compat'
        Run-Probe -Bin 'av_owned_821_cache_compat'
        Run-Probe -Bin 'av_collections_822_cache_compat'
        # I12: parent-side sensitivity controls.
        Run-Runner -Filter 'av_wire_826_parent_sensitivity_controls_non_green' -RequiredIds $SENSITIVITY
        Run-Runner -Filter 'av_wire_826_first_case_corpus_green' -RequiredIds $FIRST_RUNNER
    }
    'incremental' {
        Run-Probe -Bin 'av_wire_826_incremental' -RequiredIds $INCREMENTAL
        Run-Probe -Bin 'av_runtime_825_incremental'
        # V-12 sub-case 1 (shared D828 baseline): the 30-revision dual-gate
        # campaign and the hot-path red control must hold unchanged on
        # this tree.
        Run-Probe -Bin 'function_incremental_828_t08' -Filter 't08_formal_campaign_v08_dual_gate_30_revisions' -RequiredIds @('t08_formal_campaign_v08_dual_gate_30_revisions')
        Run-Probe -Bin 'function_incremental_828_contract' -Filter 'contract_828_hotpath_old_synth_pipeline_same_window_is_red' -RequiredIds @('contract_828_hotpath_old_synth_pipeline_same_window_is_red')
    }
    'compat' {
        Run-ParallelHarness
        Run-Probe -Bin 'av_wire_826_first_case' -RequiredIds $FIRST_PROBE
        Run-Probe -Bin 'av_wire_826_compat' -RequiredIds $COMPACT_COMPAT
        # V-09 compat axis: the delivered consumer families re-run
        # unchanged over their own AVAD wires (no support-face regression
        # from the compact integration).
        Run-Probe -Bin 'av_data_820_cache_compat'
        Run-Probe -Bin 'av_owned_821_cache_compat'
        Run-Probe -Bin 'av_collections_822_cache_compat'
        Run-Runner -Filter 'av_wire_826_first_case_corpus_green' -RequiredIds $FIRST_RUNNER
    }
    'bench' {
        Invoke-Benchmark
    }
    'acceptance' {
        & $PSCommandPath -GuardSelfTest
        if ($LASTEXITCODE -ne 0) { throw 'guard self-test failed' }
        # Sub-scopes run as child processes so $LASTEXITCODE reflects each
        # script's own success (an in-process '&' invocation would surface
        # the sub-scope's last native exit code instead, which the bench
        # negative control deliberately makes non-zero).
        foreach ($required in @('first', 'targeted', 'compat', 'incremental', 'baseline', 'bench')) {
            & powershell -NoProfile -File $PSCommandPath -Scope $required
            if ($LASTEXITCODE -ne 0) { throw "acceptance scope '$required' failed" }
        }
        # SD drafts must exist before review consumes them.
        if (-not (Test-Path (Join-Path $root 'docs/plans/attachments/826/sd-draft.md'))) {
            throw 'SD draft missing'
        }
    }
    default {
        throw "PLAN-826 scope '$Scope' is not implemented yet; this is an explicit acceptance gap, not PASS"
    }
}
