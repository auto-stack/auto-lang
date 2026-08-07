# Plan 399: AutoUI 示例扩展 — SSE 多事件 + CRUD 智能扩展（路径 B）

> **状态（2026-08-07）**: 🟡 代码完成 + 017 运行时验证通过，深层调研发现新遗漏。第 1-5 步落地，017-chat playwright 8/8 全绿。但第二轮深挖发现 5 项此前未记录的问题（§6-§10）：Typing 事件死代码、015 后端从未编译验证、db_fn 映射脆弱、PATCH+body 隐性 bug、混合状态分裂。详见「已知遗留与技术债」。
> **分支**: 第 1-3 步 `auto-ui-examples`（合并 `c175e9d8`）；第 4-5 步 `plan399/handler-db-state`（合并 `45619231`）；收尾 §1-§5 `plan399/cleanup`（合并 `80538fb1`）；深层修复 §6-§10 `plan399/deepfix`（worktree `D:/autostack/auto-lang-autoui`）。
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
- **实际重新生成 + 编译验证（2026-08-07 调研补做）**：用真实 `generate_api("017-chat", "rust")` 重新生成后端到 `crates/auto-man/examples/rust-workspace/017-chat-back/`，`cargo build` 通过。产物确认：`send_message` → `crate::db::create_message(&input.sender, &input.text)`、`list_messages` → `crate::db::all_messages()`、db.rs `create_message` 含 `mine: true`、main.rs 无 `with_state`。仅 2 个无害 dead-code warning（全委托时 `State`/`Db` 别名未用）
- `test_sse_handler_generation`（既有）：SSE 路径不回归
- auto-man 全部 194 lib 测试绿；auto-lang 22 个失败 = master 既有（与本计划无关，见「已知遗留 §4」）

**验收对照**：
- ✅ POST `/api/messages` 返回 `mine:true`（codegen 逻辑正确；**运行时已确认** —— curl POST 返回 `{"mine":true}`，db.rs create_message 真实执行。注：后端产物是 `.gitignore` 忽略的运行时生成物，不入仓库）
- ✅ GET `/api/messages` 返回含 POST 新增消息（状态统一到 db.rs lazy_static；运行时确认 POST 后 GET 总数 5→6）
- ✅ 015-notes 全量回归（9 端点单测 + 完整 codegen 测试 + db.rs 转译）
- ✅ 017-chat playwright 8/8（**§1 运行时验证通过** —— `auto run` 重新生成后端（handler 调 db.rs）+ 前端 + 跑 playwright 8/8 全绿。过程中发现并修复 chat_store.at 缺 `stream` import 的 SSE bug，见 §5）

---

## 已知遗留与技术债（2026-08-07 调研 + 收尾）

### §1 ✅ 017-chat playwright 运行时验证（收尾完成）
第4-5步重构 codegen 后，用 `auto run`（在 017-chat 目录，pac.at 已声明 `render:vue`/`api:rust`）重新生成前后端 + 跑 playwright，**8/8 全绿**。运行时确认：curl POST `/api/messages` 返回 `{"mine":true}`；POST 后 GET 总数 5→6（状态统一）。过程中发现 chat_store.at 缺 `stream` import 致 SSE 不接线（见 §5），修复后 T6/T7 转绿。

### §2 ✅ 前端 `sender=="You"` mine 兜底清理（收尾完成）
`message_thread.at` 气泡方向从 `if msg.sender == "You"` 改为 `if msg.mine`（后端 db.rs create_message 设 mine:true，现权威）。`smoke.spec.ts` T3 注释、`acceptance.atd` T3 契约同步更新。改后重跑 playwright 仍 8/8。

### §3 🟡 `post_process_db_rs` 仍是必需 workaround（注释已修正）
`crates/auto-man/src/api_gen.rs:319` 用字符串后处理修 6 类 a2r 转译缺陷。调研核实**全部仍必需**（a2r 无一根治），已修正过时的头注释（原误导性声称"a2r 已修 List.new/生命周期"）。根治属长远项（改 `trans/rust.rs` 的 use 路径映射、List.new 拦截、切片返回生命周期、str→String 补 to_string）。

### §4 ⚪ `endpoint.body` 标注为路线A预留（收尾完成）
`ApiEndpoint.body` 生产代码无读取者。路线B（db 委托）不依赖它。已在 `types.rs`/`mod.rs` 加注释明确"路线A预留，路线B未用"，不删除（留作未来转译任意 body 的基建）。

### §5 ✅ SSE multi-endpoint 测试 + chat_store.at 真实 bug（收尾完成）
两处修复：
1. **单测** `test_store_composable_sse_multi_endpoint`（`vue.rs:13925`）：测试数据 `api_imports` 加 `"events"`，与 Phase 4 的按 fn_name 过滤逻辑一致。
2. **真实 bug** `examples/ui/017-chat/src/front/chat_store.at`：原 `use back.api: list_messages` 缺 `stream`，致 Phase 4 过滤器（`vue.rs:9900`）丢弃 SSE 接线 → 前端不建 EventSource → T6/T7 失败。改为 `use back.api: list_messages, stream` 并加注释说明。这解释了为何文档原称"第2步 8/8"在当前 codegen 下无法复现。

### §6 🟡→✅(后端) Typing 事件补全（后端链路打通，前端待运行时验证）
原状：`ChatEvent { NewMessage, Typing }` 只有 NewMessage 可用，Typing 全链路无触发。本轮补全了**后端链路 + 前端源码**：
- **后端广播泛化**（`api_gen.rs`）：新增 `broadcast_event_name(endpoint, primary_type)` —— POST create 广播 `"New{Type}"`（如 NewMessage），POST fn 名含 `typing` 的 void 端点广播 `"Typing"`。两处广播（db 委托 + fallback）+ void-POST typing 分支（广播 `&input` payload）均改用动态名，不再硬编码 `"NewMessage"`。
- **typing 端点**：api.at 加 `POST /api/typing` (`set_typing(sender str)`)，db.at 加 `set_typing`（no-op，广播由 handler 做）。
- **前端触发链路**：composer.at 加 `on_typing` 回调，`InputChanged` 时 emit；app.at 加 `.Typing(name)` msg，调 `set_typing(name)`。
- **测试**：`test_sse_broadcast_event_name_not_hardcoded` 验证 create 广播 NewMessage、typing 广播 Typing + input payload。
- **待运行时验证**：前端 store 重生成（type-driven SSE dispatch 已支持 Typing 变体）+ playwright 新增 typing 用例。当前 codegen 层链路完整，需 `auto run` 端到端确认。

### §7 ✅→🟡 015 栈溢出已修复，编译错误转 §3
015 db.at 转译的两个独立 bug 已修，015 现在能**生成**，但生成的 db.rs 仍有编译错误（属 §3 范围）：
- **栈溢出（已修）**：根因是 parser 在 Windows 1MB 主线程栈上，对深嵌套（for + if/else + 8 字段结构体字面量 + note.x 字段访问）递归过深（每帧含 `rhs_lookahead_is_node` 的 Vec<Token>）。修复：`transpile_db_to_rs` 内部用 16MB 栈线程（仓库惯例，cf. `run_autovm` lib.rs:341）。注意根因在 **parser**（`parser.rs`）非 a2r。
- **UTF-8 切片 bug（已修）**：`strip_collection_new`（`api_gen.rs:383`）按字节步进 `i += 1` + `code[i..]` 切片，遇到 015 种子数据的 emoji（📄）panic 在字符中间。修复：按 UTF-8 字符边界步进（`utf8_len` 辅助）。017 无此 bug（全 ASCII）。
- **验证**：`regen_real_015_backend_db_delegation`（#[ignore]，默认栈）通过——015 完整生成，全 db 委托（无 State<Db>），`crate::db::all_notes`/`create_note`/`update_tags` 委托正确。
- **剩余（转 §3）**：生成的 015 db.rs 有 26 个编译错误（E0308×14、E0277 i64/i32×6、E0614×2、E0596 mut×2、E0507×1）——都是 a2r 转译复杂 db.at 的缺陷，属 §3 根治范围。015 当前仍用旧 State<Db> 产物运行。

### §8 🟡 db_fn 映射脆弱（命名偏离即静默回退）
`db_fn_candidates`（`api_gen.rs:738`）是纯字符串前缀归一（list_→all_ 等 13 个动词），**无形态学推理**：
- 不规则复数全失败：`list_person`→候选 `all_person`，db 若叫 `all_people` → 命中失败。
- 同义词/别名失败：`login`→`find_user`、`archive_note`、`publish_post` 不在白名单。
- `move_X` 归一到 `update_X` 有**语义错配风险**（move 改 folder、update 改 title，若 db 只有 update_note 会静默选错）。
- 失败时静默回退 `State<Db>` 模板 + eprintln 警告（`api_gen.rs:1031`），不报错。
- **当前影响低**（015/017 命名严格对齐 CRUD 约定），但未来示例偏离约定会踩坑。根治需改用 `endpoint.body` 的 `db.FN(...)` 调用解析（即路线A）或显式 `#[db_fn]` 注解。

### §9 🟡 PATCH+body 隐性编译 bug
`endpoint_has_body`（`api_gen.rs:700`）只认 `POST | PUT`，**PATCH 不算有 body**。后果：若 PATCH 端点带 body 参数（如 `set_pinned(id, pinned bool)`），handler 签名不加 `Json(input)` 提取器，但 `resolve_db_call` 会生成 `&input.pinned` → handler 里无 `input` 绑定 → **编译错误**。015 的 `toggle_pin` 因无 body 参数而幸免。这是 codegen 普适性缺陷。

### §10 🟡 混合状态（部分回退）运行时状态分裂
当 db.rs 只覆盖部分端点时（`db_full_cover=false`），main.rs 保留 `State<Db>`，但命中的 db 委托 handler 调 db.rs 的 `Lazy` 全局，回退 handler 调 `State<Db>` seed——**两份独立状态，写入会发散**。axum 0.8 下能编译（无 State 提取器的 handler 对任意 S 成立 `Handler`），但运行时同一资源的读写会不一致。文档/注释（`api_gen.rs:1031`）只警告"回退模板"，未提状态分裂风险。当前示例全委托不触发，但应明确记录。

---

## 后续（超出本计划）

- **修 §7（高优先）**：a2r 递归栈溢出（`trans/rust.rs`），让 015 db.at 能转译，恢复 015 的 db.rs 路径
- **补 §6**：Typing 事件端到端实现（后端广播读变体 + composer 触发 + 重生成），让"多事件"名副其实
- a2r 深层转译缺陷根治（§3 的 5 类根因，在 `trans/rust.rs` 而非后处理）
- **补 §8**：db_fn 映射改用 body 解析或注解，消除命名约定依赖
- 继续升级 018-027 为正规 App（022-kanban / 023-realworld 下一批候选）
- vm/rust 前端版（当前只做 vue + rust 后端，对标 015）

> 注：§9（PATCH+body）已在本轮修复（`endpoint_has_body` 纳入 PATCH + 回归测试 `test_patch_with_body_gets_json_extractor`）。§10（混合状态分裂）为设计权衡，文档记录即可。

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
| `fbd15c1e` | CRUD 扩展第4-5步（handler 调 db.rs + 状态统一，路线B） | CRUD 4-5 |
| `45619231` | Merge plan399/handler-db-state → master | CRUD 4-5 合并 |

注：分支上还混有另一个 agent 的 a2vue 框架提交（i18n/markdown/SSE Phase 4/6/http async），与本计划无直接关系，是历史交错所致。后续工作在 `auto-ui-examples` 分支（worktree `auto-lang-autoui`）继续，不再和 plan-musk 混。
