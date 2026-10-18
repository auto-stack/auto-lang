# PLAN-804 revision 1 — R3 规范增量冻结

本轮未改变语义合同；源版本/hash见baseline.json.txt。沿用R2的四项增量，未发布canonical Specs或ledger。

| ID | operation / target | before → after | AC / 本轮判断 |
|---|---|---|---|
| SD-01 | add `docs/specs/auto-lang/ui/design/dialog-dnd-host-capabilities.md` | 新增dialog/dnd双独立owned能力槽、降级/线程/完成/父窗合同 | AC01/02/03；三native边界实现不变，当前提交真实宿主验收与失败收尾未闭合，不写“全部实机证据齐” |
| SD-02 | modify `docs/specs/auto-lang/vm/overview.md` | 本族直调OS/UI → provider实例装配 | AC01/02；接口、值域、ID、优先关系不变，保留当前802等已落地事实 |
| SD-03 | modify `docs/specs/auto-lang/ui/overview.md` | dialog归host_adapters(ui-dialog单门)，dnd归UI(windows/native-dnd/ui门)，输入线程同步执行 | AC03/04；线程源码纠错保持；新真实宿主example/测试是验收设施，不把它描述为新生产事件接口 |
| SD-04 | modify `docs/specs/auto-lang/ui/design/shell-a2r-seams.md` | 重复rfd实现 → 单一实现+旧ui::dialog门面，VM parented/a2r unparented保留 | AC02/04；代码未变，R2实际owner与双取消证据保持历史有效 |

恒定规则：Dialog缺席弹参前原feature-disabled错误，在位取消空串；Dnd缺席弹参后false，坏/空/重入拒绝false；执行过会话（含取消）true，完成效果独立报告。调用前释放槽锁；clone/merge保留目标实例槽，注入不动binding身份；Send+Sync不授权COM跨线程。JSON与filter各由唯一宿主实现解释。

元数据不变：supersedes为三个modify目标；new为SD01；touched_goals=GOAL-003/GOAL-007。主检出最新802/799事实须增量保留，合流/777/交付闭包与新冻结提交的重判归后续相应阶段，不能以旧producer digest冒充当前整包身份。
