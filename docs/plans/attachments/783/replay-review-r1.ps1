param([string]$Tree = 'D:/autostack/.wt/lang-783/auto-lang')
$ErrorActionPreference = 'Stop'
# Read/build only the committed probe; compile the external review consumer.
$target = Join-Path $Tree 'experimental/ac-artifact-probe/target/release'
$deps = Join-Path $target 'deps'
$rlib = (Get-ChildItem $deps -Filter 'libac_artifact_probe-*.rlib' | Select-Object -First 1).FullName
if (-not $rlib) { throw 'Build the frozen probe release library first.' }
$nativeDirs = @(Get-ChildItem "$target/build" -Directory | ForEach-Object {
    Join-Path $_.FullName 'out'
} | Where-Object { Test-Path $_ })
$winlib = (Get-ChildItem "$env:USERPROFILE/.cargo/registry/src" -Filter windows.0.52.0.lib -Recurse |
    Where-Object { $_.FullName -match 'windows_x86_64_msvc' } | Select-Object -First 1).DirectoryName
$nativeArgs = @()
foreach ($dir in ($nativeDirs + @($winlib))) { $nativeArgs += '-L'; $nativeArgs += "native=$dir" }
$exe = Join-Path $env:TEMP ('ac783-review-' + [guid]::NewGuid().ToString('N') + '.exe')
& rustc --edition=2021 "$PSScriptRoot/review-counterexamples.rs" --extern "ac_artifact_probe=$rlib" -L "dependency=$deps" @nativeArgs -o $exe
if ($LASTEXITCODE -ne 0) { throw 'Review consumer compilation failed.' }
& $exe (Join-Path $env:TEMP ('ac783-review-' + [guid]::NewGuid().ToString('N')))
if ($LASTEXITCODE -ne 0) { throw 'Review consumer failed before completing controls.' }
# Expected R1 baseline output is recorded in review-r1-counterexamples.txt.
# After repair, convert each reported bad behavior into its positive oracle.
