# ACE M1 四路径一致性语料（ace-m1）

> 状态：delivered（PLAN-852 底座；本规范描述已交付事实，解码器/脚本即合同）。
> 位置：`crates/auto-lang/test/cases/ace-m1/`；验证入口
> `powershell -File scripts/verify-ace-m1-852.ps1 -Scope <contract|source|snapshot|certify|lifecycle|acceptance>`。

## 同源四路径

一份 Auto 源码（`src/*.at`，byte-identity 由 binding.tsv 的 cases.json hash 轴冻结）经四条真实路径独立判决，父侧判定，worker 不接收 expected：

| 路径 | recipe/adapter | 判决面 |
|---|---|---|
| AC 共享 MIR native | `ac/mir-native-scalar-v1`（probe/mn） | 完整 i32 exit（含符号；trap=exit 70 为 818 冻结实验合同） |
| AV ABC | `ac-av/core-run-v1`（probe/av） | 完整 i32 exit + 类型化 trap 码（`av.trap.*`） |
| 旧 AutoVM | `avm/run` | legacy-display 完整十进制 result + 类型化 stage/code |
| a2r | `a2r/emit` | 真实产物逐字节比较冻结金样（`oracle/m1_*.rs`）；行为核验见独立认证规范；日常快照含严格收据验证（类型化精确键集 schema）与认证面进程观察（周期性 CIM 祖先链采样——是采样观察器而非无损创建计数；直接 rustc/m1_* 二进制 0 观测；引擎 worker 的 memoized 维护构建属 818 纪律、单独计数报告，不算认证面） |

宿主包装只允许结果编码（认证 wrapper 计入身份），不改算法/不删分支/不用四份不同源码。

## 分母登记（binding.tsv，schema ace.m1.binding/1，12 列冻结）

63 corpus 行（58 源例 + 5 typed 入口）+ 40 scenario 行 = M1 required 分母的登记处。
逐行给出四路径单元状态：

- `ok` / `ok-emit`：能力已交付且已正式接线（base 套件真绿）。
- `diff-*`：引擎真实执行但与冻结 oracle 语义分歧（如 B08/B11 旧引擎 10/-7 vs
  oracle 7；B13/B15 旧 i64 不陷阱；B16 a2r 不做名解析）——如实登记，永不冒绿；
  改已议语义须携反例交用户裁定。
- `gap:<owner>` / `wait:D82x`：未交付能力（native 扩面、822~824 类型族、native
  在线 loader 等），带 owner/关闭 oracle；不阻塞底座，但阻止 M1 全绿。
- `snapshot-only-cert-gap`：a2r 复合产物为伪 Rust（`List<int>`/`int`），快照可比
  较、不可编译认证。

selector/局部 run 不削减 required 分母；零选择、缺行、仅 emit、SKIP=PASS 均不能
作为 M1 绿证据。

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

D01–D09 的 `av-dyn-824` 行绑定 `ac-av/dynamic-run-v1` 和真实 typed MIR/ABC/AV、portable 恢复及父侧 oracle；W01–W05 保留整数和浮点精确位型验证，I04–I06/I10–I12 追加动态会话、失效与资源证据。源码、复合 native/CTE 及旧 AVM/a2r 的 Dyn API 未由 typed fixture 关闭，仍按各自 owner/gap 记录。

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
| K06 | 共享可变捕获环境 | 编译期以 `source.closure-shared-mutable` 拒绝；C-19 仍为 `gap:R5-O`。 |

源码 AV 绑定关闭 K02/K05 的 `823-future-source` 缺口，但不修改 legacy AVM/a2r 的 `diff-legacy-grammar(parse)` 单元格。case suite 独立核对结果或诊断码，并检查闭包 session 的 live 值、分配/释放计数；SKIP 不计覆盖。

V-06 记录 20 个样本：P90 355.1596 ms、最大峰值 1,015,808 bytes、hot set 4,483 bytes，符合现行 P90 ≤ 2000 ms 门和继承资源预算。fresh/restart/body-edit 明细留在 PLAN-865 review receipt；本地测量源及其 hash 继续仅保存在 Plan 收据，不复制进规范。

## PLAN-870：T*/R* 行 ac-native 实测绑定

- 新增独立 suite `ace-m1-tr-native.suite.ad`（17 case/31 检查；**852 base
  suite 不扩**）：值/trap 行携带 ac-native+av 双腿对同一冻结 oracle；诊断
  行携带 mn 腿（家族码），av 腿仅限 862 已绑 ok 的行。T05 无腿
  （`gap:upstream-s-eq` 具名排除）。
- binding.tsv ac 列判级与 av 列逻辑逐行镜像：可执行值/trap/负诊断行=ok
  （实测出口码注记）；T03=`gap:mn-text-move`、T05=`gap:upstream-s-eq`、
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
