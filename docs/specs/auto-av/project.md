# auto-av

> **Status**: experimental

PLAN-798。实际Rust包：`experimental/ac-av-probe/core` 的auto-av-core与 `probe` 的auto-av-probe，独立workspace/锁；不替代产品旧AutoVM，不改变默认CLI/UI/CTE。

Checked HIR → Typed MIR → ABC decode/verify → 私有LoadedModule → Rust AV。解释器使用显式frame/slot栈和受限dispatch，支持i32/bool、控制流/直接调用和精确trap。

- [ABC与执行](abc-core-i32.md)
- [数据布局与纯值 record（av.data/1，PLAN-820）](data-layout.md)
- [缓存、资源与验收](cache-and-conformance.md)
- [MIR](../auto-mir/core-i32.md) | [数据 MIR](../auto-mir/data-place-profile.md)

798不是在线函数替换/完整AJ/发布器。实际增量边界以缓存合同为准。

PLAN-820 交付独立 data profile `av.data/1`（AVAD wire v1/profile 2，语义
版本 1/1/1 独立轴；core-i32 的 1/2/1 与 AVAB 字节语义不变）：宽标量数据+
纯值 record+字段 Place，合同见 [data-layout](data-layout.md)与
[data-place-profile](../auto-mir/data-place-profile.md)。非目标：宽标量算术/
str/借用/Drop/List/enum/闭包/Dyn/NaN-boxing、生产默认接线、native 新数据
lowering。825 执行优化、826 紧凑 wire 不冒领未交付能力；821–824 后继边界
以各自实际交付为准。


PLAN-827 通过既有 ac-av 消费者建立有界实验性标量源码文件验收基线。
AP-01/AP-22 执行、AP-23 精确 trap；G0 库存与语义差异保持显式。不切换默认
CLI/UI/CTE，不新增通用脚本/stdlib/host IO，不退役旧 VM。源码支持与未来
能力缺口记录于 [language-support](language-support.md) 与
[source-program-conformance](source-program-conformance.md)。
