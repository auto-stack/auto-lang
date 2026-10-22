# PLAN-815 T-07子批2定向复审 R1

2026-10-22，**needs_fix；三项P2待补齐，T-07整任务不勾选，Plan r3保持executing10/17。** work发现的两个实缺陷修复本轮核验通过；独立792既有Rust应用driver也25/25通过。问题在新tool发行验证范围、资源审计事实、完整字段/profile证据仍不足，不能把这些局部通过包装为完整T-07闭合。

## 冻结、独立性及适用范围

- 唯一树D:/autostack/.wt/lang-815/auto-lang，branch codex/plan-815-dev；HEAD **503c9f67f5fca40661ca59a230dcd79c8418ef90**，code **e22fc75924a7f058a5df8150956b2d66268c5ecd**，base f6fd0ce7c86fd3c4f70ddddec296f2d021642031。M814-B交付祖先93a25c924d1beafab130588bad4d1041dc86e48c保持。
- 实际tool SHA256 **e319c5364523dbfac0c61d61ecb087b11a8d0261cdfc767bd1dfacc1b4b803f2**。独立driver正常构建的auto.exe SHA256 a8783f38698b507fc4fdda2bf97261195c591e9421084a8005882daaf6d71ee0。开始/结束同HEAD clean、源码/锁/依赖/配置/Spec及tool输入零漂移，见[baseline](baseline.json.txt)、[终核](final-baseline.json.txt)。
- reviewer=Codex `/root`，延续独立上下文，有早期合同/账本参与限制、未参与当期实现。按/auto-plan:review重建判断，未创建agent/第二实现树。仅用唯一树测试、经确认全新且独占的Plan组内scratch作driver产物，不改实现/canonical/live ledger、不合入/归档/清树。
- review_scope=phase：用户送审子批2及“完整T-07已完成”的就绪性声称，非T-09整体终审。交接和证据冻结为frozen-batch2.md、frozen-resource-audit.md、frozen-handoff.md及work-driver-*。审查结论区分产品错误与验收覆盖/收尾缺口；不把未测字段猜成产品都已损坏。

## 已核验的实修复和实际验证

| work finding | 本轮结论与证据 |
|---|---|
| F-815-T07-B2-01 / P1：canonical双直依赖与头冲突 | 修复范围通过。交付app/dev成员/共享workspace窄接线两依赖，wrap_example使用canonical头，组件块legacy/canonical重复头均剥除。定向generated_ui_cargo_uses_canonical_double_dependency 1/1；更重要的是本轮独立792全程25/25，fresh及重定位locked真实Rust app编译分别105s/95s，未再出现E0433/E0252。 |
| F-815-T07-B2-02 / P2：Vue请求路径主pac存在性门 | 原例关闭。from_workspace_with_request不再以磁盘pac存在与否决定已求值请求；新增真实tool字段测试1/1、删除pac后同请求输出文件集合/hash一致。没有把该单Vue原例外推为所有profile都无第二读取。 |

独立运行如下，不只读取work25/25收据：

- 原driver使用全新专用scratch，保留实际fresh和重定位编译：**25/25，exit0**，[日志](driver.log.txt)、[收据](driver-receipt.json.txt)、driver-scratch.txt。含真实构建、资源1项正常/等长篡改/坏hash/缺失/未登记、漏patch/坏pin/外来目录、再交付失败保留/恢复、sidecar越界、外来同级目录、他应用存活及dev同源。
- `generated_ui_cargo_uses_canonical_double_dependency` **1/1**，[日志](canonical-manifest.log.txt)。
- resources_manifest **5/5**，[日志](resources.log.txt)。
- `generate_ui_field_matrix_and_pac_snapshot_independence`以--ignored --exact实际执行，**1/1，0.78s**，[日志](field-matrix.log.txt)。
- lifecycle全部--ignored --test-threads=1，**4/4，44.45s**，含真实Unicode/空格Vue构建及旧成功/失败/组合正反面，[日志](lifecycle.log.txt)。
- 六净改Rust文件rustfmt逐字相等，diff --check通过，见fmt.json.txt；无本批新增编译诊断，不冒称清偿无关旧诊断。未机械复跑work全包178/181、api-gen54/55或Man145，也不将其总数当独立全绿。

## 三项修复/补验收交接

### F-815-T07-B2-R1-01 / P2：旧792 app driver不能证明新tool发行完成

**AC-04、Plan§5.2“复用并扩至新工具”、T-07子批2原待办，验收缺口。不是指称25项实际红，也不是指称新tool已经不能编译。**

原T-07待办明确旧T-04收据不覆盖新增run/build/helper，要验证新tool/producer/source/embed/重定位/独立消费。本批driver.py相对子批1没有变化。源码:161构建/选择auto.exe，:175调用旧CLI gen --delivery-root；:227/:242在app为default-member的bundle根执行cargo build --locked。唯一应用是uiminimal/render rust，driver没有构建、启动或消费交付包中的auto-ui-tool。

本轮全新25/25也只产生A/B两个uiminimal.exe，两个target均没有auto-ui-tool.exe。资产identity的正式verify.py检查的是assets/avatar.png（1项），不是新tool的embed/producer独立消费；metadata收进26个crate与这些crate的bin真正可发行不是同一证据。见[实际目标及包内容审计](delivery-coverage.json.txt)、原driver日志与收据。

**关闭**：在既有约定的发行形态上扩面，明确tool/source/producer/锁/资产输入，完成新工具从受管交付来源的fresh/locked构建、重定位/剥离开发根覆盖后的真实外部消费者调用及适用正常/失败/资源控制；若约定发行件是源码包，就用该包实际构建并运行工具，不强加一种未约定安装布局。已有旧app25项保留，不用复制旧绿收据或仅声明crate已在清单中替代。必要生成/build/run代表性入口依原支持合同选择，不要求假装本机有全部设备。

### F-815-T07-B2-R1-02 / P2：资源审计删除真实Inter来源，并错误描述随包路径

**AC-04、Plan§5.2资源owner/source/许可/实际发行对账，T-07/T-08；事实错误与审计遗漏。**

batch2.md:69、resource-audit.md:38–42及handoff.md:152–156声称“全仓无任何Inter字体/无OFL许可文本”，据此删除字体来源。实际：

- auto-ui-runtime/src/ui/iced/renderer.rs:1850–1852以include_bytes消费Inter-Regular.ttf、Inter-Medium.ttf、Inter-SemiBold.ttf。
- 同级assets/INTER-OFL.txt存在，4380字节，文本明确是Inter许可；三个字重分别411640/417300/419744字节。
- 本次全新792包中四文件都在相同runtime crate相对位置，字节与源一致；SHA256/长度见[逐文件审计](inter-resource-audit.json.txt)。当前canonical auto-ui-runtime规范也写有字体随iced资产。

此外resource-audit.md:51–54说“资产组不直接进792交付包，经二进制发行”。实际delivery_copy_dir递归复制UI工具crate：本次包内gallery1/wm9/shadcn346（含LICENSE/SNAPSHOT）都与源逐字节一致，见delivery-coverage.json.txt。应区分app assets allowlist、vendored crate构建输入、编译后的embed、用户工程materialize四条路径，而不是以1项app资源检查代表所有源/许可面。

**关闭**：纠正batch2/resource-audit/handoff来源清单，给Inter正确owner/runtime路径、许可及实际随包/消费身份；完整盘点本交付闭包实际编译期/embed资源并与实物对账，保留已经正确的三组源唯一性证据。原“vue.rs内嵌”位置不准确应改owner/位置，不能改成全仓不存在。以文件/内容/真实发布路径为oracle，不靠只搜特定目录或忽略规则误判不存在。本项是仓内资产事实核验，不作额外法律结论或新增许可制度。

### F-815-T07-B2-R1-03 / P2：静态Vue代表性用例尚不是完整字段/profile矩阵

**AC-02/04、Plan§5.1/V-02/V-04、T-07原待办“逐profile消费/固定限制对照”，就绪性缺口。不是要求所有字段在所有profile产生相同效果。**

新增field_matrix的真实覆盖有价值：静态Vue请求中的Unicode名/空格根、title_zh优先、front节点、theme、npm_deps、styles、i18n、dep根及删pac再生一致均有断言；lifecycle另证明zh-only标题在实际Vue产物中保留。这些本轮已通过，不应重做或否定。

但batch2§2把它和总测试数当作完整V-01..04闭合，所供材料没有原待办的“字段×profile×生产入口/不适用规则×oracle”对照。该新用例只生成Vue，没有设置/验证GenerationRequest.mode、output_root、input_sources、has_rust_sidecar、external_backend的实际效果；代码中mode字符串是theme.mode，并不是请求模式。dep_roots虽无字面字段赋值，确实由dep节点解析并在产物验证，审查不把它误列漏项。详[原文新测试](new-field-matrix.rs.txt)、[覆盖核对](field-coverage-audit.json.txt)。

旧R4输出根/多目标、Rust front与VM外部后端/主题等已有局部关闭证据，可按来源复用；协议往返/metadata路径校验只能证明数据/准入，不自动证明generate/build/run生产路径消费。动态/静态adapter、其他支持profile、sidecar和外部back的既有规则需要明确映射，不能用一条Vue静态用例覆盖。

**关闭**：补齐可审的原范围矩阵，逐字段列支持profile/操作、唯一来源与相对根、消费点、既有允许的metadata-only/固定限制/不适用依据、实际结果及具体证据。复用未变且适用旧证据，未覆盖部分补有区分性的正常/冲突/失败控制；先明确既有语义，不为补表擅自增加运行模型或新支持面。代表性组合即可，不要求无意义全笛卡尔积，也不允许仅改“全字段”标题绕过原验收项。

## 状态、规范与后续

[SD-01..07核验](spec-delta-reviewed.md)确认canonical双依赖修复与已发布M814-B规范一致；Vue快照原例关闭；但T-08来源清单/完整发行保证与以上事实不符，不能定稿发布。前轮已闭finding及子批1通过范围保持，不因为本批缺口全线重开。

AC-01/02已测局部通过，AC-04三处完整性/事实缺口未闭；AC-05当前卫生范围通过但T-09整体仍pending。work披露shell_pack freshness及plan734等预存红作为带边界的既有事实保留，不在本轮凭“预存”豁免合同内缺口，也不把未改且尚未复现的旧问题冒认本批新回归。

**T-07继续未勾，T-08完整前置未满足，Plan executing10/17。** work原唯一树修正资源审计并完成/补证新tool发行与逐profile字段矩阵，再提交冻结复审；保留本轮25/25、两个实修复关闭及有效局部证据，仅重审受影响内容。777 master并集归T-10；最终T-09 pass后merge owner才统一冻结裸cargo t+实际触面档、串行实际合入及Spec/ledger/收据/归档清理。本轮未运行整档/改实施/合入/归档/清树，不授权缩面或把合同内验收缺口降为非阻断债务。
