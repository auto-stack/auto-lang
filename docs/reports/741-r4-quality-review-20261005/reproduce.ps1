param(
    [Parameter(Mandatory=$true)][string]$RepoRoot,
    [Parameter(Mandatory=$true)][string]$CargoTargetDir,
    [Parameter(Mandatory=$true)][string]$ScratchRoot,
    [switch]$ProductionDeadline
)
$ErrorActionPreference='Stop'
$repo=(Resolve-Path -LiteralPath $RepoRoot).Path
$target=(Resolve-Path -LiteralPath $CargoTargetDir).Path
if(Test-Path -LiteralPath $ScratchRoot){throw 'Use a new isolated scratch directory'}
New-Item -ItemType Directory -Path $ScratchRoot | Out-Null
$scratch=(Resolve-Path -LiteralPath $ScratchRoot).Path
$deps=Join-Path $target 'debug/deps'
$rlib=(Get-ChildItem -LiteralPath $deps -Filter 'libauto_ac_prototype-*.rlib' | Select-Object -First 1).FullName
if(-not $rlib){throw 'Build the reviewed prototype first (cargo test --locked)'}
$native=Join-Path $env:USERPROFILE '.cargo/registry/src'
$windowsLib=Get-ChildItem -LiteralPath $native -Filter 'windows.0.52.0.lib' -Recurse |
    Where-Object {$_.FullName -match 'windows_x86_64_msvc-0\.52\.6'} | Select-Object -First 1
if(-not $windowsLib){throw 'windows_x86_64_msvc-0.52.6 native library is unavailable'}
$helper=Join-Path $scratch 'link-cleanup-probe.exe'
& rustc --edition=2021 (Join-Path $PSScriptRoot 'link-cleanup-probe.rs') -L "dependency=$deps" -L "native=$($windowsLib.DirectoryName)" --extern "auto_ac_prototype=$rlib" -o $helper
if($LASTEXITCODE -ne 0){throw 'Review helper compilation failed'}
$cli=Join-Path $target 'debug/auto-ac-prototype.exe'
$baseline=Join-Path $scratch 'baseline'
& $cli build (Join-Path $repo 'experimental/ac-core/fixtures/native/add-2-3.atom') --entry d_entry --output (Join-Path $baseline 'old.exe')
if($LASTEXITCODE -ne 0){throw 'Real PE/COFF baseline build failed'}
function Invoke-Probe([string]$Mode,[int]$TimeoutSeconds){
    $caseDir=Join-Path $scratch $Mode
    $stdout=Join-Path $scratch ($Mode+'.txt')
    $stderr=Join-Path $scratch ($Mode+'.stderr.txt')
    $probeArgs=@($Mode,$caseDir)
    if($Mode -ne 'public-deadline'){$probeArgs+=@($baseline)}
    $psi=New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName=$helper
    $psi.Arguments=($probeArgs | ForEach-Object {'"'+$_+'"'}) -join ' '
    $psi.UseShellExecute=$false
    $psi.CreateNoWindow=$true
    $psi.RedirectStandardOutput=$true
    $psi.RedirectStandardError=$true
    $p=[System.Diagnostics.Process]::Start($psi)
    $outTask=$p.StandardOutput.ReadToEndAsync()
    $errTask=$p.StandardError.ReadToEndAsync()
    try {
        if(-not $p.WaitForExit($TimeoutSeconds*1000)){throw "Outer timeout: $Mode"}
        [IO.File]::WriteAllText($stdout,$outTask.GetAwaiter().GetResult(),[Text.UTF8Encoding]::new($false))
        [IO.File]::WriteAllText($stderr,$errTask.GetAwaiter().GetResult(),[Text.UTF8Encoding]::new($false))
        if($p.ExitCode -ne 0){throw "Probe exited $($p.ExitCode): $(Get-Content -LiteralPath $stderr -Raw)"}
        Get-Content -LiteralPath $stdout
    } finally {
        if(-not $p.HasExited){$own=Get-Process -Id $p.Id -ErrorAction SilentlyContinue;if($own -and $own.Path -eq $helper){Stop-Process -Id $own.Id -Force}}
        $pidFile=Join-Path $caseDir 'tree-child.pid'
        if(Test-Path -LiteralPath $pidFile -PathType Leaf){
            $childId=[int](Get-Content -LiteralPath $pidFile -Raw)
            $child=Get-Process -Id $childId -ErrorAction SilentlyContinue
            if($child -and $child.Path -eq (Join-Path $caseDir 'tree-child.exe')){Stop-Process -Id $child.Id -Force}
        }
    }
}
Invoke-Probe 'nonzero' 15
if($ProductionDeadline){Invoke-Probe 'timeout' 75;Invoke-Probe 'public-deadline' 75}
Write-Output "Evidence directory: $scratch"
