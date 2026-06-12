# Plan 277: 计划 240/243/244/245/299 审计修复

**日期**: 2026-06-12
**状态**: 待实施
**目标**: 修复 Plans 240/243/244/245/299 审计中发现的关键正确性 bug 和架构问题
**优先级**: P0（LSP 崩溃/MCP 核心功能缺失）→ P1（性能/架构）→ P2（完整性补齐）

---

## 审计背景

对 Plan 240 (Cookbook a2r)、Plan 243 (LSP)、Plan 244 (Language Tour)、Plan 245 (TAPL)、Plan 299 (AutoUI MCP V2) 进行了 7 角度 × 6 候选的全面审查，产生 10 个确认发现。本文档将这些发现转化为可执行的修复任务。

---

## Phase 1: LSP 正确性修复（P0 — 用户可见崩溃/数据损坏）

### 1.1 修复 UTF-16 / 字节偏移不匹配

**问题**: `completion.rs:38` 和 `signature_help.rs:7` 将 LSP `Position.character`（UTF-16 code unit 偏移）当作字节偏移直接切片 `&str`。非 ASCII 文本（中文注释、字符串）导致 panic。

**影响**: 含中文的 .at 文件触发 completion/signature_help 时 LSP 静默崩溃（catch_unwind 吞掉 panic）。

**修复方案**: 在 `crates/auto-lsp/src/` 中创建 `position.rs` 工具模块：

```rust
/// LSP Position.character (UTF-16 offset) → byte offset in a Rust &str
pub fn utf16_to_byte_offset(line: &str, utf16_offset: u32) -> usize {
    let mut utf16_count = 0u32;
    for (byte_idx, ch) in line.char_indices() {
        if utf16_count >= utf16_offset {
            return byte_idx;
        }
        utf16_count += ch.len_utf16() as u32;
    }
    line.len()
}

/// Safely slice a line at a UTF-16 character offset
pub fn slice_line_at_char(line: &str, char_offset: u32) -> &str {
    let byte_offset = utf16_to_byte_offset(line, char_offset);
    &line[..byte_offset]
}
```

替换所有 `&line[..position.character as usize]` 为 `slice_line_at_char(line, position.character)`。

**文件**:
- `crates/auto-lsp/src/position.rs`（新建）
- `crates/auto-lsp/src/completion.rs`（~line 38, 296）
- `crates/auto-lsp/src/signature_help.rs`（~line 7, 134）
- `crates/auto-lsp/src/goto_def.rs`（~line 258）
- `crates/auto-lsp/src/hover_info.rs`（~line 197）
- `crates/auto-lsp/src/inlay_hints.rs`（~line 95-118，char count → UTF-16 转换）
- `crates/auto-lsp/src/lib.rs`（添加 mod position）

**验证**: 创建含中文的测试 .at 文件，验证 completion/hover/goto_def 不 panic。

### 1.2 修复 Windows CRLF 偏移计算

**问题**: `backend.rs:944-966` 的 `position_to_offset` 用 `content.lines()`（自动剥离 `\r\n`）但每行只加 1 字节换行符。Windows `\r\n` 文件每行累积偏移差 1 字节。

**影响**: Windows 用户增量编辑时文档内容被截断或错位。

**修复方案**: 不用 `.lines()`，改用按 `\n` 手动分割保留 `\r`：

```rust
fn position_to_offset(content: &str, position: &Position) -> usize {
    let mut offset = 0;
    let mut line_index = 0;
    for ch in content.chars() {
        if line_index == position.line as usize {
            break;
        }
        offset += ch.len_utf8();
        if ch == '\n' {
            line_index += 1;
        }
    }
    // 在当前行内，将 UTF-16 character offset 转为字节偏移
    let remaining = &content[offset..];
    let line_end = remaining.find('\n').unwrap_or(remaining.len());
    let line = &remaining[..line_end];
    offset + utf16_to_byte_offset(line, position.character)
}
```

**文件**: `crates/auto-lsp/src/backend.rs`（~line 944-966）

**验证**: 构造 CRLF 换行的 .at 文件，验证 `apply_text_change` 后内容不变。

### 1.3 修复 diagnostics 警告定位硬编码 line:0

**问题**: `diagnostics.rs:72-78` 的 `warning_to_diagnostic` 始终返回 `Position { line: 0, character: 0 }`。所有 parser warning 都显示在文件开头。

**修复方案**: 从 warning 的 span 信息中提取位置，复用已有的 `extract_location_from_error` 逻辑（参考 `auto_error_to_diagnostic` 在 line 54 的实现）。

```rust
fn warning_to_diagnostic(warning: &ParseWarning, content: &str) -> Diagnostic {
    let range = if let Some(span) = &warning.span {
        span_to_range(span, content)
    } else {
        Range { start: Position::new(0, 0), end: Position::new(0, 0) }
    };
    Diagnostic::new(range, DiagnosticSeverity::WARNING, ... ...)
}
```

**文件**: `crates/auto-lsp/src/diagnostics.rs`（~line 72-78）

**验证**: 编写产生 warning 的 .at 文件，验证波浪线出现在正确行。

---

## Phase 2: LSP 架构清理（P1 — 消除重复代码 + 性能）

### 2.1 提取共享工具模块，消除 5 处重复

**问题**: `get_word_at_position` + `is_identifier_char` 复制到 5 个文件（~75 行重复）。

**修复方案**: 移入新建的 `crates/auto-lsp/src/position.rs`，所有模块 import 使用。

**文件**:
- `crates/auto-lsp/src/position.rs`（新建，含 get_word_at_position, is_identifier_char, utf16_to_byte_offset, slice_line_at_char）
- `crates/auto-lsp/src/backend.rs`（删除 ~line 899-940 的重复实现）
- `crates/auto-lsp/src/completion.rs`（删除 ~line 296 的重复实现）
- `crates/auto-lsp/src/goto_def.rs`（删除 ~line 258 的重复实现）
- `crates/auto-lsp/src/hover_info.rs`（删除 ~line 197 的重复实现）
- `crates/auto-lsp/src/signature_help.rs`（删除 ~line 134 的重复实现）

### 2.2 修复 `infer_variable_type_heuristic` 不识别 `var` 关键字

**问题**: 3 份复制的 `infer_variable_type_heuristic` 只匹配 `let` 和 `mut`，不匹配 AutoLang 的 `var`。

**修复方案**: 统一到 `position.rs`（或 `type_inference.rs`），添加 `var` 匹配：

```rust
fn infer_variable_type_heuristic(line: &str) -> Option<String> {
    let trimmed = line.trim();
    // AutoLang: let, var, const
    if trimmed.starts_with("let ") || trimmed.starts_with("var ") || trimmed.starts_with("const ") {
        // ... 解析类型注解 ...
    }
    // legacy: mut
    if trimmed.starts_with("mut ") { ... }
    ...
}
```

**文件**: `crates/auto-lsp/src/completion.rs`（~line 395）, `hover_info.rs`（~line 251）, `goto_def.rs`（~line 309）

### 2.3 缓存 workspace state，避免每次查询重解析

**问题**: `build_workspace_state` 在每个 LSP 请求（completion, hover, goto_def 等 ~7 个入口）中重新解析所有文件 + 阻塞磁盘 I/O。无缓存。

**修复方案**: 在 `Backend` struct 中缓存 `WorkspaceState`，仅在 `did_change` / `did_open` 时失效：

```rust
struct Backend {
    // ...
    workspace_cache: Arc<Mutex<Option<WorkspaceState>>>,
    cache_uri: Arc<Mutex<Option<Url>>>,
}

impl Backend {
    fn get_workspace_state(&self, uri: &Url, content: &str) -> WorkspaceState {
        let mut cache = self.workspace_cache.lock().unwrap();
        if let Some(ref state) = *cache {
            if self.cache_uri.lock().unwrap().as_ref() == Some(uri) {
                return state.clone(); // 命中缓存
            }
        }
        let state = build_workspace_state(uri, content);
        *cache = Some(state.clone());
        *self.cache_uri.lock().unwrap() = Some(uri.clone());
        state
    }
}
```

在 `did_change` handler 中调用 `workspace_cache.lock().unwrap().take()` 使缓存失效。

**文件**:
- `crates/auto-lsp/src/backend.rs`（添加 workspace_cache 字段 + get_workspace_state 方法）
- `crates/auto-lsp/src/workspace.rs`（不改动，但所有调用方改用缓存版本）

### 2.4 清理死代码和冗余

- 删除 `backend.rs:822-826` 的 `publish_diagnostics` deprecated 方法
- 合并 `hover_info.rs` 中的 `get_typestore_docs_direct` 和 `get_typestore_docs`（两者仅参数类型不同）
- 统一 `backend.rs:544` 和 `backend.rs:858` 的 `FragKind → SymbolKind` 映射到单独函数

---

## Phase 3: MCP 核心功能修复（P0 — 状态追踪 + 并发）

### 3.1 修复 execute_action_on_shared 返回空 state_changes

**问题**: `mcp_server.rs:1210` 的 `execute_action_on_shared` 始终返回 `state_changes: vec![]`，Plan 299 Phase 3.4 的状态追踪目标未达成。

**修复方案**: 在 `execute_action_on_shared` 中实现 `wait_for_state_changes` 调用（已有此函数但未在此路径使用）：

```rust
fn execute_action_on_shared(...) -> ActionResult {
    let before_state = {
        let shared = shared_handle.lock().unwrap();
        shared.state.clone()
    };
    // 执行 action
    shared_handle.send_action(msg)?;
    // 等待状态变化（最多 500ms）
    let state_changes = wait_for_state_changes(shared_handle, &before_state, 500);
    ActionResult { state_changes, ... }
}
```

**文件**: `crates/auto-lang/src/ui/mcp_server.rs`（~line 1200-1215）

### 3.2 修复 tool_type 双重构建 + 竞态

**问题**: `tool_type` 构建两次完整 SnapshotBuilder（clear + type）。clear 后 iced 可能重渲染，导致第二次 build 使用过期的 element_id。

**修复方案**: 合并为一次 snapshot build + 两次 action 分发：

```rust
fn tool_type(...) {
    let snapshot = SnapshotBuilder::build(&shared); // 只构建一次
    let input = find_first_input(&snapshot.root, element_id)?;

    // 在同一个锁周期内发送 clear + type_text
    let mut shared = shared_handle.lock().unwrap();
    if clear_first {
        shared.send_action(ActionMessage { event: "clear".into(), ... });
    }
    shared.send_action(ActionMessage { event: "type_text".into(), value: text.clone(), ... });
    drop(shared);

    // 等待状态变化
    let state_changes = wait_for_state_changes(shared_handle, &before_state, 500);
    ...
}
```

**文件**: `crates/auto-lang/src/ui/mcp_server.rs`（~line 929-988）

### 3.3 MCP 并发模型改进（P1）

**问题**: `new_current_thread` + `std::thread::sleep` 导致所有请求串行。

**修复方案**: 两步走：
1. **短期**（不改 runtime）：将 `tool_wait` 和 `wait_for_state_changes` 的 sleep 改为 `std::sync::Condvar::wait_timeout`，让 iced 线程在状态变化时 `notify`。
2. **中期**：切换到 `new_multi_thread` runtime，将 `tool_wait` 改为 async + `tokio::sync::watch`。

**文件**: `crates/auto-lang/src/ui/mcp_server.rs`

### 3.4 tool_keyboard 添加执行验证

**问题**: `tool_keyboard` 发送 `key_{name}` 事件后返回成功，但不验证 iced 是否实际处理。

**修复方案**: 检查 `send_action` 返回值。如果 iced 返回 "unhandled" 或 "unknown event"，返回错误而非成功：

```rust
let result = shared.send_action(msg);
if result.contains("unknown") || result.contains("unhandled") {
    return error_result(format!("Key '{}' not handled by application", key));
}
```

**文件**: `crates/auto-lang/src/ui/mcp_server.rs`（~line 992-1019）

---

## Phase 4: MCP 代码简化（P1 — 消除 AuraNode 遍历重复）

### 4.1 提取 AuraNode walker

**问题**: `collect_issues`, `count_elements`, `find_first_input`, `find_aura_node` 四个函数各自实现相同的递归 AuraNode 遍历。

**修复方案**: 创建共享 walker：

```rust
fn walk_aura_nodes<F>(node: &AuraNode, visitor: &mut F)
where F: FnMut(&AuraNode) -> bool { // true = continue
    if !visitor(node) { return; }
    match node {
        AuraNode::Element { children, .. } => {
            for child in children { walk_aura_nodes(child, visitor); }
        }
        AuraNode::ForLoop { body, .. } => {
            for child in body { walk_aura_nodes(child, visitor); }
        }
        AuraNode::Conditional { then_body, else_body, .. } => {
            for child in then_body { walk_aura_nodes(child, visitor); }
            if let Some(else_body) = else_body {
                for child in else_body { walk_aura_nodes(child, visitor); }
            }
        }
        _ => {}
    }
}
```

**文件**: `crates/auto-lang/src/ui/mcp_server.rs`（重构 4 个函数使用 walker）

### 4.2 消除 format_auto_val 重复

**问题**: `mcp_server.rs:1064` 的 `format_auto_val` 与 `mcp_types.rs:254` 的 `format_value` 完全重复。

**修复方案**: 删除 `mcp_server.rs` 中的 `format_auto_val`，改为调用 `mcp_types::format_value`（已 `pub` 且已 import）。

**文件**: `crates/auto-lang/src/ui/mcp_server.rs`（删除 ~line 1064 的重复函数）

### 4.3 MCP tool annotations 模板化

**问题**: 9 个工具定义中 6 个有相同的 annotations JSON 块。

**修复方案**: 提取 `fn tool_annotations(read_only: bool) -> serde_json::Value` 辅助函数。

**文件**: `crates/auto-lang/src/ui/mcp_server.rs`（~line 303-531）

---

## Phase 5: 完整性补齐（P2 — 计划范围缺口）

### 5.1 Plan 244: 补齐 18 个缺失的 Tour 示例

**缺失分布**:

| 章节 | 当前 | 目标 | 缺失 |
|------|------|------|------|
| ch02-types | 5 | 9 | -4（元组、可空类型、类型别名、基本类型独立示例） |
| ch03-functions | 4 | 7 | -3（默认参数、泛型函数、闭包捕获） |
| ch04-control | 5 | 7 | -2（for-cond 条件循环、if 表达式） |
| ch05-patterns | 4 | 7 | -3（is 基础、is 解构、is 类型守卫） |
| ch06-errors | 4 | 6 | -2（Result 完整流程、?? 空值合并） |
| ch07-collections | 5 | 8 | -3（数组与切片、迭代器适配器、对象字面量） |
| ch08-methods | 5 | 6 | -1（方法链 builder pattern） |

**执行**: 按章节补齐，每个示例遵循已有格式（.at 文件 + 注释说明）。

**文件**: `docs/tour/ch02-types/` ~ `docs/tour/ch08-methods/`（新建 18 个 .at 文件）

### 5.2 Plan 245: 评估 TAPL Cookbook 集成是否仍需实施

Plan 245 完全未实施（0/20 listings）。需要决策：

**选项 A**: 按原计划实施 — 在 TAPL ch04/ch09/ch14/ch21/ch22 添加 ~20 个 cookbook 衍生的 listing
**选项 B**: 合并到 Tour — 将这些内容融入 Plan 244 的 Tour 示例，TAPL 书暂不复用 cookbook
**选项 C**: 推迟 — 标记为 "未来工作"，当前聚焦 Phase 1-3 修复

**建议**: 选项 C（推迟），优先修复已实施计划的质量问题。

### 5.3 Plan 240: 对去桩化的 B-tier 文件添加真实 assert

当前 45 个 stub 文件使用 `assert(true)` 空断言。对已去桩化的文件（~33 个）应替换为验证实际结果的 assert。

**优先处理**:
1. `cryptography/002_pbkdf2.at` — 已有真实 sha2 API，但 assert 为空
2. `cryptography/003_hmac.at` — 同上
3. `encoding/004_base64.at` — 已有真实 base64 API
4. `concurrency/004_crossbeam_spsc.at` — 已有真实 channel API

**文件**: `test/cookbook/` 下的 33 个已去桩化 .at 文件

---

## 执行优先级和依赖关系

```
Phase 1 (P0) ── LSP 正确性修复
  1.1 UTF-16/字节偏移    ← 独立，可立即开始
  1.2 CRLF 偏移          ← 依赖 1.1 的 position.rs
  1.3 警告定位            ← 独立

Phase 2 (P1) ── LSP 架构清理
  2.1 共享工具模块        ← 依赖 Phase 1.1 的 position.rs
  2.2 var 关键字支持      ← 依赖 2.1
  2.3 workspace 缓存      ← 独立
  2.4 死代码清理          ← 独立

Phase 3 (P0/P1) ── MCP 核心修复
  3.1 state_changes 修复  ← P0，独立
  3.2 tool_type 竞态      ← 依赖 3.1
  3.3 并发模型改进        ← P1，独立
  3.4 keyboard 验证       ← 独立

Phase 4 (P1) ── MCP 简化
  4.1 AuraNode walker     ← 独立
  4.2 format_auto_val     ← 独立
  4.3 annotations 模板    ← 独立

Phase 5 (P2) ── 完整性补齐
  5.1 Tour 补齐           ← 独立
  5.2 TAPL 决策           ← 需讨论
  5.3 Cookbook assert     ← 独立
```

## 验证

每个 Phase 完成后运行：

```bash
# Phase 1-2 验证
cargo build -p auto-lsp
cargo test -p auto-lsp

# Phase 3-4 验证
cargo build -p auto-lang
cargo test -p auto-lang

# 全局回归
cargo build
cargo test
```

Phase 1 额外验证：创建含中文的测试文件 + CRLF 换行文件，手动测试 LSP。

## 收尾：更新原始计划状态

本计划全部修复实施完毕后，必须回到各原始计划文件更新状态，反映审计修复带来的变化：

| 原始计划 | 需更新的内容 |
|----------|-------------|
| `docs/plans/240-rust-cookbook-a2r-tests.md` | 更新 §13.1 审计发现的 stub/assert 问题状态；更新 §14 Phase 14 进度（Phase 5.3 完成后标记 B-tier assert 完成）；更新测试通过率 |
| `docs/plans/243-lsp-vscode-modernization.md` | 更新 Phase 2 状态（regex fallback → compiler-native 迁移进度）；添加 "审计修复" 备注（UTF-16 偏移、CRLF、var 关键字等） |
| `docs/plans/244-auto-lang-tour.md` | 更新示例数量（59 → 77，Phase 5.1 完成后）；更新每章示例统计表 |
| `docs/plans/245-tapl-cookbook-integration.md` | 明确标注实施决策（选项 A/B/C），如果推迟则标记为 "暂停/合并到未来计划" |
| `docs/plans/299-autoui-mcp-v2.md` | 更新 Phase 3.4 状态（state_changes 修复）；更新工具状态（keyboard 验证）；标注并发模型改进 |

每个原始计划文件的更新格式：

```markdown
## 审计修复（Plan 277）

> 2026-06-12 审计发现并于 Plan 277 中修复：

- ✅ [问题描述] — 已通过 Plan 277 Phase X.Y 修复
- ⏸️ [问题描述] — 推迟，原因: ...
```

## 提交计划

1. `fix(lsp): UTF-16/byte offset conversion for non-ASCII text (Plan 277 Phase 1.1)`
2. `fix(lsp): CRLF-aware position_to_offset for Windows (Plan 277 Phase 1.2)`
3. `fix(lsp): warning diagnostic span extraction (Plan 277 Phase 1.3)`
4. `refactor(lsp): extract shared position utils, eliminate 5x duplication (Plan 277 Phase 2.1)`
5. `fix(lsp): add 'var' keyword to type heuristic (Plan 277 Phase 2.2)`
6. `perf(lsp): cache workspace state between queries (Plan 277 Phase 2.3)`
7. `fix(mcp): return real state_changes from execute_action_on_shared (Plan 277 Phase 3.1)`
8. `fix(mcp): eliminate tool_type double-build and race condition (Plan 277 Phase 3.2)`
9. `refactor(mcp): extract AuraNode walker, eliminate duplication (Plan 277 Phase 4.1)`
10. `feat(tour): add 18 missing examples for ch02-ch08 (Plan 277 Phase 5.1)`
11. `docs: update Plans 240/243/244/245/299 with audit fix status (Plan 277 收尾)`
