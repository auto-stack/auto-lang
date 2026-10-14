# PLAN-774 r3 第四轮独立最终复审

2026-10-13，review_scope=`final`，outcome=`needs_fix`。第三轮的两个具体缺陷已清偿；预算不变量还有一个累计计数宽度缺口。本轮不重新打开已通过的旧反例。

## 基线和验证

- 同一树 `D:/autostack/.wt/lang-774/auto-lang`，branch `plan-774-dev`，reviewed_commit=`c5e631bd43d1855ba76e144dfac31be3da487bd1`，代码修复提交=`c2d5f7e86`；base=`5e21561d2c303a02319ff7a29e4607933e09a38f`，previous=`b3bab4278c9f065a5609a57d02d1f2552f640eab`。入/出场同 HEAD、实现树 clean、diff-check clean。
- 主 Plan revision3，入场 execution_done6/8；主 HEAD=`685026eb5103ff3284c3022abf8de7699c4eb1db`。审前全文：[plan-before-review.md](plan-before-review.md)。其他主检出 WIP 保留。
- 当前 Codex /root 独立审阅上下文未参与实施/修复，未另建 agent/模型/checkout，不冒称另起的新会话。
- 依赖、Spec、报告/代码/锁文件、技能：[input-hashes.tsv](input-hashes.tsv)。ac-core/旧 gates/根定义无774 diff，当前主相关Spec无并行语义漂移；770/771同步状态沿当前主检出核对。
- 冻结SD：[proposed-spec-delta.frozen.md](proposed-spec-delta.frozen.md)，SHA256=`deb2a6ef71c43227572635e0687bad6cc71bde6d80b3d0551ed16e991e9b0a3a`。

| 本轮实际入口 | 结果 | 证据 |
|---|---|---|
| 774 PS gate（locked check/full probe/fmt/warning controls/scope/hash） | 7步、63 passed，0 failed/ignored | [gate输出](774-gate.log)、[收据](774-gate-receipt.txt)、[check](probe-check.txt)、[test](probe-test.txt)、[fmt](probe-fmt.txt) |
| 完整 ac-core locked test | 145 passed，既有1 ignored | [日志](core-full.log) |
| 741 -SkipMainGates | 13步通过 | [日志](741-gate.log) |
| 745 | 13步、74例通过 | [日志](745-gate.log) |
| 772（脚本历史名767） | 14步、45例通过 | [日志](767-gate.log) |
| r1 / r2 / r3 独立控制重放 | 7/7、3/3、4/4全部通过 | [r1](r1-replay.log)、[r2](r2-replay.log)、[r3](r3-replay.log) |
| 新累计记账受控状态探针 | 1个合同断言失败，exit101 | [初次输出](counter-state.log)、[可再生重放输出](state-replay.log)、[入口](replay-state.ps1)、[断言](counter-state-footer.rs) |

测试在同树运行；ac-core与旧三gate串行，未争用其exe。无crates/root定义改动，不跑根cargo t/tv/docs_gen。本轮新gate绑定正确提交后HEAD和实际raw目录（已复制）；原型零warning、fmt和scope通过。真实PE仍复用contained executor/60s/locked；AV与Deadline前驱oracle在63探针内重跑。

## 闭合判断

- **P774-R3-01 旧setter截断样例清偿**：预算已用Option<u64>原值保存，第三轮4GiB/4GiB+1下2-byte请求现通过；u32最大值正控仍通过。
- **P774-R3-02清偿**：V05-15a保留16 owner重试，真实触发扩表，读回全部17 buffer、逆序释放、peak17；V05-15b另外保留空位复用，peak16。独立r3恢复控制重放通过。
- **P774-R3-D1整理完成**：SD显式声明§3.9/3.10现行规则、63用例、8合法/16非法；不把旧过程附录作为canonical现状。
- 早先来源/EndView、static literal/跨行、参数模式/下游身份和拼写、LIFO/临时/退出/clone、首表/扩表失败、成功free与invalid分离、60s/locked/警告/收据等通过面保留。本轮没有新增完整语言/公开ABI/GC/List/ACC自举要求。

## P774-R4-01 — P2：预算扩为u64，累计存活字节仍u32饱和，预算会被低估

类型：预算完整不变量残留（延续R3-01 / D-06），修复后的新数值路径；AC-03/04；T-02/03/04/06；V-08。

`shim/src/lib.rs:77–79` 的 budget现为u64，但live_bytes/peak仍u32。`:232`只把**已经保存的live_bytes**转u64比较；`:261`用saturating_add保存，`:300`按释放长度减去。达到u32上限后丢掉高位，转换到u64不能恢复已丢的真实用量。该结构不能维持“完整账本背书的准确存活字节 / live+req≤budget / 释放归还额度”合同。

**受控状态观察**：两个单对象长度各`i32::MAX`，初始总数`4294967294`，其外部账本和计数保持一致；budget=`4294967298`。增加4 bytes应恰好达到预算，随后再加2 bytes应拒绝。

实际原实现函数观察：

```
expected_live=4294967298
actual_live=4294967295
extra_2_admitted=true
```

不是4GiB的真实native申请实测：本轮只申请1、1、4、2字节物理buffer，在原外部账本和计数中注入可表示的逻辑长度，随后调用原`try_alloc`/`owned_free`。这是一条源码算术核查加受控host状态probe，**不宣称完整大内存PE验收**。小块内存避免重负载，同时明确验证了原累计/比较函数丢精度并放行超额请求。源码本身也直接证明u32不能存这个精确总量。

构建方式透明：从已提交shim原文复制，**只移除crate级no_std/no_builtins与panic_handler**来接入std测试器，记账函数体保持原样；添加test footer来设置计数/账本。snapshot见[counter-state-probe.rs](counter-state-probe.rs)，可再生入口[replay-state.ps1](replay-state.ps1)每次读取目标树当前shim，适用于修复后重放。std适配器的freestanding符号lint与生产gate分开处理，未屏蔽生产warning。

这不是新增无限规模/性能承诺。当前setter已冻结接受非负i64预算，单资源长度仍合法i32，合同没有以隐含饱和值限制累计用量；若明确设置资源表示上限，应在申请入口受控拒绝并有规则/码/边界oracle，不能接受资源后静默丢账。

**修复要求**：累计live_bytes、peak、加减/预算比较采用一致准确表示（或者明确且受控的表示边界），不可saturating_add/sub掩盖丢失的高位。同步观测通道/解析/规范增量的数值口径；补跨u32累计边界、恰等预算/超一、释放后剩余精确与再申请额度的oracle。保留当前0/负值预算语义、setter非截断、真实扩表恢复及历轮合法控制。

## AC与规范增量

| AC | 本轮结果 |
|---|---|
| AC-01 | partial：D01..07、取材与下游同步基本闭合，D06累计数值域仍不一致 |
| AC-02 | pass：五类真实ABI、顺序/字段/嵌套/out-return与rec2通过 |
| AC-03 | partial：寿命/非法状态/故障/trap/恢复均通过，累计记账不满足完整不变量 |
| AC-04 | fail：Unicode/索引/转换/现有规模与预算控制通过，新增累计预算边界失败 |
| AC-05 | pass（当前基线）：隔离/旧gate/规范增量边界正确，无feature/ABI/自举冒称；修复D06后须重新冻结对应SD |
| AC-06 | fail：存在上述单一阻断finding，不能最终pass |
| AC-07 | pending：merge-owned未执行 |

SD目标仍为auto-ac/data-resource-contract新增与auto-ac/project、auto-hir/stage-contract、auto-acc/project修改；当前版本/hash已核对。SD-01/03按3.9/3.10正确恢复有效规则，D06新修复必须进入新的冻结文本。touched_goals空值有书面解释，不推进AC/ACC native自举。未发布canonical/live ledger。

## 收敛与下一步

本轮只有P774-R4-01阻断。前3轮样例证明setter保存和小计数，未覆盖“累计值保存→下一请求比较→释放后剩余”链。上轮仅将比较临时加宽，存储改为饱和并未完成准确记账；因此下一修复只需闭合这一数值链，无需重写来源模型/ABI或重新缩减支持范围。

`needs_fix → executing1/8`；T01保留，T02..06重开，T07/08未勾；历史通过/失败及原ID全部保留。plan_revision仍3表示本轮没改变语义合同。next：同树work修复累计表示与观测口径、提交、重放本包状态probe和旧7/3/4控制，再交独立review。

pending_closeout：通过后的Spec/导航/ledger/index、合入授权、归档/任务数、批回归到期、wt-guard和同树清理。此次未修实现、合入/归档或发布canonical/live ledger。
