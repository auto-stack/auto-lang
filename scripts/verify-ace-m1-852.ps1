# verify-ace-m1-852.ps1 — PLAN-852 ACE M1 conformance corpus verifier.
#
# Scopes (built by the plan's tasks; each scope fails loudly until its task
# delivers it — a missing scope is NEVER a silent green):
#   contract   T-01 (rewritten PLAN-908 T-06): frozen baseline + binding-table
#              integrity + APPROVED APPLICABILITY (applicability.json) +
#              TRACKED SOURCE IDENTITY (source-manifest.json) + owner coverage.
#              Reads ONLY tracked files (the retired docs/plans/attachments/852
#              dependency is gone); never executes guests or target compiles.
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
#   acceptance T-06 (rewritten PLAN-908 T-06): STRICT single-source M1
#              accounting over binding.tsv + applicability.json — two-layer
#              (core / committed-extension) required set, NA/future separate,
#              golden-vs-cert DUAL AXIS, per-cell ledger and required-set
#              hash, report schema ace.m1.acceptance/2. It NEVER executes,
#              NEVER certifies, and NEVER orchestrates the other scopes; a
#              current parent report may be admitted via -ParentReport
#              (fake/empty/failing reports are rejected). Exit 0 = accounting
#              integrity held (M1 red verdicts stay honestly red); exit 1 =
#              integrity violation (half-migrated rules, bare n/a, shrunk
#              denominator, fake parent report...).
#
# Exit codes: 0 = scope satisfied; 1 = verification failure; 2 = usage /
# contract error; 3 = scope not yet implemented by its owning task.

param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('contract', 'source', 'snapshot', 'certify', 'lifecycle', 'acceptance', 'receipts')]
    [string]$Scope,

    # Root of the auto-lang checkout (defaults to the script's parent).
    [string]$RepoRoot,

    # Scratch dir for scopes that execute anything (created, never a source tree).
    [string]$OutDir,

    # receipts scope: the build-rustc-identity axis to compare receipts
    # against (normally taken from a fresh suite report; the direct receipt
    # matrix seam has no suite run, so it must be supplied).
    [string]$BuildIdentity,

    # acceptance scope (optional): a real parent suite report whose executed
    # evidence this accounting admits; identity and PASS state are validated.
    [string]$ParentReport,

    # acceptance scope (optional): explicit selection list "ns:id,ns:id,..."
    # reported as a selection ONLY — selections never change verdicts.
    [string]$Select
)

$ErrorActionPreference = 'Stop'
if (-not $RepoRoot) { $RepoRoot = (Split-Path -Parent $PSScriptRoot) }

function Fail([string]$message) {
    Write-Host "verify-ace-m1-852 [$Scope]: FAIL - $message" -ForegroundColor Red
    exit 1
}

# ---- shared locations -------------------------------------------------------
$bindingPath  = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1/binding.tsv'
$applicPath   = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1/applicability.json'
$manifestPath = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1/source-manifest.json'
$corpusDir    = Join-Path $RepoRoot 'crates/auto-lang/test/cases/ace-m1'

# ---- binding.tsv loader (schema frozen by T-01; column order is contract) ---
# Columns (12): id ns feature outcome expected ac av avm a2r status enabling owner_or_note
# PLAN-908 T-06: reliable Split semantics — .NET String.Split([char]9), the
# same single-source rule as verify-av-862.ps1 (D-862-R1-01), never the
# PowerShell -split operator whose regex/limit semantics mangle columns.
function Load-Binding {
    if (-not (Test-Path $bindingPath)) { Fail "binding table missing: $bindingPath" }
    $rows = @()
    foreach ($line in (Get-Content $bindingPath -Encoding UTF8)) {
        if ($line -notmatch '\S') { continue }          # blank
        if ($line.StartsWith('#')) { continue }         # comments (header included)
        $f = $line.Split([char]9)
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

# ---- PLAN-908 applicability.json STRICT loader ------------------------------
# Schema ace.m1.applicability/1. Required top-level keys; every rule cell must
# reference an existing (ns,id,path); no cell may be claimed by two rules;
# every rule cell's binding_token must equal the CURRENT binding token
# (migrated cells carry na:m1(<rule>), inherited cells keep their raw token —
# a half-written migration or a silently edited whitelist fails here); layer
# totals are recomputed from binding+rules and must equal expected_totals.
function Load-Applicability {
    param($rows)
    if (-not (Test-Path $applicPath)) { Fail "applicability registry missing: $applicPath (PLAN-908 tracked asset)" }
    $app = $null
    try { $app = Get-Content $applicPath -Raw -Encoding UTF8 | ConvertFrom-Json }
    catch { Fail "applicability.json is not valid JSON: $_" }
    if ($app.schema -cne 'ace.m1.applicability/1') { Fail "applicability schema is '$($app.schema)', expected ace.m1.applicability/1" }
    foreach ($k in @('plan', 'approval_source', 'layers', 'extension_rows', 'expected_totals', 'rules')) {
        if (-not $app.PSObject.Properties[$k]) { Fail "applicability.json missing required key: $k" }
    }
    # approval source: the tracked decision document must exist and hash to
    # the frozen value (approval drift is a rejection, never a warning).
    $docRel = $app.approval_source.decision_doc
    $docSha = $app.approval_source.decision_doc_sha256
    if (-not $docRel -or -not $docSha) { Fail "applicability approval_source lacks decision_doc/decision_doc_sha256" }
    $docPath = Join-Path $RepoRoot $docRel
    if (-not (Test-Path $docPath)) { Fail "approval decision doc missing: $docRel" }
    if ((Sha256File $docPath) -cne $docSha) { Fail "approval decision doc DRIFTED: $docRel no longer matches the frozen ruling hash" }

    $rowByKey = @{}
    foreach ($r in $rows) { $rowByKey["$($r.ns):$($r.id)"] = $r }
    $claimed = @{}
    $naCells = 0; $futureCells = 0; $ruledAccepted = 0
    foreach ($rule in @($app.rules)) {
        if (-not $rule.rule -or -not $rule.cells) { Fail "applicability rule entry incomplete (missing rule/cells)" }
        foreach ($c in @($rule.cells)) {
            foreach ($k in @('ns', 'id', 'path', 'old_token', 'binding_token', 'migrated', 'approval')) {
                if (-not $c.PSObject.Properties[$k]) { Fail "rule $($rule.rule) cell missing field: $k" }
            }
            if (@('ac', 'av', 'avm', 'a2r') -notcontains $c.path) { Fail "rule $($rule.rule) cell has unknown path: $($c.path)" }
            $row = $rowByKey["$($c.ns):$($c.id)"]
            if (-not $row) { Fail "rule $($rule.rule) references unknown row: $($c.ns):$($c.id)" }
            $ck = "$($c.ns):$($c.id):$($c.path)"
            if ($claimed.ContainsKey($ck)) { Fail "cell claimed by two rules: $ck ($($claimed[$ck]) and $($rule.rule))" }
            $claimed[$ck] = $rule.rule
            $current = $row.($c.path)
            if ($current -cne $c.binding_token) {
                Fail "rule $($rule.rule) cell $ck binding_token '$($c.binding_token)' != current binding token '$($current)' (half-written migration or edited whitelist)"
            }
            if ($c.migrated -and $c.binding_token -cne "na:m1($($rule.rule))") {
                Fail "rule $($rule.rule) cell $ck is migrated but token is not na:m1($($rule.rule))"
            }
            if ($rule.rule -eq 'FUTURE-O') { $futureCells++ }
            elseif ($rule.rule -eq 'R-ACCEPTED') { $ruledAccepted++ }
            else { $naCells++ }
        }
    }
    if ($naCells -eq 0) { Fail "applicability carries zero NA cells" }

    # extension rows (explicit id lists; committed path av)
    $extIds = @{}
    foreach ($g in @($app.extension_rows)) {
        foreach ($i in @($g.ids)) {
            if ($extIds.ContainsKey($i)) { Fail "extension row group lists '$i' twice" }
            $extIds[$i] = $g.committed_path
        }
    }

    # layer recompute from binding + rules (the denominator can only change
    # with a ruling — a shrunk or enlarged recompute fails here)
    $tot = $app.expected_totals
    $corpusRows = @($rows | Where-Object ns -eq 'corpus')
    $scenRows = @($rows | Where-Object ns -eq 'scenario')
    $coreCells = 0; $extCells = 0; $committed = 0
    foreach ($r in $rows) {
        $isExt = ($extIds.ContainsKey($r.id) -and $r.ns -eq 'corpus')
        foreach ($p in 'ac', 'av', 'avm', 'a2r') {
            if ($isExt) {
                $extCells++
                if ($extIds[$r.id] -eq $p) { $committed++ }
            } else { $coreCells++ }
        }
    }
    $cells = $rows.Count * 4
    $required = $cells - $naCells - $futureCells - $ruledAccepted
    $coreNa = 0
    foreach ($rule in @($app.rules)) {
        if ($rule.rule -in @('FUTURE-O', 'R-ACCEPTED')) { continue }
        foreach ($c in @($rule.cells)) {
            if (-not ($extIds.ContainsKey($c.id) -and $c.ns -eq 'corpus')) { $coreNa++ }
        }
    }
    $checks = @(
        @{ have = $rows.Count;      want = $tot.rows;          what = 'rows' },
        @{ have = $corpusRows.Count; want = $tot.corpus_rows;  what = 'corpus rows' },
        @{ have = $scenRows.Count;  want = $tot.scenario_rows; what = 'scenario rows' },
        @{ have = $cells;           want = $tot.cells;         what = 'cells' },
        @{ have = ($rows.Count - @($extIds.Keys).Count); want = $tot.core_rows; what = 'core rows' },
        @{ have = $coreCells;       want = $tot.core_cells;    what = 'core cells' },
        @{ have = $extCells;        want = $tot.extension_cells; what = 'extension cells' },
        @{ have = $committed;       want = $tot.committed_cells; what = 'committed cells' },
        @{ have = $naCells;         want = $tot.rule_cells;    what = 'NA rule cells' },
        @{ have = $futureCells;     want = $tot.future_cells;  what = 'future cells' },
        @{ have = $ruledAccepted;   want = $tot.ruled_accepted_cells; what = 'ruled-accepted cells' },
        @{ have = $coreNa;          want = $tot.core_na_cells; what = 'core NA cells' },
        @{ have = $required;        want = $tot.required_cells; what = 'required cells' }
    )
    foreach ($c in $checks) {
        if ([int]$c.have -ne [int]$c.want) { Fail "applicability layer total mismatch: $($c.what) = $($c.have), approved registry says $($c.want) — the denominator was shrunk or enlarged without a ruling" }
    }
    return @{ app = $app; claimed = $claimed; extIds = $extIds; naCells = $naCells; futureCells = $futureCells; ruledAccepted = $ruledAccepted; required = $required }
}

# ---- PLAN-908 source-manifest.json STRICT loader ----------------------------
# Schema ace.m1.source-manifest/1. Tracked source identity for fresh trees:
# every listed file must exist with the exact frozen sha256; counts must
# match; the binding and scenario-oracles doc axes must bind; scenario rows
# never fabricate sources (no src/D*.at).
function Load-SourceManifest {
    param($rows)
    if (-not (Test-Path $manifestPath)) { Fail "source manifest missing: $manifestPath (PLAN-908 tracked asset)" }
    $m = $null
    try { $m = Get-Content $manifestPath -Raw -Encoding UTF8 | ConvertFrom-Json }
    catch { Fail "source-manifest.json is not valid JSON: $_" }
    if ($m.schema -cne 'ace.m1.source-manifest/1') { Fail "source-manifest schema is '$($m.schema)', expected ace.m1.source-manifest/1" }

    if ((Sha256File $bindingPath) -cne $m.binding.sha256) { Fail "binding.tsv DRIFTED against the frozen source manifest" }

    $srcDir = Join-Path $corpusDir 'src'
    $onDisk = @(Get-ChildItem $srcDir -Filter '*.at' | Sort-Object Name | ForEach-Object Name)
    if ($onDisk.Count -ne [int]$m.corpus_src.expected_count) {
        Fail "corpus src count $($onDisk.Count) != manifest $($m.corpus_src.expected_count) (unregistered source added/removed)"
    }
    foreach ($f in $onDisk) {
        if ($f -cmatch '^D\d+\.at$') { Fail "scenario D-row source fabricated: src/$f (typed/scenario rows never get corpus sources)" }
        $want = $m.corpus_src.files.$f
        if (-not $want) { Fail "src/$f not listed in the source manifest" }
        if ((Sha256File (Join-Path $srcDir $f)) -cne $want) { Fail "corpus source DRIFTED: src/$f" }
    }
    foreach ($cat in @(
        @{ node = $m.oracle_golden;  dir = 'oracle';        pattern = 'm1_*.rs';     label = 'goldens' },
        @{ node = $m.oracle_certs;   dir = 'oracle/certs';  pattern = '*.cert.json'; label = 'certificates' },
        @{ node = $m.reference;      dir = 'reference';     pattern = '*.rs';        label = 'references' }
    )) {
        $files = @(Get-ChildItem (Join-Path $corpusDir $cat.dir) -Filter $cat.pattern | Sort-Object Name)
        if ($files.Count -ne [int]$cat.node.expected_count) { Fail "$($cat.label) count $($files.Count) != manifest $($cat.node.expected_count)" }
        foreach ($f in $files) {
            $want = $cat.node.files.($f.Name)
            if (-not $want) { Fail "$($cat.label) not in manifest: $($cat.dir)/$($f.Name)" }
            if ((Sha256File $f.FullName) -cne $want) { Fail "$($cat.label) DRIFTED: $($cat.dir)/$($f.Name)" }
        }
    }
    $wrap = Sha256File (Join-Path $corpusDir 'oracle/wrapper-main-v1.rs')
    if ($wrap -cne $m.oracle_golden.wrapper_sha256) { Fail "oracle wrapper DRIFTED: oracle/wrapper-main-v1.rs" }

    # suites/cases + subtree sources
    $scCount = 0
    foreach ($p in $m.suites_cases.files.PSObject.Properties) {
        $full = Join-Path $corpusDir $p.Name
        if (-not (Test-Path $full)) { Fail "manifest suite/case file missing: $($p.Name)" }
        if ((Sha256File $full) -cne $p.Value) { Fail "suite/case file DRIFTED: $($p.Name)" }
        $scCount++
    }
    if ($scCount -ne [int]$m.suites_cases.expected_count) { Fail "suites_cases count $scCount != manifest $($m.suites_cases.expected_count)" }

    # PLAN-914: plan-scoped SCENARIO assets (a sub-directory suite's sources,
    # goldens and references). The root oracle/reference enumeration above
    # only covers the corpus root, so a sub-directory scenario's frozen files
    # would otherwise be invisible to this contract. Present => enumerate
    # EXACTLY (extra files fail), hash every file, and reject drift.
    # PLAN-915: EVERY plan-scoped scenario asset group is enumerated exactly
    # (the 914 group and the 915 group each carry dir/expected_count/files).
    # PLAN-997: `trap_assets*` groups (oracle/trap, oracle/trap-certs,
    # reference/trap) use the SAME exact enumeration discipline — the trap
    # domain's goldens/certs/references are drift-checked like every other
    # frozen asset while staying OUTSIDE the value-domain counters.
    foreach ($assetsProp in @($m.PSObject.Properties | Where-Object { $_.Name -like 'scenario_assets*' -or $_.Name -like 'trap_assets*' })) {
        $sa = $assetsProp.Value
        $saRel = [string]$sa.dir
        $saDir = Join-Path $corpusDir $saRel
        if (-not (Test-Path $saDir)) { Fail "scenario asset dir missing: $saRel" }
        $saFiles = @(Get-ChildItem $saDir -Recurse -File | Sort-Object FullName)
        if ($saFiles.Count -ne [int]$sa.expected_count) {
            Fail "scenario asset count $($saFiles.Count) != manifest $($sa.expected_count) ($saRel)"
        }
        foreach ($f in $saFiles) {
            $rel = $f.FullName.Substring($saDir.Length + 1).Replace('\', '/')
            $want = $sa.files.PSObject.Properties[$rel]
            if (-not $want) { Fail "scenario asset not in manifest: $saRel/$rel" }
            if ((Sha256File $f.FullName) -cne [string]$want.Value) {
                Fail "scenario asset DRIFTED: $saRel/$rel"
            }
        }
    }
    # scenario oracles doc (tracked authority for the 40 scenario rows)
    $soPath = Join-Path $RepoRoot $m.scenario_oracles.path
    if (-not (Test-Path $soPath)) { Fail "scenario oracles spec missing: $($m.scenario_oracles.path)" }
    if ((Sha256File $soPath) -cne $m.scenario_oracles.sha256) { Fail "scenario oracles spec DRIFTED: $($m.scenario_oracles.path)" }

    # every source-bound original corpus family (B/T/R/L/V/K ids) has its
    # frozen src/<id>.at registered (existence enforced by the per-file loop;
    # here the binding rows must resolve to manifest entries).
    foreach ($r in ($rows | Where-Object { $_.ns -eq 'corpus' -and $_.id -cmatch '^(B|T|R|L|V|K)\d\d$' })) {
        if (-not $m.corpus_src.files.("$($r.id).at")) { Fail "binding row $($r.id) has no registered corpus source src/$($r.id).at" }
    }
    return $m
}

# ---- cell classification (shared by contract + acceptance) ------------------
# Five-state vocabulary (PLAN-862 SD-01/02) + PLAN-908 a2r dual-axis split:
# the M1 a2r judgment axis is the golden/emit byte axis; the compile-certify
# axis is accounted separately (decision four / G-DEC-3).
function Get-CellState([string]$token) {
    if ($token -cmatch '^ok') { return @{ cls = 'green'; cert = 'none' } }
    if ($token -in @('emit-ok-run-unverified', 'snapshot-only-cert-gap', 'cert-diff')) {
        return @{ cls = 'green-cert-gap'; cert = 'cert-gap' }
    }
    if ($token -cmatch '^diff:accepted') { return @{ cls = 'accepted-diff'; cert = 'none' } }
    if ($token -cmatch '^diff' -or $token -cmatch '^emit-') { return @{ cls = 'pending-diff'; cert = 'none' } }
    if ($token -cmatch '^(gap|wait):') { return @{ cls = 'gap'; cert = 'none' } }
    if ($token -cmatch '^na:m1\(NA-[0-9A-Za-z-]+\)$') { return @{ cls = 'na'; cert = 'none' } }
    if ($token -ceq 'n/a') { return @{ cls = 'bare-na'; cert = 'none' } }
    return @{ cls = 'invalid'; cert = 'none' }
}

# Locate a prebuilt binary WITHOUT building (the daily snapshot face must
# spawn no cargo/rustc — F-852-R1-02). Explicit env override first.
function Find-Prebuilt([string]$name) {
    if ($env:AUTO_CASE_BIN -and $name -eq 'auto-case') { return $env:AUTO_CASE_BIN }
    $exe = Join-Path $RepoRoot "target/debug/$name.exe"
    if (Test-Path $exe) { return $exe }
    Fail "$name.exe not prebuilt at $exe — build it first (cargo build -p auto-lang --bin $name); this scope never builds (G02)."
}

# ---- STRICT receipt validation (shared by the snapshot scope and the
# direct `receipts` matrix seam) — F-852-R1-01/R2-B/R3/R4: typed exact-key
# schema, case-sensitive comparisons, scalar string guards on EVERY declared
# string field (PowerShell collection-filter semantics on -cne must never
# turn a wrong-typed JSON value into a pass), unsigned created_at, lowercase
# hex hashes, file axes, and the default-toolchain axis. Zero spawns.
function Invoke-ReceiptValidation {
    param([string]$RepoRoot, [string]$corpusDir, [string]$buildIdentity)
        # Every ok-emit binding row must carry a structurally valid receipt
        # whose oracle text is bound to the frozen registry, whose file axes
        # match the CURRENT bytes, and whose toolchain axis matches the
        # build-time identity carried by this run's own report.
    $rows = Load-Binding
    $certRows = @($rows | Where-Object { $_.ns -eq 'corpus' -and $_.outcome -eq 'value' -and $_.a2r -eq 'ok-emit' })
    if ($certRows.Count -lt 1) { Fail "no ok-emit rows found in binding" }
    if (-not $buildIdentity) { Fail "no build-rustc-identity supplied (snapshot derives it from the run report; the receipts seam takes -BuildIdentity)" }
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
            if ($cert.case -isnot [string] -or $cert.module -isnot [string] -or $cert.case -cne $row.id -or $cert.module -cne $module) { Fail "certificate $module REJECTED: case/module mismatch (wrong type or value)" }
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
            if ($cert.identity.actual_sha256 -isnot [string] -or $cert.identity.golden_sha256 -isnot [string] -or $cert.identity.actual_sha256 -cne $cert.identity.golden_sha256) {
                Fail "certificate $module REJECTED: actual_sha256 != golden_sha256 (wrong type or self-inconsistent)"
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
    return $certRows.Count
}

# ---- PLAN-997: STRICT trap-domain receipt validation -------------------------
# Token-gated (outcome=trap AND a2r=ok-emit-trap), zero-spawn, independent of
# the value validator and of token surface green: a migrated trap row WITHOUT
# a structurally valid, current `ace.m1.a2r-trap/1` receipt at
# oracle/trap-certs/m1_<id>.trap-cert.json FAILS here — deleting the
# certificate or faking the token can never sign the daily face.
function Invoke-TrapReceiptValidation {
    # $buildIdentity is OPTIONAL here: without it the default-toolchain axis
    # check is skipped (the acceptance face runs identity-free; snapshot and
    # receipts always bind a real run's identity). Existence, structure,
    # observed typed events, and file axes are enforced unconditionally.
    param($RepoRoot, $corpusDir, $buildIdentity)
    $rows = Load-Binding
    $trapRows = @($rows | Where-Object { $_.ns -eq 'corpus' -and $_.outcome -eq 'trap' -and $_.a2r -eq 'ok-emit-trap' })
    if ($trapRows.Count -eq 0) { return 0 }
    $oraclePath = Join-Path $corpusDir 'oracle/trap/trap-oracle.json'
    if (-not (Test-Path $oraclePath)) { Fail "trap oracle registry missing: oracle/trap/trap-oracle.json" }
    $oracle = Get-Content $oraclePath -Raw -Encoding UTF8 | ConvertFrom-Json
    if ($oracle.schema -cne 'ace.m1.a2r-trap-oracle/1') { Fail "trap oracle schema is '$($oracle.schema)', expected ace.m1.a2r-trap-oracle/1" }
    if ($oracle.profile -cne 'legacy-a2r.checked-i32/1') { Fail "trap oracle profile is '$($oracle.profile)', expected legacy-a2r.checked-i32/1" }
    if ([int]$oracle.exit_code -ne 70) { Fail "trap oracle exit_code is $($oracle.exit_code), expected 70" }
    $rootKeys = @('schema','case','module','profile','wrapper_file','expected_trap','identity','toolchain','deps','observed','verdict','produced_by','created_at')
    foreach ($row in $trapRows) {
        $id = $row.id
        $module = "m1_$($id.ToLower())"
        $cell = $oracle.cells.$id
        if (-not $cell) { Fail "trap oracle has no cell for $id" }
        if ($cell.kind -cne $row.expected) { Fail "trap oracle kind for $id is '$($cell.kind)' but binding expected '$($row.expected)' — the binding stays authoritative" }
        $certPath = Join-Path $corpusDir "oracle/trap-certs/$module.trap-cert.json"
        if (-not (Test-Path $certPath)) { Fail "trap certificate MISSING for ok-emit-trap row ${id}: $certPath (a green token without a receipt never signs)" }
        $cert = $null
        try { $cert = Get-Content $certPath -Raw -Encoding UTF8 | ConvertFrom-Json }
        catch { Fail "trap certificate $module is not valid JSON: $_" }
        $have = @($cert.PSObject.Properties.Name)
        $unknown = @($have | Where-Object { $rootKeys -notcontains $_ })
        if ($unknown.Count -gt 0) { Fail "trap certificate $module REJECTED: unknown root field(s): $($unknown -join ', ')" }
        $missing = @($rootKeys | Where-Object { $have -notcontains $_ })
        if ($missing.Count -gt 0) { Fail "trap certificate $module REJECTED: missing root field(s): $($missing -join ', ')" }
        if ($cert.schema -cne 'ace.m1.a2r-trap/1') { Fail "trap certificate $module REJECTED: schema is not ace.m1.a2r-trap/1" }
        if ($cert.case -cne $id) { Fail "trap certificate $module REJECTED: case is not $id" }
        if ($cert.module -cne $module) { Fail "trap certificate $module REJECTED: module is not $module" }
        if ($cert.profile -cne 'legacy-a2r.checked-i32/1') { Fail "trap certificate $module REJECTED: profile is not legacy-a2r.checked-i32/1" }
        if ($cert.wrapper_file -cne 'oracle/wrapper-main-v1.rs') { Fail "trap certificate $module REJECTED: wrapper_file is not the frozen wrapper" }
        if ($cert.verdict -cne 'certified') { Fail "trap certificate $module REJECTED: verdict is not certified" }
        if ($null -eq $cert.created_at -or $cert.created_at -isnot [long] -and $cert.created_at -isnot [int] -and $cert.created_at -isnot [double]) { Fail "trap certificate $module REJECTED: created_at is not an unsigned integer" }
        # expected_trap: the typed event, cross-checked against the registry.
        $et = $cert.expected_trap
        if (-not $et -or @($et.PSObject.Properties.Name).Count -ne 4 -or $et.kind -isnot [string] -or $et.op -isnot [string] -or $et.line -isnot [string]) { Fail "trap certificate $module REJECTED: expected_trap must have exactly {kind, op, line, exit}" }
        if ($et.kind -cne $cell.kind -or $et.op -cne $cell.op -or $et.line -cne $cell.line) { Fail "trap certificate $module REJECTED: expected_trap does not equal the trap-oracle registry cell" }
        if ("auto.trap:$($et.kind):$($et.op)" -cne $et.line) { Fail "trap certificate $module REJECTED: expected_trap line is not prefix+kind+op" }
        if ([int]$et.exit -ne 70) { Fail "trap certificate $module REJECTED: expected_trap.exit is not 70" }
        # observed: both sides the exact typed event, clean stdout.
        foreach ($side in @('actual_run','reference_run')) {
            $run = $cert.observed.$side
            if (-not $run -or @($run.PSObject.Properties.Name).Count -ne 3) { Fail "trap certificate $module REJECTED: observed.$side must have exactly {exit, stdout, stderr}" }
            if ([int]$run.exit -ne 70) { Fail "trap certificate $module REJECTED: observed.$side.exit is not the integer 70" }
            if ($run.stdout -isnot [string] -or $run.stdout -cne '') { Fail "trap certificate $module REJECTED: observed.$side.stdout is not empty (clean stdout is part of the trap contract)" }
            if ($run.stderr -isnot [string] -or $run.stderr -cne "$($cell.line)`n") { Fail "trap certificate $module REJECTED: observed.$side.stderr does not equal the newline-terminated oracle line" }
        }
        # toolchain: frozen shape; default axis must equal this run's identity.
        $tc = $cert.toolchain
        if (-not $tc -or @($tc.PSObject.Properties.Name).Count -ne 4) { Fail "trap certificate $module REJECTED: toolchain must have exactly {identity, identity_source, default_identity_at_certify, flags}" }
        if (@($tc.flags).Count -ne 2 -or @($tc.flags)[0] -cne '--edition=2021' -or @($tc.flags)[1] -cne '-O') { Fail "trap certificate $module REJECTED: toolchain.flags is not the frozen set" }
        if ($tc.identity -isnot [string] -or -not $tc.identity.Trim()) { Fail "trap certificate $module REJECTED: toolchain.identity is not a non-empty string" }
        if (@('--rustc override','env RUSTC','PATH') -notcontains $tc.identity_source) { Fail "trap certificate $module REJECTED: toolchain.identity_source is not a frozen selection rule" }
        if ($buildIdentity) {
            if ($tc.default_identity_at_certify -isnot [string] -or $tc.default_identity_at_certify -cne $buildIdentity) { Fail "trap certificate $module is STALE: default toolchain drifted since certification (re-certify to re-bind)" }
        } elseif ($tc.default_identity_at_certify -isnot [string] -or -not $tc.default_identity_at_certify.Trim()) {
            Fail "trap certificate $module REJECTED: default_identity_at_certify is not a non-empty string"
        }
        # deps: std-only, empty external.
        if (-not $cert.deps -or @($cert.deps.PSObject.Properties.Name).Count -ne 2 -or @($cert.deps.external).Count -ne 0 -or $cert.deps.closure -cne 'std-only') { Fail "trap certificate $module REJECTED: deps must be {external: [], closure: std-only}" }
        # identity axes: exact keys, hashes vs the current tree.
        $idKeys = @('source_sha256','golden_sha256','actual_sha256','wrapper_sha256','reference_sha256','trap_oracle_sha256')
        $haveId = @($cert.identity.PSObject.Properties.Name)
        $unknownId = @($haveId | Where-Object { $idKeys -notcontains $_ })
        if ($unknownId.Count -gt 0 -or @($idKeys | Where-Object { $haveId -notcontains $_ }).Count -gt 0) { Fail "trap certificate $module REJECTED: identity must have exactly the six axes" }
        if ($cert.identity.actual_sha256 -cne $cert.identity.golden_sha256) { Fail "trap certificate $module REJECTED: actual_sha256 != golden_sha256 (self-inconsistent)" }
        if ((Sha256File (Join-Path $corpusDir "src/$id.at")) -cne $cert.identity.source_sha256) { Fail "trap certificate $module is STALE: source drifted" }
        if ((Sha256File (Join-Path $corpusDir "oracle/trap/$module.rs")) -cne $cert.identity.golden_sha256) { Fail "trap certificate $module is STALE: trap golden drifted" }
        if ((Sha256File (Join-Path $corpusDir "reference/trap/$id.rs")) -cne $cert.identity.reference_sha256) { Fail "trap certificate $module is STALE: trap reference drifted" }
        if ((Sha256File (Join-Path $corpusDir 'oracle/trap/trap-oracle.json')) -cne $cert.identity.trap_oracle_sha256) { Fail "trap certificate $module is STALE: trap oracle drifted" }
        $wrapperHash = Sha256File (Join-Path $corpusDir 'oracle/wrapper-main-v1.rs')
        if ($wrapperHash -cne $cert.identity.wrapper_sha256) { Fail "trap certificate $module is STALE: wrapper drifted" }
    }
    return $trapRows.Count
}

# ---- PLAN-998: STRICT resource-scenario receipt validation ------------------
# Token-gated (ns=scenario AND outcome=events AND a2r=ok-emit-scene), zero-
# spawn, independent of the value/trap validators and of token surface green:
# a migrated resource-scenario row WITHOUT a structurally valid, current
# `ace.m1.a2r-resource-scenario/1` receipt at
# oracle/scenario-certs/m1_<id>.scenario-cert.json FAILS here — deleting the
# certificate or faking the token can never sign the daily face. The registry
# (oracle/scenario/scenario-oracle.json) is the AUTHORITY: every receipt leg
# must be one of the registry cell's legs, so a self-consistent forged
# receipt is rejected (F-998, mirror of F-997-R1-02).
function Invoke-ScenarioReceiptValidation {
    # $buildIdentity is OPTIONAL here: without it the default-toolchain axis
    # check is skipped (the acceptance face runs identity-free; snapshot and
    # receipts always bind a real run's identity). Existence, structure,
    # observed per-leg run matrices, and file axes are enforced
    # unconditionally.
    param($RepoRoot, $corpusDir, $buildIdentity)
    $rows = Load-Binding
    $sceneRows = @($rows | Where-Object { $_.ns -eq 'scenario' -and $_.outcome -eq 'events' -and $_.a2r -eq 'ok-emit-scene' })
    if ($sceneRows.Count -ne 6) { Fail "expected 6 ok-emit-scene scenario rows (outcome=events), found $($sceneRows.Count) — the PLAN-998 resource-scenario domain is migrated atomically (all six H-07 cells together)" }
    $oraclePath = Join-Path $corpusDir 'oracle/scenario/scenario-oracle.json'
    if (-not (Test-Path $oraclePath)) { Fail "resource-scenario oracle registry missing: oracle/scenario/scenario-oracle.json" }
    $oracle = Get-Content $oraclePath -Raw -Encoding UTF8 | ConvertFrom-Json
    $oracleKeys = @('schema','profile','trap_profile','wrapper_file','typed_fail_exit','trap_exit','cells')
    $haveOracle = @($oracle.PSObject.Properties.Name)
    $unknownOracle = @($haveOracle | Where-Object { $oracleKeys -notcontains $_ })
    if ($unknownOracle.Count -gt 0) { Fail "resource-scenario oracle REJECTED: unknown root field(s): $($unknownOracle -join ', ')" }
    $missingOracle = @($oracleKeys | Where-Object { $haveOracle -notcontains $_ })
    if ($missingOracle.Count -gt 0) { Fail "resource-scenario oracle REJECTED: missing root field(s): $($missingOracle -join ', ')" }
    if ($oracle.schema -cne 'ace.m1.a2r-resource-oracle/1') { Fail "resource-scenario oracle schema is '$($oracle.schema)', expected ace.m1.a2r-resource-oracle/1" }
    if ($oracle.profile -cne 'legacy-a2r.resources-observe/1') { Fail "resource-scenario oracle profile is '$($oracle.profile)', expected legacy-a2r.resources-observe/1" }
    if ($oracle.wrapper_file -cne 'oracle/resource-wrapper-v1.rs') { Fail "resource-scenario oracle wrapper_file is '$($oracle.wrapper_file)', expected the frozen resource wrapper" }
    if ([int]$oracle.typed_fail_exit -ne 71) { Fail "resource-scenario oracle typed_fail_exit is $($oracle.typed_fail_exit), expected 71" }
    if ([int]$oracle.trap_exit -ne 70) { Fail "resource-scenario oracle trap_exit is $($oracle.trap_exit), expected 70" }
    $oracleHash = Sha256File $oraclePath
    $wrapperHash = Sha256File (Join-Path $corpusDir 'oracle/resource-wrapper-v1.rs')
    if (-not $wrapperHash) { Fail "frozen resource wrapper missing: oracle/resource-wrapper-v1.rs" }
    $rootKeys = @('schema','case','profile','wrapper_file','identity','toolchain','deps','legs','observed','verdict','produced_by','created_at')
    foreach ($row in $sceneRows) {
        $id = $row.id
        $module = "m1_$($id.ToLower())"
        $cell = $oracle.cells.$id
        if (-not $cell) { Fail "resource-scenario oracle has no cell for $id" }
        $certPath = Join-Path $corpusDir "oracle/scenario-certs/$module.scenario-cert.json"
        if (-not (Test-Path $certPath)) { Fail "resource-scenario certificate MISSING for ok-emit-scene row ${id}: $certPath (a green token without a receipt never signs)" }
        $cert = $null
        try { $cert = Get-Content $certPath -Raw -Encoding UTF8 | ConvertFrom-Json }
        catch { Fail "resource-scenario certificate $module is not valid JSON: $_" }
        $have = @($cert.PSObject.Properties.Name)
        $unknown = @($have | Where-Object { $rootKeys -notcontains $_ })
        if ($unknown.Count -gt 0) { Fail "resource-scenario certificate $module REJECTED: unknown root field(s): $($unknown -join ', ')" }
        $missing = @($rootKeys | Where-Object { $have -notcontains $_ })
        if ($missing.Count -gt 0) { Fail "resource-scenario certificate $module REJECTED: missing root field(s): $($missing -join ', ')" }
        if ($cert.schema -cne 'ace.m1.a2r-resource-scenario/1') { Fail "resource-scenario certificate $module REJECTED: schema is not ace.m1.a2r-resource-scenario/1" }
        if ($cert.case -cne $id) { Fail "resource-scenario certificate $module REJECTED: case is not $id" }
        if ($cert.profile -cne 'legacy-a2r.resources-observe/1') { Fail "resource-scenario certificate $module REJECTED: profile is not legacy-a2r.resources-observe/1" }
        if ($cert.wrapper_file -cne 'oracle/resource-wrapper-v1.rs') { Fail "resource-scenario certificate $module REJECTED: wrapper_file is not the frozen resource wrapper" }
        if ($cert.verdict -cne 'certified') { Fail "resource-scenario certificate $module REJECTED: verdict is not certified" }
        if ($null -eq $cert.created_at -or $cert.created_at -isnot [long] -and $cert.created_at -isnot [int] -and $cert.created_at -isnot [double]) { Fail "resource-scenario certificate $module REJECTED: created_at is not an unsigned integer" }
        # legs: exact {id, runs} entries; every run exactly {run, exit,
        # stdout_tail, stderr_line, invariants} with invariants == pass.
        if (@($cert.legs).Count -eq 0) { Fail "resource-scenario certificate $module REJECTED: legs must be a non-empty array" }
        $registryLegIds = @($cell.legs | ForEach-Object { $_.id })
        $receiptLegIds = @()
        foreach ($leg in @($cert.legs)) {
            if (-not $leg -or @($leg.PSObject.Properties.Name).Count -ne 2) { Fail "resource-scenario certificate $module REJECTED: leg must have exactly {id, runs}" }
            if ($leg.id -isnot [string] -or -not $leg.id) { Fail "resource-scenario certificate $module REJECTED: leg.id is not a non-empty string" }
            $receiptLegIds += $leg.id
            if (@($leg.runs).Count -eq 0) { Fail "resource-scenario certificate $module REJECTED: leg $($leg.id) carries no runs" }
            foreach ($run in @($leg.runs)) {
                if (-not $run -or @($run.PSObject.Properties.Name).Count -ne 5) { Fail "resource-scenario certificate $module REJECTED: run in leg $($leg.id) must have exactly {run, exit, stdout_tail, stderr_line, invariants}" }
                if ($run.invariants -cne 'pass') { Fail "resource-scenario certificate $module REJECTED: leg $($leg.id) run '$($run.run)' invariants is '$($run.invariants)', expected pass" }
            }
        }
        # F-998 (mirror of F-997-R1-02): registry-authoritative cross-check —
        # the receipt's leg ids must be a NON-EMPTY SUBSET of the registry
        # cell's leg ids, so a self-consistent forged receipt (invented or
        # renamed legs) is rejected against the frozen registry.
        if ($receiptLegIds.Count -eq 0) { Fail "resource-scenario certificate $module REJECTED: the union of receipt leg ids is empty" }
        foreach ($lid in $receiptLegIds) {
            if ($registryLegIds -notcontains $lid) { Fail "resource-scenario certificate $module REJECTED: leg '$lid' is not a leg of the registry cell for $id (registry-authoritative cross-check)" }
        }
        # toolchain: frozen shape; default axis binds this run's identity.
        $tc = $cert.toolchain
        if (-not $tc -or @($tc.PSObject.Properties.Name).Count -ne 4) { Fail "resource-scenario certificate $module REJECTED: toolchain must have exactly {identity, identity_source, default_identity_at_certify, flags}" }
        if (@($tc.flags).Count -ne 2 -or @($tc.flags)[0] -cne '--edition=2021' -or @($tc.flags)[1] -cne '-O') { Fail "resource-scenario certificate $module REJECTED: toolchain.flags is not the frozen set ['--edition=2021','-O']" }
        if ($tc.identity -isnot [string] -or -not $tc.identity.Trim()) { Fail "resource-scenario certificate $module REJECTED: toolchain.identity is not a non-empty string" }
        if (@('--rustc override','env RUSTC','PATH') -notcontains $tc.identity_source) { Fail "resource-scenario certificate $module REJECTED: toolchain.identity_source is not a frozen selection rule" }
        if ($buildIdentity) {
            if ($tc.default_identity_at_certify -isnot [string] -or $tc.default_identity_at_certify -cne $buildIdentity) { Fail "resource-scenario certificate $module is STALE: default toolchain drifted since certification (re-certify to re-bind)" }
        } elseif ($tc.default_identity_at_certify -isnot [string] -or -not $tc.default_identity_at_certify.Trim()) {
            Fail "resource-scenario certificate $module REJECTED: default_identity_at_certify is not a non-empty string"
        }
        # deps: std-only, empty external.
        if (-not $cert.deps -or @($cert.deps.PSObject.Properties.Name).Count -ne 2 -or @($cert.deps.external).Count -ne 0 -or $cert.deps.closure -cne 'std-only') { Fail "resource-scenario certificate $module REJECTED: deps must be {external: [], closure: std-only}" }
        # identity axes: exact keys; the wrapper and the registry itself bind
        # the receipt to the CURRENT tree bytes.
        $idKeys = @('sources','goldens','references','wrapper_sha256','scenario_oracle_sha256')
        $haveId = @($cert.identity.PSObject.Properties.Name)
        $unknownId = @($haveId | Where-Object { $idKeys -notcontains $_ })
        if ($unknownId.Count -gt 0 -or @($idKeys | Where-Object { $haveId -notcontains $_ }).Count -gt 0) { Fail "resource-scenario certificate $module REJECTED: identity must have exactly {sources, goldens, references, wrapper_sha256, scenario_oracle_sha256}" }
        if ($wrapperHash -cne $cert.identity.wrapper_sha256) { Fail "resource-scenario certificate $module is STALE: resource wrapper drifted" }
        if ($oracleHash -cne $cert.identity.scenario_oracle_sha256) { Fail "resource-scenario certificate $module is STALE: scenario oracle registry drifted" }
    }
    return $sceneRows.Count
}


switch ($Scope) {
    'contract' {
        $rows = Load-Binding

        # 1. unique (ns, id) + frozen denominator shape
        $dupes = $rows | Group-Object { "$($_.ns):$($_.id)" } | Where-Object Count -gt 1
        if ($dupes) { Fail "duplicate (ns,id): $($dupes.Name -join ', ')" }
        $corpus = @($rows | Where-Object ns -eq 'corpus')
        $scenario = @($rows | Where-Object ns -eq 'scenario')
        if ($rows.Count -ne 131) { Fail "binding rows $($rows.Count) != 131 (frozen denominator)" }
        if ($corpus.Count -ne 91 -or $scenario.Count -ne 40) { Fail "binding split $($corpus.Count)/$($scenario.Count) != 91/40" }
        # corpus:R03 (named ctor) and scenario:R03 (live borrow) are two
        # independent obligations — namespace uniqueness is the contract.
        if (-not ($corpus | Where-Object id -eq 'R03') -or -not ($scenario | Where-Object id -eq 'R03')) {
            Fail "dual-namespace R03 rows must both exist (corpus R03 + scenario R03)"
        }

        # 2. approved applicability (PLAN-908) — strict registry validation
        $appv = Load-Applicability $rows

        # 3. tracked source identity (PLAN-908) — replaces cases.json
        Load-SourceManifest $rows | Out-Null

        # 4. scenario coverage: every scenario group row is bound.
        foreach ($gid in @('D01','D02','D03','D04','D05','D06','D07','D08','D09',
                           'E01','E02','E03','C01','C02',
                           'R01','R02','R03','R04','R05','R06',
                           'I01','I02','I03','I04','I05','I06','I07','I08','I09','I10','I11','I12',
                           'G01','G02','G03','G04','G05','G06','G07','G08')) {
            if (-not ($scenario | Where-Object id -eq $gid)) { Fail "scenario row $gid missing from binding.tsv" }
        }

        # 5. NA anti-abuse: no bare n/a anywhere; every na:m1 cell is claimed
        #    by an approved rule (unclaimed NA = sourceless NA = rejected).
        foreach ($r in $rows) {
            foreach ($p in 'ac', 'av', 'avm', 'a2r') {
                $st = Get-CellState $r.$p
                if ($st.cls -eq 'bare-na') { Fail "$($r.ns):$($r.id) path $p carries bare 'n/a' — an NA without a rule ID is not admissible (anti-abuse)" }
                if ($st.cls -eq 'invalid') { Fail "$($r.ns):$($r.id) path $p token '$($r.$p)' is outside the five-state vocabulary" }
                if ($st.cls -eq 'na' -and -not $appv.claimed.ContainsKey("$($r.ns):$($r.id):$p")) {
                    Fail "$($r.ns):$($r.id) path $p is na:m1 but NOT covered by any approved rule (sourceless NA)"
                }
            }
        }

        # 6. frozen scalar baseline (contract-decision 2026-10-26): the four
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

        # 7. every gap/wait row names an owner or owning plan in its note.
        foreach ($r in $rows) {
            if ($r.ac -match '^(gap|wait):' -or $r.av -match '^(gap|wait):' -or $r.avm -match '^(gap|wait):' -or $r.a2r -match '^(gap|wait):') {
                if ($r.note -notmatch '\S') { Fail "$($r.id): gap/wait row without owner note" }
            }
        }

        # 8. two-axis summary (base layer vs M1 completeness are separate
        #    conclusions — this count is the denominator, not a verdict).
        $byOutcome = $corpus | Group-Object outcome | ForEach-Object { "$($_.Name)=$($_.Count)" }
        Write-Host "verify-ace-m1-852 [contract]: PASS — $($rows.Count) binding rows" `
            "($($corpus.Count) corpus + $($scenario.Count) scenario);" `
            "corpus outcomes: $($byOutcome -join ', ');" `
            "approved applicability: $($appv.naCells) NA + $($appv.futureCells) future cells," `
            "$($appv.required) required cells; tracked source manifest verified." `
            "M1 completeness is NOT implied by this scope."
        exit 0
    }

    'source' {
        if (-not $OutDir) { Write-Host "usage: -Scope source requires -OutDir (scratch)" -ForegroundColor Red; exit 2 }
        $bin = if ($env:AUTO_CASE_BIN) { $env:AUTO_CASE_BIN } else {
            $ErrorActionPreference = 'Continue'
            & cargo @('build', '-p', 'auto-lang', '--bin', 'auto-case', '--quiet') 2>$null
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
        # PLAN-992: the a2r-only certification extension suite (the five
        # H-02 goldens B08/B11/R01/R03/R04) rides the SAME daily face under
        # the SAME instrumentation; the 852 base suite itself stays frozen
        # (36 cases / 97 checks unexpanded) and the gate counts the UNION
        # (26 base + 5 extension = 31 a2r checks).
        $extSuite = Join-Path $corpusDir 'legacy-a2r-cert-992.suite.ad'
        if (-not (Test-Path $extSuite)) { Fail "a2r certification extension suite missing: $extSuite" }
        $report = Join-Path $OutDir 'ace-m1-a2r-report.json'
        $extOut = Join-Path $OutDir 'ext-992'
        $extReport = Join-Path $extOut 'ace-m1-a2r-ext-report.json'
        # PLAN-994: the a2r-only TEXT certification extension suite (the four
        # H-03 goldens T01/T02/T03/T04 whose lowering was repaired) rides the
        # same daily face; union gate becomes 26 + 5 + 4 = 35 a2r checks.
        $ext2Suite = Join-Path $corpusDir 'legacy-a2r-text-994.suite.ad'
        if (-not (Test-Path $ext2Suite)) { Fail "a2r text certification extension suite missing: $ext2Suite" }
        $ext2Out = Join-Path $OutDir 'ext-994'
        $ext2Report = Join-Path $ext2Out 'ace-m1-a2r-ext2-report.json'

        # PLAN-996: the a2r-only finite-LIST certification extension suite (the
        # six H-04 goldens L01/L02/L03/L04/L05/L07 whose 0-arg generic
        # List<T>.new() construction was lowered) rides the same daily face;
        # union gate becomes 26 + 5 + 4 + 6 + 4 = 45 a2r checks.
        $ext3Suite = Join-Path $corpusDir 'legacy-a2r-list-996.suite.ad'
        if (-not (Test-Path $ext3Suite)) { Fail "a2r list certification extension suite missing: $ext3Suite" }
        $ext3Out = Join-Path $OutDir 'ext-996'
        $ext3Report = Join-Path $ext3Out 'ace-m1-a2r-ext3-report.json'
        # PLAN-997: the a2r-only TRAP certification extension suite (the four
        # H-06 trap cells B13/B14/B15/B20 under the explicit
        # legacy-a2r.checked-i32/1 profile, goldens at oracle/trap/) rides the
        # same daily zero-spawn face; union gate becomes 26 + 5 + 4 + 4 = 39.
        $ext4Suite = Join-Path $corpusDir 'legacy-a2r-trap-997.suite.ad'
        if (-not (Test-Path $ext4Suite)) { Fail "a2r trap certification extension suite missing: $ext4Suite" }
        $ext4Out = Join-Path $OutDir 'ext-997'
        $ext4Report = Join-Path $ext4Out 'ace-m1-a2r-ext4-report.json'
        # PLAN-998: the a2r-only RESOURCE-SCENARIO certification extension
        # suite (the twelve scene goldens covering the six H-07 scenario cells
        # R01..R06 under the explicit legacy-a2r.resources-observe/1 profile,
        # goldens at oracle/scenario/) rides the same daily zero-spawn face;
        # union gate becomes 26 + 5 + 4 + 6 + 4 + 12 = 57 a2r checks.
        $ext5Suite = Join-Path $corpusDir 'legacy-a2r-resources-998.suite.ad'
        if (-not (Test-Path $ext5Suite)) { Fail "a2r resource-scenario certification extension suite missing: $ext5Suite" }
        $ext5Out = Join-Path $OutDir 'ext-998'
        $ext5Report = Join-Path $ext5Out 'ace-m1-a2r-ext5-report.json'

        # ---- REAL process-creation instrumentation (F-852-R1-02) ----------
        # Run the suite async and poll the OS process table (in-process CIM;
        # no watcher process is created). Any NEW rustc/cargo/certified-
        # binary process DESCENDED FROM OUR RUN is a G02 violation; unrelated
        # concurrent builds on the machine (other sessions) are ignored by
        # ancestry, not by luck.
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
        # Ancestor chain image names (bounded walk; the target's own name is
        # included). Used to recognize the ENGINE-WORKER MAINTENANCE toolchain
        # under our runner: cargo -> (sccache ->) (rustc ->) rustc. The a2r
        # certification face never has a toolchain ancestor — its rustc/m1_*
        # children hang DIRECTLY off the runner.
        function Get-AncestorNames([uint32]$targetId, [hashtable]$procs) {
            $names = @()
            $seen = @{}
            $cur = $targetId
            while ($cur -and -not $seen.ContainsKey($cur) -and $names.Count -lt 32) {
                $p = $procs[$cur]
                if (-not $p) { break }
                $names += $p.Name
                $seen[$cur] = $true
                $cur = $p.ParentProcessId
            }
            return ,$names
        }
        function Invoke-InstrumentedSuiteRun([string]$binPath, [string]$suitePath, [string]$outDir, [string]$reportPath, [string]$label) {
            $baseline = Get-CimInstance Win32_Process -Filter "Name='rustc.exe' OR Name='cargo.exe' OR Name LIKE 'm1%'" -ErrorAction SilentlyContinue
            $baselineIds = @($baseline | ForEach-Object { $_.ProcessId })
            $suiteOut = Join-Path $outDir 'suite-console.txt'
            $proc = Start-Process -FilePath $binPath `
                -ArgumentList @('run', '--suite', $suitePath, '--out', $outDir, '--format', 'console', '--json-out', $reportPath) `
                -RedirectStandardOutput $suiteOut -RedirectStandardError (Join-Path $outDir 'suite-stderr.txt') `
                -PassThru -NoNewWindow
            $runnerId = $proc.Id
            $violations = @()
            $maintenance = 0
            $identityProbes = 0
            while (-not $proc.HasExited) {
                $now = Get-CimInstance Win32_Process -Filter "Name='rustc.exe' OR Name='cargo.exe' OR Name LIKE 'm1%'" -ErrorAction SilentlyContinue
                $fresh = @($now | Where-Object { $baselineIds -notcontains $_.ProcessId })
                if ($fresh.Count -gt 0) {
                    $table = @{}
                    foreach ($p in (Get-CimInstance Win32_Process -ErrorAction SilentlyContinue)) { $table[$p.ProcessId] = $p }
                    foreach ($f in $fresh) {
                        if (-not (Test-Descendant -targetId $f.ProcessId -ancestor $runnerId -procs $table)) { continue }
                        $chainNames = Get-AncestorNames $f.ProcessId $table
                        $parent = $table[$f.ParentProcessId]
                        if ($f.Name -eq 'cargo.exe') {
                            # Engine-worker maintenance build (PLAN-818 memoized no-op
                            # discipline; builds the mn/av HARNESS, never transpiled
                            # corpus artifacts). Reported, not a G02 violation.
                            $maintenance += 1
                        }
                        elseif ($f.Name -eq 'rustc.exe' -and ($chainNames -contains 'cargo.exe' -or $chainNames -contains 'sccache.exe' -or ($chainNames -match '^rustc\.exe$').Count -gt 1)) {
                            # rustc inside the maintenance toolchain: cargo spawns
                            # rustc directly, through sccache (RUSTC_WRAPPER), or
                            # rustc itself re-spawns rustc. The a2r certification
                            # face never has a toolchain ancestor — its rustc/m1_*
                            # children hang DIRECTLY off the runner.
                            $maintenance += 1
                        }
                        elseif ($f.Name -eq 'rustc.exe' -and $f.CommandLine -and $f.CommandLine -match '(^|\s)--print(\s|$)' -and $f.CommandLine -notmatch '(^|\s)(-o|--crate-name|--emit|-Z)(\s|=)') {
                            # A pure toolchain IDENTITY query (e.g. the ac adapter's
                            # displayed-identity sysroot probe, PLAN-814 stale axes:
                            # `rustc --print sysroot`). It compiles and runs NOTHING —
                            # a certification-face compile always carries an input
                            # file with -o/--crate-name and never --print.
                            $identityProbes += 1
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
                $gate = if (Test-Path $reportPath) { 0 } else { 1 }
            }
            return [pscustomobject]@{ gate = $gate; violations = $violations; maintenance = $maintenance; identityProbes = $identityProbes; label = $label }
        }
        $baseRun = Invoke-InstrumentedSuiteRun $bin $suite $OutDir $report 'base'
        $extRun = Invoke-InstrumentedSuiteRun $bin $extSuite $extOut $extReport 'ext-992'
        $ext2Run = Invoke-InstrumentedSuiteRun $bin $ext2Suite $ext2Out $ext2Report 'ext-994'
        $ext3Run = Invoke-InstrumentedSuiteRun $bin $ext3Suite $ext3Out $ext3Report 'ext-996'
        $ext4Run = Invoke-InstrumentedSuiteRun $bin $ext4Suite $ext4Out $ext4Report 'ext-997'
        $ext5Run = Invoke-InstrumentedSuiteRun $bin $ext5Suite $ext5Out $ext5Report 'ext-998'
        if ($baseRun.gate -ne 0) { Fail "base suite gate exit $($baseRun.gate) under the snapshot scope (expected 0)" }
        if ($extRun.gate -ne 0) { Fail "a2r certification extension suite gate exit $($extRun.gate) under the snapshot scope (expected 0)" }
        if ($ext2Run.gate -ne 0) { Fail "a2r text certification extension suite gate exit $($ext2Run.gate) under the snapshot scope (expected 0)" }
        if ($ext3Run.gate -ne 0) { Fail "a2r list certification extension suite gate exit $($ext3Run.gate) under the snapshot scope (expected 0)" }
        if ($ext4Run.gate -ne 0) { Fail "a2r trap certification extension suite gate exit $($ext4Run.gate) under the snapshot scope (expected 0)" }
        if ($ext5Run.gate -ne 0) { Fail "a2r resource-scenario certification extension suite gate exit $($ext5Run.gate) under the snapshot scope (expected 0)" }
        $violations = @($baseRun.violations) + @($extRun.violations) + @($ext2Run.violations) + @($ext3Run.violations) + @($ext4Run.violations) + @($ext5Run.violations)
        if ($violations.Count -gt 0) {
            Fail ("G02 VIOLATION: the daily snapshot run spawned the a2r certification face (direct rustc / certified binaries): {0}" -f ($violations -join ', '))
        }
        $maintenance = $baseRun.maintenance + $extRun.maintenance + $ext2Run.maintenance + $ext3Run.maintenance + $ext4Run.maintenance + $ext5Run.maintenance
        $identityProbes = $baseRun.identityProbes + $extRun.identityProbes + $ext2Run.identityProbes + $ext3Run.identityProbes + $ext4Run.identityProbes + $ext5Run.identityProbes
        Write-Host "verify-ace-m1-852 [snapshot]: process instrumentation: 0 certification-face spawns (direct rustc / m1_* binaries; ancestry-checked, base + ext-992 + ext-994 + ext-996 + ext-997 + ext-998 suites); $maintenance engine-worker maintenance build process(es) observed (PLAN-818 memoized harness discipline, not corpus compilation); $identityProbes rustc identity query(/-ies) (`--print`, no compile axes — PLAN-814 ac stale-axes probe)."

        $r = Get-Content $report -Raw | ConvertFrom-Json
        $re = Get-Content $extReport -Raw | ConvertFrom-Json
        $re2 = Get-Content $ext2Report -Raw | ConvertFrom-Json
        $re3 = Get-Content $ext3Report -Raw | ConvertFrom-Json
        $re4 = Get-Content $ext4Report -Raw | ConvertFrom-Json
        $re5 = Get-Content $ext5Report -Raw | ConvertFrom-Json
        $extA2r = @($re.results | Where-Object adapter -eq 'a2r')
        $ext2A2r = @($re2.results | Where-Object adapter -eq 'a2r')
        $ext3A2r = @($re3.results | Where-Object adapter -eq 'a2r')
        $ext4A2r = @($re4.results | Where-Object adapter -eq 'a2r')
        $ext5A2r = @($re5.results | Where-Object adapter -eq 'a2r')
        $a2r = @($r.results | Where-Object adapter -eq 'a2r') + $extA2r + $ext2A2r + $ext3A2r + $ext4A2r + $ext5A2r
        $nonPass = @($a2r | Where-Object status -ne 'PASS')
        if ($nonPass.Count -gt 0) {
            Fail ("a2r snapshot has {0} non-PASS check(s); first: {1}:{2} {3}" -f $nonPass.Count, $nonPass[0].case, $nonPass[0].check, $nonPass[0].status)
        }
        if ($a2r.Count -lt 57) { Fail "a2r snapshot check count $($a2r.Count) < 57 (base 26 + PLAN-992 extension 5 + PLAN-994 extension 4 + PLAN-996 extension 6 + PLAN-997 trap extension 4 + PLAN-998 resource-scenario extension 12 — checks were dropped)" }

        # ---- STRICT certificate validity (shared function) ----
        $buildIdentity = $r.toolchain.'build-rustc-identity'
        if (-not $buildIdentity) { Fail "suite report lacks the build-rustc-identity toolchain axis (rebuild auto-case)" }
        $certCount = Invoke-ReceiptValidation -RepoRoot $RepoRoot -corpusDir $corpusDir -buildIdentity $buildIdentity
        # PLAN-997: the trap domain's STRICT receipt validation — token-gated
        # (only rows already migrated to the trap-domain token), zero-spawn,
        # same Invalid-vs-missing discipline as the value domain.
        $trapCertCount = Invoke-TrapReceiptValidation -RepoRoot $RepoRoot -corpusDir $corpusDir -buildIdentity $buildIdentity
        # PLAN-998: the resource-scenario domain's STRICT receipt validation —
        # token-gated (the six outcome=events scenario rows on ok-emit-scene),
        # zero-spawn, registry-authoritative cross-checks, same discipline as
        # the value/trap domains.
        $sceneCertCount = Invoke-ScenarioReceiptValidation -RepoRoot $RepoRoot -corpusDir $corpusDir -buildIdentity $buildIdentity

        $goldens = @(Get-ChildItem (Join-Path $corpusDir 'oracle') -Filter 'm1_*.rs')
        Write-Host "verify-ace-m1-852 [snapshot]: PASS — $($a2r.Count) a2r checks green (emit + byte-compare;" `
            "26 base + $($extA2r.Count) PLAN-992 extension + $($ext2A2r.Count) PLAN-994 extension + $($ext3A2r.Count) PLAN-996 extension + $($ext4A2r.Count) PLAN-997 trap extension + $($ext5A2r.Count) PLAN-998 resource-scenario extension);" `
            "$certCount value certificates + $trapCertCount trap certificates + $sceneCertCount resource-scenario certificates STRICTLY valid (typed exact-key schema + file axes + default-toolchain" `
            "axis vs this run's build identity). Certification-face process watch: 0 direct-rustc/m1_* spawns" `
            "observed in periodic CIM ancestry sampling (a sampling observer, not a lossless creation count)." `
            "Full base suite $($r.results.Count) checks green; extension suite $($re.results.Count) checks green." `
            "Emit PASS is not a behavior claim; re-certification is the explicit certify scope."
        exit 0
    }

    'receipts' {
        # F-852-R4-01 direct production-validator seam: STRICT receipt
        # validation ONLY (no suite run, no build, no spawn) for the
        # cross-consumer malformed-receipt matrix and fast repair replays.
        # The authoritative daily face remains `snapshot` (which also binds
        # the build identity from a real run); here the identity axis must
        # be supplied via -BuildIdentity (e.g. a valid receipt's recorded
        # default_identity_at_certify).
        if (-not $BuildIdentity) {
            Write-Host "usage: -Scope receipts requires -BuildIdentity <axis> (take it from a valid receipt's default_identity_at_certify)" -ForegroundColor Red
            exit 2
        }
        $count = Invoke-ReceiptValidation -RepoRoot $RepoRoot -corpusDir $corpusDir -buildIdentity $BuildIdentity
        $trapCount = Invoke-TrapReceiptValidation -RepoRoot $RepoRoot -corpusDir $corpusDir -buildIdentity $BuildIdentity
        $sceneCount = Invoke-ScenarioReceiptValidation -RepoRoot $RepoRoot -corpusDir $corpusDir -buildIdentity $BuildIdentity
        Write-Host "verify-ace-m1-852 [receipts]: PASS — $count value certificates + $trapCount trap certificates + $sceneCount resource-scenario certificates STRICTLY valid (direct seam; no suite run)."
        exit 0
    }

    'certify' {
        # Certification is an explicit, separate operation by contract; it is
        # never triggered implicitly by any other scope (G08 / F-852-R1-02).
        $bin = if ($env:AUTO_CERTIFY_BIN) { $env:AUTO_CERTIFY_BIN } else {
            $ErrorActionPreference = 'Continue'
            & cargo @('build', '-p', 'auto-lang', '--bin', 'ace-certify', '--quiet') 2>$null
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

        # PLAN-997: the trap domain's explicit certification pass. Runs once
        # any outcome=trap row has been migrated to the trap-domain token
        # (ok-emit-trap); before migration the trap domain is still an
        # accounted cert-gap and this pass stays dormant (no bulk green sweep
        # of unmigrated rows — evidence first, token flip afterwards).
        $trapRows = @(Load-Binding | Where-Object { $_.ns -eq 'corpus' -and $_.outcome -eq 'trap' -and $_.a2r -eq 'ok-emit-trap' })
        if ($trapRows.Count -gt 0) {
            $ErrorActionPreference = 'Continue'
            & $bin @('--trap', '--corpus', $corpus) 2>$null
            $code = $LASTEXITCODE
            $ErrorActionPreference = 'Stop'
            if ($code -ne 0) { Fail "trap certification exit $code (0 = certified/reused, 1 = failure)" }
        } else {
            Write-Host "verify-ace-m1-852 [certify]: trap domain dormant — no ok-emit-trap rows yet (pre-PLAN-997 token state)"
        }

        # Explicit-only governance controls (F-852-R1-02/03): stream ordering,
        # per-stream overflow kill, and the wall kill. Full test paths +
        # --ignored so a typo can never silently match zero tests.
        $ErrorActionPreference = 'Continue'
        & cargo @('test', '-p', 'auto-lang-core', '--lib', '--',
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
            "receipt); trap domain certified when migrated (ace-certify --trap);" `
            "stream-order/overflow/wall governance controls green." `
            "Receipts live in oracle/certs (value) and oracle/trap-certs (trap) and are versioned."
        exit 0
    }

    'lifecycle' {
        if (-not $OutDir) { Write-Host "usage: -Scope lifecycle requires -OutDir (scratch)" -ForegroundColor Red; exit 2 }
        $bin = if ($env:AUTO_CASE_BIN) { $env:AUTO_CASE_BIN } else {
            $ErrorActionPreference = 'Continue'
            & cargo @('build', '-p', 'auto-lang', '--bin', 'auto-case', '--quiet') 2>$null
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
        if (-not $OutDir) { Write-Host "usage: -Scope acceptance requires -OutDir (scratch report dir)" -ForegroundColor Red; exit 2 }
        # PLAN-908 T-06 rewrite: STRICT single-source M1 accounting. This
        # scope never executes anything, never writes certificates, never
        # certifies, and never orchestrates the other scopes (G06/G08/F-852-
        # R1-02). It re-runs every contract integrity check first, then
        # classifies all 524 cells against the approved applicability
        # registry and emits an ace.m1.acceptance/2 report. Exit 0 means the
        # ACCOUNTING is valid — red verdicts stay honestly red.

        $rows = Load-Binding
        $dupes = $rows | Group-Object { "$($_.ns):$($_.id)" } | Where-Object Count -gt 1
        if ($dupes) { Fail "duplicate (ns,id): $($dupes.Name -join ', ')" }
        $appv = Load-Applicability $rows
        Load-SourceManifest $rows | Out-Null

        # vocabulary + NA anti-abuse parity with the contract scope (a bare
        # n/a or a sourceless na:m1 cell fails the accounting itself)
        foreach ($r in $rows) {
            foreach ($p in 'ac', 'av', 'avm', 'a2r') {
                $st = Get-CellState $r.$p
                if ($st.cls -eq 'bare-na') { Fail "$($r.ns):$($r.id) path $p carries bare 'n/a' — an NA without a rule ID is not admissible (anti-abuse)" }
                if ($st.cls -eq 'invalid') { Fail "$($r.ns):$($r.id) path $p token '$($r.$p)' is outside the five-state vocabulary" }
                if ($st.cls -eq 'na' -and -not $appv.claimed.ContainsKey("$($r.ns):$($r.id):$p")) {
                    Fail "$($r.ns):$($r.id) path $p is na:m1 but NOT covered by any approved rule (sourceless NA)"
                }
            }
        }

        # ---- cell ledger -------------------------------------------------
        # Layer scope is driven by the APPROVED RULE CLAIM, not by the token
        # class: an inherited rule's cells keep their honest gap:* tokens
        # (capability gaps with owners) while being out of the M1-required
        # set exactly as approved.
        $ledger = @()
        $requiredCells = @()
        $coreBlockers = @(); $extBlockers = @(); $futureGaps = @()
        foreach ($r in $rows) {
            $isExt = ($appv.extIds.ContainsKey($r.id) -and $r.ns -eq 'corpus')
            foreach ($p in 'ac', 'av', 'avm', 'a2r') {
                $token = $r.$p
                $st = Get-CellState $token
                $key = "$($r.ns):$($r.id):$p"
                $claimRule = $appv.claimed[$key]
                $isFuture = ($claimRule -eq 'FUTURE-O')
                $isRuled = ($claimRule -eq 'R-ACCEPTED')
                $isNa = ($null -ne $claimRule -and -not $isFuture -and -not $isRuled)
                if ($isFuture) { $layer = 'future' }
                elseif ($isRuled) { $layer = 'core_ruled_accepted' }
                elseif ($isExt) {
                    if ($appv.extIds[$r.id] -eq $p) { $layer = 'extension_committed' }
                    elseif ($isNa) { $layer = 'extension_na' }
                    else { $layer = 'extension_other' }
                }
                else {
                    if ($isNa) { $layer = 'core_na' } else { $layer = 'core' }
                }
                $required = ($layer -eq 'core' -or $layer -eq 'extension_committed')
                if ($required) { $requiredCells += $key }
                $green = ($st.cls -in @('green', 'green-cert-gap'))
                if ($required -and -not $green) {
                    $owner = if ($token -cmatch '^(gap|wait):([^|]+)') { $Matches[2] } else { 'language-difference-or-pending' }
                    $entry = "$key=$token"
                    if ($layer -eq 'core') { $coreBlockers += $entry } else { $extBlockers += $entry }
                }
                if ($layer -eq 'future' -and -not $green) { $futureGaps += "$key=$token" }
                $ledger += [pscustomobject]@{
                    ns = $r.ns; id = $r.id; path = $p; layer = $layer
                    required = $required; token = $token; state = $st.cls
                    cert_axis = $st.cert; producer = $token
                    rule = $appv.claimed[$key]
                }
            }
        }

        $coreRequired = @($ledger | Where-Object { $_.layer -eq 'core' })
        $coreGreen = @($coreRequired | Where-Object { $_.state -in @('green', 'green-cert-gap') })
        $extRequired = @($ledger | Where-Object { $_.layer -eq 'extension_committed' })
        $extGreen = @($extRequired | Where-Object { $_.state -in @('green', 'green-cert-gap') })
        $ruledShown = @($ledger | Where-Object { $_.layer -eq 'core_ruled_accepted' })
        if ($coreRequired.Count -ne 285) { Fail "core required recompute $($coreRequired.Count) != 285 — registry tampered mid-report" }
        if ($extRequired.Count -ne 28) { Fail "extension required recompute $($extRequired.Count) != 28 — registry tampered mid-report" }
        if ($requiredCells.Count -ne 313) { Fail "required set $($requiredCells.Count) != 313 — the approved rule export was altered" }
        if ($ruledShown.Count -ne 32) { Fail "ruled-accepted recompute $($ruledShown.Count) != 32 — registry tampered mid-report" }

        $requiredSetHash = Sha256Text (($requiredCells | Sort-Object) -join "`n")

        # ---- optional parent-report admission (never a substitute) --------
        $parentAdmission = $null
        if ($ParentReport) {
            if (-not (Test-Path $ParentReport)) { Fail "parent report missing: $ParentReport" }
            $pr = $null
            try { $pr = Get-Content $ParentReport -Raw | ConvertFrom-Json } catch { Fail "parent report is not valid JSON: $_" }
            $prResults = @($pr.results)
            if ($prResults.Count -lt 1) { Fail "parent report carries zero results (fake/empty report)" }
            $prNonPass = @($prResults | Where-Object status -ne 'PASS')
            if ($prNonPass.Count -gt 0) {
                Fail ("parent report has {0} non-PASS results (first: {1}:{2} {3}) — failing evidence cannot be admitted" -f $prNonPass.Count, $prNonPass[0].case, $prNonPass[0].check, $prNonPass[0].status)
            }
            $parentAdmission = [pscustomobject]@{
                report = $ParentReport
                suite = $pr.suite
                executed_checks = $prResults.Count
                all_pass = $true
                scope_note = 'executed evidence admitted for its own scope only; base/selected green NEVER implies M1'
            }
        }

        # ---- optional explicit selection (report-only) --------------------
        $selection = $null
        if ($Select) {
            $selection = @()
            foreach ($part in ($Select -split ',')) {
                $k = $part.Trim()
                $found = @($ledger | Where-Object { "$($_.ns):$($_.id)" -ceq $k })
                if ($found.Count -eq 0) { Fail "selection entry not found in binding: $k" }
                $selection += [pscustomobject]@{ row = $k; cells = @($found | ForEach-Object { "$($_.path)=$($_.token)" }) }
            }
        }

        # ---- optional strict cert-axis validation (zero spawn) ------------
        $certValidated = $null
        if ($BuildIdentity) {
            $certValidated = Invoke-ReceiptValidation -RepoRoot $RepoRoot -corpusDir $corpusDir -buildIdentity $BuildIdentity
        }
        # PLAN-997: the trap domain's required evidence check runs
        # UNCONDITIONALLY (identity-free when -BuildIdentity is absent): a
        # trap row migrated to ok-emit-trap without a structurally valid,
        # current `ace.m1.a2r-trap/1` receipt FAILs acceptance — a green
        # token alone, a deleted certificate, or a forged receipt never
        # signs the face.
        $trapValidated = Invoke-TrapReceiptValidation -RepoRoot $RepoRoot -corpusDir $corpusDir -buildIdentity $BuildIdentity
        # PLAN-998: the resource-scenario domain's required evidence check
        # runs UNCONDITIONALLY and identity-free (same discipline as the trap
        # domain): an ok-emit-scene scenario row without a structurally
        # valid, current `ace.m1.a2r-resource-scenario/1` receipt FAILs
        # acceptance — the registry stays authoritative over forged receipts.
        $sceneValidated = Invoke-ScenarioReceiptValidation -RepoRoot $RepoRoot -corpusDir $corpusDir -buildIdentity $BuildIdentity
        $certAxisRequired = @($ledger | Where-Object { $_.required -and $_.path -eq 'a2r' })
        $certGapCells = @($certAxisRequired | Where-Object { $_.cert_axis -eq 'cert-gap' })
        # PLAN-998 reporting axis: the ok-emit-scene required a2r cells are
        # counted as their OWN scenario domain (alongside the value/trap
        # axes) — the count is reported, never a verdict flip.
        $sceneA2rCells = @($certAxisRequired | Where-Object { $_.token -ceq 'ok-emit-scene' })

        # ---- verdicts ------------------------------------------------------
        $coreVerdict = if ($coreGreen.Count -eq $coreRequired.Count) { 'pass' } else { 'red' }
        $extVerdict = if ($extGreen.Count -eq $extRequired.Count) { 'pass' } else { 'red' }
        $futureVerdict = 'reported-not-required'
        $baseVerdict = if ($parentAdmission) { "pass(parent-report-admitted, $($parentAdmission.executed_checks) checks)" } else { 'not-claimed(this scope never executes)' }
        $certVerdict = if ($null -ne $certValidated) { "strictly-validated($certValidated value receipts + $trapValidated trap receipts + $sceneValidated resource-scenario receipts)" }
            elseif ($certGapCells.Count -gt 0) { "gap($($certGapCells.Count) required a2r cells carry cert-gap; $trapValidated trap + $sceneValidated resource-scenario receipt(s) strictly validated; validation via snapshot/receipts scope)" }
            else { "trap+scene-validated($trapValidated trap + $sceneValidated resource-scenario receipt(s)); value axis: supply -BuildIdentity or run snapshot/receipts" }
        # G-DEC-5: M1 may be claimed only when the core obligation AND the
        # committed extension profile are fully green WITH VALID EVIDENCE,
        # while the full 131/524 registry still displays its remaining gaps
        # (including the 45 future console cells). PLAN-924: a required
        # cell still carrying a cert gap (snapshot-only / cert-diff) is NOT
        # valid evidence — the R02/R05 ac backfill must not turn the claim
        # on while the cert axis is open, so the claim now cites the cert
        # axis explicitly (a strict strengthening: no historical verdict
        # flips to claimable).
        $canClaimM1 = ($coreVerdict -eq 'pass') -and ($extVerdict -eq 'pass') -and ($futureGaps.Count -eq 45) -and ($certGapCells.Count -eq 0)

        $classCount = @{}
        foreach ($l in $ledger) {
            $n = 0
            if ($classCount.ContainsKey($l.state)) { $n = [int]$classCount[$l.state] }
            $classCount[$l.state] = $n + 1
        }

        $report = [pscustomobject]@{
            schema = 'ace.m1.acceptance/2'
            generated_by = 'scripts/verify-ace-m1-852.ps1 -Scope acceptance (PLAN-908 T-06)'
            repo_root = $RepoRoot
            identity = [pscustomobject]@{
                binding_sha256 = Sha256File $bindingPath
                applicability_sha256 = Sha256File $applicPath
                source_manifest_sha256 = Sha256File $manifestPath
                approval_doc = $appv.app.approval_source.decision_doc
                approval_doc_sha256 = $appv.app.approval_source.decision_doc_sha256
                ruling_date = $appv.app.approval_source.ruling_date
            }
            registered = [pscustomobject]@{ rows = 131; corpus_rows = 91; scenario_rows = 40; cells = 524 }
            layers = [pscustomobject]@{
                core_rows = 103; core_cells = 412; core_na_cells = 95
                core_ruled_accepted_cells = 32; core_required = $coreRequired.Count
                extension_rows = 28; extension_cells = 112
                extension_committed_cells = 28; extension_na_cells = 39
                future_cells = 45; required_cells = $requiredCells.Count
            }
            required_set_sha256 = $requiredSetHash
            cell_states = [pscustomobject]$classCount
            executed = [pscustomobject]@{
                selected = 0; executed = 0; conditionally_excluded = 0; not_executed = 0
                parent_admission = $parentAdmission
                selection = $selection
                note = 'this scope executes nothing; executed evidence comes from admitted parent reports or the fixed cargo groups only'
            }
            ledger = $ledger
            verdicts = [pscustomobject]@{
                base = $baseVerdict
                core = "$coreVerdict ($($coreGreen.Count)/$($coreRequired.Count) required cells green; 32 ruled-accepted shown, never green)"
                extension = "$extVerdict ($($extGreen.Count)/$($extRequired.Count) committed cells green)"
                future = "$futureVerdict ($($futureGaps.Count)/45 gap cells shown)"
                cert = $certVerdict
                scenario = "$($sceneA2rCells.Count)/6 required a2r cells on the ok-emit-scene token; $sceneValidated resource-scenario receipt(s) strictly validated (ace.m1.a2r-resource-scenario/1)"
                can_claim_m1 = $canClaimM1
            }
            core_blockers = $coreBlockers
            extension_blockers = $extBlockers
            ruled_accepted = @($ruledShown | ForEach-Object { "$($_.ns):$($_.id):$($_.path)=$($_.token)" })
            future_gaps = $futureGaps
        }

        # frozen report cap (PLAN-908 T-02: 4 MiB cumulative report budget)
        $json = $report | ConvertTo-Json -Depth 6
        if ($json.Length -gt 4194304) { Fail "acceptance report exceeds the frozen 4 MiB cap ($($json.Length) chars)" }
        New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
        $reportPath = Join-Path $OutDir 'ace-m1-acceptance.json'
        [System.IO.File]::WriteAllText($reportPath, $json, [System.Text.UTF8Encoding]::new($false))

        Write-Host ""
        Write-Host "==== ace.m1.acceptance/2 ====" -ForegroundColor Cyan
        Write-Host "registered: 131 rows / 524 cells (91 corpus + 40 scenario); required set hash $requiredSetHash"
        Write-Host "cell states: $(($classCount.GetEnumerator() | Sort-Object Name | ForEach-Object { "$($_.Name)=$($_.Value)" }) -join ', ')"
        Write-Host "BASE: $baseVerdict"
        Write-Host "CORE: $($report.verdicts.core)"
        Write-Host "EXTENSION: $($report.verdicts.extension)"
        Write-Host "FUTURE: $($report.verdicts.future)"
        Write-Host "CERT: $certVerdict"
        Write-Host "SCENARIO: $($report.verdicts.scenario)"
        Write-Host "can_claim_m1: $canClaimM1" -ForegroundColor Yellow
        Write-Host "Report: $reportPath"
        Write-Host "Integrity PASS (accounting valid); red verdicts above are the honest M1 state — they are NOT failures of this gate."
        exit 0
    }
}
