# package-lock — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 795 | package-lock-core-extraction（建 crate） | ✅（reviewed→archived） | archive/ | 本 crate 随 PLAN-795 建立：pac.lock 模型/codec/校验核心（依赖仅 auto-val+serde/toml+std，闭包零 auto-lang/auto-man/auto-gen/auto-vm，由 test/fixtures/modularity/package-lock-consumer 外部探针 + verify-deps.sh 钉住并登记 module-boundaries man-ecosystem 域）——排序/查找/verify/编解码单一定义（LockEntry/LockFile serde 形状逐字段同历史、LockedEntryView 零 DTO 视图、ResolvedDependency Target-free 投影）；579f6dec0 探针冻结 golden 三面钉死（TOML 反斜杠字面量形态/JSON 无尾换行/错误阶段与文本/重名首条）；此 pac.lock 与 792 Cargo.lock 为不同合同；冻结语义五条入 project.md |
