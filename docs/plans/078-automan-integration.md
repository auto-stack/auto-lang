# 任务计划：AutoMan 迁移与依赖解析重构 (Auto-Lang Edition)

**目标**：将 `auto-man` 仓库迁移至 `auto-lang` 的 Monorepo 结构中，并重构核心架构，将依赖查找（Resolution）逻辑从 VM 核心（即 `auto-lang` crate）中剥离，交由 `auto-man` 实现。

**前置条件**：

1. 当前位于项目根目录。
2. 拥有 `auto-man` 的 Git 仓库地址（如果无法访问远程，假设本地存在副本）。
3. Rust 工具链 (`cargo`) 已安装。

---

## 阶段一：物理迁移与 Workspace 设置 (Physical Migration)

此阶段目标是建立 `crates/` 目录结构，并合并代码库，确保编译通过。

### 任务 1.1：重组 auto-lang 结构

1. 创建目录 `crates/auto-lang`。
2. 将根目录下原有的 `src/` 目录移动到 `crates/auto-lang/src/`。
3. 将根目录下原有的 `Cargo.toml` 移动到 `crates/auto-lang/Cargo.toml`。
4. **修改配置**：编辑 `crates/auto-lang/Cargo.toml`，确保 `[package]` 下的 `name` 为 `"auto-lang"`。
5. **验证**：检查 `crates/auto-lang/src/main.rs` (或 lib.rs) 是否存在。

### 任务 1.2：合并 auto-man 仓库

*(注：如果 Agent 无法访问外部 Git，请跳过 Git 合并步骤，假设代码已拷贝到 `crates/auto-man`)*

1. 添加 `auto-man` 远程仓库：`git remote add -f auto-man-repo <AUTO_MAN_GIT_URL>`。
2. 合并代码：`git merge -s ours --no-commit --allow-unrelated-histories auto-man-repo/main`。
3. 读取文件树：`git read-tree --prefix=crates/auto-man/ -u auto-man-repo/main`。
4. 提交更改：`git commit -m "Migrate auto-man into crates/auto-man"`。

### 任务 1.3：配置 Cargo Workspace

1. 在根目录新建 `Cargo.toml`，内容如下：
```toml
[workspace]
resolver = "2"
members = [
    "crates/auto-lang",
    "crates/auto-man",
    # 如果存在 cli 目录，也加进来
]

```


2. **验证**：运行 `cargo build`，确认 Cargo 能识别 workspace 成员。

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