# PLAN-857 T-02 三维视图与深度绘制决策

基面856 `4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`；可复查输入逐文件SHA见 decision-inputs.json.txt。当前代码尚未提交，随后改动必须重新验证受影响项。

- 深度接法：真实GPU对照先以场景网格mesh strip与前景遮挡块证明可见与遮挡，再比较同画面置顶ScreenOverlay，后者覆盖遮挡块因而不满足合同。mesh复用会占原512实例/阴影caster配额（`pipeline.rs`阴影和颜色均遍历同一groups），因此选独立有界LineList辅助pass：颜色与场景共用depth，Load已有深度，LessEqual，depth_write=false，bias constant -1/slope -1，位于场景与shadow pass之后，不进入场景实例或shadow。实际GPU测试同时见共面线、前景遮挡、错误置顶、关闭无上传、固定mesh无重传、GPU辅助buffer≤64KiB；256顶点=128线、实际buffer7232字节。空grid无资源创建。试验中把depth bias归零后此特定场景仍绿，故不能声称已证明该偏移必要或其他驱动共面鲁棒；后续T-05需更强共面/深度负控。
- 真实绘制负控：临时将实际grid draw顶点数改为0，原同一GPU测试在`depth grid visible outside blocker`具名失败，恢复后重新绿；原始 `no-grid-mutation.log.txt` 与 `grid-restored.log.txt` 保留。该定向探针不是计划要求的最终OS→合成链负控。
- 视图：默认直接返回原 `Camera3d::sample_camera(2)` 全部数值；orbit Δyaw=-dx/800×2π，Δpitch=dy/400×π，pitch钳±85°；pan沿相机right/up，以目标面可见高度/400世界单位每逻辑像素；wheel每档1.1倍（±8档有限钳制），正交缩放高度、透视缩放距离。切换用高度 `2*d*tan(30°)`，边界饱和须提示。
- 反算：视线沿axis分量剔除后的向量作为拖动平面法线，方向长度<1e-3拒绝；NDC 0/1的有限射线与平面求交，平行/非有限/段外样本拒绝，候选沿冻结世界轴位移并在世界原点吸附。矩阵使用有限逆及 `VP*inverse` 各分量残差≤0.01，不再使用旧固定det<1e-10；合法大视野与零矩阵对照，以及独立数学几何对照通过。此阈值的更广数值条件仍须T-04覆盖。
- 实际Host→Auto透视X轴移动、一次history/Undo、导航取消及Play相机恢复通过；这是定向Host证据，不冒充真实OS事件。旧 authoring_interaction 七项通过；原SDK 146条既有warnings未扩到driver。日志 `D:/autostack/plan-artifacts/plan-857/work-t02/`。T-03..06仍需实现及检查跨尺寸/XYZ/多mesh/视图快照/焦点/资源和原生变异恢复，consumer与全量merge门均未执行。

2026-10-27 收口补记：此文件原T-02状态和decision-inputs为历史。当前代码、实测共面/遮挡、变异、限制与review/merge边界见 [工作完成记录](completion.md)，来源清单和冻结提交以该记录为准；zero-bias在早期单例通过的限制仍保留。
