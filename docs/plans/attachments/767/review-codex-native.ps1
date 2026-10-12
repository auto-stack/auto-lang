# Independent runtime probes. No product writes; artifacts go under reviewed target.
param([Parameter(Mandatory=$true)][string]$Worktree)
$ErrorActionPreference = 'Stop'
$Worktree = (Resolve-Path -LiteralPath $Worktree).Path
$scratch = Join-Path $Worktree 'experimental/ac-core/target/review-codex-767'
New-Item -ItemType Directory -Force -Path $scratch | Out-Null
$gate = Get-Content -LiteralPath (Join-Path $Worktree 'scripts/verify-ac-scalar-767.ps1') -Raw
$start = $gate.IndexOf('using System;')
$end = $gate.IndexOf("`n'@", $start)
if ($start -lt 0 -or $end -le $start) { throw 'contained runner source not found' }
Add-Type -TypeDefinition $gate.Substring($start, $end-$start) -Language CSharp
$binary = Join-Path $Worktree 'experimental/ac-core/target/debug/auto-ac-prototype.exe'
$support = Join-Path $Worktree 'experimental/ac-core/test-support/target/debug/ac_trace_support.lib'
$cases = @(
    @{ Id='ARITHMETIC'; Source='review-codex-arithmetic.at'; Trace=$false; Exit=0; Stderr='' },
    @{ Id='SHORTCIRCUIT'; Source='review-codex-shortcircuit.at'; Trace=$true; Exit=0; Stderr='abaabaabababaabaaba' }
)
$results = @()
foreach ($case in $cases) {
    $source = Join-Path $PSScriptRoot $case.Source
    $output = Join-Path $scratch ($case.Id + '.exe')
    $buildArgs = @('build-source', $source, '--source-profile', 'auto.source.scalar-i32.draft', '--entry', 'main', '--output', $output)
    if ($case.Trace) { $buildArgs += @('--capability', 'hir.test.trace.v1', '--support-lib', $support) }
    $build = [AcContainedRunner]::Run($binary, [string[]]$buildArgs, $Worktree, 60000)
    if ($build.Error -or $build.TimedOut -or $build.Code -ne 0) { throw ($case.Id + ': build failed ' + $build.Error + $build.Stderr) }
    $run = [AcContainedRunner]::Run($output, [string[]]@(), $Worktree, 60000)
    $ok = -not $run.Error -and -not $run.TimedOut -and $run.Code -eq $case.Exit -and $run.Stderr -ceq $case.Stderr -and $run.Stdout -ceq ''
    $row = [pscustomobject]@{
        Case=$case.Id; Ok=$ok; Exit=$run.Code; Stderr=$run.Stderr; Stdout=$run.Stdout
        SourceHash=(Get-FileHash -Algorithm SHA256 -LiteralPath $source).Hash
        ExeHash=(Get-FileHash -Algorithm SHA256 -LiteralPath $output).Hash
        BinaryHash=(Get-FileHash -Algorithm SHA256 -LiteralPath $binary).Hash
        SupportHash=(Get-FileHash -Algorithm SHA256 -LiteralPath $support).Hash
    }
    $results += $row
    Write-Output ($case.Id + ' exit=' + $run.Code + ' stderr=' + $run.Stderr + ' ok=' + $ok)
    if (-not $ok) { throw ($case.Id + ': native assertion failed') }
}
$results | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $scratch 'native-results.json') -Encoding UTF8
