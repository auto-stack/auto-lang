# auto-parser-api — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 801 | dialect-api-dependency-inversion（建 crate） | ✅（reviewed→archived） | archive/ | 本 crate 随 PLAN-801 建立：方言端口合同单一实现（轻 CompilerSession/Scenario/DialectPriority + ExternalDialect + 对象安全 ParseContext + 快照凭据 ParseCheckpoint），依赖闭包仅 auto-ast/auto-lex/auto-diagnostics（lang-core 域），dialect-api-consumer 探针+verify-deps 钉住外部文法实现零语言实现依赖；auto-lang 侧 ParserBridge 实现端口、ExternalDialectAdapter 进同一三段派发表（register_external_dialogue 注册，778 恢复不变量零漂移）、session/DialectPriority 薄门面类型身份不变；快照寿命按签发作用域失效（F-801-R1-01：槽位记 scope+port_release_scope，弃长度水位——tombstone 复用可绕过表长截断，12 例 Ident/token×Some/None/Err 永久回归 r1_scope_expiry 钉住）；792 交付闭包 11→12 成员经正式 driver 25/25（fresh 锁真构建/重定位 locked/十四态负控） |
| 806 | ui-declarations-dialect-extraction（端口扩展） | ✅（reviewed→archived） | archive/ | PLAN-806 扩端口新增具名通用能力 `ParseContext::probe_next_kind`（原始单 token 探测：成功路径零副作用/幂等/与首次 advance 一致；失败路径遵循旧回推前瞻失败消费协议——是否推进输入由 Lexer 失败模式决定、错误丢弃不记录、不承诺幂等；auto-lang 侧 `port_probe_next_kind`=旧 lexer.next+push_token 逐字同构，消费方为 auto-ui-dialect msg named-payload 判定，成功/失败双路径均有永久回归）；dialect-api-consumer 自本轮起增 auto-ui-dialect 轻依赖（model 子集方言包装在此、字段算法单一份已委托，不保第二份）；792 交付闭包经 auto-ui-dialect 增至 13 成员 |
