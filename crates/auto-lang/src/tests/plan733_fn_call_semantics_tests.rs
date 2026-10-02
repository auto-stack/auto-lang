//! PLAN-733 T-01: VM 运行时模块级 fn 调用语义最小复现矩阵。
//!
//! 缺陷族（auto-musk PLAN-097 baseline 附页③，2026-10-02）：VM 渲染目标下
//! store handler 域调用模块级 fn（同文件内联与跨模块 import 同症；观测
//! 形态为多参含列表首参）不抛错、返回空列表。已证可用形态 = handler 域
//! 1~2 参（obj/Value/str/int 参型）跨模块调用。
//!
//! 本矩阵把调用形态按 参位数 × 参型 × 同文件/跨模块 逐格落地，handler 域
//! 逐格写状态字段，Rust 侧直读断言真值（不依赖 .at 侧断言原语——观测面
//! 本身可能是缺陷面）。

#![cfg(feature = "ui-iced")]

use std::path::PathBuf;

// ── 语料（plan632 build_embedded 同款 tempfile 布局）──

/// 同文件模块级 fn 族 + store。六参列表首参形 = musk canvasProgressRows
/// 同构（内联实验 b2285ac）；二参列表首参 = canvasSourceRows 同构（已证
/// 可用对照格）。
const FCS_STORE: &str = r#"
use fcs_helpers: sixCross, twoCross, sixStr

fn zeroLocal() list {
    var rows = []
    rows.push({ key: "z" })
    rows
}

fn oneLocal(s str) str {
    s + "!"
}

fn twoLocal(lines list, hl int) list {
    var rows = []
    var n int = 0
    for l in lines { n = n + 1 }
    rows.push({ key: "n", n: n })
    rows.push({ key: "hl", n: hl })
    rows
}

fn threeLocal(w int, h int, fit bool) str {
    if fit {
        "fit:" + w.to_string() + "x" + h.to_string()
    } else {
        "raw:" + w.to_string() + "x" + h.to_string()
    }
}

fn sixLocal(messages list, cv_state str, cv_seq int, cv_app str, cv_gen int, cv_error str) list {
    var rows = []
    if cv_state == "running" {
        rows.push({ key: "run", state: "running", tool: cv_app })
    }
    var cnt int = 0
    for m in messages { cnt = cnt + 1 }
    rows.push({ key: "seq", state: cv_error, tool: cnt.to_string() + ":" + cv_seq.to_string() + ":" + cv_gen.to_string() })
    rows
}

store FcsStore {
    model {
        var msgs list = [{ k: "tool" }, { k: "run" }]
        var f_zero list = []
        var f_one str = "unset"
        var f_two list = []
        var f_three str = "unset"
        var f_six list = []
        var f_sixx list = []
        var f_twox list = []
        var f_err str = "unset"
        var f_has bool = false
        // 消费探针格：handler 域对 fn 结果的消费运算真值。
        var c_len_direct int = -1        // zeroLocal().length
        var c_len_local int = -1         // 局部绑定后 .length
        var c_loop int = -1              // for-in 直接迭代计数
        var c_loop_local int = -1        // 局部绑定后 for-in 计数
        var c_eq_empty bool = true       // 结果 == [] 判空
        var c_len_field int = -1         // 落字段后再读 .length
        var c_loop_field int = -1        // 落字段后 for-in 计数
        // 跨轮次格：Write 轮写列表参数入字段，Read 轮 for-in 读回计数
        //（musk 缺陷族定位格：rbn=0 形态）。
        var cross_msgs list = []
        var cross_n int = -1
        var cross_len int = -1
        var cross_first str = "unset"
        // 鉴别格：Write 轮各形态写入 + 字符串标记（区分写丢失 vs 整态复位）。
        var w_marker str = "init"
        var w_literal list = []
        var w_local list = []
        var w_fn list = []
        var w_readback_same str = "unset"
        var w_param str = "init"
        var w_param_n int = -1
        // timer Tick 面（probe Poll 同构）。
        var tick_n int = -1
        var tick_len int = -1
        var ticks int = 0
    }
    timer {
        Tick (every_ms: 50)
    }

    msg Msg { Run, Write(list), Read, WriteFn, WriteForms, Tick }
    on {
        .Write(msgs list) -> {
            .w_param = "ran"
            var pn int = 0
            for m in msgs { pn = pn + 1 }
            .w_param_n = pn
            .cross_msgs = msgs
        }

        // timer Poll 形（probe 同构）：每拍重读 .cross_msgs，独立于 Read 轮。
        .Tick -> {
            var n int = 0
            for m in .cross_msgs { n = n + 1 }
            .tick_n = n
            .tick_len = .cross_msgs.length
            .ticks = .ticks + 1
        }

        .WriteForms -> {
            .w_marker = "wrote"
            .w_literal = [{ key: "L" }]
            var local = []
            local.push({ key: "P" })
            .w_local = local
            .w_fn = zeroLocal()
            var same = .w_fn
            var ns int = 0
            for r in same { ns = ns + 1 }
            .w_readback_same = "n" + ns.to_string()
        }

        .WriteFn -> {
            .cross_msgs = twoLocal([7, 8, 9], 1)
        }

        .Read -> {
            var n int = 0
            for m in .cross_msgs { n = n + 1 }
            .cross_n = n
            .cross_len = .cross_msgs.length
            var first = .cross_msgs[0]
            .cross_first = first.key ?? "(none)"
        }

        .Run -> {
            try {
                .f_zero = zeroLocal()
                .f_one = oneLocal("x")
                .f_two = twoLocal([10, 20], 3)
                .f_three = threeLocal(640, 480, true)
                .f_six = sixLocal(.msgs, "running", 5, "app", 2, "")
                .f_sixx = sixCross(.msgs, "running", 5, "app", 2, "")
                .f_twox = twoCross([10, 20], 3)
                .f_err = ""
                .f_has = true
            } catch {
                .f_err = "caught"
            }
            // 消费探针（每格独立 try——单格毒化不遮蔽其余格读数）。
            try { .c_len_direct = zeroLocal().length } catch { .c_len_direct = -2 }
            try {
                var lz = zeroLocal()
                .c_len_local = lz.length
            } catch { .c_len_local = -2 }
            try {
                var n int = 0
                for r in sixLocal(.msgs, "running", 5, "app", 2, "") { n = n + 1 }
                .c_loop = n
            } catch { .c_loop = -2 }
            try {
                var ls = sixLocal(.msgs, "running", 5, "app", 2, "")
                var n2 int = 0
                for r in ls { n2 = n2 + 1 }
                .c_loop_local = n2
            } catch { .c_loop_local = -2 }
            try { .c_eq_empty = zeroLocal() == [] } catch { .c_eq_empty = true }
            try { .c_len_field = .f_two.length } catch { .c_len_field = -2 }
            try {
                var n3 int = 0
                for r in .f_two { n3 = n3 + 1 }
                .c_loop_field = n3
            } catch { .c_loop_field = -2 }
        }
    }
}
"#;

/// 跨模块 fn 族（musk canvas_helpers.at 同构）。
const FCS_HELPERS: &str = r#"
pub fn sixCross(messages list, cv_state str, cv_seq int, cv_app str, cv_gen int, cv_error str) list {
    var rows = []
    if cv_state == "running" {
        rows.push({ key: "run", state: "running", tool: cv_app })
    }
    var cnt int = 0
    for m in messages { cnt = cnt + 1 }
    rows.push({ key: "seq", state: cv_error, tool: cnt.to_string() + ":" + cv_seq.to_string() + ":" + cv_gen.to_string() })
    rows
}

pub fn twoCross(lines list, hl int) list {
    var rows = []
    var n int = 0
    for l in lines { n = n + 1 }
    rows.push({ key: "n", n: n })
    rows.push({ key: "hl", n: hl })
    rows
}

pub fn sixStr(messages list, cv_state str, cv_seq int, cv_app str, cv_gen int, cv_error str) str {
    var cnt int = 0
    for m in messages { cnt = cnt + 1 }
    cv_state + ":" + cnt.to_string() + ":" + cv_seq.to_string() + ":" + cv_app + ":" + cv_gen.to_string() + ":" + cv_error
}
"#;

/// 宿主 widget：触发器 + computed 面（跨模块 fn 调用，str 返回）。
/// Init 链 = musk 同构形态：模型字段读作调用实参（store.ProgInput(.msgs)
/// 的值通道——prop/字段读 → arg → msg param → 状态字段）。
const HOST: &str = r#"
use fcs_store: FcsStore
use fcs_helpers: sixStr

widget App {
    model {
        var push_seed list = [{ key: "a" }, { key: "b" }]
    }
    computed {
        cc => sixStr([{ k: "m" }], "running", 9, "hostapp", 4, "")
        cc2 => sixStr(.store.msgs, "running", 9, "hostapp", 4, "")
        plain => "P" + 9.to_string()
    }
    view {
        col {
            text f"gate=${.store.f_has}"
            text f"one=${.store.f_one}"
            text f"three=${.store.f_three}"
            text f"err=${.store.f_err}"
            text f"cc=${.cc}"
            text f"pc=${.plain}"
            text f"cc2=${.cc2}"
            text .cc
            text .plain
            button "run" { onclick: .RunAll }
        }
    }
    msg Msg { RunAll }
    on {
        .Init -> { store.Write(.push_seed) }
        .RunAll -> { store.Run() }
    }
}
"#;

fn build_p733() -> crate::ui::dynamic::DynamicComponent {
    let dir = tempfile::TempDir::new().unwrap();
    let base = dir.path();
    for (rel, src) in [
        ("host.at", HOST),
        ("fcs_store.at", FCS_STORE),
        ("fcs_helpers.at", FCS_HELPERS),
    ] {
        let p = base.join(rel);
        std::fs::write(&p, src).unwrap();
    }
    let host_path: PathBuf = base.join("host.at").to_path_buf();
    let host_src = std::fs::read_to_string(&host_path).unwrap();
    // TempDir 在 build 期间必须存活：模块解析按宿主路径相对展开。
    let comp = crate::build_dynamic_component(&host_src, Some(&host_path.to_string_lossy()))
        .expect("p733 fixture component builds");
    // 保持 temp 目录存活至断言完成（泄漏到测试进程尾清理即可）。
    std::mem::forget(dir);
    comp
}

fn list_len(comp: &crate::ui::dynamic::DynamicComponent, field: &str) -> usize {
    // 状态列表字段的持有约定 = 堆引用（VmRef）——物化读（read_all_state_
    // materialized 的文档口径：无堆访问的消费方按此展开 for-in 源）。
    match comp
        .read_all_state_materialized()
        .get(field)
        .cloned()
        .unwrap_or(auto_val::Value::Nil)
    {
        auto_val::Value::Array(a) => a.len(),
        other => panic!("{field} not list: {other:?}"),
    }
}

fn state_str(comp: &crate::ui::dynamic::DynamicComponent, field: &str) -> String {
    match comp.read_state(field).unwrap_or(auto_val::Value::Nil) {
        auto_val::Value::Str(s) => s.as_str().to_string(),
        other => panic!("{field} not str: {other:?}"),
    }
}

fn fmt_val(v: &auto_val::Value) -> String {
    let mut s = format!("{v:?}");
    if s.len() > 120 {
        s.truncate(120);
        s.push('…');
    }
    s
}

/// T-01 矩阵主断言：handler 域全形态真值返回（修复后绿；修复前多参格红）。
#[test]
fn p733_store_handler_module_fn_call_matrix() {
    let mut comp = build_p733();
    // 首帧渲染（挂载/Init 泵）驱动至静默，隔离挂载噪声。
    let _ = comp.view_with_debug();
    comp.drive_scheduler_to_quiescence(10_000);

    // 派发 App.RunAll → store.Run()（widget→store 调用链），再直发
    // store handler 兜底（两路同 handler fn，链路覆盖面更宽）。
    comp.on_with_input_for("App", "RunAll", None);
    comp.on_with_input_for("FcsStore", "Run", None);
    comp.drive_scheduler_to_quiescence(10_000);

    // 诊断面：全字段实落形态（VmRef=未物化引用 / 真值 / 空）。
    for (k, v) in comp.read_all_state() {
        eprintln!("[p733] {k} = {v:?}");
    }
    for (k, v) in comp.read_all_state_materialized() {
        eprintln!("[p733-mat] {k} = {}", fmt_val(&v));
    }

    let err = state_str(&comp, "f_err");
    let has = matches!(comp.read_state("f_has"), Ok(auto_val::Value::Bool(true)));
    // 无抛错 + handler 走完（musk 观测①：不抛错返空——err 空而 has 真说明
    // 调用本身"成功"，空值来自返回路径）。
    assert!(has, "Run handler completed; f_err={err}");
    assert_eq!(err, "", "no caught error in Run handler");

    // 消费探针格（musk 观测②：handler 域消费 fn 结果按空处理——定位
    // 具体运算格）。-2 = 该格抛错（消费原语本身缺陷）。
    for (field, expected) in [
        ("c_len_direct", 1i32),
        ("c_len_local", 1),
        ("c_loop", 2),
        ("c_loop_local", 2),
        ("c_len_field", 2),
        ("c_loop_field", 2),
    ] {
        let v = match comp.read_state(field).unwrap_or(auto_val::Value::Nil) {
            auto_val::Value::Int(i) => i,
            other => panic!("{field} not int: {other:?}"),
        };
        assert_eq!(
            v, expected,
            "{field}: handler-side consumption of fn list result"
        );
    }
    let eq_empty = matches!(comp.read_state("c_eq_empty"), Ok(auto_val::Value::Bool(b)) if b);
    assert!(
        !eq_empty,
        "c_eq_empty: zeroLocal() == [] must be false (non-empty result)"
    );

    // ── 跨轮次读写格（musk 缺陷族定位：param 列表 → 字段 → 下轮 for-in）──
    // 鉴别轮：WriteForms 各形态写 + 字符串标记，Read 轮读回。
    comp.on_with_input_for("FcsStore", "WriteForms", None);
    comp.drive_scheduler_to_quiescence(10_000);
    comp.on_with_input_for("FcsStore", "Read", None);
    comp.drive_scheduler_to_quiescence(10_000);
    eprintln!(
        "[p733-forms] marker={} same={} | literal={:?} local={:?} fn={:?}",
        state_str(&comp, "w_marker"),
        state_str(&comp, "w_readback_same"),
        comp.read_all_state_materialized()
            .get("w_literal")
            .map(fmt_val),
        comp.read_all_state_materialized()
            .get("w_local")
            .map(fmt_val),
        comp.read_all_state_materialized().get("w_fn").map(fmt_val),
    );

    // 轮次 1：Write（参数列表入字段）——经 App.Init 链（首帧视图构建期已
    // 派发 store.Write(.push_seed)，musk ProgInput 同构通道）。轮次 2：Read。
    assert_eq!(
        state_str(&comp, "w_param"),
        "ran",
        "Write handler must run via App.Init chain (w_param marker)"
    );
    assert_eq!(
        match comp.read_state("w_param_n").unwrap_or(auto_val::Value::Nil) {
            auto_val::Value::Int(i) => i,
            other => panic!("w_param_n not int: {other:?}"),
        },
        2,
        "param list arrival count (probe input=3 same cell)"
    );
    comp.drive_scheduler_to_quiescence(10_000);
    // 写后立即物化读：字段此刻的真实内容。
    let after_write = comp
        .read_all_state_materialized()
        .get("cross_msgs")
        .cloned()
        .unwrap_or(auto_val::Value::Nil);
    eprintln!("[p733] cross_msgs after Write = {}", fmt_val(&after_write));
    comp.on_with_input_for("FcsStore", "Read", None);
    comp.drive_scheduler_to_quiescence(10_000);
    let cross_n = match comp.read_state("cross_n").unwrap_or(auto_val::Value::Nil) {
        auto_val::Value::Int(i) => i,
        other => panic!("cross_n not int: {other:?}"),
    };
    let cross_len = match comp.read_state("cross_len").unwrap_or(auto_val::Value::Nil) {
        auto_val::Value::Int(i) => i,
        other => panic!("cross_len not int: {other:?}"),
    };
    let cross_first = state_str(&comp, "cross_first");
    eprintln!("[p733] cross: n={cross_n} len={cross_len} first={cross_first}");
    assert_eq!(
        cross_n, 2,
        "cross-run for-in over list field written from msg param (musk rbn cell)"
    );
    assert_eq!(cross_len, 2, "cross-run .length over list field");
    assert_eq!(cross_first, "a", "cross-run element member read");

    // 轮次 3/4：fn 结果入字段 → 下轮读回（WriteFn 形）。
    comp.on_with_input_for("FcsStore", "WriteFn", None);
    comp.drive_scheduler_to_quiescence(10_000);
    comp.on_with_input_for("FcsStore", "Read", None);
    comp.drive_scheduler_to_quiescence(10_000);
    let cross_n2 = match comp.read_state("cross_n").unwrap_or(auto_val::Value::Nil) {
        auto_val::Value::Int(i) => i,
        other => panic!("cross_n(fn) not int: {other:?}"),
    };
    assert_eq!(
        cross_n2, 2,
        "cross-run for-in over list field written from fn result"
    );

    // timer Tick 面（probe Poll 同构）：确定性直发 timer（fire_timer）+
    // 帧视图重建，复刻"挂载写入 → 数拍后消费"的时间窗。
    for _ in 0..3 {
        assert!(comp.fire_timer("FcsStore", "Tick"), "timer Tick fires");
        comp.drive_scheduler_to_quiescence(1_000);
        let _ = comp.view_with_debug();
    }
    let tick_n = match comp.read_state("tick_n").unwrap_or(auto_val::Value::Nil) {
        auto_val::Value::Int(i) => i,
        other => panic!("tick_n not int: {other:?}"),
    };
    let tick_len = match comp.read_state("tick_len").unwrap_or(auto_val::Value::Nil) {
        auto_val::Value::Int(i) => i,
        other => panic!("tick_len not int: {other:?}"),
    };
    let ticks = match comp.read_state("ticks").unwrap_or(auto_val::Value::Nil) {
        auto_val::Value::Int(i) => i,
        other => panic!("ticks not int: {other:?}"),
    };
    eprintln!("[p733] ticks={ticks} tick_n={tick_n} tick_len={tick_len}");
    assert!(ticks >= 1, "timer Tick fired at least once (got {ticks})");
    assert_eq!(
        tick_n, 2,
        "timer-driven cross-run for-in over list field (probe rbn cell)"
    );
    assert_eq!(
        tick_len, 2,
        "timer-driven cross-run .length over list field"
    );

    // 对照格（已证可用形态）。
    assert_eq!(list_len(&comp, "f_zero"), 1, "0-param same-file");
    assert_eq!(state_str(&comp, "f_one"), "x!", "1-param str same-file");
    assert_eq!(list_len(&comp, "f_two"), 2, "2-param (list,int) same-file");
    assert_eq!(
        state_str(&comp, "f_three"),
        "fit:640x480",
        "3-param same-file"
    );
    assert_eq!(
        list_len(&comp, "f_twox"),
        2,
        "2-param (list,int) cross-module"
    );

    // 缺陷格（musk 观测形态）：六参列表首参，同文件与跨模块。
    assert_eq!(
        list_len(&comp, "f_six"),
        2,
        "6-param (list-first) same-file: expected [run,seq] rows"
    );
    assert_eq!(
        list_len(&comp, "f_sixx"),
        2,
        "6-param (list-first) cross-module: expected [run,seq] rows"
    );
}

/// T-01 computed 面探针：computed 内跨模块 fn 调用（str 返回）真值渲染。
#[test]
fn p733_computed_fn_call_face() {
    let mut comp = build_p733();
    let _ = comp.view_with_debug_gated(false);
    comp.drive_scheduler_to_quiescence(10_000);
    // 懒挂载/异步泵驱动后再取终态帧（PLAN-711 pending demand 首帧骨架）。
    let _ = comp.view_with_debug_gated(false);
    comp.drive_scheduler_to_quiescence(10_000);
    let debug = {
        let (view, _, _) = comp.view_with_debug_gated(false);
        format!("{view:?}")
    };

    // T-02 修复面直钉：call_vm_fn 列表实参栈编码（H2 统一约定 encode_object；
    // 修复前 encode_list/TAG_LIST 使 fn 内 for-in 恒零迭代）。VmRef 实参
    // 与 Rust Array 实参两形都断真值。
    let msgs_ref = comp
        .read_state("msgs")
        .expect("store list field readable (VmRef form)");
    let scalars = [
        auto_val::Value::str("running"),
        auto_val::Value::Int(9),
        auto_val::Value::str("hostapp"),
        auto_val::Value::Int(4),
        auto_val::Value::str(""),
    ];
    let expect_true = "running:2:9:hostapp:4:";
    {
        let mut args_ref = vec![msgs_ref.clone()];
        args_ref.extend(scalars.clone());
        let out = comp
            .bridge_call_for_test("sixStr", &args_ref)
            .expect("call_vm_fn with VmRef list arg");
        assert_eq!(
            out,
            auto_val::Value::str(expect_true),
            "VmRef list arg must iterate 2 elements"
        );
    }
    {
        let mut arr = auto_val::Array::new();
        for k in ["a", "b"] {
            let mut o = auto_val::Obj::new();
            o.set("key", auto_val::Value::str(k));
            arr.push(auto_val::Value::obj(o));
        }
        let mut args_arr = vec![auto_val::Value::Array(arr)];
        args_arr.extend(scalars);
        let out = comp
            .bridge_call_for_test("sixStr", &args_arr)
            .expect("call_vm_fn with Array list arg");
        assert_eq!(
            out,
            auto_val::Value::str(expect_true),
            "Array list arg must iterate 2 elements"
        );
    }
    // computed cc = sixStr([{k:"m"}], "running", 9, "hostapp", 4, "")
    // 期望真值 "running:1:9:hostapp:4:"。musk 观测 = 恒空（use.web.fn/use.web
    // 两形态）——本格断言普通 use 导入形态；use.web 形态异根性由 T-01 结论
    // 登记边界。
    // f-string 面：fn 调用 computed 与纯表达式 computed 对照。
    assert!(
        debug.contains("pc=P9"),
        "plain computed via f-string must interpolate, got: {}",
        debug.chars().take(900).collect::<String>()
    );
    assert!(
        debug.contains("cc=running:1:9:hostapp:4:"),
        "fn-call computed via f-string must render true value, got: {}",
        debug.chars().take(900).collect::<String>()
    );
    // 状态引用实参形（musk canvasProgressRows 同形——实参为 store 字段读）。
    assert!(
        debug.contains("cc2=running:2:9:hostapp:4:"),
        "fn-call computed with state-ref args must render true value, got: {}",
        debug.chars().take(1100).collect::<String>()
    );
    // 直读面（plan632 F1 已证形态）。
    assert!(
        debug.contains("\"running:1:9:hostapp:4:\""),
        "fn-call computed direct text binding must render true value, got: {}",
        debug.chars().take(900).collect::<String>()
    );
}
