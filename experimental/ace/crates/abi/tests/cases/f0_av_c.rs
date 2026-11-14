//! PLAN-967 T-12 (V-COVERAGE AV-C): the C provider drives the AV lane —
//! the cl /LD-built provider DLL loads into the AV stub table and the
//! same ordinary `use.c` source executes against REAL C machine code
//! through the AV interpreter (no Rust provider anywhere in this leg).
#![cfg(windows)]
use std::path::PathBuf;

use ace_abi::rust_provider::{
    build_c_dll, c_authority, f0_c_header, f0_c_source, open_c_provider, RawLibrary,
};
use ace_av_core::foreign_exec::{ForeignCallable, ForeignStub, ForeignStubTable};

fn scope(tag: &str) -> PathBuf {
    let root = PathBuf::from("D:/autostack/plan-artifacts/plan-967/tests")
        .join(format!("{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn run_av_c(
    source: &str,
    table: &mut ForeignStubTable,
    environment: &ace_foreign_contract::manual::AdmittedForeign,
) -> ace_av_core::contract::ExecOutcome {
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p967avc", "m967avc");
    query.set_foreign(Some(environment.clone()));
    let product = ace_source_query::compile_text_record_product(
        source, "f0avc", &caps, "p967avc", "m967avc", &mut query,
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
fn av_runs_the_c_provider_through_a_real_dll() {
    let root = scope("av-c");
    let dir = root.join("proj");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("f0_c.h"), f0_c_header()).unwrap();
    std::fs::write(dir.join("f0_c.c"), f0_c_source()).unwrap();
    let provider = open_c_provider(&dir, "f0_c").expect("the C provider imports");
    let dll = build_c_dll(&provider, &root).expect("the provider DLL builds");
    let library = RawLibrary::load(&dll).expect("the DLL loads");

    // bind the scalar-view entries into the AV stub table — the SAME
    // physical frames the Rust lane uses (view->i32, view+out->status)
    let authority = c_authority(&provider, 1);
    let environment = provider.manifest.clone().admit(&authority).expect("admits");
    let mut table = ForeignStubTable::new(1);
    for decl in environment.source_declarations_ref() {
        let symbol = decl.function.symbol.as_str();
        let lanes = decl.resource().map(|r| r.params.len()).unwrap_or(0);
        let callable = match (symbol, lanes, decl.is_fallible()) {
            ("f0c_scaled_len", 1, false) => unsafe {
                ForeignCallable::PureI32(std::mem::transmute::<
                    _,
                    unsafe extern "C" fn(*const u8, u64) -> i32,
                >(library.symbol(symbol).expect("export")))
            },
            ("f0c_status_of", 1, true) => unsafe {
                ForeignCallable::FallibleOutI32(std::mem::transmute::<
                    _,
                    unsafe extern "C" fn(*const u8, u64, *mut i32) -> i32,
                >(
                    library.symbol(symbol).expect("export")
                ))
            },
            // the AV-C slice covers the scalar-view frames; record,
            // callback, inline and constant entries stay on their own
            // delivered lanes (native/a2x-c).
            _ => continue,
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

    // the AV interpreter executes REAL C code through the DLL
    let scaled = run_av_c(
        "use.c f0_c\nfn main() int { return f0c_scaled_len(\"hello\") }",
        &mut table,
        &environment,
    );
    assert!(matches!(
        scaled,
        ace_av_core::contract::ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(15),
            ..
        }
    ));
    let status = run_av_c(
        "use.c f0_c\nfn main() int { return f0c_status_of(\"A\") }",
        &mut table,
        &environment,
    );
    assert!(matches!(
        status,
        ace_av_core::contract::ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(65),
            ..
        }
    ));
    let failed = run_av_c(
        "use.c f0_c\nfn main() int { return f0c_status_of(\"\") }",
        &mut table,
        &environment,
    );
    assert!(matches!(
        failed,
        ace_av_core::contract::ExecOutcome::Trap {
            kind: ace_av_core::contract::TrapKind::ForeignStatus,
            ..
        }
    ));
    assert!(table.callee_count() >= 3);
}
