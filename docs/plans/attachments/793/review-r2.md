# PLAN-793 独立最终复审 R2

2026-10-16；stage=review；review_scope=final；outcome=pass。

## 基线与审查边界

- plan_id=PLAN-793；plan_revision=1；读取主检出当前合同，保留 R1 历史。
- reviewer_identity：当前 Codex 审查上下文，未参与实施；独立于实施证据重建结论，未另派 agent。
- worktree_path=`D:/autostack/.wt/lang-793/auto-lang`；branch=`codex/plan-793-dev`。
- reviewed_commit=`a8faf1e311debb96df0c3ed660b7d7ccbb0e23a4`；base_commit=`579f6dec0941fb711664679a9cb8c29fa71bad20`；R1基线=`1bb5660b44d7835617fd88aa0dfbf7ee433b280e`。
- 入场/结束 HEAD 一致，受管实现树 clean；auto-down 组内依赖仍为 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，auto-lex/auto-val 为同树 path 依赖。
- R1→R2 提交差异仅 `.gitignore` 精确例外、golden 入库、门面格式、互操作消费者测试；未修改生产诊断算法、parser、infer、owner侧转换、依赖或测试配置。

## Finding closure

| 原 finding | 结果 | 本轮独立证据 |
|---|---|---|
| F-793-R1-01 / P1 | CLOSED | `git ls-files` 已列 fixture；直接读取 `git show HEAD:crates/auto-diagnostics/tests/fixtures/v02-golden.json` 的字节，与工作树件和主检出旧臂冻结件三方相等，SHA256 均为 `e97b9450031c73a3ca05280489e67b92b5236ab06f5fd7ff47a87dee2742a09d`；38例重放通过。期望不由新实现重生成，提交缺件的根因已关闭 |
| F-793-R1-02 / P2 | CLOSED | 互操作消费者7/7通过：真实 parser→check_field_type→TypeError::FieldMismatch/E0106、labels(57,1)，有效 int 字段控制成功；attach后仍Type族；真实 check_symbol→Name/E0201及有效名称控制通过。源码确认 parser.rs Pair 臂调用 infer::check_field_type 并将错误加入 parser.errors，未人工构造期望错误；原 compile 错误已正确重标 parser恢复并断言 SyntaxWithSource/E0007/source=t.at |
| F-793-R1-03 / P3 | CLOSED | 同六文件 `rustfmt --edition 2021 --check` exit0；原门面重导出只做格式变化，20项保持；`git diff --check 579f6dec0 HEAD` exit0 |
| F-793-R1-04 / P2 | CLOSED | SD-03明确同步 auto-lex 的 From/attach_source/聚合归属及已消除的Name边；保留 parser错误收集上限归属；SD-02门面/实现路径已分清；frontmatter supersedes加入auto-lex/project.md，公开面计数20。未在review写 live Spec |

旧例闭合：新旧路径同一类型双向互传、坏语法单调返回、route→Msg、String/io的?传播均由本轮V03重跑；V02重跑保持CJK、source、重复attach、聚合顺序及各诊断族逐字段投影。
修复没有替换 parser/traversal/executor；新增测试直接覆盖上轮遗漏的生产类型/名称路径，未缩小既定合同。

## 验收映射与证据

| AC | tasks | 结果 | artifacts / evidence |
|---|---|---|---|
| AC-01 | T01/T02/T05 | pass | V01独立consumer 5/5；verify-deps.sh exit0；strict pass/0 violations/2 grandfathered。基础闭包无auto-lang/man/vm/ui，fresh manifest而非借用主库rlib |
| AC-02 | T01/T02/T03/T04/T05 | pass | 提交内V02期望38例重放、V03七项及有效/无效控制通过；类型身份、诊断结构和错误阶段保持 |
| AC-03 | T03/T04/T05 | pass | 20项薄re-export、单一实现、owner侧两个From保持；parser/type/name/route生产接线和跨路径传播通过 |
| AC-04 | T01/T06 | pass | 消费清单/剩余DAG/净迁移账可回放；阶段证据归属已修正，永久fixture已交付；保留未外置parser/AST等边及owner |
| AC-05 | T07 | pass | R1四项全闭合；未发现新合同内遗漏、未经批准延期、workaround或新增宽豁免；SD01/02/03冻结通过 |
| AC-06 | T08 | pending | merge-owned，不能将review通过解释为已完成合入交付 |

本轮命令均在原worktree运行，`RUSTC_WRAPPER=`沿用环境故障旁路：

```text
cargo test -p auto-diagnostics --offline
  exit0；v02_golden_replay 1 passed（内部38例），1文档示例ignored；新crate零警告
cargo test --manifest-path test/fixtures/modularity/diagnostics-consumer/Cargo.toml --offline
  exit0；5 passed
cargo test --manifest-path test/fixtures/modularity/diagnostics-interop-consumer/Cargo.toml --offline
  exit0；7 passed
bash test/fixtures/modularity/diagnostics-consumer/verify-deps.sh
  exit0 PASS
python scripts/modularity-audit.py --mode strict --output <本附件目录>/review-r2-audit.json
  exit0；strict verdict=pass violations=0 grandfathered=2
rustfmt --edition 2021 --check <新crate三文件、旧门面、两个consumer src/lib.rs>
  exit0
git diff --check 579f6dec0 HEAD
  exit0
```

原始日志：`review-r2-v01.log`、`review-r2-v02.log`、`review-r2-v03.log`、`review-r2-deps.log`、`review-r2-audit.json`。
原有auto-lang 411警告及shim-metadata既有警告仍在；本轮修复无生产逻辑改动，新增测试无新增警告。

证据复用说明：逐项 `git diff --quiet 1bb5660b4 HEAD -- <path>` 确认根Cargo.toml/lock、两个consumer manifest、新crate manifest/lib/error/golden重放程序、`.cargo/config.toml`、边界配置、parser、infer及两个owner转换均未变；依赖revision亦未变。
故复用R1 `cargo t error_tests` 4/4、`cargo t error_spans` 8/8 的生产诊断证据及R1逐字搬迁审计，不重复未受影响测试。LSP/CLI消费源码与manifest未变，沿用T04编译证据；本轮互操作fixture再次实际编译auto-lang。

## 规范增量冻结

批准 `sd-materials.md` 的 SD01新增auto-diagnostics、SD02修改frontend、SD03窄同步auto-lex；该文件为冻结正文，SHA256=`cfa16b87d5f5d4fbc8e2671244cbf50546bf51aeb7f647ac62e221a575e994fc`。
规范目标的现行输入SHA256：

- `docs/specs/auto-lang/frontend/overview.md`：`1e860a18f54aa3b5b324da8b009b24bf609e9666da1aaeb19b301ff7cb2d729a`。
- `docs/specs/auto-lex/project.md`：`0209bf7dd192b4a95a308308ff224de9bcc664afcfd1269f00ae0ae310a18544`。
- `docs/specs/goals.md`：`c26944f8f2d701b84088cca0c40ba7fe1db9f3d8ca2c4fc472ce5fa2482202d9`；GOAL-003仍存在。
- 新目标 `docs/specs/auto-diagnostics/project.md` 当前不存在，由merge新增。

元数据与上述三目标一致。SD只澄清既定抽取归属，不改变实现合同，plan_revision保持1；merge若遇规范输入漂移须先对照增量调和。

## 下一步与未执行门禁

Plan→reviewed，T07完成，current_step=7/8；无open findings。
pending_closeout_gates：merge在最终冻结提交执行裸 `cargo t` + `cargo tv` + 新crate/consumer验收，再沉淀SD01/02/03、ledger/索引、归档、guard后清树，并按T08处理792闭包刷新互链。
未触trans/UI生成/aavm，不增其档；tf仅到期时主检出批量单实例。本轮未跑整档、未跑docs_gen、未合并或清理worktree。
