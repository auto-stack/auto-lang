# PLAN-804 revision 1 — final review R2 规范增量冻结

合同含义沿用R1，未修改plan_revision。本附件冻结本轮实际审查结论，不发布canonical Specs。源版本/hash见baseline.json.txt；详细原合同见../review-r1/spec-delta.md。

| ID | operation / target | before → after | AC / 当前核对 |
|---|---|---|---|
| SD-01 | add `docs/specs/auto-lang/ui/design/dialog-dnd-host-capabilities.md` | 新增三native的双独立owned provider、降级、同步线程、完成事件与父窗策略合同 | AC01/02/03；实现边界、值域、存活期owner、双取消已核；实际宿主事件链仍缺，不能写成已完成验收 |
| SD-02 | modify `docs/specs/auto-lang/vm/overview.md` | dialog/dnd直调OS/UI → 两独立provider与实例装配；删除本族直边，不宣称其余族全部搬出 | AC01/02；源码一致；保留当前主检出已落地的802 editor独立provider事实 |
| SD-03 | modify `docs/specs/auto-lang/ui/overview.md` | dialog/dnd未迁及历史专用STA叙述 → dialog归host_adapters、ui-dialog单门；dnd归UI、windows+native-dnd+ui门，输入线程内联同步，完成事件经宿主交付 | AC03/04；native_dnd源码头/start_drag文档本轮已纠正；R2新增App测试手工接线和输入清理回归不能作为完整实机证据 |
| SD-04 | modify `docs/specs/auto-lang/ui/design/shell-a2r-seams.md` | 两处rfd构建 → 唯一host_adapters实现+旧ui::dialog门面；VM parented/a2r unparented差异保留 | AC02/04；V04当前提交真实复跑通过，门面与单一实现相符 |

恒定规则：dialog缺席在弹参前保旧feature-disabled文本；在位取消/不可用为空串。dnd缺席弹参后false，坏/空/重入拒绝false；会话结束（含取消）true，效果独立交付。JSON只由UI原parser解释，filter只由单一OS实现解释。双槽的显式默认优先、构造快照、实例覆盖、clone/merge、binding身份规则不变；调用provider前释放槽锁。Send+Sync不授权COM跨线程；HWND进程缓存和拖入实现保持原owner。

元数据维持：supersedes_spec_components为上表三个modify目标；new_spec_components为SD01；touched_goals=GOAL-003/GOAL-007。新设计目标当前未创建，待merge沉淀。

并行合流：主检出802已合入，804树仍在旧基面；合入时须取provider/安装入口并集、保留799/802等规范事实，重核实际闭包/777基线。792受管源码包括本轮新增模块，不能再用签名相同判定输入未变。R2独立交付收据须绑定12e893628及实际根锁/source digest。
