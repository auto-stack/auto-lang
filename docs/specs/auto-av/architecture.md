# auto-av architecture

| 日期 | 来源 | 决策 | 结果与边界 |
|---|---|---|---|
| 2026-10-20 | PLAN-798 | Typed MIR/portable ABC与私有授信执行分层；持久恢复不恢复trusted，实际工作量如实计量 | 有限标量实验合同；当前函数cache不等于全链路增量，后继828建设驻留函数查询/代码安装机制 |
| 2026-10-30 | PLAN-863 | host import 与 guest import 分表、typed capability 准入先于执行；snapshot 对所有函数 sidecar 实核 | av.host-console/1 whole-module；provider 为可信原生调用，无内存隔离承诺；不恢复授权、不支持 console resident units |

## 2026-10-30 / PLAN-889：共享集合语义与原生资源消费

决策：复用同一source/HIR/共享MIR与候选协议，显式collections profile进入原生Cell/容器原语；
不用AV解释旁路或旧HIR→CLIF。冷态有界scaffold批处理，body编辑保真实producer身份；恢复不持久化trust。
备选整模块重算会超物理峰且冒充carry，扩大typed入口会混淆支持域，因此均不选。
后果：14原L/V格按原源关闭，全帧清理/预算/身份与失败原子性成为此profile合同；
旧AV source前端差额、宽typed入口/在线native帧保持各自边界。状态：accepted，见
[native collections](../auto-mir/native-collections-variants.md)。

## 2026-11-18 / PLAN-1026：可选 CPython 适配器与 py 通道

决策：ACE 普通源码经 `use.py` 前门、共享受检 HIR/MIR 与唯一 lowering 进入 AV 的
foreign stub，再由 967 自动生成的 C 兼容 shim 调用**可选** Rust 适配器（内部 PyO3 →
CPython C API）。`av-core`/共享 HIR/MIR/`foreign-contract` 不引入 PyO3 依赖；适配器
不依赖 AV 解释器或 964 具体动态实现。Python 外部值复用 964 IF-VAL 的外部值形态
（AV 侧 i32 token，provider 私有 registry 持真实对象），资源 owner/回收复用 967
IF-RES 唯一账本，异常映射到 965 错误体系；公共输出走 947/972 的 console 面。
成员/可调用性运行期按对象协议判断，不静态枚举 PyTorch 成员、不按库名特判。

后果：`FOREIGN_CALL`/`FOREIGN_CALL_CB` 编解码族门由 `ForeignScalar|Owned` 扩为
`ForeignScalar|Owned|Closures`（opcode/payload/wire 版本不变，故不新增 ABC 世代）；
lowering 契约版本升为 `"5"`（console host-import 准入域扩面 + py 重建 intrinsic
调用点发射 + 该域 F32/F64 interning，见
[hir-lowering-boundary](../auto-mir/hir-lowering-boundary.md)）；console import 在
非 profile-8 的 foreign-carrying 模块内准入（见
[host-import-and-console](host-import-and-console.md)）。详细运行合同见
[python-interop](python-interop.md)。边界：AV-only 切片；新 A2PY 与其他后端的
Python 运行适配未实现，不因此宣布 967 四端或 M2 退役全量通过。状态：accepted。
