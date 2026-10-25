# PLAN-821 verify script — scoped gates for the av.owned/1 profile.
# Usage: powershell -File scripts/verify-av-owned-821.ps1 -Scope <scope>
#   first-case (T-02) | semantics (T-03) | admission | resources (T-04) |
#   cache | compatibility (T-05) | autodown | acceptance (T-06)
# One scope per invocation (contract-decision §10). Scopes land with their
# tasks; an unimplemented scope is a NON-ZERO exit with an explicit
# message, never green. Every implemented scope enforces selected>0 (0
# selected / skipped required cases = red) and passes cargo exit codes
# through.
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("first-case", "semantics", "admission", "resources", "cache", "compatibility", "autodown", "acceptance")]
    [string]$Scope
)

$ErrorActionPreference = "Continue"  # cargo writes progress to stderr; exit codes are the gate
$repo = Split-Path -Parent $PSScriptRoot
$probeWs = Join-Path $repo "experimental/ac-av-probe"
$reportDir = Join-Path $repo "docs/reports/plan821"
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null

if ($env:SCCACHE_SERVER_PORT -eq $null) { $env:SCCACHE_SERVER_PORT = "28101" }

function Test-Selected([string]$Out, [string]$Label, [string[]]$RequiredIds = @()) {
    # Returns $true iff the run output proves: a NON-EMPTY selection, ZERO
    # really-skipped/ignored cases among the selected, and every REQUIRED
    # case id observed as exactly one passing result (the 827 validator
    # semantics — filter exclusion is not a skip; near-miss names do not
    # satisfy an id).
    $selected = 0
    if ($Out -match '(\d+) tests run') { $selected = [int]$Matches[1] }
    elseif ($Out -match '(\d+) passed') { $selected = [int]$Matches[1] }
    if ($selected -le 0) {
        Write-Host "FAILED: $Label selected 0 tests (0 selected is never a pass)"
        return $false
    }
    $realSkips = ([regex]::Matches($Out, '(?m)^\s*SKIP \[')).Count
    foreach ($m in [regex]::Matches($Out, 'test result:[^\r\n]*?(\d+) ignored')) {
        if ([int]$m.Groups[1].Value -gt 0) { $realSkips += [int]$m.Groups[1].Value }
    }
    if ($realSkips -gt 0) {
        Write-Host "FAILED: $Label has $realSkips skipped/ignored SELECTED cases (never green)"
        return $false
    }
    $seen = @{}
    foreach ($id in $RequiredIds) {
        if ($seen.ContainsKey($id)) {
            Write-Host "FAILED: $Label required list declares '$id' more than once"
            return $false
        }
        $seen[$id] = $true
        $esc = [regex]::Escape($id)
        $nextest = [regex]::Matches($Out, "(?m)^.*PASS.*[ :]$esc\r?$")
        $libtest = [regex]::Matches($Out, "(?m)^test $esc \.\.\. ok\r?$")
        $hits = $nextest.Count + $libtest.Count
        if ($hits -eq 0) {
            Write-Host "FAILED: $Label required case '$id' did not pass (missing/renamed/red)"
            return $false
        }
        if ($hits -gt 1) {
            Write-Host "FAILED: $Label required case '$id' matched $hits times (duplicate/ambiguous)"
            return $false
        }
    }
    Write-Host "   selected: $selected (0 real skips; required $($RequiredIds.Count) all passed)"
    return $true
}

function Assert-Selected([string]$Out, [string]$Label, [string[]]$RequiredIds = @()) {
    if (!(Test-Selected $Out $Label $RequiredIds)) { exit 1 }
}

function Run-Probe-Tests([string]$TestBin, [string]$Label, [string[]]$RequiredIds = @()) {
    Write-Host "== $Label (ac-av-probe --test $TestBin)"
    Push-Location $probeWs
    & cargo test -p auto-av-probe --test $TestBin 2>&1 | Tee-Object -Variable out | Write-Host
    $code = $LASTEXITCODE
    Pop-Location
    $out | Out-File (Join-Path $reportDir "$Label.log") -Encoding utf8
    if ($code -ne 0) {
        Write-Host "FAILED: $Label (exit $code)"
        exit 1
    }
    Assert-Selected ($out -join "`n") $Label $RequiredIds
}

function Run-Runner-Tests([string]$Filter, [string]$Label, [string[]]$RequiredIds = @()) {
    Write-Host "== $Label (main workspace, filter: $Filter)"
    Push-Location $repo
    & cargo t $Filter 2>&1 | Tee-Object -Variable out | Write-Host
    $code = $LASTEXITCODE
    Pop-Location
    $out | Out-File (Join-Path $reportDir "$Label.log") -Encoding utf8
    if ($code -ne 0) {
        Write-Host "FAILED: $Label (exit $code)"
        exit 1
    }
    Assert-Selected ($out -join "`n") $Label $RequiredIds
}

function Not-Wired([string]$Scope, [string]$Owner) {
    Write-Host "NOT WIRED: scope '$Scope' lands with $Owner (explicit non-green, never silent)"
    exit 1
}

function Invoke-Scope([string]$Scope) {
    # Run one scope as a REAL child script; $true iff it exits 0 (the
    # acceptance aggregate and failure propagation share this one path).
    $out = & powershell -File $PSCommandPath -Scope $Scope 2>&1
    $code = $LASTEXITCODE
    $out | Write-Host
    if ($code -ne 0) {
        Write-Host "scope '$Scope' red (exit $code)"
        return $false
    }
    return $true
}

function Invoke-Owned-Request([string]$Worker, [string]$Src, [string]$Store) {
    # One fresh-process owned request; returns the parsed JSON line.
    $out = & $Worker owned-request --store $Store --src-file $Src --entry d_main 2>$null | Where-Object { $_ -match '^\{' }
    if ($null -eq $out) { return $null }
    try { return ($out | ConvertFrom-Json) } catch { return $null }
}

switch ($Scope) {
    "first-case" {
        # V-01/V-08/V-10 first case, full formal path: the probe first-case
        # family (whole path + wire round trip + same-store restore + move
        # negatives + native independent comparison), the recipe-side
        # fixture/canonical/seeds-identity pins, and the formal corpus gate
        # (real worker, parent-side frozen-table judgement + sabotage and
        # worker-fault counter-controls).
        Run-Probe-Tests "av_owned_821_first_case" "first-case" @(
            "av_owned_821_first_case_whole_path_and_live_zero",
            "av_owned_821_first_case_wire_round_trip",
            "av_owned_821_first_case_same_store_restore",
            "av_owned_821_first_case_move_negative_controls",
            "av_owned_821_first_case_native_independent_comparison"
        )
        Run-Probe-Tests "av_owned_821_recipe" "first-case-recipe" @(
            "av_owned_821_recipe_seeds_identity",
            "av_owned_821_recipe_fixture_face_matches_hand_audit",
            "av_owned_821_recipe_canonical_face_shapes"
        )
        Run-Runner-Tests "av_owned_821" "first-case-corpus" @(
            "av_owned_821_corpus_green",
            "av_owned_821_negative_oracle_fails",
            "av_owned_821_worker_faults_are_infra"
        )
        Run-Probe-Tests "av_data_820" "first-case-regression-820" @()
        Push-Location $probeWs
        & cargo test -p auto-av-core --lib 2>&1 | Tee-Object -Variable coreOut | Write-Host
        $coreCode = $LASTEXITCODE
        Pop-Location
        $coreOut | Out-File (Join-Path $reportDir "first-case-regression-core.log") -Encoding utf8
        if ($coreCode -ne 0) {
            Write-Host "FAILED: auto-av-core regression (exit $coreCode)"
            exit 1
        }
        Assert-Selected ($coreOut -join "`n") "first-case-regression-core" @()

        # ---- T-02 CPU/time/memory evidence (plan §5.2) --------------------
        # Fresh-process owned-request samples on BOTH first-case sources;
        # the receipt binds worker sha256 + git head and the script
        # re-validates the hand-audited faces (result bits/bytes, live=0)
        # before writing the evidence JSON.
        Write-Host "== first-case: CPU/time/memory evidence (owned-request samples)"
        $worker = Join-Path $probeWs "target/release/av_probe_worker.exe"
        if (!(Test-Path $worker)) {
            Write-Host "FAILED: release worker not found at $worker (the corpus gate builds it; rerun if cleaned)"
            exit 1
        }
        $exeHash = (Get-FileHash $worker -Algorithm SHA256).Hash
        $gitHead = (& git -C $repo rev-parse HEAD 2>$null) -join ""
        $gitHead = $gitHead.Trim()
        $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
        $samples = @()
        foreach ($pair in @(
            @{ seed = "first-case.at"; n = 5 },
            @{ seed = "first-case-int-twin.at"; n = 3 }
        )) {
            $src = Join-Path $repo "crates/auto-lang/test/cases/av-data/821/src/$($pair.seed)"
            for ($i = 0; $i -lt $pair.n; $i++) {
                $store = Join-Path $env:TEMP "av821-evidence-$stamp-$($pair.seed)-$i"
                $r = Invoke-Owned-Request $worker $src $store
                if ($null -eq $r) {
                    Write-Host "FAILED: owned-request produced no JSON line (seed $($pair.seed) sample $i)"
                    exit 1
                }
                if ($r.ok -ne $true) {
                    Write-Host "FAILED: owned-request not ok (seed $($pair.seed) sample $i)"
                    exit 1
                }
                if ([int]$r.owned.live_after_release -ne 0) {
                    Write-Host "FAILED: live_after_release != 0 (seed $($pair.seed) sample $i)"
                    exit 1
                }
                $samples += [pscustomobject]@{
                    seed = $pair.seed
                    sample = $i
                    wall_ms = [double]$r.wall_ms
                    private_bytes_peak = [int64]$r.private_bytes_peak
                    result = $r.owned.result
                }
                Remove-Item -Recurse -Force $store -ErrorAction SilentlyContinue
            }
        }
        # Independent hand-audit re-check (script-side, not the worker's word):
        # the record face is Note{body:"Auto",n:4}; the twin's bits are 194.
        $rec = @($samples | Where-Object { $_.seed -eq "first-case.at" })
        $twin = @($samples | Where-Object { $_.seed -eq "first-case-int-twin.at" })
        if ($rec.Count -ne 5 -or $twin.Count -ne 3) {
            Write-Host "FAILED: evidence sample counts (record $($rec.Count)/5, twin $($twin.Count)/3)"
            exit 1
        }
        foreach ($s in $rec) {
            if ($s.result.name -ne "t_rec_Note" -or $s.result.fields[0].utf8 -ne "Auto" -or $s.result.fields[1].bits -ne "4") {
                Write-Host "FAILED: record face deviates from the hand-audit: $($s.result | ConvertTo-Json -Compress)"
                exit 1
            }
        }
        foreach ($s in $twin) {
            if ($s.result.kind -ne "i32" -or $s.result.bits -ne "194") {
                Write-Host "FAILED: twin face deviates from the hand-audit (404 = 0x194): $($s.result | ConvertTo-Json -Compress)"
                exit 1
            }
        }
        $evidence = [pscustomobject]@{
            stamp             = $stamp
            scope             = "first-case"
            worker_sha256     = $exeHash
            git_head          = $gitHead
            machine_note      = "single-instance serial, fresh process per sample (poll-quantized peaks)"
            samples           = $samples
            wall_ms_p95       = [math]::Round((($samples | ForEach-Object { $_.wall_ms } | Sort-Object)[[int][math]::Floor(($samples.Count - 1) * 0.95)]), 3)
            wall_ms_max       = [math]::Round(($samples | ForEach-Object { $_.wall_ms } | Measure-Object -Maximum).Maximum, 3)
            peak_bytes_max    = ($samples | ForEach-Object { $_.private_bytes_peak } | Measure-Object -Maximum).Maximum
            budget_note       = "worker B-line increment gate (128MiB) is the T-04/T-06 tier; these are the T-02 baseline samples"
        }
        $evPath = Join-Path $reportDir "first-case-evidence-$stamp.json"
        $evidence | ConvertTo-Json -Depth 8 | Out-File $evPath -Encoding utf8
        Write-Host "   evidence: $evPath (p95 wall $($evidence.wall_ms_p95)ms, max peak $($evidence.peak_bytes_max) bytes)"
    }
    "semantics" {
        # V-01 reservation control / V-02 move+borrow / V-05 handles /
        # V-06 text / V-11/V-12 control-flow joins — T-03.
        Not-Wired "semantics" "T-03"
    }
    "admission" {
        # V-02..V-06 admission faces — T-04 (with resources).
        Not-Wired "admission" "T-04"
    }
    "resources" {
        # V-03/V-07 allocator/budget axes + the dual-line memory tier — T-04.
        Not-Wired "resources" "T-04"
    }
    "cache" {
        # V-08 restore/re-key/corruption faces — T-05.
        Not-Wired "cache" "T-05"
    }
    "compatibility" {
        # V-09 core-i32/data/817/818 replay + unknown-profile refusals — T-05.
        Not-Wired "compatibility" "T-05"
    }
    "autodown" {
        # V-10 full counter-control matrix — T-06.
        Not-Wired "autodown" "T-06"
    }
    "acceptance" {
        # Aggregates EVERY implemented scope (failures propagate) — T-06.
        Not-Wired "acceptance" "T-06"
    }
}
Write-Host "PASS: scope '$Scope' green"
exit 0
