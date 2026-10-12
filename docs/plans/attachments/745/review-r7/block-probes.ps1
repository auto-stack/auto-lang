$ErrorActionPreference='Stop'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r7'
$helper=(Get-Content (Join-Path $r 'probes.ps1') -Raw).Split('$probes=@(')[0];Invoke-Expression $helper
$cases=@(
 @{id='N01-sibling-if-joins';src='fn main() int { var x=0; if true { x=1 } else { x=2 }; if false { x=x+4 } else { x=x+8 }; return x }';native=9},
 @{id='N02-partial-return-join';src='fn f(flag bool) int { var x=1; if flag { return 7 } else { x=3 }; return x+2 } fn main() int { return f(false)+f(true) }';native=12},
 @{id='N03-nested-return-arms';src='fn f(flag bool) int { if flag { if false { return 1 } else { return 2 } } else { if true { return 4 } else { return 8 } } } fn main() int { return f(true)+f(false) }';native=6},
 @{id='N04-continue-in-if';src='fn main() int { var n=0; var sum=0; while n<3 { n=n+1; if n<2 { continue } else { sum=sum+n } }; return sum }';native=5},
 @{id='N05-nested-loop-nearest-break';src='fn main() int { var i=0; var n=0; while i<3 { i=i+1; var j=0; while j<4 { j=j+1; if j<2 { continue } else { n=n+1; break } }; n=n+10 }; return n }';native=33},
 @{id='N06-scope-shadow-rhs';src='fn main() int { let x=2; if true { let x=x+3; if false { let x=x+10; return x } else { return x+1 } } else { return x } }';native=6},
 @{id='N07-branch-local-escape';src='fn main() int { if true { let x=1 } else { let x=2 }; return x }';code='source.undefined'},
 @{id='N08-duplicate-in-child';src='fn main() int { if true { let x=1; let x=2 }; return 0 }';code='source.duplicate'},
 @{id='N09-loop-scope-restored-break';src='fn main() int { loop { break }; break; return 0 }';code='verify.loop-target-scope'},
 @{id='N10-loop-body-returns';src='fn main() int { loop { return 7 } }';native=7},
 @{id='N11-dead-after-returning-loop';src='fn main() int { loop { return 7 }; let x=0; return x }';code='verify.block-structure'},
 @{id='N12-unexecuted-loop-trace';src='fn main() int { while false { let x=mark_a() }; if true { let y=mark_b() }; return 0 }';trace=$true;native=0;stderr='b'},
 @{id='N13-branch-trap-not-lifted';src='fn main() int { let a=mark_a(); if false { return 2147483647+1 } else { let b=mark_b() }; return 3 }';trace=$true;native=3;stderr='ab'},
 @{id='N14-stray-branches-target-first';src='fn main() int { if true { break } else { continue }; return 0 }';code='verify.loop-target-scope'},
 @{id='N15-loop-scope-restored-continue';src='fn main() int { while false { continue }; continue; return 0 }';code='verify.loop-target-scope'},
 @{id='N16-unexecuted-else-still-typed';src='fn main() int { if true { return 1 } else { return true } }';code='source.type-mismatch'},
 @{id='N17-if-depth-1000-native';src='fn main() int { '+('if true {'*1000)+'return 1'+('}'*1000)+'; return 0 }';native=1},
 @{id='N18-if-else-depth-1500-native';src='fn main() int { '+('if true {'*1500)+'return 1'+('}else{return 0}'*1500)+' }';native=1},
 @{id='N19-if-depth-20000-check';src='fn main() int { '+('if true {'*20000)+'return 1'+('}'*20000)+'; return 0 }';checkOnly=$true},
 @{id='N20-while-depth-2048-native';src='fn main() int { '+('while false {'*2048)+'return 1'+('}'*2048)+'; return 9 }';native=9},
 @{id='N21-unclosed-blocks-20000';src='fn main() int { '+('if true {'*20000);code='source.syntax'},
 @{id='N22-partial-else-tree-drop-20000';src='fn main() int { '+('if true {'*20000)+'return 1'+('}'*20000)+'else {';code='source.syntax'},
 @{id='N23-loop-if-while-state';src='fn main() int { var x=0; loop { if true { while false { return 3 }; x=7; break } else { return 2 } }; return x }';native=7},
 @{id='N24-branch-mutation-joins';src='fn main() int { var x=1; if false { x=3 } else { x=4 }; while x<7 { if true { x=x+1 } else { x=x+2 } }; return x }';native=7}
)
$rows=@()
foreach($c in $cases){
 $d=Join-Path $r ('extra-probes/'+$c.id);New-Item -ItemType Directory -Force $d|Out-Null;$path=Join-Path $d 'input.at';[IO.File]::WriteAllText($path,$c.src)
 $extra=@();if($c.trace){$extra=@('--capability','hir.test.trace.v1')}
 $check=Invoke-ReviewProcess $bin (@('check-source',$path,'--source-profile',$profile)+$extra) 30
 $build=$null;$run=$null;$hashes=@()
 if($c.ContainsKey('native') -and $check.exit -eq 0){if($c.trace){$extra+=@('--support-lib',(Join-Path $repo 'experimental/ac-core/test-support/target/debug/ac_trace_support.lib'))};$exe=Join-Path $d 'app.exe';$build=Invoke-ReviewProcess $bin (@('build-source',$path,'--source-profile',$profile,'--entry','main','--output',$exe)+$extra) 60;if($build.exit -eq 0){$run=Invoke-ReviewProcess $exe @() 10;foreach($n in @('app.exe','app.obj','app.ac-link.txt')){$hashes+=@{name=$n;sha256=(Get-FileHash (Join-Path $d $n)).Hash.ToLower()}}}}
 $expectedStderr=if($c.ContainsKey('stderr')){$c.stderr}else{''};$ok=if($c.code){$check.exit -eq 1 -and $check.stderr.Contains('/'+$c.code+']')}elseif($c.checkOnly){$check.exit -eq 0}else{$check.exit -eq 0 -and $build.exit -eq 0 -and $run.exit -eq $c.native -and $run.stderr -ceq $expectedStderr}
 $row=@{id=$c.id;bytes=$c.src.Length;check=$check;build=$build;run=$run;want_native=$c.native;want_code=$c.code;ok=$ok;artifact_hashes=$hashes;input_sha256=(Get-FileHash $path).Hash.ToLower()};$rows+=$row;$row|ConvertTo-Json -Depth 8|Set-Content (Join-Path $d 'result.json') -Encoding utf8
 Write-Output ($c.id+' ok='+$ok+' check='+$check.exit+' native='+$run.exit+' diag='+$check.stderr.Trim())
}
$rows|ConvertTo-Json -Depth 8|Set-Content (Join-Path $r 'block-results.json') -Encoding utf8
Write-Output ('block controls '+@($rows|Where-Object ok).Count+'/'+$rows.Count)