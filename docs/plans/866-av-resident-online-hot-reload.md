---
plan_id: PLAN-866
status: drafting
feature_name: AV常驻在线热重载（RF-H）
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-av/resident-online-reload.md
touched_goals: [GOAL-002]

affects: [auto-av, auto-mir, auto-test-case]
current_step: 0
total_steps: 9
---

# [PLAN-866] AV常驻在线热重载（RF-H）

## 0. 变更摘要

[Design43 §6.1](../design/43-ac-av-typed-mir-and-abc.md)把“解释器在线热重载”列为ACE必须**较早**落地的结构责任，并要求“早期最小AV profile就可单独验证，不依赖JIT或完整语言；**缓存恢复/下一次重新运行/只验证generation标签不能当在线替换通过**”。[AV源码路线 §4](../design/strategy/av-source-programs-and-avm-replacement-roadmap.md)的工作包 **RF-H** 至今未被任何Plan领取。

现有交付的边界：PLAN-828 已给出 prepare/commit 事务、新顶层调用取新 snapshot、在途调用链 pin 旧 snapshot、旧代在引用期间保活、每函数内容史有界（HISTORY_CAP=32），并有 852 的 I07“旧调用暂停/恢复”证据。但 828 的验证形态是**在两次顶层调用之间**（PausedRun/驱动逐次请求）——这不能证明“持续运行的 guest 在编辑到达后不停机地采用新版本”。

本Plan交付这一缺口：一个**持续运行**的常驻执行形态，在冻结的安全点上看版本、在途链留旧代、坏/迟到候选不打断当前运行、旧代有界释放，并给出可反控的重载可见性证据。它关闭 852 binding 中 I08 行的 **AV 列** `gap:av-online`，而 I08 的 **mn 列** `gap:ac-native-loader` 保持（native 在线装载另属能力包）。

不含：native 在线装载/热替换（I08，需 native loader 能力包）、任意活帧迁移、OSR/deopt、对象状态跨布局迁移、产品 CLI/UI 默认切换、完整 AJ。

## 1. 目标

- 定义并实现**常驻运行（resident run）**：guest 以燃料切片持续推进，宿主不因编辑而暂停它；编辑以候选形式到达，在冻结的安全点生效。
- 冻结**版本可见规则**：哪个安全点之后的新调用/新迭代看到新版本；在途调用链看到什么；不出现“同一链里新旧任意拼接”。
- 失败/取消/迟到/外来候选：不覆盖当前可用版，不产生部分安装；可观察拒绝原因。
- 旧代回收有界：pin 全部释放后旧代码/旧 env 被释放（Weak/句柄 oracle），常驻进程总内存与旧代数量有上限。
- 证据形态必须是**在线**：程序不重启、不重跑、不靠“下一次运行”；用负控证明“偷偷重启/暂停/只换标签”必红。
- 重载可见性按现行开发期周转口径测量（[TestCase v2](../specs/auto-test-case/profile-v2-native.md)：P90≤2000ms），并按结构门报告“未改函数零重做”。

## 2. 架构方案

```text
source(编辑) → 候选 prepare（复用828事务/精确失效）→ 候选就绪队列
                                                         ↓ 安全点
常驻 guest（分段执行，fuel 计量） → 版本选择器 → 调用绑定 → 单元执行
                                                         ↓
                                    pin/lease（旧代保活）→ 释放即回收（有界）
```

| 组件 | 本Plan动作 | 复用/边界 |
|---|---|---|
| 常驻执行器 | 新 driver 形态：按 fuel 切片推进、不打断 guest、可取消/可停止 | 复用 `execute_program`/`resume_program` 与 825 的预算账本；不新写解释器 |
| 安全点 | 冻结定义（候选：guest 调用边界 + 常驻循环回边）；选版动作只在安全点发生 | 不做任意 block/语句级切换（Design43 §8.1 属后续） |
| 版本选择器 | 逻辑入口 → 当前 revision 的代码绑定；在途链固定旧 snapshot | 828 `ProgramSnapshot`/pin 语义 |
| 安装 | prepare/commit 事务；失败/迟到/外来拒绝 | 沿用 828，新增“运行中安装”的准入点 |
| 旧代寿命 | lease 计数 + 有界历史；释放即回收 | 828 HISTORY_CAP 形状，不无限保活 |
| 观测 | 安装集合、每站点代际、旧代存活数、重载可见迭代号/时间、生产阶段计数 | 852 计数器口径，新增在线专项计数 |

## 3. 技术栈

- 核心：`experimental/ac-av-probe/core/src/{program_snapshot,function_code,av,loaded_program,contract}.rs`。
- 消费者：`experimental/ac-av-probe/probe/src/{session,driver,formal_worker,worker_restore}.rs`；拟建 `resident.rs`（常驻执行器）与 `resident_churn.rs`（编辑/回收压力）。
- 语料：`crates/auto-lang/test/cases/ace-m1/`（新增常驻/在线行，不改既有行语义）、`crates/auto-lang/src/tests/ace_m1_tests.rs`。
- 拟建 `scripts/verify-av-866.ps1`（Scope=targeted/compat/resident/acceptance），前缀 `av_resident_866`；当前不存在。

唯一实施树 `D:/autostack/.wt/lang-866/auto-lang`，分支 `plan-866-dev`。

## 4. 需求分析与背景调查

用户 2026-10-28 授权同上（只规划）。事实基础：

- [resident-function-install](../specs/auto-av/resident-function-install.md)：prepare 只构造候选；commit 在同一发布点原子更新 root/source/revision/env；失败/取消/迟到/foreign 不覆盖当前版（session_id/base_revision 守卫）；新顶层调用取新 snapshot，在途调用链（PausedRun/pin）固定旧 snapshot；旧代码在 snapshot/frame/lease 引用期间保活，最后 owner 释放即回收（Weak oracle）。
- 同文“安装事务与寿命”节的 HISTORY_CAP=32 与“驻留当前条目永不逐出”给出有界历史的既有形状。
- 852 语料 I07 = “AV旧调用暂停/恢复” `ok`；I08 = “native在线热重载” `gap:ac-native-loader`/`gap:av-online`——本Plan**只**关闭 AV 侧，native 仍在缺口。
- 852 conformance 明确：“native 在线 HMR（I08）不由本规范声明——需要 native loader 能力包，重启 exe 不满足”。同一标准适用于 AV：重启进程/重新运行不满足在线替换。
- [TestCase v2 周转口径](../specs/auto-test-case/profile-v2-native.md)：开发期有效样本 P90≤2000ms 硬门，P95/P99 仅诊断；本Plan的重载可见性按此口径报告，不写旧 P95<1s 承诺。

## 5. 详细设计

### 5.1 前置与写面

硬前置：`D828`（安装事务/寿命）、`D862`（入口/结果通道，常驻运行需要可判定的入口与结果）。与 863/864/865 **无依赖**，可并行。写面见 §3；不触碰 native 路径、产品入口、852 binding schema。

### 5.2 冻结项（T-02 决策件）

`attachments/866/work/resident-reload-contract.md`：

| 决策项 | 要求 |
|---|---|
| 常驻形态 | 单 guest 一次启动持续运行；宿主以 fuel 切片推进（每切片上限）+ 取消/停止；切片边界**不是**版本生效点（否则等同暂停） |
| 安全点 | 冻结为（建议）**guest 调用指令边界**：一条调用完成后才允许换版；常驻循环回边是否也算安全点由 T-02 定，但必须给出可观察的“第 N 次迭代起生效” |
| 可见规则 | 安全点后的新调用看到新 revision；在途调用链整体留旧 revision 至返回；同一链不得新旧混拼（含闭包/env） |
| 候选生命周期 | prepare（可失败）→ 就绪 → 在下一安全点 commit；过期（基础 revision 已变）候选作废；外来会话候选拒绝 |
| 失败面 | 候选编译/验证失败不打断运行；运行期 trap 仍按语言语义呈现；安装失败与 guest 失败分列 |
| 回收 | pin/lease 归零后旧代码与旧 env 释放；同时存活的旧代数量上限与总线预算写死；释放 oracle 用 Weak/句柄 |
| 计量 | 重载可见延迟（编辑时间 → 首次观察到新行为的时间/迭代号）、安装集合、未改函数生产计数=0、旧代峰值 |
| 禁止替代 | 重启进程、重新运行、暂停-恢复、只改 generation 标签——都必须被负控判红 |

### 5.3 与 G2–G6 的关系

本Plan只证明**机制可用**（最小profile、单文件源、标量/入口语义）。UI/服务常驻会话（G5/G6）的真实产品形态另立计划，且必须消费本Plan的可见规则/回收合同，不得各自发明“热重载”定义。Design43 §6.1 的“数据与跨层寿命”“并发与发布一致性”两项在本Plan按标量范围交付，复合值/活对象迁移留后续。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/resident-online-reload.md | 无 → 常驻运行/安全点/可见规则/失败面/回收/计量 | 在线替换合同 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-av/resident-function-install.md | 追加“运行中安装”的准入点与安全点映射，原有暂停态语义保持 | 事实扩展 | AC-02/03 |
| SD-03 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 新增常驻/在线行（I07 保持，I08 仍 gap） | 证据落点 | AC-05 |
| SD-04 | modify | docs/specs/auto-av/cache-and-conformance.md | 追加在线重载的预算/计数口径，不放松既有门 | 计量一致 | AC-04/05 |
| SD-05 | modify | docs/specs/auto-av/project.md | “798不是在线函数替换”的边界按本次实际交付更新 | 事实纠正 | AC-05 |

## 6. 测试设计

| V | AC/不变量 | 实际/拟建入口 | 可区分错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 常驻运行 | 常驻循环程序，多 fuel 切片连续推进；取消/停止 | 计数连续（无重启）、切片边界不改变行为；重启即红（进程启动计数/世代证明） | T-03 |
| V-02 | AC-02 在线安装 | 运行中编辑被调函数 body，观察生效迭代号 | 精确“第 N 次调用起新值”，不是“最终变了”；暂停-恢复实现必红 | T-04 |
| V-03 | AC-03 在途保活 | 长调用进行中提交新版本，新旧值可区分 | 在途链整体旧值、后续新值；混拼（新 callee/旧 caller）必红 | T-04 |
| V-04 | AC-04 失败/迟到/外来 | 坏候选、过期 revision 候选、foreign 会话候选 | 运行不中断、当前版不变、拒绝原因可观察 | T-04 |
| V-05 | AC-05 回收/预算 | pin 释放后旧代释放；多轮编辑 churn；旧代上限 | Weak/句柄 oracle 证明释放；超预算受控失败；无限保活必红 | T-05 |
| V-06 | AC-06 可见性与计量 | 重载可见延迟样本（按 v2 口径）+ 结构计数 | P90≤2000ms（样本不减、缺失不计绿）+ 未改函数生产=0 | T-05 |

定向命令：两独立 workspace 定向 `cargo test`；`scripts/verify-av-866.ps1 -Scope <targeted|compat|resident|acceptance>`；完整档归 merge。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 常驻运行成立：连续推进、取消/停止正确、无重启；V-01 |
| AC-02 | work | 在线安装按安全点生效，生效迭代号可精确观察；V-02 |
| AC-03 | work | 在途链留旧、后续用新，无新旧混拼；V-03 |
| AC-04 | work | 坏/迟到/外来候选不打断运行且当前版不变；V-04 |
| AC-05 | work | 旧代有界回收与预算成立（Weak oracle）；V-05 |
| AC-06 | work | 重载可见性按 v2 口径报告 + 未改函数零重做；V-06 |
| AC-07 | review | 独立核 AC/V/SD、安全点定义是否被实现规避、计数是否真实生产点、warnings/fmt |
| AC-08 | merge | 冻结提交裸 `cargo t` + `tv` 通过；失败不豁免 |
| AC-09 | merge | Spec 沉淀、I08 仍标 gap、ledger、归档、guard 清理完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | D828、D862 | 核实 828 安装/寿命/计数实现点、AV 执行分段能力、852 I07 形态；产出 baseline | AC-01/02 | [ ] |
| T-02 | work | T-01 | 冻结 §5.2 全部决策（安全点/可见规则/回收/计量/禁止替代） | AC-01..06 | [ ] |
| T-03 | work | T-02 | 常驻执行器（切片/取消/停止/观测）；V-01 | AC-01 | [ ] |
| T-04 | work | T-03 | 在线安装与在途保活/失败面；V-02/03/04 | AC-02/03/04 | [ ] |
| T-05 | work | T-04 | 回收/预算/churn + 计量脚本与语料；V-05/06 | AC-05/06 | [ ] |
| T-06 | review | T-05 | 独立最终review；merge 门 pending | AC-07 | [ ] |
| T-07 | merge | T-06 pass | 冻结提交裸 `cargo t` + `tv` | AC-08 | [ ] |
| T-08 | merge | T-07 | Spec/ledger/归档/guard 清理，I08 保持 gap 的事实登记 | AC-09 | [ ] |
| T-09 | merge | T-08 | 交接 G5/G6 常驻会话与 native loader 需求 | AC-09 | [ ] |

## 9. 复审记录

2026-10-28 new r1：`stage: new | plan_id: PLAN-866 | plan_revision: 1 | outcome: pass（等 D862；与 863/864/865 无依赖） | next: D862 后 work T-01`。全部 AC/V/T 未执行；安全点具体形态、旧代上限、延迟预算数值待 T-02 按既有预算口径冻结；未建树、未跑构建、未改在途Plan。批次排序依据见[后继批次规划](reports/ace-next-batch-2026-10-28.md)。

## 10. 待澄清事项

- 安全点是否包含常驻循环回边（“同一次调用内换版”）。默认**只**在调用边界换版（保守、可解释）；若要回边换版，须先冻结边界状态兼容合同（Design43 §8.1 的显式block阶段），本Plan不预支。
- 常驻会话是否需要持久化（跨进程恢复后继续同一次“运行”）：默认不需要，跨进程恢复仍按 828 的 store 重验语义，不宣称“运行状态恢复”。
- 旧代同时存活上限/总内存预算的数值需按 798/852 既有口径在 T-02 给出，本草案不编造。
