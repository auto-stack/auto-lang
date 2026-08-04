# Plan 013 已迁移至 auto-ai 仓库

> **2026-07-29**：Plan 013（auto-ai → Auto 语言移植）本质上是 **auto-ai 项目**
> 的计划，此前误放在 auto-lang。现已迁移到它应在的位置：
>
> 👉 **`auto-ai` 仓库 `docs/plans/013-auto-ai-port-to-auto.md`**
> （以及配套的 `013-handoff-for-new-session.md`）
>
> 本文件在此仅作重定向占位，不再维护。Auto 版 ReAct MVP 的最新进度与剩余
> 差距请读迁移后的 `013-auto-ai-port-to-auto.md` 文末「★ MVP 里程碑与剩余差距」。

---

## 为什么原来在 auto-lang？

移植工作（`.at` 源码 + a2r codegen 修复）的产物落地在 `auto-lang/crates/`
（`auto-ai-agent/`、`auto-ai-client/`、`ai-config` 的 Auto 版），且 a2r 生成器
修复在 `auto-lang/crates/auto-lang/src/trans/rust.rs`，所以早期会话把计划随手
放在了 auto-lang。但计划本身追踪的是"把 auto-ai 的架构用 Auto 复刻"这一
**auto-ai 侧的目标**，归 auto-ai 的 plans 目录更合理。

## 相关的 a2r codegen 计划（留在 auto-lang，因为它们是 a2r 生成器的改动）

这些是 a2r 生成器的修复计划，**仍属 auto-lang**，不随 plan 013 迁移：

- `docs/plans/372-a2r-rust-correctness-fixes.md` — 3 个系统性 a2r 根因
- `docs/plans/373-a2r-b1-papercuts.md` — B1 类 codegen 细节（plan 013 的 B1 解锁）
