---
plan_id: PLAN-916
status: archived
feature_name: 旧A2R生产入口静态合法性收口
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-02
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-lang/trans/architecture.md, docs/specs/auto-lang/trans/overview.md, docs/specs/auto-lang/types/design/ownership.md, docs/specs/auto-test-case/a2r-golden-certification.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-gap-audit.md]
new_spec_components: [docs/specs/auto-lang/trans/rust-semantic-admission.md]
touched_goals: [GOAL-002]
affects: [auto-lang/trans, auto-lang/types, auto-test-case]
depends_on: [PLAN-908, PLAN-911]
current_step: 10
total_steps: 10
---

# [PLAN-916] 旧A2R生产入口静态合法性收口

## 0. 变更摘要

为旧production TransRust在发射前建立真实静态准入，认领corpus B16/B17/B18/B19/T06/R09/R10和scenario E03的a2r共8格，按原oracle拒绝非法程序；合法近邻继续转译。独立于917的R07/T08发射两格。依据[Design43 §17](../design/43-ac-av-typed-mir-and-abc.md#17-m1下一阶段场景观察av资源与旧a2r2026-11-02target-state)。archived r1、10/10：delivery edfdabc2e已ff-only落地，七份Spec读回、九组冻结门和主线smoke通过；worktree/分支/组/plan-artifacts全清理。merge pass尚待到期的主检出单实例批量回归。

## 1. 目标

G1：不可达分支也解析名称，bool/int分离、immutable write、missing return、borrow escape、nominal mismatch、use-after-move在Auto前端拒绝。G2：真实原因/stage/code/span/source身份经public API→adapter→严格父判保真，infra/Rust错误不冒语言拒绝。G3：现有合法单文件/项目/CLI/SDK消费者兼容，泛型/import/外部名字/动态合法边界不误拒；失败不发布accepted产物，修复可再次成功。G4：8格实证回填/独立近邻与强制红/预算/全部受影响消费者闭合。

非目标：新HIR-A2R替代、R07/T08 emit修复、38复合认证、旧AVM B18、通用完整语言checker重写、default切换/旧生成器删除、913恢复或native语义扩面。

## 2. 架构方案

真实source/project→完整声明/可见域→现有parser/infer/ownership能力的受控复用/补齐→源位置保真的typed Auto拒绝或已准入AST→原CTE/TransRust发射→现有a2r adapter父判。不能在TestCase按case id/源码文字命中特判；生产公共入口必须真实共享，不能只在M1 adapter前加过滤器。静态合法性先于会消去不可达分支的变换；不通过执行程序查名，也不重复执行CTE。无语言不合法时才进入生成器。

## 3. 技术栈

Rust，auto-lang-core parser/infer/types/ownership与trans/rust；复用auto-diagnostics typed来源与既有TestCase错误分类，AutoDown独立suite与cargo t固定组。先T-02最小闭环，再选等价局部机制，不预造通用新编译器或依赖ACE worker。

## 4. 需求分析与背景调查

### 4.1 授权/基面

规划时用户授权下一阶段几个计划，只规划；继承G-DEC全部裁定及旧合法性差异R-E不接受规则。2026-11-02用户以/goal“完成计划916”授予本计划work→review→bounded repair→merge全链执行权，目标/验收/范围不变。施工基面main48126444f（规划基面0e93e3cc9）；D911ca4848f5d/D91213a465336归档，913开发中不改其状态。242只是旧A2R长期tracker，不是覆盖本8格的既有施工合同。34静态阻塞中本包8格，非正式最新执行报告。交付后binding SHA256=d173d8ad95c020e186ed500086caa4eab6ec24da5690281cddbfd27352d81c9c，实现提交6b855f1ae。

| 当前来源 | SHA256 / 实读事实 |
|---|---|
| binding.tsv（crates/auto-lang/test/cases/ace-m1） | 67bf520d26e7d541f8fd5f44b8c218da9d40b67d3fd808b4b3e801b89ec47fb9 |
| docs/specs/auto-lang/trans/architecture.md | bccb309d0e22d1c519a3ffd9beee1deae530d3af9885dfd11e8426e6e5429881；旧Spec含搬迁前路径，实现在auto-lang-core |
| docs/specs/auto-test-case/a2r-golden-certification.md | bd78434370a41394162b1641aa28be65d0231996e61a4a0db196d7e2e34ee706；六负例义务是前端拒绝，不生成金样 |
| crates/auto-lang-core/src/trans/rust.rs | 68f68d0dded4056893b5390d97cb0db176a00768186cfc9e4fed0165b07cbe37 |
| crates/auto-lang-core/src/test_cases/a2r.rs | 32a4b98d753912df81c0cbacfbfe5ccbbffc4b4d09c3883507c71616a9b61a40 |

### 4.2 真实入口与残余

transpile_rust_full_with_options先Parser::set_dest(TransRust)/parse后CTEE::transform；transpile_rust_project解析模块显式skip_check=true；RustTrans::infer_fn_body当前丢弃check_body返回（let _），推导≠已检查合法。parser.rs::check_symbol可由skip_check跳过。故不能仅开启一个flag就假定七族完整检查，或只修single-file而项目路径继续绕过。

test_cases/a2r.rs::run_check通过真实single/project API发射，classify_staged按AutoError类型分类；plain Msg/工具panic不能冒typed语言原因。mod.rs为公共分类/父报告，若需扩字段须全部consumer审查。原八格src/B16/B17/B18/B19/T06/R09/R10.at已实读（E03复用B16及原B04/B20对照）；没有独立原src/E03，不造同名假基线。

原source SHA256冻结：B16=169433498bcf01026e128ec7b07141b703252f5d8ac749fa55e5274d7356339a；B17=90c03f2756b676a0cab70158b65f130e19b04634725325eeca325418e8247184；B18=cf9358d904e6fa3b11a731a86f8cfb536fb25042c1c40348cdcafee8bf3b89df；B19=1de47798f2993250be6f5e3fd88f2c1989483b3b8bb56f284fe5c5bafb5c5217；T06=90ae2418febb4d43831eb50ec50dc75e52d6ebeaaa86e42cea3c67b87b997a6f；R09=b0211dcb9bd4d2130f0d9006a64a74214c7946cf32e53d652cb68d89228d25b4；R10=f29a3a60b9289e0a238e144252fa1c70ebf7535db8a02dff01ac13d4594421ce。T-01只重核，不修改输入。

## 5. 详细设计

### 5.1 原输入与语义族

| 格位 | 独立原oracle / 需要拒绝的机制 |
|---|---|
| corpus B16 + scenario E03 a2r | resolve：if false中missing()仍查名；与B04跳过除法/B20执行除法的可达性对照分开 |
| corpus B17 a2r | type：true+1非法，不让Rust后端隐式转换 |
| corpus B18 a2r | assign-immutable：let x重写非法；合法var写回可用 |
| corpus B19 a2r | missing-return：if false{return1}不能证明int函数全路径返回 |
| corpus T06 a2r | borrow-escape：返回局部owned Str的view非法；不偷改返回owned来接受原负例 |
| corpus R09 a2r | nominal-type：相同布局B不能传给A形参 |
| corpus R10 a2r | use-after-move：a.move后a.code非法，不能靠Rust Clone自动补源 |

新增合法近邻至少覆盖正确定义missing、bool运算、var赋值、全路径return、合法owned返回/有来源view、同名义类型、move后只用新owner；原NB17/18/19值2/2/0也须旧A2R成功生成，显式独立认证其行为才能报行为已证。新增nearby放独立目录，不改原负例/expected。补反例含嵌套作用域/shadow、参数/捕获、分支move合并、同宽异名、未执行分支；规则按语义结构一般化，不能硬编码七种源片段或case id。

### 5.2 准入与兼容入口

T-02做全部语义族最小真实production路径，冻结所用现有检查事实、诊断映射及合法域/外部import处理。可新增私有trans/rust_semantics.rs（新路径候选）并接已有checked事实，不能把ACE闭包受检凭证或runtime cell导入旧AST后端。静态诊断绑定原源span并在CTE/去死代码之前保留所需义务；元数据/类型/实例/CT结果使用真实依赖，文本同不代表命中有效。

single-file/source-dir/siblings/full-options/project（含多模块声明与forward reference）及真实CLI/SDK consumers列矩阵，公共API准入不可绕过；项目多文件先建完整可见声明再查body，不能因当前skip_check而逐文件误报跨模块名。对use.rust/use.c/模块/动态合法名字沿现有语言授权解释，不能用“非本M1域”泛拒其他合法项目；若现有事实不足以安全保持兼容且需通用平台，应needs_replan，不删八格或只做adapter开关假闭环。生产入口开关若存在必须明确相同默认义务/版本，不能创建另一个profile后悄然替旧M1身份。

### 5.3 错误、失败修复与预算

typed AutoError/已有诊断对象传stage/code/span/raw message，不按expected或message关键词猜语义族；父侧保实际来源/工具/操作/source/producer身份。未准入时accepted Rust artifact发布=0、Rust工具spawn=0、运行=0；临时输出目录可存在但不得被登记为成功产物。语法/语义拒绝、资源、工具/线程panic、缺文件、CTE授权撤销分别分类。保留上一已接受产物与内容hash、失败不覆盖；修复后同源项目实际重新生成，旧artifact/旧PASS不复用。

fresh/hit/restart/body-edit及失败→修复/实际再读均有文件化首例；旧cold emit如实记录重发射，不声称函数级P0。若改共享ACE frontend/infer/cache出口，必须用原受影响暖API验证未改函数真实P/IO0、同对象/代际、实际deps失效与恢复重验；默认方案不改ACE生产链，因此不强造第二层函数cache。source相同但依赖/CT权限/target语义改变的旧检查结果不得误用。

沿原source/元数据4MiB、流1MiB/流、含emit原128MiB栈保留既有预算；新诊断事实≤4MiB且先限额后物化，父峰值按原更严cap记录。现有正式周转腿P90≤2000ms、原样本/规模保持，新增日常emit/检查代表性四状态各30样本另列；显式认证编译120s/运行30s独立。语义失败不能靠超时/栈溢出满足负例；过限按资源拒绝。

### 5.4 与913、914、917的写面

P=新semantic模块/定向单测/独立输入；S-L=auto-lang-core parser/infer/ownership错误出口，917主RustTrans发射。rust.rs公共入口/全部wrapper、test_cases/a2r.rs与mod.rs、auto-diagnostics/auto-test-case、fixed组、binding/manifest为H单写短窗。默认916先冻结准入API/类型给917，917可先私有emit/ref方案；正式可并行任务无需D916整包，只消费已稳定提交或主线接口并列consumer回执，不能消费混合WIP。

913主ACE恢复，不设语义依赖；若碰共享AutoError/父合同/注册取实际窗，914 A2R观察也按同一adapter版本交接。每窗owner+revision/SHA+符号/字段/构造匹配/全consumer+验证/释放，漂移STALE受影响证据；不持窗等机器，正式认证错峰，不另开会话。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/trans/rust-semantic-admission.md | 生产准入/七语义族/所有入口/边界 | 不做M1特判 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-lang/trans/architecture.md | 校正真实路径并补旧A2R准入位置 | 当前知识 | AC-02 |
| SD-03 | modify | docs/specs/auto-lang/trans/overview.md | 补检查支持/兼容与残余 | 不冒完整语言 | AC-01/02 |
| SD-04 | modify | docs/specs/auto-lang/types/design/ownership.md | 补真实旧A2R borrow/move准入证据，不改语义 | 资源事实共享 | AC-01 |
| SD-05 | modify | docs/specs/auto-test-case/a2r-golden-certification.md | 八格负例判面/真实族映射，双轴保持 | 不造负例golden | AC-03 |
| SD-06 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 八格凭当前父事实重绑 | 不拿Rust拒绝充Auto拒绝 | AC-03 |
| SD-07 | modify | docs/specs/auto-test-case/ace-m1-gap-audit.md | 记录八格关闭/未闭owner及剩余能力 | 不回开242/908 | AC-03 |

## 6. 测试设计

新目录crates/auto-lang/test/cases/ace-m1/a2r-semantics-916/含case/suite/合法近邻/独立reference；原七src只引用。拟case-ace-m1-a2r-semantics-916组由T-05接正式首例，目前未存在。work自查/同树独立上下文review/merge；所有V未运行。

| Case ID | AC / invariant | real entry / inputs | expected / distinguishing control | owner_stage / task |
|---|---|---|---|---|
| V-01 | AC-01 七语义族/八格 | 原src经public single/project/adapter | 正确Auto族且无accepted产物/无Rust运行；跳过false分支/漏type/吞check_body红 | work T-02/03 |
| V-02 | AC-01 合法近邻 | 独立近邻、原NB17/18/19、B04/B20成组 | 合法emit、显式认证2/2/0；全拒/把运行trap当静态type红 | work T-02/04 |
| V-03 | AC-02 入口/作用域 | source-dir/siblings/full/project/CLI/SDK，跨模块forward/import/dynamic | 相同非法义务、合法不误拒；只adapter修复/多文件绕过红 | work T-03/04 |
| V-04 | AC-03 typed父准入 | 真语言/infra/坏类型/重复key/错source/过期工具 | 保raw原因且父区分；拿rustc失败/panic/generic充负例红 | work T-05 |
| V-05 | AC-04 失败修复/依赖 | 合法→非法body-edit→修复/新进程，依赖/CT权限变化 | 失败不覆盖旧accepted、修复真emit；旧PASS/仅源码hash命中红 | work T-04/06 |
| V-06 | AC-04 资源/时间 | 四状态30样本、原消费者规模/超额输入 | 原cap/P90与正确性，资源非语言诊断，计量不假P0 | work T-06 |
| V-07 | AC-03 八格回填 | 独立source hash+真实parent报告 | 仅本八格，无expected/NA/分母变；E03重复名/漏row/旧报告红 | work T-05/06 |
| V-08 | AC-02/04 交接/回归 | 917/914/913版本与所有consumer | public入口无绕过、shared变化补下游，旧产物金样漂移具名非自动更新 | work T-01/06 |

职责组a2r parser infer ownership auto-test-case infra-fixed infra-ace-m1及case-ace-m1-base/nb-diags-908/新916组；诊断公共分类改动加vm、a2c/a2py/a2ts等实际转译消费者（T-01核固定名与理由），若真碰ACE共享桥接加ace-adapters/ace-source/ace-semantics/ace-lowering及真实下游，未触ACE不为标题补建worker。完整组不只新checker；显式认证不入日常快照，不减合法旧语料/金样/忽略条件。

## 7. 验收标准

| ID | owner_stage | closing condition |
|---|---|---|
| AC-01 | work | V-01/02全部族及近邻正确，真实production准入而非case特判 |
| AC-02 | work | V-03/08所有实际入口/作用域/合法消费者兼容，无单文件/adapter绕过 |
| AC-03 | work | V-04/07父原因准入与八格绑定实证，负例无Rust编译/成功产物，原语义规则不变 |
| AC-04 | work | V-05/06/08失败修复/依赖/预算/时间/完整职责组/健康与窗口通过 |
| AC-05 | review | 独立逐AC/V/SD/遗漏/延期/workaround审查，冻结SHA绑定且必需项全pass |
| AC-06 | merge | 主线真实交付、Spec读回/归档/全输出与guard清理、剩余owner交接，不签M1 |

## 8. 执行步骤

| T | owner_stage | 前置/窗 | 工作与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施指令；D911；917/914/913实际H | 冻结基面/八格/所有入口与合法域/consumer/cap/组，登记唯一树材料和窗口 | AC-01..04 | [x] |
| T-02 | work | T-01；P/准入短窗 | 七族single+project最小真实拒绝、合法近邻/失败修复、冻结typed/依赖方案；不可行needs_replan | AC-01/02/03 | [x] |
| T-03 | work | T-02；S-L/公共wrapper短窗 | 正式生产静态检查/事实与typed传播，保原scope/import/CTE语义，V-01/03 | AC-01/02 | [x] |
| T-04 | work | T-02/03；私有/受影响消费者 | 合法近邻/跨模块/所有入口与失败原子/修复/依赖矩阵，V-02/03/05 | AC-01/02/04 | [x] |
| T-05 | work | T-03/04；H/M1 | case/suite/父比较/独立reference/新固定组与八格正式证据回填，V-04/07 | AC-03 | [x] |
| T-06 | work | T-01..05；错峰运行 | V-01..08、完整组/必要消费者/正式原门/健康自查，冻结SD/SHA→execution_done | AC-01..04 | [x] |
| T-07 | review | T-06同树冻结SHA | 独立原源→public准入→typed事实→父判，查遗漏/绕过，修复重绑pass→reviewed | AC-05 | [x] |
| T-08 | merge | T-07 pass | 冻结职责/必要consumer并集或有效证据复用，batch仅due主检出单实例 | AC-06 | [x] |
| T-09 | merge | T-08 | 合入/SD-01..07读回、917/914接口/残余交接及archive delivered | AC-06 | [x] |
| T-10 | merge | T-09 | 全实际输出对账折计划，guard clean后清唯一树/分支/组/整个scope和登记 | AC-06 | [x] |

唯一树D:/autostack/.wt/lang-916/auto-lang、plan-916-dev，基面48126444f、delivery edfdabc2e；唯一材料D:/autostack/plan-artifacts/plan-916/已登记owner/用途/在途，所有探针/日志/reference生成输出均入此，未写attachments/仓内reports或其他盘散落；merge摘要已折计划，worktree/分支/空组目录和整个plan-artifacts scope已于2026-11-02删除，登记无hold。

## 9. 复审记录

2026-11-02 | stage:new | PLAN-916 r1 | 规划待静态预检，非功能/最终review PASS。Design43 §17先写/注册，再排他allocation.lock由new-plan.sh实际取914..917，next918。单/项目skip-check及吞推导错误已实读，不把一个开关当七族已实现；与917发射职责明确，两包各有精确格位。只有规划，没有修改当前Spec/原源/expected/binding/913，未建树材料/运行编译测试探针认证/agent/chat。后续T-01，所有V/AC待执行。

2026-11-02规划静态预检通过：0..10节有序、T10/AC6/V8/SD7定义唯一且引用齐、10任务全未勾、drafting r1/0-of-10；现有Spec路径存在、新Spec仍未建立、本计划设计链接/anchor/空白检查通过。固定组名称对照test-groups.toml有效，新914..917场景组明确为T-05未来接线目标；D911ca4848f5d与D91213a465336均为main祖先(exit0)。四包各认领12/2/8/2格、无交集且都是当前34阻塞子集，剩余10格清单已核，非正式运行PASS。914..917新号各唯一、分配后next918，无对应worktree或plan-artifacts，原binding SHA256仍67bf520d26e7d541f8fd5f44b8c218da9d40b67d3fd808b4b3e801b89ec47fb9；本轮未改当前Spec/代码/脚本/913。所有语言功能、资源、时间、认证和最终review证据仍待执行。

2026-11-02 | stage: work | plan_id: PLAN-916 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-916/auto-lang | branch: plan-916-dev | base_commit: 48126444f6ceeb6ee346b3a688da509822b766ff | code_commit: 6b855f1ae42bcbabe818ec805d63ba3b8ade4cdf | dependency_revisions: none | task_ids: T-01..T-06 | evidence: AC-01原七源+NB/分支move控制通过；AC-02 single/full/project/merged共享准入与完整a2r合法消费者通过，当前仓无auto trans CLI bin故未伪造CLI项；AC-03新17-check父suite及八格ok:P916回填通过；AC-04失败→修复、cold四态30样本×2进程及职责并集通过。最终命令`cargo t a2r parser infer ownership auto-test-case infra-fixed` exit0（391 a2r tests，run-l4wr6g2x，log SHA256 280b8dabf2267ba61ca26447f5e086ddf5e77e3956f42c638f21d30aa2cef34c）；`cargo t infra-ace-m1 case-ace-m1-base case-ace-m1-nb-diags-908 case-ace-m1-a2r-semantics-916` exit0（23治理+97+9+17 checks，run-13tyd64m，log SHA256 de6b82addba235b03348e2388e027ca7bf74a5811d3efd454ad1ef7c918beff8）。四态P90进程1=17600/17114/50/17609us、进程2=17004/16972/51/17116us，均<2000ms且如实cold（logs 617f8fe50396277a999521420ea536be4cae50be43298dd7a9190e93b9eb331f / 89bf2aeab254814bb9cf1a1f3d3b40b3044cd35b62eb15fee9a948313e623759）。rustfmt--check+git diff--check exit0；无开放work finding/未批延期，review/merge统一冻结门待相应owner | blockers: none | next_responsible_role: review

2026-11-02 | stage: goal | plan_id: PLAN-916 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-916/auto-lang | commit: 6b855f1ae42bcbabe818ec805d63ba3b8ade4cdf | next: review | 授权：用户/goal完成计划916，继承work→review→bounded repair→merge全链。

2026-11-02 | stage: review | review_scope: final | reviewer_identity/limitations: 同一实现会话的新goal-turn、无独立model；不据此自称独立角色，结论由仓外consumer与实读diff重建 | plan_id: PLAN-916 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-916/auto-lang | branch: plan-916-dev | reviewed_commit: 6b855f1ae42bcbabe818ec805d63ba3b8ade4cdf | base_commit: 48126444f6ceeb6ee346b3a688da509822b766ff | dependency_revisions: none | spec_inputs: SD-01..07已实读但判当前行为描述不完整，冻结diff SHA256 3033156d9864204e12fa6c36d15a3d5bdae5c954eb5cb3c11efd6497035a83de | acceptance_results: AC-01 fail（嵌套borrow/Expr-If move/immutable）；AC-02 fail（closure/import名称绕过）；AC-03 partial（原17-check绿但生产反例accepted）；AC-04 fail（覆盖矩阵遗漏，旧组绿不能覆盖）；AC-05 needs_fix | pending_closeout_gates: 全部review/merge门及Spec重读 | findings: R916-F01..F04，T-02..T06重开 | evidence: `CARGO_TARGET_DIR=D:/autostack/plan-artifacts/plan-916/review-target cargo run --manifest-path D:/autostack/plan-artifacts/plan-916/review-probe/Cargo.toml` exit1；nested-borrow/expr-if-move/expr-if-immutable/closure-missing/import-missing均ACCEPTED，log SHA256 e08d3387742c72121dbc53f321768f98b46c5030d1a1eca58ef243111cd9542c，probe SHA256 e38e73c54d4cd29884b59538abbb276838ef45ffedc8f186694d1a243d6b985e | next: work

| finding → invariant | affected paths/states | 实证与关闭oracle |
|---|---|---|
| R916-F01：所有return位置均执行borrow-escape | single/project及嵌套If/Block/Try/Is等；`semantics.rs:595`仅扫顶层return | nested-borrow经production API accepted；补齐递归return遍历，永久嵌套不可达+合法owned/view控制 |
| R916-F02：move/immutable检查覆盖完整表达式与分支合并 | Expr::If/Block/Closure/Async/Is/Try及shadow；`semantics.rs:389,502`缺Expr::If等臂 | expr-if-move与expr-if-immutable均accepted；补共享结构遍历和作用域/合并反例，不能只修两个文本形 |
| R916-F03：闭包/捕获体名称也解析 | closure/lambda/async及项目路径；`semantics.rs:159`未进入Closure | closure-missing accepted；闭包内未定义调用拒绝，合法捕获/外部名字控制不误拒 |
| R916-F04：import不豁免未定义普通调用 | 任一Use即`allow_external=true`（`semantics.rs:110`）导致全文件跳过unknown call | import-missing accepted；按真实import/builtin/task可见域判定，import存在时missing仍拒，合法Rust/C/module调用继续通过 |

2026-11-02 | stage: goal | plan_id: PLAN-916 | plan_revision: r1 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-916/auto-lang | commit: 6b855f1ae42bcbabe818ec805d63ba3b8ade4cdf | next: work | repair_cycle: 1/3

2026-11-02 | stage: work | plan_id: PLAN-916 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-916/auto-lang | branch: plan-916-dev | base_commit: 48126444f6ceeb6ee346b3a688da509822b766ff | code_commit: 1b6f88897e225d351acc13b69059fc414df76eed | dependency_revisions: none | task_ids: T-02..T-06 repair-1 | evidence: F01共享enter/exit遍历覆盖嵌套return；F02绑定ID+scope区分shadow并覆盖If/Try/Is/Closure/Async；F03闭包名解析与合法helper控制；F04按Use items/path/wildcard/C/task及真实runtime面判定，无全文件豁免。永久8非法+4合法控制过；原外部probe五结果均typed REJECTED（log SHA256 7626bf7b3ba70150f2e03ed619a0a3110987820b663845cf4075ea15c0d7e9bc）；`cargo t a2r case-ace-m1-a2r-semantics-916` exit0（393 a2r tests+17 checks，run-0ul550hq，log SHA256 b5a327df2ea291b7c26bdaaa90e145ce11f857b3448e4838110427a7b5ad7fbb）。旧重复遍历整体删除，rustfmt/diff check exit0、无本模块dead-code。开放finding=0，未改验收/范围；未触ACE共享链。merge统一冻结并集仍待merge owner | blockers: none | next_responsible_role: final review

2026-11-02 | stage: goal | plan_id: PLAN-916 | plan_revision: r1 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-916/auto-lang | commit: 1b6f88897e225d351acc13b69059fc414df76eed | next: review | repair_cycle: 1/3

2026-11-02 | stage: review | review_scope: final repair-re-review | reviewer_identity/limitations: 同会话新goal-turn；仅重放受影响finding与兄弟形态，复用1b6f88897完整组证据 | plan_id: PLAN-916 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-916/auto-lang | branch: plan-916-dev | reviewed_commit: 1b6f88897e225d351acc13b69059fc414df76eed | base_commit: 48126444f6ceeb6ee346b3a688da509822b766ff | dependency_revisions: none | spec_inputs: SD-01..07当前文本仍待nested-expr行为闭合 | acceptance_results: F01..F04原输入pass；F01扩展nested-expr-borrow fail；AC-01/03/04未闭，AC-05 needs_fix | pending_closeout_gates: repair-2定向重放、受影响组与merge门 | findings: R916-F05 | evidence: 同一仓外probe新增`return if false { s.to(str) } else { "ok" }`，其余五项REJECTED而nested-expr-borrow ACCEPTED；命令同前exit1，log SHA256 248220c9016c3960bec9643384c5fd63dde354feb92dd5270422df3d313fd489，probe SHA256 995ac5cc66338448b112bf7c57f707f13f5a564d2ba5e70df7af9c2f85311be0 | next: work repair-2

| finding | 两次失败诊断检查点（repair commit前必须完成） | 设计/oracle调整 |
|---|---|---|
| R916-F05 返回表达式内部borrow逃逸 | 先前检查只验证Return语句直接表达式，完整组没有`return <复合表达式含局部view>`反例，因此旧例闭而兄弟形态漏检；影响single/project的If/Block/Is/闭包等所有返回组合 | 将borrow检测递归应用于每个Return的完整返回表达式树，而非再加If特判；永久覆盖If/Block返回、合法owned/有来源view控制及项目路径，重放原F01和F05 |

2026-11-02 | stage: goal | plan_id: PLAN-916 | plan_revision: r1 | round: 4 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-916/auto-lang | commit: 1b6f88897e225d351acc13b69059fc414df76eed | next: work | repair_cycle: 2/3 | two_failure_diagnosis_required: true

2026-11-02 | stage: work diagnosis checkpoint | plan_id: PLAN-916 | plan_revision: r1 | outcome: verified adjustment before repair-2 commit | finding: R916-F05 | missed_invariant: 先前只查Return直接表达式，测试没有复合返回树 | verified_design: Return进入时`return_depth+=1`，复用共享walker检查整棵返回表达式，退出恢复；无If/Block特判，single/project共用 | oracle_results: 永久nested-expr single+project反例及原F01..F04/合法owned/view控制通过（tests log SHA256 4a654e3f61f36e17d44ad27157b6870769a8ae84c45fef70f2e42e4bcac0abc4）；外部六项全typed REJECTED（probe SHA256 6c9eea05de6b1a1d6b03adc7850a350b8aded656e52bfc1d064b0716991b001d）；`cargo t a2r case-ace-m1-a2r-semantics-916` exit0（groups SHA256 12596f588e3f4f00cd378ab907b3293f18ddc261f952dc32e9988d7dcd6539a9） | remaining_gap: none in declared F01..F05 | next: commit repair-2 then final review

2026-11-02 | stage: work | plan_id: PLAN-916 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-916/auto-lang | branch: plan-916-dev | base_commit: 48126444f6ceeb6ee346b3a688da509822b766ff | code_commit: 9ee15d466c767d2c9212d2225667dd21c1427d5b | dependency_revisions: none | task_ids: T-02..T-06 repair-2 | evidence: 两次失败诊断和验证调整已于commit前落盘；F05 single+project反例与合法控制、原F01..F04、外部probe及`cargo t a2r case-ace-m1-a2r-semantics-916`均exit0，logs见上一诊断记录hash。当前树clean、rustfmt/diff check通过，无开放in-contract finding，未改范围或ACE链。统一merge冻结门待merge owner | blockers: none | next_responsible_role: final review

2026-11-02 | stage: goal | plan_id: PLAN-916 | plan_revision: r1 | round: 5 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-916/auto-lang | commit: 9ee15d466c767d2c9212d2225667dd21c1427d5b | next: review | repair_cycle: 2/3

2026-11-02 | stage: review | review_scope: final repair-re-review | reviewer_identity/limitations: 同会话新goal-turn，独立按finding/oracle复核；无额外model | plan_id: PLAN-916 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-916/auto-lang | branch: plan-916-dev | reviewed_commit: 9ee15d466c767d2c9212d2225667dd21c1427d5b | base_commit: 48126444f6ceeb6ee346b3a688da509822b766ff | dependency_revisions: none | spec_inputs: SD-01..07实读通过，final delta SHA256 3033156d9864204e12fa6c36d15a3d5bdae5c954eb5cb3c11efd6497035a83de | acceptance_results: AC-01..05 pass；F01..F05原输入/兄弟输入和合法控制均pass，开放finding=0 | pending_closeout_gates: merge冻结职责并集、主线landing/Spec读回/archive/guard+scope清理 | findings: none | evidence: 定向永久测试exit0（SHA256 0de7cf473ce68f824e7507629ee25a8796b17f3da56befdc32269cd7f20c7847）；外部六项全typed REJECTED（SHA256 bd76b02725a7e5736d30799987331c7d4aa36a636f2cb173403b0c2bdf7d9c22）；同内容完整`cargo t a2r case-ace-m1-a2r-semantics-916` exit0（SHA256 12596f588e3f4f00cd378ab907b3293f18ddc261f952dc32e9988d7dcd6539a9）；rustfmt/diff check exit0 | next: merge

2026-11-02 | stage: goal | plan_id: PLAN-916 | plan_revision: r1 | round: 6 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-916/auto-lang | commit: 9ee15d466c767d2c9212d2225667dd21c1427d5b | next: merge | repair_cycles_used: 2/3

2026-11-02 | stage: merge | plan_id: PLAN-916:r1 | outcome: landed+canonical_verified | delivery_commit: edfdabc2e4e2fa32ae579fede9d4bf31b24ad0e8 | prepared: reviewed 9ee15d466；ff-only rebase映射6b855f1ae→9b2c763ef2d624f61d1a0922b68867b2ce70bc70、1b6f88897→798bcb5eec2be2e5e05708cf84336301f2c4739d、9ee15d466→edfdabc2e，range-diff三行全`=`（SHA256 a4212c4f6ba4d7649b7b4deaf538f10f929b213429f60afbbde815cfe3ffeca9） | gates: rebased tip执行`cargo t a2r parser infer ownership auto-test-case infra-fixed` exit0（log 4af1d86d0fe39ef54671ba7f5fffe4c1e193359450417c257299d127493e72f9）与`cargo t infra-ace-m1 case-ace-m1-base case-ace-m1-nb-diags-908 case-ace-m1-a2r-semantics-916` exit0（log 969fb41470e85cc34029c8274f622a8001e88b5c02ee6fb011f01acbf65d2493）；select-only已核对九组但不当执行证据 | landed: master `f422b0ef4..edfdabc2e` fast-forward，master tip==delivery；`wt-guard` clean | canonical_verified: 七份Spec均在主线提交，hash清单SHA256 d4d24336f46413a1bcbffabab08bedcb6ac9e32bcbb540c2347bb26d819f641f（architecture 450a02fd、overview 85a9b6bb、rust-semantic-admission 54b23e0d、ownership 6c754e84、a2r-golden e6ab0be2、conformance a6e6d3a0、gap-audit 3655ee44）；主线定向smoke exit0（log ba9e903428c29ef54a3042f37a98a7de61b0c59e832cc1b6a2b2edc3f9fc7e35） | handoff: 917复用同一公共准入/rust.rs提交，914共享adapter版本；未消费WIP、未改R07/T08/ACE/证书 | next: archive+guard cleanup

2026-11-02 | stage: merge | plan_id: PLAN-916:r1 | outcome: cleaned pending batch regression | delivery_commit: edfdabc2e4e2fa32ae579fede9d4bf31b24ad0e8 | archive_commit: 9647820ebba6f9a11e566050578f5b75c3dd16a6 | archive_path: docs/plans/archive/916-legacy-a2r-semantic-admission.md | cleaned: 2026-11-02T17:55:52+08:00，`wt-guard`对worktree与plan-artifacts均clean；`D:/autostack/.wt/lang-916/auto-lang`、`plan-916-dev`、空组目录及`D:/autostack/plan-artifacts/plan-916/`已删除；worktree list/分支/dir存在性复核通过，registry `auto-lang/PLAN-916` status=deleted-at-merge、holds=none | artifact_observation: 本计划为编译器库/测试adapter变更，未授权生产部署；主线`target/release/auto*`与`gen/front/vue/dist`均不存在，无运行中生产进程需重启，未伪称重建。批量due：last covered 895且多计划合并>48h；检测到PLAN-913 ace-av nextest运行，待其结束后主检出单实例`cargo t all`，不并发 | next: batch regression then final merge receipt

2026-11-02 | stage: merge batch regression | plan_id: PLAN-916:r1 | outcome: new_reds with 916 own regression fixed | batch_commit: cf46948803034eefb205025fca23bcf0424c31cf | evidence: 主检出单实例`cargo t all` exit1，run-ffy4jmss，约15.5min，log SHA256 c7ee1c903e3c643ce44793653e1908c860a3f5e44a825bafabe7539bfa0d48fd；原条件aavm/a2r-actor-tests/auto-vm NOT_APPLICABLE，其他排除不冒PASS | own_regression: `plan724_probe_stream_free_fn_typed_dispatch`因stream_next undefined失败，L0 `fix-916-stream-builtins`仅补stream_next/stream_is_done/stream_close三内建名，原失败测试+语义控制通过，ff-only至cf4694880，guard后树/分支/scope全删 | other_new_reds: PLAN-007 auto-media-core fixture_manifest缺manifest.json；PLAN-900 auto-lsp包已迁出；PLAN-901 auto-lang-core ACE host ownership与infra-pilot-cli legacy worker；auto-macros Box<Obj>编译错；sys_natives_vm断言；web-delivery-controls为已知D-910-1 execution_ids未定义。均记录fix-forward，不在916/回归中混修 | next: receipt+debt then done（非全绿声明）

## 10. 待澄清事项

| D | owner / bounded closure |
|---|---|
| D-01 现有checker复用 | 已闭并经final pass：Return整棵表达式树经return_depth共享遍历，F05 single+project及原控制通过 |
| D-02 typed传播 | 已闭repair-1：import下missing typed拒绝，合法runtime/import继续通过；普通Msg/infra仍不得充负例 |
| D-03 917公共入口 | 非阻塞交接：共享`trans/rust/semantics.rs`与rust.rs入口最新提交9ee15d466c767d2c9212d2225667dd21c1427d5b；merge向917/914注记，不消费彼此WIP |
| D-04 缓存/CT依赖 | 已闭：未新增语义缓存；每次真实重新解析/发射，四态均如实cold，无假P0/旧PASS复用 |
