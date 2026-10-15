//! PLAN-785: clipboard text native 2925/2926 host provider 注入——结构性
//! 剪断与装配不变量（in-crate 面；外部 consumer oracle 见
//! test/fixtures/modularity/clipboard-provider-consumer，V-01..05）。
//!
//! V-04/V-05 对应锚：
//! - core 剪断：vm/native.rs（shim 实现体）不再含 `ui::clipboard::` 直引
//!   （`ui::clipboard_native` files/image 族为计划非目标保留臂；iced
//!   renderer 侧直调同属未迁 UI 内部面，不在断言范围）。
//! - shim 缺席契约：无 provider → 原 feature-disabled 形 VMError（文本
//!   逐字；函数名即 2925/2926 诊断定位面）。
//! - UI 创建路径注入：VmBridge 构造即注册进程默认（arboard adapter）。
//! - merge 保 provider：merge_native_interface 经 Clone+merge 重包 Arc，
//!   实例 provider 不丢。
//! - 装配优先关系（UI 开列；no-ui 列由外部 fixture 承担）：高层入口
//!   （run_with_capture）的兼容默认装配不覆盖显式默认；空槽补缺后读侧
//!   恢复（内容不定，仅断言成功与单行——红线：不写真实剪贴板）。

mod plan785_provider {
    use std::sync::{Arc, Mutex};

    /// core 剪断（永久回归锚）：选定 shim 实现文件零 `ui::clipboard::`
    /// 直引（files 族 `ui::clipboard_native` 保留臂不匹配本串）；
    /// host_capabilities 零 `ui::` 引用（能力接口与装配层分离）。
    #[test]
    fn core_clipboard_shims_have_no_ui_clipboard_reference() {
        let native_rs = include_str!("../vm/native.rs");
        assert!(
            !native_rs.contains("ui::clipboard::"),
            "vm/native.rs 不得直引 ui::clipboard（PLAN-785 剪断；供值经 host_capabilities provider）"
        );
        let host_caps_rs = include_str!("../vm/host_capabilities.rs");
        assert!(
            !host_caps_rs.contains("ui::"),
            "vm/host_capabilities.rs 是 core 能力接口，不得引用 ui"
        );
    }

    /// shim 缺席/委托契约（直调 shim，无程序编译）：无 provider → 原
    /// feature-disabled 错误文本逐字保持；实例注入 fake → String/Bool
    /// lane 委托供值、set 收精确参数。
    /// F-785-R2-01：缺席断言锚 = **构造式**（`new_without_host_snapshot`
    /// 跳过进程默认快照）——共享 lib 测试进程的全局槽不可假定缺席（普通
    /// 程序测试经 run 族在独立 VM 线程上隐式 if-absent 装配）；**全局**
    /// 缺席时序合同由外部 fixture no-ui 形承担（v03：真实高层入口 Err
    /// 文本逐字）。本测试因此零全局槽操作，无需隔离锁。
    #[test]
    fn shim_absent_error_and_provider_delegation() {
        use crate::vm::engine::AutoVM;
        use crate::vm::host_capabilities::ClipboardTextProvider;
        use crate::vm::native::{shim_clipboard_set_text, shim_clipboard_text};
        use crate::vm::task::AutoTask;
        use crate::vm::virt_memory::VirtualFlash;

        // 缺席：构造式（不快照进程默认——无论全局槽当前被谁装配）。
        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
        assert!(
            vm.clipboard_text_provider().is_none(),
            "构造式缺席：跳过快照=无 provider（不得依赖全局槽时序）"
        );
        let mut task = AutoTask::new(0, 64, 0);
        let err =
            shim_clipboard_text(&mut task, &vm).expect_err("缺席须报原 feature-disabled 形错误");
        assert!(
            format!("{err:?}").contains("clipboard_text: the `ui-clipboard` feature is disabled"),
            "get 缺席错误文本逐字保持（got {err:?}）"
        );
        let err_set = shim_clipboard_set_text(&mut task, &vm).expect_err("set 缺席同形错误");
        assert!(
            format!("{err_set:?}")
                .contains("clipboard_set_text: the `ui-clipboard` feature is disabled"),
            "set 缺席错误文本逐字保持（got {err_set:?}）"
        );

        // 委托：实例注入 fake → get 走 String lane（add_string+rc_push），
        // set 走 Bool lane 且收到精确 UTF-8 参数。
        struct Fake {
            text: Option<String>,
            received: Mutex<Vec<String>>,
        }
        impl ClipboardTextProvider for Fake {
            fn text(&self) -> Option<String> {
                self.text.clone()
            }
            fn set_text(&self, text: &str) -> bool {
                self.received.lock().unwrap().push(text.to_string());
                true
            }
        }
        let fake = Arc::new(Fake {
            text: Some("委托值-β🎉".to_string()),
            received: Mutex::new(Vec::new()),
        });
        vm.set_clipboard_text_provider(fake.clone());

        let mut task = AutoTask::new(0, 256, 0);
        shim_clipboard_text(&mut task, &vm).expect("provider 在=成功");
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_string(nv), "String lane 出口");
        let idx = auto_val::decode_string(nv);
        let got = vm.get_string(idx).expect("字符串池可读");
        assert_eq!(got, "委托值-β🎉".as_bytes().to_vec());

        // set：压参数（String lane 入口形）后 pop 断言 Bool true + 精确收到。
        let arg_idx = vm.add_string("参数-甲🎉".as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, arg_idx);
        shim_clipboard_set_text(&mut task, &vm).expect("set 成功");
        assert!(auto_val::decode_bool(task.ram.pop_nv()), "Bool true 出口");
        assert_eq!(
            *fake.received.lock().unwrap(),
            vec!["参数-甲🎉".to_string()],
            "set 收到精确 UTF-8 参数"
        );
    }

    /// UI 组件创建路径（VmBridge）注入：bridge 构造后进程默认 clipboard
    /// provider 在位（arboard adapter）。F-785-R1-01：bridge 构造会
    /// if-absent **写槽**——须持共享隔离锁，与 clear→构造窗口的缺席
    /// 断言测试互斥（本地锁域不相交=无互斥）。
    #[test]
    fn vm_bridge_creation_installs_process_default() {
        let _g = serialized_default_slot();
        let src = concat!(
            "widget T785Probe {\n",
            "    view {\n",
            "        col {\n",
            "            text \"probe\"\n",
            "        }\n",
            "    }\n",
            "}\n",
        );
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|st| match st {
                crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
                _ => None,
            })
            .expect("decl");
        let widget = crate::aura::extract_widget_from_decl(&decl).expect("extract");
        let _bridge = crate::ui::vm_bridge::VmBridge::new(&widget).expect("bridge");

        assert!(
            crate::vm::host_capabilities::process_default_clipboard_text_provider().is_some(),
            "VmBridge 构造须注册进程默认 clipboard text provider（PLAN-785 UI 装配）"
        );
    }

    /// merge 保 provider：VM 实例注入 provider 后 merge 外部 NI，provider
    /// 不丢；binding_revision 按 merge 既有规则 bump（宿主能力面不受
    /// merge 触碰）。
    #[test]
    fn merge_native_interface_preserves_instance_provider() {
        use crate::vm::engine::AutoVM;
        use crate::vm::host_capabilities::ClipboardTextProvider;
        use crate::vm::native::NativeInterface;
        use crate::vm::virt_memory::VirtualFlash;

        struct Fixed(Option<&'static str>);
        impl ClipboardTextProvider for Fixed {
            fn text(&self) -> Option<String> {
                self.0.map(|s| s.to_string())
            }
            fn set_text(&self, _text: &str) -> bool {
                false
            }
        }

        let mut vm = AutoVM::new(VirtualFlash::new(0), 64);
        vm.set_clipboard_text_provider(Arc::new(Fixed(Some("keep-785"))));
        let rev_before = vm.native_interface.binding_revision();

        let mut extra = NativeInterface::new();
        extra.register_static(2090, |_, _| Ok(()));
        vm.merge_native_interface(&extra);

        assert!(
            vm.native_interface.binding_revision() > rev_before,
            "merge 须按既有规则 bump revision"
        );
        let p = vm
            .clipboard_text_provider()
            .expect("merge 后 provider 保留");
        assert_eq!(
            ClipboardTextProvider::text(&*p).as_deref(),
            Some("keep-785")
        );
    }

    /// F-785-R2-01 永久组合反例：**构造式缺席锚** × 普通高层入口并发。
    /// R2 原发（plan320 普通测试并行 → run_with_capture → 独立 VM 线程
    /// install → if-absent 写槽 → 毒化"clear→AutoVM::new 断言缺席"窗口，
    /// 14 轮实录）证明：缺席断言不得依赖共享进程的全局槽时序。本测试把
    /// 该混合场景钉为永久回归——线程 A 反复做构造式缺席断言
    /// （`new_without_host_snapshot`，零全局槽操作）；线程 B 与 R2 反例同
    /// 路径 hammer 普通高层入口（零锁配合）。若缺席锚退回全局时序实现
    /// （clear→new），A 将随 B 的隐式装配随机变红（R2 证据）；构造式下
    /// A 全轮通过、B 全轮成功。不触真实剪贴板。
    #[test]
    fn absence_anchor_survives_concurrent_implicit_installs() {
        use crate::vm::engine::AutoVM;
        use crate::vm::virt_memory::VirtualFlash;

        let a = std::thread::spawn(|| {
            for _ in 0..50 {
                let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
                assert!(
                    vm.clipboard_text_provider().is_none(),
                    "缺席锚不得依赖共享进程的全局槽时序（F-785-R2-01 反例形态）"
                );
            }
        });
        let b = std::thread::spawn(|| {
            for _ in 0..100 {
                // 普通高层入口：零锁配合——隔离责任在断言锚的构造方式。
                let _ = crate::run_with_capture("print(1)");
            }
        });
        a.join().expect("A 构造式缺席全轮通过");
        b.join().expect("B 高层入口全轮成功");
    }

    // ── 装配优先关系（UI 开列；默认测试构建=ui-iced on）────────────────
    //
    // 覆盖矩阵：高层入口 × {显式默认, 空槽} 供值；no-ui 列由外部 fixture
    //（default-features=false）承担；实例覆盖列由 fixture v01(b)/v04 与
    // 本文件 shim 直调测试承担。红线（计划 §5）：空槽补缺断言只走读侧
    //（真 arboard adapter 不写用户剪贴板）。

    fn serialized_default_slot() -> std::sync::MutexGuard<'static, ()> {
        // F-785-R1-01→R2-01：crate 级共享测试锁——持有者仅限**清空/替换
        // 进程默认槽**的测试（跨操作原子）；普通程序测试零配合豁免（其
        // 唯一槽写 if-absent 补缺对非缺席断言无害）。
        crate::vm::host_capabilities::test_default_slot_lock()
    }

    struct DefaultGuard(Option<Arc<dyn crate::vm::host_capabilities::ClipboardTextProvider>>);

    impl DefaultGuard {
        fn new() -> Self {
            Self(crate::vm::host_capabilities::process_default_clipboard_text_provider())
        }
    }

    impl Drop for DefaultGuard {
        fn drop(&mut self) {
            match self.0.take() {
                Some(p) => {
                    crate::vm::host_capabilities::set_process_default_clipboard_text_provider(p)
                }
                None => {
                    crate::vm::host_capabilities::clear_process_default_clipboard_text_provider()
                }
            }
        }
    }

    struct FakeClipboard(Option<&'static str>);

    impl crate::vm::host_capabilities::ClipboardTextProvider for FakeClipboard {
        fn text(&self) -> Option<String> {
            self.0.map(|s| s.to_string())
        }
        fn set_text(&self, _text: &str) -> bool {
            false
        }
    }

    /// 高层入口（run_with_capture → execute_autovm_with_deadline →
    /// install_ui_host_capabilities）优先关系：(a) 空槽=兼容默认补缺且
    /// 读侧恢复（成功+单行；内容 headless/桌面不定，不作内容断言）；
    /// (b) 显式默认不被覆盖、经真实 VM stdout 供值；(b2) 显式替换
    /// （A→B）经高层入口持续生效。
    #[test]
    fn high_level_compat_default_respects_explicit_choice() {
        use crate::vm::host_capabilities::{
            clear_process_default_clipboard_text_provider, process_default_clipboard_text_provider,
            set_process_default_clipboard_text_provider,
        };

        let _g = serialized_default_slot();
        let _guard = DefaultGuard::new();

        let program = "print(clipboard_text())\n";

        // (a) 空槽：兼容默认补缺（ui 构建下 run 管线装配 arboard adapter）；
        // 读侧恢复=程序**成功**（缺席形此处应为 VMError——补缺前后的可
        // 观察差即本断言；内容/行数不定，不作内容断言——真实剪贴板可为
        // 任意多行文本，2026-10-16 实测）。
        clear_process_default_clipboard_text_provider();
        let _ = crate::run_with_capture(program).expect("空槽补缺后读侧须成功");
        assert!(
            process_default_clipboard_text_provider().is_some(),
            "空槽时兼容默认须被补缺（ui 构建 run 管线装配）"
        );

        // (b) 显式默认经高层入口供值：兼容装配不得覆盖。
        set_process_default_clipboard_text_provider(Arc::new(FakeClipboard(Some("explicit-785"))));
        let (_, stdout) = crate::run_with_capture(program).expect("run explicit A");
        assert_eq!(
            stdout.lines().collect::<Vec<_>>(),
            vec!["explicit-785"],
            "显式默认须经高层入口真实供值（兼容装配不得覆盖；got {stdout:?}）"
        );

        // (b2) 显式替换（A→B）同样穿透。
        set_process_default_clipboard_text_provider(Arc::new(FakeClipboard(Some("replaced-785"))));
        let (_, stdout) = crate::run_with_capture(program).expect("run explicit B");
        assert_eq!(
            stdout.lines().collect::<Vec<_>>(),
            vec!["replaced-785"],
            "显式替换须经高层入口生效（got {stdout:?}）"
        );
    }
}
