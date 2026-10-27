# PLAN-828 T07-R3-01 窄修复续轮

同树`D:/autostack/.wt/lang-828/auto-lang`，`plan-828-dev@b846f45dc10abb580136ff97f0562db7c1f62e41`，用`/auto-plan:work`。不reset、不撤required图/帧映射/严格目录/候选机制。Plan r3/executing0/11不变。

**旧例已通过但同一不变量仍OPEN**。请先读[报告](review.md)与`rebuilt-closure-controls.rs`的有效前提；不要再只轮换固定format版本。

根因对账：format1→2隔离“旧空边配方”，但format2内部仍可能有同body＋不同required目标：真实primary缺块→合法位置变化源码→cold重建primary ID变化。新鲜reference store证明manifest body逐字节相等；实际primary重建成功，manifest仍用旧bodyfp键，store正确Conflict。不是repair key存在性问题，不要求store改授信/Conflict规则。

在interface-decision将错误的“同body不同边永不共键/剩余边界无”改成有证据的当前设计。明确**实际body＋实际required闭包**如何决定持久身份：可把规范化目标ID/RefKind集合的稳定指纹纳入key，或给出等效可证明设计。规范化应同时用于key和实际发布的edges，避免排序/重复处理不一致再次同键异身份；相同集合重开幂等，目标变化不占旧key。旧行只读/隔离、新根注册成功后才退旧根；不吞Conflict、不覆写、不停persist，不要求用户清缓存。

永久收编最终`rebuilt-closure-controls.rs`两项（原样前提）：

- 原源码缺primary，正式recover后cold自愈成功，再warm admitted3。
- 仅插入5空格，fresh reference manifest body与原body逐字节相等，缺primary重建ID确实不同；restore必须成功，继而新required根→真实GC→再恢复，值与trap来源同冷编。fixture只在自建临时root、canonicalize验证后删除单个blob，不能删除任意用户缓存。

补“引用集合顺序变化不换身份/实际target变化换身份”的区分正反控（走实际publisher，非只测hash helper）。保持R3五旧迁移控制及永久mig6、T07 required根/用户行/退役控制绿。先完成全部修复，只做定向：mig、t07、b1/b2/session(明确skip两512)/contract/t01r/r1、cache_tier/实际commit组件、受影响sidecar/关闭。AV源码与依赖/config未改可按报告边界复用帧映射证据，不再重做前端。

一次冻结同树交审，给完整SHA/clean、实际输入/库身份/exit、旧例关闭与不变量关闭两栏；不把一组原例绿写成全闭。新固定版本号本身不能解决同版本不同refs这个反例。T08正式测量/IPC、双线、512、T09与merge-owned门保持pending，不整档/不docs_gen/不merge。
