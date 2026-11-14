# auto-ac Linux x64 原生平台：ELF 启动、直接系统调用与 OS 区域来源（PLAN-1040）

> Status: active（2026-11-14 PLAN-1040 交付）。同一普通 Auto 源经共享前门/MIR 可在
> Linux x86-64 生成静态 ELF（自有入口 + 直接内核调用）并在真实 Linux 内核环境执行；
> Windows 原生路径与合同不变。本稿为当前事实与约束的规范源，验收环境证据见
> 归档 PLAN-1040。

## 1. 交付面（终态）

- **目标**：`x86_64-unknown-linux-gnu`（System V 调用约定载体）为第二个正式 mn 目标；
  身份轴（triple/ABI/runtime）与 Windows 目标互不共享（`native/src/mn_target.rs`、
  `config.rs`）。Windows 车道冻结不变。
- **产物**：静态 ELF64 ET_EXEC——无 `PT_INTERP`、无 `PT_DYNAMIC`/`DT_NEEDED`、无未解析
  符号（`platform_io_linux.rs::audit_elf_dependencies` 拒绝三者；链接失败本身是未定义
  外部的第一证人）。
- **入口/退出**：原始机器码 `_start` 三件套（`linux_support.rs`）：栈 16 对齐、内核栈
  auxv 走读取 `AT_PAGESZ` 入 `mn_pagesz`、`call ac_start`；退出走 `mn_exit` =
  `exit_group` 裸 syscall（编号 231），陷阱出口 70 不变。`mn_syscall1..6` 为 System V
  入、内核 ABI 出（rax=nr；rdi/rsi/rdx/r10/r8/r9；rcx/r11 破坏）的短机器码 stub——
  编译器生成并装入目标对象，不构成第二语义管线。
- **链接**：同一 rust-lld 二进制以 `-flavor ld` GNU 风格链接；无 SDK/导入库/CRT 启动
  文件；stage/backup/回滚纪律与 Windows `link_only` 相同（`link_driver.rs::link_elf`）。
- **缓存**：A09-ELF 层（`A09E`，schema `ac1040/a09-elf/v1`，carrier `mn-elf 1`）与 COFF
  层键空间隔离；恢复按头轴等价 + sha/len 绑定 + ELF 结构准入
  （`av::elf_admission::check_elf_relocatable`——COFF 字节进 ELF 槽按 magic 拒绝，反向亦然）。

## 2. OS 边界原语（1039 目录的 Linux 腿）

- **AV 宿主**（`av-core::platform_io::LinuxPlatformIo`）：stable `core::arch::asm!` 裸
  syscall（write/mmap/munmap/openat/close），`EINTR` 有界重试且每次重查取消；页大小取自
  本进程 auxv（裸 syscall 开读 `/proc/self/auxv`，OnceLock 缓存）——零 libc `syscall()`、
  零 `std::fs/std::io`、零 errno TLS。`virtual_free` 合同携带精确 mapped 长度
  （`munmap(base,len)` 无"整区域"形式；Win32 实现忽略该参保持原语义）。
- **区域来源**：匿名私有 RW `mmap`（fd=-1/offset=0/无 MAP_FIXED）与 `munmap`；
  粒度=页大小；账本/代际/借用/预算语义与 Windows 腿共享同一 ledger 代码。
- **AC shim**（`native::platform_io_linux::emit_platform_shim_elf`）：同一
  `mn_pio_state` 槽位布局与负错误码；OS 边界经 `mn_syscall*`；页大小读 `mn_pagesz`。

## 3. 内部探针缝（platform.probe/1，非公共 API）

- 动机：以普通 Auto 源见证裸 syscall 编组（第 4 参区分性：`openat` 创建 mode 落盘、
  errno 区间 `-4095..=-1` 原码分类），不向 guest 开放任意 syscall。
- 操作：`probe_openat`（仅 dirfd=`AT_FDCWD`(-100)）、`probe_write_fd`（fd>0）、
  `probe_close`（fd>2，永不关 std）。返回**裸结果**（正 fd/计数，或负 errno 原码）。
- 预检（AV dispatch 与 AC shim 双侧一致，含 flags==0 / mode<0 → `ERR_INVALID_VALUE`）：
  dirfd、路径界（≤4096、拒内嵌 NUL）、每次/累计字节预算。路径 NUL 终止只在 OS 边界。
- 边界：完整公共 File/path 合同归 973；本缝不注册公共 provider，不冻结 Storage。

## 4. 验收环境（WSL2，2026-11-14 用户约束）

- 当前唯一 Linux 验收渠道为 **WSL2**（T-01 冻结：Ubuntu-22.04、kernel
  6.18.58.1-microsoft-standard-WSL2、x86_64；`wsl.exe --exec` 传播子退出码、`wslpath`
  运行时解析、`/mnt/*` 为 9p——DrvFs 语义不外推完整 POSIX）。
- **AC 执行**：测试内 `wsl.exe --distribution <显式发行版> --exec`；目标进程经
  `/bin/sh -c 'echo $$ > pid; exec ...'` 定点记录；截止时杀 wsl.exe 侧后按记录 PID
  精确清理（绝不 `wsl --shutdown`/终止整发行版）；stdout/stderr 原始字节保真（UTF-8/NUL）。
- **AV Linux**：静态 musl 二进制 `mn-linux-av-runner`（`x86_64-unknown-linux-musl`，
  rust-lld 链接，无外部 Linux 工具链）在 WSL 内重验证 console ABC 后以真 Linux 后端执行；
  入口结果经 summary 文件回传（runner exit 0 = infra OK，guest 失败不与 runner 混淆）。
- **回归**：Windows 腿独立通过；构建 host 与执行 target 分账。证据限定该 WSL2 环境，
  不冒裸机/全部发行版/GUI/性能认证；WSL1 不可替代（其时按 Infra/pending 记录）。

## 5. 边界与未交项

- console 车道 ELF 源含浮点 `%`（fmod CRT 引用）时链接失败关闭——不支持而非静默；
  完整跨平台 runtime/线程/信号/动态加载非本包目标。
- AV 宿主进程本身可为 musl/glibc 程序（依赖分账第③级）；guest 平台原语链零 libc。
- `os-linux` platform adapter 已入 provider 选择（AV/Native、Linux target 五记录）；
  macos 等仍 fail-closed `assembly.no-provider`。
