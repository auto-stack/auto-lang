---
plan_id: PLAN-667
status: reviewed
feature_name: 现有内存安全底线加固与支持边界
作者说明: 本计划仅处理当前机制漏洞；下一版内存模型另案设计。
author: [Codex]
created_at: 2026-09-20
updated_at: 2026-09-20
plan_revision: 1
current_step: 5
total_steps: 6
supersedes_spec_components:
  - docs/specs/auto-lang/types/design/ownership.md
  - docs/specs/auto-lang/trans/design/escape-analysis-tiers.md
  - docs/specs/auto-lang/vm/architecture.md
new_spec_components:
  - docs/specs/auto-lang/vm/design/memory-safety-boundary.md
touched_goals: [GOAL-001]
affects: [auto-lang/types, auto-lang/trans, auto-lang/vm]
---

# [PLAN-667] 现有内存安全底线加固与支持边界

## 0. 变更摘要

本版目标：已支持的行为有可核查保证，未支持的危险行为在触碰失效引用前明确受限。
修复/限制现有 VM 捕获、引用计数及 a2r 逃逸降级路径；不承诺完成 Rust 等价的全语言内存安全证明。
本计划是 L1 现有机制修复与边界收紧，不更换 VM 所有权协议或公共字节码架构；若调查发现必须进行此类 L2 重构，停止对应实现、按 AGENTS.md 先建设计文档并修订本计划，不能隐式扩大范围。

## 1. 目标

- 将静态审计发现转换为最小复现、明确判定及闭环修复/拒绝。
- 保留现有已验证局部捕获与同步调用行为；对无法安全维持的情况提供可定位诊断。
- 不以未计数别名的短期存活、改变共享语义的 clone、静默 nil/0 或生成代码碰巧通过代替正确性。
- 区分运行时引用失效、错误结果、生成 Rust 编译失败、泄漏和性能问题。
- 非目标：新增 lifetime/from/weak/parent/extract/region 语法；全新 borrow checker；消除现有 RC；重新定义普通赋值语义；AAVM 自举；全仓 unsafe/FFI 安全认证。

## 2. 架构方案

沿用现有 AST、VM、a2r 管线。优先修复局部实现；不能证明支持的入口 fail-closed。
VM 优先在编译入口识别不安全捕获流；静态无法覆盖的帧捕获必须在读写前验证任务身份与帧实例/代际（不能仅判断 bp/sp 范围），或拒绝相应能力。运行期诊断属于明确的受限能力，不能对外称为完整静态借用保证。
RC 修复以每个活引用具备正确持有关系为底线。不得直接删除宽限窗来宣称修复；必须先定位未计数引用的生产/转移/释放路径。延迟回收如保留，只能是对不可访问对象的策略，不能承担有效引用保活。
a2r 逃逸分析是生成决策辅助，不是完整 borrow checker；保留 safe Rust/rustc 边界，同时拒绝已知不能保持语义的 fallback。

## 3. 技术栈

Rust；既有 nextest/cargo t、tv、tt、tf；Auto .at 回归语料；a2r 生成 Rust 后真实编译/执行。
实现、构建和测试仅在 D:/autostack/.wt/lang-667/auto-lang（plan-667-dev）进行；主检出只维护计划及合入时 Specs。

## 4. 需求分析与背景调查

### 授权与范围

用户于本会话明确授权先做基础漏洞封堵，将完整内存机制设计及新语法/检查器留至下个版本。
本计划仅涉及 auto-lang 仓，允许既有机制修复、必要限制、测试和准确的文档边界；没有授权静默改变值/共享语义，也没有预算或自动续跑上限。
AGENTS.md §1 L1 要求计划呈交确认后执行；本修订提交审阅，尚无对 revision 1 的确认。用户已有的第一阶段范围授权保持有效，不重新询问是否要做安全加固。

### 基线与来源

主检出 HEAD：899db807f698491b6bd72c3d3791f02949efa6cf。取号前唯一已存在工作区改动为 examples/rust-workspace/Cargo.toml，不属于本计划，不修改/提交。
已读 docs/specs/overview.md、auto-lang/types/overview.md、vm/overview.md；约束取自 AGENTS.md 和 .cargo/config.toml。
内容 SHA256：
- docs/specs/auto-lang/types/design/ownership.md：599838C938CD296483D7849A5D49D47E32DBF6AA5EC8736724602C4FCDDAC600
- docs/specs/auto-lang/trans/design/escape-analysis-tiers.md：B50EF7EC038BEFA63DC8CE70A49A98F21D685C563A1871307C6ACF829A5029F4
- crates/auto-lang/src/vm/rc.rs：BC4264E987FFF9F1A3240632AE0265F17AE54E449FA75C90F4885E5708FB7E7A
- crates/auto-lang/src/trans/escape/analyzer.rs：C1A9B2D5A6E7B9ED84E96D65BB75D871956C458C6E189748C7335426DD342048

### 候选问题（未复现不等于已证明宿主 UB）

| ID | 代码/来源 | 静态事实及调查目标 |
|---|---|---|
| F-01 | vm/engine.rs Closure.capture_slots/param_abs、CAPTURE_VAR 及捕获写入；KNOWN-DEBT-AND-RISKS.md Plan 385 | 保存创建者栈槽；验证返回后、复用同 bp、跨任务调用及嵌套闭包失效 |
| F-02 | vm/rc.rs rc_release_id/reap_dying/reap_all/rc_stats | 8192 步宽限补偿 raw 别名；rc_stats 强制回收；查持有协议缺口及观测副作用 |
| F-03 | trans/escape/analyzer.rs find_escapes/gather_var_refs | 闭包传入收集器后被跳过；验证普通读取/写入/嵌套/遮蔽捕获 |
| F-04 | escape_map.rs BindingId；analyzer.rs introduce/apply_lowering/默认 match 臂 | 名字+深度不区分兄弟作用域，未覆盖 AST 静默忽略，不逃逸被当作安全借用 |
| F-05 | trans/rust.rs emit_borrow；Plan 310 | ArcMutex 仅 clone，普通逃逸默认 Clone，完整 RC/Arc 回退保证不成立；验证 mut/别名行为 |
| F-06 | ownership/lifetime.rs、borrow.rs、cfa.rs 及生产调用搜索 | 数字 ID 不等于区域包含；当前独立模块不构成主路径完整执法；不得直接接入作为安全证明 |
| F-07 | vm/rc.rs heap_ref_id/is_heap_ref_nv | 大整数与旧式裸 heap id 判别可能冲突，需构造与实际活 id 相等的整数并核对持有计数 |

Specs 存在漂移：自动 Arc/RC fallback 与代码不符，VM 旧闭包天然安全描述与栈槽捕获不符。Plan 310 是历史来源，不作为当前已实现能力证明。

## 5. 详细设计

### 支持边界与失败策略

新增报告 docs/plans/reports/667-memory-safety-evidence.md（执行阶段创建）。逐项记录：后端/入口、最小源码、基线结果、期望、根因、修复或限制、实际错误阶段、测试名、提交与命令。
边界分类只有：已验证支持 / 明确拒绝或受检运行期错误 / 不属于本次保证。不能把未调查写成安全。
拒绝路径必须通过生产入口覆盖，不能仅添加测试开关；对持久会话、回调及任务路径分别说明适用性。既有可正常使用的无捕获/创建者存活捕获应有正例保护。

### VM 捕获与 RC

T-01 在不改架构前提下选定最小可行修复或守卫，记录对现有闭包用户的影响。只在检测到不安全行为前失败，不允许读取失效槽后再诊断。保存值副本只有符合原语义时才允许，不能用快照替换可变别名。
RC 审计覆盖本次复现触及的 LOAD/STORE/POP/RET、容器替换和捕获、native/FFI 交接边界；若暴露同族缺口，必须修复或关闭可达入口，不以无限扩展整个 FFI 审计代替交付。检查 retain/release 与回收并发的适用线程模型，发现竞争需测试并处理或明确限制。
统计应与回收动作分离；保留显式 drain/收尾操作供测试，不让读取健康统计改变被测程序生命周期。

### a2r 分析与降级

为节点/绑定采用真实唯一身份或等效可靠的解析映射，同时让分析和生成使用同一身份；不只修 map 的键。
补全捕获自由变量和相关 AST 遍历，包含命名参数、块、条件、返回、循环及遮蔽；未知或暂未覆盖的危险形式不能被标为已证明安全。
修复不代表实现完整 NLL。对可能改变 mut/共享行为的 clone、未实现 Arc 共享生成，不再用 warning 伪装支持；提供可定位错误及现有可行写法。普通 safe Rust 生成仍以实际编译为门禁，不能只比对文本。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/types/design/ownership.md | 理念/独立模块与实际主路径保证混用 → 已接入/未接入/限制清楚分列 | 不夸大保证，不定新模型 | AC-01,AC-06 |
| SD-02 | modify | docs/specs/auto-lang/trans/design/escape-analysis-tiers.md | 无条件 RC/Arc 可用及不逃逸即安全 → 实际 fallback 条件、拒绝与 rustc 边界 | 与实现一致 | AC-04,AC-05,AC-06 |
| SD-03 | modify | docs/specs/auto-lang/vm/architecture.md | 捕获安全笼统描述 → 当前捕获有效期、守卫和关闭入口 | 当前状态准确 | AC-02,AC-06 |
| SD-04 | add | docs/specs/auto-lang/vm/design/memory-safety-boundary.md | 无统一支持边界 → 后端入口/持有协议/回收及诊断边界矩阵 | 本版安全底线 | AC-01,AC-02,AC-03,AC-06 |

合入时同步相关 overview、plans.md、.autoos/specs.json；纠正 docs/design/04-memory-ownership.md 的当前状态描述，不加入下版候选设计决策。

## 6. 测试设计

- 新增 Rust 回归模块 crates/auto-lang/src/tests/plan667_memory_safety_tests.rs，在既有测试入口注册；按现有 harness 调用生产 VM/a2r 路径。具体 harness 名在 T-01 从仓库核实，不自建解释器。
- 捕获矩阵：局部即调、返回后调用、嵌套捕获、参数/局部/同名遮蔽、创建帧复用、持久回调、跨任务来源；读和写、整数和堆对象均覆盖。
- RC 矩阵：合法别名跨越超过 8192 解释步后仍有效；无引用对象能按规则回收；统计前后行为一致；显式收尾 live shares 无新增泄漏；与活 heap id 相等的整数不能冒领/扣除持有份额。
- a2r 矩阵：只读捕获、写捕获、嵌套及兄弟同名绑定、返回 .view/.mut、未知调用、Send 边界、未支持 AST；正例真实 rustc 编译运行且结果符合语义，负例 Auto 诊断或明确声明的下游 rustc 边界（不能把已知有害 fallback 推给 rustc）。
- 补充单元测试验证分析，但不以枚举 tier 断言代替端到端结果。
- 本阶段仅计划文档，不运行 cargo 测试。执行期先 cargo check -p auto-lang 和 cargo t plan667；最终 cargo tv、cargo tt，若触及 VM 核心协议则最终一次 cargo tf。变更文件格式检查，不全仓格式化。
- 不触及 AAVM 路径则不跑 taa；不改文档生成/schema/语法参考则不单独跑 docs_gen；既有 alias 自带的测试按仓库规则执行。

## 7. 验收标准

- AC-01：F-01..F-07 每项有源码定位、最小探针和结论；确认问题已修复/明确受限，否定问题有证据；没有仅靠登记债务继续执行危险路径的情况。
- AC-02：捕获矩阵中合法情况保持结果；失效、跨任务错帧、同 bp 复用等在任何槽位读写前拒绝；不静默回落 nil/旧值/副本。适用入口在矩阵中完整列出。
- AC-03：本次所支持持有路径不依赖宽限时长维持安全；边界前后、统计插入和收尾测试通过，计数误识别探针通过。未计数有效别名必须消除或关闭来源；仅延长窗口不合格。
- AC-04：兄弟作用域和嵌套遮蔽绑定互不污染；闭包读取/写入捕获及危险未覆盖节点不再漏报；至少一个真实源码复现覆盖每类修复。
- AC-05：已支持 a2r 正例实际编译执行通过；不支持的共享/Arc/可变借用降级明确诊断；不得用 clone 改变原本可观察的别名修改行为。
- AC-06：Specs/设计状态、代码、测试的支持矩阵一致；不得宣称完整 Rust 级安全、全自动 RC/Arc、或一层调用者分析已实现。下一版语法均未实现。
- AC-07：适用门禁通过，无新增警告/调试输出；既有失败有同基线复现和影响分类，不能掩盖相关回归。独立 review 绑定本修订及最终代码版本后方可合入。

## 8. 执行步骤

| ID | 依赖 | 文件/符号与操作 | 结果及验收 | 验证 |
|---|---|---|---|---|
| T-01 | revision 1 确认 | [x] worktree .wt/lang-667（组内补 auto-down 依赖位）；21 探针模块+evidence 报告；F-01..07 全部静态实锤（另发现 STORE_CAPTURED 双 pop/标签截断/env 漏 stake/祖父帧错锚 四个同族缺陷）；修复策略=帧身份表+RC 三重门+analyzer 下降/折叠/fail-closed | 调查决策不超过本计划范围；AC-01 | cargo check 绿；红基线 12 红 9 绿在案（evidence §1） |
| T-02 | T-01 | [x] task.rs frame_ids 帧身份表+engine.rs 守卫/CLOSURE/LOAD/STORE_CAPTURED 重写；提交 f181051ba | 捕获矩阵 7/7 绿（正例 3 保护+拒绝 4）；AC-02 | cargo t plan667 绿；closures/known_limits 回归绿 |
| T-03 | T-01,T-02 | [x] rc_stats 纯化+rc_retain_id 存在性门+rc_push_slot 份额出处门+防御补持收紧 tagged+stdlib 裸推迁移；提交 5c8f5ce8e | RC 矩阵 5/5 绿；P419_UAF_TRACE 事件链取证整数零 retain；AC-03 | cargo t plan667 绿；cargo t plan510 8/8 绿 |
| T-04 | T-01 | [x] analyzer 闭包自由变量下降+fold_to_root+Try/Reply+fail-closed+emit_borrow 双拒绝；提交 faaeeaf0e+2bb8771cf（R-1 修复） | a2r 矩阵 9/9 绿+rustc 实编正例绿；AC-04,AC-05 | cargo t plan667 绿；cargo tt 与 stash 基线零差 |
| T-05 | T-02,T-03,T-04 | [x] SD-01..04（含新文件 memory-safety-boundary.md）+evidence 完整化+design/04 纠正+P667-D1/D2 债入册；提交 bc186a39b | 支持矩阵及门禁结果；AC-01..07 | tv 仅 2 预存红；tf(no-fail-fast) 仅 6 预存红（stash 双跑归因，零新增）|
| T-06 | T-05 | auto-plan-review 独立复审，逐 AC/遗漏/workaround/警告检查；通过后 auto-plan-merge 沉淀 Specs、归档、guard 后清理 | 修订与代码版本绑定的 review pass；AC-07 | review/merge 规定检查；python scripts/spec-index.py；bash D:/autostack/wt-guard.sh D:/autostack/.wt/lang-667/auto-lang 必须 clean |

不因单个失败难复现而删除验收项；不在主检出实现/测试；不创建 worktree junction/symlink。

## 9. 复审记录





2026-09-20 merge 收据（auto-plan-merge，键 PLAN-667:r1）：
- prepared ✓：delivery 候选=worktree 新提交（docs-only 后继于 reviewed 2bb8771cf）——specs.json P667-1/P667-2（614→616）+vm plans.md 667 行+INDEX 再生（664 ui 注记 verbatim 回灌，P665-D6 四复发拦截）；spec delta 四文件即 reviewed 冻结版（哈希见二轮复审）。

2026-09-20 第二轮复审（R-1 修复后）：
- stage: review
- plan_id: PLAN-667
- plan_revision: 1
- outcome: pass
- reviewed_commit: 2bb8771cf（=bc186a39b+R-1 探针修复；基线 21f0b7f72）
- base_commit: 21f0b7f72
- dependency_revisions: auto-down detached fba6563ed
- spec_inputs（SHA256 前 16，@2bb8771cf）: ownership.md B376BC77A3C21D95 / escape-analysis-tiers.md D546B21028CFBC48 / vm architecture.md 8199EC6C93054B2A / memory-safety-boundary.md 825740A4E996D342
- acceptance_results: AC-01..AC-07 全 pass
- findings: R-1 已修复并哨兵法验证非空泛（SENTINEL_XYZ 替换即红，恢复即绿）；evidence §4/§7 同步真实边界（a2r 解析层即拒 `.go`）
- evidence: cargo t plan667 21/21（@2bb8771cf 重跑）；tt 复验仅 6 预存红（stash 双跑归因在案）；rustc 实编门禁绿；diff 零调试输出；plan510 8/8；AC-03 live_shares 由 plan510 三处 ==0 覆盖
- next: auto-plan:merge 沉淀 Specs、归档、wt-guard 后清理 worktree

2026-09-20 第一轮复审（auto-plan-review，同会话声明：结论自工件/门禁重跑重建，不采信执行自述）：
- stage: review
- plan_id: PLAN-667
- plan_revision: 1
- outcome: needs_fix
- reviewed_commit: bc186a39b（基线 21f0b7f72；依赖 auto-down detached fba6563ed）
- spec_inputs: types/design/ownership.md、trans/design/escape-analysis-tiers.md、vm/architecture.md、vm/design/memory-safety-boundary.md（worktree 版）
- acceptance_results: AC-01 pass/AC-02 pass/AC-03 pass/AC-04 pass/**AC-05 partial**/AC-06 pass/AC-07 pass
- findings: **R-1（medium）** plan667_a2r_go_capture_rejected 空泛通过——源 `fn() -> str` 在 a2r 解析即败（Expected term, got Arrow），Err 臂 `contains("go")` 空泛命中 "got Arrow" 的 "go"；F-05 ArcMutex 拒绝路径无有效动态探针。证据：同模块同参诊断 REVIEW-ERR=Expected term, got Arrow（复现两次）；`.go` 后缀为 VM-dest 专属解析（parser.rs:3055 经词法 Go 关键字，TransRust 下走字段访问路径拒绝）。修法：探针改为 `s.go`+`s.view` 真实形态，断言收紧到 `contains("Go")`（大小写敏感，避开 got）或 Send/Arc 专名，注释钉定"a2r 解析层即拒=Send 边界的实际生效层"。
- evidence: 复审重跑 cargo t plan667 21/21、plan510 8/8、rustc 实编门禁绿；diff 审计零调试输出新增；plan510 live_shares==0 三处覆盖 AC-03 收尾项；mut 探针经插桩确认真触达 emit_borrow 拒绝消息（REVIEW-MUT-ERR 全文在案）。
- next: work 修复 R-1（T-04 重开，1 个修复周期）→ 复审补验 → pass 后 merge。

2026-09-20 work 执行记录（auto-plan-work）：
- stage: work
- plan_id: PLAN-667
- plan_revision: 1
- outcome: pass
- code_commit: bc186a39b（T-05；前序 f181051ba T-02 / 5c8f5ce8e T-03 / faaeeaf0e T-04；基线 21f0b7f72）
- worktree: D:/autostack/.wt/lang-667/auto-lang（plan-667-dev；组内 auto-down 依赖位 detached fba6563ed）
- task_ids: T-01..T-05 完成；T-06（独立复审+merge）待 auto-plan-review/auto-plan-merge
- evidence: docs/plans/reports/667-memory-safety-evidence.md（21 探针全绿、门禁账、支持边界矩阵、既有红 stash 双跑归因）；探针 crates/auto-lang/src/plan667_memory_safety_tests.rs
- 门禁：cargo check 零新增警告；cargo t plan667 21/21；plan510 8/8；tv 零新增红（2 预存 ui_gen）；tt 零新增红（4+2 预存）；tf(no-fail-fast) 零新增红（6 预存）；rustc 实编正例绿（--run-ignored=only）
- blockers: 无
- next: auto-plan-review 独立复审（绑定 revision 1 + bc186a39b）→ auto-plan-merge 沉淀/归档/清理

2026-09-20 起草自检：任务覆盖 AC-01..07 和 SD-01..04；候选漏洞与动态证据明确分离；第一阶段范围已授权，revision 1 按仓规待确认。无代码实施及安全保证完成声明。
- stage: new
- plan_revision: 1
- outcome: blocked
- next: 用户确认此执行契约后交 auto-plan-work；首任务 T-01 为复现与范围内修复策略，不进入下一版模型设计。

## 10. 待澄清事项

- 必需：按 AGENTS.md §1 L1 确认本 revision 1。限制危险行为可能使原先侥幸运行的代码得到明确错误；该变化是本计划安全目标的一部分。
  - 2026-09-20 已确认：用户指令"计划667: 用 auto-plan-work 实施"即 revision 1 执行确认，状态翻 executing。
- 调查负责：T-01 确认捕获守卫的最小可行落点、RC 同族缺口实际可达入口、a2r 负例诊断落点；由执行代理提供证据，无需用户预先选择实现细节。
- 超范围：若必须更换闭包/字节码核心协议或重定义值/共享语义，由执行代理提出独立设计与计划修订，不能以本授权代替下一版本设计决定。
