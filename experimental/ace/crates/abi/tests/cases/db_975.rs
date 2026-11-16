//! PLAN-975 T-02 — the sqlite `:memory:` leg on the 967 pinned-provider
//! lane (S2 selection, T-01 freeze): the SAME content-pinned SQLite
//! 3.46.0 amalgamation provider, opened with the `:memory:` path — no
//! host file, no 973 dependency. Independent oracle: the SUM is
//! computed by hand (4+5=9) AND cross-checked against a standalone
//! cl-built exe from the same amalgamation running the SAME statements
//! in its own private :memory: database. Negatives: not-open exec
//! refusal, double close.
//!
//! The Auto-side API face (the second case): the tracked
//! `stdlib/ace/std/sqlite.at` module (verbatim, PLAN-1041 inline)
//! driving the REAL pinned-provider DLL through the delivered AV
//! foreign lane (`use.c` admission + the stub table). The console
//! serving lane has no foreign dispatch today — the named seam; it
//! follows 973's io assembly.
#![cfg(windows)]
use std::path::PathBuf;

use ace_abi::rust_provider::{
    build_c_dll_with, build_c_oracle, c_authority, compile_c_unit, locate_pinned_sqlite,
    open_c_provider, p967_sqlite_header, p967_sqlite_source, sdk_include_dirs, stage_pinned_sqlite,
    verify_sqlite_pin, RawLibrary,
};
use ace_av_core::contract::{ExecOutcome, TrapKind};
use ace_av_core::foreign_exec::{ForeignCallable, ForeignStub, ForeignStubTable};

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
    let root = std::env::temp_dir().join(format!("p975-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    Scope(root)
}

/// fallible-status raw call helper (the F0 physical frame):
/// (out-ptr path, len) -> Err(raw_status) | Ok(out_value)
fn call_view(
    lib: &RawLibrary,
    name: &str,
    arg: &[u8],
) -> Result<i32, i32> {
    let ptr = lib.symbol(name).expect("symbol");
    let f: extern "C" fn(*const u8, u64, *mut i32) -> i32 = unsafe { std::mem::transmute(ptr) };
    let mut out = 0i32;
    let status = f(arg.as_ptr(), arg.len() as u64, &mut out);
    if status != 0 {
        Err(status)
    } else {
        Ok(out)
    }
}

fn call_ack(lib: &RawLibrary, name: &str) -> Result<i32, i32> {
    let ptr = lib.symbol(name).expect("symbol");
    let f: extern "C" fn(*const u8, u64, *mut i32) -> i32 = unsafe { std::mem::transmute(ptr) };
    let mut out = 0i32;
    let ack = b"ack";
    let status = f(ack.as_ptr(), ack.len() as u64, &mut out);
    if status != 0 {
        Err(status)
    } else {
        Ok(out)
    }
}

#[test]
fn db975_sqlite_memory_leg_on_pinned_provider() {
    let root = scope("db-mem");
    let Ok(sqlite_346) = locate_pinned_sqlite("3.46.0") else {
        eprintln!(
            "p975 sqlite :memory: leg pending: f0.sqlite-missing — no pinned SQLite 3.46.0 in the cargo registry"
        );
        return;
    };
    verify_sqlite_pin(&sqlite_346, "3.46.0").expect("the staged pin matches by content");

    // the clean provider project (dep + provider only)
    let proj = root.join("proj");
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::write(proj.join("p967_sqlite.h"), p967_sqlite_header()).unwrap();
    std::fs::write(proj.join("p967_sqlite.c"), p967_sqlite_source()).unwrap();
    stage_pinned_sqlite(&proj, &sqlite_346).expect("stages the pinned amalgamation");
    let provider = open_c_provider(&proj, "p967_sqlite").expect("imports");
    let sqlite_obj =
        compile_c_unit(&root, &proj.join("sqlite3.c")).expect("the amalgamation compiles once");
    let dll = build_c_dll_with(
        &provider,
        &root,
        &[sqlite_obj.clone(), proj.join("sqlite3.h")],
        &sdk_include_dirs(),
    )
    .expect("the provider+amalgamation DLL builds");
    let lib = RawLibrary::load(&dll).expect("the DLL loads");

    // ---- negatives FIRST: exec before open = named -1 (not-open)
    let exec_before = call_view(&lib, "p967_exec", b"SELECT 1");
    assert_eq!(exec_before, Err(-1), "exec before open -> not-open");

    // ---- the :memory: roundtrip (hand oracle 4+5=9)
    let mem = b":memory:";
    let opened = call_view(&lib, "p967_open", mem);
    assert_eq!(opened, Ok(1), ":memory: open succeeds");
    let created = call_view(
        &lib,
        "p967_exec",
        b"CREATE TABLE t(a INTEGER); INSERT INTO t VALUES (4); INSERT INTO t VALUES (5);",
    );
    // sqlite3_changes counts the MOST RECENT statement only: the second
    // INSERT changed 1 row (the CREATE before it changed 0)
    assert_eq!(created, Ok(1), "changes = last statement's rows (1)");
    let summed = call_view(&lib, "p967_query_i32", b"SELECT SUM(a) FROM t");
    assert_eq!(summed, Ok(9), "hand oracle: SUM(4,5) = 9");
    let closed = call_ack(&lib, "p967_close");
    // the close fn's `out` view carries its own status word (1 = closed)
    assert_eq!(closed, Ok(1), "close acks with out=1");
    // double close = named -1 (not-open)
    let double = call_ack(&lib, "p967_close");
    assert_eq!(double, Err(-1), "double close -> not-open");

    // ---- INDEPENDENT oracle: a standalone exe from the same pinned
    // amalgamation, its OWN :memory: database, same statements
    let oracle_src = r#"
#include "sqlite3.h"
#include <stdio.h>
int main(void) {
    sqlite3* db = 0;
    if (sqlite3_open(":memory:", &db) != SQLITE_OK) { return 1; }
    sqlite3_exec(db, "CREATE TABLE t(a INTEGER); INSERT INTO t VALUES (4); INSERT INTO t VALUES (5);", 0, 0, 0);
    sqlite3_stmt* st = 0;
    sqlite3_prepare_v2(db, "SELECT SUM(a) FROM t", -1, &st, 0);
    sqlite3_step(st);
    printf("sum=%d\n", sqlite3_column_int(st, 0));
    sqlite3_finalize(st);
    sqlite3_close(db);
    return 0;
}
"#;
    let oracle_exe = build_c_oracle(
        &root,
        oracle_src,
        &[sqlite_obj, sqlite_346.join("sqlite3.h")],
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
    let out = String::from_utf8_lossy(&run.stdout).into_owned();
    assert!(out.contains("sum=9"), "the independent oracle agrees: {out}");
}

// ---- the Auto-side sqlite.* API face: `stdlib/ace/std/sqlite.at`
// (verbatim) over the REAL pinned-provider DLL through the AV foreign
// lane — the delivered `use.c` admission + stub-table dispatch. ----

/// The delivered AV foreign pipeline (the 967 T-15 shape): text-record
/// product with the admitted environment → lower → verify → abc →
/// admit → execute against the stub table.
fn run_p975(
    source: &str,
    table: &mut ForeignStubTable,
    environment: &ace_foreign_contract::manual::AdmittedForeign,
) -> ExecOutcome {
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p975auto", "m975auto");
    query.set_foreign(Some(environment.clone()));
    let product = ace_source_query::compile_text_record_product(
        source, "p975auto", &caps, "p975auto", "m975auto", &mut query,
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
        &ace_av_core::av::CancelFlag::new(false),
        table,
    )
    .expect("authorized run")
    .0
}

#[test]
fn db975_auto_sqlite_api_face_on_av_foreign_lane() {
    let root = scope("db-auto");
    let Ok(sqlite_346) = locate_pinned_sqlite("3.46.0") else {
        eprintln!(
            "p975 sqlite auto face pending: f0.sqlite-missing — no pinned SQLite 3.46.0 in the cargo registry"
        );
        return;
    };
    verify_sqlite_pin(&sqlite_346, "3.46.0").expect("the staged pin matches by content");

    // the clean provider project + the once-compiled amalgamation object
    let proj = root.join("proj");
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::write(proj.join("p967_sqlite.h"), p967_sqlite_header()).unwrap();
    std::fs::write(proj.join("p967_sqlite.c"), p967_sqlite_source()).unwrap();
    stage_pinned_sqlite(&proj, &sqlite_346).expect("stages the pinned amalgamation");
    let provider = open_c_provider(&proj, "p967_sqlite").expect("imports");
    let sqlite_obj =
        compile_c_unit(&root, &proj.join("sqlite3.c")).expect("the amalgamation compiles once");
    let dll = build_c_dll_with(
        &provider,
        &root,
        &[sqlite_obj, proj.join("sqlite3.h")],
        &sdk_include_dirs(),
    )
    .expect("the provider+amalgamation DLL builds");
    let lib = RawLibrary::load(&dll).expect("the DLL loads");

    // admit + bind: every frozen p967 frame is a fallible view frame
    let authority = c_authority(&provider, 1);
    let environment = provider.manifest.clone().admit(&authority).expect("admits");
    let mut table = ForeignStubTable::new(1);
    for decl in environment.source_declarations_ref() {
        let symbol = decl.function.symbol.as_str();
        if !symbol.starts_with("p967_") || !decl.is_fallible() {
            continue;
        }
        let callable = unsafe {
            ForeignCallable::FallibleOutI32(std::mem::transmute::<
                _,
                unsafe extern "C" fn(*const u8, u64, *mut i32) -> i32,
            >(lib.symbol(symbol).expect("export")))
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
    assert_eq!(table.entries().len(), 4, "the four frozen frames are bound");
    assert_eq!(table.callee_count(), 0, "nothing dispatched yet");

    // the tracked stdlib module, VERBATIM, on the foreign lane
    let stdlib_sqlite = include_str!("../../../../../../stdlib/ace/std/sqlite.at");

    // ---- negative 1 FIRST (g_db == 0 on a fresh DLL): exec before
    // open traps with the lane's named ForeignStatus (not-open)
    let neg1 = run_p975(
        &format!(
            r#"use.c p967_sqlite
{stdlib_sqlite}
fn main() int {{ return sqlite_exec("SELECT 1") }}"#
        ),
        &mut table,
        &environment,
    );
    assert!(
        matches!(
            &neg1,
            ExecOutcome::Trap {
                kind: TrapKind::ForeignStatus,
                ..
            }
        ),
        "exec before open -> ForeignStatus trap, got {neg1:?}"
    );

    // ---- the wrapper face roundtrip: open/exec/query/close with the
    // hand oracle SUM(4,5)=9; every wrapper return checked in-source
    let ok = run_p975(
        &format!(
            r#"use.c p967_sqlite
{stdlib_sqlite}
fn main() int {{
    let o = sqlite_open_memory()
    if o != 1 {{ return 100 + o }}
    let c = sqlite_exec("CREATE TABLE t(a INTEGER); INSERT INTO t VALUES (4); INSERT INTO t VALUES (5);")
    if c != 1 {{ return 200 + c }}
    let s = sqlite_query_int("SELECT SUM(a) FROM t")
    if s != 9 {{ return 300 + s }}
    let n = sqlite_query_int("SELECT COUNT(*) FROM t")
    if n != 2 {{ return 400 + n }}
    let cl = sqlite_close()
    if cl != 1 {{ return 500 + cl }}
    return s
}}"#
        ),
        &mut table,
        &environment,
    );
    assert!(
        matches!(
            &ok,
            ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(9),
                ..
            }
        ),
        "the sqlite_* wrapper face roundtrip returns the hand oracle 9, got {ok:?}"
    );

    // ---- negative 2: double close traps (not-open after a real close)
    let neg2 = run_p975(
        &format!(
            r#"use.c p967_sqlite
{stdlib_sqlite}
fn main() int {{
    let o = sqlite_open_memory()
    if o != 1 {{ return 1 }}
    let a = sqlite_close()
    if a != 1 {{ return 2 }}
    let b = sqlite_close()
    return b
}}"#
        ),
        &mut table,
        &environment,
    );
    assert!(
        matches!(
            &neg2,
            ExecOutcome::Trap {
                kind: TrapKind::ForeignStatus,
                ..
            }
        ),
        "double close -> ForeignStatus trap, got {neg2:?}"
    );
    assert_eq!(
        table.callee_count(),
        9,
        "real dispatches: 1 (neg1 exec) + 5 (open/exec/query/query/close) + 3 (neg2 open/close/close)"
    );

    // ---- the seam negative: WITHOUT `use.c p967_sqlite` the wrapper's
    // provider calls must be a NAMED compile rejection — the sqlite_*
    // face has no builtin resolution (no silent host fallback)
    let mut query = ace_source_query::SourceQuerySession::new("p975neg", "m975neg");
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let rejected = match ace_source_query::compile_text_record_product(
        &format!(
            r#"{stdlib_sqlite}
fn main() int {{ return sqlite_open_memory() }}"#
        ),
        "p975neg",
        &caps,
        "p975neg",
        "m975neg",
        &mut query,
    ) {
        Ok(_) => panic!("no provider admission -> the wrapper must not compile"),
        Err(ds) => ds,
    };
    assert!(
        !rejected.is_empty() && rejected.iter().all(|d| !d.code.is_empty()),
        "a named diagnostic, got {rejected:?}"
    );
}
