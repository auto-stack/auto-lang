# Transitional verifier argv adapter. No test catalog, source scan or dependency graph.
# Callers declare current Cargo packages and semantic Rust modules; Cargo remains the runner.
function Convert-AceCargoArguments {
    param([string[]]$CargoArguments, [string]$WorkingDirectory = (Get-Location).Path)
    $CargoArguments=@($CargoArguments)
    # Current semantic packages always resolve against the independent workspace,
    # even when a verifier starts in the language checkout.
    if(($CargoArguments -match '^ace-').Count){
        $manifestIndex=[Array]::IndexOf($CargoArguments,'--manifest-path')
        $workspaceManifest=Join-Path $PSScriptRoot '../../experimental/ace/Cargo.toml'
        if($manifestIndex -ge 0){$CargoArguments[$manifestIndex+1]=$workspaceManifest}
        else {$CargoArguments = @($CargoArguments[0], '--manifest-path', $workspaceManifest) + @($CargoArguments | Select-Object -Skip 1)}
    }
    if(-not $CargoArguments.Count -or $CargoArguments[0] -ne 'test'){return ,$CargoArguments}
    $isAce=($WorkingDirectory -replace '\\','/') -match '/experimental/ace(?:/|$)'
    foreach($value in $CargoArguments){if(($value -replace '\\','/') -match 'experimental/ace|^ace-'){$isAce=$true}}
    if(-not $isAce){return ,$CargoArguments}
    $native=New-Object System.Collections.Generic.List[string]
    $modules=New-Object System.Collections.Generic.List[string]
    $filters=New-Object System.Collections.Generic.List[string]
    $harness=New-Object System.Collections.Generic.List[string]
    $native.Add('test');$afterSeparator=$false
    $takesValue=@('-p','--package','--manifest-path','--features','--target','--target-dir','--profile','--jobs','-j','--exclude','--config')
    for($i=1;$i -lt $CargoArguments.Count;$i++){
        $value=$CargoArguments[$i]
        if($afterSeparator){
            if($value -in @('--skip','--test-threads','--format','--logfile','--shuffle-seed')){
                $harness.Add($value);$i++;$harness.Add($CargoArguments[$i]);continue
            }
            if($value.StartsWith('-')){$harness.Add($value)}else{$filters.Add($value)}
            continue
        }
        if($value -eq '--'){$afterSeparator=$true;continue}
        if($value -eq '--test'){
            $i++;$target=$CargoArguments[$i]
            if($target -eq 'conformance'){$native.Add('--test');$native.Add($target)}else{$modules.Add($target)}
            continue
        }
        if($takesValue -contains $value){$native.Add($value);$i++;$native.Add($CargoArguments[$i]);continue}
        if($value.StartsWith('-')){$native.Add($value)}else{$filters.Add($value)}
    }
    if(-not $modules.Count){return ,$CargoArguments}
    if($native -contains '--lib'){throw 'Mixed lib/module verifier: declare separate fixed groups explicitly'}
    $native.Add('--test');$native.Add('conformance');$native.Add('--')
    foreach($module in $modules){
        if($filters.Count){foreach($filter in $filters){$native.Add($module+'::'+$filter);$native.Add($module+'::performance::'+$filter)}}
        else{$native.Add($module+'::')}
    }
    foreach($flag in $harness){$native.Add($flag)}
    return ,$native.ToArray()
}

function Invoke-AceCargo {
    $original=@($args)
    $converted=Convert-AceCargoArguments -CargoArguments $original
    # A caller-supplied mocked cargo stays mocked (required-ID negative controls).
    $command=Get-Command cargo -ErrorAction Stop
    $previousPreference=$ErrorActionPreference
    try {
        $ErrorActionPreference='Continue'
        $output=& $command @converted 2>&1
        $code=$LASTEXITCODE
    } finally {$ErrorActionPreference=$previousPreference}
    $output | ForEach-Object {Write-Output $_}
    if($code -eq 0 -and $converted[0] -eq 'test' -and $converted -notcontains '--no-run' -and (($converted -join ' ') -ne ($original -join ' '))){
        $summaries=@($output | Select-String 'test result:.*?([0-9]+) passed;')
        $listed=($converted -contains '--list') -and @($output | Select-String ': test$').Count -gt 0
        if(-not $listed -and (-not $summaries.Count -or -not @($summaries | Where-Object {$_.Matches[0].Groups[1].Value -ne '0'}).Count)){
            Write-Output 'ACE verifier selected zero actual tests; update the declared semantic module'
            $code=1
        }
        for($i=0;$i -lt $original.Count-1;$i++){
            if($original[$i] -eq '--test' -and $original[$i+1] -ne 'conformance'){
                $module=$original[$i+1]
                $prefix=[regex]::Escape($module)+'::'
                $rows=@($output | Select-String ('^(?:test )?'+$prefix+'.*(?: \.\.\. ok|: test)$'))
                if(-not $rows.Count){Write-Output "ACE verifier module has no passing/listed case: $module";$code=1}
            }
        }
    }
    $global:LASTEXITCODE=$code
}
