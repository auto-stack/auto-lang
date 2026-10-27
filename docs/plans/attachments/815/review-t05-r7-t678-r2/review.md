# PLAN-815 T-05 R7 / T-06～T-08 提前 R2

2026-10-22，**needs_fix**。八项历史finding中四项CLOSED、四项PARTIAL，新增一项P2测试回归；当前共五项需修。T-07已承认未做的完整发行/资源/字段矩阵仍待办。本轮非T-09终审，Plan r3保持executing 10/17，T-05～T-08均未勾。

## 冻结对象与独立性

- 唯一实现树D:/autostack/.wt/lang-815/auto-lang，codex/plan-815-dev；HEAD **e3fdcdc76c5d28ec9274a9101acc3f6a80c5c6a6**，code **d039f23995cb7763787b86cd27f4144627d69d3d**，包含T-05修复9291a7ee3；base bb4f0a1caeb252ccb82d722659ee827703b82fc1。M814-B已发布祖先93a25c924d1beafab130588bad4d1041dc86e48c。
- 实际工具SHA256 **385ffef8f131e08a08fcc41487881f54ba4a4815cbaebe0968284116ad696f16**。开始/结束同HEAD clean、生产源码/依赖/配置/锁/当前Specs/工具输入零漂移，见[baseline](baseline.json.txt)、[终核](final-baseline.json.txt)。只在私有target创建反例与测试产物；未修改实现、canonical或live ledger，未创建agent/第二树、合入/归档/清树。
- reviewer=Codex `/root` 延续独立审查上下文；有早期合同/账本参与限制，未参与当期实现。依据源码、真实工具与独立控制重建判断，不直接采信work的全闭声称。范围=用户交接的T-05 R7两项及T-06～08六项修复，review_scope=phase。遵守/auto-plan:review；VM会话使用仓库autoui-verifier现成MCP客户端。
- [当期规范核验](spec-delta-reviewed.md)与frozen-handoff.md冻结交接事实，不发布规范。所有新增控制只验证既有完整接线/实际运行/有效产物/有界验收要求，不增加设备支持或产品超时SLA。

## 历史关闭表

| ID | 状态 | 独立结果与边界 |
|---|---|---|
| F-815-T05-R4-05 / P2 | CLOSED | 原bound-overrun 10.35s具名failed、truncated 0.35s具名failed，含DrainEnd/teardown；非owned holder仍活。自然EOF空列表合法completed、held-pipe取消0.31s cancelled且释放holder前宿主EOF退出0。spawn/非零/等待失败源码为具名传播，不再折成无设备；原P1及本P2范围关闭。 |
| F-815-T05-R6-01 / P2 | CLOSED | PM矩阵单独继承/合法bun环境均exit0，真实工具无pkg/磁盘npm仍实际选bun，具名安装失败；来源观测及磁盘翻转/移除已代替错误值排除。下方新增测试缓存回归另编号，不能据此给T-05整体pass。 |
| F-815-T06-R1-01 / P2 | PARTIAL | VM默认、front prop、自定义node优先三会话均validate ok并真实MCP snapshot出现T05 probe，cancel/EOF正常。Rust同样有效请求仍读src/front失败，详下。 |
| F-815-T06-R1-02 / P2 | PARTIAL | 缺wrapper不再假completed，实际链已接到build/probe/install/start；生成JSON5元数据读错导致aa start参数非法，详下。 |
| F-815-T07-R1-01 / P2 | PARTIAL | 静默install exit7使原成功测试exit101，大日志例也exit101；真实成功31.19s及新负控通过。报告/文件存在断言已加强，但坏引用仍通过，详下。 |
| F-815-T07-R1-02 / P2 | PARTIAL | 131072字节stderr原堵塞例已修，约1.29s收口；close_and_wait仍无界，详下。不是将已修排水问题重新认作未修。 |
| F-815-T07-R1-03 / P2 | CLOSED | 原脚本无注入PASS，真实auto-ui-dialect/auto-bindgen各注入后cargo tree成功且包确入图、validator具名exit1。四包允许集合与实际纯数据闭包一致，双空格匹配风险已消除。 |
| F-815-T07-R1-04 / P3 | CLOSED | 12个净改Rust文件全部rustfmt相等，纯协议fixture本轮重新编译5/5且无本目标新warning；lifecycle编译不再有原unused imports/kill等测试warning。core/ui-tools原诊断不冒称全库已清。 |

## 五项修复交接

### F-815-T06-R1-01 / P2：Rust生成与运行仍未全链消费front声明

AC-02/V-02、T-06，原要求缺口继续PARTIAL；VM原例已关闭，保留证据。

独立重放[rust-front-cwd.py](rust-front-cwd.py)，真实协议run_ui/render=rust，项目只有ui/front/app.at，front_decl=./ui/front；Rust默认目录正控通过生成并抵达cargo run（私有cargo记录argv/CWD，不计真实Rust UI运行成功）。同内容自定义目录请求在11ms具名failed：`Front directory not found: .../custom/src/front`，尚未调用cargo，见[rust-front-cwd.json.txt](rust-front-cwd.json.txt)。VM三种目录正控及validate结果见[front-paths.json.txt](front-paths.json.txt)。

原因：tool_ops改了UI准入，但rust_ui.rs:643–650的compile_rust_ui_app仍用find_front_dir，不消费PacSource声明；运行:5829仍固定src/front，delivery/resource相邻入口也用find_front_dir，须按原范围对账。不是新的运行模型需求，不重开T-04已闭迁移主体；当前T-06修复声称的请求贯穿尚未满足。

关闭：同一有效请求贯穿Rust生成源、增量/资源判定及运行CWD，不能仅放行准入。补默认/自定义/节点优先、默认目录与声明目录冲突的区分性对照，并按既有合同核对相邻入口；不得用改支持文案或拒绝原有效请求代替修复。

### F-815-T06-R1-02 / P2：Ark从生成JSON5取出的启动ID带标点

AC-01/03、T-06及T-08声明，原真实运行缺口继续PARTIAL。

[ark-launch.py](ark-launch.py)使用真实生成器和工具run链，只用私有hvigor wrapper产出标记HAP、hdc程序记录并校验argv；不声称真实SDK/设备运行。构建→list targets→install均实际抵达，启动收到：`-b com.example.t05probe",`、`-a EntryAbility",`，而生成文件的字符串值为com.example.t05probe/EntryAbility。hdc参数oracle拒绝后RunOk failed exit9，见[原生成文件、完整argv与终态](ark-launch.json.txt)。

源码ark.rs:970–973及:992–995对`"bundleName": "...",`/`"mainElement": "...",`仅trim_matches引号；行尾逗号保留，导致结束引号也未去掉。不是缺机器设备，而是本生成器的正常输出就不能被新增消费者正确读取。

关闭：按实际JSON5值语义可靠读取并验证必需ID，失败具名，不靠默认ID/删标点猜值静默启动错误应用；保留legacy手工指导边界、缺前置失败与取消。至少让原样生成产物→启动argv的正控成立，补相邻格式/读取失败/启动非零控制；真实设备执行仍按T-07支持/环境矩阵据实记录。

### F-815-T07-R1-01 / P2：dist引用断言仍是假绿

AC-04/V-04，T-07，安装失败原例关闭、完整产物oracle继续PARTIAL。

[dist-reference-oracle.py](dist-reference-oracle.py)逐字提取冻结lifecycle.rs的产物断言块并编译运行（仅提供私有root与日志尾巴对象，不改断言逻辑）。有效入口/已有main.js正控exit0；把入口引用改成不存在的MISSING.js，或完全移除script引用、只留div id，仍各exit0。三例均有同一个真实assets/main.js文件，见[原文断言与变异结果](dist-reference-oracle.json.txt)。这验证验收器灵敏度，不声称真实Vue编译器已经生成坏引用。

原因：:408的take_while不接受`/`，从assets/开始提取只得到assets，再拼assets_dir形成assets/assets；随后:416的`html.contains("<div id") && bundles>0`足以绕过全部引用核验。现有真实构建测试通过不能证明这个检查能拒绝损坏产物。

关闭：成功报告、入口内容、实际JS资产及入口真实引用共同成立，损坏引用/无引用负控必须使成功oracle失败，正常构建继续通过；不能以存在任意JS文件或重命名用例替代所承诺的可消费产物验证。

### F-815-T07-R1-02 / P2：stderr已排水，关闭等待仍无界

AC-03/04、T-07有界测试设施，原要求的有界等待部分继续PARTIAL。

原噪声反例已经收口，勿重修已闭排水。lifecycle.rs:160–165的close_and_wait仍直接Child::wait且没有截止；上方注释称“with a bound”，实际仅next_response有30s接收界限。Drop也直接kill/wait，没有为关闭路径建立检查/诊断收据。

[lifecycle-close-bound.py](lifecycle-close-bound.py)保留原文Tool辅助器，只将BIN选择到私有不退出子进程，直接调用close_and_wait；34s外层观察仍停在ENTER_CLOSE，最终由探针清理自己PID树，见[lifecycle-close.json.txt](lifecycle-close.json.txt)。这是辅助器面对被测对象挂起时自身是否有界的故障控制，**不是观察到真实工具EOF挂起，也不把30s响应界限扩成所有构建的产品SLA**。

关闭：给close/失败清理实际有界的退出等待和诊断，正常EOF保持exit0；不退出/断言失败控制也应由辅助器自身可靠收口，不依赖外部nextest/探针强杀。保留stderr完整排水，并按适用平台核对owned子进程/reader收尾，不用单纯延长超时或吞日志掩盖。

### F-815-T05-R7-01 / P2：新PM矩阵受legacy OnceLock缓存顺序影响

AC-03/04与验收有效性，T-05/T-07，修复引入的测试回归；不是重开已修的bun合法回退。

Windows命令`cargo test --locked --offline -p auto-ui-tools --lib pkg::tests:: -- --test-threads=1`：7过1红，tool_pm_state_matrix在:541期望(Bun,DiskLegacy)，实际(Pnpm,DiskLegacy)，见[pkg.log.txt](pkg.log.txt)。WSL Ubuntu-22.04相同冻结源码/命令、既有target离线重编也7过1红，见[unix-pkg.log.txt](unix-pkg.log.txt)。不涉及并行竞争。矩阵单独在继承与bun-only环境均通过，[PM原例回放](pm-fallback-environment.json.txt)保留。

原因：同模块test_command_names_are_consistent/test_resolve_is_cached先触发legacy resolve的进程级CACHED_PM OnceLock；新矩阵把之后切cwd/写pac bun视为必然重新读取磁盘，但产品legacy缓存语义未改，resolve_with_source返回已有Pnpm。PM_STATE_LOCK只覆盖新矩阵和auto_detect测试，既不能清空OnceLock，也不能消除先前缓存。不是将缓存值Pnpm判作产品单源错误。

关闭：验收在合法既有缓存/测试顺序下有效且不改变legacy缓存语义，来源与磁盘独立性仍由有区分性的正反例证明；隔离需要首次解析的测试或修正其状态前提，不靠ignore/强制测试排序/排除缓存测试补绿。上述完整pkg模块串行Windows/Linux及原bun环境对照都须通过。

## 验证账与777收据

- Windows：proc_host默认9/9+指定hard-bound 1/1；pkg模块7/8（明确1红）；tool_process默认21/21+指定ignored串行8/8；legacy ignored 2/2；lifecycle成功/组合/失败三项各通过（成功31.19s，组合13.56s）；纯协议consumer重新编译5/5。PM单测双环境、真实bun/ADB/VM/Ark/私有产物断言等重放另列，不与Rust唯一用例重复相加。
- Linux：WSL Ubuntu-22.04既有工具链与/home/visus/plan815-unix-target，--locked --offline同冻结源码pkg模块7/8。先前R6 Unix4项等未变来源的关闭证据保留，不把本轮没有重跑的旧测试列为新绿。
- 格式：净改12个Rust源全部相等，见fmt.json.txt；生产diff --check通过。fixture自身新增告警已清；依赖/runtime235/ui-tools22等既有诊断继续明确，不据此要求无关清偿。私有复审harness的未使用import告警不计产品告警。
- **777 strict本分支独立复跑pass：0 violations、3 grandfathered**。与bb4基线相比，仅base_revision由5c4dc2773更新为d039f2399，declared_edges/config/text_scan未拓宽，见777-baseline-delta.json.txt、777-report.json.txt、777.log.txt。旧stale_baseline关闭；这不是master并集、792或完整发行通过。
- 本轮未跑整档cargo t/tv/tt/tb/tu/taa/tf/docs_gen。T-09通过后merge-owned冻结裸t+实际触面档、master串行窗口及发布收口仍pending。

## T-07待办、T-08与下一步

792/fresh/locked发行、完整master并集、资源/许可证审计、V-01..04完整字段/兼容/失败矩阵、协议fixture四种服务端回执与深度负控仍是work已承认未完成项，不重复编号。旧real_tree测试Windows命令的已披露Linux可移植缺口仍须完整平台面收口。本轮没有将局部测试通过包装为这些面已完成。

T-08指纹相符，捕获失败/手工准备两处原文已改；Rust front与Ark启动的实际残留仍使完整声明不成立。SD-01/02须基于当前A规范增补，保留A来源，详spec-delta-reviewed.md。AC-01/02/03局部fail，AC-04验收器partial且完整面pending；AC-05本轮改动格式/新增告警范围通过但T-09整体仍pending。A/L与T-04历史关闭不受本轮重开。

回原唯一树work统一修齐五项，回放当期共享反例和相邻状态，提交新SHA送受影响定向复审；也可按既有授权先补完整T-07并定稿T-08，再交T-09整体审查。不逐条触发整档，不缩面/延期/改弱成功oracle，Plan继续executing 10/17。

探针校正披露：Ark首版错误把Progress当终态，修正筛选后才形成finding；close辅助器首版直接rustc链接到另一编译器版本rlib失败，改为Cargo私有fixture解决，均不当产品失败。持久证据只采用上述实际抵达并有正反控的结果。
