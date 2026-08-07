# Plan 399: AutoUI 示例扩展 — SSE 多事件 + CRUD 智能扩展（路径 B）

> **状态（2026-08-07）**: ✅ 完成。第 1-3 步（SSE 端到端 + CRUD 扩展数据层 + db.rs 转译）+ 第 4-5 步（handler 调 db.rs + 状态模型统一）全部落地并验证。
> **分支**: 第 1-3 步在 `auto-ui-examples`（已合并 master `c175e9d8`）；第 4-5 步在 `plan399/handler-db-state`（worktree `D:/autostack/auto-lang-autoui`）。
> **动机**: 调研 016-027 全部是「单文件静态玩具」，015-notes 是唯一完整 App。本计划把 017-chat 升级为首个 SSE 实时聊天 App，并在过程中打通 SSE 多事件 codegen + 修复后端 CRUD 模板丢弃业务逻辑的根因（mine:false bug）。

---

## 背景

### 调研结论（016-027 现状）
- 016-027 全部是单文件静态玩具（无后端、散装变量 `msg1..msg5`、no-op handler），与 015-notes（5 模块前端 + 2 模块后端 + playwright）差一个量级。
- "三版本"（vue/vm/rust）只有 015 基本通；016-027 既无 rust 版也无 vm 验证。
- SSE 端到端从未被任何示例走通（codegen 基建存在但硬编码单契约）。

### mine:false bug 根因（CRUD 扩展的触发点）
017-chat 发现 POST 返回 `mine:false` 而非 db.at 写的 `mine:true`。根因：rust 后端生成器 `api_gen.rs` 的 handler 是元数据驱动 CRUD 模板，只填 id/params/time，**完全丢弃函数体**（`extract_endpoint` 不读 `fn_decl.body`），db.at 的业务函数也只挖种子字面量、无视逻辑。`mine` 落到 `..Default::default()` = false。VM 后端不受影响（直接 `run_file` 忠实执行）。

---

## 已完成（第 1-3 步）

### 第 1 步：SSE 多事件 codegen — 前端 ✅
（commit `45361b10`，分支原有基础 + `e355955a`/`4601aa6b` 补全）
- `StreamEndpoint` 加 `discriminator`/`variants` 字段（`aura/types.rs`）
- `resolve_stream_variants` 解析 `pub tag T{...}` 声明（`ui_gen/api.rs`）
- vue store composable 数据驱动 dispatch + 多端点 per-path guard（`ui_gen/vue.rs`）
- legacy fallback 保留（空 variants → command_output/command_result 向后兼容）

### 第 2 步：017-chat 完整 App + 后端 SSE 生成 ✅
（commit `3b50f44a`/`74557e10`/`e355955a`）
- 017-chat 升级：pac.at(api:rust) + 后端(api.at Message+pub tag ChatEvent+~Stream / db.at) + 前端(store+message_thread+composer+app+types)
- 后端 SSE 生成（`api_gen.rs`）：`~Stream<T>` → `Sse<impl Stream>` handler + `events.rs` 广播总线 + POST 广播 + Cargo deps + start_api_server 首次自动生成 + workspace 时机修复
- **playwright 8/8 全绿**（T1-T8：加载/双气泡/发送后右侧/空消息/清空/SSE 跨标签页推送/EventSource 建连/控制台无错误）
- mine 兜底：前端按 `sender=="You"` 派生（CRUD 模板 mine 不可靠）

### 第 3 步：codegen bug 修复 ✅
（commit `4601aa6b`）
- callback relay：handler body 的 `props.on_xxx()` 重写为 `emit('<Pascal>', ...)`（修复 `props.on_send is not a function`）
- `~Stream` return-体不包 `stream!` 宏（`scan_body_has_yield` 检测，仅 yield 体才包）

### CRUD 智能扩展（路径 B）第 1-3 步 ✅
（commit `66d07fee` 第1-2步 + `1788e9b8` 第3步）

**第1步 — ApiEndpoint 捕获函数体 AST**
- `ApiEndpoint` 加 `body: Option<Body>` 字段（`api/types.rs`）
- `extract_endpoint` 捕获 `fn_decl.body`（`api/mod.rs`，之前一直被丢弃）
- 单测 `test_extract_endpoint_captures_body`

**第2步 — a2r 单函数转译入口**
- `RustTrans::transpile_fn`（复用 `fn_decl`，`trans_incremental` 先例）
- `RustTrans::register_type`（预注册 struct 让构造体能解析）
- 单测 `test_transpile_fn_single_function`

**第3步 — db.rs a2r 整文件转译 + 后处理**
- db.at 含 `pub fn` 时，`transpile_db_to_rs` 整文件 a2r 转译成 db.rs（复用 Tauri 先例）
- `post_process_db_rs` 后处理修 6 类 a2r 缺陷：
  - `crate::api` → `crate::types`（类型可见性）
  - `List<T>.new(expr)` → `expr`（剥包装，括号配平 `strip_collection_new`）
  - `&[T]` 返回 → `Vec<T>` + `.clone()`（生命周期 `fix_borrowed_slice_returns`）
  - `*G.lock().unwrap().push` → 去 deref（guard 解引用）
  - str 参数赋 String 字段 → `.to_string()`（`append_tostring_for_str_fields`）
  - `id: *NEXTID.lock()` 加 `as i64`（types.rs int→i64）
- Cargo.toml has_db 时加 `once_cell`（a2r 全局变量 Lazy）
- main.rs has_db 时声明 `mod db;`
- trans/rust.rs: `List<T>.new(expr)` 剥离特例（GenName/Ident/Bina<Lt> 三种 receiver）
- **验证：db.rs 编译通过，含 create_message 真实 `mine:true` 逻辑**

---

## 已完成（第 4-5 步）

### 第 4-5 步：handler 调 db.rs 函数 + 状态模型统一 ✅
（分支 `plan399/handler-db-state`）

**问题**：db.rs 可编译但未被 handler 使用。POST 用 `State<Db>` 模板（`..Default::default()`）→ `mine:false`；GET 读 `State<Db>`（main.rs 种子）与 POST 写 db.rs lazy_static 两套状态不一致。

**方案：路线 B（db 委托模式识别）** —— 不依赖 `endpoint.body`。关键障碍：017-chat/015-notes 的 api.at 含 `use db` → 全量解析失败 → 回退 `extract_api_lenient`（正则）→ `endpoint.body` 为 `None`。而路线 A（转译 body）需要 body，对这两个示例不成立。路线 B 利用「api.at body 全是 `return db.FN(args)` 单行委托、db.rs 全覆盖」的结构约定，按端点签名推断要调的 db.rs 函数。

**实施（`auto-man/src/api_gen.rs`）**：
1. **db 函数清单提取** — `generate_rust_server` 在 `has_db` 时从 db.rs 转译产物用 `extract_db_fn_names`（正则 `^\s*pub\s+fn\s+(NAME)\s*\(`）提取函数名集合，透传给 `generate_api_rs`。
2. **端点→db 函数映射器** — `db_fn_candidates` 按优先级生成候选名：精确名 → CRUD 动词归一（`list_X`→`all_X`、`send_/create_/add_X`→`create_X`、`get_/find_X`→`find_X`、`update_/edit_/move_X`→`update_X`、`delete_/remove_X`→`delete_X`、`toggle_X`、`search_X`）。`resolve_db_call` 取首个在 db_fns 中的候选，并按 extractor 映射参数：path→裸 ident、query→`query.X`、body→`input.X`；**str 类型参数借用**（`&input.X`）——a2r 把 `str` 转成 `&str`，extractor 持 `String`，deref coercion 桥接。
3. **handler 生成重构** — `generate_api_rs` 主循环：命中 db 委托时**不 push `State<Db>`**，体改为 `crate::db::FN(args)`（注意：api.rs 用 `crate::db::` 而非 `db::`，因为 `mod db` 在 main.rs 声明）；POST 命中时仍保留 SSE 广播；Option 返回用 `.map(JsonResponse).ok_or(NOT_FOUND)`。未命中 → 回退模板 + 警告（混合状态保护）。
4. **状态统一** — `all_endpoints_covered` 判定全委托；`generate_main_rs` 加 `db_full_cover: bool` 参数，为 true 时**去掉 `use api::Db`/`Arc`/`Mutex`/`with_state`**（种子由 db.rs 的 `Lazy::new(vec![...])` 承担）；为 false（无 db 或部分回退）时保留旧 `State<Db>` 路径。

**验证（代码层，全部通过）**：
- `test_handler_calls_db_for_chat`：017-chat api.at → handler 委托 `crate::db::create_message(&input.sender, &input.text)`，无 `State<Db>`，POST 后有 `events::broadcast`
- `test_handler_calls_db_for_notes_regression`：015-notes 9 端点全转 db.rs（list→all_notes、get→find_note、create→create_note、update→update_note、delete→delete_note、toggle_pin、update_tags、search_notes），无 `State<Db>`
- `test_main_rs_no_state_when_db_full_cover`：db 全覆盖时 main.rs 无 `with_state`/`use api::Db`；false 时保留
- `test_017_chat_db_rs_has_real_logic_and_fn_names`：真实 017-chat db.at 端到端 —— db.rs 含 `mine: true`、`extract_db_fn_names` 提取出 `all_messages`+`create_message`、api.rs 委托正确、main.rs 去 state
- `test_gen_015_notes_rust`（既有）：015 完整 codegen 不破坏
- **实际编译验证**：把生成的 017-chat 后端三件套（api.rs/db.rs/main.rs/types.rs/events.rs）独立 `cargo build` 通过（仅 2 个无害 dead-code warning：全委托时 `State`/`Db` 别名未用）
- `test_sse_handler_generation`（既有）：SSE 路径不回归
- auto-man 全部 194 lib 测试绿；auto-lang 22 个失败 = master 既有（与本计划无关）

**验收对照**：
- ✅ POST `/api/messages` 返回 `mine:true`（db.rs create_message 真实执行，编译验证产物含 `mine: true`）
- ✅ GET `/api/messages` 返回含 POST 新增消息（状态统一到 db.rs lazy_static，单进程内 POST 写 == GET 读）
- ✅ 015-notes 全量回归（9 端点单测 + 完整 codegen 测试 + db.rs 转译）
- ⏳ 017-chat playwright 8/8（需 `auto run` 运行时验证，代码层已确认 handler 行为对外不变）

---

## 后续（超出本计划）

- 继续升级 018-027 为正规 App（022-kanban / 023-realworld 下一批候选）
- a2r 深层转译缺陷修复（当前靠 post_process_db_rs 后处理；根治应在 a2r 的全局变量初始化、集合返回生命周期、跨模块类型可见性等转译逻辑）
- vm/rust 前端版（当前只做 vue + rust 后端，对标 015）

---

## 提交历史（auto-ui-examples 分支，相对 master）

| commit | 内容 | 步骤 |
|---|---|---|
| `45361b10` | SSE 多事件 discriminator 数据驱动（Phase 1） | 第1步 |
| `3b50f44a` | rust 后端 SSE 生成（Phase 2） | 第2步 |
| `74557e10` | 017-chat 升级为 SSE App | 第2步 |
| `4601aa6b` | callback relay + yield 检测 | 第3步 |
| `e355955a` | playwright 8/8 + SSE 广播 PascalCase | 第2步 |
| `66d07fee` | CRUD 扩展第1-2步（ApiEndpoint.body + transpile_fn） | CRUD 1-2 |
| `1788e9b8` | CRUD 扩展第3步（db.rs 转译 + 后处理） | CRUD 3 |

注：分支上还混有另一个 agent 的 a2vue 框架提交（i18n/markdown/SSE Phase 4/6/http async），与本计划无直接关系，是历史交错所致。后续工作在 `auto-ui-examples` 分支（worktree `auto-lang-autoui`）继续，不再和 plan-musk 混。
