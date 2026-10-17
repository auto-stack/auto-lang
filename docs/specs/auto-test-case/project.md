# auto-test-case

> **Status**: active
> 路径：`crates/auto-test-case` | 技术栈：Rust（仅依赖 vendored `autodown-test-parser`；auto-atom/auto-val 仅 dev）

TestCase 协议核心（`autolang.case/1` / `autolang.suite/1` / `autolang.test-report/1`）：
把"一份 source 工程登记多个独立后端检查"做成严格合同——发现、校验、选择、比较、
报告都在本 crate，后端执行 adapter 留在 auto-lang（依赖方向只有 auto-lang → 本 crate）。

## 目标与范围

- 严格合同模型：未知字段/坏类型/非法组合=定位错误，绝不解析后忽略。
- 原始 bytes 保真：载荷与 oracle 按字节比较（显式 `lf` 仅 CRLF→LF）。
- 身份绑定：suite/case 文件/源内容/oracle 内容（含 sidecar 字节）/合同文本 +
  toolchain（vendored parser 指纹、crate 版本、生效限额）逐轴入报告；旧报告经
  `stale` 语义核查，任何轴漂移即 STALE。
- 只读纪律：输出与输入/oracle 根不相交（canonical 双向比较 + 祖先 reparse 审计）。
- 非目标边界随 B 阶段（PLAN-788）收缩：native（ac core/scalar、Windows
  x86_64-pc-windows-msvc）、contained 进程隔离/deadline/输出 cap、783 统一
  cache、syntax 观测已按 v2 交付（[profile-v2-native.md](profile-v2-native.md)，
  worker 在 `experimental/ac-case-worker`）；仍未导入：text/record/text-record
  profile、XFAIL、语言内非 ASCII 输出（随 text 批）。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| document / raw_ranges | pinned 块树 ↔ 原始 fence 字节区间双向校验 | active |
| atom / contract | 静态 Atom 合同解析 + 严格 schema（含 per-adapter 观测支持表） | active |
| admission | 资源限额（有界读）+ 路径域/reparse 审查 | active |
| ir | payload/oracle 解析为 owned bytes；oracle 身份=逐字节内容哈希 | active |
| suite / discovery | suite 编排、显式 defaults 深合并、逐文件发现（错误保留不静默丢） | active |
| compare / report | 观测比较（bytes/lf/text/message）、逐 check 报告与 gate 退出码 | active |
| sha256 | 极简指纹（无外部依赖） | active |

## 支持表

**v1 冻结（avm/a2r）**：

| 通道 | avm/run | a2r/emit |
|---|---|---|
| completion（success / error{stage,code?,message?}） | ✓ | ✓ |
| result（legacy-display） | ✓ | — |
| stdout / stderr | ✓（错误完成下不可声明） | — |
| artifacts（多产物逐项 bytes） | — | ✓ |

- error completion 必须带稳定辨识（code 和/或 message exact/contains）；空 code、
  空 matcher、裸 stage 一律 admission ERROR（无辨识力的强 oracle被拒）。
- 声明本 adapter 观测不到的通道 = admission ERROR，不是忽略。

**v2 native/syntax（PLAN-788，B 阶段）**：ac×{check,build,run}（core/scalar，
typed stages/exit/原始 bytes 流/预算硬 cap/部分产物边界/783 cache 三模式/逐行
身份轴）与 syntax×{parse,edit}（两 grammar、三投影、fresh↔incremental 等价），
语料 42 case + legacy-map 对账；见 [profile-v2-native.md](profile-v2-native.md)。

> 来源：docs/plans/archive 待归档的 773-autodown-testcase-protocol-pilot.md（A 阶段）、
> docs/plans/788-autodown-ac-native-turnaround.md（B 阶段）；
> 合同细则见 [profile-v1.md](profile-v1.md) / [profile-v2-native.md](profile-v2-native.md)，
> 结构与依赖约束见 [architecture.md](architecture.md)。
