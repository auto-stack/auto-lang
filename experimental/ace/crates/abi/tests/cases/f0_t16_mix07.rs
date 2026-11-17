//! PLAN-985 T-08[A] / MIX-07 (AC-06 / V-08 / V-09): the ONE-PROJECT
//! composition across the DELIVERED A lanes —
//!   platform file selection (the 1043 suffix assembly with
//!   TargetOs::Windows picks the `.rs.at` implementation)
//!   -> the selected implementation's REAL machine code reads the
//!   project's config file and parses it through the pinned serde_json
//!   (=1.0.151) public API (`use.rs`)
//!   -> the parsed value flows into the `use.c` SQLite lane (the pinned
//!   3.46.0 amalgamation): written, aggregated and read back
//!   -> the cooperative cancel channel aborts a polling variant and the
//!   project answers identically afterwards.
//! Independent oracle: an in-process serde_json copy (a different copy
//! than the provider unit's pinned dependency) computes the config
//! truth; the SQL aggregate is deterministic arithmetic over that
//! truth (the p967 lane's own C-oracle equality is the t15 owner
//! evidence). V-08 negatives: deleting the selected implementation
//! (`load.no-provider-impl`), a corrupt config document (parse failure
//! is the VALUE -1, never a silent green) and a missing config path
//! (-8) are each NAMED. The owned/read-frame axis binds the delivered
//! `out_frame_1064` AV OutBytes evidence (this case keeps the delivered
//! scalar frames); MIX-07's off-repo install/retire re-check stays B
//! (986/987).
#![cfg(windows)]

use std::path::PathBuf;

use ace_abi::rust_provider::{
    build_c_dll_with, c_authority, compile_c_unit, locate_pinned_sqlite,
    open_assembled_rs_provider_session, open_c_provider, p967_sqlite_header, p967_sqlite_source,
    sdk_include_dirs, stage_pinned_sqlite, RawLibrary,
};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::foreign_exec::{ForeignCallable, ForeignStub, ForeignStubTable};
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, ProviderKind, TargetOs,
};

/// T-05 F3: transient scratch under the OS temp dir, removed on drop
/// (test working dirs are not plan evidence).
struct Scope(PathBuf);
impl std::ops::Deref for Scope {
    type Target = std::path::Path;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn scope(tag: &str) -> Scope {
    let root = std::env::temp_dir().join(format!("p985-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    Scope(root)
}

/// The platform-selected `.rs.at` implementation: the ONLY dependency
/// configuration is its own `dep` annotation (the same clean-project
/// rule as the t15 owner case).
const RS_IMPL: &str = r#"//! auto-ffi provider mix07/1 target=x86_64-pc-windows-msvc profile=mix07.v1
// auto-ffi dep serde_json =1.0.151
// auto-ffi fn mix07_cfg_sum symbol=mix07.cfg.sum.v1 form=pure ret=i32
pub fn mix07_cfg_sum(path: &str) -> i32 {
    let Ok(text) = std::fs::read_to_string(path) else { return -8 };
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(&text) else { return -1 };
    let serde_json::Value::Object(map) = doc else { return -1 };
    let mut sum: i64 = 0;
    for v in map.values() {
        if let Some(i) = v.as_i64() {
            sum += i;
        }
    }
    i32::try_from(sum).unwrap_or(-1)
}
"#;

/// The public Auto adapter (`mix07.at`): `use.rs mix07` driving the
/// selected implementation over the project's config document.
fn adapter(cfg_lit: &str) -> String {
    format!("use.rs mix07\nfn main() int {{ return mix07_cfg_sum(\"{cfg_lit}\") }}")
}

/// A polling adapter variant (the cancel channel needs a loop).
fn polling_adapter(cfg_lit: &str) -> String {
    format!(
        "use.rs mix07\nfn main() int {{\n    var x = 0\n    for 0 < 1 {{\n        x = mix07_cfg_sum(\"{cfg_lit}\")\n    }}\n    return x\n}}"
    )
}

/// The assembled-session AV lane (the load1043 runner shape).
fn run_session(
    source: &str,
    session: &mut ace_abi::rust_provider::F0Session,
    cancel: bool,
) -> ExecOutcome {
    let environment = session.admitted.clone();
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p985mx", "m985mx");
    query.set_foreign(Some(environment));
    let product = ace_source_query::compile_text_record_product(
        source, "mix07", &caps, "p985mx", "m985mx", &mut query,
    )
    .expect("compiles");
    let mir = ace_lowering::hir_adapter::lower_checked_owned(
        &product.compiled.checked,
        "d_main",
        ace_av_core::contract::fnv1a64(source.as_bytes()),
    )
    .expect("lowers");
    ace_av_core::mir_verify::verify_mir(&mir).expect("mir");
    let encoded = ace_av_core::abc::encode_abc(&mir).unwrap().to_flat();
    let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
    let loaded = ace_av_core::abc_verify::verify_abc_foreign(
        decoded,
        &mir.owner,
        session.session.table_shared(),
    )
    .expect("admits");
    session
        .session
        .execute(
            &loaded,
            &ace_av_core::av::ExecLimits::default(),
            &ace_av_core::av::CancelFlag::new(cancel),
        )
        .expect("authorized run")
        .0
}

/// The table-backed AV lane (the t15 runner shape) for `use.c` sources.
fn run_use(
    source: &str,
    table: &mut ForeignStubTable,
    environment: &ace_foreign_contract::manual::AdmittedForeign,
    cancel: bool,
) -> ExecOutcome {
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p985mx", "m985mx");
    query.set_foreign(Some(environment.clone()));
    let product = ace_source_query::compile_text_record_product(
        source, "mix07", &caps, "p985mx", "m985mx", &mut query,
    )
    .expect("compiles");
    let mir = ace_lowering::hir_adapter::lower_checked_owned(
        &product.compiled.checked,
        "d_main",
        ace_av_core::contract::fnv1a64(source.as_bytes()),
    )
    .expect("lowers");
    ace_av_core::mir_verify::verify_mir(&mir).expect("mir");
    let encoded = ace_av_core::abc::encode_abc(&mir).unwrap().to_flat();
    let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
    let loaded =
        ace_av_core::abc_verify::verify_abc_foreign(decoded, &mir.owner, table).expect("admits");
    ace_av_core::av::execute_foreign(
        &loaded,
        &ace_av_core::av::ExecLimits::default(),
        &ace_av_core::av::CancelFlag::new(cancel),
        table,
    )
    .expect("authorized run")
    .0
}

fn expect_i32(out: ExecOutcome, want: i32, what: &str) {
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => assert_eq!(v, want, "{what}"),
        other => panic!("{what}: expected Success(I32({want})), got {other:?}"),
    }
}

fn assembled_ctx() -> PackageAssemblyContext {
    PackageAssemblyContext::new(
        AssemblyBackend::Av,
        ProviderChoice::Kind(ProviderKind::Rs),
        TargetOs::Windows,
    )
}

#[test]
fn t16_mix07_one_project_platform_config_rust_sqlite_cancel() {
    let root = scope("t16-mix07");

    // ---- THE ONE PROJECT: the public adapter, the platform-selected
    // `.rs.at` implementation and the config document side by side.
    let proj = root.join("proj");
    std::fs::create_dir_all(&proj).unwrap();
    let cfg = r#"{"rows":3,"base":6}"#;
    let cfg_path = proj.join("config.json");
    std::fs::write(&cfg_path, cfg).unwrap();
    let cfg_lit = cfg_path.to_string_lossy().replace('\\', "/");
    let ad = adapter(&cfg_lit);
    std::fs::write(proj.join("mix07.at"), &ad).unwrap();
    std::fs::write(proj.join("mix07.rs.at"), RS_IMPL).unwrap();

    // ---- platform file selection: the Windows/AV configuration
    // selects the `.rs.at` implementation; the public adapter is the
    // composed face (the delivered 1043 platform-file form).
    let (mut session, adapter_out) =
        open_assembled_rs_provider_session(&proj, "app::mix07", &mut assembled_ctx(), 1)
            .expect("the assembled provider session opens");
    assert_eq!(adapter_out, ad, "the public adapter is the composed face");

    // ---- config read + Rust parse through REAL machine code, checked
    // against the INDEPENDENT in-process serde_json oracle (a different
    // copy than the provider unit's pinned dependency).
    let oracle_n: i32 = {
        let doc = serde_json::from_str::<serde_json::Value>(cfg).unwrap();
        doc.as_object()
            .unwrap()
            .values()
            .map(|v| v.as_i64().unwrap_or(0))
            .sum::<i64>() as i32
    };
    assert_eq!(oracle_n, 9, "the oracle truth is rows+base");
    let n = match run_session(&ad, &mut session, false) {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("config parse: expected Success(I32), got {other:?}"),
    };
    assert_eq!(n, oracle_n, "the config value crosses with the oracle");
    assert!(
        session.session.table_shared().callee_count() >= 1,
        "real dispatch happened"
    );

    // ---- the value flows into the `use.c` SQLite write lane: the REAL
    // parsed value n is written (n and 2n) and the aggregate read back
    // (= 3n) — the project's data chain, not a parallel constant.
    if let Ok(sqlite_346) = locate_pinned_sqlite("3.46.0") {
        let sproj = root.join("sqlite-proj");
        std::fs::create_dir_all(&sproj).unwrap();
        std::fs::write(sproj.join("p967_sqlite.h"), p967_sqlite_header()).unwrap();
        std::fs::write(sproj.join("p967_sqlite.c"), p967_sqlite_source()).unwrap();
        stage_pinned_sqlite(&sproj, &sqlite_346).expect("stages the pinned amalgamation");
        let provider = open_c_provider(&sproj, "p967_sqlite").expect("imports");
        let sqlite_obj = compile_c_unit(&root, &sproj.join("sqlite3.c"))
            .expect("the amalgamation compiles once");
        let dll = build_c_dll_with(
            &provider,
            &root,
            &[sqlite_obj, sproj.join("sqlite3.h")],
            &sdk_include_dirs(),
        )
        .expect("the provider+amalgamation DLL builds");
        let library = RawLibrary::load(&dll).expect("the DLL loads");
        let authority = c_authority(&provider, 1);
        let environment = provider.manifest.clone().admit(&authority).expect("admits");
        let mut table = ForeignStubTable::new(1);
        for decl in environment.source_declarations_ref() {
            let symbol = decl.function.symbol.as_str();
            if !symbol.starts_with("p967_") {
                continue;
            }
            let callable = unsafe {
                ForeignCallable::FallibleOutI32(std::mem::transmute::<
                    _,
                    unsafe extern "C" fn(*const u8, u64, *mut i32) -> i32,
                >(
                    library.symbol(symbol).expect("export")
                ))
            };
            table
                .insert(ForeignStub {
                    provider: decl.provider.clone(),
                    symbol: decl.function.symbol.clone(),
                    content_version: decl.content_version.clone(),
                    generation: 1,
                    callable,
                    frame: false,
                })
                .unwrap();
        }

        let sql_src = format!(
            "use.c p967_sqlite\nfn main() int {{\n    let a = p967_open(\":memory:\")\n    let b = p967_exec(\"CREATE TABLE mix(a INTEGER); INSERT INTO mix VALUES ({n}); INSERT INTO mix VALUES ({dbl});\")\n    let c = p967_query_i32(\"SELECT SUM(a) FROM mix\")\n    let d = p967_close(\"\")\n    return c\n}}",
            dbl = 2 * n
        );
        expect_i32(
            run_use(&sql_src, &mut table, &environment, false),
            3 * oracle_n,
            "the flowed config value is written and aggregated (3n)",
        );
        assert!(table.callee_count() >= 4, "all four real calls dispatched");

        // ---- CANCEL on the SQLite lane: a pre-armed flag aborts a
        // polling variant cooperatively; the project answers identically
        // afterwards (coherence).
        let cancelled = run_use(
            "use.c p967_sqlite\nfn main() int {\n    var x = 0\n    for 0 < 1 {\n        x = p967_open(\":memory:\")\n    }\n    return x\n}",
            &mut table,
            &environment,
            true,
        );
        assert_eq!(
            cancelled,
            ExecOutcome::Cancelled,
            "the cancel channel fires"
        );
        expect_i32(
            run_use(&sql_src, &mut table, &environment, false),
            3 * oracle_n,
            "a normal run after cancel still answers",
        );
    } else {
        eprintln!(
            "MIX-07 sqlite leg pending: f0.sqlite-missing — no pinned SQLite 3.46.0 in the cargo registry"
        );
    }

    // ---- CANCEL on the assembled Rust lane (independent of sqlite
    // availability).
    let cancelled_rs = run_session(&polling_adapter(&cfg_lit), &mut session, true);
    assert_eq!(
        cancelled_rs,
        ExecOutcome::Cancelled,
        "the cancel channel fires on the assembled lane"
    );

    // ---- V-08 negative (a): deleting the selected implementation is
    // the NAMED rejection (a missing MIX input is never green).
    drop(session);
    std::fs::remove_file(proj.join("mix07.rs.at")).expect("the implementation leaves");
    let err = match open_assembled_rs_provider_session(&proj, "app::mix07", &mut assembled_ctx(), 2)
    {
        Err(e) => e,
        Ok(_) => panic!("a project without its implementation must reject"),
    };
    assert!(err.contains("load.no-provider-impl"), "named: {err}");

    // ---- recovery: the implementation returns and the SAME entry
    // answers the config value again.
    std::fs::write(proj.join("mix07.rs.at"), RS_IMPL).unwrap();
    let (mut session3, _) =
        open_assembled_rs_provider_session(&proj, "app::mix07", &mut assembled_ctx(), 3)
            .expect("the restored project rebuilds");
    expect_i32(
        run_session(&ad, &mut session3, false),
        oracle_n,
        "the restored project answers the config value",
    );

    // ---- V-08 negative (b): a CORRUPT config document makes the parse
    // failure the VALUE -1 (named, never a silent green); restoring the
    // document recovers the value.
    std::fs::write(&cfg_path, "{\"rows\":}").unwrap();
    expect_i32(
        run_session(&ad, &mut session3, false),
        -1,
        "a corrupt config is the named parse-failure value",
    );
    std::fs::write(&cfg_path, cfg).unwrap();
    expect_i32(
        run_session(&ad, &mut session3, false),
        oracle_n,
        "the restored config recovers",
    );

    // ---- V-08 negative (c): a missing config path is the named read
    // failure (-8).
    let missing = adapter(
        &proj
            .join("no-such-config.json")
            .to_string_lossy()
            .replace('\\', "/"),
    );
    expect_i32(
        run_session(&missing, &mut session3, false),
        -8,
        "a missing config path is the named read-failure value",
    );
}
