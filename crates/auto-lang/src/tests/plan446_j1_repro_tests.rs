//! Plan 446 J1 headless repro: b3-era os-config export renders as an empty
//! shell on current master — root `Sidebar { }` child-widget reference comes
//! out as the literal fallback text `<Sidebar />` instead of expanding.
//!
//! `#[ignore]` — needs the b3check corpus (in-repo tmp-corpus/, rebuilt from
//! os-config b955004 via git archive). Run:
//!   cargo test -p auto-lang --features ui-iced plan446_j1 -- --ignored --nocapture

#[cfg(test)]
mod plan446_j1_repro {
    fn locate_b3_app() -> Option<std::path::PathBuf> {
        // 语料入库本仓 tmp-corpus/(b955004 git archive 重建,PLAN-446 L 节):
        // crates/auto-lang → 3 级上溯 = worktree 根。
        let rel = "../../tmp-corpus/b3check/auto/src/front/app.at";
        let cand = std::env::var("CARGO_MANIFEST_DIR")
            .ok()
            .map(|d| std::path::PathBuf::from(d).join(rel));
        cand.filter(|p| p.exists())
    }

    #[cfg(feature = "ui-interpreter")]
    #[test]
    #[ignore = "requires local b3check corpus (os-config 008 worktree)"]
    fn b3_child_widget_expansion() {
        let app = match locate_b3_app() {
            Some(p) => p,
            None => {
                eprintln!("plan446 J1 repro: SKIPPED — b3check corpus not found");
                return;
            }
        };
        let dc = crate::plan370_test_support::build_component_from_app(&app)
            .expect("b3 app must build headlessly (parse+codegen+link)");
        // 双路径对照:tracked 裸(view_with_debug) vs gated(带 computed/preview,
        // 运行时 renderer 同款)——J1 症状只在运行时出现,先在这里分离变量。
        let (view, _, _) = dc.view_with_debug();
        let (view_gated, _, _) = dc.view_with_debug_gated(true);
        let g = format!("{:?}", view_gated);
        let side_gated = g.matches("Search settings").count();
        eprintln!("---- gated view: sidebar input occurrences = {side_gated} (tracked-probe on) ----");
        let rendered = format!("{:?}", view);
        eprintln!("---- view dump (first 2000 chars) ----");
        eprintln!("{}", &rendered[..rendered.len().min(2000)]);
        let bare_refs: Vec<&str> = ["<Sidebar />", "<DaemonView />", "<ConfigEditor />", "<ConfigEditorVm />", "<CollectionBrowserVm />"]
            .iter()
            .filter(|r| rendered.contains(*r))
            .copied()
            .collect();
        assert!(
            bare_refs.is_empty(),
            "child-widget references rendered as bare fallback text (J1): {bare_refs:?}"
        );
        assert!(
            side_gated > 0,
            "gated(运行时同款)路径下 Sidebar 未展开(J1 复现) — gated dump: {}",
            &g[..g.len().min(1500)]
        );
        // 运行时差异最后一步:fire_init 跑过 Init(HTTP 失败 → error 态)后重建。
        let mut dc = dc;
        dc.fire_init();
        let (view_post, _, _) = dc.view_with_debug_gated(true);
        let post = format!("{:?}", view_post);
        let side_post = post.matches("Search settings").count();
        eprintln!("---- post-Init view: sidebar input occurrences = {side_post} ----");
        assert!(
            side_post > 0,
            "post-Init 视图 Sidebar 未展开(J1 精确复现!) — dump: {}",
            &post[..post.len().min(1500)]
        );
    }
}
