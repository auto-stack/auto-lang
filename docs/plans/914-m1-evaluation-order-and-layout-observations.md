---
plan_id: PLAN-914
status: executing
feature_name: M1命名求值顺序与布局场景观察
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-02
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-gap-audit.md, docs/specs/auto-test-case/ace-m1-scenario-oracles.md, docs/specs/auto-test-case/a2r-golden-certification.md, docs/specs/auto-ac/function-incremental-source.md, docs/specs/auto-av/cache-and-conformance.md]
new_spec_components: [docs/specs/auto-test-case/m1-order-layout-observations.md]
touched_goals: [GOAL-002]
affects: [auto-test-case, auto-ac, auto-av, auto-lang/trans]
depends_on: [PLAN-908, PLAN-911, PLAN-912]
current_step: 1
total_steps: 10
---

# [PLAN-914] M1命名求值顺序与布局场景观察

## 0. 变更摘要

承911/912已交付，为原M1 scenario E01/E02四路径8格、I04/I09的ac/av四格建立真实producer→有界观察→严格父判→逐格证据链，合计12格。只规划drafting r1、0/10；不建实施树/材料、不运行编译/测试/探针/认证、不改当前Spec/原source/expected/binding/913进度。

按[Design43 §17](../design/43-ac-av-typed-mir-and-abc.md#17-m1下一阶段场景观察av资源与旧a2r2026-11-02target-state)拆有消费者的四包；915=AV资源两格，916=旧A2R静态八格，917=旧A2R发射两格。本包不吞R01/R04/R06清理事件、native R02/R03/R05、B18 avm或913来源恢复。

## 1. 目标

- G1：E01值7/trace精确ba；E02值103/trace精确ba，四路径各自保真实生产身份与原始观察。
- G2：I04真实layout依赖精确失效、无关函数不生产；I09活状态不被新布局重解释，当前明确拒绝/重启策略有运行反证。
- G3：父侧仅认同操作、当前source/config/producer/依赖/执行事实，不能预置trace或缓存PASS；原NA/accepted/cert双轴不改。
- G4：现有支持域的函数增量、恢复重验、预算/正式周转与下游消费者保持，最终只按12格真实证据回填。

非目标：native闭包/Dyn ABI、在线loader/OSR、资源事件全谱、完整A2R认证/旧产品切换、修D-909-1、全局runner/cache重构。

## 2. 架构方案

新文件化scenario fixture（明确新增，不伪称原src/E01存在）→各真实AC/AV/AVM/旧A2R入口→同操作值/trace或layout决策事实→既有TestCase adapter/产物→父侧严格比较→M1正式父报告。新观察通道只承载事实，不把oracle交给worker。新私有模块优先，公开结构/版本变动须核全部consumer。缺生产能力须在T-02最小全链暴露并needs_replan，不能只交“gap调查完成”。

## 3. 技术栈

Rust；现有auto-test-case、auto-lang-core adapters、experimental/ace独立workspace；文件化AutoDown .case.ad/.suite.ad、现有contained Job与显式ace-certify。沿原固定cargo t组，不建新调度器。

## 4. 需求分析与背景调查

### 4.1 授权、基面与来源

用户授权提前规划下一阶段几个计划，继承M1裁定/预算；本轮只文档、不启动其他会话或施工。main=0e93e3cc9；D911=ca4848f5d（archived10/10）、D912=13a465336（archived12/12）；913实际已开工T-01，头drafting滞后不动。最新binding/applicability静态复算34阻塞；本包不是完整M1验收。

| 来源 | 本轮内容SHA256 / 观察 |
|---|---|
| crates/auto-lang/test/cases/ace-m1/binding.tsv | 67bf520d26e7d541f8fd5f44b8c218da9d40b67d3fd808b4b3e801b89ec47fb9 |
| docs/specs/auto-test-case/ace-m1-scenario-oracles.md | c0539b968ba59034d09dce6fc6169c76a97888524482c47f15b092b07b75ce3a |
| docs/specs/auto-ac/function-incremental-source.md | 863ac6a042579edc84ca1459ce92b5f98527115a83a2f0171712cbe687432068 |
| applicability.json / source-manifest.json（同语料根） | 74c9109bbe7ec8596e6fa1bbc2ab9f4f5640725a748d0e63e53eb5e07ee275c0 / 34d8512b43b11e23dc551e6220288d8748a5d997b337da11171014bc17ca1fca |

### 4.2 已读代码与真实差距

- auto-lang-core/src/test_cases/{ac_av.rs,ac_mn.rs,a2r.rs,avm.rs,mod.rs}为当前宿主；旧probe目录已退役，不用旧路径造第二入口。
- av-core/src/av.rs::ExecTrace/execute_program含真实events/dispatched/代际；owned_text.rs已有alloc/free事实，但本包不把其当完整R族事件已接线。
- native/src/worker.rs及typed_session.rs消费trace_lib；已有native runner≠I04/I09正式布局场景已接。
- a2r.rs::run_check真实调用transpile_rust_with_source_dir/transpile_rust_project，仅发射；不能在日常腿偷偷编译Rust获得trace。现有M1标量证书不能直接假装包含新trace。
- scenario Spec冻结E01/E02语义但无对应原src；新输入/手写reference须由T-02先验证，不能凭历史av.core.trace-order文字就称当前recipe可运行。I04/I09当前支持策略亦须最小链实证。

## 5. 详细设计

### 5.1 12格合同与最小入口

E01/E02的ac/av/avm/a2r各4格；I04/I09仅ac/av各2格，ns=scenario。原值/trace/拒绝oracle不变；fixture使用普通命名调用/构造，不能把参数预先排好序再测。命名参数/字段调换、嵌套副作用、合法同类型近邻与强制重排应可区分；标记函数只承担冻结场景副作用，不扩console profile8产品承诺。

I04必须观察真正layout读依赖；变更相关字段/类型后重编或按已冻结不兼容策略拒绝，旧成功状态仍安全，无关函数真实parse/resolve/typecheck/plan/bind/verify/lower/encode/codegen/load/工件IO=0、对象/代际同进程不变。T-02必须给至少一个真实依赖失效场景及拒绝/非布局body编辑的对照；不能靠“一律reject任何type编辑”关闭I04。I09持旧值/pin进行不兼容候选，必须在使用旧值的新布局危险读前拒绝；宿主重启只可作为明确策略结果，不能冒在线HMR。

### 5.2 事实、父准入及A2R双轴

每次事实绑定case/check/operation/source/config/工具/profile/实际deps/进程与执行序；只从实际执行点读trace、实际依赖失效点读layout决策，包含失败/取消attempt。父准入拒重复key、错类型/族/顺序/身份、旧artifact/空选择、静默截断与缺tool；expect不入worker事实生产。完整parent报告及artifact哈希读回后才回填目标token。

a2r仍旧TransRust身份，不用892/902/904替代。首次E01/E02真实actual+独立手写reference的值/trace行为认证为显式操作，必须先actual==人工审阅golden。新增私有场景证书域或等价严格绑定，包含全部actual文件/工具/flags/deps/输入/预期及双侧实测值+trace，不能改旧ace.m1.cert/1字段含义。日常只emit+bytes/证书轴核查，rustc/cargo/目标Rust spawn=0；错误、panic和缺证书不满足Auto负例。生成漂移只产proposal、不自动改golden。新schema由T-02冻结，坏/错/缺字段强制拒，旧证书不授新trace权。

### 5.3 恢复、资源与交接

fresh/hit/restart/body-edit各有文件化首例、独立oracle、失败后修复/下一次读取；支持的AC/AV暖路径未改真实P=0，恢复decode/重验R与元数据M另记，不能用比较提交数冒P0。A2R旧cold emit如实计量，不虚构函数增量能力。原正式样本与规模不缩，已有source域P90≤2000ms（四状态各30有效样本，原更严门保留），准备/冷编译/执行/清理分列；显式Rust认证编译120s/运行30s独立，不归入日常P90。

新观察≤4MiB/操作、stdout/stderr≤1MiB/流、原更严cap优先；超额非PASS。AV值8MiB/受管32MiB与父增量128MiB、深链/512 churn原门保持；有界artifact不带host指针或序列化authority。新增场景规模/输入哈希/可达函数及至少3未改函数在T-02冻结，不能用空函数计P0；已有正式大样本仍原样。

913的closures_source_recovery/manifest/root/lease/restore/accepted-context不写。P私有fixture/父比较可先做；S需core依赖/前端/av-core观察时与913按实际符号单写；H=test_cases/{mod,ac_av,ac_mn,a2r,avm}.rs、auto-test-case、fixed组、binding/manifest全文件单写。915资源API、916/917旧A2R adapter交集亦取窗；owner+SHA/版本+构造/匹配/全部consumer+验证/释放入计划，接口再变STALE受影响证据；无D913整包前置，正式采样错峰。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/m1-order-layout-observations.md | 无正式12格观察→真实来源/父准入/场景合同 | 单源可审计 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | gap→有父证据才逐格重绑 | 不凭token绿 | AC-04 |
| SD-03 | modify | docs/specs/auto-test-case/ace-m1-gap-audit.md | 记录本包已闭/真实残余 | owner分明 | AC-04 |
| SD-04 | modify | docs/specs/auto-test-case/ace-m1-scenario-oracles.md | 语义oracle保持、补实际输入/入口/身份 | 可恢复验证 | AC-01/02 |
| SD-05 | modify | docs/specs/auto-test-case/a2r-golden-certification.md | 标量旧schema不变、新场景证据隔离 | 双轴不混 | AC-03 |
| SD-06 | modify | docs/specs/auto-ac/function-incremental-source.md | 补真实layout依赖与零生产支持边界 | 不冒完整语言 | AC-02/05 |
| SD-07 | modify | docs/specs/auto-av/cache-and-conformance.md | 补场景恢复/准入证据边界 | 不缓存PASS | AC-02/05 |

## 6. 测试设计

用户未指定模型/代理分工；work自查→同一专属树独立上下文review→merge，不能把自查当最终复审。本轮所有V未执行。新例拟置crates/auto-lang/test/cases/ace-m1/order-layout-914/（新增），case/suite、src/typed、独立reference均长期跟踪；固定组case-ace-m1-order-layout-914为拟新增，由T-05接可运行首例。

| Case ID | AC / invariant | input / entrypoint | observable oracle / distinguishing control | owner_stage / task |
|---|---|---|---|---|
| V-01 | AC-01 命名调用 | E01各真实生产入口 | 7+ba；强制声明序ab/只返7/旧trace复用均红 | work T-02/03 |
| V-02 | AC-01 命名构造 | E02四路径 | 103+ba；字段声明序/二次求值红，近邻正控 | work T-02/03 |
| V-03 | AC-02 layout依赖 | I04 ac/av真实类型编辑 | 相关依赖正确失效、未改P/IO0/对象代际；全编译后filter红 | work T-02/04 |
| V-04 | AC-02 活状态 | I09旧值/pin+候选 | 不重解释、危险读0；正确兼容近邻接受，强制旧值新layout红 | work T-04 |
| V-05 | AC-03 父准入 | 坏类型/重复key/错source/deps/op/旧工具/超额 | 一律非绿，有效非零id仍准入 | work T-03/05 |
| V-06 | AC-03 A2R双轴 | 新场景actual/reference+日常snapshot | 显式双侧值/trace认证；日常Rust spawn0；改golden/伪旧证书/expected入worker红 | work T-03/05 |
| V-07 | AC-05 恢复/预算 | 四状态/失败修复/原规模/30样本/原压力 | 结果/零生产/重验/资源/P90同过，原门不缩 | work T-04/06 |
| V-08 | AC-04/05 交接与回填 | 当前整合SHA、实际12格父报告 | 漏/错格/部分成功/过期报告不全绿；913及其他binding列不动 | work T-05/06 |

最终固定组并集按真实触面：ace-source ace-semantics ace-lowering ace-av ace-native ace-native-worker ace-adapters auto-test-case ace-host infra-fixed infra-ace-m1，以及case-ace-m1-base/nb-diags-908/native-diags-911/collections-864/native-collections-889/closures-fn-912；修改旧路径再加a2r/vm及必要parser/infer/ownership。新场景组明确加入。ace-performance只按受影响原命名正式腿、保样本/条件，和913错峰。T-01冻结取舍及实际consumer理由；不能只测新增checker或无端全experimental。

## 7. 验收标准

| ID | owner_stage | observable closing condition |
|---|---|---|
| AC-01 | work | E01/E02八格真实值/trace与区分性控制通过V-01/02，输入与reference独立 |
| AC-02 | work | I04/I09四格真实布局/活状态/未改P0/恢复和强制红控制通过V-03/04 |
| AC-03 | work | 同操作严格父准入/预算/A2R显式认证与日常零编译通过V-05/06 |
| AC-04 | work | 仅本12格有正式报告后更新binding/manifest；原source/expected/NA/分母不改，V-08 |
| AC-05 | work | V-07/08及受影响完整组/消费者/健康门有效，913窗口与支持边界真实 |
| AC-06 | review | 独立逐AC/V/SD与遗漏/延后/workaround审查绑定冻结SHA，所有必需项pass |
| AC-07 | merge | 正式落地主线、SD读回/归档/全部输出对账与guard清理；材料摘要折计划，不签整个M1 |

## 8. 执行步骤

| T | owner_stage | 前置/写面 | 工作、验证与产出 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施指令；D911/912；913/915/916/917实际H/S窗 | 核基面/12格、hash/实际consumer、原更严预算与完整组，登记唯一树/材料/窗口 | AC-01..05 | [x] |
|  |  |  | 基面主检出 3140cceb4；binding/applicability/source-manifest SHA256 与 §4.1 三项逐一相符；D911 ca4848f5d、D912 13a465336 均为 HEAD 祖先（exit0）。树 `D:/autostack/.wt/lang-914/auto-lang` 分支 `plan-914-dev`（2026-11-02 建，基面同上）。材料唯一 `D:/autostack/plan-artifacts/plan-914/`（owner plan-914，用途 T-01/T-02 探针与后续 fixture 认证副本，在途）。913 窗实测：其工作树 `lang-913` 写面为 ace av/src/{closures_fn_session,closures_session,closures_source_recovery,lib}.rs、av/tests、core/src/source/{closures_context,closures_fn,mod,resolve}.rs、core/src/verify.rs、core/Cargo.toml，与本包写面（test_cases/{mod,ac_av,ac_mn,a2r,avm}.rs、auto-test-case contract、ace-m1 语料、test-groups.toml、binding/manifest、specs）无交集，仅 av/src/lib.rs 相邻且本包无需改。915/916/917 尚无树/材料，无窗口冲突。固定组名与 test-groups.toml 对照有效；新场景组仍为 T-05 目标。原更严预算（≤4MiB/操作、≤1MiB/流、AV 8/32/128MiB、P90 四态 30 样本）未改动。证据：plan-artifacts/plan-914/t01-t02-probe-evidence.md |
| T-02 | work | T-01；私有P/必要短窗 | E01/E02四路径+I04/I09最小真实全链，冻结输入/布局策略/场景证据schema与强制红；缺能力须needs_replan | AC-01/02/03 | [ ] |
|  |  |  | 真实探针（master 3140cceb4 构建的 auto-case，suite 在 plan-artifacts/plan-914/probes，输出 out*）已给出逐路径能力矩阵：E01 源 `sub(a,b)=a*3+b*2` + `sub(b: mark_b(), a: mark_a())` 目标 7/"ba"；E02 源 `type Pair{left,right}` + `Pair{right: mark_b(), left: mark_a()}` + `fan=left*101+right` 目标 103/"ba"（声明序左先、构造右先，故 trace 可分辨声明序求值）。实测：ac(mn scalar) E01 exit7/stderr_hex 6261、E02 exit103/stderr 6261（值+trace 已真实，仅父侧缺 mn stderr 准入）；av(scalar) E01 exit7 且 receipt step.trace=["b","a"]（父侧无 trace 判据）、E02 需 data profile 且 step.trace=[]（记录路径 mark 未接）；ac 适配器 core-run 不吃 record（"type is not part of this source profile"）且 record worker 曾报 "missing stderr_b64" 协议缺陷；avm 命名实参仍按位置绑定（`sub(b:5,a:4)` 得 23≠22）、无 mark 内建（Undefined symbol）、print 记号为行式 "b\na\n"、capture API 无 stderr；a2r 旧发射器丢名（`sub(5, 4)`/`sub(mark_b(), mark_a())`）、无 mark 运行时、ace.m1.cert/1 只判 stdout。I04/I09 另见 T-04 前调查。→ 最小全链对 E01/E02 ac/av 可经父准入小改达成；avm/a2r 需旧引擎命名绑定修复与 trace 观测决定；E02 av 需记录路径 mark 接线。未改 oracle/输入之外的任何现有语料。 | 追加：旧路径通道用 `write`（无换行，native id 2900）即得精确字节 "ba"——avm 探针 `result=8`+`stdout="ba"` 已 PASS（值错仅因位置绑定），a2r 发射把 `write` 映射为 `print!` 亦无换行；故四路径 trace 都可按字节 "ba" 观测，不需为旧引擎新增 intrinsic。旧 avm 缺口收敛为一处语义修复：ParamInfo 需加 `name`（vm/codegen.rs:116、stmt_emit.rs:402），`expr_calls.rs:2850` 命名实参按源码序求值入隐藏局部后按形参序 load（保留位置回退）；E02 另需 expr_calls.rs:403 处 type literal 字段按名绑定（风险较高，须附正控）。旧 a2r 缺口为调用实参丢名（结构体字面量已保名，E02 语义已正确），需同一「源码序求值→按名落位」发射（临时量+块表达式）。
| T-03 | work | T-02；adapter/观察H/N | 实现真实trace与有界父准入、独立旧A2R场景证据；V-01/02/05/06 | AC-01/03 | [ ] |
| T-04 | work | T-02；layout实际S/N窗 | I04/I09生产/活状态/恢复/失败原子/真实计量与预算；V-03/04/07 | AC-02/05 | [ ] |
| T-05 | work | T-03/04；H/M1窗 | 正式case/suite/独立reference/新固定组与所有consumer、12格证据回填；V-05/06/08 | AC-03/04/05 | [ ] |
| T-06 | work | T-01..05；错峰运行 | V-01..08、相关完整组/正式原门/健康与遗漏自查；冻结SD/证据/提交→execution_done | AC-01..05 | [ ] |
| T-07 | review | T-06冻结SHA同树 | 独立源码→真实事实→父判→格位审查，修复重绑证据后reviewed | AC-06 | [ ] |
| T-08 | merge | T-07 pass | 冻结受影响组/必要consumer并集；有效同身份证据复用，batch仅due主检出单实例 | AC-07 | [ ] |
| T-09 | merge | T-08 | 合入/SD-01..07规范读回、真实窗口交接及残余owner、archive delivered | AC-07 | [ ] |
| T-10 | merge | T-09 | 全实际输出核查；摘要折计划、guard clean后清树/分支/组/整个scope并刷新登记 | AC-07 | [ ] |

拟树D:/autostack/.wt/lang-914/auto-lang、分支plan-914-dev，尚未创建。开工材料唯一D:/autostack/plan-artifacts/plan-914/，登记owner/用途/在途；探针/输出/认证副本/外部target只在此，不进attachments或仓内reports。merge全删，具名未解问题才登记hold；不动913/store/其他会话目录，不建链接。

## 9. 复审记录

2026-11-02 | stage:new | PLAN-914 r1 | outcome:规划合同待静态预检，非运行/独立最终review PASS。Design43 §17先设计/注册，再allocation.lock排他下new-plan.sh实际分配914..917，next918。12格与915两格/916八格/917两格互斥；不回开911/912。只文档规划，无实施树/材料/构建/测试/认证/agent/chat/current Spec或原source/expected/binding变更。后续执行自T-01，913不是整包前置；所有AC/V待实施。

2026-11-02规划静态预检通过：0..10节有序、T10/AC7/V8/SD7定义唯一且引用齐、10任务全未勾、drafting r1/0-of-10；现有Spec路径存在、新Spec仍未建立、本计划设计链接/anchor/空白检查通过。固定组名称对照test-groups.toml有效，新914..917场景组明确为T-05未来接线目标；D911ca4848f5d与D91213a465336均为main祖先(exit0)。四包各认领12/2/8/2格、无交集且都是当前34阻塞子集，剩余10格清单已核，非正式运行PASS。914..917新号各唯一、分配后next918，无对应worktree或plan-artifacts，原binding SHA256仍67bf520d26e7d541f8fd5f44b8c218da9d40b67d3fd808b4b3e801b89ec47fb9；本轮未改当前Spec/代码/脚本/913。所有语言功能、资源、时间、认证和最终review证据仍待执行。

2026-11-02 | stage:goal | PLAN-914 r1 | round 1 | from: drafting | to: executing | outcome: progress | worktree: D:/autostack/.wt/lang-914/auto-lang | commit: 3140cceb4 | next: work
T-01 已完成（hash/祖先/913窗口/树/材料登记见 §8 T-01）。T-02 用 master 同基面构建的 `auto-case run` 做了 7 轮真实探针（suite/输出留 plan-artifacts/plan-914/，摘要见 t01-t02-probe-evidence.md）：E01/E02 的 ac(mn scalar) 值+trace 已真实（exit 7/103、stderr_hex 6261），E01 av 的 step.trace=["b","a"] 已真实，仅父侧无 mn stderr/av trace 准入；E02 av 走 data profile 值 103 但 step.trace=[]（记录路径 mark 未接）；旧 avm 命名实参/命名字段按位置绑定（23≠22、203≠103），无 mark 内建、print 记号是行式 "b\na\n" 且 capture API 无 stderr；旧 a2r 调用丢名（sub(5,4)/sub(mark_b(),mark_a())）但结构体字面量保留字段名（E02 语义正确）；ac 适配器不吃 record 且曾报 missing stderr_b64 协议缺陷。未改任何代码/语料/Spec/binding；原 oracle 与 12 格分母未动。下一轮 work：冻结 E01/E02 四路径 fixture 与 trace 证据 schema（按路径归一为事件序），随后 T-03 落父准入与旧引擎绑定修复入口。

## 10. 待澄清事项

| D | owner / bounded closing condition |
|---|---|
| D-01 四路径trace最小链 | T-02冻结真实fixture/mark来源/新场景证书；不可拿模拟器或“缺能力已登记”关闭AC |
| D-02 layout策略 | T-02实证至少一个真依赖失效及活状态不兼容拒绝；若需扩大语言/ABI则needs_replan，不偷偷改oracle |
| D-03 913及其他窗口 | T-01/05逐符号正式SHA/全部consumer收据，漂移仅重核受影响，不互等整包 |
| D-04 残余M1 | ACE路线负责后继R族10剩格/B18/native边界及独立签章，未立项不预占号；本包12格必须全闭才交付 |
