# PLAN-788 verification gate (T-08; R3 F06 rewrite: exact executed-ok ID
# parsing, per-binary binding + counts, nextest-aware closeout — a required
# test can no longer be satisfied by prose text, a name prefix, another
# binary's totals, or an assumed summary format).
#   -Scope acceptance : every V-matrix family with REQUIRED test IDs (exact,
#                       bound to their test binary), per-binary minimums,
#                       ignored=0 (named allowlist only)
#   -Scope selftest   : parser unit tests over canned outputs, including the
#                       prose-substitution and prefix-substitution negatives
#   -Scope closeout   : merge-owned full surfaces on the frozen commit; the
#                       root cargo t/tv/tt families run under NEXTTEST and are
#                       verified against nextest's actual output format
# Usage: powershell -NoProfile -File scripts\verify-autodown-ac-788.ps1 [-Scope acceptance|selftest|closeout]

param(
    [ValidateSet("acceptance", "selftest", "closeout")]
    [string]$Scope = "acceptance"
)

$ErrorActionPreference = "Continue"
$env:RUSTC_WRAPPER = ""
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$fails = 0

# ---- libtest parsing --------------------------------------------------------
# Splits cargo test output into per-binary segments ("     Running tests\x.rs
# (target\...)" headers), collecting each segment's executed-ok test names,
# ignored names and its "test result:" summary. Returns an array of
# @{ Binary; OkNames; IgnoredNames; Passed; Failed; Ignored }.
function Split-LibTestSegments {
    param([string]$Text)
    $segments = @()
    $cur = $null
    foreach ($line in ($Text -split "`r?`n")) {
        # -cmatch (CASE-SENSITIVE): libtest also prints "running N tests"
        # which must NOT be taken for cargo's "Running <binary>" header.
        # Headers: cargo's "Running <binary>" AND the "Doc-tests <crate>"
        # section (which has NO Running header — its 0-passed summary used
        # to merge into the previous segment and clobber its counts).
        if ($line -cmatch '^\s*Running\s+(?:unittests\s+|benchmarks\s+|doctests\s+|tests\\)?(\S+)') {
            if ($null -ne $cur) { $segments += ,$cur }
            $cur = @{ Binary = $Matches[1]; OkNames = @(); IgnoredNames = @(); Passed = 0; Failed = 0; Ignored = 0; HasSummary = $false }
        } elseif ($line -cmatch '^\s*Doc-tests\s+(\S+)') {
            if ($null -ne $cur) { $segments += ,$cur }
            $cur = @{ Binary = "Doc-tests $($Matches[1])"; OkNames = @(); IgnoredNames = @(); Passed = 0; Failed = 0; Ignored = 0; HasSummary = $false }
        } elseif ($null -eq $cur) {
            # output before any Running header still belongs to a (single)
            # implicit target — cargo always prints the header, but be robust
            $cur = @{ Binary = "<implicit>"; OkNames = @(); IgnoredNames = @(); Passed = 0; Failed = 0; Ignored = 0; HasSummary = $false }
        }
        if ($line -cmatch '^test (\S+) \.\.\. ok\s*$') {
            $cur.OkNames += $Matches[1]
        } elseif ($line -cmatch '^test (\S+) \.\.\. ignored(?:\s*,.*)?$') {
            # libtest may append the ignore REASON after a comma ("ignored,
            # oracle regeneration tool: ...") — the name still counts.
            $cur.IgnoredNames += $Matches[1]
        } elseif ($line -match '(\d+) passed; (\d+) failed; (\d+) ignored') {
            $cur.Passed = [int]$Matches[1]; $cur.Failed = [int]$Matches[2]; $cur.Ignored = [int]$Matches[3]
            $cur.HasSummary = $true
        }
    }
    if ($null -ne $cur) { $segments += ,$cur }
    return $segments
}

# libtest family contract: exit 0; >=1 segment with a summary; failed==0 in
# every summary; ignored tests are an explicit named allowlist and match the
# summary counts; every RequiredId is an EXACT executed-ok name in the
# RequiredInBinary segment (if bound) or in exactly one segment (if not);
# per-binary minimums hold (a fat sibling binary cannot substitute a missing
# family), and the family total >= MinPassed.
# Exact binary matching (R4 F04): a bound binary name matches a segment
# only when the segment's file LEAF equals "<bin>.rs" / "<bin>.exe" / the
# bare name — substring containment ("alphaX.rs") does not.
function Test-BinaryLeafMatch {
    param([string]$Binary, [string]$Bin)
    $leaf = Split-Path $Binary -Leaf
    return ($leaf -eq "$Bin.rs" -or $leaf -eq "$Bin.exe" -or $leaf -eq $Bin)
}

function Test-LibTestFamily {
    param(
        [string]$Text, [int]$ExitCode,
        [string[]]$RequiredIds, [hashtable]$RequiredInBinary = @{},
        [int]$MinPassed, [hashtable]$MinPerBinary = @{},
        [string[]]$AllowedIgnored = @()
    )
    if ($ExitCode -ne 0) { return "exit=$ExitCode" }
    $segments = @(Split-LibTestSegments -Text $Text)
    $summarized = @($segments | Where-Object { $_.HasSummary })
    if ($summarized.Count -eq 0) { return "no test summaries parsed" }
    foreach ($s in $summarized) {
        if ($s.Failed -ne 0) { return "failed=$($s.Failed) in a summary ($($s.Binary))" }
        if ($s.Ignored -ne $s.IgnoredNames.Count) {
            return "summary ignored=$($s.Ignored) does not match named ignored tests ($($s.IgnoredNames.Count)) ($($s.Binary))"
        }
        # R4 (F04): per-binary CONSERVATION — every executed-ok line the
        # segment claims must be backed by its summary count (and vice
        # versa); a binary claiming a required ok with a 0-passed summary
        # refuses here, for every family.
        if ($s.OkNames.Count -ne $s.Passed) {
            return "summary passed=$($s.Passed) does not match executed-ok lines ($($s.OkNames.Count)) ($($s.Binary))"
        }
    }
    $allIgnored = @()
    foreach ($s in $summarized) { $allIgnored += $s.IgnoredNames }
    foreach ($n in $allIgnored) {
        if ($AllowedIgnored -notcontains $n) { return "ignored test not allowlisted: $n" }
    }
    $totalPassed = 0
    foreach ($s in $summarized) { $totalPassed += $s.Passed }
    if ($totalPassed -lt $MinPassed) { return "totalPassed=$totalPassed < required $MinPassed" }
    foreach ($bin in $MinPerBinary.Keys) {
        $seg = @($summarized | Where-Object { Test-BinaryLeafMatch -Binary $_.Binary -Bin $bin })
        if ($seg.Count -eq 0) { return "required binary '$bin' produced no summary" }
        $p = 0
        foreach ($s in $seg) { $p += $s.Passed }
        if ($p -lt [int]$MinPerBinary[$bin]) {
            return "binary '$bin' passed=$p < required $($MinPerBinary[$bin]) — a sibling binary cannot substitute it"
        }
    }
    $okAny = @{}
    foreach ($s in $summarized) { foreach ($n in $s.OkNames) { $okAny[$n] = $true } }
    # Exact executed-ok matching, tolerant of the libtest MODULE PREFIX:
    # a required id matches a name only on FULL equality or equality of the
    # final `::` segment ("tests::ac_case_tests::x" satisfies "x"; it does
    # NOT satisfy "x_extra" or prose text).
    foreach ($id in $RequiredIds) {
        $idLast = ($id -split '::')[-1]
        if ($RequiredInBinary.ContainsKey($id)) {
            $bin = $RequiredInBinary[$id]
            $seg = @($summarized | Where-Object { Test-BinaryLeafMatch -Binary $_.Binary -Bin $bin })
            $hit = $false
            foreach ($s in $seg) {
                foreach ($n in $s.OkNames) {
                    if ($n -eq $id -or (($n -split '::')[-1]) -eq $idLast) { $hit = $true }
                }
            }
            if (-not $hit) { return "required id missing from binary '$bin' executed-ok set: $id" }
        } else {
            $hit = $false
            foreach ($k in $okAny.Keys) {
                if ($k -eq $id -or (($k -split '::')[-1]) -eq $idLast) { $hit = $true; break }
            }
            if (-not $hit) { return "required id never executed-ok: $id" }
        }
    }
    return $null
}

# ---- nextest parsing (closeout: root cargo t/tv/tt) -------------------------
# nextest's ACTUAL semantics (verified against real runs, R7 correction of
# the R5/R6 model): "run" counts tests that actually EXECUTED; tests the
# tier filters exclude (repo default-filter / explicit -E) are counted in
# "skipped" SEPARATELY — `1 test run: 1 passed, 1 skipped` is a legal
# one-selected-one-excluded result, NOT a required-test skip. Conservation
# therefore applies to EXECUTED tests: passed + failed == run. Filter
# exclusions are reported, not refused; a REQUIRED test that did not
# execute (skipped or ignored) still refuses via the required-PASS check,
# as does a zero-run or short-run tier, any failure, a PASS-line/summary
# mismatch, and any unknown summary tail. The slow annotation "(N slow)"
# is a known grammar component (R6).
function Test-NextestFamily {
    # R9: tier-bound full ids ("<binary-id> <fully-qualified-name>") must
    # match EXACTLY — a miss refuses; the short-segment fallback exists
    # ONLY for explicitly hand-written short ids and is gated behind
    # -ShortIdFallback (the real tier calls never set it).
    param([string]$Text, [int]$ExitCode, [string[]]$RequiredIds = @(), [int]$MinPassed = 1, [switch]$ShortIdFallback)
    if ($ExitCode -ne 0) { return "exit=$ExitCode" }
    $totalRun = 0; $totalPassed = 0; $totalSkipped = 0
    foreach ($lm in [regex]::Matches($Text, '(?m)^\s*Summary\s+\[\s*[\d.]+s\]\s+(\d+) tests? run: (\d+) passed(?: \((\d+) slow\))?(?:, (\d+) failed)?(?:, (\d+) skipped)?\s*$')) {
        $run = [int]$lm.Groups[1].Value
        $passed = [int]$lm.Groups[2].Value
        $failed = if ($lm.Groups[4].Success) { [int]$lm.Groups[4].Value } else { 0 }
        $skipped = if ($lm.Groups[5].Success) { [int]$lm.Groups[5].Value } else { 0 }
        if ($failed -ne 0) { return "failed=$failed in a nextest summary" }
        # executed-test conservation: every test that RAN passed or failed
        if ($passed + $failed -ne $run) { return "summary run=$run != passed+failed ($passed+$failed)" }
        $totalRun += $run; $totalPassed += $passed; $totalSkipped += $skipped
    }
    if (-not [regex]::IsMatch($Text, 'Summary\s+\[')) {
        return "no nextest summaries parsed"
    }
    if ($totalRun -eq 0) {
        if ([regex]::IsMatch($Text, '(?m)^\s*Summary\s+\[\s*[\d.]+s\]\s+0 tests? run:')) {
            return "no tests were run (0 tests run)"
        }
        return "summary present but no full-line summary parsed (unknown shape)"
    }
    if ($totalRun -lt $MinPassed) { return "testsRun=$totalRun < required $MinPassed" }
    # R5/R8: nextest ids may contain SPACES ("crate::bin testname" shape) —
    # capture the rest of the line; tier-bound required sets are thousands
    # of ids, so exact lookup goes through a HashSet with the segment
    # fallback only for hand-written ids.
    $passedNames = @{}
    # nextest pads counters to the total's width ("( 15/163)") — the
    # counter group tolerates that padding or the padded id capture
    # swallows the counter and breaks exact matching.
    foreach ($m in [regex]::Matches($Text, '(?m)^\s*(?:PASS|DONE)\s+\[\s*[\d.]+s\]\s+(?:\(\s*\d+/\d+\s*\)\s+)?(.+?)\s*$')) {
        $passedNames[$m.Groups[1].Value.Trim()] = $true
    }
    if ($passedNames.Count -ne $totalPassed) {
        return "PASS lines ($($passedNames.Count)) do not match summary passed ($totalPassed)"
    }
    $passedSet = [System.Collections.Generic.HashSet[string]]::new([string[]]$passedNames.Keys)
    foreach ($id in $RequiredIds) {
        if ($passedSet.Contains($id)) { continue }
        if (-not $ShortIdFallback) {
            # R9: no fallback for tier ids — a same-named test in another
            # binary or module must NOT substitute the required item.
            return "required id never reported PASS by nextest: $id"
        }
        $short = ($id -split '::')[-1]
        $hit = $false
        foreach ($k in $passedNames.Keys) {
            if (($k -split '::')[-1] -eq $short -or (($k -split '\s+')[-1]) -eq $short) { $hit = $true; break }
        }
        if (-not $hit) { return "required id never reported PASS by nextest: $id" }
    }
    # R8: the selection receipt — selected/required sizes and the tier's
    # filter-excluded count travel with every nextest family result.
    $script:LastNextestReceipt = "selected-required=$($RequiredIds.Count) passed=$totalPassed filter-excluded=$totalSkipped"
    return $null
}

# ---- R8: tier selection binding --------------------------------------------
# Derive the EXPECTED selected identity set from the SAME frozen config the
# tier runs under: `cargo nextest list <tier args minus run> --message-format
# json`. JSON per test carries filter-match.status (matches/mismatch) and an
# ignored flag; the bound required set is every matched non-ignored test as
# "<binary-id> <name>" — the exact shape PASS lines use, so a sibling
# binary's or partial log cannot substitute the tier's result.
function ConvertFrom-NextestListJson {
    param([string]$Text)
    $required = @(); $ignored = @()
    $jsonLine = ($Text -split "`r?`n") | Where-Object { $_.Trim().StartsWith('{') } | Select-Object -First 1
    if (-not $jsonLine) { throw "nextest list produced no JSON object" }
    $j = $jsonLine | ConvertFrom-Json
    foreach ($suiteProp in $j.'rust-suites'.PSObject.Properties) {
        $bin = $suiteProp.Value.'binary-id'
        foreach ($tcProp in $suiteProp.Value.testcases.PSObject.Properties) {
            $tc = $tcProp.Value
            if ($tc.'filter-match'.status -eq 'matches') {
                $full = "{0} {1}" -f $bin, $tcProp.Name
                if ($tc.ignored) { $ignored += $full } else { $required += $full }
            }
        }
    }
    return @{ Required = $required; Ignored = $ignored }
}

function Get-NextestSelection {
    param([string[]]$ListArgv)
    $text = (& cargo @ListArgv 2>&1 | Out-String)
    if ($LASTEXITCODE -ne 0) {
        throw "nextest list failed (exit=$LASTEXITCODE): $text"
    }
    return ConvertFrom-NextestListJson -Text $text
}

function Run-NextestTier {
    # SimulateSelectionJson/SimulateRunText are a SELFTEST-ONLY seam: they
    # replace the cargo list/run boundary with canned data so the actual
    # tier entry path (bind -> exact check -> fails propagation) can be
    # counter-controlled without running whole tiers (the R9 controls).
    param([string]$Name, [string[]]$RunArgv, [string[]]$ListArgv,
          [string]$SimulateSelectionJson, [string]$SimulateRunText, [int]$SimulateExit = 0)
    Write-Host "== $Name" -ForegroundColor Cyan
    if ($PSBoundParameters.ContainsKey('SimulateSelectionJson')) {
        $sel = ConvertFrom-NextestListJson -Text $SimulateSelectionJson
    } else {
        $sel = Get-NextestSelection -ListArgv $ListArgv
    }
    if ($sel.Required.Count -eq 0) { throw "$Name`: the tier's frozen selection bound ZERO required tests" }
    Write-Host ("   tier selection bound: {0} required, {1} ignored (filter-excluded reported at verdict)" -f $sel.Required.Count, $sel.Ignored.Count)
    $reqIds = [string[]]$sel.Required
    $minReq = [int]$sel.Required.Count
    if ($PSBoundParameters.ContainsKey('SimulateRunText')) {
        $why = Test-NextestFamily -Text $SimulateRunText -ExitCode $SimulateExit -RequiredIds $reqIds -MinPassed $minReq
        if ($why) {
            $script:fails++
            Write-Host "   FAMILY FAIL ($why)" -ForegroundColor Red
            return $why
        }
        Write-Host "   family ok" -ForegroundColor Green
        if ($null -ne $script:LastNextestReceipt) {
            Write-Host "   receipt: $($script:LastNextestReceipt)" -ForegroundColor DarkCyan
        }
        return $null
    }
    Run-CargoFamily $Name $RunArgv $reqIds $minReq -Format "nextest"
    if ($null -ne $script:LastNextestReceipt) {
        Write-Host "   receipt: $($script:LastNextestReceipt)" -ForegroundColor DarkCyan
    }
}

function Run-CargoFamily {
    # Positional call shape (as used below): Name, Argv, RequiredIds,
    # MinPassed, AllowedIgnored — Format/RequiredInBinary/MinPerBinary are
    # named-only.
    param(
        [string]$Name, [string[]]$Argv,
        [string[]]$RequiredIds = @(), [int]$MinPassed = 1,
        [string[]]$AllowedIgnored = @(),
        [hashtable]$RequiredInBinary = @{},
        [hashtable]$MinPerBinary = @{},
        [string]$Format = "libtest",
        [switch]$AllowShortIds
    )
    Write-Host "== $Name" -ForegroundColor Cyan
    $text = (& cargo @Argv 2>&1 | Out-String)
    $code = $LASTEXITCODE
    $why = if ($Format -eq "nextest") {
        Test-NextestFamily -Text $text -ExitCode $code -RequiredIds $RequiredIds -MinPassed $MinPassed -ShortIdFallback:$AllowShortIds
    } else {
        Test-LibTestFamily -Text $text -ExitCode $code -RequiredIds $RequiredIds `
            -RequiredInBinary $RequiredInBinary -MinPassed $MinPassed -MinPerBinary $MinPerBinary `
            -AllowedIgnored $AllowedIgnored
    }
    if ($why) {
        $script:fails++
        Write-Host "   FAMILY FAIL ($why)" -ForegroundColor Red
        Write-Host $text -ForegroundColor Red
        return
    }
    Write-Host "   family ok" -ForegroundColor Green
}

if ($Scope -eq "selftest") {
    $cases = @(
        # legal controls
        @{ name = "fully-legal"; text = "running 1 test`ntest a ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @("a"); min = 1; want = $null },
        # R3 F06 negatives (permanent): prose text and name PREFIXES can no
        # longer satisfy a required ID.
        @{ name = "id-only-in-prose"; text = "note: needed something`ntest unrelated ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @("needed"); min = 1; want = "required id never executed-ok: needed" },
        @{ name = "id-prefix-substitution"; text = "running 1 test`ntest needed_extra ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @("needed"); min = 1; want = "required id never executed-ok: needed" },
        # binary binding: the required id exists but in the WRONG binary
        @{ name = "wrong-binary"; text = "     Running tests\alpha.rs (a.exe)`ntest other ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored`n     Running tests\beta.rs (b.exe)`ntest needed ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @("needed"); bind = @{ "needed" = "alpha" }; min = 1; want = "required id missing from binary 'alpha' executed-ok set: needed" },
        # the lowercase "running N tests" libtest banner must NOT be taken
        # for cargo's "Running <binary>" header (case-sensitive match) —
        # no binary segment exists, so the bound id cannot be found there
        @{ name = "lowercase-running-not-header"; text = "running 7 tests`ntest needed ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @("needed"); bind = @{ "needed" = "alpha" }; min = 1; want = "required id missing from binary 'alpha' executed-ok set: needed" },
        # module prefixes satisfy a bare required id (exact last segment)
        @{ name = "module-prefix-legal"; text = "running 1 test`ntest tests::ac_case_tests::needed ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @("needed"); min = 1; want = $null },
        # R4 (F04) negatives: an "okay-but-not-executed" state word is not
        # an ok line; a claimed ok name without summary backing refuses.
        @{ name = "ok-prefix-state"; text = "running 1 test`ntest needed ... okay-but-not-executed`ntest result: ok. 0 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @("needed"); min = 0; want = "required id never executed-ok: needed" },
        @{ name = "zero-passed-but-ok-name"; text = "running 1 test`ntest needed ... ok`ntest result: ok. 0 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @(); min = 1; want = "summary passed=0 does not match executed-ok lines (1) (<implicit>)" },
        # per-binary minimum: a fat sibling cannot substitute a missing family
        @{ name = "sibling-substitution"; text = "     Running tests\alpha.rs (a.exe)`ntest result: ok. 0 passed; 0 failed; 0 ignored`n     Running tests\beta.rs (b.exe)`ntest filler ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored`n"; exit = 0; ids = @(); min = 1; perbin = @{ "alpha" = 3 }; want = "binary 'alpha' passed=0 < required 3 — a sibling binary cannot substitute it" },
        # legacy shapes still refused
        @{ name = "zero-selected"; text = "running 0 tests`ntest result: ok. 0 passed; 0 failed; 0 ignored"; exit = 0; ids = @(); min = 1; want = "totalPassed=0 < required 1" },
        @{ name = "partially-skipped"; text = "test result: ok. 4 passed; 0 failed; 999 ignored"; exit = 0; ids = @(); min = 1; want = "summary ignored=999 does not match named ignored tests (0) (<implicit>)" },
        @{ name = "unnamed-skip"; text = "test mystery_tool ... ignored`ntest result: ok. 0 passed; 0 failed; 1 ignored"; exit = 0; ids = @(); min = 1; want = "ignored test not allowlisted: mystery_tool" },
        @{ name = "allowlisted-skip"; text = "test fine ... ok`ntest pilot_oracle_bootstrap ... ignored`ntest result: ok. 1 passed; 0 failed; 1 ignored"; exit = 0; ids = @(); min = 1; want = $null },
        @{ name = "ignored-with-reason"; text = "test fine ... ok`ntest pilot_oracle_bootstrap ... ignored, oracle regeneration tool: run explicitly`ntest result: ok. 1 passed; 0 failed; 1 ignored"; exit = 0; ids = @(); min = 1; want = $null },
        @{ name = "doc-tests-section-separate"; text = "     Running tests\vendor.rs (v.exe)`ntest vendored_ok ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored`n   Doc-tests crate-x`ntest result: ok. 0 passed; 0 failed; 0 ignored"; exit = 0; ids = @(); min = 1; want = $null },
        @{ name = "nonzero-exit"; text = "test result: FAILED. 1 passed; 1 failed; 0 ignored"; exit = 101; ids = @(); min = 1; want = "exit=101" },
        @{ name = "failed-in-summary"; text = "test x ... ok`ntest result: FAILED. 1 passed; 1 failed; 0 ignored"; exit = 0; ids = @(); min = 1; want = "failed=1 in a summary (<implicit>)" },
        # nextest parser
        @{ name = "nextest-real-success"; fmt = "nextest"; text = "        PASS [   0.552s] (1/2) alpha::needed`n        PASS [   0.911s] (2/2) alpha::other`n     Summary [   0.911s] 2 tests run: 2 passed, 0 skipped"; exit = 0; ids = @("alpha::needed"); want = $null },
        @{ name = "nextest-singular"; fmt = "nextest"; text = "        PASS [   0.010s] (1/1) alpha::needed`n     Summary [   0.010s] 1 test run: 1 passed"; exit = 0; ids = @("alpha::needed"); want = $null },
        @{ name = "nextest-real-slow-success"; fmt = "nextest"; text = "        PASS [   0.400s] (1/2) alpha::slow_one`n        PASS [   0.790s] (2/2) alpha::slow_two`n     Summary [0.790s] 2 tests run: 2 passed (2 slow), 0 skipped"; exit = 0; ids = @("alpha::slow_one"); want = $null },
        @{ name = "nextest-slow-count-zero-run"; fmt = "nextest"; text = "     Summary [   0.001s] 0 tests run: 0 passed (0 slow), 0 skipped"; exit = 0; ids = @(); min = 1; want = "no tests were run (0 tests run)" },
        @{ name = "nextest-failed-component"; fmt = "nextest"; text = "     Summary [   1.0s] 5 tests run: 4 passed, 1 failed, 0 skipped"; exit = 0; ids = @(); want = "failed=1 in a nextest summary" },
        @{ name = "nextest-conservation-run-mismatch"; fmt = "nextest"; text = "     Summary [   1.0s] 5 tests run: 4 passed, 1 skipped"; exit = 0; ids = @(); want = "summary run=5 != passed+failed (4+0)" },
        @{ name = "nextest-real-filtered-success-shortid"; fmt = "nextest"; shortok = $true; text = "        PASS [   0.100s] (1/1) ac-artifact-probe::plan788_r1_ r1_observed_spans_carry_real_frontend_boundaries`n     Summary [   0.120s] 1 test run: 1 passed, 1 skipped"; exit = 0; ids = @("r1_observed_spans_carry_real_frontend_boundaries"); min = 1; want = $null },
        @{ name = "nextest-shortid-fallback-off"; fmt = "nextest"; text = "        PASS [   0.100s] (1/1) beta::bin r1_observed_spans_carry_real_frontend_boundaries`n     Summary [   0.120s] 1 test run: 1 passed, 1 skipped"; exit = 0; ids = @("r1_observed_spans_carry_real_frontend_boundaries"); min = 1; want = "required id never reported PASS by nextest: r1_observed_spans_carry_real_frontend_boundaries" },
        @{ name = "nextest-filtered-required-excluded"; fmt = "nextest"; text = "        PASS [   0.100s] (1/1) alpha::other`n     Summary [   0.120s] 1 test run: 1 passed, 1 skipped"; exit = 0; ids = @("alpha::needed"); min = 1; want = "required id never reported PASS by nextest: alpha::needed" },
        @{ name = "nextest-required-ignored-skipped"; fmt = "nextest"; text = "        PASS [   0.100s] (1/1) alpha::other`n     Summary [   0.120s] 1 test run: 1 passed, 2 skipped"; exit = 0; ids = @("alpha::needed"); min = 1; want = "required id never reported PASS by nextest: alpha::needed" },
        @{ name = "tier-bound-partial-substitution"; fmt = "nextest-bound"; listjson = '{"rust-build-meta":{},"test-count":2,"rust-suites":{"alpha":{"binary-id":"alpha::bin","testcases":{"one":{"kind":"test","ignored":false,"filter-match":{"status":"matches"}},"two":{"kind":"test","ignored":false,"filter-match":{"status":"matches"}},"gone":{"kind":"test","ignored":false,"filter-match":{"status":"mismatch"}}}}}}'; text = "        PASS [   0.100s] (1/1) alpha::bin one`n        PASS [   0.120s] (2/2) alpha::bin stranger`n     Summary [   0.120s] 2 tests run: 2 passed, 1 skipped"; exit = 0; want = "required id never reported PASS by nextest: alpha::bin two"; wantsel = "2/0" },
        @{ name = "tier-bound-ignored-excluded"; fmt = "nextest-bound"; listjson = '{"rust-build-meta":{},"test-count":2,"rust-suites":{"alpha":{"binary-id":"alpha::bin","testcases":{"one":{"kind":"test","ignored":false,"filter-match":{"status":"matches"}},"tool":{"kind":"test","ignored":true,"filter-match":{"status":"matches"}}}}}}'; text = "        PASS [   0.100s] (1/1) alpha::bin one`n     Summary [   0.120s] 1 test run: 1 passed, 1 skipped"; exit = 0; want = $null; wantsel = "1/1" },
        @{ name = "tier-bound-zero-selection"; fmt = "nextest-bound"; listjson = '{"rust-build-meta":{},"test-count":0,"rust-suites":{}}'; text = "     Summary [   0.1s] 0 tests run: 0 passed"; exit = 0; want = "no tests were run (0 tests run)"; wantsel = "0/0" },
        @{ name = "tier-e2e-exact-hit"; fmt = "tier-e2e"; listjson = '{"rust-build-meta":{},"test-count":1,"rust-suites":{"alpha":{"binary-id":"alpha::bin","testcases":{"tests::needed":{"kind":"test","ignored":false,"filter-match":{"status":"matches"}}}}}}'; text = "        PASS [   0.100s] (1/1) alpha::bin tests::needed`n     Summary [   0.120s] 1 test run: 1 passed, 1 skipped"; exit = 0; want = $null },
        @{ name = "tier-e2e-padded-counter-exact"; fmt = "tier-e2e"; listjson = '{"rust-build-meta":{},"test-count":1,"rust-suites":{"alpha":{"binary-id":"alpha::bin","testcases":{"tests::needed":{"kind":"test","ignored":false,"filter-match":{"status":"matches"}}}}}}'; text = "        PASS [   0.221s] ( 15/163) alpha::bin tests::needed`n     Summary [   0.221s] 1 test run: 1 passed, 1 skipped"; exit = 0; want = $null },
        @{ name = "tier-e2e-wrong-binary-same-name"; fmt = "tier-e2e"; listjson = '{"rust-build-meta":{},"test-count":1,"rust-suites":{"alpha":{"binary-id":"alpha::bin","testcases":{"tests::needed":{"kind":"test","ignored":false,"filter-match":{"status":"matches"}}}}}}'; text = "        PASS [   0.100s] (1/1) beta::bin tests::needed`n     Summary [   0.120s] 1 test run: 1 passed, 1 skipped"; exit = 0; want = "required id never reported PASS by nextest: alpha::bin tests::needed" },
        @{ name = "tier-e2e-wrong-module-same-name"; fmt = "tier-e2e"; listjson = '{"rust-build-meta":{},"test-count":1,"rust-suites":{"alpha":{"binary-id":"alpha::bin","testcases":{"tests::needed":{"kind":"test","ignored":false,"filter-match":{"status":"matches"}}}}}}'; text = "        PASS [   0.100s] (1/1) alpha::bin other::needed`n     Summary [   0.120s] 1 test run: 1 passed, 1 skipped"; exit = 0; want = "required id never reported PASS by nextest: alpha::bin tests::needed" },
        @{ name = "nextest-missing-id"; fmt = "nextest"; text = "        PASS [   0.010s] (1/1) alpha::other`n     Summary [   0.010s] 1 test run: 1 passed"; exit = 0; ids = @("alpha::needed"); want = "required id never reported PASS by nextest: alpha::needed" }
        @{ name = "nextest-zero-run"; fmt = "nextest"; text = "     Summary [   0.001s] 0 tests run: 0 passed, 0 skipped"; exit = 0; ids = @(); min = 1; want = "no tests were run (0 tests run)" },
        @{ name = "nextest-short-run"; fmt = "nextest"; text = "        PASS [   0.010s] (1/1) alpha::one`n     Summary [   0.010s] 1 test run: 1 passed"; exit = 0; ids = @(); min = 2; want = "testsRun=1 < required 2" },
        @{ name = "nextest-trailing-junk"; fmt = "nextest"; text = "     Summary [   1.0s] 2 tests run: 2 passed, 0 skipped, slow: compile pauses`n        PASS [   0.1s] (1/2) a::x`n        PASS [   0.2s] (2/2) a::y"; exit = 0; ids = @(); min = 1; want = "summary present but no full-line summary parsed (unknown shape)" },
        @{ name = "nextest-pass-count-mismatch"; fmt = "nextest"; text = "        PASS [   0.1s] (1/2) a::x`n     Summary [   0.2s] 2 tests run: 2 passed"; exit = 0; ids = @(); min = 1; want = "PASS lines (1) do not match summary passed (2)" }
    )
    foreach ($c in $cases) {
        $allow = if ($c.name -eq "allowlisted-skip" -or $c.name -eq "ignored-with-reason") { @("pilot_oracle_bootstrap") } else { @() }
        $bind = if ($c.bind) { $c.bind } else { @{} }
        $perbin = if ($c.perbin) { $c.perbin } else { @{} }
        $min = if ($null -ne $c.min) { [int]$c.min } else { 1 }
        $fmt = if ($c.fmt) { $c.fmt } else { "libtest" }
        if ($fmt -eq "tier-e2e") {
            # R9: the ACTUAL Run-NextestTier entry with the cargo list/run
            # boundary replaced by canned data — wrong-binary and
            # wrong-module same-name substitutions must refuse with fails
            # propagation; the exact hit must pass.
            $before = $script:fails
            $why = Run-NextestTier "[selftest] $($c.name)" @("simulated") @("simulated") -SimulateSelectionJson $c.listjson -SimulateRunText $c.text -SimulateExit $c.exit
            $delta = $script:fails - $before
            # the negative controls legitimately increment the shared
            # counter — the DELTA is the assertion; restore the baseline so
            # the selftest total only counts real selftest failures.
            $script:fails = $before
            $ok = if ($null -eq $why) { ($null -eq $c.want) -and ($delta -eq 0) } else { ($why -eq $c.want) -and ($delta -eq 1) }
            if ($ok) {
                Write-Host "selftest $($c.name): ok" -ForegroundColor Green
            } else {
                $script:fails++
                Write-Host "selftest $($c.name): FAIL (got '$why' delta=$delta, want '$($c.want)')" -ForegroundColor Red
            }
            continue
        }
        if ($fmt -eq "nextest-bound") {
            # R8: the TIER-BINDING control path — the required set is
            # derived from a canned `nextest list` JSON exactly as the real
            # closeout tiers do, then the gate checks the run log against
            # that bound selection.
            $sel = ConvertFrom-NextestListJson -Text $c.listjson
            $why = Test-NextestFamily -Text $c.text -ExitCode $c.exit -RequiredIds ([string[]]$sel.Required) -MinPassed ([Math]::Max(1, $sel.Required.Count))
            if ($null -eq $why -and $c.wantsel) {
                $got = "{0}/{1}" -f $sel.Required.Count, $sel.Ignored.Count
                if ($got -ne $c.wantsel) { $why = "selection parsed $got, want $($c.wantsel)" }
            }
        } elseif ($fmt -eq "nextest") {
            $sw = if ($c.shortok) { [switch]$true } else { [switch]$false }
            $why = Test-NextestFamily -Text $c.text -ExitCode $c.exit -RequiredIds $c.ids -MinPassed $min -ShortIdFallback:$sw
        } else {
            $why = Test-LibTestFamily -Text $c.text -ExitCode $c.exit -RequiredIds $c.ids `
                -RequiredInBinary $bind -MinPassed $min -MinPerBinary $perbin -AllowedIgnored $allow
        }
        $ok = if ($null -eq $why) { $null -eq $c.want } else { $why -eq $c.want }
        if ($ok) {
            Write-Host "selftest $($c.name): ok" -ForegroundColor Green
        } else {
            $script:fails++
            Write-Host "selftest $($c.name): FAIL (got '$why', want '$($c.want)')" -ForegroundColor Red
        }
    }
    if ($fails -gt 0) { throw "selftest: $fails case(s) failed" }
    Write-Host "PLAN-788 selftest: ALL PASS" -ForegroundColor Green
    exit 0
}

if ($Scope -eq "acceptance") {
    # V-01/07/15 contract face
    Run-CargoFamily "auto-test-case (V-01)" @("test", "-p", "auto-test-case") `
        @("contract_v2_report_json_round_trips_through_standard_parser", "contract_v2_cache_declaration_requires_participation", "wrong_schema_is_rejected", "minimal_case_decodes") 86
    # V-02/05/06 executor level
    Run-CargoFamily "ac-core link_capture_ (V-05/06)" @("test", "--manifest-path", "experimental/ac-core/Cargo.toml", "--test", "link_capture_") `
        @("process_budget_deadline_kills_the_tree", "process_budget_output_cap_terminates_immediately", "native_bytes_invalid_utf8_round_trips_exactly", "native_bytes_nul_and_crlf_preserved", "containment_failure_replays_through_captured_path") 7
    # V-02/04/07/08/09/10 worker faces — every required ID bound to ITS
    # binary, with per-binary minimums (R3 F06).
    $workerBind = @{
        "ac_stage_parse_boundary_attributed" = "protocol_t03_"
        "native_cache_fresh_then_object_hit" = "cache_t05_"
        "native_invalidation_body_edit_full_miss" = "cache_t05_"
        "r1_budget_one_ms_total_is_enforced_not_ignored" = "review_r1_"
        "r1_product_run_invalid_exe_is_infra_error" = "review_r1_"
        "r1_backend_missing_entry_typed_through_store_path" = "review_r1_"
        "syntax_case_edit_equivalence_and_counters" = "syntax_t06_"
        "revision_trace_sequential_edits_advance" = "syntax_t06_"
    }
    $workerPerBin = @{ "feasibility_t01_" = 3; "protocol_t03_" = 10; "cache_t05_" = 7; "review_r1_" = 8; "syntax_t06_" = 7 }
    Run-CargoFamily "worker protocol+cache+r1+syntax" @("test", "--manifest-path", "experimental/ac-case-worker/Cargo.toml", "--test", "feasibility_t01_", "--test", "protocol_t03_", "--test", "cache_t05_", "--test", "review_r1_", "--test", "syntax_t06_") `
        @("ac_stage_parse_boundary_attributed", "native_cache_fresh_then_object_hit", "native_invalidation_body_edit_full_miss", "r1_budget_one_ms_total_is_enforced_not_ignored", "r1_product_run_invalid_exe_is_infra_error", "r1_backend_missing_entry_typed_through_store_path", "syntax_case_edit_equivalence_and_counters", "revision_trace_sequential_edits_advance") 35 `
        -RequiredInBinary $workerBind -MinPerBinary $workerPerBin
    # V-08/09/10 store faces
    Run-CargoFamily "probe plan788_r1_ + 783 regressions" @("test", "--manifest-path", "experimental/ac-artifact-probe/Cargo.toml", "--release", "--test", "plan788_r1_", "--test", "native_resume_", "--test", "invalidation_", "--test", "identity_", "--test", "concurrency_", "--test", "store_boundary_") `
        @("r1_compiler_recipe_change_invalidates_hir_and_object", "r1_observed_spans_carry_real_frontend_boundaries", "native_resume_four_fixtures_fresh_restart_hirhit_and_source_absent") 20
    # V-03/04/11/13/14 runner integration
    Run-CargoFamily "auto-lang ac_case_tests (V-03/04/11/14)" @("test", "-p", "auto-lang", "--features", "test-cases", "--lib", "ac_case_tests") `
        @("ac_native_run_exit_oracle_and_red_sibling", "ac_native_typed_negative_stage_attribution", "ac_product_handoff_run_consumes_real_build", "r1_consumer_invalid_exe_is_error_not_pass", "r1_response_admission_table", "syntax_parse_and_edit_dispatch_with_goldens", "ac_native_suite_full_run_and_ledger_reconcile") 13
    # pilot_oracle_bootstrap is a PRE-EXISTING deliberate #[ignore] manual
    # regeneration tool from Plan 768 (documented in the suite), not a
    # coverage skip — allowlisted by name, nothing else.
    Run-CargoFamily "auto-lang v1 regression" @("test", "-p", "auto-lang", "--features", "test-cases", "--lib", "test_cases_tests") `
        @("pilot_suite_full_run", "stale_check_detects_oracle_drift") 26 @("tests::test_cases_tests::pilot_oracle_bootstrap")
    # V-05/06 + F01/F04/F05 R3 closures (single serial process: the perf
    # scenarios and subprocess stale paths must not fight each other for
    # the machine; no --nocapture — libtest splits "test NAME ... ok" when
    # output interleaves, and the receipts live in the raw JSONL files).
    $r3Bind = @{
        "r3_parent_io_lifecycle_overcap_and_deadline" = "plan788_r3_"
        "r3_hir_backend_negative_keeps_legal_partial_hir" = "plan788_r3_"
        "r3_stale_production_paths_per_adapter_action" = "plan788_r3_"
        "r3_native_bytes_real_worker_path_producers" = "plan788_r3_"
        "turnaround_native_six_scenarios_p95_gates" = "plan788_r3_"
        "turnaround_syntax_gate_and_revision_stream" = "plan788_r3_"
    }
    Run-CargoFamily "auto-lang plan788_r3_ R3 closures + turnaround (V-12/13 gates)" @("test", "-p", "auto-lang", "--features", "test-cases", "--test", "plan788_r3_", "--", "--test-threads=1") `
        @("r3_parent_io_lifecycle_overcap_and_deadline", "r3_hir_backend_negative_keeps_legal_partial_hir", "r3_stale_production_paths_per_adapter_action", "r3_native_bytes_real_worker_path_producers", "turnaround_native_six_scenarios_p95_gates", "turnaround_syntax_gate_and_revision_stream") 6 `
        -RequiredInBinary $r3Bind

    if ($fails -gt 0) { throw "acceptance: $fails family/ies failed or refused" }
    Write-Host "PLAN-788 acceptance: ALL PASS" -ForegroundColor Green
    exit 0
}

# closeout (merge-owned): frozen-commit full surfaces. The root aliases
# (t/tv/tt) run under NEXTEST — verified per its real output format; the
# remaining families are libtest multi-binary runs.
# R10 merge-scope correction: the full single-process libtest family ran
# env-dependent plan738/746/748/749 suites that the daily tier excludes BY
# DESIGN (process isolation); contract §11.5/§14 define closeout as the
# nextest tiers + independent suites. The plan's own integration target
# runs scoped and serial here; the lib face is the bare-t family below.
Run-CargoFamily "auto-lang plan788_r3_ (integration, serial)" @("test", "-p", "auto-lang", "--features", "test-cases", "--test", "plan788_r3_", "--", "--test-threads=1") @("turnaround_native_six_scenarios_p95_gates", "turnaround_syntax_gate_and_revision_stream") 2
Run-NextestTier "root cargo t (nextest)" @("t") @("nextest","list","-p","auto-lang","--lib","--test","schema_drift","--test","docs_gen","--test","component_registry_test","--features","ui-iced,test-cases","--message-format","json")
Run-NextestTier "root cargo tv (nextest)" @("tv") @("nextest","list","-p","auto-lang","--lib","--profile","tv","--features","test-cases","--message-format","json")
Run-NextestTier "root cargo tt (nextest)" @("tt") @("nextest","list","-p","auto-lang","--lib","--features","test-trans,test-cases","--message-format","json")
Run-CargoFamily "worker full (release)" @("test", "--manifest-path", "experimental/ac-case-worker/Cargo.toml", "--release") @() 1
# gc_budget_churn_512 is the probe's OWN design ignore: its closeout
# contract executes it explicitly with --ignored (documented in the test).
Run-CargoFamily "probe full (release)" @("test", "--manifest-path", "experimental/ac-artifact-probe/Cargo.toml", "--release") @() 1 @("gc_budget_churn_512_converges_between_watermarks")
Run-CargoFamily "ac-syntax runtime (release)" @("test", "--manifest-path", "experimental/ac-syntax/runtime/Cargo.toml", "--release") @() 1

if ($fails -gt 0) { throw "closeout: $fails surface(s) failed" }
Write-Host "PLAN-788 closeout: ALL PASS" -ForegroundColor Green
