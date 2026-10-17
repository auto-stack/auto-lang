# PLAN-806 独立最终复审 R1：送审基线缺失

2026-10-17（Asia/Shanghai） / stage: review / review_scope: final / outcome: blocked。

本会话为用户另行请求的检查会话，未参与 806 实施，未创建其他 agent。
可观察身份为当前 Codex 会话；不臆造模型或会话 ID。本轮是送审就绪核查，
不是对不存在的实施提交完成了全量验收。没有修改 Rust、构建或测试，也没有合入。

## 受检基线

- 主检出：`D:/autostack/auto-lang`，branch `master`，HEAD
  `2b57be3a6838a63ba7d175d7693b7aaa65c2edc7`。
- Plan：revision 1，检查前 `status: drafting`、`current_step: 0`，T-01～08 全未勾选；
  原始 Plan blob：`8110a31fa6f232015a0dc257511c28ccb32cda04`。
- 计划指定实施路径：`D:/autostack/.wt/lang-806/auto-lang`，branch
  `codex/plan-806-dev`；目录不存在，`git worktree list --porcelain` 无对应登记，
  `git branch --all --list '*806*'` 无输出。未创建替代审查树。
- reviewed_commit: **无**；base_commit: **无实施差异基线**。
  上述 master HEAD 仅是现状核查基线，不冒充 806 实施提交。
- `git log --all --format='%H %s' --regexp-ignore-case
  --grep='806\|ui-declarations\|auto-ui-dialect' -20` 中，与本计划直接相关的最新记录
  为 `8f4a757088e235fbfb11007d99b0150e54211bff`：只读预检，明确零代码、未建树。
- 本轮不干预主检出其他计划已有脏文档/附件；检查前 806 Plan 和附件无脏项。

## 前置核查

799 已归档，主检出 `crates/auto-ast` 在位；801 也已归档，实际
`crates/auto-parser-api/Cargo.toml` 在位，801 归档状态 8/8。
801 代码历史可追溯至合入后的 `b95e51d064ce10d58a62bf4e9560c2b72f2c67a5`，
沉淀准备 `786c499643f24b7c10801220038de1740c602021`，合入归档收据
`f6b62f3017b83c1579dee8ff4997e22a3e9df711`，清理终回执即受检 master HEAD。
因此历史预检中的“801 未合入”**已经解除**，不能继续引用为当前阻塞。
实际阻塞是找不到 806 的实施位置、提交及验收交接。

## F806-R1-01（P1）：没有可绑定的实施基线

类型：送审交接/就绪缺口；影响 AC-01～05、T-01～07。

当前可访问仓库缺少 `crates/auto-ui-dialect` 和
`test/fixtures/modularity/ui-declarations-consumer`。主检出仍保留旧实现：

- `crates/auto-lang/src/parser.rs:14917`：`parse_msg_decl_inner` 完整算法，
  包含 legacy 名字丢弃、quoted variants、payload 及 lexer lookahead。
- `parser.rs:15010`：`parse_model_block_inner` 完整字段算法；
  `parser.rs:15066`：`parse_model_field_annotations` 完整注解算法。
- widget：`parser.rs:13259/13262`；store：`14532/14535`；
  component：`16002/16006`，均直连上述旧 inner。
- `crates/auto-lang/src/dialect/ui.rs:49/50` 顶层分支仍调用 Parser 旧入口。
- `test/fixtures/modularity/dialect-api-consumer/src/lib.rs:92` 仍有独立
  `parse_model_subset`，并未迁成 806 所要求的同源策略。
- `docs/plans/attachments/806` 检查前仅有 `t01-readonly-prep.md`，没有
  work 完成收据、实现 SHA、V01～06 实际结果或 Spec 备稿。

这些证明本地可见成果尚不足以审查“实施完成”，不证明另一未提供位置也没有实现。
不能把 master 的旧实现当作一个新提交的回归，也不能仅把 frontmatter 改为
execution_done 就授予通过。

处理：定位原实施树及已提交代码，核对仓库/分支/依赖/cleanliness，并补充
T-01～06、V01～06 与 SD 交接；若确未实施，则先 work 执行原合同。
不需要改变合同，也不批准删减验收；不要新建 reviewer 专用树。

## AC → task → 实物 → 证据

| AC | tasks | 本轮结果 | 实物/核查方式与边界 |
|---|---|---|---|
| AC-01 | T-01/02/05 | fail（可见现状） | 无轻方言 crate/consumer，旧算法仍在 Parser；没有实施提交可验独立闭包 |
| AC-02 | T-01/02/04/05 | partial | 源码确认旧 msg/model/注解与 pilot 仍在；没有迁出后的行为对拍，V02/03 未验证 |
| AC-03 | T-01/03/04/05 | partial | 核对顶层与三类结构实际调用仍为旧实现；新实现的 import/消费/恢复/注册/无 UI-feature 矩阵均未验证 |
| AC-04 | T-04/05/06 | fail（可见现状） | 旧算法未删，缺单源/792/777/SD 完成证据；不运行不存在交付物的 driver |
| AC-05 | T-07 | fail（就绪） | 缺 reviewed_commit 与实施树，不能完成 revision-bound 最终审查 |
| AC-06 | T-08 | pending（merge-owned） | 未获 final pass；冻结裸 t+tv、实际触面档、沉淀/ledger/归档/guard 均未执行 |

源检查命令：`rg -n 'parse_msg_decl_inner|parse_model_block_inner|parse_model_field_annotations|parse_model_subset|ExternalModelDialect' crates/auto-lang/src/parser.rs crates/auto-lang/src/dialect crates/auto-parser-api test/fixtures/modularity/dialect-api-consumer`。
配合读取完整三算法、UiDialect、worktree/ref/目录清单。无 Rust 测试、无整档、无 docs_gen；
必要运行验收均保留未验证，不能拿其他计划的绿灯替代。

## Spec delta

冻结合同中的 SD-01～04 见 [review-r1-spec-delta.md](review-r1-spec-delta.md)。
该副本 Git blob：`266757a656f305a681cef96dff3e08d030a8081c`。
对照源以 Git blob（内容标识，不冒称 SHA-256）冻结：

| Spec input | Git blob |
|---|---|
| docs/specs/auto-lang/frontend/overview.md | 75b52a445e99a66878f4abb29ef46cca557c72e0 |
| docs/specs/auto-lang/frontend/design/dialect-system.md | 461850beb2ee007c9b52e52f596705101e1778c5 |
| docs/specs/auto-lang/ui/design/scenario-dialect.md | 0d04e4dd6e76c759e89fb1af9b2310dfb1fe156c |
| docs/specs/auto-parser-api/project.md | cd8be9d54e136784202b51bea132fdfa14e53e1d |

现有 impact 路径与目标 GOAL-003/007 保留；这些是拟变更，尚无代码/备稿支撑其最终正确性。
未写 canonical Spec 或 ledger，未增加/放宽合同。

## 路由

Plan 置 `executing`、0/8，所有未完成任务保持未勾选；保留历史预检。
next: 提供或恢复 806 原实施树/提交和 work 收据，再做对应验收与最终复审。
本轮只确认送审前提不足，未声称检查完全部语义风险，也不授权 merge。
落盘前复核 master HEAD 未变，806 指定目录及新 crate/consumer 仍不存在，
未出现 806 worktree；本轮 Plan/债务记录差异通过 `git diff --check`。
