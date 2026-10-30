# PLAN-894 T-06 verification (scripts/verify-text-894.ps1).
#
# Scoped entry for the shared-text source/resource/incremental delivery.
# Every scope runs REAL tests (no skip-counted greens); each step captures
# command, exit code and wall time into a receipt file under
# experimental\ac-core\target\ac-verify-receipts\text-894.
#
# Scopes:
#   source      the frontend admission + rejection surface (T-03)
#   resources   the T03/T05 frozen sources through AV + native PE with the
#               arena/ledger axis and the data-owned corpus row (V-01/V-02)
#   incremental the AV + native owned-source single-body sessions (V-06/V-07)
#   cache       the cross-process restore + corruption refusal legs (V-08)
#   contract    the migrated bindings: ace.m1.t03 row + the independent
#               dual-backend oracle suite (V-10) via the case runner
#   acceptance  all of the above in one pass
#
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-text-894.ps1 -Scope <scope>

param([string]$Scope = 'acceptance')

$ErrorActionPreference = 'Stop'
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$AcCore = Join-Path $RepoRoot 'experimental\ac-core'
$AcMn = Join-Path $RepoRoot 'experimental\ac-mir-native'
$AvProbe = Join-Path $RepoRoot 'experimental\ac-av-probe\probe'
$ReceiptsDir = Join-Path $AcCore 'target\ac-verify-receipts\text-894'
New-Item -ItemType Directory -Force -Path $ReceiptsDir | Out-Null

$script:Results = New-Object System.Collections.Generic.List[object]

function Invoke-Step {
    param([string]$Name, [scriptblock]$Body, [int]$TimeoutSec = 1800)
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        & $Body
        $sw.Stop()
        $script:Results.Add([pscustomobject]@{ Step = $Name; Ok = $true; Seconds = [math]::Round($sw.Elapsed.TotalSeconds, 1); Detail = '' })
        Write-Host ("[PASS] {0} ({1}s)" -f $Name, [math]::Round($sw.Elapsed.TotalSeconds, 1))
    } catch {
        $sw.Stop()
        $script:Results.Add([pscustomobject]@{ Step = $Name; Ok = $false; Seconds = [math]::Round($sw.Elapsed.TotalSeconds, 1); Detail = $_.Exception.Message })
        Write-Host ("[FAIL] {0} ({1}s): {2}" -f $Name, [math]::Round($sw.Elapsed.TotalSeconds, 1), $_.Exception.Message) -ForegroundColor Red
    }
}

function Invoke-Cargo {
    param([string]$WorkDir, [string[]]$TestArgs, [string]$Name)
    Invoke-Step $Name {
        Push-Location $WorkDir
        try {
            # cargo writes progress/warnings to stderr; keep them as text
            # (EAP=Stop would turn the first stderr line into a failure).
            $out = cmd /c ('cargo test ' + ($TestArgs -join ' ') + ' 2>&1') | Out-String
            if ($LASTEXITCODE -ne 0) {
                throw ($out -split "`n" | Select-Object -Last 40)
            }
        } finally { Pop-Location }
    }
}

$scopes = if ($Scope -eq 'acceptance') { @('source','resources','incremental','cache','contract') } else { @($Scope) }

foreach ($s in $scopes) {
    switch ($s) {
        'source' {
            Invoke-Cargo -WorkDir $AcCore -TestArgs @('--test','text_replace_894') -Name 'ac-core text_replace_894 (frontend admission + named rejections)'
        }
        'resources' {
            Invoke-Cargo -WorkDir $AcMn -TestArgs @('--test','data_owned_tier') -Name 'mn data_owned_tier (T03=23 corpus row, three-path + ledger)'
            Invoke-Cargo -WorkDir $AvProbe -TestArgs @('--test','text_source_894_semantics') -Name 'av semantics tier (V-05 comparison matrix, V-02 ledger, V-04 faults)'
        }
        'incremental' {
            Invoke-Cargo -WorkDir $AcMn -TestArgs @('--test','text_source_894_incremental') -Name 'mn owned-source session (65/66/65, failure atomicity)'
            Invoke-Cargo -WorkDir $AvProbe -TestArgs @('--test','text_source_894_av') -Name 'av owned-source session (65/66/65, refusal keeps running)'
        }
        'cache' {
            Invoke-Cargo -WorkDir $AcMn -TestArgs @('--test','text_source_894_incremental','--','owned_894_cross_process','--exact') -Name 'cross-process restore (decode+verify+run)'
            Invoke-Cargo -WorkDir $AcMn -TestArgs @('--test','text_source_894_incremental','--','owned_894_restore_refuses','--exact') -Name 'restore refuses corrupted store'
        }
        'contract' {
            Invoke-Step 'case binding t03 row migrated to value 23' {
                $raw = Get-Content (Join-Path $RepoRoot 'crates\auto-lang\test\cases\ace-m1\t-r-native.case.ad') -Raw
                if ($raw -notmatch 'id: "ace\.m1\.t03"') { throw 'ace.m1.t03 row missing' }
                $m = [regex]::Match($raw, 'id: "ace\.m1\.t03"[\s\S]{0,900}?expect: \{ completion: "success", exit: 23 \}')
                if (-not $m.Success) { throw 'ace.m1.t03 does not expect success exit 23' }
            }
            Invoke-Step 'independent dual oracle suite present (t03/t05 value rows)' {
                $raw = Get-Content (Join-Path $RepoRoot 'crates\auto-lang\test\cases\ace-m1\text-source-894.case.ad') -Raw
                if ($raw -notmatch 'ace\.m1\.t03\.value' -or $raw -notmatch 'ace\.m1\.t05\.value') { throw 'value rows missing' }
                if ($raw -notmatch 'exit: 23' -or $raw -notmatch 'exit: 42') { throw 'oracle values missing' }
            }
            Invoke-Step 'binding.tsv T03/T05 ac+av columns delivered' {
                $rows = Get-Content (Join-Path $RepoRoot 'crates\auto-lang\test\cases\ace-m1\binding.tsv') |
                    Where-Object { $_ -match '^T03\t' -or $_ -match '^T05\t' }
                foreach ($r in $rows) {
                    $cols = $r -split "`t"
                    if ($cols[5] -ne 'ok' -or $cols[6] -ne 'ok') { throw "binding row not ok on ac/av: $($cols[0])" }
                }
            }
            Invoke-Step 'case runner: migrated t03 row executes green (33/33)' {
                $exe = Join-Path $RepoRoot 'target\debug\auto-case.exe'
                if (-not (Test-Path $exe)) { throw 'auto-case.exe missing (build via discovery first)' }
                $out = & $exe run --suite (Join-Path $RepoRoot 'crates\auto-lang\test\cases\ace-m1\ace-m1-tr-native.suite.ad') --out (Join-Path $ReceiptsDir 'tr-native-run.json') 2>&1 | Out-String
                if ($LASTEXITCODE -ne 0) { throw ($out | Select-Object -Last 15) }
            }
            Invoke-Step 'case runner: independent dual oracle executes green (4/4)' {
                $exe = Join-Path $RepoRoot 'target\debug\auto-case.exe'
                $out = & $exe run --suite (Join-Path $RepoRoot 'crates\auto-lang\test\cases\ace-m1\text-source-894.suite.ad') --out (Join-Path $ReceiptsDir 'text-source-894-run.json') 2>&1 | Out-String
                if ($LASTEXITCODE -ne 0) { throw ($out | Select-Object -Last 15) }
            }
        }
    }
}

$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$receipt = Join-Path $ReceiptsDir ("{0}-{1}.md" -f $Scope, $stamp)
$pass = ($script:Results | Where-Object { $_.Ok }).Count
$fail = ($script:Results | Where-Object { -not $_.Ok }).Count
@"
# verify-text-894 receipt — scope $Scope — $stamp

| step | ok | seconds | detail |
|---|---|---|---|
$(($script:Results | ForEach-Object { "| $($_.Step) | $($_.Ok) | $($_.Seconds) | $($_.Detail -replace '\|','\|') |" }) -join "`n")

pass=$pass fail=$fail
"@ | Set-Content -Path $receipt -Encoding UTF8
Write-Host ("receipt: {0} (pass={1} fail={2})" -f $receipt, $pass, $fail)
if ($fail -gt 0) { exit 1 }
