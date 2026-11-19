---
plan_id: PLAN-1106
status: archived
feature_name: ASE组件事件增量会话与历史重放验收修复
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 1
current_step: 10
total_steps: 10
depends_on: [PLAN-1023, PLAN-980, PLAN-981]
supersedes_spec_components: [docs/specs/ase/m0-interactive-sessions.md]
new_spec_components: [docs/specs/ace-migration/component-event-sessions.md]
touched_goals: [GOAL-002, GOAL-007, GOAL-010]
---

# [PLAN-1106] ASE组件事件增量会话与历史重放验收修复

## 0. 变更摘要

**已交付（2026-11-20）**：r1 全 10 任务完成（work 8 + review 1 + merge 1）；独立复审三轮 pass；两仓 ff-only 落地；D-ASE-1023-REPLAY closed。承接1023归档后的源码核查发现：对象与最终状态保留并不代表业务增量执行，`AceStateEngine::dispatch_impl→refresh`仍重编/重跑累计历史场景，外层`loads/dispatch_log`漏计内部执行。新计划实现真实程序/模型持有与新事件单次执行，在同一真实路径用执行层oracle修复原验收盲区。

[ASE路线](../../design/strategy/ase-m0-delivery-roadmap.md)当前把M0-SESSION的无重放子门恢复为pending（债`D-ASE-1023-REPLAY`）；保留1023 archived、原revision/勾选和历史收据，仅追加可见更正。可视交互原型及3/7行为证据继续有效，不宣称本修复已实现。

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

依据[Specs overview](../../specs/overview.md)、[ASE交互现状](../../specs/ase/m0-interactive-sessions.md)、[970会话边界](../../specs/ace-migration/m2-sessions.md)、[UI组件状态与宿主规范](../../specs/ace-migration/m2-autoui.md)、[源码重放规范](../../specs/auto-test-case/ace-source-only-replay.md)及[1023归档](1023-ase-m0-interactive-sessions-and-acceptance.md)。此次canonical更正是用户授权的已观察事实纠正，不提前沉淀未来增量支持。

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

- [x] T-01 冻结重放反例与会话接缝
  - owner_stage: work；依赖：1023/980/981已交来源及后续实施指令。核基面/唯一树/材料scope/并发写面，定位真实编译/初始化/handler观察口，创建V-01及最小V-06反例，最多两策略先证普通Node最长链。
  - 涉及：已核core/av/av-core与UI桥路径；产出单一策略、完整受影响支持表、资源/错误/序号合同。关联AC-01/06；先证不成立走needs_replan，不进入T-02。
  - **[✅ 2026-11-19 已完成]** 基面：7个源文件SHA全核对一致（lang 3 + ui 4，ui 实路径
    `crates/auto-ui-runtime/src|tests/`）；跨仓组 `D:/autostack/.wt/lang-1106/{auto-lang,auto-ui}`
    分支 `plan-1106-dev`（AGENTS.md 规范名；计划原文 `codex/plan-1106-dev` 为起草方拟名，
    同一交付、无范围变化）；基 lang=20231c041 / ui=481a3cb5；主检出 WIP 仅 docs/plans 簿记
    （1107 并行会话），无代码路径 WIP；材料 scope `D:/autostack/plan-artifacts/plan-1106/`
    已登记（t01-probe-evidence.md）。
  - **先证（探针 `av/tests/cases/plan1106_t01_probe.rs`，sha16=f815432027a9cb67
    [初审时为 50b5919209179271，T-07 rustfmt 提交后文件重排——复审 F-4 更正]，
    `cargo test -p ace-av --test conformance plan1106` 3/3 绿）**：
    P1 候选①（选定）——手装配 store-cell 增量脱糖经真实 AGE 前门
    （`compile_program_parts_age`+`verify_text_record_age` 接纳 `store`+`module.global.get/set`
    形态）→`lower_checked_owned_with_init`（globals 进 MIR）→split→admit→`link_with_globals`→
    新公共接缝 `ace_av_core::execute_program_entry_at`（def-id 定向派发+实参，init 链每代一次）：
    每事件恰一次派发（进程 `execution_seq` 增量==事件数）、载荷绑定 add(5)→7、seed 覆盖、
    双快照独立 cell（3/7 隔离最小形）、reset、未链函数具名 Infra 拒。
    P2 候选②——per-def `execute_entry` 实参穿线机械可行，但状态根落宿主 Rust 值
    （所有权对齐弱：字符串/资源不经 arena、读取绕过执行器），**弃**。
    P3 V-01 事实——当前场景车道 def 集只有状态函数（无任何 handler 入口 def）、
    单次状态函数执行返回整段历史终值（3 Inc→3、5 Inc→5：单次调用工作量随历史线性）、
    per-def `execute` 不进 `EXECUTION_SEQ`（旧车道无入口计数端口）、每次观察=全新编译。
  - **单一策略（冻结）**：候选①——编译器生成的实例状态/事件入口由 ProgramSnapshot
    全局 cell 与 init 纪律承接；读取=`global_cells()` 纯观察（零派发）；释放=drop 快照。
    观察口合同：进程 `execution_seq`（快照车道入口计数）+ 会话模块在生产者/执行器真实
    调用边界计 compile/link/init/handler-entry/release（V-06..08 负控的 oracle 位置）。
  - **受影响支持表（冻结，T-05 逐项核当前树）**：桥面类型/枚举（AceStateValue
    Int/Bool/Str/F32、DispatchOutcome/DropReason、UnmountOutcome、AsyncTicket/Outcome/SpawnError）、
    AceStateEngine 公共方法面（new 试验编译/mount keyed+世代/unmount/dispatch[_with_args]/
    read_state/read_all/generation_of/spawn_async/await_async/MAX_INSTANCES=4）、
    NodeDesktopBridge（load/load_package/open/reopen/close_session/step/inject_text/states/
    dispatch_log/install_script/evidence）、ase_m0_sessions（open/select/close/click/
    step_selected/engine_states/dispatches_of/AseMetrics）、载荷域（声明拼写定类型：
    int/bool/str/f32；unit=Send；typed=SendArgs；owned-str 形参 Move）、异步完成
    （spawn 钉 identity+generation；await 先跑 scheduler 再世代纪律回写）。
    兼容 oracle=现有 `tests/ase_m0_sessions.rs` 全套 + `plan980_state_face.rs` 等真语料。
  - **资源/错误/回滚合同（冻结）**：handler 体=locals 先行（hook_stmts_lower 同一脱糖）、
    末尾统一 cell 提交（业务中途 trap ⇒ 零提交，旧态保留）；预算耗尽跨提交边界的残余
    半提交由会话层 cell 备份/恢复兜底（T-03 加 `restore` 最小接缝）；未知/已释放/错代在
    效果前拒（Dropped 语义不变）；未声明消息/arity/类型错在 ACE 面具名拒且不入队。
    序号合同（T-04 实装）：宿主生成 session/generation/sequence，有界水位去重，
    同成功事件重投不重复执行（返回原结果或具名 duplicate），同名合法新事件不去重。
- [x] T-02 生成可保留的组件执行入口
  - owner_stage: work；依赖T-01。在core共同Node语义/lowering与现有唯一verify中生成初始化/单事件/状态读取接面，禁止按历史生成新驱动；拟新增case由本任务创建。
  - 关联AC-02/03；V-02/03普通源→受检编译，源码改名/业务表达式变化仍正确；不新增ASE解释器。
  - **[✅ 2026-11-19 已完成]** `core/source/age_component.rs`：
    `compile_component_event_program(src, path, alias, caps, node_index, emit_texts)`
    ——与场景车道同一形状门/字段门/重复门/绑定检查/body_foreign与通道调用
    fail-closed 边界，脱糖产物为 store cell（`__age_ev_g_<field>`）+
    `__age_ev_init()`（声明初值）+ `__age_ev_seed(<字段形参>)`（props 覆盖）+
    每 handler `__age_ev_on_<msg>(<载荷形参>)`（**locals 先行→hook_stmts_lower
    同一脱糖→末尾统一 cell 提交**：业务中途 trap ⇒ 零提交）+
    文本叶 `__age_ev_text_<ti>()`（`emit_texts`，纯读 cell；f-string 源走
    states-only 重试留痕）。`FieldBinding::GlobalCell` 新绑定（字段引用→
    `module.global.get:` 调用）；`__age_ev_` 保留名族补入。str 载荷/种子按
    **借用 str 形参 + 一次 `.clone()` owned 局部**铸造（执行器入口准入只收
    StrView）。经同一 `compile_program_parts_age`+`verify_text_record_age`。
- [x] T-03 实现AV增量事件会话
  - owner_stage: work；依赖T-02。复用公开代码/状态根执行机制，在av原owner增加拟公共事件会话模块，必要av-core最小接缝沿同一执行器；记录实际program/model/handler/lifetime事实。
  - 关联AC-02/03/05；V-02/03/05，热事件/读取/切换不重编和不重放，owned引用保活可查。
  - **[✅ 2026-11-19 已完成]** av-core 最小接缝：`execute_program_entry_at/_args`
    （def-id 定向派发+实参；init 链每代一次；未链函数具名 Infra 拒）、
    `EntryArg`（标量/文本 Cow→本执行 arena StrView）、
    `ProgramSnapshot::restore_global_cells`（事务回滚）。av 新公共模块
    `component_event_sessions.rs`：`ComponentEventSessionProgram`（一次编译
    [带文本→states-only 重试留痕]、共享 `Arc<FunctionCode>`、link 计数）/
    `ComponentEventSession`（每会话独立 cell 状态根、`dispatch_seq`、
    `read_states` 纯观察、`read_texts` 纯读、`close`/Drop 释放）/
    `SessionAudit`（handler_entries、entry_seqs=执行器进程序号、init_runs、
    links、rejected、duplicate_ignored、released——全部真实调用边界计量）。
    生命周期事实：`StrHandle` 自持 arena Rc（cell 跨派发持 str 合法）。
- [x] T-04 闭合事件事务与关闭
  - owner_stage: work；依赖T-03。实现先验拒绝/有界去重、失败候选丢弃/合法恢复、关闭取消与最后owner释放，不用op-log恢复。
  - 关联AC-04/05；V-04/05，包括两个合法Inc和同seq重投、trap/资源/取消、陈旧输入与关开。
  - **[✅ 2026-11-19 已完成]** 双层事务：编译期 locals 先行/统一提交（业务
    中途 trap 零提交）+ 会话层派发前 `global_cells()` 备份/失败
    `restore_global_cells`（预算边界残余半提交兜底）。先验拒绝：未声明消息/
    载荷 arity/类型错（错误码保持旧合同 `source.age.component-event`/
    `source.type-mismatch`）。序号水位：`dispatch_seq(Some(seq))` ≤ 水位 =
    重投不重复执行（返回 Duplicate + 上次已提交状态）；同名合法新事件各有
    新 seq 正常执行。失败后合法事件直接处理（不回放恢复）。关闭：
    `close()`/Drop 标记 released 并释放状态根；关闭后事件/读取具名拒。
- [x] T-05 接入UI和ASE真实消费链
  - owner_stage: work；依赖T-03/04。`ace_state_bridge`/`NodeDesktopBridge`接新公开会话；ASE sessions沿原路消费，状态差量和选中车道策略保留。冻结并验证当前合法类型/事件/包桥受影响面。
  - 关联AC-02..05/07；V-02..05/09，无旧观察链在线fallback、无Rust业务代算，原外壳不重做。
  - **[✅ 2026-11-19 已完成]** `ace_state_bridge.rs` 引擎内部重接：一次编译
    `ComponentEventSessionProgram`，每挂载身份一个会话（mount=link+init+seed，
    unmount=close 释放），dispatch=恰一次 handler 派发，read=零派发，
    spawn_async/await_async 沿原世代纪律；公共 API/枚举/错误合同全保持。
    `SyncSession` 单线程纪律包装（宿主静态 RwLock 串行化；HeldLibrary 先例
    同族）。`ace_node_desktop.rs`：文本叶刷新改 `engine.read_texts`（纯读），
    场景 op 历史累积退役；装载期观察/包桥（load_package）/ace_native 不变。
    受影响面复验全绿：ase_m0_sessions 9/9（含窗口 e2e resize/focus）、
    plan041_instance_ownership 10/10、ace_node_desktop 11/11、ase_m0_text 7/7、
    desktop_behavior 11/11（ace_ui_hello 7/7，其窗口 exe 预构建为 prep 缺失
    补跑后绿——非代码回归）。组仓补齐：`.wt/lang-1106/{auto-web,auto-down}`
    真实 detached worktree（路径依赖字面解析；无链接）。
- [x] T-06 固化执行层反例与负控
  - owner_stage: work；依赖T-05。沿真实adapter/executor接缝执行V-06..08，内部强制重放/重建但保持外层计数与结果不变；补getter重复业务、错版来源及UI差量证据。
  - 关联AC-06/07；负控红且正常绿，原1023负控保留为外部重装检查而非增量oracle。
  - **[✅ 2026-11-19 已完成]** 执行层 oracle `EngineExecutionAudit`
    （events_delivered/handler_entries/links/compile_count/replay_compiles/
    rebuild_compiles + 增量不变量 events==handlers），真实调用边界计量。
    三个负控旋钮（同 owner/同计数器、真实 producer/执行器调用、生产路径
    恒关）：V-06 重放道（`observe_component_scenario_states` 真调用，开启时
    以当前状态为场景基线——外层数字一致）；V-07 重建道（每事件真实
    recompile+relink+seed 补数）；V-08 getter 派发道（读取触发真实 handler
    派发）。永久用例 `auto-ui tests/component_event_sessions_1106.rs` 4/4 绿
    （正常道绿 + 各负控红被 oracle 抓获 + 恢复绿；close/reopen 正控不误杀）。
    旋钮关闭时 delivered 重定基到真实入口数（诊断注入不计生产账）。
- [x] T-07 验证完整职责与原生原型
  - owner_stage: work；依赖T-06。执行V-01..10完整适用面与§6冻结职责/必要消费者，原3/7/同app/归零/限额/20轮和排窗链不减；健康/格式/告警与首次红/重放绿分账。
  - 关联AC-01..07；每个PASS绑定来源/观察口/真实终态，人工门未跑保持pending。
  - **[✅ 2026-11-19 已完成]** V-01（探针 P3 事实基线 + 既有车道行为保留）、
    V-02/03/04/05（av `component_event_sessions_1106` 6/6：0/1/3/7/32 线性入口、
    3→0→1、A×3/B×7、四型+种子、str 载荷、trap 回滚零半态、序号水位、
    20 轮关开 init 恰一次/compile 恒 1）、V-06..08（UI 负控 4/4）、
    V-09（ase_m0_sessions 9/9 含真实窗口 e2e resize/focus；ase-m0 crate 7/7）。
    V-10 收据适用性见 T-08 映射（旧 1023 PASS 只作行为参考）。
    冻结职责组：`cargo t ace-source` **全组 PASS**；`cargo t ace-av` 1426 例
    1424 过、2 红（`host_console::av_console_entry_selection_cannot_bypass_main_policy`
    / `av_console_owned_front_door_enforces_console_entry_only`）——**主检出
    master HEAD 同命令同样 2 红**（预存红，与本计划改面无关，留痕不判 PASS）；
    `ase-m0` 组因 nextest-full.toml 的 `package(ace-aac-support-conformance)`
    override 对 ase-m0 工作区失配而无法装载（**主检出同样失败**——预存基建
    问题，PLAN-1056 配置面），按默认发现配置实跑 **7/7 绿**。
    健康：rustfmt 应用于全部改面（未触及文件的 fmt 漂移已还原）。**新增告警声明
    更正（复审 F-1）**：初审 T-07 曾记"新增告警零"，独立复审查出 5 个新告警
    （未用导入/失读字段/多余 mut）；修复提交 lang 404745197 / ui 489d3889 清零后，
    `cargo check` 改面文件零新告警（残余 `BridgeEngine::mount never used` 为基线
    预存）。
    首次红/重放绿分账：探针 P1 初版 dispatched 断言误解（观察口语义）与
    str 载荷实参准入发现（owned Str 非法入口实参→改借用+clone）均为开发
    期修正记录，最终绿证据见上述各文件。
- [x] T-08 整理更正与修复候选证据
  - owner_stage: work；依赖T-07。将新oracle逐条映射原1023 AC-01/02/V-07/08及M0-SESSION/AC-D待修子门，拟SD-01..04与债关闭候选，摘要折§9。
  - 关联AC-07；不在work阶段宣称已正式恢复M0，不修改1023旧收据/勾选/终态。
  - **[✅ 2026-11-19 已完成]** 映射（新 oracle → 1023 待修子门）：
    | 1023 待修面 | 修复证据（本 Plan 持有） | 状态 |
    |---|---|---|
    | AC-01 装载/真实执行解释 | 固定源 compile_count==1、link/会话/init 计数（av v02/v05 + SessionAudit） | 证据就绪 |
    | AC-02 无重放单事件执行 | 每事件恰一次 handler 入口（entry_seqs=执行器序号）、读取/切换零派发（av v02 + UI v02_bridge） | 证据就绪 |
    | V-07/08 旧负控盲区 | 旧负控保留为外部重装检查（p1023_v07/v08 绿）＋新执行层负控 V-06..08（同 owner 强制重放/重建/getter 派发必红，4/4） | oracle 已升级 |
    | M0-SESSION 无重放子门 | 上行全链（含真实窗口 e2e）+执行层审计 | 修复证据就绪，正式恢复归 AC-D |
    | AC-D 状态根释放 | close/Drop released、20 轮 links 线性/init 恰一次、关后事件/读取具名拒 | 证据就绪 |
    SD 候选：SD-01 新增 `docs/specs/ace-migration/component-event-sessions.md`
    （编译入口/会话生命周期/事务/序号/审计/states-only 与 body_foreign/
    通道调用边界/包面不涉）；SD-02 修改 `docs/specs/ase/m0-interactive-sessions.md`
    （无重放 pending→按本证据恢复，历史保留）；SD-03 修改 auto-ui
    `docs/specs/runtime/ase-m0-frame-consumption.md`（消费真实增量会话与
    执行/释放证据）；SD-04 修改 roadmap + KNOWN-DEBT
    （D-ASE-1023-REPLAY OPEN→closed 条件核对）。**正式落稿归 merge（T-10）**，
    本任务不写 canonical、不改 1023 旧收据/终态。
- [x] T-09 独立复审增量会话
  - owner_stage: review；依赖T-08。独立上下文、同一树核源码/实际执行而非数字，重建所需支持表和原始反例/真实负控/失败资源，检查所有AC及未授权延后；AC-R。
  - 失败返work定向修复，不以原1023 review PASS免审；本任务未闭不阻work交review。
  - **[✅ 2026-11-19/20 已完成（三轮）]** 独立上下文 agent（同权威树）全审：
    AC-01..AC-07 全 pass、生产路径无重放残留、负控全走真实 producer；needs_fix
    ×2（F-1 五个新告警→修复关闭；F-2 折入账未落地→循环 2 补全关闭，含
    v02_bridge 关开全程不变量永久闭合与 v07 oracle 对齐计划定义）；最终
    **pass**（@ lang 404745197 / ui 471013b1）。复审报告与关闭表折 §9。
- [x] T-10 落地规范并关闭修复债
  - owner_stage: merge；依赖独审pass。冻结受影响完整组/必要消费者并集，正式合入并核SD与运行来源，关闭D-ASE-1023-REPLAY并按实际证据恢复M0相关门；AC-D。
  - 1023仍archived且原收据保留。命令/退出码/关键计数/hash折本Plan，逐项对账所有输出目录后删scope，无具名hold；各树pwsh guard clean后移除，刷新retention登记，不运行无关性能/GUI全套。
  - **[✅ 2026-11-20 已完成]** SD-01..04 落地（worktree 内规范提交后随分支合入）；
    冻结组门（merge 所有，最终基线）：`cargo t ace-source` 全组 PASS、
    `cargo t ace-av` 1441 例 1439 过/2 预存红（host_console 对，主检出同红留痕）、
    23 skipped；UI 主检出落地检查 4/4＋9/9＋10/10。**rebase 适配**：master 并行
    前移（PLAN-1080 给 `ast::FnItem` 加 `proto`）→ 事件程序三处构造/探针补
    `proto:false`（098b8533d），range-diff 全等价后落地；另修并行落地的
    `auto-lang-ace-host::render_type` 非穷尽（`TypeRef::Inferred => "_"`，
    fix worktree 64e518f52——非本计划 diff 面，L0 规程独立合入）。
    D-ASE-1023-REPLAY **closed**；roadmap M0-SESSION/M0-CLOSE 恢复；
    KNOWN-DEBT 更新。1023 保持 archived/原收据。

## 9. 复审记录

stage: merge | plan_id: PLAN-1106:r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1106/{auto-lang,auto-ui}（+auto-web/auto-down detached 兄弟）| delivery_commit: lang 64e518f52（= 1106 分支 098b8533d 落地后含并行缺口 L0 修复；1106 自身交付线：72ab54c7a→…rebase 映射→7ae978347→098b8533d，range-diff 全等价）/ ui 70f06862 | checkpoints: prepared=复审 pass 基线+冻结 Spec 差异；landed=两仓 --ff-only（rebase 后），主检出 tip=交付线；canonical_verified=SD-01 新增 docs/specs/ace-migration/component-event-sessions.md、SD-02 docs/specs/ase/m0-interactive-sessions.md、SD-04a docs/design/strategy/ase-m0-delivery-roadmap.md（lang 7ae978347）、SD-03 auto-ui docs/specs/runtime/ase-m0-frame-consumption.md（70f06862；起草更正 WIP 由分支版本收编）；archived=git mv 至 archive/＋status 翻转＋本收据（docs 提交）；cleaned=wt-guard clean 后移除两 plan worktree/分支/组目录（web/down 兄弟 detached 同组移除）、plan-artifacts/plan-1106 整目录删除、retention 登记刷新 | merge门: 冻结组 ace-source 全组 PASS／ace-av 1439/1441（2 预存 host_console 红，主检出同现留痕）／UI 主检出 4/4+9/9+10/10；批回归未到期（收据 2026-11-20、1106%5=1） | 环境修复（非本计划 diff 面）: auto-lang-ace-host render_type 补 Inferred 臂（fix worktree 64e518f52，PLAN-1080 并行落地整合缺口） | evidence: 见 §8 各任务行与 §9 测试账目；原始日志已折摘要，scope 删除

stage: work | plan_id: PLAN-1106 | plan_revision: r1 | round: 2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1106/auto-lang（+ 组内 auto-ui/auto-web/auto-down 兄弟 detached 树）| branch: plan-1106-dev | base_commit: lang 20231c041 / ui 481a3cb5 | code_commit: lang 929a6c743（72ab54c7a→0d0f530d2→929a6c743）/ ui 1e2af0f5（bbffc0ad→1e2af0f5）| dependency_revisions: auto-web@26d085f, auto-down@72b0970（detached）| task_ids: T-01..T-08（work 阶段全部 8/8）| evidence: 见 §8 各任务 [✅] 行与下方测试账目 | blockers: 无 | next_responsible_role: review（T-09 独立复审）

### work 阶段测试账目（2026-11-19，全部同机同日； warmed debug/dev profile）

| 测试模块/实际范围 | 结果与执行/排除数 | 准备/构建 | 发现 | 运行 | 总时长 |
|---|---|---|---|---|---|
| av conformance `component_event_sessions_1106`（新，V-02..05 av 侧） | 6/6 过 | 复用既有构建（warm） | — | 0.07s | 0.07s |
| av conformance `plan1106_t01_probe`（新，T-01 先证+V-01 基线） | 3/3 过 | 同上 | — | 0.03s | 0.03s |
| UI `component_event_sessions_1106`（新，V-06..08+v02 桥面） | 4/4 过 | 同上 | — | 0.61s | 0.61s |
| UI `ase_m0_sessions`（既有完整目标，V-09） | 9/9 过（含真实窗口 e2e） | 同上 | — | 4.98s | 4.98s |
| UI `plan041_instance_ownership` | 10/10 过 | 同上 | — | 0.33s | 0.33s |
| UI `ace_node_desktop` | 11/11 过 | 同上 | — | 1.49s | 1.49s |
| UI `ase_m0_text` | 7/7 过 | 同上 | — | 1.89s | 1.89s |
| UI `desktop_behavior` | 11/11 过、1 ignored（原条件） | 同上 | — | 0.27s | 0.27s |
| UI `ace_ui_hello` | 7/7 过 | prep：example exe 预构建 43.47s（缺该 prep 首跑红为预存环境条件） | — | 3.14s | 3.14s |
| ase-m0 crate（`cargo t ase-m0` 等价默认配置实跑） | 7/7 过 | 复用 | — | 0.385s | 0.385s |
| `cargo t ace-source`（冻结组） | 全组 PASS | prepare 7.2s（manifest） | — | 组运行 | 见组输出 |
| `cargo t ace-av`（冻结组） | 1426 例：1424 过、2 红（主检出同红=预存，留痕）、23 skipped（原条件） | 同上 | — | 35.46s | 35.46s |

新用例计时（≤10s 门）：av 6 例合计 0.07s（均 <0.05s/例）；UI 4 例合计 0.61s
（最大单例 v06 约 0.3s）；探针 3 例合计 0.03s。无新用例超 10s。

预存问题留痕（非本计划改面所致，主检出同现）：①`cargo t ase-m0` 的
nextest-full.toml `package(ace-aac-support-conformance)` override 对 ase-m0
工作区失配（PLAN-1056 配置面）；②ace-av 组 host_console 两例红
（`Option::unwrap` on None）；③ace_ui_hello 窗口 exe prep 缺失（补 prep 后绿）。
以上建议另立轻量修复任务（fix- 前缀 worktree），不阻塞本 Plan 交付面。

stage: review | review_scope: final（循环 2，仅核 F-2）| reviewer_identity: 同一独立上下文 agent（三轮连续：全审→循环 1→循环 2）| plan_id: PLAN-1106 | plan_revision: r1 | outcome: **pass** | worktree_path: D:/autostack/.wt/lang-1106/{auto-lang,auto-ui} | branch: plan-1106-dev | reviewed_commit: lang 404745197 / ui 471013b1 | base_commit: lang 20231c041 / ui 481a3cb5 | dependency_revisions: auto-web@26d085f, auto-down@72b0970 | spec_inputs: SD-01..04 候选 | acceptance_results: AC-01..AC-07 全 pass + AC-R 完成（循环 0 全表证据；循环 1 关闭 F-1/3/4/5/6；循环 2 关闭 F-2：折入账单一累计口径、关开全程不变量有永久测试闭合、v07 红信号对齐计划 V-07 oracle 定义且不弱化 V-06/V-08 检测）| pending_closeout_gates: merge 冻结并集复核（ace-source 全组 PASS、ace-av 1424/1426[2 预存红]、ase-m0 7/7[默认配置]——基线 929a6c743/1e2af0f5 上跑；修复提交只动审计簿记与测试断言，不影响组面）| findings: 无遗留 | evidence: 循环 2 报告（F-2 关闭表 + 4/4 复跑 0.05s + 零新告警）折本节 | next: merge（T-10）

stage: work | plan_id: PLAN-1106 | plan_revision: r1 | round: 5 | outcome: repair-done | worktree: 同上 | commit: ui 471013b1（lang 无新提交）| findings_fixed: **F-2 补全（循环 1 复审判未闭：round-4 只加字段未改求和，且收据表述超前于代码——本收据更正）**：execution_audit 的 handler_entries 现为 存活+retired 折入真和（current_real_handler_entries 单一累计口径，两旋钮关闭重定基同用）；v02_bridge 补关开全程不变量断言（unmount 后 delivered==live+retired，复审反例的永久闭合）；v07 红信号改为 prepare/init 证据（rebuild_compiles/links——模拟道为拿正确终值真实派发一次，不变量不是 V-07 的 oracle，注释具名） | repair_checks: UI 负控 4/4（0.05s）、ase_m0_sessions 9/9（2.16s，含窗口 e2e）；UI 树清洁 | next: review 复审（循环 2/3，仅核 F-2 关闭）

stage: work | plan_id: PLAN-1106 | plan_revision: r1 | round: 4 | outcome: repair-done | worktree: 同上 | commit: lang 404745197 / ui 489d3889 | findings_fixed: F-1(5 告警清零：ScenarioOp 导入/src 字段/mut/alias/seed_def_id)、F-3(initials 字段移除)、F-4(计划 hash 更正 f815432027a9cb67)、F-5(conformance.rs 注册序还原)、F-6(错误串伪影/死代码清理)；F-2 仅加 retired 字段与折叠写入（**求和未落地——循环 1 复审抓出，见 round-5**） | repair_checks: av 会话 6/6+探针 3/3、UI 负控 4/4、plan041 10/10、ase_m0_sessions 9/9（修复后复跑全绿；改面 cargo check 零新告警）| next: review 复审（循环 1/3）

stage: review | review_scope: final | reviewer_identity: 独立上下文 general-purpose agent（同权威树；实现会话仅转述，未参与验证）| plan_id: PLAN-1106 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1106/{auto-lang,auto-ui} | branch: plan-1106-dev | reviewed_commit: lang 929a6c743 / ui 1e2af0f5 | base_commit: lang 20231c041 / ui 481a3cb5 | dependency_revisions: auto-web@26d085f, auto-down@72b0970 | spec_inputs: SD-01..04 候选（plan §5 规范增量表）| acceptance_results: AC-01..AC-07 全 pass（含新跑：probe 3/3、av 会话 6/6、UI 负控 4/4、ase_m0_sessions 9/9 含窗口 e2e、plan041 10/10、ace_node_desktop 11/11、ase_m0_text 7/7；生产路径无重放残留、负控全走真实 producer） | pending_closeout_gates: merge 冻结并集复核 | findings: F-1(major) 5 个新编译告警（ui: ScenarioOp 未用导入/src 字段失读/mount 处多余 mut；lang: 会话程序 alias 字段失读、会话 seed_def_id 失读）——T-07"新增告警零"声明不实；F-2(minor) unmount 后增量不变量因 live-sum 口径可被健康道伪证（域未文档化）；F-3(note) AceStateSlot.initials 只写；F-4(note) T-01 探针 hash 随 fmt 提交漂移未更新；F-5(note) core/tests/conformance.rs 模块注册序被动 fmt 重排未列；F-6(note) arity 错误串宽空白/v04 死代码 | evidence: 审计报告全文折本节（上方），命令/退出码/计时见其表 | next: work 定向修复 F-1..F-6 后复审（修复循环 1/3）

stage: goal | plan_id: PLAN-1106 | plan_revision: r1 | round: 3 | from: review | to: work | status: executing | outcome: needs_fix | worktree: 同上 | commit: lang 929a6c743 / ui 1e2af0f5 | next: work

stage: goal | plan_id: PLAN-1106 | plan_revision: r1 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1106/auto-lang + auto-ui | commit: lang 929a6c743 / ui 1e2af0f5 | next: review

stage: new；PLAN-1106 r1；outcome: pass（有界修复合同可交接，不是修复实现/独立review通过）；next: 后续取得实施指令后T-01实际原红与最长链先证。0/10，所有V-*、编译/测试/GUI、独立实现review/merge均未执行。本轮只以静态源码确认派发链与旧oracle边界，并纠正文档中已被该事实否定的结论。

1023 archived/r2/10-of-10与原交付SHA保留；其无重放/完整M0结论由2026-11-19更正说明限定，新闭合证据未来只由本Plan持有，不倒填旧执行记录。

本轮文档预检：10个独立checkbox任务/10个V用例/9个AC（含review/merge）及0–10章节映射、22处本地链接、依赖唯一身份与无新增回边、围栏/空白检查通过。1020/1021/1022归档及7个已核源码/测试文件内容hash未变；从1023剥除两块具名更正后，原文本hash及frontmatter与读前完全一致，证明只追加更正。静态检查不证明修复能力或新反例已运行；全部实现任务仍未勾选。

stage: goal | plan_id: PLAN-1106 | plan_revision: r1 | round: 1 | from: archived-verify | to: done | status: archived | outcome: pass | worktree: 无（lang-1106 组已移除） | commit: 6708a123e（归档链接补正，文档-only）/ 交付 64e518f52（lang）＋70f06862（ui） | next: done

- **归档态核验（2026-11-20）**：master tip＝交付线+归档 docs；`git log -1 -- docs/plans/archive/1106-*` 命中 94ac2d062（归档提交）；SD-01 `docs/specs/ace-migration/component-event-sessions.md`、SD-02（[ASE交互现状](../../specs/ase/m0-interactive-sessions.md) 头部 2026-11-20 修复段）、SD-04a `docs/design/strategy/ase-m0-delivery-roadmap.md`（M0-SESSION/M0-CLOSE 已恢复）、SD-03 auto-ui `docs/specs/runtime/ase-m0-frame-consumption.md`（70f06862）均在落地提交内；`D-ASE-1023-REPLAY` 记 **CLOSED**；auto-ui 70f06862 为 master 祖先（`git merge-base --is-ancestor` 退出码 0）；分支 `plan-1106-dev` 已删，`D:/autostack/.wt/lang-1106` 与 `D:/autostack/plan-artifacts/plan-1106` 均 `Test-Path=False`。
- **归档门补正（文档-only，未重开实现/验收）**：`git mv` 入 archive/ 后未同步的入链共 10 处（本文件 7、m0 spec 1、roadmap 2、00-intro 1）从 `docs/plans/archive/` 解析失败（如 `archive/1023-*`→`archive/archive/1023-*`、`../specs/*`→`archive/../specs/*`）。已按 merge 归档门 “links resolve from the real parent” 改为 `../../design|specs`、`1023-*`、`plans/archive/1106-*`，并更正 00-intro 过期“规划/pending”为已交付已闭。提交 6708a123e；复核本次扫描 1106 相关断链 **0**（全表 170→160，余 160 为 KNOWN-DEBT 等既有历史断链，非本计划改面）。
- **未到期门**：批回归收据 last_covered 1098（2026-11-20）→ 1106%5=1 且 <48h，非 due；本会话仅 docs 补正，未触发新一次 `cargo t all`。主检出 WIP 仅并行会话（1107/1081 规划）docs 簿记，未纳入本会话提交（00-intro 仅暂存本会话 hunk）。

## 10. 待澄清事项

T-01负责选择实际可行的代码/状态根/单事件入口与事务观察方法，最多两策略；ProgramSnapshot全局cell与owned会话只是候选，不能因名称存在跳过普通Node整链。涉及共享UI在途改动先核owner/短写窗；若必要新语义无法保持当前支持或原验收，明确needs_replan及最小范围变更供决定，不默降为事件重放或仅返回数字。
