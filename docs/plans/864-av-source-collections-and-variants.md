---
plan_id: PLAN-864
status: drafting
feature_name: AV集合与变体源码闭环（R2-L 前半）
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-ac/source-collections-variants.md
touched_goals: [GOAL-002]

affects: [auto-av, auto-mir, auto-ac, auto-test-case]
current_step: 0
total_steps: 9
---

# [PLAN-864] AV集合与变体源码闭环（R2-L 前半）

## 0. 变更摘要

PLAN-822 已交付执行侧 `av.collections/1`（profile id 4、AVAD v3）：受控 List、Option/Result、有限名义 Variant、Index Place 的 typed 存取/转移/清理，但**明确排除 source 支持面（fixture-only）**（[auto-av/project](../specs/auto-av/project.md)、[collections-variants](../specs/auto-av/collections-variants.md)）。827 的源码前门记录“av source front door rejects List（`source.unsupported`）”。

本Plan补齐**源码到执行**这一段：真实Auto源文件中的 `List<T>`、`?T`、`Result<T,E>`、带载荷有限 `enum`、`is` 模式与 `for-in` 迭代，经前端 → Checked HIR → Typed MIR → ABC → 受检验证 → AV 全链执行，并关闭 852 语料 **L01–L07 / V01–V07** 的源码层缺口行。

不新增执行语义、不改 profile 4 的已冻结指令/布局；新增的是**源码域 + lowering 覆盖 + 消费者绑定**。

## 1. 目标

- 新源码 profile 支持：`List<T>` 构造/`push`/`get`/`set`/`len`/`clear`（与 822 typed 面一致的子集）、`?T`/`Some`/`None`、`Result`、有限 `enum` 声明与带载荷构造、`is` 分支与载荷绑定、`for x in <List>` 迭代。
- 把 852 语料 L/V 行从 `gap:av-unsupported-list` / `wait:av-variants-822` 变为**真实源码执行**或带 owner 的诚实拒绝；不允许把 typed fixture 通过当成源码行关闭。
- 冻结 **SEM-02**（List 负下标/越界/空容器语义）与集合/变体的错误边界（trap vs Result vs 资源失败），并写入 Spec。
- 源码编辑走既有 828 函数级增量会话形状（未改函数身份/生产工作=0），不新造平行会话。

## 2. 架构方案

| 层 | 本Plan动作 | 边界 |
|---|---|---|
| 源码前端 | 新 `SourceProfile`（含 862 入口语义）+ 集合/变体/模式/迭代语法与推导、诊断 | 不引入 Map/Set/JSON、用户泛型、record 容器字段、List 嵌 List（822 非目标保持） |
| HIR | 新 descriptor 与精确可失败三元组；继承 822 的名义 Variant/Index Place 事实 | 不改旧 descriptor |
| MIR/lowering | 复用 822 集合指令与 Index Place；`for-in` 降为显式索引循环；`is` 降为受检分支+载荷投影；无新指令即不新增指令 | 不在 lowering 里偷偷做元素拷贝/借用扩展 |
| 执行 | `av.collections/1` 原样消费，不新增 profile id | 不改 822 指令语义/布局 |
| 增量 | 接 828 单元接口（source 侧会话，822 `collections_session` 同型扩源） | 授权预检/身份/失效沿用 822 合同 |

## 3. 技术栈

- 源码：`experimental/ac-core/src/source/{lexer,parser,resolve,typecheck,adapter,ast}.rs`、`src/{hir,descriptor,verify}.rs`。
- lowering/执行：`experimental/ac-av-probe/core/src/{mir,mir_verify,abc,abc_verify,av}.rs`、`probe/src/{hir_adapter,collections_session,session}.rs`。
- 语料/判定：`crates/auto-lang/test/cases/ace-m1/{src/L*.at,src/V*.at,binding.tsv}`、`crates/auto-lang/src/tests/ace_m1_tests.rs`、`scripts/verify-ace-m1-852.ps1`。
- 拟建 `scripts/verify-av-864.ps1`（Scope=targeted/compat/source/acceptance），测试前缀 `av_collections_864`；当前不存在。

唯一实施树 `D:/autostack/.wt/lang-864/auto-lang`，分支 `plan-864-dev`。

## 4. 需求分析与背景调查

用户 2026-10-28 授权同上（只规划）。事实基础：

- [collections-variants](../specs/auto-av/collections-variants.md) 与 [collection-place-profile](../specs/auto-mir/collection-place-profile.md)：List/Option/Result/Variant 的 typed 存取、Index Place、转移/清理已冻结；`av.collections/1` 语义版本 1/1/1。
- 852 语料 L01–L07、V01–V07 已有**真实 `.at` 源**（`src/L01.at` … `src/V07.at`），当前 AV 列判为 `gap:av-unsupported-list`/`wait:av-variants-822`；`src/L01.at` 形如 `var xs = List<int>.new()` + `push`/`get`，`src/V01.at` 形如 `fn maybe(x int) ?int` + `is x { Some(v) -> … None -> … }`。
- 能力矩阵 C-15/C-16/C-17（List/Option-Result/enum 载荷与 `is`）源码层为“—”，typed 层为 T；C-09（for-in/range/迭代器）为“—”，本Plan只认领 `for x in List` 的有限迭代。
- SEM-02：旧 VM 保留 List 负索引（null-family），822 负下标先拒 → 必须显式冻结，不能拿两种语义直接对拍。
- 827/852 纪律：typed fixture 通过不等于源码支持；改源/删 import 只能作为新变体 ID，不能把原 AP 标绿。

## 5. 详细设计

### 5.1 前置与写面

硬前置：`D862`（入口/Unit）、`D822`（已具备）、`D828`（增量会话）、**`D868`（HIR→MIR lowering 出库）**——集合/变体的源码 lowering 必须落在共享 crate，否则后续 native 迁移要二次搬移。写面见 §3；语料只**追加行与新源文件**（若需为某行新增变体，必须同时保留原行并注明理由）。

### 5.2 源码语义冻结（T-02 决策件）

`attachments/864/work/collections-source-contract.md` 逐项冻结：

| 决策项 | 要求 |
|---|---|
| 支持子集 | 声明/构造/方法/模式/迭代的**精确**清单；超出子集给 `source.unsupported`，不得静默降级为 typed fixture |
| 推导 | `List<int>.new()`/`let xs = …` 的推断规则；`?T` 与 `Result` 的构造/返回；元素类型不隐式转换 |
| 方法签名 | `push/get/set/len/clear`（及 822 已交付面的必要项）逐条：参数、返回、越界/失败形态 |
| SEM-02 | 负下标明确拒绝（沿 822）并具名诊断；越界 trap 码固定；空 List 的 `get` 语义与 `len` 边界写死；旧 VM 差异登记为 profile 差异，不静默统一 |
| 模式 | `is` 分支穷尽性判定、载荷绑定形式、guard 是否在首批；失败诊断码；不得走“首个匹配即成功” |
| 迭代 | `for x in xs` 的求值次数/顺序、集合在迭代中被修改的规则（拒绝或冻结快照，二选一并反控） |
| 资源 | 元素转移/清理与 822 的 once-release 合同一致；禁止以深拷贝冒充；失败路径必须清理 |
| 增量 | 哪些编辑走单函数道、哪些触发精确失效（容器接口变化 vs body 变化），与 822/828 同判据 |

### 5.3 lowering 与执行接缝

- `for-in` → 索引循环（不新增迭代器对象）；边界条件使用 List 长度读取一次或每轮读取必须固定并测。
- `is` → 受检分支 + 载荷投影（822 已有 checked branch projection）；穷尽性缺失在编译期拒。
- 构造 `Some/None/Ok/Err`/Variant 载荷走 822 既有转移/清理语义，不在 lowering 引入额外 clone。
- 与 820 的 record、821 的 owned 面组合的**受限**程度按 822 非目标明确写出（例如 List<owned record> 是否在首批）；不得用“组合未测”冒称支持。

### 5.4 测试框架接入（既有 TestCase 协议，不新造框架）

L/V 两族目前只有 **typed 语料**走协议（`ac-av/collections-run-v1` + `av.entry: "collections:<id>"`，实质期望在父侧 `COLLECTIONS_CASES` 冻结表）。本Plan把**同源真实源码**接入：

- case 形态不变：在 `crates/auto-lang/test/cases/` 追加 source 行（复用 `ace-m1/src/L*.at`、`V*.at`），一份 source × 多 check（av 主判 + legacy/a2r 按实际适用性，ac-native 仍为 `gap:mn-scalar-only`）。
- recipe：既有 `collections-run-v1` 的 entry 语义是 fixture id，**不改造它**。T-02 在两者间冻结：(a) 新注册 `ac-av/collections-source-run-v1`（三缝 append-only + worker 源码域分支，推荐）；(b) 若既有 av binding 已有可用的 source/profile 字段则追加该字段（要求旧 case 逐字节不变）。二者都不得让 case 命名可执行文件。
- 期望落点：源码行的 `expect.exit` / 错误 `stage,code` 写在 case 文件；资源台账（alloc/free/live）若仍需父侧冻结表，按 822 `judge_*` 形状扩展，但“case 声明了什么就必须被判什么”，禁止声明后忽略。
- `binding.tsv` 只改状态/owner 列（`gap:*`→实测或换 owner），不改 12 列 schema；852 base suite 不被悄悄扩缩。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/source-collections-variants.md | 无 → 源码域：子集/推导/方法/模式/迭代/诊断/SEM-02 | 源码闭环 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-av/collections-variants.md | 追加 source 绑定与实测覆盖，typed-only 边界改写 | 事实纠正 | AC-02/05 |
| SD-03 | modify | docs/specs/auto-mir/collection-place-profile.md | 追加 source lowering 约束（for-in/is 不新增指令） | IR 合同 | AC-02/03 |
| SD-04 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | L/V 行重绑定与实测映射 | 证据落点 | AC-05 |
| SD-05 | modify | docs/specs/av-language-capability-matrix.md（design/strategy） | C-15/16/17/C-09 源码列按实测更新 | 路线事实 | AC-05 |

## 6. 测试设计

| V | AC/不变量 | 实际/拟建入口 | 可区分错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-01 源码面 | L01–L07/V01–V07 原 `.at` 全量；构造/方法/模式/迭代正例 | 手算 exit 值与 852 冻结 expected 一致；源不改写 | T-03 |
| V-02 | AC-02 边界与SEM-02 | 负下标、越界、空容器、非穷尽 `is`、错型构造、迭代中修改 | 各自具名诊断/trap 码；把负下标当合法必红 | T-03/04 |
| V-03 | AC-03 资源 | 元素转移/失败路径清理事件（alloc/free 台账，once release） | 台账精确；多释放/漏释放/深拷贝冒充必红 | T-04 |
| V-04 | AC-04 增量 | 单函数 body 编辑、元素类型/接口编辑、强制整备反控 | 未改函数生产阶段=0/身份不变；退化必红 | T-04 |
| V-05 | AC-05 语料绑定 | 852 binding 行更新 + 独立 oracle + SKIP 审计 | 分母不缩；`wait:` 走完关闭或留 owner；改 expected 必红 | T-05 |

定向命令：两独立 workspace 定向 `cargo test`；`powershell -File scripts/verify-av-864.ps1 -Scope <targeted|compat|source|acceptance>`；完整档归 merge。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 源码子集按冻结件成立：L/V 原源全部真实执行或诚实拒绝（带 owner）；V-01 |
| AC-02 | work | SEM-02 与边界/诊断矩阵成立，无静默降级；V-02 |
| AC-03 | work | 资源台账精确、失败路径清理正确；V-03 |
| AC-04 | work | 源码编辑走 828 增量形状，未改函数零重做，退化反控必红；V-04 |
| AC-05 | work | 852 binding/独立 oracle/分母审计落地，typed 通过不冒领源码行；V-05 |
| AC-06 | review | 独立核 AC/V/SD、lowering 是否私加拷贝、profile 4 语义零漂移、warnings/fmt |
| AC-07 | merge | 冻结提交裸 `cargo t` + `tv`（改 VM/编译器触面）通过 |
| AC-08 | merge | Spec 沉淀、矩阵更新、ledger、归档、guard 清理完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | D862、D822 | 核实 822 typed 面与源码前门拒绝点、L/V 源文件实际语法、828 会话接口；产出 baseline | AC-01/02 | [ ] |
| T-02 | work | T-01 | 冻结源码子集/SEM-02/资源/增量判据合同 | AC-01..05 | [ ] |
| T-03 | work | T-02 | 前端：语法/推导/诊断/descriptor；V-01/02 部分 | AC-01/02 | [ ] |
| T-04 | work | T-03 | lowering（for-in/is/构造）+ 会话扩展 + 资源/增量测试；V-02/03/04 | AC-02/03/04 | [ ] |
| T-05 | work | T-04 | 语料绑定/oracle/脚本与矩阵更新；V-05，提交代码与证据 | AC-05 | [ ] |
| T-06 | review | T-05 | 独立最终review；merge 门 pending | AC-06 | [ ] |
| T-07 | merge | T-06 pass | 冻结提交裸 `cargo t` + `tv` | AC-07 | [ ] |
| T-08 | merge | T-07 | Spec/矩阵/ledger/归档/guard | AC-08 | [ ] |
| T-09 | merge | T-08 | 交接 865（闭包/HOF）与 866（在线重载）实际基面与限制 | AC-08 | [ ] |

## 9. 复审记录

2026-10-28 new r1：`stage: new | plan_id: PLAN-864 | plan_revision: 1 | outcome: pass（等 D862，合同可预检） | next: D862 后 work T-01`。全部 AC/V/T 未执行；子集清单、诊断码、SEM-02 具体形态待 T-02 冻结；未建树、未跑构建、未改在途Plan。批次排序依据见[后继批次规划](reports/ace-next-batch-2026-10-28.md)。

## 10. 待澄清事项

- `List<owned record>`/`List<Str>` 是否进入首批：822 非目标含“record 容器字段/List 嵌 List”，本Plan默认**不含**，仅标量元素 + 已交付面；若用户要求同时覆盖 owned 元素，需在 T-02 前明确扩大范围与资源预算。
- 迭代中修改集合的语义（拒绝 vs 冻结快照）需用户或语言owner裁定；本Plan默认**拒绝**并给具名诊断（保守），如要允许快照语义须另立合同。
- `for-in` 是否同时支持 range（`0..n`）/数组/Tuple：默认只 `List`，其余留 R2-L 后续切片。
