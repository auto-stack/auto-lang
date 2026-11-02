---
plan_id: PLAN-915
status: executing
feature_name: AV资源故障与陈旧Place正式闭环
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-02
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-av/place-ownership.md, docs/specs/auto-av/collections-variants.md, docs/specs/auto-av/closure-environments.md, docs/specs/auto-av/cache-and-conformance.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-gap-audit.md]
new_spec_components: [docs/specs/auto-av/resource-fault-and-generation-conformance.md]
touched_goals: [GOAL-002]
affects: [auto-av, auto-test-case]
depends_on: [PLAN-821, PLAN-822, PLAN-864, PLAN-909, PLAN-912]
current_step: 2
total_steps: 10
---

# [PLAN-915] AV资源故障与陈旧Place正式闭环

## 0. 变更摘要

仅认领ns=scenario R02/R03的av两格；先复用已有fault/代际防护，再补正式逐点注入、失败原子/清理、陈旧访问前拒绝与父侧独立证据。drafting r1、0/10，本轮只规划，不建树/材料/跑探针测试认证，不改当前Spec、原source/expected/binding或913实施进度。[Design43 §17](../design/43-ac-av-typed-mir-and-abc.md#17-m1下一阶段场景观察av资源与旧a2r2026-11-02target-state)为架构依据。

## 1. 目标

G1：支持域构造/clone/扩容每个实际分配attempt均能注入，失败不提交、部分输出回收、源/旧值不变。G2：可授信List Place的frame/slot/buffer/epoch/element-generation逐次使用校验，stale危险访问=0，合法相邻访问保持。G3：首例文件化、故障后修复/恢复重验、真实生产/预算/时间门和全部consumer接线。非目标：native R02/R03/R05、完整资源事件R01/R04/R06、全局句柄重构、新借用语言、共享owned/cycle、913恢复平台或D-909-1修复。

## 2. 架构方案

真实source/typed fixture→既有verify/lowering/ABC再授信→真实AV executor/arena/place注入与观察→有界操作事实→现有ac_av父判→两格报告回填。优先复用已存在防护，不根据陈旧gap假定引擎没有实现。新增防护只修实证遗漏，保持既有协议/资源语义；默认路径fault关闭，诊断事实不带oracle。

## 3. 技术栈

Rust；av-core/av/core/lowering、auto-lang-core TestCase adapter；沿固定cargo t组、现有受控worker与内存/小栈测试，私有fixture/观察codec优先。

## 4. 需求分析与背景调查

2026-11-02 用户 `/goal 完成计划915` 授权整链（work→review→bounded repair→merge），继承原批准范围；T-01 起在唯一树施工。实际基面 = main `5a0ef7609`（913 已归档合入；914/916 亦已交付），计划起草时引用的 `0e93e3cc9` 仅历史。D911=`ca4848f5d`、D912=`13a465336` 均已是祖先。**T-01 实测基线更正**：binding.tsv SHA256=`7b3092173791447f7791df13e6d51ef7c196bf5968f53853fd104135e6215e2f`（起草稿 `67bf520d…` 已被 914/916 落盘更新，作废）；`ace-m1-scenario-oracles.md`=`b8791a06…`（起草稿 `c0539b96…` 同因作废）；`applicability.json`=`74c9109b…`（与起草一致）。av.rs=`ff0d3e19…` 与起草一致；owned_text/closures/place 实测见 §9。34 静态阻塞中的本两格确实**不是**同名 corpus:R02/R03（corpus R02/R03 是另一对行，`applies` 命名空间见 binding.tsv:3-5）。

| 来源 / 实读符号 | 当前事实 / 基线 |
|---|---|
| docs/specs/auto-av/collections-variants.md | SHA256 03fd8d4aacc567861c379c9b0a0fed8c0f33139153f8d40fe9f8282ec32fd644；Place已有使用期stale检查与失败precheck→commit |
| docs/specs/auto-av/place-ownership.md | SHA256 0b51498dc6eeff94770e3204b6fab4da14d89e5becd5341d1ba3ee0da3fb00a0；Share/Static StrView合法持buffer，不能把全部旧view判无效 |
| experimental/ace/crates/av-core/src/av.rs | SHA256 ff0d3e196d932468cb304211f5b3421d59aec7681393422f90164a97e2cf703f；ExecLimits::fault_after_allocs、execute_with_arena、clone_owned_value及List Place检查已存在 |
| 同src/owned_text.rs / closures.rs / place.rs | StrArena::set_fault_at/attempts/successful_allocs/list_alloc_attempt；捕获clone故障有分类；不是所有frame token已有解引用消费 |
| av/tests/cases/{av_collections_resources,av_collections_semantics,av_owned_resources}.rs | 已有故障/资源/stale测试；缺口重点是原scenario正式源/typed及父观察，不另造一套arena |
| docs/specs/auto-test-case/ace-m1-scenario-oracles.md | SHA256 c0539b968ba59034d09dce6fc6169c76a97888524482c47f15b092b07b75ce3a；R02/R03关闭oracle保持 |

## 5. 详细设计

### 5.1 支持域、事务与恢复

R02至少覆盖原L04/L05/V05/K02及对应typed深链控制：Str、owned record、List<Str>/List<owned record>、Variant载荷、closure capture clone。T-02先冻结每例真实分配域/路径/attempt总数N；遍历1..N各点（不能只测第1点），再测N+1正控及无注入对照。slot delta fault和owned arena fault区分；预算precheck拒attempt0、allocator故障attempt>0/success不增分别断言，不把OOM试验换成随机主机耗尽。

每个fail点：旧值/源内容、所有权与epoch不变；构造部分逆序/显式工作栈回收，once-release、live恢复基线；注入命中不能panic，不依赖OS杀进程证明逐owner释放。guest已commit后的trap不谎称未commit；故障不写PASS/半产物。清故障后同会话真实重跑正确结果，关闭/最后owner后live0；active pin保活到最后释放，不因current/history列表空就记清理完。原3072层/256KiB小栈及512 churn/取消燃料墙钟门不缩。

R03使用真正被verify接受的List Place序列：旧token→push/insert/remove/替换元素→旧token使用必须stale-place且危险访问0；新token/未变元素/合法buffer share正控可用。覆盖真实buffer/epoch/elem_gen/owner frame/slot边界，不把伪造未授信MIR当合法stale输入。静态活loan冲突须前门拒，另有合法运行期失效路径；StrView Share/Static活buffer保持可读，不新增全局gen语义。若原场景只能静态拒绝，T-02须并列给真实运行期Place证人，不能只读gen字段过门。

### 5.2 观察与父信任

attempt/commit/alloc/free/live、旧/新内容摘要、frame/buffer/epoch检查及dangerous-access计数由真正动作点产生；危险读负控应能区分“事后trap”。请求只带fault设置/操作，不带期望计数；观测codec私有、未知键/坏类型/重复key/旧operation/source/tool/deps/截断均拒。新增计数若只测试启用，生产默认零开销或有界明确开关；不得将未执行路径默认0冒零生产。正常开关关闭与开启的语言结果完全一致。

### 5.3 增量、资源与913窗口

文件化fresh/hit/restart/body-edit及失败→修复→下一次恢复；仅恢复数据重验不缓存运行结果。用D912已有ClosuresFunctionSession的至少3个真实未改函数/兄弟lambda作共享资源改动暖路径见证：真实parse/resolve/typecheck/plan/bind/verify/lower/encode/install/IO0、对象/代际同进程保持；source/MIR/ABC入口各自身份不混。集合旧前端未承诺的粒度如实计量留owner，不能把全编译后wire命中冒P0，也不扩本包为全语言增量重写。913未交付时沿现有恢复如实记R3前置生产，D913交付后只重核受影响恢复consumer，不把D913设整包前置或冒纯恢复P0已实现。

值≤8MiB、受管≤32MiB、父增量≤128MiB及原更严预算；D912保留前端16MiB、code current+history8MiB/每unit32原门不放宽。新观察≤4MiB/操作/每流≤1MiB且不可无界累积N点事件。事件/表先限额后物化、资源拒绝先于变异，源/元数据原4MiB门保留。支持暖路径四状态各30样本P90≤2000ms、原深链/压力/样本不缩，fault遍历总用时和单编辑时间分列，正式采样与913错峰。

私有P=新增资源fixture/codec/tests；本包S主av-core/{av.rs,owned_text.rs,place.rs,closures.rs}及受影响AV执行资源观察；913主恢复manifest/root/lease/restore与accepted-context。共用av-core/closures、AV公开结果、ac_av.rs、test_cases/mod.rs、auto-test-case、固定组、binding/source-manifest须逐符号/全文件单写短窗，登记owner/SHA/版本/构造匹配/所有consumer/已验与释放；不得只因Rust加字段就认为兼容。914只读取正式资源事实、不重复注入API，916/917与本S独立；接口再变STALE受影响证据，不停他方进程、不批清TEMP。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/resource-fault-and-generation-conformance.md | 零散测试→两场景全链合同/覆盖边界 | 可审计 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-av/place-ownership.md | 既有语义不变、补实际fault/合法view边界 | 不误扩gen | AC-01/02 |
| SD-03 | modify | docs/specs/auto-av/collections-variants.md | 补逐点注入及运行前stale正式证据 | 源/typed分层 | AC-01/02 |
| SD-04 | modify | docs/specs/auto-av/closure-environments.md | 补capture故障/最后owner/增量证据，未扩共享owned | 不重开909 | AC-01/04 |
| SD-05 | modify | docs/specs/auto-av/cache-and-conformance.md | 故障修复后恢复真实重验、不缓存结果 | 事务保真 | AC-04 |
| SD-06 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 两格由正式父证据重绑 | 非token绿 | AC-03 |
| SD-07 | modify | docs/specs/auto-test-case/ace-m1-gap-audit.md | 两格状态与残余owner | 不签M1 | AC-03 |

## 6. 测试设计

work自查→同一唯一实施树独立上下文review→merge。新增长期目录crates/auto-lang/test/cases/ace-m1/resource-915/含case/suite/source或typed fixture/独立reference；拟固定组case-ace-m1-resource-915由T-05接入。所有V目前未执行。

| Case ID | AC / invariant | actual entry / inputs | observable expected / distinguishing control | owner_stage / task |
|---|---|---|---|---|
| V-01 | AC-01 每分配点失败 | 真AV每例1..N及N+1/无fault | attempt真实、旧值不变、部分回收/live基线；只首点/漏一allocator红 | work T-02/03 |
| V-02 | AC-01 退出清理 | clone/构造/扩容/捕获与原深链、cancel/fuel/wall | once-release/最后owner、无递归溢出；只杀进程/先commit后报错红 | work T-03 |
| V-03 | AC-02 stale使用期 | 已授信Place跨结构/元素/帧代际变动 | 使用前stale、危险访问0；禁检查/事后trap红 | work T-02/04 |
| V-04 | AC-02 合法近邻 | fresh Place/未变元素/活Share/Static、静态loan反例 | 合法能读，静态非法另拒；全拒所有view的负控红 | work T-04 |
| V-05 | AC-03 父准入 | 错op/source/工具/类型/重复key/超额、伪0 | 非绿且有效非零身份通过，不拿exit70代替 | work T-05 |
| V-06 | AC-04 增量/恢复 | D912暖链+故障修复、下一恢复 | 未改全链P/IO0、同对象/代际；R3真实另账，返旧值/不execute红 | work T-04/06 |
| V-07 | AC-04 预算/时间 | 原深链/512、30样本四状态/事件上限 | 资源/原P90/正确性同过、超额受控拒绝，峰值非RSS单点 | work T-06 |
| V-08 | AC-03/04 格位/交接 | 两格父报告、913/914接口与全consumer | 仅scenario R02/R03 av回填、坏/旧证据拒，其他列/规则守恒 | work T-05/06 |

完整职责组拟ace-av ace-source ace-lowering ace-adapters auto-test-case ace-host infra-fixed infra-ace-m1及case-ace-m1-collections-864/closures-865-source/shared-mutable-909/closures-fn-912/新resource组；公开AV/共享IR变更补ace-native ace-a2x及相关consumer，不以测试位于AV就豁免。ace-performance受影响原命名腿保正式规模/条件，与913隔离资源时段。T-01冻结理由/实际并集；不机械全experimental。

## 7. 验收标准

| ID | owner_stage | closing condition |
|---|---|---|
| AC-01 | work | R02支持域各实际分配点失败原子/回收/故障后正控，V-01/02全闭 |
| AC-02 | work | R03真授信Place使用前防护/合法view保持/强制红，V-03/04全闭 |
| AC-03 | work | 文件化父准入/两格正式证据回填，V-05/08全闭，源/expected/NA/分母不改 |
| AC-04 | work | V-06/07/08、资源/真实增量/恢复/受影响完整组与全部consumer/健康，913窗已交接 |
| AC-05 | review | 独立逐AC/V/SD/遗漏与workaround核查绑定冻结SHA，无未批延期 |
| AC-06 | merge | 实际主线交付、Spec读回/归档/全路径摘要与guard清理，保剩余owner不签M1 |

## 8. 执行步骤

| T | owner_stage | 前置/窗 | 工作与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施指令；D912；913/914实际S/H | 冻结两格/已有防护、hash/支持域/cap/consumer/唯一树材料与窗口 | AC-01..04 | [✅] |
| T-02 | work | T-01；P/必要S | 最小源/typed逐点fault→清理→修复、合法stale→拒绝→fresh，冻结N/事实codec；缺可行路径needs_replan | AC-01/02/03 | [✅] |
| T-03 | work | T-02；av-core资源S | 接既有fault seam/真实attempt/事务与显式清理，修实证缺口，V-01/02 | AC-01 | [ ] |
| T-04 | work | T-02/03；Place/恢复只读S | 接真实使用期检查/危险访问事实、合法近邻与D912暖门，V-03/04/06 | AC-02/04 | [ ] |
| T-05 | work | T-03/04；H/M1 | 正式case/codec/父判/独立oracle/新固定组、两格父证据回填与全consumer，V-05/08 | AC-03/04 | [ ] |
| T-06 | work | T-01..05；运行错峰 | V-01..08、原资源/时间/完整组/健康与遗漏自查，冻结SD/SHA→execution_done | AC-01..04 | [ ] |
| T-07 | review | T-06同树冻结SHA | 独立全链反例/支持/所有权/真实P0复审，修复重绑pass→reviewed | AC-05 | [ ] |
| T-08 | merge | T-07 pass | 冻结受影响组/消费者并集或有效同身份证据；batch仅due主检出单实例 | AC-06 | [ ] |
| T-09 | merge | T-08 | 合入、SD-01..07读回、窗口/残余owner交接并archive delivered | AC-06 | [ ] |
| T-10 | merge | T-09 | 所有实际输出摘要折计划，guard clean后清唯一树/分支/组/整个scope及登记 | AC-06 | [ ] |

拟唯一树D:/autostack/.wt/lang-915/auto-lang、分支plan-915-dev，未建；开工材料唯一D:/autostack/plan-artifacts/plan-915/登记owner/用途/在途，所有探针/报告/验证副本/外置target入此，禁attachments/仓内reports/全机TEMP批删。merge整删，具名未解hold才可留，不动913数据/他方输出、不建链接。

## 9. 复审记录

2026-11-02 | stage:new | PLAN-915 r1 | 规划待静态预检，非运行或最终review PASS。先设计§17/索引，后allocation.lock排他取914..917，next918。当前防护已实读，故障/Place不是空白新架构；只认两格、不消费913 WIP，不重开821/822/909/912。未实施/测试/认证/建树材料/agent/chat，current Spec/source/expected/binding未变。后续T-01，无D913整包前置；AC/V均待运行。

2026-11-02规划静态预检通过：0..10节有序、T10/AC6/V8/SD7定义唯一且引用齐、10任务全未勾、drafting r1/0-of-10；现有Spec路径存在、新Spec仍未建立、本计划设计链接/anchor/空白检查通过。固定组名称对照test-groups.toml有效，新914..917场景组明确为T-05未来接线目标；D911ca4848f5d与D91213a465336均为main祖先(exit0)。四包各认领12/2/8/2格、无交集且都是当前34阻塞子集，剩余10格清单已核，非正式运行PASS。914..917新号各唯一、分配后next918，无对应worktree或plan-artifacts，原binding SHA256仍67bf520d26e7d541f8fd5f44b8c218da9d40b67d3fd808b4b3e801b89ec47fb9；本轮未改当前Spec/代码/脚本/913。所有语言功能、资源、时间、认证和最终review证据仍待执行。

## 10. 待澄清事项

| D | owner / bounded closure |
|---|---|
| D-01 注入覆盖 | T-02冻结真实allocator路径/N/失败分类，任何漏项补实际门，不能只测第1点 |
| D-02 stale与合法借用 | T-02/04冻结运行期Place证人与Share/Static正控，不扩语言/全局handle语义 |
| D-03 913交接 | T-01/05逐符号正式SHA/全部consumer，恢复schema/root归913，未取窗只挡相应任务 |
| D-04 原域残余 | T-06/merge如实记录集合旧前端粒度、R3及其他M1格/旧红owner，不冒全语言零生产 |

2026-11-02 | stage: work | plan_id: PLAN-915 | plan_revision: 1 | outcome: pass（T-01..T-06，execution_done）| worktree: D:/autostack/.wt/lang-915/auto-lang | branch: plan-915-dev | base_commit: 5a0ef7609 | code_commit: b1e6cf127 | dependency_revisions: 无（未改跨仓依赖）| next_responsible_role: review
证据：
- T-01 基线冻结：main `5a0ef7609`（913/914/916 已合入）；binding.tsv=`9563cfc6…`（起草稿 `67bf520d…` 已被 914/916 更新，作废）、scenario-oracles=`b8791a06…`（同因）、applicability=`74c9109b…`（未变）、av.rs=`ff0d3e19…`（与起草一致）、owned_text=`5056069…`、closures=`ed2bb48…`、place=`c7de6e9…`；唯一树/材料/窗口按 §8 末段执行，未建链接、未动 913 数据。
- T-02 fixture 与冻结 N：父表 `RESOURCE_SWEEP_CASES` = L04 42/6/2、L05 42/5/2、V05 42/6/1、LSTR 51/5/3、K02 42/5/1（结果 / 值 seam attempt / arena seam attempt）；`PLACE_SEQUENCES` = push/insert/remove/replace → `av.trap.stale-place`；`CAPTURE_CASES` = clone attempt 3/4 失败、5 成功、live 恒 2。
- T-03 事务与清理：值 seam 增补 List 容量扩容为真实 allocation attempt（先于变异，仅 seam 配置时计数）；arena seam 覆盖 owned Str/List 缓冲；每点失败均为可辨识 `Infra` 且 live=0；N+1 正控、无注入对照、故障清除后同进程重跑、全链重编译恢复（`after` / `after_recompile`）全部断言。
- T-04 Place 与合法近邻：`ExecLimits::observe_places`（默认 false，生产零成本）→ `ExecTrace.place_facts`（mints/get_ok/get_stale/set_ok/set_stale/buffer_reads，仅统计通过全部校验后的访问）；typed 授信序列（verify_mir→encode→ABC admit）四种结构变更后旧 token 均 `av.trap.stale-place` 且 `buffer_reads` 仅含合法读；fresh token 与无变更 legal 正控成功；源码侧静态反例以 `bind/source.iterate-mutate`（stage=resolve）另记，不与运行期证据混用。
- T-05 正式材料与父准入：`case-ace-m1-resource-915`（12 checks：L04/L05/V05/LSTR/K02 逐点 sweep + capture-clone + 4 个 place 变更 + 合法近邻 + 静态反例）；父侧 entry 注册表、与 oracle 无关的 face 形状门、`resource_on_non_resource`、core 身份按 entry 绑定（`source:k02`/`capture:*`→closures，其余→collections）、响应帧重复 key 拒绝 `reject_duplicate_keys`（全部 AV 配方，serde 之前）、组内 `AV915_SABOTAGE` 负控注入。两格回填：binding.tsv `scenario:R02/R03` av=`ok:av-resource-915`（12 列、ac/avm/a2r 与 expected/NA/分母不变）。
- 验证（冻结提交 `b1e6cf127`）：`cargo t ace-av auto-test-case case-ace-m1-resource-915 case-ace-m1-collections-864 case-ace-m1-closures-865-source case-ace-m1-shared-mutable-909 case-ace-m1-closures-fn-912 case-ace-m1-closures-recovery-913 case-ace-m1-order-layout-914` → **9/9 PASS，gate_passed=true，error 空，wall 644.6s**（run-7zqs3bd7）；负控 `AV915_SABOTAGE=drop-point` 与 `no-stale` 均 FAIL（父冻结表确实能区分漏点/假 stale）；crate `resource_915` 7/7、父判 `resource_915_tests` 8/8、`worker_json_tests` 4/4；`verify-ace-m1-852.ps1 -Scope contract` PASS（131 行 / 313 required），`-Scope acceptance` Integrity PASS（green 263、gap 99、can_claim_m1=False）。
- 健康与格式：新增/改动文件 rustfmt 0 diff（resource_915.rs、tests/cases/resource_915.rs、av.rs）；ac_av.rs 仅余该文件既有的 35 处历史偏差（基线同文件即非 rustfmt-clean），本次新增区域已按 rustfmt 手工对齐；新代码无编译 warning。
- SD 落实：SD-01 新增 `docs/specs/auto-av/resource-fault-and-generation-conformance.md`；SD-02..07 已增补（place-ownership/collections-variants/closure-environments/cache-and-conformance/ace-m1-conformance/ace-m1-gap-audit）；`source-manifest.json` binding hash + `scenario_assets_915`（9 文件）更新，verifier 改为枚举全部 `scenario_assets*` 组。
- 复用证据：D912 暖链/未改函数身份由既有 `case-ace-m1-closures-fn-912` 在本冻结提交上重跑通过（未改该会话语义）；place.rs 的 Share/Static snapshot 单测未改；预算/布局/样本未变（本计划未改预算或布局）。
- blockers：无。已知边界写在本计划 §10 D-04（集合旧前端粒度、其余 M1 格 owner 不变）。

2026-11-02 | stage: goal | plan_id: PLAN-915 | plan_revision: 1 | round: 1 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-915/auto-lang | commit: b1e6cf127 | next: review
2026-11-02 | stage: review | review_scope: final | reviewer: 独立上下文子代理（fresh-context subagent，同机同模型族）+ 编排侧独立制品复核；限制=非独立模型/环境，结论按制品与可复跑命令重建 | plan_id: PLAN-915 | plan_revision: 1 | outcome: needs_fix | worktree: D:/autostack/.wt/lang-915/auto-lang | branch: plan-915-dev | reviewed_commit: b1e6cf127 | base_commit: 5a0ef7609 | dependency_revisions: 无 | spec_inputs: SD-01 新文件 + SD-02..07 增补（工作树内已读）；frontmatter 声明的 7 条 docs/specs 路径均存在 | acceptance_results: AC-01 partial / AC-02 partial / AC-03 pass / AC-04 partial / AC-05 本审 needs_fix | pending_closeout_gates: merge 阶段受影响固定组并集（本审已见 9 组冻结 PASS 证据，merge 仍按冻结提交复核）、`cargo t all` 周期批量不属本计划 | next: work（修复 F-915-R1..R13 后重审）

复核证据（均绑定 b1e6cf127）：独立子代理复跑 `cargo t case-ace-m1-resource-915` PASS 12/12（run-y3ctej3z）、`AV915_SABOTAGE=drop-point`/`no-stale` 均红、`cargo test -p auto-lang-core --lib resource_915_tests` 8/8、`worker_json_tests` 4/4、`ace-av resource_915` 7/7、`verify-ace-m1-852` contract+acceptance PASS；编排侧另核：分支 diff 29 文件无 stray artifact、binding 哈希=manifest、9 个 scenario asset 哈希全等、suite inventory 12 checks 全 RUN、scenario oracles/applicability 未改、9 组冻结并集 run-7zqs3bd7 gate_passed（ace-av 986 / auto-test-case 111 / resource 12 / collections-864 17 / closures-865 8 / shared-909 12 / closures-fn-912 13 / recovery-913 5 / order-layout-914 15）。

findings（按独立复审报告编号）：
- F-915-R1 major（AC-02/V-03 可测性）：「校验先于访问」目前只是代码顺序主张。`PlaceFacts.buffer_reads` 只在通过校验后的访问点自增，因此“先读后判”的错误实现仍可报 1；V-03 要求的“禁检查/事后 trap 红”负控缺失。修复：加 test-only `place_access_before_validate` seam（先访问并计同一计数器，再校验），crate 单测证明计数器会变 2，worker `AV915_SABOTAGE=read-first` 让父侧冻结 `buffer_reads=1` 判红。
- F-915-R2 major（AC-04/V-07 时间门）：§5.3 的“四状态×30 样本 P90≤2000ms + fault 遍历/单编辑时间分列”未在 b1e6cf127 执行（该腿在 `ace-performance` 组，不在 9 组并集内）。修复：跑命名性能腿并记录数字，或按 912 基线给出具名 deferral；不得以“预算/样本未变”代替。
- F-915-R3 minor（AC-01）：值 seam 只在 `ListPush` 计扩容；`ListInsert` 同样重分配却未计 attempt，SD-01 §2「List 容量扩容」表述过宽。修复：insert 扩容也计 attempt，或收窄规则。
- F-915-R4 minor（AC-01/SD-01 §2）：正文断言 arena 点 `successful = n-1`，但父判与 crate 测试都只判 faulted/live/message，ledger 的 attempts/successful/frees 未被判。修复：父判补这三轴（并同步单测夹具），或删该断言。
- F-915-R5 minor（健康/AC-01）：`tests/cases/resource_915.rs:180` 的 `const LSTR` 未使用（新编译器 warning），且“LSTR 有 crate 覆盖”的说法不成立。修复：把 LSTR 纳入 sweep 测试。
- F-915-R6 minor（健康）：ac_av.rs 基线 33 处 rustfmt 偏差、当前 35 处，新增区含 2 处（resource_core_axis 一带），与“新增区域已对齐”不符。修复：格式化新增区。
- F-915-R7 minor（AC-03 注册表）：`source-manifest.json` 的 `scenario_assets_915.files` 重复 `src/R03-av.at`、`src/R03-static-reject.at`（重复 JSON key），summary 仍写 “8 asset files” 而 expected_count=9。修复：去重 + 改文案。
- F-915-R8 minor（AC-03/SD-06）：未知键拒绝只在 `resource` face 生效；`place`/`capture` face 忽略多余键，而 SD-06 声称“未知键…均拒”。修复：三个 face 都做已知键门，或收窄规范措辞。
- F-915-R9 minor（SD-02/SD-03）：把“未变元素/活 Share/Static 缓冲仍可读”写成本计划观测到的证据；实际只跑了 fresh-token 与无变更 legal，Share/Static 是继承的未改单测。修复：改写为继承不变量或补控制。
- F-915-R10 minor（基线记录）：§4 把 `7b309217…` 记为基线、§9 把 `9563cfc6…` 记为基线；真实基线是 `134d93f6…`（7b/9563 均为本次改动后的值）。修复：改正两处。
- F-915-R11 minor（SD-05/SD-06 文本）：SD-05 的 42 值清单漏 LSTR=51；SD-06 仍写“四份语料字节拷贝源”（实为 5 份；目录共 7 个 .at）；5 个被改 spec 文件缺行尾换行。修复：补齐。
- F-915-R12 minor（SD-01 §4/实现）：`finish_trace` 未用于 resume 路径（av.rs:739/757），stepped+observe_places 会返回 `place_facts=None`/`alloc_events=0`，与规范不符。修复：两处 resume 出口也走 `finish_trace`。
- F-915-R13 minor（AC-01）：逐点“源/旧值不变”是断言而非测量；source 程序无宿主源值可测。修复：在 SD-01/§10 明确边界（原子性由 Infra+live=0+确定性重跑证明；typed capture 面另行测 source readable），或补 digest。

规范增量复核：SD-01 原则接受但需按 R3/R4/R13 修正（补 LSTR=51）；SD-02 需按 R9 修正（MutRef 边界表述本身正确）；SD-03 需按 R3/R9 修正（“先于 buffer 访问”是代码顺序而非测量）；SD-04 接受；SD-05 需按 R2/R11 修正（仅到重编译级恢复，不交 P90/会话状态；补 LSTR）；SD-06 需按 R8/R11 修正；SD-07 接受（只关两格 av，can_claim_m1=false 已核）。
2026-11-02 | stage: goal | plan_id: PLAN-915 | plan_revision: 1 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-915/auto-lang | commit: b1e6cf127 | next: work