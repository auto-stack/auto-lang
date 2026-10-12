# PLAN-767 规范增量提案（SD-01..05，review 定稿、merge 发布）

状态：**提案冻结**（work 阶段产出；canonical specs 与 ledger 在 merge 授权前不改动）。
依据：plan r2 §5.4；实现证据见 [verification.md](verification.md) 与
[design.md](design.md)。本文件不发布任何 canonical 文件，不修改账本。

## SD-01 add — docs/specs/auto-ac/source-scalar-i32.md（新文件）

- before：源码面仅存在 core-i32 一档规范（source-core-i32.md：`+ * <`、
  while、负字面量仅直接前缀数字、其余运算 source.unsupported）。
- after（新增）：`auto.source.scalar-i32.draft` revision 1 显式源码 profile：
  - 运算/优先级：`||` < `&&` < `== !=` < `< > <= >=` < `+ -` < `* / %` <
    一元 `- !` < 调用/分组；二元同级左结合、一元链右结合；
  - 精确语义：signed i32 精确计算，`+ - *` 越界运行期 trap70；`/ %` 商向零
    截断、余数与被除数同号或 0，除数 0、MIN/-1、MIN%-1 均 trap70；一元 `-`
    MIN trap70；比较产规范 0/1 bool；`== !=` 限同类（i32/i32 或 bool/bool）；
    `!` bool→bool；`&& / ||` 左侧恰好求值一次、RHS 条件执行；
  - 条件 for（r2 §5.2.1）：`for <bool-expr> { ... }` 每轮入口重判条件、
    初值 false 零次、continue 回条件、break 离最近循环；`while` 在本
    profile 报 source.unsupported；范围/集合/初始化/更新 for 拒绝；
  - 字面量域与 745 相同（[-2147483648, 2147483647]，`- 2147483648` 合法、
    孤立 2147483648 拒绝、`-(2147483648)` 拒绝、`-(-2147483648)` 可构建
    运行 trap70）；`a-1` 为二元减；一元 `+` 排除；
  - 静态合法性独立于运行路径：未执行的短路 RHS 仍查名字/类型；
  - 验收绑定：AC-01..04、06；证据 V-01/02/03/04/06/08/09/12。
- rationale：定义新 profile 而非改写旧语义；旧 core-i32 源码规范原文不动。

## SD-02 modify — docs/specs/auto-hir/project.md

- before：单 descriptor（auto.hir.core.draft/1/core-i32-draft）、三 op
  （add/mul/lt）、`hir::Bundle` 不携带身份。
- after：双身份域并存——core 域原样；scalar 域
  `auto.hir.scalar.draft / 1 / scalar-i32-draft`（schema/scalar-i32.atom，
  core 的超集）：BinOp 扩至 11（+sub/div/rem/le/gt/ge/eq/ne），新增
  `Unary{neg_i32,not_bool}` 与 `Logical{and,or}`（lazy，与 eager Binary 分离
  表示，禁止映射为双参 intrinsic 或无条件发射 RHS）；`overflow:` 字段仅
  add/sub/mul。Bundle 携带 `HirDomain`，由 descriptor 三重身份导出、随
  binder→verify→Checked→canonical writer 行走；verify 入口按域分设
  （`verify` 限 core、`verify_scalar` 限 scalar；core 域程序化混入新 op/
  节点 → verify.profile-op）；verify 增引用边界门（跨 body 引用
  → verify.bad-reference，受控拒绝非 panic）；canonical writer 身份冲突
  显式拒绝（canonical.identity-conflict）。
- 验收绑定：AC-01、04、05；证据 V-01/07、roundtrip。

## SD-03 modify — docs/specs/auto-hir/stage-contract.md

- before：S0–S6 仅按 core-i32 已实现面描述。
- after：追加 scalar 域的阶段事实——S0 前端接受 scalar 运算面与条件 for；
  S1 按文档头三重身份选择注册 descriptor（未知身份 bind.descriptor-unknown，
  不猜测、不回退）；S2 Bundle 携带域身份；S3 verify 按域入口与 profile 边界
  校验；S4 CheckedModule 记录域；S5/S6 共享尾部不变。条件求值规则：短路
  RHS 只能发射进条件执行块，命名实参先按源码序求值再按 bindings 落位；
  `for cond {}` 降级为 `loop { if cond { body } else { break } }`（每轮
  重判、条件不 hoist），不新增低层控制流模型。
- 验收绑定：AC-01、03..05；证据 V-01..05/12。

## SD-04 modify — docs/specs/auto-ac/project.md

- before：单一源码 profile（auto.source.core-i32.draft）与单一 HIR 身份。
- after：显式双 profile 消费者契约——CLI `--source-profile` 仅接受两个
  已注册字面量（缺失/未知 = 用法错 2；流水线拒绝 1；成功 0）；HIR check/
  build 按文档头身份分派；build-source 收据在同一发布事务内记录
  source-profile、HIR 三重身份（schema/revision/profile）、输入 hash 与
  后端/工具链；事务语义（失败不覆盖既有制品）对两域一致。
- 验收绑定：AC-01、06、07；证据 V-01/10/12。

## SD-05 modify — docs/specs/auto-acc/project.md

- before：MD-409/410（整数运算/比较与布尔短路类能力）required、open。
- after（只追加有限域结论，不改项状态）：记录 MD-409/410 在
  `auto.source.scalar-i32.draft / auto.hir.scalar.draft` 有限域内已有
  真实 PE 证据（signed 除/余截断与 trap、全序比较、eq/ne、一元、短路、
  条件 for；见 verification.md 收据与 hash）。完整语言 int 宽度、char/str、
  List、record/enum、ACC 主体迁移与 Gen1→Gen3 代际仍 open；GOAL-017 不
  关闭，不宣称 ACC Gen1 或自举达成。
- 验收绑定：AC-07、08。

## 边界与后续工作包（不改队列结论）

- char 索引/返回类型、str 资源协议、List ownership、完整 int 宽度保持
  open，由 §4.3 具名工作包（文本运行时首批 → 容器/聚合首批 → 首个真实
  ACC 词法组件）认领；本计划不裁定。
- 生产 ABI 线（NEXT-C）顺序、范围不变。
- 26–36 总量仍为历史工程估算；不以本计划速度推算完成日期。
