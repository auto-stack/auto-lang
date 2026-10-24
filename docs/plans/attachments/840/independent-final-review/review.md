# PLAN-840 独立最终复审

结论：**pass**。可记录「独立子 agent 最终复审通过」。范围是 r8 已落地/已归档交付的历史最终验收，不重开计划、不补做合入、不追认旧同会话审查具有独立性。全部原定必需项已有适用证据，无未验证的必需项被标成通过；pending_closeout_gates: none；open_findings: none。

## 身份与冻结

review_scope: final historical landed/archived；reviewer_identity: `/root/plan840_final_independent_review`，独立子 agent 上下文；未接收实施总结作为结论，按主计划、Git 对象、源码、实际原始日志和 JSON 业务断言重新建立判断。前次 postdelivery-review 仅作为索引；其两个审计脚本已经逐段阅读，并在本目录另存、重新执行；独立补充检查在 independent-checks.py，不依赖其既有 pass 文件。

- inspected_main：83adc3419c41662fc3f28384b7784c2cde05c52c（master）。Plan revision8，开始核验时 Plan SHA256 为2035d0d8cba2c64b07615478bf5e709c59b1a646de21e7e78a52f1d8d953719e。
- 原行为实现：1f19395254d7e676bd188e43e3d8f734d57aaa12；基面a239653365a0c3451c85aa9921189fcf011bd8cf；正式 autographics-dev：8b62db5cba54b9bfe7f759f4920b22ccb9b75e88。
- SDK artifact：codex/plan-840-sdk →932ad4877e5968f1aa7227d9dcd7fb69b4c69a21。canonical扫描补丁0676dc13398f27655b4ecc1c34f70a7e95374736与5e2f15806、SDK提交的四文件 stable patch-id 均为842d36b4e00b54af7d6320b2b9482f04771a774a。
- 最终修复冻结：ba6479da53b7999e490dd0e8ac8d701d5ffe86b2；实际master交付：53298cd273e0c04fabc508745ce342655394f1a4；归档c0705306371d4c2c399f9040577be17aa52d3fcf；清理确认01c421a1a4ec9622a287be33f5251fff0cdd04ff。逐对 Git ancestry 与实际 ref 核验通过。

## 完整证据复用边界

不是以“后来只有文档”一句话推定有效。独立脚本比较 ba6479、53298、当前83adc341 的完整声明 source_paths Git tree：crates/packages/test/experimental/stdlib/scripts/.cargo/.config/Cargo.toml/examples/vendor/.agents等相等；这些路径无未提交实现改动。六份实际磁盘锁SHA、auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8、冻结关键源码/测试配置SHA均重新计算。后续Git改动逐路径确认为docs，差异清单在independent-checks.json.txt；canonical/live ledger也无WIP。故最终门的代码、依赖、配置与输入未失效。原graphics frozen1f与正式8b的可执行源码/依赖一致，master未混入原图形实验源码；后续生产修复由根最终门覆盖。

| 原始完整门 | 冻结绑定与核验 |
|---|---|
| Phase1 63有效调用 | merge-r1/gates.json.txt逐条exit/expected/code_commit=1f；实际driver140、AGP163、consumer66+4；16官方case逐个report；全部具体正反oracle重新读取。75历史记录保留先前失败/被替代artifact，不以它们关闭门 |
| Phase1根门 | merge-r1/root-t-r4.txt 5332及root-tv.txt163最终Summary通过 |
| 最终合入门 | phase4/merge-r8/results.json.txt四项全部绑定ba6479；对应raw t5345、tv163、tt5717、tb5316全部通过，失败/超时0 |
| 主检出批量 | batch-r3/baseline.json.txt绑定53298；legs与raw tf6291、tt5717、tb5316全部通过，失败/超时0；原skip518/1800/1548未消失 |

七条最终raw日志SHA重新计算，完整数字和哈希在verified-closeout.json.txt。主检出last-batch receipt明确840/53298/pass/new_reds为空，P796既有报告与批量前备份字节相同。已验证完整门直接复用，不重跑Cargo全档或原生全矩阵。

## 原 AC 逐项映射

| AC / 任务 | 结果、源码与证据 |
|---|---|
| AC-01 T01/01a/01b/08 | pass：G839基面、正式来源及独立消费；四文件scanner补丁身份、固定SDK932、源码闭包/锁和漂移反控；merge-r1完整源审计、review-r2冻结收据与Git对象核验 |
| AC-02 T02/03 | pass：behavior.rs有界take(MAX_USER+1)、原UTF8字节/拒BOM、API/bridge64KiB和combined320KiB、区域与哈希；authoring_behavior真实边界/CRLF/快照不热载/新读源/篡改身份拒绝断言；driver完整140门包含该族 |
| AC-03 T03 | pass：真实Parser、重复根符号/精确callback签名与Rules全表达式遍历；嵌套函数重置Unit规则、源序immutable常量/FStr保持。读取12项behavior测试实际assert及R1反例→R2关闭日志；原非法return、动态/循环/私有反控与Host修源恢复无遗漏 |
| AC-04 T04 | pass：实际Auto API源19函数，首错粘滞、handle0/缺actor、axis/flag/2D z受检、dirty后derive确保同callback读后写；consumer整帧拒绝/关闭VM保last-good；原生group/world/全mesh像素及API-error具体原因重核 |
| AC-05 T05/06/07 | pass：Builtin6/User7明确区分；初始化、tick、Pause/中性Step/Stop/Fault/预算与Editor隔离。完整Play正例27phase、6个pixels阶段及有序tick trace/Stop reopen值重核；cpu-budget失败实际到预算oracle |
| AC-06 T06/07 | pass：Host正式来源选择和CLI路径，不借probe替产品入口；producer/consumer各move/patrol/fast/relocated正例，同runner SHA，source原字节SHA与trace对应；missing/callback等失败不fallback，Save/root边界有Host测试 |
| AC-07 T04/08 | pass：旧profile/AGP16分母及原素材保持；128节点、600tick、20A/B、弱owner/GPU Arc复用与真实leak/upload负控；原owners退出[101,101,0]及具体原因重核。未将heap估量当硬配额 |
| AC-08 T07/08 | pass：两套13场景各4正9负，具体像素检查/错误phase与reason、partial等完整性均重核；真实OS click和counter/content各自oracle有正负区分；公共API/CLI/版本/有限支持面与canonical一致 |
| AC-09 T09 | pass（本次独立最终历史审查）：逐原AC/V、修复、规范、健康与遗漏扫描完成。历史R1/R2均如实披露同会话限制，本次独立上下文不改写该历史事实；旧review明确merge门后置 |
| AC-10 T10 | pass：完整63门后正式graphics8b及SDK-only canonical落地；规范/台账/归档/guard清理与840完整批量收据闭合。源码未误落master |

## V-01..10 逐项映射

| V | 结果与独立核验路径 |
|---|---|
| V-01 | pass：固定来源/SDK/AGP/两graphics锁及六root锁、source闭包/漂移反控；Git patch/tree与merge-r1 source/consumer门 |
| V-02 | pass：SourceSnapshot源码与authoring_behavior边界、raw hash、regions、UTF8/CRLF/BOM/大小/版本错误及不热载assert，完整driver日志 |
| V-03 | pass：Parser/Rules源码、callback_unit_returns等18嵌套反例/4合法执行及immutable常量动态负控；review-r2原回放与Host修源证据 |
| V-04 | pass：19个API实际Auto函数、句柄/派生/有限写，tick实际trace与全part像素；API故障拒帧并释放VM |
| V-05 | pass：typed Program/Play与真实27phase/逐tick序/暂停恢复/预算错误/Stop reopen；原Starting取消回归在完整driver门中保持 |
| V-06 | pass：正式Host/CLI选择、活动态禁止切换与不排队、relative/SaveAs/root/no fallback的Host断言及缺源原生错误 |
| V-07 | pass：两逻辑示例加fast/relocated，每套同一exe；实际源SHA→trace→GPU端点/原位置/全mesh逐条重核；consumer独立SDK/target |
| V-08 | pass：容量/轮次与实际VM/GPU owner反控，完整owners raw退出与失败原因；未夸大计量边界 |
| V-09 | pass：OS实际输入、16官方case、旧游戏/编辑/交互partial/truncated、层级/资产/Play/用户原生GPU矩阵；透明图保counter而content红，blank两者红 |
| V-10 | pass：SD01..05对应行为API/scene-play/authoring/四实验边界与SDK；八目标canonical及CLI/示例/诊断/预算与实际代码对应；ledger20来源hash重新计算 |

## 追加 phase AC/V

| AC | 结果及证据 |
|---|---|
| AC11 | pass：tests.rs的CTE与Case/AV同feature门控，SDK test-cases仍可选；phase2 off/book no-run与on实际定向原始记录、最终全档编译通过 |
| AC12 | pass：别名/原filters未删；资源override只分配池；full/t3增加既有ffi-dual实际族绑定。phase2 selection-audit历史814删ID负控/现配置清单与r7实际23成员及23/23原始执行保持；最终完整选择面通过 |
| AC13 | pass：sema四个共享Instant边界闭合计量空隙，原30样本/100us tiling/P95<1000ms不变；r4 gaps最大2us、P95=278ms，最终完整池验证不再以隔离绿替代 |
| AC14 | pass：phase2冻结/定向check/健康/SD、原r3漏计时证据与作废复审均保留；r4实际源码改动符合具体修订合同 |
| AC15 | pass：r7/r8受影响review、原支持面复用绑定，完整门后置；本次独立查源码与实际分组/原始控制 |
| AC16 | pass：最终ba门→53298落地→main完整tf/tt/tb→receipt840→归档/clean guard实际清理全闭合 |
| AC17 | pass：test-only完成态驱动先view重登记再真实pump；48项原始Summary；原5调用文件assert未删除，生产dynamic.rs与RQHost production前缀同基面逐字相等 |
| AC18 | pass：pending零deadline确定红、原多片早退负控；RQHost等真实server gamma发布、唯一1条后再原末窗断言，4真实管道controls；未把客户端手塞pending |
| AC19 | pass：Phase3冻结/SD与affected review，生产helper未改；完整门及原几何/交互断言保持 |
| AC20 | pass：旧r5/r6失败/未完成档未冒充pass；最终ba完整门、53298批量及实际清理替代pending闭合 |
| AC21 | pass：Windows错误5真实legacy disposition反例；旧立即降级执行→deadline内retry；25ms真实超时、释放后Executed、永久错误一次执行、stale与100次竞争控制。保留2s/10s及best-effort，未声称超时降级后仍保证互斥 |
| AC22 | pass：读取两生产点Arc clone后释放DashMap guard再执行的修复；旧first-pull冲突5s红及inner mutation5s红，修复nested值/EOF/回收绿；原所有SSE assert行保留、5s/3s/600ms/spawn<30未放宽；34定向、200 stress、4 review回放raw验证 |
| AC23 | pass：共同ba源码/配置/六锁/check/fmt/SD与复审冻结，完整门完成且无新pending。SDK与stdlib源规则符合实际锁降级/registry重入语义 |

V11=off/book no-run；V12=on CTE/adapter；V13=实际选择/删ID负控；V14=原IPC30样本+边界、census及full/t3实际FFI成员；V15=冻结复审/SD；V16=最终完整门及main批量，均pass，对应phase2原始记录和上述最终门。
V17=Init48及RQHost4真实病例；V18=原早退/新完成/零截止控制；V19=生产helper与原assert身份；V20=最终t/tt/tb及main tf/tt/tb，均pass。Phase4未另定义稳定V21..23，按AC21..23明确的真实竞争/瞬态/硬错/timeout/stale、SSE事件/EOF/取消/相邻等待、健康冻结与完整门逐项核验，不伪造V编号。

## 规范、收尾与限制

三次ledger投影78→79、79→80、80→81的before/after实际Git字节SHA、版本增量、11/6/6目标、1171/1180/1183无关raw项及顶层字段保留全部重算。20当前相关行source-sha256匹配canonical或原review文档；frozen spec delta与当前规范的规则对应已读。没有在复审发布canonical/live ledger。

归档Plan是Git已跟踪的archived/delivered26/26；auto-lang/auto-down两个worktree注册及目录不存在，plan-840-dev不存在，各fresh guard日志clean及remove记录对应；组内非Git消费者/复现资产有意保留。实际SDK/artographics refs均正确。主检出其他计划WIP仅盘点未修改，本次只写本目录。

检查范围外：未新执行任何完整Cargo/native/GPU运行；历史producer exe已按合同删除，只能由独立过程/资产报告相互绑定其SHA，保留consumer exe实际重算SHA；不保证在不同机器/显卡重跑结果。旧批量锁日志没有errno，不将其所有历史失败断言为错误5；已确认的真实Windows机制与完整池闭合足够支持现合同。D-762-1生产UI quiescence helper、沙箱/热重载/通用稳定SDK/任意heap硬配额等原非目标不列作必需验收，也未宣称交付。

审计器自身一条初始whole-file SDK lib.rs等价假设无效：SDK按旧pin生成，canonical含先前PLAN-816导出。Git差异仅此继承基线，已改用合同要求的精确四文件patch-id及scanner主体一致校验；详情harness-correction.md。未修改旧证据/产品/合同，不是产品finding。

本次无阻断或非阻断新finding，无未批准缩面、延后、ignore、删输入/断言、放宽阈值或workaround关闭项。健康结论仅为变更触面无新增未处置warning/debug，既有全仓warning已披露，不冒称全仓零warning。下一步：父agent在原唯一Plan追加本次独立历史最终review记录；保持archived/delivered26/26，不再work/merge/清理。
