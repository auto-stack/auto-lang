# Plan 423: Action 配置层 Phase 3 — 热重载 / OS 用户层 keymap / 表达式引擎 / enabled-if

> **状态**: ✅ 已实施(2026-08-23,master 直接提交;矩阵 47/47 含 T10 新组;P1-P4 全落地)
> **来源**: 418 §4("热重载不做,重启生效"/"OS 层 keymap 为 Phase 3"/"不做表达式引擎(列后续)")+ §8.4(enabled-if 已解析未渲染)
> **关联**: 418(action_config 管线与三源绑定)/ 275(键绑定管道,archive)/ 032 系(并行进行中的键绑定相关,注意协调)

---

## 0. 一句话结论

**配置层从"启动读一次的静态注册表"升级为"可热更、可分层、可表达条件"的运行时设施**——四个工作项共享 action_config.rs 同一改造面,一轮立项。

## 1. 现状盘点

- **ACTION_CONFIG: `OnceLock<Option<UiActionConfig>>`**(进程首访问即定型,重启生效);调用面 `action_config() -> Option<&'static …>`(builder 合成 + 键盘回退层 + MCP)。
- **keymap 层叠**:仅 app 内置层(auto-edit.at 经 pac.at `ui_config:` 注入);OS 用户层(跨应用个人键位)未实施(418 Phase 3 设计约束:键位声明三源 union,OS 层覆盖 app 层)。
- **表达式**:checked-if 仅支持 `.field` 裸标识符(`convert_menubar` §8.4③ 实现时显式限定);enabled-if 在 `ActionDef` 已解析、**全链路零消费**(按钮永远可点);builder 已有 `resolve_expr_to_value(expr, bindings)`(Expr 解析+求值,bindings/state 双源)——表达式引擎不必从零造。
- **disabled 渲染前置**:Plan 402 autoui_check 常驻警告"button is always clickable; disabled prop not implemented"——enabled-if 落地时一并补齐按钮禁用态,消掉该警告。

## 2. 方案要点

- **配置热替换**:`OnceLock` → `Arc<RwLock<Arc<UiActionConfig>>>`(读侧 clone Arc,零锁读);`auto-edit.at` mtime 轮询(复用 16ms 订阅节拍,变更才触发)或手动 `action_config_reload()` native + MCP 工具;坏配置降级=保留旧值 + eprintln(与首载语义一致)。
- **OS 用户层 keymap**:路径 `%APPDATA%/auto/keymaps/{app}.at`(`~/.auto/keymaps/`);schema 复用 auto-edit.at 的 action/shortcut 段(仅覆盖绑定,不复制 handler);层叠 = app 内置 → OS 层 by action id 覆盖;诊断:启动日志打印生效层与覆盖数。
- **表达式引擎**:checked-if/enabled-if 统一走 `Expr` 解析(auto-atom 已有表达式语法)+ `resolve_expr_to_value` 求值 → `Value::Bool`;求值失败=未勾选/禁用(保守);安全边界:表达式只读 state,无副作用(与既有 ${} 插值同级能力,无新面)。
- **enabled-if 渲染**:`View::Button` 增加 disabled 态——iced `Button` on_press=None(§8.4 探针事件仍记录,快照可见)+ 灰样式(text-zinc-500/60 + cursor 默认);DSL `button (disabled-if)` 属性同期落地,消 402 警告。
- **测试基座**:action_config 单测扩(层叠合并/表达式求值/坏配置降级/热替换原子性);041 增 enabled-if 示例动作 + 矩除断言(禁用项点击无消息、快照带 disabled 标记)。

## 3. Phases

- **P1 配置热替换基建**(ArcSwap 化 + reload 通道 + 降级语义)。
- **P2 OS 用户层 keymap**(路径/层叠/日志)。
- **P3 表达式引擎 + enabled-if 渲染 + disabled 按钮态**(三者一体交付,含 402 警告清理)。
- **P4 MCP reload 工具 + 041 示例与矩阵扩展**。

## 4. 验收

- 改 auto-edit.at(加动作/改快捷键)不重启即生效(mtime 通道或 reload 工具);坏文件保留旧配置且有日志。
- OS 层 keymap 覆盖 app 层生效;表达式 checked-if(如 `.edits > 0`)随状态翻勾;enabled-if 为假时按钮灰态、点击无消息、MCP 快照可见 disabled。
- action_config 单测 + 矩阵全绿。

## 5. 风险

- `action_config()` 返回 `&'static` 的调用面改造(builder/update/键盘回退三处,编译器会全部指出——纯机械但面广)。
- mtime 轮询与并行会话写配置文件的误触发(编辑器半写状态读取——临时文件+rename 原子替换约定)。
- 与 032 系(键绑定)并行会话的改动面重叠——开工前同步。


---

## 6. 实施记录(2026-08-23)

### 落地内容

- **P1 配置热替换**:`ACTION_CONFIG` 从 `OnceLock<Option<UiActionConfig>>` 改 `RwLock<Option<Arc<UiActionConfig>>>`(读侧 Arc clone);`reload_action_config()` 全量重读 + **坏配置保旧值**(parse 失败/读失败均保留上一个 Arc + eprintln);`CONFIG_GENERATION` 原子计数,update 闭包每条消息核对(含 heartbeat)变更即 view_dirty;mtime 轮询挂 heartbeat 节拍 500ms 节流(stamp = 配置文件+OS 层的 (mtime,len) 组合)。
- **P2 OS 用户层 keymap**:`%APPDATA%/auto/keymaps/<app>.at`(非 Windows `~/.auto/keymaps/`),app id = 配置文件名 stem;`parse_keymap_overrides` 只读 action 段的 id+shortcut(handler-less by design);层叠按 action id 覆盖 + `rebuild_shortcut_bindings` 重建;坏 OS 层日志忽略保 app 层;启动/重载日志带覆盖数。键盘回退层每次事件实时查 `action_config()`,重载即时生效。
- **P3 表达式引擎 + disabled**:checked-if/enabled-if 统一走 `eval_condition_with`(字符串表达式引擎:state 引用/比较运算/字面量;求值失败=未勾选/禁用);`View::Button` 增 `disabled: bool` —— iced `on_press=None` + zinc-500 灰样式;**MCP 点击分派拒绝禁用按钮**(`extract_action_from_view` 门控);disabled 进 AURA 快照(vtree Button props + 序列化)与 SnapshotBuilder;DSL `button (disabled-if: "...")` / `disabled: true`;menubar/toolbar 合成项接 enabled-if;render_support 的 402 "always clickable" 警告消除(button 转 Full)。
- **P4 reload 工具 + 041 + 矩阵 T10**:MCP 工具 `action_config_reload`(返回生效计数 + generation);041 `file.save` 带 `enabled-if : ".tab_count > 0"`、`edit.undo` 带表达式 `checked-if : ".edits > 0"`;矩阵 T10(独立新鲜进程):关双 tab→save 禁用标记可见+点击零副作用→+ 打开后恢复→改写 auto-edit.at(根节点内追加 action+menu)→reload→新菜单出现且可派发。

### 验证

- 041 矩阵 47/47(T1-T8 既有 + T9 420 组 + T10 423 组)。
- action_config 新单测:OS keymap 层叠(by id、绑定重建、坏文档拒析)+ 坏配置 parse 失败面。
- `button` 标签 autoui_check 分类升 Full(disabled 实装)。

### 已知问题(挂账)

1. **plan370_015_behavior_tests 的 d3/d4/d6/d7 四项在本会话开工基点(46fd548d)即已失败**(feature 门控 `--features ui-iced` 的测试从未被无特性跑法覆盖;表现为 payload 参数化 handler 副作用未落地,如 SelectNote(3) 后 active_id 仍 "0")。bisect 确认非 420/423 引入。待单独立项修复。
2. 同因暴露:auto-musk 合并(424-426)留下的 `plan370_test_support.rs`/`vm_bridge.rs` 测试构造缺 `setup` 字段(feature 构建 E0063)——本次顺手修复(`setup: None`)。
3. OS keymap 层仅单元级验证(层叠逻辑);端到端(写 %APPDATA% 文件 + reload + 键盘事件)未进矩阵 —— 键盘事件注入工具(AUTOUI_KEYBOARD)走 key_bindings 快照,config 层短路查询与其解耦,人工验收项。
