//! PLAN-779: frame native 9918/9919 host provider 注入——结构性剪断与
//! 装配不变量（in-crate 面；外部 consumer oracle 见
//! test/fixtures/modularity/native-provider-consumer，V-01..03）。
//!
//! V-04 对应锚：
//! - core 剪断：vm/native.rs（shim 实现体）不再含 `ui::frame_bench` 直引
//!   （a2r_std.rs / handler 发射为计划 §5 明示保留臂，不在本断言面）。
//! - UI 创建路径注入：VmBridge 构造即注册进程默认（frame_bench adapter）。
//! - merge 保 provider：merge_native_interface 经 Clone+merge 重包 Arc，
//!   实例 provider 不丢（shim/contract 合并不触碰宿主能力装配）。
//! - F779-R2-01 装配优先关系（UI 开列；no-ui 列由外部 fixture 承担）：
//!   高层入口（run_with_capture）的兼容默认装配不覆盖显式默认；空槽补缺。

mod plan779_provider {
    use crate::vm::host_capabilities::FrameProvider;
    use std::sync::Arc;

    /// core 剪断（永久回归锚）：选定 frame shim 实现文件零 `ui::frame_bench`
    /// 直引；host_capabilities 零 `ui::` 引用（能力接口与装配层分离）。
    #[test]
    fn core_frame_shims_have_no_ui_frame_bench_reference() {
        let native_rs = include_str!("../vm/native.rs");
        assert!(
            !native_rs.contains("ui::frame_bench"),
            "vm/native.rs 不得直引 ui::frame_bench（PLAN-779 剪断；供值经 host_capabilities provider）"
        );
        let host_caps_rs = include_str!("../vm/host_capabilities.rs");
        assert!(
            !host_caps_rs.contains("ui::"),
            "vm/host_capabilities.rs 是 core 能力接口，不得引用 ui"
        );
    }

    /// UI 组件创建路径（VmBridge）注入：bridge 构造后进程默认 provider
    /// 在位（frame_bench adapter）。nextest 每测一进程——本测内 bridge 是
    /// 唯一可能的注册者（裸 cargo test 并行下亦为"UI 路径必注册"的合取断言）。
    #[test]
    fn vm_bridge_creation_installs_process_default() {
        let src = concat!(
            "widget T779Probe {\n",
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
            crate::vm::host_capabilities::process_default_frame_provider().is_some(),
            "VmBridge 构造须注册进程默认 frame provider（PLAN-779 UI 装配）"
        );
    }

    /// merge 保 provider：VM 实例注入 provider 后 merge 外部 NI（FFI 桥
    /// 形态），provider 不丢、读数不变；binding_revision 按 merge 既有
    /// 规则 bump（shim 面失效照旧，宿主能力面不受 merge 触碰）。
    #[test]
    fn merge_native_interface_preserves_instance_provider() {
        use crate::vm::engine::AutoVM;
        use crate::vm::host_capabilities::FrameProvider;
        use crate::vm::native::NativeInterface;
        use crate::vm::virt_memory::VirtualFlash;
        use std::sync::Arc;

        struct Fixed {
            begin: i64,
            present: i64,
        }
        impl FrameProvider for Fixed {
            fn begin_ms(&self) -> i64 {
                self.begin
            }
            fn present_ms(&self) -> i64 {
                self.present
            }
        }

        let mut vm = AutoVM::new(VirtualFlash::new(0), 64);
        vm.set_frame_provider(Arc::new(Fixed {
            begin: 123,
            present: 456,
        }));
        let rev_before = vm.native_interface.binding_revision();

        let mut extra = NativeInterface::new();
        extra.register_static(2090, |_, _| Ok(()));
        vm.merge_native_interface(&extra);

        assert!(
            vm.native_interface.binding_revision() > rev_before,
            "merge 须按既有规则 bump revision"
        );
        let p = vm.frame_provider().expect("merge 后 provider 保留");
        assert_eq!(FrameProvider::begin_ms(&*p), 123);
        assert_eq!(FrameProvider::present_ms(&*p), 456);
    }

    // ── F779-R2-01: 装配优先关系（UI 开列；默认测试构建=ui-iced on）────
    //
    // 覆盖矩阵（计划 §9 R2-repair）：高层入口 × {显式默认, 空槽} 供值；
    // 门开真值列由 plan716 frame_program_track_gate_on 承担；no-ui 列由
    // 外部 fixture（default-features=false）承担；实例覆盖列由 fixture
    // v01(b) 低层双活与本文件 merge 测试承担。

    fn serialized_default_slot() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        let m = LOCK.get_or_init(|| std::sync::Mutex::new(()));
        m.lock().unwrap_or_else(|e| e.into_inner())
    }

    struct DefaultGuard(Option<Arc<dyn FrameProvider>>);

    impl DefaultGuard {
        fn new() -> Self {
            Self(crate::vm::host_capabilities::process_default_frame_provider())
        }
    }

    impl Drop for DefaultGuard {
        fn drop(&mut self) {
            match self.0.take() {
                Some(p) => crate::vm::host_capabilities::set_process_default_frame_provider(p),
                None => crate::vm::host_capabilities::clear_process_default_frame_provider(),
            }
        }
    }

    struct FakeFrame {
        begin: i64,
        present: i64,
    }

    impl FrameProvider for FakeFrame {
        fn begin_ms(&self) -> i64 {
            self.begin
        }
        fn present_ms(&self) -> i64 {
            self.present
        }
    }

    /// 高层入口（run_with_capture → execute_autovm_with_deadline →
    /// install_ui_host_capabilities）优先关系：(a) 空槽=兼容默认补缺且
    /// 门关读 0；(b) 显式默认不被覆盖、经真实 VM stdout 供值；(b2) 显式
    /// 替换（A→B）经高层入口持续生效。
    #[test]
    fn high_level_compat_default_respects_explicit_choice() {
        use crate::vm::host_capabilities::{
            clear_process_default_frame_provider, process_default_frame_provider,
            set_process_default_frame_provider,
        };
        let _g = serialized_default_slot();
        let _guard = DefaultGuard::new();

        let program = "print(frame.begin_ms())\nprint(frame.present_ms())\n";

        // (a) 空槽：兼容默认补缺（ui 构建下 run 管线装配 adapter）；门关
        // 无泵 → 0（与缺席语义不可区分的合法形——"补缺发生"由 is_some 断言）。
        clear_process_default_frame_provider();
        let (_, stdout) = crate::run_with_capture(program).expect("run absent");
        assert_eq!(stdout.lines().collect::<Vec<_>>(), vec!["0", "0"]);
        assert!(
            process_default_frame_provider().is_some(),
            "空槽时兼容默认须被补缺（ui 构建 run 管线装配）"
        );

        // (b) 显式默认经高层入口供值：兼容装配不得覆盖（R2 反例=0/0）。
        set_process_default_frame_provider(Arc::new(FakeFrame {
            begin: 17,
            present: 31,
        }));
        let (_, stdout) = crate::run_with_capture(program).expect("run explicit A");
        assert_eq!(
            stdout.lines().collect::<Vec<_>>(),
            vec!["17", "31"],
            "显式默认须经高层入口真实供值（兼容装配不得覆盖；got {stdout:?}）"
        );

        // (b2) 显式替换（A→B）同样穿透。
        set_process_default_frame_provider(Arc::new(FakeFrame {
            begin: 43,
            present: 59,
        }));
        let (_, stdout) = crate::run_with_capture(program).expect("run explicit B");
        assert_eq!(
            stdout.lines().collect::<Vec<_>>(),
            vec!["43", "59"],
            "显式替换须经高层入口生效（got {stdout:?}）"
        );
    }
}
