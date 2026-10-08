---
plan_id: PLAN-738
status: executing
feature_name: stdlib-assembly-manifest-and-core-validation
author: [agent]
created_at: 2026-10-03
updated_at: 2026-10-08
plan_revision: 1
current_step: 3
total_steps: 8
supersedes_spec_components:
  - docs/specs/stdlib/project.md
  - docs/specs/stdlib/design/backend-assembly.md
  - docs/specs/auto-lang/frontend/design/module-resolution.md
  - docs/specs/auto-lang/trans/overview.md
  - docs/specs/auto-lang/runtime/design/networking-stdlib.md
  - docs/specs/auto-man/design/api-generation-integrity.md
  - docs/specs/auto-cli/project.md
new_spec_components:
  - docs/specs/stdlib/design/assembly-manifest.md
touched_goals: [GOAL-003]
affects: [crates/auto-lang/src/compile.rs, crates/auto-lang/src/autovm_persistent.rs, crates/auto-lang/src/module_cache.rs, crates/auto-lang/src/lib.rs, crates/auto-lang/src/parser.rs, crates/auto-lang/src/trans/rust.rs, crates/auto-lang/src/trans/c.rs, crates/auto-lang/src/vm/codegen.rs, crates/auto-lang/src/vm/native_registry.rs, crates/auto-lang/src/vm/native.rs, crates/auto-lang/src/vm/native_catalog.rs, crates/auto-lang/src/vm/ffi/stdlib.rs, crates/auto-man/src/api_gen.rs, crates/auto/src/main.rs, stdlib]
---

# [PLAN-738] 标准库后台装配契约与 manifest：真实来源、核心符号校验和缓存一致性

## 0. 变更摘要

接续 [Design 33](../design/33-stdlib-runtime-and-http.md) D3a。736正在实现HTTP部署基线；本计划回到最初标准库诉求：公共.at与目标实现如何缝合、谁承担宿主实现、不同环境是否可用，须由实际装配过程证明。

首期交付三层：
1. **全库清点**：stdlib/auto公共模块、目标文件、公开符号与已知provider来源可机器读取；无法解析/未覆盖逐项记录，不能默默遗漏。
2. **真实装配manifest**：明确target/environment，公共源在前、选定目标层随后；VM/Rust/C实际入口采用共同装配计划，并记录实际使用的native/runtime/Auto body。没有http.rs.at不等于Rust HTTP不存在。
3. **核心门禁**：io/net/async/http/json/sse的已声明符号按真实provider/签名/能力分类，引用缺实现、重复实现、签名漂移、目标或环境不适用均有构建/装载诊断；不会空返回或落另一个后台。

这是D3基础和核心门禁，不承诺全标准库所有符号跨后台语义相同，不批量实现当前缺失的Rust net/io、C HTTP或浏览器监听。非核心模块先清点和记录验证等级，进一步签名/ABI和语义parity作为D3b。全actor/CPU/内置TLS/通用WS不在本期。

## 1. 目标

- G1：把声明、Auto目标层、VM native、Rust宿主映射和UI适配明确分开，记录模块/符号/实际目标的唯一执行来源与证据等级。
- G2：生产入口使用同一装配计划，不能依赖未使用的parser后缀helper；VM常规/persistent、Rust/C转译及生成API依赖分别接线。
- G3：核心公开符号检查声明+实现签名、合法ext补全与一符号一实现；native名字/ID已登记不能代替实际shim可调用。
- G4：执行目标vm/rust/c、运行环境native/browser与HTTP/IPC/merged拓扑分别记录；前端Vue不自动让后端成为browser。
- G5：公共源/目标层/provider和编译模式共同决定指纹/缓存，新源或目标变化不能用旧符号/旧生成产物；源码错误定位到真实层文件。
- G6：提供JSON诊断/检查CLI与全库覆盖报告，失败可定位；用真VM和Rust/C实编、假同名/缺实现/缓存变更等反例证明。
- G7：保持734/736 API与HTTP协议合同及普通use隔离，不把本期当更换调度/统一服务传输或通用依赖解析重构。

## 2. 架构方案

```text
resolved public module + explicit execution target/environment
        ↓ shared AssemblyPlan (ordered sources + explicit host provider)
        ↓ source segments / normalized declarations / symbol identities
   ├─ VM codegen + bound NativeInterface proof
   ├─ Rust emission + explicit runtime/Auto provider lowering
   └─ C emission + C layer / external declaration requirement
        ↓ per compilation/session AssemblyManifest + diagnostics
        ↓ cache dependency fingerprint / CLI inspect / 734 generation receipt
```

拟新增：
- `crates/auto-lang/src/stdlib_assembly/{mod,model,loader,validate,providers}.rs`：纯装配模型、源段映射、验证/诊断、版本化provider描述；不依赖Axum/Tokio/UI。
- `stdlib/assembly-providers.json`：外部宿主提供者及明确Unsupported/能力条件的声明目录。由运行生产者/发射分派校验，不作为第二份“声称已实现”的独立表。
- `crates/auto/src/cmd_stdlib.rs`：拟`auto stdlib inspect [--module auto.http] --target vm|rust|c --environment native|browser --format json [--check]`。
- `crates/auto-lang/src/tests/plan738_stdlib_assembly_tests.rs`、`examples/stdlib/assembly/`、`docs/plans/reports/738-stdlib-*.{md,json}`：新测试、同源样例及报告。

保留正常ModuleResolver搜索/pac声明门控/use命名空间语义；shared loader从已解析的public路径组装同目录目标层，不重新猜全项目路径。host-mapped Rust标准库和Auto层实现是可选择的provider形态，显式选择后只有一个活实现；存在未消费.rs.at只表示candidate，不标为实际source。

全库清点与“某次编译实际加载了什么”分别输出，二者可关联，不能用目录扫描代替真实manifest。本期默认strict门只覆盖六核心模块的依赖/引用闭包；全库之外的行为维持，但未知必须在report标明，不称已验证。不会把全部stdlib重新生成或强行自动拼接每个现存.rs.at。

## 3. 技术栈

现有Auto parser/AST/TypeStore、ModuleResolver、CompileSession/ModuleCache、NativeInterface与Rust/C emitter；serde/serde_json与项目已有内容指纹能力。解析使用实际语言parser，不用regex搜fn/contains类型来做契约。VM语义与native逻辑ABI分开，Rust映射复用实际调用分派；ABI字节槽不能从函数参数数目直接推断。

JSON schema仅服务装配清单/工具输出，不改语言语法或公开File/Response/Task ABI。原始stdlib API暂有目标属性/宿主声明混合，首期允许显式分类迁移，不要求一轮搬空公共.at。

## 4. 需求分析与背景调查

### 4.1 来源、版本与授权

起草master基线 `8d33eeb95`，736 executing:r1 1/8，T-01报告引用 `a043bb4fc`，worktree入口 `de3a64353`。734已review R2 pass并归档，delivery=`ec0eae41f`，不再沿用其历史needs_fix阻塞。738取号提交 `f1a1671c8`，独占锁重扫max737后运行new-plan.sh；其他计划不修改。

| 来源 | 本期使用的事实/要求 |
|---|---|
| `docs/specs/overview.md`、`stdlib/project.md` | 多目标标准库、公共/目标层现状，不保证所有后台同构。 |
| `docs/specs/stdlib/design/backend-assembly.md` | 696建立覆盖表，VM与persistent装载分叉，Rust宿主/生成服务/UI适配独立；尚无实际manifest门。部分HTTP表格仍保留早期措辞，须与729/730/734增量对照。 |
| `docs/specs/auto-lang/frontend/design/module-resolution.md` | 路径解析与装载/缓存职责、545 bare/wildcard/named规则、635声明门控不变。 |
| `docs/specs/auto-lang/trans/overview.md`、`runtime/design/networking-stdlib.md` | Rust stdlib映射及HTTP lowering，不得只因缺rs文件拒绝已有host能力；目标草案不等于已用装载链。 |
| `docs/design/raw/stdlib-organization.md` | ext物理补全历史设计；它不是当前实现/可移植ABI的证明，需T-01核对后提出具体规范。 |
| `docs/specs/stdlib/design/api-transport-contract.md`、`auto-man/design/api-generation-integrity.md` | 734共同API分类/真实实现/新鲜度，拓扑能力与stdlib实现能力不是同一个维度。 |
| [PLAN-736](736-http-service-deployment-and-operations.md) T-01、§5/§8 | service配置/ready/生成与HTTP回收正在实现；本期不并行覆盖其lib/auto-man/CLI入口。 |

授权：“736正在实施中，请继续规划下一个计划”；允许只读调查、本仓计划与Design33簿记，不实施/跑Cargo/改其他计划状态/调用外部服务。**执行前置为736独立review pass并合入**；即使部分模型文件可独立写，生产接线须消费736最终来源/生成/ready合同，统一顺序实施。用户无需现在提供更多输入。

### 4.2 代码观察（静态，T-01复现）

| 文件/符号 | 观察与验证方向 |
|---|---|
| `compile.rs::load_module_inner` | context_ext硬编码.vm.at，公共文本+换行+目标文本；再resolve/parse/bytecode，目标层错误可能落公共文件名。装配应保留源段映射。 |
| `autovm_persistent.rs::load_and_register_module` | 另建stdlib目录/root/context路径；auto/前缀处理root与context不同，需真实入口确认，不能仅静态断言全persistent都坏。 |
| `parser.rs::get_file_extensions` | dead_code helper列.at/.vm.at/.rs.at/.c.at；不能用它证明实际Rust/C已选择正确层。 |
| `lib.rs::trans_c_with_session/trans_rust_with_session` | 复用CompileSession后再设parser dest/emit；loader本身仍VM选层，需防类型预解析与实际发射target不一致，不在Rust里编VM shim。 |
| `trans/rust.rs::use_stmt`与call分派 | auto.*及裸stdlib名映a2r_std；provider存在与签名要按实际导出/调用分派核对，模块名称在表中不等于真实模块存在。 |
| `module_cache.rs::ModuleCache`、`compile.rs`早退 | cache只记单file_path/hash，compile传合并源；另有compiled_modules/path跳过。需复现错误cache miss、目标层变动失效和跨target串台，不能只改单个hash字段。 |
| `vm/native_registry.rs`、`native.rs`、`native_catalog.rs`、`ffi/stdlib.rs::register_stdlib_ffi` | 名称/ID/返回类别、真实shim数组/手工注册分散。catalog明确不含所有stdlib手工shim；绑定缺失、ID别名/冲突与逻辑签名须区分。 |
| `stdlib/auto/io.at/io.vm.at/io.c.at` | 公共声明已有#[vm]，VM在ext填方法、C填物理字段；不是纯声明+自动相同ABI。 |
| `stdlib/auto/json.rs.at`与Rust host | rs文件存在，但Rust映射可能消费host runtime；实际选择须记清，不把两个provider重复激活。 |
| `ui/ext_stubs.rs`、`lib.rs`use.web adapter链 | 前端适配是独立机制，不能凭空假定io.vue.at自动拼接；先记录事实/能力，不重写Vue链。 |

## 5. 详细设计

### 5.1 两类清单与验证等级

Inventory扫描`stdlib/auto/**`，公共模块/目录入口、目标层与全部公开fn/method/type/公开字段有stable ID、源位置、解析结果与provider candidate；非公开shim helper也保留映射供验证但不扩张公共API。每个无法解析的文件有diagnostic，不从总分母删除；公共模块+目标文件数量分别统计，T-01冻结实际数据，不沿用本轮99顶层.at计数当公开模块数。

AssemblyManifest v1只记录该次请求/编译的真实选择：
- compiler/provider schema版本、execution target、environment、consumer/transport、features、stdlib来源身份。
- 按顺序的public与target source段、文件内容hash、依赖module及fingerprint，符号decl/impl origin、normalized signature、provider kind/locator、能力与明确拒绝原因。
- selected/unsupported/missing/conflict/unverified等状态；验证等级至少declared、resolved、bound、signature_checked、executed。文件存在/名称登记最多到resolved/bound，不能升为executed。
- 稳定JSON排序；便携清单用stdlib/工程相对source ID，不把机器绝对路径或运行文件根塞进公开收据。需要找文件的local诊断可带实际源路径，与portable fingerprint分开。

每个编译session独立immutable snapshot；两个同名模块不同root/target/session不能覆盖全局“当前manifest”。inspect不执行main、native网络或FS业务操作；缓存/解析读取有界，只写指定报告位置。

### 5.2 目标层选择与provider

AssemblyContext明确执行/发射目标Vm|Rust|C以及Native|Browser环境。渲染Vue不改变后端Rust/VM的Native环境；HTTP/IPC/merged的734能力矩阵另行组合。环境未知不能自动当native或加载其他目标文件。

顺序：resolved public.at在前，然后**选定**的同目录目标层；只有明确Auto-layer provider才加载相应.vm/.rs/.c.at。公共Auto body可作为shared实现；host-mapped/provider模式显式声明由哪个runtime/adapter承担，以及哪个目标层只是未消费candidate。无目标层但已有native/host实现是合法；根本没有实现时调用失败，不能猜另一个suffix补上。

目录入口、别名、re-export、裸/具名/wildcard、循环依赖保持既有语义。重复导入同一source不重复定义；同名异root按resolver身份区分；目标不在模块名后临时拼一个auto/前缀。禁止根据环境目录存在自动落到其他target。

producer签名与调用方声明分别读取/核对：VM实际注册携带独立逻辑签名/alias/feature信息，HostMapped Rust provider由其实际导出/emit map支持，C外来接口用已选C声明/类型。若provider表称支持但callee不存在，strict失败；不以从公共AST复制同一签名到两侧充当校验。

首期provider目录重点六模块；catalog生成/对照到实际注册或发射点，禁止手写一个catalog再让运行继续用不同表。允许既有内联调用lowering保留，但发射时上报实际provider与signature，T-01定位所有核心分派入口并绑定同一identity。async模块与Rust task actor词汇不强行视为同一个调度provider；能验证映射才标supported。

### 5.3 符号缝合、native与能力门

声明可无body，有合法目标body、native绑定或宿主provider提供实现。公共声明+匹配目标实现不算重复；两个活body/两个互不等价provider、类型/参数/返回/async/generic约束漂移则冲突，诊断给双方位置。方法签名归一化显式/隐式self、static与返回类型身份，不靠文本contains。同一声明允许的re-export与native别名明确登记，别名不是额外执行实现。

ext允许目标内部必要字段/实现补全，但公开字段/方法合同不能静默改变；物理layout与opaque资源属于target，manifest记录layout身份，不宣称VM/C/Rust共享物理ABI或跨目标资源互传。混合#[vm]公共声明保持为可追踪legacy事实，不能把所有公共.at属性搬迁造成无关破坏。

native验证两步：name→ID resolution与该NativeInterface里真实shim绑定同时成立。ID复用只限明确别名同一callee；两不同callee争ID报错而不last writer wins。逻辑签名描述不是栈ABI证明；槽宽/资源生命周期更深ABI在D3b，但核心变参/self/async差异不能以Unknown假称signature_checked。声明本来允许动态值/泛型/变参时，以明确Any/Generic/Variadic身份及约束保留，不能误当未解析类型；兼容lowering的参数适配变体须单独记录。缺独立签名的既有provider标unverified，调用若属于本期strict核心须补生产者元数据/真实测试，不能豁免到“已支持”，也不能把原已工作的能力仅因缺元数据降为Unsupported以过门。

缺实现按引用/导入项/生成依赖闭包校验，未使用的unsupported声明不使整个http模块无法导入。正常use namespace的可见性不放宽，用户同名http/json/File不当stdlib。Browser下监听/本地FS/native sockets无适配须Unsupported，不假返回0/空串；native-backed源不能被Browser简单链接进去。确有浏览器fetch/console adapter时另报consumer/environment和已验证来源，不通过Vue renderer名字假造能力。

核心公开符号全部有状态和原因；现存占位实现标DeclaredStub/Unsupported，例如未交付Server.static，不因已绑定shim而标实用能力。报表unsupported是诚实覆盖结果，不要求此计划新造net.rs.at/io.rs.at。

### 5.4 实际入口、源位置与失败传播

VM CompileSession与persistent loader共同使用AssemblyPlan；source segment保留original source ID/range，解析/冲突错误能反映目标文件位置；保持Database dirty/type_store/use语义。Rust/C入口先确定真实target，再进行类型装载/发射；若存在VM专用bytecode预处理阶段，其模块绑定不得被当成目标实现，也不得给Rust/C sidecar做VM CALL_NAT编译。

T-01用最小synthetic public+三目标fn验证真实consumer；若现有incremental管线需要超出本期的整体改造才能选层，应needs_replan并保留AC，不做绕过实际consumer的“新helper绿”。共用计划不是必须共用全部编译器/TypeStore或把foreign Rust源码交给Auto parser。

生成API的Auto核心调用依赖manifest进入734 generation receipt：业务指纹与assembly fingerprint各自记录，源/provider/features改变时新鲜度门识别；736启动/ready引用最终版本。失败传播至auto-man/build/serve，不吞错启动旧generation；Config profile不改变source assembly指纹，target/environment/features等会改变。

单纯本地Rust原生直接调用a2r-std没有Auto装配阶段，不强行声称有Auto manifest；其导出验证作为Host provider证据登记。

### 5.5 缓存一致性

cache key包含resolved模块身份、target/environment、provider schema/features和public/选定层/host版本/依赖闭包的内容指纹；记录missing sidecar的存在性，新增/删除层也会失效。只改mtime不能伪造新/旧内容；公共+目标合并hash不能拿公共文件单独重算冒称valid。

修复CompileSession早退与ModuleCache的一致性，编译epoch中新源必须更新真实模块/类型/bytecode和manifest。同一active VM不得热换stdlib ABI；执行后改target或stdlib root须明确SessionTargetMismatch/重建，不能让旧堆资源继续跑新impl。跨target cache可以共享纯内容存储，不能共享同一活manifest/impl选择。

源码/provider变动后的新编译失败，保留旧进程/缓存资产只作为旧版本，当前build非零且不启动它；不修改全局AutoCache SQLite设计或重做所有增量缓存。本期指纹不是密码学信任/签名系统，校验旨在正确性与新鲜度。

### 5.6 inspect与核心覆盖报告

inspect输出inventory或某次AssemblyPlan/Manifest（模式字段明确），check输出machine-readable diagnostic code、module/symbol/target/env、双源位置、缺provider/feature原因及改用支持目标/实现的指引。dry inspect的selected provider须来自实际resolver/provider策略，执行/生成另有消费证据；不伪造executed。

--check语义：核心被请求的closure不得有missing/conflict/unverified；明确unsupported被调用也非零，未引用的unsupported是inventory项。全库inventory存在解析失败给完整JSON且有非零/partial状态，不能统计漏项为pass。非核心未验证项明确列出，不将全库report称全库strict已完成。

最终报告包括六模块所有公开符号×vm/rust/c×native/browser（实际入口支持/不支持都有原因），VM/Rust已验证HTTP/file/upload例与C已有支持样例；浏览器仅测试能力拒绝与已有adapter事实，不把host server放到webview中。D3b后续逐模块扩展严格门和行为parity，不能从本期signature_checked推定同取消/错误/ABI语义。

### 规范增量

起草只提出增量，不修改canonical。736落地后再对最终服务Spec兼容。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/stdlib/design/assembly-manifest.md` | 文件存在覆盖 → actual plan/manifest、源段/目标/环境、验证等级、provider与核心门 | 明确装配合同 | AC-01..08 |
| SD-02 | modify | `docs/specs/stdlib/design/backend-assembly.md` | VM路径/宿主覆盖分散 → actual provider索引与核心能力；修正旧HTTP矩阵，区分inventory与执行 | 原始多后台诉求 | AC-01/03/05/08 |
| SD-03 | modify | `docs/specs/auto-lang/frontend/design/module-resolution.md` | 解析与两loader分叉 → 解析后统一装配、源位置、原use隔离/循环语义和cache边界 | 保持模块语义 | AC-02/04/06 |
| SD-04 | modify | `docs/specs/auto-lang/trans/overview.md` | Rust/C实现来源不透明 → selected target/provider和真实emit证据/unsupported诊断 | 不把VM preparse当Rust实现 | AC-02/03/05/07 |
| SD-05 | modify | `docs/specs/auto-man/design/api-generation-integrity.md` | body/source指纹 → 额外assembly/provider依赖闭包与新鲜度门 | 防改stdlib仍用旧服务 | AC-06/07 |
| SD-06 | modify | `docs/specs/auto-cli/project.md` | 无inspect → inventory/actual manifest JSON及check退出码 | 可检查交付 | AC-01/07/08 |
| SD-07 | modify | `docs/specs/stdlib/project.md`、`docs/specs/auto-lang/runtime/design/networking-stdlib.md` | 后台/物理填充草案不清 → 新合同入口、六模块能力与D3a/D3b边界 | 不声称补齐全部后台 | AC-05/08 |

## 6. 测试设计

### 6.1 最小同源与反例

| 族 | 必须观察 |
|---|---|
| 目标装配 | synthetic public+vm/rs/c层不同返回值，真实VM/Rust/C对应选择与执行；公共pure body共享、显式host provider无sidecar合法；foreign层不能串入。 |
| VM两入口 | CompileSession及persistent同模块选择/来源；auto.io前缀与目录module正确，bare/named/wildcard/re-export/循环无新增泄漏。 |
| 冲突/来源 | 声明+目标合法，重复body/provider、self/static/async/返回/参数漂移、未知类型/同名用户模块；双源位置，包括目标文件故意语法错误。 |
| native绑定 | 有名字/ID无真实shim、冲突ID/合法alias、feature off、stub占位；装载/链接失败且不执行业务。独立producer签名改动可被识别，不能复制同一AST蒙混。 |
| 六模块 | 每公开符号有支持/拒绝/未验证分类；被引用Unsupported/missing明确错误，未引用不害整个module；已有json/io/native HTTP/task语义不擅改。 |
| 环境 | Native VM/Rust backend、Browser不可监听/文件/socket、同Vue前端Native后端的双context；UI adapter不假冒stdlib suffix层。 |
| 缓存 | 只改目标body/签名、增加/删除层、依赖/host feature与版本变更；同mtime不同内容、同名异root、双session、target切换；真实返回或当前诊断变化、旧ready不能被消费。 |
| 生成/CLI | auto stdlib inspect JSON稳定、check退出码/完整分母；真实生成Rust API→manifest→736启动；改assembly但不改api.at，新指纹或失败不启动旧impl。 |

inspect验证不执行网络/文件业务；真实执行witness在单独fixture，IO只用temp/loopback。C真MSVC/rustc/cargo单fixture实编；没有C HTTP provider是Unsupported而非造一个手写server。Rust普通stdlib同源witness及734/736实际generated API；不能只测loader helper或静态JSON金样。

### 6.2 核心与全库门

全库inventory所有公共/目标文件必须有记录及完整总数；真实未解析记录为partial、整体检查非零，不隐藏错误。六核心正常声明/provider需可解析并全部分类；可验证支持集必须signature_checked+必要exec证据，缺实现是Unsupported/missing且不冒称支持。

新测试族plan738_stdlib_assembly；真HTTP注册http_e2e_plan738并纳test-http-e2e串行，尽量复用734/736 fixture。不把全stdlib每符号执行一遍（阻塞stdin、任意FS/network、进程启动等）作为inventory。签名级fixture无需重网络；行为与资源parity另属D3b。

### 6.3 实施门禁（本轮不跑）

在 `D:/autostack/.wt/lang-738/auto-lang`：
- `cargo check -p auto-lang`、`cargo check -p auto-man`、`cargo check -p auto`；开发`cargo t plan738`、compile/module_cache/native scoped族、`cargo test -p auto --bin auto stdlib -- --test-threads=1`（新增CLI族）及auto-man API/provider scoped。
- 复审裸`cargo t` + `cargo tv` + `cargo tt` + `cargo th --test-threads=1`；实际ui_gen消费点改动才`cargo tu`。a2r-std若为provider元数据必要改动，补该crate scoped/适用全crate；不改auto/lib/aavm，无taa。
- 真Rust/C/witness生成实编、736代表CLI/配置/ready与729/730/734兼容族；三目标unsupported另有负测。
- 不改语法参考/文档生成器，assembly JSON为工具模型而非语言Schema；若实际改变文档Schema定义触发docs_gen，按最终diff判定。tf只merge到期主检出单实例，不为本期每plan跑。
- 零新增确定性红，按同命令基线逐名分诊，不靠改公开API、删provider/跳过fixture清账。

报告新`docs/plans/reports/738-stdlib-{decision,inventory,assembly,providers,cache,verification}.md`及inventory/manifest JSON；绑定最终revision、source/deps/hash、每AC/SD证据与验证等级。

## 7. 验收标准

| ID | 可观察交付 | 验证/期望 |
|---|---|---|
| AC-01 | 全stdlib/auto目录清点与六模块公开符号完整 | 实际文件/AST数量、inventory JSON和partial诊断；没有“解析失败就略过”的绿；支持集/缺口有分母与原因。 |
| AC-02 | 生产VM/Rust/C/persistent实际选择共同装配计划 | 同源不同target真执行+manifest源/provider一致，无VM层串Rust/C；合法host provider缺sidecar不误拒；不是孤立helper。 |
| AC-03 | 核心声明/实现/实际native绑定校验 | 合法声明+一impl绿，冲突/签名漂移/native空ID/假名字/feature off精确诊断；IDalias与独立producer元数据有证据。 |
| AC-04 | 源段错误位置与既有use隔离保留 | 目标层语法/冲突指向实际文件行，root/body同名反例、bare/wildcard/named/re-export/循环回归不泄漏；双session不串。 |
| AC-05 | target/environment与核心能力严格区分 | 六模块全矩阵，Browser不可用明确Unsupported，Vue frontend不改变Native backend；stub不当支持，不新造未支持backend。 |
| AC-06 | 内容/目标/provider/依赖变化正确失效 | 同mtime、增删sidecar、双root/session、provider/features与target变更真编译结果/诊断；旧bytecode/manifest/APIready不冒新实现。 |
| AC-07 | CLI与生成HTTP服务真实消费manifest | inspect/check JSON/退出码/副作用隔离；generated Rust实编、736实际startup/ready与assembly fingerprint，改stdlib后新鲜度失败或重新生成。 |
| AC-08 | 核心/全库验证等级、兼容与规范可沉淀 | 全SD对应证据、零新增确定性红；729/730/734/736既有行为保持，D3a未验证/ABI/调度范围明示，不称全部stdlib已parity。 |

## 8. 执行步骤

### T-01：最终736基线、装配调用图与六模块provider冻结 ✅

- [x] 前置736review pass+merge，读最终config/ready/生成与canonical增量；`bash scripts/new-wt-group.sh lang-738 --branch plan-738-dev`，auto-down兄弟只读，禁止链接。
  [✅ 已完成] 2026-10-08 用户指令「实施它」进入 executing（原 736 前置的替代满足与 736 现状勘验见 `docs/plans/reports/738-stdlib-decision.md` §5/§7）；worktree 组 `D:/autostack/.wt/lang-738/{auto-lang,auto-down}` 已建（基面 master@c3ccd32c3，auto-down detached 895f8d0 只读，无链接）。T-06 消费 736 最终合同保持 gated（§8 复核清单在报告中）。
- [x] trace CompileSession/persistent、Rust/C实际CLI及生成API、native实际注册和a2r分派，复现后缀目标/缓存/源位置缺口。确认实际active consumer，不把未调用方法当验证面。
  [✅ 已完成] 四装配面实勘冻结（报告 §1/E1-E4）：①VM 管线 resolve_uses→load_module 硬编码 .vm.at；②persistent 双 auto/ context 恒缺失+stdlib-only 查找；③Rust trans 零模块装载+三处重复 26 名名称表（~10 名无真实 a2r-std 模块）；④C trans 头包含路由+cmd_a2c_stdlib 逐文件生成。ModuleCache 双层恒 miss（E2）；native 名 surface=磁盘扫描 CWD stdlib/auto + 目录固定 ID + ~54 手工 shim 三机制（E3）；find_std_lib 项目根分支恒不命中的来源身份裂缝（E3）。
- [x] bounded调查产出738-stdlib-decision：六模块完整符号/producer签名来源、provider目录schema、active native别名/ID、混合public属性兼容、target/environment来源、源段与cache key、初始全库分母/parse失败。
  [✅ 已完成] `docs/plans/reports/738-stdlib-decision.md`（§2 六核心 13 层 parser 精确分母：11 parse-OK + async.at/json.rs.at parse 破损冻结；§3 provider 目录要点与名称漂移矩阵；§4 设计裁决建议）。
- [x] 最小公共+三目标/host mapped真入口原型；无法闭合真实target接线/全核心分类则needs_replan，不将AC换成inventory-only。涵盖全AC/SD。
  [✅ 已完成] 探针族 `crates/auto-lang/src/tests/plan738_assembly_probe_tests.rs` @worktree `ac41c2d1e`：P1-P6 六测全绿（真实 CompileSession 管线/trans 双入口/persistent/六核心 parser 清点）；target 接线可闭合评估=不触发 needs_replan（报告 §4.2）。

### T-02：共同模型、provider目录与全库inventory ✅

- [x] 依赖T-01。新增stdlib_assembly模块和stdlib/assembly-providers.json，lib.rs导出；复用AST/类型身份，inventory全文件/符号与明确验证等级。
  [✅ 已完成] worktree `d70d4c8a9`：`stdlib_assembly/{mod,model,loader,providers,validate}.rs` + lib.rs 导出；全库扫描 115 `.at`（74公共+23vm+13rs+5c）分母恰清点、FNV-1a 内容指纹、验证等级 Declared 起步、符号归一身份（parent/ext owner → `Owner.name`，遍历 TypeDecl.methods）；provider 目录六核心逐目标声明 + 发射名称表 11 名 unsupported。
- [x] provider目录与真实注册/emit表有机器对照，文件hash/源段/候选与选择不混淆；非核心解析错误完整记录。
  [✅ 已完成] `cross_check_real_surfaces`：vm 声明对照 BIGVM_NATIVES 名 surface（CWD 钉仓根后扫描）、rust a2r-std 对照 `pub mod`、c 对照磁盘 locator；unsupported 必填 reason；发射表漂移差集与目录 unsupported 集合哨兵断言。实勘修正：sse parse_sse 无任何 callee（落册 unsupported）；发射表缺 `file`→`a2r_std::file` 共 11 名漂移。全库 isolated-parse 基线 39/115 三类构成（决策报告 §2b）。
- [x] `cargo t plan738`/模型单测与inventory JSON完整性；AC-01/03/05，SD-01/02/07。
  [✅ 已完成] `cargo t plan738` 11/11 绿（6 探针 + 5 正式族）；稳定 JSON（双序列化逐字节一致 + 排序自检 + 无绝对路径泄漏）；scoped：module_cache 10/10、native_registry 13/13；`cargo check -p auto-lang` 新文件零警告。

### T-03：VM/目标转译/persistent装配接线与源映射 ✅

- [x] 依赖T-02。修改compile.rs::load_module_inner、autovm_persistent.rs::load_and_register_module、lib.rs trans_*入口、parser必要诊断/source映射、trans/{rust,c}.rs实际provider消费。
  [✅ 已完成] worktree `340de9ccd`：①load_module_inner 层选择由 `AssemblyTarget::context_extension()` 驱动（硬编码 ".vm.at" 收编；Rust/C 目标零装载面——层记 candidate 不合并，发射面如实走名称表/头包含）；②`LayerSelection` 记录进 session（manifest v0 seam，T-06 消费）；③persistent context 路径修复（原 `with_extension` 吃掉 `.vm` + 未剥 `auto/` 前缀双 bug 使 `.vm.at` 恒不可见）+ ext canonical 对齐（`auto.<stem>.<target>.<fn>`，TypeDecl 并入型与 ext 型双臂）+ edition2021 `if let` MutexGuard 双锁死锁修复（context 修复使查找首次可达后暴露）；④trans_c/trans_rust 入口先声明装配目标；⑤Rust sibling 扫描排除 .vm.at/.c.at foreign 层（VM/C 层不再串入 Rust 发射类型上下文，.rs.at 为 Rust 选定层保留）。
- [x] resolved public路径+显式context选源，root在前/选层后；public/native host映射与pure body均可追踪，不把Rust解析预处理当VM执行实现。保持545/635路径/可见性/循环语义。
  [✅ 已完成] 源段边界记录（context_byte_boundary）+失败路径有界归因：公共段单独可解析 ⇒ 错误归因目标层真实文件（`target_layer_syntax_error_attributed` 实证）；545/635 语义由 use_semantics 族回归守护（117/117 含 use_semantics/module_cache/native_registry/repl/plan727/729/730）。
- [x] 三目标同源真正执行/emit与VM两入口对拍、源层错误位置；AC-02/04/05，SD-01/03/04。
  [✅ 已完成] `target_driven_layer_selection_and_record`（VM 合并 vm 层、Rust/C 上下文无 foreign 层符号、candidate 齐、记录全字段）；`vm_two_entry_same_layer_visibility`（session 与 persistent 对 io ext 方法/net 顶层 fn 同可见——P5 探针由破损证据翻转为修复后契约）；`trans_entries_declare_assembly_target`。门禁：plan738 15/15、scoped 117/117（plan730_staged_lease_expiry 首跑并行负载 flake，复跑单测 0.44s 绿——上传服务非本触面）、`cargo tv` 162/162。

### T-04：核心符号/native校验与环境能力

- 依赖T-02/03。修改vm/{native_registry,native,native_catalog}.rs与ffi/stdlib.rs必要provider描述/实际绑定检查、codegen所需引用闭包消费；Rust核心provider分派使用真实identity。
- 六模块全部decl状态，独立逻辑signature/alias/feature/stub与实际shim绑定；避免name存在即绿，缺实现/冲突/不适用在业务前错误。目标物理字段身份记录，不重做资源ABI。
- native欠绑定/ID冲突/签名漂移/假同名/Browser拒绝与现存核心支持正测；AC-03/05/08，SD-01/02/04/07。

### T-05：缓存依赖与编译epoch一致性

- 依赖T-03/04。修改module_cache.rs与compile.rs早退/dirty链、persistent manifest状态；缓存包含全部selected source/provider/target/env/deps/存在性。
- 只改target层或host feature新编译不能旧返回；活VM禁止无契约ABI热换，明确target/root改变需新session。缓存错误不降级旧module。
- 同mtime/增删层/双root/session/target/依赖变更真实结果与diagnostic；AC-04/06，SD-01/03。

### T-06：检查CLI和API generation/serve收据

- 依赖T-02..05。新auto/cmd_stdlib.rs，main.rs挂接；改auto-man/api_gen.rs及736最终真实生成/启动消费者（T-01锁路径），assembly fingerprint进入734receipt并由736ready引用。
- inspect只读取装配所需源码/元数据，不执行业务网络/文件操作；稳定JSON与非零check，inventory/actual mode区分；runtime serviceconfig不是assembly内容指纹，provider变化必须新鲜度失效。
- CLI JSON/负测及真实生成Rust→serve，改stdlib但api.at不变；AC-01/06/07，SD-01/05/06。

### T-07：同源样例、核心完整矩阵与实际执行证明

- 依赖T-03..06。新plan738_stdlib_assembly_tests.rs/tests.rs接线、examples/stdlib/assembly/及target witness；依现有C/Rust构建工具生成真实产物，不手写业务替代。
- 完成§6.1全部反例/真入口、六模块全符号target/env矩阵和全库inventory；HTTP串行兼容734/736与729/730，报告selection/exec等级/真实方法，旧root未消费侧层不能报已合并。
- AC-01..08，SD-01..07；无法执行目标/witness明确阻塞，不只生成串金样。

### T-08：门禁、独立review与规范交接

- 依赖T-01..07。§6.3分级门、warnings/fmt/debug/未批准延期扫描，验证报告绑定最终revision/每AC/SD；新红基线对照，未覆盖核心不能标“全库通过”。
- /auto-plan:review独立验证真实callee和target调用点、分母/验证等级/缓存反例；准备SD沉淀稿，merge再canonical/ledger/Design33/索引和归档。
- 清理前lang-738两兄弟各wt-guard clean；tf仅merge到期主检出单实例。全AC闭合才reviewed/archived。

## 9. 复审记录

### work 交接（2026-10-08，T-01..T-03 完成）

- stage: work
- plan_id: PLAN-738
- plan_revision: 1
- outcome: pass（T-01/T-02/T-03 完成；整体 executing 继续）
- code_commit: worktree plan-738-dev `ac41c2d1e`（T-01）→ `d70d4c8a9`（T-02）→ `340de9ccd`（T-03）；基面 master c3ccd32c3
- task_ids: T-01、T-02、T-03（current_step 3/8）
- evidence: 决策报告 §1-E1..E5 + §2b/2c/2d；plan738 15/15、use_semantics/module_cache/native_registry/repl/plan727/729/730 117/117、`cargo tv` 162/162、新改文件零警告
- blockers: T-06 消费 736 最终生成/ready 合同——保持 gated（736 已推进至 T-05 完成，合入后按报告 §8 复核）；不阻塞 T-04/T-05/T-07
- next: T-04（核心符号/native 校验与环境能力——六模块全 decl 状态、独立逻辑签名/alias/feature/stub 与实际 shim 绑定；CWD 依赖名 surface 的生产构成冻结[报告 §2d]）

T-03 实施要点（复审注意）：③persistent 修复为**行为变更**——`.vm.at` 层对
persistent/REPL 由不可见变可见，短别名注册面扩大（P5 探针由 is_none 破损证据
翻转为 is_some 修复契约，见探针头注）；edition2021 `if let` 持锁双锁死锁系
context 修复使查找首次可达后暴露的潜伏 bug（查找临时量须提升出 scrutinee）。
sibling 扫描排除 .vm.at/.c.at 为 AC-02 行为变更（VM/C 层不再进入 Rust 发射
类型上下文），plan727/729/730 金样绿。

### work 交接（2026-10-08，T-01+T-02 完成）

- stage: work
- plan_id: PLAN-738
- plan_revision: 1
- outcome: pass（T-01/T-02 完成；整体 executing 继续）
- code_commit: worktree plan-738-dev `ac41c2d1e`（T-01 探针）→ `d70d4c8a9`（T-02 模块族）；基面 master c3ccd32c3
- task_ids: T-01、T-02（current_step 2/8）
- evidence: `docs/plans/reports/738-stdlib-decision.md`（§1-E1..E5 + §2b/2c/2d T-02 增补）；`cargo t plan738` 11/11；module_cache 10/10；native_registry 13/13；新文件零警告
- blockers: T-06 消费 736 最终生成/ready 合同——736 尚 executing 3/8 未合入，保持 gated（报告 §5/§8）；不阻塞 T-03..T-05/T-07 主体
- next: T-03（VM/persistent/trans 装配接线与源映射——依赖 T-02 模型；AssemblyContext 驱动层选择替换 context_ext 硬编码，persistent context 路径修复，ModuleCache 死缓存修正在 T-05）

T-02 增补实勘（详见报告 §2b/2c/2d）：全库 isolated-parse 39/115（三类：语法破损
str.at/list.at/iter 族、跨模块依赖假阳性候选、.rs.at 镜像漂移；`use auto.str`
stdlib 导入面破损系在案已知——stdlib_tests.rs #[ignore] 佐证）；sse parse_sse
声明无 callee；native 名 surface 扫描 CWD 依赖。三者均落册为 inventory 诊断/
目录 unsupported/报告证据，不阻塞、不弱化 AC。

### work 交接（2026-10-08，T-01 完成）

- stage: work
- plan_id: PLAN-738
- plan_revision: 1
- outcome: pass（T-01 单任务完成；整体 executing 继续）
- code_commit: worktree plan-738-dev `ac41c2d1e`（探针族 6/6 绿；基面 master c3ccd32c3）
- task_ids: T-01（current_step 1/8）
- evidence: `docs/plans/reports/738-stdlib-decision.md`（E1-E5 全锚定）；`cargo check -p auto-lang` 绿；`cargo t plan738` 6/6
- blockers: T-06 消费 736 最终生成/ready 合同——736 尚 executing 3/8 未合入，保持 gated（报告 §5/§8 复核清单）；不阻塞 T-02..T-05/T-07 主体
- next: T-02（stdlib_assembly 模块 + provider 目录 + 全库 inventory，依赖本报告冻结的分母与四装配面边界）

入场说明：计划 §4.1 原执行前置为「736 独立 review pass 并合入」；2026-10-08 用户
明确指令「计划738 实施它」，据此从 drafting 进入 executing。该前置的替代满足方式
（736 worktree 只读勘验 + T-06 gated + 其余任务生产耦合分离论证）记录于决策报告
§7。736 合入后按报告 §8 四项复核，如与 §5 描述漂移按最终代码重锚（不改 AC）。

### 起草交接（2026-10-03）

- stage: new
- plan_id: PLAN-738
- plan_revision: 1
- outcome: blocked
- blocker: 736 executing尚无独立review pass+merge的最终生成/启动合同；本期共享lib/auto-man/CLI生产接线。
- next: 736 reviewed+merged → T-01真实装配/provider/缓存原型冻结 → /auto-plan:work PLAN-738。
- changed_tasks: T-01..T-08（新）
- changed_acceptance: AC-01..AC-08（新）
- 本次只计划/Design簿记，未创建738 worktree、未跑Cargo/编译/网络、未改canonical或736进度。blocked是实施顺序依赖，不缺用户规划输入。
- 起草结构检查通过：11章节、8任务/8AC/7SD覆盖、已有影响路径/Spec路径存在、编号唯一、next-id已推进、新增链接有效、git diff --check通过。Design索引既有513计划链接不在本期修复面；此检查不是实施独立复审。

## 10. 待澄清事项

- T-01负责六模块producer签名与当前parser/source-map/target实际入口原型；事实与canonical冲突提出具体修订，不以“代码如此”擅改公开API/语义。
- 全库inventory不等于全库strict/语义parity；D3b扩展非核心provider门和资源/错误/取消ABI对拍，不能因D3a有manifest称所有符号能用于所有环境。
- .vue等环境文件目前非统一stdlib后缀机制；有适配路径才登记，不能假造Io浏览器实现或因Vue前端误禁Native后端。
- 736最终服务实现会改变生成/ready路径，执行须用其merge后版本；738不并行改其核心入口、不代实施736。
- CPU/native阻塞、TLS/完整WS/更大上传/崩溃恢复按HTTP路线独立需求；本期提升的是标准库实现来源与错误可见性，不增加这些HTTP协议能力。
