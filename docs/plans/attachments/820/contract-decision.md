# PLAN-820 contract-decision.md — T-01 冻结决策（r1）

2026-10-24，work stage T-01（用户授权"实施"）。本文件冻结 `av.data/1` 独立 data
profile 的身份/版本、类型/指令/布局、预算、oracle 与 case→Scope→真实入口矩阵。
后续 T-02..T-09 在此基础上实现并验收；任何变更须回本文件记 revision。未列出项
一律按 plan §5 合同执行，不在下游实现中临时猜规则。

## 0. 依赖门核对结论（2026-10-24 实测）

| 门 | 结论 | 证据 |
|---|---|---|
| D-798 | ✅ 已交付 | `docs/plans/archive/798-typed-mir-abc-av-probe.md` archived；`experimental/ac-av-probe/{core,probe}` 实存，798 合同 `experimental/ac-av-probe/contract-decision.md`（r2..r7 修订链完整，双线内存门 A=202.1MiB/B≈3.5MiB 实测在档） |
| D828 | ✅ 已交付 | `docs/plans/archive/828-av-function-incremental-session.md` archived 11/11；818 T-02 记录：门禁裸 t 5331/5331+tv 163、SD-01..06、ledger v70、合并祖先核验过 |
| 817 | ✅ 已合入 | master `ec6a54c5d..0277673f1`（CTE spec landing + closeout） |
| 818 | 在途 8/11，**非硬前置** | 独立 workspace `experimental/ac-mir-native/`，不持 ac-av-probe 写面；公共注册写面竞争（auto-test-case contract/suite、auto-lang-core test_cases/mod、auto-lang tests.rs）按批次并行合同 §2 合并时串行——818 分支基于无 817 注册 master，其自身记录了 rebase 义务；本计划同规则 |
| ac-av-probe 单写窗口 | ✅ 空闲 | 821–827 全部 drafting 0 步（本计划为串行首位）；无其他在途计划持有 |
| 注册 owner | `crates/auto-lang-core/src/test_cases/ac_av.rs`（814 大迁移后落位）+ `crates/auto-lang-core/src/test_cases/mod.rs` + auto-lang `tests.rs` | 818 T-02 记录确认 git 重命名检测落位；本计划只追加自己的 recipe/身份，串行窗口同上 |
| 跨兄弟消费缺陷 | 不阻塞本计划 | KNOWN-DEBT（2026-10-24）：828 给 auto-av-probe 加 `ac-syntax-core-adapter` 继承依赖后，experimental/ 兄弟 workspace 经 path 链消费会错析到根；818 分支携带四成员字面量修复随其合入。本计划**直接在 ac-av-probe workspace 内工作**（不新建兄弟消费链），不受影响；820 合入早于 818 时该债仍由 818 携带 |
| 主检出预检 | ✅ 代码路径零 WIP | `git status` 仅 docs/plans、docs/design、roadmap 簿记改动 + .autoos 备份 untracked；crates/test/experimental/src 干净 |

唯一实施树（T-02 创建）：`D:/autostack/.wt/lang-820/auto-lang`，分支
`plan-820-dev`，基面 master@`11ba8fd0a`。实施/内部检查/最终 review 同树顺序进行。

## 1. 身份、版本与路由（§5.1.3）

| 项 | 冻结值 |
|---|---|
| MIR schema（data 单元 OwnerScope.schema） | `auto.mir.av-data/1` |
| ABC wire schema | `auto.abc.av-data/1`：magic `AVAD`，u16 version=1，u16 profile id=2 |
| 执行 profile | `av.data/1`（profile id 2；现网仅 1=core-i32，无冲突——冻结） |
| producer | `plan820/av-data`（store 信封校验必填） |
| 语义版本（data 轴） | MIR=1、ABC=1、AV=1；core-i32 的 1/2/1 三元组**不变** |
| 信封 | 沿用 783 store `AVEN` v2 kind/producer/schema/owner/stage/三元语义版本/snapshot 校验；data 单元走独立 producer/schema 轴 |
| 路由矩阵 | 旧 `AVAB` 入口遇非 profile-1 → `abc.decode.profile`（现状保持）；新 `AVAD` 入口遇 version≠1 / 非本 producer schema → `abc.decode.version`/`abc.decode.schema`；**两 wire 族互不可读**（magic 不同，天然隔离）。旧 core-i32 语义/版本/拒绝矩阵零改动 |
| 缓存键 | data 单元键携带 producer/schema/语义版本/profile/类型表指纹轴；与 core-i32 键空间不相交，无歧义旧缓存问题 |

## 2. 类型系统与等价（§5.4/V-11）

`MirType` 新增：`I64`、`U64`、`F32`、`F64`、`Record(RecordTypeRef)`：

```rust
RecordTypeRef { owner_module: String, name: String,
                fields: Vec<MirFieldDef { name: String, ty: TypeIdx }> }
```

- **类型等价**（全部消费点同一规则：Const/Copy/Bin/Un/Call/Return/Aggregate/
  FieldGet/FieldSet/entry/导入签名）：结构相等（intern 归一，primitive 重复条目
  天然统一）；record 名义身份 = (owner_module, name, fields 逐项完全相等——字段
  名+声明序+类型索引)；**同宽不同 owner/不同名 record 不可互换**
  （`mir.type-mismatch`，verify 阶段拒绝，dispatch=0）。Fn 比较参数/返回逐项
  结构相等 + import effect 一致。Bool 与 I32 恒为不同类型，不借 I32 命名隐式
  扩大（bool 槽接 i32 Bin → `mir.type-op`）。
- **record 类型图**：字段类型限标量（6 种）/已存在的其他 record 类型索引；
  无环（verify 迭代检测，work-step 计费；自环/互环/断开环均
  `mir.type-cycle`）；空 record（零字段）首版**不支持**（`mir.record-empty`，
  §5.5 r3——不作为已支持能力，后续支持须显式 revision 定零 size/stride 规则）。
  每类型字段数 ≤1024（`av.resource.table-entries` 族内防护）；类型表属于模块
  表族合计 ≤65536（§5.5：族合计，非每类独享）。
- typed fixture 是**未信任输入**：手构 MirModule/ABC 与源码产物走同一
  verify_mir/verify_abc 授信，无旁路。宽标量（i64/u64/f32/f64）仅经 typed
  fixture 进入（源码 record 域只有 int/bool），不伪称 Auto 源码支持。

## 3. 常量与位型（V-03）

`MirConst` 新增 `I64(i64)`、`U64(u64)`、`F32(f32)`、`F64(f64)`。

- f32/f64 **位型保真**：wire 编码 `to_bits()`（u32/u64 小端），解码
  `from_bits()`；一切比较/断言用 `to_bits()`（NaN payload 保真，±0 保真，
  不 canonicalize）。本 profile 对宽标量无算术（仅 const/copy/call/return/
  entry），不会产生 NaN 传播改写。
- bool 常量仅 0/1；i64/u64 全宽不截断。
- V-03 冻结位型样本：`u64::MAX = 0xFFFF_FFFF_FFFF_FFFF`、
  `i64::MIN = 0x8000_0000_0000_0000`、`f64 -NaN(payload) = 0xFFF8_0000_DEAD_BEEF`
  （负 quiet NaN，非默认 payload）、`f64 = -0.0 (0x8000_0000_0000_0000)`、
  `f32 = NaN(payload 0x00DE_AD00) = 0x7F80_AD00`… 实现 fixture 以常量表
  逐位断言 store→ABC→call→return 往返相等。

## 4. 指令集与 InstructionSpec 合同表（§5.4/V-11）

保留 opcode（两 profile 共用编码语义，data 侧仅放宽操作数类型面）：
`01 CONST dst,const | 02 COPY dst,src | 03 BIN sub,dst,lhs,rhs | 04 UN sub,dst,
operand | 05 CALL callee,dst,nargs,args* | 06 TRACE event`；terminator
`10 JUMP t | 11 BRANCH cond,then,else | 12 RETURN value`。

新增（**仅 data profile**）：

```
07 FIELD_GET  dst, base, u16 path_len, path*(u32 typeidx+u16 field…见 §5 wire)
08 FIELD_SET  root, u16 path_len, path*, value
09 AGGREGATE  dst, u32 record_ty, u16 nfields, operand*(u32 槽)
```

InstructionSpec 合同表（`core/src/instr_spec.rs` 静态表 + 覆盖检查；T-03 实现，
V-11 断言 codec/verifier/loader/executor 四层对每个 opcode 都有登记覆盖——
漏项使覆盖检查测试失败）：

| opcode | 指令 | operand 域/角色 | 类型规则 | read/def/kill | 借用/owner | trap | fuel | 来源 | profile |
|---|---|---|---|---|---|---|---|---|---|
| 01 | CONST | dst:W, const:R | const 类型 == dst 槽类型 | def dst | 全 Copy | 无 | 1 | HIR 常量/fixture | 双 |
| 02 | COPY | dst:W, src:R | src 类型 == dst 类型（含 record 名义） | read src, def dst | 全 Copy（浅 Rc 克隆=语义深拷贝，不可变） | 无 | 1 | HIR let/赋值/参数 | 双 |
| 03 | BIN | sub:R, dst:W, lhs:R, rhs:R | i32 算术子码限 I32×I32→I32；比较子码限同标量类（I32 或 Bool 的 eq/ne）→Bool；**宽标量操作数拒绝** `mir.type-op` | read lhs,rhs; def dst | Copy | i32 溢出/除零族 | 1 | HIR Binary | 双（data 含 i32/bool 面） |
| 04 | UN | sub:R, dst:W, operand:R | NegI32 限 I32；NotBool 限 Bool | read operand; def dst | Copy | NegI32 MIN 溢出 | 1 | HIR Unary | 双 |
| 05 | CALL | callee:R, dst:W, args:R* | 实参逐项类型等价被调签名（含 record 名义）；结果类型==dst | read args; def dst | Copy（聚合按值） | 被调 trap 传播 | 1+被调 | HIR Call | 双 |
| 06 | TRACE | event:R | 仅 trace 能力 | — | — | 无 | 1 | HIR 测试内建 | 双 |
| 07 | FIELD_GET | dst:W, base:R, path:R | base 槽类型==path 首段 owner record；逐段名义 owner 校验；dst 类型==终点字段类型 | read base; def dst | Copy（rvalue 值拷贝） | 无 | 1 | HIR FieldGet | data |
| 08 | FIELD_SET | root:W, path:R, value:R | root 槽类型==path 首段 owner；终点字段类型==value 类型；root 必须 Local/Param 槽 | read root,value; def root | Copy（结构重建替换；value 已物化在槽=重叠不变式天然成立） | 无 | 1 | HIR Assign FieldPath | data |
| 09 | AGGREGATE | dst:W, ty:R, operands:R* | operands 数==字段数且逐项类型等价（源序求值已由 adapter/fixture 完成，本指令按**声明字段序**消费 operands 映射字段） | read operands; def dst | Copy（新不可变聚合） | 无 | 1 | HIR RecordCtor | data |
| 10/11/12 | term | — | BRANCH cond 限 Bool；RETURN 类型==签名结果 | — | — | — | 1 | HIR 块树 | 双 |

- BIN/UN **不新增任何子码**：宽标量出现在 BIN/UN → verify `mir.type-op` 执行前
  拒绝。data profile 继承 core-i32 的 i32 算术/bool 逻辑原样。
- kind/entry 篡改：wire `FLAG_IS_UNIT` 载荷语义沿用 r5/r6 合同（unit 类别任何
  公开入口 dispatch=0）；entry 签名按 profile 合同（§6）校验，篡改 → 重验拒绝。

## 5. AVAD wire 布局（v1，小端）

```
magic "AVAD"(4) | u16 version=1 | u16 profile=2 | u16 flags(bit0 sidecar, bit1 IS_UNIT) | u16 reserved=0
u32 n_types, n_consts, n_imports, n_funcs, entry
u16 n_caps + 能力表（同 AVAB）
types:  u8 tag(0=I32,1=Bool,2=Fn,3=I64,4=U64,5=F32,6=F64,7=Record)
        Fn: u32 nparams + params(u32)* + u32 result
        Record: u16 owner_len + bytes + u16 name_len + bytes + u16 nfields
                + (u16 fname_len + bytes + u32 field_ty)*
consts: u8 tag(0=I32:i32, 1=Bool:u8, 2=I64:i64, 3=U64:u64, 4=F32:u32bits, 5=F64:u64bits)
imports/funcs/slots/instrs 01..06/term：同 AVAB v2 编码（tag/opcode 域扩展）
07 FIELD_GET dst,base,u16 plen,(u16 field_idx)*   // 路径段=声明内字段索引
08 FIELD_SET root,u16 plen,(u16 field_idx)*,value
09 AGGREGATE dst,u32 ty,u16 nfields,(u32 slot)*
sidecar/trailer/分页/解码顺序/分配前配额：同 AVAB v2 合同（§4 798 CD 逐条适用）
```

- 路径段用**声明内字段索引**（u16）；owner/名义校验由 verify 对照类型表执行
  （wire 不重复携带 owner 字符串——类型表是唯一身份源）。
- 解码纯结构：未知 tag/opcode/子码、bool 合法性、checked 长度算术、索引域、
  尾随字节、校验和、16MiB 解码堆前置（r4 合同适用）。
- **不 reinterpret**：wire 只描述逻辑结构；物理存储=执行器 Value 枚举。

## 6. 布局、Place 与执行表示（§5.1）

- 新 `core/src/layout.rs`：`LayoutDesc { logical_size, align, field_offsets,
  is_copy }` 从**受检类型图**推导（输入自称无效力——构造输入即类型表，无
  Copy/drop/offset 声明面，V-12 伪描述符无处进入）。标量逻辑宽/对齐：
  i32=4/4、bool=1/1、i64=8/8、u64=8/8、f32=4/4、f64=8/8；record 按 C 式
  递增对齐布局（字段 offset 对齐到字段 align，尾部 pad 到 record align），
  **checked 加法**（溢出 → `mir.layout-overflow` verify 拒绝，非法 layout 不
  发布、无 panic）。布局是逻辑描述符：物理存储为 Value 枚举，描述符 offset
  不用于定位内存。descriptor 不入 wire；load/restore 时**重算并校验**，不信
  磁盘（本设计 wire 本就不携带，无 trusted 面可言——重算=verify 推导，缓存
  侧依赖键含类型表指纹）。
- `Value` 新增 `I64(i64)/U64(u64)/F32(f32)/F64(f64)/Record(Rc<RecordValue>)`，
  `RecordValue { ty: TypeIdx, fields: Vec<Value> }` **不可变纯值**；COPY=浅
  Rc 克隆（不可变 ⇒ 语义等价深拷贝）；所有字段读写定位原根（FIELD_SET 读
  根→结构重建→写回根槽；FIELD_GET 拷出值）。
- **slot-bytes 记账扩展**：`deep_size(Value)`（record 全深计；Rc 别名保守
  重复计——over-count 合规，绝不 under-count）；每次槽写按
  `deep_size(new)-deep_size(old)` delta 调整 `slot_bytes`；预算仍
  `MAX_AV_SLOT_BYTES=8MiB`（槽+聚合合计，分配前检查，peak 上 trace）。
  AGGREGATE/FIELD_SET 构造真实分配，其成本天然入账。deep_size 遍历计执行
  fuel（有界显式遍历，无递归）。
- **entry 合同**（data profile 完整模块）：entry 参数/返回允许全部 6 标量；
  **record 不出 entry 边界**（内部 CALL/RETURN 支持聚合参数/返回）；零参
  entry 合法。fixture 驱动 `execute_entry(args)` 逐位核验（V-03）。core-i32
  entry 零参/i32 合同不变；`execute_entry` 参数类型准入扩展为按 profile 分派。
- unit 模式、闭包装配、imports 闭合语义：沿用 798 r6 合同原样。

## 7. HIR adapter 支持面（record 源域）

- 入口：`auto_ac_prototype::source::compile_record_source`（实存于
  `experimental/ac-core/src/source/adapter.rs`）→ `CheckedModule`
  （`HirDomain::Record`）。前门 `front_door_admit` 计深/输入帽复用；data
  driver 入口 `compile_record_source` 分派（scalar 路径零改动）。
- **支持**：`DefKind::Function`（自由函数）；类型 i32/bool/嵌套 record；
  `RecordCtor`（位置/命名——eval_args 源序求值→bindings 映射→AGGREGATE，
  禁止按声明序重排求值）；`FieldGet`（路径→FIELD_GET）；
  `Stmt::Assign` PlaceKind::Local（→COPY）与 `FieldPath`（→FIELD_SET，写根）；
  Let/Return/If/Loop/Break/Continue/Drop(no-op)；Call（含 record 参数/返回，
  eval_args 源序→bindings 映射）；i32/bool 全算术/逻辑短路（沿用）。
- **拒绝**（显式 unsupported 码，非静默）：`DefKind::Method`（含 static fn/
  实例方法，`lower.method`）；`BorrowView`/`BorrowMut`（`lower.borrow`）；
  `Move`/`RecordClone`（text-record 域节点，本域不出现，出现即
  `lower.expr`）；`mut` 参数模式（HIR 域已由 verify_record 拦借位面，adapter
  防御性拒绝）；宽标量源码（record 源域语法不存在，天然不进）。
  Text/TextRecord/Core/Scalar 域 bundle → `lower.domain`。
- 值捕获/目的地不相交不变式（771 R1/R2）：AGGREGATE/FIELD_SET 操作数均先
  物化于槽（源序），构造/替换时只读已物化值——不变式由指令形状保证；
  V-02 反例验证 `b=copy(a); 改 b; a 不变` 与嵌套 `p.inner.x` 写后原根命中。

## 8. 预算与资源（适用上限交集，798 冻结值零放宽）

| 预算 | 值 | 分类 |
|---|---|---|
| 输入 | ≤4 MiB | `av.resource.input-size` |
| codec 页 | ≤256 KiB | `av.resource.codec-page` |
| 表族 | 模块表(types+consts+imports+funcs) ≤65536；每函数(slots+blocks+instrs) ≤65536；每类型字段 ≤1024——族合计，分配前 | `av.resource.table-entries` |
| 验证 | ≤2M 工作步、wall ≤5s；类型图/路径/布局推导遍历全部计步（迭代，无递归） | `av.resource.verify-steps/-wall` |
| 验证分析内存 / 解码堆 | ≤16 MiB / ≤16 MiB（前置） | `av.resource.verify-memory/-decode-memory` |
| 执行 | ≤1M 步、≤1024 帧、活值数据（槽+聚合）≤8 MiB、wall ≤2s | `av.resource.instr-steps/frames/slot-bytes/execution-wall` |
| 字段路径深 | ≤64（u16 域内显式上限，类型图无环保证有限） | `av.resource.table-entries` 族 |
| trace / 取消 | ≤64 KiB / 每 1024 步轮询 | 同 798 |
| 受管 store 分区 | 32 MiB（hot8/staging16/execution8）沿用 783 硬门 | store 侧 |
| worker 内存 | 双线 A≤224MiB(ac-core owner)/B≤128MiB（798 r7 方案 C 口径）；data worker 相对空载增量计入 B 线实测 | bench 门 |

- 计步定义不变（验证=逐指令/块访问各 1 步+块内字；执行=每指令 1 步；
  AGGREGATE/FIELD_* 各 1 步，deep_size 按字段计）。溢出=resource 分类，
  非 unknown-opcode/非法语法。
- 分态：OOM/预算/cancel/infra 与 invalid/unsupported/trap 分态不变；准入错误
  decode/verify/load 门拒绝、dispatch=0；已准入程序运行期资源/取消按 §5.5
  （先前合法事件保留、危险目标访问/提交 0、无 panic）。

## 9. 存储、恢复与缓存（§5.3）

- 783 ManagedStore：data 单元独立 producer `plan820/av-data`、独立 LayerSpec；
  恢复重走 decode→verify→load 门（重验），依赖键含类型表指纹（类型/字段集/
  layout/profile/语义版本变化 → 键失效，**好 hash 坏 payload 不执行**——envelope
  +深层 admission 双层，沿用 798 模式）。活帧/borrow/宿主地址不入 cache。
- 同 store 新 worker 恢复重验：`av_probe_worker` data 模式 + 新进程恢复
  （V-08：fresh/同 store 重启/依赖变化不复用/好 hash 坏 payload 四态）。

## 10. 注册、recipe 与 AutoDown

- 测试前缀 `av_data_820`（core+probe 两档，`selected>0` 断言防 0 例 PASS）；
  case 目录 `test/cases/av-data/820/`；recipe id `ac-av/data-run-1`；脚本
  `scripts/verify-av-data-820.ps1`，Scope
  `first-case|semantics|admission|resources|cache|compatibility|autodown|acceptance`。
- 注册：`crates/auto-lang-core/src/test_cases/ac_av.rs` **追加** data recipe
  分派（同 worker 二进制 `av_probe_worker` 增加 data case-serve 分支，信封
  `avworker/1` 协议域内扩展 profile 字段；不复制 sema worker、不改 v1/v2
  解码、不触他方 recipe 合同）。父侧期望判定独立（worker 不见 expected），
  判定面扩展：Success 宽标量**逐位**断言（bits 相等，非字符串近似）+ record
  域 completion 分态。公共注册写面按并行合同串行（见 §0）。
- 工具身份：data recipe 工具轴含 worker exe sha256 + `av-worker-version`
  （data worker 版本 `avworker/1.0` 域内）+ recipe + profile 轴。

## 11. oracle（手审，非被测算法生成）

- **首例**（T-02）：record 源码 `type Inner { x: int } type Outer { inner: Inner }`
  `fn main() -> int { var o = Outer { inner: Inner { x: 0 } }; o.inner.x = 41 + 1;
  return o.inner.x }`（exact 源码 T-02 落 fixture 时冻结）→ 手审 expected=42；
  AV Success(I32(42)) 且 FIELD_SET 原根命中（o.inner.x 读回 42）。
  旧 HIR→native 参照按 `probe/src/native_reference.rs` 实际支持面执行
  （scalar 子集对照；record native 未交付，不冒称——首例 native 对照以同源码
  的等价 scalar 化对拍或以手审为准，T-02 落地时按实际面记录）。
- **宽标量往返**（T-02）：§3 冻结位型样本 typed MIR fixture → ABC → AV →
  entry/call/return 逐位 `to_bits()` 相等。
- **V-01..V-13 期望值**全部手审冻结于各 fixture；期望不由被测布局/清理算法
  生成（V-10 反控：错 bits/漏 case/skip/错工具身份 → 总体非零/ERROR）。

## 12. case→Scope→真实入口→oracle 矩阵（必需=本计划验收必跑；适用=代码触面变化时）

| Case | 子例 | Scope | 真实入口 | 阶段/码 | oracle 区分断言 | 必需/适用 |
|---|---|---|---|---|---|---|
| V-01 | 同宽异 owner 互换 FieldRef/AGGREGATE；合法同 owner 对照 | semantics | verify_mir/verify_abc + av_data_820 | verify `mir.type-mismatch`；dispatch=0；对照 Success(42) | 错 owner 执行前拒 + execute 计数 0；合法值 42 | 必需 |
| V-02 | 嵌套 p.inner.x 写；b=copy(a) 改 b；RHS 旧值先物化 | semantics | 全链 + av_data_820 | Success | 原根命中、a 不变、trace/结果固定 | 必需 |
| V-03 | u64::MAX/i64::MIN/-NaN/-0.0 位型往返（store/ABC/call/return/entry） | first-case+semantics | fixture→ABC→AV execute_entry | Success | 逐位 to_bits 相等，不误判 handle | 必需 |
| V-04 | 按值递归类型；size/align/offset 溢出布局；错型字段/非法 bool | admission | verify_mir/verify_abc/layout | `mir.type-cycle`/`mir.layout-overflow`/`mir.type-op`/decode 结构拒 | 受控拒绝、dispatch=0、无 panic、非法 layout 不发布 | 必需 |
| V-05 | 构造缺/重字段、分支单臂初始化、零次循环 | semantics+admission | adapter/verify/AV | adapter `lower.*`/verify init 拒 | 未初始化读取前拒；完整构造接受 | 必需 |
| V-06 | 字段声明序≠源码求值序（trace ba）、嵌套调用未走 trap | semantics | av_data_820 全链 | Success | trace 沿 eval_args；未执行分支 trap 潜在 | 必需 |
| V-07 | cap 内/超 1 单位、瞬态峰值、故障第 n 次注入 | resources | ExecLimits/预算门/分配 hook | Resource 分类 | 预算前拒 attempt=0；故障注入 attempt>0 但 successful_alloc=0；peak 真实；无 panic | 必需 |
| V-08 | fresh/同 store 新 worker 重启/改 type/layout/profile/好 hash 坏 payload | cache | 783 store 恢复→重验→AV | 恢复门 | 合法恢复结果相同；依赖变化不复用；非法执行计数 0 | 必需 |
| V-09 | core-i32 旧 profile 重放；817/818 已合入 scalar 路径重放；data 送旧 consumer | compatibility | 原有 recipe/consumer | 原码位 | 旧结果/诊断不变；未知 profile 明确拒绝 | 必需 |
| V-10 | worker 错结果/错 bits、漏 case/skip、错工具身份 | autodown | 正式 recipe + 父侧 oracle | 父侧判定 | 每项错都使总体非零/ERROR；缺工具/路径不可绿 | 必需 |
| V-11 | 重复 primitive 归一判定；同宽异名/错 owner（Const/Copy/Call/Return/Aggregate 全点）；缺失指令合同项；kind/entry 篡改；合法别名索引 | semantics+admission+cache | 四层覆盖检查 + 全消费点等价 | `mir.type-mismatch`/覆盖检查失败/decode 拒 | 类型判定全部消费点一致；错名义执行前拒 dispatch=0；合同漏项=覆盖检查红；好 hash 不绕过 | 必需 |
| V-12 | 同布局异 owner 与伪 Copy/drop/layout 描述符（构造面不存在=反控确认）；合法构造/拷贝=42；源码空 record 保持既有拒绝；模块/函数族合计 cap 两侧 | admission+semantics+cache | verifier/ABC/AV/store | 源域既有 `verify.*`/资源分类 | 伪造字段不能获复制/布局权限；非法输入授信前拒；合法派生描述符结果 42；资源超额非类型错误；计数按准入/运行分态 | 必需 |
| V-13 | D828 source30 修订基线；本计划两函数局部 body 更新；依赖/layout/授权变化精确失效；强制 whole 前端退化 | acceptance（增量） | 828 session/`lower_function_unit`(data)/edit 接口 | session 计数/身份/IO | 未改对象/代际相同、生产阶段/IO=0；P95<1s；强制退化非零 | 必需（T-09） |

## 13. 本轮不冻结/显式留 T-02 实现核实的项（2026-10-24 T-02 轮核实结果）

- **native_reference record 面（已核实）**：`probe/src/native_reference.rs` 为
  scalar-only（零 record 支持）——首例 native 对照按合同以**手审 oracle=42 +
  值语义反例组**为准，不做 record native 对拍、不冒称。scalar 子集的 native
  对照由既有 core 套件覆盖。
- **B 线内存（T-02 口径）**：data 管线进程内 private-bytes 增量保守归因
  （含测试壳开销）≤128MiB 实测入档；真实 spawn-worker 外部采样峰值归
  T-04/T-06 resources scope（与故障注入 hook 同批）。
- 其余三项中：首例 exact 源码已冻结于 fixture；avworker/1 信封内 profile 以
  recipe id 区分（data recipe 复用信封不扩字段）；828 session data 接口缝隙
  仍留 T-09。

- 首例 exact 源码文本（含 trace 观察点与否）——T-02 落 fixture 时以 §11 形状
  为准冻结并记 hash。
- native_reference 对 record 源码的实际支持面——T-02 按实际面记录对照策略，
  不冒称。
- avworker/1 信封内 profile 字段的具体编码位——T-02 实现最小接线时落定，
  不改 v1/v2 解码语义。
- 828 session data 消费的接口缝隙（`lower_function_unit` record 域 bundle 形
  状）——T-09 实现时按 828 已合入接口对接，不做前置猜测改动。

任何与本文冲突的实现选择都是 needs_fix；支持面/预算/兼容承诺扩大须回 new
显式 revision。
