param([string]$Tree='D:/autostack/.wt/lang-786/auto-lang')
$ErrorActionPreference='Stop'
$dep=Join-Path $Tree 'target/debug/deps'
$lib=Get-ChildItem $dep -Filter 'libauto_project_model-*.rlib' | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$exe=Join-Path $env:TEMP 'p786-r3-probe.exe'
rustc --edition=2021 (Join-Path $PSScriptRoot 'counterexamples.rs') -L "dependency=$dep" --extern "auto_project_model=$($lib.FullName)" -o $exe
if ($LASTEXITCODE -ne 0) { throw 'compile failure' }
$failed=$false
foreach ($case in @(@('plain',512),@('multiline',512),@('single-unclosed',32),@('single-unclosed',128),@('single-unclosed',512),@('single-closed-later',512),@('f-unclosed',512),@('call-unclosed',512))) {
 $id="$($case[0])-$($case[1])"
 $out=Join-Path $env:TEMP "p786-r3-$id.out"; $err=Join-Path $env:TEMP "p786-r3-$id.err"
 $inputFile=Join-Path $PSScriptRoot "$id.at"
 $p=Start-Process -FilePath $exe -ArgumentList @($case[0],$case[1],$inputFile) -WindowStyle Hidden -PassThru -RedirectStandardOutput $out -RedirectStandardError $err
 if (-not $p.WaitForExit(10000)) { $p.Kill(); $p.WaitForExit(); $failed=$true; Write-Output "$id TIMEOUT killed/reaped" } else { Write-Output "$id exit=$($p.ExitCode)"; if ($p.ExitCode -ne 0) { $failed=$true } }
 Get-Content $out; Get-Content $err
}
if ($failed) { exit 1 }
