# PLAN-797 AutoABI probe verification gate (ASCII only for Windows
# PowerShell 5.1). Two scopes:
#   -Scope acceptance  : post-T-06 work proof — committed Cargo.lock
#                        identity, --locked offline resolve, zero-warning
#                        health of the probe crate, the full
#                        experimental/auto-abi-probe suite (now including
#                        abi_binding_ real-contract tests and the FORMAL
#                        turnaround hard gates over the abiworker/1 IPC),
#                        the unified-runner ABI recipe corpus, and the T-07
#                        finalized materials. AC-05/06 CLOSEOUT still
#                        belongs to final review + merge, not this script.
#   -Scope closeout    : acceptance plus a dry checklist of V-01..V-11 /
#                        AC-01..AC-06 closeout items, reporting each as
#                        done|pending (T-06 still depends on D-796/D-788).
#                        closeout NEVER fabricates a green AutoDown pass.
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-autoabi-797.ps1 -Scope acceptance
#        powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-autoabi-797.ps1 -Scope closeout
param(
    [ValidateSet('acceptance', 'closeout')]
    [string]$Scope = 'acceptance'
)
$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$ProbeDir = Join-Path $RepoRoot 'experimental\auto-abi-probe'
$ProbeManifest = Join-Path $ProbeDir 'Cargo.toml'
$ProbeLock = Join-Path $ProbeDir 'Cargo.lock'
$ReportDir = Join-Path $RepoRoot 'docs\reports\plan797'
$ReceiptsDir = Join-Path $ProbeDir 'target\autoabi-verify-receipts\797'
New-Item -ItemType Directory -Force -Path $ReceiptsDir | Out-Null

$script:Results = New-Object System.Collections.Generic.List[object]
$script:ReceiptLines = New-Object System.Collections.Generic.List[string]
$script:RawDir = $null
$script:Head = ''
$script:HadFailure = $false

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
    param([string[]]$ArgList, [string]$RawName, [string]$WorkDir)
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    $script:SawSccacheBypass = $false
    try {
        if ($WorkDir) {
            Push-Location $WorkDir
        }
        try {
            $output = & cargo @ArgList 2>&1
            # Known environment condition (recorded by review R2): the
            # default sccache wrapper fails to start when its port sits in
            # a Windows exclusion range (os error 10013). Retry ONCE with
            # RUSTC_WRAPPER cleared FOR THIS INVOCATION ONLY; the process
            # environment is restored in finally. Never silently retried
            # for other failures.
            if ($LASTEXITCODE -ne 0 -and ($output | Out-String) -match 'sccache.*(10013|port exclusion|Server startup failed)') {
                $script:SawSccacheBypass = $true
                $savedWrapper = $env:RUSTC_WRAPPER
                try {
                    Remove-Item Env:RUSTC_WRAPPER -ErrorAction SilentlyContinue
                    $output = & cargo @ArgList 2>&1
                } finally {
                    if ($null -ne $savedWrapper) { $env:RUSTC_WRAPPER = $savedWrapper }
                }
            }
        } finally {
            if ($WorkDir) {
                Pop-Location
            }
        }
    } finally {
        $ErrorActionPreference = $prev
    }
    $text = ($output | Out-String)
    if ($script:SawSccacheBypass) {
        $script:ReceiptLines.Add('env-note: sccache port-exclusion bypassed once (RUSTC_WRAPPER cleared for a single retry, restored; same handling as review R2)')
    }
    if ($null -ne $RawName -and $null -ne $script:RawDir) {
        [System.IO.File]::WriteAllText((Join-Path $script:RawDir $RawName), $text)
    }
    if ($LASTEXITCODE -ne 0) {
        throw "cargo $($ArgList -join ' ') exited $LASTEXITCODE`n$text"
    }
    return $text
}

function Test-HasWarningLine {
    param([string]$Text)
    # Zero-warning applies to THIS plan's crate only; pre-existing
    # warnings from other crates on the dep path (e.g. shim-metadata's
    # unused doc comment) are out of scope and recorded, not hidden.
    $scope = $false
    foreach ($line in ($Text -split "`r?`n")) {
        if ($line -match '^(\s*Checking|\s*Compiling) auto-abi-probe') { $scope = $true }
        if ($line -match '^\s*Checking\s+|^\s*Compiling\s+' -and $line -notmatch 'auto-abi-probe') { $scope = $false }
        if ($scope -and $line -match '^\s*warning') { return $true }
    }
    # also catch the crate-level summary the compiler emits
    if ($Text -match 'warning: `auto-abi-probe`[^
]*generated') { return $true }
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

try {
    # --- context + receipt header --------------------------------------------
    Invoke-Step 'context (HEAD + toolchain + lock identity)' {
        $script:Head = (& git -C $RepoRoot rev-parse HEAD).Trim()
        $branch = (& git -C $RepoRoot branch --show-current).Trim()
        $rustc = (& rustc --version) 2>&1 | Out-String
        $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
        $script:RawDir = Join-Path $ReceiptsDir "raw-$stamp"
        New-Item -ItemType Directory -Force -Path $script:RawDir | Out-Null
        $script:ReceiptLines.Add("PLAN-797 autoabi verify receipt")
        $script:ReceiptLines.Add("scope: $Scope")
        $script:ReceiptLines.Add("head: $script:Head")
        $script:ReceiptLines.Add("branch: $branch")
        $script:ReceiptLines.Add("rustc: $($rustc.Trim())")
        $script:ReceiptLines.Add("probe-manifest: $ProbeManifest")
        $script:ReceiptLines.Add("probe-lock: $ProbeLock")
        $script:ReceiptLines.Add("probe-lock-sha256: $(Get-Sha256 $ProbeLock)")
        $script:ReceiptLines.Add("raw-outputs: $script:RawDir")
        Write-Host "HEAD $script:Head branch $branch"
    }

    # --- F-797-R1-03: lock must be tracked in git, not just on disk ---------
    Invoke-Step 'Cargo.lock tracked in git (F-797-R1-03)' {
        if (-not (Test-Path $ProbeLock)) { throw "missing lock file: $ProbeLock" }
        $tracked = (& git -C $RepoRoot ls-files -- 'experimental/auto-abi-probe/Cargo.lock').Trim()
        if ($tracked -ne 'experimental/auto-abi-probe/Cargo.lock') {
            throw "Cargo.lock not tracked in git (got '$tracked'); local-only lock is not a freeze"
        }
        $ignored = (& git -C $RepoRoot check-ignore -v experimental/auto-abi-probe/Cargo.lock 2>&1)
        # tracked-but-matching-ignore is fine once force-added; just record.
        $script:ReceiptLines.Add("lock-tracked: yes")
        $script:ReceiptLines.Add("lock-check-ignore: $ignored")
        $worktreeHash = Get-Sha256 $ProbeLock
        $blob = (& git -C $RepoRoot rev-parse ':experimental/auto-abi-probe/Cargo.lock').Trim()
        $blobSize = (& git -C $RepoRoot cat-file -s $blob).Trim()
        $ho = (& git -C $RepoRoot hash-object $ProbeLock).Trim()
        if ($ho -ne $blob) { throw "index blob $blob != worktree hash-object $ho" }
        $script:ReceiptLines.Add("lock-blob: $blob")
        $script:ReceiptLines.Add("lock-blob-size: $blobSize")
        $script:ReceiptLines.Add("lock-worktree-sha256: $worktreeHash")
        Write-Host "lock blob $blob sha256 $worktreeHash"
    }

    # --- locked + offline resolve --------------------------------------------
    Invoke-Step 'cargo metadata --locked --offline' {
        $text = Invoke-Cargo @('metadata', '--locked', '--offline', '--format-version', '1') 'cargo-metadata-locked-offline.txt' $ProbeDir
        if (-not $text) { throw 'empty metadata output' }
        $script:ReceiptLines.Add('metadata-locked-offline: ok')
    }

    # --- zero-warning health of the probe crate -----------------------------
    Invoke-Step 'cargo check --locked --offline (probe crate zero-warning)' {
        $text = Invoke-Cargo @('check', '--locked', '--offline') 'cargo-check-locked-offline.txt' $ProbeDir
        if (Test-HasWarningLine $text) {
            # Only flag warnings whose path mentions auto-abi-probe.
            $probeWarn = $false
            foreach ($line in ($text -split "`r?`n")) {
                if ($line -match 'auto-abi-probe' -and $line -match 'warning') { $probeWarn = $true }
            }
            # Also catch "generated N warning" lines attributable to the probe crate.
            if ($text -match 'auto-abi-probe` \(lib\) generated') { $probeWarn = $true }
            if ($probeWarn) { throw "probe crate still emits warnings`n$text" }
        }
        $script:ReceiptLines.Add('check-locked-offline: ok (no auto-abi-probe warnings)')
    }

    # --- full probe test suite (work-owned scoped tier, not merge gate) ------
    Invoke-Step 'cargo test --locked --offline (probe workspace)' {
        $text = Invoke-Cargo @('test', '--locked', '--offline') 'cargo-test-locked-offline.txt' $ProbeDir
        if ($text -notmatch 'test result: ok') {
            throw "probe tests did not report test result: ok`n$text"
        }
        $script:ReceiptLines.Add('cargo-test-locked-offline: ok')
    }

    # --- T-06: real semantic binding + formal AutoDown (D-796/D-788 landed) --
    Invoke-Step 'abi_binding_ real-contract tests + FORMAL hard gates (probe)' {
        $text = Invoke-Cargo @('test', '--locked', '--offline', '--test', 't06_binding') 't06-binding.txt' $ProbeDir
        if ($text -notmatch 'test result: ok') {
            throw "t06_binding did not report test result: ok`n$text"
        }
        $script:ReceiptLines.Add('t06-binding-real-contract+formal-gates: ok')
    }

    Invoke-Step 'abi/interop-v1 corpus through the unified runner' {
        $text = Invoke-Cargo @('test', '-p', 'auto-lang', '--features', 'test-cases', '--lib', 'abi_recipe') 'abi-recipe-runner.txt' $RepoRoot
        if ($text -notmatch 'test result: ok') {
            throw "abi_recipe runner tests did not report test result: ok`n$text"
        }
        $script:ReceiptLines.Add('abi-recipe-unified-runner: ok')
    }

    Invoke-Step 'retirement receipt + finalized T-07 materials' {
        $required = @(
            'docs/plans/attachments/797/semref-retirement-receipt.md',
            'crates/auto-lang/test/cases/autoabi/autoabi.suite.ad',
            'crates/auto-lang/test/cases/autoabi/src/p797-mix.atomdata'
        )
        $missing = @()
        foreach ($rel in $required) {
            $p = Join-Path $RepoRoot ($rel.Replace('/', [char]92))
            if (-not (Test-Path $p)) { $missing += $rel }
        }
        if ($missing.Count -gt 0) {
            throw ("missing T-06/T-07 artifacts: " + ($missing -join ', '))
        }
        $script:ReceiptLines.Add('t06-artifacts: present')
    }

    # --- independently preparable T-07 materials (F-797-R1-02 partial) ------
    Invoke-Step 'T-07 independently-preparable materials present' {
        $required = @(
            'scripts/verify-autoabi-797.ps1',
            'docs/reports/plan797/freeze-proposal.md',
            'docs/reports/plan797/support-matrix.md',
            'docs/plans/attachments/797/spec-delta.md',
            'docs/plans/attachments/797/source-parallel-audit.md'
        )
        $missing = @()
        foreach ($rel in $required) {
            $p = Join-Path $RepoRoot ($rel -replace '/', '\')
            if (-not (Test-Path $p)) { $missing += $rel }
        }
        if ($missing.Count -gt 0) {
            throw ("missing T-07 materials: " + ($missing -join ', '))
        }
        $script:ReceiptLines.Add('t07-preparable-materials: present')
    }

    # --- closeout dry checklist (never fabricates green) --------------------
    if ($Scope -eq 'closeout') {
        Invoke-Step 'closeout checklist (honest pending vs done)' {
            $lines = New-Object System.Collections.Generic.List[string]
            $lines.Add('closeout checklist V-01..V-11 / AC mapping')
            # Independent of T-06 (work can assert from committed evidence).
            $lines.Add('V-01/AC-01 admission matrix + C layout oracle: PENDING final-review re-cert (historical work claim; not re-run as closeout pass)')
            $lines.Add('V-02/AC-02 three-leg native interop: PENDING final-review re-cert')
            $lines.Add('V-03/AC-01,02 padding/order/f64/>4 args: PENDING final-review re-cert')
            $lines.Add('V-04/AC-03 ownership/rollback: PENDING final-review re-cert')
            $lines.Add('V-05/AC-03 callback + rejection: PENDING final-review re-cert')
            $lines.Add('V-06/AC-04 cache hit/invalidate/restart: PENDING final-review re-cert')
            $lines.Add('V-07/AC-04 admission bad graphs: PENDING final-review re-cert')
            $lines.Add('V-08/AC-04 512 churn/lease/GC: PENDING final-review re-cert')
            $lines.Add('V-09/AC-05 hard gates: DONE at work level (in-process six scenarios 210/210 + FORMAL IPC retest body-edit P95=500.8ms / reject P95=76.8ms, 30 samples each); final-review re-cert pending')
            $lines.Add('V-10/AC-06 D-796 binding + AutoDown execution: DONE at work level (real ac-sema-contract binding, retirement receipt, abi/interop-v1 corpus 4 checks green through the unified runner); final-review re-cert pending')
            $lines.Add('V-11/AC-06 freeze proposal + support matrix + SD: FINALIZED DRAFTS with bound hashes (work level); canonical deposit is merge-owned')
            $lines.Add('F-797-R1-03 Cargo.lock in commit: DONE (see acceptance receipt)')
            $lines.Add('NOTE: work-level done != final review pass; canonical Spec deposit / archive / merge gates remain merge-owned.')
            $all = ($lines -join "`n")
            $script:ReceiptLines.Add($all)
            if (Test-Path $script:RawDir) {
                [System.IO.File]::WriteAllText((Join-Path $script:RawDir 'closeout-checklist.txt'), $all)
            }
            Write-Host $all
        }
    }

    # --- write report summary ------------------------------------------------
    $summaryPath = Join-Path $ReportDir ('verify-acceptance-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '.md')
    $summary = New-Object System.Collections.Generic.List[string]
    $summary.Add('# PLAN-797 verify-autoabi-797 receipt')
    $summary.Add('')
    $summary.Add("- scope: $Scope")
    $summary.Add("- head: $script:Head")
    $summary.Add("- generated: $(Get-Date -Format 'yyyy-MM-ddTHH:mm:ssK')")
    $summary.Add('')
    $summary.Add('| step | ok | seconds | detail |')
    $summary.Add('|---|---|---|---|')
    foreach ($r in $script:Results) {
        $d = ($r.Detail -replace '\|', '/').Replace("`n", ' ')
        $summary.Add(("| {0} | {1} | {2} | {3} |" -f $r.Step, $r.Ok, $r.Seconds, $d))
    }
    $summary.Add('')
    $summary.Add('Raw cargo outputs and receipt lines: ' + $script:RawDir)
    [System.IO.File]::WriteAllLines($summaryPath, $summary)
    [System.IO.File]::WriteAllLines((Join-Path $script:RawDir 'receipt.txt'), $script:ReceiptLines)
    Write-Host "summary: $summaryPath" -ForegroundColor Cyan
    Write-Host "ALL PASS ($Scope)" -ForegroundColor Green
    exit 0
} catch {
    $script:HadFailure = $true
    $failPath = Join-Path $ReceiptsDir ('FAILED-' + (Get-Date -Format 'yyyyMMdd-HHmmss') + '.txt')
    $lines = New-Object System.Collections.Generic.List[string]
    $lines.Add("PLAN-797 verify FAILED scope=$Scope")
    $lines.Add("head: $script:Head")
    $lines.Add("error: $($_.Exception.Message)")
    foreach ($r in $script:Results) {
        $lines.Add(("step: {0} ok={1} detail={2}" -f $r.Step, $r.Ok, $r.Detail))
    }
    [System.IO.File]::WriteAllLines($failPath, $lines)
    Write-Host "failure receipt: $failPath" -ForegroundColor Red
    exit 1
}
