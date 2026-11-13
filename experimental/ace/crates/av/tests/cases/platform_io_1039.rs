use ace_av::console_host::{compile, restore, ConsoleCapture};
use ace_av_core::{
    av::{CancelFlag, ExecLimits},
    contract::{ExecOutcome, Value},
    platform_io::{
        CapturePlatformIo, Win32PlatformIo, ERR_ALREADY_RELEASED, ERR_BORROWED, ERR_INVALID_BOUNDS,
        ERR_INVALID_HANDLE, ERR_INVALID_VALUE, ERR_OS_FAILED, ERR_WRONG_ORIGIN,
    },
};

fn platform_grants() -> Vec<String> {
    vec![
        "platform.io.stdout".into(),
        "platform.io.stderr".into(),
        "platform.io.write".into(),
        "platform.mem".into(),
    ]
}

fn compile_and_run_capture(
    src: &str,
    grants: &[String],
    cap: &mut CapturePlatformIo,
) -> (ExecOutcome, ace_av_core::PlatformSessionSummary) {
    let program = compile(src, "d_main").unwrap();
    let mut console = ConsoleCapture::default();
    let (out, _trace, summary) = program
        .run_platform(
            grants,
            &ExecLimits::default(),
            &CancelFlag::new(false),
            Some(&mut console),
            cap,
        )
        .unwrap();
    (out, summary)
}

#[test]
fn pio01_av_std_streams_write_some_utf8_nul_and_partial_writes() {
    let src = r#"
fn write_all(h int, s str) int {
    var off = 0
    var rem = s.len_bytes()
    for rem > 0 {
        let w = io_write_some_slice(h, s, off, rem)
        if w <= 0 {
            return w
        }
        off = off + w
        rem = rem - w
    }
    return off
}

fn main() int {
    let out = io_stdout()
    let err = io_stderr()
    let a = write_all(out, "你好\n")
    let b = write_all(out, "a\0b")
    let c = io_write_some(err, "err!")
    let m = mem_alloc(8)
    let bad_h = io_write_some(999, "no")
    let bad_o = io_write_some(m, "no")
    let bad_b = io_write_some_slice(out, "abc", 2, 5)
    let _f = mem_free(m)
    if bad_h != -2 { return -101 }
    if bad_o != -3 { return -102 }
    if bad_b != -4 { return -103 }
    return a + b + c
}
"#;
    let mut cap = CapturePlatformIo {
        max_write_chunk: Some(3),
        use_synthetic_std_handle: true,
        ..Default::default()
    };
    let (out, summary) = compile_and_run_capture(src, &platform_grants(), &mut cap);
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(13),
                ..
            }
        ),
        "got {out:?}"
    );
    assert_eq!(cap.stdout, "你好\na\0b".as_bytes());
    assert_eq!(cap.stderr, b"err");
    assert_eq!(summary.written_total_bytes, 13);
    assert_eq!(summary.live_mapped_at_exit, 0);
    assert_eq!(summary.leaked_regions_cleaned, 0);
}

#[test]
fn pio02_av_os_error_last_error_and_per_run_grant_restore() {
    let err_src = r#"
fn main() int {
    let h = io_stdout()
    let w = io_write_some(h, "fail")
    if w != -8 {
        return -900
    }
    return io_last_error()
}
"#;
    let mut cap = CapturePlatformIo {
        inject_write_error: Some(109),
        use_synthetic_std_handle: true,
        ..Default::default()
    };
    let (out, summary) = compile_and_run_capture(err_src, &platform_grants(), &mut cap);
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(109),
                ..
            }
        ),
        "got {out:?}"
    );
    assert_eq!(summary.last_os_error, 109);
    assert_eq!(cap.win32.dispatches.write_some, 1);

    // Pre-OS rejection when write grant is missing: host.unauthorized and zero OS dispatches.
    let mut cap_denied = CapturePlatformIo {
        use_synthetic_std_handle: true,
        ..Default::default()
    };
    let program_err = compile(err_src, "d_main").unwrap();
    let mut console_denied = ConsoleCapture::default();
    let denied_err = program_err
        .run_platform(
            &["platform.io.stdout".into()],
            &ExecLimits::default(),
            &CancelFlag::new(false),
            Some(&mut console_denied),
            &mut cap_denied,
        )
        .unwrap_err();
    assert_eq!(denied_err.code, "host.unauthorized");
    assert_eq!(cap_denied.win32.dispatches.write_some, 0);

    // Persist + restore: no restored grants, no restored handles from prior runs.
    let reuse_src = r#"
fn main() int {
    let bogus = io_write_some(1, "stale")
    return bogus
}
"#;
    let dir = std::env::temp_dir().join(format!("av-pio-1039-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let program = compile(reuse_src, "d_main").unwrap();
    program.persist(&dir, "plan1039-test").unwrap();
    let restored = restore(&dir, &program.reference).unwrap();

    let mut console = ConsoleCapture::default();
    let mut cap_no_grant = CapturePlatformIo {
        use_synthetic_std_handle: true,
        ..Default::default()
    };
    let res_no_grant = restored
        .run_platform(
            &[],
            &ExecLimits::default(),
            &CancelFlag::new(false),
            Some(&mut console),
            &mut cap_no_grant,
        )
        .unwrap_err();
    assert_eq!(res_no_grant.code, "host.unauthorized");
    assert_eq!(cap_no_grant.win32.dispatches.write_some, 0);

    let mut cap_fresh = CapturePlatformIo {
        use_synthetic_std_handle: true,
        ..Default::default()
    };
    let (res_fresh, _, _) = restored
        .run_platform(
            &platform_grants(),
            &ExecLimits::default(),
            &CancelFlag::new(false),
            Some(&mut console),
            &mut cap_fresh,
        )
        .unwrap();
    assert!(
        matches!(
            res_fresh,
            ExecOutcome::Success {
                value: Value::I32(ERR_INVALID_HANDLE),
                ..
            }
        ),
        "got {res_fresh:?}"
    );
    assert_eq!(cap_fresh.win32.dispatches.write_some, 0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn vmem_av_os_region_bounds_borrow_and_failure_semantics() {
    let src = r#"
fn main() int {
    let ps = mem_page_size()
    let ag = mem_alloc_granularity()
    if ps < 4096 { return -1 }
    if ag < ps { return -1 }

    let m = mem_alloc(5000)
    if m <= 0 { return -2 }
    if mem_logical_len(m) != 5000 { return -3 }
    let mapped = mem_mapped_len(m)
    if mapped < 5000 { return -4 }
    if (mapped % ps) != 0 { return -4 }

    if mem_write_u8(m, 0, 65) != 0 { return -5 }
    if mem_write_u8(m, 4999, 90) != 0 { return -6 }
    if mem_read_u8(m, 0) != 65 { return -7 }
    if mem_read_u8(m, 4999) != 90 { return -8 }

    // Logical vs mapped check: offset 5000 is within mapped page (>=8192) but outside logical_len.
    if mem_write_u8(m, 5000, 1) != -4 { return -9 }
    if mem_read_u8(m, 5000) != -4 { return -10 }

    // Negative / invalid allocation values.
    if mem_alloc(0) != -4 { return -11 }
    if mem_alloc(-5) != -4 { return -12 }

    // Wrong origin rejection: stream handle cannot be freed or mutated as mem.
    let out = io_stdout()
    if mem_free(out) != -3 { return -13 }
    if mem_read_u8(out, 0) != -3 { return -14 }

    // Borrow active blocks release pre-OS.
    if mem_borrow(m) != 1 { return -15 }
    if mem_free(m) != -5 { return -16 }
    if mem_unborrow(m) != 0 { return -17 }
    if mem_unborrow(m) != -4 { return -18 }

    // Release and double-free check.
    if mem_free(m) != 0 { return -19 }
    if mem_free(m) != -6 { return -20 }
    if mem_read_u8(m, 0) != -6 { return -21 }

    return 100
}
"#;
    let mut cap = CapturePlatformIo {
        use_synthetic_std_handle: true,
        ..Default::default()
    };
    let (out, summary) = compile_and_run_capture(src, &platform_grants(), &mut cap);
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(100),
                ..
            }
        ),
        "got {out:?}"
    );
    assert_eq!(cap.win32.dispatches.virtual_alloc, 1);
    assert_eq!(cap.win32.dispatches.virtual_free, 1);
    assert_eq!(summary.leaked_regions_cleaned, 0);
    assert_eq!(summary.live_mapped_at_exit, 0);

    // Simulated OS failure in mem_free preserves owner + live_mapped_at_exit; ledger cleanup reclaims at exit.
    let retry_src = r#"
fn main() int {
    let m = mem_alloc(4096)
    if m <= 0 { return -1 }
    let f1 = mem_free(m)
    if f1 != -8 { return -2 }
    if io_last_error() != 487 { return -3 }
    if mem_write_u8(m, 0, 77) != 0 { return -4 }
    return mem_read_u8(m, 0)
}
"#;
    let mut cap_fail = CapturePlatformIo {
        inject_free_error: Some(487),
        use_synthetic_std_handle: true,
        ..Default::default()
    };
    let program = compile(retry_src, "d_main").unwrap();
    let mut console = ConsoleCapture::default();
    let (out_fail, _, summary_fail) = program
        .run_platform(
            &platform_grants(),
            &ExecLimits::default(),
            &CancelFlag::new(false),
            Some(&mut console),
            &mut cap_fail,
        )
        .unwrap();
    assert!(
        matches!(
            out_fail,
            ExecOutcome::Success {
                value: Value::I32(77),
                ..
            }
        ),
        "got {out_fail:?}"
    );
    assert!(summary_fail.leaked_regions_cleaned >= 1);
    assert!(summary_fail.live_mapped_at_exit >= 4096);
    assert_eq!(cap_fail.win32.dispatches.virtual_free, 2);
}

#[test]
fn pio04_av_win32_provider_real_os_mem_roundtrip() {
    // Runs directly through Win32PlatformIo (zero std::io / libc delegation).
    let src = r#"
fn main() int {
    let ps = mem_page_size()
    let m = mem_alloc(ps)
    if m <= 0 { return -1 }
    let _w = mem_write_u8(m, ps - 1, 123)
    let v = mem_read_u8(m, ps - 1)
    let f = mem_free(m)
    if f != 0 { return -2 }
    return v
}
"#;
    let program = compile(src, "d_main").unwrap();
    let mut console = ConsoleCapture::default();
    let mut win32 = Win32PlatformIo::default();
    let (out, _, summary) = program
        .run_platform(
            &platform_grants(),
            &ExecLimits::default(),
            &CancelFlag::new(false),
            Some(&mut console),
            &mut win32,
        )
        .unwrap();
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(123),
                ..
            }
        ),
        "got {out:?}"
    );
    assert_eq!(summary.leaked_regions_cleaned, 0);
    assert_eq!(summary.live_mapped_at_exit, 0);
    assert_eq!(
        (
            ERR_INVALID_HANDLE,
            ERR_WRONG_ORIGIN,
            ERR_INVALID_BOUNDS,
            ERR_BORROWED,
            ERR_ALREADY_RELEASED,
            ERR_INVALID_VALUE,
            ERR_OS_FAILED
        ),
        (-2, -3, -4, -5, -6, -7, -8)
    );
}

#[test]
fn vmem_av_internal_stdlib_helper_wrapper_roundtrip() {
    let helper = include_str!("../../../../../../stdlib/ace/internal/platform_mem.at");
    let src = format!(
        "{helper}\nfn main() int {{\n  let out = io_stdout()\n  let n = io_write_all(out, \"ok\\n\")\n  let m = mem_region_alloc(64)\n  if m <= 0 {{ return -1 }}\n  if mem_region_write_byte(m, 0, 99) != 0 {{ return -2 }}\n  let v = mem_region_read_byte(m, 0)\n  if mem_region_free(m) != 0 {{ return -3 }}\n  return v + n\n}}\n"
    );
    let program = compile(&src, "d_main").expect("compile");
    let mut cap = CapturePlatformIo {
        use_synthetic_std_handle: true,
        ..Default::default()
    };
    let mut console = ConsoleCapture::default();
    let (out, _, summary) = program
        .run_platform(
            &platform_grants(),
            &ExecLimits::default(),
            &CancelFlag::new(false),
            Some(&mut console),
            &mut cap,
        )
        .expect("run_platform");
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(102),
                ..
            }
        ),
        "got {out:?}"
    );
    assert_eq!(cap.stdout, b"ok\n");
    assert_eq!(summary.leaked_regions_cleaned, 0);
    assert_eq!(summary.live_mapped_at_exit, 0);
}
