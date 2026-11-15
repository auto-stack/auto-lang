//! TEMPORARY probe (PLAN-981): what is `e5` in the emitted document?
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{bind_packages_age, PackageModuleInput, PACKAGES_CAPABILITY};
use ace_core::source::age_component::ComponentScenario;

const RES: &str = include_str!("../fixtures_g01/resources.at");
const SHAPES: &str = include_str!("../fixtures_g01/shapes.at");
const MAIN: &str = include_str!("../fixtures_g01/main.at");

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

#[test]
fn probe_e5() {
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let product = bind_packages_age(
        &[input("resources", RES), input("shapes", SHAPES), input("main", MAIN)],
        "app::main",
        &caps,
        FrontBudget::WORK_OWNED_64MIB,
        &ComponentScenario { instances: 0, ops: Vec::new() },
    )
    .expect("bind");
    let resolved = ace_core::source::resolve_with(
        &product.bound.program,
        &caps,
        ace_core::source::SourceProfile::TextRecord,
    )
    .expect("resolve");
    let parts = ace_core::source::adapter::emit_resolved_packages_parts_profile(
        &resolved,
        &product.bound,
        "packages",
        ace_core::source::SourceProfile::AgePackage,
    )
    .expect("emit");
    let mut shown = 0;
    for line in parts.atom_text.lines() {
        let t = line.trim();
        if t.starts_with("expr e4") || t.starts_with("expr e5") || t.starts_with("expr e6")
            || t.starts_with("local l") || t.contains("param_mode")
        {
            println!("PROBE {t}");
            shown += 1;
            if shown > 12 { break; }
        }
    }
}
