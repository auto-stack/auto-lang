---
plan_id: PLAN-1015
status: execution_done
feature_name: 拆分重构收口补验与合同对齐
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12
plan_revision: 1
current_step: 6
total_steps: 9
depends_on: [PLAN-989, PLAN-1006, auto-os/PLAN-068]
supersedes_spec_components: [docs/specs/repository-boundaries/split-refactor-milestone-acceptance.md]
new_spec_components: []
touched_goals: []
affects: [generated-delivery, rq-recovery, repository-boundaries, milestone-readiness]
---

# [PLAN-1015] 拆分重构收口补验与合同对齐

## 0. 变更摘要

只关闭[收口状态复核](reports/split-refactor-closeout-status-audit-2026-11-12.md)的F-989-CLOSE-01/02，对应债务D-989-CLOSE-01/02：UI交付恢复的Counter/Todo业务及013包对缺证；RQ旧组合与原恢复合同不同。补真实交互、指定正式旧组→N的恢复证据，统一支持对象/来源/规范与路线结论。

用户2026-11-12明确“先建立正式计划，明确两项差额范围、依赖和验收，暂不实施”。本r1 drafting、0/9；本轮只中央取号和文档立约同步。未建实施树/材料、运行构建/GUI/补验/安装/恢复、清缓存、提交或打tag。1006与989保持archived，原交付/历史ready出处不改写、不倒签已测。

## 1. 目标

- G1：Counter和013 Todo各自有真实可导出的Rust交付形态，旧A/新B/恢复A均有可观察交互业务与资源证据，不再以窗口存活代业务。
- G2：按原RC-RQ-01的O-RQ/U-RQ/L-RQ/M-BASE组合补指定旧包→N的两Host/真实Todo/数据备份恢复及旁观隔离证据，旧合成组合证据保持其原范围。
- G3：两项原义务、实际包对/运行/负控与来源逐项对齐，独立复审后真实关闭债务并沉淀当前Spec/恢复清单/路线；给出有限本路线收口结论及当前联合tag门状态。

非目标：重做1003/1006/OS068全套、重开归档989、完整产品/性能矩阵、高刷/光子/原生输入延迟的新承诺、Video后端、Linux/ACE/AGE接管、通用安装器/测试平台。不是生产实现修复包：允许本计划专用测试驱动/夹具/收据与文档，不能顺手改编译器、渲染器、产品业务/存储或公共交付生产代码。若必需路径实测失败归生产缺口，回真实owner有界修复再消费，不换简单项目/静态种子使本包假绿。

## 2. 架构方案

原恢复合同+989冻结N/正式收据→T-01核对象/源/工具与有界交互可行性→UI指定对象A/B/R交互补验、RQ指定旧组合→N演练→逐finding正反控与读回→合同/Spec增量→独立review→正式落地/债务关闭/清理。

| 责任 | owner与界面 | 本包边界 |
|---|---|---|
| 总协调/测试 | auto-lang中央计划，有限测试路径/收据与路线判断 | 不把整个989验收重新搬来；只核两差额与实际新增耦合 |
| UI对象/入口 | auto-ui delivery owner提供002/013真实源、生成包/事件与资源；Man仅已有公共CLI | 隔离源副本的Rust入口适配；不改主检出pac、不删store/API/主题或重写业务 |
| RQ装配 | auto-os owner的公开App/两Host/工具启动；UI/语言提供协议与版本来源；auto-todo产品供持久数据 | 指定旧组对N，不偷换旧侧U-C/L-C；同schema备份/恢复，不扩数据迁移 |
| 规范/结论 | language milestone Spec、恢复清单、债务/路线当前声明 | 实施后才发布验证结论；不改历史签章，不自动创建tag |

硬依赖989、1006、OS068均已正式交付，T-01可在以后获启动时进入；不等待OS062、ACE/AGE M2或1014全局治理。PLAN-1013正处理ACE M1来源/签章准入，**不阻本包UI/RQ补验，却是联合tag消费ACE收据时必须核的当前领域门**；本包不能拿1000/1005旧总数自动签joint_m1_ready。1014负责固定回归治理，本包保留真实红/原条件，不在此修全局组或机械跑全仓。

## 3. 技术栈

已有Auto CLI、UI delivery、Iced/wgpu两Host、AutoUI MCP或原生窗口交互、Python标准库/SQLite读回及Git对象身份；只新增有限驱动/测试，不搭通用框架。

未来权威实施树拟`D:/autostack/.wt/lang-1015/auto-lang`，branch=`codex/plan-1015-dev`；确需测试代码落其他仓时只在同组各仓一树，分支`codex/plan-1015-<repo>`，work/review同树顺序，不另造review副本。旧/N源码用统一scope内只读正式源副本作测试材料；它们不是第二个实施checkout，禁止junction/symlink。构建必须显式源/输出/target/工具；不得借主检出WIP或旧PATH安装。

## 4. 需求分析与背景调查

### 授权与只读来源

本轮仅新计划/依赖/验收及路线文档，不执行，不指定代理/模型、不新建聊天或goal。以后实施需用户启动；实际tag/推送/部署仍未授权。已读[overview](../specs/overview.md)、[里程碑Spec](../specs/repository-boundaries/split-refactor-milestone-acceptance.md)、[生成交付回归](../specs/repository-boundaries/generated-delivery-regression.md)、[UI/RQ恢复合同](reports/split-refactor-recovery-objects.md)、[OS068 Spec](../../../auto-os/docs/specs/shell/windows-rq-complete-delivery.md)，再核989/1006最终收据、真实示例与930驱动。Spec的ready=true与其明确未执行业务的限制是本包要对齐的现知识差额，不据此免除原义务。

| 已观察来源 | 内容/身份 |
|---|---|
| milestone Spec SHA256 | `bd78c01845212d1f47325c8e445f6c265a56b53d8aac2c6568cc881ed829d56a`；§3明示UI业务未做、旧RQ混组；§4仍有历史两个ready=true |
| 恢复合同r1（修订前内容hash） | `c75c35f9bceb3448018e97953e215da922c207e829d0165df455d5ef5cb2b031`；RC-UI要求002/013各一对与恢复后业务，RC-RQ指定正式旧组合 |
| 930 driver | `test/fixtures/modularity/generated-ui-consumer/driver.py` SHA256 `90036a35461193794b48f3a2ed021c407ae71039d28a587bfd33cf8652176c2d`；V06a窗口存活、V06e VM协议/取消，均非Counter数值断言 |
| UI示例002 | `examples/ui/002-counter/pac.at` render:vue；`src/front/app.at`有count=0、+/-/Reset和官方主题，源SHA256 `04937c35325faa74ac790dee8ca97eef5cb34715badaeecc04eb1a3fe26d0d69` |
| UI示例013 | `examples/ui/013-todo/pac.at` render:vue、api:rust；多组件/真实TodoStore→back.api→back.db，后者是内存列表。store源SHA256 `80090867afe41d0954b507dd11a6cb8d212fb439112a2ef11a0d95c837bd55bf` |
| 现有UI交互参考 | `examples/ui/013-todo/tests/desktop_mcp.py`有autoui_snapshot/action/type/keyboard/state，但该旧harness跑VM，且有skip/输入路由局限；不能直接用其self-check替代生成Rust包 |
| OS/RQ与真实持久产品 | `scripts/rq-final-acceptance/{build-demos,observe,n-sweep,negative}.ps1`等存在；O-RQ树的`apps/auto-todo`gitlink=`0510e0501b3783bd68b5ff6f00b7a5f81fc1d0f0`。auto-todo现README/Spec说明AUTO_TODO_DATA、todos.db/bak与owned进程边界；实际旧/N支持以各冻结源为准 |
| 068 Spec SHA256 | `00646f3282a58d1f1c00f88415432b8ede18541d8d585acb43e018e925ec4a26`；公开两Host/业务/设备范围/修约保持 |
| 1013 / 1014 | 1013 drafting0/10，ACE分域facts来源准入修复、joint签前输入；1014本轮读到待填草案骨架，不能据标题虚构已交能力或设本包硬依赖 |

源码/Spec内容身份是观察记录，未运行新补验；T-01重核漂移并读真实help/端口/配置。不能从父exit0、已删scope路径或归档总数推断断言成立。

## 5. 详细设计

### 5.1 有限来源集合与N/N'规则

| 集合/仓 | 精确旧A | 新B基准N |
|---|---|---|
| UI语言 | L-C=`11e1f1fd5e79766266ce8d54356155f018b8d661` | `075f65ccb683d9c290e011200e68f3f404d015eb`（989冻结源；落地28803252…仅后续规范/簿记差额另核） |
| UI工具 | U-C=`393ea17ceaaea19c58da294ea292c19b273263b8` | `691047651590e716b8b77651ea68bbdc9773c119` |
| Man CLI | M-BASE=`347c29f7ddc6650a94548f4523b3bc8f0e7cae17` | `54fe4070bdbd06015ea4521074479ae2e2f8b14f` |
| RQ旧OS | O-RQ=`3ea7ead67d2da739a04c771209b1cc18a0a18210` | `868c80d3e697ad6eaa9cce27596430a9abe08d5a` |
| RQ旧UI | U-RQ=`691047651590e716b8b77651ea68bbdc9773c119` | 同N UI，不把相同子件冒跨版本覆盖 |
| RQ旧语言 | L-RQ=`f29f6facc6a31db429e0076ba35222171bf397dc` | 同N语言 |
| Web依赖源 | W-BASE=`2d698413b368896f913eba375ff788c795f395cd` | `8b1742a0f84c24173768265cb999a648039d1e2b` |
| Down/主题/产品 | Down=`58ea33f699efa5cb1194473a2d45dcec51006842`；主题取各UI正式树；Todo取各OS正式gitlink | N各正式树/产品pin逐项取回，不用产品当前HEAD或教学013替产品 |

RQ-A必须为O-RQ/U-RQ/L-RQ/M-BASE，UI-A仍用L-C/U-C/M-BASE，两者不能共用一个“old”别名覆写。T-01逐repo列source/exporter/CLI/tool/Host/backend/锁/配置/有效C策略/字体主题/资源/protocol/schema、实际源码文件清单和工件hash。组装源副本只能由可取回正式Git对象或已支持SDK生成，额外主题/资源显式闭包；禁止伪造Git HEAD、仅env标签自证身份或自动fallback到主检出。

计划实施代码基于当时干净master的唯一工作树，**测试逻辑所在提交与被测生产N是不同身份**。T-01对测试提交所用生产触面作diff/输入审计；不能分开冻结则形成N'并说明受影响证据/有限重验。N→N'不减少对象/断言；实质支持/预算改变须修订合同。N工件若已按政策删除，在本scope按原正式源重建，允许重建，不声称旧hash工件仍在。新运行必须记录实际工件hash，不用源等价替代运行进程绑定。

### 5.2 T-01有界可行性与UI对象适配

开工先核两个候选交互通道：（a）生成Rust包实际支持、端口能绑定本进程的既有MCP，通过真实用户事件后读状态/可见文本；（b）原生窗口操作与前后可见业务值/截图读回。择可用通道或组合，不为本包新做生产MCP/可访问性框架。只在生成交付包实跑，源项目VM、自建self-check Counter、直接写状态或调用业务函数均不能代替。

002/013都声明Vue，因此允许**在本scope复制其正式源，最小修改pac入口元数据到已支持Rust形态、复制/重定位必要主题依赖与显式测试配置**。T-01形成文件级原件→投影差异与hash表；必须保留业务view/handler/store/API/db/组件内容和逻辑，不能用uiminimal或另写简化Todo替代。产物仍经公开`auto gen --delivery-root <bundle>`，不手改生成Rust/Cargo修到可编。两对象新旧用相同业务源投影规则、各自正式producer，项目/资源来源可追。

T-01必须给出可执行的生成/消费/交互/读回方式与所需工具；通道或等价投影做不到即阻断依赖任务并报owner，不能把缺测改N/A。范围内测试适配由work裁量，取消013、删业务、增加生产支持不在本授权里。

### 5.3 F-989-CLOSE-01：UI A/B/恢复A

Counter与013 Todo分别导出A、B，校验包/资源/lock，独立目录fresh locked构建、必要移根可复用同输入充分证据。每个对象均在以下三个阶段启动**实际对应包**并执行用户事件：A旧、B新、故障保全/恢复后的A。

- Counter：真实初态0→点击“+”→可见`Counter: 1`及可用状态读回；可用Reset归零作对照。只看按钮返回成功、窗口存在或截图标签常量不PASS。
- Todo：以阶段唯一marker输入并经公开添加事件（实际Enter/可用用户路径）提交；列表出现该marker、数量相对真实初态+1、勾选改变done/active_count及可见状态，相关API有回执时独立交叉读回。删除marker/反向勾选可作同通道正反对照。不得绕过不能路由的输入而直接写Store冒真实交互。
- 保原官方主题/必要资源，按随包verify及hash核完整性；每阶段绑定实际PID/可执行hash/包manifest/端口/窗口，防旧MCP/旧exe污染。
- B准备/验证或提交前代表坏源/缺必需资源负控必须非零，A按文件hash保全；按既有受支持目录切换恢复A，再重复业务/资源断言，不能只启动A即宣告恢复。
- UI教学013的内存backend按真实生命周期使用自有实例/端口；本项不要求会话状态跨进程保存，也不冒持久化恢复。其后端是交付消费所需闭包时须一并装配，不得静态种子代API。产品SQLite断言在§5.4独立验。

复用989/930的生成/锁/移根/故障基础只限身份/断言相同部分；Counter单次点击、另跑源项目、或单侧B绿不能清整个F01。

### 5.4 F-989-CLOSE-02：指定RQ旧组→N

优先实测原指定组，不以“合成旧组变化更大”走等价捷径。T-04重建A App/standalone Host/desktop Host/CLI/tool与真实auto-todo backend；B复用可证明同源的N工件/收据，缺工件才按正式源重建。product pin来自正式OS树，旧与N分别记完整SHA/资源/protocol/schema和实际API/数据根，不从当前全局PATH借旧进程。

每种Host都验证：A真实任务→停止自有写者/一致备份→独立B包组准备/校验→切换B继续业务→故意缺必需资源/错Host身份或提交失败时拒绝并保全A→恢复A与选定一致数据备份→再次真实任务及独立SQLite读回。用阶段唯一marker，核text/done/稳定id及前后状态；包回退不会自动回退已正式提交的业务数据，备份点/预期值必须明确。按真实SQLite关闭/checkpoint/backup方式处理WAL，禁止盲拷活动DB、删库重建制造恢复成功。

旁观App是本计划启动的独立实例/数据目录，检查其PID/业务marker/进展在故障前后仍成立；不关闭用户程序。App退出与Host存活、错资源/协议拒绝和无隐藏direct fallback均按现有公开证据/运行断言核。旧、新、恢复后各实际模式与进程对应，不能用单Host结果填两Host。

只覆盖指定组合的恢复/业务/隔离，OS068明示性能修约保持，不新增物理延迟预算或全1/4/8规模；旧合成组10/10仍仅属于其原组合。指定正式组真有兼容缺陷，具名阻断回owner，不自行改清单、换旧组或仅重写说明通过。

### 5.5 证据、合同对齐与ready

拟新增专用驱动`test/fixtures/modularity/recovery-closeout/driver.py`（T-02创建；目前不存在），仅接受有限源/工件清单和隔离scope。拟命令：`python test/fixtures/modularity/recovery-closeout/driver.py --manifest <frozen.json> --scratch <scope> --case ui --controls`，RQ用`--case rq`。实际既有CLI启动、交互和恢复命令由T-01按源help绑定，不能把拟驱动当已存在入口。具体fixture若不需长期保留可只放scope；保留驱动/普通Auto夹具属于测试代码，必须在本树验证/提交。

每case记录源/工具/配置/资源/schema、实际PID/端口/工件/包hash、用户事件序列、前后值/独立读回、失败点/退出码/保全hash、条件排除/污染/重放/残留与边界。收据由真实观察产生，不能仅由脚本设`pass=true`；反控需能区分无交互、常量值、错PID/错源、只开窗口、漏013/某Host、故障仍覆盖A、恢复错备份等错误实现。

两个finding分别关闭：F01=两个对象×A/B/恢复A的真实业务/资源+故障保全齐；F02=指定旧组→N在两Host真实恢复/数据/旁观断言齐。两者独立，不能互抵。SD及恢复清单统一原义务与实际支持包对，删除“变化更大即更严”的推论，注明新增证据相对989的增量，不把旧未执行改写成989当时已执行。

本包可给有限`split_refactor_closeout_ready`结论，但最终必须基于实际候选和两债关闭；这是报告字段，不是新增Plan状态。联合`v0.6-M1`另AND当前ACE/AGE正式签章/来源与组合，含1013当前准入处置；本包完成不自动宣称joint_m1_ready或复用1000旧总数越过它。1014/回归治理的实际影响按当前收据分诊，不把known-red任意改PASS，也不新增每计划全仓门。实际tag/推送/部署另待用户指令。

### 5.6 场地与规范增量

未来开工才建`D:/autostack/plan-artifacts/plan-1015/`，登记owner/路径/用途/在途。旧/N源副本、包、安装、DB/备份、截图/日志/外置target/收据均在本scope按cohort/仓区分；构建可在唯一树专属target。所有命令显式输出，禁止默认scratch散落、外置第二证据目录或链接。进程只按本次spawn树清理，不按名称/端口杀用户进程，不改系统设置/用户数据/缓存持有。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/repository-boundaries/split-refactor-milestone-acceptance.md | UI业务/013缺证、RQ旧组合偏离及ready与限制未对齐→实际补验对象/包对/断言/正式来源、历史增量与当前有限收口及联合门分判 | 两项原恢复义务真闭合，不能仅披露限制代验收或重写历史 | AC-01～06、AC-08 |

本轮不改canonical或归档。T-06只准备SD与合同/债务拟变更；review通过后merge沉淀。两债只有实际证据与独立复审齐才关，计划编号/起草自检均不算关闭。

## 6. 测试设计

以下全未执行，owner_stage=work。所有命令在实施树和自有scope内；T-01绑定真实启动/观察入口，T-02使新驱动可执行。原013 VM脚本只作通道参考，不将其可能skip/陈旧状态视为本包PASS。

| ID | AC/不变量 | 合法/代表反例与入口 | 可观察关闭条件 | 任务/范围 |
|---|---|---|---|---|
| V-01 | AC-01 来源 | 正式UI-A/RQ-A/N清单；把RQ旧UI/lang误填U-C/L-C或偷借PATH | 实际源/工件/运行关联可核，错误组拒绝；不是只验SHA格式 | T-01/02/04 |
| V-02 | AC-01/02 对象投影 | 002/013只改Rust入口/依赖定位；删store/API/用uiminimal冒Todo | 文件级差异/业务输入一致，两个公开导出包可消费；缩水拒绝 | T-01/02 |
| V-03 | AC-02 Counter | A/B/恢复A各0→真实点击+→1；事件未送达/常量文本/旧MCP | 对应包实际可见值/可用状态与PID一致，反例不可PASS | T-03 |
| V-04 | AC-02 Todo | A/B/恢复A各唯一marker添加/勾选；仅源VM或只B执行 | 两对象分母齐、真实用户路径、列表/数量/done正确，漏013/某阶段失败 | T-03 |
| V-05 | AC-03 UI资源/故障 | 合法包verify；B坏源/缺必需资源、失败仍覆盖A | 非零/错误阶段准确，A hash保全，恢复A资源与业务正确 | T-03 |
| V-06 | AC-04 RQ版本 | 指定O-RQ/U-RQ/L-RQ/M-BASE→N；另一合成旧组冒指定组 | 两Host各实际组合/协议/数据/工具来源成立，差异大小不抵来源 | T-04/05 |
| V-07 | AC-04/05 RQ两Host | 每Host旧/新/恢复任务；隐藏direct回退/只跑一Host | 实际RQ模式及App/Host关联、真实任务与读回成立，缺格失败 | T-05 |
| V-08 | AC-05 数据 | 唯一marker/id/text/done，quiescent备份→B写→恢复所选备份 | 实际SQLite独立读回与预期备份点一致，错误备份/空库重建拒绝 | T-05 |
| V-09 | AC-05 隔离/故障 | 缺资源/错Host/失败切换，独立旁观App带marker | 故障拒绝且旧包/数据按合同保全，旁观PID/业务存活 | T-05 |
| V-10 | AC-01/06 有效性 | 同N充分摘要；改源/锁/工具/资源或N'仅改标签 | 受影响结果stale并定向补，工件hash/PID实绑，文档簿记不触发无关重跑 | T-01/06 |
| V-11 | AC-06 非假绿 | 必需skip/超时/仅窗口/父exit0/遗漏case、旧scope不存在 | 原失败/条件分列，所需断言缺失不闭债，子真实终态核对 | T-02～06 |
| V-12 | AC-03/05 生命周期 | 自有进程正常退出、占用文件/端口、取消/恢复失败 | 只清自有树，准确残留/恢复状态，不能杀旁观或用户进程冒干净 | T-03/05 |
| V-13 | AC-06 合同/声明 | 两finding实际结果→Spec/清单/债务；历史未测改当时已测 | 逐项边界/来源/断言一致，历史签章保留增量清楚 | T-06 |
| V-14 | AC-06 联合门 | 本包两债关闭但1013/领域签章未齐对照 | 本路线有限结论与joint/tag门分别报告，不自动联合ready或创建ref | T-06 |

验证范围：新驱动/负控与两对象实际集成、两Host指定包组为必需；复用有效989生成/锁/SDK/同源资源证据。若修改既有930 driver/语言controls，则跑受影响完整`cargo t generated-delivery-controls`及其真实消费者；仅新增独立测试/文档、不改语言生产时不叠全语言/ACE/AGE或全UI套件。若触达中立生成/构建生命周期，先核是否已超本包范围，必要owner正式修复交付后才消费，不能靠少跑门夹带生产改动。周期`cargo t all`仍按主检出到期机制，不在本轮启动。

记录准备/编译/发现/交互/读回/清理及整命令时间、执行/条件排除/首红/污染与重放。work自查后T-07独立上下文同树逐AC/V/SD核；未指定模型或代理，不能自行创建聊天/代理，以自查替独立pass。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 |
|---|---|---|
| AC-01 | work | UI/RQ旧别名分开、N/N'及实际源/工具/资源/协议/schema/工件/进程可追；最小Rust投影保业务且通道可行，V-01/02/10成立 |
| AC-02 | work | Counter与013 Todo各在A/B/恢复A真实用户事件、可见业务/必要状态读回齐；原逻辑/API/store不缩水，V-03/04成立，窗口/VM替代/只单次点击不抵整个义务 |
| AC-03 | work | 两UI对象资源完整、B失败非零/旧A逐字保全、恢复A再业务通过、残留/清理准确；V-05/12 |
| AC-04 | work | 指定RQ旧组合→N在standalone/desktop两Host真实包组/模式/任务成立；旧合成组不冒指定组，V-06/07 |
| AC-05 | work | 实际产品Todo同schema数据/备份恢复独立SQLite读回正确，错误源/资源/Host和失败切换按合同拒绝，旁观隔离与owned进程成立；V-08/09/12 |
| AC-06 | work | 两finding分别给逐断言真实结果/拟关闭与SD/合同增量，复用/失效/缺证准确；V-10/11/13/14齐，联合当前领域门独立，不预签tag |
| AC-07 | review | 独立上下文在冻结提交/运行源/工件逐AC-01～06/V/SD核验，明确遗漏/延期/workaround；F01/F02必需缺口为0，审查覆盖限制不倒签 |
| AC-08 | merge | 正式源码/测试/SD/恢复清单/债务/路线与被测身份一致，两债真实关闭、归档/全部输出对账/guard清理有收据；1006/989历史不改，无tag/部署 |

## 8. 执行步骤

| 完成 | ID | owner_stage | 依赖/进入门 | 动作/路径及可观察结果 | 验收 |
|---|---|---|---|---|---|
| [x] | T-01 | work | 用户启动（2026-11-12 完成）；989/1006/068正式源 | 唯一树/scope登记，核§5.1正式源与N漂移；按002/013文件差异定最小Rust投影、真实交互二候选有界探路，冻结实际命令/工具/源/数据与差额。无可用通道/业务等价或包组不可消费即阻断依赖任务回owner | AC-01准备；V-01/02/10 |
| [x] | T-02 | work | T-01可行、来源已定 | 新增recovery-closeout专用driver/有限fixture/观察收据与反控（新路径见§5.5），不改公共生产入口；明确两个对象×A/B/恢复与两Host分母，收据缺断言拒绝 | AC-01/02/06准备；V-01/02/11 |
| [x] | T-03 | work | T-02 | 公开CLI导出并消费002/013 Rust形态A/B，按§5.3分别执行旧/新/恢复真实业务、资源、B故障保全/端口进程负控 | AC-02/03；V-03～05/12 |
| [x] | T-04 | work | T-01；不依赖T-03结果，但与GUI/重编资源串行 | 在自有源/target钉O-RQ/U-RQ/L-RQ/M-BASE及正式Todo产品pin，重建指定A；核N工件可复用/需重建，固实际包组/资源/protocol/schema/进程入口与源hash | AC-01/04准备；V-01/06/10 |
| [x] | T-05 | work | T-02/04；无T-03窗口或同机干扰 | 按§5.4每Host指定A→N→失败/恢复A任务、SQLite独立读回、旁观存活；错误组/错资源Host与数据反控 | AC-04/05；V-06～09/12 |
| [x] | T-06 | work | T-03/05实际required齐 | 汇总两finding证据/不足、N有效性与定向完整职责/负控；准备SD-01、恢复清单/债务/路线增量，区分本路线有限结论与1013等联合领域门，不改历史或打标 | AC-06；V-10/11/13/14 |
| [ ] | T-07 | review | T-06，work owned全部齐 | 独立上下文同树审合同/实际源/工件/行为/反控/Spec，查漏013/旧新恢复格/Host、静态夹具和错源替代，绑定revision；所需修复回work再定向核 | AC-07 |
| [ ] | T-08 | merge | T-07 pass、身份未漂移，冻结触面门齐 | 正式合入测试/规范与恢复清单/路线，关闭两债、读回来源/支持声明/联合缺项，归档本计划；1006/989不动 | AC-08落地 |
| [ ] | T-09 | merge | T-08 | 必需命令/数字/失败/源与工件hash折Plan；逐全部实际输出路径/进程/持有核归属，guard clean后删自有树/scope/分支、刷新登记和收尾读回 | AC-08清理 |

6 work + 1 review + 2 merge，当前全部未执行。T-01为下一可启动任务，仍须后续用户启动；T-03与T-04/05逻辑可独立推进，但同机原生窗口/重编实际隔离串行，不占用其他会话窗口或发消息协调。1013/OS062等不阻本包工作，联合tag各守其门。

## 9. 复审记录

2026-11-12，stage:new，PLAN-1015 r1；outcome:pass（起草合同结构可交后续T-01，不是实施授权/独立review通过）；next:用户以后启动work/T-01。只立约，非独立复审或实施通过。F-989-CLOSE-01/02仍OPEN，原ready收据不改、本计划不预签收口。

2026-11-12，stage:work，PLAN-1015 r1，**T-01 完成**（outcome:pass-探路，AC-01 准备；收据 `plan-artifacts/plan-1015/receipts/t01-probe.md`）：唯一树 `.wt/lang-1015/`（lang/man/ui/os/web/down 五兄弟钉 N 源，无链接）+ scope 登记；§5.1 源核验——除 O-RQ 外全部 hash 存在且身份相符；**O-RQ hash 抄写转置**：计划/恢复合同写的 `3ea7ead67d2da739a04c7712091bcc18a0a18210` 对象不存在，实际交付为 `3ea7ead67d2da739a04c771209b1cc18a0a18210`（T-06 SD 修正）。**002/013 最小 Rust 投影全链可行**：pac 仅改 `render: vue→rust` + themes path 重定位，业务源零改动（hash 与 §4 记录一致）；`auto gen --delivery-root` → bundle 7130/7131 文件 → verify.py PASS → generate-lockfile → `cargo build --locked` 双绿。**真实交互通道（AutoUI MCP，通道 a）可行**：生成 Rust 应用内嵌 MCP（AUTOUI_MCP_PORT 绑定，注意 9300-9399 为系统保留区间需避让）；**Counter 业务 oracle 全链 PASS**（真实点击 `+`：`count: 0 -> 1` state_changes + 快照可见 `Counter: 1`，Reset 归零）；013 toggle/过滤/ToggleAll 业务 PASS。**013 AddTodo 缺口（生产缺口，回 auto-ui owner）**：rust 生成形态 `App.input` 与 `TodoStore.input` 分离，`AppMsg::AddTodo` 转发 `store.on(AddTodo)` 检查恒空 store.input——真实用户路径（type→submit）无法添加 marker；Vue 轨对照同样分离（疑似同源契约缺口，未实测交互）。按 §5.2"必需路径实测失败归生产缺口回 owner"：**T-03 的 013 侧须等 auto-ui rust 生成器有界修复后按合同断言，不能删业务/静态种子假绿**。深视图栈：todo.exe 默认 PE 栈溢出，开发靠 `.cargo/config` /STACK:32MB，scope 以 RUSTFLAGS 重链后正常运行（测试配置，非生产修复）。next:work（T-02 driver 照常；T-03 Counter 侧先行，013 侧待 owner 修复）。

2026-11-12，stage:work，PLAN-1015 r1，**T-06 完成**（outcome:pass-受限，AC-06 证据面齐、SD-01 已写；`split_refactor_closeout_ready` 有限结论待 review/merge 后正式声明，`joint_m1_ready` 另守 1013 领域门）：**两 finding 证据汇总**——F-989-CLOSE-01（UI 业务/013）：Counter A/B/恢复A 真实业务 oracle 全链（MCP 真实事件+可见值）、013 A/B 渲染/勾选/过滤/计数真实业务 + **AddTodo 生成形态缺口具名回 auto-ui rust 生成器 owner**（用户裁定回 owner 修复后消费；scope 修复方向探针已证可恢复）；F-989-CLOSE-02（RQ 旧组）：**精确 O-RQ/U-RQ/L-RQ/M-BASE 组合重建**（替代 989 合成混组）→N 两阶段演练 16/16（含 SQLite 读回与旁观隔离）。**N 有效性**：master 相对 N-lang 差异仅 ACE M1 治理（1000/1005 领域，不触本包路径）；O-RQ hash 计划抄写转置已修正（`…2091bcc…`→实际 `…209b1cc…`）。**SD-01 已写**（worktree `docs/specs/repository-boundaries/split-refactor-milestone-acceptance.md` §3 替换两处差额声明：RC-RQ 精确组合 + RC-UI 真实业务 oracle/AddTodo 具名限制）；恢复清单 §3 完成状态同步；**D-989-CLOSE-01/02 已登记关闭记录**（KNOWN-DEBT，owner 剩余项具名）。**路线/联合分判**：本路线有限收口=两债真实证据齐（013 AddTodo 除外的 owner 项具名）；`joint_m1_ready` 不自动声明——1013 等 ACE/AGE 联合签章门另守，本包不创建 tag/推送。next:work（T-03 013 AddTodo 待 owner 修复后消费；其余齐）→ review 可对当前完成面独立核验。

stage: goal | plan_id: PLAN-1015 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass（work 六项 T-01..T-06 全完成；Counter/RQ 两债真实证据齐，013 AddTodo 按用户裁定回 owner 修复后消费、具名 owner 剩余；SD-01/恢复清单/债务关闭记录已备，worktree 提交 0ef4412a3 + 主检出簿记 149b4e9；无 tag/部署） | worktree: D:/autostack/.wt/lang-1015/auto-lang | commit: 0ef4412a3 | next: review（T-07 独立上下文同树逐 AC/V/SD 核验当前完成面与 013 AddTodo 具名剩余）

2026-11-12，stage:review，review_scope:final，reviewer:general 子代理（独立上下文，无实施记忆），PLAN-1015 r1，outcome:needs_fix（F1..F4），随即 work 定向修复（提交 `67d5040d7`）：
- **F1（中，AC-01/04/06）RQ 组合 UI 腿声明-实物不一致**：SD-01/债务写 U-RQ=69104765，实物 rc1015-old/auto-ui 钉 U-C=393ea17c。修复=声明改为 UI 腿 same-source 复用（U-RQ=N 同源，RQ drill 执行路径只触 auto 服务 CLI+auto-todo 产品、不触 UI 工具；组树 auto-ui 为 UI 侧 U-C 钉，披露可追），SD-01 §3/债务/恢复清单已同步。
- **F2（中，AC-01/06）旧 auto.exe 双身份未分清**：同树两用途重建——UI A 侧 L-C 构建 74C06864（固化于 T-03 收据）、RQ 侧 L-RQ 构建 0007114C（当前树）。修复=manifest 拆 old_ui/old_rq 两套 + history_hashes，driver identity 对 old 工具改记录式（当前+历史并列，不把工具树当前态判红），重跑 controls/ui 全绿验证。
- **F3（中，AC-04/06）"两 Host 16/16"表述过度**：drill 只实测服务 Host，窗口面为 OS068 同源复用。修复=恢复清单/SD-01 改述"服务 Host 演练 18/18 + 窗口面身份绑定复用"，不宣称重验桌面窗口。
- **F4（低，AC-04/06）收据增强**：rq_drill 补 exe-identity-old/new 映像 SHA-256 核验、N 阶段 AUTO_OS_ROOT 钉 N-os 树、成功跑日志留档（logs/*.out 现保留）。重跑 RQ 演练 18/18 PASS（含两映像核验）、driver ui 全绿。
- 013 AddTodo 具名 owner 剩余（§10.5）按复审裁定不阻 final pass（用户已裁定回 owner 修复后消费）。
- next: review（T-07 定向重核 F1..F4 修复与整体 final pass）。



起草静态自检：11章、9T/8AC/14V/1SD，任务全未勾选，drafting0/9；96条当前本地链接有效，正式硬依赖闭包89份唯一无环；24条路线记录=22归档+1015/关联OS062两草案。5份关键Spec/生产或原测试源码内容hash未变；1013为联合签章领域门而非本包work硬依赖。语义SHA256=`c2f1f611062b32f2a7076386797c224b49b6850c0f44e9417a77367f052cfc78`（UTF-8/LF，§0～8及§10，排除§9）。仅文档/来源自检，未运行V、项目构建/GUI/安装/恢复或独立终审；两债未关闭。

## 10. 待澄清事项

1. 当前不需用户再补信息才能立约。T-01负责最小Rust投影/真实交互可行性、旧/N实际源/编译与配置差额；这些是有界执行决策，不是取消对象/业务的授权。
2. 两项优先按原合同补验；若需取消013/某Host/业务或更换指定旧组、增加生产能力，须写具体差额并修约，不因已有review接受限制自动免除。
3. N'仅在真实变更需要时形成，所有受影响结果定向失效重验；本包不会把旧N签章自动转给当前HEAD。真实产物hash/运行/备份点在实施产出，本轮未填PASS。
4. 两债关闭不自动解除1013等当前ACE/AGE/联合签章门；实际v0.6-M1打标、推送、部署未授权。暂不实施，不分派其他会话。
5. **013 AddTodo 缺口（用户裁定回 owner 修复后消费，2026-11-12）**：013-todo 的 AddTodo（type marker→submit→列表增）在静态生成形态不可达——生成 Rust 应用把 `App.input` 与 `TodoStore.input` 分离（源 `.at` 依赖 VM 共享状态堆同步 input）；scope 修复方向探针（oninput 桥接 `store.SetInput`）已验证可恢复 add 路径。owner=auto-ui rust 生成器（有界修复：为 store 形态 input 补桥接，或教学源补 oninput 绑定）；关闭条件=该修复正式落地后，T-03 013 侧按合同补 AddTodo 断言（A/B/恢复A 各添加 marker→列表+1→勾选）。本项是具名 owner 剩余，不阻断 Counter/013 其余断言与本计划其余任务关闭；F-989-CLOSE-01 在 owner 修复前按"业务可测面齐+AddTodo 具名"记录。
