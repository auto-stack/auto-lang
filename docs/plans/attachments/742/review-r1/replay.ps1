param([switch]$Build)
$ErrorActionPreference = 'Stop'
$evidence = $PSScriptRoot
$repo = (Resolve-Path (Join-Path $evidence '../../../../..')).Path
Set-Location -LiteralPath $repo
if ($Build) {
    cargo build -p auto-man --message-format=json > (Join-Path $evidence 'auto-man-build.jsonl') 2> (Join-Path $evidence 'auto-man-build.log')
    if ($LASTEXITCODE -ne 0) { throw 'auto-man build failed' }
}
$records = Get-Content (Join-Path $evidence 'auto-man-build.jsonl') | ForEach-Object { $_ | ConvertFrom-Json }
$native = @($records | Where-Object { $_.reason -eq 'build-script-executed' } | ForEach-Object { $_.linked_paths } | Select-Object -Unique)
$scratch = 'D:/autostack/tmp/p742-review-r1'
New-Item -ItemType Directory -Force -Path $scratch | Out-Null
foreach ($name in @('probe','runtime-controls')) {
    $argsForRust = @('--edition=2021',(Join-Path $evidence "$name.rs"),'-L','dependency=target/debug/deps','-C','linker=rust-lld.exe','-C','link-arg=/STACK:33554432','-o',"$scratch/$name.exe")
    $deps = if ($name -eq 'probe') { @('auto_man') } else { @('rusqlite','serde_json','serde') }
    foreach ($dep in $deps) {
        $files = @($records | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq $dep -and $_.profile.test -eq $false } | ForEach-Object { $_.filenames } | Where-Object { $_.EndsWith('.rlib') } | Select-Object -Unique)
        if ($files.Count -ne 1) { throw "Artifact identity ambiguous for $dep" }
        $argsForRust += @('--extern',"$dep=$($files[0])")
    }
    foreach ($path in $native) { $argsForRust += @('-L',$path) }
    rustc @argsForRust *> (Join-Path $evidence "$name-build.log")
    if ($LASTEXITCODE -ne 0) { throw "$name compilation failed" }
    $output = if ($name -eq 'probe') { "$scratch/projects" } else { "$scratch/runtime-fs" }
    & "$scratch/$name.exe" $output *> (Join-Path $evidence "$name.log")
    if ($LASTEXITCODE -ne 0) { throw "$name execution failed" }
}
& (Join-Path $evidence 'execute-generated.ps1')
Get-Content (Join-Path $evidence 'runtime-controls.log')
# This script records observations. generated-results.json must be compared
# with the expected literal values in REVIEW.md; exit 0 is not an ALL-PASS gate.
