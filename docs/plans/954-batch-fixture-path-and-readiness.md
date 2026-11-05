---
plan_id: PLAN-954
status: drafting
feature_name: 批量回归夹具的路径字面量与就绪证据
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
plan_revision: 1
supersedes_spec_components: [docs/specs/tool-execution/project.md]
new_spec_components: []
touched_goals: []
affects: [tool-execution, auto-lang-tests]
depends_on: [PLAN-953]
current_step: 0
total_steps: 5
---

# [PLAN-954] 批量回归夹具的路径字面量与就绪证据

## 0. 变更摘要

950触发的主检出周期回归发现两类夹具问题：Windows临时路径直接嵌入Auto字符串会把\r解释为回车；stdin非拥有后代负控在原250ms调用后等待心跳，批量时一次未观察到所需后代就绪。固定源码隔离重放绿不抹首红。本包只修夹具与证据有效性，不改产品时限或行为，不作为944的新前置。

## 1. 目标

生成Auto源码的文件路径必须保留真实路径值；超时/非拥有后代case必须证实所声明状态真实出现，同时保留250ms deadline、elapsed<1500ms、拥有/非拥有语义与退出回收断言。目标是在合法平台/输入下排除夹具造成的假红，而非提高阈值或略过失败。

非目标：修改lexer转义语义、HTTP文件实现、owned capture产品代码、全仓测试入口/ignore/资源门、复开已归档953、跨仓写入；本包不直接宣称批量all-green。

## 2. 架构方案

复用现有Auto字符串序列化和auto-tool-execution夹具/生命周期设施。T-01先核实际状态与original预算能否同时成立；未知机制留给有界探针，不预设“加sleep/增deadline”方案。没有第二套进程执行器。

## 3. 技术栈

Rust tests、原fixed cargo组、Windows x64原oracle、现有te-child和字符串literal规则。

## 4. 需求分析与背景调查

授权来源：用户goal944全链进入auto-plan-merge后触发auto-plan-regress，其Triage new reds要求建立fix-forward计划；本次只登记可复现的后继合同，未获本包实施授权、未建树/改代码/运行本包新测试。无模型/预算指派。Spec取材docs/specs/overview.md与tool-execution/project.md（953结束后最后字节及夹具实际退出规则）。

实际基线5b71983bad7973127f4be4d1e5c3829acc455d12（全量期间protected source/config未漂移，后继6b6b191c3仅debt文档）。D953已归档。观察来源为batch-regression-20261106主检出单实例cargo t all：37组/2828执行，27组PASS、7组FAIL、3组NA；新观察与旧媒体fixture/sys-native/宏编译/legacy ACE ownership/LSP/Python pilot红分开。

实际相关文件：crates/auto-lang/src/tests/plan349_tests.rs（download/resume/multipart路径format）、plan727_http_transfer_tests.rs（VM transfer路径format）；crates/auto-lex/src/lexer.rs（双引号字符串\r→回车，正确语言行为）；crates/auto-tool-execution/tests/exec_lifecycle.rs::deadline_covers_blocked_stdin_with_unowned_holder、wait_heartbeat_fixture_exit；src/bin/te-child.rs（既有夹具）。前两个HTTP测试文件/lexer/transfer实现在3b09e32b0至当前无改动；新增路径暴露是旧夹具输入生成缺陷，未证明HTTP实现回归。

## 5. 详细设计

路径闭环：真实Path→符合原Auto转义合同的literal→原run_with_capture→真实HTTP本地mock/文件内容；不能通过改目录名/选无转义路径作为永久修复。合法包含r/n/t开头分段、Unicode及普通控制对照；无效Windows路径不伪装合法oracle。

就绪闭环：原stdin-holder/非拥有后代→实际可观察状态→原run deadline/停止→原存活与实际退出。T-01分辨首红发生于创建、调度、heartbeat还是退出清理，记录确证/假说；若原条件不可能共存，提出确切冲突并请求原owner裁定，不能自行放宽250ms/1500ms/2s或给case加ignore。

### 规范增量

| ID | 操作 | target | before/after | 理由/AC |
|---|---|---|---|---|
| SD-01 | modify | docs/specs/tool-execution/project.md | 保留原产品生命周期合同；补明确夹具须证明被测状态实际发生，产码输入必须正确转义 | 避免把无效输入/未发生状态归产品失败；AC-01/02 |

## 6. 测试设计

原观察可直接重建（以下历史实证非本包修复PASS）：

- F-PATH输入：Windows TEMP/TMP=`D:/autostack/plan-artifacts/batch-regression-20261106/runtime-temp`。GetTempPathW/2W真返回反斜杠形式，包含\runtime-temp；原format直接嵌入双引号Auto字符串，lexer正确消费\r，最终文件路径无效。原全量3个349 case失败，727 transfer报staging create failed/os123。相邻合法对照将同scope叶名设scratch（仅诊断控制，不是修复），原`cargo t auto-lang --test-name plan349_tests` 12/12、`--test-name plan727_http_transfer_tests::vm`3/3通过；源码/断言/预算未改。
- F-READY输入：原deadline_covers_blocked_stdin_with_unowned_holder，stdin_data1MiB、250ms、DirectChildOnly、stdin-holder5s。全量后置wait_first_tick失败“grandchild heartbeat never started within 2s”，exec_time2.429s；同一固定入口单case隔离执行1/1绿（9.136s整命令）。这证明批量首红/隔离绿，尚未证明某个产品根因；不得以最终timeout码代证真正出现非拥有后代。

| Case | owner | oracle | 边界 |
|---|---|---|---|
| V-01 | work/T-02 | 原4个HTTP case在含转义分段路径成功、目标文件真实内容正确 | 原Windows profile |
| V-02 | work/T-02 | 产码literal对真实Path逐字符往返，普通路径合法对照；破坏escape负控变异实际触发原失败 | 只修fixture，不改lexer |
| V-03 | work/T-01/03 | 具名后代实际已出现、原deadline/elapsed/ownership断言成立、最终真实退出；不凭sleep充证据 | 原预算与调度条件 |
| V-04 | review | 独立复核前述原输入、合法近邻、丢项/阈值/ignore与Spec delta | 同一冻结树 |

完整职责组预定`cargo t auto-lang auto-tool-execution`，定向修复先跑原反例与相邻实际fixture；merge final后核受影响并集，身份未变证据复用。旧不相关红不删、不改本包AC，批量cargo t all另归原due门，不叠全仓或增加warm样本。首红/环境归因/重放绿分别留账。

## 7. 验收标准

| ID | owner_stage | 可观察标准 |
|---|---|---|
| AC-01 | work | 真实原4个路径case及literal合法/负控闭环正确，任意合法临时根不被误解释 |
| AC-02 | work | 原就绪/超时/非拥有/退出状态都被真实观察，原全部时限与断言保持；未发生状态不伪装产品通过 |
| AC-03 | review | 独立逐AC/遗漏/健康/Spec核验并绑定r1/SHA |
| AC-04 | merge | canonical源码一致、归档、guard/材料路径全部安全清理 |

## 8. 执行步骤

拟唯一树D:/autostack/.wt/lang-954/auto-lang、branch plan-954-dev；尚未创建。开工材料scope=plan-artifacts/plan-954并登记，所有探针/target副本只在scope，原batch原始证据摘要已在本Plan，无需保留旧raw。不得使用953旧树或master改代码。

- [ ] **T-01｜owner_stage: work｜AC-02** 有界核F-READY创建→heartbeat→deadline→退出真实时序，冻结可行修法/状态证据，原预算不可兼容则具名停止，不预设放宽方案。
- [ ] **T-02｜owner_stage: work｜AC-01** 原两个HTTP测试owner以原language literal合同修路径产码，V-01/02永久正负控及真实文件结果。
- [ ] **T-03｜owner_stage: work｜AC-02** 依T-01修fixture就绪与清理，V-03原预算/所有权与生命周期永久回归，保留批量/隔离分类。
- [ ] **T-04｜owner_stage: review｜AC-03** 同唯一树独立final，逐原输入/阈值/状态与delta审查，绑定revision/SHA。
- [ ] **T-05｜owner_stage: merge｜AC-04** final后冻结职责组/有效证据核验、落Spec/归档、guard clean删树/branch/scope，路径对账。

## 9. 复审记录

2026-11-06 drafting/new：源于950到期batch发现，未实现、未改变生产语义或原验证预算，0/5。原始全量是exit1，隔离1/1及scratch路径12+3绿均为诊断，不能把本计划checkbox提前勾选。旧raw由batch关闭按政策删，反例输入/错误/命令/预期已完整折入本文；后续按当前源码重测，路径反例用自身plan-954/runtime-temp等价重建，不复建已清batch scope。

`stage: new | plan_id: PLAN-954 | plan_revision: 1 | outcome: pass (draft execution contract) | next: future work authorization then T-01/T-02 | task_ids: all pending | scope: AutoLang test fixtures only`

## 10. 待澄清事项

本包尚未实施授权；由test governance/tool-execution owner后继认领，不成为944依赖。T-01负责原deadline与状态证据的可行决策；任何门限/支持集变化需另有明确裁定。路径转义和真实就绪状态目标已确定，无需借此扩写产品执行器。
