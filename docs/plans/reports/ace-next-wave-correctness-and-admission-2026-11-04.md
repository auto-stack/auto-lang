# ACE下一波：正确性、增量与可重放验收（2026-11-04）

> **初订历史快照，当前以[r2复核/执行窗口与M1距离](ace-931-933-plan-review-and-m1-distance-2026-11-04.md)为准。** 927已交付、928/929已实施；下文932“真签名必须ERROR”未核914已批准typed策略verdict，现由932 r2纠正。历史59/60及混合现象记录不抹除，合法body与故意签名根因仍须实施重建。

状态：用户授权规划下一步，本轮只调查/立项/更新路线，不施工/建树/构建测试。取材main `aa6791649`；工作区原有三个未跟踪项保留。下列运行结果引用既有交付/债务收据，不是本轮新实测。

## 1. 已交付与实际在途

| Plan | 当前事实 | 近期安排 |
|---|---|---|
| 923 | archived；D923=`ddd42ef2b`，头9/12为簿记残留；fault/Place资源门 | 已交付，不重开 |
| 924 | archived；D924=`48c2d8066`，头11/14为簿记残留；R02/R05 ac有限闭包 | 已交付，保原NA/域外K06等余项 |
| 925 | archived 14/14；D925=`a37245cd6`，C record/text/mode/方法/资源与来源 | 已交付，内部C表示不是公开AutoABI；留下D-925-1/2等具名债 |
| 926 | archived 14/14；D926=`3fce61b9a`，C collections/variant/owned资源及认证 | 已交付，跨target/恢复边界保其实际合同 |
| 927 | reviewed r2，10/12；唯一lang-927树`35ad072b2`，修复复审pass，尚无D927 | 由原owner完成合入/归档，不能把分支当交付 |
| 928 | 头drafting r2 0/12，但已有goal轮1授权/等待927合入回执，未见实施树 | 按原owner接D927推进，不由本会话启动/改进度 |
| 929 | drafting r2 0/14，未见实施树 | 仍是全入口/首编辑/重复parser退役必需后继，不能漏列 |

D923/924/925/926提交在本轮只读核对为main祖先；归档头步骤未齐不自动重开。927头reviewed与实际唯一树一致，928等待授权回执与头状态分开记录。M1当前不能签章，三类原机制余账SQ/INST/CT、CA-D、最终组合验收保持。

## 2. 为什么本波先补已知缺口

原本最自然的新能力是CA-D：925/926已交付，A2C/ABI桥的资源前置已具备。但源码与新收据同时暴露以下四项。按M1机制完整要求，不能一边把这些当长期存量红，一边宣称基础已成熟。本波选三个独立可交付包，不把互不相关的代码修复塞进ABI大包。

| 缺口 | 本轮源码/现行收据依据 | 新owner与关闭范围 |
|---|---|---|
| D-925-2：Rust多实参临时值过早清理 | `a2x-rust/src/lower.rs::{lower_pure,lower_move_value,lower_view_value}`在新pin前全量`drain_frees`；925 C侧已改scoped drain。现有C测试`a2c_rt_restore.rs`的Wrap.mk("ab","cde")应保head=ab/tail=cde、span=5；债务记录指出Rust同形head空串 | [931](../931-a2r-owned-temporary-consumer-lifetime.md)：真实消费者作用域、源码序、move/view/嵌套/失败清理与Rust缓存版本；不重做ownership分析，不把延到函数结束当修复 |
| D-927-2：native合法body暖编辑被拒/拒绝分类损坏 | `av/src/session.rs::prepare_edit`保留环境接口门、`native/src/incremental.rs::edit/close_exit`、真实host `mir_native_incremental_formal_runner_gate`；既有性能收据59/60、唯一mn.t11.refused | [932](../932-native-warm-edit-admission-and-refusal-closure.md)：先区分合法body编辑与故意签名变更，合法按原快路径成功，真签名变更仍拒；typed原因/父侧ERROR与语言FAIL分清，原P0/时间门保持 |
| D-925-1：sema测试读git忽略附件 | `semantics-store/tests/cases/sema_boundary_audit.rs`两测试读`docs/plans/attachments/796/contract-decision.md`；主检出残留可绿、新树缺文件红 | [933](../933-ace-tracked-oracle-and-fresh-checkout-closure.md)：改依赖跟踪规范/最小独立冻结夹具，保原schema/版本/描述面断言，不恢复ignored附件 |
| 904缺manifest金样 | 20个collections case要求golden-*/manifest.json逐字节，原目录仅main.rs/runtime/generated.rs；925收据明确历史缺件，当前L04源码查证一致 | 933同属“从干净源码可重放验收”的有界职责；保全部原source/结果/三份已有金样及manifest断言，由独立合同推导补缺，不能复制actual充oracle/删expect |

只读调查确认代码形状和缺件；本轮没有复跑，不虚报新失败或关闭。D-927-1已由927修复，不能重复立项。D-911-3非收敛fallback具名覆盖缺口、其他环境/图形/产品债均按原owner保留，不在本波偷偷并入。

## 3. 本波正式计划与执行顺序

| Plan | 范围 | 状态/优先安排 |
|---|---|---|
| [931](../931-a2r-owned-temporary-consumer-lifetime.md) | Rust owned临时值消费者生命周期，跨实参/嵌套/模式、资源与增量/缓存 | drafting r1 0/10；高优先，私有Rust lowering可与语法线并行 |
| [932](../932-native-warm-edit-admission-and-refusal-closure.md) | native暖编辑准入/真实分类与正式父门 | drafting r1 0/12；高优先，native/父侧可准备，av session/core接口与928/929取窗 |
| [933](../933-ace-tracked-oracle-and-fresh-checkout-closure.md) | sema跟踪来源与904缺失oracle，干净源码可重放 | drafting r1 0/10；可并行先做证据面，优先释放sema/904材料给931/932最终消费者验证 |

推荐实施并行三线：原owner `927合入→928→929`；`931 Rust资源`；`932 native准入`。933证据面可先于或穿插其间，不自动启动会话。933的独立跟踪输入/904manifest先释放稳定提交，931若必须改变manifest的明确生产者版本字段，则由931在该基线上做最小可追溯更新；933不依赖931生成actual，不形成循环。源码/值语义不因补证据改变，原金样不批量重生。

本轮取号期间其他会话已创建930 generated-delivery-regression-and-source-boundary-closure骨架，本会话避号为931–933；930现已填为drafting r1、0/9，归生成UI交付回归/L1-L2来源边界治理，明确排除SG语法；它与本波代码主体独立，不冒称其负责上述ACE源码缺口。公共test-groups/infra等H若重叠须核实际930范围，不替其owner承诺窗口。

## 4. 写面与验收合同

| 包 | 私有/公共写面 | 与当前线交接 |
|---|---|---|
| 931 | P=`a2x-rust/{lower,lower_coll,provider,print,rt}.rs`按实际同因检查；S只读受检资源计划；H=worker Rust认证/注册 | 不改C已修drain/runtime；不写927/928 grammar/AST。A2xSession恢复等由929保主责，931只接缓存语义版本及补例。共用worker/manifest字段逐符号单写 |
| 932 | N=`native/incremental/source_diagnostics/worker`；S=`av/session`与core retained接口必要修复；H=ac_mn/strict parser/host formal tests | 先冻结D927真实版本，928/929语法body目录/typed结果接口单写；不放宽签名拒绝/类型规则，不用整模块重算使暖门绿；正式性能与927/语法campaign错峰 |
| 933 | P=sema边界测试/跟踪规范夹具、904 oracle补缺；H=必要case来源清单/验证入口 | 不改sema求解器/emit逻辑，不删manifest断言。若发现实际产物错误，具名转代码owner而非重写oracle；903/908旧规则/分母/accepted哈希不改 |

每窗须记录owner、plan revision、base/交接SHA、schema与全部producer/reader/construct/match、验证/STALE/pending和释放条件。H全文件单写，未登记不能并写。各包包含fresh/hit/restart/body-edit与失败修复、真实未改函数生产0/必要恢复重验、原资源与P90≤2000ms适用门；门的负载与样本范围不擅扩大/缩减。

## 5. 后续能力规划仍需继续

- **CA-D有限ABI桥**具备A/B资源前置，可先调查/设计；本轮未取其号码。现`a2x-core::ExternalFn`只是单体视图中的兄弟函数签名，HIR DefKind只有Function/Method/Intrinsic，native scalar对AV HostCall明确拒绝。不得把这些接口当成已有通用FFI。建议正式立项时按“中立接口事实＋真实typed foreign标量三腿”与“fallible/buffer/record/handle/同步callback生命周期”拆包；继承797 candidate/descriptor3，不引入任意use.c/header importer，不让A2X依赖Cranelift，必须有native ABI与A2C两个实际消费者。此处只是后继拆分建议，完整范围/AC待其正式计划，不算已承诺实施包。
- **SQ/INST/CT**：796模型与817真实AV执行基线保留；普通源码查询/真实实例/冻结事实接线仍待有界设计/立项，不因931–933修复就冒ARCH-02/04全闭。
- **CA-E/ARCH总签章**：先准备版本化证据组合，最终等原门+C门+SYN/ARCH必需项；932应恢复其正式腿，933应消除隐形附件/缺件造成的假环境绿，不能代总签章。
- **M2**：继续922全应用/目标/宿主/工程增量/部署/disable盘点后继，v0.6 M1+M2硬门不变；本轮不替产品线自动创建或启动计划。

## 6. 本轮验证

仅计划结构/任务-AC-V/文件链接/固定组/编号/交付祖先及写面核对，未运行cargo构建测试、计量或认证。931–933均drafting，原在途/归档Plan不修改进度，未建worktree/材料scope；开工材料仍仅plan-artifacts/plan-NNN，merge折叠摘要后清理。

终检主线推进至`6e021ccdc`（他方AG文档提交）；927仍reviewed 10/12、唯一树35ad072b2，928/929仍原状态，以上11项源码输入hash未漂移。931/932/933分别0/10、0/12、0/10，ID/本地链接/固定组/任务-AC-V/空白及D923–926祖先核对通过；无实现/运行认证结论。

## 7. 源码调查输入身份

开工先比漂移；这些hash不代交付/复审收据，不依赖主检出隐形附件。

| 输入 | SHA256 |
|---|---|
| `experimental/ace/crates/a2x-rust/src/lower.rs` | `99e51d864379db218c804e64c9cb861c3a639a7bb971e48011584c6ef19d42c1` |
| `experimental/ace/crates/a2x-rust/src/lower_coll.rs` | `1cb83f1b90d0e28afb9969dcc85abd7b1e327441ce68378aada61f38421e506f` |
| `experimental/ace/crates/a2x-rust/src/provider.rs` | `716a18936a0db2c64bb5549dd32fa75c6d7beed3ea90eb2e2910c887dd73a94b` |
| `experimental/ace/crates/a2x-c/src/lower_rt.rs` | `ec85d1de02e83f93f94bf2c496d69cbaaf897b3ab62e4d1785d310e3c11c2d69` |
| `experimental/ace/crates/av/src/session.rs` | `5f8a603fe642d80cc85c705dcbda3da0d49f43b3f2c469c400b8f61961844b8e` |
| `experimental/ace/crates/native/src/incremental.rs` | `93e988dae6311e8431aade9ed74ef9bb4c9fa134a546e27f5b88c76e379a9298` |
| `experimental/ace/tests/conformance/tests/host/mir_native_protocol.rs` | `1f853313dbb51ad5b067d1c222d1ec8ad1271bd49987b4d1123306b00b505112` |
| `experimental/ace/crates/semantics-store/tests/cases/sema_boundary_audit.rs` | `8c64dbb62d9b7dac98e14374c44326c4909b50e21317fa32c0114691df804036` |
| `crates/auto-lang/test/cases/a2r-hir-collections-904/L04.case.ad` | `809e271828a1dd7312c3259b926155bf068627c78de2d289edb124085698569c` |
| `experimental/ace/crates/a2x-rust/src/coll.rs` | `329db39a00d336cb27c15afe87215341b2b330fe053434b36a2287ce2190ce34` |
| `scripts/test-groups.toml` | `c9ac36b50a314f228e5ed628456402d562a877a297dbfc5d7bdaa3f5237643fe` |
