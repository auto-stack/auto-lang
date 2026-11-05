# auto-semantics 普通源码查询会话（SQ-A，934）

> **Status**: experimental（PLAN-934 冻结；实现 = `experimental/ace/crates/source-query/`）。
> 现行规则以本文为准；旧 `S796` 页语义、`ac796/*` 命名域与 796 探针**不变**，
> 934 是并行的新命名域，不占用也不改写字节含义。

## 定位与边界

- 934 位于 **core 绑定** 与 **consumers（AV/native/A2X/Rust/C）** 之间：
  依赖 core 公开 AST、796 中立求解器、语义契约与统一 artifacts；**无**
  AV/native/目标发射依赖。core 不依赖本层（方向单向）。
- 本层拥有：每 body 的**不可变事实页**、完整查询键、正负依赖、宿主安装协议、
  恢复准入与页持久化。**不**拥有：名字/作用域/签名绑定（core）、求解算法
  （复用 `ac796-bidir-worklist/1`，只扩输入不复制求解器）、HIR 发射/校验（core）、
  目标产物（各 consumer）。
- 有限面：Core/Scalar 的 i32/bool 普通函数/局部/控制流/直接调用/命名绑定。
  面外（record/enum/集合/闭包/text 方法等）是**受控拒绝**，不静默 Any、不回落旧推导。
- 与 935/936：本层不实现用户模板/CT 节点；935 实例、936 普通源码 CT 在其上叠加。

## 身份与版本（键）

生产/模式/算法/目标语义：
`producer=plan934/source-sq`、`schema=ac934/source-sq/v1`、
`profile=source-sq-scalar-i32/1`、`algo=ac796-bidir-worklist/1`（复用）、
`target=logical-only/v1`。

| 轴 | 内容 | 不入键 |
|---|---|---|
| 逻辑身份 | package + 真实 module + owner（`ac934/logical/v1` 哈希） | 928 目录 `content_sha`、目录 instance、ordinal |
| 签名轴 | 名字+参数名/mode/类型+结果（`ac934/fn-sig/v1`），**不含 body** | body 内容、span |
| body 轴 | 结构 token（kind/字面量值/名字/算子/实参绑定，`ac934/fn-body/v1`），**不含 span** | 绝对位置、空白、注释 |
| 能力集 | 调用方 capabilities（排序去重） | 调用顺序 |

- `QueryKey = query kind + logical + 签名轴 + body 轴 + profile/algo/schema/producer/target + caps`，
  canonical 为域分离长度前缀哈希；**同键不同 payload 必须拒绝并定位**，不靠清库。
- **逐查询正/负依赖**：
  - InferBody 正依赖 = 自身签名 + 实际读到的 callee 签名版本；负依赖 = 求解期
    "该 fn 名不存在/唯一"的候选集；不含无关 body/候选域。
  - FnSig 只读声明面；body-only 编辑不得使其失效。
  - 526/新增 record 字段闭包/SCC 属 935/后续，不在本层承诺。

## 事实页（`InferBodyFacts`）

- 自描述：`name`、`logical`、`body_version`、信封四元组、`revision`、`steps`。
- 节点事实按 **body 结构 id**（规范前序 walk）索引：`kind + PortableTy(i32/bool/void)`；
  binding 事实按声明节点，diagnostic 以节点引用（不持久化 span）。
- **不含** name/span/intern slot 作为身份；持久 payload 只有 portable 类型与名义引用。
- 准入（全部通过才可用）：信封（producer/schema/algo/profile）→ body 版本 →
  node id 存在/kind 匹配/无重复 → 覆盖每个表达式节点 → 无诊断。失败即拒，
  **不得**"查不到就推导"。

## 消费（事实驱动 HIR）

- core 保留绑定：`resolve_with_supply` 下每个 scalar 表达式/`let` 的 VT 由事实页安装，
  core 自身规则降为**校验**；不匹配/缺失 = bind 阶段 `source.fact-mismatch`/`source.fact-missing`，
  HIR 之前拒绝。旧 `compile_*` 入口保持原行为，仅作兼容/差分参照。
- 发射经 `emit_resolved_parts`（不 resolve、不 typecheck），随后真实 HIR 校验。
- 共享入口 `compile_via_facts`：parse → 每 body 事实 → 事实供给 bind → emit → verify → `CheckedModule`；
  有限面内与 legacy 入口产出**逐字节相同 HIR**（同一 checked 产物供四端消费）。
- 只读查询返回绑定本 revision 的 view；命中报告 ≠ 可执行替换。

## 宿主安装（两阶段）与并发

1. `prepare`：按完整键查/解事实，不安装、不发布、不建 target。
2. target-ready：需要可执行目标的调用方附加 ready 句柄。
3. `commit` 重验：取消标志 → base revision（迟到不装）→ 当前源完整键 → 全部正依赖版本。
4. 一次切换 source/语义 root/target 的**单一 current 记录**；任何失败保旧。
- 提交点前依赖变更必须失效/重试；提交点后丢 ack 按原请求身份幂等恢复，不称回滚。
- single-flight 按完整键：一次计算共享；waiter 取消不污染他人；同键异 payload 拒。
- `PageSink` 为唯一存储缝；统一 ManagedStore 适配（层 `S934`、carrier `SnapshotRecord`、
  页 codec 版本化 + 读回准入 + project soft root）已实现；required-closure/根注册策略随 929 协议。

## 恢复（R 轴）

- 来源/CST/声明恢复归 929；934 只负责 facts 页与其准入。
- R 只做 decode/深验（当前源 body walk → 页版本 → id/kind/覆盖/诊断 → 信封），
  **不调 solver、不重建语义 body 再称 verify**；缺必要页 = 拒绝该恢复车道，
  显式 cold 是另一请求，不静默降级。
- 恢复页可 `seed` 入会话（须通过同一深验且 logical 属本模块），使恢复后**首个编辑**
  只重解被改 body；坏首编辑的页带诊断被拒，修复后再出干净页。
- 计数器在恢复前安装；未改单元额外 I/O = 0。

## 预算

- 求解沿用 796 原门：5s / 1,000,000 steps；页 ≤512KiB、managed 16MiB、hot8+scratch8；
  A2X 48MiB 含本层新增保留物；正式 release 父窗 P90≤2000ms（fresh 冷准备分列）。
- 缺样本/伪造 0/越界即红；预算/取消是资源状态，不持久成语言诊断。
