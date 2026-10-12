$ErrorActionPreference='Stop'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r3'
$w='D:/autostack/.wt/lang-745/auto-lang'
$full=[IO.File]::ReadAllText((Join-Path $w 'scripts/verify-ac-source-745.ps1'))
$start=$full.IndexOf('function Invoke-Captured {');$end=$full.IndexOf('function Get-Sha256 {')
$exact=$full.Substring($start,$end-$start)
[IO.File]::WriteAllText((Join-Path $r 'script-helper-exact.ps1.txt'),$exact)
Invoke-Expression $exact
$env:AC745_REVIEW_PID_PATH=Join-Path $r 'ps-descendant-pid.txt'
$exe=Join-Path $w 'experimental/ac-core/target/review-r3/helper_probe.exe'
$sw=[Diagnostics.Stopwatch]::StartNew()
$run=Invoke-Captured $exe @('--root-exits') $w 1
$elapsed=$sw.ElapsedMilliseconds
$taskPid=[int]([IO.File]::ReadAllText($env:AC745_REVIEW_PID_PATH))
$alive=$null -ne (Get-Process -Id $taskPid -ErrorAction SilentlyContinue)
if($alive){Stop-Process -Id $taskPid}
$utf=Invoke-Captured $exe @('--invalid-utf8') $w 1
$result=@{deadline_ms=1000;elapsed_ms=$elapsed;run=$run;descendant_alive_after_return=$alive;utf8=$utf}
$result|ConvertTo-Json -Depth 6|Set-Content (Join-Path $r 'script-helper-results.json')
$result|ConvertTo-Json -Depth 6|Write-Output
