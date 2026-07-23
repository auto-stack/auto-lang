# 工具集契约

## 范围

`mcp/server.rs` 注册的 7 个 MCP 工具的行为契约，以及 `auto_patch` 的
文本替换算法。描述的是**代码现状**，不是 plan-265 的原始设计
（差异见文末与 `../overview.md` 已知坑）。

## 原则

- 所有工具结果都是"text 块里装 JSON"（见 `protocol-stdio.md`）。
- 缺参数/会话不存在 → `ToolResult::error`；执行失败 → 正常结果里
  `status: "error"` + `diagnostics`，两层错误通道不混用。
- 诊断结构当前只有 `{severity, message}` 两个字段。

## 工具清单

| 工具 | 必填参数 | 可选参数 | 行为 |
|---|---|---|---|
| `auto_session_create` | — | `sandbox: bool` | 建会话，返回 `{session_id, status:"created"}`；sandbox 仅存不生效 |
| `auto_evaluate` | `session_id`, `code` | — | `session.run(code)`；成功返回 `{status:"ok", output, value, diagnostics:[]}` 并 `append_source`；失败返回 `status:"error"` + 诊断，**不**入历史。`value` 取 `format_last_result()` 或 `get_last_result()` |
| `auto_session_reset` | `session_id` | `action: "reset"\|"delete"` | reset 清 VM 状态（留 history）；delete 删会话 |
| `auto_inspect` | `session_id` | `kind: "functions"\|"variables"\|"all"` | 返回 `stats{bytecode_size, heap_objects, arrays}` + 函数名/局部变量名清单（只有名字，无签名无值） |
| `auto_typecheck` | `code` | — | **无会话**，只 `parse_preserve_error`：成功返回 `valid:true` + 顶层符号清单（fn 参数数/返回类型、type 字段数、enum variant 数）+ `use` 导入列表；失败 `valid:false` + 诊断。不做类型推断 |
| `auto_patch` | `session_id`, `old_name`, `new_code` | — | 见下节算法；重建成功返回 `status:"ok"` + 新会话的 output |
| `auto_snapshot` | `session_id` | — | 返回 `{source, lines}`，即 `source_history` 的 `"\n\n"` 拼接；空会话返回空串 + 提示 |

## auto_patch 替换算法（server.rs:patch_replace_definition）

1. 逐行扫描，找行首（trim 后）匹配 `fn|type|enum|spec|ext <old_name>`
   且名字后跟词边界（`(`/` `/`{`/`<` 或行尾）的行，作为定义起点。
2. 找不到 → 把 `new_code` 追加到源码末尾（视为新增定义）。
3. 找到后从起点数花括号：`{` +1、`}` -1，深度归零的行为定义终点；
   单行无括号定义（如 `type Alias = X`）以下一定义起点为界。
4. 拼接"起点前 + new_code + 终点后"，先 `parse_preserve_error` 验证，
   语法不过则原样报错、不动会话。
5. `rebuild_with_source` 换新会话，再 `session.run(全量源码)` 重跑；
   重跑失败返回 `status:"error"`（此时会话已是新源码重建后的状态）。

**已知局限**：纯文本算法不解析 AST——注释/字符串里的花括号会干扰定界；
`reset` 保留 history 而 `patch` 重建 history，两条路径对"会话当前源码"
的理解不一致，混用时 snapshot 结果可能出乎意料。

## 显式非目标

- plan-265 设计中的 `auto_define` 独立工具：未实现，define 语义合入
  `auto_evaluate`（顶层定义即执行即生效）。
- `docs/design/14` 列出的 `execute`/`type_check` 之外的
  `explain`/`suggest`/`format`/`test`/`doc`/`self_describe` 工具：
  均无对应实现。
- 类型推断查询（plan-265 Decision 4 的 infer 集成）：未落地。
- 诊断里的 `code`/`span`/`suggestions`（plan-265 统一诊断 schema）：
  未实现。

> 来源: `crates/auto-lang/src/mcp/server.rs`、`docs/plans/old/265-autovm-mcp-server.md`、`docs/design/14-developer-tools.md`
