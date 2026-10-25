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

## 函数级私有凭证（PLAN-828）

S1–S4 完整模块入口保持；828 增设函数级私有通道：`verify_function` 对单个 def 产出
`CheckedFunction`（body 检查+全 def 签名上下文，缺 body 的混合/未调用 def 在整模块
`check_module` 走查被拒——`verify.missing-body`；函数级脚手架成对合法）。凭证绑定
`context_revision`（内容规范化）与实际依赖，不从旧 CheckedModule 切片盖章；恢复走
A07 五门重验（见 [函数级源前端](../auto-ac/function-incremental-source.md)），凭证不落盘。
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
text/record 域身份（text-i32-draft/record-i32-draft）已由 770/771 注册（见下两节）。

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

### text 域阶段责任（PLAN-770，已实现·experimental）

text-i32-draft 的 S0–S6 已闭环（source-text.md 合同，实验性内部 ABI）：
S1 词法/解析按 profile 门控（str/Str/char 类型词、mut/move 模式词、文本/
字符字面量含转义与 \u{} 词法层解码校验、跨行保真）；S2/3 resolve/
typecheck 文本类型与 VT 扩面、方法按（接收者类型,方法名）绑定、遮蔽优先；
adapter 发射文本节点与 `requires: ["ac.runtime.text.v1"]`（含纯清理义务）。
S4 verify 前门见 auto-hir/project.md text 域条目；S5 native 独立 text
lowerer（(ptr,len) 双变量表示、ABI-C 拥有返回出参槽、ac_rt_* import、
TextResourcePlan 出口 LIFO 释放、短路 RHS 临时 join 块参数携带、内联探测
帧）；S6 真实 PE（runtime/text 静态库 + 合同 sidecar）。通用 pass（完整
token/lexer 编译、容器、record 文本字段）仍 planned。

## 非法变换负面清单

X1 分支 trap 提升；X2 命名参数按声明序重排；X3 变换后沿用旧 Checked 凭证；
X4 输入 checked 标记越权；X5 未知 effect 当 pure；X6 公共 HIR 泄漏低层布局；
X7 后端补做名字解析/typecheck；X8 凭证来自输入标记而非本进程 verify。

## text-record 组合域阶段责任（PLAN-782，已实证范围）

- binder 绑定组合 descriptor（record ∪ text 节点集 + record_clone/move）；
  verify 前门执行结构/类型/资源分类/借用与所有权走查（excl 定点 + 计划
  驱动能力派生；拒绝时 native 调用计数 0；结构界检先行防畸形 panic）。
- native 阶段为**合并 lowering**（组合域经 text lowerer 扩展，旧 text/
  record 路径零变化）：栈槽聚合局部、指针形参数/结果（ABI-D/E，move-in
  `*mut`）、逐出口 buffer_frees 与 RHS 后 replace_frees、跨类型创建序
  统一的语句尾/出口 LIFO、短路聚合临时存在标记（跨嵌套组合）、组合域
  status 失败部分输出 trap 前逆序回滚 + 旧目标不提交——真实 PE 实证
  （256 深/4096 循环/0 次零分配/真实故障注入）。
- 通用 pass（跨域 lowering/优化）与 ACC 桥仍 planned。
## 持久产物的信任与溯源规则（PLAN-783 探针冻结）

- **持久 payload 不携带信任**（R4/X4 的持久面）：磁盘 trusted 标志不能构造
  `CheckedModule`——探针二进制恢复路径重过本域 verify 的实测依据。
- **恢复重验义务**：任何持久 HIR（A07）消费前必须经本域 verify 入口
  （`verify`/`verify_scalar`）;探针对象/HIR 命中路径均遵守。
- **provenance 非强 pin**：对象与其 HIR 输入哈希的关联是溯源边（图 metadata）,
  不保活上游——HIR 回收后对象仍可链接运行。

## 语义事实接缝（PLAN-796，信任边界）

语义事实页（签名/字段集/绑定/来源投影）是 checked 准入的**下游事实来源**
（探针面，[auto-semantics](../auto-semantics/project.md)）：HIR verify 不改、
不创建公共 Checked 构造（唯一 verify 构造保持）；语义库的推导诊断以 typed
`diags` 步（stage=parse/typecheck/resolve）经 probe 通道回到测试判定，由父
侧按声明 completion 准入——协议/IO/资源失败仍是 ERROR，不冒充语言负例。
CT 片段（comptime 输入）只经 CTRequest/CTResult 模型接缝（SchemaOnly，
拒绝地址/宿主 handle/未检查可执行输入）；本合同不切换任何在途实现。

## 边界与已知限制

不冻结桥编码/字段编号；Effect 全集、多模块 Bundle 身份、宽度映射待实施计划裁定；
"未决即拒绝"无隐式缺省。

## 相关 plan

见 [plans.md](plans.md)。


## PLAN-798：已实现的AV消费边界

旧S0–S6及各domain保证保持。798以CheckedModule为HIR→Typed MIR消费门，直接保留类型/求值/来源/初始化语义；旧source/Atom的最长真实构造路径128预算在递归parser前受控检查，超限是探针resource，不是永久语言语法上限。前端仍整模块处理，miss仍whole MIR lower/split；安全函数级凭证及source/HIR复用归828，798不扩大旧Checked保证。见[MIR标量合同](../auto-mir/core-i32.md)。

## PLAN-818：共享 MIR 的 native 消费门（加法）

818 新后端以 798 CheckedMir 为唯一输入面：mn 消费门对（增量编辑的）每个单元
重过 798 `verify_mir` 才进入 codegen；Cranelift IR/平台 ABI 留在后端内，
**不返回公共 HIR/MIR**，不创建绕过 S0–S6 的新信任构造。旧 S0–S6 及 798
保证原样（见上节）；828 的函数级凭证/source 复用仍是唯一来源，818 只消费。

## record 域到 av.data/1 的消费支持面（PLAN-820）

record 域（auto.hir.record.draft）到 av.data/1 的实际消费面：自由函数+
i32/bool 算术+RecordCtor（源序求值→声明序映射）/FieldGet/FieldPath 赋值
（RHS 先物化）/record 按值参数返回（BorrowView 传递包装直通、BorrowMut
拒）/Method 含 static 显式拒（lower.method）；前端 compile_record_source
前门复用（4MiB/128 深）。非扩大：core/scalar 域消费面不变。V-02/05/06、
semantics 族 13/13。