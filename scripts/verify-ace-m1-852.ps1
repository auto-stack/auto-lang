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
        #    minimum required positives are all-path bindable with exact values.
        $baseline = @{ B01 = '14'; B02 = '-21'; B04 = '42'; B09 = '120' }
        foreach ($k in $baseline.Keys) {
            $row = $rows | Where-Object { $_.ns -eq 'corpus' -and $_.id -eq $k }
            if (-not $row) { Fail "baseline case $k missing" }
            if ($row.expected -ne $baseline[$k]) { Fail "baseline $k expected $($row.expected), frozen contract says $($baseline[$k])" }
            foreach ($p in 'ac', 'av', 'avm', 'a2r') {
                if ($row.$p -ne 'ok') { Fail "baseline $k path $p is '$($row.$p)' — the frozen baseline requires bindable (ok) on all four paths" }
            }
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
        Write-Host "verify-ace-m1-852 [snapshot]: FAIL — scope owned by T-03, not implemented yet" -ForegroundColor Red
        exit 3
    }

    'certify' {
        # Certification is an explicit, separate operation by contract; it is
        # never triggered implicitly by any other scope.
        Write-Host "verify-ace-m1-852 [certify]: FAIL — scope owned by T-04, not implemented yet" -ForegroundColor Red
        exit 3
    }

    'lifecycle' {
        Write-Host "verify-ace-m1-852 [lifecycle]: FAIL — scope owned by T-05, not implemented yet" -ForegroundColor Red
        exit 3
    }

    'acceptance' {
        Write-Host "verify-ace-m1-852 [acceptance]: FAIL — scope owned by T-06, not implemented yet" -ForegroundColor Red
        exit 3
    }
}
