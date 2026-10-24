---
status: accepted-direction
describes: target-state-roadmap
created_at: 2026-10-20
updated_at: 2026-10-24
implemented_by_plans: []
first_followup_plan: PLAN-827
---

# AV真实源码运行与旧AVM替换路线

上层入口：[ACE总路线 §0](auto-native-backend-evolution.md)（原称AC/AV/AJ总路线）。后续规划先按其步骤核实现状与依赖，再由本路线选择G阶段/工作包，已有匹配Plan优先复用，不因候选排序自动新建或执行计划。

## 1. 目标与边界

2026-10-24用户确认：本路线扩面/迁移属于v0.6 Rust版ACE的M1–M4成熟度目标，需复刻现有AutoVM/A2X全部示例及对应消费路径；Auto实现的AACE延后至v0.7，须先满足Rust版成熟度门。四里程碑与G0–G8不是相同编号体系，后者继续逐项提供AV取证；A2X按对应后端/目标另补实际转译运行证据，详见[版本合同](../../plans/reports/ace-milestones-version-boundary-2026-10-24.md)。当前各Plan支持面/进度不扩大，也不把本轮文档修改当迁移完成。

目标是逐步让**原始Auto程序经真实前端→受检共享MIR→新ABC→AV运行，并最终由AV承接所有决定保留的旧AVM入口**。内核结构正确、typed fixture通过、单个例子能启动、默认CLI改名分别是不同成果。

能力分母与本波缺口见[语言能力矩阵](av-language-capability-matrix.md)，真实程序见[AP-01..23目录](../../plans/reports/av-source-program-catalog-2026-10-20.md)。语义来源仍为[Design43](../43-ac-av-typed-mir-and-abc.md)/[44](../44-av-data-layout-place-and-ownership.md)、各current Spec及具名profile；此路线不扩798/817/818/820–826原验收。本轮只改文档，未执行程序、开实施树、安装工具或切默认。

替换范围先盘点后冻结，不能以“不支持”删除旧功能分母；旧bug不当然成为新规范，显式破坏变更另获决定。完整AAC/AAV自举、任意第三方库、JIT/OSR和NaN-boxing不是旧AVM退役的自动前置；保留消费者实际依赖的语言/宿主能力则必须落实。

## 2. 三条轨道与现有计划

| 轨道 | 已有/拟Plan | 可认领成果 | 不可据此宣称 |
|---|---|---|---|
| K：安全执行与数据内核 | 798→828→820→821→822→823→824 | 指定profile、Place/owner/集合/闭包/Dyn及恢复验证 | 完整Auto源码/stdlib/产品迁移 |
| P：性能与编码 | 798→828→825→826，按实测择机插入 | 标量私有装载表示/PC/调用优化、新wire/体积测量 | 新语法/数值/库/宿主能力 |
| S：真实源码与消费者迁移 | D828后优先827首个G0/G1；后续R2-*..R8工作包 | 未改写源程序、强oracle、完整依赖/入口/SKU迁移 | 用旧AVM回退执行算AV成功 |

817/818是共享MIR的有限CTE/native消费者，不阻塞普通AV源码闭环；新复合CTE/native另按实际需求扩面。796/743提供语义/阶段身份，不能代替新语言支持。783是统一store；814–816只按已交付接口/实际owner交接，不自动替换任何运行引擎。

**2026-10-21当前排期：828后优先817/818、827和820功能线，825/826择机插入。** G0只读静态库存可提前准备；827 T-02及G1实际执行须D828，不等待817/818或820–826全部完成。G2a随D821接text-record源码，G2b随D822/D823接集合/enum/闭包源码；824只在Dyn/JSON相关真实需求触发时成为前置。每次扩内核尽早补实际程序，不将源码工作拖到内核链结束。旧“G1只等D798”的候选排期由D828门覆盖。

公共MIR/ABC/AV定义仍单写串行，三轨不能同时改同一个核心；独立817/818/827消费层可并行。T-01准备、T-02刷新已合入基面/Spec/API版本、写面owner/消费者与窗口接受；前owner记录合入/兼容结果、限制及释放，后owner核验接收，公共注册追加也串行。硬前置满足后取窗，不能持窗等上游或复制WIP；拆分只按实际owner交接。性能工作由实测阻塞决定，在计划交付/核心交接边界插入825→826；若源程序尚跑不通，优先正确性/源码接缝，完整新wire不能挤掉G1/G2前门。具体见[统一合同](../../plans/reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)。

### 2.1 横贯阶段的增量/cache/热重载基础

ACE粒度演进按[Design43 §8.1](../43-ac-av-typed-mir-and-abc.md#81-函数级首版与block级扩展方向2026-10-20)：首版声明/函数级，架构保留显式block区域及未来自动语义分块空间。RF工作包先交有限函数级证据，后续block复用/入口选版另冻结合同，不把活帧迁移或自动分块加入本批Plan，也不将未来区域接口封死。

用户2026-10-20澄清：本路线的语言覆盖分期不能把平台基础变成末期接线。[Design43 §6.1](../43-ac-av-typed-mir-and-abc.md#61-早期架构约束与分期实现2026-10-20澄清)规定增量、层级化全局cache和热重载须较早进入工程结构；Design39/40管理语法/查询/产物，Design44管理多代数据与资源寿命。

新增横向候选RF-I/C/H见§4，均未分配Plan ID或自动授权实施。最小AV在线热重载从最小已交付profile验证，不等待824、完整模块/stdlib、G6或AJ；并不要求尚未交付的未来类型同时支持重载。与源码轨按共享写面交接，不把现有798/820–827合同改成完整重载计划，也不反向等待基础新计划才能交付798。选取后续实施切片时先检查这些基础缺口，不能只按G表行序连续扩语言。

### 2.2 本批完成后的架构覆盖

798、828、817/818和820–827全部实际交付后，可以认领ACE首版核心骨架基本成型，作为继续扩大真实源码/库/消费者的基础。各profile分别证明共享IR/AV、函数增量/cache/安装、有限CTE/native、数据/所有权、装载/编码与scalar文件入口；不把typed fixture能力或不同profile独立通过合并成全语言、全后端组合支持。

G2及后续源码/stdlib、复合CTE/native、用户泛型/元编程/反射、宿主/产品迁移按现有语言及全示例缺口立Plan，纳入v0.6 Rust版成熟度；AACE/AAC/AAV主体迁移移至v0.7且在该门之后。完整AJ、增量发布器、block级增量及活对象/活帧迁移不在本批，仍按独立需求安排；G7/G8负责默认切换/旧AVM退役。下一批优先由827分母/程序与消费者缺口选切片，持续复验增量与资源合同，不为提前自举改变顺序。

## 3. 分阶段门槛

D-N均指最终独立review、冻结门禁、实际合入、Spec与merge收据齐全；executing8/10或分支能编译不算D。以下G为路线门，既不是Plan状态也不是已经实现的功能。

| 阶段 | 真实交付 | 主要前置/owner | 出口门槛与优先级 |
|---|---|---|---|
| G0 库存与差异基线 | 能力/程序/stdlib/入口分母、SEM-01..08材料、平台/features/SKU与强oracle清单 | 827 T-01只读可先；T-02等D828 | 全量盘点有来源、未知有owner/阻塞门；不得靠缺oracle/skip删分母。P0 |
| G1 真实纯源码闭环 | AP-01/AP-22原文件成功，AP-23正确trap；实际file→frontend→MIR→ABC→AV→typed result | 827 T-03..06，D798/D828及注册窗口 | entry0/0和准确trap，native适用对照+手审oracle，fresh/hit/restart、源变更和坏实际反控；不用旧VM执行。P0 |
| G2a 数据源码可用 | AP-02/AP-03、独立小型文本/record程序从原source运行；Unit/main/print/转换/资源规则补合同 | D820/D821；R2-E/D/N/X按实际交接 | 字段原位/副本/重叠、entry0/9、内容及全出口资源责任；普通输出不是trace替身；SEM相关决定先冻结。P0 |
| G2b 常用纯语言程序 | AP-04/05/06/08的List/HOF/enum/Result/模式/捕获与真实源码 | G1、对应D822/D823；R2-L/X；不强等824 | 固定stdout3；60/5/5；404/timeout/200；2，合法/非法捕获和错误传播/cleanup；原source不剪裁。P0 |
| G3 项目与常用库 | AP-07多模块、AP-17泛型/spec、AP-19动态对象/Map、AP-09JSON及必要字符串/Math | G2相关子门，R3-M/T/C/H；Dyn用D824时显式声明 | 真实解析/初始化/包与stdlib实际manifest、typed host grants；callee/依赖修改正确失效；不硬编码imports或复用旧bytecode。P0 |
| G4 可显式选用的脚本产品 | AP-10/11及一个有限输入→计算→文件/JSON输出的真实工具；CLI/SDK opt-in AV | G3所用能力、R4-P，当前CLI/SDK owner | argv/cwd/stdio/exit/错误阶段、sandbox副作用和取消都正确；专属scratch无泄漏；安装独立可运行，未改全局默认。P0 |
| G5 有状态/异步/服务 | AP-12/13/14与AP-15及service项目真实请求 | G4相关能力，R5-O/A/S；独立挂起/共享设计 | 顺序/消息偏序、取消/背压/资源跨await、socket/reader/进程真实退出；并发输出仅按合同归一化。P1 |
| G6 UI/会话/嵌入/FFI | AP-16、REPL/daemon/模板/库API、决定保留的Rust/C/Python消费者（AP-18/21） | G4，async依G5；R6-U/I/F | 原应用真实交互/持久会话/回调/外部资源及关闭正确，所有保留入口有AV执行证明；缺工具/未决语义为gap。P1，专门外部生态P2 |
| G7 分SKU切默认 | CLI先、service/UI/embedding按各自完成度独立候选发布 | 对应G4/G5/G6、R7，每SKU逐项独立验收 | 功能/语义/资源/安装/性能门全部通过，显式AVM回滚可用且不隐式fallback；切一个SKU不宣称全产品完成。P1 |
| G8 旧AVM退役 | 全保留SKU默认AV、移除旧engine/codegen及新AV对旧NaN布局的执行依赖与隐含调用；必要旧API留薄兼容层 | 全保留SKU G7、R8，旧工件/调试/自举展示去留已决定 | 冷安装/无旧引擎构建+运行、零旧dispatch/回调/依赖、所有分母闭合、退役兼容说明/归档。P1收口 |

阶段是门集合，不是强制全序：R3-M的纯函数/模块解析窄切片可从G1提前交付；若G2b的真实List/HOF库需要模块装配，则消费该已交付切片，不等待完整G3。G3完整门仍汇总后续相关语言/库能力，不能形成“所有G2等完整G3、所有G3又等所有G2”的循环。具体R包取号时必须展开实际功能/接口依赖，不按阶段序号猜启动门。

G0允许“发现但未裁定”的差异带着owner进入清单，**只阻塞使用该语义的门**；G1采用现有明确的非冲突scalar域，不要求先决定所有未来语言。G3不把“builtin List<Note>成功”算用户泛型；G5不把普通函数环境存活算真实async；G6不把首帧/能import外库算业务操作通过。

每个阶段均须逐程序证明原始源、真实共享编译/执行阶段、当前profile/依赖/工具身份、结果与副作用、正常/失败/取消清理、persist/restart、正式父侧判定。cold、warm、局部编辑和诊断路径分测；不缓存PASS，不以两引擎同错代替独立oracle。

## 4. 后续工作包和拆Plan规则

本轮只实际领取[827真实源码验收基线](../../plans/827-av-source-program-baseline.md)。以下为**尚未分配Plan ID、尚未授权实施**的中等工作包候选；前门调查后按可独立验收的程序切片用中央脚本取号，不一次创建“完整语言”大计划。

| 工作包 | 输入/最小程序 | 拟交付与边界 | 必需前门/优先级 |
|---|---|---|---|
| RF-I 增量共同合同与真实粒度核验 | 同源cold/warm/body/signature/layout/配置编辑及错诊断 | 复用既有query/store与身份，核各已实现前端/语义/MIR/ABC层的依赖和真实重算；缺层有owner和后续切片，不复制前端 | 先只读审计相关交付；实现按实际owner/D门拆Plan；横向P0 |
| RF-C 全局分层cache覆盖与生命周期 | 已实现producer的跨工程/重启/损坏/GC/恢复用例 | 明确层清单、required-data/失效边/来源、统一root/lease/预算；未实现层保留future，不以MIR命中冒领全阶段完成 | 复用783及相关已交付producer；不等全部语言；横向P0 |
| RF-H 最小AV在线热重载 | 持续存活AV、旧调用尚未完成时提交函数新版本 | 候选快照验证/安装、版本可见规则、旧调用及依赖保活、新调用选版、坏/迟到候选不破坏当前版本、旧代有界释放；精确策略先冻结，非任意状态/活帧迁移 | 最小适用D798 profile及统一生命周期；不等完整G3/G6/AJ；横向P0 |
| R2-E 源入口/Unit/基础输出 | AP-04或一个原有print程序、普通main/顶层文件 | Unit/尾值/脚本/格式化、typed console及有限cap；不扩任意IO | G1、SEM-05/06；P0 |
| R2-D record/text源码补齐 | AP-02/03 | strict域adapter、参数/字段/返回/原位/Drop；旧str字段另按SEM-03 | D821、实际source域；P0 |
| R2-N 数值源码与运算 | AP-20及宽整数/NaN/溢出原例 | 按类型族拆有界profile，存取与算术分开、显式转换/Math装配 | D820、SEM-01；P0主类型，其他P1 |
| R2-L List/enum/closure源码 | AP-04/05/06/08 | 推导/构造/方法/for-in/is/HOF；先内建实例化，不冒领任意泛型 | 相关D822/D823、R2-E；P0 |
| R2-X 错误与Dyn源码 | AP-06/18及显式Any程序 | Result/?/trap/catch分别设计；Dyn只按真实需求消费D824 | SEM-06/08、受限来源/权限；P0/P1 |
| R3-M 多模块与初始化 | AP-07及真实项目 | source resolver/包/导出/全局/闭包/依赖身份；复用现有resolver/783 | G2必要能力、语义事实owner；P0 |
| R3-T 方法/用户泛型/spec | AP-17及保留ext程序 | 真实约束/实例化/两receiver派发和错误诊断；不再造AV推导器 | R3-M、796实际接口；P0 |
| R3-C 集合/对象/JSON | AP-09/19 | Map/Set/匿名对象/JSON语义与字段访问、类型擦除/恢复安全 | SEM-07、D824仅适用面；P0 |
| R3-H stdlib与typed host | AP-09/10/20 | 738实际选层/签名/权限→executed；FS/IO等按库族拆Plan | R3-M及typed值/资源；P0 |
| R4-P 产品opt-in与工具 | AP-10/11+真实小工具 | 实际CLI/SDK engine selection、cwd/env/argv/stdio/安装/故障；不切默认 | G3、814/816实际owner；P0 |
| R5-O 跨挂起共享/资源 | 真实回调/请求持有对象 | 句柄/loan/send/资源寿命及共享策略，不能用裸RC补齐 | 生命周期先设计，SEM-04；P1 |
| R5-A async/Task/generator | AP-12/13/14 | 可取消状态机、mailbox/迭代/await、stop/join和背压 | R5-O适用部分、已有服务设施；P1 |
| R5-S 网络与服务 | AP-15和service工程 | typed transport/provider、请求/响应/返回资源、停止/超时/流 | R5-A及库族；P1 |
| R6-U/I/F UI/会话/嵌入/FFI | AP-16/18/21及所有保留调用点 | 按consumer拆Plan，独立callback/持久状态/资源和实际用户动作 | 对应G4/G5、宿主owner；P1/P2 |
| R7/R8 默认/退役 | 每SKU实际完整闭包 | 先候选安装和回滚→后移除旧依赖/工件消费者；不合成一个大merge | 对应G门及独立证据；收口P1 |

优先级表示排期，不是支持面豁免。P0验证统一基础与常用程序，P1补长期存活/业务入口；P2生态切片可按真实消费者提前，但AACE实施遵守v0.7和Rust版成熟度门，不再随P2提前主体迁移；P3是JIT/OSR/NaN-boxing等无实际替换阻塞的优化。低优先级能力若属于全示例/保留SKU仍阻塞其替换，不能永久标后续。RF工作包具体依赖/范围由Plan冻结，不自动成为所有G门串行前置。

RB现在是v0.7 AACE子线：先满足v0.6 Rust版ACE全示例及对应成熟度验收，再按[743主体合同](auto-acc-bootstrap-contract.md)冻结compiler-source-demand、compiled-language-support、组件/宿主桥和代际范围；不能仅以编译器源码子集可用提前实施AAC/AAV。旧AAVM/AA2R展示不作新AV或AACE证据，不新增双重进程内解释塔重型门；保留展示按真实编译+运行形态另验。

## 5. 产品入口分母：最后替换的不只是auto run

| 入口ID | 必须盘点的现有入口 | 默认迁移门/证据 |
|---|---|---|
| EP-01 | CLI脚本/项目run、.at/.as、stdin/argv/exit与测试命令 | G4→CLI SKU G7；实际调用走AV，语义profile可追溯 |
| EP-02 | SDK run/run_with_capture/path/staged等公开兼容接口 | G4/G6→SDK G7；签名/结果/诊断/资源和各features一致 |
| EP-03 | REPL、persistent/daemon、serve/req、debug/bytecode工具 | G6；多轮更新/隔离/调试/退出，不能每次新建空会话替代 |
| EP-04 | AutoUI VM渲染、handler/computed/store、native provider与callback | G6；真实交互+视觉/状态+关闭，-r vm既有后端语义不能偷改成新engine选择 |
| EP-05 | HTTP/service、async任务/流、watch/reload入口 | G5/G6；有限请求与受控停止，no leak/no stale dispatch |
| EP-06 | AutoGen模板、Man动态配置/工程工具、库内嵌求值 | G4/G6；真实生产入口/资源与失败出口，非仅导入接口 |
| EP-07 | Rust/C/Python互操作与反向调用、.as动态脚本 | G6；保留环境/features/依赖版本、线程/生命周期和错误通道 |
| EP-08 | CTE/meta、构建期执行等实际编译消费者 | 817只认其标量；保留复合需求另案，不能把824当已接通 |
| EP-09 | 包/部署/生成app内嵌VM、旧ABC/ABT缓存/反汇编消费者 | G7/G8；独立冷安装、版本迁移/拒绝/重建与公开工具去留 |
| EP-10 | 旧AAVM展示/内部工具、书籍/Playground/示例轨 | G0判断保留/正式弃用；保留者G6/G8逐项，新自举另走RB |

此表是入口类别，827必须扩成真实symbol/call-site/binary/package清单，不以10个类别宣称10个入口全迁完。初始源码锚有auto-lang lib::run_with_capture_and_path_staged、autovm_persistent、autovm_daemon、test_cases::avm、UI vm_bridge、AutoGen及Man的实际依赖；拆包后跟随唯一owner，不复制旧facade当新实现。

允许在候选期由用户**显式**选择AVM回滚；AV unsupported不能在函数/脚本内部静默回退AVM。fallback会执行到何处、副作用是否已发生必须可审计；不同引擎的重跑只在隔离fixture/sandbox或可证明无副作用场景进行，不能在用户真实文件/网络/数据库上自动影子双跑。

默认切换以进程/会话启动边界为首版策略：旧会话/请求先按其原生命周期有界drain或受控取消，活帧/对象/loan不得按新布局重解释。需要保留的持久状态另做版本化转换/回滚证明；没有该证明时明确重启/再加载边界，不冒称原位迁移。G8核旧会话、callback及lease已清空后才能移除引擎。G7可保留显式AVM回滚；G8当前包不再含旧engine，回滚转为恢复经验证的上一发行包/数据备份，不偷偷重新链接旧引擎。

## 6. 每个G门的证据与判定

| Gate ID | 必需证据/反控 | 不通过时的处理 |
|---|---|---|
| Q-01 完整性 | 冻结capability/program/entry/SKU/provider分母、每项owner、source/dep/expected hash；变更分母有理由与确认记录 | unknown/缺依赖/无oracle/skip保留gap，0例不能绿 |
| Q-02 真实新管线 | source bytes→共享前端/Checked→MIR→ABC→verify→AV的实际span/产物；旧dispatch=0；修改源值必须改变结果/重键 | 不认标签/伪计数/替换MIR/内藏旧解释器；回work |
| Q-03 语义 | 程序强oracle+适用旧AVM/native/a2r差分；SEM差异按profile/确认决定处理 | 不统一不同语义的输出，不因两实现同错PASS |
| Q-04 安全/资源 | 类型/Place/loan/move/drop、非法准入与runtime计数分开；fault第n次/取消/预算/清理原始事件和峰值 | 不用panic/强杀替代guest trap/Drop，未达数字不能以记债豁免 |
| Q-05 项目/恢复 | imports/stdlib/providers/权限/target/profile/类型及实际内容依赖，fresh/hit/new process及修改/损坏反控 | cache命中不跳当前准入，不恢复trusted/PASS |
| Q-06 可用与性能 | 固定同义负载、工具/机器/flags、端到端冷/热/编辑、吞吐/总峰值/IO/退出；原始样本全保留 | 每SKU T-01前冻结预算/绝对门与可比基线；不以“更快/差不多”验收或删失败样本 |
| Q-07 发布与回滚 | 冷安装/搬移/缺件/版本错配/启动/退出、显式AVM回滚、无旧引擎执行反控 | 产品owner复审/修复，不能只在开发树可用就切默认 |
| Q-08 真退役 | 所有保留入口/feature/平台组合在候选二进制实际执行、无旧engine/codegen链接或调用、旧工件策略及compat facade单向指向AV | 任一保留consumer仍需AVM则不能宣称完整替换；JIT不是用来延迟本门的必需借口 |

既有798编辑profile的P95<1s、32MiB受管分区/128MiB进程增量以及各Plan约定不被路线放宽；其未达/当前证据由原owner收口。完整应用/服务有不同新product profile，需要在实施前以代表输入冻结独立预算和新版本，不能把实验8MiB执行额当所有UI/服务的承诺，也不能用“大应用需要”反向豁免旧profile。新增数值属于拟议政策，用户未指定统一性能比率/内存上限。

G7是“该SKU默认AV”；G8才是“旧AVM已退役”。是否归档历史源/保留compat API与是否仍执行旧VM分别核验；仅把CLI默认分支改掉不是退役。公开旧ABC/ABT兼容若确有保留需求，提供明确转换/版本路由；没有需求的内部旧工件按经确认策略重建/拒绝，不承诺新AV直接执行旧ABC。

## 7. 本轮规划交接

新建827只负责G0证据体系与G1现有scalar源文件闭环，保留798等原合同/进展。其后的R包在827给实际库存/语义/公共接口与owner证据后再拆适中Plan；当前不预报完整语言实施工期或自动推进代码。

2026-10-20初稿输入/身份与文档预检见[planning inputs](../../plans/attachments/av-language-roadmap-20261020/inputs.json.txt)和[preflight](../../plans/attachments/av-language-roadmap-20261020/preflight.md)。2026-10-21排期修订与输入见[本轮预检](../../plans/attachments/ace-order-20261021/preflight.md)；仅更新文档，不将本路线程序执行、review、门禁、默认切换或退役自动判绿。
