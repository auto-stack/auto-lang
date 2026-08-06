---
plan: 393
title: a2r-method-dispatch-fixes
affects: [auto-lang/a2r, auto-lang/trans-rust]
status: draft # draft | in-progress | complete
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

## §6 实施记录（2026-08-06）

### E3 — ✅ 闭环（commit `21201342`，已合并 master）
if_stmt 的 if/else 分支 `Stmt::Expr` 尾表达式改为一律 `;\n`。12 个 golden expected
同步更新（if/else 尾 `println!()`/赋值表达式加 `;`，Rust 仍编译）。golden 328/0。

### E1 — ⏳ 待续（关键新发现）
调查报告（invest/a2r-method-dispatch worktree）认为发射点是 6318 的 generic remap，
但实施时实证发现：**`updated.append(msg)` 根本不走 `fn call()`（3462）**！在
`fn call()` 入口加 debug（`call.name` 含 append）无任何输出，说明该方法调用的
AST 不是 `Expr::Call { name: Expr::Dot(..), .. }`，或它的发射入口不是 `fn call()`。

**下一步**：用 `eprintln!` 在 `fn expr()`（1689）的 `Expr::Call` 分支 + `Expr::Bina`
分支 + `Expr::Dot` 分支入口加 debug，确认 `updated.append(msg)` 的 AST 形态和实际
发射入口。可能 parser 把 `obj.method(args)` 解析成了非 `Expr::Call` 的形式（如
`Expr::Bina(Expr::Call(append, args), Dot, updated)` 或方法调用内联在 expr 层）。

**已排除**：
- 5127 的 `Expr::Dot` dispatch（debug 无输出，未进入）
- 4736 的 `maybe_module_method` 块（Bina-Dot debug 无输出）
- 6314/6328 的 generic remap 守卫（debug 无输出）
- `fn call()` 3462 入口（debug 无输出）

所以发射点在**以上所有都不经过**的某条路径——需从 `fn expr()` 重新追踪。

### E2 — ⏳ 待续（方案清晰，未实施）
`fix_result_none_unit`（15959）的 `content.replace("Ok(None)", "Ok(())")` 全局替换。
方案 A（推荐）：重写为先扫描 fn 签名收集 `Result<(), _>` 函数集，再逐函数（brace-depth
tracking）只在这些函数体内替换。参考 `fix_fn_field_calls`（15965）的扫描模式。
