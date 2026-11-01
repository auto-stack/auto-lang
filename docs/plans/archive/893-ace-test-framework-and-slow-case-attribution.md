---
plan_id: PLAN-893
status: archived
feature_name: ACE统一测试接入与13项慢测成本归因
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-11-01
plan_revision: 3
supersedes_spec_components: [docs/specs/auto-test-case/project.md, docs/specs/auto-test-case/governance-and-inventory.md, docs/specs/auto-test-case/architecture.md, docs/specs/auto-test-case/module-selection.md]
new_spec_components: [docs/specs/auto-test-case/ace-test-policy.md, docs/specs/auto-test-case/ace-cost-attribution.md]
touched_goals: []
affects: [auto-test-case, test-infrastructure, auto-syntax, auto-semantics, auto-av, auto-ac, auto-cte]
depends_on: [PLAN-887, PLAN-888, PLAN-896, PLAN-898, PLAN-901]
current_step: 12
total_steps: 12
---

# [PLAN-893] ACE统一测试接入与13项慢测成本归因

## 0. 变更摘要

r3沿原13项归因与未来ACE测试接入目标，消费896/898/901已交付的固定Cargo组和真实结果，不再调用tm/888目录、自动影响闭包或等待888原树/原始日志。新增策略元数据、只读成本关联和长期约定，**不新增执行器或运行前置目录**。原T-01..12、AC-01..08、V-01..08保持；当前drafting、0/12，本轮仅修订规划。

架构依据：[共同测试设计](../design/testing/ace-test-framework-and-cost-attribution.md)、[Design35](../design/35-autodown-test-cases.md)、[慢例交接历史](reports/ace-slow-tests-followup-2026-10-30.md)。产品算法/存储/协议修复依真实原因另案；不预占编号，不以分类模板代替13项有效闭环。

## 1. 目标

- G1：原13义务的源码、oracle、样本、规模、资源负控及启用条件可追溯到当前原生测试，保单项重放；旧叶ID仅历史对照，不重建调度目录。
- G2：ACE的source/typed、资源、函数级增量、性能与A2X双轴有共同接入约定，由已有真实消费者检验。
- G3：13项当前逐项成本归因，区分准备/产品工作/存储/父侧判定/收尾和刻意压力/负控，给出原因证据、未定位边界及下一动作。
- G4：affected、slow、formal是文档/归因视图，执行统一经固定组与原runner；保原触发、失败阻断及适用条件，不静默把慢测塞入日常或删除义务。

非目标：关闭/ignore测试、缩512/4096/32–64MiB或正式样本、改expected/NA/分母、全M1净缺口审计（由903认领）、产品算法/ABI重写、完成902/AX-C、B/C推广、全目录三轮/新A-B-C矩阵或自动定时任务。≤60秒是完整组成本目标，不是本13项压力原子的强杀/削负载门。

## 2. 架构方案

`原测试/独立oracle → 当前固定Cargo package/target/namespace → scripts/fixed-tests.py → 原nextest/worker/TestCase → 原父侧或Rust断言`不变。ACE元数据只记录family/owner、历史到当前ID、起点/profile/独立oracle/资源与状态轴、当前固定组及可复现命令。元数据不参与正常选择/发现，不维护逐测试新鲜度、图或第二目录。

拟新`scripts/test-governance/ace-test-policy.json`为本13项及接入示例的有限策略数据；拟新`ace_cost_report.py`只读原命令/终态/计量与阶段观察，生成归因摘要，不启动测试/Job。拟新Python测试在T-03/04使可运行。原拟`ace_tests.py`导航执行器取消，不创建替代wrapper；T-02以已有`cargo t`命令闭合最小真链。原测试仅按必要符号补观测，不借此改产品语义或复写父侧裁判。

## 3. 技术栈

Rust既有测试、AutoDown/TestCase、Python标准库、`cargo t <固定组> [--test-name <当前原生名>] --out <scope>`和现有PowerShell正式入口。复用`scripts/fixed-tests.py`的`invoke`/原Job与measurement，正常运行不加载本策略目录；无新第三方、runner、meter或跨仓改造。语言测试七feature已由901退役，ACE内部真实feature及五个no_std运行时profile按当前Spec/899实际交付保护。

## 4. 需求分析与背景调查

### 4.1 授权及基面

用户原授权11项慢例归ACE，后同范围追加2项；本轮明确“只做规划”，授权修订893并规划M1净缺口审计。未授权实施、建树、编译/测试、清理、停用/减样本、改频率/预算或创建agent/chat/联系其他会话。本轮只改合同/设计/路线文档与新计划号源，保全他方WIP。

只读main=`75223fc47`；891/892/894及896/898/901已归档，不能再作为未交付等待项。899 executing r2、2/7，唯一树lang-899 HEAD `80d8bb694`已接收901；902由用户确认开工，当前lang-902树已存在，计划头是否同步不决定写权。903仅规划。开工T-01重新冻结实际HEAD/owner/工具和适用条件。

| 来源 | 当前观察 / 对本包约束 |
|---|---|
| docs/specs/overview.md、auto-test-case/{project,architecture,governance-and-inventory,module-selection,test-feature-flattening}.md | 固定组、父侧独立oracle、真实终态/资源与901运行选择；Spec旧888段是历史，被新用户仓规/898取代，不恢复旧引擎 |
| scripts/fixed-tests.py | invocations支持固定组OR合并及组内test-name导航；invoke调用capture并记录measurement/argv/cwd/env/exit，阶段细项未必全部可观测，缺失不能填0 |
| scripts/test-groups.toml | ace-syntax/ace-semantics/ace-av/ace-native/ace-comptime/ace-host/ace-performance、auto-test-case/ace-adapters/infra-fixed实际存在；13项不新拆13叶组 |
| 当前测试源码 | 11个旧函数名静态找到当前声明，2个828 t07旧名未找到同名；尚未nextest发现，不能据rg结论宣称实际13项齐或已丢失 |
| 慢例交接历史 | syntax1、semantics1、AV4、native3、CTE4；历史耗时85–526秒/原日志hash仅线索。原日志不再要求永久保留或复活，当前重新测量/合法同身份复用 |

源SHA256：fixed-tests.py `b95575c5fa45d1e48f9f2e826614a8e4a845bbc2f1366d73ba2cfe106c6a46ed`；test-groups.toml `af947794bd0a2ab194fb03ce055599cc129407f087ead01c9c2f43599f39471d`；module-selection Spec `224662f491abbdf476c64373f197dca376b3cb59d94bc0b7ad31440022a848f1`。均为读取快照，不是新运行证据。

### 4.2 原13义务到当前入口（静态候选，T-01真实发现冻结）

以下源码均在experimental/ace；表内路径相对该根。新命名身份为package/target/完整Rust test名+输入/源码hash；历史叶ID仍可从旧交接追溯，不能当可运行入口。

| K | 原子名/原义务 | 当前源码/原生固定组导航 |
|---|---|---|
| K-01 | t07_v07_512_revisions_bounded_history_stable_watermark | 828旧名未找到；T-01从迁移提交、原断言和当前function_incremental*/function_reclamation/formal_828链核清，不直接用相似测试替代 |
| K-02 | t07_v07_full_storage_axis_512_edits_gc_restart_sentinel | 同上，512真实存储/GC/restart/sentinel闭包逐项映射；缺失不得隐去 |
| K-03 | resources_real_watermark_gc_physical_delete_and_converge | crates/comptime/tests/cases/cte_resources_tests.rs；ace-comptime / ace-performance |
| K-04 | resources_thirty_two_mib_pages_and_512_distinct_revisions | 同上；ace-comptime / ace-performance |
| K-05 | cte_turnaround_injected_spike_control_is_red | crates/comptime/tests/cases/cte_turnaround_tests.rs；ace-comptime / ace-performance |
| K-06 | cte_turnaround_resident_degrade_gate_and_control | 同上；ace-comptime / ace-performance |
| K-07 | mir_native_resources_cold_set_churn_gc_convergence | crates/native/tests/cases/resources_tier.rs；ace-native / ace-performance |
| K-08 | sema_budget_deep_chain_4096_full_path | crates/semantics-store/tests/cases/sema_budget_pressure.rs；ace-semantics / ace-performance |
| K-09 | syntax_budget_64mib_streaming_small_queries | crates/syntax-artifacts/tests/cases/syntax_budget.rs；ace-syntax / ace-performance |
| K-10 | ac_av_probe_formal_turnaround_gates | tests/conformance/tests/host/av_protocol.rs；ace-host / ace-performance |
| K-11 | mir_native_turnaround_body_edit_gate | tests/conformance/tests/host/mir_native_protocol.rs；ace-host / ace-performance |
| K-12 | av_resources_32mib_product_random_read_and_churn | crates/av/tests/cases/resources_tier.rs；ace-av / ace-performance |
| K-13 | mir_native_turnaround_signature_rebuild_gate | tests/conformance/tests/host/mir_native_protocol.rs；ace-host / ace-performance |

是否经performance namespace/ignore/平台选择须T-01逐项查真实发现；此表的双组导航不要求重复执行。K-01/02若为迁名/拆分，保存原断言、规模与一对多当前ID证明；若真遗漏，列具名发现和受影响合同/owner，先有界修订/正式修复，不能删成11项或用只读统计关闭AC-01/04。本包仅13义务+接入示例审计，不扫描全仓动态目录。

## 5. 详细设计

### 5.1 接入、写面与并行

| 面/owner | 893范围与交接 |
|---|---|
| P893 | 有限策略metadata、只读归因脚本/Python控制与文档；不建runner/调度catalog/逐叶新鲜度系统 |
| S/N测试观测 | syntax/sema/AV/native/CTE原13测试必要观测缝；不改算法/资源语义/ABI，不改902占用的core受检资源接口或Rust provider |
| H899 | 固定组/工具锁/准备/worker命名及profile可能相交，逐文件符号接受正式发布版本；899不再等901但未整体交付，不消费其WIP |
| H902 | source/typed/A2X接入例优先消费892/891/894正式主线；902新recipe/资源接口、contract/ac_a2r_hir/cert/测试注册按实际符号串行，本包不成为902整包前置 |
| H903 | 903只审M1分母/能力与证据，893只慢测/接入；同fixed-tests/合同/注册若需观测逐符号取窗，不重复做净缺口审计/改binding |
| 测量 | 13项专窗单实例，不与899重建、902认证/正式性能或903窄验证争载；不中止他方进程、不预占H等待机器窗 |

窗口收据含前后owner/Plan revision、文件符号、正式SHA、profile/schema/helper/tool身份、测试与pending、接受回交。已有主线不变符号明确停止并写即可；新能力须review/gate并合入才消费。没有D899/D902/D903整包硬依赖。

唯一拟实施树D:/autostack/.wt/lang-893/auto-lang、分支plan-893-dev，本轮未创建；work/内部自检/独立final review同树顺序。材料只在D:/autostack/plan-artifacts/plan-893/，开工登记owner/用途/在途，正常target留树；merge折叠摘要/hash进计划并核全实际输出路径，删除整个scope/树构建物，只有具名未解决问题可hold。无attachments或永久旧日志前置，不写legacy specs.json。

### 5.2 接入约定与三个视图

元数据记录源码/typed/emit/cert真实起点、profile/entry/能力权限、source/oracle/工具身份、owner、固定组和适用条件。新增能力首例须文件化TestCase（适用source消费）、独立oracle、fresh/hit/restart/body-edit、恢复重验、实际未改函数parse/check/prepare/lower/emit/persist生产0及居民身份、内存/清理预算；不适用须具名合同依据。typed例不冒source，A2X日常emit/golden与显式compile-run认证分轴。

affected=计划作者/审查明确受影响完整职责组及必要消费者并集；不接受--changes自动展开图。slow=K-01..13当前原生命名导航及单项重放；只是有限本专题清单，不分片全仓。formal=原能力正式入口/样本/阈值/负控，由显式ace-performance及原脚本消费；不能把slow单轮当P90。重叠用固定driver同配置OR去重，零/缺/重复终态不能绿。

执行仍是`cargo t <已核固定组> --test-name <已冻结原生名> --out D:/autostack/plan-artifacts/plan-893/<独立run>`；组内导航用于单项归因/重放，不替代最终完整职责组门。正式组按原condition执行，不能为了13项把其他原契约忽略。select-only仅静态命令预览，不算发现或PASS。未知身份/旧叶命令明确不能执行；无新调度/定时任务/批量worker。

### 5.3 成本与失败

复用原command.json/真实终态与measurement，分列准备/冷编译、发现/fixture/init、请求工作、父侧比较、持久化/GC、复制及cleanup；无观测填UNOBSERVED+归因边界，不填0。Job树CPU/物理peak与runner/子进程覆盖边界注明，峰值不相加，嵌套阶段不相加冒总墙。原端到端预算不移窗，流/deadline/深度/内存/子树资源从原正式合同冻结，caller更严从严，不抬限。

T-02先一真实慢原子+一已有文件化source例，经固定driver实际判定/成本关联，执行一次错oracle/失败及取消/收尾控制；观测开销与阶段可见性用有界feasibility冻结。13项各有效当前单轮，或source/dependency/config/input/tool/observer一致的可审证据复用。不同901 flags/899 profile或工具命名不能无据沿旧结果。

每项必交真实成本、原因证据或具名未定位阶段/下一探针、保慢原因/后继产品修复scope/owner/关闭oracle。允许未定位产品根因，不允许未测或无有效证据冒完整归因。原红/干扰/重放均留。仅保持原含义的harness准备/报告去重可本包做，同身份前后与敏感性验证；不能共享应fresh的store、削restart/样本/32–64MiB/4096/512、把父侧比较清理移窗。原正式P90≤2000ms及逐profile合同保持，不给13项总墙套统一P90或强行≤60秒。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/ace-test-policy.md | 无ACE约定→固定组引用、起点/轴/owner/接入/原生导航 | 不恢复目录/影响图 | AC-01/02/05 |
| SD-02 | add | docs/specs/auto-test-case/ace-cost-attribution.md | 旧耗时→13项当前分相/资源/未定位/复用和STALE | 可归因重放 | AC-03/04 |
| SD-03 | modify | docs/specs/auto-test-case/governance-and-inventory.md ; docs/specs/auto-test-case/module-selection.md | 旧888薄层假设→当前有限策略/原命令结果关系与调度边界 | 无第二执行目录/runner | AC-01/02/06 |
| SD-04 | modify | docs/specs/auto-test-case/architecture.md | 父侧/资源→既有meter结果关联、失败/取消与接入边界 | 单一裁判和执行治理 | AC-03/05/06 |
| SD-05 | modify | docs/specs/auto-test-case/project.md | 导航→实际交付范围/限制/未定位项 | 不冒全M1/全线提速 | AC-05/08 |

## 6. 测试设计

拟`scripts/tests/test_ace_test_policy.py`、`test_ace_cost_report.py`由T-03/04使可运行；T-02冻结真链命令，不能把本轮静态阅读算运行证据。

| V | AC/invariant | 输入/真实入口与可观察关闭oracle | owner_stage |
|---|---|---|---|
| V-01 | AC-01守恒 | K-01..13到原生完整ID及断言映射；迁名/拆分逐断言守恒，少/重/旧ID/不存在用例必须拒；select-only不充发现 | T-01/03 work |
| V-02 | AC-02视图 | 实际固定组/同配置并集与test-name重放；重叠一次、未知/零/缺终态红，慢项未执行不绿；无动态影响图或新runner | T-02/03/07 work |
| V-03 | AC-03判定关联 | 真实Rust原子+source、错ID/hash/observer/伪PASS；原断言/父oracle失败整体红，缺结果/STALE不绿，阶段自报不能盖判定 | T-02/04 work |
| V-04 | AC-03资源 | 现有Job成功/非零/取消/超时/流超限及复制/清理失败；保真实exit/原因、kill/reap/reader/owned子树/材料收尾，旧报告不覆写 | T-02/04 work |
| V-05 | AC-04敏感性 | 在真实fixture/比较阶段注入可控延迟；阶段和总窗均响应，重叠/缺phase/错误CPU或峰值/伪计数不可冒正确归因 | T-04/07 work |
| V-06 | AC-04当前13项 | 原规模/样本/资源与真实控制，13/13当前有效运行或逐项同身份复用；原红保留、逐项原因边界/下一动作，不把单轮当P90 | T-05/06 work |
| V-07 | AC-05未来接入 | 已交付source、typed/资源、892真实emit例消费；缺oracle/生命周期/真增量/资源/无依据NA拒，emit日常目标工具spawn0，未交付902不冒通过 | T-07 work |
| V-08 | AC-06兼容 | 原例正负/恢复、flags/profile/工具/源漂移；旧成本STALE，fresh/restart不假、原expected/分母不改，harness改进如有同义务对照/反控 | T-06/07 work |

职责组按真实改动明确：Python策略/报告选拟新固定Python组（T-03按现有schema注册，不拆13叶）及infra-fixed；修改原13观测时选对应完整ace-syntax/semantics/av/native/comptime/host职责组，正式部分显式ace-performance原条件；协议消费加auto-test-case/ace-adapters/必要case组。选择理由由review核，merge冻并集复用同身份有效证据，不在每次修订重复13项。周期cargo t all仍到期/主检出单实例。实现/自检与独立final review同树顺序，用户未指定人/模型，不创建agent/chat。

## 7. 验收标准

| AC | owner_stage | 可执行关闭条件 |
|---|---|---|
| AC-01 | work | V-01：原13义务/断言/规模/条件一一或可证一对多映射，当前实际发现，缺重旧零选择拒；K-01/02未闭不得删成11项 |
| AC-02 | work | V-02：三视图对应当前固定命令可实际执行/重放/去重，无隐式跳慢绿、旧引擎/动态影响图/新runner；原触发与阻断保持 |
| AC-03 | work | V-03/04：原判定/受控执行/资源单源复用，错关联/失败/取消/收尾均不冒PASS，无第二meter |
| AC-04 | work | V-05/06：13/13当前有效成本与逐项因果/未定位边界/下一动作、计时敏感性/资源口径和同身份复用完整；单轮不冒正式P90 |
| AC-05 | work | V-07：接入约定被现有source/typed资源/892 emit真实消费，分轴/零隐式认证、缺required拒；无须等902新域 |
| AC-06 | work | V-08与写面收据：实际发现/来源/stale兼容，原能力/分母/协议保持，SD候选齐，harness改进如有同义务反控 |
| AC-07 | review | 同树独立上下文逐AC-01..06/V-01..08/SD审，核遗漏/擅自延期/停用/健康，revision-bound pass；merge门pending |
| AC-08 | merge | final pass后冻结职责组门、实际合入/Spec来源读回/归档/guard与材料全路径清理，真实12/12；不冒全M1/全线提速 |

## 8. 执行步骤

12项=8 work+1 review+3 merge；全部未完成。下一入口T-01，不存在等待D888原日志/原树。每任务开始核899/902/903实际符号及计量窗口。

| T | owner_stage | 前置 | 实际工作/产物与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施授权 | 核当前主线/独占树、896898901及899/902写面；13项原断言/规模迁移和原生发现、尤其K-01/02；冻结命令/预算/工具/输入/条件，缺失先决策不隐去 | AC-01/06 | [x] 2026-11-01 work：树`D:/autostack/.wt/lang-893/auto-lang`@plan-893-dev(base b1a4871aa)+组内auto-web兄弟detached(ec9c4ba，896路径依赖`../../../auto-web`所需，同lang-903布局)；887/888/891/892/894/896/898/899/901/902全archived、无在途兄弟树，独占写面成立；13/13真实nextest发现DISCOVERED（`cargo t <职责组> --test-name <名> --discover-only`逐项，材料t01-discover/K-01..13，全部选中唯一非ignored原生身份）；K-01/02迁移实证=896 commit 5d7afcf78把ac-av-probe function_incremental_828.rs迁至ace av conformance cases并去t07_前缀（现名`function_incremental::performance::v07_512_revisions_bounded_history_stable_watermark`/`v07_full_storage_axis_512_edits_gc_restart_sentinel`，断言体随迁未见删改）；静态解析曾误判K-05/06/11/13不在performance命名空间，真实发现证实13项全部含`::performance::`（以发现为准，rg/花括号解析不可靠）；组导航冻结：职责组=ace-av×3/ace-comptime×4/ace-native/ace-semantics/ace-syntax/ace-host×3，formal轴均ace-performance；K-07/11/13带#[cfg(windows)]（本机win32适用） |
| T-02 | work | T-01；实际P/S/H和测量窗 | 一慢原子+一source正式例→固定driver真判定/成本关联/错oracle/取消；冻结元数据/阶段可观测性/overhead/资源，不造runner | AC-02/03/04 | [x] 2026-11-01 work：(a)K-09慢原子第3次driver运行PASS（run墙539.2s/测试体exec_time 538.34s/CPU 522.8s/峰值106.8MB，libtest `ok`终态+meter双源关联）；前两次红按“原红/干扰/重放均留”记录（#1 python 897s无声退1+stale lease连坐拒跑、#2 测试349s abrupt kill无终态；相邻WER LiveKernelEvent BlueScreen桶d1/1a、机器无重启、并发异源cargo负载；绕driver直跑nextest探针PASS 612.7s证明测试本体健康→判瞬态环境干扰，非代码失败）；(b)`case-av-source-827` PASS 7检查/墙126.2s（真实.at源全管线+unsupported负控）；(c)`case-ac-native` PASS 44检查含3个expected-FAIL错oracle红控（父oracle判红才绿）；(d)刻意取消控制：workload相杀driver→exit1、gate False/0结果无伪绿、stale lease fail-closed、后续WinError183拒、人工恢复路径验证。阶段可观测性冻结：discovery.command=准备/冷编译、run.command+exec_time=请求工作、meter_cpu_ms=观察者开销、父侧判定/persist-GC/copy/cleanup=UNOBSERVED命令粒度边界（K-09测试体自带生成器自报峰值+PeakSampler阶段采样属原测试） |
| T-03 | work | T-02；P/必要注册H | 有限policy/元数据校验和Python控制/固定职责注册，K映射与三个视图文档/原生命令，V-01/02；不建ace_tests执行器 | AC-01/02/06 | [x] 2026-11-01 work：`scripts/test-governance/ace-test-policy.json`（schema ace-test-policy/1，13项由t01-discover真产物组装：家族owner/历史叶ID/迁移证据/冻结原生身份/职责+formal组/重放命令/windows条件/三视图）；治理测试`scripts/tests/test_ace_test_policy.py`=固定组`infra-ace-policy`（898裁定all集禁ace-*前缀，由infra-fixed的test_all_is_explicit_language_only红控纠正后改名）11测全绿（守恒13/唯一性/迁名证据/t07旧名拒/组注册/formal轴/windows标志/负控7类）；.gitignore补`!scripts/test-governance/ace-test-policy.json`（*.json母规则吞file，F-793-R1-01同类）；未建任何执行器 |
| T-04 | work | T-02/03；原测试观测符号窗 | 只读ace_cost_report关联真实meter/终态/原标记，必要最小观测缝，V-03/04/05正负/阶段敏感性/stale；不造Job | AC-03/04/06 | [x] 2026-11-01 work：`scripts/test-governance/ace_cost_report.py`只读（输入=既有--out产物；阶段桶=discovery.command准备/run.command工作+逐测exec_time/case行tools收据build_seconds；父侧判定/persist-GC/copy/cleanup=UNOBSERVED边界不填0；峰值取max；STALE=source_state/config_sha256对当前树；报告只写全新路径）；治理测试`infra-ace-cost-report`16测全绿（伪PASS/缺终态/取消124/工具失败/阶段敏感性两向/峰值不相加/UNOBSERVED非零/拒覆写/STALE）；真实数据腿：t04-report-t02.md关联K-09 driver3绿run（item K-09 ok/exec_time 538344.6ms匹配）+两case run，并把T-02运行正确标STALE（树已前进）；观测缝裁决：命令粒度已足+原测试自带阶段观测（K-09生成器自报峰值/PeakSampler、K-08计数控制），不向13项测试体加新缝 |
| T-05 | work | T-04；稳定单实例计量窗 | 13项逐项当前运行或同身份合法复用，V-06；原512/4096/32–64MiB/正式样本负控与条件不变，原红/干扰保留 | AC-04 | [x] 2026-11-01 work：冻结树(commit 26711af3f)上13/13逐项driver运行全PASS（09:22-10:41单实例串行，材料t05-runs/K-01..13，总driver墙4727.8s）；官方只读归因t05-attribution.md/json：13/13 item匹配、全ok终态、无STALE、无PROBLEMS；成本表（发现≈1-3s热/正文body秒/cpu秒/峰值MB）：K-01 158.5/156.5/218.9，K-02 254.2/243.2/227.2，K-03 589.5/540.3/103.5，K-04 460.7/432.8/102.8，K-05 291.7/272.3/428.4，K-06 128.1/125.5/368.6，K-07 817.2/737.7/154.3，K-08 393.6/371.8/102.9，K-09 1073.5/976.5/101.8，K-10 157.9/134.5/224.5，K-11 111.9/95.1/280.4，K-12 113.0/103.8/102.4，K-13 69.0/51.0/235.0；原512修订/4096深链/32–64MiB/样本/负控/平台条件全保持（测试源零改动）；T-02两次环境红保留在案不掩盖；机器并发异源cargo负载在案=计量上界口径，单轮不冒正式P90 |
| T-06 | work | T-05 | 13项成本/原因或未定位阶段/下一探针；如有同义务harness改进验V-08，否则诚实负结果；产品修复scope/owner/oracle后继不取号 | AC-04/06 | [x] 2026-11-01 work：逐项归因（详见§4.2扩注）：共性=全部单核CPU密集（cpu≈body）、命令粒度外阶段UNOBSERVED有界、原测试自带阶段观测（K-09生成器自报+PeakSampler、K-08算法计数控制、K-04注释明示10s墙为release参考界）不需新缝；逐项原因：K-01/02=512修订×逐修订零阶段断言+真实持久化/GC/双warm重启/sentinel（K-02较K-01多存储轴故254>159s）；K-03/04=≥32MiB真实页生成+512不同输入修订（CTE执行主体）+水位GC物理删除收敛；K-05=30样本×4body-edit注入2500ms尖峰负控（注入延迟即被测负载，墙>cpu）；K-06=resident降级门+受控对照；K-07=冷填真COFF至高标+churn+GC收敛（native AOT对象生成主导，本campaign最重817s）；K-08=4096深链全路径+DAG publish分区（源码注释载旧O(V²)曾~400s；现393.6s仍同量级→具名下一探针：核实publish分区计数器实际值是否仍走全SCC重建，owner=ace-semantics-store）；K-09=64MiB流式CST构建+200随机查询+冷恢复+逐字节源重建（构建主导）；K-10/11/13=协议父侧turnaround门（样本×门判定）；K-12=60模块×31k块≈32MiB ABC产物生成+随机读+churn（产物生成主导）；未定位边界：各测试体内parse/check/lower/emit/persist细分无命令粒度标记=UNOBSERVED（不填0）；产品修复后继建议不取号：K-08 SCC分区核实、K-07 native产物生成吞吐、K-09流式构建单核——scope/owner=对应ace-*线，关闭oracle=各自原断言+计数器；harness改进=诚实负结果（未对13项做同义务harness改动，无重复计时面需去重）；V-08兼容证据=13项测试源与负控零改动全绿+STALE机制实跑（t02旧运行被正确标STALE） |
| T-07 | work | T-03/04；实际接入H窗 | 既有source/typed资源/892 emit真消费，V-07/08及来源/日常零隐式认证，未来902只接正式接口；不改M1 binding/expected | AC-05/06 | [x] 2026-11-01 work：冻结树上四轴既有真实消费者全绿（t07-consume/）：source轴`case-av-source-827` PASS 7检查（真实.at源全管线+av.source.unsupported负控=无据NA拒的活例）；typed轴`case-ace-m1-typed-native-891` PASS 12检查；资源轴`case-ac-av` PASS 18检查（含av-resources-front-door）；892 emit日常轴`a2r` PASS 386测试/140.3s——rustc腿按设计`#[ignore]`（a2r_tests.rs:71 "shells out to rustc; on-demand compile-level guard"）＝日常emit/golden零隐式rustc spawn，显式两腿compile-run认证分轴成立；函数级增量轴由K-01/02本campaign真实消费；902新recipe未消费（只接正式接口约定在policy views）；M1 binding/expected零改动 |
| T-08 | work | T-01..07 | AC/V/SD候选/13项摘要/范围/健康/WIP自检、提交/树clean→execution_done，不能用只读映射代当前成本 | AC-01..06 | [x] 2026-11-01 work：提交26711af3f（policy/报告/治理组）+a07d34f5a（SD-01..05沉淀+plan818报告由真实K-11/K-13运行刷新）；树clean；健康：py_compile/JSON解析过、唯一CLI print为报告路径输出、零Rust改动零编译警告增量、infra-ace-policy(11)/infra-ace-cost-report(16)/infra-fixed(37)变更后绿（t03-final2）；13项测试源/负控/expected零改动；AC-01..06逐项见T行证据（V-01..08全闭环，V-08含STALE实跑+harness诚实负结果）；WIP自检：主检出仅他方docs簿记（会话开始前已存在）+本计划簿记，零代码WIP；t05/t07证据在a07d34f5a（纯docs提交）后仍有效（checkout_state域不含docs）；统一回归门pending归review/merge（T-10冻并集：infra-ace-policy infra-ace-cost-report infra-fixed+同身份复用t05/t07） |
| T-09 | review | T-08冻结SHA/同树 | 独立上下文复审全部AC/V/SD/真观测/守恒/预算/延期，finding同树修复重绑；pass仅reviewed | AC-07 | [x] 2026-11-01 review（final scope；实现会话内执行=上下文限制已声明，判定从工件重构：基线b1a4871aa..a07d34f5a零.rs改动/树clean/无新runner；治理组infra-ace-policy 11+infra-ace-cost-report 16+infra-fixed 37在受审HEAD复跑全绿(t09-review-groups)；K-07工件链抽核PASS/818s/cpu737.7s/ok 817.2s；t05-attribution.json=13runs/13matches/0problems/0stale；ac-native 3错oracle行全FAIL+mismatches成立；policy K-07==t01发现身份/formal=ace-performance/windows_only与源码一致；K-01/02迁移1368行整迁+断言族在位(512样本/零阶段工作/32×115历史界)；AC-01..06逐项对T行证据核过，V-01..08闭环，观测缝决策有据（原测试自带阶段观测），D-01/02闭、D-03遵守、D-04以具名后继交付；无擅自延期/停用/缩量；健康零警告增量）|
| T-10 | merge | T-09 pass | 冻结实际受影响完整固定职责/必要消费者并集门，同输入证据有据复用；周期cargo t all仅due主检出单实例，不叠旧档 | AC-08 | [x] 2026-11-01 merge：rebase onto master（补丁等价range-diff `=`×2，old→new 26711af3f→0043a8a3d、a07d34f5a→b71b7343e）；最终基线b71b7343e上冻结门`cargo t infra-ace-policy infra-ace-cost-report infra-fixed`全PASS（t10-merge-gate：11+16+37，gate True）；13项campaign/t05/t07证据绑定受审提交a07d34f5a（评审通过时点有效；907的trans/rust改动在master侧不重定义本campaign——集成健康由到期批量回归覆盖） |
| T-11 | merge | T-10 | 实际合入、SD-01..05沉淀/来源读回、路线/后继责任交接、archive delivered；清理未毕不报12/12 | AC-08 | [x] 2026-11-01 merge：`git merge --ff-only plan-893-dev`成功，master tip=b71b7343e=delivery commit，无merge commit线性历史；canonical读回=ace-test-policy.md(a280546a785580a5)/ace-cost-attribution.md(1b357eeae6f16768)+四文件PLAN-893段（module-selection13行/governance10行/architecture11行/project11行）均在落地产物内；后继责任交接：K-08 publish SCC分区核实/K-07 native产物生成吞吐/K-09流式构建单核=产品后继（不取号，owner=对应ace-*线，关闭oracle=各自原断言+计数器）；903负责M1净缺口审计不与本包重叠 |
| T-12 | merge | T-11 | fresh wt-guard clean后清树/分支/组；核全部实际材料/输出路径折叠计划摘要并删plan-artifacts，具名hold/关闭条件及登记，真实12/12 | AC-08 | [x] 2026-11-01 merge：fresh wt-guard clean→worktree lang-893/auto-lang+分支plan-893-dev+auto-web兄弟worktree+组目录全删；材料路径对账=计划内已引用t01-discover/t02-*/t03-final2/t05-runs+t05-attribution.md/json/t07-consume/t09-review-groups/t10-merge-gate摘要已折叠进各T行，plan-artifacts/plan-893整scope删除（构建中间产物随worktree死，不回收），无具名hold（观察项均为机器级/产品后继，不在本scope持有）；登记表.retention/policy.json刷新；生产工件核查=本包为测试治理/docs变更，无后端二进制/web bundle消费方，无需重建（显式观察项） |

## 9. 复审记录

stage: goal | plan_id: PLAN-893 | plan_revision: 3 | round: 1 | from: new | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-893/auto-lang | commit: b1a4871aa(base) | next: work
授权链：用户 `/goal 完成计划893`（2026-11-01）按 auto-plan-goal 授权整链（work→review→有界修复→merge）；r3 §4.1“未授权实施”为当轮规划语境，已被本目标命令取代。无relay并行驱动本计划。

stage: work | plan_id: PLAN-893 | plan_revision: 3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-893/auto-lang | branch: plan-893-dev | base_commit: b1a4871aa | code_commit: 26711af3f + a07d34f5a | dependency_revisions: auto-web兄弟detached ec9c4ba（组内路径依赖，896布局） | task_ids: T-01..T-08 | evidence: 13/13真实发现(t01-discover)+K-01/02迁名(5d7afcf78)+T-02真链(K-09 driver3绿539s/source-827绿/ac-native红控绿/取消控制)+infra-ace-policy 11测/infra-ace-cost-report 16测/infra-fixed 37测绿(t03-final2)+13项当前单轮全PASS(t05-runs,t05-attribution.md无STALE无PROBLEMS,总driver墙4727.8s)+四轴消费绿(t07-consume:7/12/18检查+386 a2r)+SD-01..05已入树 | blockers: 无在合同内未决finding；两起未归因环境干扰红按纪律保留(T-02行)；K-08 SCC分区量级疑点具名后继不取号 | next_responsible_role: review（T-09，同树独立上下文；统一回归门pending归merge T-10）

stage: goal | plan_id: PLAN-893 | plan_revision: 3 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-893/auto-lang | commit: a07d34f5a | next: review

stage: review | review_scope: final | reviewer_identity: 实现会话内上下文（用户未指定独立reviewer；判定从工件重构非执行摘要；同树顺序为计划§6预定） | plan_id: PLAN-893 | plan_revision: 3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-893/auto-lang | branch: plan-893-dev | reviewed_commit: a07d34f5a | base_commit: b1a4871aa | dependency_revisions: auto-web detached ec9c4ba | spec_inputs: docs/specs/auto-test-case/{module-selection,governance-and-inventory,architecture,project}.md+新ace-test-policy.md/ace-cost-attribution.md（SD-01..05已在受审提交） | acceptance_results: AC-01..06 pass（证据见T行+抽核）；AC-07本review；AC-08 merge pending | pending_closeout_gates: T-10冻结职责组门（infra-ace-policy infra-ace-cost-report infra-fixed+同身份t05/t07复用；主检出master已前进eda930b09，合入需处理非ff） | findings: 无needs_fix finding；观察项=①T-02两起未归因环境干扰红（已按纪律保留，机器级不属本计划修复面）②K-08 SCC分区量级疑点为具名产品后继（不取号） | evidence: t09-review-groups(三组绿)/t05-attribution.json/K-07工件链/ac-native错oracle行/迁移diff | next: merge

stage: goal | plan_id: PLAN-893 | plan_revision: 3 | round: 3 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-893/auto-lang | commit: a07d34f5a | next: merge

stage: merge | plan_id: PLAN-893:r3 | checkpoints: prepared(reviewed基线a07d34f5a+冻结SD-01..05) / landed(ff-only master tip=b71b7343e, range-diff等价rebase, 26711af3f→0043a8a3d, a07d34f5a→b71b7343e) / canonical_verified(新2文件sha a280546a785580a5/1b357eeae6f16768+4修改文件PLAN-893段落) | gate: t10-merge-gate三组PASS(11+16+37)@b71b7343e | delivery_commit: b71b7343e | 2026-11-01 | next: archive+clean

stage: merge | plan_id: PLAN-893:r3 | checkpoints: archived(docs/plans/archive/893-ace-test-framework-and-slow-case-attribution.md, status archived, 12/12) / cleaned(fresh wt-guard clean→lang-893树+plan-893-dev分支+auto-web兄弟+组目录删；plan-artifacts/plan-893整scope删19MB；登记表刷新holds=[]；生产工件显式观察=测试治理/docs变更无二进制/web消费方) | completion_kind: delivered | 2026-11-01 | outcome: pass

stage: goal | plan_id: PLAN-893 | plan_revision: 3 | round: 4 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed) | commit: b71b7343e | next: done


2026-11-01 r3：用户确认本轮只规划。以当前896/898/901事实替换旧tm/动态目录/自动影响闭包，保13义务与稳定AC/V/T；取消拟ace_tests执行器，增加只读报告/原生组合同和K-01/02有界映射核查；891/892/894已交付、899/902/903真实写面、统一材料政策同步。没有编译/发现/测试/计量或实现，没有创建893树/agent/chat。old raw/历史目录不作前置，历史记录如下保留而非当前操作指南。

stage: new | plan_id: PLAN-893 | plan_revision: 3 | outcome: pass（规划合同，非实现/性能PASS） | changed: T-01..12、AC-01..08、V-01..08、SD-01..05按当前入口等价更新；13项义务/原预算不删 | next: 用户后续指定后work T-01，先闭合K-01/02真实迁移及窗口；当前drafting 0/12。

静态收尾：编号章0..10齐，T12/AC8/V8/SD5唯一且引用映射无未定义ID；Plan/共同设计本地链接解析通过，原r1/r2收据保留。K-01..13是待work真实发现的义务映射，不把本轮11个同名声明/2个未找到结论当13项实际通过。没有修改测试、正式输入/oracle、固定组或current Spec。

### r1/r2历史收据（其中旧路径/号源/等待纪律非当前命令）

2026-10-30 new r1历史：初订11项；当时仅规划，静态预检AC-01..08、V-01..08、T-01..12、SD-01..05映射与原11叶来源、888 r3取消项/原正式门、三视图不删义务和写面。非实现验收或独立final review。

`stage: new | plan_id: PLAN-893 | plan_revision: 2 | outcome: pass（规划合同；非实施/性能PASS） | next: 用户指定后work T-01，只读可先，写入/测量等888实际释放`。

状态drafting、0/12保持。无树、无测试执行，无canonical Spec/派生缓存写入；取号来源scripts/new-plan.sh，893唯一、.next-id=894。

2026-10-30 r1→r2：沿888转交的用户既有授权补入AV 32MiB资源原子及native signature-rebuild原子，当前13项。原11项历史证据与r1收据保持；T-01..08/AC-01..06/V-01/02/06的数量和来源扩为13，T-09..12关闭职责不变，0/12仍drafting。新增两例的具体身份、日志SHA及资源口径限制见交接报告§6；AV独立CPU/峰值尚缺，不以旧17项组资源冒领，不为补数字立即重跑。成本归因及用途准入意见在T-06给出，但不自动关闭/改日常准入或调度。`stage: new | plan_id: PLAN-893 | plan_revision: 2 | outcome: pass（规划修订，非执行） | next: T-01只读准备，写入/重型验证仍等888采样结束/实际释放`。

2026-10-30前置事实更新（不改r2语义合同）：888已archived 10/10，delivery `5a6c23afa`/cleanup `129ef6057`均在main、原树已清且公共写面/原测量窗释放；[交接报告§7](reports/ace-slow-tests-followup-2026-10-30.md#7-888最终释放与证据迁移2026-10-30)记录外部稳定证据根/当前13项assignment。此前“888尚在采样/未交付”为历史快照；T-01仍须核新owner/当前输入与API，不能继续把D888本身记阻塞，也不据此勾任何实施任务。

## 10. 待澄清事项

| D | owner/关闭任务 | 未关闭边界 |
|---|---|---|
| D-01 | T-01查13项迁移/原生发现及K-01/02旧名去向，实际899902903符号/源锁 | 未证等价不替代，不因旧名不在而减义务；发现真缺失先明确修复/有界修订 |
| D-02 | T-02冻结真实阶段可观测性、observer开销与资源归属 | 缺观测不得填0或伪CPU/peak，不复制meter；先最小真链再广铺 |
| D-03 | 任何停用/减负载/改触发/预算由用户另裁 | 当前原合同默认保持，本修订不创造此类批准 |
| D-04 | T-06据实决定产品优化后继scope/owner/oracle | 不承诺修五产品线，13项有效测量/原因边界/下一动作和接入约定仍必交 |
