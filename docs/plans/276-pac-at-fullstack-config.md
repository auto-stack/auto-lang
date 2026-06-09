# Plan 276: pac.at 全栈项目配置重设计

## Context

当前 `pac.at` 的 `backend` 字段存在歧义问题，且缺少后端 API 服务的配置支持。
015-notes 项目暴露了以下问题：

### 问题 1：`backend` 命名歧义

`backend` 同时被用于两个完全不同的含义：
- **UI 渲染目标**：`backend: ["vue", "arkts", "jet"]` 表示前端 UI 用什么框架渲染
- **桌面端方案**：`backend: "rust"` 在 Tauri 上下文中表示用 Rust 桌面库渲染
- **API 后端**：`rust/` 目录实际是 Axum API 服务器，跟 UI 渲染无关

### 问题 2：前后端分离配置

`api-example` 把前后端分成两个独立 pac（`front/pac.at` + `back/pac.at`），导致：
- API 函数签名需要手动保持前后端一致
- 无法从单一配置推导完整的项目结构
- `auto run` 不知道要启动后端服务

### 问题 3：后端未自动启动

`auto run` 只启动前端 dev server（Vite），不启动 Rust API 后端。
当前没有配置项告诉 AutoMan 是否需要启动后端服务。

---

## 设计方案

### 1. 重命名 `backend` → `render`

`render` 明确表示 UI 渲染目标平台：

```auto
// 之前（有歧义）
backend: ["vue", "jet", "arkts", "rust"]

// 之后（清晰）
render: ["vue", "jet", "arkts"]
```

> `rust` 在旧语境中指 Tauri 桌面端，如果仍需支持，应改为 `render: ["tauri"]` 或单独的 `desktop` 字段。

### 2. 新增 `api` 字段

表示后端 API 技术栈：

```auto
api: "rust"    // 生成 Axum HTTP 服务器
api: "node"    // 未来：生成 Express/Fastify 服务器（预留）
```

不设置 `api` 字段 = 纯前端项目，无需启动后端。

### 3. 目录约定

统一在单个 `pac.at` 中管理：

```
project/
├── pac.at              ← 统一配置
├── src/
│   ├── front/          ← 前端 .at 代码（按 render 翻译）
│   │   ├── app.at      ← 主 UI widget
│   │   └── editor.at   ← 子 widget
│   └── back/           ← 后端 .at 代码（按 api 翻译）
│       └── api.at      ← API 类型 + 端点定义
├── gen/
│   ├── vue/            ← 生成的 Vue 项目
│   └── ark/            ← 生成的 ArkTS 项目
└── rust/               ← 生成的 Rust API 服务器
```

**约定优于配置**：
- `src/front/` 存在 → 自动识别为前端代码目录
- `src/back/` 存在 → 自动识别为后端代码目录
- `src/back/api.at` → API 定义文件（自动生成前端 client + 后端 server）

### 4. 完整 pac.at 示例

```auto
name: "notes"
version: "1.0.0"
render: ["vue", "arkts"]    // UI 渲染目标（可多个，每个生成一套）
api: "rust"                  // 后端 API 技术栈
```

纯前端项目（无后端）：

```auto
name: "counter"
version: "1.0.0"
render: ["vue"]
// 无 api 字段 → 不生成/启动后端
```

### 5. 类型共享机制

`api.at` 中的类型定义是**唯一的真相来源**：

```auto
// src/back/api.at
pub type Note = {
    id: int
    title: str
    body: str
    time: str
}

#[api(method = "GET", path = "/api/notes")]
pub fn listnotes() []Note { ... }
```

自动生成：
- **前端**：`dist/src/lib/api.ts` 中的 TypeScript interface + fetch client
- **后端**：`rust/src/types.rs` 中的 Rust struct + Axum CRUD handler

两端类型永远一致，无需手动同步。

---

## 实现步骤

### Step 1: 解析器支持新字段

**文件**: `crates/auto-man/src/pac.rs`（或对应的 pac.at 解析模块）

- 在 PacConfig 结构体中添加 `render` 字段（`Vec<String>`）
- 在 PacConfig 结构体中添加 `api` 字段（`Option<String>`）
- 修改解析逻辑：先尝试读取 `render`，回退到 `backend`（向后兼容）
- 解析 `api` 字段，缺失则为 None

### Step 2: 全局重命名 `backend` → `render`

**涉及文件**：
- `crates/auto-man/src/pac.rs` — 配置结构体
- `crates/auto-man/src/vue.rs` — 所有 `backend` 引用
- `crates/auto-man/src/ark.rs` — ArkTS 生成器
- `crates/auto-man/src/jet.rs` — Jetpack 生成器
- `crates/auto-man/src/api_gen.rs` — API 生成
- `examples/ui/*/pac.at` — 所有示例项目配置

> 注意：保持 `backend` 作为别名向后兼容一个版本周期。

### Step 3: `auto run` 自动启动后端

**文件**: `crates/auto-man/src/vue.rs` → `run_vue_project()`

在启动 Vite dev server 之前：
1. 检查 `api` 字段是否设置
2. 检查 `rust/Cargo.toml` 是否存在（后端已生成）
3. 如果满足条件，`cargo run` 启动后端服务器（后台进程）
4. 等待后端就绪（健康检查或固定等待）
5. 再启动 Vite dev server

### Step 4: `auto build` 根据 `api` 字段生成后端

**文件**: `crates/auto-man/src/api_gen.rs`

当前已有 `generate_api()` 逻辑，确保只在 `api` 字段设置时执行：
- 解析 `api` 字段值（`"rust"` → Axum 生成器）
- 调用 `generate_rust_server()` 生成完整 CRUD

### Step 5: 更新示例项目

将 `examples/ui/015-notes/pac.at` 更新为：
```auto
name: "notes"
version: "1.0.0"
render: ["vue", "arkts", "jet"]
api: "rust"
```

---

## 关键文件

| 文件 | 改动 |
|------|------|
| `crates/auto-man/src/pac.rs` | 添加 `render`、`api` 字段解析 |
| `crates/auto-man/src/vue.rs` | `backend` → `render`，添加后端启动逻辑 |
| `crates/auto-man/src/ark.rs` | `backend` → `render` |
| `crates/auto-man/src/jet.rs` | `backend` → `render` |
| `crates/auto-man/src/api_gen.rs` | 使用 `api` 字段决定是否生成 |
| `examples/ui/015-notes/pac.at` | 更新配置格式 |
| `docs/design/08-ui-systems.md` | 更新文档 |

## 向后兼容

- `backend` 字段在 1-2 个版本内仍可使用（映射到 `render`）
- 如果两个都设置，`render` 优先
- `api` 字段为可选，缺失 = 纯前端项目

## 依赖

- Plan 288（Notes 全栈 API）已完成大部分前端对接工作
- 本 plan 的 `api_gen.rs` CRUD 生成已完成（`generate_api_rs` / `generate_main_rs`）
- 本 plan 的核心增量是：配置重设计 + `auto run` 自动启动后端
