# auto-ac

> **Status**: experimental
> 路径：`experimental/ac-core`（独立 Cargo workspace，bin `auto-ac-prototype`，"ac-probe" 为历史文档名） | 技术栈：Rust + Cranelift 0.126.2 + rust-lld

AC（计算核心 profile）首个原生 AOT 闭环：Checked HIR → Windows x64 COFF → PE 原生执行。
**HIR → 原生执行与有限源码 profile → 原生执行均已完成**；PLAN-745 的
[source-core-i32.md](source-core-i32.md) 提供 A1–A3 源码闭环，PLAN-772 的
[source-scalar-i32.md](source-scalar-i32.md) 提供标量扩面（减/除/余、全序比较、
同类 eq/ne、一元、布尔短路、条件 for）。完整语言面、AAC/ACC native 自举与
其它能力仍由后续计划承接。

## 目标与范围

- 消费 `verify::CheckedModule`（唯一合法来源）做 native lowering；无寄存器/SSA 残留进 checked 层。
- 目标平台 Windows 11 x86-64（`x86_64-pc-windows-msvc`）；调用约定 WindowsFastcall；bool 内部/传参
  均为 i32 0/1。
- 静态库/DLL/热重载/生产 ABI 冻结不在本层（Native ABI RFC 主线，不因 exe 里程碑延后）。

## 工具链（T-01 收据冻结）

- 后端：cranelift-codegen/frontend/module/object 0.126.2（`ObjectModule::emit()` 直接产 COFF）。
- 链接：首选 rust-lld `-flavor link`（sysroot 内路径可预测）；MSVC link.exe（vswhere 定位）为备选，
  双链路实测通过。PATH 上的 `link.exe` 是 Git coreutils——必须绝对路径取 MSVC 链接器。
- SDK：注册表 `KitsRoot10`（本机 D:\Windows Kits\10，10.0.26100.0）→ 已知根回退 →
  `AC741_SDK_UM_LIB` 环境覆盖；kernel32.lib（um/x64）为唯一链接输入。
- 发现逻辑源：`src/link.rs`（`AC741_RUST_LLD` 环境覆盖可注入故障用于测试）。

## 入口与启动

- CLI 入口 = 零参数、返回 i32 的已声明 Function DefRef（`--entry <DefId>`；intrinsic/签名不符拒绝，
  跨模块 id 歧义拒绝）。
- 生成的 `ac_start` 包装器通用（不按 fixture 硬编码）：调用 entry → `ExitProcess(结果)`；Auto 函数
  不直接作为 OS 加载入口；PE 以 `/entry:ac_start /subsystem:console kernel32.lib` 无 CRT 链接。
- HIR 模式模块内函数名即 COFF 符号：重名在 backend 阶段拒绝；intrinsic 按声明 symbol 文本作 import。
- 源码模式入口按源码函数名查 DefId；源码 native 符号加 sf_ 前缀，用户可合法声明
  ac_start/ExitProcess；不改变 HIR 身份/绑定与 OS 入口包装。

## 溢出 trap（测试 profile 约定）

- `add_i32`/`mul_i32` 的 `overflow: trap` 走独立错误退出：检测（可移植有符号溢出判定 / i64 加宽比较）
  → `ExitProcess(70)`。退出码 70 为**测试 profile 约定**，不是全语言异常 ABI；正常测试入口退出 0。
- 不依赖 Rust debug overflow、不回绕出正常结果（native_execution 边界测试锁定）。

## 能力门（capability）

- 文档 `requires`（hir 级）声明的能力必须由 `--capability` 显式提供才可 build（`capability.missing`
  拒绝）；无静默跳过路径。
- `hir.test.trace.v1`：测试专用 trace 能力。实现为 `test-support/` 独立 no_std 静态库
  （`hir.test.mark_a`/`hir.test.mark_b` 经 kernel32 直写 stderr），`--support-lib` 显式链接；
  **不实现任何 Auto 函数、不解释 HIR**，非生产字符串/I/O/插件 ABI。

## CLI 与制品

- 源码模式 check-source/build-source 必须显式 source profile——**已注册三档**：
  `auto.source.core-i32.draft`、`auto.source.scalar-i32.draft` 与
  `auto.source.record.draft`/`auto.source.text.draft`（未知值用法错 2，各域支持/拒绝面互不扩大：
  scalar 拒 while、core 拒 for/新运算符、record 增名义 record 面且旧域拒绝
  该面）。源码 →
  Atom/span表 → 真正 bind/verify（按域分派 descriptor 与 verify 入口）→ Checked
  → 与 HIR 模式共用 finish_build/native/link/发布尾部。支持/拒绝、0/1/2、trace、
  来源映射与无固定嵌套排除见两份源码规范；一键 scripts/verify-ac-source-745.ps1
  与 scripts/verify-ac-scalar-767.ps1。
- HIR 文本模式 check/build 按文档头三重身份选择**已注册 descriptor**（core/
  scalar），未知身份 `bind/bind.descriptor-unknown` 受控拒绝，不猜测不回退。
- build-source 收据在同一发布事务内记录 source profile、**HIR 三重身份**
  （schema/revision/profile）与输入 hash；源码函数 native 符号 sf_ 前缀与
  入口规则两域一致。

- `auto-ac-prototype check <file>`：bind+verify，诊断 `file:line:col: error[stage/code]`；退出码 0/1/2
  （成功/流水线拒绝/用法错）。
- `auto-ac-prototype build <file> --entry <DefId> --output <exe> [--capability N]... [--support-lib L]...`：
  产物 `.obj`/`.exe`/`.ac-link.txt` 收据落 `--output` 目录；**失败构建不覆盖既有成功制品**：
  exe/obj/收据全部先以进程唯一暂存名构建，发布前备份既有制品，**任一步失败（收据暂存/备份/
  三次发布）整体回滚**——暂存文件（含已链接 exe）全出口回收、还原备份、清除已放置文件；
  回滚自身受阻返回 `link.restore` 诊断并列出幸存备份路径（不吞错、不为无残留删唯一旧制品）；
  **清理（remove/rename）失败同样不吞**：`link.cleanup` 诊断携带原失败、事务提交状态
  （NOT committed / COMMITTED）、每个自有残留路径与实际 OS 错误（NotFound=已回收；
  目录=用户占位，不删不计）；发布已提交但备份回收受阻单列 COMMITTED 状态、不伪称普通成功；
  **pre-publish 的 `link_object_staged` 两错误出口（链接器非零退出、执行器错误
  spawn/deadline/containment/wait）同受此约束**：自有暂存 exe 回收受阻时同样以 `link.cleanup`
  报告原失败+残留完整路径+实际 OS 错误+NOT committed+恢复办法，不得仅返回
  `link.failed`/`link.deadline` 而吞掉删除错误；用户占位目录永不备份/删除；
  SENTINEL 前置失败与准备/备份/发布各阶段失败测试锁定。
- 全部子进程（工具发现 rustc/reg、链接 lld、运行 exe）走带硬截止时间的执行器；
  **单一单调截止同时覆盖进程等待与输出收集**；**受控启动**——子进程以 CREATE_SUSPENDED
  起跑、入 Job（KILL_ON_JOB_CLOSE）后方恢复执行（ToolHelp32 快照恢复主线程），
  首个指令前必被管控、后代无法在约束外创建（r3 的 spawn→分配窗口与后代逃逸已消除）；
  **约束失败=受控拒绝**：安全收口（挂起态终止+wait）+ `link.containment` 诊断，
  不先放行子进程再无声降级为 detach；任一出口关闭 Job 即终止自有子树并 join 回收
  reader（无 detach 分支）；子进程退出后后代继承管道不再绕过截止；stdout/stderr 并发
  排空防管道阻塞；输出以 read_to_end+lossy 转换，不返回截断的成功输出（read_to_string
  全有全无 UTF-8 校验遇本地化输出会丢弃合法前缀）。
- 一键验证：`scripts/verify-ac-741.ps1`（§6 全命令 + CLI 实机构建运行 + 主仓门禁；PS 5.1 兼容，
  异步管道读防死锁；含 `cargo check --all-targets` 零 warning 健康检查）。

## text profile（PLAN-770）

- [source-text.md](source-text.md)：str/Str/char 首批有限值语义合同
  （字面量/索引/比较/码点转换、拥有-借用语境与 `.move`、清理责任不变式、
  能力与运行期合同门）。
- `runtime/text`：独立 workspace 静态库（23 符号内部 ABI：LIT/CONCAT/
  CLONE/TEXT-SCALARS/FREE 生命周期 trace、eq/ne、RT-FAIL 失败通道、
  预算/故障注入控制面 + `<lib>.contract.json` sidecar）；**实验性内部
  ABI，非生产模块 ABI、非 ACC Cranelift 桥**（774 D-05 边界重申）。
- source/HIR→native 文本编译：verify_text → TextResourcePlan → 独立 text
  lowerer → 真实 PE（V-02..V-09 真实运行实证，4096 深链探测帧）。
- text-record 组合未立项：770/771 各自 pass 不构成组合验收（774
  interface-contract §6）。

## record profile（PLAN-771）

- [source-record.md](source-record.md)：名义 record 的首批有限值语义合同
  ——声明（含前向引用/无环）、位置/命名构造（源码序求值 + 值捕获不变式）、
  字段路径读写、嵌套 record、聚合参数/返回（ABI-D/E）、`static fn` 与只读
  实例方法、显式 `mut` 参数与 per-call 排他；source/HIR→native 闭环经
  24 步门禁 23 例与 57 测试实证（V-01..V-10）。
- 内部 ABI 边界重申（774 D-05）：非生产模块 ABI、非 ACC Cranelift 桥；
  存储不重叠义务（目的地不相交门控 + 操作数求值位捕获）为 lasting rule。
- text-record 组合未立项：770/771 各自 pass 不构成组合验收，第二合入者
  必须在新整合基线重放全门禁并重新 review（774 interface-contract §6）。

## text-record 组合 profile（PLAN-782）

- [source-text-record.md](source-text-record.md)：拥有 Str 字段 record 的
  首批有限组合合同——派生资源分类（传递闭包）、完整根显式 move/深 clone、
  拥有形态 allowlist（let/assign/return/ctor 四位）、字段视图与替换（字段
  写永不复活已移出根）、770 清理不变式全套扩展至资源 record 根、统一
  跨类型 LIFO、短路聚合临时跨嵌套存在标记、故障双通道（CLI 测试面注入
  既有 runtime 控制符号 + 组合域部分输出 trap 前回滚）。
- source/HIR→native 闭环经 25 步联合门禁 11 例与 90 测试实证
  （V-01..V-12，含 5 个真实故障注入 oracle）；runtime/text 23 符号零新增。
- 770 gate 的 per-plan scope 步在 successor 树按**归因裁定**消费：782
  gate 实跑其全程，断言唯一 [FAIL] 即该步、被标集恰为本计划域内 artifact
  且 ⊆782 允许清单（真实越界仍红）。

## 数据/资源合同（PLAN-774）

- [data-resource-contract.md](data-resource-contract.md)：文本/标量聚合首批的
  共同类型（i32/char/内部宽度）、文本规则（UTF-8/标量索引/字面量与转义）、
  str/Str 逐语境、受限借/拥有前门（8 码）、内部 ABI 五类形态、资源合同
 （完整账本/u64 全精度计数/预算/失败分类/trap 边界）——经六轮独立复审收敛，
  770/771 以其为唯一接口来源（interface-contract.md 为消费面明细）。
- 探针 workspace：`experimental/ac-core/probes/data-resources/`（独立 Cargo
  workspace + no_std shim staticlib，64 探针测试 + 永久记账状态探针；一键门禁
  `scripts/verify-ac-data-resources-774.ps1`）。**非生产 ABI、非语言 feature
  支持声明**；text（str/Str/char）与标量 record 的共同合同已冻结、实现层仍
  open（List 未冻结，仍为后续 owner）。
- char/str/List/完整 int 宽度的语言实现状态仍 open（合同≠实现，双态表述）。

## 边界与已知限制

- 不宣称：完整 Auto 源码编译（仅两个独立 source profile，旧 AST 未接）、生产 ABI 冻结、DLL/热重载、Linux 发布验收、
  AAC 自举、性能承诺。
- 标量扩面的 MIN%-1 trap70 是 scalar-i32 profile 固定测试语义，不推导全语言/
  生产异常 ABI；char/str/List/完整 int 宽度仍 open。
- 单模块 lowering；跨模块调用/动态分派不在本层。
- 工具链发现依赖本机状态（注册表/env），CI 化需先固化 SDK 路径策略。

## 相关 plan

见 [plans.md](plans.md)。
