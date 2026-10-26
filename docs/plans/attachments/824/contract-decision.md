# PLAN-824 contract-decision.md — T-01 支持矩阵/身份/预算/oracle 冻结

2026-10-26,work stage T-01。用户指令"计划824 实施"即实施授权。本文件按计划 r6 §8
T-01 冻结:实际基面核对、T-10 支持矩阵与预算、身份/编号、访问语义、portable wire
布局、错误分态、case→入口→oracle 映射、同源模块/harness 接缝与写面登记。T-02 集成
门(真实 verify→ABC→AV/store/new-worker/AutoDown)仍等 D822/D828 + 核心/注册窗口,
本文件只冻结其身份/规则,不冒领其证据。任何变更须回本文件记 revision。

## 1. 实际基面核对(2026-10-26,主检出 master 9c7a7436f)

| 依赖 | 实际状态 | 证据 |
|---|---|---|
| D-821 (place-owned-access) | **满足**:archived r6 9/9,delivery 92b8daf60 | archive/821-*.md;core `contract.rs` owned 面(Value::Char/Str/StrView/MutRef、StrHandle arena hook、AV_OWNED_PROFILE) |
| D-828 (function-incremental-session) | **满足**:archived 11/11 | archive/828-*.md(02af08442/d46e03e8a);增量接口由 T-09 接入 |
| D-822 (typed-collections-variants) | **满足(r2 更新)**:archived r5 9/9,merge 1e1386b1d + cleaned 893f8ac97 + ledger v97(c6c5ddb25) | archive/822-*.md;交付 `av.collections/1`(profile id 4/AVAD v3)、`Value::{List,Variant,Place}`、`MirType::{List,Variant,Place}`、LIST_*/VARIANT_*/PLACE_* 指令、MAX_LIST_ELEMS=65536 |
| D-817/818 (cte-query-frozen-cache / shared-mir-native-parity) | 均 archived,owner 明确 | V-09 兼容消费面:T-05 定向重放,不改动其路径 |

**r2 集成基面**:T-02 起唯一树 `D:/autostack/.wt/lang-824/auto-lang`(分支
`plan-824-dev`)已合并 master(merge 提交 b7837c63d,master 侧至 c6c5ddb25);
T-10 代码提交 29461c706。parallel/824 harness 在 r1 面(821 值集)上的 22 绿
证据按基面绑定,不冒领新依赖 pass——r2 扩展后按 §9 重跑。

唯一实施树 `D:/autostack/.wt/lang-824/auto-lang`(分支 `plan-824-dev`;r1 基面
7513bcee0/T-10 代码 29461c706;r2 起已并 master,见 §1)。生产源两文件 + 独立
harness workspace,全部在本树新增,详见 §8。

主检出预检:无 tracked 代码 WIP(仅未跟踪 fixtures Cargo.lock 与一个未跟踪 fixture
目录,非本计划面);docs/ 上有他轮未提交簿记(含本计划 r5→r6 修订),提交时只携带
`docs/plans/824*` 与 `docs/plans/attachments/824/**`。

## 2. T-10 支持矩阵(容器/冻结白名单)

值面(以已交付 `auto_av_core::contract::Value` 为准,11 变体逐一裁定):

| Value 变体 | pack(is/borrow_as/take_as 载体) | portable freeze | 裁定依据 |
|---|---|---|---|
| Bool/I32/I64/U64 | ✅ | ✅ 精确值 | §5.1.1 白名单标量 |
| F32/F64 | ✅ | ✅ **逐位**(to_bits,NaN payload/±0 不规范化) | V-03 逐位保真 |
| Char | ✅ | ✅ u32 标量域检查(Rust char 域=774 域) | 白名单 char |
| Str(StrHandle) | ✅ owned | ✅ 仅 UTF-8 bytes,**不存** allocation id/hook | §5.1.4 拒进程身份;Str 按字节相等 |
| Record(Rc\<RecordValue\>) | ✅ 纯 owned record | ✅ 名义头(owner/name/字段名序)+ 子树 | §5.1.1;ty:u32 是位置身份不入 wire(§5.1.5) |
| List(ListHandle) | ✅(**r2 起**,D822 已交付) | ✅ 元素子树按序;id/epoch/elem_gens/cap 为运行期簿记**不入 wire**,恢复为 detached 新表(同 Str id 政策) | §5.1.1 白名单 List;元素闭包递归同规(Fn/view/Place 任何深度拒) |
| Variant(VariantValue) | ✅(**r2 起**) | ✅ 名义头(owner/name/臂名序+每臂 payload 有无)+disc+payload 子节点;ty 位置索引不入 wire | Option/Result/有限 enum 均降为此形(822) |
| Place(ListPlaceV) | ❌ pack 递归拒(**r2 新增类别**,运行期索引 token,同 borrow 类) | ❌ | §5.1.4 拒 frame/borrow token;V-12 封闭性 |
| StrView(StrView) | ❌ pack 递归拒绝 | ❌(pack 已挡,结构性不可达) | §5.1.1 borrowed str 拒绝;V-12 record 含借用 |
| MutRef(MutRefV) | ❌ pack 递归拒绝 | ❌(同上) | §5.1.4 拒 frame/borrow token |
| (Fn/闭包) | ❌ 类型级无此类别(DynType 不含 Fn 变体) | — | §5.1.1;审计§4 |

类型面 `DynType`(本计划新增,dynamic.rs 内自足有限树):

```text
DynType ::= Bool | I32 | I64 | U64 | F32 | F64 | Char | Str
          | Record { owner: String, name: String, fields: Vec<DynField> }   // r1
          | List(Box<DynType>)                                            // r2
          | Variant { owner: String, name: String, arms: Vec<DynArm> }     // r2
DynField ::= { name: String, ty: DynType }   // 声明序;重名字段 = 类型不良,pack 拒
DynArm   ::= { name: String, payload: Option<DynType> }  // 声明序;重名臂 = 类型不良
```

- List **结构**等价(仅元素型等价,镜像 MirType::List);Variant/Record **名义**
  等价(owner+name+臂/字段声明序),镜像 822/820 的 MirType 冻结规则。
- List 元数沿 822 `MAX_LIST_ELEMS=65_536`(pack/freeze 同规);portable 总节点
  预算(§7)仍为总控,两层不相加、逐层先到先拒。

- 类型等价 = **820 冻结规则**:`DynType` 派生 `PartialEq`(record 比对
  owner_module+name+全字段名与类型声明序)。**同宽异名/异 owner 不相容**;
  bool/int/char/Str 互不猜测转换;`take_as` 用同一等价,V-01 名义身份可判。
- `MirType::Record(RecordTypeRef)` 的 TypeIdx 递归解析在模块外(需模块类型表),
  由 T-02/T-03 生产 adapter 完成;T-10 不假造该解析。
- Record 值为**非递归纯值树**(820 构造面如此):值有限,DynType 有限,无递归类型项。
- pack 语义:value+type **成对准入**——递归白名单(StrView/MutRef 任何深度拒绝)
  + 值形与 DynType 结构一致(record 元数/字段类型逐一);失配=Invalid 类,不是
  unsupported。pack 按值消耗(Rust move);"pack 非 Copy 值须 move 或显式 clone"
  = borrow_as 只给共享引用,复制必须显式 `deep_clone`(§3)。

## 3. 访问语义(pack/is/borrow_as/take_as/deep_clone)

| 操作 | 签名要点 | 失败语义 |
|---|---|---|
| `pack(value, ty) -> Result<DynValue, DynPackError>` | 唯一 `DynValue` 构造入口(字段私有);move 语义 | Err 时 value 已被消耗性检查但**无泄漏**:非法类别在返回前 drop,资源 handle 正常释放;无部分构造体逃逸 |
| `is(ty) -> bool` | 用 §2 等价 | — |
| `borrow_as(ty) -> Option<&Value>` | 共享引用,**无释放权、无复制权** | 类型不符 → None,源不动 |
| `take_as(self, ty) -> Result<Value, DynTakeError>` | 受检恢复,消耗 self | **错型:Err 携带原 DynValue 原样返回**(原值仍可读/再 take,未 drop)——运行期 checked 失败而非坏输入(§5.5);V-01/12 主断言 |
| `deep_clone(&self, opts) -> Result<DynValue, PortableError>` | 结构深拷贝(独立 Rc/buffer),预算/取消/展开节点计数(共享 DAG 展开=树扩展) | 超预算/取消 → Err,源不动 |

共享 DAG(合法纯值共享,820 语义)在**容器内合法**、在 **portable 边界拒绝**
(§4);这是两条不同门,测试分别断言(V-04 共享图拒于 freeze;pack 正控通过)。

## 4. portable 冻结 wire 布局(PORTABLE v1,小端)

身份(本轮冻结,与 §6 一致;`AVPF`/profile 4 经全仓 grep 无冲突):

| 项 | 冻结值 |
|---|---|
| portable schema | `auto.portable.av-dynamic/1`(常量在 portable.rs,生产源内) |
| magic / version | `AVPF` / 1;kind=1(DynValue 树);flags=0(未知位拒绝) |
| 校验 | 尾部 u64 FNV-1a(core `contract::fnv1a64`,同源原语) |

```text
header: "AVPF"(4) u16 version=1 u8 kind=1 u8 flags=0 u32 node_count u32 reserved=0
body(DFS 先根序,根在最前;node_count=值节点总数,读头后分配前预算检查):
  tag u8:
    0 I32 +i32 | 1 Bool +u8∈{0,1} | 2 I64 +i64 | 3 U64 +u64
    4 F32 +u32 bits | 5 F64 +u64 bits | 6 Char +u32(域检查)
    7 Str +u32 len +bytes(decode 强制 UTF-8 校验)
    8 Record +u16 owner_len+bytes +u16 name_len+bytes +u16 nfields
              +每字段 u16 name_len+bytes(声明序;重名=Invalid)
              +nfields 个子节点(先根序)
trailer: u64 fnv1a64(全部前序字节)
```

- decode 顺序:**界限/预算后语义**(§5.1.5):magic/version/kind/flags →
  node_count/输入字节预算(分配前)→ 逐节点结构(未知 tag/布尔域/char 域/UTF-8/
  重名字段/截断/尾随字节/checksum)→ RestoreContext 准入(§5)。
- 不存:进程地址、allocation id、arena hook、ExecTypeIdx、`RecordValue.ty`
  位置索引、frame/borrow token、任何 trusted 凭证(§5.1.4)。
- freeze 确定性:同内容同类型 → 逐字节相同(独立构建的两实例亦然,V-14 同源 hash);
  unfreeze(freeze(v)) 与 v 满足 `Value::eq`(位型逐位、Str 按字节、record 名义
  ty+fields)+ `DynType::eq`。
- **唯一入边检查**(freeze 时 visited-set:Rc\<RecordValue\> 与 Rc\<StrBuf\> 指针):
  同 owned 子节点双引用(含无环 DAG)/环 → `portable.shared-graph` 拒绝,
  **原 DynValue 不消耗**(freeze 取 &self),仍可 take(V-04/12)。
- 恢复:每 record 节点经 `RestoreContext` 重验当前名义依赖并解析位置索引(§5)。

## 5. RestoreContext(新执行域准入)

```text
RecordBinding { owner, name, fields: &[&str](声明序), ty_index: u32 }
RestoreContext<'a> { bindings: &[RecordBinding<'a> }   // 只读,无凭证
unfreeze(bytes, &ctx, opts) -> Result<DynValue, PortableError>
```

- (owner,name) 未命中 → `portable.unknown-type`(准入拒);wire 字段名/序与
  binding 声明不符 → `portable.layout-mismatch`(布局/revision 变化受控拒绝,
  §5.1.5,不 reuse 旧 offset);命中 → RecordValue.ty = binding.ty_index。
- 恢复的 Str 一律 `StrHandle::detached`(id=0,无 arena hook)——模块级无 ledger,
  与 core 对 out-of-execution 拷贝的既有语义一致;T-02+ 真实执行域由 arena 铸造
  attached handle 并报台账,届时 free 计数入 V-07 oracle(本轮不冒领)。
- 恢复不反序列化凭证、不猜 handle;坏 bytes 不消耗输入(&[u8] 借用)。

## 6. T-02 身份冻结(r2 修订:冲突回记)

| 项 | 冻结值 | 状态 |
|---|---|---|
| 执行 profile | `av.dynamic/1`(id **5**——r1 冻结的 4 已被 822 `av.collections/1` 占用,按 §6 预案回记) | 冻结;本轮实现 |
| MIR/ABC schema | `auto.mir.av-dynamic/1` / `auto.abc.av-dynamic/1` | 同上 |
| ABC wire | `AVAD` 家族 version **4**(v3 已被 collections 占用;data=1/owned=2/collections=3/dynamic=4 互不可读) | 同上 |
| producer | `plan824/av-dynamic` | 同上 |
| 三语义版本 | MIR=1/ABC=1/AV=1(独立轴) | 同上 |
| portable schema | `auto.portable.av-dynamic/1`(magic `AVPF` v1,T-10 已实现,不变) | 已实现 |
| recipe/测试前缀 | `av_dynamic_824`;recipe 名按 822 模板(`ac-av/collections-run-v1` → 本计划 `ac-av/dynamic-run-v1`,登记于实施时以实际 owner 位置为准) | 本轮实现 |

## 7. 预算/取消/错误分态(T-10 模块级适用上限交集)

| 预算 | 冻结值 | 分类码 |
|---|---|---|
| 单 frozen 载荷输入/输出字节 | ≤ **8 MiB**(§5.2 值数据合计口径;分配前检查) | `av.resource.dyn-bytes` |
| 值节点数(freeze/unfreeze/deep_clone 一致) | ≤ **65 536** | `av.resource.dyn-nodes` |
| 值树深度(pack/freeze/unfreeze/deep_clone 一致,pack 与 freeze 同规:packable≡freezable) | ≤ **256**(源值受前门 128 界;所有遍历为迭代式,深度界保证后续递归 drop 栈安全) | `av.resource.dyn-depth` |
| 工作步(逐节点/逐元素计 1;克隆按**展开后**节点计) | ≤ **2 000 000** | `av.resource.dyn-steps` |
| 取消探测 | 每 1024 节点轮询一次 hook(对齐 CANCEL_CHECK_INTERVAL) | 独立 `Cancelled` 态 |
| 故障注入(测试缝) | `fail_after_nodes: Option<u32>`:第 n 节点后失败,分类=同 Budget(资源边界等价,V-06/07 锚) | 同 `av.resource.dyn-nodes` |

错误分态(§5.2"OOM/预算/cancel 与 invalid/unsupported/trap 分态"):

| 态 | 码前缀 | 覆盖 |
|---|---|---|
| Unsupported | `av.dyn.unsupported` / `portable.unsupported` | StrView/MutRef/未知类别(递归);共享图/环 |
| Invalid | `portable.invalid` | 坏 wire:magic/version/kind/flags/未知 tag/布尔/char 域/坏 UTF-8/重名字段/截断/尾随/checksum |
| Admission | `portable.unknown-type` / `portable.layout-mismatch` | RestoreContext 未命中/布局不符 |
| Budget | `av.resource.dyn-*` | §7 表;注入故障同此类 |
| Cancelled | — | 独立态 |

## 8. 同源模块/harness 接缝与写面登记(T-10)

**生产源(本树新增,唯一同源)**:`experimental/ac-av-probe/core/src/dynamic.rs`、
`experimental/ac-av-probe/core/src/portable.rs`。**不注册进 core/lib.rs、不改
父 workspace members**——core 构建不编译它们;由 harness 以 `#[path]` 路径编译
同一源文件(单一生产源,无第二实现)。

**harness(本树新增)**:`experimental/ac-av-probe/parallel/824/` 独立 workspace
(自有 `[workspace]`+Cargo.lock,父 members 不变即无冲突):
`Cargo.toml`(package `av-dynamic-824-harness`,依赖仅 `auto-av-core = { path = "../../core" }`,
与 core 同为纯 std 零外部依赖)+ `src/lib.rs`:

```rust
#[path = "../../../core/src/dynamic.rs"]  pub mod dynamic;
#[path = "../../../core/src/portable.rs"] pub mod portable;
pub use auto_av_core::{contract, owned_text};   // 重导出已交付公共模块
// (owned_text 供 ledger oracle 测试;portable/dynamic 源内只引 contract),
// 使路径编译源的 crate::contract 引用解析(审计§2 同源方案)
```

**只读消费的 core 公开符号**(全部已在 lib.rs 公开,无需任何 core 改动):
`contract::{Value, RecordValue, StrHandle, fnv1a64, CANCEL_CHECK_INTERVAL,
MAX_TABLE_ENTRIES}`、`mir::`(仅 T-03 adapter 用,T-10 不触)。不消费
`av.rs::ModuleView`(pub(crate))/`abc_verify` 私有字段/任何 WIP(822 树)。

**写面登记**:
- T-10 私有写面(已完成)= 2 生产源 + parallel/824 + 本附件;**无公共写面**。
- **T-02 核心窗口登记(r2 起占用,2026-10-27)**:
  - Plan/revision:PLAN-824 r6(本合同 r2);已合入 base:master c6c5ddb25。
  - 公共写面文件:`experimental/ac-av-probe/core/src/{contract,mir,mir_verify,abc,
    abc_verify,av,instr_spec}.rs`(新增 av.dynamic/1 身份/Dyn 指令/typed host
    seam,只增不改旧 profile 语义)、probe 新增 dynamic producer/恢复门与
    fixtures、`crates/auto-lang/src/tests/` 新增 av_dynamic_824 族、
    `scripts/verify-av-dynamic-824.ps1`、recipe `ac-av/dynamic-run-v1` 注册
    (位置以侦察到的 822 实际 owner 为准)。
  - 只读依赖:822 已合入面(Value/MirType::{List,Variant,Place}、MAX_LIST_ELEMS、
    LIST_*/VARIANT_* 指令)、783 ManagedStore、828 增量接口、798 门与预算。
  - 消费者:parallel/824 同源 harness、av_dynamic_824 测试族、av-data 旧
    consumer(T-05 兼容重放)。
  - 开始:2026-10-27;状态:**T-02 公共面已实现并提交(645378ab5/ff120762c/
    9882277dd,plan-824-dev);窗口由本计划继续持有至 T-03..T-06/合入或交接**
    (串行单写;823 会话须与本登记协调)。

**测试命令**(T-10 建立,selected>0 硬要求):
`cargo test --manifest-path experimental/ac-av-probe/parallel/824/Cargo.toml`
(测试名前缀 `av_dynamic_824`;不触发整档 t/tv/tt/tb/tu/taa,符合 2026-10-14 裁定)。

## 9. case→入口→oracle 映射(T-10 可执行子集;其余 Scope 等 T-02+)

| Case(T-10 子集) | 真实入口 | 区分性 oracle |
|---|---|---|
| V-14a 冻结/恢复等价 | parallel/824 `av_dynamic_824_portable_*` | 全白名单值(含 MAX_U64/MIN_I64/NaN payload/±0/±∞/多字节 Str/深嵌套 record)roundtrip:Value::eq+DynType::eq;freeze 二次逐字节相同;跨实例 freeze 同字节 |
| V-14b 错型 take 保源 | `av_dynamic_824_take_*` | 错型 Err 携回原 DynValue→仍可 borrow_as/正确 take;同宽异名 record 拒;bool/int/char/Str 不互转 |
| V-14c 分配失败半图回收 | `av_dynamic_824_rollback_*` | fail_after_nodes=k→Err(Budget,dyn-nodes 类);**arena ledger oracle**:attached Str 源值 freeze 失败后 live 不减、free_events 空(源未消耗无泄漏)、take 后字节可读、终放 live=0;**bytes 未消耗**:同输入重试全成功(无半安装状态);半图回收为所有权结构性回滚,T-02+ 真实 arena 恢复时 free 计数入 V-07 oracle(不冒领) |
| V-14d 递归拒绝 | `av_dynamic_824_reject_*` | record 含 StrView/MutRef 任意深度 pack 拒(`av.dyn.unsupported`);共享 DAG(双引用 Rc)pack 过、freeze 拒且源可 take;Fn/List 无类别不假造(类型级缺席断言) |
| V-14e wire 负控 | `av_dynamic_824_wire_*` | 未知 magic/version/kind/flags/tag、布尔域、char 域、坏 UTF-8、重名字段、截断、尾随字节、坏 checksum → `portable.invalid.*`;RestoreContext 未命中/布局不符 → admission 拒;**坏 hash 好内层同拒**(checksum 最后但结构检查不可跳过) |
| V-14f 预算/取消 | `av_dynamic_824_budget_*` | node_count+1/超 8MiB/超步数/第 1024 节点取消 → 各自分态;预算前拒 attempt=0;deep_clone 展开计数 |

T-02+ 才有的面(V-01..V-13 全量、profile 执行、store/new-worker、AutoDown、P95/
内存、增量):本表不含,不冒领。V-12 在 T-10 的适用子集=V-14d/e(递归拒绝+重名/
缺字段 wire 检查);真实 Closure/保留 tag 组合按审计§4 由最后集成者验。

## 10. 边界重申

- 不复制 core 值/IR/store/VM;不 unsafe;不依赖 822 树任何代码;不改 796/817 冻结
  schema;817 scalar 结果合同不动(V-09 归 T-05)。
- T-10 通过 ≠ profile/Plan 交付(计划 §5 两阶段门);本文件 §6 身份在获窗后如与
  当时 master 冲突,回本文件记 revision 再实施。

## 11. r1 实施修订记录(2026-10-26,T-10 落地 29461c706)

T-10 实施确认/细化以下决策(不改变 §1–§10 已冻结合同的性质,均为其具体化):

1. **深度预算入表**:新增 `MAX_DYN_DEPTH=256`(`av.resource.dyn-depth`,§7 已补行)。
   依据:值树递归 `Rc` drop 的栈安全——迭代遍历消除构造侧风险,深度界消除释放侧;
   源程序值受前门嵌套 128 界,256 留裕。pack 与 freeze/unfreeze 同规,packable≡freezable。
2. **字节计费口径分离**:freeze 按发射 wire 字节计;unfreeze 输入长度为独立前置
   常量检查 + 重建值估计计费(每节点 1 cell + 每 record 一 Vec 头 + Str payload,
   节点恰计一次);deep_clone 同重建口径。三者各自 ≤8MiB,不相加(避免同值双计)。
3. **pack 亦执行深度与类型良构检查**:重名字段/超 `MAX_RECORD_FIELDS`(1024,复用
   core 常量)宽度在 pack 即拒(`av.dyn.invalid-type`),与 wire 解码同规则。
4. **wire 完整性补充**:declared node_count 与实际解析节点数必须相等
   (`portable.invalid.node-count`);`reserved≠0` 拒(`reserved`);owner/name/字段名
   长 u16 域,超长拒(`name-length`);空字段 record(0-field)在流内即时完成。
5. **取消探测**:每 1024 节点轮询一次(复用 core `CANCEL_CHECK_INTERVAL`),节点粒度。
6. **V-14c oracle 定稿**:原草案"Rc strong_count 探针"对全新重建的半图不可观测
   (重建节点与源不共享 Rc)——改为 arena ledger(源保活/零 free 事件/终放归零)+
   同输入净重试成功 两轴;arena 可观测的回滚 free 事件 oracle 归 T-02+ 执行集成。
7. **测试族命名**(22 用例,全部前缀 `av_dynamic_824_`):portable_roundtrip_{scalars,
   str_char,nested_records}、portable_freeze_deterministic、take_wrong_type_preserves_
   source、take_no_cross_kind_coercion、deep_clone_independent_expands_sharing、
   rollback_unfreeze_fail_then_retry、rollback_freeze_failure_preserves_source_ledger、
   reject_nested_borrows、shared_dag_pack_ok_freeze_rejects、type_surface_closed_no_fn_
   or_list(无通配穷尽 match=封闭性编译期断言)、malformed_types_rejected、wire_header_
   negatives、wire_payload_negatives、wire_checksum_and_node_count_integrity、admission_
   unknown_and_layout_mismatch、budget_{nodes_freeze,bytes,depth}、cancel_probe、
   traversal_stop_codes_distinct。实测 22 passed/0 failed,零 warning,rustfmt 通过;
   `cargo check -p auto-av-core` 确认 core 构建面零改动。

## 12. r2 修订记录(2026-10-27,D822 合入后 T-02 集成门刷新)

D822 满足(archive 9/9 + merge/ledger 收据齐全)触发本修订,按计划 r6 T-02
"先刷新 T-01 接口/版本/预算合同,旧准备不冒领新依赖 pass":

1. **身份冲突回记(§6)**:profile id 4 与 AVAD v3 已被 822 `av.collections/1`
   实际占用 → 本计划改为 **id 5 / AVAD v4**(名称/schema/producer/语义版本
   不变)。portable `AVPF`/v1 无冲突,保持。
2. **白名单扩展(§2)**:List/Variant 入白名单(D822 已交付,计划 §5.1.1 原文
   即含);`Place` 新类别递归拒(运行期索引 token,同 borrow 类);List 运行期
   簿记(id/epoch/elem_gens/cap)与 Variant 位置 ty 不入 wire,恢复铸 detached。
3. **基面**:worktree 并 master(b7837c63d);T-10 的 22 绿证据按 r1 基面绑定,
   白名单扩展后须重跑(§9)。
4. **核心窗口登记(§8)**:2026-10-27 起占用,串行单写,823 会话须协调。
5. parallel/824 harness 在 r2 基面因 Value 新变体导致 freeze 匹配非穷尽而编译
   失败——预期状态,由同源模块 r2 扩展(dynamic.rs/portable.rs)修复,非回归。

## 13. r3 实施记录(2026-10-27,T-02 集成门,代码 645378ab5/ff120762c/9882277dd)

D822 满足后 T-02 全链路落地(§5.2 首例 + §5.3 recipe 交付):

1. **核心面**(核心窗口占用中):`av.dynamic/1` 身份(§6 冻结值:AVAD v4/profile
   5/`auto.mir.av-dynamic/1`/`plan824/av-dynamic`)、`Value::Dyn(Rc<DynValue>)`
   (deep_size/PartialEq/type_name 臂)、`MirType::Dyn`(wire tag 16)、
   DYN_PACK/IS/TAKE 指令(0x2F..0x31,ABC 编解码+WireFamily::Dynamic)、验证器
   (`collections_surface()` 谓词覆盖子sume、`dyn_whitelisted` 路径式环检测+256 深度
   界、DYN typing 臂、dynamic entry 合同)、执行器(resolve_dyn_type 类型解析器
   + 三执行臂)、instr_spec 三行+穷尽清单、layout/ownership/program_snapshot/
   cache 的穷尽性补臂。
2. **指令语义落位**(v1 wire):DYN_PACK=显式克隆共享(§5.1.2"须 move 或显式
   clone"的 clone 形——源保持有效,Rc 末主释放);DYN_TAKE 失败=运行期 checked
   `TrapKind::DynTypeMismatch`(`av.trap.dyn-type`),源 Dyn 槽原样保持、零 drop;
   DYN_IS 永不 trap;成功 take 物化共享(Value::clone 语义)。
3. **entry 合同**:params 同 owned(scalar/char);result 可增 Dyn——容器经 Dyn
   过界,不裸过(首例冻结)。
4. **分域存储**:A23(代码 ABC,LayerSpec `ac824/a23-dynamic-abc/v1`,R14/R15/R16
   同款绑定)与 A24(portable 冻结载荷,`ac824/a24-dynamic-portable/v1`,键含
   portable-schema+value-fp 轴)独立层;portable 恢复=unfreeze+当前类型表准入
   (portable_bindings_of)。
5. **首例证据**(av_dynamic_824_first_case 4/4):first-case 分支计分 7+1+1=9
   (pack/is/take 于 Note/List<Note>/Opt);台账冻结 allocs bytes=[3,0]、frees
   LIFO ids=[1,0]、live=0(Dyn 持共享,零额外分配);dyn-out 返回 Dyn→冻结→A24→
   同 store 恢复→unfreeze 与原值相等且冻结字节逐位一致;take-mismatch 陷阱;
   AVAD v4 与 v1/v2/v3 互不可读;decode→re-encode 字节稳定。
6. **recipe 链**:`ac-av/dynamic-run-v1` 注册(auto-test-case 5 处清单;ac_av
   RecipeFamily::Dynamic+av-profile 轴=5+DYNAMIC_CASES 冻结表+judge_dynamic_
   fixture 含 portable_roundtrip 轴;破坏缝 AV824_DYNAMIC_EXPECT_SABOTAGE);
   worker `dynamic-request` 子命令+case-serve dynamic 分支;语料 av-data-824
   (3/3:corpus_green/negative_oracle/worker_faults);脚本
   verify-av-dynamic-824.ps1(guard 自测+first-case+autodown+acceptance 全绿;
   semantics/admission/resources/cache/compatibility Scope 归 T-03..T-06)。
7. **组树**:补 auto-ui/auto-down detached 依赖 worktree(相对路径依赖解析,
   无 junction;主 workspace 首次构建所需)。
8. **归 T-03+ 的面**:V-02..07 反控全集、资源/峰值采样档、V-09 旧 consumer
   兼容重放、V-13 增量(828 接口)。V-08 的恢复重验+依赖转 miss 轴已由
   restore_reverifies 覆盖。

## 14. r4 实施记录(2026-10-27,T-03/T-04/T-05/T-09/T-11/T-06,代码 30c1532de..209367573)

本节冻结 T-03..T-06/T-09/T-11 实施确认的边界事实与裁定(不改变 §1–§13 已冻结合同
的性质;均为其具体化或如实回记):

1. **宽标量进入缝**:profile 常量为 i32/bool/char/Str(798 冻结规则:宽标量常量
   data-profile 专属),I64/U64/F32/F64 经 **entry-param 缝**进入 dynamic profile
   (entry_param_ok 含全部标量形状,§13.3 的具体化)。V-03 位保真正控经
   `execute_entry(args)` 注入——这也是 dynamic 入口参数合同的首个参数测试面。
2. **容器内类别封闭的实际门层**:Fn 作 List 元素/variant 载荷=`mir.type.list-elem`
   /`mir.type.variant-payload`;Fn 作 record 字段=`mir.record-field-type`;StrView/
   MutRef/Place 作字段=`mir.field-borrow`;record 包 Dyn 值=Aggregate `mir.aggregate-type`
   ——V-12 的"递归白名单"在这些类别上由**类型门先于白名单**封闭(更早更严);
   直接 Fn/Place/Dyn 槽位的 pack 仍由 `mir.dyn.whitelist` 拒;RecDyn{d:Dyn} 类型
   合法、白名单拒(真白名单递归臂)。
3. **跨计划穷尽 match 修复(818)**:T-02 核心窗口新增 DYN 三指令后,
   `experimental/ac-mir-native`(817/818 独立 workspace)fingerprint.rs/lower.rs 两处
   MirInstr 穷尽 match 漏臂导致 mn worker 构建红(ac_mn_probe 20 红)。按 820/821/822
   既定跨计划补臂模式修复:fingerprint 纯渲染 + lower 受控拒绝("av.dynamic/1 dyn
   instructions are outside the mn scalar surface"),818 恢复 24/24 绿。T-02 回归面
   未含 818 worker 的教训在案:compatibility Scope 已将其纳入永久重放。
4. **编号并行登记冲突(回记)**:823 树(plan-823-dev e9bb30606)与本树在同一
   D822 合入后窗口(相差 2 分钟)各自登记了 id5/AVAD v4/opcode 0x2F..31。两树不共
   构建,当前无实质冲突;按共同审计"公共编号由核心 owner 统一登记/最后集成者重核
   消费者并集",**merge 时后合者改号**(常量集中于 contract.rs+冻结表,机械操作),
   本条为裁定依据在案。
5. **T-11 消费身份**:D852=archived(landed 4a81d2ebc,ledger v95,早于本树基面
   c6c5ddb25,树内已含);binding.tsv 仅翻 824 自有 D01..D09 行 av 格
   (`gap:av-dyn-824`→`ok:av-dyn-824`)+owner 注追加指针;W/I 行只追加证据指针,
   格值归原 owner;schema/金样/通用合同零改动(下游只追加原则)。
6. **脚本终态**:verify-av-dynamic-824.ps1 共 10 个验证面
   (first-case/semantics/admission/resources/cache/compatibility/incremental/
   baseline/autodown/acceptance),acceptance=9 Scope 子进程聚合+包级 fmt/check
   零警告健康门;必需 ID 恰一次守卫覆盖全部冻结面;0 例/ignored>0/缺 ID 均非绿。
7. **V-10 负控扩展**:expect-sabotage 的 result 轴对 take-probe 行必红(其 ledger
   单事件,rotation 恒等——ledger 轴对该行如实不适用,仅声明可区分轴)。
