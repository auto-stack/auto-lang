# ACE 标准库宿主服务合同（console/1 与有界资源前探）

> 状态：PLAN-947 交付稿（worktree 预备；merge 时正式沉淀）。AV/native 的
> implemented 状态以 `crates/auto-lang/test/cases/ace-m2/p947-console/` 的
> 双 lane 真实执行为证据；Rust/C 与资源端口缺口按"capability-missing 单列"
> 记录，不冒领、不静默扩权。

## 1. console/1 宿主合同（冻结合同）

- **类别与符号**：`console/1`；`auto.console.print.{i32,bool,str,char}.v1`（stdout）、
  `auto.console.eprint.*`（stderr）、公共拼写 `say(str)` ≡ `auto.console.print.str.v1`
  （stdout；旧 `io.say` 冻结证据链：`io.at:21` 声明 → `io.vm.at:51` `say→print` →
  VM codegen `("io","say")→auto.print_str` → `vm_print` 在字符串后恰追加一个 `\n`）。
- **格式化**（guest 侧，AV 解释器与 mn 机器码逐字节一致）：i32 十进制 ASCII 无正号/
  前导零（MIN 保留负号）；bool `true`/`false`；char 按 Unicode 标量 UTF-8（≤4 字节）；
  str 原字节（内嵌 NUL/换行保留）；每调用恰追加一个 `0a`。
- **预算**：单调用（含追加 LF）≤ `MAX_CALL_BYTES=65536`；每 run 累计 ≤
  `MAX_RUN_BYTES=1048576`；每模块 ≤8 个 console overload 声明；两次取消检查点之间
  原子提交（不半提交）；`ProviderScope` 无 guest 重入。
- **授权**：每 run 注入 `console.stdout`/`console.stderr` caps，先于任何 dispatch
  （`av-core host::authorize` 共享实现）；恢复产物只携带身份事实（owner/entry/
  snapshot/指纹），永不携带授权、provider 地址或凭证；重新授权即可执行。

## 2. 四端支持分账（console 公共输出）

| 端 | 状态 | 证据/边界 | 后继 |
|---|---|---|---|
| AV | implemented | `ac-av/console-run-v1`；语料 p947 双流字节精确 + Freshness | — |
| native (AC/mn) | implemented | `ac/mn/console-run-v1`：共享前门→persist/restore 重接纳→每轮授权→逐函数 Console 发射（guest 格式化+per-call 预算）→shim（累计预算+kernel32 WriteFile 每调用原子）→真实 PE 运行。**Unit-result 入口=命名能力边界**（mn 冻结零参 i32 入口 ABI），AV lane 承接 Unit 通道（corpus O07/O09） | Unit 入口随 mn 入口 ABI 演进单列裁定 |
| A2R (Rust host) | capability-missing | 无 Rust host console provider；host-console profile 不入 Rust 发射面 | PLAN-973（四端补齐） |
| A2C (C host) | capability-missing | 无 C host console provider | PLAN-973 |

机器账：`stdlib/contracts/compatibility.toml` `console.say` 条目
（ace_av/ace_native=supported、hir_rust/hir_c=missing_capability、owner=PLAN-947），
由 `stdlib_assembly::contracts` 审计词汇表与 `contracts/tests.rs` 钉扎测试双锁定。

## 3. 有界资源前探分账（File/Task/Stream；本包仅前探，已交付=无）

前探结论（2026-11-13，基线 HEAD 见 PLAN-947 §4.1；每行有实际代码依据）：

| 资源面 | 现状（实际 producer/consumer） | 状态集合覆盖 | 939 依赖事实 | 缺口归属 |
|---|---|---|---|---|
| File | 旧 VM 专属：`stdlib/auto/io.at` `type File`（open/read_text/read_line/read_char/read_buf/write_line/flush/close）经 `io.vm.at` ext 绑定到 `crates/auto-lang-core` VM native（assembly `validate.rs` File.* 登记）；ACE 侧无任何 host 端口 | 旧 VM 有创建/读/写/flush/close；失败/close 后使用/跨会话恢复无合同。ACE：全部状态 capability-missing | 939 已归档交付 Handle/Callback/ViewText **契约形态**（`foreign-contract/src/resource.rs`，`RESOURCE_CAP="foreign.resource"`）；无 Auto 源 foreign import 模型（938 范围），File 端口未接 | 系统服务：PLAN-973；异步端口组合：PLAN-967 |
| Task | 旧 VM 专属：`async.at`/`async.vm.at` chan/spawn/sleep/task 族（#[vm] native）；ACE 无任务端口，无执行器（本包不造） | 创建/join/取消仅旧 VM 行为；ACE：capability-missing | 939 无任务语义；调度器非本包/非 939 | 异步端口：PLAN-967；调度：PLAN-968 |
| Stream | 旧 VM HTTP 流（`http.vm.at` 流方法 + a2r-std 有限客户端）；ACE data/collections lane 无流端口 | 旧 VM/旧 Rust 发射有有限流；ACE 统一流端口：capability-missing | 939 无流形态 | 系统服务/分发：PLAN-973/978 |

**capability-missing 输出形状（V-08 第三腿的合同化）**：上述三面在任何装配
请求下必须得到具名拒绝（`assembly.no-provider`），绝不允许以"选择成功"或
空收据冒充支持——`provider_selection::plan947_tests` 的 File/Task/Stream
前探 oracle 钉住该形状。

## 4. 默认自有实现与 FFI 边界（AC-NATIVE 分账基线）

默认 core/alloc/基础系统库由 Auto 自有实现与薄 OS 适配承担；不以 libc 或 Rust std
间接委托为默认；公共 AutoABI 与 OS/外部 ABI 分开建模，Rust/C/provider 依赖显式
选择。缺默认 provider 或自动回落 libc 必须拒绝；合法自有 provider 可装配。
Storage/allocator 接口保持 M2 内存专题待定（本包不冻结签名/ABI 布局）。

## 5. 目标条件语义与 sys/OS 迁移表（交 966 实施；本包冻结合同）

`#if`/`#is` 按**目标平台**（非宿主）求值——本包四轴选择合同
（`stdlib_assembly::provider_selection`）是其装配层判据：宿主 Windows 而
目标 Linux 时 provider 集取 Linux（V-09 已钉）；未选分支仅做语法检查不
进入装配；导入提升到所在模块；无/歧义 provider 具名失败
（`assembly.no-provider`/`assembly.ambiguous-provider`）。源码级 CTE 归
PLAN-966，本包不另造。

现行 sys/OS 公共名字迁移表（基线：`stdlib/auto/sys.at` 仅 `getpid() int`，
实现位于 `crates/auto-lang/src/libs/sys.rs`；按现行 `#` 语法——HashIf/HashIs
语句级构造——迁移，不新增关键字）：

| 现行公共名 | 目标条件映射（966 实施面） | 装配层 provider（本包合同） |
|---|---|---|
| `sys.getpid()` | `#if TARGET_OS == windows` → Win32 `GetCurrentProcessId`；`linux` → `getpid` | `OwnAuto`/`PlatformAdapter`（`PlatformAbi` 如实标注；平台差由薄 OS 适配吸收） |

新增 sys/OS 名字=合同修订（新行须同步本表与选择注册面）；不在本表的名字
不因 `#if` 分支存在而自动成为公共 API。

## 6. 装配收据与源码/运行加载边界（T-10）

四轴选择以 schema 5 增量节进入 `AssemblyManifest`（`freeze_with_selection`）：
选择记录只携带身份事实（operation/module/symbol/layer/target/provider/abi），
参与共同与消费者双指纹；schema 4 旧线型逐字节兼容（空选择节），交叉形态
（schema 4 带选择节 / schema 5 无选择节）拒绝。源码装载（编译期装配/选择）
与运行装载（每轮 grant 注入、dispatch 前检查）边界固定：收据永不携带授权
或 provider 地址；恢复产物重新入场必须本轮重新授权
（`assembly.missing-grant`，V-10）。

## 来源

- 冻结合同与证据：[PLAN-947](../../../plans/947-ace-stdlib-console-and-resource-host-contract.md)（§5 T-01 冻结记录、§9 验证记录）、[PLAN-1039](../../../plans/archive/1039-ace-platform-io-primitives.md)（受检 `platform.io/1` 与 `platform.mem/1` 交付）
- console/1 合同规范：[auto-ac/source-console-output.md](../../auto-ac/source-console-output.md)、[auto-av/host-import-and-console.md](../../auto-av/host-import-and-console.md)
- 平台原语与 OS 内存合同：[native-platform-io.md](native-platform-io.md)
- 939 资源形态：`experimental/ace/crates/foreign-contract/src/resource.rs`

## 7. platform.io/1 与 platform.mem/1 分账（PLAN-1039，2026-11-14）

在保留第 1–2 节 `console/1` 合同与 `p947-console` 不变的前提下，ACE `HostConsole`/`AVHC` 经独立指令 `MirInstr::PlatformCall`
（`OP_PLATFORM_CALL = 0x3C`）交付 Windows x64 受检平台 I/O（`io_stdout`/`io_stderr`/`io_write_some`/`io_write_some_slice`/`io_last_error`）
与 checked OS 虚拟内存（`mem_page_size`/`mem_alloc_granularity`/`mem_alloc`/`mem_free`/`mem_logical_len`/`mem_mapped_len`/`mem_borrow`/`mem_unborrow`/`mem_write_u8`/`mem_read_u8`）：

| 端 | 状态 | 证据/边界 | 后继 |
|---|---|---|---|
| AV | implemented（Windows） | `av-core/src/platform_io.rs` + `Win32PlatformIo`；`ace-av::conformance platform_io_1039`（5 用例，含 `persist`/`restore` 授权重放与 `stdlib/ace/internal/platform_mem.at`） | — |
| native (AC/mn) | implemented（Windows x64） | `native/src/platform_io.rs`（`mn-platform-shim` 纯 `KERNEL32.dll` + `audit_pe_imports`）；`ace-native::conformance platform_io_1039`（6 用例，含匿名管道 `ERROR_NO_DATA`=232、`VirtualFree` 失败保原 owner 与 PE 白名单负控） | Linux ELF 归 PLAN-1040 |
| A2R / A2C | capability-missing | 未新增 Rust/C 公共 `platform.*` provider；公开 `Storage` API 与 `compatibility.toml` 未冻结/未改动 | PLAN-972/973 |
