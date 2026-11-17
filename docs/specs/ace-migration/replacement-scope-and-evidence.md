# ace-migration: replacement scope and evidence

> Status: proposed (PLAN-922 SD-01/SD-02)。本文件只规范 M2 替换盘点的**口径与证据边界**，
> 不把任何条目标为 implemented；实现状态以各模块 Specs 与已交付计划收据为准。
> 来源：[PLAN-922](../../plans/archive/922-ace-m2-replacement-inventory-and-critical-path.md)（merge 时按实际路径修正）、
> [M2 合同 r2](../../plans/reports/ace-replacement-first-m2-contract-2026-11-02.md)、
> [v0.6 发布门增补](../../plans/reports/ace-v06-m2-release-gate-2026-11-02.md)。

## SD-01 全场景分母、分类与证据边界

### 分母构成（迁移台账的强制行集）

M2 替换分母 = 当前真实可达的：

1. **语言公共入口**：auto-lang facade 全部执行/转译 pub API、CLI 子命令与 runner 分派、
   REPL/持久会话/daemon、测试工具入口；
2. **领域消费者**：AutoUI（全部已支持模式/后端）、AutoMan（全部命令/求值/生成）、
   auto-web（服务宿主链）、AutoGen/SDK/模板、其余仓与安装物中真实调用旧引擎者；
3. **全部在用 A2X 目标**：Rust/C/Python/JavaScript/TypeScript/GDScript/Godot/TSCN 与
   r2a 逆向工具，按真实生产/测试/研究归属区分；
4. **发行/安装面**：默认与 all-features 构建、注册/feature/隐藏开关/fallback、
   包内二进制、安装/重定位/升级/回滚路径。

规则：以"入口 → producer → consumer"双向可追溯为准入；grep 零命中、目录名、
examples 计数均不构成覆盖证明。跨仓不可读/owner 未明/未发布安装物按具名 unknown
登记，不填"支持/无差额"。

### 状态分类（每行必选其一，禁混合百分比）

| 类别 | 判据 |
|---|---|
| implemented-verified | 新 ACE producer 存在且同一来源身份下有已执行独立 oracle 证据 |
| implemented-unwired | producer 已实现但该入口/consumer 仍走旧引擎（接线缺口） |
| evidence-stale | 声称已迁但证据非当前祖先/来源身份不适用/未重验 |
| capability-missing | 新引擎无该能力（须给最小真实工程切片与独立 oracle） |
| unknown | 未盘点/不可达/依赖未发布（须给 owner 与下一步只读核查） |

### 证据边界

- 运行证据必须绑定 repo/SHA/入口身份；历史 M1 收据对 M2 只作输入不作完成证明。
- "审计完整" ≠ "产品通过"：本口径下的任何登记都不替代 REP/V06 门的实际执行。
- 旧 required 静态格、M1 corpus（binding.tsv）、新 C/M2 门分账保持，不得互相冲销。

## SD-02 新旧义务、中立库与 disable/安装证据归属

- **义务迁移**：旧执行/转译义务经版本化 successor ledger/recipe/cert 迁移到新
  producer；原 source/expected/批准 NA/accepted 不改。旧 golden/证书不冒充新 producer
  身份；disable 后新版日常不为对拍重建旧引擎，追溯靠 Git 历史、独立已发布旧版本与
  行为 oracle。
- **中立库归属**：共享值类型/诊断/词法与文本工具/中立 runtime/native 静态库/渲染器/
  仍需的 Rust 库按真实依赖保留或迁中立 owner；判定依据是"是否夹带旧执行循环或旧
  AST 发射器"，不是目录名含 vm/trans。薄兼容 API 只能转发 ACE，不得包装旧算法。
- **disable/安装证据**：旧实现退出以 default+all-features 依赖图/构建单元/真实
  dispatch/发行包内容/日常测试选择为零为证；安装证据要求 clean 安装/仓外/重定位/
  升级/独立旧版回滚的实际收据，dev worktree、未发布绝对路径、本机旧缓存不算 fresh。
- **owner 交接**：跨仓迁移单写窗由对应仓 owner 以正式合同认领；盘点行只登记接口
  变动与需重核 consumer（带 SHA），快照不复制成新的永恒状态源，后续正式 SHA 到来
  只刷新受影响行。

## SD-03 分母台账与后继责任挂接（PLAN-959 SD-02）

- SD-01 的分母构成/五类状态/证据边界由 [m2-denominator.md](m2-denominator.md)
  落为**现行分母台账**：L（语言语法逐项）/S（标准库域）/K（发行安装冻结副本）新表 +
  922 A–G 行刷新引用；每行带来源身份（repo@rev + file:line）、四类语言分账
  （规范已定/真实支持/已知旧bug差异/未决，另设"无此形"防凭空新增）与行级后继归属。
  本节不改变 SD-01/SD-02 的任何分类与 REP/V06 门。
- **行级后继责任**：每行 required 与 unknown 必须指向正式后继 Plan/批次（945–948、
  960–987、auto-os/064）及其 T-01 决策门；"计划文本未逐名"的项（平台条件、TaskDef、
  Hash 各形态、vscode 模式等）由台账行显式补齐归属，缺行或无 owner 即盘点未完成，
  unknown 仍阻 M2 终签。
- **delta 规程**：台账冻结带源身份；新增/变更按其 §10 以 delta 行追加并使受影响旧
  证据陈旧（只重核受影响行），不重写已冻结行、不复制第二份永恒状态源；台账自身
  绝不声明实现，状态提升只引用后继计划的交付收据。

## SD-04 完整模块包 producer 挂接（PLAN-960 SD-02）

- 分母台账 L-37（use/dep）、L-38（别名/重导出解析）、L-41（全局 store/常量初始化次序/once/
  diamond/失败回滚）与 L-44（模块面）的 ACE 侧**已验证 producer** 为
  [m2-modules.md](m2-modules.md)（能力 `ac.source.modules.packages/v1`，TextRecord 域，
  四端冷运行+恢复收据见 `av::conformance::module_packages_960`）。本节只挂接来源，不改 SD-01
  的分类口径，也不把任何行标为 implemented；状态提升仍只引用 960 归档收据。
- **未闭缺口（具名，带 owner）**：934 事实页类型集不覆盖 text/record（D-960-1，query 层）；
  mn 端全局 cell（D-960-2，ace-native）；a2x 端全局 cell 发射与类型方法片段（A2X 后继）；
  L-42 跨模块泛型实例归 963；L-38 的 `alias`/type alias 语句形归 963；模块+CT 组合归 966
  （**已交付 A 片**：项位/语句位 CT、宏与目标条件，证据见
  [m2-metaprogramming.md](m2-metaprogramming.md)；仍未闭：读取集型装配身份、arch/env/ABI 显式事实来源）。
  以上缺口在台账中保持 required/unknown 原状，不因 960 交付而消失。

## SD-05 数值面 producer 挂接（PLAN-961 SD-02）

- 分母台账 L-01/L-02/L-49（数值类型/字面量/运算转换）的 ACE 侧**已验证 producer** 为
  [m2-numbers.md](m2-numbers.md)：`SourceProfile::Numeric` 显式源码 profile（grammar numeric
  裁剪门控，旧 profile 字节恒等），同型运算纪律、checked 窄化/浮转整、IEEE 浮（÷±0 trap 沿
  legacy、% ±0=NaN 不 trap 的具名不对称）、移位界 trap 的 T-01 冻结矩阵在四端一致落地——AV
  （eval_num/eval_conv）、native（CLIF→COFF→PE）、a2x-rust（真实 rustc）、a2x-c（真实 MSVC
  /W4 /WX），全部真实 dispatch 无 legacy fallback；823 对 closures/collections 族的冻结排除由
  m2-numbers §MIR/wire 取代。本节只挂接来源，不改 SD-01 分类口径，也不把台账行标为
  implemented；状态提升仍只引用 961 归档收据。
- **具名差异（对 legacy）**：混合宽度隐式晋升矩阵不采纳（显式 as/to 替代）；legacy `as`
  截断/clamp 改 checked；位运算/移位为新增冻结面（legacy 无此面）。**未闭缺口（具名，带
  owner）**：semantics 查询通道数值注解具名拒绝 `numeric-annotation`（query 后继）；CTE 值面
  维持 I32/Bool、comptime admission 未扩（具名；966 已在其 packages-ct 前门验证 i32/bool 谓词域与
  具名拒，本行状态提升仍只认 961 收据）。聚合/泛型/动态/控制效应组合面归 962–966。
  以上缺口在台账中保持 required/unknown 原状，不因 961 交付而消失。

## SD-06 组合数据结构面 producer 挂接（PLAN-962 SD-02）

- 分母台账 L-06..L-19/L-48（view/mut/move、Hold、Box、take、R 行、enum/union/tag、grid、tuple、
  array/List 字面量、Option/Result、模式、range、索引 place）的 ACE 侧**已验证 producer** 为
  [m2-aggregates.md](m2-aggregates.md)：`SourceProfile::Aggregates` 显式源码 profile（numeric 全表面
  超集，冻结面字节恒等），准入键=文档级 capability `av.aggregates/1`（mir_verify/native/a2x 同键放宽，
  旧面 822 一层元素域逐字节保持）；tuple=结构内联+匿名位置 record（零 schema/MIR 新指令）、Box=单元素
  List 胞+不透明边递归合法化、grid=列元组+静态维检查、整元素读=PLACE 视图（物化全拒）——四端一致落地：
  AV（真全链+资源账 8 fixture+21 探针腿）、native（真 PE 执行+collections COFF 发射）、a2x-rust（真实
  rustc 4 案例）、a2x-c（真实 MSVC cl 3 案例），全部真实 dispatch 无 legacy fallback；六职责组零代码红。
  本节只挂接来源，不改 SD-01 分类口径，也不把台账行标为 implemented；状态提升仍只引用 962 归档收据。
- **具名差异（对 legacy）**：位置式索引读以 `.get(i)` 为正式读形（`xs[i]=v` 写位已交付；裸读拼写=
  CST 机制待办 D-962-C1）；range 界需空格分隔（affix-num 词法）；union/grid 语义为新定（旧引擎
  parser-only/panic）。**未闭缺口（具名，带 owner）**：递归 Box 的 collections-unit 车道（D-962-C2）、
  C 侧 tuple 调用实参临时物化（D-962-C3）、List use-after-move 不拒（D-962-C4，冻结既有）、杂项诊断
  拼写（D-962-C5）；StructCover 嵌套模式/is 表达式→965；Map/Set 归 964/946-972（963 实例）。

## SD-07 泛型/约束见证/方法实例化 producer 挂接（PLAN-963 SD-02）

- 分母台账 L-24（默认/命名实参）、L-38（alias）、L-39（spec/约束/关联类型）、L-40（ext/
  方法接收者）、L-42（泛型 type/const 参数与跨模块实例）与 L-47（调用语义的函数值面）的
  ACE 侧**已验证 producer** 为 [m2-generics.md](m2-generics.md)：三个能力裁剪 profile
  （`scalar-instances` / `record-instances` / `text-record-instances`）+ `generic-call` 原子；
  935 单参身份字节不变、页 codec v4（值参数 `constc`），旧页按版本 MISS 重产；四端
  （AV / ac-native / a2x-rust / a2x-c）真实运行收据见该文件 V 行具名入口与
  `run-2v_8abue`（2453 Rust 用例 + 10 case check，七组 PASS）。本节只挂接来源，不改 SD-01
  的分类口径，也不把任何行标为 implemented；状态提升仍只引用 963 归档收据。
- **未闭缺口（具名，带 owner）**：管道算子 `|>`（D-963-C1，963 T-08 后续/965）、可变实参
  无用户声明位拼写（D-963-C2，962 collections 裁定）、`#[with(T)]` 属性形迁移拒
  （D-963-C3，language attribute grammar plan）、记录类型模块全局 `.move` 转移
  （D-963-C4，960 模块面）、默认实参求值作用域边界（D-963-C5，963 T-08 后续）、ACE
  workspace rustfmt 门禁（D-963-C6，ACE 治理）、泛型局部注解诊断质量（D-963-C7，ACE
  语法/投影诊断面）。以上缺口在台账中保持 required/unknown 原状，不因 963 交付而消失；
  963 自身覆盖行（L-38/39/40/42 全、L-24/47 部分）不因 B 未交付改 NA。

## SD-08 动态值对象反射 producer 挂接（PLAN-964 SD-02）

- 分母台账 L-16（array/List 字面量）、L-20（对象/map）、L-21（nil/null）、L-22（可调用值）、
  L-23（动态派发）的动态/对象/闭包切片与 FStr 插值的 ACE 侧**已验证 producer** 为
  [m2-dynamic.md](m2-dynamic.md)：`SourceProfile::Aggregates`/`Closures` 显式源码 profile
  （对象=Rc 共享可变单元、nil 统一、DynType::Callable、通用闭包捕获、FStr 编译前文本变换 +
  I32 显示）、`ac-av/aggregates-source-run-v1` worker recipe、INT-1026/IF-VAL 通用外部值交接
  （不持 PyO3/第二账本）。四端按原支持域矩阵（AV 全能力；native/a2r/a2c 仅 FStr/闭包族，
  对象/动态面具名拒）真实运行收据见该文件 V 行具名入口与 plan §9 并集记录。本节只挂接来源，
  不改 SD-01 的分类口径，也不把任何行标为 implemented；状态提升仍只引用 964 归档收据。
- **未闭缺口（具名，带 owner）**：`&&` 链 f-string 条件 lowering 边（D-964-C1，ACE
  lowering/verify 后续）、冻结措辞 `av.dyn.unsupported` 与实现 `portable.unsupported` 的
  命名空间差异（D-964-C2，文档级，语义一致）、grid 单行异构列语义澄清（D-964-C3，962
  语义裁定者）。以上缺口不因 964 交付而消失；964 自身覆盖行保持 required 原状。

## SD-09 控制流错误传播与生成器 producer 挂接（PLAN-965 SD-02）

- 分母台账 L-18（`is` 语句/表达式）、L-21（nil/null 与 `??` 合并）、L-26（if/else、循环与
  `break`/`continue`/`return`）、L-27（try/catch/finally）、L-28（`.?` 错误传播）、L-30（yield）
  的 ACE 侧**已验证 producer** 为 [m2-control-effects.md](m2-control-effects.md)：
  `SourceProfile::Aggregates` 显式源码入口 + 共享 MIR 四端（AV interpreter、native Cranelift PE、
  A2X-Rust、A2X-C）；语言级 TestCase 组 `case-ace-m2-control-effects-965` 15 case（9 个 negative 标签）。
  语言错误以 `ExecOutcome::Error{message,span}` 与 `Trap`/`Resource`/`Cancelled`/宿主拒绝四分账，
  wire 以 `LANG_FAIL=0x40` + `Terminator::LangFail` 承载。本节只挂接来源，不改 SD-01 的分类口径，
  也不把任何行标为 implemented；状态提升仍只引用 965 归档收据。
- **部分覆盖与具名边界**：
  - L-29（async 块 `~{}`/`.await`/`.go`）**不在本包交付**，调度与流所有权归 PLAN-968 T-08；
    `yield` 只在 `~Iter<T>` 生成器内合法（`source.yield-outside-generator`），不得据此宣称
    async 面已实现。
  - L-18 的 `Cover`/`StructPattern` 形与 L-21 的 nil 字面量归 964/后续，本包只交付 aggregates
    轮廓的 `is` 表达式/语句与 `??`/`.?(d)` 合并/传播语义。
  - **生成器借用参数为合法有界拒绝**（D-965-C1）：状态记录按值保存参数，故 `mut` 参数、记录
    视图参数与 `str`（借用文本）参数即使从不跨 `yield` 使用也以 `source.generator-borrow-param`
    具名拒；支持 owned 句柄（`List`/`Object`/`Variant`/owned `Str`）与纯标量。是否支持只读记录的
    按值拷贝是后续增强，非本包闭合条件。
  - **跨帧 `fail` 不被调用者 catch 绑定**（D-965-C2）：catch 只绑定本帧语言错误；跨调用边界
    的 `fail`/未抑制 `.?` 是终止性语言错误（AV `Error`、native `Trap(9)`、A2X class-only），
    与 D3 四分类一致。
  - **表达式语句分账**（D-965-C3）：入口/宿主轮廓保留"表达式语句只允许作脚本尾"的
    `source.entry-tail` 拒绝；aggregates 轮廓对非尾部、非 unit、非控制效果的表达式语句求值后
    丢弃（调用必须真实执行），不提供隐式尾返回。
  - **`int` 函数末尾 try 可正常完成**（D-965-C5）：`fn f() int { try { g() } catch e { return 42 } }` 保持 `verify.no-return-path` 拒绝（无隐式尾返回）；完全发散的 try/catch 之后的语句为不可达死代码、不再触发 `verify.block-structure`。\n  - **非法跳转具名拒**（D-965-C4）：`break`/`continue` 无用户循环时
    `source.break-outside-loop`/`source.continue-outside-loop`，`try` 帧不计入循环。
- **未闭缺口（具名，带 owner）**：`ace-runtime` 19 例 text shim 链接红与 6 例 `text.char-multi`
  既有红（`master` 既有，独立 fix/Plan，见 965 §10-2）；V-11 表 964 的 `OBJECT_NEW/GET/SET`、
  `TEXT_DISPLAY` 漏行债（§10-6，ACE 治理）。以上缺口在台账中保持 required/unknown 原状，
  不因 965 交付而消失；965 自身覆盖行（L-18/21/26/27/28/30 的 aggregates 切片）不因 L-29 未交付改 NA。

## M2-A 宿主/FFI 生产者已验（PLAN-967，2026-11-15）

PLAN-967（完整 C/Rust FFI、自动 binding 与通用宿主资源）M2-A 全部 work 任务交付并两轮独立
复审（final needs_fix → scoped pass @517920443）。已验生产者/能力与边界见
[m2-host.md](m2-host.md)：F0 通用 AV↔Rust 切片、`ForeignSession` 会话层、异步完成端口、
工程入口（use.rs/use.c）+ 真预处理 C importer + Rust importer 混合帧/p967.frame/1 tag 帧、
六路径消费（AV/native/AV-C/AC/A2R/A2PY）、精确构建缓存/BuildRecord、钉版本真实库
（serde_json cargo 单元 + SQLite 内容钉版）、1043 Rs 装配消费、Windows ABI 兼容轴
（PE 交叉/可选增量/双代并存）。开放项（GrantSet owner/grant 全维、owned lanes、F4 lane-split、
978 运行依赖轴、#in 属性选择）见 KNOWN-DEBT-AND-RISKS D-967-T05——`binding.rs` 拒项目级
use.c/use.rs 的旧阻塞已由本包公共工程入口解除（SD-10 该行由 967 收口）。

## SD-10 标准库领域差额与发行就绪挂接（PLAN-948 SD-02）

- 分母台账 `S-01..S-25`、`K-01..K-08`、`UK-01..13` 与 Design56 增补 `FFI-M2-01..08` / `LIB-M2-01..08` 的 M2-A 领域差额追踪、代表性真实消费者前探（JSON/HTTP/Async/UI）及发行接管阻塞清单见 [ace-domain-migration-readiness](../stdlib/design/ace-domain-migration-readiness.md)。
- **未闭硬阻塞保持**：`stdlib_assembly/loader.rs:68,78` 开发仓 `CARGO_MANIFEST_DIR` 回退、`~/.auto/libs/stdlib` junction、陈旧 `PATH` `auto`、跨仓安装器漂移（`web-install.py`/`musk/install.sh`）、`binding.rs` 拒项目级 `use.c`/`use.rs`（阻塞 `AV/C`/`AV/Rust`）、`sys.arch`/`env`/`ABI` 缺失与 Design46 Gate A/B 未闭。以上阻塞在台账中保持 `missing`/`unknown` 原状，未完成洁净仓外实跑前严禁签发 M2 发行通过。

## M2-A 装配标准库核心（PLAN-972 交付增量，2026-11-15）

装配 std 集（8 模块 62 公共符号，packages-aggregates-ct lane）经五执行端
（AV/AC-native/A2R/A2C/A2PY）真跑同值，独立 oracle 与正反例齐
（[m2-stdlib-core](m2-stdlib-core.md)）。未闭缺口按 owner 分账：AV×C/
AV×Rust 外库 dispatch 挂 967 merge（外部时序依赖）；Linux 执行腿 NOT_RUN/
owner=1063（2026-11-14 裁定）；regex/crypto/全 Unicode case 归 967；
FStr 归 964；泛型/Str 键集合归 963。旧义务不减；本节为已验现状记录，
非范围声明。
## M2-A 产品执行配置与嵌入前门已验（PLAN-969，2026-11-15）

PLAN-969（ACE 产品执行配置与嵌入前门）M2-A 全部任务交付并经独立终审 pass
（0 BLOCKER/0 MAJOR/4 MINOR 全数折叠）。已验现值：公开显式 ACE 同步执行门
（`auto-lang-core::ace_entry`：引擎门/六态结果/具名拒绝，零 legacy 引用）＋
参考执行桥（`crates/auto-lang-ace-host`：scalar/core/console/F0 foreign 四车道，
每 run 重授权，provider 生命周期，console 捕获）＋结构化 profile 表面事实选路
（AUDIT-03，消息文本零参与）＋事实权威表（AUDIT-08）＋兼容门审计（旧 ABT 受控
拒、args/stdin 具名拒、旧导出各保 legacy owner）。接口/预算/边界单源见
[m2-facade.md](m2-facade.md)。未闭缺口（AV argv/stdin 车道、增量会话经公开门、
console×foreign 组合、撤权竞态 async 语义）见 KNOWN-DEBT-AND-RISKS D-969；
旧义务不因本包 A 切片改 NA，985/A 消费以本收据为开工门之一。

## M2-A 任务actor调度器已验（PLAN-968，2026-11-15）

PLAN-968（任务async actor与事件调度）M2-A 全部 work 任务交付并经 needs_fix→修复→final pass
（reviewed aa630a2eb）。L-29 的调度与所有权切片（`task`/`on`、`Task.spawn`、`send`/`ask`/
`reply`、`await`、`go`、`sleep`、有界邮箱、虚拟时钟、停止钩子 drain）在 AV 与 a2r 两端以
相同可观测语义交付；native/a2c 按原支持域具名拒。已验能力、语义冻结（D1/D2）、MIR
`SCHED_*` 操作码与执行模型见 [m2-scheduler.md](m2-scheduler.md)。原 965 台账中 L-29 的
"归 PLAN-968" 义务由此闭合调度切片；async 块 `~{}` 语法面与流所有权深水区仍按分母保持
required，不因 968 的 A 切片改 NA。B 阶段消费者替换（985）以本包 A 收据为开工门。

## M2-A REPL持久会话daemon已验（PLAN-970，2026-11-16）

PLAN-970（REPL持久会话daemon与在线更新）M2-A 全部 work 任务交付并经 needs_fix→修复→
final pass（reviewed 295e93f7c）。A-06/D-07 的会话/daemon/MCP 切片以显式 ACE 面交付：
中立契约 `auto_lang_core::ace_session`、参考宿主 `auto-lang-ace-host::session_bridge`
（冻结声明集通道模型、变量携带表、resident 异步混代、`ac.session.state/1` op-log 持久
重验）、`ac.session.daemon/1` daemon/client（版本纪律、幂等重放、取消）与 MCP ACE 变体
（9工具）。旧 autovm_persistent/daemon/client/mcp 不动（退役归987）；auto-man CLI 接线归
985/B。已验接口、边界与永久验证入口见 [m2-sessions.md](m2-sessions.md)。原分母 A-06/D-07
的 CLI/产品接线义务不因本包 A 切片改 NA；文本/console/collections 会话域按 1024 F05 裁定
保留各自 session。B 阶段消费者替换（985）以本包 A 收据为开工门之一。

## PLAN-978 A 切片：工程增量多目标构建与预编译候选 SDK（SD-02 工作稿）

978 A 交付：`experimental/ace/crates/m2-build`（`ac-build` CLI）——工程构建身份/精确
缓存/CURRENT CAS 发布/恢复、五 lane（AV 进程内、native 真 PE、a2r/a2c/a2py 文件集
+ build-lane 真编译 A2R(rustc)/A2C(MSVC cl)）、可重定位候选 SDK（完整性/未声明文件/
授权/兼容具名拒绝，异根无开发缓存运行）。新增已验证 producer 未关闭的缺口：F0
foreign 执行链复用 967 既有面（978 侧为具名边界）、Linux 腿 deferred/1063、正式安装
升级归 986/987。规范落点 [m2-build.md](m2-build.md)。本节为工作稿，独立 review 定稿。

## PLAN-973 A 切片：系统文件进程时间与资源库（SD-02 挂接）

973 A 交付：`stdlib/ace/sys/` 10 模块系统库（File/path/env/process/time/rand/console 面）
经 1043 装配+50 冻结平台操作四表（source/av-core/AC shim/a2x 端口）——Windows 默认
provider kernel32/bcrypt 直调零 libc/Rust std 委托、统一资源账本（文件/目录/进程/
socket/listener、lo+hi、预算沿 1039 冻结值、cleanup 恰好一次）、诚实拒绝分态。
六路径覆盖（AV/native+AV×C+AV×Rust 外库+A2R 真 rustc+A2PY 真 Python+AC stub 负控）
与宿主 facade 全绿（归档收据含命令/计时）。分母挂接：18_ffi/19_rust_std 系统操作
producer 就位；新增已验证 producer 未关闭的缺口：byte→Str 面（read_text/dir_names/
env get 族/zip/storage 等）→1064、AC 50 操作真身→1065（用户裁定 b）、Linux 实跑
→1063、967 读出参帧。规范落点 [m2-stdlib-io.md](m2-stdlib-io.md)。B 阶段消费者
替换（985）以本包 A 收据为开工门之一。

## PLAN-979 增量：Schema 领域接面与 AGE 投影（SD-02 挂接，2026-11-16）

按本文件 SD-01 分类口径登记本包已验 producer（不复制子计划分母；状态=实现可核对，产品门另计）：

| 入口/producer | repo/SHA | 状态 | 证据边界 |
|---|---|---|---|
| `ace_core::source::age_h0::compile_age_h0` + `ace_av::age_h0::observe_age_h0` | auto-lang `e352e462b` | implemented-verified（H0 切片） | node/view/text + 普通函数文本；完整 AGE 语法不在本项 |
| `ace_core::source::domain::compile_domain`（包/typed 投影/身份/注册表/`mapping_complete`） | auto-lang `e352e462b` | implemented-verified（通用接面） | role 粒度；字段粒度见 `field_manifest` |
| `ace_core::source::field_manifest` + owner 分母（UI 115 行 / AGE 12 行） | auto-lang `e352e462b`；auto-ui `f864f33b` | implemented-verified（清单/覆盖机制） | 分母数据与产品字段归 owner；L04/AG-A 范围外 |
| `compile_age_package`（多模块 AGE 包前门）+ `observe_age_package` | auto-lang `e352e462b`（981/SOURCE） | implemented-verified（CPU 腿） | A2R 实例 1 未取值；g01 不进 A2R；模块级 const/var 拒绝 |
| 传统 AutoUI 字段分母（`ui.widget-decl/v1`） | auto-ui `f864f33b`（980） | implemented-verified（字段面） | 全模式/窗口由 980/PARITY 等继续 |

未闭/范围外（保持显式，不填“支持/无差额”）：

- AGE-L04 选择/条件（981/LANGUAGE + 966）、AG-A01..09（980 + 981/LANGUAGE）、AGE-L03 的
  `view-portrait`/`view-widget`（981/LANGUAGE）；
- GUI/FFI/宿主与两游戏（981/SESSION/RUST/LANGUAGE/PROOF）、完整产品分母（985）、980 其余批次；
- auto-ui 仓 `ace-migration-feasibility.md` 的领域侧增量归 UI owner（本仓不可沉淀）。

规范落点：[m2-schema.md](m2-schema.md)；证据与边界见 PLAN-979 §9。

## PLAN-975 增量：M2-A 数据标准库与跨端业务（SD-02 挂接，2026-11-16）

PLAN-975（M2-A 数据标准库：`json` / `sqlite` / `redis`）全部 work 任务经 round-1 `needs_fix`（`F-01..F-06`）→ 修复 → round-2 `final pass` 归档：

- **三模块与原语**：`char.to(Str)` 物化原语（六路径 AV/`MnDriver`/AV-C/A2R/A2PY/AC）；纯 Auto `stdlib/ace/std/json.at`（encode/decode 族、数组点路径、`i32::MIN`、UTF-16 代理对、控制字符与非法转义拒绝）五端（AV/`MnDriver`/A2R/A2C/A2PY）同得 `42`（含修复 `a2x-c/src/lower_rt.rs` 内容视图参数转发）；纯 Auto RESP2 `stdlib/ace/std/redis.at`（严格 UTF-8 逐字节校验 + `badutf8`/`overlong`/`surr`→`""`）；canonical `pub fn` `stdlib/ace/std/sqlite.at` 单源服务 `text-record`（`TextRecord` `pub fn` 准入）与 `packages-modules`（`:memory:` CRUD + `BEGIN`/`ROLLBACK` + SQL 失败 `ROLLBACK` = `9`）。
- **结构感知与模块边界**：`spec JsonDecodable` + `<T has JsonDecodable> decode<T>` + `966` `ct_field_count`/`ct_has_field` + `app::service` `pub type Service` 跨模块解码；`V-LOAD-MODULE` 私有 helper `source.module-private`、`pub`-stripped RED、`std::json` `ace.stdlib-reserved`、`PackageStore::restore` 与私有体变更 hash 移动全验。
- **具名边界与外部承接**：动态 `JsonValue` AST / 泛型 `encode[T]` / 任意字段遍历反射（M2-B / `966`）；`encoding/csv.at` 未迁移占位；SQLite 文件持久化（`973`/`1064`）；`packages` `p967_*` provider 执行装配（`1043`）；schema 迁移（`986`）；Linux 实跑 `L-01..L-05`（`1063`, `deferred-required / NOT_RUN`）。规范落点：[m2-stdlib-data.md](m2-stdlib-data.md)。

## PLAN-980 增量：传统 AutoUI 全模式 ACE 接入 producer 挂接（SD-02 挂接，2026-11-16，r11）

五批落主（DIALECT/STATE/HELLO/RUST-AXR0/NATIVE，r11 口径 15/29；JET/ARK 归
M3 裁定）。已验 producer（状态=实现可核对，产品门另计）：

| 入口/producer | repo/SHA | 状态 | 证据边界 |
|---|---|---|---|
| 组件面场景族（`compile_component_scenario*`：initials/states-only/免驱动）+ `observe_component_states(_and_texts)` | auto-lang `1d2f01238` | implemented-verified（STATE/NATIVE CPU+真窗） | 免驱动仅 980 入口；打包车道口径不变；观测字段限 int（driverless 除外） |
| `AceStateEngine`（实例世代/异步完成世代纪律=MIX-03 无 HTTP） | auto-ui `ae1c53ad` | implemented-verified | 多单元接线待 IF-981-MODULE 消费；HTTP 实腿 983 |
| HELLO 首窗链（H0→AV→F0×E0→run_app 真窗；分派时重观察防漂移） | auto-ui `e529dde6` / lang `1991021a5` | implemented-verified | PACK 完整打包闭包 pending；单 text 族 |
| F0 构建新鲜度缓存 + ace-host Exit 臂修复 | lang `1991021a5` | implemented-verified | 缓存只认输入身份 |
| AXR0 双腿（组件 A2R `value:2`；F0×A2R 链接 `value:9`）+ 交付闭包修复链（strip_dev_deps/feature 门/foreign-contract vendor） | auto-ui `e2aadd45` | implemented-verified（driver 43/43） | 多文件 emitter 替换待 981 面；A2R 文本出岛 pending（902 冻结协议） |
| ACE 原生 run 通道（`AUTO_UI_ENGINE=ace`；001 真源 Accept 别名/属性树/状态写主题翻转/关闭重开） | auto-ui `34bb4326` + lang `1d2f01238` | implemented-verified | 交互档=既有手工/环境门控；路由运行语义未交付；opt-in 默认链不切换 |
| 分母/命令冻结（T-01） | 本文件 + m2-autoui §10 | implemented-verified | WEB/TAURI/PARITY 行待各自 I 冻结 |

未闭/范围外（保持显式）：WEB 批适配器切换（976 producer 已归档 660fe72b5，
切换归 980/WEB）、TAURI、PARITY 模式网格、T-05..07 终门；JET/ARK=M3 裁定；
KNOWN-DEBT 各具名边界（f-string 文本叶 AV 车道/emit 归 981/默认链 987/交互
档归属等）。规范落点：[m2-autoui.md](m2-autoui.md) §6–§10。

## PLAN-976 增量：M2-A 脚本目标（TS/JS）producer 挂接（SD-02 挂接，2026-11-16）

PLAN-976（共享 Checked-HIR TypeScript/JavaScript provider）work 全毕 → T-05 final pass
（`9da498211`）→ 归档：

- **已验证 producer**：[m2-typescript.md](m2-typescript.md)——新 crate `ace-a2x-ts`
  双目标 `TargetProvider`（backend `auto-a2x-typescript`/`auto-a2x-javascript`，层
  `AXTS1..3`/`AXJS1..3`，fileset schema `ace.a2ts-hir.*.fileset/1`，producer
  `ace-a2x-ts/modules/1`）；i64/u64=bigint、UTF-8 字节文本、checked 算术 trap、
  计划驱动 owned-Str 清理（复合值 JS GC=具名目标差异）；UI 中立 fragment 接口
  `emit_stmt_fragment`/`emit_expr_fragment`；worker `daily-emit-ts`/`run-ts` 真实
  node 通道；多模块 packages lane（`PackageA2xTarget`）与 source map 原 span、
  tsc `--strict` 类型腿（缺 tsc=具名 infra skip）。全部真实 dispatch 无旧
  `TypeScriptTrans`/`JavaScriptTrans` fallback（boundaries 白名单/扫描面已纳入）。
- **范围裁定沿 2026-11-16 r6**：Godot 后置；独立全语言通用 a2ts/a2js CLI 非自动
  门；生产闭包（Vue/Ark/Web/982/983/984 的模块、runtime helper、工具链）为 required，
  产品适配器切换归 980，旧 emitter 退役归 987。
- **具名未闭（KNOWN-DEBT D-976）**：`a2ts.foreign-unsupported`/
  `a2ts.closure-owned-capture` 两族源级永久负控缺位（代码路径复审核读+a2py 同形
  先例）；sched/depth 两族负控已补 `ts_named_rejections.rs`。台账相关行保持
  required/unknown 原状，不因 976 交付消失。

## PLAN-983 增量：M2-A Web 切片 producer/consumer 挂接（SD-02 挂接，2026-11-17 phase fold）

PLAN-983 M2-A 切片（T-01/B-PORT/B-PROOF 共 7 任务，phase review pass
`bdc48e8`）落主；B 组 12 任务在途（worktree 保留）。已验
producer/consumer（RG-A02）：

| 入口/producer | repo/SHA | 状态 | 证据边界 |
|---|---|---|---|
| 中立受检 Web 宿主端口（`web_host_port`；公共面无 AutoVM） | auto-web `bdc48e8` | implemented-verified | 内部执行面仍 legacy 臂（B/AVSERVICE 换）；core trait 桥已文档化 |
| AV 真服务腿（`tests/fixtures/ace-p983/av-service.at` + http_runtime 2 用例；console 前门+run_platform+file/net/mem grants） | auto-web `bdc48e8` | implemented-verified（RG-A02 AV 腿） | member-delivery/b API 面语义；counter.txt 经 file_open_w16（Win32 无 file_open_bytes）；ASCII 数据根；单 owner 串行 |
| A2R 新道生成腿（`ace_a2r_lane`+api_gen 早门/覆写；`AUTO_WEB_A2R_ENGINE=ace` 显式） | auto-web `bdc48e8` | implemented-verified（RG-A02 生成腿） | 零参 int/str/bool 子集；store/db/伴生留旧道（B/GENERATED）；坏源保旧已验 |
| ACE 发射侧生产依赖（ace-core/ace-a2x-core/ace-a2x-rust） | auto-lang `e85096628`（钉基，零改动） | implemented-verified | 解释器 ace-av 保持 dev-only |

规范落点：[m2-web.md](m2-web.md)（A 切片接口+边界）。台账 Web 相关行
不因 A 切片消失：全 Web API 面、AVSERVICE/GENERATED/CONSUMERS、四仓
生产链与 T-05..07 仍 required（B 域）。

## PLAN-977 增量：其余 A2X 目标与独立转换工具 producer 挂接（SD-02 挂接，2026-11-17）

PLAN-977 的 PY 切片（T-01/B-PY-I/V/R，phase fold `56aadead4`）与 REVERSE 切片
（B-REVERSE-I/V `f2606fa91`；前置 M2A-08 = PLAN-985 delivery `95c7fbad8`）已验
producer/consumer（台账 B-03/B-08/B-09 行）：

| 入口/producer | repo/SHA | 状态 | 证据边界 |
|---|---|---|---|
| A2PY 车道（`ace-a2x-python` AXPY1..3 + `a2x-worker run-py`：Checked HIR→确定性 Python 模块→真实 OS Python） | auto-lang `56aadead4` | implemented-verified | 标量/记录文本/集合面；观察五轴（值/stdout/短路/异常/资源）；旧 99 例字节金样属旧实现，新面按家族代表+路由台账承接并持续逐例扩充；Linux runner 归 1063 |
| r2a 独立逆向工具（中立 `syn` 解析+自有发射；不消费 Auto 前端/AST、不经旧执行器或旧目标发射；37 例 golden 逐字节） | auto-lang `f2606fa91` | implemented-verified | 已支持形态保行为；受检面 `transpile_r2a_checked` 为**库级**具名拒绝（17 具名键+1 兜底 `lossy-marker`），CLI 命名开关归 982/REVERSE |
| s2s 源码改写（消费旧前端；`EmitDialect::{Legacy,AceCanonical}`+行级 source map；规范产物经 ACE AV 真跑、诊断回原源行） | auto-lang `f2606fa91` | implemented-verified | Legacy 产物逐字节不变；ACE 执行面只覆盖纯 Auto 标量面；`use.py` 桥面仍归旧脚本管线 |
| 六路径覆盖语料 `cover-977`（同一公共源 expected 14：av-native/ac-native/a2py-run 真跑；a2r/a2c 发射字节锁 + certify 轴真编译运行） | auto-lang `f2606fa91`+本批 | implemented-verified（覆盖） | AV/C、AV/Rust provider 腿用 provider 前门源（provider 身份在源内）绑定 ace-abi F0 证据；legacy A2C 绑 `a2c` 组；Linux 全部 NOT_RUN→1063 |

规范落点：[m2-other-targets.md](m2-other-targets.md)（PY/REVERSE 已验接口、边界、
覆盖矩阵与 Linux 移交审计）。台账 B-03 行证据 pin 随该 spec 更新；B-08/B-09 在
T-06 落主后由 `inflight` 翻 `landed`。本节只挂接来源，不改 SD-01 分类口径；
未闭项（s2s→ACE 记录面、r2a CLI 拒绝面、Linux 腿）具名于该 spec。
