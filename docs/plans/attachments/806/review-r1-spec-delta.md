# PLAN-806 revision 1 — R1 冻结规范增量

来源：主检出 Plan blob `8110a31fa6f232015a0dc257511c28ccb32cda04` §5。
状态：拟增量，无可绑定实施提交及 Spec 备稿，尚未核准沉淀。

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ui-dialect/project.md | 无独立生产文法包→完整msg/model及端口/依赖边界 | 由pilot进入真实文法迁移 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/frontend/overview.md | 默认内建算法→组合装配/兼容委托及真实结构/import调用 | 不漏生产内部路径 | AC-02/03/04 |
| SD-03 | modify | docs/specs/auto-lang/frontend/design/dialect-system.md | 声明分派与端口权限/恢复规约、默认与pilot策略区别 | 不改变Core和外部优先级 | AC-02/03 |
| SD-04 | modify | docs/specs/auto-lang/ui/design/scenario-dialect.md | msg/model正式外置，其他UI grammar仍上层 | 不夸大全UI文法已拆出 | AC-03/04 |
