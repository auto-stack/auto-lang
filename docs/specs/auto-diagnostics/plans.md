# auto-diagnostics — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 793 | diagnostics-foundation-extraction（建 crate） | ✅（reviewed→archived） | archive/ | 本 crate 随 PLAN-793 建立：AutoError 聚合与各族诊断（语法/类型/名称/运行时/comptime/警告）+ WithSource 包装 + attach_source/format_error/find_best_match 单一实现，自 auto-lang/src/error.rs 逐字迁入（唯三补丁：Name→AutoStr×2+头注；TypeError::CannotModifyViewParam 载荷=AutoStr 同一类型，error→AST 边切断）；依赖仅 auto-lex/auto-val/miette/thiserror（lang-core 域），diagnostics-consumer 外部探针 + verify-deps 钉住闭包零 auto-lang/Man/VM/UI；V02 冻结 golden 38 例/12 族（SHA256 e97b9450…）永久重放 + interop-consumer 同一类型互传/真实 parse/type/name/route 阶段保持门；auto-lang error.rs 为 20 名薄门面，owner 侧 From（route/discovery、infer/unification）合法留存 |
