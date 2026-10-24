---
plan_id: PLAN-831
status: execution_done
feature_name: auto-ui 独立仓承接与完整发行交付
author: [Codex]
created_at: 2026-10-22
updated_at: 2026-10-24
plan_revision: 2
current_step: 9
total_steps: 12
supersedes_spec_components: [docs/specs/repository-boundaries/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md, docs/specs/auto-ui-compiler/project.md, docs/specs/auto-ui-runtime/project.md, docs/specs/auto-ui-tools/project.md, docs/specs/auto-ui-dialect/project.md, docs/specs/language-sdk/project.md, docs/specs/project-model/project.md, docs/specs/auto-lang/project.md, docs/specs/auto-man-core/project.md, docs/specs/auto-man/project.md]
new_spec_components: [docs/specs/auto-ui/project.md, docs/specs/repository-boundaries/ui-repository-delivery.md, docs/specs/auto-api-gen/project.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-ui/repository, language-sdk, auto-lang/compat, project-model, repository-boundaries]
depends_on: [PLAN-814, PLAN-815, PLAN-816]
---

# [PLAN-831] auto-ui 独立仓承接与完整发行交付

> **r2启动锁：G831未满足前，T-01及全部准备/实施任务均禁止开始。** 必须等待815、816整Plan完整交付合入并收尾。 旧r1提前准备/并行规则已撤销；执行前先核验§5.1。

## 0. 变更摘要

完成R08并吸收R07 UI发行：把814/815已交付UI文法、compiler/runtime/tools及其实际资源/测试/示例/脚本/CI与上层兼容承载迁进**既有auto-ui仓**；切换生产消费者，删除语言仓原实现，以独立clone/第三方工程/安装与失败恢复证明可独立交付。不是把当前crates再复制一份存档。

全部准备和实施统一等待815、816完整交付合入及收尾；G831满足后才从T-01开始，不提前盘点/保全/建树/编写验证设施。不重做815/816运行合同，不重开814；不在本Plan做UiAst终态/大规模去重/后端退役或远端发布。

## 1. 目标

- auto-ui独立workspace持UI生产实现、兼容和适用输入，依赖固定语言SDK/轻数据版本，零Man实现/旧语言大库反向边及隐式兄弟源码。
- 语言SDK无UI checkout的parse/compile/VM与normal/build/dev闭包继续成立；UI生成/运行与旧路径/feature/类型身份不退化。
- UI发行及外部生成工程fresh、locked、重定位、升级/失败保护、资产/工具/producer/锁全闭环，真实支持矩阵可复跑。
- legacy可恢复、新树与来源映射明确、迁出代码真实删除、独立CI入口和本地执行结果齐备，正式交接固定UI版本给832。

## 2. 架构方案

仓界遵循[本批owner/阶段/事务合同](reports/repo-delivery-parallel-order-831-832.md)§1–4。auto-ui承接auto-ui-dialect/compiler/runtime/tools；旧auto-lang兼容crate属于上层UI组合候选，保package名/公开类型/feature，不能再放入纯SDK闭包。shell-pack、UI测试/画廊/构建patch/blueprints等按真实消费者闭包承接，不顺手搬未使用的旧仓GPUI实现继续双生产。

中立project-model/I-TOOL/AST/值及std-only auto-tool-execution owned进程设施仍单源语言基础（UI与Man真实共享）；auto-api-gen属语言服务adapter。tools/API生成对Man声明规则的残边必须在本Plan内去掉（§5），不能把整个auto-man-core也vendoring进UI冒充独立。目标包名/版本和fixed-rev清单T-01有界冻结，重定位不依赖原路径。SDK build.rs、compiler指纹及生成器canonical import路径随实际源根更新。

## 3. 技术栈

现有Rust/Cargo、UI各后端、auto-tool-protocol与I-PROJECT单源codec、792受管发布/consumer driver、777 strict、784状态寻址保全/恢复工具。CI脚本随owner迁，复用现有autoui-verifier双端设施，依赖本地Git固定commit验收渠道；不新建仓/重新实现发布器或runner。

## 4. 需求分析与背景调查

用户2026-10-22进一步明确采用保守串行方案：**815/816完整交付合入→831完整交付→832开始**。本r2取代r1提前准备/并行开工授权；目标、原任务/验收/规范增量ID和全部发行范围不变。本轮仅修订调度，未实施；[本轮来源](attachments/repo-delivery-r2-serial-20261022/inputs.json)。

用户2026-10-22明确要求建立两份大模块搬仓合同并包含发行验收、明确与815/816并行。**本轮仅在语言主检出写规划，不执行跨仓修改/保全/建树/推送**；未来work按本合同范围与既有授权开展，外仓权限按执行环境实际授予。本地既有仓追加提交与经审查的默认分支推进是计划交付；远端更名/force/push不是隐含授权。

当前Spec从[全局入口](../specs/overview.md)定位，以[SDK](../specs/language-sdk/project.md)、[compiler](../specs/auto-ui-compiler/project.md)、[runtime](../specs/auto-ui-runtime/project.md)、[tools](../specs/auto-ui-tools/project.md)、[旧仓治理](../specs/repository-boundaries/project.md)、[发行](../specs/repository-boundaries/generated-ui-distribution.md)为准。总览部分数量/旧描述滞后，以模块Spec+代码/收据为证。814 B=93a25c924及后继丢测修复f97a9e2a9已合入；815仍executing、B运行与验收修复，816仍executing、完整CLI待门，未当成已交付。

已观察写面：`crates/auto-ui-{dialect,compiler,runtime,tools}`、旧`crates/auto-lang`上层再导出、`crates/shell-pack`、`crates/auto-api-gen/src/sidecar.rs`、`crates/auto-ui-tools/src/{vue,rust_ui}.rs`、`crates/auto-man-core`声明来源及`crates/auto-project-model`；Cargo/build.rs/资源及`test/fixtures/modularity/{ui-*,generated-ui-consumer,language-sdk-consumer}`、`.cargo/config.toml`/nextest/CI/777/792均需真实映射。

旧仓`../auto-ui/Cargo.toml`与CLAUDE.md已读；旧指令“master已清空”在784已证过时，**本轮未重采refs/WIP，不用历史tip假充今日基线**。784保全及恢复输入见[历史报告](reports/784-repo-preservation.md)，执行T-01/02刷新。冻结输入hash、源SHA与前置状态见[本轮输入](attachments/repo-delivery-batch-20261022/inputs.json)。

## 5. 详细设计

### 5.1 启动门与下一任务入口

**启动门是整Plan硬门，覆盖T-01..T-12及所有准备工作。** U815/U816/U831均指对应整Plan全部约定验收与最终独立审查通过、merge-owned冻结门通过、实现/Spec/ledger实际合入、完整版本/来源交接收据读回且归档收尾闭环；必须查真实交付与归档记录，不能只看状态字样或勾选数。阶段A/B局部发布、某项pass、分支HEAD或仅T-11代码落地均不代替整项完整交付。

**G831 = U815且U816；G832 = G831且U831。** 前者未满足，831不得开始；后者未满足，832不得开始。等待期间只允许读上游计划状态、归档/合入与交付收据以核实启动门，或用户授权的计划簿记；不能开展本Plan T-01实质盘点、保全/恢复演练、fixture/driver编写、候选导入、测试、分支/工作树创建或认领。不得以“只读准备”“不改共享文件”或旧r1允许并行为例外。

启动时先在主检出核实G和完整源版本，将上游Plan revision、实际delivery commit、Spec/ledger与归档/交接收据记录到本Plan基线，再进入T-01。门未满足时保持drafting/current_step 0（如误建树则保全停止写入并如实报告，不自行删除），只报告等待上游，不标实施开始、不勾任何任务、不启动work agent。现有depends_on现在也是全Plan启动依赖，不只是终验摘要；这是Markdown合同/调度规则，未新增后端状态或自动检查能力。

G831满足后，T-01以815/816完整交付后的实际源码为基线，新增attachments/831/contract-baseline.md：逐包/输入/测试/消费者owner与DAG、profile、当前legacy状态、兼容与共享声明切口可行性。不得沿用r1候选或未合入接口当最终source；必要未知项具名交后继任务，不缩小原AC。门内T-02/03可按各自依赖推进，但不与832实施并行。

### 5.2 数据流与归属约束

| 规则 | 实际producer→consumer/状态 | owner/关闭条件 |
|---|---|---|
| C1 单源源码/类型 | auto-lang-core/轻数据固定source→UI包→旧兼容/外部工程；导入前权威在语言仓，切后在UI | 831；metadata/tree同名包单source+跨包AST/Value/协议类型实编，原生产目录删除/薄转发无第二算法 |
| C2 最小Man去边 | tools对pac_from_decl的theme/external_backend、API-gen的SidecarDecl/sidecar_decl_from_root→现有project-model轻规则→Man旧入口再导出 | 831 T-05；代表缺省/有效/动态求值/主题覆盖/sidecar根规则保持；SDK/API/UI不依Man仓实现，共享执行设施留语言基础；不复制规则或改变I-PROJECT版本语义 |
| C3 build/生成来源 | SDK具名build输入、UIC指纹、stdlib/schema/patch/UI资产→目标workspace/受管生成包 | 831；源码指纹与实际producer同步，缺源/错source/等长篡改/漏patch负控红，不能靠扩大allowlist/禁测骗绿 |
| C4 兼容与寿命 | 旧auto_lang入口、默认/显式Core/UI组合→真实provider/renderer/生成应用 | 831；旧API编译/feature/默认装配/状态单源保持，真实窗口/终端/媒体退出及错误/取消可观测 |

### 5.3 来源迁移与发行

T-04按冻结源commit/path/hash导入目标候选和完整输入，映射rename/copy/generated/compat区别；旧仓legacy在历史分支，不把两套框架同时作为新生产实现。T-05共享轻声明去边与必要类型再导出，T-06在隔离组实现source/生成import/模板根/消费者切换和语言原实现删除，实际默认分支推进归T-11；编译/测试依赖根、宏/include_str/build.rs、许可、font/DLL外部前置均随清单。全量测试语义随owner迁移并由跨仓编排保持，不能从根别名简单删掉迁出测试包。

UI安装/发行profile包含编译库、runtime、tool实际二进制和运行必需资产；生成应用Rust canonical双直依赖、Vue与各现行生成后端支持保持。源码仓独立clone与生成bundle分别fresh/locked/重定位实测；纯SDK的无UI消费是反向控制。auto-down等现行optional源码也必须固定依赖/显式profile处理，Cargo仍读取optional path，不能留不存在的兄弟路径。环境缺口不可当绿，也不自动批准停用后端。

发行/安装事务复用792，其保旧字节/owned清理/误删保护按原精确合同；升级/坏版本/半复制/取消/producer漂移有反例。三仓切换按本批§4 prepared→landed→switched→verified收据，无跨Git原子假设。832只能在本PlanT-12全部收尾闭环构成U831后开始，再消费T-11已发布且T-12复核的固定版本，不能仅凭UI候选或代码发布提前开工。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ui/project.md | 仓内UI包组→既有UI仓owner、入口、依赖/兼容与实际版本 | 独立模块交付 | AC-01,02 |
| SD-02 | add | docs/specs/repository-boundaries/ui-repository-delivery.md | 局部生成包→UI仓/安装/生成应用的来源、profile、事务与消费合同 | R07 UI+R08 | AC-03,04,05 |
| SD-03 | modify | docs/specs/repository-boundaries/project.md | 784准备→本次保全/来源与本地切换实况；未执行远端如实记 | 恢复可验 | AC-05,08 |
| SD-04 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 单仓source表→固定多源producer/锁/资源与适用profile | 保792合同 | AC-03,04 |
| SD-05 | modify | docs/specs/auto-ui-compiler/project.md, docs/specs/auto-ui-runtime/project.md, docs/specs/auto-ui-tools/project.md, docs/specs/auto-ui-dialect/project.md | 实际仓属/源码、测试/资源/接口与兼容路径迁移；不扩大支持 | 真实owner | AC-01,02,06 |
| SD-06 | modify | docs/specs/language-sdk/project.md, docs/specs/project-model/project.md, docs/specs/auto-lang/project.md, docs/specs/auto-man-core/project.md, docs/specs/auto-man/project.md | SDK独立源、共享声明单源与旧compat迁属，保类型/动态语义 | 去仓间环 | AC-01,02 |
| SD-07 | add | docs/specs/auto-api-gen/project.md | 通用API生成语言adapter职责/仓属及共享声明单源，零Man管理实现依赖 | 修复真实跨仓残边 | AC-01,02 |

## 6. 测试设计

下列Case均owner_stage=work，新增driver/fixture由T-03建立并在T-06..09按最终源完善；**本轮尚未执行**。复用`python scripts/legacy-preservation.py inventory/preserve/verify/recover-drill`及其已存在参数、`python scripts/modularity-audit.py --mode strict --no-cargo-check`、现有generated-ui-consumer driver；新增`test/fixtures/modularity/ui-repository-consumer/`与`scripts`下有界三仓fixture入口（具体接口T-03冻结，不假称现有）。独立clone目标仓现有Cargo入口按新package实际执行`cargo check/test --manifest-path ...`定向及真实工程build/run；不因缺源偷偷读取主检出。

| Case | AC / invariant | 输入/真实入口 | 区分性oracle与边界 |
|---|---|---|---|
| V-01 | AC-01 单源/DAG | 独立UI clone、SDK-only consumer，清AUTO根/兄弟path；混入同名双source/Man边负控 | UI真实编译/SDK执行成功；metadata/tree/类型跨包唯一；禁止边/未登记source拒绝，不只扫描字符串 |
| V-02 | AC-02 兼容/声明 | 旧Parser/VM公开构造、pub字段/回调与UI/Core/默认feature；theme/sidecar/external_backend缺省/覆盖/动态字段 | 旧消费可编译/结果一致，单份声明解析与状态，未知/动态未求值不假报；原合法路径不能只为过轻层而拒绝 |
| V-03 | AC-03 独立发行 | 真目标仓→真实tool生成→Vue/VM-Iced/Rust及其他现行适用profile；fresh lock+locked重定位A→B+外部SDK消费 | 来源仅固定版本/登记资源，真实生成/构建/运行成功，锁/工具hash/producer匹配；UI checkout缺席不影响纯SDK |
| V-04 | AC-04 失败恢复 | 旧可用安装/生成物，缺源/漏patch/坏pin/等长资产篡改/中断/超时/交换失败/陌生目录 | 应具名非零，无假completed；按受管事务保旧字节/可恢复点，陌生目录不删；恢复后真应用复绿 |
| V-05 | AC-05 旧仓与切换 | 新refs/WIP/ignored保全、异tip锚冲突、bundle恢复、目标落地/消费切换任一仓失败 | 历史refs/tree/保留字节可恢复；状态漂移新增快照不覆旧；失败不误归档/强移默认分支，重试幂等可执行 |
| V-06 | AC-02,06 生产与测试 | 真实双端provider/终端/媒体/窗口启动退出；迁前后测试ID/alias列表+全部必要source owner映射 | 默认/显式组合与退出保真，无丢测试/双注册；各用例按原档仍选中，源码/资源删除和UI目标唯一owner可复核 |

最终独立review同组每仓原树、独立上下文核全AC/SD/反例/支持矩阵，未指定agent/model不创建会话。work/review只check/定向包与fixture/必要实机；审查pass后merge冻结三仓输入统一裸cargo t+实际tv/tu/tt等与目标仓适用定向门，串行资源队列，tf主语言检出到期单实例。根/依赖改变须重验受影响scope；full suites不能替代未做的独立发行/失败控制。纯规划Category A无Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | UI仓独立包图、Man残边清零、SDK无UI消费、类型/source唯一、必要原件删除；V-01 |
| AC-02 | work | 旧公开API/default与全部现行feature、声明动态语义/类型身份、宿主寿命保持；V-02/06 |
| AC-03 | work | 独立clone与真正外部生成工程的支持profile/fresh/locked/重定位/工具与资源全闭环；V-03 |
| AC-04 | work | 资源/锁/producer拒绝、升级/安装/生成/切换失败与取消保护有真实正反控；V-04 |
| AC-05 | work | 旧仓当前状态保全/恢复/来源映射/本地切换恢复演练有效，未做远端如实记；V-05 |
| AC-06 | work | 测试ID/门选择及跨仓CI入口完整、777/792受影响面通过，SD/来源/profile/handoff完整；V-06 |
| AC-07 | review | 独立final review绑定r2与全部仓冻结SHA/依赖输入，AC/SD/遗漏/告警/格式/debug审查通过；merge门pending |
| AC-08 | merge | 冻结门通过、目标本地分支及语言消费者实际交付/来源读回与Spec/受管ledger齐备；给832版本收据，guard后清理归档 |

## 8. 执行步骤

12项；**本表每一行都先满足G831，包括T-01/T-02/T-03准备**。既有任务ID/AC/SD和工作内容保留；依赖列不能解释成允许跳过全Plan启动锁。

| 状态 | ID | owner_stage | 依赖/启动门 | 写面/交付与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | G831完整核验 | [✅ 已完成] [contract-baseline.md](attachments/831/contract-baseline.md) @源 a61209cc0：逐包迁移映射（迁入五包+留仓清单+C2 adapter）、兼容面（C4）、C2 切点实测定位（api-gen/sidecar.rs:54-60、tools/rust_ui.rs:3367/5931/5951→project-model 单源）、跨仓 autodown-core optional 路径处理要求、fixture/测试档/777/792 影响面、旧仓现状（f7d92ee+WIP，不承接 GPUI）、profile/发行面；5 项未闭决策具名交 T-05/T-06/T-09/T-11 | AC-01..06 |
| [x] | T-02 | work | G831；T-01旧仓事实 | [✅ 已完成] [preservation.md](attachments/831/preservation.md)：盘点（10 refs/WIP=Cargo.lock/无 stash）+快照 45044cf9507d5628（bundle d48d04c1…，payload=.recovery/831-legacy-preservation）+锚点 reuse+verify ok+recover-drill ok（10 refs/树 31ede719…）；执行组 lang-831 双树建立（auto-lang@2dafba909/auto-ui@f7d92ee，分支各 codex/plan-831-dev）；主检出 WIP 未触碰 | AC-05 |
| [x] | T-03 | work | G831；T-01 | [✅ 已完成] `test/fixtures/modularity/ui-repository-consumer/` + `scripts/ui-repository-fixture.py` @lang-831 树 e199caf8a（合入前在 codex/plan-831-dev）：五族 driver（source/type/test-id/distribution/failure）接口冻结 + migration-phases.json 阶段表（pre-migration=五包闭包无 auto-man 大库/无 lang-monolith；source-switched 同断言收紧零 auto-man-core；结构判别+部分导入拒绝）+ 三仓有界入口（解析序 cli>env>组内兄弟>主检出，默认零 cargo）。预迁移态实跑 pass=25/fail=0/recorded=6/pending=12：Man-core 台账=2 条实测边（auto-ui-tools+auto-api-gen→auto-man-core，修正 T-01 单边认知）、legacy auto-ui→auto-lang optional 边记档、autodown 按文档序解析在案、type 16 锚点+C2 声明恰一单源+feature 32 项钉住、test-id 清点（dialect=86 契合契约基线）+档位别名 8/8；selftest 10/10（受控突变区分性 oracle：vendoring/走私边/收紧态红+合法对照绿）；发行/失败族 pending 归 T-07/T-08（现行同构控制 792/815-A）。Category A 无 cargo 门；.gitignore *.json 母规则否定行按 PLAN-793 惯例补入 | AC-01,03,04,06 |
| [x] | T-04 | work | G831；T-02/T-03 | [✅ 已完成] [import-ledger.md](attachments/831/import-ledger.md)：冻结源=lang `e199caf8a`（五包内容 a61209cc0≡master≡worktree 恒等）；五包候选 841 文件导入 auto-ui 迁移分支 `c6d3ac6`（逐字节=源，仅 manifest 机械改写：跨仓 path 前缀 15 边+继承内联 25 行/16 键，语义等价校验 PASS）；过渡 Man-core 边按冻结台账保留；legacy/语言仓零触碰（权威保持）。T-03 表具名修订 +`import-window` 阶段（lang `fb729b04a`，selftest 16 项）。门禁：driver import-window 全绿 + cargo metadata 双树解析（38/46 包；补 down detached 兄弟 @c39ac7f9 修既有 worktree 悬挂，§4 预案）+ 逃逸面 A/B/C/D 与许可面冻结入台账（dialect/shell-pack 缺 license 字段具名交 T-06/T-09） | AC-01,02,05 |
| [x] | T-05 | work | G831；T-04及共享写面交接 | [✅ 已完成] lang `cd9c7af2a` + auto-ui 候选同步 `50a9b94`：C2 最小单源切法——sidecar.rs（163 行逐字+测试）与 pac_display（theme{}块/back 提取逐字切出+C2 正反控）迁 auto-project-model；man-core pac.rs 两字段委派（零复制 oracle 不变）+sidecar 旧入口再导出 shim；api-gen（3 符号+manifest 换 project-model）与 ui-tools（rust_ui×2/vue×1+manifest 去边）切源；DELIVERY_CRATES 26→24（man-core/toolchain-core 出交付闭包）。冻结决策：auto-api-gen 留语言仓（adapter，SD-07）；pac_from_decl 定义恒居 man-core（Man 域完整归一），展示面提取单源 project-model。定向验证：project-model 42/42、man-core pac 12/12、ui-tools 语义+delivery 18/18、api-gen 54/55（tauri fixture A/B 证实预存）；driver TYP-D01/02/03/05/06 恰一（定义形态判别）+SRC-P3 台账清零、候选 P2u 清零（闭包 19→18） | AC-01,02 |
| [x] | T-06 | work | G831；T-05 | [✅ 已完成] lang `c6c46735e`+`61a7f110d`+`e331d47fd`、auto-ui `2ea9b5a`：UI 独立 workspace 终形（legacy 全量移除 317 文件/根 manifest 重写 members=crates/*/Cargo.lock 首次受管 8713 行/license 补齐/UI 侧 t·tu 档位）；lang 切源（auto-lang/auto-man/auto+六 fixture 路径切组内兄弟 ../../../auto-ui/crates、根 members 删五包=41 成员、t/tt/tb/tu 别名拆分）；B 类逃逸收口（aura.at 随 owner 迁 compiler 本地 schema/；a2gd 语料单源留 lang、dialect 86 项以兄弟路径读）；777 窄豁免登记（auto-lang→UI 仓 compiler/runtime 跨根边= C4 兼容，owner PLAN-831-C4/exit R8）+基线重采样（141→116 边 strict pass 0v/5gf）。验证：driver source-switched 33/0 全绿（P2 收紧零 man-core/P4 legacy 清/P6 删除/P5 可解析/TYP-D04 切源/TID-A3 5/5）；cargo check -p auto-lang -p auto-man 跨仓零错；dialect 86/86（=基线数）；TID-A1 清点 86/1101/2245/229/4 与预迁移恒等=零丢测试。冻结决策：固定源机制=组内兄弟相对 path+受管锁（发布渠道 T-07/T-09）；六 UI fixture 留 lang（消费者边界探针属性）；792 producer 源根更新归 T-07 | AC-01,02,06 |
| [x] | T-07 | work | G831；T-06 | [✅ 已完成] lang `461c609`(UI)·`c765211fd`·`836bf471d`：①792 producer 双根布局（delivery_crate_source+path_rewrites 投影，UI 仓源接入+跨仓前缀改写包内兄弟；delivery 10/10 含双根 vendoring 端到端）②792 driver 全程 32/32（fresh-lock/真实 build/A→B 重定位/负控/工具 framed I-TOOL 外部消费/资产字节=包内源；含工具宿主 bin 陈旧假红修复+driver 构建步升级）③独立 clone DST-1..4 5/5（三仓组 clone/fresh 五包入锁/locked 157s/整组重定位 locked 重放 185s/隔离无 UI 兄弟 SDK consumer）④收据绑定 driver 升级：全族 pass=43/fail=0（DST-1..5+FLR-1..4,6 转证据绑定 pass；FLR-5 归 T-08）⑤VM 双端实机：MCP 冒烟全过（窗口/MCP 首帧/select_rect 正反控/正常退出）。附带修复：lang Cargo.lock 转受管（独立 clone 可构建——auto-lang-core build.rs 身份输入硬读根锁+792 preflight 同要求）；fix gen_request_run output_root（815 诚实门×816-R1 清理的 master 源级预存缺陷，旧二进制掩盖，VM 实机暴露；auto-man-runtime 122/122） | AC-02,03,06 |
| [x] | T-08 | work | G831；T-07 | [✅ 已完成] lang `eaf1027e5`：①792 受管升级/缺源/坏hash/漏patch=792 driver V03 收据绑定（T-07 已绿）；②三仓切换事务工具 scripts/ui-repository-switch.py——合同 §4 四状态收据链（prepared→ui-landed→lang-switched→verified，append-only JSONL），仅 ff-only、prepared 锚校验（第三方推进零覆盖）、脏树拒绝、幂等对账、RED 事件携带恢复指令，T-11 以真仓参数调用；③FLR-5 drill 19 检全过（scratch 合成组故障注入：正常链/锚冲突零覆盖+吸收重锚/中断幂等/交换失败保旧+ui landed 保留/prepared-only 取消零副作用/阻塞解除重试+真 cargo 最小组 verified 绿），收据 switch-drill-receipt.json；④driver FLR-5 绑定后全族 pass=44/fail=0/pending=1；旧仓保全/恢复演练=T-02 legacy-preservation 在案（preservation.md：verify ok+recover-drill 10 refs 读回） | AC-04,05 |
| [x] | T-09 | work | G831；T-08 | [✅ 已完成] 首轮核查冻结 needs_fix 快照（六缺口）后同树修复轮全部闭合：F-01 布线（前轮）；F-02 编译探针实编（16 路径×三 feature 组合+类型单源+双 source 负控，TYP-PROBE=pass 24/24）；F-03 V06 真实 run/生成六 profile/构建矩阵（vue·rust·vscode outcome=completed，jet/ark 环境阻塞具名红）/安装中取消恢复/vm 运行+三族寿命 MCP 冒烟，暴露并修复 api-gen 双根路径与 vscode PM 混用两产品缺陷；F-04 档位语义（t/tu/ti/tn+nextest 配置，ti 实跑 28/28）+777 跨仓围栏真实化（pattern 退格转义+字符字面量状态机修复，strict 0v/5gf 零警告）+cross-repo-tiers 编排入口+迁前后正典 ID 对拍（三包逐 ID 相等/tu 863 恰等）；F-05 收据身份 admission（三载体 sources 齐记+S17 负控，RCPT-ID 双 pass 终源 admitted）；F-06 根 LICENSE+双 #[test]+fastrand+告警对拍零新增。[handoff.md](attachments/831/handoff.md) 重冻结为可送 T-10 候选（终态 lang `1635835d3`/UI `464569d`/down `c39ac7f9`）；fixture 全族 46/0/1/1、792 38/39（唯一红=具名环境阻塞）、distribution 5/5、drill 19/19 | AC-01..06 |
| [ ] | T-10 | review | G831；T-09冻结提交 | 独立三仓输入/AC/SD/真实消费与恢复审查，发现遗漏/降级归因，final pass后merge门pending | AC-07 |
| [ ] | T-11 | merge | G831；T-10 pass；共享发布窗口 | master并集影响复审，冻结统一适用门；本地目标分支/语言消费有序落地，生产source/版本/工具hash/Spec来源读回，新增ui-release.md；此时尚不放行832 | AC-08 |
| [ ] | T-12 | merge | G831；T-11全部收据 | canonical Specs/受管ledger及索引沉淀、归档；保留legacy/恢复/已交付分支，wt-guard clean逐树清理本Plan临时树/分支/组；最终交付/归档/清理收据全部读回后更新ui-release.md，构成U831才放行832 | AC-08 |

**G831未满足前不得创建/认领下述组、分支或实施会话；路径只登记未来执行位置。**

执行组`D:/autostack/.wt/lang-831/{auto-lang,auto-ui}`，分支各`codex/plan-831-dev`（属于各自仓）；必要auto-down兄弟依既有解析补detached，**不修改其实现**。784树如已有owner不得抢占，确认保全后另建本Plan分支或经明确owner交接认领，始终每仓仅一个权威树。review不建第二checkout。主检出只簿记，树内禁junction/symlink；移除前逐树wt-guard clean，不删legacy或恢复载荷。

## 9. 复审记录
2026-10-24 / stage: work(T-09 修复轮闭合) / plan_id: PLAN-831 / plan_revision: 2 / outcome: **T-09 完成（F831-09-01..06 全部 CLOSED；handoff 重冻结为可送 T-10 候选）** / worktree_path: D:/autostack/.wt/lang-831/{auto-lang,auto-ui} / branch: codex/plan-831-dev / code_commit: lang 1635835d377bd3faf12bb9ab642e14cebc150f94（修复轮 7 提交：77114ec6f/5339085ea/b90374988/70ea2a462/de4af104c/0d1d8690a/1635835d3）、UI 464569d（4 提交：15e3b14/fb5a2f4/071396e/464569d） / dependency_revisions: down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8 / task_ids: T-09 / evidence: [handoff 重冻结](attachments/831/handoff.md)、[freeze](attachments/831/freeze.json)（open_findings=[]）、[CI 入口终态](attachments/831/ci-entrypoints.md)、[tier-parity](attachments/831/tier-parity-f831-09-04.json)、[warn-license-parity](attachments/831/warn-license-parity-f831-09-06.json)、[run-profile-evidence](attachments/831/run-profile-evidence-f831-09-03.json)；fixture 全族 46/0/1/1（t09-final-all.json；RCPT-ID/RCPT-ID-F 双 admitted）、792 38/39（唯一红=jet/ark 环境阻塞具名）、distribution 5/5、drill 19/19、ti 档 28/28、三族 MCP 冒烟全过 / 产品修复: api-gen 双根路径（直接生成形态生成工程构建红→修）、vscode build 脚本 PM 混用、native_dnd fastrand dev-dep 泄漏、node_converter 534 行化石死区、777 扫描器两缺陷（pattern 退格转义/字符字面量奇偶翻转）/ blockers: 无 / limitations: 本轮为 work 修复轮闭合、非 T-10 独立终审；整档 t/tv/tt/tb/tu/taa 未运行（merge-owned 显式 pending）；jet/ark 构建面=本机工具链缺席具名红（非静默跳过）；T-11/T-12 全部 pending，U831 未成立，832 不得开始 / next_responsible_role: T-10 独立终审（绑定终态三仓 SHA 审核 AC/SD/反例/收据身份/净迁移）

2026-10-24 / stage: work(T-08 切换失败控制) / plan_id: PLAN-831 / revision: 2 / outcome: **T-08 完成（FLR-5 全绿：失败控制面证据齐）** / worktree_path: D:/autostack/.wt/lang-831/auto-lang / code_commit: eaf1027e5 / task_ids: T-08 / evidence: 切换事务 drill 19 检全过（收据 scratch-ui-repo-fixture/switch-drill-receipt.json：D1 正常链/D2 锚冲突零覆盖/D3 幂等零移动/D4 脏树拒绝+master 不动+ui landed 保留/D5 prepared-only 零副作用/D6 阻塞解除重试+mini cargo 组 verified 绿+status 整体判定）；全族 driver pass=44/fail=0/pending=1（FLR-5 绑定转 pass；余 pending=TYP-PROBE 编译探针形态记录）；selftest 16/16 / 交付形态: 工具四子命令+drill；真仓 land/switch 归 T-11 按合同 §4 窗口执行（工具即其执行载体）/ blockers: 无 / next_responsible_role: work T-09（按最终来源复核全部 V/支持面/净迁移非净删/SD-01..07 与实际 CI 入口，handoff.md 冻结交付候选）

2026-10-24 / stage: work(T-07 发行面实测) / plan_id: PLAN-831 / revision: 2 / outcome: **T-07 完成（发行 profile 全证据链 + 双端实机）** / worktree_path: D:/autostack/.wt/lang-831/{auto-lang,auto-ui} / code_commit: auto-ui 461c609（producer 双根）、lang c765211fd（锁转受管）+836bf471d（发行实测+修复） / task_ids: T-07 / evidence: 792 driver 32/32（收据 scratch-consumer/driver-receipt.json）；distribution DST-1..4 5/5（收据 scratch-ui-repo-distribution/distribution-receipt.json：fresh 3.6s/locked 156.8s/重定位 184.6s/隔离 SDK 101.7s，源 ui=461c60984 lang=c765211fd）；全族 driver 收据绑定 pass=43/fail=0/recorded=1/pending=2（FLR-5→T-08、TYP-PROBE 编译探针形态记录）；VM MCP 冒烟全过（001-helloworld：窗口 fit/标题 pac.at、merged 进程内、MCP 首帧、select_rect 正反控、正常终止）；delivery 单测 10/10；auto-man-runtime 122/122 / 实测发现与修复: ①lang Cargo.lock gitignore → 独立 clone 构建红（build.rs 身份输入硬读）→ 转受管 ②auto-ui-tool bin 陈旧（cargo -p auto 不刷依赖 bin）→ 792 driver 构建步补建 ③gen_request_run 缺 output_root（815 run 诚实门 × 816-T05-R1 generate 侧字段清理——master 源级预存，旧二进制掩盖）→ 补工程根锚点 / 冻结决策: 独立 clone 布局=三仓组同级（autodown optional path 的 manifest 加载需其在场）；发行 bundle 级 A→B 重定位=792 V02 证据位 / blockers: 无 / next_responsible_role: work T-08（三仓切换失败控制+792 受管升级/恢复演练，FLR-5）

2026-10-24 / stage: work(T-06 切源) / plan_id: PLAN-831 / revision: 2 / outcome: **T-06 完成（五包源权威=UI 仓；语言仓原件删除；777 重采样 pass）** / worktree_path: D:/autostack/.wt/lang-831/{auto-lang,auto-ui} / code_commit: lang c6c46735e（切源）+61a7f110d（777 窄豁免）+e331d47fd（基线重采样）、auto-ui 2ea9b5a（终形） / task_ids: T-06 / evidence: driver source-switched 33/0（收据 scratch/t06-switched.json：SRC-P2 零 man-core/P4 legacy 清理零残留/P6 lang 41 成员零五包/P5 autodown 可解析/TYP-D04=project-model/TID-A3 5/5）；TID-A1 五包清点与预迁移恒等（86/1101/2245/229/4）=零丢测试；cargo check -p auto-lang -p auto-man 跨仓编译零错、auto-ui check tools+shell-pack 零错、metadata 41/38；dialect 86/86；777 strict pass 0v/5gf（基线 116 边锚定 61a7f110d） / 冻结决策: ①固定源机制=组内兄弟相对 path+UI 仓受管 Cargo.lock（重定位同布局成立；发布渠道/固定 rev 归 T-07/T-09）②六 UI consumer fixture 留语言仓（探针属性不变，路径已切）③a2gd 语料单源留 lang（tt 档共享），dialect 回归以兄弟路径读 ④777 并集=语言仓重采样表+两条 C4 窄豁免（R8 退出） ⑤792 producer 源根随 UI 仓更新归 T-07 / 接口事实: B 类逃逸 aura.at 收口为 compiler 本地 schema/（include 改 crate 相对）；预迁移 8 项 SRC 中 P1b 布线断言全程绿 / blockers: 无 / next_responsible_role: work T-07（真目标 clone/外部工程/发行 profile 实测）

2026-10-24 / stage: work(T-05 C2 去边) / plan_id: PLAN-831 / revision: 2 / outcome: **T-05 完成（C2 切除：两条 auto-man-core 边全消）** / worktree_path: D:/autostack/.wt/lang-831/{auto-lang,auto-ui} / code_commit: lang cd9c7af2a、auto-ui 50a9b94（候选同步） / task_ids: T-05 / evidence: 定向测试 project-model 42/42、man-core pac 12/12（委派后 oracle 同值）、ui-tools 语义+delivery 18/18、api-gen 54/55（test_plan734_tauri_fixture A/B stash 证实预存红非本触面）；driver TYP-D01/02/05/06=project-model 恰一、D03=man-core 恒居（定义形态判别再导出不计）、D04 记档已切；SRC-P3 语言仓台账清零（C2 完成）+P2u 候选清零（闭包 19→18） / 冻结决策: auto-api-gen 留语言仓（SD-07 adapter）；pac_from_decl 定义恒居 man-core、展示面提取单源 project-model；touchpoints 具名修订（定义形态+D05/D06） / blockers: 无 / next_responsible_role: work T-06（切源）

2026-10-24 / stage: work(T-04 来源导入) / plan_id: PLAN-831 / revision: 2 / outcome: **T-04 完成（任务域 handoff，Plan 续 executing）** / worktree_path: D:/autostack/.wt/lang-831/{auto-lang,auto-ui,auto-down} / branch: 各 codex/plan-831-dev（down 为 detached 只读） / base_commit: lang 2dafba909→e199caf8a / code_commit: lang fb729b04a（接口修订）、auto-ui c6d3ac6（候选导入） / dependency_revisions: down 兄弟补 detached@c39ac7f9（§4 预案；worktree optional 悬挂为语言仓既有态非导入引入，主检出恒绿） / task_ids: T-04 / evidence: 冻结源恒等（五路径 git diff 空+导入后 841 文件逐字节对比除 manifest 零差异）；manifest 语义等价 PASS（cargo 继承语义对拍）；driver import-window 全绿（SRC-P1/P1b×2/P2/P2u/P4a/P4b pass；P3/P3u/P5/P6 记档，台账观察=窗口期望恰等）；cargo metadata --no-deps 双树 38/46 包；selftest 16/16（S4 窗口含走私反例） / 接口修订: T-03 冻结表具名新增 import-window 阶段（原因=T-04 双侧五包态无法命名；加细非放宽；预迁移基线仍绿 26/0/6/12） / blockers: 无 / next_responsible_role: work T-05（共享声明最小单源去 Man 边，依赖 T-04+共享写面交接已满足）

2026-10-24 / stage: work(T-03 验证设施冻结) / plan_id: PLAN-831 / revision: 2 / outcome: **T-03 完成（任务域 handoff，Plan 续 executing）** / worktree_path: D:/autostack/.wt/lang-831/auto-lang / branch: codex/plan-831-dev / base_commit: 2dafba909 / code_commit: e199caf8a / dependency_revisions: auto-ui@f7d92ee（T-02 组树，未触）；auto-down 主检出只读 / task_ids: T-03 / evidence: 预迁移态实跑 `python scripts/ui-repository-fixture.py` = pass 25/fail 0/recorded 6/pending 12（收据 .wt/lang-831/scratch-ui-repo-fixture/run-receipt.json，两次运行字节一致）；selftest 10/10 区分性 oracle（vendoring 守卫/非 legacy 走私边点名/收紧态 SRC-P2 点名 auto-man-core/合法对照绿/部分导入 exit 3）；运行面 py_compile+JSON/TOML 解析过；Category A（无 crates/ 改动）未跑 cargo 门，merge 冻结门 pending 归 T-10/T-11 / 实测修正: Man-core 过渡台账=T-01 记 1 边、实为 2 边（auto-api-gen→auto-man-core 亦在五包闭包内，C2 范围含 tools+API 生成两消费方，contract-baseline §1.3/§3 本就双列）——migration-phases.json 台账按实测冻结 / blockers: 无 / next_responsible_role: work T-04（来源导入旧 UI 仓迁移分支，依赖 T-02/T-03 已满足）

2026-10-24 / stage: work(G831 启动门核验+T-01 开始) / plan_id: PLAN-831 / revision: 2 / outcome: **G831 满足，启动锁解除，进入 T-01** / gate_evidence: U815=archive/815（delivery `33c9451fb` git merge-base --is-ancestor master ✓，status archived，17/17，五检查点闭环收据 6336ab95b，ledger v68）；U816=archive/816（delivery `ac7c1369c` master 祖先 ✓，status archived，14/14，五检查点收据在 §9 续 14/15，ledger v76）；无误建 lang-831 树；目标仓 D:/autostack/auto-ui 在场 / baseline: 源版本=master@a61209cc0（含他 session 簿记）（815+816 合入后实际源）；上游 revision：815 r?/delivery 33c9451fb、816 r2/delivery ac7c1369c；Spec/ledger=v76 / next: T-01 契约基线审计（attachments/831/contract-baseline.md）


r1以下记录为历史合同预检，其提前准备/并行放行和pass仅适用于旧合同，已由r2启动锁取代，不是当前执行授权。

2026-10-22 / stage: new / plan_id: PLAN-831 / revision: 1 / outcome: pass（规划合同预检，非实现验收）/ next: work T-01及门内T-02/03准备；T-04等U815，T-05起等U816 / changed: T-01..12、AC-01..08、SD-01..07、V-01..06。见[预检](attachments/repo-delivery-batch-20261022/preflight.md)。当前无任务完成、未运行未来driver/Cargo/搬仓。

2026-10-22 / stage: new / plan_id: PLAN-831 / revision: 2 / outcome: blocked（仅执行启动门；r2文档合同预检通过）/ next: 等待G831真实满足后重新核验完整交付/归档与源版本，再work T-01 / changed: T-01..12全局启动依赖、§0/4/5/6/8/10调度；原目标、AC/SD/V ID与发行验收保留。当前815/816仍executing、831尚未交付，故不启动实现/准备/建树/agent；Plan保持drafting 0/12。见[预检](attachments/repo-delivery-r2-serial-20261022/preflight.md)。

## 10. 待澄清事项

调度已明确，无须重复确认；启动锁是用户本轮要求，不能用早期并行建议覆盖。T-01有界解决package/compat/source/共享声明布局与实际支持环境；机制选择可等价，但不能以固定package名字、默认feature或GUI环境未知为由偷偷缩面。若必须改变现行公开兼容/支持、复制状态或引入新仓/远端动作，具体needs_replan后只暂停受影响任务。832全部准备/实施必须等待本计划整Plan完整交付及归档收尾，不能以T-11局部发布提前放行。
