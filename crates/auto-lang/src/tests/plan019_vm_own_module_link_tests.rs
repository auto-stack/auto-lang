//! PLAN-019 T-06 regression tests: own-module binding for bare intra-module
//! calls inside module-qualified fns (VM merged single-module synthesis).
//!
//! ## Root cause (019 §10.6 vm-form link failure; real corpus
//! "Undefined symbol: mux_resize_branch / mux_tab_id_at in module App")
//!
//! handler_codegen flattens every imported module's fns into ONE VM module
//! with qualified names (`api.X`, `db.X`). A bare call inside such a fn body
//! (`mux_resize_branch(...)` inside `db.mux_resize_pane`) is ambiguous at
//! module scope when the bare name exists under more than one module prefix
//! (api + db both define it): the unique-suffix fallback in
//! `resolved_func`/`resolve_call_symbol` refuses to guess, the call emits a
//! BARE `FuncCall` reloc, and the linker cannot bind it — the entry module's
//! exports are all dotted, and the Plan 545 own-module fallback expects
//! `mod#sym` keys the entry module never produces. 018 never hit this: its
//! front never called any fn transitively containing a bare intra-module
//! call (§10.6's "GET with params" attribution was coincidental — the
//! bare-calling fns happened to take params).
//!
//! Fix: `Codegen::current_fn_module` records the module prefix of the fn
//! being compiled; bare names bind to their own module's export first
//! (mirrors loader Plan 322/545 own-module semantics at codegen level).
//!
//! Corpus: `test/ui/plan019_vm_own_module_link/` — `db.get_val` bare-calls
//! `bump` (declared earlier in db.at, ambiguous with `api.bump`); `api.bump`
//! deliberately computes `db.bump(x) + 100` so the consumed value pins the
//! binding target: own-module → 4, wrong-module wrapper → 103.

#[cfg(test)]
mod plan019_vm_own_module_link_tests {
    use auto_val::Value;

    fn locate_corpus() -> Option<std::path::PathBuf> {
        let rel = "test/ui/plan019_vm_own_module_link/src/front/app.at";
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
            Ok(Value::Int(i)) => i.to_string(),
            Ok(Value::Bool(b)) => b.to_string(),
            Ok(other) => format!("{:?}", other),
            Err(e) => panic!("read_state('{}') failed: {}", field, e),
        }
    }

    /// REGRESSION: before the fix the whole component failed to build —
    /// the bare `bump(x)` call inside `db.get_val` left an unresolvable
    /// bare reloc ("Undefined symbol: bump in module App") and init never
    /// ran. After the fix it binds `db.bump` (own module) and computes 4.
    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn own_module_bare_call_links_and_init_runs() {
        let dc = match build() {
            Some(c) => c,
            None => {
                eprintln!("plan019: SKIPPED — corpus app.at not found");
                return;
            }
        };
        // Init ran through the full chain: api.get_val(3) → db.get_val →
        // bare bump → db.bump (own module, NOT the api wrapper) → 4.
        assert_eq!(state_str(&dc, "v"), "4");
    }
}
