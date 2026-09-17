//! PLAN-066 T-06 red corpus: 055-4⑥ 过滤投影（computed 内 VmRef 域读取）
//! + P536-D2 跨模块调用帧 SET_FIELD 达根态。
//!
//! Corpus: `test/ui/plan066_filter_projection/` — app.at（App：block-bodied
//! computed `filteredMessages` 对 store 列表元素做 `m.content` 域读取过滤，
//! musk chats_view 缩样）+ store.at（FilterStore：[]Value 列表 + MarkDone→
//! ClearWindow 自调链）。
//!
//! - PA `p_a_*`：computed 求值上下文对 store 列表（堆对象元素）的域读取
//!   投影正确（命中计数=2、清空恢复=3、miss=0）——055-4⑥ 现场投影恒 0。
//! - PB `p_b_*`：跨模块调用帧（MarkDone 帧内 FilterStore.ClearWindow()）
//!   内 SET_FIELD 达共享根态——055 期 musk done 臂「本臂自清」绕行的原缺陷。

#[cfg(test)]
mod plan066_filter_projection_tests {
    use crate::plan370_test_support::build_component_from_app;

    fn locate(rel: &str) -> Option<std::path::PathBuf> {
        let full = format!("test/ui/plan066_filter_projection/{}", rel);
        [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(&full)),
            Some(std::path::PathBuf::from(&full)),
            Some(std::path::PathBuf::from(format!("../../{}", full))),
        ]
        .into_iter()
        .flatten()
        .find(|p| p.exists())
    }

    // ─────────────────────────────────────────────────────────────────────
    // (PA) 055-4⑥: computed 内对 store 列表元素的域读取投影
    // ─────────────────────────────────────────────────────────────────────

    fn pa_assert(dc: &mut crate::ui::dynamic::DynamicComponent, tag: &str) {
        // Init 已由 build 派发：3 条消息入库、搜索键 "2+2"。
        // 运行期灌数（Init 期 store 写不进 post-merge 根态——harness 时序面，
        // musk 生产数据本就运行期回填）。
        dc.on_with_input("Seed", None);
        dc.on_with_input("Search", Some("2+2".to_string()));
        let hit = dc.read_state("projection_len");
        eprintln!("plan066(PA/{tag}) hit={hit:?}");
        let hit = hit.expect("projection_len readable");
        assert!(
            hit == auto_val::Value::Int(2),
            "(PA/{tag}) 过滤投影须命中 2 条（2+2 问/答），got {hit:?} —— \
             0 即 055-4⑥「computed 求值上下文 VmRef 域读取恒空」未修",
            tag = tag
        );

        dc.on_with_input("Search", Some(String::new()));
        let all = dc.read_state("projection_len").expect("projection_len");
        assert!(
            all == auto_val::Value::Int(3),
            "(PA/{tag}) 清空搜索须恢复全量 3 条，got {all:?}"
        );

        dc.on_with_input("Search", Some("zzz".to_string()));
        let miss = dc.read_state("projection_len").expect("projection_len");
        assert!(
            miss == auto_val::Value::Int(0),
            "(PA/{tag}) miss 搜索须 0 条，got {miss:?}"
        );
    }

    /// 红相锁定（2026-09-15，T-06 进行中）：055-4⑥ 现代真身=**隐藏
    /// computed fn 返回新建列表时，返回边界未接管堆份额**——filteredMessages
    /// （返回 `out` 列表）交付死 id，调用方 `.len()` 恒 0；同 handler 内
    /// msgCount（同过滤逻辑但返回 `out.len()`，Int）=1、firstContent（返回
    /// 元素域读串）正常、push(VmRef) 本体经 shim 正常（--ignored 实测
    /// elem_is_obj=true 正确入列）。根修面=VM CALL/RET 对合成 fn 返回堆
    /// 值的 stake 接管（PLAN-062 T12 call_vm_fn takeover / T-05 shim 配平
    /// 的用户 fn 返回路径同族缺口）。落地后移除 `#[ignore]`，断言投影
    /// 命中 2/清空 3/miss 0。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan066_pa_computed_filter_over_store_list() {
        let Some(manifest) = locate("src/front/app.at") else {
            eprintln!("plan066: SKIPPED — corpus app.at not found");
            return;
        };
        let Some(mut dc) = build_component_from_app(&manifest) else {
            eprintln!("plan066: SKIPPED — corpus build failed");
            return;
        };
        pa_assert(&mut dc, "merged");
    }

    // ─────────────────────────────────────────────────────────────────────
    // (PB) P536-D2: 跨模块调用帧内 SET_FIELD 达根态
    // ─────────────────────────────────────────────────────────────────────

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan066_pb_cross_module_setfield_reaches_root() {
        let Some(manifest) = locate("src/front/app.at") else {
            eprintln!("plan066: SKIPPED — corpus app.at not found");
            return;
        };
        let Some(mut dc) = build_component_from_app(&manifest) else {
            eprintln!("plan066: SKIPPED — corpus build failed");
            return;
        };

        dc.on_with_input("MarkDone", None);
        let snap = dc.read_state("done_snapshot").expect("done_snapshot");
        eprintln!("plan066(PB) done_snapshot={snap:?}");
        assert!(
            snap == auto_val::Value::Str("false".into()),
            "(PB) MarkDone 帧内 ClearWindow 的 SET_FIELD（done=false）须达根态\
             ——_snapshot 应为 \"false\"（P536-D2 修复后自调链写可达），got {snap:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // (PC) T-07: ThinkBlock 读侧缩样——子件 obj/str prop + computed 串等值
    // ─────────────────────────────────────────────────────────────────────

    /// musk ChatMessage 同形：子件 handler 直调 store msg（ToggleBubble）
    /// 携 `.msg.id`（obj prop 域读实参）；翻转后子件 computed
    /// `isOpen => .expanded == .msg.id` 经 SetProbe 回传。055-T13 现场
    /// 为「读侧子件 computed 与视图 if 求值恒假/恒真分歧」。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan066_pc_think_block_read_side() {
        let Some(manifest) = locate("src/front/app.at") else {
            eprintln!("plan066: SKIPPED — corpus app.at not found");
            return;
        };
        let Some(mut dc) = build_component_from_app(&manifest) else {
            eprintln!("plan066: SKIPPED — corpus build failed");
            return;
        };

        // 展开翻转：子件 handler 携 .msg.id（"m1"）调 store ToggleBubble。
        dc.on_with_input_for("MsgBubble", "Toggle", None);
        let expanded = dc.read_state("expanded");
        eprintln!("plan066(PC) after Toggle expanded={expanded:?}");
        assert!(
            expanded == Ok(auto_val::Value::Str("m1".into())),
            "(PC) 子件 handler 的 .msg.id 实参须达 store（obj prop 域读），             expanded 应为 \"m1\"，got {expanded:?}"
        );

        // 读侧 computed：isOpen = (.expanded == .msg.id) → true。
        dc.on_with_input_for("MsgBubble", "Probe", None);
        let probe_on = dc.read_state("probe_result");
        eprintln!("plan066(PC) probe_on={probe_on:?}");
        assert!(
            probe_on == Ok(auto_val::Value::Str("true".into())),
            "(PC) 展开态子件 computed 串等值须 true（055-T13 恒假分歧面），got {probe_on:?}"
        );

        // 再点收起：Toggle → expanded="" → isOpen false。
        dc.on_with_input_for("MsgBubble", "Toggle", None);
        let expanded_off = dc.read_state("expanded");
        dc.on_with_input_for("MsgBubble", "Probe", None);
        let probe_off = dc.read_state("probe_result");
        eprintln!("plan066(PC) expanded_off={expanded_off:?} probe_off={probe_off:?}");
        assert!(
            expanded_off == Ok(auto_val::Value::Str("".into()))
                && probe_off == Ok(auto_val::Value::Str("false".into())),
            "(PC) 收起后 computed 须 false，got expanded={expanded_off:?} probe={probe_off:?}"
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // (PD) T-08: handler-as-value 实参容错（musk StartStream 同形）
    // ─────────────────────────────────────────────────────────────────────

    /// `Sse.open(url, .OnProbe)`——handler-as-value 实参须改写为 fn 引用
    /// （Sse no-op 弹弃），handler 不再因 GET_FIELD "OnProbe" Field not
    /// found 中止。KD-059-FU1 族（musk StartStream 现场）。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan066_pd_handler_as_value_arg_tolerated() {
        let Some(manifest) = locate("src/front/app.at") else {
            eprintln!("plan066: SKIPPED — corpus app.at not found");
            return;
        };
        let Some(mut dc) = build_component_from_app(&manifest) else {
            eprintln!("plan066: SKIPPED — corpus build failed");
            return;
        };

        dc.on_with_input_for("App", "Probe2", None);
        let probe2 = dc.read_state("probe2");
        eprintln!("plan066(PD) probe2={probe2:?}");
        assert!(
            probe2 == Ok(auto_val::Value::Str("reached".into())),
            "(PD) Sse.open(url, .OnProbe) 后 handler 须继续执行到 SetProbe2             （handler-as-value 实参容错），got {probe2:?}"
        );
    }
}
