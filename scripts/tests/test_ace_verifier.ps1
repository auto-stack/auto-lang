# Real verifier admission controls, no Cargo build required.
. (Join-Path $PSScriptRoot '../test-governance/ace-cargo.ps1')
$ErrorActionPreference='Stop'
function Assert($condition,$message){if(-not $condition){throw $message}}
function cargo {
    $script:received=@($args)
    $global:LASTEXITCODE=$script:mockExit
    $script:mockOutput
}
$script:mockExit=0
$script:mockOutput=@('test records::roundtrip ... ok','test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out')
$null=Invoke-AceCargo test -p ace-core --test records '--' --test-threads 1 --skip rejected
Assert ($LASTEXITCODE -eq 0) 'valid semantic module rejected'
Assert (($script:received -join ' ') -match '--test conformance -- records:: --test-threads 1 --skip rejected') 'native flags/filter scope lost'
$null=Invoke-AceCargo test -p ace-core --test records --test missing
Assert ($LASTEXITCODE -ne 0) 'missing second module accepted beside valid module'
$script:mockOutput=@('test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out')
$null=Invoke-AceCargo test -p ace-core --test records
Assert ($LASTEXITCODE -ne 0) 'zero execution accepted'
$script:mockOutput=@('records::roundtrip: test')
$null=Invoke-AceCargo test -p ace-core --test records '--' --list
Assert ($LASTEXITCODE -eq 0) 'real selected listing rejected'
$script:mockExit=101
$null=Invoke-AceCargo test -p ace-core --test records
Assert ($LASTEXITCODE -eq 101) 'Cargo failure not propagated'
$script:mockExit=0;$script:mockOutput='build finished'
$null=Invoke-AceCargo build -p ace-core
Assert ($LASTEXITCODE -eq 0) 'build mistaken for executed test'
$untouched=@('test','-p','auto-test-case','--test','contract_decode')
Assert (((Convert-AceCargoArguments -CargoArguments $untouched) -join '|') -eq ($untouched -join '|')) 'non-ACE target changed'
Write-Output 'PASS: ACE verifier controls (valid, zero, missing module, list, native error, build, non-ACE)'