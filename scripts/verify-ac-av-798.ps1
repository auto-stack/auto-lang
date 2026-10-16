# PLAN-798 verify script — scoped gates for the ac-av probe workspace.
# Usage: powershell -File scripts/verify-ac-av-798.ps1 -Scope <scope>
#   admission | semantics | cache | resources | turnaround | binding |
#   autodown | acceptance | closeout
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("admission", "semantics", "cache", "resources", "turnaround", "binding", "autodown", "acceptance", "closeout")]
    [string]$Scope,
    [int]$Samples = 30,
    [double]$GateMs = 1000.0
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
        # T-06 scope: real 796 semantic-ref binding. BLOCKED on D-796
        # (PLAN-796 not merged). Fails closed until unblocked.
        Write-Host "binding: BLOCKED on D-796 (796 independent review + frozen gates + actual merge). No fabricated evidence."
        exit 3
    }
    "autodown" {
        # T-07 scope: formal AutoDown recipe/worker on the landed probe/run
        # seam. BLOCKED on D-788 + D-796. Fails closed until unblocked.
        Write-Host "autodown: BLOCKED on D-788/D-796 (unified runner seam not merged). Rust-directed coverage: test case_binding."
        Run-Tests "case_binding" "autodown-directed"
        Write-Host "formal dispatch remains pending (exit 3 to keep the gate honest)"
        exit 3
    }
    "turnaround" {
        Write-Host "== turnaround release bench: 6 scenarios x $Samples samples, hard gate P95 < $GateMs ms"
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
        Run-Tests "vertical" "acceptance-vertical"
        Run-Tests "mir_tier" "acceptance-mir"
        Run-Tests "abc_semantics_tier" "acceptance-abc"
        Run-Tests "cache_tier" "acceptance-cache"
        Run-Tests "resources_tier" "acceptance-resources"
        Run-Tests "case_binding" "acceptance-cases"
        Run-Tests "native_trace_parity" "acceptance-native"
        Write-Host "acceptance: V-01..V-11 Rust-directed legs green; T-06/T-07 legs pending D-796/D-788 (see plan §9)."
    }
    "closeout" {
        Write-Host "closeout: merge-owned — full frozen-commit gates per plan §6; nothing executed here."
        exit 0
    }
}
Write-Host "scope $Scope done"
