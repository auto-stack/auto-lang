# Plan 333：CompileSession 迁移 — 让 VM/Rust 模式都能跑 015-notes

> **For Claude:** 本计划是 Plan 327（015-notes vm render）的延续与收口。前置已合并：点分路径(323)、collect_module_imports 递归(327-1.1)、state-field 方法接收者重写(`37e50d42`)、SelectNote 参数名(`d4eccb53`)、sibling 模块解析+Store/Use 收集(`a2b226b3`)。本计划处理**最后一层阻断**：`back/db.at` 在 collect 路径解析失败（"undefined variable: Note"），根因是该路径未做依赖预解析。目标：VM 模式与 Rust 模式都能端到端运行 015-notes。

## 背景与定位

015-notes 是首个带 **HTTP 后端 + 跨模块导入** 的 full-stack 示例。它在 vue/rust 渲染模式下工作（前端把 `use back.api: ...` 重写成 HTTP/ureq 调用），但在 `--render=vm` 下失败。

调查（systematic-debugging，逐层揭开）确认这是**回归**：script 路径在 commit `8552e0a4`(6/20) 已用 `CompileSession + Linker` 正确实现多模块加载（含依赖预解析），但 **UI/widget 渲染路径漏迁移**，仍停留在 6/18 的 `collect_module_imports`（AST 复制）机制。

### 已修（无需重复）

| 层 | 修复 | Commit |
|---|---|---|
| 1 | 点分路径 `back.api` → `back/api.at` | `64ba5f21`(323) |
| 2 | `collect_module_imports` 递归收传递依赖 | `a656f53a`(327-1.1) |
| 3 | state-field 方法接收者重写（`notes.remove()`） | `37e50d42` |
| 4 | SelectNote handler 显式参数名 `.SelectNote(id)` | `d4eccb53` |
| 5 | sibling 模块解析（父目录回退）+ 收集 Store/Use + back 模块用 Core session | `a2b226b3` |

### 剩余阻断（本计划）

```
link failed for 'App': Undefined symbol: db.all_notes
```

根因：`back/db.at` 顶部 `var notes List<Note> = List<Note>.new([Note{...}])`，其中 `Note` 类型来自 `use api: Note`。collect 路径用裸 `Parser::parse()`，**没有依赖预解析**，解析器对 `Note` 报 "undefined variable" → db.at 整体解析失败 → 它的函数（`all_notes`/`find_note`/...）从未进入导入列表 → 链接时 `db.all_notes` 无法解析。

Script 路径**不**有此问题：`execute_autovm_with_path` 先跑 `session.resolve_uses(code)` 预填 `type_store`，再用 `Parser::new_with_type_store(code, session.type_store())` 解析，`Note` 提前注册。

## 目标

1. **VM 模式**：`auto run --render=vm -B <port>` 在 015-notes 上端到端成功（窗口打开、Init 加载笔记、CRUD 正常）。
2. **Rust 模式**：`auto run --render=rust -B <port>` 同样成功（回归保证）。
3. **架构统一**：widget 导入路径复用 script 路径已验证的多模块机制，消除两条分叉路径的维护负担。
4. **无回归**：016-calendar vm 模式、vue 模式、script 路径全部不变。

## 核心方案：把 CompileSession 预解析接入 widget 导入路径

不是从零重建 UI 编译管线，而是**复用 script 路径已验证的依赖预解析**（`resolve_uses` → `type_store` → `new_with_type_store`），让 `collect_module_imports` 在解析每个导入模块时也能解析类型依赖。

### 关键代码对照（script 路径，lib.rs:554-576）

```rust
let mut session = compile::CompileSession::new();
if let Some(p) = path {
    if let Some(dir) = std::path::Path::new(p).parent() {
        session.add_source_dir(dir.to_path_buf());
    }
}
session.collect_rust_imports(code)?;
session.collect_py_imports(code)?;
session.resolve_deps(code)?;
session.resolve_uses(code)?;                    // ← 预填 type_store
let mut parser = Parser::new_with_type_store(code, session.type_store());  // ← 用它
let mut ast = parser.parse()?;
```

## Phase 1 — collect_module_imports 接入 type_store 预解析（最小改动）

**文件**：`crates/auto-lang/src/lib.rs`（`collect_module_imports`）

**问题**：当前 `collect_module_imports` 对每个模块用 `Parser::from(code).with_session(session)` 裸解析，无 `type_store`，故 `db.at` 的 `Note` 未定义。

**改法**：在解析每个导入模块前，先用 `CompileSession` 跑 `resolve_uses` 预填 `type_store`，再用 `Parser::new_with_type_store` 解析。

```rust
fn collect_module_imports(module_path, visited, out, seen) {
    // ... existing canon/visited guard ...
    let code = fs::read_to_string(module_path)?;
    let is_back_module = module_path 某段 == "back";  // a2b226b3 已加
    let scenario = if is_back_module { Scenario::Core } else { Scenario::UI };

    // 【新增】预解析依赖，预填 type_store（复用 script 路径机制）
    let mut session = CompileSession::new(scenario);
    if let Some(dir) = module_path.parent() {
        session.add_source_dir(dir.to_path_buf());
        // 也加父目录，覆盖 back.api 从 front 解析的场景（与 resolve_module_path 一致）
        if let Some(parent) = dir.parent() {
            session.add_source_dir(parent.to_path_buf());
        }
    }
    let _ = session.resolve_uses(&code);   // 容错：失败不阻断（保持现有 Err→return 行为）

    let mut parser = Parser::new_with_type_store(&code, session.type_store())
        .with_session(scenario);            // 保留 session 选择
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(_) => return,
    };
    // ... 现有的声明收集逻辑（Fn/TypeDecl/EnumDecl/Ext/Store/Use）不变 ...
}
```

**验收**：db.at 解析成功，`all_notes`/`find_note` 进入导入列表；VM 模式 `link failed: db.all_notes` 消失。

**风险**：`resolve_uses` 内部可能递归加载又触发解析（递归深度）。需确认 `resolve_uses` 是否递归到传递依赖——若是，可能与 `collect_module_imports` 自身的递归重叠。Phase 1 调查确认（见 Phase 0）。

## Phase 0 — 调查（不动代码，< 30 min）

1. **resolve_uses 递归性**：读 `compile.rs:398` `resolve_uses` 全文，确认它是否递归加载 `use` 链、是否预填跨模块类型。若它已递归，`collect_module_imports` 的递归可能与之重复——需避免双重加载/双重注册。
2. **type_store 跨模块共享**：确认 `session.type_store()` 是否包含所有传递依赖的类型（db.at 的 Note 来自 api.at）。若 resolve_uses 只注册当前模块的 use，可能需手动把 api.at 的 Note 注入。
3. **CompileSession 在 UI 解析器的兼容性**：`Parser::new_with_type_store` 是否兼容 UI session（widget 语法）？确认 front 模块（editor.at 等）不会因注入 type_store 而误解析。
4. **最小复现**：写一个 2 文件微例（front 用 back 的类型），确认 Phase 1 改法在该微例上 work，再上 015-notes。

**产出**：本计划追加「Phase 0 结论」，锁定 Phase 1 的确切改法。

## Phase 2 — 链接器对 db.func() 前缀回退验证

**文件**：`crates/auto-lang/src/vm/loader.rs`（`Linker::link`，约 246-302）

Phase 1 后，db.at 的函数应进入合成模块的导出表。App handler 调 `db.all_notes()` 生成 reloc 符号 `db.all_notes`，链接器前缀回退（`db.all_notes` → `all_notes`，loader.rs:284）应解析。

**验收**：VM 模式 Init 成功调用 `list_notes()` → `db.all_notes()`，返回 3 条种子笔记。

**风险**：`api.at` 和 `db.at` 都有 `delete_note`/`create_note`/`update_note`，重名。loader 对重名用 `module#name` 限定（loader.rs:260）。App handler 调 `delete_note` 走精确匹配（命中 api.at 版本），其内部 `db.delete_note` 走前缀回退（命中 db.at 版本）。**需端到端验证无错配**——若 api 版本被 db 版本遮蔽，CRUD 会行为异常。

## Phase 3 — 端到端验收（VM + Rust 双模式）

### VM 模式
```bash
cd examples/ui/015-notes
auto run --render=vm -B 9090
```
- [ ] 窗口打开（iced）
- [ ] Init 加载 3 条种子笔记
- [ ] 点击笔记切换（SelectNote）
- [ ] 新建笔记（NewNote）
- [ ] 删除笔记（DeleteNote）
- [ ] 保存笔记（SaveNote）
- [ ] 无 "Undefined symbol" / "undefined variable" 错误

### Rust 模式（回归）
```bash
auto run --render=rust -B 9090
```
- [ ] 同上 CRUD 全通过

### 回归（不能破坏）
- [ ] `auto run --render=vm`（016-calendar）窗口正常
- [ ] `auto run`（015-notes vue 模式）vite 启动 + CRUD
- [ ] `cargo test -p auto-lang --lib --features ui handler_codegen`（5/5+）
- [ ] `cargo test -p auto-lang --lib`（全量，无新增失败）

## Phase 4 — 清理与文档

- [ ] 移除 a2b226b3 中的 `is_back_module` Core/UI session 切换 hack（若 Phase 1 的 type_store 预解析让 UI session 也能解析 db.at，则不再需要按位置切 session——更干净）
- [ ] 更新 `collect_module_imports` 注释，说明它现在复用 CompileSession 机制
- [ ] 在 Plan 327 文档标注本计划收口了其剩余阻断点
- [ ] 若 Phase 1 证明 collect_module_imports 可完全由 CompileSession 驱动（resolve_uses 已递归），考虑后续把 collect 简化为薄包装

## 不做的事（范围控制）

- **不**把整个 widget 编译管线换成 CompileSession（仅给 collect 路径加预解析）。完整统一是更后续的事。
- **不**新建 HTTP client FFI。确认：VM 模式下后端函数在同进程同 VM 内链接调用（用户提出的"合并"思路的雏形），无需 HTTP 往返。完整的 vm+vm/rust+rust 进程合并作为单独 roadmap。
- **不**改 vue 模式代码路径（它工作正常）。
- **不**改 loader.rs 的链接算法（Phase 2 仅验证现有前缀回退够用）。

## 依赖与风险

- **Plan 325-autovm**：记录了"跨模块调返回字符串不可靠、enum 实例方法不调"。若 015-notes 的 CRUD 触及这些（Note 是 struct 不是 enum，应不触发；但 `list_notes` 返回 `[]Note` 数组需确认跨函数返回数组可靠）。Phase 3 验收时关注。
- **递归解析深度**：resolve_uses + collect_module_imports 双递归可能爆栈或重复。Phase 0 必查。
- **符号重名**：api/db 双 `delete_note` 等。Phase 2 必验。

## 与"合并"架构的关系

用户提出的"同进程直接调用"（vm+vm / rust+rust 合并成单程序，跳过 HTTP/IPC）是**更优的最终架构**。本计划实质是在向 vm+vm 合并演进：前端 VM 直接持有并调用后端函数，无 HTTP 往返（后端 HTTP 服务器在 vm 模式仍启动，但前端不依赖它）。完整的跨 render 模式合并（rust+rust 打包单 exe）作为后续 plan。

## 验收标准（Definition of Done）

1. `auto run --render=vm -B <port>` 在 015-notes 端到端 CRUD 成功
2. `auto run --render=rust -B <port>` 回归通过
3. 016-calendar vm、015-notes vue、全量单测无回归
4. `db.at` 不再解析失败；`db.func()` 链接成功
5. a2b226b3 的 session hack 被更干净的 type_store 预解析取代（若适用）
