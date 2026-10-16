# 804–806 新合同预检

2026-10-17 / stage: new / baseline: 9932962bc157be13bcaa0520011c7c78082cb2fe。此为起草合同预检，不是实现测试或对799/802/803的独立review。

| Plan / revision | outcome | next | 判断依据 |
|---|---|---|---|
| 804 / 1 | pass | 合同确认后work | 不依赖799/801；已有provider模式足够，802共享接线有明确窗口/后合入责任；真实模态/拖出证据不能用fake/组件结果替代 |
| 805 / 1 | pass | 合同确认后work | 三纯模块无AST/执行依赖；与803资产不重叠，生产Ninja消费/旧规则和adapter留上层有验收 |
| 806 / 1 | blocked | 799和801交付后T-01 | 默认完整msg/model文法迁移必须使用真实模型/端口；内部结构调用、pilot与默认差异均有owner，不能绕前置复制接口 |

每项6条AC、8任务；work=6项、review=1项、merge=1项。实现正确性/必要OS证据全部work-owned；整档门是final review后的merge职责。源路径与入口已读、未知机制由T01有界验证，不以未定决策放宽验收。所有fixture都是拟建入口，本轮没有执行结果。

本批只写plans/design簿记；既有799/801/802/803合同、Rust源码、canonical Specs/ledger与旧仓不改。804保留VM/a2r父窗策略差异及dnd取消true语义，805保留命令输出/公开类型，806不扩默认bare-field文法。

编号/章节/任务/AC/owner、已交付前置与阻塞前置、相对链接及差异检查见[机器收据](preflight.json)；SHA与取材时在途header见[inputs.json](inputs.json)。[编排](../../reports/extraction-followup-parallel-order-20261017.md)是本批调度依据。未来状态变化须重新核对，不把该快照当永久放行凭据。
