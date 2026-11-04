# auto-a2x Rust owned 临时值消费者生命周期

> 状态：active（AX-B record/text Rust 发射；PLAN-931 关闭 D-925-2）。承接
> [checked-resource-facts](checked-resource-facts.md) 单源计划与
> [architecture](architecture.md) ADR-3（源序求值/落位）。

## 合同（谁在何时释放）

- **view 消费的 owned 临时活到真正消费者**：pin Let 或语句根（let/assign/
  return/if 条件锚）读取它之后**立即**释放（`drain_since(mark)` 作用域，
  mark 前注册的临时属于尚未运行的外层消费者，不得提前释放）。多实参调用
  的后继实参 pin、语句根的先导释放均为违例（D-925-2 原缺陷：先 free 后
  读——Rust `A2xStr.buf.take()` 后 `as_str()` 读空串，且临时 pin 非 mut
  与 `a2x_str_free(&mut)` 不容直接编译红）。
- **嵌套消费者只清自己的责任域**：`lower_view_value`/`lower_pure`/
  `lower_move_value` 入口取 mark，pin 后释放 mark 之后注册的操作数临时；
  外层实参跨内层消费者存活。待释放临时 pin 渲染 `mut`。
- **短路隔离**：`Logical` RHS 的 pin 与其临时释放都在侧块内、以入口 mark
  作用域化——未执行侧块分配/释放均为 0，外层 pending 不被侧块偷走；侧值
  为块尾表达式时若仍持有临时则先物化为侧内 temp 再释放。
- **return/if 锚**：return 值引用待释放临时时先物化（沿用计划 buffer_frees
  的 pin 模式）再释放再 return（return 后不可达）；if 条件同理先物化条件
  再释放（取最早清理点，两分支前）。
- **所有权转移**：`move Str` 实参按值传入（Rust move 即转移），callee 在
  其排出口清理恰一次；return 的 owned 值转 caller 责任；已移出源不再释放。
  片段 external 调用消费受检 `ExternalFn.modes`（move/mut/view 原样），
  禁止默认 View 或后端自行推导。
- **失败语义沿冻结合同**：fault 的分配尝试不发事件不算分配；失败路径零
  free、立即 trap 传播、无部分提交（`check_trap` 同形）；不新增 catch。

## 观测 oracle（不是新产品配额）

- 事件账本（`A2X_RT_EVENTS=1` 的 `rt:alloc:/rt:free:`）核对"分配→最后
  消费→释放"顺序与计数守恒；live-byte 峰值由账本独立行走计算，界=单
  消费者操作数足迹（延到函数尾释放的反控形状峰值翻倍）。
- fault 注入（`A2X_RT_FAULT_ALLOC=<k>`）证明失败原子性；C 侧同形修复
  仅作差分参照，不是唯一 oracle。
