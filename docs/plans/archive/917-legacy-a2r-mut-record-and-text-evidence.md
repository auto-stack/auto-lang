---
plan_id: PLAN-917
status: archived
feature_name: 旧A2R的mut-record发射与文本金样接线
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-02
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-lang/trans/overview.md, docs/specs/auto-lang/trans/design/a2r-string-ownership.md, docs/specs/auto-test-case/a2r-golden-certification.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-gap-audit.md]
new_spec_components: [docs/specs/auto-lang/trans/legacy-rust-mut-and-text-evidence.md]
touched_goals: [GOAL-002]
affects: [auto-lang/trans, auto-test-case]
depends_on: [PLAN-908, PLAN-881]
current_step: 6
total_steps: 10
---

# [PLAN-917] 旧A2R的mut-record发射与文本金样接线

## 0. 变更摘要

认领corpus R07/T08的a2r两格：修旧production TransRust的R07未定型发射，原源mut参数写回真实42；核T08真实产物并独立审阅冻结文本金样、按已批准双轴接线。依[Design43 §17](../design/43-ac-av-typed-mir-and-abc.md#17-m1下一阶段场景观察av资源与旧a2r2026-11-02target-state)与916准入包分开，不吞七语义族或38格复合认证。drafting r1、0/10，本轮只规划，未建树/材料/测试/认证或改production/current Spec/原source/expected/binding。

## 1. 目标

G1：原R07完整实际Rust文件集类型正确、mut借用/求值/写回符合原语义，生成器与手写reference分别实跑42。G2：R07/T08独立审阅golden与稳定source/deps身份，日常实际emit+bytes核查零Rust编译；诊断/运行陷阱/认证欠账不混。G3：与916共用正式准入/入口，不以复制新HIR产物或改原源/expected补绿；失败原子/资源与全部旧consumer闭合。

非目标：916静态检查、旧A2R全复合认证、HIR新A2R迁移、mut实例方法/ext/泛型/新ABI、native/AV新语义、913恢复、全语言函数增量。T08行为认证如未实际完成保持独立cert-gap，不用Rust panic冒Auto slice-oob。

## 2. 架构方案

source→既有生产准入（916正式稳定接口可接，不依赖其整包）→RustTrans中参数模式/名义类型/返回与调用侧一致发射→实际文件集manifest→父侧实际emit vs独立审阅golden；显式R07 actual/reference认证另行执行，绑定全部文件/真实依赖/工具/flags。旧ace.m1.cert/1标量std-only schema不偷扩，复合新证据用独立版本/域并严格准入；旧证据/新HIR证据互不冒充。

## 3. 技术栈

Rust、auto-lang-core trans/rust.rs与TestCase a2r/ace_certify入口；既有a2r-std/真实运行依赖（如确实需要）、contained Job。AutoDown case/suite与独立Rust reference，cargo t固定组。

## 4. 需求分析与背景调查

用户授权下一阶段规划，无施工。main0e93e3cc9；D911ca4848f5d/D91213a465336已归档；913开发不动。R07 AV/native已P881=42，但这些不等旧A2R已交；902/904新HIR的成功不得替换旧a2r分母。242仅tracker。

| 来源 | SHA256 / 当前事实 |
|---|---|
| binding.tsv（ace-m1语料根） | 67bf520d26e7d541f8fd5f44b8c218da9d40b67d3fd808b4b3e801b89ec47fb9；R07=gap:a2r-untyped-emit，T08=gap:a2r-golden |
| docs/specs/auto-test-case/a2r-golden-certification.md | bd78434370a41394162b1641aa28be65d0231996e61a4a0db196d7e2e34ee706；日常零Rust编译/认证独立，旧schema范围标量10例 |
| crates/auto-lang-core/src/trans/rust.rs | 68f68d0dded4056893b5390d97cb0db176a00768186cfc9e4fed0165b07cbe37；current_fn_mut_params/fn_mut_params/fn_merge_mut_params已存在，不应重建另一套模式表 |
| crates/auto-lang-core/src/test_cases/a2r.rs | 32a4b98d753912df81c0cbacfbfe5ccbbffc4b4d09c3883507c71616a9b61a40；run_check实际发射完整产物，不编译Rust |
| docs/specs/auto-lang/trans/overview.md | fbe6a03274be7588ffd8cf7880ae6201735993269fd02f3aee1a1f31e9dbfe7f；需按实际旧producer写支持，不把新HIR范围混入 |

原R07=Counter{value int}、bump(mut x,delta)修改x.value，main以20调用+1，now+x.value=42。原T08="中".slice(0,2).len()，冻结语义slice-oob，不能改成ASCII或合法范围。T08原token从source-bound归一为gap，属于证据/发射面核查，不可仅把现输出复制成oracle。public single/project入口、128MiB emit线程/typed失败/contained认证原门继承；语义准入复用916，不在本包实现第二套checker。

原source SHA256：R07=6d07888a9383b7e8a7d4d6a50dabc961e0e2f35137cb9823c5cf0463e43b6c28；T08=f2db55ba67a6b4a77cc630c6c1643310a90d7856bc077fb9294c2b7c5c4dfc26。原R07只观测最终42；now/caller各21由独立新增近邻分别返回相应投影证明，不为观测中间值改写原源。

## 5. 详细设计

### 5.1 R07模式/类型/资源与完整产物

T-02最小真实source→生成所有实际Rust文件→编译/运行→reference先证可行，冻结支持矩阵与手算输出。命名类型/参数模式/返回/调用实参与写回使用同一事实，不能在callee加&mut而caller按值复制，或call后忘写回；只编过、只returned42、隐式clone掩盖alias均不足。R07同时观察now=21、caller.x.value=21、最终42，且参数求值恰一次。保Auto type值复制语义与实际mut路径，不全局把所有record改成引用。

补近邻覆盖原支持record的view/mut/move、两个不同根mut、同根冲突、返回/嵌套直接调用和默认/命名实参已支持形状；非法借用按916/现有规则由前端拒绝，不拿Rust checker拒绝当Auto负例。已有资源record支持必须保原move/clone/清理行为；不扩mut实例方法等新语法。T-02列明真实支持/当前合法域，需扩大语言或通用ABI则needs_replan，不删原R07。不得删生成声明、手修actual、用reference作actual、只剪取main来编译；wrapper/全部实际依赖明确生成与身份绑定。

### 5.2 T08与双轴

T08只按原源码/冻结slice-oob语义核查实际产物与golden判据，人工审阅应能解释Unicode scalar/边界操作；不能把本轮输出直接复制后自证。若产物真实语义错误导致金样不应批准，修对应发射/helper或needs_replan，不能冻结已知错误。若编译运行认证尚缺当前受控trap通道，M1按已批准emit/golden轴、认证gap另账；Rust panic/编译失败/exit非零本身不能充Auto trap认证，也不以例外把坏证书降成普通cert-gap。

R07新golden须独立审阅，显式真实actual和手写reference都得42；适配器读真实producer产物，不读expected来生成。T08若新增显式行为认证，必须另有已绑定工具/文件集/依赖/有区别的合法邻居和准确Auto trap原因。只声称实际完成的认证，不自动关闭其余38格。

### 5.3 日常、认证与失败恢复

日常emit、完整产物manifest、逐字节golden/证书身份核对，rustc/cargo/目标Rust spawn=0；任何漂移FAIL并仅产proposal，不自动覆盖golden/证书。显式认证沿CREATE_SUSPENDED+Job、编译120s/运行30s/每流1MiB及严格工具/deps/flags/actual/golden/source/reference/oracle/wrapper绑定。若std-only旧schema不能表达真实依赖，新增本域严格版本，旧消费者不能误收；缺/坏/陈旧证书分开，坏证书不以--force覆盖。新证书不得携oracle指导生成器，不对外发布/默认切换。

现有单文件/source-dir/siblings/full-options/project/CLI/SDK及旧金样消费者逐一核；生成失败/取消/发布故障不得污染已接受产物或写半有效manifest，明确commit阶段。失败→修复→再emit/snapshot/独立认证读取闭环，不缓存PASS或运行value。fresh/hit/restart/body-edit四状态纳文件化首例，但旧cold emit如实生产；不因未改源码虚称函数级P0。若改真正共享ACE/infer cache路径，原支持暖API须验真实未改P/IO0、代际与完整deps/CT权限，不能改名计量。

原source/元数据4MiB、每流1MiB、emit栈/实际profile更严预算保持，新文件集/证据元数据≤4MiB且先限额后物化，父峰值记录。正式日常emit周转四状态各30样本P90≤2000ms，原样本/数据规模/更严门不缩；显式编译认证耗时独立。owned资源清理不靠OS杀进程证明，超额/取消与语言trap区分。

### 5.4 写面交接与执行顺序

本包P=RustTrans发射辅助/新fixture/reference/证书域；916主准入、typed语义和公共wrapper。rust.rs同文件默认916先冻结准入接缝后交917；917可先设计私有发射/认证和反例，取已提交稳定阶段无需等D916整包。若916尚无接口，本包先沿当前生产入口完成正例，不擅自认领八负例或绕过未来准入；最终整合SHA重核V-01..08。

H=test_cases/a2r.rs/ace_certify.rs/mod.rs、auto-test-case合同、fixed组、binding/source-manifest、证书读者，全文件单写短窗，与914旧A2R场景证书隔离域/版本。913主ACE恢复，无整包前置；仅共有父合同/注册/实际共享推导取窗。每窗owner/revision/SHA/字段符号/构造匹配/全部consumer/已验/释放登记，漂移STALE受影响证据；不持写窗等认证/913采样、不停止其他进程。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/trans/legacy-rust-mut-and-text-evidence.md | R07/T08真实模式/产物/双轴/支持矩阵 | 明确生产边界 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/trans/overview.md | 旧A2R两格实现/证据与残余 | 不冒新HIR替换 | AC-01/02 |
| SD-03 | modify | docs/specs/auto-lang/trans/design/a2r-string-ownership.md | 补T08 Unicode/slice边界与实际emit证据，不改语义 | 避免错金样 | AC-02 |
| SD-04 | modify | docs/specs/auto-test-case/a2r-golden-certification.md | 新本域证据隔离、日常零编译/旧证书保持 | 严格双轴 | AC-03 |
| SD-05 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 两格凭正式父报告回填，cert单列 | 不拿emit冒运行 | AC-03 |
| SD-06 | modify | docs/specs/auto-test-case/ace-m1-gap-audit.md | 两格状态/证据与真实残余 | 不关闭38认证账 | AC-03 |

## 6. 测试设计

长期新目录crates/auto-lang/test/cases/ace-m1/a2r-emit-917/（case/suite/补源/reference/显式认证材料），原src/R07.at/T08.at只引用；golden新增/修改须独立审阅及语义理由，不自动取actual。拟新固定组case-ace-m1-a2r-emit-917由T-05落地，日常不隐式认证。work自查→同树独立上下文review→merge；所有V尚未执行。

| Case ID | AC / invariant | actual inputs / entry | observable expected / distinguishing control | owner_stage / task |
|---|---|---|---|---|
| V-01 | AC-01 原R07真链 | public single/project emit→完整实际文件集/显式认证 | now21+caller21+总42，各与reference相符；只return/忘writeback/剪actual/用reference替actual红 | work T-02/03 |
| V-02 | AC-01 模式/资源 | 已支持record模式/不同根mut/求值一次/合法与冲突近邻 | 正确别名/值复制/借用/清理；隐式clone/两次求值/全改引用红 | work T-02/03 |
| V-03 | AC-02 T08金样语义 | 原Unicode source真实emit及合法slice近邻 | 独立审阅产物+原trap义务，已知错误golden/改ASCII/把panic当Auto trap认证红 | work T-02/04 |
| V-04 | AC-03 双轴/身份 | 日常snapshot+显式本域认证与旧证书 | 日常Rust spawn0；坏/缺/陈旧分开；schema串域/漏deps/自动更新golden/伪编译红 | work T-04/05 |
| V-05 | AC-04 失败/恢复 | emit/发布/认证失败与取消→修复→下一读 | 已接受产物不污染，阶段诚实；复用旧PASS/旧值/半manifest红 | work T-04/06 |
| V-06 | AC-04 资源/时间 | 原规模/四状态30样本/超额/Job | 原cap/P90与正确性，编译/准备/运行/清理分列，无悬挂/静默截断 | work T-06 |
| V-07 | AC-03 两格父证据 | 当前source/工具/产物hash及正式报告 | 仅corpus R07/T08 a2r回填、cert缺口保真，旧报告/错格/改expected红 | work T-05/06 |
| V-08 | AC-01/04 916/914交接 | 全public入口/合法旧金样/所有证书消费者 | 无绕过916/串证书域，接口漂移STALE、全部consumer按整合SHA复验 | work T-01/06 |

固定完整职责组a2r auto-test-case infra-fixed infra-ace-m1、case-ace-m1-base/新917组；实际运行库依赖若触a2r-std须加其完整组，公共推导/ownership变更加infer ownership parser及真实转译消费者，T-01冻结理由；若真改ACE共享桥接再加ace-adapters/ace-a2x及必要下游，未触不补建worker。显式认证独立命令/产物/hash和退出码折计划，不改日常组为隐式rustc。

## 7. 验收标准

| ID | owner_stage | closing condition |
|---|---|---|
| AC-01 | work | V-01/02原R07真实完整actual/reference=42、模式与资源语义无旁路 |
| AC-02 | work | V-03原T08真实产物及独立golden审阅/合法近邻、认证边界诚实 |
| AC-03 | work | V-04/07严格双轴/两格正式证据/日常零编译，原source/expected/规则不改 |
| AC-04 | work | V-05/06/08失败恢复/预算/P90/全部consumer与职责组/健康及交接通过 |
| AC-05 | review | 独立逐AC/V/SD、actual来源/所有权/证据/遗漏复审绑定冻结SHA，无未批延期 |
| AC-06 | merge | 主线落地、Spec读回/归档/所有实际输出对账与guard清理，剩余认证/M1 owner保留 |

## 8. 执行步骤

| T | owner_stage | 前置/窗 | 工作与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施指令；916公共入口/914/913实际H | 冻结原两格/hash/模式/所有consumer/预算/职责组，唯一树材料及窗口 | AC-01..04 | [x] |
| T-02 | work | T-01；P/必要rust.rs短窗 | R07实际完整产物→42/reference及T08语义/golden可行性，冻结模式/真实deps/证据域；不可行needs_replan | AC-01/02/03 | [x] |
| T-03 | work | T-02；916释放入口接缝/发射R | 补参数模式/名义类型/调用返回与写回发射，完整合法模式近邻/失败，V-01/02 | AC-01 | [x] |
| T-04 | work | T-02/03；证书/父入口H | T08独立golden、日常snapshot、本域显式认证/严格reader/失败恢复，V-03/04/05 | AC-02/03/04 | [x] |
| T-05 | work | T-03/04；H/M1 | 正式case/suite/reference/固定组与仅两格父证据回填，全部consumer版本，V-04/07 | AC-03 | [x] |
| T-06 | work | T-01..05；认证/采样错峰 | V-01..08、完整职责/必要consumer与原时间资源/健康自查，冻结SD/SHA→execution_done | AC-01..04 | [x] |
| T-07 | review | T-06冻结同树SHA | 独立源→实际文件集→认证/金样→父报告审查，修复重绑pass→reviewed | AC-05 | [x] |
| T-08 | merge | T-07 pass | 冻结职责/必要consumer并集或有效证据复用，batch仅due主检出单实例 | AC-06 | [ ] |
| T-09 | merge | T-08 | 合入/SD-01..06读回、916/914接口与cert残余交接并archive delivered | AC-06 | [ ] |
| T-10 | merge | T-09 | 全部实际输出/安装副本/认证目录对账折计划，guard clean后清唯一树/分支/组/整个scope与登记 | AC-06 | [ ] |

拟唯一树D:/autostack/.wt/lang-917/auto-lang、plan-917-dev，未创建；开工材料唯一D:/autostack/plan-artifacts/plan-917/登记owner/用途/在途，实际编译/认证/外部消费根/target显式置此，不散落其他盘或仓内reports/attachments。merge全部摘要折计划后整删，具名未解hold才留；不动913或其他store，不建链接。

## 9. 复审记录

2026-11-02 | stage:new | PLAN-917 r1 | 规划待静态预检，非功能/最终review PASS。Design43 §17先写/注册，后allocation.lock排他由new-plan.sh真实取914..917，next918。R07/T08原source/oracle及现有模式表/发射入口实读，两格与916八格互斥，日常/显式认证独立。只规划，不改当前Spec/原源/expected/binding或913/916，未建树材料/运行构建测试探针认证/agent/chat。后续T-01，所有V/AC待执行。

2026-11-02 | stage: merge | PLAN-917:r1 合并收尾回执（五检查点）。prepared：reviewed=017ed2243 @ 基 5a0ef7609，Spec 增量 SD-01..06（hash 见 review 记录），delivery=9ae8df983。landed：master ff-only 至 9ae8df983（无 landing 合并提交；采用 master 合入分支再 ff 的路线，等价性经 git range-diff 实证——六个评审提交 2008e5f22/875957386/2a11ae3d0/a08558c11/6fc7e7239/017ed2243 在落地区间全部 patch-equal「=」，仅插入 915 的既有提交）。canonical_verified：SD-01..06 在落地提交逐一读回通过（路径与节标题核对）。archived：docs/plans/archive/917-legacy-a2r-mut-record-and-text-evidence.md，status=archived，计划文件已入库（cef72a72a→amend d47f4802c），线性的历史段 5a0ef7609→9ae8df983→d47f4802c。cleaned：wt-guard 双仓 clean（exit0），worktree+分支+组目录已删，plan-artifacts/plan-917（12MB）整删，保留登记处 maintenance_log 记删除，无名持有。landing 非部署（显式观察项）：本计划改编译器发射路径与测试基建，后端 release 二进制 / 依赖仓 release 二进制 / gen/front/vue/dist Web 包三者为代码前构建，现均 stale——记录为观察项，未重建重启（本会话无在跑生产进程需接续；下次常规构建窗口随 cargo build --release / auto build 刷新）。batch 回归核查：.last-batch-regression.json last_covered=916、时间戳当日——917%5≠0 且 <48h，未到期，不触发。outcome=pass。
2026-11-02 | stage:merge T-10 | 输出对账：本计划全部外部输出=D:/autostack/plan-artifacts/plan-917/（probe发射产物、compile/ rustc验证产物、cert-scratch（认证器自清理）、FINDINGS.md、registration.md）；无其他盘符/仓内 reports/attachments/外部安装副本；target 构建中间产物在 worktree 内随树删除。证据折叠：FINDINGS 关键数字已折入 §9 各条（发射根因三件套、42/panic 实跑、9/9 认证与 reused、V-06 P90 29/29/53/212ms、组并集 PASS、receipt 轴绑定），原始 scope 按 2026-10-31 裁定整删；具名未解持有=无（两项债务已入 KNOWN-DEBT-AND-RISKS，owner 明确）。收尾路径对账无遗漏目录。随后 wt-guard → 移除 worktree/分支/组目录。
2026-11-02 | stage:merge T-09 | landing=ff-only：master 5a0ef7609→9ae8df983（合并提交，线性历史），delivery commit 9ae8df983。合并提交冻结组并集重跑全 PASS：a2r / case-ace-m1-a2r-emit-917 / case-ace-m1-a2r-semantics-916 / case-ace-m1-base。SD-01..06 在落地提交逐一读回核对通过（新文件+五处 modify 均含 917 节）。认证器在合并基线复跑：9/9 reused（身份轴稳定）。916/914 接口无漂移（916 组绿；cert 域独立）。T08 Auto trap 认证 gap 与同根双.mut 欠账随 KNOWN-DEBT 保留，owner 移交 M1/前端准入缝。归档：本文件移 docs/plans/archive/，status=archived（终态）。
2026-11-02 | stage:merge T-08 | master 已前进（915 落地 e33af97c8），合并入 plan-917-dev 得 9ae8df983。冲突三处全解：ace-m1-conformance.md/gap-audit.md=两版本节并存（915 R02/R03 节 + 917 R07/T08 节）；source-manifest.json=binding.sha256 重绑为合并后 binding.tsv 实测 hash 30c5d680…（语义契约未变，review 证据按规约重绑到合并基线）。binding.tsv 自动合并干净：corpus R02/R03 保持原状、scenario R02/R03 取 915 版、917 的 R07/T08 行完整。冻结组并集在合并提交重跑（见下条）。
2026-11-02 | stage: review | review_scope: final | reviewer: 交付会话内自复审（限制：与 work 同会话，未独立上下文；已按技能要求从制品重建裁决——基线 diff 行级核查、金样区分性控制 grep、receipt 全轴 sha256 比对、冻结组并集重跑，未依赖 work 自述）| plan_id: PLAN-917 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-917/auto-lang | branch: plan-917-dev | reviewed_commit: 017ed2243 | base_commit: 5a0ef7609 | dependency_revisions: master@5a0ef7609（注意：master 已前进至 e33af97c8=915 落地，landing 需合并对账）| spec_inputs: SD-01 legacy-rust-mut-and-text-evidence.md(0edfce1e…) add；SD-02 trans/overview.md(ff6bae44…) modify；SD-03 a2r-string-ownership.md(e708434d…) modify；SD-04 a2r-golden-certification.md(32c2c4dc…) modify；SD-05 ace-m1-conformance.md(821183ab…) modify；SD-06 ace-m1-gap-audit.md(7d45b489…) modify——六目标均存在于 docs/specs，文本描述当前行为（非计划日记），与现行规范无冲突（a2r-string-ownership.md 原文未涉 slice，无矛盾）| acceptance_results: AC-01 pass（发射修复 diff 实证+金样控制+认证42）；AC-02 pass（T08 界断言金样+trap 边界三处一致声明）；AC-03 pass（adapter 0 Command 实证+receipt 全轴绑定+binding 仅两行+manifest 1行）；AC-04 pass（严格校验单测+漂移探针+V-06 实测+消费者核查）；AC-05 pass（本条）| pending_closeout_gates: merge 拥有——合并新 master 后冻结组并集重跑、SD-01..06 读回核对、归档与清理、batch cargo t all 按到期触发另算 | findings: F-917-R1 新引入未用变量警告（severity 低，已修复于 017ed2243，修复后 917 组复跑 PASS，无遗留）| evidence: plan-artifacts/plan-917/probe/FINDINGS.md（T-02..T-06 addenda；merge 时折叠入本文件并整删 scope）| next: merge
此前记录：2026-11-02 复审摘要 PASS（AC-05），冻结树 plan-917-dev @ HEAD（review后 017ed2243）。复审方式：对基线 5a0ef7609 逐 AC 独立重核，不复用 work 自述。AC-01：diff 实证三处发射修复；金样区分性控制实查（全部 R07 金样无 `&mut &mut`、含单次 `bump(&mut x,1)`）；认证 R07 实跑 42、近邻矩阵 rustc 级证据在册。AC-02：T08 金样含界断言与冻结 marker；trap=行为观察的边界在 receipt/case/spec 三处一致声明；合法近邻 nb_slice_ok 在场。AC-03：a2r adapter 全文 0 处 Command（日常零 spawn 实证）；认证域 schema/目录隔离；binding.tsv 仅 R07/T08 两行变更（diff 行级核实）；source-manifest 仅 1 行 hash 重绑；9 份 receipt 的 source/wrapper/reference/goldens 全轴与磁盘 sha256 逐一比对一致。AC-04：严格 receipt 校验单测 2/2、漂移 fail-closed 探针、V-06 四状态 P90 实测记录、852/verify/base/916 消费者核查绿。健康：发现 1 条新引入警告（certifier 未用变量）——已修复并提交（review 后 HEAD），917 组复跑 PASS；rust.rs:4391 未用 import 为基线既有。遗漏/延后扫描：无未批延后；两项如实偏差入册——①认证器未记录 Job 峰值内存（不在任何 V oracle， containment/cap 仍生效）；②master 已前进（915 落地 e33af97c8），landing 需合并 master 并重新对账 binding.tsv 与 manifest hash（merge 职责）。债务候选已录 KNOWN-DEBT（同根双.mut、T08 trap 认证）。结论：pass，无 needs_fix；next=merge。
2026-11-02 | stage:work | T-06完成，PLAN-917 execution_done，交付冻结于 plan-917-dev @ 6fc7e7239（基 5a0ef7609）。冻结提交复核：a2r / case-ace-m1-a2r-emit-917 / case-ace-m1-a2r-semantics-916 三并集 PASS；case-ace-m1-base 此前 PASS；trans::escape 16/16、plan667 21/1ign、ace_certify 1/4ign、认证域单测 2/2；认证器在冻结提交上 9/9 reused（身份轴稳定）。V-06 周转探针 plan917_emit_turnaround_v06（显式 --ignored 入口）：fresh P90=29ms / hit 29ms / body-edit 53ms / restart 212ms，n=30，门 P90≤2000ms 全绿。健康： touched-file fmt 净（仓级 fmt 漂移为 master 既有）、无新警告、无调试打印。V-01..08 闭图见 plan-artifacts FINDINGS T-06 addendum。债务候选：同根双 .mut 前端未拒绝（产物 rustc E0499）——已记 KNOWN-DEBT-AND-RISKS（owner=前端/916 准入缝，非本包闭合范围）；T08 Auto trap 认证通道保持 cert-gap；默认/命名实参在旧 producer 不支持（矩阵如实记录）。 receipts: stage:work | PLAN-917 r1 | outcome=pass | worktree=D:/autostack/.wt/lang-917/auto-lang | branch=plan-917-dev | base=5a0ef7609 | code=6fc7e7239 | deps=master | tasks=T-01..T-06 | evidence=plan-artifacts/plan-917/probe/FINDINGS.md（sha见登记） | blockers=无 | next=review(T-07)
2026-11-02 | stage:work | T-04完成（认证域交付，V-03/04/05主体）。新域 ace.m1.a2r-emit917.cert/1：模块 test_cases/a2r_emit_917_cert.rs + bin a2r-emit-917-cert；receipt 隔离于 a2r-emit-917/certs/（852 域不扫描）；复用 852 受限助手（compile/run_capped/run_bounded/capture_toolchain_contained 升 pub(crate)）；严格 receipt 校验单测2/2。认证实录：9/9 certified（exit0，R07 实跑42、T08 按冻结 panic），二次全 reused，漂移探针 FAIL 不覆写。golden 审阅理由见 plan-artifacts FINDINGS T-04 addendum；trap oracle=行为观察非 Auto trap 认证（gap 另账）。修复：actual 键加 single:/project: 前缀消除同键碰撞；compile() 无exe即失败 surfaced。
2026-11-02 | stage:work | T-05完成（正式组+父证据回填，V-04/07）。固定组 case-ace-m1-a2r-emit-917 入 test-groups.toml：11 checks（R07/T08 单+项目、7近邻单）全 PASS（零 Rust 编译）。daily goldens 走适配器 proposal 流程取自真实适配器产物并审阅；单模式字节与认证域 golden 完全一致（双源互证）。project 模式 case 按 contract 需 files，附未引用 helper.at（产物集不变=main.rs+Cargo.toml，name="project" 属输入身份）。binding.tsv R07/T08 a2r 列→ok:P917、状态→ok（T08 trap 认证 gap 如实保留），source-manifest binding.sha256 重绑（1行diff）。V-08 consumer 核查：852 certifier 只收 ok-emit（本格 ok:P917 不收）、verify-852 走 manifest（reference/oracle 目录未动）、base 组含 R07/T08 发射且 PASS、旧伪发射 reference/R07.rs 无日常消费者（未动）。临时探针 test 已移除。SD-01..06 已备于 docs/specs（merge 沉淀）。
2026-11-02 | stage:work | T-03完成（发射修复+合法近邻矩阵，V-01/02主体证据）。修法四处（详见 plan-artifacts/plan-917/probe/FINDINGS.md T-03 addendum）：①analyzer加copy_field_pred+gather_return_refs（Return臂精化：谓词证明Copy投影读跳过升级；返回表达式内嵌套裸.view/.mut操作数不升级；root .mut/.view与其余形态保历史保守）；②rust.rs构建struct_fields+per-fn注解局部类型表接谓词（fn与method两处）；③call_generic_fallback对已是Expr::Mut的实参跳过&mut前缀（原`&mut &mut x`）；④emit_str_slice_chars两参形态改带界断言块（recv按引用绑一次、各界一次、assert a<=b<=char count；单参形态不动）。验证：R07单+project实跑42；T08两入口按预期panic「auto str slice out of range」；近邻nb_now21=21/nb_caller21=21/nb_two_roots=53/nb_nested=121/nb_view=40/nb_move=20 全部rustc 1.98真编译真跑过（compile/目录）；`cargo t a2r` PASS；escape analyzer单测16/16；plan667 21过1ignore。支持矩阵冻结：mut参数写回/Copy投影返回读/嵌套直接调用/双根mut/view绑局部/move传值=已支持；默认实参（producer丢默认→E0061级产物）、mut路径命名/重排实参（914纯标量）、同根双.mut（前端未拒绝→产物rustc E0499）=不支持，如实记录不入金样；同根冲突记入KNOWN-DEBT候选（owner=前端/916准入缝，非917范围）。新fixture源已落 crates/auto-lang/test/cases/ace-m1/a2r-emit-917/src/（nb_*.at，正式case/suite/golden接线属T-04/05）。注：本轮曾误把analyzer.rs编辑落在主检出，已即时revert（git checkout），主检出无917代码WIP，工作全部在worktree。

2026-11-02 | stage:work | T-02完成（可行，不修则needs_replan不发生）。探针=临时test探针 plan917_probe（worktree内，T-05折正式case或移除），产物/记录存 D:/autostack/plan-artifacts/plan-917/probe/FINDINGS.md。事实：R07单文件入口原ERROR「cannot take '.mut' of 'x'…returned from function」（expr_emit.rs:340 F-05拒绝）；project入口原emit `bump(&mut &mut x,1)`双borrow。T08两入口原emit `chars().take(2).skip(0).collect`——1字符静默截断不trap、`.len()`为字节长=3，与冻结slice-oob语义不符，不得冻结为金样。根因三：①analyzer.rs Return臂gather_var_refs对Dot接收者无差别升级（false positive）；②call_generic_fallback is_mut_param前缀不识别arg已是Expr::Mut；③emit_str_slice_chars两参形态无界内检查（Plan 019字符位设计本身保留）。支持矩阵冻结：record mut参数 `fn f(mut x Counter,…)`→`x: &mut Counter`；调用 `f(x.mut,…)`→`f(&mut x,1)`；写回靠&mut别名；Copy字段投影读（x.value）不移动绑定；非Copy字段投影/Index/裸ident返回维持保守升级；真实依赖=std-only（Cargo.toml无deps）。blast radius：全仓0个既有golden用chars().take/skip形态。修复后实际emit+rustc 1.98实跑：R07=42（手算reference一致：now=21,x.value=21,总42）；T08按预期panic「auto str slice out of range」（Rust panic≠Auto trap认证，认证gap另账）。产物在 plan-artifacts/plan-917/{probe,compile}/。

2026-11-02 | stage:work | T-01冻结（基线=master 5a0ef7609b2da45c176ba8543c08d736d4425bd9，2026-11-02）。worktree已建：D:/autostack/.wt/lang-917/auto-lang @ plan-917-dev（new-wt-group.sh 一键建组，auto-down 兄弟仓 detached 就位）；材料根 D:/autostack/plan-artifacts/plan-917/registration.md 已登记（owner=auto-plan-goal交付会话，in-flight）。基线hash重测（§4表为规划时main0e93e3cc9旧值，已漂移，以本轮为准）：binding.tsv=9563cfc61f909d3e1b407a3963b382f8954f7de05d2271c5b2c7ecf15893876e；a2r-golden-certification.md=6bfa9203bdbd2be9b1a3f6789039733737b610a12d442e8c3c3ceac09a8698f4；rust.rs=7718d4764fb10cc7ee645826b38ba01f42fbc01776d7bc83fecd3b0075a13835；a2r.rs=32a4b98d753912df81c0cbacfbfe5ccbbffc4b4d09c3883507c71616a9b61a40（与§4一致）；trans/overview.md=85a9b6bbc80c91ca9a18be4bf62ed223ef26f0296596815b17c67583b40ec45a。原R07.at/T08.at SHA256 与§4登记一致（6d07888a…/f2db55ba…）。固定组核实（test-groups.toml）：a2r/auto-test-case/infra-fixed/infra-ace-m1/case-ace-m1-base/case-ace-m1-a2r-semantics-916 均在册；新组 case-ace-m1-a2r-emit-917 为 T-05 接线目标。916准入已入master（9b2c763ef），模式表（current_fn_mut_params/fn_mut_params/fn_merge_mut_params）实读存在，发射入口 transpile_rust_with_source_dir/transpile_rust_project 与 a2r adapter run_check 实读。主检出观测（不处置）：binding.tsv 工作区脏（R02/R03两行=915会话证据，非本计划写面，T-05只改R07/T08两行）；docs/design 等多文件脏属其他会话簿记。D-01冻结：R07=Counter{value int}+bump(mut x Counter, delta int)+main(var x=Counter(20); bump(x.mut,1); now+x.value=42)，T08="中".slice(0,2).len() 冻结slice-oob。

2026-11-02规划静态预检通过：0..10节有序、T10/AC6/V8/SD6定义唯一且引用齐、10任务全未勾、drafting r1/0-of-10；现有Spec路径存在、新Spec仍未建立、本计划设计链接/anchor/空白检查通过。固定组名称对照test-groups.toml有效，新914..917场景组明确为T-05未来接线目标；D911ca4848f5d与D91213a465336均为main祖先(exit0)。四包各认领12/2/8/2格、无交集且都是当前34阻塞子集，剩余10格清单已核，非正式运行PASS。914..917新号各唯一、分配后next918，无对应worktree或plan-artifacts，原binding SHA256仍67bf520d26e7d541f8fd5f44b8c218da9d40b67d3fd808b4b3e801b89ec47fb9；本轮未改当前Spec/代码/脚本/913。所有语言功能、资源、时间、认证和最终review证据仍待执行。

2026-11-02 | stage: goal | plan_id: PLAN-917 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-917/auto-lang | commit: 6fc7e7239 | next: review

2026-11-02 | stage: goal | plan_id: PLAN-917 | plan_revision: r1 | round: 2 | from: work | to: review | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-917/auto-lang | commit: 017ed2243 | next: merge

2026-11-02 | stage: goal | plan_id: PLAN-917 | plan_revision: r1 | round: 3 | from: review | to: merge | status: archived | outcome: pass | worktree: removed (guard clean) | commit: 9ae8df983 | next: done

## 10. 待澄清事项

| D | owner / bounded closure |
|---|---|
| D-01 R07真实支持 | T-02冻结参数/类型/资源/求值与真实依赖最小可编译全文件集，不能删actual兜底 |
| D-02 T08产物/认证 | T-02/04核Unicode/slice语义与独立golden，未证trap认证保gap；已知错误不得批准golden |
| D-03 独立证据schema | T-02/04按真实deps冻结，旧std-only证书不扩义；坏证书不降格，日常不编译 |
| D-04 916/914窗口 | T-01/03/05公共rust.rs/adapter/证书reader/M1全文件单写，稳定接口可提前交接、不等整包 |
