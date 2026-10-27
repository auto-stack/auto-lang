# PLAN-824 r9 修复独立复审

- outcome: pass；reviewer: /root/review824_sync；reviewed: cf2aa647ab33628f1d639d08ede502c3ba6f8ddc；parent: 0face6c9402b59b22ff4e18ce4800da8c27332a5；base: 05684bed8acbac1b6723d2d261e280f0ace9c967。
- F-R9-01 CLOSED：两处I11说明改为858 nearest-rank P90≤2000、P95诊断，852原历史字段与512修订逐次<1000ms独立门保留。
- F-R9-02 CLOSED：probe测试程序指定 `--test-threads=1`，incremental另开启原始样本输出；选中/跳过/退出码guard不削弱且自测通过。两份日志各30个样本独立复算：incremental P90/P95=461.9672/487.955ms，baseline 474.8966/505.0909ms；incremental 6/6、baseline各段及resources 18/18通过。
- 上轮深度/F01/F02/F03/858集成结论保留，无新增 finding。本轮未运行整档，acceptance、裸 cargo t、tv 等冻结门及823合入前核对、批量到期判定仍pending。
