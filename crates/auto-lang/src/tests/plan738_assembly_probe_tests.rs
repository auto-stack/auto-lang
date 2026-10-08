//! PLAN-738 T-01：装配原型探针（evidence，非产品测试；T-02 起由
//! plan738_stdlib_assembly_tests 正式族替代/收编）。
//!
//! 用真实入口冻结装配事实，作为 `docs/plans/reports/738-stdlib-decision.md`
//! 的 E 系证据。实勘结论（2026-10-08，基线 c3ccd32c3）：
//! - 模块装载只发生在 VM 执行管线（execute_autovm/test_code/run_with_path
//!   先 resolve_uses → load_module）；`CompileSession::compile_source` 本身
//!   不解析 use —— trans_rust/trans_c 入口**从不装载模块**。
//! - 四个装配面各自为政：
//!   ① VM session：公共.at + 硬编码 .vm.at 合并（compile.rs context_ext）；
//!   ② VM persistent：stdlib-only 查找 + context 路径双 auto/ 恒不存在
//!      （静默跳过）→ 实际实现面 = 全局 native 注册表；
//!   ③ Rust trans：发射期名称表（3 处重复硬编码 26 名单）→ a2r_std，
//!      其中 io/net/path/char/conv/log/may/hashset/btreemap/vecdeque 无真实
//!      a2r-std 模块；.rs.at 生产不消费（仅 parity 测试镜像）；
//!   ④ C trans：`use auto.X` → `#include "X.h"`（依赖 cmd_a2c_stdlib 逐文件
//!      预生成）；.c.at 仅被该生成器消费。
//! - ModuleCache 仅参与①：按模块名键控，content_hash=合并源 vs is_valid
//!   重读公共文件 → 含 .vm.at 层的模块缓存永不命中。

use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// P1: VM 管线目标层选择（resolve_uses → load_module）
// ---------------------------------------------------------------------------

/// 合成四层模块：公共契约 + 三个互异的目标层（各带独有 fn 名）。
fn write_proto_fixture(dir: &Path) -> PathBuf {
    fs::write(
        dir.join("main.at"),
        "use proto: *\n\nfn main() {\n    let x = 1\n}\n",
    )
    .unwrap();
    fs::write(
        dir.join("proto.at"),
        "type Widget {\n    pub id int\n}\n\npub fn pub_fn() int;\n",
    )
    .unwrap();
    fs::write(dir.join("proto.vm.at"), "#[vm]\npub fn vm_only() int;\n").unwrap();
    fs::write(
        dir.join("proto.rs.at"),
        "pub fn rs_only() int {\n    return 1\n}\n",
    )
    .unwrap();
    fs::write(
        dir.join("proto.c.at"),
        "pub fn c_only() int {\n    return 2\n}\n",
    )
    .unwrap();
    dir.join("main.at")
}

/// 复刻 VM 执行管线的装载步（execute_autovm 同序：resolve_uses → compile）。
fn vm_pipeline_load(main_path: &Path) -> crate::compile::CompileSession {
    let mut session = crate::compile::CompileSession::new();
    session.add_source_dir(main_path.parent().unwrap().to_path_buf());
    let source = fs::read_to_string(main_path).unwrap();
    session.resolve_uses(&source).expect("resolve_uses");
    session
        .compile_source(&source, main_path.to_str().unwrap())
        .expect("compile_source");
    session
}

#[test]
fn p1_vm_pipeline_selects_vm_layer_only() {
    let tmp = tempfile::tempdir().unwrap();
    let main_path = write_proto_fixture(tmp.path());
    let session = vm_pipeline_load(&main_path);

    let store = session.type_store();
    let store = store.read().unwrap();
    let module = store
        .lookup_module("proto")
        .expect("proto module registered in session store");
    let names = module.store.pub_fn_names();

    assert!(
        names.contains(&"pub_fn".to_string()),
        "公共层符号应在模块 store: {names:?}"
    );
    assert!(
        names.contains(&"vm_only".to_string()),
        "VM 层(.vm.at)符号应被合并（context_ext 硬编码证据）: {names:?}"
    );
    assert!(
        !names.contains(&"rs_only".to_string()),
        "Rust 层不得被 VM 管线选择: {names:?}"
    );
    assert!(
        !names.contains(&"c_only".to_string()),
        "C 层不得被 VM 管线选择: {names:?}"
    );
}

// ---------------------------------------------------------------------------
// P2: ModuleCache 双层模块死缓存机制
// ---------------------------------------------------------------------------

#[test]
fn p2_module_cache_never_valid_for_two_layer_modules() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("proto.at");
    let public_only = "pub fn pub_fn() int;\n";
    fs::write(&root, public_only).unwrap();

    // compile.rs 实际形态：with_file 收到的是合并源（公共 + "\n" + context）
    let merged = format!("{public_only}\n#[vm]\npub fn vm_only() int;\n");
    let store = crate::types::TypeStore::new();
    let cache = crate::module_cache::ModuleCache::with_file(
        "proto",
        store.clone(),
        root.to_string_lossy(),
        &merged,
    );

    // is_valid() 重读 file_path（公共文件，内容=public_only）重算 hash 与
    // content_hash（合并源的 hash）比较 → 恒不等 → 未做任何修改也判失效。
    assert!(
        !cache.is_valid(),
        "双层模块缓存未改动也应判 valid==false（死缓存证据）；file={}",
        root.display()
    );

    // 对照组：无 context 层的模块（单层），缓存机制正常。
    let cache_single = crate::module_cache::ModuleCache::with_file(
        "single",
        store,
        root.to_string_lossy(),
        public_only,
    );
    assert!(cache_single.is_valid(), "单层模块未改动应 valid");
}

// ---------------------------------------------------------------------------
// P3: trans 入口从不装载模块（C 头文件包含路由证据）
// ---------------------------------------------------------------------------

#[test]
fn p3_trans_entries_never_assemble_modules() {
    let tmp = tempfile::tempdir().unwrap();
    let main_path = write_proto_fixture(tmp.path());

    // C 入口：不 resolve_uses → proto 模块（公共与任何层）不进 session；
    // use proto 只变成 #include "proto.h"（依赖外部预生成头文件）。
    let mut session = crate::compile::CompileSession::new();
    session.add_source_dir(tmp.path().to_path_buf());
    crate::trans_c_with_session(&mut session, main_path.to_str().unwrap())
        .expect("trans_c_with_session");
    {
        let store = session.type_store();
        let store = store.read().unwrap();
        assert!(
            store.lookup_module("proto").is_none(),
            "C 转译入口不应装载模块（当前无共同装配计划的证据）"
        );
    }
    // C 发射面（静态事实 c.rs::use_stmt → libs.insert("\"proto.h\"")）：
    // include 的具体落盘位置随 incremental/全量模式变化，这里只记录产物
    // 供报告引用；断言核心=模块未进 session（上方）。
    if let Ok(c_out) = fs::read_to_string(tmp.path().join("main.c")) {
        println!("P3EVIDENCE main.c contains proto.h = {}", c_out.contains("proto.h"));
    }
    if let Ok(c_header) = fs::read_to_string(tmp.path().join("main.h")) {
        println!("P3EVIDENCE main.h contains proto.h = {}", c_header.contains("proto.h"));
    }

    // Rust 入口：同样不装载；合成模块名不在名称表 → 落 crate::proto。
    let mut session2 = crate::compile::CompileSession::new();
    session2.add_source_dir(tmp.path().to_path_buf());
    crate::trans_rust_with_session(&mut session2, main_path.to_str().unwrap())
        .expect("trans_rust_with_session");
    let rs_out = fs::read_to_string(tmp.path().join("main.a2r.rs")).unwrap();
    assert!(
        rs_out.contains("crate::proto"),
        "合成模块名不在 a2r 名称表 → crate:: 落点（无 provider 校验的证据）"
    );
}

// ---------------------------------------------------------------------------
// P4: Rust 发射名称表 → a2r_std 路由 vs provider 真实存在性
// ---------------------------------------------------------------------------

#[test]
fn p4_rust_emission_routes_net_to_missing_a2r_module() {
    let tmp = tempfile::tempdir().unwrap();
    let main = tmp.path().join("net_main.at");
    fs::write(&main, "use auto.net\n\nfn main() {\n    let x = 1\n}\n").unwrap();

    let mut session = crate::compile::CompileSession::new();
    crate::trans_rust_with_session(&mut session, main.to_str().unwrap())
        .expect("trans_rust_with_session");

    let emitted = fs::read_to_string(tmp.path().join("net_main.a2r.rs")).unwrap();
    assert!(
        emitted.contains("a2r_std::net"),
        "发射应把 use auto.net 路由到 a2r_std::net（名称表证据）"
    );

    // provider 真实面：a2r-std 无 net 模块 —— 表中名称 ≠ 真实模块存在。
    let a2r_lib = include_str!("../../../a2r-std/src/lib.rs");
    assert!(
        !a2r_lib.contains("pub mod net"),
        "a2r-std 不应有 net 模块（若已存在，本探针的漂移断言已过时，更新证据）"
    );
    // 阳性对照：json 路由且有真实模块。
    let main2 = tmp.path().join("json_main.at");
    fs::write(&main2, "use auto.json\n\nfn main() {\n    let x = 1\n}\n").unwrap();
    let mut session2 = crate::compile::CompileSession::new();
    crate::trans_rust_with_session(&mut session2, main2.to_str().unwrap())
        .expect("trans json");
    let emitted2 = fs::read_to_string(tmp.path().join("json_main.a2r.rs")).unwrap();
    assert!(
        emitted2.contains("a2r_std::json") && a2r_lib.contains("pub mod json"),
        "json 是名称表与 a2r-std 双侧一致的阳性对照"
    );
}

// ---------------------------------------------------------------------------
// P5: persistent（REPL）装载跳过 .vm.at 层
// ---------------------------------------------------------------------------

#[test]
fn p5_persistent_loads_vm_layer_short_alias() {
    // T-03 修复后契约：context 路径与 root 同基名（<stdlib>/io.vm.at 形态），
    // .vm.at 层对 persistent 可见 → wildcard 导入注册 #[vm] 短别名，与 VM
    // session 装配对齐。（T-01 实勘的破损形态——双 auto/ 前缀 + with_extension
    // 吃掉 .vm 使 context 恒不存在、别名永不注册——见决策报告 E1②/E3，
    // 由 T-03 修复；本断言由 is_none 翻转为 is_some。）
    let mut sess = crate::autovm_persistent::AutovmReplSession::new();
    let _ = sess.run("use auto.net: *");

    let reg = crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap();
    // qualified 全名来自 ffi/stdlib.rs 全局注册（与 .at 层无关）：
    assert!(
        reg.get_id("auto.net.tcp_listener_accept").is_some(),
        "auto.net.tcp_listener_accept 应在全局注册表（stdlib.rs 手工 shim 面）"
    );
    // T-03 修复核心断言：顶层 #[vm] fn 短别名现在应注册。
    assert!(
        reg.get_id("tcp_listener_accept").is_some(),
        "persistent 应注册 .vm.at 层 #[vm] fn 短别名（T-03 context 路径修复）"
    );
}

// ---------------------------------------------------------------------------
// P6: 实际 Parser 清点六核心模块分层符号（初始分母）
// ---------------------------------------------------------------------------

#[test]
fn p6_parser_inventory_six_core_modules() {
    // 直接钉仓库 stdlib（CARGO_MANIFEST_DIR=crates/auto-lang → ../../ 上两级
    // 到仓根）。不 find_std_lib()：实勘其项目根分支拼出 <root>/stdlib/stdlib/
    // auto（三级上溯 + 重复 stdlib）恒不命中，cargo 构建下实际解析到
    // ~/.auto/libs/stdlib/auto（本机是指向仓库的 symlink；无该 link 的机器
    // 上行为完全不同）——stdlib 来源身份是 G5/manifest 字段，见决策报告。
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib/auto");
    let stdlib = stdlib.canonicalize().expect("repo stdlib/auto");
    assert!(stdlib.is_dir(), "stdlib/auto 不存在: {}", stdlib.display());

    // 六核心模块 × 现有层（2026-10-03 结构扫描冻结；新增层会在此红，
    // 提示更新分母——正是门禁想要的）。
    let matrix: &[(&str, &[&str])] = &[
        ("io", &["io.at", "io.vm.at", "io.c.at"]),
        ("net", &["net.at", "net.vm.at"]),
        ("async", &["async.at", "async.vm.at"]),
        ("http", &["http.at", "http.vm.at"]),
        ("json", &["json.at", "json.vm.at", "json.rs.at"]),
        ("sse", &["sse.at"]),
    ];

    let mut total_files = 0usize;
    let mut parse_failures: Vec<(String, String)> = Vec::new();
    for (module, layers) in matrix {
        for layer in *layers {
            let path = stdlib.join(layer);
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("读取 {} 失败: {e}", path.display()));
            // 生产装载同款构造（parse_module_to_type_store 用
            // new_with_type_store）。实勘：auto.async.at 的 `type Sender[T]`
            // 泛型声明语法当前 parser 无支持（type_decl 不消费 `[`）——
            // `use auto.async` 在 VM 装配面 today 即失败。738 不改语言语法，
            // 该文件按 §5.1 记录为 diagnostic（不从分母删除），预期失败集
            // 冻结在下方断言；修复后更新此冻结。
            let shared: std::sync::Arc<
                std::sync::RwLock<crate::types::TypeStore>,
            > = std::sync::Arc::new(std::sync::RwLock::new(
                crate::types::TypeStore::new(),
            ));
            let mut parser =
                crate::parser::Parser::new_with_type_store(content.as_str(), shared);
            let ast = match parser.parse() {
                Ok(ast) => ast,
                Err(e) => {
                    parse_failures.push((layer.to_string(), format!("{e:?}")));
                    println!("INV738FAIL\t{module}\t{layer}\tparse_error");
                    continue;
                }
            };
            total_files += 1;

            let mut fns = 0usize;
            let mut pub_fns = 0usize;
            let mut vm_fns = 0usize;
            let mut types = 0usize;
            let mut ext_blocks = 0usize;
            let mut ext_methods = 0usize;
            let mut names: Vec<String> = Vec::new();
            let mut count_fn = |f: &crate::ast::Fn,
                                names: &mut Vec<String>,
                                fns: &mut usize,
                                pub_fns: &mut usize,
                                vm_fns: &mut usize| {
                *fns += 1;
                if f.is_pub {
                    *pub_fns += 1;
                }
                if matches!(f.kind, crate::ast::FnKind::VmFunction) {
                    *vm_fns += 1;
                }
                names.push(format!(
                    "{}{}{}",
                    if f.is_pub { "pub " } else { "" },
                    if matches!(f.kind, crate::ast::FnKind::VmFunction) {
                        "#vm "
                    } else {
                        ""
                    },
                    f.name
                ));
            };
            for stmt in &ast.stmts {
                match stmt {
                    crate::ast::Stmt::Fn(f) => {
                        count_fn(f, &mut names, &mut fns, &mut pub_fns, &mut vm_fns);
                    }
                    crate::ast::Stmt::TypeDecl(t) => {
                        types += 1;
                        names.push(format!("type {}", t.name));
                    }
                    crate::ast::Stmt::Ext(e) => {
                        ext_blocks += 1;
                        for m in &e.methods {
                            ext_methods += 1;
                            count_fn(m, &mut names, &mut fns, &mut pub_fns, &mut vm_fns);
                        }
                        names.push(format!("ext {}", e.target));
                    }
                    _ => {}
                }
            }
            names.sort();
            println!(
                "INV738\t{module}\t{layer}\tfn={fns}\tpub={pub_fns}\tvm={vm_fns}\ttype={types}\text={ext_blocks}\text_methods={ext_methods}"
            );
            println!("INV738NAMES\t{module}\t{layer}\t{}", names.join(","));
        }
    }
    println!("INV738\tTOTAL\tparsed={total_files}\tfailed={}", parse_failures.len());
    // 分母冻结：12 层全部计入；parse 失败集冻结为 async.at（`type Sender[T]`
    // 两处）+ json.rs.at（镜像文件语法漂移）——不静默略过，也不因既存破损
    // 让门禁常红。
    assert_eq!(total_files + parse_failures.len(), 13, "六核心模块现有层数（增删层须更新分母）");
    assert_eq!(
        parse_failures.iter().map(|(f, _)| f.as_str()).collect::<Vec<_>>(),
        vec!["async.at", "json.rs.at"],
        "parse 失败层冻结（修复后更新此冻结与分母）: {parse_failures:?}"
    );
}
