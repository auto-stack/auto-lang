# auto-cte 标量查询 profile

> Status: active (plan-817)；实现 `experimental/ac-cte-probe/{contract,core,probe}`。

## 值级与查询节点

- 值级仅 i32/bool（参数/返回/pure 直接函数调用/有限控制流；精确算术与 trap 按
  D-798 已支持面）；普通函数递归由 AV fuel 管，不等同查询依赖环。
- 查询节点只有：字面量、已完成 query 引用（合法 DAG 依赖，非环）、`Call(decl,args)`
  、显式能力下的 `FieldCount(record)`/`HasField(record,name)`——后两者经 796 当前
  签名/字段投影读取（meta_query 计数，不冒称 AV 执行）；缺 record=解析/语义错误，
  现存 record 缺字段=false+集合负依赖。
- Meta 能力默认不授予，需具名 `cte.meta.fields/v1`；不允许文件/环境 broker 或任意
  宿主函数扩面。
- **Meta→AV 实参（合同路径）**：deck `meta << ... >>` 块携带 record 声明
  （record-profile 源文本），投影合并进 core 输入 record 域——FieldCount/HasField
  结果作为**真实 AV 调用实参**；字段集/缺席字段变化经 manifest meta 事实精确失效。
  纯 record-profile deck（无 meta 块）为 Meta-only 受限模式。
  meta 输入负控：未授 cap → `cte.capability.meta`（AV 前）；缺 record →
  `cte.meta.record-not-found`；meta 解析错 → 语言类诊断；含 meta deck 同受 4MiB
  深输入界。

## 请求、调度与失败

- 请求 key 含签名/实际 callee identity、常量参数、算法/profile、引擎与
  MIR/ABC/冻结 codec 版本、权限/目标语义；同名异包与方法 owner 区分。
- 显式查询栈/worklist + InProgress/Completed/Failed；依赖环（A→B→A）在调用链定位，
  不靠"全程序先 Checked"绕过；查询取消/超限/旧 snapshot 迟到不发布当前结果；
  纯查询可 single-flight，失败/取消不缓存成永久语言错。
- compiletime 已执行溢出/trap → 带调用链的 `cte.*` 语义诊断（trap 诊断携带 callee+
  实参；未执行分支 trap 保持潜伏）；fuel/内存/取消/宿主崩溃 → 资源/基础设施分态
  （`cte.resource.deadline` 等为受控资源态，非语言裁决）；ct_execute 对到达 AV 的
  trap/resource/cancelled 同样计数（执行事实先于 consumer 裁决）。
- 单请求累计墙钟 ≤10s（含调度/缓存/编译执行/consumer/必要等待），逐调用
  ≤D-798 2s cap。

## consumer 双向绑定与回灌

- consumer 推导面严格限 796 已交付 core i32/bool 操作；输入为有限语义模板+与 HIR
  常量位置的 owner/type 绑定；CT 结果投影成 796 InputExpr 字面量才做局部推导。
- HIR 模板对应位置回填 `Constant` 后**重新 verify**（owner-only 源编译集=
  OWNER ∪ ENTRY ∪ entry 传递调用闭包，owner≠entry 不丢入口链；verify 失败回退
  全源——非缩小验证范围）；错误绑定/重复绑定/类型错/跨 owner 全拒绝。
- 首例 sum(20,22)=42：真实 AV 返回 I32(42) → consumer 推导 i32 → HIR 常量 42 →
  native 可观察（exit 42，无源码重 parse）；bool 例分支翻转 2→1。不能只输出
  JSON 即称回灌完成。

## 828 snapshot 车道消费（exec snapshot）

- warm-up 一次装载全部单元+link ProgramSnapshot；逐调用实参=wrapper 增量编辑
  （尾部锚点对齐保证收敛）；ProgramSnapshot 单元视图执行。
- 车道为缓存身份轴（engine.exec 入 envelope key——closure/snapshot 永不共享条目）；
  增量纯度=未改函数零重编译/代际不变、warm 为唯一整模块事件，由 lane 退化计数器
  与 828 计数器双重观测（resident-edit rewarms≠0 即红）；冷启动首调用即 bool callee
  属正常支持面；新能力为追加车道，默认 closure 车道保持，不退回整模块热重编/装载。

## 不声称

全语言 comptime/类型生成/泛型/runtime 反射；consumer 面不因 callee 支持更多 scalar
操作而扩 796 求解器。
