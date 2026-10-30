# auto-mir architecture

| 日期 | 来源 | 决策 | 结果与边界 |
|---|---|---|---|
| 2026-10-20 | PLAN-798 | Typed MIR/portable ABC与私有授信执行分层；持久恢复不恢复trusted，实际工作量如实计量 | 有限标量实验合同；当前函数cache不等于全链路增量，后继828建设驻留函数查询/代码安装机制 |

## 2026-10-30 / PLAN-889：共享集合语义与原生资源消费

决策：复用同一source/HIR/共享MIR与候选协议，显式collections profile进入原生Cell/容器原语；
不用AV解释旁路或旧HIR→CLIF。冷态有界scaffold批处理，body编辑保真实producer身份；恢复不持久化trust。
备选整模块重算会超物理峰且冒充carry，扩大typed入口会混淆支持域，因此均不选。
后果：14原L/V格按原源关闭，全帧清理/预算/身份与失败原子性成为此profile合同；
旧AV source前端差额、宽typed入口/在线native帧保持各自边界。状态：accepted，见
[native collections](../auto-mir/native-collections-variants.md)。
