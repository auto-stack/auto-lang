---
plan_id: PLAN-816
status: drafting
feature_name: 工具执行适配、管理服务与薄 CLI 组合收口
author: [Codex]
created_at: 2026-10-19
updated_at: 2026-10-19
plan_revision: 1
current_step: 0
total_steps: 11
supersedes_spec_components: [docs/specs/auto-cli/project.md, docs/specs/auto-man/project.md, docs/specs/stdlib/design/http-service-deployment.md, docs/specs/repository-boundaries/generated-ui-distribution.md]
new_spec_components: [docs/specs/tool-execution/project.md]
touched_goals: [GOAL-003]
affects: [auto-cli, auto-man/service, tool-execution, language-adapters]
depends_on: [PLAN-809, PLAN-814, PLAN-815]
---

# [PLAN-816] 工具执行适配、管理服务与薄 CLI 组合收口

## 0. 变更摘要

完成R06：消费已合入管理核心、纯语言SDK和I-TOOL/UI工具，整理语言/服务/系统构建adapter与工具执行设施，auto CLI保持命令体验但只做解析/路由/显示，管理正常服务闭包不静态链接UI工具实现。脚本/工程/UI/服务/watch/daemon等真实生产链、stdio/exit/取消/资源与独立发行一起收口。

不再抽一套UI生成器/工具协议（815拥有），不重做809管理算法、814 UI runtime或792发布器；不搬旧Git仓/切默认分支，不将兼容退役R11提前取消旧命令。

## 1. 目标

- auto命令flags/默认、脚本argv透传、JSON输出、工作目录/env/端口/后端选择和现行错误支持矩阵保持；UI命令通过实际tool能力/进程适配，非UI脚本不加载UI实现。
- 管理核心/服务、语言执行、UI工具、CLI边界清楚；旧auto-man若暂保UI public门面，标为上层兼容，不能进入纯管理服务正常闭包。
- 一份实际执行设施拥有子进程、stdin/stdout/stderr、deadline/cancel/reader/watch/daemon寿命；现有设施先盘点复用，不新建第二套永远并存runner。
- 默认完整交付profile具备所需tool及资源，不以“分包后请手工安装”悄缩既有支持；版本/能力/缺件/失败有准确受控出口，不偷偷fallback到静态UI大库。

## 2. 架构方案

`auto CLI → 管理服务/轻协议/语言SDK`，管理服务复用809核心及明确FS/Git/工具链/语言adapter；UI执行用815实际tool。CLI正常闭包无UI compiler/runtime/tools或旧auto-lang兼容大包，管理服务同样不反向链接UI实现。

如旧auto-man公开UI模块需要保留，可保旧名上层compat，并新建具名`auto-man-runtime`服务包承接非UI管理/执行实现；CLI切真正服务，旧Pac/Target等保持单源类型/方法，不复制可变工程状态。是否需要该包由T-01有界API/DAG probe冻结；禁止保旧UI重导出又宣称同一服务包纯净。

## 3. 技术栈

现有clap/Man核心/语言SDK、815 I-TOOL、std进程和既有watch/service执行设施。跨平台process/stdio控制按原支持面提供，不把daemon/独立service混为一个协议。协议数据只消费815，不重新声明project/request或版本字段。

## 4. 需求分析与背景调查

用户授权本批规划并要求明确合入等待；本轮仅文档，不实施/创建agent或树、不写远端。[路线](reports/v06-extraction-execution-roadmap.md)、[批次调度](reports/sdk-tools-parallel-order-814-816.md)、[输入hash](attachments/sdk-tools-batch-20261019/inputs.json)为来源。

权威：[CLI](../specs/auto-cli/project.md)、[Man](../specs/auto-man/project.md)、[独立HTTP服务](../specs/stdlib/design/http-service-deployment.md)、[发行](../specs/repository-boundaries/generated-ui-distribution.md)。实际入口`crates/auto/src/{main,cmd_watch,cmd_service,cmd_ui,cmd_bp,cmd_stdlib}.rs`、`auto-man/src/{automan,http_service,builder,git,resolver,pac,target}.rs`与UI工具桥；有cmd_vue/cmd_tauri等orphan文件，不能当已挂载命令。`auto service`为独立服务、`auto serve`为daemon，前者不kill占端口进程，配置优先级/ready/exit有现行硬合同。

CLI当前直接依赖auto-lang/auto-man，main含工程路由/运行env以及UI/后端编排；cmd_watch持源扫描/缓存与HMR；http_service有VM进程内与Rust生成/启动等待，其他命令也各有Command/reader/kill逻辑。T-01从真实调用盘点，未调查不能宣称已有完整supervisor或先造平行runner。

### 开工门

- T-01只读命令/执行设施/支持矩阵可立即开始，不冻结815/809 WIP API。
- T-02可在唯一树实施**与新接口无关且不改他线共有文件**的现有执行设施封装/定向正反例；T-01若发现无法分离的共享写面则等对应owner合入后做，不为提前开工造第二runner。不改809 Builder或当前路由、不给未来协议先造替身，不激活新命令路径。
- **T-03起最终服务/协议/CLI接线须等809、814、815整体合入**；814通常已被815前置覆盖，但显式核对交付。不得cherry-pick、分支堆叠或消费未合入I-TOOL。

## 5. 详细设计

### 5.1 命令与adapter全集

T-01从clap树/挂载模块形成逐命令矩阵：裸REPL/脚本.at/.as与透传、语言trans/docs/stdlib、new/init/build/run/test/clean/add/fetch/deps/export、UI/backend/blueprints、service/serve/req/watch及实际其他支持。每项列参数/默认/机器输出/环境/输出目录/依赖工具/生命周期、真实owner与测试。orphan命令不假新增，新flags/退役单独决策。

CLI只保clap与显示/路由，业务/扫描/增量cache/构建与运行策略移入对应服务或UI工具；Compiler/VM/trans/config加载属语言adapter，工具链/PATH/Git/Builder等属管理adapter，UI/生成与资源属815。生成server所需通用API与UI外壳以815 owner清单交接，不能再复制一份api_gen或把全部server依赖推UI工具。

### 5.2 执行资源与结果语义

复用现有facility并将必要共享缺口合一：argv数组传参、cwd/env显式、stdin交互/继承与管道、stdout/stderr流/JSON/control隔离、退出/错误分类、取消/deadline、watcher与reader/子进程树退出。不持管理/VM锁等待或回调，不将!Send VM跨OS线程；进程内VM服务与独立UI进程各守原线程归属。

正常退出/失败/取消/超时/输出或资源限额不同，必须观察owned子进程/reader/watcher真正停止；不把caller返回或kill发出当已reap。新设施的超时/取消清理只处理本次拥有进程，不强杀共享auto.exe/无关服务；旧run链已有的显式legacy端口处理策略由T-01按现行合同具名隔离，不能扩散到新工具/fixture或`service`。保`service`端口冲突非零、无假ready且不kill端口占用者。所有新预算在T-01冻结单位/入口/诊断/边界控制，不把既有合法脚本输出无声截断。交互命令保stdin/TTY及输出次序适用合同。

### 5.3 工具发现与默认可用性

按815能力/版本/producer身份选择实际tool，生产路径不搜索任意兄弟仓或隐式构建依赖；独立安装profile包含相应二进制/资源/锁。开发工作流须有明确、可重放的发现/构建或安装接线：首选已配置/已安装匹配tool；若支持源码开发按需构建，由运行期具名开发adapter/脚本完成，不能build.rs偷偷编译UI或Core命令启动就拉全部UI。

默认完整profile维持原UI功能；新增纯Core profile可以无UI，但必须显式区分，不能拿缺UI的Core profile冒充旧完整交付。缺工具/版本不兼容/能力不足在执行前具名失败，无静态大库偷偷fallback；首次UI调用、重定位、升级、工具崩溃和普通脚本无UI安装均有实际控制。具体bootstrap布局T-01结合814/815合入后probe冻结。

### 5.4 兼容、发行与测试

旧auto/auto-man公开入口和flags按矩阵保留，兼容facade与正常服务闭包分开；词法/AST/VM/请求身份复用，服务层Pac/Target等单份状态。UI资源发布与producer/lock仍复用815/792，不新写万能发布器；错误/部分发布按既有profile及新I-TOOL承诺精确验证，不把803遗留破坏性模板窗口顺手改造。

777精确边、792/安装manifest含新CLI/service/protocol/tool全闭包，Core消费不需UI源；实际CI/脚本调用别名与旧CLI file hash/产物身份有交接。跨仓承接R08/R09以后再做，本Plan先提供真实可分发命令组合，不能fake tool判全链完成。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/tool-execution/project.md | 分散启动/等待→具名设施/adapter、进程/stdio/取消/发现/bootstrap支持 | 真执行边界 | AC-02, AC-03 |
| SD-02 | modify | docs/specs/auto-cli/project.md | 大量业务/静态UI→薄CLI/真实owner与默认兼容/命令矩阵 | 入口可独立交付 | AC-01, AC-04 |
| SD-03 | modify | docs/specs/auto-man/project.md | 纯核心/服务/旧兼容/语言与UI工具adapter分层 | 无反向UI闭包 | AC-01, AC-04 |
| SD-04 | modify | docs/specs/stdlib/design/http-service-deployment.md | 入口搬迁/执行owner明确，原config/port/ready/exit合同保持 | 防service/serve漂移 | AC-02, AC-04 |
| SD-05 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md | 命令/工具profile与producer/resource/bootstrap消费闭包 | 真独立消费 | AC-03, AC-04 |

## 6. 测试设计

T-01冻结命令/旧oracle/执行facility与bootstrap；T-07建新增`test/fixtures/modularity/tool-execution-consumer/`和CLI真实进程plan816 driver。V均owner_stage=work，当前未运行。

| Case | AC / invariant | 输入/真实入口 | 区分性oracle/边界 |
|---|---|---|---|
| V-01 | AC-01 真依赖/命令路由 | 真auto二进制裸脚本/透传/REPL/管理与语言命令；独立Core profile，无UI checkout/tool | CLI/管理服务normal/build/dev无UI实现/旧大库；仍实际执行语言/管理核心；CLI无第二业务算法，旧同名包兼容不污染纯服务 |
| V-02 | AC-02 执行寿命 | 真实正常/exit非零/大stdout+stderr/stdin交互/超时/取消/child+grandchild/reader错误/watch变更 | 参数/cwd/env/输出/exit保持；超时后进程/reader/watcher实际退出，非只parent返回；已配置预算正反控制，清理不波及无关进程 |
| V-03 | AC-03 工具/发现 | 真815 tool正常UI、缺件/版本错/能力不足/坏结果/崩溃/取消，首次开发启动/已安装/重定位完整profile | 真实生成/build/run可用，缺能力前无副作用，logs/control不串；无偷偷静态UIfallback，Core启动不拉UI；tool真实身份可追溯 |
| V-04 | AC-04 旧行为与组合 | 实际命令矩阵、--format json、service vm/rust+端口冲突/配置覆盖/ready，serve/req、watch/HMR、UI代表profile及fresh/locked/升级失败 | 旧flags/结果与支持面保持；service不杀占用进程/不假ready；真正CLI→服务→tool→产物链成功，失败与旧产物/资源恢复按适用合同，777/792负控红 |

work只相关check/包测试/执行设施与CLI定向driver；独立consumer源码与测试进程均T-02/T-07建立，自带截止/拥有的子进程清理。T-01在接口门后冻结实际包名/命令，`cargo check -p auto -p auto-man`及最终真实service package；不会在新包不存在时声称跑过。真实工具chain必须work验收，fake仅异常控制；环境缺失pending。

独立final review同树独立上下文验全部AC/SD/命令矩阵及资源；merge冻结提交裸cargo t、实际CLI/Man/协议包门，VM/trans/UI生成触面按最终diff加tv/tt/tu。已有服务真TCP定向也必须有适用证据；开发/送审/复审不整档，tf主检出到期单实例。未指定agent/model不创建会话，本轮文档不跑Cargo/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | 薄CLI/管理服务真消费核心、正常闭包无UI大库，旧API/命令owner完整，业务算法单源；V-01 |
| AC-02 | work | argv/cwd/env/stdio/exit/取消/deadline及owned进程/reader/watch退出完整，VM线程亲和及service不kill保持；V-02 |
| AC-03 | work | 真815工具协议/发现/开发bootstrap/安装profile可用，缺能力无隐式fallback/副作用；V-03 |
| AC-04 | work | 真实命令/服务/watch/UI支持及错误矩阵、777/792/fresh/locked/身份与失败证据通过、SD/来源齐全；V-04 |
| AC-05 | review | 同树独立最终修订审查/遗漏/警告/格式/debug通过，merge门pending明确 |
| AC-06 | merge | 冻结门、Spec/ledger/归档、CLI/service/tool/profile交接与guard clean后清理有实际收据 |

## 8. 执行步骤

11项；T-01只读可现在，T-02独立执行设施可先，T-03起等809/814/815实际合入。

| 状态 | ID | owner_stage | 依赖/启动门 | 文件/交付与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 现在只读；新接口probe等门 | attachments/816/contract-baseline.md（新增）：真实clap命令/旧oracle/adapter归属/现有runner与资源、服务和compat DAG、bootstrap/profile/预算候选；门后整链probe定稿 | AC-01..04 |
| [ ] | T-02 | work | T-01 facility映射，无WIP引用 | 在唯一树复用/封装现有执行设施及独立argv/stdio/进程退出正反例；不改809 Builder/既有路由，不实现临时协议替身/重复runner | AC-02 |
| [ ] | T-03 | work | T-01决策+809/814/815已合入 | 管理服务/语言adapter与上层compat真实单源分层、依轻I-TOOL；旧Pac/Target/API保持，check与V-01 | AC-01 |
| [ ] | T-04 | work | T-03 | 真tool discovery/版本/能力/stdio/cancel与完整profile/developer bootstrap，接815实际tool，V-03 | AC-03 |
| [ ] | T-05 | work | T-04 | main/clap及管理/语言/UI路由改薄组合、业务迁对应owner，旧命令/argv/JSON/flags矩阵，V-01/04 | AC-01,04 |
| [ ] | T-06 | work | T-05 | service/serve/req/watch/HMR与子进程/reader/watcher/VM线程归属全链接线，V-02/04 | AC-02,04 |
| [ ] | T-07 | work | T-06 | 新consumer/真实CLI driver V-01..04、777/792/完整与Core profile独立fresh/locked/重定位及失败负控 | AC-01..04 |
| [ ] | T-08 | work | T-07 | SD-01..05、命令/ABI/资源/producer/cli hash交接与attachments/816/handoff.md（新增），提交送审 | AC-04 |
| [ ] | T-09 | review | T-08 | 同树独立完整命令/资源退出/实际tool/profile审查，保留merge门pending | AC-05 |
| [ ] | T-10 | merge | T-09 pass | master并集/变化复审、冻结统一适用门，失败归因修复后重审 | AC-06 |
| [ ] | T-11 | merge | T-10 | 规范/ledger/归档、真实CLI/service/tool消费交后续发行/搬仓，guard clean清理与计数 | AC-06 |

唯一树`D:/autostack/.wt/lang-816/auto-lang`，分支`codex/plan-816-dev`，同树work/review/merge，work确认实际base/兄弟依赖；禁止叠加815送审代码/自建阶段合流，本合同无phase merge，必须整体独立审查/合入。无WIP依赖，不在树内建junction/symlink。

## 9. 复审记录

2026-10-19 / stage: new / revision: 1 / outcome: pass（合同预检，非实施/独立验收pass）/ next: work T-01及合边界的T-02，T-03起等809/814/815合入 / changed: T-01..11、AC-01..06、SD-01..05。当前未实施/运行未来测试或创建工作树/agent。

## 10. 待澄清事项

无需用户重复授权；T-01具名冻结兼容/管理服务包布局、现有facility复用、交互与bootstrap。若必须改变旧命令/支持、默认安装增加不可接受人工步骤、破坏public API或跨仓写入，具体needs_replan。不能凭fake工具/只测CLI退出码宣布进程和全链交付完成。
