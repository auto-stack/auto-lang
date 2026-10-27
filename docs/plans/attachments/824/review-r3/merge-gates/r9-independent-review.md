# PLAN-824 r9 独立复审

- outcome: needs_fix；reviewer: /root/review824_sync；reviewed: 0face6c9402b59b22ff4e18ce4800da8c27332a5；base: 05684bed8acbac1b6723d2d261e280f0ace9c967。
- F-R9-01：824新增 I11 binding.tsv 与 binding.md 仍称30修订 campaign 为 P95 双门，当前应记 nearest-rank P90≤2000ms，P95为诊断；852原始历史字段和512修订逐次<1000ms独立阈值保持。
- F-R9-02：824脚本的增量测试程序含campaign与512-churn两个计时用例，libtest默认并发，违反串行采样合同；须串行并定向复验incremental和baseline。
- 已核：portable深度257归Budget正确，一致深度256/257及F02错配负控有效；semantics 21+32、resources 18、incremental 6、cache 9、baseline/autodown/first-case与F01/F02/F03原反例均见证据；858算法/等值/无效样本及规范未覆盖，resources_tier为纯格式。823在途尚未合入并抢编号，后合者改号并复审。
- 本轮未跑整档；修复后须独立复审，acceptance/cargo t/tv/workspace冻结门及批量到期核查仍pending。
