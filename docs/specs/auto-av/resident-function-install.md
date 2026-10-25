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
## PLAN-818：mn 消费者的增量继承（SD-06）

- 818 消费 828 函数 IR/代码复用模型：未改函数不重新 lower/codegen（驻留工作
  集+store 持久层）、内容命中=历史单元复用（cv→fp 映射、代际不跳）、每编辑
  恰一单元重发——**新增能力不退回整模块热重编/装载**（退化仅记录性：注入缝/
  异常触发的全模块回退=结构门非零）。
- 用户明确的顺序与增量机制保留：编辑序即提交序（前缀 exactly-once 保留，
  语法回退从 session 当前源重建）；代际/身份变更只落在被编辑单元。
- mn 消费门对每个单元重验证（verify_mir）后安装；emission 失败永不提交
  revision——受检安装/依赖/寿命语义与上节安装事务一致。

## data profile 接入（PLAN-820）

data profile 接入 828 公共函数级增量接口：`lower_function_data_unit`（record
域单函数 AVAD unit）→ `FunctionCode::admit_unit` → `ProgramSnapshot::link`
（fn: 导入解析+签名检查+owner 一致性）→ unit view 执行。**全链增量**：

- 纯 body 编辑走 828 单体事务：字节级分段 diff 命中唯一被改函数（否则显式
  non-fast 拒绝）、等长注释掩码的单函数解析文档（解析面=1）、保留 CheckedEnv
  的 recheck（resolve+typecheck 仅单 body，签名漂移受控拒）+
  emit_function_body（收据=1 真 body/n−1 scaffold）+ body_work 生产窗（未改
  函数零 body 工作）；未改函数 HIR Arc 与 FunctionCode 指针/代际不变、生产
  阶段/IO=0；结构门内建于 edit_body 生产路径（实现退化即使编辑本身红）。真实
  生产事件在调用点计量（verify_mir/decode/verify_abc/admit 逐 def），HIR 身份
  绑定生产 bundle Arc（非消费者克隆）；失败/陷阱编辑零安装（build-aside +
  运行先于发布）。
- 类型/签名/声明集变化走依赖事实计算的精确失效：record mention 嵌套闭包+
  跨 bundle 签名形状等价——签名内容变化→全部 unit 重 lower（unit import 表
  嵌入每个 def 的签名），body 未变函数保留 HIR Arc；helper body 变化不重
  lower 调用者（快照按槽位绑定调用）。
- caps 变化在同一成功发布点同步三面：全 unit 重 lower/admit（HIR 身份
  不动）+ 驻留前端环境按新授权重建（grant 后使用新能力的热编辑可解析、
  被用撤销受控拒且旧值全保、成功撤销后无权热编辑被拒）。
- 跨修订签名失配=link 拒且前快照存活；强制 whole 前端与"只保 hash 换对象"
  双退化反控均使同一结构门非零。

V-13：incremental 族 9/9 + 828 标量套件绿=共享基线。


## PLAN-827 源码编辑消费 D828 会话

PLAN-827 的源码编辑消费已交付 D828 会话。签名/依赖/profile/配置/授权不变的
纯 body 编辑保持函数 HIR/MIR/代码对象与代际不变；未改函数生产阶段与工件 IO
保持零。稳定源基线 273575B/115 函数与至少 30 个不同修订保持独立值、精确
安装目标、结构身份观测与 P95<1s。仅时间绿不能开脱整前端/重装载退化；等
内容强制换对象必须使身份检测器红。源签名或未授权编辑是保留当前修订的记录
性拒绝；helper body 变更不移动仅接口依赖的调用者身份。

暂停链在 resume 期间 pin 原快照并拥有槽位缓冲；完成/取消/drop 释放这些
引用。prepare 永不发布，cancel/drop 保留当前状态，close 释放拥有的条目。
新帧槽位 reserve 失败使用同一生产 dispatch 缝并保留这些所有权规则。持久化
raw 观测必须包含实际基线与当前身份、可归因于受审输入/运行时，并在失败
verdict 下留存。cold/整源/重启路径分开报告；有限源语法拒绝不退回整编译。
