# PLAN-783 r1 · independent final R7 affected review · needs_fix

2026-10-16（Asia/Shanghai）。本次独立Codex复审上下文，未参与实施或merge；未创建代理/联系其它聊天。
**RuntimeInjection调用点适配通过源审和真实纵线复验；closeout人读报告有1项P2缺陷，归档清理暂不通过。**

## 基线及范围

- 原计划同树 `D:/autostack/.wt/lang-783/auto-lang`，branch `plan-783-dev`；受审
  **c00f77b526be85e758afaf2187f316eaa01559d8**。R6基线
  `aa8c666caa793891c9085efa3f8675ef8f6fe7d1`；补丁已rebase，不能把二者间全部master差异归给本Plan。
  本次适配提交 `adfd66a65`，报告提交 `c00f77b52`。
- main HEAD `631a83afce79a8fa5e1309e90f2f1f06e532db1a`，已验证delivery为其祖先。
  Plan仍未归档；main Plan回写前SHA256
  `AFFE0CE5372DBAB64E9B7388475FEDA7C0D7BA31BDFE28E554A655476478CFE6`。
  当前是**已落地后的受影响复审**，不撤销/伪造历史落地事实；只暂停后续归档清理。
- ac-core tree `324210641c2d2f7b5a3eac36ed3ed07632630cef`（已含PLAN-782，R6依赖证据不能原样套用）；
  auto-cache tree `c4524240cf1a0e6c50a9176fe673cdf2527076e9`、probe lock blob
  `93582090320f0b07fd6c12d83c15f867ec1d3366`、聚合脚本blob
  `078a9270e5be7f16fb1f5573c031865935798940`未变。
- [冻结SD](review-r7-spec-delta.md) SHA256
  `EB210819336C0D80D1A4590155E6E59B42F65AD9B0B040CE951FB2C538875080`，与R6相同。
  [冻结人读报告](review-r7-perf-report.md) SHA256
  `78A5A8C8832FF068DDE20C3A3C021AB240E0314C604C54FF837669F7356C0C09`；
  [冻结JSON](review-r7-perf.json) SHA256
  `D4A711B467621237BB88658576FA3BFBE2508F6043B73A7FA58860332EFE8DAB`。

已读当前auto-plan-review skill及main交接。其merge收据明确：先落地后发现PLAN-782 API语义冲突，
再补默认参、重跑closeout、再次rebase落地，归档清理等R7。R6 pass不覆盖这次代码/依赖变化。

## 受影响代码与验收

探针相对R6仅改 `driver.rs` 1处和 `tests/admission_.rs` 3处lower_object调用，
均传入 `RuntimeInjection::default()`。新ac-core结构derive(Default)，三个Option字段
fail_nth_alloc/fail_ledger_nth/set_budget均None；注入仅在text lowerer读取，
core/scalar路径不因默认参改变。支持面未扩大到text/text-record；未绕过verify或新增生产入口。

本轮在冻结树设置 `RUSTC_WRAPPER=''` 后运行
`scripts/verify-ac-artifact-probe.ps1 -Scope acceptance`，原始输出见
[R7验收](review-r7-acceptance.txt)：**PASS，18族93 passed、0 failed、1白名单ignored**。
进出实施树clean，提交/依赖/报告指纹未漂移。
cargo fmt --check及范围内git diff --check通过，无探针新增warning，仅未改shim基线warning。
旧存储/布局代码未改，原R1–R6独立反例结论在新依赖下相应永久回归重跑后保留；
native/admission/invalidation/dep_audit等真实消费面重新验收，不只依赖机械适配的说明。

closeout的JSON source_commit为 `a3cbe7643ad5d41f13910eb66f5de79c319c7c49`，该对象仍可读，
虽非rebase后祖先，但对当前HEAD比较probe/ac-core/auto-cache/验证脚本全输入目录，
`git diff --exit-code`为0：[审计输出](review-r7-report-audit.txt)。
六场景各30/30、semantic_failures=0，GC21趟/73删除/69重叠存在。
不因rebase后SHA不同单独否定输入等价；merge仍须保存可复用的完整512门禁收据并核对
运行配置/输入等价。当前没有在本轮真实重跑512或30样本，不把定向验收冒称closeout。

## F-783-R7-01 · P2 · AC-05/06 / T-06/07 — closeout报告再生破坏表头且混入多轮旧读数

位置 `docs/reports/plan783/perf-report.md:1,15,19–28,42`。
这是R6 pass之后merge再生产物的新缺陷，非重新否定R6已关闭输入。

- 第1行在标题前串入多段残缺GC表格片段，包括516/707/774及其它旧数字，标题不再独立成行。
- 当前JSON及表格主体：fresh P50约246、hot187、restart217；正文却写fresh346、
  hot327、restart232，以及hot/cross约270，混用了前轮数据和本轮归因。
- 当前JSON GC P99=502.541（约503ms），正文仍写774ms；场景表内仍写22趟/77删/73重叠，
  而JSON与报告前言均为21/73/69。同一文档内部矛盾。
- 末段冷编译/命中P95仍398/437，JSON实际约258/206。正文还同时声称与512并行/串行，
  应明确描述实际采样轮，不能继续沿用旧轮负载解释。

区分性证据是冻结JSON与冻结人读报告逐项直接比较，不依赖性能重跑，也不引入新性能上限。
修复应恢复标题、清除拼接垃圾，统一六场景表格、GC事件计数和正文/比较/负载说明到同一份
冻结JSON。若生成/替换工具导致该形态，纠正对应方法并读回完整文档；不只替换表内数值。
**此项可用docs-only提交修复，不要求为文档纠错重新执行512或30样本。**

## Spec输入与路由

当前canonical已经由merge沉淀，受审哈希：

| 文件 | SHA256 |
|---|---|
| auto-ac/artifact-contract.md | B4378653DDAEF6309A9E30C0DE3D3D83EDE1B0B010E00FAF067D3B243C63FB8A |
| auto-ac/artifact-probe.md | 31FEF2E8711D66E9E00BD5DFA724B7F62F81EBBA198C9F983C332B6D10134911 |
| auto-ac/project.md | 0040541B14F86741F36A72639AFFF8ADA4958F559811A30DDDCD8CE0E2DE081C |
| auto-cache/project.md | 7A48BA94FDAE6B2A80CF51CBFF9E49A101942C28217D782FAFBB9A81AA4FB30C |
| auto-hir/stage-contract.md | 13050FD6364E83BBEB106148CE1EEBAF85087D9A403881A706F83D7C2B1F3472 |

SD目标路径/实验边界/持久HIR信任规则与新canonical相容，touched_goals=[]仍合理；
本轮未改canonical或live ledger。归档计划链接目前依赖merge完成，属于pending收尾而非另立finding。
AC-01/02/03/04 pass（分别由lib/identity/admission、layout、native_resume/invalidation及
reachability/concurrency/recovery/gc_budget/store_boundary连同R1–R5回归重验）；
AC-05/06因报告为partial（六场景JSON与dep_audit通过），AC-07未通过本轮最终复审，
AC-08已落地/沉淀部分保留，归档清理仍pending。R6其余已闭finding不重开。

next=work/merge在原树提交报告纠错后，做文档受影响复审；禁止在main直接改实施报告。
当前Plan置executing 5/9，保留T-01..05，重开T-06/07/08，T-09pending。
本Plan新适配只改experimental探针消费调用点；master其它Plan的crates改动不归入本次触面，
Category A，未跑根cargo档/docs_gen。复审不重做merge、不删除工作树或修报告。
