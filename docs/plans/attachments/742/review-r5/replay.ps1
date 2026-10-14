param([switch]$Build)
$ErrorActionPreference = 'Stop'
$evidence = $PSScriptRoot
$repo = 'D:/autostack/.wt/lang-742/auto-lang'
Set-Location -LiteralPath $repo
if ($Build) {
    cargo build -p auto-man --message-format=json > (Join-Path $evidence 'auto-man-build.jsonl') 2> (Join-Path $evidence 'auto-man-build.log')
    if ($LASTEXITCODE -ne 0) { throw 'build failed' }
}
$records = Get-Content (Join-Path $evidence 'auto-man-build.jsonl') | ForEach-Object { $_ | ConvertFrom-Json }
$files = @($records | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'auto_man' -and $_.profile.test -eq $false } | ForEach-Object { $_.filenames } | Where-Object { $_.EndsWith('.rlib') } | Select-Object -Unique)
if ($files.Count -ne 1) { throw 'ambiguous auto_man artifact' }
$scratch = 'D:/autostack/tmp/p742-review-r5'
New-Item -ItemType Directory -Force -Path $scratch | Out-Null
$rustArgs = @('--edition=2021',(Join-Path $evidence 'probe.rs'),'--extern',"auto_man=$($files[0])",'-L','dependency=target/debug/deps','-C','linker=rust-lld.exe','-C','link-arg=/STACK:33554432','-o',"$scratch/probe.exe")
foreach ($path in ($records | Where-Object { $_.reason -eq 'build-script-executed' } | ForEach-Object { $_.linked_paths } | Select-Object -Unique)) { $rustArgs += @('-L',$path) }
rustc @rustArgs *> (Join-Path $evidence 'probe-build.log')
if ($LASTEXITCODE -ne 0) { throw 'probe compilation failed' }
& "$scratch/probe.exe" "$scratch/projects" *> (Join-Path $evidence 'probe.log')
if ($LASTEXITCODE -ne 0) { throw 'probe execution failed' }
Get-Content (Join-Path $evidence 'probe.log')
