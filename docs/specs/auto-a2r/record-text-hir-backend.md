# auto-a2r record/text HIR 后端（AX-B）

> 状态：active（PLAN-902 交付）。承 [auto-a2r/project.md](project.md) 与
> [auto-a2x](../auto-a2x/architecture.md)；recipe `ac-a2r/hir-record-text-emit-v1`
> 与标量通道 `ac-a2r/hir-scalar-emit-v1` 并列、严格互拒。

## 支持面（record/text/text-record 三显式源域）

- **数据**：i32/bool/char、str/Str、具名非递归 record（纯值嵌套与含 Str
  字段资源 record）；同布局异 owner 不混同（名义身份）。
- **构造/字段**：位置/命名构造按源序求值后落位；字段路径读写；纯 Copy
  按求值点快照（后续 mut 不追溯改写）；资源字段读取为 view（不隐式
  clone/移动）。
- **方法**：static/view（receiver=params[0]、先于实参恰一次求值、
  BorrowView/BorrowMut 调用点标记）；方法身份 owner 限定
  （`mt_<Owner>_<name>`），同名不同 owner 并存；**mut 实例方法/ext/
  trait/任意重载不扩**。
- **参数/返回**：view/mut/move 显式模式——view 文本=内容视图（`&'a str`，
  mut 文本排他性由验证器调用点证明，文本无运行时变异面故共享视图观测
  完备）；view record=`&'a Rec`、mut record=`&'a mut Rec`（回写内禀）、
  move=按值（被调方按计划清理）；重赋值/move-out 被验证器拒。
- **文本**：literal/Unicode/转义/NUL；len=标量数、len_bytes=UTF-8 字节、
  eq/ne=完整字节无归一化；char_at/slice 标量索引、越界/负值=typed
  `TextIndex`；码点往返、非法标量=`InvalidScalar`；to(Str/str)/clone/concat。
- **资源**：显式 clone/move、被调方拥有责任、词法出口/短路/循环/分支、
  Str 局部替换（RHS 物化后 free 旧再装新）、资源 record 字段级替换与
  深克隆（每非空 Str 字段一次分配，嵌套传递）、拥有返回不 free、
  move-in 根由被调方清理；无 NLL/用户析构/GC。
- **拒绝面**（具名）：同根 mut+view/双 mut（`verify.mut-conflict` 源级）、
  borrow-escape、部分 move、宽源码/891 私有 ABI、Unit 入口、闭包/HOF、
  用户泛型/模块、console/host/FFI。

## 目标运行时与观测

- 拥有 Str=运行时 `A2xStr`；**显式清理**消费验证器计划原样（无 Drop 依赖；
  漏 free=可观测泄漏非静默）；分配前 checked 算术+try_reserve，宿主失败=
  typed `Resource` trap。
- 文件集 schema `auto.a2x.rust.record-text.fileset/1`（main.rs/
  a2x_runtime.rs/generated.rs+manifest，target 含 `resource_runtime:
  "a2x-rt/1"`）；证书 `ace.a2r-hir.record-text.cert/1`；观测协议同冻结
  `a2xworker/1`（stdout `value:<i32>`/`trap:<Name>`、stderr marks）。
- 测试期默认关闭钩子：`A2X_RT_FAULT_ALLOC`（第 n 次分配失败→Resource、
  无部分提交）、`A2X_RT_EVENTS`（rt:alloc/free 字节事件序）。

## 边界（AX-C 与前端遗留）

- List/Variant、闭包/HOF、mut 实例方法、用户泛型不在本域。
- record 族源（含 `type` 声明）的 warm **片段**编辑暂不可用：828 graft 的
  type 声明 span 重锚为前端工作项；其编辑走整路径（居民/store 复用仍是
  函数级增量）。
- 源形态约束（前端语法非后端限制）：`==` 两操作数须同名义类型；条件以
  标识符紧邻 `{` 结尾解析为命名构造（比较经辅助函数/调用形式书写）；
  `tag` 为保留字。
