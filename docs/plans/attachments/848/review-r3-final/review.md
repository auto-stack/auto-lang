# PLAN-848 r3 — independent final review: pass

stage: review | review_scope: final | plan_revision: 3 | outcome: pass | 2026-10-25

独立审查身份：用户授权的只读子代理 `/root/review_848_r3`。本上下文从主计划、仓库规则、冻结代码、R1 finding 和原始证据重建判断，没有参与方案探索、产品实现或实施会话内部复核；没有用内部 pass 代替本结论。可见身份仅限 agent task path，未虚构模型或 session ID。未创建代理/工作树，没有修改实现、永久测试、共享 Plan、canonical Specs 或 live ledger；未合入或提交。

审查提交 `6bc5abc6080b2261a43f55a54f127cd779239edb`；repair base `cef25b5c41dbbfbec6b7bfd4b13507d8d5f1e34d`；Plan base `bf22db3465203f3babd649494f4b37aa2a012027`。唯一实施树 `D:/autostack/.wt/lang-848/auto-lang`，分支 `plan-848-dev`。实际 worktree inventory、HEAD、分支、clean 在开始及结束核验。SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`，AGP/三锁保持。完整基线、Spec 输入及锁 hash 在 [baseline.json.txt](baseline.json.txt)。主检出别的会话 WIP 保留。

## F-848-R1-01：关闭，完整不变量修复成立

原 finding 属于既有 128 节点/两次 group 写/2cast+10getter+6AABB=32credits 的 CPU 支持缺口。当前没有减少节点/收费动作、提高100000steps/120resumes/250ms、换SDK或用Rust求交。

本轮自行阅读了 shared `derive` 提取和 setters 的变更、实际生成/命名空间/准入路径、2D/3D slab 与延后物化。修复不再只依赖首轴 nearest 剪枝：v3 在同一 world/effective 数组原位更新 dirty X/Y/Z/visibility；每轴严格保留 local+parent 的一次 f32 加法，父缓存局限于该轴当前 derive，parent 改变即重读。初始化/数组形状恢复全量；没有标记却显式 derive 时全量回退。setter 标志与原 `_ag_dirty`、最后 callback 发布路径相接；move 保守标三轴，visibility 独立传播，没有第二图或跨 callback 的父缓存。

受信紧凑布局只改变 v3 managed 数值代码的 SOURCE_LINE 开销，仍走原 Parser/Codegen/VM 和累计指令收费。literal 防线失败关闭；shared 原公式与 setter 边界都须唯一匹配。原用户字节、v1/v2 分支和身份保持。冗余换行变异确实重新耗尽100000steps，恢复正控通过，证明实际预算仍收费。

扫描在匹配候选任何 nearest 剪枝前先检全部所需维度的有限 world box；filtered 节点不参加几何验证。显式 float bounds/quotients 保住错误9/12分类。ascending ordinal + 严格更小 t 替换保 tie；胜者才物化 point，按相同商恢复 XYZ 首进入面；outside 正 t 下溢0和 inside/boundary normal0区分保留。每次 cast 尝试先清旧 token，callback 两端清 token，错误 sticky，序列上限不回绕。

独立运行的关闭 oracle：

- 原 R1 exact Session 三分布、正式 Host 三分布均通过，Host 是完整初始化后 Paused/tick1，而不是只断言未 panic。
- 永久预算检查1个测试内60个具名子例，2D/3D、宽/深、等深/晚轴miss/ordinal逐个变近/正负与非等比例斜向、各有效单轴双写，全部32credits，独立 hit/node/t/point.x/normal.x 断言通过。
- 真实 Cylinder 正式 Host 六分布通过独立几何结果及最终可见性/tick1断言。
- 新选独立反例 [review848_independent.rs.txt](review848_independent.rs.txt)：128宽节点，A=(10,-20,30)，B=(810,-620,430)，正负混合斜向，ordinal逐个更近，两查询分别写group.X与group.Z，读取Y面法线/point.y。预期 node127、t≈0.45791667、point.y=-294.75、normal.y=+1，32credits及callback后token0均通过。此例未复制既有原点/同轴/normal.x oracle。
- dirty混合轴/visibility/move、缺标记变异、parent切换/错误父缓存变异、最终非派生 `_ag_field` 发布、f32顺序及用户字节、有限几何先验/过滤/溢出、underflow、每轴正负法线、token/收费/非finite producer 各流转均有具名定向重放。

原始记录：[budget60.log.txt](budget60.log.txt)、[targeted.log.txt](targeted.log.txt)、[independent.log.txt](independent.log.txt)。本轮共18个单独选择测试通过、0ignored；其中60个预算子例与6个Host子例不另充测试数。独立反例观察98次yield，永久矩阵最高97；它们仅是调度切片观察，不是精确指令数或普遍余量保证，也不扩大到任意用户额外计算的预算承诺。

## AC → task → evidence

| AC | 结果 | 核查与证据 |
|---|---|---|
| AC-01 → T01/08 | pass | Plan-base全部改动限定driver；repair13文件同限。独立 verify_source通过373 source/asset、SDK681、17local/29edges/479metadata；三锁/spec输入对比readiness与旧review基线无漂移。source4反控及恢复正控原始记录复核。 |
| AC-02 → T02/04 | pass | 两维实际Auto扫描/finite-before-prune/ordinal/closed/XYZ及原式检查；具名几何/API重放及新增混合方向非零起点独立oracle通过。剩余closed/parallel/endpoint/ULP等未变测试断言及work-r4原始结果保留。 |
| AC-03 → T03 | pass | SourceSnapshot显式v3/profile9、准入/reserved与managed生成路径；旧身份/旧helper定向重放通过。v1/v2 API/ENTRY/bridge选择未进入新specialization，CLI/Binding/CodePayload调用点与repair-base无变更。 |
| AC-04 → T04/05 | pass | token每attempt/begin/end、单调上限、快照/新查询、dirty后最新world、非派生最终字段与错误父缓存变异有区分断言；token/收费测试本轮通过，work-r4其余快照原始结果复用。 |
| AC-05 → T02/05 | pass | F01关闭，原Session/Host反例及60/6控制；本轮具名token/exact-credit/nonfinite参数局部返回容器测试通过。全配置门、last-good/Fault/Starting/Stop机制未变，work-r4 Host4及R1安全面证据适用。 |
| AC-06 → T03/06 | pass | 两用户Auto项目、UI/CLI/API/Binding及来源历史机制源码检查，repair未改其调用/输入。r3同binary四正控/两短源变更/重开原始报告逐项审计；版本事务与正式SaveAs/搬迁旧证据在未变路径内复用。 |
| AC-07 → T05/08 | pass | 同数组原位更新/局部cache，11结果+4dirty=15标量60B；dirty检查含heap不增。work-r4 storage原值与额外flags分列、64query heap及consumer原始断言复核；VM/Descriptor/source/GPU owner、扫描/上传代码未变，R1 20轮及5变异控制按下述边界复用。 |
| AC-08 → T07/08 | pass | r3四native报告/两reopen、16traces的API/source及每part像素子断言核对；旧14native含8拒绝结果和实际阶段核对；SD01..04/API/数值/credits/handoff及完整门分母一致。native运行是原始证据复用，本轮未重跑OS/GPU。 |
| AC-09 → T09 | pass | 当前独立最终复审，基线与真实assertions重建，遗漏/延后/workaround及健康检查完成，无开放合同内finding。 |
| AC-10 → T10 | pending | 完整merge-owned统一门及实际落地/知识沉淀/归档/guard清理尚未执行；本结论不宣称交付完成。 |

## Spec 输入与规范提案

读取当前canonical相关规则与Plan SD01..04。8个既有目标、1个新增 segment-query-profile 路径与前后规则准确，GOAL-007/008存在且适用；显式版本、closed AABB/f32 tie、token、收费/CPU及I2/I3边界无冲突。SD03准确描述四dirty标量、同图按轴推导、局部父缓存/原顺序和受信源码紧凑布局。SD01/03的128支持现有有效关闭证据，没有用提案弱化验收。

冻结提案 [spec-delta.md](spec-delta.md)，SHA256 `a07b8bff14af5359d9eef139da1d4204d3e0a9bca907a541b57005800529d89a`；10份Spec输入hash逐项匹配work-r4及R1（见baseline）。此文件是审查副本，未发布canonical/live ledger。merge须以这些输入版本再次核对并沉淀，不盲写覆盖并发知识变更。

## 原始证据复用边界

本轮重新执行受影响的几何/derive/identity/token/预算关键路径；未运行全package或整档。r3 repair-base diff仅behavior managed组合、segment数学及tests/docs/source receipt相关13文件；Session预算实现、SDK/依赖、spatial Descriptor/Host/Play/Binding/CLI/项目输入、资源/GPU owner与native oracle代码未变。

- work-r4 `repair-affected-final.log.txt` 与 `consumer-affected.log.txt` 的33/12去重测试及断言/选择数已核，consumer自有SDK/target/source373刷新记录明确复用依赖编译缓存，未冒称重新空缓存构建。consumer最终source receipt与本次冻结一致。
- r3 runner hash `7ba32aa93a71433e1a1774b58fc704b4d9e5417ed353cc94c7cdfa826cce972a` 实际再hash匹配；四个具名报告phase14/exit0、每组3个pixel记录的全部part checks、4个segment traces的版本/用户源hash逐项核，另两new-process reopened结果通过。本轮只读raw，没有再次控制OS或GPU。
- work-r1 native14=6正/8反，透明/漏part/错资产在phase5 pixel-final-content失败；缺源/坏API/缺GLB等原入口未改。R1 owner5反控exit101且oracle命中、恢复正控exit0保留；20轮VM/Descriptor/source/GPU owner、scan/upload输入及机制未变，故不重复完整owner/native矩阵。新math/derive改变后的运行正控以r3证据补齐，不把旧binary冒充当前binary。
- work-r4 source四反控/恢复正控原始日志保留；本轮独立verify_source确认实际373/681/17/29/479。没有为重播反控而编辑冻结实现。

[audit.py](audit.py)保存上述raw/基线/选择数断言，可重复运行；它不是运行时native测试替身。已有测试的ALL-PASS banner不单独充当验收，检查实际测试断言与raw子记录。

## 命令、健康与未完成门

所有Cargo运行目录均为实施树 `experimental/autographics-auto-driver`。18个测试命令的target/filter在targeted日志逐条记录；budget与independent分别用 `cargo test --locked --test segment_budget original_counterexamples_and_diagonal_controls_128_exact_credits -- --exact --nocapture --test-threads=1` 和 `--test review848_independent independent_nonzero_origin_mixed_direction_switch_dirty_axis_128`。临时probe内容已归档，运行后删除；重放时将txt复制至tests/review848_independent.rs，执行对应命令后finally删除即可。

`python tools/verify_source.py`、`cargo fmt --all -- --check`、repair diff `git diff --check`均通过。固定SDK146 warnings保留，当前driver无新增warning或产品debug输出。遗漏/未批准延后/workaround扫描未发现新的合同内缺口，不需要把阻塞降为KNOWN-DEBT。独立probe、永久tests的eprintln只属验证诊断。

**合入门仍pending**：T10在该冻结提交执行完整locked driver（预期216=178+38，须按实际选择核数）/AGP test+build、原图形/AGP16/双游戏/spatial/Step/project/native-owner-source矩阵、producer与fresh own-SDK/own-target consumer、新segment正反/搬迁矩阵。848未触根源码，root t/tv/tt/tb/tu/taa/docs_gen不触发；tf仍主检出批量单实例到期判定。不能以本次18定向测试替代这些门。

建议父代理单写Plan：F-848-R1-01 CLOSED；T01..08保持完成，T09关闭，status=reviewed/current_step=9/10；T10保持pending，遵守用户“先不合入”。任何语义代码或依赖/配置/输入漂移需受影响复审，不能将此pass覆盖未来修改。实施树最终HEAD/clean与本报告基线一致，主检出只新增此审查附件，无产品修复/合入/归档/清树。
