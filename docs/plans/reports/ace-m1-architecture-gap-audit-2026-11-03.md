# ACE M1架构机制剩余缺口与统一语法拆包调查（2026-11-03）

> **2026-11-05 CA-E／ARCH正式立项（仅规划，旧快照保留为历史）**：[941最终组合验收](../941-ace-m1-expanded-and-architecture-acceptance.md)drafting r1、0/15，新增独立target/机制registry与acceptance/3父侧总门，旧313 required/批准hash/NA/accepted/future及source/expected不改；Q4各对独立oracle、完整Rust/C认证、真实函数级P0/IO0、无预热恢复首编辑/首坏修复、原资源/样本/P90与同PID HMR均有独立反控。只读基面5f6737467：929 D29a7d7007与937 Dfa35943ac已归档交付；934已reviewed待D，935/936 executing1步，938/939仍drafting。941私有登记/reader/oracle可先，最终联合取D934/935/936/938/939；公共S/N/X保持原owner，H全文件与正式采样分窗，不把reviewed/WIP当D。另静态核出原313能力token无非绿，但38旧A2R cert-gap仍被924现行守卫阻签，按CE-BASE-01显式保留、不得以新HIR证书替旧域或默撤守卫；验收系统交付不等M1签章。详[基线/38项/写面排期](ace-ca-e-arch-baseline-and-schedule-2026-11-05.md)。M2全部旧场景接管/disable仍是v0.6硬门，本轮不施工。

> **2026-11-05 CA-D正式立项（本条为当前快照，下列早期状态保留为历史）**：有限ABI/FFI桥拆[937 CA-D1共享事实/标量底座](../937-shared-foreign-interface-and-scalar-abi-bridge.md)→[938 CA-D2受检源码foreign标量](../938-checked-source-foreign-scalar-calls.md)→[939 CA-D3资源/句柄/同步回调](../939-foreign-resource-handle-and-callback-bridge.md)，均drafting r1、0/12、0/13、0/14，仅规划。937带797 native与新A2C真实consumer/source导出、独立C认证，保持不改core/source/shared lowering/a2x-session恢复，可与929/934主体并行；H整文件与正式采样仍需短窗。938取D937及929稳定source接口，934若已接管则消费其facts权威，939取D938及已交925/924资源底座；不无条件等935/936整波，也不绕开实际读到的实例/CT依赖。完整CA-D须三包齐，最终CA-E/ARCH组合签章仍未立项，不预占号。当前main122725e9a：929executing r2 10/14由934获授权收尾；934 header drafting1/13但已有执行授权，935/936 drafting0步；928/931/932/933已交付。未跑实现/测试，不改在途合同/旧expected/分母/NA；797仍candidate/descriptor3/原预算，v0.6 M1含C与M2替换旧引擎disable硬门保持。详[基线/拆包/排期](ace-ca-d-baseline-and-schedule-2026-11-05.md)。

> **2026-11-04后继规划更新**：本报告原“SQ/实例/CT未立项/或1–2包”为当时快照，现已正式拆[934普通源码query](../934-source-semantic-query-and-invalidation.md)、[935有限真实实例](../935-source-scalar-template-instances.md)、[936普通源码AV CT](../936-source-av-cte-query-and-frozen-recovery.md)，均drafting r1、未实施。详[新基线与写窗](ace-source-sq-instance-cte-baseline-and-schedule-2026-11-04.md)。CA-D有限ABI与最终ARCH组合准入仍未立项；计划成立不等于机制已交付。

> **927–929规划复核更新**：三包当前drafting r2，0/12、0/12、0/14，新增入口授权/typed资源态、同文闭包与底层子体复用、无预热首编辑/真实计数与I/O、原子来源发布/预算适用性和具体写面交接；[完整复核及近期执行表](ace-927-929-plan-review-and-schedule-2026-11-03.md)为当前状态入口。下文初订快照保留，非实施通过证据。

状态：只读源码/Spec/收据调查与规划，不是本轮测试认证。基面 `ec3e70414`；用户授权核清M1机制并正式立项统一源码语法前端，不授权施工。适用[最新M1裁定](ace-m1-architecture-completeness-decision-2026-11-03.md)，不改原required/NA/accepted-diff、expected或已有批准。

## 1. 调查结论与分类

M1的主体执行与资源机制不是从零开始；最大确定缺口是**多profile源码语法事实与各类冷/暖/恢复入口尚未统一**。此外，796语义查询/实例模型与817真实CTE的有限交付不能证明普通源码前端已经全面使用同一语义查询/实例生命周期；797 ABI探针不等于A2X有限桥已接入。新增SYN/ARCH门也尚无组合签章实现。不能只数旧native三格或C包来估算新M1。

本表使用：`已交付机制/待迁移重验`、`真实接线缺口`、`在途能力`、`待联验`。后两类不冒PASS；“未找到所需生产接线”不表示原有独立机制没有价值。未实测不虚报现状性能或clean百分比。

| 门 | 已核基线与实际生产/消费 | 剩余缺口、owner与是否立项 |
|---|---|---|
| ARCH-01 语法 | 787通用scanner/CST/codec/恢复，828 scalar AV真实 `session.rs::incremental_reparse/project_decl_strict`；core `source/parser.rs`仍含独立完整/函数/lambda parser；仅auto-core/auto-scalar/interface-mini语法资产 | **真实机制与接线缺口**。七基础profile→collections/closures→全消费者/恢复/退役，正式分配927/928/929。最终三包共同关闭SYN-01..06；任何单包不签M1。 |
| ARCH-02 语义/查询 | core `source/{resolve,typecheck}.rs`、`CheckedEnv`与body事实，912/913单函数/闭包上下文；`semantics/{query,infer,reflect}.rs`及semantics-store持久化/依赖页；AV `semrefs.rs`消费语义引用合同 | **部分机制交付，普通前端语义查询接线与实例机制证据不足**。796 `inference-profile.md`明确泛型/CT为模板/模型，不是用户实例化；core当前无semantics查询依赖，不能由AV语义引用推断已统一推导。后继职责 `M1-SQ/INST` 待立项：先冻结有限真实声明/类型/实例/meta依赖闭环与代表例，不要求全泛型语法；927–929只保留事实身份/失效接缝，不代实现求解器。 |
| ARCH-03 受检IR/四端 | shared lowering、Checked MIR/ABC/AV、native；892/902/904 Rust HIR目标，921 C scalar已归档；typed entry891保独立位型通道，不冒宽数值源码 | **机制已有，适用能力在途与新增总验收未闭**：923故障/Place、924有限native闭包；925 C record/text、926 C collections。各包原owner不变，语法迁移后真实Q4认证由929与最终总签章消费，不能使typed MIR fixture新增Auto语法。 |
| ARCH-04 真实AV CTE | `comptime/src/driver.rs::query_step`、`av_driver.rs`；`comptime-core/src/consumer.rs::{substitute_and_infer,backfill_and_verify}`确为真实AV→冻结→推导→HIR重验→native；817支持helper/meta负依赖、snapshot车道 | **不是缺真实执行器**；普通源码前端通用查询调度/CT结果进入统一语义事实的接线尚无完整入口证据，当前主要由deck/专用driver编排。后继 `M1-SQ/CT` 待立项，复用817而非另做host执行器；与SQ/INST是否合包先核依赖与工作量。源码CT语法覆盖可有限，机制不得只留模型。 |
| ARCH-05 cache/资源工作集 | artifacts ManagedStore的内容身份/准入/required边/root/lease/GC，AV/native/A2X/CTE已有消费者，913有源码来源恢复 | **共享机制已交付，统一前端接线后待重验**。927/928提供语法与投影版本身份；929核源码、语法、接口、artifact恢复链、预算与未改生产0。SQ/CT后继须补其语义依赖键；D921纯恢复后首次编辑的`ensure_env`整源重建（记M/env_rebuilds）不满足新门，925/926若先补则929消费其交付并重验，否则由929补接缝；最终组合收据待总验收owner，不重建store。 |
| ARCH-06 资源/ABI | 881/894/864/865/909/912既有模式、ownership/捕获事实；915/919资源故障/事件；797 `ace-abi`候选profile有真实C互调，但声明源为受限IR | **资源扩面在途，ABI产品接线缺口**：923/924/925/926各自原域；`CA-D有限ABI桥` 仍未立项，须从统一接口事实连接实际A2X/执行消费者。不是全header importer/Rust shim生态，也不能拿各目标内部ABI当公共AutoABI。927–929不改这些语义/布局。 |
| ARCH-07 在线更新 | 866 `resident-online-reload.md`真实常驻guest链、安全点换版、旧调用链保活/回收；后续函数凭证及closures恢复各有消费者 | **机制已交付，语法迁移后待联验**。929复验同PID/入口次数1/新旧版本观察、取消与旧代回收；不能把restart来源恢复当在线HMR。当前没有依据另造完整AJ作为M1前置。 |
| ARCH-08 证据/父侧准入 | 908 strict适用性；911诊断事实；912/913真实生产/恢复；893成本归因；AutoDown/TestCase与固定职责组；旧required313及C新认证分别存在 | **新增组合门未实现/未立项**。929交付语法门及真实父侧收据，不能冒认SQ/CT/ABI全闭。`M1架构总验收/CA-E组合` 待立项，需版本化合并原门+C门+SYN/ARCH门、具名owner/证据SHA/STALE判定；只更新Markdown不算父侧已执行。 |

因此，统一语法三包之外，尚有**语义查询/真实实例/CT接线、有限ABI桥、最终组合验收**三类具名职责未立项；第一类应先做有限闭环与拆包决策，不能据本轮调查承诺固定计划总数。现有模块级机制有效证据可复用，不重复实现。

## 2. 为什么拆三个计划

| 包 | 正式计划/可独立验收产物 | 不承诺的范围 |
|---|---|---|
| SG-A 基础 | [927](../927-unified-source-syntax-foundation.md)：消除依赖环风险，配置驱动语法/通用scanner表达能力、严格AST投影；Core/Scalar/Text/Record/TextRecord/EntryUnit/HostConsole七profile与完整诊断/资源合同，基础冷入口切换 | 不以七profile交付冒全部语法统一；复杂profile和各会话全链恢复留具名后继。 |
| SG-B 复合 | [928](../928-unified-source-collections-closures-syntax.md)：List/enum/is/for-in、函数类型/闭包/间接调用、lambda/owner单元语法更新与保留子体；九profile覆盖完整 | 不改变捕获/资源分析；不以单独AST等价冒全部driver/持久化入口已切换。 |
| SG-C 接管 | [929](../929-unified-source-entry-incremental-retirement.md)：各真实冷/热/失败/恢复入口统一、版本化来源与未改函数生产0、Q4适用认证、手写重复生产parser退役、SYN父侧准入 | 不替CA-D、语义查询/实例/CT后继或最终ARCH总签章；不替换旧产品AutoVM/A2X。 |

将三者合一会把通用语法运行时扩展、复杂嵌套单元身份与所有消费者恢复生命周期绑成一次交付；按backend拆则会再次复制语法。三包按语法机制与消费职责拆分，顺序927→928→929，后继私有准备可提前，不并写公共S。

## 3. 架构选择与真实依赖方向

现状 `ace-syntax-core → ace-core`（投影使用core AST/Span），因此**不能直接让ace-core再依赖ace-syntax-core**。默认设计为：

```text
ace-syntax-runtime ← ace-syntax-grammar（auto-atom/auto-val为中立数据依赖）
          ↑                   ↑
          └──── ace-core：source AST/SourceProfile/诊断 + Auto专属CST投影
                              ↑
                ace-syntax-core：兼容适配/测试与产物接缝
                              ↑
                      AV/native/A2X消费者
```

通用runtime/grammar不得反向依赖core/语义/VM；Auto专属投影从syntax-core迁入core新模块（拟`source/cst.rs`），保留公共类型/API的薄重导出与类型身份。也允许经927有界探针证明等价的中立model抽取，但必须在下游修改前记清完整依赖图，不能两边复制AST或引入环。`hir_domain`等语义映射仍归core，grammar只表达语法。

共用版本化语法源/组合规则+profile能力清单，不为每个backend复制keyword/precedence；不同profile历史合法/拒绝边界须显式保留。投影不是第二份源码parser，不得重新扫描源码并独立决定优先级。脱离上下文的片段请求必须验证范围、当前profile、边界与全部消耗，不能把未知语法状态默认当普通代码。

AST语法层保留源码序、模式拼写、名义type/enum、函数与lambda边界；resolve/typecheck/verify仍唯一决定名称、类型、捕获、ownership与Checked凭证。Atom HIR文本reader、Atom grammar配置parser、旧产品parser与不同领域语法不是重复Auto源码parser，本轮不误删。

## 4. 与923–926的交接

读取时923 executing r1、3/12，lang-923在途；925头drafting但current_step=2且lang-925存在，按实际在途对待；924/926 drafting r1、0/14。头部文字滞后不代改。923树HEAD为ca9bcafb9且有native collections/runtime WIP；925树HEAD为ec3e70414且有未跟踪私有C `rt_runtime.rs`/`seed.rs`，均只读核查，不消费WIP为交付。921/922已归档，旧路线中“921仍待合入”属历史快照。

| owner/写面 | 保持原职责 | 与927–929的窗口和释放条件 |
|---|---|---|
| 923 N | native fault/Place、runtime/observe/故障原子性 | 927/928私有语法不等待D923；不写N。公共source/AST/diagnostic/candidate签名变更前登记923全部consumer，保持资源语义与fault seam版本。929若迁其collections session，须消费稳定SHA及受影响原故障反例。 |
| 924 N+closures S | native闭包/K02/env生命周期、原资源事实 | 928与其core closures AST/候选/凭证/lowering消费重叠：先冻结语法/单元接口再串行取窗；不得两包同时改`closures_fn.rs`等同一符号。无D924整包前置，最终native闭包腿须待实际交付或明确记录未验，不能签适用腿通过。 |
| 925 P(A2X)+S/H | C record/text、方法/模式/资源、函数增量/来源恢复 | 927基础规则/私有case可并行；`source/parser.rs`、AST、A2xSession和manifest锁为冲突面。925先完成已有原语义职责，语法包保持兼容门面；双方按method/type-free/type片段/恢复API和真正SHA取短窗。929负责最终入口迁移，不把925未闭功能接走冒交付。 |
| 926 P(A2X)+S/H | C集合/变体/owned资源/认证 | 928可先研究规则；A2X store/candidate/type-enum fragment与恢复接线在929按稳定925/926接口取窗。926可继续沿当前前端实现其目标；不新增等待927/928整包的语义依赖。最终929跨C collections认证须实际可用并核证据身份。 |
| 公共S/H | core source/AST/候选/诊断、syntax-core重导出、AV/native/A2X session；driver/SDK/TestCase/workspace manifest-lock/fixed组/来源登记 | 逐具体文件/符号owner、plan revision、base+handoff SHA、schema、全部producer/consumer/construct/match、有效/STALE/未验及释放条件登记。H全文件单写；正式性能采样错峰，不持写窗等慢认证。 |

不存在“923–926整波完成才可研究语法”的新增依赖；也不存在“私有case先绿就可不交接公共consumer”的豁免。代码接口改变后按输入身份判断旧证据STALE，不能无条件拼收据。

## 5. 一手资料与规划核验

研究仅用于机制分工，不宣称ACE已达到它们全部能力，也不引入新parser依赖：

- [Rust编译器总览](https://rustc-dev-guide.rust-lang.org/overview.html)：lexer/parser/AST/后续IR与查询职责可分层；不推出“手写parser必然不成熟”。
- [Rust增量查询](https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation-in-detail.html)：缓存有效性取决于实际依赖，持久化与稳定身份也需单独处理；支持本轮把语法统一与语义依赖接线分列。
- [Tree-sitter增量编辑](https://tree-sitter.github.io/tree-sitter/using-parsers/3-advanced-parsing.html)：编辑坐标更新与旧树结构复用是明确机制；它不自动提供语义依赖、Checked凭证或函数后端零生产。

本轮仅文档结构/路径/任务-AC映射/计划号唯一性检查。未运行cargo构建、测试、性能、认证或fault探针；全部新测试均由各Plan的work任务使能。当前计划材料政策为开工后`D:/autostack/plan-artifacts/plan-NNN/`、merge摘要入计划并删scope，不建第二证据目录。

## 6. 调查输入身份

本表为本轮取材身份，开工时先比较漂移，不把在途分支或文件hash当交付收据。

| 输入 | SHA256 |
|---|---|
| `experimental/ace/crates/core/Cargo.toml` | `3d2dd07e155f957b725e8f8cfda9947974dc9c81efe1b70577348900812bbea2` |
| `experimental/ace/crates/core/src/source/parser.rs` | `b12bf97815150a2d3f5f20471c495d2ef75581c6a9010d073d4526a2c4889801` |
| `experimental/ace/crates/core/src/source/lexer.rs` | `2e7bce2f09cf2bb9d5d7cc8db8a773df2bf8f02f0d434ab42bcdfe1ace76daf0` |
| `experimental/ace/crates/core/src/source/ast.rs` | `18072ea2dd934fac16b2f1e8e42a2dba3b15e7ce08b89f6ef5f0b8ddfa6d3971` |
| `experimental/ace/crates/syntax-core/Cargo.toml` | `a01538c218704ff3a274e2a904fdf405266ea3b757fbd65dca828310388502a5` |
| `experimental/ace/crates/syntax-runtime/src/program.rs` | `84508b564e1e5b1b7addf65889c5e717930af660c8e40918802ba170d5e1540c` |
| `experimental/ace/crates/syntax-core/src/lib.rs` | `299253fe9f1d67714cd853316260929ca9b2f22c2782083c9d35111d34adc101` |
| `experimental/ace/grammars/auto-scalar.at` | `66200289b8b0650be9abbb60b994361f89e4b16bc3fa273b09bb38a811aceeab` |
| `experimental/ace/crates/av/src/session.rs` | `5f8a603fe642d80cc85c705dcbda3da0d49f43b3f2c469c400b8f61961844b8e` |
| `experimental/ace/crates/a2x-store/src/lib.rs` | `163343e5967febc842114805accd080e28caafe607b51252caa8bd6833267156` |
| `experimental/ace/crates/core/src/source/closures_fn.rs` | `dda3ad2cf0ba2a3b400c3a921efb3f87622419667a99cec732470736d0657c4c` |
| `scripts/test-groups.toml` | `385c55ee1459fbd6c2424008783ba7c4698fa1098b035bd654492d6bf6b33ea5` |
