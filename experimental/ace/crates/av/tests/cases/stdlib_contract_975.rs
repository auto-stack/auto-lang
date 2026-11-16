//! PLAN-975 contract closure tests:
//! - Record-typed + generic `<T has JsonDecodable>` `decode<T>` and 966 CT/metadata
//!   (`ct_field_count` / `ct_has_field`) integration with canonical `json.at`.
//! - `V-LOAD-MODULE` canonical `json.at` / `redis.at` boundary (private helper isolation,
//!   `pub`-stripped mirror RED, `std::*` reservation, `PackageStore::restore`).
//! - `V-COVERAGE` canonical `json.at` business roundtrip across all 5 packages backends:
//!   AV, AC/native (`MnDriver`), A2R (`rustc`), A2C (`cl`), and A2PY (real OS Python).
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_a2x_c::CTargetProvider;
use ace_a2x_python::PythonTargetProvider;
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::modules::packages::PackageA2xTarget;
use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value, AV_DATA_PROFILE};
use ace_av_core::{abc_verify, decode_abc, encode_abc};
use ace_core::source::cst::{FrontBudget, CT_CAPABILITY, INSTANCES_CAPABILITY};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use ace_source_cte::{run_entry_on_av, CtService, MemoryEnvelopeSink};
use ace_source_query::modules::packages::{compile_package_product, PackageProduct};
use ace_source_query::modules::{PackageStore, PackageStoreReceipt};
use ace_source_query::{compile_record_product, CompileLane, SourceQuerySession};
use std::path::PathBuf;
use std::process::Command;

const STDLIB_JSON: &str = include_str!("../../../../../../stdlib/ace/std/json.at");
const STDLIB_REDIS: &str = include_str!("../../../../../../stdlib/ace/std/redis.at");

fn scratch(tag: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../target/p975c/{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn pkg_input(package: &str, id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: package.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn compile_pkg(inputs: &[PackageModuleInput], entry: &str) -> Result<PackageProduct, Vec<String>> {
    compile_package_product(
        inputs,
        entry,
        &[PACKAGES_CAPABILITY.to_string()],
        FrontBudget::WORK_OWNED_64MIB,
    )
    .map_err(|ds| {
        ds.into_iter()
            .map(|d| {
                format!(
                    "{}: {} {}",
                    d.module, d.diagnostic.code, d.diagnostic.message
                )
            })
            .collect()
    })
}

fn compile_rec(src: &str) -> Result<ace_source_query::ProductCompile, Vec<String>> {
    let mut session = SourceQuerySession::new("plan975", "typed_json.at");
    compile_record_product(
        src,
        "typed_json.at",
        &[INSTANCES_CAPABILITY.to_string()],
        "plan975",
        "typed_json.at",
        &mut session,
    )
    .map_err(|ds| {
        ds.into_iter()
            .map(|d| format!("{} @{}-{}: {}", d.code, d.span.start, d.span.end, d.message))
            .collect()
    })
}

fn strip_pub_fn(src: &str) -> String {
    src.lines()
        .map(|line| {
            if let Some(rest) = line.strip_prefix("pub fn ") {
                format!("fn {rest}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

const COVERAGE_MAIN: &str = r#"use app.json.json_is_valid
use app.json.json_kind_at
use app.json.json_parse_str
use app.json.json_parse_int
use app.json.json_parse_bool
use app.json.json_object_open
use app.json.json_object_close
use app.json.json_kv
use app.json.json_str
use app.json.json_int
fn main() int {
    let doc str = "{\"services\":[{\"name\":\"ClearDiagnosticInformation\",\"port\":8080,\"active\":true}],\"grid\":[[1,2],[4,5]],\"emoji\":\"\\uD83D\\uDE00\",\"lone\":\"\\uD800\"}"
    if json_is_valid(doc) != true { return 1 }
    if json_kind_at(doc, "services.0.name") != 4 { return 2 }
    if json_kind_at(doc, "missing") != 0 { return 3 }
    let svc Str = json_parse_str(doc, "services.0.name")
    let want Str = "ClearDiagnosticInformation"
    if svc != want { return 4 }
    if json_parse_int(doc, "services.0.port") != 8080 { return 5 }
    if json_parse_bool(doc, "services.0.active") != 1 { return 6 }
    if json_parse_int(doc, "grid.1.0") != 4 { return 7 }
    let em Str = json_parse_str(doc, "emoji")
    let want_em Str = "😀"
    if em != want_em { return 8 }
    let lone Str = json_parse_str(doc, "lone")
    if lone.len_bytes() != 0 { return 9 }
    let bad str = "{\"bad\":\"\\q\"}"
    if json_is_valid(bad) != false { return 10 }
    let k_name Str = "name"
    let k_port Str = "port"
    let comma Str = ","
    let k1 Str = json_kv(k_name, json_str(svc))
    let k2 Str = json_kv(k_port, json_int(8080))
    let enc Str = json_object_open() + k1 + comma + k2 + json_object_close()
    let want_enc Str = "{\"name\":\"ClearDiagnosticInformation\",\"port\":8080}"
    if enc != want_enc { return 11 }
    return 42
}"#;

fn coverage_product() -> PackageProduct {
    let inputs = vec![
        pkg_input("app", "json", STDLIB_JSON),
        pkg_input("app", "main", COVERAGE_MAIN),
    ];
    compile_pkg(&inputs, "app::main").unwrap_or_else(|e| panic!("coverage compile: {e:#?}"))
}

/// Generic `<T has JsonDecodable>` `decode<T>` + 966 `ct_field_count`/`ct_has_field`.
#[test]
fn json975_typed_decode_and_ct_metadata() {
    let ok_src = r#"spec JsonDecodable {
    fn decode_score(raw int) int
}
type ServiceModel as JsonDecodable {
    bonus int
    fn decode_score(raw int) int {
        return .bonus + raw
    }
}
type CounterModel as JsonDecodable {
    scale int
    fn decode_score(raw int) int {
        return .scale * raw
    }
}
fn decode<T has JsonDecodable>(seed T, raw int) int {
    return seed.decode_score(raw)
}
fn main() int {
    let s ServiceModel = ServiceModel { bonus: 10 }
    let c CounterModel = CounterModel { scale: 3 }
    return decode(s, 80) + decode(c, 4)
}"#;
    let p = compile_rec(ok_src).unwrap_or_else(|ds| panic!("typed decode compile: {ds:#?}"));
    assert_eq!(p.lane, CompileLane::Instances);
    assert_eq!(
        p.instances.len(),
        2,
        "two concrete decode<T> instantiations (ServiceModel & CounterModel): {:?}",
        p.instances
    );

    let mir = ace_lowering::lower_checked_data(&p.compiled.checked, "d_main", 975)
        .expect("lower record data");
    assert_eq!(mir.profile, AV_DATA_PROFILE);
    ace_av_core::verify_mir(&mir).expect("MIR verifies");
    let wire = encode_abc(&mir).expect("ABC encode").to_flat();
    let decoded = decode_abc(&wire).expect("ABC decode");
    let loaded = abc_verify::verify_abc(decoded, &mir.owner).expect("ABC admission");
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(102),
                ..
            }
        ),
        "hand oracle (10 + 80) + (3 * 4) = 102, got {out:?}"
    );

    // Negative control 1: unbound record type rejected with source.instances-bound.
    let unbound_src = r#"spec JsonDecodable { fn decode_score(raw int) int }
type Plain { x int }
fn decode<T has JsonDecodable>(seed T, raw int) int { return seed.decode_score(raw) }
fn main() int {
    let p Plain = Plain { x: 1 }
    return decode(p, 1)
}"#;
    let ds = compile_rec(unbound_src)
        .err()
        .expect("unbound type must be rejected");
    assert!(
        ds.iter()
            .any(|d| d.contains("source.instances-bound") && d.contains("JsonDecodable")),
        "expected source.instances-bound naming JsonDecodable, got {ds:#?}"
    );

    // Negative control 2: declared conformance missing required spec member.
    let missing_member_src = r#"spec JsonDecodable { fn decode_score(raw int) int }
type Incomplete as JsonDecodable { x int }
fn decode<T has JsonDecodable>(seed T, raw int) int { return seed.decode_score(raw) }
fn main() int {
    let b Incomplete = Incomplete { x: 1 }
    return decode(b, 1)
}"#;
    let ds2 = compile_rec(missing_member_src)
        .err()
        .expect("missing spec method must be rejected");
    assert!(
        ds2.iter().any(|d| d.contains("decode_score")),
        "expected diagnostic naming decode_score, got {ds2:#?}"
    );

    // 966 CT/metadata contract check paired with JSON record schema fields.
    let ct_src = r#"type ServiceMeta {
    name int
    port int
    active int
}
fn pack(fc int, has_port bool, has_missing bool) int {
    var n = fc * 10
    if has_port { n = n + 1 }
    if has_missing { n = n + 100 }
    return n
}
fn main() int {
    return #{ pack(ct_field_count(ServiceMeta), ct_has_field(ServiceMeta, "port"), ct_has_field(ServiceMeta, "missing")) }
}"#;
    let mut svc = CtService::new(MemoryEnvelopeSink::new());
    let caps = vec![
        CT_CAPABILITY.to_string(),
        INSTANCES_CAPABILITY.to_string(),
        "cte.meta.fields/v1".to_string(),
    ];
    let ct_prod =
        ace_source_cte::compile_scalar_ct(&mut svc, ct_src, "meta975.at", &caps, "plan975", "meta")
            .expect("966 ct meta compile");
    let (val, _) = run_entry_on_av(
        &ct_prod.checked,
        "d_main",
        975,
        &ExecLimits::default(),
        &CancelFlag::new(false),
    )
    .expect("966 ct meta AV run");
    assert_eq!(
        val,
        Value::I32(31),
        "3 fields * 10 + port(1) + missing(0) = 31"
    );

    // CT metadata negative control: without cte.meta.fields/v1 capability must fail.
    let mut svc2 = CtService::new(MemoryEnvelopeSink::new());
    let no_meta_caps = vec![CT_CAPABILITY.to_string(), INSTANCES_CAPABILITY.to_string()];
    assert!(
        ace_source_cte::compile_scalar_ct(
            &mut svc2,
            ct_src,
            "meta975.at",
            &no_meta_caps,
            "plan975",
            "meta",
        )
        .is_err(),
        "missing cte.meta.fields/v1 capability must reject ct_field_count/ct_has_field"
    );
}

/// V-LOAD-MODULE closure for canonical `json.at` and `redis.at`.
#[test]
fn json975_v_load_module_closure() {
    let svc_mod = r#"use app.json.json_parse_str
use app.json.json_parse_int
use app.json.json_object_open
use app.json.json_object_close
use app.json.json_kv
use app.json.json_str
use app.json.json_int
pub type Service {
    name Str
    port int
}
pub fn decode_service(doc str, name_path str, port_path str) Service {
    let n Str = json_parse_str(doc, name_path)
    let p int = json_parse_int(doc, port_path)
    return Service { name: n.move, port: p }
}
pub fn encode_service(s Service) Str {
    let k_name Str = "name"
    let k_port Str = "port"
    let comma Str = ","
    let blank Str = ""
    let sname Str = blank + s.name
    let k1 Str = json_kv(k_name, json_str(sname))
    let k2 Str = json_kv(k_port, json_int(s.port))
    return json_object_open() + k1 + comma + k2 + json_object_close()
}"#;
    let main_mod = r#"use app.service.Service
use app.service.decode_service
use app.service.encode_service
fn main() int {
    let doc str = "{\"services\":[{\"name\":\"ClearDiagnosticInformation\",\"port\":8080}]}"
    let s Service = decode_service(doc, "services.0.name", "services.0.port")
    let enc Str = encode_service(s)
    let want Str = "{\"name\":\"ClearDiagnosticInformation\",\"port\":8080}"
    if enc != want { return -1 }
    return s.port + s.name.len()
}"#;
    let inputs = vec![
        pkg_input("app", "json", STDLIB_JSON),
        pkg_input("app", "service", svc_mod),
        pkg_input("app", "main", main_mod),
    ];
    let prod = compile_pkg(&inputs, "app::main").expect("V-LOAD-MODULE business compile");
    let av = PackageAvTarget::cold(&scratch("vlm-av"), &prod).expect("AV cold");
    assert_eq!(
        av.run_value(),
        Ok(Value::I32(8106)),
        "8080 + \"ClearDiagnosticInformation\".len()(26) = 8106"
    );

    // Negative 1: private helper in json.at (`json_cp`) must reject cross-module import/call.
    let priv_inputs = vec![
        pkg_input("app", "json", STDLIB_JSON),
        pkg_input(
            "app",
            "main",
            "use app.json.json_cp\nfn main() int { return json_cp(\"a\", 0) }",
        ),
    ];
    let priv_err = compile_pkg(&priv_inputs, "app::main")
        .err()
        .expect("private helper import must reject");
    assert!(
        priv_err.iter().any(|d| d.contains("source.module-private")),
        "expected source.module-private for json_cp, got {priv_err:#?}"
    );

    // Negative 2: pub-stripped json.at mirror RED.
    let stripped_json = strip_pub_fn(STDLIB_JSON);
    let stripped_json_err = compile_pkg(
        &[
            pkg_input("app", "json", &stripped_json),
            pkg_input(
                "app",
                "main",
                "use app.json.json_int\nfn main() int { return json_int(1).len() }",
            ),
        ],
        "app::main",
    )
    .err()
    .expect("pub-stripped json.at must reject");
    assert!(
        stripped_json_err
            .iter()
            .any(|d| d.contains("source.module-private")),
        "expected source.module-private on pub-stripped json.at, got {stripped_json_err:#?}"
    );

    // Negative 3: pub-stripped redis.at mirror RED.
    let stripped_redis = strip_pub_fn(STDLIB_REDIS);
    let stripped_redis_err = compile_pkg(
        &[
            pkg_input("app", "redis", &stripped_redis),
            pkg_input(
                "app",
                "main",
                "use app.redis.redis_cmd_ping\nfn main() int { return redis_cmd_ping().len() }",
            ),
        ],
        "app::main",
    )
    .err()
    .expect("pub-stripped redis.at must reject");
    assert!(
        stripped_redis_err
            .iter()
            .any(|d| d.contains("source.module-private")),
        "expected source.module-private on pub-stripped redis.at, got {stripped_redis_err:#?}"
    );

    // Negative 4: forged std::json package fails ace.stdlib-reserved.
    let forged_err = compile_pkg(
        &[
            pkg_input(
                "std",
                "json",
                "pub fn json_encode_i32(x int) Str { return \"0\" }",
            ),
            pkg_input(
                "app",
                "main",
                "use std.json.json_encode_i32\nfn main() int { return json_encode_i32(1).len() }",
            ),
        ],
        "app::main",
    )
    .err()
    .expect("user std::json forgery must reject");
    assert!(
        forged_err.iter().any(|d| d.contains("ace.stdlib-reserved")),
        "expected ace.stdlib-reserved, got {forged_err:#?}"
    );

    // PackageStore prepare / restore + tampered receipt rejection + private-helper hash drift.
    let root_dir = scratch("vlm-store");
    let receipt = PackageStore::prepare(&root_dir, &prod, &inputs, &[]).expect("store prepare");
    let (restored, _) = PackageStore::restore(&root_dir, &receipt).expect("store restore");
    assert_eq!(restored.bound.input_hash, prod.bound.input_hash);
    let bad_receipt = PackageStoreReceipt {
        artifact: receipt.artifact.clone(),
        digest: "0".repeat(64),
    };
    assert!(
        PackageStore::restore(&root_dir, &bad_receipt).is_err(),
        "tampered receipt must reject"
    );

    let mutated_json = STDLIB_JSON.replacen(
        "fn json_cp(s str, i int) int {",
        "fn json_cp(s str, i int) int {\n    let _p = 0",
        1,
    );
    assert_ne!(mutated_json, STDLIB_JSON);
    let mutated_prod = compile_pkg(
        &[
            pkg_input("app", "json", &mutated_json),
            pkg_input("app", "service", svc_mod),
            pkg_input("app", "main", main_mod),
        ],
        "app::main",
    )
    .expect("mutated private helper compiles");
    assert_ne!(
        mutated_prod.bound.input_hash, prod.bound.input_hash,
        "private-helper body edit must invalidate package input_hash"
    );
}

/// V-COVERAGE leg 1: canonical `json.at` via packages on AV, native/mn, and A2R (`rustc`).
#[test]
fn json975_packages_coverage_av_mn_a2r() {
    let prod = coverage_product();

    // 1. AV
    let av = PackageAvTarget::cold(&scratch("cov-av"), &prod).expect("AV cold");
    assert_eq!(av.run_value(), Ok(Value::I32(42)), "AV json.at oracle 42");

    // 2. native/mn
    let mn_dir = scratch("cov-mn");
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let built = driver
        .build_checked(&prod.compiled.checked, "d_main", &mn_dir, None, None, 975)
        .unwrap_or_else(|e| panic!("mn build: {e:?}"));
    let status = Command::new(&built.exe_path)
        .current_dir(&mn_dir)
        .status()
        .expect("run mn exe");
    assert_eq!(status.code(), Some(42), "native/mn json.at oracle 42");

    // 3. A2R (rustc)
    let store = scratch("cov-a2r-store");
    let rust = PackageA2xTarget::cold(
        &store,
        "p975cov",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &prod,
    )
    .expect("A2R cold");
    let rdir = scratch("cov-a2r-build");
    for (path, content) in &rust.current.files {
        std::fs::write(rdir.join(path), content).unwrap();
    }
    let exe = rdir.join("p975cov_rs.exe");
    let mut cmd = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    cmd.args(["--edition", "2021", "main.rs", "-o"])
        .arg(&exe)
        .current_dir(&rdir);
    let comp = auto_tool_execution::run_command_capture(
        cmd,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .unwrap();
    assert_eq!(
        comp.kind,
        auto_tool_execution::ExitKind::Success,
        "rustc stderr: {}",
        String::from_utf8_lossy(comp.stderr.as_deref().unwrap_or(&[]))
    );
    let mut rcmd = Command::new(&exe);
    rcmd.current_dir(&rdir);
    let run = auto_tool_execution::run_command_capture(
        rcmd,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .unwrap();
    assert_eq!(run.kind, auto_tool_execution::ExitKind::Success);
    assert_eq!(
        String::from_utf8_lossy(run.stdout.as_deref().unwrap()).trim(),
        "value:42",
        "A2R json.at oracle 42"
    );
}

/// V-COVERAGE leg 2a: canonical `json.at` via packages on A2C (`cl`).
#[test]
fn json975_packages_coverage_a2c() {
    let prod = coverage_product();
    let store = scratch("cov-c-store");

    let c = PackageA2xTarget::cold(
        &store,
        "p975covc",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &prod,
    )
    .expect("A2C cold");
    let cdir = scratch("cov-a2c-build");
    for (path, content) in &c.current.files {
        std::fs::write(cdir.join(path), content).unwrap();
    }
    std::fs::write(cdir.join("manifest.json"), &c.current.manifest).unwrap();
    let cexe = cdir.join("p975cov_c.exe");
    ace_a2x_worker::crun::compile_c_fileset(&cdir, &cexe).expect("cl compile json.at");
    let crun = ace_a2x_worker::crun::run_exe(&cexe).expect("run cl json.at exe");
    assert_eq!(
        String::from_utf8_lossy(&crun.stdout).trim(),
        "value:42",
        "A2C json.at oracle 42"
    );
}

/// V-COVERAGE leg 2b: canonical `json.at` via packages on A2PY (OS Python).
#[test]
fn json975_packages_coverage_a2py() {
    let prod = coverage_product();
    let store = scratch("cov-py-store");

    let py = PackageA2xTarget::cold(
        &store,
        "p975covpy",
        "app",
        16 << 20,
        PythonTargetProvider,
        &prod,
    )
    .expect("A2PY cold");
    let pydir = scratch("cov-a2py-build");
    for (path, content) in &py.current.files {
        std::fs::write(pydir.join(path), content).unwrap();
    }
    let rt = ace_a2x_python::python_runtime_source(&[]);
    let rt_hash = ace_core::runtime_contract::sha256_hex(rt.as_bytes());
    let tool = ace_a2x_worker::pyrun::resolve_python(&rt_hash)
        .expect("OS Python required (missing Python cannot PASS)");
    let pyrun = ace_a2x_worker::pyrun::run_py_fileset(&tool, &pydir, &[]).expect("run py fileset");
    assert_eq!(
        pyrun.exit_code,
        Some(0),
        "py stderr={}",
        String::from_utf8_lossy(&pyrun.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&pyrun.stdout).trim(),
        "value:42",
        "A2PY json.at oracle 42"
    );
}
