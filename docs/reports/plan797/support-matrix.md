# PLAN-797 有限 profile 支持/拒绝矩阵（support-matrix）

> 状态：**v0.1-candidate（R4 修复后重置）**。绑定
> `autoabi.win-x64.v0.1-candidate`。拒绝面是有限 profile 支持表，
> 不是永久删去战略。R4 指出的支持表缺口已按实现闭合（ctx 首参、
> UTF-8 校验入口、bool/char 值校验入口、描述符 param 面积）；每一行
> 均有正反原生 oracle（见 t02/t03/t04 与 §0）。

## 0. 正式通路面（T-06 新增）

| 面 | 值 |
|---|---|
| AutoDown recipe | `abi/interop-v1`（REGISTERED_PROBE_RECIPES 追加；绑定互斥解码：sema/abi 绑定挂错配方=admission error） |
| deck 源类别 | `.atomdata` DATA（非可执行源；case 永不携带 shell/exe） |
| worker 协议 | `abiworker/1`（信封同 acworker/semaworker：nonce/request_hash 回显、deny_unknown_fields、4MiB 界、Job containment、退出状态/版本镜像、exe-sha256 入 stale 轴） |
| 可执行语料 | `crates/auto-lang/test/cases/autoabi/`：leg-a（publish→hit→跨进程 restart hit→body-edit republish）、leg-b、leg-c、bad-sig（类型化 deck 诊断负例） |
| 语义身份 | `ac-sema-contract`（D-796 合同）只读消费；临时快照已退役（收据在案） |
| 正式复测（R4 统一冷采样） | worker 传输 body-edit P95 699.8ms / reject 87.3ms；父路径 run_suite 端到端 body-edit P95 820.7ms / reject 479.1ms（各 30 样本；mix-alt<seed> 每样本冷生产，冻结 C 消费者恒过；原始 JSONL 在 target/abi-formal-gates） |

## 1. 类型与物理形态

| 逻辑面 | v0.1-candidate | 物理形态 | 备注 |
|---|---|---|---|
| unit / 无返回值 | 支持 | 无 ret 或 status-only | fallible 用 status |
| i32 / u32 | 支持 | 4B | |
| i64 / u64 | 支持 | 8B | 跨栈槽 |
| f64 | 支持 | 8B XMM | 用于 GPR/XMM 混合对拍 |
| bool | 支持 | u8 仅 0/1 | 非 0/1 → -8（`p797_crec_valid` + t04 反例 7→拒） |
| char | 支持 | u32 合法 Unicode 标量 | surrogate / >U+10FFFF → -8（t04 反例 0xD800/0x110000→拒） |
| 借用 byte buffer | 支持 | `{i64 ptr, u64 len}` | len=0 允许 null |
| 借用 UTF-8 text buffer | 支持 | 同上 + UTF-8 标记 | 非法 UTF-8（overlong/surrogate/>U+10FFFF/断尾）→ -7，out 不写（`p797_cbuf_sum_utf8` + t04 双向控制） |
| record const view | 支持 | 显式 pointer | 声明序无环 |
| record mut view | 支持 | 显式 pointer | 排他规则见合同 |
| caller-owned out record | 支持 | 显式 out pointer | 成功才提交 |
| opaque owned handle | 支持 | 代际身份 + 原方 destroy | |
| 同步 per-call callback | 支持 | 函数指针 + 显式 ctx | 不保存 |
| 按值聚合传参/返回 | **拒绝** | — | 走 pointer/out 协议 |
| Rust String/Vec/enum 按值 | **拒绝** | — | 显式缓冲/句柄 |
| variadic | **拒绝** | — | |
| bitfield | **拒绝** | — | |
| 任意 union | **拒绝** | — | |
| C++ ABI / 异常穿界 | **拒绝** | — | 无 unwind |
| 持久/跨线程 callback | **拒绝** | — | 不是 loader 承诺 |
| 热重载 / DLL 事务 | **拒绝** | — | 能力位预留，未实现 |
| 跨域自动转发 | **拒绝** | — | 未知版本调用前拒绝 |
| Auto 源码 parser 全 ABI 类型 | **非目标** | — | emit 源=声明 IR |

## 2. 平台 / 工具

| 项 | 值 |
|---|---|
| target | `x86_64-pc-windows-msvc` |
| 调用约定 | Windows x64 |
| profile | `autoabi.win-x64.v0.1-candidate` |
| C 编译 | MSVC cl.exe（可定位；缺 = infra 阻断） |
| 链接 | rust-lld（ac-core `find_rust_lld`）+ SDK um/x64 |
| 其他平台 | **未声明**（不是“支持”，也不是“永久拒绝”） |

## 3. 调用前拒绝（native 计数必须为 0）

| 维度 | 动作 |
|---|---|
| 未知 descriptor format | 拒绝 |
| 错 domain / version | 拒绝 |
| 错 target / profile / packing | 拒绝 |
| semref 不匹配 | 拒绝 |
| 缺符号 / 签名不符 | 拒绝 |
| 布局不符 | 拒绝 |
| 缺 runtime 服务能力 | 拒绝 |
| 递归 record | 拒绝 |
| 嵌套超深（>4096） | 拒绝 |
| 超尺寸 record（>64MiB 预算） | resource 拒绝（与 ABI 不兼容分列） |
| 好 frame 错地址 / 坏图 | 受控错误 / 重算，不 panic |

## 4. 三腿与负控（oracle 必须可区分）

| 腿 | 形态 | 必过 |
|---|---|---|
| A | AutoABI → AutoABI | 独立双 obj + 独立 PE |
| B | AutoABI → C | cl.exe 独立 C callee |
| C | C → AutoABI | C caller 自带 oracle；可嵌套 |

负控：swap 参数、错 record offset、漏调用、tampered descriptor、
uninjected-against-failure-expectations → 必须 exit 恰 1 或调用前拒绝。

## 5. 与 774 内部 ABI 的对照

| | 774 ABI-A–E | autoabi v0.1-candidate |
|---|---|---|
| 域 | 内部 probe | 公共候选 |
| 对象 | 770/771/782 | 独立 COFF/conformance |
| 兼容关系 | 与 autoabi **无** | 与 774 **无** |
| 本计划是否改写 | **否** | 新增 |

## 6. 状态声明

- 当前：**candidate**，历史证据来自 `experimental/auto-abi-probe` 测试
  与 `contract-decision.md`；正式 AutoDown 复测归 T-06。
- 冻结：仅在 T-08 独立 review 通过 + T-09 merge 门禁后，登记
  “本 target 有限 profile 冻结”。不宣布所有平台/未来类型 ABI 稳定。
