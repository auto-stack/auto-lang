# PLAN-820 r6 R4 — needs_fix (T-06 only reopened)

2026-10-24；final readiness / affected repair review；本聊天 Codex，独立于实施上下文，未启动代理。已核对R3具名根因×4及逐finding关闭表。唯一树 D:/autostack/.wt/lang-820/auto-lang；plan-820-dev；受审HEAD a789a2adc1afdab97025a0c294bb6121c97aa797；repair base e8497f2f8d275873f55b4d7c0e9dd6ac1b75e3bf；前后HEAD/clean一致；diff-check绿。

入审master已推进92f709da857e66598a3dec4cbc247258edd6c073（用户交接簿记59f00f13b之后）；共享Plan SHA256 68978B14E87D8022650F65E7EC3D0DA3442902204EAD70BD61D87F968710E9CE，实际executing7/9。SD draft SHA256 50B2176B9E9AB83017A26235C0A680B25B771D315C2DC06056FC1B910DF727D8。未把在途主检出变化并入受审实现。

## 本轮关闭

- **F820-R3-01 CLOSED**：布局memo/color跨record持久保留，步骤直接扣Verifier.steps，Budget映射Resource(VerifySteps)。独立 `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_data_820_resources av_data_820_layout_`，2 passed、0 ignored，0.21s。原2100链允许通过符合优化后的实际工作量；近顶共享budget反控Budget；过大模块资源拒绝，不以原O(N²)重复工作要求优化后的合法图仍拒绝。
- **V-04 overflow coverage CLOSED**：同一两测中，真实62层溢出通过compute_layout/MIR/ABC三层受控拒绝，61层2^63合法布局/MIR对照通过；原不可达注释已替换。
- **F820-R1-01 CLOSED（约定真实操作反控面）**：`av_data_820_v13_metering_injection...`独立1/1、0 ignored，0.02s。代码明确从给定路径实际std::fs::read，按返回长度计量；未改def从原bundle实际lower+verify_mir后入计数，同一门拒绝且不安装。此前模拟数字问题已消除。已有身份/生产窗等未变证据保留。
- **F820-R2-02 partial**：root全名、执行相对listing漏项、ignored、重复/意外ok行均有改进；GuardSelfTest独立3正7负全绿。但尚有下面原合同的必需集合缺口。

## F820-R2-02 remains PARTIAL / P2 / T-06 AC-06 V-10

原规则是必需case集合不能漏项，不是仅保证当前二进制列出的测试都运行。Run-Tier现在把同一cargo选择的 `--list` 结果直接赋给required；没有独立的合同/固定case-ID清单与listing对账。因此测试被删、cfg排除或选择表达式漏掉时，listing和执行一起少项，门禁仍可绿。

独立failure-propagation控制：[脚本](missing-required-control.ps1)、[输出](missing-required-result.txt)。模拟first-case的listing及执行都只剩 `av_data_820_first_case_whole_path` 一项（native/bits/restore/owner/其他必需输入全消失），真实验收脚本仍exit0，`required ids:1` / `first-case PASS`。没有运行真实产品/整档，不把模拟输出作为行为证据。R3原“输出漏掉listing已存在用例”这一子面现在可拦截，但**合同必需用例从listing消失**仍不被检测。

最小关闭：保留自动解析全限定实际ID的优点，再以独立合同manifest（case/subcase→选中完整ID或唯一短名映射）核对必需项都在listing中，不能从被验证对象自行生成唯一oracle；加入“listing和执行同时缺某必需项”的实际脚本负控，合法当前集合和root全限定输出保持绿。允许重命名时显式同步绑定，不要求锁死不相关测试总数。此为原T-06“漏必需用例非绿”的未闭余量，不新增产品支持范围。

## F820-R4-01 P2 / T-06 health — 本轮新增资源测试未格式化

定向命令（没有改实现）：

```powershell
rustfmt --edition 2021 --check --config skip_children=true experimental/ac-av-probe/probe/tests/av_data_820_resources.rs
```

受审树exit1，差异从244/254/261等新增行开始，见[原始输出](format-result.txt)。同时检查本轮改动的layout/mir_verify/data_session/incremental四文件没有格式差异。整workspace `cargo fmt --all -- --check`亦失败，但它递归进入既有auto-down生成文件及根仓旧example错误，不能全归本计划；本finding只绑定本轮新增资源测试。关闭只需原树格式化该文件、提交并重跑定向格式检查；不要求为格式修改跑整档或重复资源性能矩阵。

## 状态与范围

回executing6/9，只重开T-06；T-04/T-09本轮修复完成标记保留。前轮R2-01事务、R1-02父侧采样、R1-03native等相关实现/依赖未改，复用已绑定证据；这些关闭不等于所有AC独立pass。AC-06门禁完整性仍partial，AC-07未过；其他未审AC/V/SD与依赖祖先/目标Spec逐项final对账pending。没有继续跑已知仍需修复代码的全final聚合。

work仅修独立必需集合oracle与格式，提交后再审此两项及未审面；已闭问题没有新机制变化时不重复全量。未改实现/canonical/ledger，未合入。merge-owned裸cargo t+tv+独立workspace完整有限合同档仍pending；本轮未跑整档或真实acceptance聚合。
