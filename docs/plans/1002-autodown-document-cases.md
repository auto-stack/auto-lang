---
plan_id: PLAN-1002
status: drafting
feature_name: autodown-document-cases
author: [Codex]
created_at: 2026-11-10
updated_at: 2026-11-13
plan_revision: 7
current_step: 0
total_steps: 4
depends_on: [PLAN-1000]
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-test-case/document-cases.md]
touched_goals: []
affects: [auto-test-case]
---

# [PLAN-1002] ACE M2 测试框架：AutoDown 文档式用例四例对照试点

## 0. 变更摘要

2026-11-12 r6：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

本计划排入 [ACE M2 路线图](../design/strategy/ace-m2-delivery-roadmap.md)的 A 阶段测试工作。M1 进入条件已满足。本地正式标签 `v0.6-ACE-M1` 指向 `b923362f4`，主线最终验收见 [PLAN-1025](archive/1025-ace-m1-mainline-identity-and-final-signoff.md)。已确认的[作者格式](../design/testing/autodown-document-cases.md)用于此次试点。实现一条贯通现有执行器的文档读取路径，把四个真实 ACE 用例的源码和预期写进一份可读的 AutoDown 文件，按原有七项检查逐行对照。以[双例 hello 范例](../design/testing/examples/printing.case.ad)作为编写参考。试点期间旧 suite 继续作为正式入口，新 suite 单独显式运行。

## 1. 目标

验证三个问题：作者能否在一份文档里看全源码和预期；两个入口能否运行同一组后端并得出相同观测；写错预期或遗漏检查时能否准确定位。成功后才启动其余 ACE 用例的逐批迁移。C/Python 生成、多文件 C、R03 资源见证和多步会话归后续批次，保留原有覆盖。本文的样本数按 case ID 计，负控属于验证手段。

## 2. 架构方案

suite 发现后按文档结构分派，旧合同走原路径。新入口复用既有 AutoDown 块树及围栏原文范围，将标题、pair、内联源码和预期降为同一套执行模型。生成文件的预期目前经外部文件解析，首批须给解析和比较链补入内联载荷及其源位置；禁止为了适配旧入口先写临时 expected 再读回。实际产物照常由后端独立生成。保持原有输入限制、路径检查、父侧裁判、退出码和工具身份。

## 3. 技术栈

Rust。读取链 `crates/auto-test-case/src/{discovery,document,raw_ranges,contract,ir}.rs`；执行与比较经 `crates/auto-lang-core/src/test_cases/`；面向用户的入口为 `crates/auto-lang/src/bin/auto-case.rs`。实现分支 `plan-1002-dev`，worktree `D:/autostack/.wt/lang-1002/auto-lang`；实施期试验输出统一用 `D:/autostack/plan-artifacts/plan-1002/`，按 old/new/control 分开。

## 4. 需求分析与背景调查

2026-11-10 用户确认格式，要求先设计 3～5 个实际 case 的实施与对照计划，试点成功再扩散。静态取材基线 `003fa8816`。现有 [profile v2](../specs/auto-test-case/profile-v2-native.md) 和 [v1](../specs/auto-test-case/profile-v1.md) 定义执行合同与观测语义。本次选例均从现有 ACE suite 原位核对；下表是旧合同规定的预期，实际运行数据在 T-01/T-03 取得。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：AutoDown文档式四例七检查试点。当前知识以当地Specs/本Plan所引现行合同为准。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [crates/auto-test-case/src/document.rs](../../crates/auto-test-case/src/document.rs)：已有文档读取，补inline payload/原位置，不写临时expected再读；读取SHA256前16 `4c69434e5ce2743c`

- [crates/auto-test-case/src/ir.rs](../../crates/auto-test-case/src/ir.rs)：复用resolved载荷/产物/oracle模型；读取SHA256前16 `1e256f0cf852be7a`

- [crates/auto-lang-core/src/test_cases](../../crates/auto-lang-core/src/test_cases)：原执行与比较，不增第二runner；目录职责已核，确切符号/闭包开工再核

### 2026-11-13 M1 标签后的源码复核

主检出 `84c575a54` 包含标签提交；从标签到本次复核，试点源码和测试驱动无代码差异。自 r6 的 `f9fb3854b` 之后，native 父端增加了 aggregates 诊断准入，固定测试驱动增加了分组截止，正式压力已分组。四例源文件、旧合同、B01 Rust 金样及三个旧 suite 均可原位对照。旧证据绑定具体构建，T-03 以同一冻结构建重跑两种写法。

[PLAN-1016](archive/1016-ace-comptime-short-repro-and-cost-repair.md) 已交付正式压力隔离。[PLAN-1017](1017-ace-perf-m2-cost-and-workload-optimization.md) 负责按标签后的真实 nextest 发现冻结正式用例集合 S0，再测同一集合的优化前串行总成本 B0。1025 将六项 M1 必测检查恢复到常规命名空间，1016 时的局部计数只作历史参考。1002 的四例属于 `runner = "case"` 的文档正确性试点；`ace-perf` 在 ACE workspace 用 nextest 选择根级/嵌套的 `performance::`，使用单测试线程、14400 秒父截止。

两条路径有实际交点。[turnaround.rs](../../experimental/ace/tests/conformance/tests/host/turnaround.rs) 的正式 IPC 检查生成旧格式合同，以 1 MiB 输入运行 30 个样本，每个样本在 `run_suite` 前启动整段计时；另有一次采样的常规正确性检查复用该路径。建议 1017 先在注明的提交冻结 S0、用短探针定位并在独占时窗取得 B0，再落地 1002 的读取器。若 1002 先行开发，1017 仍可在固定的改动前构建采基线；共享执行路径若跨版本，用同负载短探针区分格式改动的成本。两个计划无整包依赖；下一批语料迁移仍以本计划四例对照通过为前提。

## 5. 详细设计

旧文件均相对于 `crates/auto-lang/test/cases/ace-m1/`。固定以下四个 case 和完整 check 集，双方独立的 suite 保留相同 case/check ID。

| 用例及旧声明 | 输入 | 旧 check 与必须对照的结果 |
|---|---|---|
| `ace.m1.b01` · [标量](../../crates/auto-lang/test/cases/ace-m1/b-scalar.case.ad) | [B01.at](../../crates/auto-lang/test/cases/ace-m1/src/B01.at)，`2 + 3 * 4` | `ac-native`、`av` 各真实退出 14；`autovm` 的 legacy-display 为字符串 `14`；`a2r-snapshot` 生成且逐字节匹配 [m1_b01.rs](../../crates/auto-lang/test/cases/ace-m1/oracle/m1_b01.rs)。共 4 项。 |
| `ace.m1.o03` · [控制台](../../crates/auto-lang/test/cases/ace-m1/console-863/console-863.case.ad) | [O03.at](../../crates/auto-lang/test/cases/ace-m1/console-863/src/O03.at)，分别打印中文和重音字母 | `av-console` 的一次运行同时比较 stdout `你好\n\n`、stderr `é\n`、exit 0。共 1 项。 |
| `ace.m1.ia920.b18` · [不可变赋值](../../crates/auto-lang/test/cases/ace-m1/immutable-avm-920.case.ad) | [B18.at](../../crates/auto-lang/test/cases/ace-m1/src/B18.at)，给 let 绑定再赋值 | `autovm` 必须报告阶段 `type`、诊断码 `auto_name_E0203`。共 1 项。 |
| `ace.m1.ia920.nb18` · 同一旧声明 | [NB18.at](../../crates/auto-lang/test/cases/ace-m1/src/NB18.at)，相同位置改为 var | `autovm` 成功返回 legacy-display 字符串 `2`。共 1 项。 |

拟在 `crates/auto-lang/test/cases/ace-m2/document-pilot-1002/` 建独立试点目录，含一份 suite 和一份四例 `.case.ad`。每例一段 Auto，B01 的 Rust 金样、O03 的两段输出全文就地保留。沿用 program、虚拟路径 main.at、B01 的 module-name=m1_b01、原 tags/links 与各 check ID。迁移可用显式 pair 稳定旧身份，新写 case 的默认值另由格式测试验证。VM result 比较文本 `14`、`2` 时按设计显式控制末尾换行。

首批执行标题为 Auto、Rust、VM result、VM diagnostic 和具名 Probe。B01 的两个 Probe 依旧分别以 `ac/mir-native-scalar-v1`、`ac-av/core-run-v1` 运行，各自映射 mn/av 的 entry=d_main；O03 使用 `ac-av/console-run-v1`，保留同入口、stdout/stderr 两个 caps 和一次 run 步骤。O03 具体写法建议在 `### Probe av-console` 下写 recipe、entry、caps、exit 四种 pair，以及 `#### stdout`、`#### stderr` 两个 text 围栏。两个流归同一 check，保留原始 UTF-8 和末尾换行。未接通的标题或 recipe 给出定位错误。

### 5.1 四个用例的完整文件

拟交付文件为 §5 所列目录下的 `pilot.case.ad`。下面整个代码块就是文件正文。文件层只共享 `ace-m1` 标签。显式 ID、虚拟路径和 check 名用于沿用 M1 的报告身份。B01 的模块名只传给 a2r，其余三条执行路径按原选项运行。

````ad
# ACE 基础行为
tags: ["ace-m1"]

## Case: precedence
id: "ace.m1.b01"
tags: ["four-path", "scalar"]
links: ["PLAN-852"]
module-name: "m1_b01"

### Auto
path: "main.at"

```auto
fn main() int { return 2 + 3 * 4 }
```

### Probe ac-native
recipe: "ac/mir-native-scalar-v1"
entry: "d_main"
exit: 14

### Probe av
recipe: "ac-av/core-run-v1"
entry: "d_main"
exit: 14

### VM result
check-id: "autovm"
final-newline: false

```text
14
```

### Rust
check-id: "a2r-snapshot"

#### m1_b01.rs

```rust
// Auto-generated by a2r transpiler

fn main() -> i64 {
    return 2 + 3 * 4;
}
```

## Case: unicode-console
id: "ace.m1.o03"
tags: ["console", "utf8-newline"]
links: ["PLAN-863"]

### Auto
path: "main.at"

```auto
fn main() int {
 print("你好\n")
 eprint("é")
 return 0
}
```

### Probe av-console
recipe: "ac-av/console-run-v1"
entry: "d_main"
caps: ["console.stdout", "console.stderr"]
exit: 0

#### stdout

```text
你好

```

#### stderr

```text
é
```

## Case: immutable-assignment
id: "ace.m1.ia920.b18"
tags: ["diagnostic", "immutable-avm-920"]
links: ["PLAN-920"]

### Auto
path: "main.at"

```auto
fn main() int {
    let x int = 1
    x = 2
    return x
}
```

### VM diagnostic
check-id: "autovm"
stage: "type"
code: "auto_name_E0203"

## Case: mutable-assignment
id: "ace.m1.ia920.nb18"
tags: ["diagnostic", "immutable-avm-920"]
links: ["PLAN-920"]

### Auto
path: "main.at"

```auto
fn main() int {
    var x int = 1
    x = 2
    return x
}
```

### VM result
check-id: "autovm"
final-newline: false

```text
2
```
````

B01 的两个 `exit: 14` 是两个执行后端各自的预期退出值。`VM result` 的期望是两个字节 `31 34`（十六进制）。O03 的 `你好` 后特意留了一行空行，stdout 应有八个字节 `E4 BD A0 E5 A5 BD 0A 0A`。stderr 应有三个字节 `C3 A9 0A`。两段都属于同一次 `av-console` 调用。B18 只有在实际报告类型阶段和诊断码均正确时通过。NB18 用成功返回值检验同一处修改对可变绑定合法。

### 5.2 登记与默认值

同目录的 `pilot.suite.ad` 只登记上述文件。发现用例按二级标题顺序，B01/O03/B18/NB18 分别产生 4/1/1/1 个检查。拟新增固定组 `case-ace-document-pilot-1002` 在 `scripts/test-groups.toml` 使用 `runner = "case"`、`domain = "ace"`、显式 suite 路径和 `all = false`，与旧组分开选取。

````ad
```test-suite
suite {
  schema: "autolang.suite/2"
  id: "ace-document-pilot-1002"
  include: ["pilot.case.ad"]
}
```
````

新写的普通用例可以省略迁移身份。下面是供格式单测使用的最小 Rust 快照。若这一段单独写在 suite 根下的 `basics/printing.case.ad`，则默认 ID 为 `basics/printing#hello`，输入名为 `hello.at`，生成文件名为 `hello.rs`，检查 ID 为 `rust`。它演示零 pair 的作者体验，正式对照仍以 §5.1 的四例为准。

````ad
## Case: hello

### Auto

```auto
print("hello, world!")
```

### Rust

```rust
// Auto-generated by a2r transpiler

fn main() {
    println!("hello, world!");
}
```
````

### 5.3 从作者文档到执行器

| 改动位置 | 实施内容 | 可观察结果 |
|---|---|---|
| `document.rs`、`raw_ranges.rs` | 用 AutoDown 块树识别结构。将顶层标题、紧随的属性、直接围栏与原文位置配对。四级标题按父节解释，Rust 下为产物路径，Probe 下为输出流。文本按 LF 构造，只在明确要求时剥掉一个尾换行。 | §5.1 一次读取产生四个 case 范围。嵌在围栏中的标题、属性和空行均属于载荷。拼错标题、混合旧合同与新 Case、重复流或未闭围栏在启动后端前给出文档行。 |
| `discovery.rs::load_suite`、`contract.rs` | 当前发现路径只收集 `test-case` 围栏。在选中文件内分派新旧格式，将新写法编译为现有 CaseSpec/CheckSpec 及来源范围，沿用现有类型、限额和重复 ID 校验。按 recipe 确定 mn 或 av 配置，三个试点 recipe 默认各一步 run。 | list 出现七条 RUN。O03 的 caps、entry、exit 和两段流归到同一条 probe/run。其他 case 不继承 O03 的配置。 |
| `ir.rs::resolve_case`、`resolve_oracle` | 对选中的源码与产物提供围栏范围来源；旧 sidecar 保留其有界文件读取。内联产物携带逻辑路径、原文位置、比较方式、期望字节和摘要。构造后交同一套父侧比较器。 | B01 真实生成 `m1_b01.rs`，逐字节比较 80 字节。选测 B18 时仅运行其 autovm，测试数据目录无写入。 |
| `test_cases/inventory.rs`、`auto-case.rs` | 当前 inventory 会将 case 与旧合同围栏逐个配对。改用共同解码产生的 case/check 范围，区分实际文件依赖与文档内联载荷。接入 list、inspect、lint、inventory、identity-inventory，以及定向和整套运行。 | 固定组能发现七行并按 recipe 准备工具。清单里能定位文档范围，不会把内联 Rust 产物名当作要打开的外部文件。清单中的源码和期望身份与本格式的执行报告相符。 |
| `test_cases/mod.rs`、`materialize.rs`、`stale.rs` | 核查执行、候选差异和身份重核对产物 `file` 的依赖。区分内联位置和旧文件路径，使修改内联金样可定位、可判旧报告过期；候选差异只写隔离输出目录。 | 修改 B01 金样一个字节后，生成产物仍来自同一输入；原报告的期望身份失效，差异指向 `#### m1_b01.rs` 的围栏。 |

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r6当前）

**具体输入与期望。** 原件[crates/auto-lang/test/cases/ace-m1/src/B01.at](../../crates/auto-lang/test/cases/ace-m1/src/B01.at)（下列节选L1–L1；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
fn main() int { return 2 + 3 * 4 }
```

本包所需独立结果：严格保四case七check：B01四条、O03一条、B18/NB18各一条；新旧源码/期望字节相同，实际后端观测一致，不新增/漏算检查。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** AutoDown块树/原文围栏范围→现ResolvedCase/ResolvedArtifact/ResolvedOracle→原父执行/比较器→带新文档源位置的同check结果。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
enum ExpectedPayload { File(ResolvedFile), Inline { bytes: ByteBuffer, origin: SourceSpan } }
struct DocumentCase { identity: CaseId, source: SourcePayload, checks: CheckSet, ranges: SourceMap }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [crates/auto-test-case/src/document.rs](../../crates/auto-test-case/src/document.rs) | 已有文档读取，补inline payload/原位置，不写临时expected再读 |
| [crates/auto-test-case/src/ir.rs](../../crates/auto-test-case/src/ir.rs) | 复用resolved载荷/产物/oracle模型 |
| [crates/auto-lang-core/src/test_cases](../../crates/auto-lang-core/src/test_cases) | 原执行与比较，不增第二runner |

**状态、错误与资源。** 改Rust一字节、删stdout末尾LF、改诊断码各只红对应check；删B01 AV即使余下绿也因缺检查失败。 新旧suite分开显式运行，旧正式suite不替换；源码/expected内联一次读取、外部读取0按原合同测，输出/实际产物不复用期望造假。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/cases/ace-m1/b-scalar.case.ad](../../crates/auto-lang/test/cases/ace-m1/b-scalar.case.ad) | B01原4 checks/14/旧Rust金样逐字保 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `758863818ad3c5b5` | 原ace.m1.b01：ac-native/av/autovm/a2r-snapshot四check；原任务 T-01/T-02/T-03 |
| [crates/auto-lang/test/cases/ace-m1/console-863/console-863.case.ad](../../crates/auto-lang/test/cases/ace-m1/console-863/console-863.case.ad) | O03一次运行核双流UTF-8与LF | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `aa519502ec2f563e` | 原ace.m1.o03：av-console一次双流check；原任务 T-01/T-02/T-03 |
| [crates/auto-lang/test/cases/ace-m1/immutable-avm-920.case.ad](../../crates/auto-lang/test/cases/ace-m1/immutable-avm-920.case.ad) | B18 type/auto_name_E0203、NB18显示2 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `96ef60067b3c1dba` | 原ace.m1.ia920.b18/nb18：原两autovm check；原任务 T-01/T-02/T-03 |
| [docs/design/testing/examples/printing.case.ad](../design/testing/examples/printing.case.ad) | 复用已交作者格式范例，未接通C形态不扩大试点 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `0620d69b39a92a1b` | 作者格式参考，不新增试点case；原任务 T-01/T-02/T-03 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

本补充不增加首批case/check：仍4例7行，作者格式与原V-01..04已足够的部分直接复用；只补载荷类型/代码职责与旧新来源映射。后续推广仍须试点真实通过。

**原任务内的决定和交接。** 标题/围栏/继承pair/重复ID/空流/无尾换行及UTF-8源范围精确规则；保原四例七行、C/Python/多文件与R03后续，不提前扩大试点。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-01`、`T-02`、`T-03`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-04；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | 变更 | specs 目标 | 实施前 → 实施后 | 理由 | 验收 |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/document-cases.md | 显式 v1/v2 合同入口 → 增加文档式试点入口、已接通标题/探针表及未验证面 | 分开记录已对跑能力与后续推广范围 | AC-01..05 |


| SD-DETAIL | add | `docs/specs/auto-test-case/document-cases.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-04 |
## 6. 测试设计

**对照方法。** 在同一冻结构建、后端二进制、平台和配置下执行新旧两份输入。旧侧分别通过原 suite 的 `auto-case module-case --suite <旧suite> --case <完整ID> --out <old/ID>` 读取 `module-case.json` 中的 selected_report；不把同 suite 其他未选行计为失败或成功。新侧通过 `auto-case run --suite <新suite> --out <new>` 一次运行全部四例。仅对准七个选中行，另核新 suite 完整 gate。试点可单独选测，禁止将两个同 ID 的 suite 合并后重复计入覆盖。

| 验证 | 输入和操作 | 必须观察到 |
|---|---|---|
| V-01 格式与作者负担 | 四例同文档；另测零 pair、文件 tags/links 继承、case 隔离、精确标题与围栏、空流和无尾换行。修改标题大小写、给 Rust 节放 c 围栏、制造重名 case。 | 文档内直接读全四份源码和全部载荷。有效格式的 ID/默认值可复算；无效格式在执行前报出行号。记录每例旧/新需打开的文件数、必写 pair 数；以实际文件打开计数验证每份文档只读一次，普通源码和预期外部读取为零。 |
| V-02 对跑 | 先比较四份源码字节、虚拟路径、入口、选项和七行期望，再真跑新旧两侧。按 (case ID, check ID) 连接两份结果，预先固定 §5 行集。 | 各侧 4 例、7 条选中行全部真实执行且通过。逐行核 adapter/action、语义终态、实际返回/退出、流字节、生成路径及字节、诊断阶段/码。缺失的 actual 通过执行器实际观测取证。允许源文档位置、suite ID、表示层指纹、时间等差异，单列说明；源码或后端身份漂移不能归入允许项。 |
| V-03 负控 | 在隔离副本中分别改一个 Rust 字节、删 stdout 的最后一个 LF、改诊断码；保留 NB18 作合法邻例。另删 B01 的 av 检查。 | 前三种各自只使目标 check 失败，报告指出 case、check、出错内容及新文档位置，其他行照常。最后一种即使剩余检查全绿，对账仍因缺一行失败。基础设施错误绝不能满足 B18。 |
| V-04 兼容与成本 | 普通 v1/v2 接受和拒绝用例、旧 suite 门；记录新旧文件读取、读入字节、冷/热加载与后端执行分别耗时。 | 旧入口和比较器照常工作，测量报告标明环境、样本及误差；首批不以假定加速倍率决定推广。 |

### 6.1 可重放的对照命令

以下 PowerShell 片段从专属 worktree 根执行。T-01 先取旧侧基线，T-03 用同一冻结构建重跑两侧。使用该树 `scripts/fixed-tests.py::run_case` 同款工具准备流程，根据 inventory 中的 adapter/recipe 通过 `scripts/test-governance/prepare_tools.py` 准备 parent、AV、native worker，并在以下命令所在进程应用其环境。记录二进制路径与摘要。`$out` 取本次已登记、独占的材料输出子目录；多次运行用不同目录以便保留首次失败。

```powershell
$runner = (Resolve-Path '.\target\debug\auto-case.exe').Path
$cases = 'crates/auto-lang/test/cases'
$old = "$cases/ace-m1"
$new = "$cases/ace-m2/document-pilot-1002/pilot.suite.ad"
$out = 'D:/autostack/plan-artifacts/plan-1002/pair-01'
$baseline = [ordered]@{
    'ace.m1.b01'        = 'ace-m1-base.suite.ad'
    'ace.m1.o03'        = 'console-863/console-863.suite.ad'
    'ace.m1.ia920.b18'  = 'immutable-avm-920.suite.ad'
    'ace.m1.ia920.nb18' = 'immutable-avm-920.suite.ad'
}
foreach ($id in $baseline.Keys) {
    $suite = Join-Path $old $baseline[$id]
    & $runner module-case --suite $suite --case $id --out "$out/old/$id"
    if ($LASTEXITCODE -ne 0) { throw "旧例 $id 失败，见 $out/old/$id" }
}

& $runner lint --suite $new
if ($LASTEXITCODE -ne 0) { throw '新文档未通过格式检查' }
& $runner inventory --suite $new
if ($LASTEXITCODE -ne 0) { throw '新 suite 的用例清单生成失败' }
& $runner inspect --suite $new --case ace.m1.b01
if ($LASTEXITCODE -ne 0) { throw 'B01 的源码或预期未能解析' }
& $runner run --suite $new --out "$out/new"
if ($LASTEXITCODE -ne 0) { throw "新 suite 失败，见 $out/new" }
```

### 6.2 七行如何判定

1. 从四份 `old/<id>/module-case.json` 分别取 `selected_report.results`，要求 `selected_gate_exit = 0`。每份只含该 ID 的行。原 suite 的其他 case 可以显示 NOT_RUN。新侧取 `new/report.json` 的 `results`，要求完整运行返回 0。两侧都核 `load_errors`、`coverage_gaps` 为空。
2. 用 `(case, check)` 对应 §5 表。先检出重复，再比较成员和数量。检查每一行的 adapter/action、状态和实际证据，不能仅按顺序或返回码配对。每次对照保留“身份、旧观测、新观测、首个差异”四列。
3. B01 与 NB18 的 VM 返回值从运行生成的 `result.txt` 取字节。B01 的 Rust 从报告 `artifacts` 中找逻辑路径 `m1_b01.rs`，读实际产物再核 hash 与 80 个原始字节。O03 从 worker 本次返回给父侧的 console capture 解码两段流。B01 的两项退出值、O03 的退出值以及 B18 的阶段/码，从各自执行端传给父侧的实际观察取得。若成功行只持久化状态，在 T-03 的试点比对入口从父侧裁判接收实际值的位置取证，记录与该次运行的对应关系。对照表的实际值不能从期望反填。
4. 按 §6 的 V-03 分别运行四份隔离变体。改 Rust 金样、stdout 或诊断时，完整 suite 的期望是退出 1、目标行 FAIL、其余六行 PASS。删除 B01 的 `### Probe av` 后，独立的七行清单必须点出缺少 `(ace.m1.b01, av)`。每个变体都从同一未改样本单独派生。

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-04；T-01/T-02/T-03 | 严格保四case七check：B01四条、O03一条、B18/NB18各一条；新旧源码/期望字节相同，实际后端观测一致，不新增/漏算检查。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-04；T-01/T-02/T-03 | 改Rust一字节、删stdout末尾LF、改诊断码各只红对应check；删B01 AV即使余下绿也因缺检查失败。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-04；T-01/T-02/T-03 | 新旧suite分开显式运行，旧正式suite不替换；源码/expected内联一次读取、外部读取0按原合同测，输出/实际产物不复用期望造假。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-04；T-01/T-02/T-03 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

### 当前固定组和旧消费者验收

T-02/T-03 对试点固定组核 `runner = "case"`、`all = false`，以 `cargo t --select-only case-ace-document-pilot-1002` 核对选择，再实跑该组。T-03 在原定四个完整组及新组之外，用 `cargo t infra-fixed` 核固定组配置。共享读取/报告入口的普通宿主消费者通过 `cargo t ace-host` 核验，其中包括 BASE 的 97 行、B01 错退出码反控及单样本 IPC 对照。保留原 M1 suite 的物理输入路径与旧 inventory 输出语义：M1 证据 reader 会核 `input_paths`，AV 的运行和 wire 基准也以 `include_str!` 直接读取 B01.at。新文档的内联位置单独表达。后续推广前逐个迁移这些消费者。

V-04 记录每次加载选中文档和载荷的打开次数，另列工具准备、发现、执行、清理及整命令时间；区分 inventory 和 run 两次独立进程。四例加载数据只用于比较作者格式。正式串行全套、同工作量十倍目标及新增负载另账归 1017。

## 7. 验收标准

- [ ] AC-01（work）四例在一个可独立阅读的文档内；V-01 的正反例和源位置符合设计。
- [ ] AC-02（work）旧、新各自七行均通过 V-02；新增或漏掉行、SKIP、NOT_RUN、工具错误或实际观测不同均无法过关。
- [ ] AC-03（work）通过 V-03，证明新写法能区分错字节、错换行、错诊断与缺项。
- [ ] AC-04（work）旧 suite 与旧基准完整保留，试点作为单独显式入口，V-04 留下加载与可读性对照记录。
- [ ] AC-05（review）独立审查四例七行和故意改错产生的真实失败，核对已支持子集与 SD-01；形成“可按批推广”或列明未通过项的结论。

## 8. 执行步骤

| 任务 / owner_stage | 依赖 | 实施与验证入口 | 交付物与结束条件 |
|---|---|---|---|
| T-01 / work | M2-A，M1 标签已核 | 建专属 worktree 与材料登记。按 §5 静态清单和旧 module-case 真实基线记七行、源码/预期 SHA-256、后端配置。旧基线异常先定位并保留原结果。 | 在材料目录记录源码提交、工具身份、三个旧 suite 路径、四份旧报告、§5 原始文件摘要及七行预期。基线有红项时记录真实错误和重放结果，再决定可比较的构建。 |
| T-02 / work | T-01 | 接入 §2 两条作者入口和内联产物，完成 lint/list/inspect/run 使用的共同解码，写四例新文档、独立 suite 及显式固定组（拟名 `case-ace-document-pilot-1002`）。执行 V-01 与 `cargo t auto-test-case`。 | 先以 `document_fences.rs`、`discovery_suite.rs` 和合同测试锁住标题/围栏边界、pair 作用域、无尾 LF 和旧格式拒绝集。再实现 §5.3 的执行链，交付 §5.1/5.2 两个文件及固定组。用新 suite 的 lint、list、inspect 和 inventory 确认可发现、可定位、可准备工具。 |
| T-03 / work | T-02 | 按 V-02..04 对跑、收集差异、负控和加载数据，记命令、退出码、行数与样本。固定组与旧消费者按 §6 验证。受影响完整组含 `auto-test-case`、`case-ace-m1-base`、`case-ace-m1-console-863`、`case-ace-m1-immutable-avm-920` 及新增组；实际修改后端执行代码时补相应职责组。 | 给 §6.2 的七行对照编写可重放的校验，按期望集合检测缺项、重复、状态和实际值差异。在冻结构建完成新旧对照、四个独立负控以及 §6 V-04 的读取计量。验证新增固定组从正常入口跑到结束，归集“4 个 case、7 项检查、4 个负控”的逐项结果。 |
| T-04 / review、merge | T-03 | 独立复审和冻结提交定向复验，复用身份一致的证据；核 SD-01、归档并清理。保留可重新执行的试点入口，结论写进计划。 | 复审从旧声明独立推导七行，逐项核对新文档的范围、实际运行观测和负控结果。把已交付标题与 recipe 写入 SD-01，记录首批仍需外置的特殊资产或尚未验证的能力，作为下一批选例依据。 |

**推广条件与顺序。** 五项验收均满足后，以已注册 ACE suite 和实际 case/check 身份建立迁移总表，列出每一项现有验证及其消费者。先推广已验证的单文件值、输出、诊断、Rust 快照，再按新增能力处理多产物（含 C/Python）、资源探针 R03、多步会话等。每批独立新旧对跑，差异清零再切换其正式 suite；确认全部引用已转移后再移除旧副本。发现差异就保留该批旧入口，修复后重比；总表所有项有去向、原有必测项全部可执行时才报告全量迁移完成。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-01/T-02/T-03分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-12 stage:new / PLAN-1002 r6：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原4任务/全部AC与V/依赖/阶段/预算和drafting 0/4；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

r1 提出 hello 与 R03 的直接接入。r2 明确短标题、多例、元数据继承。r3（2026-11-10）用户确认格式并指定先做 3～5 例，本版据此重排 AC-01..05、T-01..04；原先的 C/Python 与 R03 验证转入成功后的相应批次。stage=new，实施计划可评审；§5 的来源和七行计数已静态核对。实施、对跑与独立复审均待执行。

r4（2026-11-10）按用户决定，将试点及其通过后的推广排入 M2。四个 M1 样本沿用旧身份作为对照来源。

r5（2026-11-12）补入四例可复制全文、suite、零元数据 hello 和新旧对跑命令。依据当前主检出 `f9fb3854b` 重核旧声明、源码、Rust 金样、清单与报告的读取入口。细化 T-01..04 和 AC-01..05 的取证方法。样例核验属于静态检查，V-01..04 须在实施树实跑。PLAN-1000 的功能成果已归档，M1 正式收口口径参见 M2 路线图 §15。

r7（2026-11-13）核实 M1 标签、1016/1025 归档与 1017 成本口径，更新 T-01/T-03 和 V-04。四例七检查、AC-01..05 与实施进度保持原记录。此次仅核源码、配置与样本。

## 10. 待澄清事项

试点样本和成功条件已具体列出，未决事项为实际对跑结论。试点可从专属 worktree 的 T-01 起步。
