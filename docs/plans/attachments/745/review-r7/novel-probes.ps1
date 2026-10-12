$ErrorActionPreference='Stop'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r7'
$helper=(Get-Content (Join-Path $r 'probes.ps1') -Raw).Split('$probes=@(')[0];Invoke-Expression $helper
$cases=@(
 @{id='H01-rhs-parens-16';src='fn main() int { return '+('1+('*16)+'0'+(')'*16)+' }';want=0},
 @{id='H02-rhs-parens-200';src='fn main() int { return '+('1+('*200)+'0'+(')'*200)+' }';want=0},
 @{id='H03-rhs-parens-1000';src='fn main() int { return '+('1+('*1000)+'0'+(')'*1000)+' }';want=0},
 @{id='H04-rhs-parens-4096';src='fn main() int { return '+('1+('*4096)+'0'+(')'*4096)+' }';want=0},
 @{id='H05-unclosed-rhs-parens-4096';src='fn main() int { return '+('1+('*4096)+'0 }';want=1},
 @{id='H06-pure-parens-100000';src='fn main() int { return '+('('*100000)+'1'+(')'*100000)+' }';want=0},
 @{id='H07-calls-1001';src='fn f(x int) int { return x } fn main() int { return '+('f('*1001)+'1'+(')'*1001)+' }';want=0},
 @{id='H08-if-blocks-1001';src='fn main() int { '+('if true {'*1001)+'return 1'+('}'*1001)+'; return 0 }';want=0},
 @{id='H09-parens-precedence';src='fn f(x int,y int) int { return x * 10 + y } fn main() int { return f(((1)+2)*3, (2*(1+2))) + (1+(2*(3+4))) }';want=0;native=111},
 @{id='H10-parens-comparison';src='fn main() int { if ((1+(2*3)) < ((2+2)*2)) { return 5 } else { return 0 } }';want=0;native=5}
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
$rows|ConvertTo-Json -Depth 7|Set-Content (Join-Path $r 'novel-results.json') -Encoding utf8