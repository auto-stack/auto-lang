//! PLAN-051 C7 regression tests: `timer { ... }` 声明块——widget/store 的
//! 周期计时器 DSL（条目头 = msg 变体名；`every_ms` 周期；`when` 门控条件假
//! 不派发不停止底层计时）。
//!
//! ## 设计（PLAN-051 详细设计 C7）
//!
//! ```auto
//! widget Clock {
//!     msg { Tick }
//!     timer { Tick (every_ms: 1000) }
//!     on { .Tick -> { … } }
//! }
//! ```
//!
//! - 条目头须在本 widget/store 的 msg{} 声明——解析期校验（沿 Plan 451
//!   actions handler 校验承诺）。
//! - vue 轨：widget = onMounted setInterval / onUnmounted clearInterval；
//!   store = 模块级 interval（应用生命周期）。
//! - VM 轨：AppTickRecipe 泛化（widget+event 打标订阅）→ 既有 handler 泉；
//!   `when` 门控在派发前对合并根 state 求值（假 → 丢弃本拍）。
//!
//! Corpus: `test/ui/plan051_timer/`（根 widget 计时器 + store 门控计时器）。

#[cfg(test)]
mod plan051_timer_tests {
    // ── parser：widget timer 块 ────────────────────────────────────────────

    fn parse_ui(code: &str) -> Result<crate::ast::Code, crate::error::AutoError> {
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(code).with_session(session);
        parser.parse()
    }

    /// widget timer 块解析：条目 event/every_ms/when 三元组形状。
    #[test]
    fn plan051_timer_parses_widget_block() {
        let code = r#"
widget Clock {
    msg { Tick, Guarded }
    model { var streaming bool = false }
    timer {
        Tick (every_ms: 1000)
        Guarded (every_ms: 500, when: .streaming)
    }
    view { col { text "x" } }
    on { .Tick -> { } .Guarded -> { } }
}
"#;
        let ast = parse_ui(code).expect("parse ok");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d),
            _ => None,
        }).expect("widget decl");
        let timer = decl.timer.as_ref().expect("timer block present");
        assert_eq!(timer.entries.len(), 2);
        assert_eq!(timer.entries[0].event.as_str(), "Tick");
        assert_eq!(timer.entries[0].every_ms, 1000);
        assert!(timer.entries[0].when.is_none());
        assert_eq!(timer.entries[1].event.as_str(), "Guarded");
        assert_eq!(timer.entries[1].every_ms, 500);
        assert_eq!(timer.entries[1].when.as_deref(), Some(".streaming"));
    }

    /// store timer 块解析（StoreDecl 同语法）。
    #[test]
    fn plan051_timer_parses_store_block() {
        let code = r#"
store TickerStore {
    msg { PollTick }
    model { var n int = 0 }
    timer { PollTick (every_ms: 50, when: .gate_open) }
    on { .PollTick -> { .n = .n + 1 } }
}
"#;
        let ast = parse_ui(code).expect("parse ok");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::StoreDecl(d) => Some(d),
            _ => None,
        }).expect("store decl");
        let timer = decl.timer.as_ref().expect("timer block present");
        assert_eq!(timer.entries.len(), 1);
        assert_eq!(timer.entries[0].event.as_str(), "PollTick");
        assert_eq!(timer.entries[0].every_ms, 50);
        assert_eq!(timer.entries[0].when.as_deref(), Some(".gate_open"));
    }

    /// 校验承诺：条目头未在 msg{} 声明 → 解析期报错（点名符号）。
    #[test]
    fn plan051_timer_rejects_undeclared_msg_variant() {
        let code = r#"
widget Clock {
    msg { Tick }
    timer { NotDeclared (every_ms: 100) }
    view { col { text "x" } }
    on { .Tick -> { } }
}
"#;
        let err = parse_ui(code).expect_err("must reject undeclared timer event");
        let msg = format!("{:?}", err);
        assert!(
            msg.contains("NotDeclared") && msg.to_lowercase().contains("timer"),
            "error should name the symbol and the timer block: {msg}"
        );
    }

    // ── vue codegen：widget SFC + store composable ─────────────────────────

    fn locate_corpus(rel: &str) -> Option<std::path::PathBuf> {
        [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ]
        .into_iter()
        .flatten()
        .find(|p| p.exists())
    }

    /// 根 widget（corpus app.at）提取 → SFC：interval 建立/清理 + when 门控。
    #[cfg(feature = "ui")]
    #[test]
    fn plan051_timer_vue_emits_interval_and_cleanup() {
        let path = locate_corpus("test/ui/plan051_timer/src/front/app.at")
            .expect("corpus app.at");
        let code = std::fs::read_to_string(&path).unwrap();
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(code.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::WidgetDecl(d) => Some(d.clone()),
            _ => None,
        }).expect("widget decl");
        let widget = crate::aura::extract::extract_widget_from_decl(&decl)
            .expect("extract");
        assert_eq!(widget.timers.len(), 1, "root timer extracted");
        let mut gen = crate::ui_gen::vue::VueGenerator::new();
        let sfc = gen.generate_sfc(&widget).expect("generate SFC");
        assert!(sfc.contains("setInterval"), "interval started: \n{sfc}");
        assert!(sfc.contains("clearInterval"), "interval cleared on unmount");
        assert!(sfc.contains("onUnmounted"), "unmount hook present");
        assert!(sfc.contains("40"), "period emitted");
    }

    /// store（corpus ticker_store.at）提取 → composable：模块级 interval +
    /// when 门控。
    #[cfg(feature = "ui")]
    #[test]
    fn plan051_timer_vue_store_composable_emits_interval() {
        let path = locate_corpus("test/ui/plan051_timer/src/front/ticker_store.at")
            .expect("corpus store");
        let code = std::fs::read_to_string(&path).unwrap();
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(code.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast.stmts.iter().find_map(|s| match s {
            crate::ast::Stmt::StoreDecl(d) => Some(d.clone()),
            _ => None,
        }).expect("store decl");
        let store = crate::aura::extract::extract_store_from_decl(&decl)
            .expect("extract store");
        assert_eq!(store.timers.len(), 1, "store timer extracted");
        assert_eq!(store.timers[0].every_ms, 50);
        let out = crate::ui_gen::vue::VueGenerator::generate_store_composable(&store);
        assert!(out.contains("setInterval"), "module interval: \n{out}");
        assert!(out.contains("50"), "period emitted");
    }

    // ── VM 轨：条目收集 + 派发 + 门控 ──────────────────────────────────────

    /// 组件构建后 timer_entries 收集（root widget + store-as-child 双源）。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan051_timer_vm_collects_entries() {
        let corpus = locate_corpus("test/ui/plan051_timer/pac.at")
            .expect("corpus pac.at");
        let dc = match crate::plan370_test_support::build_component_from_app(&corpus) {
            Some(c) => c,
            None => {
                eprintln!("plan051: SKIPPED — corpus not found");
                return;
            }
        };
        let entries = dc.timer_entries();
        let has = |w: &str, e: &str, ms: u64| {
            entries.iter().any(|t| t.widget == w && t.event == e && t.every_ms == ms)
        };
        assert!(has("App", "LocalTick", 40), "root widget entry: {entries:?}");
        assert!(has("TickerStore", "PollTick", 50), "store entry: {entries:?}");
    }

    /// 派发：timer 事件经既有 handler 泉触发（root widget + store 各一）。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan051_timer_vm_dispatch_fires_handlers() {
        let corpus = locate_corpus("test/ui/plan051_timer/pac.at")
            .expect("corpus pac.at");
        let mut dc = match crate::plan370_test_support::build_component_from_app(&corpus) {
            Some(c) => c,
            None => { eprintln!("plan051: SKIPPED"); return; }
        };
        // root widget 计时器
        dc.on_with_input_for("App", "LocalTick", None);
        let local = dc.read_state("local_count").expect("local_count");
        assert_eq!(local, auto_val::Value::Int(1), "LocalTick fired once");
        // store 计时器（store → 无视图 child WidgetDecl 同泵）
        dc.on_with_input_for("TickerStore", "PollTick", None);
        let poll = dc.read_state("poll_count");
        assert_eq!(poll, Ok(auto_val::Value::Int(1)), "PollTick fired once");
    }

    /// 门控：when 条件假 → fire_timer 丢弃本拍（计数不动）；条件真 → 派发。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan051_timer_vm_when_guard_blocks_and_passes() {
        let corpus = locate_corpus("test/ui/plan051_timer/pac.at")
            .expect("corpus pac.at");
        let mut dc = match crate::plan370_test_support::build_component_from_app(&corpus) {
            Some(c) => c,
            None => { eprintln!("plan051: SKIPPED"); return; }
        };
        // gate_open 默认 false → 门控拦截
        assert!(!dc.fire_timer("TickerStore", "PollTick"), "guard blocks");
        assert_eq!(dc.read_state("poll_count"), Ok(auto_val::Value::Int(0)));
        // 开闸 → 派发
        dc.on_with_input_for("TickerStore", "SetGate", Some("true".to_string()));
        assert!(dc.fire_timer("TickerStore", "PollTick"), "guard passes");
        assert_eq!(dc.read_state("poll_count"), Ok(auto_val::Value::Int(1)));
        // 无门控条目恒派发
        assert!(dc.fire_timer("App", "LocalTick"));
        assert_eq!(dc.read_state("local_count"), Ok(auto_val::Value::Int(1)));
    }
}
