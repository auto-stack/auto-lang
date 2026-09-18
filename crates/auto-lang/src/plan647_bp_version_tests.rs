//! PLAN-647: bp 版本面裁定——两护栏机器面（contract Q5 终版规则的负测试）。
//!
//! 护栏 A：spec.md frontmatter 顶层 `version` 键 → parse 显式错误（此前 serde
//! 静默忽略——半吊子版本化口子①）。护栏 B：pac.at `dep` 声明版本类键
//! （version/pin/rev/…）→ 解析拒绝 + 显式报错（此前 `path:` 之外键面视而不
//! 见——口子②）。裁定正文：docs/specs/blueprint/contract.md Q5；债源：
//! KNOWN-DEBT P639-D3。
//!
//! 正测试不回归面：`[dataSource]` 表内的 `version` 槽名不是版本键（fetcher
//! 签名可叫 version）；无版本键的 path dep 照常解析。

use std::path::PathBuf;

// ── 护栏 A：spec.md frontmatter `version` 键 ────────────────────────────

#[test]
fn spec_frontmatter_version_key_is_rejected() {
    let md = "+++\nkind = \"form\"\nname = \"login\"\nversion = \"2.0.0\"\npalette = [\"Input\"]\nvariants = []\n+++\nbody";
    let err = crate::ui_gen::bp::BlueprintSpec::parse_document(md).unwrap_err();
    assert!(
        err.contains("Q5"),
        "guardrail error must point at the contract ruling, got: {err}"
    );
    assert!(
        err.contains("version"),
        "guardrail error must name the offending key, got: {err}"
    );
}

#[test]
fn spec_frontmatter_version_key_rejects_pinline_forms() {
    // pin/rev 等变体键同属版本面——同样拒绝（护栏语义键集对齐护栏 B）。
    for key in ["pin", "rev", "tag", "branch", "commit"] {
        let md = format!(
            "+++\nkind = \"form\"\nname = \"x\"\n{key} = \"abc123\"\npalette = []\nvariants = []\n+++\nbody"
        );
        let err = crate::ui_gen::bp::BlueprintSpec::parse_document(&md).unwrap_err();
        assert!(err.contains("Q5"), "key {key} must be rejected, got: {err}");
    }
}

#[test]
fn spec_frontmatter_datasource_version_slot_is_not_a_version_key() {
    // 护栏 A 只看顶层键：[dataSource] 表内的 `version` 是合法槽名。
    let md = "+++\nkind = \"form\"\nname = \"x\"\npalette = []\nvariants = []\n\n[dataSource]\nversion = \"(id) -> Meta\"\n+++\nbody";
    let (spec, _) = crate::ui_gen::bp::BlueprintSpec::parse_document(md).unwrap();
    assert_eq!(spec.data_source.get("version").unwrap(), "(id) -> Meta");
}

// ── 护栏 B：pac.at `dep` 声明版本类键 ───────────────────────────────────

/// Pure-scan surface: version-ish keys inside a dep declaration are flagged,
/// with the error pointing at the Q5 ruling.
#[test]
fn pac_dep_version_keys_are_flagged() {
    for key in ["version", "pin", "rev", "tag", "branch", "commit"] {
        let pac = format!("name: \"app\"\ndep bps {{\n    path: \"../blueprints\"\n    {key}: \"0.1.0\"\n}}\n");
        let err = crate::pac_dep_version_violation(&pac, "bps")
            .unwrap_or_else(|| panic!("key {key} must be flagged"));
        assert!(err.contains("Q5"), "error must point at Q5, got: {err}");
    }
}

#[test]
fn pac_dep_clean_declaration_is_not_flagged() {
    let pac = "name: \"app\"\ndep bps {\n    path: \"../blueprints\"\n}\n";
    assert!(crate::pac_dep_version_violation(pac, "bps").is_none());
}

#[test]
fn pac_dep_word_boundary_keys_do_not_false_positive() {
    // `versions:` / `prev:` 等含版本键子串的邻近名不得误伤（词首边界）。
    let pac = "name: \"app\"\ndep bps {\n    path: \"../blueprints\"\n    versions: []\n    preview: true\n}\n";
    assert!(crate::pac_dep_version_violation(pac, "bps").is_none());
    // 别的 dep 块带版本键不影响本 dep 的判定（按名扫描）。
    let pac = "name: \"app\"\ndep bps {\n    path: \"../blueprints\"\n}\ndep other {\n    version: \"1.0\"\n}\n";
    assert!(crate::pac_dep_version_violation(pac, "bps").is_none());
    assert!(crate::pac_dep_version_violation(pac, "other").is_some());
}

/// tempdir fixture：`app/`（pac.at + src/front/app.at）+ `app/<dep_dir>/` 包体。
/// 返回 base_dir（解析起点 = src/front，与生产 use 解析同位）。
fn dep_fixture(dep_dir: &str, pac_body: &str) -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let app = tmp.path().join("app");
    let front = app.join("src").join("front");
    std::fs::create_dir_all(&front).expect("mkdir front");
    let lib_dir = app.join(dep_dir).join("form");
    std::fs::create_dir_all(&lib_dir).expect("mkdir dep");
    std::fs::write(app.join("pac.at"), pac_body).expect("write pac.at");
    std::fs::write(lib_dir.join("login.at"), "widget Login {}\n").expect("write login.at");
    std::fs::write(front.join("app.at"), "use bps.form.login: Login\n").expect("write app.at");
    (tmp, front)
}

/// 集成面：带版本键的 dep 声明 → `resolve_module_path` 拒绝（即使 path 本可
/// 解析——硬失败语义，防"看起来有版本约束"的假象）。
#[test]
fn resolve_module_path_rejects_dep_with_version_key() {
    let (_tmp, front) = dep_fixture(
        "bpslib",
        "name: \"app\"\ndep bps {\n    path: \"bpslib\"\n    version: \"0.1.0\"\n}\n",
    );
    assert!(crate::resolve_module_path(&front, "bps.form.login").is_none());
}

/// 正对照：同构 fixture 去掉版本键 → 照常解析（护栏只打版本键，不伤 path dep）。
#[test]
fn resolve_module_path_still_resolves_clean_path_dep() {
    let (_tmp, front) =
        dep_fixture("bpslib", "name: \"app\"\ndep bps {\n    path: \"bpslib\"\n}\n");
    let resolved = crate::resolve_module_path(&front, "bps.form.login");
    let ok = resolved.as_ref().is_some_and(|p| p.ends_with("login.at"));
    assert!(ok, "expected clean resolution, got {resolved:?}");
}
