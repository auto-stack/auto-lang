---
plan_id: PLAN-938
status: archived
feature_name: CA-D2 受检有限foreign标量调用与源码增量闭环
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-07
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-abi/project.md, docs/specs/auto-abi/conformance-cache.md, docs/specs/auto-a2x/architecture.md, docs/specs/auto-a2x/incremental-and-cache.md, docs/specs/auto-a2c/project.md, docs/specs/auto-ac/function-incremental-source.md, docs/specs/auto-test-case/architecture.md, docs/specs/auto-test-case/ace-ca-d-bridge-certification.md, docs/specs/auto-mir/core-i32.md, docs/specs/auto-mir/hir-lowering-boundary.md, docs/specs/auto-mir/function-incremental.md]
new_spec_components: [docs/specs/auto-abi/checked-source-foreign-calls.md]
touched_goals: [GOAL-002]
affects: [auto-abi, auto-ac, auto-av, auto-mir, auto-a2x, auto-a2c, auto-test-case]
depends_on: [PLAN-937, PLAN-929]
current_step: 13
total_steps: 13
---

# [PLAN-938] CA-D2 受检有限foreign标量调用与源码增量闭环

## 0. 变更摘要

在937真实共享facts/标量物理桥上，将显式、有限typed foreign声明注入普通Auto源编译环境，让真实源函数调用独立C provider。native经唯一共享MIR直接发机器调用，A2C从受检HIR发薄调用，不在外层wrapper绕过语言编译；消费929统一source/恢复接口。加全链函数级增量/恢复首编辑、依赖失效与独立认证。本包不扩任意use.c/header importer，也不接资源桥（939负责）。

## 1. 目标

首例普通源`fn main() int { return foreign_add(20,22) }`加独立typed声明/授权输入，依次走当前统一语法、绑定、类型/能力校验、Checked HIR、native或C发射，实际C callee得到42；callee body+1、调用点不改得到43。除成功外覆盖fallible status/out、拒绝准入、精确失效、失败候选不安装、无预热恢复首编辑和真实未改函数生产0。

同一中立facts服务两个实际consumer：native机器调用无需先转C，A2C保持HIR消费。自动头摄入/全语言外语语法不是前置，但手工typed声明必须有审定来源、精确人工契约与当前provider绑定，不冒自动推断安全。四端普通无foreign业务按独立oracle一致；新增foreign矩阵如实列native/A2C已支持、AV/A2R受控未支持，不以两端绿冒四端FFI或新增NA豁免旧required行。

## 2. 架构方案

typed declaration manifest（候选）+当前编译宿主显式权限/人工契约→937候选校验/physical plan→受检foreign环境→普通source resolver绑定→Checked HIR外部调用事实→共享MIR typed ForeignCall/native直接调用，或HIR/A2C调用。拟新core/src/foreign.rs、lowering/src/foreign.rs、native/src/foreign_lower.rs、a2x-c/src/foreign.rs；精确结构由T-02冻结，路径均为新增。不把console HostCall/兄弟ExternalFn改名冒通用FFI，不伪造本地函数空body，不复制CST/parser/ownership/typechecker。

source compiler核心只依赖中立contract，不能依赖abi/Cranelift/tool runner；binding/符号/真实object/library/caps/provider生命周期在host组合根。937为本包提供逻辑/物理计划/codec/原三腿；929提供单一source入口/单元目录/原恢复protocol；若934已落则foreign事实作为同一query体系的显式依赖消费者，不旁建另一solver。core→backend反依赖、A2X消费MIR、以旧AutoVM执行source均禁止。

## 3. 技术栈

core/{hir,verify,source/**}、lowering、av-core/{mir,verify}、native/{lower,emit,incremental,link_driver}、a2x-core/ a2x-c/ a2x-store会话、937 contract/abi bridge、artifacts、case-worker与语言父adapter。未来文件/enum/opcode/required页/recipe必须T-02冻结后全reader适配，不现在假定已有ForeignDef或ForeignCall。

## 4. 需求分析与背景调查

2026-11-05 执行授权：用户 `/goal 完成计划938` 授权在原合同内串行 work→review→有界修复→merge/归档清理，无需逐阶段再批准。基面 `a0750086c`，唯一实现树 `D:/autostack/.wt/lang-938/auto-lang` / `plan-938-dev`。

本轮仅规划，授权/当前SHA/实际源hash见[基线报告](reports/ace-ca-d-baseline-and-schedule-2026-11-05.md)。929 executing r2 10/14，934会话正获授权收尾929；934 header drafting1/13但已有执行授权；935/936 drafting0步。源facts由其owner持有，不能认为新包可以同时修改core/source。937与本包都尚未实现。现DefKind仅Function/Method/Intrinsic、ExternalFn只是保留兄弟签名、HostCall仅原console车道，这正是“不能在外wrapper偷调C”的缺口。

消费canonical Specs：auto-abi三件、auto-a2x/{architecture,checked-resource-facts,incremental-and-cache}、auto-ac/{function-incremental-source,native-source-diagnostics}、auto-mir/native-typed-entry、auto-test-case/architecture。891私有宽typed入口不是公开ABI，既有i32/bool source能力不扩大成宽源码；新source只i32/bool标量函数，宽类型继续937typed入口，unsafe raw声明不自动safe。

拟唯一树`D:/autostack/.wt/lang-938/auto-lang`、`plan-938-dev`；未来scope仅`D:/autostack/plan-artifacts/plan-938/`。scope登记/外部输出显式归属、merge摘要/全路径对账/删除及guard清树沿仓规则；本轮不建树/测试/agent/goal，也不改929/934..936进度或合同。

## 5. 详细设计

### 5.1 先冻结有限source表面

新manifest拟schema`ace.foreign.manual/1`，每个声明含稳定逻辑id、source可见名、名义版本/逻辑签名、provider域与符号、方向、profile/target/callconv、error/effect/cap、来源header+独立审定物理表、raw/manual约束。manifest作为单独输入注入普通编译环境，source仍用已有调用语法；解析manifest是typed数据codec，不是第二套Auto grammar或任意C头parser。名字按正常scope/冲突/遮蔽规则绑定DeclRef，不用扫描源码字符串替换call。

source标量只i32/bool（bool公开u8↔内部0/1）；纯显式人工审定无失败C形态与fallible ctx/status/out均可，但未知错误/ownership/权限只raw候选并拒source安全调用。manual契约是审核过的信任来源，不能只凭任意文件自称safe；当前宿主授权输入/来源id与provider内容绑定，持久safe标志不授信。任意外库真实签名无法从obj符号名自动证明，首批provider由真实独立header/物理表、同头编译和内容绑定构成可审定证据；未审核外库不进入本有限safe lane。

定义形态不能表达/未支持的extern/mode/类型/effect/ABI，在绑定或verify前门具名拒，禁止编译失败才判断unsupported。source cap必须显式开启；普通CT/无权限profile拒foreign，不能给936真实CT服务扩大IO/FFI权限。所有参数按Auto源序恰一次求值，非执行短路分支不调用；每次实际call记录callee counter/trace，signatures与逻辑bool不因载体相同混用。

### 5.2 最小全链可行性与信任

T-02以首例验证source→受检foreign decl→HIR→共享MIR→native真实C call及同源HIR→C call，并同时验证restart→第一编辑和失败首编辑→修复。明确foreign DefRef/credential、signature与目标plan页，严禁生成本地stub body绕verify。若只能跳过checker/制造MIR裸地址/把foreign call搬到main.c包装外壳，needs_replan。新MIR/schema与穷尽匹配至少覆盖AV/MIR verifier/native/A2X/codec；AV对新op无stub时在编译准入具名拒，不能执行NOP、错当console或把未执行标PASS。

候选声明→当前owner/来源/名义与shape/mode/权限校验→process-local verified接口→绑定实际provider/符号/对象→prepared selected-target产物→重验base/依赖/cap/cancel→唯一安装点。错误依次保候选source/diagnostic stage/code/span/message/顺序；raw binary只是符号有无，不能替代物理签名/人工契约。纯C函数不能unwind穿界；任意地址/内存故障不承诺可恢复。known ABI错配不实际调用，合法非零ctx/混合实参不得过度拒绝。

### 5.3 精确依赖及函数级增量

拆声明逻辑身份/签名内容版本、physical plan版本、provider实现/object内容及当前binding代际。普通非内联caller只读签名/mode/effects/caps，不因callee provider body改变重新parse/solve/lower/emit；link root/当前binding必须改用新实现。wrapper只绑定它实际读到的契约/布局，不把整个库hash塞每个函数key。完整绑定/证书闭包另含实际库内容、header/flags/工具、target/runtime/provider；共享header变化影响所有真实reader，未读者不得失效。新增同名decl/遮蔽/删除/解析位置是负依赖；不能以调用点source hash未变直接hit。

真实source/unit projection、bind/type facts、资源verify、HIR→MIR、native COFF、C fragment、接口提取/adapter/tool分别记录attempt/produced/reused/discarded与P/M/R/IO；无成功输出的失败attempt也计。未改函数P=0、其producer/payload额外IO=0/保留resident身份；合法声明变更只重检查真实reader。shared assembly/link本身所需工作单列，不能将全源重建重命名R/M。暖编辑支持source函数体/foreign声明签名与能力、库实现不改调用点、前缀变长漂移；不把typed bridge edit当source edit。

### 5.4 恢复/发布/权限

持久source/catalog页由929深验；foreign逻辑/physical/binding依赖页由本包required集合持有。934若已接入则其query页仍由934重验，本包注册依赖事实，不替其solver。restore R只解码/深验证已有页/名义与引用；不得parse/重新solve/lower/encode/调用工具。缺required页拒该P0 restore，显式cold另计，不能恢复成功后首编辑偷偷cold。SafeCallable/宿主地址/Checked/PASS不持久化，来源/当前权限/provider/runtime每次重新资格化。

测试必须无ensure_env/空编辑/预热，restart之后立即编辑读者或非读者；再覆盖恢复后第一编辑非法、保持旧root/运行结果→修复→等价fresh→再恢复/再编辑。source/hir/foreign接口/target artifacts当前组原子发布，不在metadata更新而native仍旧的混合状态回成功。旧provider在途/reader lease不因新binding安装提前清理；纯静态重链接与真实HMR分别记录，不称重启为在线换版。

### 5.5 预算与认证

ABI请求沿937/797原build60s/run5s/total70s、IPC4MiB、每流1MiB、hot8MiB+暂存索引8MiB、单产物64MiB/请求128MiB；原A2X48MiB合计resident/128MiB store与source/native原更严帽不放宽。并存候选/恢复暂存/索引/复制都计费，先reserve再分配，失败不中断必要cleanup。六场景×30原ABI样本、原source/native/A2X负载与独立认证规模全保，body-edit/descriptor-reject及适用暖门P90≤2000ms，父入口到typed判定落盘完整窗口。

source-import首例TestCase从T-03接入，native/C实际callee各独立编译，源码same而reference手写、期望只父侧。四端无foreign业务分别对独立oracle；不得因为AV/A2R尚未支持foreign就改旧Q4列或required/NA/accepted-diff/expected。新foreign报告必须显式方向/输入profile/编译器/ABI/来源/实际调用/异常/计数，daily纯emit/metadata无spawn，显式certify独立compilation/linked执行；原source→C export与797三腿回归，不只测试新import孤链。

### 5.6 写面与前置

| 面 | owner/依赖 | 本包规则/释放条件 |
|---|---|---|
| 937契约/ABI | D937真实交付后消费；private fixture/header可先读 | 不能从937未合WIP接口假设稳定；变公共model另协商ABI短窗 |
| S source/core/恢复 | D929稳定交付；如原owner明确释放一个已冻结接缝可先限定集成，但最终验收/合入须D929 | 929收尾优先。934若已接管core事实端口则其稳定SHA/reader清单先释放再接本包；不同时改其source-query |
| query/instance/CT | 934..936保持owner | 不等无关实例/CT整包；本包不新读模板/CT但必须跟踪现有checked事实依赖，若actual foreign签名依赖它们就具名加入该切片D门，不能绕开 |
| shared lowering/MIR/AV穷尽匹配 | 本包S短窗；原console及AV profile保行为 | 同一次版本迁移全部consumer/source restore reader核齐，AV unsupported显式前拒，无opcode错解 |
| N/C/X | native direct-call与C HIR emitter、本包a2x view/store函数键/恢复 | whole-file单写，与934公共session/source环境短窗交接；不让C backend依赖native/AV |
| H | workspace/lock/worker/父decoder/SDK/注册/fixture/M1reader | 与929/934..936逐文件释放/接受、完整producer-reader清单；不得append-only并写。正式采样单实例错峰 |
| 939 | 消费D938调用/资源钩子/required页/trace合同 | 标量整包交付前可独立准备资源oracle；不占S等D门、不要求全部SQ波完成 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-abi/checked-source-foreign-calls.md | 仅桥typed事实→显式manifest/受检source foreign调用、actual方向与拒绝矩阵 | 不冒任意importer | AC-01,AC-02,AC-03 |
| SD-02 | modify | docs/specs/auto-abi/conformance-cache.md | 逐source/声明/实现/binding分层依赖与required恢复 | 真实P0与失效 | AC-04,AC-05,AC-06 |
| SD-03 | modify | docs/specs/auto-a2x/architecture.md | sibling ExternalFn与foreign facts边界/中立HIR消费 | 分责不复制分析 | AC-01,AC-02 |
| SD-04 | modify | docs/specs/auto-a2x/incremental-and-cache.md | foreign reader/adapter target键/恢复首编辑/失败原子 | 来源与目标绑定 | AC-04,AC-05 |
| SD-05 | modify | docs/specs/auto-ac/function-incremental-source.md | shared MIR foreign调用/符号绑定/增量计数 | native不经转译 | AC-02,AC-04 |
| SD-06 | modify | docs/specs/auto-abi/project.md; docs/specs/auto-a2c/project.md | 新有限source import与typed边界、AV/Rust实际矩阵，不升公共ABI | 如实陈述支持面 | AC-01,AC-07 |
| SD-07 | modify | docs/specs/auto-test-case/ace-ca-d-bridge-certification.md; docs/specs/auto-test-case/architecture.md | 在937交付规范上扩本包独立recipe/严格身份与source增量恢复或资源方向 | 各包真实首例/认证不互冒 | AC-07 |
| SD-08 | modify | docs/specs/auto-mir/core-i32.md; docs/specs/auto-mir/hir-lowering-boundary.md; docs/specs/auto-mir/function-incremental.md | 新ForeignCall/AVFI与原AV执行准入隔离，单函数Lower/恢复重锚owner，不扩大旧私有typed ABI | 实际共享MIR读者与持久格式变化必须沉淀 | AC-02,AC-03,AC-04,AC-05 |

### 首例协议/文件集身份与完整测试注册

原正式父路径`crates/auto-lang-core/src/test_cases/abi.rs`、`mod.rs`与`crates/auto-test-case/src/contract.rs`实际只认原`abi/interop-v1`/`abiworker/1`，新链不能借该recipe冒已支持source。T-02冻结本包新recipe、输入profile/协议schema、source/typed来源、ABI domain/profile/target/工具完整身份、required页清单、实际方向与certificate精确字段集；首例接线任务同步迁移全部父/worker/SDK构造与reader，旧ABI/C exe/Rust证书互拒，unknown字段/错版/丢方向/空选择不PASS。可以复用正式runner，但不复用旧PASS判定。

C库导出采用正式新增library fileset/assembler而非认证时剪旧exe main.c；完整业务片段/runtime/公开header/adapter/manifest/必要CRT依赖都入身份，不能只编可通过的小段。source输入与typed adapter输入分别具名，后者不得核销普通source验收。存在`autotests=false`的包新增test须登记对应Cargo `[[test]]`，固定组必须实际选到完整包/新target并记录selected/executed/条件排除，不以写了测试文件即算覆盖；拟组在本包H任务正式注册前均未执行。

## 6. 测试设计

新增拟`core/tests/foreign_source.rs`、`native/tests/foreign_calls.rs`、`a2x-c/tests/foreign_calls.rs`、`abi/tests/source_foreign.rs`及`crates/auto-lang/test/cases/ace-ca-d-source-938/`，T-03/10使可运行；所有V当前未执行。实现/内部自查与独立上下文复审依次同一树，未指定额外角色，不自动派agent。

| Case ID | AC / invariant | setup / entry / work owner | observable oracle / distinguishing assertion | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01/02 真实语义驱动 | T-03..06，source main→manifest foreign_add→native/C独立callee | 20,22=42、provider+1=43；切断decl事实则编译拒绝/counter0；伪本地stub/外wrapper偷调反控失败 | 真source i32/bool，两端均需实际边 |
| V-02 | AC-02/03 源序/错误 | T-04..06，短路未走call、非零ctx/fallible、多次有序参数/副作用 | 每参数/调用恰一次且顺序等于独立trace；status非0不消费out、保原因；bool2拒、0/1合法 | 标量，非资源939 |
| V-03 | AC-01/03 准入 | T-03/06，raw无契约、错来源/符号/target/ABI/权限/服务/过期semref、重名遮蔽 | 真实入口stage/code/span可定位，callee counter0；合法人工审定控制可用；不得以链接报错替语义拒 | 符号存在≠物理签名已证明 |
| V-04 | AC-04 依赖/真实P0 | T-07，caller源不改，修改provider body/签名/cap/header搜索或无关func | body仅其实现/link变，caller parse/solve/lower/emit/adapter=0；签名真实reader重验、非读者payload额外IO0 | 不内联/未读取body，link装配另列 |
| V-05 | AC-05 恢复首编辑 | T-08，真实worker restart→立即source body/decl首编辑或首错→修复→再恢复 | required R深验、P0 restore；first edit未改P0、無预热；结果fresh等价、前缀漂移来源正确 | 不用937typed恢复替代 |
| V-06 | AC-03/05/06 坏页与发布 | T-08/09，错owner/缺页/伪Checked/旧schema/权限撤销、取消/迟到/发布故障 | 拒后旧source+foreign+target root不变，无错误call；repair可续；合法restore不过度拒 | 本包页与929/934页分owner |
| V-07 | AC-06 资源/周转 | T-09/11，原load/6×30/512修订、超限/timeout/GC/在途lease | 原限额、P90≤2000ms、无新cacheDB；元数据/R预留/清理与树回收真实 | ABI与source原更严门AND |
| V-08 | AC-07 首例/认证/支持矩阵 | T-03/10/11，文件化实际两端+手写reference/普通业务Q4、旧桥三腿 | 每actual/reference分别等于独立oracle；manifest/cert/recipe/输入profile互冒/空选择/丢调用/晚失败拒 | native/C foreign非四端foreign等价 |
| V-09 | AC-08 完整合同 | T-12，独立上下文逐AC/所有consumer读者/Spec delta/遗漏扫描 | 没有隐藏fallback/未批准延后或NA，审查绑定冻结SHA/r1 | 不签M1/939资源 |

固定组按真实diff冻结：`cargo t ace-source ace-lowering ace-native ace-av ace-a2x ace-abi ace-artifacts auto-test-case ace-adapters infra-fixed infra-ace-m1 case-a2c-hir-scalar-921`；实际新增包/C tests沿937拟组`ace-foreign-contract ace-a2c-abi`，本包拟新`case-ace-ca-d-source-938`由T-10注册。934事实端口若改加`ace-semantics`及其已落组；只消费不等于全consumer均需测试，但新HIR/MIR全穷尽匹配及真实native/C/AV拒绝兼容必须验。原专项条件/ignored/原规模不减，最终review后merge在冻结提交跑职责并集，身份输入不变的有效证据复用，不堆叠全experimental/all。

## 7. 验收标准

| ID | owner_stage | 通过条件 / closing evidence |
|---|---|---|
| AC-01 | work | V-01/03候选→受检source env→HIR真实消费，manifest审核/权限/来源可复核，unknown非safe，无另一parser/solver |
| AC-02 | work | V-01/02 native经共享MIR直接机器call、C消费HIR薄call，两端真callee结果/trace正确 |
| AC-03 | work | V-02/03/06调用前严格拒绝/源序恰一次/错误保真/out不滥用，AV/A2R未支持不执行fallback |
| AC-04 | work | V-04真实函数级P0/IO0、读依赖精确失效和provider body变化当前结果更新 |
| AC-05 | work | V-05/06真实源码无预热恢复首编辑/失败修复/再恢复，R无隐形生产、required页与发布原子完整 |
| AC-06 | work | V-06/07原预算/资源状态/lease/cancel/GC与P90及原正式样本全部兑现 |
| AC-07 | work | V-08首例独立材料/实际编译运行认证、原797三腿/937 source exporter回归、支持矩阵不造NA |
| AC-08 | review | V-09独立复审逐AC/差异/债/warnings/fmt/全部读者/SD核对，pass绑定r1与提交 |
| AC-09 | merge | 冻结职责并集/漂移复验、SD沉淀读回、交付归档、全输出scope清理/guard收据 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 修改路径/可验证成果；对应AC/验证 | 状态 |
|---|---|---|---|---|
| T-01 | work | 用户后续执行指令 | 重核D937/D929、934..936当前owner/事实端口、source全部consumer、scope；AC-01/05 | [x] D937/D929/D934已归档且代码在基面；935/936树clean，935只读冻结/936等待前置，无活动公共代码写入。934 `compile_scalar_product`/`TypeSupply`/`SignatureCatalog`为现接缝；恢复声明`core-decl-env/1`。scope已建并登记。 |
| T-02 | work | T-01，D937，S稳定窗口（最终D929） | §5.2最小真实source双端+restore首编辑/失败修复；冻结DefRef/MIR/schema/权限/页/预算；V-01/05，不成needs_replan；AC-01/02/05 | [x] 最小真实两端调用及原ManagedStore独立进程restart→首错不安装→修复实际43已证明；接口/required页身份与限额见§9 checkpoint 4。T-03..11完整矩阵/原子发布/预算认证仍开放，不由本勾替代。 |
| T-03 | work | T-02，H首例短窗 | core新foreign manifest/受检env/绑定与凭证；首例TestCase/独立C材料先入；V-01/03/08；AC-01/03/07 | [x] 中立manual准入、source/HIR凭证、当前宿主独立policy与source首例正式注册；run-r4y14802 11项（10正常PASS+1登记的wrong-oracle FAIL）包装gate通过。source-query 48/48与manual/contract反控见§9；完整T-10认证/SDK义务未由首例代替。 |
| T-04 | work | T-03，shared lowering/MIR窗口 | typed foreign调用/verify/codec，全AV穷尽匹配具名拒/旧console兼容；V-01/02；AC-02/03 | [x] 共享ForeignCall/AVFI verifier与所有enum reader已接；真实AV admission拒foreign.unsupported、Rust拒a2r.foreign-unsupported，原无foreign编码不升版；ABI定向矩阵与source/HIR编解码反控通过，原console完整职责回归待冻结并集。 |
| T-05 | work | T-04，N窗口 | native foreign_lower/emit/link真实直接C符号/错误lane，不经C转译；V-01/02/03；AC-02/03 | [x] native共享MIR→直接C符号机器call；i32/bool/5混合槽/ctx/status/out/源序/短路实测通过；真实provider COFF定义及text/data kind在parse/codegen/link前核验，错对象/ctx借函数符号反控通过。 |
| T-06 | work | T-03/04，C/X窗口 | a2x-core view/facts与a2x-c foreign HIR call/纯provider、host binding准入；V-01/02/03；AC-01/02/03 | [x] C经SemanticView/Checked HIR单体与独立ForeignTargetProvider发薄call、完整library fileset，未消费MIR；同源实际调用/错误矩阵通过；裸普通C入口拒foreign，当前authority逐声明对照。 |
| T-07 | work | T-05/06，934若已接管需稳定事实接口 | 分层keys/reader依赖/producer与IO计数/单函数暖编辑，V-04；AC-04 | [x]  当前35+1项closure重放与实际cap正反控制见§9 8bn；分层keys/真实reader/P0/IO0/共享host账本通过，未由此核销T-09预算或T-10完整错误/SDK报告。 |
| T-08 | work | T-07，D929及各已用事实required页 | 新foreign required页/旧协议reader、无预热source恢复首编辑/首错修复/再恢复/原子发布；V-05/06；AC-05 | [x]  当前真进程/required反控/原子发布完整重放见§9 8bn；首错修复/fresh/再R/前缀、current authority与旧root保持通过，资源/GC全部失败清理仍属T-09。 |
| T-09 | work | T-08 | 先预留预算/取消/超限/lease/512修订GC与全部失败清理，V-06/07；AC-06 | [x] 8b..8bx逐owner先准入+取消/超限/lease/512GC/失败清理均有永久正控与变异负控（见§9 8bx收口评估）；残余明示：parser scratch归原FrontBudget、WAL极端增长未单测、小工作元数据按wire量级、namespace按space()扫描；V-07的P90/6×30与AC-06关闭留T-11。 |
| T-10 | work | T-03..09，H窗口 | worker/父decoder/SDK/新固定组/首例完整元数据与独立认证、旧三腿/937回归；V-08；AC-07 | [x] 8bh/8bq认证与8by最终代码重放：正式门PASS、SDK report/2镜像互拒/1、legacy 94腿全绿（ace-abi 239/240）、a2x/native(226/226三段)/artifacts全绿、daily无spawn在册；6×30性能门归master基线同红（环境），处置留T-11。 |
| T-11 | work | T-01..10，正式采样窗 | 原sample/P90/完整职责自查、SD/939稳定调用与资源钩子交接；AC-01..07 | [x] 6×30/暖P90门绿（8bz 375ms+常驻门）；Q4矩阵diff审计无旧行变化+四端组全绿（AV 1030/1030修复8l/8f漏适配后、native/C/Rust 8by）；SD-01..08核对仍准确并列沉淀要点；939八族接缝确认稳定。 |
| T-12 | review | T-11 execution_done | 同树独立上下文逐V/AC/受检source事实驱动与所有reader/债/SD审查；AC-08 | [x] §9复审记录：AC-01..08逐条证据核验pass（绑定r1@34b2a8fb7）；扫描0 TODO/0生产调试打印/4处文档化expect；fmt全净/零新警告；spec-impact与SD一致；D-938-1..4入KNOWN-DEBT。 |
| T-13 | merge | T-12 pass，D929，后续合入指令 | 冻结组并集及触码漂移重验、SD读回/合入归档/所有输出清理与guard；AC-09 | [x] 冻结并集13组gate_passed=True（2328项0not-ok，2922s）；master自8l pin零ACE文件重叠=零漂移（合树=分支ACE树逐字）；SD-01..08沉淀11文件；merge 8893cfd53；归档+全路径清理+guard见§9收尾记录。 |

独立准备限T-01来源/私有oracle；主链接D937与source稳定窗口。不存在“934–936必须全部结束才能开938”的整波门；其已接管的具体source/query/required页须按实际D/接口接入。

## 9. 复审记录

2026-11-05 stage:work | PLAN-938 r1 | outcome:in_progress | worktree D:/autostack/.wt/lang-938/auto-lang | branch plan-938-dev | base/code a0750086c | task_ids T-01 complete, T-02 in progress | dependency_revisions D929/D934/D937 archived and present in master; auto-down 7c4d5cb | evidence: static code inspection only, V-01..09 not yet executed | next: work T-02 feasibility. 共享source reader清单：core resolve/adapter/verify/decl_env; source-query catalog/produce/apply/consumer/session/restore; lowering; av-core MIR/verify/codec; native emit/incremental/link; a2x-core SemanticView/store; a2x-c; av/a2x-rust显式拒绝; case-worker/abi-worker/语言父adapter/contract/SDK/固定组。主检出既有变更已排除：文档修改、015-notes pnpm lock、native-fixture Cargo.lock、reader-import-budget数据文件，本计划不修改/暂存它们。


2026-11-05：stage:new；PLAN-938 r1；只做静态规划，未执行任何V。source foreign以显式受检声明注入，固定两端真实调用与源码恢复责任；新schema/opcode/权限/资源支持由有界T-02证明；无未知source语法暗约或减分母。最终静态核查见CA-D报告，不冒implementation/review pass。

静态规划校验已完成：各节/稳定ID及交叉引用、当前固定组/新组标注、新链接、来源hash、编号唯一/next-id=940和文档范围/空白检查通过；全部实施与复审验收仍未执行。

2026-11-05 T-02 进行中 checkpoint（不是可行性通过/不是 AC 关闭）：实现提交 `cab2cc0502e312622f3851702b074cf834ef417f`，新增 `foreign-contract/src/manual.rs`：严格 `ace.foreign.manual/1`、显式 i32/bool 单结果、有限 pure/fallible 形态、独立 HostAuthority（review/contract/header/physical-table/provider/target/caps/symbol）与非序列化 AdmittedForeign，契约身份与 provider implementation/generation 分离。真实 source/HIR/MIR/native/C/恢复链尚未接入；该模块当前仅是 T-02 私有可行性实现基础，不能冒整链交付。

验证：`cargo test --manifest-path experimental/ace/Cargo.toml -p ace-foreign-contract --lib manual::tests` exit0，5/5，0 ignored；准备冷编译22.48s，运行0.01s。`cargo fmt --manifest-path experimental/ace/Cargo.toml -p ace-foreign-contract` exit0；`cargo t ace-foreign-contract` exit0，selected=30/executed=30/条件排除=0（27 lib+3 conformance），真实 nextest wrapper gate_passed=true；本轮无真实 C 工具运行/无性能采样。固定入口工作记录在该树 `target/test-groups/run-rh7zvfse/`（树内正常验证输出，随树全删，不另保副本），raw结果hash见scope的checkpoint；未执行V仍全部pending。`git diff --cached --check` exit0，提交后实现树clean。

下一执行入口：继续T-02（不能直接核销T-03）。源码主门必须消费934 query facts：现 `SignatureCatalog::build` 与 solver `SemaInput` 仅本地函数签名，需添加真正外部声明输入，禁止伪本地空body；core `TypeSupply`/resolver/adapter、`DefKind` 与 verify 的受检外部声明需同步接入，再到共享 MIR/native/C。先原例20+22=42/body+1=43及恢复首编辑/首错修复证明，未成不得T-02打勾。已读 verification-contract.md；源码reader列表见T-01回执；935/936基线clean且未修改其文件/进度。

stage: goal | plan_id: PLAN-938 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: in_progress (no acceptance pass) | worktree: D:/autostack/.wt/lang-938/auto-lang | commit: cab2cc0502e312622f3851702b074cf834ef417f | next: work T-02

2026-11-05 T-02 checkpoint 2（仍未通过T-02，代码尚待本轮完整职责组完成后提交）：已接普通源码→934共享solver外部签名输入（`ExternalSignature`，不伪造`InputFn`空body）→`SignatureCatalog`逐声明版本/实际reader失效→`BodyTypeSupply`当前admission→core resolver→显式`DefKind::Foreign`/scalar descriptor `foreign`节点→`verify_foreign_scalar`。普通/CT验证入口通过`foreign_allowed=false`前门拒绝Foreign；新HIR工件版本2仅含foreign的bundle使用，旧无foreign bundle仍为版本1，解码不赋信任，需当前环境/逐签名重新验。

本轮定向证据：`cargo test --manifest-path experimental/ace/Cargo.toml -p ace-source-query --lib foreign_tests --message-format short` 最终exit0，4/4，0 ignored/37 filtered，编译6.83s/执行0.01s。断言含：真实普通source绑定外部声明且仅1本地body（无stub）；移交普通verify拒绝；篡改symbol或同宽i32→bool签名拒绝；solver disabled时暖命中；header契约变化仅main真实reader再solve、independent保持hit；HIR二进制restore候选能重新qualify、降版本拒绝。首次定向红（verify.binding-invalid，迭代call emitter遗漏bindings）已修复为两个emitter路径统一外部索引映射后绿，保留为本轮oracle区分错误绑定的证据。

构建接线验证：`cargo check --manifest-path experimental/ace/Cargo.toml -p ace-lowering -p ace-a2x-core -p ace-a2x-c -p ace-native -p ace-av -p ace-a2x-rust --message-format short` exit0（27.67s）；AV既有3 warning（closures_fn_session env_layout未读/mix_u64未用、owned_session shell_lo/hi未读）和source-query tests既有unused-mut已观察，尚非本包健康关闭，后续审查核归属。新Foreign在lowering目前具名`lower.foreign`拒绝，A2X仍会拒不支持call，**不是native/C交付**；下一步必须接共享MIR ForeignCall及两个真实consumer，不能保留此暂态作为最终实现。

完整职责命令 `cargo t ace-source ace-semantics ace-artifacts ace-foreign-contract` 已启动并持续存活；工具会话handle **71627**（工作目录D:/autostack/.wt/lang-938/auto-lang），输出根`target/test-groups/run-zezrp1rq/`，当前准备case-parent/av等原职责工具，不是PASS。继续时优先poll同一handle，不因观察超时重启。测试进入前的source_state=`a00f1776e18c0013489b0134d88a52b2579f443ce2b2c1053ecca77af1de1dbe`；启动后仅补两个comptime测试SemaInput新字段空集初始化（不在本轮执行组选例中）。`cargo fmt`修了数处既有format，后续提交前剔除无关format-only路径；完整组仍待终态，当前无review/merge资格。源码材料均在938树/统一scope；未生成C/object外部探针。

stage: goal | plan_id: PLAN-938 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: in_progress (source/HIR checkpoint; acceptance pending) | worktree: D:/autostack/.wt/lang-938/auto-lang | commit: cab2cc050 + owned uncommitted source/HIR integration | next: poll 71627, repair any reader/constructor failures, scoped commit, then T-02 shared-MIR/native/C/restore-first-edit

2026-11-05 T-02 checkpoint 3：提交 `27b1c0413dbd03754b951d8fb86646db1696aee4`（61文件，2204+/68-，树clean）。已实现最小**真实源码双端调用**：同源 `fn main() int { return foreign_add(20,22) }`，native经唯一共享HIR→MIR `ForeignCall`→Cranelift→COFF，与独立MSVC C provider对象链接，42；仅provider body+1变43，caller COFF逐字节相同。C消费同受检HIR→真实业务fragment直接call→独立C provider对象，亦42→43，generated.c逐字节相同。对照源/头/物理表为永久回归夹具`experimental/ace/crates/abi/cside/source_foreign938.{c,h,contract}`，不是外壳代调用。暂存provider原稿及每次实际外部输出全部在`D:/autostack/plan-artifacts/plan-938/{provider,tests}/`。

实现身份：中立`SourceDecl`无provider body digest/宿主地址；显式context data symbol（实际进程target对象内地址，由native/C取符号地址，非持久指针）；原普通codegen拒foreign，显式native codegen需current AdmittedForeign且逐声明重验；logical bool u8/I32边界有值检查，fallible status先分支、只在success读out（这两支**已实现但还没有本轮运行证明，不核销AC-03**）。共享MIR新指令spec行`FOREIGN_CALL`0x36；独立候选wire AVFI v1/profile9/schema `auto.abc.foreign-scalar/1`（纯scalar MIR型，旧AVAB/AVAD不升版）；严格decode后仍需verify_mir/currenthost，AV准入显式`foreign.unsupported`；compact codec仍具名不支持。native指纹/真实imports元数据含实际foreign声明键/符号、ctx和status runtime，不含库body。C新library fileset `auto.a2x.c.foreign-library.fileset/1` / recipe `ac-a2c/hir-foreign-scalar-import-v1`，完整generated/runtime/imports/status runtime，**无main.c、未剪旧exe**；Rust provider具名`a2r.foreign-unsupported`。

本轮核查修复：实际product entry `compile_scalar_product`漏接专门verify（此前只direct facts entry接了），现已同门且5th定向测试覆盖；foreign query错误不落到看不到manifest的旧diagnostic oracle，保solver有序诊断/当前structural span。C CRT工具原`/TC`会把外部COFF当C源，新增结构化`compile_bridge_crt_with_objects`将objects明确放`/link`之后，原helper行为不变。

验证（真实终态）：(a) `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-abi --test conformance source_foreign:: --message-format short` 最终exit0，2/2（native+C实际独立编译/链接/执行，0 ignored/94 filtered），最后编译1.29s/执行2.83s；原始native第一红为`mir.capability-unknown`，已补结构准入并保AV前拒；C第一红为`/TC`误处理obj，修工具参数后绿。(b) `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-source-query -p ace-foreign-contract -p ace-semantics --lib --message-format short` exit0，42+27+21=90/90，0ignored，编译2.47s/运行0.06+0.00+0.01s；source-query旧unused-mut本轮清除。(c) native/a2x-c/a2x-rust相关cargo check exit0；AV三个旧warning仍在案（本包未宣称健康关闭）。改动Rust文件定向rustfmt(skip_children=true)/git diff --check/pre-commit均绿，无关format-only路径已退回。

**完整职责组71627终态已结束，exit1，不能复用句柄当live**：run-zezrp1rq各子组用例outcome=PASS：ace-source359/359、ace-semantics157/157、ace-artifacts40/40、ace-foreign-contract30/30，共586/586；但总gate_passed=false、discovery_complete=false，原因`code/config/inputs changed during execution`（本会话补了comptime空外部集构造与product接线/diagnostic反例，非外部故障），整命令713.00s。保留此红，不称冻结门通过。未受影响旧路径的子项证据可按输入归属复核复用；本轮补改已以上述90项+双端2项验证。最终职责并集由实现全部完成/冻结后再执行一次，不为此checkpoint重跑全部工具/正式sample。

下一执行重点：T-02还缺**无预热source恢复首编辑/首错修复证明**，故仍1/13、T-02不勾。先补双端fallible/ctx/bool/源序/短路实测，再接929/934原环境/required页与原会话事务，不建第二parser/solver或用整源重编冒P0。Native现`foreign_lower::compile_source_units`仍whole cold组合，C现`foreign::assemble_library`仍whole库装配；二者不是函数级增量实现，不核销AC-04/05。正式worker/TestCase/新固定组/认证/6×30预算等T-03..11全部未关；review/merge不可开始。下一轮需检查现有legacy测试temp输出根（原固定组测试自行用系统temp如ac796-*）；后续任何测试进程必须显式把TEMP/TMP指向938 scope子目录，收尾对已有temp根按命令/时间/归属核查、未知归属不得盲删。

stage: goal | plan_id: PLAN-938 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: in_progress (real native/C source calls proven; restore/incremental pending) | worktree: D:/autostack/.wt/lang-938/auto-lang | commit: 27b1c0413dbd03754b951d8fb86646db1696aee4 | next: work T-02 restore-first-edit and failure matrix

2026-11-05 T-02 checkpoint 4（最小可行性通过；仍非完整 work pass）：提交 `b2c82198d0967ab490b498d804c0c514bc784ee0`，实现树clean，24文件3020+/76-。已证明本包所需双端链与真实进程restart→立即首错→修复，T-02勾选、计数2/13；未核销T-03..11完整合同。

新增共享 `source-query::frontend::Frontend`：复用929的UnitCatalog/单函数parse及core候选事务，934的同一query solver/事实准入和BodyTypeSupply；cold走实际product entry，暖只提交被改FnItem（其他callee为中立签名，不伪造InputFn body），`prepare_candidate_document_with_supply` + `verify_foreign_function` 产生真单体HIR及当前权威绑定的凭证。环境/目录验证在任何安装前同时完成；非法/取消drop/迟到候选不写current。core旧None入口保行为；恢复未改AST仍None，旧body只保声明元数据，不会为first edit whole-parse/ensure_env。已补核心scalar-only事实recheck门。

冻结最小接口/页身份：原`ace.foreign.manual/1`、HIR foreign codec2、AVFI v1/profile9/`ForeignCall`0x36保留；foreign声明环境用`core-decl-env/2`，非foreign仍/1（/2无foreign不是/1别名、旧None restore不接foreign）。共享frontend根/事实结构见`ac-source-facts-session/1`：受接受root的独立SHA256、source/catalog/declaration/signature/完整required per-body事实+结构walk+冻结签名依赖版本+unit源码SHA256，来源漂移只平移元数据；未知字段/错owner/key/node/kind/span/漏表达式/漏let/缺页/内容替换/权限变化拒绝。body页在生产时编码为不可变Arc，checkpoint只改root/声明/原点元数据，不重走或重编码未改body。root<=8MiB、source<=8MiB、事实页<=1MiB、最多8192函数为本source入口上界，其他原更严帽与T-09先reserve义务不变。

host组合`abi::source_foreign::Session`：选择Native或C；Native复用唯一`lower_function_unit`→native外部调用codegen，C通过原`TargetProvider`接缝的`ForeignTargetProvider`消费Checked HIR单体，新增独立fragment层AXCF1在a2x-store三处registry映射，旧AXC1/AXR不借用；C仍完整library fileset，无剪main。target snapshot=`abi/source-foreign-session/1`，provider对象/当前契约/代际与source前端/实际target页一起绑定，未改产物保同一Arc。provider body单独再准入/重绑定，不改任何caller parse/solver/HIR/MIR/native/C生产。持久化通过原ManagedStore S938页/RequiredData边/原生lease，非另一个cacheDB；复用上一出版物中未改页id时不额外读取或写body payload；新root的required边在store原子liveness事务里校验。存储路径/预算完整失败矩阵仍T-08/09负责。

真实运行证据：本轮5个`source_foreign::` ABI测试均PASS（0ignored/94filtered）：20次双端实际执行覆盖bool0/1、fallible非零ctx、除零status=-1、MIN/-1=-2且先status后out、bool2 raw原因=-2147483647、多次参数trace/counter、短路与named-arg源序、5混合ABI位置；原42→provider+1=43双端保持caller字节；selected-target drop/restore首错旧root仍运行42→修复43（并与独立cold业务产物逐字节一致）→前缀变长非读者编辑→再恢复读者编辑44→provider-only45，未改函数生产/事实payload生产0且Arc/环境body identity保留。进程证明由新`source-foreign-restore-probe`承担：父关闭cold session，只向独立子进程给原ManagedStore接受root及当前宿主契约/provider权限，子进程required深验后**无预热**先非法编辑再修复，父独立oracle实际43；restore whole/body parse、solver、HIR、body encode、MIR/native/C生产均0，失败尝试与修复共2次真实solve。此bin是T-02有界可行性probe，**不是正式TestCase新recipe/证书已接好**，T-03/10仍必须正式接父worker/SDK/严格元数据与互拒。

验证终态与复用：(a) `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-abi --test conformance source_foreign:: --message-format short` exit0，5/5，编译8.91s/执行21.70s；新注册probe读者也实际运行。(b) 随后只修`core-decl-env/2`空lane拒绝与seed递归自签名去重，最终`-p ace-source-query -p ace-semantics --lib` exit0，45+21=66/66，编译3.04s/执行0.06+0.01s，含新增递归restore反例。(c) `-p ace-core --test conformance decl_env_roundtrip_929` 最终exit0，2/2、339filtered，编译1.95s/执行0.01s；`... source_units_928` exit0，25/25、316filtered，准备0.19s/执行0.76s。(d) 对最终decoder/seed再次跑`-p ace-abi --test conformance source_foreign_real_process` exit0，1/1、98filtered，编译12.11s/执行4.25s；复用未受该decoder空lane修复影响的物理调用矩阵，不再无意义跑20次。(e) a2x-store最终cargo check exit0（1.95s），本轮新增库/bin无新warning，改动Rust定向rustfmt、diff --check、pre-commit均通过。上轮3个既有AV warning仍在案，尚未宣称health通过。所有新测试进程TEMP/TMP显式938/test-temp；独立C/PE/store输出均938/tests/PID子目录。

红→绿与调查：矩阵首红为fixture `}` 与return缺换行；native CRT wrapper首红为C不能直接命名带点的私有符号，改用原`emit_wrapper_coff` ac_start真实入口，foreign call仍在native业务体。恢复首红发现ABC本就不持久化owner（decode给wire schema+空module），现从接受root中的required envelope重锚canonical MIR schema/m_src后再深验/核私有symbol，未执行错误对象。旧929坏schema控制首红暴露/2空lane须拒，已保原负控而未删测试；递归seed主动审查发现owner与sig_reads self重复，去重并新增真实递归restore回归。

开放项/下轮入口：继续T-03..06的正式首例与严格拒绝矩阵，不把本轮通过的最小proof冒完整AC关闭。T-07还必须实现foreign声明签名/cap/header搜索域变更的实际reader批量事务（现Frontend::prepare只支持签名不变body，provider-only已支持）；补真实attempt/discard/IO及shared wrapper/tool工作分项。T-08补目标ready前唯一跨source/foreign/target安装点、声明首编辑、required损坏/发布故障/权限撤销全矩阵，当前session.commit尚非已完成该合同的出版API。T-09先reserve覆盖所有并存/JSON/恢复索引、8MiB hot+8MiB index/48MiB resident/128MiB store AND、lease/取消/512修订GC全部未关；T-10正式recipe/父decoder/SDK/固定组/独立certificate、T-11原6×30/P90/原负载/旧桥回归未关。最终冻结职责并集仍pending，71627旧gate red事实不变、不再当live。

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | base_commit: a0750086c | code_commit: b2c82198d0967ab490b498d804c0c514bc784ee0 | dependency_revisions: D929/D934/D937 archived baseline, auto-down 7c4d5cb | task_ids: T-02 complete; T-03..11 open | evidence: scoped commands above + plan-artifacts/plan-938/checkpoint.json | blockers: none | next_responsible_role: work T-03..06 formal first case/admission matrix

stage: goal | plan_id: PLAN-938 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: in_progress (T-02 feasibility passed, full work still open) | worktree: D:/autostack/.wt/lang-938/auto-lang | commit: b2c82198d0967ab490b498d804c0c514bc784ee0 | next: work T-03..06

2026-11-06 work checkpoint 5（上一轮为明确progress；本轮T-03..06关闭，执行计数6/13，未到execution_done）：实现提交 `086c5b81837c010d8c8b7dc5c68dfb106023c545`，33文件1992+/52-，唯一树clean。保留r1目标/阈值/仓与前置；main自其他会话推进到f05b28ead，但本树仍在原基面与自己提交链上，未混入935/936/943代码，main既有文档WIP不暂存。

正式首例已接既有受限ABI通道（不是另造runner）：recipe `abi/source-foreign-certify-v1`，沿原`abiworker/1` envelope，独立`abi.source-import-deck/1`与`abi.source-import-report/1`严格typed schema；consumer只native/c。父adapter/dispatcher/contract/recipe名单/stale识别/SDK侧纯镜像/固定组`case-ace-ca-d-source-938`同步注册。源码仍是普通Auto source，manifest为单独typed输入；显式certify才调用独立C编译/实际PE，不称此recipe为daily emit。新的source-provider policy从独立审定add表构造宿主期待契约，candidate不自授安全；C参考`cside/source_foreign938_reference.c`手写20,22调用，独立cl编译/链接/执行，只返回观测数据；**期望42/43仅在case父侧**。provider事件来自真实C函数entry的条件观测钩子，actual/reference各自进程，父核原始stderr与有序事件，不拿“两个结果相等”当oracle。库仍由真实业务fragment调用，无main wrapper偷调外库。

Report精确含方向、ordinary-source profile、AutoABI domain/version/profile/callconv/target、实际consumer/compiler、cl路径/banner/内容hash/flags身份、审定review/contract/header/table、当前provider对象、当前普通source SHA256、两个实际退出值、实际事件/原始stderr、各层真实生产与有序source diagnostics(stage/code/message/span)。native compiler身份更正为本身`ace-native-foreign/1+ace-native/0.1.0+cranelift/0.126.2`；C维持独立printer身份。父对actual/reference各自比较独立exit oracle，源hash对当前deck重算；错方向/typed来源/错ABI/错consumer/错compiler/无call/重放source/unknown或丢字段均拒，旧ABI/C/Rust报告不借用。错参数真实source诊断保`source.foreign.type-mismatch`、resolve阶段、原序/消息/span35..39，target生产/实际call=0；失败cold解析入口与solver的真实尝试也计（2 parse+1 solve），不是因无产物写0。

额外准入核查：manual scalar lane拒未知ownership/effects、缺explicit source cap、伪CheckedSource、未有本lane当前service权威的semantic_ref（明确unsupported，未自称已资格化stale ref）；保已审定mixed checked/pure接口合法。core新增**共享**declaration-only `validate_foreign_context`供query前门和原binder复用，冲突/无权限直接用当前源名span拒、solver0，不复制另一个解析器/solver；正常local shadowing保留，local被当callee时source.not-callable，普通读取local不伪造foreign依赖。UnitCatalog parse/fragment错误现在保原Diagnostics，未降成字符串infra。Source replacement定位只用现有目录与字节替换范围，不全源parse。

实际provider不仅核digest/宿主清单，还以标准object reader核AMD64可重定位COFF、真实required定义及function=Text/context=Data；只保存被选择符号的借用元数据，不把全库名字拷贝进索引。真实对象删/改provider_add定义时，即使宿主清单声称存在，也在**不可解析的Auto source之前**以foreign.provider-symbol拒；把ctx错绑到现有函数符号时以foreign.provider-symbol-kind拒；合法非零data ctx两端cold与原实际运行矩阵保绿。header/table归一LF并用cside局部.gitattributes固定，否则固定manifest的内容hash会随checkout换行漂移；不改物理表含义。

协议/判定反控：request envelope与end对象严格deny_unknown_fields；丢方向/空选择/错版不绿；source case须以query结束，restart-only即使有exit=42也不可PASS，programmatic父入口同门。父stdout读到EOF后才解码，迟到第二个JSON不能越过第一换行；原contained Job/单deadline/输出caps/clean exit继续生效。永久root单测覆盖迟到重复stdout、响应后hang，ABI集成覆盖unknown wrapper/end、empty selection、被忽略的from/to字段（全部不建store/不调producer）。T-08复杂发布故障和T-09所有预算压力并未据这些控制核销。

验证终态：
- `cargo t case-ace-ca-d-source-938` 最终run-r4y14802，exit0/gate_passed=true；11个注册检查实际执行、0条件排除：10 PASS + **1预登记expected FAIL**（case-control-oracles新wrong-oracle控制）；wrong-oracle里actual=42/ref=42/父expected=43，两个独立mismatch均存在。最终实际运行39.45s/整命令61.39s（包括工具准备）；该组all=false，非默认daily隐式执行。
- 前一run-cve5xg0c和run-lyrfib97也11项同义通过；随source原始诊断/failed-attempt计数、对象准入及restart-only修正后，才使用上面的最终run，不借旧更弱报告。最初run-ido4ebxh 8/8第一首例证明，新增两个type负控+wrong oracle后分母11。固定runner的discovery_complete是discover-only专用字段，普通执行false不代表漏发现；以inventory/executed与gate真实判定，旧71627 code/config changed的红仍不得当绿。
- `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-source-query -p ace-foreign-contract --lib --message-format short` exit0，48+28=76，0ignored，编译5.46s/执行0.07+0.00s；含shared source conflict/cap span、local shadowing、parser failure计量与unknown effects/service反例。
- ABI `--test conformance source_foreign::` 6/6 PASS（4.11s编译/22.67s运行，94filtered），原实际bool/status/ctx/order/short-circuit/混合槽、无预热真实进程恢复、provider-only都重放；后补context-kind正/负双consumer控制1/1（3.57s/0.18s）与unknown/empty协议控制1/1（1.33s/0.28s）；改变context kind guard后只重放同因missing-symbol控制1/1（0.15s/0.20s），不再重复20次未改的物理调用。Rust目标runtime_source实际拒a2r.foreign-unsupported、AV loader拒foreign.unsupported均永久断言。
- `cargo test -p auto-test-case --test source_foreign_938` 2/2（0.94s/0.00s），合法native/C与旧legs/无独立oracle/restart-only互拒；`cargo test -p auto-lang-core --lib test_cases::abi` 3/3（编译16.62s/执行0.45s，1628filtered）验证typed report身份/actual calls/当前span，以及真实peer的late output/hang；使用当前已构建release ABI_CASE_WORKER，未降deadline或跳过清理。首轮root cold libtest编译65s仅跑2个相关控制，不运行全1630测试。
- 新代码定向rustfmt、diff/cached diff --check、pre-commit通过。SDK初次check为cold大依赖准备，exit0；root既有warning基线131(lib)/145(libtest)，AV旧3条仍在案，本轮无新增warning，不能据此声称整个仓health已零。

尚未关闭的完整合同：T-07 actual foreign声明/签名/cap/header搜索域变化的actual reader批量事务、完整P/M/R/attempt/reuse/discard/IO和shared wrapper/tool分账（现在body与provider-only链已能工作；正式worker遇provider variant/module变更仍显式拒，而非假装已热换）；T-08来源/权限/runtime每次资格化、声明首编辑、目标ready前跨层唯一发布与全坏页/故障/取消/迟到矩阵；T-09并存/JSON/恢复索引先reserve、8MiB hot+8MiB index/48MiB resident/128MiB store AND、lease/512 revisions/GC；T-10完整多fixture certificate、source emit daily无spawn入口、全部SDK producer-reader/旧桥三腿与937回归；T-11原6×30/P90/原source/native/A2X负载、Q4原required oracle矩阵、SD/939调用钩子交接。**首例report不是完整certificate，T-03..06任务关闭不提升这些AC为已满足**。最终冻结职责并集仍由review/merge阶段按真实diff运行；不跑全experimental/all凑绿。

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | base_commit: a0750086c | code_commit: 086c5b81837c010d8c8b7dc5c68dfb106023c545 | task_ids: T-03,T-04,T-05,T-06 complete; T-07..11 open | evidence: commands above + plan-artifacts/plan-938/checkpoint.json | blockers: none | next_responsible_role: work T-07 actual reader transactions/measurement

stage: goal | plan_id: PLAN-938 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: in_progress (registered independent source certification and complete scalar call/admission paths) | worktree: D:/autostack/.wt/lang-938/auto-lang | commit: 086c5b81837c010d8c8b7dc5c68dfb106023c545 | next: work T-07


### Goal checkpoint 6 — declaration reader batch and formal provider binding (2026-11-06)

代码提交 `4b2b894de`（15 files，1208+/89-）；T-07实质推进，但完整计量尚未关闭，任务仍6/13、executing。本轮没有新阻塞、没有复审/merge，不把最小批事务通过算全部AC通过。

- core新增不拷贝/不遍历保留body的`ForeignContext`声明候选；临时声明graft只在core私有单体prepare端口内使用，退出/错误/unwind先恢复旧声明表；批commit在写任何声明/body前核全部env identity/base/slot/context与重复owner，统一推进generation。普通FnCandidate也绑定准备时foreign表/槽位，不能在不同声明上下文安装。
- 纠正位置索引风险：ResolvedProgram保process-local稳定foreign槽位，增名追加、删名留tombstone，所有实际adapter/typecheck reader经统一slot→logical-name查询，未改body不重写。restore声明环境本就无已解析body，可在R重新建立当前声明槽位；无AST/solve/lower。source_declarations按source名规范排序，传输重排不改变语义；每decl增加明确`content_version`，名义版本单改也真实失效，provider实现digest仍不进入caller声明key。
- 共享Frontend以**已接受resident页的依赖元数据**选择实际reader，不依赖可能含未安装候选的query缓存；通过SignatureCatalog替换声明元数据。实际reader冷有current-span AST则Rc复用，否则由原UnitCatalog同一fragment parser投影该owner（恢复首编辑/前缀漂移均如此）；不parse全模块、不物化非读者。每reader经原SQ solver/BodyTypeSupply/core单体HIR/当前foreign verify，目标native或C都准备好后才能批安装；非读者fact/target Arc与body identity保持。
- 冷与暖共用原ordered solver diagnostic重锚helper，非法签名保原`source.foreign.type-mismatch`与source span，未降成ApplyError字符串。恢复后第一次声明非法编辑不安装，旧程序实际42；合法effects契约变更、取消、过期base、再次restore首声明edit都验证。新cap未获当前session授权时在parse/solve前拒，旧root不变。
- 真实共享header文件加注释、重新独立C编译并由fixture owner审定新header/object，两个foreign reader各重产一次，main/independent的producer增量0且fact/target Arc不变；实际链接运行42，随后required restore solver0。增删/重命名声明控制通过，旧reader删除为真实诊断/旧root保留，新名字追加后真实call可执行，旧not槽位在下一次编辑/再恢复仍正确，非读者不重写。
- 正式worker允许provider variant切换：先按当前fixture policy资格化旧绑定、required R恢复接受根，再显式rebind新对象；不把新对象强灌旧root、不cold fallback。provider对象复用原ManagedStore C cache；旧provider如确有冷生产亦计入compute。独立进程传同一普通source/同一manual声明、同一store，native/C均真实42→43→42，actual/reference各自独立运行且事件都为provider_add；后两次source_parse/solver/HIR/MIR/native/C片段全0，首次新provider compute1、切回已有provider compute0。shared assembly/link仍实际执行，并未冒全部工具/IO为0。

验证（每个测试进程TEMP/TMP显式938/test-temp，所有provider/PE/store在938/tests/PID；无live session）：

1. `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-source-query -p ace-foreign-contract --lib --message-format short` exit0，49+28=77/77，无排除；编译6.72s，执行0.07+0.00s。含新增名义版本真实reader反控。此前48+28也绿，最终用49分母。
2. `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-abi --test conformance source_foreign:: --message-format short` exit0，10/10、94filtered、0ignored，编译17.27s/执行54.25s；包括原20实际物理调用矩阵、provider/ctx准入、原独立进程恢复、现声明/header/cap/稳定槽位两项。随后只改worker provider transaction并新增进程验证：`... source_foreign_formal_worker_rebinds_provider_without_any_caller_production` exit0，1/1、104filtered、0ignored，编译4.40s/执行17.64s；注册conformance共105项、source_foreign切片现11项。二者代码触面/证据对应，未为新增worker无关改动重复旧20物理运行。
3. `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-core --test conformance --message-format short` exit0，341/341、0ignored/filtered，编译11.97s/执行2.49s，涵盖普通单体候选、collections/closures、decl_env/UnitCatalog/源诊断所有原控制。
4. ace-source-query与ace-abi定向check exit0；改动Rust定向rustfmt、diff/cached diff --check与pre-commit通过。本轮新代码无新增warning；继承AV的env_layout/mix_u64/shell_lo+hi三条既有warning仍在案，未据此声称全仓health零。

首红与修正：测试误用PeRun tuple字段在编译期报错，改原exit_code；未审定同symbol双别名由既有manual选择门正确拒，控制改为原未用函数的source-name迁移，不放宽模型；冷session累计计量不能断言绝对0，改核对已观测cold基线的真实增量0。原负控保留，无减样本/改独立oracle。

下一轮：T-07仍需完整attempt/produced/reused/discarded、P/M/R/IO/shared wrapper/tool分账、声明/权限/header搜索域与历史复用细节；T-08需强制每次权限/runtime资格化、目标ready/发布故障/取消/迟到全矩阵及正式声明首编辑路径；T-09原AND预算先reserve/512修订/lease/GC，T-10完整certificate/daily emit无spawn/SDK与旧桥义务，T-11原样本/P90/Q4/SD/939交接保持全部开放。最终冻结固定职责并集仍pending，旧run-zezrp1rq input-changed red不改写。

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | base_commit: a0750086c | code_commit: 4b2b894de | task_ids: T-07 actual declaration reader transactions/proof; T-07..11 remain open | evidence: commands above + plan-artifacts/plan-938/checkpoint.json | blockers: none | next_responsible_role: work full measurement/atomic publication/resource gates

stage: goal | plan_id: PLAN-938 | plan_revision: r1 | round: 6 | from: work | to: work | status: executing | outcome: in_progress (actual declaration readers, stable slots and formal provider binding verified) | worktree: D:/autostack/.wt/lang-938/auto-lang | commit: 4b2b894de | next: work T-07/T-08


### Goal checkpoint 7 — ready publication/current qualification and genuine declaration-first process (2026-11-06)

代码提交 `d1bba7ae5`（15 files，2296+/347-）。前轮分类为progress；本轮同树work推进T-07/08，不进入review/merge。仍executing 6/13：T-07完整计量/历史复用、T-08跨宿主边界与资源义务尚未整体核销；不因本地发布矩阵通过提前关闭AC-04..07。

机制与实际结果：

- UnitCatalog新增候选manifest/候选位置reader投影，复用**同一**fragment parser；Frontend可从既有声明/目录元数据及候选事实页生成候选checkpoint，无需安装、克隆整个Env、遍历未改AST或重编码未改事实页。支持foreign契约与单个普通source body一起准备/发布，prefix变长时只投影实际reader；非读者页/对象Arc保留。warm冲突/cap前门复用core的声明metadata校验，按当前source位置重锚；新增prefix drift后冲突span反控，parse/solve/HIR全不增。
- 新`source_publication.rs`：Edit(Current/Body/Foreign)→候选snapshot+借用目标overlay→真实selected-target链接ready→当前HostAuthority/runtime/emitter重验→持久required闭包→独占宿主pointer锁→精确base比较→临时文件sync→最后cancel/authority/base重验→Windows同卷REPLACE_EXISTING|WRITE_THROUGH→安装同一source/foreign/target候选。失败不写当前source/HIR/binding/body artifacts；temp/文件锁/lease走RAII释放。Session无无权限run API；测试便利trait仅在fixture模块，真实worker/probe均须当前host callback。执行与发布前流式核ready PE内容并持只读共享句柄，拒被改exe。
- Root显式绑定runtime composition identity（persisted runtime page+当前SDK failure/event assets+emitter），current权限/provider/binding代际/runtime/emitter均由独立当前host输入重新资格化；authorized restore也核当前runtime/权限。`AdmittedForeign::requalify`仅检查当前registry/cap/symbol/binding元数据，不重投影PhysicalPlan；准入时缓存不可变声明/digests，R的signature核对不重编码声明。untrusted SourceDecl结构校验使用已有名义content_version，无需为结构校验生成新声明编码。
- 正式worker的body/绑定变化现在走ready publisher；provider generation按接受记录单调增，不再用bias+1伪代际。源码首错保原有序stage/code/message/span，未转infra字符串，失败parse/solver尝试如实计，未运行provider、接受pointer不变。pointer解码与expected SHA取自**同一次有界读取的相同字节**，避免读旧record后又取新pointer hash造成lost update。
- native/C从真实R后的首次非法编辑保旧42；合法修复ready实际43，安装后snapshot digest与事先候选完全相同；再次R不生产，第一编辑ready实际44，未安装时旧43根保持。矩阵覆盖链接前/后撤权、runtime变化、取消（含最后一次qualification期间取消）、metadata故障、Windows替换失败、伪/迟到pointer、另一个真正旧base会话迟到、ready exe篡改；错误后旧接受pointer/root均保留且旧程序实际42。
- 新签名的C provider、header、独立人工物理表三者一起从`i32(i32,i32)`改为`i32(u8,i32)`（第一参逻辑bool），source旧实参20被真实类型门拒。再将普通source修复为bool+42与新契约一起发布，两端实际43；warm目标产物与独立fresh冷编译逐字节相同、非读者producer0/Arc不变。新增独立子进程：接受旧required根→**无预热首声明错误**→同source+新契约修复→实际43；当前候选contract/header/table/provider的授权来自CLI独立host参数而非manifest自证；R各生产0、两次solve、main/independent HIR0。旧body-first process也改用ready发布而非先raw commit。
- required页逐页缺失/损坏、错schema/compiler/contract/provider、伪safe字段、重新hash的错source/foreign声明/target owner/未获当前SDK授权runtime全部拒；正例R不parse/solve/emit。当前canonical source与目标对象没有伪Checked持久化。

验证（所有测试进程TEMP/TMP显式938/test-temp；原始provider/header/PE/store/request均938/tests/PID；无live sessions）：

1. nextest full配置：`cargo nextest run --manifest-path experimental/ace/Cargo.toml -p ace-abi --test conformance --config-file .config/nextest-full.toml -E 'test(/^source_foreign::/)' --no-fail-fast` exit0，run `d8164c15-3226-447b-9df5-682d98dce9fa`，15/15 PASS、94过滤、无条件排除，运行251.691s（准备0.17s）。保原20实际bool/status/ctx/order/短路/混合槽矩阵与所有旧reader/restart用例。慢原子项包含物理矩阵69.450s、formal binding58.966s；owner PLAN-938，未按耗时拆/减样本。机器同时有主检出`cargo t all`、936 ace-comptime等独立任务；本轮不是性能采样证据。
2. 随后自查修最后cancel检查、相同pointer字节basis及metadata-only requalify；只重放受影响控制：同命令用精确OR选择publication_faults+formal_restart_first_error，run `6e2ca5b7-4c21-41dd-9222-b08a5677e65d`，exit0，2/2、107过滤、89.354s（46.054+43.297s；准备0.23s）。required restore最终单独exit0，1/1、108过滤，编译12.41s/执行0.32s。未因这几个metadata/边界修正再次重跑未受影响的20次物理调用。
3. `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-source-query -p ace-foreign-contract --lib --message-format short` 最终exit0，50+29=79/79、0排除；编译9.40s，执行0.23+0.22s。含新增当前registry重验与prefix drift冲突定位；先前50项重放也是绿，最终采79分母。
4. core conformance本轮exit0，341/341、0ignored/filtered，编译8.04s/执行5.93s；覆盖原UnitCatalog/decl_env/各profile。其后只把foreign声明前门提取为同一个metadata helper、线性metadata遍历，相关foreign source正反已由最终50+29和ABI定向控制覆盖；冻结职责并集仍待最终review/merge。
5. `.config/nextest{,-full,-t3}.toml`均对source_foreign模块使用原`threads-required=num-test-threads`隔离本invocation的嵌套工具/短门控制，未改任何发现/编译/运行/父IPC截止或原条件。full实际运行通过；daily/t3 list均total109、selected15，exit0（daily准备29.64s、t3 0.26s）；仅list不冒执行。改动Rust定向rustfmt、diff/cached diff --check、pre-commit通过；AV旧3条warning仍在案，本轮无新增compiler warning。

红记录：新测试曾误取StepOut.status而不是原outcome（Null断言红），修为原协议字段；新增child材料引用曾move路径后借用，编译红修为借用读取；genuine declaration-first child一次原10s `cl -?` Deadline，exit2（整个测试43.73s），当时可见主检出全量/936等并行工作，未放宽门。隔离同一测试重放exit0（准备0.41s/运行66.86s），随后完整nextest切片中的同一用例exit0（16.633s）。保红与环境观察，不把超时当PASS或性能基线。提交时git自动packing提示unreachable loose objects，本轮未prune任何对象。

开放/下一轮入口：T-07完整attempt/produced/reused/discarded、P/M/R/IO、shared wrapper/tool分账与真实历史复用尚待实现；T-09所有并存/JSON/恢复索引/复制先reserve，原8MiB hot+8MiB index/48MiB resident/128MiB store按owner AND，lease/取消/512修订/GC与scratch清理仍未完成，不能据本轮RAII小矩阵核销。T-08与T-10交界还须按canonical contract核对**父typed判定/transport拒绝与宿主accepted-root的边界**：当前正式worker在输出report前写自己的compiler current pointer；不得无证据宣称父IPC拒绝会回滚此pointer，须查清compiler缓存root与parent accepted root角色并用真实反控修正任何违反原AC的安装。T-10完整多fixture certificate（runtime/closure/binding等身份）、daily纯emit无spawn、全部SDK读者/797三腿/937 export仍开放；独立review-table当前文字写SHA256而实现/固定值为BLAKE3，须随准确证书/规范统一，不能对外错标算法。T-11原6×30样本、P90≤2000ms、原source/native/A2X/Q4负载、SD/939交接全部保留，尚未执行/不宣称通过。最终冻结固定职责并集pending，旧run-zezrp1rq input-changed red不改写。

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | base_commit: a0750086c | code_commit: d1bba7ae5 | task_ids: T-07/T-08 local readiness/current qualification/atomic publisher proofs; T-07..11 remain open | evidence: commands above + plan-artifacts/plan-938/checkpoint.json | blockers: none | next_responsible_role: work complete accounting/resource gates and parent acceptance/certification audit

stage: goal | plan_id: PLAN-938 | plan_revision: r1 | round: 7 | from: work | to: work | status: executing | outcome: in_progress (genuine declaration-first process and qualified ready publisher verified) | worktree: D:/autostack/.wt/lang-938/auto-lang | commit: d1bba7ae5 | next: work T-07/T-09 and T-08/T-10 boundary

2026-11-06 work checkpoint 8a（同一goal round 8继续执行，非stage handoff，6/13未变）：提交 `687aecb16befae8bf5eb79732526669a69c5d115`，21文件1482+/172-，唯一实现树clean。本单元落目标历史复用与实际工作观察；T-07..11仍开放，不据部分计量宣布execution_done。

目标函数键在Checked HIR语义指纹处计算、先于MIR lower/native emit/C print；owner/package/module、decl、实际foreign声明依赖、target/emitter、适用runtime ABI及caps分轴，不含未读provider body。新S938F是原ManagedStore内的可回收目标字节层，不是第二个DB；内存history≤512条/8MiB（仅此cache本身的当前上界，**尚不冒T-09合计resident已满足**）。C miss复用原A2xSession的AXCF1 semantic/fragment生产端口，当前provider即使命中也资格检查；新观察端口使用有界store read，未动其他既有cache入口。Native/C同进程A→B→A命中真实原Arc、producer尝试不增；持久B后drop/restart→第一编辑A从原store读到原A字节、lower/emit/print=0；purge仅S938F后C的B仍从原AXCF1命中，不另建C生产管线。

新增source-query::work::Ledger：按owner/stage记attempted/produced/reused/accepted/discarded/pending及P/M/R、逻辑payload I/O。solver计数移到真实InferSession入口，失败/取消/资源耗尽不漏attempt，前门未进入solver仍0；infer产物属于query缓存，候选project/facts-apply/resolve/HIR plan/bind/document/verify/page、MIR lower/verify/ABC/native/C/target encode票据由候选持有，commit才accepted，drop/取消/迟到/异常展开转discarded。core中立observer直接消费原真实producer hooks，不引入反向依赖，不覆盖外层body_work观察窗口，暖事实路径legacy typecheck保持0。checkpoint/store lookup/write为M，required/cache decode+验证为R；实际共享装配/entry wrapper/编译链接工具列owner `*` 的P，不冒未改函数生产0；装配写入字节单列。**I/O数值是API提交/接收的逻辑payload字节，不是物理磁盘I/O估算**，父Job原物理I/O仍需T-11采集。新表尚未接完整worker certificate，接口/physical-plan准入等所有owner端口的最终计量核对及ledger索引预算属后续T-07/09/10工作，不能仅因已有表就核销完整合同。

验证（所有进程TEMP/TMP显式938/test-temp，无live sessions）：

- source-query最终`cargo test --manifest-path experimental/ace/Cargo.toml -p ace-source-query --lib` exit0，54/54、0ignored/filtered；编译2.61s/执行0.11s。永久反例含cancel/solver resource真实attempt与profile preflight 0、非法源没有HIR attempt、候选pending→discard/accept守恒、未改函数P=0、observer异常展开清理及外层真实resolve/plan/bind计数保留。
- core共享hook涉及descriptor及各profile规划，故跑其完整conformance：`cargo test --manifest-path experimental/ace/Cargo.toml -p ace-core --test conformance` exit0，341/341，0ignored/filtered，编译10.53s/执行3.98s；未改语法或生产算法。最后相关cargo check exit0，仍仅原AV三条warning，本单元无新warning。
- ABI新增history+work反例先定向nextest，run `8928ad00-5c20-4b3d-a6ca-0fd9a345c509`，exit0，2/2、109过滤，准备/编译16.33s、运行21.768s。更早一次cargo test误用无模块前缀的`--exact`选到0/111，明确**不计测试通过证据**；随后上述选择实际执行两项。
- 最终source组合路径接入新cache且core观察hook变化，跑完整source_foreign模块：`cargo nextest run --manifest-path experimental/ace/Cargo.toml -p ace-abi --test conformance --config-file .config/nextest-full.toml -E 'test(/^source_foreign::/)' --no-fail-fast` exit0，run `be12084c-8013-448a-b97f-a12ebe84a05c`，17/17 PASS、94过滤、0条件排除，准备/编译16.64s、运行358.102s。保原20次物理bool/status/ctx/order/混合槽矩阵、独立first body/decl process、provider-only、required坏页、故障/撤权/取消/迟到发布控制；新增非读者发布事实/target两页复用、额外payload读写0与全R恢复后ledger P=0断言。慢原子publication故障矩阵127.031s、formal provider swap80.310s按实记录，此轮**不是**正式性能采样，不据此改原截止/P90门。
- 改动Rust定向rustfmt(skip_children=true)、diff/cached diff --check及pre-commit通过。提交时git再次提示unreachable loose objects，未运行prune。

继续本round下一单元：T-09先reserve/并存/恢复与编码暂存、8MiB hot（原ABI是磁盘hot/8→6MiB GC，不等于把AC冷AST硬塞8MiB）+8MiB layout/query index、native原16/24/8分区及48MiB/A2X合计、128MiB请求/namespace按实际owner AND；新cache/history和票据索引也须纳入。当前serde JSON/ABC/复制多处仍先分配再检查，snapshot/current软root仍会累积，正式worker run临时输出未收敛，均保开放。T-08 parent typed判定边界仍待准确审查：worker compiler current与父oracle PASS不是同一凭证，禁止把缓存当PASS，须以真实负控界定而非臆造“IPC失败已回滚”。T-10完整report/certificate/daily纯emit无spawn/SDK/旧797/937、review table算法文字修正、T-11原6×30/原负载/Q4/SD/939均未关。最终冻结职责并集仍pending，旧input-changed gate red不改写。

2026-11-06 work checkpoint 8b（仍是goal round 8的在途work单元，非handoff）：提交 `53f792f06bc757d392fb4e211a40a1e25aabd8a7`，12文件1696+/170-，实现树clean；仍6/13，T-07..11开放。

本单元落实部分T-09分配前门：中立codec增加有界JSON计数/精确try_reserve，第二遍序列化不能超出计数结果；facts二进制页先计数再分配、各count/最小剩余wire长度先核再分配Vec，解码字符串先验UTF-8再预留。source root、walk/deps与store-root用借用视图序列化，去掉整份source/root和结构walk/deps的临时副本，wire字节保持一致；SQ query key改为借用组件流式哈希，算法/字节身份不变。有限source wire的JSON流式预检分metadata、byte arrays、source/payload text；先预留typed模型/索引及parser scratch，再进入真实Deserialize，decode错误与budget错误分开。

新增共享Resources账号：48MiB总量，native MIR16/staging24/I/O8及ABI index8同时约束（额度只可收紧）；预留票据在失败/取消/drop退还，原子转交到不可变Arc的弱身份登记，外部pin仍占用、alias不重复收费；索引扩容先计旧新重叠，登记表自身容量也收费。已接source facts编码/恢复、target encoded+decoded持有、source/root快照、S938/S938F原store有界读与提交副本；ManagedStore读先为frame+payload预留，再由同票据转为resident，carrier kind严格核SnapshotRecord。history索引预留固定上界且内存键改为64字节完整QueryKey hash；R的cache构造只保存owner字段，C会话与owner编码延至实际函数工作。C当前SDK runtime页pin=`c7cf3971f2058ef57ca4d55386336abceb7af60ceb9fcef71ed4641954de6427`，资格检查不再通过runtime emitter重建期望材料，永久控制核pin等于当前SDK实际材料。

**这些是已接端口的预留/所有权上界，不冒充全进程实测内存或完整AC-06通过**。仍须纳入未接的source查询/候选HIR/声明索引、native MIR decode/codegen原分区及A2X片段读写、请求生产128MiB、namespace/ABI disk hot8→6 GC、source/native原更严source帽、所有lease/失败/重启清理；原负载/512修订与正式父Job内存/I/O还未跑。现Snapshot.publish仍为新根留soft root，pointer失败/成功替换没有完成其回滚/旧根退休，故GC义务明确未关。

验证终态（TEMP/TMP均938/test-temp，无live sessions）：

- 最终`cargo test --manifest-path experimental/ace/Cargo.toml -p ace-source-query -p ace-foreign-contract --lib` exit0，61+32=93/93、0ignored/filtered，编译2.43s、执行0.41+0.00s。覆盖有界JSON exact字节/超限不进入输出序列化、stateful serializer不能增长、真实wire内存表示上界、metadata越界不进入typed Deserialize、I/O不足不调store、错carrier拒、外部Arc pin与回收、并存额度及64KiB收紧额下真实payload压力→拒绝不改current source/页Arc→释放后repair。
- 压力反例首红60/61（32 contract绿，编译2.42s/源码执行0.07s）：实际上预算已拒绝，但generic Debug包装丢失资源错误名称；修为Frontend::Error::Resource及保留resource/decode的Display分类，原反例单独1/1绿（编译1.42s/0.01s），最终93项再绿。未靠放大额度或删断言修复。
- source ABI本单元先6项覆盖history、publication_faults、真实first-edit process、required坏页、SDK runtime pin及work/IO：run `f9096cd0-5224-4f0f-95de-9056955ae25a` exit0，6/6、106过滤，准备/编译29.58s、运行65.439s；publication fault46.605s。随后把JSON parser scratch归统一decoder预留，仅重放相关4项：run `4280a59e-3b4a-4dd5-9349-d36aae3afaaa` exit0，4/4、108过滤，准备/编译6.26s、运行4.876s。ABI conformance现注册112项、source_foreign共18项；本单元不是全部112或18项重跑，不冒正式性能样本。8a完整17项结果按未变输入/路径复用，最终冻结职责并集仍pending。
- 定向rustfmt、diff/cached diff --check、pre-commit通过；中途格式检查一次仅报PowerShell改写留下的EOF空行，rustfmt已修。无新增warning，原AV三条仍在案。旧input-changed gate red与原性能阈值不变。

本轮审查新增明确入口：当前源码恢复API接收已经AdmittedForeign的宿主输入，但`ManualManifest::admit`每次仍PhysicalPlan::project；S938当前闭包没有独立logical/physical/binding环境页。**不能把宿主构造移出计数窗口就宣称接口/plan的全链P0**。后续T-07/08需消费原937 codec/store接缝、补当前authority下的R-only环境深验/required闭包与真实接口计数；physical plan原validate仅核domain/唯一symbol，恢复不能据此冒完整slot形状深验。provider-only须复用不变逻辑/plan，声明变化才生产新plan。同步完成current root租约化持久/失败撤销/旧根退休、512修订GC和scratch收敛，再做T-10完整多fixture certificate、daily emit无spawn、父/SDK读者/旧桥三腿与937 export、T-11原6×30/P90/Q4/负载/SD/939。parent oracle/IPC判定与compiler current的角色仍须用真实反控审清，未擅自增加用户审批流程或降低原要求。

2026-11-06 work checkpoint 8c（同一goal round 8继续；非stage handoff，仍6/13 executing）：提交 `593dfc176d03730f6372d00da00157cb18b5269f`，13文件963+/68-，实现树clean。补齐先前审出的接口恢复缺口，不据此核销整个T-07/08/09。

foreign required闭包新增四种独立内容页：canonical manual逻辑声明、原937 `plan_codec` 物理页、source声明/内容摘要页、仅含contract/三页依赖/provider/generation的binding元数据页。它们不携带地址、SafeCallable、Checked或PASS；根以SHA256指向内容，人工contract/header/table/provider仍按既有BLAKE3身份。`AdmittedForeign`共享不可变接口/plan/声明与原字节Arc，provider-only `rebind`只新建binding页；普通body编辑全部复用。`PhysicalPlan::validate_against`逐函数/形式/返回/参数lane/名称/位置/register或stack bank/ctx/out依赖检查，**不调用project生成替代物**；source声明逐项与当前逻辑facts交叉核对，重新hash的伪声明/slot也拒。

正式worker与独立restart-probe的R入口改收当前HostAuthority，从required页解码并深验，不再在R前`ManualManifest::admit`生成plan。worker前门改为与独立有限registry的结构比较，当前policy contract pin=`702b82e7fb398a265182605b2aad5a85c076645691ebd15f1c8e8f599c24854c`，永久测试与实际canonical policy互核；R当前资格化不再编码policy。保留低层“已有admitted环境”的恢复入口，但仍要求读取四页并与该当前受检环境原字节完全一致。恢复之后runtime/emitter与当前权限仍重新资格化。SourceDecl摘要改为流式hash校验，不物化/重建持久字节。

中立contract实际producer端口新增可嵌套、异常展开安全observer：logical encode、physical project、source projection、plan/declarations/binding encode均记真实attempt/output；source-query Ledger可收这些票据，正式cold及provider-only候选已接。独立process的`restored`零值矩阵新增`interface_production`，不能再把plan生产藏到Session计量前。required typed JSON分配前门独立metadata profile（这里facts/source是对象，不能套source-session byte facts/text profile），预留parser scratch与decoded index后才入严格codec；保留metadata charge至共享admitted接口最后一个owner释放，scratch在解码后退还。**cold准入/新binding生产及所有query/HIR/native/A2X端口的完整预留仍属T-09开放面**，不冒该局部R预留为全量预算证据。

验证（所有进程TEMP/TMP显式938/test-temp，当前无live sessions）：

- 最终`cargo test --manifest-path experimental/ace/Cargo.toml -p ace-source-query -p ace-foreign-contract --lib --message-format short` exit0，62+35=97/97、0ignored/filtered，编译2.60s，执行0.06+0.00s。新增永久反控：cold真实六producer均被观察；R任何producer入口直接panic；provider-only仅binding encode、前三页与typed plan同Arc；撤权与旧binding拒；重新hash的slot/name/lane/return/删参、声明provenance/effects/logical及generation、所有页伪safe字段拒；扩展mixed FP/stack与ctx/out深验；metadata profile及外部pin/scratch回收。先前中间95项也绿，不替代最终97分母。
- `cargo nextest run --manifest-path experimental/ace/Cargo.toml -p ace-abi --test conformance --config-file .config/nextest-full.toml -E 'test(/^source_foreign::/)' --no-fail-fast` exit0，run `22430338-7b07-4e2f-8ef9-e4a6e80f2608`，18/18 PASS、94过滤、0条件排除，准备/编译15.90s、执行184.919s。覆盖全部source双端/20次物理调用矩阵、真实body/declaration首错修复process、provider-only、history、required坏页、撤权/取消/晚到/发布故障、runtime/policy pin及work/IO；新增四required页的逐缺/逐坏由同一实际循环执行。慢原子物理矩阵46.223s、publication fault36.891s按实保留，非正式性能采样。
- policy pin首次以`pending`占位校准时定向1项红，编译25.31s/执行0.00s，实际canonical digest由断言输出取得；随即写入固定pin，最终18项中的pin测试PASS。未变算法/门限以修绿。编译check exit0，仍仅原AV三条warning；rustfmt定向、diff/cached diff --check、pre-commit通过，无新增warning。

继续本round：发布目前仍在pointer切换前留soft root、旧root未退休，下一单元须在原ManagedStore上完成原子put→lease保护、发布回滚/旧根退休、实际reader lease及崩溃/GC收据；512修订/8→6MiB hot/128MiB namespace与请求、所有cold/候选/ABC/A2X预留、scratch收敛仍开放。T-10完整多fixture certificate/daily无spawn/全部reader与SDK/旧桥/export，T-11原6×30及负载/P90/Q4/SD/939均未关；parent oracle PASS与compiler current不同角色仍待真实反控。最终冻结职责并集pending，旧input-changed gate red保留。

2026-11-06 work checkpoint 8d（仍同一goal round 8继续，非handoff；6/13 executing）：提交 `1e24933e8034563e636d854a95969ad5c2542fa7`，7文件672+/43-，实现树clean。

原ManagedStore新增`put_artifact_leased`，同一个shared liveness区间内完成rootless put与OS-backed lease，封住put返回到grant之间的GC空窗；原put/lease/GC语义不变，无新DB。snapshot候选叶/根及旧required闭包都有lease，只有显式accepted snapshot API才注册soft root；current publisher不再用该API提前留soft root。stored恢复在第一次payload read之前取root lease，并由Session持有到读者释放或新根安装，不在restore尾过早释放。

每个current pointer拥有独立根项目，kind为**实际pointer字节hash**。publisher在pointer锁内先清理不匹配现current的已崩溃代，候选只以lease落盘；metadata/晚到/cancel等前门完成后登记新根，再末次重验/原子switch。登记后撤权、取消或replace失败显式移除新根；相同已接受根不误撤。switch后安装同候选并转交root lease，退休旧kind；旧读者自己的lease仍保护旧required闭包。未知current字节没有可识别根时fail closed，不能因外部坏pointer清空仍可恢复的旧根；路径键不case-fold，以免Windows大小写敏感目录的不同文件共用退休域。

提交后的根退休是单独、可重试的`PublicationMaintenance`，若失败明确保留`cleanup_error`，不会把已改变current的提交谎报为“候选被拒且旧根未变”；正式worker会将该清理待办报告为`store.cleanup-pending`，不出certify成功。下次publisher或显式retry按实际pointer重试。永久控制在第三次资格检查后将原store连接query_only置ON，验证source/target/pointer确已提交、维护错误可见，恢复连接写权限后retry退休1个旧根并实际运行44。此维护状态与parent oracle PASS分层；全证书/全部reader仍属T-10。

验证（所有进程TEMP/TMP为938/test-temp，当前无live sessions）：

- 新artifacts conformance注册case `leased_put_and_owned_root_retirement_preserve_readers_and_other_projects` 最终exit0，1/1、22过滤，编译0.72s/运行0.67s。原生lease存活时GC不删、外项目hard root不被退休；去外root后活跃旧lease仍保留；drop旧lease后真实GC `ran/deleted_chunks/freed_bytes > 0`，当前root在最后读者释放后仍存活。首红是新调用把`remove_root(project,kind,artifact)`后两参颠倒，实际旧root未移除；同因production rollback与控制一起修正，并显式核removed=1，未删GC断言。首红编译2.36s/运行0.42s。
- publication两个用例最终run `cfcc38b9-7b2b-4614-9b8e-70ab5b79ff9b`，exit0，2/2、111过滤，编译2.82s/运行55.573s（38.110+17.462）。故障矩阵新增登记后撤权/取消，并对每个失败断言owned root恰1且仍为旧id；成功替换后旧根确退休。新双端实际子进程在新根登记后、pointer switch前exit73，跳过析构；父持有旧current/旧读者，retry回收孤根、GC真实删其块；随后发布43、GC期间旧读者仍运行42；释放旧读者后GC回收旧根而新根可读，用户source/exe哨兵不变；含上述维护错误/修复控制。
- 最后仅修路径case-fold与一次性project身份计算，相关crash/reader/维护完整用例重放：run `21a2c173-2e69-4280-a2da-03204710b483`，exit0，1/1、112过滤，编译3.19s/运行22.628s。更早两个case中间run `cccbc5ee-447a-4464-a2da-31802ec2cb53` 2/2、52.878s也绿，不替代最终反控。
- publisher持久/lease接缝还覆盖5个相邻路径：run `d1cf302b-7274-4284-a633-c4dc4e818f13`，exit0，5/5、108过滤，准备0.15s/运行39.131s，正式provider-only、历史/A2X命中、真实body-first process、真实声明/源同时repair及非读者payload IO0均保。ABI共113/source19；没有声称重跑全部113或19。8c未变interface/source测试97/97及全18证据按输入/路径复用，冻结固定职责并集仍pending。
- 初次ABI test编译遗漏新helper的publication re-export，编译exit101（尚未执行case），补export后最终均绿。cargo check exit0；定向rustfmt、diff/cached diff --check及pre-commit通过，仍仅原AV三条warning，无新增warning。

开放边界：真实exit73会留下`current.tmp-<childPID>-1`，**当前根维护只处理CAS根，尚未处理这类崩溃指针临时文件**；例如938/tests/root-reader-gc-43208/{Native,C}/下仍有该材料，均在统一scope内，不把fixture teardown冒production cleanup。下一单元须补可证明归属的scratch/重启清理，不能仅按后缀盲删用户文件。常规失败Temp析构控制已通过。完整cold/候选/接口新binding/query/HIR/native/A2X/ledger索引预留、source/native原更严帽、128MiB请求与namespace、ABI hot8→6、512修订及正式父Job/原6×30/P90还未关；T-10全证书/daily无spawn/多fixture/readers/SDK/旧桥/export、T-11 Q4/SD/939保留。根维护增加的M/lease/GC明细须随完整worker ledger/certificate准确接入，不因此核销T-07..11。旧input-changed gate red不改写。

2026-11-06 work checkpoint 8e（同round 8继续，6/13 executing，非handoff）：提交 `d35ad415fcc5f1e03cd170a50188beac8d26b6f9`，6文件462+/67-，实现树clean。

关闭8d新增的**新协议崩溃指针副本清理缺口**：指针临时件改放原ManagedStore的`staging/<pointer-project>/pointer-<PID>-<seq>.tmp`，原store marker/独立pointer项目及本子目录固定owner marker共同限定清理域，不在用户输出目录按文件名搜索。原store新增`StagingTurn`复用同一shared liveness→exclusive staging-budget锁；清理可在已超限时入turn，但每次新物理写入前都按原8MiB cap核全部现存staging+CAS tmp+新副本，含其他producer/崩溃残留；GC与其他staging producer不穿过该回合。候选root put与pointer staging分回合，避免递归抢budget锁。正常/取消/资格失效/replace失败显式清理副本，失败清理具名返回；真实exit73后retry先清本域副本，再退休孤根，记临时文件数/字节。部分合法marker前缀仅在无其他条目时修复，未知marker拒绝且保留旧pointer；reparse、未知文件名/非文件拒绝或保留，不创建任何链接。旧8d在parent输出的legacy tmp只作为已登记938原始材料保留至merge全scope删除，不把后缀扫描作为迁移依据。

临时副本持有Windows deny-WRITE/allow-DELETE句柄到原子rename，末次宿主回调无法改写已经hash并登记的pointer字节；Vec按实际capacity记共享I/O份额，并由Temp自身持有至释放，字段顺序确保先销毁payload再退还预留。新副本与current pointer须同卷；正式worker把两者放同一store根。MoveFileEx没有COPY_ALLOWED，异卷是受控replace拒绝，本单元未计跨卷成功覆盖。所有stage仍属于原ManagedStore并进入其space/GC口径，无第二个DB或外置未计临时树。

验证（TEMP/TMP均938/test-temp，当前无live sessions）：

- 最终`cargo test --manifest-path experimental/ace/Cargo.toml -p ace-artifacts --test conformance leased_publication:: --message-format short` exit0，2/2、22过滤，编译1.95s/运行0.69s。新增原staging turn压力：写前准入、现存恰8MiB再写1B拒、外部超帽残留时仍可入维护turn但grant拒、清残留后lease-backed put继续；原root/reader/GC控制保留。中间同2项也绿（0.85s），不替代最终分母。
- Source ABI相关3项（正式provider-only、全部publication_faults、真实双端崩溃/reader/GC）run `32f0171b-ae79-4a47-9a1a-ccbbcdb79aae` exit0，3/3、110过滤，编译4.14s/运行86.103s。含新协议staging1文件→retry归零、removed bytes>0；用户parent相似`.tmp`哨兵与本域未知名字保留；登记后host回调真实改写临时文件被OS拒；清理readonly故障/修复仍绿。
- 随后仅补部分marker初始化/未知marker反控及payload所有权释放顺序，相关完整crash/GC/维护case最终重放：run `d6294fa1-78cb-4964-8274-5792e8a0e49c` exit0，1/1、112过滤，编译10.06s/运行23.994s；前一marker单项run `69bd0a6e-c4f6-45c3-aacc-bc99c56009f2` 1/1、22.348s也绿。更早新staging双case `45dad667-2623-46bb-ac6a-4035d9d2a2e5` 2/2、60.409s保中间记录。没有借元数据小修重跑8c未变物理调用/全scope压力。
- 初次check有Result错误类型推断缺失与移除旧writer后unused Write，编译红已修，后续check exit0/全部测试无新增warning，仅原AV三条；定向rustfmt、diff/cached diff --check、pre-commit通过。两次patch因上下文不匹配被工具原子拒绝后按实际内容重写，没有留半应用代码。

继续本round：T-09剩余完整内存端口（cold/new接口binding、query/声明/ledger索引、候选source/HIR、native MIR/decode/codegen、原A2X读写与并存）、source/native原更严帽、128MiB request/namespace、ABI hot8→6/512修订/正式父Job/P90；worker run/reference工具输出目录生命周期也待关闭。generic encode_pointer宿主回调的Vec目前在移交到Temp时收费，完整集成者仍需在其编码前预留，不能据本单元称所有JSON/copy端口已先reserve。T-07新M/lease/GC/staging明细与完整证书/全部reader接线随T-10核对；T-10多fixture/daily无spawn/SDK/旧桥/export与T-11原样本/Q4/SD/939仍未完成。最终冻结职责并集pending、旧input-changed gate red保留。

2026-11-06 work checkpoint 8f（同round 8继续，仍6/13 executing，非handoff）：提交 `c297a2f1ba00e2a1da0324aae07364183840d46d`，7文件427+/9-，实现树clean。

本单元核实并修复三个实际缺口。① `MirModule::heap_estimate`原先通过`serde_json::to_vec(SourceDecl)`估内存，在R深验中隐形重编码；真实allocator控制观察到3次分配。现改`SourceDecl::heap_bytes`按String/Vec实际capacity及error/status/context/semantic-ref等所有owned字段直接计量，统计本身零分配，spare capacity不漏。② 新foreign声明原本内嵌MirInstr，把**所有**标量指令扩到448B，侵蚀旧16MiB MIR份额；改为boxed声明，逐声明显式记box+owned数据，永久门保证指令inline≤64B，wire JSON/AVFI格式不变。lowering/decoder两个构造点同步，native所有真实consumer编译与运行验证。

③ ABC foreign声明decode先按有限metadata wire估typed模型、parser scratch及validation并存，再在原累计16MiB decoder预算内入Deserialize；args及boxed输出逐项计入，函数名也改在分配前记账。foreign encode改用有界JSON前门。把原AVFI u8参数宽度前移到manual/source声明准入：最多255逻辑参数，256在plan producer之前具名`foreign.signature`拒，避免known unsupported拖到native emit后的wire编码；255仅作candidate shape正控，不冒255参真实C调用授权/执行。

永久新target `ace-av-core::foreign_resources` 已显式Cargo注册，原固定ace-av组的`-p ace-av-core --lib --tests`覆盖，nextest实际list选出4项；不建新固定组。测试allocator为该独立target的System wrapper，按线程测请求分配，peak保守包含realloc旧/新可能重叠，**不是OS private-bytes或父Job测量**。2个90000项checked-effect声明的合法codec/verify控制请求峰10,343,869B；4项压力wire仍<原4MiB输入帽，production decoder在14,120,393B内资源拒（原帽16,777,216B）。针对实际producer临时关闭新预留检查的负控，decoder错误接受并升到17,896,837B，永久断言真实红；finally恢复源码，前后SHA256均`e2ff261ee4b064d2433a2e0e088c12a58a350454baa48f4777b94ec1a28939ec`，最终代码没有关闭guard。未缩样本/改帽/删断言。

验证终态（TEMP/TMP显式938/test-temp，无live sessions）：

- `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-av-core --lib --tests --message-format short` exit0，59 lib+44既有公开closures/dynamic+4新资源=107/107、0ignored/filtered，编译1.31s，运行0.09+0.11+0.58s。因MirInstr布局/heap估算触及所有既有变体，故完整选该包的原模块职责；无普通ACE全workspace扩测。首次该命令58/59红，发现T-04新增FOREIGN_CALL已正确标traps但旧trap分类expected列表遗漏；补该reader expected及语义说明（status/bool可失败，AV仍unsupported），未把traps改false，随后107全绿。另新测试曾借用类型写错导致编译红，修为verify_mir(&module)。
- 初始无分配反例exit101：测量3次分配（编译3.91s/运行0.00）；修后绿。inline预算反例随后1/2红，观测448B（0.50s/0.00），boxing后2/2绿。实际关闭guard反控exit101，编译1.14s/运行0.60s，错误接受与峰值如上；恢复后全部107通过。单项正常压力也exit0（1.10s/0.55s）。这些红均保留，不归因环境。
- `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-foreign-contract -p ace-source-query --lib --message-format short` exit0，36+62=98/98，0排除，编译5.59s，执行0.00+0.07s。含新255/256前门控制，保全部共享query/恢复/失败事务。`cargo check -p ace-abi` exit0，完整依赖读者均构建；仅原AV三条warning，无新增。
- ABI source实链5项run `46a6a75f-a635-41f2-982b-65ed4802fde6` exit0，5/5、108过滤，编译10.53s/运行49.927s：原20次bool/status/ctx/顺序/短路/混合槽实际C调用、Native/C历史与原A2X命中、真实process首编辑、真实声明+source repair发布、required逐缺/逐坏均绿。未改的8d/e publisher故障与GC证据复用，不重复全部source19或ABI113。
- nextest `list -p ace-av-core --lib --tests ... -E 'binary(foreign_resources)'` exit0实际4 selected（准备0.09s），list不冒执行；4项已在107中实际运行。定向rustfmt、diff/cached diff --check及pre-commit通过，只有git提示instr_spec工作副本换行规范化。

继续本round：当前新foreign decoder尊重自身原16MiB帽，**尚未把其完整typed MIR/临时峰与Session共享48MiB/16/24/8预留一体连接**；旧decoder capabilities/header/Vec增长等前门仍需按实际owner核查，不能据本局部foreign保护宣布T-09完成。native lower/codegen/ABC copies、cold/new接口与binding、query/catalog/ledger元数据、原A2X读写、pointer encoder预留、source/native原更严source帽、128MiB request/namespace与hot8→6/512修订/全部工具scratch及正式样本继续开放。T-10完整多fixture/证书/全部reader/SDK/daily纯emit无spawn/旧797三腿与937 export、T-11原性能/Q4/SD/939也未关；最终冻结固定职责并集pending，旧input-changed gate red保留。

2026-11-06 work checkpoint 8g（同round 8继续，仍6/13 executing，非handoff）：提交 `cfa92eeffa5c0f9322befd708c479d9a45f273c4`，1文件153+/5-，实现树clean。

修复cold接口声明展开的先分配后拒绝：合法小于原1MiB的manual manifest可把共享provider/逻辑/审定字段复制到每个函数，展开声明页才超帽。现以借用字段、流式logical join及过滤effects精确计算完整声明页与digest map的JSON字节，在physical/source projection前拒绝超帽；不克隆投影模型、不改变原1MiB或函数/签名上限。canonical仅重排序，故在其clone/sort前先测原输入精确wire大小；实际声明页编码借用Vec/map，去掉整份声明与digest副本；effects过滤不再逐项format临时String。此为局部输出前门，未冒充全部cold typed/index共享预算闭环。

永久正控32函数含转义/Unicode provider、重复checked/capability，预估页大小等于实际编码大小，required restore成功；同32函数放大共享provider、manifest仍<1MiB，正式入口在任何plan/source producer前foreign.budget拒绝。实际去掉新guard的负控exit101，观察到错误进入foreign-plan-project；finally恢复文件，前后SHA256同`9c2e0576cdbde59956b476bea6af801bad7f142e8311abc1c969eb9098a99c16`。未缩输入/放帽。

最终`cargo test --manifest-path experimental/ace/Cargo.toml -p ace-foreign-contract -p ace-source-query --lib --message-format short` exit0：37+62=99/99、0排除，编译9.13s，执行0.04+0.07s。ABI相关2项run `9ef922d2-aed7-43b8-98a4-a3e6e62a4f0e` exit0，2/2、111过滤，编译12.68s/运行16.180s，正式provider-only15.882s及required缺/坏页0.297s；未变codegen/发布故障/GC证据复用。TEMP/TMP全为938/test-temp，无live sessions。rustfmt、diff/cached diff --check、pre-commit通过；仍原AV三条warning。原始日志在唯一938 scope的8g-*.log，merge折叠后全删。

继续：MIR decoder共享48/16/24/8前门与旧header/Vec并存、其余query/HIR/ledger/native/A2X与cold/input预算、128MiB request/namespace、hot8→6/512修订和工具scratch仍开放；T-10完整认证/daily无spawn/reader与SDK/桥/export以及T-11原样本/P90/Q4/SD/939未关。最终冻结职责并集pending，旧input-changed red不改写。
2026-11-06 work checkpoint 8h（仍goal round8在途work，6/13 executing，非handoff）：提交 `cfaed8aadec10315039762c8a2a2682b1dabb0e5`，10文件569+/274-，实现树clean。

恢复target与S938F内存/磁盘命中三个真实consumer现共用Session Resources：原子预留min(MIR剩余份额、48MiB合计剩余)，ABC limited入口只可收紧原16MiB，不另给独立16MiB。宿主新增restore_host_with_resources组合端口，供request decode与恢复持有同一账号；默认入口仍建原默认账号。临时decoded MIR先销毁再释放预留。实际pressure控制持有16MiB−64旧MIR额度：required其他页可读、在MIR header深验资源拒；失败不改旧snapshot，MIR/io无泄漏；释放旧占用，同一Resources修复恢复并实际C provider运行42。实际把consumer改回默认decode的反控run `10f7037b-8d94-4f91-948d-4a0982d0b616` exit100（编译6.64s/运行0.238s），错误接受触发永久断言；finally恢复source_foreign.rs，前后SHA256同`9fc38fbb1016ce82f530e8f37c9110a632eec6d6b7fd0169adf4f215ed27c755`。

ABC原header quota/decoded guard前移到capabilities和host-import分配前；capability字符串、host记录/字段、owner/profile、普通call args、closure签名/captures/args/env声明均在分配前计量。所有已声明长度且经guard的Vec精确预留，不靠几何增长而漏旧新并存。函数直接建最终table，sidecar直接移交所属函数，去掉raw_funcs/snapshot/span_tables与span全表clone；sidecar内部重算checksum仍带尾字节的伪页具名拒。既有wire版本/原4MiB输入、16MiB decode、表/arity帽不变。

MIR只读深验与拥有凭证构造共用同一Verifier；新增validate_mir仅返回统计、不给执行凭证，原verify_mir仍是CheckedMir唯一构造且保原owning API。native foreign admission改用只读门，避免为判定复制整个MIR。资源反例最初5/6绿、只读压力仍分配2,790,558B，定位SourceDecl.validate为构造InterfaceFacts克隆整份effects/caps；改共享borrowed validate_functions，借用symbol集合，error code排序/唯一检验改windows扫描，保原拒绝优先次序和错误文本。最终同输入只读深验峰202B、拥有凭证对照5,582,069B；坏return引用两门错误完全一致。60,000项sidecar decode峰961,575B，对象估962,206B，不再双份；foreign原90,000 effects×2声明合法控制峰8,954,753B，×4拒峰12,731,277B（仍原16,777,216B帽）。这些是线程System allocator请求字节峰（含realloc并存），不是父Job/OS private bytes。MirInstr实测inline40B。

最终验证（TEMP/TMP938/test-temp，无live sessions）：

- `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-av-core -p ace-foreign-contract -p ace-source-query --lib --tests --message-format short` exit0，216/216：59 AV lib+44旧公开target+6资源、37 contract lib+3旧conformance、63 source lib+2 restore929+2 parity934；0条件排除。编译6.12s；分target执行0.10/0.11/0.52/0.04/0.00/0.09/0.01/0.05s。新增capability头32KiB前拒峰<4KiB、call table128KiB内拒、host超大limit不能扩原16MiB、共享pool双约束/释放，以及sidecar与只读判定实际峰正反控。初始107旧模块绿；中途5/6红修复如上；最终216全绿。
- 原ABC与旧data/collections/dynamic/closures格式consumer选38项，run `1479bd0b-b148-4c44-8469-cb20265f8d9e` exit0，38/38、856过滤，编译14.84s/运行0.386s。包含core/unit byte稳定、格式/坏页/credential、实际标量trap/顺序、原缓存格式隔离/恢复；非完整AV894重跑。此test binary另暴露4个原测试warning（closures_recovery913 mut put、method_incremental edited、method_perf s2、owned_first_case FIRST_CASE_INT_TWIN），各源相对基面无本计划修改，按既有owner保留，不当新增实现warning；原AV lib3条仍在案。
- 最终source ABI6项run `829f0159-0b68-40a3-b091-ae34179851bf` exit0，6/6、108过滤，编译6.40s/运行56.080s：20次双端status/bool/ctx/次序/短路/mixed26.944s、history4.363s、真实process R首编辑6.023s、真实source/签名repair16.962s、required缺/坏页0.277s、新共享MIR压力+修复1.509s。新压力初始单项run3234c558也绿（编译18.53s/2.041s）。ABI现114/source20；8d/e未变publisher/GC证据复用，未重跑全部114。最终编译无新增warning；负控暂时unused maximum随finally恢复消失。rustfmt、diff/cached diff --check及pre-commit通过。

下一实际开放面：本单元预留覆盖decoded MIR及去掉完整校验副本，**Verifier内部analysis/owner restamp与native lower/codegen/ABC encode等仍须在共享staging/并存额度内先reserve**，不能把原Verifier16MiB独立guard当共享预算。其余cold/查询/HIR/ledger/input/A2X/pointer encoder、source/native更严source帽、128MiB request/namespace、ABI hot8→6、512修订及tool scratch未关。T-10完整certificate/daily emit无spawn/readers/SDK/旧桥/export与T-11原样本/P90/Q4/SD/939仍开放。最终冻结固定职责并集pending、旧input-changed red保留。
2026-11-06 work checkpoint 8i（同goal round8继续，仍6/13 executing，非handoff）：提交 `75c55ab5c334a87755777d8d419a7288ced31458`，9文件556+/38-，实现树clean。

正式worker deck入口现先核原4MiB transport、预留Base64输出I/O，再按新finite request profile预检typed JSON/index及parser scratch，最后才进Deck Deserialize。request profile分source text与structured manifest facts（不能套source-session的byte-valued facts），typed model票据持有至请求结束；cold Frontend与stored/current restore接收同一Resources，未另建48MiB池。原restore默认API保兼容，新增with_resources组合端口；实际stored R也用旧MIR占用16MiB−64反控拒、释放后同账号修复，保原lease与当前authority重验。

补原native4MiB source cap AND 中立Frontend8MiB：cold/cold_stored在provider对象准入与parse前检查，已有Frontend组合入口、warm全source/单owner replacement/声明+source事务与R-before-target一致检查；Frontend新增仅目录/span元数据的replacement_source_bytes，无warm整模块parse或源码复制。C保中立原8MiB，未改全局限额。实际双端冷超帽用空provider作先后顺序反控，warm三个入口全部拒且parse/query计数与旧snapshot不变；小body修复实际运行43。4MiB/8MiB exact边界在同一判定函数作metadata正控，不把它称为4/8MiB完整编译成功负载。

父decoder补“源码生产前resource拒绝”的严格判定：此时可以没有尚未解码/处理的source摘要，但外层nonce/request-hash仍绑定输入；要求所有source/solver/HIR/MIR/native/C生产为0、无provider/tool资格化声称、parse阶段foreign.resource且无伪span，原report深验继续保证无actual/reference执行证据；拒绝携带exit、改称diags/ok、假source digest/生产/工具/调用均拒。不是丢失source identity后仍发PASS；resource仍是typed失败，原成功oracle不被绕过。wire字段/schema未改，常规有source的拒绝与成功仍核当前source hash。worker资源步明确outcome=resource/cache=-/rejections=1，不谎报hit或编译diags。

验证（TEMP/TMP938/test-temp，无live sessions）：

- `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-foreign-contract -p ace-source-query --lib --message-format short` exit0，38+63=101/101、0条件排除，编译4.15s/执行0.06+0.91s。profile mixed text+facts正控、原source/metadata profile不混用；generic真实Deserialize counter在收紧index下未进入，正常request可decode，错误/票据释放均核。新增worker lib两项最终exit0（1.24s编译/0.00s运行、21过滤），原ace-abi固定组含--lib --tests，正式可选到这些case。
- 实际把worker模型decode改成独立Resources::default的负控exit101（编译1.29s/执行0.00s），永久断言检测当前request index未持有typed模型；finally恢复worker_source.rs前后SHA256同`cb92065bdb4f4983e9d260577ccb4ff965bad62819af67641e43e061959eb137`，最终两项再绿。保正常manifest、未知safe字段、非法Base64、index/I/O先拒及释放控制。
- root父reader定向3项run `cf1c9784-1916-470f-9ac7-5f3d453c7679` exit0，3/3、1628过滤，准备/编译75s、运行0.041s；包括所有原source report/当前诊断反控与新增resource不能替代source/执行证据。root lib test有145条既有warning（未指向本次abi/abi_source修改），不声称零warning，冻结职责审查时按原owner/债核对；原AV lib3条仍在案。
- ABI本单元5项初始run `aed3f42d-3578-466d-901c-6ea6f2d2f0ee` exit100，4/5 PASS、111过滤，编译4.51s/运行50.999s：formal首错repair18.838s、provider-only24.667s、共享MIR及stored R压力1.726s、Native/C source caps与repair3.102s均绿；request压力最初20,000 effects仅约1.24MiB index，未越8MiB，正确走独立registry拒，测试误期望resource而红。按真实预算把压力增至200,000（没有放帽/减样本）；直接preflight先核metadata>8MiB、wire<3MiB，再发实际worker。
- 最终仅该压力case重放run `256509c9-35dd-49f1-9070-861b71907620` exit0，1/1、115过滤，编译1.75s/运行12.899s；Native/C均wire=2,001,041B、metadata=12,409,111B、index_cap=8,388,608B，resource步compute0/no provider/no actual，旧pointer逐字不变，后续独立进程合法R query actual42且source parse/solver0。其余4项代码/输入不变复用，不重跑5项。ABI现116/source22。cargo check exit0（7.42s），仅原AV3警告；rustfmt、diff/cached diff --check、pre-commit通过。

开放：请求raw IPC自身的既有有界transport与本单元deck模型不是整个编译峰证明；cold typed接口/新binding、所有query/catalog/HIR/ledger元数据、Verifier analysis/owner restamp/native lower/codegen/ABC encode/原A2X及pointer encoder复制仍须原AND共享预留；128MiB request/namespace、hot8→6、512修订及全部tool scratch保留。T-10完整多fixture certificate/daily无spawn/全部reader与SDK/旧桥/export、T-11原6×30/原负载/P90/Q4/SD/939未关，最终冻结职责并集pending；旧input-changed red不改写。
2026-11-06 work checkpoint 8j（同goal round8在途work，6/13 executing，非handoff）：提交 `0a4be16d9577088557f779ccf49a05caf7bd8414`，4文件235+/28-，实现树clean。

关闭native最终COFF Vec的分配前门：使用当前已锁Cranelift re-export的同一object::write WritableBuffer（非新序列化器/依赖），真实COFF writer在写任何buffer内容前一次提供精确输出size。新buffer先核原16MiB COFF帽、取得同一Resources staging票据，再try_reserve_exact；resize/write不允许越过声明长度，返回时核完整填充；失败字段按payload先于票据销毁。legacy emit也先核原16MiB再分配，成功字节与原object.write逐字相同；没有扩原codec/对象帽。原producer增加emit_foreign_unit_coff_reserved端口，source组合实际消费，并把同一票据在TargetWire交接时重分类/扩为decoded resident，最终绑定不可变Artifact Arc，避免COFF裸Vec在缓存交接前无计费或重复创建第二个COFF票据。

这仅证明**最终COFF owned output buffer及其交接**，不冒充Cranelift/context/内部section/string-table/index全部峰值或完整native lower/Verifier/ABC资源闭环。ManagedLedger原本是事后证据，不是guard；其余实际owner前门仍须继续。

验证（TEMP/TMP均938/test-temp，当前无live sessions；末次CIM未发现匹配938/source_foreign的cargo/nextest/worker进程）：

- native真实object writer正控4,096B text→4,160B COFF，与原object.write逐字相等且capacity=len，票据在bytes存活期占用4,160B staging；收紧staging4,096B时before output allocation具名managed-memory拒，份额不泄漏；16MiB text+COFF header在无host账号时也按原16MiB object-size拒。初次fixture试图对已有section重复set_data触发库assert，改用独立oversized object（测试设置错误），未改production门。初红编译6.32s；修后单项绿1.35s/0.00s。
- 实际去掉writer共享reservation入口的负控exit101（编译1.64s/运行0.00s），永久断言检测应持有4,160B却为0；finally恢复coff_buffer.rs前后SHA256同`5edbcfa979b04ec98b1c41d21cd9f5723c953d1c6e5534be4b316b5fdff607f7`。最终`cargo test --manifest-path experimental/ace/Cargo.toml -p ace-native --lib --message-format short` exit0，12/12、0排除，编译1.90s/运行0.06s，覆盖新buffer及原profile/admission/exact COFF metadata/packed MIR/失败发布控制。
- 共享legacy emitter的原完整codegen模块6项run `8f8b9cbe-dde8-4bca-8d5c-1d0586de0282` exit0，6/6、184过滤，编译8.59s/运行3.860s：原arithmetic/trap、lazy branch、loop/break/continue、call shapes、trace order及bad MIR/codegen0；既有native conformance另1条closures_restore_tier close Result未用warning，源未被本计划修改，按原owner保留，未静默suppress。原AV3条仍在案。
- source ABI6项run `05789567-881f-4b90-97dc-e56955adec97` exit0，6/6、111过滤，编译11.73s/运行134.358s：真实20次双端physical矩阵82.809s（明显较前次慢，原样保留，未减case/强杀/用重放覆盖）、正式provider-only30.411s、history6.054s、新native output压力/修复0.301s、真实process R首编辑14.253s、required缺/坏页0.524s。新控制从真实source产物的MIR深验后调用同一reserved native emitter：共享24MiB staging全部在途则输出拒、旧snapshot不变；释放后COFF与原缓存字节相等、票据持有/释放精确。ABI现117/source23；未变8i input/source-cap/父decoder与8d/e publication/GC证据复用，不扩为全117。**本134秒功能回归不是原6×30/P90性能样本，也未环境归因豁免性能门**。
- rustfmt/diff/cached diff --check/pre-commit通过；最终无新增warning，负控临时unused资源字段warning随finally恢复消失。所有原始8j日志在唯一938 scope，merge折叠后全删。

继续：Verifier workspace/owner restamp、HIR→MIR lower及其元数据/符号、ABC encode/copies、Cranelift与原A2X实际managed owner、cold/new binding/查询/HIR/ledger/index及pointer encoder预算仍开放；request128/namespace128、ABI hot8→6/512修订及所有tool scratch生命周期、完整正式Job峰/I/O仍未关闭。T-10完整多fixture证书/daily纯emit无spawn/全部readers与SDK/旧797三腿/937 export，T-11原采样/P90/Q4/SD/939仍待完成，最终冻结职责并集pending、旧input-changed red保留。
2026-11-06 work checkpoint 8k（仍同goal round8在途work，6/13 executing，非handoff）：提交 `87e9ea378bdc1b0134dc1d4742cb48c49e72b016`，5文件445+/39-，实现树clean。

ABC编码新增中立reserve callback：先借用扫描诊断/单函数validate暂存上界，再同一encoder的count-only通道精确计wire，取得分页+扁平并存额度后才物化输出。count通道不建页/JSON/sidecar副本；真实页与页索引按精确长度try_reserve_exact，失败具名，不以Vec几何增长隐含旧新复制。sidecar逐字段直接写目标页，不先建整份sc Vec。4MiB为原decoder input帽，encoder现在在物化前拒绝注定无法R的>4MiB wire，原256KiB分页和所有版本/位布局保留。foreign JSON count路径仅json_size，实际生产仍用原bounded JSON并深验，不把预检冒一次持久化输出。

source组合真实消费同一Resources：ABC预检与页/flat并存先记staging，flat接收后销毁pages再收紧ticket；与8j COFF票据在同一账号原子merge，随后同一TargetWire/Artifact转resident与弱身份持有，不先释放再重新申请。不同账号merge受控拒且两票据安全释放。诊断预留仅按真正可被格式化的borrowed name最大值计（含formatter重分配及小validation集合），不把整个MIR误算为第二份诊断副本；永久3份90,000 effects的大合法紧凑wire在原24MiB staging内仍可编码，防过度拒绝。完整lower/Verifier等其他owner仍未由本单元核销。

真正旧producer字节基准：临时载入上一已提交8j的abc.rs，先核Git blob严格等于`cad460d2c4111580c8661bf59eaa0bd7cfda7850`再运行原编码器；scalar/foreign各带60,000 span分页sidecar，FNV完整wire分别`6788358879573569296`/`15294436905299535179`。初始0占位校准exit101（2.22s编译/0.02s运行），将两已观测旧值固定为永久门，新encoder均逐hash相等。finally恢复新代码，前后SHA256同`cd827ad47ec41dbcde1db925fcc7026559646efb828c65e6694111a80befe5d6`，不是仅以新encode/decode自证旧兼容。

验证（TEMP/TMP938/test-temp，无live sessions）：

- `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-av-core -p ace-foreign-contract -p ace-source-query --lib --tests --message-format short` exit0，220/220、0排除：AV59+44+8，contract38+3，source64+2+2；准备/编译7.54s（含已有package cache锁等待），分target运行0.09/0.11/0.66/0.04/0.00/0.08/0.01/0.05s。单独资源target8/8也绿（2.19s编译/0.66s执行）。960,668B wire真实页+flat请求峰1,921,456B，预留1,925,600B；各页capacity=len；callback先拒时只有其自身1次错误String分配、无payload；超4MiB sidecar在<4KiB临时内拒，未输出超帽页；大合法紧凑wire正控及同账号merge/外部pin/不同账号拒保留。
- 实际去掉初始preflight reserve负控exit101（3.92s编译/0.01s运行），断言观察到4次而非仅错误String1次分配，证明预留前已进入真实校验分配；finally恢复前后SHA256同`95931ef6bae50fdfe283160c0902be482996e64c9fd3e723f19a8fd0427f73f2`。
- 实际source候选反控：额度足以先产真实facts和COFF，余量不足ABC诊断/编码前门；正式实现foreign.mir-encode/CodecPage拒，native_produced=1、page_encodes=0、staging只余原外部占用、旧snapshot不变；释放后同编辑提交并实际运行43。临时把consumer callback改为不预留，仅保留事后flat resize的负控run `731abb56-66ec-4116-b8c9-e85a0de59d8f` exit100（3.49s编译/0.460s运行），已分配801B flat才在余410B处晚拒，永久断言区分这个错误阶段；finally恢复source_foreign.rs SHA256同`ea31ca15e06fa5be3f5c85ed95d8f4279a1d6cd7e6c2208d566ebf3fbb302188`。最终该完整组合case单项run `ce41f1fc-5058-4e2f-a40a-8df672a6c134` exit0（16.76s编译/8.469s运行）。
- 旧ABC/core unit/data/collections/dynamic/closures reader范围38项run `975618ff-dec2-490a-81c7-6f86a18c15f4` exit0，38/38、856过滤，编译10.42s/运行0.512s；保原格式隔离、坏页、执行credential、标量业务/原cache语义。
- 最终source ABI6项run `d9771b79-060b-48ea-85fa-b768533cb2a6` exit0，6/6、111过滤，编译2.94s/运行114.171s：20次双端physical45.152s、正式provider-only44.552s、history14.412s、真实native输出/ABC压力+修复2.069s、真实process R首编辑7.594s、required缺/坏页0.390s。未用更快重放覆盖8j慢结果，也不冒正式原6×30/P90采样。ABI仍117/source23，未变8i request/父decoder及8d/e publisher/GC证据复用。
- cargo check exit0（7.04s）；定向rustfmt、diff/cached diff --check、pre-commit通过，最终无新增warning，原AV3/旧conformance4仍在案；负控unused helper/bytes警告均随finally恢复。原始8k日志仍统一938 scope，merge折叠后全删。

下一入口：MIR lower/符号与Verifier workspace/owner restamp的完整共享预留，cold接口/binding与查询/catalog/HIR/ledger/digest/index、原A2X及pointer encoder owner仍开放；请求128MiB/namespace128MiB、ABI hot8→6/512修订、全部tool scratch生命周期与正式父Job峰/I/O保留。T-10完整多fixture证书/daily无spawn/全部reader与SDK/旧797三腿/937 export、T-11原采样/P90/Q4/SD/939未关；最终冻结职责并集pending，旧input-changed red保留。
2026-11-06 work checkpoint 8l（同一goal round8继续，6/13 executing，非handoff）：内部同步提交 `38f8960bbb84d9eaa770ad2e9cf9852412f1501d`，合入固定master `204904312de710ff958c41cc8a27738197e7c0b2`；非向master交付。同步935实例、950有限模块、943/944语义契约与双消费者装配。5个文本冲突逐项合并：保留foreign签名/逐声明版本与声明context；保留原query speculative不可变cache roots及累计counters；A2X唯一fragment实现同时接context和observer，原key/context与真实producer均保留；新模块候选补foreign上下文。935新增type_params在借用SignatureView接入（具体foreign为空），未扩foreign模板表面。同步前后普通模块/装配能力保持，未绕过产品TypeSupply。

新增永久foreign+declaration-context+speculative-cache控制通过：只改实际reader声明context，非读者命中；恢复cache根后禁solver仍命中，实际工作counter不回滚。上游module恢复计数有一处仍期待旧逻辑typecheck=1，改为真实TypeSupply路径typecheck=0；原actual_solver_runs=1断言保留，未删除验证。

定向验证：`cargo check -p ace-abi` exit0（最终7.04s；初次重复serde依赖、候选foreign字段和SignatureView类型集成错误均已修复，原日志保留）。source-query/foreign-contract/semantics `--lib --tests` 最终176/176；core lib+conformance 370通过、原60秒专项1 ignored明确排除，编译30.76s/运行7.18+2.55s；a2x-store lib1/1；AV assembly_944 5/5（含双消费者原子发布/失败保旧/独立首编辑，编译23.97s/运行3.54s）；stdlib真实reference→两消费者manifest集成1/1（冷编译148s/运行2.15s，根core原131 warning与AV3/4 warning归原路径，未新加warning抑制）。

源码ABI本次全部23项均实际执行。首次命令误用nextest `--message-format short` 退出2，无测试执行；修正后漏传显式nextest配置，发生20路并发：run `c63444c9-542e-4dc1-936e-5bf42b907b94` 22通过、1个真实子进程30秒Deadline失败（203.942s），此红保留。改用仓原 `.config/nextest-full.toml` 的source_foreign独占规则，仅重放该失败用例，run `7d0bcaf0-8fcd-4f17-9cb2-72340c95b0a3` 1/1通过（35.405s，编译0.21s），原截止不改；未把首次全局exit100改写为绿。其余22项证据复用，所有23项功能有绿证据；这不是正式原6×30/P90。marker/冲突/空白检查和提交钩子通过；同步顺手去掉上游935文件末尾一个多余空行。该内部merge完成后tracked clean；下一单元未接线的source_store.rs为本任务自有WIP，未混入同步提交。

继续T-07..11：先关闭原ABI hot8→6/原collector接线与真实GC/在途lease，再收敛请求128/namespace128及工具scratch、其余明确未关闭的内存owner；完整certificate/daily无spawn/readers/SDK/旧桥/export与原性能/Q4/SD/939均未关闭。最终冻结职责并集pending，旧input-changed gate red仍保留。
2026-11-06 work checkpoint 8m（同goal round8继续，6/13 executing，非handoff）：提交 `bef252499b5c2ddec27310ef10fdb19b0515d0ec`，4文件337+/18-，实现树clean。补此前明确缺少的源码请求hot collector接线，未核销整个T-09。

新增source StorePolicy仅调用原ManagedStore `run_gc`，默认严格保ABI 8MiB高/6MiB低水位，压力/宿主接缝仅可收紧，原required闭包、在途reader lease、元数据收缩和锁顺序仍归原collector，无第二套缓存/清扫算法。正式worker在成功、source diagnostics和前门resource结果的实际出口运行维护，并在原MeterOut报告真实gc_runs/deleted_chunks/freed_bytes与维护后hot total，不再恒填0。GC错误或live floor导致backoff明确返回 `store.cleanup-pending`，文案说明source operation可能已完成；不伪装候选被拒/旧current未变，不删除活产物强凑低水位。其他早期Err仍属于已有显式失败，所有tool scratch生命周期/请求全异常路径尚待后续单元完成。

永久源码控制 `source_foreign_hot_collection_reclaims_revisions_and_preserves_current_and_reader_closures`：native与C各32次真实源码body prepare/commit、snapshot/required页持久与唯一current root替换；1MiB/768KiB收紧水位实跑原collector，实际触发Native14/C20次，累计释放351076/292382B，最终store total320918/432847B。每次GC无backoff且回到低水位，未改independent产物Arc不变；持有初始root真实reader lease跨所有修订，旧reader实际42；当前闭包P0 R恢复、禁隐形solver计数0、立即首编辑实际43。释放old reader后原reclaim-dead模式确实删除旧根，当前root仍可读取。此为源码生命周期定向证据，**不是新增32样本替代原512/6×30/负载**。

实际worker出口控制在默认8→6MiB下注入12MiB真实unrooted块，断言原collector实际跑/删除/释放、hot≤6MiB、current受保护；query_only造成GC写失败显式可见，解除后可重试；再持有9MiB真实live roots，backoff显式失败且三个live roots仍在。实际省略worker collector、用空receipt代替的源码mutation使该永久用例exit101（gc_runs实际0期望1），finally恢复SHA256 `94a4157e8dfe8d8a6a3427fadeb7783a70dcd1c886b76bed3add5bbb9736d952`，恢复用例exit0；之后补强backoff/已完成操作文案再定向绿（编译1.36s/运行1.30s）。无负控留存代码。

验证：source_store/worker_source lib定向4/4，run `8.387s`，0条件排除（21未选）；源码hot控制1/1，run `15f742cc-7a82-4d8b-9d52-433f15ad43bb`，编译3.05s/运行52.075s；原样 `cache_turnaround::performance::abi_cache_512_revision_churn_gc_convergence` 与3个正式worker sibling（前门metadata resource保pointer→修复、恢复首错→修复、provider-only no caller production）共4/4，run `cb638e04-e3b6-4624-b65a-5cddd4588798`，编译3.26s/运行93.219s，其中原512全量21.367s。首编译test old_reader少mut错误已修复，无改变产品语义/门槛。rustfmt/diff/cached diff检查及pre-commit通过，原AV3 warning不变。ABI conformance现118项/source_foreign24项，非宣称全部职责已跑；8l相关输入未变的证据复用，不重跑全部23。

继续：该collector接线/原512回归已完成；请求累计128MiB、namespace128MiB所有实际写口原子预留、tool输出/scratch与所有失败/重启收敛仍开放。其余cold/new binding/query/catalog/HIR/ledger/native lowering+Verifier workspace/A2X/pointer encoder内存owner、完整认证/daily无spawn/全部readers与SDK/旧三腿/export、原6×30/P90/Q4/SD/939均保留。下一单元先核原ManagedStore预算锁内的namespace+framing前门与真实source/A2X/producer调用端，避免仅外围space检查的并发越额。最终冻结职责并集pending、旧input-changed gate red保留。
2026-11-06 work checkpoint 8n（同goal round8继续，仍6/13 executing，非handoff）：提交 `5e3001695a1784189823e5cb8effe358b006b2f7`，4文件230+/7-，实现树clean。原子namespace设计核查先发现两个确定的分配前门缺口，本单元先修这两个共享实际调用口，未把它们冒全namespace闭合。

原ManagedStore `put_inner` 曾在原8MiB staging gate之前调用frame_chunk，先分配完整frame再拒超限。本单元在原liveness→budget锁顺序内用payload长度+固定CHUNK_HEADER_LEN及checked算术先核2份实际framed暂存、现存staging/CAS residue，许可后才分配frame/hash/身份；wire/key、原水位和锁边界保持。新增独立store_resources真Rust全局allocator控制：修复前oversized请求发生1个大frame分配，永久assert期望0失败exit101；修复后超大payload及合法128KiB payload遇在途8MiB residue均0个大frame分配、blocks=0，drain后同一合法payload实际分配frame/写入/深读回精确bytes且staging=0。测试目录带PID+nanos，均在938统一scope。

source FunctionCache实际put之前曾只检查磁盘space，未在共享Resources预留payload.clone+ManagedStore frame等并存IO副本。新增与snapshot leaf一致的3×(payload+header)保守IO ticket，先reserve后clone，保留到真实put返回；失败记录target-cache-put metadata attempt但written payload IO=0、成功才计写入。永久控制真实共享IO仅余64B：修复前put错误地成功，unwrap_err反例exit101；修复后拒绝、磁盘blocks=0/写计数0/无history安装/IO压力未泄漏，释放后原key精确写入一次、原store bytes与resident bytes一致，IO回0。该case使用opaque缓存字节，不宣称实际C执行或新Checked凭证。

验证：原artifact put为所有存储调用共享，故跑整个ace-artifacts包 `--lib --tests` 43/43（0排除），run `a59b8f01-9282-4aac-8666-ec98e544991e`，编译1.38s/运行23.667s；包括原双layout至少128MiB负载、查询/hot/损坏、并发预算、crash residue、GC/真实reader/root与恢复publish控制，规模/条件未改。新frame用例更新唯一目录后再定向1/1（编译0.64s/运行0.04s）。新target-copy预算用例最终1/1（编译1.52s/运行0.03s，含失败attempt断言）。源码history A→B→A/原A2X恢复cache和target discarded/accepted/未改函数payload IO控制2/2，run `874b242a-bcd5-48f4-9772-d66b6e6640e8`，编译3.56s/运行4.905s。之后仅将同一个metadata begin提前到quota/reserve前补记录失败attempt，新单元已重放；正常成功路径次数与其余输入未变，复用这2项证据。rustfmt/diff/cached diff/提交钩子通过，原AV3 warning不变。ABI lib新增1case，conformance仍118/source24；本单元无正式性能/P90采样。

保留边界：原store budget锁仍在CAS后/metadata commit前释放；外围store.space+预测写入检查不能证明并发namespace128MiB。SQLite元数据增长/各种owner及工具目录归属须一并按实际写口闭合，不能随意猜一个metadata余量就称上限已证。请求累计128MiB、全部tool/scratch生命周期；cold/binding/query/HIR/ledger/digest/index、native lowering/Verifier workspace、原A2X观察读写与pointer encoder仍开放。完整T-10/T-11、原6×30/P90/Q4/SD/939、最终冻结职责并集也未关；旧input-changed red仍保留。
2026-11-06 operational checkpoint 8o（同round8 work在途，6/13；code HEAD仍 `5e3001695a1784189823e5cb8effe358b006b2f7`）：D盘耗尽造成下一Verifier改动的apply_patch写失败并将mir_verify.rs截为0B。已从当前已提交HEAD单文件restore，文件226552B、git diff/status clean；拟议Verifier reservation补丁**没有落盘**，不能当已实现。无已提交源码丢失。

两次批量/动态路径清理被自动审核以blocked by policy拒绝（未提供更具体理由）；未执行这些命令。改为先核验静态具名缓存文件及全部父目录无reparse、确认938无活动cargo/rustc/conformance/worker，再仅删除已结束8m GC-omission负控产生的单个3145776B可重建CAS块：`D:/autostack/plan-artifacts/plan-938/tests/source-worker-gc-12032-1793922733582405700/blocks/blobs/ch/chk3bf9273bf02af8b620a0f82898d4dad89538a1e4768424af29fd33efd3f042ee`，删除前SHA256 `2f1f2d975ea24e712c5b22b5cd6c2d69d025778a37746a064cb77b83f13047d6`；日志/原始输入算法/永久反例/计划摘要保留。此静态单文件方式获执行，未借拒绝命令改删未知目录。

核验cargo metadata target确为本计划唯一树 `experimental/ace/target`，target/debug与子目录无reparse、无活动进程后，用原cargo clean显式target-dir及profile dev清理本计划可重建开发缓存，exit0，删除48918文件/16.9GiB，D余量恢复约16.5GB。命令/原日志在938 scope的8o-owned-dev-cache-clean.log。此为在途owner必要缓存收敛，不是merge清理完成，不涉及其他树/仓/owner；统一scope和工作tree保留，旧源码验证记录未改成未测或新测试通过。后续重建准备时间须重新如实记录，继续使用CARGO_INCREMENTAL=0控制可重建缓存体积（未改优化级别/测试规模/运行门槛）；最终冻结配置照实登记。全部功能AC、未完成T-07..11与最终review/merge门继续开放。
2026-11-06 work checkpoint 8p（同goal round8继续，6/13 executing，非handoff）：提交 `485900a4c527b6e12a8e6e073d1ad36a0810328b`，6文件281+/53-，实现树clean。

实际Verifier新增初始化分析预留回调，复用同一完整校验，不复制checker/不制造Checked凭证；在bases第一分配前对原16MiB analysis帽与宿主shared staging一并准入，覆盖3份blocks×words矩阵、3份工作bitset、CFG/前后继Vec/增长并存、DFS/RPO/queue/flags、bases与最大use_slots临时copy。原仅约80B/block的表头估计漏了多个Vec和DFS/queue，改为按这些实际并存对象的256B/block+128B bootstrap保守界；不把整个MIR再乘若干倍。普通函数/直接callee签名改借用，去掉其params Vec副本。legacy owning credential仍只原verify_mir构造；非初始化的ownership-profile分析不冒已共享收费。

native source具体scalar通路 `admit_reserved` 将回调绑定同一Resources staging，原16/24/8与合计48MiB继续AND；真正发射入口和source selected artifact/恢复验证三处均接，Resource保留 `mn.resource.verify-memory`，不归unsupported。恢复MIR decoder仍先占共享剩余16MiB上限，成功后在owner重锚字符串复制**前**按decoded heap+全部新owner标签预留，重锚后缩到实际保守heap，避免把未用16MiB整额长期占用后再申请workspace。R仍无projection/solver/lower/encode/工具。

永久实测控制：1024块/65槽、含合法自环的真实scalar/foreign CFG，同一Verifier实际初始化workspace峰239608B≤事前315580B，宿主回调拒绝时总分配峰仅21B、尚无bases分配。真实源→共享HIR/MIR unit，在共享staging只余64B时native admission与发射入口都在VerifyMemory拒绝且无输出；释放后两入口均通过、COFF输出ticket保留到交回后释放。将真实native emitter有意改回旧不接共享分析入口的源码mutation，永久控制exit100（错误变成较晚COFF managed-memory而非应有verify-memory），finally恢复emit SHA256 `e79c798aa7b955a2d455760947644f3cfb0ac3eb4ac60ee93b631ef654d97d07` 后绿。原COFF输出门由native coff_buffer实对象/精确字节/事前reserve独立用例保留；原source全24MiB占用反控现在合理由更早analysis门拒绝，明确更新其错误owner，未删除该反控；原ABC encode专属压力（允许facts/analysis/COFF、仅encoder超限）保持并通过，未将更早拒绝充作ABC证明。

验证（后续构建均CARGO_INCREMENTAL=0，未改opt/test数量/截止）：cold `cargo check -p ace-abi` exit0/30.72s；av-core完整lib+tests 112/112=59+44+9、0排除，最终运行0.08+0.11+0.68s（8p-av-core-final-2.log）；native lib12/12、0排除、运行0.05s。源消费者6/6，run `5b722660-1dd8-484c-b36d-6a3786f7b277`，编译15.87s/运行18.779s：新analysis前门、既有COFF/ABC handoff、历史cache、真实process restart首编辑、required缺损/错绑定、共享MIR decoder压力→修复；源声明/target身份与P0恢复维持。mutation复原后单项1/1，run `5d71975f-90ab-4001-88af-de0cf37036a1`，编译3.44s/运行0.286s；allocator metrics单项1/1/0.01s。fmt/diff/cached diff/提交钩子通过，原AV3 warning不变。ABI conformance119/source25，lib26；不是全体冻结职责门。

8o恢复补记：随后尝试cargo clean本树root target/debug，cargo因根target缺/坏CACHEDIR.TAG主动exit101，未删除任何文件；未补造marker或绕过工具保护。其日志保留 `8o-owned-root-dev-cache-clean.log`。实际只完成此前ACE dev缓存16.9GiB清理与具名3MiB负控缓存块清理，其他树/主检出未动，现有root target/test-groups原证据保留；正式全输出对账仍属merge。

继续边界：本单元仅初始化analysis及decoded owner重锚，不冒Verifier全峰已关；global type-table/诊断字符串/非scalar ownership分析、真正native lower/符号/opaque backend managed边界仍须按原账本owner核对。cold接口/新binding/query/catalog/HIR/ledger/digest/index、A2X共享读写及pointer encoder预留、请求累计128MiB、并发namespace128MiB（含SQLite/工具目录）与全部tool失败/重启收敛仍开放。完整认证/daily无spawn/readers/SDK/旧桥/export、原样本/P90/Q4/SD/939也未关闭。最终冻结职责并集pending，旧input-changed red保留。
2026-11-06 work checkpoint 8q（同goal round8继续，6/13 executing，非handoff）：提交 `cf63252dd2d11ff30f8b77ca62a82dbc38146d92`，7文件763+/76-，实现树clean。

实际certification prepare/link/run/reference/publish改用原ManagedStore的 `logs/source938-tools/run-*`，空间由原store.space计入；不旁建缓存或删除调用方持久输出。有限owner marker绑定原store稳定store_id与child名，跨alias不靠绝对路径或PID推断存活；原staging_turn+namespace OS锁串行初始化/回收，每child独立跨进程锁保护活跃工作。持有root/logs/namespace/child目录handle拒绝DELETE与reparse替换，owner marker有限512B且拒绝链接/改写。先验证完整有限目录shape（flat+reference flat）再删任何文件，marker最后删；未知目录、reference非目录或junction fail closed，绝不递归穿透。清理用的index与IO在创建前预留并保留到结束，即使共享池已满也不再申请额度。正常/异常均尝试清理，失败留marker与原始文件且返回具名 `store.cleanup-pending`，消息区别publication committed true/false及原失败，不冒失败都没提交。

ManagedStore仅多保留已解码store_id并暴露借用identity/canonical_root，不改wire/schema；20项原lib+leased_publication控制通过（run `a378e1cd-f900-4e6a-a4bf-3e7b64ffba5d`，0.694s，22未选）。实际工具首测暴露MSVC将NT扩展路径误解析为双反斜杠provider.obj（真实LNK1104，非环境归因）；现规范路径用于边界验证，工具参数转同一DOS/UNC路径，拒绝未知device prefix。此前helper junction测试fixture mklink的斜杠错误也如实保留，修fixture后真实junction边界1/1绿。

最终helper 4/4、26未选，run `48f181af-e82c-4074-b983-65c768863707`，编译2.26s/运行0.165s：活动scope保护、池满正常/展开退出清理、真实占用文件导致清理失败→保持owner→释放后新请求收敛、真实子进程exit73不执行Drop→下次创建回收、未知目录与真实junction拒绝且外部sentinel原样保留。junction仅在938统一artifact scope，未放worktree，移除仅链接本身。实际formal worker三项3/3，run `7f01b539-7f0a-4cdf-bcbe-7a5cb257fff4`，编译5.38s/运行41.185s，保留metadata拒绝/重启首错误→修复及provider42→43→42、未改caller P0；新增每次执行后实际输出目录仅owner marker断言。

新增真实reference I/O失败控制1/1，run `04396e79-1821-42a8-83f5-68caa05c5f65`，编译2.46s/运行14.046s，两consumer先发布42，watcher等完整owner marker后放入具名reference占位文件，让实际candidate43之后reference mkdir失败；返回cleanup-pending、committed=false/original=infra，旧current pointer逐字节不变、未知fixture保留；仅移除fixture文件后同请求真实startup回收并得到actual/reference43。反控直接把实际ToolScratch.finish改成不清理、同时禁Drop回收，真实worker返回后永久断言发现run-*泄漏，nextest exit100（run `7d519398-43c6-4716-b0d8-4580a35a3436`）；finally精确恢复helper SHA256 `4630b3d01bb443c63413583812b4548e8058e0190b9d4bfa2b621a36604757a4`，重放1/1绿（run `e73372cf-84d0-4a70-94ae-41c7d72925d1`，编译3.60s/运行22.763s）。fmt/diff/cached diff/提交钩子通过，原AV3 warning不变；现lib30、conformance120/source26，不是全体职责门。

原始日志均在 `D:/autostack/plan-artifacts/plan-938/`：8q-tool-units-final.log SHA256 `1f9890150794ec19415db77a527a0da8f39b69c4c940872b1166f11298f0f7a0`；8q-real-tool-failure-final.log `0faed0eba04108b73d20739e39173ac126544c0a154b81e6b820f0737452caa6`；8q-real-omission-negative.log `a5cf015e78860d3f6a0604dc899b13350116a30783a5fdb43524fe97fa82dd9f`；8q-real-omission-restored.log `ca706cfffccc38a056792d278a9148652bd003eef383287e4aa4f6747ac1543b`；8q-store-identity-final.log `619603eb122aa4aaaafc317af16059f2600b226262a4edcb17b93eaf66b72f510`。

继续边界：该单元只管actual run/reference/publish临时输出；旧provider输入目录仍在store根且不受完整owned lifecycle/space管理，早期输入/声明诊断路径未承诺扫旧tool residue；scratch_peak目前仍0、request128/并发namespace128含SQLite/工具写口未闭。其余native lower/Verifier全峰、cold/interface/query/HIR/ledger/digest/index/A2X/pointer encoder预留及完整T-10/T-11、SDK/readers/daily无spawn、原6×30/P90/Q4/SD/939继续开放。最终冻结职责并集pending，旧input-changed red保留。
2026-11-06 work checkpoint 8r（同goal round8继续，6/13 executing，非handoff）：提交 `b33f44d88acc629a5f1f9c2f1d4306f7ae98720f`，9文件569+/110-，实现树clean。

发布宿主callback不再先返回未收费Vec，改接同一 `PointerEncoder`，借用完整JSON record；原中立bounded codec先测量、共享Resources IO先reserve，再分配精确wire buffer。ticket与bytes一起转交原store Temp，保持到move/remove/失败Drop，不释放再申请，不改变JSON顺序/内容和原8MiB帽。missing/重复/忽略编码失败/serializer变大均拒最终发布，失败ticket释放。实际worker用PublicationRef借用module/Stored，移除编码前克隆；全部12个publish caller（含restore SDK probe/永久fault/cancel/crash控制）已适配；这是本包新接口的同revision实现，不改旧非foreign协议。

实际allocator独立新target source_resources（原ace-abi固定组--tests自动覆盖，source_foreign命名沿原串行资源规则）保留2MiB借用pointer record、原store/真实C provider及真实prepare/publish：修复前额度仅余128KiB时已发生4次≥1MiB分配才拒，永久断言红exit100；改后拒绝路径0次大分配、callback确实到达、旧pointer逐字节/旧session snapshot不变，释放后同record实际发布成功、完整payload读回、IO与staging回基线。首次test编译Stored无Debug的unwrap_err写法已改err.expect，未给产品加Debug或修改门槛。

该观察又发现共享SHA256 helper会data.to_vec+padding扩容；正常2MiB发布在合法输出之外另发生2次大分配，真实反控exit100（3vs1）。改同一SHA256为固定64B块/128B尾块和增量update，所有旧调用仍同名sha256_hex、算法/wire identity不变；pointer identity检查改同一固定8KiB读buffer流式摘要，不先建完整pointer Vec。新增独立CPython hashlib向量11个长度（55/56/63/64/65/119/120/127/128/129/1MiB）、8种分片宽度及空update；原FIPS三向量保留，两lib用例2/2，run `ac58afaa-6fcf-4c54-b866-6a1e8dd67cef`，编译19.47s/运行0.409s。实际发布allocator最终1/1（run `fe8d8701-90ca-4ea3-86a8-93d892b97116`，编译7.31s/运行6.138s）：拒绝0、成功仅1个整记录分配、读取2MiB已发布pointer的identity大分配0；不把全pointer payload读取/typed Deserialize冒已共享预留。

验证：编码改完原发布/工作账/正式provider/实际I/O拒绝控制含unit/allocator共7/7、145未选，run `0fbcd0aa-1957-434a-961c-2e392dd99e71`，编译8.63s/运行93.255s。随后因共享hash/identity实现变化，再重放真正受影响的provider42→43→42、发布fault/cancel/revoke、真实exit73→reader/GC恢复、声明+源码首错→修复→fresh→再恢复首编辑4/4，两consumer均保留，run `4c68e110-c9ba-41d7-a613-4948fdd92e7e`，编译3.20s/运行148.744s，116未选；未放宽子项截止或删原fault。fmt/diff/cached diff/钩子通过；去掉迁移后未用Scratch.resources字段，无新warning，原AV3项不变。现ABI lib31/conformance120/source26、额外source_resources1项；正式P90样本未在本单元运行。

原日志统一938 scope，SHA256：8r-pointer-before-fix-2.log `77b292651e9c20ee1a4ef0b987e4215cb471ff1d1c4fea6a27f57b6b3a854c82`；8r-pointer-hash-before-fix.log `a30fc53130d557cf0bb67b017ebb1b1fd13afd67d3fd9381a74f24c7e5c75a61`；8r-pointer-final-3.log `5250f15b7432996863492ad5e0f669227ed2ffeb20808e26ae0720fa874897dc`；8r-sha256-core-final.log `ccf89a1f5699d325573101ac721014f183d2ff9be738976b988b03b18176456e`；8r-streaming-publication-final.log `c2a6edf4c0463c8be94faea1f3c2bdbe8da864126bf0ae99175f33f9ff41f79c`。独立输入/摘要列表8r-sha256-independent-vectors.log保留在同scope。

开放边界：已关闭pointer encode前门及全输入摘要副本；现read_pointer_bytes返回完整payload与worker直接typed Publication Deserialize还需共享IO/index预留，其他已列metadata/native lower/Verifier/冷接口/HIR/ledger/A2X端口不据此关闭。下一单元把provider source/object放同一owned工具namespace，固定源路径保原C cache的完整header/负搜索域身份，真实复用P0及异常后收敛必须重验；request128/namespace128/实际scratch峰、完整认证/SDK/readers/daily无spawn/旧桥export、正式原6×30/P90/Q4/SD/939与最终冻结职责并集仍开放。旧input-changed red保留。
2026-11-06 work checkpoint 8s（同goal round8继续，6/13 executing，非handoff）：提交 `95ad04821c6f1267bc88b5c1964dc999bda1c4d5`，3文件246+/77-，实现树clean。

provider源码/header/COFF也纳入原store logs的同一owned工具namespace：仅有限固定provider-0/1，源/包含/输出路径每次重建恒定，仍走原C object缓存的源码内容/完整source/include域、实际header closure/negative listing/工具身份；不忽略搜索域、不伪造命中。prepare成功/异常均显式finish，cleanup失败带original与committed=false。request在decode前执行仅sweep，故deck/源码早诊断也会收敛旧quiescent provider/run；OS锁活跃scope不删，同bias并发创建显式busy不覆盖。该preflight释放自己的票据后再进入decoder，不占用正常请求的后续额度。固定provider与随机run共同复用原owner/pin/先全shape校验/非递归删除/失败重试机制；run name严格三段数字，未知子项fail closed。历史本包未交付checkpoint的未标记source938-provider-*目录不盲删，原938 scope内旧输入/存储材料留待merge全路径清理；固定输入路径变化使其旧C cache证据input-changed，不冒无迁移复用。正式当前存储身份以本提交新建/同实现恢复为准，旧非foreign/937协议未改。

验证：首先真实native/C provider42→43→42、caller全部生产0、第三次provider生产0保持，连既有helper4项5/5，run `1dfe1047-1e91-47ff-8aa3-3ae06bb554cb`，编译10.66s/运行24.334s，146未选。最终5 helper+3实际worker共8/8（run `dc3b3bd7-8e7d-4611-87e0-fd1ddeb7a11e`，编译5.63s/运行43.500s，145未选）：新增固定provider路径/活跃排他/实际占用COFF导致cleanup失败→保持owner→解除占用sweep→同路径再建；原真实子进程exit73控制扩为run/provider两种、都在无新child的sweep中回收，外部delivered sentinel不动。新增实际worker early return控制：具名quiescent owner residue fixture同时放provider/run，decode早错及真实源码parse早错各自回收、保持旧pointer逐字节/用户delivery不变，合法修复actual42且compute_calls/source_parse/solver=0。该fixture不冒真实进程退出；真实退出由前述OS子进程控制独立证明。原metadata超额→修复、reference实际I/O失败→修复两端同样通过。

区分反控：在真实prepare_provider中单独省掉finish且forget guard，永久worker断言看到provider-0残留，exit100；单独省掉decode前sweep，早deck诊断后看到provider-1/run-0-1-1残留，exit100。两次finally精确恢复worker SHA256 `f71b0e02a5e0836af0e61ab25107819de72aef597f1e25bcd1483c1c777e11a5`，同一实际worker控制重放1/1（run `604c0497-425a-43f4-adf5-70c3265a2e57`，编译2.87s/运行12.646s，120未选），不是只在单元手工删fixture冒生产cleanup。fmt/diff/cached diff/提交钩子通过，原AV3 warning不变。现lib32/conformance121/source27、额外source_resources1；未跑正式P90。

原日志同938 scope，SHA256：8s-provider-owned-first.log `e8713ebb1abe0f721daba020138659c912af6eb53ff3964fe1164ff33832b70b`；8s-provider-final.log `47472e64901be98b6326d2db6d525789522a305cad85b910b269a39de37b7934`；8s-provider-finish-omission-negative.log `f8fad219cf9811a7468d0592267cd9cf65959fef71898fe1897f731f0f2c1f25`；8s-early-sweep-omission-negative.log `e651b8ecb700182225f58af362d9037eb0564910d9ef2e0fbe3ca7c00e99aafd`；8s-omissions-restored-final.log `85847464f7d457a4841cd7376af801b681bf0542195e8e4f2e10bc6993c3276b`。

开放：当前provider/run/ref的已知成功/失败/崩溃/早诊断生命周期有明确owned收敛；实际scratch_peak报告仍0，工具/metadata内存前门与request128/namespace128全写口、pointer payload read+typed decode、native lower/完整Verifier/冷接口/query/HIR/ledger/A2X等剩余owner未冒完成。完整T-10/T-11、原正式样本/P90/Q4/SD/939及最终冻结职责并集仍pending，旧input-changed red保留。
2026-11-06 work checkpoint 8t（同goal round8继续，6/13 executing，非handoff）：提交 `db0535ba0ec22a22515f573fa9c532a87239c87a`，5文件109+/37-，实现树clean。

恢复实际pointer payload新增 `read_pointer_bytes_reserved`/`PointerRead`，同一Resources按被只读handle钉住的实际长度先预留IO再精确分配/读入，不扩容追逐变化文件；短读/多一字节拒绝，原8MiB/MAX_ROOT与share read+delete/deny write保持。不可变借用bytes及内含ticket使raw payload一直收费到消费结束；旧raw有界read API保留给自行持有输入预算的caller，不把它冒隐式共享收费。worker实际恢复改新入口，同一buffer计算pointer base digest与decode，避免identity/record两次独立读取竞态。

原Resources严格typed JSON decoder新增纯metadata profile，仍先收费parser scratch、再依据原中立预检器计全部model/index、再实际Deserialize，不套source/byte-array特例。worker Publication/Stored.pages模型票据放在与record同一tuple owner，字段顺序保证record先析构再释放预算，持续覆盖后续恢复/发布与所有早返。预算拒绝保foreign.resource、wire/typed decode错误保restore，不改变已有source/request profile的准入或Serde行为。

实际allocator对既有真实发布的2MiB完整pointer：IO只余64B时拒绝、0次大payload分配；释放后恰1次精确读入，bytes逐字节一致，返回对象存活时current.io等于payload长度，drop回0。把实际reader reserve改为0的反控让超额读取成功，永久assert抓到exit100，finally恢复publication SHA256 `0e2f93d02a4cdc90d8ee5a34bf885dcb4c9802946b0d9c2123c401c4fd75790e`。扩展原typed decoder ENTERED控制：metadata超额时自定义Deserializer进入0次，合法小记录进入；单独把metadata实际Deserialize提前至index预留之前，ENTERED=1vs0反控exit100，finally恢复resources SHA256 `778d348f7b5a7e340d9f79d2b907b3d6c3aefadc9e5306c745a39736f10453b8`。未删原source/request控制。

验证：原source-query resources职责7/7（75未选），最终复原run `d12230fb-e12c-41b4-9c41-62784ce6ee41`，编译2.75s/运行0.044s；含并存/总帽/MIR读/alias/foreign metadata/copy票据。实际early诊断/provider42→43→42/publish fault+取消+权限/真实exit73→reader GC及allocator 5/5，run `3fad0806-1cc8-40aa-b236-aac1fd369e21`，编译14.62s/运行88.487s，117未选。随后仅完善record/ticket最后释放次序并做两项反控，复原后实际provider warm/restart与allocator2/2，run `41b6e89a-486b-4469-90f6-a2b9a6b17c94`，编译6.41s/运行24.551s，120未选；其余输入/动作路径未变，复用5项证据，不重复全fault。fmt/diff/cached diff/钩子通过，原AV3 warning不变。

同938 scope日志SHA256：8t-read-restore-final.log `2477814a0dcc2f4cb001fc978630268b882988213190c510d3d1d775c8b7726c`；8t-read-omission-negative.log `35c76eb17d6fce4e2c1425690e038a29afdef5c19acf086482e17abd6886134b`；8t-metadata-omission-negative.log `28b78083e5558332645bcbaae66e8bf17942c36973b173e476d943be7ebc0e06`；8t-resource-restored-final.log `b26668e7ecf8786f87f14d003c7cb852eb25944887e6362215208afc4534ed16`；8t-read-restored-final.log `6ca25921d54ceb8177dcfa85e0e42573b7b0871d0351e9ac4b30c180d569165a`。

继续：pointer encode/read/identity与worker typed pointer model已接共享前门；全source/catalog/HIR/ledger/cold接口/native lower/Verifier/A2X/工具元数据与跨owner实际峰不由该小切片关闭。实际scratch_peak、request128/namespace128、全T-07计量/全T-10认证及daily无spawn/SDK全部reader/旧三腿export、原6×30/P90/Q4/SD/939及最终冻结职责并集仍pending。后续需先核原quota/managed-ledger真实责任，不以外围space估计或现有绿色数量冒完整预算。旧input-changed red保留。
2026-11-06 work checkpoint 8u（同goal round8继续，6/13 executing，非handoff）：提交 `7ccde743dd418ce9763727d26427f086f5b0d8f4`，9文件66+/19-，实现树clean。

清除只读消费者整表临时复制：AdmittedForeign新增借用已缓存不可变声明的slice接缝，旧owned API保持兼容；ABI实际对象符号准入、C header发射、query producer/catalog读取、provider-only frontend比较、core foreign名字/签名读者与decl-env恢复资格比较均改借用。catalog旧foreign名字过滤集只保&str，不复制每个名字；真正保留在Env/ForeignContext中的owned副本未删、不冒其完整预算已关。未改权限/声明顺序/codec/wire，也未让source核心依赖backend。

验证：foreign-contract38+source-query82完整lib 120/120、0未选，run `4fe73df8-dea7-432e-a886-82374cbcbaac`，编译11.88s/运行0.632s；core原decl-env roundtrip/tamper/zero-parse及foreign stale restore 3/3、344未选（run `50a556d9-efaa-428d-a4d4-e67d110bfdbe`，编译8.42s/运行0.037s）。实际ABI双端status/bool/顺序/短路/混合槽原矩阵、声明真实reader批事务、formal42→43→42/第三次P0及全required缺损/错绑定4/4、117未选，run `2cc5529c-f5af-4fbc-9ff7-676632622004`，编译15.97s/运行45.874s，原规模/断言未减。

独立allocator target扩展实际Frontend重绑定端口：中立资格声明（不冒COFF执行）变更当前binding generation，事实页同一Arc、parse/HIR meter与solver不变；按真实SourceDecl整表单项416B分配界观察，whole-copy=0，保留64B binding digest等正常小元数据。把真实frontend比较改回两个owned getter，永久控制观察whole-copy=2、exit100，finally恢复frontend SHA256 `8d25434455edc21c659a12a71e0370ab25a995796c153346fba0d31fa95cbc1f`。包含既有2MiB pointer分配控制的target2/2（run `a3cd2ac5-c912-458a-adec-b5f407610d86`，编译0.96s/运行3.507s）；复原后新反例单项1/1、1未选（run `9bc7192f-5095-460a-9390-93692f5812a2`，编译2.07s/运行0.015s）。最终仅统一decl_env.rs原CRLF到仓LF以消除Git行尾警告，正常源码内容不变；fmt/diff/cached diff/钩子通过，原AV3 compiler warning不变。

原日志同938 scope SHA256：8u-foreign-query-readonly-final.log `6cf7418b1cf3218bbb4b16a1984a8cbb45faea7531d0f312915c50d8ac9802be`；8u-core-readonly-final.log `b1ad021ece5563f4e1ae1dadc682b08ae3acf3c542d8a834cbab5cf5f3760b41`；8u-actual-readonly-final.log `bcaab786d64a410f84ca9937f3f1a533bcd059ad81d80ee01bac008eab4828f6`；8u-declaration-copy-negative.log `3aacc0327cd4a7543890973bb4802c4adf6f04ce0c1ef1d5b3044fba8ef6d143`；8u-declaration-copy-restored-final.log `445b34b56aa1a15c5fe00960cd6e69fc9edc3b244704a2c79a1c594cc904548d`。

继续边界：本单元消除多处无必要整声明复制，ABI符号Set/Map索引及真正owning catalog/Env/ForeignContext/HIR/ledger/new binding/native lower/完整Verifier/A2X等预算仍须原owner逐口核；实际scratch峰/请求与store quota、完整计量/认证/daily无spawn/全部SDK reader/原桥export、正式样本/P90/Q4/SD/939与最终冻结职责并集不据此关闭，旧input-changed red保留。
2026-11-06 work checkpoint 8v（同goal round8继续，6/13 executing，非handoff）：提交 `791213cc10ff43cec35f33933f2cfc91744cfa72`，4文件124+/4-，实现树clean。

关闭此前“compiler current与parent oracle认证边界尚待实际反控”的oracle部分：原TestCase runner已有PLAN-773 F-03语义，执行/产物可用性与独立oracle verdict分开；S938 pointer是编译器当前source+foreign+target，不是被缓存的父侧PASS凭证。父侧预期写错不能把语法/类型/当前binding都合法的编译器root冒为失败候选并要求回滚；source/权限/目标/取消/过期/发布故障的拒安装义务仍由原唯一publication gate及既有fault矩阵负责，未放宽。外层late/非法transport仍不得授予认证，原EOF/重复JSON/hang门继续有效；本项不冒late错误一定从未提交compiler root。

新增真实父adapter→bounded Job worker→实际native/C callee+独立reference→父typed judgement→raw落盘控制：每consumer使用同一个artifact/store、相同普通source/deck，独立oracle按43→42→43三次解析/resolve（oracle哈希首末相同、中间不同），得到FAIL→PASS→FAIL。六次均实际query，actual/reference均42；每轮compiler pointer逐字节一致，后两轮compute_calls=0，两个consumer各自JSONL行ok=[false,true,false]，没有把先前PASS缓存或把错误oracle送进worker。源report实际值/独立reference mismatch存在性一起核验，非只看最终status。只省掉真实父侧reference_exit比较（actual侧仍照常失败）的源码反控会遗漏reference mismatch，永久断言exit100；finally恢复parent SHA256 `0c8e2c252623d01f99a51b979beff914e5b35dd1b48d9df9657ec1b38633eb1d` 后2/2绿。

同模块原迟到重复stdout/响应后hang控制保留、未改300ms测试故障截止。增加测试内mutex与RAII故障复位，避免裸libtest并发时两控制互染；三份nextest配置将该父模块并入既有source_foreign整池保留规则，保留所有选择/原阈值，仅串行真实nested Job控制。新真正Windows x64 provider测试有Windows条件，不改原protocol control的平台条件；配置统一仓LF，消除行尾warning。

验证：首次真实parent1/1（run `a709d9cd-608b-4db6-9a6e-8fa13ad8d70e`，编译51.67s/运行22.613s，1647未选）；反控exit100，复原2/2运行28.660s。仅再加bare-libtest隔离/复位后最终父模块2/2（run `3562136f-5c05-4440-b73b-bd31dee06b44`，编译29.39s/运行23.383s，1646未选），源callee+reference六query不减。父测试全过程固定显式ABI_CASE_WORKER为本树debug/abi-worker.exe，SHA256 `84392a6b176f1ba33ecea09fd5cfaf4dc7df1ddb7648f8ce04d083694a41be20`，验证前后相同，无竞争ACE构建；TEMP/TMP仍938 scope，CARGO_INCREMENTAL=0。root auto-lang-core lib-test配置显示145个既有其他文件warning，本次编辑的abi.rs无warning；此前131来自不同root target配置，不混称同一次数。fmt/diff/cached diff/钩子通过。

原日志同938 scope SHA256：8v-parent-oracle-first.log `30e470510fa35b5ad01a10c90423b492c115df2a4eb61843787ca8cef5639237`；8v-reference-oracle-omission-negative.log `034e0ef4cdbfc12bc2370c930bbd4c3087dc3065cb5ff330bb7b3ce85bf51a22`；8v-parent-isolation-final.log `e640302a838954061f83908d409f2c1e1be00f884dd0d9026805453945d9633f`。

继续：该控制关闭oracle/编译器发布角色混淆，不提升T-08或完整certificate为全部通过。完整计量/资源owner/实际scratch峰/128MiB配额、审定表SHA256误标与实际BLAKE3 pins一起修正、完整多fixture/全部reader/SDK/daily无spawn/旧桥export、原样本/P90/Q4/SD/939及最终冻结职责并集仍开放。旧input-changed red保留。
2026-11-06 work checkpoint 8w（同goal round8继续，6/13 executing，非handoff）：提交 `2a8545771eb73b1b90db71bed72fa898a8149b47`（验证器诊断home，2文件5+/2-）及 `0a434c7e7f58003718b52e298e50b035afca351f`（review说明/全部pin图，7文件53+/7-），实现树clean。

修此前明确登记的SHA256误标：独立source_foreign938.contract的review_rule改为实际使用的BLAKE3（header/review-table/actual object），不换算法、不伪新ABI。表BLAKE3现 `0ea8be97d1026bd57dcaff7b35673e45a6a9f17e2f5c991bdd1b1fb04a94daa3`，对应手工add契约pin现 `cef4c3885f1686a00c0bb8c44ca6b95559a634fff297f8193becca2d1d35d71c`；同一补丁更新host const与五个成功/拒绝输入夹具。HEADER未改（working/index均LF），普通source_sha256及S938 root SHA256仍各自原算法，不能混写。新增pin闭包控制逐个核add/bias/denied/unreviewed/wrong-type的header/table身份与实际资格域，确保拒绝fixture仍因其原拒绝原因进入，不因旧pin漂移假红。改表后、尚未更新pin时该控制真实exit100，打印新表/契约值；更新整图后绿。

验证：registry闭包、SDK runtime pin、实际native/C provider42→43→42与真实restart首错→修复4/4（run `f995740e-65d5-476d-a88b-67accf55395b`，编译5.65s/运行36.009s，150未选）。随后正式 `cargo t case-ace-ca-d-source-938 --out D:/autostack/plan-artifacts/plan-938/8w-source-case-gate`：总115.592s、实际case执行24.874s，11/11执行，正常/预期拒绝共10 PASS，独立wrong-oracle实际FAIL且actual/reference42，按既有case-control-oracles.json登记整体gate_passed=true；真实auto-case child exit1保留，不以包装exit0抹掉反控。add两端重复query仍cache hit/compute0、body bias实际43、权限/未审定/类型错保持规定原stage/code。不是6×30正式P90，也不是完整T-10多fixture。该次review表字节变化使旧输入绑定证据过期；保留历史记录，新正式首例已绑定新pin，最终冻结职责并集仍由后续owner运行。

执行正式gate前核出fixed-tests.py将准入diagnostics硬写仓内target/test-groups，虽显式--out仍漏出，故作必要配套修复：diagnostics改可信调用方TEMP/TMP的独立临时目录，仍在最终output获得Case准入之前创建，绝不移到尚未准入的用户output内。扩展原“owner拒绝先于任何最终输出写”控制，同时断言诊断home；mock mkdtemp保证反控不在仓内实际写目录。原fixed groups治理37/37（5.793s）；把真实入口改回仓内dir则该控制exit1，finally恢复script SHA256 `518448502925e69b0de6d96c2bba90f8523164445761e7f4965345a19457f654`后单项绿。正式gate回读确认diagnostics=`D:/autostack/plan-artifacts/plan-938/test-temp/fixed-admission-42_9pl7s`，运行/复制证据均在统一scope；normal build仍只本专属worktree target。该配套不改选择、oracle、Case安全准入、原限制或工具等待。fmt/diff/cached diff/钩子通过，原AV3 warning不变。

同938 scope日志SHA256：8w-review-pin-before-update.log `92ff0b281a347ce4d0302a80da7b84af66e7733308310913c908bcb1860efc6d`；8w-review-provenance-final.log `1ab8109c51da7c0c512cb2e05c28367d38eb89d542742d65f40c029ff54e08e5`；8w-diagnostics-home-final.log `24c0e7f2f5b83069ad1412a6515261122b45a49c575bde9cdd5e406627b0ebdf`；8w-diagnostics-home-negative.log `a4129cb248624246ea021eda1d53a1750d6f105e112b2c158111f0b113b9971b`；8w-source-case-gate/results.json `2453a2708a2e34e15fb2565502564442c1ed21e45f299a2540af99ebc63dc849`。

继续：审定表算法说明/全部add pin闭包已关；并非新增全部7函数认证policy。原完整T-07计量/managed预算owner与实际scratch峰/request128/namespace128、完整T-10 policy/元数据/readers/SDK/daily无spawn/旧桥export，原正式样本/P90/Q4/SD/939、最终冻结职责并集仍开放。现lib33/conformance121/source27、source_resources2；旧input-changed大gate red仍保留，不用本首例gate覆盖。
2026-11-06 work checkpoint 8x（同goal round8继续，6/13 executing，非handoff）：提交 `bf674a5fc45b020d718a51986bf8f68dab7dfae7`，3文件322+/35-，实现树clean。

实际provider对象资格检查的两棵符号索引接共享Resources：先按声明/ctx名字数量与当前libstd B=6节点字段（11 key/value、12 edge、64B父/索引/长度/对齐余量、每插入key至多一节点、两索引）核checked算术并reserve index，才parse对象和建索引；改显式insert避免FromIterator的额外临时sort Vec，保持原名字/定义过滤与重复键最后值语义。六个调用口（直接cold/cold_stored/from_frontend、provider重绑、声明事务、R恢复）均传同一源会话账户；两个cold入口将同一新Resources交原cold_observed_with_resources，未旁建对象验证专用池。布局依据读取本机rustc 1.98.0（88d9e12ae 2026-08-18）libstd源码node.rs的实际B=6与字段，不猜一个统一MB余量。

同时在资格检查入口adopt实际Arc<Vec<u8>> provider输入，按capacity+40B Vec/Arc所有权与共享resident AND帽计费，发生在符号解析及所有source/target生产之前。R loaded page/调用方alias不重复计费；被拒候选若仍由调用方pin持有，其真实并存字节仍收费直到最后owner释放。此为composition消费已给定输入的前门，不冒worker C-cache/工具产物最初分配或Arc构造/clone已经全部事前收费；这些生产口继续开放。

永久实际控制：两consumer先接受旧provider；共享index只余64B时新provider重绑在provider-symbol-index拒绝，旧provider/源meter/同一caller Artifact Arc不变；调用方仍pin的新provider精确增加capacity+40 resident，释放压力后同请求成功且实际调用43、caller不重新生产。新snapshot验证在释放测试压力之后做，不要求其他metadata消费者在64B余量仍能分配。省掉真实索引reserve时请求错误地成功，永久控制exit100（8x-provider-index-omission-negative）；只对初始代际adopt、漏新binding输入时原index拒绝仍成立，但resident观测11790vs应13983抓到缺费，exit100（8x-provider-owner-omission-negative）。两次finally分别恢复当时完整源码hash，后者 `cebfb3a68e4c8e4ad2003e0e19fe1d0761645aa1a91869f0dbbac71d1624b391`；之后仅修“当前支持libstd”注释，不改产品逻辑。

另用独立global allocator观测真实Session.rebind_provider窗口，在强制index高水位后按实际账本peak差取得事前费694B，实际新live heap峰392B≤694B，同一caller Arc保留。观察器仅跟踪窗口中新分配的地址，旧对象释放不能抵扣；真实realloc按旧+新并存保守计费，固定128条记录满即失败不归零；增加旧块释放+两个新live块及真实Vec扩容校准。此为该微观端口分配证明，不冒整个进程/Job峰。

验证：实际wrong object先于source生产、ctx必须Data且非零合法、新quota拒绝→修复3/3（run `77a4a5e5-01fe-4560-8ff0-076f4aa16ee3`，编译5.55s/运行4.271s）；接输入owner后新控/R共享MIR压力/两端立即编辑及首错→修复3/3（run `506c3890-b604-4c95-b62c-30a39e551a61`，编译5.11s/运行17.036s）。owner反控复原后实际formal42→43→42/全部required页/新SDK quota与allocator等6/6（run `aba6859e-0a5d-40ec-a71a-66477f8696d9`，编译3.44s/运行22.568s，119未选）。随后仅增强allocator校准与注释，最终source_resources 4/4（run `12b6d10a-fc88-4839-ae1b-af6c3eb30f8a`，编译2.10s/运行5.329s，0未选），保原2MiB encode/read/identity与无声明副本控制，仍观测694/392。fmt/diff/cached diff/钩子通过，原AV3 warning不变。现conformance122/source28、lib33、source_resources4。

同938 scope日志SHA256：8x-provider-index-omission-negative.log `7ca9bc29a8567e4e7ec4c0c895ab15f38ed889ad4952c146a27ddf6d122da518`；8x-provider-owner-omission-negative.log `7964864d8f073216ff9b62c4f080dd61b866cbc440d15a063cbb371e2db591f0`；8x-provider-owned-restored-final.log `45cf27fb94ed353be1e3a34c1239eced78bee07313a79e7cede0e08c47a88ea0`；8x-provider-index-allocation-final-4.log `511789c0dbb0a88249f031f9282ca5a4e0135bc6b4a47d6f16eeada5f6c0f044`。

继续：provider资格输入/符号索引已接共享前门；真正worker provider-cache/tool生成与clone、cold/new-binding接口、owning catalog/Env/HIR/ledger/native lower/完整Verifier/A2X等其余owner、实际scratch峰/request128/namespace128、完整计量/certificate/daily无spawn/所有SDK reader/原桥export、正式6×30/P90/Q4/SD/939和最终冻结职责并集仍pending。8w首例新pin gate保留，旧大input-changed red不改成绿。

2026-11-06 work checkpoint 8y（T-09 provider-only绑定生产端口；仍executing 6/13）：提交 `fa3f180585ad03273e20cdd7e87206327c78cbab`，4文件179+/25-，唯一工作树clean。复核main pin `bdd73416fbd8556a09abb829e2ee2c8bf8531c91`相对8l输入新增的source/core与query实例/CT改动；此端口未取这些未合输入，最终漂移核验/合入仍pending，不能把本树证据称最新master证据。

原ForeignPages绑定编码换借用DTO与栈上BLAKE3 hex，原schema/字段顺序/内容保持。AdmittedForeign两个摘要改共享Arc<String>，clone不再复制每个摘要；provider rebind仅新建自身digest。新增中立rebind_reserved回调先按精确bounded JSON大小+Vec/Arc40B与新digest String/Arc40B预留，才生成新页和digest。source-query Resources.rebind_foreign接同一账户，页按resident、独立digest按index收费，分别retain到各自Arc；页别名与环境别名有独立正确生命周期，snapshot adopt不会重收。真实worker body-provider事务已改用这个端口，未新增池、未更改权限验证或required页格式；旧rebind保调用方管理预算接口。冷admit、工具/cache输入生产、其他拥有型metadata等仍未核销。

永久分配控制：仅余64B index时新绑定在foreign.resource.binding拒绝，旧generation及resident不变；释放压力后同一请求成功。真实生产+clone窗口新live heap峰509B等于事前resident405B+metadata104B；共享logical页及interface allocation identity不变，新的provider digest identity不同。环境最后owner释放时仅metadata退费，外部required页alias仍收费，最后页owner释放后退费；adopt同页无重复费。将真实metadata reserve改0使拒绝断言失败，nextest exit100；finally恢复resources SHA256 `257f718a8644caac71f20cc29109e99cce3712a709731bb882403ded9dd3dc35`。随后仅将unwrap_err改为err/expect以避免反控打印整个对象，产品逻辑未变。

验证：source_resources全5/5（run e354cc0e-36c0-4c5f-944d-5e560c77ed21，编译15.65s、运行5.652s，0未选；原symbol-index 694/392、2MiB pointer和readonly声明控制全保）；foreign-contract/source-query完整lib120/120（run04c0bd51-c759-4680-be9e-cff6012cf8a5，编译4.66s、运行2.764s，0未选）；真实ABI声明reader事务/正式42→43→42且caller P0/首错→修复/每个required页坏失配4/4（runbb7f59d5-4fec-47fc-bf9d-1cb091af252f，编译3.08s、运行52.078s，118未选）。fmt逐文件/diff/hook通过，原AV3 warning保持。初次误用cargo fmt --all触及无关格式并遇原mem_profile缺失；已依据开始时唯一树clean事实将所有非本次4文件的纯格式改动恢复HEAD，最后diff仅以上4文件，未包含其他owner改动；改逐文件rustfmt，未扩大最终写面。

同scope日志SHA256：8y-binding-first.log `b48e142d2611353bac0cdb435c73545b3459322693ffedca30695db241a260fb`；8y-binding-reservation-omission-negative.log `7f24c47110b26e6266af59ec7f189fd0ae5d6ae860d5076a3149214bbee3a36a`；8y-binding-allocation-final.log `beb5b2c7a35756d9d3aef9f1279ed4626e90eecc6fbf7d547f2e54c39947c96e`；8y-binding-neutral-final.log `c7906fe935de29fc72df6e13dd5fb836053e16cca4d00b1d21b689a2d97c7026`；8y-binding-worker-final.log `f0215c7fe423f84c81827709b34bdc860215e71306369f5ad58c585367c0f910`。

继续原T-07..11：MIR共享lower/完整Verifier诊断、cold/new-interface/owning Env/HIR/catalog/ledger/A2X、真实scratch峰/request/store quota、完整certificate/daily零spawn/SDK镜像/原桥、正式6×30/P90/Q4/SD/939及最终冻结并集均pending。此为一个端口通过，不是work交接或final review。


2026-11-06 work checkpoint 8z（T-09 scalar MIR验证器全局类型辅助分配；仍executing 6/13）：提交 `45ad22dc41985a3b3441bffa040ba40148203370`，2文件107+/0-，唯一树clean。原8p已在初始化bases/CFG/bitset前预留，但check_record_acyclic在其之前对合法scalar/Fn表仍先分配colors与逐root栈。永久4096额外合法unused I32类型反例实测未修复时先分配4114B，nextest exit100；不能用减少类型表支持面或提前拒合法输入规避。

原唯一Verifier的无Record/Variant图路径不再建无用DFS空间；仍逐root tick(1)，完整保原工作步数及每1024步deadline检查，非批量跳过时钟。含任何Record/Variant仍走原图检验和layout，不复制Verifier或放行环。相同4098项类型表在共享账户拒绝时分配峰<256B（仅错误对象）；账户接受后全验证实际峰202B≤真实CFG/dataflow预留448B。补合法nested+disconnected Record、直接/间接环均mir.type-cycle、修复回合法永久控制。

最终验证：av-core lib59+foreign_resources11=70/70（run63e664e3-b9ce-4f10-8ff3-aa75a874be56，编译3.95s/运行1.072s，0未选；原解码/编码/剩余额度/1024块初始化分配观察均保）；native lib12、原AV data/variant直接/间接环与合法nested clone4、实际938共享MIR恢复额度释放重试1共17/17（runfb74316f-9aba-4d10-8b47-d4fbd58667d9，运行1.748s，1254未选）。第一选择式误写的actual matrix子项未选择，未作通过；随后按实际注册名单独运行两端status/bool/order/short-circuit/mixed/named参数矩阵（20次真实独立C调用）与native共享staging拒绝时无codegen/释放修复，共2/2（run4bda5fe7-65c1-431f-831b-b187fb6e0f56，编译0.25s/运行37.174s，120未选）。fmt/diff/hook通过，原AV库3warning及AV conformance旧unused fixture warning不在本diff，未抑制。

同scope日志SHA256：8z-scalar-type-workspace-before.log `ebb9139864ae28fe017dc5a507562c1efa6a2db79a51e53e6d28998c97c35879`；8z-scalar-verifier-controls-final.log `0cfc5d7762dc7e6a9ab8082179274ab3a8dd9aba321f284d166d4a96b2bd5b03`；8z-scalar-consumers-final.log `37365a69b9aed3166ee8b22b40e83f8f7f817834d2f8b7587aeb890d7fff462e`；8z-scalar-native-real-final.log `524040abdee0b056d8fab2aa13373a2ee2a8f8e1ac50f7054bba93f680befb86`。

继续：此关闭scalar无边类型图额外分配；未称完整Verifier所有profile/所有错误诊断、shared native lower/码生成、cold接口与Env/HIR/catalog/ledger/A2X等全端口已闭。实际scratch峰/原request128与namespace128、完整计量与证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4、SD/939/最终冻结并集仍pending。未到work pass或独立final review。


2026-11-06 work checkpoint 8aa（主线CT接缝同步与真实共享前门回归修复；仍executing 6/13）：唯一树以merge `6fa8332b599f0b2ea72b10ec7ca6f01d8f363afb`纳入已落master pin `bdd73416fbd8556a09abb829e2ee2c8bf8531c91`，119文件9954+/963-（主线已有936 CT/958 A2X前门/stdlib/文档等真实依赖，未把它们计为938新增功能）；无文本冲突，主检出仅其他owner文档WIP且未暂存/未改动。随后本计划修复提交 `9fee722ca82c86efee331679fcd188e3fd2f9f99`，3文件49+/14-。auto-down pin仍7c4d5cb。主线pin之后新增提交未包含，最终merge漂移核验仍pending。

首次core冻结回放不能称绿：380实际执行中379PASS，原deep_unclosed_structures_are_controlled_rejections持续回溯；比对旧8l同责任conformance347测试2.55s、读取并确认本次唯一活跃PID5716精确命令后，定位grammar中936 CT与950 modules两个profile列表分别引入同体递归unary alt，重叠profile有两条minus与bang路，深层失败指数回退。用git show核实该重复已存在于所消费master pin（不是归因环境）。在201.059s明确人工停止这一个已诊断用例，原nextest终态exit100/ABORT(0xffffffff)，全命令202.213s；不是自然超时、不是PASS、没有把样本/规模/原deadline改小。

实际修复把两递归分支profile并成包含scalar-ct和scalar-modules的唯一minus/唯一bang路径，保两面、普通profile、原源序/投影/拒绝语义；没有另一parser/solver或小输入替代。原完整core再跑380/380 PASS+原ignore1（rund7038bf1-9df2-4b3a-a395-4d250cc6b68d，编译10.74s/运行2.678s），原4096/8192等深层未闭合case0.267s；保core/profile差分及CT新门。补普通foreign Frontend权限边界控制：无/有CT capability时此有限入口均以Text/source.syntax原# span23..24拒，solver0/无hir-document产物，不能因此给该入口增CT服务或FFI执行；相同权限普通foreign_add仍成功。真实测试先观测后将分类/span约束写成精确断言。去掉936迁移后已全仓无调用的pub(crate) first_lex_fact shim，消除纳入的core dead-code warning；不抑制warning。

其他验证：source-query lib及3个实际变更integration targets choose_chain_935/instance_host_935/instances_restore_controls_935共110/110（rund951289e-876f-43b0-a815-233abba519af，编译11.32s/运行0.495s，0未选）；真实ABI actual符号先拒/声明reader事务/正式provider换版caller0/两端恢复立即编辑与首错修复4/4（runf4f3c82d-e72a-4a91-b56d-99f3b4f1bdf8，编译18.12s/运行35.922s，118未选）；新增CT/foreign控初版1/1（runecd3b277-9dae-4852-a08b-c96af59e0864，0.046s，82未选）。删死shim并加强断言后core/source-query完整lib106/106+原ignore1（运行1.224s，0条件偷改），输出无core新增warning。fmt/diff/hook通过。仅后续8ab自有source_resources中的未运行C hash分配反例仍WIP，未混入本次3文件提交；不是整体work交接。

同scope日志SHA256：8aa-core-integration.log `7b66079898e1ea5c32ee6198a99b1639a6e02026c2ca4d21021dc4720b9966d9`；8aa-core-integration-repaired.log `d7e0306cc4d064f73e28693aa0a5360dcc6954e89505ce33e05921d9ce5df7cc`；8aa-query-integration.log `ad30a7a81cfb9d56b5b4ad565627eadd65c15d8b90d20e809cf68d90c43e6fd0`；8aa-source-abi-integration.log `6918e1107b8757e2b36ead1766e8d1f6caff28be56122f2ff7d87ab8ef6702a1`；8aa-ct-foreign-boundary.log `f535b353ed8cec3c36fd041f47f328fabe8dcc3dee1f67d8783eaecf163d6ee1`；8aa-query-boundary-final.log `ed76c7c70ae441cdb7fff3dc6286815b0a0afeda89db9523a5d5c6f0d2708723`。

继续原T-07..11开放项；接下来C cache内容hash全文件读缓冲的真实分配反例/原有cache依赖控制。没有进入final review、没有消费修复周期，原6×30/P90/Q4/全证书/SDK/每日无spawn/SD/939/冻结并集与其他预算owner仍pending。


2026-11-06 work checkpoint 8ab（T-09 原C cache源/依赖内容哈希读缓冲；仍executing 6/13）：提交 `9bc198678e4459c1b98f658d43a67fabd30fdd3e`，2文件57+/4-，唯一树clean，无活动测试。沿原cache::c_source_identity及hash_dep，用同一8KiB栈缓冲的BLAKE3流式hash_file代替完整fs::read Vec；保原source文件名/内容摘要、头/系统头/传递依赖摘要、错误分类/消息前缀，Interrupted重试，不改缓存schema/查询key/真实依赖与负依赖验证/producer计数。没有新C cache或另一个发现流程。

真实分配永久反例在改前对2MiB源观测2097323B峰并exit100，改后同一入口峰194B。源名与完整摘要相同，空输入与8191/8192/8193等最后块边界、文件不存在控制通过。source_resources现在6/6（runc3c8c966-0bef-41c1-becf-e37c7306dab9，编译3.67s/运行3.948s，0未选），原2MiB pointer/声明无副本/provider index694/392/绑定509/509等控制均保。原C cache真实include多种预处理形式变化、系统头闭包、外工具代际拒、传递header目录shadow、嵌套artifact cleanup复用及938正式provider42→43→42且caller P0共6/6（rune14655db-c3e9-45c0-86f5-1f661b1d9a58，编译3.07s/运行25.419s，116未选，-j1单实例工具）；原5M/64M预算和原样本未减少。fmt/diff/hook通过，原AV3warning保持。

同scope日志SHA256：8ab-c-source-hash-before.log `dd04d300a5916c3f6f561fd74e3f668dedb8ee15bf920959e3ee87337b40af7f`；8ab-c-hash-allocation-final.log `f5d7d06fc695ce1017e80e950fa25e7402a75c7938a75fa279c6bd6d2edb01c9`；8ab-c-hash-cache-consumers-final.log `d953e2ced7013fc16d1cab1f167463f48bf47f2fe51a86bcbfd467df57afc9b3`。

继续：此仅消除原C cache源码/头内容hash全量读副本，非provider cache entry/serde/base64/object生成/clone全部预留；这些与cold接口/owning Env/HIR/catalog/ledger/native lower/完整Verifier错误诊断/A2X、真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/最终冻结并集仍pending。


2026-11-06 work checkpoint 8ac（T-09 worker provider共享输入所有权/两处真实clone；仍executing 6/13）：提交 `9b93ce77e8bd03ecc95da8b520692c8279886c58`，3文件77+/7-，唯一树clean，无活动测试。新增Resources.own_payload消耗调用方已给定Vec：按真实capacity+Vec/Arc40B在Arc构造前预留resident并保留到该Arc，调用方Vec最初生产不冒已预留。实际prepare_provider把原C cache返回的Vec直接交此所有权口，在原ToolScratch操作闭包内完成（失败仍走finish）；返回Arc。真实cold与foreign/source事务两个Arc::new(bytes.clone())改为同一Arc别名，reference写文件借用slice；旧binding R资格重新编译/查cache路径保留，provider生产/命中计数不变。

永久实际分配/所有权控：2MiB spare-capacity输入地址与capacity原样移交，resident精确capacity+40；alias/adopt不重复收费，最后owner退resident，账号自己的所有权索引capacity保留并仍计index。小total账户拒绝候选且不漏live fee，释放压力后同尺寸成功。冷账户首版测试错误地把整个峰都限制40B，实际3624B含已有Resources保留索引首次扩容3584B（源码在try_reserve_exact前独立index收费）；据此把oracle改为40B+实际index费用增量，并检验索引保留/对象0，未放宽payload地址/容量/费用/拒绝修复断言。校准后1/1；人为将真实移交改bytes.clone()时地址断言失败exit100，finally恢复resources SHA256 `8ce3c1357df53e037edffd45c8e9d8b0db8c1e300591981404c416258810161e`。

最终source_resources7+真实正式worker provider42→43→42且caller0、首错→修复、早期诊断清旧provider残留且不发布、required页逐项坏失配4，共11/11（run9ea2e153-82f2-432a-912c-6a96e44b7b0c，编译4.04s/运行46.317s，118未选）。fmt/diff/hook通过，原AV3 warning保持。以上是消费已返回Vec的所有权/复制口，不冒C cache JSON/manifest发现/SQL ID列表/base64解码/原producer fs::read等更早端口已全部收费。

同scope日志SHA256：8ac-owned-provider-first.log `cb17049876da547525204450c72c48945b898db0c414fc5607b4bfcd96fb8c93`；8ac-owned-provider-calibrated.log `ec709eb184f1b2f19c1a5dd2a4fc34b2e2ad7acfd88d9f5e19b87562875871ac`；8ac-owned-provider-copy-negative.log `5c0fc7a1a1be2b1955f0183b38a0a533e330711e606c97709a01b03a9473f518`；8ac-owned-provider-final.log `216215264435d863207f4396e424291237df6dfd8683a2b104c417d48d685d61`。

继续原T-07..11开放项：provider cache更早生产/解码/索引、cold接口/owning Env/HIR/catalog/ledger/native lower/完整Verifier错误诊断/A2X、真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/最终冻结并集。没有work完成交接或final review。


2026-11-06 work checkpoint 8ad（T-09 原C cache discovery SQL→ID拥有型索引；仍executing 6/13）：提交 `6c765b2088320bfe9e03648d46fc3633e988f199`，4文件316+/12-，唯一树clean，无活动测试。原produce_c_object保持同一implementation/SQL适用性/发现与v4验证算法；新with_resources入口供938 worker传同一共享账户，legacy入口仍调用同实现且保原调用方预算。SQL row改借用ValueRef str，先checked计算Vec<String>容量+所有ID字节、扩容时旧/新Vec并存峰并resize index，才try_reserve_exact与to_owned；完成扩容后退到steady，整个发现期间持有。IDs/费用置同tuple，按字段序先销毁IDs再退费用；不在DB锁内加载payload，原锁顺序保留。未冒entry JSON/base64/搜索listing/原工具输出等所有端口已闭。

永久实际cache控用独立编译COFF和原store预建真实v4命中：只余64B index时在C cache discovery index拒绝，producer不能调用；另将该SQL行ID故意改2MiB，真实Rust allocator窗口必须<64KiB，修复该行/释放压力后仍命中原字节且无producer，index费用全退。将borrow后、reserve前人为加id.to_owned时即使最后仍ResourceExhausted，测得2099207B、exit100，证明能抓“先复制后拒”的错实现；finally恢复cache SHA256 `073497f0d3a06579f34d356442f011f55a9e4b3a789f44d9a0c63c2627b3724f`。初版1/1及扩坏row1/1通过；该峰为Rust新拥有型分配，未称SQLite C allocator/全进程峰已全计。

恢复后source_resources8+原5项C cache真实依赖/系统头/外工具代际/传递shadow/嵌套artifact复用+938正式provider换版1，共14/14（runa5a8637f-5d1b-47d6-894c-0773f1b1b7f7，编译3.01s/运行25.453s，116未选，-j1）。随后新增真实worker原预算反例：两端先正式42，再故意将ABI-O1那一行ID改9MiB；相同deck请求在默认8MiB index门拒且不得变pointer或留tool scratch。此控先抓到真实Worker将StoreError::ResourceExhausted误映infra（exit100，4.286s），产品修正该枚举分支为foreign.resource，其余infra保原。测试修复唯一坏行并VACUUM该9MiB删除cell，以实际释放故意制造的磁盘压力，再按原空间帽重放；不是抬帽或产品路径加VACUUM。两端repair实际/reference42、compute0、parse/solver/HIR/MIR/native/C六项0，旧pointer在拒绝时字节不变、临时目录清洁。新控+原request metadata压力恢复控最终2/2（runf008cdd2-a621-4281-8034-7308c214b6a2，编译6.53s/运行31.517s，121未选）。14项成功路径代码未变及direct cache错误枚举未变的证据复用，未因分类小修再重复其全组。fmt/diff/hook通过，原AV3warning保持。现conformance123/source29、source_resources8。

同scope日志SHA256：8ad-c-discovery-first.log `b83fe4c9cf97c2ebd353ffcabad77fe26f9b03aa70fd8b546b35baee8b61e38d`；8ad-c-discovery-corrupt-control.log `7bc1aac478d2ad3b46f0f3a39f1cff8ad4d0320a60f0a8c88f33d0f3c6bd119a`；8ad-c-discovery-premature-copy-negative.log `4b9f09163cf794992ffd0c8e3050336c37f8024d572b6acca9be3f14e594a13d`；8ad-c-discovery-final.log `8ad211f96dd448d5f72c027a48c8e2f200b1a7b2c069d57010b0f5f1cb24089e`；8ad-c-discovery-worker-taxonomy-before.log `20d431768457f2d898245dcd25eb86a887c3a292b8ee27d76e3f0c387a612e73`；8ad-c-discovery-worker-final.log `0920c0be3335ceedc9433e72f865abecec34f16a9dd0e0d00b7ffff24e37bb19`。

继续：原C cache entry读取/serde/base64/producer fs::read/encoded写入/搜索listing等metadata、shared native lower/完整Verifier错误诊断/cold与owning Env/HIR/catalog/ledger/A2X、实际scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/最终冻结职责并集仍pending；未到work pass或final review。


2026-11-06 work checkpoint 8ae（T-09 原C cache entry物理页读缓冲；仍executing 6/13）：提交 `1145e0cefb04f6c6e24c05726964a942756b0da1`，3文件117+/3-，唯一树clean，无活动测试。资源版C cache沿原ManagedStore get_artifact_by_id_bounded读Object载体，复用Resources.load_payload在真正文件读取前按剩余8MiB I/O/48MiB总量预留framed/returned页并传原读取上限，返回Arc保原capacity+40 resident直到最后raw页owner释放；legacy调用方仍旧Vec路径，不加旁存储或换key。内联enum只区分两种既有所有权，不复制payload。原共享load_payload对底层StoreError::ResourceExhausted改保Budget并增加拒绝数，其他损坏仍Decode；缓存读桥将Budget保StoreResourceExhausted，真实worker沿8ad映foreign.resource，不能伪装成坏codec/infra。

永久反例用真实C编译额外2MiB非零data全局的COFF，断言对象>2MiB，原cache写入v4，再把共享IO压到只余1024B。读取在原framed文件界限前ResourceExhausted，Rust分配峰<64KiB、当前费用恢复到压力前、拒绝数恰+1，producer不能调用；释放压力后同一原字节对象命中，raw页resident/IO退费且无live owner。人为先get_artifact_by_id无界完整读再做原bounded读时，最后仍相同resource拒绝，但实测分配5601846B使永久控制exit100，证明能抓“读完再拒”；finally恢复cache SHA256 `d5a38c5ae015604b5f241aec68cbbfbb334bfb3413f5419977ee8df73266adea`。8ad的单独ID费用控制在窗口前用own_payload(empty)预置保留索引capacity，避免把本次新增raw页owner记录首次增长误认为ID费用泄漏；其64B额度/2MiB坏row分配/命中修复断言保留，冷owner增长仍由8ac独立控制覆盖。

验证：首个大COFF IO 控1/1（rune7fbb2b0-054c-478c-8246-36aa3ef28bb2，编译13.80s/运行0.458s）；共享resource模块7/7（run201a24fa-762d-4d36-b846-f25be956160d，运行0.015s，76未选）；最终source_resources9+原5 cache依赖/工具/清理+正式provider换版/每required页坏失配/两端恢复立即编辑首错修复3，共17/17（run3afa775d-c544-4fc0-8dd8-a684d748df44，运行35.635s，115未选，-j1）。fmt/diff/hook通过，原AV3 warning保持。当前conformance123/source29、source_resources9。

同scope日志SHA256：8ae-c-entry-io-first.log `8200775b7b59b4206c063429c01fb7ff600df3d52889bca67e55da7620a8195c`；8ae-c-entry-premature-read-negative.log `c15dbc95eb7ec61a061b297196c09150228e4ac9424cb6cf0c6df9c0c6054c22`；8ae-shared-read-resource-final.log `9e6f2f0b0df2488a386cb5ff3d6d4a4c8ada9dfefb8ee6d04349da1e31a642df`；8ae-c-entry-read-final.log `c44807aa33a6158977a7ba109c305a6a1d1559fa62027ccb828904edb237e9b5`。

继续：此为raw entry读与所有权，CObjectEntry typed serde metadata/base64 decode输出及原工具read/encoded写/搜索listing、cold/owning Env/HIR/catalog/ledger/native lower/Verifier诊断/A2X等未核销；真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/冻结职责并集仍pending。未到work pass或独立final review。


2026-11-06 work checkpoint 8af（T-09 原C cache typed entry解码拥有型metadata；仍executing 6/13）：提交 `dab5077e5989098a60699238088669aff77fe79d`，2文件179+/8-，唯一树clean，无活动测试。资源版入口沿原CObjectEntry类型，使用Resources.decode_metadata_json的metadata-only预分析、同一原serde typed codec；先预留parser scratch/拥有型index，再进入serde。DecodedCObjectEntry字段顺序先value后ticket，整个v4/目录listing/依赖匹配期间保费用，未消费还在引用的deps时提前退费。改借用entry/deps避免拆走Vec而丢owner；原malformed JSON/不匹配格式仍miss，Budget一律StoreResourceExhausted且不调producer。legacy无资源版仍走原codec与验证流程，schema/key无变化。

大对象fixture抽共享helper，真实编译原2MiB非零data COFF输入不缩减。永久控制先通过同一原store raw-only读取测其必须的大buffer分配数（本机2），再压index只余4096B、读相同cacheentry。改前依然命中，exit100；改后entry decode预算拒绝，大分配仍恰raw-only的2，没有进入拥有型metadata codec，不调producer，费用恢复压力前；释放后同对象真命中、费用退清。人为先serde完整解码再做原reserve，最后仍Resource拒绝，但大分配3vs2触发exit100，finally恢复cache SHA256 `5ef4a9e814de6cd3f78b9d60de1035b35f460bd2bda10ef931a1c0566e692427`，证明不是仅看错误结尾。另永久控制经原store写一个有效Object载体内的坏JSON，资源路径必须miss并真实调用一次给定producer、后续复用有效替代对象，不能把解码损坏当resource或serve坏字节。

验证：改前控制exit100；改后大entry decode+原IO control2/2（rundd9e6dfa-b984-4e6f-8ae7-340d057fcb32，编译4.89s/运行0.867s，8未选）；最终source_resources10+原6 cache兼容（包含legacy v3）+真实formal provider换版且caller0共17/17（runfd18917f-3bf3-497e-8451-94c3e8c7cd28，运行32.340s，116未选，-j1）。随后仅添加坏JSON控制1/1（run5b5ab0f5-8709-4f81-9261-2e1ce47f530d，编译1.31s/运行0.281s，10未选），产品及前17项输入/helper未变，复用其有效证据。fmt/diff/hook通过，原AV3warning保持。现source_resources11、conformance123/source29。

同scope日志SHA256：8af-c-entry-decode-before.log `6915c04b29444a3acd3d723520a5b75f205c9cc64bc7a5cb0b7ef7efdca13c09`；8af-c-entry-decode-first.log `b1aa88315f8bb9e1b4951bb663561354be302416e62331a0619244226349ce09`；8af-c-entry-premature-decode-negative.log `54169ba0e3682b17a0b1ee9616ba8e036e818930ef32452dc4bcac34dd86de4f`；8af-c-entry-decode-final.log `ab3c21aa2c9ec09c5beae4490030d764f89b6b9406b909067e5397245444d0dc`；8af-c-entry-malformed-control.log `124a1b45bf560820216aa3d29955d3694bfe56af08e3dfc9467fa459f5ca115c`。

继续：cache raw entry与typed元数据已接原账户；base64 decode返回对象的首次分配/完整输出owner、原producer object read/encoded写/搜索listing、cold/owning Env/HIR/catalog/ledger/native lower/Verifier诊断/A2X等仍开放，真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/冻结职责并集仍pending。未到work pass或final review。


2026-11-06 work checkpoint 8ag（T-09 原C cache base64对象输出的首次分配/完整owner；仍executing 6/13）：提交 `0fb633398bb9ce55a6b67499965779c72889e4f2`，3文件329+/53-，唯一树clean，无活动测试。旧produce_c_object仍返回Vec/String producer，原legacy调用者形状保留；938新增的with_resources端口现在明确接收/返回Arc<Vec<u8>>与typed StoreError，唯一实际worker及全部调用/测试已迁移。private CacheBytes区分原Vec与共享Arc，不是新cache或持久wire；模式由入口决定，预算拒绝不走panic或未管理fallback。

源资源路径按encoded长度checked推算解码最大capacity并保原64MiB单对象帽，先reserve resident(capacity+40)，才建Vec、调用原base64 0.22.1 STANDARD decode_slice、truncate有效长度，再Arc构造并把同一ticket retain至该Arc。读取本机锁定crate原实现/docs确认slice上界/返回长度，不改解码算法/原字节。坏base64仍miss并退费。源producer返回不可变owner，入口adopt验证同账户/同费，冷producer在真实worker中将已读出的Vec直接own_payload后再交cache，输出全程保费用；原工具fs::read的最初Vec分配仍明确未闭，不能冒已先预留。worker不再在cache返回后重新构造owner，provider/link/reference/会话别名共享该Arc，SourceSession再adopt不重收。legacy键/entry v4/schema/实际对象字节不变。

永久allocator oracle在真实2MiB COFF解码输出那一个实际layout分配返回前读取原Resources resident（观测器仅在独立source_resources测试target、精确匹配该fixture布局，未给产品加探针/计数器）。断言原raw entry页与将分配输出同时已收费、真实输出分配恰一次；返回后精确capacity+40，adopt/外部alias不重收，最后owner释放退清。将真实reserve移动到Vec分配之后时，最终账本/字节仍会正确，但本控明确exit100：allocated before reservation；finally恢复cache SHA256 `c43cb188182b7139dd30260d0bac6fe05fd0269813778aad17f137ff487d2655`。其余repair控制更新为输出Arc活着时必须保费，不能把真实live输出费误判为泄漏；drop后仍原baseline。坏JSON分别保持miss；另用真实v4元数据仅损坏base64的永久控，必须调用一次给定producer、只保新输出费，失败decode费不泄漏，最后退清。

验证：迁移后5个cache资源控5/5（run7813ba35-0832-48f3-a99f-4ae4c7de52cf，运行2.066s）；真实分配时点控1/1（run668a3ad2-cbdc-43c5-b7df-321f43f778ca，0.438s）。反控复原后source_resources12+6旧cache兼容+真实两端provider换版caller0/9MiB SQL预算拒绝保持pointer并修复2，共20/20（run0a4a21f7-9f59-4c9e-a067-5e4878f8f7a9，运行49.387s，115未选，-j1）。其后仅加坏base64控1/1（runc94a10db-ebc5-4cbf-a77a-5842dcae2873，编译3.31s/运行1.133s，12未选），产品与先前20项输入不变，证据复用。fmt/diff/hook通过，原AV3warning保持。当前source_resources13，conformance123/source29。

同scope日志SHA256：8ag-c-output-first.log `a8959288ce728e13c4ba72d70517b0ca0a17c94e3655821c2ea2cccc28dfeb78`；8ag-c-output-allocation-first.log `7fcccdd87dd97bec6c735fbbbf5ac0bc0086ef3bb4ff9e8b0ae2364d88bc755f`；8ag-c-output-late-reserve-negative.log `65ca4387758a9513011f90301371b6617ffb518b2c4b9d1a1414158e7e1b94b3`；8ag-c-output-final.log `3e65ec5a00eccbb6f318ce4aed66cc0b8dcd0defe73c3d008b1ea7911e9c9d39`；8ag-c-output-invalid-base64-control.log `88f1d87891a5b75a53a019314a9101ad482e627c411d626b24ec4f5f33003ed6`。

继续：工具object文件首次read、C cache encoded写/生成与搜索listing metadata、cold接口/owning Env/HIR/catalog/ledger/native lower/Verifier诊断/A2X等尚开放；真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/冻结职责并集仍pending。未到work pass或final review。


2026-11-06 work checkpoint 8ah（T-09 compiler object文件首次Vec读取；仍executing 6/13）：提交 `c0cc72571cb695aa515c9fb7dd7aba2c58b0bb0e`，3文件162+/10-，唯一树clean，无活动测试。实际worker冷C cache producer不再std::fs::read后才adopt：调用read_c_object_file，在同Resources先给路径转换/metadata预留index与256B IO，打开原完成的object，检查普通文件/Windows reparse叶及原64MiB单artifact帽，再按实际长度先reserve IO才Vec。read_exact+额外1B EOF判定，不接受变化或截断；Windows只共享读、拒已有writer，并明确关闭文件后将同一ticket转resident(capacity+40)，才Arc构造/retain，无拷贝/费用空窗。typed StoreError保resource进入原worker分类，原producer/缓存/Root协议未换。工具scratch清理前File已关闭，外部Arc持有仅保字节不锁文件。

测试分配器的独立精确layout probe推广为Charge维度，原resident控制仍有效，新IO probe在真实2MiB COFF文件data Vec分配返回前检查IO至少长度+metadata256；成品数据相同且IO退为0、resident精确capacity+40、alias最后释放退清。只余1024B IO时不造大buffer且费用回原；writer占用/目录/缺文件拒绝；超过64MiB的具名测试文件metadata判超、不分配大buffer。该oversized.obj为本scope内纯反例材料，未入git，merge按整scope删除。人为将实际IO reserve移到Vec之后，最终仍Resource失败，但峰2100008B触发exit100，finally恢复cache SHA256 `b3135690fffdcac6263a8b7ac3435d8a9929fa77c494d2c787721da79537fab1`。初次反控脚本假设rustfmt多行而未找到区间、未执行cargo或改产品，finally同hash；随后按实际单行精确匹配的真实反控记录以上exit100。

验证：新file/原base64分配probe2/2（rune6698d54-708f-4e64-afde-8c78e55bcc1f，编译4.73s/运行0.971s，12未选）；反控恢复后source_resources14+原6 cache兼容+真实两端provider换版caller0/9MiB SQL预算pointer保持及修复/早期诊断残留恢复3，共23/23（run630f8c73-ad58-4cd7-8e1c-0c870da74471，运行58.141s，114未选，-j1）。fmt/diff/hook通过，原AV3warning保持。当前source_resources14，conformance123/source29。

同scope日志SHA256：8ah-c-object-file-first.log `e9315f756359af74abe723eed89f9f1c07be2debe7dc3f0d0cc6477c20f01aca`；8ah-c-object-file-late-reserve-negative.log `7b0e42b082fcc9006f83a6269da9d8f56600021d1d5f8bdea5d41d695eaaf249`；8ah-c-object-file-final.log `6696a517638f4a6753ef851c74aa99d6b5d3beb659f6e8d648b5ccba690d5796`。

继续：C cache encoded写与生成/搜索listing metadata、cold接口/owning Env/HIR/catalog/ledger/native lower/Verifier诊断/A2X等仍开放；真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/冻结职责并集仍pending。未到work pass或final review。


2026-11-06 work checkpoint 8ai（T-09 原C cache encoded写与原store封装并存窗口；仍executing 6/13）：提交 `025d3a0acac5ae1f30626127c373e9f074a472b5`，2文件285+/13-，唯一树clean，无活动测试。source资源入口以借用CObjectEntryRef+base64 0.22.1 Base64Display流式collect_str生成原v4 JSON；查锁定serde_json1.0.154 collect_str与base64 Display实际实现确认固定chunk写出、无完整base64 String。原legacy None路径保原serializer/调用形状；新路径预先json_size（无完整输出），checked预留IO 2*(JSON大小+原CHUNK_HEADER_LEN)，才exact bounded JSON Vec。该费用覆盖调用方payload与原store.frame_chunk单Vec，原BlobStore::put是fs::copy文件、不另读全Vec；原store 8MiB原子磁盘暂存门保留不替代。borrowed字段不再为本源写口克隆完整entry strings/encoded字段。Store返回时PutRequest与framed内存已释放，立即drop编码票据，再做原Conflict canonical rediscovery，避免无实际buffer的费用挡合法修复。无新cache/key/schema，未放宽帽。

真实2MiB COFF冷写控制：只留3MiB IO时允许原object文件读/owner产生，但encoded coexistence门拒绝，不造JSON/封装buf、不写artifact表；真实大分配恰1（该已准入object文件data）。释放压力同请求成功。精确layout观察器分别在JSON Vec与原store framed Vec真实分配时核仍预留全并存费，两点各实际命中1次。新字节与原legacy路径payload逐字节相等，存回原v4后命中、退清费用。把reserve挪到JSON后，最终仍Resource拒但大分配2vs1，exit100；把编码fee提前到store.put前释放，最终字节仍正确但framed分配观察捕获无费，exit100；两次finally均恢复cache SHA256 `71e264f85289c2a5e2408fdb435e62fdda0566f443b29a5861b5baa2210ecfd0`。

另控制同generation冲突：在source discovery后、其producer返回前受控插入另一调用方的合法canonical对象；本次contender仅改COFF timestamp模拟原并发差异，实际producer确已运行1次。必须保持hit=false、释放已完成编码票据再用原发现/验证取canonical旧字节，discard contender fee；返回后只剩canonical Arc费，最后退清。此是受控发布交错，未冒真实并行压测。原store私有锁/commit/cleanup无修改。

验证：首个冷写/字节控制1/1（run5faab955-e7fe-433c-8ea6-c25a89b0079e，编译7.48s/运行0.681s）；加强framed点1/1（run3d28dceb-8916-4f7e-8244-e330fb2c3261，0.723s）；canonical冲突控制1/1（runf5b7f650-14bd-43a9-942f-6f79f43fc1fa，0.635s）。反控复原后source_resources16+原6 cache兼容+真实两端provider换版caller0/9MiB SQL预算pointer保持修复/早期诊断残留恢复3，共25/25（run9e91436f-bb9d-459a-9e37-ee4b216e357c，运行60.806s，114未选，-j1；原deadline/样本无改变，未机械拆组）。fmt/diff/hook通过，原AV3warning保持。当前source_resources16、conformance123/source29。

同scope日志SHA256：8ai-c-encoded-write-first.log `ec871bcdb3550634dd468702dfb48b91cd0ada083c9658a5be53f255eb098fc4`；8ai-c-encoded-frame-control.log `60eb1e4e023d1c84a108cab3a3baacb1144002e5060633fe408d0dc9596a9675`；8ai-c-encoded-late-reserve-negative.log `546ddfd87196a93ae237fea51d42ca4f54e6c40fb871d325418fb3c2f69b9156`；8ai-c-encoded-early-release-negative.log `ee2e894257b2a16dd75e21bfa443a52cdf6bb98dc6eb073602f1f65abac677c7`；8ai-c-encoded-conflict-control.log `1df0b71a057acf639293f0664b5413a7ea52680ecfab5be4e60d2eaba84adf36`；8ai-c-encoded-write-final.log `47cb4f28e9516238e3e8f155e488b8b6b16b5e97441c692ae1d6479441046f6e`。

继续：C cache生成/负依赖搜索listing及path元数据、cold接口/owning Env/HIR/catalog/ledger/native lower/Verifier诊断/A2X等仍开放；真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/冻结职责并集仍pending。未到work pass或final review。


2026-11-06 work checkpoint 8aj（T-09 原C cache负依赖扫描owned内存；仍executing 6/13）：提交 `8f0556e1b9329eba976f40ea3bd8ebbeea60e4ac`，2文件405+/66-，唯一树clean，无活动测试。search_listing改为同一fallible实现供legacy/source调用，保持原完整递归、按file_name稳定排序的深度优先序、非UTF8/读目录失败原处理、完整路径大小写不敏感的已知artifact和.tmp-*排除。显式Vec帧栈替代host递归；Windows std1.98.0实际DirEntry内联WIN32_FIND_DATAW/共享Arc根，按sizeof和实际容量预留旧/新数组与稳定排序辅助内存，文件名转换/目录root/相对完整路径临时费在read_dir前预留。输出String容量和Vec先计费；各扫描输出ticket无空窗merge，持有至借用JSON消费者结束。artifact比较改为借用ASCII字节比较，去掉每项lowercase/format临时副本。预算拒绝为typed Resource，命中检查不把拒绝变miss再编译；未新建缓存或改变v4/schema/key。

实际独立provider+300候选文件、深层extensionless/settings、别处同名provider.obj及大写.tmp输出控制：只留64KiB index时在收集完整目录前拒，128KiB以上分配0、artifact表0、旧account恢复；放开后冷写与命中均在实际512*size_of(DirEntry)数组分配点检查预留2倍费用。原候选推导保NT/plain两种目录写法，实际观察每个记录扫描各一次（本fixture两次），未去重缩减搜索域。删除已知临时产物保持命中、增加深层任意新候选确实重编译；原legacy与resource payload逐字节相同。另在所有扫描帧关闭后的真实JSON分配点核所有listing输出仍带fee。把reserve挪到数组分配后，以及把合并ticket提前释放，两种错误实现各exit100，分别由分配点/最后消费者捕获；finally均恢复cache SHA256 `0475179f13488e7e66575b4a8e08ade7badde877754cb1124c1af1b55321bdda`。

首轮新控发现测试预期遗漏输入Arc仍活着时的resident/ownership-record index（exit100，非产品泄漏）；按实际既有owner预先adopt并比较baseline。第二轮发现原NT/plain扫描两次而测试硬写一次（exit100），改为与原payload记录数对账，保每次实际分配断言。随后扫描控制1/1（run295c12bd-5e98-4c96-a301-e63bae732d0c，0.367s）、最后consumer控制1/1（runf79a39e9-2b19-4f16-a5bd-827a4ba3c324，0.518s）。反控复原后source_resources17+原6 cache兼容+真实两端provider换版caller0/9MiB SQL预算pointer保持修复/早期诊断残留恢复3，共26/26（run631fc026-79ee-4c35-9c3b-3fc827e7a485，编译4.80s/运行63.458s，114未选，-j1，原deadline/规模无改）。fmt/diff/hook通过，原AV3warning保持；conformance123/source29。

同scope日志SHA256：8aj-listing-build.log `6d734eccf54bb9b8a1a9c9ec1edde6f46dec83e160cbb5b6cd4902bf4645437a`；8aj-listing-first.log `db0c8c202b1578f2b193b364c589c5ee0a08288ef2be1362c9e83904fac764b7`；8aj-listing-control.log `dbf38bafbddb3e7996689a952fd1c08a9dc3855d77d758f8e7f3e02f9bfab3fc`；8aj-listing-control-final.log `fc0a90c677b92ab2b62eecb32e8bc2ffe3bdecaa6ad7b75561b2242a705e2a43`；8aj-listing-ownership-control.log `046d42f83986df6d0e4971fdfe6599e0cad0dd932383aa2d92388a0d9e1a644c`；8aj-listing-late-reserve-negative.log `e1ef09ed2f2225ac6493f1b12bf4c277c2d258661d5461e58d52eefd95b8aca5`；8aj-listing-early-release-negative.log `c52993c7e2e466c63f97c64270483083b8f4ae506e7af419004afae5de48f987`；8aj-listing-final.log `c2dbe250de18700c3fe5821da4150b1f9cb8573e753f84e165800c8bcb722e8e`。

继续：外层C cache生成/其他path元数据（hashed/chain_dirs/dir_listings外Vec及gen_lines等）、cold接口/owning Env/HIR/catalog/ledger/native lower/Verifier诊断/A2X等仍开放；真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/冻结职责并集仍pending。未到work pass或final review。

2026-11-06 work checkpoint 8ak（T-09 C cache generation排序/摘要不复制全路径正文；仍executing 6/13）：提交 `6e9e2ebf98b23e02ee511ba3a67cef75ec7ddcb4`，2文件245+/34-，唯一树clean、无活动测试。gen_lines整批String格式化、stable-sort scratch及join全量String改为借用4片段Line；checked计算完整行数/Vec尺寸，先reserve index(count*sizeof(Line)+64)才exact Vec。以拼接后的实际字节比较、按相同字节dedup，原有BLAKE3按原换行位置流式输入，无全量正文/路径复制，无新key/schema。unstable仅用于相同字节的等价项，canonical结果保持原String排序；释放Vec后ticket缩至实际64B摘要String，随所有权存活。legacy和source仍共用同一代际算法。

新增控制用独立保留的原format/String sort/dedup/join算法核真实v4 payload对应canonical key：4096个重复artifact行（全部参与预留，最终按原规则dedup）、Unicode负依赖文件、互为前缀的两条header依赖路径；后者是显式保守附加依赖，不冒实际showIncludes最小闭包。只留64KiB index时listing可执行、代际Vec先拒，不造128KiB以上分配、不写artifact表；放开后在实际count*4*sizeof(&str)分配点观测原account已预留全额，source/legacy key和wire逐字节一致，最后释放owner退清。故意把预留移到Vec后，最终同样Resource错误但多出大分配1，exit100；另一错误实现按四片段数组字典序排序而不按拼接字节排序，被前缀header的独立旧算法key捕获exit100。两次finally恢复cache SHA256 `e52c2cdbe28cdb5d3e6c0fd1c4e53af2858c12d434b5cc4c01cc941d9167e885`。

验证：首个控制1/1（runab1f21cb-89ba-4f1f-8b0a-97cd4df45248，编译5.67s/运行0.802s）；最终source_resources18+原6 cache兼容+真实worker provider换版/SQL预算修复/诊断残留恢复3，共27/27（runee3b09b1-9d7f-4c8d-8e70-003d92ac8fee，编译5.84s/运行77.539s，114未选，-j1；原deadline/规模未改，未按时间机械切分）。fmt/diff/hook通过，原AV3warning保持；conformance123/source29。

同scope日志SHA256：8ak-generation-first.log `120367fac24bf07c53b124d6ec64ce8ae8a75409ff615daa7be7d77d716ace9b`；8ak-generation-late-reserve-negative.log `fc8d62a2eb6842a7bbe09b74b2f3dffd9ea67e1392be58aa00ad712e75306887`；8ak-generation-piece-order-negative.log `46e1f46123faa6dc9fb661d3090a4a98de1133cb7be5e718cafa70024cc9dbdf`；8ak-generation-final.log `599228d7c930f5f20a43680fdda8093d41e6b4b24aef912a9aa9602251f885b5`。

继续：C cache其他path元数据（hashed/chain_dirs/dir_listings外Vec、初始目录/SQL key等）、cold接口/owning Env/HIR/catalog/ledger/native lower/Verifier诊断/A2X等仍开放；真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/冻结职责并集仍pending。未到work pass或final review。

2026-11-06 work checkpoint 8al（T-09 依赖hash数组/路径与listing外层metadata；仍executing 6/13）：提交 `99c048db517414389fafe4f664c893bc84eeeb64`，2文件207+/4-，唯一树clean、无活动测试。checked预留hashed数组sizeof<(String,String)>*deps.len，才exact try_reserve；每条Windows path预留encoded OsStr长度+64B digest，预尺寸String直接Display写入，去掉ps.clone，不再在格式化与存储间复制路径。核本机std1.98 core::wtf8::Display实际逐段写出、未配对surrogate3B替换仍3B，无lossy全量中间String；String容量在格式化前已有保证。实际path/digest容量累计计入同ticket，数组/正文随最后借用JSON消费者存活。外层dir_listings<(String,Vec<String>)>数组与每条目录克隆同样在Vec/clone前预留；内部listing输出仍持8aj票据，未重收费这些String。legacy和source共用原路径、依赖hash及v4表示。

新控4096条保守重复依赖（输入列表不是最小showIncludes闭包，未据此声称原compiler生产分配已关闭）：只留64KiB index时在大数组前typed Resource拒绝、>=128KiB新分配0、artifact表0；放开后观察真实deps.len()*sizeof<(String,String)>分配点已有全额index。另一独立cold写在真实JSON输出分配点核数组+全部path+digest仍收费；source/legacy key与wire相等，返回后费用回原。缺依赖hash失败不发布/不泄漏，创建该依赖后同源修复可发布且object bytes相同。把reserve挪到数组后多出大分配1、exit100；提前drop hashed ticket被最后consumer probe捕获exit100；finally均恢复cache SHA256 `fd11417619cfb675ca8cd621b88ba6424958077d42fbc222a17344c998ddb264`。

首轮控制把producer输入deps.clone的PathBuf数组也算入>=128KiB测量，exit100；改为窗口外准备已拥有输入，只移入producer，仍不核销producer原分配。一次窄补丁定位错到先前listing测试同形代码导致编译exit101，已移除该误插并按test名限定区间，最终clean；一次补充断言patch因rustfmt单行未匹配，原子无修改。修正后初控1/1（runbe948bca-308a-488d-9e5b-815da8da6dd2，编译5.54s/运行3.614s），反控复原后source_resources19+原6 cache兼容+真实worker provider换版/SQL预算修复/诊断残留恢复3，共28/28（rundb8a09bd-ed24-4fa4-ad61-7efb68b5266d，编译4.80s/运行69.098s，114未选，-j1，原deadline/规模不改）。fmt/diff/hook通过，原AV3warning保持；conformance123/source29。

同scope日志SHA256：8al-metadata-first.log `b5cb1b6ac7fcfda3b912867ad9858d83f10f158ed1d8f25f9e968a7ef0b27172`；8al-metadata-control.log `2cfb990b6dbb42133ce32ae824f573925df1df7db86426a7d16a3d9d88c729ad`；8al-metadata-control-final.log `9b2f79894d4ac42969244aef7fc4f35ec05a947213064244ed3163b246cabf7d`；8al-metadata-late-reserve-negative.log `826924e7213e2a439188919a0aef6e2d919b5153e51ece164466b307a57f203b`；8al-metadata-early-release-negative.log `99009b95a053ff56c5be8743ddae3f9e9f1324a28df435fc2d0ca6a5d1a5750a`；8al-metadata-final.log `8885264b4e63a7a0fc2c4d1be47d9c7b8b4b33c3e9d1f4bf65e00c89b41e69ba`。

继续：C cache其他path/候选目录元数据（canonicalize/chain_dirs/dirs/rel_spec/join_lex、初始目录/SQL key）、原producer deps owned分配、cold接口/Env/HIR/catalog/ledger/native lower/Verifier诊断/A2X等仍开放；真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/冻结职责并集仍pending。未到work pass或final review。

2026-11-06 work checkpoint 8am（T-09 借用lexical路径与候选目录owner；仍executing 6/13）：提交 `74e3038fefe9adc3abb848c9cf6512dc3c8daeb8`，3文件541+/136-，唯一树clean，无活动测试。rel_spec用借用分段/共同前缀迭代，按精确相对结果长度先reserve才String，不复制两份String分段；join_lex只给借用&str栈预留，计算最终长度后先预留数组/输出并存费，再写结果，释放分段栈后仅保结果费。parent_of借用原UTF8 String子片；CPathList对chain_dirs/dirs的每个String复制、Vec旧/新并存先预留，sort_unstable+dedup后按实际保留容量/正文减费。原跨盘/大小写/NT-prefix/..不弹出首分段/超过1024候选不cache等算法不改，无搜索削减。共用helpers移至私有cache_paths.rs，主consumer真实调用该单份实现。

独立保留旧owning算法作compat oracle：16种路径×16种relative、16×10种join，各None与source account两种，含空/相对/drive/跨盘/NT前缀/Unicode/escape/互为前缀header；父目录借用值也与旧Path.display相等。另为实际分配观察，把同一私有源模块直接编入已登记source_resources allocator target（无公共测试hook、无第二份算法/工作区）。大relative/join结果的真实分配点检查预留/分段并存，candidate目录数组增长实际点检查旧/新容量+新String费，String副本分配点也检查费用；失败不改原目录值、无大副本，放开可修复，dedup释放重复正文且保留实际Vec容量。两种晚预留错误（relative分配后reserve、join分配后resize）分别由大分配数和分配点fee捕获exit100；可弹出首分段的错误由独立旧算法捕获exit100（首先命中空dir/sub/./../config边界）。三次finally恢复私有模块SHA256 `d80a3c09bea893f0b1ab99bd8b1a9cb6d0edc9b1c51f14e49ea4cb79ee0061fd`。

初次新unit误以为Resources基面index为0，实际原account自身184B，exit100；改为保既有baseline而非忽略费用。随后5/5（run66dc5573-b4eb-4a71-b17d-13d2833f255e，编译5.43s/运行2.647s）和真实分配4/4（runb4abdea8-087a-4f54-9902-a0c35b6c5ca1，编译7.05s/运行0.096s）。因本次实改路径推导，最终定向扩到全部25个abi_cache_命名反例（含原512 revision churn，6.598s；不减原规模）、source_resources21、path unit2和真实source worker3，共51/51（rund90765fd-5dd5-4a7f-a537-ab90fea53af4，编译3.85s/运行77.966s，128未选，-j1）。覆盖父路径逃逸/深层无扩展名/传递subpath/同名output/cwd/legacy升级等全部原相关cache反例，未跑全experimental。fmt/diff/hook通过，原AV3warning保持。ABI lib35/source_resources21/conformance123/source29。更正前checkpoint8al编译时长手写4.80s为笔误，8al原日志实际4.00s，run/69.098s/28项不变。

同scope日志SHA256：8am-paths-first.log `71868340b0828f8905a80d61fee73d50bea8dc7abc4fe9279aea59901d6b57e1`；8am-paths-control.log `2f8cee0a967e6640accce60a904aa96a6ed9c5d9c81604ca34134366caf87a24`；8am-paths-allocation-control.log `d04eb671223e30e2d63c48dade572c7a09e104f60435915a290145e09c12e931`；8am-relative-late-reserve-negative.log `d31519ccad3aec691d51b9885afb240343879633e0ea0c9a2f8bd8a61f1b3dd7`；8am-join-late-reserve-negative.log `c17b3c7f36cd80165afe7028e70698269647e251ac73c3bede144855e1764853`；8am-drive-pop-negative.log `2f436c2c4d99b878a6361d2e8914256d52364ed648b7e28124fb5d58b7811961`；8am-paths-final.log `079abaece8affaf7673d1d4dcfb7c5b9955b276f8fbf9aae76b76b0eb057d6b2`。

继续：C cache初始/依赖canonicalize临时与目录display、artifact路径归一、SQL needle/source/toolchain/key元数据及原producer deps分配仍开放；cold接口/Env/HIR/catalog/ledger/native lower/Verifier诊断/A2X、真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/冻结职责并集均pending。未到work pass或final review。

2026-11-06 work checkpoint 8an（T-09 source/tool/query身份owned文字；仍executing 6/13）：提交 `7e67d449f9ff7d99561478ef8e72bdf2ac0064af`，3文件309+/63-，唯一树clean，无活动测试。source_identity在原8KiB streaming BLAKE3读前预留64B digest+精确filename|digest输出并存，exact String逐段写，digest释放后ticket只保最终String；公共c_source_identity legacy签名/错误文本/内容维持。hash_file实现移入单一私有模块供source与dep使用，未复制算法。命中逐dep的临时64B结果也先预留，预算拒绝hard Resource而非miss再生产；cold hashed ticket原已覆盖digest，不重复收该结果费。toolchain_text借用原OnceLock解析出的真实exe/version/hash/flags，exact String写原格式、flags不另join全String；target常量String也先预留。discovery_needle精确统计\\/%/_转义增量，在原account先reserve才逐字符写原LIKE文本，去掉连续replace的全量副本，持有至stmt/rows读取结束。原query/key/schema及解析工具owner未换。

分配点控制观察真实64B digest与final source String时同时有全并存费，错误文件IO退费；大工具banner/Unicode/未配对UTF16 path Display与旧format+flags.join独立oracle一致，真实工具文本分配先收费。LIKE控制与旧三次replace+format oracle逐字节相等（含反斜杠、百分号、下划线、Unicode），在已拥有toolchain输入费之上再预留query结果，分别释放退清。三种晚预留（digest已分配、工具String已分配、query String已分配）各exit100，遗漏反斜杠escape也exit100，均finally恢复cache_paths SHA256 `2d99c56b98a1aa05901bb2b7ab893267b636c92879479e05d2147d328bd0e84f`。此未核销std File::open路径转换临时/原首次工具resolver缓存owned内存，见继续项。

早期只余64B的SQL控制现在如实在更早source身份metadata门拒（typed Resource、不调producer）。为保持原SQL巨大行值控制的区分力，第二阶段放开至1MiB余量让身份/query已准入，再读被改坏的2MiB borrowed row；仍必须在复制前拒，释放压力修复命中、pointer/producer规则不变。再次故意在ID检查前to_owned，该控制heap peak2099022B捕获exit100，finally恢复cache SHA256 `a77586b0ab91c5aa3260c9a8948ee3f0a59d1d97bab5f521736d1ecd4616caeb`，并非靠更早拒绝掩盖巨行反例。

首3/3（run32445836-6abb-46f6-9cbf-7e938d7e46e9，编译5.73s/运行0.565s）。最终path unit2+source_resources23+原C/普通cache命名反例24+真实source worker3，共52/52（runada91a51-cc03-46eb-92bf-42aece0e339d，编译7.86s/运行103.915s，129未选，-j1，原deadline/规模不变）。本次用锚定/^cache_turnaround::abi_cache_/避免8am广名称匹配再次选中performance子模块；8am原512 churn当前代码路径/输入未变（它调用未修改的c_object_key/原ManagedStore），复用其6.598s绿，不减其512规模。fmt/diff/hook通过，原AV3warning保持。ABI lib35/source_resources23/conformance123/source29。

同scope日志SHA256：8an-identities-first.log `d047ec3e10ac6efca08d60c92f3f03813c0a05145f65f3db2d5abb0e8488b1a3`；8an-digest-late-reserve-negative.log `44071b5f9861b08f70810b36f4cfedb609b9d146c861a1f54f6fff5e39e3ab43`；8an-toolchain-late-reserve-negative.log `f75f34c04d127b9ea6d596e90ff1b1a18dc153d2f904e7891f50d0ac095f5d57`；8an-needle-late-reserve-negative.log `70797a1a6da76cf4d8a3e8659b3fc05181f9a91833e28a46c58f6f19cf1fb7f1`；8an-needle-escape-negative.log `d3aad780ee40357ae879e554929e5bdd2d6db4a7532d740f7c782b53721e7840`；8an-discovery-premature-copy-negative.log `588ed3a9d19196d2309cc153db9ee63eb328a351081d36df4c9b4f4a077299fd`；8an-identities-final.log `481c2814f844f5fb5b0a2afbda1abee8394330e6bc23b23f1ad24b079f52047c`。

继续：std File::open/canonicalize等Windows路径转换及临时PathBuf/display、artifact路径归一、原producer deps/工具resolver及store QueryKey相关元数据仍开放；cold接口/Env/HIR/catalog/ledger/native lower/Verifier诊断/A2X、真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/冻结职责并集均pending。未到work pass或final review。

2026-11-06 work checkpoint 8ao（T-09 Windows native转换/canonical目标/artifact归一；仍executing 6/13）：提交 dc4a069ccee37d6e307513f85b3ad7be836f8db6，3文件463+/86-，唯一树clean、无活动测试。按本机std1.98.0 with_native_path/to_u16s/get_long_path实际代码，verbatim与短absolute路径先预留精确UTF16输入buf，长absolute按旧/新转换上界、relative按可能长CWD的临时namespace上界预留；guard在std打开返回后即退，不占后续hash/data窗口。覆盖source/header hash_file与原read_c_object_file打开；relative read_dir也补转换临时费，absolute扫描原frame16*path+8192已覆盖。IO与预算错误保持区别，缺依赖仍miss，而Resource不再混入miss/producer。

canonical_text保持std access_mode(0)/BACKUP_SEMANTICS/默认share与DOS normalized选项，直接用同一GetFinalPathNameByHandleW先查实际所需长度，checked预留旧/新UTF16 Vec并存再分配；若长度变化循环扩容仍先reserve。输出UTF8长度由decode_utf16计算（未配对按原Display替换），先给数组+最终String并存预留，释放数组/handle后只保结果String费，无中间PathBuf/display副本。初始src/include、依赖parent和artifact parent共用此路径；资源错误hard拒、原artifact IO失败才按原lexical display fallback。artifact basename/斜杠归一直接写预尺寸结果，CPathList接手每个数组/正文费用。原目录/次序/重复项/key/schema/old API语义不改。API依据：[Microsoft GetFinalPathNameByHandleW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew)（不足时给含NULL所需长度，成功长度不含NULL、DOS/normalized均0）；relative临时上界参考[Windows路径限制](https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation)及本机std旧/新缓冲机制，非新目标路径长度帽。

真实短junction→长Unicode目标控制：所有junction仅建于统一plan-artifacts/tests的具名fixture，绝不在worktree。预算只余1024B时，查得更长目标后在Vec前拒、整buffer分配0；可用时在native输入Vec/实际目标UTF16 Vec/最终String分配点各命中1次并核相应费用。结果与std::canonicalize.display及原artifact归一算法相等，missing IO/原fallback可续，budget不能伪装IO fallback。长absolute与合法短junction CWD下relative两种std native转换均验证，新account隔离历史压力peak：实测UTF16 1942B/UTF8 973B，long heap5816B≤预留11848B；relative heap2915B≤临时预留524296B，最终仅剩结果费。曾尝试直接设置长CWD，本机OS返回206，未执行该长CWD helper，未冒PASS；改用本机合法短junction CWD，仍解析同一长Unicode目标，并直接观察relative展开分配点，CWD guard始终恢复。每例只unlink junction，目标keep文件不变。额外核12个canonical fixture根，剩余short-link为0；wt-guard: clean，无任何reparse point（本次是只读核查，不是merge清理）。

新artifact集合入账后，旧gen控制在更早path预算拒：改为原4096项全保、producer内在初始集合准入后才施加64KiB余量，仍单独验证generation Vec拒绝；其分配点最低费也加上已拥有artifact集合，避免已有大票据掩盖缺费。重复原晚gen预留错误，仍多出大分配1/exit100。另5错误：UTF16数组后reserve、输出String后resize、预算伪IO fallback、relative native少计、long native少计，分别由实际分配/typed分类/分配点/全新account真实heap峰捕获exit100。finally恢复cache_paths SHA256 56241ef875c492ecbda2f881c9484b910ea6dcc2278b65bc038aa51494392083，gen恢复cache SHA256 d301b3a7fc5547c1f867d4951d00ddaa11b464bc07dedff242b5cf086a09daf8。一次输出mutator锚到rustfmt前一行导致实验版本编译101（非反控成功），已恢复并按多行区间修正，replay exit100；其原log不覆盖。一次JS raw字符串内PS反引号导致脚本解析前失败（无命令/写入），两次patch锚不匹配原子无修改，均已改正。产品首编译map_err闭包入参ResourceError/StoreError不符exit101，显式StoreError映射后通过。fixture首MKLINK因正斜杠判Invalid switch，改为Windows分隔符后成功；未改产品路径算法绕过。

验证：修正后的依赖/SQL/gen/listing4/4（run9d1fad69-3062-4892-bb48-b1de92859453，编译2.29s/运行3.398s）；canonical/native/source分配2/2（rune58fca9d-0649-4c8d-820b-2698fb3b9c70，2.54s/0.512s）；Unicode1/1（run123f2fc6-f8cc-4e9a-9f70-6af8b69da3bf，5.92s/0.263s）；完整native控制1/1（run9a727286-6840-4690-9528-0524ea1a0b7f，4.27s/0.376s）；具名数字输出1/1（run40eeddfb-c4ce-4ef1-a710-82dbf1d12ae6，2.85s/0.282s）。反控复原后path unit2+source_resources24+原cache直接命名24+真实source worker3，共53/53（run8c6eb2d6-9ae4-4402-8328-60435735bf01，编译4.25s/运行88.474s，129未选，-j1；原deadline/规模不改）。8am旧512 churn路径/输入仍未改，复用其结果。fmt/diff/hook通过，新临时test unused warning已由直接consumer测试消除，仅原AV3warning保持。ABI lib35/source_resources24/conformance123/source29。

同scope证据SHA256：
- 8ao-artifact-budget-fallback-negative.log fac3f2eab67979dbf8c533314e5a5c48a5e4207f25d0231d021d72ff194e6c38
- 8ao-canonical-allocation-control.log 9b9a96bda4276f443f18c3eaf6638c949ec4f8088137c67f787e7c2a72d4bdeb
- 8ao-canonical-allocation-first.log d631cfbd70d6f9da502086b10309a9f8a2f0c3ed0b45676e0a584e8061b9db9c
- 8ao-canonical-output-late-reserve-negative.log 058b9e884c4dd98855d6220a5a985b7616b9dd2de4c3063c31c1622606c12cc9
- 8ao-canonical-output-late-reserve-replay-negative.log f684e10435b42c01ba341304b15bba7b3937fb21023c9ea3e28902b38368ced1
- 8ao-canonical-unicode-control.log 5bd55ee5defbee6e587c47c58f71922b2f935cad31d22f7ecd6c24efeb37f799
- 8ao-generation-temporal-replay-negative.log 3a14aa0d240c23a4b3643cc639a6f99b949961b1aa33839aa26989f427832a86
- 8ao-long-native-undercharge-negative.log 4ba5143da5f3ca5b2d7299ae71c4f311f9d391206449bc238472fc5da89b0f2d
- 8ao-native-conversion-final-control.log f91257c7f70a92b45ede96096bbbeadfc641b29c1423eda3e6e57e4717f3cfb6
- 8ao-native-conversion-measured-control.log 9f283f788813154bc6fcc6eaa26419e728e7d8ae21258c5f1072db2e76737b18
- 8ao-native-conversion-peak-control.log 88d5b75ebef4ddc2620027e2012eb38f0a96dfb44337cb56a702603496aaa3a0
- 8ao-relative-native-undercharge-negative.log fe2b96f1a53fbd3beb58d3de0ea5714774870197073faec1a1b64a4ead9ea9e9
- 8ao-wide-late-reserve-negative.log 763224f604d8c7b995a5a5753e7414628673bd2ab5ea76bbb04e77b4a6067fe7
- 8ao-windows-paths-control.log 4d7d5b5fec5722931a9def7ce4e7837e42e181c8f08c0803bac4dcedcfb9016f
- 8ao-windows-paths-controls-final.log a1af66acfe5cf0bf9d1e7af01e95a439e09f61f4ebb956b55a46647da35c7cd7
- 8ao-windows-paths-final.log e1f9a0c43885200ae1e9087bad87f603a80cb8fa9ee855da1df089ae36fbbf7d
- 8ao-windows-paths-first.log cdce7e09d251530fc5b9febe440e5d69c86d52116a4ac383e65fedd75d57a110
- 8ao-worktree-link-guard.log 167c295822afffe6c774971b54930aaddc12b6bc5f922970d25c9fb7e9911e6a
- 8ao-link-cleanup.json 54d32252a58323e38883fbc8c065def2405f22459bbd6eb709a1b77cabebe2e8

继续：实际ctool producer deps/输出与工具resolver owned缓存、store QueryKey/hash/canonical/内部metadata等仍待核；cold接口/Env/HIR/catalog/ledger/native lower/Verifier诊断/A2X、真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/冻结职责并集均pending。未到work pass或final review。

2026-11-06 work checkpoint 8ap（T-09 冷工具image整份hash副本与showIncludes文字副本；仍executing 6/13）：提交 da4ae71b64228cca375b83a7348652b2a41a8e27，4文件78+/23-，唯一树clean，无活动测试。实际ctool.resolve_cl原fs::read(cl.exe)再blake3_hex，678008B整图只为digest；改为File+固定8KiB blake3_reader，source/header与工具共用唯一流式算法，未复制hash loop或改字节身份。公共原blake3_hex保持；reader只负责raw bytes，调用方仍拥有Reader及预留，未借此声称已给tool resolver所有全局metadata/子进程计量。cache_paths保原native路径准入与typed IO映射后委托reader。showIncludes提取返回原行借用slice、Path也先借用做存在/去重，只有真正新增依赖才造PathBuf，去掉每条先String再PathBuf及重复/无效路径的副本；旧drive-letter/locale前缀/first-seen顺序/UNC未扩表面均保持。

新实际冷进程控制直接在首次find_cl的OnceLock resolver调用窗口观察>=64KiB分配，包括真实vswhere/banner解析；修复前exit100，原大副本1；修复后0，真实cl.exe678008B，返回hash与独立fs::read+blake3 oracle相等。source2MiB原hash边界/缺文件/堆控制复用并经过同一新reader，heap峰194B不变；source digest/native/最终输出预留点控也通过。新增include extraction unit核英文/中文prefix、Unicode/空格路径及无drive/UNC原拒边界，结果指针必须在原行范围，非分配后泄漏冒借用。

验证：首红run52f72984-fd9a-4156-84b0-7a72baa262ac（编译3.38s/运行0.171s，exit100）；修复后冷工具/source hash/预留点3/3（run35a5f244-a3f0-4f5c-968d-a19c1f5de480，运行0.438s）。最终path/include unit3+source_resources25+原cache直接命名24+真实source worker3，共55/55（run98ba5d56-63ac-407b-87e6-bfe2c99e35c7，编译5.94s/运行111.425s，129未选，-j1；deadline/原规模不改）。8am 512 churn输入/原put路径仍不变，复用其已通过证据。fmt/diff/hook通过，仅原AV3warning。ABI lib36/source_resources25/conformance123/source29。

同scope日志SHA256：8ap-tool-image-before.log a9b4db0ead37f8e7ffb34c878c7df3bd288baf71153cbae6b943f7e2d15df40d；8ap-tool-image-streaming-control.log 9aedd6d5a0fc9ded3d77424ecc4e0c4621ffe7f1fabcd831196fef4ffcf4f643；8ap-tool-dependency-copy-final.log aa13f732990bcf0335b04b4e28c175ea1d8980a227a8ec8cf4173bd4cdfa11fd。

继续：实际依赖Vec/PathBuf与工具stdout/stderr/全局resolver metadata尚未预留；读到ctool discovery的run_deadlined还走core::link::run_deadlined_split(CaptureCaps::none())，不同于compile的1MiB×2原capped入口，需核本包发现阶段与原完整窗口/流预算的接线，不据正常banner小输出冒cap已闭。store key/hash/canonical/内部metadata、cold接口/Env/HIR/catalog/ledger/native lower/Verifier诊断/A2X、真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/冻结职责并集仍pending。未到work pass/final review。

2026-11-06 work checkpoint 8aq（T-09 工具发现raw流门不再绕原1MiB帽；仍executing 6/13）：提交 c5cbf68b989f6aad16e85b1878f393be20f38bd1，2文件72+/3-，唯一树clean，无活动测试。实际ctool discovery原run_deadlined_split用CaptureCaps::none，合法版本行可伴随2MiB help仍被find_cl接受。其私有发现入口改走同一原run_deadlined_captured，stdout/stderr各1MiB，与compile共用TOOL_STREAM_MAX；发现10s/编译60s/原flags不变，保原contained-start/job/tree-kill/reader-reap，不截断为绿、不改core legacy wrapper。lossy字串沿用原内容，into_owned避免Cow已有owned时再复制；新字段/descriptor/key版本无变。

新控制在统一scope造test-only fake-cl.cmd：输出合法Compiler Version 19.44.35222行和2MiB raw burst，分别stdout/stderr。父用原受限执行器启动全新精确case子进程，工具override和OnceLock仅该子进程持有，父/其他libtest不改env；不能靠已warm工具绕过发现。修复前真实子find_cl接受该identity，父exit100（runad26709a-bd5f-4c95-9206-6f38cf63acf4，编译约7s、运行0.300s）。修复后两流均由core OutputCapExceeded拒，真实cl冷解析/内容hash正常。首次修复回放测试预期误写OutputLimit，实际已正确拒为OutputCapExceeded（stdout observed1048618、cap1048576），保该红记录，改为真实enum名后3/3（run969a8501-4d62-426a-a3d9-88ec52edea74，编译3.59s/运行0.897s）。该observed允许原chunk读取超出后拒，未据其声称恰1字节/新内存预留已闭。

最终按本改动仅选发现两流/真实冷工具/原cl解析/include slice/实际C依赖失效/foreign toolchain miss/真实两端provider换版caller0，共7/7（rundd851a32-4388-42b0-bd50-c8df6b347dbc，编译3.19s/运行26.144s，178未选，-j1）。8ap原55项身份/依赖算法/配置/输入未变的证据复用，不为本流门重复全部allocator/churn/性能。fmt/diff/hook通过，原AV3warning保持；ABI lib36/source_resources26/conformance123/source29。

同scope日志SHA256：8aq-discovery-cap-before.log dfed905a743f719a5c967b300d1fd31b162313330fcfe35795504b71aa2ee416；8aq-discovery-cap-control.log ca841f6228f6ad618e51d635e9a69691e5c8e8fd5c1675057336d52f4aee4841；8aq-discovery-cap-control-final.log 91cd43ebf77cd234a196515c4a7beb6a26c8622c1e8fd357928ec56e5998b9a2；8aq-discovery-cap-final.log 2a3c815baa7799a33478aa93ee9e9d12e2736d5bf122d159198d6c1857603cff。

继续：这仅关闭raw发现流门。CToolError当前只保human String且Source把工具错误映射infra，后续reserved/typed producer入口还需保resource分类；实际依赖Vec/PathBuf、captured raw/decoded outputs及resolver metadata未预留。core reader虽帽后拒，extend增长/64KiB末chunk及lossy/错误拼接并存须按真实容量计费，不能只按len/父exit0。store key/hash/canonical/内部metadata、cold接口/Env/HIR/catalog/ledger/native lower/Verifier诊断/A2X、真实scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/原桥export、原6×30/P90/Q4/SD/939/冻结职责并集仍pending。未到work pass/final review。

2026-11-06 work checkpoint 8ar（T-09 实际C compiler依赖集合预留；仍executing 6/13）：提交 927b0241b321fd7239eede985a89ca82d51b8176，7文件267+/48-，树clean。ctool原showIncludes walk提为唯一共享ctool_dependencies，公共旧入口也使用同算法。每条借用路径的native存在探测先reserve；真正新增项先checked预留Vec旧/新数组并存与全部PathBuf正文，再try_reserve_exact/to_path_buf。缺文件/源码/重复项排除、locale前缀、首遇顺序均保持。source ticket由prepare_provider外层持有到cache完成依赖hash/generation/JSON消费，错误同样释放；actual compile的Resource失败以CToolFailure::Resource保留，source映射StoreResourceExhausted/foreign.resource，legacy CToolError签名保持。compile原OutputCapExceeded也保Resource；discovery类型/raw与decoded输出/全局resolver所有权尚未关闭。

永久控制：257个真实存在header配合合成showIncludes行，包含源码/缺项/中文重复/唯一长尾；该fixture只验证同一实际parser，不冒称真实编译器最小include闭包。压力在256项Vec扩容前拒，目标大数组0次分配；接受时512项旧/新数组并存和最后唯一PathBuf正文的allocator分配点均已有完整费用，顺序与retained实际capacity精确相等，drop回baseline。另实际cl编译stdint源先产object、依赖准入拒为Resource，随后原入口修复再编成功、含stdint.h、不含source。三个实际错误对照均exit100：Vec扩容后reserve观察禁止数组1次；每条提前PathBuf克隆观察最终正文2次；Resource伪装Tool由typed断言拒。finally字节恢复ctool_dependencies SHA256 89dbe8ad0c07a7b69121393c460b9a98d07da52a102873da1d3283ff960bd592、ctool 0269db87850429cc6ea7a7f5f0f8824b0e65224b4b66137746116668e37d47ed。

验证：real compile+allocation 2/2（run8bb99d55-5c4e-4b76-a97f-940a58a84c1b，编译5.79s/运行0.566s）；最终typed/allocation/borrow/include/toolchain+实际source缓存压力repair/early residue/provider换版7/7（rundb466fcc-1e57-4e7c-8665-5b3b5cb7501f，4.00s/48.385s，180未选）；随后原真实include失效重用+系统header完整闭包2/2（runef5ead9c-706a-4201-bfbc-84f284a75d8e，0.18s/5.286s，121未选）。均cargo nextest --manifest-path experimental/ace/Cargo.toml --config-file .config/nextest-full.toml -p ace-abi，相应--lib/--test source_resources/--test conformance具名过滤，-j1；原deadline/样本不变。fmt/diff/hook通过，仅原AV3warnings；ABI lib37/source_resources27/conformance123/source29。8ap/8ao中未变的metadata/path/GC/大样本证据复用，不重复跑整包。
同scope日志SHA256：
- 8ar-dependency-admission-first.log 69d685bfdce757bbbbfaa8886c67d222ae36846f96fe0bba84f48424fc350dae
- 8ar-dependency-allocation-first.log 2a1f5cf7e34600ee7d5c226573e8eb234b028941668767e9418f79f133e521b3
- 8ar-dependency-producer-final.log df6f79685ffdef3ac2b2cd0ee607355449636736df86467596da07e1a077c600
- 8ar-late-dependency-array-negative.log ebb0146c9ba4f5ba3fd62ddeace24d7c280f9433f79df24bf5dba701bf81e01c
- 8ar-original-include-closure.log 858658eceb12f01d4efd6c050aac9da4ef11eee641517d1b4b0499102e88a0f6
- 8ar-premature-dependency-text-negative.log 1caf6053d163253f7412121a7ac0506d1c227f9923cccb693162043a86092ba0
- 8ar-real-compile-admission-control.log 6460bd06a2ec33c01e3c4f24d4d14297000db7b87f0115e1cd503aa3d91ec40d
- 8ar-resource-as-tool-negative.log 57d6973b10114980fc01d0ef86d0bcd260ab7ce67185957c4d3cb55ea233df29
继续：capture raw/decoded输出、工具resolver全局metadata、store key/internal metadata、native shared lower/诊断与cold owning Env/HIR/catalog/ledger/A2X、真实scratch峰/request128/namespace128、完整certificate/daily无spawn/SDK/桥export、原6×30/P90/Q4/SD/939及冻结并集仍pending。未到work pass/final review。

2026-11-06 work checkpoint 8as（T-09 源码compiler raw capture预分配；仍executing 6/13）：提交 0b474a0b2d298bb4b3e45fefc958faf7682f9186，7文件344+/39-，树clean。core添加run_deadlined_captured_into，调用方提供empty且capacity覆盖finite cap的两Vec；不合格在spawn前受控InvalidInput拒。与legacy/typed共用原唯一inner executor、Job、deadline、reader/reap/channel，非另建执行器。reader读出一chunk后先核总字节再extend，超过仍OutputCapExceeded/observed>cap而非截断成功，消除超帽chunk触发Vec再增长。源码ctool_capture在两Vec创建前reserve原stdout/stderr各1MiB合计2MiB IO；ticket随raw字节穿过工具结果消费、字段drop顺序先bytes后ticket。公共legacy入口保持同core算法但不借此宣称已有source资源账户；无caps/deadline/flags/schema变化。compile_resource入口把IO不足保Resource，实际cl依赖准入/修复unit也改走Some(Resources)，各错误/成功退出IO均0。

永久控制：source actual module allocator下8191/4093两异尺寸，预算只差1B即在任何buffer/child前拒，marker不生成；两个实际allocation点都有完整IO费、各命中1，raw输出/错误及CRLF保持、容量准确，drop与missing-tool Spawn错误回baseline。core用65539B完整0..255字节模式分别写两流，包含无效UTF8/NUL/CRLF；返回两个地址与调用方原Vec均相等、capacity不变、exact cap接受；两流4097对4096超帽拒、3种无效buffer不启动marker。晚reserve错误在拒绝前观察buffer分配2次exit100；丢弃调用方Vec错误由返回stderr地址变化捕获exit100。finally恢复capture SHA256 61f888473051d9d0619e8e875fe566817fd055654aa772da2b26836e2ba59ae4、core link 693bf07ee73fb1825ffbb8ea1b2b89cf49e4dd4c52a98adaa8d2c5b94e2b959a。

验证：首编译test未限定UNIX_EPOCH exit101（保留，不算反控），补std::time后capture+原link_capture全模块9项+actual compile共11/11（run7449e727-b809-4cc0-930b-d5729961e734，编译7.44s/运行3.306s）。反控恢复后实际source预算/清理/provider+原include2+发现cap+typed compile/raw+core原7/new2+containment lib，共18/18（run2d0158d5-96b9-4073-9ebb-6494a44b2d7d，10.39s/105.391s，553未选，原各deadline不改）。额外legacy wrapper四生命周期3PASS/1FAIL（run5f8896e2-2dc7-467e-a188-c26b5081c56f，0.37s/42.132s）：large_output_does_not_deadlock原3000行/30s触发Deadline，case39.646s；collect deadline/直接deadline/descendant success通过。保首次红，未缩输出/加deadline。只读现场看到lang-941大批A2X nextest及多个cl并发、可用RAM约9GiB；只能视为并发环境线索，未证明根因。对同代码/同3000行/同30s单例重放1/1（run774d0818-1be8-4c6d-b0a0-c883bf7d9da5，0.20s/0.119s，case0.115s），暂记瞬时红/隔离重放绿，最终冻结门仍须核。未停或修改其他任务进程。上述均显式manifest/config，-p ace-core -p ace-abi --lib --test conformance --test source_resources并具名过滤-j1；mutant用所属package/target最小过滤，输入未降。fmt/diff/hook通过，只有原AV3warnings；source_resources28/ABI lib37/conformance123。
同scope日志SHA256：
- 8as-capture-buffer-control.log 7ea39e8e8cd71d5ec7a49015e41458b93dd43ae8acbf2f305820bc7449e64ebe
- 8as-capture-buffer-final.log 13bbf387abc8a758fabede21446e5351ebfb3ea6991b30e72cc50f00a4f2d85f
- 8as-capture-buffer-first.log 038167721138d7448d5a0b7af302ca68bea277dc7203f1a35be960c3e447f4cb
- 8as-discard-caller-buffers-negative.log da2e06fdc2d98a78536996d1016c1951595cff3efc1d109231c5dff0e05ac4f2
- 8as-large-output-isolated-replay.log 8c5e288fb3ff731ad275c0cfbcfcad1f9ba10890b3900be76fe0aea2aaa9cdaf
- 8as-late-raw-reservation-negative.log 928b66f99999e3134eecca432887ce034de82722da8d90a46a7f54c3e5d9aa3f
- 8as-legacy-wrapper-lifecycle.log 9e6475203ffaf082eda321ca667f020b12582b4fe398c85b613d119d2e3cd954
继续：raw source capture已补，decoded输出/错误拼接及discovery/resolver所有权仍开放；接下来合法UTF8借原buffer、invalid UTF8精确预留。store metadata、native shared lower/诊断与cold owning Env/HIR/catalog/ledger/A2X、scratch峰/request128/namespace128、完整证书/daily无spawn/SDK/桥export、原6×30/P90/Q4/SD/939/冻结并集均pending；未到work pass/final review。

2026-11-06 work checkpoint 8at（T-09 compiler输出decoded文字；仍executing 6/13）：提交 8f3c4e478551ce3778b87f40a30660f1c506c8f6，4文件164+/8-，树clean。compile_c_output现在把Captured原字节及ticket交给同一依赖parser消费，不先造整份stdout/stderr String。ctool_text的合法UTF8 Cow借原raw buffer，错误流仅编译nonzero时解码；无效UTF8使用本机std1.98 String::from_utf8_lossy相同bytes.utf8_chunks边界，checked预数每个valid段及每个invalid段3字节，先reserve IO再try_reserve_exact并写最终String，不靠std默认增长/重复Cow.into_owned。raw fee与lossy费并存到各最后consumer，owned Text字段先正文后ticket。None legacy入口也共用同文字语义，不增协议/阈值/工具flags；错误拼接String及其后续诊断owner仍开放，不据decoded端口通过冒全诊断已计费。

永久allocator控制独立与原String::from_utf8_lossy比较：空/valid/NUL/CRLF/中文emoji、孤立非法首字节、截断多字节、overlong、surrogate、不连续invalid片段。owned容量等于oracle结果len、保留actual IO直至drop；借用指针等于输入。在raw2MiB同账户预留下，1MiB全0xff结果恰3MiB，剩余额度差1B在大副本0次分配时拒；可用时3MiB allocation恰1次且分配点已具5MiB总IO，释放只回raw2MiB。IO已满8MiB时合法1MiB π文本仍原地址借用、whole-copy0、无额外收费。错误对照：晚reserve→拒前3MiB分配1次exit100；valid偷偷to_owned→5B实际正文未计费用断言exit100。finally恢复ctool_text SHA256 69a302feada14e9450c082a6162e02feaa668e258b3924f88280549c8574b08c。实际cl unit增加真实未声明符号错误，public legacy与Some(Resources)诊断文字一致、错误不发布deps、退出IO0；字节解码独立oracle由上述direct port提供，不把两个共用decoder入口冒独立解码器。

验证：新text+real compile2/2（run933891c6-34dc-4147-86e6-ac352502cafd，编译6.00s/运行0.390s）；错误对照恢复后text/raw/deps三个allocation端口+real compile拒绝/repair/diagnostic+原真实include2+actual source压力repair/early cleanup/provider换版3，共9/9（run17a2f323-5cf5-4286-93e5-10aac5744a5c，6.14s/62.286s，563未选，-p ace-core -p ace-abi --lib --test conformance --test source_resources具名过滤-j1，同manifest/config）。8as core raw executor/legacy生命周期的代码与输入未变，复用其18项/3项及large-output隔离绿，不再次重复core矩阵。fmt/diff/hook通过，仅原AV3warning；ABI lib37/source_resources29/conformance123/source29。
同scope日志SHA256：
- 8at-copy-valid-output-negative.log 75f7ad5a10c03af8bf5165a3b762270f58d13cf9a2215d7354c6b905e5df7bb8
- 8at-late-lossy-reservation-negative.log 1bfb99b9f3f6da8874697f70b9a02e81ba40c715b8171913910116066e989114
- 8at-output-text-final.log 1fecf04fe58df074a9a325aa7e898cb2d76fae268b5989a43ee5a1ee70965e23
- 8at-output-text-first.log a207e8bc1249a3344e4bc77dbe99c2c5a43ff60bc40897e78f769a56c5189747
继续：工具commandargs/native探测/resolver owned metadata与discovery输出仍开放；nonzero错误拼接/完整诊断owner、store QueryKey/hash/canonical内部metadata、native shared lower/完整诊断、cold owning Env/HIR/catalog/ledger/A2X、scratch峰/request128/namespace128、完整certificate/daily无spawn/SDK/桥export、原6×30/P90/Q4/SD/939/冻结并集均pending。未到work pass/final review。

2026-11-06 work checkpoint 8au（T-09 discovery raw/text准入、typed失败不污染cache；仍executing 6/13）：提交 3e1b5e8f352c0f5ff8bf31cd7d356fe3cf420a20，2文件193+/34-，树clean。find_cl增加private find_cl_admitted(Resources)，Source cache与实际compile接此入口。vswhere与cl banner发现共用ctool_capture原两流1MiB/2MiB IO预分配及ctool_text borrowed/精确lossy准入，不再发现时复制完整valid文字；原工具顺序、10s discovery/60s compile、flags及fingerprint字段保持。唯一OnceLock以Mutex串行初始化，ordinary Tool失败保持原cache行为，而任何Resource（预算/超帽/decoded不足）直接返回、不写process cache；避免一次请求不足永久污染后续修复。Source current_toolchain收到Resource硬报StoreResourceExhausted，不落ctool-unresolved placeholder/miss/producer；legacy未资源化的placeholder行为保持。未宣称command/PathBuf/版本banner/globalcache元数据或错误拼接所有权已计费。

永久fresh exact-child控制（也隔离bare libtest并发）：新Resource先压IO到6MiB+1，2MiB discovery准入拒；OnceLock仍None/current回原压力，无tool identity发布。再从实际produce_c_object_with_resources调用同冷发现，要求typed Resource且producer闭包不可达，排除吞错成miss。释压后在同一子进程换新request账户，test-only over-cap-cl.cmd分别向stdout/stderr输出合法banner+2MiB，均Resource/OutputCapExceeded且不cache、IO0；override只该单用例child临时持有，恢复原值后真正MSVC发现修复成功/非空version/64Bhash，原账户也可用同地址identity，修复新账户IO peak恰2MiB/current0。测试首红是把先前6MiB压力peak误当2MiB发现峰（product已修复），改用隔离新request账户测峰，保原失败log，不降预算/样本。

三个错误对照exit100：把Resource写OnceLock→明确cache poisoned断言；超帽改Tool→明确wrong cap classification（stdout observed1048874/cap1048576）；Source current_toolchain吞Resource→实际调用了禁止的producer而panic。finally恢复ctool SHA256 da85a0672657eaa83a69e5d41f4d13849c2ee319b6652e54149c513e9e80e341、cache 0471c82d5b723ba073fb30c05d02c864616f10e530f79fbf1f50c598da328013。三者敏感点不同，非仅终态exit非零。

验证：修正峰oracle后发现/原cl解析/foreign toolchain miss/冷image hash/原两流cap/identity SQL文字共6/6（run92194e4e-10be-4b25-b24a-5daecf69bd5c，编译6.06s/运行1.613s）；扩fresh两流typed控制1/1（run292f54c0-46ad-4f1b-85a1-4486c441c09f，6.18s/0.613s）。反控恢复后以上6+actual compile/原真实include2/actual source压力repair及cleanup/provider换版3/compiler deps/raw/text/file四端口，共16/16（run6dcd89f3-1ea1-4fd6-b182-78b7ebf79718，3.69s/49.700s，557未选；compiler_过滤额外选入原object_file_read case，保其真实结果）。统一cargo nextest显式ACE manifest/config，-p ace-core -p ace-abi --lib --test conformance --test source_resources具名过滤-j1，原deadline/负载不变。8as core executor/8at text底层代码与输入未变，复用其原错误/生命周期与大样本证据，不为新入口再跑整包。fmt/diff/hook通过，仅原AV3warnings；ABI lib38/source_resources29/conformance123/source29。
同scope日志SHA256：
- 8au-cache-resource-denial-negative.log f0e238795b7c93b5dfeacbcd206417177b705032bdae46789064d888188e1818
- 8au-discovery-admission-control.log 818160142b7a10b60ce55c468e9b23744e0d8804002901c175a7338624b30747
- 8au-discovery-admission-final.log 168f2996f3d398111015dba74809c4709451a061b137587972cb5a69118cff55
- 8au-discovery-admission-first.log 5130fc82d10e537fdad333eedba6f684f41fdce3bc8a803c540b3d080783858d
- 8au-discovery-cap-as-tool-negative.log 8fff7d20aafda1a9fcb7347f708b98790e9fb1a79d0cd75ec6cf4cb9737acd87
- 8au-discovery-cap-classification-control.log ddca4f46e23c51944434f2c9d2a86cc28760eb611cbdd37d9879155d049c36da
- 8au-discovery-resource-as-miss-negative.log 07bac03f852de97a5573e5f596005f4035ef217dc2104b9083fc70aa82b45233
继续：discovery raw/decoded及失败分类已接线，tool command/native探测/目录枚举/global owned identity与error metadata仍开放；store key/hash/canonical内部metadata、native shared lower/完整diagnostic/cold owning Env-HIR/catalog/ledger/A2X、scratch峰/request128/namespace128、完整certificate/daily无spawn/SDK/桥export、原6×30/P90/Q4/SD/939/冻结并集均pending。未到work pass/final review。

2026-11-06 work checkpoint 8av（T-09 shared QueryKey hash整份metadata副本；仍executing 6/13）：提交 3af5c3ae48b9b4651ebeb82c82be1663a949bba4，2文件114+/8-，树clean。原QueryKey.hash先clone每个name/value成Vec<Vec<u8>>，每对还有临时Vec，再建refs Vec和family String。统一domain_hash_iter借用原字段迭代，family以qk/与layer两个stack片段喂同Hasher；公开domain_hash也委托唯一算法。原HASH_ALGO/NUL/family/NUL/逐part LE-u64 len+bytes和排序稳定次序保持，最后只分配原64B hex结果；不是新hash或key/schema版本。最终hex的source owner/QueryKey本体/canonical与artifact_id等其他store内部metadata仍须后续预留，未冒本端口即全store计费。

永久actual QueryKey allocator控制含2MiB π字段、重复同名键的原先后次序、空name/value、NUL、emoji和分隔字符；独立旧wire Hasher oracle在测窗外构建。首旧实现hash值相等但整份大copy1次exit100；修复后>=64KiB分配0、>=64B恰1个最终hex。A04/A07空key及a=bc/ab=c独立旧oracle/part长度区分通过。实际ManagedStore发布此大key、SQLite持久key_hash等于旧oracle、按key读取payload/artifact ID，再open同store仍读同内容和ID（raw store身份/持久性检查，不声称其opaque DeclSummary测试bytes已被语义codec接纳）。实际去part长度错误由旧domain separation反例抓exit100（两例选择仅首例执行，另一未跑不计pass）；单独重放新oracle也exit100、实际值与旧oracle分离。finally恢复identity SHA256 b2179739dcfa5ab69e2a9f3281defb3bab6ed3c6ebf188c5ffcf2158661e4c77。

验证：identity4+新hash allocation5/5（runca64f304-55e2-4894-a49d-b203fa019ea6，编译1.47s/运行0.034s）。扩实际publish/reopen并反控恢复后：identity4/chunk framing1/两个store_resources/leased publication2/原C include2/toolchain miss1/实际source缓存压力repair、cleanup和两目标provider换版3，共15/15（rune3415a50-2227-45c7-98d8-397c2fdef231，16.84s/47.596s，602未选）。cargo nextest显式ACE manifest/config，-p ace-core -p ace-abi -p ace-artifacts --lib --test conformance --test source_resources --test store_resources相应具名过滤-j1；mutant只artifacts identity/新oracle，未跑大layout/GC/perf：其规则、旧hash字节已独立核，前期512 churn等不因HEAD换反复测。fmt/diff/hook通过，仅原AV3warnings；store_resources现在2例。
同scope日志SHA256：
- 8av-key-hash-before.log 3a7359c4278d7c3a9850ca25a977bb0501fb05d736bcb3de0488156d669b3dab
- 8av-key-hash-control.log a7452658a95903c3742da8cb0fc7d67c14be6c24c8cb2f5c948019373e4c8800
- 8av-key-hash-store-consumers-final.log 28ee068159710ac9931be7e4d1df73ea5e2be82e93714927b97b12dc91f92a56
- 8av-part-framing-oracle-replay-negative.log af78cc6ff98ede528aac9f1d52b33e74f8972b57a69d6c9677a897f8e41cb522
- 8av-remove-part-framing-negative.log ace3560c13e730faaeaded424cfe9efd43cbf87c3aa328ea36eb67f8755df4d9
继续：Source QueryKey/canonical/artifact identity/store内部持有与native路径元数据、tool command/resolver/global identity/error ownership、native shared lower/完整diagnostic/cold owning Env-HIR/catalog/ledger/A2X、scratch峰/request128/namespace128、完整certificate/daily无spawn/SDK/桥export、原6×30/P90/Q4/SD/939/冻结并集均pending。未到work pass/final review。

2026-11-06 work checkpoint 8aw（T-09 C generation QueryKey所有权；仍executing 6/13）：提交 c0f7b1707cbe778b87571c32324a8c9b9692ecbe，3文件176+/12-，树clean。cache_paths添加CQueryKey：固定原generation/kind/source/target/toolchain五字段已按canonical次序，checked计算5*(String,String)数组及每个name/value正文，先index reserve再造Vec/全部String，不再构造后sort。Source直接复用先前已资格化的toolchain文本作输入，克隆入key仍另收费，避免public fingerprint再格式化一份；public旧c_object_key也委托同布局，None保持原签名/字段/hash/text。key作为完整value-first/ticket-second owner在JSON编码前构造，编码预算/序列化失败时也先释放key再费用；编码完成到put之间仅无失败解构，外部ticket在PutRequest/内部借用consumer全退后才drop，并在Conflict重发现前释放，避免新增early-return让已编码payload的费用先退。QueryKey.hash/canonical和store内部其他费用仍非本key票据的范围。

永久实际helper allocator控制：合成2MiB toolchain字段加原字段、caller输入独立入账；只差1B时大字段0次分配。可用时240B数组/2MiB唯一字段两个allocation点各命中1且完整费用已在账，结果与独立旧QueryKey::new构造的五字段/顺序/hash/canonical完全相等，容量总量等于实际保留费用；共享hash consumer的64B结果分配点仍看见完整key费，最后owner drop回caller输入baseline再全退。三个错误对照exit100：晚reserve→拒前大clone1次；不把票据移入owner→返回时current index2097439而应4194979；generation字段误名aaa-generation→字段/顺序合同拒。最后一log沿脚本标签名8aw-unsorted-key-fields-negative保留，实际操作者是字段改名，未冒称执行了另一次排序mutation。finally恢复cache_paths SHA256 e7cdeeac721b66bfc955f85673fc7081f93e60a6dc25f0635f6cee87ad9e304b。

验证：新key+原真实include2共3/3（run68a7f67b-e513-4338-8a03-8a84d68fd98b，编译6.15s/运行5.163s）。将key owner移到编码前后，最终原cache正常职责族24（锚定regex，明确不选performance512）+actual source压力repair/early cleanup/双目标provider换版3+新key/gen precharge/Conflict释放重发现/SQL IDs4，共31/31（runbca81838-e224-4271-8261-80b41bb9e022，6.07s/61.596s，587未选）。cargo nextest显式ACE manifest/config，-p ace-core -p ace-abi -p ace-artifacts --lib --test conformance --test source_resources --test store_resources相应过滤-j1，原deadline/规模保持。8av shared hash与8am512 churn等身份规则/数据字节/GC路径未变，复用其证据，不因新key预算重跑性能或全layout。fmt/diff/hook通过，仅原AV3warning；ABI lib38/source_resources30/conformance123/source29。
同scope日志SHA256：
- 8aw-c-generation-key-final.log 823ac6fd97f71cf14a0962eafc649543f78e849dbb99204c694272bc64a5cd36
- 8aw-c-generation-key-first.log 37529d919bfd14ce1b094b2d2ca699ed0340afb3787d81bc10d388721aca8572
- 8aw-early-key-release-negative.log d2c41b211617f8cc0103acf85057867153cb58e02fe2d548f9f9b98f754c1960
- 8aw-late-key-admission-negative.log 49f39111772876579e00f7afd9a2bc3d93d960ce3e6e7d4e0d9cbe0473a120e0
- 8aw-unsorted-key-fields-negative.log 6fe6764a42d7732f12a42860c9213ef190943ffca1b7efae492ededa88f444f1
继续：C generation key本体已补；canonical/artifact_id/SQL/文件锁路径等store内部metadata、tool command/resolver/global identity/error ownership、native shared lower/完整diagnostic/cold owning Env-HIR/catalog/ledger/A2X、scratch峰/request128/namespace128、完整certificate/daily无spawn/SDK/桥export、原6×30/P90/Q4/SD/939/冻结并集均pending。未到work pass/final review。

2026-11-06 work checkpoint 8ax（T-09 canonical key text隐式大副本；仍executing 6/13）：提交 ccecb01474181a0d5dffb6f8f36b9ad343da4274，2文件59+/2-，树clean。QueryKey.canonical原逐字段format!再append并增长，改checked canonical_len精确计UTF8/层名/括号/每个等号与逗号，一次预尺寸最终String直接push borrowed fields；不拷贝整份name/value临时文本。canonical_len公开供实际host在分配前准入，下轮将接source store对应owner；本轮仅消除隐式扩容/格式副本，不冒最终canonical buffer已计入source账户。原组件顺序/不转义规则不变：canonical保持存储次序，只有QueryKey::new负责排序，不在renderer新增sort。

永久实际方法控制含2MiB π值、直接构造的unsorted组件、同名多值、空name/value、NUL/emoji/分隔字符，独立旧format循环在测窗外作wire oracle。首旧实现文本相等但>=64KiB大分配3次（应1）exit100；修复最终仅1次、capacity=len、canonical_len精确等于旧oracle；空key仍A04[]/5B。把长度少算一个括号的实际错误虽内容仍相等，却触发Vec/String再增长，大分配2次exit100，finally恢复identity SHA256 4d62b0ff2e3268a43b62969f7fe8f964b677313bbc90311eac0d228b5d54decb。共享hash算法未再修改，其旧字节独立oracle/分帧错误证据复用8av。

验证：新canonical+identity4共5/5（run212997fa-108a-412f-86d2-0f79091597fc，编译1.95s/运行0.061s）；反控恢复后原C cache正常族24+实际source预算repair/early cleanup/双目标provider换版3+Conflict恢复1+identity4+store_resources3，共35/35（runa0754330-dd39-4c3e-9349-91b78ef90bb2，16.73s/62.288s，584未选）。cargo nextest显式ACE manifest/config，-p ace-core -p ace-abi -p ace-artifacts --lib --test conformance --test source_resources --test store_resources具名过滤-j1；原deadline/输入规模不变，GC/layout/perf条件不变且未因本格式实现重复跑。fmt/diff/hook通过，仅原AV3warnings；store_resources3例。
同scope日志SHA256：
- 8ax-canonical-before.log fac56e6179dbd48b77713b86b42625b705d275acbeb10fd437809ce036bc0166
- 8ax-canonical-control.log cc1975bc52a6481f60dd364dfe4d651c2f1cb29fda5c3d120e9066a188594c2b
- 8ax-canonical-size-short-negative.log d1741206d47b0165a1b0097a5d57bf28a46acc8817b87aba6b495796bffe5f5d
- 8ax-canonical-store-consumers-final.log 310e93ec37d5774facbfb357135ee0ebc3b79970f69dfe0dce7d3066970b4ba4
继续：canonical最终buffer在source put的预留/Conflict错误owner与释放仍需接线；artifact_id/SQL/file路径内部metadata、tool command/resolver/global identity/error ownership、native shared lower/完整diagnostic/cold owning Env-HIR/catalog/ledger/A2X、scratch峰/request128/namespace128、完整certificate/daily无spawn/SDK/桥export、原6×30/P90/Q4/SD/939/冻结并集均pending。未到work pass/final review。

2026-11-06 work checkpoint 8ay（T-09 source store canonical Rust String准入/错误owner；仍executing 6/13）：提交 30d8683e0e8e7d8ed3bc0df016960bfe6e3a50cc，2文件199+/1-，树clean。Source C cache用已拥有QueryKey.canonical_len在JSON/put之前另reserve精确canonical正文费；该费与key各字段费并存。shared store只在两个互斥分支造此String：SQL参数临时值，或Conflict.key错误所有者。成功/普通失败在store临时文本已退后释放；Conflict显式drop(error)再drop票据，之后才重发现，避免仅忽略scrutinee字段仍把它们留到arm末尾。票据不冒称覆盖SQLite内部binding/C allocator、其他SQL rows/paths/artifact_id/error字段，后者仍待核。

新增永久真实cache路径控制，未加product hook：test-only allocator在producer返回后、固定240B key数组实际分配点（整key费已预留）向同账户加入具名压力，分别只余canonical_size-1与恰canonical_size。thread-local guard在正常/断言失败均退压力，不改默认8MiB index/48MiB total或生产代码；若误命中更早数组，key预留会先失败，实际要求canonical错误前缀以区分阶段。真实独立cl provider fixture/真实produce_c_object_with_resources/store调用，canonical实际516B。修复前控制触发accepted=true、text_allocations=1、published=1，exit100；修复后同压力typed Resource/canonical key，实际文本分配0、SQLite artifacts0、CAS blocks0。释压后同账户修复时仅留精确516B：store真正分配516B的allocator回调必须看到index已满8MiB，唯一文本1次、produce1、内容相等；压力退后再次缓存命中、producer不可达，证明可恢复而非一律拒绝。

两个实际错误对照exit100：put前提前drop canonical票据→拒绝场景仍正确，但合法精确窗口实际文本分配点费用不足；把准入挪到put后→最终虽Resource，却已text_allocations=1/published=1，被实际未发布/分配断言抓。finally恢复cache SHA256 5ab0d0bd0d40fc8dd0cf18701ba75dbaeae8013ad2c4e57e960b682dc5c3047c。错误原型均为真实代码变更，非改expected/counter凑红。

验证：接线后新canonical/原Conflict恢复/key owner3/3（runc22a1e43-80e4-4afe-8466-d24ebcefb0af，编译6.04s/运行0.847s）；精确可用窗口控制1/1（runba2a6f47-d557-4f5e-ab61-ffacfb70da12，2.53s/0.261s，保success metrics）。反控恢复后受影响c_cache_ metadata全13+key owner1+actual source预算repair/early cleanup/双目标provider换版3，共17/17（run2045d916-e442-432d-bd49-ba1bece4a565，3.52s/48.480s，603未选，success-output immediate保分配数字）。cargo nextest显式ACE manifest/config，-p ace-core -p ace-abi -p ace-artifacts --lib --test conformance --test source_resources --test store_resources具名过滤-j1。8ax原35项的legacy None/canonical wire/hash/store结果无相关输入/实现变化，复用其结果；本轮不为Source Some新增预留重复全legacy/GC/layout/perf。fmt/diff/hook通过，仅原AV3warning；ABI lib38/source_resources31/conformance123/source29。
同scope日志SHA256：
- 8ay-canonical-admission-before-measured.log 43296bc43ba969810e7b9d092b9d3c13d4e165eb2611eb403c8f92e62f2927ca
- 8ay-canonical-admission-before.log 94f6fed44a22862de50c60684a280c6e507d97722e95d81485778fd297884ac9
- 8ay-canonical-admission-control.log fd5b20d831defbfae130674e9680e633ab091a0098033c6c573436d62fb69c42
- 8ay-canonical-exact-boundary-control.log fa81b36b98a15601df406337ae5dba4a368bcce02b7274be6b67752e5e037fe6
- 8ay-early-canonical-release-negative.log b82b5fb9e33264f1e7b7a3093f89ad59093970133cf72884b38a293477a1d9c0
- 8ay-post-publication-admission-negative.log cc558ca0bec709078f0a507cf93e3976e4591d41896022443f1d056fb4337667
- 8ay-source-canonical-owner-final.log 8cb1e77934ce37dab3052e1831842a2c6dc3efce2561f9d60db98d514a4dc4ec
继续：canonical Rust最终String的Source生命周期已接，artifact identity/SQL binding与row/C内部metadata/file路径、tool command/resolver/global identity/error ownership、native shared lower/完整diagnostic/cold owning Env-HIR/catalog/ledger/A2X、scratch峰/request128/namespace128、完整certificate/daily无spawn/SDK/桥export、原6×30/P90/Q4/SD/939/冻结并集均pending。未到work pass/final review。

2026-11-06 work checkpoint 8az（T-09 shared artifact identity的边/分段副本；仍executing 6/13）：提交 8e0c6d4da6020d54f45e51c9b138ab7adfbafc27，3文件225+/28-，树clean。原ManagedStore.artifact_id克隆4个头字段、每条边format String、parts Vec<Vec<u8>>及refs Vec，并有stable-sort临时区。现在唯一private artifact_identity借原edges作Vec<&Edge>索引，用原完整id:kind字节流顺序in-place unstable sort（相等字节即相同hash part，保所有重复次数）；逐边作为同一逻辑part流式喂id/冒号/kind，免每边正文/parts/ref的第二套表示。统一private DomainHasher保原HASH_ALGO/NUL/family/NUL与逐逻辑part LE-u64长度，允许同part多个借用段；公开domain_hash与QueryKey hash仍委托它。结果直接写art+64hex的一份67B String，无中间64B digest String。原ID family/artifact/key/schema不升版、不改ChunkKind Debug拼写。

新lib-test-only系统allocator转发计数器不进入产品依赖/配置；永久实际private方法控制含2MiB key与8192条proper ArtifactId边（required/provenance、反序），独立旧Vec/format/sort算法及手写旧hash wire loop在测窗外作oracle。旧实现结果相同但>=64KiB数组分配6次exit100；新仅1次65536B引用索引，>=64B总3次（引用索引、key hash64、最终ID67），空边仅后两项2次。另前缀edge/edge!、冒号/Unicode/NUL/空id/同边重复与反序（hash结构压力，非声称这些任意opaque id均已通过required-edge liveness准入），与旧concat-string-sort oracle一致。Tuple字段顺序错误只在prefix反例才分离；拆边成三个独立hash part错误在原8192案例即分离，均exit100，finally恢复artifact_identity SHA256 36c52da0c906898b9a08d34f8ba11fbfb6bca96d7a0263de5eee1c0715a178e8。

验证：新identity+原identity4/store_resources3共8/8（run5f06df1e-ed39-42f0-8226-ee63f0be7d61，编译1.72s/运行0.203s）。反控恢复后原C cache正常族24、actual source预算repair/early cleanup/两目标provider换版3、source canonical精确预算/Conflict2、artifacts codec6/identity4/new1/store_resources3/真实leased publication2，共45/45（run059b2923-0049-4b6a-bc8d-3d8db8983c24，16.22s/72.112s，576未选）。cargo nextest显式ACE manifest/config，-p ace-core -p ace-abi -p ace-artifacts --lib --test conformance --test source_resources --test store_resources具名过滤-j1；未重复unchanged GC512、全layout、perf，原规模/条件未改。fmt/diff/hook通过，仅原AV3warning；artifacts lib19/source_resources31/store_resources3。
同scope日志SHA256：
- 8az-artifact-identity-before.log 6ba6df7e09d2cde664f505f91f2083639d716691f0f6a71987e8132be5496309
- 8az-artifact-identity-consumers-final.log fda9c249ee2b0d9fcaed086ad72768776f154793ca2502a5166d9d99ef6065c5
- 8az-artifact-identity-control.log e7e4b48c51f0728475e63b41b12d0b6452072a1583d8180133cdef1fef8d177c
- 8az-split-logical-edge-negative.log 36878cfef2e4f7cfb696925ea1015406004be6c1e3b65ed0a5e73feefea98c95
- 8az-tuple-edge-order-negative.log fd1772eafce975c52a00dea8fb0113a3b7daa42dcf31af0d579f5fc57ba6a930
继续：仅去除identity冗余表示；最终hash/id/kind小正文、非空边引用索引的准入及SQL/row/file-path内部metadata仍开放。tool command/resolver/global identity/error ownership、native shared lower/完整diagnostic/cold owning Env-HIR/catalog/ledger/A2X、scratch峰/request128/namespace128、完整certificate/daily无spawn/SDK/桥export、原6×30/P90/Q4/SD/939/冻结并集均pending。下一轮可切native实际lower的FnLower Vec/slots/blocks/import映射（现lower_function_unit_impl仍无resources，new_block/emit/temp infallible），或继续store owner；不因micro端口通过把T09打勾。未到work pass/final review。
2026-11-06 work checkpoint 8ba（T-07/T-09仍open，work非完成）：提交 `1668db267531b41a8bd7e20531d5158e4d46b73a`。实际共享FnLower新增可选emission reservation端口，旧调用不启用时保原路径；native source consumer用同一Resources/MIR类持有ticket。指令Vec与来源span Vec增长前计入旧/新数组并存，分配成功后退回旧数组费用；103个emit点与结束view/索引slot错误传播，Resource分类同步原AV/native消费者，非第二lowerer。实际MIR在encode完成后先drop再释放ticket；encoded pages另有原预算。该切片只覆盖instruction/span数组，不宣称slots/blocks/types/imports/args/consts/maps或cold HIR完成。

永久实际分配反例：40次可变赋值真实source触发64条指令数组2560B，旧32指令+32span仍存时临时4352B，余4351B拒且该数组alloc=0；接受时allocator在真实分配点看到4352B费用，最终两个数组保7168B，ABC与旧入口逐字节一致且MIR深验通过。第二来源span增长差1B同样alloc=0。真实Session native源码编辑压到Mir余1B时mir-lower attempt+1/produced0、native-emit attempt0、旧source/Arc不变，释放压力后实际PE exit43并发布current.json。

验证：2新case run9ac67173-e86a-4699-8173-62b29e3bbf29，2/2，编译2.65s/运行2.307s；三种有区别负控（先分配后预留、漏算旧数组、实际source绕费用）均exit100/原字节恢复，非compile错误。`cargo t ace-lowering --out .../8ba-lowering-group` exit0，selected5/executed5/条件排除0，discovery exit0=8.398s、execution exit0=.711s、child validation valid无required children，整命令约9.82s。最终必要并集run e5949350-8ff0-4bfc-bbc2-c990653531e9，8/8，1685 not-selected，编译9.48s/运行73.093s：2新资源控制、原实际双端status/bool/ctx/源序/混合槽矩阵、3真实worker provider重绑定/预算修复/早错误残留恢复、原native first-path与data-owned差分；success输出无tools unavailable/infra-blocked，两个条件测试实际执行。first-path代码中old HIR-native腿已被871退役，其注释说明用历史影子证据，不能据case名称声称本次运行3个活动实现。native conformance额外已有unused close Result警告与既有AV3条保在案，本切片无新警告；目标rustfmt与diff/staged check通过。

保留首红：新增错误enum初次check遗漏native穷尽匹配exit101，已补正确Resource映射；4指令160B/8指令320B观察与其他block/type Vec同layout导致测试假阳性，后将真实source扩到40次/2560B独特数组，未缩规模；固定回调试验仍撞layout已移除，最终为真实Resources。初版fixture用let写入触发verify.immutable-write，改var符合语言，不改checker。原红日志均保留，不充PASS。原性能完整样本、全证书/SDK与T-07..11仍open，未交final review。

证据均统一scope，merge摘要后全删。日志SHA256：8ba-emission-check-first.log=66ac8ebdf0023da39b1f3cd67c8baf44d6b96d7191df4ab5850e1ca843cefa9b; 8ba-emission-allocation-first.log=760b404e8a593332f6d732094078ce54df2464400fc59065651e88339463866c; 8ba-emission-allocation-control.log=7f7d0453f7bc9b68cdb64faa70ea0ae222cbbd1f884893e8477b3b569fa55b74; 8ba-emission-fixed-rejection-control.log=cde775bbc8ecb46cc26e21a01f02f48b15e75eadcceb5a89f3d4c91826d4d492; 8ba-emission-isolated-layout-control.log=9969a18bd484889e7cb311af6ed9fa56117d2d80b854f921f6ae0be7cfa3a3b7; 8ba-emission-span-pipeline-control.log=4ca236b306f06b7b2a50d59548f3041d521e021f3ffbb7942cdf47dfcf38596b; 8ba-late-emission-admission-negative.log=1c37de2528c0a43edb2b60952e8c43145338a0ae9e1af4d4ab86b12d7c581c5a; 8ba-omit-old-emission-buffer-negative.log=b54517c64a391360a569dd47d0da0ba989bb5df22140d65a26376e7771864bda; 8ba-source-bypasses-emission-fee-negative.log=84f0eac08950beede5ffb7e076721c791b8962914efc531c9a15b639c16b684f; 8ba-lowering-fixed-group.log=76fd095e8f67529d10c3e8a713ea1393882883dc0e0ca0038748de2b5b458a5a; 8ba-native-emission-consumers-final.log=adf3a692a92b0002f63474fc4e9f1dcaf8cbc3479da46fd63c71b8dea945b8b5; 8ba-lowering-group/results.json=54ffc3fa0772fa7da63f9ae43b992c5a5a126edf8d368d6eadddc1e34a526e08

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: 1668db267531b41a8bd7e20531d5158e4d46b73a | task_ids: T-07/T-09 partial | evidence: checkpoint8ba | blockers: none | next_responsible_role: work remaining native lowering ownership and full contract

2026-11-06 work checkpoint 8bb（T-07/T-09仍open，work非完成）：提交 `08dbfea90a26fe82cf1a71bcf4a62d67550c8955`。沿原shared FnLower可选端口把base/local映射/type缓存、局部name clone、loop目标、blocks、temp slots、pending joins和实际任务stack扩容纳入同一MIR费用；84个slot/block调用传播Result，7个任务push走原唯一LIFO算法的预留包装。最终slot append先预留base+extra+新最终数组并存，之后drop真正临时数组/indices/stack再退费；None旧消费者仍用原容量/语义，非第二lowerer。module types/imports/consts/map、call/declaration/args及其他profile workspace仍open，不冒完整native预算。

新增真实分配控制：初始4-block数组160B、初始task48B、if/else真实多pending任务增长192B（旧栈48B仍存）、250locals基表8000B、40次赋值最终82slot数组2624B均在差1B时Resource拒，目标数组alloc=0，caller错误ticket在partial对象先drop后释放。最终slot复制实际allocator同时看见14137B；成功返回只剩真实MIR数组/slot name合计9953B，临时槽与遍历索引费用为0；ABC与原入口逐字节一致且validate_mir通过。前8ba instr/span原控制依然保留，加入真实并存scaffold后2560B指令数组准入5625B（余5624拒），未减40赋值/原源序负载。

永久控制1/1 run85dace76-0324-4dc9-a7b2-a755431f2f77，补真实if/else栈增长后run d2e0ac46-34fb-478c-948c-f3007384d093 1/1。四个区别负控分别去掉blocks、stack growth、base slots、final slot copy admission，全exit100且按真正错误断言拒（前三个目标alloc=1，最后实际候选错误接受），每次finally恢复bytes；初次负控脚本在前两项完成后第三项rustfmt锚不匹配而停，源已恢复，后两项修正锚另脚本执行，不充额外测试结果。初次编译遗漏match中join_block的?（101）已修；原8ba控制首次因新增并存费用期望旧值而100，保留并将fixture精确预算更新为全owner费用，不放宽产品帽。

最终`cargo t ace-lowering --out .../8bb-lowering-group` exit0：selected5/executed5/excluded0/not_run0，discovery0=1.559s/execution0=.729s，child valid。当前实现最终必要并集run899d7bb9-1e9d-469e-8ce4-3cb026700c3b，9/9，1685未选择，准备15.20s/运行69.052s：3新/原native资源控制、原真实native/C scalar/status/bool/ctx/order/short-circuit/mixed矩阵、3正式worker provider/资源修复/早错清理、原native first与data-owned差分；没有tools unavailable条件排除。旧AV3/native conformance close Result警告保在案，未声称全仓health零。目标fmt/diff/staged check与钩子通过。完整证书/原性能/SDK及T-07..11未完成；复审/冻结全职责并集尚未进入。

统一scope日志SHA256（merge摘要后全删）：8bb-container-check.log=4fd11ae5618de1b1f8843b16c9edfffbc932e3f0ce8cae46f4939de1bf535716; 8bb-container-original-controls.log=ba0412730ae61c6b156bc120270e3327bb8e87d4c3d70e98d12dd00593da6922; 8bb-container-emission-controls.log=ee79c5c1a67256ad5c8f1e0b1999e0c6cacc306e0a7279cd806bdc79112141cf; 8bb-container-admission-control.log=71e4d6d77593a50f08d7fac776c6dd319914c30b48d13e1d65d44b0842858396; 8bb-container-stack-growth-control.log=84200866c9b08c756aaacacec6bfc1751dfb8f9e342ce18b665ead5c0b45e08e; 8bb-blocks-unreserved-negative.log=0f28fdc8d8ae16b378c824685add71259b7e5271377d32e5cfad81caedf5d3ce; 8bb-stack-growth-unreserved-negative.log=56a963261bc14af8b80ff2bedda7161773cf9a843e2e40df9ef8f55686fa7f52; 8bb-base-slots-unreserved-negative.log=427f3d010a43350b423779086d6084476800ac08fc1a2100aa40af850aff13c2; 8bb-final-slot-copy-unreserved-negative.log=2dde90b5c8be7838edd499a72145f95084f11274498aa4dfff937dc298b5c9db; 8bb-lowering-fixed-group.log=e349c464dc394158e45fdf857a9389fac6488a44c123dca82ab6c9310df218d8; 8bb-lowering-group/results.json=2b019bc5df085c61fb7ef568dd0c04e5954befa793eb1275921378dc83956653; 8bb-native-container-consumers-final.log=9bda1c3ecd07f58f5ade7a1389c58581071cd6b3df9a189eb0f2e47a0f5382bc

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: 08dbfea90a26fe82cf1a71bcf4a62d67550c8955 | task_ids: T-07/T-09 partial | blockers: none | next_responsible_role: work remaining native module/call metadata and complete contract

2026-11-06 work checkpoint 8bc（T-07/T-09仍open，work非完成）：提交 `f7a9bef958f0ab79c5a4c3591aa74d38c4881fb8`。沿shared FnLower原算法给i32/bool constant pool去重后增长预留（老/新数组并存），4个常量调用传播Resource。Call源序eval_args先预留实际临时slot Vec，绑定重排后的actuals独立预留并随Call/ForeignCall指令持有；ForeignCall的SourceDecl Box+整个nested string/表clone bound先预留再clone/box。Call原有各返回分支收拢到同一结果闭包，先drop eval_args Vec再退其费用，typed失败也走此路径；不改变源序/短路/错误lane/参数绑定。SourceDecl.heap_bytes按原capacity保守覆盖clone，是该owner的保守额度而非声称每个spare capacity都被复制。module types/imports/def-map/owner/caps/name及普通call冗余签名clone仍待后续，不冒完整native预算。

永久分配控制：常量元素24B，初始96B数组临时费用432B，64项常量数组1536B增长临时8144B（原32项768B与其他slots/instr/span/block/stack同时存活）；差1B均目标alloc0，接受真实分配点有全费，ABC与原入口一致。真实受检foreign_add source以宿主当前审定声明进入Frontend，Box416B+nested clone bound464B=880B，连两个eval参数/已发射数组临时1544B；余1543拒且Box alloc0。失败后eval参数8B已drop并退费；成功后实际指令参数表保8B，临时eval表与遍历/中间槽已释放，剩余费用1464B，与returned MIR各owner及保守声明额度逐项相符。该新分配控制不编译physical provider、不冒实际C调用；真实PE双端矩阵在最终消费链另跑。

四个负控各exit100且恢复原bytes：去掉constant pool admission→96B提前alloc1；whole声明clone/Box后才预留→Box提前alloc1；漏指令参数fee→1464期望实际1456；提前释放eval参数费→余1543错误允许Box alloc1。最后一项实际在拒绝控制抓到，不谎称由接受分支minimum捕获。恢复SHA d065ec58393c3bacb898ce2de351a2318463d6dda1f736041f2300a888603aa1（其后仅API文档comment更新）。新增控制首编译sum泛型推断101，两份日志均保同因；显式sum::<usize>后已编译。第一修复运行4/5绿，foreign错误ticket期望忘记common error返回已退8B，校正该可观察owner生命周期后run5aa91c9f-9767-4a0f-8f92-f2c67748471e 1/1；非产品帽放宽。原instr/span/slots控制同步加入96B常量池并存费用，保40次/250locals/原数组规模，最终returned subtotal10049B。

最终`cargo t ace-lowering --out .../8bc-lowering-group` exit0，selected5/executed5/excluded0，discovery0=1.907s/execution0=1.021s，child valid。实际消费者最终run65f79f1d-154b-409c-9a31-2f273d8ea2f3，11/11，1685未选择，编译15.82s/运行69.427s：5native资源控制、原实际native/C status/bool/ctx/order/short-circuit/mixed物理矩阵、3真实worker binding/资源拒绝修复/早错清理、原native first与data-owned差分；无条件tools unavailable。目标fmt/diff/staged check及钩子通过；既有AV3/native close Result warnings保在案无新增。全合同T-07..11/完整证书/全部SDK/原6×30及原native/A2X/Q4/937规模仍open；复审/冻结职责并集未开始。

统一scope日志SHA256（merge摘要后全删）：8bc-call-const-check.log=3c3e009fc2b9e387ee0a156fca0c8df3b5a919b224f66ae4a4341c821a2a6775; 8bc-original-controls.log=0a81384ecc40fbd5dd80cce55b4ab249986b1fbd545e6e3c9acc37f17047ebd8; 8bc-const-foreign-allocation-controls.log=f7291cb799c8ea52a9a2651772e72a936640404e76ecb6ea1d54fcc87510a31b; 8bc-const-foreign-controls-repaired.log=ab22475faaa737c98df4ac98a4df6adb2d49d26f711b872e677c409cdabcbb22; 8bc-call-error-release-control.log=9c42219a3e67f9a2b67f87901cb580a734d087b52adcc5275e2c503dbfb89625; 8bc-constant-pool-unreserved-negative.log=bf2cf89ed7d255308ee0f7ccf824ea0e0c4e771ae57c11016c155edee78b9b71; 8bc-late-foreign-declaration-negative.log=9d502e61ebe25ecc059c2480a700aeb0c7425f5f5f3bdf6a6b74c7612cf960e8; 8bc-instruction-arguments-unreserved-negative.log=ca19ea35f31a293f512ee73e0464a062d51ea93747e4be57bd15632b1fae5126; 8bc-early-evaluation-arguments-release-negative.log=672a3ff0cd6eb5d6c4970e102da87cd63d92af8667d6ddda82165665ef73e360; 8bc-lowering-fixed-group.log=321ce300eb7d09239c435385b8a273bb1bda2f20a3d8d88cf341e742cc61577f; 8bc-lowering-group/results.json=eae94b0b0e89b437c713425c6714d518c007c383211295d42877ddd27904b955; 8bc-native-call-const-consumers-final.log=0b5df2523641e7274d150aee4be29c7c7e723ebea69c60929b8754acae5713d7

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: f7a9bef958f0ab79c5a4c3591aa74d38c4881fb8 | task_ids: T-07/T-09 partial | blockers: none | next_responsible_role: work native module type/import/owner/index accounting and remaining contract

2026-11-06 work checkpoint 8bd（T-07/T-09仍open，work非完成）：提交 `0de33b314779b6c0ad88a6591a6a0ff222abd26e`。新增`lower_function_unit_reserved`作为同一unit实现的完整scalar metadata组合入口，实际native source producer已改用它；旧emission-only入口保原精确子费用测试，不冒实际consumer当前全覆盖。模块identity/profile/caps字符串与Vec、shared fn signature参数/类型表、import symbol/表、最终func表、临时def-map均先预留。types/imports去重先drop候选再退费；def-map保守libstd B=6 node bound在insert前计，实际drop后全退；外层完成func表时仍保护已返回function的费用。方法名改借用owner名后一次准确输出分配，不另clone owner/format临时。admitted scalar call直接借已经深验/实习化的import signature，无额外签名重建/clone；原data/owned mut-record包装事实路径保行为。无新MIR wire/schema字段或第二lowerer。

SourceDecl先用原capacity作整个Box+clone graph上界预留；clone完成后按新owner真实capacity退多余费用（真实fixture150B），不在clone前释放保护。8bc的发射子费用永久控制因此仍在Box实际分配点要求1544B，但成功retained从保守1464B精确到1314B；全unit foreign返回真实heap=fee2708B。常量唯一值已查重后直接push，避免再次调用intern_const作同一全池扫描；序/索引/wire不变。所有新owned成功MIR/indices端口在该scalar unit组合中已逐owner计，但不能据此宣称cold Env/HIR/catalog/query/ledger、错误/诊断、工具/存储或完整A2X预算完成。

新永久full-unit观察用本测试目标原System allocator（仅观察，不是生产global-allocator拒绝或预算panic）：每个实际owned allocation/realloc点检查已预留MIR费≥该窗口真实new-live峰，realloc仍含旧/新并存，库存从128扩4096且overflow不PASS；另独立遍历返回MIR所有String/Vec/函数/指令嵌套owner核精确fee，临时def-map/stack/duplicate签名候选无残费。四个scene：1,048,607B caller-owned oversized capability在fee差1B时不clone，接受raw lowering后新旧encoder均以原`abc.encode.capabilities`拒（不是合法产物PASS）；原u8 arity上界255参数/1020B签名数组差1B不分配，合法新旧ABC逐字节同且MIR深验；512 helper+main=513imports包括type去重/map split/正常call时实际heap peak107226B、retained67728B；真实checked foreign source的Box416B，actual heap peak=retained2708B。所有实际分配点已有费用，合法三类独立旧入口wire一致与MIR深验均绿。

红→绿保留：两个变换脚本分别期望6实际5个signature点、复用片段end变量令内存锚重复，均断言拦在写文件前；初次shell继续对旧码check的`8bd-unit-composition-check.log`不是wired PASS证据，修正后check-wired exit0且无新增warning，早期临时unused warnings不留最终码。首次full-unit case误把有意超长cap当可编码正例，实际nextest summary FAIL(.127s)，log1,051,588B；3个仅做尾读的本会话pwsh PIDs34788/16492/50108在核CommandLine与无测试子进程后停止，outer reader sessions exit-1，不能把该wrapper当测试timeout或环境归因。保原log，校正为新旧encoder同拒code后run850e2739-61c7-408f-998d-10a5e0cef5d6 1/1；所有后续日志读取逐行截断。随后加foreign full walk发现旧保守clone费多150B（e4223549...100，2858 vs2708），实现clone后退多余费，run6b0769f1-dd70-4032-a4ac-5622e98b7dd4 2/2（18.43s准备/.243s执行）。同机941正式native组并发曾被观察，未据此为上述两个确定oracle/fee差异甩锅。

4种区别负控各100并finally恢复：绕完整module port→错误接受metadata压力；signature Vec先分配后计→1020B alloc1；dead def-map漏退→returned1050791 vs owned1050499（差292）；外层忽略returned-function offset→真实allocation超过已计费用。恢复hash99c76c56aef825e00d1da53e2553919e946878184b33426f57840c80b700840a，其后仅常量单次查重插入优化与实际consumer ticket变量命名；最终lower文件SHA256 `f07a8e0d3a3b3ff8ab8f8c896d1ede430c8e3266c32d9291ccdbd8b0b98ac829`。

验证终态：`cargo t ace-lowering --out .../8bd-lowering-group` exit0，selected5/executed5/excluded0，discovery0=2.272s/execution0=.959s，child valid。大消费回放run6db8b785-0c64-4b77-8600-2747d12d27fc，19/19、1678未选择，18.32s准备/102.724s执行：6 native资源控制、原实际双端status/bool/ctx/order/short-circuit/mixed、3正式worker binding/预算修复/早错回收、原native first/data-owned与7个record-method/static/owned/mut-writeback兼容case；无tools unavailable条件排除。最后单次常量查重优化后只重放完整受影响资源+实际native source PE修复6项，run dd44175d-ccb0-47a3-b30f-2320c9b71af8 6/6，15.50s准备/2.098s执行，31未选择；复用前19项中未改动的metadata/方法名/布局/worker兼容证据，不把它说成最终冻结全职责门。目标fmt/diff/staged check及钩子通过，既有AV3/native close Result warnings保在案。T-07..11、完整报告/证书/SDK/所有原性能样本/Q4/旧桥/SD/939仍open；final review/冻结职责并集未进入。

统一scope日志SHA256（merge摘要后全删）：8bd-unit-type-port-check.log=528dae189cacc37b98caef867d8887312e8a4930f065d24df90202a6f187d981; 8bd-unit-composition-check.log=a203b9c38b5b38160b5cb9f8e0719a7f4355772bb256b5353448da000d17aabd; 8bd-unit-composition-check-wired.log=29013b896d988efef375f416a3e5ae2d6b20951b1e983f6efd81a00482b7d00e; 8bd-existing-emission-ports-and-actual-consumer.log=7fd1729043f24358377f0d382779b48124e5aa495bfcf4117171cd318e55695e; 8bd-unit-metadata-allocation-control.log=37641a822dee3a4252f38bb6fdc3182e0e585f8b84f00cb248687c41e31b28c5; 8bd-unit-metadata-control-bounded-oracle.log=5a352c232e240aa7f05912483ffac68d4435a4e35ed6ce423941f0627e80ed02; 8bd-unit-metadata-and-foreign-heap-control.log=704c58cdb6153636b0a34e19fea8461e5aec19baff089a7935ca49112f24f188; 8bd-unit-declaration-excess-release-control.log=1247f178b2bf0f334701cdbac8db982ce6a0556eea71c3c983d8690c3ddddb7b; 8bd-bypass-unit-metadata-negative.log=78b1a6bd1f3325ff0d12b3c3d95d180740db3a95a083408f6817c8c6006b90f4; 8bd-late-signature-array-negative.log=6790fad5637706f9622131362f5292dae3668c2e5792dc38c946f676d8c5d8de; 8bd-retain-dead-definition-map-negative.log=a404fd5c576422ff5d29b193c3d04cce1a9f407d298d3a33f2cb283631ba4ad7; 8bd-omit-returned-function-offset-negative.log=e091a102e803e3ae4e57509105e8614cbfdbb9b1dcc3a3c47ce8589d3bfbc2f0; 8bd-lowering-fixed-group.log=cc482e6ecaf290c0c0a915c7cd3ade3ff7d1d65ab1214ec30690af2589d52d71; 8bd-lowering-group/results.json=dda22ef61827118988267b43ab711b819470fa504e50c712b7afc075fd507e87; 8bd-native-unit-consumers-final.log=1d3755354d6ef23cdaa063a70220ff5345f3f10a1a565050cc93b974109a0f33; 8bd-final-single-constant-lookup-controls.log=2af9cbaef54b1dce95364fd7f538455068d9dd57a3f031712525fec6a0461703

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: 0de33b314779b6c0ad88a6591a6a0ff222abd26e | task_ids: T-07/T-09 partial | blockers: none | next_responsible_role: work audit remaining frontend/metadata ownership and complete original contract

2026-11-06 work checkpoint 8be（T-07/T-09继续，work非完成）：提交 `e119dab0111aa356ee3aaac7f00adeb587a6456a`，仅Frontend cold组合5+/10-。核实际owner边界发现ProductCompile早已有事实入口采用的canonical Program（其API文档明确retain env必须消费它），Frontend却把cold catalog每个FnItem再deep-clone成另一个Program，仅为随后resolve_env_with_supply/catalog提供借用。现两处直接借product.program，catalog分页生产仍保源顺序，未再parse/solve或更换solver。删除这次整AST及body/name复制，不造替代AST/前端；复用已授权source入口，原2次cold parse尝试数量不变，未冒P0或称冷compile只解析1次。该低影响复制删除不补镜像实现测试，用现有真实职责/恢复反例验证。

验证：SourceQuery全lib run84cbef2b-373e-464d-a5d3-1e9a081e2373，83/83，0skips，4.72s编译/2.045s运行，含shared facts/foreign reader失效、cold/warm/required恢复、失败pending/discard与observer unwind（最后panic为catch_unwind正控，不是失败）。消费者runfd9d23b7-ab2e-4376-9549-ee89dcaf0ae0，3/3，157未选择，17.61s准备/38.716s运行：原真实native/C status/bool/ctx/order/短路/mixed矩阵、独立进程required restore→无预热首错修复、native预算拒绝旧Arc保持→释放后实际43发布。最初把显示名`ace-source-query::conformance`当nextest binary导致filter parse94、没有测试执行，原log保留；改package表达式后SourceQuery没有conformance target，因此这个消费者命令真实只有3项，不能虚增预想的27；SourceQuery职责由前83个lib实际用例承担。定向fmt与diff/staged检查/钩子通过，无新增warning。

后续审计边界：resources.rs明确48MiB为source managed total，native16MIR/24staging/8IO与index8同时约束；上游syntax已有独立FrontBudget（本路径显式64MiB work-owned），本轮不把它静默改成48或重建全parser计费。仍需按真实retained owner审查新增CheckedEnv/ForeignContext、catalog/query/ledger/HIR/恢复暂存及A2X/工具/SQL/edges等managed对象，不能仅因有上游FrontBudget或已删一次复制就核销T-09。scratch_peak字段与request128/namespace128时序实际峰、完整cert/daily emit/readers/SDK/原正式样本/Q4/旧桥/SD/939均仍open；没有移交final review。

统一scope日志SHA256（merge摘要后全删）：8be-canonical-program-query-package.log=227d9f3960df1b196c7fa61aa1f701f258f419065d5f2f8f230a3394002d311f; 8be-canonical-program-consumers.log=3f641dc62f5568ef84c0ee7867251e2f3c30b7b1c487c89b5c9214bb5f20a2c5; 8be-canonical-program-consumers-correct-filter.log=d7813a50cac66c2dee6ab54232518ff33986fa7e6a581e7b2ac14248a42d4866

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: e119dab0111aa356ee3aaac7f00adeb587a6456a | task_ids: T-07/T-09 partial | blockers: none | next_responsible_role: work remaining actual managed ownership/certification/performance

2026-11-06 work checkpoint 8bf（T-07/T-09仍open，work非完成）：提交 `af26e0d47b91202fd8fdd58156e6ea705639f7f4`。新增Reservation::try_retain(&mut self)：索引登记失败时不消费/释放原ticket，成功才置空交给weak-owner账；旧consuming retain API语义保留。真正owned工厂在value前声明mutable ticket并用try接口，使Arc/Vec在错误退出先销毁，随后ticket释放。已接Resource.own_payload/load_payload、Page.encode、C cache base64 decode/文件读取、source target encode/restore共享Arc attachment，以及provider rebind两份新binding guard。rebind不把较早创建的Option guards在next值之后move成新locals，而借用Option内guard，保正确drop次序。该切片只关闭这些attachment失败窗口，不冒所有DTO构造前字段复制或其余临时/诊断ownership已完。

永久实际释放点反例：先压满原index8MiB可用额，真实own_payload的128KiB+31B Vec及Arc取得Resident fee后，原ownership-index扩展被Resource拒；在System allocator即将释放该实际Vec pointer时，必须仍持有capacity+40费用，且该pointer恰1次释放，退出后回到受压baseline。另同一value用mutable ticket在索引拒绝后保原current、Arc strong=1；释放压力后同ticket重试成功，drop ticket不提前释放，alias保费到末个owner释放，最终resident0/live_objects0。旧consuming接口负控exit100，准确由释放点费用提前归零断言抓住；finally恢复resources SHA c77b275a2e3d08edef8c7380c961d5700f01e12db84feb797f86bd8a6c2f2391。测试仅观测allocator，未注入OOM或预算panic。

验证：新增case先run c2d22717-58dd-449a-9a1e-6f793c68c1cb 1/1（18.86s准备/.012s执行），库check exit0 7.20s。迁移全部具名owned调用后最终run49487f7b-75bd-414f-bca1-7c8e68e15238，149/149，133未选择，122.102s运行；分母为SourceQuery全lib83 + source_resources全38 + 原cache_turnaround::abi_cache_固定24 + 实际双端矩阵/3正式worker共4。Source资源38含所有原分配/IO/UTF16/目录/工具发现/JSON/编码/actual native压力与本新增释放控制，真实fake-tool子进程仍由原断言核验，不以父exit0代替。原C cache24、provider-only caller P0、预算拒绝旧root/repair和早错误回收均绿；source unwind测试的捕获panic为正控。原512修订重型性能不因本次attachment-only错误路径重跑，旧有效证据边界保留；正式冻结职责/原性能仍pending。定向fmt/diff/staged check/钩子通过，无新增warning，旧AV3等已知基线在案。

统一scope日志SHA256（merge摘要后全删）：8bf-owner-attachment-refusal-lifetime-control.log=80b62c6779a751315c466fcabc5c7d4ebef0716df0c3dc05a8b03d29a3f8aa24; 8bf-owned-attachment-check.log=bb1874a3650eeaebe7e8c71ca17bf3fe1ec6ae67d3106159a60e68696770c73c; 8bf-consuming-attachment-early-release-negative.log=6da6c4b52dcdebf4844e9d27cbb1a9e2a27e6bcf7d9a2b394c256fe5167397b0; 8bf-owned-attachment-consumers-final.log=5845b603723a6419410efe97e40fbb7155cde39b123ddcda6ea36e9c564342fa

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: af26e0d47b91202fd8fdd58156e6ea705639f7f4 | task_ids: T-07/T-09 partial | blockers: none | next_responsible_role: work remaining managed metadata and full certificate/readers/original workloads

2026-11-06 work checkpoint 8bg（T-10 partial，work非完成）：提交 `37e1e584d8cda18bde250c3d66b8850834aa441c`，只改language SDK纯镜像abi_source.rs，39+/4-。此前父端只检查contract/header/physical-table digest是否64位lowerhex，可把审定内容换成无关合法hash。现本有限source-import-report/1 recipe在父侧独立pin精确ADD contract `cef4c3885f1686a00c0bb8c44ca6b95559a634fff297f8193becca2d1d35d71c`、header `4051cd6af2acd0c07eeed8495f50ad51fdfac94c3b91928f86422e486b408b98`、物理table `0ea8be97d1026bd57dcaff7b35673e45a6a9f17e2f5c991bdd1b1fb04a94daa3`；值来自既有独立review材料与registry测试校验的fixture，非从report自授信。不固定provider实现digest，保合法body0/1与当前C工具身份变动；也未升新公共ABI或扩大当前只add的正式recipe。拒绝/未解source的Resource报告仍必须匹配当前可信policy材料，候选manifest不是这些trusted字段的来源。仓内协议文字producer/typed reader搜索仅worker_source/abi_source两处，SDK仍需最终真实packaged consumers验证，不能据搜索就称全部SDK已关。

永久新反例先红：run8770d524-dd68-4d7e-a893-7e83bdf8da02，exit100，1/1 FAIL（44.53s准备/.039s执行），正确格式但无关contract hash被接受；保原log。修复后source-report完整4个case run3d6cbc75-ca7b-4d73-ba6e-2d829e37cd28，4/4、1648未选择，31.99s准备/.067s执行，新case的3个轴分别换contract/header/table为另一合法64hex并全部拒；合法trusted-control先通过。旧跨recipe/consumer/ABI/compiler/漏call/重放source、diagnostic span、未解source Resource不能冒执行证据等全保。Root libtest既有145warnings在案，无新增warning；定向fmt/diff/staged check与钩子通过。

实际正式注册`cargo t case-ace-ca-d-source-938 --out .../8bg-formal-source-review-pins` exit0/gate_passed=true，selected/executed11：10 PASS+原预登记wrong-oracle FAIL，0新增豁免；底层auto-case原exit1，不能谎称全子项exit0。wrong-oracle actual42/reference42各自与父expected43产生两个独立mismatch，正常native/C add42/provider-body43、无cap/unreviewed/wrong-type拒绝等保原规模。实际run27.321s，整命令109.204s，runtime_cleanup_complete=true，raw runtime和artifact-copy都在统一scope。原公开SourceRecipe/Report字段未加删，新建fake PASS或旧ABI报告依然不能借用。原warm native第二query本次完整parent-request=2876ms（source-import=2817ms），仅此原首例两个query，不是30样本/P90；说明T-11适用warm≤2000ms工作仍待优化/正式采样，不能据注册gate绿替代性能合同。scratch_peak仍有literal0、完整P/M/R/tool/IO及完整多fixture certificate仍open。

统一scope日志SHA256（merge摘要后全删）：8bg-parent-unrelated-review-digest-before.log=74ccd85aa242374540fbd1c1482936c7982e6c8cc52a4202e71b52ffc7cdb03b; 8bg-parent-exact-review-content-controls.log=872662ccdbc0d67142be69197ada1e9a78006dba544c1ecfaea5f4f14294556c; 8bg-formal-source-review-pins.log=1a6b7dc5221dea020dcbda6401c77d1d5e8813733baacdc1b6052c9aaf380567; 8bg-formal-source-review-pins/results.json=537b38c61a6a734fefa3eb536e97dcdb83746b4d88ef2acb3cc132e7c059cf31

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: 37e1e584d8cda18bde250c3d66b8850834aa441c | task_ids: T-10 partial | blockers: none | next_responsible_role: work full certificate/readers/instrumentation and remaining T07..11

2026-11-06 work checkpoint 8bh（T-10正式有限矩阵与T-07/08契约切换；work仍非完成）：提交 `e339a843a801392ea809736f2d074b10fd9f5419`，20文件2640+/56-。先核现有父路径已把完整typed SourceReport连nonce/request_hash/实际worker exe SHA保存，fixed runner artifact-copy清单含全部报告文件逐个SHA256，故不重复增加issuer/build字段。新有限Review::Scalar与原Add并列：独立人工声明7函数（bool、ctx/status/out、tick/trace/counter、5混合槽）精确pin contract BLAKE3 `aaa3a8b32d0723d77c2c12202d86445ca8626a9368374fd787517801601ebd7e`；header/table仍原独立审定物。10份新ordinary-source deck/20个正式native/C case，期望仅父侧。手写C reference按有限场景编译，显式局部变量保Auto参数源序，不借C未指定实参求值顺序；不是由Auto/HIR/MIR生成reference。provider原7函数实际call hook全部记事件，parent根据提交deck.reference独立核当前审定contract、完整callee序列及zero=-1/overflow=-2原始状态；actual/ref还各对独立parent exit oracle，不以双方一致自授PASS。原add/bias/denied/unreviewed/type/wrong-oracle11项不删。

实际worker可先R原review/binding，再经既有foreign/source事务切换至另一独立review，代际递增；不是把新decl伪装body-only rebind。原add pointer序列化字节形态保留，缺contract_digest仅映射原精确Add policy，required root仍独立深验；Scalar新指针显式含contract_digest，去掉该字段不能变成合法add root。新进程链两端：add42→Scalar首次type错拒且旧指针不变→修复mix30211→真实whole-entry warm R零foreign producer/零编译器生产→删新字段拒且不写指针→修复指针后再恢复30211→切回add42。观察包住worker_source.execute全部入口，包括前置qualification/provider与required restore，无先预热后reset。故本证据关闭此有限registry的R隐形规范化/编码疑点，不称所有T-07计量已全。

验证与负控：source_provider新policy case验证合法全manifest、原Add互拒、cap拒及ctx/类型/error-model/source名4轴篡改；10新file manifest都与独立registry逐字段相等。初次待填pin控制真实100并打印上述实际digest。原first正式运行31项=26PASS+1既有expectedFAIL+4ERROR，4个short-circuit reference因原-WX下C4127常量条件拒编译；修为volatile本地选择值，不降warning或优化flags。新pointer测试初次手工重写旧JSON改变已注册pointer hash，正确被store publication owner拒（run 1FAIL/5.714s），不是产品回归；改为实际writer对Add保原字节形态，合法reader与Scalar去字段拒分别实测。

3个有区分力变异均exit100并finally原字节/hash恢复：父漏status oracle→axis0误接受被抓；只信actual/ref事件相等而漏独立序列→axis4漏调用被抓；在实际worker warm前偷偷调用review.manifest().contract_digest()但原compute仍0→观察到foreign-logical-encode attempt/produced而被抓（11.664s）。父恢复SHA256 `5abfc9ab87bb88dbd80b10081e46685b60e8c8ed1053ac2e4e8ab31cca11ef91`，worker恢复 `2c3dd7c843adcd23141579e826fce2621379d2a98f8693b41efb06dcd18c932d`。最终source-query全lib83 + ABI policy2 + 实际双端矩阵/原binding P0/原请求metadata压力/原restart首错修复/新review转换5 =90/90（run f4c4afc5-f12d-495f-acb9-1483b7d60d39；6.74s准备/110.107s执行，156未选）。新转换单项34.944s，原真实矩阵25.558s。父source-report全5/5（run dd63f5cf-56f3-471c-ab53-286413a7a642；31.15s准备/.073s执行，1648未选）。一并修8be留下的production-only unused Item import：移至原wire_tests，仅测试代码使用；最终仅原AV3基线warning。

最终注册 `cargo t case-ace-ca-d-source-938 --out .../8bh-final-formal-scalar-certificate` gate_passed=true、executed31（30PASS+原预登记wrong-oracle FAIL，底层auto-case exit1），实际执行93.617s/整命令153.650s；runtime_cleanup_complete=true。没有扩大expected-FAIL豁免，没有条件跳过新增case。提交前staged diff check发现新case文件EOF多空行，去除1B并amend同一提交；随后原inventory实际重读31项：suite/source/oracle/expected/recipe/profile/selection等全同，仅20项case_file_sha256随空行变化。复用本轮相同源/期望/工具的运行证据，不能冒冻结raw file gate已跑；最终review/merge固定职责并集仍pending。fmt、最终diff/staged check、hook通过，无活动进程。

开放边界：该轮正式policy已覆盖完整7函数的10种场景，不再重复旧“正式只有add”缺口；完整report P/M/R/attempt/reuse/discard/IO及shared tools、daily纯emit实际spawn0、所有SDK/旧797与937/Q4回归、原6×30/P90、managed frontend/cold/ledger/A2X/其余metadata预算及scratch/request128/store128仍须核清。T-07..11仍open、status executing6/13，未移交final review。

统一scope日志SHA256（merge摘要后全删）：8bh-final-formal-scalar-certificate.log=b3f42cbb977f3d1cc8fd5fba1c36a862a22bed5dee9882dfccf8d38a9f029b75; 8bh-final-parent-scalar-controls.log=39f5ebd4c4f9fc4c54652ae042fecd3b4d3d60a8c9a8fa3db72749a60e922710; 8bh-final-source-policy-worker.log=34df324f8e7bd868998b231be2075ca7e317a117f6dab57b882e27be3c118448; 8bh-formal-scalar-certificate.log=b718301ef4d8f18fe9692a89810f3bd1a83a74e426e103030b7e585558a0840b; 8bh-negative-agreement-only-events.log=26bb71c6ffae05be226801b08b4d58bfe42023a1e6cdf7e8988f2893d4e16b68; 8bh-negative-hidden-interface-production.log=bddbe40bd73432e6ab083c993586f4c2c11d607e9879a318f875edbdb232c29f; 8bh-negative-missing-status-oracle.log=997f10aeccc692b8ed4fb348c9d692862f4f6f97ccf41427242cad7a35199481; 8bh-parent-scalar-controls.log=800307c37c07fbea6f34286a884d6ca8cf96bcc8d3d3e498f87509c6a83b58bf; 8bh-scalar-policy-pin-before.log=3b6d0c070e1cd555662047aebdd3ff7ffab103bcda905c1aaa00dae6b3395791; 8bh-scalar-policy-tests.log=af49b65dedc94ddde0eef369a507ec43e855b10024a1ce6926c3734fee96b07d; 8bh-scalar-worker-transition-after.log=d483d5e8ea5e2dd1288285ed692a8d5e50e3bb893f2b80f213f4b6dd291f129f; 8bh-scalar-worker-transition.log=c3264bfa59eaa88a66a91e1d8ba8f3684aba75e811c6fd3f7db81dbb9b04ffee; 8bh-formal-scalar-certificate/results.json=7df344f9ea2f1bb565c14579518cac23f4355926e62fffee1dcf7002e65764a0; 8bh-final-formal-scalar-certificate/results.json=7006841fceeed0e7fdab962cefcc8c39dee21c9abc39a86d627692b20a15de9d; 8bh-parent-negative-results.json=4f1fc34ffac3cd77dd54e25c0388e967febe98932c8cf84a143ffc4111b245a3; 8bh-worker-negative-result.json=4f95f7d3fc744e774d252713dede7df5ca1bf704bef885dedbc25cdf359cd9a7; 8bh-final-inventory-eof-trim.json=528245b424401972007008322f76af2f7763d21eeee96893e3a49e30e0e21c79

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: e339a843a801392ea809736f2d074b10fd9f5419 | task_ids: T-07/T-08/T-10 partial | blockers: none | next_responsible_role: work daily pure emit no-spawn, actual work/resource report, remaining managed budgets and original workloads

2026-11-06 work checkpoint 8bi（T-10 daily纯emit/metadata真实进程观察；仍executing6/13）：提交 `7e393d07f64cb1dc8888f6f1860b5d6e3af437f9`，仅既有conformance注册文件source_foreign.rs新增160行。先核Session::cold→同一Frontend/HIR/shared-MIR native或HIR C fragment→Session::prepare/commit→snapshot已有明确纯入口，未为测试新建另一发射器/新recipe/SDK model。永久case把实际独立C fixture编译留在显式外部准备，给fresh日常子进程已有provider对象+独立宿主BLAKE3授权及受审manifest；**整个外层test确实会启动C工具，不能称整个test spawn0**。日常测量窗口仅含既有输入读取、独立qualification、两端真实cold emission、两次元数据snapshot保同Arc、main body编辑真实产物/digest变化；不link/run，不声称验证运行结果（业务语义复用8bh实际双端矩阵）。

复用仓内scripts/test-governance/native_meter.py suspended-start Job累计进程数，比同一Rust harness空入口与真实daily入口，保留OS控制台helper而不把它偷算工具。两窗口各native job_processes=2，extra tool processes=0；不是结束后进程快照，瞬时退出子进程也累计。各子进程exit0/errorNone，返回native/C两个不同target的非空artifact以及不同before/after digest，links=0且shared-assembly/compile-link-tool/entry-wrapper attempted全0；元数据窗口内没有C编译“预热”。首次测试错误是Python原meter返回bytes不能直接json.dumps（exit100，未获得绿），改显式UTF8 decode原stdout/stderr后通过。再在真实Session::cold插入隐藏cmd /c exit0，所有内部links/生产断言仍可通过，但外部Job累计从baseline2变为daily6，负控准确exit100，finally恢复source_foreign.rs SHA256 `3540df97c5e04076dbbb0cea080f8afa52c228fbfdee25a257628463eddb493a`。

最终单项run6dc6e5d8-c545-417b-9e31-013a3c2a8116 1/1（4.23s准备/.596s整test运行，124未选择），实际daily窗口98.874ms、空harness71.167ms；这不是正式30样本/P90。原8bh完整职责90/90与正式31项的实现、C材料/源/期望/工具输入未变，新增仅test，不为提交重跑全矩阵。fmt/diff/cached diff/hook通过，原AV3 warnings保在案，无新增warning。Scope meter `tests/daily-emit-existing-input-25224/daily-process-meter.json` 含实际child输出/计数，不记录完整环境，不写项目内缓存。新case由原conformance target实际选到，不是未注册test。

关闭此前daily pure emit无spawn的待证项（边界为已有qualified provider输入）；完整正式metadata/P/M/R/shared tool计量、48MiB/索引与request128/store128所有剩余managed owner、真实scratch峰、全部SDK与原797/937/Q4/原6×30/P90/SD/939仍须完成。没有提前标T-10完成或移交复审。

统一scope日志SHA256（merge摘要后全删）：8bi-daily-emit-no-tool-processes-after.log=faf13ca6117d48ec0b98928696bed36a54ea235412b024108aba94de26164f86; 8bi-daily-emit-no-tool-processes.log=3e5a358caf69e318f2fa492f95fced65d80fab576917dbcd6d9a9afbf047afcc; 8bi-final-daily-emit-no-tool-processes.log=f190f11464d917a2171f08f6e7f0d6a5e7f0f815190244b0ff1e49e8001718f4; 8bi-negative-hidden-daily-spawn.log=b69f33ec0849d95ddec32bebf029e0f3114c107b5eff515894ef5864f9b19eac; 8bi-negative-result.json=585357979768447c1518f3b9ea30bd12435a7b98f8b5230623430664924fbcf1; tests/daily-emit-existing-input-25224/daily-process-meter.json=49238f6af81dbc462138549be15fdb5ad499fe7e501c9d2735a9ea69261a3d73

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: 7e393d07f64cb1dc8888f6f1860b5d6e3af437f9 | task_ids: T-10 partial | blockers: none | next_responsible_role: work actual request work/resource report and appropriate warm performance, then remaining original contract

2026-11-06 work checkpoint 8bj（T-10 CRT完整材料/共享预算端口与T-11 source暖门；work仍非完成）：提交 `29151ee3840179556a0810058a80ae69cb1b98fd`，12文件937+/52-，唯一树clean。既有首例warm>2000ms从真实parent窗口调查，不改阈值/样本/独立oracle。原每个actual/reference编译都启动vcvars，外部有界探针同真实C源/固定-MT/W4/WX/O2，两次shell构建1549/1543ms，而同一已解析SDK环境直接cl187.9/169.8ms；vcvars一次1592ms/39进程。未记录环境原值，只保存计数/byte量与工具输入/结果。并行构建原型正式31项绿，但相同真正30暖样本两端P90仍2457/2296ms（各30，28/22项>2000，功能均PASS），保留红，不据两项首例1963/1995ms冒P90。该并行原型已完全移除，不把线程、环境缓存或PE缓存作为交付。

最终source_crt::Toolchain：重用core原SDK选择策略（AC741_SDK_UM_LIB→注册表KitsRoot10→原known roots/最高version），只新增caller-owned有界registry查询接缝；旧find_sdk_um_dir保原选择/回退语义，owned显式输入不额外clone。source默认直接同一真实MSVC，显式VC/SDK include/lib/bin路径，同一CRT冻结flags；不依赖反复启动vcvars。Source SDK模式清除CL/_CL_/LINK/_LINK_隐式追加，确保报告里的固定参数实际有效；显式A2C_VCVARS64仍走真实batch授权路径。无新DB、持久环境/可执行缓存或source emitter；legacy bridge None入口仍原vcvars腿，797/937所有权不混。worker在真实core R之后只解析一次工具上下文，用于actual/ref两个独立编译；两者成功才执行任一PE，最后仍经过旧fresh authority/唯一发布点。该上下文费用保留到typed结果交给原有界transport（含cleanup/GC），不是在报告字符串仍活着时提前退费。

工具身份：source report现带实际cl路径/version/BLAKE3、provider冻结flags、CRT模式与固定CRT flags、选中SDK/MSVC位置及实际link.exe BLAKE3；显式batch带path和脚本BLAKE3。父纯SDK镜像要求完整字段集/唯一key/模式/64lowerhex/frozen flags，丢CRT信息、未知模式、缺link/script hash、改优化flags、重复字段或额外自称字段都不绿。原worker exe SHA+nonce/request hash与逐文件artifact-copy签名闭包继续复用，没有新自授compiler-role字段。metadata-only拒绝尚未用到CRT时不伪造实际编译材料。

预算端口：source CRT与平台查询复用原ctool_capture的两份1MiB先reserve/预分配/contained executor；共享原Resources与48MiB/8MiB I/O/8MiB index帽，旧build60/run5/total70及每流1MiB不放宽。注册表quota拒绝不能变known-root fallback；普通原平台工具缺失仍保原fallback。新增ctool_environment用Windows sizing API先读长度、预留UTF16+WTF8 old/new转换峰，再造owned OsString；缺失/空值区别、unpaired UTF16原样，最终退到实际capacity，ticket随owner并入工具context。before-read失败/重试也先drop数据后退费。本文仅关闭本新增端口，不冒全Env/HIR/catalog/query/ledger/A2X/命令/SQL/edges等其余managed metadata及错误文本所有权已全闭。

永久控制与红事实：新真实CRT共享IO差1B拒（无两个大buffer分配、rejection恰1），释放后原C程序实编译/执行42、current回baseline；正常default SDK query+CRT各两份1MiB、实际总4次大分配（显式无query时2），不是把额外两个SDK buffer忽略。初次仍按原2个计数真实100，改为逐原query存在性计实际端口数。新环境12289 UTF16 units/24578B buffer在index差1B时alloc0；合法读取在实际分配点已有完整fee，结束只留真实OsString capacity，末owner释放清；空/缺失/unpaired均对原API验证。显式CRT unit在Unicode/空格目录：ambient VALUE=99与错误link flags不能改变SDK结果42；坏AC741 override具名拒、不掉默认；真实显式batch marker与结果43证明没有忽略覆盖，hash匹配实际脚本；脚本正文保持ASCII并通过独立env/相对marker避免代码页假设。4个变异各100并finally按原字节恢复：环境先分配后reserve、CRT/SDK capture不传共享账户、未清除ambient flags、父漏CRT完整证据；负控日志与恢复hash见scope JSON。

最终实现验证合并证据：第一完整受影响选择81（source resources40 + ABI lib policy2/CRT1 + source conformance33/937 legacy5），实际73项执行=72PASS+1FAIL，8未执行（run日志保留）。唯一红是原pointer-encode控制在真实CRT准备之前已把IO压至128KiB，现正确的CRT2MiB前门先拒，故没有到其预期encoder；把同一压力/同一阈值挪到ready后，保指针payload2MiB、128KiB剩余、原root不变/repair与实际alloc0断言，无预算豁免。随后resource全40+CRT unit=41/41，14.200s（该次含前述8项，完整module不skip）。复用首72中未改的source33、旧typed三腿+负控、source export/body flip/required restore及其余消费者，合起来81个当前受影响case均有通过证据；不是误报首轮81全绿。父source report完整6/6，run日志含Root原145warning基线，无新warning；最后Resource模块仅原AV3。新test-only allocation字段显式先drop数据再drop ticket，未以allow压warning。

固定输入最终注册 `cargo t case-ace-ca-d-source-938 --out .../8bj-final-explicit-sdk-formal-certificate`：gate_passed=true，31 executed =30PASS+唯一原登记wrong-oracle FAIL，child exit1正确保留，runtime cleanup=true；执行44.289s/整命令126.699s。前一次direct SDK原型该31项功能结果同，但因本会话运行期间修正调用而source-state changed，outer gate=false，明确不计PASS；随后本最终门在整个执行期间未再写任何仓文件。定向fmt/diff/cached check/hook通过，工作阶段不是最终review冻结职责总门。

适用source性能完整窗口：同一个正式父adapter/worker/typed verdict/原始行逐请求落盘，数据取其post-persist parent-request span（body另加parent edit_ms），非worker内部span。每target一cold+30 unchanged warm：native P90=1116ms、C=1220ms，所有60暖样本≤2000；再每target一cold+30真实source编辑、共61原步骤（<原64步帽），20,22与19,23交替且oracle一直42，actual/ref各自对父值，30个body源hash均与前次不同、source_parse/solver真>0。body native P90=1079ms、C=1166ms，全部60编辑样本≤2000，未缩样本或将原6场景替成这4组。材料/独立源副本/输入suite和输出都在统一scope且输入输出分离：首次误把输出放输入root内被原output admission拒（exit2），没有绕过守卫；改scope内兄弟输出目录后正常执行。原并行性能red始终保留；原797/937六场景×30、其他source/native/A2X原负载/Q4与所有最终SDK消费者仍pending，不据此勾完整T-11。

仍executing6/13：T-07全真实P/M/R/attempt/reuse/discard/IO（含shared host/ref tools）和T-08全矩阵需按现码复核、managed frontend/冷接口/ledger/A2X/工具其余metadata、scratch实际峰/request128/store128全写口、旧全scale/SDK/Q4/SD/939仍开放。新有限矩阵31、daily零tool spawn、CRT材料准入、本轮source warm/body30两端性能已取得当前证据，不要后续重复列为未实现。未移交final review，未改变目标或验收。

统一scope日志SHA256（merge摘要后全删）：8bj-complete-crt-parent-controls.log=9a52c68a9bcedc03330e294e84cba442fb628fdc938459362555458980c04d14; 8bj-concurrent-crt-check.log=48d78596794e76a00314a1176d1449e095ecf7d61d479f14b9c1d20bd8ee3d0b; 8bj-crt-capture-admission-controls.log=98815dcb0a88750e56efab827fa75c9b232091a96d65e77e67d3fd57fa42da15; 8bj-crt-env-investigation.log=8d313318ec2d631241da94ae45a716b6ae3ff1b9c7c51aff771a71ae80efc77b; 8bj-crt-environment-and-real-mode-controls.log=b2708a204922a96cccf1a686b912f762c8af98e9dcaa714c56ff2f592bcf7096; 8bj-crt-mode-and-frozen-flags-control.log=86647572a05526e043372058bf10a838f46dee15c2cb290843c802f3869b8c93; 8bj-explicit-sdk-capture-controls-counted.log=495dfda6ab011d6e1bc02bd0688505b85ca044858d79628ba5838d993f69b505; 8bj-explicit-sdk-capture-controls.log=7c93ed099087c3510642bc68ca955bb16175505e9ac2c99a246db4c5e131f263; 8bj-explicit-sdk-check.log=9accf381fd02f120cbabc578a6d8fe4f2b08ba666ddf9ed384bdf3babe64053e; 8bj-explicit-sdk-environment-check.log=998c0a1ece28f8b085a1e9307427cad662b3e3367e8bf7f879e48c5a54399a8b; 8bj-explicit-sdk-formal-scalar-certificate.log=a12c910cae1d70d5045ac3f5842ebbe868913b570fd7221bbb6703f8ad3c8126; 8bj-explicit-sdk-probe.log=885639f8b6247356b496aa3ea1768bf73d8ed795cddb30bd118db424518a9422; 8bj-final-explicit-sdk-formal-certificate.log=09b1175a1ad359826434492d230514eb6205217f4cd2ec16c018a4ef8669eabf; 8bj-final-parent-crt-controls.log=9fa153a8a0028a51be1f48f992a55d778f785c1e35731c65c8ea1b63752bd2b7; 8bj-final-resource-module-after-pointer-seam-fix.log=11315525529d4f06074664677b7029264fd3dc8abbaf625c456532e391624aa3; 8bj-final-source-body-results.log=3aee744b7fa3e4c57e803dbfed2dc0dd08df22c1193ace45314f317432f0da25; 8bj-final-source-crt-resources-and-consumers.log=6805a03db9a6a79e695b0f68c1eefa1043cf4726990da7f6ad409bf8f947fe6d; 8bj-final-source-warm-results.log=d0935cdafd8be6cb078c411aff6b97222def3ec1a0b03458dd74e52e7de7b1d9; 8bj-negative-ambient-compiler-flags.log=cae1096c2e91fb72b7d07d3c808c0a800224a73d6eb29ed23b45936448df1f3f; 8bj-negative-late-environment-reservation.log=f96b941e2c61e066e3b8e572bde7a7275702b24d1b7649ad058ba98c45473318; 8bj-negative-parent-omits-crt-proof.log=81c6efd197bb4831531dcb28ad3757b0512f19073522aaa18f9bef4faeeee0e2; 8bj-negative-unadmitted-crt-streams.log=33a823ae6db52bbe189b053b8d2c42c608b3cd3f924392a7add188733791ddd6; 8bj-parallel-formal-scalar-certificate.log=a1820ef3113a2c35df40af68535faeffa3462b1868c8fe2e713000234b41b862; 8bj-source-warm-results.log=d0935cdafd8be6cb078c411aff6b97222def3ec1a0b03458dd74e52e7de7b1d9; 8bj-source-warm-samples.log=27a0346913f2feee696434274f1c3204c6dfdcc8d9b5c656c22bfefe31971072; 8bj-abi-negative-results.json=df650ecdcf5430d479745913d8502a0b757e4737db75f7209031cebd5e31e45f; 8bj-parent-negative-result.json=dc551c49883aa727734d40bd68e34d0f4adc22bac8bc41c9b06d84c51f82ef58; 8bj-final-explicit-sdk-formal-certificate/results.json=f29b4c54fbac937240aaadccd6bf51397b20dc2a2b867d610094942f90626996; 8bj-parallel-formal-scalar-certificate/results.json=67a60ef86b40f74efafc7aef1f3aaa7b558a9a1fbd3dd5f3eb5a3f099ded7ad6; 8bj-explicit-sdk-formal-scalar-certificate/results.json=d562408d0eb6d9a606b7e363ad0a07504e2a580ec85add96170ef45fd3ffe14e; 8bj-final-source-warm-results/warm30-full-parent-p90.json=76d3745705a432787b7b6663fdb01905690e15567fccc48b5cc80ea6d9bbe81c; 8bj-final-source-body-results/body30-full-edit-parent-p90.json=a7f7ed62489e38703fa53cc5313ecb09d007b7ae99b5e2404ab77a000c6b520a; crt-env-investigation/summary.json=3938e8ca2f6d4c6c1b668ef9ae13441b425f543214b92c8a0bc3c72072100211; crt-env-investigation/explicit-sdk-summary.json=ae59575d003b99e2ad67e5d80494a4f33fd00162b6d99364ffbfa666df501fed

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: 29151ee3840179556a0810058a80ae69cb1b98fd | task_ids: T-09/T-10/T-11 partial | blockers: none | next_responsible_role: work complete actual request work/resource report and remaining owners/original workloads

2026-11-06 work checkpoint 8bk（T-07/10真实请求work报告端口；仍executing6/13）：提交 `29e2de48dad761f28acbde458c84696cb70dd753`，7文件504+/8-，唯一树clean。沿现有source-query Ledger导出完整逐owner/stage的attempted/produced/reused/accepted/discarded/pending、P/M/R与logical read/written bytes；新增借用Serialize直接持锁遍历现有行，不调用rows()克隆全部owner/另建Vec或第二份索引。worker/report与父纯SDK镜像同步迁移到`abi.source-import-report/2`，旧/1、缺任一账本与未知字段拒，旧ABI/bridge协议未改。cold从真实ColdObservation开始绑定账本，restore从真实restored Frontend账本开始，涵盖之后首编辑、错误与publish；最终六个兼容生产数也改为发布后读取现有meter，避免pre-link快照。前置准入拒绝可有空账本，但不能凭空带执行；predecode resource拒绝不得带任何work。

共享host端口单列：原provider-cache结果/命中、仅实际miss时provider-compile、原ctool_capture的每个有界命令调用、CRT上下文M、reference独立compile、actual/reference各自run与实收stdout/stderr字节；写出的provider/reference工具输入记logical written bytes。tool-process是原contained command入口次数，不冒其所有OS子孙进程数（后者仍由原Job原生计量）；各层独立端口不把嵌套P相加冒CPU/唯一生产数。新host账本只允许8个固定stage和'*' owner，先reserve16KiB index覆盖其BTree/Arc/短owner/tickets，再分配；report/当前观察scope共享真正owner，最后一个释放才退费，unwind恢复外层observer。source已有其余动态Ledger/maps/HIR/query等metadata原开放项未由此16KiB冒全闭。IO是实际观察端口的logical payload，不冒物理磁盘/SQLite内部流量；顶层infra/cleanup失败仍沿原error envelope，完整错误报告与剩余未观察端口待后续核查，未以此勾完整T-07/T-10。

父侧每行检查严格有序唯一、非空owner/stage、checked-add防整数overflow、attempt=P+M+R、produced=accepted+discarded+pending、终态pending=0；host owner/固定port白名单，成功必须有真实source行及actual/ref run/reference compile/CRT/provider/cache/tool端口，phase正确且必要单次动作恰1、至少2个已完成工具调用。保留正常工具发现失败尝试（attempt增加而不凭空produced），允许后续合法原SDK fallback，未误要求所有发现尝试成功。

永久真实控制：新host unit在共享IO差1B时记录attempt但真实cmd marker不存在，释放压力后同命令实执行且捕获字节入账；scope在report owner先drop时仍持费用，末owner释放回原baseline；panic后不泄漏observer。最初把Resources自身184B index基线误断言为0，测试100，改比较实际baseline；原cmd args额外引用导致stderr68>测试64帽，测试100，改Windows raw_arg保原容量/真实marker断言，未放宽帽。一次重复--lib的命令用法exit2修正为原包选择，不计测试PASS。新父case含11个独立坏轴（丢source/host端口、pending/accepted/phase/重复/overflow/错schema/错owner/未知port/漏真实工具）与合法发现失败控制。已有真实进程首错→修复→再restore两端控制新增逐行平衡断言；warm每函数P=0同时真实R>0、host reference编译/两次run仍P=1，provider hit=reused1且无compile，cold必须实际compile。声明Add→Scalar→Add整条旧协议过渡控制同时保留。

验证：source work两项原失败/取消/accept/unwind语义与新hostunit共3/3，run24ee331f-2e43-4c1c-a0fc-72e7ceb0a6f9，0.095s；三变异各100后finally逐字节SHA恢复：删真实工具观察、删restore账本、删父平衡检查，分别由实际attempt缺失、warm R缺失、pending坏账被误接纳断言捕获。恢复后最终ABI hostunit+真实source两case=3/3，run4d212949-31ba-4d7d-b7e8-e44e38580b2a，25.996s；父source全部7/7，run日志，0.148s；无新warning（AV原3、root lib-test原145）。fmt/diff/cached check与提交钩子通过。

正式`cargo t case-ace-ca-d-source-938 --out .../8bk-final-work-report-formal-certificate`原注册31项执行=30PASS+唯一原wrong-oracle FAIL，child exit1、包装gate_passed=true；actual执行61.596s，整命令143.583s。门内未改源码。父实际落盘33个带nonce/request_hash/worker exe SHA的/2 source报告，cold Native32行source+8行host、C30+8，实际call/reference/oracle均保原规模，新增phase/IO确有实数而非置零字段。原完整职责冻结并集仍属最终复审后merge，未由定向门代替。

剩余：scratch_peak_disk_bytes仍0；request128/store128所有写口与原A2X quota owner复用、其余managed metadata/失败出口、原797/937六场景30/全原负载/SDK/Q4/SD/939与最终独立review→merge/清理均开放。已关闭本报告导出和以上具体host端口，不再重复作为全未实现；不宣称所有IO/错误出口/配额已闭。证据统一scope，merge摘要后全删：8bk-final-work-ports.log=02ce41e4b7cfca7192f8787503229a11953dcc8e1210c34b3203ca9e97266526; 8bk-restored-final-abi-work-ports.log=97dae887be9d341abf2371333c89e39a55a77605395e574c79d297471b89e346; 8bk-restored-final-parent-work-controls.log=338b166eed3b96444626efc0cd32f7e1076d3a0e1062a69e3133b7aaad4dafb2; 8bk-negative-results.json=9eb604373672b0fd2694459422bc03f39e1d1e6110dbd6bb34ca5a62d512b639; 8bk-final-work-report-formal-certificate/results.json=266d719591e04c68f418e906d37515789e777b3002f6e70e5a32697b4894568d; 8bk-negative-tool-port-omitted.log=468775748cb262f84a88163dbecc7edac725de7d330823556842994dfa47fe75; 8bk-negative-restored-ledger-omitted.log=50cca5fd869311f358b76f95b11043996c074f6dc80ef6886c3a902b9d26bd6b; 8bk-negative-parent-balance-omitted.log=7acfacdb6b89fe94ae01dbc0da54ff96eb8a2131db73a7b3725f0a77b7818262

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: 29e2de48dad761f28acbde458c84696cb70dd753 | task_ids: T-07/T-10 partial | blockers: none | next_responsible_role: work scratch actual peak / cumulative quota and remaining owners

2026-11-07 work checkpoint 8bl（T-09真实工具磁盘采样与资源生命周期；仍executing6/13）：提交 `f340faa750b2944de62f7973575e9103067033b3`，6文件385+/16-，唯一树clean。删除source StepOut scratch_peak_disk_bytes常量0；不拿原store的process-global RESERVED transient峰或事后剩余目录大小冒实际工具峰。复用core原受控启动/单调截止/流帽/树终止与reader回收，在同一executor新增caller-owned观察接缝：启动前、原10ms wait轮询、正常退出后观察；旧入口默认noop，既有caller-owned capture两buffer预算仍先reserve/分配。观察失败显式终止/等待/回收整个包含树，观察panic捕获为原SubprocError::Wait，同样回收，不新增漏迁enum或第二执行器/采样线程。

source私有ctool_capture安装观察，已分配buffer的费用维持到返回/失败回收完成；观察错误经原Resource分类，不伪装编译产物或PASS。每次只扫描本请求注册的ToolScratch有限flat+reference形状，不跟随reparse，不看另一活动请求目录；求实际logical file length和，记录单调sampled high-water。provider与run顺序所有权、必要的两个同时活动scope均保；跨scope的同时文件合计而非只取单目录max，scope.finish清理前补测，删除后仍保峰值。必须如实称采样最大值：10ms两点之间的短暂文件、等待子进程结束后的pipe collection间隙不会获得精确连续峰；不据此冒全物理分配量、所有namespace写口或配额执行已闭。已审定有限actual/reference PE只输出原stderr事件、没有额外文件IO；native/C产物和reference编译及其输入都在被观察的原工具scope。磁盘元数据/分配粒度不冒payload bytes。

所有权：ToolScratch在原path/cleanup index预留后才建立Arc<PathBuf>，向请求固定两个观察槽共享同一path bytes，未复制路径；registration先注销共享path再释放原index费用。host固定账本16KiB预留同时覆盖新增固定槽/Mutex/Arc头，path buffer仍由原ToolScratch票持有；末真实owner释放，旧observer scope/panic恢复保原8bk规则。无新DB/原始环境/持久PASS或cache。测量出错仍执行原owned cleanup，unknown/reparse不穿透，锁阻塞保可重试。

永久真控制：新unit两个scope+reference1024B，工具写65536B持有250ms后在退出前删除，末态目录回初值但采样峰≥初值+65536；结束-only实现被负控100抓住。unknown子目录在启动前Resource拒、marker不存在，移除后原cleanup可继续。另一真实Python持open文件131072B并睡30s，原1s deadline终止包含树，峰保留且返回后立即删除真实持有文件成功，reader收齐；费用末owner回baseline。新core控制分别在真实child持文件时observer返回Err或panic，两者均Wait具名失败、可立即删文件，证明不是只声明取消。原ToolScratch全部active/dead/锁清理/unknown/junction控制继续绿。

证据：initial ace-abi check exit0（11.77s），新scratch+host unit模块7/7，1.917s，run3e83ea50-618f-4899-a412-e3d38f31ba62；core link完整原模块+新控制16/16，3.547s，run0ff966fd-95fd-4d30-ba47-bff7a69b30af（原60s显式ignore仍不改、不冒PASS）。3变异各100且finally恢复原字节SHA：去wait轮询只末态测、漏observer panic回收、报告重新置0。恢复后一次明确受影响并集25/25，27.858s，run95e4a0da-ad60-4d59-a41b-e7487eeacbf1，526 filtered/original excluded（不是526执行）；包含core link16+scratch/host7+真实两端首错/修复/再恢复及合同过渡2，旧root不安装错误候选与独立实际callee结果全保。无新warnings（既有AV3、conformance unused-close基线按原日志），fmt/diff/cached/提交钩子通过。

正式原注册31项gate_passed=true、30PASS+唯一原wrong-oracle FAIL、child exit1；actual 53.246s、整命令109.953s，门内无源码修改。父已有严格Meter reader与逐请求native counters无需协议字段变更，/2 work账本保持原样。父实际落盘31个case meter中27个有真实正峰，范围8296..2274025B；前置声明拒等未创建工具scope的0保留，不能强改成伪峰。各case明细在统一scope 8bl-formal-scratch-meters.json。

仍未关闭T-09全部：request produced128/store namespace128全写口原owner接线、动态Env/HIR/catalog/query/ledger/其他metadata、完整错误envelope与原规模性能/SDK/Q4/SD/939等前述开放项保留；此单元只关闭上述采样字段与实际端口控制，不交最终review，不改原阈值/样本/支持矩阵。证据统一scope merge摘要后全删：8bl-initial-scratch-check.log=a95d6d38343d23c90e47389001d1cd312017cab0b629c922b8228e31bddd87d9; 8bl-scratch-owned-samples.log=5939c5c9d54e3e29b108ec0bc11e82e9e9c71cdfb5f73cb1c32a2b3c9ab612b8; 8bl-core-link-observer-and-legacy.log=1cb4b464051ff23be6a479711822beb952d54c8e2f6dbc0604d543a64ceb09a0; 8bl-final-owned-observation-and-consumers.log=924510102df29dfa789046fc62b7f08cbbcf6a313998d3d804e855eaf377b792; 8bl-negative-results.json=c380c04b20ce0a80234106ac8fd4752bb99e271e9d396d712851f8a95b747261; 8bl-formal-scratch-meters.json=d9b1818c7b7edaa899c61d8b92d52318e3696cd04e3827a846ad3b3485b71f70; 8bl-final-scratch-observed-formal-certificate/results.json=95a40a9900969baf032f937dd6cf5029ff67648d6e21432baeb762320453c26b; 8bl-negative-terminal-only-disk.log=eea5364aa8d029f502a369d20b5b08a8a167a95ad9712217de782ad99c794c6a; 8bl-negative-lost-observer-panic-reap.log=2a906f9e57bf8d08490335418bcfc8d7bc7814922dea0127b16a262007a896b8; 8bl-negative-zero-disk-report.log=5b1bb3c521b5d68314452c203b39d857e992f925b3ba2d46fe5613e079c88319

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: f340faa750b2944de62f7973575e9103067033b3 | task_ids: T-09 partial | blockers: none | next_responsible_role: work request128 / namespace128 and remaining owners

2026-11-07 work checkpoint 8bm（T-09已知物化产物先预留与累计请求账户；仍executing6/13）：提交 `915e88d50a92fbe86c8e1f4450c7095ff503f738`，9文件463+/16-，唯一树clean。先核canonical ABI windows-x64-v01资源条款：单产物64MiB/请求累计128MiB，生产前检查预留、生产后复查、超限本请求终止。原worker legacy produced_bytes只在producer前检查已花费值、事后加obj长度；本包不把该实现冒完整先预留证明，不把新累计账户做成第二DB/另一GC/持久PASS。新账户在现有source-query Resources内固定State（自身新增字节随原State size先计index），与48MiB live memory/16M MIR/24M staging/8M IO/8M index并行；实际source认证请求一开始启用64/128，旧长期library Session仍由host定义请求边界，不将512次生命周期churn冒同一请求。payload不能开关/重置/加限。

OutputPermit：先以精确size或已证明bound同时检查单产物与produced+pending≤request，再分配；取消未完成者只退pending，完成者按真实len转produced，后续丢弃/释放payload不退累计已花费。实际完成超过预留也记实数并拒；一次quota拒后锁定本请求，连零字节后继与先前pending输出的成功接纳都拒，必要cleanup不经此生产许可；下一独立请求新账户可修复。Host只复用原ABI_TEST_PRODUCED_BUDGET_BYTES收紧入口，native sizing env getter先预留owned UTF16/WTF8，名字栈上固定；值至多128MiB，非法/缺失沿原默认，不由deck输入授信。新的quota拒在source/target/publication/store出口按账户真实refused位分类foreign.resource，不靠输入自称码；既有其他错误envelope与managed错误细化仍待全项核对。

本单元实际接线范围：Frontend事实Page、Frontend checkpoint root、source target JSON artifact、source session root、原store carrier root以及真正native COFF output。JSON均重用原json_size→reserve→有界encode，COFF直接在object::write真实reserve(size)边界先请求许可再try_reserve_exact，完成后计真实bytes；distinct编码产物各计一次，不把Arc重用/恢复decode/纯拷贝/整份IO当新产物。单体既有更严page/root/native帽原样保留。外部C编译新obj/PE、C fragment emitter、其他MIR/foreign逻辑codec生产端口尚未全部接此账户，**不能由这些已知端口称请求128全闭**；namespace128 all-writer与SQL/transient/工具写口仍开放，后续沿原store owner接齐。

永久控制：真实4096B section COFF，精确对象大小L，两次真实物化逐字节等于原object.write，每次真实buffer allocation恰1、produced累计L/2L且drop payload后不变；第三次在全局allocator阈值L的探针下alloc0返回request-output Resource，单产物差1B同样alloc0。primitive覆盖pending竞争/取消退款、计真实小于bound、终态拒绝、超出bound记实数/拒、禁止扩64/128与禁止reset、独立下一请求。真实native/C worker均cold42→restart第一source编辑host预算0返回foreign.resource且无ok/旧pointer逐字节不变→独立正常请求修复、actual/reference各42并更新pointer；不只是输出一个错误旗。Host测试0/123/u64MAX/invalid对固定64及request≤128验证。

补齐测试目标接线：8bk/8bl使被source_resources直接编译的私有ctool_capture依赖source_request_work/owned scratch，该特殊allocator target先前未同步这些模块；本轮显式编入同一原实现，保已注册source_resources入口，不新建探针算法。其完整49项（原40+同实现host/scratch7+COFF原unit1+新allocator控制1）实际执行通过。首次补模块只产生reuse未使用warning，补原高风险host-counter unit的reuse不造attempt/output语义断言并按stage找序列行，未allow压警告；实际provider cache hit仍由原worker复原集成控制证明，未用合成counter代替真实命中。

证据：ace-abi check exit0，12.36s；真实allocator控制1/1；完整受影响选择55/55，runb329b3da-298e-4ab1-810c-6483f5347486，52.589s，451过滤/原条件排除，含完整资源49+native COFF原1+primitive2+实际worker3（quota拒绝修复/原首错修复/合同过渡），不是55=全包。随后新增host控制及加强pending拒绝断言3/3，runf29802b4-5857-4a89-8146-a0503dc10d50，0.036s。四个变异各100、finally按原字节SHA恢复：COFF先alloc后quota、quota拒绝非终态、request cap可扩大、worker忽略host收紧；分别由实分配计数、零后继控制、caps控制、真实请求pointer/Resource控制捕获。恢复后最终7/7，run8c289d37-1f25-4064-91d5-d233ab41f332，32.064s；对未改身份的其余49项复用前55证据，当前56个不同受影响case都有绿证据。没有新warning（AV原3/native conformance unused-close基线）。fmt/diff/cached与提交钩子通过，测试无缩规模或预算豁免。

正式`cargo t case-ace-ca-d-source-938 --out .../8bm-final-known-output-quota-formal-certificate`：31 executed=30PASS+唯一原登记wrong-oracle FAIL，child exit1、outer gate_passed=true，实际89.994s/整命令136.745s；全过程未再编辑源码。/2 parent/worker身份、独立oracle、8bk source/host账本及8bl采样磁盘峰均保留，parent wire未增字段。

下一工作仍完整保留：核T-07/08当前实现与已有证据的闭合状态（不能因旧open_details反复重做）；接齐余下动态/外部生产端口的同一request账户和namespace128原owner；managed Env/HIR/catalog/query/ledger等真实归属、完整错误报告、原六场景30/负载/SDK/Q4/SD/939与最终独立review→冻结并集→merge/spec/archive/全路径清理。无work-complete交接，未改变原验收。所有日志/backup在统一scope，merge折叠摘要后全删：8bm-initial-output-quota-check.log=21fbe1b929b7d7467568ac6159c850e762d9f66930214dc44e6ea9b26a3c178d; 8bm-coff-request-preallocation-control.log=1a6807cb514e9b25ed7238ddf25cc4914ebe7c8d3ff9069bf9d1aa1bcfde29c5; 8bm-output-budgets-and-consumers.log=4002fac29568bddbeff3aaa503516bca007d87102215fbc21a13c822108e3822; 8bm-host-output-scope-and-limits.log=dcd4b723a68852840d3364bf50c2378a0a8a01e9805596c906612d125034e0c6; 8bm-final-output-quota-controls.log=915745ff48fadc073c80ae85d7446a2b45e2707eb24bf891e318999d601ce25e; 8bm-negative-results.json=c54862fc68a978f99d527cc81e8497add24849915086a3d48209d8c737ff605c; 8bm-final-known-output-quota-formal-certificate/results.json=b494e7b37fe2f64d83b1b9b2f8c992ca70c81f28860a8cba7b440b6b32a6f26f; 8bm-negative-late-coff-reserve.log=16162aa4b352b30d849db0a6fbfcddce31320ea7d8b6c9a9231782a957333d5f; 8bm-negative-request-refusal-not-terminal.log=9b4f99e6d44649789e452a339e2cdd94db7b4b38a012c982b1a4a01e3a3182d0; 8bm-negative-request-cap-expandable.log=258f1673f5e3d09b304c5c4befb6bfecb260975036b0d46c50fae69fb6047ba1; 8bm-negative-worker-ignores-host-tightening.log=b9767459d0004c2af577bf2ced43a7d304db8996db3a780690fabf8337207d06

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: 915e88d50a92fbe86c8e1f4450c7095ff503f738 | task_ids: T-09 partial | blockers: none | next_responsible_role: work reconcile T-07/T-08 evidence then finish remaining quota writers/owners

2026-11-07 work checkpoint 8bn（T-07/T-08按实际合同与当前证据闭合；不是final review）：提交 `f0abefda16d41a076ea6b8e139f14d9a6a950af1`，仅conformance新增143行正向控制，生产源码未改，唯一树clean。上一goal turn=progress（8bk/8bl已提交），本轮8bm账户与本35+1项审计均有实际状态/证据推进，不是反复open umbrella；本次据§5.3/5.4、V-04/05/06、AC-04/05逐项核验后勾T-07/T-08，current_step8/13。T-09全部预算/metadata/failure cleanup/512与T-10完整report/SDK及T-11原规模门仍独立开放，不因勾本两项而核销。

| 合同规则 | 实际当前路径/owner | 直接证据与边界 |
|---|---|---|
| 分层逻辑签名/physical/provider/body/binding、真实非内联caller P0 | source-query SignatureCatalog replace_foreign与resident正/负依赖选择→checked单函数facts；a2x semantic_fingerprint→FunctionCache key（name/semantic/target/runtime/caps，不含整provider实现）；provider只改绑定/link | 当前ABI formal_worker_rebinds_provider 0→1→0两端actual/reference42→43→42，六个caller生产数全0；source/target artifact历史命中先于lower、原A2x store命中；SQ signature/negative/name/source-map当前重放。不是typed桥替代source。 |
| declaration/effect/header/cap真实reader、不读者P0/IO0与payload身份 | Frontend.prepare_foreign_source按resident读取版本/negative-candidates选读者；未读body不恢复AST；Snapshot.persist_leased unchanged id跳过payload IO | declaration_readers_atomic after R：首错签名保旧root42、有效effects变更只helper；真实header文件重编译+审定manifest变化使helper/flag重查，main/independent meter/Arc/facts/body不变；name_changes保retained slot；work_disposition丢候选discard、接受计accept，independent store-leaf read/write不变且reused+2。header搜索/目录负事实与Unicode边界沿8al..8ao已记录实际C-cache控制，不把未改cache实现重跑成新全量门。 |
| 已授权能力变更必须可用，不能只测拒绝 | 新case在原已支持的SOURCE_CAP+hir.test.trace.v1 profile、原provider/header/table/source/逻辑version全不变，仅manual声明要求trace权限变化；当前host独立批准新contract/generation | 新正向case两端cold→真R→第一编辑加cap，helper/flag恰2 solve、各一次HIR与选中target产物；main/independent全部meter/事实与目标Arc/body身份不变；host撤销trace在link/tool前拒、目录不存在、root不变；合法host实执行42→再R首编辑撤要求、恰2solve且不读者P0→实42。没有扩Native cap支持集或变oracle。 |
| restore仅R、不ensure_env/空编辑预热，首错/修复/fresh/再R和前缀漂移 | Session.restore_current / restore_stored_current_with_resources→required foreign/source/query/selected target页深验；query/core own facts恢复；source/foreign edit唯一安装 | selected_targets_restore first edit首错保持旧root42，helper修复43与fresh产物字节等价，main/independent meter0/Arc同一；再R先无关长前缀编辑、第三R再改helper44、provider-only45不生产caller。独立进程restart-probe实际restored全部producer0、首错拒/第二修复43（父独立oracle）；real_signature_and_source_repair另跑独立process首声明错误→同源与新真实C原型一起repair。当前formal首错/修复/再恢复与合同Add↔Scalar完整entry反隐生产控制另已在8bm final7通过。 |
| required坏页/错owner/未知旧协议/伪Checked或自洽重哈希不能授信 | Root/StoreRoot与ForeignPages严格typed decoder、current host contract/provider/runtime/target authority，per-target artifact owner+MIR身份检查 | required_restore逐个缺/坏所有页，错schema/target/contract/provider、伪safe、权限撤销、错SDK runtime、整组自洽重哈希runtime/source/foreign-declaration/target-owner均拒；合法R接受且source parse/solve/MIR/native/C为0。原source/query页面仍由各owner恢复，无另一parser/solver。 |
| source+HIR+foreign+selected target原子组发布，拒/迟到/撤权保旧源与产物 | PreparedPublication持snapshot+selected exe digest+owner/base，run和唯一pointer switch均fresh authority；ManagedStore required edges/leases与pointer项目root | publication_faults当前完整before/after link撤权/cancel、错runtime、ready改文件、registration前后/encode/replace/late-base/late-pointer等拒绝均守旧root/pointer/不读者；独立旧root可运行，合法repair可恢复。运行前授权失败没有callee执行，已授权实际run后再发现迟到不伪称从未运行。资源超限/GC/cleanup所有权另归T-09，不冒512完整门。 |
| actual P/M/R/attempt/produced/reused/accept/discard/pending/IO | 原source-query Ledger实际producer hooks、target/cache/snapshot ports、8bk /2 stream source+host账本；原Job采物理工作、8bl实际owned sampled disk | 当前SQ frontend_work与work failure/cancel/unwind及ABI work_disposition、正式两端actual source42/43独立reference；8bk/8bm的新父严格行守恒、wrong-stage/掉端口反控均沿当前未改report/SDK镜像保留。IO定义是所观察payload端口logical bytes，根checkpoint/shared assembly/tool全局行明确单列，不冒OS物理流量或整源重编译为R。顶层infra/error envelope的完整证书细化仍由T-10完成，未以此丢弃剩余任务。 |

当前重放集合：`cargo nextest ... -p ace-source-query -p ace-abi --lib --test conformance -E <8bn日志所载表达式> -j1`，35 selected/executed=35PASS，219过滤/原条件排除，run d7906f41-f3d4-4690-8cb6-f493cda7b135，38.252s。25项SQ foreign/work与SQ04/06/25/34/35/36，加10项ABI声明/名字/真实process/rebind/publication/签名repair/required/history/disposition，验证职责明确，不是全experimental。冻结生产提交915e上的结果与本test-only提交相关生产/输入/deps/config身份相同，故复用；不为test注册改动重跑原正式31或全部性能。原冻结职责总并集仍由final review后的merge承担。

新正向cap first run真实100，原因测试用了本来不受支持foreign.extra，Native前门正确mir.capability-unknown；这是fixture边界，不是产品bug，保红日志。换成原支持的hir.test.trace.v1后1/1（1.907s）绿；没有放宽白名单。随后实际变异SignatureCatalog.replace_foreign强行保留旧reader版本，新case在cap第一编辑后solver=0而期望2，真实100，证明cap依赖oracle不是靠其它变化过关；finally逐字节恢复catalog SHA。恢复后最终1/1，run449373de-4522-4975-b0c4-15124eaddd1a，1.666s。总36个当前closure cases有绿证据；同一条负控与此前actual边/状态/IO/bad页控制保正反区分。fmt/diff/cached/提交钩子通过，无新warning。

本次为work验收核销，不是独立复审pass，不把status改reviewed。下一步T-09继续8bm同一累计账户的剩余C/compiler/MIR/foreign端口、原namespace128全部实际写口与动态metadata/cleanup，T-10/T-11/SD/939/最终review/merge/全路径收尾不缩减。证据唯一scope，merge摘要后全删：8bn-t07-t08-current-closure.log=61dfc4110d480a9c405e42a24b7d15431fbf64f35cb10505964defdc3db7d7d0; 8bn-authorized-capability-first-edit.log=140491a312f7517b5af712868dca9be0abb1778d3d810131feed24dc344b8057; 8bn-known-capability-first-edit.log=74159587ee722190c496c513186202120c7ca73250feb2ffbdcb0b71bce0a4b4; 8bn-final-authorized-capability-first-edit.log=13b5da8c44f6720db263efc544f1caa8e00791c779e8531e586767c3ac20e78e; 8bn-negative-ignores-capability-reader-version.log=534ccd027597f3160ba4c87efcff9341a6be0228fafbff1033c63fe48d7c5aab; 8bn-negative-result.json=f18cfe0f50696c8bd53d04058388641a63553723c28b54360397087d41b58d08

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: f0abefda16d41a076ea6b8e139f14d9a6a950af1 | task_ids: T-07/T-08 complete | blockers: none | next_responsible_role: work T-09 through T-11

2026-11-07 work checkpoint 8bo（T-09 MIR/entry-wrapper累计产物与真实失败释放顺序；仍executing8/13）：提交 `aa7cdc702651d4b49c9cb79a62c8678df3005625`，4文件438+/48-，唯一树clean；上一goal turn为progress（8bm账户与8bn关闭T-07/T-08），本轮继续真实T-09端口，没有重开已闭任务。

实际接线：沿原ABC counting preflight加独立output-size回调，取得包含header/trailer的真实wire总长后，先经同一Resources请求64/128许可再创建page payload；原encode_abc_reserved入口通过noop回调复用同一encoder，各family/schema/bytes不变，不增加源/编解码实现。native source在MIR编码成功时按paged.total_len记一次完成，之后原to_flat只是同一产物的拷贝，不重复花累计产物；paged/flat并存仍沿原staging峰预留。native source每次link使用新emit_wrapper_coff_reserved，通过同一真实COFF writer先reserve实际output，再生成Vec；以opaque ReservedWrapper的bytes先、allocation后字段保持真实memory费用，write失败也先销毁data。旧legacy wrapper仍走原ObjectProduct.emit路径，未给797/937入口改支持面/预算，caller/root/普通wrapper字节保持原值。新接口为私有实验Rust owner seam，不升公开AutoABI。

由真实quota反例修复两处原native failure生命周期：COFF已完成后，MIR产物quota拒绝时原tuple绑定使票先于object销毁；改为先声明票再持payload。target JSON quota拒绝时原factory局部decoded_charge先于入参decoded销毁；改closure持decoded而外层持原票，resize/size/encode/quota/index所有失败都先释放真实data，再退费用；成功仍用try_retain附到同一Arc，不加副本。没有改变48/16/24/8/8、64/128或暂停必要cleanup。Wrapper内部CLIF builder及其他既有metadata工作态不由本输出guard冒全已计费，T-09的动态metadata归属审计仍留。

永久控制四项：1) 合法scalar MIR加16384个未使用i32常量，原wire为独立control；request比真实wire差1B时全局allocator>=4096的真实page分配0，合法同大小生成逐字节原wire，flat copy后produced只等于一份wire、末ownerstaging归0。2) startup wrapper与旧legacy逐字节等价，produced=真实COFF len，drop后累计不退；在实际Vec deallocation回调确认staging费仍存在，然后归0，单产物差1B具名拒。3) 真source native先facts+COFF耗尽预算或facts+COFF+MIR耗尽预算，分别在MIR/target JSON拒，allocator在COFF真实分配时按pending=L定位其唯一ptr，在实际free时验证staging/resident对应最低费用未提前退；当前source与事实/target Arc都仍旧，pending0、已完成字节精确保留，后继snapshot生产也终止。4) source prepare_publication的预算只差wrapper1B，实际source调用端口触发拒、compile-link-tool尝试数不变、原payload同Arc、source未装新版本。不是只测新增helper或人为counter。

验证：initial ace-abi check exit0；初四项控制4/4 runb864019d-320f-45f0-b9b6-97e99d2c9af8，0.602s。五个真实变异各100并finally按原字节SHA恢复：MIR先page分配再quota、wrapper用另一无累计限制账户、wrapper票先于Vec drop、还原native tuple旧错误顺序、还原target factory旧错误顺序。后两项分别在target_phase=false/true真实free时费用不足，证明这两处修复不是理论改写；wrapper bypass由真正source消费者拒绝控制捕获。恢复后一次明确受影响完整选择60/60，run c7558868-91ad-4f20-bf72-540b5130cd00，44.901s，481过滤/原条件排除：source_resources完整53（原49+4），原AV-core scalar/foreign/paged编码金样与先预留两项，native旧COFF精确输出1、native/C真实首错修复/合同过渡/累计quota拒绝修复/已授权cap首编辑4。所有原条件保留，无新warning（AV原3、native conformance unused-close原1），fmt/diff/cached与提交钩子通过。

正式`cargo t case-ace-ca-d-source-938 --out .../8bo-final-mir-wrapper-formal-certificate`：31 executed=30PASS+唯一原wrong-oracle FAIL，child exit1、outer gate_passed=true；实际45.166s/整命令79.825s，门内未写源码。原schema/2 source/host账本、nonce/worker exe身份、独立oracle与实际磁盘采样保原消费者约束。

范围仍明确：8bm的6个已知物化输出端口现在补MIR wire和startup COFF两个，同一累计账户并未因此覆盖剩余C fragment/assembler、foreign逻辑codec、真实外部C compiler obj/PE等所有生产。C编译输出需与已输入的provider/native object副本区分，不能按整个目录重复计产物；原namespace128与SQL/工具写口、其余metadata/失败清理/原512与六场景30/SDK/Q4/SD/939等仍按原任务完成。T-07/T-08当前闭合状态不变，T-09未勾、不交final review、不缩原门。唯一scope证据merge摘要后全删：8bo-initial-mir-wrapper-quota-check.log=8cd44a4e37f81c08d56a95e38169dd618e5094889933f2ce5f189353c3d3200a; 8bo-mir-wrapper-output-controls.log=326b818d161143ba12e4e845b6dfa46e8af23bc091d9cf7a75515b6311100132; 8bo-four-output-controls.log=0d11fa71dfa3e02e51c5fb5271623bd6e3adc414167e5712ccaf9fa82be650d6; 8bo-final-codec-wrapper-and-consumers.log=d984585bef459219e6fa9b1d1163f96a07cba92e0ae30282dfaf27b72e80af80; 8bo-negative-results.json=d626b9243b34aaceac006db97bfa32033eddd021ab1090e19ee8e8a4a47d7f5f; 8bo-final-mir-wrapper-formal-certificate/results.json=236e51d085982fcb3a91c04dc23bac9a2513d1fc6a147a7c756ab03fbe53ff7a; 8bo-negative-late-mir-admission.log=fb1d1c5ac29ecede5b554b6b16da3f4d9d45a7c01e6c5b799d1827ce38d47b12; 8bo-negative-wrapper-outside-request-account.log=1d7fc795a5ef4d5146746c80ba8fd607669908aa8f0dec975b2fac337692a26e; 8bo-negative-wrapper-refunds-before-bytes.log=6dd1194259b13f382a59b588b2d7418b8fec7e31c0b4541a0359dafb1d574988; 8bo-negative-native-refunds-before-mir-error-data.log=03d0aa2481fdc7a2e8f909d467e24bb224c160c6d3e391025ce58c2c28bb0907; 8bo-negative-target-refunds-before-error-data.log=c9ced117f0f39981117a85178789dc16e97fda966b8eb89b6f07f3605ea2fb40

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: aa7cdc702651d4b49c9cb79a62c8678df3005625 | task_ids: T-09 partial | blockers: none | next_responsible_role: work remaining actual output ports / original namespace owner and metadata cleanup

2026-11-07 work checkpoint 8bp（T-09真实C compiler产物累计准入；仍executing8/13）：提交 `2c2ecba24dfa75ead7cbc332d74aaa1b28befaa6`，9文件607+/17-，唯一树clean。沿8bo基面核实真正外部producer：provider cache miss的compile_c_output输出一个显式obj；source CRT的实际/独立reference编译各输出每个C source对应obj加PE；传入的provider/native/caller/wrapper obj已是输入副本，不重复计产物。cache hit不调用producer，也不新花这项累计额度。

实现复用原Resources与OutputPermit，不新quota DB/进程执行器。新ExternalOutputPermit在实际启动前一次预留当前请求剩余额度，避免给多个未知大小文件各保留64MiB而错误拒绝小任务；单文件64MiB/整请求128MiB仍分别约束，payload不能提升/重置。ctool_output仅持具名输出借用路径与各file高水位，Vec及native path转换scratch在第一次filesystem操作前走原index预算；源CRT新输出PathBuf数组也先独立预留其实际路径所需上界。原捕获器新同步callback在原contained executor既有poll/成功退出处组合输出观察与原owned-scratch观察；不加线程/工具启动，错误仍由原树kill/wait/reap与reader收集处理。结束后无论成功/编译错/timeout/stream cap都再查具名文件并完成许可；失败/删除/截短不把已观察生产退回累计账户，仅未使用pending返还。前一次mutable compiler输出槽先在原准入后清理，避免一次失败误消费旧exe/obj；原accepted source/foreign/target/store页不在这个清理集合。zero admission在清理/启动前拒绝。公开legacy C None入口保持原路径；同一源library无显式请求账户时不把512 revision整段寿命当一个128MiB请求。

实际边界明确：这是启动前预留+原10ms polling+reap后终验，具名文件logical长度高水位，不宣称轮询之间硬filesystem配额或OS物理块计量。Windows linker实测先把PE扩到约2MiB，再截短；本控制两次各terminal109333B/observed2098453B，不能拿最终109KB当全部已观察生产。没有变原deadline、stream/sample/正式预算阈值，也不通过额外compiler dry-run估算大小。原namespace128 all-writer原子性与其SQL/WAL/其他metadata仍待原store owner接线，本完成项不能替代该义务。

永久controls新增5种真实路径（一个private模块同时在source_resources allocator target编译）：①共享账户已有4B，batch余16B，两个artifact合12B但各<=8合法；中途删除仍花12B，失败drop的5B仍花、pending0、拒绝不可重置，另9B单artifact独立拒，防把batch总量错误当单文件。②实际CL先legacy产物取obj长度L；共享quota再编两次每次花L、第三次拒且旧output字节不变，单文件L-1在真实生产后拒、下一新账户可修。③实际CRT编两个C source+既有provider.obj，逐份计新obj/PE、provider输入字节保持、两次PE真返回42；耗尽同一账户后不能启动/清旧exe。④真实Python contained子进程产物存在250ms后删除，两轮仍花32768各次；另持65536B文件睡30s，分别因单file quota和1s deadline终止，pending0/费用回原baseline/真实文件可删除证明原句柄关闭；metadata预算0拒时旧file保持。⑤实际新worker cold→R后首body edit用64KiB host tightening，native/C都必须抵达external artifact/request refusal（不是前置source/helper拒），旧current指针逐字节原样、owned scratch只余owner；下一全新请求repair actual/reference均42且新指针发布。

初试红如实保留：test错误格式化非Debug Captured导致101，仅修测试显示；以terminal PE大小作为artifact fixture帽的初试100暴露真实linker中间2MiB，改为合法4MiB/8MiB tightening并单列terminal/observed，未缩原门；batch finish误复用单artifact检查由独立7+5<=16控制抓到100，修为仅batch累计检查、每file仍在observe独立检查；同次测试误断言Resources本体index0（实际原State232B），改与入场baseline比较，不豁免费用。

六个真实变异均exit100，finally按原字节SHA恢复：provider output绕账户、CRT把input obj列成output导致真实callee链接失败、只终验丢掉中途已删产物、失败drop返还累计生产、忽略单file帽，以及source CRT换另一Resources让实际worker超额仍PASS。最后一项真worker错误实现得到actual/ref42且发布，新consumer控制在error.code实际Null处抓住，证明非helper-only绿。当前所有源码/输入身份恢复，门内无写源码。

验证：最终一次明确影响并集78/78，run071a3fcb-db1f-47f6-9765-aaa61ff92132，64.947s，236过滤/原条件排除；source_resources完整54 + ctool全5 + ctool_output1 + sourceCRT全2 + source_request_work1 + source_tool_scratch全6 +原输出账户2/新batch1 + native/C真实request refusal/repair、首错修复、合同过渡、provider rebind P0、授权cap首编辑/新增external refusal修复6。原MIR两金样/native旧COFF等8bo身份输入未变有效证据复用，不以commit机械全重跑。初新增4control修正后4/4 run585b28d3-7baa-4123-9079-2c0cd16553d5，3.527s；先前独立worker新case1/1 run1304e964-2d0d-467d-8761-6b55b94edf95，5.272s，后已在78并集中再验。没有新warning（原AV3），fmt/diff/cached/precommit通过。

冻结提交正式`cargo t case-ace-ca-d-source-938 --out .../8bp-final-external-output-formal-certificate`：31 executed=30PASS+唯一原wrong-oracle FAIL，child exit1，outer gate_passed=true；actual 47.782s/整命令 81.836s，parent全窗口/worker身份/手写独立reference与既有schema/2报告原样。不能把该31例冒原六场景×30/完整Q4/SDK规模或暖P90完成。

当前任务状态T-01..08已闭，T-09未勾，known output families由8bo八项补provider obj与sourceCRT actual/reference两类（仍不声称全部覆盖）；foreign logical/physical/declaration/binding codecs、C fragment/fileset文本及其工作态/其他动态metadata、namespace128原owner与全失败/512/lease/GC、T-10完整hard-error certificate/SDK、T-11原规模/六场景30/Q4/SD/939与final review/冻结职责并集/merge/archive/全路径清理仍按原约定继续。main已漂至4aa542688，未整合、不触他人docs WIP；原integration pin bdd73416不伪报最新，最终冻结需做真实漂移核验。唯一scope本单元所有日志/变异backup/test roots/正式certificate受保护在途、merge摘要后全删。证据SHA256：8bp-external-output-audit.md=871e64d266c0774220065713d4d9fb9232ee2ea1a260ecda11893845299defb3; 8bp-external-output-initial-check.log=594fd5d87412e3c4f4ad22a338de3ba4edc96c2db653c0dd000adeb52b243c21; 8bp-output-controls.log=40b29aceb1e77514e898c338ef4c82c073b373a8a5cd37670549d06f02c33c50; 8bp-output-controls-rerun.log=3acb50ebf9744c377100cca438d58c76ea005428b4cde4fc5d968cf715b8e719; 8bp-output-controls-final.log=254def9e2ecad7f955bd92a0f0c58832ba3a865d5167bf8393c9f70a11925c73; 8bp-output-controls-corrected.log=65ee28353f6ab766700fed7565f93416e48f8b9b48d4e59971a1c51b5d9b5dda; 8bp-real-worker-output.log=c82c79924596b155dcb4504e779105bbbeaaf2e15e04bd240f90dffd09530f3f; 8bp-final-output-and-consumers.log=94d1c0979c9df9342c2e102762cdc296be5f3974cfc2cdca86a6ce56ef44dc84; 8bp-negative-results.json=6ff319455159989d9ba0f8a744d02c4162a1b284a1b44a6ccaa9db0b7056be25; 8bp-final-external-output-formal-certificate/results.json=d36ff55d7f348d28bb84f9c9aedcb3252f9a1ed2ac8498e8fce1b32e19ca66af; 8bp-negative-provider-bypass.log=a55dfb56a5c17089fb7b8a1bb64c881b830674098a2f0eae3c49f7375f50754c; 8bp-negative-crt-input-as-output.log=81a04e4f80342b987b7223941c90d620d20b6cf5f07ed4572150b2cbcde7c054; 8bp-negative-terminal-only.log=ead4a132b1fd79220f025f31a323b752dd0cfad5325a3bc20e1539c31a6f5381; 8bp-negative-failure-refunds-output.log=9e9cdfa8cff5dd5170f466b1bfbf9efa222d25cc43566aa3b9d12434495acf9f; 8bp-negative-artifact-cap-bypass.log=eafc1793bd937f56090f2402b3b191b5f9180b8bcbd767d8595fee6bd971d009; 8bp-negative-worker-bypass.log=aad7c678f3b0ed0e18d7e0ae6171fd8bfa6bc689c7529b04e43d73d04fcee6e3

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: 2c2ecba24dfa75ead7cbc332d74aaa1b28befaa6 | task_ids: T-09 partial | blockers: none | next_responsible_role: work remaining foreign/C text outputs and original store namespace owner

2026-11-07 work checkpoint 8bq（T-09四个foreign required页与binding-only累计产物；仍executing8/13）：提交 `26f2262f1c49ec36fb55bafb5415c8dc608bae6a`，7文件343+/40-，唯一树clean；前一8bp两个真实external compiler生产族已提交并过门，本单元不重开该实现。

真实source路径：显式worker cold和restore后的first contract edit均从原ManualManifest/admit入口改走Resources.admit_foreign；provider-body-only仍原Resources.rebind_foreign。neutral foreign-contract不依赖core/backend/source-query/store，只给现有producer一个process-local PageProduction reserve/complete回调；Resources的stack adapter持同一请求账户的一张pending OutputPermit，不增加cache/DB/线程/解析器/solver。common encode_json_bounded沿原counting writer/固定容量writer抽出admitted变体，在真实Vec try_reserve前交精确JSON长度；老入口noop复用同一codec，wire/schema/域/人工授权不改。logical/physical/source-declarations/binding四页分别在实际序列化成功后立刻complete，后面页或host qualification失败/候选丢弃不退已完成页。physical源manual页也使用既有MAX_CODEC_BYTES decode帽的bounded writer，legacy937 plan_codec/facts_codec/typed codec入口保持原形；没有扩大或新造支持面。原binding memory预留与last-owner Arc身份仍保持，单独加精确wire累计，不把Vec/Arc头内存当产物字节。provider-only重新编码只产生binding，logical/plan/decl页沿旧Arc共享。

worker资源拒绝通过原budget_error以账户sticky refusal判foreign.resource，actual required页端口附foreign.required-page.output原因；非预算review/authority拒绝保持原分类。restore_pages/required R没有进入新增PageProduction/encoder；原whole-worker observer R无逻辑生产的断言继续通过。SD-01/02/07最终沉淀须记这个host生产账户接缝及保持原format/当前权限，不能把新neutral Rust callback冒公开AutoABI/持久安全凭证。

永久新控制3类：①16384字符合法JSON与原serde字节等价；请求比exact wire少1B时真实allocator>=4096的输出分配0，不是只观察counter；合法exact permit complete正好一份wire。②真实独立审定Add manifest的四required页和旧入口逐字节一致，physical页再与原plan_codec比较、contract仍固定cef4c388...；request总长exact成功，逐一在每页quota差1B拒，produced精确等于此前完整页之和/pending0/粘性拒绝，drop环境不退累计。binding-only两个同大小actual encoder输出各花B、第三拒，逻辑Arc同一、原generation仍1；另一新账户bad host在logical页已完成之后authority拒仍花logical、不错误标quota拒，随后bad-host rebind在生产前拒不多花。③实际新进程worker native/C先Add42，R后第一次切到七函数Scalar mixed合同用host0，必须在foreign.required-page.output拒、旧pointer字节保持且owned scratch清完；下一请求repair actual/reference均30211、新pointer发布，再R compute_calls0并得30211。不能以body quota的后置失败核销这个合同页端口。

四个变异实际exit100且finally按原字节SHA恢复：common JSON Vec先分配再admit由真实allocation0控制抓到；physical页绕policy使真实四页spent少一份；rebind绕policy使实际binding spent0；worker first-contract分支还原原manifest.admit会绕页账户而在后面source路径才失败，新worker控制在required-page原因缺失处拒（不伪称该错误实现一定最终PASS）。没有新正式红/环境豁免/阈值或样本变动。

验证：initial ace-abi check exit0；新增allocator/四页controls2/2 run805c0c94-529c-457b-b6bf-9a9ac4e9978f，0.048s；新真实first-contract worker1/1 run17fd2bd6-d626-4fb3-bb0c-de0e40ca191d，6.688s。四变异恢复后统一实际影响并集133/133，run46431a84-11ba-4eb0-a45d-311d5f38b3bb，67.422s，225过滤/原条件排除：ace-abi conformance7（新required页拒绝/修复/再R加原actual external quota、request quota、首错修复、review transition、provider rebind P0、已授权cap），source_resources完整56，foreign-contract完整lib38+conformance3，source-query foreign/resources/work29。覆盖原literal/strict codec/stateful serializer、来源/权限定界、共有资源fee/alias、精准依赖/恢复首编辑与observer清理；8bp原ctool/CRT/owned-disk模块的独立128/流/deadline证据在实现/配置/输入未变处复用，不为每提交机械重跑。没有新warning（原AV3），fmt/diff/cached/precommit通过。

冻结提交正式`cargo t case-ace-ca-d-source-938 --out .../8bq-final-foreign-page-formal-certificate`：31 executed=30PASS+唯一原wrong-oracle FAIL，child exit1，outer gate_passed=true；actual 50.782s/整命令 117.007s，门内源码不写。原nonce/worker身份/独立reference/协议2工作报告/错误oracle语义不动，required页输出预算增量不替代原六场景×30/Q4/SDK/P90义务。

T-09边界：四required页精确累计准入已接通，但其canonical clone/projection/model/map临时heap等不能仅凭output guard称48MiB全覆盖；managed dynamic metadata仍按原合同继续。下一具名端口见8br audit：source cold的native/C runtime先分配再adopt，C真正cache-miss fragment→原A2xSession/provider→TargetWire、fileset generated/imports/manifest及其owner/ticket交接。须区别已有runtime/provider/object副本与新artifact，不复制C lowerer或旁建quota/cache；runtime R不生产。原namespace128 all-writer/SQL-WAL、其余metadata/失败/512/lease/GC，T-10 hard-error certificate/SDK，T-11原规模/六场景30/Q4/SD/939以及最终独立same-tree review/冻结职责并集/漂移/merge/archive/全路径清理仍未完成。T-07/T-08保持已闭，不勾T-09、不提前交final review。

证据唯一scope owner PLAN-938在途受保护，merge摘要后全部删除（含新的8br探路笔记，无hold）：8bq-foreign-output-initial-check.log=db72f8e0f682c2441d41519a9c33cbeb65f54672f9d9d7a50eb66eeeff376ee7; 8bq-foreign-output-controls.log=65bfe4ef71a216c5564585a3daa2157455a830d0bff00f66532fa97f54b7b437; 8bq-real-contract-edit-output.log=dcccf3ccc80014ae41c0d412c1405ad8c44d22ad64215108fad0123e2dba6146; 8bq-negative-results.json=ebccace67d87bfd3b37eb6ee581c46cb7576057ae37e4f59b8a2d8f1f69f3f92; 8bq-final-pages-and-consumers.log=c38ab68909d7b7459d99e9c6db23bdbb2c23ae60ce25c317abe5c532a3b377fd; 8bq-final-foreign-page-formal-certificate/results.json=0e8b8671b282735a0550417dec7bc9b2ef5169cfd4bab727ad8886b1e8c2cf28; 8br-next-c-text-runtime-audit.md=6b18faa66bb35d728105408a603ff34aeede9e08aebc4c81e49e30c75871c99d; 8bq-negative-late-json-admission.log=406a4eff11710d3c170d2ff0a434e8ab5c387abebf40c36a92e97f337073f64f; 8bq-negative-physical-page-bypass.log=e9dfe99704aced9e6f962b6021c08efa5adbfc562721965ea8974a652306760d; 8bq-negative-rebind-bypass.log=f52ba9ba005259234e036e7448fc048e421656d4c3eeaf0aa51d1a9aa8e906ca; 8bq-negative-worker-contract-bypass.log=909b1d051c8e21de3e68f6d06bf640269114dfbba156c495c6e2d15c836f014e

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | code_commit: 26f2262f1c49ec36fb55bafb5415c8dc608bae6a | task_ids: T-09 partial | blockers: none | next_responsible_role: work source runtime/C text production and original namespace owner

2026-11-07 work checkpoint 8br（T-09 source C text/runtime/fragment/fileset精确生产准入；仍executing8/13）：提交 `25f24d0d2`，11文件1292+/195-，唯一树clean；前一8bq四required页端口保持，本单元不重开其实现。

按8br审计实现三个具名端口，全部先计数、后宿主准入、再单一精确分配：①`a2x-c::runtime`重排为唯一formatter `write_runtime`，legacy `runtime_source`字节不变，新增`runtime_source_admitted`经共享`render_admitted`（零分配counting sink→admit→try_reserve_exact→有界sink；stateful renderer第二遍漂移具名拒）。marks收集的工作元数据以新增`SemanticView::trace_symbol_upper_bound`上界先计index票。`ForeignTargetProvider`可携带`SharedTextProduction`（Arc<Mutex<dyn PageProduction+Send>>，`&self`中立seam下唯一合法内态），`runtime_source_admitted`/`produce_fragments`经其reserve/complete。②`print_fragment`同样重排`write_fragment`（参数直接分隔写出，不再临时Vec/join），C fragment经`FunctionCache::c_fragment`每次调用的新policy生产：exact输出permit+live resident票在分配前，live票经`Artifact::new_reserved(previous)`原子转交为target artifact resident charge（无第二费用/无孤儿票），A2xSession store命中不产生任何输出账目。③`assemble_fileset_admitted`：generated.c/foreign_imports.h/manifest三个真新品逐个exact输出准入（manifest用借字段struct+`encode_json_admitted`，无serde_json Value树；字段序=历史json!插入序，本workspace serde_json保序），runtime header/FAILURE_RUNTIME字节相同副本仅live staging票不入输出账；link_artifacts真实消费并保留原assembly-output IO行。宿主`Session::cold_with_resources(source,..,resources,Option<&ManagedStore>)`为显式请求组合seam（镜像cold/cold_stored，预算先于任何冷生产安装）。a2x-c新增直接serde derive依赖（同一lock内版本）。

永久新控制4项（source_resources，全走真实cold C/Native session或真实provider）：C leg预算=frontend+runtime−1时runtime admission先于分配（layout pressure未触发）、produced恰为frontend；恰=+runtime时首fragment拒且runtime只付一次；native leg authored runtime字节load-bearing（total−182拒、total恰过）；stored路径fresh store上fragment policy在produce内拒（pressure未触发）、second session同store全命中仅新frontend+runtime产品、artifact字节相等；fileset recorder恰[generated,imports,manifest]三次reserve/complete、memory两段覆盖含副本、manifest拒时其分配0次、legacy trait字节与历史json!表达式逐字相等。a2x-c单测3项：runtime admitted==legacy且no-trace材料sha256==SDK pin `c7cf3971...`、fragment admitted==legacy（含原型行形状）、stateful renderer漂移拒。

四个变异负控均真实exit红并finally按字节SHA恢复（1b95bdb9/7b68b432/0ad31ab1/0ad31ab1）：N1还原runtime allocate-then-adopt由C leg pressure+精确账目抓到；N2 c_fragment改`ForeignTargetProvider::new`无policy，账目位置1776≠frontend+runtime红；N3 manifest直接to_string，recorder缺837B红；N4副本也收输出费，[440,173,2659,182,837]≠[440,173,837]红。

验证终态（全部CARGO_INCREMENTAL=0、TEMP/TMP显式938/test-temp、无live session）：a2x-c lib 8/8；abi lib 46/46（一次`real_compile_dependency_admission`因cl输出多一行D9025 warning红，隔离重放绿、全量重跑绿，按环境归因保留记录于8br-abi-lib-final.log）；source_resources 60/60（含4新）；conformance source_foreign 35/35两次（pre/post最终二进制各一次，148-170s，含20次真实双端C调用、SDK runtime pin、正式worker、发布故障/GC矩阵）；rustfmt定向/`git diff --check`/cached check/pre-commit通过；无新增warning（原AV3条在案），删除已无调用者的私有`Artifact::new`。a2x-core/a2x-store/foreign-contract/source-query代码未动，其证据按输入未变复用。

边界：本单元只关C文本三端口的输出/活内存前门；CFn/locals/formatting工作元数据、manifest条目构造、ABC/Cranelift/native lower其余owner、cold接口/查询/HIR/ledger元数据、pointer typed decode后的更大读取、request/namespace并发128、scratch峰、完整T-10 certificate/daily无spawn/SDK/旧桥/export、T-11原6×30/P90/Q4/SD/939与最终冻结职责并集仍开放；旧input-changed gate red保留。8br审计笔记内容已由本单元消费，保留至merge折叠。

同scope证据SHA256：8br-negative-runtime-bypass.log=1f7ff464e83bd62f7649a6c30b65181de25374f46a804c434d52f4b827346a34; 8br-negative-fragment-bypass.log=805510282b32f0bb0850d1e4080cfba8f7ac95a2dc5b416581748b0091c158a7; 8br-negative-manifest-bypass.log=3540d35ad6aec874738c464349cc9f9d49dfd0dddcc701337df78a74f738db22; 8br-negative-copies-charged.log=92ca87e32feccdd69d811458ee1afe443e7435f743314d5355d928d6edb6c27c; 8br-new-controls-final.log=00a37771599f15c694e4c6c7a10c6256f28fc8c1337c0e74760ea3180862d5e8; 8br-source-resources-final.log=e0b6b8cf23db0923d17ef17241bece96ff44e06281a7311b9fb4cf16897f02e8; 8br-abi-lib-final.log=d6c89b8948a308fd8ccde02463f01368fb44e57b0245129f1a4232d00f91b95d; 8br-conformance-final.log=3644598e3c304b7df31afa23c691e26384da899539371c50f8150f908c01165c

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | code_commit: 25f24d0d2 | task_ids: T-09 partial (source C text/runtime/fragment/fileset production admission) | blockers: none | next_responsible_role: work remaining managed owners then namespace128/request writers, T-10 certificate/SDK/readers, T-11 original workloads

2026-11-07 work checkpoint 8bs（T-09 cold接口生产工作集先准入；仍executing8/13）：提交 `32e76a6f5`，3文件187+/1-，唯一树clean；8br三文本端口保持，不重开。

关闭8c/8bq两次点名的cold接口内存缺口：`ManualManifest::admit_produced`曾对PhysicalPlan::project、声明表展开（含每声明克隆整份capabilities）、digest map、binding页（内部memory闭包为noop）零准入，且`Resources::admit_foreign`对四个页Arc/声明表/digest map零retain。现neutral新增`cold_interface_memory_bound`（零分配推导：canonical clone+facts重解析3×wire+64KiB、plan模型逐函数界、声明模型+digest map按manifest逐串精确界（effects过滤谓词与投影一致）、四页resident+Arc头+binding固定界，checked求和溢出具名拒）与`admit_produced_with_memory`（生产前一次性memory(bound)）；host `admit_foreign`先reserve `Charge::resident(bound)`（foreign.resource.interface分类）、生产后经既有`retain_foreign_metadata`挂到`allocation_identity`弱身份——charge随环境最后owner释放，与restore路径的decode共存票同模式。legacy `admit`/`admit_produced`保noop行为（自持预算调用者不变）；worker cold与restore后first contract edit自动经新端口。

永久新控制`cold_interface_admission_reserves_before_production_and_retains_until_release`（真实fixture manifest+真authority）：成功路径真实allocator观测工作峰9715B≤预留bound且charge覆盖四页+声明模型heap、drop环境后resident回基线；拒绝路径（比bound少1B）foreign.resource拒、preflight自身Display-adapter小字符串≤1KiB允许（serde collect_str工作元数据，已注明）而**输出账户produced/pending均0**（内存先于任何页生产拒绝）、释放后同manifest修复成功。

四个变异负控真实红并按字节SHA恢复（93623d06/c71390d3×2）：N1 host还原plain admit_produced→charge覆盖断言红；N2 memory调用移到canonical+plan之后→输出账目produced>0红（页先于内存拒绝生产）；N3不retain直接drop ticket→charge在生产后即回落红；N4 bound恒64B低估→真实工作峰9715>64红。N4初版"仅去掉pages分量"对该小fixture不构成真实低估（64KiB canonical余量仍覆盖），按实不当作负控证据，改用真低估变异；另一次变异脚本残留占位符编译错误当即废弃重做，两事件均如实保留在日志。

验证终态（CARGO_INCREMENTAL=0、TEMP/TMP显式938/test-temp、无live session）：foreign-contract lib 38/38、source-query lib 86/86、abi lib 46/46、source_resources 61/61（含新控）、conformance source_foreign 35/35（最终二进制重放167.9s，含20次真实双端C调用、正式worker cold经新admit端口、发布故障/GC矩阵）；rustfmt定向/diff/cached diff --check/pre-commit通过；无新增warning（原AV3条在案）。

边界：该端口只关cold接口生产的工作/常驻集；preflight serde Display-adapter小字符串（≤wire量级）为已登记工作元数据未逐字节准入；restore路径decode票、rebind binding票维持原各自模式；其余owner（ABC/Cranelift工作区余量、查询/HIR/ledger元数据、pointer decode后读取、并发namespace128含SQLite/工具写口、scratch峰）与T-10/T-11、最终冻结并集/独立复审/merge清理保持开放；旧input-changed gate red保留。

同scope证据SHA256：8bs-negative-host-bypass.log=158871c7647eed527e3a01c266098bd2e984f5379059300bc4cb41f04396782b; 8bs-negative-late-admission.log=36cf44420f3cf186f96765226a2857133bb03e6be99d610fc613e13d607e2cf5; 8bs-negative-no-retain.log=7656c11838f8e2dacc9df16251963d71cc06eaf3e62a41e28075f63cdb407c38; 8bs-negative-bound-underestimate.log=59081a6f4c5a575489776ef37e79e172201c011fa2951f9257451822b0e60dda; 8bs-source-resources-final.log=2cdfa8c1d957aa6ba50d57922fa7d8dc3606858821dc7ca48ee4a23825f42ce2; 8bs-conformance-final.log=9e0abb532196db4379640c7bd63fc2021df662e934698a0b304e2381b7fb1270

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | code_commit: 32e76a6f5 | task_ids: T-09 partial (cold interface working-set admission) | blockers: none | next_responsible_role: work remaining managed owners (query/HIR/ledger metadata, ABC/Cranelift remainder), then concurrent namespace128 and scratch peak

2026-11-07 work checkpoint 8bt（T-09 Frontend会话元数据一次性准入；仍executing8/13）：提交 `fe60ccf0a`，2文件157+/0-，唯一树clean；8bs端口保持。

审计确认：Frontend的work ledger逐owner/stage行（含warm edit新行）、per_function meter图、residents图与SourceQuerySession的sigs/bodies应答缓存在构造与编辑期间零准入。现`admit_session_metadata`（cold在items排序后、restore在edges循环前）按目录函数名推导一次index票：8KiB bootstrap（整模块行/表头/ticket向量/restore负依赖边）+每owner（4个map键+节点、Meter/Resident值、跨编辑至多14个ledger行的Totals+节点、1KiB查询页余量），`_metadata` Reservation随Frontend持有——编辑在票内增长，无逐编辑充值；构造失败原`Error::Resource`分类。

永久控制`frontend_session_metadata_is_admitted_once_and_covers_real_row_growth`（真实fixture）：Frontend-only冷构造测得整个前端index消费（本准入+构造期既有消费者；`_checked`的长存份额由release侧归因）；断言①charge≥文档化8KiB bootstrap+真实cold行字节（rows()精确归因：owner+节点+Totals）、②drop后释放≥8KiB（随Frontend释放）、③真实session一次充值后`prepare`暖编辑index零增长、④编辑后真实行字节仍≤charge、⑤index余量耗尽时构造以foreign.resource拒、释放后同输入修复成功。

四个变异负控真实红并按字节SHA恢复（67a9a2c6）：N1两端点零charge→①红；N2 bound恒64→①红；N3b每编辑持久8KiB泄漏充值→③红（首版N3票随prepare返回释放、无净值变化，按实非可观察错误形态不作证据，改为会话持有形态）；N4构造尾drop真票换0票→①/②红。N2与N3各有一次变异脚本语法/基线错误当即废弃重做（编译错误/误对pristine跑出PASS），均如实记录。

验证终态（CARGO_INCREMENTAL=0、TEMP/TMP显式938/test-temp、无live session）：source-query lib 86/86、foreign-contract lib 38/38、abi lib 46/46、source_resources 62/62（含新控）、conformance source_foreign 35/35（171.9s，含真实进程restore路径经新restore端点）；rustfmt定向/diff/cached diff --check/pre-commit通过；无新增warning（原AV3条在案）。

边界：本端口关Frontend会话元数据；构造期其他既有index消费者（parser/目录/查询会话内部）仍按原owner开放，控制采用总量+文档化下限锚点如实区分；solver/HIR/ABC/Cranelift工作区、pointer decode后续读取、并发namespace128（SQLite/工具写口）、scratch峰与T-10/T-11、最终冻结并集/独立复审/merge清理保持开放；旧input-changed gate red保留。

同scope证据SHA256：8bt-negative-no-admission.log=7e55f3573cae54a2b018406d27559988308e209b7200f7f700e860d54fb835c0; 8bt-negative-underbound.log=fb0666465e27edf89830774903e99136f9631ce0a269aa70c20ae6a598eb9b3a; 8bt-negative-per-edit-recharge.log=64ac684b06f7100fbb0668c5327bb4a7a3f4edccaed1064480f376ea3de7bd94; 8bt-negative-dropped-ticket.log=f457417d800c6d7018ff0698a37ac7722245ef5335423ed16ec59942f95c362e; 8bt-source-resources-final.log=fc8422dd143267a90a97168c783cceeb3825f4827252229218992c4f11f2f1a5; 8bt-conformance-final.log=d94f49326ad487279ba90a9010a340da016fbb25382356ce29f2a06da541fcf1

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | code_commit: fe60ccf0a | task_ids: T-09 partial (frontend session metadata admission) | blockers: none | next_responsible_role: work ABC/Cranelift workspace remainder and pointer decode follow-on reads, then concurrent namespace128 and scratch peak

2026-11-07 work checkpoint 8bu（T-09 Cranelift emit工作区共享准入；仍executing8/13）：提交 `adf4abdad`，3文件145+/3-，唯一树clean；8bt端口保持。

实测确认缺口：真实source单元的native emit窗口（ISA/ObjectModule/CLIF context/codegen/COFF写）allocator峰52713B，而窗口内staging准入仅257B（8p分析票随admit返回释放、8j COFF票只盖输出）——约52KB后端工作区未准入。现`codegen_workspace_bound`（96KiB固定界=实测最小单元53KiB×~1.8余量，逐元素：instr/block 1KiB、import 2KiB、type/const 256B、slot 64B，checked溢出拒；cranelift版本钉在emitter identity，升版漂移由控制峰比对抓）在`make_isa`前先reserve `Charge::staging`，窗口票与COFF输出票`merge`为同一账户条目随返回值交到调用方cache交接——无释放再申请间隙；legacy无resources路径行为不变。

永久控制`native_codegen_workspace_is_admitted_across_the_cranelift_emit_window`（真实session wire解码的真单元）：成功路径持合并票时admitted staging≥真实allocator窗口峰52713、drop后staging归零；拒绝路径留64KiB（低于固定界）→codegen-memory/foreign.resource拒、拒绝调用峰≤8KiB（仅分析工作区先分配）、释放后修复成功。

四个变异负控真实红并按字节SHA恢复（a2dff282）：N1无workspace准入→emit不拒/覆盖红；N2 bound恒64→"peak 52713 exceeded admitted staging 7937"红；N3准入移到define_function后→"backend allocations preceded the workspace admission (52713)"红；N4 workspace票drop不merge→"exceeded admitted staging 257"红。

既有8j控制`native_output_reservation_is_live_until_cache_handoff`按合并票语义更新：返回票额=workspace界+COFF精确输出、drop归零不变；其后半段紧凑staging余量按新workspace准入加宽（96KiB+8KiB slack），拒绝点仍钉在原`foreign.mir-encode`/CodecPage preflight、native_produced=1、修复43不变。全切片一次publication_faults并行红，隔离重放绿按环境归因保留，随后全切片干净重放35/35。

验证终态（CARGO_INCREMENTAL=0、TEMP/TMP显式938/test-temp、无live session）：native lib 12/12、abi lib 46/46（一次cl D9025环境红重跑绿，沿用8br记录口径）、source_resources 63/63（含新控）、conformance source_foreign 35/35（172.6s最终重放）；rustfmt定向/diff/cached diff --check/pre-commit通过；无新增warning（原AV3条在案）。

边界：该端口只关native emit窗口的后端工作区；`emit_wrapper_coff_impl`（startup wrapper，同构Cranelift窗口）与entry-wrapper的既有输出计费并存——wrapper工作区仍按原owner开放（下一候选）；ABC encode/decode已由8k/8h覆盖其输出/解码前门，其余余量（诊断字符串、sidecar写路径细分）未重开；pointer decode后续读取、并发namespace128（SQLite/工具写口）、scratch峰、T-10/T-11、最终冻结并集/独立复审/merge清理保持开放；旧input-changed gate red保留。

同scope证据SHA256：8bu-negative-no-workspace.log=034299751f3e499d25b6efd757c1ec804a6cfc6351fd1ede78623cb744f56781; 8bu-negative-underbound.log=de420b4a5680d39f06df211df223fc0278f691eee69ad7e45c302842af86eb0f; 8bu-negative-late-admission.log=50a9cc66f8fac1933d8f82d9f9816b290882b657a6305e0b24f885363cd0d1f1; 8bu-negative-no-merge.log=6e28217378dc278a7f1b7bfea65afc7753bf0fb4487bb7d4b6eab1e06c666321; 8bu-source-resources-final.log=27f2ca84a2966efd90f947976df3d4f0823345b4cf322443bfc964682e3b7f12; 8bu-conformance-final.log=582e6753f48dd7e52e704325990ad05cb87c51d9a0414cb07013aba83d1329e0; 8bu-8j-control-updated.log=a6e31544ace1bff5d8f65d04bb05297b089c053c21113519f3bc681bb0ba1f06

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | code_commit: adf4abdad | task_ids: T-09 partial (cranelift emit workspace admission) | blockers: none | next_responsible_role: work wrapper emit workspace + pointer decode follow-on reads, then concurrent namespace128 and scratch peak

2026-11-07 work checkpoint 8bv（T-09 startup-wrapper Cranelift工作区准入；仍executing8/13）：提交 `8bd0a610a`，2文件88+/3-，唯一树clean；8bu端口保持。

8bu同型收口wrapper缺口：`emit_wrapper_coff_impl`（每请求真实发射、不缓存）的ISA/ObjectModule/CLIF context/codegen窗口现按`WRAPPER_WORKSPACE_BOUND`（96KiB固定界+8KiB slack，与unit emit同基线）在`make_isa`前reserve staging，窗口票与wrapper输出票`merge`为一条账户随`ReservedWrapper`持有到字节最后使用（含文件写失败路径，原8bo语义不变）；legacy无resources路径行为不变；原request-output拒绝顺序不变（workspace先入、输出permit仍由coff_buffer前门拒）。

既有控制`entry_wrapper_output_has_the_original_bytes_and_keeps_its_fee_until_real_deallocation`按合并票语义更新（staging≥size+96KiB；free-category探针与drop归零、request-output拒绝断言不变）。新永久控制`native_wrapper_workspace_is_admitted_across_the_emit_window`：成功路径持票admitted staging≥真实allocator窗口峰、drop归零；拒绝路径64KiB余量→codegen-memory拒、峰≤4096（backend零分配）、释放修复。四个变异负控真实红并按字节SHA恢复（79ed93e6）：N1无准入/N2 bound 64/N4不合并在覆盖断言红，N3准入移到define后（backend先分配）在拒绝峰断言红；控制文件两次插入/导入脚本失误（重复锚点块、漏Charge导入、unused mut）当即修复，如实记录。

验证终态（CARGO_INCREMENTAL=0、TEMP/TMP显式938/test-temp、无live session）：native lib 12/12、abi lib 46/46、source_resources 64/64（含新控+更新控）、conformance source_foreign 35/35（177.0s，真实publication路径消费合并票）；rustfmt定向/diff/cached diff --check/pre-commit通过；无新增warning（原AV3条在案）。

边界：native侧两个Cranelift窗口（unit+wrapper）均已先准入并合并输出票；ABC encode/decode由8k/8h覆盖输出/解码前门；剩余T-09 owner：pointer decode后续读取、并发namespace128（SQLite/工具写口）、scratch峰、solver/HIR余量与构造期既有index消费者；T-10/T-11、最终冻结并集/独立复审/merge清理保持开放；旧input-changed gate red保留。

同scope证据SHA256：8bv-negative-no-workspace.log=d3143bfee3a909d736f3445e3e3fd5b1ab5c9e84fba26c960c4b72757c75fe41; 8bv-negative-underbound.log=6b0e43717b4ffbc0d2fedd5c7bc0088fcca72644c5c3766ef9e9fbc769c22163; 8bv-negative-late-admission.log=efc0ed172a5e2cf01651d22e402a8621b4f4108eabf2f9016b5c532dddb5a8f3; 8bv-negative-no-merge.log=c01e59abb5a04f171239963dd032bca83834808f36980b8f5ffec22938f691ff; 8bv-source-resources-final.log=9df2b5c876b02e7aadfb249d12dd96b643afcf229d1f12da5b7967026a45be7e; 8bv-conformance-final.log=a00bf5f4009a4cbd4770fa9a914866bf1e797af39c91ff419f68c323b06e9f71

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | code_commit: 8bd0a610a | task_ids: T-09 partial (startup wrapper emit workspace admission) | blockers: none | next_responsible_role: work pointer decode follow-on reads, then concurrent namespace128 and scratch peak

2026-11-07 work checkpoint 8bw（T-09 pointer读取审计+整namespace提交准入；仍executing8/13）：提交 `b8987bb0b`，3文件205+/0-，唯一树clean；8bv端口保持。

先按next清单审计pointer decode后续读取：worker重启链（pointer→typed Publication→Stored→restore_stored）全部经8r/8t已准入端口（read_pointer_bytes_reserved/decode metadata profile/load_payload/decode_json），publication_root_project仅路径派生、read_pointer_identity流式（8r）、raw read_pointer_bytes在abi内无调用者——无真实缺口，按审计记录不造端口。scratch_peak已由8bl经host_work::scratch_peak真实接线（worker.rs/worker_bridge各自独立），亦无缺口。故本单元落在8n点名的并发namespace128：原put_inner的staging-budget排他锁在CAS后释放、metadata commit前无任何namespace级准入，调用侧space+预测检查无法证明并发不越帽。

实现：`NAMESPACE_BUDGET`=128MiB冻结上限，`open_with_namespace_budget`仅可收紧（放宽具名拒）；put_inner在staging预算准入后、任何frame/staging写入前取`namespace-budget`排他文件锁，读`space().total`并投影now+framed(新块)+framed+4096(元数据余量)超帽具名ResourceExhausted拒，锁持有横跨staging写→CAS→metadata commit到函数尾。锁序liveness(shared)→budget(excl,中段释放)→namespace(excl)→chunk→sqlite；GC/recovery持liveness(exclusive)不取budget/namespace，无环。幂等重投按保守投影计（瞬态副本真实存在）。调用侧原128MiB预测保留为纵深。

永久控制`namespace_budget_admits_before_writes_and_spans_the_metadata_commit`（artifacts conformance新case）：收紧帽拒放宽；顺序64KiB填至具名namespace拒绝（≥4次成功、blocks≤cap、无staging残留、拒后小put仍收）；并发段预填至恰容1个300KiB块（余量96KiB吸收WAL噪声），2线程Barrier同时发布——断言恰1成功1具名拒且blocks≤cap（锁横跨commit使后者看到前者提交）。三连跑稳定。

负控：N1无准入红（帽永不触发）、N2投影忽略增长红（顺序段越帽）、N3'宿主seam允许放宽红。如实记录两个不可观察变异：锁前陈旧读与"准入后即释放锁"均不红——原staging-budget排他锁已横跨staging写→CAS放置，而space()按FS计已放置块，故后到者必然看到先行者的块增长；commit横跨锁的边际覆盖仅为commit期间WAL增长（小且无法确定性触发），不据此宣称其已被控制证明，按保守正确性保留。

验证终态（CARGO_INCREMENTAL=0、TEMP/TMP显式938/test-temp、无live session）：ace-artifacts lib+tests 47/47（含LayoutStore ≥128MiB负载——独立layout存储不经put_artifact，与帽无冲突）；a2x-store lib 1/1、abi lib 46/46、source_resources 64/64、conformance source_foreign 35/35（212.3s，512修订churn与32次修订GC项照常，逐put的space扫描开销如实记录）；rustfmt定向/diff/cached diff --check/pre-commit通过；无新增warning（原AV3条在案）。

边界：namespace准入按space()文件系统扫描在锁内读取（大库下为性能开口，正确性不受影响，未来可换索引）；工具目录（logs/source938-tools）已在store.space口径内但各自owned lifecycle由8q/8s负责；SQLite WAL超4096余量的极端增长未单测；solver/HIR余量、构造期既有index消费者仍开放；T-10/T-11、最终冻结并集/独立复审/merge清理保持开放；旧input-changed gate red保留。

同scope证据SHA256：8bw-negative-no-admission.log=c4add3f0cb9fddc2ca4abd2f56e04f2c2d6a15e12fbd15522ac80f8cf231d78b; 8bw-negative-ignore-growth.log=8621f7ec22304fd9494098f66e8330b12010b79f39263aa0397cd4217b137557; 8bw-negative-stale-read.log=b13c07cd1749f0df9677f0ece7a8b90729a7b9ced7b9838cdc759bfec1a5db30; 8bw-negative-lock-not-spanning.log=71a3357954eebfe0908f45d781055afc5dbac5bee901c1a99db0627df5e72965; 8bw-negative-widening-allowed.log=ff5f9703813a7268de7903781236eb9e72c9b7d0bbc040e081d87bc8b4b2cf68; 8bw-artifacts-final.log=0f423b29cb198bfbf2921ceecf5d917f38b62b0814a343ff018e75feeebd6e04; 8bw-source-resources-final.log=72668172c29cace2ddfbf0805a30c8f85d87d7e14848ec78d81b0b60bea3050b; 8bw-conformance-final.log=44b86d6c23f3852ec87bb4d7413d47df5bb194a6959e59b5e64d5c489dd7e288

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | code_commit: b8987bb0b | task_ids: T-09 partial (whole-namespace commit admission; pointer-decode reads audited admitted) | blockers: none | next_responsible_role: work solver/HIR remainder and construction-time index consumers, then T-09 closure assessment before T-10

2026-11-07 work checkpoint 8bx（T-09 solver工作集准入 + T-09收口评估；current_step 9/13）：提交 `bde1b7952`，4文件209+/3-，唯一树clean；8bw端口保持。

审计：每次body查询的`InferSession`（scopes/vars/eq_pairs/bindings/expr_types/diags逐节点家族）与本层span cursor/typed映射在共享账户零准入；parser scratch沿原`FrontBudget::WORK_OWNED_64MIB`独立冻结预算（原更严帽，非共享账户，按设计保留）；构造期其余index消费者（parser/目录/查询内部）经核为**已经**通过账户reserve的既有charge（8bt测得alive 15792=本准入10.4K+既有5.1K reserved），非未准入缺口。

实现：`solver_workspace_bound(nodes, externals)`=8KiB+512B/节点+256B/外部签名（节点数来自walk、受原4096节点帽）；`produce_infer_body_with_catalog_observed`新增`resources`参数在`InferSession::new`前reserve staging、持有至facts返回（含映射）；`SourceQuerySession.with_resources`接线，Frontend cold与restore构造均传同一账户；pub`produce_infer_body_with_catalog_admitted`供宿主/控制；legacy入口None不变。

永久控制`solver_workspace_is_admitted_before_the_session_and_covers_the_real_peak`：真实Program直接admitted查询——allocator整查询峰≤bound、solver attempted=1、票随返回归零；staging余量4KiB拒为ProduceError::Resource且solver attempted=0（先于会话创建）、释放后修复；frontend级：C冷会话（唯一staging消费者即solver票）在4KiB余量下foreign.resource拒、释放后成功且solver attempted≥1、staging归零。四个变异负控真实红并按字节SHA恢复（produce ba96b360/frontend 405077dd）：N1无准入、N2 bound 64、N3准入移到infer之后（"solver ran before its workspace admission"）、N4 Frontend会话未接线（frontend级expect红）。

验证终态（CARGO_INCREMENTAL=0、TEMP/TMP显式938/test-temp、无live session）：source-query lib 86/86、source_resources 65/65（含新控）、conformance source_foreign 35/35（218.1s）；abi lib并发全量三次`real_compile_dependency_admission`红（"lossy diagnostic text changed"：同库`ctool.rs:520`/`source_crt.rs:127`在兄弟测试中改进程级`AUTOABI_MSVC_CL`/批处理env，并发线程下其中一次真实cl看到额外`/W0`→D9025行差异），隔离两次绿、`--test-threads=1`全量46/46绿——归因为既有测试隔离竞争（本计划未改ctool/env代码），红日志与串行日志均保留，**登记为T-12复审债务候选**（env变更测试应共享互斥串行）。rustfmt定向/diff/cached diff --check/pre-commit通过；无新增warning（原AV3条在案）。

**T-09收口评估（对照§5.5/V-06/V-07 T-09部分，逐owner）**：
- 先reserve：codec/JSON(8b)、facts二进制页(8b)、ABC decode/encode(8h/8k)、MIR decoder共享(8h)、Verifier init/scalar type(8p/8z)、native COFF(8j)、Cranelift unit/wrapper工作区(8bu/8bv)、native lower常量池/eval_args/模块元数据(8bc/8bd)、provider符号索引与输入(8x)、binding rebind(8y)、cold接口工作集(8bs)、四required页输出(8bq)、C runtime/fragment/fileset文本(8br)、pointer encode/read/identity与typed decode(8r/8t)、request deck(8i)、Frontend会话元数据(8bt)、solver工作集(8bx)、外部编译器产物累计(8bp)、MIR/entry-wrapper累计(8bo)、target cache put副本(8n)、store frame前门(8n)、整namespace提交(8bw)——全部有永久正控+变异负控。
- 取消/超限/迟到/撤权：8c/8d/8e发布故障矩阵、请求累计128MiB sticky拒(8bm)、单产物64MiB、原4/8MiB source帽(8i)。
- lease/GC/512修订：leased put+root lease(8d)、staging/pointer临时件与crash回收(8e)、hot collector 8→6MiB与原512 churn(8m)、owned工具scratch生命周期与崩溃回收(8q/8s)。
- 全部失败清理：每端口失败路径票据RAII退还、staging残留核对（本轮namespace控制亦断言拒后无staging）。
- 残余（明示、不冒）：parser scratch归原FrontBudget owner；SQLite WAL超4096余量极端增长未单测；manifest条目/CFn/诊断字符串等小工作元数据按wire量级登记未逐字节；namespace准入按space()扫描（性能开口）；8bw两个并发变异不可观察（已记）；P90≤2000ms与原6×30样本属V-07 T-11部分、8bj实测暖P90 2457/2296ms红保留——**AC-06不随T-09勾选而关闭**。
据此勾T-09（预算/取消/超限/lease/512GC/失败清理各项均有可重放证据与具名残余），current_step 9/13；T-10/T-11/T-12/T-13保持开放。

同scope证据SHA256：8bx-negative-no-admission.log=21fe6437342b110415ced730a727aff89f24a4f01c5898c5417b6a4b07536d43; 8bx-negative-underbound.log=361a0907983bb1a708c6b86386031a511b6c096129271d0a577d7a014a205d14; 8bx-negative-late-admission.log=3913fcd300e0eb4ca2c67c418b8b7f1c203c8945f71875298ffd61f031f627f8; 8bx-negative-session-not-wired.log=f6848bc98fc1290cbefa30304dac05dfd02f312e27dcb93ce79127fa7ee9a2f4; 8bx-source-resources-final.log=3c338880c7d615fbd5d1076d1d6c446380ccd44fede1999a1c2bb9932ac68983; 8bx-abi-lib-final.log=cddeaf1ae43dd96e6cc4bc70b02fecb69230a11b2b9aa4839e9853c81cb134b5; 8bx-abi-lib-serial.log=1780b04289e6e65999358fcc176fde5e7730197e3e15d115291bd3782a975b0c; 8bx-conformance-final.log=114290aabf5a203ac7586cebcf49aac0c6d9be92c3f9ef14d3cf68d82a77dff5

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | code_commit: bde1b7952 | task_ids: T-09 complete (residuals named; AC-06 P90/6x30 part stays with T-11) | blockers: none | next_responsible_role: work T-10 full certificate/daily no-spawn/SDK readers/legacy bridges/937 export

2026-11-07 work checkpoint 8by（T-10回归腿+正式门重跑+6×30归因；current_step 10/13）：无代码变更（树clean，code仍bde1b7952），本单元为最终代码上的T-10证据收口。

回归腿1（固定组`ace-abi ace-foreign-contract ace-a2c-abi`，nextest进程隔离即正式入口）：ace-abi **239/240**——94条legacy用例（797三腿/937桥/typed/collections cache等）全绿，唯一红为`scalar_bridge::bridge_turnaround_six_scenarios_30_samples`（6×30原正式样本，body-edit P90 2306ms>2000）。ace-foreign-contract、ace-a2c-abi（937 abi_scalar export面）PASS。

6×30红归因（三重证据）：①禁用8bw namespace准入重跑同测P90 2358ms同带；②**主检出master基线（fd3b3116a，无任何938代码）同测P90 2354ms同红**；③现码两跑2306/2431。四次同带，最小样本1.3-1.7s全由每次cl调用的vcvars启动主导（8bj实测vcvars≈1.5s、直接cl≈180ms）——**本机环境独立于PLAN-938即红**，与8bj保留的源认证暖P90红同根因。红保留、不改阈值/样本/独立oracle，处置决策（如进程内SDK环境解析复用——8bj已测直接cl≈180ms可达、但其明确未作为交付）留T-11与用户裁定。

回归腿2（固定组`ace-a2x ace-native ace-artifacts`）：ace-a2x PASS（197）、ace-artifacts PASS（47）；ace-native组runner两跑均30分钟deadline截断（exit124，非用例失败；首跑含冷编译）——已执行203/203全绿，剩余23条直接nextest补跑19+3(+1重号)全绿，**ace-native 226/226全绿**（组截止截断如实记录为吞吐/环境，非代码红）。

正式门：`cargo t case-ace-ca-d-source-938`在最终代码（bde1b7952树）重跑 **PASS**（gate_passed=true，31项=30正常+1预登记wrong-oracle FAIL同8bq语义）。SDK读者核验：root `auto-lang-core/src/test_cases/abi_source.rs`读`abi.source-import-report/2`且/1具名拒、worker产/2、conformance断言/2——producer/reader镜像一致。daily无spawn观察用例（8bi）在册且随每次切片重放。

**T-10收口**：worker/父decoder（8v/8bi协议与父判定控制+本轮35切片重放）、SDK镜像（本轮核验）、新固定组（正式门本轮PASS）、首例完整元数据与独立认证（8bh Scalar矩阵+Add共11 deck，8bq与本轮双跑）、旧三腿/937 export回归（腿1全绿，除归属T-11的6×30性能门）、共享消费者（a2x/native/artifacts全绿）——各项均有最终代码证据，勾T-10；6×30/P90与AC-06关闭显式留T-11（带本轮三重归因）。AC-07的"原797三腿回归"按功能腿全绿主张，性能门不在此AC内。

同scope证据SHA256：8by-legacy-abi-legs/results.json=f4ac5ffb1842f9fd25f143d90d8149725983971d304eafff3da305653585eba4; 8by-6x30-without-namespace-admission.log=8c04a390520eb20eab75c7603b5897aa908c8b165085ff52518168d1e22040f4; 8by-6x30-without-namespace-summary.txt=284c0b30275eff4a022b97662f5d3b66e5c571f06713fe1afa12c007b45979fa; 8by-6x30-master-baseline.log=309565892c850b91a9af284d0970317c7272179e7a4256f1dc20dec2917a8f92; 8by-6x30-master-baseline-summary.txt=3c5c563c1ae0c148f13dfc22ea329be8d3d146883a0b35a3d5fa45c28ddc7ba8; 8by-shared-consumers/results.json=070acb67d5567cc8ab7972fb16a4897b5ab811083e792dff4020d76ab3bfb113; 8by-ace-native-rerun/results.json=5da5d48528ac93ed03091c2e1b2ab1614ff78e13988f2920701f55870293b58d; 8by-native-remaining.log=ab0a1ebdb87ad5f9f5e72f9db2bfeacc51d169d02967df6acc983071b36aba94; 8by-native-remaining-2.log=4869f35fdfa77e5348961a8796fe4b5dae22af83c5d714e856495c31b6e8d713; 8by-formal-gate/results.json=3faa97946f1efe55772bd1eef0f2bde7cd2c484ddd7879e7b513d12eb3cf259d

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | code_commit: bde1b7952 | task_ids: T-10 complete (6x30/P90 + AC-06 closure stay with T-11 with triple attribution) | blockers: P90<=2000ms unreachable on this machine without toolchain-env reuse decision (user) | next_responsible_role: T-11 original samples/P90 decision with user, Q4/SD/939 handoff; then T-12 independent review

2026-11-07 work checkpoint 8bz（T-11 P90门修复：SDK闭包进程级解析复用；用户裁定2026-11-07）：提交 `0e42ff761`，3文件293+/130-，唯一树clean。

按用户裁定实施环境解析复用（推翻8bj"不作交付"仅在用户明示范围内）：`Toolchain::resolve`的闭包解析（cl发现/内容哈希/平台查询/SDK布局）改为进程级缓存，键=(A2C_VCVARS64, AC741_SDK_UM_LIB, cl路径)，失败不缓存、≤8键FIFO；每请求仍按存储尺寸reserve/释放自己的元数据index票（账目不变），identity/fingerprint语义与材料完全不变；显式batch override仍走Batch原语义。legacy bridge腿（无共享账户）改走同一解析闭包（本地默认账户记账），删除3000字节逐编译`cmd /C call vcvars64.bat && cl`批处理——冻结flags/闭包/输出不变。计数器（RESOLUTIONS_DONE/SDK_DIRECT_COMPILES）供控制观测。

**P90门双双转绿**：797 6×30 body-edit P90 **375ms**（原2306-2431ms，6×改善；整测233s→46.5s，全六场景min 129-345ms）；新增永久`source_foreign_warm_body_edit_p90_gate`（两腿各30真实prepare→commit→run样本，P90≤2000ms原门不动）PASS 14.9s——8bj的临时探针红转为常驻门绿。

控制与负控：正控`crt_resolution_is_reused_across_requests_and_recharges_each_request`（第二次请求零新解析、identity相同、每请求charge相等且drop归零）与`legacy_bridge_leg_compiles_through_the_resolved_direct_closure`（SDK直调计数+1、真PE 42）；四个变异真实红并按字节SHA恢复：N1缓存永不命中（"re-ran the closure resolution"）、N4缓存路径零充值、N2 legacy腿改逐编译batch（直调计数不动）、N3缓存无视override键（既有direct_sdk测试的"invalid explicit SDK silently fell back"红——也证明该既有测试是override语义的有效检测器）。模块内env敏感测试加共享ENV_LOCK（nextest进程隔离不受影响，plain libtest并发竞态在本模块消除——部分收窄8bx登记的债务候选）。

验证终态（CARGO_INCREMENTAL=0、TEMP/TMP显式938/test-temp、无live session）：source_crt模块4/4、abi lib并行全量**48/48**（原env竞态红在该包消失）、source_resources 65/65、conformance最终重放**37/37**（35源切片+6×30+新暖门，192.0s）；rustfmt定向/diff/cached diff --check/pre-commit通过；无新增warning（原AV3条在案）。

T-11剩余：Q4原required-oracle矩阵核对、SD沉淀核对、939交接；6×30/P90与AC-06关闭条件现已满足（两门绿+归因记录），待T-11其余项完成后一并主张。

同scope证据SHA256：8bz-6x30-with-reuse.log=ac7467248a56923aab0db1db6fd5a345cdd421a57cbc864e1b0521f505f4e864; 8bz-6x30-summary.txt=742665f80459a80c896ced0bf9e8fb06a55e3caf22ba17393c8c317df7cf64fe; 8bz-warm-p90.log=a1ca8bb0b17fa1a84df13ee09e9f9100c09f93859262f25ac64abd778d69db1c; 8bz-negative-no-cache.log=95426069ac24abb045def745512fac0cb25abdd3246891cf2578f2efc69edaf6; 8bz-negative-no-recharge.log=27b9a76dca48207050b26e340ba622bf280385cad65e9b13f1514885212ce671; 8bz-negative-legacy-batch.log=6e3b90db5831402d1355c3a26814059e76e1e58f42e5347a79f6585d666badc8; 8bz-negative-override-blind.log=a56b624256257d5a0b3bd44990fc9ecac368be0f43524a203f03e72fdf635b9f; 8bz-abi-lib-serial.log=975ba3e1a2ac50b8596af721d3a700eb239ea2b5061804d17eaabb53109f7128; 8bz-abi-lib-parallel.log=6130e46b0235aa431651430fd5624814f044d3d1bc1edda832a4f288037c7e27; 8bz-source-resources-final.log=4cca27c65f8a3eac6b907801d911f04028627f06a2fd069d8012f4f478bde70c9c; 8bz-conformance-final.log=104140514a9eada74a59e14e5dfca650428b0ab4649f459841c9bf5d46c660fd

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | code_commit: 0e42ff761 | task_ids: T-11 partial (P90/6x30 gates now green; Q4/SD/939 remain) | blockers: none | next_responsible_role: work T-11 Q4 required-oracle matrix, SD deposit check, 939 handoff; then T-12 independent review

2026-11-07 work checkpoint 8ca（T-11收口：Q4矩阵/SD核对/939交接；current_step 11/13）：适配修复提交 `34b2a8fb7`（2文件8+/3-），唯一树clean；8bz保持。

**Q4原required-oracle矩阵核对**：①diff审计（基面a0750086c→HEAD）——Q4行owner（auto-lang-core/test_cases/abi.rs、auto-test-case/contract.rs）无旧行期望语义变化：非source行仍按Some(0)/"0"独立oracle判定，contract.rs删除仅为recipe白名单条件改写（新recipe加入同一结构校验），mod.rs/stale.rs为注册新增；四端无foreign普通业务未新增任何NA/accepted-diff/expected变化。②四端腿在最终代码全绿：ace-av组首跑1030执行3红——经隔离重放确认为两处**8l合并漏改的同型陈旧期待**（modules_950两测试共用的warm edit计数`(1,1,1,1)`→按8l已裁定的真实TypeSupply路径`(1,0,1,1)`）与一处**8f冻结wire的漏适配**（av_data指令普查53→54，FOREIGN_CALL 0x36已在instr_spec且AV准入具名拒、从不执行）；按既有裁定补齐读者适配后三条全绿，ace-av全组重跑**1030/1030 PASS**（739s）。native 226/226（8by）、C腿legacy 94绿+a2c-abi PASS（8by/8bz 37/37含6×30）、Rust a2x 197 PASS（8by）。

**SD-01..SD-08沉淀核对**：11个目标文件全部在册（auto-abi三、a2x两、ac一、a2c一、test-case两、mir三）。各delta与最终实现核对后仍准确，沉淀时须并入后续单元事实：SD-01/02记8bq四required页PageProduction接缝与8bs cold接口工作集/allocation_identity retain、8bw namespace128提交准入、8bt会话元数据、8bx solver工作集；SD-03/04记8br SharedTextProduction/borrowed manifest与C fragment/fileset文本输出准入；SD-05记8bu/8bv Cranelift emit工作区合并票；SD-06记支持矩阵（native/C源foreign、AV/Rust具名拒）与8bz SDK闭包进程级解析复用（identity语义不变）；SD-07记report/2、wrong-oracle控制、正式门与8bz新常驻暖P90门；SD-08记FOREIGN_CALL 0x36/AVFI v1/profile9、core-decl-env/2、boxed声明与内存计量。实际写入归T-13 merge。

**939交接确认**：939(r2 drafting, depends_on含938)消费的八族接缝在最终树全部在册稳定——ForeignCall MIR op+AVFI `auto.abc.foreign-scalar/1`+AV `foreign.unsupported`/Rust `a2r.foreign-unsupported`具名拒；`core-decl-env/2`；`ace.foreign.manual/1`+PageProduction；四接口页（logical/physical/declarations/binding）+`admit_produced_with_memory`；ForeignTargetProvider/TargetProvider/SharedTextProduction；`source938_events.c`事件契约（source938.call=%s）；Resources账户API（reserve/adopt/reserve_output/lease）；foreign.*错误分类。939 §5.6所列"938的call错误/required页"已交付，其T-02正式集成条件满足。

**T-11收口**：原6×30/P90门绿（8bz）、Q4矩阵核对+四端全绿（本单元）、SD核对完成、939交接确认——勾T-11；AC-06/AC-07关闭条件均已满足，最终主张留T-12独立复审核实。遗留：plan827测试副产物写入docs/reports（本次按可再生清理，根因归plan827测试自身，登记复审债务候选）。

同scope证据SHA256：8ca-ace-av/results.json=0d72b1698aba0a0c5d5c5026c8f6ed8a6fe39a8f6dd6291dd8bedf849d824fb9; 8ca-av-reds-isolated.log=9ff622fb2d062c2e775880ea83fa322e94601257a01145197ba0abde05c5a8ba; 8ca-av-reds-repaired.log=f6cc0bee897227b4302f6a3d5c25c096bd81f3183ffe1c001fb300f1be2fe271; 8ca-ace-av-final/results.json=fe7da7ab215832ce80c76c654496bfa133012620c321bd7e8ae6341d8bb0cfe7

stage: work | plan_id: PLAN-938 | plan_revision: r1 | outcome: in_progress | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | code_commit: 34b2a8fb7 | task_ids: T-11 complete | blockers: none | next_responsible_role: T-12 independent same-tree review (execution_done pending review confirmation)

## T-12 独立复审记录（2026-11-07，同树独立上下文；pass 绑定 r1 @ 冻结提交链末 `34b2a8fb7`+fmt提交）

复审范围：938 自有 delta=`diff(6fa8332b^2..HEAD)`=157文件37295+/1759-（排除8l合入的master上游）；逐AC对照实际代码/控制/运行证据核验，勾选按主张逐条追证据。

**逐AC核验（V→证据→结论）**：
- AC-01（V-01/03 候选→受检env→HIR真实消费）PASS：manual准入/权限/来源矩阵（formal 11项含wrong-oracle，8w/8by正式门双跑gate_passed）；unknown非safe/无第二parser（source-query 86/86+conformance 37/37）；SDK pin闭包（8bg/8w）。
- AC-02（V-01/02 双端真callee）PASS：20次真实独立C调用矩阵（bool/ctx/status/源序/短路/混合槽）每切片重放，最近37/37（8bz）；native共享MIR直调/C经HIR薄call（8bh 20 case双端42/43）。
- AC-03（V-02/03/06 严格拒/恰一次/out不滥用）PASS：准入拒矩阵（权限/未审定/类型/签名/伪Checked）+AV/Rust具名拒不执行fallback（av-core/a2x-rust断言）；恢复首错保序诊断。
- AC-04（V-04 P0/IO0/精确失效）PASS：T-07收口8bn 35+1项审计；reader批量事务/header变化仅真实reader/未改函数P0/provider-only重绑生产0（8bz重放）。
- AC-05（V-05/06 恢复/发布原子）PASS：T-08收口8bn；真实进程restart首编辑/首错修复/再恢复（conformance）；required逐缺/逐坏拒；发布故障/撤权/取消/迟到保旧root。
- AC-06（V-06/07 预算/资源/P90/原样本）PASS：T-09收口8bx逐owner表+具名残余（D-938-1）；6×30 body-edit P90 375ms+新常驻暖门（8bz）；512修订churn/hot collector（8m）；原样本门绿。
- AC-07（V-08 首例认证/旧桥/矩阵）PASS：T-10收口8by（legacy 94绿、937 export PASS、正式门最终树PASS）；Q4审计无旧行变化+四端组全绿（8ca，含ace-av 1030/1030修复8l/8f漏适配）。
- AC-08（V-09 本复审）PASS：见下健康检查与扫描。
- AC-09：归T-13 merge。

**遗漏/延后/workaround扫描**：TODO/FIXME/HACK新增0；生产路径调试打印0（34处println全在cfg(test)/测试文件/probe bin/C夹具串）；新生产模块非cfg(test) unwrap/expect共4处均为文档化不可失败场景（String格式化/不可争锁/infallible序列化）。具名残余按D-938-1登记（parser FrontBudget边界/WAL极端/小元数据/space()扫描/8bw两不可观察变异——均保守正确性保留，非未准入owner）。无未批准延后：8bj"环境缓存不作交付"经用户2026-11-07明示裁定推翻后实施（8bz）。

**健康检查**：938 delta 97个src文件rustfmt全净（1处drift已修提交）；改动crates零新增warning（仅av crate既有3条D-...前计划存案：env_layout/mix_u64/shell_lo+hi，本delta未触av lib）。worktree clean。

**spec-impact元数据校验**：frontmatter supersedes(11项)/new_spec_components(checked-source-foreign-calls.md)/touched_goals(GOAL-002)与§规范增量SD-01..08一致；SD核对（8ca）确认11目标在册、delta仍准确、沉淀要点已列——实际写入T-13。

**结论**：pass（r1）；D-938-1..4已登记KNOWN-DEBT-AND-RISKS.md；AC-01..08全过、AC-09留merge。冻结基线=34b2a8fb7+fmt提交，T-13以该树跑冻结固定组并集。

stage: review | plan_id: PLAN-938 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-938/auto-lang | branch: plan-938-dev | review_bound_commit: 34b2a8fb7(+style fmt) | task_ids: T-12 complete (AC-08 pass) | blockers: none | next_responsible_role: T-13 frozen fixed-group union, merge/archive, full-path cleanup

## T-13 收尾记录（2026-11-07）

- **冻结并集正式门**：`cargo t ace-source ace-lowering ace-native ace-av ace-a2x ace-abi ace-artifacts auto-test-case ace-adapters infra-fixed infra-ace-m1 case-a2c-hir-scalar-921 case-ace-ca-d-source-938 --out .../8cb-frozen-union` 于冻结树 75330f456——**13/13 组 PASS、gate_passed=True**：383+5+226+1030+197+243+47+113+84 及三 case/infra 组共 **2328 项执行 0 not-ok**，全程 2922s（ace-native 热构建无争用下 1431s 完整通过；D-938-2 为负载期债务）。results.json SHA256=82bf46ca3fa0773fff0fc66ae4ee7d12b28d0235f3ec8d394ed506412da483de。
- **漂移核验**：master 自 8l pin（bdd73416）至合入前 HEAD（3ca0cfff7）134 提交与 938 delta（157文件）**零文件重叠**——合树中 ACE 工作区逐字等于冻结树，触码漂移为零，不需重跑。
- **SD 沉淀**：SD-01 新建 `auto-abi/checked-source-foreign-calls.md`；SD-02..08 增量落入 conformance-cache/a2x×2/ac/abi-project/a2c-project/test-case×2/mir×3 共 11 文件（含 8bq/8bs/8bw/8bt/8bx/8br/8bu/8bv/8bz 沉淀要点）。
- **合入**：merge commit `8893cfd53`（feat(abi): checked limited foreign scalar calls and source incremental loop (Plan 938)）。
- **清理与guard**：plan-artifacts/plan-938 摘要折叠于本文件各 checkpoint 后整目录删除（含 8by/8bz/8ca/8cb 全部原始日志——SHA 已逐条记录于 §9）；wt-guard clean 后移除 worktree/分支/组目录。

stage: merge | plan_id: PLAN-938 | plan_revision: r1 | outcome: complete | merged_commit: 8893cfd53 | frozen_union: gate_passed=True 2328/0-not-ok | task_ids: T-01..T-13 complete | acs: AC-01..AC-09 closed | work_complete: true

## 10. 待澄清事项

无需再批准任意header importer或新ABI扩容，本包没有这些目标。最终foreign opcode/DefKind与人工契约准入机制由T-02证明，不能以“manifest已写safe”解决。若发现既定required foreign行需要AV/A2R真实支持，具名保留阻塞并据原批准补计划，不得静默NA；本包不擅自扩所有后端FFI。实际source/query接缝漂移先核owner/具体接口，不因935/936未整包交付而空等，也不绕开已安装事实服务。
