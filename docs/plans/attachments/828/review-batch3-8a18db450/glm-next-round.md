# PLAN-828 第三批 needs_fix：同树一次修齐

基线 `plan-828-dev@8a18db4503a6b30e53a0affa9cb17c4126d35e6d`，树 `D:/autostack/.wt/lang-828/auto-lang`。先读本目录review.md和主Plan，按/auto-plan:work处理C01..05；不要直接进入T-10，不要重做已闭候选/单元派发、reset或清用户cache。

先补一张紧凑关闭表，沿真实数据流确认责任，再实施：

| ID | 必须解决的机制/出口 | 同一门的关闭证据 |
|---|---|---|
| B3-C01/P2 | A07未采纳时也不忘自己已有的root归属；分别处理可采纳性与发布/退役责任 | 原hir-roots-controls位置变化例root3→最多3、GC有界；不移动正控继续绿；代表性再次restart/内容回访、结果/哨兵正确；不放宽A07门 |
| B3-C02/P1 | 每个父侧publication绑定独立预期的当前revision与确切FunctionId；不只认install长度/worker自报changed_fn | stale revision、wrong install两个原控必红，合法多修订/命中及缺样本/拒绝路径有准确状态与诊断 |
| B3-C03/P1 | 在真实AV消费处观察未改payload identity/generation/content，并进入正式父侧结构门 | 原真实深复制且AV执行副本的控制必须使同一formal总gate红；原复用正控绿；代表性共享消费入口保持 |
| B3-C04/P2 | 测量终点在必要父侧校验完成后；正式观测含约定阶段wall/计数/IO/身份和完整成功失败raw，初始化另报 | 30样本与失败控制可留存复核，错误/中断不丢失败；重测P95及同798预算；raw不是仅建String后丢弃 |
| B3-C05/P2 | 修正后继API和Spec真实路径、计数与观测归属 | resident_weak参数正确；热路径miss/hit与实际consumer一致；b1数量据实际选择；SD01/05/06和覆盖清单一致 |

原始控制可直接复用，方法与期待值见review.md。若协议调整使旧测试无法编译，机械适配新字段/签名并记录差异，保留同输入/同故障/同oracle，不能把拒绝预期改成允许。复制控制须真让AV使用副本，不能换成未执行包装对象或伪造布尔字段。常量ready身份可以前置核，逐修订actual核验必须在窗内。

不要求通用observations平台、专用漂亮报告、额外全场景性能战役；只交付原合同必需字段与可留存证据。旧两个512和其他未改模块无需反复跑；先修完全部findings，跑相关反例/模块与必要check，最后选受影响定向集合。涉及存储/正式gate的最终代码上重放512完整存储轴和30轮正式战役各一次，并保存raw/输入/库/worker身份；不跑整档t/tv/tt/tb/tu/taa/tf或docs_gen。

所有代码/永久测试留同WT，主检出仅Plan簿记。提交包含全部实际修改的新冻结根，交付五行关闭表和实际选择/退出码。第三批复审通过后才T-10；整份最终审查与merge-owned门均未完成。不要启动另一chat或代发消息。
