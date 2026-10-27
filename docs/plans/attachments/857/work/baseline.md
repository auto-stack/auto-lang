# PLAN-857 T-01 基线

- 856已归档10/10，实际代码 `autographics-dev@4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`；交付和完整双端/原生/所有权证据见 [856收据](../../856/merge-r1/receipt.md)。856 在落地后才补做工作树 guard 的流程遗漏已如实记录。
- 本Plan唯一树 `D:/autostack/.wt/lang-857/auto-lang`，分支 `plan-857-dev`，起点同上；固定SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`。`prepare_sdk.py --verify-only` 和 `verify_source.py` 实际通过，日志 `D:/autostack/plan-artifacts/plan-857/work-t01/`：462源码身份、681 SDK文件、479 packages、17 local、29 path edges。
- 原856完整 producer/consumer各300项，继承原288项；两端原native、owner、content及23个合格AGP槽按856收据。857允许的AGP深度接点改动会使相关复用须重新判定。
- 当前真实实现：`View2d` 与导航事务已存在；`interaction::pick3_with` 读取传入VP但使用绝对det阈值，`Gesture::document` 对3D轴使用屏幕投影近似；`Camera3d` 支持正交与透视投影，`MeshPipeline` 的场景色彩pass已有深度，`ScreenOverlay` 不能证明深度遮挡。上述只是决策输入，不是T-02实验通过。
- 允许driver编辑Host/interaction/View3d/bin/probe/tests/tools/research/README，AGP只限最小render3d和surface_runtime辅助接点、计量及测试；不改SDK/依赖/两锁、公共游戏/模型语义或原预期。T-02待完成两种有限depth接法比较和透视拖动端到端/退化正反对照后冻结。
