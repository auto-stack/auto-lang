---
status: accepted-direction
describes: target-state
created_at: 2026-10-28
updated_at: 2026-10-28
implemented_by_plans: [PLAN-868, PLAN-870, PLAN-871]
---

# Design 49：native 单源 MIR lowering（HIR→MIR→CLIF 收敛）

> **2026-10-28续接基线**：868已归档且共享lowering出库完成，当前仍保留旧HIR→CLIF native参照路径；870/871待实施，不能提前宣称native单源收敛。871须按旧五profile和方法能力过G-COV/G-DELETE，保留TextResourcePlan的verify层分析；覆盖不齐让出共享窗继续功能线。 统一进度快照与未完工作见[ACE续接基线](strategy/ace-implementation-handoff.md)，后续按current Spec/Plan及merge收据重核；本注不扩大原Plan验收。

> 定位：本文件冻结"**共享 Typed MIR 是唯一语义源，native 只是它的一个 consumer**"的迁移合同。
> 它不新增语言/执行能力，只规定既有两条 native lowering 如何收敛为一条，以及收敛期间的
> 身份、ABI、切换与退役判据。技术语义仍由 [Design43](43-ac-av-typed-mir-and-abc.md)、
> [Design44](44-av-data-layout-place-and-ownership.md) 与各 current Spec 拥有。

关联：[Design43 §2/§10](43-ac-av-typed-mir-and-abc.md)、[Design44](44-av-data-layout-place-and-ownership.md)、
[Design40 产物/cache](40-compiler-artifacts-and-cache-lifecycle.md)、[Design41 ABI](41-interface-metadata-and-foreign-abi.md)、
[ACE 总路线 §0](strategy/auto-native-backend-evolution.md)、[HIR 战略](strategy/auto-hir-design.md)。

## 1. 问题陈述

ACE 现在有**两条 native lowering**，从两个不同的 IR 出发：

| lowering | 输入 | 实现 | 语言面 | 消费者 |
|---|---|---|---|---|
| HIR 直达（旧） | Checked HIR | `experimental/ac-core/src/native.rs`（资源分析由verify层`text_resources.rs`共享，非整文件退役对象） | core / scalar / text / record / text-record（741/745/767/770/771/772/782 交付） | `ac` 协议适配器（经 `ac-case-worker`）、`auto-ac-prototype` CLI |
| MIR→CLIF（新） | Typed MIR | `experimental/ac-mir-native/src/{lower,emit,driver,...}.rs` | **仅 scalar**（818） | `ac/mir-native-scalar-v1` recipe |

由此产生的结构性问题：

1. **语义双规范**：同一语言特性在 HIR 层与 MIR 层各有一份"应该怎么执行"的隐含规范；AV 只认 MIR，所以两条 native 之间、以及旧 native 与 AV 之间没有共同参照。
2. **实际重复**：scalar 有 **3 份实现**（旧 native / mn / AV 解释）；record+text 有 **2 份**（旧 native / AV）。
3. **扩展面错位**：mn 对 data/owned/collections/closures/dynamic 是**受控拒绝**（`lower.rs` 明确 "outside the mn scalar surface"），所以四路径语料里 `ac` 列在 record/text 及以上一律 `gap`，M1 无法收口。
4. **边界倒置**：HIR→MIR lowering 目前住在测试宿主 `probe` 里，却被 `ac-mir-native`、`ac-cte-probe` 跨 crate 依赖（由 [PLAN-868](../plans/archive/868-hir-mir-lowering-shared-crate.md) 修复）。

时序原因（不是设计失误）：旧 lowering 的 record/text 交付在 2026-10-14 前完成，而 Typed MIR/ABC/AV 路线在 2026-10-16 才确认（798）。[Design43 §2](43-ac-av-typed-mir-and-abc.md) 明确"设计更新不使既有成果失效"，因此当时的正确选择是保留双轨、另立迁移。

## 2. 目标架构

```text
                    ┌─────────────── ac-core ───────────────┐
source ──▶ AST ──▶  │ resolve/typecheck · Checked HIR · verify │
                    └────────────────────────────────────────┘
                                   │ Checked HIR
                                   ▼
                        ┌──────────────────────┐
                        │   auto-mir-lower     │  唯一 HIR→MIR lowering
                        └──────────────────────┘
                                   │ Typed MIR
                    ┌──────────────┴───────────────┐
                    ▼                              ▼
            auto-av-core (MIR)             auto-av-core (MIR)
                    │                              │
        ┌───────────┴───────────┐                  │
        ▼                       ▼                  ▼
  ABC 编解码/验证           MIR→CLIF          AV 解释执行
        │                  (ac-mn)              (语义基准)
        ▼                       │                  │
       AV 解释执行 ◀────────────┘                  │
                                                    │
        消费者：probe / ac-mn / ac-cte / ac-case-worker
```

不变式（本设计要冻结的四条）：

- **I1 单源语义**：任何 native 执行都必须从 Typed MIR 出发；不得再从 HIR 直达机器码。
- **I2 双后端**：MIR 的两个 consumer（解释 / CLIF）是**同一语义的两种实现**，不是两套语言。
- **I3 单一归属**：HIR→MIR lowering 只有一个实现、一个所属 crate、一个版本身份。
- **I4 判定在父侧**：差分/等价判定由测试宿主父侧完成，后端不得自证。

## 3. 模块边界

| crate | 职责 | 允许依赖 | 禁止 |
|---|---|---|---|
| `ac-core`（`auto-ac-prototype`） | 源前端、AST/HIR、descriptor、verify、COFF/PE 链接设施 | — | 依赖`auto-av-core`/`ac-mn`（会成环）；G-DELETE后保留旧native lowering（迁移期仅允许既有参照路径） |
| **`auto-mir-lower`**（新，868） | HIR→MIR 唯一 lowering + `MIR_LOWERING_VERSION` | `ac-core`（HIR 类型）、`auto-av-core`（MIR 类型） | 依赖 Cranelift；依赖 probe/会话/测试类型 |
| `auto-av-core` | MIR 模型/verifier、ABC codec/verifier、AV 解释器 | 仅 std | 依赖 `ac-core`（执行核心不得被前端 HIR 污染） |
| `ac-mn` | MIR→CLIF、per-function COFF、单元 cache、PE 链接 | `auto-av-core`、`auto-mir-lower`、Cranelift、`ac-artifact-probe` | 依赖 `ac-core` 的 **native lowering 符号**；自建第二份 HIR→MIR |
| `ac-case-worker` / driver 层 | **切换点**：为测试协议/CLI 选择 native 后端 | `ac-core`、`auto-mir-lower`、`ac-mn` | 在 `ac-core` 内做后端选择（成环） |
| `auto-lang`（产品） | 测试编排、协议核心消费 | `auto-test-case`（+ 受控 worker IPC） | 直接依赖任何实验栈/Cranelift |

**关键边界（易踩）**：后端切换**不能**放进 `ac-core`——`ac-mn` 依赖 `ac-core`，若 `ac-core` 反向调用 `ac-mn` 即成环。切换点必须在 `ac-case-worker`/driver 层（协议侧）或独立 CLI wrapper；`ac-core` 的终局角色是"前端 + verify + 链接设施"，不再是 native 代码生成器。

## 4. 布局与 ABI 合同

MIR 是语义事实，native 布局是其**下游投影**（与 Design44 的分工一致）：

| MIR 类型 | native 表示 | 约束 |
|---|---|---|
| `I32` / `Bool` | 32 位；bool 规范 0/1 | 沿 741/798 冻结表示 |
| `I64`/`U64`/`F32`/`F64` | 精确宽度、位型保真 | 值搬运保真 ≠ 已交付算术（算术属 R2-N） |
| `Record`（名义） | 按 Design44 字段顺序的聚合；同名不同 owner 不互换 | 布局是语义投影，不得按"同宽度"合并 |
| `Str`（拥有）/ `str`（借用） | 拥有缓冲 + 借用视图；drop 责任在拥有方 | 借用不得逃逸；禁用宿主 `String` 布局假设 |
| `char` | Unicode 标量（u32 语义） | 与 text 域索引单位一致 |
| borrow/`RecordView`/mut 参数 | 按引用传递 + 写回 | 与 821 的静态借用边界一致；不承诺运行期句柄校验 |

调用与入口：

- 入口仍是"模块级语义"（entry kind/signature 属于语义，见 Design43 §12.1）；native 包装器（旧 `ac_start`）只做结果→进程退出映射，不是语言 ABI。
- trap 通道：语言 trap 与资源/infra 分列；旧路径的 `ExitProcess(70)` 是**测试 profile 约定**，迁移后必须与 AV 的 typed trap 码有明确映射表（不得用同一数字代表两种语义）。
- drop/cleanup：**全出口清理**、无宿主递归（迭代式 worklist）；不得依赖 Rust drop glue 处理 guest 深链（821/822 已付过代价）。
- 资源账本：native 侧的可观察分配/释放计数必须与 AV 台账口径一致（差分门比对项之一）。
- 与 797 AutoABI 的区分保持：AV/native 内部调用约定不是对外 AutoABI（Design41）。

## 5. 身份与 cache 迁移

| 维度 | 旧（HIR 直达） | 新（MIR 单源） | 迁移规则 |
|---|---|---|---|
| 编译单元身份 | HIR 声明/模块 | MIR 函数单元（818 per-function COFF） | 单元身份随 MIR；不得以 HIR 序号冒充 |
| 产物层键 | 783 artifact probe 的 HIR 键 | MIR/单元键 + lowering 版本 | 换键必须版本化；两套键不得混用同一 store 条目 |
| lowering 版本身份 | 无 | `MIR_LOWERING_VERSION`（868 引入） | 进报告轴；语义真变时升版本并显式失效 |
| 恢复/重验 | 反序列化 HIR 产物 | decode + verify MIR/ABC 后重跑准入 | 恢复永不沿用 trusted 标记 |
| 跨进程 | 视层而定 | store 跨进程，重新准入后执行 | fresh/hit/restart 分列报告 |

约束：迁移期**允许两套产物并存**（不同层/不同键），但**禁止同一逻辑产物两种键**；旧层在 871 退役时按"显式失效或删除"处理，不做静默兼容。报告新增轴会使旧报告判 STALE——预期且仅影响报告，**收据 `ace.m1.cert/1` 键集不动**。

## 6. 逐 profile 切换与退役判据

五个旧profile **core / scalar / text / record / text-record**分别保留身份、合法集合与拒绝边界；默认按此顺序shadow与切换。不能将text仅经text-record通道或只跑三组样本当完整覆盖。具体合同落[871](../plans/871-ac-native-switch-and-old-lowering-retirement.md)§5.2.1/§5.4。

| 门 | 通过条件 |
|---|---|
| S0搬移 | D868版本轴及消费者等价交付；不改变868在途合同 |
| G-COV旧能力覆盖 | 每profile按旧current Spec/永久测试列完整能力与消费者，逐行有共享lowering+mn覆盖、独立expected与实测。含静态/只读方法、mut/move/clone、alias字段、Unicode/资源/清理、CLI/脚本/cache |
| shadow及默认 | G-COV齐全后同源旧native/mn/AV及独立oracle判值、typed trap/诊断/台账；坏MIR/ABC/预算反控成立，连续绿次数由871 T-02冻结，逐profile切换 |
| G-DELETE | 五profile全过、全部协议/CLI/脚本/cache消费者迁完、旧键失效/恢复重验及残留检查就绪；注入缺方法/消费者必须阻塞删除并保持旧可用路径 |
| 最终退役 | 只删除native.rs旧HIR直达及证明独占的辅助，保留verify层TextResourcePlan分析、text_resources.rs共享接口及verify真实调用；无旧lowering消费者后回写Spec/矩阵/ledger |

**D870不是旧能力全集证明**：870只覆盖AV已有data/owned子集，旧方法等缺口由共享方法lowering能力包补齐（待立项，871记录producer/阻塞owner/oracle）。不得以NA、unsupported、删用例或改expected消掉旧合法行为。部分profile切换不等于D871或单源退役完成。

**回退与写面**：删除前按profile回退，不回滚旧语言能力；删除仅在全门通过后实施。871删除实际改ac-core，需S及所需N/H同时取窗；优先864/866后插入，未就绪让窗给865/863，后续再接。公共S队列及D门/写面顺序区别见[波次交接合同](../plans/reports/ace-wave-handoff-2026-10-28.md)。

迁移完成的对等判据是同一共享MIR、两个consumer和独立冻结oracle；旧路径/旧Spec仍为迁移期行为约束，AV不能以自己的缺能力替代它们。

## 7. 明确不在本设计范围

| 排除项 | 理由 |
|---|---|
| collections / closures / dynamic 的 native 化 | AV 侧源码面尚未闭合（822/823/824 为 typed-only）；先扩 AV 源码再谈 native，否则同时做两件事 |
| 宽标量算术（R2-N） | 数据面搬运（820）与算术是新能力，不是迁移 |
| native typed entry（W01–W05） | 独立小切面，可随 R2-N 或单独计划 |
| AV 在线热重载（866）/ native loader（I08） | 运行期发布语义，与 lowering 收敛正交 |
| 增量发布（Design43 §9） | 消费层级/闭包/复用策略，需独立设计 |
| CTE 复合扩面（C-39） | 817 只认标量；属执行语义扩面 |
| 完整 AJ/OSR/deopt | 长期方向，非本迁移前置 |

## 8. 风险与反控

| 风险 | 反控 |
|---|---|
| 迁移期"三套语义"并存（旧 native / mn / AV）且无人守差分 | 差分门做成**常驻 merge 门**，不是一次性脚本；每个 profile 切换前必须连续绿 |
| cache 混键导致旧产物被新后端误用 | 键版本化；同一逻辑产物单一键；跨后端共享条目在准入处拒绝 |
| ABI 微差（exit 70 vs typed trap、bool 规范、Str 布局）被差分门放过 | 差分门比对**类型化通道**（值/exit/trap kind+code/诊断 stage+code/资源台账），不只比"进程退出码" |
| 退役过早导致证据链断裂 | 退役前保留旧报告/金样归档；AV 永不退役，始终是独立 oracle |
| `ac-core` 反向依赖 `ac-mn` 造成环 | 结构性守卫：依赖清单断言 + review 检查项 |

## 9. 计划映射

| 计划 | 交付 | 门 |
|---|---|---|
| [PLAN-868](../plans/archive/868-hir-mir-lowering-shared-crate.md) | lowering 出库为 `auto-mir-lower`；三消费者；`MIR_LOWERING_VERSION` 进报告轴 | 等价（无行为变化） |
| [PLAN-870](../plans/870-ac-mn-record-text-native-expansion.md) | mn 扩 data/owned：record / text / text-record；关 `lower.rs` 的 3 处受控拒绝 | 三路差分（旧/新/AV） |
| [PLAN-871](../plans/871-ac-native-switch-and-old-lowering-retirement.md) | driver 层按 profile 切换；旧 HIR 直达 lowering 退役；脚本/金样/Spec 收口 | 五profile G-COV/shadow + G-DELETE + 资源分析保留 + 无旧lowering消费者 |
| 后续（未立项） | 旧方法共享lowering覆盖包；native第二阶段collections/variants+typed entry（D864+D871）；Dyn/发布/native loader各自专项 | 各真实D门及窗口，不预占编号 |

## 10. 登记与回写

- **Design43 §2/§10 已回写**（2026-10-28）：§2 迁移句子与 §10 "机器码持久化/共同编译服务"行指向本文，作为迁移合同的正式落点。
- **章号49登记已补齐（2026-10-28）**：[00-intro](00-intro.md)登记本设计及ACE续接专题；定点追加并保留他方未提交内容，下一域级章号按实际目录核验，不复用49。
- 本文不修改 Design43/44 的语义合同，只补"如何收敛"的工程合同；若收敛过程中发现语义需要变更，回 Design43/44 显式修订，不在实现中暗改。
