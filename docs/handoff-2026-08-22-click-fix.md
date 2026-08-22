# 交接:418 P2-3 真实鼠标点击问题——已结案(2026-08-22 晚,第五会话)

> 本文档由第四会话的"待排查"更新为第五会话的**结案状态**。结论:真实点击路径没有应用层缺陷,可直接把分支合回 master。

## 分支状态(下一步就做这个)
- worktree:`D:\autostack\auto-lang\.worktrees\auto-edit-ux`,分支 `plan-418-auto-edit-actions`,**干净**
- cfcee3c6:已合并 master(原 renderer.rs 整文件冲突系行尾噪音——仓库 blob 历来 CRLF、分支侧已归一 LF;三版本 tr -d '\r' 后 git merge-file 零冲突,统一 LF 写回)
- 0fdf5400:收官提交(探针清理 + 矩阵哨兵修复 + §8.6 结案记录)
- **剩余动作:`git checkout master && git merge plan-418-auto-edit-actions`**(master 若有新提交,renderer.rs 冲突按同样方法处理:两侧 tr -d '\r' 归一后 merge-file,取 LF)

## 结案结论(详见 docs/plans/418-auto-edit-actions-and-config.md §8.6)
第四会话的"工具栏/菜单项/console 图标真实点击零消息"是三个因素叠加,**均非 iced 事件路由/遮蔽缺陷**:
1. **旧 200ms 心跳事件饿死**(主因)——master 29c3f93e 已改 2s+30s 活联门控,经本次合并带入分支。插桩证实 200ms 节拍下消息队列积压、press/release 配对被打断。
2. **坐标方法学漏了标题栏偏移**——"逻辑×2+窗口 origin"中 GetWindowRect 含标题栏而 iced 逻辑原点在客户区,系统性上偏 ~30 物理像素,h-7 图标整枚打偏。正确公式:物理 = 窗口 origin + 标题栏高 + 逻辑×2(或用 ScreenToClient)。
3. **锁屏输入干扰(实验室陷阱)**——工作站锁定时 LockScreenBackstopFrame(explorer)接管全部输入:WindowFromPoint 返回 explorer/LockApp、SetCursorPos 被弹回 (0,0) 即为特征。第五会话中段所有"零事件"皆此因。
- 解锁时段内实测:**ActNew、ActUndo 经真实 SendInput 成功触发,menubar 菜单开合正常**——当前构建真实点击路径是通的。
- ml-auto Fill 容器、overlay hoist(Stack/opaque/tooltip overlay)逐一实测排除。

## 修复的顺手 bug
- `tests/desktop_mcp.py` T1 哨兵 `code_editor`→`"(rendered)"`:模板快照(首渲染前 ~1.5s 窗口)含 code_editor 原始标签但无合成按钮,轮询恰在窗口内提前退出导致 4 项误报;修复后矩阵 **29/29**。
- 移除第四会话遗留的 [DBG-BTN] 探针(renderer.rs 按钮臂)。

## 回归基线(全部通过)
- MCP 矩阵 29/29:`cd examples/ui/041-code-editor/tests && python desktop_mcp.py`
- `cargo test -p auto-lang --lib --features ui-iced iced` 44/44;`action_config` 3/3;`mcp_server` 6/6
- 已知既有问题(勿归本次):全量 `--features ui-iced` 单测套件在 plan370 `d10_edit_fills_edit_fields` 栈溢出(深递归+调试栈,旧测试)

## 遗留(低优,不阻塞合并)
- 工具栏图标偶发近黑:svg::Style.color 的 base 在元素构建时捕获,主题色解析与首建时序竞态——纯视觉,复现随机
- §8.4 原有缺口不变:①合成子树 probe 路径对齐;③菜单项 checked 勾选态渲染

## 环境注意(传给下个会话)
- 并行会话常跑 `taskkill //F //IM auto.exe`(会误杀你的实例)并占用 9247 端口——自己的实例用 `AUTOUI_MCP_PORT` 指定独立端口,按命令行 `*auto-edit-ux*` 过滤清理
- 测真实点击前先确认工作站未锁(WindowFromPoint 检查);DPI=200%,GetWindowRect 含标题栏
- D 盘曾写满;残留 auto.exe 锁 exe 时先 `taskkill //F //IM auto.exe`
