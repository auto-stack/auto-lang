# PLAN-827 verify script — scoped gates for the av-source baseline.
# Usage: powershell -File scripts/verify-av-source-827.ps1 -Scope <scope>
#   first-case (T-03) | semantics | admission | cache | resources |
#   compatibility | autodown | inventory | acceptance
# Scopes land with their tasks (T-03..T-06); an unimplemented scope is a
# NON-ZERO exit with an explicit message, never green. Every implemented
# scope enforces selected>0 (0 selected / skipped required cases = red),
# and cargo exit codes are passed through.
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("first-case", "semantics", "admission", "cache", "resources", "compatibility", "autodown", "inventory", "incremental", "acceptance", "selftest", "fail-shim")]
    [string]$Scope
)

$ErrorActionPreference = "Continue"  # cargo writes progress to stderr; exit codes are the gate
$repo = Split-Path -Parent $PSScriptRoot
$probeWs = Join-Path $repo "experimental/ac-av-probe"
$reportDir = Join-Path $repo "docs/reports/plan827"
New-Item -ItemType Directory -Force -Path $reportDir | Out-Null

if ($env:SCCACHE_SERVER_PORT -eq $null) { $env:SCCACHE_SERVER_PORT = "28101" }

function Test-Selected([string]$Out, [string]$Label, [string[]]$RequiredIds = @()) {
    # Returns $true iff the run output proves: a NON-EMPTY selection, ZERO
    # REALLY-skipped/ignored cases among the selected, and every REQUIRED
    # case id observed as a passing result (R1-F01 + R2 correction).
    #
    # Skip semantics distinguished (R2): nextest reports the FILTERED-OUT
    # global population in the "Starting N tests across M binaries (K tests
    # skipped)" line and in the Summary's skipped total — filter exclusion
    # is NOT a required-case skip. The real signals are (a) nextest SKIP
    # result lines (a SELECTED test that skipped) and (b) libtest
    # "test result: ... N ignored" counts.
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
    # EXACT required-case binding (R3): a nextest PASS result line ENDS
    # with the exact test id (optionally \r), preceded by `::` or a space;
    # a libtest result line names it exactly before " ... ok". Substring
    # collisions (`<id>_disabled`, `x_<id>`, `<id>_v2`) do NOT match. A
    # required id must match EXACTLY ONCE — 0 = missing/renamed, >1 =
    # duplicated/ambiguous relay.
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

function Test-WorkerBinding([string]$Exe, [string]$BoundSha256) {
    # R7 F-827-R7-02: the release worker a resource gate exercises must be
    # BYTE-IDENTICAL to the binary recorded in the run's binding receipt
    # (built from THIS workspace). A stale/foreign binary, a tampered
    # receipt hash, or a missing exe is RED — checked BEFORE any campaign
    # runs, so an old artifact can never supply the current gate.
    if ([string]::IsNullOrEmpty($BoundSha256)) { return $false }
    if (!(Test-Path $Exe)) { return $false }
    return ((Get-FileHash $Exe -Algorithm SHA256).Hash -eq $BoundSha256)
}

function Get-BenchStdoutSummary([string[]]$Out) {
    # R8 F-827-R8-01: the bench worker prints exactly ONE JSON summary line
    # to stdout (it carries `hard_gate_failures`). Extract and return it
    # parsed; $null if absent, throw if ambiguous (>1 summary line).
    $found = @()
    foreach ($line in $Out) {
        $t = "$line".Trim()
        if ($t.StartsWith('{') -and $t.Contains('"hard_gate_failures"')) {
            try { $found += ($t | ConvertFrom-Json) } catch { }
        }
    }
    if ($found.Count -gt 1) {
        throw "bench stdout carried $($found.Count) summary JSON lines (expected exactly 1)"
    }
    if ($found.Count -eq 1) { return $found[0] }
    return $null
}

function Test-BenchSummary([string]$Path, [bool]$ExpectGateFailure) {
    # R8 F-827-R8-01: a retained bench-summary artifact must carry the REAL
    # stdout summary and the EXACT exit code of its run, bound to the
    # binary/raw it certifies, and its real fields must CORROBORATE the
    # recorded outcome:
    #   gate-0 (ExpectGateFailure): exit != 0 AND hard_gate_failures
    #     non-empty — the INTENDED threshold rejection. An unrelated nonzero
    #     exit with NO gate failures (the case the durable files previously
    #     could not distinguish) is RED.
    #   normal bench: exit == 0 AND hard_gate_failures empty AND both
    #     dual-line memory targets met, with all peak fields present.
    if ([string]::IsNullOrEmpty($Path) -or !(Test-Path $Path)) { return $false }
    try { $a = Get-Content $Path -Raw | ConvertFrom-Json } catch { return $false }
    foreach ($f in @('stamp', 'exe_sha256', 'git_head', 'exit_code', 'raw_file', 'summary')) {
        if ($null -eq $a.$f) { return $false }
    }
    $s = $a.summary
    if ($null -eq $s.hard_gate_failures) { return $false }
    foreach ($f in @(
        'frontend_peak_bytes', 'private_bytes_peak', 'child_private_bytes_peak',
        'worker_private_bytes_delta', 'frontend_transient_target_met',
        'worker_private_bytes_target_met'
    )) {
        if ($null -eq $s.$f) { return $false }
    }
    if ($ExpectGateFailure) {
        # R9: the intended gate-0 control is the 0ms-THRESHOLD rejection of
        # the two frozen hard-gate paths — not merely any failed bench. The
        # real summary must show gate_ms == 0 (the 0 gate was actually
        # exercised) and every failure must be the threshold-rejection form
        # of signature-conflict/single-body-edit at 0.0ms; an unrelated
        # correctness/resource/infra failure (e.g. 'fresh: failing samples
        # [0]') or a healthy-threshold failure is RED.
        if ($null -eq $s.gate_ms -or [double]$s.gate_ms -ne 0) { return $false }
        $f = @($s.hard_gate_failures)
        if ($f.Count -eq 0 -or ($a.exit_code -eq 0)) { return $false }
        $patSig = '^(hard gate: )?signature-conflict P90 [0-9.]+ms > 0\.0ms$'
        $patSbe = '^(hard gate: )?single-body-edit P90 [0-9.]+ms > 0\.0ms$'
        foreach ($x in $f) {
            if (($x -notmatch $patSig) -and ($x -notmatch $patSbe)) { return $false }
        }
        return ((@($f | Where-Object { $_ -match $patSig }).Count -ge 1) `
            -and (@($f | Where-Object { $_ -match $patSbe }).Count -ge 1))
    }
    return (($a.exit_code -eq 0) `
        -and (@($s.hard_gate_failures).Count -eq 0) `
        -and $s.frontend_transient_target_met `
        -and $s.worker_private_bytes_target_met)
}

function Save-BenchSummaryArtifact(
    [string]$Path, [string]$Stamp, [string]$ExeSha256, [string]$GitHead,
    [string]$BenchArgs, [int]$ExitCode, [string]$RawRel, $Summary, [string]$RawPath
) {
    # R8 F-827-R8-01: persist the run's ACTUAL stdout summary and exact exit
    # as a durable artifact bound to exe/raw/head — written even when the
    # bench gate fails, so failure evidence is retained.
    # R9: the parameter is `BenchArgs`, NOT `Args` — `$args` is a PowerShell
    # automatic variable; the previous `Args`-named parameter bound the call
    # site but silently wrote an empty string (review-r9 writer control).
    $inputHash = $null
    if (Test-Path $RawPath) {
        $first = Get-Content $RawPath -TotalCount 1
        try { $inputHash = ($first | ConvertFrom-Json).source_hash } catch { }
    }
    [pscustomobject]@{
        stamp                      = $Stamp
        exe_sha256                 = $ExeSha256
        git_head                   = $GitHead
        args                       = $BenchArgs
        exit_code                  = $ExitCode
        raw_file                   = $RawRel
        input_source_hash_observed = $inputHash
        summary                    = $Summary
    } | ConvertTo-Json -Depth 8 | Out-File $Path -Encoding utf8
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
    . (Join-Path $repo 'scripts/test-governance/invoke-modules.ps1')
    Invoke-ScopedTestModules -Repo $repo -TestNames @($Filter) -Output (Join-Path $reportDir ($Label+'-modules')) -RequiredIds $RequiredIds
    $out=@('Original identities checked in module execution receipt')
    $code = $LASTEXITCODE
    Pop-Location
    $out | Out-File (Join-Path $reportDir "$Label.log") -Encoding utf8
    if ($code -ne 0) {
        Write-Host "FAILED: $Label (exit $code)"
        exit 1
    }
    # Required IDs were checked against the actual module receipt.
}

function Not-Wired([string]$Scope, [string]$Owner) {
    Write-Host "NOT WIRED: scope '$Scope' lands with $Owner (explicit non-green, never silent)"
    exit 1
}

function Invoke-Scope([string]$Scope) {
    # Run one scope as a REAL child script and return $true iff it exits 0.
    # The acceptance aggregate and the selftest failure-propagation control
    # share this one path (R1-F01). Child output is captured and shown via
    # Write-Host so the function's return value is ONLY the boolean.
    $out = & powershell -File $PSCommandPath -Scope $Scope 2>&1
    $code = $LASTEXITCODE
    $out | Write-Host
    if ($code -ne 0) {
        Write-Host "scope '$Scope' red (exit $code)"
        return $false
    }
    return $true
}

switch ($Scope) {
    "first-case" {
        # V-01 first case, full path: real AP-01 frozen bytes (identity
        # receipts) -> ephemeral -> store put/hit -> NEW-worker restore, plus
        # the formal-recipe corpus gate judged parent-side.
        Run-Probe-Tests "av_source_827_first_case" "first-case" @(
            "av_source_827_seeds_identity",
            "av_source_827_first_case_real_file_ephemeral",
            "av_source_827_first_case_store_round_hit_new_worker",
            "av_source_827_first_case_native_cross_check",
            "av_source_827_native_missing_tool_is_red_first_case_control"
        )
        Run-Runner-Tests "av_source_827" "first-case-corpus" @("av_source_827_corpus_green")
    }
    "semantics" {
        # V-02/03 (+ targeted V-04/05/09 axes): AP-22 argument-value change,
        # AP-23 precise trap + false-only control + native exit70 channel,
        # trace authorization, same-path/frozen-bytes identity, front-door
        # rejections with zero dispatch, budget/cancel axes.
        Run-Probe-Tests "av_source_827_semantics" "semantics" @(
            "av_source_827_ap22_argument_value_change_changes_result",
            "av_source_827_ap23_precise_trap_and_false_only_control",
            "av_source_827_ap23_native_exit70_channel",
            "av_source_827_native_missing_tool_is_red_ap23_control",
            "av_source_827_trace_order_exact_and_capability_gated",
            "av_source_827_same_path_different_bytes_and_frozen_snapshot",
            "av_source_827_front_door_rejects_disallow_no_dispatch",
            "av_source_827_resource_budget_and_cancel_axes"
        )
        Run-Runner-Tests "av_source_827_corpus" "semantics-corpus" @("av_source_827_corpus_green")
    }
    "admission" {
        # V-04 runner-level admission negatives: invalid UTF-8 = infra (not
        # a trap), missing file = resolution error, path escapes rejected
        # at decode. Plus the unsupported front-door corpus faces.
        Run-Runner-Tests "av_source_827_admission" "admission" @(
            "av_source_827_admission_invalid_utf8_is_infra_not_trap",
            "av_source_827_admission_missing_file_is_resolve_error",
            "av_source_827_admission_path_escape_rejected_at_decode"
        )
        Run-Runner-Tests "av_source_827_corpus" "admission-corpus" @("av_source_827_corpus_green")
        # R5 V-06 binding — the INHERITED D798 consume-gate counterexamples
        # and positives as REQUIRED members (unchanged at the consumed
        # revision; removal of any row breaks acceptance):
        Run-Probe-Tests "cache_tier" "admission-inherited-cache" @(
            "av_cache_corrupt_frame_rejected",
            "av_cache_conflict_preserves_committed",
            "av_cache_wrong_identity_rejected",
            "av_cache_valid_payload_under_wrong_body_key_rejected"
        )
        Run-Probe-Tests "abc_semantics_tier" "admission-inherited-abc" @(
            "abc_admission_only_verify_produces_executable_handle",
            "abc_admission_tampered_inner_never_executes",
            "abc_codec_rejects_truncation_and_trailing"
        )
        Run-Probe-Tests "native_admission_tier" "admission-inherited-native" @(
            "native_admission_real_coff_positive",
            "native_admission_identity_axes_rejected"
        )
        # 827 thin source-entry replay of the same invariant:
        Run-Probe-Tests "av_source_827_semantics" "admission-thin-consume-gate" @(
            "av_source_827_consume_gate_corrupt_abc_never_executes"
        )
    }
    "resources" {
        # V-09 heavy tier (single instance, serial):
        # 0. R7 F-R7-02 binding: the release worker is BUILT from THIS frozen
        #    workspace, its SHA256 + build command + source identity recorded
        #    in a per-run receipt naming the raw files it certifies, and
        #    stale/foreign binaries are rejected BEFORE any campaign runs;
        # 1. gate-0 counter-control: bench-turnaround with --gate-ms 0 MUST
        #    fail (a threshold of 0 can never be green);
        # 2. the real P90 gate: 30 samples, gate 2000ms, raw JSONL kept;
        # 3. Line-A frontend attribution + Line-B residual via the frozen
        #    828 formal-session campaign (V-08 dual gate, >=30 revisions).
        # Small-file cold data is reported separately by the first-case log.
        $worker = Join-Path $probeWs "target/release/av_probe_worker.exe"
        # Every bench run keeps its OWN raw JSONL (failed runs included —
        # failure samples are never deleted or overwritten).
        $stamp = Get-Date -Format "yyyyMMdd-HHmmss"

        Write-Host "== resources: build the release worker from the frozen workspace (F-R7-02)"
        Push-Location $probeWs
        & cargo build --release -p auto-av-probe --bin av_probe_worker 2>&1 | Tee-Object -Variable wbuild | Write-Host
        $wbcode = $LASTEXITCODE
        Pop-Location
        $wbuild | Out-File (Join-Path $reportDir "resources-worker-build-$stamp.log") -Encoding utf8
        if ($wbcode -ne 0) {
            Write-Host "FAILED: release worker build (exit $wbcode) — no unbound binary may run the gate"
            exit 1
        }

        # Binding receipt: binary hash + build provenance + frozen source
        # identity (the ac-av-probe workspace whose build certifies the
        # binary), naming the raw AND retained-summary files it certifies —
        # the deliberately FAILED gate-0 run included.
        $exeHash = (Get-FileHash $worker -Algorithm SHA256).Hash
        $gitHead = (& git -C $repo rev-parse HEAD 2>$null) -join ""
        $gitHead = $gitHead.Trim()
        $probeDirty = @(& git -C $repo status --porcelain -- experimental/ac-av-probe 2>$null)
        $binding = [pscustomobject]@{
            stamp            = $stamp
            exe              = $worker
            sha256           = $exeHash
            built_by         = "cargo build --release -p auto-av-probe --bin av_probe_worker (resources scope)"
            git_head         = $gitHead
            probe_ws_entries = @($probeDirty)
            certifies        = @(
                "bench-gate0-$stamp.jsonl", "bench-30-$stamp.jsonl",
                "bench-summary-gate0-$stamp.json", "bench-summary-30-$stamp.json"
            )
            created_at       = (Get-Date -Format o)
        }
        $binding | ConvertTo-Json -Depth 4 | Out-File (Join-Path $reportDir "resources-worker-binding-$stamp.json") -Encoding utf8

        # Stale/foreign-worker negatives + the current-worker positive, all
        # BEFORE a campaign runs: a foreign binary, a tampered receipt hash
        # and a missing exe must each FAIL the binding check.
        $stale = Join-Path $env:TEMP "av827-stale-worker-$stamp.exe"
        Set-Content -Path $stale -Value "stale worker bytes (F-R7-02 binding negative)" -Encoding ascii
        if (Test-WorkerBinding $stale $exeHash) {
            Write-Host "FAILED: a foreign binary satisfied the worker binding (stale-worker negative)"
            exit 1
        }
        # Tamper to a GUARANTEED-different hash (the real worker's SHA256
        # itself starts with '0', so a naive "0"-prefix would be a no-op).
        $flip = if ($exeHash[0] -eq '0') { '1' } else { '0' }
        $tamperedHash = $flip + $exeHash.Substring(1)
        if (Test-WorkerBinding $worker $tamperedHash) {
            Write-Host "FAILED: a tampered receipt hash satisfied the worker binding"
            exit 1
        }
        if (Test-WorkerBinding "$($worker).missing" $exeHash) {
            Write-Host "FAILED: a missing exe satisfied the worker binding"
            exit 1
        }
        if (!(Test-WorkerBinding $worker $exeHash)) {
            Write-Host "FAILED: the freshly built worker does not match its binding receipt"
            exit 1
        }
        Write-Host "   worker bound: sha256=$($exeHash.Substring(0,16))... head=$($gitHead.Substring(0, [Math]::Min(10, $gitHead.Length))) probe-ws dirty entries=$(@($probeDirty).Count) (3 negatives red, current green)"

        Write-Host "== resources: bench gate-0 counter-control (must FAIL)"
        Push-Location $probeWs
        & $worker bench-turnaround --samples 3 --gate-ms 0 --out "$reportDir/bench-gate0-$stamp.jsonl" 2>&1 | Tee-Object -Variable g0 | Write-Host
        $g0code = $LASTEXITCODE
        Pop-Location
        # R8 F-827-R8-01: persist the ACTUAL stdout summary + exact exit,
        # bound to exe/raw/head, then validate the intended rejection FROM
        # THE RETAINED REAL FIELDS (an unrelated nonzero exit with no gate
        # failures must be red, not accepted as the counter-control).
        $g0Summary = Get-BenchStdoutSummary $g0
        $g0SummaryPath = Join-Path $reportDir "bench-summary-gate0-$stamp.json"
        if ($null -eq $g0Summary) {
            Write-Host "FAILED: gate-0 stdout carried no summary JSON — retention evidence incomplete"
            exit 1
        }
        Save-BenchSummaryArtifact -Path $g0SummaryPath -Stamp $stamp -ExeSha256 $exeHash -GitHead $gitHead `
            -BenchArgs "bench-turnaround --samples 3 --gate-ms 0 --out bench-gate0-$stamp.jsonl" `
            -ExitCode $g0code -RawRel "bench-gate0-$stamp.jsonl" -Summary $g0Summary `
            -RawPath (Join-Path $reportDir "bench-gate0-$stamp.jsonl")
        if (!(Test-BenchSummary $g0SummaryPath $true)) {
            Write-Host "FAILED: retained gate-0 summary does not corroborate the intended hard-gate rejection (exit $g0code)"
            exit 1
        }
        # Inline tampered-summary negative in the REAL gate path: the same
        # artifact with its hard_gate_failures emptied must FAIL validation
        # (a doctored intended-failure summary cannot pass).
        $tampered = Get-Content $g0SummaryPath -Raw | ConvertFrom-Json
        $tampered.summary.hard_gate_failures = @()
        $tamperedPath = Join-Path $env:TEMP "av827-tampered-gate0-$stamp.json"
        $tampered | ConvertTo-Json -Depth 8 | Out-File $tamperedPath -Encoding utf8
        if (Test-BenchSummary $tamperedPath $true) {
            Write-Host "FAILED: a tampered gate-0 summary (failures emptied) satisfied validation"
            exit 1
        }
        if ($g0code -eq 0) {
            Write-Host "FAILED: gate-0 counter-control was green (a 0ms gate must be red)"
            exit 1
        }
        Write-Host "   gate-0 red as required (exit $g0code; retained summary shows the intended rejection; tampered negative red)"
        Write-Host "== resources: bench-turnaround 30 samples gate 2000ms"
        Push-Location $probeWs
        & $worker bench-turnaround --samples 30 --gate-ms 2000 --out "$reportDir/bench-30-$stamp.jsonl" 2>&1 | Tee-Object -Variable b30 | Write-Host
        $b30code = $LASTEXITCODE
        Pop-Location
        # R8 F-827-R8-01: persist the actual summary + exact exit FIRST (a
        # failing gate leaves its evidence), then validate green-ness from
        # the retained real fields.
        $b30Summary = Get-BenchStdoutSummary $b30
        $b30SummaryPath = Join-Path $reportDir "bench-summary-30-$stamp.json"
        if ($null -eq $b30Summary) {
            Write-Host "FAILED: bench-30 stdout carried no summary JSON — retention evidence incomplete"
            exit 1
        }
        Save-BenchSummaryArtifact -Path $b30SummaryPath -Stamp $stamp -ExeSha256 $exeHash -GitHead $gitHead `
            -BenchArgs "bench-turnaround --samples 30 --gate-ms 2000 --out bench-30-$stamp.jsonl" `
            -ExitCode $b30code -RawRel "bench-30-$stamp.jsonl" -Summary $b30Summary `
            -RawPath (Join-Path $reportDir "bench-30-$stamp.jsonl")
        if (!(Test-BenchSummary $b30SummaryPath $false)) {
            Write-Host "FAILED: bench-turnaround gate (exit $b30code; retained summary does not corroborate green)"
            exit 1
        }
        Write-Host "   bench-30 green from retained fields (exit $b30code, 0 hard-gate failures, both memory targets met)"
        Write-Host "== resources: 828 formal-session dual-gate campaign (V-08)"
        Push-Location $probeWs
        & cargo test -p auto-av-probe --test function_incremental_828_t08 t08_formal_campaign_v08_dual_gate 2>&1 | Tee-Object -Variable f828 | Write-Host
        $f828code = $LASTEXITCODE
        Pop-Location
        if ($f828code -ne 0) {
            Write-Host "FAILED: formal campaign (exit $f828code)"
            exit 1
        }
        Assert-Selected ($f828 -join "`n") "resources-formal-828" @("t08_formal_campaign_v08_dual_gate_30_revisions")
        # R5 V-09 binding — inherited finite resource oracles as REQUIRED
        # members (cap-boundary, churn/cleanup under the 32MiB envelope).
        Run-Probe-Tests "resources_tier" "resources-inherited-finite" @(
            "av_resources_over_table_budget_is_resource",
            "av_resources_front_door_nesting_boundary",
            "av_resources_32mib_product_random_read_and_churn"
        )
        # 827 thin cap-boundary pair (at-cap passes / over-one rejects).
        Run-Probe-Tests "av_source_827_semantics" "resources-thin-cap-pair" @(
            "av_source_827_input_cap_at_and_over_one",
            "av_source_827_resource_budget_and_cancel_axes",
            "av_source_827_nth_frame_allocation_failure_cleans_owned_slots",
            "av_source_827_frame_allocation_pause_cancel_trap_cleanup"
        )
    }
    "autodown" {
        # V-10: the formal recipe positives + every parent-side injection:
        # corpus green (with required population), negative oracle, worker
        # faults (exit9/extra-frame/wrong-hash/delay = infra).
        Run-Runner-Tests "av_source_827" "autodown" @(
            "av_source_827_corpus_green",
            "av_source_827_negative_oracle_fails",
            "av_source_827_worker_faults_are_infra"
        )
    }
    "inventory" {
        # V-11: the G0 denominators re-derived from the real discovery
        # rules; the JSON artifact is the inventory deliverable.
        Run-Runner-Tests "av_source_827_inventory" "inventory" @(
            "av_source_827_inventory_denominators_replayable",
            "av_source_827_inventory_negative_controls"
        )
    }
    "acceptance" {
        # V-01..12 ALL required sub-cases across EVERY implemented scope —
        # including resources (R1-F01: a resource gate failure must break
        # the aggregate) — selected>0 with 0 skips everywhere, plus the
        # SEM decision material (SEM-01..08) presence check.
        $sem = Join-Path $repo "docs/plans/attachments/827/sem-decisions.md"
        if (!(Test-Path $sem)) {
            Write-Host "FAILED: SEM decision material missing ($sem)"
            exit 1
        }
        $semRows = (Select-String -Path $sem -Pattern '^\| SEM-\d\d').Count
        if ($semRows -lt 8) {
            Write-Host "FAILED: SEM table must carry SEM-01..08 (saw $semRows rows)"
            exit 1
        }
        Write-Host "== acceptance: replaying every implemented scope (resources included)"
        foreach ($scope in @(
            "first-case", "semantics", "admission", "cache", "autodown",
            "inventory", "compatibility", "incremental", "resources"
        )) {
            if (!(Invoke-Scope $scope)) {
                Write-Host "FAILED: acceptance — scope '$scope' red"
                exit 1
            }
        }
        Write-Host "PASS: acceptance (all scopes incl. resources + SEM material)"
    }
    "selftest" {
        # R1-F01/R2 repair evidence: negative and positive controls replayed
        # against the REAL validator/aggregator pieces. Bad shapes are RED,
        # the legitimate filtered run is GREEN.
        $fails = 0
        # R2 regression control (the R2 misfire): a REAL filtered nextest
        # run — the Starting line and the Summary carry the FILTERED-OUT
        # population as "skipped" (6926), but no SKIP result line exists
        # and every REQUIRED case passed -> must be GREEN.
        $filtered = "Starting 2 tests across 8 binaries (6926 tests skipped)`n" +
            "        PASS [ 0.04s] auto-lang tests::av_source_827_tests::av_source_827_inventory_denominators_replayable`n" +
            "        PASS [ 0.01s] auto-lang tests::av_source_827_tests::av_source_827_inventory_negative_controls`n" +
            "Summary [ 0.085s] 2 tests run: 2 passed, 6926 skipped"
        if (!(Test-Selected $filtered "filtered-positive" @(
            "av_source_827_inventory_denominators_replayable",
            "av_source_827_inventory_negative_controls"
        ))) { Write-Host "CONTROL FAILED: filtered-out population must not be a skip failure"; $fails++ }
        # A real SKIP result line (a SELECTED test skipped) -> red.
        if (Test-Selected $filtered.Replace("PASS [ 0.04s]", "SKIP [ 0.04s]") "real-skip") {
            Write-Host "CONTROL FAILED: a selected skipped case must be red"; $fails++
        }
        # A required case deleted (its PASS line absent) -> red.
        $missingReq = $filtered.Replace("        PASS [ 0.01s] auto-lang tests::av_source_827_tests::av_source_827_inventory_negative_controls`n", "")
        if (Test-Selected $missingReq "missing-required" @("av_source_827_inventory_negative_controls")) {
            Write-Host "CONTROL FAILED: a missing required case must be red"; $fails++
        }
        # R3 collision matrix — the closure strategy verified as a matrix,
        # not just the old example. Every near-miss name shape is RED:
        $reqId = "av_source_827_inventory_negative_controls"
        $base = "PASS [ 0.01s] auto-lang tests::av_source_827_tests::${reqId}`nSummary [0.1s] 1 tests run: 1 passed, 6927 skipped"
        foreach ($shape in @(
            @{ n = "suffix-disabled"; s = $base.Replace($reqId, "${reqId}_disabled") },
            @{ n = "prefix-collision"; s = $base.Replace($reqId, "x_${reqId}") },
            @{ n = "versioned"; s = $base.Replace($reqId, "${reqId}_v2") },
            @{ n = "unrelated"; s = $base.Replace($reqId, "unrelated_case") }
        )) {
            if (Test-Selected $shape.s $shape.n @($reqId)) {
                Write-Host "CONTROL FAILED: $($shape.n) name must NOT satisfy the required id"; $fails++
            }
        }
        # The EXACT id stays green (boundary positive).
        if (!(Test-Selected $base "exact-positive" @($reqId))) {
            Write-Host "CONTROL FAILED: the exact required id must be green"; $fails++
        }
        # Duplicated PASS lines for one required id -> red (ambiguous relay).
        $dupOut = "$base`nPASS [ 0.02s] other-binary tests::$reqId"
        if (Test-Selected $dupOut "duplicate-pass" @($reqId)) {
            Write-Host "CONTROL FAILED: a duplicated required PASS must be red"; $fails++
        }
        # Duplicated required-list entry -> red (config error).
        if (Test-Selected $base "duplicate-required-list" @($reqId, $reqId)) {
            Write-Host "CONTROL FAILED: a duplicated required-list entry must be red"; $fails++
        }
        # libtest exact binding: the precise "... ok" line is green, a
        # suffixed rename is red.
        $libOk = "running 2 tests`ntest $reqId ... ok`ntest other_case ... ok`ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured"
        if (!(Test-Selected $libOk "libtest-exact" @($reqId))) {
            Write-Host "CONTROL FAILED: the exact libtest line must be green"; $fails++
        }
        if (Test-Selected $libOk.Replace("test $reqId ...", "test ${reqId}_disabled ...") "libtest-renamed" @($reqId)) {
            Write-Host "CONTROL FAILED: a renamed libtest case must be red"; $fails++
        }
        # libtest ignored -> red.
        if (Test-Selected "test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured" "ignored") {
            Write-Host "CONTROL FAILED: ignored must be red"; $fails++
        }
        # Zero selection / no summary -> red.
        if (Test-Selected "Summary [ 0.1s] 0 tests run: 0 passed" "missing") { Write-Host "CONTROL FAILED: zero-selected must be red"; $fails++ }
        if (Test-Selected "build finished, no summary matched" "no-summary") { Write-Host "CONTROL FAILED: no-summary must be red"; $fails++ }
        # R7 F-R7-02 worker-binding controls: the binding validator must
        # reject a foreign binary, a tampered receipt hash and a missing
        # exe, and accept the exact recorded build.
        $bindExe = Join-Path $env:TEMP "av827-selftest-worker.exe"
        Set-Content -Path $bindExe -Value "selftest worker bytes" -Encoding ascii
        $bindHash = (Get-FileHash $bindExe -Algorithm SHA256).Hash
        if (!(Test-WorkerBinding $bindExe $bindHash)) { Write-Host "CONTROL FAILED: the exact recorded build must be green"; $fails++ }
        $bindForeign = Join-Path $env:TEMP "av827-selftest-foreign.exe"
        Set-Content -Path $bindForeign -Value "different bytes" -Encoding ascii
        if (Test-WorkerBinding $bindForeign $bindHash) { Write-Host "CONTROL FAILED: a foreign binary must be red"; $fails++ }
        $bindFlip = if ($bindHash[0] -eq '0') { '1' } else { '0' }
        if (Test-WorkerBinding $bindExe ($bindFlip + $bindHash.Substring(1))) { Write-Host "CONTROL FAILED: a tampered receipt hash must be red"; $fails++ }
        if (Test-WorkerBinding "$($bindExe).gone" $bindHash) { Write-Host "CONTROL FAILED: a missing exe must be red"; $fails++ }
        if (Test-WorkerBinding $bindExe "") { Write-Host "CONTROL FAILED: an empty receipt hash must be red"; $fails++ }
        # R8 F-827-R8-01 retained-summary controls: the summary validator
        # must accept corroborating positives and reject missing/tampered/
        # non-corroborating artifacts. R9 tightens the gate-0 expectation to
        # the 0ms-threshold rejection of BOTH frozen hard-gate paths and adds
        # the reviewer's sibling shape (healthy threshold + unrelated failure
        # text) plus an exact-text writer control ($args automatic-variable
        # collision regression).
        $sumDir = Join-Path $env:TEMP "av827-selftest-summaries"
        New-Item -ItemType Directory -Force -Path $sumDir | Out-Null
        $mkSummary = {
            param($file, $exit, $failures, $targetsMet, $dropPeak, $gateMs)
            $s = [pscustomobject]@{
                gate_ms = $gateMs
                frontend_peak_bytes = 210501632; private_bytes_peak = 217378816
                child_private_bytes_peak = 211693568; worker_private_bytes_delta = 7819264.0
                frontend_transient_target_met = $targetsMet
                worker_private_bytes_target_met = $targetsMet
                hard_gate_failures = $failures
            }
            if ($dropPeak) { $s.PSObject.Properties.Remove('frontend_peak_bytes') }
            [pscustomobject]@{
                stamp = 'selftest'; exe_sha256 = 'A' * 64; git_head = 'deadbeef'
                exit_code = $exit; raw_file = 'bench.jsonl'; summary = $s
            } | ConvertTo-Json -Depth 6 | Out-File (Join-Path $sumDir $file) -Encoding utf8
        }
        & $mkSummary 'gate0-ok.json' 5 @(
            'hard gate: signature-conflict P90 33.5ms > 0.0ms',
            'hard gate: single-body-edit P90 495.8ms > 0.0ms'
        ) $true $false 0
        if (!(Test-BenchSummary (Join-Path $sumDir 'gate0-ok.json') $true)) { Write-Host "CONTROL FAILED: a corroborating gate-0 summary must be green"; $fails++ }
        & $mkSummary 'normal-ok.json' 0 @() $true $false 2000
        if (!(Test-BenchSummary (Join-Path $sumDir 'normal-ok.json') $false)) { Write-Host "CONTROL FAILED: a corroborating normal summary must be green"; $fails++ }
        if (Test-BenchSummary (Join-Path $sumDir 'absent.json') $false) { Write-Host "CONTROL FAILED: a missing summary file must be red"; $fails++ }
        & $mkSummary 'normal-failures.json' 0 @('hard gate: single-body-edit P90 2190.7ms > 2000.0ms') $true $false 2000
        if (Test-BenchSummary (Join-Path $sumDir 'normal-failures.json') $false) { Write-Host "CONTROL FAILED: exit 0 with recorded gate failures must be red"; $fails++ }
        & $mkSummary 'gate0-unrelated-nonzero.json' 5 @() $true $false 0
        if (Test-BenchSummary (Join-Path $sumDir 'gate0-unrelated-nonzero.json') $true) { Write-Host "CONTROL FAILED: an unrelated nonzero exit (no gate failures) must be red"; $fails++ }
        # R9 sibling (review-r9's minimal counterexample): healthy 1000ms
        # threshold + an unrelated correctness failure is NOT the gate-0
        # threshold rejection, even with exit 5 and complete peaks.
        & $mkSummary 'gate0-unrelated-failure.json' 5 @('fresh: failing samples [0]') $true $false 2000
        if (Test-BenchSummary (Join-Path $sumDir 'gate0-unrelated-failure.json') $true) { Write-Host "CONTROL FAILED: a healthy-threshold unrelated failure must be red as the gate-0 control"; $fails++ }
        # R9 sibling: the 0ms gate with a non-threshold failure text.
        & $mkSummary 'gate0-foreign-failure-text.json' 5 @('fresh: failing samples [0]') $true $false 0
        if (Test-BenchSummary (Join-Path $sumDir 'gate0-foreign-failure-text.json') $true) { Write-Host "CONTROL FAILED: a non-threshold failure text must be red as the gate-0 control"; $fails++ }
        # R9 sibling: threshold rejections of only ONE frozen path.
        & $mkSummary 'gate0-one-path.json' 5 @('hard gate: single-body-edit P90 495.8ms > 0.0ms') $true $false 0
        if (Test-BenchSummary (Join-Path $sumDir 'gate0-one-path.json') $true) { Write-Host "CONTROL FAILED: one-path-only threshold rejection must be red as the gate-0 control"; $fails++ }
        & $mkSummary 'normal-missing-peak.json' 0 @() $true $true 2000
        if (Test-BenchSummary (Join-Path $sumDir 'normal-missing-peak.json') $false) { Write-Host "CONTROL FAILED: a summary missing peak fields must be red"; $fails++ }
        # R9 writer control: the retained args text must equal the exact
        # nonempty invocation text (the `Args`-named parameter previously
        # collided with the automatic variable and wrote "").
        $writerOut = Join-Path $sumDir 'writer-args.json'
        $writerSummary = [pscustomobject]@{
            gate_ms = 0; hard_gate_failures = @('hard gate: single-body-edit P90 1.0ms > 0.0ms')
            frontend_peak_bytes = 1; private_bytes_peak = 1; child_private_bytes_peak = 1
            worker_private_bytes_delta = 1
            frontend_transient_target_met = $true; worker_private_bytes_target_met = $true
        }
        Save-BenchSummaryArtifact -Path $writerOut -Stamp 'selftest' -ExeSha256 ('C' * 64) -GitHead 'deadbeef' `
            -BenchArgs 'bench-turnaround --samples 3 --gate-ms 0 --out bench-gate0-selftest.jsonl' `
            -ExitCode 5 -RawRel 'bench-gate0-selftest.jsonl' -Summary $writerSummary -RawPath (Join-Path $sumDir 'absent.jsonl')
        $writtenArgs = (Get-Content $writerOut -Raw | ConvertFrom-Json).args
        if ($writtenArgs -cne 'bench-turnaround --samples 3 --gate-ms 0 --out bench-gate0-selftest.jsonl') {
            Write-Host "CONTROL FAILED: the writer must retain the exact nonempty args text (saw '$writtenArgs')"; $fails++
        }
        # A REAL failing child scope propagated through the same
        # Invoke-Scope path acceptance uses -> red.
        if (Invoke-Scope "fail-shim") { Write-Host "CONTROL FAILED: failing scope must be red"; $fails++ }
        if ($fails -gt 0) {
            Write-Host "FAILED: selftest ($fails controls mis-behaved)"
            exit 1
        }
        Write-Host "PASS: selftest (validator + aggregator controls: bad shapes red, filtered run green)"
    }
    "fail-shim" {
        # Selftest-only scope: a deliberately red child.
        Write-Host "fail-shim: intentional failure"
        exit 5
    }
    "incremental" {
        # V-13 (T-09): the 828 session reuse over this plan's source entry.
        Run-Probe-Tests "av_source_827_incremental" "incremental" @(
            "av_source_827_incremental_d828_baseline_30_structure_and_p95",
            "av_source_827_incremental_raw_shape_rejects_missing_baseline",
            "av_source_827_incremental_failing_verdict_still_writes_raw",
            "av_source_827_incremental_two_fn_local_updates",
            "av_source_827_incremental_precise_refusals",
            "av_source_827_incremental_degradation_caught",
            "av_source_827_incremental_detector_negative_controls"
        )
        # R5 V-13(3) binding — inherited D828 lifetime/transaction axes:
        Run-Probe-Tests "function_incremental_828_contract" "incremental-inherited-lifecycle" @(
            "contract_828_transaction_sequential_cancel_then_commit",
            "contract_828_transaction_close_releases_owned_entries",
            "contract_828_transaction_prepare_does_not_publish_and_drop_is_inert"
        )
    }
    "cache" {
        # V-07/V-08: overlay misses only the changed unit; caps re-key;
        # corrupt same-key republish conflicts (good survives); cross-
        # project content-addressed reuse; position shift = new identity.
        # Formal face: the session-overlay corpus case (new worker per
        # check, shared warm store, parent-side judgement).
        Run-Probe-Tests "av_source_827_cache" "cache" @(
            "av_source_827_overlay_misses_only_changed_unit",
            "av_source_827_cross_project_content_addressed_reuse",
            "av_source_827_caps_change_rekeys",
            "av_source_827_corrupt_republish_conflicts_and_good_survives",
            "av_source_827_position_shift_new_identity_same_result"
        )
        Run-Runner-Tests "av_source_827_corpus" "cache-corpus" @("av_source_827_corpus_green")
        # R5 V-08 binding — inherited root/lease/grant/span-rebind axes:
        Run-Probe-Tests "cache_tier" "cache-inherited-lifecycle" @(
            "av_cache_gc_and_lease",
            "av_cache_consumption_leases_cover_siblings_and_release",
            "av_cache_snapshot_manifest_lease_survives_position_edit_gc",
            "av_cache_restore_requires_current_trace_grant",
            "av_cache_position_drift_keeps_trap_span",
            "av_cache_hit_lifecycle",
            "av_cache_dependency_invalidation"
        )
        # 827 thin replays: grant revocation blocks cached dispatch; the
        # trap span re-binds to shifted COORDINATES.
        Run-Probe-Tests "av_source_827_semantics" "cache-thin-grant-span" @(
            "av_source_827_trace_grant_revocation_blocks_execution",
            "av_source_827_position_shift_rebinds_trap_span_coordinates"
        )
    }
    "compatibility" {
        # V-11: the ORIGINAL consumers keep their support faces — the ac-av
        # corpus and the delivered 817/818 recipe gates run unchanged.
        Run-Runner-Tests "ac_av_probe" "compatibility-ac-av" @("ac_av_probe_corpus_executes_green")
        Run-Runner-Tests "cte_probe" "compatibility-ac-cte" @("cte_probe_corpus_executes_green")
        Run-Runner-Tests "ac_mn_probe" "compatibility-ac-mn" @("ac_mn_probe_corpus_executes_green")
    }
}
Write-Host "PASS: scope '$Scope' green"
exit 0
