# PLAN-825 LoadedProgram scoped acceptance driver. Unimplemented scopes fail
# explicitly; a zero-selected, ignored, or missing mandatory case is never PASS.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("first", "targeted", "compat", "bench", "baseline", "acceptance")]
    [string]$Scope
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
        $count = ([regex]::Matches($Output, "(?m)^test $([regex]::Escape($id)) .*$")).Count
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

$FIRST_PROBE = @(
    'av_runtime_825_first_case_direct_loaded_and_reference_match',
    'av_runtime_825_first_case_snapshot_and_resume_use_same_loaded_program',
    'av_runtime_825_first_case_cache_hit_and_new_worker_restore',
    'av_runtime_825_first_case_unit_never_dispatches'
)
$FIRST_RUNNER = @('tests::av_runtime_825_tests::av_runtime_825_first_case_corpus_green')

switch ($Scope) {
    'first' {
        Run-Probe -Bin 'av_runtime_825_first_case' -RequiredIds $FIRST_PROBE
        Run-Runner -Filter 'av_runtime_825_first_case_corpus_green' -RequiredIds $FIRST_RUNNER
    }
    default {
        throw "PLAN-825 scope '$Scope' is not implemented yet; this is an explicit acceptance gap, not PASS"
    }
}
