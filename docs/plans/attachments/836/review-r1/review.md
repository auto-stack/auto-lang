# PLAN-836 final review R1 — needs_fix

2026-10-23；plan_revision 1；独立审查上下文 Codex（本聊天未参与实施；未创建子 agent，也不声称另一个模型）。使用主检出当前 Plan、同一实施树的提交代码、当前主检出 canonical Specs 重建结论。

- 实施树：`D:/autostack/.wt/lang-836/auto-lang`，`plan-836-dev`。
- reviewed_commit：`87fb0b051d8ab9b17fba0fcfe2580c774a79d058`。
- base / 当前 autographics-dev：`2a7b10250245332f46b7867dc140da1a4f9d0969`，835 已归档；SDK `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`。
- 入口/结束实施均 clean；审查期间只在 research 生成审查输入/日志，补充 Rust 输入已迁存本附件，没有实施修复。全部 implementation diff 位于 driver 子树（22 文件）；AGP、SDK、原资源、根 workspace/lock、profile1/2 源码未改。
- Plan 输入、Spec/依赖/技能指纹见 [baseline.json](baseline.json)，原始 Plan 副本 [plan-input.md](plan-input.md)。

## 结论与任务路由

**needs_fix**，存在一项原合同内 P2：F-836-R1-01，原生验收工具在未完成时可能成功退出。T-07 重开，依赖最终验收、身份及准确交接的 T-08 重开；T-01..06 保留完成。Plan 返回 executing 6/10；T-09 未通过，T-10/AC-09 未开始。登记债务不能豁免本项。

本次不修实现、不合入、不发布 canonical/ledger、不归档/清树。修复在原树完成并提交后，再复核受影响的原生 gate、失败出口和交接身份。

## F-836-R1-01 / P2 — incomplete interaction check exits successfully

- **类型 / 影响**：原验证合同缺口；AC-05/AC-07，V-09/V-10，T-07/T-08。Plan §6 要求真实 OS 正负门成立，wrong-target/drop-release 同门负控须非零；canonical prototype-profile 也约定 skip 即失败。未到验收断言的 gate 不能被退出码判为通过。
- **代码**：`experimental/autographics-auto-driver/src/bin/autographics-authoring.rs:281`，连同 `197` 的帧上限出口。`--interaction-check` 创建 `native_probe`，不设置旧 `probe` 标志；run_app 返回后的未完成保护仅检查 `app.probe`。这是已核实的控制流原因，不是鼠标环境假设。
- **固定输入 / 入口**：在上述冻结树的 `experimental/autographics-auto-driver` 运行：

```powershell
./target/debug/autographics-authoring.exe --interaction-check --frames 1 --report research/viewport-review-r1-truncated.json
./target/debug/autographics-authoring.exe --interaction-check --sabotage-target --frames 1 --report research/viewport-review-r1-wrong-truncated.json
./target/debug/autographics-authoring.exe --interaction-check --drop-release --frames 1 --report research/viewport-review-r1-drop-truncated.json
```

- **预期 / 实际**：三者都未完成任何 interaction case，必须失败或明确拒绝不兼容参数。实际全部 exit **0**、`frames=1`、`passed=false`、`error=null`、records **0**。反控甚至还没到预期错误 phase，就已给调用方成功退出码。原始报告及 [汇总](viewport-review-r1-incomplete-cases.json) 已保留。无正门鼠标噪声依赖：frame1 在自检首次输入 frame12 之前。
- **合法 / 兄弟控制**：相同帧上限的旧 `--window-probe --frames 1` exit **1**，证明不是“所有帧上限模式都允许未验收成功”的继承合同；旧报告在附件。普通 observational `--interaction-probe` 的有界退出可保留，不应混同 self-checking 模式。clean CloseRequested 等在完成前正常返回的出口也会到达同一遗漏保护；本次已动态确认的反例是帧上限，未把其他出口冒称已动态覆盖。
- **关闭 oracle**：给 self-checking interaction 模式统一的完成性/退出传播保护；上述三输入全部非零或参数拒绝，报告保留可定位未完成原因；有界 observational 模式仍合法。完整正门应 exit0 且 passed=true/所有 required records 到齐；未截断 wrong-target/drop-release 保持原断言阶段非零。覆盖完整/部分/正常提前退出三类状态，不只特判 `--frames 1`。冻结新代码并刷新 manifest/交接后再送审。

## 验收复核

| AC | task / code / artifact | 结论与证据 |
|---|---|---|
| AC-01 | T01/02/08；source verifier、225 来源 manifest、Cargo/SDK/资产 | pass：fresh verify_source exit0，680 SDK 文件、479 closure、17 local packages/29 edges，225 源/资源 hashes。Git diff 限 driver；AGP/root/SDK/资产锁未漂移。独立打包源消费者 work 证据按同一输入指纹检查，完整 consumer 重跑仍属 merge 门。 |
| AC-02 | T03/06/07；pick2、Host pointer/selection、Panel | 几何/顺序/clip/DPI/纯选择与实际物理鼠标路径通过；原生自检完成性未通过，因此整体 native 证明保留 partial。7 interaction+7 host 测试，另有实际窗口 circle dx40/dy20、一次撤销，文档/历史/dirty 数值和画面一致。 |
| AC-03 | T03/05/07；all-mesh bounds、pick3、TRyRxS、axes/prepare | 几何和 Host 三轴运行通过；完整 native 独立重放本轮未完成，partial。补充 actual camera 近远/隐藏/深度相同ID/union空隙 holdout 通过；已有几何 NDC/首mesh实际源码反控确到断言，未借构建失败作红。源实现用实际 Scene3dContent camera、NDC0/1有限线段与统一参数t；精度/禁用轴提示公开。 |
| AC-04 | T04/05；Gesture→Building→checked output→Prepared→commit_gesture | pass：临时画面而 accepted 不变、1000 moves 合并、每轴目标/非目标数值、最后端点/Auto mismatch/资源失败回滚、重复release/no-op、一次Undo/Redo。另补非法最终坐标恢复相同accepted capture、历史准入失败保留redo，并验证下一合法操作。 |
| AC-05 | T04/06/07；真实事件分支、captured CompositionInput、busy/cancel | fail：F01使真实OS验收工具存在未执行成功出口。原取消/跨面板/zero-size/resize/camera/stale/busy/dirty-close定向通过保留；不把原生受干扰失败判为产品取消缺陷。 |
| AC-06 | T04/06/07/08；schema/Storage/弱owner/mesh pin | pass（受整体native门限制）：storage 3、20实际VM/mesh owner周期及历史拒绝 holdout通过；resident7 meshes/14 writes/136952B，Arc/target身份不变、外部pin last-owner成立。新进程inspect原已保存文件，hash `9db84cb370bdb59104de742161376420ff93b4bcee2778840ce61e35ee64a534` 完全一致，selection=null/history=[0,0,0]/dirty=false。持久schema未改。 |
| AC-07 | T08；README/research/SD01..04 | partial：Spec目标/支持面/来源边界合理，F01修复及native重放结果、最终来源manifest/准确交接仍需刷新。 |
| AC-08 | T09；本轮独立验收 | 未通过，needs_fix。 |
| AC-09 | T10 | pending，merge-owned；没有交付结论。 |

## 实际验证、原生边界与健康

- `cargo test --locked --test authoring_interaction --test authoring_viewport_feasibility --test authoring_host --test authoring_session --test authoring_storage -- --nocapture --test-threads=1`：22 passed，0 failed/ignored，见 [fresh log](viewport-review-r1-targeted.log)。未运行完整 driver/AGP 或 root 整档。
- [独立 supplemental 输入](viewport-review-r1-holdouts.rs) 使用当前已构建 rlib，rustc --test 在实施树 research 中构建运行：4/4，通过 [log](viewport-review-r1-holdouts.log)。分别为阈值/最终no-op/合法恢复，非法最终端点/像素恢复，history预算拒绝/redo保留，实际camera下OBB近远/隐藏/union空隙。最初输入类型推断和直接rustc缺native link搜索路径属于审查夹具准备错误，修正后才执行；不计产品失败或负控。
- `cargo check --locked --all-targets` exit0；`cargo fmt -- --check` exit0；`git diff --check base..HEAD` exit0。SDK 146 个既有 warnings；无新增 driver/AGP warnings。最初误用 `cargo fmt --all` 会进入 SDK vendor 多workspace而报metadata错误，改为 README 所用本workspace fmt 后通过；不是格式失败。
- 实际窗口使用 computer-use：物理拖动把 circle 从[-55,0,0]→[-15,-20,0]、revision1/history1/dirty=true；一次真实Undo→[-55,0,0]、revision2/history undo0 redo1/dirty=false，随后clean关闭。[原生记录](viewport-review-r1-manual.json)、[撤销后的截图](viewport-review-r1-manual.png)。这些补充不会替代完整原生3D/取消矩阵。
- 独立普通 `--interaction-check` 三次未取得完整正门：两次在phase1按下后手势已丢，另一次走到phase9超时。原始报告 `positive*.json` 保留；第三次 inputs 混入非fixture预设的真实指针坐标和额外 down/up，不能据此宣称正式逻辑回归，也不能记录pass。初始两次无足够取消原因字段；原因未独立确诊。实现阶段成功报告同SHA与本次Host三轴/几何通过可作辅助，但本次自动原生完整路径不外推通过。后续重跑应在无额外桌面输入的环境记录完整正门；不要修改合法取消行为来让夹具过门。
- work 原 near-depth/first-mesh/retained-candidate/reupload-mesh source mutations 报告均检查到相应断言，restored控制绿；在同代码、同依赖、相同配置/输入指纹下作为既有敏感性证据引用，本轮未在暂停实施写入期重复变异源码。其结论只覆盖对应缺陷，不覆盖F01。
- 已检查完整 diff 的调用点、持久字段、真实Auto→render数据流、失败出口、owner释放、容量、历史/dirty、UI焦点/命令互斥；未发现新增未披露scope reduction或Rust移动后回填Auto旁路。probe中的资源故障注入有显式模式标记。没有把后续I2/I3/三角形精确拾取变成当前要求。

## Spec delta 与后续门

冻结 [SD01..04](spec-delta-frozen.md)，SHA256 `76b1201dde4b2088cd78431eeb3629316e3b8daccf76a3d2254364712025d84f`。新增viewport-interaction路径在主检出尚不存在，符合add；其余5个modify目标和GOAL-007/008元数据已核。schema1不变、OBB近似、正交世界轴、有界真实Auto临时owner、一次提交、取消/dirty/本地保存和计量排除与实现主体一致。SD04的可复跑原生验证承诺须在F01修复、证据更新后再终审；未发布规范，不降低既定合同。

**pending_closeout_gates**：独立final pass后，由merge在同树冻结提交统一执行完整locked driver/AGP test+build、原16 case、profile1/2双游戏/835原生正负门、新interaction完整正负及reopen、两道counter/content合成门、来源/资产/独立consumer；实际落autographics-dev、canonical/ledger/archive、guard清理和批量到期检查仍属T10。root t/tv/tt/tb/tu/taa/docs_gen本轮未运行，也不因本实验写面自动扩为root门；tf只在主检出到期批量执行。

next：work在同树修F01、补永久完成性断言并刷新T07/T08冻结交接；再按新提交受影响复审。
