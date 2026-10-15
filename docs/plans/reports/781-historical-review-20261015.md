# PLAN-781 历史独立审计（2026-10-15）

stage: review
review_scope: historical-final-audit
reviewer_identity: 本次 Codex 上下文；未参与 781 实施；未使用子 agent。
plan_id: PLAN-781
plan_revision: 2
outcome: needs_fix（历史审计结论；归档终态不回退）
inspected_checkout: D:/autostack/auto-lang
branch: master
reviewed_commit: 715d96a9c9c1aa1ef97abdaa0eba7afa12add4e0
historical_implementation_tip: e53b85fe7bf8ccffc3113b87a9f957e154f82f0e
historical_reviewed_commit: dba0d7e5c（原记录；完整历史 SHA 可由 git rev-parse 解析）
delivery_specs_commit: 3e1b5fc03
dependency_revisions: Reader@29b5dec（原证据依赖；本次未运行 consumer）、auto-down@c39ac7f（原证据依赖；本次未构建）。

## 基线与限制

用户请求检查 review 时，781 已归档且原工作树/分支已删除。确认 e53b85fe7 是当前 HEAD 祖先；本报告涉及的 scroll_anchor.rs、ui_gen/vue.rs、test_reader_import_budget.py、plan781_budget_http_tests.rs 在 e53b85fe7..HEAD **零差异**，所列缺口属于已交付 781 面，不是后续漂移。

遵守 archived 不重开与主检出不构建规则，本次没有创建 reviewer 工作树，没有修改实现，也没有运行 cargo 档位或重放 native/browser 全矩阵。独立证据为源码路径反例、原始 JSON 与验收合同对照。反例探针明确不冒充窗口实测。主检出的 778/KNOWN-DEBT/ledger_770.py 在途改动不属于本审计。

规范输入：归档计划 §5/6/7，reports/781-interface-decision.md（r1-r6）、781-budget-decision.md（r1-r3）、781-channel-decision.md，以及 docs/specs/auto-lang/ui/design/scroll-child-anchor.md、native-acceptance-channel.md、docs/specs/auto-lang/vm/design/handler-execution-budget.md。原 Spec delta SD-01..07 已发布于 3e1b5fc03；本审计未修改 canonical Specs 或 ledger。

## Findings

### F-781-R01 — P1：标准 controller 句柄无法在原生锚点 ID 中往返

影响 AC-01/02/03，T-02/03/05。`scroll/controller.rs::next_controller_handle()` 生成 `@scrollctl:N`，builder 将完整 controller key 注入 ScrollAnchorMark；`iced/scroll_anchor.rs:29-36` 将 pane/key 以冒号拼接，却在第一个冒号拆分。

输入 `(pane="@scrollctl:1", key="chapter-50")` 编码为 `scrollanchor:@scrollctl:1:chapter-50`，实际解码为 `("@scrollctl", "1:chapter-50")`。采集表无法命中请求的原二元组，controller 将其判为 not_found；标准 `scroll_controller()` 用户因此不能使用新锚点。现有双端 fixture 的三个 controller 都是无冒号的手写字符串，漏掉标准构造入口。

修正要求：采用无歧义编码/结构化身份并正确读取 iced Id（当前通过 Debug 字符串取值也不能保证引号/反斜杠 key 往返）；加入真实标准 controller + 含分隔符 key 的双端验收，保留手写句柄兼容。

### F-781-R02 — P1：Vue 目标查找包含嵌套 pane，违反最近 pane 作用域

影响 AC-01/02，T-04/05。`ui_gen/vue.rs:3938-3942` 对整个 pane 子树 querySelectorAll 后取最后一个，没有排除内层 scroll-pane。outer 与 inner 各有 key=same 时，outer 请求会选中 inner 的目标，滚动 outer 并报告 applied；只有 inner 定义 key 时也不会返回 not_found。相同子树查询还会把内层 managed 标记误当 outer 自己的 unsupported 条件。

规范持久层明确要求最近 ordinary scroll-pane 归属、不同 pane 同名不串线。原 fixture 的并列 pane 不覆盖嵌套关系。本次执行的是从实际生成器提取的 helper，DOM 查询结果用标准后代查询形状作桩；证明 helper 缺少归属过滤，未声称已重放浏览器几何。

修正要求：查找时按最近 pane 所有者过滤（包括无 controller 的嵌套 pane），managed 判据采用同一归属规则；补嵌套同名、仅内层有 key、内层 managed/外层 ordinary 的真实 Vue/原生对照。动态 key 进入 CSS selector 时也须安全转义，避免合法引号 key 导致异常。

### F-781-R03 — P2：10MiB 全部章节一致性尚无验收证据

影响 AC-13，T-17/18。V-17 要求章节按规范拆分逐字重建一致。`test_reader_import_budget.py:220,247,283` 固定调用默认 deep=False，只检查首/中/尾三章；deep=True 没有接入 run_v17。冷启动只检查一章。原 out-cold 报告明确为 `chapter_bodies_sample`、3 checked，计划 T-17 记录也承认抽样。

受管 original 文件 hash 只能证明原件复制正确，TOC 数量相同不能证明未抽查章节的拆分/正文正确。不能把 3/1347、3/1346 抽样扩大为全部章节逐字重建 PASS。

修正要求：提供可执行且必选的全章/等价完整内容 oracle，绑定 TXT、MD 输入 hash 与冷重启持久化结果，加入非抽样章损坏反例证明 oracle 会失败。若成本无法接受，应先明确修订合同，不能静默降级。

### F-781-R04 — P2：V-18 生命周期声明超过实际断言覆盖

影响 AC-12/14，T-16/18。`plan781_budget_http_tests.rs:126-141` 发请求后立即断开，只断言稍后 ping=200：没有确认 handler 已开始，没有观察中断之后的业务写入、scope/task/许可计数或 deadline/关闭终态。原最终复审的“中途杀进程”反例也不能证明活进程取消后不再写入。

本项是 required oracle 缺口，不据此断言生命周期实现一定有 bug。预算提升可让一次同步忙段持续数分钟；“服务最终还能 ping”无法排除取消后继续执行并写库。旧生命周期族证据需具体映射到 LegacyHttp 长忙路径及断言，不能只写 th 全族通过。

修正要求：用进入 handler 的握手、可观测副作用和资源基线验证 deadline/取消/断连/关闭；明确单终态、无迟到写入及回收。补齐实际请求证据后才能将 V-18 判 pass。

## 验收与 Spec 判定

| AC | 本次结果 | 依据 |
|---|---|---|
| AC-01/02/03 | fail / 覆盖不足 | R01/R02；不否认已有普通命名 pane 正例 |
| AC-04 | partial | 接口交接在位，但作用域/标准句柄能力声明需随修复复核 |
| AC-07/08/09/10 | 未重新授 pass | 原阶段 A 证据仍在 D:/p781-t07/rb-a；本次未重放其完整真实窗口矩阵 |
| AC-11 | 原证据支持，未重跑 | 默认/显式预算及非法配置的旧证据，未发现本项新反例 |
| AC-12/14 | partial | 原累计计数/命名/closure证据与 R04 的生命周期覆盖区分 |
| AC-13 | partial | 原导入成功与原件 hash 证据有效；全章 oracle 缺失 R03 |
| AC-05/06 | 历史 pass 不能证明上述缺口关闭 | 保留历史交付事实，不追改终态 |

SD-01/02/03 对锚点可用性及 pane 隔离的声明须经 R01/R02 修复后重新核对。SD-05/06/07 的生命周期与 10MiB 验收表述须由 R03/R04 补齐证据。规范的 nearest 公式仍存在“部分可见不动”文字与对齐公式的解释张力，本次未把它另立为已确认代码缺陷；后续修复应核清其既有合同。

## 可重放证据与下一步

命令：`node docs/plans/attachments/781/historical-review/probe.cjs`（仓库根目录执行，退出 0 表示确认三个源路径反例/缺口仍存在，不是产品验收通过）。探针输出输入/实际/期望及三源文件 SHA256。历史原始证据抽查：D:/p781-t07/rb-c/reader_import_budget_report.json、D:/p781-t18/out-cold/reader_import_budget_report.json。合入门与批量门见原 T-08 及 .last-batch-regression.json，批量收据已有 781 覆盖，不能替代上述行为验收。

pending_closeout_gates: 本次不进入 merge；修复需独立后续 Plan/worktree，受影响验收与独立复审通过后，按新冻结提交执行必要 merge-owned 档位。
next: 将 R01..R04 纳入后续修复 Plan；781 保持 archived，不在本次 review 中修实现或新建实施计划。
