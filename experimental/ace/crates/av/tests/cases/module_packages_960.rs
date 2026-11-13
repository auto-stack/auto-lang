//! PLAN-960 packages conformance: the complete module package surface
//! (packages, aliases/re-exports, public types/consts, module stores) through
//! the SAME producer chain as the 950 lane, observed on FOUR ends with
//! independent oracles — AV (value), native/mn (exit code), Rust (rustc exe
//! stdout) and C (cl exe stdout) — plus recovery from the packages root and
//! the a2x fragment stores.
//!
//! Oracles are independent expected values computed by hand from the source;
//! the ends are never compared against each other.
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_a2x_c::CTargetProvider;
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::contract::Value;
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use ace_source_query::modules::packages::{compile_package_product, PackageProduct};
use ace_source_query::modules::{PackageStore, PackageStoreReceipt};
use std::path::{Path, PathBuf};
use std::process::Command;

fn input(package: &str, id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: package.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
    }
}
fn caps() -> Vec<String> {
    vec![PACKAGES_CAPABILITY.into()]
}
fn compile(inputs: &[PackageModuleInput]) -> Result<PackageProduct, String> {
    compile_package_product(inputs, "app::main", &caps(), FrontBudget::WORK_OWNED_64MIB).map_err(
        |ds| {
            format!(
                "{}: {} {}",
                ds[0].module, ds[0].diagnostic.code, ds[0].diagnostic.message
            )
        },
    )
}
fn err(inputs: &[PackageModuleInput]) -> String {
    match compile(inputs) {
        Ok(_) => panic!("expected rejection"),
        Err(e) => e,
    }
}
fn scratch(tag: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/pkg960/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The four-end product: a public Str function in one package consumed by
/// the entry of another (text/record domain, no module globals — global
/// cells are AV/mn-side state; their a2x emission is a named follow-up).
/// main = base() + "m2".len() = 40 + 2 = 42.
fn four_end_inputs() -> Vec<PackageModuleInput> {
    vec![
        input(
            "lib",
            "std",
            "pub fn base() int { return 40 }\npub fn greet() str { return \"m2\" }\nfn hidden() int { return 1 }",
        ),
        input(
            "app",
            "main",
            "use lib.std.greet\nuse lib.std.base\nfn main() int { return base() + greet().len() }",
        ),
    ]
}

fn rust_value(files: &[(String, String)], dir: &Path, expected: i32) {
    std::fs::create_dir_all(dir).unwrap();
    for (path, content) in files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("pkg.exe");
    let mut cmd = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    cmd.args(["--edition", "2021", "main.rs", "-o"])
        .arg(&exe)
        .current_dir(dir);
    let compile = auto_tool_execution::run_command_capture(
        cmd,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .unwrap();
    assert_eq!(
        compile.kind,
        auto_tool_execution::ExitKind::Success,
        "{}",
        String::from_utf8_lossy(compile.stderr.as_deref().unwrap_or(&[]))
    );
    let mut cmd = Command::new(&exe);
    cmd.current_dir(dir);
    let run = auto_tool_execution::run_command_capture(
        cmd,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .unwrap();
    assert_eq!(run.kind, auto_tool_execution::ExitKind::Success);
    assert_eq!(
        String::from_utf8_lossy(run.stdout.as_deref().unwrap()).trim(),
        format!("value:{expected}")
    );
}

fn c_value(files: &[(String, String)], manifest: &str, dir: &Path, expected: i32) {
    std::fs::create_dir_all(dir).unwrap();
    for (path, content) in files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), manifest).unwrap();
    let exe = dir.join("pkg_c.exe");
    ace_a2x_worker::crun::compile_c_fileset(dir, &exe).unwrap_or_else(|e| panic!("cl failed: {e}"));
    let run = ace_a2x_worker::crun::run_exe(&exe).unwrap();
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).trim(),
        format!("value:{expected}")
    );
}

/// native/mn: `ac_start` forwards the entry's i32 to ExitProcess bit-exact.
fn native_value(product: &PackageProduct, dir: &Path, expected: i32) {
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let built = driver
        .build_checked(&product.compiled.checked, "d_main", dir, None, None, 960)
        .unwrap_or_else(|e| panic!("mn build: {e:?}"));
    let status = Command::new(&built.exe_path)
        .current_dir(dir)
        .status()
        .expect("run mn exe");
    assert_eq!(status.code(), Some(expected), "mn exit code");
}

#[test]
fn packages_960_four_ends_run_public_str_and_recover() {
    let inputs = four_end_inputs();
    let product = compile(&inputs).unwrap();
    // identity is input-order independent (V-05)
    let mut reordered = inputs.clone();
    reordered.reverse();
    assert_eq!(
        compile(&reordered).unwrap().bound.input_hash,
        product.bound.input_hash
    );

    // ---- AV: value + the Str function observed as a view ----
    let av_dir = scratch("av");
    let av = PackageAvTarget::cold(&av_dir, &product).unwrap();
    assert_eq!(av.run_value(), Ok(Value::I32(42)));
    let greet =
        PackageAvTarget::cold_with_entry(&av_dir, &product, "d_p3_libm3_std_greet").unwrap();
    assert!(matches!(greet.run_value(), Ok(Value::StrView(_))));

    // one shared managed store for the a2x fragments and the packages root
    // (required edges must resolve inside one store)
    let store = scratch("store");
    // ---- Rust: cold fileset → rustc → run; receipt → restore reuses fragments ----
    let rust_dir = scratch("rust");
    let mut rust = PackageA2xTarget::cold(
        &store,
        "pkg960",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &product,
    )
    .unwrap();
    rust_value(&rust.current.files, &rust_dir.join("build"), 42);
    let rust_receipt = rust.receipt().unwrap();
    let restored_rust = PackageA2xTarget::restore(
        &store,
        "pkg960",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &rust_receipt,
        &product,
    )
    .unwrap();
    assert_eq!(restored_rust.stats().lower, 0, "restore re-emits nothing");
    assert_eq!(restored_rust.stats().print, 0);
    assert_eq!(restored_rust.current.files, rust.current.files);

    // ---- C: cold fileset → cl → run; receipt → restore ----
    let c_dir = scratch("c");
    let mut c = PackageA2xTarget::cold(
        &store,
        "pkg960",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &product,
    )
    .unwrap();
    c_value(
        &c.current.files,
        &c.current.manifest,
        &c_dir.join("build"),
        42,
    );
    let c_receipt = c.receipt().unwrap();
    let restored_c = PackageA2xTarget::restore(
        &store,
        "pkg960",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &c_receipt,
        &product,
    )
    .unwrap();
    assert_eq!(restored_c.stats().lower, 0);
    assert_eq!(restored_c.current.files, c.current.files);

    // ---- native/mn: real exe, exit-code oracle ----
    native_value(&product, &scratch("mn"), 42);

    // ---- packages root: identity-only persistence, recovery re-admits ----
    let root_dir = store.clone();
    let targets = vec![
        (
            "a2x-rust".to_string(),
            PackageStoreReceipt {
                artifact: rust_receipt.artifact.clone(),
                digest: rust_receipt.digest.clone(),
            },
        ),
        (
            "a2x-c".to_string(),
            PackageStoreReceipt {
                artifact: c_receipt.artifact.clone(),
                digest: c_receipt.digest.clone(),
            },
        ),
    ];
    let receipt = PackageStore::prepare(&root_dir, &product, &inputs, &targets).unwrap();
    let (restored, restored_targets) = PackageStore::restore(&root_dir, &receipt).unwrap();
    assert_eq!(restored.bound.input_hash, product.bound.input_hash);
    assert_eq!(restored_targets.len(), 2);
    let av2 = PackageAvTarget::cold(&scratch("av2"), &restored).unwrap();
    assert_eq!(av2.run_value(), Ok(Value::I32(42)));
    let payload =
        String::from_utf8(PackageStore::root_payload(&root_dir, &receipt).unwrap()).unwrap();
    assert!(payload.contains("source-packages-root/1"));
    for forbidden in ["\"cells\"", "\"globals\"", "addr"] {
        assert!(
            !payload.contains(forbidden),
            "{forbidden} leaked into the root"
        );
    }
    // a tampered receipt never restores
    let bad = PackageStoreReceipt {
        artifact: receipt.artifact.clone(),
        digest: "0".repeat(64),
    };
    assert!(PackageStore::restore(&root_dir, &bad).is_err());
    // a foreign product never restores an a2x root
    let other = compile(&[
        input(
            "lib",
            "std",
            "pub fn base() int { return 41 }\npub fn greet() str { return \"m2\" }",
        ),
        input(
            "app",
            "main",
            "use lib.std.greet\nuse lib.std.base\nfn main() int { return base() + greet().len() }",
        ),
    ])
    .unwrap();
    assert!(PackageA2xTarget::restore(
        &store,
        "pkg960",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &rust_receipt,
        &other,
    )
    .is_err());
}

#[test]
fn packages_960_isolation_rejections_and_repair() {
    // V-01: same-name private symbols in two packages stay isolated.
    let p = compile(&[
        input(
            "one",
            "m",
            "fn helper() int { return 1 }\npub fn get() int { return helper() }",
        ),
        input(
            "two",
            "m",
            "fn helper() int { return 2 }\npub fn get() int { return helper() }",
        ),
        input(
            "app",
            "main",
            "fn main() int { return one.m.get() * 10 + two.m.get() }",
        ),
    ])
    .unwrap();
    let av = PackageAvTarget::cold(&scratch("v01"), &p).unwrap();
    assert_eq!(av.run_value(), Ok(Value::I32(12)));

    // V-02: re-exporting a private type is rejected by name.
    let e = err(&[
        input(
            "lib",
            "geo",
            "type Secret { x int }\npub fn f() int { return 1 }",
        ),
        input(
            "app",
            "main",
            "pub use lib.geo.Secret\nfn main() int { return 1 }",
        ),
    ]);
    assert!(e.contains("source.module-reexport-private"), "{e}");

    // V-04: a missing export is rejected at the binding gate and repaired
    // by adding the export; the repaired program runs.
    let mut inputs = vec![
        input("lib", "std", "pub fn answer() int { return 42 }"),
        input("app", "main", "fn main() int { return lib.std.missing() }"),
    ];
    let e = err(&inputs);
    assert!(e.contains("source.module-symbol"), "{e}");
    inputs[0]
        .source
        .push_str("\npub fn missing() int { return 99 }");
    let repaired = compile(&inputs).unwrap();
    assert_eq!(
        PackageAvTarget::cold(&scratch("v04"), &repaired)
            .unwrap()
            .run_value(),
        Ok(Value::I32(99))
    );

    // V-03: a dependency body update changes only the reader's observed
    // value; the unrelated unit keeps its source identity.
    inputs[0].source =
        "pub fn answer() int { return 42 }\npub fn missing() int { return 7 }".into();
    let updated = compile(&inputs).unwrap();
    assert_eq!(
        PackageAvTarget::cold(&scratch("v03"), &updated)
            .unwrap()
            .run_value(),
        Ok(Value::I32(7))
    );
    let seg = |p: &PackageProduct, m: &str| {
        p.bound
            .segments
            .iter()
            .find(|s| s.module == m)
            .map(|s| ace_core::runtime_contract::sha256_hex(s.source.as_bytes()))
            .unwrap()
    };
    assert_eq!(seg(&repaired, "main"), seg(&updated, "main"));
    assert_ne!(seg(&repaired, "std"), seg(&updated, "std"));

    // V-06: a value-initialization cycle is rejected by name and the
    // previously published program keeps running.
    let good = compile(&[input(
        "app",
        "main",
        "const A int = 2\nconst B int = A + 1\nfn main() int { return B }",
    )])
    .unwrap();
    let old = PackageAvTarget::cold(&scratch("v06"), &good).unwrap();
    assert_eq!(old.run_value(), Ok(Value::I32(3)));
    let e = err(&[input(
        "app",
        "main",
        "const A int = B + 1\nconst B int = A + 1\nfn main() int { return B }",
    )]);
    assert!(e.contains("source.module-init-cycle"), "{e}");
    assert_eq!(
        old.run_value(),
        Ok(Value::I32(3)),
        "old generation still runs"
    );
}

#[test]
fn packages_963_owned_generic_parameters_and_returns_cross_modules() {
    // PLAN-963（T-08，V-08「资源泛型跨模块参数/返回」）：模板 `Holder<T>`
    // 声明在 lib、以 **owned 文本**实参在 app 实例化；参数与返回都跨模块。
    // 手算 oracle：wrap("m2") 得到的 Holder<str> 其 get() 仍是同一文本 → len = 2。
    let inputs = vec![
        input(
            "lib",
            "std",
            "pub type Holder<T> { item T\n fn use_it() int { return .item.len() } }\npub fn wrap(move v Str) Holder<Str> { return Holder { item: v.move } }",
        ),
        input(
            "app",
            "main",
            "use lib.std.wrap\nuse lib.std.Holder\nfn main() int { let s Str = \"m2\"\n let h Holder<Str> = wrap(s.move)\n return h.use_it() }",
        ),
    ];
    let product = compile(&inputs).expect("owned generic instance across modules");
    let av_dir = scratch("av-963-owned");
    let av = PackageAvTarget::cold(&av_dir, &product).expect("AV cold admission");
    assert_eq!(av.run_value(), Ok(Value::I32(2)), "closed-form oracle");
}

#[test]
fn packages_960_assembly_whitelist_admits_native_and_c_producers() {
    use ace_semantics_contract::assembly::{
        Consumer, Manifest, SemanticAssembly, C_PRODUCER, NATIVE_PRODUCER,
    };
    for producer in [NATIVE_PRODUCER, C_PRODUCER] {
        let consumer = Consumer {
            producer: producer.into(),
            abi: "abi/1".into(),
            runtime: "rt/1".into(),
            toolchain: "tc/1".into(),
            providers: Vec::new(),
        };
        let semantic = SemanticAssembly {
            profile: PACKAGES_CAPABILITY.into(),
            host: "ace.explicit-finite-host/1".into(),
            platform: "x86_64-pc-windows-msvc".into(),
            declarations: Vec::new(),
            references: Vec::new(),
            sources: Vec::new(),
        };
        let m = Manifest::freeze(semantic, consumer).expect("freeze");
        assert!(m.validate().is_ok(), "{producer}");
    }
}
