# 拆分与重构M1：有限恢复对象合同

2026-11-12，r2（规划对齐）。关联[1006归档](../archive/1006-web-generated-member-independent-delivery-and-recovery.md)、[989归档](../archive/989-split-refactor-joint-delivery-and-tag-readiness.md)与[1015 r1](../1015-split-refactor-closeout-supplement-and-contract-alignment.md)。1006/989均已正式交付；本清单的原四类对象/业务/指定包对义务保持。UI业务/013对象及RQ旧包来源差额由1015补验，当前仅立约、未补验、不预签关闭。本文件是有限恢复合同，现有能力及历史实测以canonical/归档收据为准。

## 1. 来源与冻结时点

| 来源别名 | 已能指认的正式Git对象 | 用途 |
|---|---|---|
| L-C | auto-lang `11e1f1fd5e79766266ce8d54356155f018b8d661` | 1003 C/P1修复后的可用旧来源，不回退已知缺semantics-contract的坏包 |
| U-C | auto-ui `393ea17ceaaea19c58da294ea292c19b273263b8` | 1003正式UI vendor修复，与L-C配套 |
| L-RQ | auto-lang `f29f6facc6a31db429e0076ba35222171bf397dc` | OS068正式语言落地主线来源 |
| U-RQ | auto-ui `691047651590e716b8b77651ea68bbdc9773c119` | OS068正式UI交付来源 |
| O-RQ | auto-os `3ea7ead67d2da739a04c771209b1cc18a0a18210` | OS068正式Windows RQ来源，避免把旧用户进程当已重建包 |
| M-BASE | auto-man `347c29f7ddc6650a94548f4523b3bc8f0e7cae17` | 旧正式Man/Core/Full与CLI基线 |
| W-BASE | auto-web `2d698413b368896f913eba375ff788c795f395cd` | 990已落地的Web源；仍不具有1006成员导出能力 |
| D-BASE | auto-down `58ea33f699efa5cb1194473a2d45dcec51006842` | 当前下游工具/SDK所需Down源基线 |
| N | 已由989冻结，见[里程碑Spec](../../specs/repository-boundaries/split-refactor-milestone-acceptance.md)与[1015 §5.1](../1015-split-refactor-closeout-supplement-and-contract-alignment.md#51-有限来源集合与nn规则)的完整源码表 | 1015 T-01核实际生产输入/工件可复用性；漂移则说明N'及定向失效，不能把当前HEAD或新产物直接冒旧N |

这些是源码重建基线，不声称磁盘上尚有旧安装或历史二进制可复用。旧源取回/组合可构建与支持配方须按owner实际文档核定；若组合不可消费，保留失败、由owner修正同等可用正式基线，不能挑坏旧包制造收益或直接N/A。新候选依赖变更按实际消费者绑定，不任意混合表内别名。

原r1定义对象/来源规则、数据/恢复判据；989已冻结N并完成其所记验收，不能仍将其T-02记未启动。当前差额是[收口复核F-989-CLOSE-01/02](split-refactor-closeout-status-audit-2026-11-12.md)与1015的两个合同任务：前者补两个UI对象旧/新/恢复后业务，后者补指定正式旧RQ组→N的受影响恢复断言。对象/编号/说明存在不抵实际执行；新工件hash在真实产出后填。

## 2. 四个对象与责任

### RC-UI-01：UI独立交付包

- owner：auto-ui tools delivery；语言供应SDK契约。1015承担当前有限补验，989历史交付不改。
- 1015拟适配：002/013正式源副本只作最小Rust入口/依赖定位变更，保留真实view/handler/store/API/主题；原013是内存教学后端，不以auto-todo产品SQLite替代，也不新承诺持久化。该适配及交互可行性由1015 T-01冻结，尚非已补验。
- 对象：002-counter与013-todo各一对独立源交付包；旧=L-C/U-C及M-BASE正式CLI，新=N实际选中生产工具/UI/SDK。旧包在隔离scope由正式源重建，不拿P1缺件包当可用基线。
- 入口：已存在`auto gen --delivery-root <bundle>`，项目选择及参数以Man正式help和930/1003 driver实际入口为准；`test/fixtures/modularity/generated-ui-consumer/driver.py`是现有消费探针，全部scratch/tool/producer/target须显式指定。新旧都fresh/locked/移根构建，不能手改Cargo补依赖。
- 资源/数据：包内字体/主题/stdlib契约等只读资源按内容hash；Counter从初态增加一次应为1；Todo用隔离合成任务。未证明该形态有持久化时，Todo只作会话业务/资源oracle，**不冒数据库恢复**；跨进程持久化由RC-RQ-01和RC-WEB-01承担。
- 配方：保持A目录；在独立B目录完成导出/校验，停本测试拥有进程后切换启动路径。损坏B必需SDK/资源或导出失败时，不替换A；恢复启动A并读Counter/Todo任务与原资源hash。这里恢复的是代码/资源和可运行性，无用户DB迁移承诺。
- oracle：旧A/新B均真实业务成立；故障非零且A文件hash不变；恢复A后独立启动/资源与相应业务正确。只见窗口/构建成功不够。

### RC-SDK-01：语言SDK与Man/Web实际消费形态

- owner：语言基础SDK、Man Core/Full交易及Web SDK owner，各自按既有支持入口负责；989协调同一候选。
- 对象：语言L1最小消费者、Man Core和Full的实际CLI/SDK安装、Web SDK服务消费。旧=L-C、M-BASE、W-BASE及各SDK锁定所需U-C/D-BASE；新=N各真实消费形态。Full所需UI资源纳入闭包，Core不得偷借UI实现。
- 已有入口：Man `python scripts/man-install.py install --profile core|full --install-root <root>`及`verify`；Web `python scripts/web-install.py source --output <source>`、`verify-source --source <source>`、`install --source <source> --install-root <root> --target-dir <target>`与`verify --install-root <root>`。两owner的安装交易已有`--fail-point prepare|verify|swap`；实际执行前按该冻结源help核参数，不发明跨源恢复flag。各输出均在对应计划scope。
- 恢复配方：以真实A安装/SDK为基线，以B独立staging准备/验证，使用现有owner交易失败点；若owner仅支持“停自有进程后从保留A完整目录重建/重新安装”，就按该正式方式验，不能宣称无停机自动升级。恢复前保全A完整源/锁/manifest及配置备份；恢复后verify和真实L1/Core/Full/Web consumer再运行。
- 数据/配置：只用本scope合成配置、provider/端口值；SDK只读源/资源核hash，业务数据不装进SDK目录。现有自定义provider不得被安装覆盖。
- oracle：manifest/版本/源hash与CLI实际执行一致；三个阶段故障准确、旧安装/合成配置保全；恢复后不是只报verify绿，还须真实语言值/Full消费/Web HTTP对应断言及端口释放。
- 边界：只验上述实际使用profile及Windows形态；不推广所有发行渠道/平台，也不重复建设安装器。SDK安装成功不抵RC-WEB-01成员包导出。

### RC-RQ-01：Windows App/Host/CLI/backend包组与Todo数据

- owner：OS装配负责包组，UI/Man负责公开工具/Host，Todo backend归产品owner。1015补指定旧组在两Host的有限演练；沿OS068支持/修约，不扩大Video移交。989既有合成旧组O-RQ/U-C/L-C证据保留其范围，不视为下行指定组合已验。
- 对象：旧包组由O-RQ/U-RQ/L-RQ、M-BASE及O-RQ记录的正式产品pin重建；新=N的App、standalone Host、desktop Host、CLI、必要backend与同套资源。协议/resource generation、App/Host组合和具体Todo产品源在989 T-01逐正式pin取完整SHA，不能以repo总HEAD代产品来源。
- 入口：沿OS068生产Spec和归档T记录的真实release三件套/启动参数/后端启动方式；现有定义没有通用`rq upgrade`命令，**不新增它**。989 T-01从正式源提取实际参数化命令填候选清单，未知参数不得编造。无受支持方式则具名阻E-04并回生产owner，不能只做冷启动。
- 数据：专属scope/data/todo.sqlite（具体basename服从现有backend配置），在包目录外。用实际Todo API增加一项唯一marker并切换完成状态，关停自有服务后用Python sqlite3独立读精确行/字段；无实际schema就不得先写猜测SQL。989冻结真实schema及查询，恢复使用同schema或受支持备份，不承诺跨schema原地降级。
- 配方：A包组启动并写数据→停止自有App/Host/backend→一致备份DB/关联文件与配置（WAL状态按真实DB支持方式处置）→独立B组校验/启动/继续读写→错Host/缺资源/失败切换保留A与备份→恢复A包组及需要的数据备份→两Host各核实际任务/旁观App存活与持久值。不得用不同版Host与App混配后隐藏direct回退。
- oracle：对应进程实际映像hash、协议/资源身份、无direct回退；前后真实Todo任务/独立DB值正确，失败损害不传播到旁观App；恢复后包组/数据一致。仅有帧/attach/空库Video进展不算业务恢复。
- 版本边界：若某个子件在新旧组合未改变，明确记same-source复用，不虚构该子件跨版本证明；包组至少包含N实际变化的CLI/SDK/声明输入，不能把完全相同包复制两份冒跨版本兼容。

### RC-WEB-01：Web成员包

- owner：1006/auto-web主责，语言中立支撑，Man薄CLI；989消费其正式生产交付。
- 首版来源：没有既有历史Web成员导出包。A/B均由**1006正式交付的同一导出器/SDK**从两份有实际版本/只读资源差异的普通Auto源生成；不宣称是两个历史公开发行版。拟夹具`auto-web/crates/auto-web/tests/fixtures/member-delivery/{a,b}/`由1006 T-04创建，A/B完整源hash及exporter/SDK完整SHA在正式交付绑定。
- 公开入口：拟`auto service-export --dir <project> --delivery-root <bundle>`，1006实现前不可执行。导出后仓外`cargo build --locked`、真实TCP服务；运行参数/受控关停按生成服务实际公开帮助/配置，由1006交付时记录。包内无开发根path及用户数据，不能用手工打包旧backend冒A。
- 数据：同一schema v1，显式scope/data下合成状态文件（或现有db公开API对应SQLite）。普通Auto POST先写`marker=before`、独立磁盘读回；B能读A状态，再写`marker=after`并独立读回。A/B返回各自版本/资源标记区分执行身份。文件名/API签名以1006真实夹具固定，POST仅改内存不PASS；不接用户HOME/聊天真实账户/生产库。
- 正常替换：保存不可变A与初态快照→停自有服务/确认端口释放→备份数据→B独立构建、验证、启动→核版本B/资源B及before值→写after→停机再读after。原子目录提交保护代码/资源，数据一致性另靠quiescent备份，不能把文件rename当数据库事务。
- 失败/恢复：B准备、manifest/依赖验证及提交点各注入代表故障；A包hash与故障前数据须保全（若B已正式写数据，不声称回滚包会自动逆转业务数据）。按支持路径恢复A及明确选中的一致备份，核版本A/资源A/预期marker与独立磁盘读回。恢复步骤失败要如实记录残留位置，不能先删除数据再声称恢复成功。
- oracle：A/B包/锁/manifest/源/资源hash可追，旧源不可达后两者均独立构建/HTTP；正常替换、失败保全与实际恢复逐断言成立。1006还须验真实auto-chat与auto-database导出路径，不能靠A/B空壳回避产品闭包。
- 边界：A/B受控同schema验证首版交付和恢复机制；不替任意历史版本/数据库升级、双机不停机升级、Linux或离线构建背书。没有把原承诺直接N/A；能力与首版真实包对由1006实现/验收。

## 3. 每行执行时必填字段与关闭门

统一候选清单逐RC对象记录：owner/渠道、生产Plan revision与正式review/merge/Spec、旧/新各repo完整SHA、项目源/工具/锁/config/resource/protocol/schema、公开命令与全部输出目录、合成初态及备份一致性方式、成功与代表失败断言、受支持恢复步骤、恢复后独立读回、实际包/数据hash、子进程/端口真实终态、结果/排除/残留/限制。值未产出标“未产出”，不填虚构hash或PASS。

- **当前完成状态**：四类原对象/责任/判据已定义，1006/989已正式交付；1015已补验并关闭两处原义务差额（F-989-CLOSE-01/02，见下），本清单与Spec已同步；1006/989历史ready收据不改。不把旧r1“待生产交付/T-02未启动”当当前状态。
- **P2**：1006已公开导出/移根/HTTP/持久状态/首版A-B恢复及独立review/merge完成，P2正式关闭；1015不重做Web生产出口。
- **RC-UI/RC-RQ补验（PLAN-1015，2026-11-12）**：1015 T-01核源/最小Rust形态与真实交互后——UI两对象各A/B/恢复A真实业务+资源（Counter 0→+→1→Reset 全链经 AutoUI MCP 真实事件+可见值读回；013 渲染/勾选/过滤/计数；013 AddTodo 生成形态缺口具名回 auto-ui rust 生成器 owner 有界修复后消费），RQ 指定 O-RQ/U-RQ/L-RQ/M-BASE→N **服务 Host** 真实任务/数据/恢复/旁观存活演练 18/18（含独立 SQLite 读回与旁观隔离；RQ 组合 UI 腿按 same-source 复用，drill 执行路径不触 UI 工具）；**窗口面（desktop/standalone-RQ attach/帧流）按 OS068/989 身份绑定复用**（OS/UI 源 SHA 与 N 相同、仅语言交付附加差，见 Spec §3）——服务 Host 补验为主、窗口面为身份复用，不宣称本包重验桌面窗口。原“旧窗口存活或合成旧包差异大小代替业务”的差额已用真实断言替代。
- **关闭与发布**：1015独立复审/正式沉淀后逐债关闭、同步本清单与Spec；本轮不改canonical/历史ready。联合tag另守1013等当前ACE/AGE签章与跨仓组合门，1015完成不自动打标；实际运行结果未产出不得预填PASS。

只在本计划自有隔离数据/owned进程内操作。真实用户安装、PATH默认、持有缓存/证据不动；无远程发布、tag或通用升级系统。将来实际owner支持能力与本配方冲突时保留缺口并修合同，不自动删对象/降门。
