//! PLAN-1039: Native Windows x64 Platform I/O (`platform.io/1`), Checked OS
//! Virtual Memory (`platform.mem/1`), and PE Import Dependency/Symbol Audit.

use ace_native::console::{
    console_authorize, console_compile, console_native_build, console_native_run,
};
use ace_native::link_driver::{link_only, ObjectInput};
use ace_native::platform_io::{audit_pe_imports, ALLOWED_KERNEL32_IMPORTS};
use std::process::{Command, Stdio};

fn platform_grants() -> Vec<String> {
    vec![
        "platform.io.stdout".into(),
        "platform.io.stderr".into(),
        "platform.io.write".into(),
        "platform.mem".into(),
    ]
}

fn temp_workdir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "mn-pio1039-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn pio01_native_win32_stdout_stderr_write_some_utf8_and_nul() {
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
    let mir = console_compile(src, "d_main").expect("compile");
    console_authorize(&mir, &platform_grants()).expect("authorize");
    let dir = temp_workdir("pio01");
    let pe = console_native_build(&mir, "d_main", &dir).expect("native build");

    let pe_bytes = std::fs::read(&pe.exe_path).expect("read exe");
    let report = audit_pe_imports(&pe_bytes).expect("PE import audit pass");
    assert_eq!(report.dlls.len(), 1);
    assert!(report.dlls[0].eq_ignore_ascii_case("kernel32.dll"));
    for (_, sym) in &report.symbols {
        assert!(
            ALLOWED_KERNEL32_IMPORTS.contains(&sym.as_str()),
            "unexpected symbol {sym}"
        );
    }

    let out = Command::new(&pe.exe_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run pe");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(out.status.code(), Some(14));
    assert_eq!(out.stdout, "你好\na\0b".as_bytes());
    assert_eq!(out.stderr, b"err!");
}

#[cfg(windows)]
#[test]
fn pio02_native_win32_broken_pipe_last_error_and_grant_rejection() {
    use std::os::windows::io::{FromRawHandle, OwnedHandle};

    unsafe extern "system" {
        fn CreatePipe(
            hReadPipe: *mut *mut core::ffi::c_void,
            hWritePipe: *mut *mut core::ffi::c_void,
            lpPipeAttributes: *const core::ffi::c_void,
            nSize: u32,
        ) -> i32;
        fn CloseHandle(hObject: *mut core::ffi::c_void) -> i32;
        fn SetHandleInformation(hObject: *mut core::ffi::c_void, dwMask: u32, dwFlags: u32) -> i32;
    }

    let src = r#"
fn main() int {
    let h = io_stdout()
    let w = io_write_some(h, "fail")
    if w != -8 {
        return -900
    }
    return io_last_error()
}
"#;
    let mir = console_compile(src, "d_main").expect("compile");
    let denied = console_authorize(&mir, &["platform.io.stdout".into()]).unwrap_err();
    assert_eq!(denied.code, "host.unauthorized");
    console_authorize(&mir, &platform_grants()).expect("authorized");

    let dir = temp_workdir("pio02");
    let pe = console_native_build(&mir, "d_main", &dir).expect("build");

    // Create a Win32 anonymous pipe, make the write end inheritable, close the read end,
    // and pass the write end as the child's stdout => WriteFile returns FALSE,
    // GetLastError() == 232 (ERROR_NO_DATA: pipe read end closed), zero libc/std::io involvement.
    let mut read_h: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut write_h: *mut core::ffi::c_void = core::ptr::null_mut();
    let ok = unsafe { CreatePipe(&mut read_h, &mut write_h, core::ptr::null(), 0) };
    assert_ne!(ok, 0, "CreatePipe");
    unsafe {
        // HANDLE_FLAG_INHERIT = 1
        let _ = SetHandleInformation(write_h, 1, 1);
        let _ = CloseHandle(read_h);
    }
    let write_owned = unsafe { OwnedHandle::from_raw_handle(write_h as _) };
    let status = Command::new(&pe.exe_path)
        .stdout(Stdio::from(write_owned))
        .stderr(Stdio::null())
        .status()
        .expect("spawn broken pipe child");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        status.code(),
        Some(232),
        "expected Win32 ERROR_NO_DATA (232) on read-closed pipe"
    );
}

#[test]
fn pio04_pe_import_audit_positive_and_negative_control() {
    // 1. p947-console remains clean with zero VirtualAlloc / CRT imports.
    let p947_mir = console_compile("fn main() int { print(42)\n return 0 }\n", "d_main").unwrap();
    let dir = temp_workdir("pio04");
    let p947_pe = console_native_build(&p947_mir, "d_main", &dir).unwrap();
    let p947_report = audit_pe_imports(&std::fs::read(&p947_pe.exe_path).unwrap()).unwrap();
    let p947_syms: Vec<&str> = p947_report
        .symbols
        .iter()
        .map(|(_, s)| s.as_str())
        .collect();
    assert!(!p947_syms.contains(&"VirtualAlloc"));
    assert!(!p947_syms.contains(&"VirtualFree"));
    let _ = std::fs::remove_dir_all(&dir);

    // 1b. Platform I/O + V-MEM PE uses ONLY KERNEL32.dll Win32 primitives, zero CRT/libc.
    let pio_src = r#"
fn main() int {
    let out = io_stdout()
    let _w = io_write_some(out, "hi")
    let _e = io_last_error()
    let _ps = mem_page_size()
    let m = mem_alloc(16)
    let _f = mem_free(m)
    return 0
}
"#;
    let pio_mir = console_compile(pio_src, "d_main").unwrap();
    let dir_pio = temp_workdir("pio04-plat");
    let pio_pe = console_native_build(&pio_mir, "d_main", &dir_pio).unwrap();
    let pio_report = audit_pe_imports(&std::fs::read(&pio_pe.exe_path).unwrap()).unwrap();
    assert_eq!(pio_report.dlls, vec!["KERNEL32.dll"]);
    let pio_syms: Vec<&str> = pio_report.symbols.iter().map(|(_, s)| s.as_str()).collect();
    for expected in [
        "ExitProcess",
        "GetStdHandle",
        "WriteFile",
        "GetLastError",
        "GetSystemInfo",
        "VirtualAlloc",
        "VirtualFree",
    ] {
        assert!(
            pio_syms.contains(&expected),
            "missing {expected} in {pio_syms:?}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir_pio);

    // 2. Negative control: auditing a real Windows binary (this Rust test executable)
    // that links CRT/ucrt/system DLLs must reject cleanly via pe.audit.forbidden-dll/symbol.
    let self_exe = std::fs::read(std::env::current_exe().unwrap()).unwrap();
    let err = audit_pe_imports(&self_exe).unwrap_err();
    assert!(
        err.code() == "mn.unsupported.pe.audit.forbidden-dll"
            || err.code() == "mn.unsupported.pe.audit.forbidden-symbol",
        "unexpected error: {err:?}"
    );
}

#[test]
fn vmem_native_os_region_bounds_borrow_and_real_win32_lifecycle() {
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

    if mem_write_u8(m, 5000, 1) != -4 { return -9 }
    if mem_read_u8(m, 5000) != -4 { return -10 }

    if mem_alloc(0) != -4 { return -11 }
    if mem_alloc(-5) != -4 { return -12 }

    let out = io_stdout()
    if mem_free(out) != -3 { return -13 }
    if mem_read_u8(out, 0) != -3 { return -14 }

    if mem_borrow(m) != 1 { return -15 }
    if mem_free(m) != -5 { return -16 }
    if mem_unborrow(m) != 0 { return -17 }
    if mem_unborrow(m) != -4 { return -18 }

    if mem_free(m) != 0 { return -19 }
    if mem_free(m) != -6 { return -20 }
    if mem_read_u8(m, 0) != -6 { return -21 }

    return 100
}
"#;
    let mir = console_compile(src, "d_main").expect("compile");
    console_authorize(&mir, &platform_grants()).expect("authorize");
    let dir = temp_workdir("vmem");
    let run = console_native_run(&mir, "d_main", &dir).expect("native run");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(run.exit_code, Some(100), "{run:?}");
}

#[test]
fn vmem_native_os_free_failure_preserves_owner_and_handle() {
    use cranelift_codegen::ir::{types, AbiParam, InstBuilder, Signature};
    use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
    use cranelift_module::{default_libcall_names, Linkage, Module};
    use cranelift_object::{ObjectBuilder, ObjectModule};

    // Emit a test-only object defining a failing `VirtualFree` returning FALSE (0)
    // and calling kernel32!SetLastError(487) to prove that `mn-platform-shim` keeps
    // the slot owner + mapping intact when the OS rejects VirtualFree.
    let isa = ace_native::emit::cranelift_sig_of;
    let _ = isa; // ensure emit types linked cleanly
    let triple: target_lexicon::Triple = ace_native::config::MN_TARGET.parse().unwrap();
    let mut b = cranelift_codegen::settings::builder();
    use cranelift_codegen::settings::Configurable;
    b.set("opt_level", "none").unwrap();
    b.set("is_pic", "false").unwrap();
    let flags = cranelift_codegen::settings::Flags::new(b);
    let target_isa = cranelift_codegen::isa::lookup(triple)
        .unwrap()
        .finish(flags)
        .unwrap();
    let builder =
        ObjectBuilder::new(target_isa, "test-fail-vfree", default_libcall_names()).unwrap();
    let mut om = ObjectModule::new(builder);
    let sle_sig = Signature {
        call_conv: cranelift_codegen::isa::CallConv::WindowsFastcall,
        params: vec![AbiParam::new(types::I32)],
        returns: vec![],
    };
    let sle_id = om
        .declare_function("SetLastError", Linkage::Import, &sle_sig)
        .unwrap();
    let vf_sig = Signature {
        call_conv: cranelift_codegen::isa::CallConv::WindowsFastcall,
        params: vec![
            AbiParam::new(types::I64),
            AbiParam::new(types::I64),
            AbiParam::new(types::I32),
        ],
        returns: vec![AbiParam::new(types::I32)],
    };
    let vf_id = om
        .declare_function("VirtualFree", Linkage::Export, &vf_sig)
        .unwrap();
    let mut ctx = om.make_context();
    ctx.func.signature = vf_sig;
    {
        let mut fb_ctx = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let blk = fb.create_block();
        fb.append_block_params_for_function_params(blk);
        fb.switch_to_block(blk);
        fb.seal_block(blk);
        let sle_ref = om.declare_func_in_func(sle_id, fb.func);
        let code487 = fb.ins().iconst(types::I32, 487);
        fb.ins().call(sle_ref, &[code487]);
        let zero = fb.ins().iconst(types::I32, 0);
        fb.ins().return_(&[zero]);
        fb.finalize();
    }
    om.define_function(vf_id, &mut ctx).unwrap();
    let fail_vf_obj = om.finish().emit().unwrap();

    let src = r#"
fn main() int {
    let m = mem_alloc(4096)
    if m <= 0 { return -1 }
    if mem_free(m) != -8 { return -2 }
    if io_last_error() != 487 { return -3 }
    if mem_write_u8(m, 0, 77) != 0 { return -4 }
    return mem_read_u8(m, 0)
}
"#;
    let mir = console_compile(src, "d_main").unwrap();
    let units = ace_av::cache::split_units(&mir).unwrap();
    let mut objects: Vec<ObjectInput> = units
        .iter()
        .enumerate()
        .map(|(i, u)| ObjectInput {
            name: format!("unit-{i}"),
            bytes: ace_native::emit::emit_console_unit_coff(u).unwrap().bytes,
        })
        .collect();
    objects.push(ObjectInput {
        name: "mn-platform-shim".into(),
        bytes: ace_native::platform_io::emit_platform_shim_coff().unwrap(),
    });
    objects.push(ObjectInput {
        name: "test-fail-vfree".into(),
        bytes: fail_vf_obj,
    });
    let entry_sym = ace_native::symbols::function_symbol(&mir.owner, "d_main");
    let wrapper = ace_native::console::emit_console_wrapper_coff(&entry_sym, true).unwrap();
    let dir = temp_workdir("vmem-fail");
    let pe = link_only(
        &objects,
        &wrapper,
        ace_native::config::START_SYMBOL,
        &dir,
        &[],
        "vmem-os-fail",
    )
    .unwrap();
    let out = Command::new(&pe.exe_path).output().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(out.status.code(), Some(77));
}

#[test]
fn vmem_native_internal_stdlib_helper_wrapper_roundtrip() {
    let helper = include_str!("../../../../../../stdlib/ace/internal/platform_mem.at");
    let src = format!(
        "{helper}\nfn main() int {{\n  let out = io_stdout()\n  let n = io_write_all(out, \"ok\\n\")\n  let m = mem_region_alloc(64)\n  if m <= 0 {{ return -1 }}\n  if mem_region_write_byte(m, 0, 99) != 0 {{ return -2 }}\n  let v = mem_region_read_byte(m, 0)\n  if mem_region_free(m) != 0 {{ return -3 }}\n  return v + n\n}}\n"
    );
    let mir = console_compile(&src, "d_main").expect("compile");
    console_authorize(&mir, &platform_grants()).expect("authorize");
    let dir = temp_workdir("internal-wrapper");
    let run = console_native_run(&mir, "d_main", &dir).expect("run");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(run.exit_code, Some(102));
    assert_eq!(run.stdout, b"ok\n");
}
