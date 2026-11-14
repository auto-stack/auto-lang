# 文档式用例（AutoDown Document Cases）

> 状态：首批推广已交付（PLAN-1057：三份语料 30 例 / 60 检查进入日常固定组；PLAN-1002
> 四例七检查试点保留为独立回归）。本文记录当前已交付的文档式入口能力与边界；完整目标格式见
> [docs/design/testing/autodown-document-cases.md](../../design/testing/autodown-document-cases.md)。
> 解码器即合同：`crates/auto-test-case/src/author.rs`（逐标题/属性拒绝集以它为准）。

## 入口与分派

- suite 仍用既有 `test-suite` 围栏显式选文件；**case 文件**按文件级分派：
  顶层（围栏外）出现 `## Case: <key>` 二级标题 → 文档式解码；出现
  `test-case` 围栏 → 旧合同路径。同一文件混用两种入口是定位错误。
- 文档式文件编译为既有 `CaseSpec`/`CheckSpec`（schema 记 `autolang.case/2`），
  走同一 resolve/execute/比较/报告链；lint、list、inspect、inventory、
  identity-inventory、run、module-case、stale 全部可用。
- 结构判定用 AutoDown 块树 + 原文围栏范围（`document.rs`）；标题行号按
  顺序+层级与树配对（树的标题文本会剥内联标记，故不做逐字对比；配不上
  的结构在文档式解码处定位拒绝，旧合同文件不受影响）。

## 已接通标题（首批）

| 三级标题 | 产物 |
|---|---|
| `Auto` | 源码（单文件一个 `auto` 围栏；`path`/`final-newline` pair；默认 `<key>.at`、入口 program；用例层 `entry-mode` 可覆盖为 script） |
| `VM result` | avm/run 成功 + `legacy-display` 结果文本（默认 check id `vm`） |
| `VM diagnostic` | avm/run 失败（`stage`+`code` 必填；parse/type/run） |
| `Rust` | a2r/emit；单直接 `rust` 围栏或 `#### <产物路径>` 多产物；期望内联 |
| `Probe <check-id>` | probe/run；已接通 recipe 见下；`recipe`/`entry`/`caps`/`exit`，console 另可 `channel` 或成对 `stage`+`code` |
| `Notes` | 自由说明（不参与执行） |

- 四级标题：`Rust` 下=产物相对路径（载荷选项 `final-newline`/`compare`）；
  console Probe 下=`stdout`/`stderr`（各一个 `text` 围栏，空围栏=零字节）。
- 已接通探针 recipe：`ac-av/core-run-v1`、`ac-av/console-run-v1`、
  `ac/mir-native-scalar-v1`（默认各一步 run）。console 要求 `entry=d_main` 与
  stdout/stderr 双流显式断言（失败例亦不得省略）；成功例可带 `exit` 与可选
  `channel`（`value`/`unit`），失败例以成对 `stage`+`code` 表达
  （parse/resolve/typecheck/verify/run），且不得同时声明 `exit` 或 `channel`。
  `channel: unit` 不得带 `exit`；`caps` 显式空列表原样保留，加载器不补授权。
  其余 recipe 在文档式中定位拒绝。
- 属性紧跟所属标题（`key: value`，Atom 值语法），遇空行/下一标题/围栏即止；
  正文段落后出现的属性行、未知键、重复键、未知/未接通标题均带行号拒绝。
- 元数据：文件层 `tags`/`links`；用例层 `id`/`tags`/`links`/`module-name`/`entry-mode`
  （两级按序合并去重；`module-name` 只传 a2r；`entry-mode` 取 program/script，默认
  program 且不跨 case 继承）。默认 case id =
  suite 相对路径（去 `.case.ad`）+ `#<key>`；默认 check id：vm/rust；
  Probe 的 id 即节名。
- 文本载荷：LF 逻辑行（CRLF 规整），默认保留末尾换行；`final-newline: false`
  只删紧邻闭栏的一个换行；`compare: bytes|lf`。

## 内联期望与身份

- 期望产物（如 Rust 金样）以 `ArtifactPayload::Inline { bytes, open_line,
  close_line }` 随文档一次读入：执行/比较链不再打开 sidecar 文件；报告、
  inventory 与 proposal 以文档行号定位（`inline@<line>`）。
- oracle 内容哈希覆盖内联字节（改一个字节即 oracle 轴漂移→旧报告 STALE）；
  旧 sidecar 引用的哈希公式不变（console-863 / immutable-avm-920 旧 suite
  四身份轴与改动前逐字节一致；ace-m1-base 仅携带 file 产物的 24 个 check 行
  contract/check 表示轴随结构体表示变化漂移，source/oracle 内容轴不变）。
- 对照投影：`auto-case migration-inventory --suite <path>` 只读导出 schema
  `autolang.case-migration/1`，分 `semantic`（case/check/options/expect 的规范化值，
  可选值以 null 表达缺席，空串/空列表与缺席分别比较）与 `origin`（suite/fixture 根、
  case_file 与原文行号、payload 来源与来源摘要、input_paths）两部分。迁移对照工具
  `scripts/test-governance/document_case_migration.py`（pair/verify/controls）以它做
  新旧等价、actual 记录与具名负控；既有 `inventory`/`identity-inventory` schema 不变。
- 迁移对照记录：`pair` 每侧产出 `autolang.case-migration-record/1`（冻结提交、源码树
  状态、工具 sha/配置、原 suite/文档摘要、规范化投影、精确命令与返回码、report 路径/
  hash，以及每个 `(case, check)` 的唯一 run 目录与 actual 文件清单（路径/长度/hash））。
  `verify` 以"独立固定预期 + 规范化声明 + 真实 actual"三方核对；缺文件、来源不匹配、
  坏 hash、重复/missing 行与换回执均拒绝（成功 0 / 语义差异 1 / 缺证与格式错 2）。
- 试点语料：`crates/auto-lang/test/cases/ace-m2/document-pilot-1002/`
  （4 例 7 检查，B01/O03/B18/NB18 与旧 suite 同 ID 对照）；固定组
  `case-ace-document-pilot-1002`（`runner = "case"`、`domain = "ace"`、
  `all = false`）。
- 零检查用例在加载期 fail-close（定位到 `## Case:` 行），不产出可运行的
  空报告——比"六行清单点名缺行"更早拒绝（复审 F1 建议记录）。

## 活跃语料与固定组（PLAN-1057 首批）

三份文档在 `crates/auto-lang/test/cases/ace-m2/documents/`，各自 suite 与固定组同根于
`crates/auto-lang/test/cases/`；固定组名沿用调用方历史名，`suite` 字段指向活跃文档：

| 文档 | 例/检查 | 活跃 suite | 固定组 |
|---|---:|---|---|
| `scalar.case.ad`（标量四路径：ac-native/av/autovm/a2r-snapshot） | 10/40 | `ace-document-base.suite.ad`（另 include 5 份旧 M1 文件 = 57 检查，共 36 例 97 检查；`fixture_root: "ace-m1"`） | `case-ace-m1-base` |
| `console.case.ad`（console 成功/unit/value 与 parse/resolve/run 失败） | 15/15 | `ace-document-console.suite.ad` | `case-ace-m1-console-863` |
| `immutable-assignment.case.ad`（B18 三负例 + NB18/NB18S 合法邻例） | 5/5 | `ace-document-immutable.suite.ad` | `case-ace-m1-immutable-avm-920` |

- 被取代的 `ace-m1/b-scalar.case.ad`、`ace-m1/console-863/console-863.case.ad`、
  `ace-m1/immutable-avm-920.case.ad` 及其 suite 保留为**冻结历史**：仍是 M1 验收、
  来源摘要、`ace-m1-expanded` 义务表与旧回放（`document_case_migration.py pair`
  的 old 侧）的输入，日常入口不再经过它们。改写语料时新文档是唯一作者源；
  旧 suite 的被动运行结果不构成对新语料的覆盖。
- 宿主侧接线：`experimental/ace` 的 `ace_m1_base_suite_green`、
  `ace_m1_base_negative_oracle_fails`、`ace_m1_golden_drift_is_red` 与
  `console.rs::corpus()` 已指向文档式 suite，负控变异落在内联载荷上。

## 未验证面（下一批选例依据）

- 多文件 Auto（`####` 源码节）、`VM`/`VM stderr`/`VM exit` 合并单次运行、
  C/Python 产物、诊断完整消息断言（本批 message 断言为 null）、多步 recipe、
  非试点 recipe 接线、集合资源与多步会话扩面、具名 fixture 引用
  （非 UTF-8/CRLF 逐字节资产）。
- 已知既有缺口（非本格式引入）：`auto-case stale` 对 `ac/mir-native-scalar-v1`
  行的 worker 家族解析落入 sema 桶（recipe 轴噪声），修复属 stale 家族表
  owner（1024 线）。
