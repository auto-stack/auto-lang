# Shared consumer of the actual module execution receipt, not nextest banner text.
function Invoke-ScopedTestModules {
    param([Parameter(Mandatory=$true)][string]$Repo,
          [string[]]$TestNames=@(), [string[]]$TestRegex=@(),
          [string]$ChangesFile, [Parameter(Mandatory=$true)][string]$Output,
          [string[]]$RequiredIds=@())
    $runOutput=Join-Path $Output ([guid]::NewGuid().ToString('N'))
    $arguments=@('tm','--out',$runOutput)
    foreach($name in $TestNames){$arguments+=@('--test-name',$name)}
    foreach($pattern in $TestRegex){$arguments+=@('--test-regex',$pattern)}
    if($ChangesFile){$arguments+=@('--changes',$ChangesFile)}
    if(-not $TestNames.Count -and -not $TestRegex.Count -and -not $ChangesFile){throw 'An explicit affected test/module input is required'}
    Push-Location $Repo
    try {
        & cargo @arguments | Out-Host
        if($LASTEXITCODE -ne 0){throw "Module execution failed ($LASTEXITCODE); preserve $Output"}
        $receipt=Get-Content -LiteralPath (Join-Path $runOutput 'execution/results.json') -Raw -ErrorAction Stop | ConvertFrom-Json
        $active=@($receipt.results | Where-Object {$_.outcome -ne 'NOT_APPLICABLE'})
        if($receipt.scope -ne 'module' -or -not $receipt.complete -or -not $receipt.gate_passed -or -not $active.Count){throw 'Incomplete or failed module receipt'}
        foreach($module in $active){
            if($module.outcome -ne 'PASS' -or $module.exit -ne 0 -or -not @($module.applicable).Count){throw 'Module body did not pass'}
        }
        $executed=@($active | ForEach-Object {$_.applicable})
        foreach($id in $RequiredIds){
            $suffix='(^|\||::|\.)'+[regex]::Escape($id)+'$'
            if(-not @($executed | Where-Object {$_ -match $suffix}).Count){throw "Required original identity absent from executed modules: $id"}
        }
    } finally {Pop-Location}
}
