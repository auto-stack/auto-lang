# PLAN-824 verify script: the av.dynamic/1 acceptance driver.
# Template: verify-av-collections-822.ps1's guard ideas. T-02 scope:
# first-case + autodown (+ acceptance = both); the remaining scopes
# (semantics/admission/resources/cache/compatibility) land with T-03..T-06.
# The output completeness guard REJECTS ignored/skipped cases and verifies
# every REQUIRED case id appears EXACTLY ONCE in the run output.
[CmdletBinding()]
param(
    [Parameter(ParameterSetName = 'Scope', Mandatory = $true)]
    [ValidateSet("first-case", "autodown", "acceptance")]
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
    'autodown' {
        Run-Lang-Tests -Filter 'av_dynamic_824' -Label 'autodown' -RequiredIds $REQ['autodown']
    }
    'acceptance' {
        # Replay every implemented scope in child processes (isolated
        # working directories, same shapes the docs promise).
        foreach ($s in @('first-case', 'autodown')) {
            & pwsh -File $PSCommandPath -Scope $s
            if ($LASTEXITCODE -ne 0) {
                throw "acceptance: scope $s failed"
            }
        }
    }
}
Write-Host "PLAN-824 verify [$Scope]: PASS"
