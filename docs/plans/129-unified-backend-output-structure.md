# Plan 129: 统一后端输出目录结构

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 统一 Auto 项目的后端输出目录结构，使不同后端（Vue、Jetpack、Tauri 等）的生成代码有一致的目录命名和组织方式。

**Architecture:**
1. 在 `pac.at` 中定义 `backend` 配置，指定输出目录名（vue/, jet/, tauri/ 等）
2. `auto build` / `auto run` 读取 backend 配置，生成到对应目录
3. 废弃旧的 `auto vue` / `auto jet` 命令，统一用 `auto build` / `auto run`

**Tech Stack:** Rust, clap, serde

---

## 关键文件

| 文件 | 作用 |
|-----|------|
| `crates/auto-lang/src/config.rs` | 后端配置解析 |
| `crates/auto-shell/src/cmd/commands/mod.rs` | 命令注册 |
| `crates/auto-shell/src/cmd/commands/build.rs` | 新增 build 命令 |
| `crates/auto-shell/src/cmd/commands/run.rs` | 新增 run 命令 |
| `crates/auto-lang/src/ui_gen/vue.rs` | 修改 Vue 输出路径 |
| `crates/auto-lang/src/ui_gen/jet/project.rs` | 修改 Jet 输出路径 |

---

## Task 1: 添加 BackendConfig 结构体

**Files:**
- Modify: `crates/auto-lang/src/config.rs`

**Step 1: 添加 BackendType 和 BackendConfig 枚举**

在 `config.rs` 末尾添加:

```rust
/// 后端类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendType {
    Vue,
    Jet,
    Tauri,
    Gpui,
    Iced,
    Arkts,
    Cangjie,
    Godot,
    Rust,
}

impl BackendType {
    /// 从字符串解析后端类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "vue" => Some(Self::Vue),
            "jet" => Some(Self::Jet),
            "tauri" => Some(Self::Tauri),
            "gpui" => Some(Self::Gpui),
            "iced" => Some(Self::Iced),
            "arkts" => Some(Self::Arkts),
            "cangjie" => Some(Self::Cangjie),
            "godot" => Some(Self::Godot),
            "rust" => Some(Self::Rust),
            _ => None,
        }
    }

    /// 获取输出目录名
    pub fn output_dir(&self) -> &'static str {
        match self {
            Self::Vue => "vue",
            Self::Jet => "jet",
            Self::Tauri => "tauri",
            Self::Gpui => "gpui",
            Self::Iced => "iced",
            Self::Arkts => "arkts",
            Self::Cangjie => "cangjie",
            Self::Godot => "godot",
            Self::Rust => "back",
        }
    }
}

/// 后端配置（单后端或多后端）
#[derive(Debug, Clone, PartialEq)]
pub enum BackendConfig {
    /// 单后端：整个项目都是同一种类型
    Single(BackendType),
    /// 前后端分离
    Split {
        front: Vec<BackendType>,
        back: BackendType,
    },
}

impl BackendConfig {
    /// 从字符串解析
    pub fn parse(s: &str) -> Option<Self> {
        BackendType::from_str(s).map(Self::Single)
    }

    /// 从 Value 解析（支持对象形式）
    pub fn from_value(value: &auto_val::Value) -> Option<Self> {
        match value {
            auto_val::Value::Str(s) => Self::parse(s),
            auto_val::Value::Obj(obj) => {
                let front = obj.get("front").and_then(|v| match v {
                    auto_val::Value::Str(s) => BackendType::from_str(s).map(|t| vec![t]),
                    auto_val::Value::Array(arr) => Some(
                        arr.iter()
                            .filter_map(|v| match v {
                                auto_val::Value::Str(s) => BackendType::from_str(s),
                                _ => None,
                            })
                            .collect()
                    ),
                    _ => None,
                });
                let back = obj.get("back").and_then(|v| match v {
                    auto_val::Value::Str(s) => BackendType::from_str(s),
                    _ => None,
                });
                match (front, back) {
                    (Some(f), Some(b)) => Some(Self::Split { front: f, back: b }),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// 获取所有前端后端类型
    pub fn frontends(&self) -> Vec<&BackendType> {
        match self {
            Self::Single(t) => vec![t],
            Self::Split { front, .. } => front.iter().collect(),
        }
    }

    /// 获取后端类型
    pub fn backend(&self) -> Option<&BackendType> {
        match self {
            Self::Single(_) => None,
            Self::Split { back, .. } => Some(back),
        }
    }
}
```

**Step 2: 运行编译测试**

Run: `cargo build -p auto-lang`
Expected: 编译通过

**Step 3: Commit**

```bash
git add crates/auto-lang/src/config.rs
git commit -m "feat(config): add BackendType and BackendConfig for unified output"
```

---

## Task 2: 添加 `auto build` 命令

**Files:**
- Create: `crates/auto-shell/src/cmd/commands/build.rs`
- Modify: `crates/auto-shell/src/cmd/commands/mod.rs`

**Step 1: 查看现有命令结构**

Run: `grep -rn "impl Command" crates/auto-shell/src/ | head -5`
Expected: 了解现有命令实现模式

**Step 2: 创建 build.rs 文件**

创建 `crates/auto-shell/src/cmd/commands/build.rs`:

```rust
//! `auto build` 命令
//!
//! 生成后端代码并执行构建
//!
//! # 用法
//!
//! ```bash
//! auto build                 # 构建所有后端
//! auto build --target vue    # 只构建 vue
//! auto build --target jet    # 只构建 jetpack
//! ```

use super::Command;
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use miette::Result;
use std::path::Path;

/// `auto build` 命令
pub struct BuildCommand;

impl Command for BuildCommand {
    fn name(&self) -> &'static str {
        "build"
    }

    fn description(&self) -> &'static str {
        "Generate code and build for configured backends"
    }

    fn execute(
        &self,
        args: &ParsedArgs,
        shell: &mut Shell,
        current_dir: &Path,
    ) -> Result<Option<String>> {
        // 1. 读取 pac.at 配置
        // 2. 解析 backend 配置
        // 3. 按顺序生成每个后端
        // 4. 执行构建命令
        todo!()
    }
}
```

**Step 3: 在 mod.rs 中注册命令**

在 `commands/mod.rs` 中添加 `mod build;` 和 `pub use build::BuildCommand;`
并在注册函数中添加 `registry.register(BuildCommand);`

**Step 4: 运行编译测试**

Run: `cargo build -p auto-shell`
Expected: 编译通过

**Step 5: Commit**

```bash
git add crates/auto-shell/src/cmd/commands/
git commit -m "feat(cli): add `auto build` command skeleton"
```

---

## Task 3: 添加 `auto run` 命令

**Files:**
- Create: `crates/auto-shell/src/cmd/commands/run.rs`

**Step 1: 创建 run.rs 文件**

创建 `crates/auto-shell/src/cmd/commands/run.rs`:

```rust
//! `auto run` 命令
//!
//! 生成后端代码并启动开发服务器
//!
//! # 用法
//!
//! ```bash
//! auto run                 # 运行所有后端
//! auto run --target vue    # 只运行 vue dev
//! auto run --target tauri  # 只运行 tauri dev
//! ```

use super::Command;
use crate::cmd::parser::ParsedArgs;
use crate::shell::Shell;
use miette::Result;
use std::path::Path;

/// `auto run` 命令
pub struct RunCommand;

impl Command for RunCommand {
    fn name(&self) -> &'static str {
        "run"
    }

    fn description(&self) -> &'static str {
        "Generate code and start dev server for configured backends"
    }

    fn execute(
        &self,
        args: &ParsedArgs,
        shell: &mut Shell,
        current_dir: &Path,
    ) -> Result<Option<String>> {
        // 类似 build，但最后执行 dev 命令
        todo!()
    }
}
```

**Step 2: 在 mod.rs 中注册命令**

**Step 3: 运行编译测试**

Run: `cargo build -p auto-shell`
Expected: 编译通过

**Step 4: Commit**

```bash
git add crates/auto-shell/src/cmd/commands/run.rs
git commit -m "feat(cli): add `auto run` command skeleton"
```

---

## Task 4: 修改 Vue 生成器支持自定义输出目录

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs`

**Step 1: 查找现有输出逻辑**

Run: `grep -n "output_dir\|dist" crates/auto-lang/src/ui_gen/vue.rs | head -10`
Expected: 找到硬编码的输出路径

**Step 2: 修改函数签名**

将输出目录从硬编码的 `dist/` 改为参数传入

**Step 3: 运行测试**

Run: `cargo test -p auto-lang ui_gen::vue`
Expected: 测试通过

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ui_gen/vue.rs
git commit -m "feat(vue): support custom output directory"
```

---

## Task 5: 修改 Jet 生成器支持自定义输出目录

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/jet/project.rs`

**Step 1: 查找现有输出逻辑**

**Step 2: 运行测试**

Run: `cargo test -p auto-lang jet::project`
Expected: 测试通过

**Step 3: Commit**

```bash
git add crates/auto-lang/src/ui_gen/jet/project.rs
git commit -m "feat(jet): support custom output directory"
```

---

## Task 6: 更新项目示例结构

**Files:**
- Modify: `examples/component-gallery/pac.at`
- Modify: `examples/jetdemo/pac.at`
- Move: `examples/component-gallery/source/front/` → `examples/component-gallery/front/`
- Move: `examples/jetdemo/source/front/` → `examples/jetdemo/front/`

**Step 1: 更新 pac.at 配置语法**

更新两个示例项目的 `pac.at` 文件，使用新的 `backend` 字段

**Step 2: 迁移源代码目录**

```bash
# component-gallery
mv examples/component-gallery/source/front examples/component-gallery/front

# jetdemo
mv examples/jetdemo/source/front examples/jetdemo/front
```

**Step 3: 删除旧生成目录**

```bash
rm -rf examples/component-gallery/dist
rm -rf examples/jetdemo/dist
```

**Step 4: 更新 .gitignore**

添加新的输出目录到 `.gitignore`

**Step 5: Commit**

```bash
git add examples/
git commit -m "refactor: migrate project structure to new layout"
```

---

## Task 7: 添加迁移文档

**Files:**
- Create: `docs/migration-guide.md`

**Step 1: 创建迁移文档**

创建简明的迁移指南，说明新旧结构对比

**Step 2: Commit**

```bash
git add docs/migration-guide.md
git commit -m "docs: add migration guide for unified backend structure"
```

---

## Success Criteria

1. `auto build` 能正确解析新 `backend` 配置语法
2. 生成代码输出到正确的目录（`vue/`, `jet/`, `tauri/` 等）
3. 多后端项目能顺序构建，失败即停
4. `--target` 参数能选择性构建单个后端
5. 现有示例项目能正常迁移

## Related Plans

- Plan 113-118: a2jet (Jetpack Compose 代码生成)
- Plan 121: Task/Msg 系统
- AutoMan 文档
