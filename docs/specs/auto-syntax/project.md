# auto-syntax — Atom 语法驱动的增量词法与无损 CST

> PLAN-896当前组织：上述ACE相关实现统一属于`experimental/ace/Cargo.toml`独立workspace（单Cargo.lock/同树target），各子目录为成员包，不再是独立workspace。ace-syntax-runtime、ace-syntax-grammar、ace-syntax-core与ace-syntax-artifacts位于crates/syntax-*；语法资产位于grammars。普通语言cargo t不构建ACE；cargo t ace选择常规集合，完整包/all及ace-performance保原专项和适用条件。

> **Status**: experimental（PLAN-787/828已交付有限语法机制；scalar常驻AV暖编辑已有真实消费，尚未统一所有源码入口）
> 路径：`experimental/ace/`（独立 Cargo workspace：runtime/grammar/双 adapter）
> 来源：PLAN-787（[归档计划](../../plans/archive/787-ac-grammar-incremental-cst.md)）；
> 关联 [grammar-contract.md](grammar-contract.md)、[snapshot-cst.md](snapshot-cst.md)，
> A01/A02/A03 产物接入见 [../auto-ac/artifact-contract.md](../auto-ac/artifact-contract.md)。

## 定位与依赖方向

有限 profile（auto-core / PLAN-828 auto-scalar / interface-mini-v1声明描述小语言）
语法服务。依赖方向单向：snapshot→grammar-program/scanner→CST→结构查询；
运行时不依赖 ac-core/Cranelift/auto-cache/Auto 语义环境；Atom 配置为唯一规则源
（经 auto-atom 静态读取编译，无宿主代码执行）。

## 实验边界（不冒称的部分）

- 非完整 Auto 版 tree-sitter、非任意 CFG/GLR/regex、非动态 scanner 插件、
  非 LSP 服务、非正式 Batom 格式，也未全面替换所有源码parser入口。PLAN-787的
  “投影仅对照”是早期边界；PLAN-828 scalar常驻AV暖编辑已调用
  `incremental_reparse` + `project_decl_strict`，见[snapshot-cst.md](snapshot-cst.md)
  与[函数增量源前端](../auto-ac/function-incremental-source.md)。
- 规则程序限定 ordered-choice + Pratt（编译期拒绝左递归/可空 repeat/未知
  引用/非前进 scanner，定位到节点路径）；其它 profile 的合法语法不得称为
  Auto 错误。
- 布局算法演进时必须为 A03 键引入布局版本分量隔离新旧页（现行键形历史稳定）。

## 包结构

| 模块 | 内容 |
|---|---|
| runtime | snapshot/position/handle/lexer/parse/cst/recovery/edit/outline（词法-解析-CST-增量运行时） |
| grammar | .at 配置→GrammarProgram 编译器（grammars/{auto-core,auto-scalar,interface-mini-v1}.at） |
| adapters/core | CST→现有Program/单声明严格投影；对照测试及scalar常驻会话增量消费，尚未全面替换整模块/其他profile入口 |
| adapters/artifacts | 783 store 真实产物生产者（A01/A02/A03 + 流式构建器 + 查询 bin） |

## 验证

`scripts/verify-ac-syntax-787.ps1 -Scope acceptance`（16 族矩阵，含 64MiB
work-owned 预算档）；矩阵与性能读数 `docs/reports/plan787/matrix.md`。

## 相关 plan

2026-11-03用户已将**统一源码语法前端纳入M1出口**，目标和SYN-01..06、ARCH门见
[架构机制完整性裁定](../../plans/reports/ace-m1-architecture-completeness-decision-2026-11-03.md)。
这是已批准目标，不是已实现状态；后继尚未取号。须覆盖M1承诺源码profile及冷编译、
暖编辑、恢复和适用AC/AV/A2R/A2C入口，不得以永久手写parser兜底或删减支持面达成。

见 [plans.md](plans.md)。
