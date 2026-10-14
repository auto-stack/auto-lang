# PLAN-770 one-shot text/char native gate (ASCII only for Windows
# PowerShell 5.1). Serially runs the frozen gates from plan section 6:
# the ac-core workspace (cargo check --locked --all-targets zero-warning
# health check with an in-script positive/negative matcher control, the
# FULL ac-core test suite — now including text_hir/text_source/
# text_binding/text_native/text_resources — and cargo fmt --check), the
# runtime/text workspace (check/test/fmt, same policy; its 19-test suite
# owns the fault-injection/budget faces the language cannot reach), the
# OLD gates 741/745/767 (the scalar-767 gate covers the post-772 fixed
# computation surface; PLAN-771 has not merged, so no record gate runs),
# and the plan's scope isolation (no changes outside the 770 paths —
# which also proves the strict acc_inventory premise: no ACC binding or
# product crates/test tree was touched). Writes a receipt with HEAD,
# toolchain/runtime-lib/contract/source hashes AND the raw captured
# output of every step — on success AND on failure. Receipts land under
# experimental/ac-core/target/ac-verify-receipts/770/ (gitignored).
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-text-770.ps1
$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$CoreDir = Join-Path $RepoRoot 'experimental\ac-core'
$CoreManifest = Join-Path $CoreDir 'Cargo.toml'
$RtDir = Join-Path $CoreDir 'runtime\text'
$RtManifest = Join-Path $RtDir 'Cargo.toml'
$ReportDir = Join-Path $RepoRoot 'docs\reports\770-text'
$ReceiptsDir = Join-Path $CoreDir 'target\ac-verify-receipts\770'
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
# (mirror of the 774 gate; catches warnings not on the first line).
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
        $script:ReceiptLines.Add("PLAN-770 text gate receipt")
        $script:ReceiptLines.Add("head: $script:Head")
        $script:ReceiptLines.Add("rustc: $($rustc.Trim())")
        $script:ReceiptLines.Add("rust-lld: $lld")
        $script:ReceiptLines.Add("rust-lld-sha256: $(Get-Sha256 $lld)")
        $script:ReceiptLines.Add("raw-outputs: $script:RawDir")
        Write-Host "HEAD $script:Head"
    }

    # --- ac-core workspace ----------------------------------------------------

    Invoke-Step 'cargo check --locked --all-targets (ac-core)' {
        $t = Invoke-Cargo @('check', '--manifest-path', $CoreManifest, '--locked', '--all-targets') 'core-check.txt'
        if (Test-HasWarningLine $t) { throw "zero-warning health check failed (per-line matcher)`n$t" }
        $script:ReceiptLines.Add('ac-core check: ok (zero warnings, per-line matcher with controls)')
    }

    Invoke-Step 'cargo test --locked (ac-core full suite incl. text faces)' {
        $t = Invoke-Cargo @('test', '--manifest-path', $CoreManifest, '--locked') 'core-test.txt'
        $ms = [regex]::Matches($t, 'result: ok\. (\d+) passed')
        $total = 0
        foreach ($m in $ms) { $total += [int]$m.Groups[1].Value }
        if ($total -le 0) { throw "no tests counted; raw output in core-test.txt" }
        # the T-05 faces must be present in the run (binary headers appear
        # per integration test in cargo's output)
        foreach ($bin in @('text_native.rs', 'text_resources.rs', 'text_hir.rs', 'text_source.rs', 'text_binding.rs', 'scalar_native.rs', 'source_native.rs', 'native_execution.rs')) {
            if ($t -notmatch [regex]::Escape($bin)) { throw "test binary $bin missing from the ac-core run" }
        }
        $script:ReceiptLines.Add("ac-core test: ok ($total tests; text_native/text_resources present; every PE under the contained executor)")
    }

    Invoke-Step 'cargo fmt --check (ac-core)' {
        Invoke-Cargo @('fmt', '--manifest-path', $CoreManifest, '--check') 'core-fmt.txt' | Out-Null
        $script:ReceiptLines.Add('ac-core fmt: clean')
    }

    # --- runtime/text workspace ------------------------------------------------

    Invoke-Step 'runtime/text cargo check --locked --all-targets' {
        $t = Invoke-Cargo @('check', '--manifest-path', $RtManifest, '--locked', '--all-targets') 'rt-check.txt'
        if (Test-HasWarningLine $t) { throw "runtime zero-warning check failed`n$t" }
        $script:ReceiptLines.Add('runtime/text check: ok (zero warnings)')
    }

    Invoke-Step 'runtime/text cargo test --locked (runtime-owned fault/budget faces)' {
        $t = Invoke-Cargo @('test', '--manifest-path', $RtManifest, '--locked') 'rt-test.txt'
        $ms = [regex]::Matches($t, 'result: ok\. (\d+) passed')
        $total = 0
        foreach ($m in $ms) { $total += [int]$m.Groups[1].Value }
        if ($total -lt 19) { throw "expected at least the 19 baseline runtime tests, counted $total" }
        $script:ReceiptLines.Add("runtime/text test: ok ($total tests; AP fault injection, budget, trap/hang, contract sidecar)")
    }

    Invoke-Step 'runtime/text cargo fmt --check' {
        Invoke-Cargo @('fmt', '--manifest-path', $RtManifest, '--check') 'rt-fmt.txt' | Out-Null
        $script:ReceiptLines.Add('runtime/text fmt: clean')
    }

    # --- old gates (plan V-10: 741/745/772-surface; 771 not merged) -----------

    # 741 runs with -SkipMainGates: the main-repo cargo t/tv tiers belong to
    # the batch regression gate (/auto-plan:regress), not per-plan review —
    # and this plan touches nothing under crates/ or test/.
    foreach ($gateScript in @('verify-ac-741.ps1', 'verify-ac-source-745.ps1', 'verify-ac-scalar-767.ps1')) {
        # NB: the loop variable must NOT be named $name/$Name — dynamic
        # scoping inside Invoke-Step's scriptblock would resolve it to the
        # step-name parameter instead of this loop variable.
        Invoke-Step "old gate: $gateScript" {
            $gateArgs = @()
            if ($gateScript -eq 'verify-ac-741.ps1') { $gateArgs = @('-SkipMainGates') }
            $prev = $ErrorActionPreference
            $ErrorActionPreference = 'Continue'
            try {
                if ($gateArgs.Count -gt 0) {
                    $out = & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot $gateScript) $gateArgs 2>&1
                } else {
                    $out = & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot $gateScript) 2>&1
                }
                $rc = $LASTEXITCODE
            } finally {
                $ErrorActionPreference = $prev
            }
            $text = ($out | Out-String)
            if ($null -ne $script:RawDir) {
                [System.IO.File]::WriteAllText((Join-Path $script:RawDir ($gateScript -replace '\.ps1$', '.txt')), $text)
            }
            if ($rc -ne 0) { throw "$gateScript exited $rc`n$text" }
            $script:ReceiptLines.Add("old gate ${gateScript}: ok (scalar-767 covers the post-772 fixed surface; 741 main gates skipped per tiering)")
        }
    }

    # --- V-10 scope isolation ---------------------------------------------------

    Invoke-Step 'scope isolation (V-10: no old-surface changes)' {
        $base = (& git -C $RepoRoot merge-base HEAD master).Trim()
        $changed = & git -C $RepoRoot diff --name-only $base HEAD
        $allowed = @(
            '^experimental/ac-core/src/',
            '^experimental/ac-core/tests/',
            '^experimental/ac-core/fixtures/',
            '^experimental/ac-core/schema/',
            '^experimental/ac-core/runtime/text/',
            '^experimental/ac-core/Cargo\.toml$',
            '^experimental/ac-core/Cargo\.lock$',
            '^experimental/ac-core/README\.md$',
            '^scripts/verify-ac-text-770\.ps1$',
            '^docs/reports/770-text/',
            '^docs/plans/770-ac-text-char-native\.md$'
        )
        $illegal = @($changed | Where-Object {
            $p = $_ -replace '\\', '/'
            -not ($allowed | Where-Object { $p -match $_ })
        })
        if ($illegal.Count -gt 0) {
            throw "out-of-scope changes:`n$($illegal -join "`n")"
        }
        $script:ReceiptLines.Add("scope: clean (diff $base..HEAD within ac-core/runtime/script/report paths; product crates/, test/ and every ACC binding untouched - the strict acc_inventory premise)")
    }

    # --- artifact + source contract hashes --------------------------------------

    Invoke-Step 'hashes (runtime lib + contract sidecar + gate inputs)' {
        $lib = Join-Path $RtDir 'target\release\ac_text_runtime.lib'
        $contract = Join-Path $RtDir 'target\release\ac_text_runtime.lib.contract.json'
        $script:ReceiptLines.Add("runtime-lib-sha256: $(Get-Sha256 $lib)")
        $script:ReceiptLines.Add("runtime-contract: $(Get-Sha256 $contract)")
        foreach ($rel in @(
            'src\native.rs', 'src\main.rs', 'src\text_resources.rs',
            'src\source\adapter.rs', 'src\source\mod.rs',
            'tests\text_native.rs', 'tests\text_resources.rs',
            'runtime\text\shim\src\lib.rs', 'runtime\text\src\lib.rs',
            'runtime\text\Cargo.toml', 'runtime\text\Cargo.lock',
            'fixtures\text-hir\valid\n01-native-ops.atom'
        )) {
            $p = Join-Path $CoreDir $rel
            $script:ReceiptLines.Add(("core {0}: {1}" -f ($rel -replace '\\', '/'), (Get-Sha256 $p)))
        }
        $script:ReceiptLines.Add(("gate-script: {0}" -f (Get-Sha256 "$PSScriptRoot\verify-ac-text-770.ps1")))
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
    Write-Host 'PLAN-770 text gate summary' -ForegroundColor Cyan
    $script:Results | Format-Table -AutoSize
    Write-Host "receipt: $receiptPath"
    if ($script:HadFailure) { exit 1 }
    $failed = @($script:Results | Where-Object { -not $_.Ok })
    if ($failed.Count -gt 0) { exit 1 } else { exit 0 }
}
