# auto.mir.av-owned/1 owned 访问 MIR 合同（PLAN-821）

> **Status**: experimental（PLAN-821 交付；实际位于 auto-av-core 的
> mir/mir_verify/ownership，无独立 crate）

[av.owned/1 值面合同](../auto-av/place-ownership.md)的 MIR 侧（含 r6 V-05
静态类型/借用边界与运行期不承诺）。

## 类型与编码

- `MirType` 增 Char/Str/StrView/MutRef/RecordView(RecordTypeRef)；RecordView
  为单向强制转换（借入 record 参数的签名形态），结构同 Record、语义为只读
  借入（无 caller kill、外部根字段视图）。
- RecordView 载荷计费与 Record 同式（owner/name/字段表/字段名 String 堆），
  FIELDMOVE 路径 Vec 计入 heap_estimate；abc 解码卫哨逐元素镜像。

## FIELDMOVE 编码

- 路径解析到 owned 字段类型；dst 与字段等型；**前缀重叠 once**（祖先/后代
  双向——移出祖先后子路径死、移出后代后父路径非完整值）。
- 稀疏 moved_fields 数据流：合并保守并集、定义点重置（新完整 owner 清零）；
  同路径二次抽取/部分消费后整根读均 `mir.use-after-move`。

## mut record 参数写回（782 ABI-E `*mut`）

- callee 侧：MutBorrow record 参数为 move-in owned 槽；存在任一 mut record
  参数时，MIR 签名结果包装为 `t_MutRet_<fn>` record `{f0: 源结果,
  wb*: 按 callee mut-record 参数序}`，Return 经 AGGREGATE 装配。
- caller 侧：Call 得包装后，f0 可复制结果 FIELD_GET、owned 结果 FIELD_MOVE
  转出；wb* 按 **callee 自身 mut 参数位**驱动逐字段 FIELDMOVE+MOVE 回写——
  只读 record 实参与非 record 实参不消耗 wrapper 字段；包装必须被完全消费
  （f0+全部 wb）。永久例：单 mut 808、混只读 810、双 mut 8917、owned Str
  返回 801、mut 循环 9。

## 借用与状态

- RecordView/StrView 参数外部根；活 MutRef 排他（阻新 view 与冲突写），
  BorrowEnd 释放；语句尾临时 view 终结。
- owned Str 文本接收者先 VIEW 再入 STR_* 指令（view 操作数直通）。
- 多参 `.move` 调用标记关联错位为 782 源域已知缝（KNOWN-DEBT 归 782 owner）。

## 证据锚点

`experimental/ac-av-probe/probe/tests/av_owned_821_r1_replay.rs`（R3/R4
永久例与逐字段不变量）、`av_owned_821_incremental.rs`（stale-link 拒）、
corpus `av-owned-mut-writeback`（exit 808）。


## native 消费者覆盖（PLAN-870）

- mn 消费 av.owned/1 的 text/text-record 子集：本合同的指令语义在 native
  侧以"死槽=零化指针"纪律+全出口 sweep 复现（合同全文见
  [native-data-owned-lowering](native-data-owned-lowering.md)）；MutRet
  写回（§mut record 参数写回）经 FieldMove/Move 实机三路验证。
- 台账差分口径冻结为三级（alloc bytes 序/free 多重集+单次性/指令点释放
  精确；exit 内部序仅非 clone 行逐序）——clone 派生 record 的宿主 Rc
  drop 时点序不属冻结语义。
- 视图 `==`/`!=` 源级形态被共享 lowering 阻塞（mir.bin-type，
  `gap:upstream-s-eq`，S owner）；ACE-M1 T05 行据此具名排除。
- ACE-M1 T*/R* 的 ac 列已实测判级（ok/gap:mn-text-move/
  gap:mn-mut-param/gap:mn-lower-method——与 av 列判定逻辑逐行镜像，
  见 [ace-m1-conformance](../auto-test-case/ace-m1-conformance.md)）。
