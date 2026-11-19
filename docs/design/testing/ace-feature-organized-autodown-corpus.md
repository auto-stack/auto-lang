# ACE 按语言特性组织的共享 AutoDown 语料

2026-11-19，设计 r2。用户已明确要求：ACE 的 AutoDown 语言测试按语言特性集中组织，同一份 Auto 源码供 AV、AC、A2R 等后端验证，避免重复维护；补入1078与相关设计、计划，**本轮只规划，不迁文件、不改用例或实现、不执行测试**。本设计细化[文档式用例设计](autodown-document-cases.md)，由[1078](../../plans/1078-av-parity-baseline-and-autodown.md)建立机制，各语言/后端计划分批迁移和验收。当前实现与拟新增能力必须分开记录。

## 1 组织与作者权威

**公共语言源码的唯一活跃作者根拟为 `crates/auto-lang/test/cases/language/`。** 目录与文件按语言行为命名，不以 AV/AC/A2R、M1/M2、计划号划分。新用例与迁入的旧用例都遵守同一规则；已有 `av.parity.*` 等稳定 Case ID 可保留作历史身份，不据ID前缀决定存放位置或执行后端。

```text
crates/auto-lang/test/cases/
├─ language.suite.ad                   # 公共语料总选择，拟新增
└─ language/
   ├─ basics/expressions.case.ad
   ├─ numbers/literals-conversions.case.ad
   ├─ text/interpolation-unicode.case.ad
   ├─ calls/binding-pipeline.case.ad
   ├─ functions/references-closures.case.ad
   ├─ collections/list-places.case.ad
   ├─ types/variants.case.ad
   ├─ patterns/nested-match.case.ad
   ├─ dynamic/objects-null.case.ad
   ├─ modules/imports-globals.case.ad
   ├─ ownership/borrows-moves.case.ad
   ├─ errors/catch-finally.case.ad
   ├─ generics/spec-methods.case.ad
   ├─ generators/yield-close.case.ad
   ├─ async/tasks-actors.case.ad
   ├─ comptime/evaluation-macros.case.ad
   ├─ metadata/attributes.case.ad
   ├─ ffi/imports-callbacks.case.ad
   ├─ script/python-context.case.ad
   ├─ domain/node-values.case.ad
   ├─ composition/ordinary-projects.case.ad
   ├─ shared/                          # 确需跨Case复用的受控源码/资产
   └─ migration/legacy-map.tsv         # 旧义务→唯一作者/检查映射，拟新增
```

以上是目标路径，不是本轮已建立的测试目录。一个语言特性可以有多份文档、一份文档可以有多个Case；文件大小由可读性与语义边界决定，不能为每个后端或计划再建一套源码目录。新增特性沿语义目录扩展。

每个公共Case维护一份 `### Auto` 源码或多文件工程闭包，AV、AC、A2R分别在该Case内声明检查。文件正文是唯一作者源；各suite/固定组只引用、选择，不能复制Case来形成后端视图。

## 2 同源验证与后端独立结果

| 项目 | 共同维护 | 各后端独立声明/观察 |
|---|---|---|
| 输入 | 同一Case、Auto源码字节、多文件内容、资源、虚拟路径、入口模式 | target、工具链、SDK、合法provider/ABI及明确的授权/预算参数 |
| 公共语义 | 独立确定的值、输出、事件顺序、普通错误与资源义务 | 真执行路径、诊断编码/产物格式、实际资源观察、平台适用性和结果 |
| AV | 同一源码 | 真实新AV运行；禁止旧VM或固定返回值 |
| AC | 同一源码 | 正例编译链接后执行机器产物；反例按约定parse/bind/type等阶段验收 |
| A2R | 同一源码 | 生成Rust、真实rustc构建、运行与行为分别有证据；emit金样不代compile/run |
| A2C/A2PY/其它 | 适用且已承诺的同一源码 | 在原范围内追加目标检查；不因共用目录自动扩大M2硬门 |

每个公共语义预期有一个维护位置；格式要求在多个check中显式写同一个数值/流断言时，按同一公共合同一致维护。生成代码金样、目标诊断、平台换行等保各自独立预期，不能用被测AV输出回填AC/A2R的expected。是否需要共享payload引用，由1078先核已有能力，不能在文档中假称任意新字段已经可用。

后端adapter接收同一份规范化source/project payload，报告保相同公共源码内容指纹。为目标生成的入口包装、ABI shim与SDK事实另列身份；禁止为某后端手改Auto源、换mini例、把map改loop或把`xs[i]`改get后认领原Case。规范确有不同输入时保独立Case或显式变体，按第4节判定能否共享内容。

### 2.1 已有格式的同源例子

下例展示当前已有AutoDown写法和已注册AV/AC recipe的组织方式；是设计示例，**本轮未运行**；AC检查按用户要求先#ignore。A2R编译运行适配器仍须由真实owner接入，不以 `### Rust` 发射金样代签运行。

`````autodown
## Case: baseline-value
id: "av.parity.1078.baseline-value"

### Auto
path: "main.at"

```auto
fn main() int { return 2 + 3 * 4 }
```

### Probe av
recipe: "ac-av/core-run-v1"
entry: "d_main"
exit: 14

### Probe ac-native
#ignore
recipe: "ac/mir-native-scalar-v1"
entry: "d_main"
exit: 14

### Notes

复用1078既有Case ID，不另注册重复的arithmetic Case；同一Auto源码，公共预期为14；本AV阶段只记录AV实际结果，AC检查先#ignore，后续AC阶段打开再记录其真实结果。
A2R的生成/构建/运行检查由其适配器交付后在本Case追加。
`````

## 3 选择、覆盖与状态

### AV-ONLY阶段与检查级#ignore（2026-11-19用户补充）

**1078–1096只开发/验收AV。** 共享语料并不要求本轮运行所有后端；AC/A2R/A2C/A2PY及旧AVM对照等非AV检查先在其检查小节加`#ignore`或保持未选，不启动目标工具准备、绑定/编译/链接/运行，不因其未实现、结果不符或缺工具阻塞AV的work/review/merge。1078原“最小AC真执行”要求撤销，最小实证只需AV与数据组织/选择隔离。

裸`#ignore`紧跟检查标题、位于该检查属性之前；只影响这个检查，不能标整Case或共享Auto源码。例如上文同一Case里AV开启、AC-native忽略；以后新增/接入A2R检查时同样先忽略。Auto围栏内的同形文字仍是源码数据，不影响测试选择。

忽略/选择判定也应早于目标专属oracle文件、artifact、SDK与外部工具解析；这些依赖缺失不能成为AV运行前置。共享Auto/工程资产及公共文档结构仍须有效。FC-08还需覆盖“被忽略的非AV黄金文件不存在但AV正常运行”，由1078在原输入解析链落实，不创建旁路。

当前已有`CheckSpec.skip/SkipSpec`及父侧Skip/未选NotRun路径，但文档author创建检查时尚为`skip: None`，**#ignore尚未接通**。1078/T-03负责把检查级标记接到既有skip机制；T-01冻结默认阶段元数据：reason“AV阶段暂缓非AV检查”、issue“AV-ONLY-1078-1096”、阶段语料维护owner。真实后端producer/关闭条件仍在原义务映射中，不能覆盖已有平台/历史ignore。这里是拟实现合同，不是当前格式已接受的声明。

AV视图只选择AV目标，不能仅靠检查有没有ignore来界定本轮门；即使AC后续已打开，AV视图也不选它。已忽略行在inventory保定义/预期/原因，报告分列SKIP(IGNORE)与未选NOT_RUN，均不计PASS。AV required错误、AV零有效检查/全部被误ignore、公共文档/schema/重复ID或AV必需共享编译/IR准入错误仍必须红。未来未注册recipe先保Notes/义务表pending，不利用ignore绕过公共格式准入。

AV阶段只选AV及其共享正确性验证。混合Rust职责组的非AV执行腿按1078冻结的具名阶段选择/配置隔离；不因共享层变更自动引入ace-native/ace-a2x完整功能组，不为忽略check主动预构建非AV worker/SDK。不批量停用其它路线既有独立非AV回归；AV真正依赖的Rust/FFI provider和共享代码编译仍须正确。当前组名/选择能力如有不足，由1078在原固定组/nextest机制有界补接，不造新测试框架。

后续AC/A2R等实施任务逐check认领并移除**本阶段**新增#ignore，保原源码/oracle/平台条件与原ignore，按当前身份真实运行，红项归本目标任务修复；不能整文档批量删ignore。启用某检查不是把它加入AV视图。本目标完整门与1068/985最终多端签章仍要求各自required真实关闭，AV阶段的忽略不得当通过。


公共总suite和按特性/目标的选择复用现有TestCase/auto-case/固定组机制。1078拟定的 `case-av-language-parity` 是公共语料的**AV视图**，不再拥有独立的AV作者目录。特性view与AC/A2R view均只引用同一作者文档。

后端选择依据已注册adapter/recipe的真实目标、动作及显式leg信息；保原check ID，不假设所有AV检查都叫`av`。`--check av`只精确选择叫av的检查。1078/T-01核现有选择表达能力，T-03/T-04必要时在既有模型/选择器有限扩展按目标/动作过滤，不新增发现器或第二注册体系。当前未交的选择命令、字段和recipe只写为拟议接口。

suite须显式枚举作者文件，每个canonical文件在一次选择集中只include一次；重复Case ID、同名check冲突继续按现有准入规则拒绝，不能靠静默覆盖去重。多个视图不是多份源码；相同完整执行键在本次去重并集中只计一次，原义务各自引用该结果，不能把重复引用计成多个PASS。

支持/适用性仍归既有M2语言/后端义务表。后端尚未交付的required行记pending或NOT_RUN并保owner/关闭条件，不向活跃文档塞未注册recipe，也不从完整性分母删掉它。AV视图通过只证明AV已选行；完整多端门必须核缺失check、未跑、错目标和emit冒run。新增正式后端按实际能力逐Case补检查，不必等AV全路线或整个M2完成；全量对账由1096/1068/985分别按职责收口。

## 4 旧用例迁移与去重

### 4.1 范围与映射

1078先冻结当前真实可达输入与消费者：现有ACE `test/cases/**`的活跃AutoDown语言文档，旧 `test/vm/**`、`test/a2r/**`及适用a2c/a2ts/a2p/cookbook语言角色，Rust内联/外仓真实语言切片。506份VM文档快照只是其中一部分；其它族按实际发现/适用性对账，不推断它们全是语言必需例。ACE AutoDown都须在共同选择/职责分类中可追溯；非语言typed IR/ABI/运行时专项保原owner，不强造一份假Auto源码，也不能将公共语言用例改标“专项”逃避共源要求。

版本化迁移表拟为 `language/migration/legacy-map.tsv`，由1078建立，后续批次维护对应行。它只记旧义务与来源映射，不决定测试发现、支持状态或生成新expected；suite、既有CaseSpec/报告和M2义务表继续各负其责，不另造运行注册器或索引缓存。

每个旧case/check/判决角色至少记录：旧repo/路径/Case ID/check ID/动作；source与project/fixture内容指纹、入口/虚拟路径/模式；旧独立oracle内容/比较规则；provider/授权/预算/平台/ignore/性能条件；canonical特性路径/Case ID/check及新编译器目标；保留/合并/拆变体/仅历史的原因；producer owner与迁移验证状态。旧ID不能在映射中丢失，即使多个旧义务合并到同一canonical执行。

### 4.2 判定规则

| 情况 | 处理 |
|---|---|
| 完整输入闭包、模式、条件和语义相同，仅原后端目录不同 | 合并为一个canonical Case、一份Auto源，追加各后端checks，保每条旧义务映射 |
| 源码相同，目标产物或后端诊断不同 | 同Case独立checks/独立目标oracle，不能用一个目标的输出覆盖其它目标 |
| 仅文件名/测试标题/最终输出相同，源码或副作用不同 | 保独立Case，不判重复；如Hello和Hello, World!输入不同不能合并 |
| 源字节相同但虚拟路径、entry/script模式、依赖、授权、预算或适用条件影响语义 | 保显式变体与各原oracle，不因hash相同合并角色；确需共享字节时使用受控shared fixture引用 |
| 负例触发不同阶段/诊断、资源/恢复不变量不同 | 保独立判决角色/check；不得因“都报错/都返回42”折掉区分性断言 |
| 已知旧bug/已批准规范差异 | 保存原输入/历史oracle及裁定来源，新目标用已批准独立预期，禁止趁去重改expected |
| smoke/error-presence等弱旧检查 | 保原义务，增加有辨识力的新检查；不能把新增强检查假称旧行为已有证明 |
| typed MIR/ABC坏包、后端实现内部、正式压力或外仓完整产品 | 保原owner与原入口；可共享公共Auto部分，但不强转成假Auto程序、不关闭原专项条件 |

迁移过程中活跃文档不继续回读同名历史`.at`/`.expected.*`。相同语义源跨Case确有复用需求时，先复用/核实既有显式fixture引用；若文档author尚不支持，1078/T-01先作有界设计，T-03在现有CaseSpec/author/admission链补**受根约束、内容身份绑定、缺失即错**的有限引用能力。未交该接口前不得暗读其它Case代码、引入模板/字符串替换，或保两份可编辑源码。既有合法读取可先迁，不等待新引用全部完成。

### 4.3 每批顺序与出口

1. **冻结与分类**：逐原case/check/角色记输入、oracle、条件、所有活跃消费者/显式历史读者，按特性选canonical位置；同名不同源和疑似旧bug先单列。
2. **建立唯一作者**：完整搬运源码/工程/预期，保显式稳定ID/虚拟路径/模式；只对已判等价者合并。复用shared资产必须把其字节与依赖纳入身份，不做隐式搜寻。
3. **旧新并列验证**：在既有迁移/报告机制中比较规范化声明、每条旧角色映射、输入与oracle字节、原条件，本AV阶段只运行AV并对原声明/字节/条件/既有有效历史做对账，不要求重跑旧AVM、legacy A2R或新A2R。非AV运行对照和最终检查随对应后端启用再验，不自动生成expected。
4. **切消费者**：更新受影响suite/fixed group、worker/conformance include、CLI/工具及显式读者，均读canonical作者。公共source变化一次失效所有相关目标，单目标oracle变化只影响该目标；物理路径变化须保逻辑ID/正确判断旧报告stale，不能假装身份未变。
5. **撤重复活入口**：证明无剩余活跃消费者后才移除旧作者或将其明确冻结为只读历史。新旧两套在本批对照窗口有具名owner，批次完成后不留双活作者。历史M1签章输入/registry及旧归档收据不得重写；确需原样留存者只供具名历史回放，不再参与日常作者维护。
6. **逐原义务关账**：映射总数/原条件分列，canonical唯一Case/check数另列；合并降低文件数不等于减少验收义务。每批正式来源须证明零丢行/零无owner/零双活/原oracle未弱化，失败保旧选择可用且恢复后首次消费正确。

1078建立基线、唯一作者、#ignore/选择隔离和迁移机制并完成AV最小实证；1079–1095负责各特性真实迁移/补例，既有AC/A2R生产计划追加本目标checks；1096关AV与迁移完整性，1068核AV/AC/A2R，985/B最终接收。未使用的shared引用等机制按实际需要推进，不要求1078一次实现所有后端或先搬完所有旧目录。1078/T-01必须为缺失的目标adapter/check核具体producer Plan/task与接收方；现有生产任务无法覆盖时登记需重规划的具体缺口，不能仅写“外部owner”而不落实归属，也不能把1068接收任务再当实现任务。

## 5 必须可区分的验收控制

| ID | 输入/变化 | 必须观察的结果 |
|---|---|---|
| FC-01 单源多端 | 同Case的真实Auto函数由乘2改乘3 | 本轮仅AV读取新source指纹并按固定新场景值执行；其它目标旧证据标陈旧/待验，不要求运行、不阻AV，旧body不能冒AV通过 |
| FC-02 非重复邻例 | 同名hello源码/输出不同，或同源不同script/授权/虚拟路径 | 保两个语义角色，合并/删任一应使迁移完整性门红 |
| FC-03 目标oracle独立 | 仅篡改AC或A2R运行oracle，AV保持正确 | AV阶段：忽略的AC/A2R不dispatch、不阻AV；改AV oracle须红。对应目标启用阶段才执行其变异控制，届时不能以AV或emit替其结果 |
| FC-04 原义务守恒 | 多旧角色合并后删一条旧→新映射/原ignore条件，或丢一种资源断言 | 完整性验收拒绝；不能只检查canonical文件更少 |
| FC-05 作者与选择 | 活跃目录再放重复Case ID，旧suite仍读旧副本，或后端选择零命中 | 准入/消费者审计/选择门分别明确失败，不能双计PASS或静默覆盖 |
| FC-06 失效与迁移恢复 | canonical source/shared资产改字节，坏引用/错provider后恢复 | 受影响旧证据stale；本轮验AV的坏输入保旧/恢复首次读取，不从旧目录旁路；非AV记录待验而不执行 |
| FC-07 状态与真实动作 | required A2R仅emit、AC未运行/缺工具或missing check | 记对应阶段/未跑/缺口并阻完整多端签章，不能降required或改平台NA；这些非AV状态不阻当前AV门 |
| FC-08 忽略隔离 | 同Case的非AV检查加#ignore，制造该目标缺工具/运行失败/错oracle；另改AV oracle | 非AVprepare/dispatch=0且不阻AV；AV错oracle仍红；全部有效AV被ignore或零命中必须红，不能只靠SKIP得到AV成功 |
| FC-09 逐项恢复 | 某AC/A2R任务认领一个check并移除AV阶段ignore | 只在对应后端选择内运行并据实际结果判定；AV视图仍不选它，原平台/历史ignore保留，失败不能冒多端通过 |

FC-01..07按当前门的目标范围执行：AV计划只做AV和共同结构/义务守恒控制，跨端实际执行/目标oracle故障留其启用阶段。这些是未来实施的必验控制，均未运行。由1078原work任务和各特性迁移任务落实，独立review逐控制检查真实producer/consumer/报告；不以静态grep/计数代语义对照。既有performance、重内存、平台/ignore原条件不减；Windows不执行Linux/WSL，专用Linux证据独立。

## 6 计划接口与规范增量

1078交 `IF-ACE-FEATURE-CORPUS/1`（拟）及原 `IF-AV-PARITY-BASE`：含canonical路径/Case/check/source身份、实际选择接口、原义务迁移映射/批次状态、AV真实执行与非AV忽略/选择隔离证据，明确尚未接入的目标/行。它不声明AC/A2R全语言已经齐平。

后续语言/后端计划消费已交特性即可追加检查，不反等全部19个AV计划、AAC完整自举或UI/GPU产品。AAC的CL输入复用公共语言Case，Gen0/1/2/3只是不同编译器检查身份；AAC自身CS源码/中间IR专项保原范围。UI/AGE产品原源仍在其canonical外仓，由实际语言切片/引用与原GUI/GPU检查关联，不能复制成另一套可编辑demo。

拟在真实1078交付后新增 `docs/specs/auto-test-case/feature-organized-corpus.md`，同步现有document-cases/语言parity规范；本轮不把该设计写成current Spec已支持事实。旧已归档计划不重开；活动计划保任务ID、进度与独立阶段出口，只补作者位置/迁移规则/验收关系。
