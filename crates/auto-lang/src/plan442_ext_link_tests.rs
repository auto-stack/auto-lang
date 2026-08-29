//! Plan 442 A3 regression tests: `use.web` ext imports on the VM render
//! target (adapter-chain loading + platform stubs).
//!
//! ## Root cause (musk KNOWN-DEBT 028 ③ "ext link")
//!
//! The VM-render loader ignored `Stmt::UseWeb` entirely. Pure-Auto helpers
//! imported via `use.web platformInjectStyles from "…/ports/platform.at"`
//! never got a definition, and a handler calling them left an unresolved
//! CALL reloc — one missing web-platform helper killed the whole VmBridge
//! init with `LinkError: Undefined symbol`.
//!
//! The fix: `.at` ext sources load through the port-adapter chain
//! (X.at → X.vm.at → X.web.at, mirroring auto-man's target gating) and the
//! remaining TS/npm ext symbols get no-op platform stubs whose arity matches
//! the call sites (the VM's RET unwinds with `bp - n_args` — arity mismatch
//! corrupts the caller frame).
//!
//! Corpus: `test/ui/plan442_ext_link/` mirrors the musk app.at shape —
//! a root `use.web` from a port `.at` (adapter-resolved), a nested TS import
//! inside the adapter, and a direct TS composable import.

#[cfg(test)]
mod plan442_ext_link_tests {
    use auto_val::Value;

    fn locate_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan442_ext_link/src/front/app.at";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ];
        candidates.into_iter().flatten().find(|p| p.exists())
    }

    fn build() -> Option<crate::ui::dynamic::DynamicComponent> {
        crate::plan370_test_support::build_component_from_app(&locate_corpus()?)
    }

    fn state_str(dc: &crate::ui::dynamic::DynamicComponent, field: &str) -> String {
        match dc.read_state(field) {
            Ok(Value::Str(s)) => s.as_str().to_string(),
            Ok(Value::Bool(b)) => b.to_string(),
            Ok(other) => format!("{:?}", other),
            Err(e) => panic!("read_state('{}') failed: {}", field, e),
        }
    }

    /// REGRESSION: the whole component must build — before the fix,
    /// `.Init -> { platformInjectStyles(); … }` failed the link with
    /// "Undefined symbol: platformInjectStyles" and init never ran.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn ext_link_builds_and_init_runs() {
        let dc = match build() {
            Some(c) => c,
            None => {
                eprintln!("plan442: SKIPPED — corpus app.at not found");
                return;
            }
        };
        // Init ran: the handler set both fields after the adapter call.
        assert_eq!(state_str(&dc, "styled"), "true");
        assert_eq!(state_str(&dc, "adapter"), "web-adapter");
    }

    /// REGRESSION: the `.at` ext source loaded through the ADAPTER chain —
    /// `platformKind()` must come from platform.web.at ("web-adapter"), not
    /// the bare port file ("port"), proving the X.at → X.web.at gating.
    /// (Covered by the adapter assertion above; this test pins the export
    /// shape: the adapter fns exist as module exports.)
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn ext_link_adapter_fns_exported() {
        let dc = match build() {
            Some(c) => c,
            None => {
                eprintln!("plan442: SKIPPED — corpus app.at not found");
                return;
            }
        };
        let table = dc.debug_fn_table();
        let names: Vec<String> = table.iter().map(|(n, _)| n.clone()).collect();
        for required in ["platformInjectStyles", "platformKind"] {
            assert!(
                names.iter().any(|n| n.ends_with(required)),
                "adapter fn `{}` missing from exports: {:?}",
                required,
                names
            );
        }
    }

    /// REGRESSION: TS-source ext symbols get no-op platform stubs — `useT`
    /// (from composables.ts) and the adapter's nested `injectStyles` (from
    /// inject_styles.ts) must exist as stub exports so the Refresh handler
    /// (`let t = useT()`) and the adapter body link and run cleanly.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn ext_link_ts_symbols_stubbed() {
        let mut dc = match build() {
            Some(c) => c,
            None => {
                eprintln!("plan442: SKIPPED — corpus app.at not found");
                return;
            }
        };
        let table = dc.debug_fn_table();
        let names: Vec<String> = table.iter().map(|(n, _)| n.clone()).collect();
        for required in ["useT", "injectStyles"] {
            assert!(
                names.iter().any(|n| n == required),
                "stub `{}` missing from exports: {:?}",
                required,
                names
            );
        }
        // The stubbed handler runs without exploding.
        dc.on_with_input("Refresh", None);
        assert_eq!(state_str(&dc, "adapter"), "refreshed");
    }

    /// PLAN-050 T9 (C7): void ext stub must read as None — musk
    /// WorkspaceSelector 的 `.current = ws_load_current().await`（子 Init 每帧
    /// 重放）此前把 void 哨兵 Int(0) 写进 `obj = None` 模型变量，
    /// `.current != None` 恒真而 `.current.name` 落空 → rail 底部渲染裸
    /// "${currentName}"。stub 改返 Nil 后 computed 应落 else 分支。
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn plan050_void_stub_reads_as_none_in_computed() {
        fn collect_texts(
            view: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>,
            out: &mut Vec<String>,
        ) {
            use crate::ui::view::View;
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
                View::Grid { cells, .. } => {
                    for c in cells {
                        collect_texts(c, out);
                    }
                }
                _ => {}
            }
        }

        let rel = "test/ui/plan050_stub_nil/src/front/app.at";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
            Some(std::path::PathBuf::from(format!("../../{}", rel))),
        ];
        let path = candidates.into_iter().flatten().find(|p| p.exists()).expect("corpus app.at");
        let dc = crate::plan370_test_support::build_component_from_app(&path).expect("build component");

        // stub 赋值结果必须读作 Nil（此前为 Int(0) void 哨兵）。
        eprintln!("DEBUG state[current] = {:?}", dc.read_state("current"));
        eprintln!("DEBUG fn_table has ws_load_current = {:?}", dc.debug_fn_table().iter().any(|(n, _)| n.contains("ws_load_current")));
        eprintln!("DEBUG fn names: {:?}", dc.debug_fn_table().iter().map(|(n, _)| n.clone()).collect::<Vec<_>>());
        assert_eq!(dc.read_state("current").unwrap(), Value::Nil, "void stub assignment must yield Nil");

        let (view, _, _) = dc.view_with_debug_gated(false);
        let mut texts = Vec::new();
        collect_texts(&view, &mut texts);
        assert!(
            texts.iter().any(|t| t.contains("选择工作目录")),
            "computed else 分支必须上屏; got {:?}",
            texts
        );
        assert!(
            !texts.iter().any(|t| t.contains("${currentName}")),
            "裸 placeholder 不得漏出; got {:?}",
            texts
        );
        // i18n.t() 文本位：查表命中 → 文案上屏（此前提取畸形 + 求值恒空）。
        assert!(
            texts.iter().any(|t| t.contains("设置")),
            "i18n.t 查表文案必须上屏; got {:?}",
            texts
        );
        assert!(
            !texts.iter().any(|t| t.contains("$t(")),
            "$t 标记不得漏出; got {:?}",
            texts
        );
    }
}
