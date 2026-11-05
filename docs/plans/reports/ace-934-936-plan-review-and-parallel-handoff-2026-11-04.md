# ACE 934–936 r2：事实消费、实例/CT依赖与恢复首编辑复核（2026-11-04）

用户仅授权规划复核/遗漏直接修订，不施工、建树、运行测试或派发。本轮按auto-plan:new复核后，934/935/936均drafting r2，仍为0/13、0/12、0/14；任务/AC编号和所有原阈值/适用义务保留，补例全未执行。这是stage:new静态合同pass，不是独立实施review。初读main f6b0f918d，核查期间推进0771a3b9e；以当前Spec/源码为事实、Design42为目标，未修改current Spec或他方计划进度。

## 1. 状态与实际接口刷新

| 计划 | 当前证据 | 对本线的意义 |
|---|---|---|
| 928 | archived r2，D928=6b429502f为main祖先；头10/12为簿记残留，merge/cleaned已记录 | 九profile/UnitCatalog/sealed body是可用基线，不再等待其整包；不把语法reuse等同语义依赖 |
| 929 | executing r2 1/14；lang-929保留，T-01旧清单/其用户挂起纪律存在 | D928前置现已具备，原会话按其§10恢复/刷新；本会话不代resume。source/env/恢复required/会话仍是其公共S |
| 931 | archived r2 10/10，D931=84c894b92为main祖先；Rust print/2 | 消费已修资源/modes和新缓存轴；D-931-1来源恢复首编辑未闭归929，D-931-3补入manifest的独立来源仍需933审计。归档不等于这些余项绿 |
| 932 | executing r2 0/12，lang-932已建（基面84c894b92） | native/AV候选、typed结果/父判是真实活写面，不能按draft看作空窗；继承914策略verdict |
| 933 | drafting r2 0/10 | sema跟踪来源、已补904 manifest独立出处/负控仍待核，不能继续按“20件缺文件”排期或拿actual补件作oracle |
| 934–936 | drafting r2，未见实施树 | 本轮仅修计划；新source-query/source-cte包与事实端口尚未实现 |

931核查中从reviewed进入delivered，其树在只读查询之间被原owner清除；本轮没有操作其树/分支/材料。796/817历史delivery非当前祖先的原说明保持，当前重组source和Spec/hash为继承基线；D928/D931是真实当前祖先，不能混为一种状态。

## 2. 发现与当前修订合同

| Finding | r1不足与当前源码证据 | r2关闭要求/对应任务与补例 |
|---|---|---|
| PR-01 事实是否真实驱动 | core compile_program_observed先resolve_with（已产生VT/ResolvedExpr）再typecheck_with，单禁后者不证明换权威 | 934 T-02/04/08，V-09：绑定与类型综合职责分离；正式Core/Scalar consumer默认走新服务，miss禁query红/合法facts禁旧综合绿/坏facts真core拒且无fallback/hit双禁solver绿，真实type-use和独立HIR对账 |
| PR-02 facts信息不足 | 796 InferFacts按name及span稀疏编码，同名遮蔽、source漂移不能直接作为完整事实键；named args当前Unsupported | 934 §5.5、V-09/10：scope/local/expr结构ID+body/context绑定/完整覆盖，actual E01 value7、trace=b,a；同类型错scope/同签名异callee的负控，不靠类型不兼容证明身份 |
| PR-03 目录module不是LogicalModule | D928 UnitCatalog::build令module=content_sha(src)，SealedFn载荷span为mint，current span在包装 | 934 T-03/05/07、V-10：显式LogicalModule/Decl映射，源FNV/目录instance只是来源授权；不将全source变化灌进语义key，也不原地改旧pin span |
| PR-04 根安装责任有环风险 | source-query宣称backend-free，r1却说它原子切source/target root，未给宿主协调点 | 934 T-06/08，V-11；935 V-10；936 V-14：kernel prepare→selected target ready→提交前全dep/base/caps/cancel重验→宿主线性安装。纯语义/meta query不构建target、不冒执行替换；四个target可独立版本，不新增全球事务框架 |
| PR-05 实例逻辑/实现/集合的分层不足 | template body改后既要caller P0又要target调用新实现，不能靠换逻辑id/重写全部caller；最后user消失不能永pin | 935 §5.5/6，V-09/10/11：逻辑InstanceId稳定，实现recipe/effective checked deps换版；目标闭包/代际钉新实现，slot重映射/集合更新/历史pin最后回收。机器码可CAS去重，但类型身份不能合并 |
| PR-06 仅源码body依赖漏子CT | 模板或helper内Hash经回灌改变checked实现，而模板/caller源码未变，原817源码级fn fingerprint不能单独证明新源 | 936 V-11：flag内CT→pick<T>实现→outer CT，true→false导致40→42；子query值/类型/dep与实例effective body传递。935只预留typed dep，不以schema认领实际CT |
| PR-07 执行key与source绑定冲突 | 817 CteExecutedV1持source_fp；若source/owner全入新ExecKey，无关编辑也全miss；若完全忽略则旧owner可误授信 | 936 §5.7/V-12：ExecKey只读真实执行语义，UseBinding逐当前owner/node/source/期望/context重验；原执行origin≠本次来源，差额换source schema/namespace，不改同key旧payload |
| PR-08 真AV/值投影还需可区分证据 | 正确结果+自填count1不够；旧cte_value_of按任意非零I32作bool | 936 V-12：独占空store冷miss、实际execute边界故障已到达、hit禁执行、严格Bool或已审0/1投影，I32(2)/错tag/错query结果拒；不能hostfold、伪marking、未消费/旧constant冒回灌 |
| PR-09 CT相位与lazy语义未定清 | nested Hash与普通runtime短路不是同一需求时机；旧schedule以deck声明序不能当源码限制 | 936 §5.8/V-13：所需owner/实例中显式Hash是编译需求，嵌套Hash先依赖；单Hash普通pure执行的未走runtime trap潜伏，类型/权限错误仍检查。权限与宿主分域、CT无IO；不是旧CTEE已交付行为 |
| PR-10 恢复首编辑新页owner不足 | 只说“等929”会漏query/实例/Exec-Use/执行页；decode调用producer再标R会伪0 | 934 §5.6、935 §5.6、936 §5.9及V-14：929 source/CST/decl、934 facts/deps、935实例、936 Exec/Use/编码执行页各自产/验required，选中target宿主安装。R只复装/深验已有；缺页P0拒、显式cold另计，首实参编辑旧helper/实例P与I/O0 |
| PR-11 计数/等待/新需求边界可被滥用 | 失败早退/全body walk改M、跑后把额外lower称新需求、子CT各自10s、单飞取消/锁跨AV都会掩盖真实成本 | 三包T-02先定真实stage与affected集合；936 V-13/14要求累计remaining deadline、每AV≤min(2s,remaining)、solver≤其原预算/remaining，等待纳父窗；一个waiter取消不污染另一合法waiter，锁不跨执行，原固定负载零门不松 |
| PR-12 并行边界已变化 | 928/931已交，932开始，source-query是B/C共同写面，933不再只是补缺文件 | 本报告§3及三包§5当前交接：私有面与公共集成/最终验收分层，D931 /2与真实余账继承；source-query/core/grammar/Cargo/decoder/H均单写，接口漂移STALE，不豁免原红或自动改原owner进度 |

原全部V保留，补V分别934:09..11、935:09..11、936:11..14；最终V数11/11/14。AC仍11/11/12、T仍13/12/14。新例均由各原work任务使能；原sample/ignore/platform/预算/阈值/P90/expected/NA/accepted-diff与分母保持。

## 3. 并行、公共写面与最终前置

| 包/阶段 | 后续获实施指令后可先独立做什么 | 等待/交窗 | 最终关闭条件 |
|---|---|---|---|
| 929原线 | 原owner依据D928恢复入口刷新清单/稳定source/env/required接口 | 当前source/AST/grammar与AV-native-A2X会话S仍其owner；本轮不替它解除挂起 | 实际source恢复→首编辑/SYN及适用消费者证据 |
| 934 T-01及私有query设计 | 单体facts/读集/键/独立用例/legacy切断方案可与929、932、933分私有面并行 | T-02事实消费整链/T-03..08公共接线需929稳定来源/事实port窗，core retained/typed若与932同符号须先释放 | 不能只跑独立probe；正式四端source入口、D929或正式等价交付恢复、933跟踪材料真实可验 |
| 935 T-01准备 | 有限模板语料/身份/实例key与独立reference可提前 | 实际instantiate/事实消费需934已审稳定端口；grammar/AST/source-query在934与929释放后串行 | 实际D934/D929或正式等价接口、具体四端实例/集合/首编辑全闭，不带pending送审 |
| 936 T-01准备/服务面 | 817调用图/旧控制/无环最小提取设计可先；934端口就绪后服务实现可与935模板主体并行 | source-query/共享codec不是私有；935模板规则窗后936 Hash/CT依赖接线，同Cargo/lock/H单写；T-08实例组合要935真实能力 | 真AV/checked-child依赖与Exec-Use、D934/D935/D929或等价已正式交付、四端/恢复/原资源样本全部实证 |
| 932已实施 | native/AV暖准入/typed拒绝与914父判 | 与934 core facts、936 source diagnostics/worker同文件/符号取窗；不由本线补其未完成阶段 | 本线消费稳定typed接口，原ERROR与策略verdict不回退 |
| 933材料面 | sema跟踪fixture、当前904 manifest独立来源审计，可与query/实例/AV服务主体并行 | 测试/现golden目录与H单写；新schema不能顺带重生旧oracle、改旧796常量 | 必要职责组原红按真实owner解决；有文件/旧组绿不能代独立来源证据 |
| 931已交付 | 资源/modes/print/2为基线，无需等其整包 | 不重开Rust lower/runtime；A2X provider/manifest/current-source恢复字段若变仍需929/相关owner窗 | D-931-1首编辑与D-931-3独立oracle保具名，不能据归档一笔划闭 |

**推荐序**：先推进929公共来源与934私有query（932/933各自独立线继续）；934稳定接线释放后，935模板主体与936最小AV服务可并行；共同source-query/grammar/AST的建议短窗是`929→934事实端口→935模板/实例→936 Hash/CT`。该顺序只是实际写面释放，不要求前包全部归档才准私有研究；也不表示没有稳定端口时就能完整实施935/936。若同文件已有其他owner持窗，按实际状态先让其完成稳定切片，不能凭此建议抢窗。

每窗记录owner/plan_revision/base+handoff SHA、实际文件/符号、profile/schema/producer/lane/target与全producer-reader-construct-match，已验/STALE/pending/反控/释放。H含workspace Cargo/lock、worker/SDK/父decoder、mod.rs/test-groups/manifest来源，全文件单写；慢认证前释放写窗，正式性能单实例错峰，不新增整波依赖或新调度器。

## 4. 通过前必须能逐条回答的事实

1. 哪个真实query结果字段进入哪个core类型/调用绑定/具体HIR节点？禁旧resolver综合后普通worker是否仍成功？坏同类型事实是否被主编译拒？
2. 逻辑id、source/catalog身份、语义body、checked有效实现、target recipe、ExecKey与UseBinding各在哪张页/哪次request中？不变source而子CT变的链是否真的重验？
3. 冷miss真实AV是否可由边界故障证明；bool投影与缓存准入是否严格；命中/恢复是否禁执行仍可用，而本次owner facts/原target观察确实当前？
4. restore入口、current root、required页、R加载与首编辑P/attempt/I/O是否同一连续父窗？缺页有没有隐式重编？每个新页是否明确由934/935/936某一包生产而非都推给929？
5. 目标准备/部署失败时，source/query/实例/冻结与选中target handle是否还同代？提交后丢ack能否按原版本/请求身份恢复，而不是再提交另一根？

原796/817有限机制、shared MIR/AV/store与866在线更新不重建；这次只补真实接线合同/证据。新finite profile语义不等于完整泛型/CT/反射，M1仍须原门+C+SYN/ARCH组合；CA-D与最终组合准入仍独立待立项，M2/v0.6全替换硬门不变。

## 5. 仅规划核验与输入身份

本轮不修改934–936实现或current Spec、不建树/材料scope/目标产物，不调用cargo或运行源例/故障/认证。已有他方WIP和931/932实时推进原样保留。27项原基线仅作初订历史，下表为r2重新读回的真实输入hash；未来实施先核漂移，未知来源或mandatory前置未就绪不能假PASS。

| 输入 | SHA256 |
|---|---|
| `experimental/ace/crates/core/src/source/adapter.rs` | `42a597804be233ff1f5bef99a7b2a290ebf0c5d88b7f5bb8f06bb49a9e2fa996` |
| `experimental/ace/crates/core/src/source/resolve.rs` | `75524330204f13d4998ba7e05d0f96894610b4a7a7523ccf3a4abb9faffa3abc` |
| `experimental/ace/crates/core/src/source/typecheck.rs` | `dafbc64a68cf5202f81dfea803ac9fbac12fcfc47618fbada2ba171941840aa6` |
| `experimental/ace/crates/core/src/source/cst/units.rs` | `28c0ab5bb8b670082a714d08ccff035888f502e9f39b8ff6f6a2fa55d02dcc12` |
| `experimental/ace/crates/semantics/src/infer.rs` | `24df508fbf24b1003d384be9c18a48bc950814b71441e04060287eb4fe9ef33d` |
| `experimental/ace/crates/semantics/src/input.rs` | `c54b3d4e1d1bdc6bf4e8669206aebf8b3bc7a5e84615c04ecd29ecfc01bb87f9` |
| `experimental/ace/crates/semantics/src/adapter.rs` | `9f351233d219b63b15bb3f4ee45bff474ce80feddd65ba4af5ab8dab0a39bf22` |
| `experimental/ace/crates/semantics/src/query.rs` | `ca84232dd58871488d8f81f1bfb92dc7c83ed4636b90818608205084b8c28ca1` |
| `experimental/ace/crates/semantics-contract/src/codec.rs` | `cef3ff5fe6eae4ee4f68a0dcdbec405301253511e71cd74197aaafb4da7e105e` |
| `experimental/ace/crates/semantics-store/src/store.rs` | `0d9ca64a68259d25b634ddf2e270bfdf032ea5438bd699faf8119ed80a189ddd` |
| `experimental/ace/crates/comptime/src/av_driver.rs` | `16333d4b65fe1e1aff3d4919a306a8e71bfbb6e03f351ce69672269755dc52e9` |
| `experimental/ace/crates/comptime/src/driver.rs` | `d6385d801d6bd89b7e8f324d16277d94341633da6f4ffaf57b7061eb61a6e45d` |
| `experimental/ace/crates/comptime-core/src/consumer.rs` | `c2ae6972ab74f848c2412feac29606c80ceb004c3cf14afc704f63368dfae8d7` |
| `experimental/ace/crates/comptime-core/src/dependencies.rs` | `372d2d514fdf892f607bf12ed35030e0f19e0d82dc444c9d0dc250064f4c24ae` |
| `experimental/ace/crates/comptime-core/src/schedule.rs` | `b69473fb6aad533267fba69dfd75ecd8d30bf44013575e12c823be7bd78daa38` |
| `experimental/ace/crates/comptime-contract/src/frozen.rs` | `0739f697b0fea07e060acec78d2dfbaceaf79e92b27a92a77a504986817eca79` |
| `experimental/ace/crates/a2x-rust/src/provider.rs` | `bf77b1dcb2e6ccc180c0042cefcbd95565ee21444e61c1895a4c6dbaf68ea60f` |
| `docs/specs/auto-syntax/composite-source-projection.md` | `529bc7eccd6f9028d4ad0224d5db86f66d36b54689e4e11ad1a50a44fa8bb891` |
| `docs/specs/auto-a2x/temporary-consumer-lifetime.md` | `8cfdc8e5833a27772e923723bb07ba18d300dd068a3ebb43b451221a27efc8b0` |
| `docs/specs/auto-semantics/inference-profile.md` | `e570ff2a3e55800c915409315933cedfb9c4f7ea3d538a625a8c16cfe2b6441a` |
| `docs/specs/auto-cte/scalar-query-profile.md` | `35fd17ba1ba199ec731182dd6f187945b3f5110b2ac9d015c7fe81da4e2781ee` |
| `crates/auto-lang/test/cases/ace-m1/order-layout-914/src/E01-ace.at` | `4863d1c2afe44fd41d844a0443085fa0e4fb9b7e6f5f0dbead97b38600339dcc` |
| `scripts/test-groups.toml` | `bcf1593d98c366552c6e08b8bc87519e28ede3cccfe0e38d1aee38cbd0ac22b5` |


## 6. 规划静态终检

终检HEAD=`353a2ffa1`；23项取材源码/Spec/group hash均未漂移，D928/D931为当前祖先。929仍executing r2 1/14；931 archived10/10；932 executing r2 0/12；933 drafting r2 0/10，本轮未改其进度/授权。934/935/936分别0/13、0/12、0/14，章节0–10、T/AC/V/SD唯一连续、补例任务映射、已用固定组/新文档本地链接、空白及无实现树/材料scope核对通过；计数为11AC/11V/4SD、11AC/11V/4SD、12AC/14V/5SD。new-stage静态pass，未运行编译、源码用例、故障、性能或认证，不为任何新AC声明实现通过。
