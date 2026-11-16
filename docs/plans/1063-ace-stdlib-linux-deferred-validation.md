---
plan_id: PLAN-1063
status: drafting
feature_name: ACE标准库Linux分支延后验证与平台支持认证
author: [Codex]
created_at: 2026-11-15
updated_at: 2026-11-15
plan_revision: 1
current_step: 0
total_steps: 8
depends_on: [PLAN-1061, PLAN-972, PLAN-973, PLAN-974, PLAN-975, PLAN-978]
supersedes_spec_components: [docs/specs/auto-test-case/ace-test-policy.md, docs/specs/auto-ac/linux-native-platform.md, docs/specs/stdlib/design/assembly-manifest.md]
new_spec_components: [docs/specs/stdlib/design/linux-validation.md]
touched_goals: [GOAL-002]
affects: [stdlib, auto-ac, auto-test-case]
---

# [PLAN-1063] ACE标准库Linux分支延后验证与平台支持认证

## 0. 变更摘要

执行用户2026-11-15裁定：**整个ACE标准库路线先实现Linux分支，当前不执行Linux测试；环境完善后由本计划统一验证。** Windows验证继续作为各建设包的当前完成门；Linux行保留原分母、源与独立oracle，标记`deferred-required / NOT_RUN`，不改为PASS/NA，不由交叉编译或Windows结果替代。本计划所有任务未执行，不启动WSL/VM、不编译或运行Linux程序。

[1061](1061-ace-linux-suite-isolation.md)负责测试隔离、去WSL运输与Linux本地执行设施；本计划负责972–975等后续标准库的完整Linux能力验收，两者不重复。1040/1041历史已跑收据保留，不重开归档。本计划不反向阻塞建设包和985当前Windows交付，不授权发布未验证的Linux支持声明。

## 1. 目标

收齐Linux目标下公共标准库的Auto实现、平台接线、各后端/provider、资源和SDK证据，完成后才将本次延期行升为verified。实现缺陷在本树修复并按原能力owner/规范回归，不削减原支持集。非目标为搭建VM/远程调度器、重写第三方数据库/TLS引擎、修改已归档收据或自动新增macOS/arm64产品支持。

## 2. 架构方案

原公共Auto源码→1043装配/#in选择Linux→共享Checked HIR/MIR或A2X→真实Linux AV/ELF/Rust/Python/C产物→本地OS→原TestCase独立oracle。AV/native与独立AC仍为Auto算法＋Linux直接syscall，无libc/Rust std默认委托；显式C/Rust provider及目标语言运行时另列依赖。

验证须1061原生Linux入口已正式交付、可访问的非WSL Linux x86-64 VM及相关producer正式来源。VM生命周期、连接与工具准备在测试外；不从Windows自动SSH/启动VM，也不回退WSL。不等待977的GD/SCENE/REVERSE全包，A2PY用1042及实际消费的977/PY切片；968调度由973/974传递消费。

## 3. 技术栈

沿现ACE workspace、Auto标准库、TestCase/worker、1061拟交的显式`cargo t ace-linux`及目标工具链。唯一拟实施树`D:/autostack/.wt/lang-1063/auto-lang`，分支`plan-1063-dev`；主检出只做计划簿记，不创建链接。统一材料scope为`D:/autostack/plan-artifacts/plan-1063/`；VM本地输出路径/owner/回收条件由T-01登记。

## 4. 需求分析与背景调查

用户已授权此次设计/路线/计划修订及Linux先代码后测试的排程；本轮仅文档，无实施或测试，无新工期/模型预算。来源：[Specs总览](../specs/overview.md)、[Linux平台规范](../specs/auto-ac/linux-native-platform.md)、[stdio](../specs/stdlib/design/native-stdio.md)、[测试策略](../specs/auto-test-case/ace-test-policy.md)、[平台设计](../design/stdlib/native-platform-io.md)、[覆盖合同](../design/stdlib/backend-provider-coverage.md)。旧规范/1040的WSL收据是历史事实，未来环境由1061用户裁定取代，不改历史expected。

已静态确认的真实接缝：`experimental/ace/crates/native/tests/cases/linux_elf_1040.rs`、`console_out_linux_1041.rs`、`experimental/ace/crates/av-core/src/platform_io.rs`、`stdlib/auto/platform/io.at`及`io.at.at`、`scripts/test-groups.toml`、`scripts/fixed-tests.py`。1061迁移后用其正式新路径，不能假定WSL helper可在Linux直接运行；完整库源/新增case由972–975提交，本轮未生成。

起草取材HEAD为`4295a23bb062b43e57fcd405029fa6cdad2f6e14`。T-01重冻结源码SHA、provider/ABI、SDK/工具/runner摘要、VM发行版/kernel/arch及实际路径；旧绿不自动复用。

## 5. 详细设计

### 延后义务移交与执行

建设包逐操作保留：原case/check身份、完整输入与oracle、backend/provider、Linux目标/ABI、Auto源/OS适配/外部依赖、工具、未跑原因及本计划owner。`implemented`只说明代码已写，`verified`须有实际运行证据；Windows成功不传播到Linux。沿原分母/manifest/Plan维护，不造平行注册系统。

| owner | 接收的Linux范围 |
|---|---|
| 972 | math/text/bytes/parse/format、集合、Storage/allocator与原完整操作 |
| 973 | stdin/stdout/File/path/env/process/time/rand、错误/权限/取消/资源与原系统行为 |
| 974 | 地址/DNS/TCP/UDP/HTTP/stream、redirect/proxy/WebSocket/SSE原范围；Auto协议＋OS socket/事件入口，TLS显式分账 |
| 975 | Auto JSON/编码/序列化/缓存及原标准能力；SQLite/Redis等显式引擎不冒Auto重写 |
| 978、1042/所需977-PY | 真Linux候选SDK/恢复；六主路径及原A2C中全部已承诺Linux格 |

1040/1041底座仅按身份仍适用范围复用；新/变化Linux行全部保留未验状态。新API/增量并入交接分母，起草列表不是永久上限。

环境就绪后在VM内显式准备debug工具，再运行`cargo t ace-linux`及T-01核定的实际Linux标准库/后端职责组；缺注册由本包扩既有组/真实case，不在普通Windows组塞Linux腿。Linux文件/信号/errno/epoll/mmap/ELF与AV本地进程分别观察；差分同时须满足独立expected。

网络用loopback动态端口、独立对端见证字节，不用公开互联网/假response。合法/拒绝/部分进度/EOF/取消/close/最后owner/恢复保持原规模；新增测试遵现ACE时间门，正式性能只显式专项。来源不符、零选择、仅emit、工具缺失不得PASS。修复影响Windows共享代码时补受影响Windows固定组，不重复无变化全仓检查。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/stdlib/design/linux-validation.md（新） | 延期账→逐操作Linux真实来源/结果；不改历史 | 兑现延后验证 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-test-case/ace-test-policy.md | 承1061→新标准库Linux显式组归属与完整映射 | 无隐藏Linux执行 | AC-01/02 |
| SD-03 | modify | docs/specs/auto-ac/linux-native-platform.md、docs/specs/stdlib/design/assembly-manifest.md | 已交底座→核实后扩Linux库/SDK支持；宿主/目标/外库分账 | 支持声明有证据 | AC-03/05 |

## 6. 测试设计

全部拟议；T-02..05将移交源接成实际case，T-06执行。旧case先迁移/复用，不因下表生成第二框架。

| V / AC / owner | 输入、入口 | 独立观察与拒绝控制 | 边界 |
|---|---|---|---|
| V-01 / AC-01 / T-01、02 | 原交接分母与1061入口 | 原ID/输入/oracle逐条有去向；漏Linux/漏A2PY、错源码或零选择须拒绝 | 含后续新增API |
| V-02 / AC-02 / T-02、03 | Linux AV/AC ELF，同源core/集合/alloc/File/双流；中文/emoji/NUL/空/大块/错误输入 | 原字节/值、文件副作用、mmap回收、errno/权限/短读写/EINTR/取消与无残留；Windows PE不代Linux执行 | 六路径适用OS腿，不外推整个AV无Rust |
| V-03 / AC-02 / T-04 | Auto net，经真实loopback/socket/事件API及独立对端；原TLS样本 | 原请求/响应字节、framing/超时/断连/背压/取消/端口释放；来源审计发现隐藏libcurl/reqwest/hyper默认委托 | OS内核TCP/IP允许，TLS引擎显式 |
| V-04 / AC-02 / T-05 | 数据/显式引擎；AV三provider、AC、A2R、A2PY/原A2C Linux格 | 独立expected＋真实dispatch；错provider销毁、fallback、emit-only和共同错误负控 | 同身份纯算法可说明复用，adapter不能冒名 |
| V-05 / AC-03 / T-05、06 | Linux仓外SDK/restore、错target/ABI/依赖、失败后再恢复 | ELF依赖及默认调用链审计，实际再次运行；未跑仍NOT_RUN，不升verified | 宿主/目标/外库分账 |

公共拼写沿交付源码与原expected；T-01先证1061入口/工具可运行，再冻结实际组和命令，不凭假想函数写fixture。未提供VM为环境阻塞，不要求本轮用户立即提供。

## 7. 验收标准

| ID | owner_stage | 标准与验证 |
|---|---|---|
| AC-01 | work | 延期行及所有增量逐条归属、无漏项或错误降级；V-01闭合 |
| AC-02 | work | 适用Linux格真实执行，原值/字节/错误/资源及来源oracle满足；V-02..04闭合 |
| AC-03 | work | Linux候选/恢复/依赖审计齐全，只对有证据行升verified；V-05闭合 |
| AC-04 | review | 独立上下文核AC/V、原义务、来源与修复影响；未闭不得pass |
| AC-05 | merge | 验证知识落Spec/支持范围，合回/归档及实际全路径清理；草案不冒Linux可用 |

## 8. 执行步骤

全部0/8。T-01可只读取材；依赖与真实Linux环境就绪后才执行测试。本包不是建设包Windows完成的反向前置。

- [ ] **T-01｜work｜AC-01**：核1061入口/VM本地路径/工具及972–975/978来源；冻结分母/哈希/实际命令/材料scope；缺环境记录阻塞。
- [ ] **T-02｜work｜AC-01/02**：承972与1040/1041历史case恢复core/集合/Storage、stdout与ELF/AV义务，接V-01/02；依T-01。
- [ ] **T-03｜work｜AC-02**：承973 Linux系统操作，接真实权限/errno/进度/取消/资源case，关闭V-02；依T-01/02所需接缝。
- [ ] **T-04｜work｜AC-02**：承974 Auto net与TLS分账，经真实loopback/独立对端关闭V-03；依T-03所需来源。
- [ ] **T-05｜work｜AC-02/03**：承975、967 adapter、1042/所需977-PY及978，完成Linux六路径/旧A2C和仓外restore，关闭V-04/05；依T-02..04相应库。
- [ ] **T-06｜work｜AC-01..03**：显式运行冻结的Linux完整受影响组，核每case结果/无残留；修复后按变化复验；收齐分母/支持标记，未跑不关闭；依T-02..05。
- [ ] **T-07｜review｜AC-04**：停止实现并发写入，同树独立核源/命令/结果/遗漏/SD，缺陷回同树修复并复核；依T-06。
- [ ] **T-08｜merge｜AC-05**：review pass后复用冻结证据、按变化补门，落Spec/支持范围、合回/归档；摘要折Plan，清材料scope及VM实际输出/树/分支，登记路径并跑wt-guard；依T-07。

## 9. 复审记录

2026-11-15 STD-LINUX-POLICY 静态核验：18份设计/计划717处本地链接有效；11份原计划115项任务的ID/勾选/状态/进度与原依赖保持，974仅T-08正文收紧默认transport来源，另增1063为drafting 0/8。新1063可达139份计划无缺失/环，不为能力建设增加反向依赖；章节、围栏与表格列宽检查通过，git diff --check退出0。仅规划静态结果，非独立实现review或产品PASS；本轮未改产品代码/测试配置、未构建/执行Windows或Linux测试，canonical Specs与归档历史未改。

2026-11-15 stage:new / r1：用户已确定延期方向；本稿为未来验证合同，依赖1061及能力正式来源。所有V/T均未执行；仅文档静态预检。环境未就绪不阻当前Windows建设，本包尚不能开跑实际验证。静态结果另记本节。

## 10. 待澄清事项

VM访问、发行版/kernel/架构、工具路径与最终分母由T-01核定；缺少时不执行依赖验证。额外架构/OS须另定范围。当前不请求重批已授权的设计/计划修订。
