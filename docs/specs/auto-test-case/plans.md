# auto-test-case 相关 Plan 索引

> 状态以各 plan 文件自身为准（`plans/` = docs/plans/ 活跃，`archive/` 归档）。

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 865 | AV闭包源码语法与间接调用 | reviewed；merge gates pending | [计划](../../plans/archive/865-av-source-closures-and-hof.md) | 追加 K01–K06 源码 suite，recipe ac-av/closures-source-run-v1；legacy 差异与 base suite 不变；[ace-m1-conformance.md](ace-m1-conformance.md) |
| 858 | ACE 开发期周转口径与 feature 边界 | 已复审（r4） | [858](../../plans/archive/858-autographics-m2-game-samples-and-api-gaps.md) | 当前 ACE 周转门 P90≤2000ms、真实样本和非时延约束保持；历史 P95 数据不改写，长期目标待架构成形后实测。 |
| 824 | AV动态Variant与可移植冻结 | ✅ delivered（R3 最终 PASS） | [计划](../../plans/archive/824-av-dynamic-values-freeze.md) | 动态 recipe、父侧 oracle 与 typed M1 绑定；[architecture.md](architecture.md) |
| 821 | av-owned recipe（ac-av/owned-run-v1） | ✅（delivered，R6 最终 PASS） | [计划](../../plans/archive/821-av-place-owned-access.md) | owned recipe 三缝注册+sabotage 双面+故障三态+corpus 借入/mut 写回正控；见 [architecture.md](architecture.md) PLAN-821 节 |
| 847 | 恢复 probe 可选依赖测试门控 | ✅ delivered；全局新红归850 | [计划](../../plans/archive/847-restore-probe-test-feature-boundary.md) | 旧probe与MIR-native各自受test-cases门控，optional/default-off保持；2177完整ID恒等、独立审查及裸t3181通过，主tb3109通过；tf跨workspace与tt执行状态另归850，未推进全局绿色锚点 |
| 827 | 源码程序 corpus/验证消费者（ac-av/core-run-v1 复用） | ✅（delivered） | [archive/827-av-source-program-baseline.md](../../plans/archive/827-av-source-program-baseline.md) | 源文件 corpus av-source-827 复用既有 recipe/worker/父侧判定，worker 只收字节永不读期望；见 [architecture.md](architecture.md) PLAN-827 节 |
| 818 | ac-shared-mir-native-parity（runner 面） | ✅（delivered，R1–R4 四轮复审链收敛） | [archive/818-ac-shared-mir-native-parity.md](../../plans/archive/818-ac-shared-mir-native-parity.md) | mn recipe/rounds 协议（常驻 worker×逐步一问一答/失败活跃性与限额边界夹具化）+有效计时语义（父侧观测连续窗口含独立期望比较，观察者缝期望不入 worker）；[architecture.md](architecture.md) 818 节 |
| 773 | autodown-testcase-protocol-pilot | ✅（delivered） | [archive/773-autodown-testcase-protocol-pilot.md](../../plans/archive/773-autodown-testcase-protocol-pilot.md) | TestCase 协议 A 阶段落地：核心 crate + adapters + auto-case CLI + 21 case 引用式 pilot；R1 复审 11 阻断全闭合（admission/路径域/类型化阶段/产物交接/stale），R2 pass |
| 788 | autodown-ac-native-turnaround | ✅（delivered，r10；R1..R9 修复链清偿后 final pass） | [archive/788-autodown-ac-native-turnaround.md](../../plans/archive/788-autodown-ac-native-turnaround.md) | B 阶段 v2 native：ac check/build/run（真实 COFF/PE/执行、typed 阶段、部分产物边界、783 cache 三模式、逐行身份适用集合）+ syntax parse/edit（787 runtime 绑定）；ac-case-worker 独立 workspace + 父侧全量判定（I/O 生命周期/解码后流 cap/即时 cap 终止）；42 case 语料+账本；周转六场景硬门（hot/body-edit/syntax P95<1s）与 50 修订并发流（双车道/真实错序全拒）；验收器七族+档位必验集合绑定 |

## 前史

- 历史交付（768 编号时期的 vendor + 核心实现）经 Plan 773 R1 复审追溯确认已合入
  master（677e84771/129c4474f 祖先）；773 本体含 R1 修复轮与永久控制。
- Design 35（docs/design/35-autodown-test-cases.md）定义 A/B/C 阶段边界：B =
  真编译消费/deadline/进程隔离，C = 编辑矩阵——均未在本模块启动。
| 828 | av-function-incremental-session | ✅（delivered，T-10 最终 PASS） | [计划](../../plans/archive/828-av-function-incremental-session.md) | 正式持续会话入口（session-serve worker+父侧 judge 测量窗/三门/证据先落盘）；one-shot recipe 保留 |
| 817 | AV驱动的CTE查询与冻结缓存闭环 | ✅ delivered | [计划](../../plans/archive/817-av-cte-query-frozen-cache.md) | cte/scalar-query-v1 recipe + cte-worker stale 轴 + outcome admission（隔离态=ERROR） |
| 822 | AV类型化集合与Variant载荷访问 | ✅（delivered，R19 最终 PASS） | [计划](../../plans/archive/822-av-typed-collections-variants.md) | ac-av/collections-run-v1 recipe+av-data-822 语料+父侧冻结期望表与 sabotage/故障反控；[architecture.md](architecture.md) |
| 862 | AV源码入口、Unit与尾值语义 | 已复审（reviewed，T-07 R2 pass） | [计划](../../plans/archive/862-av-source-entry-unit-and-tail-semantics.md) | 追加 U01–U10 与 av_channel 判定轴；24 probe + 27 wait 逐格实测定级（probe/wait 0，clean 20/73）；五态词表 + NA-4；见 [ace-m1-conformance.md](ace-m1-conformance.md) PLAN-862 节 |
| 887 | 测试治理与ACE首批消费（守恒/提速/补覆盖） | ✅ delivered（R final PASS，r2） | [计划](../../plans/archive/887-test-governance-and-ace-first-corpus.md) | 盘点闭包+子case身份+迁移/退役守恒（runner 844+869、floor 3189）、first-wave 30 复用+3 新增 D 消费、八类长期控制（V-04..V-08）、覆盖矩阵与弱判差额、性能负结果记录（worker store 机制轴）；见 [governance-and-inventory.md](governance-and-inventory.md) |
