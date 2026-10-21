//! PLAN-780 T-03 — explicit old-vs-new probe adapter.
//!
//! This module is a **pilot probe only**: no production pac read path
//! consumes it (all defaults stay on the legacy VM pipeline). It exists to
//! compare, on real corpus + fixture inputs:
//!
//! - the **old arm** (oracle): `AutoConfig::from_code` (config VM) →
//!   `Pac::new` — the production model extraction in force today;
//! - the **new arm**: `auto_project_model::parse_document` — the light
//!   static model (PLAN-780 T-02).
//!
//! Frozen-field tuples are compared per file; files the model classifies
//! non-static (`unsupported` non-empty) are counted and never claimed
//! equal (AC-02: dynamic/unknown never fake green).

use crate::pac::{Pac, Scene};
use crate::TargetKind;
use auto_lang_core::config::{AutoConfig, BackendConfig};
use auto_project_model::{
    parse_document as model_parse, ErrorStage, ProjectDescriptor, RenderSpec, SceneKind,
    SourceInfo, WindowSpec,
};
use auto_val::{Obj, Value};

/// Frozen comparison tuple — every field mirrors `Pac::new`'s normalization
/// (see the T-01 report §4 for the per-field alignment table).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FrozenNormalized {
    pub name: String,
    pub version: String,
    pub lang_declared: Option<String>,
    pub backend_str: String,
    /// `Some(RenderSpec-shape)` mirrors `Pac.backend_config`; `None` = the
    /// oracle's `from_value → None` (undeclared or declared-but-unknown).
    pub backend_shape: Option<RenderSpec>,
    pub api: Option<String>,
    pub scene: SceneKind,
    pub members_declared: Option<Vec<String>>,
    pub defines: Option<Vec<String>>,
    pub title: Option<String>,
    pub title_zh: Option<String>,
    pub icon: Option<String>,
    pub category: Option<String>,
    pub theme: Option<String>,
    pub accent: Option<String>,
    pub desktop_render: Option<String>,
    pub ui_config: Option<String>,
    pub media_root: Option<String>,
    pub photo_root: Option<String>,
    pub window: Option<WindowSpec>,
    pub front_port: Option<u16>,
    pub back_port: Option<u16>,
    pub external_backend: Option<String>,
    pub shadcn: bool,
    pub default_classes: bool,
    /// (name, path) per dep declaration, in declaration order.
    pub deps: Vec<(String, Option<String>)>,
}

/// Per-file comparison outcome.
pub struct ComparisonReport {
    pub path: String,
    /// `Some(err)` when the file itself could not be read (missing /
    /// invalid UTF-8 — R1 F-04). Required-input failures are observable
    /// gate failures, never "both arms parsed an empty config".
    pub read_error: Option<String>,
    /// `Ok(tuple)` = old arm parsed; `Err(msg)` = the legacy VM itself
    /// failed on this input (recorded — the old capability is the fallback
    /// for everything the static model rejects).
    pub old: Result<FrozenNormalized, String>,
    /// `Ok((static_equivalent, tuple))` = parsed with the descriptor's
    /// static-equivalence flag; `Err(msg)` = new-arm syntax error (recorded).
    pub new: Result<(bool, FrozenNormalized), String>,
    /// Structural error stage of the new arm when `new` is `Err` — bound to
    /// the expected-bad allowlist entries (R2 F-02 diagnosis surface).
    pub new_error_stage: Option<ErrorStage>,
}

impl ComparisonReport {
    /// True only when both arms parsed AND the model claims static
    /// equivalence AND every frozen field matches.
    pub fn equal_claim(&self) -> Option<bool> {
        let old = self.old.as_ref().ok()?;
        let (static_eq, new) = self.new.as_ref().ok()?;
        Some(*static_eq && *old == *new)
    }
}

fn probe_args() -> Obj {
    // Mirror the production probe shape (config.rs external_backend_dir /
    // Automan::parse_pac): port feeds config globals, unused by static files.
    let mut probe = Obj::new();
    probe.set("port", Value::Str(auto_val::AutoStr::from("win32")));
    probe
}

fn scene_kind(scene: &Scene) -> SceneKind {
    match scene {
        Scene::Workspace => SceneKind::Workspace,
        Scene::Ui => SceneKind::Ui,
        Scene::C => SceneKind::C,
        Scene::Default => SceneKind::Default,
    }
}

fn backend_shape(cfg: &Option<BackendConfig>) -> Option<RenderSpec> {
    let bc = cfg.as_ref()?;
    Some(match bc {
        BackendConfig::Single(t) => RenderSpec::Single(t.as_str().to_string()),
        BackendConfig::Multi(v) => {
            RenderSpec::Multi(v.iter().map(|t| t.as_str().to_string()).collect())
        }
        BackendConfig::Split { front, back } => RenderSpec::Split {
            front: front.iter().map(|t| t.as_str().to_string()).collect(),
            back: back.as_str().to_string(),
        },
    })
}

fn model_backend_shape(spec: &RenderSpec) -> Option<RenderSpec> {
    match spec {
        RenderSpec::Absent | RenderSpec::Unknown(_) => None,
        other => Some(other.clone()),
    }
}

fn opt_str(v: &Option<auto_val::AutoStr>) -> Option<String> {
    v.as_ref().map(|s| s.to_string())
}

/// Old arm: config VM → Pac::new → frozen tuple.
pub fn old_normalized_from_code(code: &str) -> Result<FrozenNormalized, String> {
    let mut cfg = AutoConfig::from_code(code, &probe_args())
        .map_err(|e| format!("old arm AutoConfig::from_code failed: {e}"))?;
    // cfg-level facts must be read before Pac::new consumes the config.
    // (Pac::new's root-wrapper hoist for `.am` history files is transparent
    // for managed pac.at, which never uses the wrapper — T-01 corpus fact.)
    use crate::node_ext::NodeExt;
    let lang_declared = match cfg.root.get_prop("lang") {
        Value::Str(s) => Some(s.to_string()),
        _ => None,
    };
    let members_declared = cfg.root.has_prop("members").then(|| {
        cfg.root
            .get_str_vec_or("members")
            .iter()
            .map(|s| s.to_string())
            .collect()
    });
    let defines_declared = cfg.root.has_prop("defines").then(|| {
        cfg.root
            .get_str_vec_or("defines")
            .iter()
            .map(|s| s.to_string())
            .collect()
    });
    let pac = Pac::new(cfg);
    Ok(FrozenNormalized {
        name: pac.name.to_string(),
        version: pac.version.to_string(),
        lang_declared,
        backend_str: pac.backend.to_string(),
        backend_shape: backend_shape(&pac.backend_config),
        api: opt_str(&pac.api),
        scene: scene_kind(&pac.scene),
        members_declared,
        defines: defines_declared,
        title: opt_str(&pac.title),
        title_zh: opt_str(&pac.title_zh),
        icon: opt_str(&pac.icon),
        category: opt_str(&pac.category),
        theme: opt_str(&pac.theme),
        accent: opt_str(&pac.accent),
        desktop_render: opt_str(&pac.desktop_render),
        ui_config: opt_str(&pac.ui_config),
        media_root: opt_str(&pac.media_root),
        photo_root: opt_str(&pac.photo_root),
        window: if pac.window_fit {
            Some(WindowSpec::Fit)
        } else {
            pac.window.map(|(w, h)| WindowSpec::Size(w, h))
        },
        front_port: pac.front_port,
        back_port: pac.back_port,
        external_backend: opt_str(&pac.external_backend),
        shadcn: pac.shadcn,
        default_classes: pac.default_classes,
        deps: pac
            .targets
            .iter()
            .filter(|t| t.kind == TargetKind::Dep)
            .map(|t| {
                let path = t.from.to_string();
                (t.name.to_string(), (!path.is_empty()).then_some(path))
            })
            .collect(),
    })
}

/// New arm: static model → frozen tuple.
pub fn new_normalized_from_code(
    code: &str,
    source: &SourceInfo,
) -> Result<(ProjectDescriptor, FrozenNormalized), String> {
    let d = model_parse(code, source.clone()).map_err(|e| format!("new arm parse failed: {e}"))?;
    Ok((d.clone(), new_tuple(&d)))
}

fn new_tuple(d: &ProjectDescriptor) -> FrozenNormalized {
    let deps = d
        .deps
        .iter()
        .map(|dep| (dep.name.clone(), dep.path.clone()))
        .collect();
    FrozenNormalized {
        name: d.identity.name.clone().unwrap_or_default(),
        version: d.identity.version.clone().unwrap_or_default(),
        lang_declared: d.identity.lang.clone(),
        backend_str: d.render.effective_backend.clone().unwrap_or_default(),
        backend_shape: model_backend_shape(&d.render.render),
        api: d.runtime.api.clone(),
        scene: d.runtime.scene.unwrap_or_default(),
        members_declared: d.members.clone(),
        defines: d.defines.clone(),
        title: d.display.title.clone(),
        title_zh: d.display.title_zh.clone(),
        icon: d.display.icon.clone(),
        category: d.display.category.clone(),
        theme: d.display.theme.clone(),
        accent: d.display.accent.clone(),
        desktop_render: d.display.desktop_render.clone(),
        ui_config: d.display.ui_config.clone(),
        media_root: d.runtime.media_root.clone(),
        photo_root: d.runtime.photo_root.clone(),
        window: d.runtime.window,
        front_port: d.runtime.front_port,
        back_port: d.runtime.back_port,
        external_backend: d.external_backend.clone(),
        shadcn: d.toggles.shadcn.normalized(),
        default_classes: d.toggles.default_classes.normalized(),
        deps,
    }
}

/// Compare one file's old and new arms. Read failures (missing file,
/// invalid UTF-8) surface on `read_error` and as errors on BOTH arms —
/// they must fail any corpus gate instead of comparing empty configs
/// (R1 F-04).
pub fn compare_file(path: &std::path::Path) -> ComparisonReport {
    let rel = path.to_string_lossy().to_string();
    let code = match std::fs::read_to_string(path) {
        Ok(code) => code,
        Err(e) => {
            let msg = format!("required input unreadable: {}: {e}", path.display());
            return ComparisonReport {
                path: rel,
                read_error: Some(msg.clone()),
                old: Err(msg.clone()),
                new: Err(msg),
                new_error_stage: Some(ErrorStage::Io),
            };
        }
    };
    let old = old_normalized_from_code(&code);
    let (new, new_error_stage) = match new_normalized_from_code(&code, &SourceInfo::from_path(path))
    {
        Ok((d, tuple)) => (Ok((d.is_static_equivalent(), tuple)), None),
        Err(msg) => (Err(msg.clone()), stage_from_message(&msg)),
    };
    ComparisonReport {
        path: rel,
        read_error: None,
        old,
        new,
        new_error_stage,
    }
}

/// Recover the structural stage from a wrapped new-arm error message
/// (`new arm parse failed: [<stage>] …`).
fn stage_from_message(msg: &str) -> Option<ErrorStage> {
    let rest = msg.split_once("parse failed: [")?.1;
    let stage = rest.split_once(']')?.0;
    match stage {
        "io" => Some(ErrorStage::Io),
        "document" => Some(ErrorStage::Document),
        "entry" => Some(ErrorStage::Entry),
        "value" => Some(ErrorStage::Value),
        "node-body" => Some(ErrorStage::NodeBody),
        _ => None,
    }
}

/// V-03: the explicit legacy adapter remains fully capable on inputs the
/// static model rejects — same code path production uses today.
pub fn legacy_adapter_still_parses(code: &str) -> bool {
    old_normalized_from_code(code).is_ok()
}

// Re-exported for tests asserting toggle parity between the two rule sets.
pub use auto_project_model::UnsupportedKind;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    /// Expected-bad fixtures: the ONLY inputs allowed to be rejected by
    /// BOTH arms (VM parse error + model syntax error), each bound to its
    /// expected new-arm error stage (R2 F-02: identity + diagnosis).
    /// R3 F-02: entries are FULL repo-relative paths — matching is exact
    /// (`exempted_by`), so same-name/same-suffix files in other
    /// directories can never inherit an exemption.
    const EXPECTED_BOTH_REJECTED: [(&str, &str); 2] = [
        (
            "test/fixtures/modularity/project-model/fixtures/bad-missing-brace.pac.at",
            "value",
        ),
        (
            "test/fixtures/modularity/project-model/fixtures/bad-missing-value.pac.at",
            "value",
        ),
    ];
    /// Inputs whose OLD arm fails config-VM parse (render-pipeline-only
    /// template / undefined-variable fixtures) while the model CLASSIFIES
    /// them non-static — full repo-relative paths, exact match only.
    const EXPECTED_TEMPLATE_REJECTED: [&str; 2] = [
        "crates/auto-man-assets/assets/templates/clib/pac.at",
        "test/fixtures/modularity/project-model/fixtures/dynamic-expressions.pac.at",
    ];

    /// Normalize an absolute managed path to a repo-relative `/`-separated
    /// string (exact identity for the exemption lists).
    fn rel_identity(f: &Path, root: &Path) -> String {
        f.strip_prefix(root)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/")
    }

    /// Exact-match exemption lookup — suffix/same-name shadows never match.
    fn exempted_by<'a, T>(rel: &str, list: &'a [T]) -> Option<&'a T>
    where
        T: AsRef<str>,
    {
        list.iter().find(|p| p.as_ref() == rel)
    }

    /// R3 F-02: exemption identity is exact — shadow files sharing a name
    /// or suffix with an allowlisted input must NOT be exempted.
    #[test]
    fn project_model_r3_exemption_identity_is_exact() {
        let root = repo_root();
        let shadow_bad = root.join("scratch/shadow/bad-missing-brace.pac.at");
        let shadow_tpl = root.join("scratch/shadow/fixtures/dynamic-expressions.pac.at");
        assert_eq!(
            rel_identity(&shadow_bad, &root),
            "scratch/shadow/bad-missing-brace.pac.at"
        );
        assert!(exempted_by(
            &rel_identity(&shadow_bad, &root),
            &EXPECTED_BOTH_REJECTED.map(|(p, _)| p.to_string())
        )
        .is_none());
        assert!(exempted_by(
            &rel_identity(&shadow_tpl, &root),
            &EXPECTED_TEMPLATE_REJECTED.map(|p| p.to_string())
        )
        .is_none());
        // the real allowlisted identities do match
        assert!(exempted_by(
            &rel_identity(&fixture("bad-missing-brace.pac.at"), &root),
            &EXPECTED_BOTH_REJECTED.map(|(p, _)| p.to_string())
        )
        .is_some());
        assert!(exempted_by(
            &rel_identity(
                &root.join("crates/auto-man-assets/assets/templates/clib/pac.at"),
                &root
            ),
            &EXPECTED_TEMPLATE_REJECTED.map(|p| p.to_string())
        )
        .is_some());
    }

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .canonicalize()
            .expect("repo root")
    }

    fn fixture(name: &str) -> PathBuf {
        repo_root()
            .join("test")
            .join("fixtures")
            .join("modularity")
            .join("project-model")
            .join("fixtures")
            .join(name)
    }

    fn managed_pac_files() -> Vec<PathBuf> {
        let out = Command::new("git")
            .args(["ls-files", "*.at"])
            .current_dir(repo_root())
            .output()
            .expect("git ls-files");
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        text.lines()
            .map(|l| l.replace('\\', "/"))
            .filter(|l| l.ends_with("pac.at"))
            .map(|l| repo_root().join(l))
            .collect()
    }

    /// The config VM's object-literal key iteration order is NOT stable
    /// across process runs (probed PLAN-780 T-03: same input, different
    /// repr order per run). Verbatim comparison of object-form reprs would
    /// compare noise — normalize them to sorted `key=value` pairs.
    /// (Consumers read keys by name, which is why this never mattered
    /// before; recorded in the T-04 owner notes.)
    fn structural_repr(s: &str) -> String {
        let t = s.trim();
        if !(t.starts_with('{') && t.ends_with('}')) {
            return s.to_string();
        }
        let inner = &t[1..t.len() - 1];
        let mut pairs = Vec::new();
        let mut depth = 0i32;
        let mut cur = String::new();
        let mut in_str: Option<char> = None;
        for c in inner.chars() {
            match in_str {
                Some(q) => {
                    cur.push(c);
                    if c == q {
                        in_str = None;
                    }
                }
                None => match c {
                    '"' | '\'' => {
                        in_str = Some(c);
                        cur.push(c);
                    }
                    '{' | '[' => {
                        depth += 1;
                        cur.push(c);
                    }
                    '}' | ']' => {
                        depth -= 1;
                        cur.push(c);
                    }
                    ',' if depth == 0 => {
                        pairs.push(cur.trim().to_string());
                        cur.clear();
                    }
                    _ => cur.push(c),
                },
            }
        }
        if !cur.trim().is_empty() {
            pairs.push(cur.trim().to_string());
        }
        pairs.sort();
        format!("obj[{}]", pairs.join("|"))
    }

    fn field_diff(old: &FrozenNormalized, new: &FrozenNormalized) -> Vec<String> {
        let mut diffs = Vec::new();
        macro_rules! cmp {
            ($f:ident) => {
                if old.$f != new.$f {
                    diffs.push(format!(
                        "{}: old={:?} new={:?}",
                        stringify!($f),
                        old.$f,
                        new.$f
                    ));
                }
            };
        }
        cmp!(name);
        cmp!(version);
        cmp!(lang_declared);
        if structural_repr(&old.backend_str) != structural_repr(&new.backend_str) {
            diffs.push(format!(
                "backend_str: old={:?} new={:?}",
                old.backend_str, new.backend_str
            ));
        }
        cmp!(backend_shape);
        cmp!(api);
        cmp!(scene);
        cmp!(members_declared);
        cmp!(defines);
        cmp!(title);
        cmp!(title_zh);
        cmp!(icon);
        cmp!(category);
        if structural_repr(old.theme.as_deref().unwrap_or(""))
            != structural_repr(new.theme.as_deref().unwrap_or(""))
        {
            diffs.push(format!("theme: old={:?} new={:?}", old.theme, new.theme));
        }
        cmp!(accent);
        cmp!(desktop_render);
        cmp!(ui_config);
        cmp!(media_root);
        cmp!(photo_root);
        cmp!(window);
        cmp!(front_port);
        cmp!(back_port);
        cmp!(external_backend);
        cmp!(shadcn);
        cmp!(default_classes);
        cmp!(deps);
        diffs
    }

    /// R3 F-01 parity matrix: every legal trivia/separation shape parses
    /// on BOTH arms with equal frozen fields (the review's oracle-first
    /// methodology, made permanent).
    #[test]
    fn project_model_r3_trivia_parity_matrix() {
        let cases = [
            "members: [\"a\" // comment\n, \"b\"]\n",
            "members: [\"a\" /* block */ , \"b\"]\n",
            "cfg: { a: true\nb: false }\n",
            "cfg: [true\nfalse]\n",
            "{ name: \"ok\", back: { project: \"b\" /* comment */ } }",
            "members: [\"a\", \"b\",]\n",
            "members: [ // leading comment\n \"a\"]\n",
            "members: [true // c\n, false]\n",
            // NOTE: object comma AFTER a passed break is an old-arm ERROR
            // (R3 oracle probe) — it is a negative control in the model
            // tests, not a parity case.
        ];
        for code in cases {
            let old = old_normalized_from_code(code)
                .unwrap_or_else(|e| panic!("old arm must accept {code:?}: {e}"));
            let (d, new) = new_normalized_from_code(code, &SourceInfo::inline())
                .unwrap_or_else(|e| panic!("new arm must accept {code:?}: {e}"));
            assert!(d.is_static_equivalent(), "{code:?}");
            let diffs = field_diff(&old, &new);
            assert!(diffs.is_empty(), "{code:?}: {diffs:?}");
        }
    }

    /// V-02 (AC-02): over the whole managed corpus — every file classified;
    /// every static-equivalent file equal on the frozen subset; dynamic
    /// files classified (never faked green).
    #[test]
    fn project_model_corpus_comparison_v02() {
        let files = managed_pac_files();
        assert!(
            files.len() >= 140,
            "expected the managed corpus (149 at T-01), got {}",
            files.len()
        );
        let mut template_files: Vec<String> = Vec::new();
        let mut both_files: Vec<String> = Vec::new();
        let mut dynamic_files: Vec<String> = Vec::new();
        let mut static_eq = 0usize;
        let mut dynamic = 0usize;
        let mut template_rejected = 0usize;
        let mut both_rejected = 0usize;
        let mut failures = Vec::new();
        for f in &files {
            let report = compare_file(f);
            if let Some(e) = &report.read_error {
                // R1 F-04: a required managed input that cannot be read
                // fails the gate outright — never an empty-config pass.
                failures.push(format!("{}: {}", report.path, e));
                continue;
            }
            match (&report.old, &report.new) {
                (Ok(old), Ok((true, new))) => {
                    let diffs = field_diff(old, new);
                    if diffs.is_empty() {
                        static_eq += 1;
                    } else {
                        failures.push(format!(
                            "{}: static file with field diffs: {:?}",
                            report.path, diffs
                        ));
                    }
                }
                (Ok(_old), Ok((false, _new))) => {
                    // dynamic/unsupported — old arm still parses (V-03);
                    // equality is never claimed.
                    dynamic += 1;
                    dynamic_files.push(report.path.clone());
                }
                (Ok(_), Err(e)) => {
                    failures.push(format!("{}: new arm syntax error: {}", report.path, e));
                }
                // The legacy VM itself rejects some template assets (clib's
                // `lib ${name} {}` renders only through eval_template, not
                // eval_config_with_vm) — model classifying non-static while
                // the old arm errors is the honest pair (error preserved,
                // no silent fallback).
                (Err(_old_err), Ok((false, _))) => {
                    // R3 F-02: exact repo-relative identity — same-name
                    // shadows in other directories never inherit exemption.
                    let rel = rel_identity(f, &repo_root());
                    if EXPECTED_TEMPLATE_REJECTED.iter().any(|p| p == &rel) {
                        template_rejected += 1;
                        template_files.push(rel.clone());
                    } else {
                        failures.push(format!(
                            "{}: UNEXPECTED template-rejected input — old arm error outside the template allowlist",
                            report.path
                        ));
                    }
                }
                // both arms refuse — ONLY expected-bad fixtures may land
                // here (identity-bound allowlist below); an unexpected
                // corrupted required input must fail the gate (R2 F-02).
                (Err(_), Err(_)) => {
                    let rel = rel_identity(f, &repo_root());
                    match EXPECTED_BOTH_REJECTED.iter().find(|(p, _)| p == &rel) {
                        Some(&(_, want_stage)) => {
                            assert_eq!(
                                report.new_error_stage.map(|s| s.to_string()),
                                Some(want_stage.to_string()),
                                "{}: unexpected new-arm error stage",
                                report.path
                            );
                            both_rejected += 1;
                            both_files.push(rel.clone());
                        }
                        None => failures.push(format!(
                            "{}: UNEXPECTED both-rejected input (not in the expected-bad allowlist) — investigate before gating",
                            report.path
                        )),
                    }
                }
                (Err(e), Ok((true, _))) => failures.push(format!(
                    "{}: OLD arm failed but model claims static: {}",
                    report.path, e
                )),
            }
        }
        assert!(
            failures.is_empty(),
            "corpus comparison failures:\n{}",
            failures.join("\n")
        );
        assert!(static_eq >= 140, "static-equivalent count {static_eq}");
        assert!(
            dynamic + template_rejected + both_rejected >= 1,
            "dynamic files must be classified, got 0"
        );
        // Every managed file classifies: static-equal / dynamic-classified /
        // template-rejected (old arm errors, model classifies) /
        // both-rejected (expected-bad fixtures) — read failures already
        // failed above.
        assert_eq!(
            static_eq + dynamic + template_rejected + both_rejected,
            files.len()
        );
        eprintln!(
            "P780 corpus: total={} static_equal={} dynamic_classified={} template_rejected={} both_rejected={}",
            files.len(),
            static_eq,
            dynamic,
            template_rejected,
            both_rejected
        );
        for (bucket, list) in [
            ("template_rejected", &template_files),
            ("both_rejected", &both_files),
            ("dynamic_classified", &dynamic_files),
        ] {
            eprintln!("P780 bucket {} = {:?}", bucket, list);
        }
    }

    /// V-03 (AC-02): dynamic inputs — model rejects/classifies, the explicit
    /// legacy adapter still parses them, no silent fallback.
    #[test]
    fn project_model_dynamic_inputs_v03() {
        for name in ["dynamic-template.pac.at", "dynamic-expressions.pac.at"] {
            let code = std::fs::read_to_string(fixture(name)).unwrap();
            let (d, _tuple) =
                new_normalized_from_code(&code, &SourceInfo::from_path(fixture(name))).unwrap();
            assert!(
                !d.is_static_equivalent(),
                "{name} must not be static-equivalent"
            );
            assert!(!d.unsupported.is_empty(), "{name} needs reasons");
        }
        // The real scaffold template (corpus's dynamic quintet) still parses
        // on the legacy arm — that capability is untouched.
        let template = repo_root()
            .join("crates")
            .join("auto-man-assets")
            .join("assets")
            .join("templates")
            .join("app")
            .join("pac.at");
        let code = std::fs::read_to_string(&template).unwrap();
        let (d, tuple) =
            new_normalized_from_code(&code, &SourceInfo::from_path(&template)).unwrap();
        assert!(!d.is_static_equivalent());
        assert!(
            legacy_adapter_still_parses(&code),
            "legacy VM arm must parse the template (old capability preserved)"
        );
        // and the model never fakes the interpolated field
        assert_eq!(tuple.name, "");
    }

    /// V-04 (AC-03): path base points — dep path / back.project resolve
    /// against the pac.at's own directory, matching the production
    /// `external_backend_dir` behavior exactly.
    #[test]
    fn project_model_path_base_points_v04() {
        // Stage a real `pac.at` in a temp dir (production readers look the
        // file up by that exact name).
        let tmp = std::env::temp_dir().join(format!("p780-v04-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let code = std::fs::read_to_string(fixture("static-nested-objects.pac.at")).unwrap();
        std::fs::write(tmp.join("pac.at"), &code).unwrap();
        // legacy behavior
        let legacy = auto_lang_core::config::external_backend_dir(&tmp)
            .expect("legacy external_backend_dir resolves");
        // model declaration + documented base point (the pac.at's own dir)
        let d = auto_project_model::parse_file(&tmp.join("pac.at")).unwrap();
        let declared = d.external_backend.as_deref().expect("model back.project");
        let model_resolved = tmp.join(declared);
        assert_eq!(legacy, model_resolved);
        std::fs::remove_dir_all(&tmp).unwrap();
        // dep path base point: declaration + source path carry the base
        let deps_fixture = fixture("static-dep-forms.pac.at");
        let deps_dir = deps_fixture.parent().unwrap();
        let dd = auto_project_model::parse_file(&deps_fixture).unwrap();
        let rel = dd.deps[0].path.clone().unwrap();
        assert_eq!(
            dd.source.path.as_deref(),
            Some(deps_fixture.as_path()),
            "source path recorded for base-point resolution"
        );
        assert!(
            deps_dir.join(&rel).exists() || rel.starts_with(".."),
            "dep path documented as relative to the pac.at dir"
        );
    }

    /// V-05 (AC-03): bad inputs — clear stage/path diagnosis from the model;
    /// parsing has no side effects (input bytes and directory unchanged).
    #[test]
    fn project_model_bad_inputs_and_no_effects_v05() {
        let before = std::fs::read_to_string(fixture("bad-missing-value.pac.at")).unwrap();
        let dir_before: Vec<_> =
            std::fs::read_dir(fixture("bad-missing-value.pac.at").parent().unwrap())
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
                .collect();
        let err = auto_project_model::parse_file(&fixture("bad-missing-value.pac.at"))
            .expect_err("missing value must error");
        assert_eq!(err.stage, auto_project_model::ErrorStage::Value);
        assert!(err.to_string().contains("bad-missing-value.pac.at"));
        // duplicate keys: classified (recorded), later-wins on both arms
        let code = std::fs::read_to_string(fixture("static-dup-keys.pac.at")).unwrap();
        let old = old_normalized_from_code(&code).unwrap();
        let (d, new) = new_normalized_from_code(&code, &SourceInfo::inline()).unwrap();
        assert!(d.is_static_equivalent());
        assert_eq!(d.duplicates.len(), 2);
        assert_eq!(old.title, new.title);
        assert_eq!(old.backend_shape, new.backend_shape);
        // no effects
        let after = std::fs::read_to_string(fixture("bad-missing-value.pac.at")).unwrap();
        assert_eq!(before, after, "parse must not modify its input");
        let dir_after: Vec<_> =
            std::fs::read_dir(fixture("bad-missing-value.pac.at").parent().unwrap())
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
                .collect();
        assert_eq!(dir_before, dir_after, "parse must not write new files");
    }

    /// Fixture edge cases beyond the corpus (V-02 frozen subset forms).
    #[test]
    fn project_model_fixture_edge_forms_v02() {
        for name in [
            "static-basic.pac.at",
            "static-dep-forms.pac.at",
            "static-render-forms.pac.at",
            "static-render-multi.pac.at",
            "static-render-unknown.pac.at",
            "static-workspace-fields.pac.at",
            "static-nested-objects.pac.at",
            "static-target-nodes.pac.at",
            "static-dup-keys.pac.at",
            "static-toggles.pac.at",
            "static-dep-dup-path.pac.at",
            "static-unknown-node-content.pac.at",
            "static-empty.pac.at",
        ] {
            let path = fixture(name);
            let report = compare_file(&path);
            let old = report.old.as_ref().expect("old arm parses fixture");
            let (static_eq, new) = report.new.as_ref().expect("new arm parses fixture");
            assert!(static_eq, "{name} should be static-equivalent");
            let diffs = field_diff(old, new);
            assert!(
                diffs.is_empty(),
                "{name} frozen-field diffs: {diffs:?}\nold={old:#?}\nnew={new:#?}"
            );
        }
    }

    /// R1 F-01/F-05: structured-value variables and the root wrapper never
    /// fake static equivalence; the old arm still evaluates them.
    #[test]
    fn project_model_r1_dynamic_structured_and_wrapper() {
        for name in [
            "dynamic-structured-values.pac.at",
            "dynamic-top-object-values.pac.at",
            "dynamic-root-wrapper.pac.at",
        ] {
            let code = std::fs::read_to_string(fixture(name)).unwrap();
            let (d, _tuple) = new_normalized_from_code(&code, &SourceInfo::inline()).unwrap();
            assert!(!d.is_static_equivalent(), "{name} must classify");
            assert!(!d.unsupported.is_empty(), "{name} needs reasons");
            // old arm keeps evaluating (V-03: no capability loss)
            assert!(
                old_normalized_from_code(&code).is_ok(),
                "{name}: legacy arm still parses"
            );
        }
        // old arm resolves the injected probe arg where the model refuses
        let code = std::fs::read_to_string(fixture("dynamic-structured-values.pac.at")).unwrap();
        let old = old_normalized_from_code(&code).unwrap();
        assert_eq!(old.members_declared, Some(vec!["win32".to_string()]));
        let (d, new) = new_normalized_from_code(&code, &SourceInfo::inline()).unwrap();
        assert_eq!(new.members_declared, None);
        assert!(d
            .unsupported
            .iter()
            .any(|u| u.kind == UnsupportedKind::BareIdentValue));
    }

    /// R1 F-04: read failures are observable gate failures — never an
    /// empty-config comparison. Legal empty file stays a valid control.
    #[test]
    fn project_model_r1_read_failures_propagate() {
        let missing =
            std::env::temp_dir().join(format!("p780-r1-missing-{}-pac.at", std::process::id()));
        assert!(!missing.exists());
        let report = compare_file(&missing);
        assert!(report.read_error.is_some());
        assert!(report.old.is_err() && report.new.is_err());
        assert_eq!(report.equal_claim(), None);

        let dir = std::env::temp_dir().join(format!("p780-r1-utf8-{}", std::process::id()));
        assert!(!dir.exists());
        std::fs::create_dir(&dir).unwrap();
        let bad = dir.join("pac.at");
        std::fs::write(&bad, [255u8, 254u8]).unwrap();
        let report = compare_file(&bad);
        std::fs::remove_file(&bad).unwrap();
        std::fs::remove_dir(&dir).unwrap();
        assert!(
            report.read_error.is_some(),
            "invalid UTF-8 must not become empty equality"
        );
        assert!(report.old.is_err() && report.new.is_err());

        // difference control: an empty file IS a legal empty config
        let code = std::fs::read_to_string(fixture("static-empty.pac.at")).unwrap();
        assert_eq!(code, "");
        let old = old_normalized_from_code(&code).unwrap();
        let (d, new) = new_normalized_from_code(&code, &SourceInfo::inline()).unwrap();
        assert!(d.is_static_equivalent());
        assert_eq!(old, new);
    }
}

#[cfg(test)]
mod p780_separator_semantics {
    /// R3 F-01 oracle pin (asymmetric separator grammar): object commas
    /// are same-line-only; array commas/newlines mix freely after a value.
    #[test]
    fn oracle_asymmetric_separator_semantics() {
        use super::old_normalized_from_code as old;
        // object: legal comma placements
        assert!(old("cfg: { a: 1, }\n").is_ok(), "obj trailing comma Ok");
        assert!(
            old("cfg: { a: 1,\nb: 2 }\n").is_ok(),
            "obj comma before newline Ok"
        );
        // object: comma AFTER a passed break errors ("Expected key")
        assert!(
            old("cfg: { a: 1\n, b: 2 }\n").is_err(),
            "obj comma after newline Err"
        );
        assert!(
            old("cfg: { a: 1 // c\n, b: 2 }\n").is_err(),
            "obj comment then comma Err"
        );
        assert!(
            old("cfg: { a: 1,, b: 2 }\n").is_err(),
            "obj double comma Err"
        );
        // array: comma runs and comment-then-comma are legal after a value;
        // a comma at element position errors
        assert!(
            old("cfg: [1 // c\n, 2]\n").is_ok(),
            "arr comma after comment Ok"
        );
        assert!(old("cfg: [1,, 2]\n").is_ok(), "arr double comma Ok");
        assert!(
            old("cfg: [\n, 1]\n").is_err(),
            "arr leading comma after newline Err"
        );
    }

    /// R2 F-01 oracle pin: the config VM's separator grammar for structured
    /// values — same-line adjacency errors; newline/comma/trailing-comma
    /// are legal. The static scanner's `check_separation` mirrors this.
    #[test]
    fn oracle_separator_semantics() {
        use super::old_normalized_from_code as old;
        assert!(
            old("members: [1 2 3]\n").is_err(),
            "same-line adjacency errors"
        );
        assert_eq!(
            old("members: [\n1\n2\n]\n").unwrap().members_declared,
            Some(vec!["1".to_string(), "2".to_string()]),
            "newline separation is legal"
        );
        assert_eq!(
            old("members: [\n1,\n2\n]\n").unwrap().members_declared,
            Some(vec!["1".to_string(), "2".to_string()]),
            "multiline commas are legal"
        );
        assert_eq!(
            old("members: [1, 2,]\n").unwrap().members_declared,
            Some(vec!["1".to_string(), "2".to_string()]),
            "trailing comma is legal"
        );
    }
}
