# Plan 451: actions 声明 DSL 化——auto-edit.at 并入 widget DSL

> **状态**: ✅ P1 已实施并合并（2026-08-26，worktree .worktree/plan-451 →
> master）。desktop_mcp.py **50/0 全绿**（T10 热重载经 DSL 源路径转绿且
> 根治了 Plan 449 期间的 mtime 轮询 flake）。
> ✅ P2/P3 已实施（2026-08-26 续作，见 §7）：vue 侧消费全链路（keydown
> 回退层 + menubar/toolbar 组件树合成 + 条件转译）+ 顶层 actions 声明
> （actions 可拆独立模块经 use 引用）。
> **来源**: Plan 449 复盘讨论——auto-edit.at 是前端 UI 的声明（action 注册表/
> menubar/toolbar/快捷键），长期归属应在 AutoUI DSL 内（类比 routes/router），
> auto-atom 外挂文件是 Plan 418 的临时形态；工具链收集规则（vue build 按
> 文件收集、auto test 递归）是引擎缺陷而非架构约束。
> **基线**: master 3a3ecf84f
> **性质**: 编译器/DSL 特性（P1：vm 全链路 + 041 迁移；P2/P3：vue 消费 +
> 顶层声明，见 §7）。

## 1. 目标

把 041 的 `auto-edit.at`（auto-atom 格式动作配置）迁移为 `widget App` 内的
`actions {}` 声明块，vm 运行时三源绑定（键盘回退层 / menubar·toolbar 合成 /
MCP 同源派发）行为完全不变；外部配置文件路径保留为兼容层。

**非目标（P1 轮）**：vue 侧 actions 消费（P2）、cmd_vue 按声明种类分派（P3
随 P2）——两者已于 2026-08-26 续作实施，见 §7。

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
  不变）。P2 起另接受**裸 DSL 表达式拼写**（`enabled_if: .store.tab_count > 0`，
  解析期按 view if 条件同文法校验）；AST 内两种拼写同为**规范条件串**而非
  Expr AST——单一表示贯通 vm 求值（eval_condition_with）、vue 转译
  （convert_condition）与 auto-atom 文件层，这是有意的设计取舍（复审记录
  见 §8）。
- 属性名用下划线（`enabled_if`），因 DSL 标识符不含连字符。
- `sep` 裸标识；`item (action: "...")` 引用 action id。
- P3 起另支持**顶层** `actions { ... }` 声明（模块级，可拆独立文件经 `use`
  引用到宿主；`actions` 后继 `{` 才接管，普通表达式用法不受影响）。

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

## 7. 后续阶段（✅ 已于 2026-08-26 续作实施）

### P2 vue 侧消费（✅ 完成）

- **全局 keydown 监听**：`AuraWidget.actions`（extract 接线）→ vue.rs
  `generate_sfc` 预计算 `actions_key_bindings`（`normalize_shortcut` 归一，
  首声明者胜，与 vm `shortcut_bindings` 同碰撞规则）→ `generate_script`
  发射 `__autoActionsKeymap` 常量 + `__autoActionsKeydown`（window
  addEventListener/removeEventListener 成对，onMounted/onUnmounted）。
  语义对齐 vm：元素级 onkeydown 之下的回退层；命中即 preventDefault
  （抑制 Ctrl+S 存页等浏览器默认）；无修饰键击键在输入框聚焦时跳过
  （对应 vm 焦点件捕获）；mac Cmd 映射到 Ctrl 层（web 侧便利）。
- **menubar/toolbar 组件树**：view 中 `menubar {}` / `toolbar (style: …) {}`
  占位标签（无显式子节点 + shadcn 模式）→ 从 actions 声明合成。menubar =
  shadcn Menubar 家族树（MenubarMenu/Trigger/Content/Item/Separator；
  勾选槽 16px + lucide Check + 右对齐快捷键文本，镜像 vm convert_menubar）；
  toolbar = ghost 图标按钮（lucide 组件 + title 原生 tooltip + enabled_if →
  :disabled）+ 分隔线。schema/aura.at 补 `menubar_separator` 的 vue 行
  （web: component + MenubarSeparator 导入）+ registry 补 MenuBarSeparator
  spec（extras 导入解析）。
- **enabled_if/checked_if 转译**：经 `convert_condition`（与 view if 条件
  同一翻译表：`.store.x > 0` → `store.x > 0`，方法映射/None→null 同源）。
  条件拼写升级为真 DSL 表达式：`enabled_if: .store.tab_count > 0`（裸表达式，
  解析期按 view if 条件同文法校验，含分组括号/逻辑运算/方法调用）与引号串
  等价并存（AST 内同为规范条件串，vm/vue 两端零分叉）。
- **已知边界**：占位标签合成仅在 shadcn 模式（`shadcn: off` 的 plain 模式
  保持占位直通——vm 无此分叉，合成不挑模式）；keydown 回退层不挑模式，
  任何模式都随 actions 声明发射。
- **验收**：041 vue 生成（`auto build --gen-only --render vue`）产出完整
  App.vue（keymap 9 项 + 四菜单 menubar + 9 按钮 toolbar + Check/禁用态）；
  vue.rs 内联单测 5 例（test_actions_*）全绿；desktop_mcp.py 50/0（vm 行为
  零回归）；lib 全量 3701/0；schema_drift/docs_gen/gallery_golden 绿。

### P3 工具链收集规则（✅ 完成）

- **顶层 `actions {}` 声明**（`Stmt::ActionsDecl`）：UI 方言关键字接管
  （`actions` 后继 `{` 才接管，普通表达式用法不受影响；词法前瞻一格取回
  压回）。handler 引用合并进宿主后由消费端校验（独立声明无宿主可校验）。
- **合并优先级**（vm 与 vue 同序）：宿主 widget 自带块 > 同文件顶层声明 >
  use 引入模块的顶层声明（导入序首个）。vm：run_file_dynamic_ui_inner
  （import_stmts 收齐后回退安装）；vue：generate_component_from_file
  （含 use 模块文件解析，坏模块跳过不致命）；热重载：
  extract_actions_from_source 同优先级提取（模块声明改动需 touch 宿主
  文件触发——只 watch 宿主 mtime）。
- **已知边界**：vm 侧 use 拾取走 import_stmts **传递闭包**（孙模块的
  actions 也命中）；vue 侧 collect_use_module_actions 只扫**一级** use
  模块（孙模块不追）——actions 放孙模块是极端形态，两端的差已如实记录。
- **分派容忍**：actions-only 文件（无 widget/store）不再报错，产出空工件；
  cmd_vue（Phase 1/3）与 auto-man（pages 扫描）跳过空工件文件（无 junk
  SFC）。auto test 确认无需改动（reports.is_empty() 天然跳过）。
- **验收**：api.rs 合并测试 4 例（同文件拾取/自带块优先/actions-only 容忍/
  use 模块拾取）+ parser 顶层声明测试 + action_config 提取优先级测试。

## 8. finish-plan 复审记录（2026-08-26）

按 finish-plan 流程对 P1/P2/P3 全任务逐项对照代码复审，全部验证命令在最终
提交状态重跑：`cargo build --features ui-iced --bin auto` ✓、lib 全量
**3701/0** ✓、schema_drift 4/0 + docs_gen 1/0 + gallery_golden 1/0 ✓、
041 vm `auto build` exit 0 ✓、041 vue 生成合成完整 ✓、desktop_mcp.py
**50/0** ✓（本次会话复现；前两次失败为残留进程/端口占用的已知 flake，
清理后稳定）。P1 3.6 实物核对：auto-edit.at 已删、pac.at 无 `ui_config:`、
README 已更新。

复审发现（均已处置）：

1. **计划文本 vs 实现的分叉（§2）**：§2 原文"升级为真 DSL 表达式 AST"；
   实装为**规范条件串 + 裸表达式拼写**（解析期 token 文法校验）。这是有意
   取舍：单一表示贯通 vm 求值（eval_condition_with）、vue 转译
   （convert_condition）与 auto-atom 文件兼容层，避免 Expr↔串双表示的
   往返损耗。§2 已改写为实装语义。残余差距：条件表达式无类型级编译期校验
   （仅 token 文法级）——如未来需要，可升级为 Expr AST 并以序列化器保持
   三端兼容。
2. **vm/vue use 拾取深度不对称**（见 §7 已知边界）：vm 传递闭包 vs vue
   一级。极端形态（actions 放孙模块）vue 不拾取；如遇实际需求，vue 侧
   collect_use_module_actions 补递归即可。
3. **plain 模式（`shadcn: off`）占位标签不合成**（见 §7 已知边界）：
   vm 合成不挑模式，vue 合成依赖 shadcn 组件族；plain 模式保持占位直通。
4. **格式修复**：parser.rs `parse_actions_block_inner` 函数头与首语句在
   P2 编辑中被粘行（无行为影响），复审时已修正并随收尾提交。
5. **与本计划无关的预存问题**（不阻塞归档，如实登记）：
   `tests/ui_snapshots.rs::snapshot_editor` 快照在 master 基线上已过期
   （stash 本计划全部改动后实测同样输出 6644B，与本计划无关）；
   041 vue 严格构建存在预存 R006 警告（vm 期 tab 循环缺 :key，
   vue 验证经 --lenient 门禁）。

**结论**：A——全部任务完成并验证，无未完成项；发现项均为文档化边界或
预存问题。归档。
