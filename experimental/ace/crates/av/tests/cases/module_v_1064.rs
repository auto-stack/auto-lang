//! PLAN-1064 T-MODULE-V — the MODULE slice's real-consumer legs: the
//! §5.2 demo EXECUTES to 42 through the packages chain on the AV, and
//! the REAL canonical author sources (json.at / redis.at, byte-exact
//! tracked files) are consumed cross-module through the assembly with
//! execution oracles. SQLite's canonical leg is named-blocked on 975's
//! T-03 pub migration (its tracked file is the foreign-lane plain-fn
//! spelling today; the contract forbids mirror sources).
//!
//! Negatives: a pub-stripped mirror of json.at must fail the cross-
//! module import (visibility), the reserved `std` package identity
//! rejects a forged module, and duplicate module identities reject.
use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::contract::Value;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use ace_source_query::modules::packages::compile_package_product;

fn caps() -> [String; 1] {
    [PACKAGES_CAPABILITY.to_string()]
}

fn input(package: &str, id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: package.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn scratch(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("p1064v-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn run_value(product: &ace_source_query::modules::packages::PackageProduct) -> Value {
    let dir = scratch("run");
    let target = PackageAvTarget::cold(&dir, product).unwrap();
    let value = target.run_value().unwrap_or_else(|e| panic!("run: {e}"));
    let _ = std::fs::remove_dir_all(&dir);
    value
}

/// The §5.2 demo executes to 42 (private helper in-module, pub answer
/// cross-module, module-scope use).
#[test]
fn module_1064_v_demo_executes_42() {
    let product = compile_package_product(
        &[
            input(
                "app",
                "demo",
                "fn helper(x int) int { return x + 1 }\npub fn answer() int { return helper(41) }",
            ),
            input(
                "app",
                "main",
                "use demo\npub fn main() int { return demo.answer() }",
            ),
        ],
        "app::main",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    )
    .expect("compiles");
    assert_eq!(run_value(&product), Value::I32(42));
}

/// The REAL json.at (tracked file, byte-exact) consumed cross-module:
/// user.name reads "Michi" (nested object) and the services array path
/// resolves — both through the assembly, executed on the AV.
#[test]
fn module_1064_v_json_at_real_source_executes() {
    let json_at = include_str!("../../../../../../stdlib/ace/std/json.at");
    let product = compile_package_product(
        &[
            input("app", "jsond", json_at),
            input(
                "app",
                "main",
                concat!(
                    "use app.jsond.json_kind_at\n",
                    "use app.jsond.json_parse_str\n",
                    "use app.jsond.json_is_valid\n",
                    "pub fn main() int {\n",
                    "    let doc str = \"{\\\"user\\\": {\\\"name\\\": \\\"Michi\\\"}, \\\"services\\\": [{\\\"port\\\": 8090}]}\"\n",
                    "    if !json_is_valid(doc) { return 9001 }\n",
                    "    if json_kind_at(doc, \"user.name\") != 4 { return 9002 }\n",
                    "    let n Str = json_parse_str(doc, \"user.name\")\n",
                    "    let want Str = \"Michi\"\n",
                    "    if n != want { return 9003 }\n",
                    "    if json_kind_at(doc, \"services.0.port\") != 3 { return 9004 }\n",
                    "    return 42\n",
                    "}",
                ),
            ),
        ],
        "app::main",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    )
    .expect("the real json.at assembles and binds cross-module");
    assert_eq!(run_value(&product), Value::I32(42));
}

/// The REAL redis.at consumed cross-module through the assembly at the
/// compile/verify level (checked HIR): the pure public helpers
/// (redis_int_to_str / redis_cmd_get) bind and the module's platform
/// intrinsics ride the package profile. EXECUTION on the owned lane is
/// a NAMED boundary here — the av.owned/1 first cut lowers no platform
/// intrinsics, and redis's net face runs on the platform lane (per the
/// 974/975 runtime tests); this leg pins the assembly/binding face.
#[test]
fn module_1064_v_redis_at_real_source_binds() {
    let redis_at = include_str!("../../../../../../stdlib/ace/std/redis.at");
    let product = compile_package_product(
        &[
            input("app", "redisd", redis_at),
            input(
                "app",
                "main",
                concat!(
                    "use app.redisd.redis_int_to_str\n",
                    "use app.redisd.redis_cmd_get\n",
                    "pub fn main() int {\n",
                    "    let s Str = redis_int_to_str(123)\n",
                    "    let want Str = \"123\"\n",
                    "    if s != want { return 9001 }\n",
                    "    let cmd Str = redis_cmd_get(\"k\")\n",
                    "    if cmd.len_bytes() != 22 { return 9002 }\n",
                    "    return 42\n",
                    "}",
                ),
            ),
        ],
        "app::main",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    )
    .expect("the real redis.at assembles and binds cross-module");
    assert!(
        product
            .compiled
            .checked
            .bundle()
            .modules
            .iter()
            .any(|m| m.defs.iter().any(|d| d.id_text == "d_main")),
        "the consuming app::main reaches checked HIR"
    );
    // named boundary: the owned-lane lower refuses the module's platform
    // intrinsics — redis execution stays on the platform lane
    let dir = scratch("redis-lane");
    let err = match PackageAvTarget::cold(&dir, &product) {
        Ok(_) => panic!("owned-lane lower must refuse the platform intrinsics"),
        Err(e) => e,
    };
    assert!(
        err.contains("lower.intrinsic"),
        "named intrinsic boundary, got {err}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Negative: a pub-STRIPPED mirror of the json surface (the "删pub"
/// tamper) must fail the cross-module import — visibility is enforced
/// against the real declaration, not the consumer's expectation.
#[test]
fn module_1064_v_pub_stripped_mirror_is_red() {
    let mirror = "fn json_is_valid(s str) bool {\n    return s.len() >= 0\n}";
    let err = match compile_package_product(
        &[
            input("app", "jsond", mirror),
            input(
                "app",
                "main",
                "use app.jsond.json_is_valid\npub fn main() int { return 1 }",
            ),
        ],
        "app::main",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        Ok(_) => panic!("the pub-stripped mirror must not be importable"),
        Err(ds) => ds,
    };
    assert!(
        err.iter()
            .any(|d| d.diagnostic.code == "source.module-private"),
        "named module-private, got {:?}",
        err.iter()
            .map(|d| d.diagnostic.code.clone())
            .collect::<Vec<_>>()
    );
}

/// Negative: the reserved `std` package identity rejects a forged
/// module (the 错std身份 tamper — no arbitrary `std` files).
#[test]
fn module_1064_v_forged_std_identity_is_red() {
    let err = match compile_package_product(
        &[input(
            "std",
            "json",
            "pub fn json_is_valid(s str) bool {\n    return true\n}",
        )],
        "std::json",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        Ok(_) => panic!("a forged std module must be rejected"),
        Err(ds) => ds,
    };
    assert!(
        err.iter()
            .any(|d| d.diagnostic.code == "ace.stdlib-reserved"),
        "named stdlib-reserved, got {:?}",
        err.iter()
            .map(|d| d.diagnostic.code.clone())
            .collect::<Vec<_>>()
    );
}

/// Negative: duplicate module identity (同名模块污染) rejects.
#[test]
fn module_1064_v_duplicate_module_identity_is_red() {
    let err = match compile_package_product(
        &[
            input("app", "demo", "pub fn a() int { return 1 }"),
            input("app", "demo", "pub fn b() int { return 2 }"),
        ],
        "app::demo",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        Ok(_) => panic!("duplicate identities must be rejected"),
        Err(ds) => ds,
    };
    assert!(
        err.iter()
            .any(|d| d.diagnostic.code == "source.module-identity"),
        "named module-identity, got {:?}",
        err.iter()
            .map(|d| d.diagnostic.code.clone())
            .collect::<Vec<_>>()
    );
}

/// V-MODULE-RESTORE: the packages store roundtrip re-admits the SAME
/// declaration set (identity hash equal, the demo still executes 42
/// from the restored product), and a tampered source (changed private
/// helper body) cannot ride the old product — prepare validates the
/// product/inputs identity, so stale signatures do not admit.
#[test]
fn module_1064_v_module_restore_roundtrip_and_stale_rejection() {
    use ace_source_query::modules::{PackageStore, PackageStoreReceipt};
    let demo_inputs = vec![
        input(
            "app",
            "demo",
            "fn helper(x int) int { return x + 1 }
pub fn answer() int { return helper(41) }",
        ),
        input(
            "app",
            "main",
            "use demo
pub fn main() int { return demo.answer() }",
        ),
    ];
    let product = compile_package_product(
        &demo_inputs,
        "app::main",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    )
    .expect("compiles");

    // ---- roundtrip: prepare -> restore -> same identity, executes 42
    let root = scratch("store");
    let receipt = PackageStore::prepare(&root, &product, &demo_inputs, &[]).expect("store prepare");
    let targets = vec![(
        "av".to_string(),
        PackageStoreReceipt {
            artifact: receipt.artifact.clone(),
            digest: receipt.digest.clone(),
        },
    )];
    let _ = targets; // identity-only persistence: restore from the receipt
    let (restored, _restored_targets) =
        PackageStore::restore(&root, &receipt).expect("store restore");
    assert_eq!(
        restored.bound.input_hash, product.bound.input_hash,
        "the restored product carries the SAME declaration identity"
    );
    assert_eq!(run_value(&restored), Value::I32(42));

    // ---- a changed PRIVATE helper body moves the module identity (the
    // "改helper正文后真实readers失效" mechanism): the old receipt/artifact
    // cannot serve the new source; and a tampered digest never admits
    let tampered = vec![
        input(
            "app",
            "demo",
            "fn helper(x int) int { return x + 2 }
pub fn answer() int { return helper(41) }",
        ),
        demo_inputs[1].clone(),
    ];
    let tampered_product = compile_package_product(
        &tampered,
        "app::main",
        &caps(),
        FrontBudget::WORK_OWNED_64MIB,
    )
    .expect("the tampered source compiles on its own");
    assert_ne!(
        tampered_product.bound.input_hash, product.bound.input_hash,
        "a changed private helper body moves the identity — stale readers invalidate"
    );
    let bad = PackageStoreReceipt {
        artifact: receipt.artifact.clone(),
        digest: "0".repeat(64),
    };
    assert!(
        PackageStore::restore(&root, &bad).is_err(),
        "a tampered/stale digest must not admit"
    );
    let _ = std::fs::remove_dir_all(&root);
}
