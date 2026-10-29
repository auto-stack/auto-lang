# auto-mir overview

PLAN-798：Typed MIR与独立验证器；实际位于auto-av-core，不是单独已交付crate。

当前粒度、限制和来源以[project](project.md)及所链接合同为准。函数持久产物复用已实现；完整函数级增量已由 828 交付（resident-function-install）。PLAN-820 起本模块含 av.data/1 数据 profile（data-place-profile）；PLAN-821 起含 av.owned/1 owned 访问 MIR（owned-access-profile）；PLAN-822 起含 av.collections/1 类型化集合 MIR（collection-place-profile）；PLAN-870 起 mn native 消费 av.data/1+av.owned/1 的 record/text 面（native-data-owned-lowering）。

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

PLAN-881 增加 [record/text-record 方法共享 lowering](record-method-lowering.md)：
static/view 方法降为普通 MIR 函数（owner 基 `m.<Owner>.<name>` 单元身份），
av.data/1 增按值 mut-record 写回 ABI（`t_MutRet`），copyable `.mut` 为 Copy
表示；`MIR_LOWERING_VERSION` 升 "2"（三消费者镜像同步）。

PLAN-871 起 native 消费面的**唯一后端**是共享 MIR：`ac-mn` 消费
`auto-mir-lower` 的 Typed MIR，旧 `ac-core` HIR→CLIF lowering 已删除，
`ac-core` 不再依赖 Cranelift。native 默认后端、`AC_NATIVE_BACKEND` 选择语义、
报告轴 `ac-native-backend`（linked 行必报，按行 profile 重算 stale）与
A09'/A10' cache 身份规则见
[auto-ac/native-backend-and-retirement.md](../auto-ac/native-backend-and-retirement.md)；
本案的 `MIR_LOWERING_VERSION` 仍是其后端身份轴的一部分。
