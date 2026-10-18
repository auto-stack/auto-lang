# auto-ui-dialect — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 806 | ui-declarations-dialect-extraction（建 crate） | ✅（reviewed→archived） | archive/ | 本 crate 随 PLAN-806 建立：UI msg/model 声明文法（含字段注解）单一份实现，完整算法自 auto-lang Parser 逐字迁出（行为冻结，未收紧/未扩展），经 `dyn ParseContext` 端口驱动、expr/type 委托核心文法，依赖闭包仅 auto-parser-api 轻层（777 lang-core 域，增量恰 2 合法声明边）；named-payload 前瞻用端口原语 `probe_next_kind`（F806-R2-01 引入/R3-01 定形：成功=零副作用/幂等，失败=旧回推前瞻失败消费协议——错误丢弃/可能推进/不承诺幂等，与旧 lexer.next+push_token 逐字同构）；`ModelGrammar::{Builtin,ExternalSubset}` 单份策略承载默认文法（无裸字段）与 778 冻结外部子集（裸字段显式 pilot 专用），dialect-api-consumer 不保第二份字段算法；生产全路径（顶层 UiDialect 派发/widget/store/component 结构直连/import with_session 默认装配）真实委托，Parser pub 方法薄兼容委托、原算法净删 213 行；ui-declarations-consumer 轻消费方+verify-deps 钉住零 auto-lang 闭包；792 交付闭包 12→13 成员经正式 driver 25/25（R2 轮 fresh/locked/十四态负控，R4 轮完整复验） |
