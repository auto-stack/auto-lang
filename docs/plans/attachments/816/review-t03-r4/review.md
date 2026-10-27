# PLAN-816 T-13 / T-03 受影响复审 R4 — pass

2026-10-21（客户端日期），Plan revision 2，review_scope=phase，仅为 T-03 的受影响
审查。本聊天 Codex /root 独立于实施会话，未创建 agent，具体模型/会话 ID 未暴露。
不是整 Plan 或 M816-B 最终验收，不改变已发布 M816-A 的审查与交付收据。

- 原树 `D:/autostack/.wt/lang-816/auto-lang`，`codex/plan-816-dev`，入场/收尾 clean。
- 受审 `5720a90594b8e5964b82c3772c2f52814597cbb9`。
- 修复基线 `d93106fd20a10ca6e4a435b8edbbdfc1518e0e34`；仅
  `auto-man-runtime/src/language.rs` 测试/注释 +12/-3，生产实现未改。
- T-03 原始基线 `7a2323cc364000dca8777d06c8d16629d1b8456f`；main
  `b81580553052b8663cdf3f183ed6888f78835825`，auto-down
  `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。T-03 尚未合入 main。
- [126 项指纹及 6 个原始 blob](inputs.json)，Plan hash 取本轮簿记前。
  [冻结 SD](spec-delta-frozen.md) 与 R1/R2/R3 相同，SHA256
  `3c9a1fc2fcc19f32a230d15169e82ef69e5f9c34c9ebe4a00612e89dcf7f63bc`。

**F-816-T03-R3-01 CLOSED，无新增阻断 finding。T-03 受影响代码/验收/规范审查 pass；
最终 B 冻结门、完整集成与交付收口仍 merge-owned pending。**

## 新测试的真实正负控制

新 oracle 将 `process.args().len()` 的 VM 返回值解析为 `usize` 并断言 `len == 1`。
有效结果 1 被接受，真实泄漏结果 2 被拒绝，不再把两者都按一个字符串片段判绿。

本轮从 **5720a9059 当前源码重新生成**同一晚复位变体，未执行 R3 冻结旧测试冒充闭合：

- [晚复位副本](language-late-reset-mutant.rs) 仅把 argv 清空移到执行完成后，并保持
  读失败清空；整个当前 `#[cfg(test)]` 块逐字保留。
- [入口先复位的正控副本](language-correct-control.rs) 完整保留当前生产实现和测试。
- [同 hash 证明](mutation-proof.json)：当前提交、新变体、正控三者的测试块相同，
  且与 R3 冻结旧测试块不同。没有修改工作树生产源码。
- [回放脚本](replay-mutation.ps1)，独立 consumer、单线程运行，避免 argv 进程全局态
  干扰；固定 target-dir，使用不同 test target 名称。

[负控输出](mutation-output.txt)：

```text
assertion `left == right` failed: VM 内 argv 仅程序路径（长度 1）；泄漏前次哨兵会得到 2，got 2
  left: 2
 right: 1
test language::tests::run_language_file_resets_script_args_like_legacy_run_file ... FAILED
real_vm_correct="1" real_vm_late_reset="2" existing_oracle_n=1
test result: FAILED. 1 passed; 1 failed
cargo exit 101
```

失败准确发生在**新测试的标量长度断言**，不是编译、fixture 或读文件失败。另一个
观察用例只保留旧 oracle 的诊断，确认真实 VM 的正确/泄漏值分别为 1/2、晚复位后
全局表为空；它不替代当前测试的负控判定。

[恢复正确入口的同测试正控](correct-control-output.txt)：1 passed / exit 0。
[当前包 lib](runtime-tests.txt)：118 passed / 4 ignored / exit 0，成功执行、VM 内
观察及读取失败 argv 复位控制保持。由此确认原漏检被消除。

## R1/R2、闭包与健康证据的适用性

本轮只改测试 oracle，已对照 R3 的全部 113 项输入指纹：变化仅为共享 Plan 簿记和
`language.rs`；后者 `#[cfg(test)]` 之前的生产前缀与 d93106fd2 完全相同。
依赖、Cargo.lock、验证配置、原 probes 和其他生产源码均未变。
[逐项比较与生产前缀证明](prior-evidence-applicability.json)。因此复用以下已实际执行
且仍适用的 R3 证据，明确区分复用与本轮重跑：

| 项目 | 当前结论 | 适用证据 |
|---|---|---|
| R1 五项 | 保持 CLOSED | [R3 原例回放](../review-t03-r3/replay-output.txt)：0 失败组；旧 Jetpack 默认真实生成、非 UI 有效 API 到真实 bind、六 util 路径、Git 状态、恢复的四条 adapter 回归 |
| R2-01 | 保持 CLOSED | [R3 原例](../review-t03-r3/regressions-summary.txt) 及 [runtime 集测](../review-t03-r3/runtime-integration-tests.txt) / [compat 集测](../review-t03-r3/compat-tests.txt)：显式先装、默认兜底、链接 ctor 后显式胜出、后装不覆盖；生产双槽实现未变 |
| R2-02 产品实现 | 保持 CLOSED | [R3 语言原例](../review-t03-r3/language-output.txt)：旧/新 native argv 均 1、sentinel=false、空参数/读失败保持；本轮新正负控进一步核验 VM 内观察 |
| 类型与闭包 | pass | [R3 闭包摘要](../review-t03-r3/closure-summary.json) / [完整图](../review-t03-r3/runtime-tree.txt)：normal/build 420 包，auto-lang / auto-ui-tools/compiler/runtime / iced 家族为零；类型身份编译通过，re-export 未变 |
| 管理与其他集测 | pass，复用 | [R3 管理 8/8](../review-t03-r3/management-tests.txt)、集成 10/10、外部旧 adapter 2/2；真实 npm 成功/失败传播，相关代码/输入未变 |
| 三包 check / 测试唯一性 | pass，复用 | [R3 check](../review-t03-r3/check-diagnostics.json)、[122 描述符/122 唯一](../review-t03-r3/test-list-summary.json)；本轮 runtime lib 重编执行已绿，未新增测试名 |

本轮重跑：fmt --check / git diff --check 通过；
[777 strict](modularity.json) / [输出](modularity-output.txt) 为 pass / 0 violations /
3 grandfathered / 132 边，原绑定 2f2b54888 的基线及精确规则未改。

本轮 clippy exit 101，60 描述符 / 30 唯一旧 unused_io_amount 错误仍只在两份旧
Ninja builder / CMake 文件；[当前诊断](clippy-diagnostics.json)，与 R3 相同的
[基线同构证明](../review-t03-r3/clippy-baseline-proof.json) 适用。改动的 language.rs
没有诊断，不新增 health finding，也不声称全 clippy 绿。

用户附带的 Plan 文本替换失败不作为代码或簿记状态证据；本轮从实际共享 Plan 检查
任务与历史记录，追加本次受审 SHA 的独立收据，保留既往 needs_fix 历史。

## AC、Spec delta 与后续门

AC-01 / T-03 当期范围（管理/语言 adapter、compat、单源、闭包、旧行为及所需回归）
通过；F-816-T03-R1-01..05、R2-01/02、R3-01 均无开放残项。AC-07/08 的 A 历史 pass
及 AC-09 已发布保持。本次不是 AC-01 的薄 CLI 全范围或 AC-02/03/04 的 B 完整链验收。

SD-03/04 的层次与入口保持合同通过 T-03 当期审查；冻结候选未变，spec-impact 精确
路径与 GOAL-003 有效。B 的 as-built 文本仍按后续任务收口，未把本次局部 pass 当作
规范已发布，未修改 canonical Specs 或 live ledger，SD-01-A/SD-05-A 发布子集保持。

T-03 与本次复用 T-13 标记完成，Plan 维持 `executing`、current_step=6/14，不能标记
整 Plan `reviewed`。下一步 T-04 的窄能力门为 T-03 + 已发布窄 tool，现可继续；完整
UI 正控/profile/bootstrap 仍按 M815-B 既有门推进。

merge-owned pending：最终 B 代码/验收/规范通过后，在冻结提交统一裸 cargo t 与最终
diff 所需触面档、完整 CLI/service/tool/profile/来源证据、实际合入与 Spec-ledger 收口。
若另行独立合入 T-03，也须相应冻结门后才可合入；本次审查不授权跳门。代码变化需
受影响复审并在新冻结提交检查证据。tf 仅主检出到期批量。

本轮未整档 t/tv/tt/tb/tu/taa/tf、未 GUI、未修改生产代码/提交/合入/归档/清树。
