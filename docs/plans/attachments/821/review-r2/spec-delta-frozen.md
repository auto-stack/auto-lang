# PLAN-821 SD 草案（R1 修复后重生，供 review 复核 / merge 沉淀）

状态：草案——canonical Spec 写入由 merge 拥有。本文按 R1 修复后的实际交付
（plan-821-dev R1 提交链：e8e1e369d + 后续 R1 提交）整理 before/after 与证据
锚点；与 r1 前草案的差异均已按复审 findings 修正，不再包含被反例推翻的现状
描述。

## SD-01 add `docs/specs/auto-av/place-ownership.md`

- before：无本 profile 规则。
- after（R1 后实际面）：
  - 身份/版本/缓存键：同 r1 草案（av.owned/1=profile 3、AVAD v2 v1↔v2 互拒、
    producer plan821/av-owned、语义版本 1/1/1、types-fp 键轴）。
  - 类型：MirType Char/Str/StrView/MutRef + **RecordView(RecordTypeRef)**（R1-F08，
    wire tag 12；782 ABI-D 借入 record 参数——只读、非 owned、调用无 kill、
    字段视图外部根、字段写入被类型拒）；资源分类派生不变。
  - 访问状态机：Uninit→Init→{Moved|Dropped}；**R1-F01**：Return 检查全部终态
    （dropped 不得以 typed zero 充功返回）且逃逸门按来源 provenance 判定
    （BorrowEnd 不合法化逃逸）；**R1-F02**：Call 结果 view 携带真实溯源
    （view 实参根并集+外部 sentinel）；MutRef 排他双向（活排他阻新共享 view/
    字段读）；owned def（含 StrToOwned/StrConcat）目的地冲突门；owned 转移
    （Aggregate/FieldSet/move-in）源 loan 检查。
  - 运行期：arena 台账（attempt/successful 分账+fault_at 缝）；静态 view 随带
    产出单元已解析字节（**R1-F03**——跨单元常量域正确）；Aggregate/FieldSet
    提交后 owned_kill_slot 源槽（**R1-F04**——静态 kill 与运行时责任一致）；
    **R1-F06**：Const/Clone（字节+字段）/Concat/ToOwned 按冻结单位计 fuel；
    clone 深度上限=MAX_FRONTEND_NESTING。
  - 分析内存（**R1-F05**）：token_roots 稀疏（只 intern 实际使用根）；bound 按
    真实布局计量（8192 槽 0 token：bound≈1.6MB、实测分配峰值≈1.37MB）。
  - 预算/拒绝码族：同 r1 草案 + mir.drop-uninit 族扩展到 Return 终态。
- 证据：semantics 25 + resources 7 + r1_replay 7 + first 5 + corpus 5 check。

## SD-02 add `docs/specs/auto-mir/owned-access-profile.md`

- before：无本 profile MIR 合同。
- after：SD-01 的 MIR 侧细化 + **RecordView 的 MIR 编码**（类型/tag/名义恒等/
  借用参数位置；调用实参单向强制转换 record→RecordView；签名驱动的调用 kill）；
  owned 单元 OwnerScope.schema；typed fixture 未信任授信；entry 合同；部分
  字段 move 源域权威保持。
- 证据：r1_replay（call 溯源/终态/静态跨单元）+ incremental stale-link 拒。

## SD-03 modify `docs/specs/auto-av/cache-and-conformance.md`

- before：core/data producer 恢复规则。
- after 增：owned A21 层 + **R1-F10**：真新进程恢复入口（worker
  `owned-restore-run`——信封→decode→深授信→执行，跨进程同结果）；自洽坏内层
  （重封正确 checksum 发布于好键）由深验证拒绝、dispatch=0；其余同 r1 草案
  （types-fp clean miss/坏 checksum/身份三向隔离）。
- 证据：cache_compat 7 例（含 r1 两例）。

## SD-04 modify `docs/specs/auto-av/project.md`

- before：core-i32/data 两 profile。
- after 增：av.owned/1 登记（含 RecordView 借入面）+ 非目标更新：
  **mut record 参数（782 ABI-E `*mut` 排他写回借用）超出冻结论域**——需要
  写回通道（写穿借用/out-param 类机制）属显式合同修订项（见"待裁定"）；
  其余非目标同 r1 草案。

## SD-05 modify `docs/specs/auto-test-case/architecture.md`

- before：六 recipe 登记。
- after 增：ac-av/owned-run-v1（r1 草案面不变：三缝注册/owned 面判决/冻结表/
  sabotage 双面反控/故障三态）+ corpus 借入正控（run-owned-borrowed exit 15）。

## SD-06 modify `docs/specs/auto-hir/stage-contract.md`

- before：HIR 域语义与 core/data 消费。
- after 增：TextRecord→owned 实际消费支持面——**R1-F08 后**：借入 record 参数
  （`fn read(p Note)`）真实接入（RecordView/无 kill/字段视图/调用后有效）；move
  参数面不变；**mut record 参数（`mut p Note`/`a.mut`）为 lower.mut-record 受控
  拒**——782 源域允许、owned MIR 无写回表达力，按复审裁定以显式合同修订路由
  （不得静默错语义或单方面删支持）；语句尾临时 view 终结（adapter 规范形）。

## SD-07 modify `docs/specs/auto-av/resident-function-install.md`

- before：D828 函数 IR/代码复用、受检安装/依赖/寿命。
- after 增（**R1-F07 后已回填**）：owned 生产安装通道 `owned_session::
  OwnedFunctionSession`——每修订一次真实前端编译、HIR 内容身份（去 span）自动
  失效检测（body 局部变更=单函数重产；类型上下文变更=全模块——模块级 import
  表语义）；真实生产计数（frontend/lower/encode/admit 在真实生产点自增）；
  正式退化检测器 `incremental_gate`（增量集外生产=红；forced whole 经同一
  checker 必红）；原子批量编辑（接口重定型无单点中间形）；拒绝中间形回滚
  （revision/identity 不动）。
- 证据：incremental 5 例（two_fn 自动检测+对象/代际复用+精确 oracle+live=0；
  签名拒绝中间形回滚/一致批量=全模块失效+1101 oracle；forced whole 门红；
  stale-link 单元拒；标量 session 对照）。

## 待用户裁定的合同修订项（复审 F821-R1-08 遗留）

**mut record 参数写回借用**：782 源域 `fn set(mut p Note)`/`set(a.mut)`（ABI-E
`*mut`）要求 callee 写对 caller 可见——需要 MIR 写回通道（写穿借用/out-param/
结果回写约定），超出本计划冻结论域。当前实现：`lower.mut-record` 受控拒（精确
诊断、不静默）。处置选项：(a) 回 new 显式修订 MIR 合同补写回机制；(b) 修订
SD-06 支持面将 mut record 列为显式非目标（需用户授权——782 已验证形态的缩减）。

## 交付边界记录（review 关注项，R1 后更新）

1. ~~陈旧读在受检 MIR 不可达~~（保持：root 保守门禁；值层 Share oracle 在
   core place.rs）。
2. ~~invalid-handle 无消费指令~~（保持：类型层隔离；V-05 实现面）。
3. share 延长下成功出口释放序可非严格 LIFO（恰一次+live=0 恒定）。
4. 预存债清偿：820 resources cap 重校准、818 mn consumer 受控拒适配（r1 前
   完成，保持）。
5. **mut record 写回**：受控拒+合同修订路由（见上）。
6. **V-05 旧帧/跨 store/generation 运行期探测**：冻结指令集无句柄解引用指令
   ——"无消费指令不可达构造"与复审"pending contract reconciliation"判定的
   和解待 review 裁定（若需运行期探测面须扩指令或以故障注入探 MutRefV 域）。
