//! Plan 823: the scalar native consumer refuses the new profile.
#[test]
fn closures_units_are_rejected_before_native_codegen() {
    let module = ace_av::closures_fixture::first_case_module();
    let units = ace_av::cache::split_units(&module).expect("split closures fixture");
    assert!(!units.is_empty());
    for unit in &units {
        let error = ace_native::emit::admit_unit(unit)
            .expect_err("closures profile must not enter native codegen");
        assert_eq!(error.code(), "mn.unsupported.profile");
        let error = ace_native::fingerprint::unit_fingerprint(unit)
            .expect_err("closure instructions must not acquire a scalar native fingerprint");
        assert_eq!(error.code(), "mn.unsupported.instruction");
    }
}
