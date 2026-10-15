# PLAN-783 r1 · independent final R8 document affected review · pass

2026-10-16（Asia/Shanghai）。本次独立Codex复审上下文，未参与实施或merge；未创建代理/联系其它聊天。
**F-783-R7-01关闭；恢复reviewed 8/9。归档清理与可复用closeout收据核验仍由merge完成。**

## 基线

- 原树 `D:/autostack/.wt/lang-783/auto-lang` / `plan-783-dev`，受审commit
  **46dc5f442851bbcc6d7e7c647d21b0966f1f5a64**，本轮base
  `c00f77b526be85e758afaf2187f316eaa01559d8`。进出clean，HEAD未漂移。
- main HEAD `1cb4009eb1a2195ae39c69ace6faaa0123128ac5`；main Plan r1回写前SHA256
  `9BC9C3A4CF123EE8D10DBAF66C65957FDA928AA293A2696D764ED5F45DAE8A50`。
- ac-core tree `324210641c2d2f7b5a3eac36ed3ed07632630cef`、auto-cache tree
  `c4524240cf1a0e6c50a9176fe673cdf2527076e9`、probe lock blob
  `93582090320f0b07fd6c12d83c15f867ec1d3366`、验证脚本blob
  `078a9270e5be7f16fb1f5573c031865935798940`均与R7一致。
- [冻结SD](review-r8-spec-delta.md) SHA256
  `EB210819336C0D80D1A4590155E6E59B42F65AD9B0B040CE951FB2C538875080`。
  [修复后冻结报告](review-r8-perf-report.md) SHA256
  `3B420A2560F0C62C245F3C9BE16D819451709E2D0A89D459510121DE974B73DC`。
  perf.json SHA256仍为 `D4A711B467621237BB88658576FA3BFBE2508F6043B73A7FA58860332EFE8DAB`。

## 受影响验证

本轮实际diff仅 `docs/reports/plan783/perf-report.md`，14+/11-。
已阅读work的原因诊断与修复方法：closeout脚本仅发布JSON，人读报告由手工维护；
此次纠正的是先前手工拼接残片，并完整读回核对，未声称修复不存在的自动生成器。

完整读回文档，独立执行[审计脚本](review-r8-check.ps1)，
[输出](review-r8-check.txt)全绿：

- 标题在文件首独立成行，连续表格碎片消失。
- 六场景各行P50/P95/P99与30/30计数逐值等于JSON四舍五入结果。
- GC21趟/73删除/69重叠、尾延迟503ms；fresh/hot/restart P50=246/187/217ms，
  fresh/hot P95=258/206ms，表格和正文一致。restart比hot高约30ms的方向正确。
- 旧轮数字残片清除；约270ms及并行负载仅保留在明确标注R5历史段落，
  当前closeout采样说明一致。PEAK_TRANSIENT仍明确为成功预留上界。
- 当前JSON与R7冻结JSON逐字节hash一致，未以修改原始读数来迎合人读报告。

`git diff --check c00f77b52 HEAD`通过。对probe/ac-core/auto-cache/验证脚本/JSON执行
`git diff --exit-code c00f77b52 HEAD -- ...`结果0，未变验收输入。
因此明确复用[R7验收](review-r7-acceptance.txt)的18族93 passed/0 failed/1白名单ignored，
以及R7接口适配源审结论；不把本轮文档核对说成重新执行测试。
Category A：未运行根cargo档、docs_gen，也未为文档纠错重跑512churn或30样本。

## AC、Spec与交接

AC-01..04：pass，依赖/代码/测试配置未变，复用R7证据；AC-05/06：pass，
报告与JSON/SD一致，F-R7-01关闭；AC-07：本轮final pass；AC-08：先前已落地/沉淀
事实保留，剩余merge收尾pending。没有新的合同内阻断、未批准延期或旁路。

canonical Spec哈希与R7完全一致：

| 文件 | SHA256 |
|---|---|
| auto-ac/artifact-contract.md | B4378653DDAEF6309A9E30C0DE3D3D83EDE1B0B010E00FAF067D3B243C63FB8A |
| auto-ac/artifact-probe.md | 31FEF2E8711D66E9E00BD5DFA724B7F62F81EBBA198C9F983C332B6D10134911 |
| auto-ac/project.md | 0040541B14F86741F36A72639AFFF8ADA4958F559811A30DDDCD8CE0E2DE081C |
| auto-cache/project.md | 7A48BA94FDAE6B2A80CF51CBFF9E49A101942C28217D782FAFBB9A81AA4FB30C |
| auto-hir/stage-contract.md | 13050FD6364E83BBEB106148CE1EEBAF85087D9A403881A706F83D7C2B1F3472 |

SD-01..05、supersedes/new路径及空touched_goals保留：实验不扩大语言/自举闭包。
本轮未改canonical/live ledger。R6历史存储/布局闭合与R7新依赖/适配证据继续有效。

next_responsible_role=merge；Plan **reviewed 8/9**，T-08通过，T-09pending。
merge需核验既有512/30完整收据及输入/运行配置等价，再落本文档修复、完成剩余
登记/归档与wt-guard清理。R7已经证明a3cbe7643与delivery输入目录等价，本轮未变；
不因docs-only修复自动要求重跑载荷，也不代替merge宣称收据已全部核验。
本轮未修实现、未merge、未归档或删除工作树，未宣称交付收尾完成。
