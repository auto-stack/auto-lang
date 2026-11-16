
#[test]
fn p981_tmp_6i_probe() {
    use ace_core::source::age_h0::LegacyWidgetAlias;
    use ace_core::source::age_package::compile_age_package;
    use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
    let src = "node M {\n    msg Msg {\n        Add(str),\n        Set(int)\n    }\n    on {\n        .Add(t) -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let r = compile_age_package(
        &[PackageModuleInput { package: "app".into(), version: "1.0.0".into(), id: "main".into(), source: src.into(), provider: None }],
        "app::main", LegacyWidgetAlias::Reject, &[PACKAGES_CAPABILITY.to_string()]);
    match r { Ok(out) => { let c=&out.description.components[0]; println!("6I OK msgs={:?}", c.messages); println!("6I handlers={:?}", c.handler_msgs); println!("6I members={:?}", c.members.iter().map(|m| (m.role.as_str(), m.name.clone(), m.text.clone())).collect::<Vec<_>>()); }, Err(ds) => for d in ds.iter().take(3) { println!("6I ERR {} @{} {}", d.code, d.span.start, d.message.chars().take(70).collect::<String>()); } }
}
