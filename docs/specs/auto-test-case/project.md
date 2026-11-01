# auto-test-case

> PLAN-896测试调度：固定Cargo包+Rust模块组取代tm动态目录，入口cargo t <组>、显式cargo t all。
> 本crate的TestCase协议/oracle/准入比较保持；ACE宿主迁至独立experimental/ace workspace的ace-conformance包。PLAN-901测试feature拍平：本crate为auto-lang/auto-lang-core常驻依赖（原test-cases optional门退役），三工具bin恒真实形态；语言侧ACE桥接测试常驻编译、仅显式ace组选择（cargo t ace为常规集合，ace-performance保正式计量原合同，all含完整包条件集合）。普通语言日常/具名组/静态list不选择ACE桥接，语言all不准备/构建ACE worker。

> **Status**: active
> 路径：`crates/auto-test-case` | 技术栈：Rust（仅依赖 vendored `autodown-test-parser`；auto-atom/auto-val 仅 dev）

TestCase 协议核心（`autolang.case/1` / `autolang.suite/1` / `autolang.test-report/1`）：
把"一份 source 工程登记多个独立后端检查"做成严格合同——发现、校验、选择、比较、
报告都在本 crate，后端执行 adapter 留在 auto-lang（依赖方向只有 auto-lang → 本 crate）。

## 目标与范围

- 严格合同模型：未知字段/坏类型/非法组合=定位错误，绝不解析后忽略。
- 原始 bytes 保真：载荷与 oracle 按字节比较（显式 `lf` 仅 CRLF→LF）。
- 身份绑定：suite/case 文件/源内容/oracle 内容（含 sidecar 字节）/合同文本 +
  toolchain（vendored parser 指纹、crate 版本、生效限额）逐轴入报告；旧报告经
  `stale` 语义核查，任何轴漂移即 STALE。
- 只读纪律：输出与输入/oracle 根不相交（canonical 双向比较 + 祖先 reparse 审计）。
- 非目标边界随 B 阶段（PLAN-788）收缩：native（ac core/scalar、Windows
  x86_64-pc-windows-msvc）、contained 进程隔离/deadline/输出 cap、783 统一
  cache、syntax 观测已按 v2 交付（[profile-v2-native.md](profile-v2-native.md)，
  worker 在 `experimental/ace/crates/case-worker`）；仍未导入：text/record/text-record
  profile、XFAIL、语言内非 ASCII 输出（随 text 批）。

## 模块清单

### PLAN-865：源码闭包绑定

ACE M1 闭包源码套件以 append-only 文件方式消费未改动的 K01–K06 .at 输入。ac-av/closures-source-run-v1 显式选择源码闭包 profile 并进入既有 AV 闭包 session；父侧独立检查值、编译期诊断、缓存生命周期和资源释放。852 base suite、typed closures-run-v1 与 legacy AVM/a2r grammar-difference 状态保持原义，见 [ace-m1-conformance.md](ace-m1-conformance.md)。

### PLAN-852 ace-m1 语料与认证（正式资产，非本 crate）

- 语料/分母登记：`crates/auto-lang/test/cases/ace-m1/`（binding.tsv 当前 131 行 =
  91 corpus[58 源例 + 5 typed + U10 入口 + H3 常驻 + O15 console] + 40 scenario，
  四路径 524 登记格；base 套件 36 例 97 检查；金样 `oracle/m1_*.rs` 24 + 单源
  wrapper `oracle/wrapper-main-v1.rs` + 证书 registry `oracle/certs/*.cert.json`
  10，schema `ace.m1.cert/1` 随库版本化）。登记总量≠M1 required 分母：适用分层以
  [ace-m1-applicability.md](ace-m1-applicability.md)（PLAN-908 `applicability.json`，
  required 313 = 核心 285 + 扩展已承诺 AV 28）为准，净缺口/owner 以
  [ace-m1-gap-audit.md](ace-m1-gap-audit.md)（PLAN-903 冻结 + §11 908 后继）为准；
  来源身份由 `source-manifest.json` 冻结，40 场景 oracle 正文见
  [ace-m1-scenario-oracles.md](ace-m1-scenario-oracles.md)。
- 验证入口：固定组 `cargo t case-ace-m1-*`（12 组，含 908 近邻
  `case-ace-m1-nb-diags-908`）+ 常驻 cargo 负控为日常绿证据入口；
  `scripts/verify-ace-m1-852.ps1` 六+一 scope（PLAN-908 修复：.NET Split、去
  attachments 依赖、contract 严格校验 applicability/manifest、acceptance 零执行 strict
  核算 `ace.m1.acceptance/2`）为可运行的 M1 完整性门，治理反控组 `infra-ace-m1`；
  认证 bin `ace-certify`（certify 恒显式）。合同与差异行语义见
  [ace-m1-conformance.md](ace-m1-conformance.md) /
  [a2r-golden-certification.md](a2r-golden-certification.md)。


| 模块 | 职责 | 状态 |
|---|---|---|
| document / raw_ranges | pinned 块树 ↔ 原始 fence 字节区间双向校验 | active |
| atom / contract | 静态 Atom 合同解析 + 严格 schema（含 per-adapter 观测支持表） | active |
| admission | 资源限额（有界读）+ 路径域/reparse 审查 | active |
| ir | payload/oracle 解析为 owned bytes；oracle 身份=逐字节内容哈希 | active |
| suite / discovery | suite 编排、显式 defaults 深合并、逐文件发现（错误保留不静默丢） | active |
| compare / report | 观测比较（bytes/lf/text/message）、逐 check 报告与 gate 退出码 | active |
| sha256 | 极简指纹（无外部依赖） | active |

## 支持表

**v1 冻结（avm/a2r）**：

| 通道 | avm/run | a2r/emit |
|---|---|---|
| completion（success / error{stage,code?,message?}） | ✓ | ✓ |
| result（legacy-display） | ✓ | — |
| stdout / stderr | ✓（错误完成下不可声明） | — |
| artifacts（多产物逐项 bytes） | — | ✓ |

- error completion 必须带稳定辨识（code 和/或 message exact/contains）；空 code、
  空 matcher、裸 stage 一律 admission ERROR（无辨识力的强 oracle被拒）。
- 声明本 adapter 观测不到的通道 = admission ERROR，不是忽略。

**v2 native/syntax（PLAN-788，B 阶段）**：ac×{check,build,run}（core/scalar，
typed stages/exit/原始 bytes 流/预算硬 cap/部分产物边界/783 cache 三模式/逐行
身份轴）与 syntax×{parse,edit}（两 grammar、三投影、fresh↔incremental 等价），
语料 42 case + legacy-map 对账；见 [profile-v2-native.md](profile-v2-native.md)。

> 来源：docs/plans/archive 待归档的 773-autodown-testcase-protocol-pilot.md（A 阶段）、
> docs/plans/788-autodown-ac-native-turnaround.md（B 阶段）；
> 合同细则见 [profile-v1.md](profile-v1.md) / [profile-v2-native.md](profile-v2-native.md)，
> 结构与依赖约束见 [architecture.md](architecture.md)。

## 治理交付（PLAN-887，2026-10-29）

- 状态校正：runner 退役通道现加载 844+869 双清单（此前仅 844）；language 完整档 floor =
  3189（3205 − 869 退役 16，逐 ID 理由落档）；语料三族（vm_file_tests/cookbook/conformance）经
  tv 档白名单枚举，聚合包装不冒充子 case 绿。
- 受管治理工具：scripts/test-governance/（inventory/check_inventory/measure-first-wave/
  coverage-matrix）+ scripts/tests/test_test_governance.py；正式 integration
  plan887_test_governance（显式 --test 接线，不可被裸 --lib 漏掉）。
- 剩余缺口（具名）：B17-B19 pending 语义裁定（language 语义 owner）、ui-tn 手工 pending/
  Windows 限制（不扩豁免）、D 组 hit/edit/restart 生命周期扩展（不在首批）、	f --batch
  完整批次由 merge 验收（候选 runner）。

PLAN-863 的 [console byte 观察轴](architecture.md) 由父侧独立判断 stdout/stderr/exit 或 Unit/trap；O01–O15 绑定真实 AV console 源码，其他后端保留具名 gap，见 [conformance](ace-m1-conformance.md)。

## PLAN-889：显式集合native消费

原864合法集合/Variant源码经共享MIR→native机器链，单独recipe/worker、真实拥有值清理、
source函数候选/坐标与深验恢复；正式suite40检查使原14L/V native源码格关闭，其他引擎格和W/在线帧gap保持。
见[native合同](../auto-mir/native-collections-variants.md)；旧五域/recipe/cert键集/legacy退役不变。

## PLAN-888：独立测试模块与显式批量试点（历史；已被 896/898 取代）

测试义务曾按唯一叶归属，以 `cargo tm <模块ID>...` 执行完整模块和消费者闭包的去重并集
（tm 引擎与叶目录已随 PLAN-898 删除）；现行入口为 `cargo t <固定组>`（PLAN-896），
旧包含档别名亦已删除，不再逐档叠跑。先列模块/用例数量，再各模块一次墙钟、累计CPU及
原生子树峰值观察；异常只定向重验。特别长原子交owner决定，未测/原条件/原红保持可见。
目录归属不等于日常准入；用途四类审查与日常包准入建议已由 PLAN-895 交付（建议层，默认执行不变）。见[用途与准入](test-purpose-and-admission.md)与[模块选测](module-selection.md)。
ACE批量B及D3组合C保持显式实验，公开pilot默认A；历史候选数值不能证明当前候选收益。
原父侧oracle、采样/资源门和单项重放保持，见[批量试点](batch-execution.md)。

## PLAN-895 全仓用途审查与顶级模块

交付建议层（不改变执行）：66 个职责顶级模块（lang/core/ace/case/infra 五域）重组 1181 叶、
7165 条义务逐条审查行与四类用途建议（日常 48 模块/专项 17/调研 1）、全仓来源对账台账（21 行分轴）
与日常包准入建议清单；13 项 ACE 慢原子与 4 项已知旧红具名交接。详见
[用途与准入](test-purpose-and-admission.md)、[模块选测](module-selection.md) 顶级模块节；建议未经
用户批准不改变默认选择。


## 已交付范围与限制（PLAN-893）

- 已交付：13项慢测当前原生身份冻结（真实nextest发现）、K-01/02迁名证据、四轴接入约定活例
  （source=case-av-source-827、typed=case-ace-m1-typed-native-891、资源=case-ac-av、892 emit
  日常=a2r组，rustc腿按设计ignore）、13项当前单轮成本归因（全部PASS；正文69–1074s、单核
  CPU密集、峰值102–428MB；单轮不冒正式P90）。T-02记录两次未归因环境干扰红（WER
  LiveKernelEvent相邻、重放绿）按“原红/干扰/重放均留”保留。
- 限制/未做：全M1净缺口审计归PLAN-903；产品算法/存储/ABI修复不预占号（具名后继：K-08 publish
  分区计数核实、K-07 native产物生成吞吐、K-09流式构建单核）；不缩512/4096/32–64MiB样本、
  不改原触发/预算/分母；无逐测试新鲜度/影响图/定时任务。

### PLAN-911：native 源码诊断事实通道与 B17–19 ac 实证

- `mn.diagnostics: "source-v1"`（scalar recipe 专属）与父侧严格准入/正式事实 artifact：
  [profile-v2-native.md](profile-v2-native.md) 第 "显式源码诊断观察模式" 节；通道合同见
  [../auto-ac/native-source-diagnostics.md](../auto-ac/native-source-diagnostics.md)。
- 固定组 `case-ace-m1-native-diags-911`；`b-diags.case.ad` 的 B17/B18/B19 ac 腿已迁移严格模式（真实
  raw code），binding.tsv 三格 `diff:pending → ok`（[ace-m1-conformance.md](ace-m1-conformance.md) /
  [ace-m1-gap-audit.md](ace-m1-gap-audit.md) 的 911 节）。
- 登记红控（`case-control-oracles.json`）：错族 / 错 stage / 旧泛化壳三个 expected-FAIL 控制。
