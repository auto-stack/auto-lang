# PLAN-802 final review R2 — needs_fix

2026-10-17，plan_revision=1，review_scope=final。审查者为当前 Codex 会话，
独立于实施上下文，沿用 R1 审查上下文；未创建子 agent、未修实现、未合入。

## 冻结基线

- 原树 `D:/autostack/.wt/lang-802/auto-lang` / `plan-802-dev`。
- HEAD `ad1427e3356097f91f97d3b8f76eea1e44ec5e42`。
- base `1dd123361a75aa4b683376dcb2049299122724c9`。
- auto-down sibling `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- 主检出入场 `48ff19821eed299d6966a9a3d64b48f6f48cdefe`。
- Plan 入场 SHA256 `FF5FB353ABD9807007622594E5D1DC7253D892339702AD54E835598FD227842D`。
- 通过有权限的原树命令在入场及末次核对 `git status --porcelain` 均空，HEAD 未漂移；`git diff --check 1dd123361 HEAD` exit 0。普通沙箱的 worktree status 仍不可用，已用提权只读核查解决，非遗留阻塞。

Spec 输入 SHA256（路径前缀 `docs/specs/auto-lang/`）：

| 文件 | SHA256 |
|---|---|
| vm/overview.md | 9D792387AB8822D21AED0DEBA5818DCF17BE71A5AA8BE0D9F5242DC3307F0E91 |
| ui/overview.md | F7C54E4ED71BB3598F829F06FE0B97C772ACFE0827B0D81505640B4426A1A160 |
| ui/design/code-editor-session-endpoints.md | 2225A35E4C39E7FC69C57E2177FA728FA0FF4A68C4CF9AAE6B175187C39C52C3 |

本轮读取当前 Plan/T01/T06、全族 shim/provider/NI/engine/四安装点及新增
renderer bind、fixture 与实际断言；对照原臂和上述规范。新提交仅在原迁移
上增加 VM renderer bind（+12/-1）。bind 与 desktop 臂使用同一 handle
约定，未更改编辑引擎/scroll 算法，但实际窗口验收仍受下述 oracle 缺陷限制。

## 复现与结果

| 检查 | 结果 / 持久证据 |
|---|---|
| 独立 no-ui consumer `cargo test --manifest-path test/fixtures/modularity/code-editor-provider-consumer/Cargo.toml` | 7/7 PASS；review-r2-consumer.log.txt |
| `cargo t plan802 code_editor_natives plan794 plan785 plan728` | 27/27 PASS，6866 未选择；review-r2-targeted.log.txt |
| fixture `cargo metadata --format-version 1 --offline` | 399 packages，auto-lang features=[]，无 iced/wgpu/winit/cosmic-text/arboard；review-r2-metadata.json.txt 及 summary |
| 改动边界 | Cargo.toml 仅增加 exclude；crate manifest、catalog、ui_gen 无改动；本族 shim 无 UI 实现直引，diff/dnd 等剩余边保留 |
| 格式 | rustfmt --edition 2021 --check --config skip_children=true（6 个相关文件）exit 1；review-r2-fmt.log.txt |
| V06 driver 负控 | 有效对照 exit0；错误 clamp exit0；Reader 空对象记录 FAIL 但仍 exit0；review-r2-driver-controls.py / .log.txt |

首次 consumer 重放因 sccache 服务端口 WSAEACCES 未编译；随后仅在命令
内置空 RUSTC_WRAPPER，重放通过。此为环境问题，不作实现 finding。
定向构建仍有仓库既有警告，本轮输出未定位到新 provider/test 文件的新增
警告；不能以“所有警告已清零”表述。未运行裸 t 或整档 tv/tu/tf。

## Finding 闭合与新增问题

### F802-R1-01 — partial，仍开放（AC-03 / V06 / T-05、T-06）

旧缺口已有真实进展：提交补上 VM renderer bind，新增窗口运行日志、
截图及用户滚轮轨迹。不是继续“完全未跑实机”。但新证据尚不能证明完整
V06，原因见 F802-R2-01；不授全不变量闭合。

### F802-R2-01 — P2，V06 验收探针可把必要观察失败判为成功

位置 `docs/plans/attachments/802/v06-live/v06_driver.py:346,382`，影响
AC-03、AC-04，T-05/T-06。类别：原必验 oracle 的补验缺陷。

1. clamp 只检查 `150 < oy_max <= oy_max2+0.5` 且 `<99999`，没有按真实
   content/viewport 计算上界，没有核对 reader 实际 offset。独立负控保持
   content=8020、viewport=479.2，却令两次回读均为 2000（正确上界应约
   7540.8），仍记录 clamp PASS、main exit0。现有一条 7541 的正确样本
   不能证明该 oracle 会拒绝错误行为。
2. `scroll-state-reader-observe` 的 `record` 返回值没有并入 `ok_all`，
   且只做字符串包含检查。独立负控让 Reader 返回 `{}`，实际记录
   `ok=false`，最终仍 `ok_all=true`、exit0。无法用该 exit0 支持 Reader
   必验通过；原收据“8/8”也不应代替逐项结果（JSON 实有 9 条含截图）。
3. 收据“程序化150命令值直达、无中间值”与原始 JSON 不符：trace=
   `[(0.135,0),(0.773,0),(1.431,0),(2.047,0),(2.249,150)]`。
   driver 仅要求最终 150 且≤3s，没有验证“同帧命令值投影”或窗口可见
   offset。这里不据异步采样直接判运行时违约；要求用动作/帧身份与真实
   Reader 几何消除采样/处理延迟歧义，不能把最终收敛写成同帧证明。

修正：真实运行中解析指定 pane 的 content/viewport/offset，以明确的
浮点/像素容差对拍上界与实际偏移；所有必验失败进入最终非零退出；用
关联动作/帧的观测验证程序化投影与用户滚动计时并如实描述覆盖边界。
先重放有效、错误上界、缺 Reader 三控，再在冻结产物上补跑实机，记录
代码/构建产物身份、输入、原始响应和计时。负控使用 mock 是为了验证
driver 判定逻辑，**不**作为编辑器运行验收。本轮未自行重跑窗口/真滚轮，
未将 work 的窗口结果冒充独立运行结果。

driver SHA256 `B0F612527A13883B4024073643C63ED2F32ECF2A0C18271E0DC7B72DAEEDAE3A`；
原 JSON SHA256 `3DA9BE24C5E9B0E18988729E729ABA7652443A59365ADC7B6FDDECC9CF705AEE`。

### F802-R2-02 — P2，新增代码未通过格式门

影响 AC-05 / T-02、T-03、T-04 及依赖收据 T-06。类别：交付健康检查。
定向 rustfmt 检查在新 `ui/code_editor_provider.rs:118/285/298`、
`tests/plan802_code_editor_provider.rs:189/317/400/426/576/590`、
`vm/host_capabilities.rs:459` 与新增 consumer 测试多处失败。
这些位置是本计划新增内容，不能归因于已有整仓格式债。详见原始日志。
原树按格式器规范修正，提交后重验相关文件；review 未代修实现。

## AC 与规范增量

| AC | 结果 | 覆盖和边界 |
|---|---|---|
| 01 | pass（功能） | 22 native ID/值/互异参数经 no-ui 真 VM；metadata 与源检查删本族实现边 |
| 02 | pass（已声明迁移路径） | 原臂对照、无 provider 21错误+scroll false、Unicode/负参/IO/delta/ownership 相关回归通过；未声称所有 future provider 实现已验证 |
| 03 | partial | 快照/实例/Clone/merge/真注册表测试通过；窗口 V06 oracle 不完整，R1 未全闭 |
| 04 | partial | 本族边界、未动 a2r/生成器和无新增闭包依赖已核；V06 证据包声明需修正 |
| 05 | fail | 必验 oracle 与格式问题未闭 |
| 06 | pending merge | 未合入、未发布规范/ledger、未归档/清树 |

冻结 SD 提案快照（未发布）：

- SD-01 modify `docs/specs/auto-lang/vm/overview.md`：在既有 779/785/790/794
  装配规约后加入 editor 22项→19具名 provider、owned DTO、构造快照/实例
  覆盖、Clone保留/merge保目标、binding身份不变、缺席矩阵和单次delta。
  AC-01/02/03。已有其他宿主族和尚存 UI 边必须保留。
- SD-02 modify `docs/specs/auto-lang/ui/overview.md`：记录四安装点的
  code-editor 门控 if-absent adapter、剪贴板实现级门控、单一 editor
  registry；不声称 term/media/dialog/dnd 或整体 renderer 已独立。
  AC-03/04。其他并行规范事实不得覆盖。
- SD-03 modify `docs/specs/auto-lang/ui/design/code-editor-session-endpoints.md`：
  VM 经 provider、UI adapter 与 a2r 仍委托原实现；save rope字节、光标单位、
  scroll写入/投影/≤2s合同保持，并记录 VM renderer 与 desktop 同源绑定。
  AC-02/03。新 driver 的3s最终采样预算不得沉淀为替代同帧语义的规范。

上述结构性提案与实际代码方向一致，但涉及滚动验收的完成声明须待 R1/
R2-01闭合；本轮不授可直接发布的完整 SD pass。frontmatter 三个精确路径
与 GOAL-003/007 保留，无新增组件；未修改 canonical Specs/live ledger。

## 状态与下一步

needs_fix，维持 executing。T-02..T-06 重开（格式触及 02/03/04、必验触及
05、送审收据06依赖它们），current_step=1/8；保留已经通过的定向测试
证据与历史工作描述，不能把重开理解为功能全部无效。T-07/T-08仍待办。

本次为第二次 non-pass。下次送审前 work 需要给出闭合表及诊断：为何
仅最终值+字符串判定漏掉必验，如何以负控与帧/几何观测覆盖完整不变量，
并区分旧例闭合、全不变量闭合和修复回归。原树修复、提交后复审。
merge-owned 裸 cargo t + cargo tv、实际新增触面档、规范/ledger/索引/
归档/guard 清理均 pending；未触 ui_gen/aavm，不新增 tu/taa 要求。
