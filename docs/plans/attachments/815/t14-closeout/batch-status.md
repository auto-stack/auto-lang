# 到期批量回归：尚未完成

范围：`2772c8012..4d7c5f581`，815阶段实际落地触发。始终在主检出运行，不在815树跑tf。tf完整复跑6270/6270通过（115.708s测试段；117.242s墙钟）。首轮被既有plan484 charts红提前停止，完整轮全绿，原始日志保留。

tt完整轮5671通过、3失败、1超时（5675；179.512s测试段）。失败是plan498_bar_group_emphasis、probe_adapter两例、plan776_r7_iterator_credential_native_family；其他会话随后被观察到运行cargo t，但这**不足以自动证明所有失败为负载flake**，隔离/归因及必要重跑pending。

操作失误及处置：启动tb前未取得tt终止回执；随后核验发现其他会话测试在活动，已停止本会话tb子进程树（原tb入口PID62392），未结束其他会话进程。tb尚未完成。后续独占预检再次发现其他cargo/nextest，退出75，未启动第二轮tb。保留中断日志，不计作通过。

最后成功批量收据`docs/plans/.last-batch-regression.json`保持812不变，不推进覆盖号；本文件只是可恢复进度。后续待独占窗口顺序完成tt归因/重跑和tb；新红未排除前不能报known-reds-only或全绿，必要时按批量规约建fix-forward，不回滚815落地。taa窗口零触发；lib.rs宿主输入/期限改动不是aavm source list触发。
