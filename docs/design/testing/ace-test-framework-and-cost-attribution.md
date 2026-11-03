# ACE测试接入与慢例成本归因

> **2026-11-02 M2替换并disable验收**：[REP-01..11](../../plans/reports/ace-replacement-first-m2-contract-2026-11-02.md)覆盖全部现有应用/目标/入口、工程Rust构建隔离/全局增量/同PID更新、实际发行切换与旧实现退出所有构建/调用路径；AutoUI/AutoMan须真场景，不只首帧/空命令。M3/M4未来新增扩展不规划，原迁移退役义务归M2。旧M1证据保历史，新测试/ledger迁移全部义务且不再编译旧引擎；不改变在途Plan或把普通单测都升级为性能矩阵。

> **2026-11-02新增目标合同**：用户已将A2C纳入M1，见[独立范围增补](../../plans/reports/ace-m1-a2c-scope-addendum-2026-11-02.md)、[C路线](../strategy/shared-a2x-and-hir-to-c-roadmap.md)及本文§7。893原13项与现固定组/原裁判保持，新增C/Q4不能通过改旧统计或隐式认证加入；尚未实现/执行的新增门保持pending。

> target-state r3 / 2026-11-01；实施合同：[PLAN-893 r3](../../plans/893-ace-test-framework-and-slow-case-attribution.md)。承接[Design35](../35-autodown-test-cases.md)及[当前固定组规范](../../specs/auto-test-case/module-selection.md)。896/898/901已交付，888目录/tm/自动影响闭包已退役；本轮只规划，不是新实现或验收证明。

## 1. 目标与单源边界

893合并13项慢测归因和未来ACE接入约定，保原oracle/样本/负载/条件/失败阻断，不承诺修五产品线，不缩512修订/4096深度/32–64MiB。产品算法、存储、ABI问题据实后继拆包，不预占号。完整归因包括每项真实成本、证据边界、未定位阶段/下一探针，不能只给分类模板。

`原source/typed/Rust测试 → 当前固定Cargo package/target/namespace → fixed-tests.py/原nextest/worker/TestCase → 原父侧判定/断言 → 只读成本摘要`保持单源。有限策略元数据不参与日常发现/选择，不建第二目录/runner/Job/meter/裁判。原计划拟ace_tests.py执行器取消，执行直接用cargo t；新ace_cost_report.py只读真实原command/终态/measurement及必要阶段标记。

## 2. 原13项守恒与当前入口

语义与oracle归原能力owner；package/target/完整test名+源/输入hash是当前可执行身份，旧888叶ID只作历史追溯。893 §4.2的K-01..13是一轮专题审计对象，不是运行前置叶目录。T-01真实发现/逐断言核迁名/拆分；两个828旧t07函数名当前静态未找到同名，不能直接换相似名、删成11项或假定已退役。缺失先查迁移/原断言及正式修复合同，保规模/GC/restart/sentinel义务。

原语义域syntax1/sema1/AV4/native3/CTE4；源路径已经迁至experimental/ace。旧历史秒数/hash仅线索，不能拿其认证新工具/profile，也不要求复活已清理raw。

## 3. 接入约定与视图

新增能力首例声明source/HIR/typed/MIR/ABC/native/emit/cert起点、profile/权限、独立oracle、source/tool身份、owner、适用条件、fresh/hit/restart/body-edit/恢复重验、未改函数真实生产0/居民身份、内存/cleanup预算。无据NA不准，typed不冒source；a2r日常emit/golden与显式两腿compile-run认证分轴，日常不隐式启动rustc。

| 视图 | 当前合同 |
|---|---|
| affected | 人工明确受影响完整职责组/必要消费者并集，review核理由，不建--changes自动图 |
| slow | 原13义务的当前原生导航/单项重放，重叠按固定driver同配置OR去重，缺项/未测不绿 |
| formal | 原能力入口/样本/阈值/资源负控，显式ace-performance及原脚本；单轮归因不充正式P90 |

原条件/ignore/平台及触发保持，不为本元数据新增默认日常入场。原生test-name导航只用于定位/归因，不替代最终完整职责组门。select-only是静态预览，不冒实际发现/通过；无新定时任务/批量worker/全目录三轮或A-B-C矩阵。

## 4. 观察、资源与真实性

复用原Job计CPU/树peak/输出/deadline/kill-reap，逐命令记录源/工具/flags/profile/环境。准备/冷编译、发现/fixture/init、请求工作、父比较、持久化/GC、复制及cleanup分别计；嵌套重叠不可求和冒总墙，局部峰不可相加冒同时树peak。无观测为UNOBSERVED+具体限制，不填0。适用预算从原合同冻结，caller更严取严，不抬限。

最小真链用一原慢原子+一已有文件化source，真实执行/原判定/成本关联及错oracle/取消/cleanup控制，再扩13项。实际fixture/比较延迟控制须反映到正确阶段和总窗，不能只改计数。每项当前有效单轮或严格同source/dependency/config/input/tool/observer复用；原红/干扰/重放同留。可交未定位产品根因，但不可未测冒完整归因。

仅保持原义务的harness准备/报告去重可同包做，须同身份对照/负控；应fresh/store/restart不能偷共享，不能移父比较/cleanup出计时窗。P90≤2000ms沿各原profile/正式样本，不给13项总墙套统一P90，也不以≤60秒强杀削负载。

## 5. 与893、899、902及903的交接

893负责共同接入与13项归因；[903](../../plans/903-ace-m1-net-gap-audit.md)负责M1逐格净缺口，二者不互等整包、不同任务不重复认领。899工具命名/profile改变后重新绑定身份/受影响成本；902占core受检资源/S和新Rust provider/P及注册H，本框架优先消费891/892/894正式真例，不消费902WIP。静态策略/报告可并行，观测/固定组/工具/注册按具体符号串行；正式慢采样和902认证/性能、899重建及903窄重验错峰。

交接有owner/revision、符号、正式SHA、schema/profile/helper/tool/flags身份、已跑/pending和回交，不持一窗等另一窗。原义务不依赖“等888原树/日志恢复”。

## 6. 阶段、材料与后继

T-01真实原生映射→最小真链→有限策略与只读归因→13项有效观察/原因边界→source/typed/892 emit接入验证→同树独立上下文review→冻结职责组门/规范沉淀/归档清理。角色未指定人/模型，不创建agent/chat或联系其他会话。

材料仅D:/autostack/plan-artifacts/plan-893/，开工登记owner/用途/在途；正常target留专属树。merge折叠命令/退出/数字/节选/hash进计划，核全部实际输出路径并删scope与树产物，具名未解决问题才hold。不把raw/receipts放项目attachments，不永久保留历史exe，不写legacy specs.json。上述均目标合同，未执行或已通过状态以实际Plan/收据为准。

## 7. A2C加入M1后的四端一致性接入（2026-11-02，target-state）

Q4=AC-native/AV/ACE-A2R-HIR/ACE-A2C-HIR；legacy AVM/A2R/A2C单列，不互冒producer。共同源/语义profile矩阵先冻结，每端真实source→producer→执行或显式编译运行→父oracle；缺项/未执行/infra/unsupported/规则排除和扩展域逐行显示。不能只比四端相等、用typed fixture替源码、以进程退出码认证宽位型，或把skip/零选择当PASS。

首例文件化与独立oracle从CA-A接入，CA-B/C/D各自扩域并完成资源/增量/ABI必要case；CA-E负责新增target ledger/组合报告/总门，不成为前包延后测试的接收者。已有13慢例不扩成新发现/调度系统，新C测试按实际package/target/namespace进入固定组，原条件/规模/ignored/平台/预算保留。只读成本摘要从真实测量取数，未运行不能产生产量0/资源0/周转PASS。

同源逐例核值/精确位型（按支持类型）、原输出字节、求值顺序/副作用、diagnostic语义族保raw事实、资源生命周期/失败清理；fresh/hit/restart/body-edit、原语料/手写reference及全编译/重造对象/错oracle/漏函数/旧版执行强制红控制常驻。源码生产P、恢复重验R、元数据M、实际编译器工作分账；C/Rust目标生成未改函数0不等编译器只重编一个函数。

日常A2R/A2C的emit+bytes/严格证书路径中，rustc/cargo/cl/clang/linker/目标Rust/C程序spawn=0；预构建worker可启动但不得自动build。显式认证现场actual全文件集与独立手写reference各自编译运行对oracle，绑定target/helper/backend/工具实际身份/flags/真实头库依赖与source/oracle/reference/hash；坏/缺/过期/跨目标/extra文件/重复key分别准入。新C必需认证不能沿旧A2R cert-gap豁免，证书不缓存PASS，日常漂移只报错/产proposal不覆盖golden。

“不缓存PASS”指不以旧TestCase判定替当前emit/父比较；完整身份未变的真实认证收据允许严格复用，日常继续重核文件/证书轴，不因此隐式编译或伪称本次guest已运行。

M1签章是原base-908门AND新增C语义/资源/增量/恢复/有限ABI/Q4/登记门。旧acceptance/2及其can_claim_m1只证明旧基面，新组合报告缺C/空目标/旧报告/任何必需未执行必须拒签；旧批准件/hash/313 required不改，新C来源/target ledger独立版本化，由正式CA-E Plan冻结其Spec delta与所有消费者。支持域已有正式P90≤2000ms/样本/资源门保持，原prepare/编译/认证/执行/清理及并发峰值分列；新增C工具不进入普通legacy测试隐式准备图。

## 8. M2全场景接管与旧实现disable（2026-11-02，target-state r2）

**v0.6发布绑定**：本节验收及REP-01..11现在全部是v0.6必需，见[发布门V06-ACE-01..05](../../plans/reports/ace-v06-m2-release-gate-2026-11-02.md)。正式发行须同时有M1含C、全部现有消费者/目标、真实工程快路径、旧实现退出及clean安装发行证据；不把局部测试绿或勾计划当发布通过。允许调整日期但不缩原负载/分母/资源门；一周只是容忍示例，需保留原规模回归/修复/发行验证容量再定窗口。本次不执行测试、不新增runner或改在途Plan。

工程入口用真实保留项目/source/stdlib/宿主依赖冻结分母，Rust SDK/engine准备单列。编辑Auto业务函数或handler后，父侧观察实际进程树中rustc/cargo0、稳定宿主/库不重建、目标函数真实生产、无关函数/模块生产和工件IO0及对象代际；多模块签名/type/meta/CT/权限/负依赖变更按实际读取失效。若显式选择AC，只计其约定的Auto机器码/链接工作，不能因此重编Rust宿主；A2R/A2C发布认证单独命令不混入快路径。

在线更新父报告绑定宿主PID/实际dispatch/source revision，新调用新结果、在途旧链旧结果、failed/cancelled/late不切当前、最后pin回收；保持当前工程所需UI/服务/资源生命周期。强制重启应用、先cargo重建、返回旧值或只报wrapper计数必须可区分为红；incompatible变更按真实策略另测，不能强求未承诺OSR/任意对象迁移。

正式工程周转依原固定组/受控执行/资源策略与冻结负载取样，P90≤2000ms和原更严门保持，prepare/冷编译/启动/查询/生产/安装/运行/清理/峰值/Rust构建次数分列。单源实验/部分入口通过只是进度；M2必须REP-01..11、全部场景/在用目标/真实部署和旧实现disable全闭，不再留到M4，M3/M4新增扩展当前不规划。

REP-09逐个旧A2X目标/公开API/产物核新provider与独立源/行为oracle，不能只Rust/C覆盖；REP-11对AutoUI所有已支持模式做实际交互/状态/回调/资源/退出，对AutoMan做实际配置求值/任务/生成/安装，其他消费者按全台账核。缺例/缺工具/未执行/未知消费者不因零选择或统一ignore变绿，原AG/重内存/平台策略保持，不为单包日常合并强跑里程碑全套。

REP-10审实际依赖/feature/构建单元、注册/dispatch、CLI/SDK/工具和发布物，default/all-features均无legacy engine；故意接回旧emitter/隐藏开关/旧binary/fallback控制须红。不是搜索vm字样为空，也不能只禁默认但仍在日常测试编译旧引擎。中立库和薄compat按实际实现分类，兼容入口必须真转ACE。

旧M1 binding/批准件/历史报告不为disable改expected或伪改NA；M2后继ledger记录原义务到新source/profile/producer/oracle的对应，迁移源/金样/证书身份与全部consumer。内部实现测试须具名转新合同/行为测试或历史记录，不能批量停用凑绿；新版日常不依赖旧执行器跑历史腿。退役和独立已发布版本回滚均在M2实测收据中关闭，不另留“之后产品替换”。
