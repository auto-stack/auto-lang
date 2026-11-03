# ACE M2合同：补全ACE能力，并替换旧引擎AutoVM/A2X（2026-11-02）

> **v0.6发布绑定（2026-11-02用户裁定）**：本M2 REP-01..11全部闭合是v0.6正式发布必需，见[发布门增补](ace-v06-m2-release-gate-2026-11-02.md)。用户接受为替换调整进度（如晚一周），未承诺一周可完成；旧实现未disable、场景未接管或只到“就绪”时不得以双引擎/legacy兜底发布。本轮只绑定版本/优先级，不改变本功能门、在途Plan或实际引擎状态。

状态：target-state r2，用户已明确M2完成目标；本轮只更新设计/路线，未实施、切默认或disable代码。关联[版本合同](ace-milestones-version-boundary-2026-10-24.md)、[M1 A2C增补](ace-m1-a2c-scope-addendum-2026-11-02.md)、[ACE总路线](../../design/strategy/auto-native-backend-evolution.md)。

## 1. 唯一当前目标与取代关系

用户本轮明确M2目标为：“补全ACE能力，并替换旧引擎（AutoVM/A2X）”；M2完成时旧引擎代码全部由ACE接管并disable，覆盖AutoUI、AutoMan等旧引擎所有现有应用场景。M3/M4的未来新增扩展暂不规划、不作为M2前置。

本r2取代r1中“只到替换就绪”“旧M4继续承担最终退役”“M3生态并行”的当前执行表述：原M4的迁移、部署、回退与旧路径退役职责完整并入M2，不能在M2完成后再留一波disable。未来M3/M4如继续使用这些编号，范围另行确定，不再用它们承接未完成的旧引擎替换义务。

M1含新增A2C、已批准原语义/NA/accepted、源/示例/独立oracle/资源/周转门与v0.7 AACE边界保持。现有Plan按其批准合同继续，本文不擅自取消在途修复，也不把全M2追加进某个M1包；但原M1完成不自动证明M2完成。

## 2. M2完成的必要且同时满足条件

| 当前里程碑 | 目标 | 完成边界 |
|---|---|---|
| M1 | 核心框架/有限语言基线、A2R/A2C、增量/恢复/有限ABI与已批准门 | 按现行M1及C增补验收；期间开始全场景清单/真实工程对拍 |
| M2 | 补全ACE能力，并替换旧AutoVM/A2X | 原全部现有语言/转译/产品场景接管，所有入口和实际发行改走ACE，工程增量/热更新达标，旧实现彻底disable/退役，REP-01..11全闭 |
| M3/M4 | M2之后的新增能力扩展 | 当前不定义细目、不排实施、不作为替换前置；不再保留“等M4才disable”的后门 |

迁移期间可以按成熟入口显式opt-in/分批切默认；这只是M2实施进度。M2签章时不允许仍有“部分入口待迁”“保留legacy开关”“旧转译器继续兜底”或“产品装载还用旧发行物”。全部旧义务未接管/disable未证实则M2未完成。

“旧引擎disable”指旧执行/编译/AST转译实现及其注册、链接、安装、发行和实际消费者路径退出：默认及all-features构建、所有生产CLI/SDK/服务/工具模式、自动fallback和日常测试均不能启用旧执行器/转译器。能删除的旧实现删除，历史按Git记录追溯，不在新发行物中留可启用的备用旧引擎。薄兼容API可保旧名字但只能转发ACE，不能包装旧算法冒新后端。回滚使用独立已发布旧版本/安装备份，不靠新版内置legacy执行路径。

共享值类型、诊断、词法工具、中立runtime/native库、渲染器和依旧需要的Rust库不按“目录名含vm/trans”盲删；先归属与依赖核清、必要者迁至中立owner，禁止通过保留它们夹带旧执行循环/旧发射器。disable与删除以真实依赖/dispatch/产物为证，不以一个配置标志或文本搜索无结果为证。

## 3. 全部场景分母与应用接管

M2范围按旧引擎真实支持面冻结完整台账：source/工程/公开API/CLI/工具/宿主、后端/平台/features、库与权限、模式/配置、输入/独立oracle、迁移owner、新ACE入口、证据/状态。基线新增或变化显式登记，不删未支持/未执行行。不要只统计examples或一类UI运行模式；原不支持的新跨积不强造，但任何现有支持的场景不能自行归“未来生态”。

| 起始清单（需后续全调用点审计补齐，非本轮穷尽声明） | M2必须接管的行为 |
|---|---|
| AutoUI | 现有Vue/Web、VM/Iced等实际后端与模式；UI方言/数据绑定/store/handler/computed、事件与回调、资源/窗口/退出/错误及发布链，不能只首帧 |
| AutoMan | 实际配置/脚本/表达式求值、任务/命令/build/run/dev/generate、模板/生成器、项目模型/插件及当前语言API调用，不能只CLI空跑 |
| 语言CLI/SDK/嵌入消费者 | 文件/带参执行、配置求值、持久会话/REPL、daemon/client、测试工具、AutoGen、Web/API/服务等实际调用，含安装/仓外/重定位和错误通道 |
| 旧A2X全部现有目标 | Rust/C/Python/JavaScript/TypeScript/GDScript/Godot/TSCN等实际受支持入口/工程/依赖/产物；每个有对应ACE provider或中立适配实现，不得长期调用旧AST emitter |
| 现有stdlib与宿主/外库 | 当前实际使用的文件/网络/时间/任务/并发、UI/服务/系统、C/Rust/Python桥与资源/权限/回调/退出；即便过去标M3也纳入本M2接管 |
| 共用工具和研究引用 | 逐项审查旧engine/trans模块的调用与构建边，明确产品/测试/独立转换工具/历史研究归属；不借研究引用给新版保legacy运行后门，不自动扩成AACE自举 |

本轮只读代码提供清单种子：auto-lang facade仍公开eval_config_with_vm、run_with_session、run_file/run_file_with_args、run_with_mode，以及trans_rust_merged、trans_python/javascript/typescript/gdscript/tscn/godot等入口；core仍有vm/trans模块。r2a等逆向独立工具需按实际依赖分类，不因它位于trans目录就盲删或假称Auto→X已接管。这些API存在不等于当前全部已运行通过，完整消费者普查/独立原例认证由后续Plan认领。

AutoUI/AutoMan/Web等领域逻辑仍归各自仓/owner，ACE交付中立语言/执行/转译/宿主接口，不把UI/Man/Web实现搬回语言核心。跨仓消费者迁移必须有源/SDK/包/安装与实际consumer一致性收据，单仓内部模拟绿不能证明产品接管。

只补替换这些现有场景所需能力：可先明确typed host import/有限FFI/shim/运行时服务，不要求任意库零适配、完整新header importer、任意Rust crate自动桥、全新Python生态、所有ABI形态或完整AJ/OSR。若某形态已被当前应用使用，则它就是M2必需，不得凭“通用能力”名字排除。旧bug不自动定新规范，已有裁定与独立oracle继承，新增差异按正式规则处理，不改expected凑绿。

## 4. 日常快路径与Rust依赖

默认开发方向：预构建ACE/宿主与库 → Auto源编辑 → 常驻共享语义/真实跨模块依赖 → 函数级生产 → AV或明确支持入口的AC → 同宿主安装/执行。稳定宿主下Auto业务编辑不生成整套Rust项目并cargo重建宿主/SDK，不把全目标发布/认证塞进每次编辑。

v0.6引擎仍Rust；引擎、宿主、实际Rust库/shim源码或接口/工具链/flags/features/cfg/真实依赖变化仍可低频构建。高频Auto编辑与这些构建隔离不需要先实现AACE。Rust库消费预编译版本与已验证typed host/ABI/FFI边界，保模式/错误/原方释放/回调保活；不能为省构建复用已失效接口或忽略CT/meta/内联读取的body变化。

A2R/A2C以及其他目标保新ACE转译/发布职责，显式发布/认证仍调用对应工具链；旧A2X实现的退出不等于取消这些目标或禁止编译生成的Rust/C。Vue等现有目标的快路径按其真实产品模式验收，不强制所有产品变成同一个AV运行模式，也不隐式退回旧引擎。

## 5. M2替换与disable硬门（后续正式Plan逐项认领）

原REP-01..08保持稳定ID并加严整体完成边界，新增REP-09..11。所有门是目标合同，不是本轮执行PASS。

| Gate ID | 必须观察的结果 | 区分性控制与边界 |
|---|---|---|
| REP-01 原源与全消费者 | 全场景分母及对应新ACE入口/owner齐，原源/结果/诊断/副作用/资源/退出符合独立oracle | 不删import/分支/改expected、无旧引擎fallback；缺/未执行/部分入口不算全部接管 |
| REP-02 Rust构建隔离 | SDK/engine准备单列，稳定宿主下实际Auto body编辑/暖启动/兼容更新rustc/cargo0，Rust宿主/库不重建 | 真进程树/生产点而非wrapper计数；强制每次cargo/只报0反控红，发布/认证另列 |
| REP-03 全局实际依赖 | 真实多模块/stdlib/host工程，未改函数parse/resolve/typecheck/HIR/lower/codegen/load/每函数工件IO0、同进程对象代际保持；type/meta/实例/CT/权限及负依赖真实失效 | 跨工程/target/权限隔离，不能只hash调用点或whole后filter，不能单源微例冒全局完成 |
| REP-04 真实在线热更新 | 同宿主PID，兼容编辑新调用新结果、旧在途链旧结果/保活；failed/cancelled/late不覆盖，最后pin释放后有界回收 | 重启/重跑/暂停恢复不充在线更新；不兼容layout按现有模式要求安全拒绝/明确重启，不要求新任意状态迁移 |
| REP-05 恢复与工作集 | fresh/hit/restart/body-edit与修复后再恢复，持久数据重验、热点/磁盘/lease/GC有界 | P/R/M分账，无host指针/权威/PASS反序列化；不拿重编或删他方产物冒hit/回收 |
| REP-06 现有宿主完整性 | 全当前应用所需stdlib/host/FFI/权限与UI/Man/服务生命周期真实接通 | import/首帧/空命令/模型不替实际交互/任务/回调/退出；不因旧M3分类免除已有依赖 |
| REP-07 工程级周转 | 原工程/原负载取样P90≤2000ms、原更严门与正确性/结构/资源同过，长期亚秒目标保持 | 准备/冷编译/启动/语义/生产/安装/运行/清理/Rust构建/峰值分列，不缩工程/只报parser |
| REP-08 全入口实际切换 | 所有受管CLI/SDK/宿主/工具/安装/发行真实装载ACE，fresh/仓外/重定位/权限/错误及独立旧版本回滚可用 | 部分入口仅是进度；M2完成时无“待M4部署/切默认”，新版不含legacy备用引擎 |
| REP-09 A2X全目标接管 | 每个现有受支持转译目标/工程/API/产物都映射到新HIR provider或明确中立适配，实际输出与独立行为/金样规则验证 | 只A2R/A2C绿不充所有A2X；不得委托旧AST emitter或丢未迁目标，逆向/独立工具按真实归属审查 |
| REP-10 旧实现彻底disable | 源/依赖/feature/注册/工具/构建/链接/发行/运行图核对，default与all-features均无可用旧引擎；实际legacy dispatch/构建单元0 | 故意切回旧实现/保hidden开关/间接旧wrapper/打包旧binary/测试强拉legacy均红；不是只改默认flag或搜索目录名 |
| REP-11 产品及测试迁移收据 | AutoUI各已支持模式、AutoMan实际任务/生成/求值、其他全部场景在独立安装的ACE SDK/engine/runtime上运行；新测试/证书/登记接管旧义务 | 无dev worktree/未发布绝对path/本机旧binary/cache假fresh；M1旧ledger/旧报告只历史证明，新义务不再为日常测试编译旧引擎，迁移映射/全部consumer与退役证据可读回 |

M1的旧四路径登记/批准hash与历史证据不为disable而手改；M2通过版本化后继ledger/recipe/报告记录“原义务→新producer/source/profile/oracle”的迁移，保原源/行为判据和已有差异裁定。旧golden/证书不冒新producer身份，合法新目标文本经独立审阅另存；坏证据/缺证据不能转成NA或缓存PASS。disable后不要求新版日常再编译旧引擎对拍，历史结果与独立手写reference/行为oracle承担追溯，而不是长期保留旧实现。

## 6. 排序、交接与边界

1. 继续已授权M1及在途各包，M1新增C仍必需；旧引擎在途修复沉淀为语义/兼容证据，不因此默认长期保留其生产路径。
2. 优先冻结全场景/全目标/全入口及Rust重编高成本工程；语言、stdlib/宿主、全局增量、产品化安装/SDK与跨仓迁移按真实依赖并行，形成可验收的真实工程切片。
3. 分入口切默认；覆盖、实际接线、发布/安装及原职责测试迁移随功能一起完成，不能等语言实现结束再另留“产品化阶段”。
4. 全部场景接管后，完成旧执行/转译实现及注册/构建/发行/测试依赖退役，独立复审REP-01..11并读回主线与真实部署收据；全部门过才签M2。
5. M3/M4未来新增能力当前不排计划、不增替换门；M2原场景若仍缺功能/接线/disable则继续补M2，不能改标签延期。

本轮只更新合同/设计/路线，不分配Plan号或任务预算，不修改生产/current Spec/在途Plan/旧批准与登记，不构建测试/认证、不实际disable或删除旧代码。实现继续用专属worktree、独立review、冻结真实职责/consumer门和收据；跨仓按各owner单写交接，不要求把所有WIP堆一树。v0.6 Rust版ACE与v0.7 AACE边界保持，M2替换不以自举为前置。
