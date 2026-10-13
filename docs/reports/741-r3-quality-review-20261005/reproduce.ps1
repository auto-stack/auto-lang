param(
 [Parameter(Mandatory=$true)][string]$RepoRoot,
 [switch]$ProductionDeadline
)
$ErrorActionPreference='Stop'
$repo=(Resolve-Path -LiteralPath $RepoRoot).Path
$source=$PSScriptRoot
$out=Join-Path ([IO.Path]::GetTempPath()) ('p741-r3-review-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $out | Out-Null
$manifest=Join-Path $repo 'experimental/ac-core/Cargo.toml'
$messages=@(& cargo build --manifest-path $manifest --locked --message-format=json)
if($LASTEXITCODE -ne 0){throw 'prototype build failed'}
$artifacts=@($messages | ForEach-Object {$_ | ConvertFrom-Json} | Where-Object {$_.reason -eq 'compiler-artifact'})
$lib=@($artifacts | Where-Object {$_.target.name -eq 'auto_ac_prototype'} | ForEach-Object {$_.filenames} | Where-Object {$_.EndsWith('.rlib')} | Select-Object -Unique)
$windows=@($artifacts | Where-Object {$_.target.name -eq 'windows_sys'} | ForEach-Object {$_.filenames} | Where-Object {$_.EndsWith('.rlib')} | Select-Object -Unique)
if($lib.Count -ne 1 -or $windows.Count -ne 1){throw 'expected one current prototype and windows-sys library'}
$deps=Join-Path $repo 'experimental/ac-core/target/debug/deps'
$native=Get-ChildItem (Join-Path $env:USERPROFILE '.cargo/registry/src') -Filter windows.0.52.0.lib -Recurse -File | Select-Object -First 1
if($null -eq $native){throw 'windows native import library unavailable'}
$env:LIB=$native.DirectoryName+';'+$env:LIB
$linkSource=Get-Content (Join-Path $repo 'experimental/ac-core/src/link.rs') -Raw
[IO.File]::WriteAllText((Join-Path $out 'exact-link.rs'),($linkSource -replace '(?m)^//!','//'),[Text.UTF8Encoding]::new($false))
Copy-Item -LiteralPath (Join-Path $source 'boundary-helper.rs') -Destination $out
$helper=Join-Path $out 'boundary-helper.exe'
& rustc --edition=2021 (Join-Path $out 'boundary-helper.rs') -L "dependency=$deps" --extern "auto_ac_prototype=$($lib[0])" --extern "windows_sys=$($windows[0])" -o $helper
if($LASTEXITCODE -ne 0){throw 'review helper compilation failed'}
$original=Join-Path $out 'original-helper.exe'
& rustc --edition=2021 (Join-Path $repo 'docs/reports/741-quality-review-20261005/review-helper.rs') -L "dependency=$deps" --extern "auto_ac_prototype=$($lib[0])" -o $original
if($LASTEXITCODE -ne 0){throw 'original helper compilation failed'}
function Invoke-Review([string]$Exe,[string[]]$ProbeArguments,[string]$Name,[int]$Seconds){
 $stdout=Join-Path $out ($Name+'.stdout.txt');$stderr=Join-Path $out ($Name+'.stderr.txt')
 $p=Start-Process -FilePath $Exe -ArgumentList ($ProbeArguments|ForEach-Object {'"'+$_+'"'}) -PassThru -WindowStyle Hidden -RedirectStandardOutput $stdout -RedirectStandardError $stderr
 $done=$false
 try{$done=$p.WaitForExit($Seconds*1000)}
 finally{
  if(-not $p.HasExited){$live=Get-Process -Id $p.Id -ErrorAction SilentlyContinue;if($null -ne $live -and $live.Path -eq $Exe){Stop-Process -Id $live.Id -Force}}
  $pidfile=Join-Path $out 'fallback-child.pid'
  if(Test-Path -LiteralPath $pidfile -PathType Leaf){$childId=[int](Get-Content $pidfile -Raw);$child=Get-Process -Id $childId -ErrorAction SilentlyContinue;if($null -ne $child -and $child.Path -in @($helper,$original)){Stop-Process -Id $child.Id -Force}}
 }
 if($done -and $p.ExitCode -ne 0){throw "review probe $Name exited $($p.ExitCode): $(Get-Content $stderr -Raw)"}
 [pscustomobject]@{name=$Name;exit=$p.ExitCode;completedBeforeOuterDeadline=$done;stdout=(Get-Content $stdout -Raw);stderr=(Get-Content $stderr -Raw)} | ConvertTo-Json -Depth 4
}
$cli=Join-Path $repo 'experimental/ac-core/target/debug/auto-ac-prototype.exe'
Invoke-Review $cli @('build',(Join-Path $repo 'experimental/ac-core/fixtures/native/add-2-3.atom'),'--entry','d_entry','--output',(Join-Path $out 'old.exe')) 'baseline' 20
Invoke-Review $helper @('locked-cleanup',(Join-Path $out 'locked'),(Join-Path $out 'old.exe'),(Join-Path $out 'old.obj')) 'locked-cleanup' 10
# Scheduling dependent: a successful reclamation does not rule out the startup gap.
for($i=0;$i -lt 3;$i++){Invoke-Review $helper @('fallback',$out) "process-$i" 5}
if($ProductionDeadline){Invoke-Review $helper @('public-fallback',$out,$original) 'public-60s' 70}
Write-Output "Evidence directory: $out"
