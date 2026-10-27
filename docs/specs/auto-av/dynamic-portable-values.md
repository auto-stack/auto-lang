# 动态值与可移植冻结（av.dynamic/1）

> 实验性 typed profile；来源 PLAN-824。实现 `experimental/ac-av-probe/core/src/{dynamic,portable,av}.rs`，不改变旧 AutoVM、默认 CLI/UI 或 CTE。

## 类型擦除与拥有关系

`av.dynamic/1` 的 MIR、ABC、执行和缓存分别使用独立身份：profile id 5、AVAD v4、语义版本 1/1/1、producer `plan824/av-dynamic`。旧 AVAD 代际互不当作新代际解码。`DYN_PACK`/`DYN_IS`/`DYN_TAKE` opcode 为 `0x2F`/`0x30`/`0x31`。

显式 pack 的递归白名单为 bool、精确数值、char、owned Str、纯 owned record、List 和有限名义 Variant（包括 Option/Result）。Fn、借用、Place、外部资源、闭包及嵌套 Dyn 不进入该白名单；合法外层容器不能掩盖非法内层。record/variant 依 owner、名称及声明字段/分支和类型保持名义身份；List 按元素结构等价。错型 `take` 是执行期 checked trap，源值保持可用；只读借用不赋予释放权。复制须显式执行并按展开后工作量计费。

## Portable 边界

`auto.portable.av-dynamic/1`（AVPF v1）与代码缓存分域。冻结仅允许 owned 唯一入边的树：共享子对象与环均拒绝；不持久化地址、执行类型索引、frame/borrow token 或 trusted 凭证。保存稳定类型、owner、字段/分支身份、精确数值位型及 UTF-8；恢复时校验结构、类型和当前依赖，重建域内值，不把输入要求当作宿主授权。即使值树为空或分支不活跃，根类型内所有名义依赖仍须重验；同名字段或分支类型变化受控拒绝。标量载荷与其描述符 kind 必须一致，正确校验和不能洗白坏结构。

冻结/恢复失败或取消不消耗原值或输入；部分重建对象由拥有关系回收。数据、步骤、节点和深度受现有预算限制：冻结/恢复/复制的数据线 8MiB、节点 65536、步数 2000000、类型/值深度 256；执行态的 Dyn 数据与槽及聚合共享执行分区，不单独领取额度。类型 DAG 的展开、字段/分支与名称、binding 构造、portable 描述符及复制在分配前计量；类型遍历、准入和比较参与工作和每 1024 步取消检查。恢复保留根类型并逐节点核对子描述符，避免按深度反复保留完整类型。逻辑峰值不是进程 RSS；worker 的实际 private-bytes 由父侧按 798 B 线采样。

支持面限 typed fixture / MIR→ABC→AV 与现有新进程恢复入口；不交付通用源码 Dyn API、native ABI/FFI、复合 CTE、通用反射、资源/闭包冻结、GC 或迁移活值。验证入口与 typed/source 区分见 [ACE M1](../auto-test-case/ace-m1-conformance.md)。
