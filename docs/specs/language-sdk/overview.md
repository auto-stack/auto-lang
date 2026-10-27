
# Language SDK overview

[Project contract](project.md) owns the pure non-UI language SDK. PLAN-840 fixes dependency scanning of inert comments/strings with shared lexical state; actual dependency forms and source-only closure remain intact.

Provenance: [PLAN-840](../../plans/archive/840-autographics-public-behavior-api-and-user-scripts.md), [affected final review](../../plans/attachments/840/review-r2/review.md), [complete merge receipt](../../plans/attachments/840/merge-r1/receipt.md). Graphics source: autographics-dev. Production I2/I3 remain pending.

内部模块职责 DAG/状态 owner/结构规则见[架构规范](architecture.md)（PLAN-843）；可选 Case 测试与阶段计量的当前边界见[项目规范](project.md)，实施来源见[计划索引](plans.md)。

PLAN-841：纯语言 SDK 保持无 UI 实现反向依赖；UI 上层经显式领域端口与受检 lowering 消费。 见 [project.md](project.md)。
