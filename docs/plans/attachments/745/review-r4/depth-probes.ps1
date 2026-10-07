$ErrorActionPreference='Stop'
$reviewDir='D:/autostack/auto-lang/docs/plans/attachments/745/review-r4'
$helper=(Get-Content (Join-Path $reviewDir 'probes.ps1') -Raw).Split('$probes=@(')[0]
Invoke-Expression $helper
$results=@()
foreach($case in @(@{id='E01-shallow-parenthesis-control';depth=16;close=$true;want='accept'},@{id='E02-deep-unclosed-parenthesis';depth=4096;close=$false;want='controlled source.syntax exit1'})){
 $dir=Join-Path $reviewDir ('extra-probes/'+$case.id)
 New-Item -ItemType Directory -Force -Path $dir|Out-Null
 $src='fn main() int { return '+('('*$case.depth)+'0'+$(if($case.close){')'*$case.depth}else{''})+' }'
 $path=Join-Path $dir 'input.at';[IO.File]::WriteAllText($path,$src)
 $run=Invoke-ReviewProcess $bin @('check-source',$path,'--source-profile',$profile) 10
 $row=[ordered]@{id=$case.id;depth=$case.depth;source_bytes=$src.Length;want=$case.want;result=$run}
 $row|ConvertTo-Json -Depth 5|Set-Content (Join-Path $dir 'result.json') -Encoding utf8
 $results+=$row
 Write-Output ($case.id+' exit='+$run.exit+' stderr='+$run.stderr.Trim())
}
$results|ConvertTo-Json -Depth 5|Set-Content (Join-Path $reviewDir 'depth-results.json') -Encoding utf8
