$ErrorActionPreference='Stop'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r6'
foreach($name in @('probes','extra-probes','depth-probes','new-probes','boundary-probes','novel-probes','resource-probes','crash-probes','corpus-replay','supplemental','bundle-probe','helper-probe','csharp-controls','parser-probes')){
 & pwsh -NoProfile -File (Join-Path $r ($name+'.ps1')) *> (Join-Path $r ($name+'.log.txt'))
 if($LASTEXITCODE -ne 0){throw ($name+' failed: '+$LASTEXITCODE)}
 Write-Output ($name+' completed')
}