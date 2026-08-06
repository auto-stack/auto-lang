---
plan: 396
title: a2r-auto-ai-related-improvements
affects: [auto-lang/a2r, auto-lang/trans-rust]
status: in-progress # draft | in-progress | complete
---

# Plan 396: a2r 改进（auto-ai 相关）— 滚动聚合计划

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
| auto-ai-agent | 4 类（B/C/D/E 借用推理） | Plan 019 遗留 | ⏳ 本计划 |
| auto-ai-agent | SOUL const `&str` 类型修正 | Plan 016 遗留 | 📋 可选（comptime 输出推断）|
| ai-config | unit-variant quirk（`auto_val.Value.Nil`） | Plan 021 缺口 3 | ⏳ 本计划 |
| auto-ai-client | （Plan 020 已清零，sed 在） | Plan 020 | 📋 根因修后清 |

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
- **状态**：⏳ 待修

### §2.2 借用推理 C：for-in 对 ReadDir 无条件加 `&`（Plan 019）

- **症状**：`for entry in entries {` 渲染为 `for entry in &entries {`，但 `entries` 是
  `fs::read_dir` 的 `ReadDir`，只 impl `IntoIterator`（by-value），`&ReadDir` 不是迭代器 →
  Rust E0277。
- **根因方向**：a2r 的 for-in 渲染无条件加 `&`（可能为了借用安全）；需对已知 by-value
  迭代器类型（ReadDir 等）不加 `&`。
- **影响点**：`crates/auto-ai-agent/retranspile.sh` C 段（skill.rs / roles.rs）。
- **sed 锚定**：`s#for entry in &entries {#for entry in entries {#g`
- **状态**：⏳ 待修

### §2.3 借用推理 D：函数参数 move 后重用未借用（Plan 019）

- **症状**：`read_to_string(path)` move 了 `path`，后续代码重用 `path` → Rust E0382。
  应渲染 `read_to_string(&path)`。
- **根因方向**：a2r 的 move 分析——函数参数被 owned-接收函数调用后，若后续仍用，
  需改借用。
- **影响点**：`crates/auto-ai-agent/retranspile.sh` D 段。
- **sed 锚定**：`s#read_to_string(path)#read_to_string(&path)#g`
- **状态**：⏳ 待修（命中数 0，可能已部分自愈——修时先验证）

### §2.4 借用推理 E：对 `&str` 多余插 `.as_str()`（Plan 019）

- **症状**：`after_open` 已是 `&str`，a2r 多余渲染 `after_open.as_str()` → Rust E0658
 （`&str` 无 `as_str` 方法）。
- **根因方向**：a2r 的类型注解推断——对已是 `&str` 的值不应插 `as_str()`。
- **影响点**：`crates/auto-ai-agent/retranspile.sh` E 段（skill.rs）。
- **sed 锚定**：`s#after_open\.as_str()#after_open#g`
- **状态**：⏳ 待修

### §2.5 ai-config unit-variant quirk（Plan 021 缺口 3 残留）

- **症状**：`auto_val.Value.Nil` 限定 unit variant pattern 被渲染成非法的
  `auto_val::Value.Nil`（应为 `auto_val::Value::Nil`）。
- **根因方向**：a2r 对限定 unit variant 的 path 渲染——`::` vs `.`。
- **影响点**：`crates/ai-config/retranspile.sh`（unit-variant quirk sed）。
- **状态**：⏳ 待修

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

- [ ] §2.1–§2.5 五条根因全部修复
- [ ] auto-ai 两 `retranspile.sh` 的对应 sed 全部删除（变 no-op 后清理）
- [ ] a2r golden 回归零新增失败
- [ ] auto-ai 三转译 crate 独立 build 0 错（无 sed）、workspace 全绿
