//! PLAN-981（`IF-981-GFX-HOST-PORT`）：AGE 图形宿主的 **AV 真跑**验收。
//!
//! 覆盖用户点名的四件事：
//! 1. **typed 派发**：真实 AGE 源里的 `tick.dt()`/`row.place()`/`input.*`/
//!    `render2d.spawn()` 在 AV 执行中被分流到注册 provider（参数按受检类型
//!    到达、返回值按调用槽 `MirType` 复核）；
//! 2. **注册/生命周期**：一个 `AgeGfxSession` = 一次 Run；`stop()` 后具名拒、
//!    provider 丢弃；新会话重新注册；
//! 3. **每 Tick 预算**：累计扣费超过 32 → `age-gfx.credits` 具名拒、零效果、
//!    本 Tick sticky；`begin_tick()` 重置；
//! 4. **正常 AGE 源 AV 实测**：f32 运算与 gfx 调用同一 callback（`x + vx*dt`）
//!    真值可观察。
//!
//! 诚实边界：`query2d.segment/sweep` 与 `image2d.png` 返回端口 record，多字段
//! 宿主返回 ABI 未接（lowering 具名拒 `lower.age-gfx-port-result`）——本文件
//! 不伪造它们。

use std::cell::RefCell;
use std::rc::Rc;

use ace_av::age_gfx::{AgeGfxFault, AgeGfxHost, AgeGfxReturn, AgeGfxSession};
use ace_av_core::contract::{StrHandle, Value};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn input(src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: src.into(),
        provider: None,
    }
}

fn compile(src: &str) -> ace_core::source::age_package::AgePackageCompilation {
    compile_age_package(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .unwrap_or_else(|ds| {
        panic!(
            "front door rejected the host-port source: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })
}

type CallLog = Rc<RefCell<Vec<(String, Vec<String>)>>>;

/// 记录每次派发的 fake provider（算法归 UI058，本文件只验接面）。
struct FakeHost {
    calls: CallLog,
    dt: f32,
}

fn fmt(v: &Value) -> String {
    match v {
        Value::I32(i) => format!("i32:{i}"),
        Value::F32(f) => format!("f32:{f}"),
        Value::Bool(b) => format!("bool:{b}"),
        Value::Str(_) | Value::StrView(_) => "str".into(),
        other => format!("{other:?}"),
    }
}

impl AgeGfxHost for FakeHost {
    fn dispatch(&mut self, symbol: &str, args: &[Value]) -> Result<AgeGfxReturn, AgeGfxFault> {
        self.calls
            .borrow_mut()
            .push((symbol.to_string(), args.iter().map(fmt).collect()));
        match symbol {
            "age.tick.dt.v1" => Ok(AgeGfxReturn::Scalar(Value::F32(self.dt))),
            "age.input.space.v1" => Ok(AgeGfxReturn::Scalar(Value::I32(1))),
            "age.input.x.v1" => Ok(AgeGfxReturn::Scalar(Value::F32(0.25))),
            "age.row.place.v1" => Ok(AgeGfxReturn::Scalar(Value::I32(1))),
            "age.render2d.spawn.v1" => Ok(AgeGfxReturn::Scalar(Value::I32(7))),
            // G4 冻结：`render2d.frame(i32, str, i32, i32)` 的第二参是**已声明、
            // 带帧表的图像资源名**（不是文件路径）；provider 原样收到。
            "age.render2d.frame.v1" => Ok(AgeGfxReturn::Scalar(Value::I32(2))),
            // 端口 op：一次调用返回完整字段序列（AGE_PORT_FIELDS 声明序）。
            "age.query2d.segment.v1" => Ok(AgeGfxReturn::Record(vec![
                Value::Bool(true),
                Value::I32(3),
                Value::F32(0.5),
                Value::F32(1.0),
                Value::F32(2.0),
                Value::F32(0.0),
                Value::F32(-1.0),
            ])),
            "age.query2d.sweep.v1" => Ok(AgeGfxReturn::Record(vec![
                Value::Bool(true),
                Value::I32(5),
                Value::F32(0.25),
                Value::F32(4.0),
                Value::F32(5.0),
                Value::F32(0.0),
                Value::F32(1.0),
                Value::Bool(false),
            ])),
            "age.image2d.png.v1" => Ok(AgeGfxReturn::Record(vec![
                Value::Str(StrHandle::detached(b"deadbeef".to_vec().into_boxed_slice())),
                Value::I32(16),
                Value::I32(64),
                Value::I32(32),
            ])),
            other => Err(AgeGfxFault::new(
                "unknown-op",
                format!("`{other}` is not implemented by this provider"),
            )),
        }
    }
}

fn session(dt: f32) -> (AgeGfxSession, CallLog) {
    let calls: CallLog = Rc::new(RefCell::new(Vec::new()));
    let host = FakeHost {
        calls: calls.clone(),
        dt,
    };
    (AgeGfxSession::new(Box::new(host)), calls)
}

/// 真实 AGE 源：f32 积分 + `row.place`/`tick.dt` 同一 callback。
const STEP: &str = "pub fn step() f32 {\n    let x f32 = 1.25\n    let vx f32 = 2.0\n    let p i32 = row.place(1, x, x + vx * tick.dt())\n    return x + vx * tick.dt()\n}\n";

fn run(
    session: &mut AgeGfxSession,
    compiled: &ace_core::source::age_package::AgePackageCompilation,
    name: &str,
) -> Result<Value, String> {
    let symbol = compiled
        .symbol_of(name)
        .unwrap_or_else(|| panic!("no symbol for `{name}`"));
    // HIR def id 约定：`d_<function name>`（package 函数名即导出符号）。
    session.run_def(&compiled.checked, &format!("d_{symbol}"))
}

#[test]
fn p981_gfx_host_dispatches_typed_calls_into_the_registered_provider() {
    let compiled = compile(STEP);
    let (mut s, calls) = session(0.5);
    s.begin_tick();
    let value = run(&mut s, &compiled, "step").expect("host-dispatched fn executes");
    // 1.25 + 2.0 * 0.5 = 2.25（f32 真值，不是 int 近似）。
    assert_eq!(value, Value::F32(2.25));
    assert_eq!(s.credits_used(), 3, "row.place + 2x tick.dt = 3 credits");
    let log = calls.borrow();
    let place = log
        .iter()
        .find(|(sym, _)| sym == "age.row.place.v1")
        .expect("row.place dispatched");
    // 参数按受检类型到达：row 句柄 i32，位置/目标 f32（f32 字面量已求值）。
    assert_eq!(
        place.1,
        vec![
            "i32:1".to_string(),
            "f32:1.25".to_string(),
            "f32:2.25".to_string()
        ]
    );
    assert_eq!(
        log.iter().filter(|(s, _)| s == "age.tick.dt.v1").count(),
        2,
        "both dt reads were dispatched"
    );
}

/// 第 33 credit 起 sticky 拒：零效果、provider 不再被触达、`begin_tick` 重置。
#[test]
fn p981_gfx_host_per_tick_budget_is_sticky_and_resets() {
    const BURN: &str = "fn charge(n int) i32 {\n    if n > 0 {\n        return row.place(n, 0.0, 0.0) + charge(n - 1)\n    }\n    return 0\n}\npub fn burn() i32 {\n    return charge(40)\n}\n";
    let compiled = compile(BURN);
    let (mut s, calls) = session(0.5);
    s.begin_tick();
    let err = run(&mut s, &compiled, "burn").expect_err("40 charged calls exceed 32");
    assert!(
        err.contains("age-gfx.credits") && err.contains("per-Tick budget 32"),
        "named over-budget fault: {err}"
    );
    assert_eq!(s.credits_used(), 32, "exactly 32 credits were charged");
    assert!(s.tick_faulted());
    assert_eq!(
        calls.borrow().len(),
        32,
        "the over-budget attempt never reached the provider"
    );
    // sticky：同一 Tick 内的下一个调用直接拒，不触达 provider、不改账。
    let err = run(&mut s, &compiled, "burn").expect_err("sticky within the same Tick");
    assert!(err.contains("age-gfx.credits"), "{err}");
    assert_eq!(calls.borrow().len(), 32);

    // 新 Tick：累计与 sticky 都清零，重新跑满 32。
    s.begin_tick();
    let err = run(&mut s, &compiled, "burn").expect_err("next tick charges again");
    assert!(err.contains("age-gfx.credits"), "{err}");
    assert_eq!(s.credits_used(), 32);
    assert_eq!(
        calls.borrow().len(),
        64,
        "a fresh tick re-charges from zero"
    );
}

/// 会话生命周期：`stop()` 后具名拒（provider 已丢弃）；新会话重新注册。
#[test]
fn p981_gfx_host_session_lifecycle_is_run_scoped() {
    let compiled = compile(STEP);
    let (mut s, calls) = session(0.5);
    s.begin_tick();
    run(&mut s, &compiled, "step").expect("first run executes");
    s.stop();
    assert!(s.is_stopped());
    let err = run(&mut s, &compiled, "step").expect_err("a stopped session must refuse");
    assert!(
        err.contains("age-gfx.session-stopped"),
        "named stopped-session refusal: {err}"
    );
    let before = calls.borrow().len();
    // 新 Run = 新会话：重新注册后照常执行。
    let (mut s2, calls2) = session(0.5);
    s2.begin_tick();
    run(&mut s2, &compiled, "step").expect("a new session registers again");
    assert!(calls2.borrow().len() > 0, "the new provider received calls");
    assert_eq!(
        calls.borrow().len(),
        before,
        "the stopped session dispatched nothing"
    );
}

/// 输入/精灵族同样走 typed 派发（字符串参数原样到达）。
#[test]
fn p981_gfx_host_input_and_sprite_calls_dispatch() {
    const PROBE: &str = "pub fn probe() i32 {\n    let px f32 = input.x()\n    let h i32 = render2d.spawn(\"sprite.png\", px, 0.0)\n    return h + input.space()\n}\n";
    let compiled = compile(PROBE);
    let (mut s, calls) = session(0.5);
    s.begin_tick();
    let value = run(&mut s, &compiled, "probe").expect("input/sprite host calls execute");
    assert_eq!(value, Value::I32(8), "7 + 1");
    let log = calls.borrow();
    let spawn = log
        .iter()
        .find(|(sym, _)| sym == "age.render2d.spawn.v1")
        .expect("spawn dispatched");
    assert_eq!(
        spawn.1,
        vec![
            "str".to_string(),
            "f32:0.25".to_string(),
            "f32:0".to_string()
        ]
    );
}

/// 端口 op：**一次**宿主调用返回完整 typed record；查询照原价收费（segment 8 /
/// sweep 16 / image 1），端口字段读 0 credit，且 provider 只收到一次调用。
#[test]
fn p981_gfx_host_port_record_ops_publish_one_typed_record_per_call() {
    const SEG: &str = "pub fn probe() f32 {\n    let q Cast2D = query2d.segment(1, true, 0.0, 0.0, 1.0, 0.0)\n    return q.point_x + q.point_y\n}\n";
    let compiled = compile(SEG);
    let (mut s, calls) = session(0.5);
    s.begin_tick();
    assert_eq!(
        run(&mut s, &compiled, "probe").expect("segment record op executes"),
        Value::F32(3.0),
        "field reads come from the published record (1.0 + 2.0)"
    );
    assert_eq!(
        s.credits_used(),
        8,
        "one segment call = 8 credits; field reads are free"
    );
    assert_eq!(
        calls.borrow().len(),
        1,
        "exactly one host call for the query"
    );

    const SWEEP: &str = "pub fn sweep_p() bool {\n    let s Sweep2D = query2d.sweep(1, true, 1.0, 0.0)\n    return s.start_overlap\n}\n";
    let compiled = compile(SWEEP);
    let (mut s, calls) = session(0.5);
    s.begin_tick();
    assert_eq!(
        run(&mut s, &compiled, "sweep_p").expect("sweep record op executes"),
        Value::Bool(false)
    );
    assert_eq!(s.credits_used(), 16);
    assert_eq!(calls.borrow().len(), 1);

    const IMG: &str = "pub fn img_len() i32 {\n    let img Image2D = image2d.png(\"assets/x.png\")\n    return img.byte_length\n}\n";
    let compiled = compile(IMG);
    let (mut s, calls) = session(0.5);
    s.begin_tick();
    assert_eq!(
        run(&mut s, &compiled, "img_len").expect("image2d record op executes"),
        Value::I32(16)
    );
    assert_eq!(s.credits_used(), 1);
    assert_eq!(calls.borrow().len(), 1);
}

/// 宿主返回的记录与冻结端口字段数/类型不符：具名拒、不写结果槽（零效果）。
#[test]
fn p981_gfx_host_port_record_shape_and_types_are_checked() {
    struct ShortHost;
    impl AgeGfxHost for ShortHost {
        fn dispatch(&mut self, _s: &str, _a: &[Value]) -> Result<AgeGfxReturn, AgeGfxFault> {
            Ok(AgeGfxReturn::Record(vec![Value::I32(0)]))
        }
    }
    let compiled = compile("pub fn probe() bool {\n    let q Cast2D = query2d.segment(1, true, 0.0, 0.0, 1.0, 0.0)\n    return q.hit\n}\n");
    let mut s = AgeGfxSession::new(Box::new(ShortHost));
    s.begin_tick();
    let err = run(&mut s, &compiled, "probe").expect_err("a short record must be rejected");
    assert!(
        err.contains("age-gfx.result-shape") && err.contains("Cast2D"),
        "{err}"
    );

    struct WrongTypeHost;
    impl AgeGfxHost for WrongTypeHost {
        fn dispatch(&mut self, _s: &str, _a: &[Value]) -> Result<AgeGfxReturn, AgeGfxFault> {
            // 7 个字段，但 hit 给成 i32（应为 bool）。
            Ok(AgeGfxReturn::Record(vec![
                Value::I32(1),
                Value::I32(3),
                Value::F32(0.5),
                Value::F32(1.0),
                Value::F32(2.0),
                Value::F32(0.0),
                Value::F32(-1.0),
            ]))
        }
    }
    let mut s = AgeGfxSession::new(Box::new(WrongTypeHost));
    s.begin_tick();
    let err = run(&mut s, &compiled, "probe").expect_err("wrong field type must be rejected");
    assert!(
        err.contains("age-gfx.result-type") && err.contains("hit"),
        "{err}"
    );
}

/// G4 冻结：`render2d.frame.v1(i32, str, i32, i32)` 的第二参=**图像资源名**
/// （文件路径只走 `image2d.png`），provider 原样收到、收费 1 credit。
#[test]
fn p981_gfx_host_frame_takes_a_resource_name_not_a_path() {
    const FRAME: &str =
        "pub fn frame_p() i32 {\n    return render2d.frame(1, \"atlas_main\", 0, 1)\n}\n";
    let compiled = compile(FRAME);
    let (mut s, calls) = session(0.5);
    s.begin_tick();
    assert_eq!(
        run(&mut s, &compiled, "frame_p").expect("frame executes"),
        Value::I32(2)
    );
    assert_eq!(s.credits_used(), 1);
    let log = calls.borrow();
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].0, "age.render2d.frame.v1");
    assert_eq!(
        log[0].1,
        vec![
            "i32:1".to_string(),
            "str".to_string(),
            "i32:0".to_string(),
            "i32:1".to_string()
        ]
    );
}
