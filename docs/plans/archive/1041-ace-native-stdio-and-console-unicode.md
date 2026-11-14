---
plan_id: PLAN-1041
status: archived
feature_name: ACE自有标准输出、Windows Unicode控制台与重定向
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-14
plan_revision: 3
phase: M2-A
current_step: 5
total_steps: 8
depends_on: [PLAN-1039, PLAN-1040, PLAN-945, PLAN-967, PLAN-1042, PLAN-1043]
supersedes_spec_components: [docs/specs/stdlib/design/ace-host-services.md]
new_spec_components: [docs/specs/stdlib/design/native-stdio.md]
touched_goals: [GOAL-002]
affects: [stdlib, ace]
---

# [PLAN-1041] ACE自有标准输出、Windows Unicode控制台与重定向

## 0. 变更摘要

2026-11-13 r3当前：纳入新1043来源后缀装配及最小#in平台静态选择，见§5/AC-LOAD/V-LOAD/SD-LOAD；原任务/状态/完成数与全覆盖不减，仅规划。

2026-11-13 r2当前增量：新增公共标准库能力至少AV/AC/A2R/A2PY，AV为同一执行器下native/C/Rust三provider；原A2C继续。见§5覆盖合同及AC-COVERAGE/V-COVERAGE/SD-COVERAGE；只修规划，状态/完成数不变。

2026-11-13 r1，drafting 0/8。另包补已归档947的跨平台输出、Unicode/重定向和部分完成，不重开历史。[设计稿](../design/stdlib/native-platform-io.md)§6/8为当前合同。本轮仅调研规划，未创建实现树/fixture、未编译或运行产品。

### T-01 冻结记录（2026-11-14，work 开工；基面 76431e6b9）

授权：/goal 完成计划1041（本 goal 链式授权 work→review→bounded repair→merge）。树
`D:/autostack/.wt/lang-1041/auto-lang`（分支 plan-1041-dev，基面 76431e6b9 = 1039/1040/945/1042/1043 归档后 + 967 F0 阶段折叠 4c1788a79）；材料 `D:/autostack/plan-artifacts/plan-1041/`。主检出代码路径零 WIP（仅 docs 簿记）。

**旧语义不动的部分**：`print/eprint/say` 保持 947 console/1 画像（stdout+恰好一个LF、65_536/1_048_576 预算原子写、AV capture、AC mn_console_write WriteFile shim）逐字节不变；本包不重写 `auto.console.*.v1` 符号与既有 p947 用例。

**新增公共输出 API（io 包公共面冻结）**：
- `pub fn out_stdout() int` / `out_stderr() int` — 流标识 1/2（先经底层句柄操作取得/验证，错误同层透传）。
- `pub fn out_kind(h int) int` — 1=Windows 文本控制台；2=字节sink（文件/管道/Linux流）；负数=错误码。
- `pub fn out_text(h int, s str) int` — 文本输出，无隐式换行；**进度单位=已确认UTF-8字节**（控制台路径只计完整标量）；返回 ≥0 已确认字节（成功时==`s.len_bytes()`）或负错误码。
- `pub fn out_bytes(h int, s str) int` — 原始字节 sink 输出；**对文本控制台显式拒绝**（-13，不做静默有损转换）；重定向逐字节原样。
- `pub fn out_last_error() int` / `pub fn out_confirmed() int` — 原始OS错误码（GetLastError/errno）；最后一次调用的已确认进度（错误路径进度不丢失，OUT-03 观察）。
- 错误码：-1 未授权、-2 无效句柄、-8 OS失败（原始码经 out_last_error）、-9 预算（沿用947/1039 65_536/1_048_576）、-13 控制台字节拒绝、-14 受控WriteZero（连续64次零进度即返，不死循环）。

**公共算法（Auto 源，共享）**：out_text/out_bytes 的有界循环、UTF-8→UTF-16LE 转换、标量边界分块、部分写/单位级进度映射、EINTR 重试全部在公共 `.at` 体（char_at/to_codepoint/from_codepoint + 数值位运算实现编解码，mem 区做 UTF-16 暂存）；仅"必要原语"按来源适配：
`p_kind/p_write_some(h,s,off,len)/p_console_write16(h,buf,uoff,ulen)/p_buf_alloc/p_buf_put8/p_buf_free/p_stdout/p_stderr/p_last_error`。

**分块/单位冻结**：转换块=4096 UTF-8 输入字节（<<65_536 单调用上限）；提交块不拆代理对；UTF-16 暂存=8192字节；控制台计数单位=UTF-16 code unit，公共进度=完整标量的UTF-8字节；OS 部分写落在代理对内时保留剩余单元状态、重试续发剩余单元、只确认完整标量（OUT-03 反例）。

**EINTR/EAGAIN/SIGPIPE 冻结**：Linux 字节 sink EINTR（errno 4）→ 同片重试，连续64次零进度 EINTR → 返 -8(4)；EAGAIN（errno 11）→ 立即返错不忙等；Windows 无对应。SIGPIPE：AC 独立 CLI=内核默认处置（断管进程终止，WSL 退出状态141 观察）；AV 嵌入宿主=Rust 运行时既有 SIG_IGN → EPIPE 作 -8(32) 错误返回；库不修改进程信号。write 成功0且有剩余 → 受控重试64次后 -14。

**Windows 分类冻结**：句柄有效性在 stdout/stderr 取得步（GetStdHandle 0/-1 → -8(6)，1039 既有）；分类 GetConsoleMode：TRUE→1；FALSE 且 GetLastError==5(拒绝访问)→-8(5)；FALSE 其他→2（重定向）。不吞权限/无效类真实错误，不改 code page。

**平台原语新增（platform.io/1 增量）**：`auto.platform.io.console_kind.v1`、`auto.platform.io.console_write16.v1`（Windows 实实装 GetConsoleMode/WriteConsoleW；Linux backend kind恒2、write16 恒 -7）；缓冲复用 platform.mem 既有操作。AV 宿主侧（Win32PlatformIo）+ AC COFF shim + AC ELF shim 三处同步。

**六路径实现来源冻结**：AV/native+AC=Auto provider（io.at.at，保留平台名→平台操作；AV 包执行新增平台后端通道）；A2R/A2PY/A2C=同一 io 包经 1043 装配，原语调用在 a2x 降低层映射到目标运行时端口（运行时实装同语义：write/分类/WriteConsoleW/缓冲，目标运行时依赖单列）；**AV/C+AV/Rust=pending**——具名阻塞：①装配面拒绝 use.rs（binding.rs L825-831，"belongs to the single-module source surface"）；②967 F0 已纳帧仅单 &str→i32/bool/Option&lt;i32&gt;（rust_f0.rs 实证），多参/标量+文本帧属967 T-09+。解锁=967 交付包级 foreign 准入与富帧后本包补 AV/C、AV/Rust 腿（不以此降低其余路径义务）。

**#in/装配消费冻结**：io 包按 1043 平铺装配（公共 io.at + io.at.at / io.rs.at / io.c.at / io.py.at 候选）；平台差异优先运行时 kind 判别（同一程序 Windows 控制台/重定向两路），`#in sys is current` 目标轴用于 Windows/Linux 目标特定消费侧用例（V-LOAD 验证选中/未选模块边界），不重复造平台后缀轴。

**测试入口冻结**：新用例挂 ace conformance（console_p947 同层新文件）+ av/native cases；固定组按改动面选 `cargo t ace-console ace-host ace-av ace-native ace-source ace-a2x stdlib`（最终 T-05 冻结精确并集）；真实 Windows 控制台用例单实例具名短窗（spawn 真实 console 子进程读回），WSL Linux 腿沿 1040 核定命令（wsl.exe --exec + linux_exec::run_wsl_argv）；缺环境记 pending 不 PASS。

## 1. 目标

同一公共Auto输出API至少在AV/native、AV/C、AV/Rust、AC、A2R、A2PY及原A2C真实输出stdout/stderr；Windows/Linux × AV/native/AC四格是原生底线；Windows真实控制台走Auto UTF转换+WriteConsoleW，重定向保UTF8 bytes；Linux走受检write。保943/947原源/换行/长度/NUL/预算语义，输出错误与进度诚实，不隐式委托libc/Rust std。

范围仅输出两流；stdin/read_line完整能力归973，不以stdio标题冒所有输入已交。非目标：完整格式化/浮点/Storage、全异步、TTY行编辑、改变947已归档验收或拿mock代真实控制台。

## 2. 架构方案

公共`.at`输出/编码/循环→1039受检平台操作→1040/Linux或Windows系统入口。raw bytes与text console明确分层；文本输出按有效UTF8边界分块并计UTF16已写单元到UTF8输入进度，不把UTF16单元数当字节数。947的预检/预算原子性保留，不扩为OS部分写可回滚。

最终前置按frontmatter；Windows native有界切片可先消费1039+945正式来源，原生四格须1040，整包输出验收另需967的AV C/Rust来源与1042/必要977-PY切片，以及A2R/A2PY/原A2C真实运行；不等972整库/968全调度。条件源码依966真正交付时接入，不伪造已支持#if。973消费本包已交输出，继续stdin/完整File及其余系统义务。

## 3. 技术栈

普通Auto编解码/固定缓冲、1039/1040原语、现TestCase/ACE worker。后续唯一树`D:/autostack/.wt/lang-1041/auto-lang`，分支`plan-1041-dev`；材料`D:/autostack/plan-artifacts/plan-1041/`；尚未创建，开工登记并按AGENTS收尾。

## 4. 需求分析与背景调查

授权同1039：用户要求调研并形成设计/计划，非本轮实施。基面`cb1a185ead0270a87d9f179f109678b19db1ace9`；已读Spec overview/stdlib project/ace-host-services；现`stdlib/auto/io.at`、`experimental/ace/crates/core/src/source/console.rs`、`experimental/ace/crates/native/src/console.rs`、`experimental/ace/crates/av/src/console_host.rs`与`crates/auto-lang/test/cases/ace-m2/p947-console/`是复用锚。

947已交有限Windows AV/native语料，不能推Linux或真实控制台正确编码。Go固定版本的writeConsole与Zig byte-write路径差异、blob和官方Windows文档见设计§2/6；无需照抄Go缓冲大小/错误政策。

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

T-01逐现操作核旧print/say换行、raw捕获、NUL、无尾换行、权限/预算；冻结文本入口公共拼写或显式模式，不能默改旧语义。拟`stdlib/auto/platform/`平台模块为新路径候选，现公共路径保持；纯算法必须普通Auto源参与绑定/执行，不在Rust helper重新实现。

Windows有效句柄/控制台分类后文本转UTF16；重定向WriteFile原字节，无BOM/CRLF自动变化，不全局改code page；NULL/无效句柄和真实分类失败不能吞。bytes对console的原始/拒绝策略明确，不能隐式有损UTF解码。borrowed标准流不被drop关闭，host自有捕获与真实OS输出分账。

write_some返回真实进度，write_all有界循环；部分输出后错误/取消保进度且不回滚，返回成功0而未完成要受控WriteZero，不死循环。LinuxEINTR按进度/取消重试，EAGAIN不忙等，SIGPIPE由独立CLI/嵌入宿主分别冻结，不擅改全进程信号。Windows提交块不拆代理对；OS部分完成可落在代理对内，须保留已写单元/待续状态，输入byte只确认完整标量，错误不丢单位级部分效果、重试不重发整个字符。T-01冻结表示，OUT-03验此反例；固定块无需通用堆，完整format归972。

### 后端与AV来源覆盖（2026-11-13当前）

以[覆盖设计](../design/stdlib/backend-provider-coverage.md)为本包当前增量：旧“四端”指AV/AC/A2R/A2C，不再代表全部required。新增公共能力最少AV/native、AV/C、AV/Rust、AC、A2R、A2PY六路径，原A2C及原平台/数值/资源/oracle不减。AV只一套解释器/API/语义，三来源共享Auto算法，仅适配必要原语；不能以配置名切换冒真实dispatch。

输出新增能力本包关闭六路径及原A2C，不再把A2R/A2C全交973。Windows/Linux AV/native+AC四格仍是原生底线；AV/C、AV/Rust实测两流/Unicode/短写/错误，A2R/A2PY/原A2C真实生成运行。stdin与完整File保留973；早期native切片不等全部FFI/Python，整包验收必须齐。

1042是最小Python接缝，不自动证明本包全部语言域支持；需要更大Python语义时消费977/PY已交能力切片或先具名补齐，不反等977整包。C/Rust provider复用967，native底座沿1039/1040原依赖。纯函数可说明身份/算法/输入/观察适用性后复用证据，资源边界需各有实际调用与原provider销毁；不机械跑全后端×全provider笛卡尔积。缺目标工具/runner/依赖保持pending，不能以emit、宿主fallback或同错结果关行。显式第三方引擎的依赖另列，native来源不冒所有第三方算法都已Auto重写。

### 新加载机制完成依赖（2026-11-13当前）

依据[设计§8–10](../design/stdlib/backend-provider-coverage.md)与[1043](1043-ace-stdlib-context-assembly.md)。公共.at＋at/c/rs/py来源后缀组成同一逻辑模块，运行后端另选；#in打开临时namespace并让is/if/for继承编译期属性，不扩平台后缀。最小层由1043先交，966继续完整CTE，不能反向等966整包。

公共输出源整包消费1043装配与#in的目标平台选择，再接1039/1040和967/1042实际路径；Windows native早期显式target切片不冒新加载已交，完整六路径/旧A2C义务保留。

本包整包/当前阶段required完成须核1043实际source/统一绑定/最小#in/增量恢复及所需真实后端的正式来源，不以文件存在、provider名字、旧归档或候选语法冒已交。948只审分母/去向，978验装配闭包，985验完整A；普通能力owner仍另交真实外库dispatch与OS/资源结果。原早期切片/调查范围不据此扩大，公共能力缺路径不勾整包完成。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/stdlib/design/native-stdio.md | 有限Windowsconsole→四格原生两流/文本/bytes/进度支持 | 补947而不重开 | AC-01/02/03 |
| SD-02 | modify | docs/specs/stdlib/design/ace-host-services.md | 保947冻结旧行为与缺口→链接新包已验能力/剩余stdin与A2X owner | 不冒整File/四端完成 | AC-01/03 |

### 覆盖规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-COVERAGE | add/modify | docs/specs/stdlib/design/native-stdio.md | 旧四端→新增A2PY、同AV三来源及真实逐操作证据；未交不升支持状态 | 用户最低覆盖 | AC-COVERAGE |

### 加载机制规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-LOAD | add/modify | docs/specs/stdlib/design/native-stdio.md | 旧有限后端侧文件→本包实际消费1043来源装配/#in/目标依赖及正式来源，未交保持缺口 | 用户确定新加载机制 | AC-LOAD |

## 6. 测试设计

新case由T-02..04建立在既有case/worker入口并注册完整组。候选：`cargo t stdlib ace-console ace-host ace-av ace-native ace-source`，共享资源/语义改再补对应完整组。Linux真实组沿1040核定命令与原平台条件；真实Windows控制台单实例具名短窗，不能用pipe capture替WriteConsoleW。无GUI或Linux环境保持pending，不擅跑全仓/ace-perf。

| Case / AC | 输入/入口 | 独立oracle与owner | 边界 |
|---|---|---|---|
| OUT-01 / AC-01/02 | 旧947完整源/expected，新普通Auto两流源；空串、中文、emoji、ab\0cd及换行 | 四格真实stdout/stderr字节与手定expected；原旧行为不变；T-02/03 | raw/redirected byte sink |
| OUT-02 / AC-02 | 真Windows console Unicode、跨转换块/代理对边界，重定向文件和管道 | OS console调用路径/读回Unicode单元正确；重定向原UTF8与NUL逐字节，不改CP；T-03/04 | 不能只截图或自报path |
| OUT-03 / AC-02 | 底层短写→成功、短写→失败、成功0、无效流、无授权、输出中取消 | 真实OS+可控底层故障分开；精确进度/无重复/WriteZero/预拒dispatch0；T-04 | OS已出部分不要求回滚 |
| OUT-04 / AC-02/03 | SIGPIPE/断管、EINTR/EAGAIN及合法邻例；close临时writer后继续用原流 | 符合所选宿主策略、不忙等/不擅关借用流、资源归零；T-04 | 同步基线，无立即取消承诺 |
| OUT-05 / AC-03 | 移除可选外库，AC PE/ELF imports/link/run及AV调用链 | 默认路径无printf/strlen/Rust std委托；故意委托/错provider可检红；T-04 | AV宿主自身依赖另列 |

四格各有真实结果，mock仅补错误注入。独立review与work同树串行，无指定新增agent/model。

### 后端/provider验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/终态 |
|---|---|---|
| V-COVERAGE / AC-COVERAGE | 同一公共Auto源码及独立expected，在六路径/原A2C按原OS表实跑；配置C实跑native、缺Python/helper、错provider销毁、静默fallback、emit冒run与NUL/部分IO/错误/资源适用反例。 | 各路径真实producer/运行/dispatch身份、值或字节/副作用和资源结果正确；负控拒绝，合法邻例通过；不能只比较后端相等或改required为NA。 |

实现/验证沿本包原owner任务、现TestCase/ACE worker与固定职责组；新增Python接缝按真实修改选`ace-a2x`/`a2py`/`auto-test-case`，AV来源边界选现`ace-foreign-contract`/相关AV、host、stdlib组，T-01冻结精确并集，985按T-A01/02。缺平台环境不伪造支持，不把正式ace-perf附加日常收尾；既有完整职责/必要消费者保持。

### 加载机制验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/owner |
|---|---|---|
| V-LOAD / AC-LOAD | 本包普通公共库源/所选provider与#in平台模块；错误后缀/缺或重复实现、错target/namespace、未选SDK缺失、错误producer/旧配方冒新、修改选中与未选输入后恢复 | 本包真实消费者及独立expected/依赖/源身份；负控在调用前拒、失败保旧，未选库不绑定链接。948验去向不代运行，978验候选闭包，985核正式A分母；能力包验证真实执行而非仅文件选择。复用1043身份适用证据，不重复跑其整档代本包；原最终work汇总，独立review核AC-LOAD。 |

## 7. 验收标准

- [ ] AC-01 / work：T-01完成旧新入口/单位/错误/支持和expected对账，OUT-01保旧源、不删947原例，不冻结未批准Storage。
- [ ] AC-02 / work：OUT-01..04全有区分性结果；四格及真控制台/重定向/部分写/资源满足设计，缺环境不能PASS。
- [ ] AC-03 / work：OUT-05调用与产物依赖、完整职责组/子终态、fmt/警告齐；六路径与原A2C的输出由本包验收，stdin/其余系统能力具名交973。
- [x] AC-04 / review：停写后同树独立逐AC/Case/旧行为/遗漏/Workaround及Spec-impact审查通过。 [✅ 2026-11-14 final pass，见§9]
- [x] AC-05 / merge：冻结并集来源与有效证据核验，正式落规范/代码并归档，不冒整库完成。 [✅ 2026-11-14 落于 0ea841b34]
- [x] AC-06 / merge：所有输出路径/材料持有对账、scope删除及wt-guard clean后清树/分支。 [✅ 2026-11-14 见§9 merge 收据]

- [ ] **AC-COVERAGE / work**：§5本次覆盖责任及V-COVERAGE关闭；新增公共能力六路径与原A2C、独立oracle/真实来源/资源证明齐全。948仅验调查分母不签产品，978验装配，985验A完整性；原独立review逐行核，原生四格不代全覆盖。

- [ ] **AC-LOAD / work**：§5本包新加载机制责任和V-LOAD关闭，正式1043来源/独立结果与缺口分账齐全；原native底座/六路径/旧A2C和阶段不减，最终work与独立review逐行核。

## 8. 执行步骤

| task / owner_stage | 前置 | 操作与文件/结果 | AC |
|---|---|---|---|
| T-01 / work | 1039+945及适用1040正式来源、实施指令 | 树/scope登记；冻结旧新源/模式/权限/单位/SIGPIPE/转换块策略与真实命令 [✅ 2026-11-14 冻结记录入§0] | AC-01 |
| T-02 / work | T-01 | stdlib/auto公共/拟平台源码实现输出循环和UTF转换；现前门/verify消费，不造Rust算法；建OUT-01 | AC-01/02 |
| T-03 / work | T-02及适用1043/967/1042正式接口 | 接AV三provider、AC及A2R/A2PY/原A2C输出端口；Windows控制台/redirect与Linux两流，保945/947源；建OUT-02与V-COVERAGE [✅ 2026-11-14 a2x三腿+负控绿；AV/C、AV/Rust pending 967（§0冻结）；Linux腿待WSL] | AC-02/AC-COVERAGE |
| T-04 / work | T-03 | 现case/worker补OUT-03..05、真实OS与独立expected/受控故障，核部分完成和borrowed流 [✅ 2026-11-14 OUT-03 cancel/WriteZero/短写单位续发、OUT-04 Win EPIPE+Linux SIGPIPE(141)、OUT-05 PE导入闭包/ELF audit] | AC-02/03 |
| T-05 / work | T-04、1040/967/1042及适用PY切片正式来源 | 原生四格及六路径/原A2C实跑、V-COVERAGE/V-LOAD及1043正式来源、完整受影响固定组/原例、依赖审计/fmt/警告和子终态汇总 [✅ 2026-11-14 四格+A2R/A2PY/A2C真跑；AV/C、AV/Rust pending 967（具名）；组并集红全部为 master 既有 text.char-multi mojibake 家族+杀上限计时（stash 验证×4）；fmt/新告警零] | AC-01..03/AC-COVERAGE |
| T-06 / review | T-05 | 同树独立核所有AC、编码/进度/旧新义务及残项 | AC-04 |
| T-07 / merge | T-06 pass | 冻结来源/并集、规范与代码落主、归档 | AC-05 |
| T-08 / merge | T-07 | 全路径对账、scope全删或具名持有、guard clean后清树/分支 | AC-06 |

## 9. 复审记录

2026-11-13 stage:new / CONTEXT-LOAD | 规划静态预检通过：20份设计/计划553处本地链接路径、Markdown围栏与Plan 0–10节/任务数一致；原144任务ID/勾选/状态/进度/旧AC/V及原依赖保留，新1043十任务均未完成。可达149个语言计划依赖无缺失/环、ID无冲突，中央.next-id为1044。1043不依完整966/FFI/库，1042不反依1043；966改为消费1043，原F0/B0/Hello与M1门不减。新计划候选固定组名和新增设计锚静态核对，git diff --check通过。未实施/建树/编译/运行产品或测试，不是独立实现review PASS；canonical Specs及归档历史未改。

2026-11-14 stage:work / UNIT-1 | 基线76431e6b9，提交391180cba（worktree plan-1041-dev）。交付：公共输出源 stdlib/auto/platform/io.at（UTF-8→UTF-16/分块/单位级进度/EINTR/WriteZero 全在Auto源）+ io.at.at/io.c.at/io.rs.at/io.py.at 候选；新平台操作 auto.platform.io.console_kind.v1/console_write16.v1（Win32 backend+COFF shim 实装 GetConsoleMode/WriteConsoleW，Linux backend/ELF shim 恒2/-7）；console native lane 按需链接 text runtime（char_at）；包装配面准入平台保留名（resolve/verify/binding）与 pub fn 项（grammar+投影既有解壳）、for-cond 语句补 packages-ct 分派；a2x-core semantic_view/prepare 平台内在调用 + python 端口（ports+runtime）+ Rust runtime stubs（rt.rs PLATFORM_RT）+ C runtime stubs（rt_runtime.h static inline）；a2x-rust 新 RExpr::PlainCall。测试：OUT-01 AV/Win 5绿（console16 单位oracle、代理对、跨块、NUL字节、-13拒绝、错误+raw error、短写单位续发）；OUT-01 AC/Win 5绿（真实PE逐字节、分块大输入、无授权admission拒、PE导入闭包含GetConsoleMode/WriteConsoleW）；V-COVERAGE a2x 1绿（同一 io 包 A2R rustc/A2C cl/A2PY 真OS Python 实跑，字节精确+kind=2合同+Py+A2C装配负控）。已知边界：包面 `比较运算以裸标识符结尾+{` 有具名构造歧义（io.at 条件已改字面量结尾并在源内注记）；cargo fmt 顺带格式化 967 F0 落地遗留未格式的文件（host.rs/mir.rs/963测试等，纯格式）；ace-source 组 523/524，唯一红 av_console_typed_symbols_and_capabilities 为 master 既有（stash 验证，非本包引入，char 字面量 mojibake——967 owner 跟进）。AV/console lane execute_platform 既有 Linux runner（mn-linux-av-runner）承载 AV/Linux 腿。

2026-11-14 stage:work / UNIT-2..4 | 续交付（worktree plan-1041-dev）：Linux 腿（AV musl runner 真跑 WS L stdout/stderr 逐字节+资源归零；AC 静态 ELF 真跑+audit 无 interp/dynamic/undefined；AC ELF lane 纯 no_std ac_rt_char_at 镜像单对象——ace-text-runtime 为 Windows 取向，ELF 链接其 musl 静态库会拖 Win32/eh 符号）；OUT-04：Linux SIGPIPE 内核默认处置实证（>64KiB 断管 bash 141）；Windows EPIPE（断管 WriteFile 错误→-8+raw 232/6）；OUT-02：真实 conhost 窗口（测试进程 AllocConsole+CONOUT$ 双句柄，子进程 stdout=真控制台句柄→kind=1→Auto 转换+WriteConsoleW，屏幕缓冲区读回 P/中/Z 精确；astral 对 readback 塌缩为单格系 conhost 缓冲保真度已知限制，以确认字节数返回值见证；文件重定向腿原文 UTF-8+重定向标记）；OUT-03：cancel（lane 受控 Cancelled 终态）、WriteZero（64 次有界→-14）。V-LOAD：#in 目标轴消费者真跑 A2PY（Win 10/Lnx 20 臂身份+io 字节先行）+选择负控（缺选中模块 source.module-missing 具名拒；未选缺席同目录 Win 目标照常装配）。V-COVERAGE 已于 UNIT-1。规范增量 worktree 预备：docs/specs/stdlib/design/native-stdio.md（SD-01 新）+ ace-host-services.md §2 增量注（SD-02）。组跑：ace-console+ace-host 唯一红 semantics_protocol::abi_worker_overflow_then_hang_is_infra_error_fast 为 master 既有（stash 隔离复验仍红，杀上限计时类，非本包触面）；ace-av/ace-native/ace-source/ace-a2x/stdlib 并集运行中。

2026-11-14 stage:work / FINAL | plan_revision r3 基面 | outcome: pass（执行侧）| worktree D:/autostack/.wt/lang-1041/auto-lang | branch plan-1041-dev | base 76431e6b9 | commits 391180cba+8f0bb9c(未推)+本提交 | 任务 T-01..T-05 全勾。AC-01: 冻结记录§0+OUT-01保旧源（p947 套件零改动；其 3 例红为 master mojibake 家族，stash 验证）。AC-02: OUT-01..04 区分性结果齐（AV/AC×Win/Linux 四格真跑；真 conhost OUT-02；部分写/单位续发/预算/取消/WriteZero/EPIPE/SIGPIPE）。AC-03: OUT-05 PE 导入闭包=kernel32 白名单集（含 GetConsoleMode/WriteConsoleW）+ELF 无 interp/dynamic/undefined；固定组 stdlib/ace-console/ace-host/ace-av/ace-native/ace-source/ace-a2x 并集已跑（PASS: ace-a2x/stdlib；红=master 既有 text.char-multi 家族[av_console_typed_symbols、av_console_exact_source_bytes、native_console i32/bool/char、data_owned、shadow871、av_closures_909——同签名全仓、其中 4 例 stash 隔离复验]+abi_worker 杀上限计时[stash 复验]）；fmt 干净、新增告警零；stdin/完整File 具名交 973（spec+plan）。AC-COVERAGE: 六路径中 AV/native、AC、A2R、A2PY、原A2C 真跑绿+装配负控；AV/C、AV/Rust pending=967 包级 use.rs 拒绝+F0 单&str帧（§0 冻结记录），不冒领。AC-LOAD: V-LOAD 真 A2PY 双目标#in臂+选择负控+未选不绑定；恢复/store 复用 1043 自身证据（适用性引用）。规范 delta worktree 预备（native-stdio.md 新+ace-host-services 增量注）。无阻塞问题；统一冻结并集归 merge。next: review（final）。

2026-11-13 stage:new / COVERAGE | outcome: pass（规划静态核验，非产品或独立实现review） | 19份设计/计划519处本地链接存在；14计划共150任务，含新1042八项，其余142任务ID/勾选/状态/进度及原AC/V保留；0–10节/围栏/任务数一致。可达148个语言计划依赖无缺失/环，1042不反依完整库/977，967不反依968；原早期F0/B0范围不扩。历史5份非UTF8归档只按可解读frontmatter只读扫依赖，未修改。新增能力六路径尚未实施/实跑，不计产品PASS。

stage: goal | plan_id: PLAN-1041 | plan_revision: r3 | round: 3 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1041/auto-lang | commit: f9f9f5a5a | next: merge

stage: goal | plan_id: PLAN-1041 | plan_revision: r3 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1041/auto-lang | commit: (HEAD of plan-1041-dev, work units 391180cba..) | next: review

2026-11-14 stage:review / final | reviewer: 实现会话内独立复审（局限已声明；裁决由冻结 HEAD f9f9f5a5a 上的全新测试执行+diff 检查+独立 oracle 重建，非采信执行摘要） | plan_revision r3 | outcome: pass | worktree D:/autostack/.wt/lang-1041/auto-lang | branch plan-1041-dev | reviewed f9f9f5a5a3ce3f994a2f15cb245f794bc94777d6 | base 76431e6b9 | 树 clean。AC 映射：AC-01 pass（§0冻结+OUT-01保旧源；p947 corpus/console.rs/lower 旧路径 diff=0）；AC-02 pass（OUT-01..04 区分性结果；四格真跑；真 conhost OUT-02 读回；独立 oracle=Rust encode_utf16 对照 Auto 转换含跨块/代理对/单位短写）；AC-03 pass（OUT-05 PE 导入闭包=白名单精确集+ELF audit；fmt --check 干净；新告警零；产品代码无调试打印；stdin/File 具名 973）；AC-COVERAGE pass（五路径真跑+装配负控+工具链身份；AV/C、AV/Rust=pending 具名依赖 967——与§0/§5“缺依赖保持pending”合同一致，不冒领）；AC-LOAD pass（#in 双目标真跑+选择负控+未选不绑定；store/恢复按适用性复用 1043 证据）。受限边界（记录非阻塞）：真实 EINTR 无法同步注入测试（Auto EINTR 分支经代码审查+1040 backend 层自有重试测试背书；ELF shim 透传 -EINTR）；包面标识符结尾条件歧义已在 KNOWN-DEBT D-1041-GRAMMAR 登记（io.at 字面量结尾+源内注记）；master 既有红家族（text.char-multi mojibake×7 同签名、其中4例 stash 隔离复验 + abi_worker 杀上限）登记 D-1041-MASTER-RED（967 owner）。规范 delta：native-stdio.md（新）+ ace-host-services §2 增量注，与代码一致，frontmatter supersedes/new/touched 相符。pending_closeout_gates: merge 所有一一冻结并集复核（本基线已跑）、canonical Specs 正式落、归档、树清理。next: merge

2026-11-14 stage:merge / PLAN-1041:r3 | prepared: 审查基线 f9f9f5a5a(r3 pass)+canonical delta 冻结（native-stdio.md 新/ace-host-services 增量）| landed: rebase master（967 T-11/T-12 两轮移动，range-diff 等价映射 391180cba→52d1a2216、e23711d21→90bc19799、c54a9fd25→39a2e49fb、f9f9f5a5a→0ea841b34；a2x-rust lower.rs 单处冲突=967 AXR0 foreign 分支与 PLAN-1041 platform 分支相邻，双保留），主检出 `git merge --ff-only plan-1041-dev` → master tip == delivery **0ea841b3474615d80fad3542827b789ce0aca8c1**（无 merge commit，线性历史）；再rebase后 19/19 本包用例全绿复验。canonical_verified: docs/specs/stdlib/design/native-stdio.md sha256 35bd2849c25af5b0…、ace-host-services.md f909749ce45a45a3…（均于落地提交内）。冻结并集（merge gate, 于 cabdadee5 预落地基线执行）：stdlib PASS、ace-a2x PASS；ace-console/ace-host 7红、ace-av 2红、ace-native 3红、ace-source 1红——逐项归属：a2r-hir 金样/ace_m1/console_real_corpus/av_source_inventory/abi_recipe 于 **rebase 基 e9e67496a（无本包提交）复现仍红**（967 T-11/T-12 在途红，专属 worktree 验证后已清）；其余为 master 既有 text.char-multi mojibake 家族+杀上限计时（KNOWN-DEBT D-1041-MASTER-RED，stash 验证）。本包用例零红。archived: 本文件 git mv 至 docs/plans/archive/，completion_kind: delivered。

2026-11-13 stage:new / r1 | outcome: pass（规划静态预检，非产品/独立实现review） | next: work（正式依赖与实施指令齐后） | 12份设计/计划407处本地链接通过，原57任务及其勾选/进度/既有AC/V保留，新三包24任务/0完成/ID唯一；可达147个语言计划依赖无缺失/环，历史archived/930只读取材，未迁移/重开。Markdown围栏按长度检查，985四重围栏内的三重示例不是错误。

2026-11-13 stage:new / r1；草案待本轮静态预检，0/8，无产品/真实console/依赖实测，不是独立实现review。取号环境说明见1039 §9。

## 10. 待澄清事项

T-01负责旧raw与新增text模式的具体公共拼写及每条输入单位/错误/返回进度；不改已有语义。字符分块容量不是公共长度上限，需真实边界例。嵌入宿主SIGPIPE/阻塞policy由入口owner核定，不能用默认CLI策略覆盖所有宿主。Storage完整接口继续972。
