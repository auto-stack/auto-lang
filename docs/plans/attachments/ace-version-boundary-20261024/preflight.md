# ACE/AACE版本边界文档预检

2026-10-24；outcome:pass（规划静态核对，非实现验收）。

用户明确提高v0.6 Rust版ACE成熟度，先复刻现有AutoVM/A2X全部示例，再于v0.7实施Auto版AACE。本轮将该决定写入总Roadmap、v0.6、远期池、ACE主战略、Design43、主体合同、源码迁移路线、能力矩阵和HIR战略；新增[统一版本合同](../../reports/ace-milestones-version-boundary-2026-10-24.md)供后续Plan引用。

输入提交及修订前后SHA256见[inputs.json](inputs.json)，静态检查见[checks.json](checks.json)。开始时这些规划文档已有其他改动，本轮逐文件读取后局部修订，保留既有正文和来源；未修改现行Spec/ledger、实施Plan任务/进度/预算、Rust代码、旧默认入口或其他仓。

核对重点：

- v0.6为Rust版ACE的M1–M4；M1不以827单项完成代替整批交付，M2/M3交叉推进、基础FFI提前，M4逐入口迁移并收口。
- 全示例按源码/依赖/资源/后端/平台/feature固定分母和强oracle；不剪裁、改expected、skip或旧引擎fallback补绿，废弃/破坏性变化显式裁定。
- AACE明确为v0.7目标，不是无版次候选；实施前先满足Rust版成熟度门，不以编译器源码小子集提前并行AAC/AAV。具体主体/宿主桥/代际范围仍需v0.7 Roadmap/Plan冻结。
- 当前计划、D门、共享核心单写、增量/cache/AutoDown/P95/内存义务保持；成熟度提高不将完整优化JIT、OSR/deopt、自主codegen等自动变成全示例兼容前置。
- 检查新增本地链接、修订差异与冲突排期措辞；既有日期/版次历史在明确的新合同覆盖下保留。没有新建实现Plan或完整v0.7任务表。

本轮未执行cargo、docs_gen、示例运行或性能测试；文档静态通过不证明任何M里程碑或兼容能力已经达成。
