# AutoDown 文档式测试用例

2026-11-10 · 作者格式已确认 · [完整双用例范例](examples/printing.case.ad) · [四例对照实施计划](../../plans/1002-autodown-document-cases.md)

实施排入 ACE M2-A。PLAN-1000 完成 M1 收尾后，首批以现有 ACE 的 B01、O03、B18、NB18 对照七项既有检查。本文描述完整目标格式，实施计划列明首批支持的子集。hello 是编写范例；其他 ACE 用例在试点实跑通过后逐批迁移。

一个 case 回答两个问题：给定哪一份 Auto 程序，各个目标分别应该产生什么？作者按语言行为组织文件，例如 `cases/basics/printing.case.ad`。打开文件即可读到源码、输出和生成文件全文。给现有用例增加一个目标，只在这段源码下面增加一节。目录名、计划号和启用的后端均不决定源码放在哪里。

仓里已有直接先例：[AutoGen 测试](../../../crates/auto-gen/tests/autogen_tests.md) 以二级标题分例，节内依次写数据、模板、预期输出；[Atom 测试](../../../crates/auto-lang/test/atom/atom_basics.md) 把输入和读回结果用横线隔开。多后端的输出个数和顺序会变，用有名字的小节定位。源码、流和金样放围栏里：空白、`#include`、`---` 和看似标题的源码都是数据。正文可以交代为什么这个行为值得测。

## Hello 的来源

[范例](examples/printing.case.ad) 第一例直接取现有 [pilot](../../../crates/auto-lang/test/cases/pilot/vm-basics.case.ad) 的显式 `main`、`hello\n` 和 Rust 金样。C 的两个块展示同源生成期望，参照现有 a2c 产物形状编写，待用这一份输入实跑及独立审定。第二例用用户指出的 a2r 顶层语句和它自己的 Rust 金样。旧 a2c 的同名例实际打印 `Hello, World!`，须保留其独立输入。范例位于设计目录，尚未登记为可执行语料。

## 写法

每个文档可有一个自由命名的一级标题。每遇到围栏外的 `## Case: <key>` 就开启一个独立用例，直到下一个 case 或文件末尾。key 在文档内唯一；单例和多例用同一种写法。每例须有一节 `### Auto` 和至少一项检查。代码围栏内即使出现同样的标题文字也属于载荷。

用例内的三级标题是固定词表。加载器取 AutoDown 标题节点的纯文本，去除首尾空白后按表区分大小写、完整匹配；其中 `<目标>` 和 `<check-id>` 是下表规定的参数。词表决定调用哪个后端、执行什么动作、比较哪种结果。围栏语言标签须与该载荷相符；例如 `### C` 下出现 `rust` 围栏，加载时在该行报错。添加新后端时登记一种标题及其能力，用例文件不能自行定义别名。

| 三级标题 | 执行动作与内容 |
|---|---|
| `Auto` | 读取本例的 Auto 源码。单文件用一个 `auto` 围栏；多文件改在节下写多个 `#### 相对路径`，各有一个围栏，默认入口为 `main.at`。 |
| `VM` | 运行这份源码，比较标准输出（stdout）；写一个 `text` 围栏即可，空围栏要求零字节。 |
| `VM stderr`、`VM exit`、`VM result` | 各比较标准错误、退出码、VM 返回值。与同例的 `VM` 合成一次运行，任一观测都可以单独声明。exit 的 `text` 围栏中写一个整数。 |
| `C`、`Rust`、`Python` | 各自调用 C、Rust、Python 转译器，比较该目标生成的文件。单产物写一个对应语言的围栏（`c`、`rust`、`python`）；多产物用 `#### hello.h`、`#### hello.c` 这样的实际相对产物名分块。逐文件比较完整路径集合与内容。 |
| `<目标> diagnostic` | 目标取 VM、C、Rust、Python；各走对应的运行或转译入口。以 pair 声明失败阶段 `stage` 及可用的稳定 `code`，可加 `text` 块核对完整消息。工具缺失或超时不能满足语言诊断。 |
| `Probe <check-id>`、`Notes` | 前者使用下文所列的具名 ACE 探针；后者可自由写说明。普通段落也可写在源码与预期之间。 |

例如 `### C` 的完整含义是“将本例 Auto 转译为 C 并核对生成文件”。在这版草案中，`### Expected C`、`### c` 和未知标题都会报出行号和合法写法；不会因拼错而漏跑一个目标。跳级、孤立载荷、重复观测和重名产物也在加载时报告。

单文件程序默认虚拟文件名为 `<key>.at`、入口模式为 program、模块名为文件 stem。单产物的默认逻辑名为该 stem 加目标扩展名，如 `hello.rs`；需要指定路径时即使用四级标题。多文件工程的模块名默认取入口的 stem。运行节隐含正常完成，诊断节隐含指定阶段失败；同一动作同时要求成功和失败是格式错误。各目标独立声明，未写 Python 的 case 也可完整运行 VM 和 Rust；suite 原有的必测项缺失仍报错。

### 可省略的元数据

属性紧跟所属标题，每行一对 `key: value`，遇到空行、下一标题或围栏即结束；围栏里的同形文本始终是数据。字符串、整数、布尔、字符串列表均按静态值读取；源码和期望只从围栏取。文件没有一级标题时可在第一个 case 前写文件属性。只接受该层登记的键，重复或未知键提供行号。日常例子完全可以没有 pair。

```ad
# Resource lifetime
tags: ["ace-m1", "resource"]
links: ["PLAN-915"]

## Case: stale-place
id: "ace.m1.r03.stale"
tags: ["r03"]
```

文件层只设共享的 `tags`、`links`，都默认空列表。case 可各自补充，两级按出现顺序合并去重。上例最终有三个标签和一个计划链接，下一例只继承文件层的值。case 的 `id`、入口和模块名只作用于本例；源码节的 `path`、观测节的 `check-id`、载荷的比较选项各限所属节点。默认 case ID 是 suite 逻辑根下的相对路径（去 `.case.ad`）加 `#<key>`；key 是文档内唯一的 ASCII 小写字母、数字和连字符组成的名字。迁移、重命名或跨目录移动需维持历史报告身份时显式写旧 id。例如相对路径为 `basics/printing.case.ad`、用例名为 `hello` 时，默认 ID 是 `basics/printing#hello`。普通 check ID 默认用固定目标/动作（如 vm、rust、c）；同一文档所有实际 case ID 和同例内 check ID 分别检查唯一，跨文档的 case ID 在 suite 选择集检查唯一。格式版本由加载器的文档式 profile 管理；普通作者不用写 schema、adapter、action、steps 或文件引用。

普通观测节可用 `check-id` 保留历史 check 名称；属于同一次运行的各节若给出不同 ID 则报错。源码节的 `path` 可保留原虚拟文件名，case 的 `module-name` 可保留后端需要的旧模块名。例如 pilot 的 hello 迁移时应显式保留 `id: "lang.basic.hello"`、`path: "main.at"` 和 `module-name: "hello"`。示例文件省略它们以展示新写一例的默认值。

现有 [B18 不可变赋值负例](../../../crates/auto-lang/test/cases/ace-m1/immutable-avm-920.case.ad) 的预期可直接写成：

```ad
### VM diagnostic
stage: "type"
code: "auto_name_E0203"
```

这断言 VM 编译这份程序时报告的类型诊断。相同文字若只出现在 stderr，检查仍然失败。

### 原样文本、无尾换行和特殊输入

文档是 UTF-8。文档式文本默认以 LF 表达逻辑行，保留空格、空行和最后一个换行；围栏后紧接闭栏表示空串。块内需要三反引号时外层用四反引号或波浪号，闭栏与开栏同字符同长度。某一文本块有 `final-newline: false` 时只移除紧邻闭栏的一个逻辑换行。源文件和期望文本都遵守此规则。流及产物默认按构造出的期望字节精确比较；需要同时将实际输出的 CRLF 规整为 LF 时显式写 `compare: "lf"`。迁移时核对构造后的字节与旧金样/比较器，禁止 trim。确需逐字节 CRLF、非 UTF-8 或二进制资产时，允许在该文件节点显式用受 suite 根约束的 fixture 引用替代围栏；禁止自动搜寻同名 expected。虚拟路径须通过既有安全校验。

## ACE 场景怎样落到正文

[R03 的现行声明](../../../crates/auto-lang/test/cases/ace-m1/resource-915/resource-915.case.ad) 有五个独立检查。四个修改操作各需父侧核验 stale-place 的类型化观测；合法邻例真实跑源码，退出值是 130。设计中把原 `R03-av.at` 全文写入一节 `### Auto`，指定 `path: "main.at"` 保持入口身份；四个具名 probe 节分别声明对应的已注册场景，节名保留 check ID。下面展示其中两节的完整参数形式，其余三节以同样形式保留各自入口和 ID。

```ad
### Probe av-r03-stale-push
recipe: "ac-av/resource-run-v1"
entry: "place:push"

### Probe av-r03-legal-neighbour
recipe: "ac-av/collections-source-run-v1"
entry: "main"
exit: 130
```

这两个已登记 recipe 的默认步骤都是一次 run。push、insert、remove、replace 的独立类型断言继续由 [judge_resource](../../../crates/auto-lang-core/src/test_cases/ac_av.rs) 执行。展示用的说明可以解释预期陷阱；父侧必须观察真实见证，不能从说明推算 PASS。更复杂的多步 recipe 在有等价表示前继续走旧入口。普通打印用例无需了解探针参数。

## 加载与执行

1. suite 仍显式选择文件。加载器一次读入每份选定文档，用现有 AutoDown 块树辨认标题、围栏和顺序，以原文范围索引源码、属性、产物与行号。按 case/check 选择后才物化相应内容。未选后端不启动，不为每个期望另开磁盘文件。以打开次数、字节和冷/热加载时间验证收益。
2. 文档解码器将每个 case 降为现有检查模型可消费的输入、独立期望和来源范围。适配器只接收源码及执行参数；父侧拿实际产物或诊断与原文期望比较。Rust/C/Python 文本检查只要求生成，编译或再次运行生成物必须有自己的检查。报告附 case ID、check ID、产物路径、文档行号和差异。
3. v1/v2 继续按原有 schema 精确解码。出现结构性 case 标题则选文档式入口，同文件出现旧合同和新 case 双重入口、未知目标或不支持的观测均明确报错。实现先复用 avm/run、a2r/emit；C/Python 接各自真实语言转译器，ACE probe 仍走具名 worker 和父侧裁判。各适配器能力须单独验收。
4. 按语言主题迁移，先比较输入字节、虚拟路径、入口、选项、期望、历史身份及消费者。三路同名 hello 继续各自履行旧义务。新旧格式在独立选择集对跑后切换 suite，再移除已无消费者的旧副本；输入不同只能并列为不同 case。测试框架在报告中继续区分未执行、工具错误、断言失败和通过。
