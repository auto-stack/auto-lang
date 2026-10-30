# auto-av overview

PLAN-798：有限ABC/验证/AV执行与持久产物探针；不改变旧VM产品入口。

当前粒度、限制和来源以[project](project.md)及所链接合同为准。函数持久产物复用已实现；完整函数级增量已由 828 交付（resident-function-install）。PLAN-820 起本模块含 av.data/1 数据 profile（data-layout / data-place-profile）；PLAN-821 起含 av.owned/1 owned Place profile（place-ownership / owned-access-profile）。PLAN-827 起含实验性标量源码文件验收基线（language-support / source-program-conformance）；PLAN-822 起含 av.collections/1 类型化集合 profile（collections-variants / collection-place-profile）。

[计划来源](plans.md)；[架构决策](architecture.md)。

PLAN-865 在显式[源码闭包域](../auto-ac/source-closures.md)接入现有 av.closures/1 runtime。lambda 捕获分析、函数类型值和间接调用已有源码路径；823 wire 与执行合同不变。共享可变捕获及集合方法 HOF 仍是缺口。

PLAN-824 增加实验性 [av.dynamic/1 类型擦除与 portable owned 树](dynamic-portable-values.md)；保留 typed 入口和源码/native 缺口。PLAN-823 增加实验性 [av.closures/1 类型化闭包环境](closure-environments.md)（move/clone 逃逸捕获、同步借用纪律、增量会话与 824 身份互斥）；源码闭包语法由 PLAN-865 接入，native ABI 缺口保留。PLAN-825 增加 [av.core-i32/1 私有装载执行层](loaded-program-runtime.md)（O(1) PC 查表、函数级驻留索引、共享执行预算；参考路径保留）与 [运行基准方法](runtime-benchmark-method.md)。PLAN-826 增加 [AVAC/1 紧凑 wire 世代](compact-abc-format.md)（同一标量 profile 的第二 wire：canonical ULEB、section 目录、有界解码与双世代隔离路由；参考路径与旧 codec 保留）。

PLAN-863 增加实验性 [av.host-console/1](host-import-and-console.md)：类型化宿主声明、当前授权准入、有界双流输出及 portable snapshot 一致性重验；whole-module probe，不切换产品默认引擎。

PLAN-889增加显式[native集合/变体消费者](../auto-mir/native-collections-variants.md)：
原864合法源码→同一shared MIR→真实COFF/PE，owned全帧清理/物理预算、真实source单体候选/坐标与深验恢复。
AV opcode/wire/执行语义及旧whole-source差额不改，typed/wide与native在线帧不冒领。
