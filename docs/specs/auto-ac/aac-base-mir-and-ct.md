# AAC 基础 Typed MIR 与 CT 调用

PLAN-1055 的 `aac.base-mir/1` 在 Windows x64 上消费 Auto 签发的闭合单态、纯计算 Checked HIR。源码经过 AAC grammar、词法绑定、类型检查及 HIR 复验。短路右侧只在对应 CFG 路径求值。条件循环绑定最近的 `break` 和 `continue`。直接调用先按源码顺序求值，再按声明的参数槽映射。

Auto lowerer 按函数编号分配签名、槽和块。发射前检查定义及调用归属、参数和返回类型、CFG、终结指令与可达路径的确定初始化。AVAD v1 执行体和源图交独立消费端复验。整数常量保精确位宽，浮点字面量按目标位宽舍入。基础闭包使用空导入、空全局表、空初始化序列与空能力集。

## 编译时调用与机器消费

CT 工厂将具名目标和字面实参组装为零参数直接调用，经同一 Auto 源码、HIR 和 MIR 关口。AQ01 请求携带扩展后源码、grammar、pass、SDK 和依赖的 BLAKE3 摘要，三条平台轴为 `x86_64-pc-windows-msvc`，执行域为 `hosted-av`，ABI 为 `av-value/1`。调用沿用现有 CT root 的剩余期限，根上限为 10 秒。Auto 客户端核 AR01 帧与请求绑定，区分匹配预期标量载体的值与失败，并回收文件。服务侧进行独立 Typed MIR 准入和 AV 执行。

`native_objects` 消费同一 AVAD 产物，发射 COFF 对象及启动对象。调用方通过现有链接器生成 Windows PE，零参数启动签名服从 1051 的 i32 家族 ABI。详见 [AAC 机器服务](aac-service-boundary.md)。

## 后续交接

- PLAN-1048 接 `aac/host/cte-entry` 的字面调用工厂和基础 LowerCt。待展开的元计算继续以 `aac.pending-work/1` 报告。
- PLAN-1050 扩同一 `aac/mir` 实现并保留最终兼容复验，接实例化后输入和 owned、closure、dynamic、async、领域及高级 CT。PLAN-1053 接完整 CL。
- PLAN-1068 接该切片的实际消费结果。切片以所用能力的正式来源准入。CL 继续保 41 条 required 和 1 条具名后继，三端最终义务沿 [AAC 阶段合同](aac-baseline-and-subject.md)。
