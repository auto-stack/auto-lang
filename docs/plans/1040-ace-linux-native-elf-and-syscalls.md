---
plan_id: PLAN-1040
status: execution_done
feature_name: ACE Linux x64原生ELF启动与直接系统调用
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-14T10:30
plan_revision: 5
phase: M2-A
current_step: 6
total_steps: 9
depends_on: [PLAN-1039]
supersedes_spec_components: [docs/specs/auto-ac/native-backend-and-retirement.md]
new_spec_components: [docs/specs/auto-ac/linux-native-platform.md]
touched_goals: [GOAL-002]
affects: [stdlib, ace]
---

# [PLAN-1040] ACE Linux x64原生ELF启动与直接系统调用

## 0. 变更摘要

2026-11-14 work r6（执行轮）：T-01 环境冻结（WSL2 Ubuntu-22.04, kernel 6.18.58.1-microsoft-standard-WSL2, x86_64, GNU ld/readelf/gcc 11.4/python3.10, wsl.exe --exec 传播子退出码, wslpath -a 解析, /mnt/d 为 9p 挂载, rust-lld 跨链 ELF 成功并实跑）；T-02/T-03/T-09 实现完成——x86_64-unknown-linux-gnu 目标（SystemV）emit/console+platform ELF shim、原始机器码 mn-linux-support（_start auxv AT_PAGESZ→mn_pagesz、mn_syscall1..6、mn_exit=exit_group）、rust-lld -flavor ld 静态链接、A09-ELF 缓存层（mn-elf 1 carrier + ELF 结构准入）、LinuxPlatformIo（stable asm! 裸 syscall，无 libc/std::fs）、platform.probe/1 内部探针（openat/write_fd/close 裸 errno 分类）、musl 静态 mn-linux-av-runner、os-linux provider；ELF-01..05/V-MEM/V-WSL 用例（native tests/cases/linux_elf_1040.rs 7 例）全绿。T-05 门禁与复审进行中。

2026-11-14 r5当前：用户明确当前Linux只能通过WSL验证。将Windows宿主＋WSL2 Linux x86-64设为本包当前required验收环境；T-01核实际WSL/发行版/kernel/工具与路径，AV Linux和AC ELF必须在WSL内实跑。新增AC-WSL/V-WSL/SD-WSL，由原任务承接；原9任务、依赖/状态/进度/旧AC与用例不变。本轮仅修设计/计划，未探测或启动WSL、未编译/测试/建树。

2026-11-13 r4当前：新增1043加载/#in计划后的分工：本包与1043可并行，不新增对1043/完整966的整包前置；显式target原语接线不冒provider新后缀/#in已交。最小source装配/条件选择由1043交，完整CTE仍966；公共库owner最后汇合本底座与1043。 原状态/任务数/验收不变，仅规划。

2026-11-13 r3：明确本包是内部平台原语/目标基础设施，非新增公共库全覆盖的完成计划；公共消费者必须另闭AV三provider与AC/A2R/A2PY及原A2C。

2026-11-13 r2，drafting 0/9：用户指出自有原生库必须依赖原生底座；补OS页/区域内存来源及独立验收，新增T-09/AC-MEM/V-MEM/SD-MEM，旧8任务/AC/用例全保留。本轮只修订设计/计划，不实施、不冻结Storage公共API。下段r1为历史。

2026-11-13 r1，drafting 0/8，仅规划。[原生平台I/O设计](../design/stdlib/native-platform-io.md)§3确认现AC是COFF/PE，Linux库不能只靠新增provider；本包补最小ELF/入口/直接syscall与AV Linux接缝。1039正式交后实施，本轮无产物/探针/测试或工作树。

## 1. 目标

同一最小普通Auto源经共享前门/MIR生成可在Linux x86-64执行的ELF，以自有入口及直接内核write/exit等实现可观察结果；无libc/CRT/动态解释器需求；补Linux匿名mmap/munmap页区域原语，供946/972自有分配与容器算法消费。AV Linux用1039受检操作执行相同数据语义。Windows原PE支持和失败恢复不回归。

非目标：arm64、所有Linux runtime/线程/信号/动态加载、发行版/GUI、AC自举、完整File公共API或另造编译器/执行器。可用Rust/C实现构建工具和VM桥，不能让目标I/O调用链委托libc。

## 2. 架构方案

共享Checked HIR/MIR→按target选择对象发射/原语stub→ELF linker→自有_start及退出→Linux实际执行。目标身份、object格式、ABI、入口/runtime/stub/工具版本进入现缓存/收据，不沿用PE对象命中。复用native及av现有执行设施抽取必要平台腿，不建立第二套任务生命周期系统。

## 3. 技术栈

现Cranelift/object设施、x86-64 Linux内核ABI、ELF与明确链接工具（T-01核可用项，不默认安装）。后续唯一树`D:/autostack/.wt/lang-1040/auto-lang`，分支`plan-1040-dev`；材料`D:/autostack/plan-artifacts/plan-1040/`，未创建，开工登记/收尾沿AGENTS。

## 4. 需求分析与背景调查

用户2026-11-13授权调研及设计/计划；本包尚无实施指令。已读Spec总览与native-backend-and-retirement；现`experimental/ace/crates/native/src/{emit,entry,link_driver,cache,config}.rs`及`experimental/ace/crates/av/src/native_exec.rs`锚定COFF/PE和原执行资源管理。Linux目标可选不等Linux产物可跑。基面`cb1a185ead0270a87d9f179f109678b19db1ace9`，开工重核hash/正式接口。

外部：Zig0.15.1 syscall0..6和Go go1.25.0 Syscall6实际寄存器与错误区间已读，blob与链接见设计§2；不是Auto已验证证据。当前只有Windows宿主的WSL可用于Linux验证。Linux用户态拟基线仍为Ubuntu22.04 x64；T-01先记录现有WSL实际发行版、版本及内核，若与拟基线不同明确登记覆盖差额，不改标签冒22.04已测、不默认安装/升级/切换发行版。runner/工具/权限当前尚未探测，缺项不能PASS。

### T-01 冻结事实（2026-11-14 实测，本会话）

| 项 | 冻结值 |
|---|---|
| WSL | `wsl.exe --list --verbose`：唯一发行版 `Ubuntu-22.04`（默认，Stopped→启动），VERSION=2 |
| 内核/arch | `uname -m`=x86_64；`uname -r`=6.18.58.1-microsoft-standard-WSL2 |
| 发行版 | /etc/os-release：Ubuntu 22.04.5 LTS (jammy)——与拟基线一致，无差额 |
| 工具 | GNU ld 2.38、readelf/objdump/nm 2.38、file、gcc 11.4.0、python3 3.10.12；ld.lld/clang 缺（不安装） |
| 挂载 | /mnt/d = 9p(drvfs) rw；worktree 可达，DrvFs 语义不外推 POSIX（执行从 /mnt/d 直测可行） |
| 退出码 | `wsl.exe --exec /bin/sh -c 'exit 7'` → 7（子进程退出码传播） |
| 路径 | `wslpath -a 'D:utostack'` → `/mnt/d/autostack`（运行时解析，不硬编码） |
| 跨链 | WSL gcc 组装 probe.o → Windows rust-lld `-flavor ld --entry=_start` 链接 → WSL /mnt/d 实跑：write/exit_group 裸 syscall、字节精确、exit 42、无 PT_INTERP/动态段 |
| 构建 | AC 链 Windows 交叉（cranelift ELF 对象 + rust-lld）；AV runner `x86_64-unknown-linux-musl`（rust-lld 链接，无外部 Linux 工具链）；基面 worktree HEAD 10a3dd46e |
| 命令 | 测试内 `wsl.exe --distribution Ubuntu-22.04 --exec ...`；目标进程经 `/bin/sh -c 'echo $$ > pid; exec ...'` 定点清理（绝不用 wsl --shutdown） |

### 本轮静态取材身份（非运行证据）

| 本仓路径 | SHA256 |
|---|---|
| crates/auto-lang-core/src/stdlib_assembly/provider_selection.rs | 910485b716e9c59b05c828285ac1705f945e35df8826213820212445d349f52c |
| experimental/ace/crates/av-core/src/host.rs | 30d6dcc475850cd007f4032352fda01644e4ff6ccc012b13a34854867c1eb7d4 |
| experimental/ace/crates/av/src/console_host.rs | ce2788ea93e54a1e472672e7e4f0246d4645b69d2d3ee0144492289d0725d0a2 |
| experimental/ace/crates/native/src/console.rs | e72266430d5b3c4e3de7e951b3691f4a39ecc982463aaf3d6239ffc7b385ab1c |
| experimental/ace/crates/native/src/emit.rs | b3637913863bb71a9b54378bb3219bd642aafa935ecf4825f8f030aab51965ac |
| experimental/ace/crates/native/src/link_driver.rs | 8c8bc4ea4ef6a049372eb097b9052eabf8db9146cd36e2cb7bfc38773b1cfb19 |
| experimental/ace/crates/av/src/native_exec.rs | 6358d1a5199defb087a74eaccf5349b039488ed4adab65fcb6b2e5a650b5d2e6 |
| docs/specs/stdlib/design/ace-host-services.md | 64aaf698c23cf01bae4c696243d7f5631f7012384d456b483f6444c6cbf6ac91 |
| docs/specs/stdlib/design/ace-binding-and-assembly.md | 62fda8e6ea53dc770d44330b0440b58154063d17ad2abfde2284909d9018ff69 |
| docs/specs/auto-ac/native-backend-and-retirement.md | 77ba8a3c2fa19128b7bbc958d21c5d928de788157daa39b6978b9d8fb0731a78 |

## 5. 详细设计

T-01先验证Linux目标对象/链接/入口/执行完整最小通路的可行性，冻结命令、工具、运行环境和源能力；若发现需新公共语法/更大runtime，先修合同/依赖，不能用手写ELF hello替正常Auto输入。Cranelift通用inline asm不作前提；可由编译器生成短机器码stub并装入目标对象，调用语义仍由共享verify/MIR提供。

raw syscall x64参数rax/rdi/rsi/rdx/r10/r8/r9；rcx/r11/memory效果正确。编号/flags/结构来自目标UAPI，错误按具体接口处理；syscall()和errno C包装禁止。必须有第四参数区分性见证（例如最小openat创建mode及实际结果），不只用三参数write自证所有寄存器。文件原语在此为内部探针/接缝，完整公共File由973承担。

最小自有入口不依赖main CRT；正确堆栈/调用约定及显式进程退出；无堆固定buffer程序先证。目标ELF无PT_INTERP/DT_NEEDED/libc及隐藏C支撑符号；缓存/恢复不能接受Windows对象或错ABI。AV宿主依赖单列；Linux原语不使用std::fs/std::io或libc包装绕过。

### 当前验证环境：Windows＋WSL2（2026-11-14用户约束）

当前Linux验收只走WSL，不假设另有裸机Linux、远端runner或DGX可用。本包直接Linux内核调用的required环境为**WSL2 Linux x86-64**；用户提供的是WSL渠道，WSL版本/发行版/内核尚须T-01核实，不据本段推已装WSL2或已能执行。Microsoft一手说明：[WSL版本比较](https://learn.microsoft.com/en-us/windows/wsl/compare-versions)（WSL2使用Linux内核，WSL1为转换层）、[WSL基本命令](https://learn.microsoft.com/en-us/windows/wsl/basic-commands)。WSL1可另列兼容观察，不能替本包直接Linux内核验收；WSL无法启动/权限拒绝/只有WSL1或缺工具均记录Infra/pending，不假装Linux格通过。

T-01只做本包所需环境入场，记录`wsl.exe --list --verbose`及可用的`--status/--version`，显式选择实际发行版；再在其中核`uname -m`、`uname -r`、`/etc/os-release`和实际linker/ELF检查工具版本。命令语法按工具实存冻结，不编造已安装工具或执行结果。不依赖默认发行版、默认cwd或PATH偶然选中Windows工具，不自动安装/升级/转换WSL或更改其全局设置。历史会话的WSL E_ACCESSDENIED不当作当前结果，也不因用户明确可用渠道就跳过本会话权限核实。

| Linux验收腿 | WSL内必须取得的证据 | 不能替代的结果 |
|---|---|---|
| AC Linux ELF | 正常Auto源生成的真实ELF、独立stdout/stderr bytes/exit、write/openat/mmap/munmap等实际Linux调用与资源结果、ELF依赖检查 | 只交叉编译、手写ELF hello、Windows PE运行或脚本echo |
| AV Linux | 实际Linux AV进程/工件身份，同源受检操作在WSL的Linux内核执行及关闭/取消/区域生命周期 | Windows AV代理输出或WSL互操作启动Windows.exe冒Linux AV |
| Windows回归 | 原Windows原生入口、PE及相关完整职责组 | WSL中Linux成功不替Windows回归 |

构建可按T-01确定在Windows交叉生成或在WSL内进行，二者均须将Linux产物放到上述WSL2中真实运行；构建host与执行target分列。复用现worker/native_exec的受控平台腿，Windows侧可由`wsl.exe --distribution <实际发行版> --exec ...`显式启动，不另造测试框架。wrapper核实际Linux子项终态与stdout/stderr原始字节，保UTF8/NUL，不让PowerShell/文本管道转码或父exit0掩盖Linux失败；截止/取消/输出预算与reader收敛保原规则，清理只针对本测试所持进程，不能用wsl --shutdown或终止整个发行版作为日常收尾。

正式worktree仍为既定Windows路径，WSL实际挂载路径由wslpath/挂载信息核实，不硬编码/mnt/d成立，不建junction/symlink。材料仍统一`D:/autostack/plan-artifacts/plan-1040/`及其经核实的WSL可见路径，沿AGENTS登记/清理，不另建Linux证据副本/第二套长期缓存。权限、mode、可执行位等case核对所在文件系统语义；DrvFs结果不无条件外推完整POSIX语义，环境不满足时具名记录，不能修改oracle过门。最终证据标明WSL2/发行版/kernel/fs/工具版本，仅证明本Linux机制在该环境的行为，不冒所有发行版/裸机、硬件性能或GUI认证。AV宿主依赖与独立AC无libc继续分账。

### OS内存来源原语（r2，当前）

Linux x64使用匿名私有RW mmap与munmap直接内核入口，无MAP_FIXED，fd=-1/offset=0；禁止调用libc mmap()/malloc/sysconf。页大小通过最小启动auxv AT_PAGESZ或经证实的目标机制取得，该小入口读取纳入本包，其余完整argv/env/TLS仍非目标。复用1039区域身份/借用/释放合同；Linux内核允许的任意部分munmap不直接暴露为此owned-region接口。处理6参数ABI/页对齐与长度溢出、错误区间、失败不发布owner、borrow/错代际预拒，真实释放后合法再次分配。 来源与平台细节见[设计§9.1](../design/stdlib/native-platform-io.md#91-os内存来源也是原生底座r02)。

### 与公共标准库覆盖的边界（2026-11-13当前）

[覆盖设计](../design/stdlib/backend-provider-coverage.md)要求所有新增公共标准库能力六路径及原A2C。本包只交受检平台/内存原语和相应目标运行底座，Windows/Linux AV/native、AC原有required格不减；不要求Python发同形syscall/ELF，也不为其签已支持。本包不反向依赖完整967/1042/977，946/972–975/1041等实际新增公共操作的owner消费本底座后关闭全路径，不得用基础设施名称排除真实公共API。

### 新加载层边界（2026-11-13当前）

[加载设计§10](../design/stdlib/backend-provider-coverage.md)与[1043](archive/1043-ace-stdlib-context-assembly.md)：本包与1043可并行，不新增对1043/完整966的整包前置；显式target原语接线不冒provider新后缀/#in已交。最小source装配/条件选择由1043交，完整CTE仍966；公共库owner最后汇合本底座与1043。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-ac/linux-native-platform.md | 无真实Linux native交付→有限x64 ELF/入口/syscall/运行矩阵 | 原生库先有目标产物 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-ac/native-backend-and-retirement.md | 保共享MIR唯一源/Windows→加入Linux目标对象身份及现执行器接缝 | 不造第二lowering/错误缓存复用 | AC-02/03 |

### OS内存规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-MEM | add/modify | docs/specs/auto-ac/linux-native-platform.md | I/O-only原语→受检OS区域来源/借用/释放、真实依赖与失败状态 | 自有alloc前置 | AC-MEM |

### WSL验证环境规范增量（实际交付后沉淀）

| delta_id | add/modify | docs/specs target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-WSL | add/modify | docs/specs/auto-ac/linux-native-platform.md | 未指定Linux runner→实际已验Windows/WSL2版本、Linux AV/AC运行证据与适用边界 | 用户当前唯一Linux验证渠道 | AC-WSL |

## 6. 测试设计

T-02..04补原worker/TestCase适配和永久case。职责组候选：`cargo t ace-native ace-av ace-lowering ace-host ace-artifacts`；共享source/verify改则补`ace-source ace-semantics`。Windows runner的条件排除不计Linux运行；Linux runner按现固定组原预算/ignore，原性能不附加。拟新增ELF检查/执行命令由T-01按工具实存冻结，当前不编造已经存在CLI。

| Case / AC | 输入/真实入口 | 独立oracle与owner | 边界 |
|---|---|---|---|
| ELF-01 / AC-01/02 | 普通Auto源输出手定UTF8/NUL bytes和非零exit→真实ELF运行 | 实际stdout/stderr/exit；不是脚本echo；T-02/03 | Linux x64 AC |
| ELF-02 / AC-02 | syscall第四参数、合法值、无效fd/路径与合法相邻 | OS创建mode/实际文件状态及负错误分类正确；T-03/04 | 最小原语，非完整File库 |
| ELF-03 / AC-02/03 | Windows/Linux来回构建及restore、错target/ABI/stub/对象 | 错态调用/发布前拒绝，原WindowsPE/旧文件字节保全；修复后再restore可用；T-02/04 | 现缓存/发布合同 |
| ELF-04 / AC-03 | 静态依赖检查+无开发仓/工具运行；控制引入libc/错误格式 | ELF头/动态节/undefined符号与真实执行一致，控制红；T-04 | 独立目标，不冒整AV无libc |
| ELF-05 / AC-02/03 | Linux AV同源操作及取消/运行截止/输出上限 | 真OS bytes相符，owned子进程/reader收敛，缺工具单列Infra；T-03/04 | 同步原语与既有执行资源门 |

实现/独立review同树顺序执行，无另起agent/model指定；fmt/警告及任务子终态都核。不用emit-only/跨平台编译通过代真实Linux运行。

### OS内存验证（新增，未执行）

| Case | work owner / AC | 输入与实际oracle | 边界 |
|---|---|---|---|
| V-MEM | T-09/T-04/T-05；AC-MEM | Linux AV/AC普通Auto源匿名映射→写读手定bytes→解除映射；核6参数ABI/页大小来源、溢出/错误区间、OS失败/有借用/错来源与合法邻例。真实OS范围与计数及内容正确，预拒dispatch=0，释放后再分配可用；无libc mmap/malloc/sysconf委托。 | 平台OS区域服务，不冒通用allocator或全AV无libc |

### WSL入场及真实执行验证（新增、未执行）

| V / AC | 输入/入口 | 独立预期及owner |
|---|---|---|
| V-WSL / AC-WSL | 当前Windows→显式所选WSL发行版→ELF-01..05/V-MEM实际Linux AV与AC；负控WSL入口拒绝/只有WSL1、错发行版/arch、只emit、Windows.exe冒Linux AV、Linux子项非零而父wrapper成功、stdout含NUL及超时取消 | T-01核实际WSL2/kernel/x86-64/工具和映射；T-04/T-05核真正Linux子项bytes/exit/调用/资源及父侧终态传播，错误环境记Infra/pending，负控不能PASS；原Windows完整组另验，不将交叉选择或WSL可启动当Linux运行完成。 |

## 7. 验收标准

- [x] AC-01 / work：T-01冻结（§4表）；ELF-01真实ELF经共享前门→SystemV单元→rust-lld静态链→WSL2实跑（bytes/exit=42）＋依赖闭包审计通过；musl runner交叉构建可用（无外部Linux工具链）。evidence: linux_elf_1040.rs elf01/elf00 @088d331b1。
- [x] AC-02 / work：ELF-01/02/03/05全绿——openat第4参mode 0640落盘（stat独立oracle）、ENOENT/EBADF负errno分类＋flags=0/mode<0预检AV/AC双称（F2修复）、缓存跨target/污染/轴篡改拒绝、同源AV/AC（musl runner重验证ABC后真Linux执行）。evidence: elf02/elf03/elf05 @5a6b31d65。
- [x] AC-03 / work：ELF-04红控（未定义memcpy链接失败）；冻结组并集 ace-native 266/ace-av 1096/ace-lowering 5/ace-source 518/ace-artifacts 43/auto-lang-core 1613 全PASS（含Windows PIO/console回归）；ace-host 6红与clean master基线完全一致（run-36__ja1y对照，非本包新增，具名记录）；scoped fmt、musl构建零新增警告（av-core两条unused为master既有）。evidence: target/test-groups/run-_n6w671e + run-36__ja1y @088d331b1。
- [ ] AC-04 / review：停写后同树独立逐AC与原语/寄存器/目标/产物复审；遗漏/延期/workaround与Spec-impact明确。
- [ ] AC-05 / merge：最终冻结并集、正式代码/规范交付及归档，不把交叉编译冒Linux认证。
- [ ] AC-06 / merge：所有输出路径对账、scope删除或具名持有、guard clean后清树/分支。

- [x] **AC-MEM / work**：V-MEM AC（真mmap/munmap经mn_syscall6/2、AT_PAGESZ=4096、mapped取整/借用/重复释放/预算预拒、free后再分配）＋AV（LinuxPlatformIo真内核、live_mapped_at_exit=0）双绿；全程无libc/std::fs（asm!裸syscall＋auxv页大小）；Storage公共API未动。evidence: vmem_ac + elf05(a) @088d331b1。

- [x] **AC-WSL / work**：唯一WSL2环境显式冻结（§4表：Ubuntu-22.04/kernel/arch/tools/挂载/退出码传播/wslpath）；真实Linux AV（musl runner进程）与AC（静态ELF）结果均WSL内取得；错发行版拒、定点pid清理、子非零传播、stat/cat独立oracle、AC侧截止杀除＋幸存者扫描（F1修复：syscall-bound ELF 1.5s截止→Infra、/proc exe精确扫描零幸存）；Windows回归独立绿。证据限定该WSL2环境。evidence: elf00/elf01/elf02/elf05/elf06 @5a6b31d65。

## 8. 执行步骤

| task / owner_stage | 前置 | 操作与文件/结果 | AC |
|---|---|---|---|
| T-01 / work | 1039正式交付、实施指令 | 树/scope登记，核§4源、当前WSL2入场/发行版/kernel/arch/权限/工具/路径和UAPI/linker/最小源可行性，冻结原测试并集与WSL显式命令 | AC-01/AC-WSL |
| T-02 / work | T-01 | native emit/entry/link_driver/cache/config扩目标ELF与入口/目标身份；保唯一MIR；建ELF-01/03 | AC-01/02 |
| T-03 / work | T-02 | 目标syscall stub/平台Auto包装及AV Linux薄桥；建ELF-02/05，不用libc包装 | AC-02 |
| T-04 / work | T-03 | worker/native_exec受控WSL平台腿、依赖检查/故障控制/下一restore/真实Linux case和Windows邻例；补V-WSL的Linux子终态/字节/取消/清理负控 | AC-02/03/AC-WSL |
| T-09 / work | T-02/T-03及1039的区域合同正式交付 | 在现target stub/入口/AV薄桥加直接mmap/munmap和页大小来源；Auto平台包装复用1039区域账；建V-MEM及真实Linux AV/AC和无libc调用源审计 | AC-MEM |
| T-05 / work | T-04/T-09 | 完整受影响固定组、指定WSL2内Linux AV/AC实际运行、V-WSL/AC-WSL及独立Windows回归、fmt/警告、真实子终态/计数/依赖闭包汇总 | AC-01..03/AC-WSL |
| T-06 / review | T-05 | 同树独立复审，逐验字段/机器入口/后继边界及Spec delta | AC-04 |
| T-07 / merge | T-06 pass | 冻结来源与并集，代码规范落主、归档 | AC-05 |
| T-08 / merge | T-07 | 实际输出/缓存/持有对账与scope/wt-guard清理 | AC-06 |

## 9. 复审记录

2026-11-14 r5 / WSL静态预检：三份文档157处本地链接路径/围栏与Plan 0–10节通过；原9任务ID/owner_stage/状态/进度/依赖和全部原AC/ELF/V-MEM保留。新增WSL合同落T-01/T-04/T-05及AC-WSL/V-WSL/SD-WSL；1043已移归档，所触文档的两处旧链接改为archive路径，不改其历史。未探测WSL/构建/运行测试，不计环境或产品PASS。

2026-11-14 r5 / stage:new：核对原合同仅写Linux runner/Ubuntu拟基线、未指定WSL；按用户约束补WSL2唯一当前验证渠道、真实Linux AV/AC、显式子终态和资源/路径边界。只读Microsoft官方架构/命令说明并修改文档，未执行WSL探测/安装/转换/构建/测试，原状态/进度不变；静态预检另记。

2026-11-13 stage:new / CONTEXT-LOAD | 规划静态预检通过：20份设计/计划553处本地链接路径、Markdown围栏与Plan 0–10节/任务数一致；原144任务ID/勾选/状态/进度/旧AC/V及原依赖保留，新1043十任务均未完成。可达149个语言计划依赖无缺失/环、ID无冲突，中央.next-id为1044。1043不依完整966/FFI/库，1042不反依1043；966改为消费1043，原F0/B0/Hello与M1门不减。新计划候选固定组名和新增设计锚静态核对，git diff --check通过。未实施/建树/编译/运行产品或测试，不是独立实现review PASS；canonical Specs及归档历史未改。

2026-11-13 stage:new / COVERAGE | outcome: pass（规划静态核验，非产品或独立实现review） | 19份设计/计划519处本地链接存在；14计划共150任务，含新1042八项，其余142任务ID/勾选/状态/进度及原AC/V保留；0–10节/围栏/任务数一致。可达148个语言计划依赖无缺失/环，1042不反依完整库/977，967不反依968；原早期F0/B0范围不扩。历史5份非UTF8归档只按可解读frontmatter只读扫依赖，未修改。新增能力六路径尚未实施/实跑，不计产品PASS。

2026-11-13 原生底座依赖修订预检：9份文档371处本地链接通过；保原48任务ID/阶段及已有AC/V，新添1039/1040各T-09共2项work；可达149个语言计划无缺失/环，946与972新增硬前置无反向分配器依赖。状态/勾选/进度未推进，归档计划与canonical Specs零修改，.next-id仍1042。仅规划静态检查，非产品/独立实现review PASS。

2026-11-13 stage:new / r2：补OS内存来源；旧8任务及其AC/V保留，新增T-09一项work和AC-MEM/V-MEM/SD-MEM，9任务均未执行。本轮只有规划，旧r1预检计数是历史，不冒当前完成证明。

2026-11-13 stage:new / r1 | outcome: pass（规划静态预检，非产品/独立实现review） | next: work（正式依赖与实施指令齐后） | 12份设计/计划407处本地链接通过，原57任务及其勾选/进度/既有AC/V保留，新三包24任务/0完成/ID唯一；可达147个语言计划依赖无缺失/环，历史archived/930只读取材，未迁移/重开。Markdown围栏按长度检查，985四重围栏内的三重示例不是错误。

2026-11-13 stage:new / r1；草案待静态预检，0/8；未执行Linux工具探测/编译/系统调用或独立实现review。中央取号环境与等价串行登记说明见1039 §9。


2026-11-14 stage: work | plan_id: PLAN-1040 | plan_revision: 5 | outcome: pass | worktree: D:/autostack/.wt/lang-1040/auto-lang | branch: plan-1040-dev | base_commit: 10a3dd46e | code_commit: 088d331b1 | dependency_revisions: master HEAD（1039/1043已归档） | task_ids: T-01,T-02,T-03,T-04,T-09,T-05 | evidence: §4 T-01冻结表；native tests/cases/linux_elf_1040.rs 7例全绿（ELF-01..05/V-MEM/V-WSL，真实WSL2执行）；musl runner（x86_64-unknown-linux-musl + rust-lld）构建并在WSL实跑；冻结组 ace-native266/ace-av1096/ace-lowering5/ace-source518/ace-artifacts43/auto-lang-core1613 PASS，ace-host 6红=clean master基线（run-36__ja1y）零新增；scoped fmt零警告新增 | blockers: 无（ace-host既有6红属master基线，非本包触面，已在AC-03具名；ELF源不含浮点/CRT fmod面=关闭式链接拒绝，未冒支持） | next_responsible_role: review

事故与恢复记录（2026-11-14 work轮）：提交前清理fmt附带改动时 stash 误用（`git stash push`经管道head吞错＋pop了仓内他人旧PLAN-002 stash），worktree曾回到master态；`git reset --hard`恢复后由 plan-artifacts/plan-1040/codegen/ 脚本全集重放全部tracked改动（含assert锚点），未跟踪新文件未受损；重放树重跑1040全7例＋冻结组并集全绿后提交。7个未跟踪docs/plans/888-890*.md为旧stash残留物，不属本包，未纳入提交（待其owner处置）。

stage: goal | plan_id: PLAN-1040 | plan_revision: r5 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1040/auto-lang | commit: 088d331b1 | next: review


2026-11-14 stage: review | review_scope: final | reviewer: 同会话复审（实现会话内独立重建证据——非独立上下文，结论由提交代码/测试/原始结果重构） | plan_id: PLAN-1040 | plan_revision: 5 | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1040/auto-lang | branch: plan-1040-dev | reviewed_commit: 088d331b1 | base_commit: 10a3dd46e | dependency_revisions: master HEAD | spec_inputs: docs/specs/auto-ac/native-backend-and-retirement.md@77ba8a3c（§4取材）；docs/specs/auto-ac/linux-native-platform.md 尚不存在（SD-01新增目标，merge落） | acceptance_results: AC-01 pass/AC-02 pass（F2小修后复核）/AC-MEM pass/AC-WSL partial（F1）/AC-03 pass（含master基线一致6红具名）；AC-04/05/06属review/merge未计 | pending_closeout_gates: merge冻结组并集复用（088d331b1已跑，修复后须重跑受影响项） | findings: F1/V-WSL负面控制缺AC侧超时杀除见证——`run_elf_via_wsl`的deadline/定点pid清理路径无durable测试（AV侧取消已由elf05c覆盖，AC侧"超时取消"为V-WSL列名控制项）；最小修复=新增一例长循环ELF＋~1s预算→断言Infra超时且Linux进程已被定点清理。F2/探针预检AV/AC不对称——av-core dispatch拒flags==0/mode<0，AC shim(mn_pio_probe_openat)未镜像（AC侧该两输入会直达syscall）；最小修复=shim镜像同两检（负例：flags=0→-7）。 | evidence: 1040七例复审重跑7/7绿@088d331b1；冻结组 run-_n6w671e（ace-native266/ace-av1096/ace-lowering5/ace-source518/ace-artifacts43/auto-lang-core1613 PASS；ace-host 6红=clean master基线run-36__ja1y逐名一致）；worktree tracked全净（3个未跟踪docs/plans/888-890*.md为他人旧stash残留，非本包，不处置） | next: work（修复F1/F2后受影响项复跑+本review受影响条目复核）


2026-11-14 stage: work | plan_id: PLAN-1040 | plan_revision: 5 | outcome: pass (repair F1/F2) | worktree: D:/autostack/.wt/lang-1040/auto-lang | branch: plan-1040-dev | base_commit: 088d331b1 | code_commit: 5a6b31d65 | task_ids: F1,F2 | evidence: F2=shim镜像flags/mode预检，elf02/elf05(b)双侧断言-7；F1=elf06新增（probe_close(999)×100M syscall-bound、1.5s截止→Infra、墙钟有界、/proc/<pid>/exe精确扫描零幸存）；1040套件8/8、cargo t ace-native PASS(266) run-ol7xnja3 | blockers: 无 | next_responsible_role: review

## 10. 待澄清事项

WSL2实际发行版/kernel/linker/权限/挂载与现共享源能力由T-01先证；当前仅此Linux验证渠道，缺环境required保持pending，不假设另有裸机/远端。是否扩完整argv/env/TLS/unwind/动态库不在最小基线；必要时新立后继，不冒全Linux runtime。与1039/现native并行改同文件要按正式SHA短窗串行，不消费其他WIP。
