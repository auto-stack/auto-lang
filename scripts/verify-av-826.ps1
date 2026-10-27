# PLAN-826 compact AVAC wire scoped acceptance driver. Unimplemented scopes
# fail explicitly; a zero-selected, ignored, or missing mandatory case is
# never PASS.
[CmdletBinding()]
param(
    [Parameter(ParameterSetName = 'Scope', Mandatory = $true)]
    [ValidateSet("first", "targeted", "compat", "bench", "baseline", "acceptance")]
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
    default {
        throw "PLAN-826 scope '$Scope' is not implemented yet; this is an explicit acceptance gap, not PASS"
    }
}
