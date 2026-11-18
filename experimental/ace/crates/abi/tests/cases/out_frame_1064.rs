//! PLAN-1064 T-FRAME / V-FRAME: Checked writable byte output borrow frames
//! (`&mut [u8]` in Rust, `uint8_t* dst, uint64_t cap` in C) via 967 automatic
//! source import -> generated C shim -> AV execution, with zero manual manifests.
#![cfg(windows)]

use ace_abi::rust_provider::{
    build_c_dll, build_source, c_authority, open_c_provider, open_session_from_build, RawLibrary,
};
use ace_av_core::av::{CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::foreign_exec::{
    ForeignCallable, ForeignExecError, ForeignStub, ForeignStubTable, OutBufferFrame,
    OutBufferRights,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
    let root = std::env::temp_dir().join(format!("p1064-frame-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    Scope(root)
}

fn run_av_source(
    source: &str,
    admitted: &ace_foreign_contract::manual::AdmittedForeign,
    table: &mut ForeignStubTable,
    cancel: &CancelFlag,
) -> ace_av_core::contract::ExecOutcome {
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p1064f", "m1064f");
    query.set_foreign(Some(admitted.clone()));
    let product = ace_source_query::compile_text_record_product(
        source, "f0frame", &caps, "p1064f", "m1064f", &mut query,
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
    ace_av_core::av::execute_foreign(&loaded, &ExecLimits::default(), cancel, table)
        .expect("authorized run")
        .0
}

fn rust_reader_provider_source() -> &'static str {
    r#"
//! auto-ffi provider f0_outread/1.0.0 target=x86_64-pc-windows-msvc profile=f0.outread

// auto-ffi fn read_into symbol=f0.outread.read_into.v1 form=pure ret=i32
pub fn read_into(path: &str, dst: &mut [u8]) -> i32 {
    if dst.is_empty() {
        // Explicit cap=0 test: check file readable, write 0 bytes cleanly
        return if std::path::Path::new(path).is_file() { 0 } else { -2 };
    }
    match std::fs::read(path) {
        Ok(data) => {
            let n = data.len().min(dst.len());
            dst[..n].copy_from_slice(&data[..n]);
            n as i32
        }
        Err(_) => -2,
    }
}

// auto-ffi fn oob_claim symbol=f0.outread.oob_claim.v1 form=pure ret=i32
pub fn oob_claim(dst: &mut [u8]) -> i32 {
    if !dst.is_empty() {
        dst[0] = 0x42;
    }
    (dst.len() as i32) + 5
}

// auto-ffi fn guard_scribble symbol=f0.outread.guard_scribble.v1 form=pure ret=i32
pub fn guard_scribble(dst: &mut [u8]) -> i32 {
    unsafe {
        let p = dst.as_mut_ptr();
        *p.offset(dst.len() as isize) = 0x99;
    }
    0
}
"#
}

#[test]
fn v_frame_rust_provider_read_into_and_guards() {
    let t0 = std::time::Instant::now();
    let root = scope("rust");
    let data_path = root.join("sample.bin");
    let sample_bytes: Vec<u8> = vec![
        b'A', 0xE4, 0xB8, 0xAD, 0x00, 0xFF, 0xF0, 0x9F, 0x98,
        0x80, // A + 中 + NUL + 0xFF + 😀 (10 bytes)
    ];
    std::fs::write(&data_path, &sample_bytes).unwrap();
    let path_str = data_path.to_string_lossy().replace('\\', "/");

    let build =
        build_source(&root, rust_reader_provider_source()).expect("rust out provider builds");
    let mut f0 = open_session_from_build(&build, 1).expect("opens");
    let cancel = CancelFlag::new(false);

    // 1. Full read into slice at offset=4, capacity=16, flanked by guard bytes 0xEE / 0xDD
    let mut backing = vec![0xEEu8; 4];
    backing.extend_from_slice(&[0xDDu8; 20]); // [4..20] cap=16, [20..24] trailing guard
    f0.session.table().register_out_buffer(
        1,
        OutBufferFrame::new("av", "guest", 1, backing, 4, 16, OutBufferRights::ReadWrite),
    );
    let src = format!("use.rs f0_outread\nfn main() int {{ return read_into(\"{path_str}\", 1) }}");
    let out = run_av_source(&src, &f0.admitted, f0.session.table(), &cancel);
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(10),
                ..
            }
        ),
        "got {out:?}"
    );
    {
        let frame = f0.session.table().out_buffer(1).unwrap();
        assert_eq!(frame.last_published, 10);
        assert!(!frame.is_pinned());
        assert_eq!(
            &frame.backing[..4],
            &[0xEEu8; 4],
            "leading guard bytes corrupted"
        );
        assert_eq!(
            &frame.backing[4..14],
            &sample_bytes[..],
            "published NUL+UTF8+binary bytes mismatch"
        );
        assert_eq!(
            &frame.backing[14..24],
            &[0xDDu8; 10],
            "uncommitted tail + trailing guard corrupted"
        );
    }

    // 2. Partial read (capacity=4) -> publishes only 4 bytes
    f0.session.table().register_out_buffer(
        2,
        OutBufferFrame::new(
            "av",
            "guest",
            1,
            vec![0xCCu8; 8],
            2,
            4,
            OutBufferRights::ReadWrite,
        ),
    );
    let src2 =
        format!("use.rs f0_outread\nfn main() int {{ return read_into(\"{path_str}\", 2) }}");
    let out2 = run_av_source(&src2, &f0.admitted, f0.session.table(), &cancel);
    assert!(
        matches!(
            out2,
            ExecOutcome::Success {
                value: Value::I32(4),
                ..
            }
        ),
        "got {out2:?}"
    );
    let f2 = f0.session.table().out_buffer(2).unwrap();
    assert_eq!(&f2.backing[..2], &[0xCC, 0xCC]);
    assert_eq!(&f2.backing[2..6], &sample_bytes[..4]);
    assert_eq!(&f2.backing[6..8], &[0xCC, 0xCC]);

    // 3. cap=0 legal empty borrow -> returns 0, writes 0 bytes
    f0.session.table().register_out_buffer(
        3,
        OutBufferFrame::new(
            "av",
            "guest",
            1,
            vec![0xBBu8; 4],
            1,
            0,
            OutBufferRights::ReadWrite,
        ),
    );
    let src3 =
        format!("use.rs f0_outread\nfn main() int {{ return read_into(\"{path_str}\", 3) }}");
    let out3 = run_av_source(&src3, &f0.admitted, f0.session.table(), &cancel);
    assert!(
        matches!(
            out3,
            ExecOutcome::Success {
                value: Value::I32(0),
                ..
            }
        ),
        "got {out3:?}"
    );
    assert_eq!(
        &f0.session.table().out_buffer(3).unwrap().backing,
        &[0xBBu8; 4]
    );

    // 4. Negative status (missing file) -> returns -2, backing untouched
    f0.session.table().register_out_buffer(
        4,
        OutBufferFrame::new(
            "av",
            "guest",
            1,
            vec![0xAAu8; 8],
            0,
            8,
            OutBufferRights::ReadWrite,
        ),
    );
    let src4 = format!(
        "use.rs f0_outread\nfn main() int {{ return read_into(\"{path_str}.missing\", 4) }}"
    );
    let out4 = run_av_source(&src4, &f0.admitted, f0.session.table(), &cancel);
    assert!(
        matches!(
            out4,
            ExecOutcome::Success {
                value: Value::I32(-2),
                ..
            }
        ),
        "got {out4:?}"
    );
    assert_eq!(
        &f0.session.table().out_buffer(4).unwrap().backing,
        &[0xAAu8; 8]
    );

    // 5. Native returns n > cap -> host rejects `foreign.out-overflow`, backing untouched
    f0.session.table().register_out_buffer(
        5,
        OutBufferFrame::new(
            "av",
            "guest",
            1,
            vec![0x77u8; 4],
            0,
            4,
            OutBufferRights::ReadWrite,
        ),
    );
    let out5 = run_av_source(
        "use.rs f0_outread\nfn main() int { return oob_claim(5) }",
        &f0.admitted,
        f0.session.table(),
        &cancel,
    );
    assert!(
        matches!(&out5, ExecOutcome::Infra(m) if m.contains("foreign.out-overflow")),
        "expected foreign.out-overflow, got {out5:?}"
    );
    assert_eq!(
        &f0.session.table().out_buffer(5).unwrap().backing,
        &[0x77u8; 4]
    );

    // 6. Native writes past end into guard bytes -> host rejects `foreign.out-guard`, backing untouched
    let out6 = run_av_source(
        "use.rs f0_outread\nfn main() int { return guard_scribble(5) }",
        &f0.admitted,
        f0.session.table(),
        &cancel,
    );
    assert!(
        matches!(&out6, ExecOutcome::Infra(m) if m.contains("foreign.out-guard")),
        "expected foreign.out-guard, got {out6:?}"
    );
    assert_eq!(
        &f0.session.table().out_buffer(5).unwrap().backing,
        &[0x77u8; 4]
    );

    assert!(t0.elapsed().as_millis() <= 10_000, "test exceeded 10s");
}

#[test]
fn v_frame_c_provider_read_into() {
    let t0 = std::time::Instant::now();
    let root = scope("c");
    let proj = root.join("proj");
    std::fs::create_dir_all(&proj).unwrap();

    let header = r#"
// auto-ffi-c provider f0_cout/1.0.0 target=x86_64-pc-windows-msvc
#pragma once
#include <stdint.h>

// auto-ffi-c fn c_read_into form=pure ret=i32
int32_t c_read_into(const uint8_t* path, uint64_t path_len, uint8_t* dst, uint64_t cap);
"#;
    let impl_c = r#"
#include "f0_cout.h"
#include <stdint.h>

// Freestanding C provider implementation (no CRT stdio dependency):
// writes prefix+NUL+UTF8('中','😀')+0xFF based on non-empty path view up to cap.
int32_t c_read_into(const uint8_t* path, uint64_t path_len, uint8_t* dst, uint64_t cap) {
    if (path_len == 0) return -2;
    if (cap == 0) return 0;
    const uint8_t payload[10] = { 'C', 0x00, 0xE4, 0xB8, 0xAD, 0xFF, 0xF0, 0x9F, 0x98, 0x80 };
    uint64_t n = cap < 10 ? cap : 10;
    for (uint64_t i = 0; i < n; ++i) {
        dst[i] = payload[i];
    }
    (void)path;
    return (int32_t)n;
}
"#;
    std::fs::write(proj.join("f0_cout.h"), header).unwrap();
    std::fs::write(proj.join("f0_cout.c"), impl_c).unwrap();
    let sample = vec![b'C', 0x00, 0xE4, 0xB8, 0xAD, 0xFF, 0xF0, 0x9F, 0x98, 0x80];

    let provider = open_c_provider(&proj, "f0_cout").expect("c provider imports OutBytes");
    let dll = build_c_dll(&provider, &root).expect("c dll builds");
    let library = RawLibrary::load(&dll).expect("loads c dll");
    let authority = c_authority(&provider, 1);
    let admitted = provider.manifest.clone().admit(&authority).expect("admits");

    let mut table = ForeignStubTable::new(1);
    for decl in admitted.source_declarations_ref() {
        if decl.function.symbol == "c_read_into" {
            let callable = unsafe {
                ForeignCallable::ViewOutBytesToI32(std::mem::transmute::<
                    _,
                    unsafe extern "C" fn(*const u8, u64, *mut u8, u64) -> i32,
                >(
                    library.symbol("c_read_into").expect("c_read_into export"),
                ))
            };
            table
                .insert(ForeignStub::new(
                    decl.provider.clone(),
                    decl.function.symbol.clone(),
                    decl.content_version.clone(),
                    1,
                    callable,
                ))
                .unwrap();
        }
    }
    table.register_out_buffer(
        10,
        OutBufferFrame::new(
            "av",
            "guest",
            1,
            vec![0x55u8; 16],
            2,
            12,
            OutBufferRights::ReadWrite,
        ),
    );
    let cancel = CancelFlag::new(false);
    let out = run_av_source(
        "use.c f0_cout\nfn main() int { return c_read_into(\"sample.bin\", 10) }",
        &admitted,
        &mut table,
        &cancel,
    );
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(10),
                ..
            }
        ),
        "got {out:?}"
    );
    let f = table.out_buffer(10).unwrap();
    assert_eq!(&f.backing[..2], &[0x55, 0x55]);
    assert_eq!(&f.backing[2..12], &sample[..]);
    assert_eq!(&f.backing[12..16], &[0x55u8; 4]);
    assert!(t0.elapsed().as_millis() <= 10_000, "test exceeded 10s");
}

#[test]
fn v_frame_security_lifecycle_and_import_negatives() {
    // 1. Negative controls on OutBufferFrame admission + host pin lifetime under CancelFlag
    let cancel = CancelFlag::new(false);
    static LIVE_PIN_SEEN: AtomicBool = AtomicBool::new(false);
    static mut SHARED_PIN: Option<Arc<AtomicBool>> = None;
    static mut SHARED_CANCEL: Option<*const CancelFlag> = None;

    unsafe extern "C" fn slow_native(dst: *mut u8, cap: u64) -> i32 {
        // Flip CancelFlag mid-call and check that the host buffer remains pinned!
        if let Some(c) = SHARED_CANCEL {
            (*c).store(true, Ordering::SeqCst);
        }
        if let Some(ref pin) = SHARED_PIN {
            LIVE_PIN_SEEN.store(pin.load(Ordering::SeqCst), Ordering::SeqCst);
        }
        if cap >= 2 && !dst.is_null() {
            *dst = b'O';
            *dst.add(1) = b'K';
            2
        } else {
            0
        }
    }

    let manifest = auto_bindgen::rust_import::import(
        r#"
//! auto-ffi provider f0_neg/1.0.0 target=x86_64-pc-windows-msvc profile=f0.neg
// auto-ffi fn fill symbol=f0.neg.fill.v1 form=pure ret=i32
pub fn fill(dst: &mut [u8]) -> i32 { 0 }
"#,
    )
    .expect("import OutBytes-only");
    let h_dig = ace_foreign_contract::blake3_hex(b"h");
    let p_dig = ace_foreign_contract::blake3_hex(b"p");
    let manual = auto_bindgen::rust_import::to_manual_manifest(
        &manifest,
        "f0-rust-provider/1",
        &h_dig,
        &p_dig,
    )
    .unwrap();
    let auth = ace_foreign_contract::manual::HostAuthority {
        review_id: manual.review_id.clone(),
        contract_digest: manual.contract_digest().unwrap(),
        provider: manual.provider.clone(),
        provider_digest: h_dig.clone(),
        header_digest: h_dig,
        physical_table_digest: p_dig,
        target: manual.target.clone(),
        capabilities: manual.facts.capabilities.iter().cloned().collect(),
        available_symbols: ["f0_neg_fill_v1".into()].into_iter().collect(),
        binding_generation: 1,
    };
    let admitted = manual.admit(&auth).unwrap();
    let decl = &admitted.source_declarations_ref()[0];

    let mut table = ForeignStubTable::new(1);
    table
        .insert(ForeignStub::new(
            decl.provider.clone(),
            decl.function.symbol.clone(),
            decl.content_version.clone(),
            1,
            ForeignCallable::OutBytesToI32(slow_native),
        ))
        .unwrap();

    // Negative frame checks
    table.register_out_buffer(
        1,
        OutBufferFrame::new(
            "other",
            "guest",
            1,
            vec![0; 4],
            0,
            4,
            OutBufferRights::ReadWrite,
        ),
    );
    assert!(
        matches!(table.dispatch_view_out_bytes(decl, &[], 1, &cancel), Err(ForeignExecError::Reject(r)) if r.code == "foreign.out-owner")
    );

    table.register_out_buffer(
        2,
        OutBufferFrame::new(
            "av",
            "foreign",
            1,
            vec![0; 4],
            0,
            4,
            OutBufferRights::ReadWrite,
        ),
    );
    assert!(
        matches!(table.dispatch_view_out_bytes(decl, &[], 2, &cancel), Err(ForeignExecError::Reject(r)) if r.code == "foreign.out-origin")
    );

    table.register_out_buffer(
        3,
        OutBufferFrame::new(
            "av",
            "guest",
            1,
            vec![0; 4],
            0,
            4,
            OutBufferRights::ReadOnly,
        ),
    );
    assert!(
        matches!(table.dispatch_view_out_bytes(decl, &[], 3, &cancel), Err(ForeignExecError::Reject(r)) if r.code == "foreign.out-readonly")
    );

    table.register_out_buffer(
        4,
        OutBufferFrame::new(
            "av",
            "guest",
            99,
            vec![0; 4],
            0,
            4,
            OutBufferRights::ReadWrite,
        ),
    );
    assert!(
        matches!(table.dispatch_view_out_bytes(decl, &[], 4, &cancel), Err(ForeignExecError::Reject(r)) if r.code == "foreign.out-generation")
    );

    table.register_out_buffer(
        5,
        OutBufferFrame::new(
            "av",
            "guest",
            1,
            vec![0; 4],
            2,
            4,
            OutBufferRights::ReadWrite,
        ),
    );
    assert!(
        matches!(table.dispatch_view_out_bytes(decl, &[], 5, &cancel), Err(ForeignExecError::Reject(r)) if r.code == "foreign.out-bounds")
    );

    let pinned_frame = OutBufferFrame::new(
        "av",
        "guest",
        1,
        vec![0; 4],
        0,
        4,
        OutBufferRights::ReadWrite,
    );
    pinned_frame.pin_flag.store(true, Ordering::SeqCst);
    table.register_out_buffer(6, pinned_frame);
    assert!(
        matches!(table.dispatch_view_out_bytes(decl, &[], 6, &cancel), Err(ForeignExecError::Reject(r)) if r.code == "foreign.out-aliased")
    );
    assert!(
        matches!(table.close_out_buffer(6), Err(ForeignExecError::Reject(r)) if r.code == "foreign.out-pinned")
    );

    // Mid-call CancelFlag test: frame stays pinned until native returns, then unpins cleanly
    let live_frame = OutBufferFrame::new(
        "av",
        "guest",
        1,
        vec![0; 4],
        0,
        4,
        OutBufferRights::ReadWrite,
    );
    let pin_arc = live_frame.pin_flag.clone();
    table.register_out_buffer(7, live_frame);
    unsafe {
        SHARED_PIN = Some(pin_arc.clone());
        SHARED_CANCEL = Some(&cancel as *const CancelFlag);
    }
    let res = table
        .dispatch_view_out_bytes(decl, &[], 7, &cancel)
        .expect("in-flight returns");
    assert_eq!(res, ace_av_core::foreign_exec::ForeignDispatch::PureI32(2));
    assert!(
        LIVE_PIN_SEEN.load(Ordering::SeqCst),
        "buffer must stay pinned while native executes"
    );
    assert!(
        !pin_arc.load(Ordering::SeqCst),
        "buffer must unpin after native returns"
    );
    assert_eq!(&table.out_buffer(7).unwrap().backing[..2], b"OK");
    // CancelFlag now blocks new call cleanly before native entry, and close succeeds after return
    assert!(
        matches!(table.dispatch_view_out_bytes(decl, &[], 7, &cancel), Err(ForeignExecError::Reject(r)) if r.code == "foreign.cancelled")
    );
    table
        .close_out_buffer(7)
        .expect("close succeeds once unpinned");
    assert!(
        matches!(table.dispatch_view_out_bytes(decl, &[], 7, &CancelFlag::new(false)), Err(ForeignExecError::Reject(r)) if r.code == "foreign.out-closed")
    );

    // 2. Arbitrary &mut and wrong target/ABI still rejected cleanly
    assert_eq!(
        auto_bindgen::rust_import::import(
            "//! auto-ffi provider p/1.0.0 target=x86_64-pc-windows-msvc profile=p\n// auto-ffi fn bad symbol=p.bad.v1 form=pure ret=i32\npub fn bad(x: &mut i32) -> i32 { 0 }\n"
        ).unwrap_err().code,
        "rust-import.mut-borrow"
    );
    let wrong_target = auto_bindgen::rust_import::import(
        "//! auto-ffi provider p/1.0.0 target=x86_64-unknown-linux-gnu profile=p\n// auto-ffi fn fill symbol=p.fill.v1 form=pure ret=i32\npub fn fill(dst: &mut [u8]) -> i32 { 0 }\n"
    ).unwrap();
    assert!(auto_bindgen::rust_import::to_manual_manifest(&wrong_target, "r", "h", "p").is_err());
    assert!(
        auto_bindgen::c_import::parse_provider_header(
            "// auto-ffi-c provider c/1.0 target=x86_64-unknown-linux-gnu"
        )
        .unwrap_err()
        .code
            == "c-import.target"
    );
}
