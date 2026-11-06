---
plan_id: PLAN-957
status: drafting
feature_name: Batch auto-api-gen rustc crash triage
created_at: 2026-11-06
updated_at: 2026-11-06
plan_revision: 1
current_step: 0
total_steps: 5
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---
# PLAN-957 auto-api-gen 发现阶段 rustc 崩溃分诊
## 0. 变更摘要
周期batch after956新增一项编译器进程崩溃；原样登记，不冒认为产品类型错误/机器资源问题。仅起草fix-forward调查，不在956内更改编译器/配置或重跑全队列。
## 1. 目标
AC01(work)：准确重建原发现命令、profile/features/flags/target/incremental及工具链与源码身份，定位可复现边界或有界不复现证据。
AC02(work)：将环境/缓存状态/工具链故障与源码触发区分，记录原始错误及有效正控；不能凭一次重放绿删首红，也不能凭exit0跳过实际发现。
AC03(work)：若需修复，最小生产/工具owner修复与永久负控，完整auto-api-gen固定组实际通过；若无需源码修改，须给出可审计恢复机制与关闭依据，不能无因归为flake。未知根因保持未关闭。
AC04(review)：独立语义/输入与负控审查；AC05(merge)：实际职责门、规范影响确认、ff-only与清理归档。
## 2. 架构方案
有界诊断先行。单独专属worktree/target，保原主树输入与反例；不得删除他方target、停止他方进程、降低断言/绕过发现或修改源代码来迎合compiler crash。
## 3. 技术栈
Windows x86_64-pc-windows-msvc，rustc 1.98.0 (88d9e12ae178fab0fb5cc050a94da85685d449ea，2026-08-18)，LLVM22.1.8，Cargo/nextest。
## 4. 需求分析与背景调查
授权来自auto-plan-regress对新增红的fix-forward登记要求；当前用户实施授权为956 Reader Vue+Rust，已交付，本项仅draft，无实施批准/模型/预算或额外agent授权。
依据：docs/specs/auto-test-case/build-storage-lifecycle.md、architecture.md及AGENTS固定组规则；scripts/test-groups.toml auto-api-gen定义。
原batch覆盖b9d24ff4b，37组849.43秒；auto-api-gen discovery执行cargo nextest list -p auto-api-gen --lib --tests（精确argv见原command），编译auto-lang-core default lib时rustc退出0xc0000005 STATUS_ACCESS_VIOLATION，21条warning，无常规类型error或ICE backtrace。原profile含debug=line-tables-only、incremental、linker rust-lld、/STACK:33554432。原编译器构建本体崩溃，不是业务进程。
同一队列后续case-parent cargo build --locked --offline -p auto-lang --bin auto-case --bin test-case-pilot退出0（93.77秒）；956独立冷原生后端/默认工具构建、1655职责门与Reader46项对拍均通过。这些是不同编译图/配置的正控，不能据此宣布原profile已恢复或证明纯环境原因。
原始四文件已逐字节复制到唯一scope D:/autostack/plan-artifacts/plan-957，original manifest记录来源/hash/字节；batch原scope收尾删除，957材料具名持有，owner=Rust构建/工具链分诊，关闭条件=本项原始崩溃完成原因判别/必要修复，摘要折叠后merge全删。
## 5. 详细设计
T01(work)：从command/日志重建原编译单元和精确flags，检查同轮成功构建差额；不执行全量队列。
T02(work)：同一树有界历史/缓存/配置对照，先保存输入再操作本计划target；不得盲清主树incremental。输出因果假设、支持/反证及尚不确定项。
T03(work)：仅在原因/owner清楚后最小修复与原反例/普通成功/失败退出正控；若设计或scope变化先修订Plan。
### 规范增量
| ID | 操作 | target | before/after | 理由 | AC |
| SD01 | 待诊断 | docs/specs/auto-test-case/build-storage-lifecycle.md | 暂不改既有规则；若发现构建恢复合同缺口再提出精确delta并复审 | 当前只有首次崩溃观察 | AC01/02/04 |
## 6. 测试设计
原精确发现命令为反例；单独编译同单元、同轮成功case-parent为对照（配置差异必须列明）。保原时限/样本和工具版本；不得把切换toolchain/关闭incremental当无说明PASS。最终受影响完整组至少cargo t auto-api-gen，增加组须由实际修复触面解释；不重复cargo t all，周期门独立。
## 7. 验收标准
AC01..05以实际来源/进程退出/发现的完整用例列表/原例恢复与负控为准。当前全部未执行，无修复或环境归因结论。
## 8. 执行步骤
- [ ] T01(work) 原输入和编译单元重建（AC01）。
- [ ] T02(work) 有界因果对照（AC02）。
- [ ] T03(work) 必要最小修复与完整owner组（AC03）。
- [ ] T04(review) 复审（AC04）。
- [ ] T05(merge) 实际门、交付与归档清理（AC05）。
## 9. 复审记录
stage:new | r1 | drafted only；未独立复审，未实施。
## 10. 待澄清事项
具体故障层/复现条件未知；不是要求用户先提供环境信息，可由T01本机取证开始。实施前遵守新计划授权/确认规则，不将batch登记等同实施批准。

原始证据：
- discovery.command.json: 1128 bytes，SHA256 `c740cdcb7f09310cd0ba04f93a4568872b8223fca7b78fa0888962586785f47f`
- discovery.stderr: 24980 bytes，SHA256 `cfcdb61c2f62286a1165f372f2641455381e9327c08547001b9dfa202cc76e9a`
- discovery.stdout: 0 bytes，SHA256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- result.json: 121 bytes，SHA256 `47cc882eb68bca19fd8fdb74468b47e820546a995b8df5d8905e2de9d956a1ee`
