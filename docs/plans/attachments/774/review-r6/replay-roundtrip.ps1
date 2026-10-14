param([string]$Tree = 'D:/autostack/.wt/lang-774/auto-lang')
$ErrorActionPreference = 'Stop'
$probe = Join-Path $Tree 'experimental/ac-core/probes/data-resources'
$dir = Join-Path $probe 'target/review-r6'
New-Item -ItemType Directory -Force -Path $dir | Out-Null
$source = [IO.File]::ReadAllText((Join-Path $probe 'shim/src/lib.rs'))
$source = $source.Replace('#![no_std]', '').Replace('#![no_builtins]', '')
$panicStart = $source.LastIndexOf('#[panic_handler]')
if ($panicStart -lt 0) { throw 'shim panic_handler marker not found' }
$source = $source.Substring(0, $panicStart)
$footer = [IO.File]::ReadAllText((Join-Path $PSScriptRoot 'summary-emitter-footer.rs'))
$generated = Join-Path $dir 'summary-emitter.rs'
$exe = Join-Path $dir 'summary-emitter.exe'
[IO.File]::WriteAllText($generated, "#![allow(dead_code)]`n" + $source + $footer, [Text.UTF8Encoding]::new($false))
$ErrorActionPreference = 'Continue'
& rustc --edition=2021 -A unused_unsafe -A suspicious_runtime_symbol_definitions $generated -o $exe
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$ErrorActionPreference = 'Stop'
foreach ($case in @('edge', 'wide', 'released')) {
    $stderrPath = Join-Path $PSScriptRoot "$case.stderr.txt"
    $stdoutPath = Join-Path $PSScriptRoot "$case.stdout.txt"
    $info = New-Object System.Diagnostics.ProcessStartInfo
    $info.FileName = $exe
    $info.Arguments = $case
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardError = $true
    $info.RedirectStandardOutput = $true
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo = $info
    [void]$process.Start()
    $stderrTask = $process.StandardError.ReadToEndAsync()
    $stdoutTask = $process.StandardOutput.ReadToEndAsync()
    if (!$process.WaitForExit(5000)) { $process.Kill(); throw "formatter deadline: $case" }
    [IO.File]::WriteAllText($stderrPath, $stderrTask.GetAwaiter().GetResult(), [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText($stdoutPath, $stdoutTask.GetAwaiter().GetResult(), [Text.UTF8Encoding]::new($false))
    if ($process.ExitCode -ne 0) { throw "formatter failed: $case exit=$($process.ExitCode)" }
}
Copy-Item -LiteralPath $generated -Destination (Join-Path $PSScriptRoot 'summary-emitter.frozen.rs') -Force
$testSource = (Join-Path $PSScriptRoot 'summary-roundtrip.rs') -replace '\\', '/'
$manifest = @"
[package]
name = "review774r6"
version = "0.0.0"
edition = "2021"
[workspace]
[lib]
path = "$testSource"
[dependencies]
ac-data-probe-774 = { path = "../.." }
"@
[IO.File]::WriteAllText((Join-Path $dir 'Cargo.toml'), $manifest)
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'replay-Cargo.lock') -Destination (Join-Path $dir 'Cargo.lock') -Force
$ErrorActionPreference = 'Continue'
& cargo test --manifest-path (Join-Path $dir 'Cargo.toml') --offline --locked --target-dir (Join-Path $probe 'target')
exit $LASTEXITCODE
