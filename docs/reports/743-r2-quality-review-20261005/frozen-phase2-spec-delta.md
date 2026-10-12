# PLAN-743 Phase 2 Proposed Spec Delta（SD-04/05）

> 性质：**提案**（plan_revision 2）。canonical 落地仅在独立复审 pass 后由
> `/auto-plan:merge` 执行；本计划未写 canonical 行为内容。r1 的 SD-01..03 为已交付
> 历史（另档，不覆盖）。映射 r2 合同 §规范增量 SD-04/05 与 AC-04/05/11、AC-09/10/12/13。

| delta_id | 操作 | 目标 | before → after（规则级） | rationale | acceptance |
|---|---|---|---|---|---|
| SD-04 | modify | docs/specs/auto-acc/project.md | "以 manifest 为准的能力现状" → 补充人工层严格完整/新鲜证据（47 决定 + unknown_families 双向闭环、--require-decisions）、unknown 分类政策（QA-02）与锚点/候选 owner 一致性（A8=运行时首批候选、A9=聚合能力线） | QA-01/02/03/05/06 使主体盘点可复核；不把工具观察当语义支持 | AC-01/02/03/06/07/09/10/12/13 |
| SD-05 | modify | docs/specs/auto-hir/stage-contract.md | pass 语义保持/阶段摘要 → 明确运行期 trap 与编译期诊断界限（普通运行期表达式溢出一律保持运行期 trap；仅语言显式 comptime/const 语境可按其契约编译期诊断）、保守 effect 方向（分析与语义两分，双向禁止）、已实现/目标态边界（模板/示例为合同，无已实现 pass） | QA-04 消除常量折叠示例与保真规则冲突（X9）；不实现优化器 | AC-04/05/11 |

---

## SD-04 提案正文（docs/specs/auto-acc/project.md 修改节，替换"能力现状"节并补一段）

~~~markdown
## 能力现状（compiler-source-demand × compiled-language-support）

清单可信度以盘点工具严格门为准：`scripts/acc_inventory.py --check --require-decisions`
要求人工层（决定 + unknown_families）结构合法、绑定新鲜、对全部受管输入与全部
unknown 候选双向闭环；scan-only（无人工层）明示非完成态。

- required（ACC 前置，AC 现缺）：enum(±payload)、record、方法/static new/隐式 self、
  List<T>、str（最高优先）、char、int 全序运算+bool 逻辑、while/for/break/continue、
  is-match、use 多模块、mut 参数、全局变量、f-string（或改写）、源码域来源诊断。
- implemented（仅此两项，741 域）：i32 add/mul/lt + 溢出 trap(ExitProcess 70 测试约定)、
  HIR 域 file:line:col 阶段化诊断。741 r2/r3 的 verify 归属唯一、bindings 双射、
  CLI 更名（auto-ac-prototype）与制品原子发布属 Checked/工具域强化，不改变本清单缺口。
- unknown（必须带 owner/探针/所在工作包，open 族由严格门强制）：int↔i32/i64 宽度、
  char↔int、跨行字面量语义（probe-str-ml；合法形态，engine.at:314/333 实证）、
  宿主容器所有权（probe-rt-own）、用户自定义泛型（probe-generics）。
- not-required（附理由）：Option/Result 消费（编译器自身用字符串哨兵+自定义 record）、
  |> 管道、闭包/task/as 转型（lib 源码 0 使用）。
- 分类政策（r2 QA-02）：变量接收者的方法调用一律 unknown-receiver（本地类型可声明
  同名方法，Meter.len/CG.new/Ar.new 实证），native 仅限可证明宿主命名空间
  （List/IO/process/File）；变量接收者族的语义归属见决定 MD-507 与 unknown_families，
  实现期由名字解析/类型检查主体（新建）精确化。

## 锚点/候选 owner 一致性（QA-05）

A8（str/List 微程序）owner=运行时首批候选（str/容器/int 扩面工作包）；
A9（enum/is-match）owner=聚合能力线（后续未编号工作线，前置=enum±payload/record/is
落地）。两候选依赖单向（运行时候选的源码验证依赖源码计算 adapter 落地），无环；
不抢占新编号、不以此扩大本计划实施范围。
~~~

## SD-05 提案正文（docs/specs/auto-hir/stage-contract.md 修改节，替换"Pass 合同模板"节并补一段）

~~~markdown
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
~~~

---

## 合规自检

- SD-04/05 均为 modify：不替换现有组件、不改写 r1 交付事实，仅收紧可复核性与边界
  声明；`supersedes_spec_components` 保持空（frontmatter 的该项记录 r2 所改组件清单，
  供 merge 对账）。
- 不实现优化器/桥/ACC；不改 741 运行语义；不新增 effect lattice 冻结。
- 741 Phase 3 缺陷归 741 线；本 delta 仅在 auto-acc 标注依赖可用性限制。
