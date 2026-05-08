# Plan 240: Rust Cookbook a2r 测试集

**日期**: 2026-05-08
**状态**: Phase 4 已完成
**目标**: 利用 Rust Cookbook 的真实示例建立系统化的 a2r 测试集，通过对比 a2r 输出与 Rust 原始代码来发现和修复 a2r 的问题

## 1. 概述

从 Rust Cookbook (D:\github\rust-cookbook) 收集所有示例，按翻译难度分为三个层次（A/B/C）。先实现 A 层（纯 stdlib，可直接翻译），再实现 B 层（外部 crate），记录 C 层留待后续。

每个测试用例包含：
- `{name}.at` — Auto 版本
- `{name}.expected.rs` — a2r 实际输出的 Rust 代码（精确字符串匹配）
- `reference.rs` — Cookbook 原始 Rust 代码（仅作参考对比，不参与测试断言）

## 2. 测试目录结构

```
test/a2r/cookbook/
├── algorithms/
│   ├── 001_sort_int/ ... 003_sort_struct/    (A-tier)
│   └── 004_rand/ ... 009_rand_range/         (B-tier)
├── cli/001_clap_basic/                        (B-tier)
├── compression/001_tar_compress/ ... 002/     (B-tier)
├── concurrency/001_rayon_any_all/ ... 004/    (B-tier)
├── cryptography/001_sha_digest/               (B-tier)
├── datetime/
│   ├── 001_elapsed_time/                      (A-tier)
│   └── 002_checked/ ... 007_timestamp/        (B-tier)
├── encoding/001_json/ ... 005_hex/            (B-tier)
├── errors/
│   ├── 001_boxed_error/                       (A-tier)
│   └── 002_anyhow/                            (B-tier)
├── file/
│   ├── 001_read_lines/                        (A-tier)
│   └── 002_find_files/ ... 004_modified/      (B-tier)
├── mem/001_lazy_cell/                         (A-tier)
├── os/001_env_variable/ ... 003_error_file/   (A-tier)
├── science/mathematics/
│   ├── statistics/001_central_tendency/ ... 002/  (A-tier)
│   ├── trigonometry/001_tan_sin_cos/ ... 003/    (A-tier)
│   └── complex_numbers/001_add_complex/ ... 002/ (B-tier)
├── text/001_regex_replace/ ... 004_graphemes/ (B-tier)
├── versioning/001_semver_parse/ ... 003/      (B-tier)
└── web/url/001_base/ ... 003_fragment/        (B-tier)
```

测试注册在 `crates/auto-lang/src/tests/a2r_tests.rs` 中，复用现有的 `test_a2r()` 函数。

## 3. Tier A — 可直接翻译（纯 Rust stdlib）

共 **15 个示例**，全部使用 Rust 标准库，不依赖外部 crate。

### 3.1 algorithms/sorting（3 个）

| # | 测试目录 | 标题 | 关键特性 |
|---|---------|------|---------|
| A-01 | algorithms/001_sort_int | Sort a Vector of Integers | `vec.sort()`, `assert_eq!` |
| A-02 | algorithms/002_sort_float | Sort a Vector of Floats | `vec.sort_by()`, `partial_cmp` |
| A-03 | algorithms/003_sort_struct | Sort a Vector of Structs | `#[derive(Ord,PartialOrd)]`, `sort_by` 闭包 |

### 3.2 file（1 个）

| # | 测试目录 | 标题 | 关键特性 |
|---|---------|------|---------|
| A-04 | file/001_read_lines | Read lines of strings from a file | `File::create`, `File::open`, `BufReader`, `lines()`, `Result`, `?` |

### 3.3 os（3 个）

| # | 测试目录 | 标题 | 关键特性 |
|---|---------|------|---------|
| A-05 | os/001_env_variable | Read Environment Variable | `env::var`, `unwrap_or`, `fs::read_to_string`, `Result` |
| A-06 | os/002_process_continuous | Continuously process child process outputs | `Command`, `Stdio::piped`, `BufReader`, `filter_map`, 闭包 |
| A-07 | os/003_error_file | Redirect stdout and stderr to file | `File::create`, `File::try_clone`, `Command`, `Stdio` |

### 3.4 datetime（1 个）

| # | 测试目录 | 标题 | 关键特性 |
|---|---------|------|---------|
| A-08 | datetime/001_elapsed_time | Measure elapsed time | `Instant::now`, `elapsed()`, `Duration` |

### 3.5 science/mathematics/statistics（2 个）

| # | 测试目录 | 标题 | 关键特性 |
|---|---------|------|---------|
| A-09 | science/mathematics/statistics/001_central_tendency | Mean, median, mode | `Vec`, 迭代器, 排序, 数值计算 |
| A-10 | science/mathematics/statistics/002_standard_deviation | Standard deviation | 数值计算, `sqrt`, 迭代器 |

### 3.6 science/mathematics/trigonometry（3 个）

| # | 测试目录 | 标题 | 关键特性 |
|---|---------|------|---------|
| A-11 | science/mathematics/trigonometry/001_tan_sin_cos | Verify tan = sin/cos | `f64`, `tan`, `sin`, `cos` |
| A-12 | science/mathematics/trigonometry/002_side_length | Calculate triangle side length | `sqrt`, 数值运算 |
| A-13 | science/mathematics/trigonometry/003_latitude_longitude | Distance between two points on Earth | `f64`, `sin`, `cos`, `sqrt`, `powi` |

### 3.7 mem（1 个）

| # | 测试目录 | 标题 | 关键特性 |
|---|---------|------|---------|
| A-14 | mem/001_lazy_cell | Declare lazily evaluated constant | `LazyCell`, `OnceCell`, 闭包初始化 |

### 3.8 errors（1 个）

| # | 测试目录 | 标题 | 关键特性 |
|---|---------|------|---------|
| A-15 | errors/001_boxed_error | Handle errors in main (Box<dyn Error> 部分) | `Box<dyn Error>`, `Result`, `?` |

## 4. Tier B — 外部 crate 测试（41 个已实现）

已从 107 个中选取 41 个代表性示例，覆盖所有主要外部 crate 依赖类型。

### 4.1 已实现列表

| # | 测试目录 | 标题 | 外部 crate |
|---|---------|------|-----------|
| B-01 | algorithms/004_rand | Generate random numbers | rand |
| B-02 | algorithms/005_rand_choose | Choose random element | rand |
| B-03 | algorithms/006_rand_custom | Random values of custom type | rand |
| B-04 | algorithms/007_rand_dist | Random number generation | rand |
| B-05 | algorithms/008_rand_passwd | Generate random password | rand |
| B-06 | algorithms/009_rand_range | Random numbers in range | rand |
| B-07 | cli/001_clap_basic | Parse command line arguments | clap |
| B-08 | compression/001_tar_compress | Compress to tarball | tar, flate2 |
| B-09 | compression/002_tar_decompress | Decompress tarball | tar, flate2 |
| B-10 | concurrency/001_rayon_any_all | Parallel any/all test | rayon |
| B-11 | concurrency/002_rayon_map_reduce | Parallel map-reduce | rayon |
| B-12 | concurrency/003_rayon_parallel_sort | Parallel sort | rayon |
| B-13 | concurrency/004_crossbeam_spsc | SPSC channel | crossbeam |
| B-14 | cryptography/001_sha_digest | SHA-256 digest | sha2 |
| B-15 | datetime/002_checked | Checked datetime calc | chrono |
| B-16 | datetime/003_timezone | Timezone conversion | chrono |
| B-17 | datetime/004_current | Examine date and time | chrono |
| B-18 | datetime/005_format | Format date and time | chrono |
| B-19 | datetime/006_parse_string | Parse string to DateTime | chrono |
| B-20 | datetime/007_timestamp | Date to UNIX timestamp | chrono |
| B-21 | versioning/001_semver_parse | Parse version string | semver |
| B-22 | versioning/002_semver_increment | Parse and increment version | semver |
| B-23 | versioning/003_semver_latest | Find latest version | semver |
| B-24 | encoding/001_json | Serialize/deserialize JSON | serde_json |
| B-25 | encoding/002_toml | Deserialize TOML | toml, serde |
| B-26 | encoding/003_csv_read | Read CSV records | csv |
| B-27 | encoding/004_base64 | Base64 encode/decode | base64 |
| B-28 | encoding/005_hex | Hex encode/decode | data_encoding |
| B-29 | errors/002_anyhow | Error handling with anyhow | anyhow |
| B-30 | file/002_find_files | Find files with walkdir | walkdir |
| B-31 | file/003_recursive_size | Calculate file sizes | walkdir |
| B-32 | file/004_modified | Find modified files | walkdir |
| B-33 | science/.../001_add_complex | Add complex numbers | num |
| B-34 | science/.../002_create_complex | Create complex numbers | num |
| B-35 | text/001_regex_replace | Replace text pattern | regex |
| B-36 | text/002_regex_email | Verify email with regex | regex |
| B-37 | text/003_regex_hashtags | Extract hashtags | regex |
| B-38 | text/004_graphemes | Unicode graphemes | unicode_segmentation |
| B-39 | web/url/001_base | Base URL | url |
| B-40 | web/url/002_parse | Parse URL query params | url |
| B-41 | web/url/003_fragment | URL fragment | url |

### 4.2 未实现的 B-tier 示例（62 个）

以下示例因 Auto 解析器限制或复杂度过高而暂未实现：

- **algorithms/randomness**: rand-custom（enum Distribution impl）、rand-dist（rand_distr 依赖）
- **cli**: ansi_term-basic（ANSI escape codes）
- **compression**: tar-strip-prefix（复杂 anyhow 链）
- **concurrency**: rayon-iter-mut, rayon-parallel-search, rayon-thumbnails（image 依赖）, crossbeam-complex, crossbeam-spawn, global-mut-state, threadpool-fractal, threadpool-walk
- **cryptography**: pbkdf2（ring + rand + data_encoding）, hmac（ring + rand）
- **development_tools/debugging**: 所有 9 个日志/追踪测试（env_logger, log4rs, tracing）
- **development_tools/versioning**: semver-command（ring + anyhow）, semver-complex, semver-prerelease
- **encoding**: endian-byte（byteorder）, csv/delimiter, csv/filter, csv/invalid, csv/serde-serialize, csv/serialize, csv/transform, percent-encode, url-encode
- **errors**: backtrace（anyhow + csv + serde）, retain（reqwest + thiserror）
- **file/dir**: duplicate-name, find-file, ignore-case, loops, png, recursive, sizes, skip-dot
- **file/read-write**: same-file
- **file/read**: read_lines（tempfile 版本）
- **hardware**: cpu-count（num_cpus）
- **os/external**: piped, process-output, send-input（均依赖 ring）
- **safety_critical**: heapless-alloc（heapless）
- **science/mathematics**: linear_algebra 全部 7 个（ndarray/nalgebra）, mathematical-functions（num）, big-integers（num）
- **text**: filter-log（regex + anyhow）, phone（regex + anyhow）, from_str（num）
- **web/mime**: filename（image）, string（mime）
- **web/url**: new, origin（url）

## 5. Tier C — 暂不可行（42 个）

涉及 async/await、tokio、网络编程、unsafe、数据库驱动、FFI、web framework 等，a2r 当前不支持。

### 5.1 async/tokio（12 个）

```
asynchronous/channel/bounded.md          — Bounded Channels (async, tokio)
asynchronous/channel/unbounded.md        — Unbounded Channels (async, tokio)
asynchronous/fs/create.md                — Async create files (async, tokio)
asynchronous/fs/read.md                  — Async read files (async, tokio)
asynchronous/fs/remove.md                — Async remove files (async, tokio)
asynchronous/fs/rw_traits.md             — AsyncRead/AsyncWrite (async, tokio)
asynchronous/fs/write.md                 — Async write files (async, tokio)
asynchronous/ftc/ctrl_c.md               — Ctrl+C handling (async, tokio)
asynchronous/join.md                     — Structured concurrency (async, tokio)
asynchronous/rt/tokio-rt-builder.md      — Tokio runtime builder (async, tokio)
asynchronous/rt/tokio-rt-macro.md        — Tokio macro (async, tokio)
asynchronous/timeout.md                  — Async timeout (async, tokio)
```

### 5.2 高级并发（2 个）

```
concurrency/actor/actor-pattern.md       — Actor pattern with Tokio (async, tokio)
concurrency/custom_future/custom-future.md — Custom Future impl (Pin, Waker)
```

### 5.3 数据库（6 个）

```
database/postgres/aggregate_data.md      — Aggregate data (postgres)
database/postgres/create_tables.md       — Create tables (postgres)
database/postgres/insert_query_data.md   — Insert/query data (postgres)
database/sqlite/initialization.md        — SQLite init (rusqlite)
database/sqlite/insert_select.md         — SQLite insert/select (rusqlite)
database/sqlite/transactions.md          — SQLite transactions (rusqlite)
```

### 5.4 unsafe/低级（4 个）

```
file/read-write/memmap.md                — Memory-mapped file (unsafe, memmap)
net/server/listen-unused.md              — TCP listen (TcpListener)
safety_critical/no_panic/no-panic.md     — No-panic guarantee (no-panic proc macro)
data_structures/bitfield/bitfield.md     — Bitfield type (bitflags, no_std)
```

### 5.5 FFI/构建工具（3 个）

```
development_tools/build_tools/cc-bundled-cpp.md    — Compile and link bundled C++ library (cc)
development_tools/build_tools/cc-bundled-static.md  — Compile and link bundled C static library (cc)
development_tools/build_tools/cc-defines.md         — Define C compiler flags (cc)
```

### 5.6 async 网络/reqwest（14 个）

```
web/mime/request.md                      — MIME from HTTP (async, reqwest)
web/scraping/broken.md                   — Broken link check (async, reqwest)
web/scraping/extract-links.md            — Extract links (async, reqwest)
web/scraping/unique.md                   — Unique links (async, reqwest)
web/clients/api/paginated.md             — Paginated RESTful API (async, reqwest, serde)
web/clients/api/rest-get.md              — Query GitHub API (async, reqwest, serde)
web/clients/api/rest-head.md             — Check API resource (async, reqwest)
web/clients/api/rest-post.md             — Create/delete Gist (async, reqwest, serde, anyhow)
web/clients/authentication/basic.md      — Basic auth (async, reqwest)
web/clients/download/basic.md            — Download file (async, reqwest, anyhow, tempfile)
web/clients/download/partial.md          — Partial download (async, reqwest, anyhow)
web/clients/download/post-file.md        — POST file (async, reqwest, anyhow, ring)
web/clients/requests/get.md              — HTTP GET (async, reqwest, anyhow, ring)
web/clients/requests/header.md           — Custom headers (async, reqwest, serde, url, anyhow)
```

### 5.7 Web framework（1 个）

```
web/leptos.md                            — Full stack web with Leptos framework
```

## 6. 执行步骤

### Phase 1: 基础设施搭建 ✅ 完成
1. 创建 `test/a2r/cookbook/` 目录结构
2. 在 `a2r_tests.rs` 中注册 15 个测试函数
3. 验证测试框架正确运行

### Phase 2: A 层测试用例实现 ✅ 完成
15 个测试用例全部创建完成，所有测试通过。

### Phase 3: 分析 a2r 问题 ✅ 完成
分析 P1-P13 问题，修复了 P1（数组类型）和 P4（缺少 Ok(())）。

### Phase 4: B 层测试用例实现 ✅ 完成
41 个 B-tier 测试用例创建完成，所有 56 个 cookbook 测试通过。

### Phase 5: 修复 a2r 问题 ⬅️ 下一阶段
根据 B-tier 测试结果修复更多 a2r 问题。

## 7. a2r 问题分析

### 7.1 已修复

| # | 问题 | 修复 | 提交 |
|---|------|------|------|
| P1 | 数组类型推断为 `[T; N]` 而非 `Vec<T>` | `rust_type_name()` 中 `Type::Array` 输出 `Vec<T>` | e6baefad |
| P4 | 缺少函数末尾 `Ok(())` | `body()` 方法末尾追加 `Ok(())` | e6baefad |

### 7.2 待修复（高优先级）

| # | 问题 | 影响的测试 | 描述 |
|---|------|-----------|------|
| P2 | `fn() !` 返回 `Result<(), String>` 而非正确的错误类型 | read_lines, env_variable, boxed_error | `?` 操作符无法从 `io::Error` 等类型自动转为 `String` |
| P3 | 方法链被拆解为 `self.xxx()` 语句 | process_continuous, error_file | Builder pattern 不可用 |
| P5 | `.to_f64()` 不存在于 `i32`/`usize` | central_tendency, standard_deviation | 已在 .at 中用 `.as(f64)` 规避 |

### 7.3 待修复（中优先级）

| # | 问题 | 影响的测试 | 描述 |
|---|------|-----------|------|
| P6 | type derive 缺少 `Ord`/`Eq` | sort_struct | 排序需要 `Ord` derive |
| P7 | `Duration` 使用 `{}` 而非 `{:?}` | elapsed_time | `Duration` 不实现 `Display` |
| P8 | `.cmp()` 参数缺少引用 | sort_struct | 应为 `.cmp(&a.age)` |
| P9 | 操作符优先级错误 | latitude_longitude | 方法调用优先级高于除法 |
| P10 | `use.rust std::error::Error` 不生成 `Box<dyn Error>` | boxed_error | 错误类型映射不正确 |

### 7.4 B-tier 新发现的问题

| # | 问题 | 描述 |
|---|------|------|
| B-P1 | **Auto 解析器不支持表达式中的 `::`** | 只在 `use.rust` 语句中支持。所有 B-tier .at 文件用 `.` 点号替代 `Type::method()` |
| B-P2 | **`use.rust crate::{A, B}` 花括号导入不支持** | 需拆为多行单独导入 |
| B-P3 | **`use.rust path::*` 通配符导入不支持** | 如 `rayon::prelude::*` |
| B-P4 | **`|x|` 闭包语法不支持** | Auto 用 `x => expr`，但迭代器中 `.map(|x| ...)` 需要特殊处理 |

## 8. 统计

| 层次 | Cookbook 总数 | 已实现 | 状态 |
|------|-------------|--------|------|
| Tier A | 15 | 15 | ✅ 全部完成 |
| Tier B | 103 | 41 | ✅ 代表性子集已完成，62 个待后续 |
| Tier C | 42 | 0 | 暂不处理（需 async/unsafe/FFI 支持） |
| **总计** | **160** | **56** | **35% 覆盖** |

## 9. 下一步

1. **修复 B-P1（表达式中的 `::`）** — 让解析器支持 `Type::method()` 语法，消除手动 `.` 替换
2. **修复 P2（错误类型映射）** — `fn() !` 应根据 `use.rust` 导入推断正确的错误类型
3. **修复 P3（方法链）** — Builder pattern 是 Rust 的核心惯用法
4. **补充更多 B-tier 测试** — 优先覆盖 encoding/csv、development_tools/debugging 等类别
5. **评估 C-tier 可行性** — 随着异步支持（`~T` 返回类型）的完善，部分 C-tier 可能降级为 B-tier
