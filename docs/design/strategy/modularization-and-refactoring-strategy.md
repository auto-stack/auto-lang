# 模块拆分与重构：当前战略及接续边界

> 更新：2026-10-29，新增[888模块选测与ACE成组执行草案](../../plans/888-module-test-selection-and-ace-batch-pilot.md)。当前调度唯一入口为[执行路线v17](../../plans/reports/v06-extraction-execution-roadmap.md)，长期边界仍见Design37/50。本轮仅规划，不实施/建仓/建树，不改887/885/ACE合同或产品，不联系他方；现行行为以Spec和实际交付为准。

## 1. 目标与已经完成的范围

目标是若干职责完整、依赖清楚、可独立构建/测试/发行的大模块，能够在自己的仓库持续演进。先证明契约、单一类型身份、真实消费和资源闭包，再搬仓；移走相同行数不算生态净删除，也不能以建立仓库代替产品验收。

M1 已由 831/832 归档交付：沿用既有 auto-ui、auto-man 仓身份与历史，legacy refs/恢复载荷保留，新实现以普通提交承接默认分支，不另建替代仓、不清空/orphan/强推。auto-lang 保语言 SDK/中立模型协议，auto-ui 保 UI 领域编译与运行，auto-man 保管理核心、adapter 与薄 CLI。三仓主分支目前为语言/UI/Man 的 `master`；auto-os 为 `main`。更名、远端 push、registry 发布不是已获默认授权的收尾。

M2 已由 841–844 归档交付：UI owned AST/Token/lowering 终态、旧兼容与发行路径退役、语言内部结构、跨仓去重与净瘦身。844 收据生产净删 1,221 行；这是该批有限交付，不代表全部潜在重复清零。867 已归档，完整交付收据确认测试等价出口成立；其冻结组合的执行通过仍保留 th、ui-tn、Windows aavm 等限制，不外推为所有环境和应用全绿。

允许的方向为：产品/CLI 装配→公开 SDK/工具；UI→语言基础 SDK/中立协议；Man 管理核心不拉完整 VM/renderer；语言基础不反向依赖 UI/Man/桌面产品。共享类型/算法只有一个 owner，不引入塞满跨界类型的 common 巨包。数据域及 runtime 域可先保持独立 crate；另建仓须真实收益论证和单独 go/no-go，不是 M1/M2 未完成的证据。

## 2. 当前计划与出口

以下数字是读取各 Plan 状态头得到的交接时快照，新会话先读回更新，不以此表覆盖后续进展。

| 计划 | 快照 | 交付与启动门 |
|---|---|---|
| [867 测试等价性](../../plans/archive/867-split-equivalence-test-closure.md) | r2，archived，13/13 | U867 已成立；合入/ledger v117/六树 guard 清理收据齐全，允许后继核门 |
| [869 crate 清单与减法批二](../../plans/archive/869-crate-inventory-and-subtraction-batch-2.md) | r4，archived，10/10 | 47 crate清单、生产净删2,799行及三仓合入/ledger v122/guard清理闭环 |
| [872 内建语义辨析](../../plans/archive/872-builtin-semantics-dual-track-ruling.md) | r3，archived，10/10 | 六族DISTINGUISH零行为变更，合入/ledger v124/guard清理闭环 |
| [873 构建成本与 member 决策](../../plans/archive/873-build-cost-and-member-split-decision.md) | r3，archived，8/8 | U873成立，暂不细拆crate；测量/模型/限制、合入54f4ae6e9/ledger v127及四树guard清理闭环 |
| [876 桌面入口归位与健康面](../../plans/archive/876-autos-desktop-entry-and-launch-health.md) | r3，archived，completion_kind: delivered | 三仓合入、规范/账本及四树 guard 清理已有收据；头部 current_step 仍为 12/13，不据此重开历史计划 |

U(N) 表示约定 AC、独立 final review、merge-owned 冻结门、实际修改仓合入、canonical/受管 ledger 与来源读回、归档和 guard 收尾全部闭环。work pass、list-only、阶段合入、SD 草案不等于 U。869/872/873 的门覆盖盘点、探针、测试和建树；门前只核前置收据和已授权规划。

主排程：已交付867/869/872及873 → **884收口→887测试治理→auto-web有限迁移→UI/Man结构包**。Web与UI/Man仍未编号，按完整模块交付；873不细拆crate、A1另签核及测量适用限制保持。

测试模块化是拆分质量基础设施，建议887完整交付后优先启动888有限支线，**不等Web/UI-Man全部结束**。模块映射服务完整测试义务，批处理首落ACE；888允许交接后新增最小test-only worker入口，原全局门/原子义务不取消。Web主线不设无关U888全局等待，设计可继续；共享脚本/注册短窗串行，后继接受正式交付工具。该建议不代表本轮实施授权或给887追加验收。

当前读取：[884](../../plans/archive/884-language-structure-closure-without-crate-split.md)已归档10/10，U884成立；[887 r2](../../plans/887-test-governance-and-ace-first-corpus.md)reviewed9/10，独立终审pass，U887仍待冻结门/合入/归档清理。887性能按用户批准的机制级负结果关闭，不等于已加速。888建议U887后优先；Web仍接受U887与885相关正式交付/写面释放，UI/Man排U(Web)后。各路线只按真实写面交接，不认领他方或等无关ACE全线。

Web现阶段编写边界/验收/交接设计，**887结束后再取号冻结正式迁移Plan**。这不是建仓或提前实施授权；迁移包自己扩展Web测试owner与四仓组合，不给887追加入库/运行义务。885、BLOG-004和其他应用继续原合同，不加Web硬前置；当前885无独立阶段交付时相关迁移锚按U885，真正无关写面不设全局等待。

### AutoUI 架构重设计的切换点（2026-10-28 用户决定）

先达到近期有限收口，再重审AutoUI新架构。原三仓有限判据仍为既有交付及U884/UI-Man结构完整关闭，876已交付不再等待；本轮用户选择在884与UI/Man之间安排887和Web，因此当前总体排程会先完成这两包。新增Web独立交付单列，不重开三仓历史M1/M2，也不把Web扩展和全部测试改写无限追加到出口。

现有[Design47](../47-autoui-state-design-and-feature-architecture.md)、[Design38](../38-autoui-features-and-system-services.md)及宿主/Editing草案保留候选，不提前实施新状态/Feature/设计体系/HMR，也不并入结构迁移。884继续原执行；UI/Man未编号，当前排在U(Web)后再正式立约，不自动创建或认领。

里程碑仍以有限合同的完整验收/独立终审/冻结门/合入/规范账本/归档guard为准。当前排程增加Web工作量，时间可能后移；不要求80万达标、历史债全清、数据/runtime另建仓或ACE/AutoGraphics全线。到点固定实际三仓及新增Web组合，读回剩余限制再重审架构草案；业务应用与必要修复继续，不等待所有治理结束。

## 3. 已裁定的验收口径

- **867 分母不缩减**：保留拆分前全部测试义务、来源/归属/实际执行状态；Man `t/tc/tcore/ti`、UI `tu/ti/tn` 及语言适用档/编译阻断等按 r2 完整登记。r1 的 8,315 枚举子集、list-only、将 th 编译红排除在义务外均不能证明全集运行。未解释缺口=0；新增红=0；既存红逐 ID 认证，消失红解释；编译阻断、环境缺失、ignored、未执行分列。未知或约定必需正控未执行仍阻塞。
- **869 规模与减法**：A 为约定范围全部 `.rs`（P/T2/XT/G/VEND，XO 另列）的披露/达标口径，B 为三仓生产 NET-PROD 不反弹约束，C 旧单 crate 仅历史对照；三仓同快照、同脚本/分类版本复算。测试、生成、第三方分列，迁移不算净删。历史约 100 万不能冒称当前实测；80 万是争取目标，不授权删功能/测试凑数或无限追加计划。具体完整口径仍看 869 §5.1。
- **869 批粒度**：每批 NET-PROD 净删≥2,000 行，安全候选不足则合批/重规划；结构性减法仍需替代路径、承诺迁移、门禁及用户按批签核。收据含前后值、删除 SHA、恢复配方/演练、推送状态；不强制 push。
- **872 保语义**：str.len、math、to_int/parse_int、time_now、HTTP error-reply、nv_to_value 六族全部 DISTINGUISH。先冻结名字→注册/覆盖→生效实现→语义→消费者→测试绑定表；旧名、优先级、错误和结果保持。MERGE 未授权，RETIRE 需无消费者证据与另行签核；plan755 HTTP body 不变。
- **873 先测后决定**：同版本分层测冷/热 check、build、测试二进制/链接、枚举与真实启动/初始化，至少七个真实源码编辑站点（语言≥4、UI≥2、Man≥1）；touch 只作诊断控制。独立 target/cache，禁止清共享 sccache；至少三个有效样本，污染样本作废。拆/不拆/需试点结论均允许，估算不冒充实测收益；不为测量新跑整档。

## 4. 两个结构工作包：884已立项，UI/Man待后继

语言结构包：按当前基线/消费者对账→core/entry.rs运行桥接→stdlib装配→call_inner逐分支审计与职责提取→转译主管线收口的顺序。HTTP入口已壳化部分仅核残项，不重复拆；保持872注册覆盖/名字/ID及生成结果。U869/U872/U873已满足，仍须正式立约及真实共享面交接。禁止以873结论为由启动parser/vm/trans全面拆crate或新性能改造。

UI/Man结构包：当前排程保守等U(Web)，接受884 SDK、887清单和Web公开服务出口，默认仍Man CLI→Vue生成→Aura构建→renderer消息/资源状态机。复用cmd_*、import_policy/shared，重核服务分派和HTTP客户端归属，不重复搬Web算法；CLI/生成/挂载身份/GPU交互及定向消费/视觉验收保持。历史行数立约时重核，不以拆更多文件计功。

873建议A1（根包默认ui-iced调整）另作签核候选，不并入当前结构/迁移包。细拆crate仅按873具名触发条件重评，mtime实验不动现网；测试治理按887义务守恒与实测范围推进，不以873重开分层或清共享缓存。869把U871泛化绑定A2R call_inner的旧前置已按实际写面纠正：871退役AC native HIR→CLIF，不自动退役Auto→Rust。详[当前范围](../../plans/reports/post-873-structural-work-packages-2026-10-29.md)。

**语言结构884、测试治理887已立约，Web和UI/Man均未编号，不叫874/875。** 两号已分配AutoGraphics。Web首期为服务框架/HTTP生成运行/测试发行，通用语言、Tauri/sidecar/shared workspace按真实消费者保单源，不整包搬auto-api-gen；语言基础不反依赖Web、Man核心不拉VM/renderer。原Design31浏览器/沙箱/远程仍远期。首期保已有配置、安全/预算、健康/关闭合同，不把它们误当后续扩展。细节见[Design50](../50-autoweb-service-framework-and-repo-boundaries.md)。ACE只按实际写面协调、不借WIP；既有独立终审/冻结门保持。

## 5. AutoOS 与应用可以如何独立继续

业务应用可以固定已交付三仓源码、锁、资源和工具版本，通过公开入口并行开发，不必等所有质量治理结束。共享 SDK/协议/renderer/native 修复由 owner 发布，消费方重放受影响范围；产品业务仓不复制私有模型/实现绕过接口。UI 已搬仓不表示所有应用、后端、平台、安装产物自动可用。

AutoOS [051 真实运行报告](../../../../auto-os/docs/plans/attachments/051/runtime-check.md)已经记录：新脚本真实构建三仓并启动 VM 桌面，桌面与独立 CLI 计算器均实测 2+3=5；OS main 交付 `3088dea`，报告 `234e5f6`。独立 VM 首次栈溢出在 UI 工具 owner 修复，auto-ui master `5a89bfd`，主栈 1 MiB→64 MiB，普通重建及真实交互复测通过。视频 playlist 解析错误、Music/Photos 扫描 404 仍 open，归 OS 债表；不称全应用通过。

876 已把 iced 桌面从语言 example 迁成 `auto-os/host/desktop` 的 `ui-desktop` bin，并交付启动健康面。合入收据为 lang `015884d45`、UI `f489c7b`、OS `1f9bd87`；后续主分支可再前移。只读核查确认 UI master 保留 `5a89bfd` 栈策略、OS main 保留 051 `3088dea`，现行两份 desktop 脚本 iced 轨均构建 `os-desktop --bin ui-desktop` 并从新产物路径启动。**051 报告里的旧 example 命令只作历史证据。** 本轮未重跑构建/GUI，不把代码及收据核对写成新运行通过。

AutoOS/桌面产品计划与债已按 876 在 auto-os 默认分支管理，auto-lang 只留指针；跨仓正文放主改面仓。OS876-APP-EXE/SUBMODULE 后续修复和守门也已有 OS main 收据；终端 Tick、use 解析及 Vue desktop-host 归位等仍按实际 owner 跟踪，不默认为 iced 迁移已覆盖，不追加为 869 的全局前置。

## 6. 验证与会话接续纪律

独立 scope 的 work/review 修复可以并行，同 Plan 同权威树单写者、冻结交接；共享 manifest、锁、注册、指纹/ledger 与默认分支接线排队。最终 review 前只做定向反例、模块测试、必要 check；final pass 后由 merge 在冻结提交统一裸 t 及实际触面档。tf 主检出到期单实例，873 机器测量另预约独占，不互相污染。

原始日志、探针输出、截图、gate/ledger 转储均本地保留；attachments 只允许提交 Markdown 摘要，含命令/退出码/数字/本地路径+hash。不为交接做证据历史过滤、force push、分支更名或缓存清理。主检出现有其他路线 WIP 必须保留。

新会话完整接续输入见[交接 prompt](../../plans/reports/extraction-refactoring-next-session-prompt-2026-10-28.md)。先读在途状态和实际交付，维护本路线与启动门；不要从 777 重新调研或自动执行后续治理包。
