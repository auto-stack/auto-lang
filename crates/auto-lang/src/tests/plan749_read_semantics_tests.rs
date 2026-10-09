//! PLAN-749 T-00/T-01：VM 读语义三症状三上下文最小复现矩阵（红相先行）。
//!
//! 缺陷族（auto-musk PLAN-100 E2E，2026-10-09）：
//! - 症状1：`JSON.parse` 产物顶层 bool 读出位型垃圾（`r.ok` → -2147483648
//!   = `decode_i32(TAG_BOOL)`——TAG_BOOL 低 32 位为 0x80000000），`r.ok ==
//!   true` 间歇失效（SessionsLoaded/DetailLoaded/PollBackfill 全部间歇断）。
//! - 症状2：handler 上下文 ≥4 层深链字段读全 MISS
//!   （`resp.session.messages[1].blocks[0].tool_name`），浅层/数组长度正常。
//! - 症状3：视图模板内 fn 调用静默返空（坑①家族第三型；同 fn 在 computed
//!   上下文正常）。
//!
//! 三上下文：① 模块 fn 段（plan702 `call_fn_by_name_segment` 通道）② store
//! handler 段（`build_dynamic_component` comp 驱动，musk 原生上下文）③ 视图
//! 模板/computed（`view_with_debug_gated` 渲染树断言）。Rust 侧直读断言真值
//! + raw NanoValue 位型 dump（不依赖 .at 侧断言原语——观测面本身可能是缺陷
//! 面；plan733 同款纪律）。

#![cfg(all(test, feature = "ui-iced"))]

use crate::vm::engine::SegmentOutcome;
use crate::vm::task::AutoTask;

// ── 驱动助手（fn 上下文）──

fn p749_compile(code: &str) -> (crate::vm::engine::AutoVM, std::sync::Arc<std::sync::RwLock<String>>) {
    let (vm, stdout, _entry, _object_type) =
        crate::create_vm_from_source(code).expect("compile");
    (vm, stdout)
}

/// 段驱动 fn 并取返回栈值（raw nv——位型观察通道）。
fn p749_call_ret(
    vm: &crate::vm::engine::AutoVM,
    name: &str,
) -> Result<auto_val::NanoValue, String> {
    let mut task = AutoTask::new(0, 65536, 0);
    match vm.call_fn_by_name_segment(&mut task, name, 0) {
        SegmentOutcome::Completed(Ok(())) => Ok(task.ram.pop_nv()),
        SegmentOutcome::Completed(Err(e)) => Err(format!("runtime error: {e:?}")),
        other => Err(format!("unexpected segment outcome: {other:?}")),
    }
}

/// 段驱动并取返回串（strings 池解码）。
fn p749_call_str(
    vm: &crate::vm::engine::AutoVM,
    name: &str,
) -> Result<String, String> {
    let nv = p749_call_ret(vm, name)?;
    if auto_val::is_string(nv) {
        let idx = auto_val::decode_string(nv) as usize;
        let pool = vm.strings.read().unwrap();
        let bytes = pool
            .get(idx)
            .cloned()
            .unwrap_or_else(|| b"<string-pool-oob>".to_vec());
        drop(pool);
        Ok(String::from_utf8_lossy(&bytes).to_string())
    } else {
        Err(format!(
            "expected string return, got {}",
            p749_nv_dump("ret", nv)
        ))
    }
}

/// 位型诊断 dump：tag 判定 + 原始十六进制。
fn p749_nv_dump(label: &str, nv: auto_val::NanoValue) -> String {
    let tag = if auto_val::is_bool(nv) {
        "BOOL"
    } else if auto_val::is_i32(nv) {
        "I32"
    } else if auto_val::is_string(nv) {
        "STR"
    } else if auto_val::is_object(nv) {
        "OBJ"
    } else if auto_val::is_list(nv) {
        "LIST"
    } else if auto_val::is_null(nv) {
        "NULL"
    } else if auto_val::is_f64(nv) {
        "F64"
    } else {
        "OTHER"
    };
    let s = format!("[{label}] tag={tag} raw={nv:016x}");
    eprintln!("{s}");
    s
}

// ── 语料构造（serde_json 生成 + .at 字符串转义，杜绝手写转义错）──

fn p749_inner_json() -> String {
    serde_json::json!({
        "session": {
            "messages": [
                { "id": "m0", "blocks": [ { "tool_name": "t00" } ] },
                { "id": "m1", "blocks": [ { "tool_name": "t10" }, { "tool_name": "t11" } ] }
            ]
        }
    })
    .to_string()
}

fn p749_envelope_json() -> String {
    serde_json::json!({
        "ok": true,
        "status": 200,
        "body": p749_inner_json(),
    })
    .to_string()
}

/// Rust 串 → .at 源码内字符串字面量体（反斜杠与双引号转义）。
fn p749_at_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ═══════════════ 上下文①：模块 fn 段 ═══════════════

// ── 症状1：JSON.parse 产物顶层 bool 读语义（fn 上下文）──

/// S1-a：`r.ok == true` 必须为真（musk 观测：间歇失效）。
#[test]
fn p749_s1_bool_eq_true_minimal() {
    let code = r#"
fn probe_eq() str {
    let r = JSON.parse("{\"ok\":true,\"status\":200,\"body\":\"x\"}")
    if r.ok == true {
        return "EQ_TRUE"
    }
    return "EQ_FALSE"
}
"#;
    let (vm, _) = p749_compile(code);
    let got = p749_call_str(&vm, "probe_eq").expect("segment");
    eprintln!("[s1_eq] returned str = {got:?}");
    assert_eq!(
        got, "EQ_TRUE",
        "r.ok == true must hold for {{\"ok\":true}} envelope"
    );
}

/// S1-b：bool 位型直读——raw nv 必须是 TAG_BOOL（musk 观测 -2147483648 =
/// decode_i32(TAG_BOOL)，低 32 位 0x80000000）。
#[test]
fn p749_s1_bool_tag_raw_minimal() {
    let code = r#"
fn probe_raw() bool {
    let r = JSON.parse("{\"ok\":true,\"status\":200}")
    return r.ok
}
"#;
    let (vm, _) = p749_compile(code);
    let nv = p749_call_ret(&vm, "probe_raw").expect("segment");
    let dump = p749_nv_dump("s1_raw", nv);
    assert!(
        auto_val::is_bool(nv),
        "GET_FIELD push for bool field must be TAG_BOOL (Plan 402 口径); {dump}"
    );
    assert!(auto_val::decode_bool(nv), "bool value must decode true; {dump}");
}

/// S1-c：显示面——`r.ok` 的串化必须显示 "true"（musk 观测：-2147483648）。
#[test]
fn p749_s1_bool_display_minimal() {
    let code = r#"
fn probe_disp() str {
    let r = JSON.parse("{\"ok\":true}")
    return "ok=" + r.ok.to_string()
}
"#;
    let (vm, _) = p749_compile(code);
    let got = p749_call_str(&vm, "probe_disp").expect("segment");
    eprintln!("[s1_disp] returned str = {got:?}");
    assert_eq!(
        got, "ok=true",
        "bool display must render true, not bit-pattern int"
    );
}

/// S1-d：print 面——`print(r.ok)` 的 stdout 必须含 "true"（musk 观测垃圾的
/// 原生观测通道）。
#[test]
fn p749_s1_bool_print_minimal() {
    let code = r#"
fn probe_print() {
    let r = JSON.parse("{\"ok\":true}")
    print(r.ok)
    print("done")
}
"#;
    let (vm, stdout) = p749_compile(code);
    let _ = p749_call_ret(&vm, "probe_print").expect("segment");
    let out = stdout.read().unwrap().clone();
    eprintln!("[s1_print] stdout = {out:?}");
    assert!(
        out.contains("true") && !out.contains("-2147483648"),
        "print(bool) must render true; got: {out:?}"
    );
}

/// S1-e：f-string 内插面——`f"ok=${r.ok}"` 必须内插 "true"（musk print 垃圾
/// 的另一候选通道：f-string 内插对 bool 的显示臂）。
#[test]
fn p749_s1_bool_fstring_minimal() {
    let code = r#"
fn probe_fstr() str {
    let r = JSON.parse("{\"ok\":true}")
    return f"ok=${r.ok}"
}
"#;
    let (vm, _) = p749_compile(code);
    match p749_call_str(&vm, "probe_fstr") {
        Ok(got) => {
            eprintln!("[s1_fstr] returned str = {got:?}");
            assert_eq!(
                got, "ok=true",
                "f-string interpolation of bool must render true"
            );
        }
        Err(e) => {
            // f-string 在模块 fn 语境若不支持，登记边界（comp 面另有格）。
            eprintln!("[s1_fstr] fn-context unsupported: {e}");
        }
    }
}

// ── 症状2：深链字段读（fn 上下文，逐层单变量）──

/// S2-a：浅层对照——数组长度（musk 观测：此层工作）。
#[test]
fn p749_s2_shallow_len_minimal() {
    let code = format!(
        r#"
fn probe_len() int {{
    let resp = JSON.parse("{}")
    return resp.session.messages.length
}}
"#,
        p749_at_escape(&p749_inner_json())
    );
    let (vm, _) = p749_compile(&code);
    let nv = p749_call_ret(&vm, "probe_len").expect("segment");
    let dump = p749_nv_dump("s2_len", nv);
    assert!(auto_val::is_i32(nv), "len must be i32; {dump}");
    assert_eq!(auto_val::decode_i32(nv), 2, "messages.length == 2; {dump}");
}

/// S2-b：中深链——`resp.session.messages[1].id`（3 hop + 索引）。
#[test]
fn p749_s2_mid_chain_minimal() {
    let code = format!(
        r#"
fn probe_mid() str {{
    let resp = JSON.parse("{}")
    return resp.session.messages[1].id
}}
"#,
        p749_at_escape(&p749_inner_json())
    );
    let (vm, _) = p749_compile(&code);
    let got = p749_call_str(&vm, "probe_mid").expect("segment");
    eprintln!("[s2_mid] returned str = {got:?}");
    assert_eq!(got, "m1", "mid chain (3 hops + index) must resolve");
}

/// S2-c：全深链——`resp.session.messages[1].blocks[0].tool_name`（musk 观测：
/// 四层链全 MISS 的原形态；5 hop）。
#[test]
fn p749_s2_deep_chain_minimal() {
    let code = format!(
        r#"
fn probe_deep() str {{
    let resp = JSON.parse("{}")
    return resp.session.messages[1].blocks[0].tool_name
}}
"#,
        p749_at_escape(&p749_inner_json())
    );
    let (vm, _) = p749_compile(&code);
    let got = p749_call_str(&vm, "probe_deep").expect("segment");
    eprintln!("[s2_deep] returned str = {got:?}");
    assert_eq!(got, "t10", "5-hop deep chain must resolve to t10");
}

/// S2-d：逐层单变量定位（诊断格——修复前逐层 dump 哪层开始 MISS）。
#[test]
fn p749_s2_layer_by_layer_minimal() {
    let code = format!(
        r#"
fn probe_layers() str {{
    let resp = JSON.parse("{}")
    var s = resp.session.messages[1].id
    let msgs = resp.session.messages
    s = s + "|" + msgs[1].id
    let m1 = msgs[1]
    s = s + "|" + m1.id
    let blocks = m1.blocks
    s = s + "|" + blocks[0].tool_name
    return s
}}
"#,
        p749_at_escape(&p749_inner_json())
    );
    let (vm, _) = p749_compile(&code);
    let got = p749_call_str(&vm, "probe_layers").expect("segment");
    eprintln!("[s2_layers] returned str = {got:?}");
    assert_eq!(
        got, "m1|m1|m1|t10",
        "each intermediate binding must preserve truth"
    );
}

// ═══════════════ 上下文②③：store handler 段 + 视图/computed（comp 驱动）═══════════════

/// 语料：宿主 widget（触发器 + computed 对照面 + 视图条件 CALL 面）。
fn p749_host_src() -> String {
    format!(
        r#"
use r749_store: RStore

fn extractRunIdStr(tc str) str {{
    if tc == "" {{
        ""
    }} else {{
        "run:" + tc
    }}
}}

widget App {{
    model {{
        var tc str = "run_9"
    }}
    computed {{
        c_ok => .store.env.ok == true
        c_run => extractRunIdStr(.tc)
        c_deep => .store.deep.session.messages[1].blocks[0].tool_name
    }}
    view {{
        col {{
            button "run" {{ onclick: .Run }}
            text f"s1eq=${{.store.s1_eq}}"
            text f"s1disp=${{.store.s1_disp}}"
            text f"s1fstr=${{.store.s1_fstr}}"
            text f"s2len=${{.store.s2_len}}"
            text f"s2mid=${{.store.s2_mid}}"
            text f"s2deep=${{.store.s2_deep}}"
            text f"c_ok=${{.c_ok}}"
            text f"c_run=${{.c_run}}"
            text f"c_deep=${{.c_deep}}"
            text extractRunIdStr(.tc)
            text f"vfstr=${{.store.env.ok}}"
            text f"vdeep=${{.store.deep.session.messages[1].blocks[0].tool_name}}"
            if extractRunIdStr(.tc) != "" {{
                text "VIEW_CALL_NONEMPTY"
            }} else {{
                text "VIEW_CALL_EMPTY"
            }}
            if .c_run != "" {{
                text "COMPUTED_CALL_NONEMPTY"
            }} else {{
                text "COMPUTED_CALL_EMPTY"
            }}
            if .store.env.ok {{
                text "VIEW_ENV_OK_TRUE"
            }} else {{
                text "VIEW_ENV_OK_FALSE"
            }}
            // 注：`.store.deep.session.messages[1]...` 形态的索引在条件位是
            // parser 语法级不支持（f-string/text 内插位允许）——边界登记于
            // 计划 §9，本格用无索引成员链钉条件通道语义。
            if .store.env.status == 200 {{
                text "VIEW_STATUS_EQ"
            }} else {{
                text "VIEW_STATUS_MISS"
            }}
        }}
    }}
    msg Msg {{ Run }}
    on {{
        .Run -> {{ store.Run() }}
    }}
}}
"#
    )
}

/// 语料：store（handler 段读语义格 + 对象入字段后的存储可达读）。
fn p749_store_src() -> String {
    format!(
        r#"
store RStore {{
    model {{
        var env Value = {{}}
        var deep Value = {{}}
        var s1_eq bool = false
        var s1_disp str = "unset"
        var s1_fstr str = "unset"
        var s2_len int = -1
        var s2_mid str = "unset"
        var s2_deep str = "unset"
        var s2_deep_nil str = "unset"
    }}
    msg Msg {{ Run }}
    on {{
        .Run -> {{
            let env = JSON.parse("{}")
            .s1_eq = (env.ok == true)
            .s1_disp = env.ok.to_string()
            .s1_fstr = f"ok=${{env.ok}}"
            .env = env
            let resp = JSON.parse(env.body)
            .deep = resp
            .s2_len = resp.session.messages.length
            .s2_mid = resp.session.messages[1].id
            .s2_deep = resp.session.messages[1].blocks[0].tool_name
            .s2_deep_nil = resp.session.messages[1].blocks[0].tool_name ?? "NILFALLBACK"
        }}
    }}
}}
"#,
        p749_at_escape(&p749_envelope_json())
    )
}

fn build_p749() -> crate::ui::dynamic::DynamicComponent {
    let dir = tempfile::TempDir::new().unwrap();
    let base = dir.path();
    let host = p749_host_src();
    let store = p749_store_src();
    for (rel, src) in [("host.at", host.as_str()), ("r749_store.at", store.as_str())] {
        std::fs::write(base.join(rel), src).unwrap();
    }
    let host_path: std::path::PathBuf = base.join("host.at").to_path_buf();
    let host_src = std::fs::read_to_string(&host_path).unwrap();
    let comp = crate::build_dynamic_component(&host_src, Some(&host_path.to_string_lossy()))
        .expect("p749 fixture component builds");
    std::mem::forget(dir);
    comp
}

fn p749_state_str(
    comp: &crate::ui::dynamic::DynamicComponent,
    field: &str,
) -> String {
    match comp.read_state(field) {
        Ok(auto_val::Value::Str(s)) => s.as_str().to_string(),
        other => format!("<{field} not str: {other:?}>"),
    }
}

fn p749_state_bool(
    comp: &crate::ui::dynamic::DynamicComponent,
    field: &str,
) -> bool {
    matches!(
        comp.read_state(field),
        Ok(auto_val::Value::Bool(true))
    )
}

/// 三上下文主矩阵：handler 段读格 + 视图/computed 渲染格。
#[test]
fn p749_three_context_matrix() {
    let mut comp = build_p749();
    let _ = comp.view_with_debug_gated(false);
    comp.drive_scheduler_to_quiescence(10_000);
    comp.on_with_input_for("App", "Run", None);
    comp.on_with_input_for("RStore", "Run", None);
    comp.drive_scheduler_to_quiescence(10_000);
    let _ = comp.view_with_debug_gated(false);
    comp.drive_scheduler_to_quiescence(10_000);

    // 诊断面：全字段实落形态。
    for (k, v) in comp.read_all_state_materialized() {
        eprintln!("[p749-mat] {k} = {v:?}");
    }
    let debug = {
        let (view, _, _) = comp.view_with_debug_gated(false);
        format!("{view:?}")
    };
    eprintln!(
        "[p749-view] {}",
        debug.chars().take(1600).collect::<String>()
    );

    // ── 症状1：handler 上下文（musk 原生上下文）──
    assert!(
        p749_state_bool(&comp, "s1_eq"),
        "handler: env.ok == true must hold (musk SessionsLoaded 同构)"
    );
    assert_eq!(
        p749_state_str(&comp, "s1_disp"),
        "true",
        "handler: env.ok display must be \"true\""
    );
    assert_eq!(
        p749_state_str(&comp, "s1_fstr"),
        "ok=true",
        "handler: f-string interpolation of bool must render true"
    );

    // ── 症状2：handler 上下文深链（musk DetailLoaded 同构）──
    assert_eq!(
        p749_state_str(&comp, "s2_mid"),
        "m1",
        "handler: mid chain must resolve"
    );
    assert_eq!(
        p749_state_str(&comp, "s2_deep"),
        "t10",
        "handler: 5-hop deep chain must resolve (musk 观测全 MISS 处)"
    );
    assert_eq!(
        p749_state_str(&comp, "s2_deep_nil"),
        "t10",
        "handler: ?? fallback must see non-nil truth (not forced fallback)"
    );
    let s2_len = match comp.read_state("s2_len") {
        Ok(auto_val::Value::Int(n)) => n,
        other => panic!("s2_len not int: {other:?}"),
    };
    assert_eq!(s2_len, 2, "handler: messages.length == 2");

    // ── 症状1/2 视图 TEXT 位与 computed 位（存储可达对象读的显示通道）──
    assert!(
        debug.contains("c_ok=true"),
        "computed: .store.env.ok == true must render true, got: {}",
        debug.chars().take(900).collect::<String>()
    );
    assert!(
        debug.contains("vfstr=true"),
        "view text f-string over .store.env.ok must render true, got: {}",
        debug.chars().take(1200).collect::<String>()
    );
    assert!(
        debug.contains("c_deep=t10"),
        "computed deep chain over .store.deep must render t10, got: {}",
        debug.chars().take(1600).collect::<String>()
    );
    assert!(
        debug.contains("vdeep=t10"),
        "view text f-string deep chain must render t10, got: {}",
        debug.chars().take(1600).collect::<String>()
    );
}

/// 症状3 + 视图条件通道族（红相主格——修复前红、修复后绿）：
/// - VIEW_CALL：视图条件位 fn 调用（musk chat_message.at:339 同构）；
/// - COMPUTED_CALL：比较臂单段 computed 引用（`.c_run != ""`）；
/// - VIEW_ENV_OK：truthy 臂 store 前缀多段路径（`.store.env.ok`）；
/// - VIEW_DEEP：比较臂 store 前缀成员链（`.store.env.status == 200`）。
#[test]
fn p749_view_condition_faces() {
    let mut comp = build_p749();
    let _ = comp.view_with_debug_gated(false);
    comp.drive_scheduler_to_quiescence(10_000);
    comp.on_with_input_for("App", "Run", None);
    comp.on_with_input_for("RStore", "Run", None);
    comp.drive_scheduler_to_quiescence(10_000);
    let _ = comp.view_with_debug_gated(false);
    comp.drive_scheduler_to_quiescence(10_000);
    let debug = {
        let (view, _, _) = comp.view_with_debug_gated(false);
        format!("{view:?}")
    };
    eprintln!(
        "[p749-view-cond] {}",
        debug.chars().take(1800).collect::<String>()
    );

    assert!(
        debug.contains("VIEW_CALL_NONEMPTY") && !debug.contains("VIEW_CALL_EMPTY"),
        "view condition `if extractRunIdStr(.tc) != \"\"` must take non-empty \
         branch (musk chat_message.at:339 同构——修复前恒走 EMPTY 臂)",
    );
    assert!(
        debug.contains("COMPUTED_CALL_NONEMPTY")
            && !debug.contains("COMPUTED_CALL_EMPTY"),
        "view condition comparing single-segment computed (`.c_run != \"\"`) \
         must take non-empty branch"
    );
    assert!(
        debug.contains("VIEW_ENV_OK_TRUE") && !debug.contains("VIEW_ENV_OK_FALSE"),
        "view truthy `.store.env.ok` must take true branch (PLAN-081 存储可达\
         嵌套对象读家族)",
    );
    assert!(
        debug.contains("VIEW_STATUS_EQ") && !debug.contains("VIEW_STATUS_MISS"),
        "view condition member chain `.store.env.status == 200` must take \
         equal branch",
    );
}
