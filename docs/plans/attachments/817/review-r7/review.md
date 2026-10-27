# PLAN-817 R7 final review — needs_fix

2026-10-24，Plan r3，review_scope=final，当前独立于实施的Codex上下文。worktree D:/autostack/.wt/lang-817/auto-lang，branch plan-817-dev，冻结实现d640fbce439d2e86f098958021865310fb601ce2，整体base d46e03e8a4986f3d5fed5bd09665c2aeebf0340d，本轮修复基面c7ec7c076。开始/结束HEAD未变且实现树clean。按主检出Plan及R6根因/覆盖调整审查，未修改实现。

## R6两个缺口

- **R6-01累计deadline：原反例和快档范围关闭。** 新RequestDeadline/DeadlineWatch、阶段检查、AV剩余墙钟预算、精确resource分类已接。资源测试中的300ms控制与真实32query十秒控制通过，4query同规模正控通过。另以重建worker重放review-r6原request-wall.json（换新store防hit），原始响应存deadline-result.json。没有重新定义10秒合同或缩小查询规模。
- **R6-02预留时序：partial，不通过。** counting writer→reserve→encode及payload按值移交已改善；wire有界读/base64前检已改善；普通恢复路径的事件排序测试通过。然而固定恢复预留并没有约束底层真实重组分配，见R7-01。事件日志在上层调用结束后手工记录，不能证明底层未超预留。

## F-817-R7-01 / P1 / AC-05 / T-05,T-08：恢复1MiB预留不是实际分配上界

违反原Plan§3/§5.4、contract-decision§7“分配前预留，坏块/解码全路径有界”。属于R6-02未完整闭合，不是新增支持面或规模要求。

生产调用顺序：driver先reserve Staging FROZEN_PAYLOAD_BOUND=1MiB → store::load_frozen_payload → ManagedStore::get_artifact完整重组Vec → driver检查payload.len()>1MiB然后报infra。上限检查发生在真实分配之后；loader没有接收size cap，也没先读取受界元数据。合法envelope通常小于该界不构成坏块/异质工件恢复路径的保护。

**真实加载反例**（仅使用冻结实现公开API，源码oversize-restore.rs）：在同一CTE layer/key下经ManagedStore.put_artifact合法存入2MiB LayoutData载荷（不假造坏文件、不修改codec）；按production同样先预留1MiB，再调用实际load_frozen_payload。运行输出oversize-restore-output.txt：

    reserved=1048576 actual_loaded=2097152 ledger_peak=1048576

真实Vec已返回2MiB，台账仍只有1MiB。driver后续会受控报infra，但不可能撤销已发生的超预留分配。payload内容可为非envelope数据，因为合同明确要求坏块/错内层数据在解码恢复边界也有界；本探针只测load，不将其内容当合法CT结果消费。

首次8MiB输入被783自身transient8MiB预算拒绝（staging+CAS双份），保留为探针调整历史，非817缺陷；改用2MiB后正常存取、稳定复现，证明不是依赖先拒绝而路径不可达。独立rustc最初缺windows native lib搜索目录，补齐cargo已使用的native库路径后构建成功；未修改依赖或被测实现。

同一恢复处还用payload.len作为decode Hot上界，注释称“JSON文本>=decoded结构”。该假设对Vec/String结构体/容量开销并不普遍成立，不能未经内存模型证明当可靠预算；此项是同一关闭oracle中的需审计面，本轮没有另声称实测decode OOM。

**关闭oracle**：在任何重组Vec分配前，用真实可验证长度/有界读取API落实剩余额度；超界原始工件在读取/重组上限处拒绝；对正常和异质/损坏payload两路，用实际分配高水位/allocator或底层读上限观测证明不超过预留。2MiB本反例、界内正控、缺块/错误kind/错误JSON控制、预算缩容失败保旧条目都应保留；不能仅在get_artifact返回后做长度检查或手工写Alloc事件。decode对结构体/容器及与payload同时存活的内存也要给保守有依据的预算。底层store若没有bounded API须用符合依赖单写规则的适配方案，不能取消原坏块资源验收。

## 本轮验证与限度

命令：cargo test --manifest-path experimental/ac-cte-probe/Cargo.toml --offline --release --test cte_resources_tests -- --test-threads=1。

在停止前明确通过7项：encode reserve ordering、deadline fast、real ten-second、restore ordering、wire bound、normal budget、deep input。后续real-watermark GC正控仍在运行时，本轮已用独立生产loader反例确认必需门未满足，因此主动停止**该次审查自有**测试进程（PID17000，路径核对后停止），套件结果是unfinished/人为中断，绝不记录11/11或产品GC失败。未执行完的GC/32MiB/512修订/worker峰值等保持pending；没有重跑整档t/tv/tt/tb/tu/taa。

额外：对driver/store/worker/observe四文件rustfmt --check通过；oversize-restore探针链接冻结release rlib成功，运行输出如上；原32query独立worker重放结果见JSON。代码本身没有变更。现有encode计数与wire有界方向已认可，不因本finding把其全部打回。

冻结Spec delta SHA256 8BEA8A45D22EA3C85FF141D79EFFE82EFECB5AEC3D3222D5154B6B8AB18637AB；探针源码SHA256 0CE503574E3E07CA5F20F303030F66037A004CD81333EFA588D7C54114CDA7F7。Spec delta/report均已复制本目录。canonical四目标只读；依赖为该冻结树内783/796/798/828与lock；无消费他方WIP。

## AC与路由

AC-03累计deadline相关原例本轮通过，T-03相应修复可完成；AC-05及R6-02恢复完整不变量不通过；AC-08/09不能授予最终pass，其余完整验收未重跑部分pending。保留此前R5三P1关闭结论，不重开已过路径。AC-10与merge-owned裸cargo t+tv+消费面pending；最终review通过后才执行合入门禁。

维持executing，T-03可标完成，T-05/T-08/T-11保持开放，T-07已闭合，完成6/11。下一步同树work关闭R7-01、给实际底层边界/分配证据及依赖调整收据，再冻结复审。没有修改实现/canonical Spec/ledger，没有合入或归档。