# PLAN-774 第二次交付就绪核查

2026-10-13，`stage: review`，`review_scope: final`（入口），`outcome: needs_fix`。
已知 work-owned 缺口仍在，故未进入完整最终复审周期，未重复三道旧门禁及全套 ac-core。
旧反例清偿不等于全部不变量清偿；55/55 不是最终 pass。

## 基线与证据

- Plan 主检出 revision 3，入场 `execution_done 6/8`；审前冻结见 [plan-before-review.md](plan-before-review.md)。主检出 HEAD `c151935f1b50a9aefb7164771bcc70ccf4b66c9d`；已有其他计划/设计/债务 WIP 保留。
- 原唯一树 `D:/autostack/.wt/lang-774/auto-lang`，branch `plan-774-dev`，reviewed_commit `773f9a09aeb874845dc0a0cf44d89a07e3b3a87f`；原基面 `5e21561d2c303a02319ff7a29e4607933e09a38f`；对比第一轮 `f1a26e1be51de2dbb0c0d983b5ec346a86475aed`。入场/收尾 HEAD 不变、实现树 clean，diff-check clean。
- 审阅上下文：沿用第一轮独立审阅的当前 Codex /root 会话，没有参与实施或修复；本次不是另起的新聊天/最终审阅上下文，没有冒称用户消息所提“新上下文”已创建。本次已知缺口足以返回 work，不需为确认已承认的未完工作创建新 agent。
- [input-hashes.tsv](input-hashes.tsv) 绑定新代码、决策/提案、原型依赖锁文件、当前 770/771、Specs 和技能。SD 全文见 [proposed-spec-delta.frozen.md](proposed-spec-delta.frozen.md)，SHA256 `ff5ec7d62c8538bdade8eb8e99c3302aaf710da157d6e5cfbf4934cc80f73346`。
- 本轮实际重跑第一轮 [replay.ps1](../review-r1/replay.ps1)：**7/7 passed，exit 0**；原四个失败控制全部清偿，见 [r1-replay.log](r1-replay.log)。未改这些 expected。
- 新来源组合控制：**2 passed / 1 failed，exit 101**；输入 [model-controls.rs](model-controls.rs)、输出 [model-controls.log](model-controls.log)。直接编译已提交 rules.rs，没有改实现。
- 抽取已提交 gate 的 warning matcher 定义，重放第二行警告/无警告控制，分别 True/False，见 [warning-controls.txt](warning-controls.txt)，该 helper 原问题已修。
- work 的原始 test 输出实际含 **55 passed、0 failed/ignored**，已复制 [work-test.txt](work-test.txt)。这是核对执行者证据，**不是本轮重跑完整 probe**。收据 [work-gate-receipt.txt](work-gate-receipt.txt) 的 HEAD 仍为第一轮 `f1a26e1be...`（当时带修复 dirty code），raw 实际目录为 **raw-20261013-134911**，不是 verification §8 写的 raw-20261013-134941。code hash 可支持部分输入身份；不得把旧 HEAD 收据称为提交后 `773f9a09a` 的最终完整复审凭证。

## 阻止交付的当前缺口

### P774-R2-01 — P1：来源模型尚不完整，并新增借入参数 View/EndView panic

沿用 P774-R1-02 的完整不变量；AC-01/03，T-02/03/04/06，D-03/04，V-04。

1. verification.md:135–137 和主 Plan §9 work 修复轮**明确承认**显式可变借入参数未建模。代码 rules.rs:215–218 同样注释该缺口；Op、MutBase 都没有 incoming mut-borrow provenance。当前决定明确支持 `mut Str`，而修复记录称 `mut str`，这两个源类型还需区分核对。不能用“保守拒绝/770 以后实现”完成 774 自己声明的可验证规则和合法兄弟义务。
2. `ParamBorrowed(0) → View(0) → EndView(0) → ReturnViewParam(0)` **直接 panic**：`EndView without an active view`，rules.rs:203。原因是新 View 对借入参数只记读计数、不记录 view 区域，而 EndView 只认识 Local/ParamOwned 来源。这不是坏 slot 或缺 Make 输入，实际参数已注册、此前 View 已接受。模型可以明确区分“只读访问”与“打开 view 区域”，但不能把可接受的合法借用区域关闭流程落入 panic。
3. 两个正反控制通过：借入参数直接返回合法，move-in 拥有参数返回 view 正确 borrow-escape。原已释放局部伪装参数的反例亦通过。它们保留清偿，不覆盖上述新增来源路径。

修复：建立来源×访问模式×view/mut 开闭×return/storage/call 的代表性模型矩阵；明确 incoming readonly / incoming mutable / move-in owned / local / static literal 的区别；为合法可变参数提供可执行模型与真实 PE 兄弟，为已接受 view 的正常结束提供稳定路径。仍是受限探针，不要求完整产品借检查器。

### P774-R2-02 — P1：下游未同步，并用修订章节缩减已有支持规则

沿用 P774-R1-04；AC-01/04，T-02/05，D-02/03/07。

- 主检出 770/771 均仍 revision 2：770:65=`text-draft`、:72=`.to(int)`，771:65=`record-draft`。774 decisions:277–284、interface-contract §7 只是要求“下次 revision 改齐”。原计划 §5.1 已要求结果与下游不同时**先修订对应合同**，不能把已知冲突作为下游责任延后，却宣称全部关闭、无必需 open。
- 跨行字面量被新增规则直接拒绝（decisions:285–290）。这与 770 的普通/转义/跨行形态及 V-08 CRLF 跨行文本要求、canonical auto-acc:39 的“合法形态”事实不符。代表消费者 `packages/aavm/lib/engine.at:314/333` 实际用引号内换行。未找到该缩减的用户决策或相应 Plan revision；`plan_revision` 仍 3。774 可以冻结具体语义，但不能以探针修复方便删除既有承诺。
- 原 D-03:126 明确借用返回允许**参数或常驻字面量**；新增 §3.8:255 改为“当且仅当来源是存活借入参数”，interface:132 同步为“仅借入参数合法”。来源模型没有 static literal 入口。常驻字面量没有本帧清理风险，第一轮要求区分 move-in 参数没有授权删除此合法来源。
- 转义白名单也尚未解释与既有 `\r` / char `\'` 消费者的差异；这项需作为取材/适配表核对，不在本轮自行判定新增所有 escape 都是验收要求。

修复：先校准唯一 source/HIR 身份和源语言拼写，并实际完成对应 770/771 合同修订；恢复既定合法支持或走 new/revision 的显式变更程序。literal/借用来源表、用例、SD 必须一致；禁止以“取代上文”在修复附录内偷偷改变验收。不必在 774 实现源码 parser。

### P774-R2-03 — P2：资源账本替换的新失败出口仍无执行 oracle

沿用 P774-R1-01/05 的完整不变量；AC-03，T-03/04/06，D-06，V-05。

新 `ledger_insert`（shim:150–176）另调用 HeapAlloc 扩表，失败后 `try_alloc` :223–226 回滚已申请 user buffer。该路径没有进入现有 `fail_nth_alloc`（只在 user allocation 前判定），没有独立注入点或实际失败控制；55 测和 7 重放只证明增长成功及旧 user 申请失败。新失败出口还忽略 rollback HeapFree 的结果；旧表释放也不计失败。

本轮没有声称该路径已经实测泄漏或用户操作必然失败；缺的是替换资源实现所需的代表性 failure-exit 证据。原计划要求正常受控失败清理、每申请点 oracle；新的内部元数据请求如何计 attempts/allocs/frees（当前仍写 HeapAlloc 调用总口径）也要明确，不能由旧 AP-1..4 总数隐含覆盖。

修复：给首次账本分配、扩表及 user 已分配后的 rollback 有可执行故障控制；在已有 owned 资源下证明保留旧账本/回收新增资源/状态与计数正确，明确 metadata 与 user 申请计数边界。不要求通用分配框架，不新增任意性能规模承诺。

### P774-R2-04 — P2：最终收据引用/基线和完成声明仍未校准

沿用 P774-R1-06；AC-05，T-06。

verification §8 的 raw 目录后缀写为结束时间 134941，实际是开始时间 134911；提交交付声明把旧 HEAD+dirty code 的工作收据称为同一修复 HEAD 收据。Plan §8 同时仍写第一轮“当前 executing 1/8”，frontmatter 却 execution_done 6/8。修复记录写“blockers: 无”并把六发现全关闭，但另有已知模型/下游遗漏。

修复：区分历史运行、按源 hash 可复用的证据、提交后新基线验证；精确引用已有 raw 目录；在正确合同完成后生成一致的 committed baseline 收据与冻结 SD，不把未完语境写成已清偿。原 warning matcher 与 60s/locked 的源码修复保留，不因这一收尾缺口否认其改善。

## 第一轮发现的当前闭合状态

| 第一轮 ID | 旧样例/具体修复 | 完整不变量 / 当前剩余 |
|---|---|---|
| R1-01 | 64/65 账本、全额回收、零预算：本轮重放通过 | 新账本失败出口/计数边界 pending，R2-03 |
| R1-02 | 释放后伪参数返回已拒；真 pointer/length 返回和新增 storage/move 入口已有实现/工作测试 | incoming mutable 未完成；借入 View/EndView 新 panic，R2-01；static literal 被删，R2-02 |
| R1-03 | FREE-id/LIFO、嵌套出口、临时、move-in/return/clone/AP-3/4 已增补；work raw 对应测试通过 | 本轮未重新跑完整 native 补充；待正确合同就绪后复审，未声明完整 pass |
| R1-04 | 默认 view、递归 nested 布局、R2 限定已写 | 下游实际不同步、literal 支持缩减，R2-02 |
| R1-05 | 外部账本先验、不读 freed header、成功 frees/invalid 分离的源码修复存在 | 新 ledger failure/rollback 证据 pending，R2-03；不声称本轮重跑 double-free 全档 |
| R1-06 | matcher 真正正反控制通过；raw/60s/locked/timeout 文件观测已增加 | 收据基线/引用及未完完成声明需校准，R2-04 |

## 收敛诊断（两次未通过交付，下一次就绪前完成）

| 剩余不变量 | 为什么上一轮重放没覆盖 | 覆盖/设计调整 |
|---|---|---|
| 来源模型全合法/非法语境 | 7 控制只有 freed-local 修复、local mut、旧预算，不含新 ParamBorrowed 的 region 结束和 incoming mut/static | 先冻结来源×模式矩阵，再逐格代表性组合；合法输入不可 panic，负例受控拒绝 |
| 决定与下游支持一致 | 7 控制不读取 770/771/Spec，也不审查附录替代语义 | 下游 revision/当前 Spec/原支持表三方对账，记录实际同步或必要审批，不用未来责任关闭 |
| 资源替换失败出口 | 65 控制只走 grow 成功；fail_nth 仍只打旧 user 申请路径 | 加首次/扩表失败和 rollback oracle，保留已有 owner 的合法/失败兄弟 |
| 证据绑定实际交付 | 总数绿不能证明 HEAD、raw 路径和合同完整 | 按提交/source hash/实际目录冻结，先消除已知 work 缺口才 execution_done |

AC-01 partial、AC-02 pending（未做完整第二轮 native 复审）、AC-03 fail、AC-04 fail、AC-05 partial、AC-06 fail、AC-07 pending merge。SD-01/03 的参数/返回/literal 规则尚不适用，不能 final pass；SD-02 的 List 修正与空 Goal 解释保留。规范增量未发布，依赖和测试配置完整门禁留待就绪后的同树独立 review。

## 路由

`needs_fix → executing 1/8`：保留 T-01，T-02..06 重开、T-07/08 未勾。保留所有历史 work/review 记录和本轮通过控制，不重置失败历史。plan_revision 仍3表示本轮审阅未修改语义合同；执行者须在实际需要改合同处按 new/revision 处理。

next：同树 work 先完成上述收敛诊断、合同同步与来源/失败出口 oracle，提交后再交独立 final review。未实施修复、未合入/归档/发布 canonical/live ledger；合入授权、规范沉淀、批回归到期及守卫清理仍为 pending closeout。
