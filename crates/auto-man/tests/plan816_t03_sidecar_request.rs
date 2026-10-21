//! PLAN-816 T-03 F-05：sidecar 请求→PacSource 原回归（UI owner 域）。
//!
//! 前两条纯 adapter 回归在 auto-man-runtime；本文件承接需
//! `rust_ui::PacSource` 的两条，保持旧 oracle 不因闭包纯净被删。

use auto_man::rust_ui::PacSource;
use auto_man_runtime::generation_request_adapter::request_of;
use auto_man_runtime::Pac;

/// R3-01：请求侧 sidecar 声明真实驱动生成（来源=请求，磁盘不改产物）。
#[test]
fn sidecar_source_is_request_not_disk() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let front = root.join("src").join("front");
    std::fs::create_dir_all(&front).unwrap();
    std::fs::write(front.join("app.at"), "widget S { view { text \"s\" {} } }").unwrap();
    // 磁盘声明无 sidecar；请求手工带 sidecar
    std::fs::write(
        root.join("pac.at"),
        "name: \"sc\"
render: \"vm\"
",
    )
    .unwrap();
    std::fs::write(
        root.join("extra.rs"),
        "// sidecar module
",
    )
    .unwrap();
    let pac_path = root.join("pac.at");
    let config =
        auto_lang::config::AutoConfig::from_file(&pac_path, &auto_val::Obj::new()).unwrap();
    let pac = Pac::new(config);
    let mut req = request_of(&pac, &pac_path);
    // 请求注入 sidecar 声明（等价于另一磁盘 pac 声明 rust_sidecar）
    req.raw_nodes
        .push(auto_project_model::descriptor::RawNodeDecl {
            head: "rust_sidecar".into(),
            name: None,
            raw_args: None,
            props: vec![auto_project_model::descriptor::NodeProp {
                key: "modules".into(),
                value: auto_val::Value::str_array(vec![auto_val::AutoStr::from("extra:extra.rs")]),
                line: 0,
            }],
            children: vec![],
            line: 0,
        });
    let source = PacSource::from_request(&req);
    let spec = source.sidecar_spec(root);
    assert_eq!(spec.modules.len(), 1, "请求声明驱动（磁盘无 sidecar）");
    assert_eq!(spec.modules[0].0, "extra");
}

/// R5：sidecar 对象/嵌套 prop 形态经真实 pac→request→PacSource 全链
/// 不丢模块/依赖（R4-02 修复的回归锚——重建必须挂 root prop 而非
/// wrapper kid）。
#[test]
fn sidecar_obj_and_nested_prop_forms_survive_request() {
    let dir = tempfile::tempdir().unwrap();
    let root_dir = dir.path();
    std::fs::write(
        root_dir.join("extra.rs"),
        "// m
",
    )
    .unwrap();
    // 对象 prop 形态
    std::fs::write(
        root_dir.join("pac.at"),
        "name: \"obj-side\"
render: \"vm\"
rust_sidecar: {
    modules: [\"extra:extra.rs\"]
    deps: [\"libloading:0.8\"]
}
",
    )
    .unwrap();
    let pac_path = root_dir.join("pac.at");
    let config =
        auto_lang::config::AutoConfig::from_file(&pac_path, &auto_val::Obj::new()).unwrap();
    let pac = Pac::new(config);
    let req = request_of(&pac, &pac_path);
    assert!(
        req.raw.iter().any(|p| p.key == "rust_sidecar"),
        "对象形态进入 req.raw"
    );
    let spec = PacSource::from_request(&req).sidecar_spec(root_dir);
    assert_eq!(
        spec.modules.len(),
        1,
        "对象形态不丢模块：{:?}",
        spec.modules
    );
    assert_eq!(spec.deps.len(), 1, "对象形态不丢依赖：{:?}", spec.deps);
    assert_eq!(spec.modules[0].0, "extra");
    assert_eq!(spec.deps[0].0, "libloading");
}
