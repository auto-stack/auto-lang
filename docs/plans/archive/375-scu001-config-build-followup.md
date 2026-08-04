# Plan 375: SCU001 Config-Build 兼容工作收尾

> **本文档是事后补写的收尾计划。** 原始计划只活在已丢失的会话上下文中，
> 从未落盘成文件（不在 worktree、master、tmp 中）。本文件依据已合并的代码改动
> 与 7-21/7-22 会话残留（`d988316f...jsonl`）重建，**只记录剩余工作**，
> 不重建已完成步骤的设计细节——代码本身已是已完成部分的事实来源。

## 背景：编号说明（重要）

**commit message 与代码注释里的 "Plan 364" 指的是本套工作，不是
[docs/plans/364-a2r-cosmic-replication-readiness.md](364-a2r-cosmic-replication-readiness.md)。**

- 本套工作的目标是让集成版 `auto.exe build` 在 **auto/c 项目**（IAR EWARM 嵌入式目标）
  上达到旧独立版 `auto-man.exe v0.1.3` 的功能水平，具体验证项目为汽车座椅控制器
  `D:\SCU001\code\SCU001`。
- "Plan 364" 是当时会话内部给这套工作起的临时编号，写进了 commit message 与
  `opcode.rs` 注释，意外与正式的 364（a2r COSMIC 桌面复制）**撞号**。
- 未来 `grep "Plan 364"` 会同时命中两套不相关的工作，请注意按目录/上下文区分。
- 这套工作**没有正式 plan 文件**，本 375 号是它的首个落盘文档。

## 设计血脉

被删除的 `config_codegen.rs` 最早由 **Plan 075**（2026-02，`df5e0cc06`）引入。
Plan 075 当初的决策是"config 逻辑放编译期 codegen，VM 保持 mode-agnostic"
（见 [docs/plans/archive/075-config-template-modes.md](old/075-config-template-modes.md)）。

但真实 SCU001 的 dep pac.at 里有编译期无法求值的构造，迫使 config 求值下沉到 VM
（见下方 Step 5 动机）——这是顺应"统一所有 VM 解释能力、不为每类 Auto 代码重写一套
解释器"方向的演进，而非推翻 Plan 075。

## 已完成（已合并到 master，commit 见下）

代码已在 master，此处仅列概要，细节以代码为准。

| 步骤 | commit | 内容 |
|---|---|---|
| Step 1-2 | `5d8ed3f7b` | pac.at 支持 `if`/`for`/`var`（AST 预求值 + `CompileDest::Config` + 注入 `port` 上下文） |
| Step 3 | `47d6d192e` | bare-name dep 节点（`dep xmen {}`）+ node body 内 var 替换 |
| Step 4 | `a5b090bfb` | iar/ghs/cmake builder 走 exporter（生成 `.ewp/.eww/.ewd/.ewt`） |
| (续) | `cabee5373` | `pac`/`link` 关键字作为节点名 + `for d in modules` 迭代具名 prop |
| Step 5 | `e0be3e35b` | **config 求值下沉 VM**：新增 5 条累加指令，删除 `config_codegen.rs` |

### Step 5 的动机（为何放弃编译期 flatten）

编译期 flatten（Step 1-2 路线）在真实 SCU001 dep pac.at 上走到尽头：

- **f-string 模板** `` `${kernel.heap}` `` —— flatten 阶段无法求值。
- **对象字段访问** `kernel.heap`（`Expr::Dot`）—— 同上。
- **兄弟 Pair 之间的循环依赖** —— `for d in modules` 中 `modules` 是同级 Pair，
  flatten 阶段需要它的值，但该值要等运行期 collect 才产生（鸡蛋问题）。

Step 5 的解法：新增 `PUSH_ACCUM`/`ACCUM_PAIR`/`ACCUM_NODE`/`ACCUM_MERGE`/`POP_ACCUM`
（opcode `0xD0-0xD4`），让 VM 在运行期自然处理上述构造。8 个单元测试全绿
（`crates/auto-lang/src/vm/config_eval_tests.rs`），含 f-string 与字段访问场景。

## 剩余工作（本计划的核心）

### Task 1：SCU001 端到端构建验证（最关键，阻塞项）

Step 5 只通过了单元测试，**真实 SCU001 项目从未跑通**。7-22 会话末尾实测时，
`auto.exe build` 卡在 **dep 拉取阶段**（Bsp/Mcal 的 git clone / pac.at 解析），
未生成任何项目文件。

- 验证命令：在 `D:\SCU001\code\SCU001` 执行 `auto.exe build`（builder = iar/lanshan）。
- 成功标准：生成 `build/lanshan/project/*.ewp` 等项目文件，与旧 `auto-man.exe v0.1.3`
  产出的项目文件可 diff 对比（7-22 会话曾备份为 `build.bak.plan364`，已清理）。
- 预期会触发 Task 2 / Task 3 的缺口。

### Task 2：`CREATE_NODE` 的 kids 映射（降级，不影响本计划）

> **状态变更（2026-07-30）：降级为独立小任务，移出本计划范围。**

[crates/auto-lang/src/vm/engine.rs](../crates/auto-lang/src/vm/engine.rs) 的
`OpCode::CREATE_NODE` handler 仍有遗留 TODO（`// TODO: Implement kids array/list
mapping`，行号随代码漂移，搜索该字符串定位）。它只提取 props/args，不物化 kids。

**但这不影响 SCU001**：config 求值走的是 Plan 375 的 ACCUM 指令链
（`PUSH_ACCUM`/`ACCUM_PAIR`/`ACCUM_NODE`/`POP_ACCUM`），其中 `ACCUM_NODE`
（[engine.rs](../crates/auto-lang/src/vm/engine.rs) 的 `ACCUM_NODE` handler）
独立处理 kids 追加，完全不经过 `CREATE_NODE`。Task 1 端到端跑通即证明此点。

因此该 TODO 只影响"VM 内用 `CREATE_NODE` 显式构造带子节点的 Node"的场景，
与本计划（SCU001 IAR 输出对齐）无关。**从本计划移除**，留作后续独立小任务
（若将来出现 VM 内构造嵌套 Node 的需求再处理）。

### Task 3：复杂嵌套 dep pac.at 的 f-string / 字段访问覆盖

`config_eval_tests` 覆盖了基本 f-string 与字段访问，但 SCU001 真实 dep pac.at
存在多层嵌套（如 `dep("kernel") { heap: \`${kernel.heap}\` ... }` 套在
`lib("osal"){...}` 内）。需在 Task 1 的实测中确认这些深层场景不报
`Undefined variable: kernel` 之类的错；若有残留，补 VM 路径的符号查找。

## 收尾动作（非阻塞，但建议处理）

- **worktree `scu001-compat-364`**：✅ **已清理（2026-07-30）**。确认其 HEAD
  （`e0be3e35b`，Step 5）的所有 commits 均已合并到 master（`master..HEAD` 为空）、
  工作树干净后，`git worktree unlock` + `git worktree remove` 完成删除，
  已不在 `git worktree list` 中。
  仓库里仍有 23 个历史 stash（多为 plan364 WIP），未删除——内容应在 commits 中，
  留待用户自行酌情清理。
- **commit message 撞号**：历史 commit 不重写（按仓库规范不改写历史），
  本文件的"编号说明"即为缓解措施。

## 计划完成总结

**Plan 375 的核心使命已全部达成（2026-07-30）。** 目标：让集成版
`auto.exe export -p lanshan -f iar` 对 `D:\SCU001\code\SCU001` 产出与旧独立版
`auto-man.exe v0.1.3` 一致的 IAR 工程文件。

最终验证结果：
- `SCU001.eww` / `.ewt` / `.ewd`：与基线**字节级完全一致**。
- `SCU001.ewp`：group / 文件 / include / define 四个集合**完全一致**；
  顶层 group 顺序对齐基线（`Bsp` 在前）。
- 仅剩 device（`Bsp/Mcal`）下子 group 的**内部排列顺序**差异，根因是
  `Target::dirs: HashMap` 无序，属既有设计、IAR 不敏感，不修（见下文"不修"一节）。

本计划期间的提交（按时间序）：
- `6e1f020cb` fix(config): inject Object/Array/Bool dep override args
- `812aa4ea5` fix(config): resolve file() node paths + nested Pair scope leak
- `1bde4a070` fix(target): expose deps as devices/deps/bags array props in to_node
- `55795c585` feat(interpreter): inject globals into AutoVM + Node field access
- `47a741173` fix(pac): assemble apps/deps/devices/libs/bags/tests arrays in to_node
- `eecf0ce7e`（及前序）fix(node/array): plural prop 展开顺序 + Array 消费迭代根因

## 追加：`.ewp` group 顺序根因分析（2026-07-30）

SCU001 四个 IAR 文件中，`.eww/.ewt/.ewd` 已与 auto-man 0.1.3 基线**字节一致**，
`.ewp` 的 group 集合 / 文件集合 / include 集合 / define 集合也全部一致，
仅剩 group/文件的**排列顺序**不同。本轮定位并修复了其中最关键的根因。

### 已修复：`get_kids` plural 展开逆序 + `Array` 消费迭代根因（commit 待提交）

`links: ["a","b","c"]` 这类**复数 prop** 会被 `Node::get_kids("link")` 惰性展开
为一组名为 `link` 的子节点（[crates/auto-val/src/node.rs](../crates/auto-val/src/node.rs)
的 `get_kids`，plural 分支）。app target 的直接依赖顺序、进而整个 `.ewp` group
树的顶层顺序都来自这条路径。

**直接根因**：`get_kids` 的 plural 分支写成 `for kid in simple_kids`（消费
`Array`）。而 `crates/auto-val/src/array.rs` 里 `impl Iterator for Array` 的
实现是

```rust
fn next(&mut self) -> Option<Self::Item> {
    self.values.pop()   // 从尾部弹 → 逆序 + 破坏性 drain
}
```

Rust 的 blanket `impl<I: Iterator> IntoIterator for I` 让 `for x in arr` 复用了
这个 `pop`-based `next`——于是元素被**逆序**追加。诊断证据：

- 原始 prop `links` 数组顺序正确：`[device, osal, xmen, kernel, log, common, EB, lseconfig]`
- `get_kids("link")` 返回却是逆序：`[lseconfig, EB, common, log, kernel, xmen, osal, device]`
- 顶层 group 树因此与基线相反。

**深层根因 + 根治**：`pop` 本身没错（它是栈操作，LIFO + 掏空是栈的正确语义），
错的是把"栈操作"伪装成"遍历操作"——`impl Iterator for Array { next = pop }`
让 `for x in arr` 这个本该正序遍历的语法糖，落到了逆序破坏性的栈弹出上。

按"三套接口各司其职"的原则重构 [array.rs](../crates/auto-val/src/array.rs)：

| 用途 | 接口 | 语义 |
|---|---|---|
| 栈操作 | `Array::push` / `Array::pop` | LIFO，破坏性（保留） |
| 借用遍历 | `Array::iter` / `Array::iter_mut` | 正序，非破坏（保留） |
| 消费遍历 `for x in arr` | `impl IntoIterator for Array` | **正序** drain（新增） |

具体改动：**移除** `impl Iterator for Array { next = pop }`，**新增**
`impl IntoIterator for Array { into_iter → Vec::into_iter }`（正序 move 出每个元素）。
之所以必须移除 `impl Iterator`：core 的 blanket impl 会与手写的 `IntoIterator`
冲突（`E0119 conflicting implementations`），只要 `Array: Iterator` 存在就无法
给 `for x in arr` 换一个正序的 `IntoIterator`。移除后全 workspace 编译通过，
证明没有任何代码把 `Array` 当 `Iterator` trait 对象用——所有消费都是
`for x in arr`，现在统一走正序。

同时 `get_kids` 的 plural 分支改为 `for kid in simple_kids.iter()`（更显式的
正序借用迭代，即使消费语义已修也保留），并加单元测试
`test_get_kids_plural_preserves_order` 锁定声明顺序。

修复后 SCU001 顶层 group 顺序对齐基线（`Bsp` 回到最前），group/file/inc 三个
集合全部一致；全 workspace 编译通过，auto-val（164）/auto-man（176）测试全绿，
auto-lang 的失败均为基线既有（`StringBuilder.push` FFI 缺失、router keyword、
python doctest），与本改动无关。

### 不修：device 子 group 内部顺序

`.ewp` 仍有约 585 行 diff，全部是 device（`Bsp/Mcal`）下子 group 的**内部排列**
（如 `Dio,Can,Pwm,...` vs `Dma,Lin,Gpt,...`）。根因是：

- `Target::dirs` 是 `HashMap<AutoStr, Dir>`（[target.rs](../crates/auto-man/src/target.rs)），
  无序；device 的子目录 group 顺序取决于此 HashMap 的遍历顺序。
- `extract_selects` 用 `HashSet`（仅用于过滤，不决定顺序）。
- 基线自身的子顺序也不规则（既非字母序，也非 `selects` 声明序）。

这是既有设计，与 Plan 375 无关，且 **IAR 对 group 内部顺序不敏感**（只影响
工程树展示）。改 `dirs` 为有序结构（如 `IndexMap`）风险大、收益低，**不在本计划处理**。

