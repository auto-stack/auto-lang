# PLAN-782 final R2 — needs_fix

2026-10-15；stage=review；review_scope=final；plan_revision=1。
Reviewer：本次 Codex `/root` 独立检查上下文，未参与实施；未创建额外 agent；模型/会话标识未暴露，不虚构。

## 基线与复验

- worktree: `D:/autostack/.wt/lang-782/auto-lang`；branch: `plan-782-dev`。
- reviewed_commit: `a52a54472728755027078fe712a1de6d2002f905`；base_commit: `4568e4ea795996a759b4747fc56b30a30c4308db`。
- 入口/出口 HEAD 相同、实施树 clean；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- 依赖/工具/源码 hash：baseline-hashes.tsv；canonical 输入：spec-inputs.tsv；本轮规范提案冻结：proposed-spec-delta.frozen.md。主计划从 main 读取；本轮仅写 main 的计划簿记/附件/债务，未改实现或 canonical/ledger。
- 本轮 `cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --test text_record_hir --test text_record_source --test text_record_resources --test text_record_native`：18+21+27+20 = **86 passed，0 failed/ignored**。包含真实故障注入的五个 V07 oracle。
- `cargo build --manifest-path experimental/ac-core/Cargo.toml --locked`、locked all-target cargo check、cargo fmt -- --check、`git diff --check 4568e4ea7..HEAD` 均通过，零编译警告。命令 cwd 均为实施树。
- replay.py：R1 原文 14 项重新构建运行，结果 results.json；与 R1 预期一致（mixed_temp_order 本来就是 Str/str 失配负控制，不能称 14 个 PE 成功）。
- api-replay.ps1：真实 Bundle 9 控制，valid CHECKED，其余受控拒；Move.inner 现为 verify.bad-reference，无 panic，api-results.txt。
- extra.py：新增嵌套短路四个 source→verify→PE 控制；more.py：字段部分重建四个控制。完整输入和逐例 build/run stdout/stderr/exit 在同目录 .at / extra-results.json / more-results.json。脚本退出 0 仅为采集完成，oracle 以本报告比较为准。
- 旧联合 gate/发布事务/故意失败控制本轮未全重跑：其路径未被修复改动，沿用已标基线的历史取证作背景，不能抵消新反例；fresh final pass 后仍需 merge 冻结联合收口。未跑根整档或 experimental 全套；本计划不触 crates，根 t/tv 等不适用。

## F-782-R2-01 [P1] 外层短路将内层聚合存在标记覆盖为 1

AC-04，V-09，T-05/06/07；原 F-782-R1-03 的完整不变式尚未闭合。

`native.rs:4206..4214` 的 LogicalRhs 合流将 Agg 不分是否已有 `present` 一律传常量 1。外层 RHS 执行不代表其内层 RHS 聚合真的创建；内层的 0 被抹掉，语句尾和故障回滚共用的释放器据此走陈旧槽。

`nested_and.at`：两轮，`yes() && (cond(i) && check(a.clone()))`，cond 仅 i=0 真。build=0，PE exit=3，但 trace：

```
LIT id=1
CLONE id=2
FREE id=2
FREE-INVALID
TEXT-SCALARS n=3
FREE id=1
```

`nested_or_take_skip.at` 的 `yes() && (!cond(i) || check(a.clone()))` 同样重复释放。期望两个输入均只创建/释放一次 id2，第二轮零 clone/free，绝无 FREE-INVALID。`outer_skip.at`（外层 take→skip）与 `nested_or.at`（内层 skip→take）对照当前通过，不能证明内层 take→skip。

现有永久嵌套例 `text_record_native.rs:643..670` 只在第一条 if 的全真路径 return；注释所称第二条 skipped if 根本未执行，且没有内层 take→skip 重入。通过一个命名为 nested 的测试不证明标记可组合。

修复方向：合流时保留已有 Agg.present，只有无条件创建的 Agg 才传 1；外层 skip 传 0。覆盖 &&/|| 多层、take→skip/skip→take、ctor/clone/owned-return 代表路径及共享回滚释放器；保留完整语句尾寿命和统一 LIFO。

## F-782-R2-02 [P1] 字段赋值错误复活已完整 move 的根

AC-03/04，§5.2 完整根 move / 不支持部分字段 move、§5.3 唯一责任，V-03/05/06，T-04/05/06/07。属于 F-02 修复相邻状态缺口；不要求新部分初始化能力。

`text_resources.rs:1749..1750,1826..1838` 将字段赋值一律记作 reown_root，随后清除整个根 dead 状态。RHS 已 move 整根时，只写回一个字段不可能恢复其他字段的拥有责任。Str 分支新加的 moved 扣除进一步明确放行该路径；嵌套 record 分支还会释放已转给 callee 的旧字段。

- `partial_reown.at`：T 有 a/b 两个 Str，`fn extract(move x T) Str { return x.a.to(Str) }`，`r.a = extract(r.move)` 后读取 `r.b.len()`。应前门拒绝对已移出根的部分重建；实际 build=0，callee FREE2/FREE1 后读取 b，RT-FAIL -7 / exit70。
- `nested_reown.at`：O 有 i:I 与 b:Str，`fn extract(move x O) I { return x.i.clone() }`，`r.i = extract(r.move)`。build=0、exit2，但 callee FREE2/FREE1 后，替换 FREE-INVALID，最终根清理又 FREE-INVALID。即使后续只读取新 i，责任仍不闭合。
- `field_live_control.at`：存活根上用独立 owning RHS 替换字段，exit3，旧字段和最终两字段各一次释放。
- `whole_rebuild_control.at`：`r = rebuild(r.move)` 完整返回重建，exit3，两个 buffer 出口各一次释放；R1 move-return 原例同样通过。

修复方向：按现行完整根状态合同，禁止字段赋值恢复 RHS 或先前语句已 move 的根；只有完整根重赋才可恢复。覆盖 Str/嵌套资源 record/标量字段位置、同语句与跨语句 moved 目标，同时保留存活根字段替换、完整根 move-return 重建与异根合法操作；不要通过仅隐藏 FREE-INVALID 或跳过清理放行悬空根。

## 上轮闭合与本轮范围

| R1 finding | 原文/现有定向结果 | 完整不变式 |
|---|---|---|
| 01 嵌套资源隐式复制 | 三原文前门拒；clone/copyable 正控通过；拥有形态门四入口已查 | 本轮未再发现该隐式复制缺口 |
| 02 move-return 重建 | 同根完整重建原例与 fresh 对照均正确 | 字段部分重建仍错，R2-02 |
| 03 短路聚合临时 | 原始单层 loop 原文闭合 | 嵌套标记不组合，R2-01 |
| 04 混合 LIFO | 原文及永久双向控制正确；单一 pending/出口计划消费已查 | 本轮通过已检面 |
| 05 引用越界 | 真实 API 9 控制及永久变异通过；结构门先行已查 | 本轮通过已检面 |
| 06 故障注入/oracle | 5 个真实 native V07 全过；wrapper 控制调用真实 runtime，部分 clone 槽置零、status<0 回滚后 fatal | 两 Str 第二字段故障/预算/ledger/零长已有实证；不是语言 Result/unwind；未声称所有 live 根在 fatal 前 Drop |

未止于首个错误：本轮检查全部修复实现与 V07 的真实连接、拥有形态/引用门、统一清理消费及 SD01..06；深256/4096/零次/Token/身份等在86测试中实际重跑。V07 “8 vs 6 差1”措辞实际是少一次2字节请求空间，属于收据表述，不能称差1字节；工作应更正措辞或用7作为一字节短控制。

## 验收、规范与路由

AC01/02：已检身份/拥有形态/名义构造面 pass；AC03 fail（R2-02）；AC04 fail（R2-01/02）；AC05 已复验深输入与畸形门 pass，发布面沿历史证据未全重跑；AC06 partial；AC07 本轮 final non-pass；AC08 pending merge。
SD01..06 路径与有限 profile/non-goal、touched_goals=[] 理由保持；SD01/02/04 支持完整状态与路径标记的实证、附A“零缺口”仍被反例否定，提案不批准发布。冻结副本及hash随本报告保存，不改 canonical。

outcome: **needs_fix**。重开 T-04/05/06/07，保留 T-01/02/03；status=executing，current_step=3/9；T-08/09 pending。两项为合同内阻断，不以登记债务替代修复。新基线需受影响定向复验和 fresh final review；通过后 merge-owned 联合门禁才可执行/合入。未合入、归档或清树。

本计划已连续两次 final needs_fix。下一次交接前，work 须补诊断表（remaining invariant / earlier miss / paths/states / design-or-oracle adjustment / current result）：分别说明为何“嵌套但仅全真且提前 return”和“仅完整重建”测试遗漏组合状态，并用真实分支 trace/前门拒绝永久断言验证调整。不是仅再跑总数。
