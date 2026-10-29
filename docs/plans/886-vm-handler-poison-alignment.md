---
plan_id: PLAN-886
status: execution_done        # drafting → executing → execution_done → reviewed → archived
feature_name: vm-handler-poison-alignment
author: [ZCode]
created_at: 2026-10-29
updated_at: 2026-10-29

# /auto-plan:review 结束时填写：
supersedes_spec_components: [docs/specs/auto-vm/project.md, docs/specs/auto-ui-runtime/project.md]
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm, auto-ui/runtime]
current_step: 5
total_steps: 5
---

# [PLAN-886] vm-handler-poison-alignment

## 变更摘要

计划以「VM 轨 handler 导出与 FN_PROLOG 错位」立项（auto-reader READER-001 的跨仓缺陷 N-r8-2/N-r8-2a 路由），**结论改判：不是编译器缺陷，而是应用侧非法 `let` 重赋值 + 框架侧静默失败可见性缺陷**。

- 根因（已定案）：reader `reading_page.at` F-14 身份守卫对 `let pr` 重赋值（`pr = null`）。新 CLI（auto-man 0.1.0+v0.0.20-117-g873fa23）按语言语义强制 `let` 不可变 → 该 handler 的合成在编译期失败 → 整个 `handler_reading_page_Init` 未产出 prolog（Plan 423 P5 记录的"导出已记录、prolog 未插"半成品形态）→ `poisoned_fn_exports` 按设计丢弃该导出。**毒化判定本身正确**，改 `var pr` 即愈（auto-reader plan-001-dev @268c7d4，原生驱动 16/16）。旧 CLI（2785）对 `let` 宽松，故该缺陷此前从未暴露。
- 真实缺陷（本计划交付）：失败完全不可见——`record_synth_failure` 只写 `log::warn`，宿主 CLI 未接 logger → 运行期唯一现象是"页面挂载但内容空"，无因可查；codegen 侧消息还把人引向并不存在的 "earlier handler errors"。reader 复审为此多轮外部二分。
- 交付（诊断可见性面，Plan 492 M5 的延续）：①auto-ui：`record_synth_failure` 恒向 stderr 输出 `[SYNTH-FAIL] <原因>`，并在**两条** VmBridge 合成路径（AURA 树 / decl 直读）统一 drain 已收集失败；②auto-lang：`[CODEGEN] dropping poisoned export` 消息补 entry 地址、该字节 opcode、code_len、±4 字节窗口（`*` 标 entry），并如实说明失败可能只存在于消费者侧记录。
- 不改判定语义、不改字节码/ABI/语法。原假设的"FN_PROLOG 插入/重排错位"**予以撤销**：导出地址正是合成器留值处，毒化是因为 fn 未编译完成，而非位移算错。

## 目标

1. 拿到毒化导出的 ground truth（entry 地址 + 落点 opcode + code_len）——在 `codegen.rs finish()` 的 poison eprintln 处补诊断。（已交付）
2. 定案根因：区分「编译器位移错误」与「fn 未编译完成 + 判定正确」，并给出可复现证据链。（已交付：应用侧 `let` → `var` 即愈）
3. 交付诊断可见性面：poison 丢弃点可判读、合成失败不再静默；正常路径行为与判定语义零变化。（已交付：auto-lang 3cee0fa10 / auto-ui 66062bb）
4. reader 实证复验：VM 轨 reading 页 Init 正常、恢复行为 16/16；并保留负控（坏文件下诊断面必须发声）。

## 架构方案

- auto-lang 侧：`crates/auto-lang-core/src/vm/codegen.rs::finish()` 的毒化丢弃点——纯 `eprintln!` 载荷扩展（entry/opcode/code_len/±4 窗口），判定条件 `poisoned_fn_exports` 与丢弃行为不变。
- auto-ui 侧：`crates/auto-ui-runtime/src/ui/handler_codegen.rs::record_synth_failure` 增 stderr 泄流；`vm_bridge.rs` 两处构造（`new_with_children` / `new_from_decls`）统一调用 `drain_synth_failures`。
- 不动 `stmt_emit.rs` 的导出写点/平移逻辑（开发期 `AUTO_CODEGEN_TRACE` 探针不入库：一处硬编码 app 的 fn 名，属一次性排障工具，非交付面）。
- 不改字节码指令集、不改 .at 语法、不新增运行期开关。

## 需求分析与背景调查

- 授权：auto-reader READER-001 goal 会话（2026-10-29）路由"在 auto-ui/auto-man 仓按其仓规约修复 N-r8-2"；PLAN-781 先例（reader 前置由框架仓承接）。
- 复现（reader 侧，auto-reader worktree plan-001-dev）：
  - `auto run -r vm --no-merge -B 17825`，日志出现 `[CODEGEN] dropping poisoned export 'handler_reading_page_Init'`；页面挂载后无 Init 内容。
  - 二分（串行实测）：Init 尾段（排版偏好合成段）去掉→干净；尾段四块各自干净、两两干净、三三干净、四块齐全必毒；插入额外 HTTP park 不影响；T4 拆半仍在全量上下文毒。——事后解释：这些"累计型"分界并非插桩位移累积，而是二分到**包含 `pr = null` 的那条路径**才触发编译失败。
  - auto-ui 侧 handler_codegen 预注册 `exports.entry(name).or_insert(0)`；`compile_stmt` 逐语句返回 Ok 的观察只说明**该 fn 的语句各自可编译**，并未覆盖"同一 handler 内 `let` 非法重赋值"这一编译期约束。
- 已知机制：`poisoned_fn_exports`（codegen.rs）= 非特殊命名导出且 `code[addr] != FN_PROLOG`；`finish()` 丢弃此类导出。Fn 正常路径：`stmt_emit.rs` 先 `exports.insert(name, entry_point)`，随后在 entry 插入 FN_PROLOG(+RESERVE_STACK) 并平移 `addr > entry_point` 的导出与 reloc。
- 定案链（本次）：
  1. `let pr` + `pr = null` 在严格 CLI 下无法通过编译 → handler fn 未产出 prolog；
  2. 导出已在语句发射前登记 → 落点停在当前 code 末尾（非 FN_PROLOG）→ `poisoned_fn_exports` 判定为毒 → 丢弃（**正确**，防 Plan 423 P5 的运行时内存炸弹）；
  3. 丢弃消息只说 "see earlier handler errors"，而该路径上编译器没有任何后续错误输出；`record_synth_failure` 又只进未接线的 log::warn → 全线静默；
  4. 改 `var pr`（应用侧合法化）→ Init 合成成功、导出落点 = FN_PROLOG、页面 Init 恢复。

## 详细设计

- T-01 诊断 payload（auto-lang）：`finish()` 丢弃点输出 entry 地址、该字节 opcode、`code_len`、`entry±4` 字节窗口；对 OOB 落点显式标记。
- T-02 根因定案：区分"位移算错"与"fn 未编译完成"。判据：(a) 应用侧把 `let` 改为 `var` 后 poison 消失且落点合规；(b) 失败消息与实际未产出 prolog 一致；(c) 无任何跨 fn 位移异常（同 file 其余 handler 落点正常）。
- T-03 诊断面实现：auto-ui `[SYNTH-FAIL]`（record 时即出 stderr + 桥构造时 drain 两路径）；auto-lang 消息去除误导性提示、补可判读事实。
- T-04（原"最小 .at 复现语料"降格）：缺陷在应用侧，编译器判定正确，**不存在可独立复现的最小 .at 语料**；改为"诊断面负控协议"——用修复前的 `reading_page.at`（`let pr` 形态）在含两面构建上起 VM 轨，断言 stderr 同时出现 `[SYNTH-FAIL]`（含原因）与 `[CODEGEN]` poison 行（含 entry/opcode/窗口）。该负控需要含两面的重建 CLI，**归 merge/landing 门**（review 记 pending）。
- T-05 reader 复验：`268c7d4` 上原生验收驱动 `reader001_r8_native_vm_restore.py` 全过；含两面构建上的复跑归 landing（与 001 最终回归同批）。

## 测试设计

| case | 输入 | 应观察 |
|---|---|---|
| V-886-1 | reader `reading_page.at`（真实触发文件，VM 轨 `-r vm --no-merge`） | 启动日志零 poison 丢弃；`handler_reading_page_Init` 落点 == FN_PROLOG；页面 Init 装载内容 |
| V-886-2 | 负控：修复前形态（`let pr` + `pr = null`）在同一构建上运行 | stderr 出现 `[SYNTH-FAIL] handler synthesis failed: …`（含具体原因）+ `[CODEGEN] dropping poisoned export … entry … opcode … window …`；页面空内容但**有因可查** |
| V-886-3 | 既有 codegen / auto-ui 测试族 | 零回归（裸 `cargo t` 归 merge 档；auto-ui 侧 `cargo check -p auto-ui-runtime` 已过） |
| V-886-4 | reader 原生验收驱动 | 16 项记录全 PASS（挂载/Init/恢复目标可见/pane 深位） |

## 验收标准

- [x] AC-01: reader 实证——`auto run -r vm --no-merge` 启动日志零 poison 丢弃，reading 页 Init 正常装载内容。（证据：问题修复 @268c7d4 后原生驱动 16/16；含两面构建复跑归 landing）
- [ ] AC-02: 诊断面可见性——负控下 `[SYNTH-FAIL]` 与 enrich 后的 `[CODEGEN]` poison 行必须出现在 stderr。**降格记录**：不设最小 `.at` 复现语料/自动化单测（缺陷不在判定面，语料无法独立触发；auto-ui 进程内测试无法捕获自身 stderr，需子进程 harness，列入诊断面测试基建债务）；负控为 merge/landing 门（review 显式 pending）。
- [ ] AC-03: 既有测试零回归——merge 前裸 `cargo t`（merge 档），auto-ui 侧 `cargo check -p auto-ui-runtime` 已 exit 0。

## 执行步骤

- [x] T-01: 诊断 instrumentation（codegen.rs finish poison eprintln 扩展 entry/opcode/len/window）→ 采集 reader repro 数据。 【已完成 2026-10-29】`3cee0fa10`（auto-lang plan-886-dev）；`cargo check -p auto-lang-core` exit 0。
- [x] T-02: 根因定位（依 T-01 数据；产出：一句话结论 + 涉及代码行）。 【已完成 2026-10-29】结论=应用侧 `let pr` 重赋值（严格不可变性强制正确）+ 框架侧静默失败；非 FN_PROLOG 错位。涉及：`crates/auto-lang-core/src/vm/codegen.rs`(finish/poisoned_fn_exports)、`crates/auto-ui-runtime/src/ui/handler_codegen.rs`(record_synth_failure)、`crates/auto-ui-runtime/src/ui/vm_bridge.rs`(两处构造)。
- [x] T-03: 诊断面实现（判定语义不变）。 【已完成 2026-10-29】auto-ui `66062bb`（fix-synth-surface；[SYNTH-FAIL] + 两路径 drain，`cargo check -p auto-ui-runtime` exit 0）；auto-lang `3cee0fa10`（消息去误导 + entry/opcode/window）。开发期 `stmt_emit.rs` 的 `AUTO_CODEGEN_TRACE` 探针未入库（一次性排障工具，含 app 专属硬编码）。
- [x] T-04: 最小 .at 语料 → **降格为诊断面负控协议**（见 AC-02 降格记录），负控执行归 merge/landing。
- [x] T-05: reader 复验。 【已完成 2026-10-29（@117 构建）】`let pr`→`var pr` 后 `reading_page` 在 `-r vm` 下 Init 正常（boot 日志 `handler_reading_page_Init` 正常 park/resume），原生验收驱动 16/16 全过；含两面构建的复跑归 landing（001 最终回归同批）。

## 复审记录

### 2026-10-29 work 收尾（r1，交接独立复审）

- stage: work | plan_id: PLAN-886 | plan_revision: r1 | outcome: execution_done（全部实现任务完成；AC-02 负控为 merge/landing 门，review 显式 pending）
- worktree: auto-lang `D:/autostack/.wt/lang-886/auto-lang`（plan-886-dev，基于 master 0023e21ca）| auto-ui `D:/autostack/.wt/fix-synth-surface/auto-ui`（fix-synth-surface，基于 master d5c7b86 前 3 提交）
- code_commit: auto-lang `3cee0fa10`（[CODEGEN] poison 丢弃点诊断）| auto-ui `66062bb`（[SYNTH-FAIL] + 两构造路径 drain）
- 三处散落诊断面的收拢：auto-ui 侧把 detached 兄弟树（`.wt/lang-886/auto-ui`）里的 `record_synth_failure` stderr 改动**摘出**并落到正式分支（连同 vm_bridge 泄流，补齐第二条构造路径），随后该兄弟树 `git checkout -- .` 清零（校验两处源文件在落分支后内容一致）；auto-lang 侧 `codegen.rs` finish 诊断提交，`stmt_emit.rs` 的 `AUTO_CODEGEN_TRACE` 探针**不予入库**并回退（一次性工具 + app 专属硬编码）。
- verification: `cargo check -p auto-lang-core`（lang-886 worktree）exit 0；`cargo check -p auto-ui-runtime`（与提交内容逐字节一致的树）exit 0，仅仓内既有 warning；裸 `cargo t` 留 merge 档。
- next: review（review_scope=final；独立核对两仓 diff 与「非编译器缺陷」结论、AC-01 证据链，AC-02/AC-03 的 merge-owned 门记 pending）


## 待澄清事项

- 原"根因若落在 handler_codegen（auto-ui）预注册/合成形态"的悬置项：**已定案**——预注册 `or_insert(0)` 未参与本次缺陷（毒化来自 fn 未编译完成），auto-ui 侧以 rider 交付诊断面（`66062bb`），不再需要判定面豁免改动。
- 自动化回归空缺：诊断面目前只有负控协议（人工/驱动），无进程内断言；若后续需要 CI 守护，应建 auto-ui 子进程级 stderr 断言基建（债务，非本计划范围）。
