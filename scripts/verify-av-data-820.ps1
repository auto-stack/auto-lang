# PLAN-820 verification script — av.data/1 profile gates.
# Scopes (one per invocation):
#   first-case   — T-02 first case whole-path + wide-scalar fixtures +
#                  store/new-worker restore (the av_data_820 family).
#   semantics | admission | resources | cache | compatibility | autodown |
#   acceptance — later tasks (T-03..T-06) wire these; today they exit 9
#                (not-yet-wired) so an accidental green is impossible.
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet(
        "first-case", "semantics", "admission", "resources",
        "cache", "compatibility", "autodown", "acceptance"
    )]
    [string]$Scope
)

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $repo "experimental/ac-av-probe/Cargo.toml"

function Fail([string]$msg) {
    Write-Host "[verify-av-data-820] FAIL: $msg" -ForegroundColor Red
    exit 1
}

function Need-Success([object]$proc, [string]$what) {
    if ($proc.ExitCode -ne 0) {
        Fail "$what exited $($proc.ExitCode)"
    }
}

switch ($Scope) {
    "first-case" {
        # The full first-case family must SELECT > 0 tests (0-run = fail).
        $out = & cargo test --manifest-path $manifest -p auto-av-probe --test av_data_820 2>&1
        $code = $LASTEXITCODE
        $text = ($out | Out-String)
        if ($code -ne 0) { Fail "av_data_820 family red:`n$text" }
        if ($text -notmatch "10 passed") {
            Fail "av_data_820 family must select and pass all 10 first-case tests; output:`n$text"
        }
        Write-Host "[verify-av-data-820] first-case PASS (av_data_820 10/10)" -ForegroundColor Green
        exit 0
    }
    default {
        Write-Host "[verify-av-data-820] scope '$Scope' is not wired yet (T-03..T-06 own it)" -ForegroundColor Yellow
        exit 9
    }
}
