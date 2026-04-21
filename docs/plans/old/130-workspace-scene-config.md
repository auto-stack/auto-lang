# Plan 130: Workspace 和 Scene 配置系统

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 定义 pac.at 的 workspace 和 scene 配置语法，支持多工程协作的前后端分离结构。

**Architecture:**
1. 根目录 pac.at 使用 `scene: "workspace"` 标识为 workspace
2. 子工程使用 `scene: "ui"` 或无 scene 标识工程类型
3. `backend` 字段支持字符串或数组形式

**Tech Stack:** Rust, serde, auto-val

---

## 1. Workspace 配置

**根目录 `pac.at`:**

```auto
name: "my-workspace"
version: "1.0.0"
scene: "workspace"

members: ["./front", "./back"]
```

**字段说明：**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `scene` | string | 是 | 值为 `"workspace"` 标识 workspace 类型 |
| `members` | string[] | 是 | 子工程的相对路径数组，每个路径下需有 pac.at |

**目录结构：**

```
my-workspace/
├── pac.at              # workspace 配置
├── front/
│   ├── pac.at          # 前端工程配置
│   └── app.at          # AURA 源码
├── back/
│   ├── pac.at          # 后端工程配置
│   └── api.at          # API 源码
├── vue/                # 生成的 Vue 项目
└── rust/               # 生成的 Rust 后端
```

---

## 2. 前端工程配置

**`front/pac.at`:**

```auto
name: "my-app-ui"
version: "1.0.0"
scene: "ui"

backend: "vue"
```

**多前端输出：**

```auto
backend: ["vue", "tauri"]
```

**字段说明：**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `scene` | string | 是 | 值为 `"ui"` 标识 AURA UI 工程 |
| `backend` | string 或 string[] | 是 | 输出目标，支持单个或多个 |

**输出目录规则：**
- 输出目录相对于 **workspace 根目录**
- `vue` → `<workspace>/vue/`
- `tauri` → `<workspace>/tauri/`
- `jet` → `<workspace>/jet/`

---

## 3. 后端工程配置

**`back/pac.at`:**

```auto
name: "my-app-api"
version: "1.0.0"

backend: "rust"
```

**字段说明：**

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `scene` | string | 否 | 无 scene 或其他值 = 普通 Auto 工程 |
| `backend` | string | 是 | 输出目标，如 `"rust"` |

---

## 4. Scene 值定义

| scene | 含义 | 解析模式 |
|-------|------|----------|
| `"workspace"` | workspace 根配置 | 不解析代码，只管理成员 |
| `"ui"` | AURA UI 工程 | .at 文件按 AURA 语法解析 |
| 未设置/其他 | 普通 Auto 工程 | .at 文件按标准 Auto 语法解析 |

---

## 5. 命令行行为

### `auto build` (在 workspace 根目录)

1. 读取 `pac.at`，检测 `scene: "workspace"`
2. 遍历 `members` 中的每个路径
3. 读取每个子工程的 `pac.at`
4. 根据子工程的 `scene` 和 `backend` 配置执行构建
5. 输出到 workspace 根目录下的对应目录

### `auto build --target vue`

只构建 backend 为 vue 的子工程。

### 单一工程模式

如果根目录 `pac.at` 没有 `scene: "workspace"`，则按单一工程处理：

```auto
name: "simple-app"
version: "1.0.0"
scene: "ui"
backend: "vue"
```

直接在当前目录构建，输出到 `./vue/`。

---

## 6. 支持的 Backend 类型

| backend | 输出目录 | 说明 |
|---------|----------|------|
| `vue` | `vue/` | Vue + shadcn-vue Web 应用 |
| `jet` | `jet/` | Jetpack Compose Android 应用 |
| `tauri` | `tauri/` | Tauri 桌面应用 |
| `rust` | `rust/` | Rust 后端服务 |
| `gpui` | `gpui/` | GPUI 桌面应用 |
| `iced` | `iced/` | Iced 桌面应用 |
| `arkts` | `arkts/` | 鸿蒙 ArkTS 应用 |
| `cangjie` | `cangjie/` | 仓颉应用 |
| `godot` | `godot/` | Godot 游戏引擎 |

---

## 7. 迁移指南

### 旧语法 (Plan 129)

```auto
backend: { front: "vue", back: "rust" }
app("front") {}
app("back") {}
```

### 新语法 (Plan 130)

**根目录 pac.at:**
```auto
scene: "workspace"
members: ["./front", "./back"]
```

**front/pac.at:**
```auto
scene: "ui"
backend: "vue"
```

**back/pac.at:**
```auto
backend: "rust"
```

---

## Task 1: 添加 Scene 枚举和 Pac 结构体更新

**Files:**
- Modify: `crates/auto-man/src/pac.rs`

**Step 1: 添加 Scene 枚举**

```rust
/// 工程场景类型
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Scene {
    /// Workspace 类型 - 包含多个子工程
    Workspace,
    /// UI 工程 - AURA 语法
    Ui,
    /// 普通工程 - 标准 Auto 语法
    #[default]
    Default,
}

impl Scene {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "workspace" => Self::Workspace,
            "ui" => Self::Ui,
            _ => Self::Default,
        }
    }
}
```

**Step 2: 更新 Pac 结构体**

添加 `scene` 和 `members` 字段。

**Step 3: 运行编译测试**

Run: `cargo build -p auto-man`
Expected: 编译通过

**Step 4: Commit**

```bash
git add crates/auto-man/src/pac.rs
git commit -m "feat(pac): add Scene enum and workspace members support"
```

---

## Task 2: 更新 Pac 解析逻辑

**Files:**
- Modify: `crates/auto-man/src/pac.rs`

**Step 1: 解析 scene 字段**

从 pac.at 内容中解析 `scene` 字段。

**Step 2: 解析 members 字段**

支持数组形式的 members 解析。

**Step 3: 运行测试**

Run: `cargo test -p auto-man pac`
Expected: 测试通过

**Step 4: Commit**

```bash
git add crates/auto-man/src/pac.rs
git commit -m "feat(pac): parse scene and members fields"
```

---

## Task 3: 更新 auto build 命令支持 workspace

**Files:**
- Modify: `crates/auto-shell/src/cmd/commands/build.rs`

**Step 1: 检测 workspace 类型**

如果 `scene == "workspace"`，遍历 members 执行构建。

**Step 2: 子工程构建**

读取子工程的 pac.at，根据 scene 和 backend 执行对应构建。

**Step 3: 运行测试**

Run: `cargo build -p auto-shell`
Expected: 编译通过

**Step 4: Commit**

```bash
git add crates/auto-shell/src/cmd/commands/build.rs
git commit -m "feat(build): support workspace scene type"
```

---

## Task 4: 更新示例项目

**Files:**
- Modify: `examples/unified-demo/pac.at`
- Modify: `examples/split-demo/pac.at`
- Modify: `examples/multi-frontend-demo/pac.at`
- Create: `examples/workspace-demo/pac.at`

**Step 1: 更新现有示例**

将 Plan 129 的语法迁移到 Plan 130 语法。

**Step 2: 创建 workspace-demo**

创建一个完整的 workspace 示例，包含 front/ 和 back/。

**Step 3: Commit**

```bash
git add examples/
git commit -m "refactor(examples): migrate to Plan 130 workspace syntax"
```

---

## Success Criteria

1. `scene: "workspace"` + `members` 能正确识别 workspace 结构
2. `scene: "ui"` 能正确标识 AURA 工程
3. `backend` 支持字符串和数组两种形式
4. `auto build` 能遍历 workspace 成员并执行构建
5. 输出目录正确生成到 workspace 根目录下

## Related Plans

- Plan 129: 统一后端输出目录结构
