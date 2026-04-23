# Plan 209: ac-examples 现代化 — 利用 Plan 200/201 特性重写

> 日期：2026-04-22
> 状态：Phase 0 ✅ 完成（33/33 PASS），Phase 1-6 待实施（低优先级美化）
> 前置：Plan 200（VM 缺失特性）✅ 已全部完成、Plan 201（四大核心能力）✅ 已全部完成
> 范围：将 `auto-code-rs/crates/ac-examples/src/` 中的 33 个示例重写为利用新特性的惯用 Auto 代码

## 背景

Plan 200 和 Plan 201 已为 AutoVM 添加了大量新能力（枚举多字段变体、闭包 HOF、Result 系统、spec vtable 分发、if-let、元组、范围切片等）。但 ac-examples 中的 `main.at` 文件大多仍使用旧式 workaround，且 **21/33 个示例当前无法通过 auto main.at 运行**。

## 基线测试结果（2026-04-22）

```
PASS: 01, 02, 04, 06, 07, 08, 09, 10, 12, 13
FAIL: 03, 05, 11, 14-33
```

### 错误分类

| 错误类型 | 影响示例 | 原因 |
|---------|---------|------|
| `auto_syntax_E0007` (Expected term, got RBrace) | 14-26, 30-32 | 语法不兼容 |
| `auto_syntax_E0099` | 11, 15-17, 21-22, 26, 30 | 语法不兼容 |
| `auto_lexer_E0001/E0002` | 11, 21, 26 | 词法错误 |
| `RuntimeError("Invalid opcode")` | 03 | VM opcode 不支持 |
| `RuntimeError("Invalid instance ID")` | 05 | 枚举实例问题 |
| `Module not found: fs` | 27, 28, 33 | 缺少 fs 模块 |
| `Undefined variable: List` | 29 | 类型引用错误 |

## 实施策略

先修复所有失败的示例（Phase 0），然后逐批引入新特性。

---

## Phase 0: 修复所有失败示例（21 个）

逐个修复语法/运行时错误，使所有 33 个示例都能 PASS。不引入新特性，只修复兼容性问题。

### 0A: 语法修复（`E0007`, `E0099`, 词法错误）

需要逐个阅读失败示例，定位语法问题并修复。常见原因：
- 枚举字面量值赋值语法（`= 10`）
- Result 类型 `!str` 语法
- `mut fn` / `ext` 块内方法语法
- 闭包语法 `=>`

**涉及示例**：11, 14-26, 30-32

### 0B: 运行时修复

- **03**: `Invalid opcode` — 可能是浮点运算或特定操作码问题
- **05**: `Invalid instance ID: 10` — 枚举整数值作为实例 ID 问题
- **27, 28, 33**: `Module not found: fs` — 需要添加 fs 模块 stub 或改写为不依赖 fs

### 0C: 类型修复

- **29**: `Undefined variable: List` — 可能需要 `list` 或其他引用方式

---

## Phase 1: 枚举类型重写（影响 11 个示例）

将 `kind: str` 判别器 + `if/else if` 链替换为真正的 `enum` + `is` 模式匹配。

| 示例 | 当前 workaround | 重写目标 |
|------|----------------|---------|
| 05_permission_check | `mode int` (1/2/3) | `enum PermissionMode { Allow Ask ReadOnly }` |
| 10_api_error_enum | `kind: str` + if/else chain | `enum ApiError { Http(str) Json(str) Api{status message retryable} ... }` |
| 12_stream_event_types | `kind: str` 判别器 | `enum StreamEvent` + `enum ContentBlockDelta` |
| 13_tool_trait_def | `ToolError { kind: str, msg: str }` | `enum ToolError { ExecutionFailed(str) InvalidInput(str) }` |
| 18_command_safety_check | `type CheckError { DangerousPattern str }` | `enum CheckError { DangerousPattern(str) }` |
| 22, 24, 25, 29, 31, 32 | 类似 workaround | 统一用 enum + `is` |

---

## Phase 2: 闭包 HOF 链式调用（6 个示例）

将手动 `for` 循环替换为 `.map()` / `.filter()` / `.reduce()` / `.find()` 链。

| 示例 | 重写目标 |
|------|---------|
| 04_token_estimate | `.flat_map().map().sum()` |
| 08_usage_struct | `.map().sum()` |
| 17_context_compaction | `.map().sum()` |
| 20_tool_registry | `.map()` 构建 definition list |
| 32_stream_event_agg | `.find()` 查找 pending tool |

---

## Phase 3: Result 系统规范化（4 个示例）

统一使用 `!T` Result 类型 + `Ok`/`Err` + 模式匹配。

| 示例 | 当前 | 目标 |
|------|------|------|
| 13_tool_trait_def | `ToolResult { ok: bool }` | `execute() !str` |
| 18_command_safety_check | 部分使用 Result | 完整 `!()` + `is_ok`/`is_err` |
| 19_exact_match_edit | 已使用 Result | 清理 |
| 31_tool_exec_with_perm | 手动 ok/err | `!str` Result |

---

## Phase 4: Spec 动态分发（3 个示例）

用 `spec Tool` + `ext EchoTool has Tool` 替代字符串分发。

| 示例 | 当前 | 目标 |
|------|------|------|
| 13_tool_trait_def | `if tool.name == "Echo"` | `spec Tool` + `ext EchoTool has Tool` |
| 20_tool_registry | `Map<str, Tool>` 具体类型 | spec dispatch |
| 31_tool_exec_with_perm | 字符串分发 | spec + permission |

---

## Phase 5: 命名统一（~20 个示例）

| 旧名 | 新名 |
|------|------|
| `.has()` | `.contains()` |
| `.append()` | `.push()` |
| `.to_lower()` | `.to_lowercase()` |

---

## Phase 6A: JSON 改进（4 个示例）

将字符串拼接 JSON 改为结构化 Map + `json.stringify()`。

涉及：11, 21, 22, 27

---

## 不在本 Plan 范围

| 特性 | 影响示例 | 原因 |
|------|---------|------|
| 位移运算符 | 01, 03 | 需语言层实现 |
| `std.time`/`Duration` | 03 | 需新模块 |
| `std.fmt` 格式化 | 01, 06 | 需语言层实现 |
| `impl Display/Error` | 10, 13, 18 | 手写已够用 |
| `#[derive(...)]` | 全部 | 需语言层实现 |

## 实施顺序

0 → 1 → 2 → 3 → 4 → 5 → 6A
每个 Phase 完成后提交一次。

## 验证

```bash
cd auto-code-rs/crates/ac-examples/src/XX_example_name
auto main.at    # 确认所有 assert 通过
```
