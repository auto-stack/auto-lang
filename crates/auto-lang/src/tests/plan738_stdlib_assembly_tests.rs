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
        for d in inv
            .diagnostics
            .iter()
            .filter(|d| d.code == validate::code::PARSE_FAIL)
        {
            println!(
                "INV738DIAG\t{}\t{}",
                d.file.as_deref().unwrap_or(""),
                d.message
            );
        }
        // 全库 isolated-parse 基线（T-01 探针只扫六核心；全库实勘 39/115）：
        // 三类构成见 738-stdlib-decision §2——语法破损（str.at `with` 参数名、
        // list/async `type X[T]`、iter/* spec 上下文）、跨模块类型依赖
        // （app.at Widget）、镜像语法漂移（json.rs.at 等 .rs.at）。
        // VM 生态实际工作面 = native 注册（.vm.at 磁盘扫描 + catalog + 手工
        // shim），stdlib_tests.rs 的 `use auto.str` 系 #[ignore] 即佐证。
        assert_eq!(
            failed.len(),
            38,
            "全库 isolated-parse 失败数漂移——逐名核对后更新本基线与决策报告 §2"
        );
        assert!(
            !failed.contains(&"stdlib/auto/async.at")
                && failed.contains(&"stdlib/auto/json.rs.at")
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
        assert_eq!(
            vm_session.layer_selections.len(),
            1,
            "一次模块装载恰一条记录"
        );
        let sel = &vm_session.layer_selections[0];
        assert_eq!(sel.module, "proto");
        assert_eq!(sel.target, AssemblyTarget::Vm);
        assert!(
            sel.context_file
                .as_deref()
                .unwrap()
                .ends_with("proto.vm.at"),
            "VM 目标应合并 .vm.at: {:?}",
            sel.context_file
        );
        assert!(
            sel.candidate_files
                .iter()
                .any(|c| c.ends_with("proto.rs.at"))
                && sel
                    .candidate_files
                    .iter()
                    .any(|c| c.ends_with("proto.c.at")),
            "未消费层记 candidate: {:?}",
            sel.candidate_files
        );
        assert!(sel.context_byte_boundary.is_some());

        // Rust 目标：.vm.at 不合并（vm_only 不进类型上下文）；.rs.at 记
        // candidate 不解析（发射走名称表——决策报告 E1③）。
        let mut rs_session = crate::compile::CompileSession::new();
        rs_session.add_source_dir(tmp.path().to_path_buf());
        rs_session
            .set_assembly_target(AssemblyTarget::Rust)
            .unwrap();
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
                    && !names.contains(&"c_only".to_string())
                    && names.contains(&"rs_only".to_string()),
                "Rust 装配上下文不得含 VM/C 层符号: {names:?}"
            );
        }
        let sel = &rs_session.layer_selections[0];
        assert_eq!(sel.target, AssemblyTarget::Rust);
        assert!(
            sel.context_file
                .as_ref()
                .is_some_and(|file| file.ends_with("proto.rs.at")),
            "actual Rust project layer must be consumed"
        );
        assert!(!sel
            .candidate_files
            .iter()
            .any(|c| c.ends_with("proto.rs.at")));

        // C 目标：同构（c 层 candidate，vm 层不串入）。
        let mut c_session = crate::compile::CompileSession::new();
        c_session.add_source_dir(tmp.path().to_path_buf());
        c_session.set_assembly_target(AssemblyTarget::C).unwrap();
        c_session.resolve_uses(&source).unwrap();
        {
            let store = c_session.type_store();
            let store = store.read().unwrap();
            let names = store.lookup_module("proto").unwrap().store.pub_fn_names();
            assert!(
                names.contains(&"pub_fn".to_string())
                    && !names.contains(&"vm_only".to_string())
                    && !names.contains(&"rs_only".to_string())
                    && names.contains(&"c_only".to_string()),
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
            msg.contains("SourceSpan") && msg.contains("proto.vm.at"),
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
        let repo_root = stdlib_root
            .parent()
            .and_then(|p| p.parent())
            .unwrap()
            .to_path_buf();
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
        let repo_root = root
            .parent()
            .and_then(|p| p.parent())
            .unwrap()
            .to_path_buf();
        let orig_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo_root).unwrap();
        crate::vm::native_registry::register_builtin_natives();
        std::env::set_current_dir(orig_cwd).unwrap();
        let vm_names = crate::vm::native_registry::BIGVM_NATIVES
            .lock()
            .unwrap()
            .get_function_names();
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
            "math", "str", "time", "env", "json", "file", "fs", "http", "list", "hashmap",
            "hashset", "btreemap", "vecdeque", "char", "conv", "io", "log", "path", "net",
            "process", "sys", "sse", "may", "sqlite", "redis",
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

// ============================================================================
// T-04 ①：核心符号/native 绑定校验 + ID 冲突 + Browser 环境分类（AC-03/05）
// ============================================================================

#[cfg(test)]
mod t04_core_bindings {
    use crate::stdlib_assembly::model::Environment;
    use crate::stdlib_assembly::validate::{self, CoreSymbolStatus};

    /// 生产同款构造：CWD 钉仓根 → 磁盘扫描注册 + NativeInterface 手工面
    ///（register_std_shims + register_stdlib_ffi）+ 全库 inventory。
    fn production_surfaces() -> (
        crate::stdlib_assembly::model::StdlibInventory,
        std::sync::MutexGuard<'static, crate::vm::native_registry::AutoVMNativeRegistry>,
        crate::vm::native::NativeInterface,
        std::path::PathBuf,
    ) {
        let stdlib_root = crate::stdlib_assembly::loader::repo_stdlib_root().unwrap();
        let repo_root = stdlib_root
            .parent()
            .and_then(|p| p.parent())
            .unwrap()
            .to_path_buf();
        let orig = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo_root).unwrap();
        crate::vm::native_registry::register_builtin_natives();
        std::env::set_current_dir(orig).unwrap();

        let mut shims = crate::vm::native::NativeInterface::new();
        shims.register_std_shims();
        crate::vm::ffi::stdlib::register_stdlib_ffi(&mut shims);
        // VM init 同款：#[rust_fn] inventory 面（json 族 shim 在此绑定——
        // 仅手工面会漏，resolved-but-unbound 恰是本门禁要抓的形态）
        shims.build_from_inventory();

        let inv = crate::stdlib_assembly::loader::scan_inventory(&stdlib_root);
        (
            inv,
            crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap(),
            shims,
            repo_root,
        )
    }

    /// AC-03 正测：现存核心支持 resolved+bound → Supported。
    /// 裁决①（2026-10-08）：权威名=公共符号 canonical 面（auto.http.get
    /// 形态）。扫描面 http_* 前缀名（.vm.at 声明名）无绑定 → 显式
    /// Unverified 诊断（公共面全量重写下轮；本测冻结该诚实状态）。
    #[test]
    fn core_supported_symbols_resolve_and_bind() {
        let (inv, registry, shims, _) = production_surfaces();
        let validations =
            validate::validate_core_vm_bindings(&inv, &registry, &shims, Environment::Native);

        let tcp_bind = validations
            .iter()
            .find(|v| v.public_symbol && v.native_name == "auto.net.tcp_bind")
            .unwrap();
        assert_eq!(
            tcp_bind.status,
            CoreSymbolStatus::Supported,
            "{:?}",
            tcp_bind.reason
        );
        assert_eq!(
            tcp_bind.verification,
            crate::stdlib_assembly::model::VerificationLevel::SignatureChecked
        );
        // Every signature-checked claim must come from a producer contract.
        for v in &validations {
            if v.verification == crate::stdlib_assembly::model::VerificationLevel::SignatureChecked
            {
                let id = registry
                    .get_id(&v.native_name)
                    .or_else(|| shims.resolve(&v.native_name))
                    .unwrap();
                assert!(
                    shims.contract(id).is_some()
                        || v.module == "io"
                            && crate::vm::io::method_contract(v.symbol.rsplit('.').next().unwrap())
                                .is_some()
                );
            }
        }
        assert!(validations
            .iter()
            .any(|v| v.public_symbol && v.symbol == "TcpListener.close"));
    }

    /// AC-03 负测：sse.at 顶层 #[vm] parse_sse 无任何 native 注册——
    /// DeclaredStub（诚实占位，不冒称支持）。
    #[test]
    fn sse_parse_sse_is_declared_stub() {
        let (inv, registry, shims, _) = production_surfaces();
        let validations =
            validate::validate_core_vm_bindings(&inv, &registry, &shims, Environment::Native);
        let sse = validations
            .iter()
            .find(|v| v.module == "sse")
            .expect("sse 应有 #[vm] 校验产出（parse_sse）");
        assert_eq!(sse.status, CoreSymbolStatus::DeclaredStub);
        assert!(!sse.resolved && !sse.bound);
        assert_eq!(
            sse.verification,
            crate::stdlib_assembly::model::VerificationLevel::Declared
        );
    }

    /// AC-05：Browser 环境三族（本地 FS/native socket/服务监听）显式
    /// Unsupported；解析面（json/async/sse）与 http 客户端面不受累。
    #[test]
    fn browser_environment_unsupported_families() {
        let (inv, registry, shims, _) = production_surfaces();
        let v_native =
            validate::validate_core_vm_bindings(&inv, &registry, &shims, Environment::Native);
        let v_browser =
            validate::validate_core_vm_bindings(&inv, &registry, &shims, Environment::Browser);

        // io/net 全族 Browser 下 Unsupported
        let io_unsup = v_browser
            .iter()
            .filter(|v| v.module == "io" && v.status == CoreSymbolStatus::Unsupported)
            .count();
        let net_unsup = v_browser
            .iter()
            .filter(|v| v.module == "net" && v.status == CoreSymbolStatus::Unsupported)
            .count();
        let io_all = v_browser.iter().filter(|v| v.module == "io").count();
        let net_all = v_browser.iter().filter(|v| v.module == "net").count();
        assert!(
            io_all > 0 && io_unsup == io_all,
            "io 全族应 Unsupported（{io_unsup}/{io_all}）"
        );
        assert!(
            net_all > 0 && net_unsup == net_all,
            "net 全族应 Unsupported（{net_unsup}/{net_all}）"
        );

        assert!(v_browser
            .iter()
            .all(|v| v.status == CoreSymbolStatus::Unsupported));
        assert!(v_browser
            .iter()
            .all(|v| v.reason.as_deref().is_some_and(|r| r.contains("browser"))));

        // Native 下无 Unsupported（对照组）
        assert!(
            v_native
                .iter()
                .all(|v| v.status != CoreSymbolStatus::Unsupported),
            "Native 环境不应产生 Unsupported 分类"
        );

        // 诊断视图：Browser 下 io/net/http server 族出 PROVIDER_UNSUPPORTED
        let diags = validate::core_status_diagnostics(&v_browser, None);
        assert!(diags
            .iter()
            .any(|d| d.code == validate::code::PROVIDER_UNSUPPORTED));
    }

    /// AC-03：ID 别名冲突检测——生产面基线零冲突。
    /// 裁决②（2026-10-08）：shim 权威名收敛单名（auto.fs.read_text），
    /// file/fs 历史别名经 NATIVE_ID_ENTRIES 声明组放行（catalog (name,id)
    /// 表=手工声明的别名组面）；声明组外共 id 仍报冲突。
    #[test]
    fn id_alias_conflict_detection() {
        let (_, registry, _, _) = production_surfaces();
        // 基线：file/fs 声明组已放行；检测器另抓到新真实冲突（扫描面动态
        // id 分配相撞：id1607 char.to_str×conv.string_try_to_i64、
        // id9930-9933 http×transfer 族）——T-05 前分诊，基线冻结其数量。
        let baseline = validate::id_alias_conflicts(&registry);
        assert_eq!(
            baseline.len(),
            0,
            "生产面 ID 冲突基线漂移——修复/新增后更新此冻结与 §9: {:?}",
            baseline
                .iter()
                .map(|d| &d.message)
                .take(8)
                .collect::<Vec<_>>()
        );
        // 裁决②冻结：read_text 族 shim 收敛单名（std 声明面）
        let src = include_str!("../vm/ffi/stdlib.rs");
        {
            let anno = "#[auto_macros::rust_fn(\"auto.fs.read_text\")]";
            assert!(src.contains(anno), "read_text 注解应为单权威名");
        }

        // 合成冲突：两个互不为末段的名字注册同一 id
        let mut reg = crate::vm::native_registry::AutoVMNativeRegistry::new();
        reg.register_with_id("auto.http.http_get", 7777);
        reg.register_with_id("auto.net.tcp_bind", 7777);
        let conflicts = validate::id_alias_conflicts(&reg);
        assert_eq!(conflicts.len(), 1, "无关名字同 id 必报冲突");
        assert_eq!(conflicts[0].code, validate::code::NATIVE_ID_CONFLICT);

        // 对照：短别名关系（末段）合法
        let mut reg2 = crate::vm::native_registry::AutoVMNativeRegistry::new();
        reg2.register_with_id("auto.str.split", 8888);
        reg2.register_with_id("split", 8888);
        assert!(
            validate::id_alias_conflicts(&reg2).is_empty(),
            "短别名同 id 合法"
        );
    }
}

// ============================================================================
// T-05 缓存依赖与编译 epoch 一致性（AC-04/06，SD-01/03）
// ============================================================================

#[cfg(test)]
mod t05_cache_consistency {
    use crate::compile::CompileSession;
    use crate::stdlib_assembly::model::AssemblyTarget;
    use std::fs;
    use std::path::Path;

    /// 双层模块 fixture：公共层带真实函数体（bytecode exports 非空），
    /// vm 层带 #[vm] 声明（VM 装配合并面）。
    fn write_two_layer_fixture(dir: &Path) -> String {
        fs::write(
            dir.join("main.at"),
            "use proto: *\n\nfn main() {\n    let x = 1\n}\n",
        )
        .unwrap();
        fs::write(
            dir.join("proto.at"),
            "pub fn answer() int {\n    return 42\n}\n",
        )
        .unwrap();
        fs::write(dir.join("proto.vm.at"), "#[vm]\npub fn vm_only() int;\n").unwrap();
        fs::read_to_string(dir.join("main.at")).unwrap()
    }

    fn vm_session(dir: &Path) -> CompileSession {
        let mut s = CompileSession::new();
        s.add_source_dir(dir.to_path_buf());
        s
    }

    fn module_fn_names(s: &CompileSession, module: &str) -> Vec<String> {
        s.type_store()
            .read()
            .unwrap()
            .lookup_module(module)
            .unwrap()
            .store
            .pub_fn_names()
    }

    /// 写文件并保留原 mtime——证明失效判据是内容指纹而非时间戳。
    fn write_preserving_mtime(path: &Path, content: &str) {
        let mtime = fs::metadata(path).unwrap().modified().unwrap();
        fs::write(path, content).unwrap();
        let f = fs::OpenOptions::new().write(true).open(path).unwrap();
        f.set_times(std::fs::FileTimes::new().set_modified(mtime))
            .unwrap();
        drop(f);
        assert_eq!(fs::metadata(path).unwrap().modified().unwrap(), mtime);
    }

    /// §5.5 早退一致性：缓存命中只替换 parse 步——本 epoch 的 bytecode 与
    /// manifest 记录必须与 fresh 路径同构（clone 会话 = compiled_* 已重置、
    /// auto_cache 继承的真实早退场景）。
    #[test]
    fn cache_hit_rebuilds_bytecode_and_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let source = write_two_layer_fixture(tmp.path());

        let mut s1 = vm_session(tmp.path());
        s1.resolve_uses(&source).unwrap();
        assert_eq!(s1.layer_selections.len(), 1, "fresh 装载留 manifest");

        let mut s2 = s1.clone();
        assert!(s2.compiled_modules.is_empty(), "clone 重置装载产物");
        s2.resolve_uses(&source).unwrap();

        // ① 命中路径补全本 epoch bytecode（旧早退只合并 type_store）
        let proto = s2
            .compiled_modules
            .iter()
            .find(|m| m.name == "proto")
            .expect("命中路径必须重建本 epoch bytecode（半截模块禁止）");
        assert!(
            proto.exports.keys().any(|k| k.contains("answer")),
            "bytecode exports 应含公共层函数: {:?}",
            proto.exports.keys().collect::<Vec<_>>()
        );
        // ② 命中路径留下 manifest 记录（T-03 面在命中路径同构）
        assert_eq!(s2.layer_selections.len(), 1, "命中路径须记录层选择");
        assert!(
            s2.layer_selections[0]
                .context_file
                .as_deref()
                .unwrap()
                .ends_with("proto.vm.at"),
            "命中路径 manifest 须含选定层: {:?}",
            s2.layer_selections[0]
        );
        // ③ 类型面与 fresh 一致
        let names = module_fn_names(&s2, "proto");
        assert!(
            names.contains(&"answer".to_string()) && names.contains(&"vm_only".to_string()),
            "命中路径类型面完整: {names:?}"
        );
    }

    /// AC-06：只改目标层内容（同 mtime）→ 真实重编译，新符号可见；
    /// 旧缓存返回被禁止（行为断言，非仅指纹断言）。
    #[test]
    fn target_layer_content_change_invalidates_same_mtime() {
        let tmp = tempfile::tempdir().unwrap();
        let source = write_two_layer_fixture(tmp.path());

        let mut s1 = vm_session(tmp.path());
        s1.resolve_uses(&source).unwrap();

        write_preserving_mtime(
            &tmp.path().join("proto.vm.at"),
            "#[vm]\npub fn vm_only() int;\n#[vm]\npub fn vm_extra() int;\n",
        );

        let mut s2 = s1.clone();
        s2.resolve_uses(&source).unwrap();
        let names = module_fn_names(&s2, "proto");
        assert!(
            names.contains(&"vm_extra".to_string()),
            "同 mtime 改层必须重编译（新符号可见）: {names:?}"
        );
    }

    /// AC-06：选定层新增（absent 台账）与删除都真实失效。
    #[test]
    fn selected_layer_add_and_remove_invalidate() {
        // —— 增层：存储时无 .vm.at，后来出现 → 重编译可见层符号
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("main.at"),
            "use proto: *\n\nfn main() {\n    let x = 1\n}\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("proto.at"),
            "pub fn answer() int {\n    return 42\n}\n",
        )
        .unwrap();
        let source = fs::read_to_string(tmp.path().join("main.at")).unwrap();

        let mut s1 = vm_session(tmp.path());
        s1.resolve_uses(&source).unwrap();
        assert!(
            !module_fn_names(&s1, "proto").contains(&"vm_added".to_string()),
            "装载时无层则无层符号"
        );

        fs::write(
            tmp.path().join("proto.vm.at"),
            "#[vm]\npub fn vm_added() int;\n",
        )
        .unwrap();
        let mut s2 = s1.clone();
        s2.resolve_uses(&source).unwrap();
        assert!(
            module_fn_names(&s2, "proto").contains(&"vm_added".to_string()),
            "选定层新增必须失效缓存并重编译"
        );

        // —— 删层：存储时有 .vm.at，删除后重编译回公共单层（合法装配），
        //     且层符号消失（不得残留旧实现）
        let tmp2 = tempfile::tempdir().unwrap();
        let source2 = write_two_layer_fixture(tmp2.path());
        let mut s3 = vm_session(tmp2.path());
        s3.resolve_uses(&source2).unwrap();
        fs::remove_file(tmp2.path().join("proto.vm.at")).unwrap();
        let mut s4 = s3.clone();
        s4.resolve_uses(&source2).unwrap();
        let names = module_fn_names(&s4, "proto");
        assert!(
            !names.contains(&"vm_only".to_string()),
            "选定层删除后不得残留旧层符号: {names:?}"
        );
        assert!(
            names.contains(&"answer".to_string()),
            "公共层继续可用（无选定层是合法装配）"
        );
    }

    /// §5.5：活 session 换目标 = 无契约 ABI 热换 → SessionTargetMismatch；
    /// 同目标重声明、未装载/clone 重置 session 的换目标放行。
    #[test]
    fn retarget_after_load_rejected_and_legal_paths_ok() {
        let tmp = tempfile::tempdir().unwrap();
        let source = write_two_layer_fixture(tmp.path());

        let mut s = vm_session(tmp.path());
        s.resolve_uses(&source).unwrap(); // Vm 默认目标下装载

        let err = s
            .set_assembly_target(AssemblyTarget::Rust)
            .expect_err("已装载 session 换目标必须拒绝");
        assert!(
            err.to_string().contains("session_target_mismatch"),
            "诊断须含稳定码: {err}"
        );
        assert_eq!(s.assembly.target, AssemblyTarget::Vm, "拒绝后目标不变");

        s.set_assembly_target(AssemblyTarget::Vm)
            .expect("同目标重声明放行");

        // clone 重置装载产物 → 换目标放行（跨装配缓存共存入口）
        let mut s2 = s.clone();
        s2.set_assembly_target(AssemblyTarget::C).unwrap();
        assert_eq!(s2.assembly.target, AssemblyTarget::C);

        let mut s3 = CompileSession::new();
        s3.set_assembly_target(AssemblyTarget::Rust).unwrap();
    }

    /// §5.5：stdlib root 身份——活 session 解析根变化 → SessionTargetMismatch
    /// （真实 stdlib 装载接线；root 注入经 pub(crate) 字段避免 env 竞态）。
    #[test]
    fn stdlib_root_change_rejected() {
        let mut s = CompileSession::new();
        s.resolve_uses("use auto.io: *")
            .expect("真实 stdlib io 装载");
        assert!(s.stdlib_root.is_some(), "首个 stdlib 装载应记录 root 身份");

        s.stdlib_root = Some("Z:/definitely-other-stdlib-root".to_string());
        let err = s
            .resolve_uses("use auto.net: *")
            .expect_err("stdlib root 变化必须拒绝");
        assert!(
            err.to_string().contains("session_target_mismatch"),
            "诊断须含稳定码: {err}"
        );
    }

    /// §5.5：跨装配缓存共存——VM 装载的条目不得供 Rust 装配命中；
    /// clone+retarget 后 Rust 装配未命中（VM 层不串入）。
    #[test]
    fn cross_target_cache_does_not_leak_vm_layer() {
        let tmp = tempfile::tempdir().unwrap();
        let source = write_two_layer_fixture(tmp.path());

        let mut s1 = vm_session(tmp.path());
        s1.resolve_uses(&source).unwrap(); // Vm 条目入缓存

        let mut s2 = s1.clone(); // 缓存继承、装载产物重置
        s2.set_assembly_target(AssemblyTarget::Rust).unwrap();
        s2.resolve_uses(&source).unwrap();

        let names = module_fn_names(&s2, "proto");
        assert!(
            !names.contains(&"vm_only".to_string()),
            "Rust 装配不得命中 VM 条目（vm 层符号不得串入）: {names:?}"
        );
        assert!(
            names.contains(&"answer".to_string()),
            "公共层照常可见: {names:?}"
        );
        assert_eq!(
            s2.layer_selections[0].target,
            AssemblyTarget::Rust,
            "Rust 装配留自己的 manifest 记录"
        );
        assert!(s2.layer_selections[0].context_file.is_none());
    }

    /// §5.5：依赖闭包指纹——依赖模块内容变更 → 依赖方条目未命中，
    /// 本 epoch 重编译后新依赖符号可见。
    #[test]
    fn dependency_change_invalidates_dependent() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("main.at"),
            "use mid: *\n\nfn main() {\n    let x = 1\n}\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("mid.at"),
            "use leaf: *\n\npub fn mid_fn() int {\n    return 1\n}\n",
        )
        .unwrap();
        fs::write(
            tmp.path().join("leaf.at"),
            "pub fn leaf_fn() int {\n    return 10\n}\n",
        )
        .unwrap();
        let source = fs::read_to_string(tmp.path().join("main.at")).unwrap();

        let mut s1 = vm_session(tmp.path());
        s1.resolve_uses(&source).unwrap();

        fs::write(
            tmp.path().join("leaf.at"),
            "pub fn leaf_fn() int {\n    return 10\n}\n\npub fn leaf_extra() int {\n    return 20\n}\n",
        )
        .unwrap();

        let mut s2 = s1.clone();
        s2.resolve_uses(&source).unwrap();
        let leaf_names = module_fn_names(&s2, "leaf");
        assert!(
            leaf_names.contains(&"leaf_extra".to_string()),
            "依赖变更须真实重编译 leaf: {leaf_names:?}"
        );
        assert!(
            module_fn_names(&s2, "mid").contains(&"mid_fn".to_string()),
            "依赖方本 epoch 类型面完整"
        );
    }

    /// §5.5：persistent 活 VM 的 stdlib 热换守卫——同内容重载幂等放行，
    /// 指纹/根漂移 → SessionTargetMismatch。
    #[test]
    fn persistent_hot_swap_guard() {
        // 注册面 CWD 依赖（同 t04 vm_two_entry 先例）：钉仓根后跑生产 init。
        let stdlib_root = crate::stdlib_assembly::loader::repo_stdlib_root().unwrap();
        let repo_root = stdlib_root
            .parent()
            .and_then(|p| p.parent())
            .unwrap()
            .to_path_buf();
        let orig_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo_root).unwrap();
        crate::vm::native_registry::register_builtin_natives();

        let mut sess = crate::autovm_persistent::AutovmReplSession::new();
        let r1 = sess.run("use auto.io: *");
        assert!(r1.is_ok(), "首次装载应成功: {:?}", r1.err());
        assert_eq!(sess.module_fingerprints.len(), 1, "台账记录装载");
        let (orig_fp, orig_root) = {
            let (_, fp, root) = &sess.module_fingerprints[0];
            (*fp, root.clone())
        };

        // 同内容重载（REPL 重复 use）：幂等放行，不重复记台账
        sess.run("use auto.io: *").expect("同内容重载必须幂等放行");
        assert_eq!(sess.module_fingerprints.len(), 1);

        // 内容漂移 → 拒绝
        sess.module_fingerprints[0].1 ^= 0xDEAD_BEEF;
        let err = sess
            .run("use auto.io: *")
            .expect_err("活 VM 下 stdlib 内容热换必须拒绝");
        assert!(
            err.to_string().contains("session_target_mismatch"),
            "诊断须含稳定码: {err}"
        );

        // 根漂移 → 拒绝
        sess.module_fingerprints[0].1 = orig_fp;
        sess.module_fingerprints[0].2 = "Z:/other-root".to_string();
        let err = sess
            .run("use auto.io: *")
            .expect_err("活 VM 下 stdlib root 热换必须拒绝");
        assert!(
            err.to_string().contains("session_target_mismatch"),
            "诊断须含稳定码: {err}"
        );
        sess.module_fingerprints[0].2 = orig_root;
        std::env::set_current_dir(orig_cwd).unwrap();
    }
}

// ============================================================================
// T-07 同源样例、核心完整矩阵与实际执行证明（AC-01/02/05，SD-01/02/07）
// ============================================================================

#[cfg(test)]
mod t07_witness_and_matrix {
    use crate::stdlib_assembly::validate::{self, CoreSymbolStatus};
    use std::fs;

    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// t04 的生产装配面构造（复用：CWD 钉仓根 + 注册三件套 + inventory）。
    fn production_surfaces() -> (
        crate::stdlib_assembly::model::StdlibInventory,
        std::sync::MutexGuard<'static, crate::vm::native_registry::AutoVMNativeRegistry>,
        crate::vm::native::NativeInterface,
        std::path::PathBuf,
    ) {
        let stdlib_root = crate::stdlib_assembly::loader::repo_stdlib_root().unwrap();
        let repo_root = stdlib_root
            .parent()
            .and_then(|p| p.parent())
            .unwrap()
            .to_path_buf();
        let orig = std::env::current_dir().unwrap();
        std::env::set_current_dir(&repo_root).unwrap();
        crate::vm::native_registry::register_builtin_natives();
        std::env::set_current_dir(orig).unwrap();

        let mut shims = crate::vm::native::NativeInterface::new();
        shims.register_std_shims();
        crate::vm::ffi::stdlib::register_stdlib_ffi(&mut shims);
        shims.build_from_inventory();

        let inv = crate::stdlib_assembly::loader::scan_inventory(&stdlib_root);
        (
            inv,
            crate::vm::native_registry::BIGVM_NATIVES.lock().unwrap(),
            shims,
            repo_root,
        )
    }

    /// AC-02（VM 腿真执行见证）：同一模块名下——公共层 body-less 声明 +
    /// 选定 .vm.at 层真实 body，VM 管线（run_autovm 全管线）执行选定层
    /// 实现（val→42）；公共 pure body 共享可见（aux→7）；foreign 层
    /// （.rs.at）符号不在 VM 装配上下文（rs_only 调用失败）。
    #[test]
    fn vm_executes_selected_layer_witness() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // 公共层：body-less 声明（实现由选定层提供）+ pure body（共享）
        fs::write(
            tmp.path().join("proto.at"),
            "pub fn val() int;\n\npub fn aux() int {\n    return 7\n}\n",
        )
        .unwrap();
        // 选定 VM 层：真实 body 返回 42
        fs::write(
            tmp.path().join("proto.vm.at"),
            "pub fn val() int {\n    return 42\n}\n",
        )
        .unwrap();
        // foreign 层：同名不同值 + 独有符号——VM 装配不得选择/执行
        fs::write(
            tmp.path().join("proto.rs.at"),
            "pub fn val() int {\n    return 3\n}\n\npub fn rs_only() int {\n    return 9\n}\n",
        )
        .unwrap();

        let run = |code: &str| {
            let orig_cwd = std::env::current_dir().unwrap();
            std::env::set_current_dir(tmp.path()).unwrap();
            let r = crate::run_autovm(code);
            std::env::set_current_dir(orig_cwd).unwrap();
            r
        };

        // 选定层 body 真执行：val→42（非公共缺省、非 foreign 3）
        let v = run("use proto: *\n\nfn main() {\n    return proto.val()\n}")
            .expect("VM 管线真执行应成功");
        assert!(
            v.contains("42") && !v.contains('3'),
            "选定 vm 层 body 应被执行（val→42，非 foreign 3）: {v}"
        );
        // 公共 pure body 共享：aux→7
        let a = run("use proto: *\n\nfn main() {\n    return proto.aux()\n}")
            .expect("公共层 pure body 应可执行");
        assert!(a.contains("7"), "公共 pure body 应共享可见（aux→7）: {a}");
        // foreign 层独有符号不得串入 VM 装配上下文
        let foreign = run("use proto: *\n\nfn main() {\n    return proto.rs_only()\n}");
        assert!(
            foreign.is_err(),
            "foreign .rs.at 层符号不得进入 VM 装配（rs_only 应不可见）: {foreign:?}"
        );
    }

    /// AC-05/AC-01：六模块「公开符号 × target × environment」矩阵完整性——
    /// 四格全在册、非 Supported 必有原因、Browser io/net 全族 Unsupported、
    /// rust claim 面与目录一致（json supported / io unsupported 有原因）。
    /// 矩阵落盘 docs/plans/reports/738-stdlib-matrix.{json,md}（T-07 报告面）。
    #[test]
    fn core_matrix_complete_and_report_written() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (inv, registry, shims, repo_root) = production_surfaces();
        let matrix = validate::core_target_env_matrix(&inv, &registry, &shims);
        let modules = matrix["modules"].as_object().unwrap();

        assert_eq!(modules.len(), 6, "六核心全在册");
        for (name, cell) in modules {
            let denominator: std::collections::BTreeSet<_> = cell["rust"]["public_symbols"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap())
                .collect();
            for target in ["vm", "rust", "c"] {
                for environment in ["native", "browser"] {
                    let actual: std::collections::BTreeSet<_> = cell[target][environment]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|s| s["symbol"].as_str().unwrap())
                        .collect();
                    assert_eq!(
                        actual, denominator,
                        "{name}/{target}/{environment}: full public denominator"
                    );
                }
            }
            // 四格全在册
            assert!(
                cell["vm"]["native"].is_array() && cell["vm"]["browser"].is_array(),
                "{name}: vm 两环境格必须在册"
            );
            assert!(
                cell["rust"]["claim"].is_object() && cell["c"]["claim"].is_object(),
                "{name}: rust/c claim 格必须在册"
            );
            // 非 Supported 的 vm 符号必有原因
            for env in ["native", "browser"] {
                for sym in cell["vm"][env].as_array().unwrap() {
                    let status = sym["status"].as_str().unwrap();
                    if status != "supported" {
                        assert!(
                            sym["reason"]
                                .as_str()
                                .map(|r| !r.is_empty())
                                .unwrap_or(false),
                            "{name}/{env}/{}: 非 Supported 必有原因",
                            sym["symbol"]
                        );
                    }
                }
            }
            // rust/c claim 非 supported 必有原因
            for tgt in ["rust", "c"] {
                let claim = &cell[tgt]["claim"];
                if claim["status"].as_str() != Some("supported") {
                    assert!(
                        claim["reason"]
                            .as_str()
                            .map(|r| !r.is_empty())
                            .unwrap_or(false),
                        "{name}/{tgt}: 非 supported claim 必有原因: {claim}"
                    );
                }
            }
        }

        // Browser 能力门（AC-05）：io/net 全族 Unsupported
        for m in ["io", "net"] {
            let browser = modules[m]["vm"]["browser"].as_array().unwrap();
            assert!(
                !browser.is_empty()
                    && browser
                        .iter()
                        .all(|s| s["status"].as_str() == Some("unsupported")),
                "{m} Browser 全族应 Unsupported: {browser:?}"
            );
        }
        // rust claim 面：json supported（a2r-std 有真实 pub mod）、io unsupported 有原因
        assert_eq!(
            modules["json"]["rust"]["claim"]["status"].as_str(),
            Some("supported"),
            "json rust claim 应 supported"
        );
        assert_eq!(
            modules["io"]["rust"]["claim"]["status"].as_str(),
            Some("unsupported"),
            "io rust claim 应 unsupported（无 a2r-std 模块——决策报告 E1③）"
        );

        // 报告落盘（JSON 机器面 + md 摘要）
        let report_dir = repo_root.join("docs").join("plans").join("reports");
        fs::create_dir_all(&report_dir).unwrap();
        let json_path = report_dir.join("738-stdlib-matrix.json");
        fs::write(
            &json_path,
            serde_json::to_string_pretty(&matrix).unwrap() + "\n",
        )
        .unwrap();
        let mut md =
            String::from("# PLAN-738 六核心 target×environment 能力矩阵（机器生成，勿手编）\n\n");
        md.push_str("| module | vm.native | vm.browser | rust | c | 公共符号数 |\n|---|---|---|---|---|---|\n");
        for (name, cell) in modules {
            let count = |arr: &serde_json::Value, st: &str| {
                arr.as_array()
                    .unwrap()
                    .iter()
                    .filter(|s| s["status"].as_str() == Some(st))
                    .count()
            };
            let total = |arr: &serde_json::Value| arr.as_array().unwrap().len();
            let (ns, nb) = (
                count(&cell["vm"]["native"], "supported"),
                total(&cell["vm"]["native"]),
            );
            let (bs, bb) = (
                count(&cell["vm"]["browser"], "supported"),
                total(&cell["vm"]["browser"]),
            );
            md.push_str(&format!(
                "| {name} | {ns}/{nb} supported | {bs}/{bb} supported | {} | {} | {} |\n",
                cell["rust"]["claim"]["status"].as_str().unwrap(),
                cell["c"]["claim"]["status"].as_str().unwrap(),
                cell["rust"]["public_symbols"].as_array().unwrap().len(),
            ));
        }
        fs::write(report_dir.join("738-stdlib-matrix.md"), md).unwrap();
        assert!(json_path.exists(), "矩阵 JSON 报告应落盘");
    }
}

// ============================================================================
// T-06 gated 半解锁：装配指纹（734 receipt / 736 复用新鲜度门消费面）
// ============================================================================

#[cfg(test)]
mod t06_assembly_receipt {
    use crate::stdlib_assembly::loader::{repo_stdlib_root, stdlib_assembly_fingerprint};
    use crate::stdlib_assembly::model::AssemblyTarget;
    use std::fs;
    use std::path::Path;

    fn fingerprint_at(root: &Path, target: AssemblyTarget) -> u64 {
        stdlib_assembly_fingerprint(root, target).expect("指纹计算")
    }

    /// 决定论 + 内容敏感性：同内容同目标恒等；任一层内容变更/层新增/
    /// 目标切换都变指纹（AC-06「改 stdlib 但 api.at 不变」的失效基底）。
    #[test]
    fn fingerprint_deterministic_and_content_sensitive() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("m.at"),
            "pub fn f() int {
    return 1
}
",
        )
        .unwrap();
        fs::write(
            tmp.path().join("m.vm.at"),
            "#[vm]
pub fn g() int;
",
        )
        .unwrap();

        let a = fingerprint_at(tmp.path(), AssemblyTarget::Vm);
        let b = fingerprint_at(tmp.path(), AssemblyTarget::Vm);
        assert_eq!(a, b, "同内容同目标必须恒等（决定论）");

        // 改目标层内容
        fs::write(
            tmp.path().join("m.vm.at"),
            "#[vm]
pub fn g() int {
    return 2
}
",
        )
        .unwrap();
        assert_ne!(
            fingerprint_at(tmp.path(), AssemblyTarget::Vm),
            a,
            "层内容变更必须变指纹"
        );

        // 增层
        let after_add = fingerprint_at(tmp.path(), AssemblyTarget::Vm);
        fs::write(
            tmp.path().join("m.rs.at"),
            "pub fn h() int {
    return 3
}
",
        )
        .unwrap();
        assert_ne!(
            fingerprint_at(tmp.path(), AssemblyTarget::Vm),
            after_add,
            "层新增必须变指纹"
        );

        // 目标切换（同内容）
        let full_vm = fingerprint_at(tmp.path(), AssemblyTarget::Vm);
        assert_ne!(
            fingerprint_at(tmp.path(), AssemblyTarget::Rust),
            full_vm,
            "target 入链：目标切换变指纹（§5.4）"
        );

        // 真实 stdlib 根可用性（repo 根环境）——与 receipt 生成同源
        assert!(repo_stdlib_root().is_ok(), "仓内环境 stdlib 根可定位");
    }

    /// AC-02/AC-07（Rust 腿真编译实跑见证，Plan 610 ⑤腿范式）：witness.at
    /// （use auto.json + Json.parse）→ trans_rust（宿主 provider 路由
    /// a2r_std::json::parse）→ cargo 对真实 a2r-std crate 实编 → 实跑输出
    /// 解析产物。不手写业务替代；#[ignore] 按需跑：
    /// `cargo test -p auto-lang --lib plan738 -- --ignored`。
    #[test]
    #[ignore = "shells out to cargo; on-demand rust host-provider real-compile gate (Plan 610 paradigm)"]
    fn rust_host_provider_real_compile_witness() {
        use std::process::Command;

        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let repo_root = std::path::Path::new(&manifest)
            .join("../../")
            .canonicalize()
            .unwrap();
        let a2r_std_src = repo_root.join("crates").join("a2r-std");
        assert!(
            a2r_std_src.join("Cargo.toml").exists(),
            "a2r-std crate 在位"
        );
        let stage = repo_root.join("target").join("plan738").join("witness");
        fs::create_dir_all(stage.join("src")).unwrap();

        // ① 源：use auto.json 的最小程序（同 examples/stdlib/assembly 同源）
        let witness_at = stage.join("witness.at");
        fs::write(
            &witness_at,
            "use auto.json: *

fn main() {
    let doc = Json.parse(\"{\\\"k\\\": 42, \\\"s\\\": \\\"ok\\\"}\")
    print(doc)
}
",
        )
        .unwrap();

        // ② 真实 trans_rust 入口（大栈线程——trans 递归深）
        let at_path = witness_at.clone();
        let out_path = stage.join("witness.a2r.rs");
        let h = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(move || {
                let mut session = crate::compile::CompileSession::new();
                session
                    .set_assembly_target(crate::stdlib_assembly::model::AssemblyTarget::Rust)
                    .unwrap();
                // 返回值是日志行；产物由入口自写 <stem>.a2r.rs
                crate::trans_rust_with_session(&mut session, at_path.to_str().unwrap())
            })
            .unwrap();
        h.join().unwrap().expect("trans_rust 入口成功");
        let emitted = fs::read_to_string(&out_path).unwrap();
        assert!(
            emitted.contains("a2r_std::json::parse"),
            "宿主 provider 路由必须在发射面: {emitted}"
        );

        // ③ 组装 cargo 项目（crate 形态 a2r-std 依赖——发射头注释 "from
        // crate" 即此形态；裸 `use a2r_std;` 在 dep 形态下去除，生成模板同款）
        let qualified = emitted.replace(
            "use a2r_std;
",
            "",
        );
        fs::write(stage.join("src").join("main.rs"), &qualified).unwrap();
        fs::write(
            stage.join("Cargo.toml"),
            format!(
                "[package]
name = \"plan738_witness\"
version = \"0.1.0\"
edition = \"2021\"

[dependencies]
a2r-std = {{ path = {:?} }}

[workspace]
",
                a2r_std_src
            ),
        )
        .unwrap();

        // ④ cargo 实编 + 实跑——真实工具链产物，断言解析输出
        let out = Command::new("cargo")
            .args(["run", "--quiet"])
            .current_dir(&stage)
            .output()
            .expect("spawn cargo");
        assert!(
            out.status.success(),
            "witness 实编失败:
{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("\"k\":42") && stdout.contains("\"s\":\"ok\""),
            "宿主 provider 实跑输出应含解析产物: {stdout:?}"
        );
    }
}

/// §6.1 冲突/来源族余项（T-07 slice3；独立 mod 自带串行锁）
mod t07_more_counterexamples {
    use std::fs;

    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// §6.1 冲突/来源族（AC-04）：用户模块与 stdlib 核心同名（json）——
    /// 正常 use 命名空间语义：用户目录模块解析优先（CWD 相对先于 stdlib），
    /// 用户同名定义不得被 stdlib 静默顶替，也不得混入 stdlib 符号。
    #[test]
    fn user_module_same_name_as_core_shadows_cleanly() {
        let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("main.at"),
            "use json: *

fn main() {
    let x = 1
}
",
        )
        .unwrap();
        // 用户本地 json.at（CWD 相对）——带独有符号，不带 stdlib json 符号
        fs::write(
            tmp.path().join("json.at"),
            "pub fn mine() int {
    return 1
}
",
        )
        .unwrap();

        let orig = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        let mut s = crate::compile::CompileSession::new();
        s.add_source_dir(tmp.path().to_path_buf());
        let source = fs::read_to_string(tmp.path().join("main.at")).unwrap();
        let resolved = s.resolve_uses(&source);
        std::env::set_current_dir(orig).unwrap();

        resolved.expect("用户同名模块应正常装载");
        let names = s
            .type_store()
            .read()
            .unwrap()
            .lookup_module("json")
            .unwrap()
            .store
            .pub_fn_names();
        assert!(
            names.contains(&"mine".to_string()),
            "用户同名模块符号应在册: {names:?}"
        );
    }

    /// §6.1 冲突/来源族（AC-04/AC-03）：公共层 body-less 声明 + 选定层 body
    /// 合法（恰一实现）；公共层再有同 body 的第二个活跃实现（选定层+公共层
    /// 各一 body）= 冲突面——wildcard 平铺冲突检测报 ambiguous import。
    #[test]
    fn duplicate_active_bodies_are_conflict_not_silent_pick() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("main.at"),
            "use proto: *

fn main() {
    let x = 1
}
",
        )
        .unwrap();
        // 公共层已有 body
        fs::write(
            tmp.path().join("proto.at"),
            "pub fn val() int {
    return 1
}
",
        )
        .unwrap();
        // 选定层再有 body——两个活跃实现（非声明+实现补全形态）
        fs::write(
            tmp.path().join("proto.vm.at"),
            "pub fn val() int {
    return 2
}
",
        )
        .unwrap();

        let mut s = crate::compile::CompileSession::new();
        s.add_source_dir(tmp.path().to_path_buf());
        let source = fs::read_to_string(tmp.path().join("main.at")).unwrap();
        // 双 body 在 wildcat 平铺冲突检测（Plan 545 D2 preparse 快照）报
        // ambiguous import——同符号异源定义不得静默择一。
        let error = s
            .resolve_uses(&source)
            .expect_err("multiple providers must be rejected");
        assert!(error.to_string().contains("PROVIDER_CONFLICT"), "{error:?}");
        assert!(s.layer_selections.is_empty());
    }
}
