param([string]$Tree = 'D:/autostack/.wt/lang-783/auto-lang')
$ErrorActionPreference = 'Stop'
# Validator controls in a NEW temporary copy; never runs actual 512/30 loads.
$ctl = Join-Path $env:TEMP ('ac783-r4-gate-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path "$ctl/scripts", "$ctl/bin", "$ctl/experimental/ac-artifact-probe/target/release", "$ctl/docs/reports/plan783" -Force | Out-Null
Copy-Item "$Tree/scripts/verify-ac-artifact-probe.ps1" "$ctl/scripts/verify-ac-artifact-probe.ps1"
Set-Content "$ctl/experimental/ac-artifact-probe/Cargo.toml" '# isolated R4 validator control'
$original = Get-Content "$Tree/docs/reports/plan783/perf.json" -Raw
$fixture = $original | ConvertFrom-Json
$fixture.source_commit = '__EXPECTED_HEAD__'
$fixture | ConvertTo-Json -Depth 20 | Set-Content "$ctl/good.json" -Encoding utf8
$fixture.scenarios[0].semantic_failures = 1
$fixture | ConvertTo-Json -Depth 20 | Set-Content "$ctl/bad.json" -Encoding utf8
$stub = @'
@echo off
set ig=0
for %%A in (%*) do if "%%A"=="gc_budget_" set ig=1
for %%A in (%*) do if "%%A"=="--ignored" goto churn
echo test result: ok. 999 passed; 0 failed; %ig% ignored; 0 measured; 0 filtered out;
exit /b 0
:churn
echo test gc_budget_churn_512_converges_between_watermarks ... ok
echo test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
exit /b 0
'@
Set-Content "$ctl/bin/cargo.cmd" $stub
$rust = @'
fn main() {
 if std::env::var("R4_PERF_NOOP").unwrap_or_default()=="1" {return;}
 let a:Vec<String>=std::env::args().collect();let n=a.iter().position(|s|s=="--out").unwrap();
 let s=std::fs::read_to_string(std::env::var("R4_PERF_FIXTURE").unwrap()).unwrap();
 let h=std::env::var("AC_PROBE_SOURCE_COMMIT").unwrap();
 std::fs::write(&a[n+1],s.replace("__EXPECTED_HEAD__",&h)).unwrap();
}
'@
Set-Content "$ctl/fake-node.rs" $rust
& rustc "$ctl/fake-node.rs" --crate-name fake_node -o "$ctl/experimental/ac-artifact-probe/target/release/node.exe"
if ($LASTEXITCODE -ne 0) { throw 'fake node compilation failed' }
$savedPath = $env:PATH
try {
    $env:PATH = "$ctl/bin;$savedPath"
    foreach ($mode in @('valid','invalid-report','no-output','publish-blocked')) {
        $reportPath = "$ctl/docs/reports/plan783/perf.json"
        [IO.File]::WriteAllText($reportPath,$original)
        $before = (Get-FileHash $reportPath).Hash
        $env:R4_PERF_NOOP = if ($mode -eq 'no-output') { '1' } else { '0' }
        $env:R4_PERF_FIXTURE = if ($mode -eq 'invalid-report') { "$ctl/bad.json" } else { "$ctl/good.json" }
        $hold = $null
        try {
            if ($mode -eq 'publish-blocked') {
                $hold = [IO.FileStream]::new($reportPath,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::Read)
            }
            $ErrorActionPreference = 'Continue' # PS5 wraps child stderr; capture it as evidence.
            & powershell -NoProfile -ExecutionPolicy Bypass -File "$ctl/scripts/verify-ac-artifact-probe.ps1" -Scope closeout *> "$ctl/$mode.log"
            $exitCode = $LASTEXITCODE
        } finally { $ErrorActionPreference = 'Stop'; if ($hold) { $hold.Dispose() } }
        $after = (Get-FileHash $reportPath).Hash
        Write-Output "mode=$mode exit=$exitCode old_report_preserved=$($before -eq $after) log=$ctl/$mode.log"
    }
} finally {
    $env:PATH = $savedPath
    Remove-Item Env:R4_PERF_NOOP,Env:R4_PERF_FIXTURE -ErrorAction SilentlyContinue
}
# Correct: valid=0; other three nonzero, old bytes preserved on failure.
