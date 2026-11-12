---
plan_id: PLAN-980
status: drafting
feature_name: 现有AutoUI全模式ACE兼容接入
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-12
plan_revision: 8
current_step: 0
total_steps: 35
depends_on: [PLAN-960, PLAN-963, PLAN-967, PLAN-968, PLAN-969, PLAN-972, PLAN-973, PLAN-976, PLAN-978, PLAN-979]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-autoui.md]
touched_goals: [GOAL-002]
---

# [PLAN-980] 现有AutoUI全模式ACE兼容接入

## 0. 变更摘要

2026-11-12 r8：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-11 r7当前：增加HELLO首个真实窗口I/V/R/D先行批：消费967/F0、979/H0、969/E0、978/B0最小正式接口，从单text静态及Auto函数文本到AV真FFI先交、原RUST批同源新A2R并行；不等待STATE全量async、973 File、WEB/TS或完整PARITY，保原九批全传统UI义务。 原31项任务/全部AC/V保留，新增4项早期切片任务，现35项、drafting、0完成；只规划。

历史修订：2026-11-10 r6执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r5当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-09 r4：传统AutoUI完整支持是合并AGE M2必需门，不能以2D/3D场景、少量控件或新Node语法替代；保9批31任务，补AC-07及完整分母/纯传统UI的AV+A2R验收，drafting、0/31，仅规划。


建立旧widget/model/msg/on/store/props/computed/theme/actions/routes/setup/watch等到Schema领域IR映射，消除handler合成旧bytecode。保持现有Vue/VM-Iced/Rust/Tauri/jet/ark的真实支持格；TS-only行为不默认为全端已有支持。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
传统widget/store工程可保原源码直接经ACE，不强制先改Node；向981普通view提供共同组件/布局/主题/状态/事件接面。传统UI完整迁移归本包，完整Node语言/新旧组合归981/LANGUAGE/FUSION，二者均为AGE M2必需门，不另立重复接入计划。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

正式依赖以frontmatter并集为准；各批最小接口前置见§5，稳定接口只允许受控准备，最终须正式交付来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

2026-11-11本轮授权：用户要求规划首个ACE UI Hello及AV/Rust FFI路线、核对既有M2冲突并作必要重排；仅规划/设计/导航，不实施、创建worktree或探针/fixture/SDK、构建/运行、提交或发布，未指定模型/工期。沿旧语言用例守恒、新Auto标准库与AutoABI/FFI、Demo递进的三线，不削完整M2范围。

现场语言HEAD 9e36d984e、UI HEAD d88e9c51。已核av-core/src/host.rs仅console/1（sha256前16 30d6dcc475850cd0）、a2x-rust/src/provider.rs仍拒source foreign（3ea453dc2452576f）、core/src/source/cst/frontdoor.rs（dd3d93d06799ae52）；现有auto-bindgen/src/extractor.rs是预置C manifest表（8c7f1295cc83c6fb），不能称Rust元数据导入器已实现。Iced由Rust宿主编译执行，AutoVM不解释其Rust源码；现有DynamicComponent/VmBridge/handler_codegen深依赖旧执行链。

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

本轮授权细化批次和跨仓依赖、必要时拆计划，先不施工。已核各仓Cargo、源码和测试入口：UI/Man根无AGENTS；Web守中立contract/公开SDK方向；OS域正文按当地AGENTS归OS064；没有主检出实现/测试授权。
2026-11-09用户进一步要求M2支持传统AutoUI及完整AGE语法。本轮仅修订979/980/981和路线，不实施/构建/运行或创建新Plan/实施树；980原模式/组件/兼容义务保留，不能以后继981替本包交付。本次语言HEAD `b4f1af0be67b91b601345a4284ecd4036cb1a91e`；下文11-06来源为历史。


用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `D:/autostack/auto-ui/crates/auto-ui-dialect/src/ast.rs`（本轮确认存在；开工重核符号/版本）。
- `D:/autostack/auto-ui/crates/auto-ui-runtime/src/ui/vm_bridge.rs`（本轮确认存在；开工重核符号/版本）。
- `D:/autostack/auto-ui/crates/auto-ui-runtime/src/ui/handler_codegen.rs`（本轮确认存在；开工重核符号/版本）。
- 本次读auto-ui `docs/specs/ui/structure-closure.md`、`runtime/component-and-store-instances.md`和`WidgetDecl`实际字段（含lifecycle/design/style/ext_imports等）；完整分母不能只取首段关键词。共同作者方向见auto-ui `docs/design/autographics/{declarative-authoring-model,multi-presentation-selection}.md`；不把设计草图当已实现。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：完整AutoUI声明/状态/所有运行模式接入。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [auto-ui/crates/auto-ui-dialect/src/ast.rs](../../../auto-ui/crates/auto-ui-dialect/src/ast.rs)：完整WidgetDecl，不另定义小组件全集；读取SHA256前16 `1d10f5ff2566422e`

- [auto-ui/crates/auto-ui-runtime/src/ui/vm_bridge.rs](../../../auto-ui/crates/auto-ui-runtime/src/ui/vm_bridge.rs)：替旧执行依赖，中立组件接面；读取SHA256前16 `9b6f212b63a7747c`

- [auto-ui/crates/auto-ui-runtime/src/ui/iced/renderer/projection.rs](../../../auto-ui/crates/auto-ui-runtime/src/ui/iced/renderer/projection.rs)：复用Iced呈现，Rust不解释guest逻辑；读取SHA256前16 `dcd9c869ff3873ce`

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, B-DIALECT-I, B-DIALECT-V, B-DIALECT-R, B-STATE-I, B-STATE-V, B-STATE-R, B-NATIVE-I, B-NATIVE-V, B-NATIVE-R, B-WEB-I, B-WEB-V, B-WEB-R, B-RUST-I, B-RUST-V, B-RUST-R, B-TAURI-I, B-TAURI-V, B-TAURI-R, B-JET-I, B-JET-V, B-JET-R, B-ARK-I, B-ARK-V, B-ARK-R, B-PARITY-I, B-PARITY-V, B-PARITY-R, T-05, T-06, T-07, T-HELLO-I, T-HELLO-V, T-HELLO-R, T-HELLO-D |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


本Plan各批独立实施/验证/复核，同一唯一worktree保留至全包归档；局部review不冒正式D。frontmatter为整包正式依赖并集，最小前置如下；LNNN=auto-lang/PLAN-NNN。下游可用冻结SHA准备，最终须已合入来源。共享文件单写，不建第二权威tree或链接。

唯一组D:/autostack/.wt/lang-980/，各实际改动仓一个兄弟tree；OS/App用OS064当地规约。

| 批次 | 交付 | 最小前置 | owner |
|---|---|---|---|
| 980/HELLO | 单text普通源的AV→Rust FFI→真实Iced；同源A2R由RUST批并行 | 967/F0-D、979/H0-D、969/E0-D、978/B0-D正式来源；不等完整STATE/973/PARITY | auto-ui主责，语言守公共能力 |
| 980/DIALECT | 旧UI语法与Schema投影 | L979/L960/L963 | auto-ui |
| 980/STATE | 实例Store handler与异步状态 | 980/DIALECT; L967/L968/L969 | auto-ui |
| 980/NATIVE | Iced窗口输入与资源 | 980/STATE; L973/L978 | auto-ui |
| 980/WEB | Vue与TS/JS表达式后端 | 980/DIALECT; L976/L972/L978 | auto-ui |
| 980/RUST | Rust UI静态工程 | 980/DIALECT; L978 | auto-ui |
| 980/TAURI | Tauri桌面工程 | 980/WEB; L978 | auto-ui |
| 980/JET | Jet目标工程 | 980/DIALECT; L978 | auto-ui |
| 980/ARK | Ark目标工程 | 980/DIALECT; L976/L978 | auto-ui |
| 980/PARITY | 主题控件Demo及模式联合覆盖 | 980/NATIVE; 980/WEB; 980/RUST; 980/TAURI; 980/JET; 980/ARK | auto-ui |

### 传统AutoUI完整性与AGE M2硬门

T-01及DIALECT/PARITY建立逐项分母：现行WidgetDecl、Store/主题/样式/组件语法和普通Auto表达式→Spec/原源位置→模式/平台适用格→ACE producer/runtime/实际oracle。至少核model/msg/on、props/emit/bind/computed、use/多文件、named_views/slot、keyed条件/列表挂载的既有语义、routes/setup/watch/expose/actions/menubar/toolbar/timer、lifecycle/design/style/theme/ext_imports及全部现用组件/布局/资源/错态；再与真实grammar/AST/Specs双向对账，未被Demo覆盖的必需语法/变体补永久case。可解析或首帧可见不等于实际运行支持。

纯传统UI工程（无view-2d/view-3d声明）必须经共同前门实际AV运行，并经新A2R生成/编译执行UI逻辑。保原文字选择/复制、输入/IME、焦点/捕获、主题/布局、表单/列表/滚动、路由、事件、异步/卸载/late、App Store隔离、资源/关闭恢复。NATIVE负责AV开发、普通body编辑不调用Cargo/rustc；RUST负责生成状态/handler/路由代码的实际dispatch，不用Rust宿主内嵌AutoVM/AV冒A2R。原widget源码不先改Node或生成旧脚本。

WEB/TAURI/JET/ARK保各自现用模式的全部实际支持格，不把TS-only或未承诺平台能力默称所有后端齐；必需平台缺环境保持pending，不删格。DIALECT/STATE交981版本化组件/状态/事件/生命周期接口；980不依赖981完整Node或图形增强终验，981消费本包，防PARITY↔LANGUAGE环。AGE M2须同时通过传统UI门和完整Node/融合门，图形小例不能抵消缺口。

### 980/DIALECT：旧UI语法与Schema投影
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-dialect/src/ast.rs`；`D:/autostack/auto-ui/crates/auto-ui-compiler/tests/ui_domain_validation.rs`。
I实现：按WidgetDecl全部字段逐项投影/明确模式适用性，schema→typed领域IR→ACE行为；字段覆盖表包含named_views/bind/routes/setup/watch/expose/actions/timer/theme。
V验收：普通标识符上下文保持；旧widget原源解析/绑定/错误span正确；坏字段拒绝，缺字段不能悄悄忽略。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 980/STATE：实例Store handler与异步状态
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-runtime/src/ui/vm_bridge.rs`；`D:/autostack/auto-ui/crates/auto-ui-runtime/src/ui/handler_codegen.rs`；`D:/autostack/auto-ui/crates/auto-ui-runtime/tests/plan041_instance_ownership.rs`。
I实现：用ACE执行服务替旧handler bytecode生成/调用，接组件props/emit/computed与App store、挂起/唤醒、卸载和late generation。
V验收：双App各两组件不串；异步任务返回只写仍存活实例；MIX-03无HTTP版本用真实异步host，HTTP实腿由983/联验补齐。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 980/NATIVE：Iced窗口输入与资源
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-tools/src/run_vm.rs`；`D:/autostack/auto-ui/crates/auto-ui-runtime/tests/desktop_behavior.rs`；`D:/autostack/auto-ui/crates/auto-ui-runtime/tests/image_surface_contract.rs`。
I实现：接原生run入口、路由/页面/主题/媒体/terminal等宿主；实际旧vm参数可保兼容名字但dispatch为ACE。
V验收：001原文字选择复制/主题，页面切换、输入法/捕获、媒体/终端、关闭重开与资源回收逐项实测；不是首帧即通过。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 980/WEB：Vue与TS/JS表达式后端
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-tools/src/run_vue.rs`；`D:/autostack/auto-ui/crates/auto-ui-tools/src/vue.rs`；`D:/autostack/auto-ui/crates/auto-ui-compiler/src/ui_gen/ts_adapter.rs`。
I实现：Vue生成使用新TS/JS provider，保watch/setup/expose/ext_imports/style及绑定；服务端使用已交宿主接口或隔离真实fixture，不等整个983才能证UI表达式。
V验收：真实浏览器挂载/事件/绑定与源expected一致；setup只一次、watch卸载停止；生产表达式不走旧TypeScriptTrans。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 980/RUST：Rust UI静态工程
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-tools/src/rust_ui.rs`。
I实现：先按§5 AXR0切片运行同源单text V-UI-AXR0并独立复核/正式交接；随后多文件store/组件与sidecar经新A2R生成，保持原Rust UI行为和增量产物；旧AST emitter无回退。
V验收：未改生成物冷编译运行，props/msg/route与原独立oracle相符；坏源保旧工程。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 980/TAURI：Tauri桌面工程
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-tools/src/tauri.rs`。
I实现：接新TS/JS与Rust backend生成，保持前后端invoke/事件/权限边界；旧AST emitter无回退。
V验收：真实Tauri窗的invoke和事件、失败诊断及重开，不能仅浏览器前端通过。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 980/JET：Jet目标工程
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-tools/src/jet.rs`。
I实现：保持现有Jet生成模式，以受检UI领域IR供目标adapter；不新增原未承诺能力；旧AST emitter无回退。
V验收：原支持工程在实际Jet工具链构建运行，事件/状态独立oracle，缺平台未执行。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 980/ARK：Ark目标工程
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-tools/src/ark.rs`。
I实现：接Ark领域投影和表达式adapter，保现有Ark工程/资源/模块输出；旧AST emitter无回退。
V验收：原支持工程在真实Ark工具链执行，原widget/事件/状态正确，缺平台未执行。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

### 980/PARITY：主题控件Demo及模式联合覆盖
改面（核存在）：`D:/autostack/auto-ui/crates/auto-ui-runtime/tests/gallery_golden.rs`；`D:/autostack/auto-ui/crates/auto-ui-runtime/tests/theme_scope_isolation.rs`；`D:/autostack/auto-ui/docs/specs/ui/examples-ownership.md`。
I实现：从001开始按文字布局→表单列表滚动→菜单Blueprint→媒体编辑分小组；同一覆盖表列全部现用组件/变体/模式，未被demo覆盖的错态补永久case。
V验收：主题scope与原画面/交互/生命周期对照，UI058等AGE新能力不是本包替代；每格source/backend/runtime/行为收据齐。
R关闭：独立核真实输入→新producer→consumer及正反例/来源/资源/退出；新fixture由I接真实target，缺工具/环境保未执行。

各批I的有界决定门：用该批最小真实正例、同类型非法近邻、一次失败后恢复跑最长producer→consumer链；仅对未决策略比较适用实现再冻结接口，不拿日历/语法深度缩范围。资源由session持候选、generation持已发布产物、在途pin持旧代，last-owner释放；错误/取消/late不得覆盖CURRENT。继承原预算/平台条件，新增资源单位/上限/超限分类在本批I冻结并有边界反例，改变承诺先修合同。

### HELLO：首个ACE UI text窗口的早期批次（当前最小入口）

HELLO归M2-A必要能力验证，不是B全产品迁移；不用等待M2A-08、981或AGE M2总签。原九个完整能力批次继续，新增HELLO使本包十批、35任务。最小正式来源：967/F0-D（AV/Rust受检互操作）、979/H0-D（最小widget/text及普通函数表达式受检投影）、969/E0-D（公开ACE AV同步run/授权/关闭）、978/B0-D（最小宿主/adapter构建闭包）。任一未落主只可准备，不以分支tip作产品证据。

首源拟新增在auto-ui既有examples/ui下独立`ace-hello-minimal/src/front/app.at`及必要显式项目配置（T-HELLO-I创建）。第一输入为一个widget根、一个view、一个text字面量；第二输入仍一个text，内容来自普通Auto函数返回的中文/ASCII字符串。任意合法widget/函数名及改文字都须工作，不硬编码Hello。该验证工程不修改/精简001；原001含themes包/model/design/col/selectable，后续按DIALECT/STATE/NATIVE/PARITY完整迁移。新增fixture不自动进入旧全部Demo分母。

路径为普通源→H0包/typed投影→唯一verify→AV通过E0执行文本表达式和F0受检调用→UI拥有的Rust text provider→现有Iced文字投影/主线程窗口。静态描述可直接供宿主布局，但HELLO-V还必须观察AV真实F0 dispatch与同一文本字节被provider接收；只把描述交Rust硬画文字不能关闭AV/FFI门。UI公共Auto facade声明/foreign要求归领域包，967生成/验证通用binding/shim；宿主Rust内部调用Iced合法，不把iced::Element/Rust trait/String私有布局塞进AV值。

UI text provider在返回前复制借用文本到当前UI session所有的Rust值，再在OS主线程构造/刷新Iced文字；它不能持有已失效AV视图。字体/窗口/布局由Rust宿主持有，guest尚无事件回调/状态模型；关闭窗口/取消释放当前session，旧请求/撤权不得写新窗口。F0输入上限/错误规则沿967，程序与宿主失败分态；不扩guest异步、跨线程callback或Rust泛型语义。

同源A2R首窗属于原B-RUST-I/V/R的AXR0先行切片，另消费967/T-12与978/T-03/09的AXR0正式来源；同一作者源还须经新A2R生成Rust、实际编译、链接同一逻辑UI provider并真实显示；若采用直接crate adapter须证明与F0 ABI adapter等价的字节/错误/owner规则。guest业务不能由AV/旧VM解释或Rust手写替代。HELLO-I/V/R/D只闭AV首窗全部V-HELLO-*即可交接，AXR0的V-UI-AXR0结果由B-RUST复核/phase fold另交；AV不反等A2R。完整980仍须原AV/A2R与全部模式，单text不证明按钮/组件状态、主题/选择复制/全UI语法已支持。

既有owner改面：auto-ui `crates/auto-ui-tools/src/run_vm.rs`、runtime `src/dynamic_build.rs`、`src/ui/{dynamic,vm_bridge,handler_codegen}.rs`与`src/ui/iced/renderer/{host,projection}.rs`；ACE分流由E0公开入口，旧组件支路不能被新候选隐式调用。新最小ACE组件adapter/consumer测试在原runtime/tests owner创建，拟`tests/ace_ui_hello.rs`；公共运行入口参数由I冻结，不虚构当前有--engine ace。仅窗口host可继续依赖Iced，ACE core/AV不得反依UI crate。 T-HELLO-I先核现有run_dynamic_iced的DynamicComponent具体类型及其VmBridge字段耦合，用至多两种机制比较共同Component/视图消费接口与最小ACE组件adapter；必须复用现有Iced投影/窗口服务，必要时抽出中立宿主接面，不能创建假旧VmBridge或另写一次性Iced Hello绕开AutoUI链。该决定/最小正例/同类非法近邻/失败后恢复先闭再扩实现；无法隔离旧执行依赖则具名needs_replan。旧代码是否仍被其他产品编译与Hello新链是否dispatch旧VM分开记录，全面不可达/删除仍由987终验。

本批交接顺序：最小依赖各自D→HELLO-I/V→独立R→正式D，原完整DIALECT/STATE/NATIVE/RUST等复用其代码/用例继续扩大；不重复第二UI引擎。纯文字H0/F0基础与完整语言/新库路线可以并行，共享grammar/ABI文件短窗单写。

Demo后续序列：静态text→函数/状态文本→单按钮计数→双组件隔离→原001主题/文字交互完整保真→其余原Demo按依赖逐项扩展→完整Node/2D/3D。编号不是能力依赖；每个实际Demo列需要的语言/库/宿主格。Demo未覆盖语法/错态/模式补永久case；Hello过不勾原九批或完整RG-A03。

早期切片是本Plan M2-A内部交接，不新增ACE里程碑或Plan终态；只在同一权威树按owner流程独立复核、正式落主并记source/接口/grammar/ABI/provider及真实结果后供下游消费。同Plan尚有原完整义务时不归档/删树/勾整包完成。frontmatter保最终依赖并集，早期切片只等本节最小接口；同一文件单写，不以分支reviewed/WIP当交付。最小切片通过不关闭原AC、完整Rust FFI/全UI/RG-A03/M2A-08；不新增对980/981/985整包或AGE联合门的反向依赖。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r8当前）

**具体输入与期望。** 原件[auto-ui/examples/ui/002-counter/src/front/app.at](../../../auto-ui/examples/ui/002-counter/src/front/app.at)（下列节选L5–L52；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
use themes.themes: Auto, Solarized

widget App {
    msg { Inc, Dec, Reset, UseAuto, UseSolarized, ToggleMode }

    model {
        var count int = 0
        var theme_name str = "Auto"
        var dark_mode bool = false
    }

    // 主题/模式选择器（C-06/07：数据绑定，同帧校验发布）
    design {
        theme: .theme_name
        mode: if .dark_mode { "dark" } else { "light" }
    }

    // Plan 506: 外壳改"内容即页面"（无 center 包裹，循 011 样板），配合
    // pac `window: "fit"` 窗口随内容收缩。
    view {
        col {
            text `Counter: ${.count}`
            row {
                button "-" { onclick: () => {.count -= 1} }
                button "Reset" { onclick: () => {.count = 0} }
                button "+" { onclick: () => {.count += 1} }
            }
            row {
                button "Auto" { onclick: .UseAuto, style: "px-3 py-1.5 rounded-lg bg-primary text-primary-foreground" }
                button "Solarized" { onclick: .UseSolarized, style: "px-3 py-1.5 rounded-lg bg-secondary text-secondary-foreground" }
                button "切换明暗" { onclick: .ToggleMode, style: "px-3 py-1.5 rounded-lg bg-accent text-accent-foreground" }
            }
            style: "items-center gap-4 p-6"
        }
    }

    on {
        .Inc -> { .count += 1 }
        .Dec -> { .count -= 1 }
        .Reset -> { .count = 0 }
        .UseAuto -> { .theme_name = "Auto" }
        .UseSolarized -> { .theme_name = "Solarized" }
        .ToggleMode -> { .dark_mode = !.dark_mode }
    }
}


// 对照：同一计数器的显式 msg/on 写法（与上面的内联 lambda 简写等价）。
```

本包所需独立结果：原Counter加/减/Reset轨迹0→1→0→0且两个实例不串；主题Auto/Solarized、明暗/布局/事件保原行为；单text Hello不能关闭原001或Counter。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 原widget/use/model/msg/on/design源→979受检投影→969/970/AV或新A2R→同一UI宿主/renderer→事件/布局/真实窗口。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct UiProgram { view: DomainIrRef, model: TypedModel, handlers: HandlerSet, bindings: BindingGraph, requirements: RequirementSet }
struct UiInstance { execution: SessionId, model_owner: OwnerId, mounts: MountSet, input_epoch: Generation }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [auto-ui/crates/auto-ui-dialect/src/ast.rs](../../../auto-ui/crates/auto-ui-dialect/src/ast.rs) | 完整WidgetDecl，不另定义小组件全集 |
| [auto-ui/crates/auto-ui-runtime/src/ui/vm_bridge.rs](../../../auto-ui/crates/auto-ui-runtime/src/ui/vm_bridge.rs) | 替旧执行依赖，中立组件接面 |
| [auto-ui/crates/auto-ui-runtime/src/ui/iced/renderer/projection.rs](../../../auto-ui/crates/auto-ui-runtime/src/ui/iced/renderer/projection.rs) | 复用Iced呈现，Rust不解释guest逻辑 |

**状态、错误与资源。** 错prop/事件payload/未注册主题、ABI/grant/陈旧实例拒；未授权dispatch=0，不能回退旧Parser/VM或只Rust手画图。 App/model/组件/挂载/window分owner，重挂不重复初始化；Reload坏源保旧，focus/IME/捕获/关闭资源正确；九批模式各保I/V/R。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [auto-ui/examples/ui/001-helloworld/src/front/app.at](../../../auto-ui/examples/ui/001-helloworld/src/front/app.at) | 原001含themes/model/design/col/selectable，不能精简冒完成 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `b2763be5178fa040` | 拟ace.m2.p980.source01：run-av-ui/run-a2r-ui及原各模式V；负控/资源见V-DETAIL-02/03；原任务 B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V |
| [auto-ui/examples/ui/002-counter/src/front/app.at](../../../auto-ui/examples/ui/002-counter/src/front/app.at) | 状态/事件/主题与实例隔离 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `04937c35325faa74` | 拟ace.m2.p980.source02：run-av-ui/run-a2r-ui及原各模式V；负控/资源见V-DETAIL-02/03；原任务 B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V |
| [auto-ui/crates/auto-ui-runtime/tests/plan041_instance_ownership.rs](../../../auto-ui/crates/auto-ui-runtime/tests/plan041_instance_ownership.rs) | 真实生命周期/输入/视图owner原义务 | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `3c4a05050dd86791` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 完整字段/组件/布局/Store/watch/setup/expose等按实际grammar冻结；NATIVE/WEB/RUST/TAURI/JET/ARK原适用格分别验证，不统一假定所有端等价。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`B-DIALECT-I`、`B-DIALECT-V`、`B-STATE-I`、`B-STATE-V`、`B-PARITY-I`、`B-PARITY-V`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ace-migration/m2-autoui.md | 有限切片→实际已验支持/拒绝/来源 | 不把计划当现状 | AC-01..07、全部AC-B-* |
| SD-02 | modify | docs/specs/ace-migration/replacement-scope-and-evidence.md | 959行关联新producer/consumer及差额 | 替换闭合 | AC-03/05/06 |
| SD-03 | modify | auto-ui仓`docs/specs/ui/structure-closure.md`及`docs/specs/runtime/component-and-store-instances.md`（各owner） | 保原组件/Store语义，落实际已验ACE来源/模式及Node互操作接面，未交不改现状 | 传统UI不能被图形小例替代 | AC-07、AC-B-DIALECT/STATE/NATIVE/RUST/PARITY |
| SD-PHASE | add | `docs/specs/ace-migration/m2-autoui.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |

外仓领域知识由真实owner落当地Specs；语言侧只保通用接口/迁移指针，不重复领域规则。
| SD-HELLO | add | docs/specs/ace-migration/m2-autoui.md（沿原owner） | 未有早期交接→本切片实际已验接口/源/拒绝及剩余完整义务；交付后沉淀 | 最小Hello不冒全包完成 | AC-HELLO/AC-HELLO-REVIEW/AC-HELLO-LAND |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-autoui.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | 各批V/I与原最终work验证及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


全部未执行。cwd前缀lang/ui/man为本Plan唯一tree。各I读取原runner/help/项目配置固定参数/工具版本，创建永久正反例，未冻结不能进入V；现有target绿不自动证明新ACE支持。只保原支持承诺，原TODO不冒已实现，但实际承诺的缺环境格不得免验。

材料唯一D:/autostack/plan-artifacts/plan-980/，开工登记owner/用途/在途；按仓/批区分，外置target/SDK/安装副本均归scope。窗口/GPU单实例错峰，原ignore/平台/资源预算保持；准备/运行/清理、选择/执行/排除/子终态分列。

| Case | AC | work入口 | 独立观察 |
|---|---|---|---|
| V-B-DIALECT | AC-B-DIALECT | ui: cargo test -p auto-ui-dialect --lib；cargo test -p auto-ui-compiler --test ui_domain_validation；lang: cargo t ace-source ace-syntax | 普通标识符上下文保持；旧widget原源解析/绑定/错误span正确；坏字段拒绝，缺字段不能悄悄忽略。 |
| V-B-STATE | AC-B-STATE | ui: cargo test -p auto-ui-runtime --test plan041_instance_ownership --test plan033_b1_scope_isolation -- --test-threads=1；lang: cargo t ace-av ace-host | 双App各两组件不串；异步任务返回只写仍存活实例；MIX-03无HTTP版本用真实异步host，HTTP实腿由983/联验补齐。 |
| V-B-NATIVE | AC-B-NATIVE | ui: cargo test -p auto-ui-runtime --test desktop_behavior --test image_surface_contract -- --test-threads=1；工具现有auto run -r vm执行隔离工程，记录实际binary身份 | 001原文字选择复制/主题，页面切换、输入法/捕获、媒体/终端、关闭重开与资源回收逐项实测；不是首帧即通过。 |
| V-B-WEB | AC-B-WEB | ui: cargo tu（保原单进程）；cargo test -p auto-ui-runtime --test vue_capabilities；实际生成工程按自身package.json锁定脚本构建并操作 | 真实浏览器挂载/事件/绑定与源expected一致；setup只一次、watch卸载停止；生产表达式不走旧TypeScriptTrans。 |
| V-B-RUST | AC-B-RUST | ui: cargo tu（原单进程）；lang: cargo t ace-a2x generated-delivery-controls；RUST原工程runner与工具版本由I冻结并真实执行，不能emit-only | 未改生成物冷编译运行，props/msg/route与原独立oracle相符；坏源保旧工程。 |
| V-B-TAURI | AC-B-TAURI | ui: cargo tu（原单进程）；lang: cargo t ace-a2x generated-delivery-controls；TAURI原工程runner与工具版本由I冻结并真实执行，不能emit-only | 真实Tauri窗的invoke和事件、失败诊断及重开，不能仅浏览器前端通过。 |
| V-B-JET | AC-B-JET | ui: cargo tu（原单进程）；lang: cargo t ace-a2x generated-delivery-controls；JET原工程runner与工具版本由I冻结并真实执行，不能emit-only | 原支持工程在实际Jet工具链构建运行，事件/状态独立oracle，缺平台未执行。 |
| V-B-ARK | AC-B-ARK | ui: cargo tu（原单进程）；lang: cargo t ace-a2x generated-delivery-controls；ARK原工程runner与工具版本由I冻结并真实执行，不能emit-only | 原支持工程在真实Ark工具链执行，原widget/事件/状态正确，缺平台未执行。 |
| V-B-PARITY | AC-B-PARITY | ui: cargo test -p auto-ui-runtime --test theme_scope_isolation --test gallery_golden -- --test-threads=1；实机与浏览器逐原支持模式验证，窗口单实例 | 主题scope与原画面/交互/生命周期对照，UI058等AGE新能力不是本包替代；每格source/backend/runtime/行为收据齐。 |
| V-UI-SOURCE | AC-07 | B-DIALECT-I建完整分母普通源码正反例，B-PARITY-I接字段/构造覆盖检查；V按现有ui_domain_validation/领域target运行新命名case | 合法传统UI字段有投影/模式/owner，坏字段或错事件参数报原span；删除一条必需字段/模式行检查红、合法近邻仍绿；修复后同入口成功，不能静默丢字段 |
| V-UI-FULL | AC-07 | B-NATIVE-I/B-RUST-I建纯传统UI多文件工程，V跑AV及新A2R→编译→实际UI运行；B-PARITY-V汇总原组件/模式必需格 | 无2D/3D声明仍完整工作，双App各两组件、局部2:1事件、Store/路由/异步卸载/主题/输入各对独立oracle；AV改源无Cargo/rustc，A2R实际执行生成逻辑，无VM fallback。001或少量控件绿不代未覆盖格 |

受影响共享IR/语义加完整职责与必要下游；T-05核全批/组合/原资源门，身份不变证据复用，不反复全仓。局部review不代全包final；未闭in-scope阻execution_done。

### HELLO验证（T-HELLO-I建永久源/用例与命令，全部未执行）

拟窗口命名入口为`cargo test --manifest-path <ui>/Cargo.toml -p auto-ui-runtime --test ace_ui_hello -- --test-threads=1`（新target由I建立），公开run/build参数先核现有CLI并冻结；不得把此拟命令写成已执行。

| Case | AC | 输入→producer/consumer→独立关闭oracle | owner/边界 |
|---|---|---|---|
| V-HELLO-TEXT | AC-HELLO | 单text普通源、改根名/中文文本→H0+E0+AV+F0 Rust provider→真实Iced窗口；源字节、provider复制值/dispatch和可见文字相符，非旧链/硬编码图 | work；静态窗口不冒事件/全UI |
| V-HELLO-BODY | AC-HELLO | 第二同形单text源由普通Auto函数返回值；改函数Hello→你好后真AV结果/FFI接收/窗口均变，未变host的cargo/rustc/bindgen=0 | work；不承诺970同PID热更；重复公开运行仍需新值 |
| V-UI-AXR0 | AC-B-RUST | 同源/资源→新A2R生成→真实rustc→同UI provider→窗口；改函数结果随变，禁AV/旧VM provider仍运行；两端各对独立文本/来源与字节oracle | work/B-RUST-I/V/R；不阻AV HELLO交接，emit-only或Rust手写业务不关闭 |
| V-HELLO-REJECT | AC-HELLO | 坏UI语法/文本类型报原span；错ABI/provider/权限拒绝且Rust callee=0，坏文本/宿主失败保旧；精确修复后同公开入口真Run | work；不把timeout/缺GPU/工具当语言负例 |
| V-HELLO-LIFE | AC-HELLO | guest文本释放/替换后宿主复制仍正确；20轮创建窗口/关闭和取消，session/库/文本owner无增长，关闭旧代不能写新窗口；实际窗口可resize/退出 | work；不假称物理显存归零或覆盖全部原生交互 |
| V-HELLO-PACK | AC-HELLO | B0配方构建最小host/AV闭包；A2R包随AXR0-PACK/原RUST另核；仓外异根/任意cwd运行，缺shim/错ABI拒绝修复再Run；源/grammar/provider/toolchain与包hash齐 | work；AV开发前提明示，A2R用户运行端无Rust/开发仓，非986完整SDK升级 |

GPU/窗口单实例，未来按明确授权排窗；纯文档本轮不运行。实现者自检后同一唯一树独立final，R与D分别审查/落主，不另建第二review树或本轮派agent。只有同版完整HELLO证据可交下一批，旧M1窗口截图不能抵ACE Hello。

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V | 原Counter加/减/Reset轨迹0→1→0→0且两个实例不串；主题Auto/Solarized、明暗/布局/事件保原行为；单text Hello不能关闭原001或Counter。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V | 错prop/事件payload/未注册主题、ABI/grant/陈旧实例拒；未授权dispatch=0，不能回退旧Parser/VM或只Rust手画图。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V | App/model/组件/挂载/window分owner，重挂不重复初始化；Reload坏源保旧，focus/IME/捕获/关闭资源正确；九批模式各保I/V/R。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

| ID | owner_stage | 可观察标准 | 验证 |
|---|---|---|---|
| AC-01 | work | 分母/批次/前置/原模式/命令/资源完整 | T-01及各I |
| AC-02 | work | 完整原能力经真实原源/工程执行 | 全部I/V |
| AC-03 | work | 实际consumer/权限/资源/恢复接线、无legacy fallback | 全部V/959行 |
| AC-04 | review | 独立核各批及全包AC/差额/健康/Spec | 各R/T-05 |
| AC-05 | merge | 正式来源/规范/归档/清理齐 | T-06/07 |
| AC-06 | work | 原T-08补充义务全部分配且闭合 | 全部AC-B-* |
| AC-07 | work | 传统UI全部既有语法/字段/组件/模式必需格经ACE接管，原源不强制改Node；纯UI可AV开发及A2R编译运行，输入/状态/主题/资源/恢复不减、无旧执行/发射fallback；接面供981但不等其产品PASS | V-UI-SOURCE/FULL及原各批V；T-01、DIALECT/STATE/NATIVE/RUST/PARITY的I/V |
| AC-B-DIALECT | work | 普通标识符上下文保持；旧widget原源解析/绑定/错误span正确；坏字段拒绝，缺字段不能悄悄忽略。 | B-DIALECT-V/R |
| AC-B-STATE | work | 双App各两组件不串；异步任务返回只写仍存活实例；MIX-03无HTTP版本用真实异步host，HTTP实腿由983/联验补齐。 | B-STATE-V/R |
| AC-B-NATIVE | work | 001原文字选择复制/主题，页面切换、输入法/捕获、媒体/终端、关闭重开与资源回收逐项实测；不是首帧即通过。 | B-NATIVE-V/R |
| AC-B-WEB | work | 真实浏览器挂载/事件/绑定与源expected一致；setup只一次、watch卸载停止；生产表达式不走旧TypeScriptTrans。 | B-WEB-V/R |
| AC-B-RUST | work | 未改生成物冷编译运行，props/msg/route与原独立oracle相符；坏源保旧工程。 | B-RUST-V/R |
| AC-B-TAURI | work | 真实Tauri窗的invoke和事件、失败诊断及重开，不能仅浏览器前端通过。 | B-TAURI-V/R |
| AC-B-JET | work | 原支持工程在实际Jet工具链构建运行，事件/状态独立oracle，缺平台未执行。 | B-JET-V/R |
| AC-B-ARK | work | 原支持工程在真实Ark工具链执行，原widget/事件/状态正确，缺平台未执行。 | B-ARK-V/R |
| AC-B-PARITY | work | 主题scope与原画面/交互/生命周期对照，UI058等AGE新能力不是本包替代；每格source/backend/runtime/行为收据齐。 | B-PARITY-V/R |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、各批V/I与原最终work验证；原独立review复核 |
| AC-HELLO | work | 普通单text字面量/Auto函数源在AV→真实Rust FFI→Iced窗口成立，原span/文本寿命/失败恢复/冷消费可核 | T-HELLO-I/V；V-HELLO-* |
| AC-HELLO-REVIEW | review | 同树独立逐源/ABI/dispatch/窗口/资源负控复核，绑定正式依赖来源 | T-HELLO-R |
| AC-HELLO-LAND | merge | 首Hello正式源/接口/产物/已验规范落主，保原001与全部传统UI/模式验收义务 | T-HELLO-D |

## 8. 执行步骤

当前共35任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

早期任务可按§5最小前置先行，不要求T-01完整分母冻结已完成；各I自行冻结本切片输入/接口/命令，不提前勾T-01或其它原整项。以下行序不是额外前置。

- [ ] **T-HELLO-I｜M2-A｜owner_stage: work｜AC-HELLO**：967/F0-D、979/H0-D、969/E0-D、978/B0-D齐后，按§5冻结最小源/CLI/主线程/文本所有权与资源，建ace-hello-minimal及ace_ui_hello永久case；只实现同一受检通路，不等T-01全部UI分母/STATE async/PARITY。

- [ ] **T-HELLO-V｜M2-A｜owner_stage: work｜AC-HELLO**：T-HELLO-I后跑全部V-HELLO-*与真实受影响UI命名case/必要语言职责并集，AV首窗全链记录独立oracle/窗口/FFI dispatch/退出；AXR0由原RUST并行验证，未执行腿保持pending，不阻AV部分正式交接。

- [ ] **T-HELLO-R｜M2-A｜owner_stage: review｜AC-HELLO-REVIEW**：T-HELLO-V的AV全部V-HELLO-*齐后停写同树独立核最小普通源到真实Iced、AV FFI与资源/反控；finding返I，不把AV Hello pass冒A2R或完整UI。

- [ ] **T-HELLO-D｜M2-A｜owner_stage: merge｜AC-HELLO-LAND**：T-HELLO-R pass后按各owner phase fold正式合入首Hello代码/已验规范与source/ABI/包来源，保唯一树/resync继续原九批与全包review；不改默认、不归档980或宣称完整RG-A03/M2A-08已过。



- [ ] **T-01｜M2-A｜owner_stage: work｜AC-01/07/AC-PHASE**：核959行及§5完整传统UI的grammar/AST/Spec/组件/模式分母、来源/接口与唯一树/写窗，冻结纯UI AV/A2R及各批命令/oracle/资源，不能只列图形小例。

- [ ] **B-DIALECT-I｜M2-A｜owner_stage: work｜AC-B-DIALECT/AC-02/03/06/07**：按WidgetDecl全部字段逐项投影/明确模式适用性，schema→typed领域IR→ACE行为；字段覆盖表包含named_views/bind/routes/setup/watch/expose/actions/timer/theme。 按§5真实改面补永久fixture并固定§6命令。 同时落实§5完整传统UI门与V-UI-SOURCE/FULL的本批部分，不等981产品终验。

- [ ] **B-DIALECT-V｜M2-A｜owner_stage: work｜AC-B-DIALECT/AC-07**：运行V-B-DIALECT完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。 同时核V-UI-SOURCE的完整字段/构造覆盖与遗漏负控。

- [ ] **B-DIALECT-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-STATE-I｜M2-A｜owner_stage: work｜AC-B-STATE/AC-02/03/06/07**：用ACE执行服务替旧handler bytecode生成/调用，接组件props/emit/computed与App store、挂起/唤醒、卸载和late generation。 按§5真实改面补永久fixture并固定§6命令。 同时落实§5完整传统UI门与V-UI-SOURCE/FULL的本批部分，不等981产品终验。

- [ ] **B-STATE-V｜M2-A｜owner_stage: work｜AC-B-STATE**：运行V-B-STATE完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-STATE-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-NATIVE-I｜M2-A｜owner_stage: work｜AC-B-NATIVE/AC-02/03/06/07**：接原生run入口、路由/页面/主题/媒体/terminal等宿主；实际旧vm参数可保兼容名字但dispatch为ACE。 按§5真实改面补永久fixture并固定§6命令。 同时落实§5完整传统UI门与V-UI-SOURCE/FULL的本批部分，不等981产品终验。

- [ ] **B-NATIVE-V｜M2-A｜owner_stage: work｜AC-B-NATIVE/AC-07**：运行V-B-NATIVE完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。 同时核V-UI-FULL的本批真实结果与完整分母，不以图形小例替代。

- [ ] **B-NATIVE-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-WEB-I｜M2-A｜owner_stage: work｜AC-B-WEB/AC-02/03/06**：Vue生成使用新TS/JS provider，保watch/setup/expose/ext_imports/style及绑定；服务端使用已交宿主接口或隔离真实fixture，不等整个983才能证UI表达式。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-WEB-V｜M2-A｜owner_stage: work｜AC-B-WEB**：运行V-B-WEB完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-WEB-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-RUST-I｜M2-A｜owner_stage: work｜AC-B-RUST/AC-02/03/06/07**：先按§5 AXR0切片运行同源单text V-UI-AXR0并独立复核/正式交接；随后多文件store/组件与sidecar经新A2R生成，保持原Rust UI行为和增量产物；旧AST emitter无回退。 按§5真实改面补永久fixture并固定§6命令。 同时落实§5完整传统UI门与V-UI-SOURCE/FULL的本批部分，不等981产品终验。

- [ ] **B-RUST-V｜M2-A｜owner_stage: work｜AC-B-RUST/AC-07**：运行V-B-RUST完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。 同时核V-UI-FULL的本批真实结果与完整分母，不以图形小例替代。

- [ ] **B-RUST-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-TAURI-I｜M2-A｜owner_stage: work｜AC-B-TAURI/AC-02/03/06**：接新TS/JS与Rust backend生成，保持前后端invoke/事件/权限边界；旧AST emitter无回退。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-TAURI-V｜M2-A｜owner_stage: work｜AC-B-TAURI**：运行V-B-TAURI完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-TAURI-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-JET-I｜M2-A｜owner_stage: work｜AC-B-JET/AC-02/03/06**：保持现有Jet生成模式，以受检UI领域IR供目标adapter；不新增原未承诺能力；旧AST emitter无回退。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-JET-V｜M2-A｜owner_stage: work｜AC-B-JET**：运行V-B-JET完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-JET-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-ARK-I｜M2-A｜owner_stage: work｜AC-B-ARK/AC-02/03/06**：接Ark领域投影和表达式adapter，保现有Ark工程/资源/模块输出；旧AST emitter无回退。 按§5真实改面补永久fixture并固定§6命令。

- [ ] **B-ARK-V｜M2-A｜owner_stage: work｜AC-B-ARK**：运行V-B-ARK完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。

- [ ] **B-ARK-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **B-PARITY-I｜M2-A｜owner_stage: work｜AC-B-PARITY/AC-02/03/06/07**：从001按文字布局→表单列表滚动→菜单Blueprint→媒体编辑分组；核完整语法/字段/组件/变体/模式双向表，补未被Demo覆盖的语法/错态及删除必需行负控，建V-UI-SOURCE/FULL并汇总纯传统UI AV/A2R实际结果。按§5接fixture/固定命令，不等981完整Node终验。

- [ ] **B-PARITY-V｜M2-A｜owner_stage: work｜AC-B-PARITY/AC-07**：运行V-B-PARITY完整职责与真实consumer，独立expected/源/工具/资源/子终态齐。 同时核V-UI-FULL的本批真实结果与完整分母，不以图形小例替代。

- [ ] **B-PARITY-R｜M2-A｜owner_stage: review｜AC-04**：停写后独立核本批AC/反例/差额/跨仓来源，缺陷回I；不冒整包交付。

- [ ] **T-05｜M2-A｜owner_stage: review｜AC-04**：全包独立final核全部批收据/组合/遗漏/延期/workaround、warning/fmt/debug和Spec；缺必需结果不得pass。

- [ ] **T-06｜M2-A｜owner_stage: merge｜AC-05**：按owner次序合入/落规范，核真实来源/consumer同版，全部齐后归档。

- [ ] **T-07｜M2-A｜owner_stage: merge｜AC-05**：摘要折Plan、输出逐目录对账/清scope/登记，guard clean后清树。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料



沿§3/§5既定唯一树与原owner规约实施；材料仅归本Plan的plan-artifacts按仓/批次scope，未知归属不删除，清树前必须wt-guard clean。本轮未创建树或材料scope。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；B-DIALECT-I/B-DIALECT-V/B-STATE-I/B-STATE-V/B-PARITY-I/B-PARITY-V分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-12 stage:new / PLAN-980 r8：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原35任务/全部AC与V/依赖/阶段/预算和drafting 0/35；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-11 / stage:new / PLAN-980:r7 / outcome:pass（早期Hello路线合同可审阅）；blocked（尚未授权实施，真实能力/接口待交） / next:按§5最小前置进入新增切片；原完整M2任务与终验均保留，不用小例替全包。新增T-HELLO-I, T-HELLO-V, T-HELLO-R, T-HELLO-D，当前0/35。未实现/构建/运行或派agent。

2026-11-11 规划静态核验（非产品/独立实现review）：12份本轮文档、6份Plan共106任务；对照本轮前快照，原86任务ID及原AC/V全部保留，新增20任务、全为未执行，drafting/current_step=0。0–10章节、total_steps、阶段清单与AC任务映射一致；323处本地链接已检查，新增链接/4处新锚点无失效、无新增行尾空白。跨四仓可达207份Plan的显式依赖无缺失/环，8个早期交接节点无环、AV不反等A2R或完整B；12个拟职责组名存在。6个具名源码SHA与本轮读取值相同；语言/UI/内容计数器仍1006/081/032。未新取号、建树、改产品/测试资产或canonical Specs，未构建/运行/签阶段/merge/tag。此检查不证明拟议接口已经实现。

2026-11-10 stage=new / r6：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部31个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r5：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/31；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-09 r4 / stage:new：传统UI完整支持明确进入AGE M2，新增AC-07、V-UI-SOURCE/FULL与SD-03，保原9批31任务/全部原AC/V；DIALECT/STATE供981接口、NATIVE/RUST验纯UI双后端、PARITY核完整分母和原各模式。980不反向等981，不以图形/少量控件替全UI。drafting、0/31；仅规划，实施handoff仍blocked（未授权实施/依赖未齐），非运行或独立final PASS。

本轮静态检查：31任务/16AC/11验证行，原ID全部保留、无重复且0完成。五份相关合同可达199份Plan，无缺失引用/环；批次980/PARITY不等981/LANGUAGE，新链接可解析。没有构建/纯UI运行证据，不记产品PASS。

2026-11-07 r3：细化9批，0/31。原T-02/03实现→各I，T-04验证→各V，T-08补充→各批AC；原T-01/05/06/07保留；原STATIC/TARGETS汇总批进一步逐目标拆成具名I/V/R，§5–8为唯一当前合同。只做文档静态核验，历史0/8不代表当前计数；drafting不变，未施工。


2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
§5已冻结批次；T-01只补各批接口/命令参数及实测环境，不再负责从零拆大包。新增分母项须追加具名批次或正式后继并逐AC迁移，保持总闭合责任。
本次明确完整传统UI而非无限新DSL：已有语法/组件/模式承诺不能删，源/实现冲突保持具名待闭；未承诺平台不冒已支持，必需格缺环境不记PASS。新的完整Node语法/五view/Frame实证由981负责，980不能以该分工放弃原UI语义或复制第二解析器。
