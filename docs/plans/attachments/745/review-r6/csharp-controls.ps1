$ErrorActionPreference='Stop'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r6'
$w='D:/autostack/.wt/lang-745/auto-lang'
$full=[IO.File]::ReadAllText((Join-Path $w 'scripts/verify-ac-source-745.ps1'))
$a=$full.IndexOf('$script:AcRunnerCs =');$b=$full.IndexOf('function Get-Sha256 {');if($a -lt 0 -or $b -lt $a){throw 'runner seam absent'}
$exact=$full.Substring($a,$b-$a);[IO.File]::WriteAllText((Join-Path $r 'csharp-runner-exact.ps1.txt'),$exact)
Invoke-Expression $exact
Add-Type -TypeDefinition $script:AcRunnerCs -Language CSharp
$script:AcRunnerLoaded=$true
$env:AC745_REVIEW_PID_PATH=Join-Path $r 'csharp-descendant-pid.txt'
$probeExe=Join-Path $w 'experimental/ac-core/target/review-r6/helper_probe.exe'
$rows=@()
foreach($c in @(
 @{id='hold';timeout=500;mode='--root-exits';timeoutWant=$true;stdout=''},
 @{id='late';timeout=3000;mode='--root-late';timeoutWant=$false;stdout='LATE'},
 @{id='running-parent';timeout=500;mode='--root-waits';timeoutWant=$true;stdout=''},
 @{id='invalid-utf8';timeout=1000;mode='--invalid-utf8';timeoutWant=$false;stdout=('PREFIX'+[char]0xfffd+'SUFFIX')}
)){
 $sw=[Diagnostics.Stopwatch]::StartNew();$res=[AcContainedRunner]::Run($probeExe,@($c.mode),$w,$c.timeout)
 $alive=$false;if($c.mode -ne '--invalid-utf8'){$taskPid=[int]([IO.File]::ReadAllText($env:AC745_REVIEW_PID_PATH));$alive=$null -ne (Get-Process -Id $taskPid -ErrorAction SilentlyContinue);if($alive){Stop-Process -Id $taskPid}}
 $outputOk=if($c.mode -eq '--invalid-utf8'){$res.Stdout -ceq $c.stdout}elseif($c.mode -eq '--root-late'){$res.Stdout -match '^DESCENDANT_PID=\d+\nLATE\n$'}else{$res.Stdout -match '^DESCENDANT_PID=\d+\n$'}; $rows+=@{id=$c.id;deadline_ms=$c.timeout;elapsed_ms=$sw.ElapsedMilliseconds;result=$res;descendant_alive=$alive;ok=(-not $alive -and -not $res.Error -and $res.TimedOut -eq $c.timeoutWant -and $outputOk)}
}
foreach($c in @(@{id='actual-assignment-failure';assign=$true;read=$false},@{id='actual-read-failure';assign=$false;read=$true},@{id='valid-assignment-and-read-control';assign=$false;read=$false})){
 $sw=[Diagnostics.Stopwatch]::StartNew();$res=[AcContainedRunner]::RunControl('cmd.exe',@('/c','echo READ_CONTROL'),$w,1000,$c.assign,$c.read)
 $alive=$null -ne (Get-Process -Id $res.Pid -ErrorAction SilentlyContinue)
 $ok=-not $alive
 if($c.assign){$ok=$ok -and $res.Error -eq 'containment unavailable'}elseif($c.read){$ok=$ok -and $res.Error -like 'output read failed*' -and $res.StdoutReadError -eq 6}else{$ok=$ok -and -not $res.Error -and -not $res.TimedOut -and $res.Code -eq 0 -and $res.Stdout -ceq "READ_CONTROL`r`n"}
 $rows+=@{id=$c.id;elapsed_ms=$sw.ElapsedMilliseconds;result=$res;child_alive_after_return=$alive;ok=[bool]$ok;control='Unmodified RunControl executes real native failure'}
 if($alive){Stop-Process -Id $res.Pid}
}
$wa=$exact.IndexOf('function Invoke-Captured {');$wrapper=$exact.Substring($wa)
foreach($c in @(@{id='wrapper-read-failure';assign='$false';read='$true'},@{id='wrapper-assignment-failure';assign='$true';read='$false'})){
 $fault=$wrapper.Replace('function Invoke-Captured {','function Invoke-ReviewFault {').Replace('[AcContainedRunner]::Run($Exe, $ArgList, $WorkDir, ($TimeoutSec * 1000))',('[AcContainedRunner]::RunControl($Exe, $ArgList, $WorkDir, ($TimeoutSec * 1000), '+$c.assign+', '+$c.read+')'))
 if(-not $fault.Contains('::RunControl(')){throw 'wrapper fault not wired'}
 [IO.File]::WriteAllText((Join-Path $r ($c.id+'.ps1.txt')),$fault);Invoke-Expression $fault
 $caught=$null;try{Invoke-ReviewFault 'cmd.exe' @('/c','echo READ_CONTROL') $w 1|Out-Null}catch{$caught=$_.Exception.Message}
 $rows+=@{id=$c.id;error=$caught;ok=[bool]($caught -like 'contained executor refused*')}
}
$rows|ConvertTo-Json -Depth 7|Set-Content (Join-Path $r 'csharp-control-results.json') -Encoding utf8
$rows|Select-Object id,ok,elapsed_ms|Format-Table|Out-String|Write-Output
if(@($rows|Where-Object {-not $_.ok}).Count){throw 'C# controls failed'}