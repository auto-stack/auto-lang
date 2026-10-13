$ErrorActionPreference='Stop'
$w='D:/autostack/.wt/lang-745/auto-lang'
$r='D:/autostack/auto-lang/docs/plans/attachments/745/review-r3'
$full=[IO.File]::ReadAllText((Join-Path $w 'scripts/verify-ac-source-745.ps1'))
$start=$full.IndexOf('function Invoke-Captured {');$end=$full.IndexOf('$Profile =')
Invoke-Expression $full.Substring($start,$end-$start)
function Check-Hash([string]$tag){
 try {Write-Output ($tag+' hash='+ (Get-Sha256 (Join-Path $w 'AGENTS.md')))}catch{Write-Output ($tag+' FAIL: '+$_.Exception.Message)}
}
Check-Hash 'before-executor'
for($i=0;$i -lt 3;$i++){
 $run=Invoke-Captured 'cargo' @('check','--locked','--manifest-path','experimental/ac-core/Cargo.toml') $w 60
 Write-Output ('cargo-'+$i+' code='+$run.Code)
 Check-Hash ('after-executor-'+$i)
}
Get-Module Microsoft.PowerShell.Utility | Select-Object Name,Path
Get-Command Get-FileHash -ErrorAction Continue | Select-Object Name,ModuleName
