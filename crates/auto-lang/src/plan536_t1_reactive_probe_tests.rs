//! PLAN-536 T1 复现探针：timer 写 state → 视图失效 → 视图重建全链实证。
//!
//! musk KD 059-FU1 三题的最小复现面（timer handler 写 state + 视图绑定该
//! state），语料复用 `test/ui/plan051_timer/`（根 widget LocalTick + store
//! 门控 PollTick，视图 f-string 同时绑定 `.local_count` 与 `.store.poll_count`）。
//!
//! 链路分界（计划 T1 要求逐环定位）：
//! 1. 失效广播臂：fire_timer → handler 执行 → `component.dirty` 置位；
//! 2. 消费臂：重建视图（view_with_debug_gated）读 state → 文本更新；
//! 3. update 周期契约：renderer 每拍 clear_dirty → 派发 → 尾部
//!    `is_dirty() → view_dirty`（renderer.rs update 尾）——本探针按同序
//!    模拟，验证契约成立。

#![cfg(test)]

#[cfg(test)]
mod plan536_t1_reactive_probe_tests {
    use crate::ui::view::View;

    fn locate_corpus() -> Option<std::path::PathBuf> {
        // 指向 app.at（非 pac.at——build_component_from_app 从 manifest
        // 内容里找根 WidgetDecl，裸 manifest 会静默返回 None）。
        let rel = "test/ui/plan051_timer/src/front/app.at";
        [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(format!("../../{}", rel))),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ]
        .into_iter()
        .flatten()
        .find(|p| p.exists())
    }

    fn collect_texts(view: &View<crate::ui::interpreter::DynamicMessage>, out: &mut Vec<String>) {
        match view {
            View::Text { content, .. } => out.push(content.clone()),
            View::Button { label, content, .. } => {
                out.push(label.clone());
                if let Some(c) = content.as_ref() {
                    collect_texts(c, out);
                }
            }
            View::Row { children, .. } | View::Column { children, .. } => {
                for c in children {
                    collect_texts(c, out);
                }
            }
            View::Container { child, .. } | View::Scrollable { child, .. } => collect_texts(child, out),
            View::Overlay { base, content, .. } => {
                collect_texts(base, out);
                collect_texts(content, out);
            }
            View::Grid { cells, .. } => {
                for c in cells {
                    collect_texts(c, out);
                }
            }
            _ => {}
        }
    }

    fn rendered_texts(dc: &crate::ui::dynamic::DynamicComponent) -> Vec<String> {
        let (view, _, _) = dc.view_with_debug_gated(false);
        let mut texts = Vec::new();
        collect_texts(&view, &mut texts);
        texts
    }

    /// 环节 1+2（根 widget 计时器）：timer 写 state → dirty 置位（失效广播）
    /// → 重建视图文本更新（消费）。任一臂断即题 1 在该环节复现。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn p536_t1_root_timer_write_reaches_rebuilt_view() {
        let corpus = match locate_corpus() {
            Some(p) => p,
            None => {
                eprintln!("p536 T1: SKIPPED — corpus not found");
                return;
            }
        };
        let mut dc = crate::plan370_test_support::build_component_from_app(&corpus)
            .expect("build plan051_timer component");

        // 初渲染基线
        let base = rendered_texts(&dc);
        assert!(
            base.iter().any(|t| t.contains("local=0")),
            "baseline must show local=0; got {base:?}"
        );

        // renderer update 周期契约：清 dirty → 派发 timer 拍
        dc.clear_dirty();
        assert!(dc.is_timer_entry("App", "LocalTick"), "timer entry registered");
        assert!(dc.fire_timer("App", "LocalTick"), "ungated tick dispatches");
        assert!(dc.is_dirty(), "ARM-1 失效广播: timer handler 写 state 后 dirty 必须置位");
        assert_eq!(dc.read_state("local_count"), Ok(auto_val::Value::Int(1)));

        // 消费臂：按 dirty 重建的视图文本必须更新
        let after = rendered_texts(&dc);
        assert!(
            after.iter().any(|t| t.contains("local=1")),
            "ARM-2 消费: rebuilt view must show local=1; got {after:?}"
        );
    }

    /// store 计时器（musk PollStream 同形）：store handler 写 store 字段 →
    /// 视图经 `.store.poll_count` 跨模块绑定消费。musk 题身的最近似面。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn p536_t1_store_timer_write_reaches_rebuilt_view() {
        let corpus = match locate_corpus() {
            Some(p) => p,
            None => {
                eprintln!("p536 T1: SKIPPED — corpus not found");
                return;
            }
        };
        let mut dc = crate::plan370_test_support::build_component_from_app(&corpus)
            .expect("build plan051_timer component");

        dc.clear_dirty();
        dc.on_with_input_for("TickerStore", "SetGate", Some("true".to_string()));
        assert!(dc.fire_timer("TickerStore", "PollTick"), "gated tick dispatches after SetGate");
        assert!(dc.is_dirty(), "ARM-1 失效广播: store timer handler 写 state 后 dirty 必须置位");
        assert_eq!(dc.read_state("poll_count"), Ok(auto_val::Value::Int(1)));

        let after = rendered_texts(&dc);
        assert!(
            after.iter().any(|t| t.contains("poll=1")),
            "ARM-2 消费: rebuilt view must show poll=1 via .store binding; got {after:?}"
        );
    }

    /// 待澄清 #1 定案：`when` 门是**派发前过滤**（fire_timer 内求值，假丢弃
    /// 本拍），订阅层消息（[UI_EVENT] 可见）无条件到达 update——即日志里
    /// when=false 仍见 UI_EVENT 与门语义不矛盾。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn p536_t1_when_gate_is_pre_dispatch_filter() {
        let corpus = match locate_corpus() {
            Some(p) => p,
            None => {
                eprintln!("p536 T1: SKIPPED — corpus not found");
                return;
            }
        };
        let mut dc = crate::plan370_test_support::build_component_from_app(&corpus)
            .expect("build plan051_timer component");

        // 门关：条目在表（订阅在发、消息可达 update），但 fire_timer 丢弃本拍
        assert!(dc.is_timer_entry("TickerStore", "PollTick"), "entry 在表=订阅层照发");
        dc.clear_dirty();
        assert!(!dc.fire_timer("TickerStore", "PollTick"), "gate closed must drop tick");
        assert!(!dc.is_dirty(), "被门拦的拍不得置 dirty");
        assert_eq!(dc.read_state("poll_count"), Ok(auto_val::Value::Int(0)));
    }

    /// 边界：handler 不存在/执行失败时 dirty 不得置位（防伪失效风暴）；
    /// 并验证连续多拍累积（musk 18 拍场景的累计口径）。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn p536_t1_dirty_contract_under_repeated_ticks() {
        let corpus = match locate_corpus() {
            Some(p) => p,
            None => {
                eprintln!("p536 T1: SKIPPED — corpus not found");
                return;
            }
        };
        let mut dc = crate::plan370_test_support::build_component_from_app(&corpus)
            .expect("build plan051_timer component");

        // 18 拍连打（musk PollStream 实测拍数）
        for i in 0..18 {
            dc.clear_dirty();
            assert!(dc.fire_timer("App", "LocalTick"), "tick {i} must dispatch");
            assert!(dc.is_dirty(), "tick {i}: dirty must re-arm every tick");
        }
        assert_eq!(dc.read_state("local_count"), Ok(auto_val::Value::Int(18)));
        let after = rendered_texts(&dc);
        assert!(
            after.iter().any(|t| t.contains("local=18")),
            "rebuilt view must show local=18; got {after:?}"
        );

        // 未注册事件：fire_timer 返回 false（条目不存在），不置 dirty
        dc.clear_dirty();
        assert!(!dc.fire_timer("App", "NoSuchEvent"));
        assert!(!dc.is_dirty(), "no-op dispatch must not dirty");
    }

    // ── 双臂语料（test/ui/plan536_reactive/）：直绑 vs 子件 prop ──────────

    fn locate_reactive_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan536_reactive/src/front/app.at";
        [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(format!("../../{}", rel))),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ]
        .into_iter()
        .flatten()
        .find(|p| p.exists())
    }

    /// musk 同形面：timer 写 store 字段后重建视图，直绑臂
    /// （`.store.title` f-string）与子件 prop 臂（ChatBubble.bubble_title）
    /// 是否同拍更新。题 4 的"prop 构建期快照、子件读侧恒旧"若在 builder
    /// 层成立，此探针的子件臂必红；若双臂同绿，则快照冻结只发生在
    /// iced Element 缓存层（画布未重建），题 4 的活性问题与题 1 同根。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn p536_t1_store_timer_updates_child_prop_and_direct_binding() {
        let corpus = match locate_reactive_corpus() {
            Some(p) => p,
            None => {
                eprintln!("p536 T1: SKIPPED — reactive corpus not found");
                return;
            }
        };
        let mut dc = crate::plan370_test_support::build_component_from_app(&corpus)
            .expect("build plan536_reactive component");

        // 基线：两臂都显示 initial
        let base = rendered_texts(&dc);
        assert!(
            base.iter().any(|t| t.contains("direct=initial")),
            "baseline direct arm must show initial; got {base:?}"
        );
        assert!(
            base.iter().any(|t| t.contains("bubble=initial")),
            "baseline child-prop arm must show initial; got {base:?}"
        );

        // store 计时器一拍：title → poll-1
        dc.clear_dirty();
        assert!(dc.fire_timer("ChatStore", "PollTick"), "store tick dispatches");
        assert!(dc.is_dirty(), "store handler write must dirty");
        assert_eq!(dc.read_state("title"), Ok(auto_val::Value::str("poll-1")));

        // 重建后两臂同拍更新
        let after = rendered_texts(&dc);
        assert!(
            after.iter().any(|t| t.contains("direct=poll-1")),
            "direct binding arm must update on rebuild; got {after:?}"
        );
        assert!(
            after.iter().any(|t| t.contains("bubble=poll-1")),
            "child-prop arm must re-resolve props on rebuild (题4 快照语义定界); got {after:?}"
        );
    }

    /// T2 失效根修（题1 musk 根因定案）：handler **执行中崩**（副作用已落盘
    /// 后 RuntimeError，musk "Invalid object ID: 0" 同形状）时 dirty 也必须
    /// 置位——否则 store 已拿到数据而画布永冻（059-FU1 实录）。RED 先行。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn p536_t2_handler_error_after_side_effects_still_invalidates() {
        let corpus = match locate_reactive_corpus() {
            Some(p) => p,
            None => {
                eprintln!("p536 T2: SKIPPED — reactive corpus not found");
                return;
            }
        };
        let mut dc = crate::plan370_test_support::build_component_from_app(&corpus)
            .expect("build plan536_reactive component");

        dc.clear_dirty();
        // 副作用（title="written"）先落，再 SET_FIELD on None 崩（Err 收场）
        dc.on_with_input_for("ChatStore", "Boom", None);
        assert_eq!(
            dc.read_state("title"),
            Ok(auto_val::Value::str("written")),
            "side effect before the crash must have landed"
        );
        assert!(
            dc.is_dirty(),
            "ARM-1 根修: handler Err(mid-body) 后 dirty 必须置位,否则画布永冻"
        );

        // 对照组：handler 缺失（HandlerNotFound,无副作用）不置 dirty
        dc.clear_dirty();
        dc.on_with_input_for("ChatStore", "NoSuchHandler", None);
        assert!(!dc.is_dirty(), "HandlerNotFound (no side effects) must not dirty");
    }

    /// T3 Init 重入收敛（题 2）：子件 Init=挂载语义,只随首渲染执行一次,
    /// 不随脏重建帧重放。RED：当前每渲染帧重放(437 v1 近似,债务在案),
    /// musk 单会话期子件 Init 1.6 万+次重放即此——副作用(ForgeStore.Init
    /// → LoadSessionList 打后端)随帧重入。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn p536_t3_child_init_fires_once_not_per_rebuild() {
        let corpus = match locate_reactive_corpus() {
            Some(p) => p,
            None => {
                eprintln!("p536 T3: SKIPPED — reactive corpus not found");
                return;
            }
        };
        let mut dc = crate::plan370_test_support::build_component_from_app(&corpus)
            .expect("build plan536_reactive component");

        // 首次渲染 = 挂载：ChatBubble.Init 恰好一次
        let _ = dc.view_with_debug_gated(false);
        assert_eq!(
            dc.read_state("bubble_init_count"),
            Ok(auto_val::Value::Int(1)),
            "first render must fire child Init exactly once"
        );

        // 后续 6 次脏重建帧：Init 不再重放
        for _ in 0..6 {
            let _ = dc.view_with_debug_gated(false);
        }
        assert_eq!(
            dc.read_state("bubble_init_count"),
            Ok(auto_val::Value::Int(1)),
            "rebuild frames must NOT replay child Init (题2 重入风暴根修)"
        );
    }

    // ── T6 absolute 定位原语（题5）：row/button 载体全链 hoist ────────────

    fn locate_absolute_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan536_absolute/src/front/app.at";
        [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(format!("../../{}", rel))),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ]
        .into_iter()
        .flatten()
        .find(|p| p.exists())
    }

    fn collect_views<'a>(
        view: &'a View<crate::ui::interpreter::DynamicMessage>,
        out: &mut Vec<&'a View<crate::ui::interpreter::DynamicMessage>>,
    ) {
        out.push(view);
        match view {
            View::Row { children, .. } | View::Column { children, .. } => {
                for c in children {
                    collect_views(c, out);
                }
            }
            View::Container { child, .. } | View::Scrollable { child, .. } => collect_views(child, out),
            View::Overlay { base, content, .. } => {
                collect_views(base, out);
                collect_views(content, out);
            }
            View::Button { content: Some(c), .. } => collect_views(c, out),
            _ => {}
        }
    }

    /// T6①②: absolute + 偏移 + z 的悬浮元素在 **row 父容器**内也必须
    /// hoist 为 Overlay(锚=父 bounds),不再挤压兄弟流内布局(musk 会话卡 ×
    /// 的行内形状;此前仅 convert_column 有 hoist 臂)。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn p536_t6_row_parent_hoists_absolute_child() {
        let corpus = match locate_absolute_corpus() {
            Some(p) => p,
            None => {
                eprintln!("p536 T6: SKIPPED — absolute corpus not found");
                return;
            }
        };
        let dc = crate::plan370_test_support::build_component_from_app(&corpus)
            .expect("build plan536_absolute component");
        let (view, _, _) = dc.view_with_debug_gated(false);

        let mut all = Vec::new();
        collect_views(&view, &mut all);
        let overlays: Vec<_> = all.iter().filter_map(|v| match v {
            View::Overlay { base, content, position } => Some((base, content, position)),
            _ => None,
        }).collect();

        assert!(
            overlays.iter().any(|(base, content, pos)| {
                matches!(base.as_ref(), View::Row { .. })
                    && matches!(content.as_ref(), View::Column { .. })
                    && pos.right == Some(8.0)
                    && pos.top == Some(8.0)
            }),
            "row 父容器的 absolute(z) 子件必须 hoist 为 Overlay(right/top 偏移); overlays={}",
            overlays.len()
        );
        // 悬浮层文本在树上仍可见(MCP 快照口径)
        let texts = rendered_texts(&dc);
        assert!(texts.iter().any(|t| t.contains("float-from-row")), "float text must render; got {texts:?}");
        assert!(texts.iter().any(|t| t.contains("flow-text")), "flow sibling must stay; got {texts:?}");
    }

    /// T6②: absolute 载体不止 col/row——button(× 删除钮)/container 一类
    /// 自带样式的叶子同样可 hoist。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn p536_t6_button_carrier_hoists() {
        let corpus = match locate_absolute_corpus() {
            Some(p) => p,
            None => {
                eprintln!("p536 T6: SKIPPED — absolute corpus not found");
                return;
            }
        };
        let dc = crate::plan370_test_support::build_component_from_app(&corpus)
            .expect("build plan536_absolute component");
        let (view, _, _) = dc.view_with_debug_gated(false);

        let mut all = Vec::new();
        collect_views(&view, &mut all);
        let overlays: Vec<_> = all.iter().filter_map(|v| match v {
            View::Overlay { base, content, position } => Some((base, content, position)),
            _ => None,
        }).collect();

        assert!(
            overlays.iter().any(|(_base, content, pos)| {
                matches!(content.as_ref(), View::Button { .. })
                    && pos.right == Some(6.0)
                    && pos.top == Some(8.0)
            }),
            "button 载体的 absolute(z) 必须 hoist(right=6/top=8); overlays={}",
            overlays.len()
        );
    }

    /// T6/T7 重测改判(2026-09-04): button 结构子件内的 absolute **保持流内**
    /// ——Button{content:Overlay} 在 iced 画布渲染为空壳(musk 会话列表
    /// 实测:名称/× 全失,仅剩描边),hoist 臂已撤销。× 的悬浮由消费方
    /// 结构承接(musk:× 移出卡片 button,外层 col 的 col-arm hoist)。
    /// renderer 侧 Button{content:Overlay} 渲染根修后本测试可再翻转。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn p536_t6_button_inner_absolute_hoists_in_card() {
        let corpus = match locate_absolute_corpus() {
            Some(p) => p,
            None => {
                eprintln!("p536 T6: SKIPPED — absolute corpus not found");
                return;
            }
        };
        let dc = crate::plan370_test_support::build_component_from_app(&corpus)
            .expect("build plan536_absolute component");
        let (view, _, _) = dc.view_with_debug_gated(false);

        let mut all = Vec::new();
        collect_views(&view, &mut all);
        // 找到含 "session-name" 文本的 Overlay:base 是按钮内容 Column(含
        // session-name),content 是悬浮的 × Button。
        // 卡片 button(内容含 session-name)的**直接内容**不得是 Overlay——
        // 该形态画布空壳(musk 会话列表实测),button 臂已撤销。
        // 注:col 下的 ×(第一个结构)由 col 臂合法 hoist,不受此限。
        let card_content_is_overlay = all.iter().any(|v| match v {
            View::Button { label, content: Some(c), .. } if label.is_empty() => {
                matches!(c.as_ref(), View::Overlay { .. })
            }
            _ => false,
        });
        assert!(
            !card_content_is_overlay,
            "卡片 button 的内容不得为 Overlay(画布空壳回归)"
        );
    }

    /// T6 语义定界(负面): absolute 无 z 的分层技巧(p051-min-ta textarea
    /// 叠加族)保留流内——不 hoist,防 inset-0 背景层翻到内容之上。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn p536_t6_absolute_without_z_stays_in_flow() {
        let corpus = match locate_absolute_corpus() {
            Some(p) => p,
            None => {
                eprintln!("p536 T6: SKIPPED — absolute corpus not found");
                return;
            }
        };
        let dc = crate::plan370_test_support::build_component_from_app(&corpus)
            .expect("build plan536_absolute component");
        let (view, _, _) = dc.view_with_debug_gated(false);

        let mut all = Vec::new();
        collect_views(&view, &mut all);
        // layer-text(inset-0 无 z)必须仍作为 col 的流内子件存在
        let flow_layer = all.iter().any(|v| matches!(v,
            View::Text { content, .. } if content.contains("layer-text")));
        assert!(flow_layer, "无 z 的 absolute 分层文本必须留在流内");
        // 且不在任何 Overlay 的 content 侧
        let in_overlay = all.iter().any(|v| match v {
            View::Overlay { content, .. } => {
                matches!(**content, View::Text { ref content, .. } if content.contains("layer-text"))
            }
            _ => false,
        });
        assert!(!in_overlay, "无 z 不得 hoist");
    }


fn locate_modal_corpus() -> Option<std::path::PathBuf> {
    let rel = "test/ui/plan536_modal/src/front/app.at";
    [
        std::env::var("CARGO_MANIFEST_DIR")
            .ok()
            .map(|d| std::path::PathBuf::from(d).join(format!("../../{}", rel))),
        Some(std::path::PathBuf::from(rel)),
        Some(std::path::PathBuf::from(format!("../../{}", rel))),
    ]
    .into_iter()
    .flatten()
    .find(|p| p.exists())
}

fn find_modal(
    view: &View<crate::ui::interpreter::DynamicMessage>,
    out: &mut Vec<bool>,
) {
    if let View::Popover { open, placement, .. } = view {
        if matches!(placement, crate::ui::view::PopoverPlacement::Modal) {
            out.push(*open);
        }
    }
    match view {
        View::Row { children, .. } | View::Column { children, .. } => {
            for c in children {
                find_modal(c, out);
            }
        }
        View::Container { child, .. } | View::Scrollable { child, .. } => find_modal(child, out),
        View::Overlay { base, content, .. } => {
            find_modal(base, out);
            find_modal(content, out);
        }
        View::Button { content: Some(c), .. } => find_modal(c, out),
        _ => {}
    }
}

/// T8(题6)：unknown-tag 子件（fallback Column 路径）包 alert-dialog 家族根,
/// `open` 绑定根态——翻转后渲染树必须出 open=true 的 Modal。
/// 勘误:此前误报"引擎缺陷"实为测试派发名错位（语料 msg=FlipRoot 而测试
/// 派发 "Flip"→HandlerNotFound→写入不发生）;修正后本测试锁定 fallback
/// 路径 open 绑定在带子件工程中依然成立。
#[cfg(feature = "ui-interpreter")]
#[test]
fn p536_t8_unknown_tag_fallback_resolves_open_binding() {
    let corpus = match locate_modal_corpus() {
        Some(p) => p,
        None => {
            eprintln!("p536 T8: SKIPPED — modal corpus not found");
            return;
        }
    };
    let mut dc = crate::plan370_test_support::build_component_from_app(&corpus)
        .expect("build plan536_modal component");

    // 初渲染：Modal 在树但 open=false（受控闭合态）
    let (view, _, _) = dc.view_with_debug_gated(false);
    let mut modals = Vec::new();
    find_modal(&view, &mut modals);
    assert!(
        modals.contains(&false),
        "closed alert-dialog must render as Modal(open=false); modals={modals:?}"
    );

    // 翻转 open → 重建 → Modal open=true
    dc.on_with_input_for("App", "FlipRoot", None);
    assert_eq!(
        dc.read_state("root_open"),
        Ok(auto_val::Value::Bool(true)),
        "flip write must land on root state"
    );
    let (view2, _, _) = dc.view_with_debug_gated(false);
    let mut modals2 = Vec::new();
    find_modal(&view2, &mut modals2);
    assert!(
        modals2.contains(&true),
        "open 翻转后 fallback 路径必须解析 open 绑定出 Modal(open=true); modals={modals2:?}"
    );
}

/// T8(musk chats_view 同形)：alert-dialog 在**子件视图根部**,open 绑定
/// 子件模型字段（统一根态播种）——翻转后 Modal(open=true) 须在树。
#[cfg(feature = "ui-interpreter")]
#[test]
fn p536_t8_child_widget_root_alert_dialog_resolves_open() {
    let corpus = match locate_modal_corpus() {
        Some(p) => p,
        None => {
            eprintln!("p536 T8: SKIPPED — modal corpus not found");
            return;
        }
    };
    let mut dc = crate::plan370_test_support::build_component_from_app(&corpus)
        .expect("build plan536_modal component");

    dc.on_with_input_for("ChatsLike", "Flip", None);
    let (view, _, _) = dc.view_with_debug_gated(false);
    let mut modals = Vec::new();
    find_modal(&view, &mut modals);
    assert!(
        modals.contains(&true),
        "子件视图根部的 alert-dialog 翻转后必须出 Modal(open=true); modals={modals:?}"
    );
}
}
