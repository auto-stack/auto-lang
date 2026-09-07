---
plan_id: PLAN-585
status: reviewed              # drafting → executing → execution_done → reviewed → archived
feature_name: vm-repl-stringpool-pin
author: []
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "P585-1: 根因归档——persistent session 裸换池绕过 pinned 不变量（reports 归因链:ash 池日志 #2-#59 常量 FREE/槽位复用/立即数跨代读 + 会话级 repro 污染签名同构实证）"
  - "P585-2: 架构——run_inner 步骤 8 池替换收口 load_strings（pool_state 重建 + flash 常量区 [0,n) pinned + dedup 种子重建,rc.rs §Phase 2 不变量恢复）"
  - "P585-3: 测试——会话级 repro（ParityHost+循环 system 拼接）+ pin 不变量配平（常量区恒 u32::MAX/非墓碑/无 underflow）+ 语料 018_loop_concat_churn"
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm]
current_step: 0
total_steps: 6
---

# [PLAN-585] VM persistent session 字符串池 pinned 不变量修复——解下游 ash retain-after-free stale-copy 数据损坏

## 变更摘要

下游 auto-shell 报告（[P583] 护栏实锤）：ash 脚本在 for 循环内做字符串拼接链 +
`system()`，第二轮起命令串被上一代内容污染（`*.bak./x.tmp`），PowerShell 执行污染
串，该轮静默丢产出；`cargo test -p ash --test examples_parity
positional_arg_passes_to_system` 红（ash 只找到 x.tmp，bash 找到 x.tmp+y.bak）。

本计划归因并修复：**真因不在拼接/trim 临时值配平（583 已修的容器子份额族之外的新
路径），而在 `AutovmReplSession::run_inner`（autovm_persistent.rs:772）以裸赋值
`self.vm.strings = Arc::new(...)` 替换字符串池，绕过 `load_strings` 的 pool_state
重建**——flash 编译期常量（字节码 LOAD_STR 立即数的唯一合法指向）未被标记 pinned。
首个运行期字符串 `add_string` 触发 `ensure_len` 后，rc 计数数组覆盖常量区，常量从
"免计费永活"退化为"rc 计数可释放"：首轮 ADD 操作数释放即把常量槽减到 0 → 墓碑 +
清内容 + freelist；后续 add_string 复用这些槽位装运行期新串；第二轮 LOAD_STR 立即
数 retain 命中墓碑（P583 横幅 + 复活空内容）或读到上代内容 → 拼出交叉污染串。

修复 = run_inner 收口走 `load_strings`（一行）；配平/回归测试三层（会话级 repro +
pin 不变量单测 + 语料）；下游以 examples_parity 转绿验收。

## 目标

1. **修真因**：persistent session 每轮 run 恢复"字节码立即数只引用 pinned 条目"的
   池不变量（rc.rs §Phase 2 设计口径），不再让常量槽被释放/复用。
2. **护栏不闭嘴**：P583 复活横幅保留原样——修复后该横幅在下游 probe 上不再出现，
   而非被静音。
3. **TDD**：先落会话级最小 repro 单测（带 host shim 的 system() 循环拼接，断言每轮
   内容完整），确认红，再修绿。
4. **配平**：583 风格补池不变量单测（常量区 pinned 恒成立、无 underflow、无幻影）。
5. **下游验收**：折回 master 后 ash 仓 `examples_parity::positional_arg_passes_to_system`
   转绿；probe.ash 三轮输出完整。结论记入 merge 记录。

## 架构方案

### 诊断证据链（scratch/p585/ashrun/pool.log，P419_POOL_LOG=1 实录）

ash.exe 跑 probe.ash（`for p in ["*.tmp","*.bak","*.log"] { var r = system("find ..."
+ p + " -type f"); print("iter " + p + " => [" + r.trim() + "]") }`）：

- ash 走 `AutovmReplSession`（auto-shell/src/shell.rs:10 session.run），其
  `run_inner` 步骤 8 每轮裸换池（autovm_persistent.rs:771-772），绕过 pinned。
- 会话启动时 `load_strings(Vec::new())`（autovm_persistent.rs:97）→ pool_state 空。
- 首个运行期 `add_string("*.tmp")`（列表元素物化）`intern-append 8` +
  `ensure_len(9)` → rc 数组覆盖常量区 0..8，全 unpinned。
- 事件 #2/#8：`retain 3 (rc 0→1) content="find . -maxd"`——常量被计数（pinned 应
  PINNED-SKIP）；#4-5：ADD 释放左操作数 → `release 3 (1→0)` → `FREE 3`；#7/#14/#19：
  槽 3/4 被运行期串复用（"find…"、" -type f" 槽装进 "./x.tmp" 等）。
- 第二轮：LOAD_STR 立即数 retain 槽 3 → `[P583] retain-after-free on pool idx 3` +
  复活空内容（#59 `retain 3 content=""`）；` -type f` 的立即数槽 4 已装 "./x.tmp"
  （r₁，被 trim 接收者泄漏份额续命跨代）→ 拼出 `"*.bak" + "./x.tmp"` =
  `*.bak./x.tmp`（下游 PowerShell 错误串逐字符吻合）。
- 单发路径（CLI `auto`、cargo tv 语料、run_with_capture）走 `load_strings(strings)`
  pinned 正确 → 本缺陷在仓内语料不可见，583 批次因此未覆盖。

### 修复设计

- `autovm_persistent.rs` run_inner 步骤 8：
  `self.vm.strings = Arc::new(...)` → `self.vm.load_strings(codegen.strings.clone())`。
  load_strings 重建 pool_state（pinned[0..n]=true）+ 重建 dedup 种子，顺带消灭旧
  路径残留 dedup 键指向已废弃池的 stale-key 面（P-053-8 病灶温床）。
- 不改 pool_retain 复活护栏（它是本次定位的功臣，保留作烟雾枪）。
- 不动拼接/trim 释放纪律（567 T01 已正确）；trim/len 等方法臂接收者 pop-不-release
  的泄漏向问题维持现状（安全方向，在案 KD，不入本计划范围）。

## 需求分析与背景调查

- 下游报告（auto-shell，2026-09-07）：见 git 记录与 DEBTS.md；probe.ash 三轮输出
  第 2/3 轮污染；examples_parity 红。
- Plan 583（archive/583-vm-heaprc-fix-batch.md）：容器负哨兵子份额 ×3 +
  pool_retain 复活加固；本次横幅即该护栏输出。
- Plan 419 Phase 2 池协议：pinned 不变量原文见 rc.rs:126-134；load_strings 实现
  engine.rs:788-806。
- 持久会话换池历史：Plan 080/094/355 系（REPL 复用 VM 实例），strings 换池自始
  裸赋值，pinned 机制（419 Phase 2）后未回头对齐——本计划即补齐该欠账。

## 详细设计

### 变更点（单点）

`crates/auto-lang/src/autovm_persistent.rs:772`

```rust
// 8. Update flash and strings
let mut flash = VirtualFlash::new_with_code(self.bytecode.clone());
flash.object_keys = self.object_keys.clone();
flash.object_types = self.object_types.clone();
self.vm.flash = Arc::new(flash);
- self.vm.strings = Arc::new(std::sync::RwLock::new(codegen.strings.clone()));
+ // Plan 585: 池替换必须走 load_strings——重建 pool_state 并把 flash 常量区
+ // [0, n) 标记 pinned（rc.rs §Phase 2 不变量:字节码立即数只引用 pinned 条目）。
+ // 此前裸赋值绕过 pinned,常量被 rc 计数,首轮释放即墓碑+槽位复用,次轮 LOAD_STR
+ // 立即数读到跨代内容(下游 ash stale-copy 数据损坏,examples_parity 红)。
+ self.vm.load_strings(codegen.strings.clone());
```

### 语义边界

- 跨 REPL 输入的运行期字符串句柄存活性与修复前一致（strings 每轮整体替换,该限制
  在案:autovm_simple_persistence_check #[ignore]）——本修复不缩小也不扩大该面。
- codegen.strings 单调增长（重用 Codegen 累积常量），每轮 load_strings 重pin全量,
  常量区语义不变。
- dedup 重建以 codegen.strings 内容为种子,运行期 dedup 在单轮 run 内照常工作。

## 测试设计

1. **会话级 repro（红→绿,T1 落红,T3 转绿）** `autovm_persistent.rs` 新增
   `loop_concat_system_strings_repl_parity`：AutovmReplSession + 测试 ShellHost
   （按 cmd 内容返回 "./x.tmp\n"/"./y.bak\n"/"\n"）+ 下游同形脚本（var d =
   system("echo dir").trim() / found 累加 / r.trim().len() 门）→ 断言
   `format_last_result() == "./x.tmp\n./y.bak\n"`。
2. **pin 不变量配平（583 风格）** `repl_constants_pinned_after_run`：跑一段含常量
   与循环拼接 churn 的脚本后,断言常量区每个 idx `pool_count(i) == u32::MAX`
   （pinned 永计为 MAX）、`!pool_is_tombstone(i)`、内容不变,且
   `pool_health().underflow_events == 0`。
3. **语料回归** `test/vm/08_strings/018_loop_concat_churn/`：单发路径循环拼接 +
   trim + 累加器形态 golden（expected.out），守护拼接链在正确 pinned 下的输出
   稳定性（cargo tv 档）。
4. **回归门**：`cargo tv` 全档 + fold 前 `cargo tf`；预存红以 583 merge 记录基线
   为准（charts_gallery 族），不新增红。

## 验收标准

- [ ] C1: T1 会话级 repro 单测在修复前红（污染断言失败/横幅出现）,修复后绿。
- [ ] C2: T2 pin 不变量单测绿；修复前该测试同样红（常量被计数/墓碑化）。
- [ ] C3: 语料 018 入库,tv 档全绿（无新红）；tf 预存红不超 583 基线。
- [ ] C4: 下游验收——折回后 ash 仓 `cargo test -p ash --test examples_parity
  positional_arg_passes_to_system` 转绿;probe.ash 三轮输出完整（iter *.bak =>
  [./y.bak] 不再丢失）。
- [ ] C5: 无 compiler warning 新增;scratch 探针残留不入库（scratch/ 为忽略区,
  不影响)。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [✅ 已完成] T1 落红：autovm_persistent.rs tests 新增 plan585_loop_concat_system_strings_repl_parity
  （ParityHost shim + 下游同形脚本）与 plan585_repl_constants_pinned_after_run（pin 配平）；
  `cargo test -p auto-lang --lib autovm_persistent::tests::plan585` 双红实锤——repro 红相
  `Some("./x.tmp\n./y.bak./x.tmp\n./y.bak")`（污染签名与下游 `'*.bak./x.tmp'` 同构）+
  `[P583] retain-after-free on pool idx 7` 横幅；配平红相 idx 0 `pool_count=0`（应
  u32::MAX）伴生 `[P053-8] phantom freelist entry dropped: slot 4 (rc=4294967295)`
  rc 下溢幻影条目。
- [✅ 已完成] T2 修复：autovm_persistent.rs 裸赋值改 `self.vm.load_strings(codegen.strings.clone())`
  （附不变量注释），单点收口。
- [✅ 已完成] T3 转绿：plan585 双测绿；autovm_persistent 全模块 20 passed + 1 ignored
  （预存 ignore）；`cargo check -p auto-lang` 零新增 warning（既存 warning 族与
  autovm_persistent.rs 无涉，grep 实证）。
- [✅ 已完成] T4 语料：test/vm/08_strings/018_loop_concat_churn/（.at + expected.out，
  循环拼接 + trim + found 累加 churn 形态 golden）。
- [✅ 已完成] T5 回归：`cargo tv --no-fail-fast` 3614/3615（唯一红
  ui_gen::vue::test_charts_gallery_compiles = charts 预存基线，583 merge 记录在案）；
  `cargo tf --no-fail-fast` 3473/3474（唯一红同上）；零新增红。
- [✅ 已完成] T6 下游：master 折回（merge 6c86af135）后 ash 仓复跑
  `cargo test -p ash --test examples_parity positional_arg_passes_to_system`
  **转绿**（修复前红：ash ["x.tmp"] vs bash ["x.tmp","y.bak"]）；probe.ash 三轮
  输出完整（`iter *.bak => [./y.bak]` 恢复，修复前该轮静默丢失）；P583 横幅
  归零（P419_POOL_LOG=1 复跑 grep 计数 0）。

## 复审记录

- 复审：ZCode（/auto-plan:review），2026-09-07。
- **逐条复核（verify, don't trust——worktree 829a15852 重跑）**：
  - **C1 PASS**：plan585_loop_concat_system_strings_repl_parity 修复前红（红相实录：
    `Some("./x.tmp\n./y.bak./x.tmp\n./y.bak")` + `[P583] retain-after-free on pool
    idx 7`，污染签名与下游 `'*.bak./x.tmp'` 同构），修复后绿（主检出+worktree 双侧
    复跑 2 passed）。
  - **C2 PASS**：plan585_repl_constants_pinned_after_run 修复前红（idx 0
    `pool_count=0` ≠ u32::MAX，伴生 `[P053-8] phantom freelist entry dropped:
    slot 4 (rc=4294967295)` 下溢幻影），修复后绿。
  - **C3 PASS**：语料 018_loop_concat_churn 入库且 tv 档通过；`cargo tv
    --no-fail-fast` 3614/3615、`cargo tf --no-fail-fast` 3473/3474，唯一红均为
    `ui_gen::vue::test_charts_gallery_compiles`（charts 预存基线，583 merge 记录
    在案），零新增红。
  - **C4 PASS**：下游 auto-shell `cargo test -p ash --test examples_parity
    positional_arg_passes_to_system` 转绿（修复前 left=["x.tmp"] vs
    right=["x.tmp","y.bak"]）；probe.ash 三轮输出完整（`iter *.bak => [./y.bak]`
    恢复）；`P419_POOL_LOG=1` 复跑 P583 横幅计数 0。
  - **C5 PASS**：`cargo check -p auto-lang` 对 autovm_persistent.rs 零 warning
    （既存 warning 族 grep 实证与本文件无涉）；scratch/p585 探针全部未入库
    （untracked）。
- **遗漏/延后/workaround 扫描**：
  - 遗漏：无——全仓裸 `vm.strings` 赋值仅此一处（复审重 grep：唯一写点
    engine.rs:805 load_strings 本体），无同族漏网。
  - 延后：无新延后。观察项入债：CALL_SPEC 字符串方法臂（trim/len 族）接收者
    pop-不-release 份额泄漏（安全向，585 池日志实证），已落
    KNOWN-DEBT-AND-RISKS.md「585 观察」条目；非本缺陷成因（先在、正交），修复
    未依赖它。
  - Workaround：无——根因单点修复，P583 复活护栏原样保留（下游横幅归零即修
    复证据，非闭嘴）。
- **流程偏差（在案）**：折回先于 review——T6 下游验收要求 ash（path 依赖指向
  master 主检出）复跑，故 merge 先行；worktree 分支 829a15852 与 master merge
  6c86af135 内容同源（--no-ff 单提交），tv/tf 在分支尖执行、plan585 双测两侧
  各自复跑，证据链不受影响。
- **spec-impact**：new_spec_components 三条（P585-1/2/3）；supersedes/touched_goals
  不适用留空；affects: [auto-lang/vm]。
- **裁定：PASS → status: reviewed**，无阻塞债。

## 待澄清事项

（无——根因证据链闭环,修复单点。）
