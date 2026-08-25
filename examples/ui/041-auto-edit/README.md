# 041-auto-edit — AutoLang 文本编辑器（多文件组件化）

一个"严肃应用"轨道的 AutoUI 示例：多 tab 代码编辑器（原生 CodeMirror6 内核），
带菜单栏/工具栏/全局快捷键三源动作触发、右键菜单、脏 tab 关闭确认、Console
面板、撤销重做与文件打开/保存。Plans 413/414/418/420/422/428 持续迭代，
Plan 449 完成组件化重构（单文件 486 行 → 五文件工程）。

## Concepts

- **store 承载全部状态与业务逻辑（038/013 形态）** —— `editor_store.at` 持有
  tabs/光标/Console/弹层等全部状态；App 的 `on` 薄委托 `store.Xxx()`，事件名
  与 `auto-edit.at` 的 handler 映射一一对应（Action=声明层，Event=执行层）。
  重复逻辑收口为两个内部 helper msg：`RemoveAt`（关闭 tab 的 remove+索引修补+
  激活重算，原先 CloseTab/ConfirmClose 各持一份）、`SyncCursor`（line/col/sel
  三元组读回，原先 5 处重复）。store handler 间经 `store.Xxx()` 互调（038
  先例）。派生标量仍由 handler 就地重算——VM view 不能调函数（Plan 402
  约束），模板只读字段。
- **013 式组件（无 props + 直调 store）** —— `StatusBar`/`ConsolePanel`/
  `EditorCtxMenu` 不带 props：标量/锚态经 `use editor_store` 直读 `.store.*`，
  自有 msg，handler 直调 `store.Xxx()`（013-todo 的 TodoList 形态）。
- **vm 组件边界（Plan 449 实测定性，重构设计的硬约束）**：
  1. **回调 props（`on_xxx: msg`）会使组件整体退化为空 fallback**——
     015-notes 的 NavTree/EditorPanel 在 vm 模式下即如此（015 是 vue 示例）。
  2. **组件子树对 MCP 快照不可见**（渲染正常、handler 派发正常，但
     `autoui_snapshot` 只走根模板）——测试矩阵靠快照定位的交互不能进组件。
  3. **view fn 片段的参数化条件（onclick 实参/条件样式/if 分支）在 vm
     视图构建中不求值**——片段去重 tab 条不可行（015 的 NoteItem 片段
     先例只在 vue 模式验证过）。
  因此 tab 条（x 按钮形状定位、"+" 的 `onclick: .ActOpen`）、确认弹层
  （"直接关闭" 文本定位）、code_editor 绑定与空态文本留在 App 根视图。
- **vm 字节码 bug 规避**：`code_editor_set_text` 编译进 store handler 会产出
  坏字节码（`virt_memory.rs read_i32` 越界 panic，Plan 449 实测；widget
  handler 中正常）——该调用留在 App 根 handler（`.ActNew` 内，见 app.at）。
- **动作三源绑定（Plan 418/423/451）** —— action 注册表 + menubar/toolbar
  结构以 `widget App` 内的 **`actions {}` 声明块** 表达（Plan 451 起 DSL 化，
  原项目根的 auto-edit.at 外挂配置退役；外挂文件形态仍兼容——DSL 优先）。
  vm 渲染器经 `action_config()` 消费：① 键盘回退层（DSL onkeydown 之下）；
  ② `menubar{}`/`toolbar{}` 占位的配置合成渲染；③ MCP 自动化同源派发。
  `handler: .ActXxx` 在**解析期校验**命中本 widget 的 `on{}` 事件；
  `enabled_if`/`checked_if` 为条件表达式字符串，对合并根 state 求值
  （`.tab_count` 等，无前缀）。热重载：改 app.at 后经 MCP 的
  `action_config_reload` 工具（或渲染器 mtime 轮询）重读源文件重新提取。
- **vue 模式限制** —— vue 生成器尚未接入 action 配置：快捷键全部失效、
  `menubar{}` 生成空壳、`toolbar` 无映射（vue 下只剩 DSL 每控件
  `onkeydown`）。本示例 pac.at 固定 `render: "vm"`。vue 侧支持是独立的
  编译器特性，另行计划跟踪。

## Source

| 文件 | 职责 |
|---|---|
| `src/front/app.at` 的 `actions {}` 块 | 动作注册表（14 个 action）+ menubar/toolbar 结构（Plan 451 DSL 化；原根目录 auto-edit.at 已删除） |
| `src/front/app.at` | 根 widget `App`（213 行）：actions 声明块 + view 组合 + on 薄委托；测试定位锚交互（tab 条/确认弹层/编辑区/空态）留根 |
| `src/front/editor_store.at` | `store EditorStore`（342 行）：全部状态 + 业务逻辑 + RemoveAt/SyncCursor 收口 |
| `src/front/status_bar.at` | `StatusBar` 组件（读 `.store` 标量；`ToggleConsole → store.ActConsole()`） |
| `src/front/console_panel.at` | `ConsolePanel` 组件（`Clear → store.ConsoleClear()`） |
| `src/front/ctx_menu.at` | `EditorCtxMenu` 组件（坐标锚态直读 `.store`；`Cut/Copy/SelectAll/Dismiss → store.Ctx*()`） |

## How to Run

```
cd examples/ui/041-auto-edit
auto run               # pac.at 默认 render: "vm" —— AutoVM + iced 原生窗口
auto run --render vm   # 同上（显式）
auto build             # C/ninja 移植路径（windows_ninja port）
```

需要先构建工具链：`cargo build --features ui-iced --bin auto`（仓库根）。

## Tests

```
cd examples/ui/041-auto-edit/tests
python desktop_mcp.py   # MCP 桌面动作矩阵：三源触发/tab 工作区/热重载/OS 键位层
```

前置：`pip install requests`；可选 `AUTO_BIN` 指向 auto 可执行文件；
`AUTO_OPEN_PATH`/`AUTO_SAVE_PATH` 环境变量旁路阻塞式文件对话框（不设则跳过
T9/T10 分组）。

注：Plan 451 起 T10「热重载」走 DSL 源路径（reload 工具或 mtime 轮询重读
app.at 重新提取 actions + generation bump → 视图重建），实测 50/0 全绿。
OS 用户键位层（`%APPDATA%/auto/keymaps/auto-edit.at`）保持外部文件——那是
用户偏好覆盖而非 app 代码；app id 由 pac.at 的 `name`（经 `auto run` 注入
`AUTO_APP_ID`）提供。
