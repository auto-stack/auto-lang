# auto-mir overview

PLAN-798：Typed MIR与独立验证器；实际位于auto-av-core，不是单独已交付crate。

当前粒度、限制和来源以[project](project.md)及所链接合同为准。函数持久产物复用已实现；完整函数级增量已由 828 交付（resident-function-install）。PLAN-820 起本模块含 av.data/1 数据 profile（data-place-profile）；PLAN-821 起含 av.owned/1 owned 访问 MIR（owned-access-profile）；PLAN-822 起含 av.collections/1 类型化集合 MIR（collection-place-profile）。

[计划来源](plans.md)；[架构决策](architecture.md)。

PLAN-824 增加 [av.dynamic/1 类型擦除 MIR](dynamic-cast-profile.md)，递归白名单和名义身份在准入/执行边界受检。
