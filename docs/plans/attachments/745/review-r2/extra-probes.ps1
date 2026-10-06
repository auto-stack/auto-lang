$ErrorActionPreference='Stop'
$reviewRoot='D:/autostack/auto-lang/docs/plans/attachments/745/review-r2'
$helper=(Get-Content (Join-Path $reviewRoot 'probes.ps1') -Raw).Split('$probes=@(')[0]
Invoke-Expression $helper
$cases=@(
 @{id='D01-call-multiline-then-add';source="fn f(x int) int { return x }`nfn main() int { return f(`n 2`n) + 3 }";want='accept native5'},
 @{id='D02-parenthesized-multiline-then-add';source="fn main() int { return (1 +`n 2) + 3 }";want='accept native6'},
 @{id='D03-negative-multiline-arg';source="fn f(x int) int { return x }`nfn main() int { return f(-`n 2) + 3 }";want='accept native1'},
 @{id='D04-parenthesized-multiline-in-outer-parens';source="fn main() int { return ((1 +`n 2) + 3) }";want='accept native6'},
 @{id='D05-call-multiline-no-tail';source="fn f(x int) int { return x }`nfn main() int { return f(`n 2`n) }";want='accept native2'},
 @{id='D06-unbracketed-negative-newline';source="fn main() int { return -`n2 }";want='reject'},
 @{id='D07-colliding-outer-prefix-name';source='fn ac_start() int { return 3 } fn sf_ac_start() int { return 4 } fn main() int { return ac_start() + sf_ac_start() }';want='accept native7'},
 @{id='D08-both-diverge-trailing';source='fn main() int { loop { if true { break } else { continue }; let x = 2 }; return 0 }';want='reject at let'},
 @{id='D09-mixed-return-break-trailing';source='fn main() int { loop { if true { return 1 } else { break }; let x = 2 }; return 0 }';want='conservative verify behavior inspect'},
 @{id='D10-stray-in-both-branches';source='fn main() int { if true { break } else { continue }; return 0 }';want='reject loop-target-scope'},
 @{id='D11-escaped-char';source="fn main() int { let x = '\''; return 0 }";want='source.unsupported'},
 @{id='D12-keyword-named-arg';source='fn f(x int) int { return x } fn main() int { return f(type: 2) }';want='source.unsupported'},
 @{id='D13-stray-branches-without-tail';source='fn main() int { if true { break } else { continue } }';want='reject loop-target-scope control'},
 @{id='D14-dead-statement-after-valid-break';source='fn main() int { loop { break; let x = 2 }; return 0 }';want='reject at let'}
)
$results=@()
foreach($case in $cases){
 $dir=Join-Path $reviewRoot ('extra-probes/'+$case.id);New-Item -ItemType Directory -Force -Path $dir|Out-Null
 $path=Join-Path $dir 'input.at';[IO.File]::WriteAllText($path,$case.source,[Text.UTF8Encoding]::new($false))
 $checked=Invoke-ReviewProcess $bin @('check-source',$path,'--source-profile',$profile)
 $built=$null;$run=$null
 if($case.want.StartsWith('accept') -and $checked.exit -eq 0){$exe=Join-Path $dir 'app.exe';$built=Invoke-ReviewProcess $bin @('build-source',$path,'--source-profile',$profile,'--entry','main','--output',$exe);if($built.exit -eq 0){$run=Invoke-ReviewProcess $exe @()}}
 $result=[ordered]@{id=$case.id;source=$case.source;want=$case.want;check=$checked;build=$built;run=$run};$results+=$result
 $result|ConvertTo-Json -Depth 7|Set-Content (Join-Path $dir 'result.json') -Encoding utf8
 Write-Output ($case.id+' check='+$checked.exit+' diag='+$checked.stderr.Trim()+' build='+$built.exit+' run='+$run.exit)
}
$results|ConvertTo-Json -Depth 8|Set-Content (Join-Path $reviewRoot 'extra-results.json') -Encoding utf8
