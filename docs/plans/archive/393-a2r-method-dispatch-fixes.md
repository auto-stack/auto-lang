---
plan: 393
title: a2r-method-dispatch-fixes
affects: [auto-lang/a2r, auto-lang/trans-rust]
status: complete # draft | in-progress | complete
---

# Plan 393: a2r 方法分发三项修复 — E1/E2/E3

> **For Claude:**
> - 构建/测试命令：`cargo test -p auto-lang --lib --features test-trans -- tests::a2r_tests`（a2r golden，基线 328/0）。
> - 前置：Plan 392（E4 sort_by + E5 误归因已闭环；本计划是其 E1/E2/E3 的延续）。
> - worktree：`plan-393/a2r-method-dispatch`（`D:/autostack/auto-lang-393`）。
> - 来源：深度 codegen 路径调查（2026-08-06，`invest/a2r-method-dispatch` worktree），
>   精确发射点已用 eprintln debug 实证确认。
> - 来源 auto-musk：`chats.at`（E1 push_message 命名 + E2 Ok(target) 变通 + E3 map_insert 辅助 fn）。

---

## §1 Goal / 目标

消灭 Plan 392 遗留的 E1/E2/E3 三项 a2r codegen bug。与 392 的区别：这三项的**精确发射点**
已定位（不是猜测），修复路径明确，风险可控。

**关键背景发现**（来自调查）：`fn call()` 内有两条方法分发路径——
- **4736 的 `Expr::Bina(_, Dot, _)` dispatch 是 dead code**（注释明确："the `Expr::Bina` match above is dead code kept for completeness"）。
- **5127 的 `Expr::Dot` dispatch 才是 parser 实际走的路径**。
Plan 392 E1 的守卫加错了分支（5057 在 dead-code 块），所以没生效。本计划改对位置。

---

## §2 三项缺陷（精确发射点 + 修复）

### E1 — `.append()` 过宽重映射为 `.push_str()`（struct 方法被误伤）

- **精确发射点**：`trans/rust.rs:6318`（generic name-remap 表，**活跃路径**）的
  `"append" => Some("push_str")`。最终由 6360 附近的 `write!(out, ".{}(", rust_name)` 发射。
- **dead-code 陷阱**：`5057` 行也有同款 remap，但那是 dead-code 的 `Expr::Bina` 块，改它不生效。
- **调用链**：`fn call()`(3462) → `Expr::Dot` dispatch(5127) → StringBuilder arm(5272)
  `is_sb=false` fall-through → `_ => {}`(5707) → ... → generic remap(6305) → **6318 append→push_str**。
- **修复**（6318 行）：把 `"append" => Some("push_str")` 改为条件——仅当 receiver 是 String
  类型（`local_var_types` 查到 `StrOwned/StrSlice/StrFixed`）时才 remap；struct/unknown 保持 `.append`：
  ```rust
  "append" => {
      let lhs_is_string = if let Expr::Ident(name) = object.as_ref() {
          self.local_var_types.get(name)
              .map(|ty| matches!(ty, Type::StrOwned | Type::StrSlice | Type::StrFixed(_)))
              .unwrap_or(false)
      } else { false };
      if lhs_is_string { Some("push_str") } else { None }
  }
  ```
- **auto-musk 去变通**：`chats.at` 的 `push_message` 改回 `append`（对齐 hw）。

### E2 — `Ok(None)` 被全局替换为 `Ok(())`（破坏 `Result<Option<T>, _>`）

- **精确发射点**：`trans/rust.rs:15959`（`fn fix_result_none_unit`，纯文本后处理，
  **无 AST 上下文**）的 `content.replace("Ok(None)", "Ok(())")`。
- **调用链**：`trans_rust_with_session`(lib.rs:3686 codegen 产出正确 `Ok(None)`)
  → `post_process`(3688) → `fix_counted`(14315) → **`fix_result_none_unit`(15953) 全局 replace**。
- **auto-musk 现状**：6 个 `Result<Option<ChatSession>, str>` 函数全部用 `return Ok(target)`
  变通（target 此刻为 None，绕过字面 `Ok(None)` 替换）。
- **修复**（方案 A，推荐）：重写 `fix_result_none_unit`——先扫描所有 `fn`/`pub fn` 签名，
  收集 Ok-type 为 `()` 的函数（`Result<(),`/`Result<(),>`/`Result<None,`），然后**逐函数**
  （brace-depth tracking）只在这些函数体内替换 `Ok(None)`→`Ok(())`。其他函数（`Result<Option<T>,_>`）
  体内的 `Ok(None)` 保持不动。**参考同文件 `fix_fn_field_calls`(15965) 的"先扫描再逐个 replace"模式**。
- **auto-musk 去变通**：`chats.at` 的 6 处 `return Ok(target)` 恢复为 `return Ok(None)`。

### E3 — `HashMap::insert` 在 if/else 分支尾漏 `;`（Option 泄漏成尾类型）

- **精确发射点**：`trans/rust.rs:10631-10638`（if 分支体）和 `10695-10699`（else 分支体），
  在 `fn if_stmt`(10593) 内。语句上下文的 if 分支尾表达式未补 `;`。
- **根因**：`if_stmt`（语句上下文，值被丢弃）误用了"值上下文"的尾表达式不加 `;` 语义。
  `is_last` 时只发 `\n` 而非 `;\n`，导致 `m.insert(...)`（返回 Option<V>）泄漏为分支尾类型 → E0308。
- **关键区分**：`Expr::If`(2786) = 值上下文（`r = if ...`），尾不加 `;` 正确；
  `if_stmt`(10593) = 语句上下文（Stmt::If），**尾始终应加 `;`**。
- **修复**（10631 + 10695）：语句上下文 if 的分支尾表达式一律 `;\n`：
  ```rust
  // 10631 (branch body)
  if !is_last { sink.body.write(b";\n")?; }
  else { sink.body.write(b";\n")?; }  // E3: 语句上下文尾表达式也要 ;
  // 10695 (else body) 同理
  ```
- **回归注意**：现有 golden `03_control_flow/001_if_basic` 和 `05_expressions/009_comprehensive`
  预期 if/else 分支尾 `println!()` 无 `;`，修复后变 `println!();`——**需同步更新 expected.rs**
  （Rust 仍编译，只是文本 diff）。
- **auto-musk 去变通**：`chats.at` 去掉 `map_insert` 辅助 fn，直接 `map.insert(..)`。

---

## §3 实施顺序（按风险 + 收益）

1. **E3**（最简单）：if_stmt 尾表达式补 `;`，更新 2 个 golden expected。风险最低。
2. **E1**（中等）：6318 append 守卫，仿调查给出的补丁。需确认 `object` 在该作用域可用。
3. **E2**（较复杂）：重写 fix_result_none_unit 用 fn 签名扫描。改动面较大，放最后。

---

## §4 验收标准

每项闭环需：
1. **auto-lang**：最小复现转译出正确 Rust；a2r golden 全绿（基线 328，E3 需更新 2 个 expected）。
2. **auto-musk**：对应 .at 去变通；re-transpile 零 drift；parity_chats 17/17。
3. **更新文档**：auto-musk KNOWN-DEBT-AND-RISKS.md + chats.at 注释。

---

## §5 附：实证陷阱（沿用 Plan 391 §7 / 392 §5）

1. `-o` 参数让 trans 静默退出 0 产生空文件——看默认产物 `<name>.a2r.rs`。
2. `/tmp` 残留 .at 致 CLI 递归扫描爆内存——复现样例放干净空目录。
3. **dead-code 陷阱**：方法分发修 5127 的 `Expr::Dot` 块（含 6318），不是 5057 的 `Expr::Bina`。

---

## §6 实施记录（2026-08-06，全部闭环）

### E3 — ✅ 闭环（commit `21201342`，已合并 master）
if_stmt 的 if/else 分支 `Stmt::Expr` 尾表达式改为一律 `;\n`。12 个 golden expected
同步更新。auto-musk chats `map_insert` 辅助 fn 去除，直接 `map.insert(..)`。

### E1 — ✅ 闭环（commit `ea95e3b8`，已合并 master）
**首次失败的根因**：调查报告说发射点是 6318，实施时 4 处守卫都没生效——一度误判
为"不走 fn call()"。**二次调查真相**：`updated.append()` 确实走 `fn call()`（AST 是
`Expr::Call { name: Expr::Dot(Ident("updated"), "append") }`），发射点确实是 6315 的
generic remap arm。首次失败的唯一原因是 **PATH 用了 master 旧 auto.exe 测试**（worktree
的构建产物没进 PATH），守卫一直在正确的 6315 位置，只是从没被执行过。

修复：6315 的 `"append"` arm 加 struct 守卫（`local_var_types` 查到 struct 时不 remap）。
auto-musk chats `push_message` 改回 `append`（对齐 hw），parity 17/17。

**教训**：worktree 测试必须用 `worktree/target/debug/auto.exe` 全路径，不能依赖 PATH
（PATH 指向主仓库 master 的旧构建）。

### E2 — ✅ 闭环（commit `ea95e3b8`，已合并 master）
重写 `fix_result_none_unit`：扫描 fn 签名，只在返回 `Result<(), _>` 的函数体内
（brace-depth tracking）替换 `Ok(None)` → `Ok(())`。`Result<Option<T>, _>` 函数的
`Ok(None)` 保留（成功但无值）。参考 `fix_fn_field_calls` 的扫描模式。

auto-musk chats 的 not-found 分支（3 处 `if found == false`）从 `Ok(target)` 改为
明确的 `Ok(None)`（成功路径保持 `Ok(target)`）。`Ok(target)` 依赖隐式状态（target
恰好为 None），不如 `Ok(None)` 清晰；E2 修复后 `Ok(None)` 可用，去变通后 parity 17/17。

### 验收
- a2r golden 328/0（E1/E2 无新增回归；E3 的 12 个 expected 已更新）。
- auto-musk 全量测试全绿（lib 207 + parity 全套含 chats 17/17 + tool_atoms 23）。
- 三项端到端闭环：a2r 修复 + auto-musk 去变通（E1 append + E3 map_insert）+ parity。
