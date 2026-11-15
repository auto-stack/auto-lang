//! TEMPORARY probe (PLAN-981): does the AGE resource service now resolve, and
//! do the port types check? Deleted after the measurement.
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn probe(tag: &str, src: &str) {
    let caps = [PACKAGES_CAPABILITY.to_string()];
    match compile_age_package(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps,
    ) {
        Ok(_) => println!("PROBE {tag}: OK"),
        Err(ds) => println!("PROBE {tag}: {} | {}", ds[0].code, ds[0].message),
    }
}

#[test]
fn probe_service_resolution() {
    const OK: &str = "node N {\n    view { text \"t\" {} }\n}\n";
    probe(
        "svc-call",
        &format!("{OK}fn f() int {{\n    let a = material2d.color(0.9, 0.3, 0.2, 0.5)\n    return 1\n}}\n"),
    );
    probe(
        "svc-bad-arg-type",
        &format!("{OK}fn f() int {{\n    let a = material2d.color(1, 2, 3, 4)\n    return 1\n}}\n"),
    );
    probe(
        "svc-bad-arity",
        &format!("{OK}fn f() int {{\n    let a = material2d.color(0.9, 0.3)\n    return 1\n}}\n"),
    );
    probe(
        "svc-unknown-op",
        &format!("{OK}fn f() int {{\n    let a = material2d.nope(0.9)\n    return 1\n}}\n"),
    );
}
