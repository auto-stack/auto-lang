# PLAN-788 r1 proposed Spec delta (review snapshot)


以下拟于merge沉淀，不在规划时编辑canonical Spec。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/profile-v2-native.md | v1无native→明确v2支持表/阶段/产物/bytes/预算/cache/trace/性能 | 不静默扩v1 | AC-01–06 |
| SD-02 | modify | docs/specs/auto-test-case/profile-v1.md | 保留v1合同，增加v2导航/版本拒绝边界 | 兼容可复验 | AC-01、07 |
| SD-03 | modify | docs/specs/auto-test-case/architecture.md | A阶段→B阶段worker/单向依赖/typed IPC/containment与比较分工 | 执行可信 | AC-02、03、07 |
| SD-04 | modify | docs/specs/auto-test-case/project.md | 增补已实现v2/native/syntax支持、profile/平台边界与入口 | 事实导航 | AC-01–07 |
| SD-05 | modify | docs/specs/auto-ac/project.md | 补AutoDown入口/导入范围，旧语言面/CLI门禁不变 | 不冒称完整AC | AC-02、07 |
| SD-06 | modify | docs/specs/auto-ac/artifact-probe.md | 补真实consumer/观察扩展、依赖身份与预算继承，非生产cache | 统一产物接线 | AC-04、06 |

