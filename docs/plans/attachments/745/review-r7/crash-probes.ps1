$ErrorActionPreference='Stop'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r7'
$helper=(Get-Content (Join-Path $r 'probes.ps1') -Raw).Split('$probes=@(')[0];Invoke-Expression $helper
$rows=@()
foreach($n in @(1500,2000,2500,3000,4096)){
 foreach($closed in @($true,$false)){
 $id='J'+$n+'-'+$closed;$dir=Join-Path $r ('extra-probes/'+$id);New-Item -ItemType Directory -Force $dir|Out-Null
 $src='fn main() int { return '+('1+('*$n)+'0'+$(if($closed){')'*$n}else{''})+' }';$path=Join-Path $dir 'input.at';[IO.File]::WriteAllText($path,$src)
 $sw=[Diagnostics.Stopwatch]::StartNew();$result=Invoke-ReviewProcess $bin @('check-source',$path,'--source-profile',$profile) 5
 $row=@{id=$id;bytes=$src.Length;balanced=$closed;seconds=$sw.Elapsed.TotalSeconds;result=$result};$rows+=$row;$row|ConvertTo-Json -Depth 6|Set-Content (Join-Path $dir 'result.json') -Encoding utf8
 Write-Output ($id+' exit='+$result.exit+' seconds='+$sw.Elapsed.TotalSeconds+' stderr='+$result.stderr.Trim())
 }
}
$rows|ConvertTo-Json -Depth 6|Set-Content (Join-Path $r 'crash-results.json') -Encoding utf8