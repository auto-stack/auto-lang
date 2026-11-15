//! PLAN-975 T-02 — the sqlite `:memory:` leg on the 967 pinned-provider
//! lane (S2 selection, T-01 freeze): the SAME content-pinned SQLite
//! 3.46.0 amalgamation provider, opened with the `:memory:` path — no
//! host file, no 973 dependency. Independent oracle: the SUM is
//! computed by hand (4+5=9) AND cross-checked against a standalone
//! cl-built exe from the same amalgamation running the SAME statements
//! in its own private :memory: database. Negatives: not-open exec
//! refusal, double close. The Auto-side API face (sqlite.* in Auto) is
//! a NAMED follow-up (console-lane foreign dispatch seam).
#![cfg(windows)]
use std::path::PathBuf;

use ace_abi::rust_provider::{
    build_c_dll_with, build_c_oracle, compile_c_unit, locate_pinned_sqlite, open_c_provider,
    p967_sqlite_header, p967_sqlite_source, sdk_include_dirs, stage_pinned_sqlite,
    verify_sqlite_pin, RawLibrary,
};

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
