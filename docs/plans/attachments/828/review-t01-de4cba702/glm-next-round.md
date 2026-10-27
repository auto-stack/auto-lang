# PLAN-828：仅完成 C05 候选事务，再冻结复审

基线 `plan-828-dev@de4cba7022a2dd9f2d03ee3dea91322a5088674c`，实施树 `D:/autostack/.wt/lang-828/auto-lang`。读取主检出 [Plan](D:/autostack/auto-lang/docs/plans/828-av-function-incremental-session.md) §§5.4/5.6/5.7/8/9 与 [本轮报告](review.md)，核 HEAD/branch/clean。已有授权足够，继续同树 work；不另建 Plan/树、不开始 T-02 全面换接、不 merge。

**C04 已独立 CLOSED，C01/C02/C03 与 F05b 既有闭合保持；只剩原 C05/P2。** 后置 verify 失败恢复、顺序取消后继续、closed 拒绝已过，应保留。不要重修旧完整准入或降低它的合法/非法控制。

1. 先以报告的八行事务控制画出当前 env、候选 body、owner/基面和发布点。现有 FnSlotSave 保存旧 body，却把候选长期放在当前 env；commit 没有候选 payload 或 owner/基面检查。原 11 绿中的 D03 例是旧原语示范，不证明新事务；新永久测试的 A/B 是顺序创建，foreign 用不同名字。这些诊断先写清，再实施。
2. **把准备与发布真正分开。** 准备过程中可在独占访问内作有界临时换入，执行真实单 body resolve/check/emit/verify，返回前必须恢复当前；候选在自身对象里持有新条目及所属环境/基面事实。也可采用等价的隔离表示。commit 在核 owner/基面/终态后一次安装自己持有的候选，不是丢弃 old save 就返回 Ok。禁止全 env 深 clone、重建未改 body，保留现有生产门。这里给的是必须满足的关系，不要求照抄未验证代码设计。
3. **处置不写错状态。** drop/取消未提交候选不改变 current；同名同槽但不同环境须受控拒绝且不修改任一当前语义；已不适用的处置不得把新提交回滚。T-01 只做新 helper 的最小状态闭合；完整 session root/source/revision/来源与环境的原子发布、迟到/foreign/closed 矩阵仍归 T-02/T-06，先冻结唯一接线责任，不把本报告误读为已发生 session 污染。
4. 直接收编 [transaction-controls.rs](transaction-controls.rs)：成功 prepare 前后 current=1、candidate=2；drop 保持1；A/B同时存活时取消A再提交B应为3（或对不适用提交受控拒绝）；先提交B后迟到取消A不得退回1；同名同签名同槽的 foreign restore/commit 都拒绝。保留顺序合法提交与 closed 两个正控、原后置 verify 拒绝控制。若改 API，保留冻结原件并把同输入/同语义 oracle 适配到实际新入口，不能删断言或串行化测试来伪造闭合。新 API 可以控制拒绝不适用操作；最终 session 合同中的合法并存能力不能取消。
5. 更新 interface-decision 和 §8/§9 的当前关闭表，明确 current/candidate 分别持有谁、何时可观察/释放、owner/基面在哪里校验、close 如何撤销环境拥有引用。撤销“C01..C05全闭”和“save处置即发布”中超过证据的结论，准确记录已闭子项。修正工作文档六个 ac-core 目标的名字/计数对应，以 raw 为准；不要重复累计 contract 子集。
6. 全部修完才运行定向验收：八项事务控制必须闭合；原边界11、额外准入2、contract13/t01r4、旧关闭9及实际触及的 source/session 模块回归与格式检查。不要跑整档 t/tv/tt/tb/tu/taa/tf、docs_gen、merge 门。提交新完整 SHA、依赖/配置、精确选择数与 raw、逐原反例及合法对照的关闭表后，再交受影响面复审。

Plan 保持 executing0/11；C05 未过前不开始 T-02 全面换接。其他 OPEN 与 merge-owned 最终冻结门/Spec/ledger/归档/guard 不变，不以记债批准延期或缩面。
