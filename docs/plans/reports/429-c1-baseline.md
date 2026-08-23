# plan-429 C1: 自举基线 hash 表

- 日期：2026-08-23
- **状态：临时基线**。v0.5 git tag 尚未打（release note `docs/releases/v0.5.md` 已于
  2026-08-12 写好，但 tag 停在 v0.4.1）。下表锚定 plan-429 分支基点 **master b3bd64f5**
  作为临时基线；**v0.5 tag 落地后由 plan-431 重锚定**（若 tag 点早于 b3bd64f5，以 tag 为准
  重跑本表命令）。
- 重生成命令（在仓库根）：
  `for f in <文件列表>; do git log -1 --format=%h -- crates/auto-lang/$f; done`

## 核心自举文件基线（截至 b3bd64f5）

| 文件（crates/auto-lang/ 相对） | 最后变更 commit | 行数 |
|---|---|---|
| src/token.rs | 22372374 | 516 |
| src/lexer.rs | 1fb96eac | 2,029 |
| src/error.rs | 4c4d6db1 | 1,299 |
| src/parser.rs | 4c4d6db1 | 17,708 |
| src/types.rs | e01f0f84 | 722 |
| src/ast.rs | 0eec445b | 1,734 |
| src/infer/context.rs | e01f0f84 | 613 |
| src/infer/expr.rs | d34a83dd | 1,109 |
| src/infer/stmt.rs | 0eec445b | 893 |
| src/infer/functions.rs | b4a36fe4 | 799 |
| src/infer/unification.rs | 92ea9c85 | 647 |
| src/vm/opcode.rs | d8c10d8e | 925 |
| src/vm/codegen.rs | 8c078543 | 13,195 |
| src/vm/engine.rs | e29c3b92 | 8,354 |
| src/vm/native_catalog.rs | 8c078543 | 2,319 |
| src/vm/ffi/stdlib.rs | a76e9cbe | 8,546 |
| src/trans/rust.rs | e29c3b92 | 21,065 |

核心子集合计约 **62,000 行**（未剔 UI 段；431 定稿边界后重统计）。

## 待办移交

1. v0.5 tag 打完后：`git log -1 --format=%h v0.5 -- <file>` 重跑本表，写入正式基线列；
2. `docs/guides/aavm-sync-guide.md` 的注记更新移交给 plan-431（与 v2 目录方案一起做，避免改两次）。
