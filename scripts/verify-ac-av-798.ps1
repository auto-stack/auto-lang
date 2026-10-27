# PLAN-798 verify script — scoped gates for the ac-av probe workspace.
# r7: turnaround is a strict diagnostic/measurement scope; a warm-only
# latency miss is preserved, but is not a Plan798 delivery blocker by itself.
# Formal 30-sample gates, correctness and memory remain mandatory.
# Usage: powershell -File scripts/verify-ac-av-798.ps1 -Scope <scope>
#   admission | semantics | cache | resources | turnaround | binding |
#   autodown | acceptance | closeout
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("admission", "semantics", "cache", "resources", "turnaround", "binding", "autodown", "acceptance", "closeout")]
    [string]$Scope,
    [int]$Samples = 30,
    [double]$GateMs = 2000.0
)

$ErrorActionPreference = "Continue"  # cargo writes progress to stderr; exit codes are the gate
$repo = Split-Path -Parent $PSScriptRoot
$ws = Join-Path $repo "experimental/ac-av-probe"
$reportDir = Join-Path $repo "docs/reports/plan798"
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null

if ($env:SCCACHE_SERVER_PORT -eq $null) { $env:SCCACHE_SERVER_PORT = "28101" }

function Run-Tests([string]$Filter, [string]$Label) {
    Write-Host "== $Label (cargo test filter: $Filter)"
    Push-Location $ws
    & cargo test -p auto-av-probe --test $Filter 2>&1 | Tee-Object -Variable out | Write-Host
    $code = $LASTEXITCODE
    Pop-Location
    if ($code -ne 0) {
        Write-Host "FAILED: $Label"
        exit 1
    }
    $out | Out-File (Join-Path $reportDir "$Label.log") -Encoding utf8
}

switch ($Scope) {
    "admission" {
        Run-Tests "mir_tier" "admission-mir"
        Run-Tests "abc_semantics_tier" "admission-abc"
    }
    "semantics" {
        Run-Tests "abc_semantics_tier" "semantics-av"
        Run-Tests "vertical" "semantics-vertical"
        Run-Tests "native_trace_parity" "semantics-native-parity"
    }
    "cache" {
        Run-Tests "cache_tier" "cache-units"
    }
    "resources" {
        Run-Tests "resources_tier" "resources-scale"
    }
    "binding" {
        # T-06 (D-796 merged): the landed ac-sema-contract identity domain —
        # same-name/different-owner separation, restart/ordering stability,
        # schema/version/tamper rejection, consumption-surface pin.
        Push-Location $ws
        & cargo test -p auto-av-probe --test binding_tier 2>&1 | Tee-Object -Variable out | Write-Host
        $code = $LASTEXITCODE
        Pop-Location
        if ($code -ne 0) { Write-Host "FAILED: binding tier"; exit 1 }
        $out | Out-File (Join-Path $reportDir "binding-tier.log") -Encoding utf8
        Write-Host "binding: ac-sema-contract identity domain consumed read-only (see binding-tier.log)"
    }
    "autodown" {
        # T-07 (D-788 + D-796 merged): the formal ac-av/core-run-v1 recipe on
        # the unified probe/run seam — corpus, negative oracle, decode
        # admission negatives and the FORMAL warm-edit hard gate (frozen
        # 30 samples / 2000 ms; AV798_FORMAL_SAMPLES/GATE_MS override).
        Push-Location $repo
        & cargo test -p auto-lang --lib --features test-cases ac_av_probe -- --test-threads=1 --nocapture 2>&1 | Tee-Object -Variable out | Write-Host
        $code = $LASTEXITCODE
        Pop-Location
        $out | Out-File (Join-Path $reportDir "autodown-formal.log") -Encoding utf8
        if ($code -ne 0) { Write-Host "FAILED: formal autodown tier"; exit 1 }
        Write-Host "autodown: formal recipe green incl. hard gate (see autodown-formal.log)"
    }
    "turnaround" {
        Write-Host "== turnaround release bench: 6 scenarios x $Samples samples, hard gate P90 <= $GateMs ms"
        Push-Location $ws
        & cargo build --release -p auto-av-probe --bin av_probe_worker 2>&1 | Write-Host
        if ($LASTEXITCODE -ne 0) { Pop-Location; exit 1 }
        $raw = Join-Path $reportDir "turnaround-raw.jsonl"
        $summary = Join-Path $reportDir "turnaround-summary.json"
        # The worker writes raw rows to $raw and prints ONE summary JSON line
        # to stdout; the pipe below is the ONLY writer of $summary (F798-R2-01:
        # a stale `$benchOut |` here used to truncate the file to 0 bytes).
        & (Join-Path $ws "target/release/av_probe_worker.exe") bench-turnaround --samples $Samples --gate-ms $GateMs --out "$raw" | Out-File $summary -Encoding utf8
        $code = $LASTEXITCODE
        Pop-Location
        # Sanity: the summary must be parseable JSON on BOTH the success and
        # the failure path - the gate result rides in it.
        try {
            $null = Get-Content -LiteralPath $summary -Raw | ConvertFrom-Json
        } catch {
            Write-Host "FAILED: summary at $summary is not parseable JSON: $_"
            exit 1
        }
        if ($code -ne 0) {
            Write-Host "FAILED: turnaround hard gate (exit $code) - summary: $summary raw: $raw"
            exit 1
        }
        Write-Host "turnaround PASS - summary: $summary raw: $raw"
    }
    "acceptance" {
        Push-Location $repo
        & cargo test -p auto-lang --lib --features test-cases ac_av_probe -- --test-threads=1 2>&1 | Tee-Object -Variable out2 | Write-Host
        $code2 = $LASTEXITCODE
        Pop-Location
        $out2 | Out-File (Join-Path $reportDir "acceptance-autodown.log") -Encoding utf8
        if ($code2 -ne 0) { exit 1 }
        Run-Tests "vertical" "acceptance-vertical"
        Run-Tests "mir_tier" "acceptance-mir"
        Run-Tests "abc_semantics_tier" "acceptance-abc"
        Run-Tests "cache_tier" "acceptance-cache"
        Run-Tests "resources_tier" "acceptance-resources"
        Run-Tests "case_binding" "acceptance-cases"
        Run-Tests "native_trace_parity" "acceptance-native"
        Write-Host "acceptance: scoped correctness and formal runner gates passed; current r7 delivery boundary and merge-owned gates remain recorded in the Plan."
    }
    "closeout" {
        Write-Host "closeout: merge-owned — full frozen-commit gates per plan §6; nothing executed here."
        exit 0
    }
}
Write-Host "scope $Scope done"
