//! PLAN-863: widening shared MIR must not enable native console imports.
#[test]
fn av_console_863_native_profile_and_fingerprint_reject_host_calls() {
    let p = auto_av_probe::console_host::compile("print(7)\n", "d_main").unwrap();
    let mut m = p.load().unwrap().module().clone();
    assert!(matches!(
        ac_mn::emit::admit_unit(&m),
        Err(ac_mn::MnError::Unsupported("profile", _))
    ));
    // Fingerprint formatting is independently callable, and must not silently omit a host table.
    m.is_unit = true;
    assert!(matches!(
        ac_mn::fingerprint::render_unit(&m),
        Err(ac_mn::MnError::Unsupported("host-console", _))
    ));
    for f in &mut m.funcs {
        for b in &mut f.blocks {
            b.instrs
                .retain(|i| !matches!(i, auto_av_core::mir::MirInstr::HostCall { .. }));
        }
    }
    assert!(matches!(
        ac_mn::fingerprint::render_unit(&m),
        Err(ac_mn::MnError::Unsupported("host-console", _))
    ));
}
