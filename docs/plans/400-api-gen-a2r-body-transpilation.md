# Plan 400: auto-man 后端 handler 生成走 a2r 转译核心（兑现 399 路线A）

> **状态**：📋 计划草案（2026-08-08）。等待立项评审。
> **前置**：[Plan 399](399-autoui-examples-sse-crud-extension.md)（已核心完成）。399 §4 明确把 `ApiEndpoint.body` 标为「路线A 预留，生产代码无读取者」；本计划就是兑现路线A。
> **分支**：建议 `plan400/a2r-handler-body`（基于 master，不与 `plan399/*` 混）。
> **动机**：auto-musk dogfooding 发现 `auto run` 无法一键拉起全栈——根因是 `generate_api_rs` 对 `api.at` 用手写 CRUD 模板而非 a2r 转译器，导致复杂后端（含真实业务逻辑、`use.rust`、daemon 调用）的函数体全部丢失。详见 §1。

---

## 1. 为什么需要本计划

### 1.1 现状：两条分裂的后端 codegen 路径

auto-lang 里存在**两套并存、语义分裂**的 Rust 后端代码生成路径：

| 路径 | 入口 | 机制 | `use.rust` | 函数体 | 适用 |
|---|---|---|---|---|---|
| **真 a2r**（`auto trans rust`） | `target.rs:618` `transpile_rust` / `:789` `transpile_rust_project_multi` | 逐行 AST→Rust 转译 | ✅ 完整保留 | ✅ 完整转译 | 任意 `.at`（含复杂逻辑） |
| **api_gen 模板**（`auto run`/`auto build` 的 `api.at→api.rs`） | `api_gen.rs:1081` `generate_api_rs` | 手写 CRUD 模板字符串拼接 | ❌ 静默丢弃 | ❌ 套 `Arc<Mutex<Vec<T>>>` 模板 | 仅 in-memory CRUD（015/017 类） |

讽刺的是：`api_gen.rs` 已经为 `db.at→db.rs` 调用了真 a2r（`transpile_db_to_rs:314` 调 `transpile_rust`），但生成 `api.rs` 的 handler 时却**绕过** a2r，自己拼模板。同一个文件里，db.rs 走 a2r，api.rs 走模板——这就是分裂的根源。

### 1.2 实测证据（auto-musk dogfooding，2026-08-08）

以 auto-musk 后端的 `auth_login` 为例，同一函数两种产物天差地别：

**`auto trans rust`（真 a2r）产物** —— `backend/crates/musk/src/auto_generated/server.rs`：
```rust
pub async fn auth_login(s: State<AppState>, body: Json<LoginRequest>) -> Response {
    let username: String = body.username.clone();
    let pair = auth_login_result(&s, username.clone(), body.password.clone());
    let token: String = pair.0.clone();
    let role: String = pair.1.clone();
    if (token.len() as i32) > 0 {
        return ok_response(LoginResponse { token: token.to_string(), user: UserInfo { username: username.to_string(), role: role.to_string() } });
    }
    return err_response("invalid credentials", 401);
}
```
→ 参数、函数体、`use.rust` 导入**全部完整保留**。

**`auto run`（api_gen 模板）产物** —— `examples/rust-workspace/auto-musk-back/src/api.rs`：
```rust
pub async fn auth_login(State(db): State<Db>, Json(input): Json<CreateAuthUserInput>) -> JsonResponse<AuthResponse> {
    let mut items = db.lock().unwrap();
    let new_id = items.iter().map(|n| n.user_id).max().unwrap_or(-1) + 1;
    let item = AuthUser { user_id: new_id, username: input.username, password: input.password, ..Default::default() };
    items.push(item.clone());
    // ...（CRUD 模板，auth 逻辑全丢）
}
```
→ 参数坍缩成 `State<Db>`，函数体被替换成 `db.lock()` + push 的 CRUD 模板，真实的 auth 逻辑（调 `auth_login_result`、token 判断、错误返回）**全部丢失**。

**`use.rust` 处理分裂实测**：在 `api.at` 加 `use.rust std::collections::HashMap`，`auto build` 生成的 `api.rs` 里 `HashMap=0`（丢弃）；同样语句放进 `db.at`，生成的 `db.rs` 里 `use std::collections::HashMap;` 完整保留。

### 1.3 影响

- **`auto run` 无法一键拉起 auto-musk 全栈**：前端生成 ✅，后端 ❌（90 个编译错误，类型坍缩 + 字段名 `type` 未转义）。当前只能用「`musk serve`（真后端，`backend/crates/musk/auto-src/` 经 `auto trans` + `cargo build`）+ 前端 dev server」双服务方式。
- **限制了 AutoUI 后端的表达力**：任何超出 in-memory CRUD 的后端（daemon 调用、文件持久化、relay 编排、复杂鉴权）都无法被 `auto run` 正确生成。开发者被迫把业务逻辑塞进 `db.at`（走 a2r），而 `api.at` 只能写 `return db.FN(args)` 薄委托——这是 399 路线B 的结构约定，但对真实应用过于苛刻。
- **违背"单一真源"理念**：auto-musk 后端 `auto-src/*.at`（8044 行）已经用 `auto trans` 验证了 a2r 能完整转译它们，但 `auto run` 却用不上这份能力。

### 1.4 目标

让 `generate_rust_server`（`api_gen.rs:565`）在生成 `api.rs` 的 handler 时，**对 `api.at` 的 `#[api]` 函数体调用真 a2r 转译**（`transpile_fn`），而非套 CRUD 模板。兑现 399 §4 预留的路线A。

---

## 2. 与 Plan 399 的关系（前置基线）

399 已经为本计划铺好了大部分基建，**400 是 399 路线A 的兑现，不是从零开始**：

| 399 已落地（400 复用） | 位置 | 说明 |
|---|---|---|
| `ApiEndpoint.body: Option<Body>` 字段 | `api/types.rs`（399 CRUD 第1步） | 捕获 `#[api]` 函数体 AST，但生产代码无读取者（§4 标注预留） |
| `extract_endpoint` 捕获 body | `api/mod.rs`（399 CRUD 第1步） | 之前一直丢弃 `fn_decl.body`，现已捕获 |
| `RustTrans::transpile_fn` 单函数转译入口 | `trans/rust.rs`（399 CRUD 第2步） | 复用 `fn_decl`，有 `trans_incremental` 先例。单测 `test_transpile_fn_single_function` |
| `RustTrans::register_type` 预注册 struct | `trans/rust.rs`（399 CRUD 第2步） | 让构造体在转译时可解析 |
| `transpile_db_to_rs` db.at 整文件 a2r | `api_gen.rs:301`（399 CRUD 第3步） | 证明 a2r 核心可服务于 api_gen；含 `post_process_db_rs` 后处理 |
| `extract_db_fn_from_body` 从 body 取 db fn 名 | `api_gen.rs`（399 §8） | 路线A 的轻量版——只取函数名不转译体。400 要更进一步 |

**399 走路线B 而非路线A 的原因**（`399:78`）：015/017 的 `api.at` 含 `use db` → `try_full_parse` 失败 → 回退 `extract_api_lenient`（正则）→ `endpoint.body` 为 `None`。路线A 需要 body，对这两个示例当时不成立。但 **399 §7 修复了 parser 栈溢出后**，015/017 的 api.at full_parse 在默认栈已能成功（8/8 端点有 body，见 `399:148`）。**所以路线A 的第一道障碍已被 399 清除**——400 可以直接利用 full_parse 成功后的 body。

**400 取代 399 路线B 的什么**：`resolve_db_call`（`api_gen.rs:998`）+ `db_fn_candidates`（命名启发式）+ `extract_db_fn_from_body`（取名不转体）。这些是"没有 body 时的妥协"，有了真 body 转译后，对非薄委托的端点就能直接生成真实逻辑，而非依赖"api.at body 全是 `return db.FN(args)`"的结构约定。

> **范围边界**：db.rs 整文件转译**已经**走 a2r（399 CRUD 第3步），400 不重做。400 聚焦 **api.rs handler 体层**：让 `generate_api_rs` 的 handler 函数体从 a2r 转译得来，而非模板拼接。

---

## 3. 实施阶段

### Phase 1（M1）：解决 api.at full_parse 障碍 + body 可用性验证

**问题**：399 §8 指出 `try_full_parse` 对含 `use db` 的 api.at 失败。虽 §7 修了栈溢出，但需确认 full_parse 现在对各种 api.at 都能拿到 body。

- **1.1** 编写诊断测试：对 015-notes / 017-chat / auto-musk 的 `api.at` 跑 `try_full_parse`，记录每个端点的 `endpoint.body` 是否为 `Some`。若仍有 `None`，定位根因（是 `use db` 还是别的语法）。
- **1.2** 若 full_parse 仍失败：评估是否让 `ApiExtractor`（`api/mod.rs:112`）在 lenient 回退路径也尽量捕获 body（目前 lenient 完全不抓 body）。或修复 full_parse 对 `use <module>` 的处理。
- **1.3** 建立"body 可用率"基线：记录 M1 时各示例的 body 捕获率，作为后续验收对照。
- **验收**：015/017/auto-musk 的 api.at full_parse 成功，所有 `#[api]` 端点 `body` 为 `Some`。若有无法 full_parse 的合法 api.at，登记为已知限制（非阻断）。
- **降级**：若某端点 body 仍为 `None`，handler 回退到路线B（db 委托）或 CRUD 模板——不阻塞，但记录。这正是 399 混合状态硬检查（§10/Phase 13）保护的场景。

### Phase 2（M2）：handler 体 a2r 转译（核心）

**目标**：`generate_api_rs`（`api_gen.rs:1081`）的主循环里，当 `endpoint.body` 为 `Some` 时，handler 函数体改由 `transpile_fn`（a2r）生成，而非 CRUD 模板。

- **2.1** 在 `generate_api_rs` 引入 body 转译分支：当 `endpoint.body.is_some()` 时，调 `RustTrans::transpile_fn`（399 已实现）转译函数体，得到 Rust 语句序列。
- **2.2** handler 签名处理：a2r 转译的函数体可能自带 `State`/`Path`/`Json`/`Query` 参数（如 auto-musk 的 `s State<AppState>, body Json<LoginRequest>`）。需要协调 a2r 产物与 axum handler 签名约定的衔接：
  - 评估：a2r 转译整个 `fn`（含签名）是否能直接作为 axum handler？若能，`generate_api_rs` 只需转译整个 fn + 注册路由，省去手写签名。
  - 若 a2r 的签名产物与 axum `Handler` trait 有出入（如返回类型），在 `generate_api_rs` 做最小修补（参照 `post_process_db_rs` 的后处理模式）。
- **2.3** 类型/导入收集：a2r 转译 body 可能引入新的 `use` 语句（`endpoint` 的 `use.rust`）。当前 `generate_api_rs` 硬编码 `use axum::{...}`；需改为收集所有 endpoint 的 `use.rust` + a2r 转译所需的 std/外部 crate 导入，合并去重写入 api.rs 头部。
- **2.4** 与 SSE 的协调：含 `~Stream<T>` 的端点，a2r 转译 body 会包含 `yield`/mpsc 逻辑。确认 399 第2步的 SSE handler 生成（`events.rs` 广播总线）与 a2r body 转译不冲突。auto-musk 的 SSE（mpsc + daemon）与 017-chat（内置 bus）机制不同——a2r 应忠实转译 auto-musk 风格，而非套 017 的 bus。评估是否需让 SSE 端点也走 body 转译（而非 399 的 events.rs 注入）。
- **2.5** 后处理对齐：参照 `post_process_db_rs`（`api_gen.rs:341`，399 CRUD 第3步），为 api.rs 的 a2r 产物做必要的后处理（类型可见性、生命周期、deref 等）。尽量复用 db.rs 已有的后处理函数，避免重复。
- **验收**：
  - 新增单测：含复杂 body（分支/循环/`use.rust`/extern 调用）的 api.at → api.rs handler 体完整保留逻辑。
  - 015/017 回归：handler 仍能正确委托 db.rs（body 是薄委托时，a2r 转译出 `crate::db::FN(args)`，与路线B 产物等价）。
  - a2r 327 测试不回归。
- **降级**：若某个 a2r 产物无法编译（如 auto-musk 的 `Arc<dyn Client>` 这类复杂 trait object），该端点回退路线B/模板 + 登记限制。Phase 1 的混合状态硬检查保护正确性。

### Phase 3（M3）：多 back/*.at 文件支持

**目标**：支持 `src/back/` 下多个 `.at` 文件（auto-musk 后端有 27 个），而非只有 `api.at` + `db.at`。

- **3.1** 参考 `target.rs:789` 的 `transpile_rust_project_multi`（多文件 a2r 模式），让 `generate_rust_server` 遍历 `src/back/*.at`，对每个含 `pub fn` 的文件走 a2r 转译，生成对应的 `.rs` 模块。
- **3.2** 模块组装：生成的多个 `.rs` 文件需在 `main.rs` 里 `mod` 声明 + 路由注册。评估 auto-musk 的 `build_router`（`server.at:492`）模式——它在一个函数里 `app.route(...)` 串联所有路由。让 a2r 转译 `build_router` 整体，而非逐端点拼路由。
- **3.3** `extern fn` 委托机制：auto-musk 后端大量用 `extern fn foo() ...` 委托手写 Rust。当前 AutoLang **没有 `extern fn` 语法**（`ast.rs:185` 的 `Stmt` 无 Extern 变体）。这需要语言层扩展——评估：
  - (a) 加 `extern fn` 语法（parser + ast + a2r 透传成 Rust 的 `extern "C"` 或直接当成未实现 fn 声明）；
  - (b) 用现有机制表达（如 `use.rust crate::server::auth_login_result` + 普通调 expr）；
  - (c) 后处理占位（a2r 转译时把未定义符号收集起来，生成 `extern` 块或 link 指令）。
- **验收**：auto-musk 的 `src/back/` 若放入完整后端源，`auto build` 能生成可编译的后端工程（允许 extern 委托到手写 Rust 的部分用占位/stub）。
- **降级**：`extern fn` 是大改动，Phase 3 可先不做，只支持单 `api.at` + `db.at`（Phase 2 成果），多文件 + extern 留后续。auto-musk 的完整后端对接可作为 Phase 3 的验收靶子，但不强求一次性完成。

### Phase 4（M4）：回归验证 + auto-musk 全栈一键跑

**目标**：端到端验证——`auto run` 在 auto-musk 工程一键拉起全栈。

- **4.1** 015/017 回归：playwright 测试全绿（017 9/9），015 cargo build 通过。
- **4.2** auto-musk 验证（若 Phase 3 完成）：在 auto-musk 根目录 `auto run`，确认：
  - 前端 vue 工程生成 ✅（现有能力）
  - 后端 Rust 工程生成 ✅（Phase 2/3 新能力）
  - 后端 `cargo build` 通过 ✅
  - 后端启动 + 前端 dev server 启动 ✅
  - 登录/specs/wiki/chats 基本功能可用 ✅
- **4.3** a2r + auto-man 全量测试不回归。
- **验收**：`auto run` 在 auto-musk 一键跑通全栈（或明确记录哪些端点因 extern/复杂 trait 回退，仍可用 musk serve 兜底）。

---

## 4. 关键技术决策

1. **a2r 转译整个 fn vs 仅 body**：倾向**转译整个 fn**（含签名）——因为 axum handler 的签名（`State`/`Path`/`Json`/`Query` 提取器）本身就在 api.at 的 `#[api]` 函数参数里，a2r 已能转译（auto-musk 的 `auto_generated/server.rs` 证明）。这样 `generate_api_rs` 退化为"转译每个 fn + 注册路由"，大幅简化。需验证 a2r 产物的签名满足 axum `Handler` trait bound。
2. **CRUD 模板是否保留**：保留作为**回退**（body 为 None 或 a2r 失败时）。不删除——向后兼容无 body 的简单端点 + lenient 解析路径。但默认优先 a2r。
3. **路线B（db 委托）是否保留**：保留。当 body 是薄委托（`return db.FN(args)`）时，a2r 转译出的就是 `crate::db::FN(args)`，与路线B 等价——无需特殊区分，a2r 自然处理。路线B 的启发式可作为 a2r 失败时的二级回退。
4. **`use.rust` 收集策略**：从 `ApiModule` 的 `Stmt::Use(UseKind::Rust)` 收集（当前 `ApiExtractor:115` 丢弃了）。改动 `ApiExtractor` 让它也保留 `use.rust` 语句，或单独扫一遍源文件提取。
5. **后处理复用**：`post_process_db_rs`（`api_gen.rs:341`）的 6 类修补对 api.rs 同样适用（类型可见性、生命周期、deref、to_string、i64）。抽成共享函数，db.rs 和 api.rs 都调用。399 Phase 11 已将其中部分根治到 a2r 层（P11.1-P11.4），剩余后处理（P11.5/P11.6）继续共享。

---

## 5. 风险登记

| 风险 | 级别 | 降级路径 |
|---|---|---|
| a2r 产物的 handler 签名不满足 axum `Handler` trait（如返回类型、生命周期） | 🟡 | 后处理修补（参照 db.rs）；极端情况手写签名 + a2r 只转译 body |
| `extern fn` 语言扩展工作量超预期（Phase 3） | 🟡 | Phase 3 拆出独立计划；Phase 2 先落地（单 api.at + db.at 已有价值） |
| a2r 转译 auto-musk 的 `Arc<dyn Client>`/mpsc 等复杂模式有缺陷 | 🟡 | 该端点回退路线B/模板 + 登记；auto-musk 已有 `auto trans` 手动产物兜底 |
| 多文件 back/*.at 的模块组装/路由注册复杂 | 🟡 | 参照 auto-musk `build_router` 模式；先支持单文件再扩展 |
| `use.rust` 收集不全导致编译错误 | 🟢 | 后处理扫描 a2r 产物里未解析的符号，补声明 |
| 回归 015/017（body 是薄委托时行为变化） | 🟢 | a2r 转译薄委托应等价于路线B；单测锚定（`test_handler_calls_db_for_chat` 等） |

### 5.1 auto-musk dogfooding 发现的关联 codegen 问题（非本计划核心范围）

以下问题在 auto-musk dogfooding 中发现，属**前端 vue codegen**（`ui_gen/vue.rs`）而非后端 a2r（本计划范围），但同属"dogfooding 暴露的 codegen 缺陷"，记录于此供后续跟进：

| 编号 | 问题 | 影响 | 当前处置 | 根因定位 |
|---|---|---|---|---|
| DF-1 | **嵌套 if 表达式在 style 绑定中被压平**：`style: if A { if B { "x" } else { "y" } } else { "z" }` 生成 `A ? '' : 'z'`（内层 if 丢失，true 分支变空字符串） | 选中态 class 丢失 → 布局/样式错乱（auto-musk 的 session 列表项点击后标题变居中） | auto-musk 侧简化为单层 if 绕开（`bde65a2`） | `ui_gen/vue.rs` style/attr 绑定的 if-expr 转换逻辑只取外层条件，未递归处理嵌套 if 的分支。需在 vue codegen 层修复（支持嵌套三元 `A ? (B ? 'x' : 'y') : 'z'`） |

> **注**：DF-1 属 vue codegen 范畴，不在本计划（后端 a2r）的实施范围内，但登记于此避免遗忘。修复时应在 `ui_gen/vue.rs` 加单测（嵌套 if → 嵌套三元），并可考虑纳入 a2vue golden 测试覆盖。

---

## 6. 验收标准

1. **handler 体保真**：含复杂业务逻辑（分支/循环/daemon 调用/`use.rust`）的 `#[api]` 函数，`auto build` 生成的 api.rs handler **完整保留函数体逻辑**（对齐 `auto trans rust` 的产物质量）。
2. **`use.rust` 保留**：api.at 的 `use.rust` 语句在生成的 api.rs 里完整出现（不再静默丢弃）。
3. **回归零破坏**：015-notes / 017-chat 现有测试全绿（017 playwright 9/9，015 cargo build，a2r 327 测试，auto-man lib 测试）。
4. **auto-musk 全栈（若 Phase 3 完成）**：`auto run` 在 auto-musk 一键拉起前后端，基本功能可用。
5. **零 drift**：重新 `auto build` 后 diff 产物 = 0（确定性生成）。

---

## 7. 不做什么（范围边界）

- **不改 db.rs 转译路径**：399 已让 db.at→db.rs 走 a2r，本计划不重做。
- **不删除 CRUD 模板 / 路线B**：保留作为回退，向后兼容。
- **不做 vm 后端**（`AUTO_BACKEND_IMPL=vm`）：本计划聚焦 rust 后端的 a2r 接通。
- **不做前端 codegen 改动**：前端 vue 生成不受影响（§5.1 的 DF-1 等 dogfooding 发现的前端 codegen 问题登记但不实施）。
- **`extern fn` 语言扩展**：若 Phase 3 评估为大局改动，拆独立计划，不阻塞 Phase 1/2。

---

## 8. 实施日志

| 阶段 | 内容 | 提交 | 验收 |
|---|---|---|---|
| _待启动_ | — | — | — |

---

## 参考

- **Plan 399**（前置）：`docs/plans/399-autoui-examples-sse-crud-extension.md` —— §4 路线A 预留（`:122-123`）、CRUD 第1-3步基建、§7 栈溢出修复、§8 body-based 解析、Phase 11 a2r 根治。
- **auto-musk 发现**：`docs/plans/022-frontend-auto-ization.md` §8 D 类 + `docs/plans/KNOWN-DEBT-AND-RISKS.md` 的 022 条目。
- **关键代码位置**（auto-lang master）：
  - `crates/auto-man/src/api_gen.rs:1081` — `generate_api_rs`（手写 CRUD 模板，零 a2r 调用）← **本计划改造核心**
  - `crates/auto-man/src/api_gen.rs:565` — `generate_rust_server`（编排入口）
  - `crates/auto-man/src/api_gen.rs:301` — `transpile_db_to_rs`（唯一调 a2r 处，先例）
  - `crates/auto-man/src/api_gen.rs:998` — `resolve_db_call`（路线B 启发式，保留为回退）
  - `crates/auto-lang/src/api/mod.rs:112` — `ApiExtractor`（丢弃 `use.rust`，需改）
  - `crates/auto-lang/src/trans/rust.rs` — a2r 核心（`transpile_fn` / `transpile_rust`，399 已验证）
  - `crates/auto-man/src/target.rs:618,789` — `transpile_rust` / `transpile_rust_project_multi`（多文件先例）
