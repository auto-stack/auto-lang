# Fixed-group consumer: validate original actual identities, never banner text.
function Invoke-FixedTestGroups {
    param([Parameter(Mandatory=$true)][string]$Repo,
          [Parameter(Mandatory=$true)][string[]]$Groups,
          [Parameter(Mandatory=$true)][string]$Output,
          [string[]]$RequiredIds=@(), [string[]]$RequiredNamePatterns=@())
    if(-not $Groups.Count){throw 'Explicit fixed groups are required'}
    $runOutput=Join-Path $Output ([guid]::NewGuid().ToString('N'))
    $arguments=@('t') + $Groups + @('--out',$runOutput)
    Push-Location $Repo
    try {
        & cargo @arguments | Out-Host
        if($LASTEXITCODE -ne 0){throw "Fixed group execution failed ($LASTEXITCODE); preserve $Output"}
        $receipt=Get-Content -LiteralPath (Join-Path $runOutput 'results.json') -Raw -ErrorAction Stop | ConvertFrom-Json
        $active=@($receipt.results | Where-Object {$_.outcome -ne 'NOT_APPLICABLE'})
        if($receipt.schema -ne 'fixed-test-run/1' -or -not $receipt.gate_passed -or -not $active.Count){throw 'Incomplete or failed fixed group receipt'}
        $declared=@($receipt.results | ForEach-Object {$_.groups} | Sort-Object -Unique)
        if(@(Compare-Object @($Groups | Sort-Object -Unique) $declared).Count){throw 'Fixed group receipt does not cover the requested groups'}
        $executed=@()
        foreach($group in $active){
            if($group.outcome -ne 'PASS' -or $group.executed -le 0){throw 'Fixed group body did not pass'}
            if(-not $group.results){throw 'Expected native actual test terminals in verifier receipt'}
            $terminals=@($group.results.PSObject.Properties)
            if($terminals.Count -ne $group.executed -or @($terminals | Where-Object {$_.Value.event -ne 'ok'}).Count){throw 'Incomplete or non-passing native terminals'}
            $executed+=@($terminals | ForEach-Object {$_.Name})
        }
        foreach($id in $RequiredIds){
            $suffix='(^|\||::|\.|\$)'+[regex]::Escape($id)+'$'
            if(-not @($executed | Where-Object {$_ -match $suffix}).Count){throw "Required original identity absent: $id"}
        }
        foreach($pattern in $RequiredNamePatterns){
            if(-not @($executed | Where-Object {$_ -match $pattern}).Count){throw "Required original test scope absent: $pattern"}
        }
    } finally {Pop-Location}
}
