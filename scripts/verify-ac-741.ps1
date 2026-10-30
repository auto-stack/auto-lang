# PLAN-741 one-shot verification script (ASCII only for Windows PowerShell 5.1).
# Covers plan section 6 commands + the main-repo final gates. Every subprocess
# carries a hard deadline. Receipts land in
# experimental/ace/crates/core/target/ac-verify-receipts/ (gitignored).
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-741.ps1
#        Pass -SkipMainGates to skip cargo t/tv (batch tier belongs to /auto-plan:regress).
param(
    [switch]$SkipMainGates,
    [switch]$OnlyMainGates
)
. (Join-Path $PSScriptRoot 'test-governance/ace-cargo.ps1')

$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$AcCore = Join-Path $RepoRoot 'experimental/ace/crates/core'
$AcMn = Join-Path $RepoRoot 'experimental/ace/crates/native'
$ReceiptsDir = Join-Path $AcCore 'target\ac-verify-receipts'
New-Item -ItemType Directory -Force -Path $ReceiptsDir | Out-Null

$script:Results = New-Object System.Collections.Generic.List[object]

function Invoke-Step {
    param([string]$Name, [scriptblock]$Body, [int]$TimeoutSec = 600)
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

function Invoke-Captured {
    # Run an external command with a hard deadline; non-zero exit throws.
    param([string]$Exe, [string[]]$ArgList, [string]$WorkDir, [int]$TimeoutSec = 600, [int]$ExpectedExit = 0)
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    if ($Exe -eq 'cargo') { $ArgList = Convert-AceCargoArguments -CargoArguments $ArgList -WorkingDirectory $WorkDir }
    $psi.FileName = $Exe
    # Windows PowerShell 5.1 (.NET Framework) has no ProcessStartInfo.ArgumentList
    $psi.Arguments = ($ArgList | ForEach-Object { '"' + ($_ -replace '"', '\\"') + '"' }) -join ' '
    $psi.WorkingDirectory = $WorkDir
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $p = [System.Diagnostics.Process]::Start($psi)
    # Drain both pipes asynchronously FIRST; synchronous ReadToEnd after
    # WaitForExit deadlocks once the child fills the pipe buffer (e.g. a
    # Invoke-AceCargo check with hundreds of warnings).
    $outTask = $p.StandardOutput.ReadToEndAsync()
    $errTask = $p.StandardError.ReadToEndAsync()
    if (-not $p.WaitForExit($TimeoutSec * 1000)) {
        $p.Kill()
        throw "subprocess '$Exe $($ArgList -join ' ')' exceeded ${TimeoutSec}s deadline"
    }
    $out = $outTask.GetAwaiter().GetResult()
    $err = $errTask.GetAwaiter().GetResult()
    if ($p.ExitCode -ne $ExpectedExit) {
        throw "'$Exe $($ArgList -join ' ')' exited $($p.ExitCode) (expected $ExpectedExit): $($err + $out)"
    }
    return ($out + $err)
}

# ---- 1. toolchain discovery (receipt) ----
Invoke-Step 'toolchain discovery' {
    $lines = @()
    $lines += Invoke-Captured 'rustc' @('-Vv') $RepoRoot -TimeoutSec 60
    $lines += Invoke-Captured 'cargo' @('-V') $RepoRoot -TimeoutSec 60
    $lld = Join-Path ((Invoke-Captured 'rustc' @('--print', 'sysroot') $RepoRoot -TimeoutSec 60).Trim()) 'lib\rustlib\x86_64-pc-windows-msvc\bin\rust-lld.exe'
    if (-not (Test-Path $lld)) { throw "rust-lld not found at $lld" }
    $lines += "rust-lld: $lld"
    $sdk = Invoke-Captured 'reg' @('query', 'HKLM\SOFTWARE\Microsoft\Windows Kits\Installed Roots', '/v', 'KitsRoot10') $RepoRoot -TimeoutSec 60
    $lines += $sdk
    $lines | Set-Content (Join-Path $ReceiptsDir 'toolchain.txt')
    Write-Host $lines[0]
}

$CargoManifest = $AcCore  # manifest path by dir

# ---- 2. prototype scoped gates (plan section 6 commands) ----
if (-not $OnlyMainGates) {
Invoke-Step 'cargo check (ac-core)' {
    Invoke-Captured 'cargo' @('check', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml')) $RepoRoot | Out-Null
}
Invoke-Step 'cargo fmt --check (ac-core)' {
    Invoke-Captured 'cargo' @('fmt', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--', '--check') $RepoRoot | Out-Null
}
Invoke-Step 'cargo check --all-targets, zero warnings (ac-core)' {
    # Health check must cover test targets too (QA-07: an unused import in
    # tests/common only shows under --all-targets).
    $out = Invoke-Captured 'cargo' @('check', '--locked', '--all-targets', '--manifest-path', (Join-Path $AcCore 'Cargo.toml')) $RepoRoot
    if ($out -match '(?m)^warning:') { throw "unhandled compiler warnings in ac-core all-targets build:`n$out" }
}
Invoke-Step 'cargo test --lib (ac-core: link executor deadline/pipe/publish matrix)' {
    Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--lib', '--', '--test-threads=1') $RepoRoot -TimeoutSec 300 | Out-Null
}
Invoke-Step 'cargo test text_binding' {
    Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--test', 'text_binding') $RepoRoot | Out-Null
}
Invoke-Step 'cargo test hir_verify' {
    Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--test', 'hir_verify') $RepoRoot | Out-Null
}
# PLAN-871 T-06 mapping (old step → new step): the native_execution / cli /
# trace_execution suites exercised the RETIRED HIR→CLIF lowering and its CLI
# build face. Their positive oracles are covered by:
#   - this script's ac-native walkthrough steps (add-2-3 exit 5, trace exit 12
#     with "ba" marks) — same frozen values, shared-MIR product;
#   - `experimental/ace/crates/native/tests/cases/shadow_tier.rs` (frozen oracles +
#     trace-mark rows) and `tests/cases/gap_closure.rs`.

# ---- 3. CLI build/run walkthrough (own receipts) ----
# PLAN-871 T-05: the `build` legs run through the shared-MIR `ac-native` CLI
# (the receiver of ac-core's retiring build/build-source); `check` stays on
# the ac-core binary (frontend/check role). Old assertion mapping: exit 5
# (add-2-3) and exit 12 + "ba" marks (trace, capability + support lib) are
# UNCHANGED; products become exe + <stem>.ac-link.txt (never asserted here).
$script:AcNative = Join-Path $RepoRoot 'experimental/ace/target/debug/ac-native.exe'
Invoke-Step 'build ac-native CLI once' {
    # Invoke-Captured throws on a non-zero exit; only the artifact remains to
    # be checked here.
    Invoke-Captured 'cargo' @('build', '--locked', '--manifest-path', (Join-Path $AcMn 'Cargo.toml'), '--bin', 'ac-native') $RepoRoot -TimeoutSec 900 | Out-Null
    if (-not (Test-Path $script:AcNative)) { throw "ac-native missing: $script:AcNative" }
}
Invoke-Step 'ac-probe check 01-add (exit 0)' {
    $src = Join-Path $RepoRoot 'docs\design\strategy\hir-examples\01-add.atom'
    Invoke-Captured 'cargo' @('run', '--locked', '--quiet', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--', 'check', $src) $RepoRoot -TimeoutSec 300 | Out-Null
}
Invoke-Step 'ac-probe build add-2-3 + run (exit 5)' {
    $artifactDir = Join-Path $ReceiptsDir 'cli-walkthrough'
    New-Item -ItemType Directory -Force -Path $artifactDir | Out-Null
    $exe = Join-Path $artifactDir 'add-2-3.exe'
    $src = Join-Path $RepoRoot 'experimental/ace/crates/core\fixtures\native\add-2-3.atom'
    Invoke-Captured $script:AcNative @('build', $src, '--entry', 'd_entry', '--output', $exe) $RepoRoot -TimeoutSec 300 | Out-Null
    if (-not (Test-Path $exe)) { throw "exe not produced: $exe" }
    $runOut = Invoke-Captured $exe @() $artifactDir -TimeoutSec 60 -ExpectedExit 5
    $runOut | Set-Content (Join-Path $artifactDir 'add-2-3.run.txt')
}
Invoke-Step 'ac-probe build trace (capability + support lib, exit 12)' {
    $artifactDir = Join-Path $ReceiptsDir 'cli-walkthrough'
    $exe = Join-Path $artifactDir 'trace.exe'
    $src = Join-Path $RepoRoot 'docs\design\strategy\hir-examples\03-call-order.atom'
    $lib = Join-Path $RepoRoot 'experimental/ace/target/runtime-dev/ace_trace_runtime.lib'
    Invoke-Captured $script:AcNative @('build', $src, '--entry', 'd_caller', '--output', $exe, '--capability', 'hir.test.trace.v1', '--support-lib', $lib) $RepoRoot -TimeoutSec 300 | Out-Null
    $procOut = Join-Path $artifactDir 'trace.stderr.txt'
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $exe
    $psi.WorkingDirectory = $artifactDir
    $psi.UseShellExecute = $false
    $psi.RedirectStandardError = $true
    $p = [System.Diagnostics.Process]::Start($psi)
    if (-not $p.WaitForExit(60000)) { $p.Kill($true); throw 'trace exe exceeded 60s deadline' }
    $p.StandardError.ReadToEnd() | Set-Content $procOut
    if ($p.ExitCode -ne 12) { throw "trace exe exited $($p.ExitCode), expected 12" }
    $marks = (Get-Content $procOut -Raw)
    if ($marks -notmatch 'b' -or $marks -notmatch 'a') { throw "trace marks missing: $marks" }
    if ($marks.IndexOf('b') -gt $marks.IndexOf('a')) { throw "trace order wrong: $marks" }
}
}

# ---- 4. main-repo final gates (AGENTS compiler-change tier) ----
if (-not $SkipMainGates) {
    Invoke-Step 'cargo check -p auto-lang' {
        Invoke-Captured 'cargo' @('check', '-p', 'auto-lang') $RepoRoot -TimeoutSec 2400 | Out-Null
    }
    Invoke-Step 'affected module union' {
        . (Join-Path $RepoRoot 'scripts/test-governance/invoke-modules.ps1')
        # Native implementation plus its real language-host consumer, explicitly selected.
        Invoke-FixedTestGroups -Repo $RepoRoot -Groups @('ace-native','ace-host') -Output (Join-Path $ReceiptsDir 'modules')
    }
}

# ---- summary ----
$Results | Format-Table -AutoSize | Tee-Object -FilePath (Join-Path $ReceiptsDir 'summary.txt')
$failed = @($Results | Where-Object { -not $_.Ok })
if ($failed.Count -gt 0) {
    Write-Host ("VERIFY-AC-741: {0}/{1} steps FAILED" -f $failed.Count, $Results.Count) -ForegroundColor Red
    exit 1
}
Write-Host ("VERIFY-AC-741: all {0} steps PASS" -f $Results.Count) -ForegroundColor Green
exit 0
