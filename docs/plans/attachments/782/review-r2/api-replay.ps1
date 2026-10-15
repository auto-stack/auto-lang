$ErrorActionPreference = 'Stop'
Set-Location 'D:/autostack/.wt/lang-782/auto-lang'
$taskLib = Get-ChildItem experimental/ac-core/target/debug/deps/libauto_ac_prototype*.rlib |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1
$nativeLib = Get-ChildItem "$env:USERPROFILE/.cargo/registry/src/*/windows_x86_64_msvc-0.52.6/lib/windows.0.52.0.lib" |
    Select-Object -First 1
rustc --edition 2021 "$PSScriptRoot/api_probe.rs" --extern "auto_ac_prototype=$($taskLib.FullName)" `
    -L dependency=experimental/ac-core/target/debug/deps "-Lnative=$($nativeLib.DirectoryName)" `
    -o experimental/ac-core/target/review-782-r1/api_probe.exe
if ($LASTEXITCODE -ne 0) { throw 'probe compile failed' }
& experimental/ac-core/target/review-782-r1/api_probe.exe 2>&1 |
    Tee-Object -FilePath "$PSScriptRoot/api-results.txt"
