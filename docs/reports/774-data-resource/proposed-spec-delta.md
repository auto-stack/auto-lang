# PLAN-774 规范增量提案（proposed-spec-delta）

- 计划：PLAN-774 r3 · T-06 产出（AC-05）；**本文件是提案，不是 canonical 发布**。
  沉淀由 merge 阶段（T-08，`/auto-plan:merge`）按本表执行：specs.json upsert +
  模块回写 + `python scripts/spec-index.py`。
- 原则：四项 SD 只登记**合同与探针实证状态**，不新增任何 `implemented` 语言能力，
  不关闭 Gen1–3/GOAL-017，不改变既有 core/scalar profile 的支持/拒绝面。

## SD-01 add `docs/specs/auto-ac/data-resource-contract.md`

新文件（AC-01..04 的唯一下游来源）。内容骨架（源自 decisions.md §3 + 
interface-contract.md，merge 时全文搬运并按 Spec 模板重排）：

1. 状态行：experimental（合同 + 探针实证；实现仍仅探针）。
2. D-01 类型与宽度：i32 精确、char=u32 标量、内部长度 u32/指针机器宽、三转换码。
3. D-02 文本规则：UTF-8 存储、标量索引、len/len_bytes、char 名义 + 显式转码点、
   范围/NUL/空串/组合字符/非法 UTF-8/非法标量的码与阶段。
4. D-03 str/Str 逐语境表（11 行，含前门拒绝码）。
5. D-04 受限借/拥有前门规则 + 8 个 verify.* 码 + native-count-0 义务。
6. D-05 内部 ABI 五类（表 + 布局规则 + 无 sret + panic 不穿 ABI）；明示
   **非生产模块 ABI、非 ACC Cranelift 桥**。
7. D-06 资源：R1 裁定 + R2 淘汰（含实证引用）、预算/注入/清理/trap 边界、计数口径。
8. D-07 下游指针：770/771 身份与消费表、共同热点、合并顺序（引 interface-contract.md）。
9. 探针状态节：crate 路径、39 用例、PS 门禁、V-01..09 索引（引 verification.md）。

验收 ID：AC-01..04。

## SD-02 modify `docs/specs/auto-ac/project.md`

- before：能力门/边界节无数据资源合同条目；"char/str/List/完整 int 宽度仍 open"。
- after：新增一节「数据/资源合同（PLAN-774）」：指向 data-resource-contract.md；
  声明探针 workspace（`probes/data-resources/`，独立 Cargo workspace + no_std shim）
  及其非生产边界；**text（str/Str/char）与标量 record 的共同合同已在 774 冻结并经
  探针实证、实现层仍 open** 的双态表述（List 明确不在冻结面，仍为后续 owner）；
  旧句"仍 open"保留但加合同指针。不新增任何 implemented 语句。
- rationale：防混称（探针≠语言支持）；AC-05。
- 验收 ID：AC-05。

## SD-03 modify `docs/specs/auto-hir/stage-contract.md`

- before：阶段表 S0–S6 现状只覆盖 core/scalar 两域；无数据/资源阶段责任。
- after：在阶段表后追加「数据/资源阶段责任（PLAN-774，planned）」小节：
  S3 verify 承担 D-03/D-04 前门（8 码 + native-count-0 义务——八码均已**在探针模型
  中有可达入口并通过**，production verify 仍 planned）；S4 Checked 须携带资源义务
  （所有权/清理责任）且不泄漏 ABI 布局（沿 X6）；S5 lowering 按 D-05 五类形态
  （含嵌套 record 递归布局）；S6 执行器 deadline/containment 沿 741 合同。全路径
  清理（LIFO/嵌套出口/临时/move-in/return 转移）的证据状态为**探针实证**；
  未实现部分明确 planned。
- rationale：保持 verify=Checked 唯一门；AC-01/03。
- 验收 ID：AC-01/03。

## SD-04 modify `docs/specs/auto-acc/project.md`

- before：required 清单 str/char/record/List 无合同状态注记；unknown 族
  char↔int、宿主容器所有权（probe-rt-own）owner 为探针线。
- after：在「有限源码 profile 取证」与「标量扩面 profile 取证」两节后追加
  「数据/资源合同取证（PLAN-774）」小节：D-01..07 已在 774 冻结并经真实 COFF/PE
  探针实证（Rust AC 原型域）；required **不因合同冻结改项**（合同≠Auto 实现）；
  unknown 族 char↔int 在 profile 边界的规则已定（D-02：char 名义 + 显式转换），
  全语言宽度仍 open；probe-rt-own 由 774 的 D-06 资源合同部分回应（R1/R2 裁定），
  owner 更新为 770/771 实施线。不关闭 GOAL-017。
- rationale：双清单分离（required/unknown vs 合同实证）；AC-05。
- 验收 ID：AC-05。

## 覆盖核对

| AC | SD |
|---|---|
| AC-01 | SD-01（D01..07 全量）、SD-03 |
| AC-02..04 | SD-01（探针状态节引 verification.md 用例索引） |
| AC-05 | SD-01..04（边界/防混称/不冒称） |

无 retire 项；无对既有 canonical 语义的 before 变更（全部为 add/追加注记）。


## r3 校准（P774-R3-D1 随附；冻结 SD 刷新时的有效合同指针）

- SD-01/03 沉淀时应引用 decisions **§3.9 与 §3.10 为现行有效规则**（§3 及
  §3.8 中与之冲突的历史语句不搬入 canonical：跨行字面量合法、静态字面量
  返回合法、`text.literal-newline` 作废、预算 u64 无截断、V05-15 双 oracle）。
- 探针用例数以 verification.md 最新节为准（r3 后 63 项）。
- 矩阵计数：来源×模式矩阵 8 合法 + 16 非法。

## r1 复审修订（历史，已被 §3.9/§3.10 部分取代）

- SD-02：删除"char/str/List 已冻结"的误称——冻结面是 text（str/Str/char）与标量
  record 的共同合同；**List 不在冻结面**（非目标/后续 owner）。
- SD-01/03：claims 收敛为实际执行面——八码均有探针可达入口；全路径清理（LIFO/
  嵌套出口/临时/move-in/return 转移）以探针实证表述，不写为 production 已验。
- SD-01 内容骨架追加 §3.8 修订项（参数三模式、身份表、源级拼写、字面量/转义表、
  嵌套布局、预算零语义、R2 理由限定）。
- touched_goals=[] 的解释（计划 frontmatter 同步）：本计划不推进/关闭任何全语言或
  自举 Goal（GOAL-017 的 AAVM 谱系已达成不受影响；AC/ACC native 自举 Gen1-3 不因
  本合同开启），只交付 Rust 原型域的候选合同与探针实证，故无受影响 Goal ID。
