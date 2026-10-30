# Auto 源码 text-record 组合 profile

> **Status**: experimental；`auto.source.text-record.draft` revision 1（PLAN-782）。
> 实现：`experimental/ace/crates/core/src/source/` + `verify.rs::verify_text_record` +
> `text_resources.rs`（资源走查）+ 共享 MIR lowering（`auto-mir-lower` → `ac-mn`，
> PLAN-871 起旧 `native.rs` 合并 lowering 已退役）；消费端见
> [project.md](project.md)，后端选择/退役规则见
> [native-backend-and-retirement.md](native-backend-and-retirement.md)；前置 profile 见
> [source-text.md](source-text.md)（770）与 [source-record.md](source-record.md)（771）。

本 profile 在 text 与 record 两个已交付 profile 之上交付 **拥有 Str 字段 record
的首批有限组合语义**：i32/bool/char/Str 与非递归 record 同域组合、派生资源
分类、完整根显式 move/深 clone、字段视图与替换、全路径正常清理。资源语境以
774 冻结合同（[data-resource-contract.md](data-resource-contract.md)）为唯一
接口来源；本文件只定义组合源码域的语法/语义/诊断合同。**不宣称** enum/List/
Map/use/多模块、用户泛型、部分字段 move、借 str 字段、递归堆 record、可变
实例方法、unit/隐式尾返回、record equality、GC/RC、生产 ABI、ACC 桥、真实
Token/lexer 迁移或 Gen1-3。

## 身份

- source `auto.source.text-record.draft` → HIR `auto.hir.text-record.draft / 1 /
  text-record-i32-draft`（descriptor `experimental/ace/crates/core/schema/text-record.atom`
  = text ∪ record 全节点集 + `record_clone`/`move` 双新 case，旧四 descriptor
  零改动）。
- 身份映射为**精确可失败映射**：未注册三元组 `bind.descriptor-unknown`；
  canonical writer 对未注册/错域拒绝（`canonical.identity-conflict`）；四旧
  profile 不自动获得组合能力（`verify.domain` / `text/source.unsupported`
  前门，`verify.profile-op` 双向生效）。

## 类型与资源分类

- 字段类型：`i32`/`bool`/`char`/`Str`/已声明 record（前向引用合法；图无环
  沿 771 `verify.record-cycle`）；`str` 字段拒绝（`verify.field-borrow`）；
  空 record/默认字段沿 771。char 不隐式转 int。
- **资源分类是派生属性**（禁止信任输入自称）：对每个 record 做传递字段
  闭包——任意可达字段含 `Str` ⇒ 资源 record；仅标量/无资源嵌套 ⇒ copyable
  record（771 值复制语义原样保留）。copyable record 不可能包含资源 record。
- `move` 参数模式合法当且仅当形参类型为 `Str` 或资源 record；copyable/
  标量的 move 模式 → `verify.ownership-required`。

## 构造与拥有表达式

- 位置与命名构造均支持（源码序求值再映射；逆声明序命名合法）。
- **拥有形态 allowlist**（let/assign/return/ctor 字段四个拥有位统一）：
  合法 RHS = 新构造（RecordCtor）、深 clone（`r.clone()`）、拥有调用返回、
  `x.move` 包装。其余资源 record 值形态——裸局部读取
  （`verify.ownership-required`；纯 owned `Str` 局部另见
  [source-text-replacement.md](source-text-replacement.md) 的 var 替换准入）
  与**字段读取**（FieldGet=根内聚合的视图，
  永不隐式复制 owner；同码族）——一律前门拒。
- `clone()` 内建按（record 接收者）绑定：显式递归深复制（逐 Str 字段独立
  buffer，标量按值；空 Str 零分配）；**用户声明 `clone` 方法 →
  `source.duplicate`**。
- 完整根 move：`x.move` 在实参/let/assign/return 值位整体转移（HIR 单一
  `expr.move` 包装表示；`CallBinding.moved` 在本域禁用）；move 后原根不可
  读/不可重复 move（`verify.use-after-move`/`verify.double-move`）；字段
  路径/子表达式 move → `verify.partial-move`（**未执行分支同样静态检查**）。

## 字段读取、借用与替换

- Str 字段读取 = 借用视图（HIR FieldGet 的 `Str`/`str` 两拼写等价，lowering
  均装载视图）：len/char_at/slice/eq/ne/to 等 770 面以字段为接收者。
- 借用 str 返回来源：存活 view 参数的**字段**或静态字面量合法；局部/拥有
  临时/move-in/mut 参数的字段 → `verify.borrow-escape`。
- 字段替换（Str 字段位）：根 = var/mut 参数/move-in 参数；RHS 先完整物化 →
  旧缓冲恰一次 free → 提交；活跃字段借用期间整个根替换/移动保守拒绝
  （`verify.mut-conflict`，按根不拆分）；let/view 根拒
  `verify.immutable-write`；标量字段写沿 771。
- **字段写永不复活已移出的根**：目标根在本语句 RHS 被 move 或此前已 dead
  时，任何字段位（Str/嵌套资源 record/标量）的赋值 → `verify.use-after-move`
  （部分重建不恢复其余字段责任）；只有**完整根重赋**可接收返回/新构造值。
- mut 排他对（含两层包装传播：同源拒/异源过，按 SOURCE 不按形式参数号）
  沿 770 fixpoint，owner 集合扩至资源 record 根。

## 清理责任不变式（沿 770 全套 + 组合扩展）

- join/循环重入 RE-ENTRY fixpoint/after-loop 三态/参数对排他/语句尾寿命/
  短路临时路径携带——全套沿 770；owner 集合扩至资源 record 根（含 move-in
  参数=callee 根，出口逐 Str 字段缓冲清理；owned_returns 扩 record 根）。
- **统一创建序 LIFO**：语句临时（文本 buffer 与 record 聚合）单一 pending
  表按创建序释放；函数出口逐字消费计划的交错 buffer_frees（跨类型创建序，
  根内字段逆声明）。
- **短路聚合临时路径存在标记**：LogicalRhs 合流为聚合临时携带存在标记
  （join 参数：执行边 1、跳过边 0）；**标记跨嵌套合流组合**（外层 join
  转发内层标记值、仅无条件创建传 1、外层 skip 恒 0）——循环 take→skip 重入
  零释放、栈槽陈旧指针不可达。
- 聚合 move-return 重建：RHS move 整根后由被调方返回的聚合携带原缓冲，
  不安排 replace_frees（同语句/fresh 目标同形）。

## 故障双通道（组合域）

- **注入**：build 工具测试面 `--rt-fail-nth-alloc <n>` /
  `--rt-set-budget <bytes>` / `--rt-fail-ledger-nth <n>` 于 startup wrapper
  前奏调用**既有** runtime 控制符号（23 符号内；无语言入口、无新 runtime
  符号；生产构建全缺省关闭）。
- **部分输出回滚**：组合域内 runtime status 失败（负 len 返回值）时，
  失败语句的**部分输出**（pending 临时 = ctor 字段实参缓冲/clone 逐字段
  克隆）在 fatal trap 前按创建序逆序回收（RecordClone 置零 Str 槽位并先行
  挂聚合条目；空槽 null 为 own_free no-op）；**旧目标不提交**（replace_frees
  保持 RHS 后）。回滚后仍 RT-FAIL + ExitProcess(70)（无语言级 Result/
  unwind；live 根 owner 依赖 OS 回收，不声称 fatal 前 live=0）。纯 text 域
  trap 合同原样（回滚门控组合域）。
- 失败类别按 runtime 冻结码区分（env-OOM -5 / ERR_BUDGET -4 等）。

## 能力与运行期合同门

- 组合模块（任意资源构造/clone/清理义务）派生 `requires:
  ["ac.runtime.text.v1"]`（纯 copyable 模块零需求；caller 声明不可绕过——
  双触发门沿 770）。合同 sidecar/target/hash 校验门原样。
- 布局（D-05 适用域）：Str 字段 `{ptr u64, len u32}`=12/8；资源 record
  8 对齐、copyable 嵌套保持 4 对齐；声明序公式 + checked u64/u32
  （`record.layout-overflow`）；**标量 copy（赋值=拷贝）仅限无资源类型**。
- ABI 全复用：ABI-A/B/C/D/E；move-in 资源 record 参数机器形 `*mut`（区别
  在 verify 责任簿——callee 出口清理字段缓冲）；无聚合 sret。

## 边界

- 无 enum（TokenKind）/List/Map/use/多模块/用户泛型、部分字段 move、借 str
  字段、递归堆 record、可变实例方法（`mut fn`）、unit/隐式尾返回、record
  equality、GC/RC、生产 I/O/ABI/DLL、ACC 桥、完整 lexer/parser、Gen1-3。
- 真实 Token/lexer 迁移不宣称（Token 形状微程序仅为内部数据能力取证）。
- 畸形/越界引用（含 Move.inner、binding 越界）在**结构界检**受控拒绝
  （`verify.bad-reference`），不 panic、无 Checked。

## 验证状态

- 测试族：text_record_hir（18，含 5 个真实程序化变异）/text_record_source（21）；
  旧 text_record_resources（27）/text_record_native（24，含故障注入）随 PLAN-871
  旧 lowering 退役，其资源台账/故障 oracle 由 `ac-mir-native` 的 shadow_tier_871
  （owned 行台账所有权不变式）与 `gap_closure_871`（view clone / owned 字段读取 /
  HP01 HIR 直连）承接；联合门禁 `scripts/verify-ac-text-record-782.ps1`（步骤见脚本，
  含 770/771/741/745/767 旧门与 770 scope 归因；后继计划树用
  `AUTO_LANG_PLAN_TREE=<NNN>`）。收据见
  [../../reports/782-text-record/verification.md](../../reports/782-text-record/verification.md)。
