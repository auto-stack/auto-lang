# PLAN-841 R3 合法配方导入修复验收

2026-10-26，stage=work。R3 基线为 lang `6892bad1a` / UI `60011b302`；修复提交以主计划 §9 最新记录为准。

## 原因与覆盖修正

R2 的共享 file preflight 只携带了源码和模块路径，没有携带正式组件解析所需的 recipe registry 初始化。简单 widget 合法控制无法覆盖该前置条件，因此无效输入全绿的同时误拒了冷态 named/wildcard 合法配方。此次不是放宽准入，而是让预检复用正式组件入口的初始化顺序。

`validate_ui_file_with_modules` 现在先按主文件目录调用 `prepare_style_recipe_imports`，再 `parse_validated_ui`；`validate_use_modules` 对每个直接导入模块按其自身目录执行同一顺序。没有预热 registry、跳过检查或依赖文件遍历顺序。

## Finding 关闭矩阵

| finding | invariant / affected states | 永久控制 | 当前结果与边界 |
|---|---|---|---|
| F841-R3-01 | 合法本地/named/wildcard 配方在冷态 full/incremental 均可准入并生成实际 class；私有/缺失配方及前一文件残留不得误放行 | `auto-ui-tools/tests/ui_domain_validation.rs` 新增 `cold_recipe_imports_generate_actual_classes_in_full_and_incremental` 与 `recipe_preflight_rejects_private_missing_and_stale_registry_cases`；前者实际断言 full/incremental `rounded-full`，后者覆盖 private、missing 和前一文件已注册后的缺失名 | 全部通过；无已知合同内缺口。R2 无效输入/零输出、codec 完整输入、R1 深度与格式保持关闭 |

## 最终定向重放

- 原 `review841_r3_compatibility`：2 passed；local/named/wildcard 冷态 full 与 wildcard 冷态 incremental 全成功，实际组件 class 对照保持。
- 原 R2 request 工程反例：1 passed；Jet/Ark/Rust 合法成功，重复 widget 均拒绝且零输出。
- `cargo test -p auto-ui-tools --test ui_domain_validation -- --test-threads=1`：6 passed（R2 四项 + 本轮两项）。
- `cargo test -p auto-ui-compiler --test design_tokens_recipe -- --test-threads=1`：9 passed，named/wildcard/private/transitive/collision 等既有配方语义保持。
- `cargo test -p auto-ui-compiler --test ui_domain_validation -- --test-threads=1`：3 passed，R1/R2 无效领域与导入拒绝保持。
- R3 触面 rustfmt 与 `git diff --check` 通过；临时 probe 已删除。codec 未改，R2 的 9 项与完整输入证据按代码/输入不变复用。

统一裸 `cargo t`、适用 `tv/tt/tu`、完整发行/777/792 与三仓交付门仍为 final review 后的 merge-owned pending；未执行整档、未合入、未发布 canonical/ledger。
