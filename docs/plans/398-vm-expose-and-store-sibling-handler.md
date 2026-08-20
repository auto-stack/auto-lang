---
plan: 398
title: vm-expose-and-store-sibling-handler
affects: [auto-lang/ui/handler_codegen, auto-lang/ui/vm_bridge, auto-lang/ui/dynamic]
status: partial # 核心 VM 修复完成并合并(§12 parser + §2/§3 sibling-handler + §11 log);唯一遗留 §14.1 回归测试;M0.5/M1 为 auto-shell 侧下游任务
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

---

## §11 重大修正（深入诊断后，2026-08-07）— 真根因是"静默 parse 错误"

> §1–§9 的"3 个 VM bug"框架是在**没看到真根因前**的推断。深入诊断（加 eprintln
> 到 `synthesize_from_decl` + `collect_module_imports`）后真相浮出:**ash-gui 的 vm
> 启动失败主要是 .at 文件的 parse 错误被静默吞掉**,不是 VM 链接/作用域 bug。

### 真根因(已验证)

**`collect_module_imports`(`lib.rs:2290`)的 `Err(_) => return` 静默吞 parse 错误。**
当 `back/api.at` 或 `front/shell_store.at` parse 失败,该模块的所有符号(pub fn / type)
都不进 VM module,下游报 `Undefined symbol: api.X` —— 误导我们去查"VM 链接/作用域",
而真因是 parse。

**两个 .at parse 错误(已被前面的诊断 eprintln 暴露):**

1. **`back/api.at`**:`[][]T`(嵌套数组,如 `rows: [][]RenderedCell`)与 `[](tuple)`
   (元组数组,如 `fields: [](str, RenderedCell)`)在 **Core scenario parser** 下不被
   支持(UI scenario 能 parse,但 back/ 走 Core)。验证:把 `[][]T` → `[]T`、
   `[](str, RenderedCell)` → `[]str`,api.at 立即 parse 通过(`fns=8 types=19`)。
   → **.at 层修复**:vm 模式下把这两种类型改成 `[]T`(或定义 wrapper type)。
   → **可选的 VM/parser 改进**:让 Core parser 也支持 `[][]T` / `[](tuple)`(本计划
     §12)。

2. **`front/shell_store.at:29`**:`var git_info PromptContext = PromptContext{
   git_branch: "", git_status: None }` —— `PromptContext.git_status` 类型是
   `GitStatusInfo`(非可选 struct),`None` 类型不匹配(`FieldMismatch` 错误)。
   Vue 版 `git_status: null` 能跑是因 TS 宽松;Auto 类型严格。
   → **.at 层修复**:把 `PromptContext.git_status` 改成 `?GitStatusInfo`(可选),
     或默认值用一个空 struct(各字段 0)而非 None。

### 真正的 BUG-C(确实存在,但要先修上面两个才暴露)

修了上面两个 parse 错误后,ash-gui 终于前进到真正的 BUG-C:
`Undefined symbol: PromptBar_State.Exit in module App`(PromptBar 的 expose + 内部
handler)。§2 的诊断确认 `synthesize_from_decl` **已经为 PromptBar 生成了全部 13 个
handler**(`.PickCompletion`/`.Exit`/`.AcceptGhost` 等都在),`expose=["Exit"]` 也被
parser 正确填充 —— 所以 BUG-C 不在 synthesize,而在 **`<Child>_State.<Handler>` 的
符号查找/命名**(linker 或 vm_bridge 层)。§2 的修复方向(分支 B)成立。

### 对原计划的影响(修正)

| 原判断 | 修正 |
|---|---|
| BUG-A 是"store use back.api 不透传" | ❌ 错判。真因是 api.at parse 失败 → api.X 全消失。app.at 加 `use back.api` 的 workaround 无效。 |
| BUG-B 是当前阻塞 | ❌ 不是当前阻塞。RefreshGit 互调是真 bug 但排在 parse 错误与 BUG-C 之后。 |
| 19 pub type 触发 link 失败 | ❌ 错判(三轮已证伪,本轮再次确认:简单 type 全 parse OK)。真因是 `[][]T`/`[](tuple)` 两种**类型语法**不被 Core parser 支持。 |
| 当前阻塞是 BUG-C | ✅ 修了两个 parse 错误后,BUG-C 才是当前阻塞。 |

### 修正后的执行顺序(替换 §7)

1. ✅ **(VM 侧,已做)** `lib.rs:2290` 的 `Err(_) => return` → `log::warn`(parse 错误
   不再静默,本 commit)。
2. **(ash-gui .at 层)** 修 `api.at` 的 `[][]T`/`[](tuple)` → `[]T`;修
   `shell_store.at` 的 `git_status: None` → 空 struct 或 `?GitStatusInfo`。
3. **(VM 侧,§2 BUG-C)** 修 `<Child>_State.<Handler>` 符号查找(让 expose 生效)。
   此时 ash-gui 应前进到 PromptBar 之后的下一个问题(或启动)。
4. **(VM 侧,§3 BUG-B)** 修 store handler 互调(优先级降,workaround 仍可用)。
5. ash-gui vm 完整启动 → 回 ash-gui-native-plan M0.5。

§12 是可选的 parser 改进(让 Core 也支持 `[][]T`/`[](tuple)`),做完后第 2 步的 .at
workaround 可回退。

---

## §12（可选）Core parser 支持 `[][]T` 与 `[](tuple)` 类型

### 现象
- `back/` 模块按 Core scenario 解析(`lib.rs:2280-2284` 路径启发:含 `back` → Core)。
- Core parser 不支持 `[][]T`(嵌套数组)与 `[](T1, T2)`(元组数组)作为 pub type 字段类型。
- UI parser 支持(所以 front/ 的同语法没问题;Vue codegen 也没问题)。

### 修复(可选)
在 Core parser 的类型解析分支补 `[][]T` 与 `[](tuple)`。定位:parser.rs 的 type 解析
函数(grep `[][]` 或 `ArrayType`)。做完后 ash-gui api.at 可恢复原 `[][]RenderedCell` /
`[](str, RenderedCell)` 语法,.at 层 workaround 可回退。

### 优先级
低——.at 层用 `[]T` workaround 已够 vm 跑通。只有当 Vue codegen 也需要这两种语法、
或多个项目踩同一坑时,才值得修 parser。

---

## §13 进度跟踪（修正后）

- [x] §11 真根因诊断(parse 错误被静默 + 两个 .at parse 问题 + 真正 BUG-C 定位)
- [x] §11.1 lib.rs:2290 parse 错误改 log::warn(commit 25642f91)
- [x] §11.2 ash-gui .at 修两个 parse 问题:
      - shell_store git_status: None → 内联全零 GitStatusInfo(auto-shell commit 455b02e)
      - api.at [][]/[](tuple) → **改为修 Core parser(§12),不动 .at 语义**(见下)
- [x] §12 Core parser 支持 [][]T / [](tuple)(commit 883b13cf)
      parse_array_type 三条路径(slice/runtime/static)元素类型改用 parse_type(递归)。
      api.at 原始 [][]RenderedCell / [](str,RenderedCell) 语法保留,无需 .at workaround。
- [x] §2 BUG-C + §3 BUG-B:sibling handler 调用正确 rewrite(commit cba655c8)
      一处修复覆盖两类:rewrite_expr 新增 `.X()` (self receiver + msg variant) →
      handler_<Widget>_X(__state, args)。修通 PromptBar_State.Exit(BUG-C)与
      ShellStore_State.RefreshGit(BUG-B)。
- [x] **ash-gui vm 完整启动!** AutoUI MCP: first state sync,12 工具可调,
      autoui_snapshot 返回 App 树。
- [x] 回归 015-notes / 013-todo vm 启动正常;015 desktop_mcp.py 8 pass / 2 预存 fail
      (NavTree FALLBACK + notes VmRef,均非本 plan 引入)。

### §14 待办(plan 核心已闭环,这些是后续增强)

- [ ] parser fix + sibling-handler fix 加 Rust 单元测试(回归保护,handler_codegen tests 模块)——2026-08-20 核查:仍未做,tests 模块仅 5 个测试且无一覆盖 sibling rewrite;parser.rs 无 `[][]T` 用例
- [x] synthesize_widget_module(AuraWidget 路径)同样补 set_current_widget——2026-08-14 由 commit b0434cff(Plan 056 blocker A)顺带完成(handler_codegen.rs:881)
- [ ] 回 ash-gui-native-plan M0.5:MCP 连通已证 → 搭测试骨架(conftest/desktop_mcp/test_smoke)(auto-shell 侧下游任务)
- [ ] ash-gui-native M1:in-process 后端(shell.at mock)+ SSE 桥(auto-shell 侧下游任务)

> **§4 处置说明（2026-08-20 复核）**：BUG-A 的交付物（auto-ui-creator SKILL.md 加"VM 不透传 store 导入"规则 + gotcha-probe.at）**不再需要**——§11 插桩已推翻原判断（真因是 parse 错误静默 + sibling rewrite），§13 修正后的修复清单已不含该项。

