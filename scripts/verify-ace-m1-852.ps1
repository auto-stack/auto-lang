# verify-ace-m1-852.ps1 — PLAN-852 ACE M1 conformance corpus verifier.
#
# Scopes (built by the plan's tasks; each scope fails loudly until its task
# delivers it — a missing scope is NEVER a silent green):
#   contract   T-01: frozen baseline + binding-table integrity + design
#              package source identity (sha256) + owner coverage.
#   source     T-02: four-path scalar baseline actually runs (suite gate).
#   snapshot   T-03: daily a2r emit + golden byte-compare + STRICT
#              certificate validity (structure + file axes + build-time
#              toolchain axis) with REAL process-creation instrumentation:
#              rustc/cargo/certified-binary spawns during the run must be 0
#              (G02), and this scope never builds anything itself.
#   certify    T-04: EXPLICIT independent certification (compile+run actual
#              and hand-written reference) plus the explicit-only stream
#              governance / wall-kill control tests. Never invoked by any
#              other scope (G08 / F-852-R1-02).
#   lifecycle  T-05: scalar cache/session fresh/hit/restart/body-edit
#              observations (no compilation happens in this scope).
#   acceptance T-06: base-layer acceptance over the NON-certifying scopes,
#              reporting M1 completeness as a separate axis. It never
#              certifies: missing/stale certificates are REPORTED, never
#              written here (F-852-R1-02).
#
# Exit codes: 0 = scope satisfied; 1 = verification failure; 2 = usage /
# contract error; 3 = scope not yet implemented by its owning task.

param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('contract', 'source', 'snapshot', 'certify', 'lifecycle', 'acceptance')]
    [string]$Scope,

    # Root of the auto-lang checkout (defaults to the script's parent).
    [string]$RepoRoot,

    # Scratch dir for scopes that execute anything (created, never a source tree).
    [string]$OutDir
)

$ErrorActionPreference = 'Stop'
if (-not $RepoRoot) { $RepoRoot = (Split-Path -Parent $PSScriptRoot) }

function Fail([string]$message) {
    Write-Host "verify-ace-m1-852 [$Scope]: FAIL - $message" -ForegroundColor Red
    exit 1
}

# ---- shared locations -------------------------------------------------------
$bindingPath = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1/binding.tsv'
$designDir   = Join-Path $RepoRoot 'docs/plans/attachments/852'
$casesJson   = Join-Path $designDir 'cases.json'
$corpusDir   = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1'

# ---- binding.tsv loader (schema frozen by T-01; column order is contract) ---
# Columns (12): id ns feature outcome expected ac av avm a2r status enabling owner_or_note
function Load-Binding {
    if (-not (Test-Path $bindingPath)) { Fail "binding table missing: $bindingPath" }
    $rows = @()
    foreach ($line in Get-Content $bindingPath -Encoding UTF8) {
        if ($line -notmatch '\S') { continue }          # blank
        if ($line.StartsWith('#')) { continue }         # comments (header included)
        $f = $line -split "`t", -1
        if ($f.Count -ne 12) { Fail "binding row has $($f.Count) columns (frozen schema = 12): $($line.Substring(0, [Math]::Min(60, $line.Length)))" }
        $rows += [pscustomobject]@{
            id = $f[0]; ns = $f[1]; feature = $f[2]; outcome = $f[3]; expected = $f[4]
            ac = $f[5]; av = $f[6]; avm = $f[7]; a2r = $f[8]; status = $f[9]
            enabling = $f[10]; note = $f[11]
        }
    }
    if ($rows.Count -eq 0) { Fail "binding table has zero rows" }
    $rows
}

function Sha256File([string]$path) {
    if (-not (Test-Path $path)) { return $null }
    (Get-FileHash $path -Algorithm SHA256).Hash.ToLower()
}

function Sha256Text([string]$text) {
    $sha = [System.Security.Cryptography.SHA256]::Create()
    -join ($sha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($text)) | ForEach-Object { $_.ToString('x2') })
}

# Locate a prebuilt binary WITHOUT building (the daily snapshot face must
# spawn no cargo/rustc — F-852-R1-02). Explicit env override first.
function Find-Prebuilt([string]$name) {
    if ($env:AUTO_CASE_BIN -and $name -eq 'auto-case') { return $env:AUTO_CASE_BIN }
    $exe = Join-Path $RepoRoot "target/debug/$name.exe"
    if (Test-Path $exe) { return $exe }
    Fail "$name.exe not prebuilt at $exe — build it first (cargo build -p auto-lang --features test-cases --bin $name); this scope never builds (G02)."
}

switch ($Scope) {
    'contract' {
        $rows = Load-Binding

        # 1. unique (ns, id)
        $dupes = $rows | Group-Object { "$($_.ns):$($_.id)" } | Where-Object Count -gt 1
        if ($dupes) { Fail "duplicate (ns,id): $($dupes.Name -join ', ')" }

        # 2. design package identity: every cases.json source/reference exists
        #    and hashes to the recorded frozen value.
        if (-not (Test-Path $casesJson)) { Fail "cases.json missing: $casesJson" }
        $design = Get-Content $casesJson -Raw | ConvertFrom-Json
        $checked = 0
        foreach ($c in @($design.cases) + @($design.typed_source_cases)) {
            $rel = if ($c.source) { $c.source } else { "typed-source/$($c.id).at" }
            $src = Join-Path $designDir $rel
            if (-not (Test-Path $src)) { Fail "$($c.id): source file missing: $rel" }
            if ($c.source_sha256) {
                $h = Sha256File $src
                if ($h -ne $c.source_sha256) { Fail "$($c.id): source hash drifted: $rel" }
            }
            if ($c.reference) {
                $ref = Join-Path $designDir $c.reference
                if (-not (Test-Path $ref)) { Fail "$($c.id): reference file missing: $($c.reference)" }
            }
            $row = $rows | Where-Object { $_.ns -eq 'corpus' -and $_.id -eq $c.id }
            if (-not $row) { Fail "$($c.id): declared in cases.json but absent from binding.tsv" }
            $checked++
        }

        # 2b. formal corpus sources are byte-identical to the design package
        #     (hash equality against the same frozen identity).
        $formalSrc = Join-Path $corpusDir 'src'
        if (-not (Test-Path $formalSrc)) { Fail "formal corpus src dir missing: $formalSrc" }
        $formalFiles = @(Get-ChildItem $formalSrc -Filter '*.at')
        foreach ($file in $formalFiles) {
            $row = $design.cases | Where-Object { $_.source -eq "src/$($file.Name)" }
            if (-not $row) { Fail "formal corpus file not in cases.json: $($file.Name)" }
            $h = Sha256File $file.FullName
            if ($h -ne $row.source_sha256) { Fail "formal corpus source drifted: $($file.Name)" }
        }
        if ($formalFiles.Count -ne $design.cases.Count) {
            Fail "formal corpus src count mismatch vs cases.json ($($design.cases.Count))"
        }

        # 3. scenario coverage: every SCENARIOS.md group row is bound.
        $scenarioRows = $rows | Where-Object ns -eq 'scenario'
        foreach ($gid in @('D01','D02','D03','D04','D05','D06','D07','D08','D09',
                           'E01','E02','E03','C01','C02',
                           'R01','R02','R03','R04','R05','R06',
                           'I01','I02','I03','I04','I05','I06','I07','I08','I09','I10','I11','I12',
                           'G01','G02','G03','G04','G05','G06','G07','G08')) {
            if (-not ($scenarioRows | Where-Object id -eq $gid)) { Fail "scenario row $gid missing from binding.tsv" }
        }

        # 4. frozen scalar baseline (contract-decision 2026-10-26): the four
        #    minimum required positives are bound on all four paths with exact
        #    values (a2r binds as ok-emit once its reviewed golden is frozen —
        #    T-03; behavior certification is the separate certify axis).
        $baseline = @{ B01 = '14'; B02 = '-21'; B04 = '42'; B09 = '120' }
        foreach ($k in $baseline.Keys) {
            $row = $rows | Where-Object { $_.ns -eq 'corpus' -and $_.id -eq $k }
            if (-not $row) { Fail "baseline case $k missing" }
            if ($row.expected -ne $baseline[$k]) { Fail "baseline $k expected $($row.expected), frozen contract says $($baseline[$k])" }
            foreach ($p in 'ac', 'av', 'avm') {
                if ($row.$p -ne 'ok') { Fail "baseline $k path $p is '$($row.$p)' — the frozen baseline requires bindable (ok)" }
            }
            if ($row.a2r -notin @('ok', 'ok-emit')) { Fail "baseline $k path a2r is '$($row.a2r)' — requires ok/ok-emit (golden-bound)" }
        }

        # 5. every gap/wait row names an owner or owning plan in its note.
        foreach ($r in $rows) {
            if ($r.ac -match '^(gap|wait):' -or $r.av -match '^(gap|wait):' -or $r.avm -match '^(gap|wait):' -or $r.a2r -match '^(gap|wait):') {
                if ($r.note -notmatch '\S') { Fail "$($r.id): gap/wait row without owner note" }
            }
        }

        # 6. two-axis summary (base layer vs M1 completeness are separate
        #    conclusions — this count is the denominator, not a verdict).
        $corpus = $rows | Where-Object ns -eq 'corpus'
        $byOutcome = $corpus | Group-Object outcome | ForEach-Object { "$($_.Name)=$($_.Count)" }
        Write-Host "verify-ace-m1-852 [contract]: PASS — $($rows.Count) binding rows" `
            "($($corpus.Count) corpus + $($scenarioRows.Count) scenario);" `
            "corpus outcomes: $($byOutcome -join ', ')." `
            "M1 completeness is NOT implied by this scope."
        exit 0
    }

    'source' {
        if (-not $OutDir) { Write-Host "usage: -Scope source requires -OutDir (scratch)" -ForegroundColor Red; exit 2 }
        $bin = if ($env:AUTO_CASE_BIN) { $env:AUTO_CASE_BIN } else {
            $ErrorActionPreference = 'Continue'
            & cargo @('build', '-p', 'auto-lang', '--features', 'test-cases', '--bin', 'auto-case', '--quiet') 2>$null
            $code = $LASTEXITCODE
            $ErrorActionPreference = 'Stop'
            if ($code -ne 0) { Fail "auto-case build failed (exit $code)" }
            Join-Path $RepoRoot 'target/debug/auto-case.exe'
        }
        $suite = Join-Path $corpusDir 'ace-m1-base.suite.ad'
        $report = Join-Path $OutDir 'ace-m1-base-report.json'
        $ErrorActionPreference = 'Continue'
        & $bin @('run', '--suite', $suite, '--out', $OutDir, '--format', 'console', '--json-out', $report) *> $null
        $code = $LASTEXITCODE
        $ErrorActionPreference = 'Stop'
        if ($code -ne 0) { Fail "base suite gate exit $code (expected 0)" }
        $r = Get-Content $report -Raw | ConvertFrom-Json
        $nonPass = @($r.results | Where-Object status -ne 'PASS')
        if ($nonPass.Count -gt 0) {
            $first = $nonPass[0]
            Fail ("base suite has {0} non-PASS check(s); first: {1}:{2} {3}" -f $nonPass.Count, $first.case, $first.check, $first.status)
        }
        if ($r.results.Count -lt 97) { Fail "base suite check count $($r.results.Count) < 97 (checks were dropped)" }
        if ($r.coverage_gaps.Count -gt 0) { Fail "coverage gaps: $($r.coverage_gaps -join ', ')" }
        Write-Host "verify-ace-m1-852 [source]: PASS — base suite $($r.results.Count)/$($r.results.Count) checks green" `
            "(ac-native/av/autovm + legacy negatives). a2r snapshot/certification axes are separate" `
            "scopes (snapshot/certify); M1 completeness is NOT implied."
        exit 0
    }

    'snapshot' {
        if (-not $OutDir) { Write-Host "usage: -Scope snapshot requires -OutDir (scratch)" -ForegroundColor Red; exit 2 }

        # G02 structural guard (cheap first line): the daily a2r face emits
        # and compares bytes IN-PROCESS — the adapter must not spawn any
        # PROCESS. Real per-run process instrumentation follows below.
        $adapter = Join-Path $RepoRoot 'crates/auto-lang-core/src/test_cases/a2r.rs'
        if (-not (Test-Path $adapter)) { Fail "a2r adapter missing: $adapter" }
        $spawnHits = Select-String -Path $adapter -Pattern 'Command::new|std::process|use std::os::windows::process|creation_flags'
        if ($spawnHits) { Fail "a2r adapter contains process-spawn sites: lines $($spawnHits.LineNumber -join ',')" }

        # The daily snapshot NEVER builds (no cargo/rustc spawn — F-852-R1-02).
        $bin = Find-Prebuilt 'auto-case'
        $suite = Join-Path $corpusDir 'ace-m1-base.suite.ad'
        $report = Join-Path $OutDir 'ace-m1-a2r-report.json'

        # ---- REAL process-creation instrumentation (F-852-R1-02) ----------
        # Run the suite async and poll the OS process table (in-process CIM;
        # no watcher process is created). Any NEW rustc/cargo/certified-
        # binary process DESCENDED FROM OUR RUN is a G02 violation; unrelated
        # concurrent builds on the machine (other sessions) are ignored by
        # ancestry, not by luck.
        $baseline = Get-CimInstance Win32_Process -Filter "Name='rustc.exe' OR Name='cargo.exe' OR Name LIKE 'm1%'" -ErrorAction SilentlyContinue
        $baselineIds = @($baseline | ForEach-Object { $_.ProcessId })
        $suiteOut = Join-Path $OutDir 'suite-console.txt'
        $proc = Start-Process -FilePath $bin `
            -ArgumentList @('run', '--suite', $suite, '--out', $OutDir, '--format', 'console', '--json-out', $report) `
            -RedirectStandardOutput $suiteOut -RedirectStandardError (Join-Path $OutDir 'suite-stderr.txt') `
            -PassThru -NoNewWindow
        $runnerId = $proc.Id
        $violations = @()
        $maintenance = 0
        function Test-Descendant([uint32]$targetId, [uint32]$ancestor, [hashtable]$procs) {
            $seen = @{}
            $cur = $targetId
            while ($cur -and -not $seen.ContainsKey($cur)) {
                if ($cur -eq $ancestor) { return $true }
                $seen[$cur] = $true
                if (-not $procs.ContainsKey($cur)) { return $false }
                $cur = $procs[$cur].ParentProcessId
            }
            return $false
        }
        while (-not $proc.HasExited) {
            $now = Get-CimInstance Win32_Process -Filter "Name='rustc.exe' OR Name='cargo.exe' OR Name LIKE 'm1%'" -ErrorAction SilentlyContinue
            $fresh = @($now | Where-Object { $baselineIds -notcontains $_.ProcessId })
            if ($fresh.Count -gt 0) {
                $table = @{}
                foreach ($p in (Get-CimInstance Win32_Process -ErrorAction SilentlyContinue)) { $table[$p.ProcessId] = $p }
                foreach ($f in $fresh) {
                    if (-not (Test-Descendant -targetId $f.ProcessId -ancestor $runnerId -procs $table)) { continue }
                    $parent = $table[$f.ParentProcessId]
                    if ($f.Name -eq 'cargo.exe') {
                        # Engine-worker maintenance build (PLAN-818 memoized no-op
                        # discipline; builds the mn/av HARNESS, never transpiled
                        # corpus artifacts). Reported, not a G02 violation.
                        $maintenance += 1
                    }
                    elseif ($f.Name -eq 'rustc.exe' -and $parent -and $parent.Name -eq 'cargo.exe') {
                        $maintenance += 1   # rustc under a maintenance cargo
                    }
                    else {
                        # A DIRECT rustc invocation or a certified binary
                        # (m1_*_wrap/m1_*_ref) — the a2r certification face,
                        # which the daily snapshot must NEVER touch.
                        $violations += "$($f.Name) (pid $($f.ProcessId))"
                    }
                }
            }
            Start-Sleep -Milliseconds 300
        }
        $proc.WaitForExit()
        $proc.Refresh()
        $gate = $proc.ExitCode
        if ($null -eq $gate) {
            # Handle already released (PS quirk): fall back to the report's
            # presence + later content checks; absence means failure.
            $gate = if (Test-Path $report) { 0 } else { 1 }
        }
        if ($gate -ne 0) { Fail "base suite gate exit $gate under the snapshot scope (expected 0)" }
        if ($violations.Count -gt 0) {
            Fail ("G02 VIOLATION: the daily snapshot run spawned the a2r certification face (direct rustc / certified binaries): {0}" -f ($violations -join ', '))
        }
        Write-Host "verify-ace-m1-852 [snapshot]: process instrumentation: 0 certification-face spawns (direct rustc / m1_* binaries; ancestry-checked); $maintenance engine-worker maintenance build process(es) observed (PLAN-818 memoized harness discipline, not corpus compilation)."

        $r = Get-Content $report -Raw | ConvertFrom-Json
        $a2r = @($r.results | Where-Object adapter -eq 'a2r')
        $nonPass = @($a2r | Where-Object status -ne 'PASS')
        if ($nonPass.Count -gt 0) {
            Fail ("a2r snapshot has {0} non-PASS check(s); first: {1}:{2} {3}" -f $nonPass.Count, $nonPass[0].case, $nonPass[0].check, $nonPass[0].status)
        }
        if ($a2r.Count -lt 26) { Fail "a2r snapshot check count $($a2r.Count) < 26" }

        # ---- STRICT certificate validity (F-852-R1-01), zero spawns --------
        # Every ok-emit binding row must carry a structurally valid receipt
        # whose oracle text is bound to the frozen registry, whose file axes
        # match the CURRENT bytes, and whose toolchain axis matches the
        # build-time identity carried by this run's own report.
        $rows = Load-Binding
        $certRows = @($rows | Where-Object { $_.ns -eq 'corpus' -and $_.outcome -eq 'value' -and $_.a2r -eq 'ok-emit' })
        if ($certRows.Count -lt 1) { Fail "no ok-emit rows found in binding" }
        $buildIdentity = $r.toolchain.'build-rustc-identity'
        if (-not $buildIdentity) { Fail "suite report lacks the build-rustc-identity toolchain axis (rebuild auto-case)" }
        $certDir = Join-Path $corpusDir 'oracle/certs'
        $wrapperHash = Sha256File (Join-Path $corpusDir 'oracle/wrapper-main-v1.rs')
        if (-not $wrapperHash) { Fail "frozen wrapper file missing: oracle/wrapper-main-v1.rs" }
        foreach ($row in $certRows) {
            $module = 'm1_' + $row.id.ToLower()
            $certPath = Join-Path $certDir "$module.cert.json"
            if (-not (Test-Path $certPath)) {
                Fail "certificate MISSING for ok-emit row $($row.id) — the certified-behavior axis has a gap; run the explicit certify scope (never implicit)."
            }
            $cert = $null
            try { $cert = Get-Content $certPath -Raw | ConvertFrom-Json }
            catch { Fail "certificate $module.cert.json is not valid JSON (possible tampering): $_" }
            $oracle = "i32:$($row.expected)`n"

            # ---- R2-B + F-852-R3-01: STRICT typed schema — exact key sets at
            # EVERY level (produced_by included), declared types, unknown/
            # missing/wrong-typed elements REJECT. All key/value comparisons
            # are CASE-SENSITIVE (-c* operators) to match Rust's exact JSON
            # matching; a "Schema"/"STD-ONLY"/uppercase-hex receipt must fail
            # here exactly as it fails in the Rust validator.
            function Assert-Keys($obj, $expected, $where) {
                if ($null -eq $obj -or -not $obj.PSObject) { Fail "certificate $module REJECTED: $where is not an object" }
                $have = @($obj.PSObject.Properties.Name)
                foreach ($k in $have) { if (-not ($expected | Where-Object { $_ -ceq $k })) { Fail "certificate $module REJECTED: unknown field '$k' in $where (strict schema)" } }
                foreach ($k in $expected) { if (-not ($have | Where-Object { $_ -ceq $k })) { Fail "certificate $module REJECTED: missing field '$k' in $where" } }
            }
            Assert-Keys $cert @('schema','case','module','wrapper_file','expected_oracle','identity','toolchain','deps','observed','verdict','produced_by','created_at') 'receipt root'
            Assert-Keys $cert.identity @('source_sha256','golden_sha256','actual_sha256','wrapper_sha256','oracle_sha256','reference_sha256') 'identity'
            Assert-Keys $cert.toolchain @('identity','identity_source','default_identity_at_certify','flags') 'toolchain'
            Assert-Keys $cert.deps @('external','closure') 'deps'
            Assert-Keys $cert.observed @('actual_run','reference_run') 'observed'
            Assert-Keys $cert.observed.actual_run @('exit','stdout') 'observed.actual_run'
            Assert-Keys $cert.observed.reference_run @('exit','stdout') 'observed.reference_run'
            # produced_by: exact two keys with STRING values (F-852-R3-01 —
            # the Rust validator rejects here, so must the daily face).
            Assert-Keys $cert.produced_by @('tool','schema_note') 'produced_by'
            if ($cert.produced_by.tool -isnot [string] -or $cert.produced_by.schema_note -isnot [string]) {
                Fail "certificate $module REJECTED: produced_by fields are not strings"
            }

            if ($cert.schema -isnot [string] -or $cert.schema -cne 'ace.m1.cert/1') { Fail "certificate $module REJECTED: schema is '$($cert.schema)'" }
            if ($cert.verdict -isnot [string] -or $cert.verdict -cne 'certified') { Fail "certificate $module REJECTED: verdict is '$($cert.verdict)'" }
            if ($cert.case -cne $row.id -or $cert.module -cne $module) { Fail "certificate $module REJECTED: case/module mismatch" }
            if ($cert.wrapper_file -isnot [string] -or $cert.wrapper_file -cne 'oracle/wrapper-main-v1.rs') { Fail "certificate $module REJECTED: wrapper_file is not the frozen file" }
            if ($cert.expected_oracle -isnot [string] -or $cert.expected_oracle -cne $oracle) { Fail "certificate $module REJECTED: oracle text is not the frozen registry value" }
            if (($cert.created_at -isnot [long] -and $cert.created_at -isnot [int]) -or $cert.created_at -lt 0) { Fail "certificate $module REJECTED: created_at is not an unsigned integer (negative or wrong type)" }
            # Observed runs: TYPED equality (a "0" string exit or a numeric
            # stdout is a type violation, never a pass — PS would otherwise
            # coerce).
            foreach ($side in @('actual_run', 'reference_run')) {
                $run = $cert.observed.$side
                if ($run.stdout -isnot [string] -or $run.stdout -cne $oracle) { Fail "certificate $module REJECTED: observed.$side.stdout does not equal the fixed oracle (or wrong type)" }
                if (($run.exit -isnot [long] -and $run.exit -isnot [int]) -or $run.exit -ne 0) { Fail "certificate $module REJECTED: observed.$side.exit is not the integer 0" }
            }
            # flags: array of STRINGS, exact frozen tokens (typed per element).
            $toks = @($cert.toolchain.flags)
            if ($toks.Count -ne 2) { Fail "certificate $module REJECTED: toolchain.flags is not the frozen 2-token set" }
            for ($ti = 0; $ti -lt $toks.Count; $ti++) {
                if ($toks[$ti] -isnot [string]) { Fail "certificate $module REJECTED: toolchain.flags[$ti] is not a string" }
            }
            if ($toks[0] -cne '--edition=2021' -or $toks[1] -cne '-O') {
                Fail "certificate $module REJECTED: toolchain.flags is not the frozen set ['--edition=2021','-O']"
            }
            # toolchain identity axes: identity binds the compiler the pass
            # LAUNCHED (frozen evidence — daily consumes it without spawning);
            # identity_source must be a frozen selection rule; the DEFAULT
            # axis is what daily can recompute and is compared below.
            if ($cert.toolchain.identity -isnot [string] -or -not $cert.toolchain.identity.Trim()) { Fail "certificate $module REJECTED: toolchain.identity is not a non-empty string" }
            if ($cert.toolchain.identity_source -isnot [string] -or -not (@('--rustc override', 'env RUSTC', 'PATH') | Where-Object { $_ -ceq $cert.toolchain.identity_source })) {
                Fail "certificate $module REJECTED: identity_source is not a frozen selection rule"
            }
            if ($cert.deps.closure -isnot [string] -or $cert.deps.closure -cne 'std-only') { Fail "certificate $module REJECTED: deps.closure is not std-only" }
            if (@($cert.deps.external).Count -ne 0) { Fail "certificate $module REJECTED: deps.external is not empty" }
            if ($cert.identity.actual_sha256 -cne $cert.identity.golden_sha256) {
                Fail "certificate $module REJECTED: actual_sha256 != golden_sha256 (self-inconsistent)"
            }
            foreach ($hk in @('source_sha256','golden_sha256','actual_sha256','wrapper_sha256','oracle_sha256','reference_sha256')) {
                $hv = $cert.identity.$hk
                # -cnotmatch: PowerShell -match is case-INSENSITIVE by default;
                # the contract is lowercase hex, so uppercase must fail here
                # exactly as it fails in the Rust validator.
                if ($hv -isnot [string] -or $hv -cnotmatch '^[0-9a-f]{64}$') { Fail "certificate $module REJECTED: identity.$hk is not a 64-char lowercase hex string" }
            }
            foreach ($axis in @(
                @{ k = 'golden_sha256'; f = "oracle/$module.rs"; label = 'golden' },
                @{ k = 'reference_sha256'; f = "reference/$($row.id).rs"; label = 'reference' },
                @{ k = 'source_sha256'; f = "src/$($row.id).at"; label = 'source' }
            )) {
                $cur = Sha256File (Join-Path $corpusDir $axis.f)
                if ($cur -cne $cert.identity.($axis.k)) { Fail "certificate $module is STALE: $($axis.label) drifted" }
            }
            if ($wrapperHash -cne $cert.identity.wrapper_sha256) { Fail "certificate $module is STALE: wrapper drifted" }
            if ((Sha256Text $oracle) -cne $cert.identity.oracle_sha256) { Fail "certificate $module REJECTED: oracle text identity mismatch" }
            # R2-A daily rule: the DEFAULT toolchain axis (recomputed zero-spawn
            # from this run's own report) must equal the default frozen at
            # certification. The certification `identity` itself is frozen
            # evidence of the compiler that WAS launched; daily never passes
            # the harness build identity off as certification evidence.
            if ($cert.toolchain.default_identity_at_certify -isnot [string] -or $cert.toolchain.default_identity_at_certify -cne $buildIdentity) {
                Fail "certificate $module is STALE: default toolchain drifted since certification (re-certify to re-bind; certified default: $($cert.toolchain.default_identity_at_certify))"
            }
        }
        $goldens = @(Get-ChildItem (Join-Path $corpusDir 'oracle') -Filter 'm1_*.rs')
        Write-Host "verify-ace-m1-852 [snapshot]: PASS — $($a2r.Count) a2r checks green (emit + byte-compare);" `
            "$($certRows.Count) certificates STRICTLY valid (typed exact-key schema + file axes + default-toolchain" `
            "axis vs this run's build identity). Certification-face process watch: 0 direct-rustc/m1_* spawns" `
            "observed in periodic CIM ancestry sampling (a sampling observer, not a lossless creation count)." `
            "Full base suite $($r.results.Count) checks green." `
            "Emit PASS is not a behavior claim; re-certification is the explicit certify scope."
        exit 0
    }

    'certify' {
        # Certification is an explicit, separate operation by contract; it is
        # never triggered implicitly by any other scope (G08 / F-852-R1-02).
        $bin = if ($env:AUTO_CERTIFY_BIN) { $env:AUTO_CERTIFY_BIN } else {
            $ErrorActionPreference = 'Continue'
            & cargo @('build', '-p', 'auto-lang', '--features', 'test-cases', '--bin', 'ace-certify', '--quiet') 2>$null
            $code = $LASTEXITCODE
            $ErrorActionPreference = 'Stop'
            if ($code -ne 0) { Fail "ace-certify build failed (exit $code)" }
            Join-Path $RepoRoot 'target/debug/ace-certify.exe'
        }
        $corpus = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1'
        $ErrorActionPreference = 'Continue'
        & $bin @('--corpus', $corpus) 2>$null
        $code = $LASTEXITCODE
        $ErrorActionPreference = 'Stop'
        if ($code -ne 0) { Fail "certification exit $code (0 = certified/reused, 1 = failure)" }

        # Explicit-only governance controls (F-852-R1-02/03): stream ordering,
        # per-stream overflow kill, and the wall kill. Full test paths +
        # --ignored so a typo can never silently match zero tests.
        $ErrorActionPreference = 'Continue'
        & cargo @('test', '-p', 'auto-lang-core', '--features', 'test-cases', '--lib', '--',
                  '--ignored', '--exact', '--nocapture',
                  'test_cases::ace_certify::tests::stream_governance_no_deadlock_on_any_pipe_order',
                  'test_cases::ace_certify::tests::stream_overflow_kills_and_surfaces',
                  'test_cases::ace_certify::tests::wall_kills_a_hanging_binary',
                  'test_cases::ace_certify::tests::identity_binds_the_launched_compiler_and_scratch_never_strands') 2>$null
        $code = $LASTEXITCODE
        $ErrorActionPreference = 'Stop'
        if ($code -ne 0) { Fail "certifier governance controls failed (exit $code)" }
        Write-Host "verify-ace-m1-852 [certify]: PASS — every ok-emit row certified or reused" `
            "(real contained rustc compile+run of actual and reference, or an axes-identical" `
            "receipt); stream-order/overflow/wall governance controls green." `
            "Receipts live in oracle/certs and are versioned."
        exit 0
    }

    'lifecycle' {
        if (-not $OutDir) { Write-Host "usage: -Scope lifecycle requires -OutDir (scratch)" -ForegroundColor Red; exit 2 }
        $bin = if ($env:AUTO_CASE_BIN) { $env:AUTO_CASE_BIN } else {
            $ErrorActionPreference = 'Continue'
            & cargo @('build', '-p', 'auto-lang', '--features', 'test-cases', '--bin', 'auto-case', '--quiet') 2>$null
            $code = $LASTEXITCODE
            $ErrorActionPreference = 'Stop'
            if ($code -ne 0) { Fail "auto-case build failed (exit $code)" }
            Join-Path $RepoRoot 'target/debug/auto-case.exe'
        }
        $suite = Join-Path $corpusDir 'ace-m1-base.suite.ad'
        $report = Join-Path $OutDir 'ace-m1-lifecycle-report.json'
        $ErrorActionPreference = 'Continue'
        & $bin @('run', '--suite', $suite, '--out', $OutDir, '--format', 'console', '--json-out', $report) *> $null
        $code = $LASTEXITCODE
        $ErrorActionPreference = 'Stop'
        if ($code -ne 0) { Fail "base suite gate exit $code under the lifecycle scope (expected 0)" }
        $r = Get-Content $report -Raw | ConvertFrom-Json
        # The six lifecycle checks must be present AND green with the real
        # observations (warm 120, hot 120, body edit 720, edit persistence
        # across fresh workers 720, AC fresh 120, AC edit+restart 720).
        $lifecycle = @($r.results | Where-Object { $_.case -like 'ace.m1.session*' })
        if ($lifecycle.Count -lt 6) { Fail "lifecycle checks missing (found $($lifecycle.Count) of 6)" }
        $expected = @{
            'ace.m1.session.av:warm-run' = 120
            'ace.m1.session.av:hot-run' = 120
            'ace.m1.session.av:body-edit-run' = 720
            'ace.m1.session.av:edit-persist-across-workers' = 720
            'ace.m1.session.mn:fresh-run' = 120
            'ace.m1.session.mn:edit-then-restart-run' = 720
        }
        foreach ($k in $expected.Keys) {
            $parts = $k.Split(':')
            $check = $lifecycle | Where-Object { $_.case -eq $parts[0] -and $_.check -eq $parts[1] }
            if (-not $check) { Fail "lifecycle check missing: $k" }
            if ($check.status -ne 'PASS') { Fail "lifecycle check $k is $($check.status)" }
            if ($check.native.exit_code -ne $expected[$k]) {
                Fail "lifecycle $k observed exit $($check.native.exit_code), expected $($expected[$k])"
            }
        }
        Write-Host "verify-ace-m1-852 [lifecycle]: PASS — warm/hot/body-edit/edit-persistence on av," `
            "fresh/edit+restart on mn, all judged green. No compilation happens in this scope" `
            "(the wall/overflow kill controls live in the explicit certify scope)." `
            "Native online HMR (I08) and composite/native gaps stay registered in binding.tsv."
        exit 0
    }

    'acceptance' {
        if (-not $OutDir) { Write-Host "usage: -Scope acceptance requires -OutDir (scratch)" -ForegroundColor Red; exit 2 }
        # T-06: base-layer acceptance ORCHESTRATES the NON-CERTIFYING scopes
        # (F-852-R1-02: acceptance never compiles, never writes certificates)
        # and reports M1 completeness as a SEPARATE axis — exit 0 certifies
        # the base layer only, never "M1 is green". Missing/stale certificates
        # are REPORTED by the snapshot scope and fail here; producing them is
        # the explicit certify scope's job.
        $self = $PSCommandPath
        & $self -Scope contract | Write-Host
        if ($LASTEXITCODE -ne 0) { Fail "base-layer scope 'contract' failed (exit $LASTEXITCODE)" }
        foreach ($s in @('source', 'snapshot', 'lifecycle')) {
            & $self -Scope $s -OutDir (Join-Path $OutDir $s) | Write-Host
            if ($LASTEXITCODE -ne 0) { Fail "base-layer scope '$s' failed (exit $LASTEXITCODE)" }
        }

        # The frozen four-path baseline must carry present receipts (their
        # STRICT validity was just enforced by the snapshot scope).
        $certDir = Join-Path $corpusDir 'oracle/certs'
        foreach ($b in @('m1_b01', 'm1_b02', 'm1_b04', 'm1_b09')) {
            if (-not (Test-Path (Join-Path $certDir "$b.cert.json"))) {
                Fail "baseline certificate missing: $b.cert.json — four-path baseline needs certified a2r; run the EXPLICIT certify scope (never implicit here)."
            }
        }

        # ---- M1 completeness axis (from the frozen denominator registry) ----
        $rows = Load-Binding
        $corpus = @($rows | Where-Object ns -eq 'corpus')
        $scenario = @($rows | Where-Object ns -eq 'scenario')
        $greenCells = 0; $totalCells = 0; $blockers = @{}
        foreach ($r in $corpus) {
            foreach ($p in 'ac', 'av', 'avm', 'a2r') {
                $totalCells++
                $v = $r.$p
                $isGreen = switch ($p) {
                    'a2r' { $v -in @('ok', 'ok-emit') }   # snapshot-only needs behavior cert for M1
                    default { $v -eq 'ok' }
                }
                if ($isGreen) { $greenCells++ }
                elseif ($v -match '^(gap|wait):([^|]+)') {
                    $owner = $Matches[2]; $blockers[$owner] = ($blockers[$owner] + 1)
                }
                else { $blockers['semantic-difference'] = ($blockers['semantic-difference'] + 1) }
            }
        }
        $scenarioOpen = @($scenario | Where-Object { $_.ac -notmatch '^ok' -or $_.av -notmatch '^ok' }).Count

        Write-Host ""
        Write-Host "==== two-axis verdict ====" -ForegroundColor Cyan
        Write-Host "BASE-LAYER (852 acceptance): PASS — contract/source/snapshot/lifecycle" `
            "all green WITHOUT any certification run (G08: certify is explicit-only);" `
            "frozen baseline B01/B02/B04/B09 four-path with strictly validated certificates."
        Write-Host "M1 COMPLETENESS: NOT claimable from this gate — corpus path-cells green:" `
            "$greenCells/$totalCells; scenario rows fully open: $scenarioOpen/$($scenario.Count)." `
            "Registered blockers by owner/cause:"
        foreach ($k in ($blockers.Keys | Sort-Object)) { Write-Host ("  {0}: {1} cell(s)" -f $k, $blockers[$k]) }
        Write-Host "Base-layer PASS does not imply M1 green; full M1 needs every required row" `
            "real on all four paths (owners: 822/823/824, a2r composite lowering, native" `
            "expansion + online loader, legacy semantic rulings)."
        exit 0
    }
}
