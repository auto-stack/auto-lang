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

/// V-05（串行）：内存区分力+负控 与 真实 cargo 执行/失败路径。
/// cwd 切换是进程级的——两段合入单测试避免并行互踩（R1-F01 修复注记）。
#[test]
fn v05_memory_and_cargo_real_serialized() {
    v05a_ninja_memory_discriminating();
    v05b_cargo_real_build_and_failure_paths();
    v05d_unwritable_output_and_real_ninja_when_compiler();
}

fn v05a_ninja_memory_discriminating() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("apps/main")).unwrap();
    std::fs::write(root.join("apps/main/main.c"), "int main(void){return 0;}\n").unwrap();
    std::fs::write(root.join("pac.at"), "name: \"v05a\"\napp(\"main\") { }\n").unwrap();

    let cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(root).unwrap();
    let ninja_bytes = (|| -> AutoResult<Vec<u8>> {
        let config =
            auto_lang::config::AutoConfig::from_file(&root.join("pac.at"), &auto_val::Obj::new())?;
        let mut pac = Pac::new(config);
        pac.set_port("win32".into())?;
        pac.resolve()?;
        let mut builder = pac.builder.take().expect("set_port 已装配 builder");
        builder.enable_memory_output()?;
        builder.build(&mut pac)?;
        let outputs = builder.get_memory_output();
        let key = outputs.keys().next().cloned().expect("内存输出非空");
        Ok(outputs.get(&key).cloned().expect("字节存在"))
    })();
    std::env::set_current_dir(cwd).unwrap();
    let bytes = ninja_bytes.expect("内存模式构建成功");
    let text = String::from_utf8_lossy(&bytes).to_string();
    assert!(
        text.contains("main.c"),
        "build.ninja 应含 main.c 编译规则：\n{}",
        text
    );
    assert!(text.contains("rule"), "规则段在场");
    assert!(
        text.contains("exe") || text.contains("link"),
        "链接规则在场：\n{}",
        text
    );

    // 空产物负控：无源文件工程不得产出源编译行
    let dir2 = tempfile::tempdir().unwrap();
    let root2 = dir2.path();
    std::fs::create_dir_all(root2.join("apps/empty")).unwrap();
    std::fs::write(root2.join("pac.at"), "name: \"v05b\"\napp(\"empty\") { }\n").unwrap();
    let cwd2 = std::env::current_dir().unwrap();
    std::env::set_current_dir(root2).unwrap();
    let bytes2 = (|| -> AutoResult<Vec<u8>> {
        let config =
            auto_lang::config::AutoConfig::from_file(&root2.join("pac.at"), &auto_val::Obj::new())?;
        let mut pac = Pac::new(config);
        pac.set_port("win32".into())?;
        pac.resolve()?;
        let mut builder = pac.builder.take().expect("set_port 已装配 builder");
        builder.enable_memory_output()?;
        builder.build(&mut pac)?;
        let outputs = builder.get_memory_output();
        Ok(outputs.values().next().cloned().unwrap_or_default())
    })();
    std::env::set_current_dir(cwd2).unwrap();
    let text2 = String::from_utf8_lossy(&bytes2.expect("空工程构建成功")).to_string();
    assert!(
        !text2.contains("main.c") && !text2.contains("empty.c"),
        "空工程不得产出源编译规则：\n{}",
        text2
    );
}

fn v05b_cargo_real_build_and_failure_paths() {
    if which("cargo").is_none() {
        panic!("V05-PENDING: cargo 不可用——真实 cargo 验收未执行，不能声称通过");
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("rust/src")).unwrap();
    std::fs::write(
        root.join("rust/src/main.rs"),
        "fn main() { println!(\"v05b\"); }\n",
    )
    .unwrap();
    std::fs::write(
        root.join("pac.at"),
        "name: \"v05cargo\"\nlang: \"rust\"\napp(\"main\") { }\n",
    )
    .unwrap();

    let cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(root).unwrap();
    let result = (|| -> AutoResult<std::path::PathBuf> {
        let config =
            auto_lang::config::AutoConfig::from_file(&root.join("pac.at"), &auto_val::Obj::new())?;
        let mut pac = Pac::new(config);
        pac.set_port("win32".into())?;
        pac.resolve()?;
        pac.build()?;
        let toml = root.join("rust/Cargo.toml");
        assert!(toml.is_file(), "Cargo.toml 已生成");
        Ok(toml)
    })();
    std::env::set_current_dir(cwd).unwrap();
    let toml = result.expect("真实 cargo build 成功");
    let toml_text = std::fs::read_to_string(&toml).unwrap();
    assert!(toml_text.contains("name ="), "包名在场");

    // 失败路径 1：cargo builder 无 rust 目标 → setup 阶段 Err（阶段可
    // 区分，不 spawn；render rust 选 cargo + 显式 c 目标构造该形态）
    let dir2 = tempfile::tempdir().unwrap();
    let root2 = dir2.path();
    std::fs::write(
        root2.join("pac.at"),
        "name: \"nocargo\"\nrender: \"rust\"\napp(\"main\") { lang: \"c\" }\n",
    )
    .unwrap();
    let cwd2 = std::env::current_dir().unwrap();
    std::env::set_current_dir(root2).unwrap();
    let err = (|| -> AutoResult<()> {
        let config =
            auto_lang::config::AutoConfig::from_file(&root2.join("pac.at"), &auto_val::Obj::new())?;
        let mut pac = Pac::new(config);
        pac.set_port("win32".into())?;
        pac.resolve()?;
        pac.build()?;
        Ok(())
    })();
    std::env::set_current_dir(cwd2).unwrap();
    let err = err.expect_err("无 rust 目标必须报错");
    assert!(
        err.to_string().contains("no rust target"),
        "失败阶段文本：{}",
        err
    );

    // 失败路径 2：索引缺依赖 → resolve 阶段 Err（download 面，不进入构建）
    let dir3 = tempfile::tempdir().unwrap();
    let root3 = dir3.path();
    std::fs::write(
        root3.join("pac.at"),
        "name: \"missdep\"\ndep \"no-such-dep-xyz\" { }\n",
    )
    .unwrap();
    let cwd3 = std::env::current_dir().unwrap();
    std::env::set_current_dir(root3).unwrap();
    let err3 = (|| -> AutoResult<()> {
        let config =
            auto_lang::config::AutoConfig::from_file(&root3.join("pac.at"), &auto_val::Obj::new())?;
        let mut pac = Pac::new(config);
        pac.set_port("win32".into())?;
        pac.resolve()?;
        Ok(())
    })();
    std::env::set_current_dir(cwd3).unwrap();
    let err3 = err3.expect_err("缺依赖必须报错");
    assert!(
        err3.to_string().to_lowercase().contains("not found")
            || err3.to_string().to_lowercase().contains("failed"),
        "缺依赖错误可诊断：{}",
        err3
    );
}

/// V-05c：真实 VueBuilder 执行（npm+node，离线 build 脚本）+ 失败传播。
#[test]
fn v05_vue_real_build_and_failure() {
    // PLAN-816 T-03: Builder 的 npm 编排经 UiOps seam——compat 实现注入。
    auto_man::compat::install_default_ui_ops();
    if which("npm").is_none() || which("node").is_none() {
        panic!(
            "V05-PENDING: npm/node 不可用——真实 VueBuilder 验收未执行， \
             本环境无法声称通过（AC-04 环境缺失=pending，非 pass）"
        );
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("package.json"),
        "{ \"name\": \"v05c\", \"version\": \"0.0.1\", \"scripts\": { \"build\": \"node -e \\\"require('fs').writeFileSync('v05-marker.txt','ok')\\\"\" } }",
    )
    .unwrap();
    let mut builder = crate::builder::new_builder(crate::builder::BuilderKind::Vue(
        auto_val::AutoPath::new(root.to_str().unwrap()),
    ));
    builder
        .build(&mut Pac::new(
            auto_lang::config::AutoConfig::new("name: \"v05c\"\n").unwrap(),
        ))
        .expect("真实 npm run build 成功");
    assert!(root.join("v05-marker.txt").is_file(), "build 产物标记在场");

    // 失败传播：build 脚本非零退出 → Err
    let dir2 = tempfile::tempdir().unwrap();
    let root2 = dir2.path();
    std::fs::write(
        root2.join("package.json"),
        "{ \"name\": \"v05d\", \"version\": \"0.0.1\", \"scripts\": { \"build\": \"node -e \\\"process.exit(3)\\\"\" } }",
    )
    .unwrap();
    let mut builder2 = crate::builder::new_builder(crate::builder::BuilderKind::Vue(
        auto_val::AutoPath::new(root2.to_str().unwrap()),
    ));
    let err = builder2
        .build(&mut Pac::new(
            auto_lang::config::AutoConfig::new("name: \"v05d\"\n").unwrap(),
        ))
        .expect_err("非零退出必须 Err");
    assert!(!err.to_string().is_empty(), "失败文本非空：{}", err);
}

fn which(tool: &str) -> Option<std::path::PathBuf> {
    // R2-F04：按生产 runner 语义解析——Windows 下 npm/pnpm 等经
    // .cmd/.bat 入口分发（PATHEXT），只拼 .exe 会漏并造成假跳过。
    let path = std::env::var_os("PATH")?;
    let exts: Vec<String> = std::env::var("PATHEXT")
        .unwrap_or_else(|_| ".EXE;.CMD;.BAT".to_string())
        .split(';')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    let mut exts: Vec<String> = exts
        .iter()
        .map(|e| e.trim_start_matches('.').to_string())
        .collect();
    exts.push("exe".into());
    exts.push("cmd".into());
    exts.push("bat".into());
    exts.push("ps1".into());
    for dir in std::env::split_paths(&path) {
        for ext in &exts {
            let cand = dir.join(format!("{tool}.{ext}"));
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    None
}

/// R2-F01 永久回归：Ninja 必须执行**首次访问的具体依赖实例**——
/// app.deps 内嵌的 merge_link 覆盖副本（override.c）先于同 rename 顶层
/// 目标（base.c）；顶层在前/内嵌独占两正控排除探针一概报红。
/// 以真实 NinjaBuilder 内存模式字节为 oracle（无工具可执行依赖）。
#[test]
fn r2_f01_ninja_first_encounter_identity() {
    fn build_memory(targets: Vec<Target>) -> String {
        let config = auto_lang::config::AutoConfig::new("name: \"p\"\n").unwrap();
        let mut pac = Pac::new(config);
        pac.targets = targets;
        let mut builder = crate::builder::new_builder(crate::builder::BuilderKind::Ninja(
            auto_val::AutoPath::new("build"),
        ));
        builder.enable_memory_output().unwrap();
        builder.build(&mut pac).unwrap();
        let outputs = builder.get_memory_output();
        String::from_utf8_lossy(outputs.values().next().unwrap()).to_string()
    }
    let make = |name: &str, src: &str| {
        let mut t = Target::new(name, crate::TargetKind::App);
        t.rename = name.into();
        t.srcs.insert(src.into());
        t
    };
    // 1) app 先 + 其 deps 内嵌覆盖副本：override.c 胜，base.c 不发射
    let mut app = make("app", "app.c");
    let mut override_lib = make("lib", "override.c");
    override_lib.rename = "lib".into();
    app.deps = vec![override_lib];
    let top_lib = make("lib", "base.c");
    let out = build_memory(vec![app, top_lib]);
    assert!(out.contains("override.c"), "首次访问=内嵌覆盖副本：\n{out}");
    assert!(
        !out.contains("base.c"),
        "顶层同 rename 副本不得发射：\n{out}"
    );
    // 2) 顶层在前：base.c 胜
    let mut app2 = make("app", "app.c");
    let mut dep2 = make("lib", "override.c");
    dep2.rename = "lib".into();
    app2.deps = vec![dep2];
    let out2 = build_memory(vec![make("lib", "base.c"), app2]);
    assert!(out2.contains("base.c"), "顶层先访问：\n{out2}");
    assert!(!out2.contains("override.c"), "内嵌副本被去重：\n{out2}");
    // 3) 内嵌独占（无顶层同名）：override.c 照常
    let mut app3 = make("app", "app.c");
    let mut dep3 = make("lib", "override.c");
    dep3.rename = "lib".into();
    app3.deps = vec![dep3];
    let out3 = build_memory(vec![app3]);
    assert!(out3.contains("override.c") && out3.contains("app.c"));
}

/// R2-04 矩阵补充：不可写输出失败路径（Cargo 输出位被文件占位 →
/// setup 写 Cargo.toml 失败必须 Err，不静默）；真实 Ninja 成功产物在
/// 编译器可用时执行（缺编译器=显式 pending 记录）。
#[allow(dead_code)]
fn v05d_unwritable_output_and_real_ninja_when_compiler() {
    // 失败路径：不可写输出（rust/ 下预置同名文件占位目录路径）
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("rust/src")).unwrap();
    std::fs::write(
        root.join("rust/src/main.rs"),
        "fn main() {}
",
    )
    .unwrap();
    std::fs::write(
        root.join("pac.at"),
        "name: \"uw\"
lang: \"rust\"
app(\"main\") { }
",
    )
    .unwrap();
    // rust/Cargo.toml 占位为一个**目录**——write_file 对目录失败
    std::fs::create_dir_all(root.join("rust/Cargo.toml")).unwrap();
    let cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(root).unwrap();
    let err = (|| -> AutoResult<()> {
        let config =
            auto_lang::config::AutoConfig::from_file(&root.join("pac.at"), &auto_val::Obj::new())?;
        let mut pac = Pac::new(config);
        pac.set_port("win32".into())?;
        pac.resolve()?;
        pac.build()?;
        Ok(())
    })();
    std::env::set_current_dir(cwd).unwrap();
    assert!(err.is_err(), "输出位不可写必须失败：{:?}", err.map(|_| ()));

    // 真实 Ninja（R7：经 vcvars 注入 MSVC 环境后真跑；edition 2021 的
    // set_var 在本串行测试内安全）。无任何编译器/VS 环境才 panic-pending。
    let msvc_env = dump_vcvars_env();
    if let Some(env) = &msvc_env {
        for (k, v) in env {
            std::env::set_var(k, v);
        }
    }
    let compiler = which("gcc")
        .or_else(|| which("clang"))
        .or_else(|| which("cl"))
        .or_else(|| msvc_env.is_some().then(|| std::path::PathBuf::new()));
    match compiler {
        None => {
            panic!(
                "V05-PENDING: 无 C 编译器且无 VS 环境（gcc/clang/cl/vcvars 均 \
不可用）——真实 Ninja 成功产物验收未执行，不能声称通过"
            );
        }
        Some(_) => {
            let dir2 = tempfile::tempdir().unwrap();
            let root2 = dir2.path();
            std::fs::create_dir_all(root2.join("apps/main")).unwrap();
            std::fs::write(
                root2.join("apps/main/main.c"),
                "int main(void){return 0;}\n",
            )
            .unwrap();
            std::fs::write(
                root2.join("pac.at"),
                "name: \"nj\"
app(\"main\") { }
",
            )
            .unwrap();
            let cwd2 = std::env::current_dir().unwrap();
            std::env::set_current_dir(root2).unwrap();
            let built = (|| -> AutoResult<()> {
                let config = auto_lang::config::AutoConfig::from_file(
                    &root2.join("pac.at"),
                    &auto_val::Obj::new(),
                )?;
                let mut pac = Pac::new(config);
                pac.set_port("win32".into())?;
                pac.resolve()?;
                pac.build()?; // 真实 spawn ninja
                Ok(())
            })();
            std::env::set_current_dir(cwd2).unwrap();
            built.unwrap_or_else(|e| {
                panic!("真实 ninja build 应成功（编译器在场）：{e}");
            });
        }
    }
}

/// 经 vcvars64.bat dump MSVC 环境（PATH/INCLUDE/LIB/LIBPATH）。
/// 无 VS 安装返回 None（pending 语义由调用方表达）。
fn dump_vcvars_env() -> Option<Vec<(String, String)>> {
    let vswhere = std::path::Path::new(
        r"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe",
    );
    if !vswhere.is_file() {
        return None;
    }
    let out = std::process::Command::new(vswhere)
        .args(["-latest", "-property", "installationPath"])
        .output()
        .ok()?;
    let inst = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if inst.is_empty() {
        return None;
    }
    let bat = format!("{inst}\\VC\\Auxiliary\\Build\\vcvars64.bat");
    if !std::path::Path::new(&bat).is_file() {
        return None;
    }
    let tmp = tempfile::tempdir().ok()?;
    let script = tmp.path().join("vcenv.bat");
    std::fs::write(
        &script,
        format!("@echo off\r\ncall \"{bat}\" >nul 2>&1\r\nset\r\n"),
    )
    .ok()?;
    let out = std::process::Command::new("cmd")
        .arg("/C")
        .arg(&script)
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout).to_string();

    // R8：Windows set dump 的变量名为注册大小写（常见 Path=/Include=）——
    // 大小写不敏感匹配，键名归一为大写；且注入后必须真能定位 cl（否则
    // 返回 None 走 pending，不假成功）。
    let wanted = ["PATH", "INCLUDE", "LIB", "LIBPATH"];
    let mut env: Vec<(String, String)> = Vec::new();
    for l in text.lines() {
        if let Some((k, v)) = l.split_once('=') {
            let ku = k.trim().to_ascii_uppercase();
            if wanted.contains(&ku.as_str()) {
                env.push((ku, v.to_string()));
            }
        }
    }

    if env.len() >= 2 {
        let saved = std::env::var_os("PATH");
        for (k, v) in &env {
            std::env::set_var(k, v);
        }
        let ok = which("cl").is_some();
        if let Some(p) = saved {
            std::env::set_var("PATH", p);
        }
        ok.then_some(env)
    } else {
        None
    }
}
