$ErrorActionPreference='Stop'
$repo='D:/autostack/.wt/lang-745/auto-lang'
$receipt='D:/autostack/auto-lang/docs/plans/attachments/745/review-r1'
$bin=Join-Path $repo 'experimental/ac-core/target/debug/auto-ac-prototype.exe'
$profile='auto.source.core-i32.draft'
function Invoke-ReviewProcess([string]$program,[string[]]$arguments,[int]$timeoutSec=60) {
 $psi=[System.Diagnostics.ProcessStartInfo]::new(); $psi.FileName=$program; $psi.WorkingDirectory=$repo; $psi.UseShellExecute=$false; $psi.RedirectStandardOutput=$true; $psi.RedirectStandardError=$true
 foreach($arg in $arguments){$psi.ArgumentList.Add($arg)}
 $p=[System.Diagnostics.Process]::Start($psi); $o=$p.StandardOutput.ReadToEndAsync(); $e=$p.StandardError.ReadToEndAsync()
 if(-not $p.WaitForExit($timeoutSec*1000)){$p.Kill(); $p.WaitForExit(); return @{exit='TIMEOUT';stdout='';stderr='review hard deadline'}}
 if(-not $o.Wait(10000) -or -not $e.Wait(10000)){return @{exit='COLLECT_TIMEOUT';stdout='';stderr='review output deadline'}}
 return @{exit=$p.ExitCode;stdout=$o.Result;stderr=$e.Result}
}
$probes=@(
 @{id='C01-unbracketed-rhs-newline';source="fn main() int {`n return 1 +`n 2`n}";want='reject'},
 @{id='C02-parenthesized-newline-control';source="fn main() int {`n return (1 +`n 2)`n}";want='accept'},
 @{id='C03-call-name-newline';source="fn f(x int) int { return x }`nfn main() int { return f`n(3) }";want='reject'},
 @{id='C04-escaped-string';source='fn main() int { let x = "a\"b"; return 0 }';want='source.unsupported'},
 @{id='C05-ac-start-user-function';source='fn ac_start() int { return 3 } fn main() int { return ac_start() }';want='native3';build=$true},
 @{id='C06-exitprocess-user-function';source='fn ExitProcess() int { return 4 } fn main() int { return ExitProcess() }';want='native4';build=$true},
 @{id='C07-nested-continue';source='fn main() int { var i = 0; var n = 0; while i < 2 { i = i + 1; var j = 0; while j < 2 { j = j + 1; n = n + 1; continue }; n = n + 2 }; return n }';want='native8';build=$true},
 @{id='C08-trace-binary-order';source='fn main() int { return mark_b() * 10 + mark_a() }';want='native21/ba';build=$true;trace=$true},
 @{id='C09-both-branches-return-trailing';source='fn main() int { if true { return 0 } else { return 1 }; let x = 2; return x }';want='reject'},
 @{id='C10-scope-escape-token';source="fn main() int {`n if true { let x = 1 }`n return x`n}";want='reject, token x col9 line3'},
 @{id='C11-immutable-write-token';source="fn main() int {`n let x = 1`n x = 2`n return x`n}";want='reject, token x line3 col2'},
 @{id='C12-keyword-full-language';source='fn type() int { return 1 } fn main() int { return type() }';want='keyword rejection'},
 @{id='C13-param-root-shadow';source='fn f(x int) int { let x = 3; return x } fn main() int { return f(1) }';want='inspect-scope'},
 @{id='C14-bool-to-int-local';source='fn main() int { let x int = true; return x }';want='source.type-mismatch'},
 @{id='C15-unsupported-logical-not';source='fn main() int { if !true { return 0 } else { return 1 } }';want='source.unsupported'},
 @{id='C16-unsupported-or';source='fn main() int { if true || false { return 0 } else { return 1 } }';want='source.unsupported'},
 @{id='C17-unsupported-u32';source='fn main() u32 { return 0 }';want='source.unsupported'},
 @{id='C18-zero-loop-local-escape';source='fn main() int { while false { let x = 1 }; return x }';want='reject'},
 @{id='C19-parenthesized-unary-minus';source='fn main() int { return -(1) }';want='source.unsupported'},
 @{id='C20-extra-positional-arg';source='fn f(x int) int { return x } fn main() int { return f(1,2) }';want='source.argument'},
 @{id='C21-bare-return-next-line';source="fn main() int {`nreturn`n2`n}";want='inspect-newline'},
 @{id='C22-signed-negative-overflow';source='fn main() int { return -2147483648 + -1 }';want='native70';build=$true}
)
$results=@()
foreach($probe in $probes){
 $dir=Join-Path $receipt ('probes/'+$probe.id); New-Item -ItemType Directory -Force -Path $dir|Out-Null
 $path=Join-Path $dir 'input.at'; [IO.File]::WriteAllText($path,$probe.source,[Text.UTF8Encoding]::new($false))
 $argv=@('check-source',$path,'--source-profile',$profile); if($probe.trace){$argv+=@('--capability','hir.test.trace.v1')}
 $checked=Invoke-ReviewProcess $bin $argv; $built=$null; $run=$null
 if($probe.build){$exe=Join-Path $dir 'app.exe'; $argv=@('build-source',$path,'--source-profile',$profile,'--entry','main','--output',$exe); if($probe.trace){$argv+=@('--capability','hir.test.trace.v1','--support-lib',(Join-Path $repo 'experimental/ac-core/test-support/target/debug/ac_trace_support.lib'))}; $built=Invoke-ReviewProcess $bin $argv; if($built.exit -eq 0){$run=Invoke-ReviewProcess $exe @()}}
 $result=[ordered]@{id=$probe.id;source=$probe.source;want=$probe.want;check=$checked;build=$built;run=$run}
 $result|ConvertTo-Json -Depth 7|Set-Content (Join-Path $dir 'result.json') -Encoding utf8
 $results+=$result; Write-Output ($probe.id+' check='+$checked.exit+' diag='+$checked.stderr.Trim()+' build='+$built.exit+' run='+$run.exit+' trace='+$run.stderr)
}
$results|ConvertTo-Json -Depth 8|Set-Content (Join-Path $receipt 'counterexamples.json') -Encoding utf8
