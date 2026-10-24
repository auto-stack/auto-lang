# PLAN-839 r1 复审核验 R1：needs_fix

受审提交 `e58828ca64b7e26ce984faf850fb3b1d0887517f`，基面 `37fd1a9cfe22d3d3471202ad4a8dbfe2103f9a5c`，唯一实施树 `D:/autostack/.wt/lang-839/auto-lang` / `plan-839-dev`。进入及收尾均 clean，实施源码未改。

**身份和范围限制：**用户请求独立复审，但当前仍是实施会话，没有创建另一代理或聊天。遵照 Plan §6“final review同一树独立上下文……实现会话自查披露局限，不冒充最终独立review”和 review 技能，本轮记录 `review_scope: internal`，以合同、源码、真实断言及新选反例重建结论；不能核销独立上下文最终复审 T-09。发现的合同内缺陷须先修复，无论后续由哪个上下文复审都不能据本轮绿项合入。正式 final-review 轮数仍为 0。

## F-839-R1-01 / P2：Starting 吞掉窗口取消，初始化后自动恢复运行

- **既有规则：**Plan §5.4/5.5、V-06、AC-04/05/06，失焦、离窗、resize/DPI、遮挡/最小化应清输入/时钟并 auto-Pause；不应在取消后未经 Resume 又开始派发。对应 T-04、T-06，验收和交接 T-07/T-08。
- **真实路径：**`App::window_event` 分别进入 `Host::blur`、`Host::resize` 或 `Host::pause_play`；这些最终调用 `Play::pause`。复现不修改产品源码，直接在真实 Host 的 `start_play` 返回 Starting 后调用同一生产边沿，再经 `Host::frame` 完成真实 Auto 初始化及 GPU 发布。
- **输入/命令：**持久化客户端 [probe/src/lib.rs](probe/src/lib.rs)，原场景一个合法 Quad；[probe/Cargo.toml](probe/Cargo.toml) 直接依赖受审树。`probe/bind_lock.py` 绑定 driver 的全部原 package/version/source/checksum，仅增加审查客户端，最终命令如下。

```powershell
cd D:/autostack/auto-lang/docs/plans/attachments/839/review-r1/probe
python bind_lock.py
$env:CARGO_TARGET_DIR='D:/autostack/.wt/lang-839/auto-lang/experimental/autographics-auto-driver/target'
cargo test --locked --offline -- --nocapture --test-threads=1
```

- **预期/实测：**四种 Starting 取消边沿完成合法初始化后应保持暂停、不自动派发；实际全部 `Running / tick0 / error=None`，随后三轮间隔20ms的真实 `Host::frame` 全部推进到 **actual Auto tick2**。exit101；[最终固定锁日志](holdouts-final.log.txt)。这不是仅 HUD 或枚举观测。
- **确定原因：**`src/authoring/play.rs:295` 的 pause 只处理 Running/Pausing，Starting 请求直接忽略；`accept` 在547–549行无条件将 Starting 改为 Running。Host 已收到取消，但没有保留取消意图或原因。四条入口共用同一缺口，不拆成四项 finding。
- **合法对照：**同客户端 Running 失焦立即 Paused，后续不增 tick；Starting Stop 真实 VM 弱 owner 释放且 Editor 指纹不变；真实 tick 已修改状态后最后 scalar query 故障整笔拒绝，last-good/tick0 保持且 VM 释放。三对照通过。既有 runtime/pending Pause/Step/Stop 仍通过。
- **闭合 oracle：**work 在同树修复并补永久断言；四种 Starting 取消及正常 Starting→Running、Starting Stop/初始化失败、Running/inflight auto-Pause、Paused Step/Resume 应各保留正确完整事务、owner 和初始化期限。取消可在完整初始化边界兑现，但不得自行进入可派发状态，下一显式 Resume 才恢复；保持输入清零、原 Editor 不变及 Stop 恢复。不要通过删取消规则、放宽时钟或跳过初始化输出闭合。
- **覆盖边界：**既有单窗 trusted-source/native-call profile；本例使用生产 Host 入口，四种组合未另造 OS 消息回放。完整原生正反门另外独立重跑通过；未要求同步编译可抢占，也未扩展平台或任意脚本支持。

## 验收对账

| AC / V / task | 结果 | 代码与本轮证据 |
|---|---|---|
| AC-01 / V-01 / T01 | pass | 正式838基面、27个 driver-only 改动文件；SDK680/closure479/17 local/29 edges、两锁及278 source hashes通过。[source](source.log.txt)、[冻结身份](receipt.json.txt)。 |
| AC-02 / V02/08 / T03/05 | pass | 真实 accepted dirty/redo/未存GLB、完整 Editor 指纹、busy文件sentinel、Close Cancel、Stop/Save/新进程重开通过。[Host5](play-tests.log.txt)、[native/reopen](native/positive/reopen.json.txt)。 |
| AC-03 / V03 / T03/04 | pass | 通读 profile6 Auto→typed全帧准入→prepare→publish；renderer消费实际Auto world/effective，CPU只验算。12坏输出、改Auto速度/visibility→GPU、晚query失败保last-good/释放均过。[模型6+Host5](play-tests.log.txt)、[独选晚query](holdouts-final.log.txt)。 |
| AC-04 / V04/05 / T04 | **fail** | 两600 trace/显示批次、128节点/空图、过载/期限/tick上限及Pause/Step对照通过；Starting取消违约 F-839-R1-01。[deadline1](deadlines.log.txt)。 |
| AC-05 / V06/10 / T06/07 | **partial** | 实际 Iced 文本focus、2D/3D箭头、V parity/隐藏、Running失焦、repeat/取消控制通过；Starting取消后仍派发。[native7](native/results.json.txt)、新反例。 |
| AC-06 / V05/07/08/10 / T04/05/06 | **partial** | pending Pause/Stop、stale/replayed token、Fault VM释放/保last-good、Stop不prepare/compile、启动失败→Stop→新Run及Close通过；Starting窗口取消缺口仍开放。 |
| AC-07 / V01/09 / T03/08 | pass（工作范围） | old profiles/SDK/AGP/资产/金样源码未变；20轮真实VM/GPU Arc/target/pipeline/上传、Editor最后owner及外部pin通过。保留VM/强制重传分别101，恢复0。[owner结果及原始断言](owners/results.json.txt)。完整旧双游戏矩阵按原合同仍属merge。 |
| AC-08 / V10/11 / T07/08 | **partial** | 新原生正例27phase、六像素状态，wrong-focus/drop-tick/三真实GPU故障/partial全符合退出和oracle；源码控/支持边界/SD基本正确。取消与“无缺口”的交接须在修复后刷新。 |
| AC-09 / T09 | 未通过 | 同上下文限制已披露；有开放P2，不能授最终独立pass。 |
| AC-10 / T10 | pending merge | 本轮未执行完整合入矩阵、合入/规范ledger/归档或清树。 |

## 实际检查、敏感性和范围

以下均在同冻结实施树 driver 目录执行；完整命令/版本见 [receipt](receipt.json.txt)，原始输出保留。

```text
cargo test --locked --test authoring_play --test authoring_play_host -- --nocapture --test-threads=1        6+5 pass
cargo test --locked --lib authoring::play::deadline_tests -- --nocapture --test-threads=1                  1 pass
cargo test --locked --bin autographics-authoring completion_tests -- --nocapture --test-threads=1         4 pass
cargo test --locked --test authoring_host --test authoring_session --test authoring_interaction --test scheduler_lifecycle -- --test-threads=1  7+4+7+1 pass
cargo check --locked --all-targets                                                                        exit0
cargo fmt -- --check                                                                                     exit0
python tools/verify_source.py                                                                            exit0
python tools/check_scene_play.py D:/autostack/.wt/lang-839/review839-native-r1                               exit0 / 7 invocations
python tools/check_play_owners.py D:/autostack/.wt/lang-839/review839-owners-r1                              exit0 / 101,101,0 controls
cargo build --locked --bin autographics-authoring                                                        exit0 / binary same SHA
```

永久定向测试 **35个不同case，0 failed/ignored**；owner重复对照不另加总。另有审查客户端4项，3通过、1失败（同一case含四种取消）。在保留的既有 `consumer-work839` 源码消费包运行 `verify_plan839.py --controls`，四实际源/资产/锁变异均拒绝，恢复/LF对照通过；未在实施源码上施加该文件突变。check仅 pinned SDK继承146 warnings，新增0；fmt/diff-check过。原生executable经 locked定向重建，前后SHA256都为 `9fe0362bdf149726b623765cda1ca0824c0cb96749bb697eb3c5353a7a5ddda3`，与本轮七原生报告一致。

初版独立客户端自动解析了较新 registry 版本；初始日志/锁仅保留为调查记录，不作冻结证据。随后按 driver 精确 package身份重新绑定，`--locked --offline` 最终四边沿及真实tick结果保持，见 `holdouts-final.log.txt`。晚query反例触发真实 `DivisionByZero`，证明query阶段拒绝，而非声称得到NaN。最初用例在首失败即中断，已改为全部记录再断言；最终四条输出完整。

通读状态/输入/whole-frame准入、GPU准备与发布、失败和Stop/Close/busy准入、保留资源/上传、原生完成性及source export；未见额外遗漏、未经授权延期、Rust行为替代或扩生产写面。旧受影响19测试通过。T02 scratch1和已留存独立consumer构建运行仅作历史支持；其输入受278源指纹约束，本轮没有把历史测试或完整merge矩阵算作新执行。没有在发现首缺陷后跳过其他已识别风险面。

## 规范增量和后续

冻结副本 [spec-delta-reviewed.md](spec-delta-reviewed.md) 及9个canonical输入的raw/LF指纹见receipt。SD01/03受 Starting取消修复影响，尚不能最终冻结为通过；SD02的schema3/隔离/资源pin与现行规范一致；SD04的profile/CLI/生产边界及原完整合入分母保持，最终handoff/source identity须随修复提交刷新。新scene-play-session路径确未存在，9个 supersedes 路径均存在；GOAL-007/008为当前真实目标。未改canonical或live ledger，未降约、未加新需求；plan_revision仍1。

重开 **T-04/T-06/T-07/T-08**，保留 T01/02/03/05已验成果，`executing 4/10`。下一责任角色work：同树一次修齐取消状态和兄弟路径永久assert，刷新source/SD/handoff并提交，再交独立上下文最终复审。最终pass后仍须原完整冻结driver/AGP/16case/双游戏/835..838/新play/owner/source/新consumer及counter AND content门，再实际落autographics-dev和知识沉淀/归档guard清理/到期收据；根整档/tf未在本轮运行。
