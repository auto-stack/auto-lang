# Plan 288: Notes App Full-Stack API Integration

## Context

015-notes 示例有完整的前后端代码，但前端全部是内存硬编码数据，没有调用后端 API。
需要三个阶段实现完整的前后端对接。

---

## Phase 1: Vue 前端对接后端 API

**目标**: 修改 app.at handler 调用 API，修改 Vue 生成器识别 notes API 函数。

### 1.1 修改 API 函数检测列表（3 处）

**文件**: `crates/auto-lang/src/ui_gen/vue.rs` line ~3403
**文件**: `crates/auto-lang/src/ui_gen/ts_adapter.rs` line ~32 和 ~725

在所有硬编码的 `API_FUNCTIONS` / `api_functions` / `API_FNS` 列表中添加：
- `listnotes`, `getnote`, `createnote`, `updatenote`, `deletenote`

### 1.2 修改 app.at handler 调用 API

**文件**: `examples/ui/015-notes/src/front/app.at`

- `model`: `var notes = []` (移除硬编码数据，启动时从 API 加载)
- `.LoadNotes` → `.notes = listnotes()`
- `.NewNote` → `let note = createnote("", "")` + `notes.push(note)` + `.active_id = note.id`
- `.SaveNote` → `updatenote(.active_id, .edit_title, .edit_body)`
- `.DeleteNote` → `deletenote(.active_id)` + 本地删除

### 1.3 Vue 生成器支持 `LoadNotes` → `onMounted`

**文件**: `crates/auto-lang/src/ui_gen/vue.rs`

- 当检测到 `LoadNotes` handler 时，自动添加 `onMounted` import
- 在 handler 输出循环后，生成 `onMounted(async () => { ... })`
- 即使 `LoadNotes` 不在模板中使用也要生成（跳过 `used_handlers` 检查）

### 1.4 验证

- `cargo build -p auto`
- `auto run --backend vue` → 检查生成的 App.vue 包含：
  - `import { listnotes, createnote, ... } from '@/lib/api'`
  - `onMounted(async () => { ... })`
  - 各 handler 为 async 函数并调用 await

---

## Phase 2: 动态 API 函数发现

**目标**: 从项目的 `api.at` 自动提取函数名，不再硬编码。

### 2.1 API 解析器传递函数名

**文件**: `crates/auto-man/src/api_gen.rs`（已有 `ApiModule.endpoints[].fn_name`）

- 在 `generate_api()` 后，将提取的函数名写入 `dist/api_functions.json` 或通过上下文传递

### 2.2 生成器接受动态 API 列表

**文件**: `crates/auto-lang/src/ui_gen/vue.rs`
**文件**: `crates/auto-lang/src/ui_gen/ts_adapter.rs`

- `AuraTsContext::new()` → `api_functions` 从 `&'static [&'static str]` 改为 `Vec<String>`
- `VueGenerator` 添加 `with_api_functions()` builder 方法
- `API_FUNCTIONS` 常量保留作为 fallback

### 2.3 调用链打通

**文件**: `crates/auto-man/src/vue.rs`（`run_vue_project` / `build_vue_project`）

- 在生成前，解析 `src/back/api.at` 提取函数名
- 传递给 `VueGenerator::with_api_functions()`

### 2.4 验证

- 移除所有硬编码的 notes API 函数名
- 确认 015-notes 仍能正确检测并生成 async handler

---

## Phase 3: 新增 `auto run backend=rustvm` 模式

**目标**: 一个命令同时启动 VM 运行的后端 + Iced 前端。

### 3.1 新增 BackendType

**文件**: `crates/auto-lang/src/config.rs`

- `BackendType` 枚举添加 `RustVm`（或 `Interpreter`）变体

### 3.2 VM 后端运行器

**文件**: `crates/auto-man/src/automan.rs`

- `run_backend()` 添加 `RustVm` 分支
- 逻辑：用 AutoVM 加载 `src/back/db.at` + `src/back/api.at`，暴露 HTTP 服务 (Axum)
- 在后台线程启动 HTTP 服务监听 8080 端口

### 3.3 前端 API 调用

- Rust 代码生成器 (`ui_gen/rust.rs`) 需要在 handler 中生成 HTTP fetch 调用
- 或者在 Iced 渲染器中集成 HTTP 客户端（reqwest）

### 3.4 CLI 入口

**文件**: `crates/auto/src/main.rs`

- `auto run --backend rustvm` 解析为 `BackendType::RustVm`

### 3.5 验证

- `auto run --backend rustvm` 一条命令启动前后端
- 前端 Iced UI 通过 HTTP 调用后端 CRUD

---

## 关键文件清单

| 文件 | Phase |
|------|-------|
| `examples/ui/015-notes/src/front/app.at` | 1 |
| `crates/auto-lang/src/ui_gen/vue.rs` | 1, 2 |
| `crates/auto-lang/src/ui_gen/ts_adapter.rs` | 1, 2 |
| `crates/auto-man/src/api_gen.rs` | 2 |
| `crates/auto-man/src/vue.rs` | 2 |
| `crates/auto-lang/src/config.rs` | 3 |
| `crates/auto-man/src/automan.rs` | 3 |
| `crates/auto-lang/src/ui_gen/rust.rs` | 3 |
| `crates/auto/src/main.rs` | 3 |

## 执行顺序

Phase 1 → 验证 → commit → Phase 2 → 验证 → commit → Phase 3 → 验证 → commit
