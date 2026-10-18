# PLAN-804 r1 — reviewed delta snapshot（final R1 needs_fix）

本附件冻结受审合同及素材，不发布canonical Specs。目标当前内容SHA-256见baseline.json.txt；没有修改计划语义，不增加plan_revision。以下为合入时的规范增量，不能将待完成的验收记为已证明。

## SD-01 / add

目标：`docs/specs/auto-lang/ui/design/dialog-dnd-host-capabilities.md`（当前不存在）。对应AC-01/02/03，GOAL-003/007。

新增规则：2927/2928经DialogHostProvider，2938经独立DndHostProvider；接口无UI/OS类型，filter/default_name借用shim已物化的owned String，JSON由owned String传递，返回owned String/Bool。过滤与JSON分别只由唯一OS宿主实现和既有UI parser解释。Dialog缺席在弹参前返回原feature-disabled VMError；在位取消/不可用为空串。Dnd缺席弹参后false，坏/空payload或已有会话重入false；true只表示会话执行完毕，取消同样true，drop效果由完成事件独立交付。

双槽各自进程默认/if-absent补缺/构造快照/实例覆盖；显式默认胜兼容默认；NI clone保槽，VM merge保目标实例，注入不改binding_revision/proof。调用provider前释放槽锁；Send+Sync不授权COM跨线程。真实模态与DoDragDrop保持调用输入线程同步阻塞，完成事件应归发起App。

VM默认dialog策略保留最大可见本进程顶层窗EnumWindows+进程期HWND缓存，a2r原门面继续unparented。缓存改造、拖入IDropTarget和OLE重写不在本轮。

审查：代码结构与规则一致；R1-F01/F02的V04/V05必验缺口尚未闭合，不能在规范的证据节写已完成。

## SD-02 / modify

目标：`docs/specs/auto-lang/vm/overview.md`，对应AC-01/02。

before：已迁frame/text/media/scroll，dialog与dnd仍属VM直调UI/OS的剩余族。

after：增加两独立provider的装配、缺席与值域矩阵，2927/2928/2938只保编解码/委托；从剩余族删除这三个native，其他未迁族按当前Specs保留。借用参数由shim owned值承载，不能把owned边界误写为VM池借用跨界。

审查：与当前代码一致。合入时只做增量并保留并行802/799等已落地事实。

## SD-03 / modify

目标：`docs/specs/auto-lang/ui/overview.md`，对应AC-03/04。

before：dialog/dnd未迁，线程说明仍混有历史“专用STA线程”叙述。

after：dialog实现归`host_adapters::dialog`，仅ui-dialog门控，不依赖ui renderer；高层入口在ui-dialog-only仍装配，原始AutoVM只快照。Dnd adapter归UI，windows+native-dnd+ui三门控，复用原parser/DoDragDrop/完成通道/代际锚定，输入线程内联同步。剩余族清单减去本族，不能宣称整个UI运行时已搬仓。

审查：宿主归属/feature门/线程实现一致；R1-F04指出源码文件头及旧start_drag说明尚未纠正，规范增量不能掩盖该遗漏。真实完成App链证据见R1-F02待补。

## SD-04 / modify

目标：`docs/specs/auto-lang/ui/design/shell-a2r-seams.md`，对应AC-02/04。

before：`ui::dialog`自建rfd，与VM逻辑重复；a2r父窗v1从简。

after：`ui::dialog::open/save`保留原签名并薄委托唯一`host_adapters::dialog`实现；VM parented/a2r unparented差异继续存在，去重不等于修复父窗兼容缝。旧门面仍由ui+ui-dialog编译，OS实现独立ui门；不把ui-dialog-only写成a2r门面原本可用。

审查：源码委托清楚，R1-F01要求修复无父窗真实oracle。源文件与构建闭包已变化，R1-F03要求按792真实入口重放；签名相同不赋予旧二进制/旧交付源复用权。

## Metadata

- supersedes_spec_components：`docs/specs/auto-lang/vm/overview.md`，`docs/specs/auto-lang/ui/overview.md`，`docs/specs/auto-lang/ui/design/shell-a2r-seams.md`。
- new_spec_components：`docs/specs/auto-lang/ui/design/dialog-dnd-host-capabilities.md`。
- touched_goals：GOAL-003、GOAL-007。
- 合流保留：主检出generated-ui-distribution已含auto-ast等当前事实；804树基面仅10个delivery crate，不得以树内旧清单覆盖主检出11个crate事实。
