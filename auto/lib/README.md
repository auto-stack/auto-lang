# AAVM v2(纯 Rust 模式)

Plan 432 起按 `docs/specs/aavm/` 规范逐模块移植写入本目录。
旧 v1 已归档至 `auto/lib-legacy/`。

Plan 434 增 `a2r.at`(AA2R:Auto 版 a2r 核心子集)——终极自举闭环:
Auto 写的 a2r 转译含自身的七文件 lib,产物为可独立 cargo build 的纯 Rust
(零 a2r_std),该 VM 运行 corpus_m4 30/30 与参考一致;五方对比矩阵见
`docs/specs/aavm/design/matrix-434.html`。

每个 `.at` 文件头使用 Snapshot 模板(见 docs/specs/aavm/design/divergence-rules.md §5)。
