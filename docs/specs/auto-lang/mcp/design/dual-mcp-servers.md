# 双 MCP Server 分工

## 范围

仓库里有两个互不相干的 MCP server。本文界定二者的职责边界与数据通道，
防止把 AutoUI 工具误记到本模块头上（`docs/design/14` 把两者写在同一节，
是主要混淆源）。

## 原则

- 贴近数据源：VM server 操作源代码，UI server 操作运行中的渲染树
  （architecture.md ADR-05）。
- 结构/布局/样式走 VTree 文本信道，像素走截图信道（ADR-06）。

## 对照

| 维度 | AutoVM MCP（本模块） | AutoUI MCP（ui 模块） |
|---|---|---|
| 代码 | `crates/auto-lang/src/mcp/` | `crates/auto-lang/src/ui/mcp_server.rs` + `ui/vtree_atom.rs` |
| 入口 | `auto mcp`（独立进程） | 嵌在 iced 桌面进程内，后台线程 |
| 传输 | stdio 行分隔 JSON-RPC | HTTP，`localhost:9247` |
| 操作对象 | `AutovmReplSession`（源代码/VM 状态） | `SharedState`（活体 VTree + 测量数据） |
| 工具前缀 | `auto_`（7 个） | `autoui_`（10 个：snapshot/inspect/action/check/screenshot/state/wait/type/keyboard/vtree） |
| 出身 | plan-265 | plan-278（框架）→ 279（snapshot）→ 280（render_check）→ 285（screenshot）→ 299（V2）→ 314（vtree） |

## AutoUI 侧关键机制（摘要，细节归 ui 模块 spec）

- **双信道**：`autoui_vtree`（Atom 文本，主信道）+ `autoui_screenshot`
  （PNG，像素级次信道）。`autoui_snapshot`（build-time AURA 模板 + 简单
  rect）仅保留向后兼容。
- **Atom schema**：每个渲染 VNode 1:1 一个 Atom 节点——name 是源码
  widget 关键字（`col`/`button`/…），id 是实例级 `vnode_<n>`；盒模型
  （`bbox`/`box`）、`style`、`class`、`events`、`source`、`for_iter`
  全部作为 **props** 挂在节点内部，children 拓扑严格等于 VTree 拓扑。
- **降级不变量**：任何字段未测量（如 rust 模式 bounds 未接入，
  plan-311 P2-B-3）即省略该 prop，节点仍输出，永不报错。
- **数据通道**：渲染器每帧把 `live_vtree` + `live_cache` 的可序列化
  快照（`StyledNodeSnapshot`/`ComputedNodeLite`）拷进 `SharedState`；
  捕获门控从"F12 开"解耦为"F12 开 **或** MCP 连接"（plan-314 Task 4/5）。
- **token 控制参数**：`scope`（按 `vnode_<n>` 取子树）、`depth`（折叠
  深层）、`include_box/style/events/source/props` 开关。

## 显式非目标

- 两 server 统一传输或合并进程：无人立项，且进程边界（agent 侧 vs
  被测 UI 进程内）决定合不了。
- 本文不维护 AutoUI 工具的完整契约——那是 ui 模块 spec 的职责；
  此处只记录与本模块的边界。

> 来源: `docs/design/14-developer-tools.md`、`docs/plans/old/278-autoui-mcp-desktop.md`、`docs/plans/archive/299-autoui-mcp-v2.md`、`docs/plans/archive/314-autoui-mcp-styled-vtree.md`、`crates/auto-lang/src/ui/mcp_server.rs`（工具注册表核对）
