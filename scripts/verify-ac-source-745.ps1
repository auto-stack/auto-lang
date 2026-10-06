# PLAN-745 one-shot source-profile verification script (ASCII only for
# Windows PowerShell 5.1). Consumes the full corpus case table serially:
# every positive case is compiled from .at source, linked and EXECUTED as a
# real PE (60s hard deadline per run) with exit/stderr oracles; every
# negative case is checked through check-source expecting exit 1 and the
# pinned stage/code marker. Raw receipts land in
# experimental/ac-core/target/ac-verify-receipts/source-745/ (gitignored).
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-source-745.ps1
$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$AcCore = Join-Path $RepoRoot 'experimental\ac-core'
$ReceiptsDir = Join-Path $AcCore 'target\ac-verify-receipts\source-745'
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
    # Run an external command with a hard deadline; returns stdout+stderr
    # and the exit code without throwing.
    param([string]$Exe, [string[]]$ArgList, [string]$WorkDir, [int]$TimeoutSec = 600)
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
    if (-not $p.WaitForExit($TimeoutSec * 1000)) {
        $p.Kill()
        throw "subprocess '$Exe $($ArgList -join ' ')' exceeded ${TimeoutSec}s deadline"
    }
    $out = $outTask.GetAwaiter().GetResult()
    $err = $errTask.GetAwaiter().GetResult()
    return [pscustomobject]@{ Code = $p.ExitCode; Stdout = $out; Stderr = $err }
}

function Get-Sha256 {
    param([string]$Path)
    (Get-FileHash -Algorithm SHA256 $Path).Hash.ToLower()
}

$Profile = 'auto.source.core-i32.draft'
$Trace = 'hir.test.trace.v1'

# Positive corpus: id, fixture, exit, stderr, capability, support-lib.
$Positive = @(
    @{ Id = 'P01'; File = 'p01-precedence.at'; Exit = 14; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P02'; File = 'p02-signed-positional.at'; Exit = 1; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P03'; File = 'p03-bool-param-constant-encoding.at'; Exit = 7; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P04'; File = 'p04-while-retest.at'; Exit = 3; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P05'; File = 'p05-while-zero.at'; Exit = 1; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P06'; File = 'p06-continue-retests-condition.at'; Exit = 3; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P07'; File = 'p07-break-nearest-loop.at'; Exit = 5; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P08'; File = 'p08-recursion.at'; Exit = 3; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P09'; File = 'p09-forward-signature.at'; Exit = 8; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P10'; File = 'p10-shadow-rhs-outer.at'; Exit = 5; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P11'; File = 'p11-literal-min-max.at'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P12'; File = 'p12-unexecuted-trap-not-lifted-x1.at'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P13'; File = 'p13-add-overflow-trap.at'; Exit = 70; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P14'; File = 'p14-mul-overflow-trap.at'; Exit = 70; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P15'; File = 'p15-named-eval-order-x2.at'; Exit = 12; Stderr = 'ba'; Cap = $true; Lib = $true },
    @{ Id = 'P16'; File = 'p16-observable-before-trap-x9.at'; Exit = 70; Stderr = 'a'; Cap = $true; Lib = $true },
    @{ Id = 'P17'; File = 'p17-heterogeneous-binding-map.at'; Exit = 6; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P18'; File = 'p18-bool-return-local-branch.at'; Exit = 4; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P19'; File = 'p19-user-function-not-intrinsic.at'; Exit = 6; Stderr = ''; Cap = $true; Lib = $false },
    @{ Id = 'P20'; File = 'p20-utf8-crlf-negative-min.at'; Exit = 0; Stderr = ''; Cap = $false; Lib = $false },
    @{ Id = 'P21'; File = 'p21-mutual-recursion.at'; Exit = 4; Stderr = ''; Cap = $false; Lib = $false }
)

# Negative corpus: id, fixture, stage, code.
$Negative = @(
    @{ Id = 'N01'; File = 'n01-literal-range-max.at'; Stage = 'text'; Code = 'source.literal-range' },
    @{ Id = 'N02'; File = 'n02-literal-range-min.at'; Stage = 'text'; Code = 'source.literal-range' },
    @{ Id = 'N03'; File = 'n03-undefined-name.at'; Stage = 'bind'; Code = 'source.undefined' },
    @{ Id = 'N04'; File = 'n04-type-mismatch-return.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'N05'; File = 'n05-type-mismatch-if-cond.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'N06'; File = 'n06-type-mismatch-bool-add.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'N07'; File = 'n07-immutable-write.at'; Stage = 'verify'; Code = 'verify.immutable-write' },
    @{ Id = 'N08'; File = 'n08-duplicate-local.at'; Stage = 'bind'; Code = 'source.duplicate' },
    @{ Id = 'N09'; File = 'n09-undefined-scope-escape.at'; Stage = 'bind'; Code = 'source.undefined' },
    @{ Id = 'N10'; File = 'n10-arg-missing.at'; Stage = 'bind'; Code = 'source.argument' },
    @{ Id = 'N11'; File = 'n11-arg-duplicate.at'; Stage = 'bind'; Code = 'source.argument' },
    @{ Id = 'N12'; File = 'n12-arg-unknown-name.at'; Stage = 'bind'; Code = 'source.argument' },
    @{ Id = 'N13'; File = 'n13-arg-mixed.at'; Stage = 'bind'; Code = 'source.argument' },
    @{ Id = 'N14'; File = 'n14-type-mismatch-arg.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'N15'; File = 'n15-break-outside-loop.at'; Stage = 'verify'; Code = 'verify.loop-target-scope' },
    @{ Id = 'N16'; File = 'n16-continue-outside-loop.at'; Stage = 'verify'; Code = 'verify.loop-target-scope' },
    @{ Id = 'N17'; File = 'n17-no-return-path.at'; Stage = 'verify'; Code = 'verify.no-return-path' },
    @{ Id = 'N18'; File = 'n18-stmt-after-return.at'; Stage = 'verify'; Code = 'verify.block-structure' },
    @{ Id = 'N19'; File = 'n19-var-uninitialized.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N20'; File = 'n20-char-literal.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N21'; File = 'n21-str-type.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N22'; File = 'n22-i64-type.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N23'; File = 'n23-sub-op.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N24'; File = 'n24-div-op.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N25'; File = 'n25-and-op.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N26'; File = 'n26-checked-marker.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N27'; File = 'n27-not-callable-local.at'; Stage = 'bind'; Code = 'source.not-callable' },
    @{ Id = 'N28'; File = 'n28-not-callable-param.at'; Stage = 'bind'; Code = 'source.not-callable' },
    @{ Id = 'N29'; File = 'n29-not-callable-shadow.at'; Stage = 'bind'; Code = 'source.not-callable' },
    @{ Id = 'N30'; File = 'n30-eof-block.at'; Stage = 'text'; Code = 'source.syntax' },
    @{ Id = 'N31'; File = 'n31-lex-at.at'; Stage = 'text'; Code = 'source.lex' },
    @{ Id = 'N32'; File = 'n32-duplicate-fn.at'; Stage = 'bind'; Code = 'source.duplicate' },
    @{ Id = 'N33'; File = 'n33-duplicate-param.at'; Stage = 'bind'; Code = 'source.duplicate' },
    @{ Id = 'N34'; File = 'n34-mod-op.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N35'; File = 'n35-eq-op.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N36'; File = 'n36-unary-minus-ident.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N37'; File = 'n37-suffixed-literal.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N38'; File = 'n38-hex-literal.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N39'; File = 'n39-toplevel-use.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N40'; File = 'n40-toplevel-let.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N41'; File = 'n41-call-statement.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N42'; File = 'n42-missing-ret-ann.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N43'; File = 'n43-list-type.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N44'; File = 'n44-toplevel-type.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N45'; File = 'n45-toplevel-enum.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N46'; File = 'n46-for-loop.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N47'; File = 'n47-float-literal.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N48'; File = 'n48-block-comment.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N49'; File = 'n49-unicode-ident.at'; Stage = 'text'; Code = 'source.unsupported' },
    @{ Id = 'N50'; File = 'n50-undefined-callee.at'; Stage = 'bind'; Code = 'source.undefined' },
    @{ Id = 'N51'; File = 'n51-type-mismatch-lt-bool.at'; Stage = 'verify'; Code = 'source.type-mismatch' },
    @{ Id = 'N52'; File = 'n52-tailing-garbage.at'; Stage = 'text'; Code = 'source.syntax' },
    @{ Id = 'N53'; File = 'n53-empty-file.at'; Stage = 'text'; Code = 'source.syntax' }
)

# ---- 1. context receipt ----
Invoke-Step 'context (HEAD + toolchain)' {
    $lines = @()
    $lines += 'git-head: ' + (Invoke-Captured 'git' @('rev-parse', 'HEAD') $RepoRoot -TimeoutSec 60).Stdout.Trim()
    $lines += 'git-branch: ' + (Invoke-Captured 'git' @('branch', '--show-current') $RepoRoot -TimeoutSec 60).Stdout.Trim()
    $status = (Invoke-Captured 'git' @('status', '--short') $RepoRoot -TimeoutSec 60).Stdout.Trim()
    $lines += 'git-status-short-lines: ' + (@($status -split "`n").Count)
    $lines += (Invoke-Captured 'rustc' @('-Vv') $RepoRoot -TimeoutSec 60).Stdout
    $lines += (Invoke-Captured 'cargo' @('-V') $RepoRoot -TimeoutSec 60).Stdout
    $lines | Set-Content (Join-Path $ReceiptsDir 'context.txt')
}

# ---- 2. ac-core scoped gates ----
Invoke-Step 'cargo check (ac-core)' {
    $r = Invoke-Captured 'cargo' @('check', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml')) $RepoRoot
    if ($r.Code -ne 0) { throw "cargo check exited $($r.Code): $($r.Stderr)" }
}
Invoke-Step 'cargo fmt --check (ac-core)' {
    $r = Invoke-Captured 'cargo' @('fmt', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--', '--check') $RepoRoot
    if ($r.Code -ne 0) { throw "fmt --check failed: $($r.Stdout)$($r.Stderr)" }
}
Invoke-Step 'cargo check --all-targets, zero warnings (ac-core)' {
    $r = Invoke-Captured 'cargo' @('check', '--locked', '--all-targets', '--manifest-path', (Join-Path $AcCore 'Cargo.toml')) $RepoRoot
    if ($r.Code -ne 0) { throw "check --all-targets exited $($r.Code): $($r.Stderr)" }
    if (($r.Stdout + $r.Stderr) -match '(?m)^warning:') { throw "unhandled compiler warnings in ac-core all-targets build" }
}
Invoke-Step 'cargo test source_frontend' {
    $r = Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--test', 'source_frontend') $RepoRoot -TimeoutSec 300
    if ($r.Code -ne 0) { throw $r.Stderr }
}
Invoke-Step 'cargo test source_hir' {
    $r = Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--test', 'source_hir') $RepoRoot -TimeoutSec 300
    if ($r.Code -ne 0) { throw $r.Stderr }
}
Invoke-Step 'cargo test source_native (--test-threads=1)' {
    $r = Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--test', 'source_native', '--', '--test-threads=1') $RepoRoot -TimeoutSec 900
    if ($r.Code -ne 0) { throw $r.Stderr }
}
Invoke-Step 'cargo test source_cli (--test-threads=1)' {
    $r = Invoke-Captured 'cargo' @('test', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml'), '--test', 'source_cli', '--', '--test-threads=1') $RepoRoot -TimeoutSec 900
    if ($r.Code -ne 0) { throw $r.Stderr }
}

# ---- 3. serial case-table walkthrough ----
Invoke-Step 'build tool binary once' {
    $r = Invoke-Captured 'cargo' @('build', '--locked', '--manifest-path', (Join-Path $AcCore 'Cargo.toml')) $RepoRoot -TimeoutSec 600
    if ($r.Code -ne 0) { throw $r.Stderr }
    $script:Bin = Join-Path $AcCore 'target\debug\auto-ac-prototype.exe'
    if (-not (Test-Path $script:Bin)) { throw "binary missing: $script:Bin" }
    ("binary: " + $script:Bin) | Add-Content (Join-Path $ReceiptsDir 'context.txt')
    ("binary-sha256: " + (Get-Sha256 $script:Bin)) | Add-Content (Join-Path $ReceiptsDir 'context.txt')
}

Invoke-Step 'build trace support library once' {
    $r = Invoke-Captured 'cargo' @('build', '--locked') (Join-Path $AcCore 'test-support') -TimeoutSec 600
    if ($r.Code -ne 0) { throw $r.Stderr }
    $script:SupportLib = Join-Path $AcCore 'test-support\target\debug\ac_trace_support.lib'
    if (-not (Test-Path $script:SupportLib)) { throw "support lib missing: $script:SupportLib" }
    ("support-lib-sha256: " + (Get-Sha256 $script:SupportLib)) | Add-Content (Join-Path $ReceiptsDir 'context.txt')
}

$caseSummary = New-Object System.Collections.Generic.List[object]

Invoke-Step 'positive cases: build + run (21)' {
    foreach ($c in $Positive) {
        $dir = Join-Path $ReceiptsDir ('positive\' + $c.Id)
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $src = Join-Path $RepoRoot ("experimental\ac-core\fixtures\source\valid\" + $c.File)
        ("input: " + $src) | Set-Content (Join-Path $dir 'case.txt')
        ("input-sha256: " + (Get-Sha256 $src)) | Add-Content (Join-Path $dir 'case.txt')
        $exe = Join-Path $dir 'app.exe'
        $args = @('build-source', $src, '--source-profile', $Profile, '--entry', 'main', '--output', $exe)
        if ($c.Cap) { $args += @('--capability', $Trace) }
        if ($c.Lib) { $args += @('--support-lib', $script:SupportLib) }
        $build = Invoke-Captured $script:Bin $args $RepoRoot -TimeoutSec 120
        $build.Stdout | Set-Content (Join-Path $dir 'build.stdout.txt')
        $build.Stderr | Set-Content (Join-Path $dir 'build.stderr.txt')
        if ($build.Code -ne 0) { throw "$($c.Id): build exited $($build.Code)" }
        if (-not (Test-Path $exe)) { throw "$($c.Id): exe missing" }
        $run = Invoke-Captured $exe @() $dir -TimeoutSec 60
        $run.Stdout | Set-Content (Join-Path $dir 'run.stdout.txt')
        $run.Stderr | Set-Content (Join-Path $dir 'run.stderr.txt')
        ("exit: " + $run.Code) | Add-Content (Join-Path $dir 'case.txt')
        ("exe-sha256: " + (Get-Sha256 $exe)) | Add-Content (Join-Path $dir 'case.txt')
        if (-not (Test-Path (Join-Path $dir 'app.obj'))) { throw "$($c.Id): obj missing" }
        if (-not (Test-Path (Join-Path $dir 'app.ac-link.txt'))) { throw "$($c.Id): receipt missing" }
        if ($run.Code -ne $c.Exit) { throw "$($c.Id): run exited $($run.Code), expected $($c.Exit)" }
        if ($run.Stderr -ne $c.Stderr) { throw "$($c.Id): stderr '$($run.Stderr)' expected '$($c.Stderr)'" }
        $caseSummary.Add([pscustomobject]@{ Case = $c.Id; Exit = $run.Code; Stderr = $run.Stderr; Ok = $true })
    }
}

Invoke-Step 'negative cases: check-source rejects (53)' {
    foreach ($c in $Negative) {
        $dir = Join-Path $ReceiptsDir ('negative\' + $c.Id)
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $src = Join-Path $RepoRoot ("experimental\ac-core\fixtures\source\invalid\" + $c.File)
        ("input: " + $src) | Set-Content (Join-Path $dir 'case.txt')
        ("input-sha256: " + (Get-Sha256 $src)) | Add-Content (Join-Path $dir 'case.txt')
        $extra = @()
        if ($c.Id -in @('N28', 'N29')) { $extra = @('--capability', $Trace) }
        $r = Invoke-Captured $script:Bin (@('check-source', $src, '--source-profile', $Profile) + $extra) $RepoRoot -TimeoutSec 60
        $r.Stdout | Set-Content (Join-Path $dir 'stdout.txt')
        $r.Stderr | Set-Content (Join-Path $dir 'stderr.txt')
        ("exit: " + $r.Code) | Add-Content (Join-Path $dir 'case.txt')
        if ($r.Code -ne 1) { throw "$($c.Id): exited $($r.Code), expected 1" }
        $marker = "error[$($c.Stage)/$($c.Code)]"
        if ($r.Stderr -notmatch [regex]::Escape($marker)) { throw "$($c.Id): marker '$marker' missing in: $($r.Stderr)" }
        $caseSummary.Add([pscustomobject]@{ Case = $c.Id; Exit = $r.Code; Stderr = ''; Ok = $true })
    }
}

# ---- 4. summary ----
$caseSummary | Format-Table -AutoSize | Tee-Object -FilePath (Join-Path $ReceiptsDir 'cases.txt')
$Results | Format-Table -AutoSize | Tee-Object -FilePath (Join-Path $ReceiptsDir 'summary.txt')
$failed = @($Results | Where-Object { -not $_.Ok })
if ($failed.Count -gt 0) {
    Write-Host ("VERIFY-AC-SOURCE-745: {0}/{1} steps FAILED" -f $failed.Count, $Results.Count) -ForegroundColor Red
    exit 1
}
Write-Host ("VERIFY-AC-SOURCE-745: all {0} steps PASS ({1} cases)" -f $Results.Count, $caseSummary.Count) -ForegroundColor Green
exit 0
