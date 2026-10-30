# verify-av-862.ps1 — PLAN-862 entry/Unit source profile + M1 rebind verifier.
#
# Scopes (each fails loudly; a scope that is not wired exits 3, never green):
#   entry      the entry-unit corpus (U01..U10) through the REAL
#              `ac-av/core-run-v1` recipe with the append-only
#              `av: { profile: "entry-unit" }` selector, judged parent-side;
#              includes the discrimination control (a sabotaged expectation
#              MUST redden exactly the Unit rows) and the per-row channel /
#              exit / toolchain assertions.
#   compat     the old ACE M1 base suite through the same runner — the
#              ZERO-DRIFT control for the scalar profile after PLAN-862's
#              contract/judge additions (the old case files must keep their
#              exact green set).
#   targeted   the scoped engine checks the plan names for T-05: the parent
#              judge/contract unit tests, the ac-core entry matrix tests and
#              the probe entry-unit round (SPEC/contract plumbing).
#   acceptance the rebind accounting over `binding.tsv`: integrity (12-column
#              schema, unique ids, owner notes, `na:m1` rule ids) plus the
#              five-state distribution and the measured `clean` count. It
#              runs entry+compat+targeted and NEVER claims "M1 green".
#
# Exit codes: 0 = scope satisfied; 1 = verification failure; 2 = usage /
# contract error; 3 = scope not wired yet.

param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('entry', 'compat', 'targeted', 'acceptance')]
    [string]$Scope,

    # Root of the auto-lang checkout (defaults to the script's parent).
    [string]$RepoRoot,

    # Scratch dir for scopes that execute anything.
    [string]$OutDir
)
. (Join-Path $PSScriptRoot 'test-governance/ace-cargo.ps1')


$ErrorActionPreference = 'Stop'
if (-not $RepoRoot) { $RepoRoot = (Split-Path -Parent $PSScriptRoot) }

function Fail([string]$message) {
    Write-Host "verify-av-862 [$Scope]: FAIL - $message" -ForegroundColor Red
    exit 1
}

$corpusDir = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1'
$entryDir = Join-Path $corpusDir 'entry-862'
$entrySuite = Join-Path $entryDir 'entry-unit-862.suite.ad'
$baseSuite = Join-Path $corpusDir 'ace-m1-base.suite.ad'
$bindingPath = Join-Path $corpusDir 'binding.tsv'

# ---- binding.tsv loader (schema frozen by PLAN-852 T-01: 12 columns) -------
function Load-Binding {
    if (-not (Test-Path $bindingPath)) { Fail "binding table missing: $bindingPath" }
    $rows = @()
    foreach ($line in Get-Content $bindingPath -Encoding UTF8) {
        if ($line -notmatch '\S') { continue }
        if ($line.StartsWith('#')) { continue }
        $f = $line.Split([char]9)
        if ($f.Count -ne 12) { Fail "binding row has $($f.Count) columns (frozen schema = 12)" }
        $rows += [pscustomobject]@{
            id = $f[0]; ns = $f[1]; feature = $f[2]; outcome = $f[3]; expected = $f[4]
            ac = $f[5]; av = $f[6]; avm = $f[7]; a2r = $f[8]; status = $f[9]
            enabling = $f[10]; note = $f[11]
        }
    }
    if ($rows.Count -eq 0) { Fail "binding table has zero rows" }
    $rows
}

# Locate the prebuilt runner WITHOUT building it (the corpus gate consumes a
# built binary; building is the caller's explicit action).
function Find-Prebuilt([string]$name) {
    if ($env:AUTO_CASE_BIN -and $name -eq 'auto-case') { return $env:AUTO_CASE_BIN }
    $exe = Join-Path $RepoRoot "target/debug/$name.exe"
    if (Test-Path $exe) { return $exe }
    Fail "$name.exe not prebuilt at $exe (build it first: Invoke-AceCargo build -p auto-lang --features test-cases --bin $name)"
}

function Invoke-Suite([string]$suite, [string]$outDir, [string]$label) {
    $bin = Find-Prebuilt 'auto-case'
    New-Item -ItemType Directory -Force $outDir | Out-Null
    $report = Join-Path $outDir "$label-report.json"
    $ErrorActionPreference = 'Continue'
    & $bin @('run', '--suite', $suite, '--out', $outDir, '--format', 'console', '--json-out', $report) *> $null
    $code = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
    if (-not (Test-Path $report)) { Fail "${label}: the runner wrote no report (exit $code)" }
    $r = Get-Content $report -Raw | ConvertFrom-Json
    [pscustomobject]@{ exit = $code; report = $r; path = $report }
}

function Get-Toolchain($check) {
    $h = @{}
    if ($check.native -and $check.native.toolchain) {
        foreach ($pair in @($check.native.toolchain)) {
            if (@($pair).Count -ge 2) { $h[[string]$pair[0]] = [string]$pair[1] }
        }
    }
    $h
}

switch ($Scope) {
    'entry' {
        if (-not $OutDir) { Write-Host "usage: -Scope entry requires -OutDir (scratch)" -ForegroundColor Red; exit 2 }
        if (-not (Test-Path $entrySuite)) { Fail "entry suite missing: $entrySuite" }

        $run = Invoke-Suite $entrySuite $OutDir 'entry'
        if ($run.exit -ne 0) {
            $nonPass = @($run.report.results | Where-Object status -ne 'PASS')
            Fail ("entry suite exit {0}; {1} non-PASS check(s); first: {2}:{3} {4}" -f `
                $run.exit, $nonPass.Count, $nonPass[0].case, $nonPass[0].check, $nonPass[0].status)
        }
        if ($run.report.results.Count -lt 12) {
            Fail "entry suite ran $($run.report.results.Count) checks (< 12: checks were dropped)"
        }
        if (@($run.report.coverage_gaps).Count -gt 0) {
            Fail "entry suite coverage gaps: $($run.report.coverage_gaps -join ', ')"
        }

        # Per-row assertions: the channel/exit/toolchain TRIPLE is the oracle
        # (a Unit row that reported the value channel, or a value row that
        # reported the unit channel, is red here even if some other check
        # happened to pass).
        $expectRows = @(
            @{ case = 'ace.m1.u01'; check = 'av'; channel = 'value'; exit = 42; core = 'av.entry-unit/1'; profile = '7' },
            @{ case = 'ace.m1.u01'; check = 'av-scalar-control'; channel = 'value'; exit = 42; core = 'av.core-i32/1'; profile = $null },
            @{ case = 'ace.m1.u02'; check = 'av'; channel = 'unit'; exit = $null; core = 'av.entry-unit/1'; profile = '7' },
            @{ case = 'ace.m1.u02'; check = 'av-scalar-rejects'; channel = $null; exit = $null; core = $null; profile = $null },
            @{ case = 'ace.m1.u03'; check = 'av'; channel = 'value'; exit = 42; core = 'av.entry-unit/1'; profile = '7' },
            @{ case = 'ace.m1.u04'; check = 'av'; channel = 'unit'; exit = $null; core = 'av.entry-unit/1'; profile = '7' },
            @{ case = 'ace.m1.u05'; check = 'av'; channel = $null; exit = 70; core = 'av.entry-unit/1'; profile = '7' },
            @{ case = 'ace.m1.u06'; check = 'av'; channel = $null; exit = $null; core = 'av.entry-unit/1'; profile = '7' },
            @{ case = 'ace.m1.u07'; check = 'av'; channel = $null; exit = $null; core = 'av.entry-unit/1'; profile = '7' },
            @{ case = 'ace.m1.u08'; check = 'av'; channel = $null; exit = $null; core = 'av.entry-unit/1'; profile = '7' },
            @{ case = 'ace.m1.u09'; check = 'av'; channel = $null; exit = $null; core = 'av.entry-unit/1'; profile = '7' },
            @{ case = 'ace.m1.u10'; check = 'av'; channel = $null; exit = $null; core = 'av.entry-unit/1'; profile = '7' }
        )
        foreach ($want in $expectRows) {
            $got = @($run.report.results | Where-Object { $_.case -eq $want.case -and $_.check -eq $want.check })
            if ($got.Count -ne 1) { Fail "$($want.case)/$($want.check): expected exactly one result, found $($got.Count)" }
            $tc = Get-Toolchain $got[0]
            if ($want.channel) {
                if ($got[0].detail -notmatch "channel $($want.channel)\b") {
                    Fail "$($want.case)/$($want.check): detail does not name the '$($want.channel)' channel ($($got[0].detail))"
                }
                if (-not $want.exit) {
                    if ($null -ne $got[0].native.exit_code) {
                        Fail "$($want.case)/$($want.check): the unit channel reported an exit value $($got[0].native.exit_code)"
                    }
                }
            }
            if ($null -ne $want.exit) {
                if ($got[0].native.exit_code -ne $want.exit) {
                    Fail "$($want.case)/$($want.check): exit $($got[0].native.exit_code), expected $($want.exit)"
                }
            }
            if ($want.core) {
                if ($tc['av-core-version'] -cne $want.core) {
                    Fail "$($want.case)/$($want.check): av-core-version '$($tc['av-core-version'])', expected '$($want.core)'"
                }
            }
            if ($want.profile) {
                if ($tc['av-profile'] -cne $want.profile) {
                    Fail "$($want.case)/$($want.check): av-profile '$($tc['av-profile'])', expected '$($want.profile)'"
                }
                if ($tc['av-source-profile'] -cne 'entry-unit') {
                    Fail "$($want.case)/$($want.check): av-source-profile '$($tc['av-source-profile'])', expected entry-unit"
                }
            } elseif ($tc.ContainsKey('av-profile')) {
                Fail "$($want.case)/$($want.check): a scalar check reported the entry-unit toolchain axis (av-profile=$($tc['av-profile']))"
            }
        }

        # ---- discrimination control: a saboted oracle MUST redden ----------
        # Flip exactly the Unit expectations to the value channel in a COPY of
        # the corpus. A harness that cannot tell Unit from i32(0) would stay
        # green here; a correct one reddens exactly the two Unit rows (U02,
        # U04) and nothing else.
        $sabDir = Join-Path $OutDir 'entry-sabotage'
        if (Test-Path $sabDir) { Remove-Item -Recurse -Force $sabDir }
        Copy-Item -Recurse $entryDir $sabDir
        $sabCase = Join-Path $sabDir 'entry-unit-862.case.ad'
        $original = Get-Content $sabCase -Raw
        $patched = $original.Replace('av_channel: "unit"', 'av_channel: "value"')
        if ($patched -ceq $original) { Fail "the sabotage control could not patch a unit expectation (corpus drift?)" }
        Set-Content $sabCase -Value $patched -NoNewline -Encoding UTF8
        $sab = Invoke-Suite (Join-Path $sabDir 'entry-unit-862.suite.ad') (Join-Path $OutDir 'entry-sabotage-out') 'entry-sabotage'
        if ($sab.exit -eq 0) { Fail "DISCRIMINATION FAILURE: a unit->value oracle mutation stayed green (the harness cannot tell Unit from i32 0)" }
        $bad = @($sab.report.results | Where-Object status -ne 'PASS' |
            ForEach-Object { "$($_.case)/$($_.check)" } | Sort-Object)
        $wantBad = @('ace.m1.u02/av', 'ace.m1.u04/av') | Sort-Object
        if (($bad -join ',') -cne ($wantBad -join ',')) {
            Fail "the sabotage control reddened an unexpected set: $($bad -join ', ') (expected exactly: $($wantBad -join ', '))"
        }

        Write-Host "verify-av-862 [entry]: PASS — $($run.report.results.Count) entry/Unit checks green through the real" `
            "ac-av/core-run-v1 recipe; channel/exit/toolchain triple asserted per row; discrimination control" `
            "reddened exactly U02/U04 when the Unit oracle was mutated to the value channel."
        exit 0
    }

    'compat' {
        if (-not $OutDir) { Write-Host "usage: -Scope compat requires -OutDir (scratch)" -ForegroundColor Red; exit 2 }
        if (-not (Test-Path $baseSuite)) { Fail "base suite missing: $baseSuite" }
        $run = Invoke-Suite $baseSuite $OutDir 'compat'
        if ($run.exit -ne 0) {
            $nonPass = @($run.report.results | Where-Object status -ne 'PASS')
            Fail ("base suite exit {0}; {1} non-PASS check(s); first: {2}:{3} {4}" -f `
                $run.exit, $nonPass.Count, $nonPass[0].case, $nonPass[0].check, $nonPass[0].status)
        }
        if ($run.report.results.Count -lt 97) { Fail "base suite ran $($run.report.results.Count) checks (< 97: checks were dropped)" }
        $a2r = @($run.report.results | Where-Object adapter -eq 'a2r')
        if ($a2r.Count -lt 26) { Fail "base suite a2r checks $($a2r.Count) (< 26)" }
        if (@($run.report.coverage_gaps).Count -gt 0) { Fail "base suite coverage gaps: $($run.report.coverage_gaps -join ', ')" }
        # The old profile must keep its OWN identity axis on every av check:
        # no scalar check may have been silently re-admitted as entry-unit.
        $av = @($run.report.results | Where-Object adapter -eq 'probe')
        $avChecks = 0
        foreach ($c in $av) {
            $tc = Get-Toolchain $c
            # Only the AV-protocol checks carry the av identity axes; the
            # native (ac/mir-native-*) checks have their own toolchain and are
            # not part of this drift assertion.
            if (-not $tc.ContainsKey('av-protocol')) { continue }
            $avChecks++
            if ($tc.ContainsKey('av-profile')) {
                Fail "$($c.case)/$($c.check): a scalar base-suite check carries the entry-unit profile axis"
            }
            if ($tc['av-core-version'] -cne 'av.core-i32/1') {
                Fail "$($c.case)/$($c.check): av-core-version '$($tc['av-core-version'])', expected av.core-i32/1"
            }
        }
        if ($avChecks -lt 20) { Fail "base suite has only $avChecks AV-protocol checks (expected the frozen scalar surface)" }
        Write-Host "verify-av-862 [compat]: PASS — the old ACE M1 base suite is green ($($run.report.results.Count) checks," `
            "$($a2r.Count) a2r) and every probe check still reports the scalar av.core-i32/1 identity axis" `
            "(zero drift after the PLAN-862 contract/judge additions)."
        exit 0
    }

    'targeted' {
        # The scoped engine checks: they prove the contract/judge plumbing that
        # the corpus relies on (decode of the append-only profile field and the
        # av_channel oracle, the entry matrix, the probe round). Whole-tier
        # suites belong to the merge gate — never here.
        # A filter that matches ZERO tests is a FAILURE, never a green: every
        # step declares the minimum number of executed tests it must report.
        $steps = @(
            @{ name = 'contract decode (ac_case)'; dir = $RepoRoot; min = 1;
               args = @('test', '-p', 'ace-conformance', '--test', 'conformance', '--features', 'test-cases', '--', 'tests::native_protocol') },
            @{ name = 'contract admission negatives'; dir = $RepoRoot; min = 23;
               args = @('test', '-p', 'auto-test-case', '--test', 'contract_decode') },
            @{ name = 'parent judge (ac_av)'; dir = $RepoRoot; min = 4;
               args = @('test', '-p', 'auto-lang-core', '--lib', '--features', 'test-cases,test-ace', '--', 'test_cases::ac_av') },
            @{ name = 'ac-core entry matrix'; dir = (Join-Path $RepoRoot 'experimental/ace/crates/core'); min = 26;
               args = @('test', '--test', 'entry_unit') },
            @{ name = 'probe entry-unit round'; dir = (Join-Path $RepoRoot 'experimental/ace'); min = 13;
               args = @('test', '--test', 'av_entry') }
        )
        foreach ($s in $steps) {
            $ErrorActionPreference = 'Continue'
            Push-Location $s.dir
            $out = (& Invoke-AceCargo @($s.args) 2>&1 | Out-String)
            $code = $LASTEXITCODE
            Pop-Location
            $ErrorActionPreference = 'Stop'
            $out -split "`n" | Where-Object { $_ -match 'test result' } | ForEach-Object { Write-Host "    $($_.Trim())" }
            if ($code -ne 0) { Fail "$($s.name) failed (exit $code)" }
            $passed = 0
            foreach ($m in [regex]::Matches($out, '(\d+) passed')) { $passed += [int]$m.Groups[1].Value }
            if ($passed -lt $s.min) {
                Fail "$($s.name) executed $passed passing test(s), below the $($s.min) this gate requires (a filter that matches nothing is not a green)"
            }
            Write-Host "verify-av-862 [targeted]: $($s.name) green — $passed test(s) executed (cargo $($s.args -join ' '))"
        }
        Write-Host "verify-av-862 [targeted]: PASS — contract decode, parent judge, entry matrix and probe round all executed and green."
        exit 0
    }

    'acceptance' {
        if (-not $OutDir) { Write-Host "usage: -Scope acceptance requires -OutDir (scratch)" -ForegroundColor Red; exit 2 }
        $self = $PSCommandPath
        foreach ($s in @('targeted', 'compat', 'entry')) {
            & $self -Scope $s -OutDir (Join-Path $OutDir $s) | Write-Host
            if ($LASTEXITCODE -ne 0) { Fail "scope '$s' failed (exit $LASTEXITCODE)" }
        }

        $rows = Load-Binding

        # 1. identity integrity (the frozen 12-column schema)
        $dupes = @($rows | Group-Object { "$($_.ns):$($_.id)" } | Where-Object Count -gt 1)
        if ($dupes.Count -gt 0) { Fail "duplicate (ns,id): $($dupes.Name -join ', ')" }

        # 2. the PLAN-862 rows exist exactly once each, as corpus rows.
        foreach ($id in @('U01', 'U02', 'U03', 'U04', 'U05', 'U06', 'U07', 'U08', 'U09', 'U10')) {
            $hit = @($rows | Where-Object { $_.ns -eq 'corpus' -and $_.id -eq $id })
            if ($hit.Count -ne 1) { Fail "PLAN-862 corpus row $id is missing or duplicated ($($hit.Count))" }
        }

        # 3. ownership notes on every gap/wait cell.
        foreach ($r in $rows) {
            foreach ($p in 'ac', 'av', 'avm', 'a2r') {
                if ($r.$p -match '^(gap|wait):' -and $r.note -notmatch '\S') {
                    Fail "$($r.id): $p is '$($r.$p)' without an owner note"
                }
            }
        }

        # 4. `na:m1` anti-abuse rule 1: a rule id is mandatory.
        foreach ($r in $rows) {
            foreach ($p in 'ac', 'av', 'avm', 'a2r') {
                $v = $r.$p
                if ($v -like 'na:m1*' -and $v -notmatch '^na:m1\((NA-[0-9]+)\)$') {
                    Fail "$($r.id)/${p}: '$v' is a bare na (decision §5 rule 1 requires a rule id)"
                }
            }
        }

        # 5. five-state distribution + the measured clean count (REPORT only).
        $green = @('ok', 'ok-emit', 'ok-parse-reject', 'ok+')
        $naRules = @{}
        $clean = 0
        $blocking = 0
        $probeCells = 0
        $waitCells = 0
        foreach ($r in ($rows | Where-Object ns -eq 'corpus')) {
            $rowClean = $true
            foreach ($p in 'ac', 'av', 'avm', 'a2r') {
                $v = $r.$p
                if ($v -cmatch '^ok') { continue }
                if ($v -cmatch '^ok\+cert-gap') { continue }
                if ($v -cmatch '^diff:accepted') { continue }
                if ($v -cmatch '^na:m1\(NA-[0-9]+\)$') {
                    $rule = $v.Substring(6, $v.Length - 7)
                    $naRules[$rule] = 1 + $(if ($naRules.ContainsKey($rule)) { $naRules[$rule] } else { 0 })
                    continue
                }
                if ($v -eq 'probe') { $probeCells++; $rowClean = $false; continue }
                if ($v -like 'wait:*') { $waitCells++; $rowClean = $false; continue }
                $rowClean = $false
            }
            if ($rowClean) { $clean++ } else { $blocking++ }
        }
        $corpusCount = @($rows | Where-Object ns -eq 'corpus').Count

        Write-Host ""
        Write-Host "==== PLAN-862 acceptance (two axes) ====" -ForegroundColor Cyan
        Write-Host "ENTRY/UNIT PROFILE (AC-01/02/03/04): verified by the entry + compat + targeted scopes above."
        Write-Host "M1 REBIND ACCOUNTING (AC-05/06): clean rows = $clean / $corpusCount (measured from binding.tsv)."
        Write-Host "  blocking rows: $blocking; unexecuted `probe` cells: $probeCells; `wait:*` cells: $waitCells."
        Write-Host "  na rule distribution: $(if ($naRules.Count -eq 0) { 'none' } else { ($naRules.Keys | Sort-Object | ForEach-Object { "$_=$($naRules[$_])" }) -join ', ' })"
        Write-Host "This scope does NOT claim 'M1 green': remaining cells are owned by their registered plans"
        Write-Host "(rule: every probe/wait cell must be reclassified by a REAL run, never assumed)."
        exit 0
    }
}
