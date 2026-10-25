# PLAN-848 r3 — affected independent final review: pass

stage: review | review_scope: final affected | plan_revision: 3 | outcome: pass

审查者 `/root/review_848_r3` 是上轮独立最终审查者，本轮未参与 work-r5 实施，按用户授权继续只读受影响复审。前次 capacity 错误没有产生审查证据；本轮从当前 Plan、review skill、四文件 diff、原始输入/输出及独立运行重新判断。未创建代理/工作树；未改生产实现、永久测试、共享 Plan、canonical Specs 或 live ledger；未提交或合入。

冻结 `dd7076a7f40b35ffda6b876c36f826af3443fb97`，parent/上轮独立 pass `6bc5abc6080b2261a43f55a54f127cd779239edb`，Plan base `bf22db3465203f3babd649494f4b37aa2a012027`。唯一树 `D:/autostack/.wt/lang-848/auto-lang` / `plan-848-dev`，worktree inventory/HEAD/branch/clean 在始末核对。SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`、AGP/三锁、10份Spec输入hash与上轮独立基线逐项相同。详细记录 [baseline.json.txt](baseline.json.txt)。

## M848-NATIVE-BACKLOG — CLOSED

这是原生验收驱动在合法 clock auto-Pause 清空输入后，继续按固定帧延迟评估尚未接受刺激的问题；没有证实之前不可控停顿的唯一外部来源，也没有把产品暂停政策判为缺陷。work-r5 的可控停顿前后对照及本轮独立兄弟输入证据支持当前归因。

四个文件仅为 `src/bin/authoring_probe/play.rs`、新 `tools/check_play_clock_pause.py`、handoff、source receipt374。没有生产 clock/Host/Auto/SDK修改。逐行检查结果：

- 仅 live-input phases 接受精确两种理由：`tick backlog >4` 或 `display gap >250ms`，且必须实际 Paused。恢复使用同一个原生点击函数点击 Iced Resume；随后等待 Running，恢复原输入目标。wrong-focus 保持错误目标，3D held phases 恢复3D目标，文本phase13恢复真实Iced path输入。
- 保留生产250ms/4credit门、100000steps/120resumes/250ms callback、8ms pump及原frame deadline。360帧阶段截止检查发生在恢复分支前；恢复路径没有改 `self.at`，只有原阶段真正成功才递增phase/重置at。Fault先拒绝；显式Pause/Step/焦点丢失阶段不在新增恢复范围。没有无限恢复后冒充成功或扩大deadline。
- 单V的接受凭据是发送时tick之后实际accepted report的toggle；只有未接受batch重投，已接受batch不重投。两V偶数批以后续accepted tick确认并保留原visible/parity断言；单V隐藏/恢复均先等接受，再做原visibility与实际像素验证。没有将placeholder/未接受动作当成功。
- 2D至少5、3D至少90真实动作仍由原独立trace oracle检查，新等待使刺激到达该原阈值再评估；原f32逐tick结果、连续tick身份、6阶段×6像素子断言、Step exactly-one、Pause、Stop恢复与新Run身份未删减。
- phase22先观察实际focus-loss Paused，再请求真实窗口focus以便原生Stop点击。未替换为Host Stop；原WindowFromPoint/PID/物理指针到达和遮挡拒绝仍在。焦点policy没有被恢复分支抹掉。

## 本轮独立选择并执行的反例

均为测试用 native entry/play 副本，以当前未变rlibs连接，执行实际OS→Iced/router→Auto→GPU路径；没有在实施树写入探针，也没有修改运行阈值。rlib逐文件hash与work-r5最终控制相同且运行时守卫；binary与完整命令保存在各 results/process 记录中。

| 独立输入 | 预期与实际 |
|---|---|
| phase13文本焦点前停顿300ms | 实际 `display gap >250ms` 自动暂停；通过OS Resume和文本焦点恢复，panel V隔离、原trace/动作数、全部6×6像素、Step/Stop/保存重开通过，phase27 |
| phase10已按住3D右键时停顿100ms | 实际 `tick backlog >4`，恢复3D目标与held key；≥90真实3D动作及完整原oracle通过，phase27 |
| wrong-focus phase3停顿100ms | 观察clock恢复，但错误目标未被“修好”；仍在phase5以 `wrong-focus: no real 2D actions` 拒绝。顶层control exit0代表预期拒绝成立，原生runner是失败结果 |
| 已接受V后停顿，副本强制重新投递该batch | 仅把恢复处 `(!self.v_accepted(h)).then_some(edges)` 改为 `Some(edges)`；实际重投后phase17原 `single V must hide Auto actor` 断言失败，runner exit1。确认接受凭据不是无区分装饰，也没有用新oracle替代原拒绝条件 |

记录：[independent-os/results.json](independent-os/results.json)、[independent-os.log.txt](independent-os.log.txt)、[accepted-replay-mutation/results.json](accepted-replay-mutation/results.json)、[accepted-replay-mutation.log.txt](accepted-replay-mutation.log.txt)。可复现驱动 [independent_controls.py](independent_controls.py) 与 [accepted_replay_mutation.py](accepted_replay_mutation.py) 保留，调用其各自output新目录即可；生成的entry/play源码副本、build日志、实际native报告及进程记录同目录留存。300ms控制沿用原脚本 `injected100ms` 的诊断标签，实际副本明确 `Duration::from_millis(300)`，且生产报告确实是display-gap >250ms；以源码与实际pause原因判定，不以旧标签推断时长。

## 复用的原始证据及范围

本轮审计 work-r5 `clock-controls-verified` 六具名控制的原始报告及副本binary hash：五个100ms正控（motion/held-motion/未接受hide/restore/已接受V）到phase27，全部6阶段/每阶段6像素子检查通过；single/restore确有未接受V重投见证；accepted-V没有phase17重投。移除恢复的副本在原phase4截止拒绝。不是仅依赖脚本ALL-PASS。

同最终runner `07893426fd483e85583e717a7bc197b529212149f2d86163bc615ae08820e7a4` 的Patrol/搬迁原生正控及wrong-focus负控三报告、Step normal/delayed/dropped/duplicate四报告复核。Step正控phase27；丢失Step原phase6截止红；重复Step在phase6原exactly-one断言红。Step工具还检查真实按钮down/up数与accepted tick，恢复代码没有进入phase6；无需再运行其完整四例。

F-848-R1-01的segment/dirty/128/32credits/CPU、数值/版本身份、token、项目、owner和来源验证代码及输入未变；上轮 [review-r3-final/review.md](../review-r3-final/review.md) 的独立结论继续适用，未重跑60预算/全package/全native矩阵。此前当前baseline上已有producer/consumer216等门的复用须由merge核代码/依赖/配置/输入边界；本报告不自行将旧统一门套在变化后的native输入上。变化后的完整user/project/Play/Step/native/producer-consumer闭包仍应由merge完成。

source closure已刷新374；本轮独立 `python tools/verify_source.py` 核374source/681SDK/17local/29edges/479metadata通过。[source.log.txt](source.log.txt)。`cargo fmt --all -- --check` 与冻结repair `git diff --check`通过。work-r5 check原始日志的固定SDK146 warning边界保持；本轮没有运行整档或全包，没有新增产品debug输出。历史setup/指针遮挡/has_focus等待失败仍作为原始失败保留，不计通过；没有用无条件重复运行解决失败。

## AC、任务与Spec

| AC/任务 | 当前结果 |
|---|---|
| AC01/T01/08 | pass：四文件范围、374source与SDK/锁闭包再核；固定边界保持 |
| AC02..05/T02..05 | pass复用：产品几何/预算/身份/token/clock未变，F01保持CLOSED |
| AC06/T06/07 | pass受影响：本轮真实OS恢复保持目标/路由、Patrol与搬迁证据、文本隔离和wrong-focus负控不弱化；其余正式绑定/来源/项目输入未改，复用前轮 |
| AC07/T05/08 | pass复用：生产生命周期/owner/资源未变；当前Stop/新Run路径由本轮正控再次实际覆盖 |
| AC08/T07/08 | pass受影响：clock接受凭据、动作与GPU/trace/deadline oracle及变异敏感性成立；handoff准确；source已刷新 |
| AC09/T09 | pass：独立affected最终审查无开放合同内finding；M848-NATIVE-BACKLOG关闭 |
| AC10/T10 | pending merge-owned：完整门、实际合入、Spec/ledger沉淀/归档/guard清理尚未完成 |

产品规范提案未变：冻结副本 [spec-delta.md](spec-delta.md)，SHA256 `a07b8bff14af5359d9eef139da1d4204d3e0a9bca907a541b57005800529d89a`。SD01..04/8旧目标+1新目标与GOAL-007/008仍适用。本修复只调整验收刺激与接受确认，不改产品阈值/支持范围或规范语义，无须增加Plan revision。canonical输入10份hash与上轮一致，未发布canonical/live ledger。

[audit.py](audit.py)包含冻结/clean/四文件、依赖/Spec、raw子结果、独立兄弟及已接受V变异的可重放审计；详细身份在baseline。没有发现遗漏、未批准延后或掩盖产品缺陷的workaround，不新增债务候选。

建议父代理单写Plan恢复 `reviewed`、9/10，T01..09完成，T10保持pending，关闭M848-NATIVE-BACKLOG。用户已授权后续合并，但必须先由merge在当前冻结提交完成/合法复用统一门；任何再次实现/依赖/配置/输入变化需受影响复审。根t/tv/tt/tb/tu/taa/docs_gen不触发，tf仍主检出批量单实例到期判定。本审查不合入、不提交、不清树；实施最终HEAD仍dd7076a7f且clean。
