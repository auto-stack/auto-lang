# ACE普通源码SQ／实例／CT：交付基线、正式拆包与写面交接（2026-11-04）

> **r1初订历史快照**：934–936已升r2，当前接口/实际状态和并行关闭条件见[复核报告](ace-934-936-plan-review-and-parallel-handoff-2026-11-04.md)。D928/D931已交付、932已开工；r2补事实权威、实例/CT有效实现传递、执行/消费键分层与恢复页责任/target安装，旧状态不作当前前置。

本轮仅调查/设计/正式立项，未建实施树、未运行构建/测试/性能/认证，不启动/联系其他会话。用户要求核796、817，闭合普通源依赖/实例/真实AV与函数增量恢复。取材初读main `f4062cca6`；原多会话未提交设计/计划及examples/ui/015-notes、native-fixture Cargo.lock、reader-import-budget均保留，不暂存/提交或处理他方材料。设计详[Design42 §10](../../design/42-semantic-metadata-inference-and-comptime.md#10-普通源码sq真实实例与av-cte接线2026-11-04target-state)。

## 1. 796与817交付基线：能复用什么，不能冒称什么

| 基线 | 已核正式收据/当前实现 | 可复用机制 | 缺口/注意 |
|---|---|---|---|
| [796归档](../archive/796-ac-semantic-metadata-inference-probe.md) | archived r1 9/9；原delivery b76eca8d2；当前semantics/semantics-contract/semantics-store（896归入ACE workspace） | 真实有限i32/bool推导与显式worklist、正负依赖、签名/body分离/SCC/名义身份/新鲜分类、S796页和ManagedStore/预算/恢复/AutoDown | 原来是AST→中立模型对照query探针，core默认source没有该query服务。InstanceKey是schema-only，不能宣称泛型已实例化。整Program投影、全局src-fp/current_fingerprints在暖路会导致未改body生产；新源单体页需新版本域 |
| [817归档](../archive/817-av-cte-query-frozen-cache.md) | archived r3 11/11；原delivery ec6a54c5d；当前comptime-contract/comptime-core/comptime（896重组） | 真checked HIR→shared MIR/ABC→AV、executed envelope/A05、helper闭包/meta负依赖/caps/engine完整键、freeze→infer→HIR verify→native、closure/snapshot车道及预算/取消/GC | recipe IR/deck+显式consumer锚，不是普通HashExpr自动接线；consumer会重新compile源、旧snapshot用源码wrapper；不能拿它证明新源码未改函数P0/恢复首编辑。comptime依赖AV/native，不能core反依赖其大包 |

**身份核查**：两原delivery SHA在当前图`git merge-base --is-ancestor ... HEAD`均exit1，故保“历史归档交付”但不称当前祖先或拿旧运行PASS直接迁移。当前文件内容/版本、公开Spec与下表hash是本次继承基线；934/936 T-01追当前组织/符号与原反控，变动输入重验，归档不重开。796当前query.rs最近路径提交1795038c9（896重组）；817 driver.rs最近路径提交30bb87666（927语法预热接线），说明只有旧delivery不足以定位当前实际代码。旧附件不是当前运行必要输入，933已负责其缺件。

关键现行版本：sema page `ac796/sema-page/v2`、codec2、InferAlgo `ac796-bidir-worklist/1`、S796/SnapshotRecord；CT request `ac817/cte-recipe-ir/v1`、recipe `cte/scalar-query-v1`、executed `ac817/cte-executed/v1`、producer `plan817/cte-probe`、layer `ac817/a05-cte-exec/v1`、algo `ac817-cte-query/1`，meta cap `cte.meta.fields/v1`。新source producer/lane/page版本独立，不能同键改payload或复用model标记授信；最终字面量在各T-02冻结。

## 2. 决定拆三个实施包

| 正式Plan | 独立出口 | 明确不接走的工作 |
|---|---|---|
| [934 SQ-A](../934-source-semantic-query-and-invalidation.md) | 普通Core/Scalar source→真实query推导事实→Checked HIR→四端，单体依赖/失效/source-map/统一缓存事务与恢复首编辑 | 不实现用户泛型或CT语法，不重做929来源/CST机制，不改931资源/932父判 |
| [935 SQ-B](../935-source-scalar-template-instances.md) | `fn choose<T>`有限i32/bool真实实例、同key共享/异T隔离、具体HIR/四端执行、模板依赖/函数实例P0及恢复 | 不把List<T>/InstanceKey模型代用户实例，不扩const/spec/owned泛型/runtime反射 |
| [936 SQ-C](../936-source-av-cte-query-and-frozen-recovery.md) | `#{expr}`普通源自动query→真实AV→冻结→推导/具体HIR与四端；source meta→AV实参、真实实例CT组合、完整失效/恢复/资源 | 不再造CT解释器/DB，不接任意FFI/IO、类型生成或全Hash语法，不拿旧deck背书新源 |

一个大包会同时更改语义权威、模板/新AST、执行服务依赖图和所有consumer恢复，失败难以定位；两包把真实实例作为小模型附加到SQ或CT，会隐藏其独立Checked产物/去重/目标符号/恢复义务。本次实际调查确定三包，取代前轮“SQ可能1–2包”的粗估，未放宽M1或额外预占号。

顺序为`934→935→936最终组合`。936的AV提取/原控制与935语料/身份可在934接口稳定后分私有面并行准备；公共AST/grammar建议935模板窗后936 HashExpr窗，936最终真实实例CT验收需D935或等价已正式交付能力。frontmatter依赖是最终能力/接口前置，不要求私有盘点先等整包归档。

## 3. 必须可区分的依赖与执行事实

| 改动/状态 | ordinary query/实例/CT应如何表现 |
|---|---|
| 仅helper body改，普通caller只读签名 | helper及其产物变、caller推导/语义生产0，链接/运行结果可变；FnSig缓存保持 |
| 仅helper body改，CT实际调用该helper | CT闭包依赖失效、真实AV重执行并回灌owner，未改CT调用点不是hit依据；无关函数/实例仍P0 |
| 只改callee签名/mode/type字段、候选/缺席名字 | 实际读者重验或具名诊断，不读者保持hit；meta/类型与实例键完整，名义同形异owner隔离 |
| 同实例多个调用/换一个调用者常量 | 完整key同才共享，instance生产可0，改变caller只改caller；模板body改变使所有活的真实实例实现正确换版 |
| CT依赖/能力/engine/lane/producer变 | 换键/受控拒绝旧依赖，不能因source未改盲hit；meta-only计meta，不计AV；model_compute不铸executed |
| hit/新进程restart/恢复→首编辑 | 禁compute反控、完整R深验，无source重parse/旧producer重铸key/出窗预热；失败首编辑→修复后与fresh同值/类型/来源 |
| 失败/取消/旧snapshot迟到/不完整required | 旧current/source/可执行链不变，candidate不半安装；typed语言/资源/infra与914策略拒绝判面保持 |

原796/817完整key准入和资源模型复用，但新单体lazy输入/语义与source-map分层、source CT入口必须有新schema/producer和原控制。对照双端相等、计数自行填0、JSON输出42、进程重跑正常或旧deck成功都不能关闭新AC。

## 4. 新接口的依赖方向与边界

拟source-query位于core之上、AV/native/A2X consumer之下，无backend依赖；core消费其typed facts的公开口但不反依赖其crate。拟source-cte服务从817提取，依赖query执行端口/comptime-core+contract/AV-core/shared lowering/artifacts，不依赖AV前端或native；宿主注入。A2X目标生成器仍只看HIR/资源事实，不依赖MIR/Cranelift/AV。具体提取与所需端口由934/936 T-02有界whole-path feasibility冻结，不把拟API当已存在。

有限语法与新能力明确写入Design42 §10.3：935为Auto既有`fn name<T>`拼写+隐式call类型约束；936为既有`#{expr}`拼写+新增有限cap保护的meta selector intrinsic声明。meta仅在CT读当前type字段，类型/选择子不当运行时值；不声称完整stdlib/meta/泛型已完成。旧profile原语料/拒绝/分母不改，新增profile与独立用例登记，原expected不被批量更新。

## 5. 与928、929、931–933的实际写面

| 计划/owner | 本线可先做 | 冲突/等待与释放 | 最终验收前置 |
|---|---|---|---|
| 928在途S | 934单体query/935语料/936原AV服务调查，不写其树 | 规则auto-source.at、AST、cst/{frontdoor,units,project}、FnItem构造器/typed源接口必须等原owner/D928稳定释放，不能借在途sealed-body实现当D | 原九profile支持/资源/闭包子体复用不回退 |
| 929 executing1/14用户挂起等D928 | 本线T-01/私有scope独立，不恢复/接管其goal | source-env/完整片段入口、来源manifest/restore、AV/native/A2X source会话与H需其已审稳定SHA。可交query端口稳定提交给929消费，不互等整包才研究 | 934最终ordinary source恢复首编辑必须有D929或正式等价已交付接缝；935/936继续复用，不复制源恢复 |
| 931 draft r2 P/H | query/实例/CT核心可与Rust资源私有修复并行 | 不改Rust lowering/runtime/print；A2X store/provider/manifest/证书consumer实际接线按其稳定producer/key取H窗 | 本线四端认证若触其原资源族需真实修复版本，不能用旧缓存/缺材料豁免 |
| 932 draft r2 N/S/H | 934query和936AV-core私有服务可准备 | AV/native candidate/core事实/diagnostic、ac_mn/strict/worker先由实际owner释放；本线不修暖编辑分类 | 继承914有效typed策略拒绝为verdict，坏事实ERROR；source-v1/恢复失败准入保持 |
| 933 draft r2 P/H | 新query/key/CT服务代码不等其归档 | sema测试/跟踪fixture及904 oracle目录只读，原sema组需其稳定材料；新schema不能改原796常量凑绿 | 相关固定组已知缺件真红不能豁免，稳定输入先交、原缺件和本线新证据各按实际版本 |
| 930实际树存在但头draft | 主体独立生成工程/来源治理 | 本线Cargo/lock/test-groups/infra变化若触其语言L1/L2来源/冻结边界，按文件身份交接并重验受影响项，不由本线重冻未知边 | 不新增D930整包前置，不冒称ACE绿可代其产品交付绿 |

公共S是来源/语义类型事实/候选/恢复；H是Cargo.lock/manifest、driver/SDK/worker、父decoder/TestCase注册/mod.rs/test-groups与来源边界，全文件单写。每窗记录owner+plan_revision+base/handoff SHA、schema/profile/producer、全producer/reader/construct/match、有效/STALE/pending与反控/释放。慢测前释放写窗，正式采样错峰；不把共享文件顺序扩为语义整波等待。

本轮取材时928头9/12/原tree在途，9291/14挂起，930头draft0/9但lang-930真实树已存在（按在途写面保留，不替其改状态），931–933均drafting r2 0步。终检若有新收据只更新本报告快照，不修改原Plan的进度/授权。

## 6. 验收与M1剩余安排

三包共同承接ARCH-02有限源码查询/真实实例与ARCH-04真实AV CTE接线，并复验ARCH-03/05/08受影响事实。完整门仍是原M1+C新增+SYN-01..06+ARCH-01..08；本线交付不能单独签M1。CA-D有限统一ABI/FFI与最终CA-E/ARCH组合准入仍未立项；已有cache/HMR/typed IR机制不重建，原具名债/适用性决策仍需最终owner核清。M2补全在用语法/目标/应用并disable旧AutoVM/A2X仍是v0.6硬门，不用M3/M4承接替换债。

正式工作入口各T-01；公共实现按稳定接口条件推进，最终适用四端/恢复/性能不能pending送审。三包分别13/12/14步（含独立review/merge），本轮均drafting r1 0进度。仅规划静态结构/AC-T-V-SD/固定组/本地链接/版本hash与取号唯一性核对，未产生新运行结论。

终检快照main推进至`da884ab2b`（他方AGE文档提交），27项源码/Spec/group输入hash未漂移。928 executing r2 9/12、929 executing r2 1/14用户挂起、930仍头draft0/9但真实树保留、931–933 draft r2 0步。934/935/936章节0–10、T/AC/V/SD唯一且数量分别13/11/8/4、12/11/8/4、14/12/10/5；本地链接、现固定组、任务映射、空白、跨active/archive ID唯一与号源937检查通过。未创建本三包worktree/材料scope，无实现/current Spec修改或运行认证结论。新增source-query/source-cte包测试职责分别须显式纳入ace-semantics/ace-comptime静态清单，由后续任务使能；依赖被编译不代该包测试已执行。

## 7. 当前调查输入身份

这些hash定位current source/Spec，不是新测试通过；未来开工先核漂移，并刷新实际依赖D/稳定接口身份。

| 输入 | SHA256 |
|---|---|
| `experimental/ace/crates/core/Cargo.toml` | `aaca4d881d1ea7ce3e917626cc675709f6162bc31a79cb0dd06b5bd15dfbb6bc` |
| `experimental/ace/crates/core/src/source/ast.rs` | `18072ea2dd934fac16b2f1e8e42a2dba3b15e7ce08b89f6ef5f0b8ddfa6d3971` |
| `experimental/ace/crates/core/src/source/adapter.rs` | `42a597804be233ff1f5bef99a7b2a290ebf0c5d88b7f5bb8f06bb49a9e2fa996` |
| `experimental/ace/crates/core/src/source/resolve.rs` | `75524330204f13d4998ba7e05d0f96894610b4a7a7523ccf3a4abb9faffa3abc` |
| `experimental/ace/crates/semantics/Cargo.toml` | `44065c31e09877b832f16e97936af4d21e5dff33392414cd2c711908c9cb6ae3` |
| `experimental/ace/crates/semantics/src/adapter.rs` | `9f351233d219b63b15bb3f4ee45bff474ce80feddd65ba4af5ab8dab0a39bf22` |
| `experimental/ace/crates/semantics/src/query.rs` | `ca84232dd58871488d8f81f1bfb92dc7c83ed4636b90818608205084b8c28ca1` |
| `experimental/ace/crates/semantics/src/input.rs` | `c54b3d4e1d1bdc6bf4e8669206aebf8b3bc7a5e84615c04ecd29ecfc01bb87f9` |
| `experimental/ace/crates/semantics/src/infer.rs` | `24df508fbf24b1003d384be9c18a48bc950814b71441e04060287eb4fe9ef33d` |
| `experimental/ace/crates/semantics/src/reflect.rs` | `f610865534b950da3bc4f440caa757e8866b53f5d378d2b94faf1f9744c81c26` |
| `experimental/ace/crates/semantics-contract/src/identity.rs` | `1fb29231e7d19c9a57a61b37496170b183e78ac882d5391d16f377c146f3df22` |
| `experimental/ace/crates/semantics-store/src/store.rs` | `0d9ca64a68259d25b634ddf2e270bfdf032ea5438bd699faf8119ed80a189ddd` |
| `experimental/ace/crates/semantics-store/src/budget.rs` | `be693328f6f0981b05c3216c6b5ef00d7c27d57d1165d8807e22469042f40799` |
| `experimental/ace/crates/comptime/Cargo.toml` | `05d3210836947476729c60df1dbe8fa92918e156806d190d36a8a85098398ad8` |
| `experimental/ace/crates/comptime/src/driver.rs` | `d6385d801d6bd89b7e8f324d16277d94341633da6f4ffaf57b7061eb61a6e45d` |
| `experimental/ace/crates/comptime/src/av_driver.rs` | `16333d4b65fe1e1aff3d4919a306a8e71bfbb6e03f351ce69672269755dc52e9` |
| `experimental/ace/crates/comptime/src/store.rs` | `0b22ca13b4dd7b8048f1f1429df4c62e1d935452b4241e1c1e6aa0b2388b7893` |
| `experimental/ace/crates/comptime-core/src/consumer.rs` | `c2ae6972ab74f848c2412feac29606c80ceb004c3cf14afc704f63368dfae8d7` |
| `experimental/ace/crates/comptime-core/src/dependencies.rs` | `372d2d514fdf892f607bf12ed35030e0f19e0d82dc444c9d0dc250064f4c24ae` |
| `experimental/ace/crates/comptime-core/src/schedule.rs` | `b69473fb6aad533267fba69dfd75ecd8d30bf44013575e12c823be7bd78daa38` |
| `experimental/ace/crates/comptime-contract/src/frozen.rs` | `0739f697b0fea07e060acec78d2dfbaceaf79e92b27a92a77a504986817eca79` |
| `experimental/ace/crates/comptime-contract/src/request.rs` | `8c6646ebf6c40436ae5653e0877c72d756d21e06fbc2bfe2ff338ab0f8c0a238` |
| `docs/specs/auto-semantics/inference-profile.md` | `e570ff2a3e55800c915409315933cedfb9c4f7ea3d538a625a8c16cfe2b6441a` |
| `docs/specs/auto-semantics/query-cache.md` | `9a184f09d59ecb8ccd6bc49c2f85786e9cc5051e34550a4dcadaad39fecaa43b` |
| `docs/specs/auto-cte/scalar-query-profile.md` | `35fd17ba1ba199ec731182dd6f187945b3f5110b2ac9d015c7fe81da4e2781ee` |
| `docs/specs/auto-cte/frozen-cache.md` | `f1599de88fe905499b2b50b433e2d5efe1c3f5986dbc16b042713caf80b65bd6` |
| `scripts/test-groups.toml` | `9f7d3e52a1ebd6bce9f6306c063e71ff7430c7119bb40f0d0345104ffbba290a` |
