param([string]$Tree = 'D:/autostack/.wt/lang-774/auto-lang')
$ErrorActionPreference = 'Stop'
$probe = Join-Path $Tree 'experimental/ac-core/probes/data-resources'
$dir = Join-Path $probe 'target/review-r1'
New-Item -ItemType Directory -Force -Path $dir | Out-Null
$source = (Join-Path $PSScriptRoot 'counterexamples.rs') -replace '\\', '/'
$manifest = @"
[package]
name = "review774r1"
version = "0.0.0"
edition = "2021"
[workspace]
[lib]
path = "$source"
[dependencies]
ac-data-probe-774 = { path = "../.." }
"@
[System.IO.File]::WriteAllText((Join-Path $dir 'Cargo.toml'), $manifest)
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'replay-Cargo.lock') -Destination (Join-Path $dir 'Cargo.lock') -Force
$ErrorActionPreference = 'Continue'
& cargo test --manifest-path (Join-Path $dir 'Cargo.toml') --offline --locked --target-dir (Join-Path $probe 'target') -- --test-threads=1
exit $LASTEXITCODE
