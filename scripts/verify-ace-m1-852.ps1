# verify-ace-m1-852.ps1 — PLAN-852 ACE M1 conformance corpus verifier.
#
# Scopes (built by the plan's tasks; each scope fails loudly until its task
# delivers it — a missing scope is NEVER a silent green):
#   contract   T-01: frozen baseline + binding-table integrity + design
#              package source identity (sha256) + owner coverage.
#   source     T-02: four-path scalar baseline actually runs (suite gate).
#   snapshot   T-03: daily a2r emit + golden byte-compare + certificate
#              validity check; rustc/cargo/target-process spawn count = 0.
#   certify    T-04: EXPLICIT independent certification (compile+run actual
#              and hand-written reference). Never invoked by daily scopes.
#   lifecycle  T-05: scalar cache/session fresh/hit/restart/body-edit and
#              tool containment negative controls.
#   acceptance T-06: base-layer acceptance vs M1 completeness, reported as
#              two separate conclusions.
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
                $h = (Get-FileHash $src -Algorithm SHA256).Hash.ToLower()
                if ($h -ne $c.source_sha256) { Fail "$($c.id): source hash drifted: $rel (expected $($c.source_sha256.Substring(0,12)).., got $($h.Substring(0,12))..)" }
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
        $formalSrc = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1/src'
        if (Test-Path $formalSrc) {
            foreach ($file in Get-ChildItem $formalSrc -Filter '*.at') {
                $row = $design.cases | Where-Object { $_.source -eq "src/$($file.Name)" }
                if (-not $row) { Fail "formal corpus file not in cases.json: $($file.Name)" }
                $h = (Get-FileHash $file.FullName -Algorithm SHA256).Hash.ToLower()
                if ($h -ne $row.source_sha256) { Fail "formal corpus source drifted: $($file.Name)" }
            }
            if ((Get-ChildItem $formalSrc -Filter '*.at').Count -ne $design.cases.Count) {
                Fail "formal corpus src count mismatch vs cases.json ($($design.cases.Count))"
            }
        } else {
            Fail "formal corpus src dir missing: $formalSrc"
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
            # External tools write progress to stderr; PS5.1 + Stop turns that
            # into a terminating error — relax around the call, judge by exit code.
            $ErrorActionPreference = 'Continue'
            & cargo @('build', '-p', 'auto-lang', '--features', 'test-cases', '--bin', 'auto-case', '--quiet') 2>$null
            $code = $LASTEXITCODE
            $ErrorActionPreference = 'Stop'
            if ($code -ne 0) { Fail "auto-case build failed (exit $code)" }
            Join-Path $RepoRoot 'target/debug/auto-case.exe'
        }
        $suite = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1/ace-m1-base.suite.ad'
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
        if ($r.results.Count -lt 67) { Fail "base suite check count $($r.results.Count) < 67 (checks were dropped)" }
        if ($r.coverage_gaps.Count -gt 0) { Fail "coverage gaps: $($r.coverage_gaps -join ', ')" }
        Write-Host "verify-ace-m1-852 [source]: PASS — base suite $($r.results.Count)/$($r.results.Count) checks green" `
            "(ac-native/av/autovm + legacy negatives). a2r snapshot/certification axes are separate" `
            "scopes (snapshot/certify); M1 completeness is NOT implied."
        exit 0
    }

    'snapshot' {
        if (-not $OutDir) { Write-Host "usage: -Scope snapshot requires -OutDir (scratch)" -ForegroundColor Red; exit 2 }

        # G02 structural guard: the daily a2r face emits and compares bytes
        # IN-PROCESS — the adapter must not spawn any PROCESS (in-process
        # threads are fine; process-spawn vocabulary is not).
        $adapter = Join-Path $RepoRoot 'crates/auto-lang-core/src/test_cases/a2r.rs'
        if (-not (Test-Path $adapter)) { Fail "a2r adapter missing: $adapter" }
        $spawnHits = Select-String -Path $adapter -Pattern 'Command::new|std::process|use std::os::windows::process|creation_flags'
        if ($spawnHits) { Fail "a2r adapter contains process-spawn sites (daily snapshot must spawn no process): lines $($spawnHits.LineNumber -join ',')" }

        $bin = if ($env:AUTO_CASE_BIN) { $env:AUTO_CASE_BIN } else {
            $ErrorActionPreference = 'Continue'
            & cargo @('build', '-p', 'auto-lang', '--features', 'test-cases', '--bin', 'auto-case', '--quiet') 2>$null
            $code = $LASTEXITCODE
            $ErrorActionPreference = 'Stop'
            if ($code -ne 0) { Fail "auto-case build failed (exit $code)" }
            Join-Path $RepoRoot 'target/debug/auto-case.exe'
        }
        $suite = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1/ace-m1-base.suite.ad'
        $report = Join-Path $OutDir 'ace-m1-a2r-report.json'
        # Full-suite run (a partial selection can never gate 0 by F-04 design);
        # the a2r axis is asserted from the report below.
        $ErrorActionPreference = 'Continue'
        & $bin @('run', '--suite', $suite, '--out', $OutDir, '--format', 'console', '--json-out', $report) *> $null
        $code = $LASTEXITCODE
        $ErrorActionPreference = 'Stop'
        if ($code -ne 0) { Fail "base suite gate exit $code under the snapshot scope (expected 0)" }
        $r = Get-Content $report -Raw | ConvertFrom-Json
        $a2r = @($r.results | Where-Object adapter -eq 'a2r')
        $nonPass = @($a2r | Where-Object status -ne 'PASS')
        if ($nonPass.Count -gt 0) {
            Fail ("a2r snapshot has {0} non-PASS check(s); first: {1}:{2} {3}" -f $nonPass.Count, $nonPass[0].case, $nonPass[0].check, $nonPass[0].status)
        }
        if ($a2r.Count -lt 26) { Fail "a2r snapshot check count $($a2r.Count) < 26" }

        # Two-axis honesty (G06): emit-green does NOT claim certified behavior.
        # Check each certificate's FILE-CONTENT identity axes (G05): golden,
        # oracle text, reference and source must still hash to the certified
        # values — a drift turns the certificate STALE right here (toolchain
        # axes are validated only by the explicit certify scope; the daily
        # face spawns nothing).
        $certDir = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1/oracle/certs'
        $goldens = Get-ChildItem (Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1/oracle') -Filter 'm1_*.rs'
        $certified = 0
        if (Test-Path $certDir) {
            foreach ($certFile in Get-ChildItem $certDir -Filter '*.cert.json') {
                $cert = Get-Content $certFile.FullName -Raw | ConvertFrom-Json
                $module = $cert.module
                $goldenHash = (Get-FileHash (Join-Path $certDir "..\$module.rs") -Algorithm SHA256).Hash.ToLower()
                if ($goldenHash -ne $cert.identity.golden_sha256) {
                    Fail "certificate $($certFile.Name) is STALE: golden drifted ($module)"
                }
                $refHash = (Get-FileHash (Join-Path $RepoRoot "crates/auto-lang/test/cases/ace-m1/reference/$($cert.case).rs") -Algorithm SHA256).Hash.ToLower()
                if ($refHash -ne $cert.identity.reference_sha256) {
                    Fail "certificate $($certFile.Name) is STALE: reference drifted ($($cert.case))"
                }
                $srcHash = (Get-FileHash (Join-Path $RepoRoot "crates/auto-lang/test/cases/ace-m1/src/$($cert.case).at") -Algorithm SHA256).Hash.ToLower()
                if ($srcHash -ne $cert.identity.source_sha256) {
                    Fail "certificate $($certFile.Name) is STALE: source drifted ($($cert.case))"
                }
                $oracleText = "i32:$($cert.expected_oracle -replace '^i32:', '' )"
                $oracleHashByteArray = [System.Text.Encoding]::UTF8.GetBytes($cert.expected_oracle)
                $sha = [System.Security.Cryptography.SHA256]::Create()
                $oracleHash = -join ($sha.ComputeHash($oracleHashByteArray) | ForEach-Object { $_.ToString('x2') })
                if ($oracleHash -ne $cert.identity.oracle_sha256) {
                    Fail "certificate $($certFile.Name) is STALE: oracle text identity mismatch"
                }
                $certified++
            }
        }
        Write-Host "verify-ace-m1-852 [snapshot]: PASS — $($a2r.Count) a2r checks green (emit + byte-compare," `
            "process-spawn=0 by structural guard; full base suite $($r.results.Count) checks green). Goldens: $($goldens.Count); certificates: $certified" `
            "(certified-behavior axis is the separate certify scope; emit PASS is not a behavior claim)."
        exit 0
    }

    'certify' {
        # Certification is an explicit, separate operation by contract; it is
        # never triggered implicitly by any other scope (G08).
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
        Write-Host "verify-ace-m1-852 [certify]: PASS — every ok-emit row certified or reused" `
            "(real rustc compile+run of actual and reference under containment, or an" `
            "axes-identical receipt). Receipts live in oracle/certs and are versioned."
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
        $suite = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1/ace-m1-base.suite.ad'
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
        # Tool-governance negative control: the certifier's deadline must
        # kill a hanging binary (a real rustc-compiled loop, 2s wall). The
        # filter is the FULL test path with --exact so a typo cannot
        # silently match zero tests and pass.
        $ErrorActionPreference = 'Continue'
        & cargo @('test', '-p', 'auto-lang-core', '--features', 'test-cases', 'test_cases::ace_certify::tests::wall_kills_a_hanging_binary', '--', '--exact', '--nocapture') 2>$null
        $code = $LASTEXITCODE
        $ErrorActionPreference = 'Stop'
        if ($code -ne 0) { Fail "certifier wall-kill negative control failed (exit $code)" }
        Write-Host "verify-ace-m1-852 [lifecycle]: PASS — warm/hot/body-edit/edit-persistence on av," `
            "fresh/edit+restart on mn, all judged green; certifier wall-kill control green." `
            "Native online HMR (I08) and composite/native gaps stay registered in binding.tsv."
        exit 0
    }

    'acceptance' {
        if (-not $OutDir) { Write-Host "usage: -Scope acceptance requires -OutDir (scratch)" -ForegroundColor Red; exit 2 }
        # T-06: base-layer acceptance ORCHESTRATES the five delivered scopes
        # and reports M1 completeness as a SEPARATE axis — exit 0 certifies
        # the base layer only, never "M1 is green".
        $self = $PSCommandPath
        foreach ($s in @('contract', 'certify')) {
            & $self -Scope $s | Write-Host
            if ($LASTEXITCODE -ne 0) { Fail "base-layer scope '$s' failed (exit $LASTEXITCODE)" }
        }
        foreach ($s in @('snapshot', 'lifecycle', 'source')) {
            & $self -Scope $s -OutDir (Join-Path $OutDir $s) | Write-Host
            if ($LASTEXITCODE -ne 0) { Fail "base-layer scope '$s' failed (exit $LASTEXITCODE)" }
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

        # The frozen four-path baseline must carry valid certificates.
        $certDir = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1/oracle/certs'
        foreach ($b in @('m1_b01', 'm1_b02', 'm1_b04', 'm1_b09')) {
            if (-not (Test-Path (Join-Path $certDir "$b.cert.json"))) {
                Fail "baseline certificate missing: $b.cert.json (four-path baseline needs certified a2r)"
            }
        }

        Write-Host ""
        Write-Host "==== two-axis verdict ====" -ForegroundColor Cyan
        Write-Host "BASE-LAYER (852 acceptance): PASS — contract/source/snapshot/certify/lifecycle" `
            "all green; frozen baseline B01/B02/B04/B09 four-path with valid certificates."
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
