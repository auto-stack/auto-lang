$ErrorActionPreference='Stop'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r6'
$helper=(Get-Content (Join-Path $r 'probes.ps1') -Raw).Split('$probes=@(')[0];Invoke-Expression $helper
$cases=@(
 @{id='M17-wrapped-trace-args';src='fn f(a int,b int) int { return a*10+b } fn main() int { return f(b:(mark_b()+1),a:(mark_a()*2)) }';want=0;native=23;trace=$true;stderr='ba'},
 @{id='M18-binary-trace-args';src='fn f(a int,b int) int { return a*10+b } fn main() int { return f(b:mark_b()+1,a:mark_a()*2) }';want=0;native=23;trace=$true;stderr='ba'},
 @{id='M19-unclosed-if-condition';src='fn main() int { if (1<2 { return 5 } else { return 0 } }';want=1;native=5},
 @{id='M20-unclosed-while-condition';src='fn main() int { var n=0; while (n<3 { n=n+1 }; return n }';want=1;native=3}
)
$rows=@()
foreach($c in $cases){
 $dir=Join-Path $r ('extra-probes/'+$c.id);New-Item -ItemType Directory -Force $dir|Out-Null
 $path=Join-Path $dir 'input.at';[IO.File]::WriteAllText($path,$c.src)
 $extra=@();if($c.trace){$extra=@('--capability','hir.test.trace.v1')}
 $check=Invoke-ReviewProcess $bin (@('check-source',$path,'--source-profile',$profile)+$extra) 10
 $build=$null;$run=$null
 if($c.ContainsKey('native') -and $check.exit -eq 0){$exe=Join-Path $dir 'app.exe';if($c.trace){$extra+=@('--support-lib',(Join-Path $repo 'experimental/ac-core/test-support/target/debug/ac_trace_support.lib'))};$build=Invoke-ReviewProcess $bin (@('build-source',$path,'--source-profile',$profile,'--entry','main','--output',$exe)+$extra);if($build.exit -eq 0){$run=Invoke-ReviewProcess $exe @()}}
 $row=@{id=$c.id;bytes=$c.src.Length;check=$check;build=$build;run=$run;want=$c.want;want_native=$c.native};$rows+=$row
 $row|ConvertTo-Json -Depth 7|Set-Content (Join-Path $dir 'result.json') -Encoding utf8
 Write-Output ($c.id+' check='+$check.exit+' run='+$run.exit+' diag='+$check.stderr.Trim())
}
$rows|ConvertTo-Json -Depth 7|Set-Content (Join-Path $r 'parser-extra-results.json') -Encoding utf8