# ACE 原生平台 I/O：Auto 自有标准库与 Windows/Linux 系统边界

> 2026-11-14 · Draft 0.5，调研后设计稿。新增公共标准库能力的六路径/原A2C覆盖见[单AV多provider设计](backend-provider-coverage.md)；本稿原生平台四格是其底座子集。补正原生底座的强制依赖，并把OS内存来源纳入1039/1040；纯算法可以提前开发，默认自有标准库完成必须消费底座真实交付。用户授权调研/设计/计划修订，本轮不实施产品代码、不生成探针/SDK、不运行产品、不提交或发布。已归档计划不重开。
> 衔接 [Design54](../54-stdlib-semantics-and-ace-integration.md)、[Design56 §0.2](../56-ace-m2-ffi-and-native-stdlib.md)、[Storage 专题](../memory/storage-and-allocation.md)；当前实现以 Specs/代码为准，本稿描述待实现目标。

## 1. 决定与支持范围

采用“普通 Auto 算法 → 受检平台 I/O 操作 → OS 入口”的分层。默认基础库不得委托 libc、Rust std::fs/std::io 或以第三方 shim 隐藏同类委托；显式 C/Rust 外库继续由967提供，不因本设计取消。

| 层 | 责任 | 不属于该层的工作 |
|---|---|---|
| 公共 Auto 库 | 格式化、UTF 编解码、write_all/read_exact、缓冲、文本行、错误及 owner 管理；普通源码参与现有统一前门 | 不复制 AV/AC 两份算法，不逐函数新增 MIR 指令 |
| 平台薄适配 | 平台标志、路径、整数宽度/结构布局、错误码、标准流与文件资源映射 | 不把 libc 封装当自有实现 |
| 执行边界 | AV 受授权宿主入口；AC 机器码/受检 OS ABI 调用 | 不向不受信 AV 源暴露任意 syscall 编号或进程地址 |
| OS | Linux 内核 ABI；Windows 公开系统 DLL API | OS 内部实现语言不成为 Auto 标准库实现语言要求 |

本专题原生底座 required 平台格为 Windows x86-64/AV-native、Windows x86-64/AC、Linux x86-64/AV-native、Linux x86-64/AC。公共print/File等能力还须在同一个AV的C/Rust来源及A2R/A2PY/原A2C真实可用；四格不是全标准库覆盖分母。Windows 基线沿现有正式 Win x64，Linux 用户态拟基线 Ubuntu 22.04 x86-64；用户2026-11-14明确当前仅能通过WSL验证，1040以Windows宿主＋WSL2 Linux x86-64为当前required执行环境。T-01核实际WSL版本/发行版/kernel/工具/挂载与权限，不假设Ubuntu22.04已安装，不用WSL1转换层或Windows互操作.exe代Linux AV/AC。没有 runner 的格保持 pending，不能以目标选择或交叉编译冒实际运行。Linux arm64/DGX、其他 OS 与完整 Linux runtime 移植另按后继合同，不从该四格外推。

## 2. Zig/Go 一手源码核验

固定参考 Zig **0.15.1**、Go **go1.25.0**，均是明确的历史版本样本，不宣称最新版本或全部平台都无 libc。经官方 GitHub 文件接口实读，下面 SHA 为 **Git blob SHA**，不是文件 SHA256。仅概述机制，不复制上游实现。

| 来源 | 实读文件/符号 | 观察与借鉴 | blob SHA |
|---|---|---|---|
| [Zig Linux CPU 原语](https://github.com/ziglang/zig/blob/0.15.1/lib/std/os/linux/x86_64.zig) | syscall0..6 | 内联汇编发 syscall；第4参数 r10，声明 rcx/r11/memory 破坏 | cf9714dd13ff90d6ae19fb63e9cb672de178132b |
| [Zig Linux 包装](https://github.com/ziglang/zig/blob/0.15.1/lib/std/os/linux.zig) | write/openat | 分别进 syscall3/syscall4，不必经过 C 函数 | 1aa9ea28522bebf6b54008beecbc5038eb15b1b7 |
| [Zig POSIX 分层](https://github.com/ziglang/zig/blob/0.15.1/lib/std/posix.zig) | use_libc/system/write | Linux 可按 link_libc 选择 libc 或自身 Linux 层；Windows write 有单独分支，不能只看 use_libc 就推断实际依赖 | 90ab157f0c207f914d9558a3b929ee906e5710c7 |
| [Zig Windows 层](https://github.com/ziglang/zig/blob/0.15.1/lib/std/os/windows.zig) | OpenFile/WriteFile | OpenFile 使用 ntdll.NtCreateFile；写入使用 kernel32.WriteFile，非“一律公开 Win32” | 66583d21b21809f161e03a006b5402bedebba996 |
| [Zig 文件层](https://github.com/ziglang/zig/blob/0.15.1/lib/std/fs/File.zig) | write/writeAll | 平台分派与循环写入；这条 write 本身未发现 WriteConsoleW，不能说它与 Go 的终端编码路径完全相同 | 4465701f1ab51476a67ed6ae7f8f76db493f0ca6 |
| [Go Linux 汇编](https://github.com/golang/go/blob/go1.25.0/src/internal/runtime/syscall/asm_linux_amd64.s) | Syscall6 | 参数寄存器转换→SYSCALL→错误码解码 | 3740ef1beb570d12194e31ba7db245a7333fda51 |
| [Go Linux 包装](https://github.com/golang/go/blob/go1.25.0/src/syscall/syscall_linux.go) | Syscall/Open | 调用前后通知 runtime；Open 走 openat。阻塞管理不能只抄机器指令 | d733ca9bf9230f92eb4d440cbe9eed0adc6cc718 |
| [Go Windows 包装](https://github.com/golang/go/blob/go1.25.0/src/syscall/syscall_windows.go) | CreateFileW/ReadFile/WriteFile/GetStdHandle | 通过系统 DLL 入口及目标 ABI，非 C stdio | 01c039cf2870476efc53554d561e9b09cc988526 |
| [Go Windows I/O](https://github.com/golang/go/blob/go1.25.0/src/internal/poll/fd_windows.go) | FD.Write/writeConsole | 文件/管道走 WriteFile；控制台进行 UTF-8 解码、UTF-16 编码、分块及循环写入 | acc2ab0c6e1eaf471f8aee67f5fe6eee8d1d0c68 |

结论：共同机制是自有库算法加平台适配，不是每个平台都使用同一种裸 syscall。Auto 采用这一分层；Windows 首版选择公开 Win32 API 而不照抄 Zig 的 NtCreateFile 路线。微软的[内部接口说明](https://learn.microsoft.com/en-us/windows/win32/devnotes/calling-internal-apis)建议优先等价公开接口；[NtCreateFile](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntcreatefile)本身有文档，不能笼统称所有 Nt 接口都未公开。以后确需目录句柄相对打开等能力时单独证明必要性/兼容性，不自行固定 Windows 内核调用编号。

Zig 可选链接 libc，Go 的 cgo/部分名字解析配置也可引入 C resolver（[Go net 文档](https://pkg.go.dev/net#hdr-Name_Resolution)）；本稿只从上述真实 I/O 路径推论，不宣称两门语言的所有程序无 libc。

## 3. Linux 直接系统调用与 AC 产物

初批 x86-64 Linux 的适配由目标内核 ABI 决定：syscall 编号/返回使用 rax，参数使用 rdi/rsi/rdx/r10/r8/r9；rcx/r11 和内存效果按实际调用处理。不能用普通 System V 函数第4参数寄存器替代 r10。arm64 需要另适配，不共用编号表。

Auto 编译器内部提供受限机器原语，平台模块用普通 Auto 包装；不预定公开 #syscall 语法。Cranelift 不被假定具有通用内联汇编能力：1040/T-01选择可验证的短机器码 stub/目标对象发射办法，仍消费共享 Checked HIR/MIR，不建立第二语义管线，不转译 C/Rust 业务源。

直接返回值按具体操作解码；x86-64 本批 I/O 的 -4095..-1 是错误区间，不把所有高位地址都当错。Auto 错误值携带 Linux 原始码，不调用 libc syscall()、不依赖 errno TLS。参见 [syscall(2)](https://man7.org/linux/man-pages/man2/syscall.2.html)、[intro(2)](https://man7.org/linux/man-pages/man2/intro.2.html)。

当前 native 生产链是 COFF→PE（native/src/emit.rs、link_driver.rs），Linux 不能只加 provider 名字。新增1040交付 x86-64 ELF 对象、最小自有入口与退出、无 CRT 链接、Linux 实际执行/捕获与目标身份。固定缓冲小程序可先无堆运行；不因这条链通过就宣布 text/collections 等所有 Rust runtime 已移植。

最小 ELF 无 PT_INTERP、无 DT_NEEDED/libc、无未解决 C 支撑符号；工具链可以由 Rust/C 实现，生成程序不需要这些开发工具。memcpy/memset/除法等潜在编译器支撑引用必须审计并以 Auto/明确原语提供，不能暗补 compiler-rt/libc 过门。

## 4. Windows 系统 API 与启动

通过版本化目标签名/布局和 Win64 ABI 调用系统 DLL：公开入口可用 PE 导入/IAT，不必运行 C 编译器或加载 C stdio。系统导入库是链接描述，不等于链接 CRT。宿主构建工具的依赖与最终 PE 的依赖分别记录。

基础入口为 GetStdHandle、GetConsoleMode、WriteConsoleW、CreateFileW、ReadFile、WriteFile、SetFilePointerEx、GetFileSizeEx、FlushFileBuffers、CloseHandle、GetLastError；进程由自有入口进入并显式 ExitProcess。保 x64 栈对齐、shadow space、宽度/返回值/失败哨兵规则，不能用 AutoABI 代替 Windows ABI。

CreateFileW 失败是 INVALID_HANDLE_VALUE；GetStdHandle 可为 NULL 或无效值；不能统一以零为所有 API 的失败值。按各 API 条件立即保存 GetLastError，不能调用日志/清理后再读。无 CRT 入口与导入闭包参见微软[/NODEFAULTLIB](https://learn.microsoft.com/en-us/cpp/build/reference/nodefaultlib-ignore-libraries)、[CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)。

## 5. AV/AC 共用算法与安全边界

逻辑操作描述至少包含：操作身份/版本、目标 OS/arch/ABI、输入输出类型及长度单位、缓冲区借用/可写性、权限、资源 owner/generation、可能阻塞和错误/进度语义。沿947四轴选择/944装配身份、938/939资源与当前 host 验证机制扩展；不造另一套 loader/FFI registry。字段是设计要求，不冒已经存在的数据结构。

AV 从同一 Auto 库编成 ABC，边界只负责安全解引用与调用；AC 从共享 MIR 编成机器码，边界按已验证目标发射。AV 不接收任意宿主指针/内核编号，缓冲区在调用期间稳定并保活；异步则保到真实完成。资源凭据按会话/代际/权限验证，restore 不复用旧地址、句柄或 grant。关闭/取消、在途借用与最后 owner 的规则进入验证，不仅检查类型。

阻塞调用必须进入宿主可观察的阻塞/恢复边界，不能卡 UI 事件线程，也不能拿异步包装名冒真异步。首批同步基线由1039冻结 worker/调用线程合同；未来 reactor/任务唤醒继续归968，IOCP/io_uring/epoll等不在本稿一次性实现。

依赖声明分三级：①默认 I/O 调用链无 libc/Rust std 委托；②独立 AC 可执行程序无 libc/CRT；③整个 AV/GUI 宿主或 runtime 自举。①②为本专题验收，③不是其推论。AV 宿主今天由 Rust 构建及可能含 libc 的事实必须保留；平台 primitive shim 可作为迁移缝明确存在，不许把格式化/文件缓冲偷偷放回 Rust std。

## 6. 输出和 Unicode

Linux 对 stdout/stderr 的目标 fd 写字节，内核不替终端解释 UTF-8；借用标准流，不创建/关闭调用者流。标准流缺失/失效受控报错。

Windows 验有效句柄及控制台类别：文本控制台用 Auto UTF-8→UTF-16 转换后 WriteConsoleW，文件/管道用原 UTF-8 字节 WriteFile，不修改共享控制台全局 code page。GetConsoleMode 失败不能吞掉无效句柄/权限等真实错误。控制台的计数单位为 UTF-16 code unit，公共输入进度单位为 UTF-8 byte；需维护块边界映射，避免半个多字节字符/代理对与重试重复输出。这里的分块按有效文本边界进行，容量是内部策略，不新增语言文本长度上限。

分离原始 bytes sink 与有编码的 text console；任意二进制对控制台如何处理必须显式拒绝/选择原始模式，不能静默有损解码。公共 Str/str/bytes 保长度语义，空串、多字节、ab\0cd 五字节不因内部 strlen 截断。文件内容不自动转 UTF-16、CRLF 或增加 BOM。旧 print/say 换行/预算行为按943/947保留，必要新增 text-console 入口在1041/T-01命名并对账，不静默改变旧 byte oracle。

“分块不拆代理对”约束我们提交的块，不承诺OS的每次部分写也停在字符边界。若OS只完成代理对的一部分，内部保存已写UTF16单元与剩余转换状态，只把完整输入标量计入已确认UTF8进度；重试续剩余单元，错误保留单位级部分效果信息，不能按逻辑字节计数盲重发整个字符。1041/T-01须冻结可观察的错误/续写表示，OUT-03包含此反例。

参考 [WriteConsole](https://learn.microsoft.com/en-us/windows/console/writeconsole)、[ReadFile/WriteFile 文件行为](https://learn.microsoft.com/en-us/windows/win32/fileio/reading-from-and-writing-to-files)。

## 7. 基础文件 I/O 与路径

| 行为 | Linux 直接接口 | Windows 系统接口 | 公共要求 |
|---|---|---|---|
| 打开/创建 | openat，flags 与 mode 分开 | CreateFileW，access/share/disposition 分开 | read/write/create/truncate/exclusive/append不是同一开关；保旧 API 语义 |
| 读取 | read | ReadFile | 实际字节数；非空普通文件读取返回0才是EOF；零容量读取不推EOF |
| 写入 | write | WriteFile | 返回实际进度；write_all循环，成功0且仍有数据时受控 WriteZero，不死循环 |
| 定位 | lseek | SetFilePointerEx | 64位位置；共享游标并发受管，不把管道当可seek文件 |
| 大小 | 目标布局的 fstat 等 | GetFileSizeEx | 不从先读固定大小推断文件不会增长/缩小 |
| 同步 | fsync/fdatasync | FlushFileBuffers | Auto缓冲flush与持久化分开；设备/FS差异如实保留 |
| 关闭 | close | CloseHandle | owner显式close可报告错误；析构清理与可报告错误分开 |

Linux 创建 mode 经 umask；默认加 O_CLOEXEC，Windows 默认不继承并明确共享/删除共享策略，须与943旧行为核对后落实。append 使用 OS 的追加语义，不能靠用户态 seek-end+write 冒原子追加；并发记录整体原子性不作普遍承诺。

Path 与普通 Str 分开：Linux 内核路径是排除 NUL 的原始字节，Windows native path 是 UTF-16 单元。普通 UTF-8 构造在 Windows 转换，并仅在 API 边界加终止符；原生非UTF8/非标量路径需可无损保存/往返，显示有损须显式。不能全局字符替换分隔符或据大小写规则擅自归一。长路径、设备名/UNC、symlink/reparse point由973/T-01核 API/权限支持，未覆盖项不冒全路径安全。

路径授权不能只做字符串前缀检查；授权根与打开后的对象绑定、防 symlink/重解析逃逸，具体平台可行机制须973/T-01先证，做不到的受限授权模式明确拒绝，不能把普通 CreateFileW 冒 sandbox。完整原生目录句柄方案若确需 Nt 接口按§2另修约。

## 8. 错误、部分完成和取消

| 情况 | 必须行为 |
|---|---|
| read/write EINTR | 依真实进度及取消检查有界重试，不无条件重启整个写入 |
| EAGAIN/WouldBlock | 返回或交现有调度/等待机制，禁止忙轮询 |
| Linux close 报错/EINTR | 不盲目重试已可能释放的 fd；保关闭错误和owner状态，避免关闭复用资源 |
| SIGPIPE/EPIPE | 独立 CLI 与嵌入宿主分别冻结策略，库不擅改全进程信号；受控错误合同需实际设置/验证 |
| Windows 同步/异步 | ERROR_IO_PENDING不是普通失败；未交异步时不选 overlapped 模式；buffer在完成前保活 |
| 写一部分后失败 | 返回可解释的进度与错误，不声称输出回滚；授权/预算预检失败仍应 dispatch=0 |
| 部分写后取消 | 保留已经产生的外部效果；取消不等于撤销或立刻终止 OS 调用 |
| write/close 成功 | 不自动宣称断电持久化；新建/重命名的目录持久化另处理 |

947的“单次输出原子提交”保留为其受限预算/预检合同和原样本，不把它扩大为所有设备任意长度的事务；新的通用 I/O 合同在1041补部分完成证据，不能回写归档947为新功能已实现。参见 [write(2)](https://man7.org/linux/man-pages/man2/write.2.html)、[close(2)](https://man7.org/linux/man-pages/man2/close.2.html)、[pipe(7)](https://man7.org/linux/man-pages/man7/pipe.7.html)、[fsync(2)](https://man7.org/linux/man-pages/man2/fsync.2.html)。

## 9. 所有权与内存

read_into/write_from 使用调用者缓冲区，最小输出/文件往返无需通用堆；UTF转换可用固定块。read_all、长行、动态format才消费Storage。无libc、无通用堆、完全无运行期分配分别验，不因固定外层容器就宣称所有临时值无堆。Storage公共签名不由本专题预定，沿内存专题/972讨论；缓冲区借用、失败保活、最后owner与错误进度必须明确；OS内存来源新增要求见§9.1，仍不在此冻结Storage策略API。

### 9.1 OS内存来源也是原生底座（r0.2）

仅有write/文件原语不能支撑动态Str/List/HashMap。1039交付受检页/区域取得、借用与释放合同及Windows的VirtualAlloc/VirtualFree接入；1040交付Linux匿名mmap/munmap直接调用与页大小来源。这是内部原语，不冻结Storage/allocator公共签名、不把分配策略塞进编译器。

| 层 | owner | 原生完成要求 |
|---|---|---|
| 页/区域来源 | 1039 Windows；1040 Linux | 真实OS取得/释放，不用malloc/free或Rust全局allocator提供guest默认内存 |
| Auto分配/容器策略 | 946有限原生集合；972完整Storage/alloc | 普通Auto实现已承诺算法；消费受检区域/调用者Storage，不依赖Rust Vec/String/HashMap算法冒自有实现 |
| 固定buffer/core算法 | 945既有切片及972 core | 可在无OS/无堆profile先写先测；不能据此关闭整库原生平台义务 |

Windows首批使用VirtualAlloc(NULL,size,MEM_RESERVE|MEM_COMMIT,PAGE_READWRITE)，释放用VirtualFree(base,0,MEM_RELEASE)，不混淆保留粒度与页大小。Linux首批用RW匿名私有映射、无MAP_FIXED、fd=-1/offset=0，调用自身内核原语而非C mmap()；按目标内核ABI解码返回值，不把合法指针简单按有符号负数分类。页大小用OS信息/启动auxv等真实来源，不能调用libc sysconf或跨架构硬编码。

区域记录至少保留逻辑长度/实际映射范围、来源、会话/代际、在途借用与释放状态；零长/溢出请求受控拒绝，释放只接受本记录的整个owned区域，错base/错来源/重复释放/有借用释放在OS前拒绝。OS失败/受管资源不足不能发布半初始化owner；失败后合法分配/释放应继续可用。取消/关闭清理沿既有owner资源合同，不能序列化恢复旧地址。

依据：[VirtualAlloc](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualalloc)、[VirtualFree](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualfree)、[mmap/munmap](https://man7.org/linux/man-pages/man2/mmap.2.html)。具体Auto Storage接口仍由内存专题及972/T-01具名讨论，1039/1040不能反等972整库。

资源 close、move、重复close/关闭后使用、跨会话/错代际分别有受控结果；不把 fd/HANDLE 直接暴露为通用整数假装所有权已解决。OS调用前后的拒绝与操作已经发生后的失败分别取证。

## 10. 计划承接与顺序

| owner | 本次安排 | 下游与边界 |
|---|---|---|
| 943/944/947 | 已归档，保持历史与现有有限能力，不重开 | 947现有Windows console与选择合同是起点，不证明Linux/全File已交 |
| [1039](../../plans/1039-ace-platform-io-primitives.md) | 受检平台I/O/内存区域原语、AV缓冲/授权/资源边界、Win64接入 | r2，9任务；不等待完整967/968/972，不接管Storage策略/FFI/调度 |
| [1040](../../plans/1040-ace-linux-native-elf-and-syscalls.md) | Linux x64 ELF/入口/退出/syscall、匿名内存区域与AV Linux薄适配 | r2，9任务；依1039，不声称全部runtime跨平台 |
| [1041](../../plans/1041-ace-native-stdio-and-console-unicode.md) | 补947未交的跨平台stdio、Windows文本控制台与重定向/部分完成 | 依1039/1040与945所需文本能力；保947旧源与oracle，形成独立新增义务 |
| 946 | 纯算法可先准备；拥有资源的原生实施/验收消费1039/1040 | 新增整包底座依赖；不依972整库，不以临时Rust容器冒原生交付 |
| 948 | 调查不等待底座；完整交接消费946等正式来源及其新增底座证据 | 盘点/设计与原生实施完成分别记账 |
| 972 | Auto格式化/编码/缓冲/Storage；整包依赖1039/1040/1041 | core纯算法可先验证，但原生alloc/系统接线及整包完成须底座正式交付；不反等973 |
| 973 | 消费三包，落实完整File/path/env/process/time/rand等原职责及本稿系统文件合同 | 新增公共能力六路径及原A2C；本专题不能代整973 |
| 978 / 985 | 候选包依赖审计、真实四格认证；986/987继续原安装退役门 | 早期B0/Hello不被新增Linux整包反向阻塞；完整M2-A须收应有新义务 |

新三包均drafting、未实施；设计/计划更新不自动进入work。缺Linux runner先列阻塞和可做的目标/Windows工作，不以未知状态充PASS。首步各T-01是有界全链可行性与输入冻结，不允许先实现一大批stub再用自报支持表过关。

### 10.1 依赖的强制含义（r0.2，取代此前可并行表述的歧义）

原生实施依赖不是仅有声明/接口即满足：必须取得对应操作的实现、实际OS运行、依赖来源与独立复审的正式交付。拟议provider、typed fixture、Windows-only样本、暂用旧Rust/C runtime均不能关闭相应原生子行。即使旧runtime为no_std/无libc，也不自动证明算法已由Auto实现。

执行顺序为1039→1040，再并行推进946拥有资源原生切片与1041；946→948正式交接，946/948连同语言前置与1041汇合到972完整库；再由原有语言/FFI/调度前置与三包汇合到973→974/975→完整SDK/认证。974/975通过973、978/985通过972/973已有传递依赖，无须机械重复所有边。967显式外部FFI、948前期调查、core算法准备以及UI早期F0/H0/E0/B0不因这条原生库链全面停工；它们的通过不冒原生库完成。

### 10.2 加载/#in底座与OS原语分别交付（2026-11-13当前）

[1043](../../plans/archive/1043-ace-stdlib-context-assembly.md)是新加载机制的下一实施包，负责公共+at/c/rs/py来源装配、#in namespace/阶段继承、目标条件导入及query/恢复；设计单源为[覆盖/加载§8–10](backend-provider-coverage.md)。1039/1040仍独立提供OS/内存原语，可用现正式显式target路径推进，不反等完整1043/966，也不以此签#in已支持。1043不实现syscall或交无libc File；公共能力1041/946/972/973最终汇合1043与本稿原生底座，另交真实C/Rust/Python运行证据。完整CTE由966消费1043继续，1042先交Python最小producer，避免平台底座/库/加载器互等。

### 10.3 当前Linux验证渠道仅为WSL（2026-11-14）

[1040 r5](../../plans/1040-ace-linux-native-elf-and-syscalls.md)明确采用WSL2实际Linux内核环境验收x64 ELF/raw syscall/内存区域及Linux AV同源操作，先核WSL/发行版/kernel/arch/工具/文件系统与执行权限。Windows可交叉构建，但Linux AV/AC结果必须在WSL内真实取得，Windows回归独立运行；子终态、原始bytes、取消/reader/资源回收及host/target依赖沿原门。证据限定该WSL2环境，不冒裸机/全部发行版/GUI或硬件性能认证。缺WSL/权限或仅WSL1保持Infra/pending，不假设额外Linux runner，不自动安装/升级或终止用户整个发行版。相关基础事实见[Microsoft WSL架构比较](https://learn.microsoft.com/en-us/windows/wsl/compare-versions)，命令与材料路径规则以1040 §5为准。本次只改设计/计划，未探测或启动WSL。

## 11. 实施验收的最低证据

1. 同一公共Auto源码在四个required格分别真实输出、重定向与文件往返；中文/emoji/NUL字节、空输入、大于转换块输入各有独立expected。AV宿主捕获不能代AC运行或真实Windows控制台。
2. 独立AC产物检查 PE imports 或 ELF headers/dynamic/undefined symbols，加真实运行调用/依赖观察；反控故意引入libc委托/错误目标provider/宿主文件访问绕过必须被抓住。静态无导入不单独证明调用链与算法语言。
3. OS成功/部分完成/错误/关闭/取消、错代际/无授权、目标变化后的恢复与再次调用各有区分性正反控；mock故障注入辅助，但至少真实终端、文件、管道由OS执行。
4. 沿既有TestCase/ACE worker和固定cargo组登记，缺Linux工具/终端/runner为pending。普通正确性组排除performance；不新造runner，不跑全仓/ace-perf作为本次规划门。
5. 现有旧API行为、scope、测试规模和M1/A/B来源门保留；新case不是已运行证据。摘要、实际命令/子终态/来源/输出路径进入各Plan，实施按唯一worktree和plan-artifacts政策收尾。

## 12. 未决与交接

设计已选定平台入口、分层、默认依赖、编码、进度和资源原则。局部机制由具名T-01先证：1039核现IR/资源/稳定buffer及Windows页区域接缝，1040核ELF/syscall/匿名区域与Linux runner，1041核旧bytes模式与新增text模式的公共拼写和进度，973核原生Path/共享/授权根可行性，972核Storage接口。这些决定在依赖任务施工前写回各Plan；若改变本稿公开语义/支持范围须显式修订，不能静默缩水。

本轮仅官方源码/文档与本仓静态核对，无最小无libc产物运行证据；不是独立实现review PASS。最终文件链接/任务/依赖预检结果记本次路线更新。
