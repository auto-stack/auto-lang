//! PLAN-804: dialog（2927/2928）+ dnd（2938）host provider 注入——结构性
//! 剪断与装配不变量（in-crate 面；外部 no-ui consumer oracle 见
//! test/fixtures/modularity/dialog-dnd-provider-consumer，V-01..03）。
//!
//! 锚点对应（计划 §6 V02/V03）：
//! - core 剪断：vm/native.rs（shim 实现体）不再含 rfd/raw-window-handle/
//!   dialog_parent/ui::native_dnd 直引（OS/COM 实现经 host_capabilities
//!   provider 装配；dnd 载荷 parser 留 ui 唯一）。
//! - 缺席契约：dialog 无 provider → 原 feature-disabled 形 VMError（文本
//!   逐字，**弹实参前 raise**——旧 off 臂不弹实参）；dnd 无 provider →
//!   弹实参后 false（G5 值降级，off 臂同形）。
//! - 委托契约：实例注入 fake → String/Bool lane 委托供值；raw filter/
//!   default_name/owned JSON 参数精确到达（core 不解析过滤语义）。
//! - 槽独立：dialog 在位不掩盖 dnd 缺席（反之亦然）。
//! - 装配/身份：注入不 bump binding_revision；Clone/merge 保 provider；
//!   槽锁重入安全（provider 回调再读/写槽不死锁——锁在调用前释放）。

mod plan804_dialog_dnd {
    use std::sync::{Arc, Mutex};

    /// core 剪断（永久回归锚）：shim 实现文件零 OS 对话框/拖出直引——
    /// `rfd::`、`raw_window_handle`、`dialog_parent`（Plan 428 父窗模块已
    /// 迁 host_adapters）、`ui::native_dnd::`（载荷 parser 留 ui 唯一，
    /// 针含尾随 `::`——注释提及不误伤，785/790 同款纪律）；host_capabilities
    /// 零 `ui::` 引用。
    #[test]
    fn core_shims_have_no_dialog_dnd_os_reference() {
        let native_rs = include_str!("../../../auto-lang-core/src/vm/native.rs");
        // 针含尾随 `::` 或模块声明形——注释提及不误伤（785/790 同款纪律）。
        for needle in [
            "rfd::",
            "raw_window_handle",
            "mod dialog_parent",
            "dialog_parent::",
            "ui::native_dnd::",
        ] {
            assert!(
                !native_rs.contains(needle),
                "vm/native.rs 不得含 {needle}（PLAN-804 剪断；供值经 host_capabilities provider）"
            );
        }
        let host_caps_rs = include_str!("../../../auto-lang-core/src/vm/host_capabilities.rs");
        assert!(
            !host_caps_rs.contains("ui::"),
            "vm/host_capabilities.rs 是 core 能力接口，不得引用 ui"
        );
    }

    /// 可记录 fake dialog provider：回放可配 open/save 结果，记录收到
    /// 的精确 raw 参数（含 Unicode）；可挂回调（槽重入探针）。
    struct FakeDialog {
        open_ret: String,
        save_ret: String,
        received_open: Mutex<Vec<String>>,
        received_save: Mutex<Vec<String>>,
        on_call: Option<Box<dyn Fn() + Send + Sync>>,
    }

    impl FakeDialog {
        fn new(open_ret: &str, save_ret: &str) -> Arc<Self> {
            Arc::new(Self {
                open_ret: open_ret.to_string(),
                save_ret: save_ret.to_string(),
                received_open: Mutex::new(Vec::new()),
                received_save: Mutex::new(Vec::new()),
                on_call: None,
            })
        }

        fn with_callback(open_ret: &str, cb: Box<dyn Fn() + Send + Sync>) -> Arc<Self> {
            Arc::new(Self {
                open_ret: open_ret.to_string(),
                save_ret: String::new(),
                received_open: Mutex::new(Vec::new()),
                received_save: Mutex::new(Vec::new()),
                on_call: Some(cb),
            })
        }
    }

    impl auto_lang_core::vm::host_capabilities::DialogHostProvider for FakeDialog {
        fn open(&self, filter: &str) -> String {
            self.received_open.lock().unwrap().push(filter.to_string());
            if let Some(cb) = &self.on_call {
                cb();
            }
            self.open_ret.clone()
        }
        fn save(&self, default_name: &str) -> String {
            self.received_save
                .lock()
                .unwrap()
                .push(default_name.to_string());
            if let Some(cb) = &self.on_call {
                cb();
            }
            self.save_ret.clone()
        }
    }

    /// 可记录 fake dnd provider：回放可配 bool，记录收到的精确 owned
    /// JSON（含 Unicode/裸反斜杠原文——core 不预解析）。
    struct FakeDnd {
        ret: bool,
        received: Mutex<Vec<String>>,
    }

    impl FakeDnd {
        fn new(ret: bool) -> Arc<Self> {
            Arc::new(Self {
                ret,
                received: Mutex::new(Vec::new()),
            })
        }

        fn received_snapshot(&self) -> Vec<String> {
            self.received.lock().unwrap().clone()
        }
    }

    impl auto_lang_core::vm::host_capabilities::DndHostProvider for FakeDnd {
        fn start(&self, payload_json: String) -> bool {
            self.received.lock().unwrap().push(payload_json);
            self.ret
        }
    }

    /// 缺席契约（直调 shim，无程序编译）：构造式 `new_without_host_snapshot`
    /// ——dialog 两 shim 报原 feature-disabled 文本逐字且**不弹实参**（实参
    /// 仍在栈顶可弹回）；dnd shim 消费实参后返 false（off 臂同形）。
    /// F-785-R2-01：缺席断言锚=构造式，零全局槽操作，无需隔离锁。
    #[test]
    fn shim_absence_contract_dialog_error_dnd_false() {
        use auto_lang_core::vm::engine::AutoVM;
        use auto_lang_core::vm::native::{shim_dialog_open, shim_dialog_save, shim_dnd_start};
        use auto_lang_core::vm::task::AutoTask;
        use auto_lang_core::vm::virt_memory::VirtualFlash;

        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
        assert!(
            vm.dialog_host_provider().is_none(),
            "构造式缺席：无 dialog provider"
        );
        assert!(
            vm.dnd_host_provider().is_none(),
            "构造式缺席：无 dnd provider"
        );

        // dialog_open：压实参 → Err（文本逐字）→ 实参未被消费（仍可弹回）。
        let mut task = AutoTask::new(0, 128, 0);
        let arg_idx = vm.add_string("过滤-甲🎉.at,.au".as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, arg_idx);
        let err = shim_dialog_open(&mut task, &vm)
            .expect_err("dialog_open 缺席须报原 feature-disabled 形错误");
        assert!(
            format!("{err:?}").contains("dialog_open: the `ui-dialog` feature is disabled"),
            "open 缺席错误文本逐字保持（got {err:?}）"
        );
        let left = task.ram.pop_nv();
        assert!(
            auto_val_or(left, arg_idx as u64),
            "缺席臂不弹实参（旧 off 臂栈面；弹回值应为原压入串）"
        );

        // dialog_save：同形。
        let mut task = AutoTask::new(0, 128, 0);
        let name_idx = vm.add_string("默认名-乙🎉.at".as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, name_idx);
        let err_save = shim_dialog_save(&mut task, &vm).expect_err("dialog_save 缺席须报同形错误");
        assert!(
            format!("{err_save:?}").contains("dialog_save: the `ui-dialog` feature is disabled"),
            "save 缺席错误文本逐字保持（got {err_save:?}）"
        );
        assert!(
            auto_val_or(task.ram.pop_nv(), name_idx as u64),
            "save 缺席臂不弹实参"
        );

        // dnd_start：压 JSON 实参 → 消费后 false（off 臂同形——弹参+false）。
        let mut task = AutoTask::new(0, 128, 0);
        let json_idx = vm.add_string("{\"text\":\"拖出-丙🎉\"}".as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, json_idx);
        shim_dnd_start(&mut task, &vm).expect("dnd 缺席=值降级非 raise");
        assert!(
            !auto_val_decode_bool(task.ram.pop_nv()),
            "dnd 缺席返 false（G5）"
        );
    }

    /// 委托 + 槽独立契约：dialog 在位 → String lane 精确 raw 参数/取消空串；
    /// dnd 缺席同批仍 false；反向（仅 dnd 在位）dialog 仍报缺席错误。
    #[test]
    fn shim_delegation_and_slot_independence() {
        use auto_lang_core::vm::engine::AutoVM;
        use auto_lang_core::vm::native::{shim_dialog_open, shim_dialog_save, shim_dnd_start};
        use auto_lang_core::vm::task::AutoTask;
        use auto_lang_core::vm::virt_memory::VirtualFlash;

        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 256);
        let dialog = FakeDialog::new("C:\\plan804\\选中文件-🎉.at", "C:\\plan804\\保存目标-β.md");
        vm.set_dialog_host_provider(dialog.clone());

        // open：raw filter 精确到达（core 不做分割/去点——单一 parser 在
        // host_adapters）；返回走 String lane。
        let mut task = AutoTask::new(0, 256, 0);
        let f_idx = vm.add_string(" .at, .au ;md\tpng ".as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, f_idx);
        shim_dialog_open(&mut task, &vm).expect("provider 在位=成功");
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_string(nv), "String lane 出口");
        let got = vm
            .get_string(auto_val::decode_string(nv))
            .expect("字符串池可读");
        assert_eq!(got, "C:\\plan804\\选中文件-🎉.at".as_bytes().to_vec());
        assert_eq!(
            *dialog.received_open.lock().unwrap(),
            vec![" .at, .au ;md\tpng ".to_string()],
            "open 收到精确 raw filter（未预解析）"
        );

        // save：default_name 精确到达。
        let mut task = AutoTask::new(0, 256, 0);
        let n_idx = vm.add_string("草稿-🎉.txt".as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, n_idx);
        shim_dialog_save(&mut task, &vm).expect("save 成功");
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_string(nv), "save String lane 出口");
        assert_eq!(
            *dialog.received_save.lock().unwrap(),
            vec!["草稿-🎉.txt".to_string()],
            "save 收到精确 raw default_name"
        );

        // 槽独立：dialog 在位、dnd 缺席 → dnd_start 仍 false（一槽在位
        // 不掩盖另一槽缺席）。
        let mut task = AutoTask::new(0, 128, 0);
        let j_idx = vm.add_string(b"{}".to_vec());
        vm.rc_push_str_idx(&mut task, j_idx);
        shim_dnd_start(&mut task, &vm).expect("dnd 缺席=值降级");
        assert!(!auto_val_decode_bool(task.ram.pop_nv()), "dnd 缺席 false");

        // 反向：dnd 在位（本 VM）、dialog 覆盖为缺席不可行（set 无 None
        // 面）——用第二 VM 验证仅 dnd 在位时 dialog 报缺席错误。
        let vm2 = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 256);
        let dnd = FakeDnd::new(true);
        vm2.set_dnd_host_provider(dnd.clone());
        let mut task = AutoTask::new(0, 128, 0);
        let j_idx = vm2.add_string("{\"files\":[\"D:\\x\\y.md\"]}".as_bytes().to_vec());
        vm2.rc_push_str_idx(&mut task, j_idx);
        shim_dnd_start(&mut task, &vm2).expect("dnd 在位=成功");
        assert!(
            auto_val_decode_bool(task.ram.pop_nv()),
            "dnd 在位回放 true（会话语义=provider 契约）"
        );
        assert_eq!(
            dnd.received_snapshot(),
            vec!["{\"files\":[\"D:\\x\\y.md\"]}".to_string()],
            "dnd 收到精确 owned JSON 原文（含裸反斜杠原文，core 不修复）"
        );
        let mut task = AutoTask::new(0, 128, 0);
        let f_idx = vm2.add_string("at".as_bytes().to_vec());
        vm2.rc_push_str_idx(&mut task, f_idx);
        let err = shim_dialog_open(&mut task, &vm2)
            .expect_err("仅 dnd 在位：dialog 仍缺席报错（槽独立）");
        assert!(
            format!("{err:?}").contains("dialog_open: the `ui-dialog` feature is disabled"),
            "槽独立：dialog 缺席错误不因 dnd 在位而改变"
        );
    }

    /// 取消/不可用 = 空串（值语义非 raise）：provider 回 "" → String lane
    /// 空串出口。
    #[test]
    fn shim_cancel_returns_empty_string() {
        use auto_lang_core::vm::engine::AutoVM;
        use auto_lang_core::vm::native::{shim_dialog_open, shim_dialog_save};
        use auto_lang_core::vm::task::AutoTask;
        use auto_lang_core::vm::virt_memory::VirtualFlash;

        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 256);
        vm.set_dialog_host_provider(FakeDialog::new("", ""));
        for (shim_call, arg) in [
            (
                false, // open
                "at,md",
            ),
            (
                true, // save
                "名字",
            ),
        ] {
            let mut task = AutoTask::new(0, 128, 0);
            let idx = vm.add_string(arg.as_bytes().to_vec());
            vm.rc_push_str_idx(&mut task, idx);
            if shim_call {
                shim_dialog_save(&mut task, &vm).expect("save 成功（取消=空串非错误）");
            } else {
                shim_dialog_open(&mut task, &vm).expect("open 成功（取消=空串非错误）");
            }
            let nv = task.ram.pop_nv();
            assert!(auto_val::is_string(nv), "取消仍走 String lane");
            assert!(
                vm.get_string(auto_val::decode_string(nv))
                    .map(|b| b.is_empty())
                    .unwrap_or(false),
                "取消/不可用=空串"
            );
        }
    }

    /// 槽锁重入（V03）：NI accessor 先释放槽锁再调 provider——fake 回调内
    /// 再读/写同一 NI 槽（set_dialog_host_provider 为写锁）不死锁/不 panic。
    ///（provider 持 NI Arc 回调；shim 调用点无槽锁——重入窗口关闭。）
    #[test]
    fn ni_slot_lock_released_before_provider_call() {
        use auto_lang_core::vm::engine::AutoVM;
        use auto_lang_core::vm::native::shim_dialog_open;
        use auto_lang_core::vm::task::AutoTask;
        use auto_lang_core::vm::virt_memory::VirtualFlash;

        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 256);
        let ni = vm.native_interface.clone();
        let fake = FakeDialog::with_callback(
            "C:\\reentry\\ok.md",
            Box::new(move || {
                // 读锁 + 写锁各一次：任一在位持锁都会死锁（测试无超时即
                // 挂死暴露）或被 RwLock 语义拒绝。
                let _still = ni.dialog_host_provider();
                ni.set_dnd_host_provider(FakeDnd::new(true));
            }),
        );
        vm.set_dialog_host_provider(fake);

        let mut task = AutoTask::new(0, 128, 0);
        let idx = vm.add_string("at".as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, idx);
        shim_dialog_open(&mut task, &vm).expect("重入回调不死锁");
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_string(nv), "重入后仍正常供值");
        assert!(
            vm.dnd_host_provider().is_some(),
            "回调内写 dnd 槽真实生效（锁确已释放）"
        );
    }

    /// 装配/身份：注入不 bump binding_revision；Clone/merge 保两槽
    /// provider（merge 目标实例槽不丢）。
    #[test]
    fn assembly_identity_and_merge_preserve_providers() {
        use auto_lang_core::vm::engine::AutoVM;
        use auto_lang_core::vm::native::NativeInterface;
        use auto_lang_core::vm::virt_memory::VirtualFlash;

        let ni = NativeInterface::production();
        let rev0 = ni.binding_revision();
        ni.set_dialog_host_provider(FakeDialog::new("a", "b"));
        ni.set_dnd_host_provider(FakeDnd::new(true));
        assert_eq!(
            ni.binding_revision(),
            rev0,
            "provider 注入不得撤销绑定身份/失效 proof"
        );

        // Clone 保留（merge_native_interface 经 Clone+merge 路径）。
        let cloned = ni.clone();
        assert!(
            cloned.dialog_host_provider().is_some(),
            "Clone 保 dialog 槽"
        );
        assert!(cloned.dnd_host_provider().is_some(), "Clone 保 dnd 槽");

        // VM 级 merge：目标实例两槽不丢（merge 只动 shims/绑定）。
        let mut vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 256);
        vm.set_dialog_host_provider(FakeDialog::new("x", "y"));
        vm.set_dnd_host_provider(FakeDnd::new(false));
        let mut carrier = NativeInterface::new();
        carrier.register_static(1, plan804_noop_shim);
        vm.merge_native_interface(&carrier);
        assert!(
            vm.dialog_host_provider().is_some(),
            "merge 后 dialog 槽保留"
        );
        assert!(vm.dnd_host_provider().is_some(), "merge 后 dnd 槽保留");
    }

    /// merge 载体用 noop shim（register_static 收 fn 指针）。
    fn plan804_noop_shim(
        _task: &mut auto_lang_core::vm::task::AutoTask,
        _vm: &auto_lang_core::vm::engine::AutoVM,
    ) -> Result<(), auto_lang_core::vm::engine::VMError> {
        Ok(())
    }

    // —— 顶部缺省 helper：两个 X-舱断言需要的小工具 ——

    /// 弹回值是否为压入的同一字符串索引（比较 tag+idx）。
    fn auto_val_or(nv: u64, pushed_idx: u64) -> bool {
        auto_val::is_string(nv) && auto_val::decode_string(nv) as u64 == pushed_idx
    }

    fn auto_val_decode_bool(nv: u64) -> bool {
        auto_val::decode_bool(nv)
    }
}
