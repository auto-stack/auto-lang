### r6 / Phase 3 扩展同类“生产者完成再断言”的测试设置修复

r5 bare t5342/5342通过；tt在RQHost末窗测试翻红（实际窗口全关闭后仍有待定，原断言保持），非Init修复回退。原测试持pending锁调用adopt，然后手工插入客户端endpoint；真实服务器peer仍由线程异步发布，可能晚于clear到达。生产退出分支正确拒绝未清空pending，不能改生产逻辑掩盖错误载体。

同一Phase3新增唯一测试触点 `crates/auto-ui-runtime/src/ui/desktop_protocol/rqhost.rs::tests::last_window_close_exits_daemon`：实际adopt一次，持有客户端而不发Hello，等真实服务端gamma入pending（复用现有5s采纳完成界），核验唯一1条，再驱动原双窗/有待定/末窗/boot负控。去掉错误手塞peer，不改RQHost协议、rq_update或生产helper。AC17/18扩展为“Init与真实采纳完成态设置后才断言”；V17追加此真实管道病例及并发采纳/VM fork/rendezvous相邻合法控制；SD-P3-01补充该端点/发布规则。原几何/退出assert与性能阈值不改，支持面保持，不新建计划。

r5复审对原Init写面仍有据，新RQHost修复按T-18/19/20重新执行/定向检查/冻结复审；完整门pending。前序失败/取消数原样保留，不记成通过。
