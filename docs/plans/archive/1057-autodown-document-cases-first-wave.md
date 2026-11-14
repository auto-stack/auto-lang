---
plan_id: PLAN-1057
status: archived
feature_name: AutoDown文档式用例首批推广
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-14
plan_revision: 3
phase: M2-A
current_step: 8
total_steps: 8
depends_on: [PLAN-1002]
supersedes_spec_components: [docs/specs/auto-test-case/document-cases.md, docs/specs/auto-test-case/module-selection.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/legacy-immutable-diagnostic.md]
new_spec_components: []
touched_goals: []
affects: [auto-test-case]
---

# [PLAN-1057] AutoDown文档式用例首批推广

## 0. 变更摘要

承接 [1002 已交付试点](1002-autodown-document-cases.md)，迁移三份实际语料的全部 **30 个 case、60 项检查**。十个标量例共40项，十五个控制台例共15项，五个赋值诊断/合法邻例共5项。三份文档按行为组织，源码、输出和生成文本就地可读。1002四例七项与本批重叠，推广计数按唯一身份算30/60。

完整迁移需补有限作者能力。TOP需要script入口，O07/08/09/15需要AV unit/value通道，O10..14需要错误完成状态及双流。本轮仅创建详细实施合同，所有实际执行与复审记录待work/review填入。

## 1. 目标

- 30例的源码、虚拟路径、入口、tags/links、check ID、adapter/action、recipe/entry/caps/steps和独立预期逐项对齐。
- 固定测试组正常入口消费新语料。BASE组36例/97项中十个标量例40项切到新文档，其余26例/57项由原件供给；控制台和赋值组分别15/15、5/5。三个完整组共56例/117项。
- 每批新旧对跑和负控通过后再切换，失败报告能指出新文档的case/check/流或产物/行号。
- 迁移作者入口后，旧资产按实际消费者登记。仍被其他冻结合同消费的原件列出具体路径、owner和后继关闭条件。

本包范围是上述三份语料与必要接线。B08/B11属于其他差异/认证合同；C/Python、多文件输入、集合资源和多步会话扩面沿各自合同推进。性能正式组采用1017 r6现行口径，本包选受影响常规组与同路径小样本核验。

## 2. 架构方案

复用现有 `author::decode_document → CaseSpec/CheckSpec → resolve_case/resolve_oracle → 原adapter/父比较器`。`entry-mode`、`channel`、`stage/code` 是静态pair，分别降到已有EntryMode、av_channel、Completion。

新三份文档放 `crates/auto-lang/test/cases/ace-m2/documents/`。三份活跃suite放在 **`crates/auto-lang/test/cases/`共同根**，可以用合法相对include同时登记新文档和未迁移旧文件。BASE的 `fixture_root: "ace-m1"` 使旧文件中的src/oracle/helper引用继续解析到原件。该布局沿用现有路径准入。

## 3. 技术栈与工作位置

- Rust AutoDown块树、原文范围和Atom pair，既有CLI、固定Cargo组、Python治理。
- 未来唯一worktree为 `D:/autostack/.wt/lang-1057/auto-lang`，分支 `plan-1057-dev`，按new-wt-group补auto-down兄弟树。
- 实施材料唯一scope为 `D:/autostack/plan-artifacts/plan-1057/`，登记owner/用途/在途状态。old/new/controls分别保存基线、新报告和负控。正常构建在专属树。
- 本轮由 `scripts/new-plan.sh` 分配1057，只修改计划与路线簿记。merge时摘要折入本计划并对账清理全部输出路径。

## 4. 需求分析与背景调查

### 4.1 授权和来源

2026-11-14用户明确要求先规划三类首批推广。取材主线 `10a3dd46e4a4c15948f120e0654fce2182078ccd`，1002交付为 `70a680dba`。2026-11-14 用户通过 `/goal 完成计划1057` 一次性授权全链路执行（worktree `D:/autostack/.wt/lang-1057/auto-lang` @ `plan-1057-dev`，基面 `76431e6b9ccd46339c44db591521fedf73934a0f`，auto-down `58ea33f699efa5cb1194473a2d45dcec51006842`，work → review → 有界修复 → merge）。模块入口见 [全局概览](../../specs/overview.md)及 [文档式当前规范](../../specs/auto-test-case/document-cases.md)。下表为静态核对得到的原合同预期，实施前T-01重新绑定实际源码和工具。

### 4.2 当前具体缺口

- `crates/auto-test-case/src/author.rs::decode_one_case` 最终固定Program，case只接受id/tags/links/module-name。TOP的script入口需要扩展。
- `ProbeSection` 当前仅含recipe/entry/caps/exit/streams，默认success，av_channel未填。完整控制台需要unit/value和typed rejection。复用 `contract::validate_console_oracle` 和原合同准入。
- `host/language_conformance.rs` 有97项BASE及字符串替换旧合同的B01反控；AV的runtime/wire基准直接include B01.at；batch pilot按旧围栏取来源。切换要核实际消费者。
- M1治理脚本、`ace-m1-expanded/*contract*.json`和source-manifest对旧路径/摘要有冻结绑定。新活跃suite用新ID，M1历史来源仍通过旧套件独立验证。身份差异在对照时列明。

### 4.3 旧声明基线

相对根为 `crates/auto-lang/test/cases/ace-m1/`。完整SHA供T-01对账；实施时另导出每份源/Rust/流的完整内容摘要到scope。

| 原文件 | 字节数 | SHA-256 |
|---|---:|---|
| `b-scalar.case.ad` | 10747 | `758863818ad3c5b501fbda65cecbbb6949e424bc9eb27eafa0c52df243f927ee` |
| `console-863/console-863.case.ad` | 9092 | `aa519502ec2f563e6148f94a327aa4cbf8162e4b8ef127c826df934120aa78e9` |
| `immutable-avm-920.case.ad` | 3882 | `96ef60067b3c1dbab1580ffa2cdb5d0433221011ca0d4ca0207b588f787e1e42` |
| `ace-m1-base.suite.ad` | 704 | `7fd00b8d1c79d45be1f4d5ad90083780a8acb67798c6c36b9d29b8f53d26ae63` |
| `console-863/console-863.suite.ad` | 194 | `d88920b15c9a7c04bb6bce793d814c94d3ad2c30d96efad7984660c7f8e11c6b` |
| `immutable-avm-920.suite.ad` | 294 | `be2909add47887fb2a5880daeb29a2fe6c7c7f76088994519781c047f74b1374` |

## 5. 详细设计

### 5.1 标量与Rust金样：10例/40项

原件为 [b-scalar.case.ad](../../../crates/auto-lang/test/cases/ace-m1/b-scalar.case.ad)。以下每个case展开四条同名检查，次序保持：

| 旧check → 新check | 原执行语义 | 新文档小节 | 比较对象 |
|---|---|---|---|
| `ac-native → ac-native` | probe/run，ac/mir-native-scalar-v1，mn.entry=d_main，一步run | `Probe ac-native` | 原exit与实际native退出值 |
| `av → av` | probe/run，ac-av/core-run-v1，av.entry=d_main，一步run | `Probe av` | 原exit与实际AV值 |
| `autovm → autovm` | avm/run，program、main.at | `VM result`，final-newline=false | legacy-display字符串与actual result.txt |
| `a2r-snapshot → a2r-snapshot` | a2r/emit，module_name=m1_bNN | `Rust` / `m1_bNN.rs` | 原oracle全文内联，实际路径集合与逐文件bytes |

表中Rust摘要是原SHA-256前16位，全文和完整hash在T-01核对。B08/B11在该文件没有条目，其其他原义务继续由对应语料表达。

| case ID（新旧一致） | 实际输入和行为 | native exit / AV exit / VM result | Rust逻辑路径、字节、摘要 |
|---|---|---|---|
| `ace.m1.b01` | 乘法优先级 · [src/B01.at](../../../crates/auto-lang/test/cases/ace-m1/src/B01.at) | `14` / `14` / `"14"` | `m1_b01.rs` · 80B · `f1e2f9be67615670` |
| `ace.m1.b02` | 负数除法/余数 · [src/B02.at](../../../crates/auto-lang/test/cases/ace-m1/src/B02.at) | `-21` / `-21` / `"-21"` | `m1_b02.rs` · 91B · `048176f0dd05d54c` |
| `ace.m1.b03` | 关系/逻辑 · [src/B03.at](../../../crates/auto-lang/test/cases/ace-m1/src/B03.at) | `42` / `42` / `"42"` | `m1_b03.rs` · 146B · `25d36a88b067f58e` |
| `ace.m1.b04` | AND短路 · [src/B04.at](../../../crates/auto-lang/test/cases/ace-m1/src/B04.at) | `42` / `42` / `"42"` | `m1_b04.rs` · 125B · `6b6d6654a17e05ea` |
| `ace.m1.b05` | OR短路 · [src/B05.at](../../../crates/auto-lang/test/cases/ace-m1/src/B05.at) | `42` / `42` / `"42"` | `m1_b05.rs` · 124B · `e8bf3f8e3595dd98` |
| `ace.m1.b06` | 零次循环 · [src/B06.at](../../../crates/auto-lang/test/cases/ace-m1/src/B06.at) | `0` / `0` / `"0"` | `m1_b06.rs` · 155B · `1f3a78653386146e` |
| `ace.m1.b07` | 嵌套break/continue · [src/B07.at](../../../crates/auto-lang/test/cases/ace-m1/src/B07.at) | `30` / `30` / `"30"` | `m1_b07.rs` · 404B · `eb48caedee27a940` |
| `ace.m1.b09` | 递归阶乘 · [src/B09.at](../../../crates/auto-lang/test/cases/ace-m1/src/B09.at) | `120` / `120` / `"120"` | `m1_b09.rs` · 174B · `b38838b03b5500e4` |
| `ace.m1.b10` | 互递归 · [src/B10.at](../../../crates/auto-lang/test/cases/ace-m1/src/B10.at) | `42` / `42` / `"42"` | `m1_b10.rs` · 305B · `87a90cbf81b28e66` |
| `ace.m1.b12` | 一元负号 · [src/B12.at](../../../crates/auto-lang/test/cases/ace-m1/src/B12.at) | `-2` / `-2` / `"-2"` | `m1_b12.rs` · 79B · `e4735ac284e53a81` |

文件层tags为ace-m1/four-path/scalar、links为PLAN-852。各case显式沿用id/module-name，Auto节path为main.at。

### 5.2 控制台：15例/15项

原件为 [console-863.case.ad](../../../crates/auto-lang/test/cases/ace-m1/console-863/console-863.case.ad)。新旧check都为av-console；probe/run、ac-av/console-run-v1、d_main、program、main.at、一步run。双流各是独立bytes断言，每例只执行一次。字符串采用JSON转义表示，空串、一个LF与双LF严格区分。

| case ID | 原Auto输入 | completion | payload/channel | stdout原字节 | stderr原字节 | caps |
|---|---|---|---|---|---|---|---|
| `ace.m1.o01` | [src/O01.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O01.at) | 成功 | exit 7 | `"-2147483648\n-7\n0\n42\n"` · 20B | `""` · 0B | `["console.stdout", "console.stderr"]` |
| `ace.m1.o02` | [src/O02.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O02.at) | 成功 | exit 0 | `"true\nfalse\n"` · 11B | `"true\nfalse\n"` · 11B | `["console.stdout", "console.stderr"]` |
| `ace.m1.o03` | [src/O03.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O03.at) | 成功 | exit 0 | `"你好\n\n"` · 8B | `"é\n"` · 3B | `["console.stdout", "console.stderr"]` |
| `ace.m1.o04` | [src/O04.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O04.at) | 成功 | exit 0 | `"界\n"` · 4B | `"😀\n"` · 5B | `["console.stdout", "console.stderr"]` |
| `ace.m1.o05` | [src/O05.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O05.at) | 成功 | exit 4 | `"Auto\n"` · 5B | `"Auto\n"` · 5B | `["console.stdout", "console.stderr"]` |
| `ace.m1.o06` | [src/O06.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O06.at) | 成功 | exit 0 | `"9\n3\n"` · 4B | `"8\n"` · 2B | `["console.stdout", "console.stderr"]` |
| `ace.m1.o07` | [src/O07.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O07.at) | 成功 | 无exit，channel=unit | `"7\n"` · 2B | `"unit\n"` · 5B | `["console.stdout", "console.stderr"]` |
| `ace.m1.o08` | [src/O08.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O08.at) | 成功 | exit 42，channel=value | `"1\n"` · 2B | `"2\n"` · 2B | `["console.stdout", "console.stderr"]` |
| `ace.m1.o09` | [src/O09.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O09.at) | 成功 | 无exit，channel=unit | `""` · 0B | `"done\n"` · 5B | `["console.stdout", "console.stderr"]` |
| `ace.m1.o10` | [src/O10.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O10.at) | 失败 `run` / `av.trap.div-by-zero` | 无exit | `"before\n"` · 7B | `"fail\n"` · 5B | `["console.stdout", "console.stderr"]` |
| `ace.m1.o11` | [src/O11.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O11.at) | 失败 `resolve` / `source.console-argument` | 无exit | `""` · 0B | `""` · 0B | `["console.stdout", "console.stderr"]` |
| `ace.m1.o12` | [src/O12.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O12.at) | 失败 `parse` / `host.unauthorized` | 无exit | `""` · 0B | `""` · 0B | `[]` |
| `ace.m1.o13` | [src/O13.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O13.at) | 失败 `resolve` / `source.console-argument` | 无exit | `""` · 0B | `""` · 0B | `["console.stdout", "console.stderr"]` |
| `ace.m1.o14` | [src/O14.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O14.at) | 失败 `parse` / `source.console-reserved` | 无exit | `""` · 0B | `""` · 0B | `["console.stdout", "console.stderr"]` |
| `ace.m1.o15` | [src/O15.at](../../../crates/auto-lang/test/cases/ace-m1/console-863/src/O15.at) | 成功 | 无exit，channel=unit | `"\n"` · 1B | `"\n"` · 1B | `["console.stdout", "console.stderr"]` |

文件层tags为ace-m1/console、links为PLAN-863，各case保留原第三标签。O08/O09的源是顶层表达式，**原entry.mode是program**，沿原合同迁移。TOP按自己的script合同表达。

### 5.3 类型诊断和合法邻例：5例/5项

原件为 [immutable-avm-920.case.ad](../../../crates/auto-lang/test/cases/ace-m1/immutable-avm-920.case.ad)。新旧check都为autovm，adapter/action=avm/run，虚拟路径main.at。

| case ID | 原输入 | entry.mode | 独立预期 |
|---|---|---|---|
| `ace.m1.ia920.b18` | [src/B18.at](../../../crates/auto-lang/test/cases/ace-m1/src/B18.at) | `program` | 失败 `type` / `auto_name_E0203` |
| `ace.m1.ia920.b18c` | [immutable-avm-920/B18C.at](../../../crates/auto-lang/test/cases/ace-m1/immutable-avm-920/B18C.at) | `program` | 失败 `type` / `auto_name_E0203` |
| `ace.m1.ia920.top` | [immutable-avm-920/TOP.at](../../../crates/auto-lang/test/cases/ace-m1/immutable-avm-920/TOP.at) | `script` | 失败 `type` / `auto_name_E0203` |
| `ace.m1.ia920.nb18` | [src/NB18.at](../../../crates/auto-lang/test/cases/ace-m1/src/NB18.at) | `program` | 成功，legacy-display "2" |
| `ace.m1.ia920.nb18s` | [immutable-avm-920/NB18S.at](../../../crates/auto-lang/test/cases/ace-m1/immutable-avm-920/NB18S.at) | `program` | 成功，legacy-display "6" |

三个负例写VM diagnostic，两合法邻例写VM result并去一个围栏尾LF。NB18S原金样是"6"，保留原完整输入/print行为和该值，迁移不调整旧VM shadowing行为。文件层tags为ace-m1/diagnostic/immutable-avm-920，links为PLAN-920。

### 5.4 有限作者格式增量

| pair及节点 | 类型/默认与降级结果 | 定位拒绝 |
|---|---|---|
| entry-mode / case | program或script，默认program，填CaseSpec.entry。仅TOP显式script。 | 未知值/错类型/重复键；不跨case继承。 |
| channel / console Probe | 可选value或unit，填ExpectSpec.av_channel。O08为value+exit42；O07/O09/O15为unit。 | unit+exit、失败+channel、未知值；缺省不补成value。 |
| stage与code / console Probe | 成对声明Completion::Error；缺省success。O10=run，O11/O13=resolve，O12/O14=parse。双流仍必填。 | 少一个键、空/未知stage、code错类型、失败又要求exit；适用性沿原适配器准入。 |
| caps / console Probe | O12显式空列表原样降级，worker实际给host.unauthorized。 | loader不得自动补授权；缺工具/超时由基础设施状态表达。 |

先比较规范化CaseSpec/CheckSpec和ResolvedCase/ResolvedOracle，再运行。忽略载荷来源范围的表示差异时，仍严格核源字节、逻辑路径、入口、参数、独立期望及实际结果。旧v1/v2接受/拒绝集继续测试。这些pair为T-02拟实施接入。

### 5.5 可复制的代表case

各例物化到文档时从上述文件层继承原tags/links。以下完整源码和Rust正文取原件。

#### 负数除法与Rust

````ad
## Case: signed-div-rem
id: "ace.m1.b02"
module-name: "m1_b02"

### Auto
path: "main.at"

```auto
fn main() int { return (-7 / 3) * 10 + (-7 % 3) }
```

### Probe ac-native
recipe: "ac/mir-native-scalar-v1"
entry: "d_main"
exit: -21

### Probe av
recipe: "ac-av/core-run-v1"
entry: "d_main"
exit: -21

### VM result
check-id: "autovm"
final-newline: false

```text
-21
```

### Rust
check-id: "a2r-snapshot"

#### m1_b02.rs

```rust
// Auto-generated by a2r transpiler

fn main() -> i64 {
    return -7 / 3 * 10 + -7 % 3;
}
```
````

#### unit控制台

````ad
## Case: unit-console
id: "ace.m1.o07"

### Auto
path: "main.at"

```auto
fn main() void {
 print(7)
 eprint("unit")
}
```

### Probe av-console
recipe: "ac-av/console-run-v1"
entry: "d_main"
caps: ["console.stdout", "console.stderr"]
channel: "unit"

#### stdout

```text
7
```

#### stderr

```text
unit
```
````

#### 陷阱之前的输出

````ad
## Case: trap-keeps-output
id: "ace.m1.o10"

### Auto
path: "main.at"

```auto
fn main() int {
 print("before")
 eprint("fail")
 return 1 / 0
}
```

### Probe av-console
recipe: "ac-av/console-run-v1"
entry: "d_main"
caps: ["console.stdout", "console.stderr"]
stage: "run"
code: "av.trap.div-by-zero"

#### stdout

```text
before
```

#### stderr

```text
fail
```
````

#### 顶层脚本的类型诊断

````ad
## Case: script-immutable
id: "ace.m1.ia920.top"
entry-mode: "script"

### Auto
path: "main.at"

```auto
let x int = 1
x = 2
print(x)
```

### VM diagnostic
check-id: "autovm"
stage: "type"
code: "auto_name_E0203"
````

### 5.6 suite与日常入口切换

| 新作者文件（相对test/cases） | case/check | 全部载荷来源 |
|---|---|---|
| `ace-m2/documents/scalar.case.ad` | 10/40 | 十份源+十份原Rust全文 |
| `ace-m2/documents/console.case.ad` | 15/15 | 十五份源+原completion/caps/双流/channel/exit |
| `ace-m2/documents/immutable-assignment.case.ad` | 5/5 | 五份源+三失败/两成功 |

三个suite放共同根 `crates/auto-lang/test/cases/`。BASE新suite仍36例97项，利用合法include和fixture_root复用旧57项。文件 `ace-document-base.suite.ad` 的完整示例：

````ad
```test-suite
suite {
  schema: "autolang.suite/2"
  id: "ace-document-base"
  fixture_root: "ace-m1"
  include: [
    "ace-m2/documents/scalar.case.ad",
    "ace-m1/b-traps.case.ad",
    "ace-m1/b-diags.case.ad",
    "ace-m1/legacy-composite.case.ad",
    "ace-m1/legacy-negative.case.ad",
    "ace-m1/session-lifecycle.case.ad"
  ]
}
```
````

控制台/赋值suite分别名为ace-document-console.suite.ad、ace-document-immutable.suite.ad，各include其全内联文档，ID分别为ace-document-console、ace-document-immutable。

| 现有固定组 | 对照通过后suite（相对test/cases） | 完整预期 |
|---|---|---|
| `case-ace-m1-base` | `ace-document-base.suite.ad` | 36/97 |
| `case-ace-m1-console-863` | `ace-document-console.suite.ad` | 15/15 |
| `case-ace-m1-immutable-avm-920` | `ace-document-immutable.suite.ad` | 5/5 |

保原组名供调用者使用，说明注明日常语料已切新格式。每个切换提交前记录旧suite回放入口。新lint/inventory/identity-inventory、固定组与新旧对照全过才交付。

### 5.7 实际消费者与旧文件去向

| 已核路径/消费者 | 本包处理 | 关闭断言 |
|---|---|---|
| `scripts/fixed-tests.py::run_case`、prepare_tools::case_tools | 消费新清单/原recipe，核身份与完整终态 | 三组完整PASS，来源指向新文档，单case选测只运行所需后端 |
| `experimental/ace/tests/conformance/tests/host/language_conformance.rs` | 将 ace_m1_base_suite_green、ace_m1_base_negative_oracle_fails、ace_m1_golden_drift_is_red 三个具名入口一起切到新suite。退出反控改B01 Probe ac-native的exit14→15；金样反控只在B01 Rust块将“+”改为“*”。 | 正常97P；两个变体各1F+96P，分别点名exit和m1_b01.rs。复制根/fixture_root包含原57项，金样反控实际变动新内联载荷。 |
| `experimental/ace/tests/conformance/tests/console.rs` | corpus()改新控制台suite，原15项/三轴反控/stale检查继续实际执行 | 十五项PASS、原通道/字节反控和15 fresh成立 |
| `experimental/ace/tests/conformance/tests/governance.rs` | `reuse_obligations_run_green_via_original_suites` 明确验证887的原套件来源，保其M1_BASE_SUITE直接引用。为新文档另加具名CLI接线回归并复用当前调用函数；新回归由ace-host覆盖。 | 原来源回归可追旧suite；新增CLI回归点名新suite ID与format=document的十个标量case/40行，核新BASE97行完整。 |
| `crates/auto-lang-core/src/test_cases/ac_av/test_batch/pilot.rs` | 原批处理来源合同继续读M1围栏，后续多步推广再迁；本包登记其原文件引用 | ace-batch原身份/锚/原件可读，本包兼容回归核验 |
| AV的av_runtime_baseline.rs、av_wire_baseline.rs；session-lifecycle.case.ad；其他引用 | T-01用rg逐份列出源/oracle消费、必要保留原因和后继关闭条件 | 被include_str或其他合同使用的原件逐项有归属 |
| `scripts/test-governance/ace_m1_legacy_a2r.py`、ace_m1_legacy_base.py、expanded合同、source-manifest、test_ace_m1_acceptance.py | M1历史/兼容读者继续用旧冻结套件，保其路径/摘要；日常新suite用新的suite身份 | 同构建旧inventory仍匹配自己的合同，infra-ace-m1过；新suite由当次实跑判定 |
| 1002试点语料及固定组 | 保作独立作者格式集成回归 | 4/7仍通过，其覆盖不加到本批30/60 |

T-01消费者表存scope，耐久结果折本计划§9。三份新文档是本批日常编写入口，保留旧件逐个记owner/理由/关闭条件。不存在只增加试点文件而没有切换正常测试入口的完成状态。

### 5.8 对照数据接口与来源核验（复审补齐）

当前 `identity-inventory` 已导出probe绑定、steps、completion、av_channel与产物内容摘要，但没有完整的case entry/tags/links、VM result和双流预期。实现新增只读 `auto-case migration-inventory --suite <path>`，schema为 `autolang.case-migration/1`。在 `test_cases/inventory.rs` 共用load_suite、apply_defaults、resolve_case、resolve_oracle生成规范化投影，CLI仅负责序列化。T-02交付该接口，T-03..05的对照器直接消费；原M1工具继续读取自己的现行inventory schema。

投影分为 `semantic` 和 `origin` 两部分。origin记录suite/fixture根、case_file、原文行号、payload来源及来源摘要。semantic按下表导出实际解码值，字符串保原内容；bytes用长度+SHA-256表示并在对跑时核原件/actual。可选值用null明确表达缺席，空字符串/空列表与缺席分别比较。

| 规范化字段 | 来源及必比规则 |
|---|---|
| case.id/tags/links/entry_mode、source.syntax/virtual_path、files[] | CaseSpec与ResolvedCase；tags/links保合并后顺序，source/files每件含路径/bytes/hash，TOP=script；本批files为空也须显式核对 |
| check.id/adapter/action/tier/skip/product_from，case.status_skip | effective CheckSpec；本批所有skip/product_from为null，行次序可追，空检查及重复身份拒绝 |
| options | 对现有effective Options完整导出，包括module_name、mn/av entry/caps、一步run、profile/cache/budget等现有类型字段；None按null。本批未声明的新选项一旦出现会被对照点名 |
| expect.completion/message、exit、result格式/文本、av_channel | ResolvedOracle；按原值核成功或类型化失败，不从PASS或渲染文本猜。消息断言本批为null |
| expect.stdout/stderr、artifacts[]、其他optional oracle | 流含presence/compare/长度/hash及可读文本；产物含完整逻辑路径集合、compare/长度/hash。其他如perf/trace/typed/probe字段按现有Resolved/Check类型导出，原null也核对 |

`scripts/test-governance/document_case_migration.py` 接收明确的两套suite、runner、预期集合与scope输出根，使用上述投影及既有run/module-case收集报告。三个子命令合同在T-02/03接好：

- `pair --runner <auto-case> --old-suite <path> --new-suite <path> --family scalar|console|immutable --out <fresh-dir>`，先比投影/独立集合，再跑两侧并核actual；scalar含BASE额外57行兼容验证。
- `verify --old-record <record.json> --new-record <record.json> --family <name>`，只核已采记录和实际文件；成功退出0，语义/行集/actual差异退出1，缺证/路径/格式错误退出2。
- `controls --runner <auto-case> --out <fresh-dir>`，从登记的原始文档逐个构造§6负控，各保存命令与原始退出；所有预期反控都被发现才返回0。

schema与字段有错误定位，规范化测试以独立手写的B02、TOP、O07/08/10/12预期字段逐项断言。对照器的固定集合由本计划三原件及§4摘要绑定，且成员总数30/60必须匹配；改动旧原件、缺元数据、增加skip或任何expected与actual证据错绑都产生明确失败。

### 规范增量

| delta_id | 变更 | canonical目标 | 当前 → 交付后 | 理由 | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-test-case/document-cases.md | 四例试点 → 三类30/60及entry-mode/channel/错误双流 | 完整迁移语义 | AC-01..04 |
| SD-02 | modify | docs/specs/auto-test-case/module-selection.md | 无固定组↔作者格式规则 → 固定case组跟随活跃作者格式（文档式活跃suite；旧suite仅冻结历史回放） | 调用者可找当前作者源；r3 改正 r2 的失实"当前"（该文件原本不含旧三套件条目，复审 F-1057-R1-01） | AC-05/06 |
| SD-03 | modify | docs/specs/auto-test-case/document-cases.md | pilot来源说明 → 内容/表示身份对账、migration-inventory投影、actual索引及消费者去向 | 可追溯性 | AC-04..07 |
| SD-04 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 底座/console/immutable 语料入口写成 `ace-m1-base.suite.ad`、`console-863/console-863.suite.ad`、`immutable-avm-920` → 日常入口=`ace-document-base`/`ace-document-console`/`ace-document-immutable`文档式suite（同36例97检查/15/5检查），旧suite保留为冻结历史与M1身份来源 | 交付后规范与已切换的固定组接线一致（r3，复审 F-1057-R1-01） | AC-05/06 |
| SD-05 | modify | docs/specs/auto-test-case/legacy-immutable-diagnostic.md | 语料套件行=旧 `ace-m1-immutable-avm-920` suite → 日常入口=`ace-document-immutable`（同5检查），旧suite保留为冻结历史 | 同上 | AC-05/06 |

`touched_goals` 为空的无影响说明（r3）：本包只改语料作者格式、固定组入口与相应规范条目，不改变 GOAL-016（构建/测试基础设施）或 GOAL-018（开发范式与知识账本）的既有承诺、分母或达成状态；无目标状态需要由 merge 更新。

## 6. 测试设计

### 6.1 新旧完整对照

T-01先在原始构建取得基线，T-06在同一冻结的新构建对跑新旧suite。以独立固定的 `(case,check)` 集合连接两边，先核重复再核缺失。BASE选迁移40行及剩余57行分别核验，console15、immutable5，各有完整终态。

| 必须等价的字段 | 获取与判断 |
|---|---|
| case/check、adapter/action、entry、source/files、recipe/entry/caps/steps、module-name | 逐项比较解码后的Spec与Resolved输入；源文件重建字节/hash和虚拟路径必须匹配，TOP为script，O08/O09仍program |
| completion、exit、result、channel | 从当次worker回复/父裁判及result.txt、failure.json取actual，核成功/错误阶段码/返回payload与原预期；缺实际值由执行边界取证 |
| stdout/stderr | 单独核原始bytes、长度、尾LF、空串；O03是8B/3B，O04的stderr是5B；O10失败仍必须留住两路输出 |
| Rust产物 | 比实际逻辑路径集合、逐文件bytes/hash与旧金样；原expected全文成为内联载荷，actual来自执行器生成 |
| 工具与运行条件 | 同编译器/worker/target/config，同输入、入口和资源条件；输出分别落old/new，worker会话隔离或明确记录缓存状态 |
| 允许变化的表示字段 | suite ID、case_file/source_ref/文档范围、contract/check结构指纹分别解释。inline@行号可能影响oracle表示摘要，须另外核期望内容完全相等；旧文件读取路径仍符合自己的历史合同 |

结果保存“case/check、旧source/期望/actual、新source/期望/actual、差异原因”对照表，每条可追当次report/artifact/hash，具体记录格式和来源见§6.4。新旧共同红需原样记首次结果和复现，迁移范围内实际源/预期错误必须定位解决。对账工具不能从本次动态发现列表生成自己的必测集合。

对照脚本及规范化接口的精确合同见§5.8。治理反例放 `scripts/tests/test_document_case_migration.py`（新文件），测试缺行、重复、actual篡改/缺失、TOP入口丢失、O08通道遗漏和元数据/skip漂移。复用旧检查器时仍需实现等价的§5.8投影、§6.4实际记录与独立固定集合，具体符号写入T-01。

以下PowerShell从专属树执行，由prepare_tools提供同构建parent/AV/native环境。T-01首次只运行旧三套；T-06完整重跑。命令输出由上层保留到相应scope子目录。

```powershell
$runner = (Resolve-Path './target/debug/auto-case.exe').Path
$cases = 'crates/auto-lang/test/cases'
$out = 'D:/autostack/plan-artifacts/plan-1057/pair-01'

& $runner run --suite "$cases/ace-m1/ace-m1-base.suite.ad" --out "$out/old-base"
if ($LASTEXITCODE -ne 0) { throw '旧 BASE 异常，先保留并定位' }
& $runner run --suite "$cases/ace-m1/console-863/console-863.suite.ad" --out "$out/old-console"
if ($LASTEXITCODE -ne 0) { throw '旧控制台异常，先保留并定位' }
& $runner run --suite "$cases/ace-m1/immutable-avm-920.suite.ad" --out "$out/old-immutable"
if ($LASTEXITCODE -ne 0) { throw '旧赋值诊断异常，先保留并定位' }

foreach ($family in 'base','console','immutable') {
    $suite = "$cases/ace-document-$family.suite.ad"
    & $runner lint --suite $suite
    if ($LASTEXITCODE -ne 0) { throw "加载 $family 失败" }
    & $runner migration-inventory --suite $suite
    if ($LASTEXITCODE -ne 0) { throw "规范化清单 $family 失败" }
    & $runner run --suite $suite --out "$out/new-$family"
    if ($LASTEXITCODE -ne 0) { throw "运行 $family 失败" }
}
```

### 6.2 具名正反控

隔离副本每次从原未修改文档派生，互不叠加；每次报告带该变体的来源hash。BASE正常97P、console正常15P、immutable正常5P。预期负控导致父退出1的行必须有mismatch，格式错误父退出2。缺项即便suite自行退出0也由独立集合对账返回失败。

| V-ID | 输入/入口 | 预期关闭断言 | AC及owner |
|---|---|---|---|
| V-01 | 三类全部Spec/Resolved比较；将TOP改program，将O12 caps改双授权 | 正式30/60逐字段一致；两变体分别指出entry-mode与caps漂移，即使碰巧运行同结果也拒 | AC-01/02，work |
| V-02 | B01/02/03/04/05/06/07/09/10/12真跑四路径；读十份实际Rust | 40项均PASS，正负数/短路/循环/递归结果匹配，Rust路径及字节全等 | AC-03，work |
| V-03 | 改B02 VM结果-21为-20；另一变体改B02 Rust一个字节；新BASE实跑 | 各只红autovm或a2r-snapshot，1F+96P，实际产物仍按同份源码生成，报告定位到字节/行 | AC-03/04，work |
| V-04 | O03删末LF、O04将有效UTF-8字符😀替换为😁（末字节80→81）、O06交换双流值，各独立运行 | 每次仅对应av-console失败，指出正确流与首差字节，其余14P；无trim/拼流 | AC-03/04，work |
| V-05 | O07 unit正控；unit+exit0、channel=bogus、缺stderr；O10少stage或code | 有效unit无payload；各无效组合加载时拒绝且有行号，执行前无worker调用 | AC-02/04，work |
| V-06 | O08删channel=value；O10错trap码；O12用同阶段异族码；完整console | 对照器检出O08缺通道；后两变体仅目标行FAIL；空caps的host.unauthorized来自实际worker，timeout/缺工具为ERROR | AC-03/04，work |
| V-07 | 五个赋值例，含TOP、NB18、NB18S；改TOP code，另删NB18S完整case | 三个type错误均正确；合法邻例返回2/6；错码只红TOP、其余4P；删例由固定5项清单指出缺项 | AC-03/04，work |
| V-08 | 删B07一个Probe或重复check；省略console文件；更改路径到越界位置 | 加载/对账各在正确层拒绝，点名缺行/重复行；文件丢失及路径错误不充空绿报告 | AC-01/04，work |
| V-09 | 新suite经三个固定组；日常宿主B01 exit14→15及内联Rust的+→*两反控；console原stale反控 | 实际97/15/5项齐，两个宿主反控各1F+96P并点名exit或m1_b01.rs；console15项fresh及原敏感性成立 | AC-05，work |
| V-10 | inventory/run分别测读取，旧M1清单/pin兼容，1002和batch回归 | 每文档每次加载1读，新migrated payload外部读取0；M1原来源仍匹配自己的合同，4/7试点另计 | AC-05/06，work |
| V-11 | 对B02 a2r-snapshot实际报告行做独立范围stale验证，之后修改该内联Rust一个有效字节；另移除一条真实actual证据 | 原行Fresh，变体该行因oracle内容轴Stale；缺证使verify退出2。原BASE native已知stale家族噪声逐项记账，不作为本项通过依据 | AC-04/06，work |
| V-13 | 删除scalar文件层links；对照器单测将O08已导出投影的skip改为非null；交换不同case的receipt；篡改或截断actual文件 | 元数据/skip漂移逐字段拒；源文档直接填未知skip键应在加载期拒绝。错绑/损坏/缺证不能PASS，完整合法record为0退出正控 | AC-01/04/06，work |
| V-12 | 独立从三份旧声明推导30/60，逐原件追新文档/actual/负控/资产去向 | 全部AC和Spec逐项核实，给revision/commit绑定判决及具名缺口 | AC-07，review |

### 6.3 验证组及性能边界

实施先定向跑 `crates/auto-test-case/tests/author_document.rs` 和必要合同反控。最终冻结提交验证完整组：
`cargo t auto-test-case case-ace-m1-base case-ace-m1-console-863 case-ace-m1-immutable-avm-920 case-ace-document-pilot-1002 infra-fixed infra-ace-m1`。
新治理反控用 `python scripts/tests/test_document_case_migration.py`，T-06须实现该入口。

共享宿主接线选完整 `ace-host`、`ace-console`、`ace-batch`，公共author/contract改变由review核实际下游，必要时补对应 `ace-adapters` 职责。1002记录的ace-host原红在T-01按同构建重核具体case，T-06记录新旧差异，范围内新引入红全部修复。证据身份相同才复用，避免修复/提交/review重复整套。

`cargo t ace-perf` 按既有显式排程；本批加载改动用常规单样本IPC及读次数核对，具名退化才扩大定向验证。记录用例数、执行数、原条件排除、准备/冷编译、发现、运行、清理与总时长。周期全仓回归按due规则交merge。

### 6.4 隔离副本与实际证据记录

所有pair/controls输出使用scope下唯一新目录。目录非空即拒绝，避免相同路径混入旧run。新BASE负控副本以 `<scope>/controls/<V-ID>/inputs/cases/` 作共同根，复制三份活跃suite/三份新文档，以及include涉及的旧五份合同和它们真实引用的source/files/oracle/helper闭包；闭包清单及复制前后字节/hash记录在索引。副本fixture_root仍为ace-m1。输入根和输出根 `<scope>/controls/<V-ID>/output/` 互不包含；所有路径遵守现有audit/reparse校验。

反控工具先按case ID和小节定位，再断言变体只改预定字段/载荷；写后重新lint/导投影。B01退出与金样两反控分别只改新scalar文档的相应段，不能匹配到同例另一条exit或旧sidecar。Unicode反控保持合法UTF-8，以便进入worker结果比较。故意坏UTF-8可作格式错误附测，不能替代V-04的流比较。

source身份有两种实际算法，应在record分别记字段。`ResolvedCase.source_sha256` 是按虚拟路径排序后拼接“路径+NUL+各文件SHA+NUL”的组合摘要；`probe-receipt.step.source_hash` 和native judged session的source_sha256是实际提交的源码字节摘要。VM failure的source_sha256核组合摘要、entry_sha256核入口原始字节摘要，避免用同名字段错误对比。

每侧record.json采用 `autolang.case-migration-record/1`，包含frozen commit、源码树状态、工具sha/config、原suite/doc摘要、规范化投影、精确命令/返回码、report路径/hash，以及每个(case,check)对应的唯一run目录和actual来源清单（路径/长度/hash）。从全新out下唯一的run子目录关联到case/check；出现零个/多个候选即缺证错误。本批ID只含安全ASCII，沿当前 `mod.rs` 的case_dir/check_dir布局定位，T-02锁住该规则的测试。

| 实际观察 | 现有来源与校验 |
|---|---|
| VM成功result与失败stage/code | 报告ArtifactRef指向的result.txt/failure.json，读后核长度/hash；失败文件内schema、source_sha256、entry_sha256与本次ResolvedCase对应 |
| AV标量/console成功或失败 | 本次run下 `<case>/<check>/probe-receipt.json`，读recipe/protocol/request_hash、step.source_hash、step.outcome/value/channel/trap/diag、console双流base64，按ac_av.rs现有map_diag_stage语义核诊断阶段。stdout/stderr base64严格解码后比bytes，unit与value和错误各守原结构 |
| native标量exit | report.native.exit_code，并读 `mn-probe-receipt.json` 的exit_code及judged session的source_sha256，核recipe和worker工具轴；两条观测相互对应 |
| Rust真实生成物 | report.artifacts的逻辑路径、stored路径、bytes/hash与实际文件逐项一致，路径属于本次输出；与独立的内联/旧金样比较 |

AV/native的receipt目前有部分未列在报告artifacts中。对照器的run记录在进程结束后直接收录这些真实文件及hash，记录建立前核本次run目录、source/worker身份；缺文件、来源不匹配、坏hash、重复case/check与报告多/少行都使verify拒绝。verify把独立固定预期、规范化声明和真实actual三方核对，不用仅有PASS状态替代观察。record自身是本次对照的复验索引，必须与被引用文件一同可读，原验收摘要仅在merge折叠。

## 7. 验收标准

- [x] AC-01（work）独立清单完整30case/60check、原字段/源/金样/双流/元数据，§5.8规范化接口及V-01/08/13检出缺失、重复和字段漂移。证据=§9 work 收据（r3 @4cbb3f610）：`migration-inventory` 40 文档行 `origin.input_paths=[]`（复审独立重建，117 行 0 语义差异），V-01/08 与治理 5 例通过。
- [x] AC-02（work）entry-mode、channel、stage/code按原类型接通，TOP script不会被默认program覆盖，V-01/05准入与定位正确。证据=§9 work 收据：V-01（TOP→program、O12 caps）与 V-05（unit+exit、bogus channel、缺 stage/code、缺 stderr）均加载期 exit 2 并带行号；`author_document` 12 例全绿。
- [x] AC-03（work）同一冻结构建新旧60行全部PASS且actual逐项等价，BASE额外57行完整，三个组总计117行，V-02/04/06/07。证据=§9 r3 work 收据：4cbb3f610 上 pair 三次 exit 0（scalar 97/40、console 15/15、immutable 5/5）。
- [x] AC-04（work）V-03..08/11/13各隔离负控精确失败，合法邻例通过，能指出全绿子集的缺项与缺证。证据=§9 work 收据：`controls-resync` 36/36 ok，V-03/04/06/07 逐变体只红目标行并指出字节/行。
- [x] AC-05（work）三个固定组实际切换新文档，CLI/宿主/console/stale/试点/batch真实入口核验，三项具名BASE宿主测试及V-09/10通过。证据=§9 work 收据：固定组并集 127/97/15/5/7/68/24 全 PASS，ace-batch 12 PASS，八个具名宿主/console 入口 ok。
- [x] AC-06（work）M1及仍依赖原文件的消费者逐件登记，migrated源/expected外部读取0；内容/表示身份差异、原红/重放及成本分列，§6.4真实record与V-10/11/13可复验。证据=§9 work 收据：consumers 12 条入 `t01-baseline.json`，ACE 6 红与主线既有记录逐项对齐、无新增红。
- [x] AC-07（review）独立核30/60、反控、Spec和旧资产去向，遗漏/延后/workaround均有处置，V-12判决绑定本revision及提交。证据=§9 复审收据：r3 @4cbb3f610 pass，F-1057-R1-01/02 均独立复核关闭，SD-01..05 目标与"当前"陈述逐项验真；merge 三项 closeout gate 已具名。
- [x] AC-08（merge）主线切换、规范/路线沉淀、归档、guard及scope全路径清理真实完成，关键摘要入本计划。

## 8. 执行步骤

| task / owner_stage | 依赖 | 具体改面与交付 | 验收及验证 |
|---|---|---|---|
| T-01 / work ✅ | 计划确认、1002交付可读 | 在唯一树重核源hash/实际字段/消费者，采旧三组实际baseline；冻结§5.8规范化字段、独立固定30/60集合和§6.4闭包/record索引；登记原件去向/工具身份 | AC-01/06；97+15+5基线，首次红/复现、消费者名单、材料索引；冻结后续可执行入口 |
| T-02 / work ✅ | T-01 | author.rs及author_document/合同测试接entry-mode/channel/stage-code，使用现有EntryMode/Completion/validate_console_oracle；在inventory.rs及auto-case.rs新增只读migration-inventory投影，独立字段单测；保source范围与报错行 | AC-02，V-01/05，cargo t auto-test-case及1002试点 |
| T-03 / work ✅ | T-02 | scalar文档内联10份源码与10份原Rust，生成新BASE混合suite；对照脚本按§5.8接pair/verify与固定40项，按§6.4取actual | AC-03/04，V-02/03，40项对应及BASE97项；57项原件仍可解码 |
| T-04 / work ✅ | T-02 | console文档完整15例及新suite，逐项填原completion/caps/channel/双流/exit | AC-03/04，V-04/05/06，真实15项和每个独立变体终态 |
| T-05 / work ✅ | T-02 | immutable文档5例及新suite，TOP显式script，两个合法邻例按原值；完成对照脚本5项字段 | AC-03/04，V-07，三错两对；删例、错码可检 |
| T-06 / work ✅ | T-03/04/05 | 同构建对跑及负控完成后改test-groups三入口；改host/console/governance日常消费者；完成controls及V-13治理反控，保存输入闭包和run记录；迁移三项具名BASE宿主测试；记录M1/batch/include_str原件去向 | AC-01..06，V-08..11/13；§6.3完整并集、CLI、新旧30/60与三组117项、读取计量 |
| T-07 / review ✅ | T-06、冻结提交 | 同树独立复审逐旧case/actual/反控，扫漏项/延期/替代，核SD-01..03、fmt/warnings/调试输出；失败交原owner修复 | AC-07，V-12，revision-bound判决与必要复验并集 |
| T-08 / merge ✅ | T-07 pass | 冻结并集核门后合主线，沉淀规范/路线并归档，折摘要、核所有输出路径、guard后清理树/分支/scope；登记原件保留项 | AC-08，主线来源/规范hash/归档与清理收据，具名持有按规则登记 |

T-03..05在同一权威worktree内由实现owner顺序推进。全部任务待执行。本计划明确有限接入字段与集合，执行可按当前代码等价实现；改变原30/60义务或公共准入应修订合同并取得相应范围授权。

## 9. 复审记录

### 2026-11-14 规划复审与r2修订

review_scope=planning，审阅基线为r1正文及主线 `10a3dd46e4a4c15948f120e0654fce2182078ccd`。此次由同一会话从原件重建清单，未使用独立执行上下文；下面的规划判决以重新读取的旧合同/代码为据。计划状态保持drafting、0/8，全部work/review/merge验收项仍待实施。

| finding | 原缺口及证据 | 修订与归属 | 规划关闭条件 |
|---|---|---|---|
| F-1057-01 / P1 | r1只明确迁移BASE正常执行和exit反控。`host/language_conformance.rs::ace_m1_golden_drift_is_red` 还会修改旧sidecar再执行旧suite，迁移后可能继续测旧路径。 | §5.7、V-09、AC-05、T-06列出三个具名BASE宿主测试，内联Rust反控明确B01的+→*。 | 新BASE正常97P，两个独立变体各1F+96P；Rust变体必须红a2r-snapshot并指向新文档。 |
| F-1057-02 / P1 | r1要求比较Spec字段，但当前inventory/identity-inventory/inspect不能完整提供entry-mode、tags/links及全部result/流预期，Python对照入口尚无可实现的数据合同。 | §5.8定义只读migration-inventory、完整semantic投影和origin；T-02接CLI/共享resolver，T-03接pair/verify，V-01/13有独立字段反例。 | 逐字段比较60项，缺入口/通道/metadata/skip等会拒；原57项兼容逐行核，原CLI schema仍供既有M1消费者。 |
| F-1057-03 / P1 | r1“缺实际值由执行边界取证”没有明确receipt路径/字段和来源算法。AV/native部分receipt不列在report.artifacts中，单看PASS不足以实现三方对照。 | §6.4记录actual路径、长度/hash、run对应、worker/source身份及两种源码摘要；T-06建记录，V-11/13测缺失/错绑。 | 每条actual可追本次run；不同case回执/坏hash/缺证被verify拒绝；VM失败/AV诊断/native退出/Rust字节来源分别可核。 |
| F-1057-04 / P2 | r1没有给BASE反控副本的旧57项文件闭包；“emoji改一字节”可能生成坏UTF-8，导致加载错误，测不到输出比较。 | §6.4明确副本共同根、fixture_root与真实文件闭包；V-04固定😀→😁的有效UTF-8替换，反控前后投影查命中位置。 | 隔离输入能正常加载完整97项，流反控只红1/15；不把格式错误算作流敏感性通过。 |
| F-1057-05 / P2 | governance.rs::reuse_obligations_run_green_via_original_suites本来验证旧交付来源，r1“日常CLI切换”未明确该路径去向；V-11也未限定BASE既知native stale家族噪声。 | §5.7保具名旧来源检查，新增新文档CLI检查；V-11限定B02 a2r的Fresh→oracle Stale，不借其他行噪声关闭。 | 新旧两类消费者都可定位到各自suite，内联期望改动在目标行确实敏感。 |

r1规划结论经本次核验为needs_fix。上述五项已纳入r2的任务、验证和AC；修订后规划结论为pass，表示已给出完整可评审的实施合同。原件30/60无漏列，额外57项和三个组共117项对账成立。以下r1预检记录作为历史保留。



2026-11-14 · stage=new · PLAN-1057 r1 · outcome=pass（规划静态预检）· next=work（本轮授权止于规划，具体合同供确认）。取原声明静态列出10/40+15/15+5/5，每项有源路径、原身份、参数及预期；四段范例使用完整实际源码和B02 Rust金样。实际对跑、负控和独立实现复审待执行。没有启动实施树/生成测试语料/编译或运行产品。

### 2026-11-14 work 收据（T-01..T-06，r2）

2026-11-14 stage: work | plan_id: PLAN-1057 | plan_revision: r2 | outcome: pass | worktree: D:/autostack/.wt/lang-1057/auto-lang | branch: plan-1057-dev | base_commit: 76431e6b9 | code_commit: aeab8dfe4 → 同步 master 后 c5383100e | dependency_revisions: 组内 auto-down 58ea33f6（只读，未改） | task_ids: T-01..T-06 | blockers: none | next_responsible_role: review

实施（aeab8dfe4，18 文件 +3768/-107）：`author.rs` 新增 entry-mode/channel/stage+code 解码与带行号拒绝；`inventory.rs` + `auto-case.rs` 交付只读 `migration-inventory`（schema `autolang.case-migration/1`，semantic+origin 投影）；三份文档 10/40、15/15、5/5 与三条活跃 suite（BASE 用 `fixture_root: "ace-m1"` 混合 include 余 57 项）；`document_case_migration.py`（pair/verify/controls，1375 行）与治理反控 5 例；`test-groups.toml` 三个固定组入口及 host/console/governance 日常消费者切换。附带 `experimental/ace/crates/core/src/source/fstr.rs` UTF-8 修复（旧实现按字节当字符：O04 的 `'界'`/`'😀'` 使 AV worker 侧得到非法源码，T-01 基线记 `case-ace-m1-console-863` O04=ERROR；修复后旧 suite 亦 15/15），新增单测 `fstr_preserves_utf8_chars_and_char_literals`。

同步说明：实施期间 master 前进 14 个提交（967 F0/文档），分支以 `git merge master` 干净合入（c5383100e，无冲突）。下列门禁全部在 **c5383100e 冻结树**重跑；aeab8dfe4 首轮结果（`final-fixed/`、`final-ace/`、`controls-final/`、`pair-01/`）保留为对照。

| 门 | 命令 | 结果 | 证据（scope 内相对路径 + SHA-256） |
|---|---|---|---|
| 固定受影响组并集 | `python scripts/fixed-tests.py auto-test-case case-ace-m1-base case-ace-m1-console-863 case-ace-m1-immutable-avm-920 case-ace-document-pilot-1002 infra-fixed infra-ace-m1 --out …/fixed-resync` | gate_passed=true，429.2s；执行数 127/97/15/5/7/68/24 全 PASS | `fixed-resync/results.json` d46e579a…（source_state 1619c645…） |
| 新旧对照 pair | `document_case_migration.py pair --runner target/debug/auto-case.exe --old-suite … --new-suite … --family …` | 三次 exit 0：scalar 97 对（40 迁移）/console 15/15/immutable 5/5 | `pair-resync/{scalar,console,immutable}/new-record.json` 52b0d4c2…/9780da14…/6538ba57… |
| 具名负控 | `document_case_migration.py controls --runner … --out …/controls-resync` | 36/36 `ok`，exit 0（V-01/03/04/05/06/07/08/11） | `controls-resync/controls-summary.json` 9d47e423… |
| 治理反控 | `python scripts/tests/test_document_case_migration.py` | Ran 5 tests OK | 同树 c5383100e |
| 宿主/控制台/批处理 | `python scripts/fixed-tests.py ace-host ace-console ace-batch --out …/ace-resync` | ace-batch 12 PASS；ace-host+console 164 例 158 PASS / 6 具名既有主线红 | `ace-resync/results.json` 89a19a7b…（957.6s） |

ACE 6 红逐项与主线既有记录对齐（**非本包引入**；同名 6 项在 master@fbfc7a3e7 复跑即为既有红，见 archive/1017 §9、archive/1039 收据、KNOWN-DEBT D-1001-R2/D-932-2，且本包未改这些测试或其后端）：`language_conformance::ace_m1_closures_source_suite_green`、`language_conformance::ace_m1_tr_native_suite_green`、`semantics_protocol::abi_recipe_binding_exclusivity`（unknown ABI leg "z"）、`rust_lowering::a2r_hir_first_case_green`（a2x_runtime.rs/manifest.json 金样字节差）、`rust_lowering::a2r_hir_worker_credential_pin`、`source::av_source_inventory_denominators_replayable`（AP-16 指向已迁 auto-ui 仓的 `examples/ui/015-notes/src/front/app.at`）。六项全在 host 模块，`console::`/`console_p947::` 全绿；无新增红。

AC 证据锚（c5383100e）：
- AC-01/02：40 文档行 `origin.input_paths=[]`；V-01（TOP→program、O12 caps）、V-05（unit+exit、bogus channel、缺 stage/code、缺 stderr）均加载期 exit 2 并给行号；V-08 缺文件/越界 include、V-13 治理 5 例；`author_document` 12 例在 127 内全绿。
- AC-03/04：pair 117 项等价；V-03（VM/Rust 各 1F+96P）、V-04（O03 LF / O04 emoji / O06 双流交换各 1F+14P）、V-06（O08 通道、O10/O12 错码点名目标行）、V-07（TOP 错码 1F+4P、删例）。
- AC-05：三组入口 `case-ace-m1-*` → `ace-document-*`；`document_base_cli_list_and_migration_inventory_complete`、`reuse_obligations_run_green_via_original_suites`、`first_wave_suite_runs_green_with_d_values`、`ace_m1_base_suite_green`、`ace_m1_base_negative_oracle_fails`、`ace_m1_golden_drift_is_red`、`console_real_corpus_and_stale_identity`、`console_three_oracle_axes_fail_independently_and_missing_axes_error` 均 ok。
- AC-06：`t01-baseline.json` 记 consumers 12 条（switch_daily 4 / preserve_legacy(_frozen) 6 / preserve_pilot 1 / preserve_in_hybrid_base 1）与 45 份旧件字节/hash；infra-ace-m1 24 PASS、pilot 7 PASS、ace-batch 12 PASS；migrated 载荷外部读取 0。

材料：原始证据全部在 `D:/autostack/plan-artifacts/plan-1057/`（old/new/pair-01/pair-resync/controls-final/controls-resync/fixed-resync/ace-resync/t01-*.json），merge 收尾折摘要并整目录删除。

2026-11-14 stage: goal | plan_id: PLAN-1057 | plan_revision: r2 | round: 1 | from: executing | to: execution_done | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1057/auto-lang | commit: c5383100e | next: review

### 2026-11-14 review 收据（final，r2）→ needs_fix

2026-11-14 stage: review | review_scope: final | reviewer_identity: 同一 root agent 会话（无独立 reviewer 模型/上下文，按限制声明）；判决由重建证据得出，未采信 work 收据的自述 | plan_id: PLAN-1057 | plan_revision: r2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1057/auto-lang | branch: plan-1057-dev | reviewed_commit: c5383100e（tree clean，diff base 76431e6b9） | dependency_revisions: 组内 auto-down 58ea33f6（只读） | spec_inputs: document-cases.md c2d174d1…、module-selection.md e6c54461…、ace-m1-conformance.md 66849b09…、legacy-immutable-diagnostic.md dc1d1195… | pending_closeout_gates: 受影响并集在同步后的交付基面复跑（见 F-1057-R1-02） | findings: F-1057-R1-01、F-1057-R1-02 | next: work（r3 有界修订 + 基面同步与复跑）

验收映射（独立方法 → 判定）：
- AC-01 pass：自行用 CLI `migration-inventory` 导出六份投影（`review-inv/`），按 Plan §5 表硬编码 30 例/60 项集合比对（未用计划方 pair 工具结论）：scalar 97、console 15、immutable 5 行 `semantic` 深浅比较 **0 差异**；文档行 40/15/5，`origin.input_paths` 全空；标量四检查次序与 §5.1 一致。
- AC-02 pass：TOP `entry_mode=script`；O07/O09/O15 `channel=unit` 且无 exit；O08 `value`+exit42；O10..O14 `completion=error` 且 stage/code 齐全、无 exit；O12 `caps=[]` 未被补授权。`author_document` 12 例、`located_rejection_battery` 全绿（127 例组内）。
- AC-03 pass：该提交上三组实跑 97/15/5 全 PASS（`fixed-resync/`）；pair 重跑 117 项等价（scalar 97/40、console 15/15、immutable 5/5，exit 0）；新 suite `lint`/`inventory`/`identity-inventory` 独立复跑 exit 0（36/15/5 例、97 行）。
- AC-04 pass：`controls-resync` 36/36 ok；V-03/V-04/V-06/V-07 每变体只红目标行并给字节/行；V-11 oracle 内容轴 Fresh→Stale；V-13 治理 5 例覆盖缺行/重复/TOP 入口/O08 通道/links 移除/skip 漂移/换回执/篡改 actual（exit 2）。
- AC-05 pass：三组入口已切；`document_base_cli_list_and_migration_inventory_complete`、`reuse_obligations_run_green_via_original_suites`、`first_wave_suite_runs_green_with_d_values`、三个 BASE 宿主测试、console stale 与三轴反控均 ok；ace-batch 12 PASS。
- AC-06 pass：consumers 12 条登记于 `t01-baseline.json`；旧件未被改动（`git diff 76431e6b9..HEAD -- crates/auto-lang/test/cases/ace-m1` 为空）；ACE 6 红与主线既有记录逐名一致（`language_conformance::ace_m1_closures_source_suite_green`、`ace_m1_tr_native_suite_green`、`semantics_protocol::abi_recipe_binding_exclusivity`、`rust_lowering::a2r_hir_first_case_green`、`a2r_hir_worker_credential_pin`、`source::av_source_inventory_denominators_replayable`；见 archive/1017 §9、archive/1039、KNOWN-DEBT D-1001-R2/D-932-2），非新增红。
- 健康检查 pass：改动文件 rustfmt 干净、无 `dbg!`/`todo!`/调试输出、无 specs.json 写入、无告警指向改动文件；新增两单测（`migration_inventory_projects_exact_legacy_scalar_console_and_immutable_rows`、`fstr_preserves_utf8_chars_and_char_literals`）独立复跑 ok。
- 范围观察（非 finding）：`experimental/ace/crates/core/src/source/fstr.rs` 的 UTF-8 修复不在 §5 列表，但为 O03/O04 非 ASCII 源码在 AV worker 侧可用的前提（T-01 基线 O04=ERROR），ASCII 路径不变且有单测与 ACE 组回归；按"必要接线"接受，merge 摘要须保留该范围说明。

**F-1057-R1-01（P2，spec delta 目标与基线失实；AC-07 未过）**
- 事实：SD-02 称 `docs/specs/auto-test-case/module-selection.md` 现记录"旧三套件日常入口"，该文件全文无 `case-ace-m1-*` 或三个 suite 名（rg 0 命中）。真正写底座/控制台/赋值语料的是 `ace-m1-conformance.md`（"## 底座套件（ace-m1-base.suite.ad）"、"### PLAN-863: O01–O15 console source rows"）与 `legacy-immutable-diagnostic.md`（"语料套件：`ace-m1-immutable-avm-920`"）。两者不在 SD 列表，交付后会继续把旧 suite 写成日常语料入口，与已切换的固定组接线矛盾；frontmatter `supersedes_spec_components` 也漏 `module-selection.md` 与这两个文件。
- 修法（有界，不改行为/验收）：SD-02 重述为通用规则（固定 case 组跟随活跃作者格式；旧 suite 仅作冻结历史回放）或删除；新增 SD-04（改 `ace-m1-conformance.md`：BASE/console 活跃入口=文档式 suite，旧 suite 标注冻结历史与 M1 身份）与 SD-05（改 `legacy-immutable-diagnostic.md` 的语料套件行）；`supersedes_spec_components` 列全；`touched_goals: []` 补书面无影响理由（语料作者格式切换不改变 GOAL-016/018 的达成面）。
- 复审复跑范围：只重核 SD 表、frontmatter 与上述文件的当前→交付后陈述，不重跑测试门。

**F-1057-R1-02（P2，审查基面漂移使受影响证据失效）**
- 事实：审查期间 master 自 `cd5f2e568` 前进 9 个提交，含 967 T-10 两片代码（`experimental/ace/crates/core/src/source/{parser,resolve,ast,cst/project}.rs`、abi cside/ctool、`crates/auto-bindgen/src/c_import.rs`、ace conformance 用例）。本包受影响面恰含 AV/console 用例组与 `ace-host`，其 worker/parser 输入已变，故 `fixed-resync/`、`ace-resync/`、`pair-resync/`、`controls-resync/` 只能证明 `c5383100e`，不能作为交付基面证据；与分支无文件重叠（合并应无冲突）。
- 修法：work 阶段再次 `git merge master`，重建工具后在合并提交上重跑 §6.3 并集（fixed 七组 + pair + controls + ace-host/console/batch），收据绑定新提交；review 再核该提交（实现未变时只复用新一次并集，不叠跑）。

其他观察（merge 归账，不阻断）：workspace 既有 fmt 漂移在 `crates/auto-lang-core/src/test_cases/abi.rs`、`crates/a2r-actor-tests/tests/actor_parity.rs`（非本包改动）；旧 suite 回放入口=§5.7 consumers 表与 `t01-baseline.json`，merge 折叠时保留。

2026-11-14 stage: goal | plan_id: PLAN-1057 | plan_revision: r2 | round: 2 | from: execution_done | to: executing | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1057/auto-lang | commit: c5383100e | next: work

### 2026-11-14 work 收据 r3（F-1057-R1-01/02 修复）

2026-11-14 stage: work | plan_id: PLAN-1057 | plan_revision: r3 | outcome: pass | worktree: D:/autostack/.wt/lang-1057/auto-lang | branch: plan-1057-dev | base_commit: 918426a24 | code_commit: 4cbb3f610（aeab8dfe4 实现 + master 二次同步） | dependency_revisions: 组内 auto-down 58ea33f6（只读） | task_ids: T-06（重开并复闭） | blockers: none | next_responsible_role: review

修复映射：
- F-1057-R1-01（spec delta）：SD-02 改为"无固定组↔作者格式规则 → 固定 case 组跟随活跃作者格式（文档式活跃 suite；旧 suite 仅冻结历史回放）"；新增 SD-04（`ace-m1-conformance.md`：底座/console/immutable 语料入口切文档式 suite，旧 suite 保留冻结历史与 M1 身份）与 SD-05（`legacy-immutable-diagnostic.md` 的语料套件行）；frontmatter `supersedes_spec_components` 列全四个目标；补 `touched_goals: []` 的无影响说明。语义契约、验收与任务范围未变，只改正目标清单与失实"当前"。
- F-1057-R1-02（基面漂移）：`git merge master`（918426a24）干净合入得 4cbb3f610，重建后在该合并提交上重跑受影响并集。

| 门 | 命令 | 结果 | 证据（scope 内路径 + SHA-256） |
|---|---|---|---|
| 固定七组 | `python scripts/fixed-tests.py auto-test-case case-ace-m1-base case-ace-m1-console-863 case-ace-m1-immutable-avm-920 case-ace-document-pilot-1002 infra-fixed infra-ace-m1 --out …/fixed-r3` | gate_passed=true，382.3s；127/97/15/5/7/68/24 全 PASS | `fixed-r3/results.json` 0480e770…（source_state d24edfb1…） |
| 新旧对照 | `document_case_migration.py pair`（scalar/console/immutable） | 三次 exit 0：scalar 97 对/40 迁移、console 15/15、immutable 5/5 | `pair-r3/{scalar,console,immutable}/new-record.json` 9e3467cc…/c466f02d…/5c51fc6d… |
| 具名负控 | `document_case_migration.py controls --out …/controls-r3` | 36/36 ok，exit 0 | `controls-r3/controls-summary.json` 2fb69729… |
| ACE 三组 | `python scripts/fixed-tests.py ace-host ace-console ace-batch --out …/ace-r3` | ace-batch 12 PASS；ace-host+console 164 例 158 PASS / 同样 6 个既有主线红（405.8s；`console::` 7 例全绿） | `ace-r3/results.json` |
| 新增单测 | `cargo test -p auto-lang-core --lib migration_inventory_projects`；`cargo test -p ace-core --lib fstr_preserves_utf8` | 各 1 passed | 同树 4cbb3f610 |

边界：6 红与 r2 及主线既有记录逐名一致（archive/1017、archive/1039、KNOWN-DEBT D-1001-R2/D-932-2），本包未改这些测试或其面，无新增红。master 在本次重跑后仍前进（967 T-11/T-12、1021 归档），其改动面含 `experimental/ace/**` 与 `auto-bindgen`；本收据只绑定 4cbb3f610，最终交付基面的并集复跑归 merge。

2026-11-14 stage: goal | plan_id: PLAN-1057 | plan_revision: r3 | round: 3 | from: executing | to: execution_done | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1057/auto-lang | commit: 4cbb3f610 | next: review

### 2026-11-14 复审收据（final re-review，r3）→ pass

2026-11-14 stage: review | review_scope: final（repair re-review） | reviewer_identity: 同一 root agent 会话（无独立 reviewer 模型/上下文，限制同 r2 声明）；按上一判决、修复 diff 与原始证据复建，不采信 work 自述 | plan_id: PLAN-1057 | plan_revision: r3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1057/auto-lang | branch: plan-1057-dev | reviewed_commit: 4cbb3f610（tree clean；diff base 76431e6b9，上次复审 c5383100e） | dependency_revisions: 组内 auto-down 58ea33f6（只读） | spec_inputs: document-cases.md c2d174d1…、module-selection.md e6c54461…、ace-m1-conformance.md 66849b09…、legacy-immutable-diagnostic.md dc1d1195…（四份与 r2 逐字节相同，r3 只改计划侧 delta） | pending_closeout_gates: 见下 | findings: F-1057-R1-01 closed、F-1057-R1-02 closed（reviewed commit 范围） | next: merge

findings 关闭表（finding → 复验范围 → 结果）：
- **F-1057-R1-01（spec delta）closed**：①五个 SD 行的 canonical 目标全部存在，`supersedes_spec_components` 与 SD 目标逐一相符（document-cases / module-selection / ace-m1-conformance / legacy-immutable-diagnostic）；②各"当前"陈述独立复核为真：document-cases.md 仍是 pilot 4/7（line 3/54）、module-selection.md 全文无 case 组或作者格式规则（SD-02 为"从无到有"）、ace-m1-conformance.md 仍以 `ace-m1-base.suite.ad`（line 65）/`console-863/console-863.suite.ad`（line 203）/`immutable-avm-920`（line 371）为语料入口、legacy-immutable-diagnostic.md line 41 仍写旧 suite；③`touched_goals: []` 附有书面无影响说明。语义契约、AC 与任务范围未变（r2→r3 仅改正目标清单与失实基线）。
- **F-1057-R1-02（基面漂移）closed（限 4cbb3f610）**：①实现身份稳定——本包 18 个文件在 `c5383100e..4cbb3f610` 无任何改动，合并只引入 master 侧文件；②所有绑定证据的文件 hash 与收据一致（fixed-r3 0480e770…、controls-r3 2fb69729…、pair-r3 9e3467cc…/c466f02d…/5c51fc6d…）；③`fixed-r3` 与 `ace-r3` 的 `source_state` 同为 d24edfb1…（同一树）；④选择集正确（三组指向 `ace-document-*`、pilot 独立、ace 三组如声明）；⑤六红仍与 archive/1017、archive/1039、KNOWN-DEBT D-1001-R2/D-932-2 逐名一致，无新增红。

AC-07 pass：两 finding 均以独立读取（非 work 摘要）复核关闭；末次 fix 未改变实现、验收标准或证据输入，故未重跑已有效的 r3 并集（复用同一 reviewed commit 的原始 results/records，见 §9 r3 收据）。AC-08 仍待 merge。

pending_closeout_gates（merge 持有，未在复审中声称完成）：
1. 最终同步复跑：master 自 4cbb3f610 后又前进 14 个提交（967 T-10 final sliver/T-11/T-12 的 `experimental/ace/{abi,av-core,core}` 与 `auto-bindgen`；1021 `experimental/ase-m0`/`scripts/test-groups.toml`）。合并落地后需在落地树重跑固定七组 + pair + controls + ace 三组；若其差异不触及受影响面可复用本次证据并在收据说明。
2. canonical 落地 SD-01..05（四份 docs/specs 文件），落地后复读校验"交付后"陈述与实现一致，并更新 `supersedes_spec_components` 核对。
3. 归档 + 工作树 guard 清理 + plan-artifacts scope 折叠删除 + retention 登记刷新。

2026-11-14 stage: goal | plan_id: PLAN-1057 | plan_revision: r3 | round: 5（r3 work 与其复跑跨 harness 第 3–4 轮，阶段未变） | from: execution_done | to: reviewed | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1057/auto-lang | commit: 4cbb3f610 | next: merge

### 2026-11-14 merge 收据（PLAN-1057:r3）→ pass

**prepared**：reviewed r3 @4cbb3f610（复审判决见上）；交付 delta=SD-01..05（四份 `docs/specs/auto-test-case/` 文档）；预期源版本=r2 复审记录的四份 hash（c2d174d1…/e6c54461…/66849b09…/dc1d1195…）。rebase mapping（线性落地要求）：实现提交 `aeab8dfe4 → 76969f932`（`git range-diff aeab8dfe4~1..aeab8dfe4 76969f932~1..76969f932` 输出 `1: aeab8dfe4 = 1: 76969f932`，patch 等价），spec 提交 `→ 389c80f6a`（四份文件字节与 worktree 一致：bd3b9342…/c7d2e8d4…/44fa277b…/45024cbf…）。

**gate（merge-owned，冻结受影响并集，交付树上）**：
- 固定七组 `fixed-final`：gate_passed=true，425.3s，127/97/15/5/7/68/24 全 PASS；source_state `2b45c809…`；`fixed-final/results.json` ca39d436…。
- pair/controls（`pair-land2`/`controls-land2`，同实现身份树）：scalar 97 对（40 迁移）/console 15/15/immutable 5/5 三次 exit 0；controls 36/36 ok。pair-land2 记录 4b8168fd…/409f6849…/5b8fc82a…，controls 6a116251…。
- ACE 三组 `ace-final2`：gate=false（既有红），ace-host+console 164 例 **158 PASS / 6 个既有主线红**，ace-batch 12 PASS；source_state `2b45c809…`；`ace-final2/results.json` 030c7586…。6 红=archive/1017、archive/1039、KNOWN-DEBT D-1001-R2/D-932-2 已记名者（closures/tr_native/abi_recipe/a2r_hir×2/av_source_inventory），本包未改其面。
- 首红与环境归因（保留）：首次交付树跑 `ace-deliver` 出现 13 红，其中 7 项为 `mn worker … Deadline`/派生 Error（50s 编译-执行截止，同期他计划在跑重型 ACE 套件）；空闲复跑 `ace-deliver2`（同 source_state d64c45bb…）回落为同一 6 名红 → 环境归因，首红保留。
- 最末 master 增量复用说明：落地前 master 又前进 3 个提交（967 T-14），其文件仅 `experimental/ace/crates/abi/src/{lib.rs,provider_build_cache.rs}`、`abi/tests/cases/f0_t14.rs`、`abi/tests/conformance.rs`(+2 行) 与文档——只属 ace-abi 构建缓存腿及其自测注册，不在本并集任何组的选择面（ace-abi 组未选、ace-host 读 ace-conformance 二进制），故上述 `2b45c809…` 证据对最终交付提交保持有效；其余提交面在 rebase 中逐次重跑（`c5383100e`/`4cbb3f610`/`68735eba4`/`992335bc1` 四轮并集记录在 scope）。

**landed**：主检出 `git merge --ff-only plan-1057-dev` 成功，master tip=**389c80f6a**（交付提交，无 merge commit；上一 tip 7f18642a8）；落地 diff=19 个实现/测试/脚本文件 + 四份 spec。`git status` 交付树 clean。

**canonical_verified**（master @389c80f6a 实测 hash）：`docs/specs/auto-test-case/document-cases.md` bd3b9342fabb978effc5d98184daad3b45e49756d741499d7cbf2f12a445b861（7897B）；`module-selection.md` c7d2e8d4e47c5c62fbf7297034b84df7876641d53d545160cc9792967438186a（14967B）；`ace-m1-conformance.md` 44fa277bf5953d0848dcdf1f2195be41465e0ab6c51617193d9447a3d826fbf2（39773B）；`legacy-immutable-diagnostic.md` 45024cbfce45cc2b10e3df7c8a440d96adb8f3775f62730c1350ac593a66afed（3483B）。SD-04 同时改正了旧规范里指向已不存在文件（`crates/auto-lang/src/tests/ace_m1_tests.rs`）的测试位置。无 `docs/specs/` 外知识库落点，无 legacy ledger 写入。

**archived**：`docs/plans/archive/1057-autodown-document-cases-first-wave.md`，`status: archived`；T-01..T-07 ✅、T-08 收尾中；AC-01..07 ✅、AC-08 待 cleaned 收据；归档文件随本收据提交在 master。

**cleaned**（2026-11-14）：`wt-guard.sh`（Git Bash）对 `auto-lang` 与 `auto-down` 两棵树均输出 `clean`（无 reparse point）；`git worktree remove` 两棵树成功，`git branch -d plan-1057-dev`（was 389c80f6a）成功，空组目录 `D:/autostack/.wt/lang-1057` 已删除；`plan-artifacts/plan-1057/` 折叠后整目录删除（**53,356 文件 / 300,550,795 字节**）；`.retention/policy.json` 新增 `plan-1057`（deleted-at-merge、delivery/reviewed 提交、四份 canonical hash、删除计数、holds=[]），`last_merge_closeout`/`last_merge_cleanup` 刷新为 PLAN-1057。无具名未解决问题持有。

**landing checks**：本检出无 `auto` 包（`cargo build -p auto` 报 `package ID specification 'auto' did not match any packages`；仓内亦无 `auto` bin，CLAUDE.md 的该条重建指令对本检出已过时），故以本包实际产物 `auto-case` 验证：主检出 `cargo build -p auto-lang --bin auto-case` 成功（57.75s，`target/debug/auto-case.exe` 2026-11-14 16:21:31，57,582,592B），对落地语料 `auto-case lint --suite crates/auto-lang/test/cases/ace-document-base.suite.ad` → `suite ok: 6 case file(s), 36 case(s), full reference admission passed`。部署/发行包/web bundle 不属本仓本包交付面（未改动）。构建告警：`auto-lang`/`auto-lang-core` 的既有 lib 告警（5/131，含改动前即存在者）；改动文件本身无 rustfmt/告警问题（复审核）。

**batch regression due check**：`docs/plans/.last-batch-regression.json` 覆盖至 PLAN-1043（2026-11-13T19:30:42Z，covered_commit 08a9e4d0…）；1057 % 5 = 2 ≠ 0，且距该收据 < 48h → **未到期**，本轮不触发 `cargo t all` 批门；到期后按 merge/regress 规则在主检出单实例执行。

2026-11-14 stage: goal | plan_id: PLAN-1057 | plan_revision: r3 | round: 6 | from: reviewed | to: archived | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1057/auto-lang（已删除） | commit: 389c80f6a | next: done

## 10. 待澄清事项

本计划按三类范围选完整三份语料30/60，包含所需script、unit和失败双流的有限作者接线。新的正常编写入口由本包切换，仍需旧件的消费者具名登记。本轮交付为可评审计划；执行从确认后的T-01开始。
