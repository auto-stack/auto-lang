---
plan: 392
title: a2r-codegen-fixes-from-musk
affects: [auto-lang/a2r, auto-lang/trans-rust]
status: complete # E4+E5 闭环; E1/E2/E3 移至 Plan 393 闭环 # draft | in-progress | complete
---

# Plan 392: a2r codegen 五项修复 — 来自 auto-musk 变通复审

> **For Claude:**
> - 构建/测试命令：`cargo test -p auto-lang`（回归）、`cargo test -p auto-lang --lib --features test-trans -- tests::a2r_tests`（a2r golden，基线 326/0）。
> - 前置 skill：无；需熟悉 `trans/rust.rs` 的方法表、后处理 pass、借用注入。
> - 回归要求：现有 a2r golden 326/0 不退化；auto-musk 真实文件 re-transpile 零回归。
> - worktree：`plan-392/a2r-codegen-fixes`（`D:/autostack/auto-lang-392`）。
> - 来源：auto-musk Plan 018 变通复审（2026-08-06），详见 auto-musk
>   `docs/plans/KNOWN-DEBT-AND-RISKS.md` + .at 源码注释。
> - 前置：Plan 391（D1-D6 已闭环，本计划是其后续）。

---

## §1 Goal / 目标

消灭 auto-musk 残留的 **5 项 a2r codegen 缺陷**（E1-E5），让对应 .at 源码去掉变通。
这些与 Plan 391 的区别：391 是"语言/解析层面"的限制，本计划是"已支持但 codegen
发射错误"的 bug——每项修复路径清晰、范围局部、风险可控。

**战略**：逐项闭环（仿 Plan 391 范式）—— auto-lang commit（修复 + golden 回归）↔
auto-musk commit（去变通 + parity 验证）。同 session 跨仓库推进。

---

## §2 五项缺陷（实证 + 精确定位）

> 复现命令：`auto trans --path <name>.at rust`（产物 `<name>.a2r.rs`）。
> **务必用干净空目录放复现样例**（`/tmp/` 残留 .at 会触发 CLI 递归扫描爆内存，
> 见 Plan 391 §7 教训）。

### E1 — `.append()` 无条件重映射为 `.push_str()`（过宽，误伤 struct 方法）

- **症状**：a2r 把所有 `.append(x)` 重映射为 `.push_str(x)`（String 方法），误伤
  自定义 struct 的 `.append()` 方法。
- **影响**：auto-musk `chats.at` 的 ChatSession 被迫把 hw 的 `append` 方法改名为
  `push_message`（注释原文："a2r 的 String 方法重映射表把 receiver 是 struct 的
  `.append` 也改成 `.push_str`"）。
- **定位**：`trans/rust.rs:5057` 和 `6314` 两处 `"append" => Some("push_str")`。
- **修复方向**：加 receiver 类型守卫——仅当 receiver 是 `String`/`StrOwned` 类型时
  才重映射；struct 类型的 `.append` 保持原样。参考已有的 receiver 类型判断逻辑
 （如 `needs_as_str` 的 receiver 守卫）。
- **auto-musk 去变通**：`chats.at` 的 `push_message` 改回 `append`，对齐 hw。

### E2 — `Ok(None)` 被无条件改写为 `Ok(())`（破坏 `Result<Option<T>, _>`）

- **症状**：a2r 后处理 `fix_result_none_unit`（行15953-15961）用全局字符串替换
  `content.replace("Ok(None)", "Ok(())")`，把所有 `Ok(None)` 改成 `Ok(())`。当函数
  返回 `Result<Option<T>, E>` 时，`Ok(None)`（成功但无值）被误改 → 类型错误。
- **影响**：auto-musk `chats.at` 的 not-found 分支被迫写 `return Ok(target)` 而非
  `Ok(None)`（绕过字面替换）。
- **定位**：`trans/rust.rs:15959`（`fix_result_none_unit` 的 value 位置替换）。
- **修复方向（中等难度）**：文本后处理难判断函数返回类型。两个方案：
  (a) **正则上下文**：仅当函数的返回类型签名是 `Result<(), _>`（而非 `Result<Option<_>, _>`）
  时才替换——需扫描 fn 签名建立"哪些函数该改"的集合，再针对性替换。
  (b) **AST 级**：在 codegen 阶段（非文本后处理）根据 `fn_ret_types` 决定是否
  发 `Ok(())` vs `Ok(None)`。方案 b 更彻底但改动面大。
  **建议先做 a**：grep 所有 fn 的返回类型，若含 `Option` 则标记该 fn 的 `Ok(None)`
  不替换。
- **auto-musk 去变通**：`chats.at` not-found 分支恢复 `Ok(None)`。

### E3 — `HashMap::insert` 在 if/else 分支尾漏 `;`（Option 泄漏成尾类型）

- **症状**：`HashMap::insert` 返回 `Option<V>`，a2r 在 if/else 分支末尾漏 `;`，
  导致 `Option` 泄漏成分支尾类型 → `if cond { map.insert(..) } else { ... }` 的
  两分支类型不一致（E0308）。
- **影响**：auto-musk `chats.at` 用 `map_insert` 辅助 fn（绑定结果到 let，返回 void）
  绕过（行221-225）。
- **定位**：a2r codegen 对 if/else 分支体里"返回值的表达式"未补 `;`。需定位
  if/else 分支的 codegen 路径（`is` 匹配块 / `if` 语句的分支体）。
- **修复方向**：if/else 分支体若以"返回非 unit 值的表达式"结尾，且该值未被使用
  （非尾表达式），补 `;`。或更精确：`HashMap::insert` 调用在语句位置时强制补 `;`。
- **auto-musk 去变通**：`chats.at` 去掉 `map_insert` 辅助 fn，直接 `map.insert(..)`。

### E4 — `List.sort_by(closure)` 不支持（a2r 方法表缺失，VM 已有 opcode）

- **症状**：a2r codegen 的 Vec 方法表只有 `"sort"`，没有 `"sort_by"`。VM 层已有
  `NATIVE_LIST_SORT_BY`（opcode 2068，`native_catalog.rs:52`），但 a2r 没接。
- **影响**：auto-musk `chats.at`（list 手动插入排序）、`relay_profession.at`、
  `wiki.at`（insert_sorted 手动排序）都被迫手写排序。
- **定位**：`trans/rust.rs` 的 Vec 方法表（行13605/15019/15098 三处 `"sort"` 列表）。
- **修复方向**：方法表加 `"sort_by"`，codegen 转发为 `.sort_by(closure)`。Auto 的
  闭包语法（`|a, b| ...`）a2r 已支持（`Expr::Closure`），转发即可。
- **auto-musk 去变通**：`chats.at` list 排序改用 `sort_by`；wiki/relay_profession
  的 insert_sorted 可保留（那是插入排序，非排序场景）或评估。

### E5 — `HashMap.get(&str 键)` 借用（验证是否已可用，可能无需改 a2r）

- **症状（待验证）**：auto-musk `wiki.at` load 用 List 线性查找（`find_meta`）替代
  `HashMap.get()`，注释说"a2r HashMap.get() 借用规则对 `&str` 键不可靠"。
- **探针发现**：之前 Plan 391 复审时探针证实 `HashMap.get(slug)` 当 slug 是 `&str`
  方法参数时可编译（rustc EXIT=0）。**这个限制可能已不存在**（a2r 借用注入已改进）。
- **修复方向**：**先验证**——在 wiki.at 里把 load 的线性查找改回 `manifest.get(slug)`，
  re-transpile + 编译。若成功则无需改 a2r，直接去变通；若失败再定位借用问题。
- **auto-musk 去变通**：`wiki.at` 去掉 `find_meta`，load 直接用 HashMap.get。

---

## §3 实施批次（按难度 + 收益排序）

### 批次 A（立即可做，低风险）— E1 / E4 / E5
- **E4 sort_by**：方法表加一项，最简单，收益大（3 个模块受益）。
- **E1 .append 守卫**：加 receiver 类型判断，局部改动。
- **E5 wiki get 验证**：先验证，若可用则纯去变通（零 a2r 改动）。

### 批次 B（中等难度）— E3 / E2
- **E3 insert 漏 `;`**：需定位 if/else 分支 codegen。
- **E2 Ok(None) 改写**：需上下文判断（fn 返回类型），改动面较大。

---

## §4 验收标准

每项闭环需同时满足：
1. **auto-lang**：最小复现样例转译出正确 Rust；a2r golden 326/0 不退化。
2. **auto-musk**：对应 .at 去变通；re-transpile 零 drift；parity 套件全绿。
3. **更新文档**：auto-musk KNOWN-DEBT-AND-RISKS.md 移除已解决项。

---

## §5 附：实证陷阱（沿用 Plan 391 §7）

1. **`-o` 参数让 trans 静默退出 0 产生空文件**——判断成功看默认产物 `<name>.a2r.rs`。
2. **`/tmp` 残留 .at 致 CLI 递归扫描爆内存**——复现样例放干净空目录，勿放 `/tmp` 大目录。
3. **stdout `[trans] ... ->` 日志行不是产物**——产物是 `.a2r.rs` 文件。
