# auto-test-case 相关 Plan 索引

> 状态以各 plan 文件自身为准（`plans/` = docs/plans/ 活跃，`archive/` 归档）。

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 773 | autodown-testcase-protocol-pilot | ✅（delivered） | [archive/773-autodown-testcase-protocol-pilot.md](../../plans/archive/773-autodown-testcase-protocol-pilot.md) | TestCase 协议 A 阶段落地：核心 crate + adapters + auto-case CLI + 21 case 引用式 pilot；R1 复审 11 阻断全闭合（admission/路径域/类型化阶段/产物交接/stale），R2 pass |
| 788 | autodown-ac-native-turnaround | ✅（delivered，r10；R1..R9 修复链清偿后 final pass） | [archive/788-autodown-ac-native-turnaround.md](../../plans/archive/788-autodown-ac-native-turnaround.md) | B 阶段 v2 native：ac check/build/run（真实 COFF/PE/执行、typed 阶段、部分产物边界、783 cache 三模式、逐行身份适用集合）+ syntax parse/edit（787 runtime 绑定）；ac-case-worker 独立 workspace + 父侧全量判定（I/O 生命周期/解码后流 cap/即时 cap 终止）；42 case 语料+账本；周转六场景硬门（hot/body-edit/syntax P95<1s）与 50 修订并发流（双车道/真实错序全拒）；验收器七族+档位必验集合绑定 |

## 前史

- 历史交付（768 编号时期的 vendor + 核心实现）经 Plan 773 R1 复审追溯确认已合入
  master（677e84771/129c4474f 祖先）；773 本体含 R1 修复轮与永久控制。
- Design 35（docs/design/35-autodown-test-cases.md）定义 A/B/C 阶段边界：B =
  真编译消费/deadline/进程隔离，C = 编辑矩阵——均未在本模块启动。
