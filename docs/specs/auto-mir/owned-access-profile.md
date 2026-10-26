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
