# PLAN-816 T-13 / T-04 窄能力受影响复审 R2 — pass

2026-10-22（客户端日期），Plan revision 2，review_scope=phase，仅为 T-04 窄能力
发现及 R1-01 修复的受影响审查。本聊天 Codex /root 独立于实施会话；未创建 agent，
具体模型/会话 ID 未暴露。完整 UI/profile/bootstrap 仍按 M815-B 门推进。

- 原树 `D:/autostack/.wt/lang-816/auto-lang`，`codex/plan-816-dev`，入场/收尾 clean。
- 受审 `08195b67e8652029ac66ce2dc0c42dde844a0fbc`；基线
  `5d8f6ebab6a2a3a3a04ca881795e94b06791a798`，4 文件 +178/-6。
- 实际 main / auto-down 修订及 [40 项输入指纹](inputs.json) 已冻结；Plan hash 取本轮
  簿记前。[冻结 SD](spec-delta-frozen.md) 与 R1 相同，SHA256
  `3c9a1fc2fcc19f32a230d15169e82ef69e5f9c34c9ebe4a00612e89dcf7f63bc`。

**F-816-T04-R1-01 CLOSED，无新增阻断 finding。T-04 窄能力当期代码/验收/规范审查
pass；不代表整 B 验收、完整 UI 交付或已通过冻结合入门。**

## 原反例与相邻控制

本轮沿用 R1 的独立协议宿主源码，字节 hash 相同；未用实施侧新增宿主替换原反例。
[输入身份及旧 client 方法保持证明](replay-identity-proof.json)。
consumer 断言加强为 TeardownFailed、同一候选路径及实际流错误/退出信息，未放宽拒绝
oracle；另加 workspace 首个候选收尾失败时，健康 release 候选未启动的 marker 控制。

[持久回放脚本](replay.ps1)、[最终结果](replay-final-output.txt)：FAILED_PROBE_GROUPS=0。

| 输入 | 当前 discover 实测 | 结论 |
|---|---|---|
| 合法 describe + 干净 EOF | Ok；底层 shutdown Success | 正控保持 |
| 合法 describe + EOF 后坏前缀 | TeardownFailed；reason 含 control stream violation、4294967295 与预算信息 | 原流错误反例闭合 |
| 合法 describe + EOF 后 exit 1 | TeardownFailed；reason=`probe host exit: Failure(1)` | 原内部退出反例闭合 |
| 合法 describe + EOF 后 exit 7 | TeardownFailed；reason=`probe host exit: Failure(7)` | 原异常退出反例闭合 |
| 首个 workspace 候选坏流，release 候选健康 | TeardownFailed；release 的 host-started 不存在 | 无回落 |

[坏流](host-tailbad-output.txt)、[exit 1](host-exit1-output.txt)、
[exit 7](host-exit7-output.txt)、[干净正控](host-good-output.txt) 均包含同输入直接调用
ToolClient 的对照。新的 `.shutdown().map_err(...)` 与 ExitKind 检查分别覆盖 Result
错误和成功返回中的失败分类；路径与实际原因保持，未返回 DiscoveredTool 或继续
其他候选。

新永久回归 `discovery_teardown_failures_are_named_not_ok` 确实调用 public discover，
测试宿主先发合法 DescribeOk，再等待 stdin EOF，随后执行坏流/exit1/exit7 分支；
不是在 describe 前提前失败。四态串行于同一测试，其他 fixture 宿主不使用此模式键。
reason 检查及原例重放共同验证分类和实际原因，不只统计 expect_err。

发现序、缺件、tool_name/协议版本/缺能力、默认邻接安装及早期候选拒绝不回落的 R1
控制本轮全部重跑通过。原 metadata/版本/cancel/stdio/超时/孙进程控制也通过。

## 新宿主的构建来源与测试运行条件

新增 helper 固定从 fixture 的 `target/debug/t04-discovery-host` 读取宿主，需要先在
该布局构建。因此本轮明确执行：

```text
移除审查进程自设的 CARGO_TARGET_DIR
cargo build --offline --manifest-path test/fixtures/modularity/tool-client-consumer/Cargo.toml --bin t04-discovery-host
cargo test --offline --manifest-path test/fixtures/modularity/tool-client-consumer/Cargo.toml
```

[生产者构建](fixture-host-build.txt)、[默认布局完整测试](fixture-default-tests.txt)：
22/22，exit 0。[实际工具/测试宿主二进制 hash](actual-binaries.json) 绑定本次来源。

为排除原工作目录残留，本轮还复制**当前提交的原字节** lib/host 源至全新私有 fixture，
只把 path dependencies 指回当前树、调整包名，无链接；
[源文件同字节证明](clean-source-identity.json)。
[清洁构建/测试输出](clean-fixture-supported-output.txt) 确认默认布局 build=0 / regression=0。

也记录了运行边界：设置共享 CARGO_TARGET_DIR 时，Cargo 将宿主生成到共享目录，
helper 仍查 fixture 自有 target，清洁目录会因缺宿主失败；手工提供该路径及采用
默认布局的对照均通过。这是 fixture 的 target 路径限制；本轮按其明确的默认宿主
布局验证，不声称支持任意 target-dir，也不把审查自设缓存布局的失败追加为本次
发现功能缺陷。后续复用本回归证据时必须保留这个构建前置，不能只依赖旧二进制。

最早使用包外绝对 source path 的诊断副本出现 Cargo 缓存中 manifest 环境路径混用，
记录于 clean-fixture-absolute-source-diagnostic.txt，未据此立 finding。最终清洁副本
把源文件按正常 src/ 布局复制到包内，实际 panic/通过路径与当前 manifest 一致；
没有改动工作树生产代码。

## 健康、闭包与既往闭项

- [client lib](client-tests.txt)：15/15。
- [首次共享 target fixture](fixture-tests.txt) 22/22，但可能读取已有默认路径宿主，
  故仅作诊断；最终完整验收依据为上面的默认布局显式构建后 22/22 结果及清洁控制。
- 三包 check（client/auto/auto-man）exit 0；[诊断](check-diagnostics.json)。依赖/旧
  源码告警保留，不伪称全仓零告警。client 与 fixture all-target clippy 均 exit 0 / 零
  告警：[client](clippy-diagnostics.json)、[fixture](fixture-clippy-diagnostics.json)。
- 包和 fixture fmt --check、git diff --check 通过。
- [777 strict](modularity.json) / [输出](modularity-output.txt)：pass / 0 violations /
  3 grandfathered / 132 边；未放宽配置或刷新基线。
- [verify-deps](verify-deps.txt) 真实执行 PASS，serde_json 已在轻闭包内，新增测试
  宿主未使消费方静态链接 UI/Man/旧语言/VM 实现。
- 真实 M815-A 窄工具仍为 0.1.0 / SHA256
  `6e0c433996a140b94f366e259a6812bbd8fbe33b3747433ad2d5ddd949c61a77`，
  本轮未重编或宣称已发布 M815-B。
- 旧 ToolClient/lib.rs 与 owned 执行设施未变，T-03 未触及；T03-R4 pass、A 的既往
  findings 闭合和发布收据保持。新错误分类属于 discovery 适配器，没有改写底层
  线程/进程清理算法。

## AC、Spec delta 与后续门

AC-03 / T-04 当期窄发现、身份/版本/能力及准确失败出口通过。R1-01 已闭，无未处理
的当期代码/验收项。完整 V-03/UI/profile/bootstrap 按 M815-B 既有门待做，不用本轮
fake 异常宿主代替真实 UI 正控，也不宣称 AC-03 的整 B 全范围完成。

SD-01/05 的 B 发现/真实消费方向与本轮实现一致，失败出口保持已补齐；冻结候选、
spec-impact 精确路径及 GOAL-003 保留。完整 B as-built 文本仍按既有任务收口。
未发布 canonical Specs、修改 live ledger 或重开 A/T03 规范结论。

T-04 当期与当前复用 T-13 标记完成，Plan 维持 `executing`、current_step=7/14。
后续任务按 M814-B/M815-B 实际发布依赖推进；完整 UI 门不能凭本次局部 pass 跳过。

merge-owned pending：B 最终代码/验收/规范通过后，在冻结提交统一裸 cargo t、最终
diff 触面档、完整 CLI/service/tool/profile/来源及 Spec-ledger 合入收口。代码或依赖/
配置变更后重验受影响证据；fixture 宿主须按当前默认布局从冻结源码构建，不能重用
未绑定来源的旧副本。tf 仅主检出到期批量。本轮未整档、GUI、修生产代码、提交、
合入、归档或清树，不等于整 Plan reviewed 或交付完成。
