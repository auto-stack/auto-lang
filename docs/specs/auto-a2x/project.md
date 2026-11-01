# auto-a2x

> PLAN-896当前组织：上述ACE相关实现统一属于`experimental/ace/Cargo.toml`独立workspace（单Cargo.lock/同树target），各子目录为成员包，不再是独立workspace。ace-a2x-core、ace-a2x-rust、ace-a2x-store、ace-a2x-worker位于crates/a2x-*，原Checked HIR→Rust依赖方向保持。普通语言cargo t不构建ACE；cargo t ace选择常规集合，完整包/all及ace-performance保原专项和适用条件。

> **Status**: experimental
> 路径：`experimental/ace`（独立 Cargo workspace：`a2x-core` / `a2x-rust` / `a2x-store` / `a2x-probe`） | 依据：Design 51（AX-A，PLAN-892 交付）

共享 A2X 骨架：已验证 core/scalar Checked HIR 之上的**目标中立语义视图**、
结构化准备摘要与分层 cache 会话；Rust 是首个真实目标 provider（见
[auto-a2r](../auto-a2r/project.md)）。第二目标未实现、明确不宣称多目标共享已实证。

## 目标与范围

- **语义视图**（`a2x-core::SemanticView`）：只读消费 `CheckedModule`/单函数
  `CheckedFunction` 凭证，五域准入（core/scalar＋AX-B 的
  record/text/text-record；其余域具名拒绝 `a2x.domain`），expr/stmt/place/
  type 具名拒绝防御（前端之外的二道门）；函数源名身份（自由函数剥
  `sf_`、方法 owner 限定剥 `sm_`）、显式入口选择（零参 i32 自由函数）；
  trace 授权按 capability＋绑定 DefId。AX-B 事实面（模式/名义 record/
  方法 owner/资源计划出口）见
  [checked-resource-facts](checked-resource-facts.md)。
- **准备摘要**（`prepare.rs`）：`FunctionPlan`（ops/calls/marks/计数/迭代式
  max-depth，冻结 `MAX_STRUCTURE_DEPTH=400` → `a2x.too-deep`）与**语义指纹**
  （首次使用序局部规范化、参数按位 0..n-1、span/名字/表序排除——空白等价源
  指纹不变）。
- **会话/cache**（`a2x-store`，见 [incremental-and-cache](incremental-and-cache.md)）。
- **禁止依赖**（boundaries 测试机械断言，V-13）：不依赖 auto-mir-lower、AV
  执行器、Cranelift、旧 AST/Database 前端、旧 Trans 栈；不进根产品 workspace。

## 现状边界

- AX-A 已交付（PLAN-892）：core/scalar 计算闭包全量冻结语料 58 用例真链
  （真实 rustc 编译运行对独立 oracle）；暖稳态管理工作集实测 29.7MiB ≤48MiB
  @load256 规模（冷整源峰值 ~338MB 为上游 ac-core 前端特征，单列报告非 A2X 预算面）。
- AX-B 已交付（PLAN-902）：record/text/text-record 三域真链（四冻结源
  42/42/23/42 真 rustc 认证；资源清理按验证器计划事件序钉住；AXS2/AXR2
  层真函数级增量，formal 窗口 hit/restart/body-edit P90=6/21/40ms）。
- List/Variant（AX-C）、闭包/HOF、mut 实例方法、Unit 入口、宽源码、
  泛型/meta、多模块/全 stdlib、public ABI、native loader/HMR、第二目标、
  旧 A2X 切换均不在现状。

## PLAN-904（AX-C）扩展

三通道并列：scalar(892) / record-text(902) / **collections(904)**——recipe
`ac-a2r/hir-collections-emit-v1`、fileset/cert/store 层 AXS3/AXR3 严格互拒；typed
List/Variant 通道合同见 [collections-hir-backend](../auto-a2r/collections-hir-backend.md)。
本仓仍不含 M1 全量完成或全部 A2X 面。
