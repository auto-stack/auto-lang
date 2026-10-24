# auto-av 常驻函数安装与事务快照（PLAN-828）

> Status: active（实现=`core/src/{function_code,program_snapshot,av}.rs` +
> `probe/src/session.rs`）。

## 单元派发（无 merged 载荷）

- `ProgramSnapshot::link_with_maps`：单一 owner 上下文门（schema+module 全一致）+
  实际调用 import 闭包门（每个被执行的 Call 必须有链接期绑定）+ `fn:<name>` 签名
  等值绑定表（scratch interned 类型表仅作内容等值校验）。**不复制未改 blocks/
  instrs/slots/spans**——执行视图是单元本身（`UnitView`/`ModuleView`）。
- 三公开入口（`execute_program`/`execute_program_steps`/`resume_program`）共用同一
  单元派发；嵌套/递归沿捕获 snapshot 选 callee；分段 resume 预算（段内增量比较，
  累计硬限额跨段）。
- 来源投影：payload span 经帧执行槽投影映射（payload func 号不参与键控）；强制
  复制未改执行 payload 且实际使用时结构门必红（副本负控）。

## 安装事务与寿命

- prepare 仅构造候选（PreparedEdit 自持候选 HIR/code/来源/绑定）；成功 commit 在
  同一发布点原子更新 root/source/revision/env 槽位；失败/取消/迟到/foreign 不覆盖
  当前可用版（session_id/base_revision 守卫；内部 FnCandidate 另核 env_id/base_generation）。
- 新顶层调用取新 snapshot；在途调用链（PausedRun/pin）固定旧 snapshot 至退出；
  旧代码在 snapshot/frame/lease 引用期间保活，最后 owner 释放即回收（Weak oracle）；
  close 幂等、释放 env/store 拥有引用，外 pin 存活、vacant 执行受控 Infra。
- 每函数内容历史有界（HISTORY_CAP=32，真实淘汰；驻留当前条目永不逐出）。

## PLAN-817：snapshot 车道消费（exec snapshot，追加）

817 消费面接入 828 session：deck `exec snapshot` 车道——warm-up 一次装载全部单元+
link ProgramSnapshot，逐调用实参经 wrapper 增量编辑（尾部锚点对齐），ProgramSnapshot
单元视图执行。车道为缓存身份轴（engine.exec 入 envelope key——closure/snapshot
永不共享条目）；增量纯度（未改函数零重编译/代际不变、warm 为唯一整模块事件）由
lane 退化计数器与 828 计数器双重观测（resident-edit rewarms≠0 即红）；冷启动首调用
即 bool callee 属正常支持面；新能力为追加车道，默认 closure 车道保持，不退回整模块
热重编/装载。消费契约详文见
[auto-cte/scalar-query-profile.md](../auto-cte/scalar-query-profile.md)。
