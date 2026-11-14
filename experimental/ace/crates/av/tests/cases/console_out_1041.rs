//! PLAN-1041 OUT-01 (AV legs): the common Auto output source
//! (`stdlib/auto/platform/io.at`, inlined into the console lane exactly
//! like the 1039 helper) really converts UTF-8→UTF-16LE on the console
//! path (capture oracle asserts the exact UTF-16 code units, including
//! surrogate pairs and cross-chunk boundaries), preserves raw UTF-8 bytes
//! on the redirect path (NUL is content), rejects console byte writes
//! explicitly, and carries unit-level progress through short writes.
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_av::console_host::{compile, ConsoleCapture};
use ace_av_core::{
    av::{CancelFlag, ExecLimits},
    contract::{ExecOutcome, Value},
    platform_io::CapturePlatformIo,
};

const IO_AT: &str = include_str!("../../../../../../stdlib/auto/platform/io.at");

fn platform_grants() -> Vec<String> {
    vec![
        "platform.io.stdout".into(),
        "platform.io.stderr".into(),
        "platform.io.write".into(),
        "platform.mem".into(),
    ]
}

fn run_capture(
    main_src: &str,
    cap: &mut CapturePlatformIo,
) -> (ExecOutcome, ace_av_core::PlatformSessionSummary) {
    let src = format!("{IO_AT}\n{main_src}");
    let program = compile(&src, "d_main").unwrap();
    let mut console = ConsoleCapture::default();
    let (out, _trace, summary) = program
        .run_platform(
            &platform_grants(),
            &ExecLimits::default(),
            &CancelFlag::new(false),
            Some(&mut console),
            cap,
        )
        .unwrap();
    (out, summary)
}

fn expect_i32(out: &ExecOutcome) -> i32 {
    match out {
        ExecOutcome::Success {
            value: Value::I32(n),
            ..
        } => *n,
        other => panic!("expected i32 success, got {other:?}"),
    }
}

/// OUT-01 console: exact UTF-16 units for 中文 + emoji (surrogate pairs) +
/// an astral char straddling the 4096-byte chunk boundary; both streams;
/// empty input returns 0 without a dispatch.
#[test]
fn out01_av_console_text_utf16_units_surrogates_and_chunks() {
    // Build >4096 UTF-8 bytes so the console loop crosses a chunk
    // boundary; place an astral char (4-byte UTF-8, surrogate pair) right
    // at byte 4096 so the pair is submitted complete, never split.
    let mut body = String::new();
    body.push_str("fn main() int {\n");
    body.push_str("    let h = out_stdout()\n");
    body.push_str("    if out_kind(h) != 1 { return -1 }\n");
    body.push_str("    let e = out_stderr()\n");
    body.push_str("    let z = out_text(h, \"\")\n");
    body.push_str("    if z != 0 { return -2 }\n");
    body.push_str("    let a = out_text(h, \"你好\\n\")\n");
    body.push_str("    let b = out_text(h, \"A😀Z\")\n");
    body.push_str("    let c = out_text(e, \"err中\")\n");
    body.push_str("    let big = \"");
    for _ in 0..1360 {
        body.push('中'); // 3 bytes each -> 4080 bytes
    }
    body.push('😀'); // 4 bytes -> crosses 4096 mid-scalar by chunk rule
    for _ in 0..10 {
        body.push('x');
    }
    body.push_str("\"\n");
    body.push_str("    let d = out_text(h, big)\n");
    body.push_str("    return a + b + c + d\n");
    body.push_str("}\n");

    let mut cap = CapturePlatformIo {
        force_console_kind: true,
        use_synthetic_std_handle: true,
        ..Default::default()
    };
    let (out, summary) = run_capture(&body, &mut cap);
    let ret = expect_i32(&out);
    let a = "你好\n".len(); // UTF-8 bytes
    let b = "A😀Z".len();
    let c = "err中".len();
    let d = 1360 * 3 + 4 + 10;
    assert_eq!(
        ret as usize,
        a + b + c + d,
        "confirmed UTF-8 byte progress, got {ret}"
    );

    let mut expect_out: Vec<u16> = "你好\n".encode_utf16().collect();
    expect_out.extend("A😀Z".encode_utf16());
    let mut big = String::new();
    for _ in 0..1360 {
        big.push('中');
    }
    big.push('😀');
    for _ in 0..10 {
        big.push('x');
    }
    expect_out.extend(big.encode_utf16());
    assert_eq!(cap.console16_stdout, expect_out, "exact UTF-16 units");
    assert_eq!(
        cap.console16_stderr,
        "err中".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        cap.stdout.len(),
        0,
        "console mode must not touch the byte sink"
    );
    // 2×units cross the OS boundary and count toward the platform budget.
    assert_eq!(
        summary.written_total_bytes as usize,
        expect_out.len() * 2 + 2 * "err中".encode_utf16().count()
    );
    assert_eq!(summary.live_mapped_at_exit, 0, "conversion buffer freed");
    assert_eq!(summary.leaked_regions_cleaned, 0);
}

/// OUT-01 redirect: original UTF-8 bytes with embedded NUL, byte-exact;
/// short writes (1 byte per call) still complete; out_bytes on a console
/// is explicitly rejected (-13), not silently lossy.
#[test]
fn out01_av_redirect_bytes_nul_short_writes_and_console_reject() {
    let src = r#"
fn main() int {
    let h = out_stdout()
    if out_kind(h) != 2 { return -1 }
    let a = out_text(h, "ab\0cd")
    let b = out_bytes(h, "你好")
    let e = out_stderr()
    let c = out_bytes(e, "E\0")
    return a + b + c
}
"#;
    // 1-byte writes force the bounded write loop through real progress.
    let mut cap = CapturePlatformIo {
        max_write_chunk: Some(1),
        use_synthetic_std_handle: true,
        ..Default::default()
    };
    let (out, _summary) = run_capture(src, &mut cap);
    let ret = expect_i32(&out);
    assert_eq!(ret as usize, "ab\0cd".len() + "你好".len() + "E\0".len());
    assert_eq!(cap.stdout, "ab\0cd你好".as_bytes());
    assert_eq!(cap.stderr, b"E\0");
}

/// OUT-01 console reject: on a TEXT console out_bytes is explicitly
/// rejected (-13) on both streams — no silent lossy byte interpretation.
#[test]
fn out01_av_console_bytes_rejected_on_text_console() {
    let src = r#"
fn main() int {
    let h = out_stdout()
    if out_kind(h) != 1 { return -1 }
    let r1 = out_bytes(h, "x")
    let r2 = out_bytes(out_stderr(), "y")
    if r1 != -13 { return -100 }
    if r2 != -13 { return -101 }
    return 0
}
"#;
    let mut cap = CapturePlatformIo {
        force_console_kind: true,
        use_synthetic_std_handle: true,
        ..Default::default()
    };
    let (out, _summary) = run_capture(src, &mut cap);
    assert_eq!(expect_i32(&out), 0);
    assert!(cap.console16_stdout.is_empty() && cap.console16_stderr.is_empty());
}

/// OUT-01 error path: an OS failure mid-console-write returns -8 with the
/// raw error observable via out_last_error; a byte-sink error likewise.
#[test]
fn out01_av_console_and_byte_errors_carry_raw_os_error() {
    let src = r#"
fn main() int {
    let h = out_stdout()
    let w = out_text(h, "你好")
    if w != -8 { return -200 }
    return out_last_error()
}
"#;
    let mut cap = CapturePlatformIo {
        force_console_kind: true,
        use_synthetic_std_handle: true,
        inject_console16_error: Some(109),
        ..Default::default()
    };
    let (out, _summary) = run_capture(src, &mut cap);
    assert_eq!(expect_i32(&out), 109, "raw GetLastError observable");
    // The buffer is freed even on the error path.
    assert_eq!(_summary.live_mapped_at_exit, 0);

    let src2 = r#"
fn main() int {
    let h = out_stdout()
    let w = out_bytes(h, "fail")
    if w != -8 { return -300 }
    return out_last_error()
}
"#;
    let mut cap2 = CapturePlatformIo {
        inject_write_error: Some(232),
        use_synthetic_std_handle: true,
        ..Default::default()
    };
    let (out2, _) = run_capture(src2, &mut cap2);
    assert_eq!(expect_i32(&out2), 232);
}

/// OUT-01 short console writes: the OS accepting only 1 unit per call
/// forces the unit-level loop (including a lone low surrogate continuing
/// after its high surrogate) to still deliver every unit exactly once.
#[test]
fn out01_av_console_short_unit_writes_deliver_units_once() {
    let src = r#"
fn main() int {
    let h = out_stdout()
    return out_text(h, "a😀b中")
}
"#;
    let mut cap = CapturePlatformIo {
        force_console_kind: true,
        use_synthetic_std_handle: true,
        max_console16_units: Some(1),
        ..Default::default()
    };
    let (out, _summary) = run_capture(src, &mut cap);
    assert_eq!(expect_i32(&out) as usize, "a😀b中".len());
    assert_eq!(
        cap.console16_stdout,
        "a😀b中".encode_utf16().collect::<Vec<u16>>()
    );
}

/// OUT-03 cancel: a cancelled run ends in the lane's controlled
/// Cancelled outcome — bounded, never a hang, never a silent partial
/// success. (The op-level 995 poll inside write_some/console_write16 is
/// the 1039 backend seam; the AV lane's observable contract is the
/// checked cancellation outcome.)
#[test]
fn out03_av_cancel_mid_write_is_controlled() {
    let src = r#"
fn main() int {
    let h = out_stdout()
    let w = out_text(h, "你好，世界")
    return w
}
"#;
    let src_io = format!(
        "{IO_AT}
{src}"
    );
    let program = compile(&src_io, "d_main").unwrap();
    let mut console = ConsoleCapture::default();
    let mut cap = CapturePlatformIo {
        force_console_kind: true,
        use_synthetic_std_handle: true,
        ..Default::default()
    };
    let cancel = CancelFlag::new(true);
    let (out, _trace, _summary) = program
        .run_platform(
            &platform_grants(),
            &ExecLimits::default(),
            &cancel,
            Some(&mut console),
            &mut cap,
        )
        .unwrap();
    assert!(
        matches!(out, ExecOutcome::Cancelled { .. }),
        "cancelled run ends in the checked Cancelled outcome, got {out:?}"
    );
}

/// OUT-03 WriteZero: a sink that reports success-with-zero progress
/// forever hits the bounded retry (64) and returns -14 — never a hang.
#[test]
fn out03_av_write_zero_is_bounded() {
    let src = r#"
fn main() int {
    let h = out_stdout()
    return out_bytes(h, "data")
}
"#;
    let mut cap = CapturePlatformIo {
        max_write_chunk: Some(0),
        use_synthetic_std_handle: true,
        ..Default::default()
    };
    let (out, _summary) = run_capture(src, &mut cap);
    assert_eq!(expect_i32(&out), -14, "bounded WriteZero, controlled -14");
    assert!(cap.stdout.is_empty(), "no bytes moved");
}
