# ACE 826 之后：后继批次规划与排序依据（2026-10-28）

本轮只做规划：不建实施树、不跑构建/语料/门禁、不启动其他会话，也不改在途计划（843/844/861 等）的合同或进度。本文件记录**为什么是这五个切片、依什么顺序、还欠什么**，供后继Agent按[ACE总入口](../../design/strategy/auto-native-backend-evolution.md)“规划读取步骤”复核。

## 1. 当前实际基面（作为规划前提，不是施工成果）

| 已交付 | 认领范围（按其自身合同） | 明确不覆盖 |
|---|---|---|
| D798 / D828 | Checked HIR→Typed MIR→ABC→私有LoadedProgram→Rust AV；函数级增量的 prepare/commit 事务、pin/lease、旧代保活、HISTORY_CAP=32 | 在线运行中替换、完整语言、产品默认 |
| D820 / D821 / D822 / D823 / D824 | `av.data/1`、`av.owned/1`、`av.collections/1`、`av.closures/1`、`av.dynamic/1` 五条 typed profile（AVAD v1–v5、profile id 2–6） | 除 data/owned 外**源码面**（822/823/824 明示 typed-only） |
| D825 / D826 | 标量 `LoadedProgram` 优化执行层；紧凑 wire `AVAC/1` | 不扩类型/语言/CTE/native |
| D827 | 真实 scalar 源文件验收（AP-01/AP-22/AP-23）与 G0 库存 | 非标量源码 |
| D852 | 四路径一致性语料（58 源 + 5 typed 入口 + 40 scenario）、父侧判定、a2r 金样认证 | 未实现能力一律保留 gap 行，不判绿 |
| D817 / D818 | 有限 CTE 消费者；有限标量 MIR→native 对拍 | 复合值 CTE/native |
| D859 / D860（已归档）、D861（execution_done，待复审/合入） | AV 语料特性边界恢复、AutoGraphics 运行可行性、公共动作与 2D 角色 | 与本源码扩面线无共享写面；D861 在 `plan-861-dev`，本轮回写避让 |

`826` 归档后，[Design43 §0.4](../../design/43-ac-av-typed-mir-and-abc.md)所说的“ACE 首版核心骨架基本成型”已可认领；但按[能力矩阵](../../design/strategy/av-language-capability-matrix.md)**绝大部分源码能力仍是“—”（C-06/C-09/C-11/C-15…C-18/C-20…C-29/C-30…）**，M1 的四路径分母里 `gap:mn-scalar-only`、`gap:ac-composite-native`、`gap:mn-no-typed-entry`、`gap:av-lower-method`、`gap:av-unsupported-list`、`gap:av-online`、`gap:ac-native-loader`、`gap:av-821-gen`、`gap:av-alloc-fault` 仍带 owner 未关闭。

结论：下一批的价值不在“再优化执行/编码”（825/826 已证明），而在**把真实源码覆盖面从 scalar 扩到常用程序**，并补上 Design43 §6.1 明确要求“较早安排、不能拖到 G6/AJ”的在线热重载。

## 2. 本批五份计划

| Plan | 切片 | 路线标识 | 关闭/推进的缺口 | 硬前置 |
|---|---|---|---|---|
| 862 | 源码入口、Unit、尾值、退出通道 | R2-E（入口部分） / **SEM-05** | C-06、G2a 前门 | D798/D828/D852 |
| 863 | 类型化宿主导入 + `print`/`eprint` 精确输出 | R2-E（输出部分） / C-29 | C-29、宿主导入形状（供 R3-H/R6-F 复用） | D862 |
| 864 | List/enum/Option/Result/`is`/`for-in` 源码闭环 | R2-L（集合与变体） / **SEM-02** | C-15/16/17、C-09 子集、`gap:av-unsupported-list` | D862、D822 |
| 865 | 闭包源码语法与间接调用 | R2-L（闭包） / **SEM-04** | C-18、C-11 子集、`823-future-source` | D862、D864、D823 |
| 866 | 常驻在线热重载 | **RF-H** | `gap:av-online`、Design43 §6.1 热重载责任 | D828、D862 |

排序理由：

1. **862 先行**：五份计划都要“程序能进入、结果能判定”。入口/Unit 是横向前门，也是 SEM-05 的裁定点；先冻结再让别人消费，避免三个计划各写一套入口规则。
2. **863 次之**：真实程序的第一诉求是输出与诊断可见。它同时确立**宿主导入的公共形状**（符号/签名/能力/预算/派发/取消），后续 FS/argv/stdlib/FFI 复用同一形状而不是每族自造。*注意*：它不是“加个 print 函数”，而是 ABC 新增 host-import 类别 + 准入 + 派发；当前 `CalleeRef::Import` 在 AV 派发不可执行，属真实缺口。
3. **864 第三**：822 的执行面已冻结，缺的只是源码域 + lowering + 会话绑定；投入产出比最高，且 L/V 语料源文件已在库（`test/cases/ace-m1/src/L0*.at`、`V0*.at`）。
4. **865 第四**：依赖 864（捕获 owned 值/后续 HOF 需要集合面）；且 852 已把“两条 legacy 路径都解析不了 move 闭包”记为 `diff-legacy-grammar(parse)`，语义裁定（SEM-04）必须先于实现。
5. **866 与 863/864/865 无依赖，可与任一并行**：它是横向结构责任，只消费 828 的安装事务与 862 的入口判定；越晚做，G5/G6 的 UI/服务常驻会话越容易各自发明“热重载”。

依赖图：

```text
D868（lowering 出库，见 §2.1）
   └─→ 862 ──┬─→ 863
             ├─→ 864 ─→ 865
             └─→ 866（可与 863/864/865 并行；不触 lowering）
```

单写纪律：864/865 会同时触及 lowering 与 `ac-core/src/source/**`（D868 之后 lowering 位于共享 crate，见 §2.1）；866 触及 `core/src/program_snapshot.rs`/`session.rs`/`driver.rs`。共享核心仍按[共同合同](ace-execution-order-and-incremental-compatibility-2026-10-20.md)单写交接：先合入者冻结基面并记录释放，后到者核验接收；不持窗等上游、不消费 WIP。

### 2.1 迁移前置：PLAN-868（HIR→MIR lowering 出库）

2026-10-28 用户确认迁移方向（旧的 HIR 直达改为 HIR→MIR，使 MIR→Native 共享），并要求先单独建立 lowering 出库计划 → [PLAN-868](../868-hir-mir-lowering-shared-crate.md)。

**排序事实**：868 不是 862–866 的**语义**前置（866 完全不触 lowering），但它是 862/863/864/865 的**写面**前置——这四份都要改 `HIR→MIR` lowering，而该实现当前住在测试宿主 `experimental/ac-av-probe/probe/src/hir_adapter.rs`，且被 `ac-mir-native`、`ac-cte-probe` 跨 crate 依赖。先做功能再搬移 = "改完再搬"，会产生跨计划 merge 冲突与证据重绑定。因此本批实际顺序改为：

```text
868（纯搬移 + 依赖倒置 + lowering 版本身份）
  → 862（入口/Unit） → 863（console） → 864（集合/变体） → 865（闭包）
  → 866（在线重载，随时可并行）
```

868 期间的证据影响：**收据不动**（`ace.m1.cert/1` 键集不变，852 双消费者校验不受影响）；**报告**新增 lowering 版本轴，旧报告按既有 stale 语义判 STALE 属预期。

### 2.2 迁移三段（Design49 已冻结）

用户 2026-10-28 确认：迁移批次**不含** collections/closures/dynamic（AV 源码面尚未闭合，先扩 AV 再谈 native）。迁移合同落在 [Design 49](../../design/49-native-single-mir-lowering.md)（模块边界、布局/ABI、身份/cache、逐 profile 切换与退役判据）：

| 段 | 计划 | 交付 | 门 |
|---|---|---|---|
| 1 出库 | [868](../868-hir-mir-lowering-shared-crate.md) | `auto-mir-lower` 单源；三消费者；`MIR_LOWERING_VERSION` | 搬前/搬后逐 check 等价 |
| 2 扩面 | [870](../870-ac-mn-record-text-native-expansion.md) | mn 扩 record/text/text-record；关 `lower.rs` 的 394（限 Char/Str）/462/484 | 三路差分（旧 / mn / AV）逐面一致 |
| 3 切换+退役 | [871](../871-ac-native-switch-and-old-lowering-retirement.md) | driver 层按 profile 切 mn；删旧 HIR 直达 lowering；脚本/金样/Spec 收口 | 逐 profile shadow 绿 + 无消费者残留 |

```text
868 ─→ 870 ─→ 871        （native 单源）
  └─────────→ 862 ─→ 863 ─→ 864 ─→ 865   （AV 源码扩面；862–865 的 lowering 落共享 crate）
                         866（在线重载，随时并行，不触 lowering）
```

**边界提醒**：870 只覆盖 **AV 已交付**的 820/821 面——AV 仍 `gap` 的项（如 `gap:av-lower-method` 的实例方法）没有差分基准，不随迁移进入。

## 3. 需要用户裁定或后续Plan认领的项

**语义裁定（阻塞对应计划的实现，不阻塞规划）**：

- **SEM-05**（862）：Unit main / 顶层脚本 / int main 是否同批；显示格式归 863。
- **SEM-02**（864）：List 负下标（旧 VM 保留 vs 822 先拒）与空容器/越界形态。
- **SEM-04**（865）：Copy 捕获 / owned 捕获 / 显式 clone / 共享可变捕获的首批面。
- **SEM-03**（record `str` 字段）仍未裁定：`auto.source.text-record` 拒绝 `str` 字段，而 AP-08 的 `Note.title` 就是 `str`。本批**不认领**，登记为 R2-D 缺口（见下）。

**本批刻意不做、需后继Plan认领的切片**（按建议顺序）：

| 优先 | 切片 | 归口 | 说明 |
|---|---|---|---|
| **P0（已立项）** | **HIR→MIR lowering 出库为共享 crate** | [PLAN-868](../868-hir-mir-lowering-shared-crate.md) | 迁移的硬前置；纯搬移，不改语义；862–865 的 lowering 写面依赖它 |
| **P0（已立项）** | AC 复合 native 扩面（record/text/text-record） | [PLAN-870](../870-ac-mn-record-text-native-expansion.md) | 迁移第二段；关 `lower.rs` 的 394（限 Char/Str）/462/484；只覆盖 AV 已交付面 |
| P0 | M1 分母重绑定（T*/R*/L*/V* 行的 `wait:av-*-822` 现状核实） | 852 后继 | 822 未获授权新增 T-11，`wait:av-text-822`/`av-record-822`/`av-variants-822` 既未被关闭也未被重绑；须按 D820/D821/D822 实际交付重新评估，属证据/绑定工作（不改 Rust 源码时按 Category A 处理） |
| P0 | mn collections/variants + native typed entry（关 509 与 `mn-no-typed-entry`） | 818 后继 | 必须紧跟 864，否则 L/V 行在 mn 列永久 gap；**不在本轮迁移批次**（用户 2026-10-28 裁定） |
| P0 | R2-N 数值源码与算术（i64/u64/f32/f64、位运算、Math） | 能力矩阵 C-03/C-04/C-05 | 与上一行的 typed 入口部分重叠，需合并排期；宽数值常量在 870 中仍受控拒绝 |
| P0 | R3-M 多模块/use/pkg/全局初始化 | C-21/C-22 | AP-07；复用 resolver 与 783 store |
| **P1（已立项）** | 消费面切换 + 旧 HIR 直达 lowering 退役 | [PLAN-871](../871-ac-native-switch-and-old-lowering-retirement.md) | 迁移第三段（Design49 §6）；前置=D870 差分门全绿；切换点在 driver 层（禁入 ac-core，成环） |
| P1 | R3-T 用户泛型/spec/方法派发 | C-23/C-24 | 864 只做内建实例化，不等同用户泛型 |
| P1 | R3-H stdlib 与 typed host（FS/argv/env/进程/权限） | C-30 | 直接复用 863 的宿主导入形状 |
| P1 | R3-C JSON/Map/Set/匿名对象 | C-20/C-31 | `av.dynamic` 不是 Map 实现 |
| P1 | R5-A/O/S async/Task/共享/网络服务 | C-32/C-33/C-34 | G5 |
| P1 | R6-U/I/F UI/会话/嵌入/FFI | C-35/C-36/C-37/C-38 | G6；native loader（I08）属此线 |
| P2 | 有限增量发布（Design43 §9 的闭包/复用/补编） | 独立发布器计划 | 不塞进任何 VM 探针 |
| P2 | 复合 CTE / 817 扩面（EP-08） | C-39 | 817 只认标量 |
| P2 | RF-C 分层 cache 覆盖、RF-I 增量粒度核验、block 级复用（§8.1） | Design40/43 | 横向；有明确 owner 才开工 |
| P3 | G7 分 SKU 默认切换 / G8 旧 AVM 退役 | 路线 G7/G8 | 需先满足对应 G 门与全示例分母 |

## 4. 与里程碑的关系（不夸大）

- 本批把 **M2“现有语言能力完整”** 从“scalar 单点”推进到“入口/输出/集合/闭包”四种常用面，但**不构成 M2 完成**：模块、泛型、数值族、stdlib、async、UI/FFI 仍缺。
- **M1 不会因本批变绿**：四路径要求 native 侧同样判定同一批源码；复合 native 扩面未认领前，`gap:mn-scalar-only`/`gap:ac-composite-native` 持续保留。
- **M3/M4** 不受本批影响；AACE 仍在 v0.7 且需 Rust 版成熟度门（[版本合同](ace-milestones-version-boundary-2026-10-24.md)）。
- 866 关闭的是 **AV 侧**在线重载；native 在线装载/热替换（`gap:ac-native-loader`、I08）仍是缺口，不得由 866 冒领。

## 5. 规划输入与未验证项

- 依据：Design43/44、ACE总入口 §0.0–§0.7、AV源码与替换路线（G0–G8/R2-*/RF-*）、能力矩阵、版本合同、852 交接合同、852 conformance Spec 与 `crates/auto-lang/test/cases/ace-m1/binding.tsv`（124 行：63 corpus + 40 scenario + 注释）、[auto-ac/project](../../specs/auto-ac/project.md) 入口节、[auto-av/project](../../specs/auto-av/project.md) 各 profile 段、[resident-function-install](../../specs/auto-av/resident-function-install.md)、[TestCase v2](../../specs/auto-test-case/profile-v2-native.md)。
- 代码位置声明为**拟定写面**：`experimental/ac-core/src/source/*`、`experimental/ac-av-probe/{core,probe}/src/*` 已按目录核实存在，但具体符号/行号须由各 Plan 的 T-01 复核；本文件不把“读到目录”当“读过实现”。
- **未验证**：各 profile 号（新计划建议 7/8）、新脚本名、诊断码、预算数值、上限值均为建议，须由各 Plan 的 T-02 决策件冻结；本文件与五份草案不构成任何实现通过或性能达标声明。
- `docs/design/strategy/*` 与 `docs/design/43/44` 当前在主检出带有他方未提交改动（2026-10-28 `git status` 可见），本轮**未编辑**这些文件；导航回写（§0.4 批次表、§0 导航行、能力矩阵 `planned_by`）应在该批次提交后由 owner 补，避免覆盖他方 WIP。
