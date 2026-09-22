//! Cross-repo codegen debt tests (auto-down, 2026-08-29).
//!
//! Two debts found live while .at-izing the plan-023 editing faces
//! (CodeEditorBlock / TableEditorBlock), fixed the same day:
//!
//! 1. **TDZ emit order** — a state var initialized from a prop
//!    (`model { var draft str = .code }`) emitted `const draft =
//!    ref<string>(props.code)` BEFORE `const props = defineProps...`:
//!    ReferenceError at setup time. defineProps now emits first.
//! 2. **R012** — a v-for source root member matching no declared
//!    prop/state/computed (`.header_cells` vs prop `headerCells`)
//!    emitted `v-for="... in header_cells"` over `undefined`: a silent
//!    empty render. Now an Error-severity validation rule.

#[cfg(test)]
mod autodown_codegen_debts_tests {
    use crate::ui_gen::api::{generate_component_from_file, ComponentGenOptions};
    use crate::ui_gen::validators::Severity;

    fn gen(at_source: &str, tag: &str) -> crate::ui_gen::api::GeneratedComponent {
        let tmp = std::env::temp_dir().join(format!("autodown_debts_{tag}"));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let at_path = tmp.join("app.at");
        std::fs::write(&at_path, at_source).unwrap();
        generate_component_from_file(&at_path, ComponentGenOptions::default())
            .unwrap_or_else(|e| panic!("compile failed for {tag}: {e}"))
    }

    /// Debt 1: defineProps must precede prop-reading state refs (TDZ).
    #[test]
    fn test_prop_initialized_state_var_sees_defineProps_first() {
        let out = gen(
            concat!(
                "widget T(blockId: str, code: str) {\n",
                "    msg Msg { Init }\n",
                "    model { var draft str = .code }\n",
                "    view {\n",
                "        div { text .draft }\n",
                "    }\n",
                "    on {\n",
                "        .Init -> { .draft = .code }\n",
                "    }\n",
                "}\n",
            ),
            "tdz",
        );
        let sfc = &out.vue_code;
        let props_pos = sfc
            .find("defineProps")
            .unwrap_or_else(|| panic!("no defineProps in: {sfc}"));
        let ref_pos = sfc
            .find("ref<string>(props.code)")
            .unwrap_or_else(|| panic!("expected `ref<string>(props.code)` state init, got: {sfc}"));
        assert!(
            props_pos < ref_pos,
            "defineProps must be emitted BEFORE prop-reading state refs (TDZ):\n{sfc}"
        );
    }

    /// Debt 2: unknown single-segment v-for source reports R012 at Error
    /// severity (silent-empty-render class).
    #[test]
    fn test_vfor_unknown_source_reports_R012_error() {
        let out = gen(
            concat!(
                "widget T(headerCells: Array<str>) {\n",
                "    view {\n",
                "        div {\n",
                "            for i, cell in .header_cells {\n",
                "                span { text cell.text }\n",
                "            }\n",
                "        }\n",
                "    }\n",
                "}\n",
            ),
            "r012_bad",
        );
        let hit = out
            .validation_warnings
            .iter()
            .find(|w| w.rule == "R012")
            .unwrap_or_else(|| {
                panic!(
                    "expected an R012 warning, got: {:?}",
                    out.validation_warnings
                        .iter()
                        .map(|w| (w.rule, &w.message))
                        .collect::<Vec<_>>()
                )
            });
        assert!(
            matches!(hit.severity, Severity::Error),
            "R012 must be Error severity (broken product), got {:?}",
            hit.severity
        );
        assert!(
            hit.message.contains("header_cells"),
            "R012 message must name the offending source: {}",
            hit.message
        );
    }

    /// Guard: declared prop / state / computed v-for sources stay clean
    /// (no R012 false positives).
    #[test]
    fn test_vfor_known_sources_report_no_R012() {
        let out = gen(
            concat!(
                "widget T(headerCells: Array<str>) {\n",
                "    model { var rows Array<str> = [] }\n",
                "    view {\n",
                "        div {\n",
                "            for i, c in .headerCells { span { text c } }\n",
                "            for i, r in .rows { span { text r } }\n",
                "            for i, f in .flattened { span { text f } }\n",
                "        }\n",
                "    }\n",
                "    computed {\n",
                "        flattened => .rows\n",
                "    }\n",
                "}\n",
            ),
            "r012_good",
        );
        assert!(
            !out.validation_warnings.iter().any(|w| w.rule == "R012"),
            "known sources must not trip R012: {:?}",
            out.validation_warnings
                .iter()
                .map(|w| (w.rule, &w.message))
                .collect::<Vec<_>>()
        );
    }
}
