# Plan 415: a2r 剩余大件拆粒度实施（242 tracker 收尾批）

> **状态**: 📋 已立项待实施（2026-08-22,源自审计 §5.2 B5 拆粒度）
> **来源**: Plan 242（a2r 功能差距 tracker,持续维护不归档）剩余未做项;审计判定"均为大件,独立立项"
> **前置核查**: #8 闭包推断根因已由 audit-A6 修复（`c2bd1d0c`,golden 004_closure_infer）,不在本计划范围

---

## 0. 拆解原则

242 剩余项彼此独立、体量差异大,按"可独立验收的最小粒度"拆为 5 个子项,
每项独立 worktree + 分支(`plan-fix/415-<id>`),全绿才合并。跨仓验证环
(a2r 改动必做:worktree 构建 → auto-ai 三 retranspile + cargo check 零错)
对 #2/#10 强制,#15/#16/#17 视触及面。

## 1. 子项清单

### 415-A `HashMap::from` 字面量发射（242 #2,预估 1-2 天）

- **现状**: `let m Map<str,int> = {"a": 1}` 发射 struct/object 语法而非惯用
  `HashMap::from([...])`（242 §行 59-63 已给出期望形态）。
- **入口**: `crates/auto-lang/src/trans/rust.rs` map 字面量发射臂 +
  a2r golden 新用例。
- **验收**: golden 双向通过 + auto-ai 重生成零 diff。

### 415-B Redis/SQLite a2rs backend stdlib（242 #10,预估 3-5 天）

- **现状**: Plan 121 交接的 6 个 cookbook DB stub + Plan 240 Phase 10 交接
  4 stub,均为 VM 侧占位;a2r 路径无对应发射。
- **拆粒度**: B1 先做 SQLite(rusqlite 已在依赖树,auto-man 已用)——
  a2r 侧 stdlib 声明 + 发射映射;B2 再做 Redis(需引入 redis crate,
  涉及 build 脚本与平台验证,单独立分支)。
- **入口**: `stdlib/auto/` 新增 `sqlite.rs.at`/`redis.rs.at` +
  `crates/a2r-std/` 对应手抄副本(注意 KNOWN-DEBT 396 条目:手抄漂移风险,
  本项落地时应顺带建立签名比对)。

### 415-C GPUI a2r UI generator（242 #15,预估 ≥1 周,⭐⭐⭐⭐⭐）

- **现状**: `ui/gpui/` 已有 renderer 骨架(Plan 365 的 Image 占位/Grid
  分解已知限制),但 a2r(ui 生成器侧)完全未接。
- **前置决策**: GPUI 依赖重(wgpu 全家桶已在 iced 路径),先决条件是
  Plan 386 的启动条件式评估("≥3 个 COSMIC app 跑通")是否放宽;
  建议先出 1 天 spike(AURA → GPUI 映射层 PoC)再定 go/no-go。

### 415-D 自举 Phase 2/E（242 #16,预估 ≥1 周）

- **现状**: Plan 355 完成 a2r 发射侧(#12),自举(用 Auto 写 Auto 工具链)
  Phase 2(编译器自身)与 Phase E 待做。
- **依赖**: 415-A/B 落地后再评估;自举对 stdlib 覆盖面敏感。

### 415-E dep cc + memmap2 FFI（242 #17,预估 2-3 天）

- **现状**: Plan 240 Phase 13 交接 4 个 cookbook stub。
- **入口**: build-time codegen(`build.rs` + cc 编译 C 桥)+ memmap2
  FFI 声明;Windows/MSVC 工具链验证是主要风险点。

## 2. 执行顺序建议

A（最小、独立）→ E → B1 → B2;C/D 各自 spike 后重估。每项合并后回填
242 tracker 对应行 + 本文档勾选。

## 3. 验证矩阵

| 子项 | 单测/golden | 跨仓环 | 实机 |
|---|---|---|---|
| A | golden 新用例 ×2 | 必须 | — |
| B1/B2 | a2r-std 契约测试 | 必须 | cookbook demo |
| C | 映射层单测 | 视触及 | GPUI 窗口冒烟 |
| D | 自举产物 diff | 必须 | `auto build` 自举 |
| E | FFI 冒烟 ×4 | 视触及 | Windows 构建 |
