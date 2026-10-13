$ErrorActionPreference='Stop'
$reviewDir='D:/autostack/auto-lang/docs/plans/attachments/745/review-r6'
$helper=(Get-Content (Join-Path $reviewDir 'probes.ps1') -Raw).Split('$probes=@(')[0]
Invoke-Expression $helper
$results=@()
$cases=@(
 @{id='F01-flat-add-4096-valid';source='fn main() int { return '+(('1+'*4096)+'0')+' }'},
 @{id='F02-flat-add-4096-malformed';source='fn main() int { return '+('1+'*4096)+' }'},
 @{id='F03-valid-parens-100';source='fn main() int { return '+('('*100)+'0'+(')'*100)+' }'},
 @{id='F04-valid-parens-99';source='fn main() int { return '+('('*99)+'0'+(')'*99)+' }'},
 @{id='F05-deep-unclosed-blocks';source='fn main() int { '+('if true {'*4096)},
 @{id='F06-valid-flat-add-512';source='fn main() int { return '+('1+'*512)+'0 }'},
 @{id='F07-flat-add-16384-malformed';source='fn main() int { return '+('1+'*16384)+' }'},
 @{id='F08-flat-add-16384-valid';source='fn main() int { return '+('1+'*16384)+'0 }'},
 @{id='F09-flat-add-16384-wrong-type';source='fn main() int { return '+('1+'*16384)+'true }'}
)
foreach($c in $cases){
 $d=Join-Path $reviewDir ('extra-probes/'+$c.id);New-Item -ItemType Directory -Force $d|Out-Null
 $path=Join-Path $d 'input.at';[IO.File]::WriteAllText($path,$c.source)
 $run=Invoke-ReviewProcess $bin @('check-source',$path,'--source-profile',$profile) 10
 $row=@{id=$c.id;bytes=$c.source.Length;result=$run};$results+=$row
 $row|ConvertTo-Json -Depth 6|Set-Content (Join-Path $d 'result.json')
 Write-Output ($c.id+' exit='+$run.exit+' diag='+$run.stderr.Trim())
}
$results|ConvertTo-Json -Depth 6|Set-Content (Join-Path $reviewDir 'new-results.json')
