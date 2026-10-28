# auto-av plans

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 826 | AV紧凑ABC格式与字节码体积 | ✅（delivered，T-07 最终 PASS） | [计划](../../plans/archive/826-av-compact-abc-format-and-size.md) | av.core-i32/1 第二 wire 世代 AVAC/1：canonical ULEB/section 目录/有界解码、AVAB 双世代隔离与 A09 abc-codec 键、825 LoadedProgram 同深准入、828 WireCodec 增量会话、双 wire 体积/装载/V8·Wasm 四引擎对标（ratio 0.361–0.856）；[compact-abc-format.md](compact-abc-format.md) |
| 858 | ACE 开发期周转口径与 feature 边界 | 已复审（r4） | [858](../../plans/archive/858-autographics-m2-game-samples-and-api-gaps.md) | 当前 ACE 周转门 P90≤2000ms、真实样本和非时延约束保持；历史 P95 数据不改写，长期目标待架构成形后实测。 |
| 824 | AV动态Variant与可移植冻结 | ✅ delivered（R3 最终 PASS） | [计划](../../plans/archive/824-av-dynamic-values-freeze.md) | av.dynamic/1 的受检类型擦除、portable 树及类型资源/取消边界；[dynamic-portable-values.md](dynamic-portable-values.md) |
| 821 | AV Place拥有型读写与全出口清理 | ✅（delivered，R6 最终 PASS） | [计划](../../plans/archive/821-av-place-owned-access.md) | av.owned/1 owned profile：FIELDMOVE 逐字段消费（前缀重叠/重初始化重置）+mut 写回全形状+四线资源量纲+D828 单函数常驻前端双通道会话（未改单元前端生产=0、坐标随发布平移、通道感知退化门）；[place-ownership.md](place-ownership.md) |
| 827 | AV真实源码程序验收基线与首个文件执行闭环 | ✅（delivered，R10 最终 PASS） | [计划](../../plans/archive/827-av-source-program-baseline.md) | 实验性标量源码文件验收基线：AP-01/22 typed 0、AP-23 精确 trap，G0 库存/SEM 差异、源身份/授权/缓存反控、可失败帧槽位分配与清理、bench 证据留存/绑定（0ms 阈值反控+逐字调用文本）；[language-support.md](language-support.md) / [source-program-conformance.md](source-program-conformance.md) |
| 818 | ac-shared-mir-native-parity（AV 面） | ✅（delivered） | [计划](../../plans/archive/818-ac-shared-mir-native-parity.md) | mn 消费者继承 828 函数 IR/代码复用（未改不重编/内容命中历史复用/每编辑恰一单元重发，新增能力不退回整模块热重编）；[resident-function-install.md](resident-function-install.md) 818 节 |
| 798 | Typed MIR与ABC/AV首个全路径探针 | archived | [计划](../../plans/archive/798-typed-mir-abc-av-probe.md) | 有限标量执行/信任/函数持久产物，准确记录整模块重算及828未交付边界 |
| 828 | av-function-incremental-session | ✅（delivered，T-10 最终 PASS） | [计划](../../plans/archive/828-av-function-incremental-session.md) | 常驻函数安装完整切片：单元派发（无 merged 载荷）+事务快照与寿命+执行器内派发身份观测+512/GC/存储水位+正式双门战役与双线预算；[resident-function-install.md](resident-function-install.md) |
| 817 | AV驱动的CTE查询与冻结缓存闭环 | ✅ delivered | [计划](../../plans/archive/817-av-cte-query-frozen-cache.md) | 828 session 消费：exec snapshot 车道（缓存身份轴/增量纯度双观测/退化门） |
| 823 | AV类型化闭包环境与捕获生命周期 | ✅（delivered，R3 最终 PASS） | [计划](../../plans/archive/823-av-owned-closure-captures.md) | av.closures/1 第五 profile（AVAD v5/6/tag17/0x32-34）：typed owned 环境、move/clone 逃逸捕获+静态非逃逸同步借用、prepare/commit 两相捕获、ClosuresUpdateSession 增量与 env 契约、A23 恢复身份绑定、与 824 的身份划分和交叉拒绝；[closure-environments.md](closure-environments.md) |
| 822 | AV类型化集合与Variant载荷访问 | ✅（delivered，R19 最终 PASS） | [计划](../../plans/archive/822-av-typed-collections-variants.md) | av.collections/1 第四 profile：18 指令 typed 集合/Variant/Index Place、数据量计费克隆、显式清理全链（含部分回滚）、恢复身份四绑定（profile/types-fp/entry/snapshot）、collections 会话完整候选授权预检；[collections-variants.md](collections-variants.md) |
| 862 | AV源码入口、Unit与尾值语义 | 已复审（reviewed，T-07 R2 pass） | [计划](../../plans/archive/862-av-source-entry-unit-and-tail-semantics.md) | av.entry-unit/1 第七执行 profile：Unit 真值/entry kind 双向校验/AVAU wire/恢复准入与独立 cache 命名空间；[entry-and-unit-profile.md](entry-and-unit-profile.md) |
