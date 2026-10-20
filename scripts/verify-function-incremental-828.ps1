# PLAN-828 function-incremental verification script (F-828-R1-13 repair).
# Scopes: first | frontend | compile | install | lifetime | formal | acceptance
# Usage:  pwsh scripts/verify-function-incremental-828.ps1 [-Scope formal]
#
# Real argument arrays per invocation; adapter-workspace tests run from the
# ac-syntax workspace manifest; a filter that selects ZERO tests fails the
# scope (the libtest summaries are parsed for the selected count).

param(
    [Parameter(Position = 0)]
    [ValidateSet("first", "frontend", "compile", "install", "lifetime", "formal", "acceptance", "all")]
    [string]$Scope = "acceptance"
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$probeManifest = Join-Path $root "experimental/ac-av-probe/Cargo.toml"
$syntaxManifest = Join-Path $root "experimental/ac-syntax/Cargo.toml"

function ProbeInv([string]$f) {
    @{ manifest = $probeManifest; pkg = @(); suite = "function_incremental_828"; filter = $f }
}
function R1Inv([string]$f) {
    @{ manifest = $probeManifest; pkg = @(); suite = "function_incremental_828_r1"; filter = $f }
}
function ScalarInv([string]$f) {
    @{ manifest = $syntaxManifest; pkg = @("-p", "ac-syntax-core-adapter"); suite = "scalar_projection_"; filter = $f }
}

$first = @(ProbeInv "t01_v01"; ProbeInv "t01_v04")
$frontend = @(ProbeInv "t02_"; R1Inv "r1_recursion"; ScalarInv "")
$compile = @(ProbeInv "t03_"; ProbeInv "t04_content"; R1Inv "r1_credential")
$install = @(ProbeInv "t04_unit"; ProbeInv "t05_"; ProbeInv "t06_"; R1Inv "r1_prepared"; R1Inv "r1_recursion")
$lifetime = @(ProbeInv "t07_"; R1Inv "r1_restored"; R1Inv "r1_trace"; R1Inv "r1_session_preserves")
$formal = @(ProbeInv "t08_"; R1Inv "r1_")
$acceptance = @(
    ProbeInv "t01_"; ProbeInv "t02_"; ProbeInv "t03_"; ProbeInv "t04_";
    ProbeInv "t05_"; ProbeInv "t06_"; ProbeInv "t07_"; ProbeInv "t08_";
    R1Inv "r1_"; ScalarInv ""
)

$scopes = @{
    first      = $first
    frontend   = $frontend
    compile    = $compile
    install    = $install
    lifetime   = $lifetime
    formal     = $formal
    acceptance = $acceptance
}

function Invoke-Suite($inv) {
    $manifest = $inv.manifest
    $pkg = $inv.pkg
    $suite = $inv.suite
    $f = $inv.filter
    Write-Host "==> cargo test --manifest-path $manifest $($pkg -join ' ') --test $suite $f"
    # Native stderr (compiler warnings) is data, not a script error:
    # gate on the exit code, not on the error-record stream.
    $prev = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $out = & cargo test --manifest-path $manifest @pkg --test $suite $f -- --test-threads=1 2>&1
    $code = $LASTEXITCODE
    $ErrorActionPreference = $prev
    $text = $out | Out-String
    Write-Host $text
    if ($code -ne 0) { throw "scope FAILED: $suite '$f' exited $code" }
    # Zero-selection guard: cargo exits 0 on no matches; parse the libtest
    # summaries ("test result: ok. N passed; M failed; ...").
    $results = [regex]::Matches($text, "test result: \w+\. (\d+) passed; (\d+) failed")
    if ($results.Count -eq 0) {
        throw "scope FAILED: filter '$f' on $suite produced no test-result summary (zero-match guard)"
    }
    $passed = 0
    $failed = 0
    foreach ($m in $results) {
        $passed += [int]$m.Groups[1].Value
        $failed += [int]$m.Groups[2].Value
    }
    if ($passed -eq 0) {
        throw "scope FAILED: filter '$f' on $suite matched zero passing tests (zero-match guard)"
    }
    if ($failed -ne 0) {
        throw "scope FAILED: filter '$f' on $suite had $failed failing tests"
    }
}

if ($Scope -eq "all") {
    foreach ($k in @("first", "frontend", "compile", "install", "lifetime", "formal", "acceptance")) {
        Write-Host "=== scope: $k ==="
        foreach ($inv in $scopes[$k]) { Invoke-Suite $inv }
    }
} else {
    foreach ($inv in $scopes[$Scope]) { Invoke-Suite $inv }
}
Write-Host "PLAN-828 scope '$Scope' PASSED"
