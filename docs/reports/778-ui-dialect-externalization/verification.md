# PLAN-778 合入验证收据（ui-dialect-externalization）

- 冻结代码基线：`da17a3d44`（叠加 40c69c890 实现 + a8a81aad4 R1 修复；base 87f45dc4e）。
  本收据与 Spec/ledger 投影为文档性后代提交，实现/依赖零改动。
- 复审：R1 needs_fix（F778-R1-01/02/03）→ R2 needs_fix（F778-R2-01）→ **R3 final pass**
  （reviewed 6/7；attachments/778/review-r3/REVIEW.md，绑定 da17a3d44）。
- 验收面：外部 fixture 26/26（12 基础 + 6 R1 委托组合 + 8 R2 裸字段边界矩阵）；
  独立历史组合 7/7、R2 边界 3/3、补充组合 3/3；定向 255/255；check/fmt/diff-check exit0。
- 交付物：`crates/auto-lang/src/parser_bridge.rs`（最小桥面）、Dialect trait 改收
  ParserBridge + `DialectPriority`、三段 Arc 方言表与显式注册、派发恢复不变量
  （Err 不丢注册+五旗标恢复 / None 游标+旗标回滚 / matches 派发时门控）、
  fn/on/watch/dot-RHS 四处根源 `?` 泄漏修复、外部 fixture crate
  （`test/fixtures/modularity/ui-dialect-consumer/`，根 workspace exclude）。
- canonical Spec：SD-01 落 `docs/specs/auto-lang/frontend/overview.md`
  （新增"方言外部化"节 + 原位替换 mem::take 坑点）；frontend/plans.md 追加行；
  INDEX 重生成。
- ledger：离线投影 4 条目（designs P778-1 / tests P778-T / reviews P778-R3 /
  reports P778-M），version 18→19。
- merge-owned 门禁：裸 `cargo t` + `cargo tv` 于冻结基线（结果见计划 §9 归档记录；
  本仓 2026-10-14 裁定——work/review 阶段不跑整档）。
