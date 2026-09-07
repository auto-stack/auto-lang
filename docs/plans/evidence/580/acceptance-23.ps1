# Plan 580 acceptance 2/3 probes:
#   install-arm: PATH has cargo (temp copy) but NO ninja/n2 -> install arm runs,
#                recheck finds ~/.cargo/bin/n2.exe
#   all-none:    PATH has neither ninja/n2 nor cargo -> Err guidance, no panic
param([Parameter(Mandatory=$true)][ValidateSet('install-arm','all-none')][string]$Mode)

# strip both ninja's dir and cargo's dir (which also holds n2)
$stripped = ($env:PATH -split ';' | Where-Object {
    $_ -and $_.TrimEnd('\') -ne 'D:\soft\bin' -and $_.TrimEnd('\') -ne 'C:\Users\zhaop\.cargo\bin'
}) -join ';'

if ($Mode -eq 'install-arm') {
    $shim = 'D:\autostack\.wt\lang-580\_probe\cargo-shim'
    New-Item -ItemType Directory -Force -Path $shim | Out-Null
    if (-not (Test-Path "$shim\cargo.exe")) {
        Copy-Item 'C:\Users\zhaop\.cargo\bin\cargo.exe' "$shim\cargo.exe"
    }
    $env:PATH = "$shim;$stripped"
} else {
    $env:PATH = $stripped
}

Write-Output ("ninja: " + ((where.exe ninja 2>$null | Select-Object -First 1) -join ''))
Write-Output ("n2:    " + ((where.exe n2 2>$null | Select-Object -First 1) -join ''))
Write-Output ("cargo: " + ((where.exe cargo 2>$null | Select-Object -First 1) -join ''))

# invoke the compiled lib-test binary directly (the stripped PATH may lack cargo)
& 'D:\autostack\.wt\lang-580\auto-lang\target\debug\deps\auto_man-a46d0e1bde44be84.exe' --ignored probe_resolve_logs_host --nocapture
Write-Output "PROBE_EXIT=$LASTEXITCODE"
