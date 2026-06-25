# Plan 334：vm+vm 同进程合并 — 跳过冗余后端 HTTP 进程

> **For Claude:** 本计划是 Plan 333 的直接收尾。Plan 333 已让前端 widget VM 直接链接并调用后端函数（`list_notes` → `db.all_notes` → `notes` 全局），**数据访问已经不走 HTTP**。但 `run_vm_ui` 仍冗余地启动后端 Axum HTTP 进程（`start_api_server`），它编译慢、占用端口、且前端从不通过 HTTP 调用它。本计划在 vm+vm 时跳过该进程，让前端 VM 独立运行。

## 背景与定位

### 现状（Plan 333 后）

`auto run --render=vm` 的流程（`crates/auto-man/src/rust_ui.rs:1185 run_vm_ui`）：

```rust
pub fn run_vm_ui(project_dir: &Path, _args: Vec<String>) -> AutoResult<()> {
    let mut _api_child = start_api_server(project_dir);   // ← 冗余：启动后端 Axum HTTP 进程
    // ...
    let result = auto_lang::run_file(entry);              // ← 前端 widget VM，已直接链接后端函数
    stop_api_server(&mut _api_child);
    result
}
```

- `start_api_server`（rust_ui.rs:1036）`cargo run` 一个独立的 Axum HTTP 服务器（`015-notes-back`），监听 `AUTO_HTTP_PORT`，暴露 `/api/notes` 等路由。
- 前端 widget VM（经 Plan 333）**已经**通过 `__module_init` 初始化全局 + 链接的后端函数直接访问数据，**从不向该 HTTP 端口发请求**。
- 后果：①冗余编译（每次 cargo build 后端，慢）；②占用端口（需 `-B` 避让 musk 等）；③启动慢（等就绪探针）；④浪费资源。

### 目标

当 `--render=vm`（隐含前端 vm，后端 vm）时，**`run_vm_ui` 跳过 `start_api_server`**，前端 widget VM 独立运行，零后端进程、零 HTTP。Plan 333 的函数链接机制提供支撑，本计划只是去掉冗余启动。

### 不做的事（范围控制）

- **不**做 rust+rust 合并（a2r 同进程后端调用）。用户明确选择"仅 vm+vm"。rust+rust 作为后续。
- **不**改 vue 模式、rust 模式（它们确实需要 HTTP 后端）。
- **不**改前端 widget VM 的代码生成或链接逻辑（Plan 333 已完成）。
- **不**改 `start_api_server` 本身（其它模式仍用它）。

## 核心：仅 vm+vm 跳过后端进程

### 判断"vm+vm"的依据

`run_vm_ui` 本就是 `--render=vm` 的入口（由 `run_backend` 在 `BackendType::Vm` 时路由到这里，automate.rs:1188）。即：**调用 `run_vm_ui` ⟺ 前端是 vm**。后端在 vm 模式下也应是 vm（前端 vm + 任何 HTTP 后端都没有意义，因为前端不调 HTTP）。所以 `run_vm_ui` 内部可以无条件跳过后端进程。

但为稳健，加一个配置开关（用户可能显式想要带 HTTP 后端的 vm 前端，例如调试）：

```
AUTO_VM_WITH_HTTP=1   → 保留旧行为（启动后端 HTTP 进程），用于调试
默认（未设置）        → vm+vm 合并：跳过后端进程
```

## Phase 1 — run_vm_ui 跳过后端进程

**文件**：`crates/auto-man/src/rust_ui.rs`（`run_vm_ui`）

**改法**：

```rust
pub fn run_vm_ui(project_dir: &Path, _args: Vec<String>) -> AutoResult<()> {
    // Plan 334: vm+vm 同进程合并。前端 widget VM 经 Plan 333 直接链接后端
    // 函数访问数据，不需要独立的 Axum HTTP 进程。跳过 start_api_server 可
    // 消除冗余编译、端口占用、启动等待。设 AUTO_VM_WITH_HTTP=1 可保留旧
    // 行为（启动 HTTP 后端进程）用于调试或与外部 HTTP 客户端联调。
    let mut _api_child = if std::env::var("AUTO_VM_WITH_HTTP").as_deref() == Ok("1") {
        start_api_server(project_dir)
    } else {
        println!();
        println!("  {} vm+vm merged mode: backend runs in-process (no HTTP server)", "✓".bright_green());
        None
    };

    let entry = project_dir.join("src").join("front").join("app.at");
    if !entry.exists() {
        stop_api_server(&mut _api_child);
        return Err(format!("Frontend entry not found: {}", entry.display()).into());
    }

    println!("{}", "Running VM interpreter UI (backend: vm, merged)".bright_cyan());
    // ... 其余不变（CWD 切换、run_file、恢复 CWD、stop_api_server）...
}
```

**验收**：`auto run -r vm -B 3042` 不再出现 "Starting API backend server" / "API server is ready" / cargo run 后端；窗口正常打开，notes 列表正常加载（验证 vm+vm 合并未破坏数据访问）。

## Phase 2 — 端到端验收

### vm+vm 合并模式（默认）
```bash
cd examples/ui/015-notes
auto run -r vm -B 3042     # -B 现在可能不再必要（无后端进程），但保留无害
```
- [ ] 无 "Starting API backend server" / 无 "API server is ready"
- [ ] 无 cargo run 后端编译输出
- [ ] 无端口占用（启动快，无需就绪探针等待）
- [ ] 窗口打开
- [ ] Init 加载 notes 列表（Plan 333 的 __module_init 生效）
- [ ] 点击切换笔记（SelectNote）
- [ ] 新建/删除/保存笔记（NewNote/DeleteNote/SaveNote 触达 db 函数）

### AUTO_VM_WITH_HTTP=1（调试模式，保留旧行为）
```bash
AUTO_VM_WITH_HTTP=1 auto run -r vm -B 3043
```
- [ ] 仍启动后端 HTTP 进程（回归保护）
- [ ] 行为与改造前一致

### 回归（不能破坏）
- [ ] `auto run`（vue 模式）：vite 启动 + 后端 HTTP 正常 + CRUD（vue 走 HTTP，不受影响）
- [ ] `auto run -r rust`：后端 HTTP 正常
- [ ] `auto run -r vm`（016-calendar）：窗口正常（016 无后端，本就无 start_api_server）
- [ ] `cargo test -p auto-lang --lib --features ui handler_codegen`（5/5）

## Phase 3 — 选项：移除 vm 模式下的 -B 必要性（可选优化）

vm+vm 合并后，`-B`（后端端口）对 vm 模式无意义（无后端进程）。可在 help/文档里说明 vm 模式忽略 `-B`，或在 `run_vm_ui` 里提示"-B 在 vm 合并模式下不生效"。**可选**，非必需。

## 依赖与风险

- **依赖 Plan 333**：vm+vm 合并依赖 Plan 333 的 `__module_init` 全局初始化 + 跨模块函数链接。若 Plan 333 未完成，跳过后端进程会导致数据访问全失败。Plan 333 已合并（`0b705d4d`），满足。
- **状态持久性**：vm+vm 合并模式下，后端数据（`var notes`）是进程内全局，每次启动从 db.at 的种子数据重置——这对本地 dev 工具可接受（与之前一致，因为后端 HTTP 也是每次启动从种子数据初始化）。
- **子组件 prop 赋值**（遗留，非本计划阻断）：EditorPanel 的 `.Save`/`.Delete` 仍报 `Undefined variable: self` 警告。不影响 Init 加载和 App 级 CRUD 的验证；保存到全局 db 的路径（DeleteNote handler → delete_note → db.delete_note）已由 Plan 333 链接通。
- **多实例**：vm+vm 合并后，多个 `auto run -r vm` 不再争用 HTTP 端口（各自独立进程），`-F`（前端端口）在 iced 窗口模式下也无意义。更易并行运行。

## 验收标准（Definition of Done）

1. 默认 `auto run -r vm` 不启动后端 HTTP 进程（无 cargo run、无端口占用）
2. Init 加载 notes、App 级 CRUD 在合并模式下正常
3. `AUTO_VM_WITH_HTTP=1` 保留旧行为（调试逃生口）
4. vue 模式、rust 模式、016-calendar vm 无回归
5. 启动明显更快（省掉后端编译+就绪探针）

## 与"合并"架构的关系

本计划实现 vm+vm 同进程合并——前端 VM 直接持有并调用后端函数，零 HTTP 往返。这正是用户提出的"合并"架构的 vm+vm 落地。rust+rust 合并（a2r 把前后端打成一个 exe）作为后续 plan。
