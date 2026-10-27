# PLAN-828 r2 — f1cd60778 / F-05b 独立复审

2026-10-21。**outcome: needs_fix**。取消与关闭后的入口拒绝、旧调用链保活通过；关闭释放仍有一项 P2。24 格三函数矩阵收编通过。计划保持 executing0/11，本次为用户指定的受影响面复审，不是整份最终验收。

## 基线

- 主计划读取自主检出，revision2，当前 work 记录 in_progress。原实施树 `D:/autostack/.wt/lang-828/auto-lang`、分支 `plan-828-dev`；受审 `f1cd60778f5438beecc3e35bb4b2afb56a9030fc`，修复基面 `3d14a15d613dbd79724c7dd0ac24917dc5169a28`，原合同基面 `f79b1b83591310b45b3828dd300e6d368987d6c0`。
- reviewer 为本聊天 Codex `/root`，未实施本提交；依据代码及独立运行判断，未创建 agent/聊天，不虚构模型或会话身份。
- 实际 diff 仅 session.rs 与永久回归文件。入口、冻结状态实现 clean；auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` 未消费。[冻结输入/依赖/规范/库哈希](baseline.json.txt)、[原 Plan](plan-input.md)、[结束状态](final-state.json.txt)。实施全程只读。

## F05b-C01 / P2：close 保留没有外部持有者的当前 snapshot/HIR/代码

**既有要求**：主计划 §5.4 要求“关闭、trap、fuel、cancel、安装失败均验证释放”；V-06 要求真实引用/lease释放。R1 F05 已明确“取消候选不能安装及关闭释放”；本次 close 注释还声称关闭后不持有 reuse state。因此关闭释放不是新产品范围或增加的规模承诺。

**位置**：`experimental/ac-av-probe/probe/src/session.rs:765–770`。close 设置 closed、释放 store、清空各 rec.history，但 `self.current` 仍强持有 ProgramSnapshot（含 merged 执行载荷），各 FnRecord 仍强持有当前 hir_body、code，关闭 session 继续持有整套当前程序。

**真实反例**，[源码](close-controls.rs)、[raw](close-output.txt)：

- `close_review_releases_unpinned_current_snapshot_root`：warmup 后仅保存当前 root 的 Weak，立即销毁 getter 返回的临时 Arc，没有外部 pin/调用链；close 后 Weak 仍可升级。drop session 后 Weak 死亡。实际 `after_close=true, after_drop=false`。
- `close_review_releases_unpinned_current_hir_and_code`：仅用 resident 的临时 Arc 建立 HIR/code Weak，强引用在局部块结束时全部销毁。close 后两者仍存活；drop session 后均死亡。实际 `hir_live=true, code_live=true`。

两项失败属同一关闭所有权遗漏，只有一项 P2 finding。已另设正控证明**退役历史代码确实被 close 释放**，以及**真正暂停的调用链在 close/drop session 后仍能按自身 snapshot 和正确来源坐标完成**，所以不能以“保活旧链”解释没有外部所有者的当前资源残留。历史条目数为0只证明 history.clear，不能证明 session 当前资源释放。

**修复要求**：终态关闭时撤销 session 对当前执行根、HIR及代码的拥有引用，保留必要轻量状态；外部已 pin 的 snapshot/PausedRun 继续自身保活。保留取消/关闭入口行为，明确关闭后的访问接口行为，避免取回已释放资源或 panic。将这两个 Weak 反例与已通过的退役历史、实际暂停链正控收编，在同树提交后复审。此处只要求内存所有权释放，不将 F-10 的完整 store GC/A07/预算工作并入本轮；F-10 已登记 OPEN 也不能代替 F-05b 的关闭释放要求。

## 已验证闭合的子项

| 子项 | 本轮独立证据 |
|---|---|
| cancel_prepared 不安装 | 同基面准备三个候选，取消一个不改变 root Arc/source/revision/结果，另一个仍能提交；再取消迟到候选不覆盖新结果。计数正确。候选按值消费，私有且不可 clone，取消路径不进入安装 |
| 跨会话取消 | 真实 foreign PreparedEdit 被拒、计数不增；与 commit 同会话身份规则，入口判断源码核实 |
| close 终态守卫 | prepare/edit 即使非法 MAX 坐标也优先报告 closed；commit/cancel 两个真实已准备候选均拒；run 返回受控 Infra；重复 close 幂等，source/revision不变 |
| 历史与 store | history 归零且退役 code 的 Weak 真正死亡；store=None 的实际释放赋值已核查。当前 root/HIR/code 的释放不通过，见 F05b-C01 |
| V-05 实际在途链 | 先改变前方函数长度，独立当前源冷编取正确 span；execute_program_steps 确认进入 Paused 状态，close/drop session 后 resume，完整 SpanInfo 与当前源冷编一致。不是只在 close 后重新调用 pinned snapshot |
| r22a 矩阵收编 | 逐项比对与 reviewer r21a 原相邻矩阵：24个组合、合法非零func1、真实mutation、session/公开执行、第一次 admitted计数及第二次恢复断言均保留；永久测试实际运行通过，独立冻结原矩阵再次24格通过 |

## 本轮运行

cwd 均为原实施树，单线程定向测试；[run-controls.ps1](run-controls.ps1)链接本轮 cargo 库，精确库哈希冻结。

| 命令/入口 | 结果 |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | **85/85**，exit0，6.08s；[raw](permanent-output.txt)。两个 f05b 和 r22a 永久测试均确实执行 |
| `run-controls.ps1 -Source close-controls.rs -Filter close_review_` | **4 pass / 2 fail，exit101**，0.03s；[raw](close-output.txt)。两个失败同一关闭释放机制；87个被过滤例未算本次新增覆盖 |
| `run-controls.ps1 -Source adjacent-controls.rs -Filter adjacent_review_` | **1/1，24格全过**；[raw](adjacent-output.txt) |
| `run-controls.ps1 -Source original-replay.rs` | **62/62**，exit0，8.08s；186/528/12/4/9/72各矩阵全过；[raw](original-replay-output.txt) |
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828 -- --test-threads=1` | **17/17**，exit0，8.07s；[raw](original-suite-output.txt) |
| `rustfmt --check --edition 2021 --config skip_children=true` 两个修改文件；`git diff --check 3d14a15d6 HEAD` | 均exit0；[fmt](fmt-output.txt)、[diff](diff-check-output.txt) |

各集合有重叠，不合计声称独立case总数。未重复 core22/798兼容和其余独立旧 sidecar过滤集：本轮只增加 session 终态/取消守卫并收编测试；core、cache谓词、依赖和配置未改，旧完整证据保留为历史。本轮重跑了包含这些修复的永久套件和实际 session/restore/执行路径。新增关闭行为没有沿用历史证据，使用本轮独立控制重新验证。

健康：原有 probe 四 warning仍在（SpanInfo、new_len、context_revision、history_entry），归 F-14；依赖 shim-metadata doc comment旧warning单列。无本轮新增告警；未以格式通过声称整个计划已零告警。

## AC / SD / 状态与下一步

- F-05b 为 **PARTIAL/OPEN**：取消/入口终态和 V-05 保活子集 PASS，AC-06/V-06 的关闭释放 FAIL；T-06/T-07 未整体完成。r22a永久矩阵收编 PASS。F11-S01/F10-S02 来源/修复已闭不变，未因本次生命周期失败重新打开来源算法。
- 当前[规范增量](spec-delta-input.md) SHA256 `ed0077a2a9bdf622db527d59bf4e12bbfaefb1df4f51df212768cb041d8a206e`；与 r21a 相同，相关 canonical Spec/锁/配置哈希变化0。SD-04寿命规则尚受当前失败阻断；其他SD维持既有状态。impact元数据/GOAL-002与revision2保持，无授权缩面、无canonical/ledger写入或整份delta pass。
- 原永久测试只断言 history_len，未用无外部 pin 的当前 root/HIR/code Weak 作释放 oracle，故漏掉 close 后仍持有 self.current/fns 的路径。下一轮应按 **session拥有引用 / 外部pin / PausedRun / 历史条目 / 候选 / store** 分别审查生存期，先补关闭拥有引用，再重放本包两个失败和四个已通过控制；不把清空容器或最终 drop(session)当 close 完成。
- 保持 executing0/11；其余 F-01/02/09/10其余/11其余/12/14 OPEN不变。下一责任角色为同树 work，修 F05b-C01并提交；本次请求仅复审，未代修实现。
- 整份 AC-09最终验收仍pending；最终代码/验收/规范通过后，AC-10 merge-owned冻结裸t+tv及必要consumer/触面、Spec/ledger/归档/guard仍必需。本轮未运行整档、未合入、归档或清树。
