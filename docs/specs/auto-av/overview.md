# auto-av overview

PLAN-798：有限ABC/验证/AV执行与持久产物探针；不改变旧VM产品入口。

当前粒度、限制和来源以[project](project.md)及所链接合同为准。函数持久产物复用已实现；完整函数级增量已由 828 交付（resident-function-install）。PLAN-820 起本模块含 av.data/1 数据 profile（data-layout / data-place-profile）；PLAN-821 起含 av.owned/1 owned Place profile（place-ownership / owned-access-profile）。PLAN-827 起含实验性标量源码文件验收基线（language-support / source-program-conformance）；PLAN-822 起含 av.collections/1 类型化集合 profile（collections-variants / collection-place-profile）。

[计划来源](plans.md)；[架构决策](architecture.md)。

PLAN-824 增加实验性 [av.dynamic/1 类型擦除与 portable owned 树](dynamic-portable-values.md)；保留 typed 入口和源码/native 缺口。PLAN-823 增加实验性 [av.closures/1 类型化闭包环境](closure-environments.md)（move/clone 逃逸捕获、同步借用纪律、增量会话与 824 身份互斥）；源码闭包/native ABI 缺口保留。
