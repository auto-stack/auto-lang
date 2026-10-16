# 801–803 合同预检

2026-10-17；stage: new。初次取材`143aff0a258fdca8bda9a3b8068dc8a9cced623c`，规划期间复核至`2d246b4e86c1ebf5138f6a316c1896e56a5d35f8`：800归档8/8、799执行中。本记录是规划检查，不是实现验收或独立final review。

| Plan / revision | outcome | next | 检查结论 |
|---|---|---|---|
| 801 / 1 | blocked | 799交付后work T-01 | 799已开始执行、尚未交付；完整合同已起草，AST前置不能以复制/大包反向依赖替代 |
| 802 / 1 | pass | 合同确认后work | 22项边界/缺席差异/delta/所有权/真UI及装配有owner和可区分oracle |
| 803 / 1 | pass | 合同确认后work | 三组资源/纯路径计划和生产adapter分离；不改旧模板/失败行为；真实copy/exporter和独立打包有验证owner |

三份各8任务、6条AC，work/review/merge明确分属；所有必需实现行为在work阶段验收，整档门归最终review之后的merge。Spec增量为提案，未发布现行规范。T01有界决策只选择等价机制，不可擅减目标/错误阈值/兼容承诺。

共享窗口、800交付兼容、799→801硬依赖、792实际闭包更新责任见[编排](../../reports/extraction-next-parallel-order-20261017.md)。没有创建agent/实现树、运行Cargo/docs_gen、变更旧仓或他线WIP。

`preflight.py`检查编号唯一性、章节/AC/任务/owner计数、前置实际状态、相对链接、取材SHA和本轮文档差异空白。结果见[机器预检](preflight.json)。输入SHA为本轮观察指纹；并发进度变化须重新核对，不能把历史快照作永久放行证据。
