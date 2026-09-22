//! Plan 442 A2 regression tests: legacy store facade in the VM render target.
//!
//! ## Root cause (musk KNOWN-DEBT 028 ③)
//!
//! auto-musk sources consume stores via the legacy form `use store: AuthStore`
//! (module name literally "store"). The VM-render loader resolved modules by
//! name → file (`store` → `store.at`), which never exists — the StoreDecl
//! lives in a sibling file (`auth_store.at`). Resolution failure was a silent
//! `continue`, so the store context stayed empty: handlers referencing
//! `store.X` failed with "Undefined variable: store" (surfaced only as a
//! [HANDLER-CODEGEN] warning, leaving poisoned half-compiled bytecode), and
//! views rendered their else-branches against missing state fields.
//!
//! The fix (lib.rs): on resolution failure of the literal "store" module,
//! locate the StoreDecl file by naming convention (snake_case store name) or
//! a bounded directory scan, and feed it through `collect_module_imports` —
//! the existing StoreDecl → view-less child WidgetDecl conversion then merges
//! the store's fields and handlers into the single VM module. The same
//! fallback covers transitive `use store:` deps of child widgets.
//!
//! Corpus: `test/ui/plan442_store_facade/` mirrors the musk app.at shape
//! (`use store: AuthStore` + `store.Init()` cross-store handler calls +
//! no-dot `store.authenticated` view references), including the store file's
//! own `use auth_util` dep to exercise dep collection through the facade.

#[cfg(test)]
mod plan442_store_facade_tests {
    use crate::plan370_test_support::build_component_from_app;
    use auto_val::Value;

    fn locate_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan442_store_facade/src/front/app.at";
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
        build_component_from_app(&locate_corpus()?)
    }

    fn state_str(dc: &crate::ui::dynamic::DynamicComponent, field: &str) -> String {
        match dc.read_state(field) {
            Ok(Value::Str(s)) => s.as_str().to_string(),
            Ok(Value::Bool(b)) => b.to_string(),
            Ok(other) => format!("{:?}", other),
            Err(e) => panic!("read_state('{}') failed: {}", field, e),
        }
    }

    /// REGRESSION: the legacy `use store: AuthStore` facade must load the
    /// StoreDecl — its fields merge into root state, and the store's own
    /// Init handler runs (whoami seeded via the store file's `use auth_util`
    /// dep, proving dep collection through the facade path).
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn legacy_store_facade_loads_store_fields() {
        let dc = match build() {
            Some(c) => c,
            None => {
                eprintln!("plan442: SKIPPED — corpus app.at not found");
                return;
            }
        };
        let state = dc.read_all_state();
        for required in ["status", "token", "authenticated", "whoami"] {
            assert!(
                state.contains_key(required),
                "field '{}' missing from root state; keys = {:?}",
                required,
                state.keys().collect::<Vec<_>>()
            );
        }
        // store.Init() ran during fire_init: whoami seeded by guest_label()
        // (auth_util.at), and status set by the App handler after the call.
        assert_eq!(state_str(&dc, "whoami"), "guest", "store.Init() should run");
        assert_eq!(state_str(&dc, "status"), "ready");
        assert_eq!(state_str(&dc, "authenticated"), "false");
    }

    /// REGRESSION: cross-widget store method calls (`store.Login()` from the
    /// App handler) and cross-widget field reads (`store.authenticated != true`)
    /// must work — before the fix these compiled to "Undefined variable:
    /// store" and the handlers silently no-op'd.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn legacy_store_facade_cross_widget_calls_and_reads() {
        let mut dc = match build() {
            Some(c) => c,
            None => {
                eprintln!("plan442: SKIPPED — corpus app.at not found");
                return;
            }
        };
        dc.on_with_input("Check", None);
        assert_eq!(
            state_str(&dc, "status"),
            "anonymous",
            "Check should read store.authenticated == false"
        );
        dc.on_with_input("Login", None);
        assert_eq!(state_str(&dc, "authenticated"), "true");
        assert_eq!(state_str(&dc, "whoami"), "user");
        assert_eq!(state_str(&dc, "token"), "t-ok");
        dc.on_with_input("Check", None);
        assert_eq!(
            state_str(&dc, "status"),
            "signed-in",
            "Check should now read store.authenticated == true"
        );
        dc.on_with_input("Logout", None);
        assert_eq!(state_str(&dc, "authenticated"), "false");
        assert_eq!(state_str(&dc, "whoami"), "guest");
    }

    /// REGRESSION: the view layer must see the store fields — the no-dot
    /// `store.authenticated` / `store.whoami` references (musk view form)
    /// read the merged root state, so the sign-in branch flips after Login.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn legacy_store_facade_view_reads_store_state() {
        let mut dc = match build() {
            Some(c) => c,
            None => {
                eprintln!("plan442: SKIPPED — corpus app.at not found");
                return;
            }
        };
        let snap_of = |dc: &crate::ui::dynamic::DynamicComponent| -> String {
            let state = dc.read_all_state_materialized();
            let template = dc.view_template();
            use crate::ui::aura_snapshot_builder::AuraSnapshotBuilder;
            let builder = AuraSnapshotBuilder::new(&state);
            builder.build(dc.widget_name(), template)
        };
        let initial = snap_of(&dc);
        assert!(
            initial.contains("Sign in"),
            "unauthenticated view should offer Sign in; got:\n{}",
            initial
        );
        assert!(
            initial.contains("guest"),
            "view should read store.whoami; got:\n{}",
            initial
        );
        dc.on_with_input("Login", None);
        let signed_in = snap_of(&dc);
        assert!(
            signed_in.contains("Sign out"),
            "authenticated view should offer Sign out; got:\n{}",
            signed_in
        );
    }
}
