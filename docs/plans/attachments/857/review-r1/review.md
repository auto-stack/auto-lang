# PLAN-857 独立最终复审 R1

- `stage=review`；`review_scope=final`；复审上下文 `/root/review_857`，独立于实施上下文；`plan_revision=1`。
- **结论：needs_fix。** 代码 `ecf9939c5b2e369f9ce7d6ea25b0b66439bb7da6`，基面 `4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`；唯一工作树 `D:/autostack/.wt/lang-857/auto-lang`、`plan-857-dev`，复审末 HEAD 一致且源码工作区 clean。SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`；当前来源清单、681 SDK、479包/17 local/29 edges及两锁通过实际核验。规范输入和delta冻结哈希、配置、原始日志路径及SHA见 [baseline.json.txt](baseline.json.txt) 和 [规范增量快照](spec-delta.md)。
- 独立运行计划指定3D视口20、数学2、真实GPU深度1，23项全通过；这些绿项不能抵扣以下独立反例。各 Rust 反例是冻结源码上的临时 integration test，测试后从工作树移除；`.rs.txt` 是可复用的完整输入，复制为 driver 的 `tests/review857_probe.rs`，设置 `RUST_MIN_STACK=16777216`、`RUSTFLAGS=-C link-arg=/STACK:33554432`、`RUSTC_WRAPPER=''`，用 `cargo test --locked --test review857_probe -- --nocapture --test-threads=1` 重放后删除。原始日志保存在 `D:/autostack/plan-artifacts/plan-857/review-r1/`，哈希见 baseline。

## 必须修复的发现

### F-857-R1-01 [P2] 支持范围内的透视宽视野误拒可见大对象拾取

- **违反**：AC-04、V-03、§5.2/5.3。支持最大distance 4096的有限透视视图，以及实际可辨识对象的拾取/手柄；未承诺的是极远处任意微小对象。
- **重放与结果**：[visible-pick-replay.rs.txt](visible-pick-replay.rs.txt)。实际 Host、相同目标/对象/ yaw 1.2、pitch 0，模型scale 500；距离1024正控：视图有效、原模型相对移除模型改变25/25像素，选择Some(3)。距离4096：视图有效、模型同样改变25/25实际GPU像素，投影半高21.14像素，点中心后选择None；矩阵逆有限但被拒。原始日志 `visible-pick-replay.log.txt`。故障不是距离下不可分辨的微小对象。额外边界扫描 `extreme.log.txt` 显示其他合法大target/角度/视图也会触发，未主张扫描穷尽。
- **原因与范围**：`src/authoring/interaction.rs:139` 的 `VP*inverse` 每分量绝对残差阈值0.01在该视图为0.010256588；`pick3_with:92` 和 `AxisDrag::new:188` 共用此入口，因此有效对象拾取和轴手势都可能被整体拒绝。不能仅把门槛调大而不验证反算精度。
- **关闭条件**：保留该像素/实际Host正反例，修复合法范围内数值稳定性并对大坐标、正交/透视、真实奇异/失稳矩阵及同轴拖动选择代表控制；保证正常近距离和真的退化拒绝均不退化。仅重跑相关交互和新窗口/consumer受影响验证；完整门仍属merge。

### F-857-R1-02 [P2] 子对象全部裁剪时仍产生可操作的3D组手柄

- **违反**：AC-04、V-03、§5.3 近远外不得产生可点击伪影；此为原有路径中落入857明确承诺范围的缺口，并非仅因已有代码而免责。
- **重放与结果**：[clipped-group-controls.rs.txt](clipped-group-controls.rs.txt)。组原点可见，全部后代分别位于正交远平面外、正交近平面外、透视远平面外；三例 `projected=None`、`group_handle_reason=None`、真实Host点手柄 `gesture_active=true`。同组子对象在视域内的正交/透视正控均可启动。模型偏离裁剪面数百至数万世界单位，非边界舍入或小对象；日志 `clipped-group-controls.log.txt`。
- **原因与范围**：`host/viewport.rs:552-624` 聚合后代时，`:594` 只检查投影w而没有判定几何与NDC深度区间的交集；`:786-801` 依此生成真实可按手柄。须考虑跨近平面或远平面的几何，不能简单删除所有投影角均在区间外的合法相交物体。
- **关闭条件**：全部裁剪场景不显示/不启动手柄并显示原因；相交裁剪面的合法对象、真正可见组、层级与不同投影正控仍可操作；复核组框与手柄共用真实已呈现相机。

### F-857-R1-03 [P2] 非捕获鼠标按键释放会提前结束3D导航

- **违反**：AC-05、V-05及共同合同的单一捕获/连续导航。原捕获按键仍按住时，另一按键的释放不能提交或终止当前导航。
- **重放与结果**：[button-capture-controls.rs.txt](button-capture-controls.rs.txt)。无干扰正控的右键环绕在后续40像素继续变化0.3141594弧度，中键平移继续变化2.3999996世界单位；夹入另一按钮的按下/释放后，两者后续变化均为0。日志 `button-capture-controls.log.txt`。此为直接Host入口对照；窗口路由 `autographics-authoring.rs:236-255` 确实把中键release交给`navigation_up`、右键release交给`navigation3d_up`，其入口未区分拥有者，非仅测试助手方法猜测。
- **原因与范围**：`host/navigation.rs:173-176` 对任意现有3D导航转交up；`host/navigation3d.rs:216` 对任意3D导航结束捕获。两种交叉方向均受影响。
- **关闭条件**：按捕获来源区分release，其他按钮及迟到/重复事件不得结束或提交，原按键正常release、Esc/blur/resize取消仍正确；增加真实窗口输入对照及两方向Host永久回归。

### F-857-R1-04 [P2] 原默认视图像素与856冻结基线缺少实际对照

- **违反**：AC-02/06、V-01、T-03/T-06及共同合同的默认像素不变。矩阵相等不能推出本次已改动AGP渲染的旧像素相同。
- **证据**：`tests/editor_viewport3d.rs:7-11` 验默认矩阵；`src/bin/authoring_probe/viewport3d.rs:110-156` 对比同一新版本grid off/on；工作完成记录仅列以上和新/旧2D窗口门，未给出856冻结默认3D输出的比较或逐项不变证明。本次 `render3d/**`/`surface_runtime.rs` 已改，856的“AGP未变”证据不能直接复用。
- **关闭条件**：在相同输入、尺寸/配置/设备可比范围内，用保留的856冻结来源或有效旧证据对比857默认/重置的真实3D及最终合成内容，提供可区别错误默认图像的断言与可复现证据；不提前替代merge-owned全量原有门。

### F-857-R1-05 [P2] 缺少855同条件性能与工作集对照

- **违反**：AC-06、T-06和共同合同“以855最终基线报告同条件导航off/on、默认旧视图和辅助开启后的CPU/工作集/上传/owner”。
- **证据**：`tests/editor_viewport3d.rs:717-747` 和 `work-t03/frame-costs.log.txt` 只在857当前进程比较静态grid off/on 的50个warm frame CPU与部分buffer/上传快照；未比较855最终基线、导航on/off，也没有同条件进程工作集对照。现有单次数字不能推出预算或旧路径可比结果，也不能用不同窗口阶段的峰值拼接冒充同条件。
- **关闭条件**：补可追溯855基准及857默认/导航或辅助开启情形的同条件CPU、工作集、静态mesh上传和owner结果，明确不等价负载和测量限制；允许复用已核实不变证据，不得宣称GPU时间。

## 验收及规范影响

- AC-01：来源、SDK和锁实际核验通过；修改后须重建当前来源身份与消费端。
- AC-02：默认矩阵与投影常规路径已测；原默认像素对照因F-04未完成。
- AC-03：有限深度pass及正反例、资源配额和owner的现有证据已检查；当前没有独立新增失败，修复其他项不能破坏该面。早期zero-bias单GPU仍通过，不认领bias必要性或跨GPU结论。
- AC-04：失败（F-01/F-02）；AC-05：失败（F-03）；AC-06：部分（F-04/F-05）。AC-07不能通过；AC-08/09及双端完整、受影响AGP、原新native/owner/content门均为merge-owned pending。
- SD-01..04目标文件与`GOAL-008`存在，目标当前版本及hash见baseline；增量方向在修复并重新复核前仅为提案，未沉淀canonical Spec或ledger。
- 独立测试输出原样保存。SDK既有146项warnings，未声称其已清偿。未执行裸 `cargo t/tv/tt/tb/tu/taa`、完整producer/consumer或AGP合入门禁，未合并。
