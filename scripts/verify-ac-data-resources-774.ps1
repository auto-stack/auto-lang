# PLAN-774 one-shot data/resource probe gate (ASCII only for Windows
# PowerShell 5.1). Serially runs the frozen probe gates from plan section 6:
# cargo check --locked --all-targets (zero-warning health check with an
# in-script positive/negative matcher control), the full probe test suite
# (V-02 ABI, V-03 lifetime incl. LIFO/nested exits, V-04 front-door
# rejections + R2 elimination evidence, V-05 fault injection, V-06
# trap/timeout classification with an observable predecessor trace, V-07
# text boundaries, V-08 scale/budget incl. capacity/allowance-restore/zero
# budget), and cargo fmt --check. Every PE child inside the tests runs under
# the REAL ac-core contained deadline executor (60s per subprocess; the shim
# build also runs locked under the same 60s executor). Also asserts the plan
# V-09 scope isolation and writes a receipt with HEAD, tool/source/artifact
# hashes AND the raw captured output of every step — on success AND on
# failure (review r1 P774-R1-06). Receipts land under
# experimental/ac-core/probes/data-resources/target/ac-verify-receipts/774/
# (gitignored).
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-data-resources-774.ps1
$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$ProbeDir = Join-Path $RepoRoot 'experimental\ac-core\probes\data-resources'
$ProbeManifest = Join-Path $ProbeDir 'Cargo.toml'
$ReportDir = Join-Path $RepoRoot 'docs\reports\774-data-resource'
$ReceiptsDir = Join-Path $ProbeDir 'target\ac-verify-receipts\774'
New-Item -ItemType Directory -Force -Path $ReceiptsDir | Out-Null

$script:Results = New-Object System.Collections.Generic.List[object]
$script:ReceiptLines = New-Object System.Collections.Generic.List[string]
$script:RawDir = $null

function Invoke-Step {
    param([string]$Name, [scriptblock]$Body)
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        & $Body
        $script:Results.Add([pscustomobject]@{ Step = $Name; Ok = $true; Seconds = [math]::Round($sw.Elapsed.TotalSeconds, 1); Detail = '' })
        Write-Host ("[PASS] {0} ({1}s)" -f $Name, [math]::Round($sw.Elapsed.TotalSeconds, 1)) -ForegroundColor Green
    } catch {
        $script:Results.Add([pscustomobject]@{ Step = $Name; Ok = $false; Seconds = [math]::Round($sw.Elapsed.TotalSeconds, 1); Detail = $_.Exception.Message })
        Write-Host ("[FAIL] {0} ({1}s): {2}" -f $Name, [math]::Round($sw.Elapsed.TotalSeconds, 1), $_.Exception.Message) -ForegroundColor Red
        throw
    }
}

function Invoke-Cargo {
    param([string[]]$ArgList, [string]$RawName)
    # cargo writes progress ("Checking ...") to stderr; PS 5.1 turns those
    # lines into error records that would throw under Stop. Degrade locally.
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $output = & cargo @ArgList 2>&1
    } finally {
        $ErrorActionPreference = $prev
    }
    $text = ($output | Out-String)
    if ($null -ne $RawName -and $null -ne $script:RawDir) {
        [System.IO.File]::WriteAllText((Join-Path $script:RawDir $RawName), $text)
    }
    if ($LASTEXITCODE -ne 0) {
        throw "cargo $($ArgList -join ' ') exited $LASTEXITCODE`n$text"
    }
    return $text
}

# Per-line warning matcher with an in-script positive AND negative control
# (review r1 P774-R1-06: the old single-string '^warning' regex missed
# warnings that were not on the first line).
function Test-HasWarningLine {
    param([string]$Text)
    foreach ($line in ($Text -split "`r?`n")) {
        if ($line -match '^\s*warning') { return $true }
    }
    return $false
}

function Get-Sha256 {
    param([string]$Path)
    if (-not (Test-Path $Path)) { return '<missing>' }
    $h = [System.Security.Cryptography.SHA256]::Create()
    try {
        $fs = [System.IO.File]::OpenRead($Path)
        try {
            $hash = $h.ComputeHash($fs)
            return ([System.BitConverter]::ToString($hash) -replace '-', '').ToLowerInvariant()
        } finally { $fs.Dispose() }
    } finally { $h.Dispose() }
}

$script:Head = ''
$script:HadFailure = $false

try {
    # --- context + receipt header + raw dir ---------------------------------

    Invoke-Step 'warning matcher controls (positive/negative)' {
        $positive = "   Compiling x`n    warning: unused variable y`n    Finished"
        if (-not (Test-HasWarningLine $positive)) { throw 'positive control failed: second-line warning not detected' }
        $negative = "   Compiling x`n    Finished dev profile"
        if (Test-HasWarningLine $negative) { throw 'negative control failed: clean output flagged' }
    }

    Invoke-Step 'context (HEAD + toolchain)' {
        $script:Head = (& git -C $RepoRoot rev-parse HEAD).Trim()
        $rustc = (& rustc --version) 2>&1 | Out-String
        $lld = Join-Path ((& rustc --print sysroot).Trim()) 'lib\rustlib\x86_64-pc-windows-msvc\bin\rust-lld.exe'
        $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
        $script:RawDir = Join-Path $ReceiptsDir "raw-$stamp"
        New-Item -ItemType Directory -Force -Path $script:RawDir | Out-Null
        $script:ReceiptLines.Add("PLAN-774 probe gate receipt")
        $script:ReceiptLines.Add("head: $script:Head")
        $script:ReceiptLines.Add("rustc: $($rustc.Trim())")
        $script:ReceiptLines.Add("rust-lld: $lld")
        $script:ReceiptLines.Add("rust-lld-sha256: $(Get-Sha256 $lld)")
        $script:ReceiptLines.Add("raw-outputs: $script:RawDir")
        Write-Host "HEAD $script:Head"
    }

    # --- probe gates (plan section 6) ---------------------------------------

    Invoke-Step 'cargo check --locked --all-targets (probe)' {
        $t = Invoke-Cargo @('check', '--manifest-path', $ProbeManifest, '--locked', '--all-targets') 'check.txt'
        if (Test-HasWarningLine $t) { throw "zero-warning health check failed (per-line matcher)`n$t" }
        $script:ReceiptLines.Add('check: ok (zero warnings, per-line matcher with controls)')
    }

    Invoke-Step 'cargo test --locked (probe: V02..V08 native cases)' {
        $t = Invoke-Cargo @('test', '--manifest-path', $ProbeManifest, '--locked') 'test.txt'
        $ms = [regex]::Matches($t, 'result: ok\. (\d+) passed')
        $total = 0
        foreach ($m in $ms) { $total += [int]$m.Groups[1].Value }
        if ($total -le 0) { throw "no probe tests counted; output saved to test.txt" }
        $script:ReceiptLines.Add("test: ok ($total probe tests; every PE under the contained 60s executor)")
    }

    Invoke-Step 'cargo fmt --check (probe)' {
        Invoke-Cargo @('fmt', '--manifest-path', $ProbeManifest, '--check') 'fmt.txt' | Out-Null
        $script:ReceiptLines.Add('fmt: clean')
    }

    # --- V-09 scope isolation -------------------------------------------------

    Invoke-Step 'scope isolation (V-09: no old-surface changes)' {
        $base = (& git -C $RepoRoot merge-base HEAD master).Trim()
        $changed = & git -C $RepoRoot diff --name-only $base HEAD
        $allowed = @(
            '^experimental/ac-core/probes/data-resources/',
            '^scripts/verify-ac-data-resources-774\.ps1$',
            '^docs/reports/774-data-resource/'
        )
        $illegal = @($changed | Where-Object {
            $p = $_ -replace '\\', '/'
            -not ($allowed | Where-Object { $p -match $_ })
        })
        if ($illegal.Count -gt 0) {
            throw "out-of-scope changes:`n$($illegal -join "`n")"
        }
        $script:ReceiptLines.Add("scope: clean (diff $base..HEAD within probe/script/report paths)")
    }

    # --- artifact + source contract hashes ------------------------------------

    Invoke-Step 'hashes (shim artifact + source contracts + gate inputs)' {
        $shim = Join-Path $ProbeDir 'target\release\ac_data_shim_774.lib'
        $script:ReceiptLines.Add("shim-sha256: $(Get-Sha256 $shim)")
        foreach ($rel in @(
            'src\lib.rs', 'src\rules.rs', 'shim\src\lib.rs',
            'Cargo.toml', 'Cargo.lock'
        )) {
            $p = Join-Path $ProbeDir $rel
            $script:ReceiptLines.Add(("probe {0}: {1}" -f ($rel -replace '\\', '/'), (Get-Sha256 $p)))
        }
        foreach ($rel in @(
            'decisions.md', 'probe-cases.md', 'interface-contract.md', 'verification.md'
        )) {
            $p = Join-Path $ReportDir $rel
            $script:ReceiptLines.Add(("report {0}: {1}" -f $rel, (Get-Sha256 $p)))
        }
        $script:ReceiptLines.Add(("gate-script: {0}" -f (Get-Sha256 "$PSScriptRoot\verify-ac-data-resources-774.ps1")))
    }

} catch {
    $script:HadFailure = $true
    $script:ReceiptLines.Add("gate: FAILED - see step table and raw outputs")
} finally {
    $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
    $receiptPath = Join-Path $ReceiptsDir "gate-$stamp.txt"
    if ($null -ne $script:RawDir) {
        $script:ReceiptLines.Add("generated: $stamp")
    }
    [System.IO.File]::WriteAllLines($receiptPath, $script:ReceiptLines)
    Write-Host ''
    Write-Host 'PLAN-774 probe gate summary' -ForegroundColor Cyan
    $script:Results | Format-Table -AutoSize
    Write-Host "receipt: $receiptPath"
    if ($script:HadFailure) { exit 1 }
    $failed = @($script:Results | Where-Object { -not $_.Ok })
    if ($failed.Count -gt 0) { exit 1 } else { exit 0 }
}
