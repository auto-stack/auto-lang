# PLAN-743 验收矩阵与代际判据（acceptance-matrix）

> 计划：PLAN-743 r1 · T-05。锚点=后续实施计划引用的最小验收单元；
> 本计划**不执行**任何 native 编译——证据状态为 `existing`（已有收据可引用）、
> `ready`（条件齐备，实施计划可直接认领）、`gated`（有前置缺口，标明门槛）。
> 代际判据与阶段契约见 [auto-acc-bootstrap-contract.md](../../design/strategy/auto-acc-bootstrap-contract.md)。

## 1. 代表锚点（10 项 ≥ 8 要求）

| # | 锚点 | 输入 | oracle / 预期 | 运行形态 | 工具依赖 | 前置 capability | 状态 |
|---|---|---|---|---|---|---|---|
| A1 | 算术+循环微程序 | 741 fixtures `experimental/ac-core/fixtures/valid/*`（01-add、02-control 族） | 退出码/输出与 741 收据一致；溢出 trap=ExitProcess(70) | `ac-probe build+run`（Rust AC） | ac-probe、rust-lld、SDK（link.rs 发现） | i32 加/乘/lt、溢出 trap（741 已备） | **existing**（PLAN-741 归档收据） |
| A2 | 命名参数求值顺序 | `hir-examples/03-call-order.atom` 同语义源码版 | 求值顺序（左→右存临时）保持；重排=非法变换 X2 | 同 A1 | 同 A1 | eval_args 语义（741 已备） | **existing**（样例+拒绝矩阵已物化） |
| A3 | 错误诊断形态 | `fixtures/invalid/*` 拒绝矩阵 + 新增源码级反例 | `file:line:col: error[stage/code]` 渲染、span 指向冒犯 token | `ac-probe check`（退出 1/2） | ac-probe | 诊断基建（741 HIR 域已备） | **existing**（HIR 域）/ **gated**（源码域待 adapter） |
| A4 | token/lexer 代表 | `corpus_m1`（c01–c05 等）+ token.at/lexer.at 真实入口 keyword_kind/tokenize | 词元序列 golden 与 AAVM 基线一致（ACC 编译产物行为对齐） | ACC 源码→native 编译执行 | ACC 管线（Gen1 形态） | enum(MD-401)、str(MD-406)、List\<Token\>(MD-405) | **gated**（需 S0→S2 adapter + str/容器） |
| A5 | parser 代表入口 | `corpus_m2` 语料 + parser.at 游标/Pratt 核心 | AST 结构行为与基线一致（对齐口径=行为，非 S-expr 文本） | 同 A4 | 同 A4 | record/method/隐式 self（MD-403/404） | **gated**（同上 + AST 重写层） |
| A6 | 类型检查代表 | `corpus_m3` 型推断语料 | 类型推断行为 oracle=宿主 VM `.type` 输出（对齐口径为行为） | 同 A4 | 同 A4 | int 全序运算/bool 逻辑（MD-409/410）、record | **gated**（ACC typeck 新主体） |
| A7 | 文件/模块解析 | `corpus_use` + `shared_dep` 多文件用例 | 多编译单元链接后行为一致；use 边解析正确 | 同 A4（多模块形态） | 同 A4 | use-module/跨模块 DefRef（MD-413） | **gated**（Bundle 装配未实现） |
| A8 | 字符串/容器微程序 | 待建：str 拼接/比较/切片 + List push/get/遍历的最小源码集 | native 执行输出与宿主 VM 一致 | 同 A4 | 同 A4 | str/list（最高优先缺口） | **gated**（语料待建；实施候选①首任务） |
| A9 | is-match / enum 展开 | 待建：对 TokenKind 的 is 分支 + Val 式 payload 变体 | 分派行为一致；未知变体显式拒绝 | 同 A4 | 同 A4 | is-match lowering、enum(MD-401/402) | **gated** |
| A10 | 代际自编译 | ACC 主体源码（本盘点清单域） | Gen1→Gen2→Gen3 语义/诊断/HIR 等价（contract §5） | ACC 编译 ACC | 全管线 + 桥 | §4.1 全部 required 能力 | **gated**（终点门禁） |

**不作为门禁**：任意构建字节固定点（战略 §4/契约 §5——仅在可复现条件约定后为附加门）；
A2R 转译中转产物不作为任何锚点的 native 证据。

## 2. 桥接候选的验收关联（AC-06）

两候选（进程内 C ABI / 独立后端进程）的边界、选择条件与职责矩阵在契约 §4；
实施计划须补齐的**待验证问题**：

1. A 形态单次调用延迟与批量分块开销实测（含句柄/缓冲释放路径）。
2. B 形态进程启动成本、Atom/Batom 载体定型进度对协议握手的影响。
3. 能力拒绝语义等价性：桥两侧的 `capability.missing` 行为与 ac-probe CLI 一致。
4. 错误通道保真：后端阶段码无损传回诊断渲染层（不降级为字符串拼接）。

## 3. 代际矩阵（AC-06 / A10 展开）

| 检查项 | Gen1 | Gen2 | Gen3 |
|---|---|---|---|
| 编译者 | Rust AC 工具链 | ACC 第一代 | ACC 第二代 |
| 被编译物 | ACC 全部主体模块（inventory §1/§2） | ACC 自身源码 | ACC 自身源码 + 全语料回归 |
| 等价域 | 产物可运行 | 与 Rust AC 产物语义/诊断/HIR 等价 | Gen2=Gen3 不动点 |
| 依赖清单归档 | 桥/链接器/宿主内建版本表 | 同左+自比差异说明 | 同左 |
| 字节固定点 | 不要求 | 不要求 | 仅可复现条件约定后附加 |

## 4. 锚点→验收标准映射

A1–A3 → AC-06（含 741 基线复用）；A4–A9 → AC-06 + capability 表（inventory.md §4.1）；
A10 → AC-06 代际；A8/A9 语料建设 → next-work-packages.md 候选①；桥待验证问题 → AC-06
与契约 §4。每个 gated 锚点的解锁条件都能在 next-work-packages.md 的两候选或 ABI 线找到 owner。
