# PLAN-863: real parent-side byte/result oracle gate. No full-suite aliases.
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('targeted', 'compat', 'console', 'acceptance')][string]$Scope,
    [string]$RepoRoot = (Split-Path -Parent $PSScriptRoot),
    [Parameter(Mandatory = $true)][string]$OutDir
)
. (Join-Path $PSScriptRoot 'test-governance/ace-cargo.ps1')

$ErrorActionPreference = 'Stop'
$RepoRoot = (Resolve-Path -LiteralPath $RepoRoot).Path
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$OutDir = (Resolve-Path -LiteralPath $OutDir).Path
function Fail([string]$message) { throw "verify-av-863 [$Scope]: $message" }
function Invoke-Checked([string]$label, [string[]]$Arguments, [int]$Minimum) {
    $log = Join-Path $OutDir "$label.log"
    & Invoke-AceCargo @Arguments *> $log
    if ($LASTEXITCODE -ne 0) { Get-Content $log -Tail 30 | Write-Host; Fail "$label failed; $log" }
    $passed = 0
    foreach ($line in Get-Content $log) {
        if ($line -match 'test result: ok\. (\d+) passed') { $passed += [int]$Matches[1] }
    }
    if ($passed -lt $Minimum) { Fail "$label executed $passed tests, expected at least $Minimum" }
    Write-Host "$label PASS ($passed tests)"
}
function Invoke-Suite([string]$suite, [string]$label) {
    $bin = if ($env:AUTO_CASE_BIN) { $env:AUTO_CASE_BIN } else { Join-Path $RepoRoot 'target/debug/auto-case.exe' }
    if (-not (Test-Path -LiteralPath $bin)) { Fail "prebuild auto-case with Invoke-AceCargo build -p auto-lang --features test-cases --bin auto-case" }
    $dest = Join-Path $OutDir $label
    New-Item -ItemType Directory -Force -Path $dest | Out-Null
    $json = Join-Path $dest 'report.json'
    & $bin run --suite $suite --out $dest --format console --json-out $json *> (Join-Path $dest 'runner.log')
    $code = $LASTEXITCODE
    if (-not (Test-Path -LiteralPath $json)) { Fail "$label produced no report (exit $code)" }
    $report = Get-Content -LiteralPath $json -Raw | ConvertFrom-Json
    if ($code -ne 0 -or @($report.results).Count -eq 0 -or @($report.results | Where-Object status -ne 'PASS').Count -ne 0 -or @($report.load_errors).Count -gt 0 -or @($report.coverage_gaps).Count -gt 0) {
        Fail "$label must execute all required checks as PASS (SKIP/NOT_RUN are not green); $json"
    }
    Write-Host "$label PASS ($(@($report.results).Count) checks)"
    return $report
}
function Console {
    $suite = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1/console-863/console-863.suite.ad'
    $report = Invoke-Suite $suite 'console'
    if (@($report.results).Count -ne 15) { Fail 'console corpus requires exactly 15 checks' }
    foreach ($row in $report.results) {
        $axes = @{}
        foreach ($pair in $row.native.toolchain) { $axes[[string]$pair[0]] = [string]$pair[1] }
        foreach ($want in @(@('recipe', 'ac-av/console-run-v1'), @('av-profile', '8'), @('mir-lowering', '3'), @('av-core-version', 'av.host-console/1'), @('av-wire-codec', 'auto.abc.av-host-console/1'))) {
            if ($axes[$want[0]] -cne $want[1]) { Fail "$($row.case): identity $($want[0]) mismatch" }
        }
    }
    # Actual sabotage, missing-axis, raw NUL and resource controls use the same parent dispatch.
    Invoke-Checked 'parent-controls' @('test','-p','ace-conformance','--features','test-cases','--test','conformance','console::') 5
}
Push-Location -LiteralPath $RepoRoot
try {
    switch ($Scope) {
        'console' { Console }
        'compat' {
            # Existing scalar/data fixture contracts and entry/Unit contracts remain unchanged.
            $null = Invoke-Suite (Join-Path $RepoRoot 'crates/auto-lang/test/cases/ac-av/ac-av.suite.ad') 'legacy-av'
            $null = Invoke-Suite (Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1/entry-862/entry-unit-862.suite.ad') 'entry-unit'
        }
        'targeted' {
            Invoke-Checked 'contract' @('test','-p','auto-test-case','--test','contract_decode') 25
            Invoke-Checked 'parent-admission' @('test','-p','auto-lang-core','--features','test-cases,test-ace','--lib','test_cases::ac_av::console_tests::') 2
            Invoke-Checked 'source' @('test','--manifest-path','experimental/ace/crates/core/Cargo.toml','--test','host_console') 8
            Invoke-Checked 'runtime' @('test','--manifest-path','experimental/ace/Cargo.toml','-p','ace-av','--test','host_console') 18
        }
        'acceptance' {
            foreach ($part in @('targeted','compat','console')) {
                & $PSCommandPath -Scope $part -RepoRoot $RepoRoot -OutDir (Join-Path $OutDir $part)
                if (-not $?) { Fail "scope $part failed" }
            }
            $rows = @{}
            foreach ($line in Get-Content (Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1/binding.tsv')) {
                if (-not $line.Trim() -or $line.StartsWith('#')) { continue }
                $f = $line.Split([char]9)
                if ($f.Count -ne 12) { Fail 'binding schema must retain 12 columns' }
                $key = "$($f[1]):$($f[0])"
                if ($rows.ContainsKey($key)) { Fail "duplicate binding $key" }
                $rows[$key] = $f
            }
            foreach ($n in 1..15) {
                $id = 'O{0:D2}' -f $n
                $row = $rows["corpus:$id"]
                if (-not $row -or $row[6] -cne 'ok:av-host-console-source-863') { Fail "$id lacks measured AV binding" }
                if ($row[5] -notlike 'gap:*' -or $row[7] -notlike 'gap:*' -or $row[8] -notlike 'gap:*') { Fail "$id peer coverage requires its own evidence" }
            }
            Write-Host '15 console AV bindings verified; peer parity remains an explicit gap. This is not a whole-M1 certificate.'
        }
    }
    Write-Host "verify-av-863 [$Scope]: PASS"
} finally { Pop-Location }
