# PLAN-816 T-13 / T-04 窄能力受影响复审 R1 — needs_fix

2026-10-21（客户端日期），Plan revision 2，review_scope=phase，仅检查本次 T-04
窄能力发现；完整 UI/profile/bootstrap 仍按 M815-B 门推进，不作为新增缺项。
本聊天 Codex /root 独立于实施会话，未创建 agent；具体模型/会话 ID 未暴露。

- 原树 `D:/autostack/.wt/lang-816/auto-lang`，`codex/plan-816-dev`，入场/收尾 clean。
- 受审 `5d8f6ebab6a2a3a3a04ca881795e94b06791a798`；基线
  `5720a90594b8e5964b82c3772c2f52814597cbb9`，3 文件 +387/-0。
- main `5e5eddceb49e4e45ba245bb03ea3a5ca947891e9`，auto-down
  `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。本次改动尚未合入 main。
- [34 项输入指纹](inputs.json)、[冻结 SD](spec-delta-frozen.md)，后者 SHA256
  `3c9a1fc2fcc19f32a230d15169e82ef69e5f9c34c9ebe4a00612e89dcf7f63bc`，
  与 T-03 相同，Plan hash 在本轮簿记前采集。

**新增一项 P2：discovery 忽略探测进程关闭时的流错误及失败退出，仍返回已发现成功。**
原 21 条 fixture、15 条客户端测试均通过，但未覆盖这个新组合入口的错误传播。

## F-816-T04-R1-01 [P2]：丢弃 shutdown 结果，将坏控制流/内部失败宿主认证为成功

影响 T-04 / AC-03 当期，关联既有 AC-02/04 的具名错误、stdio/退出分类保持合同。
位置：`crates/auto-tool-client/src/discovery.rs:204`。

依据：Plan §5.2 的进程/stdio/exit 与准确错误出口、V-03 的坏结果/崩溃控制，及已发布
`tool-execution` / `tool-protocol` Spec 的流错误不可吞、EOF 与失败退出分类。这个要求
适用于当前窄 `describe` 探测，不依赖未来 generate/build/run。有效 descriptor 不能
消除同一探测进程已被客户端观察到的流错误或内部失败。

新 `verify_tool` 调用 `client.describe()` 后执行 `let _ = client.shutdown()`，随后只检查
descriptor 并返回 `Ok(DiscoveredTool)`。这里同时丢弃两层真实结果：

- `shutdown` 返回 `Err(ClientError::StreamViolation(...))`；
- `shutdown` 返回 `Ok(Shutdown { kind: Failure(code), ... })`。

独立可重放输入：[协议宿主](probe-host.rs)，按真实 framing 回答一帧合法 DescribeOk，
tool 名称/协议版本/所需两个操作均匹配。收到 stdin EOF 后分别发送坏帧前缀、以规范
内部失败码 1 退出、以异常码 7 退出。好宿主在相同输入下 EOF 后 exit 0。坏流特意在
EOF 后发出，保证测试准确到达关闭阶段，不以抢在 describe 前污染造成的另一出口
冒充此缺口。

[consumer](discovery-probe.rs) 对每个宿主分别调用真实 public `discover` 与底层
`ToolClient`，做同输入对照。两者均 spawn 真实 OS 进程；未修改工作树代码。

| 输入 | 底层 describe / shutdown 实测 | discover 实测 | 预期 |
|---|---|---|---|
| 合法描述 + 干净 EOF | describe Ok；shutdown Success | Ok | 接受，正控通过 |
| 合法描述 + EOF 后坏前缀 | describe Ok；shutdown Err StreamViolation(TooLarge, observed=4294967295) | **Ok** | 具名拒绝并保留流错误 |
| 合法描述 + EOF 后 exit 1 | describe Ok；shutdown Failure(1) | **Ok** | 具名失败并保留实际退出分类/码 |
| 合法描述 + EOF 后 exit 7 | describe Ok；shutdown Failure(7) | **Ok** | 具名失败并保留实际退出分类/码 |

[坏流输出](host-tailbad-output.txt)、[内部失败 1](host-exit1-output.txt)、
[异常退出 7](host-exit7-output.txt)、[干净正控](host-good-output.txt)。
[回放脚本](replay.ps1) / [最终输出](replay-final-output.txt)：3 个拒绝断言失败，
代表同一 finding 的三个控制，不是三项 finding。

根因由新 wrapper 的实际代码与底层对照确认。这里没有证据表明旧 ToolClient 吞错，
也不声称这些例子留下了活的 owned 进程或线程；缺口是 discovery 成功结果丢失了真实
收尾失败。旧 A 的执行设施/客户端已发布结论保持有效。

修复与闭合 oracle：处理 shutdown 的 `Err`，并核验成功返回中的 ExitKind；探测失败
应具名带路径、原始原因/退出信息返回，不能返回 DiscoveredTool 或回落到其他候选。
在真实 discover 入口补上述坏流、非零退出与干净 EOF 永久正反控制，保持身份/版本/
缺能力错误和原 stdio/cancel/超时/树收尾控制。具体错误 variant 可由实现选择，不规定
新的任意字符串或数值超时承诺。

## 发现序、身份与负控质量核验

本轮按用户给出的窄面序验证 env → workspace debug/release → 邻接安装。
所有候选冲突控制都在新建私有目录复制测试二进制，无 junction/symlink，没有改写
实际安装工具或 workspace 的真实目标二进制。

- env 合法路径优先于 workspace 中实际存在的候选。
- env 指向缺失文件时，即使 workspace 有可用候选也返回 EnvPathMissing。
- workspace debug/release 同时存在，选 debug；debug 缺失后选 release。
- 两者缺失时，复制到独立安装目录的 consumer 选择邻接工具；清除运行期
  CARGO_MANIFEST_DIR、workspace_root=None 时该安装正控也通过。
- 首个存在的 workspace debug 候选身份错误，具名 VersionMismatch；健康 release
  候选未启动，以其 `host-started` 标记不存在核验，未静默 fallback。
- 独立错误 tool_name、错误 descriptor protocol_version=99、缺 validate_project 能力，
  分别严格断言 VersionMismatch / VersionMismatch / CapabilityMissing。
- 原真实 M815-A fixture 核验两个窄操作，未将 fake host 用作完整 UI 正控。

原新增 5 条永久测试覆盖基本成功与缺件/缺能力/版本分支，未覆盖“describe 已成功但
探测收尾失败”的组合，故 21/21 绿不能关闭 R1-01。没有把未实现 UI 能力或未发布完整
profile 算为本轮缺陷。

模块头仍提到组内兄弟/主检出路径，实际函数仅使用 env、workspace target、邻接 exe；
宜随修复同步说明。本轮不据此要求增加用户明确禁止的隐式搜仓。

`delayed` 宿主也保留为诊断：底层显式设 250ms 时超时，discover 使用默认无请求截止
而等待真实回答。当前调用方策略未约定这个新入口的数值超时，**未据 1.6 秒等待另立
finding，也未把 250ms 追加为产品要求**；本轮唯一阻断是确定的收尾错误丢失。

## 原测试、健康与来源

- [client lib](client-tests.txt)：15/15；[实际独立 fixture](fixture-tests.txt)：21/21，
  包含原 describe/validate、协议拒绝、cancel、stdio 分离、超时与孙进程寿命。
- 三包 check（auto-tool-client / auto / auto-man）exit 0；
  [原始诊断](check-diagnostics.json)。依赖/旧源码告警保留，不声称全仓零告警。
  改动包 all-target clippy exit 0 / 零告警，[诊断](clippy-diagnostics.json)。
- 包与 fixture fmt --check、git diff --check 过；
  [777 strict](modularity.json) / [输出](modularity-output.txt)：pass / 0 violations /
  3 grandfathered / 132 边，无 manifest/基线/规则变更。
- [实际 verify-deps](verify-deps.txt) 通过：consumer 闭包仅轻协议/数据/执行设施，
  无 auto-ui-tools、auto-man、auto-lang、auto-vm、auto-gen、auto-media-core。
  未新增 build.rs、UI 实现依赖或隐式下载/构建。
- 实际窄工具为现有 `auto-ui-tool 0.1.0`，SHA256
  `6e0c433996a140b94f366e259a6812bbd8fbe33b3747433ad2d5ddd949c61a77`；
  [两个真实二进制指纹](actual-binaries.json) / [version](tool-version.txt)。
  本轮没有重编 UI 工具，测试证明消费的是该已存在窄面二进制，不声称消费了未发布 B。
- [历史范围证明](prior-scope-proof.json)：T-03 所有受审实现输入零漂移；原 client
  文件去掉新增 module/re-export 三行后与基线相同。T03-R4 pass 与 M816-A 发布保持，
  本轮不重开其 findings。
- 探针准备时 PowerShell 的只读 Host 变量名冲突已修正为专用变量；默认 bash 返回
  127，改用明确 Git Bash 后 verify-deps 真跑通过。环境/脚本准备问题不作为产品
  finding，原壳环境记录在 verify-deps-shell-environment.txt。最终回放使用全新私有
  scratch，排除了旧候选 marker 干扰。

## AC、Spec delta 与后续门

AC-03 / T-04 当期发现序与基本拒绝通过，异常探测收尾结果保持失败，整体 needs_fix。
与 AC-02/04 关联的当期错误分类未闭不等于否定已发布 A。完整 V-03 的 UI 正控、
profile/bootstrap 按既有 M815-B 门待做，不能用本次窄面测试替代。

冻结 SD 候选保持：SD-01/05 的 B 发现/消费方向成立，R1-01 修好前不能证明真实工具
失败的准确受控出口；SD-02/03/04 和 T-03 结论无本轮变更。精确 spec-impact 路径与
GOAL-003 保留。未修改 canonical Specs 或 live ledger，不把 discovery 库新增当作已
发布薄 CLI 或完整安装 profile。

下一步回原树 work 修正这一项并补实际入口正反控，提交后受影响复审。
T-04 及本次复用 T-13 重开，`executing`、current_step=5/14；T-03 已通过、A 的
T-01/T-02/T-12/T-14 保持完成，历史 T13 A/T03 pass 收据保留。当前不推进依赖此门的
后续实施；完整 UI 门也仍按 M815-B。

merge-owned pending：B 最终代码/验收/规范通过后统一冻结裸 cargo t、最终 diff 触面
档、完整 CLI/service/tool/profile/来源和 Spec-ledger 合入收口；代码变化后重验受影响
面，tf 仅主检出到期批量。本轮未整档/GUI/修生产代码/提交/合入/归档/清树。
