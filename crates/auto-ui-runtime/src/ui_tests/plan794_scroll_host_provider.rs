//! PLAN-794: scroll controller/子锚点 natives（9900..9905 + 9922/9923）
//! host provider 注入——结构性剪断与装配不变量（in-crate 面；外部 no-ui
//! consumer oracle 见 test/fixtures/modularity/scroll-provider-consumer，
//! V-01/V-02 缺席与优先矩阵；真实 UI 实机证据见 781 既有探针与 V04）。
//!
//! 锚点对应（计划 §6/§7）：
//! - core 剪断：scroll 八 shim 段不再含 `ui::` 直引（code_editor 滚动族
//!   9911..9913 属非目标仍直引——锚按节区段圈定，不误伤）；host_capabilities
//!   零 `ui::` 引用（能力接口与装配层分离，790 同款纪律）。
//! - 降级契约（T-01 冻结矩阵，= 迁移前 no-ui 臂逐字面值）：缺席 =
//!   Int 0 / Bool false / Float 0.0 / "error:unsupported" / "unknown"
//!   （值语义非 raise），且各函数仍消费声明 arity 实参保持栈纪律。
//! - 委托契约：真实 UI adapter（ScrollControllerProvider）经 provider 面
//!   委托既有 controller 注册表——intent 落队、快照投影、锚点状态机原语
//!   （781 全语义在 controller.rs 既有测试，此处证 adapter 接线不漂移）。
//! - 装配/身份：注入不 bump binding_revision；Clone/merge 保 provider；
//!   ID 钉扎（9900..9905/9922/9923 与 canonical 名双射）。
//! - ownership：重复调用后任务栈清空 + heap/string 池引用回到基线
//!  （无悬空/增长——空句柄早退与缺席臂同样配平）。

mod plan794_scroll {
    use std::sync::Arc;

    use auto_lang_core::vm::engine::AutoVM;
    use auto_lang_core::vm::host_capabilities::{
        ScrollAnchorStateDto, ScrollAxisDto, ScrollHostProvider, ScrollIntentDto, ScrollSnapshotDto,
    };
    use auto_lang_core::vm::task::AutoTask;
    use auto_lang_core::vm::virt_memory::VirtualFlash;

    /// core 剪断（永久回归锚）：scroll 八 shim 节区段（PLAN-656 节头到
    /// Plan 418 dialogs 节头之间）零 `ui::` 直引——code_editor 滚动族
    /// （9911..9913，非目标）的 `ui::scroll::` 引用在区段外不误伤；
    /// host_capabilities 零 `ui::` 引用（能力接口与装配层分离）。
    #[test]
    fn core_scroll_shims_have_no_ui_reference() {
        let native_rs = include_str!("../../../auto-lang-core/src/vm/native.rs");
        let start = native_rs
            .find("// PLAN-656 T-03: scroll-pane controller natives")
            .expect("scroll natives 节头存在");
        let end = native_rs
            .find("// ── Plan 418: native file dialogs")
            .expect("dialogs 节头存在（scroll 节区段终点）");
        assert!(start < end, "scroll 节区段边界有序");
        let section = &native_rs[start..end];
        assert!(
            !section.contains("ui::"),
            "scroll 八 shim 节区段不得直引 ui::（PLAN-794 剪断；供值经 \
             host_capabilities::ScrollHostProvider 装配）"
        );
        let host_caps_rs = include_str!("../../../auto-lang-core/src/vm/host_capabilities.rs");
        assert!(
            !host_caps_rs.contains("ui::"),
            "vm/host_capabilities.rs 是 core 能力接口，不得引用 ui"
        );
    }

    /// ID 钉扎 + canonical 名双射（注入不改绑定身份的前提锚）。
    #[test]
    fn native_ids_and_names_pinned() {
        use auto_lang_core::vm::native::NativeInterface;
        let ni = NativeInterface::production();
        assert_eq!(ni.resolve("auto.scroll.controller"), Some(9900));
        assert_eq!(ni.resolve("auto.scroll.to_start"), Some(9901));
        assert_eq!(ni.resolve("auto.scroll.to_end"), Some(9902));
        assert_eq!(ni.resolve("auto.scroll.by"), Some(9903));
        assert_eq!(ni.resolve("auto.scroll.to"), Some(9904));
        assert_eq!(ni.resolve("auto.scroll.state"), Some(9905));
        assert_eq!(ni.resolve("auto.scroll.to_child"), Some(9922));
        assert_eq!(ni.resolve("auto.scroll.anchor_state"), Some(9923));
    }

    // ── 可区分哨兵 fake（记录 intent 参数 + 快照/锚点回供特异值） ─────────

    struct FakeScroll {
        handle_seq: std::sync::Mutex<u32>,
        intents: std::sync::Mutex<Vec<(String, ScrollIntentDto)>>,
        snapshot: std::sync::Mutex<ScrollSnapshotDto>,
        anchor_reply:
            std::sync::Mutex<std::collections::HashMap<(String, String), Result<(), &'static str>>>,
        anchor_states:
            std::sync::Mutex<std::collections::HashMap<(String, String), ScrollAnchorStateDto>>,
    }

    impl FakeScroll {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                handle_seq: std::sync::Mutex::new(0),
                intents: std::sync::Mutex::new(Vec::new()),
                snapshot: std::sync::Mutex::new(ScrollSnapshotDto {
                    offset_x: 11.5,
                    offset_y: 240.0,
                    viewport_w: 300.0,
                    viewport_h: 200.0,
                    content_w: 500.0,
                    content_h: 1000.0,
                    progress_x: 0.25,
                    progress_y: 0.3,
                }),
                anchor_reply: std::sync::Mutex::new(std::collections::HashMap::new()),
                anchor_states: std::sync::Mutex::new(std::collections::HashMap::new()),
            })
        }

        fn intents_snapshot(&self) -> Vec<(String, ScrollIntentDto)> {
            self.intents.lock().unwrap().clone()
        }
    }

    impl ScrollHostProvider for FakeScroll {
        fn next_controller_handle(&self) -> String {
            let mut n = self.handle_seq.lock().unwrap();
            *n += 1;
            format!("p794-fx-句柄-{n}-🎉")
        }
        fn enqueue_intent(&self, handle: &str, intent: ScrollIntentDto) {
            self.intents
                .lock()
                .unwrap()
                .push((handle.to_string(), intent));
        }
        fn controller_snapshot(&self, _handle: &str) -> ScrollSnapshotDto {
            *self.snapshot.lock().unwrap()
        }
        fn enqueue_anchor_request(&self, handle: &str, key: &str) -> Result<(), &'static str> {
            let replies = self.anchor_reply.lock().unwrap();
            match replies.get(&(handle.to_string(), key.to_string())) {
                Some(r) => r.clone(),
                None => Ok(()),
            }
        }
        fn anchor_request_state(&self, handle: &str, key: &str) -> Option<ScrollAnchorStateDto> {
            self.anchor_states
                .lock()
                .unwrap()
                .get(&(handle.to_string(), key.to_string()))
                .cloned()
        }
    }

    /// 压栈字符串实参（shim 弹参序列的镜像——测试侧按压序模拟调用约定）。
    fn push_str(vm: &AutoVM, task: &mut AutoTask, s: &str) {
        let idx = vm.add_string(s.as_bytes().to_vec());
        vm.rc_push_str_idx(task, idx);
    }

    fn push_f64(task: &mut AutoTask, v: f64) {
        task.ram.push_f64(v);
    }

    /// 弹出 shim 压入的字符串返回值（池份额按内容释放语义，无需显式回退）。
    fn pop_string(vm: &AutoVM, task: &mut AutoTask) -> String {
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_string(nv));
        let idx = auto_val::decode_string(nv);
        vm.get_string(idx)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default()
    }

    /// record 具名字段读出（GenericInstanceData：field_names 定位 + Float 值）。
    fn field_f64(
        inst: &auto_lang_core::vm::generic_registry::GenericInstanceData,
        name: &str,
    ) -> Option<f64> {
        let i = inst.field_names.iter().position(|n| n == name)?;
        match inst.get_field(i) {
            Some(auto_val::Value::Float(v)) => Some(*v),
            _ => None,
        }
    }

    /// 降级契约（构造式缺席——`new_without_host_snapshot` 不快照进程默认，
    /// F-785-R2-01 纪律：缺席断言锚必须构造式成立）：八函数缺席臂逐字面值
    /// + 栈纪律（各函数消费声明 arity 实参后压单个返回值，任务栈净清）。
    #[test]
    fn absent_capability_is_frozen_degradation() {
        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
        assert!(vm.scroll_host_provider().is_none());
        let mut task = AutoTask::new(0, 256, 0);
        let sp0 = task.ram.sp;

        // 9900：缺席 = Int 0（非空串）。
        auto_lang_core::vm::native::shim_scroll_controller(&mut task, &vm).unwrap();
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_i32(nv) && auto_val::decode_i32(nv) == 0);

        // 9901/9902：缺席 = false（弹 handle[+axis]）。
        task.pending_native_arg_count = 2;
        push_str(&vm, &mut task, "@scrollctl:absent");
        push_str(&vm, &mut task, "x");
        auto_lang_core::vm::native::shim_scroll_to_start(&mut task, &vm).unwrap();
        assert!(!auto_val::decode_bool(task.ram.pop_nv()));
        task.pending_native_arg_count = 1;
        push_str(&vm, &mut task, "@scrollctl:absent");
        auto_lang_core::vm::native::shim_scroll_to_end(&mut task, &vm).unwrap();
        assert!(!auto_val::decode_bool(task.ram.pop_nv()));

        // 9903：缺席 = false（delta/handle/axis 三弹——空句柄+轴组合同样配平）。
        task.pending_native_arg_count = 3;
        push_str(&vm, &mut task, "");
        push_str(&vm, &mut task, "x");
        push_f64(&mut task, 42.5);
        auto_lang_core::vm::native::shim_scroll_by(&mut task, &vm).unwrap();
        assert!(!auto_val::decode_bool(task.ram.pop_nv()));

        // 9904：缺席 = false（b/a/handle 三弹——轴形与坐标形同弹数）。
        push_str(&vm, &mut task, "@scrollctl:absent");
        push_f64(&mut task, 7.0);
        push_f64(&mut task, 8.0);
        auto_lang_core::vm::native::shim_scroll_to(&mut task, &vm).unwrap();
        assert!(!auto_val::decode_bool(task.ram.pop_nv()));

        // 9905：缺席 = Float 0.0（非 record）。
        push_str(&vm, &mut task, "@scrollctl:absent");
        auto_lang_core::vm::native::shim_scroll_state(&mut task, &vm).unwrap();
        assert_eq!(task.ram.pop_f64(), 0.0);

        // 9922：缺席 = "error:unsupported"。
        push_str(&vm, &mut task, "@scrollctl:absent");
        push_str(&vm, &mut task, "键");
        auto_lang_core::vm::native::shim_scroll_to_child(&mut task, &vm).unwrap();
        assert_eq!(pop_string(&vm, &mut task), "error:unsupported");

        // 9923：缺席 = "unknown"。
        push_str(&vm, &mut task, "@scrollctl:absent");
        push_str(&vm, &mut task, "键");
        auto_lang_core::vm::native::shim_scroll_anchor_state(&mut task, &vm).unwrap();
        assert_eq!(pop_string(&vm, &mut task), "unknown");

        // 栈纪律：全部返回值弹出后任务栈回到基线（缺席臂不留残余）。
        assert_eq!(task.ram.sp, sp0, "缺席臂弹实参压返回后任务栈净清");
    }

    /// 委托契约（直调 shim + fake provider）：intent 参数精确到达（句柄/
    /// 轴/offset/delta、to 坐标形 X 先 Y 后双 intent）；快照 DTO → record
    /// 八具名字段；锚点回执/状态编组；返回 Bool true。
    #[test]
    fn fake_provider_receives_exact_intents_and_snapshot() {
        use auto_lang_core::vm::generic_registry::GenericInstanceData;
        use auto_lang_core::vm::native::shim_scroll_anchor_state;
        use auto_lang_core::vm::native::shim_scroll_by;
        use auto_lang_core::vm::native::shim_scroll_state;
        use auto_lang_core::vm::native::shim_scroll_to;
        use auto_lang_core::vm::native::shim_scroll_to_child;
        use auto_lang_core::vm::native::shim_scroll_to_end;
        use auto_lang_core::vm::native::shim_scroll_to_start;

        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
        let fake = FakeScroll::new();
        vm.set_scroll_host_provider(fake.clone());
        let mut task = AutoTask::new(0, 512, 0);
        let sp0 = task.ram.sp;

        // 9901 轴形："x" → X；未知轴词归 Y。
        task.pending_native_arg_count = 2;
        push_str(&vm, &mut task, "@scrollctl:甲");
        push_str(&vm, &mut task, "x");
        shim_scroll_to_start(&mut task, &vm).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()));
        task.pending_native_arg_count = 2;
        push_str(&vm, &mut task, "@scrollctl:甲");
        push_str(&vm, &mut task, "nonsense");
        shim_scroll_to_end(&mut task, &vm).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()));

        // 9903 双形：简写（handle,delta）与显式轴（handle,axis,delta）。
        task.pending_native_arg_count = 2;
        push_str(&vm, &mut task, "@scrollctl:甲");
        push_f64(&mut task, 42.5);
        shim_scroll_by(&mut task, &vm).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()));
        task.pending_native_arg_count = 3;
        push_str(&vm, &mut task, "@scrollctl:乙");
        push_str(&vm, &mut task, "x");
        push_f64(&mut task, -7.0);
        shim_scroll_by(&mut task, &vm).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()));

        // 9904 坐标形：X 先 Y 后双 intent；轴形单 intent。
        push_str(&vm, &mut task, "@scrollctl:丙");
        push_f64(&mut task, 10.0);
        push_f64(&mut task, 20.0);
        shim_scroll_to(&mut task, &vm).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()));
        push_str(&vm, &mut task, "@scrollctl:丙");
        push_str(&vm, &mut task, "x");
        push_f64(&mut task, 99.0);
        shim_scroll_to(&mut task, &vm).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()));

        // 空句柄：跳过入队仍 true（在位语义），栈照配平。
        task.pending_native_arg_count = 2;
        push_str(&vm, &mut task, "");
        push_str(&vm, &mut task, "x");
        shim_scroll_to_start(&mut task, &vm).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()));

        assert_eq!(
            fake.intents_snapshot(),
            vec![
                (
                    "@scrollctl:甲".to_string(),
                    ScrollIntentDto::ToStart {
                        axis: ScrollAxisDto::X
                    }
                ),
                (
                    "@scrollctl:甲".to_string(),
                    ScrollIntentDto::ToEnd {
                        axis: ScrollAxisDto::Y
                    }
                ),
                (
                    "@scrollctl:甲".to_string(),
                    ScrollIntentDto::ScrollBy {
                        axis: ScrollAxisDto::Y,
                        delta: 42.5
                    }
                ),
                (
                    "@scrollctl:乙".to_string(),
                    ScrollIntentDto::ScrollBy {
                        axis: ScrollAxisDto::X,
                        delta: -7.0
                    }
                ),
                (
                    "@scrollctl:丙".to_string(),
                    ScrollIntentDto::ScrollTo {
                        axis: ScrollAxisDto::X,
                        offset: 10.0
                    }
                ),
                (
                    "@scrollctl:丙".to_string(),
                    ScrollIntentDto::ScrollTo {
                        axis: ScrollAxisDto::Y,
                        offset: 20.0
                    }
                ),
                (
                    "@scrollctl:丙".to_string(),
                    ScrollIntentDto::ScrollTo {
                        axis: ScrollAxisDto::X,
                        offset: 99.0
                    }
                ),
            ],
            "intent 参数/顺序精确到达（坐标形 X 先 Y 后；空句柄不入队）"
        );

        // 9905：快照 DTO → ScrollState record 八具名字段。
        push_str(&vm, &mut task, "@scrollctl:甲");
        shim_scroll_state(&mut task, &vm).unwrap();
        let nv = task.ram.pop_nv();
        let id = auto_lang_core::vm::rc::heap_ref_id(nv).expect("heap ref") as u64;
        {
            let obj = vm.get_heap_object(id).expect("record");
            let guard = obj.read().unwrap();
            let inst = guard
                .as_any()
                .downcast_ref::<GenericInstanceData>()
                .expect("GenericInstanceData");
            assert_eq!(inst.mono_name, "ScrollState");
            assert_eq!(field_f64(inst, "offset_x"), Some(11.5));
            assert_eq!(field_f64(inst, "offset_y"), Some(240.0));
            assert_eq!(field_f64(inst, "viewport_w"), Some(300.0));
            assert_eq!(field_f64(inst, "viewport_h"), Some(200.0));
            assert_eq!(field_f64(inst, "content_w"), Some(500.0));
            assert_eq!(field_f64(inst, "content_h"), Some(1000.0));
            assert_eq!(field_f64(inst, "progress_x"), Some(0.25));
            assert_eq!(field_f64(inst, "progress_y"), Some(0.3));
        }
        vm.rc_release_id(id);

        // 9922/9923：回执与状态编组（fake 特异配置）。
        {
            let mut replies = fake.anchor_reply.lock().unwrap();
            replies.insert(("裸串柄".into(), "键-空".into()), Err("empty_key"));
            replies.insert(("裸串柄".into(), "键-拒".into()), Err("unsupported"));
        }
        {
            let mut states = fake.anchor_states.lock().unwrap();
            states.insert(
                ("裸串柄".into(), "待".into()),
                ScrollAnchorStateDto::Pending,
            );
            states.insert(
                ("裸串柄".into(), "成".into()),
                ScrollAnchorStateDto::Applied,
            );
            states.insert(
                ("裸串柄".into(), "败".into()),
                ScrollAnchorStateDto::Invalid("not_found".into()),
            );
        }
        let reply = |task: &mut AutoTask, vm: &AutoVM, h: &str, k: &str| -> String {
            push_str(vm, task, h);
            push_str(vm, task, k);
            shim_scroll_to_child(task, vm).unwrap();
            pop_string(vm, task)
        };
        assert_eq!(reply(&mut task, &vm, "裸串柄", "键-正常"), "queued");
        assert_eq!(reply(&mut task, &vm, "裸串柄", "键-空"), "error:empty_key");
        assert_eq!(
            reply(&mut task, &vm, "裸串柄", "键-拒"),
            "error:unsupported"
        );

        let state = |task: &mut AutoTask, vm: &AutoVM, h: &str, k: &str| -> String {
            push_str(vm, task, h);
            push_str(vm, task, k);
            shim_scroll_anchor_state(task, vm).unwrap();
            pop_string(vm, task)
        };
        assert_eq!(state(&mut task, &vm, "裸串柄", "待"), "pending");
        assert_eq!(state(&mut task, &vm, "裸串柄", "成"), "applied");
        assert_eq!(state(&mut task, &vm, "裸串柄", "败"), "invalid:not_found");
        assert_eq!(state(&mut task, &vm, "裸串柄", "无记录"), "unknown");

        assert_eq!(task.ram.sp, sp0, "委托臂全部返回值弹出后任务栈净清");
    }

    /// 装配/身份：注入不 bump binding_revision；NI Clone 与 VM merge 保
    /// provider；构造期快照 + 两实例不串线 + 实例覆盖不污染进程默认
    ///（进程默认槽 save/restore 持 crate 级共享隔离锁——F-785-R1-01）。
    #[test]
    fn identity_clone_merge_and_instance_isolation() {
        let _slot = auto_lang_core::vm::host_capabilities::test_default_slot_lock();
        let saved = auto_lang_core::vm::host_capabilities::process_default_scroll_host_provider();

        // 进程默认 A → vm1 快照 A；换默认 B → vm2 快照 B；实例覆盖 C 只影响 vm2。
        let a = FakeScroll::new();
        let b = FakeScroll::new();
        let c = FakeScroll::new();
        auto_lang_core::vm::host_capabilities::set_process_default_scroll_host_provider(a.clone());
        let vm1 = AutoVM::new(VirtualFlash::new(0), 64);
        auto_lang_core::vm::host_capabilities::set_process_default_scroll_host_provider(b.clone());
        let vm2 = AutoVM::new(VirtualFlash::new(0), 64);

        let handle_of = |vm: &AutoVM| -> String {
            let mut task = AutoTask::new(0, 256, 0);
            auto_lang_core::vm::native::shim_scroll_controller(&mut task, vm).unwrap();
            pop_string(vm, &mut task)
        };
        let h1a = handle_of(&vm1);
        let h2b = handle_of(&vm2);
        assert_eq!(h1a, "p794-fx-句柄-1-🎉", "vm1 保持构造期快照 A");
        assert_eq!(h2b, "p794-fx-句柄-1-🎉", "vm2 取新默认 B（各自计数器独立）");
        // 两 fake 各自分配——区分靠 intents/句柄序列共享池不适用，改以
        // intents 隔离断言（同 handle 入队只落各自 provider）。
        vm2.set_scroll_host_provider(c.clone());
        let mut task = AutoTask::new(0, 256, 0);
        push_str(&vm2, &mut task, "@scrollctl:仅vm2");
        auto_lang_core::vm::native::shim_scroll_to_end(&mut task, &vm2).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()));
        assert_eq!(c.intents_snapshot().len(), 1, "vm2 覆盖后落 provider C");
        assert!(
            a.intents_snapshot().is_empty() && b.intents_snapshot().is_empty(),
            "vm1 快照 A 与旧默认 B 不串线收参"
        );
        // 进程默认不被实例覆盖改写。
        let cur =
            auto_lang_core::vm::host_capabilities::process_default_scroll_host_provider().unwrap();
        ScrollHostProvider::enqueue_intent(
            &*cur,
            "@scrollctl:默认",
            ScrollIntentDto::ToStart {
                axis: ScrollAxisDto::Y,
            },
        );
        assert_eq!(
            b.intents_snapshot().len(),
            1,
            "进程默认仍=B（实例覆盖不污染默认）"
        );

        // NI 层：注入不 bump revision；Clone 保 provider；VM merge 保 provider。
        let ni = auto_lang_core::vm::native::NativeInterface::production();
        let rev0 = ni.binding_revision();
        ni.set_scroll_host_provider(FakeScroll::new());
        assert_eq!(
            ni.binding_revision(),
            rev0,
            "provider 注入不 bump binding_revision"
        );
        let clone = ni.clone();
        assert!(
            clone.scroll_host_provider().is_some(),
            "NI Clone 不丢 provider"
        );
        let mut vm3 = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
        vm3.set_scroll_host_provider(FakeScroll::new());
        let mut carrier = auto_lang_core::vm::native::NativeInterface::new();
        carrier.register_dynamic(|_t: &mut AutoTask, _v: &AutoVM| Ok(()));
        vm3.merge_native_interface(&carrier);
        assert!(
            vm3.scroll_host_provider().is_some(),
            "VM merge 路径不丢 provider"
        );

        // 还原进程默认。
        match saved {
            Some(p) => {
                auto_lang_core::vm::host_capabilities::set_process_default_scroll_host_provider(p)
            }
            None => {
                auto_lang_core::vm::host_capabilities::clear_process_default_scroll_host_provider()
            }
        }
    }

    /// 真实 UI adapter 接线（V03 in-crate 锚）：实例注入 ScrollControllerProvider
    /// 后，八 shim 经 provider 委托既有 controller 注册表——intent 真实落队、
    /// 快照真实投影、锚点走 781 状态机（首绑/换绑/managed/重复请求语义由
    /// controller.rs 既有测试承担，此处证 provider 接线后仍可达同一注册表）。
    #[test]
    fn real_ui_adapter_delegates_to_controller_registry() {
        crate::ui::scroll::controller::reset_for_test();
        let vm = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 64);
        vm.set_scroll_host_provider(Arc::new(
            crate::ui::scroll_host_provider::ScrollControllerProvider,
        ));
        let mut task = AutoTask::new(0, 512, 0);
        let sp0 = task.ram.sp;

        // 9900 → 真实 "@scrollctl:N" 句柄。
        auto_lang_core::vm::native::shim_scroll_controller(&mut task, &vm).unwrap();
        let handle = pop_string(&vm, &mut task);
        assert!(
            handle.starts_with("@scrollctl:"),
            "真实 adapter 分配前缀句柄（{handle}）"
        );

        // 9902 → intent 落真实注册表队列；drain 后清空。
        task.pending_native_arg_count = 1;
        push_str(&vm, &mut task, &handle);
        auto_lang_core::vm::native::shim_scroll_to_end(&mut task, &vm).unwrap();
        assert!(auto_val::decode_bool(task.ram.pop_nv()));
        assert_eq!(
            crate::ui::scroll::controller::pending_intent_count(),
            1,
            "intent 经 provider 落真实注册表"
        );
        let (drained, _) = crate::ui::scroll::controller::drain_resolved_intents();
        assert!(drained.is_empty(), "未绑定 handle 静默丢弃（656 语义）");
        assert_eq!(crate::ui::scroll::controller::pending_intent_count(), 0);

        // 9905 → 真实快照投影：绑定+测量后六测量直读、progress 现算。
        crate::ui::scroll::controller::bind_controller(&handle, "pane_p794");
        let _ = crate::ui::scroll::controller::take_prime_requests();
        crate::ui::scroll::controller::note_controller_state(
            &handle,
            (10.0, 100.0),
            (300.0, 200.0),
            (500.0, 1000.0),
        );
        push_str(&vm, &mut task, &handle);
        auto_lang_core::vm::native::shim_scroll_state(&mut task, &vm).unwrap();
        let nv = task.ram.pop_nv();
        let id = auto_lang_core::vm::rc::heap_ref_id(nv).expect("heap ref") as u64;
        {
            use auto_lang_core::vm::generic_registry::GenericInstanceData;
            let obj = vm.get_heap_object(id).expect("record");
            let guard = obj.read().unwrap();
            let inst = guard
                .as_any()
                .downcast_ref::<GenericInstanceData>()
                .expect("GenericInstanceData");
            assert_eq!(field_f64(inst, "offset_y"), Some(100.0));
            assert_eq!(field_f64(inst, "viewport_w"), Some(300.0));
            assert_eq!(field_f64(inst, "progress_y"), Some(100.0 / 800.0));
        }
        vm.rc_release_id(id);

        // 9922/9923 → 781 状态机原语义（真实注册表：queued→pending；managed 拒绝）。
        crate::ui::scroll::controller::mark_controller_managed("@scrollctl:managed-p794");
        let call = |task: &mut AutoTask, vm: &AutoVM, h: &str, k: &str| -> (String, String) {
            push_str(vm, task, h);
            push_str(vm, task, k);
            auto_lang_core::vm::native::shim_scroll_to_child(task, vm).unwrap();
            let r1 = pop_string(vm, task);
            push_str(vm, task, h);
            push_str(vm, task, k);
            auto_lang_core::vm::native::shim_scroll_anchor_state(task, vm).unwrap();
            let r2 = pop_string(vm, task);
            (r1, r2)
        };
        let (r1, r2) = call(&mut task, &vm, &handle, "锚-甲🎉");
        assert_eq!(r1, "queued", "真实注册表入队成功");
        assert_eq!(r2, "pending", "781 状态机 pending");
        let (m1, _m2) = call(&mut task, &vm, "@scrollctl:managed-p794", "锚");
        assert_eq!(m1, "error:unsupported", "managed pane 入队即拒（781 语义）");

        // 781 状态机经 provider 全路径（V03）：失锚 not_found 一次性消费
        //（记录驻留 Invalid 不重试）+ 重复请求后写覆盖重新 pending。
        // 纯查询辅助：只走 anchor_state（call 的 to_child 会后写覆盖）。
        let state_only = |task: &mut AutoTask, vm: &AutoVM, h: &str, k: &str| -> String {
            push_str(vm, task, h);
            push_str(vm, task, k);
            auto_lang_core::vm::native::shim_scroll_anchor_state(task, vm).unwrap();
            pop_string(vm, task)
        };
        crate::ui::scroll::controller::note_controller_state(
            &handle,
            (0.0, 100.0),
            (400.0, 200.0),
            (800.0, 1000.0),
        );
        let _ = call(&mut task, &vm, &handle, "幽灵键");
        assert_eq!(state_only(&mut task, &vm, &handle, "幽灵键"), "pending");
        let finals = crate::ui::scroll::controller::resolve_anchor_requests(
            &std::collections::HashMap::new(),
            &std::collections::HashMap::new(),
            true,
        );
        assert!(finals.is_empty(), "key 不在当帧注册表 → 无终态");
        assert_eq!(
            state_only(&mut task, &vm, &handle, "幽灵键"),
            "invalid:not_found",
            "失锚一次性失败（781 语义经 provider 查询面）"
        );
        assert_eq!(
            state_only(&mut task, &vm, &handle, "幽灵键"),
            "invalid:not_found",
            "Invalid 记录驻留不重试（再次 resolve 无产出后仍 invalid）"
        );
        // 后写覆盖：重新入队同 key → 覆盖回 pending（用户重定位语义）。
        let _ = call(&mut task, &vm, &handle, "幽灵键");
        assert_eq!(
            state_only(&mut task, &vm, &handle, "幽灵键"),
            "pending",
            "重复请求后写覆盖重新 pending（781 语义）"
        );

        assert_eq!(task.ram.sp, sp0);
        crate::ui::scroll::controller::reset_for_test();
    }
}
