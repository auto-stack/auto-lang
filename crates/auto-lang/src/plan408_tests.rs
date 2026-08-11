//! Tests for `component fn` → independent Vue SFC synthesis (Plan 408).
//!
//! P1 (same-file synthesis) is covered by `test_a2vue_component_fn` in
//! `ui_gen/vue.rs`. This module covers the P2 residuals:
//!
//! - **Residual 2**: legacy build entry (`ui_build_shadcn_with_widgets_and_stores`)
//!   is a thin wrapper over `generate_component_from_file` (Plan 361), so it
//!   inherits component fn support automatically. This e2e test pins that
//!   behavior so a future refactor of the wrapper can't silently regress it.
//! - **Residual 1**: cross-file reuse via `use { component: X }` (no `from`)
//!   referencing a component fn synthesized in another file.

#[cfg(test)]
mod plan408_tests {
    use crate::session::CompilerSession;

    /// Residual 2: the legacy `ui_build_shadcn_with_widgets_and_stores` entry
    /// point (used by `cmd_vue` and auto-man `from_workspace`) must synthesize
    /// a `component fn` into its own SFC when an output dir is provided —
    /// proving the capability transparently passes through the thin wrapper.
    #[test]
    fn test_component_fn_legacy_entry_writes_sfc() {
        let tmp = std::env::temp_dir().join("plan408_legacy_entry_test");
        // Clean slate so stray files from a prior run don't mislead us.
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let at_path = tmp.join("app.at");
        std::fs::write(&at_path, concat!(
            "component fn Card(title: str) {\n",
            "    col { text .title }\n",
            "}\n",
            "\n",
            "widget App {\n",
            "    view { Card(title: .greeting) }\n",
            "    model { var greeting str = \"hi\" }\n",
            "}\n",
        )).unwrap();

        let (vue_code, widgets, _stores) = crate::ui_build_shadcn_with_widgets_and_stores(
            at_path.to_str().unwrap(),
            Some(tmp.to_str().unwrap()),
            None,
        ).expect("legacy entry must compile component fn source");

        // The component fn is synthesized to its own SFC on disk.
        let card_vue = std::fs::read_to_string(tmp.join("Card.vue"))
            .expect("Card.vue must be written by the legacy entry");
        assert!(card_vue.contains("defineProps"), "Card.vue must declare props: {}", card_vue);
        assert!(card_vue.contains("title"), "Card.vue must declare the title prop: {}", card_vue);

        // The host widget references it as a component (not inlined).
        assert!(vue_code.contains("import Card from '@/components/Card.vue'"),
            "App must import Card: {}", vue_code);
        assert!(vue_code.contains("<Card"),
            "App must render <Card/>: {}", vue_code);

        // Card also appears in the returned widget list (sub-widget discovery).
        let names: Vec<&str> = widgets.iter().map(|w| w.name.as_str()).collect();
        assert!(names.contains(&"Card"), "Card must be in widgets list: {:?}", names);

        // Cleanup.
        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// Plan 367 P2-3 compatibility: `view fn` (inline) parsing is unchanged by
    /// the `component fn` addition. Regression guard for the shared parser path.
    #[test]
    fn test_view_fn_still_parses() {
        let src = "view fn RenderT(a: Any) { div { text .a } }\nwidget W { view { RenderT(a: .x) } model { var x str = \"\" } }";
        let session = CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("view fn must still parse");
        // Exactly one view fn (inline) + one widget — no `component fn`.
        let view_fns = ast.stmts.iter().filter(|s| matches!(
            s, crate::ast::Stmt::ViewFragmentDecl(f) if !f.is_component
        )).count();
        assert_eq!(view_fns, 1, "expected exactly one inline view fn");
    }

    /// Residual 1 (auto-lang side): `use { component: X }` without a `from`
    /// clause must parse and resolve to `@/components/{X}.vue` — the same
    /// path a `component fn` SFC is written to. This is the cross-file reuse
    /// grammar; in a single file it overlaps with same-file sub_widgets
    /// resolution, but the parser + codegen path is identical.
    #[test]
    fn test_use_component_no_from_resolves_to_components() {
        // The `use { component: Badge }` has no `from`; Badge need NOT be a
        // component fn in this file (cross-file case). We only verify the
        // generated import path here.
        let src = concat!(
            "widget App {\n",
            "    use {\n",
            "        component: Badge\n",
            "    }\n",
            "    view { Badge(label: .greeting) }\n",
            "    model { var greeting str = \"hi\" }\n",
            "}\n",
        );
        let tmp = std::env::temp_dir().join("plan408_cross_file_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let at_path = tmp.join("app.at");
        std::fs::write(&at_path, src).unwrap();

        let result = crate::ui_gen::generate_component_from_file(
            &at_path,
            crate::ui_gen::ComponentGenOptions::default(),
        ).expect("use{component} without from must compile");

        let app_code = result.vue_code.clone();
        assert!(
            app_code.contains("import Badge from '@/components/Badge.vue'"),
            "use {{component: Badge}} (no from) must import @/components/Badge.vue: {}",
            app_code
        );
        assert!(
            app_code.contains("<Badge"),
            "App must render <Badge/>: {}",
            app_code
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// Plan 408 P3: `component fn` with a `computed { }` block. Mirrors widget
    /// computed — derived expressions over props, emitted as
    /// `const x = computed(() => ...)`. This is the capability gap that blocked
    /// the auto-musk AgentAvatar pilot (P2 residual 3): component fn had no
    /// computed, so any展示 component with derived state was unexpressible.
    #[test]
    fn test_component_fn_with_computed() {
        let tmp = std::env::temp_dir().join("plan408_computed_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let at_path = tmp.join("app.at");
        // Badge derives `label` from props via computed, then renders it.
        // Use a plain prop-ref expression (f-string in computed is a separate
        // transpile concern, out of scope for this P3 capability test).
        std::fs::write(&at_path, concat!(
            "component fn Badge(text: str) {\n",
            "    computed {\n",
            "        label => .text\n",
            "    }\n",
            "    span { text .label }\n",
            "}\n",
            "\n",
            "widget App {\n",
            "    view { Badge(text: .heading) }\n",
            "    model { var heading str = \"hi\" }\n",
            "}\n",
        )).unwrap();

        let result = crate::ui_gen::generate_component_from_file(
            &at_path,
            crate::ui_gen::ComponentGenOptions::default(),
        ).expect("component fn with computed must compile");

        let badge_code = result.all_widget_codes.iter()
            .find(|(name, _)| name == "Badge")
            .map(|(_, code)| code)
            .expect("Badge component fn must be synthesized");

        // computed → `const label = computed(() => ...)` (type may be inferred).
        assert!(
            badge_code.contains("const label = computed"),
            "Badge SFC must emit `const label = computed(...)`: {}",
            badge_code
        );
        // template uses the computed ({{ label }}), not the raw prop text.
        assert!(
            badge_code.contains("{{ label }}"),
            "Badge template must render the computed label: {}",
            badge_code
        );

        // view fn (inline) does NOT support computed — it has no component host.
        // A view fn body containing `computed { }` is parsed as a view element
        // (the `computed` tag + its `{ => ... }` children), which is not valid
        // view syntax and surfaces as a parse error. This is the desired guard:
        // computed only belongs in `component fn` (which has a real SFC host).
        let bad_src = "view fn NoComputed(a: str) { computed { x => .a } div { text .x } }\nwidget W { view { NoComputed(a: .y) } model { var y str = \"\" } }";
        let session = CompilerSession::ui();
        let mut parser = crate::Parser::from(bad_src).with_session(session);
        let parse_outcome = parser.parse();
        assert!(parse_outcome.is_err(),
            "view fn with a computed block must NOT parse cleanly (computed is component-fn-only): {:?}",
            parse_outcome);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    /// Plan 408 P4: `component fn` with `msg` / `model` / `on` blocks. The
    /// component can hold local state (model → `ref<T>`), react to events
    /// (on → handler functions), and emit messages upward (msg →
    /// `defineEmits` + `emit()`). Mirrors widget semantics; all downstream
    /// codegen (defineEmits, ref, handler auto-emit, parent `@Event` binding)
    /// is reused unchanged.
    #[test]
    fn test_component_fn_with_emit() {
        let tmp = std::env::temp_dir().join("plan408_emit_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let at_path = tmp.join("app.at");
        // CollapseBtn toggles its own `collapsed` state AND emits ToggleCollapse
        // upward so the parent can react. App subscribes via ontoggle.
        std::fs::write(&at_path, concat!(
            "component fn CollapseBtn(label: str) {\n",
            "    msg Msg { ToggleCollapse }\n",
            "    model { var collapsed bool = false }\n",
            "    on { .ToggleCollapse -> { .collapsed = !.collapsed } }\n",
            "    button {\n",
            "        text .label\n",
            "        onclick: .ToggleCollapse\n",
            "    }\n",
            "}\n",
            "\n",
            "widget App {\n",
            "    model { var count int = 0 }\n",
            "    view {\n",
            "        CollapseBtn(label: \"go\", ontogglecollapse: .Bump)\n",
            "    }\n",
            "    on { .Bump -> { .count = .count + 1 } }\n",
            "}\n",
        )).unwrap();

        let result = crate::ui_gen::generate_component_from_file(
            &at_path,
            crate::ui_gen::ComponentGenOptions::default(),
        ).expect("component fn with emit must compile");

        let btn_code = result.all_widget_codes.iter()
            .find(|(name, _)| name == "CollapseBtn")
            .map(|(_, code)| code)
            .expect("CollapseBtn component fn must be synthesized");
        let app_code = result.vue_code.clone();

        // msg → defineEmits<{ ToggleCollapse: [] }>()
        assert!(
            btn_code.contains("defineEmits"),
            "CollapseBtn must declare emits: {}",
            btn_code
        );
        assert!(
            btn_code.contains("ToggleCollapse"),
            "CollapseBtn emits must include ToggleCollapse: {}",
            btn_code
        );
        // model → ref<boolean>(false)
        assert!(
            btn_code.contains("ref<boolean>(false)"),
            "CollapseBtn must emit `const collapsed = ref<boolean>(false)`: {}",
            btn_code
        );
        // on handler → function body mutates state + emits
        assert!(
            btn_code.contains("collapsed.value = !collapsed.value"),
            "CollapseBtn handler must mutate collapsed state: {}",
            btn_code
        );
        assert!(
            btn_code.contains("emit('ToggleCollapse')"),
            "CollapseBtn handler must emit ToggleCollapse: {}",
            btn_code
        );

        // App subscribes to the event. The event name is lowercased by
        // sub_widget_event_to_vue (ontogglecollapse → @togglecollapse), so the
        // parent binds `@togglecollapse="Bump"`.
        assert!(
            app_code.contains("@togglecollapse"),
            "App must bind the toggle event on CollapseBtn: {}",
            app_code
        );

        // Regression: view fn (inline) does NOT support msg/model/on. A view fn
        // body containing these blocks must NOT parse cleanly — they belong
        // only in component fn (which has a real SFC host for state/emit).
        let bad_src = "view fn NoEmit(a: str) { msg Msg { X } div { text .a } }\nwidget W { view { NoEmit(a: .y) } model { var y str = \"\" } }";
        let session = CompilerSession::ui();
        let mut parser = crate::Parser::from(bad_src).with_session(session);
        let parse_outcome = parser.parse();
        assert!(parse_outcome.is_err(),
            "view fn with a msg block must NOT parse cleanly (msg is component-fn-only): {:?}",
            parse_outcome);

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
