# auto-av plans

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 827 | AV真实源码程序验收基线与首个文件执行闭环 | ✅（delivered，R10 最终 PASS） | [计划](../../plans/archive/827-av-source-program-baseline.md) | 实验性标量源码文件验收基线：AP-01/22 typed 0、AP-23 精确 trap，G0 库存/SEM 差异、源身份/授权/缓存反控、可失败帧槽位分配与清理、bench 证据留存/绑定（0ms 阈值反控+逐字调用文本）；[language-support.md](language-support.md) / [source-program-conformance.md](source-program-conformance.md) |
| 818 | ac-shared-mir-native-parity（AV 面） | ✅（delivered） | [计划](../../plans/archive/818-ac-shared-mir-native-parity.md) | mn 消费者继承 828 函数 IR/代码复用（未改不重编/内容命中历史复用/每编辑恰一单元重发，新增能力不退回整模块热重编）；[resident-function-install.md](resident-function-install.md) 818 节 |
| 798 | Typed MIR与ABC/AV首个全路径探针 | archived | [计划](../../plans/archive/798-typed-mir-abc-av-probe.md) | 有限标量执行/信任/函数持久产物，准确记录整模块重算及828未交付边界 |
| 828 | av-function-incremental-session | ✅（delivered，T-10 最终 PASS） | [计划](../../plans/archive/828-av-function-incremental-session.md) | 常驻函数安装完整切片：单元派发（无 merged 载荷）+事务快照与寿命+执行器内派发身份观测+512/GC/存储水位+正式双门战役与双线预算；[resident-function-install.md](resident-function-install.md) |
| 817 | AV驱动的CTE查询与冻结缓存闭环 | ✅ delivered | [计划](../../plans/archive/817-av-cte-query-frozen-cache.md) | 828 session 消费：exec snapshot 车道（缓存身份轴/增量纯度双观测/退化门） |
