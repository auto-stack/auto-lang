param([string]$InputName='review-r5-counterexamples')
$ErrorActionPreference='Stop'
$tree='D:/autostack/.wt/lang-783/auto-lang'
$target="$tree/experimental/ac-artifact-probe/target/release"
$deps="$target/deps"
$rlib=(Get-ChildItem $deps -Filter 'libac_artifact_probe-*.rlib' | Select-Object -First 1).FullName
$nativeDirs=@(Get-ChildItem "$target/build" -Directory | ForEach-Object {Join-Path $_.FullName 'out'} | Where-Object {Test-Path $_})
$winlib=(Get-ChildItem C:/Users/zhaop/.cargo/registry/src -Filter windows.0.52.0.lib -Recurse | Where-Object {$_.FullName -match 'windows_x86_64_msvc'} | Select-Object -First 1).DirectoryName
$nativeArgs=@();foreach($dir in ($nativeDirs+@($winlib))){$nativeArgs+='-L';$nativeArgs+="native=$dir"}
$exe=Join-Path $env:TEMP ("ac783-r5-"+[guid]::NewGuid().ToString('N')+'.exe')
& rustc --edition=2021 "$PSScriptRoot/$InputName.rs" --extern "ac_artifact_probe=$rlib" -L "dependency=$deps" @nativeArgs -o $exe
if($LASTEXITCODE -ne 0){throw 'rustc failed'}
$scratch=Join-Path $env:TEMP ("ac783-r5-"+[guid]::NewGuid().ToString('N'))
& $exe $scratch
if($LASTEXITCODE -ne 0){throw 'counterexample failed to execute'}
