---
plan_id: PLAN-747
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: fix-ui-focus-refcell-storm
author: [zhaopuming]
created_at: 2026-10-09
updated_at: 2026-10-09

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 7
total_steps: 7
---

# [PLAN-747] fix-ui-focus-refcell-storm：ui.focus RefCell 借用冲突风暴（UI 挂起 + abort 崩溃根修）

## 变更摘要

修复 jade-edit 上游报告的「原生窗 UI 挂起（Event 1002）+ abort 崩溃（WER c0000409/FastFail 0x7）」双签名问题的两个根因：

1. **挂起根因**：PLAN-095 T-04 引入的 `focus_pending` 消费块存在 edition-2021 if-let 守卫跨块借用冲突
   （`renderer.rs` `if let Some(..) = state.app.focus_pending.borrow().clone()` 的 Ref 临时守卫活整个块，
   块内三处 `borrow_mut()` 必 panic "RefCell already borrowed"）。panic 后 pending 永远清不掉 →
   每条 update 消息重入 panic → 被 catch_unwind 边界吞掉但 update 后半段（resize/scroll/任务返回）全丢
   → UI 冻结。退出审计日志实证：2026-10-09 当日 4611 条同位 panic（两个实例 ~1.3-2s 一条节奏）。
2. **abort 放大器**：panic 落审计后默认 hook 向 stderr 打印；stderr 管道已关（os error 232，驱动方断开）
   或系统资源枯竭（os error 1450，多实例并发）时打印自身 panic → panic-in-panic → abort（fastfail 7）
   → 进程以 0xCFFFFFFF 消失 = WER c0000409 签名、探针 stderr 空尾。审计实证：133 例打印失败 panic。

## 目标

- G1：任意 app 调用 `ui.focus()` 后，update 循环零 panic（审计日志零新增 "RefCell already borrowed"），
  focus 任务真实下发（此前 tail_tasks 随 unwind 丢弃，`ui.focus` 从未真正聚焦过——PLAN-095 自测
  过关仅因 `__focus_result` 写入发生在 panic 点之前）。
- G2：桌面进程在 stderr 管道关闭/资源紧张下遭遇**可捕获 panic** 时不再升级为 abort（容错打印）；
  开发态（RUST_BACKTRACE 置位）保留原生 hook 的 backtrace 能力。
- G3：`crates/auto-lang/src/ui/` 全树扫排同型 `if let … .borrow()…` + 块内同 RefCell 再借用隐患，
  机械同型者一并修复。
- 非目标：RC canary UAF / virt_memory 越界 / wgpu OOM 等历史 panic 家族（各有独立归因，不在本计划）；
  心跳 2s 重建节拍的 CPU 面优化；09-29 及更早 WER 崩溃（无审计 panic 行的静默族，疑栈溢出，
  本计划只消除已定谳的 abort 放大链）。

## 架构方案

纯 renderer/stdlib 层修复，无协议/生成器/VM 语义变化：

- **根修（renderer.rs）**：把 `focus_pending.borrow().clone()` 提为独立 `let` 语句（守卫随语句结束释放），
  if-let 判别式消费克隆值；同时把三臂决策抽为纯函数 `focus_pending_step()` 供单测。
- **容错打印（vm/ffi/stdlib.rs）**：`install_exit_audit_panic_hook` 在 RUST_BACKTRACE 未置位时改走
  `print_panic_tolerant()`——自组默认格式文本、`Write::write_all` 结果吞掉（io::Error 返回而非 panic），
  不再调用原生 prev hook 的打印路径；RUST_BACKTRACE 置位时保留 `prev(info)`（开发态 backtrace）。
- **扫排（ui/ 全树）**：`grep` 模式 `if let.*\.borrow\(\)` 判别式 + 块内同字段 `borrow_mut` —— 仅修
  机械同型（同 RefCell 判别式临时守卫 + 块内再借用）者，语义等价最小 diff。

## 需求分析与背景调查

- 上游报告：`D:\autostack\jade-edit\docs\analysis\2026-10-09-auto-lang-ui-hang-report.md`
  （jade PLAN-039 发布收口转来；现象=原生窗挂起 + `app exited (code 3489660927)`；WER 六份
  AppCrash c0000409/0x7 自 09-12 周级存在；jade 侧已 git stash 基线对照排除）。
- 本仓实证（2026-10-09 调查）：
  - 退出审计 `%LOCALAPPDATA%/auto-desktop/exit-audit.log`（PLAN-575 D1 落盘，默认路径无需 env）：
    - `RefCell already borrowed @renderer.rs:20538`（载体 2730 行号；当前 HEAD=20784-20801）今日 4611 条，
      10-01（lang-717 worktree 载体，行号 19994-20033 族）81 条——首次出现恰为 PLAN-095 T-04 引入窗口
      （commit 5271523c8，2026-10-01 14:52 +0800）。
    - 今日挂起窗实录：pid 22924 末条 panic 13:21:44 → 14s 后 13:21:58 Event 1002；风暴实例无
      main_err/main_return 收尾行（死在 panic 中途 = abort 链外观）。
    - `failed printing to stderr`（os 232）/`stdout`（os 232）/os 1450 共 133 例 = abort 放大器实证。
  - 触发器：jade `src/front/app.at` 调 `ui.focus(".QuickEdit")`（每次 open-file/quick-open 均触发）。
  - jade 复现配方：`node e2e/.runtime/t01_empty_body.mjs` / `node tests/probe_native_wiki_link.mjs --phase all`。
- 行为学解释力核对：stderr 空尾（打印失败即死因）、崩溃点逐轮不同（取决于哪条 panic 撞上死管道）、
  debug/release 双载体一致（逻辑 bug 与 std 行为，与优化级无关）、挂起与崩溃并存（同一风暴的两个出口）
  ——全部闭环。
- specs 面：ui 模块现状无对应「focus 消费时序」条目需改写；新增回归面挂 `ui`（affects）。

## 详细设计

### D1 根修（crates/auto-lang/src/ui/iced/renderer.rs）

现状（HEAD 20781-20802，载体 20527-20549）：

```rust
if let Some(key) = crate::vm::native::take_ui_focus_request() {
    *state.app.focus_pending.borrow_mut() = Some((key, 0));
}
if let Some((key, tries)) = state.app.focus_pending.borrow().clone() {   // ← Ref 守卫活整块（edition 2021）
    ...
    *state.app.focus_pending.borrow_mut() = None;      // 三臂均必 panic
```

修复后：

```rust
if let Some(key) = crate::vm::native::take_ui_focus_request() {
    *state.app.focus_pending.borrow_mut() = Some((key, 0));
}
// PLAN-747 根修：判别式克隆提为独立语句——edition 2021 下 if-let 判别式临时
// （Ref 守卫）活整个块，块内 borrow_mut 必 panic 且 pending 永不清除（每条
// update 消息重入 panic，update 后半段全丢 = UI 冻结；2026-10-09 审计 4611 例）。
let focus_pending = state.app.focus_pending.borrow().clone();
if let Some((key, tries)) = focus_pending {
    let (view, _, _) = state.component.view_with_debug_gated(false);
    let converted = convert_view_messages(view);
    let mut inputs: Vec<FocusableInput> = Vec::new();
    collect_focusable_inputs(&converted, &mut inputs);
    let (focus_id, done, result) = focus_pending_step(&inputs, &key, tries);
    if let Some(id) = focus_id {
        tail_tasks.push(iced::widget::operation::focus(id));
    }
    if done {
        let _ = state.component.write_state("__focus_result", auto_val::Value::str(&result));
        *state.app.focus_pending.borrow_mut() = None;
    } else {
        *state.app.focus_pending.borrow_mut() = Some((key, tries + 1));
    }
}
```

纯函数（同文件，邻 `resolve_focus_target` 放置）：

```rust
/// PLAN-095 T-04 消费步决策（PLAN-747 抽出独立可测）：
/// 返回 (focus 任务目标, 是否终结 pending, __focus_result 值)。
/// 终结=false 表示挂载竞态窗口重试（tries+1 由调用方落）。
pub(super) fn focus_pending_step(
    inputs: &[FocusableInput],
    key: &str,
    tries: u32,
) -> (Option<iced::widget::Id>, bool, String) {
    if let Some(id) = resolve_focus_target(inputs, key) {
        (Some(id), true, "ok".to_string())
    } else if !inputs.is_empty() || tries >= 5 {
        (None, true, format!("miss:{key}"))
    } else {
        (None, false, String::new())
    }
}
```

### D2 容错打印（crates/auto-lang/src/vm/ffi/stdlib.rs）

`install_exit_audit_panic_hook` 尾部 `prev(info)` 替换：

```rust
// PLAN-747：RUST_BACKTRACE 未置位（自动化/常态）走容错打印——原生 hook 的
// 打印走 stderr，管道关闭(os 232)/资源枯竭(os 1450)时 _eprint 自身 panic →
// panic-in-panic → abort(fastfail 7)，桌面实例以 0xCFFFFFFF 消失（2026-10-09
// 审计 133 例实证）。write_all 失败仅吞 io::Error，绝不升级为 abort。
// 开发态（RUST_BACKTRACE=1）保留原生 prev（含 backtrace）。
if backtrace_requested() { prev(info); } else { print_panic_tolerant(info); }
```

`print_panic_tolerant` 自组默认格式（thread 名 + location + payload + RUST_BACKTRACE 提示行），
`stderr().lock()` + `write_all` + `flush`，全部结果 `let _ =` 吞掉。payload 提取复用既有
downcast 逻辑（抽 `panic_payload_str(info)` 小助手，hook 与打印共用）。

注：本 install 位（renderer 启动）之前无其他 hook 注册者（examples/ui_desktop.rs 的 hook 是
独立 install 位，不经本函数），prev == 原生默认 hook，替换其打印路径无行为丢失；
RUST_BACKTRACE 分支保底开发态栈回溯。

### D3 同型扫排（crates/auto-lang/src/ui/ 全树）

机械模式：`if let`/`while let` 判别式含 `X.borrow()`/`X.borrow_mut()`（临时守卫跨块）且块内
对**同一 RefCell** 再次 borrow/borrow_mut。已知 `pending_window_resize.borrow_mut().take()` 判别式
块内无同字段再借用 → 安全，不动。扫排结果进执行回执（无论修否逐条记录判定）。

## 测试设计

- **单测（renderer.rs 内联 tests 模块，与 resolve_focus_target 既有测试同区）**：
  - `focus_pending_step` 三臂：命中（ok+终结+Some(id)）/稳定 miss（inputs 非空或 tries≥5 → miss+终结）/
    竞态重试（inputs 空 && tries<5 → 不终结）。
  - 借用纪律回归：构造 `RefCell<Option<(String,u32)>>` 同形态消费（克隆先行）断言不 panic——
    钉住「判别式守卫跨块」类回归的最小形态。
- **stdlib 单测**：`print_panic_tolerant` 格式（thread/loc/msg 三要素）；`backtrace_requested()`
  env 三态（未设/0/1）。打印吞错路径以写入已关管道不可移植，仅以格式单测+类型级（返回 ()）钉面。
- **e2e（验证阶段，实机）**：
  - 复现脚本基线：jade `t01_empty_body.mjs`（`AUTO_EXE` 指向修复构建）跑通且
    `exit-audit.log` 增量零 "RefCell already borrowed"；
  - `probe_native_wiki_link.mjs --phase all` 双臂通过（修复前该探针 mid-run 实例消失）。
- **门禁**：Category B —— worktree 内 `cargo check -p auto-lang`、裸 `cargo t`（日常档）。
  改动面不触 vm 编译器语料/trans/book/ui_gen/aavm → 无 tv/tt/tb/tu/taa 追加档。

## 验收标准

- [AC-1] 修复构建下运行 jade t01/probe_native_wiki_link，`exit-audit.log` 会话增量中
  `site=panic msg=RefCell already borrowed` 计数为 0（修复前：单实例 60s 内数百条）。
- [AC-2] `ui.focus` 行为修复可观察：t01/probe 流程中 `__focus_result` 达 `ok`（或稳定 `miss:*`）
  且不再重入；（顺带）focus 任务真实下发（PLAN-095 原意恢复）。
- [AC-3] 容错打印单测通过；RUST_BACKTRACE=1 时仍走原生 hook（单测钉 env 分派）。
- [AC-4] 同型扫排完成且回执逐条记录（修/不修+理由）。
- [AC-5] `cargo check -p auto-lang` 零告警新增；裸 `cargo t` 无新增红（预存红族以 §1 非目标口径豁免）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [✅ 已完成] T-01 骨架/master 提交（139486040）+ worktree `D:/autostack/.wt/lang-747/auto-lang`
  （plan-747-dev）+ auto-down 兄弟 detached（895f8d0）——跨仓依赖解析就绪。
- [✅ 已完成] T-02 renderer.rs D1 根修（克隆提为独立 `let` + `focus_pending_step` 抽取，
  worktree renderer.rs 20781-20811/3452-3466）+ 内联单测三臂 + 借用纪律回归测（4/4 PASS，
  nextest `cargo t plan747`）。
- [✅ 已完成] T-03 stdlib.rs D2 容错打印（`install_exit_audit_panic_hook` RUST_BACKTRACE
  分派 + `print_panic_tolerant`/`format_panic_report`/`backtrace_requested`/`panic_payload_str`，
  worktree stdlib.rs 962-1035）+ 单测 2 项（格式形态/env 三态）。
- [✅ 已完成] T-04 ui/ 同型扫排：脚本判据 = if/while-let 判别式含 `.borrow()/.borrow_mut()`
  且块内同字段表达式再借用 → **全 ui/ 树（renderer.rs 含）零命中**（focus_pending 修复后）。
  判定记录：`pending_window_resize.borrow_mut().take()`（renderer.rs:20863）块内无同字段
  再借用=安全不动；`source_code.borrow()`/`marquee.borrow_mut()` 等只读/取出型判别式同判。
  已知边界：仅机械同名表达式判据，别名借用（同 RefCell 异表达式）不在机械扫排面。
- [✅ 已完成] T-05a `cargo check -p auto-lang` 通过（1m20s；400 预存告警，新函数零新增）。
- [✅ 已完成] T-05b e2e 实机（修复构建 `D:/autostack/.wt/lang-747/auto-lang/target/debug/auto.exe`，
  2026-10-09 15:10）：
  - jade `t01_empty_body.mjs`：全程通过（EXIT=0；空档/清空重输/模板档追加/Save 面盘点）。
  - jade `probe_native_wiki_link.mjs --phase all`：**PASS=31 FAIL=0 BLOCKED=3**（BLOCKED 为
    仪器边界既档分列，非失败；修复前同探针 mid-run `app exited (code 3489660927)`）。
  - 审计增量：两轮会话（含多实例+长驱）`exit-audit.log` 行数 **7714→7714 零增量**
    （修复前同形态单实例 60s 内数百条 RefCell 风暴）。
- [✅ 已完成] T-05c 裸 `cargo t` 日常档：worktree 与 master 主检出两侧全量
  `--no-fail-fast` 对照——失败集合 30 vs 30，唯一差异为一对图表面测试换位
  （master 红 plan502_m3 / worktree 红 plan484_024），**交叉验证两侧均过**
  （负载 flake 族）；其余 29 项两侧一致（含 musk_vm_track_p053 预存红族——
  master 主检出复现同红，与本 diff 无关；state_file::lock_serializes 单独跑过
  =负载时序 flake）。结论：**本 diff 零新增红**。
- [✅ 已完成] T-06 独立复审（2026-10-09，见「复审记录」）。
- [ ] T-07 merge 回 master（Conventional Commit `fix(ui): … (Plan 747)`）、归档、回复 jade-edit 侧。

## 复审记录

**2026-10-09 独立复审（verify, don't trust）——结论：pass**

- **AC 核证**：
  - AC-1 ✅ `exit-audit.log` 两轮 e2e 会话（t01 + probe_native_wiki_link
    --phase all，多实例长驱）行数 7714→7714 零增量；对照：修复前同形态会话
    单实例 60s 内数百条 RefCell 风暴（13:0x-13:2x 簇 pid 36504/22924、
    14:2x 簇 pid 42676/27876）。
  - AC-2 ✅（行为恢复面）t01 四阶段全过（开档/清空重输/模板追加/Save 面
    盘点——press→snapshot→type_text 交互环全走通；修复前同族会话即风暴死）；
    probe_native_wiki_link --phase all PASS=31/FAIL=0（修复前 mid-run
    `app exited (code 3489660927)`）。`__focus_result` 未在 e2e 直接断言
    （jade 流程不回读该字段），由「零 panic + 交互环恢复 + 单测三臂」组合
    覆盖；诚实记档于此。
  - AC-3 ✅ 4/4 新测绿（renderer 三臂/借用纪律 + stdlib 格式/env 三态）。
  - AC-4 ✅ 扫排零额外命中，判定回执在执行步骤 T-04。
  - AC-5 ✅ cargo check 新函数零新增告警；fmt 改动区干净（renderer.rs/
    stdlib.rs 的 rustfmt 差异均落预存区段，行号核验 3300-3600/20750-20850/
    960-1040 无命中）；裸 cargo t 零新增红（T-05c）。
- **遗漏/延后/Workaround 扫描**：
  - 无 workaround 补丁；容错打印为设计性加固，已知取舍记档（RUST_BACKTRACE
    未置位时丢失 backtrace；ui_desktop example 的 force-backtrace hook 在
    同管线内被取代——开发态设 RUST_BACKTRACE=1 即保留）。
  - 债候选登记 KNOWN-DEBT-AND-RISKS.md：09-29 及更早「无审计 panic 行」的
    静默崩溃族（疑栈溢出路径，本计划只消除已定谳的 abort 放大链）；RC
    canary UAF / virt_memory 越界 / wgpu OOM 等独立 panic 家族原状在档。
- **Spec 影响**：bugfix 无 spec 改写；`affects: [auto-lang/ui]`；
  supersedes/new_components 空，touched_goals 空。

### 规范增量（2026-10-09 复审补记，legacy 计划按已验证实现补档）

无 canonical spec 改写（retire/modify/add 全空）——本计划为行为缺陷根修：
`ui.focus` 消费块的运行时行为恢复到 PLAN-095 T-04 的**既定意图**（快照/聚焦
语义零变化）；panic hook 沿用 PLAN-575 D1「只追加日志」契约，容错打印仅改变
打印失败时的进程存活语义。`supersedes_spec_components`/`new_spec_components`/
`touched_goals` 留空即为本节书面解释；受影响面 `affects: [auto-lang/ui]`
（运行时缺陷归属，非规约条目变更）。规约冻结验证：`git diff 7656885a3
95e7169be -- docs/specs/` 为空（零 spec 文件触及）。

---

**2026-10-09 /auto-plan:review 正式复审（用户显式要求补记；archived 终态不翻转）**

`stage: review | plan_id: PLAN-747 | plan_revision: 1（无显式修订号，按初版归一化） | outcome: pass | reviewed_commit: 95e7169be（merge，parents 189eec0c6+7656885a3；实现提交 7656885a3 ∈ HEAD 祖先已验） | base_commit: 139486040（骨架提交，diff base） | dependency_revisions: auto-down 895f8d0（组内兄弟，未触及） | spec_inputs: docs/specs/ 零输入（空 delta，见规范增量节） | acceptance_results: AC-1 pass / AC-2 pass（局限记档） / AC-3 pass / AC-4 pass / AC-5 pass | findings: F-1 流程性（非缺陷）——本次复审在归档后由实施会话补跑，独立性受限，已按 skill 规定从制品重建结论而非采信执行摘要；F-2 非阻塞——AC-2 的 __focus_result 未在 e2e 直接断言（组合覆盖，T-06 已诚实记档）；F-3 信息项——复核时点审计日志 +1 行为 code=0 site=main_return 干净退出（15:20:16，e2e 尾期实例正常收尾），非 panic，AC-1 口径不受影响 | evidence: ① 受测提交 7656885a3 与 HEAD 两实现文件逐字节一致（git diff --stat 为空）→ worktree 内全部验证按 skill"unchanged code 复用+显式理由"承接到 HEAD；② 审计日志（%LOCALAPPDATA%/auto-desktop/exit-audit.log，运行时落盘的独立制品）10-09 当日 4611 例风暴 vs 会话窗零 panic 增量（7714 基线后唯一新增为干净退出行）；③ HEAD 源位核证：focus_pending_step@renderer.rs:3484、print_panic_tolerant/backtrace_requested@stdlib.rs:1002/1012、4 项回归测在案；④ 裸 cargo t 对照（worktree vs master 各 30 预存红、图表面换位 flake 双向交叉验证均过）记于 T-05c；⑤ 脏树清点：8 文件未提交改动全属并发 media 会话（music-player/mpv 面），与 PLAN-747 实现零重叠——本计划实现全部已提交，pass 绑定 HEAD 成立；脏改动归属路由至该会话处置，不入本计划 | next: 无需返工；维持 archived 终态；观察项 P747-D1（静默崩溃族）在 KNOWN-DEBT-AND-RISKS.md 随访`

**独立性声明（skill 条款）**：本复审在实施会话内执行，不声称角色独立；
结论从可复核制品（git 祖先/字节一致性、运行时审计日志、源位核验、
既有门禁回执）重建，未复跑需重建二进制的检查（理由：受测提交与 HEAD
字节一致 + 主检出当前被并发会话 WIP 占用，重建必引入跨会话污染）。

## 待澄清事项

（无——上游报告即完整授权：jade 侧「修复落地后告诉我一声，我会回注 PLAN-039 并补跑三件
失效的 probe receipt、闭合 readiness §8 行动项」。）
