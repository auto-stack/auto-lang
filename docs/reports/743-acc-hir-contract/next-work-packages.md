# PLAN-743 后续工作包合同候选（next-work-packages）

> 计划：PLAN-743 r1 · T-05。两张**无编号**合同候选（不分配 744/745，不伞形承诺
> 26–36 计划群）；正式立项走 `scripts/new-plan.sh` 统一取号 + `/auto-plan:new`。
> 选择依据全部回指 [inventory.md](inventory.md) 能力清单与
> [acceptance-matrix.md](acceptance-matrix.md) 锚点。

## 候选①：源码计算 adapter 及精确类型边界（NEXT-B 实体化）

- **范围**：Auto 源码子集（精确 int 域、bool、控制流、函数调用）→ Checked HIR → native，
  即 A1–A3 从"HIR 域"扩到"源码域"；产出 S0→S2 adapter 首版 + 来源映射诊断。
- **输入/输出**：输入=单模块 .at 计算子集源码；输出=CheckedModule / PE 产物 / 阶段化诊断。
- **正反语料**：正向=741 fixtures 的源码改写版（int/bool 精确域内；A8 str/List
  **不在**本候选正向集——能力不归本候选交付）；反向=未定义变量、
  类型失配、悬空引用、checked 标记越权（拒绝矩阵延续）。
- **既有 API 依赖**：`atom_text/descriptor/hir/verify`（只消费，不改语义）、
  `native::capabilities_check/find_entry/lower_object`、`link.rs` 工具链发现。
- **基线恢复门**：旧前端（ast.rs/resolver/typeck）接线**不进本候选**；若实施中需要读旧
  AST，必须先有完整 v0.5 基线恢复收据（主机器同步），否则用独立解析器子集。
- **前置能力**：int↔i32 宽度映射探针 probe-int-width、char 探针 probe-char-int
  （本候选内首个任务，裁决后回写 inventory MD-409/408）。
- **可现在做 / 须基线 / 须前置**：语料设计可现在做；实现须 741 稳定 API（已归档，满足）；
  源码域诊断须 adapter 落地（本候选交付）。

## 候选②：编译器所需类型/运行时首批（由盘点结果选定的切口）

- **选择证据**：inventory.md §4.1 三项最高优先——str（MD-406，无字符串则诊断/符号表
  不成立）、List 容器（MD-405，parser/codegen/a2r 合计 170+ 注记）、int 全序运算 + bool
  逻辑（MD-409/410，现有算术面不足）。生产 ABI 探针（原 NEXT-C 形态）不选为本候选切口：
  它与独立 ABI 线（auto-native-abi-rfc）耦合且不被 ACC 主体编译阻塞；并行推进资格保留，
  由 ABI 线自行取号。
- **范围**：str/char 语义（含跨行字面量 probe-str-ml）、内置 List\<T\>（所有权/释放语义
  probe-rt-own）、sub/div/mod/eq/ne/gt/le/ge 与 &&/||/! 扩面；HIR descriptor/profile
  升版（core-i32 → 继任 profile），741 兼容性由版本化身份保证。
- **正反语料**：A8 锚点语料（str/List 微程序，本候选交付能力后立即可建）；
  A9（enum/is-match）**不在**本候选范围——其 owner=聚合能力线（后续未编号工作线，
  前置=enum±payload/record/is 能力落地）。溢出/trap 语义在新运算上的延续（X1/X9 反例回归）。
- **与候选①关系**：能力开发可并行（不同剖面）；源码级验证依赖候选①的 S0→S2
  adapter 落地（②驮在①的管线之上）。依赖方向单向：①→②，无环。
- **基线恢复门**：无旧代码接线；纯 ac-core 域内扩展。
- **可现在做 / 须基线 / 须前置**：profile 升版设计可现在做；实现须候选①的 S0→S2 落地
  才有源码级消费者（HIR 文本级测试可先行）。

## 门槛与总估算核查（AC-07）

| 工作组（后端演进战略 §7.2） | 本次盘点后核查 | 变化 |
|---|---|---|
| Atom/HIR 与 pass 基础（3–4） | stage-contract 已定义（T-04），pass 框架未实现；3–4 维持 | 无 |
| 源码前端与计算子集（3–4） | adapter 是最大未知数（S0 全新）；3–4 维持，倾向上限 | 无 |
| 聚合/泛型/所有权（4–6） | enum payload/record/method/is 证据坐实需求；4–6 维持 | 无 |
| native 运行时与编译器库（3–4） | str+容器+int 扩面=候选②，确认前排优先级 | 无 |
| ABI/模块/热重载（5–7） | 本计划未触；维持 | 无 |
| ACC 主体迁移与自举（6–8） | 八模块审定：token/lexer/parser-core adapt、六主体新建；前端占比上移但代际判据已收敛；6–8 维持 | 无 |
| 集成与发布收口（2–3） | 维持 | 无 |

**结论**：26–36 总量与关键路径（str/容器 → adapter → 聚合 → 代际）无需因本次盘点调整；
两候选覆盖战略 §7.3 顺序中"Auto 源码计算子集跑通"与"类型/运行时扩面"两步。
估算不含自主机器码后端/全生态原生化，不重复计算 Atom 主线工程（战略 §7.2 边界照抄）。

## 明确不做（本计划边界重申）

不分配 744/745；不承诺 26–36 全群排期；不动旧 parser/VM/A2X/auto-lib；
不实施桥；不改 canonical Specs（SD-01..03 走 review/merge 沉淀）。
