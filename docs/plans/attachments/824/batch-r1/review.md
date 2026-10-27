# 2026-10-27 batch retry — incomplete, failure recorded

Frozen language HEAD `bfefde1a6469e46153e822bcd5b3efde361dc073`, UI `4903bef64532750d1597627b1197f3d2cac1effc`; the run began after Plan 842's work turn completed and no cargo/rustc processes were observed. The official main-only `cargo tf --batch` driver acquired its machine mutex and stopped at its coverage guard before executing language tests: 3,711 listed, 3,211 selected, 12 baseline IDs missing and therefore 12 suppressed. The earlier 855 compilation failure did not recur in this listing, but no claim of full closure is made.

To collect independent remaining evidence without relaxing the driver, `cargo tt --no-fail-fast` and `cargo tb --no-fail-fast` ran sequentially under the same machine mutex and with the driver's report-preservation routine. Language/UI identities matched before and after both runs. `tt` ran 3,563: 3,562 passed and 1 failed (`plan707_spike_cancel_gap_active_job_future_not_stopped`); duration 459.36 s wall. The observed failure says the cancelled job future advanced; causal attribution and whether this is an intermittent pre-existing race remain unproven. `tb` ran 3,102: all passed; duration 48.51 s wall. Preserved producer reports were restored with matching SHA-256.

The UI owner was independently listed under the same full-tier options and guard: 3,126 listed, 3,108 selected, 9 baseline IDs missing and hence 9 suppressed; no UI tests executed. Identities matched before/after. All missing UI IDs have namesakes in `auto-ui-compiler/tests/design_tokens_recipe.rs`, an integration-test target not included in the UI `--lib` selection; this is a lead, not proof that coverage and all assertions are equivalent. The language changes include Plan 841 commits `56ffdb419` and `465c3c860`; the ten design-token names appear under the UI compiler, and at least one parser test has a namesake under Core. Investigate exact mapping, assertions, platform/feature selection, and any truly retired tests before updating a coverage baseline. Do not delete IDs or filter tests to force green.

Language missing IDs:

- `auto-lang :: tests::plan810_full_ui_grammar::r02_same_parser_error_leaks_handler_scope_baseline`
- `auto-lang :: tests::plan810_full_ui_grammar::r04_scene_gate_prev_matches_old_two_push`
- `auto-lang-core :: design_tokens::decl::tests::compose_overrides_and_normalizes`
- `auto-lang-core :: design_tokens::decl::tests::depth_limit_and_cycle`
- `auto-lang-core :: design_tokens::decl::tests::named_chain_layering_and_mode_inheritance`
- `auto-lang-core :: design_tokens::decl::tests::negative_cases`
- `auto-lang-core :: design_tokens::registry::tests::accent_table_fingerprint`
- `auto-lang-core :: design_tokens::registry::tests::dual_face_consistency`
- `auto-lang-core :: design_tokens::registry::tests::render_fingerprint`
- `auto-lang-core :: design_tokens::registry::tests::scaffold_extended_tokens_dual_face`
- `auto-lang-core :: design_tokens::registry::tests::theme_palettes_js_shape`
- `auto-lang-core :: design_tokens::registry::tests::themes_core_complete`

UI missing IDs:

- `auto-ui-dialect :: regression::design_tokens_recipe::tests::test_plan635_cross_package_deps_layout`
- `auto-ui-dialect :: regression::design_tokens_recipe::tests::test_plan635_name_collision_is_error`
- `auto-ui-dialect :: regression::design_tokens_recipe::tests::test_plan635_named_import_non_pub_is_error`
- `auto-ui-dialect :: regression::design_tokens_recipe::tests::test_plan635_named_import_registers_and_desugars`
- `auto-ui-dialect :: regression::design_tokens_recipe::tests::test_plan635_transitive_import`
- `auto-ui-dialect :: regression::design_tokens_recipe::tests::test_plan635_wildcard_imports_pub_only`
- `auto-ui-dialect :: regression::design_tokens_recipe::tests::test_style_recipe_desugar_engine`
- `auto-ui-dialect :: regression::design_tokens_recipe::tests::test_style_recipe_lint_palette_warnings`
- `auto-ui-dialect :: regression::design_tokens_recipe::tests::test_style_recipe_semantic_validation`

The batch is not complete or green. Plan 850 remains the last fully covered baseline; the pre-existing 855 failed attempt remains in history. No aavm trigger paths changed in the window (the lib.rs changes did not touch aavm source lists). Follow-up: independently triage the Plan 841 test ownership/coverage gap and the cancellation failure, establish a fix-forward owner, then rerun the complete batch with unchanged required assertions. The shared plan allocator contains pre-existing uncommitted changes; no independent fix plan was allocated or claimed in this attempt. The 824 ledger-checker cache remains blocked by the prior automatic safety rejection.
