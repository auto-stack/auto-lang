# 前端模型与Man依赖模型：并行边界及合入合同

2026-10-16；适用于 [799](../799-frontend-ast-model-extraction.md)、[800](../800-man-dependency-model-extraction.md) 与在途 [792](../792-generated-ui-independent-build.md)、[794](../794-scroll-host-provider.md)。这是新两份计划的实施约束与调度建议，不改写792/794已批准的语义合同、review结论或状态。

## 取材与真实写面

基面 `30d3e1d2a011e2a34243bb8a2b141259af721edc`，输入见 [指纹](../attachments/model-decoupling-20261016/inputs.json)。793/795归档已交付；792提审后仍有修复线，不能以execution_done计数当作final pass；794实现正在写入，不能只看其current_step=0断言未实施。

只读核对792实现树：提交diff含根Cargo、auto-man Cargo/automan.rs/rust_ui.rs、auto CLI main、发行fixture及相应Spec；检查时工作区无未提交文件。794实现树的未提交面包括根Cargo、auto-lang lib/tests.rs、ui/mod.rs及vm_bridge、vm/engine/host_capabilities/native，以及新增provider/consumer/双端证据。以上是瞬时观测，T01及merge须重新核对，不据此永久占用或删除别人WIP。

| 线 | 独占主要实现 | 必须保持的邻线接口 |
|---|---|---|
| 792 | 选定UI生成/交付入口、source/resource清单与admission、根source/版本/patch策略、fresh-lock/重定位driver | 不接管AST模型抽取或包resolver；新crate清单以实际依赖闭包决定 |
| 794 | 八scroll native、provider/DTO、renderer adapter/统一宿主安装及对应UI案例 | 不改AST类型/解析便利层；不改Man resolver、包锁及交付算法 |
| 799 | ast.rs/ast模型、上层ast_parse_ext适配、对应模型/解析测试、新auto-ast | 保持旧auto_lang::ast及ParserExt路径和类型身份；不改794的provider/参数/状态或792生成策略 |
| 800 | auto-man/resolver.rs、轻投影/语言适配、新auto-package-resolver及专用consumer | 保持Dependency/AutoManResolver、ModuleResolver trait及现有模式/路径行为；不改rust_ui/automan交付命令或794 |

各自一棵计划树、一套定向fixture。不得为减少文本冲突引入第二份AST/词法/路径算法或“common大包”。

## 小共享面：短窗口串行修改/合入

| 区域 | 各线允许范围及协调规则 |
|---|---|
| 根Cargo.toml | 792 owns source/版本/patch策略；799/800仅加自己的member/精确本地依赖和必要fixture exclude；794保留已有受管修改。禁止全文件重排/拿旧文件覆盖。各次合流从当前master做精确并集 |
| auto-lang Cargo/lib/tests.rs | 799加AST依赖、薄重导出/解析测试；794改宿主安装和自己的测试注册。同一文件不同片段仍先核查diff，登记短写窗口，不自行接管对方函数 |
| auto-man Cargo/lib | 800加入resolver core及必要兼容导出；792保持交付依赖/CLI接线。799/800仅在后合入且闭包受影响时，允许窄改已有source清单/依赖投影及producer身份，不借提取重写automan.rs/rust_ui.rs算法 |
| Spec/ledger/777配置基线 | 合流依照各自冻结SD和精确domain/边规则，索引/基线取真实并集；不把他线未提交ledger/WIP裹入提交。清洁受管树重新锚定，不宽豁免新反向边 |
| 交付source/resource清单、锁及缓存身份 | 实际包图新增auto-ast或resolver core时，由后合入者判定是否在选定生成应用闭包中。只更新受影响producer身份/allowlist/锁及定向回放，不假设全部workspace成员都要入包 |

## 合入顺序与责任

**可并行实施，按就绪逐项合入；不设全局等待。** 四线互不新增depends_on；799也不作为800的前置，800继续消费旧兼容AST路径。

若同时final ready，推荐短队列 **792 → 794 → 799 → 800**：优先收口已有发行修复，接回已在实施的scroll宿主，再落模型包。此队列是调度优先级，不是等待未完成计划的硬门；若799或800先完成可先合入，执行下列矩阵即可。

| 实际先后 | 后合入者必须做什么 |
|---|---|
| 792先，799/800后 | 新计划同步已交付792入口；metadata确定真实闭包变化，必要时同步source/resource清单、身份与锁；重放受影响fresh/locked/重定位及资源错误/旧产物保护案例。无需重造792协议或扩大其平台范围 |
| 799/800先，792后 | 新包提交与来源/兼容表随合入交接；792基于最新master重新冻结闭包、生成清单/锁，再执行其真实构建与admission/失败保护验收；不能复用已漏包的旧清单证据 |
| 794先，799后 | 799复核新的scroll record/类型和宿主所用AST仍同一身份，定向复验受影响编解码/接线；不默认重跑全部双端画廊 |
| 799先，794后 | 794从兼容AST入口重编，保留八native/provider/锚点状态，重放受影响record/源码VM及真实UI完成控制 |
| 799与800任意顺序 | 旧ModulePath/PathPrefix路径与类型保持；后者重放resolver prefix/trait/错误阶段，必要时检查Cargo source唯一性，不将两份AST类型通过转换伪装兼容 |

每次合流：确认他线WIP/公共接口 → 同步master并处理精确冲突 → 对改变的代码/依赖/测试输入重做受影响review（即使原review已pass） → 新冻结提交跑该计划merge-owned门 → 合入、沉淀、交接。不得仅凭“文本冲突已解决”沿用旧pass。

不为解决源码冲突在别人的树修补，也不代替该线合入。792尚未交付的接口不写成已经存在；必需接口如在实施中不稳定，给出具名冲突，先推进本计划无关部分并缩短接线窗口，不默认阻断整项。

## 测试、保全与后续

work用定向验收及必要check，merge最终冻结门串行安排；tf仅主检出到期批量单实例。800落地触发%5交接，799的AST触面在merge跑t/tv/tt/tu，不能把它们提前分散到每条finding修复。

旧auto-ui/auto-man仓、legacy refs、全部恢复载荷及原WIP不在新计划写面。工作树无链接，移除前wt-guard。此批仍是仓内边界建设；完整Parser/type-store/inference/UI文法外置、Man执行型配置/构建adapter/tool协议和正式G/H搬仓仍分别制定验收，不因AST模型或resolver核心可用而取消。
