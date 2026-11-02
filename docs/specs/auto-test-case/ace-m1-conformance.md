# ACE M1 四路径一致性语料（ace-m1）

> 状态：delivered（PLAN-852 底座；本规范描述已交付事实，解码器/脚本即合同）。
> 位置：`crates/auto-lang/test/cases/ace-m1/`；验证入口
> `powershell -File scripts/verify-ace-m1-852.ps1 -Scope <contract|source|snapshot|certify|lifecycle|acceptance>`。

## 同源四路径

一份 Auto 源码（`src/*.at`，byte-identity 由 tracked `source-manifest.json`（schema
`ace.m1.source-manifest/1`，PLAN-908）冻结；旧 attachments/852/cases.json 依赖已退出）经四条真实路径独立判决，父侧判定，worker 不接收 expected：

| 路径 | recipe/adapter | 判决面 |
|---|---|---|
| AC 共享 MIR native | `ac/mir-native-scalar-v1`（probe/mn） | 完整 i32 exit（含符号；trap=exit 70 为 818 冻结实验合同） |
| AV ABC | `ac-av/core-run-v1`（probe/av） | 完整 i32 exit + 类型化 trap 码（`av.trap.*`） |
| 旧 AutoVM | `avm/run` | legacy-display 完整十进制 result + 类型化 stage/code |
| a2r | `a2r/emit` | 真实产物逐字节比较冻结金样（`oracle/m1_*.rs`）；行为核验见独立认证规范；日常快照含严格收据验证（类型化精确键集 schema）与认证面进程观察（周期性 CIM 祖先链采样——是采样观察器而非无损创建计数；直接 rustc/m1_* 二进制 0 观测；引擎 worker 的 memoized 维护构建属 818 纪律、单独计数报告，不算认证面） |

宿主包装只允许结果编码（认证 wrapper 计入身份），不改算法/不删分支/不用四份不同源码。

## 分母登记（binding.tsv，schema ace.m1.binding/1，12 列冻结）

当前 131 数据行 = **91 corpus**（852 原 58 源例 + 5 typed 入口 W01–W05；+U01–U10
入口/Unit[862]、+H01–H03 常驻[866]、+O01–O15 console 源码[863，ns=corpus]）+
**40 scenario** 行 = 登记处；四路径（ac/av/avm/a2r）共 524 登记格。**登记总量 ≠
M1 required 分母**：适用范围按已批准规则（NA-1..NA-6、R-A..R-F、FUTURE-O）逐格
归账于 [适用性登记](ace-m1-applicability.md)（PLAN-908，`applicability.json`）：
**required = 313**（核心 285 + 扩展已承诺 AV 28），核心 NA 95、已裁定差异 32、扩展
NA 39、future 45；净缺口/owner/顺序见 [净缺口审计](ace-m1-gap-audit.md)（PLAN-903
冻结数 + §11 908 后继状态）。逐行给出四路径单元状态（五态词表，862 SD-01/02）：

- `ok` / `ok:<producer>` / `ok-emit` / `ok-parse-reject`：能力已交付且已正式接线。
- `emit-ok-run-unverified` / `snapshot-only-cert-gap` / `cert-diff`：a2r 金样轴绿、
  认证轴缺（ok+cert-gap 等价；M1 a2r 判据=金样轴，认证另账）。
- `diff:accepted(R-x)` 与 `diff-legacy-*`/`diff-old-*`（R-ACCEPTED 认领）：引擎真实
  执行但与冻结 oracle 语义分歧且**已裁定接受**——如实登记、永不冒绿、不阻 M1 核心
  （B18 avm 例外：infra 观察，`diff:pending` 仍阻）。`diff:pending`/`diff-no-name-check`
  （R-E 不接受）为未裁定/真缺口，阻塞。
- `gap:<owner>` / `wait:D82x`：未交付能力，带 owner/关闭 oracle；不阻塞底座，但阻止
  M1 全绿（除非被 NA/FUTURE 规则认领出 required）。
- `na:m1(NA-x)`：M1 不要求，**必须**被 applicability 规则认领（裸 `n/a` 自 908 起不
  存在，contract 直接拒）。

selector/局部 run 不削减 required 分母；零选择、缺行、仅 emit、SKIP=PASS、伪父报告均
不能作为 M1 绿证据（acceptance scope 拒）。

### 净缺口审计导航与待验收边界（PLAN-903，2026-11-01 冻结；**908 后继见文末**）

- ~~五态词表/`na` 反滥用/两轴分离已裁定（2026-10-28），但 binding.tsv 尚未迁移~~
  （PLAN-908 T-05 已迁移：61 格 G-DEC 批准回填 + T08 a2r 归一；raw 前态保留在 note 与
  `applicability.json.old_token`）；~~bare `n/a` 与 B17–19 ac 诊断面、H/I07/I08 的
  NA-3 扩展均待裁定~~（G-DEC-1..5 于 2026-11-01 全部裁定，见
  [适用性登记](ace-m1-applicability.md)）。862 T-06 的 20/73 实测口径仅为历史记录。
- a2r 双轴：M1 判据=金样/emit 逐字节轴；`emit-ok-run-unverified`/
  `snapshot-only-cert-gap`/`cert-diff` 为 ok+cert-gap 等价，认证缺口另账
  （owner=a2r 复合 lowering）。892/902/904 新 HIR A2R 证据**并列**，不自动替换
  旧 a2r 身份或旧 M1 分母。
- 日常绿证据入口=固定组 `cargo t case-ace-m1-*`（12 组，含 908 的
  `case-ace-m1-nb-diags-908`）+ 常驻 cargo 测试；`verify-ace-m1-852.ps1`
  ~~现 infra 红~~（PLAN-908 T-06 已修：.NET Split、去附件依赖、strict
  acceptance/2）——contract/acceptance 恢复为可运行的 M1 完整性门。
- 逐格七类归账、缺口族 owner/关闭 oracle/后继顺序以
  [ace-m1-gap-audit.md](ace-m1-gap-audit.md) 为 current 账本（903 冻结数 + §11）。

## 底座套件（ace-m1-base.suite.ad）

只装证据绿检查：36 例 97 检查（T-02 标量三路径、T-03 金样快照、T-05 会话生命
周期、类型化负例）。负例 oracle（改错期望必红）与金样漂移（必红）以 cargo 测试
`crates/auto-lang/src/tests/ace_m1_tests.rs` 常驻。完整分母（含 skip 行）由
acceptance scope 从 binding.tsv 报告，两轴结论分开：底座 PASS ≠ M1 全绿。

## 会话生命周期（T-05 事实）

同一冻结源（B09）上：av warm/hot/body 编辑（fact(5)→fact(6)=720）/跨进程编辑
持久（内容寻址 store，cache_hit_abc 观测）；mn fresh/edit+重启（新 worker 进程，
store 跨进程，cache_hit_coff）。native 在线 HMR（I08）不由本规范声明——需要

常驻/在线行（PLAN-866，H01-H03）：`ac-av/resident-run-v1` recipe 以真实 worker
进程+父侧判定执行常驻链（fuel 切片、调用边界安全点换版、在途留旧、延迟安装
守卫）；H01 连续运行=2100、H02 在线换版=2608（46旧×7+254新×9，手算审计）、
H03 双换版 churn=3070；wrong-oracle 负控判红。852 binding I08 的 AV 列由此
关闭；I08 native 列（gap:ac-native-loader）与 I07 语义保持不变。
native loader 能力包，重启 exe 不满足。

## 下游绑定（823~826 T-11）

下游仅在本目录**追加** binding/证据行；改 binding schema、金样、通用合同或
SDK adapter 须拿 852 交付后的测试窗口（关联核心合同另拿核心窗口）。消费时记录
输入 profile/wire/semantic 版本与脚本/证书身份。

## PLAN-824：动态 typed 能力绑定

D01–D09 的 `av-dyn-824` 行绑定 `ac-av/dynamic-run-v1` 和真实 typed MIR/ABC/AV、portable 恢复及父侧 oracle；W01–W05 保留整数和浮点精确位型验证，I04–I06/I10–I12 追加动态会话、失效与资源证据。PLAN-891：W01–W05 经 `ac/mir-native-typed-v1` 真实 native 交付（binding.tsv ac 列 `ok:P891-typed`，suite `typed-native-891`）——mn 侧与 AV 侧各自独立 oracle，canonical (kind,bits) 逐位一致；typed 能力不冒领 source/全 M1。源码、复合 native/CTE 及旧 AVM/a2r 的 Dyn API 未由 typed fixture 关闭，仍按各自 owner/gap 记录。

## PLAN-823：闭包 typed 能力绑定

K01–K06 与 scenario R05 的 `av-closures-823` 行绑定 `ac-av/closures-run-v1`
（AVAD v5/profile 6）真实 typed MIR→ABC→AV、代码缓存恢复与父侧 oracle；
I01/I02/I04–I06/I10/I12 追加闭包会话身份、env 漂移、跨工程身份、512 修订
churn 与 judge 反控证据（`-Scope baseline` 常驻，stale-evidence 身份控制钉
41 文件 hash+依赖 HEAD）。源码闭包语法（两 legacy 路径 parse 拒）、a2r 伪 Rust
复合认证与 native 闭包 ABI 仍按各自 owner/gap 记录，不因 typed 行关闭。

## PLAN-825：标量 Loaded 能力绑定

B01/B04/B09/B13/B14 五源（852 `src` 与 825 fixture SHA-256 逐位同源）绑定 `ac-av/loaded-run-v1` 与 loaded/reference 双路径独立 oracle：优化路径索引 build/PC/参数计数非零、reference 扫描/拷贝为零，B13/B14 绑定 overflow/div-by-zero trap。I01/I02/I05–I07/I10–I12 追加 30 修订结构+P95、两函数编辑身份、内容命中零重建、暂停/恢复同 pinned LoadedProgram、512 修订+GC 资源与 judge 反控证据（`-Scope baseline` 常驻）。源码四路径/native/M1 缺口按各自 owner 保留，typed/loaded 行不冒领同源 source 能力；绑定与 raw 见计划 attachments/825/852-validation（提交 97cecc89d）。

## PLAN-826：标量紧凑 wire 能力绑定

- 852 底座分母追加本计划实际支持起点：binding 表（`docs/plans/archive/826-av-compact-abc-format-and-size.md` 的 attachments/826/852-validation/binding.md）逐行映射 B01/B04/B09/B13/B14 双 wire 对拍与 I01/04–07/10–12 场景；`verify-av-826.ps1 -Scope baseline` 为正式入口。
- 语义 ID → 真实起点 → V 子例 → 正式入口 → 独立 oracle → 证据逐行可核；optimized==reference 双路径冻结期望。
- 范围外 gap 保留 owner：复合类型/字段布局源码编辑、source/native/AJ 面不冒领；typed fixture 不替代同源四路径。

## PLAN-862：入口/Unit 语料与 M1 重绑定

- **追加行（不改既有行语义）**：U01–U10（`crates/auto-lang/test/cases/ace-m1/entry-862/`，独立手算 oracle）经真实 `ac-av/core-run-v1` + `av: { profile: "entry-unit" }`（append-only 选择器）执行。判定轴新增 `av_channel: value|unit`：Unit 无 i32 载荷，只有通道名能把它与 `i32 0` 分开，因此 `unit` 与 `exit` 同格声明被拒（含 exit 0）。样例：U01/U03 值 42、U02/U04 Unit、U05 trap、U06–U10 五类入口诊断（missing/params/unit-return-value/entry-tail/entry-duplicate）。
- **五态词表（决策件 SD-01/02 落地）**：`ok*`（含 `ok-emit`/`ok-parse-reject`/`ok:…`）/ `ok+cert-gap`（a2r 金样轴绿、认证轴缺）/ `gap:<owner>`（应有未交付；owner=计划 ID 或具名能力包）/ `na:m1(NA-x)`（M1 不要求，须带规则 ID 与重新进入条件）/ `diff:accepted(R-x)`（已裁定旧侧差异，**不等于** ok）/ `diff:pending`（已执行有分歧、未裁定，阻塞）。`probe` 与 `wait:*` 不得作为长期状态。
- **`na` 规则与反滥用**：NA-1 legacy/a2r 不判 typed fixture 入口；NA-2 无 Dyn API；NA-3 native 不判闭包/Dyn/在线重载/暂停恢复；**NA-4**（2026-10-28 用户裁定）入口/Unit 是 AV-only profile，native/legacy 不判（重新进入条件：有计划冻结 native/legacy 对该 profile 的支持时逐行改回 `gap:<plan>`）。每条 `na` 必须带规则 ID；新增须回决策件或同等决策；merge 报告单列 `na` 行数与规则分布。
- **本轮新增 owner token**：`a2r-golden`（发射成功但无冻结金样）、`scenario-tooling`（无源码的 scenario 行观测通道未交付）、`av-text-move`/`av-text-nul`（owned 源面对 Str move / NUL 比较的缺口）、`av-mut-param`（AV mut 参数未交付）、`P864`（集合/变体源码，PLAN-864）。

### PLAN-864 交付后的 L/V 重绑定（2026-10-29）

L01–L07 / V01–V07 的 av 列已由 `gap:av-unsupported-list` / `gap:P864` 重绑为 `ok:P864-source`
（`binding.tsv` 仅动 status/owner 列，12 列 schema 与 852 分母值列未动）：真实源码经
`ac-av/collections-source-run-v1`（profile 4，指令面零改动）独立判定，typed fixture 通过不冒领源码行。
L06 保持 trap 行（`av.trap.list-index`，SEM-02 冻结；legacy 差异为 profile 差异不冒绿）。
追加变体 N01（List<Str>=51）/N02（for-in copy=42）/N03（for-in owned 投影=53）只存在于新 case 文件
（`collections-source-864.suite.ad`，17 行），不扩缩 852 基套件分母。ac 列仍 `gap:mn-scalar-only`（owner=870 线）。
- **实测口径**：24 `probe` + 27 `wait:*` 单元格逐格真实执行后定级（残留 0/0）；corpus 73 行（原 63 + U01–U10），实测 **clean = 20/73**；`probe`/`wait:*`/SKIP 均不计绿，`clean` 以脚本复算为准。决策件附录中无规则 ID 的 `na:m1` 行（C01/C02/I01–I11/G02–G08）在全表回填前仍需单独裁定。
- 门：`powershell -File scripts/verify-av-862.ps1 -Scope <targeted|compat|entry|acceptance>`；旧 852 底座套件保持 97 检查绿（本计划未改 852 脚本、金样与 12 列 schema）。


## PLAN-865：源码闭包绑定

新增 `closures-source-865.case.ad` 与独立 suite，recipe 为 `ac-av/closures-source-run-v1`。它将未修改的 K01–K06 `.at` 原文件绑定到显式 `auto.source.closures.draft` profile 和既有 823 闭包 session。852 base suite 与 typed `ac-av/closures-run-v1` 的语义不变。

| Case | 源码行为 | AV source 绑定 |
|---|---|---|
| K01 | Move lambda 的 Copy 捕获返回 42 | fresh 与 restart session 执行；另检查 body-edit 生命周期。 |
| K02 | 拥有型 `Str` 捕获经返回闭包逃逸并返回 42 | 真实经源码编译、lowering、闭包链接和 AV 调用执行。 |
| K03 | 同步非逃逸 `Str` 借用返回 42 | 实际执行；父侧检查 session 释放余额。 |
| K04 | 返回借用捕获闭包 | 编译期以 `source.closure-borrow-escape` 拒绝。 |
| K05 | 显式 clone 的拥有型闭包返回 42 | 实际执行；clone 独立性反例返回 11，若环境别名会返回 12。 |
| K06 | 共享可变捕获环境 | **PLAN-909（2026-11-02）**：原源文件真实执行返回 42（move 闭包私有可写 env，cell=单元素 `List<T>` 句柄）；AV check 由 resolve 期拒绝迁移为 success/42。非标量共享写/let 写的拒绝面移至 `shared-mutable-909` suite（S07/S08）；C-19 残余（共享 owned/循环/跨挂起）仍具名后继。 |

源码 AV 绑定关闭 K02/K05 的 `823-future-source` 缺口；PLAN-909 追加 `shared-mutable-909` suite（S01–S06 正例 fresh/body-edit/restart + S07/S08 不支持域负控，`case-ace-m1-shared-mutable-909` 组）。两者均不修改 legacy AVM/a2r 的 `diff-legacy-grammar(parse)` 单元格。case suite 独立核对结果或诊断码，并检查闭包 session 的 live 值、分配/释放计数；SKIP 不计覆盖。

V-06 记录 20 个样本：P90 355.1596 ms、最大峰值 1,015,808 bytes、hot set 4,483 bytes，符合现行 P90 ≤ 2000 ms 门和继承资源预算。fresh/restart/body-edit 明细留在 PLAN-865 review receipt；本地测量源及其 hash 继续仅保存在 Plan 收据，不复制进规范。

PLAN-912 追加 `closures-fn-912` suite（`case-ace-m1-closures-fn-912` 组，recipe
`ac-av/closures-fn-run-v1`）：F01/F02（双兄弟 lambda + 自由函数，fresh/restart/
body-edit/hit 四态）与 K01/K06/S02 原语料的同字节副本经**单函数会话**重跑。父侧
`judge_closures_fn` 独立判值（51/48/61 等）并判决 zero 轴：body-edit 必须走 fast
路由且 warm gate pass、whole 轴 0、submitted 严格小于全量；hit 全零；restart 全命中
且 `mir_lower==N` 如实上报（R3 开放不隐藏）+ 驻留池 ≤16MiB。原 865/909 suite 与
oracle 不变；binding/applicability/manifest 零改动（纯新增子树）。

## PLAN-870：T*/R* 行 ac-native 实测绑定

- 新增独立 suite `ace-m1-tr-native.suite.ad`（17 case/31 检查；**852 base
  suite 不扩**）：值/trap 行携带 ac-native+av 双腿对同一冻结 oracle；诊断
  行携带 mn 腿（家族码），av 腿仅限 862 已绑 ok 的行。~~T05 无腿
  （`gap:upstream-s-eq` 具名排除）~~（PLAN-894 闭合：文本 eq/ne 走
  StrCmp，T03/T05 均有双腿）。
- binding.tsv ac 列判级与 av 列逻辑逐行镜像：可执行值/trap/负诊断行=ok
  （实测出口码注记）；~~T03=`gap:mn-text-move`、T05=`gap:upstream-s-eq`~~
  （PLAN-894 起均 `ok`：T03 经 owned-Str 局部替换=23、T05 经 StrCmp=42，
  ac-native+av 双腿实测；独立 oracle 套件 `text-source-894.suite.ad`；
  ace.m1.t03 行由旧诊断期望迁 value=23，保 check 身份与历史依据）、
  R07=`gap:mn-mut-param`、R08=`gap:mn-lower-method`（共享栈拒绝，与 av
  gap 同因）。
- mn 诊断的 case 级期望面：probe 合法 stage 集（parse/resolve/typecheck/
  run）内的映射——前端拒绝=parse+`mn.unsupported.frontend`，adapter/MIR
  拒绝=typecheck+`mn.unsupported.adapter`（管线边界由 mn.* 码命名）；
  诊断行**具体码**由 mn 内 crate 差分层对同栈断言（强 oracle）。
- V-06 反控：期望变异必红（T01 38→39 负 oracle 测试）。

## 方法行关闭（PLAN-881）

- R08 ac/av 列翻 ok:P881（method-881 suite + t-r-native 值行：mn+av worker
  真进程 = 42）；R07 顺带关闭（av.data/1 mut 写回 ABI，42 实测；legacy diff
  与 a2r 列不动）。
- 新 method-881 suite（9 值行×mn+av 双腿：42/0/57/75/135/42/52/84/42 +
  4 诊断行）挂自身 suite 文件（852 base 冻结不扩）；V-08 反控：M03 oracle
  75→76 变异必红且仅红 m03。分母/a2r 轴不改。

## PLAN-887 首批消费与新增来源（governance-887）

- 原 required 分母/词表/oracle 不变；887 不重生成、不改 binding/expected/证书。
- 30 复用义务（B01..B20 + U01..U10）经原交付 suite（ce-m1-base.suite.ad、
  ntry-862/entry-unit-862.suite.ad）原样判定，零复制、身份保持；B08/B11 旧侧差异保留
  （oracle 7 不追随旧 VM 10/-7）。
- 3 新增 D 组消费（crates/auto-lang/test/cases/governance-887/，id gov.d.*）：原 vm corpus
  表达式（m/01_basics/{002_arithmetic,006_unary,007_group}/*.at，pilot ID
  lang.basic.arithmetic/unary/group）以**入口包装**消费——裸表达式被正式入口拒绝
  （diag source.syntax），故原表达式字节逐字嵌入 n main() int { return <expr> }，case 头记录
  原路径 + 原 sha（=pilot legacy_source_sha256）+ wrapper sha + 嵌入行 + 独立 oracle（3/-6/9）。
  不冒称原字节四路径或裸脚本支持；原 vm 源零改动。
- 适用起点：entry-unit 为 NA-4 AV-only profile（native/legacy 不要求判 U 行与 D 行）。

### PLAN-863: O01–O15 console source rows

`console-863/console-863.suite.ad` is an independent 15-row source suite under the existing 12-column binding schema. Hand-computed byte oracles cover int MIN/negative/zero/multi-line, bool, UTF-8/newline, char, owned readonly borrowing, nested guest evaluation, main/script int/Unit, output-before-trap, compile-time rejection, revoked/missing grant and empty text. Parent API tests additionally distinguish embedded NUL without changing the Atom contract escape grammar.

AV bindings are `ok:av-host-console-source-863` only after real parent-side execution. Native is `gap:ac-console-profile` (explicit profile rejection); legacy VM/a2r are `gap:console-profile-parity` (this new profile's exact parity is unverified). These are explicit remaining coverage gaps, not new `na:m1` rules or whole-M1 certification. No prior expected outcome, required denominator or certificate is rewritten. `verify-av-863.ps1` provides targeted/compat/console/acceptance scopes; sabotage of either byte oracle or exit must fail, missing axes reject and SKIP remains SKIP.

来源：[PLAN-863](../../plans/archive/863-av-console-host-import-and-print.md)。

## PLAN-888 成组试点边界

首批B保留原十个可用M1 AV标量绑定，另B08/B11以原绑定来源作独立AV消费，加原D3共15例；这不把B08/B11原其他路径diff标成绿。重复invocation、运行group和C的三个字段都不增加M1覆盖分母。原源码/期望与四路径差异/gap保持，单case可独立重放。C仅原D3 owned-record观察，不替代四路径或原单文件义务。见[批量试点](batch-execution.md)。


## PLAN-889：14个集合/变体native源码格

`native-collections-889.suite.ad`包含原17源native、原17源AV及3个owned List<Variant>/
嵌套/Option补源各两引擎，共40检查。原源/oracle/分母不变；只将L01..L07/V01..V07共14个
native格改为`ok:P889-source`及其来源注记，其余111行和其他引擎格不变。
L04/L05=42，L03=0，V03=3；L06必须真实list-index原因21+exit70，普通return70不是该trap。
Str变体/for-in补例值仍51/42/53。

显式recipe `ac/mir-native-collections-v1`使用独立`mn-collections-worker`，沿既有contained
transport和父侧独立oracle；旧scalar recipe/protocol/default、ace.m1.cert/1键集和a2r证书选择不变。
新报告的recipe/lowering/worker实际SHA/runtime ABI+build+content/lld+SDK+kernel32九轴由集合族
current查询重算stale；query不冒语言verdict。私有源/库镜像承担实际同长bitflip→stale→恢复Fresh控制，
不能改变共享库干扰并行用例。真实漏free/whole-front/lower/copy-unchanged、错oracle/type负控都须红。

[native合同](../auto-mir/native-collections-variants.md)区分source/typed/runtime起点、清理/物理峰、
真单体/恢复和正式五×30/512性能证据。独立修复采样只覆盖其标明门，复用须有revision/input/机制等价证据，
不得改源/阈值/分母或把失败full重新标绿。W01..W05、native在线帧/loader及其他旧引擎差异仍分别登记，
不因14格推进而宣称全M1或891 typed入口已完成。

## 新 A2R-HIR 证据（PLAN-892，并列不迁移）

`a2r-hir-892` 套件（B01/B04/B13/B14/NAMED-01）经新 recipe
`ac-a2r/hir-scalar-emit-v1` 产生**并列**证据：本文件的 M1 分母、expected、
旧 a2r-snapshot 检查与 NA/accepted-diff 行**不变**；新后端能力不冒认旧路径
交付，M1 a2r 消费者是否迁移由专门审计认领（892 不宣称 M1 或旧 A2X 替换完成）。

## PLAN-908：规则、登记与验收入口归一（2026-11-01）

- **适用登记**：`applicability.json`（`ace.m1.applicability/1`）冻结 G-DEC-1..5 批准的
  逐格规则（NA-1..6 / R-ACCEPTED / FUTURE-O）、批准来源 SHA、重入条件与分层总数；
  规范见 [ace-m1-applicability.md](ace-m1-applicability.md)。61 格批准迁移
  （54 裸 n/a + 7 raw gap → `na:m1(NA-3/5/6)`）与 T08 a2r 五态归一只动路径 token
  与 note，id/ns/feature/outcome/expected/status 守恒；binding 前态
  `d9f05da0…` → 后态见 source-manifest `binding.sha256`。
- **来源清单**：`source-manifest.json`（`ace.m1.source-manifest/1`）冻结 81 源
  （58 原 + 881×12 + 864×3 + 889×4 + 866×1 + 908 近邻 3）、24 金样 + wrapper、10 证书、
  43 reference、66 suite/case/子树源与 [40 场景 oracle 规范](ace-m1-scenario-oracles.md)
  及决策件的 SHA256；fresh 树 contract 不依赖任何 attachments，缺/漂移/伪造 D 源拒。
- **统一入口**：`verify-ace-m1-852.ps1` contract = 12 列 .NET Split + ns 唯一 +
  131/91/40 守恒 + applicability/manifest 严格校验 + NA 反滥用 + 基线/owner；
  acceptance = 零执行 strict 核算，报告 `ace.m1.acceptance/2`（binding/规则/清单/批准
  身份、registered/core/extension/future/ruled-accepted 账、required 集合 hash、逐格
  ledger、base/core/extension/future/cert verdict、`can_claim_m1`），可 `-ParentReport`
  准入真实父报告（空/失败/伪报告拒）、`-Select` 仅报告选择、`-BuildIdentity` 零 spawn
  严格验证证书；source/snapshot/lifecycle/certify/receipts 职责不变，certify 仍显式。
- **诊断近邻**：`nb-diags-908.suite.ad`（NB17/NB18/NB19 = B17/B18/B19 的合法近邻，
  2/2/0）在 ac-native/av/autovm 三引擎实跑 9/9 绿——证明 B17–B19 的拒绝是区分性拒绝；
  base 套件 36 例 97 检查冻结不扩。固定组：`case-ace-m1-nb-diags-908`（真实运行）、
  `infra-ace-m1`（治理反控：登记/白名单/来源/核算变异必红，all=false）。
- **T-04 实证（2026-11-01）**：base97 全绿（B16–19 ac/av 腿当前身份）；B18 avm 重探针
  仍 infra（可辨识消息、无 typed stage/code）保 `diff:pending`；B17 a2r 探针 emit 成功
  → 六诊断负例 a2r 面（G-DEC-3）保 `gap:a2r-golden`。当前 acceptance：core 247/285
  红、extension 28/28 绿、future 45 显示、cert 38 gap、`can_claim_m1=false`——治理交付
  完成 ≠ M1 通过。

## PLAN-911：B17–B19 ac 格位实证（2026-11-02，SD-04）

- 908 冻结计数（core 247/285、38 阻塞、extension 28/28、future 45、cert 38 gap、`can_claim_m1=false`）
  **保持为历史收据**；本节记录 911 之后三格的当前状态，不预言总数。
- `b-diags.case.ad` 的 `ace.m1.b17/b18/b19` × `ac-native` 腿迁移为 `mn.diagnostics: "source-v1"`，expected
  由 `parse/mn.unsupported.frontend` 改为真实 raw 面：`typecheck/source.type-mismatch`、
  `typecheck/verify.immutable-write`、`typecheck/verify.no-return-path`（raw stage `verify` → `typecheck`）；
  av 腿、id、source、其他 94 检查不变，base 套件 97 检查迁移后 PASS。
- binding.tsv 第 38–40 行 ac 列 `diff:pending → ok`（任务列 `T-02/911`，备注记事实 artifact 证据）；
  其余 9 列/128 行不动；`source-manifest.json` 同步（`plan_911_update`）。回填依据 = 正式父报告
  `mn-source-diagnostic.json`（`execution_key` 绑定、raw verify、profile scalar、worker mnworker/1.0）。
- 近邻 NB17/18/19 在严格模式亦 2/2/0 并 echo 能力；默认 908 近邻套件 9/9 不变。
- **M1 不据此签章**：K06（909）、旧 a2r 认证、B18 avm infra、scenario 等账保持。
