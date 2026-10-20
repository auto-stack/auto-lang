### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/language-sdk/project.md | 原巨型包→纯SDK职责/API/feature/显式组合与正常闭包 | 基础可独立消费 | AC-01, AC-02 |
| SD-02 | add | docs/specs/auto-ui-compiler/project.md | 原UI分析/生成→独立compiler包、公开接口/资源 | UI编译owner | AC-03 |
| SD-03 | add | docs/specs/auto-ui-runtime/project.md | 原UI运行/renderer→独立runtime与宿主/状态归属 | 运行边界 | AC-03, AC-04 |
| SD-04 | modify | docs/specs/auto-lang/project.md | old包标为上层兼容/组合，实际SDK位置明确 | 避免纯Core虚报 | AC-01, AC-02 |
| SD-05 | modify | docs/specs/auto-lang/frontend/overview.md | Parser默认/显式UI组合与旧API承载 | 解析身份/装配 | AC-02 |
| SD-06 | modify | docs/specs/auto-lang/vm/overview.md | 无UI实现的VM能力/默认组合与余边清零 | 真正解耦 | AC-01, AC-04 |
| SD-07 | modify | docs/specs/auto-lang/ui/overview.md | UI包组/旧路径/资源/消费者事实 | 完整交接 | AC-03..05 |
| SD-08 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 真实新包图/测试与资源输入进入发行profile | 迁包可分发 | AC-05 |


## 受审实施者修订提案
# F814-R4-01/R5 对账：UI 文法/宿主装配来源与合同边界（PLAN-814；R5 修订版）

> **状态（R5 修订）**：本提案是**待批准的合同修订案**，不是已对账的最终
> 形态。R5 复审纠正了本文件前版两处不实：①inventory 0.3.25 的 submit
> 宏实际生成 `unsafe extern "C" fn __ctor()` 并经 `.CRT$XCU`（Windows）
> 在启动期登记节点（[R5 源码摘录](review-r5/inventory-implementation.log.txt)）
> ——前版"无静态构造器副作用"的说法错误；②R3-01 UiCompatInstaller 的
> 行为闭合**不构成**对同类机制的授权，只说明两个收集点应按同一条原合同
> 审查。前版以"同机制已被接受"为授权来源的论证作废。

## 问题陈述（不变）

r2 §5.2/V-02/AC-02："SDK UI 解析须显式获得组合""不能靠 linker inventory
拉初始化""无隐式链接装配/双状态"；§5.1 要求 A 不先破坏旧默认 API。
当前实现中 `Parser::from(..).with_session(UI)` 直达构造的文法可用性
依赖 `UiGrammarSource` inventory 兜底（上层 `inventory::submit!` →
启动期构造器注册 → SDK 惰性收集）——与上述字面冲突。

## 约束集与不可兼得证明（诚实版）

同时要求：①直达构造（只链接上层兼容包的新进程）UI 可用（AC-02 正控，
R5 复验在案）；②SDK 清单零 auto-ui-dialect（R3-03 源码快照消费）；
③无启动构造器/inventory 注册（AC-02 原文）；④无类型身份包装/硬错误
（AC-02 旧 API 保持，R5 明确不批准破坏）。则：③排除一切"被链接即注册"
机制；①要求文法在无任何显式调用发生的进程里可用；②④排除编译期组合
与包装/报错路径。经查无同时满足四者的常规 Rust 机制（显式函数表端口、
运行时符号解析属 linker 偶然性更重，均不优于 inventory）。**本证明覆盖
已评估的常规机制，不宣称穷尽**（R5 亦未认定穷尽）。

## 修订案（需正式批准后生效）

**AC-02/V-02 条款澄清**：

1. SDK 侧不变：零 submit、零 UI 实现/清单边、表缺席=空方言回退+公开
   门面显式拒绝、无 Core 硬编码 UI factory。
2. **允许**上层兼容包/compiler 经源码中显式可见的 `inventory::submit!`
   语句注册轻量函数表（UiGrammar/UiCompatInstaller）；其生效机制为
   启动期构造器（inventory 0.3.25 `__ctor`/`.CRT$XCU`，见 R5 摘录），
   注册载荷为纯数据/纯函数指针（无 I/O、无 UI 实现拉入、无全局可变
   状态初始化），仅覆盖不经装配入口的直达构造。
3. 全部具名装配入口（run 族漏斗/旧公开入口包装/compiler 生成入口）以
   `set_ui_grammar`/安装器显式装配为第一来源；正控=显式路径，兜底仅在
   无显式路径时可见。
4. 负控保持：纯 SDK fixture（language-sdk-consumer）零提交证明 1 成立。

**若不批准**：替代路线为"直达构造未装配时改为显式错误或删除直达面"
（R5 已明确本次不批准该破坏）——即 needs_replan 破坏性变更，需用户
另行授权；或接受直达构造在纯 SDK+上层无显式装配进程中文法缺席
（AC-02 正控改判）。

## trait 归属（UiParseContext 迁 auto-parser-api）

前版称"孤儿规则下唯一合法位置"——R5 纠正：孤儿规则只约束直接 impl，
不排除适配设计（如各上层以本地 wrapper 类型实现端口——代价是端口面
在消费方间重复）。**迁移是设计取舍**：单源端口（迁轻端口 crate，旧
路径 re-export 保身份）vs 适配器重复（trait 留方言 crate）。选择前者
的理由：避免 30+ 方法端口在上层/compiler 间复制、消费方 fixture 与
grammar 单源零改动（7/7+10/10 实证）。该变更与现行
`auto-parser-api/project.md:76..84`、`auto-ui-dialect/design/full-grammar.md:28`
冲突，**作为规范变更随本修订案一并提请批准**（SD 正文见下节，T-15 受管
沉淀时不早于合同批准落 canonical）。

## SD 增量正文（A 子集，绑定 R5 批）

- **SD-01（language-sdk）**：装配端口精确语义——`set_ui_grammar`
  （显式，幂等首装优先）/`UiGrammarSource`（收集点；语义边界=修订案
  第 1-3 条）；`set_generator_fingerprint`（None→Some 升级、已装配
  首装优先；save 按缓存出生身份落盘，load 比对判失效，一侧缺席整缓存
  失效——PLAN-698 分包精确化）。
- **SD-02（auto-ui-compiler）**：`assemble_ui_grammar()` 与三个生成
  入口的显式装配时序；`generator_fingerprint()` None=foreign build。
- **SD-04/05（auto-lang/frontend）**：Scenario::UI 内建方言双层装配
  来源（显式安装器首句+批准后的注册兜底）；表缺席诊断文本
  （"not assembled ... UI grammar table"）。
- **auto-parser-api 归属变更**：UiParseContext/UiGrammar 定义入轻端口
  crate（修订案批准后为现行；re-export 保旧路径）。

## 验证绑定

批准后重验：host/parser-only/cache/autodown 四探针 + 纯 SDK 负控 +
no-ui 源码快照 + 792 driver（25/25）在合同新文本下重跑并绑定新 SHA。

## 复审整理的精确待批准增量
# PLAN-814 拟议 r3 合同增量（待用户批准，未生效）

本文件是复审准备的可审阅文本。当前主计划仍为 r2；本草案不修改其语义、canonical Specs 或 ledger，也不是阶段 pass。仅修订以下两项，其余原 AC、旧 API/类型/单份状态与 A/B 交付要求保持。

## 1. 具名的上层兼容注册例外

拟替换 r2 AC-02 / V-02 / §5.2 中有关“无隐式链接装配”的绝对约束：

> 纯 SDK 的 normal/build/dev 闭包不含 UI 文法实现、UI compiler/runtime/renderer 或 Man；SDK 不提交 UI 注册项，不硬编码 UI factory。SDK 未装配 UI 文法时的缺席/拒绝语义保持。
>
> 允许旧 auto-lang 上层兼容包及 auto-ui-compiler 经源码可见的 inventory::submit! 登记轻量 UI 文法函数表或宿主安装器的函数指针，覆盖不经显式装配入口的旧直达调用。该登记通过 inventory 启动构造器实现（当前锁定 0.3.25；Windows .CRT$XCU），会修改 inventory 自身注册表；不在启动登记阶段调用 UI 文法/宿主工厂、执行 I/O 或初始化 UI/VM 业务状态。
>
> 具名入口继续显式装配；未显式设置时才由 SDK 收集已登记来源。多个当前来源均构造自同一 auto-ui-dialect 实现，不依赖它们的遍历顺序选择不同语义。不新增不同版本同时装配的支持承诺。
>
> 上层消费者允许按其功能依赖并链接 UI 实现；不能把这一点写成“上层无 UI 实现拉入”。纯 SDK 的无 UI 源码消费者仍须成立。旧 Parser/VM API 与类型身份、单份状态、宿主默认/显式覆盖及缺席行为保持。

此例外明确承认链接时注册，不再以“显式 submit 语句”等同“调用者显式装配”，也不声称没有构造器或没有注册表状态变化。仅删除 inventory 不能自动授权砍旧 API、改成硬错误或更改纯 SDK 缺席语义。

## 2. 轻端口的归属修订

允许将 `UiParseContext` 与 `UiGrammar` 的类型/函数表定义放入轻 `auto-parser-api`；UI 文法实现继续由 auto-ui-dialect 单源拥有，SDK 不反向依赖文法实现。auto-ui-dialect 保留原公开路径再导出及类型身份，通用 `ParseContext` 的既有实现方不被强制实现 UI 端口。

这是单源端口与适配器方案之间的设计取舍，不是孤儿规则证明的唯一实现；批准后更新以下现行约束：

- `docs/specs/auto-parser-api/project.md`：原“UI 伴随端口不入本 Core 包”改为“允许 UI opt-in 轻端口定义，不带 UI 实现依赖”。
- `docs/specs/auto-ui-dialect/design/full-grammar.md`：trait 定义由方言 crate 改为 parser-api，方言 crate 保留再导出与实现 owner。
- 主计划的 SD-01/02/04/05 与上述两个 Spec 目标同步记录这一窄例外及其验收，不提前发布 B 完整 runtime 事实。

批准之后，先由 new 将主计划升为 r3 并同步相关 AC/V/SD 文本，再绑定冻结实现重新验收受影响的显式/直达/纯 SDK 正反控；阶段 pass 后仍由 merge 执行冻结门禁与实际 A 发布。未批准之前，r2 的原限制继续有效。

