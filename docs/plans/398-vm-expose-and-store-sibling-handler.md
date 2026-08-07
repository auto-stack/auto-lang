---
plan: 398
title: vm-expose-and-store-sibling-handler
affects: [auto-lang/ui/handler_codegen, auto-lang/ui/vm_bridge, auto-lang/ui/dynamic]
status: in-progress # draft | in-progress | complete
---

# Plan 398: VM 兼容性修复 — expose 生效 + store handler 互调

> **For Claude:**
> - 构建：`cargo build --features ui-iced --bin auto`（VM UI 需 `ui-iced` feature）。
> - 冒烟：`cd D:/autostack/auto-shell/ash-gui/ash-gui-auto && auto run -r vm`（应最终看到
>   `AutoUI MCP: first state sync in view()`，无 `Undefined symbol` / panic）。
> - 回归：`cd examples/ui/015-notes && auto run -r vm` 与 `examples/ui/013-todo` 必须仍正常开窗；
>   `examples/ui/015-notes/tests/desktop_mcp.py` 仍通过。
> - 断点/续接：见 `D:/autostack/auto-shell/designs/ash-gui-vm-diagnosis-resumption.md`。
> - 完整修复方案与背景：见 `D:/autostack/auto-shell/designs/ash-gui-vm-fix-plan.md`。
>
> **计划性质**：修 3 个阻塞 ash-gui 在 vm 模式启动的 VM 兼容性缺陷。BUG-C（expose 无效）
> 与 BUG-B（store handler 互调）是真 VM bug，在本计划修；BUG-A 是惯例缺口，只补文档。
> 三 bug 的诊断全过程与已排除的 13 个假因记录在 ash-gui-native-plan §9.1–§9.7。

---

## §1 Goal / 目标

让 `auto run -r vm` 能启动 ash-gui（含 PromptBar 完整 expose + 内部 handler、store
handler 互调），从而解锁 ash-gui-native 项目的 M0（MCP 连通 + 测试骨架）。

**三个 bug**：

| Bug | 一句话 | 性质 | 本计划 |
|---|---|---|---|
| **BUG-A** | App 调 `store.X()` 时 store 的 `use back.api` 不透传到 App 作用域 | 惯例缺口 | §4（补文档） |
| **BUG-B** | store handler 内部调 `.Sibling()` 另一个 store handler → `<Store>_State.X` 未定义 | 真 VM bug | §3（修 codegen） |
| **BUG-C** | 子组件 handler 仅被内部引用 → `<Child>_State.X` 未定义；`expose{}` 被解析但 VM 运行时从不消费 | 真 VM 功能缺口 | §2（修 codegen/runtime，最高优先） |

**关键事实（复核源码确认）**：`AuraWidget.exposes: Vec<Name>` 被 parser 正确填充
（`parser.rs:11326/11366`，调 `parse_expose_block_inner` `parser.rs:11630`），但 VM 运行时
**从不读取它**：
`grep -rn '\.exposes' crates/auto-lang/src/ui/{handler_codegen,vm_bridge,dynamic}.rs` → 0 匹配。
（注：`vm_bridge.rs:1084` / `dynamic.rs:1074,1207,1403,1477,1670` / `aura_view_builder.rs:2823`
的 `exposes: vec![]` 都在 `#[test]` 块里，是测试 struct literal，不是生产代码——已复核。）

---

## §2 BUG-C：让 `expose` 真正生效（最高优先，M0 真阻塞）

### 现象
子组件 handler 仅被内部引用（非模板直接绑定）→ link 报
`Undefined symbol: <Child>_State.<Handler> in module App`。PromptBar 系统性重灾区
（~10 个内部 handler：`PickCompletion` / `AcceptGhost` / `Exit` 等）逐个报错。

### 第一步：精确诊断（先判定根因在 synthesize 还是符号查找，30 分钟）

在 `crates/auto-lang/src/ui/handler_codegen.rs:172` `synthesize_widget_module` 入口加
临时 `eprintln!`，枚举每个 widget 生成的 handler 集合，跑 ash-gui vm：

```rust
eprintln!("DEBUG 398: handlers for {}: {:?}",
    widget.name, widget.handlers.keys().collect::<Vec<_>>());
eprintln!("DEBUG 398: exposes for {}: {:?}", widget.name, widget.exposes);
```

- 若 PromptBar 的 handler 集合**不含** `Exit`/`PickCompletion` → synthesize 阶段漏了，
  根因在 `synthesize_widget_module`。
- 若**含**（即所有声明 handler 都已 synthesize）→ 符号已生成但 link 找不到，根因在
  `vm_bridge.rs` 的符号查找/`<Child>_State` 命名约定（查 `call_handler` /
  `call_handler_for` `vm_bridge.rs:668,800,831,869`）。

### 第二步：按诊断结果修复

**分支 A（synthesize 漏了）**：在 `synthesize_widget_module` 里，把 `widget.exposes`
的每个名字当作"必须生成的 handler"加入合成集合（即使模板未引用）。复用现有
`handler_<Widget>_<Event>` 命名（`handler_codegen.rs:296-302` `namespaced_handler_fn_name`）。

**分支 B（符号查找/命名问题）**：理解 `<Child>_State.<Handler>` 这个命名是怎么生成的
（`State.X` 形式暗示 per-child state struct，字段 = handler 名）。修对应的符号解析，让
exposed handler 能被 `<Child>_State.X` 找到。可能要同时改 `dynamic.rs` 的 child state
struct 字段构造 + `vm_bridge` 的查找。

### 验证
- ash-gui：`auto run -r vm` 不再报任何 `<Child>_State.X`；PromptBar 完整能 link。
- 回归：015-notes、013-todo 的 vm 启动仍正常（它们的 expose 用法不能退化）。

### 风险
- 分支 B 若动到 child state struct 命名约定，影响面大——必须全量回归 015/013 + 它们的
  MCP 测试。
- 若暴露面过大，触发 §6 降级方案。

---

## §3 BUG-B：store handler 互调 `.Sibling()`

### 现象
store handler 内部调 `.Sibling()`（同 store 的另一个 handler）→ link 报
`Undefined symbol: <Store>_State.<Sibling>`。

### 根因（已验证）
`handler_codegen.rs:103-130` 的 `rewrite_expr` 只覆盖 **`store.Method()`（store 别名
调用，obj 是 store 别名，在 `STORE_WIDGET_NAMES` 注册）**，rewrite 成
`handler_<StoreName>_<Method>(__state, args)`。

但 **store handler 内部的 `.Sibling()`**（obj 是隐式 self，不是 store 别名）走
`rewrite_state_refs_stmts`（`handler_codegen.rs:60-70`）路径——它把 `.field` rewrite 成
`__state.field`（字段访问），但 `.Sibling()` 是**调用**（`Expr::Call` 包 `Expr::Dot`），
被当字段访问处理 → 生成 `__state.Sibling`（无意义）或落到未覆盖分支 → 符号未定义。

### 修复
在 `handler_codegen.rs` 的 `rewrite_expr` 补一条 **store handler 内部 sibling 调用**
的 rewrite 规则：识别 `Expr::Call { name: Expr::Dot(obj, method) }`，obj 是隐式 self
且 `method` 是当前 store 的 msg variant → rewrite 成
`handler_<CurrentStoreName>_<Method>(__state, args)`（复用 `STORE_WIDGET_NAMES` + 当前
widget 名，同 `store.Method()` 的 rewrite 逻辑 `handler_codegen.rs:110-130`）。

需把"当前正在 rewrite 的 widget 名"thread 进 rewrite（当前是 stmt 级，可能要加参数或用
thread-local，仿 `STORE_WIDGET_NAMES` `handler_codegen.rs:29` 附近）。

### 验证
- 回归测试：最小 store 用例，`.A` handler 内部调 `.B`（`.B` 存在），vm link 通过 +
  调 `.A` 时 `.B` 也执行。
- ash-gui：撤销 BUG-B workaround（`shell_store.at` 恢复 `.RefreshGit()` 调用），确认不再
  报 `ShellStore_State.RefreshGit`。

### 风险
- 低：rewrite 规则局部。015/013 回归验证即可。

---

## §4 BUG-A：补文档（不修 VM）

### 现象
App 调 `store.X()` 时 store 的 `use back.api: ...` 导入不透传到 App 作用域 →
`api.<fn> in module App` 未定义。

### 处置
- **不修 VM**（自动透传 store 导入涉及作用域模型大改，不值当）。
- ash-gui-auto 的 workaround（app.at 自己 `use back.api`）**保留**——合法 .at 写法，
  即使修了 BUG-B/C 也不冲突。
- 补 `D:/autostack/skills/auto-ui-creator/SKILL.md` 的 U1（store 访问规则）加一条：
  > 当 widget 调 `store.X()`，而 store handler 用到 `back.api` 函数时，**调用方 widget
  > 必须自己也 `use back.api: <用到的 fn>`**——VM 不透传 store 的导入。Vue/a2r 后端
  > 无此要求（codegen 自动处理）。
- 同步 `tests/probes/gotcha-probe.at` + `verify.sh` 加断言。

---

## §5 残留 `api.complete` 的二分（BUG-C 修复后第一步）

应用 BUG-A/B workaround 后仍报 `Undefined symbol: api.complete in module App`。
未判定它是 BUG-C 的表现，还是独立的第四个 bug（store handler 的 `return` 语句）。

### 二分
1. 清空 `shell_store.at` 的 `.Complete` body：`.Complete(l,c) -> { }`。
2. `auto run -r vm`：
   - `api.complete` 消失 → `.Complete` body 问题（很可能是 `return items` 或
     `var items []CompletionItem = complete(...)` 的类型注解）→ 独立 bug，本计划加 §N。
   - 仍在 → 与 `.Complete` 无关，继续二分 store 各 handler。
3. 结果记录到 `ash-gui-vm-diagnosis-resumption.md` §4。

---

## §6 风险与降级

- **BUG-C 影响面过大**（动到 child state struct 命名约定）→ **降级**：vm 模式先跑
  简化 PromptBar（去 ghost/completion/highlight，只留 input+run+history），完整
  PromptBar 留给 a2r/HTTP 模式。最后手段，会偏离 ash-gui "UI/UX 一致" 目标。
- **auto-lang 维护者排期冲突** → ash-gui-native M0 可先在"简化 PromptBar + vm"上跑通
  测试骨架，不等 BUG-C 完整修复。

---

## §7 执行顺序

```
1. §2 BUG-C 精确诊断(synthesize vs vm_bridge)→ 修复 → 回归
2. §3 BUG-B 修复(handler_codegen 补 sibling rewrite)→ 回归
3. 回 ash-gui-auto:撤销 BUG-B workaround(恢复 .RefreshGit()),验证 B/C 修复
4. §5 二分残留 api.complete
5. §4 BUG-A 补 skill 文档 + probe
6. ash-gui vm 完整启动验证 → 回 ash-gui-native-plan M0.5(MCP + 测试骨架)
```

---

## §8 验收

- **BUG-C**：ash-gui `auto run -r vm` 不再报任何 `<Child>_State.X`；PromptBar 完整
  （含 expose + 内部 handler）能 link。
- **BUG-B**：`shell_store.at` 恢复 `.RefreshGit()` 后不再报 `ShellStore_State.RefreshGit`。
- **回归**：015-notes、013-todo 的 `auto run -r vm` 仍正常开窗；015 的
  `desktop_mcp.py` 仍通过。
- **ash-gui**：vm 完整启动，`AutoUI MCP: listening on :9247`，`autoui_snapshot` 返回 App 树。

---

## §9 进度跟踪

- [ ] §2 BUG-C 精确诊断(synthesize vs vm_bridge)
- [ ] §2 BUG-C 修复 + 回归测试
- [ ] §3 BUG-B 修复 + 回归测试
- [ ] 回 ash-gui 验证 + 撤销 BUG-B workaround + §5 二分 api.complete
- [ ] §4 BUG-A 补 skill 文档 + probe
- [ ] ash-gui vm 完整启动验证
- [ ] 回 ash-gui-native-plan M0.5

---

## §10 相关文档（跨仓库）

- 诊断全过程与证据：`D:/autostack/auto-shell/designs/ash-gui-native-plan.md` §9.1–§9.7
- 改完 bug 后的续接指南：`D:/autostack/auto-shell/designs/ash-gui-vm-diagnosis-resumption.md`
- ash-gui-native 总计划：`D:/autostack/auto-shell/designs/ash-gui-native-plan.md`
- auto-ui-creator skill（BUG-A 文档更新点）：`D:/autostack/skills/auto-ui-creator/SKILL.md`
