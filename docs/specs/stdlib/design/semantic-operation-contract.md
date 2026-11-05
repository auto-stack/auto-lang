# 标准库操作语义与兼容实册

> Current-state · PLAN-943。实现位于 `stdlib/contracts/` 与
> `crates/auto-lang-core/src/stdlib_assembly/contracts.rs`。

标准库操作身份与 native 数字 ID 分开。`operations.toml` schema 1 保存稳定 ID、
语义版本、模块/符号/来源层、公共拼写与可见性、参数/返回/mode、单位与派发、
所有权/错误/显式需求、profile、独立 oracle、来源和后继 owner。
`compatibility.toml` schema 1 为每个操作保存一份兼容政策（含逻辑参数/mode/返回与
必须保留的原生绑定证据）、差额和四个 ACE
producer（AV/native/HIR→Rust/HIR→C）的支持分类。不得由名称或文件存在推定支持。

## 消费与拒绝

实际 `loader::scan_inventory` 读取所选 stdlib 根旁的 `contracts/`；不从其他 SDK
借用缺失的合同。整对缺席的独立 fixture 是未签约输入；只缺一份、读取失败、
schema 不识别、解码失败、重复/缺失 ID、政策孤儿、逻辑签名/mode、单位/派发或
可见性冲突均产生具名 `STDASSEMBLY.CONTRACT_INVALID`。
`stdlib_assembly_fingerprint` 对非法合同返回错误，不生成可用身份。
单份输入预算 256 KiB、每份至多 1024 项，未知字段拒绝。

已解析的实际 Auto 声明提供签名/可见性核对；失败层保留失败状态，不凭合同补造
AST。最终生产 `NativeInterface::production()` 提供实际名称/ID/绑定和独立
producer 签名。字节长度适配只接受 String/&str→str、i32→int、()→void 的具体
表示对应；不会把 i64/u64/usize 统一为 int。没有独立签名的原生仍为语义未验证。

政策要求原生绑定时，删除该字段属于具名错误，不能因为来源层解析失败而降级为
可用身份。合法同callee别名可用。`operation_proofs` 将实际source签名核验、
producer签名核验与仅政策一致但source签名未验证分开；未解析候选的逻辑签名
仍必须与政策一致，但这不构成实现签名或语义已验证的证明。

Inventory schema 3 保留全部层（包括解析/读取失败）、函数/类型内方法/ext、
enum 方法、spec 方法、类型、字段及 alias。声明行按源身份关联 native 名；
native 别名逐名保留最终绑定记录；provider 模块目录仍是声明，不升格为执行证据。
原生名称来自最终interface的完整注册表、catalogue和registry集合；不得用反汇编
提示专用的单名ID反向表计分母。每个实际别名保留同一ID对应的绑定/producer事实。
`contract_declared`、`missing_contract`、`parse_failed_api_unknown` 分开。
失败层的 API 数量未知，既不记成零 API，也不从文件分母删除。

合同内容的编译期身份与选中根的实际内容身份进入装配 fingerprint；两份合同也
列入各目标 manifest provider inputs。共同装配指纹与消费者收据指纹保持原分工。
此处没有新增 ACE 装配器、CAP 求解器或平行类型系统。

## 首批身份与差额

版本 1 冻结 17 个操作/政策对；完整字段以机器实册为准。

| 族 | 当前合同 | 后继 owner / 关闭条件 |
|---|---|---|
| text.byte_length | UTF-8 字节，str.rs.at::str_byte_len(str)→int；native 1526；公共 str.at 缺该声明且 isolated parse 失败 | 944/945：受检公共 source 绑定、宽度/溢出核验 |
| math.abs/min/max | 公开参数/返回 i64，不准窄化成 scalar-i32 | 945/948：真正源码宽域或保持具名缺能力 |
| math.private_square/cube | int Auto body，private | 945：先解决合法可见纯算法入口，不能绕可见性 |
| text.equal | 语言操作，byte-exact；私有 ac_rt_text_eq 是物理候选 | 945：真实受检公共绑定 |
| text.clone | Clone spec 不等于 str 已实现；独立 owner、失败保输入 | 945：拥有文本适配与分配/析构证据 |
| list.length/result.is_ok | generic library 声明与 ACE 有限 List/Variant profile 分账 | 946：有限域与共享资源事实，不推断任意泛型 |
| console.say | io.say(str) 不等于已接 AV console/1 | 947：精确签名、逐运行授权、原预算、输出提交 |
| io.read_line/read_line_opt | 原 str 返回与 Option<str> 并存；后者空行 Some("")、EOF None | 947：仅资源前探，保持旧行为 |
| 旧 len/length/parse_int 四入口 | 逐派发差异保存，下表 | 948：接受差异，排除无差异认证 |

上述四个 ACE producer 尚无公共 stdlib 装配接线；`unwired`、
`missing_capability`、`unknown` 均不是 `supported`。runtime/staticlib 存在、
typed HIR 宽数值成功或 Rust/C 编译通过，都不证明公共 Auto source 支持。
Task/Msg 的 VM park/resume、Rust Tokio actor/drain 和资源 owner/取消协议也未
证明等价；File/Task/Stream 的可行性归 947，领域与发行差额归 948。

## 保持的兼容差异

| 调用路径 | 现行行为 |
|---|---|
| 静态 Str.len(s) / s.len() | nat#1500，UTF-8 bytes |
| 泛型 receiver 经 CALL_SPEC 动态 len | 最终 registry 的字符数路径；不等于静态调用 |
| s.length | nat#170/内联字符数；legacy Rust bytes、JS UTF-16 code units 有差异 |
| Str.byte_len(s) / s.byte_len() | nat#1526，UTF-8 bytes |
| VM to_int/parse_int | 同1516，裸 i32、失败零；legacy a2r 的 Result 差异保留 |

独立区分输入：`A中😀` 为 8 bytes/3 scalars，`e`+U+0301 为 3 bytes/2 scalars；
NUL 不截断。永久守卫接入固定 `cargo t stdlib`（包括 872 的旧绑定/行为守卫）。
合法 i32 source 对照与 i64 source 的具名拒绝仅证明原有限域边界，不代表 math
公共 API 已接入。原 required/accepted-diff/NA、样本、资源预算与认证平台不变。
