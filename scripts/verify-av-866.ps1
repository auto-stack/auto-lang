# verify-av-866.ps1 — PLAN-866 AV resident online hot reload (RF-H) verifier.
#
# Scopes (each fails loudly; a scope that is not wired exits 3, never green):
#   targeted   the phase-1 private controls: the resident run driver unit
#              suite (slice invariance, restart-variant red, cancel/stop,
#              zero-fuel park, candidate-queue discipline) plus the frozen
#              reload contract document's presence.
#   resident   the ONLINE machinery: safety-point parking, swap-resolution
#              resume, effective-iteration oracle (V-02), in-flight old
#              frames (V-03), stale/foreign refusals (V-04), bounded
#              recycle churn + zero-rework + P90 visibility (V-05/V-06).
#   compat     the affected-profile compatibility pass (V-07): the legacy
#              segment/resume/contract families + core, then the ace-m1
#              baseline denominator beside the new H rows.
#   acceptance the H-corpus file-backed first case (H01-H03 through a real
#              worker process, parent-side judgement) plus the wrong-oracle
#              discrimination control. Merge owns the frozen-commit rerun.
#
# Exit codes: 0 = scope satisfied; 1 = verification failure; 2 = usage /
# contract error; 3 = scope not wired yet.

param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('targeted', 'resident', 'compat', 'acceptance')]
    [string]$Scope,

    # Root of the auto-lang checkout (defaults to the script's parent).
    [string]$RepoRoot
)

$ErrorActionPreference = 'Stop'
if (-not $RepoRoot) { $RepoRoot = (Split-Path -Parent $PSScriptRoot) }

function Fail([string]$message) {
    Write-Host "verify-av-866 [$Scope]: FAIL - $message" -ForegroundColor Red
    exit 1
}

function NotWired([string]$what) {
    Write-Host "verify-av-866 [$Scope]: NOT WIRED - $what" -ForegroundColor Yellow
    exit 3
}

$probeDir = Join-Path $RepoRoot 'experimental/ac-av-probe'

switch ($Scope) {
    'targeted' {
        if (-not (Test-Path (Join-Path $probeDir 'probe/src/resident.rs'))) {
            Fail("resident.rs (resident run driver) is missing")
        }
        $contract = Join-Path $RepoRoot 'docs/plans/attachments/866/work/resident-reload-contract.md'
        if (-not (Test-Path $contract)) {
            Fail("frozen reload contract is missing: $contract")
        }
        Push-Location $probeDir
        try {
            cargo test -p auto-av-probe --test av_resident_866
            if ($LASTEXITCODE -ne 0) {
                Fail("av_resident_866 suite exited $LASTEXITCODE")
            }
        }
        finally { Pop-Location }
        Write-Host "verify-av-866 [targeted]: OK (phase-1 resident driver controls green)" -ForegroundColor Green
        exit 0
    }
    'resident' {
        if (-not (Test-Path (Join-Path $probeDir 'probe/src/resident.rs'))) {
            Fail("resident.rs (resident run driver) is missing")
        }
        Push-Location $probeDir
        try {
            cargo test -p auto-av-probe --test av_resident_866
            if ($LASTEXITCODE -ne 0) {
                Fail("av_resident_866 suite (online + phase-1 controls) exited $LASTEXITCODE")
            }
            cargo test -p auto-av-core
            if ($LASTEXITCODE -ne 0) {
                Fail("core safety-point surface regressed: auto-av-core exited $LASTEXITCODE")
            }
        }
        finally { Pop-Location }
        Write-Host "verify-av-866 [resident]: OK (online safety-point machinery green)" -ForegroundColor Green
        exit 0
    }
    'compat' {
        # V-07 compatibility pass on the post-864 merged base: the legacy
        # segment/resume/contract families + core, then the ace-m1 baseline
        # (the existing denominator must stay green beside the new H rows).
        Push-Location $probeDir
        try {
            cargo test -p auto-av-probe --test av_closures_823_incremental --test function_incremental_828_b2 --test function_incremental_828_contract --test av_entry_862
            if ($LASTEXITCODE -ne 0) { Fail("legacy segment/resume/contract families exited $LASTEXITCODE") }
            cargo test -p auto-av-core
            if ($LASTEXITCODE -ne 0) { Fail("core exited $LASTEXITCODE") }
        }
        finally { Pop-Location }
        Push-Location (Join-Path $RepoRoot 'crates/auto-lang/..')
        try {
            cargo test -p auto-lang --features test-cases --lib ace_m1 -- --test-threads=1
            if ($LASTEXITCODE -ne 0) { Fail("ace-m1 baseline exited $LASTEXITCODE") }
        }
        finally { Pop-Location }
        Write-Host "verify-av-866 [compat]: OK (legacy families + core + ace-m1 denominator green)" -ForegroundColor Green
        exit 0
    }
    'acceptance' {
        # The H-corpus file-backed first case (real worker process,
        # parent-side judgement) + the discrimination control.
        Push-Location (Join-Path $RepoRoot 'crates/auto-lang/..')
        try {
            cargo test -p auto-lang --features test-cases --lib av_resident_866
            if ($LASTEXITCODE -ne 0) { Fail("resident-866 harness exited $LASTEXITCODE") }
        }
        finally { Pop-Location }
        Write-Host "verify-av-866 [acceptance]: OK (H01-H03 file-backed corpus + wrong-oracle control green)" -ForegroundColor Green
        exit 0
    }
}
