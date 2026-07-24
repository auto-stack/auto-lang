# Plan 013 交接摘要（用于新会话续开发）

> **2026-07-24 续作更新**：阶段 3 又完成 **roles / skill / workflow_validator /
> orchestration/{budget,flow,handoff,pipeline,driver,mod} / agent（部分） /
> workflow（占位）/ lib.at** 共 12 个文件。全部 `auto trans ... rust` 通过。
> 当前阶段 3 剩余：**agent.at 的 ReAct 循环本体**（被若干解析器限制阻塞，
> 见下「本轮新发现的 Auto 语法限制」）与 **workflow.at 的实际移植**（已弃用
> 模块，已占位推迟）。详见文末「2026-07-24 续作进度」。

## 一句话状态

将 auto-ai 的 3 个 Rust crate 用 Auto 语言复刻。阶段 1（ai-config）+ 阶段 2（auto-ai-client）**全部完成并通过 cargo check**；阶段 3（auto-ai-agent）已完成 **24/26 文件**，剩余 agent.rs(918行) + workflow.rs(1181行) + roles.rs + skill.rs + workflow_validator.rs + orchestration/*(5文件) + lib.at/mod.at 共 **~4500 行待移植**。

## 仓库与分支

- **工作目录**：`D:\autostack\auto-lang\.worktrees\plan-013-b16`（master 分支，含全部 a2r 修复）
- **Rust 原版参考**：`D:\autostack\auto-ai\crates\`
- **Auto 语法指南**：`D:\autostack\skills\auto-lang-creator\skill.md`
- **计划文档**：`D:\autostack\auto-lang\.worktrees\plan-013-b16\docs\plans\013-auto-ai-port-to-auto.md`

### 构建

```bash
cd D:/autostack/auto-lang/.worktrees/plan-013-b16
cargo build --release --bin auto   # auto.exe 用于 transpile 和 VM 运行
```

### 验证方法

```bash
# transpile 一个 .at → .a2r.rs
./target/release/auto.exe trans --path crates/<crate>/src/<file>.at rust

# AutoVM 运行（纯 Auto 文件才有效；a2r-first 文件会报桥接错误，正常）
./target/release/auto.exe crates/<crate>/src/<file>.at

# cargo check 验证（需在 workspace 内创建临时 crate，含 ai-config/auto-val 等依赖）
```

## 已完成的文件清单

### 阶段 1：ai-config（6 文件，全部 cargo check 0 错误）✅

`crates/ai-config/src/`：tier.at, wire.at, provider.at, loader.at, validate.at, lib.at

### 阶段 2：auto-ai-client（3 文件）✅

`crates/auto-ai-client/src/`：error.at, daemon.at, lib.at

### 阶段 3：auto-ai-agent（24 文件已移植）✅

`crates/auto-ai-agent/src/`：
- **基础层**：error.at, role_def.at, relay.at, tool.at, memory.at, validate.at
- **builtin_roles/**（16 文件）：mod.at + assistant/coder/architect/tester/reviewer/documenter/advisor/planner/gofer/super_advisor/super_coder/super_tester/runner/translator.at
- **config/**（2 文件）：mod.at, role_config.at

## 待移植文件（按优先级）

| 文件 | Rust 行数 | 说明 | 复杂度 |
|---|---|---|---|
| `roles.rs` → `roles.at` | 395 | RoleRegistry（用 Map<str, Role> + names 键表） | 中 |
| `skill.rs` → `skill.at` | 476 | Skill/SkillRegistry/SkillTool | 中 |
| `workflow_validator.rs` → `workflow_validator.at` | 192 | 校验 workflow 步骤 | 低 |
| `agent.rs` → `agent.at` | 918 | **核心 ReAct 循环**（async、tool-calling） | 高 |
| `workflow.rs` → `workflow.at` | 1181 | workflow 引擎（deprecated） | 高 |
| `orchestration/mod.at` | 30 | 模块导出 | 低 |
| `orchestration/budget.at` | 166 | token 预算跟踪 | 中 |
| `orchestration/flow.at` | 162 | 流程定义 | 中 |
| `orchestration/handoff.at` | 225 | 角色交接 | 中 |
| `orchestration/pipeline.at` | 502 | pipeline 引擎 | 高 |
| `orchestration/driver.at` | 432 | pipeline 驱动 | 高 |
| `lib.at` | — | crate 根（re-export） | 低 |

## 关键 Auto 语法规则（移植时必须遵守）

### 必须遵守的（否则 transpile/解析失败）

1. **构造体返回必须 `return`**：`fn foo() Type { return Type(...) }` 不能省 `return`
2. **`use <stdlib>` 会报错**：不要写 `use json`/`use http`/`use fs`，直接全局调用 `json.parse()`/`http.request()`/`fs.exists()`
3. **`||` / `or` 在 if/for 条件里不可用**：用嵌套 `if/else` 替代
4. **`is` 分支不支持多语句块体**：`is x { Some(v) -> { stmt1; stmt2 } }` 会解析失败；用 `??` 提取值 + 函数级逻辑
5. **`is` 分支里的局部赋值失败**：`Some(v) -> limit = v` 不行；`Some(v) -> cfg.x = v`（字段赋值）可以
6. **`pub const` 不支持**：用公开函数返回常量值
7. **`routes`/`route` 是保留关键字**：不能用作字段名
8. **`ext Type has Spec { ... }` 不被解析**：必须用 `type X has Spec { fields + methods }` 内联实现 spec，非 spec 方法放 `ext X { ... }`
9. **VM Map 无 iteration API**：`for k,v in map` 静默产出 0 项；用并行 `List<str>` 键表
10. **跨文件 `use` 在独立 VM 运行不可见**：`use role_def: Role` 在 `auto a.at` 单独运行时报 Module not found；但 transpile 时正确译为 `use crate::role_def::Role`

### 应当遵守的（改善 a2r→Rust 质量）

11. **所有公开类型/枚举用 `pub type`/`pub enum`**：a2r 需要显式 `pub` 才能跨模块
12. **所有公开字段用 `pub`**（a2r 已在 standalone 模式自动加，但源码侧声明也好）
13. **`byte(u8)` 赋给 `int(i32)` 需改类型或手动转换**：order() 类函数直接返回 `int`
14. **桥接类型（auto_val 的 AutoStr）边界加 `.to_string()`**：auto_val 返回 AutoStr，非 String
15. **`HashMap.get(key)` 返回 Option**：用 `is result { Some(x) -> ... }` 解构，或 `?? default`

### async 映射

- `async fn foo()` → `pub fn foo() ~Result<T, E>`
- `.await` 保留 `.await`
- 调用：`client.complete(req).await`

### trait 对象

- `Arc<dyn Tool>` → `Arc<Tool>`（**尖括号**，不是圆括号！）
- `Box<dyn Role>` → `Box<Role>`
- a2r 生成 `Arc<Box<dyn Tool>>`（多一层 Box，功能正确）
- `Map<str, Arc<Tool>>` 作为字段类型可以正常 transpile

### spec 实现

```auto
// 正确：内联 type + has Role
pub type ConfigRole has Role {
    cfg RoleConfig
    base ?Role

    pub fn name() str {
        is self.cfg.name {
            Some(n) -> return n,
            None -> return self.base_name()
        }
    }
    // ... 其他 Role 方法 ...
}

// 正确：单独的 ext 块放非 spec 方法
ext ConfigRole {
    fn base_name() str { ... }
}
```

### 桥接文件（a2r-first）模式

当 .at 文件需要用 Rust crate 的类型（如 auto_atom/auto_val）时：

```auto
dep auto_atom
use.rust auto_atom
dep auto_val
use.rust auto_val
// 这些行让 a2r 生成 use auto_atom::*; use auto_val::*;
// AutoVM 无法运行此类文件（桥接类型未知），但 transpile + cargo check 可用
```

## a2r 已修复的 codegen 问题（本计划修的）

以下 a2r 修复已在 master 上（通过合并 `plan-013/a2r-b1-fixes` 分支 + 后续直接提交）：

1. enum derive 补 Eq/PartialOrd/Ord（安全时）
2. self.field 返回补 .clone()（E0507）
3. 本地类型不误加 crate 前缀（local_struct_types 预扫描）
4. for-loop 迭代器方法调用不加多余的 `&`
5. a2r_std 前导用裸路径（非 `auto_lang::a2r_std`）
6. Err 具体枚举错误不套 Box::new
7. Err(Ident) 重抛不套 Box
8. Some(int) 返回 ?uint 时加 `as u32`
9. Map.get 自动借用（仅 owned-String 参数）
10. 结构体字段 standalone 加 pub
11. 桥接 crate glob import（auto_val::*; auto_atom::*;）
12. Cover(Tag) 桥接绑定记录 + `*(*x).clone()` 双重 deref

## 新会话需要做的事

1. 读本文件了解全部上下文
2. 读计划文档 `013-auto-ai-port-to-auto.md` 了解完整缺陷记录
3. 读 `auto-lang-creator/skill.md` 了解 Auto 语法
4. 从待移植文件表按优先级开始移植
5. 每个文件写完后用 `auto trans --path <file> rust` 验证 transpile 通过
6. 建议先做 roles.rs(中) 和 skill.rs(中)，再做 orchestration/*，最后做 agent.rs(高) + workflow.rs(高)
7. 完成后写 lib.at 收尾
8. 全部完成后提交，更新计划文档状态

---

## 2026-07-24 续作进度（阶段 3 第二批）

### 已完成（全部 `auto trans ... rust` 通过）

| 文件 | 说明 | 备注 |
|---|---|---|
| `roles.at` | RoleRegistry（Map+并行 names 键表）+ RoleSummary/RoleDetail + load/resolve/list/get/save/delete | a2r-first（桥接 dirs/std::fs/std::path） |
| `skill.at` | Skill / SkillRegistry / SkillTool(has Tool) + frontmatter 解析 | a2r-first（桥接 fs/serde_json） |
| `workflow_validator.at` | Validator 枚举（tuple 变体）+ check/check_all/check_any | 纯 Auto，无桥接 |
| `orchestration/budget.at` | TokenBudget/BudgetStrategy/BudgetAction + BudgetTracker | 纯 Auto |
| `orchestration/flow.at` | FlowSpec/FlowStep/GateType/ExitRouting/GateDecision | 纯 Auto（struct 变体→tuple） |
| `orchestration/handoff.at` | HandoffDocument + 子记录 + render() | 纯 Auto（`to` 字段→`target`） |
| `orchestration/pipeline.at` | PipelineEngine 状态机（advance/submit_handoff/resolve_gate 等） | 纯 Auto（struct 变体→tuple，类型前置定义） |
| `orchestration/driver.at` | PipelineDriver + AgentFactory spec + drive 循环 | a2r-first（依赖 agent.at；泛型类型→spec 字段；闭包→命名函数） |
| `orchestration/mod.at` | 模块导出 | 无 `as` 别名（flow.GateDecision 直出） |
| `agent.at` | **部分**：Client spec / StreamEvent / AgentResult / ToolCallRecord / Agent 结构 + 访问器；run/run_stream 为 stub | a2r-first；ReAct 循环本体未移植（见下） |
| `workflow.at` | **占位**（已弃用模块，推迟移植） | 见文末「未完成与阻塞」 |
| `lib.at` | crate 根 re-export（排除 workflow） | |

附带修复：`relay.at` 的 `delegate(task str)` 中 `task` 是保留字→改名 `task_msg`。

### 本轮新发现的 Auto 语法限制（补充到上面的 15 条规则之后）

> 这些都是 `auto trans ... rust` 实际踩到的解析器/类型检查器限制，非移植
> 错误。每条都给出触发条件与规避方法。

16. **`task` / `to` 等保留字不能做参数名/字段名**
    - `fn f(task str)` / `HandoffDocument { to str }` 都会触发解析级联失败
      （报诡异的 "Expected term, got RBrace" / "field type mismatch"）。
    - 规避：改名（`task_msg`、`target`），在注释里标注 Rust 原名。
    - 已知受影响：`task`（task/actor 语法）、`to`（range 语法）、`routes`/`route`。

17. **方法调用不能直接做 `is` 的匹配对象**（条件性）
    - `is path.extension() { Some(x) -> return x == "at", ... }` 会让解析器
      误终止当前块、吞掉后续函数。但 `is path.len() { 0 -> ... }`（分支体是
      字面量返回）有时又能过——不稳定。
    - 规避（稳妥）：先把方法调用结果赋给局部变量，再 `is` 该变量：
      `let ext = path.extension(); is ext { ... }`。roles.at / skill.at /
      pipeline.at 全部采用此规避。

18. **构造体不能直接作为深层方法调用的参数**（类型检查器）
    - `self.step_history.push(StepRecord(...))` 报 "field type mismatch"。
    - 规避：先 `let rec = StepRecord(...)` 再 `push(rec)`。
    - pipeline.at 的 submit_handoff、driver.at 的 on_event(X.Y(...)) 都踩到。

19. **`if/else` 表达式不能作为结构体字段值**
    - `RoleConfig(tools: if t.is_empty() { None } else { Some(t) })` 失败。
    - 规避：先算到局部 `var`，再赋给字段。

20. **泛型类型定义不支持 spec 约束**：`type X<F has AgentFactory>` 报
    "Expected Gt, but found has"。
    - 规避：用 spec 类型字段做动态分发（`factory AgentFactory`），等价于
      `Box<dyn AgentFactory>`。driver.at 采用。

21. **函数不能声明返回 `fn(...)` 类型**，也不能在函数体内构造闭包
    - `fn build_cb() fn(StreamEvent) { ... }` 失败；`let cb = (ev) => {...}` 失败。
    - 规避：把回调存为结构体字段（`on_event fn(PipelineEvent)`，字段是允许
      的），或用命名函数引用做 no-op 默认值。driver.at 采用。

22. **方法的 `fn(...)` 类型参数后不能再跟其它参数**（不稳定）
    - `fn drive(task str, on_event fn(PipelineEvent))` 失败；单独
      `fn drive(on_event fn(PipelineEvent))` 可过。自由函数似乎不受此限。
    - 规避：把回调移到字段（driver.at 把 on_event 存为字段）。

23. **`.await?` 应写作 `.await.?`**（点号分隔）：`x.run_stream(...).await.?`。

24. **`let _ = expr`（下划线丢弃绑定）不支持**：解析器不认 `_` 作变量名。
    规避：直接不引用该参数（注释说明），或命名后不用。

25. **`use` 语句不支持 `as` 别名**：`use flow: GateDecision as FlowGateDecision`
    失败。规避：直出原名，或调用处用全限定 `flow.GateDecision`。

26. **自由函数需先定义后引用**（无前向引用）：在 `new()` 里引用的 no-op
    默认函数必须在文件靠前定义。driver.at 把 noop_event_handler /
    noop_stream_handler 提到顶部。

27. **条件里 `&&` 不可用**（与已记录的 `||`/`or` 同）——见 memory.at 既存
    问题（见下「未完成与阻塞」B17）。

### 未完成与阻塞

#### A. agent.at 的 ReAct 循环本体（阻塞于平台限制，plan 013 class B）

`agent.rs` 的 `run` / `run_stream` / `run_inner` / `build_request` 重度依赖：
- 泛型方法 `fn new<P: Role>(role P, ...)`（规则 20 同源）；
- `Arc<dyn Fn(StreamEvent)>` 回调参数（规则 21/22）；
- 函数体内的闭包（`move |ev| {...}`、`cancelled` 闭包，规则 21）；
- 每轮 `on_event(StreamEvent::Delta{text})` 深层构造（规则 18）。

当前 agent.at 只移植了**可移植的类型层**（Client spec 的 complete、
StreamEvent/AgentResult/ToolCallRecord、Agent 结构 + 访问器 +
truncate_tool_result），run/run_stream 为返回空结果的 stub。**恢复完整循环
需先在 a2r/解析器侧支持上述构造**（特别是闭包与 dyn-Fn 参数）。

建议的恢复路径（平台支持后）：
1. `Client` spec 补 `complete_stream`（dyn-Fn 参数）；
2. `run_inner` 把 `on_event` / `cancelled` / `seen` 做成字段或命名函数，
   规避闭包；
3. 每个 `on_event(X.Y(...))` 先 `let ev = X.Y(...)` 再发（规则 18）；
4. 取消检查用命名函数替代闭包。

#### B. workflow.at（已弃用，推迟）

`workflow.rs`（~1181 行，Plan 008 标记 deprecated，建议改用 PipelineEngine）
依赖 auto_atom/auto_val 桥接 + 未移植的 Agent 循环，且同样踩闭包/async 限制。
**因收益最低、已弃用**，本轮仅留占位文件（说明推迟原因）。恢复时机：
agent.at 循环移植完成后 + Auto 自举出原生 Atom 解析器后。

#### B17（新）. memory.at 既存回归——transpile 失败

交接摘要原称 memory.at「已移植、AutoVM 可运行」，但本轮 `auto trans` 发现
**transpile 失败**（offset 5958 "Expected term, got RBrace"）。根因疑为
trim() 里 `for end < self.messages.len() && self.messages[end].role == "user"`
的 `&&`（规则 27）——AutoVM 能跑但 a2r 不能译。**这是既存问题，非本轮引入**，
记档待修（与 A 类缺陷同性质，可单独修）。

### 下一步建议（新会话）

1. **平台侧**：在 auto-lang 的 a2r/解析器上补「闭包构造」「dyn-Fn 参数」
   「泛型方法」「`&&` 条件」支持——这是 agent.at 循环与 workflow.at 的共同
   硬阻塞。
2. **移植侧**（平台支持后）：按上面「恢复路径」补完 agent.at 的 ReAct 循环。
3. **既存修复**：memory.at 的 `&&` 条件（B17）可立即改为嵌套 `if`。
4. **回归验证**：全部完成后建一个临时 workspace crate（含 ai-config /
   auto-val 等依赖）跑 `cargo check`，把阶段 3 推进到与阶段 1 同等的
   「a2r→Rust 过 cargo check」验收线。

