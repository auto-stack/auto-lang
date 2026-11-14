//! PLAN-967 T-15 (AC-13 / V-15, with V-NATIVE + V-ABI-PACK legs): the
//! pinned REAL libraries — serde_json (Rust; a real cargo unit whose
//! ONLY dependency configuration is the provider's `dep` annotation)
//! and SQLite (C; the content-pinned amalgamation) — driven from a
//! CLEAN project (dependency + public use import only; the binding,
//! shim, manifest and build unit all auto-derive from those bytes) onto
//! real machine code through the AV lane. Independent oracles close the
//! values (in-process serde_json; a standalone cl-built SQLite exe);
//! the wrong-version / missing-library / illegal-parameter / cancel /
//! resource-exit negatives are each asserted on their own.
#![cfg(windows)]
use std::path::PathBuf;

use ace_abi::rust_provider::{
    build_c_dll_with, build_c_oracle, build_cargo_provider, c_authority, f0_authority,
    f0_json_provider_source, load, locate_pinned_sqlite, open_c_provider, p967_sqlite_header,
    p967_sqlite_source, provider_deps, sdk_include_dirs, stage_pinned_sqlite, verify_sqlite_pin,
    RawLibrary, RustProviderBuild,
};
use ace_av_core::contract::{ExecOutcome, TrapKind, Value};
use ace_av_core::foreign_exec::{ForeignCallable, ForeignStub, ForeignStubTable};

fn scope(tag: &str) -> PathBuf {
    let root = PathBuf::from("D:/autostack/plan-artifacts/plan-967/tests")
        .join(format!("{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    root
}

/// The delivered AV pipeline (frontdoor -> lower -> verify -> abc ->
/// execute) over an ordinary `use.rs`/`use.c` source; `cancel` pre-arms
/// the cancellation flag (the cooperative channel).
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
    let mut query = ace_source_query::SourceQuerySession::new("p967t15", "m967t15");
    query.set_foreign(Some(environment.clone()));
    let product = ace_source_query::compile_text_record_product(
        source, "f0t15", &caps, "p967t15", "m967t15", &mut query,
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

/// One Auto string literal carrying `json` (quote-escaped for the Auto
/// source, which whitelists `\"`).
fn auto_json_lit(json: &str) -> String {
    format!("\"{}\"", json.replace('"', "\\\""))
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

fn expect_foreign_trap(out: ExecOutcome, what: &str) {
    match out {
        ExecOutcome::Trap {
            kind: TrapKind::ForeignStatus,
            ..
        } => {}
        other => panic!("{what}: expected a ForeignStatus trap, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// The Rust leg: pinned serde_json through a real cargo unit.
// ---------------------------------------------------------------------------

#[test]
fn t15_serde_json_real_library_end_to_end() {
    let root = scope("t15-json");

    // The CLEAN project: the annotated provider file is the ONLY input
    // — its `dep` annotation is the whole dependency configuration.
    let proj = root.join("proj");
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::write(proj.join("f0_json.rs"), f0_json_provider_source()).unwrap();
    assert_eq!(
        provider_deps(f0_json_provider_source()),
        vec![("serde_json".to_string(), "=1.0.151".to_string())],
        "the pin comes from the provider's own dep annotation"
    );

    let build = build_cargo_provider(&root, f0_json_provider_source())
        .expect("the real-dependency cargo unit builds");
    // the pin is REAL: the unit's lock records exactly the pinned version
    let lock = std::fs::read_to_string(root.join("cargo-unit/Cargo.lock")).unwrap();
    assert!(
        lock.contains("serde_json") && lock.contains("1.0.151"),
        "locked pin: {lock}"
    );
    assert!(
        build.cdylib.is_file() && build.staticlib.is_file(),
        "both closure products"
    );

    // the 978 handover record (V-ABI-PACK): identity axes + artifacts
    let handover = serde_json::json!({
        "provider": build.manifest.provider,
        "provider_sha256": build.provider_digest,
        "deps": provider_deps(f0_json_provider_source()),
        "rustc": build.rustc_identity,
        "target": build.manifest.target,
        "cdylib": build.cdylib.display().to_string(),
        "staticlib": build.staticlib.display().to_string(),
        "runtime_note": "msvc-crt:VCRUNTIME140+ucrt (dynamic CRT)",
    });
    std::fs::write(
        root.join("t15-serde-json-buildrecord.json"),
        serde_json::to_string_pretty(&handover).unwrap(),
    )
    .unwrap();

    // ---- the AV lane: the ordinary use.rs source drives the REAL library
    let mut loaded = load(&build, 1).expect("the cdylib loads");
    let environment = build
        .manual
        .clone()
        .admit(&f0_authority(&build, 1))
        .expect("admits");

    // object length: the INDEPENDENT in-process serde_json oracle (a
    // different copy than the pinned provider unit's) computes the truth
    let good = r#"{"a":1,"b":2,"c":3}"#;
    let oracle_len = serde_json::from_str::<serde_json::Value>(good)
        .unwrap()
        .as_object()
        .unwrap()
        .len() as i32;
    expect_i32(
        run_use(
            &format!(
                "use.rs f0_json\nfn main() int {{ return json_object_len({}) }}",
                auto_json_lit(good)
            ),
            &mut loaded.table,
            &environment,
            false,
        ),
        oracle_len,
        "object length crosses with the oracle",
    );

    // a NON-object document returns the VALUE -1 (shape observable
    // without the failure channel)
    expect_i32(
        run_use(
            &format!("use.rs f0_json\nfn main() int {{ return json_object_len(\"[1,2]\") }}"),
            &mut loaded.table,
            &environment,
            false,
        ),
        -1,
        "non-object is a value, not a trap",
    );

    // bad JSON is the NAMED failure (trap channel — the discriminative
    // pair against the -1 value above)
    expect_foreign_trap(
        run_use(
            "use.rs f0_json\nfn main() int { return json_object_len(\"{oops\") }",
            &mut loaded.table,
            &environment,
            false,
        ),
        "bad JSON is the named bad-json trap",
    );

    // the from_str -> to_string round-trip (both public APIs) against
    // the oracle
    let round = r#"{"k":true}"#;
    let oracle_round = serde_json::from_str::<serde_json::Value>(round)
        .unwrap()
        .to_string()
        .len() as i32;
    expect_i32(
        run_use(
            &format!(
                "use.rs f0_json\nfn main() int {{ return json_roundtrip_len({}) }}",
                auto_json_lit(round)
            ),
            &mut loaded.table,
            &environment,
            false,
        ),
        oracle_round,
        "round-trip length matches the oracle",
    );

    // the Value surface: stable type ids (object 0 / number 3 / null 5 /
    // parse failure -1)
    for (doc, want) in [("{}", 0), ("5", 3), ("null", 5), ("nope", -1)] {
        expect_i32(
            run_use(
                &format!("use.rs f0_json\nfn main() int {{ return json_type_id(\"{doc}\") }}"),
                &mut loaded.table,
                &environment,
                false,
            ),
            want,
            "type id",
        );
    }
    assert!(loaded.table.callee_count() >= 8, "real dispatch happened");

    // ---- WRONG VERSION: a pin cargo cannot resolve is a NAMED failure
    // carrying cargo's own diagnostics (never a silent tool change)
    let bad_pin = f0_json_provider_source().replace("=1.0.151", "=9.9.9");
    let err = match build_cargo_provider(&root.join("wrong-ver"), &bad_pin) {
        Err(e) => e,
        Ok(_) => panic!("a pin cargo cannot resolve must fail"),
    };
    assert!(err.starts_with("f0.cargo-build:"), "named: {err}");
    assert!(
        err.contains("failed to select a version"),
        "cargo's own diagnostic: {err}"
    );

    // ---- CACHE REMOVAL REBUILD (V-15): delete the generated artifacts;
    // the SAME entry rebuilds and answers identically
    drop(loaded);
    std::fs::remove_dir_all(root.join("cargo-unit/target")).expect("cache removed");
    let build2 = build_cargo_provider(&root, f0_json_provider_source())
        .expect("rebuild after cache removal");
    assert!(build2.cdylib.is_file(), "the cdylib regenerated");
    let mut loaded2 = load(&build2, 1).expect("reload");
    expect_i32(
        run_use(
            &format!(
                "use.rs f0_json\nfn main() int {{ return json_object_len({}) }}",
                auto_json_lit(good)
            ),
            &mut loaded2.table,
            &environment,
            false,
        ),
        oracle_len,
        "the rebuilt provider answers identically",
    );

    // ---- ISOLATED CONSUMPTION (V-ABI-PACK): the dist root carries the
    // artifacts + record ONLY; the whole build tree is deleted; loading
    // and calling need no dev tree / rustc / cargo
    let dist = root.join("dist");
    std::fs::create_dir_all(&dist).unwrap();
    std::fs::copy(&build2.cdylib, dist.join("provider_unit.dll")).unwrap();
    std::fs::copy(&build2.staticlib, dist.join("provider_unit.lib")).unwrap();
    std::fs::copy(
        root.join("t15-serde-json-buildrecord.json"),
        dist.join("buildrecord.json"),
    )
    .unwrap();
    drop(loaded2);
    std::fs::remove_dir_all(root.join("cargo-unit")).expect("the dev build tree is gone");
    let dist_build = RustProviderBuild {
        cdylib: dist.join("provider_unit.dll"),
        staticlib: dist.join("provider_unit.lib"),
        manifest: build2.manifest.clone(),
        shim_source: build2.shim_source.clone(),
        manual: build2.manual.clone(),
        provider_digest: build2.provider_digest.clone(),
        rustc_identity: build2.rustc_identity.clone(),
    };
    let mut isolated = load(&dist_build, 1).expect("the isolated dist loads without the dev tree");
    expect_i32(
        run_use(
            &format!(
                "use.rs f0_json\nfn main() int {{ return json_object_len({}) }}",
                auto_json_lit(good)
            ),
            &mut isolated.table,
            &environment,
            false,
        ),
        oracle_len,
        "the isolated dist answers",
    );

    // ---- RESOURCE EXIT: dropping the provider frees the library; a
    // fresh open after exit keeps working (no corrupted state)
    drop(isolated);
    let mut reopened = load(&dist_build, 1).expect("reopen after exit");
    expect_i32(
        run_use(
            &format!(
                "use.rs f0_json\nfn main() int {{ return json_object_len({}) }}",
                auto_json_lit(good)
            ),
            &mut reopened.table,
            &environment,
            false,
        ),
        oracle_len,
        "reopen after resource exit",
    );
    drop(reopened);
}

/// The A2PY leg on the REAL library (the four-lane row): the same
/// cdylib through python ctypes — the emitted bridge is generated from
/// the same manifest, nothing is hand-written.
#[test]
fn t15_serde_json_through_python_ctypes() {
    let root = scope("t15-json-py");
    let build =
        build_cargo_provider(&root, f0_json_provider_source()).expect("the cargo unit builds");
    let authority = f0_authority(&build, 1);
    let environment = build.manual.clone().admit(&authority).unwrap();
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p967t15py", "m967t15py");
    query.set_foreign(Some(environment));
    let source = "use.rs f0_json\nfn main() int { return json_object_len(\"{\\\"a\\\":1,\\\"b\\\":2,\\\"c\\\":3}\") }";
    let product = ace_source_query::compile_text_record_product(
        source,
        "f0t15py",
        &caps,
        "p967t15py",
        "m967t15py",
        &mut query,
    )
    .expect("compiles");
    let view = ace_a2x_core::SemanticView::from_checked(&product.compiled.checked).unwrap();
    use ace_a2x_core::target::{Family, TargetProvider};
    let provider = ace_a2x_python::PythonTargetProvider::default();
    let family = Family::RecordText;
    let runtime = provider
        .runtime_source(&view, family)
        .expect("the ctypes bridge admits the frozen domain");
    assert!(runtime.contains("_A2X_F0_DLL"), "the bridge loads the DLL");
    let fns: Vec<&ace_a2x_core::FnFacts<'_>> = view.functions().iter().collect();
    let fragments = provider
        .produce_fragments(&view, &fns, family)
        .expect("lowers");
    let dir = root.join("py");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a2x_runtime.py"), &runtime).unwrap();
    let mut generated = String::from(
        "# AUTO-GENERATED (PLAN-967 T-15 A2PY) — DO NOT EDIT.\nfrom a2x_runtime import *\n\n",
    );
    for f in &fragments {
        generated.push_str(f);
        generated.push_str("\n\n");
    }
    generated.push_str("if __name__ == \"__main__\":\n    a2x_run_main(a2x_fn_main)\n");
    std::fs::write(dir.join("generated.py"), &generated).unwrap();
    // the provider DLL rides beside (ctypes CDLL resolves from CWD)
    std::fs::copy(&build.cdylib, dir.join("f0_text_provider.dll")).unwrap();
    let mut cmd = std::process::Command::new("python");
    cmd.arg("-B").arg("generated.py").current_dir(&dir);
    let run = ace_core::link::run_deadlined_typed(
        &mut cmd,
        std::time::Duration::from_secs(60),
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 16,
            stderr_max: 1 << 16,
        },
        vec![],
    )
    .expect("python spawn");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.exit_code, Some(0), "python ran: {combined}");
    assert!(
        combined.contains('3'),
        "the pinned serde_json answered through ctypes: {combined}"
    );
}

// ---------------------------------------------------------------------------
// The C leg: the content-pinned REAL SQLite amalgamation.
// ---------------------------------------------------------------------------

#[test]
fn t15_sqlite_real_library_reads_and_writes() {
    let root = scope("t15-sqlite");
    let Ok(sqlite_346) = locate_pinned_sqlite("3.46.0") else {
        // named pending, never a PASS (the environment prerequisite is
        // the pinned amalgamation in the local cargo registry)
        eprintln!("V-15 sqlite leg pending: f0.sqlite-missing — no pinned SQLite 3.46.0 in the cargo registry");
        return;
    };

    // ---- the wrong-version pin is a NAMED rejection (content-verified
    // against the amalgamation's own sqlite3.h, never a directory guess)
    match locate_pinned_sqlite("3.44.0") {
        Ok(other) => {
            let err = verify_sqlite_pin(&other, "3.46.0").unwrap_err();
            assert!(err.starts_with("f0.sqlite-pin:"), "named: {err}");
            assert!(
                err.contains("3.44.0"),
                "the actual content is in the message: {err}"
            );
        }
        Err(_) => {
            eprintln!("V-15 sqlite pin-mismatch negative pending: no 3.44.0 copy in the registry")
        }
    }
    verify_sqlite_pin(&sqlite_346, "3.46.0").expect("the staged pin matches by content");

    // ---- the MISSING LIBRARY negative first (named, on a project whose
    // dependency was never staged): the compile fails with cl's own
    // C1083 — a missing tool/library is never a silent skip
    let bare = root.join("bare-proj");
    std::fs::create_dir_all(&bare).unwrap();
    std::fs::write(bare.join("p967_sqlite.h"), p967_sqlite_header()).unwrap();
    std::fs::write(bare.join("p967_sqlite.c"), p967_sqlite_source()).unwrap();
    let err = match open_c_provider(&bare, "p967_sqlite") {
        Err(e) => e,
        Ok(_) => panic!("a project without the staged library must fail"),
    };
    assert!(err.contains("c-import.compile"), "named stage: {err}");
    assert!(
        err.contains("sqlite3.h"),
        "cl's own missing-include diagnostic: {err}"
    );

    // ---- the CLEAN project: the annotated header + impl + the staged
    // pinned amalgamation (the dependency), nothing else
    let proj = root.join("proj");
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::write(proj.join("p967_sqlite.h"), p967_sqlite_header()).unwrap();
    std::fs::write(proj.join("p967_sqlite.c"), p967_sqlite_source()).unwrap();
    stage_pinned_sqlite(&proj, &sqlite_346).expect("stages the pinned amalgamation");

    let provider =
        open_c_provider(&proj, "p967_sqlite").expect("the real-library provider imports");
    let dll = build_c_dll_with(
        &provider,
        &root,
        &[proj.join("sqlite3.c"), proj.join("sqlite3.h")],
        &sdk_include_dirs(),
    )
    .expect("the provider+amalgamation DLL builds");

    // ---- the INDEPENDENT ORACLE: a standalone exe built from the SAME
    // pinned amalgamation + a plain main() — a separate process sharing
    // no FFI machinery with the provider under test
    let oracle_src = r#"
#include "sqlite3.h"
#include <stdio.h>
int main(void) {
    sqlite3* db = 0;
    if (sqlite3_open(":memory:", &db) != SQLITE_OK) { printf("open-failed\n"); return 1; }
    char* err = 0;
    if (sqlite3_exec(db, "CREATE TABLE t(a INTEGER); INSERT INTO t VALUES (4); INSERT INTO t VALUES (5);", 0, 0, &err) != SQLITE_OK) {
        printf("exec-failed\n");
        return 1;
    }
    sqlite3_stmt* st = 0;
    sqlite3_prepare_v2(db, "SELECT SUM(a) FROM t", -1, &st, 0);
    sqlite3_step(st);
    printf("sum=%d changes=%d\n", sqlite3_column_int(st, 0), sqlite3_changes(db));
    sqlite3_finalize(st);
    sqlite3_close(db);
    return 0;
}
"#;
    let oracle_exe = build_c_oracle(
        &root,
        oracle_src,
        &[sqlite_346.join("sqlite3.c"), sqlite_346.join("sqlite3.h")],
    )
    .expect("the oracle builds");
    let mut cmd = std::process::Command::new(&oracle_exe);
    let run = ace_core::link::run_deadlined_typed(
        &mut cmd,
        std::time::Duration::from_secs(60),
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 16,
            stderr_max: 1 << 16,
        },
        vec![],
    )
    .expect("oracle spawn");
    assert_eq!(run.exit_code, Some(0));
    let oracle_out = String::from_utf8_lossy(&run.stdout).into_owned();
    let oracle_sum: i32 = oracle_out
        .lines()
        .find_map(|l| {
            l.strip_prefix("sum=")?
                .split_whitespace()
                .next()?
                .parse()
                .ok()
        })
        .expect("the oracle prints sum=");

    // ---- the AV lane: the same ordinary use.c source drives the REAL
    // SQLite machine code
    let library = RawLibrary::load(&dll).expect("the DLL loads");
    let authority = c_authority(&provider, 1);
    let environment = provider.manifest.clone().admit(&authority).expect("admits");
    let mut table = ForeignStubTable::new(1);
    for decl in environment.source_declarations_ref() {
        let symbol = decl.function.symbol.as_str();
        if !symbol.starts_with("p967_") {
            continue;
        }
        assert!(
            decl.is_fallible(),
            "{symbol}: every p967 entry is the fallible view+out frame"
        );
        let callable = unsafe {
            ForeignCallable::FallibleOutI32(std::mem::transmute::<
                _,
                unsafe extern "C" fn(*const u8, u64, *mut i32) -> i32,
            >(library.symbol(symbol).expect("export")))
        };
        table
            .insert(ForeignStub {
                provider: decl.provider.clone(),
                symbol: decl.function.symbol.clone(),
                content_version: decl.content_version.clone(),
                generation: 1,
                callable,
            })
            .unwrap();
    }
    assert_eq!(table.callee_count(), 0, "nothing dispatched yet");

    // the full open -> exec -> step -> close lifecycle on :memory:,
    // checked against the oracle
    expect_i32(
        run_use(
            "use.c p967_sqlite\nfn main() int {\n    let a = p967_open(\":memory:\")\n    let b = p967_exec(\"CREATE TABLE t(a INTEGER); INSERT INTO t VALUES (4); INSERT INTO t VALUES (5);\")\n    let c = p967_query_i32(\"SELECT SUM(a) FROM t\")\n    let d = p967_close(\"\")\n    return c\n}",
            &mut table,
            &environment,
            false,
        ),
        oracle_sum,
        "the AV lane's SUM matches the independent oracle",
    );
    assert!(table.callee_count() >= 4, "all four real calls dispatched");

    // ---- real FILE persistence: create/insert/close, then reopen and
    // read back the same sum (an actual disk round-trip)
    let db_path = root.join("t15.db");
    let db_lit = db_path.to_string_lossy().replace('\\', "/");
    expect_i32(
        run_use(
            &format!(
                "use.c p967_sqlite\nfn main() int {{\n    let a = p967_open(\"{db_lit}\")\n    let b = p967_exec(\"CREATE TABLE IF NOT EXISTS t(a INTEGER); INSERT INTO t VALUES (4); INSERT INTO t VALUES (5);\")\n    let c = p967_close(\"\")\n    return c\n}}"
            ),
            &mut table,
            &environment,
            false,
        ),
        1,
        "file create+insert+close",
    );
    assert!(db_path.is_file(), "the database file is on disk");
    expect_i32(
        run_use(
            &format!(
                "use.c p967_sqlite\nfn main() int {{\n    let a = p967_open(\"{db_lit}\")\n    let c = p967_query_i32(\"SELECT SUM(a) FROM t\")\n    let d = p967_close(\"\")\n    return c\n}}"
            ),
            &mut table,
            &environment,
            false,
        ),
        oracle_sum,
        "the reopened file answers the same sum",
    );

    // ---- the named negatives, each on its own run:
    // exec before any open (the connection is closed right now)
    expect_foreign_trap(
        run_use(
            "use.c p967_sqlite\nfn main() int { return p967_exec(\"SELECT 1\") }",
            &mut table,
            &environment,
            false,
        ),
        "exec without an open connection is the named not-open trap",
    );
    // illegal SQL
    expect_foreign_trap(
        run_use(
            "use.c p967_sqlite\nfn main() int {\n    let a = p967_open(\":memory:\")\n    let b = p967_exec(\"NOT SQL (\")\n    return 0\n}",
            &mut table,
            &environment,
            false,
        ),
        "illegal SQL is the named sql-error trap",
    );
    // double close
    expect_foreign_trap(
        run_use(
            "use.c p967_sqlite\nfn main() int {\n    let a = p967_open(\":memory:\")\n    let b = p967_close(\"\")\n    let c = p967_close(\"\")\n    return 0\n}",
            &mut table,
            &environment,
            false,
        ),
        "double close is the named not-open trap",
    );
    // an embedded NUL in the path — the V-NATIVE named rejection at a
    // REAL library boundary (never a silent C-string truncation)
    expect_foreign_trap(
        run_use(
            "use.c p967_sqlite\nfn main() int { return p967_open(\"a\\0b\") }",
            &mut table,
            &environment,
            false,
        ),
        "an embedded NUL path is the named nul-path trap",
    );
    // a path beyond the bounded read-back
    let overlong = "x".repeat(2000);
    expect_foreign_trap(
        run_use(
            &format!("use.c p967_sqlite\nfn main() int {{ return p967_open(\"{overlong}\") }}"),
            &mut table,
            &environment,
            false,
        ),
        "an over-long path is the named too-long trap",
    );

    // ---- CANCEL: a pre-armed flag aborts the run cooperatively
    // (Cancelled — a classification of its own, not a trap/resource)
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
    // ...and the provider is still coherent afterwards
    expect_i32(
        run_use(
            "use.c p967_sqlite\nfn main() int {\n    let a = p967_open(\":memory:\")\n    let c = p967_query_i32(\"SELECT 1+1\")\n    let d = p967_close(\"\")\n    return c\n}",
            &mut table,
            &environment,
            false,
        ),
        2,
        "a normal run after cancel still answers",
    );

    // ---- RESOURCE EXIT: dropping the library frees it exactly once
    // (原方 destroy 一次); a fresh load+call works after the exit
    drop(library);
    let library2 = RawLibrary::load(&dll).expect("reload after exit");
    let mut table2 = ForeignStubTable::new(1);
    for decl in environment.source_declarations_ref() {
        let symbol = decl.function.symbol.as_str();
        if !symbol.starts_with("p967_") {
            continue;
        }
        let callable = unsafe {
            ForeignCallable::FallibleOutI32(std::mem::transmute::<
                _,
                unsafe extern "C" fn(*const u8, u64, *mut i32) -> i32,
            >(library2.symbol(symbol).expect("export")))
        };
        table2
            .insert(ForeignStub {
                provider: decl.provider.clone(),
                symbol: decl.function.symbol.clone(),
                content_version: decl.content_version.clone(),
                generation: 1,
                callable,
            })
            .unwrap();
    }
    expect_i32(
        run_use(
            "use.c p967_sqlite\nfn main() int {\n    let a = p967_open(\":memory:\")\n    let c = p967_query_i32(\"SELECT 6*7\")\n    let d = p967_close(\"\")\n    return c\n}",
            &mut table2,
            &environment,
            false,
        ),
        42,
        "the reloaded library answers",
    );
    drop(library2);

    // the 978 handover record for the C real library
    let handover = serde_json::json!({
        "provider": provider.manifest.provider,
        "header_sha256": provider.provider_digest,
        "sqlite_pin": "3.46.0",
        "sqlite_source": "cargo registry libsqlite3-sys (content-pinned via SQLITE_VERSION)",
        "cdylib": dll.display().to_string(),
        "exports": provider.manifest.facts.functions.iter().map(|f| f.symbol.clone()).collect::<Vec<_>>(),
        "runtime_note": "msvc-crt:LIBCMT+ucrt+um (static CRT in-DLL)",
    });
    std::fs::write(
        root.join("t15-sqlite-buildrecord.json"),
        serde_json::to_string_pretty(&handover).unwrap(),
    )
    .unwrap();
}
