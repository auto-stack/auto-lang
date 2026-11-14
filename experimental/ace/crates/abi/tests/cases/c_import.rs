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
    let unit = units.first().expect("one unit");
    let object =
        ace_native::emit::emit_foreign_unit_coff(unit, &environment).expect("foreign caller COFF");
    let wrapper = ace_native::emit::emit_wrapper_coff(&object.symbol).expect("wrapper");
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
    let run = ace_native::link_driver::link_and_run(
        &[
            ace_native::link_driver::ObjectInput {
                name: "caller".into(),
                bytes: object.bytes.clone(),
            },
            ace_native::link_driver::ObjectInput {
                name: "provider".into(),
                bytes: std::fs::read(&provider.object).unwrap(),
            },
            ace_native::link_driver::ObjectInput {
                name: "failure-runtime".into(),
                bytes: std::fs::read(&runtime_obj).unwrap(),
            },
        ],
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
