### 规范增量

下表target相对 `docs/specs/`；D798后T-01核实际add/modify。canonical/ledger仅merge沉淀，当前不写已实现状态。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | auto-ac/function-incremental-source.md | 整Program前端→受检接口环境+函数体查询、来源和失效轴；函数HIR实际A07产物/恢复及统一引用 | 完整前端增量 | AC-01/02/05 |
| SD-02 | modify | auto-hir/stage-contract.md、auto-syntax/snapshot-cst.md | 完整模块入口保留→私有函数凭证和真实CST消费；支持不减 | 不复制前端/削弱验证 | AC-02/05 |
| SD-03 | add | auto-mir/function-incremental.md | whole lower/split→函数lower/verify和稳定跨单元引用 | 中段真实增量 | AC-03/05 |
| SD-04 | add | auto-av/resident-function-install.md | 完整LoadedModule→函数代码/受检环境/事务snapshot及寿命 | 装载执行增量 | AC-04/05/06 |
| SD-05 | modify | auto-av/cache-and-conformance.md | 磁盘命中→内存复用/恢复与全链路结构/P95/资源证据 | 不能只看emit/cache计数 | AC-01/06/07/08 |
| SD-06 | modify | auto-test-case/architecture.md | one-shot保留→正式持续会话ready/edit/result准入及失败门 | 真实端到端 | AC-07/08 |
