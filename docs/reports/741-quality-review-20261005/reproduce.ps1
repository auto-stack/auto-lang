# PowerShell 7; run in a dedicated review/fix worktree.
param(
    [Parameter(Mandatory=$true)][string]$RepoRoot,
    [switch]$SkipWatchdog
)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path -LiteralPath $RepoRoot).Path
$evidence = $PSScriptRoot
$manifest = Join-Path $repo 'experimental/ac-core/Cargo.toml'
$outDir = Join-Path ([IO.Path]::GetTempPath()) ('p741-r2-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $outDir | Out-Null
$buildMessages = @(& cargo build --manifest-path $manifest --locked --message-format=json)
if ($LASTEXITCODE -ne 0) { throw 'prototype build failed' }
$targetDir = Join-Path $repo 'experimental/ac-core/target/debug'
$cli = Join-Path $targetDir 'auto-ac-prototype.exe'
function Invoke-Probe([string]$Exe, [string[]]$Arguments, [string]$Name) {
    # All input/output paths belong to this invocation; never replace project artifacts.
    $stdout = Join-Path $outDir ($Name + '.stdout.txt')
    $stderr = Join-Path $outDir ($Name + '.stderr.txt')
    $quoted = $Arguments | ForEach-Object { '"' + $_ + '"' }
    $proc = Start-Process -FilePath $Exe -ArgumentList $quoted -PassThru -WindowStyle Hidden -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    if (-not $proc.WaitForExit(20000)) { $proc.Kill(); $proc.WaitForExit(); throw "$Name exceeded outer 20s deadline" }
    return [pscustomobject]@{name=$Name; exit=$proc.ExitCode; stdout=(Get-Content -LiteralPath $stdout -Raw); stderr=(Get-Content -LiteralPath $stderr -Raw)}
}
$observations = @()
foreach ($case in @('block-self-cycle','block-disconnected-cycle')) {
    $observations += Invoke-Probe $cli @('check',(Join-Path $evidence ("fixtures/$case.atom"))) $case
}
$artifacts = @($buildMessages | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'auto_ac_prototype' })
$rlibs = @($artifacts | ForEach-Object { $_.filenames } | Where-Object { $_.EndsWith('.rlib') } | Select-Object -Unique)
if ($rlibs.Count -ne 1) { throw 'Expected one current build library artifact' }
$helper = Join-Path $outDir 'review-helper.exe'
& rustc --edition=2021 -C linker=rust-lld.exe -L ('dependency=' + (Join-Path $targetDir 'deps')) --extern ('auto_ac_prototype=' + $rlibs[0]) (Join-Path $evidence 'review-helper.rs') -o $helper
if ($LASTEXITCODE -ne 0) { throw 'helper compilation failed' }
$baseline = Join-Path $outDir 'baseline.exe'
$observations += Invoke-Probe $cli @('build',(Join-Path $repo 'experimental/ac-core/fixtures/native/add-2-3.atom'),'--entry','d_entry','--output',$baseline) 'baseline'
$observations += Invoke-Probe $helper @('receipt-prepare',(Join-Path $outDir 'receipt-prepare'),$baseline,(Join-Path $outDir 'baseline.obj')) 'receipt-prepare'
if (-not $SkipWatchdog) {
    $pidFile = Join-Path $outDir 'pipe-child.pid'
    $previousPidFile = $env:P741_REVIEW_CHILD_PID
    $env:P741_REVIEW_CHILD_PID = $pidFile
    $watchdog = $null
    $clock = [Diagnostics.Stopwatch]::StartNew()
    try {
        $watchdog = Start-Process -FilePath $helper -ArgumentList 'watchdog' -PassThru -WindowStyle Hidden -RedirectStandardOutput (Join-Path $outDir 'watchdog.stdout.txt') -RedirectStandardError (Join-Path $outDir 'watchdog.stderr.txt')
        $completed = $watchdog.WaitForExit(55000)
        if (-not $completed) { $completed = $watchdog.WaitForExit(10000) }
        $observations += [pscustomobject]@{name='watchdog'; configuredDeadlineSeconds=60; observedSeconds=$clock.Elapsed.TotalSeconds; stillRunningAfterDeadline=(-not $completed)}
    } finally {
        # Terminate only this invocation's exact helper path/PIDs, including inherited-pipe child.
        if ($null -ne $watchdog -and -not $watchdog.HasExited) {
            $live = Get-Process -Id $watchdog.Id -ErrorAction SilentlyContinue
            if ($null -ne $live -and $live.Path -eq $helper) { Stop-Process -Id $live.Id -Force }
        }
        if (Test-Path -LiteralPath $pidFile -PathType Leaf) {
            $childId = [int](Get-Content -LiteralPath $pidFile -Raw)
            $liveChild = Get-Process -Id $childId -ErrorAction SilentlyContinue
            if ($null -ne $liveChild -and $liveChild.Path -eq $helper) { Stop-Process -Id $childId -Force }
        }
        $env:P741_REVIEW_CHILD_PID = $previousPidFile
    }
}
$observations | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $outDir 'observations.json') -Encoding utf8NoBOM
Write-Output "Evidence directory: $outDir"
$observations | ConvertTo-Json -Depth 5
