# PLAN-830 r1：规划预检与来源边界

2026-10-21；仅制定后继计划与只读调查，未运行代码、原游戏、GPU/IME测试，未创建分支/树、agent或外仓改动。主检出master，调查HEAD `9b65383e0e73e94b90a065881a1333b6b54960a3`。

## 来源内容指纹

这是读取时的内容，不等于这些来源已经独立审查/clean/发布。829候选树可能继续验收修复；work必须重新核对G829与实际来源。

| source | SHA256 |
|---|---|
| `docs/plans/829-autographics-independent-research-and-game-prototypes.md` | `16aebb672eff11b7364c444693d11f2a3604bad68af8bb30a2faa3bdcccce01d` |
| `docs/plans/814-language-sdk-ui-package-separation.md` | `0e6beef93289940471ef83c3c127dfcb8b419905b1d84e2fde86f93fa29a76a6` |
| `docs/specs/overview.md` | `c23beeb31d1b44f3aa9ace38b9aae10482bf2595152d1c6f78a2e767f0de90a4` |
| `docs/specs/auto-lang/ui/overview.md` | `dcd54eb7140195c24857e38ac297ee9058c0045d5b7363db8c10f39832949c48` |
| `docs/specs/repository-boundaries/project.md` | `5d83f8e5767e9784b0aefaef3b5b3c2c40ec28a3e0c47459f8164ee5e652f764` |
| `829候选树 AGP/research/result.md` | `aee0d8bc6fb01ff8fa4b897af901db1f1a55dbbab6c3b7df0912e5c3b356c15a` |
| `829候选树 AGP/research/integration-handoff.md` | `748d59a02e81e55c7bdd5b398a36ff31c2a75e14e83a06a67de1aefa8e55f725` |
| `829候选树 AGP/src/core/scene.rs` | `c3bb323075593bcd1ecf6b730ab4f8196df232abc5af1c3643ae538b5c7e22a9` |
| `829候选树 AGP/src/core/runtime.rs` | `06b1bf3c078a3cb9d4b007caaa3ba1f6163c4fd7291e3b12bc105c5289191c10` |
| `829候选树 AGP/src/renderer/target.rs` | `c8dfa090b1df879bb9b5c06ca4a42b42aba2daf097975c270cac01825d8f0fd6` |
| `829候选树 AGP/src/host/iced_spike.rs` | `77e9790486e3e29aeefca2ddf4cac5d4f3dd21192a4fc99101283b2d0e1993ae` |
| `docs/design/45-autographics-wgpu-engine-and-game-prototypes.md` | `2b19121beb14448b306c9d1e14908409657e65d311f701c6356f7efd2643e93d` |

## 事实与选择

- 829当前execution_done r1，9/11。作者work交接与result申报原型/73测试/10probe，不是本轮独立重验；未看到本轮可引用的最终review与merge/Spec交付。
- 829交接明确未做任意UI/3D嵌套、射线/UV、焦点/IME；现有scene是平面对象/变更模型，普通离屏target没有采样usage，iced spike仅顶层组合。下一步选双向组合/输入，比继续堆新游戏或抢接生产迁移更直接验证架构。
- 当前814 executing r3（9/15），815/816亦在途；不能把局部SDK发布当迁移完成。`docs/specs/auto-graphics/`仍不存在。
- `git -C`在沙箱内读取829树报Permission denied；未因此声称其clean/当前SHA已核准。可读文件、主检出worktree清单和作者记录用于设计调查；有效源交接由T-01负责。
- 829 review-r1只有正在形成的行为探针可读，未引用其未发布结果作为正式finding；本轮不代判/修复其内容。其验收由原owner继续。

## 门与合同预检

- G829优先D829正式交付；独立实验提前消费仅在829原owner按获准阶段合同发布有效E829后成立。当前两者未具备，830不自行授权阶段发布、不把reviewed/WIP当delivered。
- G829后830独立实验不等全迁移；生产合入与Auto/UI/RQHost切换仍等829/迁移实际收据。没有“等迁移才能研究”和“研究完就算生产集成”的循环混淆。
- T-02先做真实Iced离屏→3D→输入whole-path可行性。失败回new，不改成截图、无输入假控件、第二Device或未获准fork。
- 组合model新增事务不改变829旧Scene逐项提交语义；layout挂载/渲染依赖、共享内容/独立实例、焦点路径分别定义。循环、身份/变换、alpha/clip、捕获取消、IME、单次tick与局部更新均有可区分正反例。
- 所有必需实现由work负责，review/merge只验收/收口；SD含尚待829发布的provisional目标，缺current不抢写。I1–I3/生产接线保持pending。
- 取号前活跃/归档max829，计数830；排他锁内调用现有new-plan脚本成功创建唯一830，计数831，锁已释放。本轮没有改829计划正文或实现。

结果：计划合同预检通过；`stage: new`、`outcome: blocked`仅指G829实施来源前置尚缺；`next: 原owner完成829验收/有效冻结交接后work`。Plan保持drafting、0/10任务，无执行完成宣称。

实跑文档检查通过：sections 0–10齐全，10个唯一任务、10项验收、4项SD；drafting与0/10一致；830唯一、中央计数831；五份入口文档的37个本地链接/锚点有效；本次触面格式检查通过。没有Cargo/docs_gen或游戏/GPU/IME功能通过结论。
