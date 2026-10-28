# PLAN-864 verification gate — scoped runners for the collections SOURCE
# surface (real .at sources through the collections frontend onto the
# frozen 822 execution).
#
# Scopes:
#   targeted   - the cargo test families (probe full-chain + corpus +
#                frontend compile/reject probes) — filter-matched counts are
#                asserted; a filter matching zero tests fails.
#   source     - the file-backed suite through the built auto-case runner
#                (17 rows, zero failing checks).
#   compat     - the 852 base suite stays green (the 822 fixture rows and
#                every legacy case byte-untouched).
#   acceptance - targeted + source + compat chained.
#
# Run: powershell -File scripts/verify-av-864.ps1 -Scope targeted
param(
    [ValidateSet("targeted", "source", "compat", "acceptance")]
    [string]$Scope = "acceptance"
)

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo

function Invoke-Native([string]$what, [scriptblock]$block) {
    # native stderr lines must not trip the Stop preference; the EXIT CODE
    # remains the gate.
    $ErrorActionPreference = "Continue"
    & $block | Out-Null
    $ErrorActionPreference = "Stop"
    if ($LASTEXITCODE -ne 0) { throw "$what failed (exit $LASTEXITCODE)" }
}

function Run-CargoTest([string]$filter, [int]$min) {
    Write-Host "== cargo test $filter (min $min) =="
    Invoke-Native "frontend probes" { cargo test --manifest-path experimental/ac-core/Cargo.toml --test collections_864 --test collections_864_rejects 2>&1 }
    Invoke-Native "full-chain probes" { cargo test --manifest-path experimental/ac-av-probe/probe/Cargo.toml --test av_collections_864 --test av_collections_864_corpus 2>&1 }
    $ErrorActionPreference = "Continue"
    $out = cargo test -p auto-lang --lib --features test-cases -- $filter 2>&1
    $ErrorActionPreference = "Stop"
    if ($LASTEXITCODE -ne 0) { throw "cargo test $filter failed (exit $LASTEXITCODE)" }
    $line = ($out | Select-String "test result:").Line
    if ($line -notmatch "(\d+) passed") { throw "no test-result line for $filter" }
    $passed = [int]$Matches[1]
    if ($passed -lt $min) { throw "$filter matched $passed tests (< $min)" }
    Write-Host "   $passed passed"
}

function Build-AutoCase {
    $bin = Join-Path $repo "target/debug/auto-case.exe"
    if (-not (Test-Path $bin)) {
        Write-Host "   building auto-case..."
        Invoke-Native "build auto-case" { cargo build -p auto-lang --features test-cases --bin auto-case 2>&1 }
    }
    return $bin
}

function Run-SourceSuite {
    Write-Host "== file-backed collections-source suite =="
    $bin = Build-AutoCase
    $scratch = Join-Path $env:TEMP ("av864-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $scratch | Out-Null
    $ErrorActionPreference = "Continue"
    $out = & $bin run --suite (Join-Path $repo "crates/auto-lang/test/cases/ace-m1/collections-source-864.suite.ad") `
        --out $scratch --format console --json-out (Join-Path $scratch "report.json") 2>&1
    $ErrorActionPreference = "Stop"
    if ($LASTEXITCODE -ne 0) { throw "auto-case run failed (exit $LASTEXITCODE)" }
    $json = Get-Content (Join-Path $scratch "report.json") -Raw | ConvertFrom-Json
    $total = @($json.results).Count
    $fails = @($json.results | Where-Object { $_.status -ne "Pass" })
    Write-Host "   $total checks, $($fails.Count) failing"
    if ($total -lt 17) { throw "source suite shrank: $total checks (< 17)" }
    if ($fails.Count -ne 0) { throw ("source suite failing rows: " + ($fails | ConvertTo-Json -Compress)) }
    Remove-Item -Recurse -Force $scratch -ErrorAction SilentlyContinue
}

function Run-Compat {
    Write-Host "== 852 base suite compat (zero drift) =="
    $bin = Build-AutoCase
    $scratch = Join-Path $env:TEMP ("av864c-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $scratch | Out-Null
    $ErrorActionPreference = "Continue"
    & $bin run --suite (Join-Path $repo "crates/auto-lang/test/cases/ace-m1/ace-m1-base.suite.ad") `
        --out $scratch --format console --json-out (Join-Path $scratch "report.json") 2>&1 | Out-Null
    $ErrorActionPreference = "Stop"
    if ($LASTEXITCODE -ne 0) { throw "auto-case run (base) failed (exit $LASTEXITCODE)" }
    $json = Get-Content (Join-Path $scratch "report.json") -Raw | ConvertFrom-Json
    $fails = @($json.results | Where-Object { $_.status -ne "Pass" })
    Write-Host "   $(@($json.results).Count) checks, $($fails.Count) failing"
    if ($fails.Count -ne 0) { throw ("base suite drifted: " + ($fails | ConvertTo-Json -Compress)) }
    Remove-Item -Recurse -Force $scratch -ErrorAction SilentlyContinue
}

switch ($Scope) {
    "targeted" {
        Run-CargoTest "tests::av_collections_864" 1
    }
    "source" {
        Run-SourceSuite
    }
    "compat" {
        Run-Compat
    }
    "acceptance" {
        Run-CargoTest "tests::av_collections_864" 1
        Run-SourceSuite
        Run-Compat
    }
}
Write-Host "verify-av-864 [$Scope]: GREEN"
