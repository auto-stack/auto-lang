---
plan_id: PLAN-1106
status: drafting
feature_name: ASE组件事件增量会话与历史重放验收修复
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [PLAN-1023, PLAN-980, PLAN-981]
supersedes_spec_components: [docs/specs/ase/m0-interactive-sessions.md]
new_spec_components: [docs/specs/ace-migration/component-event-sessions.md]
touched_goals: [GOAL-002, GOAL-007, GOAL-010]
---

# [PLAN-1106] ASE组件事件增量会话与历史重放验收修复

## 0. 变更摘要

r1，drafting，0/10，**只制定后续修复合同，先不启动实现**。承接1023归档后的源码核查发现：对象与最终状态保留并不代表业务增量执行，`AceStateEngine::dispatch_impl→refresh`仍重编/重跑累计历史场景，外层`loads/dispatch_log`漏计内部执行。新计划实现真实程序/模型持有与新事件单次执行，在同一真实路径用执行层oracle修复原验收盲区。

[ASE路线](../design/strategy/ase-m0-delivery-roadmap.md)当前把M0-SESSION的无重放子门恢复为pending（债`D-ASE-1023-REPLAY`）；保留1023 archived、原revision/勾选和历史收据，仅追加可见更正。可视交互原型及3/7行为证据继续有效，不宣称本修复已实现。

## 1. 目标

普通Node源经共同Schema/唯一verify与ACE编译，创建会话时初始化一次；固定源/profile/ABI下保留已验代码与运行模型，新事件只执行被选中的handler，不重新执行已成功历史事件，不依赖Rust重算count或旧VM。UI仍由受检会话状态差量驱动。

保持1023原域：Windows x64、单原生窗口、两个存活应用会话、A=3/B=7与同app隔离、关开归0、错代拒绝、第三会话拒绝、有效resize/焦点和20轮资源收敛。保留当前公共桥已支持的类型/props/载荷等真实职责，不得为修Counter删除其它消费者；更广旧源支持按T-01冻结实际基线。

非目标：重做浏览器外壳、网络服务、安装器/沙箱、AC/A2R全迁移、热编辑/checkpoint恢复、通用外部IO exactly-once或把整套AV/调度器重写一遍。源码显式改版可销毁旧会话后重建，必须具名；正常事件/切换不能偷偷触发。

## 2. 架构方案

`普通Node/工程闭包→共享领域语义与受检编译→初始化/事件/状态读取入口→ACE持有代码与状态根的事件会话→当前typed队列→只调用本次handler→读回差量→ui_apply_*→选中frame`。

代码、语义检查、MIR/ABC验证与执行仍由ACE原owner提供；状态根、对象/字符串/资源由其运行时owner保活。UI桥只保存受检会话句柄、输入路由与显示镜像，不把历史`ComponentScenario`当在线执行器，不人工把handler翻译成REPL字符串或Rust函数。历史场景观察器可留作显式离线oracle，不能在在线失败时静默fallback。

T-01最多比较两条既有公共接缝：①`ProgramSnapshot`/全局cell和初始化纪律承接编译器生成的实例状态/事件入口；②已有owned/resident会话保留对象/资源根，经同一handler降级调用。两者都必须通过最长真实链，不从API名字推断可用；选定一种后只实现一套。新公开事件会话API、测试文件名均为拟新增，准确位置与支持域先冻结。

## 3. 技术栈

语言仓ACE独立workspace（core领域lowering、av消费者，必要时av-core公开入口/资源根的最小扩展）；auto-ui的`ace_state_bridge`、`ace_node_desktop`与`ase_m0_sessions`消费层；继续现有ui980受检FFI、Iced窗口/投影，不改外壳为另一套UI。

主导计划正文在auto-lang；跨仓唯一组拟`D:/autostack/.wt/lang-1106/{auto-lang,auto-ui}`，分支`codex/plan-1106-dev`，所需web/down sibling按实际闭包补齐；本轮不创建。严禁链接，plan簿记留主检出，源码/构建/测试只在后续专属树。

## 4. 需求分析与背景调查

用户2026-11-19明确要求针对1023历史重放制定修复计划，实现只执行新事件的真实增量会话，并修正路线/验收记录；保留1023归档，先不启动实现。当前授权含本Plan、路线、事实更正/债务登记，不含新源码/fixture/探针、建树/构建/测试/GUI、提交/合入或签章；无模型/工期/token预算。其它会话WIP保留。

依据[Specs overview](../specs/overview.md)、[ASE交互现状](../specs/ase/m0-interactive-sessions.md)、[970会话边界](../specs/ace-migration/m2-sessions.md)、[UI组件状态与宿主规范](../specs/ace-migration/m2-autoui.md)、[源码重放规范](../specs/auto-test-case/ace-source-only-replay.md)及[1023归档](archive/1023-ase-m0-interactive-sessions-and-acceptance.md)。此次canonical更正是用户授权的已观察事实纠正，不提前沉淀未来增量支持。

读取auto-lang HEAD=`670c720a893d226338ef9ad4a89475bc957b2da4`，auto-ui HEAD=`481a3cb55984bbe4783f2348a6881d198aedb6ab`及当前文档WIP。源SHA256前16：语言`av/src/age_component.rs`=`02a8c65a657a9f5e`，`core/src/source/age_component.rs`=`2f5f937e01888d80`，`av-core/src/program_snapshot.rs`=`0f666b6ea4abaa9f`；UI`ace_state_bridge.rs`=`0ae496dbdc342f29`，`ace_node_desktop.rs`=`773636ae1af47ba5`，`ase_m0_sessions.rs`=`79861c027bf956ba`，`tests/ase_m0_sessions.rs`=`99fb9e8b75944c65`。开工重核来源及并发改动，不拿移动HEAD当永久证据。

已观察的producer/consumer：

| 实际路径/符号 | 事实与影响 |
|---|---|
| auto-ui `ui/ace_state_bridge.rs::{dispatch_impl,refresh}` | `ops.push`后汇总所有存活实例历史，调用`observe_component_scenario_states`；读回状态是重算的结果 |
| lang `av/src/age_component.rs`组件场景观察链 | 普通源前门/verify/lower/执行整个场景，不能从外层只调用一次dispatch推断底层一次handler |
| auto-ui `ui/ace_node_desktop.rs::{BridgeEngine,NodeDesktopBridge::load,step}` | 单模块事件走上述Component引擎；包桥另有Package面，受本次共享修改影响的范围需审计 |
| auto-ui `ui/ase_m0_sessions.rs::{open_session,select,step_selected,dispatches_of}` | 引擎对象可常驻，`loads`只在外层装载计，dispatches来自step/dispatch_log；未观察refresh内部执行 |
| auto-ui `tests/ase_m0_sessions.rs` V-07/08控制 | 主要识别外部重装/重新派发；同owner内refresh重放仍可保持这些计数与3/7结果，因此旧PASS不足以关闭原无重放要求 |
| lang `av-core/src/program_snapshot.rs`、`av.rs` | 已有code绑定、global cells/init状态、entry/steps等执行接缝；只是候选基础，尚未证明能直接运行保留Node模型 |

上述为静态核查，不是本轮已运行反例或基准结果。1023原实现/收据均保留，更正涉及AC-01/02、AC-R与M0-SESSION/M0-CLOSE/AC-D的“完整成立”结论；隔离/显示/关闭等已验子面不一并判失效。

## 5. 详细设计

### 5.1 首次入口与范围冻结

前置1023/980/981按已交公共子面与当前源码核验，不等待整个ACE/AGE M2、1074/1075全部内容或发行退役。T-01还须检查当前1071等UI后继的共享写面；它们只在实际采用其接口时成为具体子面依赖，不凭编号增加整包等待。不重开归档970/1023。

T-01在获得后续实施指令后先核原例、创建真实执行层观察口，记录当前一次新事件实际引起的编译/初始化/handler执行，与旧外层计数对照。最多两机制先证：一个普通Node实例初始化→Inc→Inc→读值2→隐藏切回→Close→新代次初值0。未通过则needs_replan，具名缺接口/owner/最短关闭条件，不进入T-02以后；不临时缩成手写HIR或纯Rust计数。

T-01冻结完整受影响公共方法/字段类型/props/同定义实例、无参与已合法payload/异步完成行为，选择真实当前语料作为兼容oracle。M0主例仍无参int，但改公共桥不能把既有合法类型/消息改成统一拒绝来省实现。包桥如未改保持原证据；若改共用执行/选择路径则补真实包消费者。

### 5.2 程序、状态与事件生命周期

| 对象/阶段 | 合同与owner | 可观察结果 |
|---|---|---|
| immutable程序 | 源闭包/Schema/profile/ABI/权限身份固定，普通源经唯一verify；编译/链接产物可共享 | 热事件不调用parse/compile/lower/link；身份变更明确拒绝或显式重开，不默用旧proof |
| 会话模型 | 每App/实例独立运行时状态根，初始化一次，字符串/对象/资源由真实owner保活 | 同app双会话3/7不串；只保留外层id、历史或显示缓存不算模型保留 |
| 单新事件 | 宿主生成session/generation/sequence路由，声明/类型/权限/代次先验证，随后只执行目标普通handler | 每次成功接纳事件一个handler入口；旧事件不再次执行；参数只按现有已验语义绑定 |
| 状态读取/呈现 | 从同一会话读回真实状态，纯读取/getter不得隐含运行历史handler；UI只提交实际变化槽 | 改业务源后结果跟源而变，Rust无业务算术；双车道current限制保留，不暗写隐藏会话 |
| 切换 | 保留业务会话，只切换/重提交显示镜像 | 业务编译/链接/初始化/handler计数不变；不得每次按历史或seed恢复相同数字 |
| 关闭 | 先撤路由/输入与显示，再取消所持任务、释放状态根及最后引用 | 晚到事件零业务/UI效果；重开新generation仅一次初始化，20轮有界收敛 |

不要求每个事件零分配或UI布局零工作，要求业务语义不重复。不可把完整历史重放改成“每次仅一条场景＋重新初始化/重编译”就称增量；也不可把所有状态搬进UI Rust表、只让AV做一次加法。

### 5.3 失败、重投与资源

沿公共执行预算与类型/权限拒绝规则。未知/关闭/错代/重复序号在业务效果前拒绝；同一已成功事件的重投不再执行，可返回原结果或具名duplicate，策略在T-01固定并有真实断言。正常新点击有新序号；不能按相同消息文本误去重两个合法Inc。以有界水位/接纳记录实现，不累积无界业务历史。

对本包状态事务域，handler的trap/资源耗尽/取消不发布半状态，保留上个已提交模型/显示；候选写入/owned资源丢弃并可受控处理下一个合法事件。不能回放全部先前事件来“恢复”旧态。外部不可逆IO的跨进程exactly-once不在本域；测试审计效果是非业务观测，不伪称通用IO回滚。若既有共享路径具有额外错误语义，T-01逐项保留，不能为了修复降低旧合同。

### 5.4 观察口必须进入执行层

计量实际编译/链接/状态初始化、从编译产物绑定的handler入口、重复历史调用、读取与状态根释放；观察点在真实ACE producer/executor或其公共trace/call/lifetime钩子，带program/session/generation/事件序号与绑定DefId/span。UI的loads/step/dispatch_log仍可辅助，但不作为增量oracle。计量来源、未覆盖路径和单位必须公开；新计量不得只在wrapper按事件数自增。

对序列Inc×3，成功handler入口应恰3；随后切回保持3，Reset再接Inc得到1且新增入口2。双会话A×3/B×7实际各3/7；闭后新会话generation变化且初始化一次，旧handler不再进入。另取0/1/3/7/32个事件分组、有限状态保持与依次关闭；这是确定性正确性计数，不用墙钟快慢代执行次数。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ace-migration/component-event-sessions.md | 当前场景观察重放→实际已验保留程序/状态根、事件单次执行/事务/关闭及支持表 | 公共能力归ACE，不藏于ASE wrapper | AC-01..05 |
| SD-02 | modify | docs/specs/ase/m0-interactive-sessions.md | 更正后无重放pending→新执行层oracle闭合后按范围恢复M0-SESSION | 修复证据链，不覆盖历史 | AC-06/07 |
| SD-03 | modify | auto-ui: docs/specs/runtime/ase-m0-frame-consumption.md | 外层对象/计数→消费真实增量会话与实际执行/释放证据 | UI端共享同一语义 | AC-02..07 |
| SD-04 | modify | docs/design/strategy/ase-m0-delivery-roadmap.md / docs/plans/KNOWN-DEBT-AND-RISKS.md | D-ASE-1023-REPLAY OPEN→全部修复/独审/正式交付后closed；1023仍archived | 不用新计划创建冒能力完成 | AC-07/AC-D |

开工建立并登记`D:/autostack/plan-artifacts/plan-1106/<repo>/`在途owner/用途。临时探针/日志/截图/外置target只在该scope，永久源码/测试是受版本控制的产品资产；正常构建在唯一worktree。无链接/第二证据目录，Windows清树前只用`pwsh -NoProfile -File D:/autostack/wt-guard.ps1 <每仓工作树>`。

## 6. 测试设计

以下测试、观察口和新API均拟议，首次红尚未实跑。本轮没有运行任何测试。T-01先建立最长链的真实观察/最小永久反例，T-06固化完整目标；不能先标PASS再等待测试创建。实现/内部自查由执行者，最终review独立上下文、同一权威树；用户未指定agent/model，本轮不创建。

| Case ID | 关联AC/保护规则 | 真实入口与输入 | 独立oracle/判别力与覆盖边界 |
|---|---|---|---|
| V-01 | AC-01 基线与先证 | 当前ASE→NodeDesktopBridge→Component引擎；普通Counter执行3事件，再隐藏切回 | 保留旧3/7结果，真实编译/初始化/handler观察暴露内部重放；记录actual，不能从源码推断数值冒实测 |
| V-02 | AC-02 单事件增量 | 新公开会话载入普通Node，0/1/3/7/32事件；Inc×3/Reset/Inc | 固定源仅一次准备/每实例初始化一次，handler入口数随新接纳事件精确线性，值3→0→1，正常读取/切回不增加handler；无按N展开的驱动源 |
| V-03 | AC-03 隔离/类型与owner | A×3/B×7、同app双会话、同定义实例，当前合法字段/props/payload基线 | 状态/资源根归属正确，公共桥合法源仍成功且错误保原stage/type；不以两份纯Rust状态表替代 |
| V-04 | AC-04 拒绝/失败 | 错代/关闭/重复seq、两个合法同名Inc；trap/资源/取消失败后合法事件 | 拒绝零业务/UI效果，重投不重复，正常两Inc各一次；失败旧态/显示不变且候选资源释放，恢复不回放成功历史 |
| V-05 | AC-05 关闭/资源 | 切换/关闭/同app重开，20轮；在途取消和最后owner释放 | 重开新gen初值0；旧gen无handler/UI效果，受管状态根/任务/目标收敛，共享缓存单列 |
| V-06 | AC-06 同owner隐蔽重放负控 | 在真实adapter/executor缝强制调用旧refresh路径，保持外层id/loads/dispatch_log与最后数字不变 | 执行层编译/初始化或旧handler次数必红；不是重新new一个bridge供另一计数器断言，恢复增量路径绿 |
| V-07 | AC-06 重建/一条场景负控 | 同owner内每事件重编/重建后用seed补数，或每次只跑一个重新初始化场景 | 最后数字一致但真实prepare/init/状态根身份不符必红；不会误杀显式close/reopen正控 |
| V-08 | AC-06 getter/UI重复执行负控 | 使read_all/文本刷新/切换意外调用业务handler，或重复提交同事件 | handler/审计效果实际次数红；UI差量和handler执行分账，静态画面通过不能掩盖 |
| V-09 | AC-07 真实产品链 | 现有ASE sessions窗口，原3/7、同app、关闭重开、限额、迟到、resize/焦点及20轮 | 业务/显示/执行层三方证据对应；自动窗口与人工真实输入分账，不借修复移除原case |
| V-10 | AC-07 收据适用性 | 旧1023 PASS、错版Schema/ABI/旧制品/旁路oracle、合法新收据 | 旧外层PASS只供行为参考，不能关闭无重放门；修复独审与正式来源齐后才恢复相关结论 |

正式入口仍以固定Cargo职责组为主：语言改面选完整`cargo t ace-source ace-av`，若动共享lower/公开执行再加完整`ace-lowering`/`ace-facade`及必要真实下游，T-01冻结选测理由；新AV测试目标在现有conformance组织内登记（拟`component_event_sessions_1106`模块，T-02/03创建，不能假称现有）。UI以`cargo test -p auto-ui-runtime --features ace-bridge --test ase_m0_sessions -- --test-threads=1`完整目标为主，补现有`ace_node_desktop`与`plan041_instance_ownership`真实受影响case/包桥目标，准确命名在T-01核当前树。`cargo t ase-m0`及`ase_m0_text`相关完整目标在消费链变化时复验；未变证据复用。

所有新增定向case尽量每例≤10s、真实GPU/焦点/窗口单实例，正式压力仅显式`cargo t ace-perf`不成为每计划门；保原样本/资源/ignore/平台条件。Windows排除Linux/WSL腿，记裁定排除归专用Linux环境，不充PASS。不为此修复重复跑全AGE套件。review后冻结受影响完整职责/必要消费者并集；准备/冷编译/发现/运行/清理与整命令分列，零选择/NOT_RUN不PASS。

## 7. 验收标准

| ID | owner_stage | 可观察验收与验证 |
|---|---|---|
| AC-01 | work | 真实源码普通前门与执行层原红/两机制先证有证据，明确单一策略、类型/资源/错误/观测合同；V-01 |
| AC-02 | work | 已验代码和独立状态根保留，热事件无重编/重初始化/历史handler，真实新事件恰一次，读取/切换无业务重跑；V-02 |
| AC-03 | work | A/B与同app/实例隔离，公共桥原实际支持域守恒；状态不在Rust显示缓存中代算；V-03 |
| AC-04 | work | 错代/关闭/重复先拒，失败不发布半状态且不回放恢复，合法同名新事件不误去重；V-04 |
| AC-05 | work | 关开新代一次初始化、最后owner与在途任务释放，20轮受管资源收敛；V-05 |
| AC-06 | work | 同owner内部重放/重建/getter重跑负控沿真实路径被执行层oracle抓红；数字/外层计数相同仍不能蒙混；V-06..08 |
| AC-07 | work | 原窗口/3与7/归零/限额/迟到/资源条件保留，新的收据足够核M0待修复子门；V-09/10 |
| AC-R | review | 独立审计源码→受检程序/状态根→handler真实入口→差量UI/关闭，重放敏感性与支持面全核，缺项不PASS |
| AC-D | merge | 受审冻结并集/SD-01..04正式交付后才关闭债与恢复M0待修复子门，1023保持归档/原历史；完成全输出对账及guard清树 |

## 8. 执行步骤

未来实施前，当前工具若有原生TODO能力，应按本节稳定ID登记后再动代码；无此工具则以本Plan为单一进度源。一个任务一行独立checkbox，依赖/owner/路径与oracle写在其下，当前全部未执行。

- [ ] T-01 冻结重放反例与会话接缝
  - owner_stage: work；依赖：1023/980/981已交来源及后续实施指令。核基面/唯一树/材料scope/并发写面，定位真实编译/初始化/handler观察口，创建V-01及最小V-06反例，最多两策略先证普通Node最长链。
  - 涉及：已核core/av/av-core与UI桥路径；产出单一策略、完整受影响支持表、资源/错误/回滚及序号合同。关联AC-01/06；先证不成立走needs_replan，不进入T-02。
- [ ] T-02 生成可保留的组件执行入口
  - owner_stage: work；依赖T-01。在core共同Node语义/lowering与现有唯一verify中生成初始化/单事件/状态读取接面，禁止按历史生成新驱动；拟新增case由本任务创建。
  - 关联AC-02/03；V-02/03普通源→受检编译，源码改名/业务表达式变化仍正确；不新增ASE解释器。
- [ ] T-03 实现AV增量事件会话
  - owner_stage: work；依赖T-02。复用公开代码/状态根执行机制，在av原owner增加拟公共事件会话模块，必要av-core最小接缝沿同一执行器；记录实际program/model/handler/lifetime事实。
  - 关联AC-02/03/05；V-02/03/05，热事件/读取/切换不重编和不重放，owned引用保活可查。
- [ ] T-04 闭合事件事务与关闭
  - owner_stage: work；依赖T-03。实现先验拒绝/有界去重、失败候选丢弃/合法恢复、关闭取消与最后owner释放，不用op-log恢复。
  - 关联AC-04/05；V-04/05，包括两个合法Inc和同seq重投、trap/资源/取消、陈旧输入与关开。
- [ ] T-05 接入UI和ASE真实消费链
  - owner_stage: work；依赖T-03/04。`ace_state_bridge`/`NodeDesktopBridge`接新公开会话；ASE sessions沿原路消费，状态差量和选中车道策略保留。冻结并验证当前合法类型/事件/包桥受影响面。
  - 关联AC-02..05/07；V-02..05/09，无旧观察链在线fallback、无Rust业务代算，原外壳不重做。
- [ ] T-06 固化执行层反例与负控
  - owner_stage: work；依赖T-05。沿真实adapter/executor接缝执行V-06..08，内部强制重放/重建但保持外层计数与结果不变；补getter重复业务、错版来源及UI差量证据。
  - 关联AC-06/07；负控红且正常绿，原1023负控保留为外部重装检查而非增量oracle。
- [ ] T-07 验证完整职责与原生原型
  - owner_stage: work；依赖T-06。执行V-01..10完整适用面与§6冻结职责/必要消费者，原3/7/同app/归零/限额/20轮和排窗链不减；健康/格式/告警与首次红/重放绿分账。
  - 关联AC-01..07；每个PASS绑定来源/观察口/真实终态，人工门未跑保持pending。
- [ ] T-08 整理更正与修复候选证据
  - owner_stage: work；依赖T-07。将新oracle逐条映射原1023 AC-01/02/V-07/08及M0-SESSION/AC-D待修子门，拟SD-01..04与债关闭候选，摘要折§9。
  - 关联AC-07；不在work阶段宣称已正式恢复M0，不修改1023旧收据/勾选/终态。
- [ ] T-09 独立复审增量会话
  - owner_stage: review；依赖T-08。独立上下文、同一树核源码/实际执行而非数字，重建所需支持表和原始反例/真实负控/失败资源，检查所有AC及未授权延后；AC-R。
  - 失败返work定向修复，不以原1023 review PASS免审；本任务未闭不阻work交review。
- [ ] T-10 落地规范并关闭修复债
  - owner_stage: merge；依赖独审pass。冻结受影响完整组/必要消费者并集，正式合入并核SD与运行来源，关闭D-ASE-1023-REPLAY并按实际证据恢复M0相关门；AC-D。
  - 1023仍archived且原收据保留。命令/退出码/关键计数/hash折本Plan，逐项对账所有输出目录后删scope，无具名hold；各树pwsh guard clean后移除，刷新retention登记，不运行无关性能/GUI全套。

## 9. 复审记录

stage: new；PLAN-1106 r1；outcome: pass（有界修复合同可交接，不是修复实现/独立review通过）；next: 后续取得实施指令后T-01实际原红与最长链先证。0/10，所有V-*、编译/测试/GUI、独立实现review/merge均未执行。本轮只以静态源码确认派发链与旧oracle边界，并纠正文档中已被该事实否定的结论。

1023 archived/r2/10-of-10与原交付SHA保留；其无重放/完整M0结论由2026-11-19更正说明限定，新闭合证据未来只由本Plan持有，不倒填旧执行记录。

本轮文档预检：10个独立checkbox任务/10个V用例/9个AC（含review/merge）及0–10章节映射、22处本地链接、依赖唯一身份与无新增回边、围栏/空白检查通过。1020/1021/1022归档及7个已核源码/测试文件内容hash未变；从1023剥除两块具名更正后，原文本hash及frontmatter与读前完全一致，证明只追加更正。静态检查不证明修复能力或新反例已运行；全部实现任务仍未勾选。

## 10. 待澄清事项

T-01负责选择实际可行的代码/状态根/单事件入口与事务观察方法，最多两策略；ProgramSnapshot全局cell与owned会话只是候选，不能因名称存在跳过普通Node整链。涉及共享UI在途改动先核owner/短写窗；若必要新语义无法保持当前支持或原验收，明确needs_replan及最小范围变更供决定，不默降为事件重放或仅返回数字。
