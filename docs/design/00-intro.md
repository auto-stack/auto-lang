# Auto Language 设计文档总览与索引

> 本文档是 `docs/design/` 的总入口，索引树内**全部**文档并标注分类与状态。
> 整理历史：2026-06-15 首次体系化（14 章版）→ 2026-08-28 Plan 467 全量重整 →
> 2026-08-28 Plan 468 需求级文档去序号化、按模块归位 →
> 2026-09-04 Plan 543 建立知识库生命周期与 current/target-state 分层。

---

## 知识资产分层（先读这个）

完整职责和 canonical-source 矩阵以
[Design 27：Knowledge Base Lifecycle](27-knowledge-base-lifecycle.md) 为准。目录层仍保持
Design / Specs / Plans 三个主入口，ADR 位于模块架构文档，Generated Catalog 由后续工具生成。

| 层 | 目录 | 职责 | 回答的问题 |
|---|---|---|---|
| **设计层** | `docs/design/`（本目录） | 意图、方案、取舍、历史素材 | "为什么这样设计 / 打算怎么做" |
| **账本层** | `docs/specs/` | 现状知识（project/module 树 + 6 段 spec ledger） | "现在是什么样、关键入口在哪" |
| **过程层** | `docs/plans/`（active + `archive/` 438 篇） | 一次开发任务的完整时间线叙事 | "某次改动是怎么做的" |

各层互相**引用**不复制：设计文档描述意图并链接 specs 现状；specs 描述现状并溯源 `(plan-NNN)`；
plans 记录过程并在收尾时经 `/auto-plan:merge` 把知识沉淀回 specs（范式详见
[Design 27](27-knowledge-base-lifecycle.md)、[autoplan-spec-ledger.md](autoplan-spec-ledger.md)
与 [specs/README.md](../specs/README.md)）。

---

## 新增设计文档的归位规则（Plan 468 确立）

写新设计文档前先定性：

| 定性 | 判据 | 去向 |
|---|---|---|
| **域级章** | 覆盖一个大的技术域、长期伞形视图（如类型系统、UI 架构、知识生命周期） | 根级拿号（下一可用章号 **51**——39–44 为编译器语法、cache、ABI、语义与AC/AV及数据域章；45 为 AutoGraphics；46 为跨生态 CAP；47 为 AutoUI 应用架构；48 为 UI 领域模型与 lowering 终态；49 为 native 单源 MIR lowering；50 为 AutoWeb 服务框架与仓库边界），在本索引登记 |
| **需求级/专题设计** | 服务于某条需求线或专题（特性、轨道、规范、研究输入） | 进对应模块子目录（`autoui/`、`blueprints/`、`strategy/`…），**slug 命名不带号** |
| **流程体系类** | 开发范式、知识体系、流程设计 | 根级不拿号（与 `plan-spec-hybrid-model.md`、`autoplan-spec-ledger.md` 同列） |

**封存号说明**：17–19、21–26 已被历史文档用过并随 Plan 468 归位（对应关系见各子目录），
**不再复用**——历史文献中的"Design NN"靠各文档头部的归位注记溯源。
注：`autoui/desktop-shell.md`（Design 25，曾用名 AutoShell/25-autoshell-dsl-unified-shell）已归位转正式
`autoui/`，届时 25 号一并封存。

---

## 分类总览

文档数量不再作为手工维护的架构事实；当前数量和目录清单应由后续 Generated Catalog 生成。

| 分区 | 数量 | 内容 |
|---|---|---|
| 一、语言核心 | 10（01–10） | 从源码到执行的语言本体设计 |
| 二、应用框架与生态 | 5（11–15）+ 1 子目录 | Shell / 并发 / 网络 / 开发工具 / AI 基础设施；标准库运行时方案见 Design 33，文档与在线体验线设计归 `documents/` |
| 三、AutoUI 与 App 生成域 | 2 章 + 2 子目录 | 域级章 16/20；需求级设计归 `autoui/`（10）与 `blueprints/`（4） |
| 四、流程与知识体系 | 1 个域级章 + 2 个根级专题 | Knowledge Lifecycle、AutoPlan 账本、Plan+Spec v1 |
| 五、战略路线图 `strategy/` | — | 生态版图伞形（三市场框架）/ Rust 脚本层 / 消费者 parity / C·Web·鸿蒙·Mobile·ROS2·Native·第三代互联网(Godot) 生态 / Python parity / Rust 库复刻 |
| 六、专题诊断（根级） | 4 | 方言体系诊断、VM 调试、ASH 设计总览（外仓主题）、[管道算子 `\|>`](pipe-operator.md)（讨论稿，Plan 514 衍生） |
| 七、附录 `forge/` | 5 | AutoForge（已迁 auto-os 生态） |
| 八、历史素材 `raw/` | 67 | 早期原始设计草稿（只读参考） |
| 0 | — | 原在途 `25-autoshell-dsl-unified-shell.md` 已归位为 `autoui/desktop-shell.md` |

> 历史文档曾维护人工“对账公式”；该公式随目录演进已失效，不再作为完整性门禁。

---

## 一、语言核心（01–10）

```
源码 (.at)
  ├─ 词法/语法 ──→ 10-language-syntax.md
  ├─ 类型系统 ──→ 02 / 03 / 04
  ├─ 编译管线 ──→ 01 / 09
  ├─ 数据结构 ──→ 07
  └─ 执行后端 ──→ 05（VM 解释）/ 06（a2c/a2r 转译）
```

| # | 文档 | 主题 |
|---|------|------|
| 01 | [Architecture](01-architecture.md) | 编译器管线、核心组件、AIE 增量编译 |
| 02 | [Type System](02-type-system.md) | 类型修饰符、推断、泛型、字符串、枚举、联合 |
| 03 | [Error Handling](03-error-handling.md) | Option/Result/Panic、May\<T\>、后缀操作符 |
| 04 | [Memory & Ownership](04-memory-ownership.md) | view/mut/move 三元组、hold、存储修饰符 |
| 05 | [VM & Runtime](05-vm-runtime.md) | ABC ISA、ART、AutoVM、MicroVM、并发模型 |
| 06 | [Code Generation](06-code-generation.md) | a2c、a2r、a2ark、a2jet、AutoGen、ASTL、FFI |
| 07 | [Data Structures](07-data-structures.md) | Node、Atom、Obj、ListData、存储式列表 |
| 08 | [UI Systems](08-ui-systems.md) | AURA、场景编程、设计令牌、前后端通信 |
| 09 | [Compiler](09-compiler.md) | AIE、AutoCache、DCE、comptime、CLI、AI 原生 |
| 10 | [Language Syntax](10-language-syntax.md) | 点号表示法、函数、位运算、OOP、模块系统 |
| 46 | [CAP System 跨生态架构](46-context-profile-capability-cross-ecosystem.md) | Draft 0.2讨论重建：场景/profile/能力推导、求解、库/包裁剪、ACE及backend；ACE与基础AutoGraphics完成后实施 |

## 二、应用框架与生态（11–15）

| # | 文档 | 主题 | 备注 |
|---|------|------|------|
| 11 | [Shell Tools](11-shell-tools.md) | AutoShell/ASH、coreutils、SmartCmd、管道 | 主体已迁独立仓 `../auto-shell` |
| 12 | [Concurrency](12-concurrency.md) | Task/Msg Actor 模型、async/await、.go 运算符 | |
| 13 | [Networking](13-networking.md) | HTTP Server 标准库、async I/O、JSON/url | |
| 14 | [Developer Tools](14-developer-tools.md) | LSP、Web Playground、AutoLab、MCP Server | Playground 细化设计归 [documents/](documents/README.md) |
| 15 | [AI Daemon Infrastructure](15-ai-daemon-infrastructure.md) | AutoOS 共享 LLM Harness、并发仲裁、Key Vault | 跨仓主题，实现在 auto-os 生态 |
| 28 | [Rust Interop Architecture](28-rust-interop-architecture.md) | Auto↔Rust 互操作：类型信息生命周期铁律、三时刻能力模型、编译预言机原则、430 管线实勘、双轨迁移模式 | 2026-09-07 立档；源 musk VM 轨 JSON 桥追问的全链定案，PLAN-591 的架构依据 |
| 31 | [AutoScape / AutoWeb](31-autoscape-autoweb.md) | Auto 原生浏览器与统一应用寻址运行时：URI/Intent/AppResolver、AppRealm/AppSession/AppProvider、AutoFrame/BrowsingContext、AutoWeb 部署模型、远程语义流；含术语×本地实现对照与差距分析 | 📝 Draft（2026-09-20 立档；外来设计输入融入重排版，v0.6 主打候选 M1 见其 §13） |
| 33 | [Stdlib Runtime & HTTP](33-stdlib-runtime-and-http.md) | 标准库多后台实际装配、AutoUI API 拓扑、Axum/VM owner 与 HTTP/task 分阶段路线 | 696–738 主线及 751/755 修复已交付；v0.6 下一项 776 返回资源/异步终值 drafting；剩余四阶段见进度审计 |
| 34 | [VM Store 所有权与 Handler 阻塞纪律](34-vm-store-ownership-and-handler-discipline.md) | 决策工件：段执行/actor 双形态下的 store 所有权（Erlang vs Go 式）与 handler 阻塞纪律（类型化 vs ui_lint+段切预算）两案对比、迁移面评估与裁定建议 | 📝 决策工件（PLAN-702 T-07 产出，待用户裁定；不含实现） |
| 50 | [AutoWeb服务框架与仓库边界](50-autoweb-service-framework-and-repo-boundaries.md) | 独立auto-web仓首期：Web/语言/Man/UI职责、HTTP生成与运行、测试发行、Blog消费及885/887交接 | target-state r1（2026-10-29）；884→887→Web→UI/Man；仅设计，887后正式立迁移Plan，未建仓/实施 |

**模块子目录**：[documents/](documents/README.md)——文档与在线体验大模块（website/书籍/Playground
线的需求级设计，slug 命名不带号；域级概要章待目录整体重组时补立，2026-09-07 用户裁定）。

在线体验演进：[Website 在线体验与 UI Playground](documents/website-interactive-experiences.md)——2026-10-01 用户确认方向；v0.5 介绍/截图、v0.5.1 Web 桌面/app、Auto→Vue 预览和分批数据适配，待实施。

## 三、AutoUI 与 App 生成域

**域级章**：

| # | 文档 | 主题 |
|---|------|------|
| 16 | [App Generation & AI Authoring](16-app-generation-and-ai-authoring.md) | Rung 1–5 分层战略、AI 生成工作流（域战略伞） |
| 20 | [AutoUI Separation Architecture](20-autoui-separation-architecture.md) | AutoUI 分离架构：a2ui/ui_gen/host 分层（域架构锚点） |
| 29 | [AutoUI Style & Theme System](29-autoui-style-theme-system.md) | 样式抽象与主题系统：ThemeRegistry 单一值源（partial override/extends 派生/hot-switch）+ style recipe 编译期组合宏；取代原 Design 19 值层（2026-09-09 立档） |
| 32 | [AutoUI Repo Extraction](32-autoui-repo-extraction.md) | AutoUI 仓分离总方案（三件套+双插件缝）：方言插件（Dialect trait 补完）+ auto-man cargo 式插件缝 + 五阶段拆仓路线 + 函数级迁移勘定账。📝 方案稿（2026-09-20 立档待评审；DEBTS `拆仓-候选` 登记，下一版规划实施） |
| 38 | [AutoUI Features & System Services](38-autoui-features-and-system-services.md) | Widget / Blueprint / Feature / App 与横向 System Service：功能包命名、分散 UI 贡献、scoped 命令/状态、裁剪与跨 App 复用、服务治理、编辑/知识能力纵向抽取。📝 调研与架构草案（2026-10-14；未冻结/实施） |
| 47 | [AutoUI State, Design & Feature Architecture](47-autoui-state-design-and-feature-architecture.md) | 基于 v0.5.0 的应用架构调研：状态所有权/通信/事务与历史、保状态热重载、类型化设计体系与 Figma/DTCG、Feature 多出口复用。📝 RFC（2026-10-22；仅分析设计，独立仓迁移后再实施） |

**需求级设计子目录**（各篇头部有归位注记，保留原 Design NN 对应）：

- [`autoui/app-presentation-host`](autoui/app-presentation-host.md)（App 呈现宿主：Region/Section/Surface 声明、Feature UI 目标绑定、灵活布局与整页栏目，Design 38 前置设计，2026-10-14 draft）

- [`autoui/`](autoui/README.md)（需求级设计 + 索引）：shared-store(原18)、theming(19)、
  examples-app-track(21)、base-styles(22)、virtual-desktop(23)、desktop-shell-and-launcher(24)、
  a2ui-composer-analysis(原25)、image-viewer-pipeline(Plan 547)、025-gap-enumeration(16a)、
  **vm-frame-budget(VM 帧预算/空转渲染，PLAN-650 设计层)**、
  **vm-loading-responsiveness(VM 加载响应性、CPU 段预算与 Init 调度，PLAN-708 r2 提议)**、
  **component-time-and-events(嵌套组件时间源/多层交互契约，PLAN-652 设计层)**、
  **universal-scroll-architecture(通用滚动架构 ScrollPane/ScrollContent 契约，设计先行 draft；Phase A+B+C 已立项 PLAN-656)**、
  **editor-kernel(编辑器内核：rope 单写者版+统一 delta+back 分块读，PLAN-673 T-00 设计先行)**
  ——**L8 留架历史文档（归档不回改）；新增桌面程序设计一律落 auto-os
  `docs/design/`**（Stage B P-5 指针登记，2026-09-07；首例即 auto-os Design 01
  Stage B 迁移定案，资产去向见 [docs/plans/INDEX.md](../plans/INDEX.md)）
- [`autoui/autoui-mcp-test-fixture`](autoui/autoui-mcp-test-fixture.md)（AutoUI MCP 测试夹具协议，Plan 623）
- [`autoui/reader-runtime-prerequisites`](autoui/reader-runtime-prerequisites.md)（Reader 框架前置设计：原生验收→child-anchor→10MiB有限执行；PLAN-781/r2，单计划三阶段，drafting）
- [`blueprints/`](blueprints/blueprints-first-class.md)（4 篇）：blueprints-first-class(原17) 为 Blueprint 层主设计（PLAN-639 更名，原 blocks/），
  配套 agent-generation-workflow / blueprint-package-format / datasource-convention

**在途**：`25-autoshell-dsl-unified-shell.md`——桌面 Shell 的 AutoUI 统一层（细化 Design 23/24，
服务 463/464/465 shell-track），稳定后归位 `autoui/`。

## 四、流程与知识体系

| 文档 | 主题 | 状态 |
|------|------|------|
| [27-knowledge-base-lifecycle](27-knowledge-base-lifecycle.md) | current/target-state、canonical source、ADR/Spec/Plan/Generated Catalog 生命周期 | ✅ **现行 canonical design**（Plan 543） |
| [35-autodown-test-cases](35-autodown-test-cases.md) | AutoDown 统一文件用例、同源多后端 oracle、执行/比较/覆盖协议与渐进迁移 | 有限文件化基础788/852已交付；a2r日常金样/独立认证分轴，完整消费者覆盖待扩 |
| [模块选测与ACE成组执行](testing/module-selection-and-ace-batching.md) | 复用887义务清单建立模块/影响闭包、ACE独立与成组/组合表达式对照、逐项定位与实测推广 | target-state r1，PLAN-888草案；建议U887后优先质量支线，未实施，不取消全局冻结门 |
| [36-ac-data-resource-expansion](36-ac-data-resource-expansion.md) | AC 文本/标量 record 扩面：类型与资源契约、内部 ABI 探针、并行分支与集成门 | 774/770/771/782有限合同已归档；本篇为历史共同设计，现状查Spec |
| [37-v06-modularization-and-repo-boundaries](37-v06-modularization-and-repo-boundaries.md) | v0.6 模块/仓界、宿主与项目模型解耦、结构/减法及交付门 | 831/832与841–844已归档；867等价出口在途，869/872/873及两个未编号结构包见[总路线v11](../plans/reports/v06-extraction-execution-roadmap.md) |
| [39-ac-incremental-syntax-and-language-platform](39-ac-incremental-syntax-and-language-platform.md) | Auto语言工具基础设施：Auto/Atom Grammar、CST/AST/HIR分层、ACE编译部署与插件、auto-edit/编码工具及可选语义LSP；复用增量/cache/有界工作集 | 📝 2026-10-22补长期定位与P1–P4路线；787有限实验已交付，完整通用产品未交付；本次仅设计 |
| [40-compiler-artifacts-and-cache-lifecycle](40-compiler-artifacts-and-cache-lifecycle.md) | 编译全阶段产物目录、HIR/MIR/ABC等查询/产物/页/装载四种粒度、持久格式与依赖验收；共享store/lease/预算及自动GC | 📝 目标态合同草案（2026-10-20补§3.2–3.6决策基线；参数/codec迁移待Plan冻结，现状查producer Spec） |
| [41-interface-metadata-and-foreign-abi](41-interface-metadata-and-foreign-abi.md) | AutoABI与C/Rust等foreign ABI分层；共用接口元信息/Foreign IR、use.c自动绑定与resolver、双向插件、target语义与cache/lease接线 | 797已有Windows x64 AutoABI v0.1候选及有限Auto/C互调；完整foreign生态仍为目标 |
| [42-semantic-metadata-inference-and-comptime](42-semantic-metadata-inference-and-comptime.md) | v0.6语义meta/类型推导/泛型与AV驱动CTE；双期反射、runtime保留、统一查询/依赖及可缓存冻结结果 | 796/817有限语义与AV标量CTE已交付；完整泛型/meta/双期反射待扩面 |
| [43-ac-av-typed-mir-and-abc](43-ac-av-typed-mir-and-abc.md) | ACE的AC/AV/AJ、Typed MIR/ABC/CTE与函数增量 | 📝 §6.4/6.5：798完成后优先828；817/818、820–827扩展须保持全链路函数增量 |
| [44-av-data-layout-place-and-ownership](44-av-data-layout-place-and-ownership.md) | AV布局、Place/owned、集合/闭包/Dyn；820–824扩面与825/826性能分工 | 📝 先798交付→优先828→其余ACE扩展；各Plan强制保留全链路增量，实际能力查Spec |
| [45-autographics-wgpu-engine-and-game-prototypes](45-autographics-wgpu-engine-and-game-prototypes.md) | wgpu 共同图形基础、2D/3D 游戏原型与生产集成边界；§13为M2有界运行对象/触发/新局事务拟议合同（882→883） | M1已交付；M2角色/相机与879纯迁仓已交付、源码权威auto-ui；880在途、882→883规划，生产AutoUI/HIR/ACE仍待相应成熟度门 |
| [autographics/m2-editor-and-public-game-prototypes](autographics/m2-editor-and-public-game-prototypes.md) | M2：编辑视口/游戏/Gallery、跨端与归仓门 | 设计v8；874/877有现场在途变化，875/878待交付，先核交接快照 |
| [autographics/m2-runtime-and-character-control](autographics/m2-runtime-and-character-control.md) | 公共角色/相机、同tick事务与并行消费/审计 | 设计v6；874→875与877/878依赖/写面保持，不由WIP推断交付 |
| [autographics/m2-gallery-and-capability-matrix](autographics/m2-gallery-and-capability-matrix.md) | C01..17、首12项顺序/难度/资源与平台维度 | 设计v4；877 executing/内容树已建，尚无内容交付证明 |
| [autographics/platform-architecture-and-research](autographics/platform-architecture-and-research.md) | 跨端/DOM/Canvas边界、XP-0..5研究步骤 | 设计v3；878仍drafting，其余候选，跨端未交付 |
| [autographics/tracking-handoff-2026-10-28](autographics/tracking-handoff-2026-10-28.md) | 长会话续接：已交付/在途状态、四Plan、并行树与固定pin、Gallery/跨端/归仓、M1标签身份差异 | 带日期快照；新会话先核最新Plan/owner，不接管或重复建树 |
| [autographics/repository-move-and-integration-gates](autographics/repository-move-and-integration-gates.md) | auto-ui承接AG实验源码与生产UI/ACE融合的分离门、时机和切源要求 | 候选建议：875实际交付后纯归仓可早于ACE；878只读评估，不启动迁仓 |
| [48-ui-domain-ast-and-lowering](48-ui-domain-ast-and-lowering.md) | UI owned领域模型、泛型领域解析、受检lowering与Core准入 | PLAN-841 架构合同 |
| [49-native-single-mir-lowering](49-native-single-mir-lowering.md) | 共享HIR→Typed MIR→native收敛、五旧profile覆盖/删除门、布局/身份/cache交接 | 868已交付；870/871待实施，不提前删除旧能力；保留verify资源分析 |
| [autoplan-spec-ledger](autoplan-spec-ledger.md)（原 26） | auto-plan 四技能范式 + 6 段账本 + 路径映射 | ✅ **现行**（开发范式权威描述） |
| [plan-spec-hybrid-model](plan-spec-hybrid-model.md) | Plan+Spec v1 五环流程（2026-07-23） | ⚠️ 已被取代（诊断与反模式清单仍有效） |

## 五、战略路线图（`strategy/`）

2026-10-16跨路线阶段入口：[v0.6总体矩阵](../roadmap-v0.6.md#7-v06路线阶段与拆分边界矩阵2026-10-16)。以下专题只补目标态，当前能力仍查Spec；候选不自动成为发布门禁，机器人算法暂不纳入，下一条独立路线待讨论。

| 专题 | 归位与复用 | 状态 |
|---|---|---|
| [system-foundations-v06](strategy/system-foundations-v06.md) | 通信/AutoAI/DevTools与Lens接缝；引用auto-os既有RFC/F0–F5，不复制协议权威 | 阶段/验收/依赖补充，未实施 |
| [autographics-forge-v06](strategy/autographics-forge-v06.md) | 第三代互联网下的场景/组合、创造工具与承载游戏 | M1独立交互原型已交付；近期顺序转M2战略，生产融合后议 |
| [autographics-m2-v06](strategy/autographics-m2-v06.md) | AG M2桌面/Gallery与静态跨端审计、归仓候选 | 战略v8；874/877在途、875/878待交付，11–15份桌面估计与来源风险见交接 |
| [knowledge-learning-v06](strategy/knowledge-learning-v06.md) | 复用AutoDown/Jade，论文、scope/来源/检索与学习 | 有限原型候选，完整流程v0.7 |
| [autodown-typesetting-v06](strategy/autodown-typesetting-v06.md) | 复用统一文档引擎，补布局/增量/资源与adapter实验 | 调研/技术原型候选，未实施 |
| [input-method-v06](strategy/input-method-v06.md) | 引擎/会话/平台/候选窗职责与独立宿主 | 调研/原型候选，未实施 |
| [platforms-v06](strategy/platforms-v06.md) | 复用鸿蒙/Mobile/Web，补Linux和真实组合验收 | 保留平台目标，支持集待Plan冻结 |

| 文档 | 主题 | 状态 |
|------|------|------|
| [autoshader-minimal-language-and-renderer-interface](strategy/autoshader-minimal-language-and-renderer-interface.md) | 小范围AutoShader：ACE受检语义、有限CAP与WGSL/wgpu产物/参数/资源接口；830只留接缝 | 📝 Draft 0.1（2026-10-22；仅设计，未实施） |
| [cross-ecosystem-programming-strategy](strategy/cross-ecosystem-programming-strategy.md) | 面向场景编程、CAP命名与CPU/MCU/GPU跨生态路线；ACE→基础AutoGraphics→CAP→扩面 | 📝 Draft 0.2讨论重建（2026-10-22；未实施） |
| [ecosystem-portfolio-strategy](strategy/ecosystem-portfolio-strategy.md) | **生态版图伞形总战略**：动静结合第一优势（全生态设计原点）+ 三大市场（需求侧 3 + 供给侧 1）+ 寄生期合作层原则 + 优先级总表与不做清单（各分战略入口） | 📝 Draft（2026-09-06） |
| [v0.6-offline-development-strategy](strategy/v0.6-offline-development-strategy.md) | v0.6 提前开发的历史基线、合并边界、近期任务与当前状态 | 规划基线 v1（2026-10-04；具体实现另行评审） |
| [modularization-and-refactoring-strategy](strategy/modularization-and-refactoring-strategy.md) | 拆分后有限治理、完整模块粒度、测试/规模/语义/测量口径与OS真实消费接续 | 2026-10-28交接；867在途，869/872/873待门，876待review/merge；不重开已交付搬仓 |
| [demo-app-independence-strategy](strategy/demo-app-independence-strategy.md) | 教学 Demo 与独立产品分流、来源同步、子模块组织与产品孵化 | 首轮建仓/接线完成；产品功能与全量验收待推进 |
| [unified-package-install-strategy](strategy/unified-package-install-strategy.md) | auto-man 系统级包网络、安装核心与实体生命周期 | 目标已确认，技术设计草案（v0.6 必做） |
| [atom-batom-v2-design](strategy/atom-batom-v2-design.md) | 新 Atom/Batom：独立于 auto-val 的模型/内存、Schema、struct codec、Node/Lisp 文本、自描述与紧凑二进制 profile | 📝 Draft RFC（2026-10-04；新方向，格式/计划未冻结） |
| [atom-schema-dsl-design](strategy/atom-schema-dsl-design.md) | Atom Schema DSL、独立描述与标注、组合/fragment、primary/secondary、业务 enum 与身份 | 📝 Draft RFC（2026-10-04；语法与分期建议待确认） |
| [auto-hir-design](strategy/auto-hir-design.md) | AutoHIR 语义优先：结构化类型表示、引用身份、求值顺序、校验与后端边界，再映射 Atom Schema | 📝 第一版语义 Draft（2026-10-04；未冻结/实现） |
| [auto-hir-atom-text](strategy/auto-hir-atom-text.md) | HIR Atom 文本映射与完整样例：双槽简写、类型化引用、函数、控制流、参数求值顺序及反例矩阵 | 📝 Draft（2026-10-04；尚未经 parser/verifier 验收） |
| [auto-ac-subset-and-target](strategy/auto-ac-subset-and-target.md) | AC 计算核心、文本/数据工具与 AAC 子集分期；Windows x64 原生目标文件、链接与执行闭环 | 📝 Draft（2026-10-04；Cranelift/自举分期已确认，具体子集待验收） |
| [auto-native-backend-evolution](strategy/auto-native-backend-evolution.md) | **ACE架构及AAC/AAV规划总入口**（原AC/AV/AJ总路线）；§0汇总Design39–44、增量/cache/热重载和block演进、AV程序/替换路线与Plan核验流程 | 路线方向已确认；2026-10-20补ACE称谓与导航，技术合同与交付证据按专题/Spec/Plan核实 |
| [ace-implementation-handoff](strategy/ace-implementation-handoff.md) | ACE长会话决策、有限交付基线、862–866/870/871任务及S/N/H交接 | 2026-10-28续接快照；current Spec与revision绑定交付收据为准，附新会话prompt |
| [auto-acc-bootstrap-contract](strategy/auto-acc-bootstrap-contract.md) | ACC 自举主体与公共 HIR 阶段契约：S0–S6 阶段表、pass 合同模板与非法变换反例、后端桥候选矩阵、Gen1–3 代际判据（PLAN-743 T-04） | 📝 Draft（2026-10-04；契约未冻结，实施计划按节认领） |
| [auto-native-abi-rfc](strategy/auto-native-abi-rfc.md) | AC/ACC 原生 ABI：独立版本域、Windows x86-64、静态/动态模块、linker 分工、跨界类型、常驻运行时与状态热重载 | 📝 Draft RFC（2026-10-04；需求已记录，技术建议待评审，未实施） |
| [auto-as-rust-script-strategy](strategy/auto-as-rust-script-strategy.md) | "Auto 作 Rust 脚本层"宣传与文档纲领 | ✅ Accepted（Plan 359） |
| [rust-library-replication-roadmap](strategy/rust-library-replication-roadmap.md) | Rust 库复刻验证路线（parity 语料） | ✅ 现行（Plan 347/348/369） |
| [python-parity-roadmap](strategy/python-parity-roadmap.md) | Python parity 第三维度（use.py + a2py） | ✅ 现行（Plan 369 首批落地） |
| [consumer-parity-strategy](strategy/consumer-parity-strategy.md) | Auto 作为库消费者（consumer-mode） | 📝 Draft |
| [c-ecosystem-strategy](strategy/c-ecosystem-strategy.md) | C ABI互操作、Embedded C、Portable C Backend；保留历史三消费面部署规划，与AutoABI/Foreign IR接线 | 📝 Draft（2026-10-15 §12更新；原a2c工程暂缓/重启条件见§11，新ABI架构不受旧source-only口径限制） |
| [web-ecosystem-strategy](strategy/web-ecosystem-strategy.md) | Web 生态：契约独立 + Vue 主臂 + React 发射臂扩展时序 | 📝 Draft 骨架（2026-09-06，待扩充） |
| [harmonyos-ecosystem-strategy](strategy/harmonyos-ecosystem-strategy.md) | 鸿蒙：ArkTS 主线（仓颉/ABC/机器码不做）+ OpenHarmony OS 终态方向 | 📝 Draft 骨架（2026-09-06，待扩充；a2ark rebaseline 为该线首个 plan） |
| [mobile-jetpack-strategy](strategy/mobile-jetpack-strategy.md) | 移动：Android+iOS 统一 Jetpack Compose（不做 SwiftUI），排鸿蒙后 | 📝 Draft 骨架（2026-09-06，待扩充） |
| [ros2-ecosystem-strategy](strategy/ros2-ecosystem-strategy.md) | AutoOS通信先行、ROS2边界兼容；旧C绑定方案留档，机器人算法暂不纳入 | 📝 2026-10-16补充分期/验收，完整兼容集待冻结 |
| [native-backend-strategy](strategy/native-backend-strategy.md) | Native 战略：当前 AC/ACC/Cranelift 分期 + 历史自主生态/开发形态/fork-rustc 论证 | 2026-10-04 文首对齐新路线；旧阶段顺序保留为历史 |
| [thirdgen-internet-strategy](strategy/thirdgen-internet-strategy.md) | 第三代互联网（AI+XR）：AutoGraphics/创造入口；Godot参考/可选实验，旧静态缝合论证留档 | 📝 2026-10-16对齐后续讨论，阶段见autographics-forge-v06 |
| [script-mode-interop](strategy/script-mode-interop.md) | 脚本模式互操作（AutoScript .as 糖→py 桥 lowering/错误模型/null 体系） | 📝 Draft（裁决闭环，待立项） |
| [av-language-capability-matrix](strategy/av-language-capability-matrix.md) | 43项AV语言/内核/库/产品能力、当前计划后缺口、语义迁移差异与证据分母 | 2026-10-20正式目标矩阵；未实施，不等完整语言 |
| [av-source-programs-and-avm-replacement-roadmap](strategy/av-source-programs-and-avm-replacement-roadmap.md) | 真实Auto程序G0–G8、23种子、入口/SKU迁移、默认/退役门及优先级；首个Plan827 | 2026-10-20路线/计划草案，未运行或切默认 |

## 六、专题诊断（根级）

| 文档 | 主题 |
|------|------|
| [dialect-extension-diagnosis](dialect-extension-diagnosis.md) | 方言/语法扩展体系结构性问题诊断与改进方案 |
| [vm-debugging](vm-debugging.md) | VM 调试方法论（ABC 汇编格式起步） |
| [ash-design-summary](ash-design-summary.md) | AutoShell(ASH) 设计总览（外仓主题，文首有新旧计划号对照） |

## 七、附录 `forge/`（AutoForge，工具本体已迁 auto-os 生态仓）

[spec-categories](forge/spec-categories.md) · [spec-driven-forge](forge/spec-driven-forge.md) ·
[spec-ui-and-relations](forge/spec-ui-and-relations.md) ·
[forge-specs-relay-frontend](forge/forge-specs-relay-frontend.md) ·
[agents-relay-orchestration](forge/agents-relay-orchestration.md)

> forge/ 的 Spec 分类思想（Goals/Architectures/Designs/Plans/Tests/Reviews）被
> [autoplan-spec-ledger](autoplan-spec-ledger.md) 的 6 段账本继承。

## 八、历史素材（`raw/`，67 篇）

早期未经整理的原始设计草稿，是 01–15 章的素材来源，**只读保留**（个别被外仓引用，
如 auto-down 仓引用 `raw/auto-down.md` 的方言三逃逸符）。按文件名主题检索即可。

---

## 设计演进时间线（要点）

```
2024 Q1–Q4  语言核心：AST/VM/类型系统 → a2c/a2r → 泛型/枚举/模式匹配 → 内存模型
2025 Q1–Q4  UI 系统(AURA/a2ark/a2jet) → Shell → 并发(Task/Actor) → AIE/AutoCache
2026 Q1–Q2  ASH 分层架构 → LSP/MCP → 设计文档 00–15 章首次体系化(2026-06-15)
2026 Q6–Q8  App 生成战略(16) → AutoUI 域需求设计族(17–25, 后归位 autoui/blueprints)
            → 分离架构(20) → 虚拟桌面三部曲(23/24) → 范式收敛(Plan 467/468)
2026 Q9     documents 大模块目录建立：Playground 在线体验设计归位子目录（design 树
            「根级章=大块+模块子目录」结构化的起点，整体重组待议）
            → 09-09 样式与主题系统立档（29：ThemeRegistry + style recipe，
            取代原 Design 19 值层）
            → 09-20 AutoScape/AutoWeb 域章立档（31：ChatGPT 外来浏览器设计融入
            重排版 + 本地差距分析/v0.6 分期建议）
```

## 如何使用本目录

- **新开发者**：01（管线）→ 10（语法）→ 感兴趣领域的章节。
- **贡献 AutoUI/桌面**：16（战略）→ 20（架构）→ [autoui/](autoui/README.md) 各专题
  （虚拟桌面线：virtual-desktop → desktop-shell-and-launcher → desktop-shell[25]）。
- **查现状/关键代码入口**：去 [docs/specs/](../specs/overview.md)（本目录存意图，specs 存现状）。
- **查某次改动的来龙去脉**：去 [docs/plans/](../plans/archive/plans-status-audit-2026-08-20.md)（最新全量审计；2026-09-01 起由 [Plan 513](../plans/archive/513-repo-integration-cleanup.md) 接任）。
- **新增设计文档**：按"归位规则"节先定性再落位，写完在本索引登记。

