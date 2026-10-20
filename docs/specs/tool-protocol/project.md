# tool-protocol — I-TOOL 轻工具协议(M815-A 阶段子集)

> 状态:PLAN-815 M815-A 已发布子集(2026-10-20,R4 阶段审查通过,delivery `e609caf1a`)。
> 本文件陈述协议版本 1 的 A 面;progress 事件实际发射、交互 stdin、运行进程寿命等完整 B 面合同未发布。

## 当前协议与预算(SD-02-A,AC-03 当期/AC-07)

- 协议版本 1;JSON 消息以 4 字节大端 u32 载荷长度分帧。stdout 为控制帧,
  stderr 为日志;`--version`/`--help` 为协议外文本。
- 请求 `ClientEnvelope{protocol_version, id, body}`,复用唯一 I-PROJECT 数据
  owner codec(auto-project-model `wire` feature),协议不复制请求结构。未知版本
  拒绝且如实携带客户端 u64 值;未知操作拒绝;具备正确帧但非法 payload 时结构化
  拒绝并继续服务。
- 单帧载荷最多 8MiB,**不含长度前缀**;输入坏帧/超限/不可解析 JSON/帧深度超限
  为流致命 exit 2。响应序列化/输出帧预算或控制流写入失败为内部退出 exit 1;
  正常 EOF 为 exit 0。不能将这些路径一概描述为 budget_exceeded 响应。
- JSON 深度按根为 1,进入对象成员/数组元素加 1,标量或空容器返回当前层计数。
  **wire 请求文档限 120,真实信封增加 2 层,因此完整帧限 122**。writer、
  read_frame、parse_frame 对完整帧同限;可解析但超 122 时具名 envelope 拒绝;
  JSON 解析器自己的递归限(128)是最后防线,拒绝亦属 Malformed。完整信封下的
  边界需用真实请求检查。
- 为后续事件/执行器保留 1MiB/op 进度载荷、128 条未消费通知预算和
  BudgetExceeded 错误类型。A 不实际发射 progress、不提供交互 stdin 或运行进程
  寿命;这些预留数据不构成 B 能力已经实现的声明。

## 错误分类

unknown_protocol_version(client u64 如实)/ unknown_operation / missing_field /
invalid_field / budget_exceeded / not_cancellable / internal——全部可序列化、
具名;版本/能力拒绝先于副作用。

## 消费合同

816 执行器依赖 `auto-tool-protocol` + `auto-project-model`(wire),经子进程协议
调用 `auto-ui-tool` 二进制;不得静态链接 auto-ui-tools 实现进 CLI。
