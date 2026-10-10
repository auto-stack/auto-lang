# 模块解析（use 扫描与路径解析）

## 范围

`use` 语句的两层处理：预处理期的轻量扫描（use_scanner）与解析后的路径落盘
（ModulePath + ModuleResolver）。模块组织概念（mod/lib/pac 三层）见 docs/design/10。

## 原则

- **快慢分离**：依赖预估用字符串级扫描，不做完整解析；精确语义走 parser 产出的
  `ModulePath` AST。
- **解析策略可插拔**：`ModuleResolver` trait 把"模块名 → 文件路径"委托给实现方，
  默认 `FilesystemResolver`，包管理器/远程 registry 可实现同 trait 接入（plan-078 Stage 2）。
- **前缀显式**：`super` 向父目录、`pac` 向包根，禁止隐式上溯。

## 细节

- `scan_use_statements(source) -> Vec<UseStatement>`（use_scanner.rs:159）：逐行匹配
  `use ` / `use.` 前缀，支持：
  - `use std.io`（整模块）、`use std.io: read, write`（指定项）、`use std.io.*`（通配）、
    `use std.io as io`（别名）
  - `use c <stdio.h>`（C 头）、`use.rs serde::json::{...}`（plan-092）、
    `use.py json5::{...}`（plan-214）、`pub use`（plan-167）
  - 按 module 名去重，同名只保留第一条。
- AST 侧：`ModulePath { prefix: PathPrefix, segments }`（ast/module_path.rs），
  `PathPrefix` 含 `Pac` 与 `Super(count)`（多级 super）。
- `FilesystemResolver::resolve_with_prefix`（resolver.rs:155）：
  - `Pac`：在 search_paths 中按序查找；
  - `Super(count)`：从当前文件目录上溯 count 级；若已到包根（等于某 search_path），
    报错并提示改用 `pac.` 前缀。
- 歧义规则：`name.at` 与 `name/mod.at` 同时存在时编译报错（docs/design/10）。
- 模块组织三层：`mod`（单文件或同名入口文件夹 `net/net.at`）→ `lib`（多模块特性集）→
  `pac`（依赖管理单元）。`lib`/`pac` 层在构建系统中尚未完全形式化（docs/design/10 §Status）。

## 不变量

- `use_scanner` 的结果只用于预估：字符串字面量内的 "use" 行、条件编译内的 use 都会被
  算入；它不回读 parser 的判定。
- `super` 不允许越过包根——越界是带指引的显式错误，不是静默截断。


## 导入语义（Plan 545：bare = 命名空间）

use 语句的**符号可见性**语义（区别于上文的路径解析机制）：

- bare `use db`：仅引入模块命名空间——限定**函数**调用 `db.load()` 可用，
  裸名 `load()` 是带提示的编译错误（提示 `db.load` 或 `use db: *`）。
  类型引用不经命名空间（`db.Note {}` 类型位为语法层未支持形态，R1 探针
  实证 545 前后同拒）——类型须具名导入（`use db: Note`）。
  平铺通路的三层收紧：TypeStore 不 merge（compile.rs load_module_inner bare 分支）、
  Linker dep 模块 exports 仅限定注册（`db#load` + 点分别名 `db.load`，入口模块
  独占裸名——add_entry_module）、native/widget 注册不视为通配
  （autovm_persistent.rs should_import + lib.rs widget/store 注册三处）。
- `use db: *`：显式平铺——TypeStore `merge_with_conflicts`（同名异源异定义 →
  编译错误，含双方模块名；同定义 re-export 不报）+ codegen wildcard 臂将导出符号
  映射为 `db.sym` 限定 reloc（import_scope）。
- `use db: a, b`：具名导入不变；主动遮蔽不报错（与 Rust use 一致）。
- 传递隔离：被导入模块自己的 bare use 不再向导入方泄漏符号（递归 resolve_uses
  走同一 bare 分支）；**例外**：传递 wildcard 仍 merge 进导入方 session store
  （显式 opt-in 面维持现状，KNOWN-DEBT 候选）。
- 语义边界：本语义只作用于 UseKind::Auto；`use.py`（Plan 214/300 独立分发，
  bare → py_modules 点调用解析）与 use.rs/use.c 不受影响。a2py 对 bare 发射
  `import X`（python.rs handle_use）本就同形；a2r bare 发射 `use crate::X;`
  命名空间导入（trans/rust.rs），不再发 glob。
- host↔aavm：aavm 自举层（auto/lib）的 use 实现未随本语义收紧（语料无 bare 平铺
  锚定，不受冲击）——分叉登记 KNOWN-DEBT。

> 来源: docs/plans/545-use-namespace-semantics.md；代码核对 vm/loader.rs、compile.rs、types.rs、vm/codegen.rs handle_use_stmt

## 装配计划与选层（PLAN-738）

- 解析产物统一进装配计划：公共 .at 在前 + 目标选定层；`AssemblyTarget::
  context_extension()` 驱动层后缀，不硬编码 .vm.at。
- 目标层声明命名空间：`c.*` 模块（`stdlib/c/<name>.c.at`）仅 C 目标会话
  可见；VM/Rust 会话维持 not-found 语义。
- 源段边界（context_byte_boundary）：公共段可独立解析时，合并源错误
  归因目标层真实文件行。
- use 语义不变：545 bare/named/wildcard、635 门控、循环、同名异 root 按
  resolver 身份区分；用户同名模块/同名函数干净遮蔽 stdlib（fn 遮蔽优先）。

## 核心导入闭包（PLAN-738 T-11）

- 具名 `use auto.<core>: f`：逐项验证绑定+公共签名；纯适配面符号拒绝
  并指明限定拼写。
- wildcard：只记台账，不因未引用 unsupported 拒绝；其裸名调用按公共面
  唯一归属验证（多模块同名=歧义拒绝）。
- 裸名调用重建限定文本携带真实实参面（AST 序列化）验证元数/async。
- `auto.<core>.<method>()` 三段形状同走 strict 门——io/net 等无 Rust
  producer 的模块诚实拒绝，不发射幻影。

## 缓存边界（PLAN-738 T-05）

段级指纹（公共+选定层各自 FNV + absent 台账 + provider schema + 依赖
闭包）；跨 target 条目共存不串；root/目标热换=SessionTargetMismatch 须
重建会话；同 mtime 改内容仍失效（内容指纹非时间戳）。

## 显式非目标

- use_scanner 不做语法校验，不报告非法 use——那是 parser 的职责。
- ModuleResolver 不含模块加载/缓存语义（加载缓存属 AIE/AutoCache 侧，plan-085/090）。
- `lib`/`pac` 的 manifest 文件形式未定（docs/design/10 §Open Questions），当前靠目录约定。

> 来源: docs/design/10-language-syntax.md §Code Organization；docs/plans/archive/131-module-path-syntax-design.md、docs/plans/archive/078-automan-integration.md；代码核对 use_scanner.rs、resolver.rs、ast/module_path.rs
