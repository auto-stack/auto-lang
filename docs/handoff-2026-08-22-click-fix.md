# 交接：418 P2-3 真实鼠标点击遮蔽问题（2026-08-22 晚）

> 新会话从这里接手。核心目标：**修通 auto-edit(041) 工具栏图标/菜单项/状态栏 console 图标的真实鼠标点击**。

## 当前状态
- **分支 4 个提交未合并回 master**：合并时 renderer.rs 与并行会话的 debug 提交(a0444f71/db9f06d1)整文件冲突,已中止留待处理——新会话先解决此合并(两侧改动均为局部,冲突系行尾/重写噪音,内容归一后对齐)
- worktree：`D:\autostack\auto-lang\.worktrees\auto-edit-ux`，分支 `plan-418-auto-edit-actions`，干净
- 计划文档：`docs/plans/418-auto-edit-actions-and-config.md`（Phase 1/2 主体已完成；§8.4 已知缺口；**§8.5 本问题排查记录**）
- MCP 自动化路径全绿（29/29）：`cd examples/ui/041-code-editor/tests && python desktop_mcp.py`
- 构建命令：`cargo build -p auto`（worktree 内），运行：`cd examples/ui/041-code-editor && auto.exe run .`

## 问题现象（已严格复现）
- **可用**：menubar「文件/编辑」按钮真实点击（菜单开合正常）；编辑器内点击/打字正常
- **不可用**：工具栏图标、菜单面板项、状态栏右下 console 图标——真实点击后 update() **零消息**
- MCP 派发（旁路订阅）一切正常 → 纯 iced 真实事件路由/遮蔽问题

## 复现方法论（踩过的坑都要避开）
1. 每个实例**实测窗口矩形**：PowerShell `SetProcessDPIAware()` + `GetWindowRect`（DPI=200%，物理=逻辑×2）
2. 点击前 `SetForegroundWindow` + 等待；单次点击可能被窗口激活吞掉，**双击**保底
3. 坐标换算：MCP 截图（2560×1600 物理）定位目标 → 逻辑坐标 ×2 + 窗口 origin
4. 插桩：`AUTO_DEBUG_MSGS=1` 运行可在 update 入口打消息流（探针当前已移除，需要时加回 `renderer.rs` update 闭包开头）

## 已排除（勿重复）
位置（左右分区）、fixed_both 包装分支、svg 内容（文本按钮同样死）、tooltip（禁用后同样死）、编辑器 Fill 高度越界（固定高度后同样死）

## 已修复（已提交，保留）
- code_editor widget 鼠标输入 `is_over(bounds)` 门控（之前吞全窗点击，CursorMain 洪流实证）
- layout `limits.max()` → `limits.resolve(Fill,Fill,max)`
- EE03 tooltip 加 300ms delay（delay=0 时 hover 即 invalidate_layout 打断点击）

## 剩余疑点（下一步从这里开始）
1. **状态栏/工具栏的 ml-auto Fill 容器**：`wrap_with_margin_top`（renderer.rs ~1293）给 ml-auto 元素包 `container.width(Fill)`——iced Container 理论不捕获鼠标，但该区域恰好是死区，需实测
2. **overlay hoist 层**（Plan 409 §10 续5，absolute/z-N 子节点 hoist 成 Stack）：菜单关闭时面板未渲染，但 hoist 机制可能有残留层
3. 建议手段：iced DevTools（F12）看层级与 bounds；或在 renderer 里 dump iced 用户树事件路由顺序；或给按钮臂临时加 mouse_area 探针
4. 关键文件：`crates/auto-lang/src/ui/iced/renderer.rs`（update 闭包 ~5714、按钮臂 ~2284）、`crates/auto-lang/src/ui/code_editor/iced/widget.rs`、`crates/auto-lang/src/ui/aura_view_builder.rs`（合成 menubar/toolbar ~3120-3320）

## 环境注意
- D 盘曾写满，worktree target 已清过一次 incremental（6.3G）；构建失败先查磁盘
- 常有并行会话在 master 上推进，开工前 `git merge master` 同步
- 残留 auto.exe 进程会锁 exe 导致构建失败：`taskkill //F //IM auto.exe`
- 主仓 `examples/rust-workspace/Cargo.toml` 的未提交改动属并行会话（plan-032 系），勿动
