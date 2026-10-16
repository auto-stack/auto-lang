# auto-gen — plans

> 纯表格：`| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |`（scripts/spec-index.py 可解析）

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 791 | generation-core-extraction（auto-gen 侧） | ✅（reviewed→archived） | archive/ | 生成核心拆分落位：本 crate 变 Auto 数据/可执行模板 adapter + 兼容门面——CodeGenerator 真实委托 auto_gen_core::generate（TemplateRenderer 边界交付 owned 渲染文本），error/guard/GenReport/TemplateSpec 单一定义移 core 经 re-export 保旧公开路径；legacy AutoGen 模板执行留 adapter 自有解释器、guard feed 改调 core::feed_guarded_legacy（字节语义不变：join 无尾换行）；V02 金样 14 + V03 失败语义 8 零 rebless 全绿冻结旧臂（AutoFile/lib_files 作用域错配、fstr_note 共享解释器路径无效、无 guard 输出丢尾换行均原样保留为债）；净删 generator.rs 编排臂与 lib.rs 重复 guard 算法，regex/thiserror 依赖移除 |
