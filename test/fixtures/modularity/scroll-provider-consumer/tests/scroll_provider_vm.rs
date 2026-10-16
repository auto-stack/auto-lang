//! PLAN-794 V-01/V-02：外部 scroll host provider 注入真 VM 断言。
//!
//! 覆盖（计划 §6 矩阵 + T-01 冻结矩阵）：
//! - V-01 真注入（no-ui 全管线）：fake provider 实值经 VM 内建
//!   9900..9905/9922/9923 到达 stdout（可区分 Unicode 句柄/键、八字段
//!   快照 record、锚点回执/状态）；intent 收精确 (handle,axis,offset/
//!   delta) 参数；provider 实现零 UI 类型（独立 manifest 证明）。
//! - V-02 arity/轴/Unicode/缺席/优先/复用矩阵：可选轴形按签名合同路由
//!   （T-01 勘定——轴在顶先弹）；未知轴词归 Y；缺席降级=T-01 冻结臂
//!   逐字面值；两 VM 不串线；显式默认不被实例覆盖改写；Clone/merge 保
//!   provider；重复调用清栈 + heap 活计数回基线（reap）。
//!
//! 进程默认槽是进程级全局——所有触槽测试经 `serialized()` 互斥串行并做
//! save/restore（F-785-R1-01 同款纪律）。

use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use auto_lang::vm::engine::AutoVM;
use auto_lang::vm::host_capabilities::{
    clear_process_default_scroll_host_provider, process_default_scroll_host_provider,
    set_process_default_scroll_host_provider, ScrollAnchorStateDto, ScrollAxisDto,
    ScrollHostProvider, ScrollIntentDto, ScrollSnapshotDto,
};
use auto_lang::vm::native::NativeInterface;
use auto_lang::vm::task::AutoTask;
use auto_lang::vm::virt_memory::VirtualFlash;

// ── 测试序列化（进程默认槽互斥） ─────────────────────────────────────────

fn serialized() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let m = LOCK.get_or_init(|| Mutex::new(()));
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// 进程默认槽 save/restore RAII（测试结束还原进入前状态）。
struct DefaultGuard(Option<Arc<dyn ScrollHostProvider>>);

impl DefaultGuard {
    fn new() -> Self {
        Self(process_default_scroll_host_provider())
    }
}

impl Drop for DefaultGuard {
    fn drop(&mut self) {
        match self.0.take() {
            Some(p) => set_process_default_scroll_host_provider(p),
            None => clear_process_default_scroll_host_provider(),
        }
    }
}

// ── fake provider（可区分哨兵；零 UI 类型） ──────────────────────────────

struct FakeScroll {
    /// next_controller_handle 递增后缀。
    handle_seq: Mutex<u32>,
    /// enqueue_intent 收到的 (handle, intent) 记录（精确断言用）。
    intents: Mutex<Vec<(String, ScrollIntentDto)>>,
    /// controller_snapshot 返回值（八字段特异值）。
    snapshot: ScrollSnapshotDto,
    /// enqueue_anchor_request 收到的 (handle, key) 记录 + 回执映射。
    anchor_calls: Mutex<Vec<(String, String)>>,
    anchor_reply: Mutex<std::collections::HashMap<(String, String), Result<(), &'static str>>>,
    /// anchor_request_state 回供映射。
    anchor_states: Mutex<std::collections::HashMap<(String, String), ScrollAnchorStateDto>>,
}

impl FakeScroll {
    fn with_sentinel_snapshot() -> Arc<Self> {
        Arc::new(Self {
            handle_seq: Mutex::new(0),
            intents: Mutex::new(Vec::new()),
            snapshot: ScrollSnapshotDto {
                offset_x: 12.5,
                offset_y: 250.0,
                viewport_w: 320.0,
                viewport_h: 200.0,
                content_w: 640.0,
                content_h: 1000.0,
                progress_x: 0.0,
                progress_y: 0.3125,
            },
            anchor_calls: Mutex::new(Vec::new()),
            anchor_reply: Mutex::new(std::collections::HashMap::new()),
            anchor_states: Mutex::new(std::collections::HashMap::new()),
        })
    }

    fn intents_snapshot(&self) -> Vec<(String, ScrollIntentDto)> {
        self.intents.lock().unwrap().clone()
    }

    fn anchor_calls_snapshot(&self) -> Vec<(String, String)> {
        self.anchor_calls.lock().unwrap().clone()
    }
}

impl ScrollHostProvider for FakeScroll {
    fn next_controller_handle(&self) -> String {
        let mut n = self.handle_seq.lock().unwrap();
        *n += 1;
        format!("p794-fx/句柄-{n}-🎉")
    }
    fn enqueue_intent(&self, handle: &str, intent: ScrollIntentDto) {
        self.intents
            .lock()
            .unwrap()
            .push((handle.to_string(), intent));
    }
    fn controller_snapshot(&self, _handle: &str) -> ScrollSnapshotDto {
        self.snapshot
    }
    fn enqueue_anchor_request(&self, handle: &str, key: &str) -> Result<(), &'static str> {
        self.anchor_calls
            .lock()
            .unwrap()
            .push((handle.to_string(), key.to_string()));
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

// ── .at 程序（短名 intrinsic 经 codegen 表解析到 9900..9905/9922/9923） ──

fn stdout_lines(s: &str) -> Vec<&str> {
    s.lines().collect()
}

// ── V-01 真注入（no-ui 全管线） ──────────────────────────────────────────

/// V-01(a) 全管线：进程默认 fake → 八函数实值/回执经真 VM stdout 到达。
#[test]
fn v01_process_default_reaches_real_vm_stdout() {
    let _g = serialized();
    let _guard = DefaultGuard::new();

    let fake = FakeScroll::with_sentinel_snapshot();
    set_process_default_scroll_host_provider(fake.clone());

    // 9900：fake 句柄（Unicode 可区分）。
    let (_, out) =
        auto_lang::run_with_capture("let h = scroll_controller()\nprint(h)\n").expect("run");
    assert_eq!(
        stdout_lines(&out),
        vec!["p794-fx/句柄-1-🎉"],
        "scroll_controller 实值到达 stdout（got {out:?}）"
    );

    // 9902/9903/9904（坐标形）：入队回执 true；intent 参数精确（T-01 勘定
    // 后的签名合同路由——轴形 9902 显式 x）。
    let prog = r#"
let h = scroll_controller()
let a = scroll_to_end(h)
let b = scroll_to_end(h, "x")
let c = scroll_by(h, 40.5)
let d = scroll_by(h, "x", -7.0)
let e = scroll_to(h, 10.0, 20.0)
print(a)
print(b)
print(c)
print(d)
print(e)
"#;
    let (_, out) = auto_lang::run_with_capture(prog).expect("run intents");
    assert_eq!(
        stdout_lines(&out),
        vec!["true", "true", "true", "true", "true"],
        "intent 族在位回执 Bool true（got {out:?}）"
    );
    assert_eq!(
        fake.intents_snapshot(),
        vec![
            (
                "p794-fx/句柄-2-🎉".into(),
                ScrollIntentDto::ToEnd {
                    axis: ScrollAxisDto::Y
                }
            ),
            (
                "p794-fx/句柄-2-🎉".into(),
                ScrollIntentDto::ToEnd {
                    axis: ScrollAxisDto::X
                }
            ),
            (
                "p794-fx/句柄-2-🎉".into(),
                ScrollIntentDto::ScrollBy {
                    axis: ScrollAxisDto::Y,
                    delta: 40.5
                }
            ),
            (
                "p794-fx/句柄-2-🎉".into(),
                ScrollIntentDto::ScrollBy {
                    axis: ScrollAxisDto::X,
                    delta: -7.0
                }
            ),
            (
                "p794-fx/句柄-2-🎉".into(),
                ScrollIntentDto::ScrollTo {
                    axis: ScrollAxisDto::X,
                    offset: 10.0
                }
            ),
            (
                "p794-fx/句柄-2-🎉".into(),
                ScrollIntentDto::ScrollTo {
                    axis: ScrollAxisDto::Y,
                    offset: 20.0
                }
            ),
        ],
        "intent (handle,axis,offset/delta) 精确到达（坐标形 X 先 Y 后）"
    );

    // 9905：八字段 record（具名可读）。
    let prog = r#"
let h = scroll_controller()
let s = scroll_state(h)
print(s.offset_x)
print(s.offset_y)
print(s.viewport_w)
print(s.viewport_h)
print(s.content_w)
print(s.content_h)
print(s.progress_y)
"#;
    let (_, out) = auto_lang::run_with_capture(prog).expect("run state");
    assert_eq!(
        stdout_lines(&out),
        vec!["12.5", "250", "320", "200", "640", "1000", "0.3125"],
        "scroll_state 八字段 record 经 VM 到达（got {out:?}）"
    );

    // 9922/9923：回执与状态（fake 特异配置——本程序是第 4 次
    // scroll_controller 分配，句柄后缀 -4）。
    {
        let mut replies = fake.anchor_reply.lock().unwrap();
        replies.insert(
            ("p794-fx/句柄-4-🎉".into(), "键-拒".into()),
            Err("unsupported"),
        );
    }
    {
        let mut states = fake.anchor_states.lock().unwrap();
        states.insert(
            ("p794-fx/句柄-4-🎉".into(), "锚-成".into()),
            ScrollAnchorStateDto::Applied,
        );
        states.insert(
            ("p794-fx/句柄-4-🎉".into(), "锚-败".into()),
            ScrollAnchorStateDto::Invalid("not_bound".into()),
        );
    }
    let prog = r#"
let h = scroll_controller()
let r1 = scroll_to_child(h, "锚-甲🎉")
let r2 = scroll_to_child(h, "键-拒")
let s1 = scroll_anchor_state(h, "锚-成")
let s2 = scroll_anchor_state(h, "锚-败")
let s3 = scroll_anchor_state(h, "无记录")
print(r1)
print(r2)
print(s1)
print(s2)
print(s3)
"#;
    let (_, out) = auto_lang::run_with_capture(prog).expect("run anchors");
    assert_eq!(
        stdout_lines(&out),
        vec![
            "queued",
            "error:unsupported",
            "applied",
            "invalid:not_bound",
            "unknown",
        ],
        "锚点族回执/状态编组经 VM 到达（got {out:?}）"
    );
    assert_eq!(
        fake.anchor_calls_snapshot(),
        vec![
            ("p794-fx/句柄-4-🎉".to_string(), "锚-甲🎉".to_string()),
            ("p794-fx/句柄-4-🎉".to_string(), "键-拒".to_string()),
        ],
        "to_child 收到精确 (handle,key) Unicode 参数"
    );
}

/// V-01(b) 低层双活实例：构造期快照 + 两 VM 不串线 + 实例覆盖独立
///（直调 shim，不经程序编译）。
#[test]
fn v01_two_live_instances_do_not_cross_talk() {
    use auto_lang::vm::native::shim_scroll_by;

    let _g = serialized();
    let _guard = DefaultGuard::new();

    let a = FakeScroll::with_sentinel_snapshot();
    let b = FakeScroll::with_sentinel_snapshot();
    let c = FakeScroll::with_sentinel_snapshot();

    set_process_default_scroll_host_provider(a.clone());
    let vm1 = AutoVM::new(VirtualFlash::new(0), 64); // 快照 A
    set_process_default_scroll_host_provider(b.clone());
    let vm2 = AutoVM::new(VirtualFlash::new(0), 64); // 快照 B

    let mut task1 = AutoTask::new(0, 256, 0);
    let idx = vm1.add_string("@vm1-句柄".as_bytes().to_vec());
    vm1.rc_push_str_idx(&mut task1, idx);
    task1.ram.push_f64(5.0);
    task1.pending_native_arg_count = 2;
    shim_scroll_by(&mut task1, &vm1).unwrap();
    assert!(auto_val::decode_bool(task1.ram.pop_nv()));
    assert_eq!(
        a.intents_snapshot(),
        vec![(
            "@vm1-句柄".to_string(),
            ScrollIntentDto::ScrollBy {
                axis: ScrollAxisDto::Y,
                delta: 5.0
            }
        )],
        "vm1 落构造期快照 A"
    );
    assert!(
        b.intents_snapshot().is_empty(),
        "vm1 的调用不得串线到默认 B"
    );

    // 实例覆盖 C：只影响 vm2。
    vm2.set_scroll_host_provider(c.clone());
    let mut task2 = AutoTask::new(0, 256, 0);
    let idx = vm2.add_string("@vm2-句柄".as_bytes().to_vec());
    vm2.rc_push_str_idx(&mut task2, idx);
    task2.ram.push_f64(6.0);
    task2.pending_native_arg_count = 2;
    shim_scroll_by(&mut task2, &vm2).unwrap();
    assert!(auto_val::decode_bool(task2.ram.pop_nv()));
    assert_eq!(
        c.intents_snapshot(),
        vec![(
            "@vm2-句柄".to_string(),
            ScrollIntentDto::ScrollBy {
                axis: ScrollAxisDto::Y,
                delta: 6.0
            }
        )],
        "vm2 覆盖后落 provider C"
    );
    assert!(
        a.intents_snapshot().len() == 1 && b.intents_snapshot().is_empty(),
        "vm2 覆盖不影响 vm1/默认"
    );
    // 进程默认不被实例覆盖改写。
    let cur = process_default_scroll_host_provider().unwrap();
    ScrollHostProvider::enqueue_intent(
        &*cur,
        "@default-句柄",
        ScrollIntentDto::ToStart {
            axis: ScrollAxisDto::Y,
        },
    );
    assert_eq!(b.intents_snapshot().len(), 1, "进程默认仍=B");
}

// ── V-02 arity/轴/缺席/优先/复用矩阵 ─────────────────────────────────────

/// 缺席降级（no-ui 无装配路径——清槽即全局缺席）：八函数 T-01 冻结臂
/// 逐字面值（程序轨：句柄 Int 0 / intent 族 false / 锚点族字符串）；
/// record/float 形经直调 shim 断言（程序轨 print 浮点格式不参与契约）。
#[test]
fn v02_absent_capability_is_frozen_degradation() {
    use auto_lang::vm::native::{
        shim_scroll_anchor_state, shim_scroll_state, shim_scroll_to_child,
    };

    let _g = serialized();
    let _guard = DefaultGuard::new();
    clear_process_default_scroll_host_provider();

    // 程序轨：9900=Int 0（print 出 "0"，非 fake 句柄、非空串）。
    let (_, out) =
        auto_lang::run_with_capture("let h = scroll_controller()\nprint(h)\n").expect("run");
    assert_eq!(
        stdout_lines(&out),
        vec!["0"],
        "缺席 controller=Int 0（got {out:?}）"
    );

    // 程序轨：intent 族=Bool false（实参照常消费，程序正常完成不死锁）。
    let prog = r#"
let h = scroll_controller()
let a = scroll_to_start(h)
let b = scroll_to_end(h, "x")
let c = scroll_by(h, 1.0)
let d = scroll_to(h, 1.0, 2.0)
print(a)
print(b)
print(c)
print(d)
"#;
    let (_, out) = auto_lang::run_with_capture(prog).expect("run degrade");
    assert_eq!(
        stdout_lines(&out),
        vec!["false", "false", "false", "false"],
        "缺席 intent 族=false（实参照常消费；got {out:?}）"
    );

    // 程序轨：锚点族字符串降级。
    let prog = r#"
let h = scroll_controller()
let r = scroll_to_child(h, "键")
let s = scroll_anchor_state(h, "键")
print(r)
print(s)
"#;
    let (_, out) = auto_lang::run_with_capture(prog).expect("run anchors");
    assert_eq!(
        stdout_lines(&out),
        vec!["error:unsupported", "unknown"],
        "缺席锚点族=error:unsupported/unknown（got {out:?}）"
    );

    // 直调 shim：9905=Float 0.0（非 record）；9922/9923 字符串逐字节。
    let vm = AutoVM::new(VirtualFlash::new(0), 64);
    assert!(
        vm.scroll_host_provider().is_none(),
        "no-ui 裸 VM 无 provider"
    );
    let mut task = AutoTask::new(0, 256, 0);
    let idx = vm.add_string("@absent".as_bytes().to_vec());
    vm.rc_push_str_idx(&mut task, idx);
    shim_scroll_state(&mut task, &vm).unwrap();
    assert_eq!(task.ram.pop_f64(), 0.0, "缺席 state=Float 0.0（非 record）");

    let pop_str = |vm: &AutoVM, task: &mut AutoTask| -> String {
        let nv = task.ram.pop_nv();
        assert!(auto_val::is_string(nv));
        let i = auto_val::decode_string(nv);
        vm.get_string(i)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default()
    };
    let idx = vm.add_string("@absent".as_bytes().to_vec());
    vm.rc_push_str_idx(&mut task, idx);
    let idx = vm.add_string("键".as_bytes().to_vec());
    vm.rc_push_str_idx(&mut task, idx);
    shim_scroll_to_child(&mut task, &vm).unwrap();
    assert_eq!(pop_str(&vm, &mut task), "error:unsupported");
    let idx = vm.add_string("@absent".as_bytes().to_vec());
    vm.rc_push_str_idx(&mut task, idx);
    let idx = vm.add_string("键".as_bytes().to_vec());
    vm.rc_push_str_idx(&mut task, idx);
    shim_scroll_anchor_state(&mut task, &vm).unwrap();
    assert_eq!(pop_str(&vm, &mut task), "unknown");
}

/// Clone/merge 保 provider + 注入不 bump binding_revision + ID 钉扎
/// （9900..9905/9922/9923；族间不撞）。
#[test]
fn v02_identity_clone_merge_rules_preserved() {
    use auto_lang::vm::native::{
        shim_scroll_anchor_state, shim_scroll_by, shim_scroll_controller, shim_scroll_state,
        shim_scroll_to, shim_scroll_to_child, shim_scroll_to_end, shim_scroll_to_start,
    };

    let ni = NativeInterface::production();
    assert_eq!(ni.resolve("auto.scroll.controller"), Some(9900));
    assert_eq!(ni.resolve("auto.scroll.to_start"), Some(9901));
    assert_eq!(ni.resolve("auto.scroll.to_end"), Some(9902));
    assert_eq!(ni.resolve("auto.scroll.by"), Some(9903));
    assert_eq!(ni.resolve("auto.scroll.to"), Some(9904));
    assert_eq!(ni.resolve("auto.scroll.state"), Some(9905));
    assert_eq!(ni.resolve("auto.scroll.to_child"), Some(9922));
    assert_eq!(ni.resolve("auto.scroll.anchor_state"), Some(9923));

    let rev0 = ni.binding_revision();
    ni.set_scroll_host_provider(FakeScroll::with_sentinel_snapshot());
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

    let mut vm = AutoVM::new(VirtualFlash::new(0), 64);
    vm.set_scroll_host_provider(FakeScroll::with_sentinel_snapshot());
    let mut carrier = NativeInterface::new();
    carrier.register_dynamic(|_t: &mut AutoTask, _v: &AutoVM| Ok(()));
    vm.merge_native_interface(&carrier);
    assert!(
        vm.scroll_host_provider().is_some(),
        "VM merge 路径不丢 provider"
    );

    // 八 shim 在 no-ui 构建下可直调（feature 中性——迁移前 no-ui 臂 cfg
    // 分离形态已统一）；每次调用前按 codegen 压序（左→右）压参。
    let mut task = AutoTask::new(0, 256, 0);
    let push_s = |vm: &AutoVM, task: &mut AutoTask, s: &str| {
        let idx = vm.add_string(s.as_bytes().to_vec());
        vm.rc_push_str_idx(task, idx);
    };
    shim_scroll_controller(&mut task, &vm).unwrap();
    let _ = task.ram.pop_nv();
    task.pending_native_arg_count = 1;
    push_s(&vm, &mut task, "@x");
    shim_scroll_to_start(&mut task, &vm).unwrap();
    let _ = task.ram.pop_nv();
    push_s(&vm, &mut task, "@x");
    shim_scroll_to_end(&mut task, &vm).unwrap();
    let _ = task.ram.pop_nv();
    push_s(&vm, &mut task, "@x");
    task.ram.push_f64(1.0);
    task.pending_native_arg_count = 2;
    shim_scroll_by(&mut task, &vm).unwrap();
    let _ = task.ram.pop_nv();
    push_s(&vm, &mut task, "@x");
    task.ram.push_f64(1.0);
    task.ram.push_f64(2.0);
    shim_scroll_to(&mut task, &vm).unwrap();
    let _ = task.ram.pop_nv();
    push_s(&vm, &mut task, "@x");
    shim_scroll_state(&mut task, &vm).unwrap();
    let nv = task.ram.pop_nv();
    let rec_id = auto_lang::vm::rc::heap_ref_id(nv).expect("record ref") as u64;
    vm.rc_release_id(rec_id);
    push_s(&vm, &mut task, "@x");
    push_s(&vm, &mut task, "k");
    shim_scroll_to_child(&mut task, &vm).unwrap();
    let _ = task.ram.pop_nv();
    push_s(&vm, &mut task, "@x");
    push_s(&vm, &mut task, "k");
    shim_scroll_anchor_state(&mut task, &vm).unwrap();
    let _ = task.ram.pop_nv();
}

/// 重复调用清栈 + heap 活计数回基线（reap：无悬空/无增长——空句柄早退
/// 与成功路径同样配平）。
#[test]
fn v02_repeated_calls_reap_back_to_baseline() {
    use auto_lang::vm::native::{shim_scroll_state, shim_scroll_to_end};

    let vm = AutoVM::new(VirtualFlash::new(0), 64);
    vm.set_scroll_host_provider(FakeScroll::with_sentinel_snapshot());
    let mut task = AutoTask::new(0, 512, 0);
    let sp0 = task.ram.sp;
    let live0 = vm.rc_stats().live_heap;

    for _ in 0..32 {
        // 成功路径：record 产生 heap 引用，弹出后释放回基线。
        let idx = vm.add_string("@reap".as_bytes().to_vec());
        vm.rc_push_str_idx(&mut task, idx);
        shim_scroll_state(&mut task, &vm).unwrap();
        let nv = task.ram.pop_nv();
        let id = auto_lang::vm::rc::heap_ref_id(nv).expect("record ref") as u64;
        vm.rc_release_id(id);
        // 空句柄早退：不入队但配平。
        let idx = vm.add_string(Vec::new());
        vm.rc_push_str_idx(&mut task, idx);
        task.pending_native_arg_count = 1;
        shim_scroll_to_end(&mut task, &vm).unwrap();
        assert!(
            auto_val::decode_bool(task.ram.pop_nv()),
            "空句柄在位仍 true"
        );
    }

    assert_eq!(task.ram.sp, sp0, "重复调用后任务栈回基线");
    // 宽限窗（PLAN-062 T12）对象在静止点全收——reap_all 后活计数回基线。
    vm.reap_all();
    assert_eq!(
        vm.rc_stats().live_heap,
        live0,
        "heap 活计数回基线（record 引用无泄漏）"
    );
}
