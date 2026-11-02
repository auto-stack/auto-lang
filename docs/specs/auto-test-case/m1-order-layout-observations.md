# M1 命名求值顺序与布局场景观察（m1-order-layout-observations）

> 状态：delivered（PLAN-914，2026-11-02；r2 判据 = 2026-11-02 用户裁定 D-05「采用引擎
> 真实策略」）。本文件是 E01/E02（命名求值顺序）与 I04/I09（布局失效/活状态）四个
> scenario 行、12 个逐路径格的**输入、真实生产入口、父侧准入与逐格证据**的规范正文。
> 语料：`crates/auto-lang/test/cases/ace-m1/order-layout-914/`，固定组
> `case-ace-m1-order-layout-914`；scenario 语义本体的 oracle 仍由
> [ace-m1-scenario-oracles.md](ace-m1-scenario-oracles.md) 冻结，本文件只补「怎么真实
> 产生并观察」。

## 1. 范围与格位

| 行 | 不变量 | 必需观察 | 格 |
|---|---|---|---|
| E01 | 命名实参按**源码序**求值、按**形参名**绑定 | 值 7 且 trace 字节恰 `ba` | ac/av/avm/a2r |
| E02 | 命名 type 构造按**源码序**求值、按**字段名**绑定 | 值 103 且 trace 字节恰 `ba` | ac/av/avm/a2r |
| I04 | type/layout 变更后依赖精确失效、无关 defs 零生产、旧值安全 | 见 §4 | ac/av |
| I09 | 布局不兼容/活状态：按实际策略拒绝替换或要求重启，不隐式重解释 | 见 §5 | ac/av |

12 格全部由 `order-layout-914/` 的正式 case 实测（15 个 check），不把「原 src 不存在」
当缺口，也不伪造 `src/E*.at` 之外的原始文件（E/I 行按设计无 corpus 源）。

## 2. 冻结输入（tracked）

- `src/E01-ace.at`：`sub(a,b)=a*3+b*2`，`main = sub(b: mark_b(), a: mark_a())`；mark
  内建 `mark_a()=1`、`mark_b()=2`（capability `hir.test.trace.v1`）。oracle：值 7、trace `ba`。
- `src/E02-ace.at`：声明序 `type Pair { left, right }`，构造 `Pair { right: mark_b(), left: mark_a() }`，
  `fan(p)=p.left*101+p.right`。oracle：值 103、trace `ba`（**构造序右先**，故声明序求值会
  表现为 trace `ab`）。
- `src/E01-legacy.at` / `src/E02-legacy.at`：旧引擎无 capability 内建，冻结副作用由普通
  Auto 函数 `write("b")/write("a")` 承担（`write` 不换行；`print` 会换行，不得混用）。
- `src/I04-ace.at`（AC 记录 + `unrelated`）、`src/I04-av.at`（AV 同形 + `unrelated(0)` 干扰项）。
- `oracle/m1_e01.rs`、`oracle/m1_e02.rs`（a2r 金样）与 `reference/E01-a2r-reference.rs`、
  `E02-a2r-reference.rs`（独立手写 reference，`print!` 记号 + `println!("i32:{}")`）。
- 判别性（必须可区分）：位置绑定 → 8 / 203；声明序求值 → trace `ab`；E02 字段错绑 → 203。

## 3. 四路径真实生产入口与观察通道

| 路径 | E01/E02 生产入口 | trace 通道 | 父判 |
|---|---|---|---|
| ac | `probe` + `ac/mir-native-scalar-v1`（E01）/ 同 recipe 的 record 源（E02） | PE 真实 **stderr** 字节 | `expect.stderr{text:"ba"}` 精确比较 |
| av | `probe` + `ac-av/core-run-v1`（E01）/ `ac-av/data-run-v1`（E02） | worker 上报的 `step.trace`（执行序事件） | `expect.trace{text:"ba"}`（仅 `ac-av/*` 可声明） |
| avm | `avm` 适配器跑同一 legacy 源 | 捕获的 **stdout** 字节 | `expect.result` + `expect.stdout{text:"ba",compare:"bytes"}` |
| a2r | `a2r` 适配器 emit（`module_name`）+ 金样逐字节 | 金样即产物字节轴；行为认证另见 §6 | `expect.artifacts` bytes |

旧引擎命名绑定：`ParamInfo` 带形参名、全命名直调按「源码序求值到隐藏局部 → 按形参序入栈」
发射；`Expr::Node` 注册类型字面量按 `Key` 变体取原始字段名后同法绑定（位置/部分/未知形状
保留历史发射）。a2r 对**标量形状**的全命名调用发射闭包形式
`(|__a0,__a1| sub(__a1,__a0))(实参源码序)`；非标量形状仍是历史发射（登记边界）。

## 4. I04（ac/av）冻结判据（D-05）

declaration/layout 变更**允许**「保留环境冷全前端 + 依赖精确 rebuild」——这是两台引擎的
既有策略，冷前端工作必须**如实计量**（AC `declaration_frontend_invalidations`、AV 冷前端
body work），不得隐藏或伪造为零。必须同时满足：

1. 受影响 defs 的失效集**由真实依赖事实决定**（AC：`prepare_declaration_change` 的
   COFF-meta 复用 + 不兼容 unit 才进 object producer；AV：`apply_source` 的
   changed-types/nesting/signature 事实）；
2. 未受影响的 defs 的 **lower/encode/codegen/load/工件IO 增量 = 0**，同进程 resident
   MIR/COFF 对象与代际不变（AC 用 `identities_before/after` 的 MIR/COFF 指针承载；
   AV 用 `identity_all` 的 ptr/generation 承载），声明的 `stable_defs` 逐个核对；
3. 旧成功值仍安全、不被按新 layout 隐式重解释；
4. 禁止以「一律拒绝任何 type 编辑」或「全量重编/只换 hash」冒充通过。

已冻结的引擎事实（不得当作 bug 隐去）：AV data profile 的每个 unit 内嵌**全部 def 的
import 签名**，因此使用中记录类型的 layout 变更按设计是**全 unit 签名变化**（rebuild 集
= 类型闭包全集）；「未受影响」的精确性由**未被任何 def 引用的声明新增**（零 rebuild、全部
身份承载）与 **helper body 编辑**（只重建该 def）两格证明。AC 侧由 records 体编辑
（`codegen_units=2/3`、`unrelated` 承载）与声明失效轮证明。

## 5. I09（ac/av）冻结判据

- **ac**：保留环境对不兼容替换给出**类型化拒绝**（`mn.incremental.refused.<kind>`，本次
  实证 `signature-or-context-drift`），父侧把它判为声明的 diagnostic completion（stage
  `verify`），**不是 infra**；同一 session 的后续请求仍运行已接受环境。
- **av**：对「只重建部分 defs 的布局候选」走受控 `apply_source_partial`，link/env-contract
  门必须**拒绝**（typed `env-contract` 或 `link-rejected`），且拒绝后旧快照仍可执行
  （后续 run 返回旧值）。父侧不推进已接受源码；不得把拒绝降级为 infra 或静默接受。
- 任何路径都不得隐式重解释旧值、不要求在线的 OSR/HMR。

## 6. a2r 双轴与显式认证（本场景域）

日常只做 emit + 金样字节比较；真实 Rust 行为认证是显式操作：对 E01/E02 的 legacy 源，
actual（发射产物）与 `reference/` 独立 reference 各自经 rustc 编译运行，两者 stdout 必须
都等于冻结 oracle 串 `bai32:7\n` / `bai32:103\n`（trace 字节 + 值一次可判）。旧
`ace.m1.cert/1` 的字段与语义不变；本场景的双实测证据以计划 §7/§9 记录，不改旧证书轴。
`write` 映射为 Rust `print!`（无换行），wrapper 的最后一行 `i32:<值>` 是值轴。

## 7. 父侧准入与预算

- 所有观察通道必须先**声明**（`expect.stderr` 仅 ac/mir-native-*；`expect.trace` 仅
  `ac-av/*`；`mn.declaration` 仅 collections；`av.invalidation` 仅 data-incremental），
  未声明不比较、声明了不可观察即准入错误；**worker 的期望永不进入请求**。
- 两向绑定：声明轮/编辑步必须有对应 oracle，oracle 不能悬空；稳定与重建的 def 不得重叠。
- 预算：单操作 receipt ≤4 MiB（本场景实测 1.0–20.2 KiB）、stdout/stderr ≤1 MiB/流、
  AV 值 8 MiB / 受管 32 MiB / 父增量 128 MiB；AC 声明轮 ledger 峰值（本次 21.3 KiB）
  计入既有 ledger 门。超额即非 PASS。
- 否认/拒绝类结果（typed refusal）是 verdict；缺字段、协议破损、非零 worker 退出、
  超时、缺产物一律 infra，不满足任何语言负例。

## 8. 非目标与边界

native 闭包/Dyn ABI 扩展、在线 loader/OSR、资源事件全谱、完整 A2R 认证/旧产品切换、
913 的 closures source recovery、916/917 的旧 A2R 静态准入与 mut/text 发射、以及
native（avm）B18 等残余 M1 格均不在本文件范围；I04/I09 的 avm/a2r 两列按 NA-6 冻结
（见 [ace-m1-applicability.md](ace-m1-applicability.md)）。
