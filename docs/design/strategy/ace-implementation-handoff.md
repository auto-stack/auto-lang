# ACE实现路线：决策与续接基线

更新时间：2026-11-03（PLAN-922 交付短窗）。用于长会话结束后的上下文恢复；[ACE总路线](auto-native-backend-evolution.md)仍为统一入口，current Spec为现状知识，具体Plan和revision绑定的review/merge收据决定交付。**当前最高优先线是 M2 全替换（v0.6 硬门）**：合同见[M2 r2](../../plans/reports/ace-replacement-first-m2-contract-2026-11-02.md)与[发布门增补](../../plans/reports/ace-v06-m2-release-gate-2026-11-02.md)，全场景分母/能力分账/后继拆包/关键路径见[M2 替换盘点](../../plans/reports/ace-m2-replacement-inventory-2026-11-02.md)，盘点口径见[replacement-scope-and-evidence](../../specs/ace-migration/replacement-scope-and-evidence.md)。下列状态是读取时快照，接手时须重新核对活动目录与`docs/plans/archive/`，不据本文替代验收或重开已归档计划。

**D871后规划更新**：871已交付87dd2b6ad、清理收据5af8e0cbe。native第二阶段已决定拆两包：[889集合/变体源码闭环](../../plans/889-native-collections-variants-source-closure.md)已立draft r1，typed entry另包未取号。889 N主体可与865准备并行，但集合现有源码会话整源重编后才carry，本包真单体前端必须取S；默认865释放后给889短窗，未就绪让863先行，H另交887/888。不得据下方较早快照再立同名包或认定889整项纯N。

## 1. 本会话收敛的架构决定

| 主题 | 已确认方向及边界 | 正式合同 |
|---|---|---|
| 名称与实现主体 | ACE是统一语言引擎；AC编译、AV解释、AJ按需编译/优化编译，A2X负责源码转译。当前Rust实现；Auto主体AACE及AAC/AAV子线延后v0.7，先满足Rust版全示例成熟度 | [Design43](../43-ac-av-typed-mir-and-abc.md)、[版本合同](../../plans/reports/ace-milestones-version-boundary-2026-10-24.md) |
| 共享执行语义 | source/CST/AST→解析/推导/实例化与CTE查询→Checked HIR；共享Typed MIR分别供ABC→AV及Cranelift→native。最小Typed MIR属于v0.6，已不采用“整个MIR等v0.7”或“新VM长期直接HIR→ABC”的旧方向 | Design43 §2–5、[Design49](../49-native-single-mir-lowering.md) |
| 转译与优化 | A2X主要消费结构化Typed/Checked HIR，不强迫从低层CFG恢复源语言结构。必要lowering/verifier/清理属于正确性；大规模优化、SSA/Machine IR、自主后端按收益后续实施。LLVM约80%/少量高收益pass是待基准冻结的研究目标，5%不是精确pass数量承诺；先实测现有Cranelift，不重复实现无收益优化 | [后端战略§5](auto-native-backend-evolution.md#5-v07按收益选择优化按需引入低层ir) |
| CTE、泛型与meta | 语义查询调度经受检片段→MIR→ABC→AV→portable冻结结果→推导/HIR再验证。不可变语义事实、一次推导工作态、选择性runtime反射投影分开；不让Codegen/可变Universe拥有权威类型事实。完整泛型、类型/声明生成、双期反射仍需逐项扩面 | [Design42](../42-semantic-metadata-inference-and-comptime.md) |
| 语法平台 | Atom/Batom+Schema配置语言包，编译解析表/专用scanner或代码；不强制热路径解释动态Atom树。通用无损增量CST、语言AST投影、Checked AutoHIR分层。旧实现提供语法案例和可证明复用的算法，不继承旧AST/ABC/类型布局兼容假设 | [Design39](../39-ac-incremental-syntax-and-language-platform.md) |
| 内存与全局cache | 热点有界常驻、冷数据分块持久化/按需装入，不能默认加载整个仓库/依赖AST。各层身份/依赖/Schema/codec及预算统一；typed view、紧凑struct/enum、Atom调试格式按用途选择，不把所有内存结构或原生对象文件强制变成通用Node | [Design40](../40-compiler-artifacts-and-cache-lifecycle.md) |
| ABI/FFI | 语义类型、AutoABI、目标foreign ABI、wire与执行布局分域。有限AutoABI尽早冻结验证，C双向边界先行，Rust适配优先明确C-shim边界；Python/完整混合生态另有支持矩阵，不假设Rust私有ABI长期稳定 | [Design41](../41-interface-metadata-and-foreign-abi.md) |
| 分层执行与增量发布 | AV开发→热点AJ/优化AJ是目标；部署固定完整快照/profile，校验及复用可重定位产物，补编未执行冷代码/不兼容单元，再链接/打包。不是转储JIT地址，也不因源码相同就复用dev机器码；无须每次全量冷重编，可信冷构建仍可作验证 | Design43 §9–10、Design40 §3.1 |

CTE缓存命中要求**完整实际依赖**不变：除调用点源码外还包括helper body、类型/候选集合、target/layout、配置、VM/profile与受控外部输入。结果属于A05，类型/签名/推导/实例事实属于A06；求解器可变状态、活VM堆和信任凭证不持久化。未知副作用不能共享结果；取消/超限不缓存为语言非法。普通runtime程序仍实际运行，编译cache不变成任意程序结果cache。

全局存储沿已有store/CAS、索引和预算合同演进；Batom读优化或其他冻结块须实测并验证边界后才能view/mmap，不承诺任意Batom零拷贝执行。required-data强引用、失效依赖与provenance分开；自动清理覆盖数据及索引、WAL/staging/orphan等总占用，保护用户交付物和活lease。分层数据库/pack方案是待验证机制，不能以“缓存已设计”宣称整个健康治理产品完成。

## 2. 成熟度与迁移边界

M1为有限共同子集/核心链路与可审计对等基线（含新增C组合门）。**M2 定义已被 2026-11-02 合同 r2 取代：补全 ACE 能力并替换旧引擎（AutoVM/A2X），覆盖 AutoUI/AutoMan 等全部现有应用场景，旧实现彻底 disable/退役；原 M4 的迁移/部署/回退/退役职责完整并入 M2，不再有"等 M4 才 disable"的后门**；M3/M4 如续用编号范围另行确定，未来新增扩展当前不规划、不作为 M2/v0.6 前置。v0.6 发布谓词=既有版本门 AND V06-ACE-01..05（M1+M2 全闭）；AACE 仍 v0.7，完整AJ不是旧引擎替换的自动前置。

“骨架成型”“四路径M1收口”“全语言”“产品默认切换/退役”“自举”分别验收。当前是M1共同子集收口与M2源码扩面交叉阶段，**863完成不自动签M1或M2**。M1适用规则沿[2026-10-28决策](../../plans/reports/ace-m1-scope-decision-2026-10-28.md)及862现行Spec的NA-4；accepted-diff保留旧差异证据，a2r金样/独立认证分轴。原始登记现为131数据行（corpus+scenario+console O 族；920 后 B18 avm 格已闭），仍有能力/接线缺口混记；50/103、20/73均为历史快照，不当当前clean数。先按[逐组对账](../../plans/reports/ace-m1-m2-closure-roadmap-2026-10-29.md)核实际证据/owner，完整复算后才签M1。native闭包/Dyn/loader的已批准NA不变，a2r名解析真缺口不豁免。

新引擎实验代码进入仓库、接入可选adapter、产品切默认是不同动作。旧引擎可继续承担生产；不要求把每项ACE新增能力再复制到旧引擎。共用语言/stdlib/UI接口与数据合同，后端适配按真实消费者另验，分支/仓库归并按实际owner与迁移Plan办理，不在此新冻结永久分支策略。G7按入口/SKU切默认，G8在所有保留消费者、旧会话/回调/lease清空后才退役；不得遇unsupported就静默回退旧引擎。旧A2X源码转译职责不能以AV替换为理由取消。

AutoUI/AutoGraphics与ACE按受检语言/宿主接口集成；初版旧VM适配不证明ACE能运行UI，也不自动要求新图形系统重做一套a2r实现。具体仓库、迁移和产品接线随其owner/Plan，不能把ACE实验crate独立推成与拆分、SDK和公共测试无冲突。

## 3. 当前已交付基础与仍有限的能力

| 基础 | 当前可认领范围 | 继续工作时的限制 |
|---|---|---|
| 783/787/788 | 统一产物/store探针、有限Atom语法驱动增量词法/CST、文件化原生测试及计量基础 | 非完整Auto通用生成器，不等于所有源码已改用787前端；分层cache产品仍按实际producer核实 |
| 796/797 | 有限语义事实/推导查询；Windows x64 AutoABI v0.1候选及Auto/C双向互调 | 非全语言反射/泛型；非全类型/平台FFI、Rust/Python产品接线 |
| 798/828 | 共享Typed MIR/ABC/Rust AV、函数级全链增量及prepare/commit/snapshot/pin/lease | 不是持续运行程序中的完整在线热替换；该缺口由866负责 |
| 817/818 | AV驱动标量CTE查询/冻结cache；标量MIR→CLIF→对象→原生对拍/单元cache | 主要i32/bool，不随AV扩类型自动变成复合CTE/native |
| 820/821 | data/owned具名profile、record/text/Place/move/borrow/清理的有限合同 | 不自动包含所有旧方法、完整语言、运行期句柄防护或native ABI |
| 822/823/824 | 集合/变体、拥有闭包、显式Dyn/portable冻结的typed执行合同 | 集合/闭包/Dyn typed fixture不等于源码语法/推导/lowering已接通；不自动扩CTE/native |
| 825/826/827/852 | 标量LoadedProgram优化、紧凑ABC、真实scalar源码基线、四路径文件化语料与a2r独立认证 | 不把执行/编码优化计作语言覆盖；852是分母/判定底座，不是所有能力已绿 |
| **868 r1** | **已归档，merge收据delivery `7754c4e4c`；共享lowering实际位于`experimental/auto-mir-lower`** | 原probe路径已移除，三消费者只读消费；不重做搬移，不扩868原合同 |
| **862/864** | 已归档：AV入口/Unit/尾值/脚本，集合/变体/is/有限List for-in原源；864 delivery `6273cd4a4` | 入口是AV-only，集合不自动扩native；完整Unit/迭代器/用户泛型另验 |
| **866** | 已归档，delivery `3b4080916`：真实AV常驻、安全点更新、在途守卫/延迟安装和有界回收 | 不再把在线AV缺口记为未交付；非native loader/任意活对象迁移 |
| **870/881** | 已归档，delivery分别`bc90ef718`/`db0773130`：mn data/owned、static/view方法与真方法函数增量；R08/R07 ac/av已闭 | 881不补文本通用eq/ne等非方法gap；typed宽数值/集合native/完整方法体系仍有缺口；871退役另验 |

上述为各有限合同的交付/现状归纳，不把“归档”单独作为全部收尾已完成的证明。824归档文件仍有T-08未勾选、最近merge条目记录已landed但清理/批回归待办；825亦有计数与delivered收据不齐的簿记现象。若新任务依赖其最终收尾，先核实际Spec/合入及遗漏收据，不篡改历史、不自动重开或重做旧实现。

## 4. 当前主任务、剩余缺口与开工边界（2026-11-03 刷新）

| 线 | 读取时状态 | 工作及下一步 |
|---|---|---|
| 862–920 已交付线 | 全 archived（865 闭包源码、863 console、887/888 测试治理、889 native 集合/变体、913 恢复、914 编辑承诺、916/917 a2r 语义/证据、915 资源 fault/Place、919 资源事件六格、920 B18） | 不重做；M1 corpus av 列基本闭合，legacy 差异行已登记（binding.tsv 分账见 M2 盘点 §2.1） |
| [921 CA-A](../../plans/921-shared-a2x-c-scalar-target.md) | executing r1，1/14（身份已冻结：a2x-c/auto-a2x-c/ac-a2c recipe） | HIR→C scalar 闭环；CA-B..E 后继沿其接缝；与 M2 线 H 面漂移按其 merge 规程处理 |
| [922 M2 盘点](../../plans/922-ace-m2-replacement-inventory-and-critical-path.md)（本短窗） | executing | 交付本报告+SD-01/02；后继拆包标签（M2-DEN/EXEC/HOST/A2X-*/UI/MAN/WEB/OS/RETIRE/REL 等）见盘点 §4，未取号 |
| 919 merge | 在途（reviewed→merge，lang-919 树） | 非本包事务；其合并后按 M2-DEN 规程刷新受影响行 |

S=共享语义/IR/runtime/session，N=ac-mn，H=公共driver/SDK/测试/binding，P=登记私有模块。已交付各线不再排重复实施。**当前开工顺序以 M2 盘点 §4/§6 为准**：M2-DEN ∥ M2-EXEC 可即启，M2-HOST 是 UI/WEB/OS 串行根，A2X-R/T1 并行，RETIRE/REL 汇尾；旧 2026-10-29 对账与[波次合同](../../plans/reports/ace-wave-handoff-2026-10-28.md)保留公共单写规则，顺序部分由 M2 盘点取代。

D门要求真实交付，不消费review/repair中的WIP，不提前合入未过审代码换取并行。前owner交接合入revision、Spec/API/profile/lowering版本、兼容结果、cache失效和限制，后owner记录接受；跨S/N/H同时取窗，不持一个窗口等另一个。新增公开枚举/签名需同步全部实际消费者的穷尽匹配，不把兼容编译失败留作他方债务。

871 **G-COV**列全core/scalar/text/record/text-record旧五域，含静态/只读方法、mut/move/clone、alias字段、Unicode、资源及全部协议/CLI/脚本/cache消费者；D870不是全集证明。**G-DELETE**要求全部覆盖/shadow/迁移及旧键恢复核验齐全；保留`text_resources.rs`的verify层`TextResourcePlan`及调用，只删旧native独占部分。

M1剩余按具名责任分组：native集合/变体与typed-entry、T03/T05共享文本真源、B17–19诊断/infra归因、场景trace/events/layout接线、资源fault/句柄代际、a2r解析/未定型发射/缺金样，以及完整分母适用性与证据归一。集合/变体现由889正式认领；typed entry及其余完整能力/接线/验收包仍未立项，先复用871等已交付producer证据再判真缺口；242只是a2r长期tracker，887不自动承担全组。865/863后先完整复算，再按剩余shared文本→诊断→场景/资源→a2r的默认优先级冻结有界包；N/R/H独立范围可提前并行。M2数值/模块/stdlib/泛型-meta-反射/复合CTE/FFI及RF-I/RF-C继续按真实缺口规划，不预占编号，不以完整AJ/发布器/block增量作为统一M1门。

拆分线867/869/872/873与图形线独立推进，只有实际SDK、语义或测试写面重叠时按owner交接。新会话先查真实状态，不因本快照启动、暂停或修改他方任务。

## 5. 首例即适用的验收和工程纪律

- 同一原始源及独立oracle，分别记录source、HIR/MIR、wire准入、AV/native执行、资源和真实消费者；不改expected、删import/分支或换fixture凑绿。新变体另编号，旧分母保留。
- fresh/hit/restart/body-edit、依赖/配置/权限改变、损坏缓存及强制退化反控从首例纳入。未改函数生产工作=0及IR/代码身份保持；全模块重算后某层cache命中不算全链增量。恢复decode+verify，不恢复trusted标记。
- 长期亚秒目标保持，**现开发期硬门P90≤2000ms**（PLAN-858及current Spec），P95/P99另报，代表负载/样本/结构与内存门不能删。T-02按上游预算交集冻结，拥有元素的clone/失败/清理、旧代回落和cache容量都要测，不自行扩大上限。
- 实施在每Plan唯一worktree，主检出仅计划/docs簿记；不建junction/symlink。保持他方未提交改动。开发/repair只跑定向检查；最终独立review后由merge在冻结提交跑裸cargo t及触面档，tf归主检出到期单实例；docs-only不跑cargo t/docs_gen。
- `attachments/**`原始源码/日志/JSON/截图证据只本地保留，禁止add/commit/push；Markdown摘要记录命令/退出码/数字/路径与hash。不自动新建会话或向其他会话发送消息；本次交接未启动施工。
