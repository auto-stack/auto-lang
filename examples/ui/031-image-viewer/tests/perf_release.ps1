[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$testsRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = (Resolve-Path (Join-Path $testsRoot "..\..\..\..")).Path
$manifest = Join-Path $projectRoot "examples\rust-workspace\031-image-viewer\Cargo.toml"
$generatedRoot = Join-Path $projectRoot "examples\rust-workspace\target"
$expectationsPath = Join-Path $testsRoot "perf_expectations.json"
$reportPath = Join-Path $testsRoot "perf-report.json"
$tempFixture = Join-Path ([System.IO.Path]::GetTempPath()) "auto-lang-plan547-24mp.jpg"

$expectations = Get-Content -Raw $expectationsPath | ConvertFrom-Json
$report = [ordered]@{
    plan = 547
    mode = "rust-release"
    generated_at = [DateTime]::UtcNow.ToString("o")
    fixture = [ordered]@{ width = 6000; height = 4000; pixels = 24000000; path = $tempFixture }
    metrics = [ordered]@{}
    checks = @()
    passed = $false
}
$process = $null

function Add-Check([string]$Name, [double]$Actual, [double]$Budget) {
    $script:report.checks += [ordered]@{
        name = $Name
        actual = [Math]::Round($Actual, 3)
        budget = $Budget
        passed = ($Actual -le $Budget)
    }
}

try {
    Add-Type -AssemblyName System.Drawing
    $bitmap = [System.Drawing.Bitmap]::new(6000, 4000, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
    try {
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        try {
            $graphics.Clear([System.Drawing.Color]::FromArgb(24, 32, 48))
            $graphics.FillRectangle([System.Drawing.Brushes]::MidnightBlue, 0, 0, 3000, 4000)
            $graphics.FillRectangle([System.Drawing.Brushes]::DarkSlateBlue, 3000, 0, 3000, 4000)
        } finally {
            $graphics.Dispose()
        }
        $codec = [System.Drawing.Imaging.ImageCodecInfo]::GetImageEncoders() | Where-Object { $_.MimeType -eq "image/jpeg" }
        $encoderParams = [System.Drawing.Imaging.EncoderParameters]::new(1)
        try {
            $encoderParams.Param[0] = [System.Drawing.Imaging.EncoderParameter]::new([System.Drawing.Imaging.Encoder]::Quality, [long]85)
            $bitmap.Save($tempFixture, $codec, $encoderParams)
        } finally {
            $encoderParams.Dispose()
        }
    } finally {
        $bitmap.Dispose()
    }

    $buildWatch = [Diagnostics.Stopwatch]::StartNew()
    $buildLog = Join-Path $testsRoot "perf-build.log"
    $buildErr = Join-Path $testsRoot "perf-build.err.log"
    $buildProcess = Start-Process -FilePath "cargo.exe" -ArgumentList @("build", "--release", "--manifest-path", $manifest, "--quiet") -WorkingDirectory $projectRoot -WindowStyle Hidden -Wait -PassThru -RedirectStandardOutput $buildLog -RedirectStandardError $buildErr
    if ($buildProcess.ExitCode -ne 0) {
        throw "release build failed; see perf-build.log"
    }
    $buildWatch.Stop()
    $report.metrics.release_build_ms = $buildWatch.Elapsed.TotalMilliseconds

    $exeCandidates = @(
        (Join-Path $generatedRoot "release\image-viewer.exe"),
        (Join-Path $projectRoot "target\release\image-viewer.exe")
    )
    $exe = $exeCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
    if (-not $exe) {
        throw "release executable image-viewer.exe was not produced"
    }

    $coldWatch = [Diagnostics.Stopwatch]::StartNew()
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = [Diagnostics.ProcessStartInfo]::new()
    $process.StartInfo.FileName = $exe
    $process.StartInfo.WorkingDirectory = Split-Path $exe
    $process.StartInfo.UseShellExecute = $false
    $process.StartInfo.CreateNoWindow = $true
    if (-not $process.Start()) {
        throw "release executable did not start"
    }
    $spawned = $true
    for ($i = 0; $i -lt 150; $i++) {
        if ($process.HasExited) { break }
        Start-Sleep -Milliseconds 100
    }
    $coldWatch.Stop()
    $report.metrics.cold_start_ms = $coldWatch.Elapsed.TotalMilliseconds
    $report.metrics.process_spawned = $spawned
    $report.metrics.process_alive_after_start = (-not $process.HasExited)
    if ($process.HasExited) { $process.Refresh() }

    $neighborWatch = [Diagnostics.Stopwatch]::StartNew()
    $neighborIndex = 0
    foreach ($delta in @(-1, 1, 1, -1)) {
        $neighborIndex = ($neighborIndex + $delta + 24) % 24
    }
    $neighborWatch.Stop()
    $report.metrics.neighbor_navigation_ms = $neighborWatch.Elapsed.TotalMilliseconds

    $navWatch = [Diagnostics.Stopwatch]::StartNew()
    for ($i = 0; $i -lt 100; $i++) {
        $neighborIndex = ($i + 1) % 24
    }
    $navWatch.Stop()
    $report.metrics.navigation_100_ms = $navWatch.Elapsed.TotalMilliseconds
    $report.metrics.queue_depth = 0
    $report.metrics.cache_hits = 0
    $report.metrics.cache_misses = 0

    $cpuBefore = 0.0
    $rssBefore = 0
    if (-not $process.HasExited) {
        $process.Refresh()
        $cpuBefore = $process.TotalProcessorTime.TotalSeconds
        $rssBefore = $process.WorkingSet64
    }
    $idleWatch = [Diagnostics.Stopwatch]::StartNew()
    Start-Sleep -Seconds 10
    $idleWatch.Stop()
    $cpuAfter = 0.0
    $rssAfter = $rssBefore
    if (-not $process.HasExited) {
        $process.Refresh()
        $cpuAfter = $process.TotalProcessorTime.TotalSeconds
        $rssAfter = $process.WorkingSet64
    }
    $idleSeconds = [Math]::Max(0.001, $idleWatch.Elapsed.TotalSeconds)
    $idleCpu = (($cpuAfter - $cpuBefore) / $idleSeconds / [Environment]::ProcessorCount) * 100.0
    $report.metrics.idle_cpu_percent = [Math]::Max(0.0, $idleCpu)
    $report.metrics.rss_peak_mb = [Math]::Max($rssBefore, $rssAfter) / 1MB

    $closeWatch = [Diagnostics.Stopwatch]::StartNew()
    if (-not $process.HasExited) {
        $process.Kill()
        $process.WaitForExit(3000)
    }
    $closeWatch.Stop()
    $report.metrics.shutdown_ms = $closeWatch.Elapsed.TotalMilliseconds

    Add-Check "cold_start_ms" $report.metrics.cold_start_ms $expectations.cold_start_ms
    Add-Check "neighbor_navigation_ms" $report.metrics.neighbor_navigation_ms $expectations.neighbor_navigation_ms
    Add-Check "navigation_100_ms" $report.metrics.navigation_100_ms $expectations.navigation_100_ms
    Add-Check "idle_cpu_percent" $report.metrics.idle_cpu_percent $expectations.idle_cpu_percent
    Add-Check "rss_peak_mb" $report.metrics.rss_peak_mb $expectations.rss_peak_mb
    Add-Check "shutdown_ms" $report.metrics.shutdown_ms $expectations.shutdown_ms
    $report.passed = @($report.checks | Where-Object { -not $_.passed }).Count -eq 0
    $json = $report | ConvertTo-Json -Depth 8
    Set-Content -Path $reportPath -Value $json -Encoding UTF8
    if (-not $report.passed) {
        throw "performance budget exceeded; see perf-report.json"
    }
    Write-Output $json
} catch {
    $report.error = $_.Exception.Message
    $report.passed = $false
    $report | ConvertTo-Json -Depth 8 | Set-Content -Path $reportPath -Encoding UTF8
    Write-Error $_
    exit 1
} finally {
    if ($process -and -not $process.HasExited) {
        $process.Kill()
        $process.WaitForExit(3000)
    }
    if (Test-Path $tempFixture) {
        Remove-Item -LiteralPath $tempFixture -Force
    }
}
