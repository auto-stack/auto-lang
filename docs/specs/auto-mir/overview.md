# auto-mir overview

PLAN-798：Typed MIR与独立验证器；实际位于auto-av-core，不是单独已交付crate。

当前粒度、限制和来源以[project](project.md)及所链接合同为准。函数持久产物复用已实现；完整函数级增量已由 828 交付（resident-function-install）。PLAN-820 起本模块含 av.data/1 数据 profile（data-place-profile）；PLAN-821 起含 av.owned/1 owned 访问 MIR（owned-access-profile）；PLAN-822 起含 av.collections/1 类型化集合 MIR（collection-place-profile）。

[计划来源](plans.md)；[架构决策](architecture.md)。

PLAN-824 增加 [av.dynamic/1 类型擦除 MIR](dynamic-cast-profile.md)，递归白名单和名义身份在准入/执行边界受检。

PLAN-823 增加 [av.closures/1 闭包捕获 MIR](closure-capture-profile.md)：闭包体自带 env 声明、捕获三态纪律与 link 期 env 等价检查。

PLAN-862 增加 av.entry-unit/1 入口/Unit MIR（身份 `auto.mir.av-entry-unit/1`，
旧 schema/语义版本不变）：`MirType::Unit` 零运行时宽度，只作 unit 入口的
`sig.result` 与唯一 `Return` 操作数（`mir.unit-placement`）；模块头新增
`EntryKind::{Int,Unit}` 并与入口签名**双向**校验（`mir.entry-kind`，kind 缺失
拒绝）。执行侧通道语义见 [auto-av/entry-and-unit-profile.md](../auto-av/entry-and-unit-profile.md)。

PLAN-868 起 HIR→Typed MIR lowering 的唯一实现出库为共享 crate
`experimental/auto-mir-lower`（自 av-probe 字节级纯搬移，probe/mn/cte 三消费者共用，
MIR 合同语义不变）；边界、依赖方向红线与 `MIR_LOWERING_VERSION` 版本身份见
[hir-lowering-boundary](hir-lowering-boundary.md)。
