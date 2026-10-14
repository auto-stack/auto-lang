param([string]$Repo = 'D:/autostack/.wt/lang-770/auto-lang', [switch]$HirOnly)
$ErrorActionPreference = 'Stop'
$evidence = $PSScriptRoot
$runnerSource = [IO.File]::ReadAllText((Join-Path $Repo 'scripts/verify-ac-source-745.ps1'))
$runnerMatch = [regex]::Match($runnerSource, '(?s)\$script:AcRunnerCs = @''\r?\n(.*?)\r?\n''@')
if (-not $runnerMatch.Success) { throw 'existing contained executor not found' }
Add-Type -TypeDefinition $runnerMatch.Groups[1].Value -Language CSharp
$outDir = Join-Path $Repo 'experimental/ac-core/target/review-770-r8'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$cli = Join-Path $Repo 'experimental/ac-core/target/debug/auto-ac-prototype.exe'
$lib = Join-Path $Repo 'experimental/ac-core/runtime/text/target/release/ac_text_runtime.lib'
$cases = if ($HirOnly) { @() } else { Get-ChildItem -LiteralPath $evidence -Filter '*.at' | Sort-Object Name }
foreach ($case in $cases) {
    $check = [AcContainedRunner]::Run($cli, @('check-source', $case.FullName, '--source-profile', 'auto.source.text.draft'), $Repo, 60000)
    $exe = Join-Path $outDir ($case.BaseName + '.exe')
    $buildArgs = @('build-source', $case.FullName, '--source-profile', 'auto.source.text.draft', '--entry', 'main', '--output', $exe, '--capability', 'ac.runtime.text.v1', '--support-lib', $lib)
    if ([IO.File]::ReadAllText($case.FullName).Contains('mark_')) {
        $buildArgs += @('--capability', 'hir.test.trace.v1', '--support-lib', (Join-Path $Repo 'experimental/ac-core/test-support/target/marks-text.obj'))
        $check = [AcContainedRunner]::Run($cli, @('check-source', $case.FullName, '--source-profile', 'auto.source.text.draft', '--capability', 'hir.test.trace.v1'), $Repo, 60000)
    }
    $build = [AcContainedRunner]::Run($cli, $buildArgs, $Repo, 60000)
    $run = $null
    if ($build.Code -eq 0 -and -not $build.Error -and -not $build.TimedOut) {
        $run = [AcContainedRunner]::Run($exe, @(), $Repo, 60000)
    }
    $result = [ordered]@{name=$case.Name; input_sha256=(Get-FileHash -LiteralPath $case.FullName -Algorithm SHA256).Hash; head=(& git -C $Repo rev-parse HEAD).Trim(); check=$check; build=$build; run=$run}
    $json = $result | ConvertTo-Json -Depth 6
    [IO.File]::WriteAllText((Join-Path $evidence ($case.BaseName + '.json')), $json)
    Write-Output $json
}
foreach ($case in (Get-ChildItem -LiteralPath $evidence -Filter '*.atom' | Sort-Object Name)) {
    $check = [AcContainedRunner]::Run($cli, @('check', $case.FullName), $Repo, 60000)
    $exe = Join-Path $outDir ($case.BaseName + '-hir.exe')
    $buildArgs = @('build', $case.FullName, '--entry', 'd_main', '--output', $exe, '--capability', 'ac.runtime.text.v1', '--support-lib', $lib)
    if ($case.BaseName -in @('mut02-no-runtime-requires','q05-cleanup-no-requires')) {
        $uncheckedLib = Join-Path $outDir 'unchecked-runtime.lib'
        Copy-Item -LiteralPath $lib -Destination $uncheckedLib -Force
        if (Test-Path -LiteralPath ($uncheckedLib + '.contract.json')) { throw 'unchecked library unexpectedly has a contract' }
        $buildArgs = @('build', $case.FullName, '--entry', 'd_main', '--output', $exe, '--support-lib', $uncheckedLib)
    }
    if ([IO.File]::ReadAllText($case.FullName).Contains('hir.test.trace.v1')) {
        $buildArgs += @('--capability', 'hir.test.trace.v1', '--support-lib', (Join-Path $Repo 'experimental/ac-core/test-support/target/marks-text.obj'))
    }
    $build = [AcContainedRunner]::Run($cli, $buildArgs, $Repo, 60000)
    $run = $null
    if ($build.Code -eq 0 -and -not $build.Error -and -not $build.TimedOut) {
        $run = [AcContainedRunner]::Run($exe, @(), $Repo, 60000)
    }
    $result = [ordered]@{name=$case.Name; input_sha256=(Get-FileHash -LiteralPath $case.FullName -Algorithm SHA256).Hash; head=(& git -C $Repo rev-parse HEAD).Trim(); check=$check; build=$build; run=$run}
    $json = $result | ConvertTo-Json -Depth 6
    [IO.File]::WriteAllText((Join-Path $evidence ($case.BaseName + '.hir.json')), $json)
    Write-Output $json
}
