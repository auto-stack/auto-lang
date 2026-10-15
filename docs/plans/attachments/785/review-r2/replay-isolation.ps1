# Run in D:/autostack/.wt/lang-785/auto-lang after building the exact baseline:
# cargo test -p auto-lang --lib --features test-cases clipboard_provider -- --test-threads=8
# The executable below is the artifact reported by that Cargo invocation on R2.
param([string]$TestBinary = 'target/debug/deps/auto_lang-dc6ba392f7d81a4f.exe')
$ErrorActionPreference = 'Stop'
for ($reviewRun = 1; $reviewRun -le 30; $reviewRun++) {
    $reviewOut = & $TestBinary clipboard_provider frame_provider host_capabilities plan320_tests --test-threads=16 2>&1
    $reviewExit = $LASTEXITCODE
    Write-Output "RUN=$reviewRun EXIT=$reviewExit"
    Write-Output $reviewOut
    if ($reviewExit -ne 0) { exit $reviewExit }
}
exit 0
