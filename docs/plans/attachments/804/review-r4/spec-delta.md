# PLAN-804 revision 1 — final R4 规范增量冻结

2026-10-18，受审1ca36a674e9199985f8bc804d1ac0f61d1bcaa97。合同无语义修订，plan_revision维持1；目标当前hash见baseline.json.txt。以下为已审查的merge输入，未写canonical Specs/ledger。

| ID | operation / target | before → after / enduring rule | AC |
|---|---|---|---|
| SD-01 | add `docs/specs/auto-lang/ui/design/dialog-dnd-host-capabilities.md` | 新增2927/2928 DialogHostProvider与2938独立DndHostProvider，owned值边界、缺席矩阵、同步线程、完成事件与父窗策略 | AC01/02/03 |
| SD-02 | modify `docs/specs/auto-lang/vm/overview.md` | 本族VM直调OS/UI → 只编解码/委托双provider，实例装配/快照/clone/merge/binding身份保持 | AC01/02 |
| SD-03 | modify `docs/specs/auto-lang/ui/overview.md` | dialog实现归host_adapters、ui-dialog单门（无需ui）；dnd归UI、windows/native-dnd/ui三门，输入线程内联阻塞与真实完成链保持，纠正历史专用STA叙述 | AC03/04 |
| SD-04 | modify `docs/specs/auto-lang/ui/design/shell-a2r-seams.md` | VM/ui::dialog两份rfd构建 → 唯一host_adapters实现+旧ui::dialog薄门面；VM parented/a2r unparented差异仍保留 | AC02/04 |

具体值域：filter/default_name是shim已物化owned String的临时&str；返回owned String路径（to_string_lossy形）。Dialog缺席在弹参前保原feature-disabled文本；在位取消/不可用为空串。Dnd接owned JSON，仅由既有UI parser解释text/files/virtual_files/base64/裸反斜杠修复；缺席弹参后false，坏/空/重入拒绝false；执行过会话（含取消）true，效果另由on_dnd_finished交付，不能把true当drop成功。

装配规则：双槽独立，显式默认恒胜兼容if-absent、构造默认快照、实例覆盖；clone保槽，VM merge保目标实例，注入不动binding_revision/proof。调用provider前释放槽锁，模态可重入；Send+Sync不授权COM跨线程。DoDragDrop输入线程内联，完成通道/代际锚定归发起方App；输入及会话资源收尾保持。

父窗：VM默认保留EnumWindows最大可见本进程顶层窗+OnceLock进程缓存，a2r旧门面unparented；不在本轮修缓存、拖入或OLE实现。dialog OS实现不受ui renderer门控制，高层ui-dialog-only继续安装；原始AutoVM只快照。

R4证据：真实生产run_dynamic_desktop→handler→DoDragDrop→生产订阅/完成臂→发起方handler落盘copy，非发起方零送达；正常与交付失败退出键态无残留，启动停滞/交付失败由ChildGuard自身kill+wait。example/反控钩子是验收设施，不新增生产API或异步拖出语义。

元数据维持：supersedes为三个modify目标；new为SD01；touched_goals=GOAL-003、GOAL-007。合流须保留当前802/799等已沉淀事实，剩余族清单按最终代码减去本族，不能宣称整个runtime已解耦。最终主检出并集后的777基线和交付包producer/digest需实态更新；历史792源digest不冒充当前整包身份。
