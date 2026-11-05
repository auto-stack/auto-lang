# ACE 931–933 r2复核、并行窗口与M1余账（2026-11-04）

> **2026-11-04后继规划更新**：本报告原“SQ/实例/CT未立项/或1–2包”为当时快照，现已正式拆[934普通源码query](../934-source-semantic-query-and-invalidation.md)、[935有限真实实例](../935-source-scalar-template-instances.md)、[936普通源码AV CT](../936-source-av-cte-query-and-frozen-recovery.md)，均drafting r1、未实施。详[新基线与写窗](ace-source-sq-instance-cte-baseline-and-schedule-2026-11-04.md)。CA-D有限ABI与最终ARCH组合准入仍未立项；计划成立不等于机制已交付。

本轮仅规划复核，未建树/派发、未修改实现/current Spec、未运行构建/测试/性能/认证。用户授权直接修订遗漏；已批准的914策略继承，不新造裁定。初读主线`ffc59e505`，终检推进`50555f3d3`（928 T-04簿记，进度由3/12→4/12；下表取终检值）；在途分支只读确认写面，均不是正式D。本报告取代[初订下一波调查](ace-next-wave-correctness-and-admission-2026-11-04.md)中的当前状态/932分类及恢复交接表述，原运行收据保留为历史。

## 1. 当前状态与基线

| Plan | 本次查得状态 | 解释 |
|---|---|---|
| 923–926 | 均archived，D923=ddd42ef2b/D924=48c2d8066/D925=a37245cd6/D926=3fce61b9a | 已交付，不因新缺口重开 |
| 927 | archived r2，12/12；D927=5d5054d5d，03e9869fb清树收据 | 主线祖先，旧“reviewed待合入”已过时 |
| 928 | executing r2，4/12；lang-928在途 | 原owner继续Collections/Closures及sealed body单元；尚无D928 |
| 929 | executing r2，1/14；lang-929基面03e9869fb | T-01已盘点；按用户明确裁定挂起等D928，不能代其恢复/接管 |
| 930 | drafting r1，0/9 | 生成UI三原红/A-B工具交付与语言L1/L2边界；是独立交付治理线 |
| 931 | drafting **r2**，0/10 | Rust消费者临时责任与片段模式事实；本轮新增验收未执行 |
| 932 | drafting **r2**，0/12 | native暖准入、typed有效性与**继承914的策略判面**；不是回退911旧断言 |
| 933 | drafting **r2**，0/10 | 两sema隐藏依赖+904二十个manifest独立来源/干净检出验收 |

原D-925-1/2、D-927-2仍open。928已提交的实际写面含core/source/cst/{frontdoor,kinds,mod,project,units}、auto-source.at与core conformance；本轮读树时其status clean，不代表公共面已释放。929尚无实现提交，但已登记的公共迁移职责仍由其owner持有。

## 2. 发现与直接修订

| Finding | r1不足/源码证据 | r2当前合同与关闭位置 |
|---|---|---|
| PR-01 消费点不是下一个pin | lower.rs三个pin全drain，语句根/return/条件在真实求值前也可能drain | 931 §5.3/V-07：RStmt真求值点、替换先RHS成功后提交、派生view底层owner保持；非法逃逸仍原verify拒 |
| PR-02 短路/嵌套责任不能串域 | Logical右支mem::take整份pending，可能偷取外层责任 | 931 T-02/03/V-02/08：左右短路/实际执行、嵌套consumer和外层pending独立反控，未执行支真实alloc/free0 |
| PR-03 cold与片段参数mode不一致风险 | ExternalFn已携modes，lower.rs external分支却填View | 931 T-03/V-08消费原modes、完整绑定/长度，不默认猜；move/mut/view在cold与片段执行等价。限定已有事实出口，不重做ownership |
| PR-04 “紧预算”验收没有真实runtime门 | rt.rs只有fault序号/try_reserve/event，没有累计存活字节配额 | 931 §5.3/V-04分开编译工作集、生成runtime原分配/fault门、独立live峰值断言；不以allocation序号冒内存峰值，不借幂等free/进程退出掩盖责任 |
| PR-05 932旧ERROR口径违反较新批准 | 914 Spec §5/7+归档复审N3明确有效typed策略拒绝为verdict且覆盖全部增量mn；911旧句/host Error断言未同步 | 932 §5.1/5.4、SD-05/06、V-02/06/08继承914：有效拒绝按diagnostic预期PASS/FAIL，缺/坏typed/协议资源为ERROR；原t11成功预期与源不改，只有过时“必须Error”测试断言具名纠偏并补有效/无效双控 |
| PR-06 合法body与故意签名现象可能混记 | D-927-2描述混合，原host确有合法body与故意新增参数两个不同输入 | 932 T-01/02先独立重建，若合法body本来正确就有证澄清，不强造core修改；签名门不放开，旧结果即使恰等expected也不跑成成功 |
| PR-07 恢复首操作含隐性跨包接缝 | Rust rt/coll无AXP来源锚，普通restart/whole run不是source-bound纯恢复；ensure_env仍可能全源重建 | 931 V-09、932 V-09明确新增首编辑最终验收消费929或等价已释放接缝，未有则pending；933保持原支持门，新SYN机制归929，不自称已完成。R深验保留、真实P/attempt/I/O分列 |
| PR-08 oracle来源只写“独立”不够 | sema两例完整约束多于token；manifest含20例源/entry/版本/执行配置/格式精确字节 | 933 §5.4/V-07/08：全部sema逐项来源与13TypeDesc/3Mode，manifest逐字段/JSON字节出处、独立参考算法、真实父侧负控；unknown来源先needs_replan，生产emitter不能生成oracle |
| PR-09 并行开始与完成条件混淆 | 旧文仍等待927；未明确933黄金目录与931的先后，以及929首编辑接口如何避免循环 | 三包§5和本报告§3：D927现已就绪；933先释放稳定独立材料，931/932先交私有修复/稳定接缝供929消费，最后各自联验，不互等整包。H/904目录单写；不改928–930进度或持窗 |

r2仍是planning stage:new静态pass，不能当实施独立review pass。T/AC编号与总步骤保持，V分别9/9/8条；所有新增例由work任务使能。当前Spec冲突只在计划SD里安排沉淀，不在此轮改Spec来制造已交付事实。

## 3. 可并行范围、写面与交接顺序

| 计划/阶段 | 可独立推进 | 必须等待或分窗 | 最终证据/释放条件 |
|---|---|---|---|
| 928原线 | Collections/Closures语法、sealed body与单位目录 | 不与932并改source/cst投影、AST/typed接口；H同文件单写 | D928后原929可按其恢复规则续，不由本会话安排启动 |
| 929原线 | 已完成T-01，当前用户挂起等D928 | T-02后仍按其既定D928前置；不能因本报告自行解除挂起 | 语法/来源恢复稳定接缝可提前按已审SHA交931/932联验，最终含实际D927/D928及适用四端认证 |
| 931 T-01..04 | Rust lower/print/provider同因私有面、运行时probe、独立值/资源轨迹 | 不写928 grammar/AST；本包读core资源事实。新增观察若改worker/provider公共接口，先登记所有consumer | 无需等D928/D929/930整包；原modes/资源事实不变。T-05/06到store/worker交929短窗 |
| 931 T-05..08 | 缓存语义版本/旧错误片段拒绝、原restart和局部编辑可先验证 | 904 golden由933 T-04稳定提交先交；无预热来源恢复首编辑等929等价稳定接缝 | 稳定修复提交先给929，不持公共窗等933归档/慢认证；最终AC-05/06/原consumer须实证，不带pending交付 |
| 932 T-01/02及私有N/H | 115函数原链；以真实914 I09、911及t11输入/回包核typed分类，native/父侧独立反控 | core/source/resolve、AV session真实修复若需要，取S符号窗；CST frontdoor/units由928先释放，929后续迁移时再取窗 | 不等D928整包才调查/修独立native与adapter。原115fn负载保，不新增虚假规模 |
| 932 T-03..09 | 可先在D927基线修有效性/过时断言，交稳定typed接缝供929消费 | 928 source/cst窗口结束后接必要S，再交929迁移；若与929同符号已开窗则由当时owner先释放，不抢窗 | 恢复首编辑最后接929稳定SHA；原60项campaign义务+914 I09/坏typed对照，正式采样单实例错峰 |
| 933 T-01..04 | sema测试/最小跟踪来源和904二十个缺manifest，可与928/931/932并行 | 原904目录在933补缺期单写，931只读；不得接受931未来actual充独立来源 | T-04独立oracle/reference及来源表稳定提交即释放给931，不等T-10归档；后续生产版本变动由931最小独立更新，不回写旧baseline |
| 933 T-05..08 | 干净唯一树、原reader/emit与独立认证、原增量负控 | 如原必要consumer真实遇931代码红，保红并等待稳定修复，而非删assert/改golden；新SYN首编辑仍归929 | 缺件债只能按实证关闭；原门代码红不可豁免，新SYNpending具名给929且不称总M1完成 |
| 930 | generated UI driver、A/B随包工具、L1/L2隔离来源主体独立 | `scripts/test-groups.toml`/infra-fixed若交叉为H全文件单写；共享职责验证输入漂移重验；重编译/性能错峰 | 无D930整包挡ACE私有代码；ACE完成也不能代930 L1/L2/产品交付绿 |

其中932私有输入只有实际914/911/t11，不新增其他未定义语料规模。

**推荐安排**：原线`928→929`保持；同时`933独立材料`、`931 Rust私有修复`、`932 native/父分类`可并行开工。共享S建议928释放其CST/AST稳定符号后，932取必要短窗并交929；若932证明仅adapter/测试分类问题，则根本不占该S窗。H的case注册/decoder/mod.rs/test-groups/manifest-lock按完整文件轮流，不能同文件分区域并写。公共写面空闲不等于本会话取得其owner授权，实际执行先核当时owner/dirty/稳定SHA。

每次交接必须有owner/plan_revision/base与handoff SHA、实际文件/符号、schema/grammar/producer/runtime/tool身份、所有producer/reader/construct/match、有效/STALE/pending及反控和释放条件。慢认证前释放写窗，不能用整波依赖替代真实冲突定位。931/932可把稳定私有修复/接口提交提前交929使用，最终联验后各自交付，**不要求先归档任一包再允许另一包研究或消费稳定接口**；若按交付政策只能合入前消费已审稳定提交，仍必须保最终D身份闭合，未经审查的WIP不是交付。

## 4. 933之后，离M1还差什么

933是**证据输入与可重放门修复**，不实现新的语法/语义查询/CTE/ABI，也不能解决931资源或932准入本身；“编号最高”没有“最后M1包”的含义。

- **仅933先完成**：按本次状态还剩四个正式未交付ACE包：928、929、931、932（状态以后续真实收据为准）。930是独立产品交付/来源边界支撑线，不额外宣称为M1小语言核心的整包前置；必要共享输入仍必须可用。
- **若928/929/931/932/933整波均完成**：旧finite四端能力与统一语法/主要已知红门更接近闭合，但仍有下面三条M1必需职责未立项。不能说“再跑一次回归就M1”。

| 未立项职责 | 已有可复用基线 | 仍需实际完成 | 建议拆分/先后（未取号） |
|---|---|---|---|
| M1-SQ/INST/CT | 796语义查询/实例模型、817真实AV CTE→冻结→回灌及受检HIR重验 | 普通源码语义查询调度真实实例/meta/CT依赖；类型/实例/CT结果入统一cache并驱动失效，调用点未改也重验；恢复与未改生产0 | 先核有限实例/源码query与CT接线是否一包可收，否则分2包；必须复用真实AV，不另造host解释器 |
| CA-D有限统一ABI/FFI | 797 descriptor/3与win-x64候选、925/926目标内部资源表示 | 中立接口事实进入真实checked源码→native与A2C消费者；标量及fallible/buffer/record/handle/同步callback的有限生命周期。内部C结构不冒公开ABI | 建议2包：中立事实+真实typed标量双consumer；资源边界/失败/清理；可与SQ线按S/H分窗 |
| CA-E/ARCH组合准入 | 原M1登记/approved NA及accepted-diff、C认证、929 SYN报告、已有cache/HMR等机制 | 版本化严格父侧将原门+C门+SYN-01..06+ARCH-01..08真正组合；四端适用一致性、源码/工具/证据版本/STALE与独立认证、资源及P90；全部债具名适用/关闭决定 | 最后1包组合联验，先准备契约，不能把缺机制塞进报告包或拼旧PASS签新范围 |

因此，**整波交付后仍是3条必需工作线，粗拆约4–5个后继实施包**（SQ/INST/CT 1–2、ABI 2、最终组合1）。这是待可行性核对的拆包估计，不是已承诺计划数/工期，不预占号码；若只933单独完成，另有上述4个正式包仍在队列。后续实际审计若发现必要机制仍只在模型/探针，须有owner补闭环，不能静默写成M2语法差异。

D-911-3自然非收敛fallback覆盖缺口及其他具名存量债保留：最终ARCH owner必须判断其适用性与实际关闭证据，未决不能签章；932注入分类反控不自动消除该债。已有Typed MIR/AV/real CTE/store/真实HMR不重建；通用完整泛型/meta语法、全部stdlib/产品替换是M2覆盖，但有限代表机制是M1门。M2替换并disable旧AutoVM/A2X及UI/Man等所有在用场景仍是v0.6硬门，不因M1通过免除。

## 5. 核验与证据边界

终检确认904原20个case的manifest金样均缺；此为文件盘点，不是运行测试。表§6的14项源码/Spec/group内容身份无漂移；D923–927均为main祖先。

本轮只核计划0–10章节、r2/步骤状态、AC-T-V-SD引用、实际文件/符号/固定组与链接、写面和D祖先；只读源调查不构成新执行PASS。未创建931–933 worktree或plan-artifacts scope；原928/929/930进度/dirty/材料保持。既有main多会话未提交文档及其他未跟踪项原样保留，未暂存/提交/删除。开工材料仍仅plan-artifacts/plan-NNN，merge摘要入计划后按政策全删。

静态终检通过：931为9AC/9V/4SD、932为12AC/9V/6SD、933为9AC/8V/3SD；0–10章节/T总数/唯一ID/AC任务引用/固定组/新文档链接/空白、D923–927祖先和14项输入hash核对通过。此结果仅为规划文档检查，无运行认证结论。

## 6. 本次取材身份

开工须重新核漂移；以下hash只定位调查输入，不是独立测试或新交付收据。

| 输入 | SHA256 |
|---|---|
| `experimental/ace/crates/a2x-rust/src/lower.rs` | `99e51d864379db218c804e64c9cb861c3a639a7bb971e48011584c6ef19d42c1` |
| `experimental/ace/crates/a2x-rust/src/rt.rs` | `ffe3c83a971da74bfc071afeeeb6e799dd3c991ec0c23b44fcf6be2a3b74d57a` |
| `experimental/ace/crates/a2x-rust/src/provider.rs` | `716a18936a0db2c64bb5549dd32fa75c6d7beed3ea90eb2e2910c887dd73a94b` |
| `experimental/ace/crates/a2x-rust/src/coll.rs` | `329db39a00d336cb27c15afe87215341b2b330fe053434b36a2287ce2190ce34` |
| `experimental/ace/crates/a2x-core/src/semantic_view.rs` | `4527afa71e9dafb3e70f978ce15e10d3e8bfa651ef4b7d01fd0638ec18e4ee34` |
| `experimental/ace/crates/av/src/session.rs` | `5f8a603fe642d80cc85c705dcbda3da0d49f43b3f2c469c400b8f61961844b8e` |
| `experimental/ace/crates/core/src/source/resolve.rs` | `75524330204f13d4998ba7e05d0f96894610b4a7a7523ccf3a4abb9faffa3abc` |
| `experimental/ace/crates/native/src/source_diagnostics.rs` | `5e9b3accb45f10306bc711c602745a12b9ea4840bf783c838c4ba20ed25b7956` |
| `experimental/ace/tests/conformance/tests/host/mir_native_protocol.rs` | `1f853313dbb51ad5b067d1c222d1ec8ad1271bd49987b4d1123306b00b505112` |
| `crates/auto-lang-core/src/test_cases/ac_mn.rs` | `2faae6ece56766313562ce65f8152d6d680774760ced38e570ac3aff5d2f6ffe` |
| `experimental/ace/crates/semantics-store/tests/cases/sema_boundary_audit.rs` | `8c64dbb62d9b7dac98e14374c44326c4909b50e21317fa32c0114691df804036` |
| `docs/specs/auto-test-case/m1-order-layout-observations.md` | `f8c01052740f016ced6c245074c3465f34ba6217e5233754401f089c78f1f875` |
| `docs/specs/auto-test-case/profile-v2-native.md` | `e3a3e17ecc481c3bd74cacb4c3bb030ef919f62368537a18290b1a6e5b50cdcc` |
| `scripts/test-groups.toml` | `9f7d3e52a1ebd6bce9f6306c063e71ff7430c7119bb40f0d0345104ffbba290a` |
