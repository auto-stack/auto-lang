# auto-hir stage contract

> **Status**: planned（契约层；实现状态逐阶段标注，不与实现混淆）
> 契约全文：docs/design/strategy/auto-acc-bootstrap-contract.md（PLAN-743 T-04）

公共 HIR 的阶段边界与 pass 合同模板，版本化演进；741 现状是其 S1–S6
core-i32 实现状态的事实来源；PLAN-772 追加 scalar-i32 域的阶段事实。

## 阶段表（S0–S6）

S0 源码工作态（PLAN-745 的 auto.source.core-i32.draft 与 PLAN-772 的
auto.source.scalar-i32.draft 已实现；源码 → span token/AST → 词法/类型解析；
scalar 面含新运算/一元/短路与条件 for）→ S1 Schema-bound Atom 文本（已实现；
两源码 profile 各自生成文本及来源表；**按文档头三重身份选择已注册
descriptor**，未知身份 bind.descriptor-unknown，程序化路径对未注册
descriptor 整体拒绝 bind.descriptor-unregistered，不猜测不回退）→
S2 bound Bundle（已实现，单模块；Bundle 携带 HirDomain 身份）→
S3 semantic verify（已实现；按域入口 verify/verify_scalar，profile 边界与
引用/签名类型边界门先于走查）→ S4 Checked 公共 HIR（已实现，core-i32 与
scalar-i32 域）→ S5 lowering+能力门（已实现，测试 profile；通用 pass 框架
未实现）→ S6 发射/链接/运行（已实现，测试 profile）。

源码入口与全 Bundle 来源投影见 [source-core-i32.md](../auto-ac/source-core-i32.md)
与 [source-scalar-i32.md](../auto-ac/source-scalar-i32.md)。
verify 仍为 Checked 唯一构造路径；通用 pass 框架未实现状态不变。

条件求值规则（PLAN-772）：短路 RHS 只能发射进条件执行块（分支 + 变量合流），
不允许无条件调用/执行可能 trap 的操作；命名实参先按源码顺序求值，再按
bindings 落位；逻辑表达式可用于任何实参/返回/初始化/赋值/条件位置。
`for cond {}` 在源码域降级为 `loop { if cond { body } else { break } }`
（每轮重判、条件不 hoist），不新增低层控制流模型。

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

### 数据/资源阶段责任（PLAN-774，planned）

S3 verify 承担数据/资源前门：str/Str 逐语境与借/拥有八码（含被拒操作 native
调用计数为 0 的义务——八码均已**探针模型可达入口并通过**，production verify 仍
planned）；S4 Checked 须携带资源义务（所有权/清理责任）且不泄漏 ABI 布局
（沿 X6）；S5 lowering 按 D-05 五类形态（含嵌套 record 递归布局）；S6 执行器
deadline/containment 沿 741 合同。全路径清理（LIFO/嵌套出口/临时/move-in/
return 转移）的证据状态为**探针实证**；合同全文见
[auto-ac/data-resource-contract.md](../auto-ac/data-resource-contract.md)。
text/record 域身份（text-i32-draft/record-i32-draft）注册归 770/771。

### record 域阶段责任（PLAN-771，已实现）

record-i32-draft 的 S0–S6 已闭环（source-record.md 合同）：S1 binder 绑定
record/field/method 结构与 CategoryRef；S3 verify 前门执行 774 D-04 借用/
所有权规则（拒绝时目标操作 native 调用计数为 0）、构造完整性与全部表
引用界检；S5 native 按 D-05 布局（字段声明序、4 对齐、size=Σ字段尺寸，
checked u64）与 ABI-D（`*const` view）/ABI-E（`*mut` + out-slot 返回，无
聚合 RAX/sret）lowering。**存储不重叠义务**（ lasting rule，771 R1/R2
冻结）：copy-elision 仅在赋值目的地与 RHS 触集不相交时直写目的地，否则
RHS 先完整物化再提交；record 值操作数（构造字段绑定的 ReadLocal）在其
自身求值位捕获，借位实参传活指针；out-slot 与全体实参存储不相交由调用
点转发规则归纳保证。通用聚合优化、公开/生产 ABI 保持 planned。

## 非法变换负面清单

X1 分支 trap 提升；X2 命名参数按声明序重排；X3 变换后沿用旧 Checked 凭证；
X4 输入 checked 标记越权；X5 未知 effect 当 pure；X6 公共 HIR 泄漏低层布局；
X7 后端补做名字解析/typecheck；X8 凭证来自输入标记而非本进程 verify。

## 边界与已知限制

不冻结桥编码/字段编号；Effect 全集、多模块 Bundle 身份、宽度映射待实施计划裁定；
"未决即拒绝"无隐式缺省。

## 相关 plan

见 [plans.md](plans.md)。
