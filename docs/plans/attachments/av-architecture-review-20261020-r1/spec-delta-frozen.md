# 修订后规范增量冻结（文档合同，非已沉淀Spec）

2026-10-20。当前canonical Spec未回写，未来review/merge按真实实现核对。输入与版本见inputs.json.txt。

## docs/plans/798-typed-mir-abc-av-probe.md

| delta_id | operation | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-mir/project.md、core-i32.md | 无实现→有限Typed MIR、slot/CFG/初始化和Checked门 | 正确性共同层 | AC-01/02 |
| SD-02 | add | docs/specs/auto-av/project.md、abc-core-i32.md | 无新版AV→有限ABC wire/验证/装载/执行及unsupported；r3 kind/entry/imports往返与准入（V-14）；r4 §5.8/K14-01..06的类别正反控、完整恢复及资源证据政策 | 不混旧AVM和新引擎 | AC-01/03/04 |
| SD-03 | add | docs/specs/auto-av/cache-and-conformance.md | 无→MIR/ABC producer身份、恢复/依赖/root/lease/资源/计量；r3 kind/版本绑定及歧义旧缓存失效（V-14）；r4 §5.8/K14-01..06的类别正反控、完整恢复及资源证据政策 | 跨工程复用可信 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-ac/artifact-contract.md | 783 A08 Future不改历史→追加798 producer的实际实现范围与carrier/引用/准入 | 不给旧producer冒领覆盖 | AC-05/08 |
| SD-05 | modify | docs/specs/auto-hir/stage-contract.md | 旧S0–S6保留→追加HIR到新MIR的消费前门/资源支持表 | 不扩大旧Checked保证 | AC-02/08 |
| SD-06 | modify | docs/specs/auto-test-case/architecture.md | 在D-796后probe/run支持表追加本recipe、worker身份/失败/观测 | 使用正式runner | AC-07 |
| SD-07 | add | docs/specs/auto-av/cache-and-conformance.md | AC/AV/AJ/profile/native产物引用为有限合同；完整AJ/发布仍future | 不把模型当运行能力 | AC-08 |

## docs/plans/820-av-typed-data-layout.md

| delta_id | operation | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/data-layout.md | 无本profile→仅本计划已验证的数据/资源/拒绝规则；r2纳入§5.4/V-11的类型等价与执行表示及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 明确支持边界 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/auto-mir/data-place-profile.md | 无本profile MIR→类型/Place/状态/准入及来源合同；r2纳入§5.4/V-11的类型等价与执行表示及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 防绕过验证 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | 798共享核心实际交付→增加本producer身份/恢复/失效、oracle及资源边界；r2纳入§5.4/V-11的类型等价与执行表示及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 不冒领旧producer能力 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-av/project.md | 798共享核心实际交付→登记本profile与明确非目标，原标量仍独立 | 现状准确 | AC-06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记av-data-820独立recipe/身份/父侧oracle与必需反控 | 正式执行非仅fixture | AC-01/04/06 |
| SD-06 | modify | docs/specs/auto-hir/stage-contract.md | 已有HIR域语义保持→追加其到av.data/1 MIR/ABC的实际消费支持/拒绝/资源责任 | 不将新后端能力反授旧域 | AC-01/02/04/06 |

## docs/plans/821-av-place-owned-access.md

| delta_id | operation | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/place-ownership.md | 无本profile→仅本计划已验证的数据/资源/拒绝规则；r2纳入§5.4/V-11的move-kill与清理分析及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 明确支持边界 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/auto-mir/owned-access-profile.md | 无本profile MIR→类型/Place/状态/准入及来源合同；r2纳入§5.4/V-11的move-kill与清理分析及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 防绕过验证 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | 前序profile实际交付→增加本producer身份/恢复/失效、oracle及资源边界；r2纳入§5.4/V-11的move-kill与清理分析及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 不冒领旧producer能力 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-av/project.md | 前序profile实际交付→登记本profile与明确非目标，原标量仍独立 | 现状准确 | AC-06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记av-data-821独立recipe/身份/父侧oracle与必需反控 | 正式执行非仅fixture | AC-01/04/06 |
| SD-06 | modify | docs/specs/auto-hir/stage-contract.md | 已有HIR域语义保持→追加其到av.owned/1 MIR/ABC的实际消费支持/拒绝/资源责任 | 不将新后端能力反授旧域 | AC-01/02/04/06 |

## docs/plans/822-av-typed-collections-variants.md

| delta_id | operation | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/collections-variants.md | 无本profile→仅本计划已验证的数据/资源/拒绝规则；r2纳入§5.4/V-11的集合/分支的状态和计量及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 明确支持边界 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/auto-mir/collection-place-profile.md | 无本profile MIR→类型/Place/状态/准入及来源合同；r2纳入§5.4/V-11的集合/分支的状态和计量及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 防绕过验证 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | 前序profile实际交付→增加本producer身份/恢复/失效、oracle及资源边界；r2纳入§5.4/V-11的集合/分支的状态和计量及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 不冒领旧producer能力 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-av/project.md | 前序profile实际交付→登记本profile与明确非目标，原标量仍独立 | 现状准确 | AC-06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记av-data-822独立recipe/身份/父侧oracle与必需反控 | 正式执行非仅fixture | AC-01/04/06 |

## docs/plans/823-av-owned-closure-captures.md

| delta_id | operation | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/closure-environments.md | 无本profile→仅本计划已验证的数据/资源/拒绝规则；r2纳入§5.4/V-11的环境所有权与调用合同及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 明确支持边界 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/auto-mir/closure-capture-profile.md | 无本profile MIR→类型/Place/状态/准入及来源合同；r2纳入§5.4/V-11的环境所有权与调用合同及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 防绕过验证 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | 前序profile实际交付→增加本producer身份/恢复/失效、oracle及资源边界；r2纳入§5.4/V-11的环境所有权与调用合同及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 不冒领旧producer能力 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-av/project.md | 前序profile实际交付→登记本profile与明确非目标，原标量仍独立 | 现状准确 | AC-06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记av-data-823独立recipe/身份/父侧oracle与必需反控 | 正式执行非仅fixture | AC-01/04/06 |

## docs/plans/824-av-dynamic-values-freeze.md

| delta_id | operation | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/dynamic-portable-values.md | 无本profile→仅本计划已验证的数据/资源/拒绝规则；r2纳入§5.4/V-11的动态类型与可移植授权及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 明确支持边界 | AC-01/02/03/04 |
| SD-02 | add | docs/specs/auto-mir/dynamic-cast-profile.md | 无本profile MIR→类型/Place/状态/准入及来源合同；r2纳入§5.4/V-11的动态类型与可移植授权及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 防绕过验证 | AC-01/02/04 |
| SD-03 | modify | docs/specs/auto-av/cache-and-conformance.md | 前序profile实际交付→增加本producer身份/恢复/失效、oracle及资源边界；r2纳入§5.4/V-11的动态类型与可移植授权及语义元数据规则；r3 §5.5/V-12的阶段计数、状态/资源及恢复规则 | 不冒领旧producer能力 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-av/project.md | 前序profile实际交付→登记本profile与明确非目标，原标量仍独立 | 现状准确 | AC-06/08 |
| SD-05 | modify | docs/specs/auto-test-case/architecture.md | 既有probe/run保持→登记av-data-824独立recipe/身份/父侧oracle与必需反控 | 正式执行非仅fixture | AC-01/04/06 |

## docs/plans/825-av-loaded-program-runtime-efficiency.md

| delta_id | operation | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/loaded-program-runtime.md | 无本Plan合同→§5已验证的私有装载/PC/调用策略、拒绝及资源规则；r2纳入§5.6/V-11版本依赖/阶段计数与预算规则 | 设计与实物一致 | AC-01/02/03/04 |
| SD-02 | modify | docs/specs/auto-av/cache-and-conformance.md | 已交付profile→本Plan恢复/版本/当前能力准入及测量反控；r2纳入§5.6/V-11版本依赖/阶段计数与预算规则 | 不恢复trusted或PASS | AC-04/05/06 |
| SD-03 | modify | docs/specs/auto-av/project.md | 已交付能力保持→登记本Plan实际优化/边界与未实现面 | 不冒领data/JIT | AC-06/08 |
| SD-04 | modify | docs/specs/auto-test-case/architecture.md | 保留既有runner→独立av-abc-825 recipe/父侧结果/统计反控 | 真实路径验收 | AC-01/06 |
| SD-05 | add | docs/specs/auto-av/runtime-benchmark-method.md | 无固定执行测量合同→同义负载/引擎tier/阶段/计数/噪声/工具缺口政策 | 不以跨层速度误导 | AC-02/04/06 |

## docs/plans/826-av-compact-abc-format-and-size.md

| delta_id | operation | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/compact-abc-format.md | 无本Plan合同→§5已验证的版本化紧凑wire/section/密度、拒绝及资源规则；r2纳入§5.6/V-11版本依赖/阶段计数与预算规则 | 设计与实物一致 | AC-01/02/03/04 |
| SD-02 | modify | docs/specs/auto-av/cache-and-conformance.md | 已交付profile→本Plan恢复/版本/当前能力准入及测量反控；r2纳入§5.6/V-11版本依赖/阶段计数与预算规则 | 不恢复trusted或PASS | AC-04/05/06 |
| SD-03 | modify | docs/specs/auto-av/project.md | 已交付能力保持→登记本Plan实际优化/边界与未实现面 | 不冒领data/JIT | AC-06/08 |
| SD-04 | modify | docs/specs/auto-test-case/architecture.md | 保留既有runner→独立av-abc-826 recipe/父侧结果/统计反控 | 真实路径验收 | AC-01/06 |
| SD-05 | modify | docs/specs/auto-av/abc-core-i32.md | 旧profile单一wire→同一语义profile的版本路由/旧缓存策略与必需元数据 | 不重解释旧版本 | AC-02/04/06 |
| SD-06 | modify | docs/specs/auto-av/runtime-benchmark-method.md | D825测量合同→追加新wire分段/压缩/loaded体积及编码域与准入域区分、可比边界 | 体积口径唯一来源 | AC-02/05/06 |
