# PLAN-871 T-06 G-DELETE verification (scripts/verify-ac-871-retire.ps1).
#
# Proves the retirement STATE, not just "tests pass":
#   - the old HIR→CLIF lowering module and its exclusive helpers are GONE;
#   - no source file references the retired symbols (grep gate);
#   - ac-core's dependency graph no longer contains the Cranelift stack
#     (`cargo tree`), i.e. the old lowering cannot come back by dependency;
#   - the three workspaces still `cargo check --locked` (compile gate);
#   - the evidence files (old lowering / old reference modules) are absent,
#     including from the ac-native/worker paths.
# Scope=retire (default). The deletion gate is a compile-time gate as well:
# any reintroduced `native::lower_object`/`find_entry` call cannot compile
# because the module no longer exists (demonstrated by the T-06 build).
#
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-871-retire.ps1

param([string]$Scope = 'retire')

$ErrorActionPreference = 'Stop'
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$AcCore = Join-Path $RepoRoot 'experimental\ac-core'
$AcMn = Join-Path $RepoRoot 'experimental\ac-mir-native'
$Worker = Join-Path $RepoRoot 'experimental\ac-case-worker'
$ReceiptsDir = Join-Path $AcCore 'target\ac-verify-receipts\retire-871'
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

function Invoke-Captured {
    param([string]$Exe, [string[]]$ArgList, [string]$WorkDir, [int]$TimeoutSec = 1800)
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $Exe
    $psi.Arguments = ($ArgList | ForEach-Object { '"' + ($_ -replace '"', '\\"') + '"' }) -join ' '
    $psi.WorkingDirectory = $WorkDir
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $p = [System.Diagnostics.Process]::Start($psi)
    $outTask = $p.StandardOutput.ReadToEndAsync()
    $errTask = $p.StandardError.ReadToEndAsync()
    if (-not $p.WaitForExit($TimeoutSec * 1000)) { $p.Kill($true); throw "deadline" }
    $out = $outTask.GetAwaiter().GetResult()
    $err = $errTask.GetAwaiter().GetResult()
    return [pscustomobject]@{ Code = $p.ExitCode; Stdout = $out; Stderr = $err }
}

# Rust sources under the experimental/crates trees (target dirs excluded).
function Rust-Files {
    Get-ChildItem (Join-Path $RepoRoot 'experimental'), (Join-Path $RepoRoot 'crates') -Recurse -File -Filter *.rs |
        Where-Object { $_.FullName -notmatch '\\target\\' }
}

try {
    if ($Scope -ne 'retire') { throw "unknown -Scope '$Scope' (expected retire)" }

    Invoke-Step 'deleted: old lowering and reference modules are absent' {
        $gone = @(
            'experimental\ac-core\src\native.rs',
            'experimental\ac-av-probe\probe\src\native_reference.rs',
            'experimental\ac-av-probe\probe\src\native_admission.rs',
            'experimental\ac-artifact-probe\src\driver.rs',
            'experimental\ac-artifact-probe\src\perf.rs',
            'experimental\ac-artifact-probe\src\bin\node.rs'
        )
        foreach ($rel in $gone) {
            if (Test-Path (Join-Path $RepoRoot $rel)) { throw "retired file still present: $rel" }
        }
    }

    Invoke-Step 'grep: no source references the retired symbols' {
        $bad = @()
        foreach ($pat in @('native::lower_object', 'native::find_entry', 'native::capabilities_check',
                           'native::START_SYMBOL', 'native_reference::', 'native_admission::')) {
            $hits = Rust-Files | Select-String -Pattern ([regex]::Escape($pat))
            foreach ($h in $hits) { $bad += ("{0}:{1}: {2}" -f $h.Path, $h.LineNumber, $h.Line.Trim()) }
        }
        # retirement-marker assertions legitimately NAME the old modules.
        $bad = $bad | Where-Object { $_ -notmatch 'retired_after_871|retiring `native' }
        if ($bad.Count -gt 0) { throw ("retired symbols still referenced:`n" + ($bad -join "`n")) }
    }

    Invoke-Step 'cargo tree: ac-core has no Cranelift stack' {
        $r = Invoke-Captured 'cargo' @('tree', '--locked') $AcCore 900
        if ($r.Code -ne 0) { throw $r.Stderr }
        $t = $r.Stdout + $r.Stderr
        if ($t -match 'cranelift|target-lexicon') { throw "cranelift still in the ac-core dependency graph" }
    }

    Invoke-Step 'cargo check --locked (ac-core)' {
        $r = Invoke-Captured 'cargo' @('check', '--locked', '--all-targets') $AcCore 1800
        if ($r.Code -ne 0) { throw $r.Stderr }
        if (($r.Stdout + $r.Stderr) -match '(?m)^warning:') { throw "unhandled warnings in ac-core" }
    }
    Invoke-Step 'cargo check --locked (ac-mir-native)' {
        $r = Invoke-Captured 'cargo' @('check', '--locked', '--all-targets') $AcMn 1800
        if ($r.Code -ne 0) { throw $r.Stderr }
    }
    Invoke-Step 'cargo check --locked (ac-case-worker)' {
        $r = Invoke-Captured 'cargo' @('check', '--locked', '--all-targets') $Worker 1800
        if ($r.Code -ne 0) { throw $r.Stderr }
    }

    Invoke-Step 'retention: verify / TextResourcePlan / link / source remain' {
        foreach ($rel in @('experimental\ac-core\src\verify.rs',
                           'experimental\ac-core\src\text_resources.rs',
                           'experimental\ac-core\src\link.rs',
                           'experimental\ac-core\src\source\adapter.rs')) {
            if (-not (Test-Path (Join-Path $RepoRoot $rel))) { throw "retained file missing: $rel" }
        }
        $vr = Get-Content (Join-Path $RepoRoot 'experimental\ac-core\src\verify.rs') -Raw
        if ($vr -notmatch 'pub fn capabilities_check') { throw 'capabilities_check did not move into verify.rs' }
        $tr = Get-Content (Join-Path $RepoRoot 'experimental\ac-core\src\text_resources.rs') -Raw
        if ($tr -notmatch 'TextResourcePlan') { throw 'TextResourcePlan analysis missing' }
    }

    Invoke-Step 'counter-control: a reintroduced old symbol cannot compile' {
        # The strongest form of the deletion gate: transplant a fragment that
        # calls the retired module into a scratch copy of the ac-core bin and
        # require the COMPILER to refuse it.
        $probe = Join-Path $ReceiptsDir 'counter-control.rs'
        @'
fn _reintroduced_old_lowering(checked: &auto_ac_prototype::verify::CheckedModule) {
    let sel = auto_ac_prototype::native::find_entry(checked, "d_main").unwrap();
    let _ = auto_ac_prototype::native::lower_object(
        checked,
        &sel,
        &auto_ac_prototype::native::RuntimeInjection::default(),
    );
}
'@ | Set-Content $probe -Encoding utf8
        Copy-Item (Join-Path $AcCore 'src\main.rs') (Join-Path $AcCore 'src\main.rs.bak871')
        try {
            Add-Content -Path (Join-Path $AcCore 'src\main.rs') -Value (Get-Content $probe -Raw)
            $r = Invoke-Captured 'cargo' @('check', '--locked', '--bin', 'auto-ac-prototype') $AcCore 900
            if ($r.Code -eq 0) { throw 'counter-control: the retired symbol still compiles' }
            if (($r.Stdout + $r.Stderr) -notmatch 'native') { throw 'counter-control failed for an unrelated reason' }
        } finally {
            Move-Item -Force (Join-Path $AcCore 'src\main.rs.bak871') (Join-Path $AcCore 'src\main.rs')
        }
        # Restored tree must compile again.
        $r2 = Invoke-Captured 'cargo' @('check', '--locked', '--bin', 'auto-ac-prototype') $AcCore 900
        if ($r2.Code -ne 0) { throw "restore after counter-control failed: $($r2.Stderr)" }
    }
} catch {
    $script:Results.Add([pscustomobject]@{ Step = 'harness'; Ok = $false; Seconds = 0; Detail = $_.Exception.Message })
}

$ReceiptsDir | ForEach-Object { } | Out-Null
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$script:Results | Format-Table -AutoSize | Tee-Object -FilePath (Join-Path $ReceiptsDir "summary-$stamp.txt")
$failed = @($script:Results | Where-Object { -not $_.Ok })
if ($failed.Count -gt 0) {
    Write-Host ("VERIFY-AC-871-RETIRE: {0}/{1} steps FAILED" -f $failed.Count, $script:Results.Count) -ForegroundColor Red
    exit 1
}
Write-Host ("VERIFY-AC-871-RETIRE: all {0} steps PASS" -f $script:Results.Count) -ForegroundColor Green
exit 0
