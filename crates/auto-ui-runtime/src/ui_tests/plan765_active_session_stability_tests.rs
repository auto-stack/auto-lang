//! PLAN-765 T-02 单测：活跃会话稳定专项（idle 内存残留根修面）。
//!
//! 1. 回填 seq 门控（F-4）：`on_http_backfill_for` 对零状态写回填撤销置脏
//!    （F1 fire_timer 语义在 HTTP 消息桥通道的延伸），恒写回填保留置脏。
//!    语料复用 `test/ui/plan062_memleak`（IdleTick=守卫零写 / BeatTick=恒写，
//!    musk PollStream deadman 形态同构——PLAN-062 soak 同源）。
//! 2. 事件名推导（F-5）：`derive_event_name` 的 `::` 截取只作用于载荷
//!    分隔符之前的事件名段——载荷体内 `::`（代码文本 `std::process` 等）
//!    不再截坏 handler 名（musk K10"长寿实例空渲染"族回归钉）。
//! 3. 预编码载荷契约（F-3）：`HttpMsgDone::split_event` 拆分 + 经
//!    `decode_payload` 往返（handler 名 + 单字符串实参）。

/// F-5：derive_event_name 契约。
mod derive_event_name_contract {
    use crate::ui::iced::renderer::derive_event_name;

    #[test]
    fn plain_name_passthrough() {
        assert_eq!(derive_event_name("PollBackfill"), "PollBackfill");
        assert_eq!(derive_event_name(".Run"), "Run");
    }

    #[test]
    fn decorated_head_truncation_kept() {
        // 路径装饰 `::` 尾段截取语义保留（无载荷形态，与旧行为零差异）。
        assert_eq!(derive_event_name("W@3::Click"), "Click");
        assert_eq!(derive_event_name(".a::b::c"), "c");
    }

    #[test]
    fn payload_suffix_passthrough() {
        let ev = format!("Loaded\u{1F}s\u{1F}{{\"ok\":true}}");
        assert_eq!(derive_event_name(&ev), ev);
    }

    #[test]
    fn double_colon_inside_payload_no_longer_truncates() {
        // PLAN-765 回归钉：载荷 body 含 `::`（如 `std::process::exit` 文本）
        // 时，旧实现按全文 rfind("::") 截断 → handler 名损坏、回填静默丢拍。
        let payload = r#"{"ok":true,"status":200,"body":"std::process::exit(1)"}"#;
        let ev = format!("PollBackfill\u{1F}s\u{1F}{payload}");
        assert_eq!(derive_event_name(&ev), ev);
    }

    #[test]
    fn decorated_head_with_payload_reassembled() {
        // 罕见形态：装饰事件名 + 载荷并存——头截断、载荷后缀原样保留。
        let ev = format!("W@3::Click{}s{}{{\"ok\":true}}", '\u{1F}', '\u{1F}');
        let want = format!("Click{}s{}{{\"ok\":true}}", '\u{1F}', '\u{1F}');
        assert_eq!(derive_event_name(&ev), want);
    }
}

/// F-4：回填 seq 门控派发（组件级契约，镜像 musk_vm_track soak 相 A/B 口径）。
#[cfg(feature = "ui-interpreter")]
mod backfill_seq_gate {
    fn locate_corpus() -> Option<std::path::PathBuf> {
        let rel = "../auto-lang/test/ui/plan062_memleak/src/front/app.at";
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

    fn build() -> Option<crate::ui::dynamic::DynamicComponent> {
        locate_corpus().and_then(|p| crate::plan370_test_support::build_component_from_app(&p))
    }

    /// 零状态写回填（IdleTick，running=false 守卫空转）→ 不置脏。
    #[test]
    fn no_op_backfill_does_not_dirty() {
        let Some(mut dc) = build() else {
            eprintln!("plan765: SKIPPED — corpus not found");
            return;
        };
        dc.clear_dirty();
        for _ in 0..20 {
            dc.on_http_backfill_for("App", "IdleTick");
        }
        assert!(
            !dc.is_dirty(),
            "no-op backfill (zero state writes) must not dirty (PLAN-765 F-4)"
        );
    }

    /// 恒写回填（BeatTick 每拍 .count+1）→ 保留置脏（有内容更新照常重建）。
    #[test]
    fn state_writing_backfill_keeps_dirty() {
        let Some(mut dc) = build() else {
            eprintln!("plan765: SKIPPED — corpus not found");
            return;
        };
        dc.clear_dirty();
        dc.on_http_backfill_for("App", "BeatTick");
        assert!(
            dc.is_dirty(),
            "state-writing backfill must dirty (content updates rebuild as before)"
        );
    }
}

/// F-3：预编码载荷契约拆分 + decode 往返。
mod preencoded_payload_contract {
    use auto_lang_core::vm::ffi::stdlib::HttpMsgDone;

    #[test]
    fn split_event_and_decode_roundtrip() {
        let payload = r#"{"ok":true,"status":200,"body":"std::process::exit(1)"}"#;
        let done = HttpMsgDone {
            widget: "ForgeStore".to_string(),
            event: format!("PollBackfill\u{1F}s\u{1F}{payload}"),
        };
        let (name, json) = done.split_event();
        assert_eq!(name, "PollBackfill");
        assert_eq!(json, payload);
        // 渲染层零拷贝直投 decode_payload（renderer 泵同式）。
        let (clean, args) = crate::ui::dynamic::decode_payload(&done.event);
        assert_eq!(clean, "PollBackfill");
        assert_eq!(args.len(), 1);
        match args.first() {
            Some(auto_val::Value::Str(s)) => assert_eq!(s.as_str(), payload),
            other => panic!("expected single Str arg, got {:?}", other),
        }
    }

    #[test]
    fn split_event_bare_name() {
        let done = HttpMsgDone {
            widget: String::new(),
            event: "Root".to_string(),
        };
        assert_eq!(done.split_event(), ("Root", ""));
    }
}
