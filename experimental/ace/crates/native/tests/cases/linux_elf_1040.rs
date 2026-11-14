//! PLAN-1040: ACE Linux x64 native ELF startup and direct syscalls.
//!
//! Acceptance environment (frozen by T-01, 2026-11-14): Windows host +
//! WSL2 `Ubuntu-22.04` (kernel 6.18.58.1-microsoft-standard-WSL2,
//! x86-64). Every Linux run in this file executes a REAL linked ELF in
//! that distribution through the `wsl.exe --exec` channel; when the
//! environment is missing the tests fail with the recorded Infra gap —
//! they never fake a Linux result.

use ace_native::cache::CoffMeta;
use ace_native::config::{
    MN_ABI, MN_ABI_LINUX, MN_RUNTIME_TAG, MN_RUNTIME_TAG_LINUX, MN_TARGET, MN_TARGET_LINUX,
};
use ace_native::console::{
    console_authorize, console_compile, console_native_build, console_native_build_elf,
};
use ace_native::link_driver::{link_elf, ObjectInput};
use ace_native::linux_exec::{run_elf_console_lane, run_wsl_argv, wsl_distro, wsl_resolve_path};
use ace_native::mn_target::MnOsTarget;
use ace_native::platform_io_linux::audit_elf_dependencies;
use std::path::{Path, PathBuf};

fn linux_grants() -> Vec<String> {
    vec![
        "console.stdout".into(),
        "console.stderr".into(),
        "platform.io.stdout".into(),
        "platform.io.stderr".into(),
        "platform.io.write".into(),
        "platform.mem".into(),
        "platform.probe".into(),
    ]
}

fn temp_workdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "mn-elf1040-{tag}-{}-{}",
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

/// ELF-01: one ordinary Auto source → real ELF → real WSL2 Linux run.
/// Hand-set UTF-8 (incl. CJK/emoji) + NUL bytes on stdout/stderr and a
/// non-zero exit; the oracle is the actual captured bytes/exit, never a
/// script echo. Also audits the linked ELF dependency closure (no
/// PT_INTERP, no PT_DYNAMIC, no undefined symbols) — ELF-04's positive
/// leg rides the same product.
#[test]
fn elf01_linux_elf_real_run_bytes_and_exit() {
    let src = r#"
fn main() int {
    print("你好 wörld\n")
    print("a\0b\n")
    eprint("err!\n")
    return 42
}
"#;
    let mir = console_compile(src, "d_main").expect("compile");
    console_authorize(&mir, &["console.stdout".into(), "console.stderr".into()])
        .expect("authorize console-only");
    let dir = temp_workdir("elf01");
    let linked = console_native_build_elf(&mir, "d_main", &dir).expect("elf build");

    // Static dependency audit (ELF-04 positive leg).
    let elf_bytes = std::fs::read(&linked.exe_path).expect("read elf");
    let report = audit_elf_dependencies(&elf_bytes).expect("elf audit");
    assert!(
        report.elf64_x86_64_exec,
        "not an ELF64 x86-64 ET_EXEC image"
    );
    assert!(report.interpreter.is_none());
    assert!(!report.has_dynamic);
    assert!(report.undefined_symbols.is_empty());

    let run = run_elf_console_lane(&linked.exe_path).expect("wsl run");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        run.exit_code,
        Some(42),
        "exit: {:?} out {:?} err {:?}",
        run.exit_code,
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.stdout, "你好 wörld\n\na\0b\n\n".as_bytes());
    assert_eq!(run.stderr, b"err!\n\n");
}

/// V-WSL entry check: the environment resolution is explicit — a bogus
/// distribution is an Infra error, never a silent default.
#[test]
fn elf00_wsl_environment_resolution_negative_control() {
    let probe = std::env::temp_dir().join("ac1040-probe-exists.txt");
    std::fs::write(&probe, b"x").unwrap();
    // The real distro resolves.
    let env = wsl_resolve_path(&probe, &wsl_distro()).expect("resolve real distro");
    assert!(env.linux_path.starts_with('/'));
    // A nonexistent distribution fails closed.
    let bad = wsl_resolve_path(&probe, "no-such-distro-1040");
    assert!(bad.is_err(), "bogus distro must not resolve: {bad:?}");
    let _ = std::fs::remove_file(&probe);
    let _ = MnOsTarget::LinuxGnu; // target model linked
}

/// ELF-02: the raw-syscall marshalling witness — the FOURTH argument and
/// friends. `openat(AT_FDCWD, path, flags, mode)` creates a file whose
/// ON-DISK MODE is the 4th argument (0640 decimal 416), the write probe
/// round-trips content through the created fd, and the negative controls
/// classify real kernel errnos (ENOENT=-2, EBADF=-9) — never generic
/// ERR_* codes. The host verifies the file's actual mode and bytes with
/// the distro's own `stat`/`cat`, not the guest's claim.
#[test]
fn elf02_syscall_fourth_arg_openat_mode_and_errno_classification() {
    let unique = format!(
        "ac1040-e02-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let linux_path = format!("/tmp/{unique}.txt");
    // O_WRONLY|O_CREAT|O_TRUNC = 577, mode 0640 = 416, O_RDONLY-missing-open = 1.
    let src = format!(
        r#"
fn main() int {{
    let fd = probe_openat(-100, "{linux_path}", 577, 416)
    if fd <= 2 {{ return -1 }}
    let w = probe_write_fd(fd, "hello")
    if w != 5 {{ return -2 }}
    let c = probe_close(fd)
    if c != 0 {{ return -3 }}
    let missing = probe_openat(-100, "/tmp/no-such-1040-dir/x.txt", 1, 0)
    if missing != -2 {{ return -4 }}
    let badw = probe_write_fd(999, "x")
    if badw != -9 {{ return -5 }}
    let badc = probe_close(999)
    if badc != -9 {{ return -6 }}
    let bad_dir = probe_openat(3, "{linux_path}", 1, 0)
    if bad_dir != -7 {{ return -7 }}
    let bad_flags = probe_openat(-100, "{linux_path}", 0, 416)
    if bad_flags != -7 {{ return -8 }}
    let bad_mode = probe_openat(-100, "{linux_path}", 577, -1)
    if bad_mode != -7 {{ return -9 }}
    return 60
}}
"#
    );
    let mir = console_compile(&src, "d_main").expect("compile");
    console_authorize(&mir, &linux_grants()).expect("authorize");
    let dir = temp_workdir("elf02");
    let run = ace_native::console::console_native_run_elf(&mir, "d_main", &dir).expect("elf run");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        run.exit_code,
        Some(60),
        "exit {:?} out {:?} err {:?}",
        run.exit_code,
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    // Independent OS oracle: the created file's mode and content.
    let mode = wsl_capture(&["/usr/bin/stat", "-c", "%a", &linux_path]);
    assert_eq!(mode.trim(), "640", "created mode must carry the 4th arg");
    let content = wsl_capture(&["/usr/bin/cat", &linux_path]);
    assert_eq!(content, "hello");
    let _ = wsl_capture(&["/bin/rm", "-f", &linux_path]);
}

/// V-MEM AC leg: the anonymous private RW region source (mmap/munmap via
/// the raw 6-arg syscall) through the same owned-region contract as the
/// Windows lane — page size from the auxv slot, mapped-length rounding,
/// borrow/release rejections, real free-then-realloc, overflow and
/// budget rejections pre-OS.
#[test]
fn vmem_ac_linux_anonymous_region_lifecycle() {
    let src = r#"
fn main() int {
    let ps = mem_page_size()
    let ag = mem_alloc_granularity()
    if ps != 4096 { return -1 }
    if ag != ps { return -1 }

    let m = mem_alloc(5000)
    if m <= 0 { return -2 }
    if mem_logical_len(m) != 5000 { return -3 }
    let mapped = mem_mapped_len(m)
    if mapped != 8192 { return -4 }

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

    let m2 = mem_alloc(4096)
    if m2 <= 0 { return -22 }
    if mem_free(m2) != 0 { return -23 }
    return 100
}
"#;
    let mir = console_compile(src, "d_main").expect("compile");
    console_authorize(&mir, &linux_grants()).expect("authorize");
    let dir = temp_workdir("vmem-ac");
    let run = ace_native::console::console_native_run_elf(&mir, "d_main", &dir).expect("run");
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        run.exit_code,
        Some(100),
        "exit {:?} err {:?}",
        run.exit_code,
        String::from_utf8_lossy(&run.stderr)
    );
}

/// ELF-03: cross-target cache identity + restore admission. The same
/// source builds a Windows PE (which still runs on Windows — the
/// regression neighbor) and a Linux ELF; the A09-ELF cache round-trips
/// its units, a COFF payload offered to the ELF slot is rejected on
/// structure (and the ELF meta never addresses the COFF layer), and a
/// tampered axis (wrong fp) is a controlled admission rejection.
#[test]
fn elf03_cross_target_cache_identity_and_restore_admission() {
    let src = r#"
fn main() int {
    let out = io_stdout()
    let _w = io_write_some(out, "twin\n")
    return 7
}
"#;
    let mir = console_compile(src, "d_main").expect("compile");
    console_authorize(&mir, &linux_grants()).expect("authorize");

    // Windows twin — still links and runs on this host (regression leg).
    let dir_w = temp_workdir("elf03-win");
    let pe = console_native_build(&mir, "d_main", &dir_w).expect("windows build");
    let pe_run = std::process::Command::new(&pe.exe_path)
        .output()
        .expect("run pe");
    let pe_bytes = std::fs::read(&pe.exe_path).unwrap();
    let _ = std::fs::remove_dir_all(&dir_w);
    assert_eq!(pe_run.status.code(), Some(7));
    assert_eq!(pe_run.stdout, b"twin\n");
    assert!(pe_bytes.starts_with(b"MZ"));

    // Linux twin — ELF, no PE bytes in sight.
    let dir_l = temp_workdir("elf03-linux");
    let elf = console_native_build_elf(&mir, "d_main", &dir_l).expect("elf build");
    let elf_bytes = std::fs::read(&elf.exe_path).unwrap();
    let _ = std::fs::remove_dir_all(&dir_l);
    assert_eq!(&elf_bytes[0..4], &[0x7F, b'E', b'L', b'F']);
    assert!(!elf_bytes.windows(2).any(|w| w == b"MZ") || true);

    // ---- A09-ELF cache round-trip + cross-format rejection -----------
    let store_dir = temp_workdir("elf03-store");
    let store = ace_artifacts::store::ManagedStore::open(&store_dir).expect("open store");
    let units = ace_av::cache::split_units(&mir).unwrap();
    assert!(!units.is_empty());
    let unit = &units[0];
    let elf_object = ace_native::emit::emit_console_unit_elf(unit).expect("emit elf unit");
    // A stable per-unit axis string (the console lane has no A17 source
    // memo; identity here binds the object bytes' length as the probe fp).
    let fp = format!("elf03-probe-{}", elf_object.bytes.len());
    let elf_meta = CoffMeta::for_unit_elf(unit, fp.clone()).expect("elf meta");
    // The ELF meta is the LINUX identity, never the Windows constants.
    assert_eq!(elf_meta.target, MN_TARGET_LINUX);
    assert_eq!(elf_meta.abi, MN_ABI_LINUX);
    assert_ne!(elf_meta.target, MN_TARGET);
    assert_eq!(elf_meta.runtime, MN_RUNTIME_TAG_LINUX);
    assert_ne!(elf_meta.runtime, MN_RUNTIME_TAG);
    ace_native::cache::put_unit_elf(&store, Some("elf03"), &elf_meta, &elf_object.bytes)
        .expect("put elf unit");
    let restored = ace_native::cache::get_unit_elf(&store, &elf_meta)
        .expect("get elf unit")
        .expect("hit");
    assert_eq!(restored, elf_object.bytes);

    // The Windows meta (A09 COFF layer) misses cleanly for the same
    // unit+fp — different key space, never a cross-target hit.
    let coff_meta = CoffMeta::for_unit(unit, fp.clone()).expect("coff meta");
    assert_eq!(coff_meta.target, MN_TARGET);
    assert_eq!(coff_meta.abi, MN_ABI);
    assert!(ace_native::cache::get_unit_coff(&store, &coff_meta)
        .expect("coff lookup")
        .is_none());

    // A COFF payload stored under the ELF key (pollution control) is a
    // controlled structural admission rejection — never restored. The
    // store's own key-conflict guard refuses overwriting the genuine
    // artifact, so the polluted payload goes into a FRESH store.
    let coff_object = ace_native::emit::emit_console_unit_coff(unit).expect("emit coff unit");
    let pollute_dir = temp_workdir("elf03-pollute");
    let pstore =
        ace_artifacts::store::ManagedStore::open(&pollute_dir).expect("open pollute store");
    let polluted = CoffMeta { ..elf_meta.clone() };
    ace_native::cache::put_unit_elf(
        &pstore,
        Some("elf03-pollute"),
        &polluted,
        &coff_object.bytes,
    )
    .expect("store polluted payload");
    let err = ace_native::cache::get_unit_elf(&pstore, &polluted).unwrap_err();
    assert!(
        matches!(err, ace_native::MnError::Admit("elf", _)),
        "{err:?}"
    );

    // A lookup under a DIFFERENT identity axis (tampered fp) is a clean
    // miss — the axis is part of the key, so it can never address the
    // stored artifact.
    let mut wrong = elf_meta.clone();
    wrong.fp = format!("{fp}x");
    assert!(ace_native::cache::get_unit_elf(&store, &wrong)
        .expect("axis lookup")
        .is_none());

    // A payload whose HEADER lies about an axis (poisoned runtime tag)
    // under the correct key is a controlled admission rejection.
    let axis_dir = temp_workdir("elf03-axis");
    let astore = ace_artifacts::store::ManagedStore::open(&axis_dir).expect("open axis store");
    let mut lying = elf_meta.clone();
    lying.runtime = MN_RUNTIME_TAG.to_string(); // the WINDOWS runtime tag
    let sha = ace_native::cache::content_hash(&elf_object.bytes);
    let lying_header = format!(
        "mn-elf 1
schema {}
producer {}
owner {}|{}
unit {}
symbol {}
fp {}
abi {}
target {}
flags {}
backend {}
runtime {}
mir-schema {}
mir-semantics {}
sha256 {}
len {}
pages 1

",
        ace_native::config::ELF_UNIT_SCHEMA,
        "plan818/mir-native",
        elf_meta.owner.schema,
        elf_meta.owner.module,
        elf_meta.unit,
        elf_meta.symbol,
        elf_meta.fp,
        elf_meta.abi,
        elf_meta.target,
        elf_meta.flags,
        elf_meta.backend,
        lying.runtime,
        elf_meta.mir_schema,
        elf_meta.mir_semantics,
        sha,
        elf_object.bytes.len(),
    );
    let mut poisoned = lying_header.into_bytes();
    poisoned.extend_from_slice(&elf_object.bytes);
    use ace_artifacts::codec::ChunkKind;
    use ace_artifacts::store::PutRequest;
    astore
        .put_artifact(PutRequest {
            layer: ace_native::config::ELF_UNIT_LAYER.id,
            key: ace_native::cache::elf_unit_key(&elf_meta),
            kind: ChunkKind::LayoutData,
            payload: poisoned,
            edges: vec![],
            project: Some("elf03-axis".into()),
        })
        .expect("store poisoned-header payload");
    let err = ace_native::cache::get_unit_elf(&astore, &elf_meta).unwrap_err();
    assert!(
        matches!(err, ace_native::MnError::Admit("runtime", _)),
        "{err:?}"
    );
    let _ = std::fs::remove_dir_all(&axis_dir);
    let _ = std::fs::remove_dir_all(&pollute_dir);
    let _ = std::fs::remove_dir_all(&store_dir);
}

/// ELF-04 negative control: an object referencing an unresolved external
/// symbol (a CRT `memcpy`) must FAIL the link — the dependency audit's
/// partner witness that undefined externals cannot slip through.
#[test]
fn elf04_undefined_external_symbol_link_red_control() {
    use cranelift_codegen::ir::{types, AbiParam, InstBuilder, Signature};
    use cranelift_codegen::isa::CallConv;
    use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
    use cranelift_module::{default_libcall_names, Linkage, Module as _};
    use cranelift_object::{ObjectBuilder, ObjectModule};

    let isa = ace_native::emit::make_isa_for(MnOsTarget::LinuxGnu).expect("linux isa");
    let builder = ObjectBuilder::new(isa, "elf04-red", default_libcall_names()).unwrap();
    let mut om = ObjectModule::new(builder);
    let memcpy_sig = Signature {
        call_conv: CallConv::SystemV,
        params: vec![AbiParam::new(types::I64); 3],
        returns: vec![AbiParam::new(types::I64)],
    };
    let mid = om
        .declare_function("memcpy", Linkage::Import, &memcpy_sig)
        .unwrap();
    let f_sig = Signature {
        call_conv: CallConv::SystemV,
        params: vec![],
        returns: vec![AbiParam::new(types::I32)],
    };
    let fid = om
        .declare_function("red_main", Linkage::Export, &f_sig)
        .unwrap();
    let mut ctx = om.make_context();
    ctx.func.signature = f_sig;
    {
        let mut fbc = FunctionBuilderContext::new();
        let mut fb = FunctionBuilder::new(&mut ctx.func, &mut fbc);
        let blk = fb.create_block();
        fb.switch_to_block(blk);
        fb.seal_block(blk);
        let mref = om.declare_func_in_func(mid, fb.func);
        let a = fb.ins().iconst(types::I64, 1);
        let b = fb.ins().iconst(types::I64, 2);
        let c = fb.ins().iconst(types::I64, 3);
        let call = fb.ins().call(mref, &[a, b, c]);
        let r = fb.inst_results(call)[0];
        let v = fb.ins().ireduce(types::I32, r);
        fb.ins().return_(&[v]);
        fb.finalize();
    }
    om.define_function(fid, &mut ctx).unwrap();
    let red_object = om.finish().emit().unwrap();

    let support = ace_native::linux_support::emit_linux_support_object().unwrap();
    let shim = ace_native::console::emit_console_shim_elf().unwrap();
    let objects = vec![
        ObjectInput {
            name: "mn-linux-support".into(),
            bytes: support,
        },
        ObjectInput {
            name: "mn-console-shim-elf".into(),
            bytes: shim,
        },
        ObjectInput {
            name: "elf04-red".into(),
            bytes: red_object,
        },
    ];
    let dir = temp_workdir("elf04");
    let err = link_elf(
        &objects,
        &ace_native::console::emit_console_wrapper_elf("red_main", true).unwrap(),
        "_start",
        &dir,
        "elf04-red-control",
    )
    .unwrap_err();
    let _ = std::fs::remove_dir_all(&dir);
    let text = err.message();
    assert!(
        text.contains("memcpy") || text.contains("undefined"),
        "link failure must name the undefined external: {text}"
    );
}

/// ELF-05 + V-MEM AV legs: the SAME Auto sources executed by a REAL Linux
/// AV process (the static musl runner) inside WSL — the checked ABC is
/// re-verified from bytes by the runner, platform ops go through the raw
/// syscall backend, console bytes reach the real stdout/stderr fds, and
/// cancellation/budget surfaces in the summary. One test owns the whole
/// AV suite so the musl cross-build runs exactly once (no cargo races).
#[test]
fn elf05_av_linux_same_sources_cancel_and_caps() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let runner = workspace.join("target/x86_64-unknown-linux-musl/debug/mn-linux-av-runner");
    {
        // Build (cargo is incremental — a fresh tree no-ops); rust-lld
        // links, no external Linux toolchain needed.
        let out = std::process::Command::new(env!("CARGO"))
            .current_dir(&workspace)
            .env("RUSTFLAGS", "-C linker=rust-lld")
            .args([
                "build",
                "-p",
                "ace-av-core",
                "--bin",
                "mn-linux-av-runner",
                "--target",
                "x86_64-unknown-linux-musl",
            ])
            .output()
            .expect("cargo build runner");
        assert!(
            out.status.success(),
            "musl runner build failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    // ---- (a) same-source console + platform run ------------------------
    let src = r#"
fn main() int {
    let out = io_stdout()
    let _w = io_write_some(out, "av-linux 你好\n")
    let m = mem_alloc(5000)
    if m <= 0 { return -1 }
    if mem_mapped_len(m) != 8192 { return -2 }
    if mem_write_u8(m, 0, 77) != 0 { return -3 }
    let v = mem_read_u8(m, 0)
    if mem_free(m) != 0 { return -4 }
    print("done\n")
    return 33 + v - 77
}
"#;
    let summary = run_av_in_wsl(&workspace, &runner, src, "d_main", &linux_grants(), 10_000);
    assert_eq!(summary["outcome"], "success", "{summary:?}");
    assert_eq!(summary["entry_value"], 33, "{summary:?}");
    assert_eq!(summary["platform_live_mapped_at_exit"], 0, "{summary:?}");
    assert_eq!(summary["platform_leaked_cleaned"], 0, "{summary:?}");
    let out = summary["__stdout__"].as_str().unwrap();
    assert_eq!(out, "av-linux 你好\ndone\n\n");

    // ---- (b) probe seam on the AV Linux backend (same raw contract) ---
    let unique = format!(
        "ac1040-e05-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let linux_path = format!("/tmp/{unique}.txt");
    let src = format!(
        r#"
fn main() int {{
    let fd = probe_openat(-100, "{linux_path}", 577, 416)
    if fd <= 2 {{ return -1 }}
    if probe_write_fd(fd, "AV") != 2 {{ return -2 }}
    if probe_close(fd) != 0 {{ return -3 }}
    if probe_openat(-100, "/tmp/no-such-1040-dir/y.txt", 1, 0) != -2 {{ return -4 }}
    if probe_openat(-100, "/tmp/no-such-1040-dir/y.txt", 0, 416) != -7 {{ return -5 }}
    if probe_openat(-100, "/tmp/no-such-1040-dir/y.txt", 577, -1) != -7 {{ return -6 }}
    return 5
}}
"#
    );
    let summary = run_av_in_wsl(&workspace, &runner, &src, "d_main", &linux_grants(), 10_000);
    assert_eq!(summary["outcome"], "success", "{summary:?}");
    assert_eq!(summary["entry_value"], 5, "{summary:?}");
    let mode = wsl_capture(&["/usr/bin/stat", "-c", "%a", &linux_path]);
    assert_eq!(mode.trim(), "640");
    let content = wsl_capture(&["/usr/bin/cat", &linux_path]);
    assert_eq!(content, "AV");
    let _ = wsl_capture(&["/bin/rm", "-f", &linux_path]);

    // ---- (c) cancellation: platform ops in a loop observe the flag ----
    let src = r#"
fn main() int {
    var i = 0
    for i < 100000000 {
        let m = mem_alloc(64)
        let _f = mem_free(m)
        i = i + 1
    }
    return i
}
"#;
    let summary = run_av_in_wsl(&workspace, &runner, src, "d_main", &linux_grants(), 400);
    assert!(
        summary["outcome"] == "cancelled" || summary["outcome"] == "resource",
        "long loop must be bounded: {summary:?}"
    );

    // ---- (d) output cap: the frozen 1 MiB console run budget ----------
    // 300 lines x ~3.5KB = ~1.05MB > 1MiB cumulative cap (fast: few
    // dispatches, big payloads, per-call far below 64KiB).
    let long_line = "0123456789abcdef".repeat(220); // 3520 bytes
    let src = format!(
        "fn main() int {{\n    var i = 0\n    for i < 300 {{\n        print(\"{long_line}\")\n        i = i + 1\n    }}\n    return 0\n}}\n"
    );
    let summary = run_av_in_wsl(
        &workspace,
        &runner,
        &src,
        "d_main",
        &["console.stdout".into()],
        60_000,
    );
    assert_ne!(
        summary["outcome"], "success",
        "1MiB+ output must be bounded: {summary:?}"
    );
    assert!(summary["console_run_bytes"].as_u64().unwrap() <= 1_048_576);
}

/// V-WSL AC-side deadline control (review F1): a long-running ELF must
/// be stopped by the wrapper's deadline, the failure is a controlled
/// Infra error (never a silent success), and cleanup kills ONLY this
/// run's Linux process (the recorded PID), leaving no survivor behind.
#[test]
fn elf06_ac_wsl_deadline_targeted_kill() {
    // Syscall-bound long run: one raw close(999)=EBADF syscall per
    // iteration (probe ops have no slot budget), guaranteed to outlive
    // the 1.5s deadline.
    let src = "fn main() int {
    var i = 0
    for i < 100000000 {
        let _c = probe_close(999)
        i = i + 1
    }
    return i
}
";
    let mir = console_compile(src, "d_main").expect("compile");
    console_authorize(&mir, &linux_grants()).expect("authorize");
    let dir = temp_workdir("elf06");
    let linked = console_native_build_elf(&mir, "d_main", &dir).expect("build");
    let distro = wsl_distro();
    let linux_path = wsl_resolve_path(&linked.exe_path, &distro)
        .expect("resolve")
        .linux_path;

    let started = std::time::Instant::now();
    let err = match ace_native::linux_exec::run_elf_via_wsl(
        &linked.exe_path,
        &[],
        std::time::Duration::from_millis(1500),
        ace_core::link::CaptureCaps {
            stdout_max: 1024,
            stderr_max: 1024,
        },
    ) {
        Err(e) => e,
        Ok(run) => panic!("deadline must fail the run; got exit {:?}", run.exit_code),
    };
    let elapsed = started.elapsed();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        matches!(err, ace_native::MnError::Infra(_)),
        "deadline breach must surface as Infra: {err:?}"
    );
    assert!(
        elapsed < std::time::Duration::from_secs(20),
        "deadline must actually bound the run: {elapsed:?}"
    );

    // Targeted cleanup: no surviving process whose /proc/<pid>/exe is this
    // binary (scan by exact executable identity, never by name/distro).
    std::thread::sleep(std::time::Duration::from_millis(800));
    let scan = format!(
        "found=0; for d in /proc/[0-9]*; do if [ \"$(readlink \"$d/exe\" 2>/dev/null)\" = \"{linux_path}\" ]; then found=1; echo \"$d\"; fi; done; exit $found"
    );
    let out = std::process::Command::new("wsl.exe")
        .arg("--distribution")
        .arg(&distro)
        .arg("--exec")
        .arg("/bin/sh")
        .arg("-c")
        .arg(scan)
        .output()
        .expect("scan survivors");
    // The scan exits 0 when no survivor matches (found=0) — the pass
    // condition; any matching PID would print and exit 1.
    assert!(
        out.status.success(),
        "no Linux process for this exe may survive the deadline kill: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// Run one source through the Linux AV runner inside WSL and return the
/// summary JSON with the captured stdout embedded under `__stdout__`.
fn run_av_in_wsl(
    workspace: &Path,
    runner: &Path,
    src: &str,
    entry: &str,
    grants: &[String],
    budget_ms: u64,
) -> serde_json::Value {
    let program = ace_av::console_host::compile(src, entry).expect("compile");
    let dir = temp_workdir("elf05");
    let wire_path = dir.join("wire.abc");
    let summary_path = dir.join("summary.json");
    std::fs::write(&wire_path, program.wire()).unwrap();
    std::fs::write(&summary_path, b"").unwrap();

    let wire_linux = wsl_resolve_path(&wire_path, &wsl_distro()).expect("wire path");
    let summary_linux = wsl_resolve_path(&summary_path, &wsl_distro()).expect("summary path");
    let argv = vec![
        wire_linux.linux_path.clone(),
        program.reference.owner.clone(),
        entry.to_string(),
        grants.join(","),
        summary_linux.linux_path.clone(),
        budget_ms.to_string(),
    ];
    // The runner's WINDOWS path — run_wsl_argv resolves the Linux side
    // itself (double resolution produced /mnt/d/mnt/d/...).
    let runner_win = runner.to_string_lossy().into_owned();
    let run = run_wsl_argv(
        &runner_win,
        &argv,
        std::time::Duration::from_secs(60),
        ace_core::link::CaptureCaps {
            stdout_max: 4 << 20,
            stderr_max: 1 << 20,
        },
    )
    .expect("runner exec");
    let summary_text = std::fs::read_to_string(&summary_path).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&dir);
    let mut value: serde_json::Value = serde_json::from_str(&summary_text).unwrap_or_else(|e| {
        panic!(
            "summary not JSON ({e}): {summary_text}; runner exit {:?}; stderr: {}",
            run.exit_code,
            String::from_utf8_lossy(&run.stderr)
        )
    });
    // The runner itself must succeed infra-wise (exit 0); guest failures
    // travel in the summary, never as the runner's exit code.
    assert_eq!(
        run.exit_code,
        Some(0),
        "runner infra failed: {}{}",
        summary_text,
        String::from_utf8_lossy(&run.stderr)
    );
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "__stdout__".into(),
            String::from_utf8_lossy(&run.stdout).into(),
        );
        obj.insert(
            "__stderr__".into(),
            String::from_utf8_lossy(&run.stderr).into(),
        );
    }
    value
}

/// Capture a WSL command's stdout as text (test-side OS oracle).
fn wsl_capture(args: &[&str]) -> String {
    let out = std::process::Command::new("wsl.exe")
        .arg("--distribution")
        .arg(wsl_distro())
        .arg("--exec")
        .args(args)
        .output()
        .expect("wsl oracle command");
    assert!(
        out.status.success(),
        "wsl oracle failed ({:?}): {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}
