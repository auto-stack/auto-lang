# auto-parser-api — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 801 | dialect-api-dependency-inversion（建 crate） | ✅（reviewed→archived） | archive/ | 本 crate 随 PLAN-801 建立：方言端口合同单一实现（轻 CompilerSession/Scenario/DialectPriority + ExternalDialect + 对象安全 ParseContext + 快照凭据 ParseCheckpoint），依赖闭包仅 auto-ast/auto-lex/auto-diagnostics（lang-core 域），dialect-api-consumer 探针+verify-deps 钉住外部文法实现零语言实现依赖；auto-lang 侧 ParserBridge 实现端口、ExternalDialectAdapter 进同一三段派发表（register_external_dialogue 注册，778 恢复不变量零漂移）、session/DialectPriority 薄门面类型身份不变；快照寿命按签发作用域失效（F-801-R1-01：槽位记 scope+port_release_scope，弃长度水位——tombstone 复用可绕过表长截断，12 例 Ident/token×Some/None/Err 永久回归 r1_scope_expiry 钉住）；792 交付闭包 11→12 成员经正式 driver 25/25（fresh 锁真构建/重定位 locked/十四态负控） |
