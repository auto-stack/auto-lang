# PLAN-856 r1: CPU统一View2d接法

基面：855实际交付05fdc81c546aff735eefdb6c39fbc5fb35dc546a；唯一实现树D:/autostack/.wt/lang-856/auto-lang，plan-856-dev。SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467，不改AGP/锁/Auto算法/API/scene4。

选择CPU统一接法。Quad相机统一变换会一并缩放逻辑像素手柄和轮廓，仍须分离辅助与世界坐标；现有DrawCmd足够，不增加公共渲染接点或GPU owner。比较限于这两种局部接法的源码/容量可行性；实际可执行全路径探针只实现CPU方案，没有声称测量另一方案。

输入经CompositionInput一次window→child换算；View2d::screen/world是唯一2D映射；Host::prepare_frame投影中心/尺寸，pick2_view逆映射精确ellipse/rect，group_geometry和组原点同映射；Gesture快照保存presented View2d和原world origin，总屏幕位移÷zoom，再world吸附减父平移回local，真实Building/Auto整帧准入、accepted提交和历史逻辑保持。frame_with_cursors成功后才更新presented_view2d；未呈现、旧stamp/camera/view的press/候选拒绝。导航和对象事务互斥。

Grid独立CPU批次先画，间距1/2/5×10^k，>=16逻辑px，最大76条（含轴）小于128合同上限，Vec固定capacity128×sizeof(GridLine)=2560bytes；不占节点。保守总DrawCmd上界：128×2场景/叶选择+128网格+4×241组选框+2×17原点十字=1382<4096。运行prepare末尾再次检查容量和所有GPU输入有限；超界拒绝view更新、保旧画面和文档。Grid不开时空批次。没有AGP改动，没有新mesh/target/pipeline/VM owner。

连续MMB保存起始view，Esc/blur/离窗/resize/关闭恢复；wheel是原子更新，后续blur不撤销。New/Open成功重置，失败保留；Save/Undo保持；Play临时原运行View2d，Stop恢复保留的编辑Prepared/View。正常对象release/noop仅释放pointer capture，保留该viewport键盘焦点；真正取消仍走清焦点路径，避免F/Home没有焦点的旧行为。

可执行证据：D:/autostack/plan-artifacts/plan-856/view-final.log（6/6，实际Auto拖动、parent/group world吸附、保存重开、Play/Stop、资源20循环）；scoped2.log中原Host7/interaction7/session4通过；native-sixth.json（真实OS20阶段，Grid像素、MMB、wheel、真实Auto/Undo、F/Esc、面板文本隔离/Home）。最初native-first的1px线采样误红已定位实际像素中心（线x20覆盖pixel19），native-second起修正，不作时序归因；旧失败保持。native-fourth揭示正常release清焦点，修复后fifth/sixth通过。

T-02接法/输入/资源决策冻结；后续T-05还需同oracle真实旧逆变换/漏网格mutation红与恢复、来源/独立consumer，T-06需最终定向/测量/提交。此文不是最终review或merge门收据。
