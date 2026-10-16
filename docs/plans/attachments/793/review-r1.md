# PLAN-793 独立最终复审 R1

2026-10-16；review_scope=final；outcome=needs_fix。

## 基线与独立性

- reviewer_identity：当前 Codex 审查上下文；本上下文未实施该计划，按源码和实际运行重建证据，未另派 agent。
- plan_revision=1；计划读取自主检出 `docs/plans/793-diagnostics-foundation-extraction.md`（本轮只更新审查簿记，不改变合同）。
- worktree=`D:/autostack/.wt/lang-793/auto-lang`；branch=`codex/plan-793-dev`。
- reviewed_commit=`1bb5660b44d7835617fd88aa0dfbf7ee433b280e`；base_commit=`579f6dec0941fb711664679a9cb8c29fa71bad20`。
- 唯一实现提交 `827452a2acbb300d5be3d25abcd0054a4a87d5b6` + 边界重锚提交；入场和结束 HEAD 一致，受管实现树 clean。clean 不包含下述 ignored fixture，不能据此认定交付完整。
- auto-down 组内依赖=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；auto-val/auto-lex 为同提交树内 path 依赖。
- spec_inputs SHA256：frontend/overview.md=`1e860a18f54aa3b5b324da8b009b24bf609e9666da1aaeb19b301ff7cb2d729a`；auto-lex/project.md=`0209bf7dd192b4a95a308308ff224de9bcc664afcfd1269f00ae0ae310a18544`；goals.md=`9f2b22ebf1362941e58c69166c8c58c32a36121b877b63e4509391206e4cd72e`。
- SD 冻结候选 `sd-materials.md` SHA256=`50535991cd2255e73923dd7b629a37c4ab7d60e19a7f555cad0c8d249ddf9499`；本轮未批准该增量，见 F-793-R1-04。未修改 live Spec/ledger。

## Findings

### F-793-R1-01 / P1 / AC-02、AC-04 / T-05、T-06：永久重放依赖未提交的 golden

`crates/auto-diagnostics/tests/v02_golden_replay.rs:312–314` 在运行时读取
`tests/fixtures/v02-golden.json`，但受审提交没有该文件：

```text
git ls-files crates/auto-diagnostics
  Cargo.toml, src/error.rs, src/lib.rs, tests/v02_golden_replay.rs（仅四文件）
git ls-tree -r --name-only HEAD -- crates/auto-diagnostics/tests/fixtures
  （空）
git check-ignore -v crates/auto-diagnostics/tests/fixtures/v02-golden.json
  .gitignore:161:*.json
```

本机文件存在，SHA256=`e97b9450031c73a3ca05280489e67b92b5236ab06f5fd7ff47a87dee2742a09d`，
所以本机测试 38/38 通过。干净检出没有该文件，`read(...).expect("fixture json")`
必然报文件不存在；当前绿色不能作为提交可重现证据。

修复：把原冻结字节纳入受管提交（精确 ignore 例外或强制加入），不得用新实现重生成
expected；核对原冻结件哈希一致，并证明 `git show <修复提交>:crates/auto-diagnostics/tests/fixtures/v02-golden.json`
存在且重放成功。主检出的旧臂附件也应在最终知识归档时随证据保存。

### F-793-R1-02 / P2 / AC-02、AC-03 / T-04、T-05：所谓真实类型错误实际是 parser 错误

`test/fixtures/modularity/diagnostics-interop-consumer/src/lib.rs:100–119`
的 `real_compile_error_keeps_stage` 使用 `let x: int = "s"`，断言
`must have either a type annotation` 与 `auto_syntax_E0099`。
本轮原测试运行通过，恰好确认它接受的是语法错误；该文本来自
`parser.rs:8626` 的 `SyntaxError::Generic`，不是类型推断诊断。
因此 V03 所需“真实 parse/type/route 错误”只有 parse/route，不能以
函数名、人工构造 TypeError 或另一个 parser 错误证明真实 type 路径。
T-05/T-06 收据 §3/4 将此例标为 infer/语义/VM 证据，结论超出实际 oracle。

修复：选用语法有效且确实触发生产类型检查的坏程序，先断言其可 parse，
再由实际类型检查入口断言 Type/Name 等约定的真实阶段、具体 code/span 和旧/新路径
投影保持；加入对应有效程序控制。若 compile_once 本来不执行该类型检查，改用真实
生产入口而非继续调低期望为 Syntax；修正收据的阶段归属。现有 parser 用例可保留。

### F-793-R1-03 / P3 / AC-05 / T-03、T-06：改动文件格式检查未净

本轮对六个本次 Rust 文件执行 `rustfmt --edition 2021 --check`，exit=1，
唯一差异为 `crates/auto-lang/src/error.rs:20–24` 的 re-export 换行。
它是本次新写门面，不属于无关文件的预存格式漂移。work 收据“本批文件 fmt 清洁”不成立。

修复：格式化该文件并提交，再执行同范围 check。`git diff --check base HEAD` 已通过，
但不能替代 rustfmt 检查。

### F-793-R1-04 / P2 / AC-05 / T-06：规范增量遗漏现行 auto-lex 契约的同步纠正

现行 `docs/specs/auto-lex/project.md` 的“词法行为与错误契约”仍规定
`From<LexError> for AutoError`、多错误/attach_source 归 auto-lang error.rs，
“边界与已知限制”仍列 `error.rs` 的 `ast::Name` 为未迁残留。
SD-01 建立新归属，SD-02 仅改 frontend，落地后会留下相互矛盾的 current-state 规范。
此外 SD-02 §2 把 `crates/auto-diagnostics/src/error.rs` 写作“门面”，又重复列
`{error}.rs` 为实现；真实门面是 `crates/auto-lang/src/error.rs`。

修复：在本计划增量/元数据中加入 auto-lex/project.md 的窄同步修改，明确
AutoError/From/attach_source 归 auto-diagnostics，多错误收集上限仍归 parser/lib；
移除已经切断的 Name 残留，修正 SD-02 两个路径。不得在 review 阶段直接发布 live Spec。
另将“18 名冻结公开面”簿记改为实际 20 个 re-export 名称；当前代码未漏公开项。
这属于既定抽取的知识一致性修正，不新增实现范围。

## 验收与反控审计

| AC | 结果 | 独立证据/边界 |
|---|---|---|
| AC-01 | pass | standalone consumer 5/5；verify-deps.sh exit0；cargo tree normal/build 无 auto-lang/man/vm/ui；777 strict pass、0 violations、2 grandfathered |
| AC-02 | partial | 本机 V02 38例逐字段通过，新旧同一类型互传通过；F01 提交缺 oracle，F02 真实 type 路径缺失 |
| AC-03 | partial | 逐字搬迁核对仅头注、Name→AutoStr及载荷注释；原路径20项完整，owner侧两个From仍在；parse/route/?反控通过；真实type证据待F02 |
| AC-04 | partial | 消费/DAG/搬迁账可核对，所有者无违规倒灌；F01重放件未交付，阶段归属簿记须纠正 |
| AC-05 | fail | 四项 finding；SD未批准，不能标 reviewed |
| AC-06 | pending | merge-owned；本轮未合入/归档/清树 |

实际命令均在上述唯一 worktree 执行，`RUSTC_WRAPPER=` 旁路已有 sccache 环境故障：

```text
cargo test -p auto-diagnostics --offline
  v02_golden_replay 1 passed（内部38例）；1文档例 ignored；新crate零警告
cargo test --manifest-path test/fixtures/modularity/diagnostics-consumer/Cargo.toml --offline
  5 passed；零警告
cargo test --manifest-path test/fixtures/modularity/diagnostics-interop-consumer/Cargo.toml --offline
  5 passed；auto-lang 411既有警告，shim-metadata 1既有警告
bash test/fixtures/modularity/diagnostics-consumer/verify-deps.sh
  exit0 PASS
python scripts/modularity-audit.py --mode strict --output <本附件目录>/review-r1-audit.json
  strict verdict=pass violations=0 grandfathered=2
cargo t error_tests
  4 passed，6872 skipped（其中3项为autodown空输入控制，1项source wrapper）
cargo t error_spans
  8 passed，6868 skipped
rustfmt --edition 2021 --check <新crate三文件、旧门面、两个consumer src/lib.rs>
  exit1；仅旧门面re-export布局有差异
git diff --check 579f6dec0 HEAD
  exit0
```

原始定向日志：`review-r1-error-tests.log`、`review-r1-error-spans.log`、
`review-r1-deps.log`；结构边界完整报告：`review-r1-audit.json`。
反控包括坏语法不成功、Lex Raw保留无code、结构化Lex保留code、重复attach不换源、
String/io的?传播、CJK span和聚合错误。未改实现或用新输出覆写golden。
纯搬迁外未发现新增包装/吞错/删除变体；未发现获准范围外的实现延期或宽边界豁免。

pending_closeout_gates：最终修复提交复审通过后，由merge统一执行冻结提交上的裸
`cargo t` + `cargo tv` + 新crate/consumer验收；无trans/UI生成/aavm改动，本轮不触发其档。
本次仅运行定向`cargo t <filter>`，未运行裸t/tv/tf等整档。未运行docs_gen。

## 路由

回 `executing`；保留 T-01/T-02，重开 T-03..T-06，current_step=2/8；T-07/T-08待办。
在原树完成 F-793-R1-01..04 并提交，更新 finding closure 和真实验收证据后再送审。
此次仅授权审查，不代为修复实现、合并或清理。
