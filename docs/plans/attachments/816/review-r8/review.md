# PLAN-816 T-13 R8 — needs_fix

2026-10-21，revision 2，review_scope=phase / M816-A。本聊天 Codex /root 独立于实施会话，
从冻结代码、旧共享输入和失败分支重建结论；具体模型/会话 ID 未暴露，未创建 agent。
沿用 R1–R7 历史，不将同实施会话内的 R3/R4 检查当作新独立上下文。

- 工作树 `D:/autostack/.wt/lang-816/auto-lang`，分支 `codex/plan-816-dev`，入场/收尾 clean。
- 受审 `2d21cff8c3eb813fbce389d5449e0bb0311f1355`；修复基线 `05d3940c6f57254bf74027079d4d00ac70ca8bac`，两文件 +68/-33。
- 阶段基线 `b65d0e963bbf39f97de5c1d6531997f6f21c760b`；M815-A `e609caf1af511b96832f511ab9d1fa6c0c11f563` 为祖先，tool/protocol/model 三包源零漂移。
- 兄弟 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- [97 项输入指纹](inputs.json)；[冻结 SD 候选](spec-delta-frozen.md)，SHA256 `cbc62a16540f5476868fb8eb9eb68b5cbd6d3999d81920d4875f4f49fdd64e59`。Plan 指纹在本轮簿记之前采集。

| Finding / 子项 | R8 结论 |
|---|---|
| F-816-R7-01 / Linux 两条新增告警 | **CLOSED**，真实两包/consumer check exit0，净新增告警为0，仅保留三条已识别旧告警 |
| F-816-R3-02 / 有效凭证下的取消时序 | **子项 CLOSED**，保留取消凭证并重复取消，入口前/内层检查后/已挂起写控制通过 |
| F-816-R3-02 / 客户端终止失败无界 wait | **子项 CLOSED**，两调用点接入 cancel_within；故障注入确认具名且有界返回 |
| F-816-R3-02 / 取消凭证初始化失败 | **仍 OPEN / P2**，`.ok()` 和 Clone 空句柄仍丢取消能力，两故障状态均在返回时留下 writer |
| F-816-R6-01 / Linux 编译、F-816-R5-01 / stop 终态 | 保持 CLOSED；无新增独立 finding |

## 唯一未闭项：F-816-R3-02 的取消凭证失败分支

影响 T-02/T-12、AC-02 当期/AC-07。此分支已列在 R6/R7 原 finding 下，本轮把源码
残项补为可直接重放的故障状态证据，没有增加新规模、平台或执行合同。

本轮交接明确说该分支“未改”，但解释为“凭证是非承力冗余，真停由重发取消+
入口拒写承担”。代码并不支持这个解释：

- `auto-tool-execution/src/lib.rs:1332`：`CancelHandle::from_stdin(&inner).ok()` 抹掉
  DuplicateHandle 错误，仍交付 `StdinPipe{cancel:None}`。
- `win_stdin.rs:75–77`：Clone 的 DuplicateHandle 失败返回 `CancelHandle(NULL)`。
  因而 `cancel_handle()` 可给控制方一个无效凭证，调用方仍正常启动 writer。
- `lib.rs:973–987`：重发取消只在 retained 存在时调用该凭证；None 时不取消，
  NULL 时反复 CancelIoEx(NULL) 而忽略 false。入口 stop 检查不能中止已经挂起的 write。
- client `:325/:733` 取得并重试同一凭证，没有独立兜底能力。一个有效凭证上的
  多次调用并不是多套独立取消机制。

独立反例 [replay-credential-failure.ps1](replay-credential-failure.ps1) /
[测试源码](credential-failure-injection.rs) 在 TEMP 复制**未修改的生产 lib 前缀**，
只追加测试；支持模块也保留生产前缀，仅追加构造 Clone 既有 NULL 结果的测试函数。
[前缀身份及故障模型说明](fault-state-proof.json)。没有修改实现树。

两个测试状态分别对应生产失败出口：

1. 初次复制失败：在边界注入 `pipe.cancel=None`，与 `.ok()` 的实际结果相同。
2. 控制方复制失败：将停止侧凭证设为 `CancelHandle(NULL)`，与 Clone 的实际结果相同；
   写线程中的 StdinPipe 仍持有效原句柄，证明原句柄存在不等于停止侧有取消能力。

均用真实 te-child、DirectChildOnly、有限寿命5秒的非owned后代持 stdin 读端；先成功
预填64KiB，再观察生产 StdinPipe 的真实阻塞写，终止/收尸根后执行生产 stop_and_join。
实际结果 [原始输出](credential-failure-output.txt)：

```text
credential=missing; outcome=Err(PipesHeld { after: 3s }); writer_alive_at_return=true; elapsed_ms=3000
credential=null;    outcome=Err(PipesHeld { after: 3s }); writer_alive_at_return=true; elapsed_ms=3000
```

一个测试函数收集两个状态结果后统一断言，cargo exit101；两者均在失败断言之前
等有限持有者退出并 join writer，不留下测试线程。这里是**凭证失败结果的边界注入**，
未声称真的耗尽 OS 句柄或让 DuplicateHandle 在系统中失败。生产失败出口的可达状态
来自上述源码。没有替换生产 write、cancel、Worker 或错误分类。

测试不是要求“无取消能力也必须魔法停止”：初始化/复制若失败，可以在启动 writer
前具名拒绝并清理；或者采用具有可验证兜底能力的实现。应避免静默降级，不能用
PipesHeld、延后释放非owned后代或“冗余”说明替代原来的真停/错误传播合同。
保留有效凭证的三个时序控制，并为两失败状态补永久负控、核对 run/client 两消费方。

## 本轮确实修复的部分

停止侧现在从共享槽取出并保留凭证，在线程未结束期间每切片重发取消，最后 join。
源码中有实际重复调用，解决了有效凭证下“检查通过但尚未入核”的一次取消窗口。
[R6 入口前栅栏](r6-race-output.txt)、[R7 内层检查后栅栏](r7-after-check-output.txt)
通过，后者100ms返回 Stopped、writer不存活；R5 run/client 已挂起写控制通过。
共享槽 None 现在表示所有权转移到停止侧本地，**不再表示句柄已关闭**；本轮依据
实际重试循环和线程结束结果判断，不将旧 probe 的注释当成新的实现约束。

client 超时 `:618`、shutdown 宽限分支 `:804` 均改为 `cancel_within(TEARDOWN_GRACE)`；
run 的 bounded_cancel 也委托给它。此 helper 使用终止/带限 wait，不再进入无界
child.wait。独立 [终止失败回放](replay-termination-failure.ps1) 在 TEMP 仅对指定
真实子进程的 terminate 边界强制返回 PermissionDenied，实际0ms具名返回，恢复
真实调用后终止/收尸成功：[结果](termination-failure-output.txt)。这是函数边界
故障注入，不伪称实际 OS 权限故障。未借恢复旧无界 cancel 来清理该测试。

Linux 字段/局部 cfg 已收敛；[同目标新旧编译](platform-output.txt) 均 exit0，均仅有
原 unused_mut/POLLIN/POLLOUT 三条，F-R7-01 闭合。Windows 健康检查同样通过。

## 验收、验证与规范

| AC / 任务 | 当前结果及证据 |
|---|---|
| AC-02 当期 / T-02 | partial：正常/非零、stdio/双向流量、deadline、owned树、普通取消和错误终态通过；凭证失败仍漏 writer |
| AC-07 / T-12 / V-05 | partial：真实窄工具、版本/能力/预算负控、流/日志、身份与独立闭包通过；资源失败退出未全闭 |
| AC-08 / T-13 | fail：仍有一项原合同内 P2 未闭 |
| AC-09 / T-14 | pending，阶段 pass 后的 merge-owned 冻结门/实际发布 |
| 原 AC-01..04 B 其余面、AC-05/06 | 原阶段门 pending；没有把未到期 B 面重报为 finding |

| 检查 | 本轮结果 |
|---|---|
| 包定向测试 | [42/42](package-tests.txt)：client15、execution9+18 |
| 真实工具独立 consumer | [16/16](fixture-tests.txt)，实际二进制先构建、offline+locked消费 |
| R1 五原反例 / R2 四控制 | [全通过](r1-replay-output.txt) / [全通过](r2-replay-output.txt)，包括正常/小请求/分次坏帧/干净EOF/双向完整字节对照 |
| R5 两writer入口+三stop终态注入 | [全通过](r5-replay-output.txt)，run/client326/316ms，线程4→4，非owned持有者存活 |
| R6/R7 栅栏、Linux | [R6通过](r6-race-output.txt)、[R7通过](r7-after-check-output.txt)、[Linux通过且净新增告警0](platform-output.txt) |
| 新终止失败控制 | [通过](termination-failure-output.txt)，0ms具名拒绝，真实清理通过 |
| 新凭证失败状态 | [两个状态均失败](credential-failure-output.txt)，3s后writer仍存活 |
| Windows fmt/fixture rustfmt/clippy --no-deps/rustdoc | [全部exit0](gates.json)，新两包无告警 |
| CLI/Man check | [exit0](cli-man-check.txt)，旧scope告警不判新增 |
| 闭包 / 777 strict | [PASS](deps.txt) / [pass，78边、0违规、2grandfathered](modularity.json) |

58个不同现有Windows测试通过；新故障控制单列，不用重复重放凑测试数。本轮没有跑
任何整档t/tv/tt/tb/tu/taa/tf，没有Linux运行通过声明。依赖、锁、测试配置、既有
测试与Spec指纹不变；只变Plan、候选和两源码，受影响检查及全部旧回放重新执行。

SD-05-A 的真实进程消费边界与闭包仍一致，A不改安装发行清单；SD-01-A 的全路径
真停/错误传播不能在该残项未闭时发布。SD-02/03/04 留原B门。现有Spec无漂移，
元数据路径/GOAL-003保持；未改canonical Specs或live ledger。

连续失败的本轮诊断：重试设计已解决有效凭证时序，不能把“同一取消凭证的重发”
解释为“没有该凭证也能取消”。本轮有明确修复边界：只余凭证创建/复制失败的
具名拒绝或真实兜底，保留已闭时序、终止失败有界化与平台健康结果，不重新扩大需求。

next：原树work修齐该初始化/错误退出残项，补两失败状态及有效凭证正控，提交新SHA
后受影响复审。重开T-02/T-12、T-13未完成，T-01保持，executing1/14。
F-R7-01/F-R6-01/F-R5-01保持关闭。阶段pass后T-14才在冻结提交执行裸cargo t、
实际触面/新包fixture/闭包与合入、SD/受管ledger/来源读回；当前无VM/trans/UI生成/
AAVM触面，最终并集复核；tf仅主检出到期批量。本轮未修实施、合入、归档或清树。
