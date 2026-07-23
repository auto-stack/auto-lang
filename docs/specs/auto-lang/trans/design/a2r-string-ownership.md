# a2r 字符串所有权映射

> 范围：a2r（`trans/rust.rs`）中 Auto 字符串类型 → Rust `String`/`&str` 的映射机制。

## 背景

Auto 有统一字符串模型（`str` / `Str` / 字面量），Rust 区分 `&str` / `String` 所有权。
映射规则按**使用语境**分三个函数（`rust.rs:771/978/1030`）：

| 语境 | 函数 | 规则 |
|------|------|------|
| 变量/字段/容器类型标注 | `rust_type_name()` | 所有字符串类型 → `String`（存储语境，避开生命周期） |
| 函数参数 | `rust_param_type_name()` | 所有字符串类型 → `&str`（借用惯例） |
| 返回值 | `rust_return_type_name()` | 所有字符串类型 → `String`（返回 owned，防悬垂引用） |

## `.to_string()` 注入

字符串字面量或 `str` 值进入 `String` 槽位时注入 `.to_string()`，已知注入点包括：

1. 变量声明：`let x str = "hello"` → `let x: String = "hello".to_string()`
2. 容器 push（容器为 `Vec<String>`）
3. Map 插入（key/value 为 `String`）
4. return 表达式（函数返回 `String`）
5. struct 字段初始化（字段为 `String`，依赖 `struct_field_types` 缓存）

注入是启发式的，必然产生冗余/误注，因此配套 `RustTrans::post_process()`
（`rust.rs:10217`）在输出字节上做正则清理 pass：去重 import、`Vec.get(i)` → 下标、
`Option` 解包、String/&str 不匹配修正等，共 20+ 个 `fix_*`/`remove_*` 函数。

## 不变量

- 生成代码必须过 rustc 编译——宁多注入后清理，不可漏注入。
- `post_process` 只删冗余/修模式，不改变语义。
- 多文件项目走 `transpile_rust_project_merged` 时用独立的 `post_process_merged`。

## 已知缺陷（文档记录，未全修）

- 不需要处注入 `.to_string()`（如 `Command.args(vec!["-la"])`）
- 应借用的参数被加 `.clone()`
- `OsStr::to_str()` 被误映射为 `.to_string()`
- `File.delete()` 与 `HashMap.remove()` 方法名冲突
- `list.get(N)` 被改写为 `list[N as usize].clone()` 而非保留 `.get(N)`

## 显式非目标

- 不从类型系统层根除启发式（plan-241 选择继续打补丁路线）。
- 不生成带生命周期参数的 struct（字段一律 `String`，见 escape-analysis-tiers.md ADR-06 决策 1）。

> 来源: docs/design/06-code-generation.md §Rust Transpiler；docs/plans/old/232-a2r-lexer-compilation.md、old/241-a2r-string-type-cleanup.md；crates/auto-lang/src/trans/rust.rs
