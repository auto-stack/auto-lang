# Plan 423: Action 配置层 Phase 3 — 热重载 / OS 用户层 keymap / 表达式引擎 / enabled-if

> **状态**: 📋 已立项待实施(2026-08-22,汇集 418 明确推迟的三项 + enabled-if 零消费缺口)
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
