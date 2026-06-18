# Plan 327：让 015-notes 在 VM 渲染模式下跑通

> **For Claude:** 本计划处理 Plan 323（Option B）合并后、015-notes 在 `render:vm` 下暴露的剩余阻断点。**前置已修**：点分模块路径（commit `64ba5f21`，`use back.api` → `back/api.at`）。改 stdlib/VM/codegen 后跑 `cargo build -p auto` + `cargo test -p auto-lang --lib`。每个新计划用专用 worktree（sibling：`../auto-lang-327`，**不要**用 `.claude/worktrees/` 嵌套——会破坏 `../auto-ai` 相对路径依赖）。

## 背景

Plan 323 让 016-calendar 在 VM 模式跑通（空网格 bug 修复，已合并 master `564c97bd`）。用户试 015-notes（`auto r -r vm`）立即撞到：

```
Undefined symbol: delete_note in module App
```

深挖后确认是 **三个叠加阻断点**，点分路径（#1）已修，本计划处理 #2、#3，并标注与 Plan 325-autovm 的依赖。

## 阻断点全景（已确认）

| # | 阻断点 | 状态 | 归属 |
|---|---|---|---|
| 1 | 点分模块路径 `back.api` → 找 `back.api.at`（应为 `back/api.at`） | ✅ 已修 `64ba5f21` | Plan 323 收尾 |
| 2 | **传递依赖 + 限定调用**：`back/api.at` 函数体 `use db; db.all_notes()`，当前 `run_file_dynamic_ui` 只收一层 `use`，不递归加载 `back/db.at`；且 `db.all_notes()` 是限定名，db.at 导出的是裸名 `all_notes` | ❌ 本计划 Phase 1 | Option B 通用能力缺口 |
| 3 | **模块级可变状态**：`back/db.at` 顶部 `var notes List<Note> = ...` + `var nextid int = 3`。AutoVM **无 global/模块级变量机制**（Plan 323 已确立）。实测常规求值器跑等价的跨函数模块级 `var` 也报 `Undefined variable: count`——**这是语言级缺口，非 VM 渲染独有** | ❌ 本计划 Phase 2（含设计决策点） | 语言/VM 缺口 |
| 4 | （依赖）**AutoVM 跨模块基础不稳**：Plan 325-autovm（`docs/plans/325-autovm-enum-method-and-cross-module-bugs.md`，与 AI-daemon 的 325 撞号）记录"跨模块调返回字符串的函数都不可靠"、enum 实例方法不被调用等。**未标记完成**，很可能仍在 | ⛔ 前置依赖 | Plan 325-autovm 范畴 |

> **关键判断**：#2 的"限定调用解析"和 #4 的"跨模块字符串返回"有重叠。**先确认 Plan 325-autovm 是否已落地**（见 Phase 0）；若未落地，#2 的部分子项可能要等 325，或与 325 合并推进。

## Phase 0 — 调查与依赖确认（不动代码）

**目的**：在写实现前，把"现状"钉死，避免 Phase 1/2 建在错误假设上。

1. **Plan 325-autovm 现状**：跑其最小复现（`enum_method_bug.at`、跨模块返回字符串），确认 enum 实例方法 / 跨模块字符串是否仍坏。若仍坏 → 本计划的 #2 验收必须等 325（或并入），在计划里标注 hard-dependency。
2. **限定调用解析**：构造最小例 `use m: foo`，m.at 里 `fn bar() { m.baz() }`（同模块限定调用）+ 跨模块 `db.all_notes()`，看 Codegen 当前对 `module.fn` 形式的 CALL 是生成 reloc 还是能解析到裸名 fn。输出：限定调用是否需要"剥前缀"后处理。
3. **模块级 var 现状**：确认常规 `run()` 路径如何编译顶层 `var`（是否进某 global slot、还是被当成 main 的 local），以及跨函数是否可见。输出文档化结论。

**产出**：在本文档追加「Phase 0 调查结论」节，更新 Phase 1/2 的具体步骤。

## Phase 1 — 递归 import + 限定调用解析（阻断点 #2）

**文件**：`crates/auto-lang/src/lib.rs`（`run_file_dynamic_ui`）、`crates/auto-lang/src/ui/handler_codegen.rs`（`synthesize_widget_module`）

### 1.1 递归 import 收集
把当前单层 `use` 收集改成 **BFS/DFS 递归**（带 `visited: HashSet<PathBuf>` 防环）：
- 入口：front widget 的 `use` 语句。
- 每加载一个模块文件（用 Phase #1 已修的 `resolve_module_path`），除收集其 Fn/TypeDecl/EnumDecl/Ext 外，**再 scan 该模块自身的 `use` 语句**（`use_scanner::scan_use_statements` 是行级扫描，能抓到函数体内的 `use db`），递归加载被依赖模块，收集其符号。
- 符号去重（按 `stmt_symbol_name`），后加载不覆盖先加载。

### 1.2 限定调用解析
`back/api.at` 里 `db.all_notes()`、`db.create_note(...)` 等。两种方案（Phase 0 调查后二选一）：
- **方案 A（剥前缀）**：在 `synthesize_widget_module` 喂给 Codegen 前，对 import 来的 Fn body 做一次 AST 改写——把 `Expr::Call(name=Dot(Ident(module), method), ...)` 中 `module` 是已加载模块名的，改写成裸名 `method(...)`。前提：裸名在本 module 内唯一（去重已保证）。
- **方案 B（注册别名）**：让 Codegen/Linker 接受 `db.all_notes` 这种限定符号解析到同名裸 fn（在 link 阶段 symbol-not-found 时剥最后一段 `.` 前缀重试）。

推荐 **方案 A**（AST 改写，与 Option B 的 `__state.field` 改写同构，可控、不污染 Linker）。

### 1.3 单测
- 最小例：app `use a.b: foo`，`a/b.at` 的 `foo()` 体 `use a.b.dep: helper`（或同目录 `dep`）调 `dep.helper()`，断言 VmBridge 能调 `foo` 且执行到 `helper`。
- 不破坏 016-calendar（已有 `test_calendar_init_builds_42_cells`）。

## Phase 2 — 模块级可变状态（阻断点 #3）—— ⚠️ 设计决策点

`back/db.at` 的 `var notes` / `var nextid` 是**跨函数持久的模块级可变状态**。这是语言级缺口。**两条路，需用户拍板**：

### 方案 α：给 VM 加模块级 statics（语言特性）
- 在 AutoVM 加一个 heap 挂载的「模块 statics 表」（按 `module.symbol` 命名键），Codegen 把模块顶层 `var X = init` 编译成"首次访问惰性初始化 + GET_STATIC/SET_STATIC"。
- 优点：通用，常规 `run()` 路径也受益（修复语言级缺口），未来所有后端 Auto 代码可用。
- 缺点：真·编译器+VM 改动，工作量大；与 Plan 325-autovm 的跨模块稳定性耦合；需设计初始化时机/线程安全。

### 方案 β：重写 015-notes，把存储下沉到 widget state（推荐）
- 把 `notes` / `nextid` 从 `back/db.at` 模块级 `var` 改成**由 widget AppState 持有**（notes 本就是单进程内存数据，没有真正的后端边界）。
- db 函数改成接收 `notes` 作参数：`all_notes(notes) []Note`、`create_note(notes, title, body) (Note, []Note)` 等（或返回新 list，函数式风格）。
- 优点：**零 VM 改动**，完全契合 Option B 的 state-as-param 模型；架构上更干净（notes 是 UI 状态）。
- 缺点：改示例代码；`#[api]` 标注的 HTTP 端点语义会变（但这些端点在 VM 渲染单进程里本就不走 HTTP）。

**推荐方案 β**（小、聚焦、不引入新 VM 机制），方案 α 作为独立语言演进计划另立。

### 2.x 步骤（按选定方案）
- **若 β**：改 `examples/ui/015-notes/src/back/db.at`（去模块级 var，函数加 notes 参数）+ `back/api.at`（透传）+ `front/app.at`（handler 里把 `.notes` 传进 db 函数）。需调 `/auto-lang-creator` 技能写 .at。
- **若 α**：另起计划（建议 Plan 328），本计划只标注依赖。

## Verification

1. **每步**：`cargo build -p auto`；`cargo test -p auto-lang --lib`（不得新增失败；现有 4 个 ui 预存失败除外）。
2. **Phase 1**：限定调用单测 + 递归 import 单测；016-calendar 不回归。
3. **Phase 2（β）**：015-notes 各 handler（Init 载入列表、+New 建条目、编辑、删除）在 VmBridge 层单测可跑。
4. **端到端（headless 或手动）**：`cd examples/ui/015-notes && auto r -r vm`，确认列表渲染、增删改响应正常。
5. **回归**：001/002/013/016 在 `render:vm` 下仍正常（Plan 323 验收 #6 收尾）。

## 备注

- 本计划**不**触碰 Plan 325-autovm 的跨模块字符串/enum 方法缺陷；若 Phase 0 确认那些仍坏且阻塞 015-notes，需先推 325-autovm（或并入），本计划 Phase 1 标 hard-dependency。
- worktree 必须用 sibling（`../auto-lang-327`），否则 `../auto-ai` 路径依赖断裂、整个 workspace 编译失败（已在 Plan 323 收尾时踩过）。
- 与 [[autoui-vm-option-b-reroute]] memory 记录的 follow-up 一致。
