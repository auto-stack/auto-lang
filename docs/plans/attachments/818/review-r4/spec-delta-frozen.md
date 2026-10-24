### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/mir-native-scalar.md | 新CheckedMir→CLIF/COFF/PE有限实现、数值/调用/能力/unsupported | 与旧HIR路径分清 | AC-01/02/03 |
| SD-02 | add | docs/specs/auto-ac/mir-native-cache.md | 单元/symbol/key/依赖/恢复/资源/周转及真实链接 | 增量可复验 | AC-04/05/06 |
| SD-03 | modify | docs/specs/auto-ac/project.md、artifact-contract.md | 追加818 consumer/producer面，旧profile/producer不扩 | 不回溯宣布生产默认迁移 | AC-01/04/08 |
| SD-04 | modify | docs/specs/auto-hir/stage-contract.md | 追加共享MIR的native消费门，旧S0–S6事实保留 | 不泄露CLIF到HIR/MIR | AC-02/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 追加正式recipe/worker/actual/stale/观测支持面 | 单一runner | AC-07 |
| SD-06 | modify | docs/specs/auto-av/resident-function-install.md | 继承D828函数IR/代码复用、受检安装/依赖/寿命；新增能力不退回整模块热重编/装载 | 用户明确的顺序与增量机制保留 | AC-11 |

