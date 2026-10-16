# auto-gen-core — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 791 | generation-core-extraction（建 crate） | ✅（reviewed→archived） | archive/ | 本 crate 随 PLAN-791 建立：无解释器生成核心（依赖仅 auto-val+regex+thiserror，闭包零 auto-lang/auto-man/auto-vm，由 test/fixtures/modularity/gen-core-consumer 外部探针 + verify-deps.sh 钉住并登记 module-boundaries man-ecosystem 域）——error/guard 逐字迁移 + feed_guarded_legacy 单一定义（legacy 字节语义：join 无尾换行/精确 marker 前缀/缺文件透传）；generate 编排（默认命名剥一尾 .at、guard 保留/overwrite、dry_run 零文件系统效应、失败模板永不覆写既有输出、单模板错收集批次继续）；12 单测 + 阶段边界/字节语义冻结五条入 project.md |
