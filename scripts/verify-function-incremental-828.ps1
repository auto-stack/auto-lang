# PLAN-828 function-incremental verification script.
# Scopes: first | frontend | compile | install | lifetime | formal | acceptance
# Usage:  pwsh scripts/verify-function-incremental-828.ps1 [-Scope formal]
#
# Every scope maps to REAL test entrypoints (cargo test filters over
# experimental/ac-av-probe); no scope fabricates a green banner — a scope's
# filter must match at least one test or the run fails.
param(
    [Parameter(Position = 0)]
    [ValidateSet("first", "frontend", "compile", "install", "lifetime", "formal", "acceptance", "all")]
    [string]$Scope = "acceptance"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $root "experimental/ac-av-probe/Cargo.toml"

# Scope -> test-name filters (cargo test substring filters, single-threaded
# for determinism).
$scopes = @{
    first     = @("t01_v01", "t01_v04")
    frontend  = @("t02_frozen_load_projection_parity", "t02_syntax_error")
    compile   = @("t03_signature_change", "t03_body_credentials", "t04_content_addressed")
    install   = @("t04_unit_admission_negatives", "t05_generations", "t06_v05", "t06_v06")
    lifetime  = @("t07_restore", "t07_512_revision", "t07_trap_span")
    formal    = @("t08_v08_formal", "t08_v09_gate_negatives")
    acceptance = @("t01_", "t02_", "t03_", "t04_", "t05_", "t06_", "t07_", "t08_")
}

# The scalar grammar/projection differential suite backs the frontend scope.
$extraFrontend = @("-p ac-syntax-core-adapter --test scalar_projection_")

function Invoke-ScopeTests([string[]]$filters, [string[]]$extraArgs) {
    $failed = $false
    foreach ($f in $filters) {
        Write-Host "==> cargo test --manifest-path $manifest $extraArgs --test function_incremental_828 $f"
        & cargo test --manifest-path $manifest @extraArgs --test function_incremental_828 $f -- --test-threads=1
        if ($LASTEXITCODE -ne 0) { $failed = $true }
    }
    if ($failed) { throw "scope FAILED" }
}

if ($Scope -eq "all") {
    foreach ($k in @("first", "frontend", "compile", "install", "lifetime", "formal", "acceptance")) {
        Write-Host "=== scope: $k ==="
        $f = $scopes[$k]
        $extra = if ($k -in @("frontend", "acceptance")) { $extraFrontend } else { @() }
        if ($extra.Count -gt 0) {
            foreach ($e in $extra) {
                Write-Host "==> cargo test --manifest-path $manifest $e"
                & cargo test --manifest-path $manifest $e
                if ($LASTEXITCODE -ne 0) { throw "scope FAILED" }
            }
        }
        Invoke-ScopeTests $f @()
    }
} else {
    $f = $scopes[$Scope]
    $extra = if ($Scope -in @("frontend", "acceptance")) { $extraFrontend } else { @() }
    if ($extra.Count -gt 0) {
        foreach ($e in $extra) {
            Write-Host "==> cargo test --manifest-path $manifest $e"
            & cargo test --manifest-path $manifest $e
            if ($LASTEXITCODE -ne 0) { throw "scope FAILED" }
        }
    }
    Invoke-ScopeTests $f @()
}
Write-Host "PLAN-828 scope '$Scope' PASSED"
