# PLAN-817 verification driver (T-02 first-case scope; T-08 aggregates
# the full scope set). Usage:
#   powershell -File scripts/verify-ac-cte-817.ps1 -Scope <name>
# Scopes: first-case | admission | schedule | cache | freeze | resources
#         | autodown | turnaround | acceptance | closeout | all
param(
    [Parameter(Mandatory = $true)][string]$Scope,
    [string]$ReportDir = "docs/reports/plan817"
)
$ErrorActionPreference = "Continue"
$repo = Split-Path -Parent $PSScriptRoot
$ws = Join-Path $repo "experimental/ac-cte-probe/Cargo.toml"
$fail = 0

function Run-Step([string]$name, [scriptblock]$body) {
    Write-Host "==> $name" -ForegroundColor Cyan
    & $body
    if ($LASTEXITCODE -ne 0) { $script:fail = 1; Write-Host "FAILED: $name" -ForegroundColor Red }
}

function Test-Scope([string]$s) {
    switch ($s) {
        "first-case" {
            Run-Step "workspace cte_first_" { cargo test --manifest-path $ws --offline cte_first_ 2>&1 | Select-Object -Last 3 }
        }
        "admission" {
            Run-Step "workspace cte_admit_" { cargo test --manifest-path $ws --offline cte_admit_ 2>&1 | Select-Object -Last 3 }
        }
        "schedule" {
            Run-Step "workspace cte_schedule_" { cargo test --manifest-path $ws --offline cte_schedule_ 2>&1 | Select-Object -Last 3 }
        }
        "cache" {
            Run-Step "workspace cte_cache_" { cargo test --manifest-path $ws --offline cte_cache_ 2>&1 | Select-Object -Last 3 }
        }
        "freeze" {
            Run-Step "workspace cte_freeze_" { cargo test --manifest-path $ws --offline cte_freeze_ 2>&1 | Select-Object -Last 3 }
        }
        "resources" {
            Run-Step "workspace cte_resources_" { cargo test --manifest-path $ws --offline --release cte_resources_ 2>&1 | Select-Object -Last 3 }
        }
        "turnaround" {
            Run-Step "workspace cte_turnaround_" { cargo test --manifest-path $ws --offline cte_turnaround_ 2>&1 | Select-Object -Last 3 }
            Run-Step "bench raw report" {
                $out = Join-Path $repo "$ReportDir/turnaround.jsonl"
                $exe = Join-Path $repo "experimental/ac-cte-probe/target/release/cte_probe_worker.exe"
                cargo build --manifest-path $ws --offline --release -p auto-cte-probe --bin cte_probe_worker 2>&1 | Select-Object -Last 1
                & $exe bench-turnaround --samples 30 --out $out
            }
        }
        "autodown" {
            Run-Step "official corpus tests" {
                cargo test -p auto-lang --features test-cases --lib ac_cte_probe -- --test-threads=1 2>&1 | Select-Object -Last 5
            }
            Run-Step "auto-case lint" {
                cargo run -q -p auto-lang --features test-cases --bin auto-case -- lint --suite (Join-Path $repo "crates/auto-lang/test/cases/ac-cte/ac-cte.suite.ad") 2>&1 | Select-Object -Last 1
            }
        }
        "acceptance" {
            foreach ($s2 in @("first-case","admission","schedule","cache","freeze","turnaround","autodown")) { Test-Scope $s2 }
            Run-Step "workspace full" { cargo test --manifest-path $ws --offline --workspace 2>&1 | Select-String "test result" | Select-Object -Last 12 }
        }
        "closeout" {
            Run-Step "read-only consumption audit (no edits under consumed crates)" {
                foreach ($p in @("experimental/ac-core","experimental/ac-syntax","experimental/ac-artifact-probe","experimental/ac-av-probe","experimental/ac-semantics")) {
                    git -C $repo diff --quiet master -- $p
                    if ($LASTEXITCODE -ne 0) { throw "WRITE VIOLATION under $p" }
                }
                Write-Host "read-only consumption: clean"
            }
            Run-Step "registration append-only audit" {
                # Semantic check: every recipe string present on master's
                # REGISTERED_PROBE_RECIPES table still exists in the file
                # (formatting may change; entries may only be ADDED).
                $masterTable = git -C $repo show master:crates/auto-test-case/src/contract.rs | Select-String -Pattern 'REGISTERED_PROBE_RECIPES' -Context 0,4
                $masterText = ($masterTable.Context.PostContext + $masterTable.Line) -join " "
                $current = Get-Content (Join-Path $repo "crates/auto-test-case/src/contract.rs") -Raw
                foreach ($m in [regex]::Matches($masterText, '"([a-z0-9/-]+v[0-9]+)"')) {
                    if ($current -notmatch [regex]::Escape($m.Groups[1].Value)) {
                        throw "recipe entry removed: $($m.Groups[1].Value)"
                    }
                }
                Write-Host "registration append-only: clean"
            }
            Run-Step "no junctions/symlinks in the worktree" {
                $hits = Get-ChildItem -Path $repo -Recurse -Attributes ReparsePoint -ErrorAction SilentlyContinue | Select-Object -First 3
                if ($hits) { throw "reparse points present: $hits" }
                Write-Host "reparse-point scan: clean"
            }
            Run-Step "coverage matrix" { Get-Content (Join-Path $repo "docs/reports/plan817/acceptance-matrix.md") | Select-Object -First 8 }
        }
        "all" { foreach ($s2 in @("acceptance","resources","closeout")) { Test-Scope $s2 } }
        default { Write-Host "unknown scope $Scope"; exit 2 }
    }
}

Test-Scope $Scope
if ($fail -ne 0) { Write-Host "SCOPE $Scope FAILED" -ForegroundColor Red; exit 1 }
Write-Host "SCOPE $Scope PASS" -ForegroundColor Green
