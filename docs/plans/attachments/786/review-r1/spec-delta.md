# PLAN-786 R1 规范增量快照（待修复后重冻结）

reviewed_commit: bde9d0523198f1a15bc908c62a5b18b59324e61a
plan_revision: 1

## SD-01 — docs/specs/auto-man/project.md

源版本 ec934b0006acb9b9838def82d0fcaf438ae0153f；SHA256 C18FFAAFD22FB30244E9886B661F56C97BEED87EAD9CB3670C660B4914B195A4。

before: 四名称/locale 规则及 Vue 行扫描。
after（拟新增 current-state）：Vue 工程生成的 name/title/title_zh 由内部 project_display_reader 一次读取；优先使用 auto-project-model descriptor。字段准入检查声明门槛行与 provenance 对应、重复键及单行归一的一致性。其余输入使用唯一具名 legacy 兼容扫描，保持旧首匹配、name 空值保留及 title 跳空行为。locale、aura-app/name 等原有兜底留在调用者；其他文本读取入口不在本轮迁移。
兼容原因：ParseError / HiddenOrPhantomLine / DuplicateKey / UnsupportedValue / LineFormMismatch。不得扩大模型语法支持面；数值字段的 astr 与旧行值相等时也可准入，不能误记为仅 Str。
理由：有限生产读取去重，AC01/02/03。修复 F786-R1-01 后补入真实安全处理边界；当前不宣称终止性通过。

## SD-02 — docs/specs/project-model/project.md

源版本同上；SHA256 C9CACD0D58A1FC5B6D023365D322A02D995A0282DDDB2B63A6BF1B6FD407C133。

before: 后续接线归属称不切换默认生产入口。
after（替换该绝对描述）：auto-man Vue 展示字段 reader 是首个有限生产消费者，使用 descriptor 及显式 legacy 兼容策略。其准入和旧行扫描的行为保持属于 consumer adapter，既不扩大 project-model 的冻结静态子集，也不表示 registry/loader 或其他 Pac/VM 生产入口已迁移。展示 locale/env 和构建仍属消费层；auto-man 尚未脱离语言依赖。
理由：准确记录消费者，AC01/02。

supersedes_spec_components: [docs/specs/auto-man/project.md, docs/specs/project-model/project.md]
new_spec_components: []
touched_goals: []（内部组织与兼容治理，无能力目标关闭）

后续迁出清单还应明确登记新路径 crates/auto-man/src/project_display_reader.rs 及兼容臂归属；784 附件目前只有 vue/rust_ui/pac 的宽泛入口。
本轮未发布 canonical Spec 或修改 ledger；needs_fix，快照不能当作 pass。
