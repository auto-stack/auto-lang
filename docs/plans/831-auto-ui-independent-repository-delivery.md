---
plan_id: PLAN-831
status: drafting
feature_name: auto-ui 独立仓承接与完整发行交付
author: [Codex]
created_at: 2026-10-22
updated_at: 2026-10-22
plan_revision: 1
current_step: 0
total_steps: 12
supersedes_spec_components: [docs/specs/repository-boundaries/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md, docs/specs/auto-ui-compiler/project.md, docs/specs/auto-ui-runtime/project.md, docs/specs/auto-ui-tools/project.md, docs/specs/auto-ui-dialect/project.md, docs/specs/language-sdk/project.md, docs/specs/project-model/project.md, docs/specs/auto-lang/project.md, docs/specs/auto-man-core/project.md, docs/specs/auto-man/project.md]
new_spec_components: [docs/specs/auto-ui/project.md, docs/specs/repository-boundaries/ui-repository-delivery.md, docs/specs/auto-api-gen/project.md]
touched_goals: [GOAL-003, GOAL-007]
affects: [auto-ui/repository, language-sdk, auto-lang/compat, project-model, repository-boundaries]
depends_on: [PLAN-814, PLAN-815, PLAN-816]
---

# [PLAN-831] auto-ui 独立仓承接与完整发行交付

## 0. 变更摘要

完成R08并吸收R07 UI发行：把814/815已交付UI文法、compiler/runtime/tools及其实际资源/测试/示例/脚本/CI与上层兼容承载迁进**既有auto-ui仓**；切换生产消费者，删除语言仓原实现，以独立clone/第三方工程/安装与失败恢复证明可独立交付。不是把当前crates再复制一份存档。

前置未齐可按§8开展准备；完整UI候选导入等U815，生产共享边界/语言切源等U815+U816。不重做815/816运行合同，不重开814；不在本Plan做UiAst终态/大规模去重/后端退役或远端发布。

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

用户2026-10-22明确要求建立两份大模块搬仓合同并包含发行验收、明确与815/816并行。**本轮仅在语言主检出写规划，不执行跨仓修改/保全/建树/推送**；未来work按本合同范围与既有授权开展，外仓权限按执行环境实际授予。本地既有仓追加提交与经审查的默认分支推进是计划交付；远端更名/force/push不是隐含授权。

当前Spec从[全局入口](../specs/overview.md)定位，以[SDK](../specs/language-sdk/project.md)、[compiler](../specs/auto-ui-compiler/project.md)、[runtime](../specs/auto-ui-runtime/project.md)、[tools](../specs/auto-ui-tools/project.md)、[旧仓治理](../specs/repository-boundaries/project.md)、[发行](../specs/repository-boundaries/generated-ui-distribution.md)为准。总览部分数量/旧描述滞后，以模块Spec+代码/收据为证。814 B=93a25c924及后继丢测修复f97a9e2a9已合入；815仍executing、B运行与验收修复，816仍executing、完整CLI待门，未当成已交付。

已观察写面：`crates/auto-ui-{dialect,compiler,runtime,tools}`、旧`crates/auto-lang`上层再导出、`crates/shell-pack`、`crates/auto-api-gen/src/sidecar.rs`、`crates/auto-ui-tools/src/{vue,rust_ui}.rs`、`crates/auto-man-core`声明来源及`crates/auto-project-model`；Cargo/build.rs/资源及`test/fixtures/modularity/{ui-*,generated-ui-consumer,language-sdk-consumer}`、`.cargo/config.toml`/nextest/CI/777/792均需真实映射。

旧仓`../auto-ui/Cargo.toml`与CLAUDE.md已读；旧指令“master已清空”在784已证过时，**本轮未重采refs/WIP，不用历史tip假充今日基线**。784保全及恢复输入见[历史报告](reports/784-repo-preservation.md)，执行T-01/02刷新。冻结输入hash、源SHA与前置状态见[本轮输入](attachments/repo-delivery-batch-20261022/inputs.json)。

## 5. 详细设计

### 5.1 启动门与下一任务入口

整项depends_on为最终摘要；**现在T-01可只读开始，T-02/03在独立组可与815/816实施/审查并行**，不得让粗粒度自动依赖把它们一并堵住。现有Runner未新增阶段调度，按Markdown任务门组织。U815/U816定义与当前已闭/未闭前置见本批合同§1。T-04只等U815且只在目标候选工作，不改在途816输入；T-05起等U816。每个门须读回master祖先/Spec/实际版本，不用WIP分支或阶段pass替代。

T-01的关闭件为新增`attachments/831/contract-baseline.md`：逐包/输入/测试/旧消费者owner与DAG、profile支持面、来源版本/授权写面、legacy当前状态、兼容布局及共享声明切口可行性（只用已交付接口）；未知项标observed/hypothesis，具名阻断后续步骤。T-01完成的是当前已知范围的可执行设计和未知项归属，未来B源码/probe的重采归T-04、共享切口实编归T-05，不让T-01又隐性等待U815/U816。无需用户再批准已经授权的常规布局选择。

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

发行/安装事务复用792，其保旧字节/owned清理/误删保护按原精确合同；升级/坏版本/半复制/取消/producer漂移有反例。三仓切换按本批§4 prepared→landed→switched→verified收据，无跨Git原子假设。下游832只消费本PlanT-11实际发布的固定版本，不因UI候选能跑就切正式来源。

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
| AC-07 | review | 独立final review绑定r1与全部仓冻结SHA/依赖输入，AC/SD/遗漏/告警/格式/debug审查通过；merge门pending |
| AC-08 | merge | 冻结门通过、目标本地分支及语言消费者实际交付/来源读回与Spec/受管ledger齐备；给832版本收据，guard后清理归档 |

## 8. 执行步骤

12项，按依赖列；U815/U816必须实际落地。T-01准备不是已授权自动执行后续搬仓；本轮交付合同后交work按用户执行指令开始。

| 状态 | ID | owner_stage | 依赖/启动门 | 写面/交付与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 现在可只读；仅用已发布输入 | 冻结attachments/831/contract-baseline.md，逐包/资源/测试/兼容/DAG与源码版本、共享声明切口可行性，未闭决策阻断对应生产任务 | AC-01..06 |
| [ ] | T-02 | work | T-01旧仓事实；可与815/816并行 | 重采旧UI仓指令/refs/WIP/ignored、保全快照/锚与恢复演练，建立或认领本Plan唯一隔离树，新增preservation.md | AC-05 |
| [ ] | T-03 | work | T-01；与T-02/815/816独立 | 新ui-repository-consumer及发行/失败/source/type/测试ID driver，基于已交付814/A做有效控制；不改生产热点 | AC-01,03,04,06 |
| [ ] | T-04 | work | T-02/T-03+U815；不等U816 | 冻结815完整来源，UI包组/资源/测试/许可/兼容候选导入旧UI仓迁移分支；只改目标候选/清单，保持语言权威与816输入 | AC-01,02,05 |
| [ ] | T-05 | work | T-04+U816，已完成共享写面交接 | tools/API生成共享声明最小单源去Man边，旧Man API再导出；冻结compat/剩余消费者/包版本owner，定向V-01/02 | AC-01,02 |
| [ ] | T-06 | work | T-05 | 目标独立workspace/build/manifest/锁/固定源；语言生产消费者切源/旧UI实现删除、别名/测试ID与777/792源根更新，V-01/06 | AC-01,02,06 |
| [ ] | T-07 | work | T-06 | 真目标clone/外部工程/安装tool与资源完整profile，fresh/locked/重定位/默认兼容、双端实机V-02/03/06 | AC-02,03,06 |
| [ ] | T-08 | work | T-07 | 792受管升级/缺源/坏hash/漏patch/取消/交换恢复及多仓切换失败控制V-04/05，owned残留与恢复演练 | AC-04,05 |
| [ ] | T-09 | work | T-08 | 按最终来源复核全部V/支持面/净迁移非净删、SD-01..07与实际CI入口，attachments/831/handoff.md冻结交付候选 | AC-01..06 |
| [ ] | T-10 | review | T-09冻结提交 | 独立三仓输入/AC/SD/真实消费与恢复审查，发现遗漏/降级归因，final pass后merge门pending | AC-07 |
| [ ] | T-11 | merge | T-10 pass；共享发布窗口 | master并集影响复审，冻结统一适用门；本地目标分支/语言消费有序落地，生产source/版本/工具hash/Spec来源读回，新增ui-release.md放行832 | AC-08 |
| [ ] | T-12 | merge | T-11全部收据 | canonical Specs/受管ledger及索引沉淀、归档；保留legacy/恢复/已交付分支，wt-guard clean逐树清理本Plan临时树/分支/组 | AC-08 |

执行组`D:/autostack/.wt/lang-831/{auto-lang,auto-ui}`，分支各`codex/plan-831-dev`（属于各自仓）；必要auto-down兄弟依既有解析补detached，**不修改其实现**。784树如已有owner不得抢占，确认保全后另建本Plan分支或经明确owner交接认领，始终每仓仅一个权威树。review不建第二checkout。主检出只簿记，树内禁junction/symlink；移除前逐树wt-guard clean，不删legacy或恢复载荷。

## 9. 复审记录

2026-10-22 / stage: new / plan_id: PLAN-831 / revision: 1 / outcome: pass（规划合同预检，非实现验收）/ next: work T-01及门内T-02/03准备；T-04等U815，T-05起等U816 / changed: T-01..12、AC-01..08、SD-01..07、V-01..06。见[预检](attachments/repo-delivery-batch-20261022/preflight.md)。当前无任务完成、未运行未来driver/Cargo/搬仓。

## 10. 待澄清事项

无须用户重复授权的当前规划问题。T-01有界解决package/compat/source/共享声明布局与实际支持环境；机制选择可等价，但不能以固定package名字、默认feature或GUI环境未知为由偷偷缩面。若必须改变现行公开兼容/支持、复制状态或引入新仓/远端动作，具体needs_replan后只暂停受影响任务。832最终UI消费等待本计划T-11实际交付，不等待本Plan归档清理。
