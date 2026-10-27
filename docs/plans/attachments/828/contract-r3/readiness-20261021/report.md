# PLAN-828 F-01 / F-02 执行合同就绪核对（2026-10-21）

**结论：原目标、支持面与验收判据已足够清楚，可继续同树 T-01 有界修正/首例验证；当前 T-01 交付尚不能作为 T-02 或 T-05/06 全面改造的已通过前置条件。** F-01、F-02 均保持 OPEN。不是待用户重新批准范围，也不是要求重做已闭合的 F-05b。

基线：Plan r3，`plan-828-dev@3e60af2c8aa2c287633f4700b377714e94c4ff4f`，工作树 `D:/autostack/.wt/lang-828/auto-lang`，入口 clean。应用技能来源 auto-musk@`7703bb6de6c366949c0f0dd6d0eeead2c2def6d7`。本次只做合同/前置条件的静态核对，不构成该提交的最终代码验收；未运行 Cargo、docs_gen、性能或资源门。原工作回归摘要保留为其作者声明，不冒充本次复现。输入与哈希见 [inputs.json](D:/autostack/auto-lang/docs/plans/attachments/828/contract-r3/readiness-20261021/inputs.json)。

## 就绪判断

| 能力 | 已明确且可继承的合同 | 尚未满足的入口条件 | 下一工作 |
|---|---|---|---|
| F-01 | 原 §5.2/5.6/5.8、C01-01..05 已规定单 body 真实语义/HIR 生产、未改工作为零、合法/非法对照与旧路径反控 | D-01 的实际脚手架仍复制 body、重建全体 body 文档；观测不能揭露这类工作；候选环境事务责任未冻结 | T-01 修正 D-01 及最小观测/事务首例；受影响面审核后才沿 T-02/03 接入真实 session |
| F-02 | 原 §5.6/5.8、C02-01..05 已规定不可变单元实际派发、局部表/绑定、旧链固定 snapshot、真副本反控 | D-02 尚缺可执行的单元消费正例和实际派发副本反控；索引空间与三入口的具体消费闭环未证明 | T-01 补最小单元派发/绑定首例与失败控制，冻结实际表示；之后沿 T-04→T-05/06 推进 |
| F-05b | r23a 既有取消/终态/关闭拥有引用已闭合 | 新 retained 环境和新 dispatch 未来会增加拥有关系 | 保留 CLOSED；相关改动后按 C05 矩阵重放，不借旧 PASS 覆盖新对象 |

范围仍为既有 scalar-i32/bool；不把 receiver/text/record 等外域放入本轮，也不删去原 scalar/trace/递归/短路/命名实参规则。A07、依赖、预算、changed-only/manifest、正式 IPC 与文档开放项仍按原 T-03/07/08/09 归属，不能因 T-01 局部正例过关勾整项。阶段落地/merge 仍不允许。

## R-01：D-01 的“只读签名脚手架”与实际工作不符

既有规则：AC-01/02；§5.2 消除热路径整 Program Atom render/bind，§5.6 只允许接口目录/必要元数据工作，禁止重建未改 body/HIR。

源码证据：
- [resolve.rs:828](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/resolve.rs:828) 的 `stub_resolved` 先 `entry.clone()`，再清空 body/非参数 locals。ResolvedFn 及 RBlock/表达式均为拥有式递归 Clone；这不是只复制签名字段。
- [adapter.rs:1981](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/adapter.rs:1981) 对所有函数（包括 real_idx）先构造这个 stub，再每个调用 `plan_fn`。
- [adapter.rs:2266](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/adapter.rs:2266) 的 render 遍历全部函数生成 body 文档；[adapter.rs:288](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/adapter.rs:288) 对整篇文本 bind。未改空 body 的 HIR 生产未因改名“scaffold”消失。

这并不否定单个 resolve_fn/check_body_in 接缝的可复用价值，也不禁止读取 callee 签名；它说明当前 emit 路径不能按“接口目录遍历”直接授予原结构门 PASS。无需猜测 wall time 就能确认工作边界不符。

**owner=T-01/D-01，T-02/03 消费；关闭条件**：只从签名/声明字段建立所需引用上下文，不先复制再丢弃任何未改 body；抽取现有 producer/binder 的单 body 接缝（需要时扩展其上下文消费），不经全部 stub body 重建 HIR。保持原私有 verifier 构造凭证的信任边界、旧完整入口、cold 对拍与错误诊断。具体新 API 由实现有界选择，不能宣称目前已经存在。如果 binder 的 DefRef 绑定机制尚无法做到，交出具体阻碍与候选方案继续 D-01 决策，不能把原禁用路径重新解释为允许。

## R-02：观测/负控尚未证明同一个门能检出真实退化

既有规则：C01-01/05、§5.8 要求真实生产入口观测；实际执行旧管线且同结构门变红，缺观测另拒绝。

源码证据：
- [adapter.rs:277](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/adapter.rs:277) 把 `real_body_plans=1` 写成固定值，不能反映 R-01 的复制/其他 body 生产。
- [resolve.rs:735](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/resolve.rs:735) 的 `env_note_cold_walks` 由消费者设 N/N；新 resolve_env 本身只 resolve。不能把该设置当“已发生冷 typecheck”的真实证据。
- [function_incremental_828_contract.rs:112](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-av-probe/probe/tests/function_incremental_828_contract.rs:112) 的负控是 `gate(None,None)`。后续运行 cold compile 用于 HIR 对拍，但并未将强制旧管线在同一会话/候选路径上的实际事件喂给同门。

**owner=T-01/D-01，T-02 生产换接；关闭条件**：事件来自实际 resolve/typecheck/HIR producer/binder/复制边界，区分 changed body 与接口目录工作。最小合法单 body 正控绿色；测试配置强制执行真实旧全部 stub 管线，保持真实观察器可用，结果可对但同一结构门因真实额外工作变红；再单独验证缺必需观测拒绝。不能靠换 receipt 类型或标签把旧路径屏蔽掉。冷热观测分开，新增字段须说明生产位置与断言；不要求通用观测平台。

## R-03：D-02 已诊断旧现状，尚未完成可行性首例

既有规则：T-01/D-02 判据及 C02-01/02/04/05；完整三入口行为归 T-05/06 实装和验收，T-01 先证明所选机制最小可行。

已成立的源码事实：
- [program_snapshot.rs:94](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-av-probe/core/src/program_snapshot.rs:94) 当前仍合并表、clone blocks/slots/spans 并改写指令。
- [av.rs:136](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-av-probe/core/src/av.rs:136)、196、219 三入口仍 dispatch merged。
- 当前 identity 测试如实断言 merged 身份/内容不复用，有诊断价值，不能当成新单元派发已可行。
- 现有 forced-copy 控制重建 FunctionCode 后仍由 linker 生成另一份 merged 供 AV 消费；它没有证明 AV 消费所变异的 unit payload，更不能替代未来单元派发的副本敏感性门。

**owner=T-01/D-02，T-05/06 消费；关闭条件**：
1. 冻结实际读取链：帧/当前函数 → 所属不可变单元 → 局部 PC/slots/types/consts → import/self-call 绑定 → 目标函数及所属 snapshot → trap 来源映射。区分单元局部编号、逻辑 FunctionId、snapshot 槽和来源表 func，不要求这些索引数值相等。
2. 双函数首例必须执行所选新单元视图：callee 修改且常量/槽/指令长度变化，新顶层结果正确，未改 caller 的实际 dispatch payload 同一；拒绝好单元配坏签名/缺目标，旧根及映射不变。不能只写绑定表设计文字。
3. 说明普通/分步/resume 共用哪一个实际 fetch/dispatch 接缝；用确定步数的真实暂停链证明不会在 resume 旁路回 merged。T-01 验证最小接缝，完整位置/递归/授权/退出矩阵仍由原 C02 各 owner 完成。
4. 测试配置强制让 AV 实际取同内容副本，最终值可对但同一个 payload 结构门红；复制一个未被 AV 读取的对象不算。旧完整 module/ABC 入口、unit 不可直接执行、签名及 capability 准入规则保持。
5. 不强制预选全局 intern 表或某个 Rust 类型；允许必要绑定元数据，禁止借表解析重建/改写未改指令。codec/公共语义若需改变，明确版本及受影响消费者，不私自扩大范围。

## R-04：retained 环境须纳入事务与关闭责任

这是接线前必须冻结的设计责任，不是声称新 API 已在 session 中产生了运行时污染。

[resolve.rs:813](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-core/src/source/resolve.rs:813) 的 recheck_function 在成功后已替换 env 条目；失败时回滚只覆盖其自身 typecheck 失败。它不自动解决“成功 prepare 后取消/后续 HIR/链接失败/兄弟候选/迟到提交”的会话事务。当前 session 尚未接入 CheckedEnv。

既有 §5.4/5.7/C01-04/C02-04/C05 要求 current/source/revision/来源与语义一致。T-01/D-01/D-03 应明确环境归属、候选持有量、临时换入/恢复或 overlay 的选择、基面校验与一次发布边界；不能每 prepare clone 整个 env 抵消 F-01。更新 close 的拥有关系表，新环境保有的 body 不能只靠旧 root/HIR/code Weak 控制冒认已覆盖，仍按原预算计入。

此外，[session.rs:1466](D:/autostack/.wt/lang-828/auto-lang/experimental/ac-av-probe/probe/src/session.rs:1466) 现行 commit 在 link 返回成功前已修改 origin/span_map/history 元数据；T-06 必须在真实 link 失败分支验证不污染，并先构造候选元数据再发布或采用等价原子处理。本次只记录生产时序及待测风险，不将未执行的失败注入写成已复现运行结果。

## 证据与收尾

- 源码可数出 contract frontend 4个 #[test]、dispatch 1个；interface-decision 的 frontend“5/5”与其选择表4不一致，须由 work 按原始输出纠正。名字存在不等于已执行完整 C01/C02。
- 原 §6“新文件尚不存在”和 §10 的旧下一步叙述已在本次 bookkeeping 中更新；T-01/AC/V/C/SD 的语义、owner、总计数不变，plan_revision 仍3，executing 0/11。
- 无新的范围/阈值/预算/跨仓授权问题，不需重复征求批准。若真实调查证明目标要求改变，才回 new 提具体差异。
- 下一轮交付见 [GLM任务单](D:/autostack/auto-lang/docs/plans/attachments/828/contract-r3/readiness-20261021/glm-next-round.md)。先完成本轮 T-01 修正/最小首例，冻结提交与原始定向证据后交受影响面检查；不要直接跳 T-02 全面换接或给整份 Plan execution_done。
- 开发/修复只做相关反例、所属模块检查及必要 check；最终 pass 后 merge-owned 裸 t/tv/必要触面档继续 pending，tf 到期主检出单实例。每次提交不触发整档。

