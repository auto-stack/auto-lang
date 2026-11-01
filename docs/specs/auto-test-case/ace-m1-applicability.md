# ACE M1 适用性登记（ace-m1-applicability）

> 状态：delivered（PLAN-908，2026-11-01；G-DEC-1..5 用户裁定后的 tracked 化）。
> 载体：`crates/auto-lang/test/cases/ace-m1/applicability.json`（schema
> `ace.m1.applicability/1`，随库版本化）+ `binding.tsv` 逐格 token。
> 验证入口：`powershell -File scripts/verify-ace-m1-852.ps1 -Scope contract|acceptance`；
> 治理反控固定组 `cargo t infra-ace-m1`（变更/白名单/来源/核算控制）。

## 1. 权威来源与反滥用

- **唯一批准来源**：`docs/plans/reports/ace-m1-scope-decision-2026-10-28.md` §13
  （2026-11-01 补充裁定 G-DEC-1..5）；
  `applicability.json.approval_source` 冻结该文档 SHA256，contract scope 逐次核对，
  漂移即拒（批准过期≠默许）。既有 NA-1..NA-4/R-A..R-F（2026-10-28 裁定）直接继承。
- **反滥用铁律**：每个 `na:m1(NA-x)` 格必须被登记规则认领（无来源 NA 拒）；裸
  `n/a` 不可再出现；规则格不得双claim；未知行/路径/字段拒；`migrated` 格 token 必须
  恰为 `na:m1(<rule>)`；**分层总数重算**（rows/cells/core/committed/NA/future/
  ruled-accepted/required）必须等于 `expected_totals`——按绿缩分母或静默扩白名单
  都会在 contract/acceptance 直接红。
- **规则不可自行扩大**：新增适用范围须先记录差异并修订合同（决策件或同等裁定），
  不得类推 NA；`R-ACCEPTED`（已裁定差异）永不等于 ok，也不移除证据 token。

## 2. 分层与判据（G-DEC-5 双层验收）

| 层 | 定义 | 格数 |
|---|---|---:|
| core | 原 103 行 ×4（除扩展行组 U01-U10/H01-H03/O01-O15 外全部行） | 412 |
| core NA | NA-1/NA-2/NA-3(旧)/NA-5/NA-6-I/NA-3-I 认领 | 95 |
| core ruled-accepted | R-ACCEPTED（R-A/B/C/D 旧引擎差异，含 B17/B19 avm `diff:accepted(R-C)`、K02/K05 a2r legacy-grammar；**B18 avm 除外**——infra 观察仍阻核心） | 32 |
| **core required** | 412 − 95 − 32 | **285** |
| extension committed | U/H/O 行的 av 格（已承诺 profile） | 28 |
| extension NA | U 非AV 30（NA-4）+ H 非AV 9（NA-6-H/NA-3-H） | 39 |
| future（未承诺） | O01-O15 非 AV（ac/avm/a2r） | 45 |
| **required 合计** | 285 + 28 | **313** |

524 = 313 required + 95 core-NA + 32 ruled-accepted + 39 extension-NA + 45 future。
required 集合 hash 由 acceptance scope 计算（`ace.m1.acceptance/2`）。

**签章条件**：`can_claim_m1` = core required 全绿（ok* 或 a2r 金样轴族）且扩展已承诺
28 格全绿，同时报告展示全部剩余 gap（含 future 45）。签章≠"131 行四路径全支持"；
future 45 保 gap/owner/重入条件，不改 NA/PASS。base97、局部选择、只 NA、零执行、
SKIP、伪父报告均不授信 full（acceptance 拒假报告，选择不改判决）。

## 3. 规则登记（2026-11-01 冻结）

| 规则 | 认领格 | 来源 | 重入条件（要点） |
|---|---|---|---|
| NA-1 | W01-W05 avm/a2r（10） | 继承 2026-10-28 | legacy/a2r typed 入口冻结计划 |
| NA-2 | D01-D09 avm/a2r（18） | 继承 2026-10-28 | SEM-07 或 legacy Dyn 计划 |
| NA-3 | K01-K06 ac（6）+ D01-D09 ac（9）继承；**H01-H03 ac（3）新增**；I07/I08 ac（2）继承澄清（G-DEC-4） | 继承+G-DEC-4 | native 闭包/Dyn ABI、常驻 loader、在线热重载/暂停恢复能力包冻结接口 |
| NA-4 | U01-U10 ac/avm/a2r（30，token 已是 na:m1(NA-4)） | 继承 2026-10-28 | native/legacy 冻结 entry-profile 支持 |
| NA-5 | C01/C02 avm/a2r（4）+ G01-G08 ac/av/avm（24，含 G01 ac/av 两个 raw gap 的专属主体排除） | **G-DEC-1 新增** | 后继正式消费相同能力/新 producer 先冻结接口/规则；普通执行/代码生成旧义务不免 |
| NA-6 | I01-I11 avm/a2r（22，G-DEC-1）+ H01-H03 avm/a2r（6，G-DEC-4） | **G-DEC-1/4 新增** | 正式接同场景接口或 consumer 迁移；旧 VM 缓存/旧 a2r 快照义务仍验 |
| R-ACCEPTED | avm 全部 diff* 格除 B18 外（30）+ K02/K05 a2r（2） | 继承 R-A/B/C/D 组裁定 | 语义裁定推翻差异时改回 gap:<owner> |
| FUTURE-O | O01-O15 ac/avm/a2r（45） | **G-DEC-5** | console 产品/M2 承诺；永不计入本次 M1-required |

批准映射去重：61 唯一迁移格 = 54 原裸 n/a + 7 原 raw gap；2 旧批准澄清
（I07/I08 ac）+ 59 本次新增批准。G01 的 `gap:av-822` 陈旧 owner 随裁定废止
（822 实际已归档 r5；903 F-REG-4"从未立项"为文字错误，本档勘误）。

## 4. binding.tsv 迁移与守恒（T-05）

- 仅路径 token 与 note 回填：61 格 → `na:m1(NA-3/5/6)`，每变在 note 保
  `908T05 <path>: <old>→<new> (G-DEC-n §13)`；另 corpus T08 a2r 的非标
  `source-bound` 按其注记事实归一为 `gap:a2r-golden`（五态词表，903 F-REG）。
- **不可变列**：id/ns/feature/outcome/expected/status/enabling 及未涉及格一字不动；
  原 131 行（91 corpus + 40 scenario）×12 列守恒由 contract 与
  `cargo t infra-ace-m1` 变异反控（漏行/截列/重复/未裁新增行必红）锁定。
- 后续合法新增行须显式更新 applicability 来源/profile/scope，不静默忽略；
  也不以 131 作永久常数拦正常新增（新增须同步修订 expected_totals 与裁定记录）。

## 5. 与其他登记的关系

- 逐格 raw token 与 owner 始终以 binding.tsv 为准；本档是**适用范围/分层**的权威，
  不是能力账本——净缺口/owner/关闭 oracle 见
  [ace-m1-gap-audit.md](ace-m1-gap-audit.md)（903 冻结数保留，908 后继状态在其
  PLAN-908 节）。
- 40 scenario 行输入/独立 oracle/反例/资源增量见
  [ace-m1-scenario-oracles.md](ace-m1-scenario-oracles.md)；来源身份（src/oracle/
  reference/suite/证书/scenario oracle 文档的 SHA256 清单）由
  `source-manifest.json`（schema `ace.m1.source-manifest/1`）冻结，fresh 树
  contract 不依赖任何 attachments。
- a2r 双轴与诊断负例面见 [a2r-golden-certification.md](a2r-golden-certification.md)；
  M1 判据=金样/emit 轴，认证轴独立账。
