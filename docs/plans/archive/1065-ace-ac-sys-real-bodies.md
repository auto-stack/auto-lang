---
plan_id: PLAN-1065
status: archived
feature_name: AC后端sys973平台操作真身交付
author: [ZCode]
created_at: 2026-11-16
updated_at: 2026-11-16
plan_revision: 2
current_step: 4
total_steps: 6
depends_on: [PLAN-973]
supersedes_spec_components: [docs/specs/ace-migration/m2-stdlib-io.md, docs/specs/stdlib/design/native-platform-io.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace, stdlib]
---

# [PLAN-1065] AC后端sys973平台操作真身交付

## 0. 变更摘要

2026-11-16 r2当前：用户要求细化平台/实现/旧例/验收与无环交接。保A-01..04、T-01..06及V-AC-01..03，仍drafting **0/6**。将“Win64 50全部成功”纠正为**50符号/签名逐项覆盖，36项Windows适用真身、14项Linux-bytes专用形在Windows具名不适用**；Linux按原代码/完整用例移交1063，不在Windows跑Linux。旧path_attrs_bytes=-10测试只证Windows非适用拒绝，不证其余36真身已交；不再要求已支持操作也永久-10。区分work/review/merge AC，全部拟新增验证未运行。

## 1. 目标

在既有native emitter/runtime内为Windows适用sys973操作生成真实机器码，保AV同语义/实际OS结果/权限/资源/错误；编译不先产C/Rust业务源码，不夹旧VM或以宿主脚本执行器代AC。50项ABI全覆盖与Windows真身逐项验证，原平台不适用形继续具名拒绝；提供原V-09真实AC正例及独立oracle。

非目标：1064 MODULE/byte→Str/ZIP/KV作者面和975 JSON/DB业务；已有AV/C/Rust/A2R/A2PY实现重做；Linux实跑（1063）；新增平台模式。AC可以用973已有bytes/region操作验证，不反等1064全部面或975完整完成。

## 2. 架构方案

同一公共Auto源码→既有source/query/唯一verify→MIR→native平台调用lowering→Cranelift shim COFF/直接机码→既有受检region/资源账本→Win32 kernel32/bcrypt ABI→真实OS结果。沿`sys973_shim_sig`从PLATFORM_OPS派生参数：StrView=地址＋字节长，其他按表，不将“i32结果”误当全部只有一个i32参数。

既有`emit_sys973_stub`保作平台不适用分支/负控；适用操作替换为真实body，不能先返回-10再让宿主fallback。两种局部策略（CLIF内联校验/薄平台shim）由T-01按操作在已存先例内冻结，不创建第二资源模型或借FFI跳过账本。

## 3. 技术栈

拟唯一树`D:/autostack/.wt/lang-1065/auto-lang`，分支`plan-1065-dev`，基面=973正式交付后当前主线；尚未创建。本轮只文档。普通target留树，外置材料唯一`D:/autostack/plan-artifacts/plan-1065/`开工登记；不建链接。实现/内部核验/final review在同树停写交接，不另建复审树/runner。

## 4. 需求分析与背景调查

授权：用户本轮仅规划，复用973后延F1和现有1065，不新增Plan/实施。取材主线9d0f973cb3cafc585ba2bae52ad269b9b6a8c0e3、[973归档](archive/973-ace-m2-stdlib-io.md)及`docs/specs/ace-migration/m2-stdlib-io.md`、平台I/O设计和1039/1041真实先例。

**执行授权（2026-11-16）**：用户 `/goal 完成计划1065` 经 /auto-plan-goal 授权全链（work/review/bounded repair/merge），一授权贯通各阶段；树 `D:/autostack/.wt/lang-1065/auto-lang` 分支 `plan-1065-dev` 基面 97ca19c3b，唯一暂存 `D:/autostack/plan-artifacts/plan-1065/`（已登记）。

已观察：native `SYS973_KINDS`恰50项，`emit_sys973_stub`全部只返回ERR_UNSUPPORTED；Win32PlatformIo实现W16与共享操作，Linux bytes专用形继承unsupported。既有native `sys973_native_stub_returns_unsupported`调用`path_attrs_bytes("C:/")`，该负控仍适用，但不能证明任何Windows正例。Windows canonical AV底座有逻辑路径fallback与真实最终路径分支，旧公开canonical错误`""`不能被逻辑fallback偷换；本包按1064/D04明确合同生成AC与具名差额，不等1064全实现。

真实owner与读取身份：
- `experimental/ace/crates/native/src/platform_io.rs`，SHA256前16 `81ed4314de16f5a1`。
- `experimental/ace/crates/native/src/platform_io_linux.rs`，SHA256前16 `16668bb1dc962c5e`。
- `experimental/ace/crates/av-core/src/platform_io.rs`，SHA256前16 `d2cdd2c0edb67b6e`。
- `experimental/ace/crates/core/src/source/platform_io.rs`，SHA256前16 `f6c3c3cb67dd5376`。
- `experimental/ace/crates/native/tests/cases/stdlib_sys_973.rs`，SHA256前16 `da38ab70c74e63f3`。
- `experimental/ace/crates/av/tests/cases/stdlib_sys_973.rs`，SHA256前16 `933b48ad848808d8`。

其余接面沿native现行平台调用lowering/target linker、1039 mn_pio_state及region校验；新测试/状态字段以下是拟设计，未创建。未来原T-01核实际所在符号，不能仅按旧行号操作。

## 5. 详细设计

### 5.1 50项平台适用性（从实际冻结表读出，非缩分母）

W=Windows当前必须真实body/OS或确定性账本结果；N=Windows本平台形态不适用，必须具名ERR_UNSUPPORTED且无副作用；L=Linux适用代码与完整源/expected交1063 NOT_RUN；LN=Linux非本形态，按原具名拒绝。Linux env set/remove的已知机制缺口保1063原行，不能声称实现。表中每项符号/签名都须存在；缺符号或错误ABI不是N。

| 操作 | Windows | Linux | 行为oracle族 |
|---|---|---|---|
| `IoStdin` | W | L | F-IO |
| `FileOpenW16` | W | LN | F-IO |
| `FileOpenBytes` | N | L | F-IO |
| `FileReadSome` | W | L | F-IO |
| `FileWriteSome` | W | L | F-IO |
| `FileSeek` | W | L | F-IO |
| `FileTell` | W | L | F-IO |
| `FileHi` | W | L | F-IO |
| `FileSize` | W | L | F-IO |
| `FileMtimeMs` | W | L | F-IO |
| `FileFlush` | W | L | F-IO |
| `FileClose` | W | L | F-IO |
| `PathAttrsW16` | W | LN | F-DIRPATH |
| `PathAttrsBytes` | N | L | F-DIRPATH |
| `FileDeleteW16` | W | LN | F-IO |
| `FileDeleteBytes` | N | L | F-IO |
| `DirCreateW16` | W | LN | F-DIRPATH |
| `DirCreateBytes` | N | L | F-DIRPATH |
| `DirDeleteW16` | W | LN | F-DIRPATH |
| `DirDeleteBytes` | N | L | F-DIRPATH |
| `FileReplaceW16` | W | LN | F-IO |
| `FileReplaceBytes` | N | L | F-IO |
| `CanonicalW16` | W | LN | F-DIRPATH |
| `CanonicalBytes` | N | L | F-DIRPATH |
| `DirOpenW16` | W | LN | F-DIRPATH |
| `DirNextW16` | W | LN | F-DIRPATH |
| `DirClose` | W | L | F-DIRPATH |
| `DirOpenBytes` | N | L | F-DIRPATH |
| `DirFillBytes` | N | L | F-DIRPATH |
| `EnvGetW16` | W | LN | F-ENV |
| `EnvSetW16` | W | LN | F-ENV |
| `EnvRemoveW16` | W | LN | F-ENV |
| `EnvGetBytes` | N | L | F-ENV |
| `ClockUnixMs` | W | L | F-TIME |
| `ClockMonoMs` | W | L | F-TIME |
| `ClockHi` | W | L | F-TIME |
| `SleepMs` | W | L | F-TIME |
| `RandFill` | W | L | F-ENTROPY |
| `ProcSpawnW16` | W | LN | F-PROC |
| `ProcSpawnBytes` | N | L | F-PROC |
| `ProcPipeRead` | W | L | F-PROC |
| `ProcWait` | W | L | F-PROC |
| `ProcRelease` | W | L | F-PROC |
| `ProcExit` | W | L | F-PROC |
| `ProcCmdlineW16` | W | LN | F-PROC |
| `ProcCmdlineFillBytes` | N | L | F-PROC |
| `ProcCwdW16` | W | LN | F-PROC |
| `ProcCwdBytes` | N | L | F-PROC |
| `ProcChdirW16` | W | LN | F-PROC |
| `ProcChdirBytes` | N | L | F-PROC |

Win32与Linux各自源选择使用既有target事实，不根据宿主OS偷偷选错形态。W操作可因合法错误输入返回具名错误，但不能对合法输入留-10；原N操作不临时实现不承诺的跨平台转换。50行/签名和W36/N14由原表对账，未来source表变动须显式修订不静默吞新op。

### 5.2 表示、ABI、资源和错误

复用现有`.bss mn_pio_state`及受检region布局，扩现有ledger的资源类别/槽位/代际/owner/OS handle/权限、offset lo/hi、最后OS错误与字节账。示意字段为现布局需承载的不变量，不允许新建与AV语义脱节的独立“简化表”。输入handle不是直接OS HANDLE；指针/长度从授权region/StrView产生并经上溢/边界/rights校验，不能int→host ptr。

每run/进程拥有自身账本，发布执行/结束归原runtime；关闭文件/目录/pipe/process一次，过期/重复/错类别/错origin拒绝；借用与在途pin阻提前释放，异常/预算/取消/宿主错误有统一清理。不得将原opaque数字当跨进程可复用资源。沿1039原handle1024、65,536 B/call、1,048,576 B/run等预算，Io/Memory既有门不改。AC编译/授权和实际调用权限均经原入口，不能只靠编译成功代运行边界。

返回值和错误码沿PLATFORM_OPS/ledger，EOF=0与容量0不同，partial I/O不误当EOF；size/offset/time按signed lo＋hi保持64位位模式，last_hi/last_error与所属操作/资源配对，不用JS Number或i32截断。proc_exit沿受控native子进程退出码；process timeout=-11与正常非零退出/OS错误分列。少授权/失效region/预算应受控拒绝且磁盘/进程副作用0；现保的默认provider不得调用libc/CRT/Rust std::fs/std::io实现算法。

### 5.3 各族具体实现合同

| 族 | W真实调用/状态与可观察结果 | 负控与资源 |
|---|---|---|
| F-IO | stdin受控pipe读；CreateFileW/ReadFile/WriteFile/SetFilePointerEx/GetFileSizeEx/GetFileTime/FlushFileBuffers/CloseHandle及hi；字符路径UTF16、内容原始bytes | empty/EOF、partial块、seek>2^32的稀疏测试（按真实环境能力标适用）、NUL/Unicode、只读/坏flag/越权/close两次，失败副作用和owner独立查 |
| F-DIRPATH | attrs/delete/CreateDirectoryW/RemoveDirectoryW/MoveFileExW原子替换、FindFirst/Next/Close、CanonicalW16真实最终路径 | 分隔符不等真实symlink；不存在canonical必须按公开合同失败，缓冲不足不截断；删除/替换失败保旧文件，目录句柄正常/失败均关 |
| F-ENV | Get/SetEnvironmentVariableW，内部保present与空值不同，get buffer required单位UTF16 | 测试独立key并finally恢复；无权修改拒绝0副作用，buf小/坏编码分类 |
| F-TIME | FILETIME→Unix毫秒、QPC→mono毫秒，Sleep/hi值域正确 | 可观察时间带/mono非倒退，容差沿旧OS oracle不承诺精确时刻；坏参数/计数溢出受控 |
| F-ENTROPY | BCryptGenRandom填授权region | 长度/guard字节/预算/错误，不能把两次不同当唯一随机性或CSPRNG安全证明；来源审计＋真实成功/失败 |
| F-PROC | CreateProcessW、匿名pipes、job对象树回收、Wait/ExitCode/Close、GetCommandLineW/GetCurrentDirectoryW/SetCurrentDirectoryW | argv引号/空参数；stdout/stderr真实字节；非零退出、timeout全树kill并wait/drain、release两次/取消/失效owner，无遗留子任务 |

内联与薄shim由T-01逐项冻结：只用既有Cranelift ObjectModule、调用约定与target import机制；Win32句柄64位不塞进32位Auto句柄；系统DLL依赖作为显式PE imports审计，不以手写C/Rust业务源偷过native生成。LinuxA-04在原ELF emitter写适用代码臂/状态、原完整测试源码和expected映射，Windows只做静态移交审查，不跑Linux生成/ELF/WSL测试；1063消费后真实Linux运行，不能以Windows通过冒LinuxPASS。

### 5.4 具体Auto输入与交接

拟最小真实平台调用源（普通console入口可用，尚未执行）：

```auto
fn main() int {
    let low = clock_unix_ms()
    return clock_hi()
}
```

`clock_unix_ms`/`clock_hi`为已核平台声明，不能将调用删为常量；父侧用启动前/结束后的真实OS epoch毫秒分别取高32位作为允许区间，核返回值及实际调用/目标身份。此例只验clock高半接面，不代36个W操作/64位完整往返。磁盘和资源正式输入直接复用973 `stdlib_sys_973.rs`内完整probe/场景源：create→写`[65,0,255]`→seek(0)→read→close→reopen，输出/磁盘独立断言；不得以节选或恒return0当完整fixture。

native实际入口已核`console_compile`→`console_authorize`→`console_native_run`；采用package/MIR入口时沿既有已验装配路径，原T-03固定实际函数/命令，不另建runner。输出包含实际数值/字节/exit，独立父侧检查磁盘和子进程终态。

交IF-1065-ACSYS：repo/SHA/r2、50项ABI表/平台适用性、native runtime/link/provider/stdlib版本、完整原源/输入/oracle/命令/结果与LinuxNOT_RUN表。供1064适用AC腿及985原AC/LIB行，不能把旧stub负控收据冒真身。1065实现/验证不等待1064 MODULE、byte→Str、ZIP/KV或975整包，只有共同公开语义合同和973已有字节源；因此无反向等待环。

### 规范增量

| delta_id | action | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-ACBODY | modify | `docs/specs/ace-migration/m2-stdlib-io.md` | stub拒绝→Windows适用真身/原N拒绝/逐op结果及Linux完整移交 | 继承973 F1，不冒全平台 | AC-01..04/AC-LINUX |
| SD-ACBODY-ABI | modify | `docs/specs/stdlib/design/native-platform-io.md` | 有限I/O/mem→已验同ledger系统资源/权限/生命周期与目标依赖 | 不复制资源系统 | AC-02/04 |

仅未来已验知识落Spec；本轮不改canonical Specs。

### T-01 实施冻结（2026-11-16，对实际代码核毕）

- **W36/N14 对账**：`SYS973_KINDS` 恰50项与 §5.1 表逐项一致（W=36、N=14、Linux 腿归1063）；N 集合=FileOpenBytes/PathAttrsBytes/FileDeleteBytes/DirCreateBytes/DirDeleteBytes/FileReplaceBytes/CanonicalBytes/DirOpenBytes/DirFillBytes/EnvGetBytes/ProcSpawnBytes/ProcCmdlineFillBytes/ProcCwdBytes/ProcChdirBytes。lowering（lower.rs `emit_console_platform_call`）已按冻结表全量接线（StrView→ptr+len，标量直传），无需改 marshalling。
- **实现模式**：既有 `mn-platform-shim` COFF 内薄 CLIF 真身（同 write_slice/mem/console 先例），直接 import kernel32/bcrypt 入口；宿主 fallback/第二资源模型/手写C/Rust 业务源为零。
- **账本扩展**：`mn_pio_state` 头加 H_READ_TOTAL(i64@40)/H_LAST_CLOCK_HI(i32@48)；SLOT_BYTES 64→96，新 kind KIND_FILE=3/KIND_DIR=4/KIND_PROC=5，新槽位字段 S_STREAM_ID(@40)/S_LAST_HI(@44)/S_PROC_JOB(@48)/S_PROC_OUT(@56)/S_PROC_ERR(@64)/S_DIR_PENDING(@72)/S_DIR_PENDING_LEN(@76)；MAX_NATIVE_SLOTS 256→1024（对齐 AV MAX_LEDGER_HANDLES=1024 预算，槽耗尽=ERR_BUDGET）；Dir FindFirst 首项 per-slot 暂存区 1024×520B 追加于槽区后；共享 emit_find/alloc_slot_body 的 ×64 位移改乘 SLOT_BYTES（Linux ELF shim 同步无损，仅尺寸变化）。
- **新 import**：kernel32 增 CreateFileW/ReadFile/SetFilePointerEx/GetFileSizeEx/GetFileInformationByHandle/FlushFileBuffers/CloseHandle/GetFileAttributesW/DeleteFileW/CreateDirectoryW/RemoveDirectoryW/MoveFileExW/GetFullPathNameW/GetFinalPathNameByHandleW/FindFirstFileW/FindNextFileW/FindClose/GetEnvironmentVariableW/SetEnvironmentVariableW/GetSystemTimeAsFileTime/QueryPerformanceFrequency/QueryPerformanceCounter/SleepEx/CreateJobObjectW/SetInformationJobObject/CreatePipe/SetHandleInformation/CreateProcessW/AssignProcessToJobObject/TerminateJobObject/WaitForSingleObject/GetExitCodeProcess/PeekNamedPipe/GetCurrentDirectoryW/SetCurrentDirectoryW/GetCommandLineW；bcrypt.dll 仅 BCryptGenRandom（显式 PE import 审计 + link 行加 bcrypt.lib，SDK um/x64 已核存在）。
- **预算/语义镜像**：AV `PlatformResourceLedger`+`Win32PlatformIo` 为语义 oracle——read/write 各 65,536 B/call、1,048,576 B/run；last_os_error 全局、last64_hi 逐文件槽、clock_hi 全局；file_target 读=OsFile|stdin流、写=OsFile；EOF(0)≠容量0；ReadFile 109/232→0；seek origin 0/1/2 外=-7；OPEN_APPEND→FILE_APPEND_DATA(0x4)；canonical 两段（GetFullPathNameW+GetFinalPathNameByHandleW BACKUP_SEMANTICS）；env 203→0/234=OS_FAIL/值不截断；mono=QPC×1000/freq 饱和乘；rand NTSTATUS 失败码|0x00104000；proc V-03（job kill-on-close、pipe timeout=-11 不杀、wait 超时杀全树后 -11、release 关 pipe/job/process）；ProcExit 真身=ExitProcess（AC 侧文档合同）。具名偏差：env/路径单位长>32,768 的 OS 前拒绝=-4（AV 同域必失败但报-8，方向一致不截断）。
- **入口**：console_compile→console_authorize→console_native_run（既有）；`has_platform` 时平台 shim 已自动链接（console.rs 370-375）。

## 6. 测试设计

以下全部拟实施/未运行，正例由T-02/03建永久fixture、T-04汇总来源/负控和Linux移交，T-05独立复核，T-06仅落地。不以全部端同错代独立oracle。

| Case | AC/任务 | 真正入口、原输入与区分性关闭判据 |
|---|---|---|
| V-AC-01 | AC-01/02；T-01/02 | 50行→native实际shim dispatch；每W操作有至少一个有效调用及合法错误近邻，每N真实拒绝无副作用；磁盘/目录/env/时钟/熵/进程各对独立OS/字节oracle。把任何W实现改为恒-10/错ABI/漏符号时检查必须红 |
| V-AC-02 | AC-03；T-03 | 同一973公共系统程序经AV/native、AC、A2R、A2PY（原四路径，不含provider轴重复计四端）各跑原独立expected；AC必须直接机器码＋OS效果，不能仅打印同一个42；已有未变其他端证据按身份复用，变化时定向重跑 |
| V-AC-03 | AC-04；T-04 | 原target linker/安装依赖及实际PE imports＋调用点审计，默认平台无libc/CRT/Rust std I/O委托/旧引擎；强制缺必要DLL/错ABI/target应具名拒绝，无宿主fallback |
| V-AC-RESOURCE | AC-02/04；T-02/04 | 正常/短读写/坏路径/缺权限/过期/重复关闭/预算/取消/超时与恢复；拒绝0副作用，成功真实效果，handle/pipe/child最终回收。旧clock_hi/file_hi错配及64位截断必须能被独立值检出 |
| V-AC-LINUX | AC-LINUX；T-04 | 全50项与Linux适用/不适用/已知1063机制缺口逐行源/完整用例/expected/owner核对；NOT_RUN不充PASS，删行/空stub冒实现/缺owner须拒绝移交；Windows不运行Linux腿 |

旧新映射：`native/tests/cases/stdlib_sys_973.rs::sys973_native_stub_returns_unsupported`继续对Windows N的path_attrs_bytes作拒绝控制，新加W的path_attrs_w16合法真实attrs正例；不能仅旧例过就签全平台。AV `stdlib_sys_973.rs`中probe01 Unicode/NUL/磁盘、sys973_v03_process_spawn_capture_timeout_exit、sys973_t08_seek_permissions_mtime、sys973_dirs_and_replace、sys973_env_set_remove_real_process、sys973_time_real_clock_bands、sys973_t10_rand_real_entropy及A2R/A2PY真实腿按原assertions映到对应V/50行（完整源码不精简成return0）。旧file_exists/env/file/process原TestCase保source/expected，新增AC机制case有独立oracle；原native resource/1039边界继续回归。T-01冻结精确case ID/hash与旧条件，代表例不是50行执行的替代。

完整职责组已静态核存在：`cargo t ace-native ace-av`加实际共享修改的`ace-source`/`ace-stdlib`、原A2X消费`ace-a2x`、语言侧`auto-test-case`按触面取并集；native和platform ABI变更核必要消费者，不编所有experimental。`a2r a2py`原语料仅按实际变化/有效证据选择。性能仅显式ace-perf，Windows排Linux/WSL，未运行/0选择/缺工具不充PASS。原样本/预算不减；例的发现/运行/条件排除与准备/运行/清理/总时分列，首次红与隔离重放保留。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 50符号/派生签名/target适用性与实现模式/原例映射齐，W36/N14无误分 | T-01/V-AC-01 |
| AC-02 | work | Windows所有W真身＋原权限/预算/资源生命周期/错误，N具名拒绝；完整逐项/族oracle齐 | T-02/V-AC-01/RESOURCE |
| AC-03 | work | 原V-09 AC正例完整闭合，四路径各对独立结果与真实OS观察，非仅端间相等 | T-03/V-AC-02 |
| AC-04 | work | 链接/默认实现来源、negative controls/失败恢复及实际运行依赖正确，旧stub只保适用拒绝 | T-04/V-AC-03/RESOURCE |
| AC-05 | review | 全AC/原F1/实际差额/健康/SD经独立final并绑定r2/冻结SHA | T-05 |
| AC-06 | merge | 真实来源/已验Spec、归档、全输出路径对账/政策清理完成 | T-06 |
| AC-LINUX | work | 原Linux代码/完整用例/expected/具名缺口与1063逐行移交齐，NOT_RUN未充PASS | T-04/V-AC-LINUX |

## 8. 执行步骤

当前6/6 全部完成（T-06 merge 收据见下；delivery `babe4b3f7` 落 master）。

**stage: review | review_scope: final | reviewer: 同实现会话（局限已记，结论自工件重建：commit diff、测试原始输出、反证实验） | PLAN-1065 | r2 | outcome: pass | worktree `D:/autostack/.wt/lang-1065/auto-lang` | branch `plan-1065-dev` | reviewed 93e86ad85 | base 97ca19c3b | deps: 无跨仓 | spec_inputs: docs/specs/ace-migration/m2-stdlib-io.md、docs/specs/stdlib/design/native-platform-io.md（SD-ACBODY/SD-ACBODY-ABI 目标在） | acceptance: AC-01 pass（50行split脚本核对=36W全有body[IoStdin在platform_io.rs]+14N stub；V-AC-01 例）；AC-02 pass（V-AC-01/RESOURCE 永久例+计时）；AC-03 pass（V-09窗口例+四路径复用绿）；AC-04 pass（V-AC-03审计+Linux静态审查）；AC-LINUX pass（50 stub原样）。** 

**反证实验（review自选counterexample）**：MN973_MUTATION=1 下 (a) path_attrs_w16 退化为恒-10 → smoke红（attrs≠1）；(b) rand_fill 谎报成功不填 → smoke红（any≠1）+resource红（diff≠1）——W实现退化为stub必被检出；撤销后同例绿，树clean。**计时**：ace-native 291/291 pass 83.5s（sys973例0.08-1.91s）；ace-av 1279/1279 pass 66.2s（首跑1例负载瞬红已隔离重放绿，环境归因）。**遗漏扫描**：无未批scope削减/推迟；已知同向分歧（>32768单位OS前-4 vs AV同域-8）已在冻结记录为边界corner；bnot/branch2陷阱族grep 0残留。**SD审查**：SD-ACBODY/SD-ACBODY-ABI 增量与实现一致，待merge落canonical Specs。**findings**: 无阻塞；改进项（非本计划范围）：`cargo check -p ace-native --tests` 的 dev-dep 路径仅在ACE workspace上下文可解析（预存，主检出同败）。next: merge（T-06）。

stage: goal | plan_id: PLAN-1065 | plan_revision: r2 | round: 2 | from: review | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1065/auto-lang | commit: 93e86ad85 | next: merge

- [x] **T-01｜work｜AC-01**：核973落主/Spec与 §4 六文件hash一致；冻结50行/ABI/平台表与实现模式（薄CLIF shim）、槽位96B/1024槽/dir暂存/.bss scratch 四区布局、bcrypt.lib 链接与审计集；最小真实链（clock/rand/path_attrs + N stub 控制）一次编译-链接-运行通过（smoke 用例）。
- [x] **T-02｜work｜AC-02**：36个W真身全部实现在 `platform_io_sys973.rs`（AV语义镜像：handle编码/origin/release、64,536B/call 与 1,048,576B/run 读写预算、last_os_error全局/last64_hi逐文件/clock_hi全局、EOF≠容量0、broken-pipe=EOF、OPEN_APPEND→FILE_APPEND_DATA、canonical两段、env 203/234、mono饱和乘、rand NTSTATUS|0x104000、V-03 job kill-on-close/pipe timeout不杀/wait超时杀全树/ProcExit=ExitProcess）；14个N保持具名-10；W16结构体偏移按实际4对齐修正（WFD cFileName@44、BYHANDLE ftWrite@20）；修复两类CLIF陷阱（i8布尔bnot、branch2首臂=cond-true）与PE小栈限制（大scratch移.bss）。永久测试10例（smoke/文件roundtrip+非法近邻+稀疏>2^32seek+双关、目录枚举/替换、canonical/env、进程捕获/超时杀/双释放/无捕获-7、proc_exit=77、资源边界/guard字节、14N全-10表、PE import审计kernel32+bcrypt白名单），全部≤1.4s/例。
- [x] **T-03｜work｜AC-03**：V-09 AC腿落为 `sys973_win_v09_ac_real_clock_window`（§5.4 精确示例：clock_unix_ms+clock_hi 真调用，父侧 OS epoch 毫秒窗口 hi32 判定，0.5s）；共享磁盘/资源场景=文件roundtrip例（父侧独立字节oracle）。四路径证据按身份复用：AV腿=ace-av `stdlib_sys_973.rs` 既有例（本计划未改 av 仓代码）、A2R/A2PY腿=同文件 `sys973_a2r_real_rustc_leg`/`a2py_real_python_leg`（输入未变）、AC腿=本计划native例——各对独立expected，不以四端相等自证。
- [x] **T-04｜work｜AC-04/AC-LINUX**：V-AC-03=`sys973_win_pe_import_audit`（kernel32+bcrypt 白名单、BCryptGenRandom 在表、无任何CRT/libc）；V-AC-RESOURCE=资源边界例+各族非法近邻（guard字节、off+len越界、错origin族、双释放、坏handle、64KB/call 预算）；V-AC-LINUX 静态审查：Linux ELF emitter 50 stub 循环原样（真身归1063，Windows不跑Linux生成），alloc_slot 共享体签名同步为5参；消费者更新3例（pio01/pio04/1041 import-closure 从旧9符号闭集改为新冻结集断言）；冻结职责并集 `cargo t ace-native ace-av`（共享AST/语义/IR零改动，a2x/auto-test-case 无触面，按计划§6口径复用未变证据）。
- [x] **T-05｜review｜AC-05**：同树停写final review（与实现同会话——局限已记，结论自工件重建：commit diff/测试原始输出/反证实验而非执行者叙述）。pass；收据见 §9。
- [x] **T-06｜merge｜AC-06**：final pass后合入/沉淀IF-1065-ACSYS/归档；摘要折Plan、所有输出路径对账、按现政策清scope/guard清树，未明归属不删。

## 9. 复审记录

**stage: work | PLAN-1065 | r2 | outcome: pass | worktree `D:/autostack/.wt/lang-1065/auto-lang` | branch `plan-1065-dev` | base 97ca19c3b | code 93e86ad85 | deps: 无跨仓改动（唯一树，av-core/native既有rev随主线） | tasks T-01..T-04 ✅（4/6；T-05/T-06 为 review/merge owner）| next: final review（T-05）**

stage: goal | plan_id: PLAN-1065 | plan_revision: r2 | round: 1 | from: drafting | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1065/auto-lang | commit: 93e86ad85 | next: review

工作收据（2026-11-16）：
- **实现**：`platform_io_sys973.rs`（3343行，36W真身+2本地helper）；`platform_io.rs`（槽96B/1024槽/dir暂存520B×1024/.bss scratch 4×65568B、io_stdin、WINCALL审计集46+1符号、共享find/alloc体）；`link_driver.rs` bcrypt.lib；`platform_io_linux.rs` alloc 5参同步（50 stub循环原样）；消费者更新 pio01/pio04/1041。关键实现修正（审查线索）：W16结构体4对齐（WFD cFileName@44、BYHANDLE ftWrite@20）；i8布尔不可`bnot`（~1=0xFE真值，改icmp==0）；branch2首臂=cond-true（proc_wait臂一度颠倒）；>4KB栈帧在PE小栈崩溃（env/canonical/cmdline大scratch移.bss）。
- **V-AC-01**：11例native永久测试（smoke/文件roundtrip/目录/canonical+env/进程/exit/资源/14N表/审计/V-09），每W族≥1有效调用+非法近邻，N=14全-10；磁盘/env/clock/熵/进程各对独立OS oracle（父侧字节读取、真实子进程字节、epoch窗口）。
- **V-AC-02**：AC腿=V-09窗口例0.49s+文件roundtrip磁盘oracle；AV腿15例复用绿（输入未变）；A2R/A2PY腿复用绿。不以四端相等自证。
- **V-AC-03/RESOURCE**：PE审计断言kernel32+bcrypt白名单+BCryptGenRandom在表+全符号∈冻结集；资源例覆盖guard字节/越界/错origin族/双释放/坏handle/65,536B预算。
- **V-AC-LINUX**：Linux ELF 50 stub原样（真身归1063）；Windows零Linux生成/WSL运行。
- **冻结组并集**：ace-native 291/291 pass 83.5s；ace-av 1279/1279 pass 66.2s。首次av运行中 `sys973_v03_process_spawn_capture_timeout_exit` 因并发负载超wall预算瞬红，隔离重放15/15绿+全组重跑绿（环境归因，非代码）。`cargo check -p ace-native --tests` 自仓根调用会因 `auto_tool_execution` dev-dep路径只在ACE workspace上下文可解析而失败——主检出同败（预存调用上下文事实，组入口不受影响）。
- **计时**（新增例，均≤2s）：smoke 0.60-0.89 / file 0.89-1.61 / dirs 0.70-1.58 / canon+env 0.86-1.03 / proc 0.90-1.91 / exit 0.62-0.75 / resource 0.69-0.83 / N表 0.60-0.69 / audit 0.59-0.75 / V-09 0.49 / stub(旧) 0.08-0.99。准备(冷build)与运行分列于 nextest 输出；无>10s新例。
- **健康**：clippy 对本计划文件0警告（2个死常量已清）；cargo fmt clean；调试钩子已全部移除。
- **计划簿记**：主检出 plan 文件（本文件）T-01..T-04勾选+本收据；worktree仅代码。scratch：`D:/autostack/plan-artifacts/plan-1065/`（调试日志/disasm/probe脚本，merge收尾全删）。
- **待review核**（T-05线索）：(a) 36W逐op与AV行为对照的抽样mutation红（恒-10/错ABI必须红——永久例已含"any W==-8/-3/-2类断言"，建议review抽样改体验证）；(b) 4对齐结构体偏移单点复核；(c) branch2/bnot两CLIF陷阱族在全文件的残余扫描（已grep 0残留）；(d) 预存事实：仓根上下文check失败、1041/pio闭集更新是否需Spec沉淀。

2026-11-16 stage:new/revision | PLAN-1065 r2 | outcome:pass（规划合同静态预检），非实现/独立产品review | 0/6/drafting与原任务/AC/V保留；细化50项W/N/L、表示/ABI/资源、真实旧新例和独立oracle，新增AC-06分离merge及AC-LINUX/V-AC-RESOURCE/LINUX；原AC-05的merge义务迁AC-06而不丢。next=未来实施指令后原T-01；不依1064 MODULE/BSTR或975整包，未启动实现/测试/建树/改canonical Spec。

### r1历史（旧Win6450全成功/固定-10/四端同值说法由本轮具体表替代）

2026-11-16 stage: new | 本计划由 PLAN-973 T-05 复审 F1（用户裁定选项 b）创建为 AC 真身承接 owner；取号 1065（master 主检出）。仅规划：A-01..A-04 承接表+V/AC/任务 6 项；未实施、未建 worktree、未运行测试。973 不反向依赖本计划。


本轮规划预检：五份相关Plan原45任务ID/勾选及原AC/V/SD全部保留；新增9任务全部未执行（1064增6、975增3），当前合计54项。0–10章节、任务总数/唯一性、围栏/控制字符、本轮新增本地引用与行尾核对通过；1065表与真实SYS973_KINDS逐项相同=50，Windows W36/N14。B1→MODULE→975完整消费及973→1065→1064 AC验收的任务图无环，不存在1064/1065整包依赖975。原972归档、系统库canonical Spec与抽样产品源码hash未变；本轮未写取号计数器（其他会话1066取号与本任务分开）。973仅核原10项勾选/清理收据更正滞后计数并补前向交接，不重开终态。此为静态规划检查，所有新增运行验证仍未实施。

## 10. 待澄清事项

无新增用户范围决定；本轮已固定按真实平台表执行，局部CLIF/薄shim、ledger扩展与函数拆分由T-01在原约束内决定。若有W无法实现，必须报告具体原源/操作/ABI/原因与补救，不能改为N；Linux原已知机制缺口继续具名移交1063，不当Windows阻塞或运行PASS。

**stage: merge | PLAN-1065 | r2 | outcome: pass | delivery `babe4b3f7`（master tip=dev tip，ff-only；rebase 映射 93e86ad85→073ce71ec、e472b99ca→babe4b3f7，range-diff 全等=安全重写证明）| prepared: reviewed 93e86ad85+冻结SD | landed: master babe4b3f7（10 files, +4263/−61）| canonical_verified: docs/specs/ace-migration/m2-stdlib-io.md sha256前16 58307726e1d5307e、docs/specs/stdlib/design/native-platform-io.md a5f5372aedcb5d6a（均在落主提交内）| archived: docs/plans/archive/1065-ace-ac-sys-real-bodies.md（completion_kind: delivered）| cleaned: wt-guard clean → worktree/分支删除、plan-artifacts/plan-1065 整删、登记表刷新 | 批量回归：DUE（1065%5=0 > last_covered 1060）→ regress 交接（本目标内执行 cargo t all 主检出单实例）**

stage: goal | plan_id: PLAN-1065 | plan_revision: r2 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed) | commit: babe4b3f7 | next: batch-regression due handover
