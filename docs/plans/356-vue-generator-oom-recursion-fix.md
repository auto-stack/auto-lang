# Plan 356: Vue 生成器 OOM / 递归爆炸修复

> **类型**: Bug 修复 + 架构改进
> **状态**: 已实施（真实根因见下方「实施结果」）
> **日期**: 2026-07-16
> **前置**: 015-notes 升级（Plan 354）过程中发现
> **影响**: Vue 生成器在特定 view 树结构下消耗 1.7GB+ 内存导致 `auto gen` / `auto run` 卡死

---

## 0. 实施结果（2026-07-16，worktree `plan-356-vue-oom`）

**原计划的前提（§1.3「不是 parser 的 `style: if` 解析问题」）在当前代码上是错的。**
通过系统化诊断（最小复现 + 二分 + 插桩），真正的根因不在 Vue 生成器，而在 **解析器**：

- **症状**：`for tag in .items { button { onclick: .SelectTag(tag) } }` 这类
  「循环体内的事件处理器以循环变量为参数」的结构会让 `auto gen` 内存爆炸到 11GB+。
- **根因**：标识符 `tag` 被词法分析器识别为保留关键字 `TokenKind::Tag`
  （见 `token.rs:368`）。而 `parse_event_arg`（`parser.rs`）的参数循环只匹配
  `TokenKind::Ident`，对 `Tag` 这类 token **没有任何分支处理** → 直接 `break`，
  返回空字符串却**不消费该 token** → 调用方的 `while !RParen` 参数循环永远停在同一个
  `tag` token 上 → 无限循环 → OOM。
- **二分矩阵**（均为最小复现，见 `crates/auto-lang/tests/fixtures/plan356_v*.at`）：
  - `for` 循环 + `onclick: .H(loopvar)` → ❌ OOM（`loopvar` 是 `tag` 等保留字标识符）
  - 去掉循环（handler 不在 for 内） → ✅ 正常
  - handler 不带参数 → ✅ 正常
  - 把循环变量改名（非保留字） → ✅ 正常
  - `style: if/else`、`msg`、`on` 均非触发条件（与原计划 §1.2 矩阵不同）。

**修复**（`crates/auto-lang/src/parser.rs`）：
1. 新增 `cur_is_soft_ident()`，在参数位置把 `Tag`/`Type`/`Union`/`Spec`/`Super`/
   `Has`/`Copy`/`Move`/`Take`/`Hold`/`Alias`/`Ext`/`Impl`/`Mod`/`Enum` 等软关键字当作
   普通标识符消费（Plan 356 主修复）。
2. `parse_event_arg` / 新增 `parse_event_arg_list` 加防失控上限，任何未来「无分支消费」
   的 token 都会产出清晰错误，而不是 OOM（防御性）。

**遗留 1（已修，2026-07-16，worktree `plan-356-for-identfield`）**：原 200 行
sidebar.at（commit `50307d51`）除上述 OOM 外，还有一个**独立的解析错误**（OOM 修复后
才暴露）：`for tag in note.tags` 的 iterable `note.tags` 是 `ident.field` 链，而
`parse_view_for_loop` 只接受 `.field` / 数字范围 / 单 ident，不支持 `ident.field`，
于是 `note` 被消费、`.tags` 残留 → 后续 `Expected term, got RBrace`。
**修复**：`parse_view_for_loop` 的单 ident 分支对称地消费后续 `.field` 链
（`crates/auto-lang/src/parser.rs`，Plan 356 §3.2 列出的改进点）。回归测试：
`test_view_for_loop_ident_field_iterable` / `..._chain_iterable`。

**遗留 2（未修，独立 bug）**：完整 sidebar 仍无法端到端生成。`for ident.field` 修复后
解析推进到 offset ~8873，又碰到**第三个独立解析错误**：作为 view 属性的
`style: if <复杂条件> { } else { }`（条件含 `==`，如
`style: if .active_tag == tag { ... } else { ... }`）无法解析。这是 view-prop 位置
`if` 表达式的解析问题，与 Plan 356 OOM 无关，需单独处理。
`test_plan356_real_sidebar_generates`（`#[ignore]`）留作该 bug 修复后的回归守卫。

---

## 1. 问题描述

### 1.1 症状

`auto gen` / `auto run` 在处理 015-notes 的 `sidebar.at`（~198 行，含文件夹分组的笔记树 + 标签筛选）时，内存暴涨到 1.7GB+ 后卡死无输出。

### 1.2 精确触发条件（已通过二分法确认）

**最小复现**：以下结构的 widget 会导致 OOM：

```auto
widget NavTree {
    msg Msg { SelectTag(str) }          // ← 必须有 msg 声明
    view {
        col {
            for tag in .all_tags {       // ← for 循环
                button {
                    text tag
                    onclick: .SelectTag(tag)  // ← onclick handler
                    style: if tag == "a" {     // ← 条件 style (if/else)
                        "bg-blue-500 text-white"
                    } else {
                        "bg-gray-100"
                    }
                }
            }
        }
    }
    on {                                 // ← 必须有 on 块
        .SelectTag(t) -> { }
    }
}
```

**触发条件矩阵**（通过逐项增减确认）：

| 组合 | 结果 |
|---|---|
| for + style:if + **有 msg/on** | ❌ **OOM** |
| for + style:if + **无 msg/on** | ✅ 正常 |
| for + 无 style:if + 有 msg/on | ✅ 正常 |
| 无 for + style:if + 有 msg/on | ✅ 正常 |
| for + style:if(无 else) + 有 msg/on | 未测（需确认） |

**关键发现**：三个条件必须同时满足才会 OOM：
1. `for` 循环（view 层迭代）
2. 循环体内的节点有 `style: if ... else ...`（条件样式）
3. widget 有 `msg` + `on` 块（handler 定义）

### 1.3 不是什么

- **不是** 生成器的 `node_to_html` 递归问题——该函数是线性的 O(N)，每个节点只访问一次
- **不是** `String` vs `Write` 的问题——String 累积是 O(N)，不会产生 GB 级数据
- **不是** AuraNode 树结构问题——提取阶段（extract_view_node）也是线性的
- **不是** 单纯的 `autodown_editor` 问题——editor.at（含 autodown_editor）单独编译通过
- **不是** parser 的 `style: if` 解析问题——无 msg/on 时同样的 style:if 正常解析

### 1.4 最可能的根因

三个条件的交叉点在 **`generate_script`（handler 生成）与 `node_to_html`（模板生成）的交互**。具体推测：

- `generate_script` 在扫描 view 树寻找 handler 绑定（onclick 等）时，遇到 `for` 循环 + 条件 style 的组合，可能对每个迭代变体展开 handler 代码
- 或者 `extract_classes` 处理 `style: Expr::If` 时，与 handler 扫描产生某种笛卡尔积
- 或者 shadcn 模式下 `generate_shadcn_attrs` 对带 handler 的 button 做了额外的属性展开，与条件 style 叠加

**需要在实施阶段通过插桩确认精确位置。**

---

## 2. 修复策略

### 策略 A：精确修复（推荐，最小改动）

**目标**：找到 OOM 的精确代码行，修复算法 bug。

**步骤**：
1. 在 `node_to_html` 和 `generate_script` 入口加内存/节点计数日志
2. 用最小复现 case 跑，看哪个函数的调用次数或内存增长异常
3. 修复具体的算法问题（如限制展开次数、避免笛卡尔积、提前终止等）

**风险**：低。修复的是具体 bug，不改变架构。

### 策略 B：防御性改造（中等改动）

**目标**：让生成器对任意 view 树都有可预测的内存使用。

**B.1 `node_to_html` 改为 `impl Write` 而非返回 `String`**

当前 `node_to_html` 返回 `GenResult<String>`，每次递归创建新 String。改为写入 `&mut impl Write`（像 `ts_adapter.rs` 那样），避免中间 String 分配。

但这不是 OOM 的根因（String 是 O(N)）——是防御性优化。

**B.2 限制 view 树深度/节点数**

在 `extract_view_node` 或 `node_to_html` 入口加深度/节点计数检查，超过阈值时报错而非 OOM：
```rust
const MAX_VIEW_NODES: usize = 1000;
if node_count > MAX_VIEW_NODES {
    return Err("View tree too complex: ...");
}
```

**B.3 `generate_script` 的 handler 扫描改为单次遍历**

如果 OOM 的根因是 handler 扫描与模板生成交叉，改为先做一次 view 树遍历收集所有 handler 绑定，再独立生成 script——避免嵌套遍历。

### 策略 C：架构改进（大改动，可选）

**C.1 分离模板生成与 handler 分析**

当前 Vue 生成器在一个 `generate()` 调用里同时做模板生成（node_to_html）和 script 生成（generate_script）。改为两阶段：
1. **分析阶段**：遍历 view 树，收集 handler 绑定、props、events（O(N) 单次遍历）
2. **生成阶段**：用收集的数据独立生成 template 和 script

这能消除模板生成与 handler 分析的交叉递归。

**C.2 View 树编译为中间表示（IR）**

将 AuraNode 树先编译为扁平的 Vue 模板 IR（`Vec<VueTemplateNode>`），再从 IR 生成字符串。避免递归遍历原始树。

---

## 3. 相关的 OOM / 递归问题

### 3.1 已知的 parser OOM 先例

- `examples/capability-tests/oom-event-binop-arg/`：parser 对二元运算符参数的解析曾导致 48GB OOM，已修复（`parse_event_arg` 消费二元运算符）
- `lib.rs:2` 设置了 `#![recursion_limit = "512"]`
- `lib.rs:289` 在专用线程上运行 parser（4MB 栈，避免 Windows 栈溢出）

### 3.2 View DSL 的 `for` 迭代器限制

View DSL 的 `for` 循环解析器（`parse_view_for_loop`）只接受三种迭代表达式：
- `.field`（点前缀字段引用）
- 数字范围
- 单个标识符

**不支持** `ident.field` 链（如 `note.tags`）。这不是 OOM 问题，但限制了 view 表达能力。修复 `for` 解析器以支持 `ident.field` 迭代器是一个独立的改进点。

### 3.3 `style:` 属性位置约束

View DSL 的 `style:` 属性在某些位置（如 `if/else` 条件式后跟子节点）可能被解析器误解析。虽然不是 OOM 的直接原因，但增加了 view 树的复杂性。

### 3.4 Store handler 中的数组运算

`all_tags = .all_tags + note.tags`（数组拼接）在 store handler 里可能被 Vue 生成器展开为复杂的 reactive 链。虽然已从 015-notes 中移除，但生成器应该能处理这种模式而不爆炸。

---

## 4. 实施计划

### 阶段 1：精确诊断（单独 session）

1. 在 Vue 生成器关键函数加调试插桩：
   - `node_to_html`：打印节点类型 + 当前递归深度 + 累计输出长度
   - `generate_script`：打印 handler 扫描进度
   - `extract_classes`：打印条件 style 处理次数
2. 用最小复现 case 运行，收集日志
3. 定位内存爆炸的精确函数和行号

### 阶段 2：修复（基于阶段 1 的诊断）

根据诊断结果选择：
- 如果是 `generate_script` 的 handler 扫描问题 → 策略 B.3（单次遍历）
- 如果是 `extract_classes` 的条件 style 展开 → 策略 A（精确修复）
- 如果是 `node_to_html` 的递归问题 → 策略 B.1（改 Write）

### 阶段 3：防御性优化（可选，与阶段 2 并行）

- B.1：`node_to_html` 改为 `impl Write`
- B.2：view 树节点数上限检查
- 修复 `for` 迭代器支持 `ident.field`

### 阶段 4：回归测试

- 加一个单元测试：用最小复现 case（for + style:if + msg/on）验证生成器不 OOM
- 加一个压力测试：100 个嵌套 if/for 节点，验证生成时间 < 5 秒、内存 < 100MB
- 恢复 015-notes 的完整 sidebar.at（文件夹树 + 标签筛选）

---

## 5. 验收标准

- [ ] 最小复现 case（for + style:if + msg/on）不再 OOM
- [ ] 015-notes 的完整 sidebar.at（198 行，含文件夹树）正常生成
- [ ] 015-notes 的 editor.at（含 autodown_editor）正常生成
- [ ] 生成器内存使用：任意 < 500 行的 .at 文件，峰值内存 < 200MB
- [ ] 生成器时间：任意 < 500 行的 .at 文件，生成时间 < 5 秒
- [ ] 回归测试覆盖 OOM 触发条件

---

## 6. 暂时措施（已实施）

在修复完成前，015-notes 使用简化版：
- sidebar.at：扁平笔记列表（无文件夹分组、无标签筛选）
- editor.at：textarea 替代 autodown_editor
- notes_store.at：无 all_tags 聚合

这些在 commit `c65afcce` 中已提交。
