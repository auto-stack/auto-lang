//! PLAN-809 T-03 V-01：公开 API 兼容 probe + core 生产委托对拍。
//!
//! - `api_compat_probe`：以字面量构造 `Pac{...全字段...}` 与
//!   `Target{...全字段...}`、调用 pin 的公开方法——证明字段构造与方法
//!   签名在 core 迁移（重导出门面 + 扩展 trait）后不被破坏
//!   （contract-baseline.md §8.2）。
//! - `production_delegation_parity`：`Pac::new`（现经
//!   `auto_man_core::pac_from_decl`）与旧臂冻结 oracle（AutoConfig 全量
//!   解析断言）逐字段对拍。
//! - `glob_import_method_syntax`：`use auto_man::*` 下扩展 trait 方法
//!   语法可用（t.scan 类效果方法的兼容半——此处只验证解析期可用性，
//!   不执行 FS）。

use auto_man::*;
use auto_val::AutoStr;

#[test]
fn api_compat_probe() {
    // ---- Target 全字段字面量构造（字段面冻结）----
    let mut t = Target {
        name: "app1".into(),
        version: "1.0".into(),
        kind: TargetKind::App,
        lang: "c".into(),
        origin: TargetOrigin::Local,
        from: "../x".into(),
        at: "apps/app1".into(),
        rename: "app1".into(),
        is_scan: true,
        recurse: false,
        show_headers: false,
        dirs: Default::default(),
        autos: vec![],
        srcs: Default::default(),
        incs: Default::default(),
        selects: Default::default(),
        skips: Default::default(),
        links: vec![],
        is_resolved: false,
        is_main: false,
        props: Default::default(),
        deps: vec![],
        dep_names: vec![],
        all_links: vec![],
        all_deps: vec![],
        rel: "".into(),
        defines: vec![],
        port: Port::default(),
    };
    // pin 的公开方法（core 单源，经门面调用）
    assert_eq!(t.local_name(), "app1");
    assert_eq!(t.config_name(), "pac.at");
    assert!(t.is_major());
    assert!(t.has_auto() == false);
    t.set_rel("/..".into());
    t.set_defines(vec!["D".into()]);
    let _ = t.to_atom();
    let _ = t.to_node();
    assert!(t.get_dir("none").is_none());
    let _ = t.libname();
    let _ = t.parent();
    let _ = t.location();
    let mut other = t.clone();
    other.props.set("k", auto_val::Value::Str("v".into()));
    t.merge(&other);
    let mut link = auto_val::Node::new("link");
    link.args.add_pos(auto_val::Value::Str("dep".into()));
    t.merge_link(&link);
    t.links.push(link.clone());
    let links = t.check_links().unwrap();
    assert_eq!(links.len(), 1);
    t.apply_link(&links[0]);
    t.set_parent("deps");
    assert_eq!(t.at, "deps/app1");

    // ---- Pac 纯投影方法（旧公开签名经 core 委托）----
    let config = auto_lang::config::AutoConfig::new(
        "name: \"p1\"\nrender: { front: \"vue\", back: \"rust\" }\napp(\"main\") {}\nlib(\"l\") {}\n",
    )
    .unwrap();
    let mut pac = Pac::new(config);
    assert_eq!(pac.frontend_types(), vec![BackendType::Vue]);
    assert_eq!(pac.backend_type(), Some(BackendType::Rust));
    assert!(pac.has_backend_config());
    assert!(!pac.has_multi_backend());
    assert_eq!(pac.default_frontend(), Some(BackendType::Vue));
    let dirs = pac.output_dirs();
    assert!(dirs.contains(&(BackendType::Vue, "vue".into())));
    assert!(dirs.contains(&(BackendType::Rust, "back".into())));
    assert_eq!(
        pac.frontend_output_dir(&BackendType::Vue),
        AutoStr::from("vue")
    );
    assert!(!pac.is_workspace());
    assert!(!pac.is_ui());
    assert_eq!(pac.workspace_members(), Vec::<AutoStr>::new().as_slice());
    assert_eq!(pac.apps().len(), 1);
    assert_eq!(pac.libs().len(), 1);
    assert_eq!(pac.build_targets().len(), 2);
    assert_eq!(pac.deps().len(), 0);
    assert!(pac.contains(&"main".into()));
    assert!(pac.get_target("main").is_some());
    assert_eq!(
        pac.copy_target(&AutoStr::from("l")).unwrap().kind,
        TargetKind::Lib
    );
    let no_ports: Vec<AutoStr> = pac.list_port_names();
    assert_eq!(no_ports, Vec::<AutoStr>::new());
    assert!(pac.all_incs().is_empty());
    let _ = pac.to_atom();
    let _ = pac.to_node();
    assert_eq!(pac.relative_path(), "");
    assert_eq!(pac.exe_path(), "build/Debug/main");
    assert!(!pac.has_device_prop("any"));
    pac.set_show_headers();
    assert!(pac.show_headers);

    // targets_map 消费 targets（mem::take 语义冻结）——放在本块末尾
    let names = Pac::get_build_names(&pac.targets_map());
    assert_eq!(names.len(), 2);
    assert!(pac.targets.is_empty(), "targets_map 消费语义保持");

    // 静态闭包函数族可从 core 公开面直接消费（同类型身份）
    let map = auto_man_core::targets_map(vec![Target::new("a", TargetKind::App)]);
    assert_eq!(auto_man_core::get_build_names(&map).len(), 1);
}

/// 生产委托对拍：Pac::new 的全部归一字段逐项断言（冻结 oracle）。
#[test]
fn production_delegation_parity() {
    let config = auto_lang::config::AutoConfig::new(
        r#"
        name: "demo"
        version: "0.2.0"
        scene: "ui"
        render: "vue"
        api: "rust"
        title: "Demo"
        title_zh: "演示"
        icon: "app-window"
        category: "tool"
        theme: "dark"
        accent: "coral"
        desktop_render: "queue"
        ui_config: "auto-edit.at"
        media_root: "E:/Video"
        photo_root: "E:/Pics"
        window: "1024x768"
        front_port: 3018
        back_port: 8018
        members: ["m1"]
        back: "../ash-server"
        shadcn: off
        default_classes: off
        defines: ["A"]
        dep "common" {
            path: "../common"
        }
        "#,
    )
    .unwrap();
    let pac = Pac::new(config);
    assert_eq!(pac.name, "demo");
    assert_eq!(pac.version, "0.2.0");
    assert_eq!(pac.scene, Scene::Ui);
    assert_eq!(pac.backend, "vue");
    assert_eq!(pac.api.as_deref(), Some("rust"));
    assert_eq!(pac.title.as_deref(), Some("Demo"));
    assert_eq!(pac.title_zh.as_deref(), Some("演示"));
    assert_eq!(pac.display_title_in(true).as_deref(), Some("演示"));
    assert_eq!(pac.icon.as_deref(), Some("app-window"));
    assert_eq!(pac.category.as_deref(), Some("tool"));
    assert_eq!(pac.theme.as_deref(), Some("dark"));
    assert_eq!(pac.accent.as_deref(), Some("coral"));
    assert_eq!(pac.desktop_render.as_deref(), Some("queue"));
    assert_eq!(pac.ui_config.as_deref(), Some("auto-edit.at"));
    assert_eq!(pac.media_root.as_deref(), Some("E:/Video"));
    assert_eq!(pac.photo_root.as_deref(), Some("E:/Pics"));
    assert_eq!(pac.window, Some((1024.0, 768.0)));
    assert!(!pac.window_fit);
    assert_eq!(pac.front_port, Some(3018));
    assert_eq!(pac.back_port, Some(8018));
    assert_eq!(pac.members, vec![AutoStr::from("m1")]);
    assert_eq!(pac.external_backend.as_deref(), Some("../ash-server"));
    assert!(!pac.shadcn);
    assert!(!pac.default_classes);
    assert_eq!(pac.defines, vec![AutoStr::from("A")]);
    assert_eq!(pac.targets.len(), 1);
    assert_eq!(pac.targets[0].origin, TargetOrigin::Local);
    assert_eq!(pac.targets[0].from, "../common");
    assert_eq!(pac.targets[0].at, "deps/common");
    // lang 缺省 c 时 target 不继承 lang 字段（旧臂规则）
    assert_eq!(pac.targets[0].lang, "c");
    // 缺省 port builder（非 rust/ts lang → ninja）
    assert_eq!(pac.port.builder, "ninja");
    // lang rust → cargo（default_port_builder 规则）
    let pac2 =
        Pac::new(auto_lang::config::AutoConfig::new("name: \"r\"\nlang: \"rust\"\n").unwrap());
    assert_eq!(pac2.port.builder, "cargo");
    assert_eq!(pac2.backend, "rust");
}

/// 扩展 trait 方法语法在 glob 导入下可用（兼容半证明；仅解析期）。
#[test]
fn glob_import_method_syntax() {
    let t = Target::new("x", TargetKind::App);
    // TargetEffects 经 use auto_man::* 可达：调用一个无副作用方法
    assert!(!t.exists() || t.is_main);
    let d = Dir::from_str("src".into(), ".".into(), TargetKind::App);
    assert_eq!(d.rpath, "src");
}

/// merge_port 解析委托 core::port_from_node 后的 Port 形状（含编译器默认）。
#[test]
fn port_from_node_parity() {
    let config = auto_lang::config::AutoConfig::new(
        "name: \"p\"\nport win32 {\n    platform: \"windows\"\n    at: \"build/win\"\n    toolchain: \"msvc\"\n}\n",
    )
    .unwrap();
    let pac = Pac::new(config);
    assert_eq!(pac.ports.len(), 1);
    let port = auto_man_core::port_from_node(&pac.ports[0]);
    assert_eq!(port.name, "win32");
    assert_eq!(port.builder, "ninja");
    assert_eq!(port.platform, "windows");
    assert_eq!(port.at, "build/win");
    assert_eq!(port.toolchain.as_deref(), Some("msvc"));
    // port_at = build/win；去尾斜杠规则在 set_port（效果半）
    assert_eq!(port.at, "build/win");
}

/// V-04（auto-man 级）：Pac::resolve 在内部 target 环上有界报错而非
/// 旧臂 while 挂死；错误具名环成员且不开始 scan（App/Lib 环不触发
/// download，隔离验证闭包段）。
#[test]
fn resolve_internal_cycle_bounded_error() {
    let config = auto_lang::config::AutoConfig::new(
        r#"
        name: "cyc"
        app("a") {
            links: ["b"]
        }
        lib("b") {
            links: ["a"]
        }
        "#,
    )
    .unwrap();
    let mut pac = Pac::new(config);
    let err = pac.resolve().expect_err("内部环必须报错");
    let msg = err.to_string();
    assert!(msg.contains("no progress"), "错误文本: {}", msg);
    assert!(
        msg.contains("a") && msg.contains("b"),
        "环成员具名: {}",
        msg
    );
    // 不开始 scan：targets 未被 scan 填充（srcs 为空）
    assert!(pac.targets.iter().all(|t| t.srcs.is_empty()));
}
