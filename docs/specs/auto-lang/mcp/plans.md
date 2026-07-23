# mcp 相关 plan 索引

> 状态以 plan 文件自身为准；归档列为当前所在目录。
> 265/269 是本模块（AutoVM MCP）直系；278 起均为 AutoUI MCP
> （`ui/mcp_server.rs`，归 ui 模块），因共享"MCP"主题与 14 章出处一并收录。

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 265 | autovm-mcp-server | ✅ Complete | old/ | 本模块奠基：stdio JSON-RPC + 会话管理 + 7 工具（define 合入 evaluate，sandbox/GC/建议诊断未落地） |
| 269 | autovm-daemon-cli | ✅ Done | old/ | `auto serve`/`auto req` 命名管道 daemon，复用本模块 SessionManager 实现跨进程会话 |
| 278 | autoui-mcp-desktop | ✅ 已完成（Phase 1） | old/ | 第二个 MCP server 嵌入 iced 进程（HTTP :9247，SharedState），确立双 server 分工 |
| 279 | aura-style-mcp-snapshot | ✅ Completed | old/ | `autoui_snapshot`：build-time AURA 模板快照（后被 vtree 取代为主信道） |
| 280 | mcp-render-check | ✅ 已完成 | old/ | `autoui_render_check` 渲染诊断（FALLBACK/PARTIAL 标注） |
| 285 | screenshot-mcp-tool | ✅ Implemented | old/ | `autoui_screenshot` 像素级次信道 |
| 299 | autoui-mcp-v2 | ✅ 已完成（Phase 1-3） | archive/ | AutoUI MCP V2 协议全面改进 |
| 314 | autoui-mcp-styled-vtree | ✅ COMPLETED | archive/ | `autoui_vtree`：实时 VTree→Atom（1:1 + 盒模型），解耦 F12 门控，确立主感知信道 |
