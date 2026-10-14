# auto-test-case 相关 Plan 索引

> 状态以各 plan 文件自身为准（`plans/` = docs/plans/ 活跃，`archive/` 归档）。

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 773 | autodown-testcase-protocol-pilot | reviewed（merge 中） | plans/ | TestCase 协议 A 阶段落地：核心 crate + adapters + auto-case CLI + 21 case 引用式 pilot；R1 复审 11 阻断全闭合（admission/路径域/类型化阶段/产物交接/stale），R2 pass |

## 前史

- 历史交付（768 编号时期的 vendor + 核心实现）经 Plan 773 R1 复审追溯确认已合入
  master（677e84771/129c4474f 祖先）；773 本体含 R1 修复轮与永久控制。
- Design 35（docs/design/35-autodown-test-cases.md）定义 A/B/C 阶段边界：B =
  真编译消费/deadline/进程隔离，C = 编辑矩阵——均未在本模块启动。
