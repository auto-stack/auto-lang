param([string]$Tree='D:/autostack/.wt/lang-786/auto-lang')
$ErrorActionPreference='Stop'
$dep=Join-Path $Tree 'target/debug/deps'
$rlib=Get-ChildItem $dep -Filter 'libauto_project_model-*.rlib' | Sort-Object LastWriteTime -Descending | Select-Object -First 1
$exe=Join-Path $env:TEMP 'p786-review-probe.exe'
# counterexamples.rs includes the frozen review worktree path; update only when intentionally rebinding the baseline.
rustc --edition=2021 (Join-Path $PSScriptRoot 'counterexamples.rs') -L "dependency=$dep" --extern "auto_project_model=$($rlib.FullName)" -o $exe
if ($LASTEXITCODE -ne 0) { throw 'probe compile failed' }
& $exe
if ($LASTEXITCODE -ne 0) { throw 'combination probe failed' }
$failed=$false
foreach ($depth in @(32,128,512)) {
 $out=Join-Path $env:TEMP "p786-depth-$depth.out"
 $err=Join-Path $env:TEMP "p786-depth-$depth.err"
 $p=Start-Process -FilePath $exe -ArgumentList "$depth" -WindowStyle Hidden -PassThru -RedirectStandardOutput $out -RedirectStandardError $err
 if (-not $p.WaitForExit(10000)) { $p.Kill(); $failed=$true; Write-Output "depth=$depth TIMEOUT" }
 else { Write-Output "depth=$depth exit=$($p.ExitCode)"; if ($p.ExitCode -ne 0) { $failed=$true } }
 Get-Content $out
 Get-Content $err
}
if ($failed) { exit 1 }
