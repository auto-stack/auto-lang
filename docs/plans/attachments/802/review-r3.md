# PLAN-802 final review R3 — needs_fix

2026-10-17 / plan_revision: 1 / review_scope: final。当前 Codex 会话独立于实施
上下文，沿用 R1/R2 审查上下文；未创建 agent、未修实现、未合入。

## 基线

- worktree `D:/autostack/.wt/lang-802/auto-lang`，branch `plan-802-dev`。
- reviewed_commit `b32d4df7d3a4a43746a22ec4b8d41d1856dc9ab2`。
- base `1dd123361a75aa4b683376dcb2049299122724c9`。
- dependency auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- main 入场 `4755b486ae29a4d8638f8e8ce75d9cc47e353717`。
- 原树 HEAD/依赖在结论前再次核验不变，`git status --porcelain` 入场/末次均空；`git diff --check 1dd123361 HEAD` exit0。
- 本轮 Rust 差异仅四文件 rustfmt（含等价 match arm 括号调整），依赖/测试配置/生成器无变化；原迁移与 renderer bind 均未再改。
- Plan 入场 SHA256 `FB33E1BA02B1735D5D338C50BE6040435281CC6240F1339192B30C8605EC0C76`。
- driver SHA256 `79ED2E4301A70B9C62203557F0AE8FECF7E58087839385F6831BC77D432C04F9`。
- work 实机 JSON SHA256 `2565C59388A9E0960A1BD399658B052AEB3A914D55D719AACD009D655280EF88`。
- 当前 auto.exe SHA256 `D916FF74D4FE7B2D2D1CFBE1B101F41C847E0C29CA7A1EF51981066CB47158F8`，与 work JSON 完全一致；未自行重跑窗口，不把该工作证据声称为本轮独立实机运行。

Spec 输入（均为 `docs/specs/auto-lang/` 下，内容与 R2 相同）：

| 文件 | SHA256 |
|---|---|
| vm/overview.md | 9D792387AB8822D21AED0DEBA5818DCF17BE71A5AA8BE0D9F5242DC3307F0E91 |
| ui/overview.md | F7C54E4ED71BB3598F829F06FE0B97C772ACFE0827B0D81505640B4426A1A160 |
| ui/design/code-editor-session-endpoints.md | 2225A35E4C39E7FC69C57E2177FA728FA0FF4A68C4CF9AAE6B175187C39C52C3 |

## 诊断、旧例与回归

已核对 work 的第二轮失败诊断和闭合表。其对“正向样本不证明 oracle 会
拒绝错误行为”、错误 clamp 区间与 Reader 返回值漏聚合的归因正确；新
Recorder 统一聚合必验失败，新几何对拍会拒绝旧错误上界。这些确有闭合，
不重复要求重新修它们。新计时路径则仍存在下述合同内缺口。

| 检查 | 本轮结果 |
|---|---|
| 6 相关文件 `rustfmt --edition 2021 --check --config skip_children=true` | PASS exit0，review-r3-fmt.log.txt（空输出） |
| 新 driver `--selftest` | PASS，有效场景通过、wrong_bound/missing_reader正确拒绝；review-r3-selftest.log.txt |
| `cargo t plan802 code_editor_natives plan794 plan785 plan728` | 27/27 PASS，6866未选择；review-r3-targeted.log.txt |
| 独立 fixture `cargo test --manifest-path test/fixtures/modularity/code-editor-provider-consumer/Cargo.toml` | 7/7 PASS；review-r3-consumer.log.txt |
| 独立计时负控（下述） | 正确0.5s对照 PASS；2.5s迟到仍 PASS，复现缺陷；review-r3-timing-controls.py / .log.txt |

两次 cargo 命令仅在进程内置空 RUSTC_WRAPPER 避免已知 sccache 端口问题。
未执行裸 t/tv/tu/tf。R2 no-ui metadata 与闭包边界证据可复用：本轮四个
格式文件未改变依赖、feature、catalog、ui_gen 或 consumer 输入的语义。

work JSON 实际有 13 条 gating + 1 条辅助，均记录 true（收据“12+1”计数
有误，修订送审包时一并纠正）；其真实样本 Reader 1.336s 并未测出违约。
本轮 finding 关乎 oracle 可误放行，**不宣称实际 UI 已延迟2.5s**。

## Finding 状态

- **F802-R2-02 CLOSED**：所有上轮点名的新增文件格式门已通过。
- **F802-R2-01 partial**：错误 clamp、缺 Reader 的旧误放行已闭合；完整时序 oracle 尚未闭合，见 R3-01。
- **F802-R1-01 partial**：已有真窗口/滚轮/Reader/几何证据，未再把组件测试当窗口证据；但 V06 的完整合同证明仍待补齐。

### F802-R3-01 — P2，新计时 oracle 放宽冻结的 ≤2s 契约

影响 AC-03/04、V06、T-05/T-06；类别：验收修复引入的判定回归。
位置 `docs/plans/attachments/802/v06-live/v06_driver.py:291–333`。

Plan §5 明确“≤2s用户滚动收敛不变”，规范端点文件57行仍为“用户滚动后
≤2s收敛”。新 driver 把门限改为 `CONVERGENCE_BUDGET_S + quantum`，
而 quantum 取 Reader `scroll_tick` 推进间隔，混入被测通道的刷新节拍，
不等于单次查询的测量不确定度。work 样本把门限实际放宽到3.243s。

独立反例：使用原 `run_oracle`、虚拟时钟及脚本化 Reader，不创建窗口，
分别注入0.5s有效响应和2.5s Reader/endpoint共同延迟。观察请求每0.05s
采样，Reader tick每1.25s推进。后者直到2.55s才采到更新，且tick由2→4
（明确多拍后捕获），仍所有必验 PASS、`rec.ok_all=True`。证据脚本与
日志保存在本目录，命令：

`python docs/plans/attachments/802/review-r3-timing-controls.py`

此脚本 exit0 表示成功复现误放行，日志分别保留 expected_pass 和实际
actual_pass；不是实际编辑器验收通过。

此外，`tick_advanced/next_tick`只写日志，没有进入通过条件；而循环先取
`meta=scroll_meta()`，又取 `off=scroll_pane()`（内部再次查询meta），
故记录的tick和offset并非同一观测。计时t0放在滚轮调用与额外meta查询
之后，读端点一致性又在计时循环结束后才采样，均不能给出端点自实际
输入起≤2s的完整上界。

修正：保留≤2s合同，使用实际输入/处理及端点观测的相关时刻，Reader
offset与tick取同一响应；明确测量误差界。证据不能证明上界时应记录
不确定/待补验，不能把通道延迟加到允许门限后判通过。至少让0.5s有效
对照通过、2.5s迟到对照非通过，并覆盖缺/停滞tick与实际观测身份。随后
在冻结产物补齐所需时序证据及原有几何/缺Reader反控。

程序化同帧投影仍是原合同：新收据诚实撤回虚假的“异步trace证明同帧”
是正确的，但不能据此删除验收要求。源检查确认 renderer.rs:23006 在
同一 drain 消费路径先 `note_controller_offset` 再安排 `__mcp_scroll`，
可保留为实现证据；完整时序验收仍应区分排队、处理帧、投影和实际窗口
滚动，不能用最终150收敛替代全部合同。

## AC / SD 与路由

| AC | 结果 | 证据/边界 |
|---|---|---|
| 01 | pass | no-ui真VM22项、ID与参数、源边界；R2闭包证据按未变条件复用 |
| 02 | pass | 缺席矩阵、单位/IO/delta/ownership及真实adapter定向回归 |
| 03 | partial | 装配/寿命/组件通过；V06时序oracle仍有合同内误放行 |
| 04 | partial | 迁位/剩余边/792闭包保持；完整证据声明须修正 |
| 05 | fail | R3-01未闭合，不能最终pass |
| 06 | pending merge | 未合入/发布/归档/清树 |

SD-01/02/03冻结文字沿用 [R2报告](review-r2.md)“AC与规范增量”三条：
VM overview记录22项owned provider/缺席/寿命；UI overview记录四安装点与
单一registry、其他族仍未迁；端点设计记录provider后的同源实现、单位/
rope IO/原滚动时序保持。目标路径/frontmatter真实goal ID不变。本轮未
批准把≤2s改为2s+量子，未批准取消同帧投影；无canonical/ledger写入。

needs_fix，维持executing；T-01..04保留完成，T-05/T-06重开，current_step=4。
仅重开仍受影响的验收及送审包，不重开已闭合格式任务。下一步原树work
修正计时判定、完成闭合表和必要实机证据，再复审。merge-owned裸cargo t
+cargo tv、实际新增触面档、Spec/ledger/索引/归档/wt-guard清理保持pending；
本轮没有合入或宣称交付。
