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
$env:CARGO_TARGET_DIR=$target
$jsonLines = & cargo build --manifest-path (Join-Path $repo 'experimental/ac-core/Cargo.toml') --locked --offline --lib --bins --message-format=json
if($LASTEXITCODE -ne 0){throw 'Reviewed library build failed'}
$artifacts = @($jsonLines | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'auto_ac_prototype' -and $_.target.kind -contains 'lib' -and -not $_.profile.test })
if($artifacts.Count -ne 1){throw "Ambiguous Cargo artifact count: $($artifacts.Count)"}
$libs = @($artifacts[0].filenames | Where-Object { $_.EndsWith('.rlib') })
if($libs.Count -ne 1 -or -not (Test-Path -LiteralPath $libs[0] -PathType Leaf)){throw 'No unique Cargo-reported library'}
$rlib=$libs[0]
$bins = @($jsonLines | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'auto-ac-prototype' -and $_.target.kind -contains 'bin' -and -not $_.profile.test })
if($bins.Count -ne 1 -or -not (Test-Path -LiteralPath $bins[0].executable -PathType Leaf)){throw 'No unique Cargo-reported CLI'}
$reviewedCli=$bins[0].executable
$jsonLines | Set-Content -LiteralPath (Join-Path $scratch 'cargo-artifacts.jsonl') -Encoding UTF8
Get-FileHash -LiteralPath $rlib -Algorithm SHA256 | Format-List
Write-Output "Cargo-reported reviewed library: $rlib"
$native=Join-Path $env:USERPROFILE '.cargo/registry/src'
$windowsLib=Get-ChildItem -LiteralPath $native -Filter 'windows.0.52.0.lib' -Recurse |
    Where-Object {$_.FullName -match 'windows_x86_64_msvc-0\.52\.6'} | Select-Object -First 1
if(-not $windowsLib){throw 'windows_x86_64_msvc-0.52.6 native library is unavailable'}
$helper=Join-Path $scratch 'link-cleanup-probe.exe'
& rustc --edition=2021 (Join-Path $PSScriptRoot 'link-cleanup-probe.rs') -L "dependency=$deps" -L "native=$($windowsLib.DirectoryName)" --extern "auto_ac_prototype=$rlib" -o $helper
if($LASTEXITCODE -ne 0){throw 'Review helper compilation failed'}
$cli=$reviewedCli
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
