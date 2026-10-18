//! PLAN-807: 文本基础抽取——核心剪断锚/快照端口装配身份/类型身份/真实
//! 编辑器路径/分页资源退出观测（in-crate 面）。
//!
//! 外部 no-UI consumer oracle 见
//! `test/fixtures/modularity/text-core-consumer`（V-06：独立 fixture 仅依
//! 赖 auto-text-core，闭包零语言/UI/VM）。本模块：
//! - core 剪断（永久回归锚）：vm/native.rs 的 diff 四 shim 不再含
//!   `ui::code_editor::diff::` 直引形态（供值经 auto-text-core 直调 +
//!   EditorSnapshotProvider 端口）。
//! - 快照端口装配身份（802 同款 fake 面）：构造式缺席/构造期快照隔离/
//!   实例覆盖只影响本 VM/NI Clone 与 merge 保留/注入不动 binding_revision。
//! - 类型身份（V-01）：旧路径门面与 auto-text-core 同一类型；真实编辑器
//!   注册表经旧门面 envelope 与 provider+core 纯形逐字节一致。
//! - 资源退出（V-03 旁证）：分页 rope 最后外部引用释放后 Windows 同名
//!   重建成功（delete-pending 语义——句柄全关前的硬观测）+ worker 退出
//!   句柄翻转。

// ── core 剪断锚（feature 中性） ──────────────────────────────────────────

mod plan807_core_cut {
    /// core 剪断（永久回归锚）：9915/9916/9917/9921 的直引形态消失。
    /// `code_editor::diff::envelope::` 是迁移前四 shim 的唯一 UI 直引面。
    #[test]
    fn diff_shims_have_no_ui_direct_reference() {
        let native_rs = include_str!("../vm/native.rs");
        assert!(
            !native_rs.contains("ui::code_editor::diff::"),
            "vm/native.rs 不得再含 ui::code_editor::diff:: 直引形态（PLAN-807 \
             剪断；纯面直调 auto-text-core，快照经 EditorSnapshotProvider）"
        );
        let a2r = include_str!("../a2r_std.rs");
        assert!(
            !a2r.contains("ui::code_editor::diff::"),
            "a2r_std.rs 不得再含 ui::code_editor::diff:: 直引形态（PLAN-807）"
        );
    }
}

// ── 类型身份（旧门面路径在 code-editor 门内——关门形态下身份由独立
//    consumer fixture 的 auto-text-core 单源消费钉住，不走 UI 门面） ────

#[cfg(feature = "code-editor")]
mod plan807_identity {
    /// V-01：旧路径门面与 auto-text-core 是**同一类型**（薄重导出，非包装）
    /// ——同一快照可跨新旧 API 传递。
    #[test]
    fn old_facade_types_are_core_types() {
        use std::any::TypeId;
        assert_eq!(
            TypeId::of::<crate::ui::code_editor::core::rope::Rope>(),
            TypeId::of::<auto_text_core::Rope>()
        );
        assert_eq!(
            TypeId::of::<crate::ui::code_editor::core::rope::RopeSnapshot>(),
            TypeId::of::<auto_text_core::RopeSnapshot>()
        );
        assert_eq!(
            TypeId::of::<crate::ui::code_editor::core::rope::PageStore>(),
            TypeId::of::<auto_text_core::PageStore>()
        );
        // 跨 API 传递：旧路径产出的快照直接喂新 core 的 diff 入口。
        let mut r = crate::ui::code_editor::core::rope::Rope::from_str("a\nb\n");
        let snap = r.snapshot();
        r.insert_bytes(4, "c\nd\n");
        let out = auto_text_core::diff::diff_snapshots(&snap, &r.snapshot(), Default::default());
        assert_eq!(out.adds, 2, "跨 API 快照 diff 可用");
    }
}

// ── 快照端口装配身份 + shim 语义（code-editor 门） ─────────────────────

#[cfg(feature = "code-editor")]
mod plan807_snapshot_port {
    use std::sync::Arc;

    use crate::vm::engine::AutoVM;
    use crate::vm::host_capabilities::EditorSnapshotProvider;
    use crate::vm::native::shim_diff_snapshots;
    use crate::vm::native::NativeInterface;
    use crate::vm::task::AutoTask;
    use crate::vm::virt_memory::VirtualFlash;

    /// fake 快照 provider：键 `"have"` 返回以构造参数为内容的快照，其余
    /// None——内容即实例标签，可区分不同默认/覆盖（F807-R1-01）。
    struct FakeSnap(&'static str);
    impl EditorSnapshotProvider for FakeSnap {
        fn snapshot(&self, key: &str) -> Option<auto_text_core::RopeSnapshot> {
            if key == "have" {
                Some(auto_text_core::Rope::from_str(self.0).snapshot())
            } else {
                None
            }
        }
    }

    /// 读出指定 VM 当前 provider 对 `"have"` 的快照内容（None 保留为
    /// None——区分"无 provider"与"provider 无此键"）。
    fn snap_text(vm: &AutoVM) -> Option<String> {
        vm.editor_snapshot_provider()
            .and_then(|p| p.snapshot("have"))
            .map(|s| s.to_string())
    }

    fn call_diff_snapshots(vm: &AutoVM, key_a: &str, key_b: &str) -> String {
        let mut task = AutoTask::new(0, 256, 0);
        // Args push left-to-right: key_a 先入栈，key_b 后入（shim 反序 pop）。
        let ia = vm.add_string(key_a.as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, ia as usize);
        let ib = vm.add_string(key_b.as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, ib as usize);
        shim_diff_snapshots(&mut task, vm).unwrap();
        let nv = task.ram.pop_nv();
        let i = auto_val::decode_string(nv);
        vm.get_string(i)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default()
    }

    /// 装配身份：构造式缺席/构造期快照隔离（显式 A/B 默认，不假设全局
    /// 槽初态——F807-R1-01：普通运行入口的 if-absent 安装不取本测试隔离
    /// 锁，共享进程的默认槽可能已被宿主装配占用，"空槽才成立"的断言在
    /// 联跑下确定性失败）/实例覆盖只影响本 VM/Clone·merge 保留/注入不动
    /// binding_revision（802 同款面，快照槽镜像）。
    #[test]
    fn snapshot_port_assembly_identity() {
        let _slot = crate::vm::host_capabilities::test_default_slot_lock();
        let saved = crate::vm::host_capabilities::process_default_editor_snapshot_provider();

        // 构造式缺席（构造式路径，不依赖全局槽时序）：跳过快照=无 provider。
        let bare = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
        assert!(bare.editor_snapshot_provider().is_none(), "构造式缺席");

        // 快照隔离：两个可区分的**显式非空默认**——vm1 构造期快照 A；
        // 换显式默认 B 后 vm2 快照 B，vm1 仍 A（后改默认不影响已建实例）。
        crate::vm::host_capabilities::set_process_default_editor_snapshot_provider(Arc::new(
            FakeSnap("A\n"),
        ));
        let vm1 = AutoVM::new(VirtualFlash::new(0), 256);
        crate::vm::host_capabilities::set_process_default_editor_snapshot_provider(Arc::new(
            FakeSnap("B\n"),
        ));
        let vm2 = AutoVM::new(VirtualFlash::new(0), 256);
        assert_eq!(
            snap_text(&vm1).as_deref(),
            Some("A\n"),
            "vm1 保持构造期快照 A"
        );
        assert_eq!(
            snap_text(&vm2).as_deref(),
            Some("B\n"),
            "vm2 快照替换后默认 B"
        );
        crate::vm::host_capabilities::set_process_default_editor_snapshot_provider(Arc::new(
            FakeSnap("D\n"),
        ));
        assert_eq!(
            snap_text(&vm2).as_deref(),
            Some("B\n"),
            "后改进程默认不影响已建实例"
        );

        // 实例覆盖只影响本 VM。
        vm1.set_editor_snapshot_provider(Arc::new(FakeSnap("C\n")));
        assert_eq!(snap_text(&vm1).as_deref(), Some("C\n"), "实例覆盖生效");
        assert_eq!(
            snap_text(&vm2).as_deref(),
            Some("B\n"),
            "兄弟 VM 不受覆盖影响"
        );

        // 注入不 bump binding_revision；NI Clone 保 provider；VM merge 保。
        let ni = NativeInterface::production();
        let rev0 = ni.binding_revision();
        ni.set_editor_snapshot_provider(Arc::new(FakeSnap("NI\n")));
        assert_eq!(ni.binding_revision(), rev0, "注入不 bump revision");
        assert!(
            ni.clone().editor_snapshot_provider().is_some(),
            "NI Clone 保 provider"
        );
        let mut vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
        vm.set_editor_snapshot_provider(Arc::new(FakeSnap("NI\n")));
        let mut carrier = NativeInterface::new();
        carrier.register_dynamic(|_t: &mut AutoTask, _v: &AutoVM| Ok(()));
        vm.merge_native_interface(&carrier);
        assert!(
            vm.editor_snapshot_provider().is_some(),
            "VM merge 保 provider"
        );

        // 还原全局槽。
        match saved {
            Some(p) => {
                crate::vm::host_capabilities::set_process_default_editor_snapshot_provider(p)
            }
            None => crate::vm::host_capabilities::clear_process_default_editor_snapshot_provider(),
        }
    }

    /// 真实宿主先安装的永久控制（F807-R1-01 要求；F779-R2-01 优先关系
    /// 镜像锚，plan802 同款纪律）：兼容 if-absent 安装**不覆盖**显式默认；
    /// 空槽补缺装上的是真实 UI adapter（对未注册键诚实返回 None，不伪造
    /// 快照——真宿主面控制，非 fake 冒充）。
    #[test]
    fn compat_install_never_overrides_explicit_default() {
        let _slot = crate::vm::host_capabilities::test_default_slot_lock();
        let saved = crate::vm::host_capabilities::process_default_editor_snapshot_provider();

        // 显式默认先在位 → 兼容安装（真实 UI adapter if-absent）不覆盖。
        crate::vm::host_capabilities::set_process_default_editor_snapshot_provider(Arc::new(
            FakeSnap("EXPLICIT\n"),
        ));
        crate::ui::code_editor_provider::install_process_default();
        assert_eq!(
            crate::vm::host_capabilities::process_default_editor_snapshot_provider()
                .and_then(|p| p.snapshot("have"))
                .map(|s| s.to_string()),
            Some("EXPLICIT\n".to_string()),
            "兼容安装不得覆盖显式默认"
        );

        // 空槽 → 兼容安装补上真实 adapter：在位、对未注册键返回 None。
        crate::vm::host_capabilities::clear_process_default_editor_snapshot_provider();
        crate::ui::code_editor_provider::install_process_default();
        let installed = crate::vm::host_capabilities::process_default_editor_snapshot_provider();
        assert!(installed.is_some(), "空槽补缺装上真实 adapter");
        assert!(
            installed.unwrap().snapshot("p807-unregistered").is_none(),
            "真实 adapter 对未注册键诚实返回 None（真宿主面）"
        );

        // 还原全局槽。
        match saved {
            Some(p) => {
                crate::vm::host_capabilities::set_process_default_editor_snapshot_provider(p)
            }
            None => crate::vm::host_capabilities::clear_process_default_editor_snapshot_provider(),
        }
    }

    /// shim 语义（V-04）：在位 provider 双有效键 → hunk 净形；缺 key_b →
    /// err 形 missing=key_b；缺 provider → err 形 missing=key_a（无法找到
    /// 相应 editor 的冻结映射）；判错优先 key_a 先。
    #[test]
    fn shim_snapshot_provider_value_and_missing_semantics() {
        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 256);

        // 缺 provider：值 err（非 raise），missing=key_a。
        let json = call_diff_snapshots(&vm, "ka", "kb");
        assert!(
            json.contains("\"err\":\"编辑器不存在: ka\""),
            "缺 provider → 无法找到相应 editor（missing=key_a）: {json}"
        );
        assert!(json.contains("\"hunks\":[]"), "{json}");

        // 实例注入后：双有效键出 hunk。
        vm.set_editor_snapshot_provider(Arc::new(FakeSnap(
            "a
b
",
        )));
        let json = call_diff_snapshots(&vm, "have", "have");
        assert!(json.contains("\"err\":\"\""), "同键自 diff 空: {json}");
        assert!(json.contains("\"adds\":0,\"dels\":0"), "{json}");

        // key_a 在位、key_b 缺 → missing=key_b；key_a 缺 → missing=key_a
        // （判错优先冻结序）。
        let json = call_diff_snapshots(&vm, "have", "nope");
        assert!(
            json.contains("\"err\":\"编辑器不存在: nope\""),
            "缺 key_b → missing=key_b: {json}"
        );
        let json = call_diff_snapshots(&vm, "nope", "have");
        assert!(
            json.contains("\"err\":\"编辑器不存在: nope\""),
            "缺 key_a 优先: {json}"
        );
    }

    /// 双 VM 隔离：两实例各持不同 provider（一 fake 一缺席），互不串线。
    #[test]
    fn two_vm_isolation() {
        let a = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 256);
        let b = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 256);
        a.set_editor_snapshot_provider(Arc::new(FakeSnap(
            "a
b
",
        )));
        let json_a = call_diff_snapshots(&a, "have", "have");
        let json_b = call_diff_snapshots(&b, "have", "have");
        assert!(json_a.contains("\"err\":\"\""), "a 走 fake provider");
        assert!(
            json_b.contains("\"err\":\"编辑器不存在: have\""),
            "b 无 provider 不受 a 的实例覆盖影响: {json_b}"
        );
    }
}

// ── 真实编辑器路径（V-01：旧门面 envelope == provider+core 纯形） ───────

#[cfg(all(feature = "ui-iced", feature = "code-editor"))]
mod plan807_real_editor {
    /// 真注册表：旧门面 `diff_snapshots_envelope(key_a,key_b)` 与
    /// provider 端口 + core 纯形逐字节一致；编辑器读取期间替换不影响已取
    /// 快照（COW 冻结）。
    #[test]
    fn old_facade_envelope_matches_provider_core_path() {
        use crate::ui::code_editor::{
            code_editor, code_editor_dispose, core, storage_key, CodeEditorConfig,
        };
        core::ensure_font_system_call();
        let _guard = core::REGISTRY_TEST_LOCK.lock().unwrap();
        let key_a = storage_key("p807-real-a");
        let key_b = storage_key("p807-real-b");
        code_editor_dispose(&key_a);
        code_editor_dispose(&key_b);
        let cfg = CodeEditorConfig::default();
        let core_a = code_editor(&key_a, &cfg);
        let core_b = code_editor(&key_b, &cfg);
        core_a.doc_replace("alpha\nbeta\ngamma\n");
        core_b.doc_replace("alpha\nbetaX\ngamma\ndelta\n");

        // 旧门面（UI adapter 路径）。
        let facade = crate::ui::code_editor::diff::envelope::diff_snapshots_envelope(
            "p807-real-a",
            "p807-real-b",
        );

        // provider 端口 + core 纯形（VM shim 同构路径）。
        let sa = crate::ui::code_editor::code_editor_with(&key_a, |c| c.doc_snapshot())
            .expect("snapshot a");
        let sb = crate::ui::code_editor::code_editor_with(&key_b, |c| c.doc_snapshot())
            .expect("snapshot b");
        let pure = auto_text_core::diff::envelope::diff_snapshots_envelope_snapshots(
            auto_text_core::diff::envelope::DiffSnapshotsPair::Both(sa.clone(), sb),
        );

        assert_eq!(facade, pure, "旧门面与 provider+core 纯形逐字节一致");
        assert!(facade.contains("\"err\":\"\""), "{facade}");
        assert!(facade.contains("\"adds\":2"), "{facade}");

        // COW：快照取得后重写编辑器，旧快照仍冻结（diff 基线不受影响）。
        core_a.doc_replace("completely different\n");
        let sa2 = crate::ui::code_editor::code_editor_with(&key_a, |c| c.doc_snapshot())
            .expect("snapshot a2");
        assert_eq!(sa2.to_string(), "completely different\n");
        assert_eq!(sa.to_string(), "alpha\nbeta\ngamma\n", "旧快照冻结");
    }
}

// ── 分页资源退出观测（V-03 旁证；crate 内永久主证见 file_backing 测试）──

#[cfg(feature = "code-editor")]
mod plan807_resource_release {
    use crate::ui::code_editor::core::rope::{PageConfig, Rope};

    /// Windows delete-pending 观测：分页 rope 最后外部引用释放 → store 同步
    /// 析构 → 基文件句柄关闭 → 同名重建成功（句柄未关时 create 被拒）。
    /// 旧 strong-clone 闭环下该重建失败（worker 持 store 持句柄永不放）。
    #[test]
    #[cfg(windows)]
    fn paged_base_handle_released_after_last_drop() {
        let p = std::env::temp_dir().join(format!("p807-release-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&p);
        std::fs::write(&p, "line\n".repeat(400)).unwrap();
        let (rope, store) = Rope::open_file_backed(
            &p,
            PageConfig {
                page_bytes: 512,
                cache_budget: 4096,
                prefetch_radius: 1,
            },
        )
        .expect("open");
        let exited = store.worker_exited_handle();
        let snap = rope.snapshot();

        drop(rope);
        drop(store);
        assert!(
            !exited.load(std::sync::atomic::Ordering::Acquire),
            "snapshot 存活期间 store/worker 存活"
        );
        assert_eq!(snap.line_count(), 401);

        drop(snap);
        assert!(
            std::fs::remove_file(&p).is_ok(),
            "remove 在句柄关闭后成功（FILE_SHARE_DELETE 下句柄未关也可删名——重建才是硬观测）"
        );
        // 硬观测：句柄全关前 delete-pending 使同名 create 被拒；全关后成功。
        let mut ok = false;
        for _ in 0..100 {
            if std::fs::File::create(&p).is_ok() {
                ok = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(ok, "同名重建成功——基文件句柄已随最后引用释放关闭");
        // worker 受控退出（有界轮询）。
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !exited.load(std::sync::atomic::Ordering::Acquire)
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(
            exited.load(std::sync::atomic::Ordering::Acquire),
            "预取 worker 受控退出"
        );
        let _ = std::fs::remove_file(&p);
    }
}
