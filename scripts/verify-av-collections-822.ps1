# PLAN-822 verify script: the av.collections/1 acceptance driver.
# Template: verify-av-owned-821.ps1 (single mandatory Scope; selected>0
# guard; required-id coverage; acceptance replays every scope in child
# processes so an env-seam cannot leak between scopes).
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("first-case", "semantics", "admission", "resources", "cache", "compatibility", "autodown", "incremental", "acceptance")]
    [string]$Scope
)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot | Split-Path -Parent
$probe = Join-Path $root 'experimental/ac-av-probe'

# selected>0 guard: a zero-match green is a FAIL, not a pass.
function Assert-Selected {
    param([string]$Output, [string]$Bin, [string]$Filter, [string]$Label)
    if ($Output -notmatch 'test result:') {
        throw "$Label : no test-result line captured"
    }
    if ($Output -match '0 passed; 0 failed') {
        if ($Output -match '0 measured|filtered out') {
            $ran = ($Output | Select-String -Pattern 'test result: (\d+) passed').Matches
        }
        throw "$Label : zero tests selected ($Bin $Filter) — a green banner with no cases is a FAIL"
    }
}

function Run-Probe-Tests {
    param([string]$Bin, [string]$Filter = '', [string]$Label)
    Push-Location $probe
    try {
        $args = @('test', '-p', 'auto-av-probe', '--test', $Bin)
        if ($Filter) { $args += $Filter }
        $out = & cargo @args 2>&1 | Out-String
        Write-Host $out
        Assert-Selected -Output $out -Bin $Bin -Filter $Filter -Label $Label
        if ($LASTEXITCODE -ne 0) { throw "$Label : cargo test failed ($Bin $Filter)" }
    } finally { Pop-Location }
}

function Run-Runner-Tests {
    param([string]$Filter, [string]$Label)
    Push-Location $root
    try {
        # The daily-tier alias semantics for a scoped runner filter.
        $out = & cargo test -p auto-lang --lib --features auto-lang/test-cases $Filter '--' '--test-threads=1' 2>&1 | Out-String
        Write-Host $out
        Assert-Selected -Output $out -Bin 'auto-lang' -Filter $Filter -Label $Label
        if ($LASTEXITCODE -ne 0) { throw "$Label : runner tests failed ($Filter)" }
    } finally { Pop-Location }
}

switch ($Scope) {
    'first-case' {
        Run-Probe-Tests -Bin 'av_collections_822_first_case' -Label 'first-case'
    }
    'semantics' {
        Run-Probe-Tests -Bin 'av_collections_822_semantics' -Label 'semantics'
    }
    'admission' {
        Run-Probe-Tests -Bin 'av_collections_822_first_case' -Filter 'av_collections_822_first_case_admission_negatives' -Label 'admission'
    }
    'resources' {
        Run-Probe-Tests -Bin 'av_collections_822_resources' -Label 'resources'
    }
    'cache' {
        Run-Probe-Tests -Bin 'av_collections_822_cache_compat' -Label 'cache'
    }
    'compatibility' {
        # V-09: the PRIOR profiles replay unchanged through their own
        # frozen corpora (798/817/818/820/821); the collections wire is a
        # new AVAD generation the old consumers reject by design.
        Run-Probe-Tests -Bin 'av_owned_821_first_case' -Label 'compatibility: 821 first-case'
        Run-Probe-Tests -Bin 'av_owned_821_cache_compat' -Label 'compatibility: 821 cache'
        Run-Runner-Tests -Filter 'av_owned_821' -Label 'compatibility: 821 corpus'
        Run-Runner-Tests -Filter 'ac_av_probe' -Label 'compatibility: 798/820 corpus'
        Run-Runner-Tests -Filter 'cte_probe' -Label 'compatibility: 817 corpus'
        Run-Runner-Tests -Filter 'ac_mn_probe' -Label 'compatibility: 818 corpus'
    }
    'autodown' {
        # V-10: the formal recipe corpus + the sabotage/fault counter-
        # controls through the unified runner.
        Run-Runner-Tests -Filter 'av_collections_822' -Label 'autodown: recipe corpus + counter-controls'
    }
    'incremental' {
        # V-13 (T-09): the D828 incremental seam over the collections
        # capability — the frozen 30-revision baseline campaign replay
        # (dual gate), the identity axes, the precise-invalidation and
        # forced-degradation counter-controls, and the parent-sampled
        # worker B-line memory gate.
        Run-Probe-Tests -Bin 'av_collections_822_incremental' -Label 'incremental: V-13 identity/baseline/gate'
        Run-Probe-Tests -Bin 'av_collections_822_resources' -Filter 'av_collections_822_worker_memory_hard_gate_parent_sampled' -Label 'incremental: worker B-line memory gate'
    }
    'acceptance' {
        # Replay EVERY scope in a CHILD process (env seams cannot leak).
        foreach ($s in @('first-case', 'semantics', 'admission', 'resources', 'cache', 'compatibility', 'autodown', 'incremental')) {
            Write-Host "=== acceptance: scope $s ===" -ForegroundColor Cyan
            & pwsh -NoProfile -File $PSCommandPath -Scope $s
            if ($LASTEXITCODE -ne 0) { throw "acceptance: scope $s failed" }
        }
        Write-Host 'PLAN-822 acceptance: all scopes green' -ForegroundColor Green
    }
}
