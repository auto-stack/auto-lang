---
plan: 396
title: a2r-auto-ai-related-improvements
affects: [auto-lang/a2r, auto-lang/trans-rust]
status: complete # draft | in-progress | complete
---

# Plan 396: a2r 改进（auto-ai 相关）— 滚动聚合计划

> **2026-08-20 核查**: §2.1–§2.5 五条均未修（0/5 落地），auto-ai 侧 sed workaround 全部仍在（retranspile.sh:172-181 / ai-config:88 / client:99-100）。
> **2026-08-20 二批更新**: **§2.1 仓内根因 ✅ 已修**（`plan-fix/396-loopvar-clone` 合并 `be9d4213`）——golden 实测甄别：结构体字面量路径（description: tc.tool）P11.4 本就覆盖；**函数实参路径是真缺口**（extract_path(tc.args) 缺 clone → E0507），已补字段类型感知的 `maybe_clone_borrowed_iter_field` + 循环变量元素类型解析（golden 19_ownership/003 三形态锁定）。待跨仓验证 retranspile.sh B 段 sed 变 no-op 后删除。剩余 §2.5。
> **2026-08-20 三批更新**: **§2.2/§2.3/§2.4 ✅ 全部根治**（`plan-fix/b3-sed-verify` 合并 efe84664，含 §2.1 裸循环变量补全与 Plan 405 前导回归修复）；auto-ai retranspile 的 Plan 019 B/C/D/E sed 段实证 no-op 后删除（auto-ai 64ba3b2）。剩余 §2.5；另发现两处 master 新回归登记审计 B11（len() 强转 as u32→i64；tool.at 解析失败致 Arc<Tool> 丢 dyn）。
> **2026-08-20 四批更新（B11 根治，auto-ai 重生成归零）**: 上批登记的两处 master 回归由 `plan-fix/b11-regressions`（合并 3f3d0ec3）根治——(a) len() 比较强转按对端类型定向 `partner_len_cast`（Eq/Neq/Lt/Le/Gt/Ge）；(b) "tool.at 解析失败"实为**四个解析缺口**：`Map<K>` 单参硬 arity 错、`Err(Type.Variant { fields })` 嵌套模式（ResultCover+inner+绑定注册+a2r 发射）、顶层 `Type.Variant { fields }` 模式（Plan 165 回归：Dot+`{` 识别 + StructPattern 绑定臂）、枚举结构变体声明冒号字段。**auto-ai 全量重生成首次归零（23→0，rust/src 全绿提交 auto-ai 156c6c8）**。
> **2026-08-21 收官（§2.5+§2.6）**: **§2.5 ✅ 根治**（`plan-fix/396-unit-variant` 合并 1ae3b33c）：两次插桩扑空的根因——`auto_val.Value.Nil` 的 AST 实为 `Dot(TagCover{auto_val,Value}, Nil)`（use.rust 模块名走 lhs_expr→tag_cover 两段吞并、`.Nil` 残留外层 Dot），a2r 由 Cover 裸分支（`{kind}::{tag}`）+ 普通字段访问（`.Nil`）拼出 `auto_val::Value.Nil`；修复在 parser `is_branch_cond_expr_inner` 补无括号三段转换（剥模块段，与带参 Call 转换对齐），golden 06/009 + 单测锁定；ai-config sed 实证 no-op 后删除（auto-ai 31a5304）。**§2.6 ✅ 新增并根治**（`plan-fix/396-nowms` 合并 283990bc）：client 重生成 4 处 E0308 暴露 a2r-std/src/time.rs 手抄滞后（i32 截断，与 stdlib time.rs.at/time.vm.at 声明的 i64 矛盾，epoch 毫秒每 ~24.8 天回绕本就是潜伏 bug），恢复 i64 后 client 归零（auto-ai e05e48d）——**三转译 crate 首次同时全绿**。a2r golden 340/340（基线 339+1）。

> **For Claude:**
> - 构建/测试命令：`cargo test -p auto-lang --lib --features test-trans -- tests::a2r_tests`（a2r golden，基线 319/0）。
> - 验证（auto-ai 侧）：`AUTO=target/debug/auto.exe` 跑 auto-ai 两 `retranspile.sh`，三转译 crate 独立 build 0 错 + workspace 全绿。
>
> **计划性质**：本计划是**滚动聚合计划**——专门承接"auto-ai 项目转译过程中发现、
> 但根因在 auto-lang a2r 侧"的小缺陷。每发现一个此类缺陷，新增一个 §N 条目，
> 而非新开独立计划文件（避免计划序号激增、每个计划过小）。
> 当某缺陷规模较大（独立成计划更清晰）时，才另开新计划并在本文件留指针。
>
> **背景**：auto-ai 的三个转译 crate（ai-config/auto-ai-client/auto-ai-agent）用 a2r
> 从 `.at` 转 Rust。转译产物有几处 a2r codegen 缺陷，长期靠 `retranspile.sh` 的 sed
> 兜底（Plan 019 既定 workaround 模式）。本计划逐个修根因，让对应 sed 变 no-op 后删除。

---

## §1 Goal / 目标

消灭 auto-ai 转译路径上的 a2r codegen 缺陷，让 `retranspile.sh` 的 sed workaround 逐条
变 no-op 并删除。每个缺陷独立修复、独立验证，修完一条勾一条。

**当前 sed workaround 分布**（auto-ai 侧）：

| Crate | sed 条数 | 类别 | 状态 |
|---|---|---|---|
| auto-ai-agent | 4 类（B/C/D/E 借用推理） | Plan 019 遗留 | ✅ §2.1–§2.4 已修，sed 删（auto-ai 64ba3b2） |
| auto-ai-agent | SOUL const `&str` 类型修正 | Plan 016 遗留 | 📋 可选（comptime 输出推断）|
| ai-config | unit-variant quirk（`auto_val.Value.Nil`） | Plan 021 缺口 3 | ✅ §2.5 已修，sed 删（auto-ai 31a5304） |
| auto-ai-client | （Plan 020 已清零，sed 在） | Plan 020 | 📋 根因修后清（tier `Some()` clone 属 Plan 019 B 类兄弟项） |

---

## §2 缺陷条目

### §2.1 借用推理 B：循环变量字段传 owned 参数未 clone（Plan 019）

- **症状**：`for tc in tool_calls { extract_path(tc.args) }` 渲染为
  `extract_path(tc.args)`（缺 `.clone()`），而 `extract_path(args: JsonValue)` 是 owned 参数 →
  Rust E0382（move out of borrowed value）。
- **现状**：a2r 对 **`self.field`** 已正确 clone（`extract_path(self.args.clone())` ✅），
  但 **loopvar 字段**（`tc.args`）未覆盖。sed 补：`extract_path(tc.args)` → `.clone()`。
- **根因方向**：a2r codegen 的借用分析——识别"loopvar.field 传给 owned 参数"需 clone。
  self.field 路径已有逻辑（识别 `self` receiver），需推广到任意循环变量绑定。
- **影响点**：`crates/auto-ai-agent/retranspile.sh` B 段（driver.rs `extract_path(tc.args)` +
  `description: tc.tool,` + agent.rs `tool_to_definition(t)`）。
- **sed 锚定**（修后变 no-op）：
  ```
  s#extract_path(tc\.args)#extract_path(tc.args.clone())#g
  s#description: tc\.tool,#description: tc.tool.clone(),#g
  s#tool_to_definition(t)#tool_to_definition(t.clone())#g
  ```
- **状态**：✅ 已修（be9d4213 函数实参路径 + B3 批 efe84664 裸循环变量补全；sed 已删 auto-ai 64ba3b2）

### §2.2 借用推理 C：for-in 对 ReadDir 无条件加 `&`（Plan 019）

- **症状**：`for entry in entries {` 渲染为 `for entry in &entries {`，但 `entries` 是
  `fs::read_dir` 的 `ReadDir`，只 impl `IntoIterator`（by-value），`&ReadDir` 不是迭代器 →
  Rust E0277。
- **根因方向**：a2r 的 for-in 渲染无条件加 `&`（可能为了借用安全）；需对已知 by-value
  迭代器类型（ReadDir 等）不加 `&`。
- **影响点**：`crates/auto-ai-agent/retranspile.sh` C 段（skill.rs / roles.rs）。
- **sed 锚定**：`s#for entry in &entries {#for entry in entries {#g`
- **状态**：✅ 已修（efe84664，by_value_iter_bindings；sed 已删 auto-ai 64ba3b2）

### §2.3 借用推理 D：函数参数 move 后重用未借用（Plan 019）

- **症状**：`read_to_string(path)` move 了 `path`，后续代码重用 `path` → Rust E0382。
  应渲染 `read_to_string(&path)`。
- **根因方向**：a2r 的 move 分析——函数参数被 owned-接收函数调用后，若后续仍用，
  需改借用。
- **影响点**：`crates/auto-ai-agent/retranspile.sh` D 段。
- **sed 锚定**：`s#read_to_string(path)#read_to_string(&path)#g`
- **状态**：✅ 已修（efe84664，三处 dispatch 借用；sed 已删 auto-ai 64ba3b2）

### §2.4 借用推理 E：对 `&str` 多余插 `.as_str()`（Plan 019）

- **症状**：`after_open` 已是 `&str`，a2r 多余渲染 `after_open.as_str()` → Rust E0658
 （`&str` 无 `as_str` 方法）。
- **根因方向**：a2r 的类型注解推断——对已是 `&str` 的值不应插 `as_str()`。
- **影响点**：`crates/auto-ai-agent/retranspile.sh` E 段（skill.rs）。
- **sed 锚定**：`s#after_open\.as_str()#after_open#g`
- **状态**：✅ 已修（efe84664，is_str_slice_var 补查 StrSlice；sed 已删 auto-ai 64ba3b2）

### §2.5 ai-config unit-variant quirk（Plan 021 缺口 3 残留）

- **症状**：`auto_val.Value.Nil` 限定 unit variant pattern 被渲染成非法的
  `auto_val::Value.Nil`（应为 `auto_val::Value::Nil`）。
- **根因方向**：a2r 对限定 unit variant 的 path 渲染——`::` vs `.`。
- **影响点**：`crates/ai-config/retranspile.sh`（unit-variant quirk sed）。
- **状态**：✅ 已修（1ae3b33c，2026-08-21；根因与修复详见顶部收官注记——
  parser `is_branch_cond_expr_inner` 补无括号三段转换剥模块段，golden 06/009
  锁定；ai-config sed 实证 no-op 后删除 auto-ai 31a5304）

### §2.6 a2r_std time i32 手抄滞后（2026-08-21 收官批新登）

- **症状**：auto-ai-client daemon.at 重生成 4 处 E0308——
  `let deadline: i64 = a2r_std::time::now_ms() + 3000` vs `now_ms() -> i32`。
- **根因**：`crates/a2r-std/src/time.rs` 是 stdlib 手抄的滞后版本——
  `stdlib/auto/time.rs.at` 与 `time.vm.at` 均声明 `now_ms()/now_sec() i64`，
  a2r_std 落成 `i32` + 截断。i32 截断 epoch 毫秒（~1.8e12）每 ~24.8 天回绕
  一次，超时比较在回绕点失准——本就是潜伏 bug，并非 a2r 推理错误。
- **修复**：恢复 i64（283990bc）；无仓内依赖者，golden 不涉及。
- **验证**：client retranspile check 0 错（auto-ai e05e48d）——三转译 crate
  首次同时全绿。
- **状态**：✅ 已修

---

## §3 实施流程（每条缺陷通用）

1. 在 `trans/rust.rs` 定位根因发射点（eprintln debug 实证，不猜）。
2. 修根因，加 golden 测试（`test/a2r/` 下新建 case 或扩展现有）。
3. 回归：a2r golden 319/0 不退步；auto-lang lib 测试不新增失败。
4. 回 auto-ai：用新 auto.exe 跑两 retranspile，确认对应 sed 变 no-op。
5. 删 sed 规则（保留注释说明"§N 已修，sed 删除"），retranspile 重验、三 crate build 0 错。
6. 勾本计划对应条目；更新 auto-ai `KNOWN-DEBT-AND-RISKS.md`。

---

## §4 完成判定

- [x] §2.1–§2.6 六条根因全部修复（finish-plan 2026-08-21 复审逐条验证）
- [x] auto-ai 两 `retranspile.sh` 的对应 sed 全部删除（agent B/C/D/E：64ba3b2；
      ai-config unit-variant：31a5304。tier `Some()` sed 属 Plan 020、SOUL const
      属 Plan 016，均非本计划范围）
- [x] a2r golden 回归零新增失败（340/340 = 基线 339 + 009_qualified_unit_variant）
- [x] auto-ai 三转译 crate 独立 build 0 错 + 重生产物追平当前 a2r
      （ai-config 0 错 / agent 0 错 / client 0 错，2026-08-21 实测；
      auto-ai 侧配套提交 31a5304 + e05e48d）

---

## §5 根因精确定位 + 实施蓝图（2026-08-07 核查，待实施）

> 以下为逐条代码核查（Explore agent）的结论：每条缺陷的 a2r 发射点（文件:行号）、
> 现有判定方式、修复切入点。**5 条均未修**（§2.3 的"可能自愈"经核实不成立）。
> 实施时按 §3 流程逐条做。核查用的 auto.exe 构建于 2026-08-07 03:57（含 Plan 391 §8）。

### §5.1 对应 §2.1（loopvar.field 传 owned 参数未 clone）

- **现有 self.field clone 逻辑**：`arg()` 函数 `rust.rs:8567`、`8575`——仅在 `Self::is_self_dot(expr)` 时追加 `.clone()`。
- **`is_self_dot`** `rust.rs:14400-14402`：只识别 receiver 为 `Ident("self")` 的 Dot，**不识别任意循环变量**（如 `Ident("tc")`）。
- **owned 参数检测**：`fn_struct_param_indices` 预扫描 `rust.rs:10449-10459`（非 Copy 即 owned-struct-param）。
- **`needs_clone` 判定** `rust.rs:7716-7726`：`matches!(arg, Arg::Pos(Expr::Ident(_)))`——只匹配裸 Ident，`tc.args`（`Expr::Dot`）不匹配 → `needs_clone=false`。
- **修复切入点（两处同改）**：(1) `arg()`/`is_self_dot` 推广为识别"任意按引用绑定的 receiver"；(2) `needs_clone` 的 `7720` 行放宽到接受 `Arg::Pos(Expr::Dot(obj,_))` 且 `obj` 是已知按引用绑定。**需新增循环变量绑定追踪集合**（在 `for_stmt` `Iter::Named` `rust.rs:10675-10747` 进入时按借用决策插入）。

### §5.2 对应 §2.2（for-in 对 ReadDir 无条件加 `&`）

- **发射点**：`for_stmt` 的 `Iter::Named` 分支 `rust.rs:10731-10737`——`is_borrowable` 仅看 AST 形式（`Expr::Ident | Expr::Dot`），**无类型判断**，无条件 `sink.body.write(b"&")`。
- **根因**：`is fs.read_dir() { Ok(entries) -> ... }` 的 match binding 未把 `entries` 的内层迭代器类型写入 `local_var_types`，故 `for entry in entries` 无法知道 `entries` 是 by-value `IntoIterator`（`ReadDir`）。
- **修复切入点**：(1) `is_stmt` 的 `Ok(binding)` 分支记录 `read_dir`/`Result<X>` 解包后的内层类型到 `local_var_types`（目前 `rust.rs:11274` 附近只处理 `Some` 不处理 `Ok`）；或 (2) `for_stmt` 加 by-value 迭代器类型白名单（`ReadDir`/`Vec`/`Lines`）比对 `local_var_types`。

### §5.3 对应 §2.3（参数 move 后重用未借用）

- **发射点**：`fs.read_to_string` 分发 `rust.rs:6234-6245`——`expr_as_str(a, out)` 直接 emit arg，无 `&`。
- **move 分析现状**：**完全缺失**。grep `move_analysis`/`borrow_infer`/`last_use`/`needs_borrow`/`owned_param`/`used_after` 全无定义。仅有 escape analysis（`rust.rs:766-929`、`17482-17505`）服务 `view`/`mut`，不覆盖"owned 函数调用让参数失效"。§2.3 自述"可能自愈"**不成立**。
- **唯一相关兜底**：`rust.rs:19324-19328` 硬编码 `str_substr(path,...)` → `&path` 文本替换，与 `read_to_string` 无关。
- **修复切入点**：新增"参数重用分析"——`fn_decl` 处理（`rust.rs:10419`）建参数名集合；调用点（`6234` 及通用 `7591`）记录被 move 的 owned 参数；两遍扫描找出"被 move 后又出现"的参数，对应调用点 emit `&`。`read_to_string` 接受 `AsRef<Path>`，emit `&path` 安全。

### §5.4 对应 §2.4（对已 `&str` 多余插 `.as_str()`）

- **发射点**：调用点参数渲染 `rust.rs:7802-7829`，`arg_is_str_slice` 判定 `7809-7816`——**只查 `current_fn_str_params`**（fn 参数集合），**不查 `local_var_types`**。
- **误命中原因**：`after_open` 来自 `is_stmt` 的 `Some(after_open)` 绑定（`rust.rs:11263`、`11313` 写入 `local_var_types` 为 `StrSlice`），是真实 `&str`，但 `arg_is_str_slice` 不查 `local_var_types` → 判否 → 多余 `.as_str()` → E0658。
- **为什么不直接查 `local_var_types`**：注释 `rust.rs:7810-7814` 解释——`local_var_types` 把 owned `String` 局部也记为 `StrSlice`，直接查会漏加 `.as_str()`（E0308）。
- **修复切入点**：新增"真实 &str 局部"集合（如 `true_str_slice_locals`），只在确定 &str 位置插入（`is_str_returning_scrutinee` 的 `Some` 绑定、`split`/`lines` 循环变量）。`arg_is_str_slice`/`needs_as_str`（`rust.rs:8366-8369`）改为 `current_fn_str_params.contains(name) || true_str_slice_locals.contains(name)`。

### §5.5 对应 §2.5（unit-variant `auto_val.Value.Nil` 渲染成 `auto_val::Value.Nil`）

- **实测（含 Plan 391 §8 的 03:57 build）**：`is v { auto_val.Value.Nil -> ... }` 仍输出 `auto_val::Value.Nil`（点），而 `Value.Str(s)` 正确输出 `Value::Str(s)`。
- **AST（关键发现）**：`auto_val.Value.Nil` 在 is-pattern 里被 parser 的 `tag_cover`（`parser.rs:3703-3736`）错误转换——`tag_cover(&"auto_val")` 读 `.` + `tag_field="Value"`，未见 `(` 走 nil 分支（`:3728-3734`），生成 `Cover::Tag { kind: "auto_val", tag: "Value", bindings: ["_"] }`——把 `auto_val.Value` 当 nil variant，**漏了 `.Nil`**。`.Nil` 残留为外层 `Dot(Cover::Tag, "Nil")`。
- **对照**：带参数的 `auto_val.Value.Str(s)` 经 `parser.rs:3789-3824` 的 Call 转换正确处理（剥 module 前缀，`kind="Value"`）；无参数的 unit variant 不经该转换。
- **值位正常**：`let v = auto_val.Value.Nil`（值位，非 pattern）输出正确 `auto_val::Value::Nil`（走 `Expr::Dot` 值位 emit `rust.rs:3298-3323`）——**仅 is-pattern 路径坏**。
- **修复切入点（推荐 parser 侧）**：扩展 `tag_cover`（`parser.rs:3703`）或 `is_branch_cond_expr_inner` 的 Call 转换（`:3789-3824`），识别"三段 Dot 链 `module.Type.UnitVariant`（无括号）"并转为 `Cover::Tag { kind: "Type", tag: "UnitVariant", bindings: [] }`（剥 module 前缀，与带参变体一致）。难点：区分"两段 module.Type"（需继续读）vs"两段 Type.Variant"（完整 nil variant）。可用 `type_store.lookup_enum_decl_str` 判断 `tag_field` 是否是已知 enum 类型。

### 共用基础缺口（影响多条）

| 缺口 | 现状 | 影响 |
|---|---|---|
| 循环变量绑定追踪集合 | 无（仅 `current_fn_str_params` 容 str-yield loopvar） | §2.1、§2.2 |
| 真实 &str 局部 vs owned String 区分 | `local_var_types` 混记 StrSlice | §2.4 |
| 参数 move-后-重用分析 | 完全缺失 | §2.3 |
| is-pattern 的 module.Type.UnitVariant 转换 | `tag_cover` 误截两段 | §2.5 |

### 实施顺序建议（独立性递增）
1. **§2.5**（parser cover，最独立，不依赖借用推理基础）。
2. **§2.4**（新集合 `true_str_slice_locals`，自洽）。
3. **§2.2**（ReadDir 类型追踪，依赖 `local_var_types` 扩展）。
4. **§2.3**（move 分析，独立新建）。
5. **§2.1**（loopvar clone，依赖循环变量追踪集合，与 §2.2 共享基础）。

每条修完跑 `cargo test -p auto-lang --lib --features test-trans`（a2r golden 回归），全部修完回 auto-ai 删 sed + 三 crate build 验证。
