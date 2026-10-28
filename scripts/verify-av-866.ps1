# verify-av-866.ps1 — PLAN-866 AV resident online hot reload (RF-H) verifier.
#
# Scopes (each fails loudly; a scope that is not wired exits 3, never green):
#   targeted   the phase-1 private controls: the resident run driver unit
#              suite (slice invariance, restart-variant red, cancel/stop,
#              zero-fuel park, candidate-queue discipline) plus the frozen
#              reload contract document's presence.
#   resident   the ONLINE machinery (safety-point pause, swap-resolution
#              resume, effective-iteration oracle, V-02/V-03/V-04 online
#              controls). Not wired until the S/H public surface lands
#              (core av.rs safety-point hook + session/driver wiring after
#              the PLAN-864 write-surface release).
#   compat     the affected-profile compatibility pass (V-07) over the
#              post-864 merged base. Not wired yet.
#   acceptance the V-05/V-06 accounting (Weak release oracle, churn, old-
#              generation cap, P90<=2000ms reload visibility, untouched-
#              function zero-rework). Not wired yet; merge-owned.
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
        NotWired("online safety-point machinery awaits the S/H public surface (post-864 release): core av.rs call-boundary hook, swap-resolution resume, session/driver wiring")
    }
    'compat' {
        NotWired("affected-profile compatibility (V-07) runs on the post-864 merged base with the H corpus rows registered")
    }
    'acceptance' {
        NotWired("V-05/V-06 accounting (Weak release, churn, old-gen cap, P90 reload visibility, zero rework) is finalized at the frozen review commit")
    }
}
