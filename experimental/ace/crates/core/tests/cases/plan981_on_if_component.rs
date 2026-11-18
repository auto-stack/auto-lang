//! PLAN-981（on-if 组件面刀，2026-11-19 用户排程 ⑤）：**组件事件体 if/let
//! 开面**——010/005/006 语料墙（`.SetTheme(t) -> { if … }` 组件面语法拒）。
//!
//! ## 本刀交付什么、不交付什么（复查者必读）
//!
//! - 交付：`age-on-let`/`age-on-if`/`age-on-else` 两面同开（G2 时组件面裁剪
//!   不含两臂的旧合同被本刀取代）；widget 形 handler 体从 `age-view-stmt`
//!   换到**专用并集规则** `age-on-widget-stmt`（赋值+调用+let/if——006 真语
//!   料形；watch/lifecycle 等 age-view-stmt 其它消费体**不动**）。投影进
//!   `handler.stmts`（源序，Let/If 全深度）；执行/校验走 G2 共享链
//!   （`hook_stmts_lower` 消费 stmts+形参，无第二求值器）。
//! - **窗口声明（fold 前 980 须先落消费门）**：`handler_assigns` 平铺兼容面
//!   含**两支赋值**（G2 全深度收平设计，包面 AV 真值依赖它）；auto-ui 两处
//!   平铺消费（`ace_package_sfc` 脚本重放、`ace_ts_provider` 计数）**没有
//!   Let/If 门**——本刀 fold 到 lang master 前，980 必须先落
//!   `handler_stmts` Let/If 消费门（具名拒或真复放），否则 on-if 组件源经
//!   components 通道会**静默双支重放**（错状态值）。
//! - AV 真值（分支选择真跑）见 av/tests `plan981_on_if_av.rs`；直接形
//!   （`on Msg { … }`）投影由 G2 翻正用例覆盖（`plan981_g2_hooks`）。
//!
//! 红相：基线 `b574c550a` 上 widget 形与直接形 on 体 if 均解析拒（2 探针）。

use ace_core::source::age_component::{project_component_candidates, ComponentStmt};

/// 006-hero-section 真语料形（widget 形 handler 体 `if/else`）：解析+投影，
/// stmts 源序承载 If；平铺 assigns 兼容面含**两支**（窗口声明的判别面）。
#[test]
fn p981_widget_handler_if_else_projects_into_stmts() {
    let src = "node W {\n    msg { Set(int) }\n    model { var mode int = 0 }\n    on { .Set(v) -> { if v == 1 { .mode = 1 } else { .mode = 0 } } }\n    view { text \"t\" {} }\n}\n";
    let nodes = project_component_candidates(src).expect("widget-form on-body if parses");
    let h = &nodes[0].handlers[0];
    assert_eq!(h.msg, "Set");
    // 参数面：`.Set(v)` 带参 handler（#6i）。
    assert_eq!(
        h.params.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
        vec!["v"]
    );
    let [ComponentStmt::If {
        cond, then, els, ..
    }] = &h.stmts[..]
    else {
        panic!("expected one If stmt, got {:?}", h.stmts);
    };
    // 分支体语句承载（then=mode 1 / else=mode 0）。
    assert!(matches!(&then[0], ComponentStmt::Assign(a) if a.field == "mode"));
    assert!(
        matches!(&els.as_ref().expect("else branch")[0], ComponentStmt::Assign(a) if a.field == "mode")
    );
    // 条件引用形参（v）。
    assert!(
        format!("{cond:?}").contains("v"),
        "cond must reference the handler param: {cond:?}"
    );
    // 平铺兼容面：两支赋值都在（G2 全深度设计——消费方见 If 必须具名拒）。
    assert_eq!(h.assigns.len(), 2, "both branches flatten into assigns");
}

/// 006 语料的裸 `if`（无 else）形：else 缺席为 None。
#[test]
fn p981_widget_handler_bare_if_has_no_else() {
    let src = "node W {\n    msg { SetTheme(str) }\n    model { var dark_mode bool = true }\n    on { .SetTheme(t) -> { if t == \"light\" { .dark_mode = false } } }\n    view { text \"t\" {} }\n}\n";
    let nodes = project_component_candidates(src).expect("bare if parses");
    let [ComponentStmt::If { els, .. }] = &nodes[0].handlers[0].stmts[..] else {
        panic!("expected one bare If stmt");
    };
    assert!(els.is_none(), "bare if has no else");
}

/// hook-local `let` 进 widget 形 handler 体（语句源序：let 先于 if）。
#[test]
fn p981_widget_handler_let_precedes_if_in_source_order() {
    let src = "node W {\n    msg { Tick }\n    model { var mode int = 0 }\n    on { .Tick -> { let step = 1  if mode < 3 { mode = mode + step } } }\n    view { text \"t\" {} }\n}\n";
    let nodes = project_component_candidates(src).expect("let+if in widget body parses");
    let stmts = &nodes[0].handlers[0].stmts;
    let let_at = stmts
        .iter()
        .position(|s| matches!(s, ComponentStmt::Let { .. }))
        .expect("let in stmts");
    let if_at = stmts
        .iter()
        .position(|s| matches!(s, ComponentStmt::If { .. }))
        .expect("if in stmts");
    assert!(let_at < if_at, "source order preserved");
    // let 不进平铺 assigns（绑定是 hook-local，不是字段赋值）。
    assert!(
        nodes[0].handlers[0]
            .assigns
            .iter()
            .all(|a| a.field == "mode"),
        "hook-local let must not enter the flat assigns face"
    );
}

/// 负控：无 if/let 的 widget 形 handler 照旧（stmts=Assign-only、assigns 同
/// 序）——新臂不扰动既有面（判别：新臂误吞赋值则本例红）。
#[test]
fn p981_widget_handler_without_control_flow_stays_assign_only() {
    let src = "node W {\n    msg { Inc }\n    model { var n int = 0 }\n    on { .Inc -> { .n = .n + 1 } }\n    view { text \"t\" {} }\n}\n";
    let nodes = project_component_candidates(src).expect("plain handler still parses");
    let h = &nodes[0].handlers[0];
    assert!(h
        .stmts
        .iter()
        .all(|s| matches!(s, ComponentStmt::Assign(_))));
    assert_eq!(h.assigns.len(), 1);
    assert_eq!(h.assigns[0].field, "n");
}
