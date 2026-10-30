# auto.mir native data/owned lowering 合同（PLAN-870）

> **Status**: experimental（PLAN-870 交付；实现在 `experimental/ac-mir-native`
> 的 lower/emit/layout，消费 auto-mir-lower 产出的 av.data/1 与 av.owned/1 单元）

## 准入面

- mn 的代码gen门（`emit::admit_unit`）准入门=三个共享 profile：
  `av.core-i32/1`、`av.data/1`、`av.owned/1`；其余（collections/closures/
  dynamic/entry-unit）在旧通用入口具名拒绝 `mn.unsupported.profile`。PLAN-889新增的
  **显式collections入口**按[独立合同](native-collections-variants.md)受检，不放宽此旧入口。
- 宽数值标量（I64/U64/F32/F64）**触达即拒**（槽/签名/record 字段：
  `mn.unsupported.type`；被引用常量：`mn.unsupported.const`）——R2-N 保持
  关闭；未触达的死表项不拒。`MirType::Unit`（av.entry-unit/1）同样防御性
  具名拒绝。
- 入口冻结为零参 i32 族结果（wrapper `ac_start` 合同）；违者
  `mn.unsupported.entry`。
- 前端按最窄域升级（scalar→record→text-record，仅 `source.unsupported`
  面拒绝升级）——B* 源保持 av.core-i32/1 身份，诊断与 AV 同栈同码。

## 布局与槽表示（与 782 参照逐位一致）

- record 字段按声明序；`Str` 字段 12B/对齐 8（`{u64 ptr @0, u32 len @8}`）；
  标量/Char 4B/4；嵌套 record 按自身 size/align；size 对齐到 align，上限
  u32::MAX；自引用布局具名拒绝 `mn.unsupported.layout`。全部走迭代式
  DFS（无宿主递归）。
- 槽表示：I32/Bool/Char=I32 变量；Str/StrView=(I64 ptr, I32 len_bytes) 对；
  Record=聚合 StackSlot（或 av.owned 参数=入参指针直通）；RecordView=单
  I64 地址；MutRef=零运行时表示（验证期排他）。
- StrView 为**字节投影**（ptr+len_bytes）；标量区间语义由 text_* 运行时
  按标量索引换算保持（len=解码计数、char_at/slice 标量索引）。

## 调用 ABI（mn 私有，Win x64 Fastcall，两端同为 mn 发射）

- I32/Bool/Char→I32；Str/StrView→(I64,I32) 双机器参；Record/RecordView→
  单 I64 地址（data 域 record 参数入口拷贝入私有槽；owned 域 move-in 参数
  =caller move-temp 指针直通，写回经 MutRet 包装的 FieldMove/Move 与
  写穿零化）。结果：标量→I32；Str/StrView→双值返回；Record→尾随隐藏
  `*mut` out 参（无返回值）。纯标量签名的发射与 manifest 渲染逐字节保持
  pre-870 形态（B* 零退化）。

## 所有权纪律（扁平内存上的 AV 语义像）

- **死槽=零化指针**：入口零化全部 owned 可及槽；一切 kill（Move/FieldMove
  出、Aggregate/FieldSet 转移、Str 参数移出）零化源；`own_free(null)` 为
  不计账 no-op。FieldSet 旧字段缓冲与 Move 落已初始化 dst 的旧值在该指令
  处释放（与 AV 单拥有者时点一致）。
- **全出口 sweep**：每个 Return（排除结果槽）与每个 trap 分支臂按
  last-owning-write 指令位置**降序**逐槽释放（record 字段**声明序**）——
  AV 运行期 min_buffer_id 降序帧扫的静态像；trap 块本身仅 ExitProcess(70)。
- 运行时面=landed `ac_text_runtime`（ac.runtime.text.v1 shim，经 probe
  `native_reference::text_runtime_lib()` 定位/构建）；ABI-C 出参槽共享
  4 字节 StackSlot；shim 负返（含预算/注入）→ trap70。av.data/1 与
  scalar 请求**不链**运行时库。

## 台账观测（三级口径，2026-10-28 裁定）

native 侧 alloc/free 事件流=shim stderr（LIT/CLONE/CONCAT/FREE id 行）；
与 AV arena 对拍：①alloc **bytes 序**精确；②free **多重集+单次性**
（每缓冲恰一次、live_after_release=0）；③指令点释放精确。exit sweep
**内部**释放序仅对非 clone 派生根逐序比对——AV 对 clone 派生 record 的
部分释放在宿主 Rc drop 时点混合发火（探针实证），其相对次序是宿主实现
细节而非冻结语义。

## 拒绝面（具名、codegen=0）

`mn.unsupported.profile/type/const/param/result/entry/layout`；旧通用入口仍拒绝collections/
closures/dynamic。PLAN-889的显式collection源码子集/typed Clone见证另按
[native collections](native-collections-variants.md)准入，不冒认整个profile4或宽typed入口。

## 已知边界（登记）

- 视图 `==`/`!=` 源级行被共享 lowering 的通用 Bin 路径阻塞
  （`gap:upstream-s-eq`，S owner；mn 机器面已由手写 MIR 单元实跑覆盖）。
- mut **资源** record 参数的字段替换释放时点：AV（Rc 共享）在 caller
  出口、mn（指针直写）在 callee FieldSet——语料无该形态，触及需裁定。
- trap 多帧回卷：本文旧owned域mn仅清当前帧（AV清全部帧并计入台账）；
  PLAN-889仅为显式collections域实现全部资源caller帧清理，不追认旧域已修。

## 验收锚点

`experimental/ac-mir-native/tests/data_owned_tier.rs`（三路差分 10 行 +
mut 回写三路 + StrCmp/码点机器面 + 拒面/静态反控）；
`crates/auto-lang/test/cases/ace-m1/ace-m1-tr-native.suite.ad`（31 检查 +
负 oracle）。

## 方法消费适配（PLAN-881）

- 方法经共享 lowering 进入 mn 链（escalating frontend → lower → verify →
  split → CLIF/COFF → PE），mn 无 Method 感知；R08=42/Token=57/P-02=75/
  P-05=135 原生实测，产物按内容指纹共存（编辑往返 42→43→42）。
- 必要适配：layout 的 `path_offset`/`path_ty`/`record_layout` 接受
  **RecordView 基座**（view receiver 的字段路径；布局等于底层 record）。
- 非方法 gap（文本 view eq/ne 的 gap:upstream-s-eq 等）独立保留，owner
  不变；不冒领 D871 的全局覆盖。
