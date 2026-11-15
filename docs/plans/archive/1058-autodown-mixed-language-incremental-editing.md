---
plan_id: PLAN-1058
status: archived
feature_name: AutoDown 混合语言增量解析与编辑定位原型
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-15
plan_revision: 1
current_step: 4
total_steps: 6
depends_on: []
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-syntax/autodown-mixed-editing.md]
touched_goals: []
affects: [auto-syntax]
---

# [PLAN-1058] AutoDown 混合语言增量解析与编辑定位原型

## 0. 变更摘要

在 auto-lang 的实验语法服务中，用现版 AutoDown 的中文正文、标题、代码围栏验证一次真实的混合语言编辑。编辑器消费者用同一源码修订查询标题大纲、光标所属语言、嵌入 Auto 的高亮范围和语法诊断。首轮交付可重放的库级原型，AutoEdit 产品接线另行决定。

## 1. 目标

首例从标题跳转到正文，进入 Auto 代码块修改标识符，再删除和恢复结束围栏。每次编辑后结果与从头解析该修订一致，并能观测实际重解析范围。代码围栏始终作为展示内容处理。只实现现版样例所需语法；不引入出版模板求值、PDF、外语 HIR lowering、全量 Markdown 或新的正文分隔符。

## 2. 架构方案

原文 snapshot → AutoDown 文档层划定标题、正文和围栏 → 按围栏语言选择子 parser → 给编辑器输出带来源的范围、查询结果和诊断。AutoDown 决定围栏归属与边界；嵌入 Auto 使用已有 GrammarProgram 和语法运行时。未装载语言标为其原始标签并保留正文范围，查询仍可返回父层数据。本期不把 TS 高亮引擎设为前置。

文档适配放在语法服务的消费者侧；运行时保留与 AutoDown 渲染器、AutoEdit UI、ACE 编译执行无关的依赖方向。首个可运行消费者是无 UI 的编辑会话测试，通过公开的打开、修改、定位、查询接口读结果。现版 AutoDown parser 用作格式参照；T-01 在已有模块中定位可复用规则，并记录原型对其支持的精确子集。

## 3. 技术栈

依托 `experimental/ace/crates/syntax-runtime/src/` 的 snapshot、position、edit、parse、outline 和恢复能力，以及 `experimental/ace/grammars/` 的 Auto 语法资产。复用 `LineIndex` 的 UTF-8/UTF-16 映射、`incremental_reparse` 和已有修订发布机制。测试归现有固定组 `cargo t ace-syntax`。

## 4. 需求分析与背景调查

- 用户 2026-11-14 授权在 auto-lang 建立本计划，实验对象为现版 AutoDown。本次只起草；auto-down 为只读格式来源。
- [仓库总览](../specs/overview.md)、[Design 39 §3–4](../design/39-ac-incremental-syntax-and-language-platform.md)、[现行语法规范](../specs/auto-syntax/project.md)及[源码快照与 CST](../specs/auto-syntax/snapshot-cst.md)给出可复用的修订、容错和局部更新机制。[788](archive/788-autodown-ac-native-turnaround.md)使用 AutoDown 描述 AC 测试，验收对象与本计划的文档编辑不同。
- AutoDown 的 [parser 出口](../../../auto-down/autodown/packages/engine/src/parser.ts)提供 `parseDocument` 和 `parse_blocks`；[Auto 实现](../../../auto-down/autodown/packages/engine/auto/parser/markdown_parser.at)已有开闭围栏、语言标签及未闭合处理。格式预期以当前公开消费者行为和现有语料核定。
- [parity.ad](../../../auto-down/autodown/packages/engine/src/rust/tests/golden/parity.ad) 的 `hardbreak_and_fence` 是真实的 `ts` 围栏样例，`emit_ids` 提供标题/锚点。[Cards Probe.ad](<../../../auto-down/jade-garden/front/e2e/fixtures-pages/Cards Probe.ad>) 有真实中文正文与带锚点标题。前者是带用例分段标记的语料，后者是独立页面。保留原例，再从两者派生带 Auto 围栏的小文档，用现版 parser 确认；不把组合例描述为原有文件。
- [AutoDown Design 10 §4.2](../../../auto-down/docs/designs/10-autodown-publishing.md)把格式定义交给正文所有者。082–086 没有整包前置关系到本计划。082 交付后可追加真实正文与独立模板作为两个文件；新版 Node、卡片等有规范、样例及预期后追加。源位置可供后续 085 块级定位消费；此处不承诺预览页映射。

## 5. 详细设计

1. 固定源文件身份、grammar 身份和基线内容摘要；每次修改提交 base revision 和替换范围。所有对外位置含文件、revision、半开 UTF-8 字节区间。合法 UTF-16 光标位置经 LineIndex 映射；覆盖 CRLF、中文和非 BMP 字符。不将位置句柄用作内容 NodeId，也不引入出版 OccurrenceId。
2. 围栏开闭标记、信息串由父树所有，子树只处理正文。若解析需要去除缩进或归一化换行，保留到原文的逐段映射。空区域、紧邻结束标记的光标按明确的父子边界处理。未知标签/无标签行为由 T-01 与现版对齐。局部语法错误仍返回当前修订的可用区域，子诊断投影回宿主文档。
3. 普通块内编辑保留未变围栏子树，按新修订重映射偏移；围栏标签或终止条件变化重新判定受影响的后继区域，直到收敛或 EOF。保留真实扫描、重解析区间、子树复用数及回退理由。任何全量回退按实记录。
4. 在一次修订内发布一致的区域、大纲、高亮与诊断。迟到任务不得覆盖新结果。同一文件重新打开后，旧位置只对旧 snapshot 有效。

### 规范增量

| delta_id | 操作 | 目标 | 规则变化与理由 | 验收 |
|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-syntax/autodown-mixed-editing.md`（拟新增） | 从单语树契约扩展到已验证的有限父子语言区间、修订发布、坐标与查询。列明支持范围、真实例及回退条件，供 AutoEdit 消费。 | AC-01～05 |

## 6. 测试设计

T-01 建立永久小样例及独立预期，T-02～04 接入真实入口。（2026-11-15：V-01～V-06 全部实跑通过，见 §9。）以 AutoDown parser 核对它能表达的块种类、标题、围栏标签和内容；精确边界从样例原始字节手工标注，避免用被测实现反推。每个编辑步骤都跑增量和同输入全新解析，分别比较范围、大纲、高亮与诊断。以下均待执行。

| 用例 | 操作 | 独立可观察结果 |
|---|---|---|
| V-01 基线 | 打开原始 golden 分段及中文派生文档；在代码块内置入貌似标题的文本 | 标题大纲只含真实标题；已知 Auto 范围内有子 token，ts 未装载时保留标签和源码；解析和查询不求值 |
| V-02 局部修改 | 设置两个独立围栏并修改其中一个标识符，再在标题前插入中文 | 未改块子树真实复用，后移光标定位到原 token；普通块内编辑不得用全文件重解析通过此例 |
| V-03 围栏重分类 | 删除、恢复关闭标记，再将 Auto 标签改为未知标签 | 与现版未闭合语义和逐修订 fresh 结果一致，恢复后后继标题再次可导航；改标签后不残留 Auto 高亮 |
| V-04 坐标和错误 | CRLF、中文和一个 emoji 置于已标注 token 前；在子区内制造未完整的 Auto 表达式 | 合法字节边界与 UTF-16 位置双向往返；诊断原文件切片命中预期 token 或缺失位置，标记/围栏边界属于父层 |
| V-05 编辑会话 | 通过无 UI 消费者重放打开、编辑、点击标题和块内字符；先提交新修订再交付旧结果 | 输出只含当前文件和修订；旧结果不能覆盖；值、诊断和高亮不串文件 |
| V-06 反控 | 用错误字节偏移、强制全量模式和过期发布样例验证断言区分力 | 错位可被手标 oracle 发现；全量虽与 fresh 等价，仍不满足 V-02 的复用条件；迟到输出被识别 |

## 7. 验收标准

| ID | owner_stage | 完成条件 |
|---|---|---|
| AC-01 | work | V-01 以现版 parser 和来源摘要确认边界，原例及派生例可区分、可重放。 |
| AC-02 | work | V-02/03/06 逐修订等价，正常局部路径有真实复用，重分类范围及回退可解释。 |
| AC-03 | work | V-04/06 的手标坐标与诊断、原文切片全部吻合。 |
| AC-04 | work | V-05 从消费者入口验证标题导航、语言定位和修订隔离。 |
| AC-05 | review | 独立复核 AC-01～04、反控、支持子集及源码依赖；记录遗漏和待办。 |
| AC-06 | merge | 在冻结提交核验对应固定组、沉淀 SD-01、归档计划并按仓规清理。 |

## 8. 执行步骤

| ID | owner_stage | 前置 | 交付与验证 |
|---|---|---|---|
| T-01 | work<br>✅ | 无 | 从 §4 两份语料及 parser 出口记录实际版本/哈希，建立现版 oracle 和手标样例。建议新测试入口 `experimental/ace/crates/syntax-runtime/tests/autodown_mixed_editing.rs`；定向运行下述命令验证 V-01 基线（AC-01）。 |
| T-02 | work<br>✅ | T-01 | 以现有语法 runtime 和 `experimental/ace/grammars/` 实现有限父层区域、子解析和大纲/高亮查询。用 V-01/04 对照标记边界、未知标签和恢复（AC-01/03）。 |
| T-03 | work<br>✅ | T-02 | 在已有 edit/position 入口上接局部失效、子树复用、位置换算和计数。在同一测试入口验证 V-02/03/04/06（AC-02/03）。 |
| T-04 | work<br>✅ | T-03 | 以无 UI 编辑器消费者调用同一库入口重放 V-05，并与 V-01～06 汇总；运行 `cargo t ace-syntax`（AC-04）。 |
| T-05 | review | T-04 | 同一 worktree 独立复核实际 diff、断言、复用计量和固定组选择；审计失败/回退及未完成项，定稿规范影响（AC-05）。 |
| T-06 | merge | T-05 | 冻结提交运行或复用同身份的 `cargo t ace-syntax` 证据；合回主线、落 SD-01、归档及 guarded cleanup（AC-06）。 |

定向命令（拟新增测试落地后，从 auto-lang worktree 根执行）：`cargo test --manifest-path experimental/ace/Cargo.toml -p ace-syntax-runtime --test autodown_mixed_editing`。输出列实际执行数和命名用例；若实现触及其他真实消费者，按其修改面补固定组。开发树拟为 `D:/autostack/.wt/lang-1058/auto-lang`，分支 `plan-1058-dev`。首次实施时登记 `D:/autostack/plan-artifacts/plan-1058/`；一次性探针、计量和日志归该处。开工入口 T-01，无出版线等待项。

## 9. 复审记录

2026-11-15 stage: work / T-01～T-04 全部关闭（T-05 复审、T-06 归并仍 open）| plan_id: PLAN-1058 | plan_revision: 1 | outcome: pass（work 全部任务完成，4/6）| worktree: D:/autostack/.wt/lang-1058/auto-lang | branch: plan-1058-dev | base_commit: 104e4daa5 | code_commit: 978fd7dc0 | task_ids: T-01, T-02, T-03, T-04 | dependency_revisions: auto-down @ 5af11a39dd436acd9a25b523f675cec507819d2d（只读格式参照；markdown_parser.at sha256 43299345…2ffea、parser.ts 390ae092…108d2、parity.ad 01e920ed…17686、Cards Probe.ad 732fb665…aaf73）；ace-syntax-grammar 以 dev-dep 接入（Cargo 循环 dev 依赖合法，编译方向不变）；Auto 语法资产 grammars/auto-core.at 经 ace_syntax_grammar::compile 实编译 | decisions: ①父层子集=ATX 标题/段落/围栏（```与~~~、精确同 run 闭栏、EOF 未闭合尾部全标记行剥离+stripTrailingSpaceLine 镜像、信息串 trim=语言标签、` ^anchor` 提升）；setext/thematic/math/$组件/表格/frontmatter/列表不在子集（列表行按段落正文，`---` 差异已记录）——派生样例不使用未支持结构；②未闭合围栏取完整文档模式（isFinal=true）语义，流式 loading 标记不在本层；③行区间直接落原文字节（normalizeNewlines 语义镜像：
/孤立 
 均行界），围栏正文逐段 1:1 映射无去缩进；④父子边界：标记行/行界/空行属父层，content_end 紧邻闭栏属父层，空正文零宽内部点报围栏语言；⑤增量重扫起点=含编辑行的最近区域起点，其上邻段落紧邻时再后退一步（段落吸收向后只达一行）；收敛=新扫描位置与旧区域起点按总位移对齐且旧区域全在未触尾部；⑥发布屏障按 (FileId,修订) 键控（语义同 edit::VersionedPublish：首个当前修订结果胜出，迟到入 late 表）| evidence: ①oracle：现版 parser 逐例确认 7 个样例（含 V-03 删闭栏吞并与改标签两景）→ D:/autostack/plan-artifacts/plan-1058/confirm_oracle.out；②实现：experimental/ace/crates/syntax-runtime/src/autodown.rs（新模块）+ tests/autodown_mixed_editing.rs（V-01～V-06）+ lib.rs 注册 + Cargo.toml dev-dep；③定向命令 `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-syntax-runtime --test autodown_mixed_editing` 6/6 PASS；④固定组 `cargo t ace-syntax` PASS（154 executed，含 autodown 9 例=6 集成+3 单元；results.json run-8_yztlnd，wall 24.2s；首跑 run-6lb9fzuq 48.0s）；⑤V-02 真实复用=未改围栏子分析 Rc::ptr_eq 命中 + rescan 严格子区间；V-06 强制全量=与 fresh 等价但 reused_subtrees=0、rescan=(0,len)，不满足 V-02 复用条件（反控区分力成立）；⑥每编辑步 fresh↔incremental 等价断言全绿（equivalent_to：区域/大纲/高亮/诊断）| test_scope: 新用例 6 集成 + 3 单元全部 ≤0.02s/例（nextest 逐例：v01 0.015 / v02 0.014 / v03 0.011 / v04 0.010 / v05 0.008 / v06 0.019；单元 3 例 0.00s）；无 >10s 用例；准备=冷编译一次，运行=上述；无资源等待与清理超时；复用证据：ace-syntax 组其余 145 例为基线随组通过 | health: cargo fmt 本包 clean；clippy 新文件 0 警告（crate 既有基线警告未触碰）；无调试输出残留 | raw_evidence: D:/autostack/plan-artifacts/plan-1058/{confirm_oracle.mjs, confirm_oracle.out, tsmap.mjs, register-tsmap.mjs}（merge 收尾折叠后删除）| blockers: none | next_responsible_role: review（T-05：同树独立复核 AC-01～04、反控、支持子集与源码依赖）

stage: goal | plan_id: PLAN-1058 | plan_revision: r1 | round: 1 | from: work | to: review | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1058/auto-lang | commit: 978fd7dc0 | next: review


stage: review | review_scope: final | reviewer: independent fresh-context agent（agent_8b3419ff，无实施上下文；探针于 plan-artifacts 内执行后自删）| plan_id: PLAN-1058 | plan_revision: 1 | outcome: **needs_fix** | worktree: D:/autostack/.wt/lang-1058/auto-lang | branch: plan-1058-dev | reviewed_commit: 978fd7dc0 | base_commit: 104e4daa5 | findings: **F1（MAJOR，AC-02/§5.3）**=首区域之前（前导空行区）编辑：partition_point==0 时 rescan_from=regions[0].start>lo，debug 断言 panic、release 静默发散且 fallback 未记录（探针实证：`"
para
"` 在 (0,0) 插入 "X" → 增量 [Paragraph(2,6)] ≠ fresh [Paragraph(0,6)]）；修法=该情形从 0 重扫+回归测试。**F2（MINOR）**=lift_anchor 字符集宽于现版（extractAnchorBlock 限 [A-Za-z0-9_-]）→ 镜像。**F3（MINOR）**=未闭合围栏末行剥离用 trim_ascii（含 tab），现版 stripTrailingSpaceLine 只剥空格(0x20) → 镜像为仅空格。**F4（MINOR）**=§5.2 空正文零宽点规则已实现（autodown.rs cs==ce 分支）但无测试钉住 → V-01 补断言。**F5（MINOR）**=§9 决策⑥"(FileId,修订) 键控"措辞 vs 实现（每文件槽首个当前修订胜出，更新修订也被 late）→ 对齐：当前修订且更新者可替换发布（旧/重复仍 late），补测试。| acceptance_results: AC-01 pass；AC-02 pass（已测范围，F1 在外）；AC-03 pass；AC-04 pass（方法与证据见代理报告，逐条独立复核：手标 oracle 全部 std find、语义逐行对拍 markdown_parser.raw.ts、run-8_yztlnd 门禁属实）| test_cost: 复跑 6/6 PASS 总 0.019s（v01 0.014/v02 0.011/v03 0.009/v04 0.010/v05 0.007/v06 0.018），全部 ≪10s | unexamined: 组内其余 145 例凭 results.json 复用未重跑；多编辑 EditRequest 未专测（收敛界保守成立）；F5 场景原未测 | next: work（同树修复 F1～F5，重跑受影响门禁后复审）| 2026-11-15

stage: goal | plan_id: PLAN-1058 | plan_revision: r1 | round: 2 | from: review | to: work | status: archived | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1058/auto-lang | commit: 978fd7dc0 | next: work


2026-11-15 stage: work / 复审修复轮（F1～F5；T-02/T-03/T-04 重开后再关闭）| plan_id: PLAN-1058 | plan_revision: 1 | outcome: fixed（待复审核销）| worktree: D:/autostack/.wt/lang-1058/auto-lang | branch: plan-1058-dev | reviewed_commit: 978fd7dc0 | code_commit: 10af3e73e | findings_closed: **F1**=incremental_regions 判 leading（partition_point==0，编辑在首区域之前的前导空行区）→ rescan_from=0、稳定前缀为空；回归 v02_regression_edit_before_first_region（`
para
` 插 X@0 → 增量=fresh、区域 (0,6)；前导空行+标题+auto 围栏 → 子树 Rc 复用=1）。**F2**=lift_anchor 镜像 extractAnchorBlock 字符集 [A-Za-z0-9_-]（V-01 增 `# 题 ^中文锚` 反例：不提升）。**F3**=未闭合围栏末行剥离改 `lb.iter().all(|&c| c==b' ')`（仅空格 0x20；tab 按内容保留，镜像 stripTrailingSpaceLine）。**F4**=V-01 钉住空正文围栏：零宽内部点 language_at=FenceBody{auto, loaded:true}、embedded=Some（空分析）、高亮 0。**F5**=deliver 键控对齐 (FileId,修订)：当前修订且 revision>已发布 → 替换发布；同修订重复/过期 → late；V-05 增 rev2 替换发布 + 重复 late 断言。| evidence: `cargo test -p ace-syntax-runtime --test autodown_mixed_editing` 7/7 PASS；`cargo t ace-syntax` PASS（run-05wfvazh：155 executed，含 autodown 10 例=7 集成+3 单元，wall 49.9s）；clippy 新文件 0 警告；fmt clean | test_scope: 新增 1 例 v02_regression（0.01s 量级，见下轮复审逐例计时）；其余 6 例断言扩展后复跑全绿 | blockers: none | next_responsible_role: review（同独立上下文核销 F1～F5）

stage: goal | plan_id: PLAN-1058 | plan_revision: r1 | round: 3 | from: work | to: review | status: archived | outcome: fixed | worktree: D:/autostack/.wt/lang-1058/auto-lang | commit: 10af3e73e | next: review


2026-11-15 stage: review | review_scope: final（第二轮，同独立上下文 agent_8b3419ff 核销）| plan_id: PLAN-1058 | plan_revision: 1 | outcome: **pass** | worktree: D:/autostack/.wt/lang-1058/auto-lang | branch: plan-1058-dev | reviewed_commit: 10af3e73e | base_commit: 104e4daa5 | findings_closed: F1（MAJOR）独立探针 debug+release 双模式复验：无 panic、equivalent=true、fallback=None（真收敛非回退掩盖）；F2 字符集镜像逐字符对拍 extractAnchorBlock（markdown_parser.raw.ts:2997）；F3 仅剥空格 0x20 镜像确认（含 end==0→保留 单行守卫）；F4 空围栏零宽点两侧边界（子层 FenceBody/父层 FenceMarker）均被钉住；F5 (FileId,修订) 键控与 V-05 断言一致 | residual_nonblocking: ①DeliveryOutcome::Published 变体注释与 published 字段注释仍写"首个当前修订胜出"（deliver() 自身注释已正确）——纯注释滞后，语义以 deliver() 与 SD-01 为准；②F3 的 tab 反例无专测（镜像已代码级对拍）；③多编辑 EditRequest 未专测（收敛界保守成立）——均记录不阻塞 | acceptance_results: AC-01～04 全 pass（第一轮全量独立核验 + 第二轮修复核销）；AC-05 本记录关闭 | test_cost: 复审自跑 7/7 PASS 共 0.019s（v03 0.008/v02 0.010/v05 0.012/v04 0.013/v06 0.014/v02_regression 0.017/v01 0.019），全部 ≪10s | evidence: 冻结提交 10af3e73e 上 `cargo t ace-syntax` PASS（run-05wfvazh：gate_passed=true、155 executed、autodown 10 例、wall 49.9s，同身份复用为 merge 门）；clippy 新文件 0 警告、fmt clean、树 clean | pending_merge_gates: SD-01 沉淀（docs/specs/auto-syntax/autodown-mixed-editing.md 拟新增）、归档、guarded cleanup | next: merge（T-06）

stage: goal | plan_id: PLAN-1058 | plan_revision: r1 | round: 4 | from: review | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1058/auto-lang | commit: 10af3e73e | next: merge


2026-11-15 stage: merge | plan_id: PLAN-1058:r1 | reviewer: 独立上下文两轮（agent_8b3419ff：第一轮 needs_fix F1～F5 → 修复 10af3e73e → 第二轮 pass）| checkpoints: prepared=冻结 Spec delta SD-01（docs/specs/auto-syntax/autodown-mixed-editing.md 新增 + plans.md 行 + project.md 关联，worktree 内提交 f71d404ff，docs-only 后裔=delivery 候选）；landed=master 基线漂移至 64f86fda8（全部为 979 docs 簿记，与代码触面零重叠）→ rebase 3 commits 全等（range-diff 全 `=`：978fd7dc0→59401b84d、10af3e73e→194622147、f71d404ff→c3b1a5d06）→ `git merge --ff-only plan-1058-dev` 成功，master tip=c3b1a5d06（delivery commit，线性历史无 merge commit）；canonical_verified=docs/specs/auto-syntax/autodown-mixed-editing.md（sha256 972ef08f…a91dc，属 c3b1a5d06）+ plans.md（01f604db…ec6d）+ project.md 关联行 | gate: 冻结固定组 `cargo t ace-syntax` 于 delivery 前置 10af3e73e（run-05wfvazh PASS 155）与 delivery 候选 f71d404ff（run-oh4j5v3l PASS 155，wall 19.3s）各跑一次（rebase 后提交哈希变化，二次绑定新基线；代码与 run 输入逐字节同 patch）；landing smoke=主检出 `cargo test -p ace-syntax-runtime --test autodown_mixed_editing` 7/7 PASS（0.01s）| next: archive + cleanup

stage: goal | plan_id: PLAN-1058 | plan_revision: r1 | round: 5 | from: merge | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1058/auto-lang | commit: c3b1a5d06 | next: merge

## 10. 待澄清事项

没有阻塞 T-01 的产品决策。未闭合围栏的流式与完整文档模式如有差异，由 T-01 对照现版固定所选模式，列明另一模式的边界。外部语言包、外语子集进入 Checked HIR、AutoEdit 正式替换及新格式适配按各自后续范围评估。