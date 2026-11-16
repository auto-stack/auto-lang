//! PLAN-975 T-B1-V (r10) — the B1 primitive slice's independent oracles:
//! NUL scalar materialization (byte-exact 1-byte, not a terminator),
//! boundary scalar U+10FFFF (4-byte max), invalid scalars trap
//! CharInvalid at the typed gate, and the owner counterexample — the
//! materialization is a REAL fresh allocation (A2X_RT_FAULT_ALLOC=1
//! faults exactly it on the A2R exe). Console lane for the trap/NUL
//! faces, four ends + the fault leg for the owned face.
use ace_av_core::platform_io::Win32PlatformIo;

#[test]
fn json975_char_to_str_b1_slice_oracles() {
    // ---- console lane: NUL + boundary positives, invalid-scalar traps
    let pos = r#"fn mk(cp int) Str {
    return char.from_codepoint(cp).to(Str)
}
fn main() int {
    let z Str = mk(0)
    if z.len_bytes() != 1 { return 1 }
    if z.len() != 1 { return 2 }
    if z.char_at(0).to_codepoint() != 0 { return 3 }
    let azb Str = mk(97) + z + mk(98)
    if azb.len_bytes() != 3 { return 4 }
    if azb.char_at(1).to_codepoint() != 0 { return 5 }
    let mx Str = mk(1114111)
    if mx.len_bytes() != 4 { return 6 }
    if mx.char_at(0).to_codepoint() != 1114111 { return 7 }
    return 0
}"#;
    let mut backend = Win32PlatformIo::default();
    let grants: Vec<String> = vec!["platform.mem".into()];
    let program = ace_av::console_host::compile(pos, "d_main").expect("compile b1 slice");
    let (out, _t, _s) = program
        .run_platform(
            &grants,
            &ace_av_core::av::ExecLimits::default(),
            &ace_av_core::av::CancelFlag::new(false),
            None,
            &mut backend,
        )
        .expect("run platform b1 slice");
    assert!(
        matches!(
            &out,
            ace_av_core::contract::ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(0),
                ..
            }
        ),
        "NUL/boundary materialization oracles, got {out:?}"
    );
    for bad in [0xD800u32, 0x110000u32] {
        let neg = format!(
            "fn main() int {{\n    let c char = char.from_codepoint({bad})\n    return c.to(Str).len_bytes()\n}}"
        );
        let program = ace_av::console_host::compile(&neg, "d_main").expect("compile negative");
        let mut backend = Win32PlatformIo::default();
        let (out, _t, _s) = program
            .run_platform(
                &grants,
                &ace_av_core::av::ExecLimits::default(),
                &ace_av_core::av::CancelFlag::new(false),
                None,
                &mut backend,
            )
            .expect("run negative");
        assert!(
            matches!(
                &out,
                ace_av_core::contract::ExecOutcome::Trap {
                    kind: ace_av_core::contract::TrapKind::CharInvalid,
                    ..
                }
            ),
            "invalid scalar {bad:#x} traps CharInvalid before materialization, got {out:?}"
        );
    }

    // ---- four ends + the owner fault leg (packages chain)
    use ace_a2x_c::CTargetProvider;
    use ace_a2x_rust::RustTargetProvider;
    use ace_a2x_store::modules::packages::PackageA2xTarget;
    use ace_av::modules::packages::PackageAvTarget;
    use ace_core::source::cst::FrontBudget;
    use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
    use ace_source_query::modules::packages::{compile_package_product, stdlib_inputs};

    let src = r#"use std.text.byte_len
fn mk(cp int) Str {
    return char.from_codepoint(cp).to(Str)
}
fn main() int {
    let z Str = mk(0)
    let a Str = mk(65)
    let all Str = z + a + z
    return byte_len(all.to(str)) * 10 + byte_len(z.to(str))
}"#;
    let mut inputs = stdlib_inputs();
    inputs.push(PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: src.into(),
        provider: None,
    });
    let product = compile_package_product(
        &inputs,
        "app::main",
        &[PACKAGES_CAPABILITY.into()],
        FrontBudget::WORK_OWNED_64MIB,
    )
    .expect("compiles on the packages chain");

    let dir = std::env::temp_dir().join(format!("p975-b1-av-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // NUL(1) + A(1) + NUL(1) = 3 bytes; *10 + 1 = 31
    assert_eq!(
        PackageAvTarget::cold(&dir, &product).unwrap().run_value(),
        Ok(ace_av_core::contract::Value::I32(31))
    );
    let _ = std::fs::remove_dir_all(&dir);

    // A2R: normal run 31; then A2X_RT_FAULT_ALLOC=1 — the FIRST
    // allocation is mk(0)'s materialization, so the exe must FAULT
    // (the owned-buffer counterexample: it really allocates).
    let store = std::env::temp_dir().join(format!("p975-b1-rs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&store);
    let rust = PackageA2xTarget::cold(
        &store,
        "pkg975b1",
        "app",
        16 << 20,
        RustTargetProvider::v1(),
        &product,
    )
    .unwrap();
    let rust_dir = store.join("build");
    std::fs::create_dir_all(&rust_dir).unwrap();
    for (path, content) in &rust.current.files {
        std::fs::write(rust_dir.join(path), content).unwrap();
    }
    let rust_exe = rust_dir.join("pkg.exe");
    let mut rust_cmd =
        std::process::Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    rust_cmd
        .args(["--edition", "2021", "main.rs", "-o"])
        .arg(&rust_exe)
        .current_dir(&rust_dir);
    let rust_build = rust_cmd.output().expect("rustc spawn");
    assert!(
        rust_build.status.success(),
        "{}",
        String::from_utf8_lossy(&rust_build.stderr)
    );
    let ok_run = std::process::Command::new(&rust_exe)
        .current_dir(&rust_dir)
        .output()
        .expect("run rust exe");
    assert!(
        ok_run.status.success(),
        "{}",
        String::from_utf8_lossy(&ok_run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&ok_run.stdout).trim(), "value:31");
    let mut fault_cmd = std::process::Command::new(&rust_exe);
    fault_cmd
        .current_dir(&rust_dir)
        .env("A2X_RT_FAULT_ALLOC", "1");
    let fault_run = fault_cmd.output().expect("run fault exe");
    assert_eq!(
        String::from_utf8_lossy(&fault_run.stdout).trim(),
        "trap:Resource",
        "A2X_RT_FAULT_ALLOC=1 must fault the materialization allocation (the first owned buffer)"
    );
    let _ = std::fs::remove_dir_all(&store);

    // AC + native/mn exit code 31
    let c_store = std::env::temp_dir().join(format!("p975-b1-c-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&c_store);
    let c = PackageA2xTarget::cold(
        &c_store,
        "pkg975b1",
        "app",
        16 << 20,
        CTargetProvider::v1(),
        &product,
    )
    .unwrap();
    let c_dir = c_store.join("build");
    std::fs::create_dir_all(&c_dir).unwrap();
    for (path, content) in &c.current.files {
        std::fs::write(c_dir.join(path), content).unwrap();
    }
    std::fs::write(c_dir.join("manifest.json"), &c.current.manifest).unwrap();
    let c_exe = c_dir.join("pkg_c.exe");
    ace_a2x_worker::crun::compile_c_fileset(&c_dir, &c_exe)
        .unwrap_or_else(|e| panic!("cl failed: {e}"));
    let c_run = std::process::Command::new(&c_exe)
        .current_dir(&c_dir)
        .output()
        .expect("run c exe");
    assert!(
        c_run.status.success(),
        "{}",
        String::from_utf8_lossy(&c_run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&c_run.stdout).trim(), "value:31");
    let _ = std::fs::remove_dir_all(&c_store);

    let mn_dir = std::env::temp_dir().join(format!("p975-b1-mn-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&mn_dir);
    std::fs::create_dir_all(&mn_dir).unwrap();
    let mut driver = ace_native::driver::MnDriver::ephemeral();
    let built = driver
        .build_checked(
            &product.compiled.checked,
            "d_main",
            &mn_dir,
            None,
            None,
            975,
        )
        .expect("mn build");
    let code = std::process::Command::new(&built.exe_path)
        .current_dir(&mn_dir)
        .status()
        .expect("run mn exe")
        .code()
        .unwrap_or(-1);
    assert_eq!(code, 31);
    let _ = std::fs::remove_dir_all(&mn_dir);
}
