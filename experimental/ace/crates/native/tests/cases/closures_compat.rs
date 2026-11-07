//! Plan 823: the scalar native consumer refuses the new profile.
#[test]
fn closures_units_are_rejected_before_native_codegen() {
    let module = ace_av::closures_fixture::first_case_module();
    let units = ace_av::cache::split_units(&module).expect("split closures fixture");
    assert!(!units.is_empty());
    for unit in &units {
        // PLAN-961: the closures family is admitted on the numeric lane
        // (the frozen 823 exclusion is superseded by the numeric surface,
        // spec delta SD-01); the scalar FINGERPRINT lane still refuses
        // closure instructions by name.
        ace_native::emit::admit_unit(unit)
            .expect("closures profile is admitted on the numeric lane");
        let error = ace_native::fingerprint::unit_fingerprint(unit)
            .expect_err("closure instructions must not acquire a scalar native fingerprint");
        assert_eq!(error.code(), "mn.unsupported.instruction");
    }
}
