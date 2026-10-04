# auto-hir stage contract

> **Status**: planned（契约层；实现状态逐阶段标注，不与实现混淆）
> 契约全文：docs/design/strategy/auto-acc-bootstrap-contract.md（PLAN-743 T-04）

公共 HIR 的阶段边界与 pass 合同模板，版本化演进；741 现状是其 S1–S6
core-i32 实现状态的事实来源。

## 阶段表（S0–S6）

S0 源码工作态（未实现）→ S1 Schema-bound Atom 文本（已实现）→ S2 bound Bundle
（已实现，单模块）→ S3 semantic verify（已实现）→ S4 Checked 公共 HIR
（已实现，core-i32 域）→ S5 lowering+能力门（已实现，测试 profile；通用 pass 框架
未实现）→ S6 发射/链接/运行（已实现，测试 profile）。

规则：R1 阶段身份随行、禁止跨阶段取用；R2 来源映射自 S1 起随行；R3 effect 只能
收窄；R4 失败=显式诊断；R5 显式 Dynamic 与推导失败不同态；R6 多模块装配不改变
verify 全或无语义。

## Pass 合同模板

id/version、input/output stage、profiles、pre/postconditions、order、
semantics_preserved（精确宽度/溢出、短路、调用顺序、place/rvalue、trap 不提升）、
effects_traps（未知 effect=observable 保守）、call_order、drops、source_maps、
analysis_invalidation、reverify（S4→S4 required）、diagnostics。
合同示例：norm.canonical-form、eval.const-fold（文本合同，非实现）。

## 非法变换负面清单

X1 分支 trap 提升；X2 命名参数按声明序重排；X3 变换后沿用旧 Checked 凭证；
X4 输入 checked 标记越权；X5 未知 effect 当 pure；X6 公共 HIR 泄漏低层布局；
X7 后端补做名字解析/typecheck；X8 凭证来自输入标记而非本进程 verify。

## 边界与已知限制

不冻结桥编码/字段编号；Effect 全集、多模块 Bundle 身份、宽度映射待实施计划裁定；
"未决即拒绝"无隐式缺省。

## 相关 plan

见 [plans.md](plans.md)。
