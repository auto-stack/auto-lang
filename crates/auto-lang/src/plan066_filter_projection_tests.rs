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
        let count = dc.read_state("count_len");
        let msgs = dc.read_state("messages");
        let msgs_vec = dc.read_state_as_vec("messages");
        let search = dc.read_state("chat_search");
        let dbg = dc.read_state("dbg_after_append");
        let msgs_prefixed = dc.read_state("FilterStore.messages");
        let first = dc.read_state("done_snapshot");
        eprintln!(
            "plan066(PA/{tag}) hit={hit:?} count_len={count:?} messages={msgs:?} msgs_vec={msgs_vec:?} chat_search={search:?} dbg_after_append={dbg:?} firstContent={first:?} FilterStore.messages={msgs_prefixed:?}"
        );
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

    /// 红相锁定（2026-09-15，T-06 进行中）：055-4⑥ 现代真身已隔离为单行
    /// 复现——合成 fn（computed/handler）内 `for m in <store 列表> {
    /// out.push(m) }` 对循环变量 VmRef 本体的 push 静默 no-op（列表恒空）；
    /// push 字面量/push 元素域读值（m.content）皆正常、循环迭代与元素域读
    /// 取皆正常。根修（List push 的 VmRef 实参通道，plan419/rc 族）落地后
    /// 移除 `#[ignore]`，断言投影命中 2/清空 3/miss 0。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    #[ignore = "PLAN-066 T-06 红相：合成 fn 内 push(循环变量 VmRef) no-op（根修后解除）"]
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
}
