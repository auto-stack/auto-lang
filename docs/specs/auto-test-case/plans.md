# auto-test-case 相关 Plan 索引

> 状态以各 plan 文件自身为准（`plans/` = docs/plans/ 活跃，`archive/` 归档）。

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
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
