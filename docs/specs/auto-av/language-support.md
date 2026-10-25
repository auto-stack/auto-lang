# AV 语言支持分层（language support）

> PLAN-827 r3 交付（R10 冻结文本）。状态：experimental。

支持证据分三个独立层级：真实源码编译/执行、typed HIR/MIR fixture、产品入口
支持。fixture 或可运行的 legacy/native 程序不构成 AV 源码支持。

AV 源码消费者使用既有 `auto.source.scalar-i32.draft` 前端与 `av.core-i32/1`
执行器。它原样读取 AP-01 `p-gcd-euclid.at`、AP-22 `p-binop-args.at`、AP-23
`p-short-circuit-trap.at`；其字节身份与独立 oracle 冻结于 PLAN-827。
AP-01/AP-22 返回 typed i32 零；AP-23 第一条 false 短路分支安全，其后必经
除法以 `av.trap.div-by-zero` 精确 trap。native exit 70 是独立参考通道。

G0 库存枚举 AP-01..23、C-01..43、EP-01..10、gate 行、实际 VM
case/source/oracle 身份、stdlib provider/file 成员与入口符号。缺失、不可读
或不支持的成员保留 gap/owner 行。库存是分母，不是所有成员可在 AV 上执行的
声明。SEM-01..08 保留 profile 区分与未来 owner；旧 expected 文件与 AP-18
见证不自动成为新语言规范。List/text/record/enum/closure、
modules/stdlib/host IO、async/UI/FFI 与产品 CLI 替换需要各自交付合同。
GOAL-002 在此有限贡献之外保持开放。
