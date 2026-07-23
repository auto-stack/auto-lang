# _archive/forge-relay — 旧 spec 金字塔体系（封存，只读）

> 封存日期：2026-07-23
> 备份副本：`.backup/2026-07-23/docs-specs/`（仓库根，gitignored）

## 这是什么

2026-05-12~14 期间为 **AutoForge Agents Relay** 工具建立的 spec 文档体系
（Goals/Architecture/Designs/APIs/Plans/Tests/Reviews/Reports 八类 `.ad` 文件 + `manifest.at` 索引），
是 forge-write-* 技能链的 dogfooding 产物。

## 为什么封存

- 描述的代码 `crates/auto-forge` 已于 2026-05-20 迁出本仓库（独立项目），
  designs/tests 中的路径全部指向已删除代码，状态标记失去意义。
- 该体系按"流程产物"而非"知识结构"组织，已被 2026-07-23 的
  [plan-spec 混合模型设计](../../design/plan-spec-hybrid-model.md) 取代。

## 可借鉴的遗产

- ADR 模板（决策 + 备选 pros/cons + Consequences）——已被新体系的
  `architecture.md` ADR 条目格式继承。
- reviews/reports 的"准则-结果-证据"表格模板——保留在 plan 生命周期内使用。

**请勿在此目录新增或修改文件。**
