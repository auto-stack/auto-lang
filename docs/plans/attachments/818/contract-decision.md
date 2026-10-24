# PLAN-818 contract-decision.md — T-01 只读设计冻结（MIR→新 native）

> **r1.2 注记（2026-10-24，D828 达成后的刷新义务履行 + T-02 重放）**：828 已
> archived 11/11 delivered（门禁裸 t 5331/5331+tv 163+probe 消费面、SD-01..06、
> ledger v70、合并祖先核验过）。刷新事实：①本合同消费的全部 798 API 存续
> （`compile_scalar_source`/`lower_checked`/`verify_mir`/`split_units` 签名未变；
> 828 新增 `lower_function_unit` 函数级 lowering 与 `fn:<self>` 快照链、
> `session.rs`/`function_queries.rs` 消费接口——为 T-11 的消费目标，不改 T-02 形态）；
> ②MIR/AV core 表面与版本轴未变（contract.rs 零 diff）；③**注册映射刷新**：
> PLAN-814 大迁移落地，`test_cases` 模块 → `crates/auto-lang-core/src/test_cases/`
> （ac_mn.rs/dispatch 落位新家；测试与语料仍在 auto-lang；auto-test-case 未动）；
> ④**跨兄弟消费缺陷与修复**（路由 KNOWN-DEBT）：828 给 auto-av-probe 新增
> ac-syntax-core-adapter 依赖后，experimental/ 内兄弟 workspace 经 path 链消费时
> cargo 将 ac-syntax 成员的 `.workspace=true` 继承错析到 auto-lang 根——以语义
> 等价字面量修复（ac-syntax 自建 39/39 零回归）。T-02 定向重放全绿：首例 3/3
> + AutoDown 3/3 @ c4819bf58（rebase 后基面）。

> **r1.1 注记（2026-10-20，D-798达成后的重读义务履行）**：以实际合入master的798面
> （archive 11/11）重读本文件§1——MIR核心表面未变（指令/终结器/类型/槽/verify规则，
> 仅新增`MirInstr::can_trap()`分类器，与本文件§2 trap家族完全一致）；`contract.rs`的
> ABC wire schema 升 `auto.abc.av-probe/2`、ABC语义升2（unit/module kind位入wire）——
> **不触及本合同A09'的key轴**（其只绑MIR schema `auto.mir.av-probe/1` + MIR语义1，
> 两者均未变）；hir_adapter变化为语义保持的on-demand join优化（F798-R7-05），golden
> 绑定的observable行为不受影响。无迫使revision的反例；T-02实现即基于此合并面。

2026-10-20，work stage T-01。本文件在**已观察支持面**内冻结新 `MIR-native` 后端的
指令表映射、逻辑符号、私有测试 ABI、entry wrapper、trap 发射、source sidecar、
预算/负载/错误分类、对象 key、缓存层/载体/恢复、工具链与参考机器身份、内存分区
及测试 owner 映射。独立期望（golden）见同目录 [golden.md](golden.md)。
T-02 起实现并验收；任何变更须回本文件记 revision。

**T-01 不构成可行性结论**：whole-path 可行性门（真 COFF/真多 obj 链接/真 PE/三路
径一致/新进程 cache 恢复/正式 AutoDown）在 T-02 以真实执行判定；本文件只是其
冻结合同。T-01 全程只读（未编译在途接口、未造 mock 后端、未开实施树）。

## 1. 观察基面与依赖身份（不冻结在途接口）

观察基面 = 798 在途分支 `plan-798-dev` @ `072cb4e14`（R3 修复轮）的
`experimental/ac-av-probe/core/src/{mir,mir_verify,contract}.rs`、
`probe/src/{hir_adapter,cache,native_admission}.rs`，以及主检出已交付的
`experimental/ac-core/src/{native,link}.rs`、`test-support/`、
`experimental/ac-artifact-probe/src/store/`。**这是 forecast，不是冻结依赖**
（batch inputs.json 已注明 authority: "forecast only; requires D-798"）。

已观察 MIR 表面（与 plan §4 预测一致）：

- 指令 `MirInstr::{Const,Copy,Bin,Un,Call,Trace}`；终结器
  `Terminator::{Jump,Branch,Return}`；typed mutable slots（Param/Local/Temp），
  **非 SSA**。
- `BinOp` 11 个：Add/Sub/Mul/Div/Rem I32 + Lt/Le/Gt/Ge/Eq/Ne I32（后 6 个结果
  bool）；`UnOp` 2 个：NegI32/NotBool；`TraceEvent::{MarkA,MarkB}`。
- `MirType::{I32,Bool,Fn(FnSig)}`；consts 表 `{I32(i32),Bool(bool)}`。
- 单元粒度：单函数 `MirModule`（`is_unit`），callee 只经 `fn:<name>` import
  （结构签名）；模块闭包 entry 合同 = 零参 i32。
- imports allowlist：`hir.test.mark_a` / `hir.test.mark_b`（效果 Observable）+
  单元内 `fn:<name>`；能力 allowlist：`trace` = `hir.test.trace.v1`。
- trap 分类（AV 侧）：int-overflow / div-by-zero / div-overflow(MIN/-1) /
  rem-overflow(MIN%-1)。
- 798 冻结身份：MIR schema `auto.mir.av-probe/1`、profile `av.core-i32/1`、
  producer `plan798/av-probe`、MIR/ABC/AV 语义版本各 1；单元名 = HIR def id
  （如 `d_main`），前端声明名（`sf_*`）是内部符号。

**D-798 重读义务**（plan §4）：T-02 起实际编译共享 MIR 前必须以实际合入的
798 API/Spec 重读本文件 §1/§2/§9——798 若改 profile/指令/验证规则，按新合同
修订本文件（记 revision），不照旧 branch 开工。798 的 A08 MIR 单元恢复准入
（R3：manifest profile/av_sem/link.sig 对解码载荷渲染比对/logical 再推导/
sema-ref envelope+payload_fp/caps 重授）由 798 交付面自带，818 只消费不复制。

## 2. 指令表映射（MIR → CLIF，全表）

约定：`V(s)` = slot s 的 Cranelift `Variable`（见 §4 槽映射）；每个 MIR 块
→ 一个 CLIF block；CFG 静态完整 → **全部块预先创建并预先 seal**（前驱集合
由 MIR 显式给出，无部分 seal 状态）。

| MIR | CLIF 发射 | 语义注记 |
|---|---|---|
| `Const{dst, value}` | consts 表取值：I32(v)→`iconst I32 v`；Bool(b)→`iconst I32 (0\|1)`；`def_var V(dst)` | bool 物理值恒 0/1 |
| `Copy{dst, src}` | `def_var V(dst), use_var V(src)` | 类型已由 verify 保证同形 |
| `Bin{AddI32}` | `checked_add`：`iadd` + 房间法溢出守卫（b≥0&&a>max−b ‖ b<0&&a<min−b）→ `branch_to_trap_on` | 与落地算法逐字对齐（native.rs checked_add） |
| `Bin{SubI32}` | `checked_sub`：`isub` + 房间法守卫（b≤0&&a>max+b ‖ b>0&&a<min+b） | 同上 |
| `Bin{MulI32}` | `checked_mul`：sext→i64 `imul`→`ireduce I32`→再 sext 比较不等 | 同上 |
| `Bin{DivI32}` | `guarded_sdiv`：除数先净化（b==0→1，b==−1→1），`sdiv`，b==−1 商选 `ineg a`，trap 条件 `bz ‖ (bm1 ∧ a==MIN)` | 硬件 div 永不见 0/−1 除数；trap 前置分支，非事后检查 |
| `Bin{RemI32}` | `guarded_srem`：同净化模式，`srem`，非 MIN 的 b==−1 余选 0，trap 条件 `bz ‖ (bm1 ∧ a==MIN)` | MIN%−1 按标量合同 trap |
| `Bin{Lt/Le/Gt/Ge/Eq/Ne}` | `icmp(IntCC::*) → uextend I32 → def_var V(dst)` | icmp 产 i8，必须 uextend 到 I32（QA-01 教训） |
| `Un{NegI32}` | `checked_neg`：`a==MIN` 守卫 → `ineg` | MIN 取负 trap |
| `Un{NotBool}` | `icmp(IntCC::Equal, x, iconst 0) → uextend I32 → def_var V(dst)` | 不依赖 0/1 前置的 `bxor`；产出严格 0/1，不变式自持 |
| `Call{Func(idx)}`（仅模块形态，实验 driver 不产生） | `declare_func_in_func` + `call`，结果 `def_var V(dst)` | 单元形态下不存在（verify 拒绝） |
| `Call{Import(mark_a/mark_b)}` | import 符号 `declare_function(Linkage::Import)` + `call`，结果 `def_var V(dst)` | 运行时冻结返回 1/2（对齐落地 test-support） |
| `Call{Import(fn:<name>)}` | 逻辑符号（§3）`Linkage::Import` + `call` | 链接期由 manifest 解析 |
| `Trace{MarkA/MarkB}` | 调用对应 import 符号，结果丢弃 | 与 Call{Import} 同通道：stderr 单字节 + 返回值 |
| `Jump{target}` | `jump block[target]` | |
| `Branch{cond,then,else_}` | `brif use_var V(cond), then, else_` | I32 0/1；brif 语义 = 非 0 走 then |
| `Return{value}` | `return_ [use_var V(value)]` | |

溢出/trap 出口（每函数一个 trap70 块，镜像落地实验合同）：

```text
trap70: iconst I32 70 → call ExitProcess(70) → trap INTEGER_OVERFLOW(不可达)
branch_to_trap_on(ovf): brif ovf → trap70 / cont；cont 预 seal 后继续
```

**未支持面**：未知 opcode/版本/profile/能力/import 在 codegen 前
`mn.unsupported.*` 拒绝（codegen 计数 0）；不 NOP、不回译 HIR、不转旧
lower_object。Cranelift verifier 失败 = `mn.backend.lowering`（后端错误），
不得重做语义推导/删分支来通过；不引入 MIR 之外的默认零值。

## 3. 逻辑符号方案

- **函数符号**：`ac8.<16hex>.<unit-name>`，其中 16hex =
  FNV-1a64(schema "\n" module)（owner scope 的内容轴；schema 取 798 实际
  合入值）。例：`ac8.1a2b3c4d5e6f7788.d_gate`。
- 符号由 **owner scope + 单元名（声明身份）+ 签名内容 + 私有 ABI 配置** 决定，
  **不含 body hash**——body 编辑后符号不变、仅对象内容更新（plan §5.3）；
  callee 签名/ABI 变化 → 签名内容变 → 相关 caller 的 key 失效（§9）。
- **导入符号直通**：`hir.test.mark_a` / `hir.test.mark_b`（与落地
  test-support 的 `#[export_name]` 一致，不改名）。
- **wrapper 符号**：`ac_start`（沿用现有实验合同符号，语义 = 调 entry、
  `ExitProcess(result)`）。
- **进程 API**：`ExitProcess`（kernel32），每个含 trap 路径的对象与 wrapper
  均可 import，链接期由 kernel32.lib 解析。
- 符号内不含绝对地址/文件系统路径；函数次序改变不改变符号（引用不串）。

## 4. 私有测试 ABI（`msvc-x64/priv-i32-bool-v1`）

| 项 | 冻结值 |
|---|---|
| 调用约定 | `CallConv::WindowsFastcall`（内部函数、wrapper、trace imports、ExitProcess 全部） |
| i32 | CLIF `I32`，按 Win x64 传参/返回规则 |
| bool | **物理 I32，逻辑值恒 0/1**（Const/比较/NotBool 三个生产者都严格 0/1；调用通道不区分） |
| 参数 | 按 MIR 签名顺序（求值顺序已由 MIR 显式化；>4 参数走 Win x64 栈传，由调用约定保证） |
| 返回 | 单标量寄存器返回；void 仅 wrapper |
| 栈/帧 | Cranelift prologue/epilogue 自管；有限递归受 5s run 墙与物理栈约束（不等同 AV 帧预算） |
| trap 出口 | ExitProcess(70)（语言 trap 出口；对齐落地实验合同） |
| entry 观察出口 | wrapper 把 entry 的 i32 **按位**作为 u32 传 ExitProcess；观察者比较完整 32 位退出码（如 i32::MIN → 0x80000000），不只比低 8 位 |

**分域声明**：此 ABI 是 818 私有测试 ABI，**不是 797 公共 AutoABI**，不拿同
版本字样证明兼容；未来 FFI 接入另立计划。

## 5. 槽映射与初始化

- 每个可变局部槽 → 一个 Cranelift `Variable`（I32；bool 槽同 I32 物理形）。
  FunctionBuilder 处理 SSA 桥，**不把 MIR 改写成强制 SSA**。
- Param 槽：entry block 参数按签名顺序 `def_var`（verify 已保证 Param 先导
  且序同签名）。
- 初始化：798 verify 的 definite-initialization（入口可达 CFG 上的保守
  must-worklist、块入口 = 前驱出口交集）保证每次 read 时所有前驱路径已定义
  → `use_var` 合法；不引入隐式零值。
- 块序：MIR 块序即 CLIF 创建序；每函数先建全部块、先 seal 全部块（CFG 静态
  完整），再按序发射指令与终结器。

## 6. source/debug sidecar

- 沿用 798 sidecar 形态：每函数 span 表 `(instr, func, start, end)`（flat
  指令索引，块主序）+ 源 snapshot 指纹 u64。**仅诊断用途**，不参与语义与
  对象 key（位置漂移共享代码对象；诊断漂移时以当前快照 sidecar 重绑，单独
  计数——798 R3 语义）。
- 818 侧存储：sidecar 作为独立小产物随 A09 对象同层引用（provenance 边，
  不永久强 pin）；trap/错误诊断经它映射回源 span。

## 7. 预算、负载与资源/错误分类

### 7.1 继承预算（798 冻结面，818 不重解释）

source/HIR/MIR 输入 ≤4 MiB；codec 页 ≤256 KiB；表条目**每表族** ≤65 536
（模块表 + 每函数表各自）；前门嵌套 ≤128（含平坦运算符链构造深度）；verify
≤2M 工作步 / wall 5s / 组合分析内存 16 MiB（分配前）/ 解码堆估算 16 MiB。
违反 = 798 `av.resource.*` 分类（在 798 交付面内拒绝，818 不复检不放宽）。

### 7.2 native 侧硬预算（本计划冻结）

| 预算 | 值 | 分类 |
|---|---|---|
| 单函数 COFF | ≤16 MiB | `mn.resource.object-size` |
| 一个请求对象合计 | ≤64 MiB | `mn.resource.request-objects` |
| link 输入合计 | ≤128 MiB | `mn.resource.link-input` |
| 元数据/IPC | ≤4 MiB | 788 冻结上限（metadata 面） |
| stdout / stderr 捕获 | 各 ≤1 MiB | `link.output-cap`（788 CaptureCaps，截断即树终止） |
| build（链接）wall | ≤60s | `mn.resource.build-wall`（对齐落地 SUBPROC_DEADLINE 60s） |
| run wall | ≤5s | `mn.resource.run-wall`（对拍用有界程序；超时/崩溃=infra 非语言 trap） |
| 请求合计 wall | ≤70s | `mn.resource.request-wall` |
| 受管数据 | ≤48 MiB（分区见 §11） | `mn.resource.managed-memory` |
| worker 私有内存峰值 | ≤512 MiB | `mn.resource.worker-peak`（Cranelift 分配器计入；前端归因单报但计入） |
| 请求树峰值 | ≤768 MiB | `mn.resource.tree-peak`（含受控 link/PE 子进程） |

不可中途取消的编译用父侧 contained 预算终止并归 resource/infra，不继续输出
成功产物。阈值=0 的时间/内存反控必须使聚合 gate 非 0；raw 中零/缺失/NaN/
失败样本不得按 0ms/0bytes 通过（T-05/T-07 反控义务）。

### 7.3 错误分类（`mn.*` 命名空间）

- `mn.unsupported.*`：version/instr/profile/capability/import——codegen 计数 0
  处拒绝（未知即 unsupported，不 NOP 不降级）。
- `mn.admit.*`：owner/target/profile/abi/callee-sig/address/manifest——恢复
  准入的**受控拒绝**（非 miss、非覆盖重建）；自洽异址/坏对象身份/内层
  manifest 错配在消费门拒绝，不链接不匹配对象。
- `mn.backend.lowering` / `mn.backend.emit`：Cranelift 发射/verifier/对象
  序列化失败（后端错误类）。
- `mn.resource.*`：§7.2 表。
- 链接/执行器失败：复用落地 `link.*` 族（failed/deadline/output-cap/
  containment/restore/publish 等，由 ac-core 公共执行器产生）。
- 语言 trap：PE 退出码 70（§4）；**native 崩溃/超时是 infra**（`mn.infra.*`），
  不得与语言 trap/资源混淆。

## 8. 工具链与参考机器身份（真实入 key）

| 项 | 参考值（2026-10-20 实测） |
|---|---|
| OS/host | Windows 11 x64（win32 10.0.26200） |
| CPU | 13th Gen Intel Core i5-13600KF（20 逻辑核，32 GB RAM）——**参考机非门禁变量**，仅记录 |
| rustc | 1.98.0（88d9e12ae 2026-08-18），host `x86_64-pc-windows-msvc` |
| rust-lld | LLD 22.1.8（rust-lang/llvm-project 52ed14fcd56），sysroot `…/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe` |
| Windows SDK | KitsRoot10 `C:\Program Files (x86)\Windows Kits\10`，um/x64 **10.0.26100.0**（kernel32.lib） |
| Cranelift | codegen 0.126.2（frontend/module/object 同族，以新 workspace Cargo.lock 为准） |
| 覆盖环境变量 | `AC741_RUST_LLD` / `AC741_SDK_UM_LIB`（`find_rust_lld`/`find_sdk_um_dir` 公共覆盖点） |

工具定位与受控执行复用 ac-core 公共 API：`find_rust_lld`/`find_sdk_um_dir`
（工具身份）、`run_deadlined_captured[_with]`（contained 执行器：suspended
start + kill-on-close job + 单一 deadline + 帽定原始字节捕获）、
`enter_self_containment`（父侧自包含）。**linker/SDK 的可执行文件内容 hash
与版本号真实入 link manifest key**；工具缺失/身份漂移 = 受控 infra 失败，
不静默换 host 机器码或 AV 兜底。多 obj 链接由 818 link_driver 组装标准
rust-lld 输入（`-flavor link /nologo /entry:ac_start /subsystem:console
/out:<staged> <obj…> /libpath:<sdk> kernel32.lib [ac-trace-support…]`），
经上述公共执行器运行；不修改旧 link.rs，不复制 Job/管道生命周期。最终 PE
的暂存/提交遵循落地 staging/commit 合同语义（stage tmp → 备份旧产物 →
原子改名 → 失败回收 owned 暂存并保留旧产物；提交后不可回滚状态须有准确
收据），由 818 自有的多 obj 发布路径实现（落地 `publish_artifacts*` 是
单 obj 形状，不复用其代码）。

## 9. 对象 key 与缓存层

### 9.1 层（783 ManagedStore，独立 producer namespace）

| 层 | schema_id | 载体 | 内容 |
|---|---|---|---|
| A08（消费） | `ac798/a08-mir/v1`（798 交付） | LayoutData | MIR 单元；恢复经 798 decode→verify_mir 重授信 |
| A09' | `ac818/a09-coff/v1` | LayoutData 分页（≤256 KiB/页 + 页 manifest） | 每函数一个可重定位 COFF + 对象 manifest（符号/导入/ABI/来源单元 key） |
| A10' | `ac818/a10-link/v1` | LayoutData | 模块链接 manifest（单元→符号→对象 id、runtime/工具身份、entry）与链接回执 |

LayerSpec 关键输入与 798 形态一致（producer/owner/unit/profile/
body-sig-content/callee-sigs/…），coverage：A09'=Backend、A10'=Link；
regeneration：FromUpstreamRecipe。producer = `plan818/mir-native`。
required 边保活当前对象/runtime/必要描述；MIR/source 仅 provenance，不永久
强 pin。大对象按页落盘使每次 publish 服从 783 暂存上限（8 MiB），不一次写
16 MiB。

### 9.2 codegen 单元 key（A09'）

FNV-1a64 over domain-separated parts：

1. producer + layer schema（`plan818/mir-native`、`ac818/a09-coff/v1`）
2. owner scope（MIR schema id + module 名——逻辑身份，无路径）
3. 单元名（函数 def id）
4. **函数语义内容指纹**：签名形状 + slots（kind/类型形状）+ 块/指令/终结器
   的规范渲染 + 引用到的 consts 值 + 类型形状 + callee import 列表及其
   **结构签名**（`fn:<name>` 的参数/结果类型形状）——**不含 span/snapshot**
5. MIR 语义版本 + MIR schema（798 实际合入值）
6. backend 身份：`cranelift/<新 workspace Cargo.lock 实锁版本>`（T-02 定档）
7. target/CPU/flags：`x86_64-pc-windows-msvc` + `cpu=baseline` +
   `opt_level=none,is_pic=false`
8. 私有 ABI 配置：`msvc-x64/priv-i32-bool-v1`（含 trap 出口 70 与 wrapper
   合同）
9. runtime 策略：trace 运行时身份 tag（`hir.test.trace.v1` marks 语义 1/2）

产物路径/时间等非语义项不入 key、不进可共享 payload。**保守粒度规则**：
codegen 实际读到 module 级信息时必须完整入 key（当前设计仅读单元自身
payload——callee 签名内嵌于 `fn:` import；若未来实现读了更多，key 必须先
扩，不漏依赖伪称细粒度）。

### 9.3 链接 manifest key（A10'）

entry 单元 key + 有序 callee 单元 key 集 + wrapper 身份（`ac_start` 合同
版本）+ link 输入身份（rust-lld 路径+内容 hash、SDK um 版本、kernel32.lib、
ac-trace-support 产物 hash）+ 链接 flags（`/entry:ac_start
/subsystem:console`）。

### 9.4 失效与命中

- callee body-only 改动 → 只重键该函数对象；caller 对象在未内联且只依赖
  签名时复用（符号不变，§3）。
- callee 签名/私有 ABI/target/CPU/flags/opt/producer/语义版本变 → 相关
  key 空间更换。
- 函数次序改变不串引用（符号与次序无关）。
- **命中证明**：`codegen_emit_units=0`（带禁调反控计数器——断言 lower 未被
  调用）+ link/run/父侧比较 >0；不缓存 PASS/进程退出结果代替运行。
- 命中对象可在其上游 MIR 缓存已回收、recipe/身份仍可计算时真实 link/run；
  CheckedMir 不从磁盘 trusted 标志恢复。
- 同键异内容 Conflict 不靠 clear-store 规避——非语义项剥离或显式换轴。

## 10. 事务与失败语义

- 只有全部对象/manifest/ABI/runtime 需求预检通过才进入 link。
- 存在旧输出时：坏对象、link 失败、超时、发布 IO 失败按既有 staging/commit
  合同**保留旧产物**并回收 owned 暂存；不以删除输出再重建冒充原子发布；
  提交后不可回滚状态须有准确收据。
- 终止（deadline/kill）回收子孙进程与 owned 暂存（contained 执行器语义）。
- 正式 AutoDown 产物/manifest 身份由父侧准入，每次真实 PE 结果重新比较。

## 11. 内存分区（受管 ≤48 MiB 的构成）

| 分区 | 份额 | 内容 |
|---|---|---|
| MIR/符号热 | 16 MiB | 恢复的 MIR 单元、符号表、闭包索引 |
| lowering/索引/codec 暂存 | 24 MiB | Cranelift 上下文之外的自有暂存、页编码、A09'/A10' 组装、验证分析折算 |
| I/O | 8 MiB | 对象/manifest 读写缓冲、receipt |

worker 私有字节峰值硬门 ≤512 MiB（Cranelift 内部 allocator 计入；前端
211.8 MiB 量级的归因单报但计入总数）；包含受控 link/PE 子进程的请求树峰值
≤768 MiB。工具首次构建内存单列，不从请求内 worker 峰值扣除前端。瞬态
分配 spike 以反控采样器捕捉（≥5ms 轮询，798 mempeek 形态）。

## 12. 首例、参考负载与测试 owner

- **首例**（T-02）：两函数 + 双 mark 的最小 vertical（源文本、手算过程与
  独立期望冻结于 [golden.md](golden.md) §1）；必须
  `CheckedMir → 新 COFF → 真link/run`、AV/旧native/独立 expected 三方一致、
  真实 store put→新进程命中跳 codegen（每次仍 link/run）、正式 AutoDown
  首例/trace/内存计量。任一路不可行即报告阻断回规划。
- **参考负载**（T-05 冻结实现）：≥256 KiB、100 有意义 caller + 14 leaf +
  main，混合 if/else、for 守卫环、抵消算术（无空白填充）；期望由生成器
  闭式镜像推导（不经任何后端）+ 采样手审（golden.md §2）。冻结编辑位：
  body 编辑 = fn_041 首个乘子 +1（期望随之变）；签名冲突编辑 = fn_leaf_2
  增参（正确 frontend 诊断）。规模不得缩小以过门。
- **测试 owner**（族名由对应 T 创建，当前不是执行证据）：

| V | owner 族 | T |
|---|---|---|
| V-01 | `mir_native_first_` + `mir_native_admit_` | T-02/03 |
| V-02/V-03 | `mir_native_codegen_` + `mir_native_parity_` | T-03 |
| V-04/V-05 | `mir_native_cache_` | T-04 |
| V-06/V-07 | `mir_native_transaction_`（+cache_） | T-04/05 |
| V-08 | `mir_native_resources_` | T-05 |
| V-09 | `mir_native_turnaround_` | T-07 |
| V-10 | `mir_native_adapter_` | T-06 |
| V-11 | T-08 审计（无新族） | T-08 |

## 13. 边界（不因实现进度改口）

- 非目标重申：不自研 codegen/SSA 优化器、不替换 Cranelift、无公开
  AutoABI/FFI/C 导入、无 text/record/堆对象/泛型/CTE 执行、无热重载/在线
  装载/OSR/deopt、无完整 AJ、无通用发布器/跨平台包、无完全增量链接器、
  不迁移生产 SDK/默认命令。机器码产物可复用 ≠ 通用增量发布已交付。
- 旧 HIR→native 仅作对拍参照，不替代 actual；新路径不得 `MIR→HIR→旧
  lower_object` 或缓存旧 native 输出冒称新 codegen。
- 唯一实施树 `D:/autostack/.wt/lang-818/auto-lang`、分支 `plan-818-dev`
  （T-02 创建；T-01 在主检出只读完成，产物落本附件目录）。
- 公共写面（TestCase recipe 支持表/模块声明/父侧 dispatch）按批次并行
  合同单写：817 先合入则同步新 master 再追加 `ac/mir-native-scalar-v1`
  注册并复验；797/798 在改时不触碰。
