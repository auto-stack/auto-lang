//! PLAN-982（982/RUST I，第二步）：**宿主驱动面**的永久用例。
//!
//! 产品当前仍走旧 `transpile_rust*`；本桥提供"源码 → 受检 HIR → view →
//! lower → FileSet"的唯一边界。这里钉住四件事：
//! 1. 真实源码产出**冻结三件套**（wrapper/runtime/generated）与 schema manifest；
//! 2. 同输入两次调用**逐字节一致**（下游逐文件 diff 的前提）；
//! 3. 失败**具名**（前端 `ace.rust/compile`、入口 `ace.rust/lower`），不返回空集；
//! 4. 入口名是降级后的 Rust 名（`fn_main`），调用方不需要自己猜。

use auto_lang_ace_host::rust_transpile::transpile_scalar_source;

const SRC: &str = r#"
fn add(a int, b int) int { return a + b }

fn main() int {
    var total = add(2, 3)
    return total
}
"#;

#[test]
fn p982_rust_host_driver_renders_the_frozen_triple() {
    let fileset = transpile_scalar_source(SRC, "p982-rust", "main").expect("rendered");
    assert_eq!(
        fileset
            .files
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        vec!["main.rs", "a2x_runtime.rs", "generated.rs"]
    );
    assert!(fileset.files.iter().all(|f| !f.text.trim().is_empty()));
    assert_eq!(fileset.entry, "fn_main");
    assert!(
        fileset.manifest.contains("auto.a2x.rust.fileset/1"),
        "manifest schema: {}",
        fileset.manifest
    );
    assert!(
        fileset.files[2].text.contains("fn_main"),
        "generated.rs carries the lowered entry"
    );
}

#[test]
fn p982_rust_host_driver_is_deterministic() {
    let a = transpile_scalar_source(SRC, "p982-rust", "main").expect("first");
    let b = transpile_scalar_source(SRC, "p982-rust", "main").expect("second");
    assert_eq!(a, b, "two renders must be byte-identical");
}

#[test]
fn p982_rust_host_driver_names_its_refusals() {
    // 前端拒绝：具名且不返回空集。
    let err = transpile_scalar_source("fn main() int { this is not auto }", "p982", "main")
        .expect_err("frontend refuses");
    assert_eq!(err.code, "ace.rust/compile", "{err}");
    assert!(!err.message.is_empty());

    // 入口拒绝：具名 `ace.rust/lower`，绝不静默挑第一个函数。
    let err = transpile_scalar_source(SRC, "p982-rust", "no_such_fn").expect_err("entry");
    assert_eq!(err.code, "ace.rust/lower", "{err}");
}
/// 家族覆盖（B-CONFIG 之后的 RUST 差额）：records 与 collections 走各自的受检
/// 前门，仍产出同形 FileSet（三件套 + schema manifest），并保持确定性。
#[test]
fn p982_rust_host_driver_covers_records_and_collections_families() {
    use auto_lang_ace_host::rust_transpile::{transpile_with_profile, TranspileProfile};

    let cases = [
        (
            TranspileProfile::Records,
            "type point { x int; y int }\n\nfn main() int {\n    return 7\n}\n",
            "fn_main",
        ),
        (
            TranspileProfile::Collections,
            "fn main() int { let a Str = \"中\"\n let b = a.clone()\n return a.len() * 40 + b.len() * 2 }\n",
            "fn_main",
        ),
    ];
    for (profile, source, entry) in cases {
        let a = transpile_with_profile(source, "p982-family", "main", profile)
            .unwrap_or_else(|e| panic!("{profile:?} must render: {e}"));
        let b = transpile_with_profile(source, "p982-family", "main", profile).unwrap();
        assert_eq!(a, b, "{profile:?} rendering must be deterministic");
        assert_eq!(a.entry, entry, "{profile:?} lowered entry");
        assert_eq!(
            a.files.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(),
            vec!["main.rs", "a2x_runtime.rs", "generated.rs"],
            "{profile:?} keeps the frozen triple"
        );
        // 家族各自有冻结的 manifest schema（collections 用 `COLL_FILESET_SCHEMA`），
        // 断言"是 a2x 的 fileset schema"而不是写死一个。
        assert!(
            a.manifest.contains("fileset/1"),
            "{profile:?} manifest schema: {}",
            a.manifest
        );
    }
}
/// collections 装配器要的 `full_source_sha256` 必须是**真实**内容摘要（round-28
/// 的占位符会污染 manifest/证书身份）。用 NIST 向量钉住。
#[test]
fn p982_rust_source_sha256_matches_the_known_vectors() {
    use auto_lang_ace_host::rust_transpile::{transpile_with_profile, TranspileProfile};

    // 空源：A2R 前端会拒绝，但 collections 装配器只在成功路径上用它；因此这里
    // 通过 manifest 里的 sha256 字段验证**非空源**的真实摘要。
    let src = "fn main() int { let a Str = \"中\"\n return a.len() }\n";
    let fileset = transpile_with_profile(src, "p982-sha", "main", TranspileProfile::Collections)
        .expect("collections render");
    let expected = {
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(src.as_bytes());
        digest.iter().map(|b| format!("{b:02x}")).collect::<String>()
    };
    assert!(
        fileset.manifest.contains(&expected),
        "the manifest must carry the real sha256 of the source (expected {expected})"
    );
}