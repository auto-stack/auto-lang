$ErrorActionPreference='Stop'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r6'
$helper=(Get-Content (Join-Path $r 'probes.ps1') -Raw).Split('$probes=@(')[0];Invoke-Expression $helper
$cases=@(
 @{id='M01-positional-binary-args';src='fn f(x int,y int) int { return x*10+y } fn main() int { return f(1+2,3*4) }';want=0;native=42},
 @{id='M02-named-binary-args';src='fn f(x int,y int) int { return x*10+y } fn main() int { return f(y:3*4,x:1+2) }';want=0;native=42},
 @{id='M03-unclosed-paren-binary';src='fn main() int { return (1+2 }';want=1;native=3},
 @{id='M04-unclosed-call-binary';src='fn f(x int) int { return x*10 } fn main() int { return f(1+2 }';want=1;native=30},
 @{id='M05-unclosed-undefined-call-binary';src='fn main() int { return missing(1+2 }';want=1;native=3},
 @{id='M06-nested-call-args';src='fn f(x int,y int) int { return x*10+y } fn main() int { return f(f(1,2)+3,f(2,1)*2) }';want=0;native=192},
 @{id='M07-comparison-call-arg';src='fn f(a bool,b int) int { if a { return b } else { return 0 } } fn main() int { return f(1+2<4,5) }';want=0;native=5},
 @{id='M08-missing-group-before-semicolon';src='fn main() int { let x = (1+2; return x }';want=1;native=3},
 @{id='M09-missing-call-before-semicolon';src='fn f(x int) int { return x*10 } fn main() int { let x = f(1+2; return x }';want=1;native=30},
 @{id='M10-parenthesized-comma-control';src='fn f(x int,y int) int { return x*10+y } fn main() int { return f((1+2),(3*4)) }';want=0;native=42},
 @{id='M11-unclosed-paren-literal-control';src='fn main() int { return (3 }';want=1},
 @{id='M12-unclosed-call-literal-control';src='fn f(x int) int { return x*10 } fn main() int { return f(3 }';want=1},
 @{id='M13-nested-unclosed-empty-call';src='fn f() int { return 3 } fn main() int { return (f()+2 }';want=1;native=5},
 @{id='M14-stray-comma-control';src='fn main() int { return (1+2,3) }';want=1},
 @{id='M15-binary-arg-last-control';src='fn f(x int,y int) int { return x*10+y } fn main() int { return f(1,2+3) }';want=0;native=15},
 @{id='M16-unsupported-op-next-line';src="fn main() int { let x = 1`n-2; return x }";want=1}
)
$rows=@()
foreach($c in $cases){
 $dir=Join-Path $r ('extra-probes/'+$c.id);New-Item -ItemType Directory -Force $dir|Out-Null
 $path=Join-Path $dir 'input.at';[IO.File]::WriteAllText($path,$c.src)
 $check=Invoke-ReviewProcess $bin @('check-source',$path,'--source-profile',$profile) 10
 $build=$null;$run=$null
 if($c.ContainsKey('native') -and $check.exit -eq 0){$exe=Join-Path $dir 'app.exe';$build=Invoke-ReviewProcess $bin @('build-source',$path,'--source-profile',$profile,'--entry','main','--output',$exe);if($build.exit -eq 0){$run=Invoke-ReviewProcess $exe @()}}
 $hashes=@();if($build.exit -eq 0 -and $build){foreach($file in @('app.exe','app.obj','app.ac-link.txt')){$p=Join-Path $dir $file;$hashes+=@{name=$file;sha256=(Get-FileHash $p).Hash.ToLower()}}}
 $row=@{id=$c.id;bytes=$c.src.Length;check=$check;build=$build;run=$run;want=$c.want;want_native=$c.native;artifact_hashes=$hashes;input_sha256=(Get-FileHash $path).Hash.ToLower()};$rows+=$row
 $row|ConvertTo-Json -Depth 7|Set-Content (Join-Path $dir 'result.json') -Encoding utf8
 Write-Output ($c.id+' check='+$check.exit+' run='+$run.exit+' diag='+$check.stderr.Trim())
}
$rows|ConvertTo-Json -Depth 7|Set-Content (Join-Path $r 'parser-results.json') -Encoding utf8