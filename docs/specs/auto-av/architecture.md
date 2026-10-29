# auto-av architecture

| 日期 | 来源 | 决策 | 结果与边界 |
|---|---|---|---|
| 2026-10-20 | PLAN-798 | Typed MIR/portable ABC与私有授信执行分层；持久恢复不恢复trusted，实际工作量如实计量 | 有限标量实验合同；当前函数cache不等于全链路增量，后继828建设驻留函数查询/代码安装机制 |
| 2026-10-30 | PLAN-863 | host import 与 guest import 分表、typed capability 准入先于执行；snapshot 对所有函数 sidecar 实核 | av.host-console/1 whole-module；provider 为可信原生调用，无内存隔离承诺；不恢复授权、不支持 console resident units |
