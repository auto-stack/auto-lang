# AIE Database（增量编译存储）

## 范围

`database/`：Auto Incremental Engine 的中心存储，替代旧 `Rc<RefCell<Universe>>`，
为查询式增量编译系统提供"单一事实源"（database/mod.rs 头注）。

## 原则

- 两层结构：LAYER 1 存储（源码、AST 片段、符号——由 Indexer 写入）；
  LAYER 2 缓存（类型、字节码、依赖——由 Query Engine 计算）。
- 标识符跨编译稳定：文件级 `FileId`、声明级 `FragId`。
- 片段（fragment）是增量编译的最小单元。

## 细节

### 数据结构（database/mod.rs）

- `FileId(u64)`：文件级稳定标识（mod.rs:41）。
- `FragId { file: FileId, offset: usize, generation: u32 }`（mod.rs:58）：
  每个顶层声明（函数/结构体/常量）一个片段；`offset` 是源文件字节偏移，
  `generation` 随修改递增（`next_generation`）。
- `FragMeta`/`FragSpan`/`FragKind`（mod.rs:94/103/112）：片段元数据与种类。
- `Artifact`/`ArtifactType`（mod.rs:127/141）：编译产物记录。
- `DependencyGraph`（mod.rs:158）：依赖图。
- `Database`（mod.rs:251）：中心结构，内部用 `DashMap` 支持并发访问。
- `QueryEngine` 在 `query.rs:157`（属查询层，不在本目录）。

### UI 产物缓存（database/ui_artifact.rs、ui_cache.rs）

- `UIArtifact`/`UIBackend`：UI 代码生成产物；`UICache`：持久化增量缓存，
  使 UI 生成只重跑变更的 `.at` 文件（plan-135 的 AIE 复用思路：文件哈希 + 脏跟踪）。
- 注意：database/mod.rs 头注写 "Plan 134: UI Artifact support"，实际来源是
  plan-135（`docs/plans/old/135-ui-incremental-compilation.md` 明确以 AIE Database 做
  UI 增量）；134 是 jet-generator-view-body，无此内容——注释编号有误。

### 与运行时分层的关系

`Database` 持有编译期 `SymbolTable`（见 design/compile-runtime-split.md 的架构图）；
运行期 `ExecutionEngine` 只以 `scope_sid` 反向链接，不向 Database 写入运行态。

## 显式非目标

- 不实现查询引擎本身（`query.rs` 属另一模块）。
- Phase 3 的片段级哈希与细粒度依赖在头注中列为阶段目标，当前粒度以文件级为主
  （头注 Phase 1-3 自述；未逐行验证完成度）。
- 不做跨进程持久化格式规范（UICache 的序列化格式是内部实现细节）。

> 来源: crates/auto-lang/src/database/mod.rs、ui_artifact.rs、ui_cache.rs；docs/plans/old/064-split-universe-compile-runtime.md、135-ui-incremental-compilation.md
