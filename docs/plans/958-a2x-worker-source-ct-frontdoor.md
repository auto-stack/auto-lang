---
plan_id: PLAN-958
status: reviewed
feature_name: A2X真实worker源CT前门接线修复
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
plan_revision: 1
current_step: 5
total_steps: 6
supersedes_spec_components: [docs/specs/auto-cte/source-query-integration.md, docs/specs/auto-test-case/a2r-hir-certification.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [auto-a2x, auto-a2c, auto-cte, auto-test-case]
depends_on: [PLAN-936]
---

# [PLAN-958] A2X真实worker源CT前门接线修复

## 0. 变更摘要

修复941真实验收发现的F-941-03：已交付936源CT在A2xSession层可用，但实际a2x-worker的Rust/C daily与certify四入口仍走普通scalar前端，含#的源均syntax拒。仅把实际worker共用scalar helper按已授CT capability委托既有source-cte服务，补真实CLI回归与认证；不重开归档936、不在941组合器模拟能力。

## 1. 目标

让原936七源经实际预构建worker的Rust/C daily发射，以及actual和手写reference独立编译/运行认证，保原独立oracle。七源是sum42=42、同调用点体编辑sum43=43、boollit主程序7、meta2=42/meta3=43、choose42=42、pick40=40。普通源仍接受，无CT capability的#源仍受控拒；原cold单次服务、预算和身份保持。完成独立复审、合入、Spec沉淀/归档/清理后继续941。

## 2. 架构方案

输入source/profile=scalar/caps → a2x-worker的compile_scalar_product_934 → 有CT capability走ace_source_cte::compile_scalar_ct（真实AV、结构回灌、事实车道），否则原source-query → 原Checked HIR/SemanticView → 原Rust/C emitter → 原daily或certify。新增直接依赖只连接现有无环source-cte服务，不造parser/solver/VM/store/runner/meter。CT服务生命周期是原cold请求owned，不在本修复扩张暖会话/恢复承诺；941原机制义务不核销。

## 3. 技术栈

Rust/现有ACE工作区；原a2x-worker CLI、serde_json严格解析、原受控rustc/MSVC compile+run、固定组cargo t ace-a2x。仓库仅auto-lang；auto-down兄弟只读，禁止链接。实施树D:/autostack/.wt/lang-958/auto-lang，分支plan-958-dev（本仓既有plan分支规约优先）。材料唯一D:/autostack/plan-artifacts/plan-958/，正常target在958专属树；TEMP/TMP显式在该scope，merge全删。

## 4. 需求分析与背景调查

2026-11-06用户已明确“授权独立修复并继续（推荐）”，对应941 §9的具体四入口接线提案和八次原正反操作；授权本独立最小修复从new/work/review/merge到继续941，无新增预算或缩样授权，不再就相同范围询问。

当前Spec overview及auto-cte/source-query-integration明确CT capability/scalar-ct面、共享真实AV服务与产品消费者；a2r-hir-certification明确daily spawn0及actual/reference显式认证。实际worker文件experimental/ace/crates/a2x-worker/src/bin/a2x-worker.rs的42行helper无条件普通source-query调用；Rust emit约207行/C emit约376行同走该helper；Cargo.toml尚无直接ace-source-cte依赖。原源在crates/auto-lang/test/cases/ace-m1/source-cte-936/src/*.at；两普通源控制和四#失败见941 scope/ct936-worker-entry-results.json，SHA256=5f9737861d33b613bc6f1b7d822b2d559f5137eaeb4bf68378bb6a575c919ba7。源CT四入口exit2 expected expression found #，同工具普通源四入口exit0（cert actual/reference PASS）。这是真实CLI失败，不以ct_lane_936发射文本含42的弱断言替代。

941新文字reader/reference变更仍在941树，不搬进958；本修复所需原936七源已在主线。主检出他方docs-only WIP保留。新计划按scripts/new-plan.sh原子取号958，不手工占号。

## 5. 详细设计

单点共用helper选择实际编译门：仅caps含ac.source.cte.scalar/v1时创建原MemoryEnvelopeSink/CtService并调用compile_scalar_ct，取同次产物checked/input_hash/atom_text构造原CompiledSource；诊断保持真实语言拒绝，PlanError映射已有source.cte.plan分类与消息。普通路径保现有SourceQuerySession。根据实际依赖增加worker的ace-source-cte直接边，离线最小更新ACE锁；不升级无关第三方包。名称可重命名以准确表达普通/CT统一入口。所有四操作都经此helper，不能只修A2xSession或只改Rust分支。

日常发射必须spawned_rustc=0、C还spawned_cl=0，完整fileset及source/recipe/profile/domain/hash核验；不能靠在daily中编译执行目标冒通过。认证使用原显式受控工具，原stdout为value:N或trap:Name，原stderr序列；不得用双方相等取代独立oracle。参考程序按七源的算术、分支、字段数与实例闭式结果分别手写，不读emitter或调用目标作为oracle。

原预算不变：daily原门；cert编译≤120s、run≤30s、命令≤360s；CT根10s/逐AV≤min(2s,remaining)，原caps/profile拒绝不放宽。资源态/语言拒绝/infra分开，不冒暖缓存或正式30样本已证明。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-cte/source-query-integration.md | 列产品消费者→明确实际a2x-worker四scalar操作也按CT cap委托同门 | 防Session/真实CLI割裂 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-test-case/a2r-hir-certification.md | 原通道→补CT源在实际Rust/C daily/cert入口的有限合同、独立oracle和拒绝控制 | 不以发射文本匹配替代真实认证 | AC-01..04 |

## 6. 测试设计

实施/内部检查/最终review按auto-plan分阶段独立上下文核查，同一958树顺序交窗；无自动子agent/新chat。补充测试是真实CLI，新的原生测试模块在worker/tests/cases下注册进现有conformance，或等价明确native集成target（Cargo autotests=false时须显式注册）；绝不写有名无入口的测试。使用原工作区工具/受控executor，不加通用runner。

| V | owner | AC/invariant | entry/input | independent observable/oracle | boundary |
|---|---|---|---|---|---|
| V-01 | work | AC-01四入口接通 | 七原源×Rust/C daily、certify | 14 daily完整发射/零编译spawn；14 certify actual/reference各自真实运行等于42/43/7/42/43/42/40 | 有限scalar-CT，28原操作，不减七源 |
| V-02 | work | AC-02前门权限 | 同sum42源删除CT capability；Rust/C两操作方向 | 受控syntax/能力拒绝，零目标compile/run；普通源两端daily/cert仍接受 | 不开放无capCT |
| V-03 | work | AC-03源/体/meta/实例/子CT身份 | sum42→43、meta2→3、choose42/pick40 | 实际运行对应原独立值；证书source hash/caps/profile/domain及文件hash正确，真实标准JSON解析 | 非暖缓存/恢复/正式性能证明 |
| V-04 | work | AC-04独立认证敏感 | 原合法样例改oracle或reference结果为相同错误值 | 认证拒，双方相等不铸绿；daily仍不接收oracle | 不改原expected以迎合输出 |
| V-05 | review/merge | AC-05/06健康职责 | 原ace-a2x完整组与新增明确入口 | 保所有原条件、warning基线与实际执行/ignored数；当前code/input身份冻结 | 不扩全仓/全ACE性能门 |

## 7. 验收标准

| AC | owner_stage | closing condition |
|---|---|---|
| AC-01 | work | V01完整七源四入口真实操作通过；actual/reference各对独立oracle，daily零目标compiler |
| AC-02 | work | 仅已授CT cap走既有CT服务，普通源兼容，无cap源受控拒，无新parser/VM/store/runner |
| AC-03 | work | V03具体变体与严格证书/完整fileset实际校验，无仅文本含42的代理证明 |
| AC-04 | work | V04真实反控拒绝，原预算/分类不放宽，新增入口永久可执行 |
| AC-05 | work | scoped检查/健康/SD草案齐，未处理缺口空；最终冻结完整ace-a2x并集归merge |
| AC-06 | review | 独立逐AC/源码/反例/遗漏/SD审查pass绑定r1和冻结提交，不信工作勾选 |
| AC-07 | merge | 冻结并集通过、交付主线、canonical读回/归档、guard清树/分支/组/scope与全路径对账，941消费稳定提交 |

## 8. 执行步骤

| T | owner_stage | dependency / scope | output / validation | state |
|---|---|---|---|---|
| T-01 | work | 原8次反例与当前source/spec | 在958树核前门/四入口/直接依赖；冻结原七源/caps/手算oracle与普通/无cap正反入口，AC01..04 | [x] 2026-11-06，见§9 |
| T-02 | work | T01 | 共用helper最小委托/直接依赖/离线锁；原服务/cold生命周期保持，AC01/02 | [x] 2026-11-06，见§9 |
| T-03 | work | T02 | 永久真实worker七源四操作与拒绝/错oracle回归，标准JSON与raw编译/运行证据，AC01..04 | [x] 2026-11-06，见§9 |
| T-04 | work | T03 | scoped验证/健康与SD草案/闭包记录，冻结提交交独立review，AC05 | [x] 2026-11-06，见§9 |
| T-05 | review | T01..04交窗 | 同树独立AC/反控/遗漏/SD核查绑定pass或具名needs_fix，AC06 | [x] 2026-11-06 final review，见§9 |
| T-06 | merge | T05最终pass | 冻结cargo t ace-a2x职责门、主线合入/Spec/归档/全路径scope清理guard退树，继续941，AC07 | [ ] |

## 9. 复审记录

2026-11-06 stage:new | plan_id: PLAN-958 | plan_revision:r1 | outcome:pass（执行合同，不是实施复审） | authorization:用户已授权独立修复并继续941 | source:941 F-941-03八次原操作与当前实际helper/spec | next:work T01（唯一958树）

2026-11-06 work closure | worktree:D:/autostack/.wt/lang-958/auto-lang | branch:plan-958-dev | base:a63a20b4e | dependency:auto-down detached58ea33f | code:**688207b67** | user authorization:独立修复并继续941（原回复已记录）。T01..04闭，4/6，execution_done；review/merge未执行。

- T01实际核查：原7源/原Case oracle、Spec CT前门与实际worker两个emitter共用helper；原941八次CLI反例保留SHA 5f973786...，没有重开936。新增直接source-cte依赖，冷请求仍ownedMemoryEnvelopeSink/CtService，原ordinary source-query分支保持。
- T02修复单点helper34行，Rust/C daily/cert四入口均经其分流；PlanError按已有source.cte.plan诊断翻译。ACE锁离线最小22行增加（已交付stdlib会员/依赖21+本次source-cte直接边1），无第三方版本升级。`cargo check --locked --offline -p ace-a2x-worker --bin a2x-worker --manifest-path experimental/ace/Cargo.toml --target-dir experimental/ace/target` exit0，18.81s；原ace-core first_lex_fact dead-code warning为既有1条，本改动无新增warning。
- T03永久module `tests/cases/worker_ct_frontdoor_958.rs`显式注册进autotests=false的conformance（不是未挂入口的ct_lane文件）。七个Rust native测试真实调用Cargo生成的a2x-worker exe；完整七源14 daily+14cert、4普通源/4无cap控制、Rust/C错oracle及错reference各2，共40原CLI操作；另外认证完成后经原owned executor直接跑32个现场actual/reference executable（14cert×2+普通2cert×2），每个独立对原手算raw value行/空stderr，不使用工作者PASS显示作唯一oracle。4拒绝路径由真实错误/无产物及实际编译链前置核查，不声称原worker已有raw编译阶段JSON。
- 第一次`ct-worker-regression`实际5测试3pass/2fail：测试误假定主线worker已有941在途的opt-in execution_observations，实为Null；这属于本测试前置误读，不是环境红。未把941未合入仪表代码搬进958：改为直接执行原worker在新独享目录中实际编译出的两腿PE，保原进程facility/deadline；再补完整cert普通/无cap及C/Rust错reference反控。最终 `cargo t ace-a2x --test-name worker_ct_958 --out <scope>/ct-worker-real-artifact-regression` exit0、**7/7**，29.553495s（准备/发现/运行分项在results），189原测试因本次具名过滤未运行，不充PASS；results SHA256=`8ff8425c432259270655f0b0845de2db11f37baa1414f4b2597a177e7205e3d7`。旧full ace-a2x冻结职责组归T06，未因过滤7项冒全组。
- T04：SD01/02两个Spec草案仅958树，Rustfmt定向与diff --check通过；提交688207b67后实际比对5个更改的非md源码/锁字节=Git blob（scope/committed-input-audit.json），原7源没有改。7门运行时源码与该提交相同，提交后仅Spec文字/索引身份变动；不为提交重复同一组。完整冻结职责并集等待独立review后由merge执行，不先铸pass。全部raw/stdout/log/临时引用/原正常target均仅958scope或958树；941原问题证据为只读父输入，owner941仍在途，不是958待删输出。

Ready for review: AC01..05工作闭包已具名验证；无未解决的本修复内功能/语义缺口，无预算放宽、无重复CT引擎、无新runner/meter。仅冷worker接线，不核销941的暖缓存/恢复/原30样本与其余M1义务。冻结树停止实现写入，下一责任角色review；仍需AC06独立逐条验证与T06完整组/合入清理。

stage: work | plan_id: PLAN-958 | plan_revision:r1 | outcome:pass | worktree_path:D:/autostack/.wt/lang-958/auto-lang | branch:plan-958-dev | base_commit:a63a20b4e | code_commit:688207b67 | task_ids:T01..04 | evidence:7/7 actual CLI tests,40 operations and32 raw executable replays; check/fmt/SD; original 8-operation failure | blockers:none within repair | next_responsible_role:review

2026-11-06 stage: review | review_scope:final | reviewer_identity/limitations:本会话新复审阶段，同一主agent，非另一个模型/会话；未获自动新建agent授权，不冒称不同reviewer。按r1/真实diff/原日志重新重建判定，并自选10项未在原回归中的holdout，不以工作总结作权威 | plan_id:PLAN-958 | plan_revision:r1 | outcome:**pass** | worktree_path:D:/autostack/.wt/lang-958/auto-lang | branch:plan-958-dev | reviewed_commit:**688207b6773e942df08f62a1776fcc8e256e64ea** | base_commit:a63a20b4e | dependency:auto-down只读58ea33f | next:merge（原用户独立修复并继续授权）

| AC | final review evidence / result |
|---|---|
| AC-01 | PASS：独立读CLI dispatch→Rust emit/C emit共用helper→CT服务→Checked HIR→原emitter/cert实际compile/run路径；复核7个真实native test的原nextest事件逐项ok、selected=executed=7、filter余189不充PASS。原七源手算42/43/7/42/43/42/40、14daily/14cert的源文件和cap表对应；每次认证后独立运行新独享目录中actual/reference两腿核raw行。日常原调用链无外部编译器；新holdout两个daily的Job process count=2（原meter wrapper+worker），不是只信spawn计数。 |
| AC-02 | PASS：精确相等CT_CAPABILITY分流，普通source-query旧分支未动；新增直接边指现有service，库/solver/AV/store/runner代码零改。独立holdout普通源授CT cap：两端daily+cert四项均接受；仅instances cap和CT/v10错误版本各两端daily受控syntax拒，零产物/Job2。证明不是“任意source cap即授权CT”或“有CT cap必须含#”的实现。 |
| AC-03 | PASS：原七源/body/meta/instance/child值证据对应实际原文件；独立核证书schema、source hash、scalar HIR域/caps/文件hash断言及standard JSON解析真实入口注册。SD只声明cold worker，不把Session文本含42冒原CLI或暖缓存/恢复证明。 |
| AC-04 | PASS：Rust错oracle实际42/ref42 vsoracle43仍REJECTED；错reference43 vsactual/oracle42仍REJECTED；C两相应受控oracle失败有真实断言。新增复审真实CT divide-by-zero两端daily均exit2 source.cte.plan/cte.trap/av.trap.div-by-zero，零产物/Job2，不在发射器忽略CT错态或mint证书。资源/编译/执行预算常量及原受控设施零改，编译120s/run30s/原根CT10s单AV2s未扩大。 |
| AC-05 | PASS：源码/锁/Spec干净且已提交；定向rustfmt和diff --check，唯一ace-core first_lex_fact warning为未改基线。TOML独立对账无第三方包版本/checksum漂移，仅新增既有stdlib-conformance本地成员及所需依赖边；其余21行是已交付manifest匹配而非新stdlib实现。7门发现/运行分别2612.2574ms/26906.1047ms，整命令29.553495s；只复用源/输入未变的原有效证据，不因role交接重复原7组。 |
| AC-06 | PASS：本次全AC/源码/原始日志及自选反控/规范差异审查，无须修项，无遗漏、未批延后或workaround债候选；确实无新语言/第二CT引擎。复审限制已明列，不冒不同模型会话。 |
| AC-07 | PENDING merge：完整cargo t ace-a2x、当前主线并发对账、合入/Spec读回/归档/guard清树与全路径scope删除尚未执行，不由review填绿。 |

复审只读实测10项在`review-holdouts/`，汇总SHA256=`f93dcd2d007e1ec91109a53737d0ae69d2f38af67b0107e8f7553fce5fa80475`。输入/输出/原Job meter真实保留：ordinary+CT cap四项exit0（cert PASS）；错误版本cap两项/仅instances cap两项exit2 source.syntax；真实CT trap两项exit2具名cte.trap；所有daily Job2、stderr空，反例均未建产物目录。没有改生产源码或测试输入迎合反控。当前HEAD/dirty再次核验仍688207b67/clean。

SD01/02均通过：`docs/specs/auto-cte/source-query-integration.md` SHA256=`eafa1082c933454dac0adeec25d70937c13c5a1a71a65e710304935c8526c208`；`docs/specs/auto-test-case/a2r-hir-certification.md` SHA256=`a32d3555b5657cb72c2ae909ce2ac000110662ef5aa46e218f96c7895561581a`。正好描述现有有限cap前门/同次HIR/日常spawn0与两腿独立oracle/冷会话边界；frontmatter两个supersedes路径、无new组件、GOAL002核对有效；零legacy specs.json/索引写。冻结Spec delta可由688207b67的二路径diff重建，不能仅信附件。

合入职责并集明确为**ace-a2x完整固定组**（a2x-core/rust/store/worker全部原条件与本次实际CLI）。触面是worker共用scalar consumer，source-cte/service与source-query/语法/AV/native生产者均未改；包依赖不推出所有依赖方测试，不堆ace-comptime或全仓/正式性能。先复审再由merge在冻结最终身份跑一次完整并集。当前无实现写入授权交还事项，进入merge。

## 10. 待澄清事项

无同范围待授权项。若发现需新语言/新CT引擎/放宽预算，停止对应扩张并记录具体问题；继续独立工作。941的暖CT复用/required页/正式样本及其他reader不由本修复冒通过。
