//! Plan 442 Phase B probe (auto-musk side verification).
//!
//! Headless link probe against the REAL auto-musk corpus (sibling checkout
//! `../../auto-musk`): builds the full musk frontend (`src/front/app.at`)
//! through the VM dynamic-component loader exactly like `--render=vm`, but
//! without the iced window. This is the reproducible form of the ad-hoc
//! "53 文件全量语料 headless 探针" used to drive the B-phase blocker list.
//!
//! `#[ignore]` — the sibling checkout is a machine-local layout assumption,
//! so this never runs in CI; run explicitly with:
//!   cargo test -p auto-lang --features ui-interpreter musk_probe -- --ignored --nocapture

#[cfg(test)]
mod plan442_musk_probe {
    fn locate_musk_app() -> Option<std::path::PathBuf> {
        // crates/auto-lang → ../../../ = autostack/ (sibling checkouts).
        let rel = "../../../auto-musk/src/front/app.at";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
        ];
        candidates.into_iter().flatten().find(|p| p.exists())
    }

    /// B5 gate: the full musk frontend must pass parse + codegen + LINK.
    /// Any remaining "Undefined symbol" / language-semantics error fails here.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    #[ignore = "requires sibling auto-musk checkout; manual Phase-B gate"]
    fn musk_full_front_end_links() {
        let app = match locate_musk_app() {
            Some(p) => p,
            None => {
                eprintln!("plan442 musk probe: SKIPPED — auto-musk not found");
                return;
            }
        };
        let dc = crate::plan370_test_support::build_component_from_app(&app)
            .expect("musk src/front/app.at must build headlessly (parse+codegen+link)");
        // Link succeeded: the component graph loaded with state wired.
        assert!(
            !dc.state_fields().is_empty(),
            "linked musk component must expose state fields"
        );
    }
}
