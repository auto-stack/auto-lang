# Plan 240: Rust Cookbook a2r 测试集

**日期**: 2026-05-08
**状态**: 待执行
**目标**: 利用 Rust Cookbook 的真实示例建立系统化的 a2r 测试集，通过对比 a2r 输出与 Rust 原始代码来发现和修复 a2r 的问题

## 1. 概述

从 Rust Cookbook (D:\github\rust-cookbook) 收集所有示例，按翻译难度分为三个层次（A/B/C）。先实现 A 层（纯 stdlib，可直接翻译），记录 B/C 层留待后续。

每个测试用例包含：
- `input.at` — Auto 版本（用 /auto-lang-creator 生成）
- `input.expected.rs` — a2r 实际输出的 Rust 代码（方案 A：精确字符串匹配）
- `reference.rs` — Cookbook 原始 Rust 代码（仅作参考对比，不参与测试断言）

## 2. 测试目录结构

```
test/a2r/cookbook/
├── algorithms/
│   ├── 001_sort_int/
│   │   ├── sort_int.at
│   │   ├── sort_int.expected.rs
│   │   └── reference.rs
│   ├── 002_sort_float/
│   ├── 003_sort_struct/
│   └── ...
├── file/
│   ├── 001_read_lines/
│   └── ...
├── os/
│   ├── 001_env_variable/
│   ├── 002_process_output/
│   └── ...
├── datetime/
│   └── ...
├── science/
│   ├── mathematics/
│   │   ├── statistics/
│   │   └── trigonometry/
│   └── ...
└── errors/
    └── ...
```

测试注册在 `crates/auto-lang/src/tests/a2r_tests.rs` 中，复用现有的 `test_a2r()` 函数。

## 3. Tier A — 可直接翻译（纯 Rust stdlib）

共 **15 个示例**，全部使用 Rust 标准库，不依赖外部 crate。

### 3.1 algorithms/sorting（3 个）

| # | Cookbook 路径 | 标题 | 关键特性 |
|---|--------------|------|---------|
| A-01 | algorithms/sorting/sort.md | Sort a Vector of Integers | `vec.sort()`, `assert_eq!` |
| A-02 | algorithms/sorting/sort_float.md | Sort a Vector of Floats | `vec.sort_by()`, `partial_cmp` |
| A-03 | algorithms/sorting/sort_struct.md | Sort a Vector of Structs | `#[derive(Ord,PartialOrd)]`, `sort_by` 闭包 |

### 3.2 file（1 个）

| # | Cookbook 路径 | 标题 | 关键特性 |
|---|--------------|------|---------|
| A-04 | file/read-write/read-file.md | Read lines of strings from a file | `File::create`, `File::open`, `BufReader`, `lines()`, `Result`, `?` |

### 3.3 os（3 个）

| # | Cookbook 路径 | 标题 | 关键特性 |
|---|--------------|------|---------|
| A-05 | os/external/read-env-variable.md | Read Environment Variable | `env::var`, `unwrap_or`, `fs::read_to_string`, `Result` |
| A-06 | os/external/continuous.md | Continuously process child process outputs | `Command`, `Stdio::piped`, `BufReader`, `filter_map`, 闭包 |
| A-07 | os/external/error-file.md | Redirect stdout and stderr to file | `File::create`, `File::try_clone`, `Command`, `Stdio` |

### 3.4 datetime（1 个）

| # | Cookbook 路径 | 标题 | 关键特性 |
|---|--------------|------|---------|
| A-08 | datetime/duration/profile.md | Measure elapsed time | `Instant::now`, `elapsed()`, `Duration` |

### 3.5 science/mathematics/statistics（2 个）

| # | Cookbook 路径 | 标题 | 关键特性 |
|---|--------------|------|---------|
| A-09 | science/mathematics/statistics/central-tendency.md | Mean, median, mode | `Vec`, 迭代器, 排序, 数值计算 |
| A-10 | science/mathematics/statistics/standard-deviation.md | Standard deviation | 数值计算, `sqrt`, 迭代器 |

### 3.6 science/mathematics/trigonometry（3 个）

| # | Cookbook 路径 | 标题 | 关键特性 |
|---|--------------|------|---------|
| A-11 | science/mathematics/trigonometry/tan-sin-cos.md | Verify tan = sin/cos | `f64`, `tan`, `sin`, `cos` |
| A-12 | science/mathematics/trigonometry/side-length.md | Calculate triangle side length | `sqrt`, 数值运算 |
| A-13 | science/mathematics/trigonometry/latitude-longitude.md | Distance between two points on Earth | `f64`, `sin`, `cos`, `sqrt`, `powi` |

### 3.7 mem（1 个）

| # | Cookbook 路径 | 标题 | 关键特性 |
|---|--------------|------|---------|
| A-14 | mem/global_static/lazy-constant.md | Declare lazily evaluated constant | `LazyCell`, `OnceCell`, 闭包初始化 |

### 3.8 errors（1 个）

| # | Cookbook 路径 | 标题 | 关键特性 |
|---|--------------|------|---------|
| A-15 | errors/handle/main.md | Handle errors in main (Box<dyn Error> 部分) | `Box<dyn Error>`, `Result`, `?` |

## 4. Tier B — 需要外部 crate 支持（107 个）

需要在 a2r 中实现 `use.rust` 或等效的外部 crate 导入机制后才能翻译。

### 4.1 按依赖分类

| 外部 crate | 示例数 | 涉及的 Cookbook 分类 |
|-----------|--------|---------------------|
| rand / rand_distr | 7 | algorithms/randomness |
| chrono | 6 | datetime/duration, datetime/parse |
| serde / serde_json | 8 | encoding/complex, encoding/csv |
| csv | 6 | encoding/csv |
| regex | 5 | text/regex |
| rayon | 5 | concurrency/parallel |
| crossbeam | 3 | concurrency/thread |
| toml | 1 | encoding/complex |
| clap | 1 | cli/arguments |
| tar / flate2 | 3 | compression/tar |
| ring / sha2 / hmac | 4 | cryptography |
| semver | 5 | development_tools/versioning |
| base64 / data_encoding | 3 | encoding/string |
| num | 5 | science/mathematics, text/string_parsing |
| ndarray / nalgebra | 7 | science/mathematics/linear_algebra |
| env_logger / log4rs / tracing | 8 | development_tools/debugging |
| anyhow / thiserror | 15+ | 多个分类（错误处理） |
| walkdir / glob | 8 | file/dir |
| url / mime | 8 | web/url, web/mime |
| reqwest | 10 | web/clients（部分涉及 async） |
| heapless | 1 | safety_critical |
| unicode_segmentation | 1 | text/string_parsing |
| image | 2 | file/dir, concurrency |
| num_cpus / threadpool | 3 | hardware, concurrency |
| percent_encoding | 1 | encoding/string |
| tempfile | 2 | file/read |
| syslog | 1 | development_tools/debugging |
| byteorder | 1 | encoding/complex |
| same_file | 1 | file/read-write |
| ansi_term | 1 | cli/ansi_terminal |

### 4.2 B 层完整列表

```
algorithms/randomness/rand.md                     — Generate random numbers (rand)
algorithms/randomness/rand-choose.md              — Create random passwords from user chars (rand)
algorithms/randomness/rand-custom.md              — Generate random values of custom type (rand)
algorithms/randomness/rand-dist.md                — Generate random numbers with distribution (rand, rand_distr)
algorithms/randomness/rand-passwd.md              — Create random passwords from alphabet (rand)
algorithms/randomness/rand-range.md               — Generate random numbers in range (rand)
cli/arguments/clap-basic.md                       — Parse command line arguments (clap)
cli/ansi_terminal/ansi_term-basic.md              — ANSI Terminal colors (ansi_term)
compression/tar/tar-compress.md                   — Compress directory to tarball (tar, flate2)
compression/tar/tar-decompress.md                 — Decompress tarball (tar, flate2)
compression/tar/tar-strip-prefix.md               — Decompress with prefix strip (tar, flate2, anyhow)
concurrency/parallel/rayon-any-all.md             — Parallel any/all test (rayon)
concurrency/parallel/rayon-iter-mut.md            — Parallel mutable iteration (rayon)
concurrency/parallel/rayon-map-reduce.md          — Parallel map-reduce (rayon)
concurrency/parallel/rayon-parallel-search.md     — Parallel search (rayon)
concurrency/parallel/rayon-parallel-sort.md       — Parallel sort (rayon, rand)
concurrency/parallel/rayon-thumbnails.md          — Parallel thumbnails (rayon, image, anyhow, glob)
concurrency/thread/crossbeam-complex.md           — Parallel pipeline (crossbeam)
concurrency/thread/crossbeam-spsc.md              — SPSC channel (crossbeam)
concurrency/thread/crossbeam-spawn.md             — Spawn short-lived thread (crossbeam)
concurrency/thread/global-mut-state.md            — Global mutable state (anyhow)
concurrency/thread/threadpool-fractal.md          — Fractal with threadpool (threadpool, image, anyhow, num, num_cpus)
concurrency/thread/threadpool-walk.md             — SHA256 sum concurrent (threadpool, num_cpus, walkdir, ring)
cryptography/encryption/pbkdf2.md                 — Salt and hash password (ring, rand, data_encoding, num)
cryptography/hashing/hmac.md                      — HMAC digest (ring, rand)
cryptography/hashing/sha-digest.md                — SHA-256 digest (ring, data_encoding, anyhow)
datetime/duration/checked.md                      — Checked datetime calc (chrono)
datetime/duration/timezone.md                     — Timezone conversion (chrono)
datetime/parse/current.md                         — Examine date and time (chrono)
datetime/parse/format.md                          — Format date and time (chrono)
datetime/parse/string.md                          — Parse string to DateTime (chrono)
datetime/parse/timestamp.md                       — Date to UNIX timestamp (chrono)
development_tools/debugging/config_log/log-custom.md      — Custom log location (log4rs, anyhow)
development_tools/debugging/config_log/log-env-variable.md — Custom env var for log level (env_logger)
development_tools/debugging/config_log/log-mod.md          — Log levels per module (env_logger)
development_tools/debugging/config_log/log-timestamp.md    — Timestamp in log messages (chrono, env_logger)
development_tools/debugging/log/log-debug.md               — Log debug message (env_logger)
development_tools/debugging/log/log-error.md               — Log error message (env_logger)
development_tools/debugging/log/log-stdout.md              — Log to stdout (env_logger)
development_tools/debugging/log/log-custom-logger.md       — Custom logger (log)
development_tools/debugging/log/log-syslog.md              — Unix syslog (syslog)
development_tools/debugging/tracing/tracing-console.md     — Tracing console (tracing)
development_tools/versioning/semver-command.md    — Check external command version (semver, anyhow, ring)
development_tools/versioning/semver-complex.md    — Parse complex version string (semver)
development_tools/versioning/semver-increment.md  — Parse and increment version (semver)
development_tools/versioning/semver-latest.md     — Find latest version (semver, anyhow)
development_tools/versioning/semver-prerelease.md — Check pre-release (semver)
encoding/complex/endian-byte.md                   — Little-endian byte order (byteorder)
encoding/complex/json.md                          — Serialize/deserialize JSON (serde)
encoding/complex/toml.md                          — Deserialize TOML (toml, serde)
encoding/csv/delimiter.md                         — CSV with different delimiters (csv, serde)
encoding/csv/filter.md                            — Filter CSV records (csv, anyhow)
encoding/csv/invalid.md                           — Handle invalid CSV (csv, serde)
encoding/csv/read.md                              — Read CSV records (csv)
encoding/csv/serde-serialize.md                   — Serialize CSV with Serde (csv, serde, anyhow)
encoding/csv/serialize.md                         — Serialize CSV records (csv, anyhow)
encoding/csv/transform.md                         — Transform CSV column (csv, serde, anyhow, ring)
encoding/string/base64.md                         — Base64 encode/decode (base64, anyhow)
encoding/string/hex.md                            — Hex encode/decode (data_encoding)
encoding/string/percent-encode.md                 — Percent-encode string (percent_encoding)
encoding/string/url-encode.md                     — URL encode (url)
errors/handle/backtrace.md                        — Error backtrace (anyhow, csv, serde)
errors/handle/retain.md                           — Avoid discarding errors (reqwest, thiserror, num)
file/dir/duplicate-name.md                        — Find duplicate files (walkdir, ring)
file/dir/find-file.md                             — Find files by predicate (walkdir, anyhow)
file/dir/ignore-case.md                           — Find files ignoring case (walkdir, anyhow, glob)
file/dir/loops.md                                 — Find path loops (walkdir)
file/dir/modified.md                              — Find modified files (walkdir, anyhow)
file/dir/png.md                                   — Find PNG files (glob, anyhow)
file/dir/recursive.md                             — Recursive directory traverse (walkdir)
file/dir/sizes.md                                 — Calculate file sizes (walkdir)
file/dir/skip-dot.md                              — Skip dotfiles (walkdir)
file/read-write/same-file.md                      — Avoid same file r/w (same_file)
file/read/read_lines.md                           — Read lines (tempfile)
hardware/processor/cpu-count.md                   — CPU core count (num_cpus)
os/external/piped.md                              — Piped commands (ring, anyhow)
os/external/process-output.md                     — Process stdout (ring, anyhow)
os/external/send-input.md                         — Send stdin to command (ring, anyhow)
safety_critical/heapless_alloc/heapless-alloc.md  — Deterministic memory (heapless)
science/mathematics/complex_numbers/add-complex.md       — Add complex numbers (num)
science/mathematics/complex_numbers/create-complex.md    — Create complex numbers (num)
science/mathematics/complex_numbers/mathematical-functions.md — Math functions (num)
science/mathematics/linear_algebra/add-matrices.md       — Add matrices (ndarray)
science/mathematics/linear_algebra/deserialize-matrix.md  — Serialize matrix (nalgebra, ndarray)
science/mathematics/linear_algebra/invert-matrix.md       — Invert matrix (nalgebra)
science/mathematics/linear_algebra/multiply-matrices.md   — Multiply matrices (ndarray)
science/mathematics/linear_algebra/multiply-scalar-vector-matrix.md — Scalar-vector-matrix (ndarray)
science/mathematics/linear_algebra/vector-comparison.md   — Vector comparison (ndarray)
science/mathematics/linear_algebra/vector-norm.md         — Vector norm (ndarray)
science/mathematics/miscellaneous/big-integers.md         — Big integers (num)
text/regex/email.md                               — Verify email (regex)
text/regex/filter-log.md                          — Filter log file (regex, anyhow)
text/regex/hashtags.md                            — Extract hashtags (regex)
text/regex/phone.md                               — Extract phone numbers (regex, anyhow)
text/regex/replace.md                             — Replace text pattern (regex)
text/string_parsing/from_str.md                   — Implement FromStr (num)
text/string_parsing/graphemes.md                  — Unicode graphemes (unicode_segmentation)
web/clients/api/rest-get.md                       — Query GitHub API (reqwest, serde)
web/clients/api/rest-head.md                      — Check API resource (reqwest)
web/clients/api/rest-post.md                      — Create/delete Gist (reqwest, serde, anyhow)
web/clients/authentication/basic.md               — Basic auth (reqwest)
web/clients/download/basic.md                     — Download file (reqwest, anyhow, tempfile)
web/clients/download/partial.md                   — Partial download (reqwest, anyhow)
web/clients/download/post-file.md                 — POST file (reqwest, anyhow, ring)
web/clients/requests/get.md                       — HTTP GET (reqwest, anyhow, ring)
web/clients/requests/header.md                    — Custom headers (reqwest, serde, url, anyhow)
web/mime/filename.md                              — MIME from filename (image)
web/mime/string.md                                — MIME from string (mime)
web/url/base.md                                   — Base URL (url, anyhow)
web/url/fragment.md                               — URL fragment (url)
web/url/new.md                                    — Create URLs (url)
web/url/origin.md                                 — URL origin (url)
web/url/parse.md                                  — Parse URL (url)
```

## 5. Tier C — 暂不可行（22 个）

涉及 async/await、tokio、网络编程、unsafe、数据库驱动等，a2r 当前不支持。

```
asynchronous/channel/bounded.md          — Bounded Channels (async, tokio)
asynchronous/channel/unbounded.md        — Unbounded Channels (async, tokio)
asynchronous/fs/create.md                — Async create files (async, tokio)
asynchronous/fs/read.md                  — Async read files (async, tokio)
asynchronous/fs/remove.md                — Async remove files (async, tokio)
asynchronous/fs/rw_traits.md             — AsyncRead/AsyncWrite (async, tokio)
asynchronous/fs/write.md                 — Async write files (async, tokio)
asynchronous/ftc/ctrl_c.md               — Ctrl+C handling (async, tokio)
asynchronous/join.md                     — Join async sets (async, tokio)
asynchronous/rt/tokio-rt-builder.md      — Tokio runtime builder (async, tokio)
asynchronous/rt/tokio-rt-macro.md        — Tokio macro (async, tokio)
asynchronous/timeout.md                  — Async timeout (async, tokio)
concurrency/actor/actor-pattern.md       — Actor pattern with Tokio (async, tokio)
concurrency/custom_future/custom-future.md — Custom Future impl (Pin, Waker)
database/postgres/aggregate_data.md      — Aggregate data (postgres)
database/postgres/create_tables.md       — Create tables (postgres)
database/postgres/insert_query_data.md   — Insert/query data (postgres)
database/sqlite/initialization.md        — SQLite init (rusqlite)
database/sqlite/insert_select.md         — SQLite insert/select (rusqlite)
database/sqlite/transactions.md          — SQLite transactions (rusqlite)
file/read-write/memmap.md                — Memory-mapped file (unsafe, memmap)
net/server/listen-unused.md              — TCP listen (TcpListener)
safety_critical/no_panic/no-panic.md     — No-panic guarantee (no-panic proc macro)
web/mime/request.md                      — MIME from HTTP (async, reqwest)
web/scraping/broken.md                   — Broken link check (async, reqwest)
web/scraping/extract-links.md            — Extract links (async, reqwest)
web/scraping/unique.md                   — Unique links (async, reqwest)
```

## 6. 执行步骤

### Phase 1: 基础设施搭建 ✅ 完成
1. 创建 `test/a2r/cookbook/` 目录结构
2. 在 `a2r_tests.rs` 中注册 15 个测试函数
3. 验证测试框架正确运行

### Phase 2: A 层测试用例实现 ✅ 完成
15 个测试用例全部创建完成，所有测试通过。

### Phase 3: 分析 a2r 问题 ⬅️ 当前阶段

## 7. a2r 问题分析（Phase 2 发现）

通过对比 `.expected.rs`（a2r 输出）和 `reference.rs`（Cookbook 原始 Rust），发现以下系统性问题：

### 7.1 高优先级（影响编译正确性）

| # | 问题 | 影响的测试 | 描述 |
|---|------|-----------|------|
| P1 | **数组类型推断为 `[T; N]` 而非 `Vec<T>`** | sort_int, sort_float, sort_struct, central_tendency, standard_deviation | Auto 的 `[1, 2, 3]` 被生成为 `let vec: [i32; 5] = vec![...]`，类型注解是数组但值是 Vec |
| P2 | **`fn() !` 返回 `Result<(), String>` 而非正确的错误类型** | read_lines, env_variable, process_continuous, error_file, boxed_error | `!` 标记的函数统一生成 `Result<(), String>`，但 `?` 操作符无法从 `io::Error` 等类型自动转为 `String` |
| P3 | **方法链被拆解为 `self.xxx()` 语句** | process_continuous, error_file | `Command.new("ls").arg("x").spawn()` 被拆为独立 `self.arg("x"); self.spawn();` 等，`self` 在 main 中不存在 |
| P4 | **缺少函数末尾 `Ok(())`** | read_lines, env_variable, error_file | 返回 `Result` 的函数缺少末尾 `Ok(())` |
| P5 | **`.to_f64()` 不存在于 `i32`/`usize`** | central_tendency, standard_deviation | `i32` 和 `usize` 没有 `.to_f64()` 方法，应使用 `as f64` |

### 7.2 中优先级（语义不等价但可编译）

| # | 问题 | 影响的测试 | 描述 |
|---|------|-----------|------|
| P6 | **type derive 缺少 `Ord`/`Eq`** | sort_struct | `type Person` 生成 `#[derive(Clone, Debug, PartialEq)]` 但排序需要 `Ord` |
| P7 | **`Duration` 使用 `{}` 格式化（Display）而非 `{:?}`（Debug）** | elapsed_time | `Duration` 不实现 `Display` |
| P8 | **`.cmp()` 参数缺少引用** | sort_struct | `b.age.cmp(a.age)` 应为 `b.age.cmp(&a.age)` |
| P9 | **操作符优先级错误** | latitude_longitude | `x / 2.0.sin()` 被解析为 `x / (2.0.sin())` 而非 `(x / 2.0).sin()` |
| P10 | **`use.rust std::error::Error` 不生成 `Box<dyn Error>`** | boxed_error | `!` 错误类型不使用 `Box<dyn Error>` |

### 7.3 低优先级（风格/非关键）

| # | 问题 | 影响的测试 | 描述 |
|---|------|-----------|------|
| P11 | **无参闭包 `=> expr` 解析失败** | lazy_cell | `get_or_init(=> ...)` 语法不支持，需改为 `get_or_init(() => ...)` |
| P12 | **`println!("{}", vec)` Vec 无 Display** | sort_int, sort_float, sort_struct | 应使用 `{:?}` 或 `vec!` 宏 |
| P13 | **`env.get_or()` 生成 `.ok().unwrap_or()` 而非 `.unwrap_or()`** | env_variable | 多了一步不必要的 `.ok()` |

### 7.4 测试结果统计

| 指标 | 数量 |
|------|------|
| 总测试 | 15 |
| a2r 能编译通过 | 3 (side_length, tan_sin_cos, elapsed_time) |
| 语义完全正确 | 1 (side_length) |
| 编译失败 | 11 |
| 解析失败 | 1 (lazy_cell, 已修复后成功) |

## 8. 统计

| 层次 | 数量 | 状态 |
|------|------|------|
| Tier A | 15 | ✅ 已实现 |
| Tier B | 107 | 待 A 层问题修复后评估 |
| Tier C | 27 | 暂不处理 |
| **总计** | **149** | |

## 9. 下一步

1. **修复 P1（数组类型推断）** — 影响面最广，5 个测试受影响
2. **修复 P2（错误类型映射）** — `fn() !` 应根据 `use.rust` 导入推断正确的错误类型
3. **修复 P3（方法链）** — Builder pattern 是 Rust 的核心惯用法
4. **修复 P4（缺少 Ok(())）** — 简单修复
5. **修复 P5（to_f64 映射）** — 应映射为 `as f64`

修复后重新运行 cookbook 测试，验证 `.expected.rs` 是否改善，然后将改善后的输出设为新的基准。
