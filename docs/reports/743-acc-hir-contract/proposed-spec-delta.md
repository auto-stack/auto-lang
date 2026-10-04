# PLAN-743 Proposed Spec Delta（SD-01..03）

> 性质：**提案**。canonical 落地仅在独立复审通过后由 `/auto-plan:merge` 执行；
> 本计划未写任何 canonical Spec 文件。映射计划 §5 规范增量表与 AC-07。

| delta_id | 操作 | 目标 | before → after（规则级） | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-acc/project.md | 无独立 ACC 合同 → 以明确主体/依赖/代际判据记录约定目标；implemented/required 分开 | 防止旧 AAVM/AA2R 自举与新 native 自举混称；给后续实施计划可引用的现状锚点 | AC-01, AC-03, AC-06 |
| SD-02 | add | docs/specs/auto-hir/stage-contract.md | 741 单 profile 事实 + 战略阶段意图 → 版本化阶段/pass 模板及各阶段实现状态/不变量 | 为 S0→S2 adapter、pass 框架、桥接提供可验证边界（引用 T-04 契约） | AC-04, AC-05 |
| SD-03 | modify | docs/specs/auto-hir/project.md | 无 stage-contract 入口 → 新增关联入口一行；其余现状/非目标/profile 声明原样保留 | 索引可发现；不把设计描述写成实现 | AC-04, AC-07 |

---

## SD-01 提案正文（docs/specs/auto-acc/project.md，新增）

~~~markdown
# auto-acc

> **Status**: planned（目标态合同；AC=首版原生编译器，ACC=Auto 实现的编译器主体）
> 主体清单与取证：docs/reports/743-acc-hir-contract/（PLAN-743）；
> 阶段/桥/代际契约：docs/design/strategy/auto-acc-bootstrap-contract.md

ACC 主体的约定目标、能力现状与自举判据。防止"Auto 写的编译器"统称掩盖
AAVM/AA2R 自举（已达成谱系）与 AC/ACC native 自举（未达成）两条线。

## 主体清单（现状：adapt/reference 分层）

- adapt（代码可迁移）：auto/lib/token.at（词法判据面）、lexer.at（游标扫描核心）、
  parser.at 游标/Pratt 核心（产物 S-expr 层除外）。
- reference（不搬迁）：typeinfo.at（Display 推断非 typeck 凭证）、codegen.at（ABC
  发射非 native 管道）、engine.at（ABC 解释器=oracle）、a2r.at（AA2R 转译谱系）、
  auto/aavm.at（CLI 形态参考）。
- 新建（现不存在）：名字解析、typeck、源码→HIR adapter、语义 lowering+能力门、
  驱动+来源诊断、后端桥客户端。
- 非主体保留：Cranelift 0.126 桥、rust-lld/link.exe、kernel32 启动包装、
  宿主内建 File/IO/process/print/List（所有权语义待 probe-rt-own）。

## 能力现状（compiler-source-demand × compiled-language-support）

- required（ACC 前置，AC 现缺）：enum(±payload)、record、方法/static new/隐式 self、
  List<T>、str（最高优先）、char、int 全序运算+bool 逻辑、while/for/break/continue、
  is-match、use 多模块、mut 参数、全局变量、f-string（或改写）、源码域来源诊断。
- implemented（仅此两项，741 域）：i32 add/mul/lt + 溢出 trap(ExitProcess 70 测试约定)、
  HIR 域 file:line:col 阶段化诊断。
- unknown（探针 owner 待立项）：int↔i32/i64 宽度、char↔int、跨行字面量语义、
  宿主容器所有权、用户自定义泛型。
- not-required（附理由）：Option/Result 消费（编译器自身用字符串哨兵+自定义 record）、
  |> 管道、闭包/task/as 转型（lib 源码 0 使用）。

## 代际判据

Gen1=Rust AC 编译 ACC 第一代；Gen2=第一代自编译→第二代，与 Gen1 语义/诊断/HIR 等价；
Gen3=不动点。字节固定点仅在可复现条件约定后为附加门；A2R 转译中转不算 native 自举。

## 边界与已知限制

- ACC 主体清单以 743 盘点 manifest 的受管输入为准；输入漂移需重审。
- 不宣称：完整 Auto 语言面、全生态原生化、自主机器码后端（v0.7 线）。
- 旧 AAVM/AA2R 绿语料不构成 AC/ACC implemented 证据。

## 相关 plan

见 [plans.md](plans.md)。
~~~

## SD-02 提案正文（docs/specs/auto-hir/stage-contract.md，新增）

~~~markdown
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
~~~

## SD-03 提案正文（docs/specs/auto-hir/project.md，修改）

在文末"相关 plan"节前插入一节（其余内容零改动）：

~~~markdown
## 阶段契约

公共 HIR 的阶段边界、pass 合同模板与非法变换负面清单见
[stage-contract.md](stage-contract.md)（planned；实现状态以本文件现状节为准）。
~~~

---

## 合规自检

- SD-01/02 为纯新增组件，SD-03 仅新增交叉链接且明示"实现状态以现状节为准"——
  无现有组件被替换，`supersedes_spec_components` 保持空。
- touched_goals 为空：GOAL-017（旧 AAVM/AA2R 自举）不改写、不重开；native 自举目标
  入账另走目标治理。
- 本计划未修改 canonical Specs / 旧源码 / 741 profile / 外仓（AC-07 边界）。
