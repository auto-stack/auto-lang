# auto-av

> **Status**: experimental

PLAN-798。实际Rust包：`experimental/ac-av-probe/core` 的auto-av-core与 `probe` 的auto-av-probe，独立workspace/锁；不替代产品旧AutoVM，不改变默认CLI/UI/CTE。

Checked HIR → Typed MIR → ABC decode/verify → 私有LoadedModule → Rust AV。解释器使用显式frame/slot栈和受限dispatch，支持i32/bool、控制流/直接调用和精确trap。

- [ABC与执行](abc-core-i32.md)
- [数据布局与纯值 record（av.data/1，PLAN-820）](data-layout.md)
- [owned Place 与清理（av.owned/1，PLAN-821）](place-ownership.md)
- [缓存、资源与验收](cache-and-conformance.md)
- [类型化集合与 Variant（av.collections/1，PLAN-822）](collections-variants.md)
- [类型化闭包环境（av.closures/1，PLAN-823）](closure-environments.md)
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


## PLAN-821：av.owned/1 owned Place profile

PLAN-821 交付独立 owned profile `av.owned/1`（profile id 3，AVAD wire v2，
语义版本 1/1/1）：char/borrowed str/owned Str 与拥有 Str 字段 record 的
Place 拥有型读写、借入 record（RecordView）、mut record 写回全部参数形状
（单 mut/混只读/双 mut/owned Str 返回）与全部 owned 指令（含 FIELDMOVE
逐字段语义）。合同见 [place-ownership](place-ownership.md) 与
[owned-access-profile](../auto-mir/owned-access-profile.md)。**当前无运行期
失效句柄校验、旧帧/跨 store 拒绝或 generation 回绕退役保证**（r6 用户裁定
的静态类型/借用检查边界）。非目标：部分字段 move（源值层面）/完整 NLL/
用户析构/循环共享/GC/List/闭包/Dyn/async/native ABI。


## PLAN-822：av.collections/1 类型化集合 profile

PLAN-822 交付第四 profile `av.collections/1`（AVAD v3/profile id 4，语义
版本 1/1/1）：受控 List、Option/Result/有限名义 Variant 与 Index Place 的
typed 元素存取/转移/清理。合同见
[collections-variants](collections-variants.md) 与
[collection-place-profile](../auto-mir/collection-place-profile.md)。
**非目标**：source 支持面（fixture-only）、Map/Set/迭代器/借用元素容器/
循环共享、record 容器字段、List 嵌 List、native/CTE 扩面。后继增量消费与
授权面见 [resident-function-install](resident-function-install.md)。

## PLAN-823：av.closures/1 闭包环境 profile

PLAN-823 交付 `av.closures/1`（profile id 6、AVAD wire v5、TY_CLOSURE tag 17、
语义版本 1/1/1）：typed owned 闭包环境、move/显式 clone 逃逸捕获、静态可证
非逃逸同步借用、受检间接调用与 env/frame 身份；A23 代码层与新进程恢复。
合同见 [closure-environments](closure-environments.md) 与
[closure-capture-profile](../auto-mir/closure-capture-profile.md)。
**非目标**：源码闭包语法/前端推导、共享 RC/Copy 闭包、async/生成器、跨线程、
native ABI/JIT、持久闭包。与 824 的身份划分（v5/6/17/0x32-0x34 vs
v4/5/16/0x2F-0x31）与交叉拒绝为现行合同，整合前旧登记为历史。

## PLAN-824：av.dynamic/1 动态值 profile

实验 profile `av.dynamic/1`（profile 5、AVAD v4）：受检显式擦除、类型查询/恢复与独立 portable owned 树冻结。实际类型、预算、取消和恢复边界见 [dynamic-portable-values](dynamic-portable-values.md) 与 [MIR profile](../auto-mir/dynamic-cast-profile.md)。不交付通用源码 Dyn、复合 CTE、native/FFI 或闭包冻结。
