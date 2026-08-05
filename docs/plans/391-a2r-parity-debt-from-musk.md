---
plan: 391
title: a2r-parity-debt-from-musk
affects: [auto-lang/a2r, auto-lang/parser, auto-lang/trans-rust]
status: complete # draft | in-progress | complete
---

# Plan 391: a2r 功能一致性 debt 清单 — 来自 auto-musk Plan 018 dogfooding

> **For Claude:**
> - 构建/测试命令：`cargo test -p auto-lang`（回归）、`cargo test -p a2r-std`（runtime）。
> - 前置 skill：无；需熟悉 `trans/rust.rs`（codegen）、`parser.rs`（语法）、`infer/mod.rs`（类型推断）。
> - 回归要求：现有 `cargo test -p auto-lang` 全套不退化；a2r golden 不改变（除非本计划明确新增）。
> - 来源：auto-musk `docs/plans/archive/018-auto-parity-functional.md` 归档复审（2026-08-06）+
>   `docs/plans/KNOWN-DEBT-AND-RISKS.md`。8 个转译探针实证（2026-08-06，基于 auto-lang
>   master `b8f26186` 及之后）。

---

## §1 Goal / 目标

消灭 auto-musk Plan 018 dogfooding 残留的 **6 项 a2r 转译器限制**，让对应的 .at 源码能去掉
变通写法、用 Rust 标准写法。每项均已用最小 .at 样例实证复现（见 §2），根因明确（见 §3）。

**战略定位**：本计划是 Auto 语言 dogfooding 的延续——auto-musk 已用 a2r 移植了 11 个模块
（11 个 parity 套件 104 测试），这些残留限制是"Auto 真正成为 Rust 开发语言"的最后一批障碍。

**不在本计划范围**：
- 休眠镜像 full parity（auto-musk tools/spec_tools/orch_tools 等）——那是接线工程，非 a2r 问题。
- HTTP 层测试缺口——auto-musk 测试工程，非 a2r。

---

## §2 六项限制（实证，2026-08-06）

> 每项给出：症状 / 最小复现 .at / 当前 a2r 产出（错误或失真）/ 期望 Rust。
> 复现命令：`auto trans --path <name>.at rust`（产物为同目录 `<name>.a2r.rs`；**勿用 `-o`，它会让进程静默退出 0 产生空文件**）。

### D1 — `fs.metadata().len()` 强转 i32（Auto int 模型 codegen）

- **症状**：`let sz u64 = m.len()` 发射为 `(m.len() as i32)`——u64 变量被强转 i32，既丢精度
  又导致 `Some(sz)` 喂 `Option<u64>` 时类型不匹配（E0308）。
- **影响**：auto-musk `wiki.at` TreeNode file 节点 `size/modified` 被迫设 `None`（无法喂
  `Option<u64>`），与手写版（从 `fs::Metadata` 丰富）有 metadata 差异。
- **最小复现**：
  ```auto
  use.rust std::fs
  type Node { size Option<u64> }
  ext Node {
      fn from_meta(path str) Node {
          let m = fs.metadata(path)
          let sz u64 = m.len()
          return Node(size: Some(sz))
      }
  }
  ```
- **当前 a2r 产出**：`let sz: u64 = (m.len() as i32);` ← 强转 i32，类型矛盾。
- **期望 Rust**：`let sz: u64 = m.len();`（`fs::metadata.len()` 本就返回 u64，无需转换）。
- **根因方向**：a2r 的 Auto int 模型把所有整数方法返回无条件 `as i32`。需在
  `trans/rust.rs` 的方法返回类型推断里，对已知返回 u64 的 stdlib 方法（`len`、`metadata.len`）
  按目标变量标注类型发射，而非一刀切 `as i32`。

### D2 — `HashMap.get()` 标注类型时缺 `&` 借用（借用注入不可靠）

- **症状**：`let v Option<List<str>> = m.get("k")` 发射为 `m.get("k")`（缺 `&`），且标注为
  `Option<Vec<String>>`（应为 `Option<&Vec<String>>`）→ E0308 类型不匹配。
- **影响**：auto-musk `task_plan.at` 的 `graph.get(node)`、`wiki.at`、`specs.at` 均被迫"不标注
  类型、直接 `is m.get(k) { Some->.. }` 匹配"——能工作但无法显式标注中间变量。
- **最小复现**：
  ```auto
  use.rust std::collections::HashMap
  ext Foo {
      fn check(m HashMap<str, List<str>>) List<str> {
          let v Option<List<str>> = m.get("k")
          is v { Some(deps) -> return deps.clone(), None -> return [] }
      }
  }
  ```
- **当前 a2r 产出**：`let v: Option<Vec<String>> = m.get("k");` ← 缺 `&`，且 owned vs 借用类型错配。
- **期望 Rust**：`let v: Option<&Vec<String>> = m.get(&"k");`（或 `m.get("k")` 配合 `Option<&Vec>` 标注）。
- **根因方向**：`trans/rust.rs` 的 `needs_ref_borrow`/借用注入逻辑对"显式标注 Option<容器>
  变量接收 .get() 结果"的场景未注入 `&`，且类型标注未把容器元素转为引用。注意：不标注直接
  match 时借用是正确的（specs/handoff_store 实证），只有"显式 let 标注"路径有问题。

### D3 — `path.split()` 标注强制 `Vec<String>`（实为 `Vec<&str>`）

- **症状**：`let parts List<str> = path.split(".")` 发射为
  `let parts: Vec<String> = path.split(".").collect::<Vec<_>>();`——split 产出 `&str`，
  collect 推断 `Vec<&str>`，与标注 `Vec<String>` 冲突 → E0308。
- **影响**：auto-musk `task_plan.at validate_handoff_path`、`handoff_store.at resolve_path`
  被迫"不标注 `let parts = ...`"（靠 Rust 推断 `Vec<&str>`）。
- **最小复现**：
  ```auto
  fn validate(path str) bool {
      let parts List<str> = path.split(".")
      if parts.len() < 3 { return false }
      return true
  }
  ```
- **当前 a2r 产出**：`let parts: Vec<String> = path.split(".").collect::<Vec<_>>();` ← 标注与推断冲突。
- **期望 Rust**：要么标注 `Vec<&str>`，要么不标注让 Rust 推断 `Vec<&str>`。
- **根因方向**：`trans/rust.rs` 对 `str` 类型的 `List<str>` 标注无条件转 `Vec<String>`（owned），
  未区分"来自 split 等借用产物的 `Vec<&str>`"。可考虑：split 的 collect 结果不强制 owned 标注，
  或引入 `List<&str>` 显式借用元素类型。

### D4 — `env::var("X").ok()` 方法链无法解析（parser 限制）

- **症状**：`let v Option<str> = env::var("AAID_URL").ok()` 报
  `Expected Asn, but found .`——parser 无法解析"`expr.method()` 整体作为赋值 RHS"。
- **影响**：auto-musk `app_config.at` 的 `AAID_URL` env 覆盖缺失（hw 用
  `std::env::var("AAID_URL").ok()` 覆盖 daemon_url），是 B 类手写边界。
- **最小复现**：
  ```auto
  use.rust std::env
  fn get_url() str {
      let v Option<str> = env::var("AAID_URL").ok()
      is v { Some(u) -> return u, None -> return "default" }
  }
  ```
- **当前 a2r 产出**：空（解析失败）。
- **期望 Rust**：`let v: Option<String> = std::env::var("AAID_URL").ok();`
- **根因方向**：`parser.rs` 的 `parse_store_stmt` → `rhs_expr` 在遇到
  `Path::method(args).method2()`（路径调用的方法链）时，`.method2()` 的 Dot 无法挂到
  `env::var(...)` 这个 Call 上。对比：变量上的方法链（`s.ok()`）可解析；路径调用上的方法链不行。

### D5 — `Result<(), T>` unit 类型无法解析（parser 限制）

- **症状**：`fn save() Result<(), str>` 报 `Expected type, got )`——类型位置的 `()` unit
  无法解析；`Ok(())` 也无法解析（`Expected type, got )` 在表达式位置）。
- **影响**：auto-musk `specs.at`/`handoff_store.at` 的写方法被迫用 `Result<bool, str>`（bool
  载荷承载成功语义），与 hw 的 `Result<(), String>` 签名不一致（跨模块约定，行为等价但签名有差）。
- **最小复现**：
  ```auto
  fn save() Result<(), str> {
      return Ok(())
  }
  ```
- **当前 a2r 产出**：空（解析失败）。
- **期望 Rust**：`fn save() -> Result<(), String> { Ok(()) }`
- **根因方向**：`parser.rs` 的类型解析（`parse_type`/`type_expr`）不支持空元组 `()` 作为类型；
  表达式解析不支持 `()` unit 字面量。需在这两处加 `LParen RParen` → unit 的分支。

### D6 — `impl Trait for Type` 解析顺序反（语言设计 + a2r bug）

- **症状**：Auto 无 trait impl 语法。若用 Rust 风格 `impl TryFrom<Node> for Foo { ... }`，
  a2r 误解析为 `impl Foo for TryFrom`（target 与 trait **顺序反**），且 `try_from` 参数多出
  `&self`。
- **影响**：auto-musk `task_plan.at` 的 `impl TryFrom<Node>` 被迫改为 `static fn from_node`
  （a2r 无 trait impl 语法；parity 测试分别调 hw `try_from` / ag `from_node` 比行为）。
- **最小复现**：
  ```auto
  impl TryFrom<Node> for Foo {
      fn try_from(n Node) Result<Foo, str> { return Ok(Foo()) }
  }
  ```
- **当前 a2r 产出**：`impl Foo for TryFrom { fn try_from(&self, n: Node) -> Result<Foo, String> {...} }`
  ← 顺序反 + 多余 `&self`。
- **期望**：要么支持正确的 `impl Trait for Type` 语法，要么在语言层面明确拒绝（给出清晰错误，
  而非静默产出错误代码）。
- **根因方向**：`parser.rs` 无 `parse_impl` 专用路径，`impl` 被当作普通 item 误解析。本项优先级
  较低（Auto 语言设计层面是否要支持 trait impl 是更大的决策）；**最低限度**应让 a2r 对
  `impl X for Y` 报清晰错误而非产出反转代码。

---

## §3 根因定位（按 a2r 源码模块）

| 限制 | 主要源码位置 | 修改性质 |
|---|---|---|
| D1 `.len() as i32` | `trans/rust.rs` 整数方法返回 codegen | codegen：按目标标注发射 |
| D2 HashMap.get 标注缺 `&` | `trans/rust.rs` `needs_ref_borrow` + 类型标注路径 | codegen：显式标注路径补借用 |
| D3 split 强制 Vec<String> | `trans/rust.rs` `List<str>` → `Vec<String>` 标注 | codegen：区分 owned/borrowed 元素 |
| D4 env::var().ok() | `parser.rs` `parse_store_stmt`/`rhs_expr` 方法链 | parser：路径调用方法链 |
| D5 Result<(), T> | `parser.rs` 类型解析 + 表达式解析（`()` unit） | parser：unit 类型/字面量 |
| D6 impl Trait for Type | `parser.rs`（无 `parse_impl`） | parser + 语言设计 |

**共性**：D1-D3 是 **codegen**（`trans/rust.rs`）问题——转译能跑通但产出失真；D4-D6 是
**parser**（`parser.rs`）问题——直接解析失败或误解析。两类都可用"最小样例 + golden 回归"驱动。

---

## §4 实施策略（建议分批，每批独立可验收）

> 仿 Plan 018 §14 / Plan 389 的 worktree 闭环范式：每项 = 一个 auto-lang commit（修复 +
> 回归 golden）+ 一个 auto-musk commit（去变通 + 跑 parity）。可同 session 跨仓库推进。

### 批次 A：codegen 三项（D1/D2/D3）—— 收益最高、风险最低
- D2/D3 影响面大（所有 HashMap/split 场景），且 auto-musk 已有 specs/task_plan/handoff_store
  的 parity 测试可直接验证去变通。
- D1 影响 wiki metadata，去变通后 wiki parity 可补充 metadata 断言。
- 每项加最小复现样例到 a2r golden（`tests/` 或 golden 目录）。

### 批次 B：parser 两项（D4/D5）—— 中等难度
- D5（unit 类型）相对独立，parser 加 `()` 分支即可。
- D4（方法链）需理解 `rhs_expr` 的路径调用 vs 变量方法链差异。
- 去变通收益：app_config env 覆盖（D4）、specs/handoff Result 签名对齐（D5）。

### 批次 C：语言设计（D6）—— 需决策
- trait impl 是否进 Auto 语言是设计决策。最低限度先让 a2r 对 `impl X for Y` 报清晰错误。
- 若决定支持，参照 Plan 380（a2r-rust-interop）的 impl 处理经验。

---

## §5 验收标准

每项限制闭环需同时满足：
1. **auto-lang**：最小复现样例转译出**期望的 Rust**（见 §2 各项）；新增回归 golden；
   `cargo test -p auto-lang` 全套不退化。
2. **auto-musk**：对应 .at 去变通（用 Rust 标准写法）；re-transpile 零 drift；对应 parity
   套件全绿。
3. **更新文档**：auto-musk `KNOWN-DEBT-AND-RISKS.md` 移除已解决项。

---

## §6 附：实证时的两个陷阱（避免重复踩）

1. **`-o` 参数会让 trans 静默退出 0 产生空文件**——判断转译成功必须看默认产物
   `<name>.a2r.rs` 是否非空且含期望函数，**不能只看 EXIT=0**。
2. **stdout 的 `[trans] ... ->` 日志行不是产物**——产物是 `.a2r.rs` 文件。
   （Plan 018 复审清理时曾因此误判 specs.at"内存爆炸"，实际是用了修复前的旧构建 + 判断方法错。）

---

## §7 闭环记录（2026-08-06）

六项限制（D1-D6）全部修复并合入 master（merge commit）。实施方式：worktree
`plan-391/a2r-parity-debt`（已合并删除），4 个诊断 agent 并行（D1/D2/D3/D4）+
人工整合 + D5/D6。

| ID | 修复 | commit | auto-musk 去变通 |
|---|---|---|---|
| D1 | `.len()` 宽类型(u64/i64/usize)抑制 as i32 cast | `492a93a6` | wiki build_tree file size 真实化 ✅ |
| D2 | `let v: Option<T> = m.get(k)` 标注改写为 `Option<&T>` | `492a93a6` | task_plan dfs graph.get 加标注 ✅ |
| D3 | `List<str>` + split() 跳过强制 `Vec<String>` 标注 | `3ba03b56` | task_plan validate_handoff_path 加标注 ✅ |
| D4 | 表达式位置 `::` 路径分隔符(`env::var(x).ok()`) | `492a93a6` | app_config env 覆盖待接线侧改动 |
| D5 | `()` → Type::Void(类型) + Expr::Tuple([])(表达式) | `6f8162f6`+`492a93a6` | handoff_store save Result<(),str> ✅ |
| D6 | `impl Trait for Type` 清晰错误(不误伤 ext for) | `3ba03b56`+`c4dff6dd` | task_plan trait impl 维持 static fn |

**回归**：a2r golden 326/0（修复了 ext_for/ext_from）；lib 2801 passed /
22 pre-existing failures（dstr/ui_gen/route/vm::codegen，与 Plan 391 无关）。
auto-musk 12 真实文件 re-transpile 零回归；全量测试全绿。

**关键发现**：/tmp 残留 .at 文件导致 CLI 递归扫描爆内存（`trans_rust_with_session`
向上回溯扫描兄弟 .at）——此前所有"孤立样例挂起"现象均源于此，非真实 codegen bug。
**教训**：转译测试须用干净空目录，不能放在 /tmp 大目录下。

**已知局限**（非本计划范围，记录于 auto-musk KNOWN-DEBT-AND-RISKS.md）：
- 多段路径 codegen：`std::env::var`（多段 `::`）parser 可解析但 codegen 发点。
- wiki modified 仍 None（`duration_since` 方法链，闭包内 `.len()` 仍 cast）。
- Auto trait impl 是语言设计决策（D6 仅清晰报错，未加语法支持）。
