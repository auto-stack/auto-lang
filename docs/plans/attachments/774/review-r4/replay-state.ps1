param([string]$Tree = 'D:/autostack/.wt/lang-774/auto-lang')
$ErrorActionPreference = 'Stop'
$probeDir = Join-Path $Tree 'experimental/ac-core/probes/data-resources'
$sourcePath = Join-Path $probeDir 'shim/src/lib.rs'
$source = [System.IO.File]::ReadAllText($sourcePath)
# Adapt only crate-level std/no_std scaffolding, never the accounting code.
$source = $source.Replace('#![no_std]', '').Replace('#![no_builtins]', '')
$panicStart = $source.LastIndexOf('#[panic_handler]')
if ($panicStart -lt 0) { throw 'Original shim panic_handler marker not found' }
$source = $source.Substring(0, $panicStart)
$footer = [System.IO.File]::ReadAllText((Join-Path $PSScriptRoot 'counter-state-footer.rs'))
$generated = Join-Path $probeDir 'target/review-r4-state.rs'
$exe = Join-Path $probeDir 'target/review-r4-state.exe'
$header = "// Review-only std harness for exact current shim functions; seeded logical accounting state.`n#![allow(dead_code)]`n"
[System.IO.File]::WriteAllText($generated, $header + $source + $footer, [System.Text.UTF8Encoding]::new($false))
# These lints arise from putting a no_std freestanding shim under a std harness.
# The production cargo check gate keeps its own zero-warning check.
$ErrorActionPreference = 'Continue'
& rustc --edition=2021 --test -A unused_unsafe -A suspicious_runtime_symbol_definitions $generated -o $exe
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& $exe --test-threads=1
exit $LASTEXITCODE
