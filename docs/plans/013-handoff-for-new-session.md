# Plan 013 交接摘要（用于新会话续开发）

> ⚠️ **2026-07-29 状态更新（请先读本条）**：本文件下文是 **2026-07-24 的历史
> 快照**，若干结论已被后续工作**超越/推翻**：
> - 阶段 3 已**全部 35 个 .at 文件移植完成**（非下文说的 24/26）。
> - **agent.at 的 ReAct 循环已完整移植**（轮询式，绕过 dyn-Fn/闭包——见下文
>   §C/D 的纠正，该判断是对的）。
> - 原 F.3 的"~344 个 cargo 错误阻塞"**已解决**：plan 372（3 个系统性 a2r 根因）
>   + plan 373（B1 类细节）+ plan-373-followup（manifest + post_process backport）
>   让 `crates/auto-ai-agent/rust/` 达到 **cargo check 0 错 + cargo run 跑通真实
>   ReAct 问答**。**Auto 版 ReAct MVP 已达成。**
> - 当前进度与剩余差距的权威记录已迁到 **`013-auto-ai-port-to-auto.md` 文末
>   「★ MVP 里程碑与剩余差距」**。**新会话请直接读那一节**，本文件仅作历史参考。
>
> 下文保留以备追溯（含 parser 限制根因分析、auto-coder 参考索引、流式路线图，
> 仍有价值）。

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

> ⚠️ **注意：本节若干条目已被文末「2026-07-24 解析器根因分析 + auto-coder
> 参考」纠错**——尤其规则 3（`\|\|` 其实可用）、规则 17（常是规则 16 的伪装）、
> 规则 21（根因已定位、可修）、规则 27（`&&` 可用，整条删除）。读本节时请
> 对照文末纠错表。原始记录保留以备追溯。

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

---

## 2026-07-24 解析器根因分析 + auto-coder 参考（重要纠正）

> 本节是对前面「本轮新发现的 Auto 语法限制」若干条目的**纠错**，以及一个
> 改变 agent.at 结论的重大发现。**读这份文档前请先读本节。**

### A. 三条事实性纠错（前面文档里的错误）

经最小复现 + parser.rs 源码追踪，前面记录的限制里有三条是错的：

| 原记录 | 纠错 | 证据 |
|---|---|---|
| 规则 3：`||`/`or` 在 if/for 条件里不可用 | **过时/错误**：当前 parser 支持 `if a=="x" \|\| b=="y"` | `auto-coder/coder/relay/turn.at:196` 就这么用；最小复现通过 |
| 规则 27：条件里 `&&` 不可用 | **错误**：`for a<10 && b<10 {...}` 合法 | 最小复现 `auto trans` 通过 |
| B17：memory.at transpile 失败因 `&&` | **误诊**：真因是 `remove_range(from int, to int)` 的参数名 `to` 是保留字（规则 16 同类） | 把 `to` 改名（如 `up_to`）后 memory.at transpile 通过；`&&` 无辜 |

**唯一真正普遍的坑是规则 16（保留字 `to`/`task`/`routes` 等做参数名/字段名）**，
而且它的报错极具迷惑性（"Expected term, got RBrace" / "field type mismatch"），
且**上下文相关、不稳定**（同代码换个无关变量名就过）——极易误判为别的根因。
本轮 relay/memory/driver/agent/handoff 至少 5 处都栽在这一个根因上。

### B. parser 限制的真根因（已定位到源码行）

| 规则 | 真根因 | 可修性 |
|---|---|---|
| 16（保留字做标识符） | 保留字在复杂位置被 lexer 当关键字 token，parser 期待 term 拿到关键字→级联报错 | **lexer/parser 可修**：至少把报错精确化（"to 是保留字"），低成本高收益 |
| 21（`fn(...)` 不能做返回类型/type alias） | `parse_type`→`parse_fn_type`（parser.rs:9399）对**字段/参数**位置生效；但**函数返回类型**与 **type alias RHS** 走了不识别 `Fn` 的路径 | **明确的 parser 缺陷，局部可修**（让那两处复用 `parse_type`）。`field fn(int)int` 与 `param fn(int)int` 都能过，唯独 `return fn(int)int` / `alias = fn(int)int` 失败 |
| 20（泛型类型不支持 `<F has S>`） | 设计性，"Expected Gt, but found has" | 改动较大 |
| 其余 17/18/19/22–26 | 多为 16/21 的伪装或小改 | 见前表 |

修复优先级建议（auto-lang 仓库）：① 规则 16 报错精确化（最高性价比）
② 规则 21 返回类型复用 `parse_fn_type`（解锁回调式 API）。

### C. 重大发现：agent.at 其实**可移植**——参考 auto-coder

前面文档说「agent.at ReAct 循环被闭包/dyn-Fn/泛型方法等平台限制阻塞」——
**这个判断被推翻了**。

**`D:/autostack/auto-coder/coder/relay/` 是一整套能跑的 Auto agent 引擎**
（auto-forge→auto-code-rs 的 Auto 化翻译，较 auto-ai 早一代）。实测
`relay/turn.at`（394 行，含完整 ReAct 循环）**在当前 parser 下 transpile 通过**。
它的架构选择恰好绕开了所有我误以为"不可逾越"的限制：

| auto-ai Rust 原版（我移植卡住的） | auto-coder Auto 版（能跑） |
|---|---|
| `complete_stream(req, on_event: Arc<dyn Fn(StreamEvent)>>` 回调式 | **`client.chat_turn(req) -> ToolChatResult{events: List<ToolChatEvent>}`**，循环 `for e<events.len()` 遍历——回调→事件列表轮询 |
| `move \|ev\|{...}` async 闭包 | 无闭包 |
| `tool.execute(args) ~Result<str,ToolError>` + `.await.?` | **`tool.execute(args_json) str`**——同步、返回 str |
| `is self.client.complete_stream(...)` | `is event {...}` 匹配局部变量（天然合规） |

**结论：不需要改 auto-lang parser，把 agent.at 的 API 从「回调推送」改成
「事件轮询」就能移植 ReAct 循环。** 这是架构适配，不是平台阻塞。

### D. 路线决策：先轮询跑起来，后增量演进到真流式（已定）

> 决策（2026-07-24）：走「先轮询、后流式」的增量路线。理由见下，不再
> "待定"。两条原始候选（套用 auto-coder / 先改 parser）合并进下面的阶段。

#### D.1 为什么回调式是"对的长期方向"（不要永久回退）

这不是口味之争。auto-ai 的流式是**真实产品需求**驱动的：
- daemon 确实是 **SSE 逐 token 推送**（`auto-ai-client/src/lib.rs:121` 的
  `resp.bytes_stream()` + 每个 delta 调一次 `on_event`）。
- 对一个 AI 编程助手，"边想边显示"vs"等 30 秒整段蹦出"是**可用性质变**，
  长答案场景尤其明显。所以 Rust 原版的回调式/流式是正确演进方向。

#### D.2 为什么短期可以先轮询——关键洞察

**真正的流式发生在 Layer 2（auto-ai-client），不在 Layer 3（agent）。**
agent.rs 的 `Arc<dyn Fn(StreamEvent)>` 只是把 client 的 SSE 流"穿透"给上层
UI。问题**只在穿透这一步**用了 Auto 暂不支持的 `dyn Fn`/闭包。

而且**轮询与回调之间只差一层缓冲带，不是推倒重来**：
- 轮询 = 事件追加进 `List<StreamEvent>`，跑完返回；
- 回调 = 事件到达即推送。
两者的事件种类、产生时机、ReAct 循环主体**完全相同**，变的只是"事件怎么
从循环内部传到外部"这一个接口。

**另一关键点**：轮询只影响**事件流**，不强制工具也退回同步。auto-coder 用
`execute(args str) str`（同步）是那代的局限；auto-ai 的
`execute(args JsonValue) ~Result<str, ToolError>`（async）在轮询路线下**可
保持不变**——`.await.?` 已验证可用。所以短期路线对工具层零改动。

#### D.3 三阶段路线图（每步都是增量，不重写）

**阶段 1 — 轮询式（✅ 已实施 2026-07-24，功能正确优先）**
- agent.at 的 `run`/`run_stream`/`run_inner` 已移植：收集事件到局部
  `List<StreamEvent>`（`events.push(ev)` 替代 `on_event(ev)`），跑完返回
  `AgentResult`。ReAct 循环主体（build_request、tool 执行 async、handoff、
  循环控制、loop-detect、max-turns、3 个 cancel 检查点）照 Rust 原版移植。
- API 形态：`Client` spec 的 `complete(req) ~Result<CompletionResponse,...>`
  保留；**未引入** `complete_stream`（需 dyn-Fn）。循环用 `complete`（非流式）。
- 验收：✅ `auto trans crates/auto-ai-agent/src/agent.at rust` 通过（16 fragments，
  仅有已知的 unbalanced-parentheses 假警告）。生成代码含完整的 run/run_stream/
  run_inner/build_request + is_cancelled/bump_seen/exec_tool 等辅助函数，
  `client.complete(req).await?` / `tools.execute(name,args).await` 正确译出。
- 已知限制：事件列表收集后未上抛给调用方（UI 不流式，整段显示）。

**阶段 2 — 事件队列（过渡）**
- 把 `List<StreamEvent>` 换成可追加的队列/通道（队列 = 带 push 的 List 或
  Auto 原生 channel），UI 侧循环拉取。首字节延迟降低（不必等整轮跑完）。
- ReAct 循环主体仍不变。

**阶段 3 — 真流式（长期）**
- 两条任选其一：
  - (a) auto-lang parser 修好规则 21（返回类型复用 `parse_fn_type`）+ 闭包
    支持，切回 Rust 原版的回调式 API（与原版完全一致）；
  - (b) 用 Auto 原生 `task`/`actor` + 消息传递（skill 文档的并发原语）替代
    dyn-Fn 回调——actor 之间天然是流式消息，且是 Auto 自举的正路。
- 选 (a) 还是 (b) 届时再定（取决于 parser 演进 vs actor 成熟度）。

#### D.4 技术债记账（明确"这是临时的"）

| 项 | 阶段 1 状态 | 目标（阶段 3） |
|---|---|---|
| 事件流 | 轮询（整段返回） | 真·流式（逐 token） |
| `Client.complete_stream` | 暂不移植 | 移植（dyn-Fn 或 actor） |
| UI 首字节延迟 | 高（=非流式） | 低 |
| 取消（cancel） | 只能在整轮间 | delta 间隙可取消 |
| 工具执行 | async（保持） | async（不变） |

**这表本身就是技术债清单**——阶段 1 完成后逐项核对，确保"临时"不被遗忘。

#### D.5 实施前的小准备（建议）

动手补 agent.at 前，建议先花一刻钟把 auto-coder 的 `relay/registry.at`、
`relay/profession.at`、`tools/*.at`（尤其 `execute str→str` 的契约）扫一眼，
做一份最小的「auto-coder → auto-ai 概念映射」：
`Profession→Role`、`ToolRegistry→ToolRegistry`、`chat_turn→complete`、
`ToolChatEvent→StreamEvent`、`TurnResult→AgentResult`。映射清楚后，阶段 1 的
移植基本是把 turn.at 的 `run_sync` 套进 auto-ai 的类型名。

### E. auto-coder 关键文件索引（供下次会话）

```
D:/autostack/auto-coder/coder/
  relay/agent.at        # AgentInstance（prompt 组装，无循环）
  relay/turn.at         # ★ ReAct 循环本体（run_sync），最关键参考
  relay/pipeline.at     # 编排状态机（与 orchestration/pipeline.at 同构）
  relay/handoff.at      # HandoffDocument
  relay/budget.at       # 预算跟踪
  relay/profession.at   # Role 的对应物
  relay/registry.at     # 工具/职业注册
  types.at              # ToolChatRequest/ToolChatEvent/ChatMessage
  tools/{bash,file_*,grep,mod,registry,spec_test}.at  # 工具实现（execute str→str）
  runtime/{agent,context,session,permission}.at       # 运行时
```

### F. 跑通 ReAct 问答的尝试与阻塞（2026-07-24，分支 plan-013/react-runnable）

目标：让 Auto 移植的 agent（ReAct 循环是 Auto 源、a2r 译来）对一个简单问题
真实跑通。结论：**组装架构验证可行，但卡在 a2r codegen 保真度（~344 错误），
属 B1 类、需 a2r 侧改进，非逐文件手修的合理范围。**

#### F.1 关键架构决策：选 A（HTTP 用真 Rust，agent 层用 Auto）

调研发现 a2r-std 的 `http` 只有 `post_sync`（硬编码 Anthropic 头，不匹配本地
aaid daemon 契约）。全栈自举（选项 B：扩 a2r-std http + Auto 重写 complete）是
独立工作。故走选项 A：agent 的 ReAct 循环是 Auto，HTTP 层用手写 Rust
（`impl Client for AiClient` 委托真 auto-ai-client）。**B 留作路线图后续。**

纯 AutoVM 跑不通真实问答（client 是 a2r-first + 跨文件依赖 VM 不可见），故验收
走 a2r→Rust→cargo。

#### F.2 已完成（分支 plan-013/react-runnable，已 checkpoint 提交）

- **memory.at**：修保留字 `to`→`up_to`（规则 16），现可 transpile。
- **rust/ 组装 crate**（`crates/auto-ai-agent/rust/`，照搬 auto-coder/coder/rust/
  已验证的模式）：
  - `Cargo.toml`：依赖真 auto-ai-client/ai-config（path 跨仓库）+ a2r-std/
    auto-atom/auto-val；`[workspace]` 隔离避免父 workspace 冲突；path 全指向
    主 auto-lang checkout（避免 package collision）。
  - `lib.rs`：**扁平模块布局**（关键——a2r 假设所有模块在 crate 根，子目录
    文件 config/orchestration/builtin_roles 全部提升到根）+ extern-crate 垫片
    （`pub mod auto_ai_client { pub use ::auto_ai_client::*; }` 等）+
    `JsonValue = serde_json::Value` 别名 + config.rs/orchestration.rs 聚合器。
  - `client_impl.rs`：手写 `#[async_trait] impl Client for AiClient`（HTTP 桥）。
  - `main.rs`：ReAct 问答入口（Assistant role + AiClient::with_url + run）。
  - `agent.rs`：Client trait 改 `#[async_trait]`（a2r 译出未定义的 `Future` 返回）。
- **依赖全部解析、结构正确**（这是本次主要成果——证明了组装架构可行）。

#### F.3 阻塞：~344 个 cargo 错误（a2r codegen 保真度，B1 类）

`cargo check` 报 344 错误，全是 a2r 生成的 Rust 不够正确，**非组装错误**：

| 错误码 | 数量 | 性质 |
|---|---|---|
| E0308 | 146 | 类型不匹配（杂项） |
| E0599 | 37 | 方法找不到（`Option` 当有 `.as_string()`、enum 当有 `.as_str()`——a2r 对 `?str`/Auto 方法分发的 lowering 错） |
| E0277 | 36 | trait 未实现 |
| E0422/E0425 | 27 | 名字找不到（级联） |
| E0614 | 13 | 对非值类型取字段（`?str`/Option lowering） |
| E0507 | 13 | 借用/move |
| E0573 | 10 | 期望 item 名（aggregator 残留问题） |
| E0782 | 9 | **spec 当裸类型用**：a2r 把 `Role` 当字段类型，该是 `Box<dyn Role>` |

**系统性根因**（修一类的杠杆点）：
- a2r 对 spec 的 lowering 不一致：`Arc<Tool>`→`Arc<Box<dyn Tool>>`（有 dyn），
  但裸 `Role` 字段→`Role`（缺 `Box<dyn>`，E0782）。
- a2r 对 `?T`（Option）的方法调用 lowering 错：把 Auto 的 `.foo`（Option 上）
  直译成 Rust `Option.foo`（不存在）。
- a2r 对 enum 的方法调用直译（enum 上调 `.as_str()`）。
- 杂项 `.clone()`/借用/move（E0308/E0507）。

**这些不是几行能改完的**——本质是弥补 a2r 应自己生成的代码。auto-coder 用一个
31KB 的 `fix_transpiled.py` 修同类问题。两条出路（待定）：
1. **写本项目专属的 fix 脚本**（扫系统性模式批量修）+ 手修零散；
2. **修 a2r 根因**（trans/rust.rs：spec→trait-object lowering、?T 方法分发、
   enum 方法分发）——从源头修好，所有未来 a2r 输出受益。

#### F.4 给下次会话的建议

- 优先级判断：若目标是"尽快看到问答跑起来"，**修 a2r 根因（出 2）性价比更高**
  ——修好 spec→Box<dyn> 和 ?T 方法分发这两类，预计能消掉大半错误。
- 若想先有"能跑的最小子集"：把 main 缩到只调 `Agent.run`（无工具、单轮），
  临时桩化/排除 tool/roles/orchestration 等 unused 模块，只让 agent+client+
  memory 过 check。放弃全 crate cargo check。
- 组装架构（F.2）已验证可行，无需重做；剩余纯 a2r 保真度工作。
- 选项 B（全栈自举：扩 a2r-std http + Auto 重写 complete）是更远期的独立路线。

#### F.5 ★ 根因已定位 → 详见 plan 372

后续 344 错误的根因已全部定位到 `trans/rust.rs` / `parser.rs` 的具体行号，
并写成独立计划：**`docs/plans/372-a2r-rust-correctness-fixes.md`**。

3 个 a2r 缺陷（均为 a2r 的错，非 .at 源码问题）：
- **A（最高杠杆）**：spec 跨模块/乱序解析 → `Type::User`（应 `Type::Spec`）。
  修法：Phase 1.5 预注册 spec（`rust.rs:12951-12961`，与 struct 同机制）。
- **B**：Option(?T) 方法调用未做 optional dispatch（`args.get().as_string()`）。
  助手 `as_string_opt` 已存在（`a2r_std.rs:322`）但从没被 emit。修法：
  `Expr::Dot` 分发加 Option-aware 守卫（`rust.rs:4214`）。
- **C（最孤立、风险最低）**：self-方法的 `i+1` auto-borrow 把 `.as_str()` 错加
  到 enum/struct 参数（`rust.rs:5791-5794`）。修法：改成类型感知的 str 参数检查。

**下次会话直接按 plan 372 实施**（顺序 C→B→A），每修一个 re-transpile + cargo
check 看错误下降。最终验收：`cargo check` 0 错 → `cargo run` 打印真实 LLM 回答。



