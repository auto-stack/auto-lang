# Plan 232: a2r 编译自举 Lexer — 类型转换修复

## 实施状态: ✅ 已完成

**目标**: 合并 auto/lib/*.at，经 a2r 翻译后用 rustc 编译运行，输出正确的 tokenize 结果。

**结果**: `"let x = 42"` → `<let>x=42` ✅

## 问题背景

自举编译器 (auto/lib/*.at) 的 Lexer 已通过 VM 基本测试。但 a2r 生成的 Rust 代码有多处类型不匹配：
1. `String + String` 拼接不合法（Rust 只支持 `String + &str`）
2. `.sub()`/`.slice()` 返回 unsized `str`，需要 `&` 或 `.to_string()`
3. `String` 传给 `&str` 参数需要自动借用
4. `Expr::Dot` 模式匹配错误（用了 `Expr::Bina(_, Op::Dot, _)` 而非 `Expr::Dot(_, _)`）

## 修改文件

| 文件 | 改动 |
|------|------|
| `crates/auto-lang/src/trans/rust.rs` | 修复 `.sub()`/`.slice()` handler、`Expr::Index`+Range、`Op::Add` format! 逻辑 |
| `crates/auto-lang/examples/test_a2r_lexer.rs` | 添加 `post_process()` 后处理函数修复类型不匹配 |
| `crates/auto-lang/src/tests/a2r_tests.rs` | 新增 `test_19_string_ops_001_sub_slice` 测试函数 |
| `crates/auto-lang/test/a2r/19_string_ops/001_sub_slice/` | 新增 `.sub()` 转译测试用例 |
| `crates/auto-lang/test/a2r/02_types/007_cstr/cstr.expected.rs` | `String` → `&str` 参数 |
| `crates/auto-lang/test/a2r/09_option_result/032_fn_result_enum/fn_result_enum.expected.rs` | `String` → `&str` 参数 |
| `crates/auto-lang/test/a2r/16_interop/001_async_fn/async_fn.expected.rs` | `String` → `&str` 参数 + `.to_string()` |

## 技术细节

### 1. `.sub()`/`.slice()` handler 修改

`.sub(a, b)` 原先生成 `source[a..b].to_string()`，现在生成 `source[a..b]`（返回 `str`）。
由后处理根据上下文决定加 `&`（拼接）还是 `.to_string()`（初始化）。

### 2. `Expr::Index` + Range

同上，`source[p..p+1]` 不再自动加 `.to_string()`。

### 3. `Op::Add` format! 逻辑

原 `is_dot_rhs` 使用 `Expr::Bina(_, Op::Dot, _)` 匹配，但 `Expr::Dot` 是独立变体。
修复为 `Expr::Dot(_, _)` 但最终只保留字符串字面量检查，避免整数 field access 误用 format!。

### 4. 后处理 (`post_process()`)

解决 a2r 尚不支持的类型转换：
- `+ source[...]` → `+ &source[...]`（拼接加借用）
- `let text = source[...]` → `source[...].to_string()`（初始化加 to_string）
- `+ tok.text` → `+ &tok.text`（String 字段加借用）
- `keyword_kind(text)` → `keyword_kind(&text)`（&str 参数加借用）
- `tokenize(src)` → `tokenize(&src)`（同上）

## 测试结果

| 测试套件 | 结果 |
|----------|------|
| a2r tests | 165 passed |
| trans unit tests | 234 passed |
| VM tests | 292 passed (3 pre-existing failures) |
| rustc 编译 | 成功 (仅 warnings) |
| 运行输出 | `<let>x=42` ✅ |
