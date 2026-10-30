# Typed MIR 标量合同

来源：PLAN-798；实现 `experimental/ace/crates/av-core/src/{contract,mir,mir_verify}.rs`。

## 支持与语义

`auto.mir.av-probe/1`，MIR语义版本1。i32与bool严格区分；有类型槽、基本块、常量/copy/二元/一元/直接call、Jump/Branch/Return。源码命名实参保持源码求值顺序，再按绑定落位；短路的未走分支不执行，不提前触发trap。i32算术溢出、除零、MIN/-1除法/余数等按当前标量profile返回结构化trap，不借宿主溢出/UB改变语义。

完整模块有零参i32入口；函数unit是可重验、待绑定单元，不能独立执行。完整模块允许合法自调用；不可按函数数猜unit/module。imports、签名、entry和kind的语义一起验证，未知能力/悬空或越owner引用不准入。类型/结构检查覆盖不可达块，初始化按可达CFG保守合流，合法循环不一概拒绝。

## 信任与资源

仅verifier构造私有CheckedMir；修改内容/上下文后必须重新授信，持久化不保存trusted。shape/checked长度及分配预算先于深验证，资源超限与非法格式/类型分态，不靠宿主panic/栈溢出处理输入。

有限探针策略：单source/HIR/ABC输入4MiB；模块types+consts+imports+functions合计≤65536，每函数slots+blocks+instructions合计≤65536。验证≤200万work steps/5s；分析暂存与单解码模块结构估计分别受16MiB限制，分配前核准。旧source/HIR前门最长真实构造路径128层，拒绝发生在递归parser前；这不是语言永久语法上限。

## 实际增量边界

当前adapter接受已检查模块，仍遍历/降低整模块。函数MIR独立持久化、命中恢复后重验已经接线，但不等于未改函数HIR/MIR对象驻留复用。真正lower_function和上下文凭证由828交付，未来新增类型/Place等必须保持其已交付机制。
