# PLAN-782 T-01 设计文档：AC 文本与 record 组合——拥有型字段、移动与清理

> 状态：**冻结**（T-02..T-06 实施契约；实现须符合本文，不得反读实现输出当 oracle）。
> 基线：master `4568e4ea7`（= plan-770-dev tip，770 实现全部祖先；Spec 沉淀 `f4b1d2e82`）。
> 前置收据：770 R11 final pass（§9.24，`afe45e2d6`，联合门禁 14/14 gate-20261015-014555）；
> 771 已归档 delivered。取材指纹漂移全部归因见 §1.3。

## 0. 范围

在 770 文本与 771 record 已合入的整合基线上，实现独立 text-record 组合 profile：
拥有 `Str` 字段的非递归 record、聚合显式 move/深 clone、字段借用与替换、参数/返回
及全路径正常清理；源码与直接 HIR 双腿经真实 verifier→Checked→COFF/PE 执行。
非目标沿计划 §1（enum/List/use/泛型/部分字段 move/借 str 字段/递归堆 record/
mut fn/unit/record equality/GC/生产 ABI/ACC 桥/Gen1-3 不变）。

## 1. 基线与输入复核（T-01 收据）

### 1.1 前置硬门（计划 §5.1）——全项满足

| 项 | 证据 |
|---|---|
| 771 修复+复审+合入+沉淀 | archived `docs/plans/archive/771-ac-record-method-native.md`（delivered）；收据 `docs/reports/771-record/verification.md` |
| 770 修复完成 | R7..R11 逐轮闭合（770 计划 §8 任务行） |
| 770 独立 final review pass | R11 **pass**（§9.24；reviewed_commit `afe45e2d6`；无新增阻断项） |
| 770 真实合入 | master HEAD `4568e4ea7` == plan-770-dev tip（merge-base 实测；实现提交 `9033cdb39`/`e60b6eacc`/`7468c553e`/`314cbbeb6` 等全为祖先） |
| 770 Spec 沉淀 | `f4b1d2e82`：source-text.md（新）+ auto-hir/auto-ac/auto-acc 四文件 SD-02..05 |
| 第二合入者联合门禁+fresh review | 联合门禁 14/14（text 361+1 / runtime 20/20 / record 24/24+23 cases，gate-20261015-014555）+ R11 fresh pass |

770 T-09 归档/清理簿记仍开放（worktree `.wt/lang-770` 在途）——非 §5.1 构成项，不阻塞。

### 1.2 本树基线验证（worktree `D:/autostack/.wt/lang-782/auto-lang`，branch `plan-782-dev`，基 `4568e4ea7`）

- `cargo check --manifest-path experimental/ac-core/Cargo.toml --locked --all-targets`：exit 0，零警告。
- `cargo build --manifest-path experimental/ac-core/Cargo.toml --locked`：exit 0（CLI 可用）。
- 正控（前置面重放）：`record_hir 21 + record_source 21 + text_hir 16 + text_source 10 = 68/68` 绿
  （`probe/baseline-boundary/positive-controls.txt`）；record 旗舰 fixture check-source exit 0。

### 1.3 planning-inputs.tsv 漂移归因（采集于 2026-10-14 合入前）

| 输入 | 漂移 | 归因 |
|---|---|---|
| 4 份 canonical（auto-ac/project、auto-hir/project、auto-hir/stage-contract、auto-acc/project） | hash 变 | 771+770 canonical 沉淀（预期内；本设计以**现行** Spec 为基） |
| `data-resource-contract.md` | **零漂移** | 774 冻结合同原样有效——本计划以其为接口来源 |
| 770/771 计划文件 | 推进/归档 | R11 pass + archive 簿记 |
| `experimental/ac-core/src/{hir,verify,native,source/adapter}.rs` | hash 变 | 770/771 实现与 R6-03 整合合入（预期内） |
| lang-771 worktree 三文件 | 缺失 | 771 归档后 worktree 清理（guard 后移除，合规） |
| lang-770 worktree 四文件 | hash 变 | 770 树推进至最终 HEAD（预期内） |
| 774/design/strategy/ownership/lexer/token 等 | 零漂移 | — |

无未归因漂移。**材料冲突：无**（§2 逐条核对）。

## 2. 合同一致性复核（D-01..07 / 770 / 771 vs 计划 §5.2）

| 计划条款 | 现行合同依据 | 一致性 |
|---|---|---|
| 字段类型 i32/bool/char/Str/非递归 record；str 字段拒 | D-03 record 字段行（仅 Str，`verify.field-borrow`）+ 771 类型规则 + D-01 char=u32 | 一致；char 字段为 782 新增面（771 排除面收窄，D-01 已给宽度） |
| 资源分类（传递含 Str 即资源） | D-04 来源五类 + 774 无"信任自称"原则 | 一致；分类为**派生**属性（§4.2） |
| 资源 record 显式 move/clone；无资源保持值复制 | D-03/D-04 move 族 + 771 值复制承诺（D-05"嵌套 inline 赋值=拷贝"限无资源类型） | 一致；SD-02 将划清 D-05 标量 copy 适用域 |
| clone 递归深复制、空 Str 零分配 | D-06 零长度行 + 探针 P5 | 一致 |
| 参数默认 view；mut/move 对称 | D-03 参数行 + §7.4 参数模式三分 | 一致；MoveOwn 从 Str 扩至资源 record（机器形 *mut，§7.2） |
| 字段替换 RHS 先完成 | 771 目的地不相交不变式（lasting rule） | 一致；资源字段版本为"物化→释放旧→提交"（探针 P4） |
| 清理逆序/每分配恰一责任方 | D-04 + 770 清理责任不变式全套（join/重入/after-loop 三态/语句尾寿命） | 一致；782 复用 TextResourcePlan 扩展而非第二套（§4.4） |
| 分配失败 status vs fatal trap 三分 | D-06 失败类别 + 770 §5.3 | 一致；探针 P6 冻结 status 路径回滚 oracle |
| ABI A/B/C/D/E 复用 | D-05 五类形态 | 一致；无 sret、无新 ABI 类（§7） |

**基线缺陷发现（F-A，T-01 探针 N01..N03）**：record 域字段声明类型位遇 `Str`/`str`/`char`
标识符时 `parser.rs::type_word` 走 `unreachable!()` panic（exit 101），违反 771 Spec
"str/Str/char 字段显式拒绝"的一等拒绝合同。根因：`member_type()` 缺少 `param_type()/
decl_ann()` 均有的非 text 域守卫（`is_type_word` surface 盲）。该面由 770 R6-03 整合引入，
无在途计划拥有。**裁定：T-03 域内修复**（5 行守卫，与 param_type 对齐），冻结 oracle：
`error[text/source.unsupported]: type \`Str\` is not part of this source profile`，exit 1。
在计划 §10 登记为 in-plan 修复（非延后债务）。

## 3. 冻结：诊断码清单（774/770/771 已冻码不重复注册）

### 3.1 新码（仅 1 个）

| 码 | 阶段 | 触发 | 语义 |
|---|---|---|---|
| `verify.partial-move` | verify（源码门同步 source.type-mismatch 引导） | `.move`/moved 旗标作用于字段路径、子表达式或非整体 local（源码与程序化 HIR 双面） | 字段级部分 move 明确拒绝；move 单位=完整根 |

### 3.2 既有码扩面（码不变，适用域扩大；SD-02 划界）

| 码 | 扩面 |
|---|---|
| `verify.field-borrow`（774 冻结） | 新可达：RecordShape 的 `str` 字段类型（源码门透传至 verify 拒） |
| `verify.resource-assign`（770 冻结） | 从"owned Str 裸赋值"扩至"资源 record 裸赋值" |
| `verify.ownership-required`（774 冻结） | 新面：资源 record 裸 `return p`；move 模式参数用于非资源类型（copyable record/标量）；缺 `.move`/`.mut` 调用点（沿 771 双门） |
| `verify.mut-conflict`（774 冻结） | 新面：资源字段活跃借用期间整个根的 mut/move/替换（按根保守）；move-in 参数与 view 并用 |
| `verify.immutable-write`（771 冻结） | 新面：let/view 根上的资源字段替换 |
| `verify.use-after-move` / `verify.double-move`（774） | 新面：资源 record move 后使用/二次 move（含聚合） |
| `verify.borrow-escape`（774） | 新面：借用 str 返回来自局部/拥有临时/move-in/mut 参数的**字段** |

### 3.3 沿用码（不扩不缩）

`verify.record-cycle / record-empty(record 域既有) / ctor-arity / field-owner / owner-mismatch /
bad-reference / binding-invalid / profile-op / domain / identity-conflict /
bind.descriptor-unknown(unregistered) / source.* 族 / capability.missing /
record.layout-overflow / res.* / text.* 运行期值族`。运行期 RT-FAIL/trap70/-1..-11 原样。

## 4. 冻结：HIR 组合面设计（T-02 实现入口）

### 4.1 身份与 descriptor

- source `auto.source.text-record.draft` → HIR `auto.hir.text-record.draft / 1 / text-record-i32-draft`；
  descriptor `schema/text-record.atom`（= record.atom ∪ text.atom 全节点集 + §4.3 增补；
  对旧四 descriptor 零改动）。
- 探针 N07 实证：新三元组当前 `bind/bind.descriptor-unknown` 且不与四旧身份碰撞
  （错误消息列名四旧域为准）；注册后旧四域身份/接受集零变化。
- `HirDomain` 增 `TextRecord`（schema 字符串 `auto.hir.text-record.draft`）；
  `SourceProfile` 增 `TextRecord`（`surface_is_scalar_or_wider` 纳入；`is_text()`/`is_record()`
  谓词语义按"组合面=两者之并"重整，旧域判定零变化）。

### 4.2 资源分类（派生，禁止信任输入自称）

- 对每个 record 类型做**传递字段闭包**（worklist 迭代，无递归；环由 771
  `verify.record-cycle` 前置排除）：任意可达字段含 `Str` ⇒ **资源 record**；否则
  **copyable record**（771 值复制语义原样保留）。
- copyable record 不可能包含资源 record（闭包定义）；char/i32/bool 字段不改变分类。
- 分类只由 verify 从 RecordShape 推导；native/adapter 消费推导结果，输入 Bundle 的
  任何"自称"不参与（V-08 变异面）。
- MoveOwn 参数模式合法当且仅当形参类型为 `Str` 或资源 record；copyable record/
  标量的 move 模式 → `verify.ownership-required`。

### 4.3 HIR 增补（schema text-record.atom；共享 Rust 结构同步）

- **新表达式** `expr.record_clone { record, base }`（`ExprKind::RecordClone`）：record 型，
  深复制语义（§6.2）。程序化 Bundle 可直接构造（V-08 面）。
- **wiring 旗标**：`stmt.let_` / `stmt.assign` / `stmt.return` 增可选 `moved: bool`
  （缺省 false；三节点在 text-record.atom 声明，旧四 descriptor 不含该字段）。
  语义：RHS 为整体 local 的 `.move`（源码前端在 resolve 解包 `RExprKind::Move` 后
  落位，与 770 调用臂 `CallBinding.moved` 同一模式）。`moved=true` 而 RHS 非整体
  local 读 → `verify.partial-move`。
- `CallBinding.moved` 语义不变（调用实参位）。`Stmt::Drop` 语义不变（清理出口）。
- RecordShape 字段 TypeUse 面：内建 `i32/bool/char/Str` + record 引用；`str` 字段 →
  `verify.field-borrow`（结构检查阶段）。空 record/默认字段沿 771 拒。
- `check_profile_ops` 四域化：text-record 域=text 面 ∪ record 面；旧四域对该域节点
  （record_clone、moved 旗标）零接受（`verify.profile-op` 双向）。

### 4.4 资源义务与清理调度（T-04 实现入口；复用 TextResourcePlan，不建第二套）

- `BodyResourcePlan.owned` 扩容：拥有根=owned Str local **+ 资源 record local（含
  move-in 参数、.move 接收位）**；`exit_frees` 的释放单位细化为**逐 Str 字段缓冲**
  （经嵌套 record 的迭代字段遍历，逆声明序；跨根逆创建序——探针 P3 顺序）。
- 字段借用：Str 字段 view 进入 `backing` 溯源（owner=根 local，generation 沿 770 构造
  代际）；活跃字段借用期间整个根的 mut/move/替换保守拒绝（`verify.mut-conflict`）。
  不承诺同根不相交字段的借用拆分（计划 §5.2）。
- join/循环重入/after-loop 三态/参数对排他/语句尾寿命/短路临时携带——**全套沿 770
  不变式**（source-text.md §清理责任不变式），owner 集合扩至资源 record 根。
- 字段替换调度（探针 P4）：RHS 完整物化 → `own_free(旧字段缓冲)` 恰一次 → 提交新
  (ptr,len)；活跃语句尾临时寿命照旧。聚合重叠：`r=r.clone()`/`r=f(r)`/`r=eat(r.move)`
  由 RHS-完成门 + 转移标记统一覆盖；未完成时读旧 owner → `verify.use-after-move`。
- move-in 参数=被调方根（callee 清理）；`return p.move`/`return <fresh>` 的缓冲责任
  随 out-slot 转移（owned_returns 扩展至 record 根）；裸 `return p`（资源 record 现存
  local）→ `verify.ownership-required`。**plain Str 的 return 语义不变**（770 直转移），
  显式 .move-at-return 规则仅作用于资源 record。
- 派生能力：模块含任意资源构造/clone/清理义务 ⇒ `requires: ["ac.runtime.text.v1"]`
  派生（纯 copyable record 模块零需求；caller 声明不可绕过——沿 770 双触发门）。

### 4.5 物理布局（T-05 消费；探针 P2 冻结数值）

- Str 字段内部表示：`{ptr: u64 @0, len: u32 @8}`——字段尺寸 12、对齐 8；零长=ptr null
  +len 0（无分配器调用）。
- 布局沿 D-05 公式：字段声明序、各自对齐；`align(R)=max(字段对齐)`；
  `size(R)=round_up(尾偏移+尾宽, align)`；checked u64，尺寸上限 u32
  （`record.layout-overflow`）。char 字段 u32（4/4）；i32/bool 照旧。
- 冻结代表值：`Token{text Str,kind,line,at,len int}` size=32、offset 0/12/16/20/24；
  `{k int,text Str}` size=24、text@8；纯标量/nested-copyable record 保持 771 的 4 对齐
  （零回归）；256 层链 closed form `24+8n`。公共 HIR 不泄漏 offset（X6）。

## 5. 冻结：runtime/text（零改动）

**不新增任何 runtime 符号**（23 符号不变；`symbol_count` 收据照旧）：深 clone=逐字段
`ac_rt_own_clone`；释放=逐字段 `ac_rt_own_free`；构造=既有 `own_from_lit/own_concat/
own_clone`；字段 view 操作=既有 `text_scalars/text_bytes/char_at/text_slice/text_eq/
text_ne`；move=纯位转移零 runtime 调用。预算/故障注入沿既有控制面
（`ac_rt_set_budget/fail_nth_alloc/fail_ledger_nth`），无语言入口（770 §9.2 裁定沿）。
合同 sidecar/`RUNTIME_CONTRACT_API`/target/hash 校验门原样。若实现需要新符号 ⇒ 先修订
本计划（§10 登记）。

## 6. 冻结：source 组合面（T-03 实现入口）

### 6.1 声明与构造

- `type Tok { text Str; kind int; c char; meta Meta }`：字段类型 i32/bool/char/Str/已声明
  record；前向引用合法；图无环（771 规则原样）；`str` 字段词法/解析透传、typecheck 放行、
  verify `verify.field-borrow` 拒（诊断经 span 映射指向源码字段位）。
- 位置/命名构造沿 771（源码序求值+按名/按位映射；`ba` trace oracle 沿）。
  **Str 字段实参**须为拥有表达式（字面量提升 AP-4、`.to(Str)`、clone/concat 结果、
  `x.move`、拥有调用返回）；借用视图实参 → 源码门 source.type-mismatch 引导
  `.to(Str)`（HIR 门 `verify.ownership-required`）。
- F-A 修复（§2）：旧 record 域 member_type 补非 text 域守卫——旧域行为从 panic 变为
  与 param_type/decl_ann 一致的 `text/source.unsupported` 受控拒（旧域其余行为零变化）。

### 6.2 方法与内建

- 内建按（接收者类型,方法名）绑定：record 接收者新增 `clone`（→ `expr.record_clone`）；
  Str 字段接收者复用 770 全套（len/len_bytes/char_at/slice/eq/ne/to/to_codepoint/
  from_codepoint/move 不适用于字段——move 单位是完整根）。**用户在 record 上声明
  `clone` 方法 → `source.duplicate`**（无用户 clone hook；计划 §5.2）。
- `static fn`/只读实例方法沿 771（receiver 求值一次先于实参；`.field` 隐式 self）；
  实例方法体内 Str 字段可作 view 接收者；写字段/替换单元见 6.3。
- `fn f(move t Tok)`+调用 `f(t.move)`：move 模式限资源类型（§4.2）；
  `mut` 参数沿 771/774（排他、区域全程、`.mut` 调用点对称）。

### 6.3 赋值、返回与字段替换

| 写法 | 规则 |
|---|---|
| `p = q`（资源 record 现存值） | 拒：`verify.resource-assign`（源码门 source.type-mismatch 引导） |
| `p = q.move` | 合法：整体根转移；q 死（use-after-move/double-move 沿） |
| `p = q.clone()` / `p = Tok{...}` / `p = f(...)` | 合法：深拷贝/新构造/拥有返回直接接收 |
| `return p`（资源 record 现存 local） | 拒：`verify.ownership-required`；`return p.move`/`return Tok{...}` 合法（plain Str return 不变） |
| `p.text = <owned RHS>` | 字段替换：根=var/mut 参数/move-in 参数；let/view 根拒 `verify.immutable-write`；RHS 先完成→旧缓冲恰一次 free→提交（P4）；活跃字段借用期间拒 `verify.mut-conflict`；标量字段写沿 771 |
| `let/var t Tok = q.move` | 合法（`stmt.let_.moved`）；非整体 local 的 move → `verify.partial-move` |
| `p.a.b = ...` 嵌套路径字段替换 | 沿字段 owner 逐段验证（771 `verify.field-owner` 扩资源段） |

借用 str 返回来源：存活 view 参数的**字段**或静态字面量；局部/拥有临时/move-in/mut
参数的字段 → `verify.borrow-escape`。未执行分支静态合法性检查沿 770（词法层）+771。

## 7. 冻结：native/CLI 面（T-05 实现入口）

- **单模块单 emitter**：text-record 域经合并 lowering（资源 record 为 inline 聚合 +
  Str 字段 (ptr,len) 相邻对；自由 str/Str 值沿 770 (I64,I32) 双变量表示）。旧域
  lowering 路径零变化（scalar+record 与 text 两路照旧）。
- ABI 复用（无新类）：ABI-A 标量/char；ABI-B 借用 buffer（字段 view 传 (field.ptr,
  field.len)）；ABI-C Str 拥有返回（RAX ptr + 尾随 `*mut i32` out-len；共享 4 字节
  显式槽沿 770——4096 深链帧恒小）；ABI-D record view `*const`；ABI-E record
  mut/out-return `*mut` + caller 预置 out-slot（转发不相交归纳沿 771）。
  **move-in 资源 record 参数机器形 = `*mut`**（与 mut 同形；区别在 verify 责任簿
  ——callee 出口清理字段缓冲，mut 参数不清理）。无聚合 sret。
- trace oracle：每缓冲一行（LIT/CLONE/CONCAT 构造侧、FREE 释放侧，带 id）；
  释放顺序=P3 冻结（根逆创建、字段逆声明）；`mark` 先于 FREE（语句尾合同沿）；
  跳过侧零分配零 FREE。资源计数断言 alloc==free、live=0（SUMMARY 通道）。
- 故障双通道（P6）：helper status 路径=部分输出逆序回收+旧目标保全（编译器发射的
  逐字段回滚）；fatal trap 路径=RT-FAIL 行+ExitProcess(70)，仅承诺前驱 trace 可观察。
  探针注入经 runtime 控制面（测试族拥有）。
- 深输入：enable_probestack/inline 沿 770；全部遍历迭代式（布局/分类/清理/字段图）。
- CLI：`check-source/build-source --source-profile auto.source.text-record.draft`
  （未知值用法错 2 沿）；build 收据记 source profile+HIR 三重身份+输入 hash；事务性
  发布/失败不覆盖旧制品沿 741 合同。`sm_<Record>_<name>` 方法符号规则沿 771。

## 8. T-01 全路径探针收据

| 探针 | 内容 | 结果 |
|---|---|---|
| whole-path 算法探针 `probe/whole_path_probe.rs`（rustc -O 直跑） | P1 分类闭包 / P2 布局+u32 上界+256 深 / P3 清理序 / P4 替换与重叠 / P5 深克隆计划 / P6 故障回滚 / P7 规模（4096 循环+显式栈深清理） | **PROBE-OK exit 0，零警告**（`probe/run-output.txt`；`files.sha256` 绑定） |
| 组合边界负探针 N01..N09（`probe/baseline-boundary/`） | 旧四域×组合面：record 域 Str/str/char 字段（**F-A：现 panic，修复后受控拒**）；text 域 record 声明 `text/source.unsupported`；record 域 `.move`/`.clone()` 受控拒（`source.unsupported`/`source.undefined`）；scalar/core 域文本类型词受控拒；未注册三元组 `bind.descriptor-unknown` 列名四旧域 | 全部捕获于 `negative-run.txt`；除 F-A 三例外全为 exit 1 受控拒 |
| 前置正控 | record 旗舰 check-source exit 0 + 四测试族 68/68 绿 | `positive-controls.txt` |

探针=设计可行性+oracle 冻结证据，**不等于 feature 已实现**（组合正例的 PE 证明在
T-05 `text_record_native` 落地；§9 V-02/05/09/10/11/12）。

## 9. Case 物化清单（V-01..V-12 → 测试落点；probe-cases.md 为逐例索引）

| Case | 落点 |
|---|---|
| V-01 身份 | `tests/text_record_hir.rs`（CLI/Atom binder/API verify/writer：组合三元组过、四旧域拒新面、改一维拒、writer roundtrip） |
| V-02 构造/类型 | text_record_hir + text_record_source + text_record_native（两 Str+异质标量+嵌套；位置/逆声明序命名；char 字段；同形异名拒；str 字段 verify.field-borrow） |
| V-03 移动/clone | text_record_resources + text_record_source（move 无新增分配、旧根拒用；clone 双份独立 alloc=free/live=0；裸赋值/部分 move 反例无 native 调用） |
| V-04 借用来源 | text_record_resources（view 参数字段返回 str 合法；escape 前门拒；两层 wrapper 同源/异源） |
| V-05 替换/重叠 | text_record_native + text_record_resources（r=r.clone()、r=f(r)、r=eat(r.move)、字段 RHS 读旧再 concat、nested ctor——精确文本值+旧缓冲恰一次 free+fresh 兄弟对照） |
| V-06 控制流 | text_record_resources + text_record_native（source 与直接 HIR 各一腿：动态臂局部、嵌套 return move、break/continue、字段 view 外逃拒、0 次循环零分配） |
| V-07 故障/部分输出 | text_record_resources（两 Str 构造/深 clone 各第 N 申请失败、账本失败、零长、预算临界；status 回滚 vs trap 双通道） |
| V-08 可信变异 | text_record_hir（真实 Bundle 改字段 TypeId/FieldId/owner、binding.arg=u32::MAX、自环/断开环、模式与清理来源变异；受控诊断无 panic/Checked） |
| V-09 顺序/临时 | text_record_resources + text_record_native（&&/|| 跳过含拥有字段临时、receiver/命名实参 marker 先于 FREE） |
| V-10 规模/静态 | text_record_native（256 层含资源嵌套、4096 生命周期循环、前置深链、malformed partial Drop、CRLF/span、未执行分支非法类型） |
| V-11 发布/能力 | text_record_native（三件旧成功制品、缺库/错 target/hash、删 requires、故意失败 fixture——拒绝前不覆盖旧 obj/exe/收据） |
| V-12 集成边界 | text_record_native + gate（Token 形状微程序真实 PE 值/trace；enum/List/use 拒；新 gate+770/771+741/745/767/772 全跑；scope diff） |

fixtures 新建：`fixtures/text-record-hir/{valid,invalid}/`、`fixtures/text-record-source/{valid,invalid}/`。

## 10. 验证命令冻结（开发/门禁；同计划 §6）

- 快检：`cargo check --manifest-path experimental/ac-core/Cargo.toml --locked --all-targets`
- 开发（scoped）：`cargo test --manifest-path experimental/ac-core/Cargo.toml --locked --test text_record_hir`（及 source/resources/native 三 target）
- 收口（T-06 门禁，唯一实施树）：完整 `cargo test`/`fmt`（ac-core 与 runtime/text 两
  workspace，locked，零警告）+ 新 gate `scripts/verify-ac-text-record-782.ps1` +
  `verify-ac-text-770.ps1`、`verify-ac-record-771.ps1`、`verify-ac-741.ps1 -SkipMainGates`、
  `verify-ac-source-745.ps1`、`verify-ac-scalar-767.ps1`（772 脚本名以 770 gate 内实跑为准）。
- 本计划不碰 `crates/`、根 Schema、docs 生成器 ⇒ 根档 t/tv/docs_gen 不适用（范围变化先 revision）。

## 11. 资源预算表（测试材料预算；单位/入口/临界两侧）

| 预算项 | 单位 | 入口 | 临界两侧 |
|---|---|---|---|
| 嵌套深度 | record 层 | text_record_native 深链 fixture | 0/1（边界）与 256（代表规模，非上限）；>256 不设语法上限，走布局 checked 数学 |
| 生命周期循环 | 迭代 | 4096 循环 fixture | 0 次迭代（零分配零 FREE）与 4096（配对+live=0） |
| 字段数 | 每 record | V-02 Token 形状 | 1..5 字段代表覆盖；无固定字段数上限 |
| 故障注入 | 第 N 次申请 | runtime 控制面 fail_nth=1..k | k=构造/克隆总申请数（成功侧）与 k-1/k（失败侧回滚） |
| 预算 | 字节 | set_budget | 恰好容纳 vs 差 1 字节（res.-budget 前/后） |
| 环境 | 峰值内存 | 单测进程 | 深链/循环 fixture 轻池（<50MB）；画廊级重载不入本族 |

## 12. 与前置计划的边界重申

- 770/771 各自 pass 不构成组合验收（774 interface-contract §6）；本计划 V-12 才交付
  组合证据。真实 Token/lexer 迁移、List/enum/use、完整 lexer 编译、ACC 桥不宣称。
- 旧四域接受/拒绝集合零变化是 V-01/V-12 硬断言；F-A 修复将 record 域 panic 变受控拒
  属于**恢复** 771 已声明合同，非扩面。
