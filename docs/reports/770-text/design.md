# PLAN-770 T-01 设计文档：文本与字符首批原生编译

- 计划：PLAN-770 r2（work 阶段；T-01 产出）
- 工作树：`D:/autostack/.wt/lang-770/auto-lang` @ `plan-770-dev`，基面 = master `e849e06f9`
- 上游合同：[774 decisions](../../reports/774-data-resource/decisions.md)（§3 + §3.8..3.12 修订链）、
  [774 interface-contract](../../reports/774-data-resource/interface-contract.md)、
  [774 probe-cases](../../reports/774-data-resource/probe-cases.md)（§0 冻结常量）
- 边界重申：本计划交付 **text/char 首批 profile**（source/HIR→真实 PE）；不编译完整
  token/lexer、不支持 List/enum/record 文本字段、不宣称 ACC Gen1、不冻结生产 ABI。

## 1. 基线与输入复核（T-01 收据）

- 前置确认：772、774 均已复审合入并归档（`docs/plans/archive/`）；774 merge receipt
  finalized（e849e06f9）。用户指令「实施它」= r2 批准；本计划由 drafting 翻 executing。
- 输入指纹：对 [774 引用核查输入表](../../plans/attachments/774/reference-update-inputs-20261013.tsv)
  全部 47 项 `origin=main` 输入在实施基线 `e849e06f9` 复核 SHA256——**39/47 零漂移**；
  8 项漂移全部归因（无未解释语义漂移）：
  - `docs/specs/{auto-ac/project.md, auto-hir/stage-contract.md, auto-acc/project.md,
    auto-ac/plans.md}`：774 canonical 沉淀提交 930d05fae（本计划消费的合同本体，非漂移）。
  - `docs/plans/771-*.md`：774 r2 下游同步 6845a7601（身份/拼写改齐，771 仍 drafting 0/9）。
  - `docs/plans/770-*.md`：本日 drafting→executing 簿记。
  - `docs/design/00-intro.md`：设计登记簿簿记（774 T-01 已记录同类漂移）。
  - `docs/plans/774-*.md`：归档移动（现 `archive/774-ac-data-resource-contract.md`）。
- **代码路径零漂移**：`experimental/ac-core/src/**`、两个 schema atom、根/产品 crates 与
  输入表一致——774 合入未触碰 ac-core 主 crate 源码（git diff d777ccbb..e849e06f9 仅新增
  `probes/data-resources/**` 与 `scripts/verify-ac-data-resources-774.ps1`）。
- 基线构建探针：`cargo check --manifest-path experimental/ac-core/Cargo.toml --locked
  --all-targets` 与 probes/data-resources 同命令在 worktree 全绿（收据见 §8）。

## 2. D-01..07 与 770 r2 一致性复核

| 决定 | 774 冻结内容（含修订链） | 770 r2 引用 | 一致性 |
|---|---|---|---|
| D-01 | int=i32；char=u32 标量位型；内部长度 u32/指针机器宽；-1/-2/-3 转换码 | §5.1 索引行/资源行引用 774 | ✅ 一致；770 不把内部长度当 i32 回传 |
| D-02 | UTF-8 存储、标量索引、char 名义、`to_codepoint/from_codepoint` 拼写、范围码 -7..-10、NUL/空串合法、跨行字面量合法（r2 恢复，`text.literal-newline` 作废）、转义白名单 `\n \t \r \0 \ \"`+`\'`+`\u{1-6hex}`、未知转义 `text.invalid-escape`、char 单标量（`text.char-multi/empty`） | §5.1 表全文引用 + 774 r2 同步注 | ✅ 一致（770 §5.1 已按 §7.2/§7.3 改齐） |
| D-03 | str/Str 逐语境表；参数默认 view；显式 `mut Str`/`move Str`；返回借用=参数或静态字面量（r2）；clone/`.to`/`.move`；容器/record 禁令 | §5.1 资源行「采用774逐语境str/Str表」 | ✅ 一致；770 无 record/List，field/container 码在本 profile 无可达入口（组合 profile 才触达） |
| D-04 | 前门 verify 层执行；8 个 verify.* 拒绝码；native 调用计数 0；LIFO 唯一清理；临时存活至语句尾；来源五类 | §5.2 可信门 | ✅ 一致 |
| D-05 | ABI-A/B/C（770 消费面）；指针 RAX+长度 out 参；panic 不穿 ABI | §5.2/§2 | ✅ 一致 |
| D-06 | 进程堆分配器；live-bytes 预算（u64 全精度，r3/r4/r5）；fail_nth/fail_ledger_nth 注入；完整外部账本；计数口径；trap=OS 回收不伪称 unwind | §5.2 | ✅ 一致；770 runtime 同构继承 |
| D-07 | 身份 `auto.hir.text.draft / 1 / text-i32-draft`；热点清单；合入顺序 | §5.1 | ✅ 一致 |

无材料冲突；无需修订 770 合同。774 §3.8 R1-04.4 的 `text.literal-newline` 已被 §3.9 R2-02
撤销——770 词法层按 **跨行合法（行界原样入值）** 实现，不实现该死码。

## 3. 冻结：新增诊断码清单（T-01 冻结，774 已冻结的码不重复注册）

### 3.1 词法/字面量层（stage=text，source 前端）

| 码 | 触发 | 备注 |
|---|---|---|
| `text.invalid-escape` | 未知转义序列（白名单外） | 774 §3.9 R2-02.3 |
| `text.invalid-scalar` | `\u{...}` 代理区/越界；char 字面量非法标量 | 774 §3.8 R1-04.4/§3.9 |
| `text.char-multi` | `'ab'` 多标量 char 字面量 | 同上 |
| `text.char-empty` | `''` 空 char 字面量 | 同上 |
| `source.lex`（沿用） | 未闭合引号、坏字节 | 既有码 |
| `source.unsupported`（沿用） | profile 外 token（raw 串/三引号首批由词法层裁量：**按普通 str 字面量路径拒绝为 unsupported**，不半支持） | 本计划冻结裁量 |

### 3.2 绑定/身份层（stage=bind，descriptor 侧）

| 码 | 触发 |
|---|---|
| `bind.descriptor-unknown`（沿用） | 未注册三元组（新表将 text 列入 accepted） |
| `descriptor.*`（沿用） | 结构违规（未知 tag/字段/枚举域） |

### 3.3 verify 层（stage=verify；774 冻结的 8 码中 770 有可达入口的 6 个 + 2 个身份/类型码）

| 码 | 774 来源 | 770 可达入口 |
|---|---|---|
| `verify.borrow-escape` | D-04 | 返回借自局部/move-in 参数的视图 |
| `verify.ownership-required` | D-03/R1-04.1 | move 参数调用缺 `.move` |
| `verify.use-after-move` | D-04 | `.move`/move 调用后再用 |
| `verify.double-move` | D-04 | 二次 `.move` |
| `verify.use-after-free` | D-04 | 已清理后再 view（程序化 Bundle 面） |
| `verify.mut-conflict` | D-04 | 活跃 view 期间 mut/move owner |
| `verify.field-borrow` / `verify.container-borrow` | D-03 | **本 profile 无入口**（无 record/List；码名保留给组合 profile，770 不注册死入口） |
| `verify.owned-scope-escape` | 770 新增 | owned 局部（或其视图来源局部）在其 let-block 子树外被读（含 if 合流后再读） |
| `verify.resource-assign` | 770 新增 | `=` 赋值目标为 owned（Str）局部；或显式 Drop 非 owned 局部 |
| `verify.domain`（沿用） | — | 域不匹配的 verify 入口 |
| `verify.type` / `verify.profile-op`（沿用语义） | — | 文本操作数类型错、旧 profile 混入文本节点 |

（T-02 实施前增补：owned 局部按块词法作用域存活——读取仅限其 let-block 子树；
str 局部的视图来源（provenance）继承被视 owned 局部的同一作用域约束；两条由
`verify.owned-scope-escape` 执行。`=` 对资源类型赋值禁止沿 Design 04 F8，
显式 Drop 语句仅程序化 Bundle 面，源码前端不生成。）

### 3.4 运行期状态值（PE 可观察；774 §0 冻结，770 原样消费）

`-1..-11` 状态码注册表原样沿用（res.len-overflow/-negative/-too-large/budget/env-oom/
shim-error、text.invalid-utf8/invalid-scalar/range-reversed/range-oob、res.index-oob）。
**PE 出口策略（本测试 profile 冻结）**：生成代码检查 runtime 调用状态，负值 → runtime
先写一行 `RT-FAIL code=<n>`（stderr trace，770 runtime 新增）→ 显式 `ExitProcess(70)`
（沿 741 trap 出口语义：范围/分配失败都是**值返回后受控出口**，不伪称 unwind；70 是测试
profile trap 码，不是语言异常 ABI）。范围 trap 与分配失败靠 `RT-FAIL code=-9/-10` vs
`code=-5` 区分取证；普通 `return 70` 不计为 trap（trap 证据=stderr RT-FAIL 行 + 退出码）。

### 3.5 能力/门（stage=capability/backend）

| 码 | 触发 |
|---|---|
| `capability.missing`（沿用） | 文档 requires `ac.runtime.text.v1` 而构建未显式提供 |
| `runtime.contract-missing` | 文本模块构建但 `--support-lib` 无伴随 runtime 合同描述 |
| `runtime.contract-hash` | 合同 hash ≠ 实际库文件 SHA256 |
| `runtime.contract-target` | 合同 target ≠ 本次构建 target（windows x64） |
| `runtime.contract-api` | 合同 API 版本 ≠ lowering 消费的 `ac.runtime.text.v1` |

## 4. 冻结：HIR 文本面设计（T-02 实现入口）

### 4.1 身份与类型

- 新 HIR 域：`HirDomain::Text` = `auto.hir.text.draft / 1 / text-i32-draft`；
  新 schema `schema/text.atom`（scalar-i32.atom 超集：枚举/节点只增不删，旧 branch 全保留）。
- 新源 profile：`SourceProfile::Text` = `auto.source.text.draft`（scalar 面 + 文本面）。
- `BuiltinType` 增 `StrSlice`（拼写 `str`）/ `StrOwned`（拼写 `Str`）/ `Char`（拼写 `char`）。
  字面量无命名类型（Design 02）：字面量表达式的 TypeUse 由语境决定（借语境=str，拥有
  语境=Str），binder 不造匿名类型。
- `ValueType`（verify）增 `Str`/`StrOwned`/`Char`；i32/bool 规则不变。

### 4.2 新表达式节点（ExprKind 增补；全部带 span/type，程序化 Bundle 可构造）

| 节点 | 字段 | 语义/资源 |
|---|---|---|
| `StrLit { value: String }` | 值为解码后 UTF-8（转义已解析） | 静态字面量来源（StaticLiteral）；native 侧入 rodata，view 语义；提升=AP-4 |
| `CharLit { scalar: u32 }` | 单 Unicode 标量 | ABI-A 值 |
| `TextLen { operand }` | → i32 标量计数 | ABI-B 调 `rt_text_scalars` |
| `TextLenBytes { operand }` | → i32 字节长度 | ABI-B 调 `rt_text_bytes` |
| `TextCharAt { operand, index }` | → char | ABI-B 调 `rt_char_at`（-7/-10） |
| `TextSlice { operand, start, end }` | → str（借用切片） | ABI-B 调 `rt_text_slice`，无分配 |
| `TextConcat { lhs, rhs }` | → Str（拥有） | ABI-C 调 `rt_own_concat`（AP-3；-1 先于分配） |
| `TextClone { operand }` | → Str | ABI-C 调 `rt_own_clone`（AP-2） |
| `TextToOwned { operand }` | str/lit → Str | AP-4/字面量提升路径（`.to(Str)`） |
| `TextToBorrowed { operand }` | Str → str 视图 | `.to(str)`：借用化不转移（view 检查） |
| `CharToCodepoint { operand }` | char → i32 | ABI-A（恒成功） |
| `CharFromCodepoint { operand }` | i32 → char | ABI-A（-8 invalid-scalar） |

- `+` 在 (text×text) 上 = `TextConcat`（源码拼写；取材 engine.at `out = out + "\n"`）。
  i32 上 `+` 仍 AddI32；类型不通融。
- `==`/`!=` 在 (text×text) 与 (char×char) 上复用 `BinOp::Eq/Ne`（类型导向 lowering：
  文本比较走 `rt_text_eq`/`rt_text_ne` ABI-B 调用，字节精确、NUL 不截断、无 normalization；
  char 比较是标量值比较）。旧 profile 的 eq/ne 语义不变。
- 文本排序（`<` 等）不在本计划：typecheck 拒绝（`verify.type`/source 面复用既有码）。

### 4.3 新局部/参数/语句面

- `LocalDef` 增参数模式 `param_mode: ParamMode { View, MutBorrow, MoveOwn }`（默认 View；
  显式 `mut` → MutBorrow；显式 `move Str` → MoveOwn——仅 Str 参数可 move，str 参数只可
  View/MutBorrow）。`mut str`（借入可变）/`mut Str`（对拥有值排他借用）都合法（774 §3.9
  R2-02.4）；本 profile **无可变文本操作面**（无 mutation 方法），mut 文本参数首批仅允许
  **声明+传递**，任何写面拒绝——排他性由 verify 前门保证（活跃期禁止 caller 侧 view/move）。
- `Stmt` 增 `Drop { local }`：**程序化/作者 HIR 显式清理节点**。source 前端不生成 Drop
  （源语言无显式 drop 拼写）；verify 对 source 通道生成的 HIR **自行计算**清理并校验
  （见 4.4）。程序化 Bundle 带 Drop 时 verify 按同一 LIFO/唯一责任规则校验（多清/漏清/
  清借用 = `verify.use-after-free`/资源义务违规）。
- `.move` 拼写：`Call` 实参加 `move` 标记（`CallBinding` 增 `moved: bool`）或
  `Let`/`Assign` 右值 `TextMove`？——**冻结：不引入 TextMove 表达式**；move 是**绑定行为**
  不是值操作：调用点 `f(x.move)` 在 AST/resolve 层标记，HIR `CallBinding { moved: true }`；
  move 参数函数内该 local 即拥有（exit 清理责任=callee）。

### 4.4 资源义务与清理调度（verify 产出，native 消费）

- verify（text 域）在既有检查之上追加**资源数据流**：
  - 每 local 的来源五类（LocalOwned/ParamBorrowed/ParamMutBorrowed/ParamOwned/StaticLit）
    与 view 活跃区间（词法区域）；
  - move/dead 点：`x.move` 调用后 x 死亡；move-in 参数 = callee 拥有；
  - 临时拥有值（未绑定 local 的 concat/to/clone 结果）存活至所在完整语句尾；
  - 出口清理：owned 活跃集按 **LIFO**（声明序倒序）在 return/break/continue/自然收尾
    生成清理序；分支合流取各路径一致清理责任。
- 产出 `ResourcePlan`（CheckedModule 内部附属结构，随 verify 构造）：
  `(body, block) → 有序清理列表 + 每 expr 的拥有性/来源标记`。native **只消费**该计划
  （后端不重做所有权分析）；计划错误=verify 拒绝，不存在后端补验证路径。
- 全部规则在 **verify 层前门**：拒绝码走 §3.3；拒绝模块**零 native 调用**（V-06 断言）。

### 4.5 与旧 profile 隔离

- text.atom 的 branch 契约对旧节点沿用 scalar 规则；新节点新 branch（require/forbid 全列）。
- `verify_text` 显式入口；`verify`（core）/`verify_scalar` 对 `HirDomain::Text` 一律
  `verify.domain` 拒绝。`bind_document` 注册表加入 text 三元组（accepted 列表更新）。
- core/scalar profile 的 source/verify/native **零行为变化**（V-10 三旧 gates 字节保持）。

## 5. 冻结：runtime/text 静态库（T-03 实现入口）

- 位置 `experimental/ac-core/runtime/text/`（独立 workspace，含 shim 式 no_std
  staticlib + 测试 crate；沿 774 probe 结构，同构重命名，**签名形态不变**）。
- 符号（`ac_rt_` 前缀；774 p774_* 同构映射）：
  `ac_rt_own_make/own_free/own_clone/own_concat/own_from_lit`、
  `ac_rt_text_scalars/text_bytes/char_at/text_slice/text_eq/text_ne`、
  `ac_rt_char_from_codepoint/char_to_codepoint`、`ac_rt_len_from_i32/len_to_i32`、
  `ac_rt_set_budget/fail_nth_alloc/fail_ledger_nth/shim_error/summary/check_invariant`、
  `ac_rt_trap_after_alloc/hang_after_alloc`（trap/超时取证）。
  **不包含**：record 族（771 面）、R2 对照族（774 淘汰证据专用）、p774_mix/many
  （ABI 探针专用）。
- `ac_rt_text_eq/ne(ptr,len,ptr,len) -> i32`：新增（774 探针无 eq；语义=完整 UTF-8
  字节值相等，返回 0/1）。
- 行为继承：4 字节头拥有 buffer、完整外部账本、live-bytes u64 全精度、预算 0=零字节、
  负值清上限、零长不触分配器、fail 注入点、LIFO 计数口径、SUMMARY/FREESTAT/trace 行
  格式与 774 逐字一致（探针解析器可复用）；新增 `RT-FAIL code=%d` 失败 trace 行。
- **合同描述文件**（T-03 产出、T-05 消费）：`<lib>.contract.json` 与静态库伴随，字段：
  `api: "ac.runtime.text.v1"`、`api_revision: 1`、`target: "x86_64-pc-windows-msvc"`、
  `lib_sha256`、`symbol_count`。构建时 finish_build 校验 hash/target/api（§3.5 码），
  收据记录真实库路径+合同指纹；**不只信 capability 字符串**。
- 库构建：`cargo build --manifest-path experimental/ac-core/runtime/text/Cargo.toml
  --locked --release` 产出 staticlib；锁文件入库跟踪。

## 6. 冻结：source 文本面（T-04 实现入口）

- 七模块扩面（lexer/ast/parser/resolve/typecheck/adapter/mod）：
  - lexer：`"`/`'` 字面量升级为正式 token（`Tok::StrLit(decoded)`/`Tok::CharLit(scalar)`，
    解码+白名单转义校验在词法层完成，码走 §3.1）；跨行合法（行界原样入值，CRLF 源保
    CRLF）；`\u{1-6hex}`；未闭合=既有 source.lex。raw `` ` `` 串与 `"""` 三引号：
    词法识别后按 `source.unsupported` 拒绝（不半支持）。新类型词 `str`/`Str`/`char`
    入 Kw；`to`/`move`/`mut` 出 RESERVED 变为上下文敏感处理（`.to`/`.move` 点后方法
    拼写；`mut` 参数修饰）。**core/scalar profile 下这些 token 仍按 Raw/Reserved 拒绝
    （零行为变化）**——文本 token 只在 text profile 激活。
  - ast：TypeWord 增 Str/StrOwned/Char；ExprKind 增 StrLit/CharLit/MethodCall{recv,
    name,args}；Param 增 mode（默认/mut/move）；实参 `.move` 标记。
  - parser：方法后缀链（`.len()`/`.char_at(i)`/`.slice(s,e)`/`.to(str|Str)`/`.clone()`/
    `.to_codepoint()`/`.move`、`char.from_codepoint(n)` 类型级调用）迭代解析（深链安全）。
  - resolve/typecheck：接收者类型绑定内建方法（局部/用户函数**遮蔽优先**——同名用户
    函数不夺内建名，反之用户声明不夺内建方法名：**冻结：内建方法按 (receiver 类型,
    方法名) 绑定，用户函数按裸名绑定，两命名空间不冲突**）；`+` 双文本→concat；
    eq/ne 双文本/双 char；无隐式 char/int；提升 lit→str（语境借用）与 str→Str **仅经
    显式 `.to(Str)`**（D-03：容器/record 语境不存在，无自动提升调用点）；borrow/move
    数据流（typecheck 产出 → adapter 生成 HIR 时写 param_mode/moved）。
  - adapter：文本表达式→§4.2 节点；span 映射保留。
- fixtures：`fixtures/text-source/{valid,invalid}/*.at`、`fixtures/text-hir/{valid,invalid}
  *.atom`（V-01..V-09 材化输入）。
- 深链保证：文本方法链/concat 链沿用既有迭代 spine+迭代 Drop 机制（deep_regression
  已证模式）；V-09 文本链 4096 + 资源循环 4096 进 T-04/T-06 测试。

## 7. 冻结：native/CLI 面（T-05 实现入口）

- native：text 域 lowering——locals 映射 Cranelift 变量（str=(ptr,len) 双变量、Str=
  (ptr,len) 拥有、char=I32）；每文本节点→§5 runtime 调用（ABI-A/B/C 形态按 774 D-05）；
  ResourcePlan 清理序→出口 free 调用；调用点 moved 参数→转移（callee 责任）；返回
  Str=指针经 RAX+长度 out 参（入口 wrapper 通道）；返回 str=只允许 param/字面量来源
  （verify 已拒其余）。运行期负状态→`RT-FAIL` trace + ExitProcess(70)（§3.4）。
- main/finish_build：`--source-profile auto.source.text.draft` 新枚举；`build`（HIR 模式）
  按 text 三元组分派 verify_text；文本模块 requires `ac.runtime.text.v1` 时强制：
  capability 显式提供 + `--support-lib` 库伴随合同描述且 hash/target/api 匹配（§3.5），
  缺任一在**发射发布前**拒绝；收据记录 runtime 库+合同指纹。旧 finish_build 事务
  （staged obj/link/publish 原子性）不分叉——文本检查在同一共享尾部之前插入。
- `check`/`check-source` 文本面走 verify_text，不产 native。

## 8. T-01 全路径可行性探针（收据）

- **运行时侧（已证，774 探针在基线复跑）**：本 worktree 上
  `cargo check --manifest-path experimental/ac-core/probes/data-resources/Cargo.toml
  --locked --all-targets` 通过（真实调用链：Cranelift caller→shim callee→COFF/PE→
  受控执行器）；并全量复跑
  `cargo test --manifest-path … --locked`：**65/65 绿**（lib 1 + smoke 1 + v02 9 +
  v03 10 + v04 16 + v05 14 + v06 2 + v07 5 + v08 7，exit 0；2026-10-14 于
  plan-770-dev@e849e06f9 worktree）。774 V-03（寿命/LIFO/临时/callee 清理/return
  转移）、V-05（AP-1..4 故障注入）、V-07（Unicode 全表）、V-08（4096 规模+预算）
  即 770 runtime 的同构 oracle——770 不重证可行性，只做同构重命名+`text_eq`+
  `RT-FAIL` 增量（T-03 自带全量 runtime 测试 V-04/05 对应面）。
- **编译器侧（设计保证+定向探针）**：文本深链复用 scalar 已证的迭代 spine/迭代 Drop/
  迭代 verify 机制（deep_regression.rs 16384 链/50000 括号先例）；T-04 物化 V-09
  （文本链 4096、调用混合 4096、资源循环 4096）为可执行断言。**支持预算冻结**：
  无语法/嵌套深度上限（拒绝码集合无深度类码）；唯一规模约束=运行期 live-bytes 预算
  （D-06）。合法语法规模截断当 crash 修复=禁止（沿用 R6 裁定）。
- **return 路径**：拥有 Str 跨帧返回（RAX 指针通道）与借用 str 跨帧返回（param/字面量
  来源）均为 774 v03-9/v04-r2c 已证形态；770 在真实 ac 管线复现（T-05 V-04）。

## 9. 与 771 的热点合入顺序登记（D-07）

- 共同热点（两分支都改）：`src/source/*`、`src/hir.rs`、`src/descriptor.rs`、
  `src/verify.rs`、`src/native.rs`、`src/main.rs`（+ schema/*.atom、lib.rs 注册表）。
- 771 当前 drafting 0/9、未建树——**770 预计先合入**。裁定沿 774 interface-contract
  §6：先完成者先合入；后完成者（771）在新整合基线重放自身全门禁 + text profile 回归；
  非文本合并冲突（身份/签名/能力门不兼容）回 new 修订，不宽松绕开。
- 770 合入时点对 771 的增量面：HirDomain/SourceProfile 枚举增 Text 变体、ExprKind/
  Stmt/LocalDef 增文本节点、bind_document 注册表加 text 三元组、native 分派加 text
  域。771 实施时按 interface-contract §5 消费 A/D/E，不与文本面冲突（record 字段首批
  i32/bool）。

## 10. Case 物化清单（V-01..V-10 → 测试落点）

| Case | 落点（新增测试文件/目录） |
|---|---|
| V-01 身份/隔离 | tests/text_hir.rs（HIR 腿）+ tests/source_frontend.rs 增 text 面（source 腿） |
| V-02 Unicode | tests/text_source.rs（check/构建腿）+ tests/text_native.rs（真实 PE） |
| V-03 char | tests/text_source.rs + tests/text_native.rs |
| V-04 正常寿命 | runtime/text/tests（runtime 单元）+ tests/text_native.rs（PE 计数）+ tests/text_resources.rs |
| V-05 失败寿命 | runtime/text/tests（fault 注入）+ tests/text_resources.rs（PE fault 面） |
| V-06 Bundle 变异 | tests/text_hir.rs（程序化变异拒绝，native 计数 0） |
| V-07 顺序/短路 | tests/text_native.rs（mark_a/mark_b trace） |
| V-08 静态/来源 | tests/text_source.rs（未执行分支静态检查/CRLF/遮蔽） |
| V-09 生命周期规模 | tests/text_source.rs（深链 check）+ tests/text_native.rs（受控 PE） |
| V-10 集成 | scripts/verify-ac-text-770.ps1（新 gate + 741/745/772 旧 gates） |

## 11. 验证命令冻结（开发/门禁）

- 开发：`cargo check --manifest-path experimental/ac-core/Cargo.toml --locked --all-targets`
  + scoped `cargo test --manifest-path … --locked --test text_hir`（及 text_source/
  text_native/text_resources）。
- runtime：`cargo test --manifest-path experimental/ac-core/runtime/text/Cargo.toml
  --locked` + fmt/check。
- 最终：完整 ac-core test/fmt、runtime 同上、`scripts/verify-ac-text-770.ps1`、
  旧 gates（741/745/772，771 未合入则无其 gate）。
- 无 crates/root 定义改动 → 不跑根 cargo t/tv/docs_gen；strict acc_inventory 只核对
  未修改绑定。真实 exe 用既有 contained runner（build/run 各 60s）+ 完整输出。
