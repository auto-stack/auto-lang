# PLAN-815 T-06～T-08 提前定向复审 R1（2026-10-22）

**needs_fix；新增六项（T-06两项P2，T-07三项P2/一项P3），与T-05 R6两项统一交回work。** T-08继续partial，T-07已承认未做的完整验收仍pending。Plan r3 executing10/17，T-05～08均未勾；本轮非T-09终审、不授予阶段交付pass。

## 对象、冻结与范围

- 用户明确授权T-06～08提前检查并与T-05一并修复。冻结HEAD **bb4f0a1caeb252ccb82d722659ee827703b82fc1**（code **c6fdaaaf1ecc2799d5ad225db3922a121c90c383**），T-06分派来源347d09d28，T-07 fixture/lifecycle来源9ca1130f9，T-08以冻结handoff为准。
- 唯一树`D:/autostack/.wt/lang-815/auto-lang`、branch `codex/plan-815-dev`，开始clean；[baseline](baseline.json.txt)绑定Plan r3/Specs/配置/锁/全部UI工具与协议consumer输入，M814-B已发布祖先93a25c924d1beafab130588bad4d1041dc86e48c。reviewer=Codex `/root`，有早期合同/账本参与限制、未参与实施，独立按源码与有效输入复核；未新建agent/第二树。
- 期间发现jet.rs/proc_host.rs/pkg.rs的T-05修复WIP，本审查未重置、覆盖或编译这些未提交更改。本轮探针使用既有冻结bin **c95748c80b512215d89d6c987d3b8b708f697d102a24a62333f7666051c42b61**，全部探针结束后再次核对仍为该指纹；结案核对时发现并行工作已重建为 **1b9387becf9de172b632e70838111c9c9590dd13e9a1dacc2ca2421e12638569**，该新二进制未获本轮验证，见final-baseline.json.txt。所有新finding的Ark/run_vm、T-07测试与闭包脚本相对冻结输入未变。新WIP不自动获得R6或本轮pass。
- 本轮审核实际分派/字段路径、已交付消费者与构建oracle、T-08逐条SD；777只读现况盘点。完整V-01..04/792发行与资源矩阵未完成，明确列入后续任务，不将承认的未做工作重复编号为产品缺陷。

## 新增六项

### F-815-T06-R1-01 / P2 — VM运行未消费已验证有效的front声明

**AC-02/V-02、§5.1字段/源交接，T-06。** 类型：已交付请求接线缺口，不重开T-04生成主体已闭范围。

- 重放`profile-paths.py`：合法I-PROJECT声明front_decl=./ui/front，真实app.at只在该目录；validate_project返回ok/零findings。VM run却立即failed，error查找固定`src/front/app.at`，没有使用声明路径。相同目录布局/同一前端源码的Vue generate合法对照成功，真实产物App.vue包含T05 probe，已冻结为vue-custom-front-App.vue.txt。完整wire/验证与终态见profile-paths.json.txt。
- 源码确认：run_vm.rs:461/485分别硬编码entry/CWD为src/front；PacSource传递其他请求字段却无front解析，tool_ops VM分派没有弥补。Rust run的UI判定同样直接要求src/front，Ark/Jet/VSCode的目录探测为相邻需审的字段矩阵，未说每条都已实测失败。
- 关闭：把已约定请求front/前端入口与相对根语义贯穿至实际执行/CWD/资源解析，保留默认src/front正控和front节点形式；补不同profile的字段来源矩阵。若某profile确有已批准的固定布局限制，须给既有合同/能力边界依据、在适当准入处诚实披露，并与§5.1/V-02对账；不能仅用旧实现的硬编码自证全部请求已接线，也不擅自扩张未经约定的运行模型。

### F-815-T06-R1-02 / P2 — Ark的run只准备工程，却报告真实运行完成

**AC-01/03、§5.4真实工具、T-06，影响T-08能力声明。** 类型：原完整执行需求缺口，不是要求新增一套未约定设备平台测试。

- 同`profile-paths.py`真实Ark请求，metadata/path验证ok，RunOk.outcome=completed、launch=None/error=None（110ms）；日志完整为生成ArkTS工程后“To run the app: Open DevEco Studio ... Click Run”。没有执行或启动任何Ark应用。
- 源码确认：ark.rs:801 run_ark_project_inner只有generate与手工步骤、返回Ok；tool_ops将其直接包装Completed。host/handoff却把Ark列为run_ui served且interactive/cancellable，T-06工作交接称“真实运行全7”。这不是缺某台设备导致一次环境失败，而是该臂根本没有执行步骤。
- 关闭：服务声明与实际效果对齐；已服务的真实运行须进入支持的执行链，缺工具/设备/前置条件明确具名结果，手工准备不能冒充运行完成。若阶段合同本意只交手工准备，必须对照原完整能力约定取得可审范围裁定/结果语义，不能简单删能力或仅改文案绕过AC。保留旧Man手工指导语义的兼容边界，补真实支持/前置缺失及取消交互的正反控。

### F-815-T07-R1-01 / P2 — “产出dist”的验收在确定安装失败时仍通过

**AC-04/V-04，T-07；R4/R5注记现正式编号，已有交付oracle无效。**

- 重放`lifecycle-oracle.py`，调用冻结的实际lifecycle test binary，单跑lifecycle_build_produces_dist；只有包管理器为私有OS负控，npm/pnpm/bun统一记录调用并exit7，不安装或联网。
- 实测当前测试**exit0/PASS**（1.32s），PM记录只有四次install，未发生生产build命令；失败场景仍通过这个“real production build producing dist artifacts”的成功验收。不是把本次负控当真实构建成功证据。
- 源码确认：lifecycle.rs:247忽略前次RunOk结果；:261-269仅检查BuildReport.profile，任意outcome/error甚至ServerBody::Error也接受；完全没有dist/入口资产读取。不能由5/5或2/2汇总声称成功构建覆盖。
- 关闭：将真正成功构建与具名环境/业务失败各自设明确oracle；成功用实际可消费dist及关键资产/内容验证，失败必须使成功用例失败或明确未执行而不计PASS。保留本次exit7负控，并检查运行安装前置确已成立；不能以调整用例名称保留原必需成功面的空白。

### F-815-T07-R1-02 / P2 — lifecycle辅助器不读取stderr，普通日志会堵死验收

**AC-03/04、§5.3日志与控制分流/真实验收，T-07，测试设施交付缺陷。**

- 重放`lifecycle-noisy-log.py`，相同实际lifecycle测试与包管理器失败入口，只增加131072字节stderr普通日志。静默exit7对照上一项约1.32s返回；有日志例12s外层截止仍未完成，PM marker证明已进入install。外层只清理本次测试PID树，结果在lifecycle-noisy-log.json.txt/log.txt。
- 源码确认：lifecycle.rs:29将tool stderr设piped，此后没有取出/排水；next_response只读stdout，wait/close也不读取stderr且没有内部截止。包管理器stderr最终写入tool stderr，管道满时链路无法继续到控制终态。12s是探针观察界限，不伪称产品有12s SLA；堵塞机制由管道持有/无读取路径确认，不笼统归因机器负载。
- 关闭：测试辅助器必须并发排水/保存日志、对等待/读取设有界诊断，并在断言失败/超时可靠清理其owned子树；静默/大量正常日志/失败输出/EOF与cancel正反控都能收口。不要吞日志、增大固定管道或只延长nextest超时掩盖设施问题。

### F-815-T07-R1-03 / P2 — 纯数据闭包门对明确高层实现依赖漏判

**AC-01/04、T-07纯协议fixture依赖闭包断言/data-stays-bottom。** 当前生产协议闭包没有被指称已污染，问题是门禁不能拒绝承诺拒绝的输入。

- 重放`closure-negative.py`：私有拷贝已交付fixture与**原样verify-deps.sh**，仅将路径改为实际仓绝对路径；有效无注入正控PASS。随后分别向私有consumer加入真实auto-ui-dialect（UI声明文法实现）、auto-bindgen（777标记lang-infra，data禁止上依赖的语言构建实现）；不改产品manifest，不编译高层实现，实际cargo tree normal/build均exit0且明确包含注入crate。
- **两个负控均仍validator_exit=0并打印pure data/UI-runtime-compiler-Man-free PASS**。closure-negative.json.txt及两个完整tree冻结。比只看正控树或assert名称更具区分性。
- 原因：verify-deps.sh:15硬编码少数包名，遗漏实际UI文法/lang-infra实现；其中auto-lang/auto-man等分支还自带空格、后接共同` v`会形成双空格匹配，是同门需校准的风险（本轮没有用它冒称单独已测败）。根777是否会另行拦住不等于此独立消费者门有效。
- 关闭：按已批准的层/闭包合同完整识别实际实现包，保留允许的数据/基础port边界；以可审的域/owner来源或精确完整集合控制，不靠泛前缀/扩allowlist。对上述真实高层注入须具名非零，正常fixture继续通过，并跑fresh/locked及适用失败对照。
- 首次manifest注入误替换注释里的[workspace]导致cargo101，保留invalid-setup，未计finding。早期auto-lex探针仅属探索：lang-core基础层是否允许不能只凭名字判断，本结论已改用明确UI文法与禁入lang-infra两个有效负控，不将lexer探索当成权威违规证据。

### F-815-T07-R1-04 / P3 — 新交付fixture格式与测试告警尚未清理

**AC-05卫生与本仓独立review要求，T-07。** 对冻结源码检查：lifecycle.rs格式相等，tool-protocol-consumer/src/lib.rs格式不相等，见t07-format.json.txt。独立协议fixture编译日志为lib7条、lib test3条（2重复）warning；lifecycle还有unused ErrorKind/json/Write/PathBuf及未用kill等5条，R6日志已指出归T-07，不能把它们当其他包235/22条旧诊断豁免。

关闭：按目标cfg收拢测试import/helper、清理未用代码与真实格式diff，复验已交付两目标；不使用广泛allow掩盖问题，也不要求清偿无关依赖的既有诊断。原T-05修复范围七文件卫生pass保留。

## T-07待完成面与T-08交接对账

以下是用户已明确承认/任务原表已有的未完成工作，不另重复编号新产品finding，但不能勾完成或后置掩盖：

| 待办 | 本轮事实 / 后续oracle |
|---|---|
| 纯协议完整消息/帧矩阵 | 独立--offline --locked 5/5通过；server“full surface”实际只覆盖5种ServerBody，缺DescribeOk/ValidateOk/GenerateOk/BuildOk；有大帧负控，缺声明的超深/深度边界对照。protocol-coverage.json.txt为准确缺项，不声称协议实现自身这些类型已损坏。 |
| 777/master并集 | 本分支只读strict **exit2，1 violation/3 grandfathered**；唯一原因stale_baseline，锚定输入crates/auto-ui-tools/Cargo.toml在libc增量后变化。不是发现一条新禁向边，也不是已验master并集。须全修复后对实际最终输入按受管流程重采样/审查，不能刷新掩盖违规。 |
| 792/fresh/locked发行 | 完整新tool/producer/source/embed资源、重定位、受管发布失败/保留/恢复与独立消费矩阵仍未做；旧T-04发行收据与DELIVERY26清单不覆盖新增run/build/helper变化。每门列源/锁/目标/正反例与真实产物，缺环境明确pending。 |
| V-01..04完整字段与兼容 | 本轮只实证front/Ark与现有oracle问题。mode/output_root/input_sources/dep/theme/sidecar/外部back等需按既有合同逐profile列消费/固定限制/不适用，不能把字段存在、helper同名或metadata validate当运行保真通过；不把未检查字段猜成已失败。 |
| Linux面与资源/许可 | R6 Unix具名4项pass保留；旧real_tree测试Windows命令的已披露可移植缺口、完整平台/资源audit仍须T-07对账，不能以“旧测试”自行免责。资产owner/license/源唯一性与真实发行内容须相互核对。 |

T-08源/bin指纹和SD-01..07正文确已交付，但能力矩阵仍把仅手工准备的Ark写成真实run，主请求字段全部贯穿的保证也与VM front反例冲突。SD-01/02目标已有A发布内容，B正文须以当前canonical为基线增补，保留A来源；不得拿迁移前历史before替代当前版本差异。详细[规范核验](spec-delta-reviewed.md)。这些文档冲突随对应finding修复，不重复计数；T-07前置与完整SD发布性尚未成立，T-08保持partial。

## 验证复用与统一交回

- 本轮新跑纯协议consumer5/5、真实VM/Ark/Vue路径、实际lifecycle静默/日志负控、闭包两个实际注入负控、777只读与T-07格式；没有跑整档t/tv/tt/tb/tu/taa/tf/docs_gen或重复R6全包计数。
- T-05 R6的98个Windows定向/Unix4项、真实SDK ABI/MCP交互、目录/Multi/legacy/PM原例仅按**冻结SHA/二进制/配置/输入未变的边界**复用，不覆盖新WIP或本轮新输入。原关闭项保持。本轮零AI implementation修改，未改canonical/live ledger、未合入/归档/清树。
- 当前T-05 R6两项（R4-05捕获非取消失败传播、R6-01合法bun环境PM测试误拒）继续关联；期间WIP是在推进这些修复，必须提交后按新SHA复审，不能据未提交diff自动闭合。与本轮**六项**合成同一修复交接；承认未做的完整T-07矩阵另列待办，避免把缺口藏入“准备稿”。
- AC-01/02/03局部fail、AC-04现有oracle/门fail且完整面pending，AC-05当期T-07卫生fail；A/L与T-04已闭历史保持。Plan executing10/17，T-05～08继续未勾。原唯一树work统一修复、补有效反例/成功oracle与T-08事实后提交冻结；T-09完整终审及最终pass后merge-owned冻结裸t+实际触面档/并集与发布收口仍pending。
