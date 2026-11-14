---
plan_id: PLAN-1059
status: archived
feature_name: AutoDown文档式用例第二批推广：legacy-composite
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-15
plan_revision: 1
phase: M2-A
current_step: 7
total_steps: 7
depends_on: [PLAN-1002, PLAN-1057]
supersedes_spec_components: [docs/specs/auto-test-case/document-cases.md, docs/specs/auto-test-case/ace-m1-conformance.md]
new_spec_components: []
touched_goals: []
affects: [auto-test-case]
---

# [PLAN-1059] AutoDown文档式用例第二批推广：legacy-composite

## 0. 变更摘要

把 `crates/auto-lang/test/cases/ace-m1/legacy-composite.case.ad` 的全部 **14 例 / 28 项检查**迁入拟新增的 `crates/auto-lang/test/cases/ace-m2/documents/composite.case.ad`。同一个 AutoDown 文件中，每例包含 Auto 原文、VM 结果和完整 Rust 金样；使用现有文档式标题与属性，不增加作者语法。

正常入口沿用 `cargo t case-ace-m1-base`，在现行 `ace-document-base.suite.ad` 中替换一个 include。BASE 迁后为 **24 个文档式 case / 68 项检查 + 12 个旧格式 case / 29 项检查 = 36 例 / 97 项**。1057 首批与本批按唯一身份累计 **44 例 / 88 项**文档式检查；1002 独立试点的重复身份另列，不重复累加。本轮只制定计划，实施、负控、入口切换和独立复审均尚未执行。

## 1. 目标

- 14 份 Auto、14 份 Rust 金样和28个原 check 在文档中一一对应，作者阅读每例即可看到输入与两种预期。
- 保持 case/check ID、tags/links、虚拟源码路径、program 入口、模块名、adapter/action、成功状态、VM 字符串与 Rust 字节预期的原合同。
- 使用同一构建的 runner 对冻结旧 BASE 与候选新 BASE 对跑，97项集合及规范化语义一致；本批28项另核真实 actual，而非只看 PASS。
- 新旧对照和具名负控通过后切换现行 BASE include，宿主及治理复制入口同步消费新文档；原标量40项仍来自文档式。
- 登记冻结旧资产的实际消费者，保留其原内容摘要与旧回放能力。

本批迁移作者格式与活跃入口。`VM result` 仍调用旧 AutoVM，`Rust` 仍调用旧 a2r 的 emit/snapshot；不会据此增加 ACE AV/native 检查或声明 Rust 编译/运行认证。M1 原件头注释是历史描述，不用它推断最新 ACE 支持范围。C/Python、多文件 Auto、额外 recipe、资源会话语法、其他旧语料的迁移均在本批之外。

## 2. 架构方案

`composite.case.ad → author::decode_document → CaseSpec/CheckSpec → 原 resolve/execute/父侧比较 → report + migration-inventory`。

一例使用一个 `## Case:`，其下固定 `### Auto`、`### VM result`、`### Rust`。围栏内载荷来自真实源文件及金样全文。文件层共享 `tags`/`links`；用例层显式保留旧 ID 和 module-name；两项检查显式保留旧 check-id，Auto 显式指定 `path: "main.at"`。默认 program、schema、success、bytes 与末尾 LF 不重复写；VM 数字文本用 `final-newline: false` 保持原字符串。

沿用现有混合 suite 布局。新文档源/期望均内联，BASE 的 `fixture_root: "ace-m1"` 仍服务其余旧文件。旧 `ace-m1/ace-m1-base.suite.ad` 保持冻结，用作 old 侧；候选和切换后的 active 侧使用现行共同根 suite。

## 3. 技术栈与工作位置

- 已交付的 AutoDown 标题/Atom pair 解码、`migration-inventory`、原 avm/a2r adapter、Python `document_case_migration.py` 和固定 Cargo 分组。
- 未来唯一实施树：`D:/autostack/.wt/lang-1059/auto-lang`，分支 `plan-1059-dev`；用 `scripts/new-wt-group.sh` 建组并准备 auto-down 兄弟树，禁止链接。规划阶段不创建实施树。
- 未来唯一材料目录：`D:/autostack/plan-artifacts/plan-1059/`。开工登记 owner、用途、在途状态；候选语料副本、输入闭包索引、对照报告、负控、终态与计时全部放该 scope。正常构建在专属树；不在项目目录写临时用例。
- 计划簿记在主检出；merge 将验证摘要折入本计划，按实际使用路径逐一核查并清理材料及 worktree。

## 4. 需求分析与背景调查

### 4.1 授权、规范与依赖

用户 2026-11-14 授权新建本批迁移计划，明确对象14例/28检查，要求实际 case、新旧对照、负控和入口切换验收，**先只规划**。本轮允许计划文件、取号与路线簿记；没有实施、切换入口或执行测试的授权。本稿 r1 的未来 work 入口是 T-01，实际执行待用户后续指令。

**2026-11-15 执行授权**：用户以 `/goal 完成计划1059`（auto-plan-goal 范式）授权全链执行——在计划专属 worktree 内实施（含创建）、独立复审、合同内有限修复与 merge；该授权即 T-01 所等待的"用户后续执行指令"。goal/acceptance/scope/budget/仓库集不变更。

取材时主线 HEAD=`4007a826ca6a9cc69ef7bdf88d16945857e17c97`；规划期间主线有并行推进，以下内容 hash 是所读文件的具体身份，T-01 必须重新核对实际基面。规范来源为 [总览](../specs/overview.md)、[文档式用例](../specs/auto-test-case/document-cases.md)、[固定组选择](../specs/auto-test-case/module-selection.md)、[M1一致性](../specs/auto-test-case/ace-m1-conformance.md)及[文档式设计](../design/testing/autodown-document-cases.md)。

[1002](archive/1002-autodown-document-cases.md)与[1057](archive/1057-autodown-document-cases-first-wave.md)已归档，现有 `Auto`/`VM result`/`Rust` 可覆盖本批。1017 已交付，遵循常规组与 `ace-perf` 分离；本批没有正式性能修改，不把 `ace-perf` 附加为门禁。1058 是混合语言编辑原型；ACE M2 语义/FFI/标准库计划也不是本批前置。1024 审计有旧文件摘要登记，属于冻结来源消费者，不是同一迁移计划；后续实施如与在途改动重叠，应同步真实基面而不是覆盖其他工作。

### 4.2 已观测到的接线与缺口

| 位置 / 符号 | 当前事实 | 本批动作 |
|---|---|---|
| `ace-document-base.suite.ad` | scalar 文档40项 + 5份旧文件57项；含 legacy-composite28项 | 仅将该 include 换为新 composite 文档；其余4份旧文件29项保留 |
| `scripts/test-groups.toml::case-ace-m1-base` | suite 已指向 active BASE，标题仍写1057/余57项 | 保持组身份/配置，更新说明为68项文档式 +29项旧格式 |
| `document_case_migration.py::FAMILY_SPECS` | scalar/console/immutable；`EXTRA_BASE_PAIRS` 已含本批28项 | 增 composite 家族，复用BASE97全集，不把28项再次追加成125项 |
| `stage_isolated_corpus` | 明列3份新文档，尚无 composite 文件 | 在隔离副本闭包中加入 composite；既有 controls 继续可跑 |
| `host/language_conformance.rs::stage_doc_base_corpus` | 复制 BASE、scalar 与 ace-m1 树 | 同步复制 composite；保留既有两个内联标量反控 |
| `ace_m1_base_suite_green` | 真实运行 active BASE，断言97项 | 增来源/成员核验，确认本批28和原标量40实际来自文档式 |
| `tests/governance.rs::document_base_cli_list_and_migration_inventory_complete` | 将所有document行命名为doc_scalar并硬断言40项 | 按固定ID分开核scalar40/composite28及旧29，全部document共68；保持总97与零外部source输入断言 |
| `verify_actual_file_integrity` / `verify_migrated_row_actual` | 前者重算files摘要，后者比较记录中的avm.result_text/a2r.artifacts；离线verify没有完整地把这些派生值重新绑定实际文件内容 | 本批actual验收直接重读VM文件/重算Rust实际摘要并与原expect比较，不能只核记录内缓存字段；NC-11覆盖摘要自洽但缓存仍旧的变体 |
| `scripts/tests/test_document_case_migration.py` | 现有对照记录/反例单测 | 加 composite 的固定14身份、28检查、实际结果差异和证据拒绝测试 |
| `ace-m1/source-manifest.json`、M1/expanded治理、旧回放 | pin旧声明、src/oracle等摘要 | 保留原件，登记具体消费者，沿原入口验证原 pin |

本次已静态读取全部14例声明及其源/Rust全文；计数来自实际声明。未运行原套件，本表的预期不是本次实测成绩。

### 4.3 声明与入口基线

| 文件（相对 cases 根，最后一行为仓库根） | bytes | SHA-256 |
|---|---:|---|
| `ace-m1/legacy-composite.case.ad` | 9055 | `c53da5c0b3ccf2942242841e2d862642563ce409fb60e64df63deef665846f44` |
| `ace-m1/ace-m1-base.suite.ad` | 704 | `7fd00b8d1c79d45be1f4d5ad90083780a8acb67798c6c36b9d29b8f53d26ae63` |
| `ace-document-base.suite.ad` | 528 | `4259589b56452b656f35f320a01f16b78f01d29a8c0d344e1f22d2c31d3be38f` |
| `scripts/test-governance/document_case_migration.py` | 61498 | `094c2b72b0ae6de0dc39e0dc3e14a3f1924763617a160b04b9d5a59604b7ae58` |

## 5. 详细设计

### 5.1 逐例清单与精确预期

旧根为 `crates/auto-lang/test/cases/ace-m1/`，新文件拟为 `crates/auto-lang/test/cases/ace-m2/documents/composite.case.ad`。顺序沿原文件。表中结果是 legacy-display 文本，不是 stdout 或进程 exit。每例恰好2检查。

| 序号 | case ID（新旧相同） | 原 Auto 文件 | 实际行为 | VM result | module-name / Rust逻辑路径 |
|---|---|---|---|---|---|
| 01 | `ace.m1.k01` | `src/K01.at` | move 闭包按值捕获 base=40，再加参数2 | `"42"` | `m1_k01` / `m1_k01.rs` |
| 02 | `ace.m1.k03` | `src/K03.at` | 非 move 闭包捕获 base=40，再加参数2 | `"42"` | `m1_k03` / `m1_k03.rs` |
| 03 | `ace.m1.k06` | `src/K06.at` | 可变 move 闭包共享捕获槽，两次调用均返回21 | `"42"` | `m1_k06` / `m1_k06.rs` |
| 04 | `ace.m1.r02` | `src/R02.at` | 嵌套 record 字段更新，7+2 | `"9"` | `m1_r02` / `m1_r02.rs` |
| 05 | `ace.m1.r05` | `src/R05.at` | 含 Str 的 record 经 move 参数和返回值传递，41+len("x") | `"42"` | `m1_r05` / `m1_r05.rs` |
| 06 | `ace.m1.r06` | `src/R06.at` | record 重建并交换字段，2*10+1 | `"21"` | `m1_r06` / `m1_r06.rs` |
| 07 | `ace.m1.r08` | `src/R08.at` | record 静态构造与只读实例方法 | `"42"` | `m1_r08` / `m1_r08.rs` |
| 08 | `ace.m1.t05` | `src/T05.at` | 含字面转义 `\0` 的字符串相等/不等分支 | `"42"` | `m1_t05` / `m1_t05.rs` |
| 09 | `ace.m1.v01` | `src/V01.at` | Option Some(42) 解构 | `"42"` | `m1_v01` / `m1_v01.rs` |
| 10 | `ace.m1.v02` | `src/V02.at` | Result Ok(84/2) 解构，保留 Err 分支 | `"42"` | `m1_v02` / `m1_v02.rs` |
| 11 | `ace.m1.v04` | `src/V04.at` | 枚举 Token.Number(42) 解构，保留 End 分支 | `"42"` | `m1_v04` / `m1_v04.rs` |
| 12 | `ace.m1.v05` | `src/V05.at` | 枚举携带 owned record，len("hello")+37 | `"42"` | `m1_v05` / `m1_v05.rs` |
| 13 | `ace.m1.v06` | `src/V06.at` | Result 从 Err("old") 赋值为 Ok(42) | `"42"` | `m1_v06` / `m1_v06.rs` |
| 14 | `ace.m1.v07` | `src/V07.at` | Option None 路径返回42，保留 Some 分支 | `"42"` | `m1_v07` / `m1_v07.rs` |

### 5.2 全部28项新旧检查对照

下表每行都是独立验收义务，不能用14例的case计数代替。Rust 对照实际发射文件与原金样的完整长度/SHA/bytes；不得只比首行、格式化文本或以当前生成结果刷新预期。

| 映射 ID | case | 旧 check → 新 check | 新三级标题 | 保留的执行和判决合同 |
|---|---|---|---|---|
| C01-VM | `ace.m1.k01` | `autovm → autovm` | `VM result` | `avm/run`；success；legacy-display=`"42"`，无末尾 LF |
| C01-RS | `ace.m1.k01` | `a2r-snapshot → a2r-snapshot` | `Rust` | `a2r/emit`；success；module=`m1_k01`；`m1_k01.rs` 对 `oracle/m1_k01.rs` 全文 bytes |
| C02-VM | `ace.m1.k03` | `autovm → autovm` | `VM result` | `avm/run`；success；legacy-display=`"42"`，无末尾 LF |
| C02-RS | `ace.m1.k03` | `a2r-snapshot → a2r-snapshot` | `Rust` | `a2r/emit`；success；module=`m1_k03`；`m1_k03.rs` 对 `oracle/m1_k03.rs` 全文 bytes |
| C03-VM | `ace.m1.k06` | `autovm → autovm` | `VM result` | `avm/run`；success；legacy-display=`"42"`，无末尾 LF |
| C03-RS | `ace.m1.k06` | `a2r-snapshot → a2r-snapshot` | `Rust` | `a2r/emit`；success；module=`m1_k06`；`m1_k06.rs` 对 `oracle/m1_k06.rs` 全文 bytes |
| C04-VM | `ace.m1.r02` | `autovm → autovm` | `VM result` | `avm/run`；success；legacy-display=`"9"`，无末尾 LF |
| C04-RS | `ace.m1.r02` | `a2r-snapshot → a2r-snapshot` | `Rust` | `a2r/emit`；success；module=`m1_r02`；`m1_r02.rs` 对 `oracle/m1_r02.rs` 全文 bytes |
| C05-VM | `ace.m1.r05` | `autovm → autovm` | `VM result` | `avm/run`；success；legacy-display=`"42"`，无末尾 LF |
| C05-RS | `ace.m1.r05` | `a2r-snapshot → a2r-snapshot` | `Rust` | `a2r/emit`；success；module=`m1_r05`；`m1_r05.rs` 对 `oracle/m1_r05.rs` 全文 bytes |
| C06-VM | `ace.m1.r06` | `autovm → autovm` | `VM result` | `avm/run`；success；legacy-display=`"21"`，无末尾 LF |
| C06-RS | `ace.m1.r06` | `a2r-snapshot → a2r-snapshot` | `Rust` | `a2r/emit`；success；module=`m1_r06`；`m1_r06.rs` 对 `oracle/m1_r06.rs` 全文 bytes |
| C07-VM | `ace.m1.r08` | `autovm → autovm` | `VM result` | `avm/run`；success；legacy-display=`"42"`，无末尾 LF |
| C07-RS | `ace.m1.r08` | `a2r-snapshot → a2r-snapshot` | `Rust` | `a2r/emit`；success；module=`m1_r08`；`m1_r08.rs` 对 `oracle/m1_r08.rs` 全文 bytes |
| C08-VM | `ace.m1.t05` | `autovm → autovm` | `VM result` | `avm/run`；success；legacy-display=`"42"`，无末尾 LF |
| C08-RS | `ace.m1.t05` | `a2r-snapshot → a2r-snapshot` | `Rust` | `a2r/emit`；success；module=`m1_t05`；`m1_t05.rs` 对 `oracle/m1_t05.rs` 全文 bytes |
| C09-VM | `ace.m1.v01` | `autovm → autovm` | `VM result` | `avm/run`；success；legacy-display=`"42"`，无末尾 LF |
| C09-RS | `ace.m1.v01` | `a2r-snapshot → a2r-snapshot` | `Rust` | `a2r/emit`；success；module=`m1_v01`；`m1_v01.rs` 对 `oracle/m1_v01.rs` 全文 bytes |
| C10-VM | `ace.m1.v02` | `autovm → autovm` | `VM result` | `avm/run`；success；legacy-display=`"42"`，无末尾 LF |
| C10-RS | `ace.m1.v02` | `a2r-snapshot → a2r-snapshot` | `Rust` | `a2r/emit`；success；module=`m1_v02`；`m1_v02.rs` 对 `oracle/m1_v02.rs` 全文 bytes |
| C11-VM | `ace.m1.v04` | `autovm → autovm` | `VM result` | `avm/run`；success；legacy-display=`"42"`，无末尾 LF |
| C11-RS | `ace.m1.v04` | `a2r-snapshot → a2r-snapshot` | `Rust` | `a2r/emit`；success；module=`m1_v04`；`m1_v04.rs` 对 `oracle/m1_v04.rs` 全文 bytes |
| C12-VM | `ace.m1.v05` | `autovm → autovm` | `VM result` | `avm/run`；success；legacy-display=`"42"`，无末尾 LF |
| C12-RS | `ace.m1.v05` | `a2r-snapshot → a2r-snapshot` | `Rust` | `a2r/emit`；success；module=`m1_v05`；`m1_v05.rs` 对 `oracle/m1_v05.rs` 全文 bytes |
| C13-VM | `ace.m1.v06` | `autovm → autovm` | `VM result` | `avm/run`；success；legacy-display=`"42"`，无末尾 LF |
| C13-RS | `ace.m1.v06` | `a2r-snapshot → a2r-snapshot` | `Rust` | `a2r/emit`；success；module=`m1_v06`；`m1_v06.rs` 对 `oracle/m1_v06.rs` 全文 bytes |
| C14-VM | `ace.m1.v07` | `autovm → autovm` | `VM result` | `avm/run`；success；legacy-display=`"42"`，无末尾 LF |
| C14-RS | `ace.m1.v07` | `a2r-snapshot → a2r-snapshot` | `Rust` | `a2r/emit`；success；module=`m1_v07`；`m1_v07.rs` 对 `oracle/m1_v07.rs` 全文 bytes |

所有旧项都为 `entry.mode=program`、`source.syntax=auto`、虚拟 `source.path=main.at`、tags精确为 `["ace-m1", "legacy-only"]`、links精确为 `["PLAN-852"]`，无skip；completion均success。原声明没有 stdout/stderr/exit/recipe/steps 等断言，迁移不补这些断言。新增1059链接写正文说明，避免改变逐行元数据语义。

### 5.3 源码与金样的独立字节基线

下表为原件内容摘要，路径相对旧根；T-01 重新生成并核对。新文档解码后的 source bytes 与 golden bytes 分别对这些原件核对，`origin.source_sha256` 若为组合摘要则遵守现有投影算法，不与单文件hash混淆。源和Rust全部保留原有末尾 LF，VM结果恰为数字字符、末尾零LF。

| 原件 | bytes | SHA-256 |
|---|---:|---|
| `src/K01.at` | 104 | `d287a88b8009e586a3643aaf2963ce360a2fc91f90c6fdb4dbbb05e92d1be0df` |
| `oracle/m1_k01.rs` | 147 | `a41dd1f0074cfcefdc8f3984f987fd71ee7aefdcd0ea11981b2d21d52837761f` |
| `src/K03.at` | 99 | `7118326786b8a6edd703cf3735a983aa8bef3bc3e83092a20b1913eae98041ae` |
| `oracle/m1_k03.rs` | 142 | `350b633e11fce94c30b050d6b30cc61777f380feeed7fb1f77077629be790f1c` |
| `src/K06.at` | 143 | `aa426014e0d01ea43505e0a2cc23cc672105481d59cba301030b7dc5282382c3` |
| `oracle/m1_k06.rs` | 414 | `e28247fafcc7676842763f2c6d65cb8b2d81af63ce01db226f6f316c9afc897a` |
| `src/R02.at` | 175 | `6c74d77b3ba20bfcf5abe4bcf97424a4717a217e9333101464b79e0fec07340f` |
| `oracle/m1_r02.rs` | 397 | `09d3f186faf52c650c3944398ecbc6f00ef2b2891190a139971193d2234be427` |
| `src/R05.at` | 191 | `f46125c461bdf55770c7642dc492ce507c9daf026ffbf0a565a797af6040b844` |
| `oracle/m1_r05.rs` | 348 | `11375a3948d661473f4d29bc032b24fbaa4e8916314a049be0d673383a9a4ca7` |
| `src/R06.at` | 159 | `c3f55da8433a4cc67cc3bba2eb03ebf56bff2cba1199bffcffcbdb698d745302` |
| `oracle/m1_r06.rs` | 301 | `14fbe8ebff2187232cbae84843060b175d2b466373034639b253397fc515a225` |
| `src/R08.at` | 196 | `b2ea23e7f6acb41100ddeee036a648f5cc92b6857608e66f5fbd1f4d5ef9c895` |
| `oracle/m1_r08.rs` | 368 | `8a015a03a73ada36499348047b7f2372f7ecae881e528362b5bc66ae31c7a697` |
| `src/T05.at` | 116 | `84541676753c15d06dc263efa57105501d87a07907102ae989de5020daf47530` |
| `oracle/m1_t05.rs` | 200 | `95f83961f608f29ae6a6c46521034e8d1587163ae0346b424dd1788a365ba18f` |
| `src/V01.at` | 191 | `900acc88f43f9505b91fa6c4096642589fa7e3ca770833106bd552c481857dee` |
| `oracle/m1_v01.rs` | 255 | `496f4c067efa8a8d001b7d945e8f6f01b3ae096012e1a9b9f43c8698a9a0f129` |
| `src/V02.at` | 230 | `281396b1a3d5ea129a5c4e34672287975bac87ed7b543af7fbc828ce7869259f` |
| `oracle/m1_v02.rs` | 307 | `dea6dda290b25c1a91f2c69758a918dc47f5713e9cd16519a9552e135b13f736` |
| `src/V04.at` | 177 | `c9303cad597133e0a517b70d5f0693a8a4961199a3148b82abffe039e22839cc` |
| `oracle/m1_v04.rs` | 284 | `76ed2a76f78a0f02914e74954dff7564bc76e970990ddf6cc5f553c934d66428` |
| `src/V05.at` | 240 | `734c211d1ea0e62d52f14617369a77c975c0a23c915e3ed9dff91fa1f4c7c0c3` |
| `oracle/m1_v05.rs` | 437 | `63b72a7074c502b69c230e5964c156471e27c95acf49958cf8f96ba0e8553418` |
| `src/V06.at` | 157 | `7145928b2c570c848f6ca7746545d1ca3d31790045855080d05db4c3d4112383` |
| `oracle/m1_v06.rs` | 220 | `4db41ba368309501028544a9886209d02ee0f07906631178087ceaed9db22aeb` |
| `src/V07.at` | 192 | `04844f208d458a043dd6ef2d2202a8ffb09789010d750d075cc164ee59135478` |
| `oracle/m1_v07.rs` | 256 | `6237a3350fa245465a6a7bd81187e31334a964842e1fd2322352fecd24fbac38` |

### 5.4 可直接落地的文档范例（4个实际case）

以下是拟新增文档的完整文件头和其中4个完整case；其余10例按§5.1/5.2同构展开。围栏外层五反引号只用于在本计划展示文档，落地文件不含该外层。四个示例中的Auto及Rust均取自当前原件，未省略内容。T05的 `\0` 在AutoDown原文中是反斜杠加数字0，围栏内容不能按属性字符串再次解转义。V05金样的空行也保留。

`````markdown
# ACE Legacy Composite Document Corpus
tags: ["ace-m1", "legacy-only"]
links: ["PLAN-852"]

PLAN-1059 migrates 14 cases / 28 checks. AutoVM values and a2r emitted bytes keep their original contracts.

## Case: k06-mutable-move-closure
id: "ace.m1.k06"
module-name: "m1_k06"

### Auto
path: "main.at"

````auto
fn main() int {
    var base int = 1
    var f = move (n int) => {
        base = base + n
        return base
    }
    return f(20) + f(0)
}
````

### VM result
check-id: "autovm"
final-newline: false

```text
42
```

### Rust
check-id: "a2r-snapshot"

````rust
// Auto-generated by a2r transpiler

#[allow(unused_imports)]
use std::rc::Rc;
#[allow(unused_imports)]
use std::cell::{RefCell, Cell};
fn main() -> i64 {
    let base = std::rc::Rc::new(std::cell::RefCell::new(1));
    let mut f = { let base = std::rc::Rc::clone(&base); move |n: i64| { { let __a2r_wc = *base.borrow() + n; *base.borrow_mut() = __a2r_wc; }; return *base.borrow(); } };
    return f(20) + f(0);
}
````

## Case: r02-nested-field-update
id: "ace.m1.r02"
module-name: "m1_r02"

### Auto
path: "main.at"

````auto
type Inner { value int }
type Outer { inner Inner; extra int }
fn main() int {
    var p Outer = Outer(Inner(3), 2)
    p.inner.value = 7
    return p.inner.value + p.extra
}
````

### VM result
check-id: "autovm"
final-newline: false

```text
9
```

### Rust
check-id: "a2r-snapshot"

````rust
// Auto-generated by a2r transpiler

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Inner {
    pub value: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Outer {
    pub inner: Inner,
    pub extra: i64,
}

fn main() -> i64 {
    let mut p: Outer = Outer { inner: Inner { value: 3 }, extra: 2 };
    p.inner.value = 7;
    return p.inner.value + p.extra;
}
````

## Case: t05-embedded-zero
id: "ace.m1.t05"
module-name: "m1_t05"

### Auto
path: "main.at"

````auto
fn main() int {
    let a str = "A\0B"
    if a == "A" { return 1 }
    if a != "A\0B" { return 2 }
    return 42
}
````

### VM result
check-id: "autovm"
final-newline: false

```text
42
```

### Rust
check-id: "a2r-snapshot"

````rust
// Auto-generated by a2r transpiler

fn main() -> i64 {
    let a: String = "A\0B".to_string();
    if a == "A" {
        return 1;
    }
    if a != "A\0B" {
        return 2;
    }
    return 42;
}
````

## Case: v05-owned-record-variant
id: "ace.m1.v05"
module-name: "m1_v05"

### Auto
path: "main.at"

````auto
type Item { name Str; code int }
enum Token {
    Data(Item)
    End
}
fn main() int {
    let t = Token.Data(Item("hello", 37))
    is t {
        Token.Data(v) -> { return v.name.len() + v.code }
        Token.End -> { return 1 }
    }
}
````

### VM result
check-id: "autovm"
final-newline: false

```text
42
```

### Rust
check-id: "a2r-snapshot"

````rust
// Auto-generated by a2r transpiler

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Item {
    pub name: String,
    pub code: i64,
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Data(Item),
    End,
}


fn main() -> i64 {
    let t = Token::Data(Item { name: "hello".to_string(), code: 37 });
    match t {
        Token::Data(v) => return (v.name.len() as i64) + v.code,
        Token::End => return 1,
    }
}
````

`````

仅需显式写历史身份、模块名和 `main.at`，其余用惯例。文件层元数据只合并到各case，不应串到其他文档；同文件的14例互不继承对方属性。`Rust` 的默认产物名由显式 module-name 生成，必须等于§5.1；不必为每例再写四级文件标题。加载后语义必须与表中相同，不能通过删除check-id而接受默认 `vm`/`rust` 身份。

### 5.5 对照器扩展、actual与记录

在现有 `document_case_migration.py` 增 `family=composite`：

- 固定 `migrated_cases` 为§5.1逐字列出的14 ID，`migrated_pairs` 为§5.2的28对；`expected_pairs` 为既有BASE97全集。单测用独立字面清单核对，不能仅从被测常量反推期待值。
- 两侧97行规范化 `semantic` 深比较：来源表示与行号可变，source逻辑路径/内容、entry、tags/links、options与expect不可变。对new侧本批28项要求 `origin.format=document`、`semantic.case.files=[]` 与 `origin.input_paths=[]`；原标量40项也保持document。剩余29项继续fences且逐ID存在，无duplicate/extra/missing/skip。
- `collect_actuals`/`verify_migrated_row_actual` 复用现有avm与a2r分支：每个VM核其真实 `result.txt` 的完整文本；每个Rust核实际产物路径集合、长度和SHA/bytes，绑定唯一case/check run目录。离线 `verify` 也必须从已校验路径重读VM文本/重算Rust摘要再对原expect判决，并核记录派生值与文件绑定；不能仅比较记录缓存的result_text/artifacts。成功状态不能代替actual文件；缺文件/坏摘要/范围错误退出2，语义、成员或实际结果不符退出1。NC-11要求实际内容与oracle不符时明确退出1，不因旧缓存值仍正确而通过。
- old/new用同一个实际runner及worker/tool配置。记录源码提交/树状态、工具hash、suite与文档hash、投影、精确命令、退出码、report路径/hash、actual路径/长度/hash。固定原件hash给独立oracle，避免两侧同时漂移仍相等。
- 更新 `comparison.json` 的表示描述，按每行实际 `origin.format` 或批次归属表达。本批视角下额外69项中有40项已文档化，不可一概写成shared-legacy。原schema不必升版。
- `controls`拟增加可选 `--family composite`：执行§6.2的本批具名控制；不传参数时保留1057原控制合同（当前36个子项）。本批控制从新文档构造scope内候选BASE副本：若仓库active尚未切换，则只在副本替换composite include，先核97项和目标28项的document来源，再施加变异，确保反控实际消费新文档。新参数、family选择、隔离复制与输出索引由T-03交付，目前尚不可运行。

除必要的本批对照器与验收补充，不扩展产品解码语法、runner或后端。若发现现有能力不能无损表达某个原件，T-01记录具体字段/字节和原因，提出修订后再继续依赖任务，不靠放宽比较绕过。

### 5.6 候选验证与正式入口切换

先在scope中复制现行BASE及其include/source/oracle/helper闭包，加入新 composite文档，构造候选suite；输入根与输出根分开，复制前后核长度/hash。候选suite的相对路径和fixture_root与正式布局相同。通过V-01～V-03、V-04的候选对照部分与NC-01～NC-15后，T-05才修改真实active suite中的这一行：

```text
ace-m1/legacy-composite.case.ad → ace-m2/documents/composite.case.ad
```

固定组 `case-ace-m1-base` 仍指向 `crates/auto-lang/test/cases/ace-document-base.suite.ad`，ID仍为 `ace-document-base`。**这里切换的是suite include，不是新建一套并行的日常固定组。** 新active BASE共6个case文件：scalar+composite两文档68项，b-traps/b-diags/legacy-negative/session-lifecycle四旧文件29项。

正常 `cargo t case-ace-m1-base`、CLI list/migration-inventory/run和宿主BASE都必须认证36例/97项，并证明这28项的来源是新文档。更新Python和Rust两个staging函数；修改 `experimental/ace/tests/conformance/tests/governance.rs::document_base_cli_list_and_migration_inventory_complete` 的40项硬断言，按独立身份集合分别核40/28/29与总document68，不能只把40改成68而漏验归属。两个既有标量宿主反控仍实际运行且各仅红目标项。旧原件不应作为新文档读取源/金样的间接依赖，NC-14的无sidecar副本要实际28项全绿。

### 5.7 冻结原件及消费者去向

| 资产 / 消费者 | 交付后的去向与验收 |
|---|---|
| 旧 `legacy-composite.case.ad`、14个src、14个oracle | 保留冻结原字节，old BASE、M1 source-manifest及相关展开义务继续引用；active composite作者源为新文档 |
| `ace-m1/ace-m1-base.suite.ad` | 保留原suite与97全集，作为本批pair old侧；不改其include和源pin |
| `source-manifest.json`、`ace-m1-expanded` pinned合同、`ace_m1_legacy_base.py`/`ace_m1_legacy_a2r.py` | T-01从真实引用产出路径、绑定字段/摘要、用途、owner与后继关闭条件清单；原固定治理组继续验证，不能为删sidecar级联重写冻结pin |
| host BASE三个具名测试 | active入口 + 新复合文档闭包；断言97、内联错exit/金样变异分别红 |
| `experimental/ace/tests/conformance/tests/governance.rs`的 `document_base_cli_list_and_migration_inventory_complete` / `reuse_obligations_run_green_via_original_suites`，以及同目录 `batch.rs` / include_str等 | T-01逐项检索实际可达来源；日常入口指新active，具名历史回放保原来源，不能把后者PASS冒充新文档覆盖 |

旧副本的物理保留不改变新active作者源。将来整体历史合同退役才讨论删除；本计划不把“无消费者原件清理”留作未认领任务。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before / after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-test-case/document-cases.md` | 首批三份30/60；BASE10/40+旧57 → 四份44/88；BASE24/68+旧29；登记composite14/28、实际标题、对照家族及冻结原件 | 当前作者入口与已验证覆盖可直接查阅，1002试点不重复计数 | AC-01～06、AC-08 |
| SD-02 | modify | `docs/specs/auto-test-case/ace-m1-conformance.md` 底座套件节 | BASE36/97总数保留，组成从40文档+57旧改为68文档+29旧；明确14复合例仍为旧AutoVM与a2r快照 | 作者格式覆盖不被误当后端扩面或编译运行认证 | AC-02、AC-05、AC-06、AC-08 |

无新增公共协议/模块，无目标达成状态翻转，`touched_goals=[]`。merge按最终评审确认的事实沉淀，规划不先改canonical Specs。

## 6. 测试设计

以下所有运行与负控为**待执行**。work负责实际迁移、内部核对与反例；review在同一实施树独立读取diff与原件，对AC逐项核实，不能只接受打勾；merge持冻结提交的受影响完整组并集及收尾，不另开第二棵审查实施树。复用身份/输入未变的有效证据，源码/工具实际漂移时才更新相关证据。

### 6.1 正例、覆盖边界与对照入口

| ID / owner_stage | 保护的规则与输入 | 实际入口 | 明确可观察结果 / 关闭条件 | AC |
|---|---|---|---|---|
| V-01 / work |14原case与28检查、原件hash及两侧BASE全集 | 既有migration-inventory + T-03 family=composite的投影验证 | 两侧97唯一对；本批28逐行语义相等；source/golden内容对应§5.3；新侧28+40为document，29为fences | AC-01/02 |
| V-02 / work |四个完整示例及其余10例，无sidecar作者依赖 | CLI lint/list/inspect/migration-inventory；候选副本 | lint36例/6文件；list97项；例/节/原文行号可定位；本批新侧semantic.case.files与origin.input_paths为空；无skip | AC-01/02 |
| V-03 / work |同一runner分别实际跑旧BASE和候选BASE | `document_case_migration.py pair --family composite` | exit0；verified_pairs=97、migrated_pairs=28；两侧97PASS；14个result为§5.1（R02=9、R06=21）；14个Rust实际字节等原金样 | AC-03 |
| V-04 / work |真实记录再次独立验证、前三家族兼容 | 候选阶段 `verify --family composite` / `pair --family scalar`；切换后原controls | composite97/28再次通过；候选scalar97/40通过且new40为document；切换后原控制具名子项无遗漏、当前36项预期全部被观察到 | AC-03/04 |
| V-05 / work |原固定组与宿主正常入口真正切换 | §6.3固定组并集；CLI新active投影 | BASE36/97且28源于composite文档；既有两个宿主反控各1F+96P；新闭包不缺文件；试点与batch原义务完整 | AC-05 |
| V-06 / work |历史来源与原身份不漂移 | infra-ace-m1及T-01具名M1消费检查 | source-manifest/旧suite/src/oracle与§4.3/5.3一致，原冻结消费者可用；负控仅修改scope副本 | AC-06 |
| V-07 / review |所有case/AC/负控、条件、Spec delta及遗漏 | 独立复审同树diff与原件、report/actual | 每个28义务有映射与证据，4示例/全部迁移内容可核；无未授权延后；错误有case/check/原文位置；最终结论绑定revision与代码提交 | AC-07 |

本批的可观察后端结果限avm值与a2r发射字节。普通正确性组不证明正式性能；编译产物可编译性及ACE新后端语义也不在这些正例的认证范围。

T-03实现后，在专属worktree中使用以下命令形状；所有 `--out` 取scope下尚未存在/为空的新目录，样例变量由work解析为实际绝对路径并写入记录。`$candidate` 是scope下布局完整的候选suite，不是项目内临时文件：

```powershell
$runner = (Resolve-Path './target/debug/auto-case.exe').Path
$cases = (Resolve-Path './crates/auto-lang/test/cases').Path
$scope = 'D:/autostack/plan-artifacts/plan-1059'
# $candidate：T-02登记的scope内候选suite绝对路径
python scripts/test-governance/document_case_migration.py pair --runner $runner --old-suite "$cases/ace-m1/ace-m1-base.suite.ad" --new-suite $candidate --family composite --out "$scope/pair-candidate"
python scripts/test-governance/document_case_migration.py verify --old-record "$scope/pair-candidate/old-record.json" --new-record "$scope/pair-candidate/new-record.json" --family composite
python scripts/test-governance/document_case_migration.py controls --runner $runner --family composite --out "$scope/controls-composite"
python scripts/test-governance/document_case_migration.py pair --runner $runner --old-suite "$cases/ace-m1/ace-m1-base.suite.ad" --new-suite $candidate --family scalar --out "$scope/pair-scalar-candidate"
# T-05切换真实include后核对原控制仍消费完整新闭包
python scripts/test-governance/document_case_migration.py controls --runner $runner --out "$scope/controls-1057"
```

候选与active内容一致时可复用同字节/工具身份的候选实际结果，但须额外导出真实active来源并核其路径、include、hash、97项与格式分布；正常固定组和宿主入口仍必须实际运行，不能只跑候选文件。

### 6.2 具名负控及合法邻例

每项独立副本，从冻结正常输入派生；保存变异前后位置/内容摘要、精确命令、返回码及判决详情。先断言变异确实命中指定case/节/次数，再判断预期失败；不能把任意报错算敏感性通过。NC-01～15由T-03/04接入 `controls --family composite`，即15个稳定控制ID；NC-14是合法自包含正控，其余反控的绿色是“预期拒绝/失败确实被观察到”。参数变体如后续增加，子ID须独立列出。

| ID / owner_stage=work | 变异 / 实际入口 | 区分错误实现的预期与邻例 | AC |
|---|---|---|---|
| NC-01 | 仅R02 `VM result` 9→42；真实run候选BASE副本 | 恰 `ace.m1.r02/autovm` 为FAIL且actual=9，点名result与文档节；其余96PASS | AC-04 |
| NC-02 | 仅R06 Rust金样 `p.left * 10 + p.right` 中10→11；真实run | 恰R06/a2r-snapshot FAIL，actual仍原m1_r06.rs，报告产物/字节差异与内联行；VM=21及其余96PASS | AC-04 |
| NC-03 | 仅K06 Auto初值 `var base int = 1`→2；两预期保持；真实run | K06 VM actual=44而期望42、K06 Rust初值字节不同，恰2FAIL+95PASS；捕获态顺序和两条独立执行腿确实被测到 | AC-04 |
| NC-04 | 仅T05 Rust块第一处 `"A\0B"` 改为 `"A\\0B"`，Auto不变；真实run | 恰T05/a2r-snapshot FAIL并给出bytes差异；T05 VM仍42及其余96PASS；不对正文做二次转义 | AC-02/04 |
| NC-05 | 删除V07整个VM result节，仍保留Rust；pair投影校验 | 退出1，明确缺 `ace.m1.v07/autovm`，96项全绿子集不能代替97项 | AC-01/04 |
| NC-06 | 完整重复K01 case；migration-inventory/lint | 非零并指向重复case ID/文档位置；禁止去重后冒充成功，恢复副本后97项合法 | AC-01/04 |
| NC-07 | 文件层删除 `links: ["PLAN-852"]`；pair | 退出1、指出缺历史link及受影响case，不能忽略meta只比代码/结果 | AC-02/04 |
| NC-08 | 仅K06 Auto path由main.at改other.at；pair | 退出1并指出source逻辑path差异；原main.at版本通过 | AC-02/04 |
| NC-09 | 仅K06 module-name改m1_k06_changed；pair | 退出1并指出module_name/产物路径差异；默认模块/文件名不得悄悄覆盖历史身份 | AC-02/04 |
| NC-10 | 仅R02 VM节删final-newline:false；真实run | 期望成为9加LF，实际仍9，恰该项FAIL+96PASS并显式字节/文本差异 | AC-02/04 |
| NC-11 | 副本中将R02/R06 VM actual内容9和21对换并同步长度/hash，保持case绑定；verify真实记录副本 | 退出1并点名actual与预期不符；即便摘要自洽且两个PASS标签保留也拒绝。相同42字节互换不是本项的区分依据 | AC-03/04 |
| NC-12 | 删除新记录内R05 Rust actual文件，保留其路径和原摘要；verify | 退出2并点名缺失actual；不能把缺证记成语义PASS | AC-03/04 |
| NC-13 | 对真实R02/a2r-snapshot行先stale检查，再仅改该内联金样一个有效字节并重查 | 原行Fresh；变体该行因oracle内容轴Stale；只用该行关闭，不借BASE native既知家族轴噪声替代 | AC-02/04 |
| NC-14 | scope中只放完整composite文档及单文件suite，不提供ace-m1/src或oracle；真实run该suite | 恰14例/28PASS，14result与14Rust逐项核actual；origin无外部input，证明执行不回读旧sidecar。此为合法正控 | AC-03/05 |
| NC-15 | 复制候选BASE（切换后同active布局）的完整输入闭包后，仅移除新composite文件；真实load/run | 失败明确指向缺失的composite include，不能自动退回legacy-composite或忽略文件；恢复文件后BASE97PASS | AC-04/05 |

NC-13沿用现有stale入口与1057的单行报告裁剪方法，不修改stale产品合同。NC-11/12只变异副本记录/actual，不伪造新产品运行成绩。各反控恢复由重新复制正常输入完成，不回写仓库文档。

### 6.3 固定职责组、完整消费者与时间记录

最终冻结提交的明确并集：

```powershell
cargo t auto-test-case case-ace-m1-base case-ace-document-pilot-1002 infra-fixed infra-ace-m1 --out D:/autostack/plan-artifacts/plan-1059/fixed-final
cargo t ace-host ace-batch --out D:/autostack/plan-artifacts/plan-1059/host-final
python scripts/tests/test_document_case_migration.py
```

- `auto-test-case`覆盖沿用的文档作者解码与比较合同；BASE认证97实际检查；pilot保4/7既有格式入口；`infra-fixed`检查固定组配置；`infra-ace-m1`检查冻结来源/适用核算。
- `ace-host`是修改 `host/language_conformance.rs` 的完整现有职责组，包含三个具名BASE入口和治理消费者；`ace-batch`验证保留的旧格式历史消费者不被作者入口切换破坏。沿现有tools/平台/ignore/资源配置，不创建筛掉慢例的新组。
- 未修改console/immutable文档或入口，既有1057 controls仍需完整通过；若T-01发现真实共享改动触及其消费者，具名补对应 `case-ace-m1-console-863` / `case-ace-m1-immutable-avm-920`，记录原因，不能仅按名称扩到全ACE。
- 若当前主线存在既知红项，逐完整测试名记录首次状态、对应债务和同基面对照，三个BASE宿主测试及本批28项必须通过，任何新增红须修复或明确阻塞；不把全组有红写成全绿。
- 单列列出数/实际执行数/条件排除数、准备和冷编译、发现、运行、清理与总时长。60秒为成本目标，不强杀、不减原样本/金样/阈值。本批不要求`ace-perf`或每次全仓回归；周期门由merge按真实due在主检出单实例判断。

`--out`为既有固定组入口；其实际内部短临时输出、copy清单和清理路径也登记到本计划scope并在merge对账。纯Python单测的临时输出按现有工具清理，留存摘要归scope。

## 7. 验收标准

| 状态 | AC | owner_stage | 可观察验收条件 | 验证 / 任务 |
|---|---|---|---|---|
| [x] | AC-01 | work | 本批精确14例/28项，逐ID映射无漏/重/额外项；BASE36/97；迁后24/68文档+12/29旧格式 | V-01/02、NC-05/06；T-01/02/03 |
| [x] | AC-02 | work | 源/金样与独立原hash及VM字符串相同；metadata/entry/path/options/比较语义完整；source与oracle内联、定位正确 | §5.2/5.3、V-01/02、NC-04/07～10/13；T-02/03/04 |
| [x] | AC-03 | work | 同runner旧/新BASE97项实跑通过；本批28实际结果逐项等原oracle，独立verify通过、记录绑定实际代码/工具 | V-03/04、NC-11/12/14；T-03/04 |
| [x] | AC-04 | work | 15个具名控制全部观察到规定结果，失效必须红到指定case/check/轴；原1057 controls无遗漏 | §6.2、V-04；T-03/04 |
| [x] | AC-05 | work | 对照与负控后才切active include；正常fixed/CLI/host消费新28项与原文档40项，BASE97；两个staging闭包齐全，既有宿主负控正确，pilot/batch兼容 | V-05、NC-14/15、§6.3；T-05 |
| [x] | AC-06 | work | 冻结旧suite/声明/源/金样/hash保留，实际消费者逐路径登记且原治理通过；active新文档是日常作者源 | V-06、§5.7；T-01/05 |
| [x] | AC-07 | review | 独立核对全部AC、28映射、15控制与Spec delta；遗漏/延后/workaround、改动告警与格式审查完成，结论绑定r1和冻结提交 | V-07；T-06 |
| [x] | AC-08 | merge | reviewed后SD-01/02落canonical并核内容/身份；冻结并集无新增红；归档、due判断、材料路径对账和guard后清理完成 | §5规范增量、§6.3；T-07 |

## 8. 执行步骤

work 阶段（2026-11-15）已完成 T-01～T-05：实施提交 `113649920`（文档+脚本+单测）与 `19e4b6c11`（入口切换+host/governance）；T-06 复审、T-07 merge 待后续阶段。

| 状态 | T / owner_stage | 前置 | 实际文件/符号及具体交付 | 验证与对应AC |
|---|---|---|---|---|
| [x] | T-01 / work | 用户后续执行指令；1002/1057已交付 | [✅ 已完成] worktree `D:/autostack/.wt/lang-1059/auto-lang`（branch `plan-1059-dev`，基面 `cd71736f8`）+ scope `D:/autostack/plan-artifacts/plan-1059/`（REGISTER.md 登记）。基面重绑：§4.3/§5.3 全部 32 份文件（4 接线 + 14 src + 14 oracle）长度/SHA-256 逐项一致（`ALL OK`）。消费者登记：`source-manifest.json`（pin legacy-composite.case.ad sha）、`ace-m1-expanded/{legacy-base-contract,legacy-required-map,legacy-a2r-contracts/legacy_base,legacy_base_probe}.json`（均含 composite 引用）、`ace_m1_legacy_base.py`（SUITE 指旧 ace-m1-base.suite.ad + CONTRACT_SHA pin）、`ace_m1_legacy_a2r.py`（suite 身份合同）、host 三具名 BASE 测试、`governance.rs` 两测试、`move_closures_1000.rs`（注释引用）。预切换 BASE97 投影导出 scope（`base97-inventory-pre-switch.json`：97=40 document+57 fences）。旧侧运行基线：pair old 侧 97 项实跑全 PASS（exit 0），无既存红。 | V-01/06，AC-01/02/06 |
| [x] | T-02 / work | T-01 | [✅ 已完成] 新增 `crates/auto-lang/test/cases/ace-m2/documents/composite.case.ad`（9974 bytes，sha256 `21712e37…`），由 scope 生成器 `gen_composite_doc.py` 从冻结原件程序化生成（载荷=原文件字节）。scope 候选 BASE：`candidate-base/inputs/cases`（581 文件闭包，closure.json 记录逐文件 hash，副本 include 已换 composite）。解码核对：standalone lint `1 case file(s), 14 case(s)`；28 项新旧语义投影深比较 **0 差异**；14 例 src/golden 字节、VM 文本（R02=9/R06=21/余42）、tags/links/path/entry/module_name 逐项 **0 mismatch**。 | V-01/02，AC-01/02 |
| [x] | T-03 / work | T-01/02 | [✅ 已完成] `document_case_migration.py`：新增 `composite` 家族（14 cases/28 pairs/BASE97 全集复用，无重复追加）；`rebind_actuals_from_files`（离线 verify 重读 VM result.txt/重算 Rust artifact 摘要，缓存与文件不一致→exit 2，重绑后对 expect 判决→exit 1）；comparison.json `representation_diff` 改按每行实际 `origin.format`；`stage_isolated_corpus` 加入 composite 文档（切换后 1057 controls 兼容）；新增 `controls --family composite`（NC-01～15 + candidate-shape，`stage_candidate_base` 候选闭包 + 97/68/28 预核）。`test_document_case_migration.py`：独立字面 14/28/97 清单、composite 97/28 verify、缺 V07 拒绝、NC-11 实换/缓存陈旧两变体、a2r artifact 删除/篡改——11 测试全绿（1.4s）。 | 单测、V-01/04、NC-05～12/15，AC-01～04 |
| [x] | T-04 / work | T-02/03 | [✅ 已完成] 同构建（runner sha 同一）对跑：`pair --family composite` exit0，verified_pairs=97、migrated_pairs=28，两侧 97 PASS（15.2s）；独立 `verify --family composite` exit0（重绑路径实跑）；`pair --family scalar` 候选兼容 exit0，97/40 且 new40=document（14.0s）；`controls --family composite`：**34 步全 ok**（candidate-shape + NC-01～15 全部观察到，57.0s），覆盖 R02=9/R06=21 实值、K06 双腿、T05 转义、NC-11 记录副本对换（文件已还原）、NC-12 缺 actual、NC-13 Fresh→单行 Stale(oracle)、NC-14 无 sidecar 28 全绿逐项 actual、NC-15 缺文件点名+恢复 97 绿。scope：pair-candidate/、pair-scalar-candidate/、controls-composite/。 | V-03/04候选部分、全部NC，AC-02～04 |
| [x] | T-05 / work | T-04通过 | [✅ 已完成] 提交 `19e4b6c11`：active `ace-document-base.suite.ad` 单 include 切换（legacy-composite→composite 文档）+ 头部说明 68+29；`test-groups.toml` 组标题更新；host `stage_doc_base_corpus` 增复制 composite；`ace_m1_base_suite_green` 增 28+40 逐 ID 来源/成员核验；`governance.rs::document_base_cli_list_and_migration_inventory_complete` 改独立身份分核 40(scalar×4)/28(composite×2)/29(fences)，总 document=68、97、零外部输入，list 增 k01/v07 在场断言。切换后核验：active CLI lint `6 case file(s), 36 case(s)`；post-switch 投影 97=68 document+29 fences（composite 28 全 document，文档来源文件=scalar+composite）；原 1057 controls 切换后 **35 步全 ok**（28.7s）；§6.3 并集1：`auto-test-case case-ace-m1-base case-ace-document-pilot-1002 infra-fixed infra-ace-m1` 全 PASS（8m56s）；并集2：`ace-batch` PASS，`ace-host` 164 测 161 绿 + 3 红（`a2r_hir_first_case_green`/`a2r_hir_worker_credential_pin`/`av_source_inventory_denominators_replayable`，主检出干净基面 cd71736f8 同 3 测同样 0 passed/3 failed——**同基面既知红，非本批引入**；av_source 系 examples/ui/015-notes 已随 33c3f60a3 移出仓库的 AP-16 扫描错误）；三个 BASE 宿主测试在并集内绿；`ace_m1_base_suite_green` 一次 5 测试并发定向跑出现瞬态红、单独重放 rc=0 绿（已知 worker 并发边界，环境归因）；`reuse_obligations_run_green_via_original_suites` 绿（旧 suite 原样可用）。Python 单测 11 OK（1.4s）。scope：base97-inventory-post-switch.json、controls-1057/、fixed-final/、host-final/、active-provenance.json、red-*.txt（红项证据）。 | V-04/05/06、NC-14/15，AC-01/04/05/06；既知红已具名+同基面对照 |
| [x] | T-06 / review | T-01～05完成 | 同实施树独立复核diff与全部原件/actual/控制；逐AC判决、遗漏/workaround与债务扫描；核Spec delta、当前revision/代码/tool证据是否匹配，输出可重放问题及修复关闭条件 | V-07，AC-07；不以计划打勾替代证据 |
| [x] | T-07 / merge | T-06 pass | 冻结受影响完整组并集并复用有效证据，按SD-01/02沉淀canonical，确认落地主线内容身份，归档计划；判断周期门due；摘要入计划，实际全部输出路径对账，整scope清理及guard后移除树/分支，更新保留登记 | AC-08；review之后完成，不计入work就绪前置 |

## 9. 复审记录

2026-11-14 起草交接：`stage: new`，`PLAN-1059 r1`，`outcome: pass`（规划可交接，不是实现或独立复审通过），`next: work`（待用户执行指令，T-01）。本轮完成实际旧声明与源/金样静态取材、14/28映射、四例完整文档范例、97全集与68/29切换口径、15控制及两个staging消费者核查。仅规划文件、中央取号与路线簿记有写入；未创建实施worktree、未迁移语料、未切入口、未运行产品/性能测试。

起草静态检查：14个唯一case ID与28条映射逐项对应原声明；28份原件长度/SHA逐项一致；四个完整示例的Auto/Rust围栏载荷与原文件逐字节一致；15个控制ID、0–10章节、7个任务、8个AC、仓内链接和1059编号唯一性检查通过。额外查明治理CLI测试硬编码document=40，以及离线verify派生值与actual文件的绑定缺口，分别纳入T-05和T-03。此检查只认证计划的来源与结构，不代表这些未来修复或迁移已完成。

实施、独立复审、冻结门禁、规范沉淀、归档及清理记录待对应阶段追加。所有任务与AC保持未完成。

### 2026-11-15 work 交接（T-01～T-05）

`stage: work | plan_id: PLAN-1059 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1059/auto-lang | branch: plan-1059-dev | base_commit: cd71736f8 | code_commits: 113649920（composite 文档 + migration 家族/重绑/NC 控制 + 单测）、19e4b6c11（active include 切换 + host/governance/组标题）| dependency_revisions: auto-down 兄弟 detached@5af11a3（未用）| task_ids: T-01..T-05 全部完成 | evidence: 见各任务行 | blockers: 无 | next_responsible_role: review（T-06）`

**Work-owned AC 结果**：AC-01～AC-06 全部达成（证据见 §7/§8 任务行）：14/28 逐 ID 无漏重、语义投影 0 差异、字节对原件 0 mismatch；同 runner 两侧 97 实跑 PASS、28 项真值逐项核对（R02=9/R06=21）；15 控制全观察到（34 步全 ok）；切换后 active 68+29、固定组/宿主/试点/batch/历史治理通过；冻结原件与消费者原样保留。开放发现：无（3 个 ace-host 红项均为主检出同基面复现的既知红，具名记录）。

**本会话测试模块/范围与时间**（准备=构建，发现/运行/清理分列；复用证据单独标注）：

| 测试模块/实际范围 | 结果与执行/排除数 | 准备/构建 | 发现 | 运行 | 清理 | 总计 |
|---|---|---|---|---|---|---|
| debug runner 构建（auto-case） | 成功 | ~3m（增量冷） | — | — | — | ~3m |
| release workers 构建（av+mn） | 成功 | 8m46s | — | — | — | 8m46s |
| python -I scripts/tests/test_document_case_migration.py | 11/11 OK | 0 | <0.1s | 1.4s | 0 | 1.5s |
| pair --family composite（两侧 97 项实跑） | exit0；97/28 | 复用已建产物 | <1s | 15.2s | 0 | 15.2s |
| verify --family composite（独立离线） | exit0 | 0 | <1s | <1s | 0 | <2s |
| pair --family scalar（候选兼容） | exit0；97/40 | 0 | <1s | 14.0s | 0 | 14.0s |
| controls --family composite | 34 步全 ok（NC-01~15+shape） | 0 | <1s | 57.0s | 0 | 57.0s |
| controls（1057 默认，切换后） | 35 步全 ok | 0 | <1s | 28.7s | 0 | 28.7s |
| cargo t 并集1（5 固定组） | 5 组全 PASS | 共享冷编译 | — | — | — | 8m56s |
| cargo t 并集2 | ace-batch PASS；ace-host 161/164（3 既知红） | 共享 | — | — | — | 12m3s |
| 3 红项主检出基面复现 | 3 failed（与 worktree 一致） | ~4.5m 重建 | — | 280.7s | — | ~5.7m |
| 5 测试定向 + base_green 单独重放 | 4 绿+1 瞬态红→单独重放 rc0 | 0 | — | 136.4s/131.0s | — | ~4.5m |

**新增用例行为与计时**：本批无新原生测试用例（28 项文档检查为既有义务的身份映射迁移；Rust 侧仅改既有测试的断言）；新增 6 个 Python 单测（合计 1.4s，单个 ≤0.5s，≤10s 门全部满足）；NC 控制为脚本控制步（34 步合计 57s，单步均值 ~1.7s）。机器占用等待（并行代理 979/972 等）合计约 20 分钟，已与执行时间分开列示。

**未变复用证据**：候选对照与切换后 active 内容/工具身份一致性见 scope `active-provenance.json`（97 项语义一致，suite 字节差异仅头部说明与 include 顺序；active 消费由切换后 1057 controls + case-ace-m1-base 组在真实 suite 上证明，不依赖候选复用）。

`stage: goal | plan_id: PLAN-1059 | plan_revision: r1 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1059/auto-lang | commit: 19e4b6c116ab2d7429c03d56d5f7ac79a9a74af7 | next: review`


### 2026-11-15 review（T-06，final）

`stage: review | review_scope: final | reviewer_identity/limitations: 与实施同会话同上下文（无独立reviewer会话可用）；判决从工件重建——diff 全量重读、scope 证据独立重验（verify 重跑 rc0、单测重跑 11/11、comparison/controls/inventory 工件复核），不采信执行摘要 | plan_id: PLAN-1059 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1059/auto-lang | branch: plan-1059-dev | reviewed_commit: df51c71a5 | base_commit: cd71736f8 | dependency_revisions: auto-down detached@5af11a3 未用 | spec_inputs: document-cases.md（before=首批30/60、BASE 10/40+57）、ace-m1-conformance.md（before=BASE 36/97=40+57）与 SD-01/02 逐项比对一致 | acceptance_results: AC-01~06 全 pass（证据见 §7/§8/§9 work 回执及本节重验）；AC-07 由本节闭合 | pending_closeout_gates: merge 冻结受影响组并集复用与落地检查（AC-08）| findings: 无阻塞发现 | evidence: 见下 | next: merge`

**复审发现与处置**：
1. （已修复）格式健康检查：`governance.rs` 基线 rustfmt 全绿而新增块漂移、`language_conformance.rs` 新增 for-行超宽——回 work 同树纯格式修复 `df51c71a5`（governance.rs 恢复全绿；language_conformance.rs 仅剩 2 处基线既有 PLAN-889 漂移块，未扩散）；修复后 `document_base_cli_list_and_migration_inventory_complete` 0.11s 绿、`ace_m1_base_suite_green` 4.81s 绿重放。
2. （环境归因记录）`ace_m1_base_suite_green` 曾在 5 测试并发定向跑出现一次瞬态红（131s），完整并集与单独重放均绿——符合已知 worker 并发边界，不判代码失败。
3. （既知红确认）ace-host 3 红项在主检出干净基面 cd71736f8 复现同样失败（0 passed/3 failed），且已在 KNOWN-DEBT-AND-RISKS.md 登记（1060-R3：a2r/a2x golden 漂移，含 a2r_hir_first_case_green/a2r_hir_worker_credential_pin；1060-R7：examples/ui/015-notes 陈旧路径）。非本批引入、非本批范围，无新债务条目。
4. （workaround 扫描）diff 无 TODO/FIXME/debug 残留；冻结 ace-m1 资产零改动；NC-12 以"删文件+备份恢复"实现（可观察行为=计划要求的缺 actual→exit2 点名，证据文件已还原）；NC-11 副本变异后双向还原。无未授权延后或范围缩减。
5. （测试纪律）无新原生用例；新增 6 Python 单测单个 ≤0.5s；测试不触发前置构建（governance 用 AUTO_LANG_CASE_BIN 预建产物，host 用进程内 run_suite；workers 为显式准备阶段构建）。

**AC→证据映射复核**（重建自工件）：AC-01=base97-inventory-{pre,post}-switch.json（97=40+57→68+29）+独立字面清单单测；AC-02=28 项语义投影 0 差异 + 字节/元数据 0 mismatch + NC-04/07/08/09/10/13；AC-03=pair-candidate（97/28 双侧 PASS，同 runner/工具 hash 记录）+verify 重跑 rc0（含 rebind 路径）+NC-11/12/14；AC-04=controls-composite 34 步全 ok + controls-1057 35 步全 ok；AC-05=固定组并集+governance/host 分身份断言+两宿主反控（各 1F+96P 内断言）绿；AC-06=ace-m1 树 diff 零改动+infra-ace-m1/reuse_obligations 绿+T-01 消费者登记。

**并集证据时效**：§6.3 两并集绿于 `19e4b6c11`；其后仅 `df51c71a5` 纯格式提交（空白变更，编译验证+两关键测试重放绿）——语义输入未变，并集证据对 `df51c71a5` 仍有效，merge 复用并按 AC-08 收尾。


`stage: goal | plan_id: PLAN-1059 | plan_revision: r1 | round: 3 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1059/auto-lang | commit: df51c71a5 | next: merge`

### 2026-11-15 merge（T-07）

`stage: merge | plan_id: PLAN-1059 | plan_revision: r1 | outcome: pass | reviewed_commit: df51c71a5 | delivery_commit: 8136dc3353d9b50acef3c7e09053a3c990be2849 | branch: plan-1059-dev→master | checkpoints: prepared✓ landed✓ canonical_verified✓ archived✓ cleaned✓`

- **prepared**：SD-01/SD-02 在 worktree 内应用于 `docs/specs/auto-test-case/document-cases.md`（两批 44/88、BASE 24/68+4 旧 29、composite 行/标题合同/对照家族与离线 verify 重绑规则、冻结原件清单增 legacy-composite）与 `ace-m1-conformance.md` 底座套件节（68+29 组成 + 复合例仍为旧 AutoVM/a2r 快照的边界）；纯文档提交 `41c0873ca`（df51c71a5 的唯一后继，实现/依赖零改动）。
- **rebase**：master 前移至 `4c6757104`（979 簿记），dev 干净 rebase，`git range-diff` 四提交全 `=`（补丁等价证明）；映射 113649920→234134025、19e4b6c11→3dc3496f0、df51c71a5→41375a35d、41c0873ca→8136dc335。
- **landed**：`wt-guard.ps1` clean 后主检出 `git merge --ff-only plan-1059-dev`，master tip = `8136dc335`，无 merge commit，线性历史。
- **canonical_verified**（主检出落地内容 sha256）：`document-cases.md` 8848B/`0c84e4998b1f5f45918c3fdc568cad0f6f64a605f9559cf2270ad1d2e2faf55d`；`ace-m1-conformance.md` 40081B/`a3a576b794921a7cf7f85b80a0da529894d9607ac01b0b3f22bb49256348496b`；`composite.case.ad` 9974B/`21712e37c60dbbebd85a457d3480c26029ed774296e708eba546142f71bb8667`；`ace-document-base.suite.ad` 605B/`ba0fead94cb16c9e7dd9c69171d8d835d366b40b222d42726f21cb4e85e59131`。
- **冻结组并集复用**（merge 不重跑的理由）：两并集绿于 `19e4b6c11`，其后仅格式提交（rustfmt，空白变更，编译+两关键测试重放绿）与 spec 文档提交；实现代码/依赖/配置/输入在 `8136dc335` 与验证时点逐字相同（range-diff 全等）→ 证据按"输入未变复用"规则对交付提交有效。Python 单测最后一次全绿运行于 41375a35d 树（11/11，1.6s）。
- **archived**：计划移入 `docs/plans/archive/1059-autodown-composite-cases-migration.md`、`status: archived`、本回执随归档提交入库（`docs(plan): archive 1059 …`）。
- **cleaned**：worktree `D:/autostack/.wt/lang-1059/auto-lang` 守卫 clean 后移除、分支删除、组目录（含 auto-down 兄弟）移除；`D:/autostack/plan-artifacts/plan-1059/` 整目录删除（证据已折入本文件；无具名未解决问题持有），`.retention/policy.json` 登记。
- **merge 会话测试**：无新增产品测试执行（复用冻结证据如上）；新增用例数 0；本次会话未触发 release 构建。占用检查：落地前后无并行 cargo 任务。
- **周期门判断（DUE）**：1059 % 5 = 4（非 5 倍数），但 `.last-batch-regression.json` 覆盖止于 1057（2026-11-14T14:54Z，距今 ~7.8h < 48h），窗口 eb76560bf..master 内已落地 **Plan 1060（5 倍数）** → 触发批量回归 due；merge 收尾后按 auto-plan-regress 在主检出单实例执行 `cargo t all`。
- **落地观察（非部署）**：本批触面为测试语料/脚本/一致性测试，无发行二进制或部署产物；主检出 `target/debug/auto-case.exe`（01:39 构建）早于落地，属构建中间产物，随下次主检出测试自建刷新，不构成生产陈旧。

`stage: goal | plan_id: PLAN-1059 | plan_revision: r1 | round: 4 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed) | commit: 8136dc3353d9b50acef3c7e09053a3c990be2849 | next: done`

## 10. 待澄清事项

当前没有阻碍规划的格式或依赖问题。用户授权限规划，实施待后续指令。T-01须重新确认实际基面、现有完整组首次状态及冻结消费者；若真实代码要求改变14/28范围、检查语义或原件保留承诺，应先修订合同并列明差异，不能静默缩减验收。

### 2026-11-15 批量回归收尾补记（merge 步骤 6）

DUE 确认后执行两轮 `cargo t all`（主检出单实例）：第 1 轮 32P/3NA/1F（auto-lang 组 HTTP e2e plan724/727 红＝确定性 TEMP e2e 工程缓存损坏——`rustversion` out 文件缺失，并发会话共路竞态；清理陈旧 TEMP 目录后隔离重放 2/2 绿）且运行中被并行 979 簿记提交打断（drift）→ 不作回执；第 2 轮（covered `b1dbab313`，wall 614.5s）31P/3NA/2F：`auto-generation-core::plan988…` TIMEOUT＝已知债务 D-1014-P988-BUDGET（与前回执同形）；`auto-tool-execution` 2 例计时敏感测试仅批内并发负载下红、同 HEAD 隔离重放 2/2 绿（0.53s/5.05s）、窗口无该 crate 源码改动→环境归因非回归。回执 `last_covered_plan_id=1060` 已提交（`99520a184`）。观察项（非本门范围）：plan724/727 e2e 泄漏确定性 TEMP 目录（数十个 plan727-e2e-* 残留），归 http e2e 卫生 owner。
