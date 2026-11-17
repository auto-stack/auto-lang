//! PLAN-1066/A slice — correctness denominator freeze + negative controls.
//!
//! Scope (plan §5 A-EXIT-20261116, §7 AC-A-CORRECTNESS, §6.3 A片段负控):
//!
//! * **A-DEN** — a frozen obligation map (`test/governance/
//!   plan1066_a_obligations.json`) over S11, N1050 (+1170 oracle roles),
//!   E78, VM cache 506/8 scripts, cookbook B108 bindings/6 multi-source
//!   dirs, P 2 projects/5 sources/19 assertions, the P250 compile domain +
//!   M guards, and C 30/33. [`derive_actual`] re-derives every set from the
//!   SAME sources the real consumers use (product discoverers, the wrapper
//!   cache, `syn` over the wrapper sources, on-disk bytes); the compare in
//!   [`compare_frozen`] diffs per obligation key — never by count alone.
//! * **A-ORACLE** — N0028/N0030 carry both `vm-stdout` and `vm-result`
//!   channels; the VM doc consumer collects both from ONE run and judges
//!   each axis independently (`vm_file_tests::run_vm_doc_once`). The nine
//!   zero-byte a2c header roles are frozen as `h-empty`, never "absent".
//! * **A-NOWRITE** — diagnostics/generation on A's executed normal paths go
//!   through `diag_scope` (the author tree stays byte-identical).
//! * **A-EVIDENCE** — [`ARReceipt`] records per-check source/oracle identity,
//!   engine identity and channel observations; [`verify_receipt`] rejects
//!   stale hashes, old-sidecar results posing as doc-face, parent-pass with
//!   failed children, missing tools and missing channels.
//!
//! Engine identity note (§5): the actuals recorded here run on the legacy
//! AutoVM interpreter — the ORIGINAL engine these golden channels were
//! captured on. Nothing in this module claims ACE producer identity; that
//! dispatch stays with the 985/A adapter contract.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use auto_test_case::sha256::{hex, sha256};
use serde::{Deserialize, Serialize};

use crate::tests::vm_file_tests::{
    diff_vm_channels, run_vm_doc_once, vm_doc_oracle, VmChannelAxis,
};

// ---------------------------------------------------------------------------
// paths + hashing
// ---------------------------------------------------------------------------

/// `crates/auto-lang` (this crate's manifest dir).
fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Repository (worktree) root — the prefix all frozen fixture paths use
/// (`crates/auto-lang/test/...`), matching the plan's tables.
fn root() -> PathBuf {
    repo()
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("manifest dir sits at <root>/crates/auto-lang")
}

fn test_dir(family: &str) -> PathBuf {
    repo().join("test").join(family)
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root())
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Join a worktree-root-relative fixture path.
fn root_join(rel_path: &str) -> PathBuf {
    root().join(rel_path)
}

fn sha256_file(p: &Path) -> Result<String, String> {
    let bytes = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
    Ok(hex(&sha256(&bytes)))
}

// ---------------------------------------------------------------------------
// frozen fixture model
// ---------------------------------------------------------------------------

pub(crate) const FIXTURE_SCHEMA: &str = "plan1066-a-obligations/1";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct Frozen {
    pub schema: String,
    /// Family defaults shared by every n-row of that family (the
    /// A/B/platform/ignore/actual-source mapping A-DEN requires).
    pub segments: BTreeMap<String, SegmentMeta>,
    pub n_rows: Vec<NRow>,
    pub vm_cache: BTreeMap<String, VmCacheEntry>,
    pub cookbook_bindings: Vec<CookbookBinding>,
    pub cookbook_multi_source_dirs: Vec<MultiSourceDir>,
    pub s11: Vec<S11Case>,
    pub p_projects: PProjects,
    pub p_compile: PCompile,
    pub c_bootstrap: Vec<String>,
    pub c_verify: Vec<CVerify>,
    pub e78: Vec<DocHash>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SegmentMeta {
    pub stage: String,
    pub platform: String,
    pub ignore: bool,
    pub actual_source: String,
    pub discovery: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NRow {
    pub family: String,
    /// Obligation key within the family, e.g. `04_control_flow/004_if_with_bool`.
    /// family + id is the obligation key — ids repeat across families.
    pub id: String,
    pub doc_rel: String,
    pub doc_sha256: String,
    /// Decoded check ids in the doc (vm-stdout/vm-result/vm-error; rust;
    /// a2c-emit; cookbook-stdout ...), sorted.
    pub doc_checks: Vec<String>,
    /// Decoded artifact logical paths (e.g. hello.expected.c), sorted.
    pub doc_artifacts: Vec<String>,
    /// Frozen old-file oracle roles: out/result/error (vm), rs (a2r/cookbook),
    /// c/h/h-empty/error-log (a2c), ts (a2ts). `h-empty` = zero-byte header
    /// judgment — an explicit role, never absence.
    pub roles: Vec<String>,
    pub source_kind: String,
    pub source_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct VmCacheEntry {
    pub is_script: bool,
    pub stdout: bool,
    pub result: bool,
    pub error: bool,
}

/// A cookbook VM binding dir with more than one `.at` candidate: the
/// wrapper selects one by its own `read_dir` scan; the selection and the
/// candidate list are frozen exactly as the wrapper sees them (same FS
/// iteration), and only a doc for the SELECTED stem is loadable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct MultiSourceDir {
    pub case: String,
    /// Sorted candidate `.at` file names (stability of the frozen set).
    pub ats: Vec<String>,
    /// The candidate the wrapper's unsorted `read_dir` scan actually picks.
    pub selected_at: String,
    /// Whether the selected stem has a migration doc (no-doc candidates are
    /// never auto-promoted).
    pub selected_doc: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CookbookBinding {
    pub binding: String,
    pub category: String,
    pub case: String,
    pub ignored: bool,
    pub cfg_attr_ignore: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct S11Case {
    pub id: String,
    pub source_rel: String,
    pub source_sha256: String,
    /// sha256 of the whole ```test-case block (pins id + expect verbatim).
    pub block_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct PProjects {
    pub sources: Vec<DocHash>,
    pub assertions: Vec<PAssertion>,
    pub old_tests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PAssertion {
    /// Ordinal within the legacy test (`<test>#<n>`, order-sensitive).
    pub id: String,
    pub test: String,
    pub product: String,
    pub kind: String, // "exists" | "contains"
    pub literal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct PCompile {
    pub gate_test: String,
    pub gate_ignored: bool,
    pub ledger_rel: String,
    pub ledger_sha256: String,
    pub ledger_entries: usize,
    pub m_guards: Vec<String>,
    pub m_products: Vec<DocHash>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CVerify {
    pub test: String,
    pub case: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct DocHash {
    pub path: String,
    pub sha256: String,
}

// ---------------------------------------------------------------------------
// Rust-source mini-lexer: comments stripped, literals hoisted to markers.
// ---------------------------------------------------------------------------

/// `clean` has `//`/`/* */` comments removed and string/char literals
/// replaced by `"§N§"` markers; `literals[N]` is the unescaped value.
fn strip_rust(src: &str) -> (String, Vec<String>) {
    let mut clean = String::with_capacity(src.len());
    let mut literals = Vec::new();
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        // line comment
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // block comment
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i = (i + 2).min(chars.len());
            clean.push(' ');
            continue;
        }
        // string literal → hoisted to a quoted marker "§N§" so patterns like
        // `contains_key("…")` still scan naturally while the payload lives in
        // the side table.
        if c == '"' {
            let (value, next) =
                read_rust_string(&chars, i).expect("valid literal in wrapper source");
            literals.push(value);
            clean.push('"');
            clean.push('\u{00a7}');
            clean.push_str(&(literals.len() - 1).to_string());
            clean.push('\u{00a7}');
            clean.push('"');
            i = next;
            continue;
        }
        // char literal
        if c == '\'' {
            // `'\''` / `'x'` / `'\n'`; raw lifetimes ('a) don't occur in bodies
            let mut j = i + 1;
            if j < chars.len() && chars[j] == '\\' {
                j += 2;
            } else if j < chars.len() {
                j += 1;
            }
            if j < chars.len() && chars[j] == '\'' {
                clean.push_str("'c'");
                i = j + 1;
                continue;
            }
        }
        clean.push(c);
        i += 1;
    }
    (clean, literals)
}

/// Read a Rust string literal starting at `chars[i] == '"'`; returns the
/// unescaped value and the index just past the closing quote.
fn read_rust_string(chars: &[char], i: usize) -> Option<(String, usize)> {
    if i >= chars.len() || chars[i] != '"' {
        return None;
    }
    let mut i = i + 1;
    let mut out = String::new();
    while i < chars.len() {
        match chars[i] {
            '\\' => {
                i += 1;
                if i >= chars.len() {
                    return None;
                }
                match chars[i] {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    other => {
                        out.push('\\');
                        out.push(other);
                    }
                }
            }
            '"' => return Some((out, i + 1)),
            other => out.push(other),
        }
        i += 1;
    }
    None
}

/// Literal value for the marker starting at `clean[i]` (`"§N§"`); returns
/// the value and the index just past the marker.
fn marker_at(clean: &[char], i: usize, literals: &[String]) -> Option<(String, usize)> {
    if i >= clean.len() || clean[i] != '"' {
        return None;
    }
    let mut j = i + 1;
    let mut num = String::new();
    while j < clean.len() && clean[j] != '"' {
        num.push(clean[j]);
        j += 1;
    }
    if j >= clean.len() {
        return None;
    }
    let idx: usize = num.trim_matches('\u{00a7}').parse().ok()?;
    literals.get(idx).cloned().map(|v| (v, j + 1))
}

/// Raw body of `fn <name>` from the CLEAN text (brace-matched; literals are
/// markers so embedded braces cannot skew the count).
fn fn_body_clean<'a>(clean: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("fn {name}(");
    let start = clean.find(&needle)?;
    let open = start + clean[start..].find('{')?;
    let bytes = clean.as_bytes();
    let mut depth = 0usize;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&clean[open + 1..i]);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

// ---------------------------------------------------------------------------
// derivation — same-source as the real consumers
// ---------------------------------------------------------------------------

fn stem_of(dir_name: &str) -> String {
    dir_name
        .split_once('_')
        .map(|(_, s)| s)
        .unwrap_or(dir_name)
        .to_string()
}

/// N rows via the PRODUCT discoverers (the same `test_runner::discover_*`
/// the sys_env shims and entry impls use). Old-file presence defines the
/// frozen selection; docs are decoded through the same `legacy_doc::load_case`
/// the wrappers use.
fn derive_n_rows() -> Result<Vec<NRow>, String> {
    let mut rows: Vec<NRow> = Vec::new();

    for case in crate::test_runner::discover_vm_tests(&test_dir("vm")) {
        let dir = case.dir.clone();
        let stem = stem_of(&dir.file_name().unwrap().to_string_lossy());
        let doc = dir.join(format!("{stem}.case.ad"));
        let entry = crate::tests::legacy_doc::load_case(&dir, &stem)?;
        let oracle = vm_doc_oracle(&entry);
        let mut doc_checks: Vec<String> = entry
            .case
            .spec
            .checks
            .iter()
            .map(|c| c.id.clone())
            .collect();
        doc_checks.sort();
        let mut roles = Vec::new();
        if case.expected_out.is_some() {
            roles.push("out".to_string());
        }
        if case.expected_result.is_some() {
            roles.push("result".to_string());
        }
        if case.expected_error.is_some() {
            roles.push("error".to_string());
        }
        roles.sort();
        // A-ORACLE: the vm doc must carry every frozen oracle channel (the
        // two restored results closed this gap for good).
        let have: Vec<&str> = [
            oracle.stdout.as_ref().map(|_| "out"),
            oracle.result.as_ref().map(|_| "result"),
            if oracle.error { Some("error") } else { None },
        ]
        .into_iter()
        .flatten()
        .collect();
        let missing: Vec<&str> = roles
            .iter()
            .filter(|r| !have.contains(&r.as_str()))
            .map(|s| s.as_str())
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "vm doc {} lost frozen oracle channels {missing:?} (doc has {have:?})",
                case.name
            ));
        }
        rows.push(NRow {
            family: "vm".into(),
            id: case.name.clone(),
            doc_rel: rel(&doc),
            doc_sha256: sha256_file(&doc)?,
            doc_checks,
            doc_artifacts: Vec::new(),
            roles,
            source_kind: if case.source_file.extension().and_then(|e| e.to_str()) == Some("as") {
                "as".into()
            } else {
                "at".into()
            },
            source_sha256: sha256_file(&case.source_file)?,
        });
    }

    for family in ["a2r", "cookbook"] {
        for case in crate::test_runner::discover_a2r_tests(&test_dir(family), family) {
            let dir = case.dir.clone();
            let stem = stem_of(&dir.file_name().unwrap().to_string_lossy());
            let doc = dir.join(format!("{stem}.case.ad"));
            let (doc_checks, doc_artifacts) = doc_check_artifacts(&dir, &stem)?;
            rows.push(NRow {
                family: family.into(),
                id: case
                    .name
                    .strip_prefix(&format!("{family}/"))
                    .unwrap_or(&case.name)
                    .to_string(),
                doc_rel: rel(&doc),
                doc_sha256: sha256_file(&doc)?,
                doc_checks,
                doc_artifacts,
                roles: vec!["rs".into()],
                source_kind: "at".into(),
                source_sha256: sha256_file(&case.source_file)?,
            });
        }
    }

    for case in crate::test_runner::discover_a2c_tests(&test_dir("a2c")) {
        let dir = case.dir.clone();
        let stem = stem_of(&dir.file_name().unwrap().to_string_lossy());
        let doc = dir.join(format!("{stem}.case.ad"));
        let (doc_checks, doc_artifacts) = doc_check_artifacts(&dir, &stem)?;
        let mut roles = vec!["c".to_string()];
        roles.push(if case.expected_h.is_file() {
            "h".into()
        } else {
            "h-empty".into()
        });
        if case.expected_error.is_some() {
            roles.push("error-log".into());
        }
        rows.push(NRow {
            family: "a2c".into(),
            id: case.name.clone(),
            doc_rel: rel(&doc),
            doc_sha256: sha256_file(&doc)?,
            doc_checks,
            doc_artifacts,
            roles,
            source_kind: "at".into(),
            source_sha256: sha256_file(&case.source_file)?,
        });
    }

    for case in crate::test_runner::discover_a2ts_tests(&test_dir("a2ts")) {
        let dir = case.dir.clone();
        let stem = stem_of(&dir.file_name().unwrap().to_string_lossy());
        let doc = dir.join(format!("{stem}.case.ad"));
        let (doc_checks, doc_artifacts) = doc_check_artifacts(&dir, &stem)?;
        rows.push(NRow {
            family: "a2ts".into(),
            id: case.name.clone(),
            doc_rel: rel(&doc),
            doc_sha256: sha256_file(&doc)?,
            doc_checks,
            doc_artifacts,
            roles: vec!["ts".into()],
            source_kind: "at".into(),
            source_sha256: sha256_file(&case.source_file)?,
        });
    }

    rows.sort_by(|a, b| a.family.cmp(&b.family).then(a.id.cmp(&b.id)));
    Ok(rows)
}

fn doc_check_artifacts(dir: &Path, stem: &str) -> Result<(Vec<String>, Vec<String>), String> {
    let entry = crate::tests::legacy_doc::load_case(dir, stem)?;
    let mut checks: Vec<String> = entry
        .case
        .spec
        .checks
        .iter()
        .map(|c| c.id.clone())
        .collect();
    checks.sort();
    let mut artifacts: Vec<String> = entry
        .case
        .spec
        .checks
        .iter()
        .flat_map(|c| c.expect.artifacts.iter().map(|a| a.path.clone()))
        .collect();
    artifacts.sort();
    Ok((checks, artifacts))
}

fn derive_vm_cache() -> BTreeMap<String, VmCacheEntry> {
    crate::tests::vm_file_tests::vm_cache_snapshot()
        .into_iter()
        .map(|(k, (is_script, stdout, result, error))| {
            (
                k,
                VmCacheEntry {
                    is_script,
                    stdout,
                    result,
                    error,
                },
            )
        })
        .collect()
}

/// Cookbook bindings — `syn` over the wrapper source (the same technique the
/// read-only inventory uses): a declaration is routing metadata, and its
/// ignore/cfg_attr status is part of the frozen obligation.
fn derive_cookbook_bindings() -> Result<Vec<CookbookBinding>, String> {
    let src = include_str!("cookbook_vm_tests.rs");
    let syntax = syn::parse_file(src).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for item in syntax.items {
        let syn::Item::Fn(f) = item else { continue };
        if !f.attrs.iter().any(|a| a.path().is_ident("test")) {
            continue;
        }
        // body: test_cookbook("category", "name").unwrap()
        let syn::Stmt::Expr(syn::Expr::MethodCall(method), _) = &f.block.stmts[0] else {
            continue;
        };
        if method.method != "unwrap" || !method.args.is_empty() {
            continue;
        }
        let syn::Expr::Call(call) = &*method.receiver else {
            continue;
        };
        let syn::Expr::Path(p) = &*call.func else {
            continue;
        };
        if !p.path.is_ident("test_cookbook") || call.args.len() != 2 {
            continue;
        }
        let lit = |e: &syn::Expr| -> Option<String> {
            if let syn::Expr::Lit(l) = e {
                if let syn::Lit::Str(s) = &l.lit {
                    return Some(s.value());
                }
            }
            None
        };
        let (Some(category), Some(case_name)) = (lit(&call.args[0]), lit(&call.args[1])) else {
            continue;
        };
        out.push(CookbookBinding {
            binding: f.sig.ident.to_string(),
            category: category.clone(),
            case: format!("{category}/{case_name}"),
            ignored: f.attrs.iter().any(|a| a.path().is_ident("ignore")),
            cfg_attr_ignore: f.attrs.iter().any(|a| {
                a.path().is_ident("cfg_attr")
                    && a.meta
                        .require_list()
                        .map(|l| l.tokens.to_string().contains("ignore"))
                        .unwrap_or(false)
            }),
        });
    }
    out.sort_by(|a, b| a.binding.cmp(&b.binding));
    Ok(out)
}

/// Cookbook VM binding dirs holding more than one `.at` candidate (the six
/// multi-source dirs of plan appendix B). The candidate set AND the
/// wrapper's actual selection (same unsorted `read_dir` scan the wrapper
/// uses) are frozen; no-doc candidates are recorded, never promoted.
fn derive_cookbook_multi_source(bindings: &[CookbookBinding]) -> Vec<MultiSourceDir> {
    let mut dirs = Vec::new();
    for b in bindings {
        let dir = test_dir("cookbook").join(&b.case);
        let entries: Vec<_> = match std::fs::read_dir(&dir) {
            Ok(rd) => rd.flatten().collect(),
            Err(_) => continue,
        };
        // Exact wrapper semantics: first `.at` in unsorted read_dir order.
        let selected = entries
            .iter()
            .find(|e| e.path().extension().and_then(|x| x.to_str()) == Some("at"))
            .map(|e| e.file_name().to_string_lossy().to_string());
        let mut ats: Vec<String> = entries
            .iter()
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("at"))
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        ats.sort();
        if ats.len() > 1 {
            let selected_at = selected.unwrap_or_else(|| ats[0].clone());
            let stem = selected_at.trim_end_matches(".at");
            dirs.push(MultiSourceDir {
                case: b.case.clone(),
                ats,
                selected_doc: dir.join(format!("{stem}.case.ad")).is_file(),
                selected_at,
            });
        }
    }
    dirs.sort_by(|a, b| a.case.cmp(&b.case));
    dirs
}

/// S11 — the scheduler suite's old fence: one ```test-case block per case,
/// each pinned by the block's own bytes plus the src/*.at bytes.
fn derive_s11() -> Result<Vec<S11Case>, String> {
    let suite_dir = repo().join("test/cases/ace-m2/scheduler");
    let text = std::fs::read_to_string(suite_dir.join("scheduler.case.ad"))
        .map_err(|e| format!("scheduler.case.ad: {e}"))?;
    let mut out = Vec::new();
    for block in fence_blocks(&text, "test-case") {
        let id = tagged_string(&block, "id").ok_or("scheduler block without id")?;
        let file = tagged_string(&block, "file").ok_or("scheduler block without source file")?;
        let src = suite_dir.join(&file);
        out.push(S11Case {
            id,
            source_rel: format!("crates/auto-lang/test/cases/ace-m2/scheduler/{file}"),
            source_sha256: sha256_file(&src)?,
            block_sha256: hex(&sha256(block.as_bytes())),
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

fn fence_blocks(text: &str, info: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        let t = line.trim();
        if t.starts_with("```") && t[3..].trim() == info {
            let mut block = String::new();
            for inner in lines.by_ref() {
                if inner.trim().starts_with("```") {
                    blocks.push(block);
                    break;
                }
                block.push_str(inner);
                block.push('\n');
            }
        }
    }
    blocks
}

/// First `key: "value"` (Rust-style escaped string) in a v2 block.
fn tagged_string(block: &str, key: &str) -> Option<String> {
    let needle = format!("{key}: \"");
    let start = block.find(&needle)? + needle.len();
    // read_rust_string expects the opening quote; `start` sits just past it,
    // so back up one byte (the quote is ASCII).
    let chars: Vec<char> = block[start - 1..].chars().collect();
    read_rust_string(&chars, 0).map(|(v, _)| v)
}

/// P projects: the five real project sources on disk plus the 19 original
/// assertions derived from the two legacy Rust tests (same source as the
/// obligation).
fn derive_p_projects() -> Result<PProjects, String> {
    let base = "crates/auto-lang/test/a2r";
    let mut paths = vec![
        format!("{base}/14_modules/005_multi_file/api/handlers.at"),
        format!("{base}/14_modules/005_multi_file/api/mod.at"),
        format!("{base}/14_modules/005_multi_file/db.at"),
        format!("{base}/14_modules/005_multi_file/main.at"),
        format!("{base}/16_interop/021_path_dep/main.at"),
    ];
    paths.sort();
    let mut sources = Vec::new();
    for p in &paths {
        sources.push(DocHash {
            path: p.clone(),
            sha256: sha256_file(&root_join(p))?,
        });
    }

    // PLAN-1066 T-05: the 19 assertions now live in the two project
    // documents (the Rust test bodies became doc consumers), so the frozen
    // denominator is derived from the documents in declaration order:
    // per product, `exists` first and then every `contains` in order.
    let mut assertions = Vec::new();
    for (test, relative) in [
        (
            "test_14_modules_005_multi_file",
            "14_modules/005_multi_file",
        ),
        ("test_16_interop_021_path_dep", "16_interop/021_path_dep"),
    ] {
        let doc = crate::tests::legacy_doc::load_document(&root_join(&format!(
            "{base}/{relative}/main.project.case.ad"
        )))?;
        let check = doc
            .case
            .spec
            .checks
            .iter()
            .find(|check| check.id == "project")
            .ok_or_else(|| format!("{test}: project document has no project check"))?;
        for artifact in &check.expect.artifacts {
            let declared_exists = artifact.match_mode.declares_exists();
            if declared_exists {
                assertions.push(PAssertion {
                    id: format!("{test}#{}", assertions.len() + 1),
                    test: test.to_string(),
                    product: artifact.path.clone(),
                    kind: "exists".into(),
                    literal: None,
                });
            }
            for needle in artifact.match_mode.contains() {
                assertions.push(PAssertion {
                    id: format!("{test}#{}", assertions.len() + 1),
                    test: test.to_string(),
                    product: artifact.path.clone(),
                    kind: "contains".into(),
                    literal: Some(needle.clone()),
                });
            }
        }
    }

    Ok(PProjects {
        sources,
        assertions,
        old_tests: vec![
            "test_14_modules_005_multi_file".into(),
            "test_16_interop_021_path_dep".into(),
        ],
    })
}

/// Ordered scan over the CLEAN body: `contains_key("§N§")` → exists,
/// `result["§N§"]` sets the current product, `.contains("§N§")` → contains.
fn parse_project_assertions(
    test: &str,
    body: &str,
    literals: &[String],
) -> Result<Vec<PAssertion>, String> {
    let mut out = Vec::new();
    let mut current_product = String::new();
    let chars: Vec<char> = body.chars().collect();
    let starts_with = |i: usize, pat: &str| -> bool {
        chars[i..].starts_with(&pat.chars().collect::<Vec<_>>()[..])
    };
    let mut i = 0usize;
    while i < chars.len() {
        if starts_with(i, "contains_key(") {
            i += "contains_key(".len();
            if let Some((lit, next)) = marker_at(&chars, i, literals) {
                out.push(PAssertion {
                    id: format!("{test}#{}", out.len() + 1),
                    test: test.to_string(),
                    product: lit.clone(),
                    kind: "exists".into(),
                    literal: None,
                });
                current_product = lit;
                i = next;
                continue;
            }
        } else if starts_with(i, "result[") {
            i += "result[".len();
            if let Some((lit, next)) = marker_at(&chars, i, literals) {
                current_product = lit;
                i = next;
                continue;
            }
        } else if starts_with(i, ".contains(") {
            i += ".contains(".len();
            if let Some((lit, next)) = marker_at(&chars, i, literals) {
                if !current_product.is_empty() {
                    out.push(PAssertion {
                        id: format!("{test}#{}", out.len() + 1),
                        test: test.to_string(),
                        product: current_product.clone(),
                        kind: "contains".into(),
                        literal: Some(lit),
                    });
                }
                i = next;
                continue;
            }
        }
        i += 1;
    }
    Ok(out)
}

/// P compile domain: the un-ignored 250-input rustc gate, its known-broken
/// ledger, the three ignored M guards and their six golden products.
fn derive_p_compile() -> Result<PCompile, String> {
    let src = include_str!("a2r_tests.rs");
    let syntax = syn::parse_file(src).map_err(|e| e.to_string())?;
    let mut fns = Vec::new();
    for item in syntax.items {
        let syn::Item::Fn(f) = item else { continue };
        if !f.attrs.iter().any(|a| a.path().is_ident("test")) {
            continue;
        }
        fns.push((
            f.sig.ident.to_string(),
            f.attrs.iter().any(|a| a.path().is_ident("ignore")),
        ));
    }
    let gate_ignored = fns
        .iter()
        .find(|(n, _)| n == "a2r_rustc_real_compile_gate")
        .map(|(_, ig)| *ig)
        .ok_or("a2r_rustc_real_compile_gate missing")?;
    let guards = [
        "a2r_compile_smoke_str_param_borrow",
        "a2r_foreign_shape_compile_run",
        "a2r_capstone_term_subset_parity",
    ];
    for g in guards {
        let ig = fns.iter().find(|(n, _)| n == g).map(|(_, ig)| *ig);
        match ig {
            Some(true) => {}
            Some(false) => return Err(format!("M guard {g} must keep its explicit #[ignore]")),
            None => return Err(format!("M guard {g} missing")),
        }
    }

    let ledger_rel = "crates/auto-lang/test/a2r/compile_gate_known_broken.txt";
    let ledger_bytes = std::fs::read(root_join(ledger_rel)).map_err(|e| format!("ledger: {e}"))?;
    let ledger_text = String::from_utf8_lossy(&ledger_bytes);
    let entries = ledger_text
        .lines()
        .map(|l| l.split('#').next().unwrap_or("").trim())
        .filter(|l| !l.is_empty())
        .count();

    let mut products = vec![
        "crates/auto-lang/test/a2r/04_strings/008_str_param_borrow/str_param_borrow.expected.rs",
        "crates/auto-lang/test/a2r/25_foreign_types/001_foreign_generic_field/foreign_generic_field.expected.rs",
        "crates/auto-lang/test/a2r/25_foreign_types/002_foreign_trait_impl/foreign_trait_impl.expected.rs",
        "crates/auto-lang/test/a2r/25_foreign_types/003_trait_object/trait_object.expected.rs",
        "crates/auto-lang/test/a2r/25_foreign_types/004_bare_thread/bare_thread.expected.rs",
        "crates/auto-lang/test/a2r/25_foreign_types/005_term_session_subset/term_session_subset.expected.rs",
    ];
    products.sort();
    let mut m_products = Vec::new();
    for p in &products {
        m_products.push(DocHash {
            path: (*p).to_string(),
            sha256: sha256_file(&root_join(p))?,
        });
    }

    Ok(PCompile {
        gate_test: "a2r_rustc_real_compile_gate".into(),
        gate_ignored,
        ledger_rel: ledger_rel.to_string(),
        ledger_sha256: hex(&sha256(&ledger_bytes)),
        ledger_entries: entries,
        m_guards: guards.iter().map(|s| s.to_string()).collect(),
        m_products,
    })
}

/// C sets: the bootstrap closed 30 (from the bootstrap body) and the 33
/// independent verifications (wrappers calling run_conformance_test).
fn derive_c_sets() -> Result<(Vec<String>, Vec<CVerify>), String> {
    let (clean, literals) = strip_rust(include_str!("conformance_tests.rs"));
    let bootstrap_body =
        fn_body_clean(&clean, "conformance_bootstrap").ok_or("bootstrap fn missing")?;
    let mut bootstrap = Vec::new();
    let chars: Vec<char> = bootstrap_body.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '"' {
            if let Some((lit, next)) = marker_at(&chars, i, &literals) {
                if lit.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    bootstrap.push(lit);
                }
                i = next;
                continue;
            }
        }
        i += 1;
    }
    bootstrap.sort();

    let syntax =
        syn::parse_file(include_str!("conformance_tests.rs")).map_err(|e| e.to_string())?;
    let mut verify = Vec::new();
    for item in syntax.items {
        let syn::Item::Fn(f) = item else { continue };
        if !f.attrs.iter().any(|a| a.path().is_ident("test")) {
            continue;
        }
        let syn::Stmt::Expr(syn::Expr::MethodCall(method), _) = &f.block.stmts[0] else {
            continue;
        };
        if method.method != "unwrap" || !method.args.is_empty() {
            continue;
        }
        let syn::Expr::Call(call) = &*method.receiver else {
            continue;
        };
        let syn::Expr::Path(p) = &*call.func else {
            continue;
        };
        if !p.path.is_ident("run_conformance_test") || call.args.len() != 1 {
            continue;
        }
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &call.args[0]
        else {
            continue;
        };
        verify.push(CVerify {
            test: f.sig.ident.to_string(),
            case: s.value(),
        });
    }
    verify.sort_by(|a, b| a.test.cmp(&b.test));
    Ok((bootstrap, verify))
}

/// E78 — every migration doc under the five N roots that is NOT one of the
/// N1050 docs (each keeps its own consumer/ignore identity; none may be
/// silently absorbed into N).
fn derive_e78(n_rows: &[NRow]) -> Result<Vec<DocHash>, String> {
    let n_docs: std::collections::BTreeSet<&str> =
        n_rows.iter().map(|r| r.doc_rel.as_str()).collect();
    let mut out = Vec::new();
    for family in ["vm", "a2r", "a2c", "a2ts", "cookbook"] {
        let mut stack = vec![test_dir(family)];
        while let Some(dir) = stack.pop() {
            let entries = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().and_then(|x| x.to_str()) == Some("ad")
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.ends_with(".case.ad"))
                {
                    let r = rel(&p);
                    if !n_docs.contains(r.as_str()) {
                        out.push(DocHash {
                            path: r,
                            sha256: sha256_file(&p)?,
                        });
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Full same-source derivation of the A denominator.
pub(crate) fn derive_actual() -> Result<Frozen, String> {
    let n_rows = derive_n_rows()?;
    let (c_bootstrap, c_verify) = derive_c_sets()?;
    let mut segments = BTreeMap::new();
    segments.insert(
        "vm".to_string(),
        SegmentMeta {
            stage: "A+B".into(),
            platform: "windows".into(),
            ignore: false,
            actual_source: "doc wrapper tests::vm_file_tests::test_vm (payload since PLAN-1062 T-14) and entry::run_vm_file_test both read the document (PLAN-1066 T-03[B]); frozen sidecars are never read".into(),
            discovery: "test_runner::discover_vm_tests: `<stem>.case.ad` is the selection authority (PLAN-1066 T-03[B]); sidecars only keep a removed document visible as a loud failure".into(),
        },
    );
    for fam in ["a2r", "cookbook"] {
        segments.insert(
            fam.to_string(),
            SegmentMeta {
                stage: "A+B".into(),
                platform: "windows".into(),
                ignore: false,
                actual_source: "doc wrapper tests::a2r_tests::test_a2r_with_base and entry::run_a2r_file_test read the document (PLAN-1066 T-03[B]); frozen sidecars are never read".into(),
                discovery: "test_runner::discover_a2r_tests: `<stem>.case.ad` selects the entry and its `.rs` artifact is the oracle (PLAN-1066 T-03[B])".into(),
            },
        );
    }
    segments.insert(
        "a2c".to_string(),
        SegmentMeta {
            stage: "A+B".into(),
            platform: "windows".into(),
            ignore: false,
            actual_source: "doc wrapper tests::a2c_tests (per-case, #[ignore]d) and entry::run_a2c_file_test read the document (PLAN-1066 T-03[B]); frozen sidecars are never read".into(),
            discovery: "test_runner::discover_a2c_tests: `<stem>.case.ad` selects the entry and its `.c`/`.h` artifacts are the oracle (PLAN-1066 T-03[B])".into(),
        },
    );
    segments.insert(
        "a2ts".to_string(),
        SegmentMeta {
            stage: "A+B".into(),
            platform: "windows".into(),
            ignore: false,
            actual_source: "doc wrapper tests::a2ts_tests and entry::run_a2ts_file_test read the document (PLAN-1066 T-03[B]); frozen sidecars are never read".into(),
            discovery: "test_runner::discover_a2ts_tests: `<stem>.case.ad` selects the entry and its `.ts` artifact is the oracle (PLAN-1066 T-03[B])".into(),
        },
    );

    let cookbook_bindings = derive_cookbook_bindings()?;
    let cookbook_multi_source_dirs = derive_cookbook_multi_source(&cookbook_bindings);
    Ok(Frozen {
        schema: FIXTURE_SCHEMA.to_string(),
        segments,
        vm_cache: derive_vm_cache(),
        cookbook_bindings,
        cookbook_multi_source_dirs,
        s11: derive_s11()?,
        p_projects: derive_p_projects()?,
        p_compile: derive_p_compile()?,
        c_bootstrap,
        c_verify,
        e78: derive_e78(&n_rows)?,
        n_rows,
    })
}

// ---------------------------------------------------------------------------
// per-key comparison (A-DEN / NC-A1)
// ---------------------------------------------------------------------------

/// Compare a (possibly tampered) frozen map against the derived actual.
/// Every discrepancy is named per obligation key — a count-only diff can
/// never stand in for a per-key diff.
pub(crate) fn compare_frozen(frozen: &Frozen, actual: &Frozen) -> Vec<String> {
    let mut errs = Vec::new();
    if frozen.schema != FIXTURE_SCHEMA {
        errs.push(format!(
            "schema: frozen {:?} != {FIXTURE_SCHEMA:?}",
            frozen.schema
        ));
    }

    // F-1066A-R2: the family-level A/B/platform/ignore/actual-source
    // mapping is itself a frozen obligation — an edited segment must not
    // keep the integrity gate green.
    if frozen.segments != actual.segments {
        for fam in ["vm", "a2r", "a2c", "a2ts", "cookbook"] {
            if frozen.segments.get(fam) != actual.segments.get(fam) {
                errs.push(format!(
                    "segment[{fam}] meta differs: frozen {:?} != actual {:?}",
                    frozen.segments.get(fam),
                    actual.segments.get(fam)
                ));
            }
        }
    }
    for family in ["vm", "a2r", "a2c", "a2ts", "cookbook"] {
        let fr: BTreeMap<&str, &NRow> = frozen
            .n_rows
            .iter()
            .filter(|r| r.family == family)
            .map(|r| (r.id.as_str(), r))
            .collect();
        let ac: BTreeMap<&str, &NRow> = actual
            .n_rows
            .iter()
            .filter(|r| r.family == family)
            .map(|r| (r.id.as_str(), r))
            .collect();
        for (id, row) in &fr {
            match ac.get(*id) {
                None => errs.push(format!(
                    "n[{family}] {id}: frozen obligation missing from actual (deleted case/doc?)"
                )),
                Some(a) => {
                    if row.doc_rel != a.doc_rel {
                        errs.push(format!(
                            "n[{family}] {id}: doc path {:?} != actual {:?}",
                            row.doc_rel, a.doc_rel
                        ));
                    }
                    if row.doc_sha256 != a.doc_sha256 {
                        errs.push(format!(
                            "n[{family}] {id}: doc bytes changed (frozen {} != actual {})",
                            row.doc_sha256, a.doc_sha256
                        ));
                    }
                    if row.roles != a.roles {
                        errs.push(format!(
                            "n[{family}] {id}: oracle roles {:?} != actual {:?} (h-empty is an explicit zero-byte role, never absence)",
                            row.roles, a.roles
                        ));
                    }
                    if row.doc_checks != a.doc_checks {
                        errs.push(format!(
                            "n[{family}] {id}: doc checks {:?} != actual {:?} (channel conservation)",
                            row.doc_checks, a.doc_checks
                        ));
                    }
                    if row.doc_artifacts != a.doc_artifacts {
                        errs.push(format!(
                            "n[{family}] {id}: doc artifacts {:?} != actual {:?}",
                            row.doc_artifacts, a.doc_artifacts
                        ));
                    }
                    if row.source_sha256 != a.source_sha256 {
                        errs.push(format!(
                            "n[{family}] {id}: frozen source {} != actual {}",
                            row.source_sha256, a.source_sha256
                        ));
                    }
                    if row.source_kind != a.source_kind {
                        errs.push(format!(
                            "n[{family}] {id}: source kind {} != actual {}",
                            row.source_kind, a.source_kind
                        ));
                    }
                }
            }
        }
        for id in ac.keys() {
            if !fr.contains_key(*id) {
                errs.push(format!(
                    "n[{family}] {id}: actual case not in frozen map (a new obligation must be registered)"
                ));
            }
        }
    }

    if frozen.vm_cache.len() != actual.vm_cache.len()
        || frozen.vm_cache.keys().ne(actual.vm_cache.keys())
    {
        let only_f: Vec<_> = frozen
            .vm_cache
            .keys()
            .filter(|k| !actual.vm_cache.contains_key(*k))
            .collect();
        let only_a: Vec<_> = actual
            .vm_cache
            .keys()
            .filter(|k| !frozen.vm_cache.contains_key(*k))
            .collect();
        errs.push(format!(
            "vm_cache keys differ (frozen {}, actual {}; missing-from-actual {:?}, not-frozen {:?})",
            frozen.vm_cache.len(),
            actual.vm_cache.len(),
            only_f,
            only_a
        ));
    }
    for (k, fe) in &frozen.vm_cache {
        if let Some(ae) = actual.vm_cache.get(k) {
            if fe != ae {
                errs.push(format!("vm_cache {k}: frozen {fe:?} != actual {ae:?}"));
            }
        }
    }

    if frozen.cookbook_bindings != actual.cookbook_bindings {
        errs.push(format!(
            "cookbook bindings differ (frozen {}, actual {}); per-binding ignore/cfg_attr or case mapping changed",
            frozen.cookbook_bindings.len(),
            actual.cookbook_bindings.len()
        ));
    }
    if frozen.cookbook_multi_source_dirs != actual.cookbook_multi_source_dirs {
        errs.push(format!(
            "cookbook multi-source dirs {:?} != actual {:?}",
            frozen.cookbook_multi_source_dirs, actual.cookbook_multi_source_dirs
        ));
    }
    if frozen.s11.len() != actual.s11.len() || frozen.s11 != actual.s11 {
        errs.push(format!(
            "scheduler S11 set differs (frozen {}, actual {})",
            frozen.s11.len(),
            actual.s11.len()
        ));
    }
    if frozen.p_projects.sources != actual.p_projects.sources {
        errs.push("P project sources differ (bytes or set)".into());
    }
    if frozen.p_projects.assertions != actual.p_projects.assertions {
        errs.push(format!(
            "P 19 assertions differ (frozen {}, actual {})",
            frozen.p_projects.assertions.len(),
            actual.p_projects.assertions.len()
        ));
    }
    if frozen.p_projects.old_tests != actual.p_projects.old_tests {
        errs.push("P legacy test fns differ".into());
    }
    if frozen.p_compile.ledger_sha256 != actual.p_compile.ledger_sha256 {
        errs.push("compile-gate known-broken ledger bytes changed".into());
    }
    if frozen.p_compile.ledger_entries != actual.p_compile.ledger_entries {
        errs.push(format!(
            "compile-gate ledger entries {} != actual {}",
            frozen.p_compile.ledger_entries, actual.p_compile.ledger_entries
        ));
    }
    if frozen.p_compile.gate_ignored != actual.p_compile.gate_ignored {
        errs.push(format!(
            "compile gate ignored flag {} != actual {} (explicit-group move is T-06[B])",
            frozen.p_compile.gate_ignored, actual.p_compile.gate_ignored
        ));
    }
    if frozen.p_compile.m_guards != actual.p_compile.m_guards {
        errs.push("M guard set differs".into());
    }
    if frozen.p_compile.m_products != actual.p_compile.m_products {
        errs.push("M golden products (old bytes) differ".into());
    }
    if frozen.c_bootstrap != actual.c_bootstrap {
        errs.push(format!(
            "C bootstrap closed set differs (frozen {}, actual {})",
            frozen.c_bootstrap.len(),
            actual.c_bootstrap.len()
        ));
    }
    if frozen.c_verify != actual.c_verify {
        errs.push(format!(
            "C 33 verifications differ (frozen {}, actual {})",
            frozen.c_verify.len(),
            actual.c_verify.len()
        ));
    }
    if frozen.e78 != actual.e78 {
        errs.push(format!(
            "E78 doc boundary differs (frozen {}, actual {})",
            frozen.e78.len(),
            actual.e78.len()
        ));
    }
    errs
}

fn fixture_path() -> PathBuf {
    repo().join("test/governance/plan1066_a_obligations.json")
}

fn load_fixture_from(path: &Path) -> Result<Frozen, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// A-EVIDENCE receipts (A-EVIDENCE / NC-A4)
// ---------------------------------------------------------------------------

pub(crate) const RECEIPT_SCHEMA: &str = "plan1066-a-receipt/1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ARReceipt {
    pub schema: String,
    pub slice: String,
    /// Engine identity for the recorded actuals (recorded, never implied).
    pub engine: String,
    pub engine_class: String,
    pub tools: Vec<String>,
    pub children: Vec<ChildRun>,
    pub checks: Vec<CheckEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChildRun {
    pub name: String,
    pub exit: i32,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CheckEvidence {
    pub key: String,
    pub family: String,
    /// "doc" (the migration doc is the authority) or "legacy" (a frozen
    /// old-entry identity run — allowed only as identity evidence, never
    /// as doc-face).
    pub face: String,
    pub source_path: String,
    pub source_sha256: String,
    pub oracle_channels: Vec<String>,
    pub observed_channels: Vec<String>,
    pub required_tool: Option<String>,
    pub status: String, // PASS | FAIL | NOT_RUN
}

/// Independent receipt verification (never trusts a self-reported PASS):
/// re-hashes the recorded sources, enforces doc-face rows to actually read
/// migration docs, and rejects parent-pass masking, missing tools, NOT_RUN
/// passes and unobserved oracle channels.
pub(crate) fn verify_receipt(r: &ARReceipt) -> Result<(), Vec<String>> {
    let mut errs = Vec::new();
    if r.schema != RECEIPT_SCHEMA {
        errs.push(format!(
            "receipt schema {:?} != {:?}",
            r.schema, RECEIPT_SCHEMA
        ));
    }
    if r.slice != "A" {
        errs.push(format!("receipt slice {:?} != \"A\"", r.slice));
    }
    if r.engine.trim().is_empty() || r.engine_class.trim().is_empty() {
        errs.push("engine identity missing (every actual must record its engine)".into());
    }
    // F-1066A-R1: an empty selection signs nothing — vacuous receipts must
    // not verify (plan §5 A-EVIDENCE lists 空选择 among what cannot sign A).
    if r.checks.is_empty() {
        errs.push("empty selection cannot sign A (no check evidence)".into());
    }
    let failed_children: Vec<&ChildRun> = r.children.iter().filter(|c| !c.ok).collect();
    for c in &r.checks {
        // Identity: the recorded source must exist and hash to what the
        // receipt claims, right now.
        match sha256_file(&root_join(&c.source_path)) {
            Ok(h) if h == c.source_sha256 => {}
            Ok(h) => errs.push(format!(
                "check {} [{}]: recorded source hash {} != current {} ({})",
                c.key, c.face, c.source_sha256, h, c.source_path
            )),
            Err(e) => errs.push(format!(
                "check {} [{}]: source unreadable: {e}",
                c.key, c.face
            )),
        }
        // Doc-face rows read migration docs; an old sidecar path posing as
        // doc-face is a lie about which input produced the actual.
        if c.face == "doc" && !c.source_path.ends_with(".case.ad") {
            errs.push(format!(
                "check {} [doc]: source {} is not a migration doc (old-file result posing as doc-only)",
                c.key, c.source_path
            ));
        }
        if c.status == "NOT_RUN" {
            errs.push(format!("check {}: NOT_RUN cannot be signed", c.key));
            continue;
        }
        if c.status == "PASS" {
            let missing: Vec<&String> = c
                .oracle_channels
                .iter()
                .filter(|ch| !c.observed_channels.contains(ch))
                .collect();
            if !missing.is_empty() {
                errs.push(format!(
                    "check {}: PASS without observed channels {:?} (single-run dual-channel required)",
                    c.key, missing
                ));
            }
            if let Some(t) = &c.required_tool {
                if !r.tools.iter().any(|x| x == t) {
                    errs.push(format!(
                        "check {}: required tool {:?} missing from receipt tools",
                        c.key, t
                    ));
                }
            }
            if !failed_children.is_empty() {
                errs.push(format!(
                    "check {}: PASS while children failed ({:?}) — parent exit0 cannot mask child failure",
                    c.key,
                    failed_children.iter().map(|c| c.name.clone()).collect::<Vec<_>>()
                ));
            }
        }
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}

/// Build the genuine A-slice receipt for the two restored dual-channel
/// cases: one doc-face run per case (single execution, both channels
/// observed) plus a legacy-face identity run over the unchanged old oracle
/// files.
pub(crate) fn build_dual_channel_receipt() -> Result<ARReceipt, String> {
    let mut checks = Vec::new();
    let mut children = Vec::new();
    for (key, dir_name) in [
        ("vm/04_control_flow/004_if_with_bool", "004_if_with_bool"),
        ("vm/04_control_flow/006_is_stmt", "006_is_stmt"),
    ] {
        let dir = test_dir("vm").join("04_control_flow").join(dir_name);
        let stem = stem_of(dir_name);
        let doc_rel = format!("crates/auto-lang/test/vm/04_control_flow/{dir_name}/{stem}.case.ad");
        let (oracle, stdout, result) =
            run_vm_doc_once(&dir, false).map_err(|e| format!("{key}: {e}"))?;
        let mismatch = diff_vm_channels(&oracle, &stdout, &result);
        if !mismatch.is_empty() {
            return Err(format!("{key}: doc-face run mismatched: {mismatch:?}"));
        }
        let mut channels = Vec::new();
        if oracle.stdout.is_some() {
            channels.push("stdout".to_string());
        }
        if oracle.result.is_some() {
            channels.push("result".to_string());
        }
        let doc_sha = sha256_file(&root_join(&doc_rel))?;
        checks.push(CheckEvidence {
            key: key.to_string(),
            family: "vm".into(),
            face: "doc".into(),
            source_path: doc_rel,
            source_sha256: doc_sha,
            oracle_channels: channels.clone(),
            observed_channels: channels,
            required_tool: None,
            status: "PASS".into(),
        });

        // Legacy identity binding: the SAME case through the old entry (old
        // source + old oracle files, unchanged). Green here proves the
        // restored doc channels equal the original oracle bytes — it is NOT
        // doc-face evidence and never lands in `face: "doc"`.
        // FileTestCase.name is `category/case` (no family prefix — the
        // vm identity keeps the original convention; only the a2r/cookbook
        // discoverers carry a suite prefix).
        let id = key.trim_start_matches("vm/");
        let case = crate::test_runner::discover_vm_tests(&test_dir("vm"))
            .into_iter()
            .find(|c| c.name == id)
            .ok_or(format!("{key}: not discoverable by the legacy discoverer"))?;
        let report = auto_lang_core::run_vm_file_test(&case);
        let ok = matches!(report.outcome, crate::test_runner::TestOutcome::Passed);
        children.push(ChildRun {
            name: format!("legacy-entry:{key}"),
            exit: if ok { 0 } else { 1 },
            ok,
        });
        if !ok {
            return Err(format!(
                "{key}: legacy identity run failed: {:?}",
                report.outcome
            ));
        }
    }
    Ok(ARReceipt {
        schema: RECEIPT_SCHEMA.to_string(),
        slice: "A".into(),
        engine: "auto-lang entry::run_with_capture (legacy AutoVM interpreter — the original engine these golden channels were captured on)".into(),
        engine_class: "legacy-autovm".into(),
        tools: vec!["cargo-nextest".into()],
        children,
        checks,
    })
}

// ---------------------------------------------------------------------------
// doc surgery helpers for the mirror probes
// ---------------------------------------------------------------------------

/// Remove one `### <title>` section (from its heading to the next same-or-
/// higher heading or EOF), preserving everything else byte-for-byte.
fn remove_h3_section(text: &str, title: &str) -> String {
    let marker = format!("### {title}");
    let Some(start) = text.find(&marker) else {
        return text.to_string();
    };
    let after = &text[start..];
    let next = after[marker.len()..]
        .find("\n### ")
        .map(|i| start + marker.len() + i + 1) // keep the newline before next heading
        .unwrap_or(text.len());
    format!("{}{}", &text[..start], &text[next..])
}

/// Remove the `###` section CONTAINING `needle` (e.g. one specific
/// `check-id:` line among several sections sharing a heading), from its
/// heading line to the next heading or EOF; the preceding newline before the
/// next heading is kept so the document still parses.
fn remove_section_containing(text: &str, needle: &str) -> Option<String> {
    let n = text.find(needle)?;
    let head_start = text[..n].rfind("\n### ").map(|i| i + 1)?;
    let after = &text[head_start..];
    // next heading start (skip the heading we are inside)
    let end = after[3..]
        .find("\n### ")
        .map(|i| head_start + 3 + i + 1) // position of the '\n' before next heading
        .unwrap_or(text.len());
    Some(format!("{}{}", &text[..head_start], &text[end..]))
}

/// Corrupt the oracle text of the first `### <title>` section by appending
/// a sentinel to its fenced payload (deterministic mismatch).
fn corrupt_section_fence(text: &str, title: &str, sentinel: &str) -> Option<String> {
    let marker = format!("### {title}");
    let start = text.find(&marker)?;
    let rest = &text[start..];
    let fence = rest.find("```")? + start; // opening fence of the section payload
    let payload_start = text[fence..].find('\n')? + fence + 1;
    let mut out = String::new();
    out.push_str(&text[..payload_start]);
    out.push_str(sentinel);
    out.push_str(&text[payload_start..]);
    Some(out)
}

/// Run `f` with env vars set; always restore (even on panic), then resume.
fn with_env(vars: &[(&str, Option<&str>)], f: impl FnOnce()) {
    let prev: Vec<(&str, Option<String>)> = vars
        .iter()
        .map(|(k, _)| (*k, std::env::var(k).ok()))
        .collect();
    for (k, v) in vars {
        match v {
            Some(val) => std::env::set_var(k, val),
            None => std::env::remove_var(k),
        }
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    for (k, v) in &prev {
        match v {
            Some(val) => std::env::set_var(k, val),
            None => std::env::remove_var(k),
        }
    }
    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

/// A-DEN: the frozen obligation map must match the same-source derived
/// actual, per key. This is the standing green control for NC-A1.
#[test]
fn plan1066_a_denominator_integrity() {
    let start = std::time::Instant::now();
    let frozen = load_fixture_from(&fixture_path()).expect("frozen obligations fixture");
    let actual = derive_actual().expect("same-source derivation");
    let errs = compare_frozen(&frozen, &actual);
    assert!(
        errs.is_empty(),
        "PLAN-1066/A denominator drift ({}):\n{}",
        errs.len(),
        errs.join("\n")
    );
    // Frozen denominator floors (anti-shrink; the per-key diff above is the
    // real gate, counts are the floor). PLAN-1066 T-03[B] raised vm 496 → 506:
    // document-authority discovery selects ten VM entries the old sidecar
    // selection silently skipped (08_strings/017_sub_semantics had its oracle
    // named by the full directory name; 10_types/033_array_len_user_type,
    // 17_cffi/{001,002,003}, 18_ffi/{047,055} and 99_bootstrap/{066,067,068}
    // carry their oracle only in the document). They were already executed by
    // the doc-reading VM cache wrapper (506 keys, unchanged); the increment
    // registers them in the discovery denominator instead of leaving a hole.
    for (family, want) in [
        ("vm", 506),
        ("a2r", 250),
        ("a2c", 118),
        ("a2ts", 85),
        ("cookbook", 101),
    ] {
        assert_eq!(
            frozen.n_rows.iter().filter(|r| r.family == family).count(),
            want,
            "{family} rows"
        );
    }
    assert_eq!(
        frozen.cookbook_multi_source_dirs.len(),
        6,
        "cookbook multi-source dirs"
    );
    assert!(
        frozen.cookbook_multi_source_dirs.iter().all(|d| d.selected_doc),
        "every multi-source dir's selected candidate has its doc (no-doc candidates are never promoted)"
    );
    assert_eq!(frozen.s11.len(), 11, "S11");
    assert_eq!(frozen.p_projects.assertions.len(), 19, "P assertions");
    assert_eq!(frozen.c_bootstrap.len(), 30, "C bootstrap closed set");
    assert_eq!(frozen.c_verify.len(), 33, "C verifications");
    assert_eq!(frozen.vm_cache.len(), 506, "VM cache keys");
    assert_eq!(
        frozen.vm_cache.values().filter(|e| e.is_script).count(),
        8,
        "VM cache .as scripts"
    );
    let h_empty = frozen
        .n_rows
        .iter()
        .filter(|r| r.family == "a2c" && r.roles.contains(&"h-empty".to_string()))
        .count();
    assert_eq!(h_empty, 9, "nine zero-byte a2c header roles stay explicit");
    eprintln!(
        "plan1066_a_denominator_integrity: {} ms",
        start.elapsed().as_millis()
    );
}

/// NC-A1: tampering with the frozen obligation map must be rejected per key
/// (deleting the N last item, collapsing a cross-family duplicate id,
/// turning a zero-byte H role into absence, dropping one P assertion, one
/// S11 row, or one C verification) — never by count alone, and the honest
/// map stays green.
#[test]
fn plan1066_nc_a1_denominator_tamper_rejected() {
    let start = std::time::Instant::now();
    let frozen = load_fixture_from(&fixture_path()).expect("fixture");
    let actual = derive_actual().expect("actual");

    assert!(
        compare_frozen(&frozen, &actual).is_empty(),
        "honest map must be green"
    );

    let mut m = frozen.clone();
    let last_pos = m
        .n_rows
        .iter()
        .rposition(|r| r.id == "versioning/006_semver_prerelease" && r.family == "cookbook")
        .expect("last N row");
    m.n_rows.remove(last_pos);
    let errs = compare_frozen(&m, &actual);
    assert!(
        errs.iter()
            .any(|e| e.contains("versioning/006_semver_prerelease")),
        "deleted N last row must be named per key, got: {errs:?}"
    );

    let mut m = frozen.clone();
    let pos = m
        .n_rows
        .iter()
        .position(|r| r.family == "a2c" && r.id == "01_basics/001_hello")
        .expect("cross-family duplicate id");
    m.n_rows.remove(pos);
    let errs = compare_frozen(&m, &actual);
    assert!(
        errs.iter()
            .any(|e| e.contains("n[a2c] 01_basics/001_hello")),
        "cross-family merge must be named per key, got: {errs:?}"
    );

    let mut m = frozen.clone();
    let mut flipped = 0;
    for r in m.n_rows.iter_mut() {
        if r.family == "a2c" && r.roles.iter().any(|x| x == "h-empty") {
            r.roles.retain(|x| x != "h-empty");
            flipped += 1;
        }
    }
    assert_eq!(flipped, 9, "exactly nine h-empty roles exist");
    let errs = compare_frozen(&m, &actual);
    assert!(
        errs.iter().any(|e| e.contains("oracle roles")),
        "empty-H turned absent must be rejected, got: {errs:?}"
    );

    let mut m = frozen.clone();
    m.p_projects.assertions.pop().expect("19 assertions");
    let errs = compare_frozen(&m, &actual);
    assert!(
        errs.iter().any(|e| e.contains("assertions differ")),
        "dropped P assertion must be rejected, got: {errs:?}"
    );

    // (f) tampered family segment metadata (A/B/actual-source mapping).
    let mut m = frozen.clone();
    m.segments.get_mut("vm").unwrap().stage = "A".into();
    let errs = compare_frozen(&m, &actual);
    assert!(
        errs.iter().any(|e| e.contains("segment[vm] meta differs")),
        "edited segment meta must be rejected, got: {errs:?}"
    );

    let mut m = frozen.clone();
    m.s11.pop();
    m.c_verify.pop();
    let errs = compare_frozen(&m, &actual);
    assert!(
        errs.iter().any(|e| e.contains("S11 set differs")),
        "dropped S11 row must be rejected, got: {errs:?}"
    );
    assert!(
        errs.iter().any(|e| e.contains("verifications differ")),
        "dropped C verification must be rejected, got: {errs:?}"
    );
    eprintln!("plan1066_nc_a1: {} ms", start.elapsed().as_millis());
}

/// NC-A2: N0028/N0030 keep stdout correct while only the result oracle is
/// mutated (sucC/11), or the vm-result section is deleted — the real doc
/// consumer must go red on the result axis (or lose the channel against the
/// frozen obligation), and the original succ/10 (no trailing LF) values
/// must pass in ONE run.
#[test]
fn plan1066_nc_a2_dual_channel_oracle() {
    let start = std::time::Instant::now();
    let frozen = load_fixture_from(&fixture_path()).expect("fixture");
    for (key, dir_name, good, bad) in [
        (
            "vm/04_control_flow/004_if_with_bool",
            "004_if_with_bool",
            "succ",
            "sucC",
        ),
        ("vm/04_control_flow/006_is_stmt", "006_is_stmt", "10", "11"),
    ] {
        let real_dir = test_dir("vm").join("04_control_flow").join(dir_name);
        let stem = stem_of(dir_name);
        let doc_path = real_dir.join(format!("{stem}.case.ad"));
        let doc_text = std::fs::read_to_string(&doc_path).unwrap();

        // Green control: the real doc, single run, both channels observed,
        // original bytes (no trailing LF on the result channel).
        let (oracle, stdout, result) = run_vm_doc_once(&real_dir, false).unwrap();
        assert!(
            oracle.stdout.is_some() && oracle.result.is_some(),
            "{key}: doc must carry both channels"
        );
        assert!(
            diff_vm_channels(&oracle, &stdout, &result).is_empty(),
            "{key}: original must pass"
        );
        assert_eq!(
            oracle.result.as_deref(),
            Some(good),
            "{key}: frozen result bytes"
        );
        assert!(!good.ends_with('\n'), "{key}: no trailing LF");

        // Red 1: only the result oracle mutated (sucC / 11), stdout untouched.
        let mutated = doc_text.replace(&format!("\n{good}\n"), &format!("\n{bad}\n"));
        assert_ne!(mutated, doc_text, "{key}: mutation applied");
        let mirror = tempfile::tempdir().unwrap();
        let mirror_case = mirror.path().join(dir_name);
        std::fs::create_dir_all(&mirror_case).unwrap();
        std::fs::write(mirror_case.join(format!("{stem}.case.ad")), mutated).unwrap();
        let (oracle, stdout, result) = run_vm_doc_once(&mirror_case, false).unwrap();
        let mismatches = diff_vm_channels(&oracle, &stdout, &result);
        assert_eq!(
            mismatches.len(),
            1,
            "{key}: exactly one axis fails, got {mismatches:?}"
        );
        assert_eq!(
            mismatches[0].axis,
            VmChannelAxis::Result,
            "{key}: result axis"
        );
        assert_eq!(
            mismatches[0].expected, bad,
            "{key}: mutated oracle observed"
        );
        assert_eq!(mismatches[0].actual, good, "{key}: engine output unchanged");

        // Red 2: the section carrying check-id vm-result is deleted (the
        // stdout section keeps its heading) — the doc loses the channel and
        // no longer matches the frozen obligation (channel conservation).
        let without_result = remove_section_containing(&doc_text, "check-id: \"vm-result\"")
            .unwrap_or_else(|| panic!("{key}: vm-result section located"));
        assert!(
            !without_result.contains("check-id: \"vm-result\""),
            "{key}: vm-result section removed"
        );
        assert!(
            without_result.contains("check-id: \"vm-stdout\""),
            "{key}: vm-stdout section kept"
        );
        let mirror2 = tempfile::tempdir().unwrap();
        let mirror2_case = mirror2.path().join(dir_name);
        std::fs::create_dir_all(&mirror2_case).unwrap();
        std::fs::write(
            mirror2_case.join(format!("{stem}.case.ad")),
            &without_result,
        )
        .unwrap();
        let (oracle2, _s, _r) = run_vm_doc_once(&mirror2_case, false).unwrap();
        assert!(
            oracle2.result.is_none(),
            "{key}: deletion surfaces as channel loss"
        );
        let id = key.trim_start_matches("vm/");
        let frozen_row = frozen
            .n_rows
            .iter()
            .find(|r| r.family == "vm" && r.id == id)
            .unwrap_or_else(|| panic!("{key}: frozen row"));
        assert!(
            frozen_row.doc_checks.contains(&"vm-result".to_string()),
            "{key}: frozen map pins the result channel (conservation)"
        );
        let mut derived: Vec<String> = Vec::new();
        if oracle2.stdout.is_some() {
            derived.push("vm-stdout".into());
        }
        if oracle2.result.is_some() {
            derived.push("vm-result".into());
        }
        if oracle2.error {
            derived.push("vm-error".into());
        }
        derived.sort();
        assert_ne!(
            frozen_row.doc_checks, derived,
            "{key}: frozen doc checks differ from the channel-deleted doc"
        );
    }
    eprintln!("plan1066_nc_a2: {} ms", start.elapsed().as_millis());
}

/// NC-A3: the normal paths A executes stay write-free in the author tree —
/// an error oracle may not create `.wrong.out`, and GENERATE_EXPECTED may
/// not create `expected.out`; candidates/diagnostics only land in the
/// explicit scope, and the wrong oracle stays red.
#[test]
fn plan1066_nc_a3_no_author_tree_writes() {
    let start = std::time::Instant::now();
    let scope = tempfile::tempdir().unwrap();

    let conf_root = test_dir("a2r/conformance");
    let cookbook_root = test_dir("cookbook");
    let vm_root = test_dir("vm");
    let conf_before = tree_hash(&conf_root);
    let cookbook_before = tree_hash(&cookbook_root);
    let vm_before = tree_hash(&vm_root);

    // (1) conformance wrong-oracle path via a scope MIRROR: same mechanism,
    // corrupted oracle fence ⇒ red; `.wrong.out` may only appear under the
    // scope, never next to any corpus.
    let mirror = tempfile::tempdir().unwrap();
    let doc_text = std::fs::read_to_string(conf_root.join("001_int_add/input.case.ad")).unwrap();
    let mutated = corrupt_section_fence(&doc_text, "VM result", "Z")
        .expect("001_int_add doc has a VM result fence");
    assert_ne!(mutated, doc_text, "oracle corruption applied");
    std::fs::create_dir_all(mirror.path().join("999_nc_a3")).unwrap();
    std::fs::write(mirror.path().join("999_nc_a3/input.case.ad"), mutated).unwrap();
    let mut panicked = false;
    with_env(
        &[
            ("GENERATE_EXPECTED", Some("1")),
            ("AUTO_TEST_DIAG_SCOPE", Some(scope.path().to_str().unwrap())),
        ],
        || {
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                crate::tests::conformance_tests::run_conformance_case(mirror.path(), "999_nc_a3")
            }));
            panicked = r.is_err();
        },
    );
    assert!(panicked, "wrong conformance oracle must stay red");
    assert!(
        !mirror.path().join("999_nc_a3/.wrong.out").exists()
            && !mirror.path().join("999_nc_a3/expected_output.txt").exists(),
        "no author-style writes beside the (mirror) case"
    );

    // (2) cookbook GENERATE_EXPECTED on a doc without a stdout oracle: the
    // candidate goes to the scope, never to an author expected.out.
    let cb_mirror = tempfile::tempdir().unwrap();
    let case_dir = cb_mirror.path().join("algorithms/001_sort_int");
    std::fs::create_dir_all(&case_dir).unwrap();
    let src_doc =
        std::fs::read_to_string(cookbook_root.join("algorithms/001_sort_int/sort_int.case.ad"))
            .unwrap();
    let no_oracle = remove_h3_section(&src_doc, "VM result");
    assert_ne!(no_oracle, src_doc, "cookbook oracle section removed");
    std::fs::write(case_dir.join("sort_int.case.ad"), no_oracle).unwrap();
    std::fs::write(
        case_dir.join("sort_int.at"),
        b"// selection anchor - payload comes from the doc",
    )
    .unwrap();
    with_env(
        &[
            ("GENERATE_EXPECTED", Some("1")),
            ("AUTO_TEST_DIAG_SCOPE", Some(scope.path().to_str().unwrap())),
        ],
        || {
            crate::tests::cookbook_vm_tests::run_cookbook_case(
                cb_mirror.path(),
                "algorithms",
                "001_sort_int",
                false, // mirror run: no governance trace for synthetic input
            )
            .expect("smoke run (no oracle) succeeds");
        },
    );
    assert!(
        !case_dir.join("expected.out").exists() && !case_dir.join("wrong.out").exists(),
        "GENERATE_EXPECTED must not write into the case dir"
    );
    let scoped = scope
        .path()
        .join("plan-1066-diag/test/cookbook/algorithms/001_sort_int/expected.out");
    assert!(
        scoped.is_file(),
        "the candidate landed in the explicit scope"
    );
    let scoped_conf = scope
        .path()
        .join("plan-1066-diag/test/a2r/conformance/999_nc_a3/.wrong.out");
    assert!(
        scoped_conf.is_file(),
        "the conformance diagnostic landed in the scope"
    );

    // Author trees byte-identical throughout.
    assert_eq!(
        conf_before,
        tree_hash(&conf_root),
        "conformance author tree changed"
    );
    assert_eq!(
        cookbook_before,
        tree_hash(&cookbook_root),
        "cookbook author tree changed"
    );
    assert_eq!(vm_before, tree_hash(&vm_root), "vm author tree changed");
    eprintln!("plan1066_nc_a3: {} ms", start.elapsed().as_millis());
}

fn tree_hash(root: &Path) -> String {
    let mut acc = String::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let mut entries: Vec<_> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let bytes = std::fs::read(&p).unwrap_or_default();
                acc.push_str(&format!(
                    "{}:{}\n",
                    p.strip_prefix(root)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .replace('\\', "/"),
                    hex(&sha256(&bytes))
                ));
            }
        }
    }
    hex(&sha256(acc.as_bytes()))
}

/// NC-A4: receipts are verified against the actual tree — stale hashes,
/// old-sidecar results posing as doc-face, parent-pass with failed children,
/// missing tools, NOT_RUN passes and unobserved channels are all rejected;
/// the genuine receipt built from a real run verifies.
#[test]
fn plan1066_nc_a4_receipt_verification() {
    let start = std::time::Instant::now();
    let genuine = build_dual_channel_receipt().expect("genuine receipt");
    verify_receipt(&genuine).unwrap_or_else(|errs| panic!("genuine receipt rejected: {errs:?}"));

    let mut r = genuine.clone();
    r.checks[0].source_sha256 = format!("{}0", &r.checks[0].source_sha256[..63]);
    let errs = verify_receipt(&r).expect_err("stale hash");
    assert!(
        errs.iter().any(|e| e.contains("recorded source hash")),
        "{errs:?}"
    );

    let mut r = genuine.clone();
    r.checks[0].source_path = r.checks[0].source_path.replace(".case.ad", ".at");
    let errs = verify_receipt(&r).expect_err("old-file pose");
    assert!(
        errs.iter().any(|e| e.contains("posing as doc-only")),
        "{errs:?}"
    );

    let mut r = genuine.clone();
    r.children.push(ChildRun {
        name: "compile-probe".into(),
        exit: 1,
        ok: false,
    });
    let errs = verify_receipt(&r).expect_err("failed child");
    assert!(
        errs.iter().any(|e| e.contains("mask child failure")),
        "{errs:?}"
    );

    let mut r = genuine.clone();
    r.checks[0].required_tool = Some("nonexistent-tool".into());
    let errs = verify_receipt(&r).expect_err("missing tool");
    assert!(errs.iter().any(|e| e.contains("required tool")), "{errs:?}");

    let mut r = genuine.clone();
    r.checks[0].status = "NOT_RUN".into();
    let errs = verify_receipt(&r).expect_err("NOT_RUN");
    assert!(errs.iter().any(|e| e.contains("NOT_RUN")), "{errs:?}");

    let mut r = genuine.clone();
    r.checks[0].observed_channels.retain(|c| c != "result");
    let errs = verify_receipt(&r).expect_err("missing channel");
    assert!(
        errs.iter().any(|e| e.contains("observed channels")),
        "{errs:?}"
    );

    // (g) empty selection signs nothing (F-1066A-R1).
    let mut r = genuine.clone();
    r.checks.clear();
    let errs = verify_receipt(&r).expect_err("empty selection");
    assert!(
        errs.iter().any(|e| e.contains("empty selection")),
        "{errs:?}"
    );

    eprintln!("plan1066_nc_a4: {} ms", start.elapsed().as_millis());
}

/// Explicit PLAN-1066/A tools (never part of the daily run):
///
/// * `AUTO_TEST_1066_FREEZE_OUT=<path>` — regenerate the frozen obligation
///   fixture from the current same-source derivation (a regenerated map is a
///   NEW freeze and is reviewed like any contract change).
/// * `AUTO_TEST_1066_RECEIPT_OUT=<path>` — emit the genuine dual-channel +
///   legacy-identity receipt (verified before writing).
#[test]
#[ignore = "explicit PLAN-1066/A tool: requires AUTO_TEST_1066_FREEZE_OUT or AUTO_TEST_1066_RECEIPT_OUT"]
fn plan1066_a_tools() {
    if let Ok(out) = std::env::var("AUTO_TEST_1066_FREEZE_OUT") {
        let actual = derive_actual().expect("derivation");
        let json = serde_json::to_vec_pretty(&actual).unwrap();
        std::fs::write(&out, json).unwrap_or_else(|e| panic!("{out}: {e}"));
        eprintln!(
            "frozen fixture written: {out} ({} bytes)",
            std::fs::metadata(&out).unwrap().len()
        );
    }
    if let Ok(out) = std::env::var("AUTO_TEST_1066_RECEIPT_OUT") {
        let receipt = build_dual_channel_receipt().expect("receipt");
        verify_receipt(&receipt).expect("receipt verifies before writing");
        std::fs::write(&out, serde_json::to_vec_pretty(&receipt).unwrap())
            .unwrap_or_else(|e| panic!("{out}: {e}"));
        eprintln!("receipt written: {out}");
    }
}
