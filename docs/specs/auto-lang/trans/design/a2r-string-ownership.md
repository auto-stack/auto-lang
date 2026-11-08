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

> 来源: docs/design/06-code-generation.md §Rust Transpiler；docs/plans/archive/232-a2r-lexer-compilation.md、old/241-a2r-string-type-cleanup.md；crates/auto-lang/src/trans/rust.rs

## 两参 str.slice 的 Unicode 边界证据（PLAN-917，SD-03）

`str.slice(a, b)`（两参）自 PLAN-917 起发射带界字符位块：接收者按引用绑定一次、
`a`/`b` 各求值一次、`assert!(a <= b <= char_count, "auto str slice out of range")`
越界即 panic。语料 T08（`"中".slice(0, 2)`）的冻结 slice-oob 语义由此在旧 A2R 产物中
真实可观察（Rust panic 级行为观察；Auto trap 认证通道仍为独立 cert-gap）。
单参 `slice(a)` 形态与所有权约定不变。

## 文本长度域拆分与码点映射（PLAN-994，SD-04）

- **声明型字符串局部的长度双域**：显式声明 `str`/`Str`/`str(N)` 的 `let`/`var`
  局部上，`.len()`=标量字符数（`chars().count()`）、`.len_bytes()`=UTF-8 字节数
  （`String::len`）——两者在多字节文本上可区分（"A中😀"=3/8、emoji=1/4）。
  声明型 `char` 局部 `.to_codepoint()`=标量值（`as u32 as i64`，'😀'=128512）。
  发射为自包含 std 表达式、接收者单求值，不引入 a2r_std 依赖。
- **所有权/语境边界不变**：参数（`&str`）、字段、模式绑定、推断局部与调用结果
  维持本文既有映射与字节 len remap（已批准差异）；`str_slice_pattern_bindings`
  等 &str 绑定机制不受影响。
- **两参 slice 守卫块为 source 字符域唯一形态**：PLAN-956 的 a2r_std 字节域
  中间态（未重冻 917 金样，D-993-2 三红）已移除；`slice`/`sub` 于字符串接收者
  恢复 PLAN-917 守卫块（接收者 `&(recv)` 绑定一次、双界各一次、越界 panic），
  单参维持 chars-skip。runtime `a2r_std::str_slice`/`str_substr` 公共字节 API
  与其显式调用方（字节偏移游标域）不变。
