# PLAN-815 T-07子批2 / SD-01..07核验

2026-10-22，Plan r3，review_scope=phase（子批2及T-07整体就绪性核对，非T-09）。HEAD 503c9f67f5fca40661ca59a230dcd79c8418ef90，code e22fc75924a7f058a5df8150956b2d66268c5ecd，base f6fd0ce7c86fd3c4f70ddddec296f2d021642031。当前canonical、源码/依赖/配置/锁hash见baseline.json.txt；当期交接/资源审计/handoff已冻结。实际tool SHA256 e319c5364523dbfac0c61d61ecb087b11a8d0261cdfc767bd1dfacc1b4b803f2与交接相符，但身份正确不等于完整发行面已经验收。

| SD / target | 当期核验 |
|---|---|
| SD-01 docs/specs/auto-ui-tools/project.md | 生产改动与现有M814-B canonical双直依赖合同一致；Rust app及dev共享manifest/壳层头修复有实际792 fresh/locked编译证据。不能外推新tool独立发行及全部字段/profile均验过。A已发布子集继续保留。 |
| SD-02 docs/specs/tool-protocol/project.md | 本批无协议/wire/预算变化；子批1四回执、深度读写、旧控制/取消已闭范围保留。当前请求的字段生产消费仍需与完整V-02对账，数据能往返不能替代实际字段效果。 |
| SD-03 docs/specs/auto-man/project.md | 旧Man薄门面未被本批修改；新增workspace依赖窄接线符合共享生成器归属。work披露shell_pack freshness红，本轮不因“预存/债务”给其合同内功能豁免；T-09须按实际调用/未变基线裁定，不能扩大本轮pass范围。 |
| SD-04 docs/specs/project-model/project.md | Vue移除主pac存在性门，真实已求值请求删pac后再生一致原例通过；静态Vue字段正例通过。缺mode/output_root/input_sources/sidecar/external-back等逐profile消费/固定限制/不适用映射与区分性证据，不能沉淀“完整字段均已验证”。不得为补表而擅自扩张原支持面。 |
| SD-05 docs/specs/auto-lang/ui/overview.md | 三组UI工具源资产及runtime依赖资产必须据实区分owner。Inter三字重与INTER-OFL.txt实际在auto-ui-runtime且被renderer include_bytes消费，handoff删除“全仓不存在”的勘误错误。 |
| SD-06 docs/specs/repository-boundaries/generated-ui-distribution.md | 独立792旧Rust/Iced应用driver25/25通过，两轮fresh/重定位locked真实编译有效；未构建/启动随包auto-ui-tool，不等于本Plan要求的新工具发行/独立消费已闭合。实际源码包包含gallery/wm/shadcn全组及Inter字体/许可，资源审计“资产组不直接进792包、全仓无Inter”与实物冲突。必须精确区分app资源allowlist、vendored crate资源、tool embed和用户工程materialize。 |
| SD-07 docs/specs/auto-ui-runtime/project.md | 本批未改runtime实现；既有运行生命周期证据只按未变适用路径继承。当前canonical文档已明确Inter字体/壁纸随iced资产，本轮资源结论不能覆盖删除这一事实。 |

当期T-08指纹正确，但来源清单及完整交付保证须纠正。SD-01/02最终仍应以现行A正文增补，不用迁移前历史before覆盖现有A发布事实。Plan影响metadata沿r3保持；本轮无合同修订、未写canonical或live ledger。T-07未通过整任务，T-08/T-09及最终pass后merge-owned冻结门、实际合入/知识沉淀/发布归档仍pending。
