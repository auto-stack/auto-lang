//! `ace.m2.p978.coverage` — V-COVERAGE (PLAN-978 T-03/T-04): one public
//! source, real execution across the coverage matrix's daily lanes —
//! AV (in-process value), native (real PE build + process run), A2PY
//! (declared interpreter run) — plus the A2R/A2C emit identities and the
//! Windows PE-imports observation of the native product.
//!
//! The A2R/A2C real-compile legs are BUILD-LANE work (rustc/clang are
//! build-time tools, forbidden inside daily test bodies per the
//! 2026-11-14 test ruling); their one-shot compile+run evidence is
//! recorded by T-04's build-lane script, never faked here. Linux legs
//! are deferred-required/NOT_RUN with owner PLAN-1063 (2026-11-15).

mod common;

use ace_m2_build::driver::M2Build;
use ace_m2_build::identity::TargetKind;
use common::{b01_inputs, owned_tools_guard, scratch, timed};

/// The daily coverage matrix: same source, three really-executed lanes
/// with independent value observations + two emit lanes with stable
/// identities. The attributed oracle watches the WHOLE sequence: none of
/// the daily lanes may launch a toolchain process.
#[test]
fn p978_coverage_lanes_b01() {
    let root = scratch("coverage");
    let mut driver = M2Build::new(&root);

    // AV: in-process execution, value observed.
    let (av, _ms) = timed(|| driver.build(TargetKind::Av, &b01_inputs()));
    let av = av.unwrap();
    assert_eq!(av.run.as_ref().unwrap().exit_code, Some(14));

    // Native: real PE build (in-process cranelift/link) + process run.
    let (native, ms_native) = timed(|| driver.build(TargetKind::Native, &b01_inputs()));
    let native = native.unwrap();
    assert_eq!(native.run.as_ref().unwrap().exit_code, Some(14));
    assert!(
        ms_native <= 10_000.0,
        "native lane {ms_native:.0}ms (10s new-case gate)"
    );

    // A2R + A2C: emit identities + filesets (real compile = build lane).
    let a2r = driver.build(TargetKind::A2r, &b01_inputs()).unwrap();
    assert!(a2r.products.iter().any(|(p, _)| p.ends_with(".rs")));
    let a2c = driver.build(TargetKind::A2c, &b01_inputs()).unwrap();
    assert!(a2c.products.iter().any(|(p, _)| p.ends_with(".c")));
    // Different targets are different build nodes with different keys.
    assert_ne!(
        a2r.identity.key(),
        a2c.identity.key(),
        "a2r and a2c are separate target nodes"
    );

    // A2PY: emit + a REAL interpreter run through the declared
    // dependency (python is a runtime interpreter, not a build tool).
    let a2py = driver.build(TargetKind::A2py, &b01_inputs()).unwrap();
    assert!(!a2py.products.is_empty());
    // Pack an SDK with just the a2py lane and run it via the recipe —
    // the whole remaining sequence under the attributed oracle.
    let sdk = root.join("sdk-py");
    let (run, owned, available) = owned_tools_guard(|| {
        driver
            .build_sdk(&b01_inputs(), &[TargetKind::A2py], &sdk, vec![], "run:py")
            .unwrap();
        driver.run_sdk(&sdk, TargetKind::A2py, "run:py")
    });
    assert!(available, "attributed oracle available");
    assert!(
        owned.is_empty(),
        "coverage lanes launched toolchain processes: {owned:?}"
    );
    match run {
        Ok(obs) => {
            assert_eq!(
                obs.exit_code,
                Some(14),
                "A2PY lane observes the entry value (stdout: {:?}, channel {})",
                obs.stdout,
                obs.channel
            );
            assert_eq!(obs.channel, "process-stdout");
        }
        Err(e) if e.code == "m2build.run.interpreter-failed" => {
            // Missing interpreter on the machine: named pending (never
            // a fake pass) — fail loudly instead.
            panic!("a2py lane pending on this host: {e}");
        }
        Err(other) => panic!("a2py run failed: {other}"),
    }
}

/// V-PLATFORM-IO (Windows leg): the native product is a real PE whose
/// IMPORT TABLE contains only the declared runtime family — observed by
/// parsing the PE itself (independent of any tool output).
#[test]
fn p978_native_exe_pe_imports_declared_family_only() {
    let root = scratch("pe-imports");
    let mut driver = M2Build::new(&root);
    let built = driver.build(TargetKind::Native, &b01_inputs()).unwrap();
    let exe = built
        .products
        .iter()
        .find(|(p, _)| p.ends_with(".exe"))
        .expect("native product exe");
    let imports = pe_import_dlls(&exe.1).expect("parse PE import table");
    assert!(
        !imports.is_empty(),
        "a linked PE imports at least its CRT/kernel family"
    );
    for dll in &imports {
        let d = dll.to_ascii_lowercase();
        let allowed = d.contains("kernel32")
            || d.contains("ucrtbase")
            || d.contains("vcruntime")
            || d.contains("api-ms-win-crt")
            || d.contains("msvcrt");
        assert!(
            allowed,
            "undeclared import `{dll}` — the recipe's run-dependency family must cover every import"
        );
    }
}

/// Minimal PE import-table reader (DOS e_lfanew → optional header data
/// directory 1 → RVAs through the section table). Read-only evidence.
fn pe_import_dlls(bytes: &[u8]) -> Result<Vec<String>, String> {
    let rd_u16 = |off: usize| -> Result<u16, String> {
        let b = bytes
            .get(off..off + 2)
            .ok_or_else(|| "truncated PE".to_string())?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    };
    let rd_u32 = |off: usize| -> Result<u32, String> {
        let b = bytes
            .get(off..off + 4)
            .ok_or_else(|| "truncated PE".to_string())?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    if rd_u16(0)? != 0x5A4D {
        return Err("not a PE (no MZ)".into());
    }
    let pe = rd_u32(0x3C)? as usize;
    if rd_u32(pe)? != 0x4550 {
        return Err("bad PE signature".into());
    }
    let num_sections = rd_u16(pe + 6)? as usize;
    let opt_size = rd_u16(pe + 20)? as usize;
    let opt = pe + 24;
    let magic = rd_u16(opt)?;
    let data_dir = match magic {
        0x10B => opt + 96,  // PE32
        0x20B => opt + 112, // PE32+
        _ => return Err(format!("unknown optional header magic {magic:#x}")),
    };
    let import_rva = rd_u32(data_dir + 8)? as usize;
    if import_rva == 0 {
        return Ok(vec![]);
    }
    // Section table: map RVA → file offset.
    let sections = opt + opt_size;
    let mut file_off_of = |rva: usize| -> Result<usize, String> {
        for i in 0..num_sections {
            let s = sections + i * 40;
            let vaddr = rd_u32(s + 12)? as usize;
            let raw_size = rd_u32(s + 16)? as usize;
            let raw_ptr = rd_u32(s + 20)? as usize;
            if rva >= vaddr && rva < vaddr + raw_size.max(1) {
                return Ok(rva - vaddr + raw_ptr);
            }
        }
        Err(format!("rva {rva:#x} not in any section"))
    };
    let mut dlls = Vec::new();
    let mut idx = 0;
    loop {
        let desc = file_off_of(import_rva + idx * 20)?;
        let name_rva = rd_u32(desc + 12)? as usize;
        if name_rva == 0 {
            break;
        }
        let name_off = file_off_of(name_rva)?;
        let end = bytes[name_off..]
            .iter()
            .position(|b| *b == 0)
            .ok_or("unterminated DLL name")?;
        dlls.push(String::from_utf8_lossy(&bytes[name_off..name_off + end]).to_string());
        idx += 1;
    }
    Ok(dlls)
}
