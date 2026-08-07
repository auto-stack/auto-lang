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

### §3 ✅(后处理层) a2r 转译缺陷后处理大幅加固（015 db.rs 26→0 编译错误）
`post_process_db_rs` 原只覆盖 017 简单情形（硬编码 MESSAGES、i32 只修 id 字段）。015 的复杂 db.at（12 fns + for/if-else + 列表重建 + emoji 种子）暴露 26 个编译错误。本轮在后处理层系统修复（a2r `trans/rust.rs` 根治仍属长远项）：
- **i32→i64 统一**：a2r 把所有 `int` 转 `i32`，但 types.rs 用 `i64` → db.rs 全 `i32`→`i64`（消 6 E0277 + 多 E0308）。
- **去 deref 泛化**：`*MESSAGES.lock().push` → 正则 `*\w+.lock().push/insert`（覆盖 NOTES/FOLDERS，消 E0614）。
- **借用字段 .clone()**：`for note in &*G.lock()` 里 `note.title`/`note.tags` 是共享引用，move 报错 → `append_clone_for_borrowed_fields` 给 struct-ctor 里的 `obj.field` 和 `Some(note)` 加 `.clone()`（消 E0507）。
- **let → let mut**：`let results = vec![]` 后 `.push()` → `add_mut_to_let_collections` 加 mut（消 E0596）。
- **str/slice 参数赋字段**：`folder: folder`（&str→String，含尾部 ` }`）+ `tags: tags`（&[String]→Vec 加 `.to_vec()`）；`append_tostring_for_str_fields` 扩展 slice_params + ` }` 结尾。
- **FOLDERS 返回**：`fix_borrowed_slice_returns` 泛化 `return *VAR.lock()` 正则（消 Folder E0507）。
- **handler []str 借用**：`resolve_db_call` 的 `[]str` 参数借用 `&input.tags`（&Vec→&\[String] deref）。
- **验证**：015 后端**实际 cargo build 通过**（Finished，仅 dead-code warning）。`regen_real_015_backend_db_delegation`（#[ignore]）断言全 db 委托 + `&input.tags`。auto-man 196 测试绿。
- **剩余长远项**：a2r `trans/rust.rs` 的根治（int 类型推断 i64、借用迭代器 clone、str→String 转换、mut 推断）——当前靠后处理，根治应在 a2r 层。

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

### §8 ✅ db_fn 映射改进（body-based 解析优先于命名启发式）
原状：`db_fn_candidates` 纯前缀归一（list_→all_ 等 13 动词），无形态学——不规则复数（`list_person`→`all_people` 失败）、同义词（`login`→`find_user`）、`move_X`→`update_X` 语义错配，全部静默回退。
**改进**：`resolve_db_call` 现在优先用 `extract_db_fn_from_body` —— 从 `endpoint.body` 的 `return db.FN(...)` AST 直接取 FN 名（路线A 的轻量版），仅在 body 无/解析失败时回退命名启发式。这覆盖了所有同义词/别名/复数情况（只要 api.at body 是 `return db.FN(args)` 薄委托，FN 名就被精确解析）。
- 前提：`try_full_parse` 成功（拿到 body）。§7 栈溢出修复后，015/017 的 api.at full_parse 在默认栈成功（8/8 端点有 body）。lenient 路径（无 body）仍走启发式。
- 测试：`test_db_fn_resolved_from_body_over_heuristic` —— 同义词端点 `lookup`（不在动词白名单）通过 body 的 `db.find_user(id)` 成功委托。
- 剩余：`move_X` 语义错配（若 body 也是 `db.update_note`）仍按 body 解析——但这反映 api.at 作者意图（body 写啥就调啥），不再是 codegen 的猜测。

### §9 ✅ PATCH+body 编译 bug（已修，前轮）
`endpoint_has_body` 纳入 PATCH（有 body_params 时），带 body 的 PATCH 端点（如 `set_pinned(id, pinned bool)`）现在正确生成 `Json(input)` 提取器。回归测试 `test_patch_with_body_gets_json_extractor`。

### §10 🟡 混合状态（部分回退）运行时状态分裂
### §10 ✅ 混合状态（部分回退）生成期硬检查（Phase 13 已落地）
当 db.rs 只覆盖部分端点时（`db_full_cover=false`），main.rs 保留 `State<Db>`，但命中的 db 委托 handler 调 db.rs 的 `Lazy` 全局，回退 handler 调 `State<Db>` seed——**两份独立状态，写入会发散**。axum **0.7**（仓库实际版本；原文误写 0.8，两者 `Handler`/`with_state` 行为一致）下能编译，但运行时同一资源的读写会不一致。
**Phase 13 落地方案 A**：`generate_rust_server` 在 `has_db && !db_full_cover` 时 `return Err`（列出未覆盖端点名），而非静默回退。escape hatch `AUTO_ALLOW_PARTIAL_DB=1` 供渐进迁移。测试 `test_mixed_state_detection_collects_uncovered`。015/017 全覆盖不受影响。

---

## 后续（超出本计划，详见 Phase 11-13 调研）

本轮（§7/§6/§3/§8）已完成的：栈溢出修复、Typing 后端补全、a2r 后处理加固（015 编译通过）、db_fn body 解析、PATCH+body 修复。剩余三项长远问题已深入调研，作为独立 Phase 记录在下：

- **Phase 11**：a2r 转译器根治（`trans/rust.rs`，6 类缺陷，当前靠 post_process 后处理覆盖）
- **Phase 12**：§6 前端 SSE typing 端到端验证（协议不匹配 bug + store 重生成 + playwright T9）
- **Phase 13**：§10 混合状态分裂根治（生成期硬检查）
- 继续升级 018-027 为正规 App（022-kanban / 023-realworld 下一批候选）
- vm/rust 前端版（当前只做 vue + rust 后端，对标 015）

> 注：§9（PATCH+body）已修复（`endpoint_has_body` 纳入 PATCH + `test_patch_with_body_gets_json_extractor`）。§10 升格为 Phase 13。

---

## Phase 11：a2r 转译器根治（6 类缺陷，`trans/rust.rs`）

> **调研结论**：`rust_type_name`/`rust_param_type_name` 是 `RustTrans` 私有方法，仅 a2r 内部用，**不影响 gdscript/tscn/typescript/javascript/python 等其他转译目标**（各自独立转译器）。axum/tauri targets 有独立的 `to_rust_type`（`api/targets/axum.rs:66` 等写 `int→i32`），改 a2r 不影响它们（若要统一需同步改，低优先）。a2r 已有 `#![allow(unused_mut, unused_parens, ...)]`（`trans/rust.rs:1680`），保守根治策略安全。

### P11.1 int 类型 i32→i64（低成本高价值，优先）
- **根因**：`trans/rust.rs:1162` `Type::Int => "i32"`（唯一类型映射点）。后端 types.rs 用 i64（`api_gen.rs:750` `auto_type_to_rust` `int→i64`）。已有 i64 先例：`trans/rust.rs:9160`（task state 字段，Plan 387）。
- **改动**：(1) `:1162` 改 `i64`；(2) 全文 `as i32`→`as i64`、`.parse::<i32>()`→`.parse::<i64>()`（机械批量 ~30 处）；(3) 删 `api_gen.rs:368` 的 `code.replace("i32","i64")` 暴力后处理。
- **风险**：低（无跨目标影响，回归由 trans/rust.rs 末尾 test fixtures 捕获）。

### P11.2 str→String 字段 to_string（低成本高价值，优先）
- **根因**：`trans/rust.rs:8583` `Arg::Name(name)` 分支**硬编码 `needs_to_string=false`**，不查 `struct_field_types`（相邻的 `Arg::Pair:8584` 和 `Arg::Pos:8578` 都正确查了）。这是 a2r 已有逻辑的遗漏分支。
- **改动**：`:8583` 照抄 `Arg::Pair` 分支，查 `struct_field_types` 判断 String 字段 → 加 `.to_string()`。
- **验证**：删 `api_gen.rs:361` `append_tostring_for_str_fields` 调用后 015/017 仍编译。
- **风险**：低（1 处改动，逻辑从相邻分支照抄）。

### P11.3 &[T] 返回生命周期 + 全局 clone（中成本高价值）
- **根因**：`trans/rust.rs:1214` `Type::Slice` 总输出 `&[T]`（不区分返回位置）；`:1928` 全局读 `*G.lock().unwrap()` 在 return 时 move guard 失败。
- **改动**：(1) 加 `in_return_position` 标志（抄 `current_fn_ret_type:226` 模式），返回位置 `Type::Slice` 输出 `Vec<T>`；(2) `return_stmt`（`:8896`/`:1120`）扩展 `needs_global_clone`（抄 `needs_self_clone` 机制），return 全局变量时发 `G.lock().unwrap().clone()`。
- **删后处理**：`api_gen.rs:348` `fix_borrowed_slice_returns`。
- **风险**：中（`Type::Slice` 在字段/参数位置仍应 `&[T]`，标志位维护需小心）。

### P11.4 借用迭代器字段 clone（中成本高价值）
- **根因**：`for_stmt:10825` 正确对集合加 `&`（借用迭代），但没记录迭代变量是借用；`write_expr_for_struct_field:8630` 不对 `iter_var.field` 加 clone。
- **现成模板**：`trans/rust.rs:10070-10080`（store 处理 `obj.field` 非 Copy → `.clone()`）。
- **改动**：(1) `for_stmt:10825`/`for_stmt_inline:10920` 记录借用迭代变量到 `borrowed_iter_vars: HashSet`；(2) `write_expr_for_struct_field:8630` 加分支：借用迭代变量的字段读无条件 clone（保守版，避开 struct_field_types 查找）；(3) `Expr::Pair:2495` 也走 `write_expr_for_struct_field`。
- **删后处理**：`api_gen.rs:385` `append_clone_for_borrowed_fields`。
- **风险**：中（需维护 iter var 作用域；保守版冗余 clone 无害）。

### P11.5 mut 推断（中成本中价值，暂缓）
- **根因**：`trans/rust.rs:9968` 完全依赖 `var`/`let` 关键字，无 body 扫描。db.at 用 `var` 已对（生 `let mut`），但源码用 `let` 后 push 会漏。
- **改动**：函数级预扫描 pass 收集被 mutate 的 binding（`push/insert/extend/assign`），`store:9968` 命中则强制 `let mut`。
- **暂缓理由**：mutating 方法名集合难完备；后处理 `add_mut_to_let_collections:409` 正则已覆盖 db.rs 90%。边际收益低。

### P11.6 去 deref（中成本中价值，暂缓）
- **根因**：`trans/rust.rs:1926-1928` 全局读无条件加 `*`（注释：算术/比较/index/cast 需解引用），但方法调用（push/insert）前加 `*` 错（E0614）。
- **暂缓理由**：a2r 在 `Expr::Ident` 层看不到下游；根治要碰方法调用全路径，回归面广。db.rs 实际 mutating 方法几乎只有 push/insert，后处理正则（`api_gen.rs:352`）够用，建议先扩 method 集合（push|insert|extend|retain|clear|remove|sort_by|swap|truncate）。

### P11 实施顺序（依赖关系）
P11.1（i64，独立）→ P11.2（str，独立）→ P11.3（&[T] 返回，删 fix_borrowed_slice_returns）→ P11.4（借用 clone，与 P11.2 共享 struct_field_types）→ P11.5/P11.6（后期）。

---

## Phase 12：§6 前端 SSE typing 端到端验证

> **状态**：✅ 完成（playwright 9/9 全绿，typing 指示器跨标签页端到端跑通）。
> **本轮修复 3 个 bug**：
> 1. **协议不匹配**：后端广播 `{"event":"Typing","name": input.sender}`（`serde_json::json!` + 固定 `name` 字段），前端 `chat_store.at` 的 `Typing` 改对象变体 `Typing(TypingEvent)`、handler 读 `evt.name`（解决 `[object Object]` 永不消失）。
> 2. **CreateInput 422**：void POST（set_typing）原复用 `CreateMessageInput`（含 text），前端只发 `{sender}` → 422。改为每个唯一 body-param set 生成独立 CreateInput（`CreateMessageSetTypingInput`），与 UpdateInput 同样的去重逻辑。
> 3. **vue codegen 吞 oninput**：input 有 `:value`+`oninput` 时 v-model 优化吞掉了 oninput handler（只 track 不生成 `@input`）。改为仍生成 `@input="handler"`（v-model + @input 共存，Vue 允许），让有副作用的 handler（如 typing 的 InputChanged）能运行。
> **验证**：`auto run` 重生成（清缓存）+ playwright T1-T9 全绿，含 T9（B 输入 → A 看到 "You is typing…"）。017-chat 现在是真正的多事件 SSE App（NewMessage + Typing）。

> **历史调研发现**：后端逻辑落地后，前端 store/后端产物全是旧的（缓存未失效）+ 协议不匹配 bug——本轮全部修复并运行时验证。

### P12.0 核心协议 bug（阻塞项）
- 后端广播（`api_gen.rs:1313-1319` void+Typing 分支）：`serde_json::to_value(&input)` → `{"event":"Typing","sender":"You"}`。
- 前端 dispatch（`vue.rs:10102`）：`Typing(data)` 传**整个对象**（不区分单值/对象变体）。
- chat_store handler（`chat_store.at:24`）：`.Typing(name) -> { .typing_name = name }` → `typing_name` 变成对象 → MessageThread 模板 `{{ typing_name }} is typing…` 渲染 `[object Object]`，且 `typing_name != ''`（对象≠''恒真）指示器永不消失。
- 根因：`resolve_stream_variants`（`api.rs:198-215`）只抓 PascalCase 变体名，**丢弃载荷类型信息**（`Typing(str)` 的 `str`），无法为单值变体生成 `Typing(data.X)` 取字段代码。
- NewMessage 巧合正确（payload 本身是完整 Message 对象，`NewMessage(data)` push 进数组恰好对）。

### P12.1 修复协议（二选一）
- **方案 A（推荐，改后端广播）**：`api_gen.rs:1313-1319` typing 分支不广播 `&input`，改广播 `serde_json::json!({"event":"Typing","name": input.sender}).to_string()`（字段名 `name`）；配合前端 dispatch 对单值变体生成 `Typing(data.name)`（需 `resolve_stream_variants` 保留载荷类型 + 约定单值变体字段名 `name`）。
- **方案 B（改前端 handler）**：chat_store `.Typing(evt) -> { .typing_name = evt.sender }`，dispatch 仍传整个对象。需 aura 支持 `.sender` 取字段。但与 `Typing(str)` 变体声明语义冲突。
- **推荐 A**：更通用（单值变体是通用 codegen 问题，非 typing 特有）。

### P12.2 store 重生成（先决）
- 删 `examples/ui/017-chat/.auto/ui-cache.json`（`chat_store.at` 的 `artifacts:[]` + 旧 hash）强制失效缓存。
- 重跑 `auto run`（017-chat），验证产物：
  - `api.ts` 出现 `set_typing` fetch 客户端
  - `useChatStoreStore.ts` 出现 `new EventSource('/api/stream')` + `data.event==='Typing') Typing(data...)`
  - 后端 `main.rs` 出现 `/api/typing` 路由 + `api.rs` 的 `set_typing` handler + `db.rs` 的 `set_typing` fn
- `resolve_stream_variants` 读 `src/back/api.at`（`api.rs:120-130`），不依赖 db.rs 转译。

### P12.3 前端增强（可选）
- typing 防抖：composer `oninput` 每字符发一次 POST 会刷屏；加 debounce 250-400ms，或 App `.Typing` handler 加超时清除（typing_name 置空定时器）。
- 当前无此机制，T9 能过但 UX 差。

### P12.4 测试（T9）
```ts
test('T9: typing 指示器跨标签页（B 输入 → A 看到 "X is typing…"）', async ({ browser }) => {
  const ctxA = await browser.newContext(); const pageA = await ctxA.newPage()
  await waitForApp(pageA)
  const ctxB = await browser.newContext(); const pageB = await ctxB.newPage()
  await waitForApp(pageB)
  await pageB.locator('input').fill('hello-typing')  // 触发 oninput → POST /api/typing
  await expect.poll(
    async () => (await pageA.locator('body').innerText()).includes('You is typing'),
    { timeout: 8000, message: 'A 应通过 SSE 看到 "You is typing…"'
  ).toBe(true)
  await ctxA.close(); await ctxB.close()
})
```
- 同步更新 `acceptance.atd` 加 T9。
- 重跑 T1-T9，确认 T6（NewMessage）仍绿。

### P12 风险
P12.1 是阻塞项——不修协议，T9 必失败（`[object Object]`）。P12.2 单独跑会让协议 bug 显形（store 接线后 typing 事件一来就炸）。

---

## Phase 13：§10 混合状态分裂根治

> **关键修正**：plan 原文 `399-...md:158` 写「axum 0.8」，**实际仓库是 axum 0.7**（`examples/rust-workspace/Cargo.toml:29`）。两者在 `Handler`/`with_state` 行为一致（无 State 提取器的 handler 对任意 S 成立 `Handler`；`with_state` 只对「缺 state」报错，对「state 没被用」静默），结论不变，但版本号要改。

### 状态分裂确认（真实风险）
触发条件：`has_db && db_fns 非空 && !all_endpoints_covered`。例：015-notes 加一个 `duplicate_note` 端点但 db.at 无对应函数（或函数名不一致）→ 该端点回退 `State<Db>` 模板，其余调 db.rs Lazy。结果两份独立 `Vec<Note>`：db.rs `NOTES: Lazy<Mutex<Vec<Note>>>` vs main.rs `State<Db>: Arc<Mutex<Vec<Note>>>`（种子初值相同但写操作发散：POST create 写 Lazy，POST duplicate 写 State<Db>，GET list 读 Lazy 看不到 duplicate）。

### 根治方案（推荐方案 A）
- **方案 A（推荐）**：生成期硬检查——`has_db && !all_endpoints_covered` 时 `return Err`（列出未覆盖端点名），而非静默回退。配 escape hatch（`pac.at` 加 `allow_partial_db` 或 env `AUTO_ALLOW_PARTIAL_DB=1`）供渐进迁移。
  - 改动 ~10-20 行（`api_gen.rs:636` `db_full_cover` 计算处 + `:1205-1212` 警告处）。
  - 正确性优先（fail-fast），不破坏 015/017（全覆盖 no-op），未来 022-kanban 等复杂 App 一旦 miss 立即报错。
- 方案 B（未命中端点也生成 db.rs fallback 函数）：改动大、回归风险高、语义不可控（模板丢业务逻辑=mine:false 回归）。
- 方案 C（放弃 db 委托）：不可接受（放弃 §3-§5 成果）。
- 方案 D（Lazy 与 State<Db> 共享内存）：对单集合可行，但 015 多集合（notes+folders）+ 需改 a2r，复杂度高。
- 方案 E（混合时当无 db.rs）：浪费已写业务逻辑，mine:false 回归。

### P13 落地判断
- **当前无真实示例触发**（015/017 全覆盖），不需独立 Phase 实施工时——但**方案 A 防御性加固应尽快落地**（防未来踩坑）。
- 升格标准：仅当某真实示例（如 022-kanban）因混合状态实际跑出 bug 时，才升格深度改造（考虑 B/D）。
- 文档：§10 落地方案 A 后改 ✅；修正 axum 0.7；`:171` 注脚更新。

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
