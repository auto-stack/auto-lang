# 任务计划：AutoMan 迁移与依赖解析重构 (Auto-Lang Edition)

**目标**：将 `auto-man` 仓库迁移至 `auto-lang` 的 Monorepo 结构中，并重构核心架构，将依赖查找（Resolution）逻辑从 VM 核心（即 `auto-lang` crate）中剥离，交由 `auto-man` 实现。

**前置条件**：

1. 当前位于项目根目录 (`auto-lang/`)。
2. **auto-man 项目位于 `../auto-man`**（与 `auto-lang` 同级目录）。
3. Rust 工具链 (`cargo`) 已安装。

**重要信息**：
- ✅ **auto-lang workspace 已配置完成** - `crates/auto-lang` 已存在并正常工作
- ✅ **源项目位置确认** - `../auto-man` 是现有的 auto-man 实现目录
- **迁移策略**：两种可选方案
  1. **直接复制方案**：从 `../auto-man` 复制实现到 `crates/auto-man`
  2. **渐进式方案**：先创建空的 `crates/auto-man` crate，然后逐步迁移功能

---

## 阶段一：物理迁移与 Workspace 设置 (Physical Migration)

此阶段目标是建立 `crates/auto-man` 目录结构，并合并代码库，确保编译通过。

### ✅ 任务 1.1：确认 auto-lang 结构（已完成）

**状态**：✅ 已完成
- ✅ 目录 `crates/auto-lang` 已存在
- ✅ Workspace 已配置，包含多个 crates
- ✅ `crates/auto-lang/Cargo.toml` 配置正确

**验证**：
```bash
ls -la crates/auto-lang/
cat Cargo.toml  # 确认 workspace.members 包含 "crates/auto-lang"
```

### 🚧 任务 1.2：迁移 auto-man 仓库（进行中）

**源位置**：`../auto-man`（与 `auto-lang` 同级）

**方案选择**：

#### 方案 A：直接复制（推荐）
**适用场景**：auto-man 代码量不大，需要快速迁移
**步骤**：
1. 检查 `../auto-man` 目录结构：
   ```bash
   ls -la ../auto-man/
   cat ../auto-man/Cargo.toml
   ```
2. 复制源代码到新 crate：
   ```bash
   mkdir -p crates/auto-man/src
   cp -r ../auto-man/src/* crates/auto-man/src/
   cp ../auto-man/Cargo.toml crates/auto-man/Cargo.toml
   ```

#### 方案 B：渐进式迁移（更安全）
**适用场景**：auto-man 代码复杂，需要逐步适配
**步骤**：
1. 创建空的 `crates/auto-man` crate 结构
2. 先建立基础框架（Cargo.toml, lib.rs）
3. 逐个迁移模块从 `../auto-man`
4. 每个模块迁移后运行测试验证

**本计划采用方案 B（渐进式）**：

1. **创建基础结构**：
   ```bash
   mkdir -p crates/auto-man/src
   touch crates/auto-man/Cargo.toml
   touch crates/auto-man/src/lib.rs
   ```

2. **添加到 workspace**：
   修改根 `Cargo.toml`，添加 `"crates/auto-man"` 到 `members`

3. **配置 auto-man/Cargo.toml**：
   ```toml
   [package]
   name = "auto-man"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   auto-lang = { path = "../auto-lang" }
   ```

4. **检查源项目结构**：
   ```bash
   # 了解需要迁移的内容
   find ../auto-man/src -name "*.rs" | head -20
   cat ../auto-man/src/lib.rs  # 查看入口点
   ```

### 🚧 任务 1.3：配置 Cargo Workspace（进行中）

**状态**：✅ 已有 workspace，需要添加 auto-man

**当前 workspace 配置**（`Cargo.toml`）：
```toml
[workspace]
members = [
    "crates/auto",
    "crates/auto-gen",
    "crates/auto-lang",
    "crates/auto-lang-macros",
    "crates/auto-val",
    "crates/auto-vm",
]
```

**需要添加**：
```toml
[workspace]
members = [
    "crates/auto",
    "crates/auto-gen",
    "crates/auto-lang",
    "crates/auto-lang-macros",
    # 🆕 Plan 078: 添加 auto-man
    "crates/auto-man",
    "crates/auto-val",
    "crates/auto-vm",
]
```

**验证步骤**：
1. 添加 `crates/auto-man` 到 `workspace.members`
2. 运行 `cargo build`，确认 workspace 识别新 crate
3. 运行 `cargo check -p auto-man`，确认 auto-man 编译通过

### 任务 1.4：修复内部依赖

1. 修改 `crates/auto-man/Cargo.toml`：
* 找到对核心库的依赖（原名可能是 `auto-core` 或 `auto`）。
* 将其修改为路径依赖，并指向新名称：`auto-lang = { path = "../auto-lang" }`。


2. **验证**：在根目录运行 `cargo build`，确保所有 crate 编译通过（无依赖路径错误）。

---

## 阶段二：Auto-Lang 核心重构 (Refactor Core)

此阶段目标是在 VM 中定义抽象接口，移除硬编码的文件读取逻辑。

### 任务 2.1：定义 ModuleResolver Trait

1. 编辑 `crates/auto-lang/src/lib.rs` (或适当的模块文件)。
2. 添加 `ModuleResolver` trait 定义：
```rust
use std::path::PathBuf;
pub trait ModuleResolver {
    fn resolve(&self, module_name: &str) -> Result<PathBuf, String>;
    fn get_std_root(&self) -> PathBuf;
}

```



### 任务 2.2：改造 VM 结构体

1. 在 `VM` 结构体中添加字段：`resolver: Box<dyn ModuleResolver>`。
2. 更新 `VM::new` 方法，要求传入 `Box<dyn ModuleResolver>`。

### 任务 2.3：重构 Import 逻辑

1. 找到 VM 处理 `import` 或 `use` 语句的代码段。
2. 将原本直接拼接路径的代码（如 `format!("./libs/{}", name)`）替换为调用 `self.resolver.resolve(name)`。
3. **验证**：此时 `auto-lang` 可能无法通过编译，因为需要修复所有调用 `VM::new` 的地方。暂时创建一个 `MockResolver` 用于通过核心测试。

---

## 阶段三：Auto-Man 逻辑实现 (Implement Logic)

此阶段目标是让 `auto-man` 实现上述接口，接管标准库和三方库的查找。

### 任务 3.1：库化 Auto-Man

1. 如果 `crates/auto-man/src/main.rs` 包含主要逻辑，将其重构为 `lib.rs`，暴露出公共结构体和函数。
2. 确保 `Cargo.toml` 中 `[lib]` 配置正确。

### 任务 3.2：实现 AutoManResolver

1. 在 `crates/auto-man/src/lib.rs` 中引入 `auto_lang` (注意 Rust 代码中使用下划线)。
2. 定义结构体 `pub struct AutoManResolver`。
3. 实现 `auto_lang::ModuleResolver` trait：
* **标准库逻辑**：判断 `name` 是否以 `std.` 开头，返回预设的标准库路径。
* **三方库逻辑**：判断 `name` 是否存在于 `pac.at` 的解析结果中。



### 任务 3.3：实现环境准备入口

1. 在 `AutoManResolver` 中实现 `pub fn prepare_env(root: &str) -> Self`。
2. 在此函数中加入读取 `pac.at` 的逻辑，并构建模块名到路径的 `HashMap`。

---

## 阶段四：集成与验证 (Integration)

此阶段目标是将两者在入口处连接起来。

### 任务 4.1：创建/更新 CLI 入口

*(如果原 auto-lang 中包含 main.rs，建议将其剥离为 `crates/auto-cli`，或者直接修改 `crates/auto-lang/src/main.rs` 作为临时入口)*

1. 在入口文件 (`main.rs`) 中引入 `auto_man` 和 `auto_lang`。
2. 修改 `main` 函数流程：
```rust
// 伪代码
let resolver = auto_man::AutoManResolver::prepare_env(".");
// 注意：这里使用的是 auto_lang::VM
let mut vm = auto_lang::VM::new(Box::new(resolver));
vm.run("main.auto");

```



### 任务 4.2：依赖修正

1. 确保入口 crate 的 `Cargo.toml` 同时依赖 `auto-lang` 和 `auto-man`。

### 任务 4.3：最终测试

1. 创建一个测试用的 Auto 项目：
* `pac.at` (包含一个测试依赖)。
* `main.auto` (包含 `import "std.io"` 和 `import "test-lib"`).


2. 运行 `cargo run`。
3. **验证标准**：
* VM 成功启动。
* VM 调用 Resolver 成功找到 `std.io` 的路径。
* VM 调用 Resolver 成功找到 `test-lib` 的路径（基于 pac.at 配置）。



---

**执行指令**：请按顺序执行上述阶段。每完成一个阶段的任务，请运行 `cargo check` 或 `cargo test` 确保没有破坏构建。如果在重构过程中遇到编译错误，请优先修复接口签名不匹配的问题。