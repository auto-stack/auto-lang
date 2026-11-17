//! PLAN-973 V-COVERAGE (AV×C / AV×Rust): a system-library PRIMITIVE —
//! `sys973_path_attrs(path) -> i32` (the path_attrs_bytes contract: bit0
//! = directory, negative on query failure) — dispatched through REAL
//! foreign machine code on the AV interpreter:
//!   * AV×C: a cl-built C DLL calling `GetFileAttributesA` (real OS call)
//!   * AV×Rust: a rustc-built cdylib calling `std::fs::metadata`
//! Both bind through the frozen 967 scalar-view frame
//! (`unsafe extern "C" fn(*const u8, u64) -> i32`); the READ-surface
//! (out-buffer frames) remains the NAMED 967 lane gap (no such frame is
//! delivered) — this leg covers the write-path-shaped primitive family
//! only and says so.
#![cfg(windows)]

use ace_abi::rust_provider::RawLibrary;
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::foreign_exec::{ForeignCallable, ForeignStub, ForeignStubTable};

struct Scope(std::path::PathBuf);
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
    let root = std::env::temp_dir().join(format!("p973-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    Scope(root)
}

const RUST_PROVIDER: &str = r#"//! auto-ffi provider sys973_rust/1 target=x86_64-pc-windows-msvc profile=f0-text.v1
//!
//! PLAN-973 AV-Rust foreign leg: a system-library primitive (the
//! path_attrs_bytes contract — bit0 = directory, negatives on failure)
//! authored as a frozen-annotation Rust provider. build_source derives
//! the manifest/shim — no hand-written manifest.

use std::fs;

/// Directory bit for a real path: 1 = dir, 0 = file, -8 = query failure.
// auto-ffi fn sys973_path_attrs symbol=p973.sys.path_attrs.v1 form=pure ret=i32
pub fn sys973_path_attrs(s: &str) -> i32 {
    match fs::metadata(s) {
        Ok(md) if md.is_dir() => 1,
        Ok(_) => 0,
        Err(_) => -8,
    }
}
"#;

fn run_av(
    source: &str,
    table: &mut ForeignStubTable,
    environment: &ace_foreign_contract::manual::AdmittedForeign,
) -> ExecOutcome {
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p973avf", "m973avf");
    query.set_foreign(Some(environment.clone()));
    let product = ace_source_query::compile_text_record_product(
        source, "p973avf", &caps, "p973avf", "m973avf", &mut query,
    )
    .unwrap_or_else(|ds| {
        eprintln!(
            "AV SRC PANIC:
{}
END",
            source
        );
        panic!("compiles: {ds:?}")
    });
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

fn expect_i32(out: ExecOutcome, want: i32, label: &str) {
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => assert_eq!(v, want, "{label}"),
        other => panic!("{label}: unexpected {other:?}"),
    }
}

fn bind_stub(
    table: &mut ForeignStubTable,
    library: &RawLibrary,
    symbol: &str,
    callable: ForeignCallable,
) {
    table
        .insert(ForeignStub {
            provider: "sys973-foreign-demo".into(),
            symbol: symbol.into(),
            content_version: "1".into(),
            generation: 1,
            callable,
            frame: false,
        })
        .unwrap();
}

/// AV×Rust: the sys primitive through a REAL rustc cdylib.
#[test]
fn sys973_av_rust_foreign_dispatch() {
    let root = scope("av-rust");
    // the REAL 967 pipeline: annotations -> manifest -> shim -> cdylib
    let build = ace_abi::rust_provider::build_source(&root, RUST_PROVIDER)
        .expect("the Rust provider builds");
    let authority = ace_abi::rust_provider::f0_authority(&build, 1);
    let environment = build.manual.clone().admit(&authority).expect("admits");
    let library = RawLibrary::load(&build.cdylib).expect("the cdylib loads");
    let mut table = ForeignStubTable::new(1);
    for decl in environment.source_declarations_ref() {
        let export = &decl.function.symbol;
        let _ = export;
        // the F0 pure scalar frame: view -> i32
        // the generated shim exports under the derived export name
        let export = build
            .manifest
            .functions
            .iter()
            .find(|f| f.rust_name == "sys973_path_attrs")
            .map(|f| f.export.clone())
            .expect("provider function present in manifest");
        let raw = library.symbol(&export).expect("shim export present");
        let callable = unsafe {
            ForeignCallable::PureI32(std::mem::transmute::<
                _,
                unsafe extern "C" fn(*const u8, u64) -> i32,
            >(raw))
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

    // a REAL directory through the Rust provider: bits = 1
    let dir_arg = {
        let t = std::env::temp_dir().to_string_lossy().to_string();
        t.trim_end_matches(char::from(92))
            .trim_end_matches('/')
            .replace(char::from(92), "/")
    };
    let src =
        format!("use.rs sys973_rust\nfn main() int {{ return sys973_path_attrs(\"{dir_arg}\") }}");
    let out = run_av(&src, &mut table, &environment);
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(1),
                ..
            }
        ),
        "AV Rust dir bit: {out:?}"
    );

    // a REAL file (this test's own source): bits = 0
    let file_arg = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cases/sys973_av_foreign.rs")
        .to_string_lossy()
        .replace(char::from(92), "/");
    let src =
        format!("use.rs sys973_rust\nfn main() int {{ return sys973_path_attrs(\"{file_arg}\") }}");
    let out = run_av(&src, &mut table, &environment);
    expect_i32(out, 0, "AV Rust file bits");

    // a query failure surfaces as the OS-failed sentinel
    let src = "use.rs sys973_rust\nfn main() int { return sys973_path_attrs(\"Z:/definitely/missing/973\") }";
    let out = run_av(src, &mut table, &environment);
    expect_i32(out, -8, "AV Rust missing-path sentinel");
}

/// AV×C: the same sys primitive through a REAL cl-built C DLL calling
/// `GetFileAttributesA` (public kernel32 entry — the same OS boundary the
/// Win32 backend uses; zero Rust on this leg).
#[test]
fn sys973_av_c_foreign_dispatch() {
    let root = scope("av-c");
    // the REAL 967 C pipeline: annotated header + C source -> cl import
    // -> manifest -> DLL (GetFileAttributesA is the same public kernel32
    // boundary the Win32 backend uses; zero Rust on this leg).
    let dir = root.join("proj");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("sys973_c.h"),
        r##"// sys973_c.h — PLAN-973 AV-C foreign leg (annotations ARE the import
// metadata; the 967 preprocessor import derives the manifest).
// auto-ffi-c provider sys973_c/1 target=x86_64-pc-windows-msvc
#ifndef SYS973_C_H
#define SYS973_C_H
#include <stdint.h>
// windows.h lives in the SDK include closure; the freestanding header
// keeps only the primitive types so the /EP import (MSVC includes only)
// stays self-contained — the .c unit includes windows.h itself.

// auto-ffi-c fn sys973_path_attrs form=pure ret=i32
int32_t sys973_path_attrs(const uint8_t* data, uint64_t len);

#endif
"##,
    )
    .unwrap();
    std::fs::write(
        dir.join("sys973_c.c"),
        r##"#include "sys973_c.h"

/* The freestanding provider unit keeps MSVC-only includes; the public
   kernel32 entry is declared directly (the same DLL boundary the Win32
   backend binds — no CRT/SDK header dependency in the import closure). */
__declspec(dllimport) unsigned long __stdcall GetFileAttributesA(const char* path);

#define SYS973_INVALID_FILE_ATTRIBUTES 0xFFFFFFFFul
#define SYS973_FILE_ATTRIBUTE_DIRECTORY 0x10ul

int32_t sys973_path_attrs(const uint8_t* data, uint64_t len) {
    char buf[4096];
    if (len == 0 || len >= sizeof(buf)) return -4;
    for (uint64_t i = 0; i < len; i++) buf[i] = (char)data[i];
    buf[len] = 0;
    unsigned long attrs = GetFileAttributesA(buf);
    if (attrs == SYS973_INVALID_FILE_ATTRIBUTES) return -8;
    return (attrs & SYS973_FILE_ATTRIBUTE_DIRECTORY) ? 1 : 0;
}
"##,
    )
    .unwrap();

    let provider =
        ace_abi::rust_provider::open_c_provider(&dir, "sys973_c").expect("the C provider imports");
    let dll =
        ace_abi::rust_provider::build_c_dll(&provider, &root).expect("the provider DLL builds");
    let library = RawLibrary::load(&dll).expect("the DLL loads");
    let authority = ace_abi::rust_provider::c_authority(&provider, 1);
    let environment = provider.manifest.clone().admit(&authority).expect("admits");
    let mut table = ForeignStubTable::new(1);
    for decl in environment.source_declarations_ref() {
        let raw = library
            .symbol("sys973_path_attrs")
            .expect("C export present");
        let callable = unsafe {
            ForeignCallable::PureI32(std::mem::transmute::<
                _,
                unsafe extern "C" fn(*const u8, u64) -> i32,
            >(raw))
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

    let dir_arg = {
        let t = std::env::temp_dir().to_string_lossy().to_string();
        t.trim_end_matches(char::from(92))
            .trim_end_matches('/')
            .replace(char::from(92), "/")
    };
    let src =
        format!("use.c sys973_c\nfn main() int {{ return sys973_path_attrs(\"{dir_arg}\") }}");
    let out = run_av(&src, &mut table, &environment);
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(1),
                ..
            }
        ),
        "AV C dir bit: {out:?}"
    );

    let file_arg = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cases/sys973_av_foreign.rs")
        .to_string_lossy()
        .replace(char::from(92), "/");
    let src =
        format!("use.c sys973_c\nfn main() int {{ return sys973_path_attrs(\"{file_arg}\") }}");
    let out = run_av(&src, &mut table, &environment);
    expect_i32(out, 0, "AV C file bits");

    let src =
        "use.c sys973_c\nfn main() int { return sys973_path_attrs(\"Z:/definitely/missing/973\") }";
    let out = run_av(src, &mut table, &environment);
    expect_i32(out, -8, "AV C missing-path sentinel");
}
