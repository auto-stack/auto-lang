---
plan_id: PLAN-979
status: drafting
feature_name: Schema领域语言包与UI AGE投影接口
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-12
plan_revision: 9
current_step: 0
total_steps: 12
depends_on: [PLAN-960, PLAN-963, PLAN-966]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md, docs/specs/auto-a2x/architecture.md]
new_spec_components: [docs/specs/ace-migration/m2-schema.md]
touched_goals: [GOAL-002]
---

# [PLAN-979] Schema领域语言包与UI AGE投影接口

## 0. 变更摘要

2026-11-12 r9：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-11 r8当前：增加H0最小widget/text受检领域前门切片；以现有普通函数/文本和grammar/CST/verify基础接Hello，不等待963泛型/966完整CT整包；原通用Schema完整构造、最小双方言双后端及全部8项任务保留。 原8项任务/全部AC/V保留，新增4项早期切片任务，现12项、drafting、0完成；只规划。

历史修订：2026-11-10 r7执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r6当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

将现有Schema规则机提升为版本化领域包，具备依赖/上下文关键词/歧义与冲突检查、CST来源、typed领域IR及受检行为投影；UI/AGE语法留UI owner，Core无反向UI依赖。codec schema不等于grammar schema。

历史修订：2026-11-09 r5：共同Schema接面必须承载传统AutoUI和完整AGE作者语言，不能以最小UI/2D/3D子集替代。979仍交通用平台与最小双后端证明，980负责传统UI全模式，981新增LANGUAGE批负责完整Node语法/语义与组合；本包不反向等待两个产品终验。8任务保留，新增AC-08/V-11，drafting、0/8，仅规划。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
作者唯一维护的Node模块工程同时表达场景和逻辑；内部CST/领域IR/HIR/MIR及资源清单属于正常编译产物，不再先生成旧world.at和.agscene后交旧Parser/AutoVM重解析。AV和A2R不得分别维护一套Node语义。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。
本次明确两条必验出口：Checked HIR→共享Typed MIR→AV；Checked HIR→现有a2x-core/a2x-rust provider→Rust文件集→真实rustc编译运行。场景/绑定/资源需求的typed领域IR供AGE宿主消费，不将renderer、输入循环或Node文法塞入AV执行器。AC/A2C等原有适用义务保留，不能用本次AV/A2R两腿替代原支持格。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

2026-11-11本轮授权：用户要求规划首个ACE UI Hello及AV/Rust FFI路线、核对既有M2冲突并作必要重排；仅规划/设计/导航，不实施、创建worktree或探针/fixture/SDK、构建/运行、提交或发布，未指定模型/工期。沿旧语言用例守恒、新Auto标准库与AutoABI/FFI、Demo递进的三线，不削完整M2范围。

现场语言HEAD 9e36d984e、UI HEAD d88e9c51。已核av-core/src/host.rs仅console/1（sha256前16 30d6dcc475850cd0）、a2x-rust/src/provider.rs仍拒source foreign（3ea453dc2452576f）、core/src/source/cst/frontdoor.rs（dd3d93d06799ae52）；现有auto-bindgen/src/extractor.rs是预置C manifest表（8c7f1295cc83c6fb），不能称Rust元数据导入器已实现。Iced由Rust宿主编译执行，AutoVM不解释其Rust源码；现有DynamicComponent/VmBridge/handler_codegen深依赖旧执行链。

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
用户2026-11-09先授权r3修订979/981的Node直接接ACE、AV开发、A2R发行与两游戏整链验收；随后提出合并AGE原M2/M3、ACE接入与2D/3D增强并行。r4同步两阶段规划，恢复相关M2文档修订，仍不授权实施/先证/素材/构建/运行或创建worktree，不改变AGE M1签章口径。
同日用户进一步要求M2包含ACE对传统AutoUI和完整AGE语法的支持。r5明确平台/产品职责和完整性分母，仍只规划，不重开历史有限profile或另立重复语言前端。
2026-11-06原调研读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。该轮现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。该轮引用会话读取及936/944状态核验保留为历史，本次基线见下。

实际入口/文件：
- `experimental/ace/crates/syntax-grammar`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/syntax-runtime`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/grammars/auto-source.at`（本轮确认存在；开工重核符号/版本）。
- `docs/design/39-ac-incremental-syntax-and-language-platform.md`（本轮确认存在；开工重核符号/版本）。
- `docs/specs/ace-migration/{m2-modules,m2-numbers,replacement-scope-and-evidence}.md`、`docs/specs/auto-a2x/architecture.md`、`docs/specs/auto-a2r/project.md`为本轮现状依据；语言HEAD `b4f1af0be67b91b601345a4284ecd4036cb1a91e`。960/961已归档，但模块/数值独立profile通过不等于Node+模块+浮点+资源组合已通过；960仍明确a2x全局cell/类型方法片段边界。
- `experimental/ace/crates/{core,source-query,lowering,av,av-core,a2x-core,a2x-rust,a2x-store}`及`a2x-worker/tests/conformance.rs`为已存在接面；新增领域包/投影API、最小双方言fixture位置由T-01在既有owner内冻结，非现成能力。
- 领域依据为`D:/autostack/auto-ui/docs/design/autographics/declarative-authoring-model.md`与`docs/specs/auto-graphics/ace-migration-feasibility.md`（后者在auto-ui仓）；UI067于2026-11-07冻结旧语言基线的no-go只说明当时作者前门缺失，不能当当前AV语义失败，也不能把调研另行重复立项。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：通用Schema领域包/typed投影。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [experimental/ace/crates/syntax-grammar](../../experimental/ace/crates/syntax-grammar)：领域规则组合/歧义；目录职责已核，确切符号/闭包开工再核

- [experimental/ace/crates/core/src/source/cst/project.rs](../../experimental/ace/crates/core/src/source/cst/project.rs)：普通与领域投影接缝；读取SHA256前16 `4b8bbdb63d41d9b5`

- [auto-ui/crates/auto-ui-dialect/src/ast.rs](../../../auto-ui/crates/auto-ui-dialect/src/ast.rs)：WidgetDecl与Node领域owner，Core不反依UI；读取SHA256前16 `1d10f5ff2566422e`

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08, T-H0-I, T-H0-V, T-H0-R, T-H0-D |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


将现有Schema规则机提升为版本化领域包，具备依赖/上下文关键词/歧义与冲突检查、CST来源、typed领域IR及受检行为投影；UI/AGE语法留UI owner，Core无反向UI依赖。codec schema不等于grammar schema。

支持集继承959分母与Design55的完整语言定义；959分母用于范围核对，不成为“等全生态迁完才做前门”的前置。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新领域接面/fixture为拟新增，必须扩现有query/verify和provider，不另造类型检查器或A2R发射器。

### 职责、最小交接与并行边界

| owner / 承载任务 | 本包交付 | 后继接收与边界 |
|---|---|---|
| auto-lang / T-01、T-02、T-08 | 通用版本化grammar包、CST/span、typed投影合同、共享绑定/实例/资源事实及增量身份；领域回调只产未受信候选，由唯一verify取得Checked HIR | 981/SOURCE消费；Core不依赖auto-ui crate，不运行旧Node源码生成器 |
| auto-ui领域owner / T-02、T-08的最小双方言fixture | 用同一领域包接口声明最小旧widget与AGE Node语法/typed投影；示例含普通Auto函数、use、独立model及msg/on | 是979直接源输入的验证消费者；981扩展同一接口至完整AGE场景，不能再建第二套Node解释器 |
| auto-lang / T-03 | 同一最小Node工程经共享verify，分别真实AV执行和A2R生成/编译运行；保留可消费的领域描述和原span | 只证通用接面与独立值/事件oracle，不承担真实GPU/游戏打包；这些归981 |
| 既有960/961/962/963/966/967/978 | 模块、数值、聚合、泛型、编译期、typed host及构建/SDK各归原计划 | 复用交付；不在979重复实现宿主/调度/构建系统；游戏实际组合缺口由981登记具名owner |

交接件须包含repo/commit、Plan revision、领域包/grammar版本、支持构造组合、投影及Checked消费API、source map、后端/runtime身份、正反例与资源边界。不得只交AST codec或手写HIR夹具；Node的model/msg/on行为也必须经同一verify，不能只检查场景字段。

顺序：T-01后，T-02的通用包/投影与T-08的相容/恢复用例可在不相交文件上并行；T-03等所需投影就绪后分AV、A2R两腿推进；T-04汇总所有work结果，之后T-05→T-06→T-07串行。960的适用接口可支持基础准备，泛型/CT组合须等963/966相应接口；整个979交付仍核frontmatter依赖并集。979不依赖980/981完工、AGE整阶段、985认证或987退役；979用自己的最小消费者闭合，981只按稳定合同准备，最终产品证据须消费已正式合入的979来源，不能拿分支tip当交付。

阶段接口边界：合并AGE M2的G2/G3图形服务可与本包并行，使用版本化typed领域描述/资源需求，新增能力声明不复制Node语义或旧生成器。具体增强验收归UI058/AG023及后续冻结能力合同；979不反向等这些计划。新增grammar/类型需求由领域owner交本包或实际语言owner确认，不能仅因“AX可转Rust”假定所有图形/运行接口已支持。AX为A2X目标族，本轮必验的编译发行目标仍是A2R，不自动扩成所有目标/平台。

### 完整作者语言接面与两类产品责任

领域包接口必须表达传统WidgetDecl全部字段/领域结构与普通Auto行为，以及完整AGE的node/use/model/msg/on、props/绑定/typed引用、多实例/多场景、五类view及嵌套、运行时if/编译期#if、Life.Start/Tick/Frame。资料取auto-ui `declarative-authoring-model.md`§1–2、`multi-presentation-selection.md`及现行WidgetDecl/组件/Store/主题/路由Specs；其中未定时序/组合策略由981/LANGUAGE的有界决定门关闭，不能永远留作“支持范围外”。不把尚未批准的无限未来DSL或新平台混入完整性分母。

T-01/T-08记录“构造/字段→来源及原span→通用投影/检查能力→980或981接收owner→产品验证格”的双向表；嵌套表达式/事件体经唯一verify，缺字段/丢投影必须拒绝，不能静默忽略。979只验证通用表达/来源/准入与已有最小CPU消费者，不等待980/PARITY或981/LANGUAGE产品PASS；完整传统UI和完整Node语法的实际AV/A2R/图形/平台格分别由980、981关闭，M2须同时具备，最小双方言绿不冒全语法已支持。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### Schema组合语言与依赖闭合

提供普通语言+领域规则组合相容表、包版本/冲突、语法恢复/畸形输入推进性及无损来源；不把完整CAP求解塞此包。其typed投影仅使用已交付语言接面，980/981最终业务组合。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；接口由T-01先冻结，T-08只依赖T-01及相应T-02投影接口，不等待T-03双后端验收；T-03可消费其已就绪fixture，T-04最终汇总，避免T-03/T-08互等。

### H0：最小UI领域包与文字表达式投影（当前早期入口）

T-H0-I最小前置为现有syntax-grammar/syntax-runtime、core CST/resolve/唯一verify、已交普通函数和text支持及960适用模块基础；不使用用户泛型/CT/#if，不等963/966整包。Schema规则由同一既有grammar包机制声明，不把UI文法硬编码成第二parser，也不调用旧Parser/handler_codegen重解析。公共版本化包/投影机制归979，最小widget/text领域声明由auto-ui owner在同一接面注册；980只扩消费/宿主，不再重复前门。

H0支持普通源码`widget <任意合法名> { view { text <UTF-8表达式> {} } }`，根一个text；表达式首批为字面量或同工程普通Auto函数返回文本，行为经共享绑定/类型/资源verify。字面量也须保原span/包身份；函数体不能留作字符串由UI二次解释。属性/字段/未知结构按本切片能力明确拒绝，不静默忽略。static描述与运行表达式引用分开，描述不携带已受信Checked或Rust对象地址；CPU consumer可观察根/叶/值及原span，不硬编码App/Hello字节。

输出是领域包版本、CST/span、未受信候选→唯一verify的受检描述/表达式标识及资源要求。H0独立CPU投影/语义检查不等967 F0、980窗口或979完整Node双后端；UI真实AV/FFI/A2R由980/HELLO验。原T-02/03/08继续在同一机制扩完整UI/Node/模板/CT、最小双方言双后端与组合恢复；H0不提前关闭它们或缩完整语言分母。

现有owner路径`experimental/ace/crates/syntax-grammar`、`syntax-runtime`、`core/src/source/cst/frontdoor.rs`、`source-query`均已核；新最小UI grammar资产/typed投影case由T-H0-I在这些owner与auto-ui领域注册位置建立，准确路径/符号/命令先冻结。980只能在H0-D正式来源后消费，H0用自己的CPU消费者闭合，不反等980完整DIALECT/PARITY或981。

早期切片是本Plan M2-A内部交接，不新增ACE里程碑或Plan终态；只在同一权威树按owner流程独立复核、正式落主并记source/接口/grammar/ABI/provider及真实结果后供下游消费。同Plan尚有原完整义务时不归档/删树/勾整包完成。frontmatter保最终依赖并集，早期切片只等本节最小接口；同一文件单写，不以分支reviewed/WIP当交付。最小切片通过不关闭原AC、完整Rust FFI/全UI/RG-A03/M2A-08；不新增对980/981/985整包或AGE联合门的反向依赖。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r9当前）

**具体输入与期望。** 原件[ag-examples/examples/g01-shapes/g01-main/src/main.at](../../../ag-examples/examples/g01-shapes/g01-main/src/main.at)（下列节选L7–L38；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
use resources: red_alpha, blue_alpha, green_alpha, yellow_opaque
use shapes: Disc

node Main {
    view-2d {
        Disc red_disc {
            x: -20.0
            y: 10.0
            diameter: 30.0
            tint: resources.red_alpha
        }
        shape2d blue_quad {
            kind: quad
            position: (0.0, 0.0)
            size: (30.0, 30.0)
            material: resources.blue_alpha
        }
        Disc green_disc {
            x: 25.0
            y: -15.0
            diameter: 24.0
            tint: resources.green_alpha
        }
        shape2d yellow_quad {
            kind: quad
            position: (-45.0, -25.0)
            size: (12.0, 12.0)
            material: resources.yellow_opaque
            visible: false
        }
    }
}
```

本包所需独立结果：g01原源四图形/visibility和来源可投影；CPU消费者核原坐标/颜色/order，不以图形字段可解析证明model/msg/on行为受检；H0文字另保。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 版本化grammar组合→无损CST/原span→领域未受信候选/行为引用→唯一verify→CPU/AV/A2R最小双方言消费者。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct DomainPackage { id: DomainIdentity, grammar: GrammarRef, projection: ProjectionContract, dependencies: DependencySet }
struct DomainCandidate { structure: DomainIr, behaviors: ExprRefs, source_map: SourceMap, requirements: RequirementSet }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [experimental/ace/crates/syntax-grammar](../../experimental/ace/crates/syntax-grammar) | 领域规则组合/歧义 |
| [experimental/ace/crates/core/src/source/cst/project.rs](../../experimental/ace/crates/core/src/source/cst/project.rs) | 普通与领域投影接缝 |
| [auto-ui/crates/auto-ui-dialect/src/ast.rs](../../../auto-ui/crates/auto-ui-dialect/src/ast.rs) | WidgetDecl与Node领域owner，Core不反依UI |

**状态、错误与资源。** grammar冲突、未知构造/字段、坏行为类型或篡改投影丢必需叶须具名拒并保原span；不静默丢字段。 grammar/domain/ABI来源改变使受影响缓存失效；候选不携Checked信任/指针，失败保旧；通用投影不串行反等969/978整包。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [ag-examples/examples/g01-shapes/g01-main/src/main.at](../../../ag-examples/examples/g01-shapes/g01-main/src/main.at) | 原Node/use/props/view源，不手写HIR充源 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `9b809a1c6d99985b` | 拟ace.m2.p979.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [auto-ui/examples/ui/002-counter/src/front/app.at](../../../auto-ui/examples/ui/002-counter/src/front/app.at) | widget/model/msg/on/主题完整需求，CPU接面不冒全产品 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `04937c35325faa74` | 拟ace.m2.p979.source02：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [docs/specs/auto-syntax/grammar-contract.md](../specs/auto-syntax/grammar-contract.md) | 既有grammar机制，不新建第二parser | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `2203a19aa79239fa` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 泛型/CT组合等963/966对应能力，T-02/08通用合同与T-03真实双端消费者区分；H0不代完整Node，980/981产品不反置前置。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-schema.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06/07/08 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-03 | modify | `docs/specs/auto-a2x/architecture.md`（语言owner）及auto-ui仓`docs/specs/auto-graphics/ace-migration-feasibility.md`（领域owner） | 在实际交付后记录领域受检投影、AV/A2R共源接面及UI067旧基线no-go的具体关闭项；未实现项仍保缺口 | 防止将规划或最小前门证明冒充完整游戏支持 | AC-06/07/08 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-schema.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-HELLO | add | docs/specs/ace-migration/m2-schema.md（沿原owner） | 未有早期交接→本切片实际已验接口/源/拒绝及剩余完整义务；交付后沉淀 | 最小Hello不冒全包完成 | AC-H0/AC-H0-REVIEW/AC-H0-LAND |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-schema.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 同一Node关键字在普通标识符与领域上下文正确区分 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 冲突包拒加载且旧程序保留 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | 未知字段/坏投影在执行前报原span | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 领域规则更新只失效实际依赖 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | 用UI最小widget和AGE Node双producer验证 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-09 | AC-07 | 正常多模块Node工程含两个同定义独立Counter：定向事件2:1、N个Life.Tick后各实例恰推进N次；普通helper参与计算，typed view描述与状态绑定可观察 | T-02/08建普通源码fixture，T-03分别经正式共享前门跑AV、A2R→真实rustc→运行，T-04记值/事件序与独立算术oracle；fixture名/命令在T-01冻结 | work；979仅最小领域/CPU消费者，不以emit成功/AV运行替A2R运行，不等待981游戏 |
| V-10 | AC-03/06/07 | 同类型错误事件参数、未声明或越权的实例引用、坏领域包与篡改投影事实在共享检查处拒绝，原文件span准确；AV/A2R均不执行坏程序；修复后原入口恢复正确结果 | T-02/08建正反例，T-03记录拒绝阶段与未dispatch证据；schema/ABI变化使旧受检产物失效，旧合法代保留 | work；禁止二次生成旧.at、legacy Parser/AutoVM/旧Trans fallback或手造Checked凭证 |
| V-11 | AC-08 | 传统UI字段与完整Node构造的投影清单可表达；具名坏字段/遗漏必需投影在共享前门报原span；删除一条必需字段/构造映射时完整性交接检查红，合法近邻仍接受 | T-08在现有grammar/CST/投影target建普通源正反例及覆盖核对，T-04核源→接口→980/981 owner；不等产品验收或手造HIR | work；979仅通用表达/检查/交接，完整运行责任不能缩为最小view样本 |

完整职责组拟定：`cargo t ace-syntax ace-source ace-semantics ace-artifacts ace-av ace-a2x`（已静态核test-groups.toml，未运行）。原`dialect`组是legacy入口，只有真实修改该兼容面才补跑，不代auto-ui领域验证。新包/用例由T-04注册进对应组；改共享lowering/runtime/provider时按真实触面并入`ace-lowering ace-runtime ace-native`等必要消费者组，不能只跑前端。A2R用新ACE `ace-a2x`，旧`cargo t a2r`不证明新链。外仓只跑真实受影响命名case，窗口/GPU单实例、全量AG留里程碑；979最小领域fixture不以980/981完整产品为前置。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 同源同时含普通函数、旧widget、新Node可分别准入；坏包/歧义规则不死循环；领域新增不能改变无关普通标识符语义；热换schema后旧证据失效。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### H0验证（永久case由T-H0-I创建，未执行）

| Case | AC | 区分性输入→实际入口→独立结果 | owner/边界 |
|---|---|---|---|
| V-H0-SOURCE | AC-H0 | 普通widget/text源、改widget名/中文文本，及普通函数返回文本的同类源→版本grammar/CST→共享verify→CPU受检描述；根/叶/表达式值与原span准确，换字符串后实际值变化 | work；不把手写AST/HIR或旧Parser输出充普通源 |
| V-H0-REJECT | AC-H0 | 缺闭合、未知字段/构造、错误文本类型、表达式名字错误报原文件；合法近邻接受，拒绝后修复同入口恢复 | work；bad projection删除必需叶/篡改事实应在verify/准入拒绝 |
| V-H0-DEPEND | AC-H0 | 同源不同grammar/domain版本不误复用；旧CST/受检身份失配拒绝，改文字/函数仅失效真实依赖；CPU消费者不等待UI窗口 | work；拟职责组cargo t ace-syntax ace-source ace-semantics ace-artifacts，实际改lowering/AV/A2X再补对应完整组 |


### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | g01原源四图形/visibility和来源可投影；CPU消费者核原坐标/颜色/order，不以图形字段可解析证明model/msg/on行为受检；H0文字另保。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | grammar冲突、未知构造/字段、坏行为类型或篡改投影丢必需叶须具名拒并保原span；不静默丢字段。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | grammar/domain/ABI来源改变使受影响缓存失效；候选不携Checked信任/指针，失败保旧；通用投影不串行反等969/978整包。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | Schema组合语言与依赖闭合的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-07 | work | 普通Node工程直接经共同前门/唯一verify，最小领域描述与行为真实到AV和A2R编译运行；值/事件/实例隔离一致且分别满足独立oracle；非法近邻报原span，无旧源码生成/重解析或后端语义分叉 | T-01/02/03/08实施，T-04核V-09/10及结构依赖闭包；981接收版本化接面，完整游戏另验 |
| AC-08 | work | 传统AutoUI与完整AGE语言分母有来源/投影/检查/接收owner的双向映射，通用接面不硬编码三种view或少量UI字段，遗漏/坏投影具名拒绝；完整产品支持交980/981逐项关闭 | T-01/02/08，T-04核V-11；不以最小双方言PASS冒完整语言PASS |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |
| AC-H0 | work | 普通widget/text及普通函数文本表达式通过唯一前门，受检领域描述/原span/错误与恢复正确 | T-H0-I/V；V-H0-* |
| AC-H0-REVIEW | review | 独立核H0前门/行为verify/拒绝/版本和CPU consumer，无旧解析或硬编码样本 | T-H0-R |
| AC-H0-LAND | merge | H0接口/grammar/来源与真实验证正式落主，保原完整Schema/UI/Node义务 | T-H0-D |

## 8. 执行步骤

当前共12任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

早期任务可按§5最小前置先行，不要求T-01完整分母冻结已完成；各I自行冻结本切片输入/接口/命令，不提前勾T-01或其它原整项。以下行序不是额外前置。

- [ ] **T-H0-I｜M2-A｜owner_stage: work｜AC-H0**：不等T-01完整分母/963/966整包；核§5最小基础，冻结最小版本grammar/typed投影接口、CPU oracle和诊断/资源策略，补V-H0-*，沿同一机制实现。

- [ ] **T-H0-V｜M2-A｜owner_stage: work｜AC-H0**：T-H0-I后实际运行V-H0-*与完整受影响职责组，核普通源/表达式/原span及错投影拒绝；GUI与FFI未执行保持不在本切片声明内。

- [ ] **T-H0-R｜M2-A｜owner_stage: review｜AC-H0-REVIEW**：T-H0-V齐后同树停写独立复核源码→CST→唯一verify→CPU描述，不信样本自报；finding回I。

- [ ] **T-H0-D｜M2-A｜owner_stage: merge｜AC-H0-LAND**：T-H0-R pass后phase fold正式合入H0 grammar/投影与已验规范，交980/HELLO；保同树/resync继续完整原任务，不能称979或完整UI已交。



- [ ] **T-01｜M2-A｜owner_stage: work｜AC-01/07/08/AC-PHASE**：核依赖与§4实际路径，冻结§5交接件、传统UI/完整AGE分母的投影/检查/owner表、最小双方言/双后端源与V-09/10/11命令；有界前探共享语义→AV/A2R最长路径，明确资源和未决项，不缩范围。

- [ ] **T-02｜M2-A｜owner_stage: work｜AC-02/07**：实现通用领域包/CST/typed投影与唯一verify接面；领域owner通过同一接口接最小widget/Node源码及永久正反例，保原span，不另造语义分析或旧源码生成链。

- [ ] **T-03｜M2-A｜owner_stage: work｜AC-02/03/07**：消费T-02/08已就绪接面，AV与新A2R两腿可并行；后者必须真实rustc编译运行；核领域描述、状态/事件、装配/权限/增量/失败恢复，交981版本化接口，不以模型替运行。

- [ ] **T-08｜M2-A｜owner_stage: work｜AC-06/07/08**：提供普通语言+领域组合相容、包冲突/版本、畸形推进性/无损来源及完整构造映射，创建V-08/09/10/11；不塞完整CAP求解。投影用正式接面，传统UI/完整Node产品运行归980/981；T-04统一汇总，不反向等待产品终验。

- [ ] **T-04｜M2-A｜owner_stage: work｜AC-01..03/06/07/08**：汇总T-02/03/08，运行§6全部V行及按修改面冻结的职责/必要消费者并集，记录独立oracle、来源、子终态与条件排除；缺最小双后端或完整分母交接证据不得pass。

- [ ] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [ ] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [ ] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-979/auto-lang`，分支 `plan-979-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-08依赖T-01和相应T-02接口；T-03按已就绪T-02/08接口推进，不要求互等最终收据；T-04等待T-02/03/08全部完成；T-05→06→07串行。跨计划并行限不相交文件；共享core/schema/verify由语言owner单写，产品消费准备不反向成为979完成门。
材料唯一目录 `D:/autostack/plan-artifacts/plan-979/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-12 stage:new / PLAN-979 r9：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原12任务/全部AC与V/依赖/阶段/预算和drafting 0/12；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-11 / stage:new / PLAN-979:r8 / outcome:pass（早期Hello路线合同可审阅）；blocked（尚未授权实施，真实能力/接口待交） / next:按§5最小前置进入新增切片；原完整M2任务与终验均保留，不用小例替全包。新增T-H0-I, T-H0-V, T-H0-R, T-H0-D，当前0/12。未实现/构建/运行或派agent。

2026-11-11 规划静态核验（非产品/独立实现review）：12份本轮文档、6份Plan共106任务；对照本轮前快照，原86任务ID及原AC/V全部保留，新增20任务、全为未执行，drafting/current_step=0。0–10章节、total_steps、阶段清单与AC任务映射一致；323处本地链接已检查，新增链接/4处新锚点无失效、无新增行尾空白。跨四仓可达207份Plan的显式依赖无缺失/环，8个早期交接节点无环、AV不反等A2R或完整B；12个拟职责组名存在。6个具名源码SHA与本轮读取值相同；语言/UI/内容计数器仍1006/081/032。未新取号、建树、改产品/测试资产或canonical Specs，未构建/运行/签阶段/merge/tag。此检查不证明拟议接口已经实现。

2026-11-10 stage=new / r7：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部8个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r6：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/8；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-09 r5 / stage:new：补传统UI和完整AGE分母接面、AC-08/V-11；保8任务和原AC/V，drafting、0/8。979不反向等980/981产品PASS，完整语言不能以最小双方言替代。仅规划，实施handoff仍blocked（未获实施指令、依赖接口待齐），非运行或独立final PASS。

本轮静态检查：8任务/8AC/9验证行、原ID全保留，状态/完成数未变；979/980/981及UI058/AG023可达199份跨仓Plan，无缺失引用/依赖环，新增链接和Cargo组名已核。此为规划检查，不是任何新语言/产品运行PASS。

2026-11-09 r4 / stage:new：沿用户新两阶段方向同步并行/交接边界，原8任务/7AC/全部验证保留，0/8、drafting；不新增Plan，不把增强终验反置为979前置。本次只修订规划，实施handoff仍blocked（依赖/实施指令未齐），非运行或独立final PASS。

2026-11-09 r3：stage=new；979/981职责和前向依赖已静态复核，新增AC-07、V-09/10，扩充T-01/02/03/04/08和SD-03；保留原8任务及AC-01..06，0/8、drafting。outcome=blocked（仅指实施交接：当前为规划授权，依赖与双后端实际接面未就绪）；next=取得实施指令后按T-01及§5最小前置进入work。此次非产品测试/独立final review PASS，未创建实现树或执行构建。

规划静态核验：两计划跨语言/UI/内容仓可达192份Plan，按本地/带仓前缀plan_id及两种archive路径解析，0缺失引用、0依赖环；979任务8/验收7/验证行8，981任务28/验收14/验证行14，ID无重复且完成数均0。批次DAG亦无环，纠正T-08旧文字对T-03的反向等待；固定Cargo组名存在。此核验不证明拟议接口已实现或产品已通过。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本轮已确定不新增Plan：通用领域接面留979，AGE实际AV/宿主/A2R和两个游戏交981；T-01只冻结符号/命令/支持组合等实施细节。未决Node状态表示、类型方法/全局cell在新后端的具体合法化由981/SOURCE、981/RUST联语言owner核现状处理，不以归档960/961当组合已支持，不重开已交付工作。
