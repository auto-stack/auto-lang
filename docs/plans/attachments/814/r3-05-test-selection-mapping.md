# R3-05/R4 测试选择面映射表（PLAN-814；R4 修订——基面回溯迁包前 master）

R4 复审判定：原映射表以中途基线 d905e796c 为参照，且把 tb/taa/ta/t3 等
原档位说成"非本族档/否"——**迁包前这些档位经 `-p auto-lang`（default 含
ui-iced）实际覆盖了后来迁出的全部 inline 测试**。本修订把映射基面回溯到
**814 迁包前的 master `203e2ce91`**（Plan 基线提交），逐档重建选择面。

## 方法论（可复现）

- 旧侧：临时 worktree（组内 detached@`203e2ce91`）按**该 commit 的原别名
  定义**逐档 `cargo nextest list`（t/tf/tv/tt/tb/taa/ta/t3 + tu 的 libtest
  `-- --list`；旧别名取 `git show 203e2ce91:.cargo/config.toml`）。
- 新侧：R4 修复后工作树按当前别名逐档同命令。
- 匹配：路径精确匹配 + 已记录迁移动作的模块重写规则
  （`parser::tests::`→`regression::parser_ui_grammar::tests::`、
  `design_tokens::recipe::tests::`→`regression::design_tokens_recipe::tests::`
  /`tests::plan814_recipe_extraction_tests::`、`trans::{tscn,gdscript}::tests::`
  →`regression::trans_*::tests::`、`ui::{action_config,style,iced::lucide_generated}::`
  →compiler 同名去前缀、`ui::style::iced_adapter::`→`ui::style_iced_adapter::`、
  `tests::heavy_gate::`→`heavy_gate::`、
  `vm::native::tests::code_editor_natives::`→`tests::plan413_code_editor_natives::`、
  back_proxy/back_prefix→`tests::plan712_*`）。

## 逐档结果（R4 修复后，方法如上）

| 档 | 旧（203e2ce91） | 新（修复后） | 未匹配（真实丢失） | 选择缺口 |
|---|---|---|---|---|
| `t` | 5303 | 5307 | **0** | **0** |
| `tf` | 6270 | 6274 | **0** | **0** |
| `tv` | 163 | 163 | **0** | **0** |
| `tt` | 5675 | 5679 | **0** | **0** |
| `tb` | 5296 | 5300 | **0** | **0** |
| `tu`（libtest） | 869 | 869 | **0** | **0** |
| `taa` | 6266 | 6270 | **0** | **0** |
| `ta` | 6720 | 6724 | **0** | **0** |
| `t3` | 6721 | 6725 | **0** | **0** |

（新计数略增=新增回归：ui_cache 身份语义 2 + build 边界负控 2 等。）

## R4 判定项的修复清单

1. **别名扩包**（`.cargo/config.toml`）：tt 增 `-p auto-ui-compiler`；
   tb/taa/ta/t3 增 `-p auto-lang-core -p auto-ui-compiler -p auto-ui-dialect`
   （taa/ta/t3 裸 feature 列表同步前缀化——多 -p 下裸 feature 会红）。
2. **aavm 档编译红修复（T-02 遗留回归）**：`aavm2_m4.rs` 两处
   `crate::remap_string_indices` → `auto_lang_core::remap_string_indices`
   （fn 迁 SDK 后提 pub）——taa/ta/t3 自 T-02 起 lib 编译红（该族短期
   不跑未暴露），本次重建清单时发现并修复。
3. **残留 `ui-iced` 门归位**：compiler `ui_gen/rust.rs`
   `image_surface_rust_codegen` 测试门 `ui-iced`→`ui`（旧包 default 链
   ui-iced→ui→compiler/ui 转发，各档恢复选择）。
4. **恢复 T-02 被丢测试**：`test_style_recipe_in_widget_extraction`
   （203e2ce91 在册、T-02 大迁移时丢失）——归属 auto-ui-compiler
   `style`（aura owner；语义逐字保持 + 显式文法装配）。

## 86 项文法测试映射（沿 R3-03b 批；R4 复审独立验证 86/86 名称相等）

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

## 采集工件

- 旧侧逐档清单：临时 worktree `target/r4-old-*.txt`（树随验后删除；方法
  与命令可复现）。新侧逐档清单与匹配脚本：工作树 `target/r4-cur/`
  （tier_diff.py/exact_diff.py）。
- R3-03b 批原始采集（d905e796c 中途基线的 86 项门控提取）沿上一版方法
  存档于 closeout 目录。
