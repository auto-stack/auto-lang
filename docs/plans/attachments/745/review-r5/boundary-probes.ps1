$ErrorActionPreference='Stop'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r5'
$helper=(Get-Content (Join-Path $r 'probes.ps1') -Raw).Split('$probes=@(')[0];Invoke-Expression $helper
$cases=@(
 @{id='G01-valid-parens-199';src='fn main() int { return '+('('*199)+'1'+(')'*199)+' }'},
 @{id='G02-valid-parens-200';src='fn main() int { return '+('('*200)+'1'+(')'*200)+' }'},
 @{id='G03-valid-calls-100';src='fn f(x int) int { return x } fn main() int { return '+('f('*100)+'1'+(')'*100)+' }'},
 @{id='G04-valid-calls-199';src='fn f(x int) int { return x } fn main() int { return '+('f('*199)+'1'+(')'*199)+' }'},
 @{id='G05-mixed-calls-binary-199';src='fn f(x int) int { return x } fn main() int { return '+('f(1+'*199)+'1'+(')'*199)+' }'},
 @{id='G06-nested-if-199';src='fn main() int { '+('if true {'*199)+'return 1'+('}'*199)+'; return 0 }'},
 @{id='G07-native-chain-300';src='fn main() int { return '+('1+'*300)+'0 }';build=$true;want=300},
 @{id='G08-trap-before-rhs-observable';src='fn main() int { return (2147483647 + 1) + mark_b() }';build=$true;trace=$true;want=70;stderr=''},
 @{id='G09-nested-trace-call-order';src='fn f(a int,b int) int { return a * 10 + b } fn main() int { return f(b:mark_b(),a:mark_a()) + mark_a() * 10 + mark_b() }';build=$true;trace=$true;want=24;stderr='baab'}
)
$rows=@()
foreach($c in $cases){
 $dir=Join-Path $r ('extra-probes/'+$c.id);New-Item -ItemType Directory -Force $dir|Out-Null
 $path=Join-Path $dir 'input.at';[IO.File]::WriteAllText($path,$c.src)
 $extra=@();if($c.trace){$extra=@('--capability','hir.test.trace.v1','--support-lib',(Join-Path $repo 'experimental/ac-core/test-support/target/debug/ac_trace_support.lib'))}
 $check=Invoke-ReviewProcess $bin @('check-source',$path,'--source-profile',$profile)
 if($c.trace){$check=Invoke-ReviewProcess $bin @('check-source',$path,'--source-profile',$profile,'--capability','hir.test.trace.v1')}
 $build=$null;$run=$null
 if($c.build -and $check.exit -eq 0){$exe=Join-Path $dir 'app.exe';$build=Invoke-ReviewProcess $bin (@('build-source',$path,'--source-profile',$profile,'--entry','main','--output',$exe)+$extra);if($build.exit -eq 0){$run=Invoke-ReviewProcess $exe @()}}
 $row=@{id=$c.id;bytes=$c.src.Length;check=$check;build=$build;run=$run;want=$c.want;want_stderr=$c.stderr};$rows+=$row
 $row|ConvertTo-Json -Depth 7|Set-Content (Join-Path $dir 'result.json')
 Write-Output ($c.id+' check='+$check.exit+' run='+$run.exit+' stderr='+$run.stderr+' diag='+$check.stderr.Trim())
}
$rows|ConvertTo-Json -Depth 7|Set-Content (Join-Path $r 'boundary-results.json')
