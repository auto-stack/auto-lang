# Plan 451: actions 声明 DSL 化——auto-edit.at 并入 widget DSL

> **状态**: ✅ P1 已实施并合并（2026-08-26，worktree .worktree/plan-451 →
> master）。desktop_mcp.py **50/0 全绿**（T10 热重载经 DSL 源路径转绿且
> 根治了 Plan 449 期间的 mtime 轮询 flake）；P2/P3 见 §7。
> **来源**: Plan 449 复盘讨论——auto-edit.at 是前端 UI 的声明（action 注册表/
> menubar/toolbar/快捷键），长期归属应在 AutoUI DSL 内（类比 routes/router），
> auto-atom 外挂文件是 Plan 418 的临时形态；工具链收集规则（vue build 按
> 文件收集、auto test 递归）是引擎缺陷而非架构约束。
> **基线**: master 3a3ecf84f
> **性质**: 编译器/DSL 特性（P1 本轮实施：vm 全链路 + 041 迁移）；P2/P3 见 §7。

## 1. 目标

把 041 的 `auto-edit.at`（auto-atom 格式动作配置）迁移为 `widget App` 内的
`actions {}` 声明块，vm 运行时三源绑定（键盘回退层 / menubar·toolbar 合成 /
MCP 同源派发）行为完全不变；外部配置文件路径保留为兼容层。

**非目标（本轮）**：vue 侧 actions 消费（P2，因 vue.rs 与 plan-044 worktree
的未提交改动冲突，且工作量独立成块）；cmd_vue 按声明种类分派（P3 随 P2）。

## 2. 语法设计

```auto
widget App {
    actions {
        action (id: "file.new",  handler: .ActNew,  title: "新建",  icon: "file-plus",   shortcut: "Ctrl+N")
        action (id: "file.save", handler: .ActSave, title: "保存",  icon: "save",        shortcut: "Ctrl+S",
                enabled_if: ".store.tab_count > 0")
        action (id: "edit.undo", handler: .ActUndo, title: "撤销",  icon: "undo-2",      shortcut: "Ctrl+Z",
                checked_if: ".store.edits > 0")
        ...
        menubar {
            menu (id: "file", title: "文件") {
                item (action: "file.new")
                item (action: "file.open")
                sep
                item (action: "file.quit")
            }
            ...
        }
        toolbar { item (action: "file.new")  sep  item (action: "edit.undo")  ... }
    }
    view { ... menubar {}  toolbar (style: "ml-auto") {} ... }
    on { .ActNew -> { store.ActNew() } ... }
}
```

要点：
- **action id 保留点分形式**（"file.new"）——它是 OS 用户键位层的跨版本契约，
  迁移零映射。
- `handler: .ActNew` 指向本 widget 的 `on{}` 事件——**解析期校验**（无匹配
  handler 即 parse error，兑现"编译期校验"承诺；现 auto-atom 形式运行时才对上）。
- `enabled_if`/`checked_if` 取**引号字符串**（条件表达式）：与现文件形态同源，
  由既有 `eval_condition_with` 对合并根 state 求值（`.tab_count` 无前缀语义
  不变）。升级为真 DSL 表达式 AST 随 P2 的 vue 编译一起做。
- 属性名用下划线（`enabled_if`），因 DSL 标识符不含连字符。
- `sep` 裸标识；`item (action: "...")` 引用 action id。

## 3. 实现面（P1）

| # | 文件 | 内容 |
|---|---|---|
| 3.1 | `ast/ui.rs` | `ActionsBlock`/`ActionEntry`/`MenubarBlock`/`MenuEntry`/`MenuItemEntry(enum Action/Sep)`/`ToolbarBlock` 类型；`WidgetDecl.actions: Option<ActionsBlock>`（构造点 ~13 处补 None） |
| 3.2 | `parser.rs` | `parse_actions_decl`（widget 块分派加 "actions" 臂）；handler 对 `decl.on` 的解析期校验；行内单测 |
| 3.3 | `action_config.rs` | ① `UiActionConfig::from_parts(actions, menus, toolbar)`：复用 atom 解析的校验（id 唯一/handler 必填/菜单工具栏引用存在/快捷键冲突）+ shortcut_bindings 计算；② DSL 源存储（`DSL_ACTION_CONFIG` + `DSL_SOURCE_PATH`）与优先级：`action_config()` DSL 优先、文件兜底；③ `set_dsl_action_config(cfg, src_path)`：应用 OS 键位层 + bump generation + 更新 LAST_RELOAD_INFO（保持 "N OS keymap overrides" 响应格式，T11 依赖）；④ app id 解析改为 `AUTO_APP_ID` env → 配置文件 stem；⑤ `reload_action_config()`：DSL 源路径存在时改为**重读源 .at → 重新解析提取 actions**（MCP reload 工具零改动获得 DSL 热重载） |
| 3.4 | `lib.rs` | `run_file_dynamic_ui_inner`：root_decl 解析后若有 actions 块 → `set_dsl_action_config`（带源 path） |
| 3.5 | `crates/auto/src/main.rs` | `auto run` 注入 `AUTO_APP_ID`（pac.at name；紧邻 ui_config 注入处）——DSL-only 项目 OS 键位层的 app id 来源 |
| 3.6 | 041 示例 | app.at 增 actions 块（自 auto-edit.at 机械迁移）；**删 auto-edit.at**；pac.at 去 `ui_config:`；README/注释更新 |
| 3.7 | `tests/desktop_mcp.py` | T10 热重载改造：编辑对象从 auto-edit.at 改为 app.at 的 actions 块（字符串手术：`actions {` 后插 action 行、`menubar {` 后插 menu 行）→ `action_config_reload` → 快照断言新菜单；T7 注释更新 |

不触碰：`ui/iced/renderer.rs`、`ui/aura_view_builder.rs`、`ui/mcp_server.rs`
（三消费点全部经由 `action_config()`，只换供给源）——规避 plan-044 活跃文件。

## 4. 兼容与优先级语义

- **DSL > 文件**：widget 带 actions 块时外挂文件被忽略（打日志）；两者都不
  存在时维持现状（DSL bind/onkeydown 直达）。不做两者合并。
- 文件路径（`AUTO_VM_ACTION_CONFIG`）完整保留——现有 atom 项目零影响。
- OS 键位层（`%APPDATA%/auto/keymaps/<app>.at`）**不变**：它本来就是用户
  偏好覆盖而非 app 代码；app id 解析加 `AUTO_APP_ID` 优先。
- `pac.at` 的 `ui_config:` 字段保留语义（仍可指向文件配置），041 不再使用。

## 5. 验证（P1 完成门禁——2026-08-26 实测）

1. `cargo build --features ui-iced --bin auto` ✓；新增 parser 单测
   （test_actions_block_parse / test_actions_block_handler_validation）✓。
2. `desktop_mcp.py` **50 passed / 0 failed**（两次复现，含撤销调试插桩后
   的干净构建）——超过 Plan 449 基线（48/1），T10 热重载经 DSL 源路径
   转绿且根治了原 mtime 轮询链的 flake（见 §5.1）。
3. T11 OS 键位层 e2e 绿（AUTO_APP_ID 链路生效——pac.at name → auto run
   注入 → keymap 层按 app id 命中）。
4. `auto build` exit 0 ✓；`cargo test -p auto-lang --lib`（ui-iced）
   **3692 passed / 0 failed** ✓（vm::ui_console 的 ring 测试偶发 flaky，
   与本计划无关，重跑即绿；action_config 的 warnings 断言在实施中修复——
   validate_refs 提取时误覆盖了 parse 循环已累积的警告，改回追加）。

### 5.1 实施中的两个额外修复（调查结论）

1. **多文件工程的整源热重载丢 store 状态**（dynamic.rs）：
   renderer 的 HOT_RELOAD_EVENT 路径只重建根 widget（component.reload），
   不重跑 use 模块加载——store/子组件的状态与 handler 全部丢失
   （实测：T10 改 app.at 后主 app 出现 "field not found: tabs"、
   T8 ActQuit 失效）。该路径在 Plan 418-449 期间从未被触发（T10 改的是
   配置文件而非源码）。修复：`check_file_changed` 对多文件工程
   （import_stmts/registry 非空）返回未变更，跳过该路径；动作配置层的
   DSL 源 mtime 轮询（action_config::check_action_config_changed）独立
   不受影响。单文件应用的热重载保持原行为。
2. **T10 等待硬化**（desktop_mcp.py）：重建链实测 ~2s
   （500ms tick → gen-check → view_dirty → view()），原固定 sleep 1.5s
   是时序边缘——改为 6s 轮询 '"T10"' 出现。

## 6. 风险

- WidgetDecl 新字段的构造点遗漏 → 编译器兜底（机械补 None）。
- T10 字符串手术对 app.at 格式敏感 → 在 actions 块内放稳定的插入锚
  （`actions {`/`menubar {` 行首字符串，重构时保持）。
- 并行 worktree（plan-044/plan-450）都在 crates 内活动 → 本计划改动文件
  与它们不交集（parser/ast/action_config/lib/main.rs vs renderer/vue/registry）。

## 7. 后续阶段（另行实施）

- **P2 vue 侧消费**：cmd_vue/ui_build 对 app.at 的 actions AST 生成——全局
  keydown 监听（normalize_shortcut → addEventListener 判定）、menubar/toolbar
  组件树（vue.rs 已有 menubar_* 组件映射）、enabled_if/checked_if 表达式
  转译（届时一并把条件升级为真 DSL 表达式）。与 plan-044 的 vue.rs 改动
  合流后实施。
- **P3 工具链收集规则**：cmd_vue 按**声明种类**分派（widget→SFC、store→
  composable、actions→随宿主、其余→解释执行），使 actions 也可拆独立模块
  经 `use` 引用；auto test 已天然跳过无测试文件（`reports.is_empty()`，
  main.rs:1023），无需改动。
