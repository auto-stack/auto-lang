# ACE 源码可重放验收（source-only replay）

> 状态：active（PLAN-933 交付；关闭 D-925-1 的 ignored-附件依赖，并给
> `a2r-hir-collections-904` 金样 manifest 独立 oracle 与干净检出重放合同）。
> 原则：**任何日常验收不得依赖 git 忽略文件或主检出磁盘残留**——新鲜
> 检出/worktree 上，跟踪输入闭包必须自足。

## 跟踪权威（不再读 ignored 附件）

- sema 冻结身份/版本/描述面（PLAN-796 D2–D8）的唯一跟踪权威 =
  [auto-semantics/semantic-contract.md](../auto-semantics/semantic-contract.md)
  的"冻结身份/版本/描述面权威"节（SD-01 转写；原 contract-decision.md 按
  2026-10-31 证据政策不入库，其 sha256 在转写节内留档）。两例边界测试
  （`semantics-store/tests/cases/sema_boundary_audit.rs`）读该 spec，断言
  强度与原 ignored-附件版逐项相同；突变负控（改 producer/schema/algo/layer、
  删描述 token、近邻值）必须红。**不得**把 `expect present` 弱化为
  `if exists`、ignore 或删除负控。
- 测试新增的读入依赖必须是跟踪文件；发现测试读 ignored 路径按同类缺件
  （D-925-1 形态）登记处理，不以主检出残留充绿。

## 904 干净检出重放合同

- `crates/auto-lang/test/cases/a2r-hir-collections-904/`：20 源 + 20×4 金样
  （含 manifest.json，D-931-3 force-add）全部入库；日常固定组
  `case-a2r-hir-collections-904`（真实预构建 worker+父门逐字节）在新鲜树上
  必须可重放。manifest 的独立可审计性由
  [a2r-hir-certification.md](a2r-hir-certification.md) §904 golden manifest
  独立 oracle 承担。
- 负控分级：sema 权威突变与 oracle 错推导拒绝是**持久日常测试**；父门
  篡改（错 hash/config/target/缺件/多件/跨例近邻）为验证腿证据（真实
  auto-case 父门对 scratch 套件逐例红、输入根只读、原 20 例发现/执行
  守恒），路径域/重复路径等合同面由 auto-test-case 既有单测持久覆盖。
- 重放不含的腿如实分列：恢复→立即编辑/恢复→坏编辑→修复（SYN 首编辑）
  归 PLAN-929（D-931-1，接缝未释放前不充 PASS）；显式 rustc 认证腿与日常
  emit 金样分轴不变。

## 计划证据与清理

- 计划相关材料唯一暂存区 `D:/autostack/plan-artifacts/plan-<NNN>/`；
  附件目录（`docs/plans/attachments/`）保持整目录不入库，不因任何验收
  依赖而恢复。

## M1 残余闭包跟踪 fixture（PLAN-999，SD-06）

`crates/auto-lang/test/cases/ace-m1-residual-999/`：五源字节级冻结（SHA256 见
PLAN-999 §5.1，只读冻结自 941 scalar-q4 测试输入，不复制其在途生产实现）、
10 正例 checks（八冻结格 R-01..08 + 两 native 正对照，recipe/oracle 与原
`ace.m1.scalarq4.*` 一对一映射）与 2 端到端负控（真语法错误在选择门前保持
parse 判决；超 512 构造深为 `mir.lower.depth` 资源态）。固定组
`case-ace-m1-residual-999`（test-groups.toml）为唯一正式入口；fixture 仅引用
跟踪 `src/` 文件，新鲜检出/worktree 自足，零依赖 941 磁盘残留或 ignored 文件。
"本包 8/8"与"941 原八格接纳转绿"分别报告；最终组合判定归 941 原 owner 或具名
后继验收责任，本包不设置 `can_claim_expanded_m1`。
