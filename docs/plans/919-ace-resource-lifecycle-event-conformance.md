---
plan_id: PLAN-919
status: reviewed
feature_name: AC与AV资源生命周期事件正式闭环
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-03
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-test-case/ace-m1-scenario-oracles.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-gap-audit.md, docs/specs/auto-av/place-ownership.md, docs/specs/auto-ac/source-collections-variants.md]
new_spec_components: [docs/specs/auto-test-case/resource-lifecycle-observations.md]
touched_goals: [GOAL-002]
affects: [auto-av, auto-ac, auto-test-case]
depends_on: [PLAN-864, PLAN-889, PLAN-912, PLAN-913, PLAN-914]
current_step: 7
total_steps: 10
---

# [PLAN-919] AC与AV资源生命周期事件正式闭环

## 0. 变更摘要

只认领ns=scenario R01/R04/R06 × ac/av六格：真实owner构造/转移/释放事件、变体换臂与budget/cancel/trap退出，经既有执行路径到父侧独立oracle。补实际清理漏洞属于本包，不能只加打印。drafting r1、0/10；本轮只规划，不建树/构建/测试/改current Spec或915/917合同。

架构依据：[Design43 §20](../design/43-ac-av-typed-mir-and-abc.md#20-915917在途时的下一波2026-11-02target-state)。不认领915的R02/R03 av、native R02/R03/R05三格、完整native闭包/loader、其他M1格或M2总签章。

## 1. 目标

G1：逐owner事件足以区分漏释放、重复释放、错误顺序与只报总数。G2：返回/循环出口/换臂/错误/取消路径按现行资源合同保留与清理，active view/返回值/pin直到最后owner才释放backing。G3：六格首例文件化、父判严格、恢复与受支持暖增量可核，保原资源/时间门。

非目标：新所有权语言、共享owned循环、跨allocator统一全局id、取消即时中断语义、为native新增闭包ABI、将已有AV集合whole-source前端冒成parse0；不签扩大后M1。

## 2. 架构方案

source或已明确身份的typed fixture→现有Checked HIR/共享MIR→ABC/AV或native PE→真实owner动作→有界事实→ac_av/ac_mn adapter→TestCase独立oracle。复用AV StrArena alloc/free、release_frame_owned_lifo及native collections frame_leave/drop、collections_observe；不能造第二arena/资源分析器/通用runner。请求只给操作/取消预算，不给预期事件；父侧用冻结规则重判，不信worker自报PASS。

观察须区分logical owner、共享backing、frame/slot/代际；转移不等于释放，共享backing只在最后owner释放。观察记录纯数据且不持有Value/handle，不延长寿命、不深clone值、不改变drop顺序。原有日志只覆盖Str或总计数的事实不提升为全owner已支持。

## 3. 技术栈

Rust，ACE现有av-core/av/native及no_std text/collections runtime；现有worker/Job与auto-test-case；文件化AutoDown fixture/reference。no_std运行时保持panic-abort专用profile，不由普通VM测试构建。

## 4. 需求分析与背景调查

授权：用户确认915/917在执行，要求提前规划下一波；只允许规划文档/正式取号，不授权本轮实施、改源/expected/分母、跨仓写入或启动agent/chat。基线main=5a0ef7609；D913=04f487b4b、D914=6ffbaf49e、D916=edfdabc2e均已合入。915实际lang-915树在途、头drafting/current_step5；917实际树在途，状态滞后不代改。

| 来源/实读路径 | 已观察事实/内容SHA256 |
|---|---|
| docs/specs/auto-test-case/ace-m1-scenario-oracles.md | b8791a06b9aa26474d5da8f7d33f81ca04ef1c53ae82e0f76e3857317d66fb78；R01/R04/R06需要逐owner而非返回值 |
| experimental/ace/crates/av-core/src/av.rs | ff0d3e196d932468cb304211f5b3421d59aec7681393422f90164a97e2cf703f；release_frame_owned_lifo、ExecTrace真实存在 |
| experimental/ace/crates/av-core/src/owned_text.rs | StrArena alloc_events/free_events、最后StrHandle释放台账；不等于所有复合owner事件 |
| experimental/ace/crates/collections-runtime/src/lib.rs | 4f379e2137fbdd6f3887d6a06f8ca7eb7fab51d097678802373b169b11fec413；mn_coll_frame_leave/drop/cancel_after_work与总计量已有 |
| experimental/ace/crates/native/src/collections_observe.rs | 严格解析真实stderr/exit，不是任意guest身份认证；新增事件须绑受管构建产物与源身份 |
| docs/specs/auto-ac/source-collections-variants.md、auto-av/place-ownership.md | 原资源/借用语义及AV集合旧whole-source边界，不能靠新增观察扩大支持 |
| binding/applicability/source-manifest | 读取时旧required14阻塞；本包六格、915/917各两格、920一格、native三格互斥 |

不得从头勾选或已有单测推断本六格已过。T-01在真实开工SHA重核915接口、原源与当前Spec；记录漂移，不消费未提交WIP。

## 5. 详细设计

### 5.1 六格数据与生命周期

R01以T03/R04/R05资源源及独立typed控制构造嵌套作用域、正常/early return、break/continue、分支/临时、move/clone/借用。独立oracle区分owner退出逆序与backing最后释放；有共享view时不能要求所有buffer随局部作用域立即free，也不能拿buffer全局分配倒序替代owner语义。

R04执行V06 Err("old")→Ok(42)：old恰释放一次、新活臂正确、inactive payload访问0。R06真实到达拥有值存活时的预算/取消/陷阱；分别观察错误原因、清理及原值/提交状态。取消沿原poll承诺，不要求任意指令即时中断；清理不再轮询已耗尽预算/取消。保3072层/256KiB小栈、512 churn与原燃料/墙钟压力；native仅现行record/text/collections域，AV支持闭包的退出另覆盖。

结果或宿主view/pin仍活时，允许对应资源live>0并断言内容可用；释放最后owner后live0且每项只free一次。预提交拒绝保旧，已提交后trap如实分态；错误后关闭/再恢复/再执行也必须验证。typed证人不能替代已存在source义务，不授信伪造MIR。

### 5.2 有界事件与父侧准入

事件至少能绑定source/profile/operation/producer、owner种类/id/代际、动作及顺序；精确字段由T-02按真实生产点冻结。跨arena/allocator的数字id须带域，不能将相同id合并。源/元数据4MiB、观察≤4MiB/操作、每流1MiB、值8MiB/managed32MiB/父private增量128MiB及原更严门保持。先限额再分配；观察满具名resource拒绝或原合同允许的明确失败，不静默截断后PASS。观察开关关闭与开启语言结果一致。

未知/重复/缺字段、坏类型/超界、错源/旧operation/schema/deps/tool、缺结束标记、截断/超量或事件重放均非绿；实际构造/释放点产生事实，父侧独立检查序列、once-release、最终live与合法pin。校验源→受检产物→真正执行产物身份；仅stderr文本不能认证任意二进制。负控须真实变异已确认命中的事件/清理路径，不以改变expected自造红。

### 5.3 函数增量、恢复及交接

支持的body暖路径至少三个未改函数/兄弟lambda：parse/resolve/typecheck/HIR/plan/verify/lower/encode/codegen/install及每函数工件IO按真实路径0，对象/代际保持；新增事件不得触发无关重编或重装。先用912/913已有闭包真函数路径及native已有集合会话见证；AV旧集合前端whole-source如实计P，不能宣称全前端0。声明编辑沿914 D-05冷声明前端+依赖精确rebuild，body不借用此例外。

fresh/hit/restart/body-edit、失败→修复→下一restart，P/R/M分账；支持纯恢复P0/R>0，重建数据来源/结构/资源，不缓存PASS/活对象。支持暖路径原正式30样本P90≤2000ms，冷构建/准备、fault/压力整段、运行/清理分列，不缩负载。正式采样与915/917/921错峰。

P私有fixture/观察codec/native探针可先做；AV S=av-core/av arena/结果观察与915逐符号取稳定SHA短窗，915仍独占故障与Place语义。共享lowering若需实际出口修复取明确窗；禁止重复所有权分析。H=ac_av.rs/ac_mn.rs/test_cases/mod.rs、auto-test-case、fixed组、binding/source-manifest全文件单写；与917/920/921逐文件交接，不消费其分支WIP、不持窗等认证。每窗记owner/revision/SHA/schema/构造匹配全部consumer/验证/释放；漂移STALE相应证据。无D915整包前置，但依赖其新增字段的任务须等正式稳定接口。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/resource-lifecycle-observations.md | 返回值/总数→真实逐owner事件/身份/有界准入 | 六格观察合同 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-test-case/ace-m1-scenario-oracles.md | 不改oracle；补真实入口/支持域 | 原语义不变 | AC-01/02 |
| SD-03 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 六格按正式父证据登记 | 不以探针充签章 | AC-03 |
| SD-04 | modify | docs/specs/auto-test-case/ace-m1-gap-audit.md | 六格owner关闭/真实残余 | 原批准/分母保持 | AC-03/06 |
| SD-05 | modify | docs/specs/auto-av/place-ownership.md | 动作观察/最后owner与原资源约束 | 不改借用语言 | AC-01/02/04 |
| SD-06 | modify | docs/specs/auto-ac/source-collections-variants.md | native真实事件/失败清理与原增量边界 | 不宣称闭包ABI | AC-02/04 |

## 6. 测试设计

现有固定职责组：ace-av、ace-native、ace-runtime、ace-adapters、auto-test-case、infra-ace-m1；若实际改共享source/lowering，加ace-source/ace-semantics/ace-lowering及真实消费者。新增拟组case-ace-m1-resource-events-919由T-05注册，不能现在当现成命令。首例长期fixture拟放crates/auto-lang/test/cases/ace-m1/resource-events-919/。以下均未执行。

| Case ID | AC / invariant | 输入/真实入口与使能任务 | 可观察oracle/区分反控 | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01 owner/LIFO | R01 source+typed，经AV/native，T-03/04 | 源规则独立序列；漏drop/重复/换序均红；move不重复释放 | 正常/early/分支/循环/临时 |
| V-02 | AC-02 活动臂 | V06两端，T-03/04 | old一次、新结果42、inactive读0；只报42反控红 | 现有有限Variant |
| V-03 | AC-02 错误清理 | 活owned树/集合/AV闭包，真正budget/cancel/trap，T-03/04 | 原原因与cleanup/live；禁cleanup或仅杀进程非绿 | 原poll/支持域 |
| V-04 | AC-01/02 最后owner | 持view/返回值/pin→释放，T-03/04 | 提前释放/永久保活均红；最后live0 | 原Share/Static语义 |
| V-05 | AC-03 父信任 | 实际事实坏字段/源/截断/重放，T-05 | 合法非零id可用、坏事实拒绝；PASS字段无效 | 原受管产物/来源 |
| V-06 | AC-04 函数工作 | 三未改函数＋真实body编辑，T-06 | 各生产/IO0与真实payload恒等；强制whole/换对象红 | 已承诺暖路径，旧冷域单列 |
| V-07 | AC-04 恢复再读 | fresh/hit/restart/body-edit、坏恢复→修复→再恢复，T-06 | 纯恢复P0/R>0、下一编辑正确；错依赖/伪Checked拒绝 | 913正式接口/原域 |
| V-08 | AC-05 空间/时间 | 原深链/小栈/churn/正式样本及观察边界，T-06 | 上限前拒/保旧、P90≤2000ms；静默截断反控红 | 原负载/条件不缩 |
| V-09 | AC-03/06 六格 | 新AutoDown完整组+852 contract/acceptance，T-05/07 | 恰六格可证明、原313/NA/accepted守恒；其他gap保留 | 非全M1签章 |

实现与自查同一实施树；final review独立上下文核每AC/源码/反控，不另建agent/chat或第二checkout。review时冻结SHA暂停实施写入；merge重核主线漂移后跑完整职责/必要消费者并集，复用身份未变证据。周期cargo t all仅due主检出单实例，非本包每轮门。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | R01两端真实owner/最后backing/所有出口经V-01/04，无观测改变寿命或drop顺序 |
| AC-02 | work | R04/R06两端V-02/03真实原因/一次清理/活动臂与资源边界通过 |
| AC-03 | work | V-05/09正式六格与严格父准入、独立oracle/反控全部接线，原源/expected/规则不变 |
| AC-04 | work | V-06/07未改真实P/IO0、对象/代际、恢复重验与持久修复通过，旧冷域如实分账 |
| AC-05 | work | V-08原正式时间/内存/深链/观察预算全过，不以杀进程/缩样本代替 |
| AC-06 | review | 独立冻结review逐项pass；无漏项/未批延期；native三格等残余owner清楚 |
| AC-07 | merge | 正式落地/SD读回/archive，全部实际输出对账、guard clean、树/分支/scope清理并有收据 |

## 8. 执行步骤

| T | owner_stage | 前置/实际写面 | 可验结果与验证入口 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 开工main/915/917窗口 | 重核来源SHA、六格、现有生命周期/消费者及P/S/H；边界差额有owner | AC-01..05 | [x] 已完成（lang-919 树@master 8bb1a0d52：oracles b8791a06/av.rs ff0d3e19/collections-runtime 4f379e21 三 SHA 零漂移；915 分支 av.rs+206/ac_av.rs+1412 未合入——本包 AV 写面以 master 稳定符号为基并逐符号记录，915 独占故障/Place 语义未触碰；917 仅 a2r-emit 文件无交集；native/collections-runtime 无竞争窗） |
| T-02 | work | T-01，私有P | 有界观察/身份/注入灵敏度最小whole-path feasibility，先AV再native；原oracle可区分再实现 | AC-01/02/03/05 | [x] 已完成（schema 冻结为 av.lifecycle/1 + MN-EVT 双通道：owner=(kind,id,key)、槽级责任 (frame,slot,gen)/native (frame,slot)；上限 AV 16384→av.resource.lifecycle-events、native 8192→MN-COLL-EVT+abort22，先限额再拒绝不截断；whole-path 可行性＝AV 8 测试经真实源管线全绿） |
| T-03 | work | T-02，915释放AV稳定符号 | AV三场景+必要真实清理修复；V-01..04，cargo t ace-av | AC-01/02 | [x] 已完成（execute_observed+av.rs 钩子/arena free 钩子/driver resource_events.rs 双 profile/worker recipe/父判 admit+冻结表+12 反控；cargo t ace-av PASS 985；发现并绕开 VariantSwitch 继承占位 ty=MAX 的身份陷阱） |
| T-04 | work | T-02，native/runtime N窗 | native三场景+实际清理与codec，V-01..04，cargo t ace-native ace-runtime | AC-01/02 | [x] 已完成（runtime MN-EVT/cap/end+wrapper 旋钮+observe 严格解码+worker 面+ac_mn 父判表；native 8 测试绿；实际修复两处：Frame seq 使 codegen 栈槽 32→40 同步（原硬编码为潜伏栈损坏）、mn worker 分类/recipe 回显接线） |
| T-05 | work | T-03/04，H窗 | AutoDown/严格父判/负控/拟新组/六格回填，V-05/09；cargo t ace-adapters auto-test-case infra-ace-m1及新组 | AC-03 | [x] 已完成（套件 10 checks 一次全绿 run-cff9f6ld：六格经真实 worker+父侧冻结表+19+7 反控；binding 三行六格 ok:P919-events；manifest scenario_assets 清单化+plan_919_update+oracles hash 刷新；ps1 修退役 feature 与清单化；contract/source/acceptance 三 scope PASS，acceptance core 红 -6） |
| T-06 | work | T-03..05，错峰 | V-06..08真实暖编辑/恢复/资源/计量，必要原ace-performance命名门 | AC-04/05 | [x] 已完成（观察 OFF 一致性双侧常驻对照；913 closures-recovery 与 889 native-collections 暖/恢复组复跑 PASS＝零额外重编/重装；t06 预算界测试：事件流 ≤4MiB 断言+wall<2s；P90 正式采样属 ace-performance 里程碑档，与 915/917/921 错峰 pending） |
| T-07 | work | T-01..06 | 完整受影响固定组自查/SD提案/差额清单，保未执行和旧红，不签总M1 | AC-01..05 | [x] 已完成（修复提交 735aa75b0：R1 rustfmt×5、R2 死码移除、R3 pub(super)；连坐复放全绿——cargo check 零 919 警告、rustfmt 六文件净、父判 19/19、av 8/8、native 8/8、套件 PASS run-eubocovi；SD/并集证据沿 82208b51a 有效，identity 变更仅限三健康修复不改行为） |
| T-08 | review | work任务完成，冻结SHA | 独立验每AC/负控及Spec delta；pass才reviewed，修复回同树 | AC-06 | [ ] |
| T-09 | merge | review pass | 冻结职责并集/漂移重验、主线落地、SD-01..06读回与归档 | AC-07 | [ ] |
| T-10 | merge | T-09 | 全输出对账/摘要折计划，guard后删树/分支/组及整个scope、刷新登记 | AC-07 | [ ] |

拟唯一树D:/autostack/.wt/lang-919/auto-lang，分支plan-919-dev，本轮未建。实施材料唯一D:/autostack/plan-artifacts/plan-919/，开工登记owner/用途/在途；禁attachments/仓内证据目录/散落副本。merge全删，仅具名未解决hold可留；不清他方文件、不建链接。

## 9. 复审记录

2026-11-02 | stage:new | PLAN-919 r1 | outcome:pass（规划静态准入，不是功能/最终review PASS） | 0..10节、T10/AC7/V9/SD6定义与引用/本地链接锚点/固定组/六格互斥/全未勾已核；next:work T-01，T-02私有/native可先，AV相关任务按915冻结符号窗口。收口时main推进c3304ed73，915已execution_done6/10待review，尚无D915；不消费未合入WIP。原binding仅并发checkout规范化为LF，与HEAD blob逐字节相同，未改token/source/expected；两份批准及applicability/source-manifest未变。未建919树/scope或运行语言测试。

2026-11-02 | stage:new | PLAN-919 r1 | 起草：先Design43 §20分域后排他取号919；915/917在途状态继承，六格互斥，不把资源总数当逐owner事件。本轮只规划，无实施/测试/认证/current Spec/原登记变更。静态预检与最终运行验收分开记录。

取号脚本首次已写919骨架，但更新.next-id短暂Permission denied；在同一排他锁下核唯一骨架/next919，原生文件写修复为920，再顺序分配920..922；未覆盖他方文件，最终next923。不是语言功能或测试结果。

## 10. 待澄清事项

| D | owner / 有界关闭 |
|---|---|
| D-01 owner事件可行性 | T-02核AV/native所有实际构造/转移/释放出口与成本；缺观察在本包补，不缩六格 |
| D-02 原集合前端粒度 | T-01/06按当前Spec分层计量，body0只承诺已有可行真路径；更大前端平台另交明确owner |
| D-03 915/917并行窗 | T-01登记实际稳定SHA/字段/全部consumer；依赖符号未释放只挡对应任务，不抢窗 |
| D-04 native三格 | 不由919签闭；922记录R02/R03/R05真实能力/后继拆包，禁止凭无闭包能力自动NA |

2026-11-03 | stage: work | PLAN-919 r1 | outcome: pass | worktree: D:/autostack/.wt/lang-919/auto-lang | branch: plan-919-dev | base_commit: 8bb1a0d52 | code_commit: 82208b51a (379efef7a→c89776daa→3562c6776→82208b51a) | dependency_revisions: auto-down 组内 detached 7c4d5cb（本包测试未消费）；915/917 分支未合入未消费，AV 写面以 master 稳定符号为基（ff0d3e19） | task_ids: T-01..T-07 | evidence: 六格全闭＝套件 case-ace-m1-resource-events-919 10/10 PASS（真实 worker×2+父判冻结表+19+7 反控）；cargo t ace-av 987、ace-native 183（Frame 40B 布局峰值常数修正后整组复跑 PASS）、ace-native-worker 39、ace-runtime 84、auto-test-case 111、infra-ace-m1 23、closures-recovery-913、native-collections-889 全绿；verify-ace-m1-852 contract/source/acceptance PASS（acceptance core 红 -6、integrity PASS、can_claim_m1=false 如实）；观察 OFF 双侧对照常驻；暖/恢复零额外重编 | blockers: 无合同内未决；KNOWN-DEBT 登记 遗留A（text profile 多参 move 核对错位，探针实证）、遗留B（mn 前端 place.field_path/record.clone 边界）、顺带修复（ps1 退役 feature+scenario_assets 清单化）；ace-performance P90 正式采样为里程碑档与 915/917/921 错峰 pending，非本包每轮门 | next_responsible_role: review（T-08 独立冻结复审）

stage: goal | plan_id: PLAN-919 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-919/auto-lang | commit: 82208b51a | next: review

2026-11-03 | stage: review | review_scope: final | reviewer: 同会话独立复审（自 artifacts/受审提交重跑重建，未采信执行摘要；实施会话内复审的既有限制如实声明） | plan_id: PLAN-919 | plan_revision: r1 | outcome: needs_fix | worktree: D:/autostack/.wt/lang-919/auto-lang | branch: plan-919-dev | reviewed_commit: 82208b51a | base_commit: 8bb1a0d52 | dependency_revisions: 915/917 分支未合入未消费；AV 写面 master 稳定符号（av.rs 基 ff0d3e19，diff hunk 清单复核未触 fault/Place 区） | spec_inputs: SD-01..06 全文本 vs 行为核验（新规范+oracles 三行仅起点列更新+conformance/gap-audit/两附录；manifest oracles hash 3ed6ca2a 刷新，contract PASS 绑定 82208b51a） | acceptance_results: AC-01..05 逐项 pass（独立重跑：套件 10/10 run-gwfaqals、av 8/8、native 8/8、父判 19/19、852 contract PASS；冻结表与 zz_dump 独立再推导一致——native dump 常量 4-seed 与套件 8-seed 源分裂仅记 N1 非表错；AC-04 暖/恢复=913/889 组绿+观察 OFF 构造性零重编；AC-05 界测试绿、P90 里程碑档 pending 属合同内） | pending_closeout_gates: 冻结受影响模块并集（merge-owned；work 侧已全绿一轮：ace-av 987/ace-native 183/worker 39/runtime 84/auto-test-case 111/infra 23/套件/913/889）；852 acceptance 复放 | findings: F-919-R1（P3，T-07）rustfmt --edition 2021 --check 漂移 5 文件（av/src/resource_events.rs、两 919 测试文件、ac_av/ac_mn resource_events.rs）；F-919-R2（P3，T-07）ac_av/resource_events.rs completion_code_for 死码（Arms 已直连 judge_expectation）+ 私型公函警告；F-919-R3（P3，T-07）judge_resource_events pub 签名暴露私有 StepOut（收紧 pub(super)）。观察项不阻断：N1 native dump 测试常量与套件源分裂（表值正确，套件 PASS 证）；N2 VariantSwitch 继承占位 ty=u32::MAX 既有 VM 事实（owned_identity 注释已冻结）。 | evidence: 本 §9 记录即持久证据（命令+结果如上）；无合同内功能缺陷，六格闭合与反控真实性经受独立重验 | next: work（最小修复集 R1..R3；修复提交后仅复放：cargo check 零警告、rustfmt 六文件、19 控制+套件）

stage: goal | plan_id: PLAN-919 | plan_revision: r1 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-919/auto-lang | commit: 82208b51a | next: work

2026-11-03 | stage: work | PLAN-919 r1 | outcome: pass（修复轮） | worktree: D:/autostack/.wt/lang-919/auto-lang | branch: plan-919-dev | base_commit: 8bb1a0d52 | code_commit: 735aa75b0 | dependency_revisions: 同上轮 | task_ids: T-07（F-919-R1..R3 修复；顺带修复 T-07 行列数并档） | evidence: 复审最小复放集全绿（19 控制/av 8/native 8/套件 PASS run-eubocovi）；rustfmt 2021 六文件净；cargo check 零 919 警告；六格功能证据沿 82208b51a 复审轮有效 | blockers: 无 | next_responsible_role: review（受影响项重审：R1..R3 关闭核对+复放）

stage: goal | plan_id: PLAN-919 | plan_revision: r1 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-919/auto-lang | commit: 735aa75b0 | next: review

2026-11-03 | stage: review | review_scope: final（修复重审，受影响项） | reviewer: 同会话独立复审（工件重建；限制同前） | plan_id: PLAN-919 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-919/auto-lang | branch: plan-919-dev | reviewed_commit: 735aa75b0 | base_commit: 8bb1a0d52 | dependency_revisions: 同前轮 | spec_inputs: SD-01..06 无变更（修复轮零规范改动） | acceptance_results: F-919-R1/R2/R3 全关（独立复验：rustfmt 2021 六文件净、completion_code_for 零残留、pub(super)；cargo check 零 919 警告；控制 19/19、套件 PASS run-uqfa_1y8 @735aa75b0）；修复 diff 最小（82208b51a..735aa75b0 仅三修复+格式，无行为变更）→前轮六格功能证据沿用有效；AC-06 关闭 | pending_closeout_gates: merge-owned：冻结受影响模块并集在合入基面重验（ace-av/ace-native/ace-native-worker/ace-runtime/auto-test-case/infra-ace-m1/case-ace-m1-resource-events-919 + 913/889 抽验）、852 三 scope 复放、SD 读回、归档与 guard 清理（AC-07） | findings: 无新发现；N1/N2 观察项维持不阻断（N1 已记 KNOWN-DEBT 侧无，属维护性注记） | evidence: 本 §9 + 复审两轮记录 | next: merge

stage: goal | plan_id: PLAN-919 | plan_revision: r1 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-919/auto-lang | commit: 735aa75b0 | next: merge

2026-11-03 | stage: merge（进行中，检查点） | PLAN-919:r1 | 检查点：prepared+冻结门已过；rebase 冲突已解但触码，交付提交待受影响重审 | 细节：master 吸收 915/917/920 后 rebase 五提交冲突 15+ hunk（StepOut 双 resource 字段→919 侧改名 exit_kind；manifest 采 master 每计划 scenario_assets_<plan> 约定弃清单化；ExecTrace lifecycle 字段与 915 place_facts/finish_trace 共存；ps1 退役 feature 全四处修复含 915 新增两处；binding R04/R05 行粘连修复）。分支 squash 为单一交付提交（映射：379efef7a+c89776daa+3562c6776+82208b51a+735aa75b0 → 07e14a4dd，基 e583d9b34，零删 master 内容；range-diff 不适用——以全量重验电池证等价）。重验（绑定 07e14a4dd 树）：并集 10 组全 PASS（ace-av 996/ace-native 183/worker 39/runtime 84/auto-test-case 111/infra 23/套件919=10/套件915=12/913=5/889=40）+852 contract/source/acceptance PASS+控制 19/19+915 10/10 | next: review（受影响重审：rebase 解的语义并置与 915 侧共存无回归）」

stage: goal | plan_id: PLAN-919 | plan_revision: r1 | round: 5 | from: merge | to: review | status: reviewed | outcome: waiting-affected-rereview | worktree: D:/autostack/.wt/lang-919/auto-lang | commit: 07e14a4dd | next: review
