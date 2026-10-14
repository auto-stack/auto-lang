### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-test-case/project.md`、`profile-v1.md` | 无共享合同 → 本期支持表/schema/bytes/ref/suite/DAG/只读/report | 可复用 current-state | AC-01–05、AC-07 |
| SD-02 | add | `docs/specs/auto-test-case/architecture.md`、`plans.md` | 无裁判来源规范 → pinned 来源/raw locator/adapter 依赖方向与执行限制 | 避免 GUI/自举环 | AC-02/08 |
| SD-03 | modify | `docs/specs/auto-lang/trans/design/test-convention.md` | 仅旧目录 → 双轨只读 pilot、emit 与未来 compile 分开 | 不虚报编译覆盖 | AC-03–06 |
| SD-04 | modify | `docs/specs/auto-lang/vm/overview.md` | 仅旧文件语料 → pilot/阶段/weak/旧 gate 边界 | 对账覆盖 | AC-04/06 |

本次仅提议 delta；review 按最终提交冻结，merge 才写 canonical Spec/ADR/模块 plans、
全局 overview 新 crate 与 GOAL-016 进展，再生 spec 索引。

