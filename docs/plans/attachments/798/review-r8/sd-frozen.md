### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-mir/project.md、core-i32.md | 无实现→有限Typed MIR、slot/CFG/初始化和Checked门 | 正确性共同层 | AC-01/02 |
| SD-02 | add | docs/specs/auto-av/project.md、abc-core-i32.md | 无新版AV→有限ABC wire/验证/装载/执行及unsupported | 不混旧AVM和新引擎 | AC-01/03/04 |
| SD-03 | add | docs/specs/auto-av/cache-and-conformance.md | 无→MIR/ABC producer身份、恢复/依赖/root/lease/资源/计量 | 跨工程复用可信 | AC-05/06 |
| SD-04 | modify | docs/specs/auto-ac/artifact-contract.md | 783 A08 Future不改历史→追加798 producer的实际实现范围与carrier/引用/准入 | 不给旧producer冒领覆盖 | AC-05/08 |
| SD-05 | modify | docs/specs/auto-hir/stage-contract.md | 旧S0–S6保留→追加HIR到新MIR的消费前门/资源支持表 | 不扩大旧Checked保证 | AC-02/08 |
| SD-06 | modify | docs/specs/auto-test-case/architecture.md | 在D-796后probe/run支持表追加本recipe、worker身份/失败/观测 | 使用正式runner | AC-07 |
| SD-07 | add | docs/specs/auto-av/cache-and-conformance.md | AC/AV/AJ/profile/native产物引用为有限合同；完整AJ/发布仍future | 不把模型当运行能力 | AC-08 |

Spec导航/ledger只在merge按实际增量登记；796语义及797ABI的权威Spec由各自owner维护，本计划只引用，不回写其未合入文件。

