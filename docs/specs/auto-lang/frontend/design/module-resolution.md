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
  - `use c <stdio.h>`（C 头）、`use.rust serde::json::{...}`（plan-092）、
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

## 显式非目标

- use_scanner 不做语法校验，不报告非法 use——那是 parser 的职责。
- ModuleResolver 不含模块加载/缓存语义（加载缓存属 AIE/AutoCache 侧，plan-085/090）。
- `lib`/`pac` 的 manifest 文件形式未定（docs/design/10 §Open Questions），当前靠目录约定。

> 来源: docs/design/10-language-syntax.md §Code Organization；docs/plans/old/131-module-path-syntax-design.md、docs/plans/old/078-automan-integration.md；代码核对 use_scanner.rs、resolver.rs、ast/module_path.rs
