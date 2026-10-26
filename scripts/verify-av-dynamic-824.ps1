# PLAN-824 verify script: the av.dynamic/1 acceptance driver.
# Template: verify-av-collections-822.ps1's guard ideas. T-02 scope:
# first-case + autodown (+ acceptance = both); the remaining scopes
# (semantics/admission/resources/cache/compatibility) land with T-03..T-06.
# The output completeness guard REJECTS ignored/skipped cases and verifies
# every REQUIRED case id appears EXACTLY ONCE in the run output.
[CmdletBinding()]
param(
    [Parameter(ParameterSetName = 'Scope', Mandatory = $true)]
    [ValidateSet("first-case", "semantics", "autodown", "acceptance")]
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
    'autodown' {
        Run-Lang-Tests -Filter 'av_dynamic_824' -Label 'autodown' -RequiredIds $REQ['autodown']
    }
    'acceptance' {
        # Replay every implemented scope in child processes (isolated
        # working directories, same shapes the docs promise).
        foreach ($s in @('first-case', 'semantics', 'autodown')) {
            & pwsh -File $PSCommandPath -Scope $s
            if ($LASTEXITCODE -ne 0) {
                throw "acceptance: scope $s failed"
            }
        }
    }
}
Write-Host "PLAN-824 verify [$Scope]: PASS"
