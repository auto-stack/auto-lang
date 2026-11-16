//! Plan 823/972: closure units on the native lanes — capture-free values
//! ride the collections lane (PLAN-972); ENV_GET bodies stay refused.
#[test]
fn closures_units_are_rejected_before_native_codegen() {
    let module = ace_av::closures_fixture::first_case_module();
    let units = ace_av::cache::split_units(&module).expect("split closures fixture");
    assert!(!units.is_empty());
    for unit in &units {
        // PLAN-961: the closures family is admitted on the numeric lane
        // (the frozen 823 exclusion is superseded by the numeric surface,
        // spec delta SD-01).
        ace_native::emit::admit_unit(unit)
            .expect("closures profile is admitted on the numeric lane");
        // PLAN-972 (T-03): capture-free closure VALUES (creation +
        // indirect call) now ACQUIRE a native fingerprint — they ride the
        // collections lane (cell-word callee address, cells-ABI indirect
        // call). The capturing-closure boundary stays named: a body unit
        // carrying ENV_GET still refuses its fingerprint.
        let has_env_get = unit.funcs.iter().any(|f| {
            f.blocks.iter().any(|b| {
                b.instrs
                    .iter()
                    .any(|i| matches!(i, ace_av_core::mir::MirInstr::EnvGet { .. }))
            })
        });
        if has_env_get {
            let error = ace_native::fingerprint::unit_fingerprint(unit)
                .expect_err("ENV_GET bodies must not acquire a native fingerprint");
            assert_eq!(error.code(), "mn.unsupported.instruction");
        } else {
            ace_native::fingerprint::unit_fingerprint(unit)
                .expect("capture-free closure units fingerprint (PLAN-972)");
        }
    }
}
