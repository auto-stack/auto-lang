# PLAN-774 决策包：AC 数据扩面的共同类型、资源与内部调用契约

- 计划：PLAN-774 r3（work 阶段；AC-01/04/05 证据）
- 工作树：`D:/autostack/.wt/lang-774/auto-lang` @ `plan-774-dev`
- 状态：T-01 事实差异表已建；D-01..07 候选/裁定/oracle 由 T-02 冻结（§3 起为 T-02 产出）
- 边界：本包是**合同决定**，不是语言 feature 交付；探针（Rust/Cranelift）是可行性证据，
  不是 Auto native 编译证据。生产 ABI、完整借检查器/GC、完整 int 宽度不在本包。

## 1. 基线与输入刷新（T-01 收据）

- 实施起点主检出/工作树基面 HEAD：`5e21561d2c303a02319ff7a29e4607933e09a38f`
  （= master；r3 引用核查基线 `d777ccbb` 之后仅 `docs/plans/**` 与设计登记簿推进）。
- 输入指纹刷新：对 [r3 输入表](../../plans/attachments/774/reference-update-inputs-20261013.tsv)
  全部 47 项 `origin=main` 输入在实施基线复核 SHA256——**46/47 零漂移**；唯一漂移
  `docs/design/00-intro.md`（33/36 两行状态描述簿记更新，无语义影响，设计正文未动）。
  语义/代码类输入（auto-ac 三 Spec、auto-hir 两 Spec、auto-acc project、types/runtime
  overview、Design 02/04/10/36、745 数值边界、aavm lexer/token/parser、ac-core
  Cargo/源码五件、770/771 计划）全部与 r3 一致。
- 772 前置确认：`docs/plans/archive/772-ac-scalar-ops-short-circuit.md` 在档（archived），
  scalar Spec（source-scalar-i32.md revision 1）与 verify/native 扩面证据已沉淀——
  与计划 §4 前置一致，无未清偿项。
- 唯一树确认：`git worktree list` 本计划唯一树 `D:/autostack/.wt/lang-774/auto-lang`
  （branch `plan-774-dev`）；未从 772 旧实施树复制任何内容。主检出在会话开始时仅有
  `docs/plans/**` 簿记改动（776 计划文档），无代码路径 WIP，符合 master-zero-WIP。
- 本任务零生产代码改动（仅新建本报告文件）。

## 2. 事实差异表（现行要求 × 真实观察；T-01 冻结）

取证输入（hash 绑定于 §1 指纹表；行号按该版本）：

| 代号 | 来源 | 关键事实 |
|---|---|---|
| F1 | docs/specs/auto-ac/source-scalar-i32.md:15-31 | 新 profile 链 int/i32=精确有符号 32 位；Bool/I32 不隐式互转；`/ %` 0、MIN/-1、MIN%-1 trap70 |
| F2 | docs/reports/745-source-core-i32/numeric-boundary.md §2-§4 | 旧宿主宽度分裂：VM 栈 Int(i32)、值存储 i32/i64/u64 并存、旧 AST 按幅值三态、trans 目标 `Type::Int→i64`；无单一全语言 int 宽度 oracle |
| F3 | packages/aavm/lib/lexer.at:44-52 | `cur_char(source str, p int) -> int`：游标 p 是 int，`p >= source.len()` 越界返回 0；`source.char_at(p)` 返回 **int 码点**，随后与 `'0'`/`'9'` 字面量按 int 比较——旧 auto-lib 表面 char 与 int 隐式混用 |
| F4 | 745 numeric-boundary §5（MD-408） | 同一"取字符"三宿主表示并存：VM builtin `str_char_at` 返回**单字符 str**、auto-lib 返回 int 码点、ValueData 存 Rust `char`（Unicode scalar）；不存在统一 char↔int 映射可作 oracle |
| F5 | docs/design/02-type-system.md §String Type System | 三分：`str`=借用 slice（**变量/参数默认**）、`Str`=拥有（**容器元素仅限 Str**）、字面量非命名类型；单向自动提升 lit→str→Str，反向显式 `s.to(str)` |
| F6 | docs/design/02-type-system.md §Rust Transpilation Mapping | a2r 实况：str 仅**参数位**映射 `&str`，变量/字段/返回/容器一律映射 `String`（拥有）——与 F5"str 是变量默认"在存储位存在实现性分歧 |
| F7 | packages/aavm/lib/token.at:33-37 + parser.at:70-80 | 真实消费者形态：`Token { kind TokenKind, text str, len int, adv int }`；`P { toks List<Token>, ... }`——记录内 `text str`（借用）出现在容器元素中，与 F5 容器拥有制在旧表面并存（未爆雷因旧无借检查） |
| F8 | docs/design/04-memory-ownership.md §Trinity/§Move | view（只读借，双侧默认）/mut（读写借，双侧显式）/move（全权转移，源死后禁用）三一；clone 显式带 `()`；资源类型 `=` 赋值禁、须 `.move`；move 参数函数内隐式可变 |
| F9 | docs/design/04-memory-ownership.md Status（PLAN-667 修正） | `ownership/`（borrow.rs/lifetime.rs/cfa.rs）是**未接线设计探针**，零生产调用点；现行安全边界=VM 运行期守卫 + a2r 逃逸分析保守超集——"编译期捕获内存错误"是目标不是现状 |
| F10 | docs/design/04-memory-ownership.md §Parameter Passing | 平凡类型（int/float/bool/char/byte）传值（寄存器拷贝），重类型传引用——语义 view、实现 copy/reference 是 ABI 关注点 |
| F11 | docs/specs/auto-ac/project.md:16-18,44-52 | 现行 native 面：Windows x64 WindowsFastcall、bool 传参 i32 0/1、无堆/无 buffer/无 struct ABI；intrinsics 按符号 import；`hir.test.trace.v1` 测试能力 = test-support no_std 静态库直写 stderr，非生产 I/O 契约 |
| F12 | docs/specs/auto-hir/stage-contract.md（R1-R6/X1-X8） | 阶段身份随行、verify 是 Checked 唯一门、效果语义双向禁止、trap 不得提升；X6 公共 HIR 不泄漏低层布局 |
| F13 | docs/specs/auto-acc/project.md §能力现状 | required 清单含 str（最高优先）/char/record/List/方法/隐式 self；unknown 族含 char↔int、宿主容器所有权（probe-rt-own）——**owner 即本计划线** |
| F14 | docs/design/10-language-syntax.md §OOP | 方法在 `type` 块内：`static fn`（类型级，无 receiver）与 `fn`（实例级，`self` 可选、`.field` 直达）；`mut fn` 声明修改实例；a2c 惯例 `TypeName_methodName(TypeName* self)` |
| F15 | docs/design/36-ac-data-resource-expansion.md §3 | 本批候选 text 规则：UTF-8 存储、Unicode 标量索引（非字节/字素）、char 独立名义类型、显式转码点、无隐式 char/int 混用 |

按决定 ID 的事实差异与适配要求：

| 决定 | 现行要求/意图 | 观察差异（证据） | 适配要求（新 profile 侧） |
|---|---|---|---|
| D-01 int 精度与内部宽度 | 新 profile 沿用精确 i32（F1） | 旧宿主四宽度并存（F2） | 源语言 int 恒 i32；**内部地址/byte length 独立类型（非源 i32）**；i32→内部长度转换显式且负值/越界失败 |
| D-02 文本索引与 char | 标量索引、char 名义类型、显式转码点（F15） | 三宿主 char 表示分裂（F4）；旧 auto-lib char/int 隐用（F3） | 新 text/record source 与 HIR 三元组**精确注册**；char 不隐式转 int；char_at 类操作按 D-02 冻结规则重定义，不照抄任一宿主 |
| D-03 str/Str 逐语境 | 三分 + 单向提升 + 容器拥有制（F5） | a2r 存储位一律拥有（F6）；旧 lib 记录内 str 入容器（F7） | 首批按 D-03 表逐语境冻结；容器借用禁令保留；记录内文本拥有制留给 text-record 组合 profile（771 首批字段仅 i32/bool，冲突不触达） |
| D-04 受限借/拥有 | view/mut/move 语义目标（F8） | 借检查器未接线（F9） | 前门（verify 层）受限规则：不支持的形态**显式拒绝**；不以运行期守卫或旧 ownership 模块存在充当安全证据 |
| D-05 内部 ABI | 平凡传值/重传引用意图（F10） | 现行 native 面无 buffer/struct ABI（F11） | 五类内部形态在探针冻结宽度/对齐/caller-callee 责任；非生产模块 ABI、非 ACC Cranelift 桥 |
| D-06 资源与失败 | 无既成承诺（F11：无堆） | 无 | 分配器/所有权策略、失败注入点、正常清理与 fatal trap 的 OS 回收边界、预算单位/入口/耗尽类别全部在本包冻结 |
| D-07 下游接口 | 770/771 无未决必需前提（Design 36 §5） | 共同热点清单已列（source/hir/descriptor/verify/native/main） | D-07 交付最小接口/profile/API/热点清单与合入顺序；语义定义一处 |

## 3. 决策 D-01..D-07（T-02 冻结后回填）

（T-02 产出：每项 ID 含候选、选择理由、支持/拒绝表、oracle、能力/profile 身份、下游责任。）

## 4. 下游消费表（T-05 冻结后回填）

（D-07：770/771 最小接口/profile/API/共享热点清单及合入顺序。）
