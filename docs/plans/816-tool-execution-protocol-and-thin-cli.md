---
plan_id: PLAN-816
status: executing
feature_name: 工具执行适配、管理服务与薄 CLI 组合收口
author: [Codex]
created_at: 2026-10-19
updated_at: 2026-10-21
plan_revision: 2
current_step: 3
total_steps: 14
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

本次r2按M816-A真实阶段发布、M816-B完整交付组织，不增加Plan编号、不降低原最终AC。执行与审查修复并行规则见[阶段交付与并行审查修复合同](reports/staged-delivery-review-814-816.md)；阶段pass不等于整Plan交付。

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

用户2026-10-20明确要求修订现有814/815/816，允许阶段交付及接口合入后放行下游、并行审查验证修复，不增加编号。原目标/最终AC/支持面不变；本轮只改主检出规划文档，不改实现树、canonical Specs或远端。 来源与当前hash见[本轮输入](attachments/sdk-tools-r2-20261020/inputs.json)。

用户授权本批规划并要求明确合入等待；本轮仅文档，不实施/创建agent或树、不写远端。[路线](reports/v06-extraction-execution-roadmap.md)、[批次调度](reports/sdk-tools-parallel-order-814-816.md)、[输入hash](attachments/sdk-tools-batch-20261019/inputs.json)为来源。

权威：[CLI](../specs/auto-cli/project.md)、[Man](../specs/auto-man/project.md)、[独立HTTP服务](../specs/stdlib/design/http-service-deployment.md)、[发行](../specs/repository-boundaries/generated-ui-distribution.md)。实际入口`crates/auto/src/{main,cmd_watch,cmd_service,cmd_ui,cmd_bp,cmd_stdlib}.rs`、`auto-man/src/{automan,http_service,builder,git,resolver,pac,target}.rs`与UI工具桥；有cmd_vue/cmd_tauri等orphan文件，不能当已挂载命令。`auto service`为独立服务、`auto serve`为daemon，前者不kill占端口进程，配置优先级/ready/exit有现行硬合同。

CLI当前直接依赖auto-lang/auto-man，main含工程路由/运行env以及UI/后端编排；cmd_watch持源扫描/缓存与HMR；http_service有VM进程内与Rust生成/启动等待，其他命令也各有Command/reader/kill逻辑。T-01从真实调用盘点，未调查不能宣称已有完整supervisor或先造平行runner。

### r2阶段启动门

- 809已交付；T-01/T-02现有执行设施可现在实施，不改其他owner热点，不造平行runner。
- **M816-A**：T-12实际协议客户端等M815-A发布（不等815-B或814）；随后T-13独立审查、T-14实际发布。
- **M816-B**：T-03管理/语言adapter等M814-A+M816-A（M815-A由后者覆盖）；T-04可先验证已发布窄能力与缺UI能力负控，完整UI正控等M815-B。
- T-05默认CLI/完整UI路由切换及最终集成必须等M814-B+M815-B；T-06..11完成全部原AC。不消费送审分支或自建镜像协议。

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

### 5.5 A/B阶段的真实交付边界

**M816-A**交付现有执行设施合一及真实I-TOOL客户端，独立consumer调用已发布M815-A的实际describe/元数据校验。参数/cwd/env/stdio、版本/缺能力/错误、日志control隔离、超时/取消及owned child/reader退出有正反例。真实窄tool不能代替长运行/孙进程等寿命反例，后者用自带截止的具名测试子进程补充；fake不能替代实际协议正控。不依赖814、未发布UI能力或静态UI库，不切旧默认CLI。

**M816-B**可在M814-A/M816-A发布后开发管理服务/语言adapter和窄能力发现，并行做独立scope审查修复；默认CLI改薄及完整UI正控等M814-B+M815-B。全部原AC-01..04、首次开发bootstrap、安装完整profile、service/watch真实资源退出仍必做。SD-01/05仅A实际执行/协议消费子集先沉淀，CLI路由/管理分层/service搬迁和完整profile在B发布后沉淀。

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

### r2阶段验证与并行复审

| Case | AC / invariant | 真实入口/设置 | 区分性oracle与边界 |
|---|---|---|---|
| V-05 | AC-07 A阶段真实客户端 | T-12建立新增独立consumer→已发布815-A实际进程；实际窄能力与缺能力负控，另加长运行/reader/child测试进程 | 不造UI假正控；实际结果/错误/版本处理正确，超时取消后owned进程与reader实退，Core无UI安装可消费；不切默认CLI |

上述新增Case与原V项均owner_stage=work，由新增阶段work任务使fixture真正可运行，当前尚未执行。原测试表/最终AC不缩减；A只取其明确范围，B仍逐项验证完整支持面。阶段发布的API/能力/依赖/source/资源和测试选择变化须有同步正反控，不把必需consumer验证全部后置。

实施者先修齐本轮已知findings，定向回放共享反例集/相邻状态组合及必要check，再提交新SHA送审。同Plan同一权威树冻结交接，不能边改边让review读工作区；不同Plan独立scope的读审、定向验证、修复可并行。缺上游的真实集成明确pending，不每轮重复算新finding、不用fake补绿。

A新增review任务只给阶段pass；原最终review任务仍验全部AC/SD。阶段和最终的裸t/触面整档均归merge冻结后统一排队，资源独占项串行；tf仅主检出到期单实例。接口修复由owner发布，影响方重放受影响scope，未变且适用证据可复用。具体失效/收据/单写者规则见[阶段交付与并行审查修复合同](reports/staged-delivery-review-814-816.md) §2–5。

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | 薄CLI/管理服务真消费核心、正常闭包无UI大库，旧API/命令owner完整，业务算法单源；V-01 |
| AC-02 | work | argv/cwd/env/stdio/exit/取消/deadline及owned进程/reader/watch退出完整，VM线程亲和及service不kill保持；V-02 |
| AC-03 | work | 真815工具协议/发现/开发bootstrap/安装profile可用，缺能力无隐式fallback/副作用；V-03 |
| AC-04 | work | 真实命令/服务/watch/UI支持及错误矩阵、777/792/fresh/locked/身份与失败证据通过、SD/来源齐全；V-04 |
| AC-05 | review | 同树独立最终修订审查/遗漏/警告/格式/debug通过，merge门pending明确 |
| AC-06 | merge | 冻结门、Spec/ledger/归档、CLI/service/tool/profile交接与guard clean后清理有实际收据 |
| AC-07 | work | M816-A真实执行客户端满足§5.5，V-05及V-02/03当期范围通过；不宣称薄CLI整体交付 |
| AC-08 | review | T-13独立阶段审查绑定r2/M816-A冻结SHA/实际上游交付及Spec子集，通过且merge门pending |
| AC-09 | merge | T-14冻结门通过、实际master祖先交付/Spec与执行consumer来源收据；不归档 |

## 8. 执行步骤

14项；保留原任务ID，追加A阶段任务，**按依赖列执行，不按编号排序**。整项depends_on为摘要，阶段启动以§4/5和本表为准；不宣称Runner已支持阶段调度。

| 状态 | ID | owner_stage | 依赖/启动门 | 文件/交付与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 现在；809已交付；SDK/tool probe按阶段门 | attachments/816/contract-baseline.md（新增）：真实clap命令/旧oracle/adapter归属/现有runner与资源、服务和compat DAG、bootstrap/profile/预算候选；门后整链probe定稿。[✅ 已完成] 基线@master 9e80ae0fb：33 命令矩阵+全局旗标+orphan 裁定（cmd_vue/cmd_tauri 未挂载）、执行设施 12 站点映射、auto-man-runtime probe=需要（auto-man 硬链 ui-iced 证据）、T-02 合同与预算入口/单位冻结、legacy 端口清场隔离；bootstrap/profile 候选待门后整链 probe（§7 记录，门未开非债） | AC-01..04 |
| [x] | T-02 | work | T-01 facility映射，无WIP引用 | 唯一设施、argv/stdio/进程退出正反例；不改809 Builder/既有路由，不造重复runner。[✅ R6 修复完成 @05d3940c6] F-R3-02 三序闭合（①stop 先到→Windows 写侧入口拒写 Ok(0),写永不入核;②写已挂起→cancel-then-close 实测中止;③写已完成→正常字节）;stop_and_join 改 is_finished 轮询+join 收割（不消费 done 通知）;R6 cancel-race 注入过（outcome=Stopped/writer_still_in_write=false）。F-R6-01 平台隔离：CancelSlot/CancelCredential 中立别名,client 同步,Linux 两包+独立 consumer check 全过（R6 platform replay exit0）。残项同步：client wait_writer_done 早退与 wait_deadline Err 走 cleanup_then_error;OwnedProcess::drop 有界收尸。R6 @a972286fd 判定证据与[独立R6](attachments/816/review-r6/review.md)为源,历史见§9 | AC-02 |
| [ ] | T-03 | work | T-14/M816-A+M814-A已发布 | 管理服务/语言adapter与上层compat真实单源分层、依轻I-TOOL；旧Pac/Target/API保持，check与V-01 | AC-01 |
| [ ] | T-04 | work | T-03；窄tool已发布；完整UI正控等M815-B | 实际discovery/版本/能力/stdio/cancel；先验M815-A支持面及缺UI能力负控，完整profile/bootstrap正控等M815-B；V-03 | AC-03 |
| [ ] | T-05 | work | T-04+M814-B/M815-B已发布 | main/clap及管理/语言/UI路由改薄组合、业务迁对应owner，旧命令/argv/JSON/flags矩阵，V-01/04 | AC-01,04 |
| [ ] | T-06 | work | T-05 | service/serve/req/watch/HMR与子进程/reader/watcher/VM线程归属全链接线，V-02/04 | AC-02,04 |
| [ ] | T-07 | work | T-06 | 新consumer/真实CLI driver V-01..04、777/792/完整与Core profile独立fresh/locked/重定位及失败负控 | AC-01..04 |
| [ ] | T-08 | work | T-07 | SD-01..05、命令/ABI/资源/producer/cli hash交接与attachments/816/handoff.md（新增），提交送审 | AC-04 |
| [ ] | T-09 | review | T-08 | 同树独立完整命令/资源退出/实际tool/profile审查，保留merge门pending | AC-05 |
| [ ] | T-10 | merge | T-09 pass | master并集/变化复审、冻结统一适用门，失败归因修复后重审 | AC-06 |
| [ ] | T-11 | merge | T-10 | 规范/ledger/归档、真实CLI/service/tool消费交后续发行/搬仓，guard clean清理与计数 | AC-06 |
| [x] | T-12 | work | T-02+M815-A已发布；不等814/815-B | 真实client/consumer、V05及当期反例/来源/777/Spec。[✅ R6 修复完成 @05d3940c6] F-R3-02/F-R6-01 已闭——①三序闭合（写侧入口拒写/cancel-then-close/is_finished+join 收割,R6 cancel-race 注入 outcome=Stopped、writer_still_in_write=false）;②平台隔离（CancelSlot/CancelCredential 中立别名,Linux 两包+独立 consumer check 全过=R6 platform replay exit0）;③清理编排补齐（client wait_writer_done 早退/wait_deadline Err→cleanup_then_error;OwnedProcess::drop 有界收尸）。R1 5/R2 4/R5 0-failed/R6 两 replay 全过;三面 9+18/15/16+压力 6/6;clippy/fmt/rustdoc 0、verify-deps/777 strict/check 全绿。R6 @a972286fd 判定与[独立R6](attachments/816/review-r6/review.md)为源 | AC-07 |
| [ ] | T-13 | review | T-12提交冻结 | 独立A实际client/进程与reader/writer退出/来源/Spec审查，绑定r2/M816-A SHA；当前R6 @a972286fd needs_fix：F-R3-02 OPEN、F-R6-01 OPEN、F-R5-01 CLOSED；merge-owned门仍pending，见[独立R6结论](attachments/816/review-r6/review.md) | AC-08 |
| [ ] | T-14 | merge | T-13 pass | 串行冻结门/实际master合入、Spec子集/受管ledger增量与真实consumer读回发布收据；不归档不清树 | AC-09 |

唯一树`D:/autostack/.wt/lang-816/auto-lang`、分支`codex/plan-816-dev`；work/review顺序交接同树，先核实实际tree/base并读取主检出r2。A发布后保留原树/分支同步master继续B，不创建第二审查树、不归档/清理。A期间Plan保持executing，只有B全部原AC完成后才依标准流程execution_done→reviewed→archived。

阶段发布是用户本轮明确授权的有界例外：按共用合同独立阶段review、冻结门、人工Git合入及非归档Spec/ledger受管增量与收据组织；**不调用会自动归档整Plan的完整merge流程冒充phase支持**。B按原最终review/merge/归档任务完成。无WIP依赖，无junction/symlink，最终清理必须guard clean。跨Plan审查/修复与共享窗口见[阶段交付与并行审查修复合同](reports/staged-delivery-review-814-816.md)。

## 9. 复审记录

r1以下记录为历史合同预检，旧启动门由r2替代；不是当前调度指令。

2026-10-19 / stage: new / revision: 1 / outcome: pass（合同预检，非实施/独立验收pass）/ next: work T-01及合边界的T-02，T-03起等809/814/815合入 / changed: T-01..11、AC-01..06、SD-01..05。当前未实施/运行未来测试或创建工作树/agent。

2026-10-20 / stage: new / revision: 2 / outcome: pass（文档合同预检，非实现/独立验收pass）/ next: 按§4阶段门work；814先确认已登记唯一树并读取本r2。changed: 阶段A/B与新增T/AC/V、Spec分阶段沉淀、并行审查修复；原目标、原T/AC/SD ID保留。r1证据仅历史，不自动覆盖新范围。未执行实现/未来fixture/门禁。见[预检](attachments/sdk-tools-r2-20261020/preflight.md)。

2026-10-20 / stage: work / plan_id: PLAN-816 / revision: 2 / outcome: 当期可执行面完成（T-01/T-02 pass；非整 Plan execution_done——T-03..T-11/T-12..T-14 阶段门未开） / worktree_path: D:/autostack/.wt/lang-816/auto-lang / branch: codex/plan-816-dev / base_commit: 9e80ae0fb（master） / code_commit: 5fcdabe14 / dependency_revisions: 组内 auto-down 兄弟 worktree @c39ac7f9（detached，跨仓 path 解析）；无 WIP 接口引用（未消费 814/815 未发布面） / task_ids: T-01, T-02 / evidence: 基线 attachments/816/contract-baseline.md（命令矩阵/设施映射/DAG probe/预算冻结）；auto-tool-execution 15 测全绿（bare cargo test + nextest 默认档 .config/nextest.toml，3.7s）；clippy 零警告；cargo check -p auto -p auto-man 过 / blockers: 无缺陷性阻塞——阶段门：T-12 等 M815-A 发布（815 现处 T-13 阶段复审，未合入）；T-03 等 M814-A+M816-A；T-05+ 等 M814-B+M815-B / next_responsible_role: work（阶段门开后续 T-12；或按合同 §4 并行规则做独立只读面）

2026-10-20（续） / stage: work(T-12, M816-A) / revision: 2 / outcome: T-12 完成——M815-A 已发布（815 m815-a-release.md §5 放行条件齐备、v59 回填、816 阻断解除）后门开；本树先合 master b65d0e963 同步消费面（e609caf1af 三 crate 源零漂移验证） / worktree_path: D:/autostack/.wt/lang-816/auto-lang / branch: codex/plan-816-dev / sync_base: b65d0e963（merge 175b62e64） / code_commit: 0373cfa38 / baseline_commit: 6b4bf9c2f（777 基线刷新） / dependency_revisions: M815-A=master e609caf1af（协议 v1 describe/validate_project + 真实 auto-ui-tool + wire codec，path 依赖直接解析）；无 814 依赖 / task_ids: T-12 / evidence: 候选 attachments/816/m816-a-candidate.md §4 全表——fixture tool-client-consumer 10/10 真实进程（2.78s）+fresh/locked（43 包）+verify-deps PASS；client 单测 4/4、设施回归 15/15、clippy 0 警告、cargo check -p auto -p auto-man exit 0；777 strict pass（78 边/0 违规/2 grandfathered） / blockers: 无——T-13 独立阶段审查可开始（绑定 r2/冻结 SHA 0373cfa38+6b4bf9c2f）；merge-owned 门（裸 t/触面/fixture 复跑/Spec 沉淀）T-14 冻结后执行 / next_responsible_role: review（T-13；按阶段合同独立上下文验 AC-07/AC-08 与候选 §1–§6）

2026-10-20 / stage: review / review_scope: phase（M816-A/T-13，非整 Plan final） / reviewer_identity: 本聊天 Codex /root，独立于实施会话，从合同/代码/反例重建结论；模型/会话编号未暴露，未创建 agent / plan_id: PLAN-816 / plan_revision: 2 / outcome: needs_fix / worktree_path: D:/autostack/.wt/lang-816/auto-lang / branch: codex/plan-816-dev / reviewed_commit: 6b4bf9c2fcf3e872b33f6cc8505cf9fa09aa550f / base_commit: b65d0e963bbf39f97de5c1d6531997f6f21c760b / dependency_revisions: M815-A=e609caf1af511b96832f511ab9d1fa6c0c11f563（祖先与三 crate 源零漂移）；无814 WIP；锁/Spec/源码/技能hash见[inputs](attachments/816/review-r1/inputs.json) / acceptance_results: AC-07 partial（真实工具/来源/闭包通过，deadline/树/reader/流处理不通过），AC-08 fail；B原AC按门pending / findings: F-816-R1-01(P1) stdin/协议阻塞写不受deadline监督，250ms配置实等约1800ms后断管；02(P1) 根先退出、后代持管道使run/shutdown越过250ms期限等约1800ms仍报Success；03(P2) 一次写入合法回答+坏前缀误报Timeout，分次写正确StreamViolation；04(P1) Windows spawn后才assign Job，不能保证早派生后代入树（源码证明，未伪称时序实测）；05(P3) 新client/fixture格式未过，execution公开文档3条broken-link警告；06(P2) reader/read/join/cancel错误被吞，成功/已收尸声明不可靠（错误分支检查） / evidence: [完整R1/AC与SD映射/修复要求](attachments/816/review-r1/review.md)、[可回放反例](attachments/816/review-r1/replay.ps1)、[原始输出](attachments/816/review-r1/probe-output.txt)、[冻结Spec候选](attachments/816/review-r1/spec-delta-frozen.md)；29个不同现有定向测试通过（包19+真实fixture10），fresh43包/locked、闭包、clippy新包、CLI/Man check、777 strict 78边/0违规/2grandfathered通过；反例5项失败exit1，含正常/小请求/分次写对照 / pending_closeout_gates: T-13 pass后T-14冻结裸cargo t及新包/fixture/闭包、实际合入/Spec/受管ledger/祖先与来源收据仍pending；当前diff无VM/trans/UI生成/AAVM触面，最终并集复核；未整档 / next: 原树work一次修齐六项、永久定向控制+共享反例回放、提交新SHA再审；T-02/T-12重开，T-01完成保留，executing1/14，T-13/T-14未完成；SD-05-A处置与候选对齐，不将B发行提前或取消；未修实现/合入/归档/清树。

2026-10-20（续） / stage: work(T-02+T-12 R1 修复轮) / revision: 2 / outcome: R1 六项 findings 全部修复并回放验证（F-816-R1-01..06；非阶段 pass——待 T-13 复审）/ worktree_path: D:/autostack/.wt/lang-816/auto-lang / branch: codex/plan-816-dev / repair_commit: 4372d56d9（基线提交 6b4bf9c2f 之上；+1169/-227，7 文件） / dependency_revisions: 不变（M815-A=e609caf1af 零漂移；无 814） / task_ids: T-02, T-12 / evidence: ①审方 replay.ps1 重放 5/5 失败反例全过（F01a run stdin 251ms Timeout、F01b client send 254ms、F02a run 持管道后代 167ms Success、F02b shutdown 127ms、F03 缓冲坏前缀 0ms TooLarge；对照 C01/C02/C03 保持；REVIEW_PROBE_FAILED_CONTROLS=0）；②永久回归新增——设施 4 集测（deadline_covers_blocked_stdin_write/root_exit_terminates_pipe_holding_descendants/early_spawned_descendant_belongs_to_owned_tree/stdin_writer_failure_is_named）+4 注入单测（ReaderIo/ReaderPanicked/PipesHeld/干净 EOF 对照），client 5 注入单测（缓冲坏前缀不读新字节/粘包 drain/reader Io/半帧 EOF/非类型化信封），fixture 2 真实进程测（request_deadline_covers_blocked_send/shutdown_bounds_pipe_holding_descendants）；③三面计数：execution 6+17、client 9、fixture 12/12（3.7s）；④clippy 两 crate 0 警告、rustdoc 0 告警、fmt --check 过、fixture rustfmt 过；⑤verify-deps PASS、777 strict pass（78 边/0 违规/2 grandfathered）、cargo check -p auto -p auto-man exit 0 / findings_closure: F01=writer 线程化+绝对 deadline（run reader 先行；client 发送+等待同截止；超时先终止 owned 树解除写阻塞）；F02=根退出后 owned 树先终止（win_job terminate/killpg，Unix ESRCH 容忍）再 reader 有界收尾， PipesHeld 具名，client abort 先 cancel 后结束 writer；F03=reader_session drain 循环（Complete 后继续解析缓冲，仅 Incomplete 才 read）；F04=CREATE_SUSPENDED+挂起中 Assign+Toolhelp 快照恢复，失败路径对挂起根终止收尸（不可能已有后代）；F05=fmt+三条文档链接修正（ProcessTree/argv 转义）；F06=ReaderIo/ReaderPanicked/PipesHeld/Panicked/WriterFailed 具名传播，Timeout 仅在 cancel 成功+收尾完成后返回，预算拒绝（pre-IO TooLarge）不再误杀连接 / spec_delta: 候选 §5 增 SD-05-A 处置（显式进程消费补文，A 无安装变化）；SD-01-A 各断言现有代码真实支撑 / blockers: 无——T-13 绑定新冻结 SHA 4372d56d9 复审 / next_responsible_role: review（T-13 R2：六 findings 闭合核验+永久回归审查+SD-05-A 处置核验）

2026-10-21 / stage: review / review_scope: phase（M816-A/T-13 R2，非整Plan final） / reviewer_identity: 本聊天Codex /root，独立于实施，从原例与生产路径重建结论；模型/会话ID未暴露，未创建agent / plan_id: PLAN-816 / plan_revision: 2 / outcome: needs_fix / worktree_path: D:/autostack/.wt/lang-816/auto-lang / branch: codex/plan-816-dev / reviewed_commit: 4372d56d90670210e90a5a7a97e672cec1867a3c / base_commit: b65d0e963bbf39f97de5c1d6531997f6f21c760b（repair base=6b4bf9c2f） / dependency_revisions: M815-A=e609caf1af511b96832f511ab9d1fa6c0c11f563（三crate源零漂移），auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，无814 WIP / spec_inputs: [90项来源/配置/锁/Spec/技能hash](attachments/816/review-r2/inputs.json)、[本轮冻结SD候选](attachments/816/review-r2/spec-delta-frozen.md) / acceptance_results: AC-07 partial、AC-08 fail；SD05-A处置对齐，SD01-A真退出/错误传播断言仍不成立；B原AC/依赖门保持 / findings: 六项中F-R1-01/02/04独立CLOSED，F-R1-03/05/06仍OPEN，无新增ID；03(P2) shutdown将reader已报的坏帧Violation丢弃、实际Success；05(P3) fixture新增测试:395 unused_mut；06(P2) 超时拆除recv timeout/read错/panic仍被吞、stderr仍合并EOF/error，250ms配置实等6268ms仍Ok(Timeout elapsed250ms)，返回后own reader线程4→6，释放测试后代管道才回4 / evidence: [完整R2闭合表/覆盖断层/修复范围](attachments/816/review-r2/review.md)、[原五反例5/5通过](attachments/816/review-r2/r1-replay-output.txt)、[相邻组合两失败+两正控](attachments/816/review-r2/probe-output.txt)；44个不同既有定向测试通过（execution6+17/client9/真实fixture12），新包clippy/fmt/fixture rustfmt/rustdoc/闭包/777 strict均通过但fixture有上述新增warning；R1锁/上游/配置/旧CLI-Man源未变，复用适用旧check证据，未整档 / pending_closeout_gates: T13 pass后才运行T14冻结裸t/实际触面/新包与fixture门及合入/Spec/受管ledger/发布来源收据；当前无VM/trans/UI生成/AAVM触面，最终并集复核 / next: 原树work先诊断helper→生产收尾路径的错误丢弃/退出观察断层，一次修齐03/05/06并补永久控制、提交新SHA再受影响复审；01/02/04保持关闭；T02/T12重开，T01保留，executing1/14，T13/T14未完成；未改实现/合入/归档/清树。

2026-10-21（续） / stage: work(T-02+T-12 R2 修复轮) / revision: 2 / outcome: R2 三残项（F-816-R1-03/05/06）全部修复并以 R1+R2 双 replay 验证（非阶段 pass——待 T-13 R3 复审；01/02/04 保持 CLOSED 未动语义）/ worktree_path: D:/autostack/.wt/lang-816/auto-lang / branch: codex/plan-816-dev / repair_commit: aa2edc8fb（@4372d56d9 之上；+1166/−263，5 文件）/ dependency_revisions: 不变（M815-A=e609caf1af 零漂移；无 814） / task_ids: T-02, T-12 / evidence: ①R2 replay 全过（close-bad=Err(StreamViolation(TooLarge{observed:4294967295}))、close-clean 对照 Success、timeout-held-readers=Timeout@251ms 且 threads 4→4 零悬留、非拥有后代 heartbeat 继续推进、C04 双向 72ms 全字节）；R1 replay 5/5 保持；②三面计数：设施 8 单测+17 集测、client 10 单测、fixture 15/15（3.5s，0 告警）；③clippy 0/rustdoc 0/fmt+fixture rustfmt 过；verify-deps PASS、777 strict pass（78 边/0 违规/2 grandfathered）、cargo check -p auto -p auto-man exit 0 / findings_closure: F03=reader TruncatedEof 变体（自然 EOF 夹半帧→关闭路径 StreamViolation(Malformed mid-frame)；我方 kill 截断在途回复→超时/取消路径噪声）+drain_until_done(natural_exit) 保留 TooLarge/Malformed 类缓冲坏帧；F06=Windows PeekNamedPipe 就绪轮询（同步管道句柄对 WaitForSingleObject 恒 signaled，实测三测试挂死后弃用）+stop/owned_closed 双标志+有界 join 仅在完成消息后+writer 16KiB 分片写+client stderr 结束方式具名 Result（读错→Io{stderr reader}）+Timeout 仅在 cancel 成功+三线程确认真停后返回+elapsed/after 取实际返回时刻+超时路径 reader Io 具名传播、writer BrokenPipe/WriteZero/NotConnected 拆除噪声不改 Timeout 分类（R1 F01a 合同保持）；F05=fixture unused_mut 清零 / blockers: 无——T-13 R3 绑定 aa2edc8fb 复审 / next_responsible_role: review（T-13 R3：三残项闭合核验+01/02/04 未回归确认+新增永久回归审查）

2026-10-21（续） / stage: review / review_scope: phase（M816-A/T-13 R3，非整 Plan final） / reviewer_identity: 本聊天会话，与 R2 修复实施同会话（用户指定）——不能声称独立上下文；结论从工件重建（冻结 diff 精读/R1+R2 replay 重放/18+40 次定向/门禁重跑），未采信实施侧摘要 / plan_id: PLAN-816 / plan_revision: 2 / outcome: **needs_fix** / worktree_path: D:/autostack/.wt/lang-816/auto-lang / branch: codex/plan-816-dev / reviewed_commit: aa2edc8fbc3889f72fcf3dcb4b82887dd28d3bd9 / base_commit: 4372d56d9（R2 修复 diff 5 文件 +1166/−263）/ dependency_revisions: M815-A=e609caf1af 零漂移保持；无 814 / acceptance_results: 指派范围核验——01/02/04 无回归（R1 replay 251/262/190/204ms+集成多轮）、03/05/06 主面闭合（R2 replay close-bad=StreamViolation(TooLarge)/threads 4→4/251ms、fixture 0 告警）、新永久回归质量合格；AC-07 partial 保持、AC-08 fail / findings: F-816-R3-01(P2) stop 先停后排空——client stderr 循环以 sstop 为第一检查、reader_run 同构，轮询片醒来即退，OS 管道已到达未读字节被弃：stderr 日志缺字（高负载全套批次 consumer_describes/shutdown_bounds 各红一次，其后 18+40 次绿=负载相关窗口≈50ms 切片）+ stdout 已到达未读完整帧/违规帧可被丢（close-bad 可间歇回退为报 Success，击穿 R2-03 保持合同，现过=时序侥幸）；F-816-R3-02(P3) 早期错误返回路径（run 的 wait/terminate-after-root-exit 失败四处、client shutdown 的 terminate/wait 失败、drain_until_done 的 writer/stderr 错误分支）不停自有线程，违反"任何返回路径不带活线程"（源码证明，未伪称实机复现，同 R2-F04 标准） / evidence: [R3 报告](attachments/816/review-r3/review.md)；门禁重跑@aa2edc8fb：crates 10+8+17、fixture 15/15 多轮、clippy 0/rustdoc 0/fmt 过、verify-deps PASS、777 strict pass（78 边/0 违规）、check -p auto -p auto-man exit 0 / pending_closeout_gates: 不变（T-13 pass 后 T-14 冻结裸 cargo t 等；未整档） / next: 原树 work 修齐两项（stop=先排空再退出+确定性注入回归；错误路径统一收尾），提交新 SHA 后 T-13 R4 受影响范围复审（未变代码/依赖可复用既有证据，R3-01/02 相关面必重验）

2026-10-21（续） / stage: work(T-02+T-12 R3 修复轮) / revision: 2 / outcome: R3 两项 findings 全部修复（F-816-R3-01/02；非阶段 pass——待 T-13 R4 复审）/ worktree_path: D:/autostack/.wt/lang-816/auto-lang / branch: codex/plan-816-dev / repair_commit: 6e03f5f26（@aa2edc8fb 之上；+404/−143，3 文件）/ dependency_revisions: 不变（M815-A=e609caf1af 零漂移；无 814） / task_ids: T-02, T-12 / evidence: ①确定性注入单测×3（stop_drains_readable_bytes_before_stopping / stop_drains_preset_frames_before_stopping / stderr_stop_drains_preset_bytes——预置数据+先行置 stop，数据不得丢，无 sleep/无负载依赖）；②三面：设施 9+17、client 12、fixture 15/15（全套连跑 15 次 0 红——R3 实机间歇场景清零）；③R1 replay 5/5+R2 replay 全 contract_pass（close-bad=StreamViolation 保持、threads 4→4、251ms）；④门禁：clippy/rustdoc 0、fmt 过、verify-deps PASS、777 strict pass（78 边/0 违规）、check -p auto -p auto-man exit 0 / findings_closure: F-R3-01=stop 语义改"先排空已可读内容（drain 至 poll 假，DRAIN_CAP 500ms 防御）再退出"——设施 CaptureWorker 排空入捕获、client parse_drain 提取为可复用函数（stop 路径=排空 OS 管道→重解析→退出，已到达帧/违规不丢）、stderr_session 提取（stop 就地排空进 sink，日志不缺字）；F-R3-02=设施 finish_early（覆盖 wait_deadline Err/超时分支收尸 Err/无 deadline wait Err/terminate-after-root-exit Err 四处）+三 finish 顺序化（任一错误返回时三线程均已终结）、client stop_all+shutdown 错误分支先停后传播+drain_until_done 改全收集后传播（坏帧记下不中断 drain）；顺带修两处测试时序缺陷（shutdown_bounds 的 heartbeat 观察先于 shutdown——否则树终止先于后代首跳则不确定；consumer_describes 的 stderr 断言移至 shutdown drain 完成后——stdout 回帧与 stderr 读取异步，即时快照必输竞争）——两处均为测试侧缺陷、非产品回归 / blockers: 无——T-13 R4 绑定 6e03f5f26 复审受影响面（R3-01/02 相关面必验；未变面可复用 R3 证据） / next_responsible_role: review（T-13 R4）

2026-10-21（续） / stage: review / review_scope: phase（M816-A/T-13 R4，非整 Plan final；受影响面复审） / reviewer_identity: 本聊天会话，与 R3 审查/修复同会话（用户指定）——不能声称独立上下文；结论从工件重建（R3 diff 逐行精读/三面+8 次压力重跑/双 replay/全门禁），机器证据独立可复现，T-14 冻结门提供合入前又一重核验 / plan_id: PLAN-816 / plan_revision: 2 / outcome: **pass（M816-A 阶段）** / worktree_path: D:/autostack/.wt/lang-816/auto-lang / branch: codex/plan-816-dev / reviewed_commit: 6e03f5f268ee0eb4a3c2b9f64d4924c226977b72 / base_commit: aa2edc8fb（R3 修复 diff 3 文件 +404/−143）/ dependency_revisions: M815-A=e609caf1af 零漂移保持；无 814 / acceptance_results: AC-08 pass；F-816-R3-01 CLOSED（stop=先排空：设施 drain_readable、client parse_drain 重解析、stderr_session 就地排空；R3 实机间歇场景 15+8 次全套 0 红）；F-816-R3-02 CLOSED（finish_early 四处+三 finish 顺序化；client stop_all+drain 全收集后传播）；三重点合格——①注入单测：client 两测先行置 stop=确定性最坏情形、设施测断言确定（覆盖交错受 API 边界所限，同构覆盖，注记 N-1 非阻塞）②drain 全收集结构正确（注记 N-2 非阻塞：stdout_err=PipesHeld/Panicked 时 broken 误标 StreamViolated，仅拒绝文案、返回错误准确，随下次触码清理）③两处测试时序修正正当（heartbeat 观察必须先于树终止；stderr 断言落在 drain 保证态上）；01/02/04 无回归+03/05/06 主面闭合（R1/R2 replay 计数符合）/ findings: 无阻塞项；N-1/N-2 非阻塞质量注记，不授权转债延期 / evidence: [R4 报告](attachments/816/review-r4/review.md)；门禁重跑@6e03f5f26 全绿（三面 12+9+17+15、压力 8/8、clippy/rustdoc 0、fmt 过、verify-deps PASS、777 strict pass、check exit 0）/ pending_closeout_gates: T-14 merge-owned——冻结 6e03f5f26 上裸 cargo t、diff 触面复核、fixture/闭包复跑收据、master 实际合入（用户授权 Git 合入）、Spec 子集（SD-01-A+SD-05-A）受管沉淀与 ledger 增量、祖先/来源终验；不归档不清树 / next: T-14（merge 阶段执行）；B 面任务按既有门（T-03 等 M814-A+M816-A 发布——后者经 T-14 落地后即开）

2026-10-21 / stage: review / review_scope: phase（M816-A/T-13 R5，非整Plan final） / reviewer_identity: 本聊天Codex /root，独立于最近实施与R3/R4同会话审查，模型/会话ID未暴露；未创建agent / plan_id: PLAN-816 / plan_revision: 2 / outcome: needs_fix（替代同提交R4阶段pass作为当前结论） / worktree_path: D:/autostack/.wt/lang-816/auto-lang / branch: codex/plan-816-dev / reviewed_commit: 6e03f5f268ee0eb4a3c2b9f64d4924c226977b72 / base_commit: b65d0e963bbf39f97de5c1d6531997f6f21c760b（repair base=aa2edc8fbc3889f72fcf3dcb4b82887dd28d3bd9） / dependency_revisions: M815-A=e609caf1af511b96832f511ab9d1fa6c0c11f563（三crate零漂移）、auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，无814 WIP / spec_inputs: [92项来源/锁/配置/Spec/技能指纹](attachments/816/review-r5/inputs.json)、[冻结SD候选](attachments/816/review-r5/spec-delta-frozen.md) / acceptance_results: AC07 partial、AC08 fail；B门保持、SD05-A对齐，SD01-A真停/全路径读错断言仍不成立 / findings: F-816-R3-02重开(P2，实机证据补强)——run/client在DirectChildOnly+非owned后代持stdin读端时，250ms配置约3254/3272ms返回PipesHeld，own writer线程4→5且仅释放测试后代后回4；finish_early忽略join失败，client早退仅stop不确认；F-816-R5-01新增(P2，R3新分支错误终态回归)——stop排空吞读错（三条流同构）、client半帧仅Done而无TruncatedEof；设施读错→Stopped、stderr读错→Ok、半帧证据消失三确定性注入全失败 / evidence: [完整R5/映射/诊断/修复范围](attachments/816/review-r5/review.md)、[真实writer两入口](attachments/816/review-r5/probe-output.txt)、[三注入原始结果](attachments/816/review-r5/injection-output.txt)、[共享回放](attachments/816/review-r5/replay.ps1)；53个不同现有测试全过（12+9+17+15），R1五原例及R2四控制全通过；fmt/rustfmt/clippy/rustdoc/闭包/777 strict通过，旧CLI-Man check按不变输入复用；真停不能被PipesHeld即过的断言替代 / pending_closeout_gates: T13 pass之后T14冻结裸t/实际触面/新包及fixture、合入/Spec/受管ledger/发布来源仍pending；当前无VM/trans/UI生成/AAVM触面，最终并集复核 / next: 原树work先诊断writer false-oracle与stop drain丢终态两处覆盖断层，一次修齐并补永久控制，提交新SHA再独立受影响复审；T02/T12/T13重开，T01保留，executing1/14；R4 pass历史保留，未改实现/整档/合入/归档/清树。

2026-10-21（续） / stage: work(T-02+T-12 R5 修复轮) / revision: 2 / outcome: R5 两项（F-816-R3-02 重开面/F-816-R5-01）修复并以 R1/R2/R5 三 replay 验证（非阶段 pass——待 T-13 R6 独立复审）/ worktree_path: D:/autostack/.wt/lang-816/auto-lang / branch: codex/plan-816-dev / repair_commit: a972286fd（@6e03f5f26 之上） / task_ids: T-02, T-12 / evidence: ①R5 replay 全过——writer 探针两入口 run/client=276/261ms（R5 实测 3254/3272ms→修复后 <300ms）、own_threads 4→4 零悬留、after_release=4、非拥有持有者 heartbeat 存活；三个确定性注入（stop 读错设施/client-stderr/半帧 TruncatedEof）全过；R1 replay 5/5+R2 replay 4 contract_pass 保持；②三面：设施 9 单测+18 集测（新增 deadline_covers_blocked_stdin_with_unowned_holder：DirectChildOnly+stdin-holder 持读者+1MiB+250ms→Timeout<1.5s）、client 15 单测（新增 stop_drain_read_error_is_named_io 直驱生产 drain_readable）、fixture 16/16（新增 abort_stops_blocked_writer_for_real）；全套压力 8/8；③clippy/fmt/rustdoc 0、verify-deps PASS、777 strict pass、check -p auto -p auto-man exit 0 / findings_closure: F-R3-02(writer)=①平台事实修正：CancelSynchronousIo 对管道同步写 ERROR_NOT_FOUND（探针）；CancelIoEx（按文件对象）实测~10ms 解除阻塞写——设施 win_stdin.rs 提供 CancelHandle（DuplicateHandle 副本+Clone），WriterWorker 共享槽：写线程正常完成关副本（EOF 不延迟——探针证实副本存活宿主收不到 EOF；另修复 client 内一份 shadow shutdown 实现未关副本导致 EOF 丢失的回归）、stop_and_join cancel-then-close（实测语义成立）；②错误分支：设施 bounded_cancel（两次终止+有界收尸）替代超时臂内无界 child.wait；finish_early PipesHeld 优先传播保持；client stop_all+cancel+drain 全收集后传播保持。F-R5-01(drain)=设施 drain_readable→DrainEnd{Empty,Eof,Io}（Io 具名、Eof 真实终态）；client reader stop 分支：排空后按"EOF 或 owned_closed"判流终——半帧保留 TruncatedEof 证据（适配审方 Cursor 型就绪源语义），读错 Violation(Io)；stderr_session 排空读错具名 Err / blockers: 无——T-13 R6 独立复审绑定 a972286fd / next_responsible_role: review（T-13 R6：两项闭合核验+01/02/04 无回归+53 旧测与三 replay 保持）

2026-10-21 / stage: review / review_scope: phase（M816-A/T13 R6，非整Plan final） / reviewer_identity: 本聊天Codex /root，独立于实施会话，模型/会话ID未暴露；未创建agent / plan_id: PLAN-816 / plan_revision: 2 / outcome: needs_fix / worktree_path: D:/autostack/.wt/lang-816/auto-lang / branch: codex/plan-816-dev / reviewed_commit: a972286fd5f0aa05338ef4ca0c09c54a56de0027 / base_commit: b65d0e963bbf39f97de5c1d6531997f6f21c760b（repair base=6e03f5f268ee0eb4a3c2b9f64d4924c226977b72） / dependency_revisions: M815-A=e609caf1af511b96832f511ab9d1fa6c0c11f563三crate零漂移、auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，无814 WIP / spec_inputs: [92项源码/锁/平台/Spec/技能指纹](attachments/816/review-r6/inputs.json)、[冻结SD候选](attachments/816/review-r6/spec-delta-frozen.md) / acceptance_results: AC07 partial、AC08 fail；F-R5-01独立CLOSED（三原注入全过）；F-R3-02仍OPEN(P2)：已挂起写取消主面修复成立，但Worker读stop=false后、WriteFile前暂停，控制侧取消并关凭证后再放行真实写，生产3s宽限后PipesHeld且writer_still_in_write=true；shutdown早退与proc Drop无界wait的错误出口也未齐；新增F-R6-01(P1)：cfg(windows)模块win_stdin被无cfg签名:407/:859引用，真实Linux target check两E0433、exit101，上一6e03原源码同目标check exit0；client另有无cfg win_stdin_reexport字段须同步隔离 / evidence: [完整R6/闭合表/修复范围](attachments/816/review-r6/review.md)、[确定性取消竞态](attachments/816/review-r6/cancel-race-output.txt)、[Linux新旧对照](attachments/816/review-r6/platform-replay-output.txt)与各自回放脚本；58个不同Windows定向测试全过（client15+execution9/18+fixture16），R1五原例/R2四控制/R5 writer两入口及三注入全通过；新包Windows健康/闭包/777 strict通过，旧CLI-Man按不变输入复用；未整档 / pending_closeout_gates: T13 pass后才T14冻结裸t/实际触面/新包fixture及闭包、master实际合入/Spec/受管ledger/发布来源；当前无VM/trans/UI生成/AAVM触面，最终并集复核 / next: 原树work一次修齐取消前后顺序/返回析构清理及平台cfg回归，保留本轮共享栅栏与Linux编译控制，提交新SHA再独立受影响复审；T02/T12重开、T13未完成，T01保持，executing1/14；F-R5-01保持关闭，不批准降低真停/跨平台支持合同；未修实现/合入/归档/清树。

2026-10-21（续） / stage: work(T-02+T-12 R6 修复轮) / revision: 2 / outcome: R6 两项（F-816-R3-02 取消时序残项/F-816-R6-01 Linux 编译回归）修复并以 R6 两 replay+R1/R2/R5 重放验证（非阶段 pass——待 T-13 R7 独立复审）/ worktree_path: D:/autostack/.wt/lang-816/auto-lang / branch: codex/plan-816-dev / repair_commit: 05d3940c6（@a972286fd 之上；2 文件 +100/−77） / task_ids: T-02, T-12 / evidence: ①R6 cancel-race replay 过——栅栏注入（Worker 读 stop=false 后、真实写前暂停;取消槽消费关闭后放行）outcome=Ok(Stopped{written:0})、writer_still_in_write=false（R6 实测 3s PipesHeld+writer 活）;②R6 platform replay exit0——Linux 目标真实两包+独立 consumer check 全过（R6 实测 exit101）;③R1 5/R2 4/R5 R5_WRITER_FAILED_CONTROLS=0 保持;④三面 9+18/15/16、fixture 压力 6/6;⑤clippy/rustdoc/fmt 0、verify-deps PASS、777 strict pass、check -p auto -p auto-man exit 0 / findings_closure: F-R3-02(三序)=①Windows StdinPipe::write_stoppable 入口拒写（stop 先到→Ok(0),写永不入核——迟到写在凭证已关后仍被拒）;②写已挂起→停止侧 take→cancel→drop（cancel-then-close 实测中止）;③写已完成→正常字节;stop_and_join 改 is_finished 轮询+join 收割（不再消费 done 通知;线程经 JoinHandle<WriterOutcome> 返回;wait() 同步改造）;残项=client shutdown wait_writer_done 早退与 wait_deadline Err 均改走 cleanup_then_error（cancel+drain 后传播）、OwnedProcess::drop 有界收尸（terminate 失败不再无界 child.wait）。F-R6-01=平台类型隔离——设施 CancelSlot（windows=Arc<Mutex<Option<CancelHandle>>>/unix=Arc<Mutex<Option<CancelCredentialStub>>>）与 CancelCredential（windows=CancelHandle/unix=无操作哨兵）中立别名,spawn_with 统一签名,cancel_handle 平台分支,client writer_cancel 改中立类型,移除 win_stdin_reexport / blockers: 无——T-13 R7 独立复审绑定 05d3940c6 / next_responsible_role: review（T-13 R7：三序闭合核验+平台隔离 Linux check+清理编排+01/02/04/05 无回归与旧 replay 保持）

## 10. 待澄清事项

无需用户重复授权；T-01具名冻结兼容/管理服务包布局、现有facility复用、交互与bootstrap。若必须改变旧命令/支持、默认安装增加不可接受人工步骤、破坏public API或跨仓写入，具体needs_replan。不能凭fake工具/只测CLI退出码宣布进程和全链交付完成。
