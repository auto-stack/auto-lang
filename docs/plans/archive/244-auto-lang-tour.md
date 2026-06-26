# Plan 244: Auto Language Tour

> **Status**: ✅ Completed — 59 个 tour 示例全部通过 AutoVM(2026-06)
> **归档**: 本计划已归档至 `docs/plans/archive/`

## Context

需要为 Auto 语言创建一个类似 [Go Tour](https://tour.golang.org) 或 [Rust by Example](https://doc.rust-lang.org/rust-by-example/) 的 Language Tour，让新用户在 5-10 分钟内了解语言全貌。

现有 Cookbook 的 143 个示例经分析后，只有 46 个（32%）适合作为 Tour 素材，且缺少多个核心语言特性。需要从 Cookbook 筛选可用示例 + 编写补充示例，组织成递进式教学结构。

## Cookbook 现有示例评估

### 适合度分布

| 评级 | 数量 | 占比 |
|------|------|------|
| Good（可直接用） | 46 | 32% |
| Maybe（需修改） | 42 | 29% |
| Not suitable | 55 | 39% |

### Cookbook 覆盖良好的特性

| 特性 | 代表示例 |
|------|----------|
| `type`（struct） | `algorithms/003_sort_struct`, `encoding/001_json`, `encoding/002_toml` |
| `enum` 定义 | `algorithms/006_rand_custom`, `algorithms/010_rand_custom` |
| `ext` 方法扩展 | `asynchronous/channel/001_bounded`, `concurrency/003_actor` |
| `is` 模式匹配（Option/Result） | `encoding/003_csv_read`, `errors/001_boxed_error`, `text/005_filter_log` |
| `!` error fn + `.?` 传播 | `errors/002_anyhow`, `errors/004_retain`, `file/001_read_lines` |
| `~T` 异步 + `.await` | `asynchronous/001_join`, `asynchronous/002_timeout`, `asynchronous/rt/001_tokio_macro` |
| `List<T>` / `Map<K,V>` | `asynchronous/channel/001_bounded`, `database/postgres/003_aggregate` |
| 2D 数组 + 嵌套循环 | `science/linear_algebra/001_add_matrices`, `002_multiply_matrices` |
| Lambda / 闭包 | `algorithms/002_sort_float`, `algorithms/003_sort_struct` |
| `use.rust` / `dep` | 几乎所有 B-tier 文件 |
| f-string 插值 | 几乎所有文件 |
| `b"..."` / `r"..."` 字符串 | `cryptography/001_sha_digest`, `text/001_regex_replace` |
| 文件 I/O | `file/001_read_lines`, `file/014_read_lines_temp` |
| 进程操作 | `os/002_process_continuous`, `os/004_piped` |

### Cookbook 缺失的核心特性

这些是 Language Tour 必须覆盖但 Cookbook 中完全没有的特性：

| 缺失特性 | 重要性 | 说明 |
|----------|--------|------|
| `spec`（trait）定义与实现 | 高 | Auto 的核心抽象机制，Cookbook 中零出现 |
| `const` 常量声明 | 高 | 只在个别文件有 `dep`，无 `const` 示例 |
| `shared` 静态存储 | 中 | 进程级生命周期变量 |
| `alias` 类型别名 | 中 | 无示例 |
| 泛型函数 `<T>` | 高 | 只用了 `List<T>` 等内置泛型，无自定义泛型函数 |
| 元组类型 / 解构 | 中 | 完全没有元组示例 |
| `if / else if / else` 链 | 高 | 只有独立 `if`，无 else 分支 |
| `loop` 无限循环 | 中 | 无示例 |
| `pub` 可见性修饰 | 中 | 无示例 |
| 模块系统 / 多文件 | 中 | 无 `use`（Auto 原生导入）示例 |
| 类型解构 | 中 | 无 `let Point(x, y) = ...` 示例 |
| 迭代器适配器链 | 中 | 无 `.map().filter().collect()` 链 |
| `if` 表达式（三值） | 低 | 无条件表达式作为值 |
| `#[...]` 属性注解 | 低 | 无示例 |
| `for cond { }` 条件循环 | 中 | Auto 用 `for` 替代 `while`，但 Cookbook 中无此类用法 |

## Language Tour 结构设计

### 递进式章节

```
Chapter 1: Hello Auto          — print, let, var, f-string, comments
Chapter 2: Types               — type (struct), enum, field access, tuple
Chapter 3: Functions           — fn, params, return types, closures
Chapter 4: Control Flow        — if/else, for, loop, break, for-cond
Chapter 5: Pattern Matching    — is, enum destructuring, Option, Result
Chapter 6: Error Handling      — ! fn, .?, Ok/Err, ?? coalescing
Chapter 7: Collections         — List, Map, array, iteration, iterator chains
Chapter 8: Methods & Extensions — ext, mut fn, static fn
Chapter 9: Generics            — <T>, spec (trait), constraints
Chapter 10: Modules            — use, pub, multi-file, use.rust
Chapter 11: Async              — ~T, .await, async blocks
Chapter 12: Interop            — dep, use.rust, sys (unsafe)
```

### 每章所需示例数

| 章节 | 现有可用 | 需新写 | 小计 |
|------|---------|--------|------|
| 1. Hello Auto | 2 | 3 | 5 |
| 2. Types | 5 | 4 | 9 |
| 3. Functions | 3 | 4 | 7 |
| 4. Control Flow | 2 | 5 | 7 |
| 5. Pattern Matching | 4 | 3 | 7 |
| 6. Error Handling | 4 | 2 | 6 |
| 7. Collections | 5 | 3 | 8 |
| 8. Methods & Extensions | 3 | 3 | 6 |
| 9. Generics | 0 | 5 | 5 |
| 10. Modules | 1 | 4 | 5 |
| 11. Async | 4 | 2 | 6 |
| 12. Interop | 3 | 3 | 6 |
| **合计** | **36** | **41** | **77** |

## 从 Cookbook 筛选的 36 个可用示例

### Chapter 1: Hello Auto
- `algorithms/001_sort_int` → 简单排序 + print
- `cli/002_ansi_term` → 字符串拼接 + 转义

### Chapter 2: Types
- `algorithms/003_sort_struct` → type + named fields + sort_by
- `encoding/001_json/json` → type + serde 序列化
- `encoding/002_toml/toml` → 嵌套 type
- `encoding/006_endian_byte` → 类型注解 + 字节数组
- `data_structures/001_bitfield` → 算术 + 布尔

### Chapter 3: Functions
- `science/statistics/001_central_tendency` → fn 签名 + Option
- `science/statistics/002_standard_deviation` → 完整函数示例
- `text/007_from_str` → 自定义解析 + type 构造

### Chapter 4: Control Flow
- `science/linear_algebra/001_add_matrices` → 嵌套 for 循环
- `datetime/001_elapsed_time` → for + +=

### Chapter 5: Pattern Matching
- `encoding/003_csv_read` → is Ok/Err
- `errors/001_boxed_error` → Result 匹配
- `concurrency/006_rayon_parallel_search` → is Some/None
- `text/005_filter_log` → is + 正则组合

### Chapter 6: Error Handling
- `errors/002_anyhow` → ! fn + .?
- `errors/004_retain` → Ok() 返回
- `file/001_read_lines` → 文件 + 错误传播
- `os/001_env_variable` → env + .?

### Chapter 7: Collections
- `science/linear_algebra/007_deserialize_matrix` → 动态数组构建
- `file/005_duplicate_name` → HashMap
- `file/008_loops` → HashSet
- `database/postgres/003_aggregate` → Map 操作
- `science/trigonometry/003_latitude_longitude` → 方法链

### Chapter 8: Methods & Extensions
- `asynchronous/channel/001_bounded` → ext + static fn
- `concurrency/003_actor` → 复杂 ext + mut fn

### Chapter 9: Generics
- （无可用示例，全部需新写）

### Chapter 10: Modules
- `os/002_process_continuous` → use.rust + Command

### Chapter 11: Async
- `asynchronous/rt/001_tokio_macro` → ~T + .await
- `asynchronous/002_timeout` → async 无依赖
- `concurrency/004_custom_future` → ~void async main
- `hardware/001_cpu_count` → stdlib 调用

### Chapter 12: Interop
- `cryptography/001_sha_digest` → dep + b"..."
- `text/001_regex_replace` → r"..." raw string
- `web/url/001_base` → use.rust url
- `encoding/005_hex` → 字节编码

## 需新写的 41 个示例

### Chapter 1: Hello Auto（3 个）
1. Hello World — `fn main() { print("Hello, World!") }`
2. 变量与类型推断 — `let`, `var`, 基本类型
3. 注释 — 单行 `//`，文档注释

### Chapter 2: Types（4 个）
1. 基本类型 — int, float, bool, str, byte
2. 元组 — `(1, "hello")`，解构
3. 可空类型 — `?int`，Some/None 构造
4. 类型别名 — `alias UserID = int`

### Chapter 3: Functions（4 个）
1. 函数基础 — 参数，返回值，无返回
2. 默认参数 — `fn greet(name, greeting = "Hello")`
3. 泛型函数 — `fn identity<T>(x T) T`
4. 闭包与捕获 — `let add = (a, b) => a + b`

### Chapter 4: Control Flow（5 个）
1. if/else — 含 else if 链
2. for-in — 范围、集合、索引
3. for-cond — `for i < 10 { ... }`（替代 while）
4. loop + break — 无限循环 + break/return
5. if 表达式 — `let x = if cond { a } else { b }`

### Chapter 5: Pattern Matching（3 个）
1. is 基础 — 值匹配，else
2. is 解构 — `is point { Point(x, y) -> ... }`
3. is 类型守卫 — `as str`, `if x > 10`

### Chapter 6: Error Handling（2 个）
1. Result 完整流程 — 定义错误，传播，匹配
2. ?? 空值合并 — `name ?? "unknown"`

### Chapter 7: Collections（3 个）
1. 数组与切片 — `[1,2,3]`, `[N]int`, `[]int`
2. 迭代器适配器 — `.map()`, `.filter()`, `.collect()`
3. 对象字面量 — `{key: value}`

### Chapter 8: Methods & Extensions（3 个）
1. 内联方法 — `type` 内的 `fn` / `mut fn` / `static fn`
2. ext 扩展 — 为已有类型添加方法
3. 方法链 — builder pattern

### Chapter 9: Generics（5 个）
1. 泛型函数 — `fn first<T>(list List<T>) ?T`
2. 泛型类型 — `type Box<T> { value T }`
3. spec 定义 — `spec Printable { fn print() }`
4. spec 实现 — `type Foo has Printable { ... }`
5. spec 约束 — `fn compare<T has Comparable>(a T, b T)`

### Chapter 10: Modules（4 个）
1. use 导入 — `use math::add`
2. pub 可见性 — `pub fn`, `pub type`
3. 多文件模块 — main.at + helper.at
4. use.rust 进阶 — 常用 stdlib 导入模式

### Chapter 11: Async（2 个）
1. async 函数 — `fn fetch() ~str` + `.await`
2. async 块 — `let future = ~{ ... }`

### Chapter 12: Interop（3 个）
1. dep 声明 — `dep serde`, `dep regex`
2. sys 块 — `sys { ... }` unsafe 操作
3. 外部函数 — `extern fn`

## 输出格式

每个示例一个 `.at` 文件，放在 `docs/tour/` 或 `website/src/tour/` 下，同时提供：

1. **Auto 代码** — 可运行的 `.at` 文件
2. **Rust 对照** — 等价 Rust 代码（side-by-side 展示）
3. **说明文字** — 1-2 句解释该特性

可参考 Rust by Example 的格式：每页一个主题，代码 + 解释。

## 验证

```bash
# 所有 Tour 示例必须通过 AutoVM
for f in docs/tour/*.at; do
    cargo run --quiet -- "$f" || echo "FAIL: $f"
done

# 所有 Tour 示例必须通过 a2r transpile
for f in docs/tour/*.at; do
    cargo run --quiet -- trans --path "$f" rust || echo "FAIL: $f"
done
```

## 实施优先级

1. **Phase 1**: Chapter 1-6（基础部分，20 个新示例）— 最小可发布
2. **Phase 2**: Chapter 7-9（进阶部分，11 个新示例）
3. **Phase 3**: Chapter 10-12（高级部分，10 个新示例）
