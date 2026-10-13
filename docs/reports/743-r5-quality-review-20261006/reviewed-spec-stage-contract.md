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

规则：R1 阶段身份随行、禁止跨阶段取用；R2 来源映射自 S1 起随行；
R3 效果分析与效果语义两分——分析自保守侧出发（未知 effect 按 observable 处理，
向 pure 精化须携带证明），效果语义保持对任何 pass 双向禁止（不得引入/消除/重排
可观察效果，含 trap 与其前驱 observable 的顺序）；运行期 trap 与编译期诊断界限及
反例 X9 见"Pass 合同模板"节；R4 失败=显式诊断；R5 显式 Dynamic 与推导失败不同态；
R6 多模块装配不改变 verify 全或无语义。

## Pass 合同模板

id/version、input/output stage、profiles、pre/postconditions、order、
semantics_preserved（精确宽度/溢出、短路、调用顺序、place/rvalue、trap 不提升）、
effects_traps、call_order、drops、source_maps、analysis_invalidation、
reverify（S4→S4 required）、diagnostics。
合同示例：norm.canonical-form、eval.const-fold（文本合同，非实现）。

### 运行期 trap 与编译期诊断界限（QA-04）

普通运行期表达式（语言未规定必须编译期求值者）的常量溢出一律保持运行期 trap
语义——无论位于必经路径还是条件分支，pass 不得将其提升为编译期拒绝或编译期
诊断（反例 X9：`mark_a(); return MAX_I32+1` 的约定语义是先观察 a 再
ExitProcess(70)）。仅当语言独立规定某语境必须编译期求值（显式 comptime/const
声明域）时，该语境按自己的契约编译期诊断；其规则属该语境，不归语义保持 pass。
不可达/未执行分支中的 trap 不得提升（X1）。

### 保守 effect 方向（分析与语义两分）

静态效果*分析*从保守侧出发：未知 effect 一律按 observable 处理，向 pure 的
精化必须携带证明——这是分析精度，不是程序改写。效果*语义保持*对任何 pass
双向禁止：不得引入原本不存在的可观察效果，也不得消除/重排已存在的可观察效果
（含 trap 与其前驱 observable 的相对顺序）。不用"收窄/放宽"描述语义保持义务。

### 实现状态边界

阶段表"现状"列是实现状态的唯一权威；pass 模板与两合同示例为合同要求
（目标态），当前不存在任何已实现 pass。声称实现任何 pass 的计划必须先按
模板补全可执行 pre/post 断言并重验。

## 非法变换负面清单

X1 分支 trap 提升；X2 命名参数按声明序重排；X3 变换后沿用旧 Checked 凭证；
X4 输入 checked 标记越权；X5 未知 effect 当 pure；X6 公共 HIR 泄漏低层布局；
X7 后端补做名字解析/typecheck；X8 凭证来自输入标记而非本进程 verify。

## 边界与已知限制

不冻结桥编码/字段编号；Effect 全集、多模块 Bundle 身份、宽度映射待实施计划裁定；
"未决即拒绝"无隐式缺省。

## 相关 plan

见 [plans.md](plans.md)。
