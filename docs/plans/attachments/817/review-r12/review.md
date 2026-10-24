# PLAN-817 R12 final review — needs_fix（代码修复项通过；T-08文档收口未完）

2026-10-24；Plan r3；review_scope=final；第12次 final review。当前Codex上下文独立于实施，未暴露模型/session ID。主检出当前Plan为准。实现树 `D:/autostack/.wt/lang-817/auto-lang`，分支 `plan-817-dev`；冻结提交 `4309f49dee1c022ed3fac95b629720810e488973`，修复基面 `b29cb2348`，整体base `d46e03e8a4986f3d5fed5bd09665c2aeebf0340d`。起止HEAD相同、实现树clean；未修改实现。

## F-817-R11-01 CLOSED

新模型以全部owned字符串总量S，加全文档共享scratch的容量上界2T计算解码额度；T取最长字符串/数字token。已核对固定serde/serde_json版本、clear保留共享scratch容量、转义串复制到owned结果的路径。当前模型补上了上轮的同时存活缺口，且不是只修改source_fp样例。

独立探针重放R11三档同形输入，成功审计仍零分配：

| 转义串N | JSON载荷B | 新审计额度B | decode实际共存峰值B | 结果 |
|---:|---:|---:|---:|---|
| 32768 | 66630 | 202418 | 197387 | covered |
| 65536 | 132166 | 399026 | 393995 | covered |
| 131072 | 263238 | 792242 | 787211 | covered |

去掉转义、保持解码长度的plain配对也在额度内。进一步通过相同公共put→真实driver恢复路径，检查schema、producer、request_identity.module、engine.av_profile四个String字段：分别受控拒绝、ct_execute=0，decode共存峰值均≤对应审计额度。没有把异质字段当合法结果接受，也没有把最终报错替代分配观测。

原空串×10000例的审计额度1765813B覆盖实际decode峰值394094B；×100000/300000例在真实driver Hot预留处拒绝，不再进入原大分配。直接decode convenience函数的测量用于验证额度模型，driver先行拒绝的结果单独列在原始记录，二者没有混用。正常原envelope继续hit、ct_execute=0；编码capacity=grant=1163B，维持R10已关闭项。非法JSON/65层JSON控制均在审计处受控拒绝。

本轮永久回归覆盖三个escaped/plain档、旧2S模型敏感性控制、100000位数字token和真实driver拒绝出口；与独立探针观察一致。verify脚本只增加resources族串行参数，与进程级allocator窗口相符；本轮审查命令也显式串行，没有执行整族。

## 本轮最小测试范围与复用

| 检查 | 对应原因 | 本轮结果 |
|---|---|---|
| cte_resources_decode_audit_escaped_scratch_bounded | R11-01直接反例/新模型/数字token/原模型红控 | 1/1 |
| cte_resources_decode_audit_bounded | S项变化影响既有数组/Hot预留边界 | 1/1 |
| cte_resources_restore_reserve_before_alloc_order | 审计额度变化影响恢复先预留/缩容拒绝控制 | 1/1 |
| 独立allocation-probe | R11实际输入、plain正控、String字段共享路径与错误出口 | 通过，见probe-run/result.txt |
| 定向rustfmt、git diff --check、release lib构建 | 变更文件健康/精确库绑定 | 通过，无告警 |

未重跑R11/R10中未改的cache/freeze族、encoder/observer重族，以及GC、32MiB/512修订、完整性能矩阵。相关测试/生产逻辑与依赖/lock未变，旧证据在原输入/配置和语义适用范围复用；表述不等于宣称旧计时就是本次新测量。body/Meta编辑硬门的失效miss车道不执行恢复审计，相关性能门和其内存/父判/退化反控沿原匹配证据保留。本次正常恢复、坏载体和额度边界已作定向验证。没有执行cargo t/tv/tt/tb/tu/taa或任何all/acceptance/closeout聚合测试。

## F-817-R12-01 / P2 / AC-08,09及AC-11覆盖收口 / T-08,T-11：验收矩阵只剩修复历史

这是交付文档缺口，**没有发现新的运行时缺陷**。Plan T-08明确交付 `docs/reports/plan817/acceptance-matrix.md`，含V-01..V-11→scope/证据/提交+AC绑定；T-11追加V-12四个必需子case及对应coverage。当前冻结文件只有6行：一个空行和R7/R8/R9/R10/R11五条修复记录，无表头，无V-01..V-12验收映射，也没有AC/task映射。不能凭旧“已完成”收据认定矩阵仍然存在。

已将实际提交文件冻结为 `submitted-acceptance-matrix.md`。重现仅需读取该文件并对照主Plan §6–8；对其搜索V-01..V-12、AC-01..AC-11均无命中。修复历史是有用证据，但尚不能替代全计划交付索引。

**最小关闭范围（无需全局测试）**：恢复当前V-01..V-12矩阵，关联AC/task、实际入口/关键断言、对应Scope、输入/代码版本及可持久化证据；V-12四子case逐项有落点。优先复用已有匹配证据及本轮R11-01关闭记录，区分新跑/复用/merge-owned pending。历史修复记录保留为链接或单列历史。若整理时识别出真正缺少的必需oracle，再补该项定向验证；不要为修表重跑整套probe/GC/性能/全量回归。同步spec-delta“覆盖”索引的V-12落点，消除旧完成声明与实物的差异。

当前SD-02的scratch/owned模型经本轮代码与反例审查通过；canonical四目标只读、版本hash见baseline。Spec delta冻结SHA256 `C891C0A7339D50B8461D21ED6112070C8D4612E672347B39889BD0D96CACB6E6`。其余SD本轮无语义变动；整计划AC-08仍待上述矩阵收口。

## 状态、证据与下一步

F-817-R11-01及既有代码finding均关闭；AC-05相关资源修复通过，T-05完成。AC-08/09因T-08矩阵缺失不能发整计划final pass；T-11实现/结构证据保留，coverage部分待V-12映射补齐。维持executing，完成数**7/11**（T-01..T-07完成，T-08/T-11/T-09/T-10开放），不把已通过资源修复继续挂为代码未完成。

`touched_goals: []`的说明：当前goals账本未单列有限AC/CTE实验目标，本Plan不变更既有Goal定义或达成状态；能力知识通过所列auto-cte新Spec及四个既有Spec增量承载，不冒领旧VM快周转或AAVM自举Goal完成。没有借此删除既定能力/性能验收。

基线、依赖树、lock、技能版本、Spec、输入与探针hash见 `baseline.json`。`replay.ps1`从本次cargo JSON artifacts绑定rlib/native路径，复现的是新隔离store中的实际CAS载体；原始结果为 `probe-run/result.txt`。输入/探针都保存在主检出附件中，避免清理worktree后证据不可达。

**next: work，只补T-08/V-12覆盖索引并冻结送审**；本轮不要求再跑未改的测试。最终代码/验收/Spec审查全过后，merge在冻结提交执行裸cargo t+必要tv/probe消费面门禁；这些统一门禁仍pending，不得提前合入。未修改实现/canonical Spec/ledger，未合入/归档。
