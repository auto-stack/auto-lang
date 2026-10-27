# PLAN-828：继续第一批 T-03 → T-04

当前基面 `plan-828-dev@85cbebea47dd8c62baac38879fc386ca67fe0b22`，实施树 `D:/autostack/.wt/lang-828/auto-lang`。核path/branch/HEAD/clean，读主检出[Plan](D:/autostack/auto-lang/docs/plans/828-av-function-incremental-session.md) §§5.6–8/最新§9、[T02专项核验](review.md)。T01接缝与T02前端当前受测面已通过；主计划最新三批节奏保持：本批T02+T03+T04全部完成后统一受影响面复审，本收据不算第一批包级通过。

1. 先按原T03落实 `verify.rs/function_queries.rs` 的函数HIR producer、私有CheckedFunction与真实依赖读取/失效：区分接口事实与body事实，callee body-only不使只读接口caller重编；凭证绑定实际owner/domain/profile/caps/版本和实际读取的事实，不能由调用方正确字符串自证。保留T02的实际单body生产窗、cold-vs-warm边界和候选隔离，未改函数不重新dump/clone/构造凭证。
2. 按Design40/现有统一产物协议接函数HIR A07有界codec、producer与当前上下文恢复重验，不只定义key或留占位存储。先用现有store/查询合同调查真实缺口和数据流，做一个实际“生产→保存→读回不可信描述→当前准入→session消费”的最小首例；错误owner/profile/语义版本/授权/来源/损坏反控和合法读回要能区分。闭合范围按原T03，统一引用/GC/生命周期的T07衔接照原owner记录，不假称整份F10/F11完成。
3. T03接口稳定后在同树继续原T04：CheckedFunction到单函数MIR/ABC、verify/admit及内容命中复用，未改对象身份/代际/内容保持；零whole lower/split/全closure roundtrip。保留局部const/type/slot/PC和稳定跨函数引用接缝，完整无merged绑定/三入口派发仍T05/06。不要以一次最终值相同代替粒度/身份和失败准入。
4. T02本轮六个生产组合控制及旧publisher3/transaction8/关闭9均可复用输入，换实现后按影响重验；以实际解析/检查/依赖读取/编码/读取/准入消费者的计数和对象身份证明，不堆手填counter。当前commit在link前修改origin/history-map的既有顺序仍是C02-04/T05/06实失败原子性待办，未因T02PASS解除；若本批触及对应分支，先保留旧状态并在实际失败出口验证。
5. 常驻新环境、查询图、HIR/MIR/代码、候选/旧snapshot/codec暂存全部照F09计预算，不移出基线。只做定向反例/模块检查与必要cargo check，遵守不整档t/tv/tt/tb/tu/taa/tf/docs_gen的开发时序。主检出写簿记，实施/测试留原worktree；r3目标/AC/V/C/SD/owner、阈值/profile保持。
6. 本批完成后交T02/T03/T04闭合表、真实读写与消费raw、合法/非法对照、准确选择数、完整冻结SHA/依赖/clean供第一批复审。T03与T04之间不再逐任务请求包级复审；重大未定设计用现有有owner的有界诊断先解决，不能擅自缩面。第一批通过后才到T05/T06第二批；整份最终验收与merge门不变，不合入。
