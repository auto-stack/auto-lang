---
plan_id: PLAN-1061
status: drafting
feature_name: ACE Linux专属测试隔离与原生Linux虚拟机执行
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-15
plan_revision: 3
current_step: 0
total_steps: 9
depends_on: [PLAN-1040, PLAN-1041]
supersedes_spec_components: [docs/specs/auto-test-case/module-selection.md, docs/specs/auto-test-case/ace-test-policy.md, docs/specs/auto-ac/linux-native-platform.md, docs/specs/stdlib/design/native-stdio.md]
new_spec_components: [docs/specs/auto-test-case/ace-linux-suite.md]
touched_goals: []
affects: [auto-test-case, auto-ac]
---

# [PLAN-1061] ACE Linux专属测试隔离与原生Linux虚拟机执行

## 0. 变更摘要

2026-11-15 r3衔接：用户进一步决定整个ACE标准库Linux分支先代码后测试，后续完整库验证另交[1063](1063-ace-stdlib-linux-deferred-validation.md)。本包仍负责隔离/去WSL/本地执行设施及原首批11项环境迁移，允许Windows隔离先交；当前不启动Linux测试。待环境完善后再执行本包T-07，未验仍pending，不把它的义务转给普通Windows计划，也不因1063新建而提前完成本包。原任务/0进度与历史收据保持。

执行用户2026-11-14新裁定：**测试不再使用WSL；Linux专属测试/计划只在虚拟机的原生Linux环境显式执行。** 增加 `cargo t ace-linux` 专项组，与 `ace-perf`同样为显式入口；普通Windows计划、`cargo t ace`、所有普通ACE模块以及Windows `ace-perf`均不准备、不运行Linux专项，不启动WSL。

当前已查明直接泄漏为1040的8项和1041的3项，共11个原测试身份，均直接挂在ace-native的conformance普通入口。它们目前依赖WSL路径/进程/信号运输，部分测试体还启动Cargo构建。需要隔离入口、改为Linux原生执行、前置准备与新旧义务对照；只改组名/加ignore或在Windows失败后跳过不算完成。

r2明确用户进一步确认的用例视角：**用例假设当前进程已经在真实Linux环境中，直接测试该环境中的程序和OS行为。** VM的创建、启动、连接与仓库/工具准备在测试外完成；测试本身没有WSL发行版、Windows路径转换或Windows远程启动Linux的包装层。此“假设”是运行前提，不能在Windows里mock成Linux并据此报告原生Linux通过。

本稿先形成可审查实施合同，尚未切换代码/配置或启动WSL/Linux测试。1040/1041归档历史保持原样，当前规范的“唯一WSL验收渠道”由本包交付后的新规则取代。

允许在VM执行尚未就绪时先交付Windows隔离切片：该切片只含普通/PERF过滤、旧Windows可运行WSL测试入口隔离、错误宿主的准备前拒绝及对应治理/说明；必须在同一实施树完成独立phase review与冻结Windows门后再落地。LinuxVM组运行义务仍pending，计划保持executing，不能宣称Linux已可用或本包完成；新增Linux运输代码不借此省略自己的验证。最终仍由T-08/09完成全包review/merge，阶段记录不提前勾选完整任务。

## 1. 目标

- Windows普通/聚合/性能入口对Linux专属case零选择、零Linux专属工具准备、零WSL进程；普通计划不以Linux验证作为review/merge门。
- `ace-linux`只在非WSL的原生Linux x86-64 VM内显式运行，在Windows/WSL入口前置拒绝，不先探测发行版、启动虚拟机、编译或下载Linux工具。
- 11旧身份逐项保持功能义务，WSL运输专属义务按新裁定退休/替换，跨目标混合case保Windows与Linux各自真实执行及身份拒绝关系；不在Linux上用模拟PE执行冒Windows结果。
- 将Linux runner/tool构建移至显式debug/test准备阶段；保真实ELF/AV执行、双流原始字节、退出/信号、取消与进程树清理。
- 全树扫描所有Linux专属测试、命令、script/recipe/配置入口及间接WSL调用，超出首批11项的增量也逐项归属，不只修一个慢case。
- Windows侧隔离证据与Linux VM实际运行证据分开。未提供VM或未实跑时记录NOT_RUN/阻塞，不能用Windows排除证明Linux通过。

本包不删除Linux能力、不重做平台IO/ABI，也不按“文本含linux”机械搬所有跨平台共享算法测试。Linux专属目标/OS行为测试归Linux专项；普通平台中立算法由实际合同判定，任何Linux腿均不能从普通组间接执行。新用例仍受10秒硬门，原迁移用例的规模/故障义务不因成本裁剪。

## 2. 架构方案

采用三个独立选择面：

| 入口 | 选择合同 | 执行环境 |
|---|---|---|
| 普通ACE组 / ace聚合 | 原职责条件 AND NOT performance AND NOT linux | 普通计划所在宿主；Windows不跨到Linux |
| ace-perf | performance AND NOT linux | 原正式套件环境；不隐含Linux腿 |
| ace-linux | linux（可含linux::performance） | 独立原生Linux VM、显式运行 |

Linux测试放入独立Cargo integration target（拟 `ace-native/tests/linux.rs`，注册 `[[test]] name="linux"`）并以 `linux::`根命名空间组织，Linux编译条件使Windows普通/bare Cargo测试不构建其可运行测试体；其他真实Linux测试包按所有权加入显式组。普通/perf表达式都排除Linux命名空间，避免将来新package或嵌套模块漏标。

Linux运行链为VM本地路径/进程→原ELF/AV入口→原判决，不再经过wsl.exe/wslpath/发行版。CLI执行/发现的环境准入在工具准备、输出域写入与Cargo发现前完成；`--list`/纯选择预览仍可只读展示ace-linux。不新建远程调度器，也不由Windows入口自动SSH启动VM任务。

## 3. 技术栈与工作位置

Rust现有ACE/native/AV执行路径、Cargo/nextest固定组、Python配置/准入治理；Windows实际PID/命令与Linux进程/信号分别观测。用户要求debug/test增量准备，普通测试体禁止隐式构建；真实compile/link专属验证留显式Linux专项并单列成本。

未来唯一实施树 `D:/autostack/.wt/lang-1061/auto-lang`，分支 `plan-1061-dev`；在Windows用原生Git/PowerShell创建与维护，**不调用WSL/bash启动WSL**。必要兄弟树以原生Git放同组，不用junction/symlink。材料唯一scope为 `D:/autostack/plan-artifacts/plan-1061/`；Linux VM本地临时输出的绝对路径、owner、清理条件在开工登记，回收验证摘要到统一scope，merge逐项对账全部VM/Windows输出。不得借用其他计划目录。

## 4. 需求分析与背景调查

### 4.1 授权、基线、规范与并行计划

用户明确取消未来WSL测试，要求Linux计划/测试移至独立VM并隔离为ace-linux。本次调查不探测或启动WSL。主线取材 `3a1741901a3cb94af33318cfd091e0387ed804c2`，文件hash见下；实际开工重新绑定。

[总览](../specs/overview.md)、[固定组](../specs/auto-test-case/module-selection.md)、[ACE策略](../specs/auto-test-case/ace-test-policy.md)、[Linux平台规范](../specs/auto-ac/linux-native-platform.md)、[native stdio](../specs/stdlib/design/native-stdio.md)及[1040](archive/1040-ace-linux-native-elf-and-syscalls.md)/[1041](archive/1041-ace-native-stdio-and-console-unicode.md)为来源。旧规范明确WSL是当时唯一Linux验收渠道，此条被本次用户裁定取代，不作为新验收约束，也不改写归档证据。

[1060](archive/1060-ace-regular-test-latency-audit.md)正在治理普通测试成本，本包独立承担Linux用途隔离与VM运输迁移。1060其他模块可继续；Windows ace-native的完整组预算在本包隔离落地后重冻集合，不能继续用WSL计时或把排除Linux冒作优化Linux本身。本包不等1060整包完成，和其owner协调test-groups/fixed-tests/prepare-tools写窗与debug准备的冻结切片，避免双写与环依赖。967/946等普通计划同理，不要求它们带Linux腿。

### 4.2 已核实际接线

- `native/tests/conformance.rs`无专项隔离地include `linux_elf_1040` / `console_out_linux_1041`；后者为 `cfg(all(windows, target_arch="x86_64"))`，表明这些目前是Windows→WSL测试，不能直接认为搬到Linux就能运行。
- `native/src/linux_exec.rs`包含wsl.exe构造、wslpath转换、PID文件/sh包装、截止时跨WSL定点清理；直接调用者是上述两份case。必须退出WSL测试链，换成Linux本地进程实现；Windows遗留测试调用不能回退到它。
- 1041 `out01_av_linux_runner_bytes_nul_both_streams`在测试体内 `cargo build --target x86_64-unknown-linux-musl`；1040也有Linux runner构建helper。准备须前置，runner来源/目标/摘要与本次源码绑定。
- `elf03_cross_target_cache_identity_and_restore_admission`混合Windows真PE构建/运行与ELF/缓存拒绝，须拆职责，保原Windows twin的exit7/stdout与Linux缓存/跨格式拒绝。LinuxVM不能执行此Windows腿；可在VM生成/检查COFF格式字节，但不冒PE实际运行。
- `scripts/fixed-tests.py`当前没有Linux专项group/platform准入；实际选择要同时处理普通与PERF，不只改ace-native一个表达式。
- `process_capture.py`有non-nt分支，仍依赖 `native_meter.workload_environment`；T-01/05需查其Windows ctypes导入以及非Windows路径，真实Linux发现/计量/截止必须可用，不能仅有CLI列表。
- 当前外层工具编译、nextest配置与Windows运行时规则由1060及原owner维持；本包Linux准备只选必要本地target，不全编Windows SDK/worker，也不触发release。

### 4.3 原件摘要

| 文件 | bytes | SHA-256 |
|---|---:|---|
| `experimental/ace/crates/native/tests/cases/linux_elf_1040.rs` | 29667 | `b8e2e48853a624e3d7ed90b9095dbe01de135b74a8ddb23919057f187f174609` |
| `experimental/ace/crates/native/tests/cases/console_out_linux_1041.rs` | 7867 | `4db5648c733a817a8cd84e345c706e9c46f94f07164ac1c267df32be3b47dfd9` |
| `experimental/ace/crates/native/src/linux_exec.rs` | 8203 | `87d6010b69a2fdc5620a2c3c9505a269421decd430f965e05881fefbe426c67e` |
| `scripts/test-groups.toml` | 58332 | `49de17b324db63b1c7004c5ae5fb1534bad41723d8a0b41d5e455e054c9ea281` |
| `scripts/fixed-tests.py` | 29758 | `075888cf4613ef1642582b423f64e4de1da047d960d0bf12991bbd0b06306ad4` |

## 5. 详细设计

### 5.1 首批11个真实测试的新旧义务映射

以下新全名为计划约定，最终以真实nextest发现冻结。函数调整/拆分允许等价命名，但每项旧断言必须有明确去向。纯WSL发行版解析的运输合同退休记录不可伪称原样通过；替代LinuxVM环境准入是新增测试，需≤10秒。

| 旧身份 | 原真实检查 / 必须保留的义务 | 新归属 / 拟身份 |
|---|---|---|
| `linux_elf_1040::elf01_linux_elf_real_run_bytes_and_exit` | 真实ELF执行、stdout/stderr UTF-8/NUL、exit42、无动态依赖 | `linux::linux_elf_1040::elf01_linux_elf_real_run_bytes_and_exit` |
| `linux_elf_1040::elf00_wsl_environment_resolution_negative_control` | 旧WSL发行版/路径解析拒绝；WSL运输义务退休，改为LinuxVM环境/错误平台准入反控 | `新LinuxVM准入反控（旧WSL专用身份退休）` |
| `linux_elf_1040::elf02_syscall_fourth_arg_openat_mode_and_errno_classification` | openat第四参mode、真实文件mode/内容与裸errno分类 | `linux::linux_elf_1040::elf02_syscall_fourth_arg_openat_mode_and_errno_classification` |
| `linux_elf_1040::vmem_ac_linux_anonymous_region_lifecycle` | 匿名mmap/munmap、页粒度、borrow/budget/释放/退出账本 | `linux::linux_elf_1040::vmem_ac_linux_anonymous_region_lifecycle` |
| `linux_elf_1040::elf03_cross_target_cache_identity_and_restore_admission` | Win/ELF身份、A09-ELF恢复与跨格式/错轴/损坏拒绝；拆分真PE执行腿与Linux腿，保同源关联 | `linux::linux_elf_1040::elf03_cross_target_cache_identity_and_restore_admission` |
| `linux_elf_1040::elf04_undefined_external_symbol_link_red_control` | 未定义外部符号真实链接失败关闭，不伪造成功 | `linux::linux_elf_1040::elf04_undefined_external_symbol_link_red_control` |
| `linux_elf_1040::elf05_av_linux_same_sources_cancel_and_caps` | 相同Auto源码AV真Linux执行、值/取消/caps、mmap/文件/errno状态 | `linux::linux_elf_1040::elf05_av_linux_same_sources_cancel_and_caps` |
| `linux_elf_1040::elf06_ac_wsl_deadline_targeted_kill` | 真实失控ELF截止和精确清理；替换为Linux本机进程/后代组终止及无残留见证 | `linux::linux_elf_1040::elf06_linux_deadline_targeted_cleanup` |
| `console_out_linux_1041::out01_av_linux_runner_bytes_nul_both_streams` | AV Linux runner双流字节、NUL/emoji、kind2、guest值20与资源退出零残留 | `linux::console_out_linux_1041::out01_av_linux_runner_bytes_nul_both_streams` |
| `console_out_linux_1041::out01_ac_linux_elf_bytes_and_audit` | AC ELF相同输出源双流字节/退出与依赖审计 | `linux::console_out_linux_1041::out01_ac_linux_elf_bytes_and_audit` |
| `console_out_linux_1041::out04_ac_linux_sigpipe_default_disposition` | 超过64KiB写入遇断管，真实SIGPIPE13及不挂起 | `linux::console_out_linux_1041::out04_ac_linux_sigpipe_default_disposition` |

ELF-03的Windows twin回普通Windows native测试，其原义务不新增、也不删除；Linux与跨格式部分归ace-linux。原函数内 `... || true` 这类非敏感断言不能当有效oracle，T-04据既有ELF/COFF magic及缓存拒绝合同以真实反例核其覆盖，不借迁移缩减义务。

全树另扫描实际 `cfg(target_os="linux")`、Unix/POSIX测试、Linux目标/ELF/裸syscall、WSL封装、脚本、Case recipe与prepare工具。静态命中需与原生身份/用途对照。`auto-tool-execution`的Unix生命周期tests属于共享执行基础设施，只有Linux专属腿才登记到专项/VM入口，不能将Windows功能覆盖整体删除；非ACE仓的Linux义务列实际owner和独立VM入口，不擅自扩展ACE全部workspace。

### 5.2 组、目标与准入

拟注册 `ace-linux`：runner=nextest，domain=ace，all=false，workspace=experimental/ace；首批args最小为 `-p ace-native --test linux`，expression=`test(/(^|::)linux::/)`。有新扫描包时按实际Linux测试目标补package/target，不自动拿ace-workspace全部package。独立调度profile可按真实信号/资源测试要求串行，保case内部必要并发；不是release编译配置。

所有普通ACE组、ace-workspace、ace-adapters及正式ace-perf的最终有效表达式都排除linux；配置守卫须覆盖根/嵌套命名空间、group alias及 `--test-name`导航不绕过。`cargo t`/all/ace不得自动展开ace-linux。Linux宿主上运行普通ace组也排除Linux专项，只有显式ace-linux才执行。

环境准入要求：原生Linux x86-64、记录VM系统/内核/架构/工具链与仓库修订；WSL kernel/WSL环境特征拒绝，不运行wsl命令作探测。Windows或WSL调用 `cargo t ace-linux` / `--discover-only` 在准备/编译前给非零明确错误，不能返回空集成功或NOT_APPLICABLE当LinuxPASS。未知/不支持架构明确失败。

Windows普通/bare Cargo也不编译WSL测试体：旧conformance移除两模块的普通include，新Linux integration target使用 `cfg(target_os="linux")`。在Windows即使裸 `cargo test`也无可运行Linux用例；配置排除之外还有编译入口隔离。单测/测试helper需要的WSL生产函数退役或改为显式不支持，不允许cache miss或环境缺失后自动启动WSL。

### 5.3 Linux VM本地执行与构建准备

- 替换内部Linux执行helper的WSL运输：VM本地绝对路径、argv直接执行、原始stdout/stderr、退出码与Unix信号分轴；不能经文本管道损坏UTF-8/NUL。
- 真实截止/取消必须清理本次拥有的进程及后代（Linux进程组/既有受管执行设施），保PID生命期身份和反控；不得杀整个VM、其他进程或用“父超时”代替无残留证据。SIGPIPE13不能被exit0或普通退出码混淆。
- AV runner在显式准备阶段debug构建，使用与VM架构/ABI匹配、可本地执行的原runner；通过路径/env与source/tool/profile/hash绑定传入，缺失/过期在准备失败，测试不得lazy Cargo构建。不强制沿用仅为Windows跨编译准备的musl运输，guest syscall/ledger合同保持。
- ELF被测产品编译/链接的专属正确性测试保留为Linux专项并计入case耗时；runner、独立oracle/工具自身的编译前置。构建host工具与执行Linux被测产物分别报告；Windows普通测试绝不因此启动Linux工具。
- Linux驱动不导入WinDLL/Windows Job接口；保Linux可用的真实终态/截止/清理观测，不能伪造Windows资源数字。无观测值如实记UNOBSERVED，测试时间使用真实墙钟。
- 核 `output_admission.py`、runtime临时根、配置别名和tool路径的Linux可用性；VM输出只准入T-01登记的原生绝对scope，不把Windows的D盘字面量当Linux路径，也不为方便放开任意输出目录。错误平台的拒绝仍早于这些输出/准备副作用。

#### 5.3.1 用例直接以原生Linux为前提（r2）

用例接收当前Linux环境已经准备好的路径/工具，直接调用本地进程/文件/OS API。测试层不感知VM产品、宿主Windows或如何进入Linux；宿主准入检查与VM调度属于外层执行入口。逐项改写原显式WSL调用及可达helper：

| 原测试依赖 | Linux用例改写规则 | 保持的oracle |
|---|---|---|
| `wsl.exe --distribution ... --exec <exe>` / `run_wsl_argv` | 直接以argv执行当前Linux文件系统中的ELF/预构建AV runner | 完整stdout/stderr原始字节、guest值、退出码/信号 |
| `wsl_resolve_path` / `wslpath` / Windows盘符路径 | 用当前Linux临时目录、原生绝对路径与本地文件元数据；无D盘或/mnt盘符映射假设 | 文件内容/mode、输入来源、独立oracle与清理 |
| 发行版名称、`WslEnv`、WSL缺失/错误发行版测试 | 退休WSL运输前置；外层核真实Linux运行条件，保或补本地缺工具/不可执行文件/错误输入等准入反控 | 环境/准入错误不能冒guest语义错误或PASS；新增控制≤10秒 |
| Windows终止wsl.exe后再跨WSL杀PID | 本地受管进程/进程组，按本次拥有的子树截止/取消并验证退出与无残留 | 原超时分类、精确清理、无误伤、正常邻例 |
| 测试内准备Linux发行版、从Windows编译runner或安装工具 | 外层在Linux显式准备debug source-bound runner/工具；case只消费准备好的路径，缺失明确失败 | 工具/源码/profile/目标绑定，禁止lazy build/WSL回退 |

不能将 `wsl.exe` 简单改名、用另一层shell/SSH/脚本继续间接调用，或仅把旧模块移到新目录。review追踪测试体→helper→实际子进程路径，确认是直接Linux执行；旧名称/历史映射文档可保留WSL文字，但可执行测试路径不含WSL依赖。Linux OS行为仍需真实VM实跑，mock只用于外层准入/治理负控。

### 5.4 守恒、负控与成本

迁前冻结旧11身份、原条件和原断言→迁后Windows twin/LINUX义务/退休WSL运输三方表。首批迁后测试条数可能因ELF-03拆分与环境反控替换变化，不能硬用“新11条”代替守恒。新名真实发现、选中/排除、实际执行分别留记录。

新增准入/隔离/工具准备反控每例≤10秒，review超时/缺测时判needs_fix；原迁移Linuxcase保原规模、signal/取消/预算/错误合同，逐项报耗时。Windows普通模块预算由1060核除去Linux后的完整真实选择，不把Linux隔离数字当编译/运行优化倍率。联合验证及其构建前检查全机其他agent测试/编译，有竞争等待并通知用户，显示具体组/阶段/阻塞对象；这不是允许继续WSL作基线的理由。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before / after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-test-case/ace-linux-suite.md` | 无Linux专项→ace-linux VM原生环境、最小target、首批11义务映射、平台准入/零WSL/准备和运行说明；r2要求case假设已在真实Linux，外层准备与case直接OS调用分开 | 独立运行与可验证隔离 | AC-01～06/09 |
| SD-02 | modify | `docs/specs/auto-test-case/module-selection.md` | 普通/PERF只按用途分→再排Linux；ace-linux为显式/不随default/all/ace，错误平台准备前拒绝 | Windows普通计划不碰Linux | AC-02/03/09 |
| SD-03 | modify | `docs/specs/auto-test-case/ace-test-policy.md` | 正式PERF隔离→性能/平台两轴；Linux性能在VM专项，WindowsPERF也不带Linux | 避免用途分离后仍跨OS | AC-02/05/09 |
| SD-04 | modify | `docs/specs/auto-ac/linux-native-platform.md` §4 | 当前唯一WSL→当前原生LinuxVM显式ace-linux；WSL旧证据仅历史，不是新渠道 | 执行用户新环境裁定 | AC-01/04/06/09 |
| SD-05 | modify | `docs/specs/stdlib/design/native-stdio.md` Linux验收 | 1041 Linux WSL运输→保AV/AC原字节/信号合同，VM原生实跑与构建准备分账 | stdout/stderr与SIGPIPE不能丢失 | AC-04/06/09 |

当前规范在review后由merge沉淀，规划不先把未交付LinuxVM结果写成已通过。GOAL-002原Linux能力声明不翻转，touched_goals=[]；这里只改变执行渠道/选择与验证组织。

## 6. 测试设计

全部待执行；本次没有运行WSL、Linux程序或产品编译。Windows侧可先完成隔离取证；LinuxVM实际运行是全包必需验收，未提供VM时允许独立工作继续，但不最终pass。

| ID / owner_stage=work | 实际输入与入口 | 明确预期 / 关闭oracle | AC |
|---|---|---|---|
| V-01 | 静态全树+真实旧/新发现 | 首批11逐义务去向和新增扫描项完整；无重复/漏项，Windows twin独立保留，WSL专用运输退休有记录 | AC-01 |
| V-02 | 配置select/default/all/ace、ace-native/host/workspace/perf最终表达式，带--test-name原Linux名 | 所有Windows普通/PERF选择没有Linux测试；只靠导航无法加回；显式Linux组与普通组分开 | AC-02 |
| V-03 | Windows/模拟WSL/错误arch调用ace-linux和discover-only；mock所有spawn/build | 明确非零/平台错误，准备/编译/WSL/VM启动0次；只读list/预览仍可展示；每新反控≤10秒 | AC-03 |
| V-04 | Windows模块的真实发现与受影响普通正确性入口；bareCargo入口检查 | 两旧Linux模块不在普通target下，无WSL可达链；独立ELF-03 Windows twin按原exit7/bytes判决；Linux条件排除不记PASS | AC-02/05 |
| V-05 | LinuxVM原生完整ace-linux，原11迁移义务+增量项；追踪测试/helper实际argv与路径 | case假设当前已在Linux，直接执行本地程序与OS调用；无WSL/发行版/Windows路径运输依赖，全部实际终态/双流/exit/signal/文件mode/ledger/缓存拒绝完整；mock不代真实OS结果 | AC-04/06 |
| V-06 | 真失控Linux进程/子进程，原截止与SIGPIPE断管 | 受控typed infra/超时，精确清理无残留；正常邻例通过，pipe信号13原样观测 | AC-04/06 |
| V-07 | 移除/改hash预构建runner、污染ELF/COFF/ABI、缺工具 | 准备/准入具名拒绝，不lazy编译、不调用WSL、不借原件/缓存假PASS；新控制≤10秒 | AC-03/04 |
| V-08 | Windows普通/PERF与Linux组的实际package/namespace集合、各自当前配置 | 选择守恒与排除明确；旧Windows功能未丢，Linux性能不回流Windowsace-perf；计时/首次红/NOT_RUN分列 | AC-01/02/05/07 |

治理负控复用 `scripts/tests/test_fixed_groups.py`：漏NOT linux、将ace-linux加进ace/all、漏package、Windows准入发生在build之后、--test-name绕过、WSL环境被误认nativeLinux、缺runner后fallback编译，各应可辨拒绝；检查配置时不得启动真实WSL。实际Windows零WSL另核受影响进程命令/可达源码，不用“没安装WSL所以失败”冒隔离。

Linux VM执行说明（拟由T-05/07交付）：在VM本地仓库、原生Linux终端中独立准备debug工具，再运行 `cargo t ace-linux --out <登记的VM绝对材料路径>`。宿主不负责SSH调度，也不从Windows共享target借二进制。输出记录VM系统/内核/架构、源/依赖/tool/profile/input身份、全部用例/内容/耗时与进程清理；回传摘要核SHA。

本计划最终门选择 `infra-fixed`及必要新增平台/执行helper治理，Windows受影响原组（native/共享配置消费者按真实改动选），Linux只显式ace-linux；不因新增分组在本机跑Linux测试，也不附加全量ace-perf。正式package/过滤变化用真实发现核全集，无新正式语义时不重跑一小时压力套件。编译/准备与测试、等待、发现、清理时间分列。现有普通工具若仍release，由1060/本包owner先交source-bound debug准备切片，不在验证时回退release；不等1060整包结案。

## 7. 验收标准

| 状态 | AC | owner_stage | 明确验收条件 | 验证 / 任务 |
|---|---|---|---|---|
| [ ] | AC-01 | work | 已扫描真实Linux/WSL触面；首批11+增量义务逐项迁移/退休/Windows腿对照完整，无静默删例 | V-01/08，T-01/04 |
| [ ] | AC-02 | work | 普通/ace聚合/all/PERF及命名导航不选择Linux专项，Windows普通/bare入口零WSL可达执行 | V-02/04/08，T-02/06 |
| [ ] | AC-03 | work | ace-linux错误平台/WSL环境在准备前拒绝；缺/过期runner不lazy构建；list预览可用；新增反控≤10秒 | V-03/07，T-02/05/06 |
| [ ] | AC-04 | work | case直接以当前真实Linux为前提，无WSL发行版/Windows路径/远程启动包装；VM原生ELF/AV、真实双流/NUL/exit/signal、syscall/mmap/缓存/恢复/准入/取消/进程清理义务不弱化 | V-05～07、§5.3.1，T-03/04/07 |
| [ ] | AC-05 | work | Windows twin原义务通过、普通功能与原PERF非Linux集合保持，条件/源/金样不误删，1060可按新Windows选择复测 | V-04/08，T-04/06 |
| [ ] | AC-06 | work | 真实LinuxVM上完整ace-linux有非零全量执行与合法/负控；环境/工具/源绑定、时长和清理可核，NOT_RUN不能算通过 | V-05/06，T-05/07 |
| [ ] | AC-07 | work | 会话报告实际模块/准备/执行/清理/等待时间、原迁移与新增case内容和时间；跨VM材料有owner/路径/hash及回收清单 | V-08，T-01～07 |
| [ ] | AC-08 | review | 同实施树独立复核全AC、过滤/准入/零WSL、11断言去向、VM真实证据、所有缺口与Spec delta；绑定r2及冻结提交 | T-08 |
| [ ] | AC-09 | merge | SD-01～05落canonical、归档，普通与VM各自门证据有效；全路径材料对账/guard清理，WSL旧渠道文字被明确取代 | T-09 |

## 8. 执行步骤

| 状态 | T / owner_stage | 前置 | 实际路径/交付 | 验证 / AC |
|---|---|---|---|---|
| [ ] | T-01 / work | 用户已确认Linux环境改写方向；1040/1041已交付 | 原生Git建唯一树/材料登记，重冻结本计划来源与旧11身份/所有断言；扫描全Linux/WSL/Unix/脚本/Case/helper/配置可达面；确定VM访问/系统/仓库和材料路径；核Linux驱动可运行性及1060写窗 | V-01/08，AC-01/07；未有VM不妨碍独立Windows隔离工作 |
| [ ] | T-02 / work | T-01 | `test-groups.toml`新增ace-linux最小target，普通/perf/alias全排Linux；`fixed-tests.py`在任何prepare/discovery前拒Windows/WSL/错arch；`native/Cargo.toml`/`tests/conformance.rs`与拟新 `tests/linux.rs`隔离Linux编译入口；补确定性配置/平台治理反控 | V-02/03/04，AC-02/03；Linux真实入口未完成前不能声称Linux组可用 |
| [ ] | T-03 / work | T-01 | `native/src/linux_exec.rs`及其测试helper按§5.3.1假设当前已在Linux，直接用原生argv/路径/退出码/信号/进程组与deadline清理；去WSL环境对象、盘符转换和回退/间接包装；必要执行设施以原合同接入，不改guest ABI | V-05～07，AC-04 |
| [ ] | T-04 / work | T-02/03 | 两份旧case逐§5.1迁新Linux目标；ELF-03拆出Windows twin与Linux/格式拒绝，elf00退休WSL运输并补原生VM准入反控，elf06改原生Linux后代清理；其余扫描项逐实际owner迁入；原规模/错误/字节/原oracle保持 | V-01/04～08，AC-01/04/05 |
| [ ] | T-05 / work | T-01/03 | `toolchains.json`/`prepare_tools.py`或原Linux显式准备说明接debug source-bound runner，去test内Cargo；补Linux驱动/observer与 `output_admission.py` / runtime根的实际非Windows路径，防WinDLL导入/假meter，限制登记的VM输出scope；确认无隐式release；登记VM本地输出及回收方式 | V-03/05/07，AC-03/06/07 |
| [ ] | T-06 / work | T-02/04/05的Windows可验部分 | 真实Windows发现/普通影响组与配置负控，验证0 Linux选择/0 WSL启动/无prepare前绕过，原Windows twin与非Linux集合守恒；提供1060可复用的组选择/代码/工具/配置身份 | V-02～04/08，AC-02/03/05/07；这不替代VM证据 |
| [ ] | T-07 / work | T-03～05；原生LinuxVM已可访问 | 在VM内显式准备并完整运行ace-linux，重放全部正反控/取消/清理/UTF-8/NUL/信号，记录每case实际时长/新控制10秒门、原生进程及源绑定；回传摘要/哈希，核原Windows边界不受影响 | V-05～08，AC-04/06/07；无VM/未跑保持未完成 |
| [ ] | T-08 / review | T-01～07完成 | 同树独立重建全AC及覆盖/过滤/环境/实际VM证据；按§5.3.1追测试/helper真实执行路径，查遗漏/延后/workaround、告警/格式、Spec delta；任一新例超10秒或缺计时needs_fix；结论绑定r2与冻结提交 | AC-08 |
| [ ] | T-09 / merge | T-08 pass | 按scope复用冻结Windows/VM证据，变更面才重验；规范沉淀、归档/实际全路径清理/guard移除树、记录Linux运行义务与旧WSL渠道替代；不自动启动WSL或强加Linux门给普通计划 | AC-09 |

## 9. 复审记录

2026-11-15 STD-LINUX-POLICY 静态核验：18份设计/计划717处本地链接有效；11份原计划115项任务的ID/勾选/状态/进度与原依赖保持，974仅T-08正文收紧默认transport来源，另增1063为drafting 0/8。新1063可达139份计划无缺失/环，不为能力建设增加反向依赖；章节、围栏与表格列宽检查通过，git diff --check退出0。仅规划静态结果，非独立实现review或产品PASS；本轮未改产品代码/测试配置、未构建/执行Windows或Linux测试，canonical Specs与归档历史未改。

2026-11-15 stage:new / r3：衔接1063未来完整标准库Linux验证，保持本包环境隔离/首批11身份/VM实际验收归属及原9任务。当前不运行Linux，Windows隔离可继续；未实施/测试，本修订不冒原生Linux入口已交。

2026-11-14起草交接：`stage: new` / `PLAN-1061 r1` / `outcome: pass`（规划可交接）/ `next: work`（本合同确认后T-01）。已静态核对旧8+3测试、普通include、WSLhelper和test内Cargo构建，明确Windows/普通PERF排除、LinuxVM前置准入、11义务映射及WSL运输退休/Windows twin分工。未改产品源码/测试配置，未启动WSL、Linux程序、编译或完整测试；本计划所有任务尚未完成。

2026-11-14 r2设计确认：用户进一步确认，显式调用WSL的用例应改成“假设我们在Linux环境中”测试。新增§5.3.1列明直接本地执行、Linux路径、信号/进程清理、外层准备及原WSL环境反控退休的逐项规则；更新T-03/T-08、AC-04/08、V-05与SD-01，原9任务/11身份与0完成状态保留。设计方向已确认，不再为相同环境改写方向重复索取确认；本次仅更新合同，未将其记作源码实现或VM验证。`stage: new | plan_revision: 2 | outcome: pass | next: work`。

## 10. 待澄清事项

原生LinuxVM的访问方式、架构/发行版、仓库位置与材料回收路径尚未给出，由T-01在实际开工时确认；目前规划不需要启动或访问它。LinuxVM不可用只阻塞T-07真实验收，Windows隔离/源码移植等独立工作可继续。普通Windows计划不得为了补这个前置而回退WSL；全包未取得真实VM证据不得宣称Linux专项通过。
