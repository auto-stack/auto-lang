# PLAN-782 规范增量提案（proposed-spec-delta）

> 状态：**提案**——本文是 T-07 冻结稿，未发布 canonical；merge 阶段
> （T-09）经独立复审通过后按 `/auto-plan:merge` 沉淀，与计划 §5 规范增量
> 表及 [design.md](design.md) §3/§4 对应。冻结内容以本文件为准；与
> 774/770/771 已冻结合同冲突处，以各自 canonical 优先（语义定义一处）。

## SD-01（add）docs/specs/auto-ac/source-text-record.md

新文件：text-record 组合源码 profile 的有限语言合同（结构对齐 770
source-text.md / 771 source-record.md）。要点：

- 身份：source `auto.source.text-record.draft` → HIR
  `auto.hir.text-record.draft / 1 / text-record-i32-draft`（探针 N07 实证
  与四旧身份零碰撞；注册后旧四域身份/接受集零变化）。
- 支持面（全部经真实 PE / 前门实证，verification.md §2）：
  - 类型：`i32/bool/char/Str` + **非递归 record**（字段可含 Str 与
    已声明 record；前向引用合法；图无环沿 771）。char 不隐式转 int。
  - 资源分类（**派生**，禁止信任输入自称）：任意传递字段含 Str ⇒
    资源 record（显式 move/clone）；仅标量/无资源嵌套 ⇒ copyable
    record（771 值复制语义原样保留）。
  - 构造：位置与命名双形（源码序求值再映射；逆声明序命名合法）；
    Str 字段实参须拥有表达式（字面量提升 AP-4、`.to(Str)`、clone/
    concat 结果、`x.move`、拥有调用返回），借用视图实参前门拒。
  - move/clone：完整根为 move 单位（局部→move 参数→拥有 return/
    let/assign 全路径）；`r.clone()` 内建深复制（用户声明 clone →
    `source.duplicate`）；裸资源赋值 `verify.resource-assign`、裸
    return `verify.ownership-required`；部分字段 move
    `verify.partial-move`（782 唯一新码；含未执行分支静态检查）。
  - 字段读取：Str 字段读取=借用视图（永不复制 owner；HIR 侧
    FieldGet 的 `Str`/`str` 两拼写等价，lowering 均装载视图）；
    len/char_at/slice/eq/ne/to 等沿 770 面以字段为接收者。
  - 字段替换：根=var/mut 参数/move-in 参数；RHS 先物化→旧缓冲恰
    一次 free→提交；活跃借用期间拒 `verify.mut-conflict`（按根
    保守）；let/view 根拒 `verify.immutable-write`；标量字段写沿 771。
  - 参数/方法：默认 view；mut/move 定义与调用 `.mut`/`.move` 对称；
    move 参数限 Str/资源 record（copyable → `verify.ownership-required`）；
    static/只读实例方法沿 771（receiver 先于实参求值一次）。
- 拒绝面（一等合同）：str 字段 `verify.field-borrow`、递归/空 record/
  默认字段沿 771；enum/List/Option/use/泛型/部分字段 move/借 str 字段/
  递归堆 record/`mut fn`/unit/隐式尾返回/record equality 不变；旧四
  profile 对上述组合面零接受（`text/source.unsupported` 前门）。
- 诊断码：新码仅 `verify.partial-move`；扩面码七枚（field-borrow/
  resource-assign/ownership-required/mut-conflict/immutable-write/
  use-after-move/double-move/borrow-escape 的新可达面，design §3.2）；
  沿用码不增不减；运行期 RT-FAIL/trap 族原样。
- 验收锚：V-01..V-12（计划 §6 Case 表；verification.md §2 逐例落点）。

## SD-02（modify）docs/specs/auto-ac/data-resource-contract.md

在 774 冻结的 D-03/04/05/07 上追加**组合消费规则**（保留全部探针身份与
既有承诺，不重写）：

- D-03 增行：record 字段语境在组合域的完整规则——Str 字段拥有
  （构造/clone/move 单位=完整根）；字段读取为借用视图；`str` 字段
  拒绝面扩至 RecordShape（源码门透传 verify）。
- D-04 增行：来源五类在组合域的**字段来源对偶**——字段借用 backing
  溯源到根（generation 沿 770）；活跃字段借用期间整个根的 mut/move/
  替换保守拒绝（不承诺同根不相交字段借用拆分）；join/循环重入/
  after-loop 三态/参数对排他（含 mut 包装两层传播——同源拒、异源过，
  不凭形式参数号独立）/语句尾寿命全套沿 770，owner 集合扩至资源
  record 根；move-in 资源 record 参数=callee 根（出口逐 Str 字段缓冲
  清理）；owned_returns 扩至 record 根（out-slot 转移）。
- D-05 适用域划清：**标量 copy（赋值=拷贝）仅保留在无资源类型**
  （copyable record/标量/嵌套无 Str record）；资源 record inline 聚合
  但 Str 字段含内部拥有 buffer 表示——显式 move=责任转移零位拷贝、
  显式 clone=逐字段深复制；资源 record 布局沿声明序公式（Str 字段
  12/8、资源 record 8 对齐、copyable 嵌套保持 4 对齐，checked u64/u32）。
- D-07 增行：派生能力——模块含任意资源构造/clone/清理义务 ⇒ 派生
  `requires: ["ac.runtime.text.v1"]`（纯 copyable 模块零需求；caller
  声明不可绕过——沿 770 双触发门）。

## SD-03（modify）docs/specs/auto-hir/project.md

追加：text-record 组合域的 HIR 形态——`HirDomain::TextRecord`（schema
串 `auto.hir.text-record.draft`）；名义 record 类型在组合域可含
Str/char 字段；field place 组合段（field-owner 逐段验证扩资源段）；
**新表达式** `expr.record_clone { record, base }` 与 **`expr.move` 包装**
（let/assign/return/实参位的整体根转移单一表示；`CallBinding.moved`
在新域禁用）；资源分类为 verify 从 RecordShape 推导的派生属性（输入
Bundle 自称不参与——V-08 变异面）；`check_profile_ops` 四域化（组合域
=text ∪ record 面，旧四域对新节点零接受，`verify.profile-op` 双向）。
旧四域接受/拒绝集合零改动。通用 pass（跨域 lower/优化）仍 planned。

## SD-04（modify）docs/specs/auto-hir/stage-contract.md

追加：组合域阶段责任**已实证范围**——binder 绑定组合 descriptor
（`schema/text-record.atom` = record ∪ text 全节点集 + 双新 case）；
verify 前门执行结构/类型/资源分类/借用与所有权走查（含 excl 定点与
计划驱动能力派生；拒绝时 native 调用计数 0）；native 阶段为**合并
lowering**（组合域经 text lowerer 扩展；旧 text/record 路径零变化）：
栈槽聚合局部、指针形参数/结果（ABI-D/E，move-in `*mut`）、逐出口
buffer_frees 与 RHS 后 replace_frees、**跨类型创建序统一的语句尾/出口
LIFO（R1 修复）**、短路聚合临时的路径存在标记——真实 PE 实证
（V-02/05/09/10/11/12，256 深/4096 循环/0 次零分配）。故障双通道（R1
交付）：CLI 测试面注入旋钮（`--rt-fail-nth-alloc/--rt-set-budget/
--rt-fail-ledger-nth`，wrapper 前奏调用既有控制符号，无语言入口）+
组合域 status 失败的**部分输出 trap 前逆序回滚**（旧目标不提交；纯
text 域 trap 合同原样）。通用 pass/ACC 桥仍 planned。

## SD-05（modify）docs/specs/auto-ac/project.md

追加：text-record profile 实验性支持面（source/HIR→组合 verify→合并
lowering→COFF/PE 闭环已实证；internal ABI A–E 复用无新类；runtime/text
23 符号零新增——深 clone/释放/构造/字段视图全部复用既有符号；事务性
发布/失败不覆盖旧制品沿 741）。Gate：`scripts/verify-ac-text-record-782.ps1`
（25 步；770 gate 的 per-plan scope 步在后继树按归因裁定消费，
verification.md §4b）。

## SD-06（modify）docs/specs/auto-acc/project.md

追加：Token 拥有文本适配的**前置已有有限证据**——`type Token { text
Str; kind/line/at/len int }` 形状微程序（static/instance 方法+clone+
move，exit 57 精确值）与直接 HIR 腿真实 PE（V-12）；组合域证明内部
数据结构可携带拥有文本。**未交付不变**：enum（TokenKind）、List/
Map、use/多模块、可变实例方法、真实 lexer/parser 主体、Gen1-3、
GOAL-017 不关闭；ACC 桥仍 planned。

## 附 A：前置反例组合闭合核对（T-07）

计划 §4 要求组合场景重建 770/771 复审反例断言，不能只继承测试总数。
逐类核对（入口=组合域测试，全绿）：

| 前置反例类 | 组合域重建入口 |
|---|---|
| 770 R6-01 循环重入读取上一轮已释放 owner | `loop_reentry_kills_entry_live_moved_root`（resources） |
| 770 R4-05 短路 RHS 拥有临时支配/语句尾 | `conditional_move_in_short_circuit_rhs_rejects` + `v09_short_circuit_skips_clone`（跳过臂零分配零 FREE） |
| 770 R4-02 终止臂污染 join | `nested_return_move_transfers_out` + `arm_local_roots_close_at_their_own_block_exit` |
| 770 R4-03 臂内局部跨臂责任误算 | `arm_local_roots_close_at_their_own_block_exit`（自臂出口闭） |
| 770 R4-04 调用点排他包装传播（两层） | `wrapper_same_source_mut_view_conflict_rejects` / `wrapper_independent_sources_stay_legal`（**T-07 补齐**，mut 链两层；同源拒/异源过） |
| 771 R1-01 目的地别名存活输入 | V-05 族（r=r.clone()/transform/move 重建，p02+gate TP02 逐步旧缓冲释放） |
| 771 R1-02 位置构造误拒 | p01 位置构造腿 + `v02_token_shape_values`（位置实参求值序） |
| 771 R1-03 CallBinding 越界 panic | `text_record_hir` binding 越界双模式（V-08） |
| 771 R1-04 失败分支 oracle 无区分力 | native 测试精确 exit 码 + gate HP01（exit 2 精确 trace）/TN 族（exit 1 钉码） |

闭合判定：T-07 自查零缺口（两层 wrapper 由 0318bcb93 补齐）；**R1 final
review 另发现六项合同内缺口（F-01..06：嵌套字段隐式复制/move-return 重建/
短路循环聚合临时/跨类型 LIFO/Move.inner 界检/V-07 实注入），已全部修复并
各带永久回归（verification.md §5b）——闭合判定以 R1 修复后基线为准
（全套 448/0）。**

## 附 B：源文件 manifest（T-07 核对）

实现面（相对基面 `4568e4ea7`，全部在允许 scope 内）：
`experimental/ac-core/src/{hir,descriptor,verify,native,main,lib,text_resources}.rs`、
`src/source/{mod,parser,resolve,typecheck,adapter}.rs`；
`schema/text-record.atom`；`tests/text_record_{hir,source,resources,native}.rs`；
`tests/record_source.rs`（F-A +1）；
`fixtures/text-record-{hir,source}/{valid,invalid}/`（2+8+1）；
`scripts/verify-ac-text-record-782.ps1`；`docs/reports/782-text-record/*`。
runtime/text 与 crates/ 零改动（gate hash 收据）。取证声明与
verification.md §2/§3 一致；支持/拒绝面以 SD-01 为准（与 design §6/§7
逐条对拍无材料冲突）。
