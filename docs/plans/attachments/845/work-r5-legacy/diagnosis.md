# F-845-M1-05 / 自动修复第5轮（上限6）

944c1c55完整merge旧window-probe在phase1全局1400frame超时。baseline真实node1 down/up与selection1成立；诊断field0 y1154，scroll返回native fixture pointer occluded by another process，原5处let _丢弃错误。旧PowerShell posted click不验证物理位置，也不建立前景焦点；共同scroll正确拒绝遮挡。根因是旧原生测试入口的输入前提和错误传播不一致，不是Host/VM或missing-widget语义。

不变量：同一旧native入口所有click/scroll/key须走可验证OS通路；准备时等待本窗焦点，失败应携phase即时上报，原全局1400预算/selection/字段/target/GPU/Save/Undo/Redo/resize/负控不变。五个scroll分支统一传播Result，click统一共同own-PID/物理可达性helper，实际键盘End+VK_PACKET/WM_CHAR走winit。仅--window-probe输入夹具；正常App、共享helper、生产Host/VM、SDK/API/锁不改。

永久入口tests/authoring_probe_exit.rs::legacy_scroll_errors_fail_before_global_timeout实际启动原生exe，经node1选择与field裁切真实分支触发scroll前提故障，断言phase1原消息、accepted0、passedfalse、frames<1400。与已有原生CLI检查同文件互斥，避免两个同标题窗口竞争。受控恢复五处let _后应在同1400预算超时，使永久断言红；恢复后绿。原完整合法window-probe作接受控制，错目标/frames1作相邻控制；完成性4例防止错误转成功。

调试保留：第一次统一按键只有WM_KEYDOWN/UP无文字，positive phase3超时；按既有asset探针加入VK_PACKET/WM_CHAR。一次扩大日志条件的文本替换误把legacy初始化改empty场景，positive2 phase0超时；diff发现并精确恢复原Document::sample，此无效设置不计产品新finding或绿证。positive3真实完整phase9/280帧/accepted3成功；wrong保持selection oracle负控、partial保持incomplete。早期scoped通过仍在最终冻结重跑。

边界：本轮不重跑完整driver/AGP/全部user/project；它们在独立审查通过后由merge继续83项。M1-04 Step/common input、其他AC未变按确切源/依赖/输入证据复用。Spec proposal不变。不得把单个正例视为完整合入。
