# Plan 449: 041-auto-edit 组件化重构——单文件拆多文件（store + 013 式组件）

> **状态**: ✅ 已实施并合并（2026-08-26，worktree .worktree/plan-449 →
> master）。desktop_mcp.py 48/1——与重构前的纯原始树（git HEAD + 全新构建）
> 完全一致，零回归；唯一 FAIL 为 T10「配置热重载」，系 master 上与本重构
> 无关的既有 flake（§6.3）。
> **来源**: 用户示例走查——041 全部 486 行集中在 src/front/app.at 单 widget，
> 无组件级抽象；期望对齐 015-notes 的多文件工程风格。
> **基线**: master 8491c7a71（含 plan-044/448/008 已提交内容）
> **性质**: 示例重构 + vm 运行时能力实测定性（不改 crates 代码）。

## 1. 背景：三个调查结论（用户三问）

1. **action/keybinding/menu 映射机制**：`pac.at` 的 `ui_config: "auto-edit.at"`
   → `auto run` 注入 `AUTO_VM_ACTION_CONFIG`（`crates/auto/src/main.rs:953`）
   → iced 渲染器懒加载（`ui/action_config.rs`）→ 三个消费点：
   键盘回退层（`iced/renderer.rs` 配置回退，DSL onkeydown 之下）、
   `menubar{}`/`toolbar{}` 配置合成渲染（`aura_view_builder.rs`）、
   MCP 自动化同源派发（`ui/mcp_server.rs`）。
   `handler: ".ActXxx"` 派发进 VM 根 widget 的 `on{}`——Action=声明层，Event=执行层。
2. **vue 模式不支持该配置**：`ui_gen/vue.rs` 对 action_config 零引用；
   `menubar{}` 只生成空壳、`toolbar` 无映射、快捷键只剩 DSL 每控件 onkeydown。
   独立编译器特性缺口，后续另立计划。
3. **auto-edit.at 位置：保持根目录**（与 pac.at 同级，Plan 418 约定）。
   src/front/*.at 会被 vue 构建当组件源收集（`cmd_vue.rs:120`）、auto test
   递归收集 .at（`main.rs:986`）；文件名 stem=app id 被 OS 键位层
   （`%APPDATA%/auto/keymaps/auto-edit.at`）与 MCP 热更测试（`desktop_mcp.py:614`
   硬编码路径）依赖。README/auto-edit.at 头注释已澄清归属。

## 2. 终态结构（617 行 vs 原 486 单文件）

```
src/front/
├── app.at            142 行：use 四模块 + view 组合 + on 薄委托；
│                     测试定位锚交互（tab 条/确认弹层/编辑区/空态）留根
├── editor_store.at   342 行：store EditorStore——全部状态+业务逻辑；
│                     RemoveAt/SyncCursor 两个 helper msg 消除三处大重复
├── status_bar.at      53 行：widget StatusBar（无 props，读 .store，直调 store）
├── console_panel.at   41 行：widget ConsolePanel（同上）
└── ctx_menu.at        39 行：widget EditorCtxMenu（坐标锚态直读 .store）
```

验收 grep：`code_editor_cursor_line` 与 `.tabs.remove` 各仅 1 处
（分别收口在 SyncCursor/RemoveAt）✓；`auto build` exit 0 ✓。

## 3. 原设计 → 实际落地的偏差与原因（vm 能力实测定性）

原设计为 015 式「props 下传 + 回调上抛」五组件。实施中逐项实测发现 vm
运行时边界，**全部用最小工程（/tmp/vmtest）或 041 实机二分定位**：

| # | 发现 | 证据 | 应对 |
|---|---|---|---|
| 3.1 | **回调 props（on_xxx: msg）使组件整体退化为空 fallback** | 最小工程：`Child(label)` 正常渲染；签名加 `on_ping: msg` 即 fallback（声明即可破坏，无需调用）；015-notes 的 NavTree/EditorPanel 在 vm 下实测同为 fallback（015 是 vue 示例，vm 从未真正跑通组件） | 组件一律无 props（013 式：读 `.store` + 自有 msg + 直调 store） |
| 3.2 | **组件子树对 MCP 快照不可见**（渲染/派发正常，快照只走根模板） | 最小工程：组件内按钮/文本在 autoui_snapshot 中不存在；`aura_view_builder.rs` 审计注释佐证（013 B12(iii)） | 测试靠快照定位的交互（x 按钮形状、"直接关闭"文本、`onclick: .ActOpen`）留根视图 |
| 3.3 | **view fn 片段的参数化条件在 vm 不求值**（onclick 实参、条件样式、if 分支全丢） | TabItem 片段实测：渲染树中 onclick 属性/X 按钮/条件样式全部缺失（015 NoteItem 片段先例仅 vue 验证过） | tab 条回退基线双分支形态（`.store.` 读取本身可用——code_editor 的 `if t.key == .store.active_key` 正常渲染） |
| 3.4 | **`code_editor_set_text` 编译进 store handler 产出坏字节码**（`virt_memory.rs:402 read_i32` 越界 panic，索引 = 负数 u64 重解释；字面量参数亦崩；`fold_toggle` 等同族内建正常；widget handler 中正常） | 041 实机 handler 二分：空体✓ → 嵌套写✓ → 仅 set_text✗（Ctrl+N/菜单点击双路径稳定复现，backtrace 定位在字节码执行而非 native） | set_text 留 App 根 handler（`.ActNew`：`store.ActNew()` 后根 handler 调 set_text；基线路径已验证） |
| 3.5 | **根 handler → `store.X()` 调用、store 字段合并、`enabled-if` 求值、`for i, t in .store.tabs`、嵌套索引读写** | 038/013 先例 + 041 实测全通过（T1-T8/T11 全绿） | store 化成立——本重构的核心结构得以保留 |

误诊教训（记录备查）：调查早期多个"矛盾"读数源于 ① 探针快照在首帧
"tree:" 即断开（应等 "(rendered)"，合成 menubar 只在渲染树）；② 探针
proc.kill() 不杀子进程树，残留 auto.exe 孤儿服务旧状态（desktop_mcp.py
的 finally 用 taskkill /T /F 正是为此）。

## 4. 去重成绩（原 app.at 的三大重复）

- CloseTab/ConfirmClose 各持一份"remove+索引修补+激活重算" → `RemoveAt`
  单点（store handler 互调 `store.RemoveAt(...)`）。
- line/col/sel 三元组读回 5 处 → `SyncCursor(key)` 单点。
- `.console = console_lines()` 尾缀 21 处保持原样（VM view 不能调函数，
  镜像字段模式是 Plan 402 约束下的正解，强行收口无收益）。
- 组件内聚：状态栏（含分隔线与 terminal 双分支）、Console 面板整体、
  右键菜单三项同款按钮。

## 5. 行为保持

- 事件名全部原样保留（.ActNew~.ActAbout、.TabActivate、.CloseTab、.SrcChanged、
  .CursorMoved、.EditorCtx、.Confirm*）——action 配置三源派发面与 MCP
  断言面零变化。
- 唯一语义微调：ConfirmClose 关闭脏 tab 后的光标读回从「置 1:1」统一为
  RemoveAt 的「读回真实光标」（更正确；测试无此断言）。

## 6. 遗留与后续计划建议

1. **vm 组件渲染补全**（建议另立 plan）：回调 props 退化（3.1）、快照
   组件子树不可见（3.2）、片段参数化条件不求值（3.3）——修好后 041 可将
   tab 条/确认弹层也组件化（tab_bar.at/confirm_dialog.at 的设计已在本
   计划早期版本验证过语法，见 git 历史）。
2. **VM 字节码 bug**（3.4）：store handler + `code_editor_set_text`（疑
   及同族 set 类内建）编译路径产出越界读。根因在 handler_codegen/
   vm codegen 对 store decl 的合成，值得专项修复（041 目前以根 handler
   规避）。
3. **T10 配置热重载 flake（master 既有，与本重构无关）**：Plan 449 期间
   在纯原始树（git HEAD 状态）+ 全新构建下复现 48/1（reload 工具返回
   "15 actions, 5 menus (generation 2)" 解析成功，但 menubar 视图 10 秒
   内不重建）；当天早些时候同码曾 50/50 通过——时序敏感，疑与
   generation→view_dirty→重建链路（renderer.rs，plan-044 活跃领域）相关。
4. **vue 侧 action 配置支持**（§1.2）：vue codegen 接入全局 keydown +
   menubar/toolbar 合成，另立计划。

## 7. 验证记录（2026-08-26）

- `cargo build --features ui-iced --bin auto` ✓（两次，含全新构建）
- `desktop_mcp.py`：重构后最终树 48/1；纯原始树对照 48/1（同 FAIL 项
  T10）——零回归 ✓
- `auto build`（C/ninja 路径）exit 0 ✓
- grep 验收：`code_editor_cursor_line` 1 处 / `.tabs.remove` 1 处 ✓
- 最小工程（/tmp/vmtest）已清理；038 实机抽测（Reset 按钮行为正确）✓
