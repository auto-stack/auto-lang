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
#   compat     the affected-profile compatibility pass (V-07) over the
#              post-864 merged base: H corpus rows registered and the
#              legacy segment/resume families re-run. Not wired yet.
#   acceptance the V-07 accounting (first file-backed case, cache
#              fresh/hit/restart/body-edit, turnover/memory budget
#              evidence). Not wired yet; merge-owned.
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
        NotWired("affected-profile compatibility (V-07) runs on the post-864 merged base with the H corpus rows registered")
    }
    'acceptance' {
        NotWired("V-05/V-06 accounting (Weak release, churn, old-gen cap, P90 reload visibility, zero rework) is finalized at the frozen review commit")
    }
}
