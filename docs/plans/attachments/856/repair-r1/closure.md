# PLAN-856 R1 修复闭合记录

基线3c39a96d21346e06b5c0ebf93b6e80ec4e5bb7a2、SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467，r1合同不变。真实复审反例文件为 ../review-r1/counterexamples.rs；运行同一输入的输出保存在 D:/autostack/plan-artifacts/plan-856/repair-r1/。producer、fresh own-SDK/独立target consumer 均使用固定锁；没有整档运行。

| Finding / 受保护不变量 | 路径与状态 | 修复及永久验证 | 当前结果与边界 |
|---|---|---|---|
| F-856-R1-01，PixelDelta以逻辑像素计量，每40px一档 | winit真实PixelDelta是PhysicalPosition；bin helper原样转发到Host；仅viewport的垂直物理位移按CompositionInput的top/window比例换算，Lines和面板路径不改；拒绝无效x/y，有限极大值在f64/f32边界安全饱和至±8档 | editor_viewport2d::pixel_scroll_uses_logical_child_units_and_preserves_anchor：DPI1/1.5/2，正负、极大值、anchor、非有限x/y、Lines和panel；bin::viewport_scroll_handoff_tests::winit_physical_pixel_handoff_reaches_logical_view_once：实际winit类型经生产helper进入Host，含NaN/f64::MAX | 冻结基线原输入DPI1/40px=1.1 control、DPI2/80px=1.21 wrong；修复原输入两者均1.1。新的永久producer和consumer定向用例、bin转发与原窗口正例记录见raw；真实触控板设备注入未单独声称。 |
| F-856-R1-02，取消连续导航必须恢复其已准入起始view及图像 | MMB按下时保存当前2D命令和view；直接cancel、blur、resize、关闭和准备失败的move均回到同一恢复；恢复不调用可能失败的资源准备，保留原错误/故障，释放capture，旧move/up不能再发布 | editor_viewport2d::cancellation_restores_admitted_frame_under_preparation_failures：两类seam×直接/blur/resize/failed-move及无故障控制，核对真实像素、状态、历史、迟到输入、mesh/target计数；rejected_wheel_preserves_last_frame_and_close_cancels_capture覆盖原子wheel失败保留最后画面与关闭捕获 | 原review 5种取消输入及合法控制重放：修复前4反例保留移动图，修复后全部回到原哈希7ca9abd3；两类seam、生命周期兄弟、failed-move、close与previous cases定向结果在raw。snapshot仅为有界既有2D DrawCmd CPU数据，不新增GPU mesh/VM owner。 |

独立final review与merge-owned统一producer/consumer/native/owner/content门仍pending。未重跑与两finding无关的GC/大规模/性能全档；重用原工作和R1中不受影响的证据时，保留原真实commit及相同SDK/锁/测试配置/输入。当前修复不改变原SD-01..04的合同，SD-01/03应由现在的实际输入换算和无条件快照取消支持；没有发布canonical Spec/ledger。

审查产物完整性说明：本轮首次直接调用原review-r1/replay.ps1时，该脚本固定写回review-r1目录，覆盖了当时探针exe和PDB。audit记录的exe SHA256=8042f368a436c56185e2aebefa583eea3a69ce99a0d5926a181c8a8bcd945037、PDB=75d1712ba64219737446b35c53720a8d327f00ebbb6690a191a8887f847893ad不再与当前文件匹配；当前分别为ff97226f158e3d927ec1ca2ee24084c114453f20386f1d1afcbdbb7dee90c4f7、726067d42baeef3b214749590d0a42e08ec84d24cf9d5861b62148adb0d47517。原audit保留未改，原counterexamples.rs、counterexamples.jsonl、主检出counterexamples.json.txt及其他review原始文件哈希未变。该两二进制不能再作为原审查哈希证据；本轮基线重放结果与原持久化输入/输出一致。之后复制同一输入/脚本到repair-r1单独重放，未再覆盖原目录。

绑定：最终修复提交 `4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`；producer 两项新永久回归、bin接线、两项旧控制、零窗口、wheel失败/close及 cargo check 通过；独立consumer两个定向用例通过；真实窗口20阶段正常通过（提交前相同源码/runner，source identity与提交blob一致）。完整证据路径、SHA和runner在 handoff.json.txt。实施工作所有finding/缺口/未决合同为空；独立affected review仍需确认，merge统一门pending。
