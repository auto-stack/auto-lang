# Plan 271: 移除 a2r 预期文件的 example 声明

**Status**: Completed
**Date**: 2026-05-28

## Context

`Cargo.toml` 中有 55 个 `[[example]]` 指向 `test/a2r/*/*/.expected.rs`，目的是验证生成的 Rust 能编译。但每个 example 独立链接 `auto_lang` crate（~39MB），55 个 example 导致链接阶段内存不足、编译极慢。

## Changes

1. **Cargo.toml**：删除 55 个 a2r `[[example]]` 声明（457→232 行）
2. **examples/**：删除 8 个不再使用的 example 文件
   - a2r_bootstrap.rs, a2r_step00.rs, a2r_test.rs
   - bootstrap_phase3.rs, feature_parity.rs, perf_benchmark.rs
   - test_a2r_lexer.rs, test_widget_macro.rs
3. **Expected files**：更新 test/a2r/ 和 test/cookbook/ 中所有 .expected.rs，与 Plan 270 条件化 a2r_std 导入对齐
4. **rust.rs**：删除未使用的 `emit_a2r_stdlib_import()` 方法

## Verification

- 271/271 a2r tests pass
- 2923 total tests pass (17 pre-existing VM/UI failures unrelated)
