# PLAN-774 决策包：AC 数据扩面的共同类型、资源与内部调用契约

- 计划：PLAN-774 r3（work 阶段；AC-01/04/05 证据）
- 工作树：`D:/autostack/.wt/lang-774/auto-lang` @ `plan-774-dev`
- 状态：T-01 事实差异表已建；D-01..07 候选/裁定/oracle 由 T-02 冻结（§3 起为 T-02 产出）；
  **r1 复审修订**见 §3.8（needs_fix 六发现的合同修订，2026-10-13）
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

## 3. 决策 D-01..D-07（T-02 冻结）

通用裁定原则（全节适用）：本节所有规则是**首批（770 文本 / 771 record）合同**，
在探针（T-03/04）中冻结 oracle 并以真实 COFF/PE 调用取证；探针通过不改变
"实现仍仅探针"的边界（AC-05）。涉及既有语言承诺的变化：无——本节所有决定
均落在 F1-F15 已授权的 profile 边界内，无一项需要修订现行 canonical Spec 语义。

### D-01 整数精度、内部宽度与转换

**裁定**：新 text/record source profile 与 HIR 域 int/i32 = 精确有符号 32 位
（沿 core/scalar，F1）；char 名义类型内部为 **u32 Unicode 标量值**（宽 4、对齐 4，
ABI 通道以 I32 位型承载）；**内部地址/字节长度/标量计数独立于源 i32**——
内部字节长度与标量计数为 u32，指针为机器宽度（x64 = 8 字节）。

| 转换 | 规则 | 失败类别/码 |
|---|---|---|
| 源 i32 → 内部长度 | 需 0 ≤ v；i32 域内自动满足 u32 上界 | 负值：运行期 `res.len-negative`（-2） |
| 内部长度(u32) → 源 i32 | 需 ≤ 0x7FFF_FFFF | 超界：`res.len-too-large`（-3） |
| 长度加法（如 concat） | i64 中间量，结果须 ≤ i32 通道上界 | 越界：`res.len-overflow`（-1），**先于任何分配与缓冲读取** |

不宣称 i64/usize 是源语言类型（F2 分裂不由此收敛）；探针仅证明上述边界可实现。

- 候选与拒绝：候选①内部长度用 i32（源同型）——拒绝：正确定义负长度无意义且
  concat 和提升需要 >2^31 空间；候选②u64 内部长度——拒绝（首批）：全部 ABI 通道
  4 字节足够，加宽无消费者。裁定 u32。
- oracle：V-07（byte/scalar 计数）、V-05（三转换码）；下游责任：770/771 的
  int 消费面不得把内部长度当 i32 直接回传。

### D-02 文本表示与 char 名义身份

**裁定**（沿 F15 候选，核查 F3/F4 后无推翻事实）：

1. 文本 = Unicode 标量值序列，**UTF-8 字节存储**；语言级文本值恒为合法 UTF-8
   （非法 UTF-8 字节序列只能作为探针对抗输入存在，运行期解码失败码
   `text.invalid-utf8`（-7），记录字节偏移）。
2. **索引单位 = Unicode 标量**（非字节、非字素）。默认长度 `len()` 返回标量计数；
   字节长度经显式 `len_bytes()`。字位/组合字符不聚类：`e`+U+0301 是 2 标量/3 字节，
   slice 可切在组合序列中间（首批无字素支持，明示非遗漏）。
3. **char 独立名义类型**：`char_at(i)` 返回 char；char 与 int 无隐式互转（F3 的
   `'0' <= c <= '9'` int 比较面**不采纳**）。显式转换：`char.to_codepoint() -> i32`
   （恒成功）；`char.from_codepoint(i32)` 可失败——代理区 [0xD800,0xDFFF]、负值、
   >0x10FFFF 拒绝，码 `text.invalid-scalar`（-8）。字面量形态的非法标量属 770 的
   词法层前门拒绝，774 冻结的是构造边界规则。
4. 范围规则：slice/char_at 的标量下标须满足 0 ≤ start ≤ end ≤ scalar_count；
   start>end → `text.range-reversed`（-9）；越界 → `text.range-oob`（-10）；
   **空范围 [s,s) 合法**（len 0）。NUL(U+0000) 是合法标量（1 字节），非终止符。
   空串合法（0 字节/0 标量）。
5. char_at 旧三宿主差异（F4）不升级：新 profile 返回 char，不改用单字符 str
   （VM builtin）或 int 码点（auto-lib）。

- oracle：V-07 全表（fixture/期望值见 probe-cases.md §V-07）；下游责任：770 实现
  len/len_bytes/char_at/slice 语义必须逐字消费本表。

### D-03 str/Str 逐语境规则

**裁定**（按 F5 三分 + F6/F7 实况收敛；容器拥有禁令保留）：

| 语境 | 首批规则 | 前门拒绝码 |
|---|---|---|
| 字面量 | 借用视图（指向常驻数据）；提升为 Str = 一次分配+拷贝（AP-4） | — |
| 局部 str | 借用视图：指向字面量或拥有者的 slice；不拥有不清理 | — |
| 局部 Str | 拥有：构造分配/接收转移；出口唯一清理（LIFO） | — |
| 参数 str | 只读借用（view）；callee 不得保存/释放 | — |
| 参数 Str | **move 语义**：调用点 `.move`，实参死后禁用；callee 负责清理 | 缺 `.move`：`verify.ownership-required` |
| 返回 | 拥有 Str = 所有权转移给调用者；借用 str = 仅允许借自**参数或字面量** | 借自局部：`verify.borrow-escape` |
| record 字段 | **仅 Str**（拥有）；str 字段首批禁止 | `verify.field-borrow` |
| 容器元素 | 仅 Str；`List<str>` 既有禁止保留（F5） | `verify.container-borrow` |
| clone | 显式 `s.clone()`：独立分配（AP-2），计数独立 | — |
| 显式转换 | `s.to(Str)` 分配拷贝；`s.to(str)` 借用化不转移 | — |
| move | `s.move`：源死后禁用，清理责任随所有权走 | `verify.use-after-move` |

- 观察差异处理：F6（a2r 存储位一律拥有）是实现性分歧，不改变 F5 语义；新 profile
  按语义裁定，a2r 不受影响。F7（旧 lib 记录内 str 入容器）记录为旧表面既有事实，
  首批 771 字段仅 i32/bool 不触达；text-record 组合 profile 必须按本表用 Str 字段。
- oracle：V-03（拥有/借用/clone/move 计数）、V-04（逃逸/容器拒绝）；下游：770 逐行
  消费，770 域内 str/Str 类型检查按本表前门执行。

### D-04 受限借/拥有规则（前门）

**裁定**：借/拥有约束在 **verify 层前门**执行（F9 的旧 ownership 模块与运行期守卫
不作安全凭证）；不支持的形态显式拒绝，不静默放行也不转运行期。

- 所有权源：let Str 局部、拥有构造、move-in 参数、调用返回、字面量（不可变常驻）。
- 活跃借用：view 在词法区域内活跃；活跃期间禁止 move/清理 owner、禁止 mut 借用
  owner；**多个只读 view 可并存**（首批允许）。
- move 后禁用：`x.move` 或 move-参数调用后，x 任何使用（含再 move/再借）前门拒绝。
- mut 排他：至多一个活跃 mut 借用，且期间无任何只读 view。
- 唯一清理：每分配恰一个清理责任方；owned 局部在出口（return/break/continue/
  自然收尾）按 **LIFO** 生成清理；move 使源路径死、清理责任转移。
- 临时延长：临时拥有值存活至所在完整语句结束（最后使用不早于语句尾）；返回转移
  将其移出。首批无闭包/捕获，逃逸仅经返回与存储位。
- receiver 规则（771）：只读实例方法 receiver = view 借用；显式 mut 自由函数
  receiver = mut 参数（排他借用）；**可变实例方法（`mut fn`）首批不支持**（F14 留后续）。

**模型拒绝码冻结**（探针规则模型与未来 verify 同名）：`verify.use-after-free`、
`verify.double-move`、`verify.use-after-move`、`verify.mut-conflict`、
`verify.borrow-escape`、`verify.field-borrow`、`verify.container-borrow`、
`verify.ownership-required`。模型拒绝时**目标操作 native 调用计数为 0**（V-04 证据）。

### D-05 内部 ABI 五类形态（探针冻结，非生产模块 ABI）

平台：Windows x64 / WindowsFastcall / Cranelift 0.126（F11 现状之上）；整型/指针
参数经 RCX/RDX/R8/R9，其后栈传；返回 ≤8 字节经 RAX。**不使用 >8 字节聚合返回**
（避免 sret 隐藏参歧义）：拥有 buffer 的指针走 RAX、长度走显式 out 参。

| ID | 形态 | 机器表示 | 大小/对齐 | caller 责任 | callee 责任 |
|---|---|---|---|---|---|
| ABI-A | i32/bool/char 值 | I32 按值（bool 规范 0/1；char = u32 标量位型） | 4/4 | 传值；bool/char 传前已合法 | 只读不改写 |
| ABI-B | 借用 buffer | (ptr: I64, len: I32) 双参 | 8/8、4/4 | 借用期有效且内容稳定 | 只读；不保存/释放/越界 |
| ABI-C | 返回拥有 buffer | 返回 ptr: I64；out 参 `*mut I32`（成功=长度≥0，失败=负状态码） | 8/8、4/4 | 取得即拥有；负责唯一清理 | 成功分配+填充；失败返回 null+负码 |
| ABI-D | record view | 单参 `*const Rec`（I64） | 按 D-05 布局表 | 借用期有效 | 只读字段访问 |
| ABI-E | record mut / out-return | mut：`*mut Rec`；out：caller 预置存储 + callee 全字段写入 | 同上 | mut 排他借用；out 提供存储 | mut 仅经此指针改写；out 完整初始化 |

record 布局规则（首批）：字段按声明序、各字段自对齐 4、size = 4×n、对齐 4，
offset(fᵢ) = 4·i；无重排无尾随填充。错误与释放责任：预期失败一律**值返回**
（负状态码），panic 不穿 ABI（shim 无展开）；ABI-C 失败时已分配资源由 callee
回收（live=0），成功时清理责任在 caller。

- 候选与拒绝：候选①聚合 {ptr,len} 按 sret 隐藏参返回——拒绝：Win x64 sret 语义
  依赖类型类别分类，Cranelift IR 无聚合类型，跨工具链手工复刻易错且无首批消费者；
  候选②（裁定）指针走 RAX + 长度显式 out 参——通道无歧义、探针可逐位断言。
- oracle：V-02（哨兵/布局/trace/异质顺序控制）；下游：770 消费 A/B/C，771 消费
  A/D/E（consumer 表见 §4）。

### D-06 资源策略、失败注入与终止边界

**裁定（首批 = 候选 R1 显式拥有/借用）**：

- 分配器：进程堆 kernel32 `GetProcessHeap`/`HeapAlloc`/`HeapFree`；不复用 Rust
  String/Vec 私有布局；无自定义分配器。
- 所有权：单拥有者；转移仅经 ABI-C 返回、move、clone（新独立分配）。
- 正常清理：作用域出口 LIFO；分支/循环/早退全路径唯一清理。
- **预算**：单位 = **存活字节数（live bytes）**；入口 = 每次分配请求在长度校验后
  检查 `live + req ≤ budget`，违反即拒绝（码 `res.budget`，-4），**不触分配器**；
  释放后额度即时回收。无固定语法/嵌套深度上限引入（拒绝码集合中不存在深度类码）。
- 失败注入点（探针）：`fail_nth_alloc(n)`——第 n 次**分配器调用**返回 null
  （模拟环境失败）；`set_budget(bytes)`。声明申请点：AP-1 拥有构造、AP-2 clone、
  AP-3 concat 构建、AP-4 字面量提升（各点故障预期见 probe-cases §V-05）。
- 失败类别（运行期状态值，与编译期拒绝、fatal trap 三分）：
  `res.len-overflow`(-1)/`res.len-negative`(-2)/`res.len-too-large`(-3)/
  `res.budget`(-4)/`res.env-oom`(-5)/`res.shim-error`(-6)。
- 零长度：合法；拥有零长 = null 指针 + len 0，**不产生分配器调用**，free(null) 无操作。
- fatal trap：进程级致命终止（硬件异常/abort）→ OS 回收全部进程内存；**不声称
  执行析构、不承诺 unwind**；trap 前驱可观察效果（trace）顺序保持（F12 R3）。
- **候选 R2（受限引用计数句柄）淘汰**：句柄表+rc 使每次值复制成为潜在所有权事件，
  无法提供 D-04 要求的前门性质——move 禁用、mut 排他、逃逸拒绝都需要静态
  所有权/借用状态，rc 只能在运行期"发现"（或静默原谅）违例；budget 记账在共享
  句柄下归属不明。探针以 R2 最小族（h_alloc/h_clone/h_drop/h_get）实证：drop 后
  复用句柄是**静默错读**（读回收槽位新内容、无任何错误码），对照 R1 前门
  native-count-0 证据，构成淘汰依据（V-04）。`shared`/跨任务 ARC 属 Design 04
  后续线，不由此重开。

### D-07 下游接口与合入顺序（摘要；明细表 = T-05 interface-contract.md）

- 770（文本）消费：D-01、D-02、D-03、D-04、D-05 A/B/C、D-06 全部；新增身份
  `auto.hir.text.draft / 1 / text-i32-draft`（770 注册）。
- 771（record）消费：D-01、D-04 receiver 规则、D-05 A/D/E；i32/bool record 无堆
  资源，D-06 仅交互面（无拥有 buffer）；新增身份 `auto.hir.record.draft / 1 /
  record-i32-draft`（771 注册）。
- 共同热点（两分支都改）：source AST/typecheck、hir、descriptor、verify、native、
  main；语义定义一处（本包 + 770/771 各自 Spec），**不得各自造通用类型系统**。
- 合入顺序：774 复审合入 → 770/771 并行实施 → 先完成者先合入，后完成者在
  新整合基线重放自身全门禁 + 首个 profile 回归；非文本合并冲突（身份/签名/Drop/
  能力门不兼容）暂停依赖实现并 revision。
- 组合 text-record、List、enum、use、生产 ABI、ACC 桥：后续 owner（Design 36 §7），
  非本包遗漏。



## 4. 下游消费表（T-05 冻结）

## 3.8 r1 复审修订（needs_fix 六发现的合同修订；P774-R1-01..06）

本节修订是第一轮独立复审的修复产物，**修订后的规则取代 §3 中与之冲突的表述**；
未提及的 §3 条款不变。逐发现：

**R1-01（资源账本/预算）修订 D-06**：

1. 资源账本为**完整外部账本**（堆支持的可增长登记表，键=用户指针，值=(长度,资源id)），
   每次成功分配登记、每次释放精确移除；**无容量上限、不静默丢记录**——账本增长失败
   时该分配按 env-oom 回滚。`live_bytes` 由账本背书，释放**即时**归还预算额度。
2. `check_invariant` 语义收紧：allocs==frees ∧ live==0 ∧ **live_bytes==0** ∧
   **invalid_frees==0** 才返回 0。
3. **预算 0 裁定**：`set_budget(v≥0)` 为真实字节预算，**0 = 零字节预算**（任何非零请求
   拒绝，零长请求仍合法）；负值清除上限。原"0=无限"的隐含缺省作废。

**R1-05（释放口径）修订 D-06 计数定义**：`frees` = **成功释放数**（外部账本验证 +
HeapFree 返回非零）；新增 `free_attempts`（非空 free 调用数）与 `invalid_frees`（账本
未命中=双释放/外来指针，或 OS 拒绝释放）。双释放检测**不读取已释放内存**（账本验证
先行）。双释放负控：invalid_frees=1 → invariant 失败 → exit 1。

**R1-02（借用返回来源）修订 D-03/D-04**：模型/前门必须追踪**资源来源**：
局部拥有（Make）、借入参数（ParamBorrowed，owner=调用方）、move-in 拥有参数
（ParamOwned，owner=本帧）。`return <view>` 合法**当且仅当**来源是存活的借入参数；
已释放局部（use-after-free）、move-in 参数（越过本帧清理边界=逃逸）均拒绝。合法兄弟
例必须把借用 (ptr,len) **真实跨调用帧返回并在调用方读取**。`verify.field-borrow`/
`verify.container-borrow`/`verify.ownership-required` 均有可达拒绝入口与合法兄弟
（field/container：存 view 拒绝、存 owned clone 合法；ownership-required：无 `.move`
传给 move 参数拒绝、显式 `.move` 转移且 **callee 负责清理**）。

**R1-03（清理顺序/形态）修订 D-04/D-06 证据义务**：出口清理顺序为 **LIFO**，以资源
id 的 FREE trace 为顺序 oracle；嵌套 break/continue/return 出口、临时值"存活至语句
尾"（使用先于清理的 trace 顺序）、move-in 参数由 **callee 清理**、被调产出的 buffer
**return 转移**回调用方，均须有代表性真实 PE 证据；每个申请点（AP-1..4）须有
**带存活 owner** 的注入例；clone 独立性须含**内容控制**（原 buffer 释放后 clone 内容
仍完整）。

**R1-04（共同决定缺项）修订 D-02/D-03/D-05/D-07**：

1. **Str 参数模式修正（D-03）**：原"参数 Str=move"一刀切与 Design 04/F8 冲突，作废。
   修正：参数默认 **view**（str 与 Str 同），三种显式模式分列：
   - `str` 参数（默认 view）：只读借用，callee 不得保存/释放/逃逸；
   - `Str` 参数（默认 view）：借用视图，同上约束；**显式 `mut Str`**：排他可变借用；
   - **显式 `move Str`**：所有权转移入函数（调用点须 `.move`），callee 负责清理，
     调用后实参禁用；缺 `.move` 前门拒绝 `verify.ownership-required`。
   a2r 的 String 存储位映射是实现事实，不替代源语义（F6 记录在案）。
2. **身份同步（D-07）**：唯一身份表=interface-contract.md §7：
   `auto.hir.text.draft / 1 / text-i32-draft` 与 `auto.hir.record.draft / 1 /
   record-i32-draft`。**770 §5.1（现为 text-draft）与 771 §5.1（现为 record-draft）
   必须在其下次 revision 采用本表**（下游责任，774 不代改两计划文件）。
3. **字符转换拼写（D-02/D-07）**：机器 shim API（p774_char_from_codepoint）与源语言
   拼写分层。源级拼写冻结：`c.to_codepoint() -> int` 与 `char.from_codepoint(n)`；
   **770 §5.1 的 `.to(int)` 需在其 revision 改齐**（下游责任）。
4. **字面量拼写/跨行/转义（D-02 补冻结表）**：
   - 普通字符串字面量单行；**跨行字面量首批拒绝**（引号内出现源换行 → 前门
     `text.literal-newline`）；字面量内容不含裸换行。
   - 行尾：源文件 LF/CRLF 均为合法行界（沿 745 UTF8/CRLF 保持），不影响字面量内容。
   - 转义白名单：`\n \t \\ \" \0 \u{XXXX}`（XXXX 为 1-6 位十六进制 Unicode 标量，
     代理区/越界 → 前门 `text.invalid-scalar`）；**其它转义序列 → `text.invalid-escape`**。
   - char 字面量恰一个标量：`'ab'` → `text.char-multi`；`''` → `text.char-empty`。
   - 诊断责任：词法层（770 实现）产上述码；774 冻结码与边界，不实现 parser。
5. **嵌套 record 布局（D-05 补规则）**：递归布局——`align(R)=max(字段对齐)`；
   `offset(首字段)=0`，其后字段按**声明序**自对齐放置；`size(R)=round_up(尾偏移+尾宽,
   align(R))`；嵌套 record **inline 值语义**（无指针，赋值=拷贝）；out-slot 互不重叠。
   尺寸算术用 u32 checked，溢出 → 前门 `record.layout-overflow`。探针物化：
   Inner{2×i32}=8/4；Outer{inner@0..8, flag@8}=12/4（rec2_out/rec2_view）。
   771 的非递归嵌套 record 承诺由此承接，不降级为后续。
6. **R2 淘汰理由限定（D-06）**：淘汰依据限定为**该候选的复杂度/前置**——要满足 D-04
   前门性质，句柄层必须补 per-slot active 追踪 + generation + 静态所有权信息
   （h_get_checked 对照实证），实质退化为在运行时表上重建 R1 的静态所有权模型，且
   预算记账在共享句柄下归属不明。**不声称"引用计数句柄方案本身不可能"**。R1 选择
   保留。

**R1-06（门禁/收据）修订验证义务（verification.md）**：逐行警告匹配须带正反控制；
check/test/fmt 原文全量落盘；失败也写收据；shim 构建纳入 60s+locked 执行器；V06-2
前驱 trace 经文件重定向可观察（deadline 分支丢弃管道输出是执行器契约，文件是补充
观察通道，不是绕过 containment——wrapper 与 PE 同在受控进程树内）。

## 4. 下游消费表（T-05 冻结）

明细见 [interface-contract.md](interface-contract.md)：冻结符号表（p774_* 全集与
机器签名）、状态/拒绝码注册表、ABI 类别与布局消费面、资源表示与计数口径、
770/771 逐条消费表、共同热点清单（source/hir/descriptor/verify/native/main）与
合并顺序（774 → 770/771 并行、先成先合、后成者重放全门禁+首个 profile 回归、
非文本冲突回 new 修订）。组合 text-record/List/enum/use 为后续 owner，
两分支单独通过不构成组合能力。

## 3.9 r2 复审修订（P774-R2-01..04；修订 §3.8 中与之冲突的表述）

**R2-02 合法形态恢复（撤销 §3.8 R1-04.4 的越权缩减）**：

1. **跨行字符串字面量合法**（恢复既有承诺；770 §5.1 普通形态、canonical
   auto-acc「合法形态」事实、消费者 packages/aavm/lib/engine.at:314/333 引号内
   换行实证）。冻结语义（对齐旧 lexer `string()` 行为）：引号内行界**原样进入值**
   ——LF 源保留 LF、CRLF 源保留 CRLF，不做归一化、不剥离续行前导空白。原
   `text.literal-newline` 拒绝码**作废**。
2. **常驻字面量是合法的借用返回来源**（恢复 D-03 原文「借自参数或字面量」）：
   `return <view>` 合法来源 = 存活借入参数**或静态字面量**（不朽存储、无清理
   边界）。§3.8 R1-04 的"当且仅当借入参数"表述作废；模型 StaticLiteral 来源
   已补（v04_r2_static_literal_return_pe）。
3. **转义白名单取材表**（旧 lexer `string()` 实测：`\n \t \r \0 \ \"` 处理，
   未知转义**原样两字符入值**；char 字面量 `\'`；另有 raw `` ` `` 串与 `"""`
   三引号形态）。第一批准则（冻结）：白名单 = `\n \t \r \0 \ \"`（str）与
   `\'`（char）+ **新增** `\u{1-6hex}`（显式标量；代理区/越界 → 前门
   `text.invalid-scalar`）；**未知转义 → `text.invalid-escape`**。与旧行为的
   差异（旧原样通过 → 新前门拒绝）作为适配项记录，不宣称旧行为继承。raw 串与
   三引号形态归 770 词法层裁量，774 不预拒绝。
4. **mut 参数区分**：`mut str`（借入可变：排他借用调用方 buffer，区域覆盖整个
   函数体，首批无中途关闭/降级形态——返回其视图拒绝 borrow-escape，无 reborrow
   降级）与 `mut Str`（对拥有值开排他借用）都是显式合法模式；模型均已建
   （ParamMutBorrowed；MutBorrowed(base)），v04_r2 矩阵与真实 PE 兄弟覆盖。

**R2-01 来源模型补全**：View/EndView 对**借入参数与静态字面量**同样是可追踪
区域（Viewed(k, base)，base ∈ {Local, ParamBorrow, ParamOwned, StaticLit}），
合法输入不再 panic（r2 复审控制已覆盖）。来源×模式×返回/存储/调用的合法/非法
矩阵见 v04_r2_source_mode_matrix（8 组合法脚本 + 15 组非法脚本，逐格冻结码）。

**R2-03 账本失败出口与计数边界（修订 D-06）**：账本元数据分配（首表+扩表）是
**内部簿记，不计入 attempts/allocs/frees**（三计数仅用户资源请求）；新增独立
注入点 `p774_fail_ledger_nth(n)`（第 n 次账本分配尝试失败），失败时用户请求按
env-oom 回滚（回滚 HeapFree 结果被捕获，拒绝释放计入 invalid_frees +
FREE-INVALID trace；扩表时旧表释放结果同样捕获）。oracle：v05-14（首表失败→
回滚→清旋钮后同请求成功）、v05-15（16 owner 存活时扩表失败→第 17 项回滚→
既有 owner 完好可释放→清旋钮后扩表成功→全平衡）。不可注入侧（真实元数据 OOM、
OS 拒绝回滚/旧表释放）为接线审计声明，不以测试冒称。

**R2-04 收据校准（验证义务）**：最终门禁收据必须在**提交后的干净树**上生成，
verification 引用实际 raw 目录名（目录时间戳=门禁开始时刻）与提交 HEAD；
历史运行按源 hash 标注可复用性，不冒称提交后基线凭证。

## 3.10 r3 复审修订（P774-R3-01/02 + D1）

1. **预算表示（修订 D-06，r1 §3.8 预算条目的宽度补全）**：`p774_set_budget(i64)`
   的 v≥0 按 **u64 原值保存**，与存活字节的比较用 `(live_bytes as u64) +
   (len as u64) > budget`——**无截断、无回绕**。4 GiB（0x1_0000_0000）与
   4 GiB+1 预算下 2 字节请求成功（oracle：v08_budget_i64_no_truncation 三例，
   即 r3 复审官的两失败控制+u32 上界正控）。0=零字节预算、负值清除、释放
   归还额度语义不变。SUMMARY 的 live_bytes/peak 打印仍为探针规模数值。
2. **V05-15 oracle 拆分（修订 D-06 账本失败证据义务）**：原"扩表失败→释放一
   个 owner→重试"实为**空位复用**（重试时 len_n<cap，未再触扩表）。拆为两条
   冻结 oracle：**V05-15a 真跨容量恢复**（16 owner 全存活时清旋钮重试第 17 项
   →扩表真实发生且成功；读回全部 17 个 buffer 内容；逆序全释放；SUMMARY
   `18 17 17 0 0 17`）与 **V05-15b 空位复用控制**（释放一个 owner 后重试进
   现有表，SUMMARY `18 17 17 0 0 16`）。两条路径各自命名、各自断言。
3. **矩阵计数勘误（D1 随附）**：v04_r2_source_mode_matrix 的非法脚本数为
   **16**（此前文档误写 15）；合法脚本 8 组不变。
