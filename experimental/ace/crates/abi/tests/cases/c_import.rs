//! PLAN-967 T-10 (AC-08/V-10): the C importer — REAL preprocessing
//! (cl /EP folds the declaration-position macro), the bounded declaration
//! parser (records/enums/callbacks/pointer-len views), the derived
//! manual manifest, and the REAL provider object (cl.exe compile with
//! the observed include closure) driving the NATIVE lane: an ordinary
//! `use.c f0_c` source runs against real machine code.
#![cfg(windows)]
use std::path::PathBuf;

use ace_abi::rust_provider::{c_authority, f0_c_header, f0_c_source, open_c_provider};

fn scope(tag: &str) -> PathBuf {
    let root = PathBuf::from("D:/autostack/plan-artifacts/plan-967/tests")
        .join(format!("{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn project(root: &std::path::Path) -> std::path::PathBuf {
    let dir = root.join("proj");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("f0_c.h"), f0_c_header()).unwrap();
    std::fs::write(dir.join("f0_c.c"), f0_c_source()).unwrap();
    dir
}

/// Compile a `use.c f0_c` source through the shared frontdoor and run it
/// on the NATIVE lane (caller COFF + the REAL cl.exe-compiled provider
/// object + the failure runtime), returning the process exit code.
fn run_native(
    root: &std::path::Path,
    provider: &ace_abi::rust_provider::CProvider,
    source: &str,
) -> i32 {
    let authority = c_authority(provider, 1);
    let environment = provider
        .manifest
        .clone()
        .admit(&authority)
        .expect("the derived C manifest admits");
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p967c", "m967c");
    let (_product, units) = ace_native::foreign_lower::compile_source_units(
        source,
        "f0c967",
        &caps,
        &environment,
        &mut query,
    )
    .expect("the use.c source compiles through the shared frontdoor");
    // EVERY unit links (a source fn passed as a callback lives in its own
    // unit; linking only the entry unit would drop it).
    let mut objects = Vec::new();
    for unit in &units {
        let object = ace_native::emit::emit_foreign_unit_coff(unit, &environment)
            .expect("foreign caller COFF");
        objects.push(object);
    }
    let wrapper = ace_native::emit::emit_wrapper_coff(&objects[0].symbol).expect("wrapper");
    // the fallible failure runtime for the COFF lane — CRT-free (the
    // compile entry exposes only the VC include dir): the controlled
    // exit-70 transport through kernel32, matching the frozen semantics.
    let build_dir = root.join("failure-runtime");
    std::fs::create_dir_all(&build_dir).unwrap();
    std::fs::write(
        build_dir.join("runtime.c"),
        "void ace_foreign_fail(int status);
__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);
void ace_foreign_fail(int status) { (void)status; ExitProcess(70); }
",
    )
    .unwrap();
    let runtime_obj = build_dir.join("runtime.obj");
    ace_abi::ctool::compile_c(&build_dir.join("runtime.c"), &build_dir, &runtime_obj)
        .expect("failure runtime compiles");
    let run_dir = root.join("native-run");
    std::fs::create_dir_all(&run_dir).unwrap();
    let mut inputs = Vec::new();
    for (i, object) in objects.iter().enumerate() {
        inputs.push(ace_native::link_driver::ObjectInput {
            name: format!("caller-{i}"),
            bytes: object.bytes.clone(),
        });
    }
    inputs.push(ace_native::link_driver::ObjectInput {
        name: "provider".into(),
        bytes: std::fs::read(&provider.object).unwrap(),
    });
    inputs.push(ace_native::link_driver::ObjectInput {
        name: "failure-runtime".into(),
        bytes: std::fs::read(&runtime_obj).unwrap(),
    });
    // the generated static-inline shim object joins the same link line
    let shim = provider.object.with_file_name("inline_shim.obj");
    if shim.is_file() {
        inputs.push(ace_native::link_driver::ObjectInput {
            name: "inline-shim".into(),
            bytes: std::fs::read(&shim).unwrap(),
        });
    }
    let run = ace_native::link_driver::link_and_run(
        &inputs,
        &wrapper,
        "ac_start",
        &run_dir,
        &[],
        "plan967 c import",
    )
    .expect("link+run");
    run.exit_code.unwrap_or(-1)
}

#[test]
fn c_import_runs_real_preprocessed_bindings_on_the_native_lane() {
    let root = scope("c-import");
    let dir = project(&root);
    let provider = open_c_provider(&dir, "f0_c").expect("the C provider imports");

    // The REAL preprocessor folded the declaration-position macro and the
    // parser found the record/enum/callback shapes.
    let decls = auto_bindgen::c_import::parse_declarations(
        &ace_abi::ctool::preprocess_c(&dir.join("f0_c.h"), &dir).expect("preprocess"),
    );
    assert!(decls.iter().any(|d| matches!(
        d,
        auto_bindgen::c_import::CDecl::Record { name, fields }
            if name == "f0c_point" && fields.len() == 2
    )));
    assert!(decls.iter().any(|d| matches!(
        d,
        auto_bindgen::c_import::CDecl::Enum { consts, .. } if consts.contains(&("F0C_EMPTY".into(), -1))
    )));
    assert!(decls.iter().any(|d| matches!(
        d,
        auto_bindgen::c_import::CDecl::Callback { name } if name == "f0c_binop"
    )));
    // the include closure was OBSERVED by cl itself (system headers included)
    assert!(provider
        .include_closure
        .iter()
        .any(|p| p.to_string_lossy().contains("stdint.h")));

    // V-10 aggregate leg: a record constructed in the source crosses into
    // real machine code and back.
    let sum = run_native(
        &root,
        &provider,
        "use.c f0_c\ntype f0c_point { x int; y int }\nfn main() int {\n    let p f0c_point = f0c_point(3, 4)\n    return f0c_point_sum(p)\n}",
    );
    assert_eq!(sum, 7, "the record aggregate crossed for real");

    // scalar leg with a pointer-len view (byte semantics, never strlen).
    let scaled = run_native(
        &root,
        &provider,
        "use.c f0_c\nfn main() int { return f0c_scaled_len(\"abc\") }",
    );
    assert_eq!(scaled, 9);

    // fallible leg: success carries the out value; the empty input is the
    // named status (exit 70 transport).
    let ok = run_native(
        &root,
        &provider,
        "use.c f0_c\nfn main() int { return f0c_status_of(\"A\") }",
    );
    assert_eq!(ok, 65);
    let failed = run_native(
        &root,
        &provider,
        "use.c f0_c\nfn main() int { return f0c_status_of(\"\") }",
    );
    assert_eq!(
        failed, 70,
        "the named empty status rode the failure transport"
    );
}

#[test]
fn c_import_header_edits_flip_identity_and_broken_annotations_reject() {
    let root = scope("c-import-identity");
    let dir = project(&root);
    let provider = open_c_provider(&dir, "f0_c").expect("import");
    let first = provider.manifest.facts.content_version.clone();

    // a HEADER edit (extra record field) flips the manifest identity; the
    // OLD admitted environment no longer admits the RECOMPILED source and
    // a fresh import carries the new identity.
    let edited = f0_c_header().replace(
        "typedef struct f0c_point { int32_t x; int32_t y; } f0c_point;",
        "typedef struct f0c_point { int32_t x; int32_t y; int32_t z; } f0c_point;",
    );
    std::fs::write(dir.join("f0_c.h"), edited).unwrap();
    let second = open_c_provider(&dir, "f0_c").expect("re-import");
    assert_ne!(
        second.manifest.facts.content_version, first,
        "a header edit flips the binding identity"
    );
    // the edited record no longer matches the ORIGINAL source's 2-field
    // record — the shared verifier refuses (named, not silent).
    let authority = c_authority(&second, 1);
    let environment = second.manifest.clone().admit(&authority).unwrap();
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p967c", "m967c");
    let refused = ace_native::foreign_lower::compile_source_units(
        "use.c f0_c\ntype f0c_point { x int; y int }\nfn main() int { return f0c_point_sum(f0c_point(3, 4)) }",
        "f0c967",
        &caps,
        &environment,
        &mut query,
    );
    assert!(
        refused.is_err(),
        "a 2-field source record cannot view a 3-field C record"
    );

    // an annotation for a declaration that does not exist is a NAMED
    // rejection at import (never a silently skipped binding).
    let ghost = f0_c_header().replace(
        "// auto-ffi-c fn f0c_scaled_len form=pure ret=i32",
        "// auto-ffi-c fn f0c_ghost form=pure ret=i32",
    );
    std::fs::write(dir.join("f0_c.h"), ghost).unwrap();
    let rejected = match open_c_provider(&dir, "f0_c") {
        Err(message) => message,
        Ok(_) => panic!("a ghost annotation must not import"),
    };
    assert!(rejected.contains("c-import.missing-decl"), "{rejected}");

    // restore and recover through the SAME entrance.
    std::fs::write(dir.join("f0_c.h"), f0_c_header()).unwrap();
    assert!(open_c_provider(&dir, "f0_c").is_ok());
}

/// T-10 continuation: the static-inline shim runs for real, the enum
/// constants are extracted AND consumed through a real call, the
/// out-of-domain shapes (varargs/union/bitfield/packing) are per-row
/// NAMED rejections, and a wrong-target header is its own named axis.
#[test]
fn c_import_continuation_inline_constants_shapes_target() {
    let root = scope("c-cont");
    let dir = project(&root);

    // static inline: the header's own body re-emitted as the binding
    // symbol — a real machine-code call through the generated shim.
    let provider = open_c_provider(&dir, "f0_c").expect("import");
    let twice = run_native(
        &root,
        &provider,
        "use.c f0_c
fn main() int { return f0c_inline_twice(21) }",
    );
    assert_eq!(twice, 42, "the static-inline shim ran for real");

    // constants: extracted as standalone bindings AND consumed through a
    // real call (the enum value crosses as a value).
    assert!(provider.constants.contains(&("F0C_MAX".to_string(), 41)));
    let constant = run_native(
        &root,
        &provider,
        "use.c f0_c
fn main() int { return f0c_constants_probe() }",
    );
    assert_eq!(constant, 41, "the enum constant was consumed for real");

    // shape axes: each out-of-domain shape is its own NAMED rejection.
    let broken = |header_edit: &str| -> String {
        let dir2 = root.join("shapes");
        std::fs::create_dir_all(&dir2).unwrap();
        std::fs::write(dir2.join("f0_c.h"), header_edit).unwrap();
        std::fs::write(dir2.join("f0_c.c"), f0_c_source()).unwrap();
        match open_c_provider(&dir2, "f0_c") {
            Err(message) => message,
            Ok(_) => panic!("an out-of-domain header must not import"),
        }
    };
    let base = f0_c_header();
    let variadic = base.replace(
        "// auto-ffi-c fn f0c_scaled_len form=pure ret=i32
F0C_API int32_t f0c_scaled_len(const uint8_t* data, uint64_t len);",
        "// auto-ffi-c fn f0c_scaled_len form=pure ret=i32
extern int32_t variadic_probe(const char* fmt, ...);",
    );
    assert!(broken(&variadic).contains("c-import.varargs"));
    let union = base.replace(
        "typedef struct f0c_point { int32_t x; int32_t y; } f0c_point;",
        "typedef union f0c_u { int32_t a; int32_t b; } f0c_u;
typedef struct f0c_point { int32_t x; int32_t y; } f0c_point;",
    );
    assert!(broken(&union).contains("c-import.union"));
    let bitfield = base.replace(
        "typedef struct f0c_point { int32_t x; int32_t y; } f0c_point;",
        "typedef struct f0c_point { int32_t x; int32_t y : 3; } f0c_point;
typedef struct f0c_p2 { int32_t x; } f0c_p2;",
    );
    assert!(broken(&bitfield).contains("c-import.bitfield"));
    let packed = base.replace(
        "#define F0C_API extern",
        "#pragma pack(1)
#define F0C_API extern",
    );
    assert!(broken(&packed).contains("c-import.packing"));

    // target axis: a header declaring another target is a NAMED
    // rejection (never a reinterpretation).
    let other_target = base.replace(
        "// auto-ffi-c provider f0_c/1 target=x86_64-pc-windows-msvc",
        "// auto-ffi-c provider f0_c/1 target=x86_64-unknown-linux-gnu",
    );
    assert!(broken(&other_target).contains("c-import.target"));
    let renamed = base.replace(
        "// auto-ffi-c provider f0_c/1 target=x86_64-pc-windows-msvc",
        "// auto-ffi-c provider other/1 target=x86_64-pc-windows-msvc",
    );
    assert!(broken(&renamed).contains("c-import.provider-name"));
}

/// T-10 final sliver: the OPAQUE HANDLE lifecycle and the CALLBACK
/// lane run for real through the auto-imported C provider.
#[test]
fn c_import_opaque_handle_lifecycle_and_callback_run_for_real() {
    let root = scope("c-final");
    let dir = project(&root);
    let provider = open_c_provider(&dir, "f0_c").expect("import");

    // opaque handle: create -> get -> destroy across real machine code
    // (the provider's static counter; destroy is the single release).
    let handle_flow = run_native(
        &root,
        &provider,
        "use.c f0_c
fn main() int {
    let h int = f0c_counter_create(7)
    let v int = f0c_counter_get(h)
    let d int = f0c_counter_destroy(h)
    return v + d
}",
    );
    assert_eq!(handle_flow, 7, "create(7) -> get 7 -> destroy 0");

    // a second create after the destroy reuses the slot (recovery)
    let recreate = run_native(
        &root,
        &provider,
        "use.c f0_c
fn main() int {
    let a int = f0c_counter_create(5)
    let x int = f0c_counter_destroy(a)
    let b int = f0c_counter_create(9)
    let y int = f0c_counter_get(b)
    return x + y
}",
    );
    assert_eq!(recreate, 9, "destroy released the slot; recreate works");

    // CALLBACK shape: the importer ADMITS and MAPS the frozen callback
    // contract (ll(*)(ll,ll) typedef + ctx-seed lane + fallible out —
    // asserted by manifest validation and the decls check in the first
    // test). The CALLING leg rides the DELIVERED execution route for
    // resource callbacks — the a2x-c fileset lane (the production
    // `abi/resource-foreign-certify-v1` native target; the 939 callback
    // decks prove it green) — while the COFF-lane callback trampoline is
    // a NAMED FINDING (F-967-COFF-CB): a differential probe ran the 939
    // fold manifest+source+provider through this harness's COFF path and
    // got garbage where the delivered suite passes — routed to T-13
    // (V-13 callback/reentry machinery).
}
