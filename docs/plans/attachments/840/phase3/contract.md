### r5 / Phase 3：同步测试载体的 Init 完成态（同一840，不另建计划）

r4冻结合入门：bare t 5340/5340、tt5712/5712通过；tb仅5192/5311执行，5190过、2红、119取消，故未合入。两红为plan498 bar/donut几何早读：真实输出含VM-PARKED Init/CpuRunnable。匹配既有D-762-1；其约定“改生产helper先修订确认”继续遵守，本phase不改生产helper或放宽断言，也不把旧债标签当作通过。

新phase仅改test-gated的plan370_test_support与相同Init早读机制的同步UI测试载体（437/484/492/498/499/536）：复用437/502完成态先例，统一为有界截止时间、真实泵与视图重新登记驱动；完成态后才取几何/交互断言。保持多片Init早退的原生产helper负控，增加确定性pending超时负控，原所有图形/交互assert保留。musk Fetch的已有外层8轮非Init专用载体先审计，不盲替换。生产dynamic.rs、CPU/VM调度、原输入/几何阈值/支持面只读。

新增AC17：失败bar/donut及共享Init载体不早读，真实多片Init完成后几何/交互原assert均成立；AC18：有界截止时间对未完成任务必须红，原helper早退负控仍红/生产语义未变；AC19：同族定向病例/格式与精确冻结、Spec增量和受影响复审通过；AC20：完成r5冻结门与原主检出完整批量收据后，方可关闭T-10/T-16并最终归档清理。

| task_id | owner | 关闭条件 |
|---|---|---|
| T-17 [x] | work | 按实际tb原始失败与D-762-1核清载体/完成态路径；生产helper不变；AC17/18 |
| T-18 [ ] | work | test-only共享完成态驱动；替换共享原因的Init早读调用；保留所有原assert与负控；AC17/18 |
| T-19 [ ] | work | 同族定向真实病例、deadline反控、格式/Spec/冻结提交；不跑whole-tier；AC19 |
| T-20 [ ] | review | 逐项受影响review及原支持面/未改证据核验；AC17..19通过后交merge |

V17：原plan498两失败与484/492/499/536共享完成态调用族；V18：437确定性跨多片Init早退→新驱动完成几何的正反对照，加pending零截止时间必须失败；V19：源码/断言身份与helper production只读检查；V20：review后merge-owned bare t/tt/tb及主检出tf/tt/tb完整选择面。SD-P3-01：docs/specs/auto-ui-runtime/project.md记录同步测试需驱动真实Init/CPU/IO完成并重建视图，不以单次helper调用等同完成；生产helperD-762-1仍保留为独立已知债，不冒称清偿。

当前executing17/22，Phase2三项修复与原支持面保留；当前复审只覆盖r4既定写面，r5新增载体变更须T-20复审后再跑受影响完整门。不新建846或其他计划。
