# 828 第一批 R2：先纠正两处机制，再一批修齐三项

当前同树 `D:/autostack/.wt/lang-828/auto-lang`，`plan-828-dev@14c929a5d9d54a90106b8e62b058e98e75a28b9f`，基面 d93a9e373；Plan r3/executing0/11。下一责任角色 work，未授权合入。保留后批提交及已过修复，不 reset、不重新实现已闭机制。

先读 review.md、最终三个独立控制文件及 raw。上轮原 12 控制已 12/12，新 b1 4/4；不要把它们全绿理解成全部不变量已闭。此次还剩：

1. **B1-R2-01 P1，完整语义内容身份**：`x = 3 → y = 3` 的真实合法 edit 命中旧 code，增量=3、冷编=1；`Place.local` 被 canonical 漏掉，同键合法但不同语义的 HIR 也被恢复采纳。需盘点用于内容身份的全部真实语义字段及独立准入规则，给出内容编码/判等的有界设计和生产消费对应，再实施。不要仅补 owner/index 或某个变量名。核 body/places/locals/expr/call bindings/控制流等字段为什么进入或不进入内容身份；cosmetic 字段不必全量误失效。语义键变更按既有 store 版本规则隔离旧产物（包括受影响的 HIR/ABC/历史/manifest 读写身份），不吞 Conflict 自证修复。
2. **B1-C03 P2 残项，依赖属于具体受检产物**：按消费者名字保存一份可覆盖的 body_reads 不够。重新读取不能更新旧产物的前提；B remint 清自己的记录不能消除仍在 history 的 A 的依赖。先写清 query 记录/冻结/消费、current/candidate/history 的拥有与发布关系，再修机制；不要简单永久保留所有版本依赖给新产物误失效。保留 scalar 本身不读 body 的范围，不新增 CTE/inline；真实 query core 仍需区分读取结果和旧凭据适用性。此次两反例分别是 reread 后无重验 hit、历史 A 依赖被 B 清除后旧 HIR/code 均复用。现永久测试里的“remint 后回 A 可直接 hit”须按**产物归属**纠正，不能沿用错误预期作为完整关闭证据。
3. **B1-R2-02 P2，新查询 API 漏 closed 守卫**：close 后 `query_body_fact` 仍返回 Some、重新创建 body_reads 并增加查询计数；应受控拒绝且不新建工作/状态，open 正控与所有旧终态行为保持。cold_bundle 的原释放已经闭合，不回滚。

永久收编/同输入同 oracle 重放本包 `adjacent-controls.rs`（5：1过4败）和 `semantic-artifact-controls.rs`（2：1过1败），并保持上轮12和既有定向面。注意 entry 控制的真实 verifier 本来拒绝；raw 中初版错误前提已归因审核装配，不能按那个前提修改 verifier。Place.local 控制是有效产品反例；真实冷编/增量值差必须归零。新增机制需覆盖候选取消/丢弃不刷新旧凭据、历史命中适用性、未读变化保持、合法重建后新事实、二次恢复与源码/code/HIR一致；按原支持面选代表，不造全语言新要求。

本批已连续两次非通过，先将“为何旧 oracle 漏掉/此次机制和判据如何调整/当前关闭证据”紧凑回写现有接口合同、Plan/关闭表，再改代码，避免只追加已知反例特判。全部三项一起修齐后只跑定向验收与必要 check，给唯一新冻结 SHA、实际选择数/exit_code/输入与配置身份、raw 和残余边界再送审。整档 t/tv/tt/tb/tu/taa 与 merge-owned 收口继续后置；整体 executing0/11不变，后批完整验收不追认。
