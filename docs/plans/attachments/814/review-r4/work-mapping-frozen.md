# R3-05 测试选择面映射表（PLAN-814 R3-03b 全文）

迁前形态：86 项 UI 文法依赖测试在 auto-lang-core 内 `#[cfg(feature = "ui-dialect")]`
门控（parser 60 / design_tokens 9 / tscn 11 / gdscript 6），经 feature 组合
（上层 default 转发 / auto-ui-compiler 显式启用 + feature 统一）进档。

迁后形态：同一 86 项随文法 owner 迁 auto-ui-dialect `src/regression/`
（dev-dep auto-lang-core 驱动真 Parser；文法表经 `setup()` 进程级装配），
无条件编译——`.cargo/config.toml` 的 t/tf/tt 别名已扩 `-p auto-ui-dialect`
保持选择（迁移当轮起生效；tu 档为 compiler 侧 ui_gen 面，不含本族）。

## 逐项映射（86/86，无丢失/无新增）

| # | 迁前 ID（auto-lang-core, feature on） | 迁后 ID（auto-ui-dialect） |
|---|---|---|
| 1 | `auto-lang-core design_tokens/recipe::tests::test_plan635_cross_package_deps_layout` | `auto-ui-dialect regression::design_tokens_recipe::tests::test_plan635_cross_package_deps_layout` |
| 2 | `auto-lang-core design_tokens/recipe::tests::test_plan635_name_collision_is_error` | `auto-ui-dialect regression::design_tokens_recipe::tests::test_plan635_name_collision_is_error` |
| 3 | `auto-lang-core design_tokens/recipe::tests::test_plan635_named_import_non_pub_is_error` | `auto-ui-dialect regression::design_tokens_recipe::tests::test_plan635_named_import_non_pub_is_error` |
| 4 | `auto-lang-core design_tokens/recipe::tests::test_plan635_named_import_registers_and_desugars` | `auto-ui-dialect regression::design_tokens_recipe::tests::test_plan635_named_import_registers_and_desugars` |
| 5 | `auto-lang-core design_tokens/recipe::tests::test_plan635_transitive_import` | `auto-ui-dialect regression::design_tokens_recipe::tests::test_plan635_transitive_import` |
| 6 | `auto-lang-core design_tokens/recipe::tests::test_plan635_wildcard_imports_pub_only` | `auto-ui-dialect regression::design_tokens_recipe::tests::test_plan635_wildcard_imports_pub_only` |
| 7 | `auto-lang-core design_tokens/recipe::tests::test_style_recipe_desugar_engine` | `auto-ui-dialect regression::design_tokens_recipe::tests::test_style_recipe_desugar_engine` |
| 8 | `auto-lang-core design_tokens/recipe::tests::test_style_recipe_lint_palette_warnings` | `auto-ui-dialect regression::design_tokens_recipe::tests::test_style_recipe_lint_palette_warnings` |
| 9 | `auto-lang-core design_tokens/recipe::tests::test_style_recipe_semantic_validation` | `auto-ui-dialect regression::design_tokens_recipe::tests::test_style_recipe_semantic_validation` |
| 10 | `auto-lang-core parser::tests::plan046_keyed_for_item_field_chain` | `auto-ui-dialect regression::parser_ui_grammar::tests::plan046_keyed_for_item_field_chain` |
| 11 | `auto-lang-core parser::tests::plan046_keyed_for_scalar_literals` | `auto-ui-dialect regression::parser_ui_grammar::tests::plan046_keyed_for_scalar_literals` |
| 12 | `auto-lang-core parser::tests::plan046_keyed_for_state_chain` | `auto-ui-dialect regression::parser_ui_grammar::tests::plan046_keyed_for_state_chain` |
| 13 | `auto-lang-core parser::tests::plan046_keyed_for_with_index_var` | `auto-ui-dialect regression::parser_ui_grammar::tests::plan046_keyed_for_with_index_var` |
| 14 | `auto-lang-core parser::tests::plan046_keyless_for_has_no_key_expr` | `auto-ui-dialect regression::parser_ui_grammar::tests::plan046_keyless_for_has_no_key_expr` |
| 15 | `auto-lang-core parser::tests::plan046_memo_block_body_nodes` | `auto-ui-dialect regression::parser_ui_grammar::tests::plan046_memo_block_body_nodes` |
| 16 | `auto-lang-core parser::tests::plan046_memo_block_exact_and_mixed_params` | `auto-ui-dialect regression::parser_ui_grammar::tests::plan046_memo_block_exact_and_mixed_params` |
| 17 | `auto-lang-core parser::tests::plan046_memo_block_parses_deps_and_body` | `auto-ui-dialect regression::parser_ui_grammar::tests::plan046_memo_block_parses_deps_and_body` |
| 18 | `auto-lang-core parser::tests::plan046_outlet_prop_parsing` | `auto-ui-dialect regression::parser_ui_grammar::tests::plan046_outlet_prop_parsing` |
| 19 | `auto-lang-core parser::tests::test_041a_event_binding_forms` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_041a_event_binding_forms` |
| 20 | `auto-lang-core parser::tests::test_actions_block_parse` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_actions_block_parse` |
| 21 | `auto-lang-core parser::tests::test_actions_cond_bare_expression` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_actions_cond_bare_expression` |
| 22 | `auto-lang-core parser::tests::test_aura_dot_prefixed_handler_syntax` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_aura_dot_prefixed_handler_syntax` |
| 23 | `auto-lang-core parser::tests::test_aura_model_var_keyword` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_aura_model_var_keyword` |
| 24 | `auto-lang-core parser::tests::test_bare_value_input_mints_oninput_sync` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_bare_value_input_mints_oninput_sync` |
| 25 | `auto-lang-core parser::tests::test_computed_multiline_body` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_computed_multiline_body` |
| 26 | `auto-lang-core parser::tests::test_dot_rhs_field_access_not_struct_construction` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_dot_rhs_field_access_not_struct_construction` |
| 27 | `auto-lang-core parser::tests::test_event_inline_lambda_parses` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_event_inline_lambda_parses` |
| 28 | `auto-lang-core parser::tests::test_gt_syntax_not_text_shorthand` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_gt_syntax_not_text_shorthand` |
| 29 | `auto-lang-core parser::tests::test_hover_card_mints_hover_enter_leave` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_hover_card_mints_hover_enter_leave` |
| 30 | `auto-lang-core parser::tests::test_hyphenated_tag_names` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_hyphenated_tag_names` |
| 31 | `auto-lang-core parser::tests::test_leading_dot_stmt_not_glued_to_previous` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_leading_dot_stmt_not_glued_to_previous` |
| 32 | `auto-lang-core parser::tests::test_modal_dialog_mints_self_managed_open` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_modal_dialog_mints_self_managed_open` |
| 33 | `auto-lang-core parser::tests::test_msg_decl_unnamed_and_legacy_name` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_msg_decl_unnamed_and_legacy_name` |
| 34 | `auto-lang-core parser::tests::test_msg_full_shellstore_declaration` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_msg_full_shellstore_declaration` |
| 35 | `auto-lang-core parser::tests::test_msg_multi_param_payload` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_msg_multi_param_payload` |
| 36 | `auto-lang-core parser::tests::test_msg_variant_named_payload` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_msg_variant_named_payload` |
| 37 | `auto-lang-core parser::tests::test_on_handler_multi_param_binding` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_on_handler_multi_param_binding` |
| 38 | `auto-lang-core parser::tests::test_pipe_syntax_works_for_text` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_pipe_syntax_works_for_text` |
| 39 | `auto-lang-core parser::tests::test_plan778_err_keeps_dialect_registration` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_plan778_err_keeps_dialect_registration` |
| 40 | `auto-lang-core parser::tests::test_plan778_none_rolls_back_cursor_for_next_dialect` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_plan778_none_rolls_back_cursor_for_next_dialect` |
| 41 | `auto-lang-core parser::tests::test_plan778_with_session_preserves_external_registration` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_plan778_with_session_preserves_external_registration` |
| 42 | `auto-lang-core parser::tests::test_stmt_after_try_catch_parses` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_stmt_after_try_catch_parses` |
| 43 | `auto-lang-core parser::tests::test_string_as_primary_prop_h1` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_string_as_primary_prop_h1` |
| 44 | `auto-lang-core parser::tests::test_string_as_primary_prop_text` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_string_as_primary_prop_text` |
| 45 | `auto-lang-core parser::tests::test_string_as_primary_prop_with_additional_props` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_string_as_primary_prop_with_additional_props` |
| 46 | `auto-lang-core parser::tests::test_string_with_special_chars` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_string_with_special_chars` |
| 47 | `auto-lang-core parser::tests::test_struct_literal_construction_unresolved_type` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_struct_literal_construction_unresolved_type` |
| 48 | `auto-lang-core parser::tests::test_struct_literal_fields_become_node_args` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_struct_literal_fields_become_node_args` |
| 49 | `auto-lang-core parser::tests::test_style_recipe_parse` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_style_recipe_parse` |
| 50 | `auto-lang-core parser::tests::test_top_level_actions_decl` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_top_level_actions_decl` |
| 51 | `auto-lang-core parser::tests::test_top_level_view_block_now_parses_in_ui_scenario` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_top_level_view_block_now_parses_in_ui_scenario` |
| 52 | `auto-lang-core parser::tests::test_try_catch_finally_parses` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_try_catch_finally_parses` |
| 53 | `auto-lang-core parser::tests::test_try_catch_param_resolves_in_catch_body` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_try_catch_param_resolves_in_catch_body` |
| 54 | `auto-lang-core parser::tests::test_try_catch_without_finally_unchanged` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_try_catch_without_finally_unchanged` |
| 55 | `auto-lang-core parser::tests::test_view_condition_none_comparison` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_view_condition_none_comparison` |
| 56 | `auto-lang-core parser::tests::test_view_dyn_component_parens` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_view_dyn_component_parens` |
| 57 | `auto-lang-core parser::tests::test_view_event_arg_nested_self_dot` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_view_event_arg_nested_self_dot` |
| 58 | `auto-lang-core parser::tests::test_view_event_modifiers_and_event_object` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_view_event_modifiers_and_event_object` |
| 59 | `auto-lang-core parser::tests::test_view_quoted_custom_event_names` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_view_quoted_custom_event_names` |
| 60 | `auto-lang-core parser::tests::test_view_single_root_stays_unwrapped` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_view_single_root_stays_unwrapped` |
| 61 | `auto-lang-core parser::tests::test_view_style_obj_binding` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_view_style_obj_binding` |
| 62 | `auto-lang-core parser::tests::test_view_top_level_bare_siblings_with_param_events` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_view_top_level_bare_siblings_with_param_events` |
| 63 | `auto-lang-core parser::tests::test_widget_block_keyword_near_miss_warning` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_widget_block_keyword_near_miss_warning` |
| 64 | `auto-lang-core parser::tests::test_widget_expose_block` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_widget_expose_block` |
| 65 | `auto-lang-core parser::tests::test_widget_parses_via_dialect_in_ui_scenario` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_widget_parses_via_dialect_in_ui_scenario` |
| 66 | `auto-lang-core parser::tests::test_widget_parsing_pipeline` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_widget_parsing_pipeline` |
| 67 | `auto-lang-core parser::tests::test_widget_watch_block` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_widget_watch_block` |
| 68 | `auto-lang-core parser::tests::test_widget_with_routes` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_widget_with_routes` |
| 69 | `auto-lang-core parser::tests::test_with_session_registers_ui_dialect` | `auto-ui-dialect regression::parser_ui_grammar::tests::test_with_session_registers_ui_dialect` |
| 70 | `auto-lang-core trans/gdscript::tests::test_godot_demo_dodge_mob` | `auto-ui-dialect regression::trans_gdscript::tests::test_godot_demo_dodge_mob` |
| 71 | `auto-lang-core trans/gdscript::tests::test_godot_demo_dodge_player` | `auto-ui-dialect regression::trans_gdscript::tests::test_godot_demo_dodge_player` |
| 72 | `auto-lang-core trans/gdscript::tests::test_godot_demo_hexagonal_troll` | `auto-ui-dialect regression::trans_gdscript::tests::test_godot_demo_hexagonal_troll` |
| 73 | `auto-lang-core trans/gdscript::tests::test_godot_demo_instancing_ball_factory` | `auto-ui-dialect regression::trans_gdscript::tests::test_godot_demo_instancing_ball_factory` |
| 74 | `auto-lang-core trans/gdscript::tests::test_godot_demo_kinematic_player` | `auto-ui-dialect regression::trans_gdscript::tests::test_godot_demo_kinematic_player` |
| 75 | `auto-lang-core trans/gdscript::tests::test_godot_signals` | `auto-ui-dialect regression::trans_gdscript::tests::test_godot_signals` |
| 76 | `auto-lang-core trans/tscn::tests::test_godot_demo_dodge_mob_scene` | `auto-ui-dialect regression::trans_tscn::tests::test_godot_demo_dodge_mob_scene` |
| 77 | `auto-lang-core trans/tscn::tests::test_godot_demo_dodge_player_scene` | `auto-ui-dialect regression::trans_tscn::tests::test_godot_demo_dodge_player_scene` |
| 78 | `auto-lang-core trans/tscn::tests::test_godot_demo_hexagonal_troll_scene` | `auto-ui-dialect regression::trans_tscn::tests::test_godot_demo_hexagonal_troll_scene` |
| 79 | `auto-lang-core trans/tscn::tests::test_godot_demo_instancing_ball_scene` | `auto-ui-dialect regression::trans_tscn::tests::test_godot_demo_instancing_ball_scene` |
| 80 | `auto-lang-core trans/tscn::tests::test_godot_demo_kinematic_player_scene` | `auto-ui-dialect regression::trans_tscn::tests::test_godot_demo_kinematic_player_scene` |
| 81 | `auto-lang-core trans/tscn::tests::test_tscn_001_hello` | `auto-ui-dialect regression::trans_tscn::tests::test_tscn_001_hello` |
| 82 | `auto-lang-core trans/tscn::tests::test_tscn_002_player` | `auto-ui-dialect regression::trans_tscn::tests::test_tscn_002_player` |
| 83 | `auto-lang-core trans/tscn::tests::test_tscn_003_timers` | `auto-ui-dialect regression::trans_tscn::tests::test_tscn_003_timers` |
| 84 | `auto-lang-core trans/tscn::tests::test_tscn_004_nested` | `auto-ui-dialect regression::trans_tscn::tests::test_tscn_004_nested` |
| 85 | `auto-lang-core trans/tscn::tests::test_tscn_005_subresource` | `auto-ui-dialect regression::trans_tscn::tests::test_tscn_005_subresource` |
| 86 | `auto-lang-core trans/tscn::tests::test_tscn_006_combined_with_gd` | `auto-ui-dialect regression::trans_tscn::tests::test_tscn_006_combined_with_gd` |

## 档位覆盖对照

| 档 | 迁前是否覆盖 | 迁后是否覆盖 | 机制 |
|---|---|---|---|
| `cargo t`（日常） | 是（feature 统一经上层 default 转发拉入） | 是（`-p auto-ui-dialect` 直选） | t 别名扩包 |
| `cargo tf`（批量回归） | 是（同上） | 是（`-p auto-ui-dialect` 直选） | tf 别名扩包 |
| `cargo tt`（trans 档） | 是（tscn/gdscript 17 项经 `-p auto-lang-core` + feature 统一） | 是（`-p auto-ui-dialect` 直选） | tt 别名扩包 |
| `cargo tu`（ui_gen 档） | 否（滤串 ui_gen:: 不含本族） | 否（不变） | — |
| `cargo tv`/`tb`/`taa`/`ta` | 否（非本族档） | 否（不变） | — |

## 采集与计数核对

- 迁前清单：`cargo nextest list -p auto-lang-core --lib --features
  auto-lang-core/ui-dialect`（受审基线 d905e796c）+ 门控行源码级提取
  （`#[cfg(feature = "ui-dialect")]` 测试标记 → fn 名）双核对，恰 86 项。
- 迁后清单：`cargo nextest list -p auto-ui-dialect --lib`（R3-03b 工作树），
  86 项全为 regression:: 族。
- 运行核对：auto-ui-dialect 86/86 绿（0.35s）；auto-lang-core 全档绿
  （表缺席形态）；上层 auto-lang 定向 UI 面 107 项绿；ui_gen 862 绿。
- SDK 计数：迁前 on 1713 / off 1627（差恰 86）；迁后无 feature 可开、
  门控测试移出后全档绿（test-cases on 1632）。
