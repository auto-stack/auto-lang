---
plan_id: PLAN-967
status: drafting
feature_name: 完整C/Rust FFI、自动binding与通用宿主资源
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-12
plan_revision: 8
current_step: 0
total_steps: 19
depends_on: [PLAN-960, PLAN-962, PLAN-963, PLAN-938, PLAN-939]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md, docs/specs/auto-bindgen/project.md, docs/specs/auto-abi/project.md]
new_spec_components: [docs/specs/ace-migration/m2-host.md]
touched_goals: [GOAL-002]
---

# [PLAN-967] 完整C/Rust FFI、自动binding与通用宿主资源

## 0. 变更摘要

2026-11-12 r8：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-12 r7当前：纳入默认自有Auto基础标准库/非libc默认provider及长度字符串ABI边界，见§5新增合同。保原任务/阶段/范围，新增AC-NATIVE/V-NATIVE/SD-NATIVE，仍只规划。

历史修订：2026-11-11 r6当前：增加F0最小可复用AV/Rust互操作先行切片；先闭普通源码→自动binding/shim→受检调用→真实Rust函数，不等完整C importer、serde_json、泛型/async/全标准库；原完整FFI-M2-01..08不减。 原15项任务/全部AC/V保留，新增4项早期切片任务，现19项、drafting、0完成；只规划。

历史修订：2026-11-10 r5执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r4当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-09 r3：按用户要求纳入[Design56：完整FFI与新版标准库](../design/56-ace-m2-ffi-and-native-stdlib.md)的本包义务；当前仍为drafting、0/15，只规划不施工。原任务/AC/历史记录保留，新增任务均在最终work验证与独立review之前。

承938/939有限桥，建立中立typed调用、资源handle、borrow/owned/transfer、同步重入与异步完成端口；C/Rust/Python既有应用桥逐项映射。generation+owner+签名验证，授权每次调用重核，旧native数字ID只作兼容映射。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

2026-11-11本轮授权：用户要求规划首个ACE UI Hello及AV/Rust FFI路线、核对既有M2冲突并作必要重排；仅规划/设计/导航，不实施、创建worktree或探针/fixture/SDK、构建/运行、提交或发布，未指定模型/工期。沿旧语言用例守恒、新Auto标准库与AutoABI/FFI、Demo递进的三线，不削完整M2范围。

现场语言HEAD 9e36d984e、UI HEAD d88e9c51。已核av-core/src/host.rs仅console/1（sha256前16 30d6dcc475850cd0）、a2x-rust/src/provider.rs仍拒source foreign（3ea453dc2452576f）、core/src/source/cst/frontdoor.rs（dd3d93d06799ae52）；现有auto-bindgen/src/extractor.rs是预置C manifest表（8c7f1295cc83c6fb），不能称Rust元数据导入器已实现。Iced由Rust宿主编译执行，AutoVM不解释其Rust源码；现有DynamicComponent/VmBridge/handler_codegen深依赖旧执行链。

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `experimental/ace/crates/av-core/src/host.rs`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/foreign-contract`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/abi`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/vm/host_bridge.rs`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-09 增补调研锚点

依据用户本轮M2范围追加与已读取的标准库引用讨论，读取基线HEAD `b4f1af0be67b91b601345a4284ecd4036cb1a91e`；本轮只读核源码，不宣称现有实现已满足新义务。Design56区分现状与目标；原§4历史快照不覆盖本次范围。

- `crates/auto-bindgen/src/extractor.rs`（现存入口/规范；开工核真实符号与SHA）。
- `crates/auto-bindgen/src/type_map.rs`（现存入口/规范；开工核真实符号与SHA）。
- `crates/auto-bindgen/src/manifest.rs`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/foreign-contract/src/typed.rs`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/foreign-contract/src/resource.rs`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/foreign-contract/src/lane.rs`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/source-query/src/modules/packages.rs`（现存入口/规范；开工核真实符号与SHA）。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：C/Rust自动导入与中立FFI/资源。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [crates/auto-bindgen/src/extractor.rs](../../crates/auto-bindgen/src/extractor.rs)：现有预置C表，不能冒完整Rust importer；读取SHA256前16 `8c7f1295cc83c6fb`

- [experimental/ace/crates/foreign-contract/src/typed.rs](../../experimental/ace/crates/foreign-contract/src/typed.rs)：共享签名/资源/borrow事实；读取SHA256前16 `fb148c0a779d5c95`

- [experimental/ace/crates/abi](../../experimental/ace/crates/abi)：独立ABI oracle及目标适配；目录职责已核，确切符号/闭包开工再核

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08, T-09, T-10, T-11, T-12, T-13, T-14, T-15, T-F0-I, T-F0-V, T-F0-R, T-F0-D |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


承938/939有限桥，建立中立typed调用、资源handle、borrow/owned/transfer、同步重入与异步完成端口；C/Rust/Python既有应用桥逐项映射。generation+owner+签名验证，授权每次调用重核，旧native数字ID只作兼容映射。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：双向ABI与异步完成端口（r2）

分import/export、调用方/被调用方、标量/聚合/资源与C/Rust/Python现用桥，核动态库装卸、原方allocator、符号/ABI版本。异步本包只交注册/完成/取消端口及受控真实provider；968消费端口实现语言await，967不反向依赖968。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 2026-11-09 正式M2范围增补（r3，当前合同）

本次把原有限接面正式扩为FFI-M2-01..08；保C/Rust/Python现用桥及原资源/授权/异步完成义务。四个阶段：A公共输入/共享合同（T-09）、B自动导入（T-10 C、T-11 Rust）、C四端/双向ABI/资源（T-12/13）、D精确缓存/真实库（T-14/15）。每批记录输入、冻结接口、实际consumer、V证据和未闭项后才交下一批；最终review仍由原T-05独立进行。

公共use.c/use.rs不要求用户逐函数手写签名/布局/extern。库路径/工具链/所有权注解可显式给，无法证明的raw边界必须显式；不猜Rust私有ABI或任意库安全性。C用真实预处理和目标布局，Rust用公共元数据/具体实例到C-compatible shim或经证明等价adapter，所有事实汇入同一Checked Foreign IR。

自动导入同时覆盖既有消费者的特征表；varargs/union/bitfield、Rust traits/宏/泛型等按真实用法逐行核，不能缩成scalar demo。具体泛型能力新增依赖963。native直接产机器码，AV/AC/新A2R/新A2C均实际调用；旧AST/VM与手写manifest不能代自动链。异步完成端口由此输出给968，967不依赖968；生成物分发/安装由978/986接，不让本包等最终安装才交基础设施。

真实库最低路径为C标准头及SQLite、Rust serde_json公开API；T-09钉版本/工具链/合法source与独立expected，并用自有callback/owned/具体泛型/future fixture补资源边界。C独立编译器caller/callee、Rust独立公开API测试为oracle；全旧分母与新增义务取并集，不将最低样本当封顶。

本次新增验收/任务如下表映射，详见§6–8；原任务不删除、原非目标中的“本包之外”按本节明确新增范围解释。所有V均为待执行设计，当前不写canonical Specs。

| 任务 | 验收 | 用例 | 本包交付/义务 |
|---|---|---|---|
| T-09 | AC-07 | V-09 | 接通工程use.c/use.rs到共享绑定和typed foreign事实，冻结真实库/全特征required表、来源/注解/工具链和合法/非法公共源码；先走通一个真实导入与失败恢复。 |
| T-10 | AC-08 | V-10 | 实现C真实预处理/头文件闭包、target/sysroot/宏/typedef和ABI布局、函数/record/enum/pointer/callback/opaque、常量/inline shim；将旧用法特征逐行闭合。 |
| T-11 | AC-09 | V-11 | 实现Rust公开元数据到具体类型/泛型实例和自动C-compatible shim；闭合String/Vec/Result/borrow/析构、panic、线程与future边界。 |
| T-12 | AC-10 | V-12 | 接AV/AC/新A2R/新A2C到同一Foreign IR；native机器码调用，A2R等价crate适配需独立证明；各端记录真实producer及dispatch。 |
| T-13 | AC-11 | V-13 | 闭合双向import/export、适用静态/动态链接、callback重入与generation/lease；接future完成/取消端口，保原allocator与跨ABI错误语义，交968调度。 |
| T-14 | AC-12 | V-14 | 使header/crate/toolchain/target/provider/ABI/生成器的真实读取依赖进入现有store，输出binding/shim/runtime分发清单和编译/运行工具分账合同给978。 |
| T-15 | AC-13 | V-15 | 用钉版本真实SQLite和serde_json原工程及完整特征fixture做用户体验与资源/失败回归；汇总四端、独立oracle、缺环境和交978/985材料。 |

### F0：首个UI Hello之前的通用AV/Rust FFI切片（当前可执行入口）

F0最小前置为已交938/939契约与现有text/scalar源码、共享verify/AV/MIR、现有artifact store；不使用新用户泛型/嵌套集合，因而不等962/963整包。T-F0-I自行核这些正式来源并有界比较至多两种Rust公共元数据/具体函数实例→自动shim策略。旧auto-bindgen预置C表不当作Rust导入器；不手写每个函数manifest、假metadata或借UI专用native数字ID通过。首次只支持冻结的一组非泛型Rust公开自由函数：借用UTF-8 str输入、i32/bool结果与结构化失败；泛型/String/Vec/Result/owned句柄/future/全部真实库仍由原T-09..15关闭。

源调用经原计划公共use.rs前门、Rust元数据/版本化注解生成binding和C-compatible shim，再由唯一verify得到签名/borrow/权限/效果事实，AV消费通用foreign指令与stub。物理边界用目标C ABI的data+byte_len，不跨界暴露Rust str胖指针/String/Vec/trait对象布局；shim内部调用Rust &str函数。先覆盖同步、同线程、noescape借用；宿主若要持久显示文本须在调用返回前复制为自身拥有数据，不能保留AV指针。AV执行Rust函数字节码并非本目标，Iced仍由rustc编译。

拟永久Rust provider与Auto调用源位于本包既有abi/av测试owner内（T-F0-I创建，具体模块/target与公共use.rs实际拼写先冻结）；Rust importer新增模块拟`crates/auto-bindgen/src/rust_import.rs`，复用foreign-contract/ABI/store，不另造FFI世界。generic positive API应能观察输入字节长度/返回值，至少含独立Rust结果对照与非UIconsumer；UI provider函数由980持有，仅消费同一通用入口。

F0资源域：单调用UTF-8最多65536字节、单session保留输出最多1048576字节（本切片候选上限，原域不扩）；分配/复制前核准，合法边界与超一字节分别验证。错签名/ABI/provider代次/缺权限在dispatch前拒绝且callee_count=0；合法空串/中文/嵌NUL按长度传递。Rust panic在边界转独立宿主失败，不允许unwind穿AV或当作正常i32；abort保受控进程失败而非语言负例。session销毁使旧provider授权失效，加载后的函数与库owner保活到调用结束；不承诺动态热卸载或跨线程回调。

A2R并行腿归原T-12/AC-10的首个简单签名切片：复用F0的自动binding/共享事实与现有a2x-rust/store，把目前a2r.foreign-unsupported的该有限形态接到生成Rust adapter或C兼容shim，真实编译并调用同一Rust函数；独立证明参数/错误/borrow与AV同合同，禁AV/旧VM解释provider仍执行。该腿按已有phase fold独立复核/正式落主后交980/RUST，不等T-12所有四端或967全包。V-AXR0-RUST是原T-12的必需证据，F0-R/D只要求AV的V-F0-*，AV首窗口不反等A2R；完整T-12和M2-A仍须全原四端/资源义务。验证使用已有构建consumer，不反等978/B0。

F0不依979/980最终结果或969/978新入口：用现有源码编译与AV执行API自证，供969/E0和980/HELLO消费；GUI初始化/文字投影不进967。新增T-F0-I/V/R/D是原T-09/11/12/13/14所需接口的先交实现与证据，原任务后续复用并扩全范围，不重复实现、不提前勾整项。T-F0-D正式交接后下游可消费，不等967整Plan归档。

早期切片是本Plan M2-A内部交接，不新增ACE里程碑或Plan终态；只在同一权威树按owner流程独立复核、正式落主并记source/接口/grammar/ABI/provider及真实结果后供下游消费。同Plan尚有原完整义务时不归档/删树/勾整包完成。frontmatter保最终依赖并集，早期切片只等本节最小接口；同一文件单写，不以分支reviewed/WIP当交付。最小切片通过不关闭原AC、完整Rust FFI/全UI/RG-A03/M2A-08；不新增对980/981/985整包或AGE联合门的反向依赖。

### 默认自有标准库与FFI边界（2026-11-12当前）

依据[Design56 §0.2](../design/56-ace-m2-ffi-and-native-stdlib.md#02-默认auto系统库可选ffi与字符串边界2026-11-12当前)。完整FFI扩面时分别处理ptr+len、CStr和Rust String/borrow/owned入出参；只在需CStr的边界转换/终止，明确内嵌NUL拒绝/显式策略、有界回读和正确allocator销毁。不得暴露Rust私有布局或把C ABI等同libc。F0保原长度借用域，其早期非UI验证/Hello不等待完整C importer。

本包验收：空串/中文/ab\0cd五字节在长度接口原样往返；CStr不适用内嵌NUL时具名拒绝，越界/无长度/错allocator/unwind控制拒绝且资源正确回收；四端各对独立expected。 原任务T-10,T-11,T-12,T-13,T-15承接；不新增实现任务，不以新义务扩大M1/F0或解除945–948的M1-TAG。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r8当前）

**具体输入与期望。** 原件[crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.at](../../crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.at)（下列节选L3–L8；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
use.c <string.h>

fn main() {
    let len = strlen(c"hello")
    print(len)
}
```

本包所需独立结果：旧显式use.c strlen(c"hello")需求为5；新F0非UI Rust输入A中😀加NUL的长度9/空0必须真实导入/生成shim/调用，不让strlen截断定义Auto字符串。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 公共C/Rust导入源/真实metadata→自动binding/C兼容shim→共享foreign事实→四端adapter→真实外部callee。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct ForeignCallContract { signature: SigRef, abi: TargetAbi, ownership: BorrowContract, effects: EffectSet, provider: ProviderId }
struct ProviderLease { generation: Generation, owner: OwnerId, grants: GrantSet }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [crates/auto-bindgen/src/extractor.rs](../../crates/auto-bindgen/src/extractor.rs) | 现有预置C表，不能冒完整Rust importer |
| [experimental/ace/crates/foreign-contract/src/typed.rs](../../experimental/ace/crates/foreign-contract/src/typed.rs) | 共享签名/资源/borrow事实 |
| [experimental/ace/crates/abi](../../experimental/ace/crates/abi) | 独立ABI oracle及目标适配 |

**状态、错误与资源。** 错签名/代次/授权/ABI先拒callee=0；泛型/String/Vec/Result/panic/callback/future按原完整FFI集逐项关闭，不能F0小例冒全库。 借用返回前宿主复制，allocator/deleter匹配，在途lease保活，cancel/late/最后owner不越代；默认Auto路径不隐式libc。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.at](../../crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.at) | 显式外库CStr互操作，不是默认libc依赖 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `4af83cfa91c6ba9c` | 拟ace.m2.p967.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [crates/auto-lang/test/ffi_dual/020_dep_traits_generics/input.at](../../crates/auto-lang/test/ffi_dual/020_dep_traits_generics/input.at) | 真实Rust依赖/trait泛型需求 | `expected_output.txt`；SHA256前16 `4534b2dfd189cc03` | 拟ace.m2.p967.source02：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [docs/design/41-interface-metadata-and-foreign-abi.md](../design/41-interface-metadata-and-foreign-abi.md) | 复用已有ABI/shim研究，不另立FFI世界 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `ef91fcbaaa7a60e1` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 真实metadata策略/注解边界、C/Rust工具依赖、callback/future和四端签名选择先冻结，保原FFI-M2-01..08与F0有限界限。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-09`、`T-10`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-host.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-03 | modify | `docs/specs/auto-bindgen/project.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07/AC-08/AC-09/AC-10/AC-11/AC-12/AC-13 |
| SD-04 | modify | `docs/specs/auto-abi/project.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07/AC-08/AC-09/AC-10/AC-11/AC-12/AC-13 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-host.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-HELLO | add | docs/specs/ace-migration/m2-host.md（沿原owner） | 未有早期交接→本切片实际已验接口/源/拒绝及剩余完整义务；交付后沉淀 | 最小Hello不冒全包完成 | AC-F0/AC-F0-REVIEW/AC-F0-LAND |
| SD-NATIVE | add | `docs/specs/ace-migration/m2-host.md`（沿原owner，仅真实交付后沉淀） | 默认实现选择未明→自有Auto/OS默认与显式FFI分账、长度字符串/目标ABI及来源验证 | LIB-M2-07/08 | AC-NATIVE |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-host.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 跨会话/过期handle及错误签名调用前拒绝 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 合法非零handle成功 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | cancel后late callback不能访问已关会话 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 原方destroy一次 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | 重入不得持锁死锁 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-06 | AC-02/03 | 未授权恢复dispatch=0 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t ace-host ace-abi ace-foreign-contract ace-av ace-native ace-a2x ffi`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 真实独立C caller调用ACE导出，ACE调用C provider各验证资源释放；卸库前callback pin仍保活；过期generation或错误ABI在执行前拒绝。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 2026-11-09 新增验证（待执行）

| Case | AC | 输入与区分性断言 | 入口/证据 | owner/阶段 |
|---|---|---|---|---|
| V-09 | AC-07 | 删除手写manifest/extern源码仍可从原工程生成binding并运行；不存在库、缺元数据/工具、歧义符号及错所有权注解在正确阶段具名拒绝。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-09；最终验证汇总 |
| V-10 | AC-08 | 改变传递header/宏/packing或target产生正确binding；独立C caller/callee验证聚合传参/返回/写回/释放；缺include/错误调用约定拒绝，static inline/常量真实消费。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-10；最终验证汇总 |
| V-11 | AC-09 | serde_json公开API自动导入并处理成功/坏JSON；独立Rust测试对值与错误；自有泛型/owned/drop计数fixture验证一次释放、借用拒绝、panic转换/隔离及不可跨线程对象拒绝。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-11；最终验证汇总 |
| V-12 | AC-10 | 相同C/Rust工程四端实际调用并正确写回/返回/释放；拒绝旧VM/AST fallback和错误签名；分别观测各端dispatch而非仅比较结果文件。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-12；最终验证汇总 |
| V-13 | AC-11 | 独立C caller调用ACE导出并反向调用；错ABI/过期generation在调用前拒绝；callback重入无锁死，cancel后late completion不发布、drop一次，在途卸库被阻/延迟，合法解除后可卸载。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-13；最终验证汇总 |
| V-14 | AC-12 | 分别修改传递header、Cargo.lock/features/cfg、target、ABI或工具链应失效；改无关Auto body不重做importer；坏缓存/生成失败保旧，修复恢复；异根可复现身份且不依赖开发绝对路径。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-14；最终验证汇总 |
| V-15 | AC-13 | 从干净工程仅配置依赖和公共use导入，自动生成并实际读写SQLite/解析JSON；去掉生成缓存重建仍成功；错版本、缺库、非法参数、取消及资源退出独立断言，无手工签名替代。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-15；最终验证汇总 |

新增触面职责组：`cargo t auto-bindgen ace-source ace-semantics ace-artifacts ace-foreign-contract ace-abi ace-av ace-native ace-a2x ace-a2c-abi`，与原§6组按实际修改取必要并集；T-01核真实参数，新增用例接固定组，不减原平台/ignore/压力条件。948若仅文档审计则只做目标静态检查，真实探针沿原指定组；安装/外库工具和外仓产品必须另核真实命令，语言组不替代它们。零选择/未执行/缺环境不充PASS；本轮没有运行这些测试。

### F0永久验证（由T-F0-I创建，当前未执行）

| Case | AC | 输入/入口及独立关闭oracle | owner/边界 |
|---|---|---|---|
| V-F0-SOURCE | AC-F0 | 普通Auto use.rs源→自动Rust元数据/binding/shim→Checked→AV真调；删手填manifest仍可从冻结Rust源生成。`A中😀\0`按UTF-8长度9、空串0，与独立Rust测试及字节oracle一致；至少另一个非UIconsumer复用 | work；非泛型借用UTF-8，非完整Rust importer |
| V-AXR0-RUST | AC-10 | 同一普通Auto import源经新A2R真实生成/编译/调用Rust函数，输入字节9/0与独立Rust结果一致；禁AV/旧VM执行provider仍运行，错签名/ABI同样拒绝且无dispatch | work/T-12；并行AXR0腿，拟职责组补ace-a2x及真实受影响native，emit-only不PASS |
| V-F0-REJECT | AC-F0 | 错签名/ABI/授权/provider代次/非法UTF-8在准入拒绝，callee_count=0；合法非零身份仍接受；panic/缺工具/abort分别保持host/infra失败 | work；不把timeout当语言诊断 |
| V-F0-LIFE | AC-F0 | 65536/65537字节、保留账边界；调用后guest文本释放/替换而宿主已复制值保持正确，20轮创建/调用/关闭无owner增长；失败保旧、修复后同入口下一次正常 | work；不承诺异步/热卸载 |
| V-F0-IDENTITY | AC-F0 | 改Rust函数/feature/shim/ABI身份使应失效产物重建；普通Auto body修改不重编未变provider，真实dispatch/计数与失败修复可核 | work；T-F0-I冻结新case/现有target参数，拟职责组cargo t ace-source ace-semantics ace-foreign-contract ace-abi ace-av ace-artifacts，实际修改a2x/native才补原组 |


### 默认库/外库与字符串验证（新增、未执行）

| Case | AC | 实际入口/输入 | 独立结果 |
|---|---|---|---|
| V-NATIVE | AC-NATIVE | T-10,T-11,T-12,T-13,T-15，沿本包既定四端/固定职责组与真实owner入口补永久case；默认provider与显式外库各一腿；源码/符号/链接/运行依赖与NUL边界对照 | 空串/中文/ab\0cd五字节在长度接口原样往返；CStr不适用内嵌NUL时具名拒绝，越界/无长度/错allocator/unwind控制拒绝且资源正确回收；四端各对独立expected。 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 旧显式use.c strlen(c"hello")需求为5；新F0非UI Rust输入A中😀加NUL的长度9/空0必须真实导入/生成shim/调用，不让strlen截断定义Auto字符串。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 错签名/代次/授权/ABI先拒callee=0；泛型/String/Vec/Result/panic/callback/future按原完整FFI集逐项关闭，不能F0小例冒全库。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 借用返回前宿主复制，allocator/deleter匹配，在途lease保活，cancel/late/最后owner不越代；默认Auto路径不隐式libc。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 双向ABI与异步完成端口的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-07 | work | FFI-M2-01自动公共入口可用，用户不手抄签名/布局，依赖和不确定所有权具名诊断；全部批准特征有关闭owner。 | T-09/V-09；原最终work验证和独立review同时核本项 |
| AC-08 | work | FFI-M2-02依独立C ABI oracle闭合，不能regex抽签名/宿主布局冒目标；required特征缺口阻本包。 | T-10/V-10；原最终work验证和独立review同时核本项 |
| AC-09 | work | FFI-M2-03不调用任意Rust私有ABI；具体实例与资源约束可追溯；不能要求用户为每个库手写ABI adapter。 | T-11/V-11；原最终work验证和独立review同时核本项 |
| AC-10 | work | FFI-M2-04四端公共源码实际执行并满足独立值/效果/资源oracle，不以emit或新语言源码等同库FFI。 | T-12/V-12；原最终work验证和独立review同时核本项 |
| AC-11 | work | FFI-M2-05/06及原AC-06共同满足；在途callback/析构/future保活，卸载无悬空/双释放，967不等待968完整await。 | T-13/V-13；原最终work验证和独立review同时核本项 |
| AC-12 | work | FFI-M2-07精确失效、失败保旧/恢复和生成物身份闭合，不存地址/授权，无新全局缓存。 | T-14/V-14；原最终work验证和独立review同时核本项 |
| AC-13 | work | FFI-M2-08真实库自动体验及全部required子行关闭；最低样本不能抵掉旧消费者/特征缺口。 | T-15/V-15；原最终work验证和独立review同时核本项 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |
| AC-F0 | work | F0自动源码导入/共享受检FFI/AV真实Rust调用及拒绝、文本寿命/费用/恢复闭环 | T-F0-I/V；V-F0-* |
| AC-F0-REVIEW | review | 同树独立复核F0普通源码、真实shim/ABI/dispatch及全部正反控，绑定revision/SHA | T-F0-R |
| AC-F0-LAND | merge | F0已审接口正式落主并交source/ABI/provider/测试来源，原完整FFI义务保留 | T-F0-D |
| AC-NATIVE | work | 空串/中文/ab\0cd五字节在长度接口原样往返；CStr不适用内嵌NUL时具名拒绝，越界/无长度/错allocator/unwind控制拒绝且资源正确回收；四端各对独立expected。 | T-10,T-11,T-12,T-13,T-15 / V-NATIVE；原独立review核新增义务 |

## 8. 执行步骤

当前共19任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

早期任务可按§5最小前置先行，不要求T-01完整分母冻结已完成；各I自行冻结本切片输入/接口/命令，不提前勾T-01或其它原整项。以下行序不是额外前置。

- [ ] **T-F0-I｜M2-A｜owner_stage: work｜AC-F0**：不等T-01全分母；按§5核已交基础、决定公共元数据策略/支持域/调用错误合同，打通通用AV foreign与自动Rust binding/shim，创建V-F0-*及实际命令。

- [ ] **T-F0-V｜M2-A｜owner_stage: work｜AC-F0**：T-F0-I后运行全部V-F0-*与冻结完整受影响组/必要消费者，记录独立Rust oracle、dispatch、权限/资源/工具身份及恢复，未执行不PASS。

- [ ] **T-F0-R｜M2-A｜owner_stage: review｜AC-F0-REVIEW**：T-F0-V的AV全部V-F0-*齐后停写，同一树独立审计F0源码→真实Rust调用/资源/负控；finding返I；只pass本切片。

- [ ] **T-F0-D｜M2-A｜owner_stage: merge｜AC-F0-LAND**：T-F0-R pass后按既有phase fold正式合入并沉淀已验F0接口，交969/E0、978/B0和980/HELLO；保同树/resync继续全部原任务，不归档或冒完整FFI已交。



- [ ] **T-01｜M2-A｜work｜AC-01/AC-PHASE**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。

- [ ] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。

- [ ] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。

- [ ] **T-08｜M2-A｜owner_stage: work｜AC-06**：分import/export、调用方/被调用方、标量/聚合/资源与C/Rust/Python现用桥，核动态库装卸、原方allocator、符号/ABI版本。异步本包只交注册/完成/取消端口及受控真实provider；968消费端口实现语言await，967不反向依赖968。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。

- [ ] **T-09｜M2-A｜owner_stage: work｜AC-07**：接通工程use.c/use.rs到共享绑定和typed foreign事实，冻结真实库/全特征required表、来源/注解/工具链和合法/非法公共源码；先走通一个真实导入与失败恢复。 验证V-09，证据汇入原最终work验证。

- [ ] **T-10｜M2-A｜owner_stage: work｜AC-08/AC-NATIVE**：实现C真实预处理/头文件闭包、target/sysroot/宏/typedef和ABI布局、函数/record/enum/pointer/callback/opaque、常量/inline shim；将旧用法特征逐行闭合。 验证V-10，证据汇入原最终work验证。 新增V-NATIVE：完整FFI扩面时分别处理ptr+len、CStr和Rust String/borrow/owned入出参；只在需CStr的边界转换/终止，明确内嵌NUL拒绝/显式策略、有界回读和正确allocator销毁。不得暴露Rust私有布局或把C ABI等同libc。F0保原长度借用域，其早期非UI验证/Hello不等待完整C importer。

- [ ] **T-11｜M2-A｜owner_stage: work｜AC-09/AC-NATIVE**：实现Rust公开元数据到具体类型/泛型实例和自动C-compatible shim；闭合String/Vec/Result/borrow/析构、panic、线程与future边界。 验证V-11，证据汇入原最终work验证。 新增V-NATIVE：完整FFI扩面时分别处理ptr+len、CStr和Rust String/borrow/owned入出参；只在需CStr的边界转换/终止，明确内嵌NUL拒绝/显式策略、有界回读和正确allocator销毁。不得暴露Rust私有布局或把C ABI等同libc。F0保原长度借用域，其早期非UI验证/Hello不等待完整C importer。

- [ ] **T-12｜M2-A｜owner_stage: work｜AC-10/AC-NATIVE**：接AV/AC/新A2R/新A2C到同一Foreign IR；native机器码调用，A2R等价crate适配需独立证明；各端记录真实producer及dispatch。 验证V-12，证据汇入原最终work验证。 新增V-NATIVE：完整FFI扩面时分别处理ptr+len、CStr和Rust String/borrow/owned入出参；只在需CStr的边界转换/终止，明确内嵌NUL拒绝/显式策略、有界回读和正确allocator销毁。不得暴露Rust私有布局或把C ABI等同libc。F0保原长度借用域，其早期非UI验证/Hello不等待完整C importer。

- [ ] **T-13｜M2-A｜owner_stage: work｜AC-11/AC-NATIVE**：闭合双向import/export、适用静态/动态链接、callback重入与generation/lease；接future完成/取消端口，保原allocator与跨ABI错误语义，交968调度。 验证V-13，证据汇入原最终work验证。 新增V-NATIVE：完整FFI扩面时分别处理ptr+len、CStr和Rust String/borrow/owned入出参；只在需CStr的边界转换/终止，明确内嵌NUL拒绝/显式策略、有界回读和正确allocator销毁。不得暴露Rust私有布局或把C ABI等同libc。F0保原长度借用域，其早期非UI验证/Hello不等待完整C importer。

- [ ] **T-14｜M2-A｜owner_stage: work｜AC-12**：使header/crate/toolchain/target/provider/ABI/生成器的真实读取依赖进入现有store，输出binding/shim/runtime分发清单和编译/运行工具分账合同给978。 验证V-14，证据汇入原最终work验证。

- [ ] **T-15｜M2-A｜owner_stage: work｜AC-13/AC-NATIVE**：用钉版本真实SQLite和serde_json原工程及完整特征fixture做用户体验与资源/失败回归；汇总四端、独立oracle、缺环境和交978/985材料。 验证V-15，证据汇入原最终work验证。 新增V-NATIVE：完整FFI扩面时分别处理ptr+len、CStr和Rust String/borrow/owned入出参；只在需CStr的边界转换/终止，明确内嵌NUL拒绝/显式策略、有界回读和正确allocator销毁。不得暴露Rust私有布局或把C ABI等同libc。F0保原长度借用域，其早期非UI验证/Hello不等待完整C importer。

- [ ] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。

- [ ] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [ ] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [ ] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-967/auto-lang`，分支 `plan-967-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-967/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-09/T-10/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-12 stage:new / PLAN-967 r8：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原19任务/全部AC与V/依赖/阶段/预算和drafting 0/19；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-12 stage=new / r7：按用户标准库/FFI方向补LIB-M2-07/08本包义务，绑定原任务与新增AC/V/SD；任务ID/总数、原AC与阶段门保留，drafting零进度。只修改规划，未实现/运行/签章或改变M1。

2026-11-11 / stage:new / PLAN-967:r6 / outcome:pass（早期Hello路线合同可审阅）；blocked（尚未授权实施，真实能力/接口待交） / next:按§5最小前置进入新增切片；原完整M2任务与终验均保留，不用小例替全包。新增T-F0-I, T-F0-V, T-F0-R, T-F0-D，当前0/19。未实现/构建/运行或派agent。

2026-11-11 规划静态核验（非产品/独立实现review）：12份本轮文档、6份Plan共106任务；对照本轮前快照，原86任务ID及原AC/V全部保留，新增20任务、全为未执行，drafting/current_step=0。0–10章节、total_steps、阶段清单与AC任务映射一致；323处本地链接已检查，新增链接/4处新锚点无失效、无新增行尾空白。跨四仓可达207份Plan的显式依赖无缺失/环，8个早期交接节点无环、AV不反等A2R或完整B；12个拟职责组名存在。6个具名源码SHA与本轮读取值相同；语言/UI/内容计数器仍1006/081/032。未新取号、建树、改产品/测试资产或canonical Specs，未构建/运行/签阶段/merge/tag。此检查不证明拟议接口已经实现。

2026-11-10 stage=new / r5：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部15个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r4：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/15；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-09 r3范围修订：新增T-09..15、AC-07/AC-08/AC-09/AC-10/AC-11/AC-12/AC-13及V-09..15，按Design56正式接M2 FFI/新版标准库责任，0/15。stage=new；仅文档合同与静态预检，非实施授权、产品验证或独立实现复审PASS。旧r1/r2/r3记录为历史。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
