$ErrorActionPreference='Stop'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r5'
$helper=(Get-Content (Join-Path $r 'probes.ps1') -Raw).Split('$probes=@(')[0];Invoke-Expression $helper
$cases=@(
 @{id='K01-call-999-native';src='fn f(x int) int { return x } fn main() int { return '+('f('*999)+'1'+(')'*999)+' }';want=0;native=1},
 @{id='K02-if-999-native';src='fn main() int { '+('if true {'*999)+'return 1'+('}'*999)+'; return 0 }';want=0;native=1},
 @{id='K03-call-1000';src='fn f(x int) int { return x } fn main() int { return '+('f('*1000)+'1'+(')'*1000)+' }';want=0},
 @{id='K04-if-1000';src='fn main() int { '+('if true {'*1000)+'return 1'+('}'*1000)+'; return 0 }';want=0}
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
$rows|ConvertTo-Json -Depth 7|Set-Content (Join-Path $r 'resource-results.json') -Encoding utf8