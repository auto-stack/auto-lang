# PLAN-820 r6 final readiness review — needs_fix

2026-10-24，独立于实施的本聊天 Codex 上下文；未创建代理。审查为 final 入口就绪核查，发现 work-owned 明确未完成后停止完整 final 验收，不声称覆盖全部 AC。

## 绑定基线

- 唯一树 D:/autostack/.wt/lang-820/auto-lang；plan-820-dev；HEAD a33538088cd4ce309176d217361edc68a1d2ced8；base 37e51253689631ffa61a9af58041c5f6646a7569；入审实现 clean。
- 主检出 Plan SHA256（入审）：700FA8FE06FE7245FF94F5154890D785FA52A39D937C0D81F5DE841D880CED3A。
- SD 草案 SHA256：3DC5A61DD32865DA0597A5D350F03E7C6177DD5A80324B2D01133E49D8D4D5D7。SD-07 当前描述前门全量重编译，与已批准 AC-09 冲突，不能按此沉淀。
- D798/D828 archived 为执行方声明，本轮未完成依赖祖先/Spec 全量再核；不将其记 pass。

## F820-R1-01 / P1 / AC-09, T-09, V-13 — 全链增量合同未实现

既有规则：计划 §5 全链路增量要求未改 HIR/MIR/代码对象及代际不变、未改生产阶段和产物 IO 为零，并且强制 whole 前端退化使正式 gate 非零。

实际路径：data_session.rs::edit_body → rebuild_unit（约91行）每次调用 compile_record_source(&self.src, …)，对整段 record 源码重建 checked bundle。只有 lower/admit 的计数器与 FunctionCode 身份被保留；没有未改 HIR 身份/前端生产/IO 观测。apply_source 的受影响 defs 由调用者手填，也未证明实际依赖/layout/helper/授权变化的精确失效。

重放：在唯一树运行 `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_data_820_incremental`，实际 **3 passed / 0 failed / 0 ignored，0.01s**。其中 forced_whole_rebuild 测试断言 moved>=2 并成功退出，未把相同退化送入正式 gate 得到非零；正控也未观测整前端重编，因此这个绿不能证明 V-13。该结果是覆盖缺口证据，不声称测试本身应无条件失败。

关闭 oracle：同树实现约定真实 data consumer 全链增量，生产入口观测 HIR/MIR/代码身份、各阶段/IO；完整四子case含布局/依赖/授权正反控；强制 whole 前端（含只保 hash 却替换对象）经正式 gate 确实非零；保留合法局部编辑结果与旧快照对照。不得以 scalar-only 语法 owner 为由豁免现有合同；无法实现应显式回规划。

## F820-R1-02 / P1 / AC-05, T-02/04/06, V-07 — 瞬态进程峰值缺证据

既有规则：§5.2 父侧采样 worker 瞬态 private-bytes 峰值，与自有计数共同判定，包含 allocator/解码/暂存。

实际：av_data_820.rs::av_data_820_worker_memory_envelope_b_line（799行）在测试自身进程取 base，25轮 request_data 返回后才取 now。没有父侧采样新 worker，也没有在 request 内采样；已释放的前端/codec/执行暂存峰值可完全遗漏。源码注释称 conservative 不能使结束时采样变成真实峰值。未证实实际超限，finding 是必需 oracle 缺失。

关闭 oracle：父侧监测实际新 worker 全生命周期峰值，绑定空载/输入/采样间隔/负载状态与 raw；按有效依赖合同的 A/B 归因与预算判定，补暂存峰值能被捕获的反控。只重跑该资源检查及受影响失败/清理控制，不跑整档。

## F820-R1-03 / P1 / AC-01, T-02/06, V-01 — 首例独立 native 对照被擅自替换

既有规则：§5.2 同一 record 源码旧 HIR-native 与手审 expected 一致；首例不可行须记录冲突回 new，不能去掉 oracle。

实际：contract-decision §11 改成 native scalar-only 时可用等价 scalar 化或手审代替；T-02 完成行仅记录 scalar-only 边界，av_data_820_first_case_whole_path 不调用 native 对照。没有批准该要求变更的证据。SD/进度不能代替授权修订合同。

关闭 oracle：同一受检 record 输入通过约定 native 入口与固定 expected 独立对照；若 native 依赖确实不支持，提供实际输入/拒绝/支持面证据，经 new 显式修订并确认，不能 work 自行降级。

## AC 与后续

AC-01 partial（F03）；AC-05 partial（F02）；AC-09 fail（F01）；AC-02/03/04/06 pending 完整行为与负控核查，已有执行方绿不转记独立 pass；AC-07 fail；AC-08 merge pending。SD-01..06 仅初读，SD-07 fail，目标 Spec/hash reconciliation pending。未判定其他实现面无缺陷。

本轮另执行 `git diff --check 37e512536 HEAD`，exit0。未跑整档/acceptance 聚合、未改实现/Spec/ledger、未合入。最终复核 HEAD 和 status 仍同基线且 clean。

回 work：先关闭上述 readiness 缺口并提交，恢复 T-02/T-04/T-06/T-09 的可验完成证据，再 final review 未审面及受影响内容。merge-owned 裸 cargo t + tv + 独立 workspace 完整有限合同档仍 pending，不能提前运行或合入。登记债务仅作跟踪，不授权延期。
