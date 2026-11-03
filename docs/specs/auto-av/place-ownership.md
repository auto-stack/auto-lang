# av.owned/1 Place 拥有型读写与全出口清理（PLAN-821）

> **Status**: experimental（PLAN-821 交付；独立 owned profile，非默认产品面）

`av.owned/1`（profile id 3，AVAD wire v2；v1↔v2 互拒）在 798 共享核心上交付
owned Place 数据面：char/borrowed str/owned Str 及拥有 Str 字段的 record，
统一 ReadCopy/Borrow/Move/Initialize/Replace/Drop 与 AV 全受控退出的清理。
PLAN-894 起源码面 owned `Str` 局部（var）完整根 `=` 替换经同一 Move 提交
映射准入（前端合同见 auto-ac/source-text-replacement.md；旧缓冲 RHS 后
释放恰一次、新值出口释放恰一次——arena 事件轴实测）。
producer `plan821/av-owned`，语义版本轴 1/1/1 独立；缓存键含 types-fp 轴。
实际 Rust 包位于 `experimental/ace`（auto-av-core / auto-av-probe）。

## 类型与指令

- **类型**：Char/Str/StrView/MutRef/RecordView(RecordTypeRef)（借入 record
  参数，wire tag 12）；资源分类派生不变（Str 与到达 Str 的 record 为 owned）。
- **指令集**：在 820 数据面上扩展 owned 语义（含 MUTREF/DROP/VIEW/
  STR_*/CHAR_* 家族与 FIELDMOVE 0x1C）；合同表在
  `core/instr_spec.rs`（28 行共享表，漏项即红）。
- **FIELDMOVE（0x1C，owned-only）为逐字段消费**：dst 收字段值（record 为
  Rc share、Str 为 handle——share 即转移），字段位回填类型化零值，root 内部
  代际 +1（root 上陈旧 view 退役；不承诺运行期失效句柄校验）。**root 对其余
  字段保持活**——t_MutRet 写回包装按 f0+wb* 逐字段消费，双 mut/owned 返回
  全形状。死亡判定为**前缀重叠**：已移祖先⇒子树全死、已移后代⇒父非完整值
  （父后子/子后父均在授信前 use-after-move，不漏到执行期）；同路径二次抽取、
  部分消费后整根读同红。**完整重初始化重置字段消费态**：任何定义点（Const/
  Copy/Call/Aggregate/Clone/Move/FieldGet/FieldMove/文本族）建立新完整值即清
  字段消费集——新 owner（重聚合/每轮 Call 包装/循环携带重定义）可再次消费；
  分支/回边合并保守并集。

## V-05 句柄域（r6 用户裁定边界）

整数/StrView/MutRef 为 typed 变体互斥，准入类型隔离：整数恒整数、错型
（如 StrView 入整数 BIN）在 MIR 前门 `mir.bin-type` 拒；活 loan 冲突由静态
借用门（`mir.mut-conflict`）拒，BorrowEnd 后合法写与同槽重定义按值/释放
oracle 验证（旧 buffer 先 free、新 buffer 出口 free、各恰一次、live=0）。
StrView 仅 Share/Static buffer 来源；MutRefV{frame,slot,gen} 无解引用消费
入口。**本 profile 不交付运行期失效句柄校验、旧帧复用/跨 store 拒绝、
generation 回绕退役**；内部 gen bump 仅为实现事实，不构成这些保证。

## 访问状态机与运行期

- Return 全终态（dropped 拒）+逃逸按 provenance；Call 结果 view 溯源；
  MutRef 排他双向；owned def 目的地门；转移源 loan 门；FIELDMOVE 双侧
  borrow 门+逐路径 once。
- arena 台账（attempt/successful/fault_at 分账）；静态 view 随带产出单元
  字节；Aggregate/FieldSet 提交后 kill 源槽；FIELDMOVE 经
  `place::field_take`（Rc::make_mut 逐层）搬出+零值回填+代际 bump；
  Const/Clone（字节+字段）/Concat/ToOwned/FieldMove 计 fuel；clone 显式
  工作表迭代（无语言深度截断）。
- 文本接收者 `.len()/len_bytes()/char_at/slice` 对 owned Str 操作数先 VIEW
  （与 to_borrowed 同派发）；view 操作数直通。
- 分析内存：token_roots 与 moved_fields 均稀疏（moved_fields 仅 record 槽
  计入 bound，8192 i32 槽零开销），bound 按真实布局。

## 资源量纲（四线）

worker B 线（父侧采样 private-bytes 峰值 ≤128MiB）与受管三分区各按自身
量纲判定：hot=驻留工作集（wire+MIR+decoded 同时存活，≤8MiB）、staging=
暂态峰（≤16MiB）、execution=guest 槽位峰（AV 自记 slot-bytes，≤8MiB）。
`heap_estimate` 将 RecordView 与 Record 同式计费（owner/name/字段表/字段名
String 堆），abc 解码卫哨逐元素镜像——超 hot 预算的输入不得估算绿。store
`space()` 为磁盘信息列，不代受管内存。

## 非目标

部分字段 move（源值层面）/完整 NLL/生命周期参数/用户析构/循环共享/GC/
List/闭包/Dyn/async/native ABI（见 [project](project.md)）。

## 证据锚点

`experimental/ace/crates/av/tests/`：av_owned_821_semantics（25+V05/v02
定向）、av_owned_821_r1_replay（12，含 mut 循环/重初始化/父子重叠/808/
810/8917/801/RecordView 载荷）、av_owned_821_resources（7）、
av_owned_821_first_case（5）、corpus `crates/auto-lang/test/cases/av-data/821/`。

## PLAN-915：List Place 运行期校验与本 profile 的边界（2026-11-02）

本文件 §V-05 的裁定不变：`av.owned/1` 的 `MutRefV{frame,slot,gen}` 无解引用消费入口，
**不交付**通用运行期失效句柄校验。PLAN-915 不扩这条边界；它关闭的是
`av.collections/1` 的 Index Place 使用期校验（`LIST_PLACE` 铸造的
frame/slot/buf_id/epoch/index/elem_gen 六元组）在 scenario R03 上的正式证据：

- typed、经 `verify_mir`→`encode_abc`→`verify_abc_bytes` 授信的序列，结构性变更
  （push/insert/remove/元素替换）后旧 token 使用 = `av.trap.stale-place`，校验先于任何
  buffer 访问（`observe_places` 下 `buffer_reads` 只计合法读）。
- 合法近邻保持：fresh token 与无变更 mint→read 由本计划实测；活 Share/Static 缓冲的
  可读性是**继承不变量**（place.rs snapshot 单测未改，本计划未新增该面证据）——
  "Snapshot 语义下旧 view 读旧字节"与"List 结构变更后旧 token 失效"是两条不同规则，
  不把全部旧 view 判无效。
- 证据入口：`case-ace-m1-resource-915`；合同见
  [resource-fault-and-generation-conformance.md](resource-fault-and-generation-conformance.md)。---

## PLAN-919 附录：动作观察与最后 owner（2026-11-03）

资源/借用语言面不变；本节冻结**观察面**：`av.lifecycle/1` 在既有执行路径上记录
逐 owner 的 own(created|received)/disown(moved-out|replaced|swept|orphan) 与
buffer free 事实（seq 连续，arena free 钩子覆盖 host drop）。转移≠释放：moved-out
之后同一 buffer id 在接收槽 received；共享 view 下 buffer 只在最后 owner 释放
（槽级 disown 早于 free 事件是合法序）。sweep 按 min buffer id 降序（跨类型 LIFO）
可父侧判伪。观察关闭＝零事件且结果逐字节一致。规范正文见
[resource-lifecycle-observations.md](../auto-test-case/resource-lifecycle-observations.md)。