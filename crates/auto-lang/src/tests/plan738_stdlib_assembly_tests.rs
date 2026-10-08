//! PLAN-738 T-02：stdlib_assembly 模型/装载/provider 机器对照（正式族）。
//!
//! 覆盖：全库 inventory 无静默遗漏（分母=层数恰等）、parse 破损冻结与
//! 探针 P6 同源、稳定 JSON、provider 目录形状 + 真实注册/发射面对照。
//! 核心符号校验门（签名/绑定）在 T-04 的本族后续测试。

// ============================================================================
// T-02 ② inventory 全库扫描
// ============================================================================

#[cfg(test)]
mod t02_inventory {
    use crate::stdlib_assembly::{loader, validate};

    /// 全库清点：无静默遗漏——modules 层数之和 == files_total；六核心在册；
    /// parse 破损分母冻结（与探针 P6 同源：async.at + json.rs.at）。
    #[test]
    fn full_scan_no_silent_skips_and_frozen_parse_failures() {
        let root = loader::repo_stdlib_root().unwrap();
        let inv = loader::scan_inventory(&root);

        let counted: usize = inv.modules.iter().map(|m| m.layers.len()).sum();
        assert_eq!(
            counted, inv.files_total,
            "每个 .at 层文件必须恰出现一次（parse 失败也不删分母）"
        );

        let failed: Vec<&str> = inv
            .diagnostics
            .iter()
            .filter(|d| d.code == validate::code::PARSE_FAIL)
            .map(|d| d.file.as_deref().unwrap_or(""))
            .collect();
        for d in inv.diagnostics.iter().filter(|d| d.code == validate::code::PARSE_FAIL) {
            println!("INV738DIAG\t{}\t{}", d.file.as_deref().unwrap_or(""), d.message);
        }
        // 全库 isolated-parse 基线（T-01 探针只扫六核心；全库实勘 39/115）：
        // 三类构成见 738-stdlib-decision §2——语法破损（str.at `with` 参数名、
        // list/async `type X[T]`、iter/* spec 上下文）、跨模块类型依赖
        // （app.at Widget）、镜像语法漂移（json.rs.at 等 .rs.at）。
        // VM 生态实际工作面 = native 注册（.vm.at 磁盘扫描 + catalog + 手工
        // shim），stdlib_tests.rs 的 `use auto.str` 系 #[ignore] 即佐证。
        assert_eq!(
            failed.len(),
            39,
            "全库 isolated-parse 失败数漂移——逐名核对后更新本基线与决策报告 §2"
        );
        assert!(
            failed.contains(&"stdlib/auto/async.at") && failed.contains(&"stdlib/auto/json.rs.at")
                && failed.contains(&"stdlib/auto/str.at")
                && failed.contains(&"stdlib/auto/list.at"),
            "已知破损代表须在列: {failed:?}"
        );

        for m in ["io", "net", "async", "http", "json", "sse"] {
            assert!(inv.module(m).is_some(), "六核心 {m} 应在 inventory");
        }
        // T-01 冻结的六核心层数（io3+net2+async2+http2+json3+sse1）
        let six: usize = ["io", "net", "async", "http", "json", "sse"]
            .iter()
            .map(|m| inv.module(m).unwrap().layers.len())
            .sum();
        assert_eq!(six, 13, "六核心层数分母漂移");
    }

    /// 稳定 JSON：两次序列化逐字节一致；排序自检通过。
    #[test]
    fn inventory_json_is_stable() {
        let root = loader::repo_stdlib_root().unwrap();
        let inv = loader::scan_inventory(&root);
        inv.assert_sorted();
        let a = serde_json::to_string(&inv).unwrap();
        let b = serde_json::to_string(&inv).unwrap();
        assert_eq!(a, b, "inventory JSON 必须确定输出");
        // 便携约束：不携带运行根绝对路径
        assert!(
            !a.contains(&root.to_string_lossy().replace('\\', "/")),
            "便携清单不得携带机器绝对路径"
        );
    }

    /// 覆盖检查：目标层无公共层 → NO_PUBLIC_LAYER 诊断（合成模块验证行为）。
    #[test]
    fn coverage_flags_target_layer_without_public() {
        use crate::stdlib_assembly::model::*;
        let modules = vec![ModuleInventory {
            module: "ghost".to_string(),
            layers: vec![LayerInventory {
                kind: LayerKind::Vm,
                file: "stdlib/auto/ghost.vm.at".to_string(),
                content_hash: 0,
                parse: ParseStatus::Parsed { symbol_count: 0 },
                symbols: vec![],
            }],
        }];
        let diags = validate::coverage_checks(&modules);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, validate::code::NO_PUBLIC_LAYER);
        assert_eq!(diags[0].module, "ghost");
    }
}

// ============================================================================
// T-03 ①：目标驱动层选择 + 层选择记录 + 源段归因 + trans 入口契约
// ============================================================================

#[cfg(test)]
mod t03_assembly_wiring {
    use crate::stdlib_assembly::model::AssemblyTarget;
    use std::fs;
    use std::path::Path;

    /// 四层合成模块（探针 P1 同款 fixture）。
    fn write_proto_fixture(dir: &Path) -> std::path::PathBuf {
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

    /// AC-02：装配目标驱动层选择——VM 目标合并选定层并记录；Rust/C 目标
    /// 零装载面（目标层只记 candidate，不进类型上下文——foreign 层不串入）。
    #[test]
    fn target_driven_layer_selection_and_record() {
        let tmp = tempfile::tempdir().unwrap();
        let main_path = write_proto_fixture(tmp.path());
        let source = fs::read_to_string(&main_path).unwrap();

        // VM（默认目标）：公共+vm 层合并；层选择记录齐备。
        let mut vm_session = crate::compile::CompileSession::new();
        vm_session.add_source_dir(tmp.path().to_path_buf());
        vm_session.resolve_uses(&source).unwrap();
        vm_session
            .compile_source(&source, main_path.to_str().unwrap())
            .unwrap();
        assert_eq!(vm_session.layer_selections.len(), 1, "一次模块装载恰一条记录");
        let sel = &vm_session.layer_selections[0];
        assert_eq!(sel.module, "proto");
        assert_eq!(sel.target, AssemblyTarget::Vm);
        assert!(
            sel.context_file.as_deref().unwrap().ends_with("proto.vm.at"),
            "VM 目标应合并 .vm.at: {:?}",
            sel.context_file
        );
        assert!(
            sel.candidate_files.iter().any(|c| c.ends_with("proto.rs.at"))
                && sel.candidate_files.iter().any(|c| c.ends_with("proto.c.at")),
            "未消费层记 candidate: {:?}",
            sel.candidate_files
        );
        assert!(sel.context_byte_boundary.is_some());

        // Rust 目标：.vm.at 不合并（vm_only 不进类型上下文）；.rs.at 记
        // candidate 不解析（发射走名称表——决策报告 E1③）。
        let mut rs_session = crate::compile::CompileSession::new();
        rs_session.add_source_dir(tmp.path().to_path_buf());
        rs_session.set_assembly_target(AssemblyTarget::Rust);
        rs_session.resolve_uses(&source).unwrap();
        rs_session
            .compile_source(&source, main_path.to_str().unwrap())
            .unwrap();
        {
            let store = rs_session.type_store();
            let store = store.read().unwrap();
            let module = store.lookup_module("proto").unwrap();
            let names = module.store.pub_fn_names();
            assert!(
                names.contains(&"pub_fn".to_string())
                    && !names.contains(&"vm_only".to_string())
                    && !names.contains(&"c_only".to_string()),
                "Rust 装配上下文不得含 VM/C 层符号: {names:?}"
            );
        }
        let sel = &rs_session.layer_selections[0];
        assert_eq!(sel.target, AssemblyTarget::Rust);
        assert!(sel.context_file.is_none(), "Rust 目标零装载面：无合并层");
        assert!(sel.candidate_files.iter().any(|c| c.ends_with("proto.rs.at")));

        // C 目标：同构（c 层 candidate，vm 层不串入）。
        let mut c_session = crate::compile::CompileSession::new();
        c_session.add_source_dir(tmp.path().to_path_buf());
        c_session.set_assembly_target(AssemblyTarget::C);
        c_session.resolve_uses(&source).unwrap();
        {
            let store = c_session.type_store();
            let store = store.read().unwrap();
            let names = store.lookup_module("proto").unwrap().store.pub_fn_names();
            assert!(
                names.contains(&"pub_fn".to_string())
                    && !names.contains(&"vm_only".to_string())
                    && !names.contains(&"rs_only".to_string()),
                "C 装配上下文不得含 VM/Rust 层符号: {names:?}"
            );
        }
        assert_eq!(c_session.layer_selections[0].target, AssemblyTarget::C);
    }

    /// AC-02：trans 入口在装载/发射前声明装配目标（入口契约）。
    #[test]
    fn trans_entries_declare_assembly_target() {
        let tmp = tempfile::tempdir().unwrap();
        let main = tmp.path().join("t.at");
        fs::write(&main, "fn main() {\n    let x = 1\n}\n").unwrap();

        let mut s1 = crate::compile::CompileSession::new();
        crate::trans_c_with_session(&mut s1, main.to_str().unwrap()).unwrap();
        assert_eq!(s1.assembly.target, AssemblyTarget::C);

        let mut s2 = crate::compile::CompileSession::new();
        crate::trans_rust_with_session(&mut s2, main.to_str().unwrap()).unwrap();
        assert_eq!(s2.assembly.target, AssemblyTarget::Rust);
    }

    /// AC-04：目标层语法错误经源段归因指向真实目标层文件（合并源原以公共
    /// 文件名 attach source——失败路径有界归因后指向 .vm.at）。
    #[test]
    fn target_layer_syntax_error_attributed() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("main.at"),
            "use proto: *\n\nfn main() {\n    let x = 1\n}\n",
        )
        .unwrap();
        fs::write(tmp.path().join("proto.at"), "pub fn pub_fn() int;\n").unwrap();
        fs::write(tmp.path().join("proto.vm.at"), "pub fn broken( [[[;\n").unwrap();

        let mut session = crate::compile::CompileSession::new();
        session.add_source_dir(tmp.path().to_path_buf());
        let source = fs::read_to_string(tmp.path().join("main.at")).unwrap();
        let err = session.resolve_uses(&source).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("target layer") && msg.contains("proto.vm.at"),
            "目标层语法错误应归因到 .vm.at 文件: {msg}"
        );
    }

    /// AC-02：VM 两入口同模块对拍——persistent 修复后与 session 同样可见
    /// .vm.at 层（io 的 ext File 方法以短别名注册；T-03 前恒 miss）。
    #[test]
    fn vm_two_entry_same_layer_visibility() {
        // io ext 方法的 canonical 名（auto.io.file.*）只在磁盘扫描面注册
        // （非静态白名单/手工 shim），而扫描是 CWD 相对 stdlib/auto（决策
        // 报告 §2d）——测试钉仓根后显式跑生产 init 同款注册。
        let stdlib_root = crate::stdlib_assembly::loader::repo_stdlib_root().unwrap();
        let repo_root = stdlib_root.parent().and_then(|p| p.parent()).unwrap().to_path_buf();
        let orig_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo_root).unwrap();
        crate::vm::native_registry::register_builtin_natives();
        std::env::set_current_dir(orig_cwd).unwrap();

        let mut sess = crate::autovm_persistent::AutovmReplSession::new();
        sess.run("use auto.io: *").ok();
        sess.run("use auto.net: *").ok();

        let reg = crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap();
        // io.vm.at 的 #[vm] ext File 方法：persistent 经
        // auto.io.file.<method> canonical 查 ID 后注册短别名。
        assert!(
            reg.get_id("read_text").is_some(),
            "persistent 应可见 .vm.at 层 ext 方法（T-03 context 路径修复+canonical 对齐）"
        );
        // net（顶层 #[vm] fn）同样可见——与探针 P5 修复后断言互证。
        assert!(
            reg.get_id("tcp_listener_accept").is_some(),
            "net.vm.at 顶层 #[vm] fn 短别名应注册"
        );
    }
}

// ============================================================================
// T-02 ② provider 目录机器对照
// ============================================================================

#[cfg(test)]
mod t02_providers {
    use crate::stdlib_assembly::{loader, providers};

    /// 目录形状 + 声明引用真实模块 + 真实注册/发射面对照：
    /// - vm supported → BIGVM_NATIVES 名 surface 有 locator 前缀命中；
    /// - rust a2r-std supported → a2r-std lib.rs 有 pub mod <leaf>；
    /// - c c-generated supported → locator 文件在磁盘。
    #[test]
    fn catalog_shape_and_real_surface_cross_check() {
        let catalog = providers::load_catalog().expect("assembly-providers.json 可解析");
        let shape = providers::shape_checks(&catalog);
        assert!(shape.is_empty(), "provider 目录形状违规: {shape:?}");

        let root = loader::repo_stdlib_root().unwrap();
        let inv = loader::scan_inventory(&root);
        for c in &catalog.providers {
            assert!(
                inv.module(&c.module).is_some(),
                "provider 声明模块 '{}' 不在 inventory——目录引用了不存在的模块",
                c.module
            );
        }

        // 真实 VM 名 surface（生产 init 同款：register_vm_declarations 磁盘
        // 扫描 #[vm] 注册）。扫描为 **CWD 相对 stdlib/auto**（实勘发现：
        // native 名 surface 依赖进程 CWD，见决策报告 §1/E3）——测试钉到
        // 仓根再恢复；nextest 每测独立进程，裸 cargo test 共进程下此窗口
        // 极短且只读注册表。
        let repo_root = root.parent().and_then(|p| p.parent()).unwrap().to_path_buf();
        let orig_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo_root).unwrap();
        crate::vm::native_registry::register_builtin_natives();
        std::env::set_current_dir(orig_cwd).unwrap();
        let vm_names =
            crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap().get_function_names();
        let a2r_lib = include_str!("../../../a2r-std/src/lib.rs");
        let drift = providers::cross_check_real_surfaces(&catalog, &vm_names, a2r_lib, &root);
        assert!(
            drift.is_empty(),
            "provider supported 声明与真实面脱节（修复目录或真实面，不得静默）: {drift:?}"
        );
    }

    /// 名称表漂移哨兵：发射表路由的 a2r 模块与磁盘 pub mod 的差集必须与
    /// 目录 unsupported 声明一致（表再漂移时此处红，提示更新目录）。
    #[test]
    fn emission_table_drift_matches_catalog_unsupported() {
        // trans/rust.rs use_stmt 三处名单的规范集（任一处漂移以其为准更新）
        const EMISSION_ROUTED: &[&str] = &[
            "math", "str", "time", "env", "json", "file", "fs", "http", "list",
            "hashmap", "hashset", "btreemap", "vecdeque", "char", "conv", "io",
            "log", "path", "net", "process", "sys", "sse", "may", "sqlite",
            "redis",
        ];
        let a2r_lib = include_str!("../../../a2r-std/src/lib.rs");
        let missing: Vec<&str> = EMISSION_ROUTED
            .iter()
            .copied()
            .filter(|m| !a2r_lib.contains(&format!("pub mod {m}")))
            .collect();
        let catalog = providers::load_catalog().unwrap();
        let declared_unsupported: Vec<String> = catalog
            .providers
            .iter()
            .filter(|c| c.target == "rust" && c.status == "unsupported")
            .map(|c| c.module.clone())
            .collect();
        let mut missing_sorted = missing.clone();
        missing_sorted.sort();
        let mut unsupported_sorted = declared_unsupported.clone();
        unsupported_sorted.sort();
        assert_eq!(
            missing_sorted, unsupported_sorted,
            "发射表→a2r 缺模块差集与目录 unsupported 声明不一致（新增漂移须登记目录）"
        );
    }
}
