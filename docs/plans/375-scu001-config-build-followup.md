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
（见 [docs/plans/old/075-config-template-modes.md](old/075-config-template-modes.md)）。

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

### Task 2：实现 `extract_node_deep` 的 kids 映射

[crates/auto-lang/src/vm/engine.rs:1879](../crates/auto-lang/src/vm/engine.rs#L1879) 有遗留 TODO：

```
// TODO: Implement kids array/list mapping
```

当前 `extract_node_deep` 只提取 props，**不提取 kids（子节点）/ args**。
影响：dep pac.at 里嵌套 Node 的子节点会在物化时丢失，下游 `merge_port`/
`target.rs` 读不到嵌套结构。这是 SCU001 dep 解析很可能卡住的点。

需补全 kids（及 args）→ Node 结构的映射，并加对应单元测试。

### Task 3：复杂嵌套 dep pac.at 的 f-string / 字段访问覆盖

`config_eval_tests` 覆盖了基本 f-string 与字段访问，但 SCU001 真实 dep pac.at
存在多层嵌套（如 `dep("kernel") { heap: \`${kernel.heap}\` ... }` 套在
`lib("osal"){...}` 内）。需在 Task 1 的实测中确认这些深层场景不报
`Undefined variable: kernel` 之类的错；若有残留，补 VM 路径的符号查找。

## 收尾动作（非阻塞，但建议处理）

- **worktree `scu001-compat-364`**：使命（合并回 master）已完成，当前 `locked`，
  且内含 20+ 个历史 stash（多数与本工作无关）。确认无需保留后可
  `git worktree unlock` + `git worktree remove`，并酌情清理 stash。
- **commit message 撞号**：历史 commit 不重写（按仓库规范不改写历史），
  本文件的"编号说明"即为缓解措施。

## 追加：`.ewp` group 顺序根因分析（2026-07-30）

SCU001 四个 IAR 文件中，`.eww/.ewt/.ewd` 已与 auto-man 0.1.3 基线**字节一致**，
`.ewp` 的 group 集合 / 文件集合 / include 集合 / define 集合也全部一致，
仅剩 group/文件的**排列顺序**不同。本轮定位并修复了其中最关键的根因。

### 已修复：`get_kids` plural 展开逆序（commit 待提交）

`links: ["a","b","c"]` 这类**复数 prop** 会被 `Node::get_kids("link")` 惰性展开
为一组名为 `link` 的子节点（[crates/auto-val/src/node.rs](../crates/auto-val/src/node.rs)
的 `get_kids`，plural 分支）。app target 的直接依赖顺序、进而整个 `.ewp` group
树的顶层顺序都来自这条路径。

**根因**：`crates/auto-val/src/array.rs` 里 `impl Iterator for Array` 的实现是

```rust
fn next(&mut self) -> Option<Self::Item> {
    self.values.pop()   // 从尾部弹 → 逆序 + 破坏性 drain
}
```

而 `get_kids` 的 plural 分支写成 `for kid in simple_kids`（消费 `Array`），
走的就是这个 `Iterator`——于是元素被**逆序**追加到结果。诊断证据：

- 原始 prop `links` 数组顺序正确：`[device, osal, xmen, kernel, log, common, EB, lseconfig]`
- `get_kids("link")` 返回却是逆序：`[lseconfig, EB, common, log, kernel, xmen, osal, device]`
- 顶层 group 树因此与基线相反。

**修复**：plural 分支改为按引用迭代 `for kid in simple_kids.iter()`（走
[array.rs](../crates/auto-val/src/array.rs) 的 `iter()`，正序、非破坏），
并加单元测试 `test_get_kids_plural_preserves_order` 锁定声明顺序。
修复后顶层 group 顺序对齐基线（`Bsp` 回到最前），group/file/inc 三个集合全部一致。

### 已知隐患（未修，记录在此）：`Array` 的破坏性 `Iterator`

`impl Iterator for Array { fn next → Vec::pop }` 本身是个反模式陷阱：

1. **逆序**：`next()` 从尾部弹，违反 `Iterator` 的"从头到尾"直觉。
2. **破坏性**：迭代会掏空数组（`for x in arr` 之后 `arr` 为空）。
3. 任何 `for x in <Array 变量>`（消费，非 `.iter()`）的写法都会踩中。

本计划只修了 `get_kids` 一处（SCU001 可观测的 group 顺序）。全量排查/根治
（改 `Array::next` 为正序 drain，或移除 `impl Iterator` 改用 `IntoIterator`）
影响面广，需单独评估所有消费点——**不在本计划范围内**，留作后续。

### 不修：device 子 group 内部顺序

`.ewp` 仍有约 585 行 diff，全部是 device（`Bsp/Mcal`）下子 group 的**内部排列**
（如 `Dio,Can,Pwm,...` vs `Dma,Lin,Gpt,...`）。根因是：

- `Target::dirs` 是 `HashMap<AutoStr, Dir>`（[target.rs](../crates/auto-man/src/target.rs)），
  无序；device 的子目录 group 顺序取决于此 HashMap 的遍历顺序。
- `extract_selects` 用 `HashSet`（仅用于过滤，不决定顺序）。
- 基线自身的子顺序也不规则（既非字母序，也非 `selects` 声明序）。

这是既有设计，与 Plan 375 无关，且 **IAR 对 group 内部顺序不敏感**（只影响
工程树展示）。改 `dirs` 为有序结构（如 `IndexMap`）风险大、收益低，**不在本计划处理**。

