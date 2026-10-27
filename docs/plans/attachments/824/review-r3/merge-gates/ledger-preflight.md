# PLAN-824 台账预检（只读）

2026-10-27，目标 `.autoos/specs.json` version=101，SHA256 `6b30043a9cacfdb2332db1fa00a52c3ef64c941b90380cdddeb263855b7bb72e`；JSON 可解析，但 `scripts/plan815a_ledger_deposit.py` 的 store-schema 校验报 57 项。`D:/autostack/auto-musk/backend/crates/musk/src/specs.rs` 的 `SpecItem` 字段为非默认字段，`SpecStatus::parse_lenient` 未收 `active`；现存5个 active 条目 P852-DESIGN-CONFORMANCE/P852-DESIGN-CERT/P822-D1/P822-D2/P822-T，另有多条历史记录缺字段或时间戳类型不符。因此不能声称当前台账可被该 store schema 加载。

本机HTTP store无响应；检查 `.autoos/specs.json.bak*`，现存备份在相同schema下均有错误（最近备份version100有61项；最少错误的version63仍有2项，且缺167条现有记录），不能安全覆盖当前数据。未编辑、恢复、重建或更新台账。依据 auto-plan-merge SKILL 的“Load failure means stop”，需先恢复/重建可加载的台账并保全现有历史，然后才能安全执行受限投影及归档收尾。824代码的门禁及独立审查可继续，绝不把无效台账当成功沉淀。

更新：858并发合入后，当前主检出台账变为version=102，SHA256 `a5c23f40350b87587602d1fecb432cd6f2da4b08b94e78f7acc90cf1a39c0516`；同一严格校验仍有54条受影响记录，五个不受状态解析器支持的active条目仍在。原version101结论为历史。未修改当前台账。

只读恢复草案：[逐项提案](ledger-recovery-proposal.json.txt)，基于version102和上列SHA，不触碰现有台账；拟变更54条历史记录的缺失/不兼容字段，补空数组和可选字段、既有ISO日期转UTC秒、未知必需时间用0占位、5条active映射stable（有语义差异，需明确授权并独立复核）。草案投影后严格校验为0项，但不代表实际台账已修复；实施前必须重新读取核对源hash、备份、逐项审查，保全内容与未变字段，验证后读回，避免并发写入。该恢复属于824之外的共享历史数据修复，未执行。
