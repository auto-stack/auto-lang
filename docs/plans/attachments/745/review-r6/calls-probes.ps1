$ErrorActionPreference='Stop'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r6'
$helper=(Get-Content (Join-Path $r 'probes.ps1') -Raw).Split('$probes=@(')[0];Invoke-Expression $helper
$cases=@(
 @{id='L01-calls-20000';src='fn f(x int) int { return x } fn main() int { return '+('f('*20000)+'1'+(')'*20000)+' }';want=0},
 @{id='L02-mixed-call-binary-4096-native';src='fn f(x int) int { return x } fn main() int { return '+('f(1+'*4096)+'1'+(')'*4096)+' }';want=0;native=4097},
 @{id='L03-unclosed-calls-20000';src='fn f(x int) int { return x } fn main() int { return '+('f('*20000)+'1 }';want=1}
)
$rows=@()
foreach($c in $cases){
 $dir=Join-Path $r ('extra-probes/'+$c.id);New-Item -ItemType Directory -Force $dir|Out-Null
 $path=Join-Path $dir 'input.at';[IO.File]::WriteAllText($path,$c.src)
 $check=Invoke-ReviewProcess $bin @('check-source',$path,'--source-profile',$profile) 10
 $build=$null;$run=$null
 if($c.ContainsKey('native') -and $check.exit -eq 0){$exe=Join-Path $dir 'app.exe';$build=Invoke-ReviewProcess $bin @('build-source',$path,'--source-profile',$profile,'--entry','main','--output',$exe);if($build.exit -eq 0){$run=Invoke-ReviewProcess $exe @()}}
 $row=@{id=$c.id;bytes=$c.src.Length;check=$check;build=$build;run=$run;want=$c.want;want_native=$c.native};$rows+=$row
 $row|ConvertTo-Json -Depth 7|Set-Content (Join-Path $dir 'result.json') -Encoding utf8
 Write-Output ($c.id+' check='+$check.exit+' run='+$run.exit+' diag='+$check.stderr.Trim())
}
$rows|ConvertTo-Json -Depth 7|Set-Content (Join-Path $r 'calls-results.json') -Encoding utf8