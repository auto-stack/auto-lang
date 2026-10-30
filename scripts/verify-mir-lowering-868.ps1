# PLAN-868 verify script — lowering extraction gates.
# Usage: powershell -File scripts/verify-mir-lowering-868.ps1 -Scope <scope>
#   equivalence | deps | identity | acceptance
# Scopes:
#   equivalence - run the 11 lowering-consuming suites, compare per-check
#                 against a PRE-MOVE baseline via -BaselineDir (required).
#                 Excludes volatile fields (durations, spans, worker exe
#                 sha256, temp paths, ledger_peak_* counters) per the T-02
#                 frozen field set (plan attachments/868/work/baseline.md).
#   deps        - dependency-direction guards: av-core stays dependency-free,
#                 ace-lowering depends only on ac-core + av-core, exactly
#                 one lower_checked definition, root workspace has no
#                 experimental path deps, all three consumers resolve the
#                 shared crate.
#   identity    - mir-lowering axis is reported (av rows + mn receipts) with
#                 the mirrored value, and the ace.m1.cert/1 receipt key set
#                 is unchanged in both consumers (Rust + PowerShell).
#   acceptance  - deps + identity + equivalence (when -BaselineDir given),
#                 else deps + identity + suite-green run.
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("equivalence", "deps", "identity", "acceptance")]
    [string]$Scope,
    [string]$BaselineDir = "",
    [string]$OutRoot = ""
)

$ErrorActionPreference = "Continue"  # cargo writes progress to stderr; exit codes are the gate
$repo = Split-Path -Parent $PSScriptRoot
$exp = Join-Path $repo "experimental"
if (-not $OutRoot) { $OutRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("mir-lowering-868-" + [guid]::NewGuid().ToString("N").Substring(0, 8)) }
New-Item -ItemType Directory -Force -Path $OutRoot | Out-Null

$suites = [ordered]@{
    "ac-av"         = "crates/auto-lang/test/cases/ac-av/ac-av.suite.ad"
    "av-data-821"   = "crates/auto-lang/test/cases/av-data/821/av-data-821.suite.ad"
    "av-data-822"   = "crates/auto-lang/test/cases/av-data/822/av-data-822.suite.ad"
    "av-data-823"   = "crates/auto-lang/test/cases/av-data/823/av-data-823.suite.ad"
    "av-data-824"   = "crates/auto-lang/test/cases/av-data/824/av-data-824.suite.ad"
    "av-abc-825"    = "crates/auto-lang/test/cases/av-abc/825/av-abc-825.suite.ad"
    "av-abc-826"    = "crates/auto-lang/test/cases/av-abc/826/av-abc-826.suite.ad"
    "av-source-827" = "crates/auto-lang/test/cases/av-source-827/av-source-827.suite.ad"
    "ace-m1"        = "crates/auto-lang/test/cases/ace-m1/ace-m1-base.suite.ad"
    "ac-cte"        = "crates/auto-lang/test/cases/ac-cte/ac-cte.suite.ad"
    "ace-native"         = "crates/auto-lang/test/cases/ace-native/ace-native.suite.ad"
}

function Fail([string]$msg) { Write-Host "FAILED: $msg" -ForegroundColor Red; exit 1 }

function Invoke-AutoCase {
    Push-Location $repo
    & cargo build -p auto-lang --bin auto-case --features test-cases 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { Pop-Location; Fail "auto-case build" }
    Pop-Location
}

function Run-Suites([string]$Label) {
    Invoke-AutoCase
    $dir = Join-Path $OutRoot $Label
    foreach ($name in $suites.Keys) {
        $suite = Join-Path $repo $suites[$name]
        $out = Join-Path $dir $name
        Write-Host "== run $name"
        & (Join-Path $repo "target/debug/auto-case.exe") run --suite $suite --out $out 2>&1 | Select-Object -Last 2 | Write-Host
        if ($LASTEXITCODE -ne 0) { Fail "suite $name exit $LASTEXITCODE" }
    }
    return $dir
}

function Compare-Equivalence([string]$PreDir, [string]$PostDir) {
    # Frozen per-check field set (T-02); volatile fields excluded by design.
    $py = @'
import json, sys, os
def norm_result(r):
    n = r.get("native") or {}
    counters = sorted([list(c) for c in n.get("counters", []) if not str(c[0]).startswith("ledger_peak")])
    arts = sorted([[a.get("logical_path"), a.get("sha256"), a.get("bytes")] for a in r.get("artifacts", [])])
    return {"case": r.get("case"), "check": r.get("check"), "adapter": r.get("adapter"),
            "action": r.get("action"), "tier": r.get("tier"), "status": r.get("status"),
            "mismatches": r.get("mismatches", []), "detail": r.get("detail"),
            "identity": r.get("identity"), "artifacts": arts,
            "native": {"exit_code": n.get("exit_code"), "cache_mode": n.get("cache_mode"),
                        "cache_hits": sorted([list(h) for h in n.get("cache_hits", [])]),
                        "counters": counters, "gaps": sorted(n.get("gaps", []))}}
def norm_report(p):
    r = json.load(open(p, encoding="utf-8"))
    return {"suite": r.get("suite"), "suite_sha256": r.get("suite_sha256"),
            "required_adapters": r.get("required_adapters"), "summary": r.get("summary"),
            "coverage_gaps": r.get("coverage_gaps"), "load_errors": r.get("load_errors"),
            "results": sorted([norm_result(x) for x in r.get("results", [])], key=lambda x: (x["case"] or "", x["check"] or ""))}
pre, post = sys.argv[1], sys.argv[2]
bad = 0
for fam in sorted(f for f in os.listdir(pre) if os.path.isdir(os.path.join(pre, f))):
    a = os.path.join(pre, fam, "report.json"); b = os.path.join(post, fam, "report.json")
    if not os.path.exists(b): print(f"[MISSING] {fam}"); bad += 1; continue
    if norm_report(a) != norm_report(b):
        print(f"[DIFF] {fam}"); bad += 1
    else:
        print(f"[EQ] {fam}")
sys.exit(1 if bad else 0)
'@
    $tmp = Join-Path $OutRoot "cmp.py"
    Set-Content -Path $tmp -Value $py -Encoding utf8
    & python $tmp $PreDir $PostDir
    if ($LASTEXITCODE -ne 0) { Fail "equivalence compare (see output above)" }
}

function Invoke-Deps {
    Write-Host "== deps: av-core dependency list must be empty"
    $coreLines = Get-Content (Join-Path $exp "ace/crates/av-core/Cargo.toml")
    $inDeps = $false; $depsContent = @()
    foreach ($line in $coreLines) {
        if ($line -match '^\s*\[dependencies\]') { $inDeps = $true; continue }
        if ($line -match '^\s*\[') { $inDeps = $false }
        if ($inDeps) { $depsContent += $line }
    }
    $real = $depsContent | Where-Object { $_.Trim() -ne "" -and $_ -notmatch '^\s*#' }
    if ($real.Count -gt 0) { Fail "av-core [dependencies] no longer empty: $real" }

    Write-Host "== deps: ace-lowering depends only on ac-core + av-core"
    $lowerLines = Get-Content (Join-Path $exp "ace/crates/lowering/Cargo.toml")
    $inDeps = $false; $lowerDeps = @()
    foreach ($line in $lowerLines) {
        if ($line -match '^\s*\[dependencies\]') { $inDeps = $true; continue }
        if ($line -match '^\s*\[') { $inDeps = $false }
        if ($inDeps) { $lowerDeps += $line }
    }
    $lowerDepsRaw = ($lowerDeps -join "`n")
    foreach ($dep in @("ace-core", "ace-av-core")) {
        if ($lowerDepsRaw -notmatch [regex]::Escape($dep)) { Fail "ace-lowering missing dep $dep" }
    }
    if ($lowerDepsRaw -match "cranelift|serde|rusqlite") { Fail "ace-lowering gained a forbidden dep" }

    Write-Host "== deps: exactly one lower_checked definition in experimental/"
    $defs = Get-ChildItem $exp -Recurse -Filter "*.rs" | Select-String -Pattern "pub fn lower_checked\(" -List
    if ($defs.Count -ne 1 -or $defs[0].Path -notmatch "[\\/]ace[\\/]crates[\\/]lowering[\\/]") { Fail "lower_checked copies: $($defs.Count)" }

    Write-Host "== deps: root workspace has no experimental path deps"
    $manifests = @((Join-Path $repo "Cargo.toml")) + (Get-ChildItem (Join-Path $repo "crates") -Filter "Cargo.toml" -Recurse | ForEach-Object { $_.FullName })
    foreach ($m in $manifests) {
        if ((Get-Content $m -Raw) -match 'path\s*=\s*"[^"]*experimental') { Fail "$m references experimental/" }
    }

    Write-Host "== deps: all three consumers resolve ace-lowering (cargo tree)"
    Push-Location (Join-Path $exp 'ace')
    try {
        $treeOut = & cargo tree --locked -i ace-lowering 2>&1
        if ($LASTEXITCODE -ne 0) { Fail 'cargo tree -i ace-lowering' }
        foreach ($consumer in @('ace-av','ace-native','ace-comptime')) {
            if (($treeOut -join "`n") -notmatch [regex]::Escape($consumer)) {Fail "missing lowering consumer $consumer"}
        }
        $treeOut | Select-Object -First 12 | Write-Host
    } finally {Pop-Location}
}

function Invoke-Identity {
    $dir = Run-Suites "identity"

    # PLAN-871 T-05: the identity version is READ from the mirror (single
    # source of truth) instead of being pinned here — the 881 bump left this
    # script asserting the old "1", so a real version move looked like a
    # pass-adjacent failure. The three mirrors must still AGREE.
    $mirrorSrc = Get-Content (Join-Path $repo "crates/auto-lang-core/src/test_cases/ac_mn.rs") -Raw
    $m = [regex]::Match($mirrorSrc, 'EXPECTED_MIR_LOWERING_VERSION: &str = "(\d+)"')
    if (-not $m.Success) { Fail "ac_mn.rs carries no EXPECTED_MIR_LOWERING_VERSION" }
    $mirrorVer = $m.Groups[1].Value

    Write-Host "== identity: av rows carry mir-lowering=$mirrorVer"
    $avReport = Join-Path $dir "ac-av/report.json"
    $avText = Get-Content $avReport -Raw
    if ($avText -notmatch ('"mir-lowering", "?' + [regex]::Escape($mirrorVer) + '"?')) { Fail "av report rows lack the mir-lowering axis ($mirrorVer)" }

    Write-Host "== identity: mn receipts carry mir-lowering=$mirrorVer"
    $receipt = Get-ChildItem (Join-Path $dir "ace-native") -Recurse -Filter "mn-probe-receipt.json" | Select-Object -First 1
    if (-not $receipt -or ((Get-Content $receipt.FullName -Raw) -notmatch ('"mir-lowering": "?' + [regex]::Escape($mirrorVer) + '"?'))) { Fail "mn receipt lacks the mir-lowering axis ($mirrorVer)" }

    Write-Host "== identity: mirrored constants agree across the three families"
    foreach ($f in @("ac_av.rs", "ac_cte.rs", "ac_mn.rs")) {
        $src = Get-Content (Join-Path $repo "crates/auto-lang-core/src/test_cases/$f") -Raw
        if ($src -notmatch ('EXPECTED_MIR_LOWERING_VERSION: &str = "' + [regex]::Escape($mirrorVer) + '"')) { Fail "$f mirror drift (expected $mirrorVer)" }
    }

    Write-Host "== identity: ace.m1.cert/1 receipt key set unchanged in both consumers"
    $rust = Get-Content (Join-Path $repo "crates/auto-lang-core/src/test_cases/ace_certify.rs") -Raw
    if ($rust -notmatch '(?s)RECEIPT_TOOLCHAIN_KEYS: &\[&str\] = &\[\s*"identity",\s*"identity_source",\s*"default_identity_at_certify",\s*"flags",\s*\]') { Fail "Rust receipt toolchain keys changed" }
    $ps1 = Get-Content (Join-Path $repo "scripts/verify-ace-m1-852.ps1") -Raw
    foreach ($k in @("identity_source", "default_identity_at_certify", "flags")) {
        if ($ps1 -notmatch [regex]::Escape($k)) { Fail "PS receipt toolchain keys changed ($k missing)" }
    }
}

switch ($Scope) {
    "equivalence" {
        if (-not $BaselineDir -or -not (Test-Path $BaselineDir)) { Fail "-BaselineDir <dir> required (pre-move baseline-reports)" }
        $dir = Run-Suites "equivalence"
        Compare-Equivalence $BaselineDir $dir
        Write-Host "PASS equivalence (frozen field set)"
    }
    "deps" {
        Invoke-Deps
        Write-Host "PASS deps"
    }
    "identity" {
        Invoke-Identity
        Write-Host "PASS identity"
    }
    "acceptance" {
        Invoke-Deps
        Invoke-Identity
        if ($BaselineDir -and (Test-Path $BaselineDir)) {
            Compare-Equivalence $BaselineDir (Join-Path $OutRoot "identity")
            Write-Host "PASS acceptance (deps + identity + equivalence)"
        } else {
            Write-Host "PASS acceptance (deps + identity; -BaselineDir not given, equivalence leg skipped)"
        }
    }
}
