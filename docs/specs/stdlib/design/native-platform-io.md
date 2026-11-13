# ACE 受检平台 I/O 与 OS 虚拟内存原语（PLAN-1039）

> 状态：PLAN-1039 已交付（Windows x64 AV/AC；SD-01 + SD-MEM 合并沉淀）。
> 前序选择合同见 [ace-host-services.md](ace-host-services.md) 与 [ace-binding-and-assembly.md](ace-binding-and-assembly.md)；
> 公共 `Storage` 接口、完整文件/目录系统与 Unicode 控制台不在本文冻结。

## 1. 操作模型与指令边界（platform.io/1 与 platform.mem/1）

- **指令编码**：新增专用 MIR 指令 `MirInstr::PlatformCall { dst, host, args }` 与 ABC 操作码
  `OP_PLATFORM_CALL = 0x3C`（`experimental/ace/crates/av-core/src/{mir,abc,instr_spec,ownership}.rs`）。
  不修改 `console/1` 专用指令 `MirInstr::HostCall` 及其 `p947-console` 公共输出合同，两套通道正交共存。
- **前端与域准入**：沿 `SourceProfile::HostConsole` → `HirDomain::HostConsole` → `AVHC` 唯一前门接入
  （`experimental/ace/crates/core/src/source/platform_io.rs`、`verify.rs`、`lowering/src/hir_adapter.rs`）；
  `mir_verify` 允许 `host-console` 模块同时声明 `CAP_STDOUT | CAP_STDERR | CAP_WRITE | CAP_MEM`。
- **15 项版本化平台原语**（`av-core/src/platform_io.rs`）：
  - `platform.io/1`：
    - `auto.platform.io.stdout.v1() -> i32`（`cap: platform.io.stdout`）
    - `auto.platform.io.stderr.v1() -> i32`（`cap: platform.io.stderr`）
    - `auto.platform.io.write_some.v1(handle: i32, data: str) -> i32`（`cap: platform.io.write`）
    - `auto.platform.io.write_some_slice.v1(handle: i32, data: str, offset: i32, len: i32) -> i32`（`cap: platform.io.write`）
    - `auto.platform.io.last_error.v1() -> i32`（无需额外 cap）
  - `platform.mem/1`：
    - `auto.platform.mem.page_size.v1() -> i32`（`cap: platform.mem`）
    - `auto.platform.mem.alloc_granularity.v1() -> i32`（`cap: platform.mem`）
    - `auto.platform.mem.alloc.v1(size: i32) -> i32`（`cap: platform.mem`）
    - `auto.platform.mem.free.v1(handle: i32) -> i32`（`cap: platform.mem`）
    - `auto.platform.mem.logical_len.v1(handle: i32) -> i32`（`cap: platform.mem`）
    - `auto.platform.mem.mapped_len.v1(handle: i32) -> i32`（`cap: platform.mem`）
    - `auto.platform.mem.borrow.v1(handle: i32) -> i32`（`cap: platform.mem`）
    - `auto.platform.mem.unborrow.v1(handle: i32) -> i32`（`cap: platform.mem`）
    - `auto.platform.mem.write_u8.v1(handle: i32, offset: i32, byte: i32) -> i32`（`cap: platform.mem`）
    - `auto.platform.mem.read_u8.v1(handle: i32, offset: i32) -> i32`（`cap: platform.mem`）

## 2. 受检资源账与跨端错误语义

- **统一账本**：AV（`PlatformResourceLedger`）与 Native Win64 shim（`mn-platform-shim` BSS `mn_pio_state`）
  统一跟踪两类来源迥异的句柄：`StdStream` 与 `OsMemRegion`，并编码 1-based 带代际（generation）句柄
  （最大 64 槽位、单次 `write_some` 上限 65536 字节、单运行总写入上限 1048576 字节、每 run 分配总上限 64 MiB）。
- **前置受检拒绝（OS dispatch = 0）**：
  - `ERR_UNAUTHORIZED = -1`（以及运行前由宿主 `authorize` 抛出的 `host.unauthorized`）
  - `ERR_INVALID_HANDLE = -2`（`<= 0`、超槽位或代际不匹配）
  - `ERR_WRONG_ORIGIN = -3`（标准流句柄误用于内存操作或反之）
  - `ERR_INVALID_BOUNDS = -4`（零/负长度、越过逻辑长度的偏移、或无匹配借用可还）
  - `ERR_BORROWED = -5`（`borrow_count > 0` 时调用 `mem_free` 在触发 OS 前拒绝）
  - `ERR_ALREADY_RELEASED = -6`（已释放区域再次访问或释放）
  - `ERR_INVALID_VALUE = -7`（`mem_write_u8` 字节值超出 `0..=255`）
  - `ERR_OS_FAILED = -8`（真实 OS 返回失败，`last_os_error` 保存 Win32 `GetLastError()` 原值；若 `VirtualFree` 失败，保留原 `owner` 与区域槽位可供继续安全读写或重试，并在 run 结束清理）
  - `ERR_BUDGET = -9`（槽位或字节/内存预算超限）

## 3. Windows x64 原生底座与依赖分级

- **Native PE（AC/mn）**：`experimental/ace/crates/native/src/platform_io.rs` 在 `MirInstr::PlatformCall`
  存在时由 `console_native_build` 注入 `mn-platform-shim`（纯 Win64 FASTCALL 机械码生成 COFF）；
  仅从 `KERNEL32.dll` 静态导入 `GetStdHandle`、`WriteFile`、`GetLastError`、`GetSystemInfo`、`VirtualAlloc`、`VirtualFree`
  （外加 `console.rs` 的 `ExitProcess`），不调用 CRT、libc、`malloc`/`free` 或 Rust `std::io`。
- **PE 审计门**：`audit_pe_imports(&[u8])` 解析 PE64 导入表并验证 `ALLOWED_KERNEL32_IMPORTS` 白名单与禁令列表
  （`ucrtbase`/`msvcrt`/`vcruntime`/`api-ms-win-crt-*` 与 `printf`/`malloc`/`free` 等），由 `pio04` 正反控钉死。
- **AV 宿主**：AV 侧真实 Windows backend（`Win32PlatformIo`）同样直连 Win32 `kernel32.dll` 入口且可由测试捕获/故障注入
  （`CapturePlatformIo`）；AV 宿主自身仍作为 Rust 二进制运行，不宣称整个 AV VM 过程无 CRT。

## 4. 内部标准库区域包装边界

- 内部模块 `stdlib/ace/internal/platform_mem.at` 提供非公开（`fn`，非 `pub fn`）的 `io_write_all` 与 `mem_region_*` 封装，
  供下游 `946`/`972` 等自有容器与区域算法消费；不导出公共符号，也不写入 `stdlib/contracts/compatibility.toml`。
- **未交付 / 待后继**：Linux ELF/系统调用归 `PLAN-1040`，Unicode/宽字符控制台归 `PLAN-1041`，
  完整 File/Network 公共库归 `PLAN-973`/`974`，Rust/C host provider（A2R/A2C）维持缺口单列，公共 `Storage` API 维持未冻结。
