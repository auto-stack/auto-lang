# Known Divergences

This file records all accepted and open divergences between AutoVM, a2r, and
native Rust for replicated libraries.

## Format

Each entry has:
- **DIV-NNNN**: unique ID
- **库**: library name
- **用例**: test case name
- **AutoVM 行为**: what AutoVM produces
- **a2r 行为**: what a2r transpiled Rust produces
- **Rust 原生行为**: what native Rust produces
- **偏差类型**: 可接受 / 待修复 / 已修复
- **状态**: accepted / open / fixed
- **原因**: explanation

---

(No divergences yet — _dummy is fully consistent across all three backends.)
