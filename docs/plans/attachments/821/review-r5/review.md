# PLAN-821 R5 — R4三项P1定向复核通过；整体needs_replan（V05待决）

2026-10-26；Plan revision5；受审 `40f4c594fd6cfdc8d051a980c64f5e57baa67e4a`，前审 `5fc1a0aa2d9848a9d6e652cb0798c8c0719fa653`。同一实施树 `D:/autostack/.wt/lang-821/auto-lang` / plan-821-dev，入审/出审clean。当前Codex审查聊天延续R1–R4，未实施修复/未委派。

[baseline](baseline.json)、[入审Plan](plan-r5-at-entry.md)、[冻结SD](spec-delta-frozen.md)。原Spec/依赖未变的证据继续有效，本轮只重核修复影响和此前pending的恢复路径。

**本轮确认R4三项P1的反例及共享修复成立，无新增P1。** 不发Plan整体最终pass：V05仍是具名未完成的work-owned必需oracle，与现指令集可执行面存在合同差异，且当前Plan§10明确要求new/用户决策。总体 outcome=needs_replan，保留执行态等待该决策；不是又把三个已关闭P1退回修复。

## 真实检查

独立重放R4完整probe，副本保留在本轮附件而未修改旧证据：

```powershell
cargo run --quiet --manifest-path docs/plans/attachments/821/review-r5/probe/Cargo.toml --target-dir D:/autostack/.wt/lang-821/auto-lang/experimental/ac-av-probe/target
```

[源码](probe/src/main.rs)、[输出](r4-probe-replay.txt)。源码和输入来自R4已冻结反例；按当前受审代码重新编译，没有用旧binary。

在实施树：

```powershell
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_owned_821_incremental --test av_owned_821_r1_replay
cargo test --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --test av_owned_821_cache_compat av_owned_821_r1_ -- --nocapture
cargo check --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe
git diff --check 5fc1a0aa2 HEAD
```

7增量+12永久回放+2新进程恢复=**21个定向测试通过，0 ignored**。[测试raw](scoped-tests.txt)、[恢复raw](restore-checks.txt)、[check](check.txt)。check与测试编译无警告，diff-check通过。没有重跑无关全族、总acceptance或merge-owned整档。

## 三项P1的独立关闭

| ID | 关闭证据/边界 |
|---|---|
| F821-R1-07（R4坐标残项） | 同main连续变长1234→8、前方helper变长后编辑后方main107→108均成功，且保持SingleBody。EnvStep::CommitBody携带decl_idx/old_hi/delta，在成功发布时更新自身hi、后续free函数和后续type坐标；失败不平移。永久族含缩回、整轮后恢复单body等相邻控制。原whole前端缺口、事务旧例证据保持，R4具体残项CLOSED。 |
| F821-R1-08（R4字段状态残项） | typed整根重新Aggregate后FieldMove成功；父字段移出后子路径在MIR准入以use-after-move拒绝；mut循环源程序结果9。define_slot统一定义点清init/moved/dropped/moved_fields，field_moved按双向前缀重叠判；永久例包含反向父子、合法兄弟。808/810/8917/801四种源码正控仍过。R4具体残项CLOSED。 |
| F821-R1-09（R4载荷残项） | 同一3,035,011B合法wire的hot估算现为**9,431,319B**，超过8MiB且覆盖独立载荷下界**9,068,791B**。RecordView按Record的动态结构计费，FieldMove路径Vec纳入；ABC解码同步计字段表/字段名字节。源码新分支与永久断言相符，R4具体漏计CLOSED；R4已验证四线量纲/同门控制证据保留。 |

合法130层Clone继续Success；单次body编辑whole-frontends1→1/gate绿；运行Trap候选revision/production不变、旧结果7；health R2-01保持CLOSED。本轮未发现需要重开这些已闭输入的新回归。

## 此前恢复pending的定向核对

F821-R1-10新worker缺口：新增真实`owned-restore-run`路径及自洽外壳坏内层测试，已在当前受审HEAD独立执行2/2通过。worker为当前源码fresh release构建，恢复走真实store/envelope/decode/深验，坏内层从store进入新进程后被拒，dispatched=false；合法record正控通过。原“无新进程恢复入口/只在内存翻checksum”两缺口CLOSED。

适用性边界仍须如实保留：这些两例不代表任意类型/状态组合的完整恢复矩阵全部重跑；原不受影响case沿此前证据。当前没有把测试kind断言扩大成新oracle能力。

## 未决项：V05合同差异，不能由review默示豁免

原Plan§6/V05及contract-decision§12要求：普通整数与资源id碰撞、旧帧复用、跨store、generation边界，通过typed AV句柄表实际观测无效访问在读取前受控拒。其 owner_stage 是work（T03/T04/T06），不是merge-owned未来操作。

当前as-built可检查事实：

- MutRefV是frame/slot/gen的值，VM创建它、静态ownership门约束权限；指令集中没有解引用此token来读写目标的消费入口。
- StrView通过Share/Static直接持有buffer，不沿MutRefV的frame/slot/gen解析；这两种view的生命周期证据不能冒充“旧帧/跨store句柄解引用拒绝”。
- 源mut record通过受检move-in/包装结果/回写实现，运行期也不通过该token解引用。
- 因而现有类型隔离、loan准入反例与整数正控提供的是实际静态/拥有权机制证据，没有执行原V05所写的runtime探测。当前Plan§10和SD已承认该差异且未获批准修订。

需要在new阶段明确一个可执行合同，不能由review添加新指令、改变验收面或把没有的探测写成pass：

1. **按现有支持面修订矩阵**：验证类型隔离、静态loan/owner门与实际代际状态；明确未交付运行期失效句柄校验，不把未验证的运行期generation退役/跨store访问承诺留在Spec。
2. **保留原runtime要求，补测试专用故障注入和必要生产校验接缝**：规划时需先证明探针到达真实消费/校验路径，不能只造一个测试模型证明自己。范围/设计重新冻结后再work。
3. **增加受检句柄解引用能力**：属于新机器能力，先new明确语义和准入，再实现与复审。

建议1：它与当前无解引用支持面的实现相符，避免为一个不存在的访问路径新增机器能力。但这是原验收合同的明确修订，需要用户选择，当前review不代作决定。未决前不发reviewed、不合入/归档。

技能依据：[auto-plan-review](C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md) 的原文："Known unfinished work-owned criteria or required oracles return to work, even if the receipt says execution_done." 本轮限制来自V05原Plan合同与这一就绪性要求，不是额外安全审批。

## 状态和下一步

- 三项R4 P1修复定向通过，之前有效证据及本轮21个检查保留；没有以换owner/换HEAD重复整档。
- V05未解决的实现/资源探针/汇总责任对应T03/T04/T06，三项仅就该具名义务重开；T01/T02/T05/T09保持完成。**executing4/9**。T07最终验收、T08merge仍未完成。
- F11其余占位/现状已改善，但SD仍不能在V05合同未决时最终冻结。当前的AC02/03/04及SD01/02相关承诺是partial；AC07不能pass，AC08仍merge-owned。其他完整验收没有因这一轮“3P1通过”自动获最终pass。
- 用户决定后，new更新准确合同revision、V05/相关AC/SD和contract-decision一致性；保留本轮有效证据，按变更影响完成剩余验收，不从头重跑全部案例。

未修改实施、canonical Spec/ledger，未合入/归档/清树。最终代码/验收/Spec通过后，统一裸cargo t+tv及实际触面仍归merge；tf保持批量主检出到期单实例。
