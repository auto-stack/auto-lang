# AAC 原生对象与 CT 执行服务

PLAN-1051 在 Windows x64 MSVC 上交付 AAC 的机器后端和 AV 编译时执行接缝。Auto 前端拥有源码语义、实例化、HIR→MIR、CT 编排与发布。服务消费已定型的 Typed MIR，以 ABC 编码传入，另带全局表和初始化序列。Rust 侧负责消费端重验、Cranelift 对象发射、AV 执行和显式运行库。

## 传输与身份

一次请求由 Windows Job 隔离的 worker 执行。`ace-aac-bridge --exchange <绝对请求路径> <绝对响应路径>` 是 Auto 客户端的入口，Rust SDK 也可通过 `Client` 调用。AQ01 请求、AR01 响应均由 4 字节 magic、两个小端 u32 长度、32 字节 BLAKE3、UTF-8 JSON 元数据和二进制载荷组成。摘要覆盖两个长度及精确的元数据和载荷字节。读入先验证长度再分配。每次只接受一帧。

请求带 16 字节 ID 和源码、grammar、pass、stdlib、依赖摘要，以及 build、host、target、执行域和 ABI。消费端校验支持的目标与类型，依据闭包 profile 重建逻辑 MIR 身份并复验类型、所有权、引用和声明能力。初始化函数只能由启动序列调用。`GlobalSet` 的结果槽为 i32。序列化的 Checked 标志没有授权含义。

父进程检查响应 ID、请求摘要、结果种类及载荷。对象按单元及启动对象返回，每个分段带长度、摘要和导入。文件交换以同目录暂存后原子发布，已有响应保留。Auto 客户端核对帧摘要和请求绑定，提供 i32 CT 值读取，并清理请求和响应文件。

| 维度 | 上限 |
|---|---|
| 整请求 / JSON 元数据 | 4 MiB / 64 KiB |
| 整响应 / 单 COFF / stderr | 64 MiB / 16 MiB / 64 KiB |
| 机器请求墙钟 | 调用方剩余预算，至多 5 s |
| CT 请求墙钟 | 调用方现有 CT root 剩余预算，至多 10 s |

超时及超限、取消、准入与语言错误、外部工具故障各有分类。进程树清理结束后才报告可恢复的取消或期限结果。外部链接和运行由调用方显式进行。

## 已验证的服务范围

| 服务 | 当前范围 |
|---|---|
| `native_objects` | 共享 Cranelift 产生 Windows COFF，支持标量全局及显式初始化。已有文本运行库保持独立。获批准的 967 外部调用走无全局状态的对象模块。 |
| `native_collections` | 共享运行时执行动态值、对象与带所有权的全局 cell；成功退出和 trap 均清理注册的全局 frame。 |
| `native_scheduler` | 共享调度器处理标量和 owned 文本，支持受检的 Windows 内存、文件和 TCP 组合。该 ABI 当前要求无程序全局及初始化序列。详见[原生调度](native-async-scheduler.md)。 |
| `compile_time` | AV 执行受检闭包，返回 typed 值、读集及资源情况。普通 CT 无环境 I/O；显式批准的纯 967 `view-text → I32` 可在隔离 worker 内执行。 |

外部 provider 由 SDK 启动器提供 `ACE_AAC_FOREIGN_CONFIG` 的绝对路径及 `ACE_AAC_FOREIGN_CONFIG_BLAKE3` 的十六进制摘要。两个值须成对配置。请求只能用摘要选择已安装 provider。worker 独立复验声明、域、库内容与可调用形状。SQLite 等有状态 provider 走真实 native 显式导入，CT 拒绝其状态帧。详见 [AutoABI](../auto-abi/project.md)。

这是 [AAC 阶段合同](aac-baseline-and-subject.md)的一项 Windows 交付。1044 的 41 条 required CL 与已有 Linux/其他 host deferred 行继续按原归属验收。1053 保完整 Gen1，1054 保代际认证，1068 保 M2 三端最终收口。
