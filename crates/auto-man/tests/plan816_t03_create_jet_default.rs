//! PLAN-816 T-03 F-01：旧公开入口默认合同——`Automan::create_jet`
//! 无需调用方显式安装 UI ops（compat 自举）。显式安装仍是合法正控。

#[test]
fn create_jet_works_without_explicit_install() {
    // 若本测试进程已由 ctor 自举，此调用应成功并产出真实工程；
    // 不以 fake/仅编译充当产物。
    let root = std::env::temp_dir().join(format!("p816-t03-jet-no-install-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let result = auto_man::Automan::create_jet(root.to_str().unwrap());
    assert!(result.is_ok(), "旧入口无显式安装应成功：{:?}", result.err());
    assert!(root.is_dir(), "应产出真实目录");
    assert!(
        root.join("settings.gradle.kts").exists() || root.join("build.gradle.kts").exists(),
        "应产出 Jetpack 工程文件"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn create_jet_with_explicit_install_still_works() {
    auto_man::compat::install_default_ui_ops();
    let root = std::env::temp_dir().join(format!("p816-t03-jet-explicit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let result = auto_man::Automan::create_jet(root.to_str().unwrap());
    assert!(result.is_ok(), "显式安装正控应成功：{:?}", result.err());
    assert!(root.is_dir());
    let _ = std::fs::remove_dir_all(&root);
}
