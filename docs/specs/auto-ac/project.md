# auto-ac

> **Status**: experimental
> 路径：`experimental/ac-core`（独立 Cargo workspace，bin `auto-ac-prototype`，"ac-probe" 为历史文档名） | 技术栈：Rust + Cranelift 0.126.2 + rust-lld

AC（计算核心 profile）首个原生 AOT 闭环：Checked HIR → Windows x64 COFF → PE 原生执行。
**"HIR → 原生执行"已完成；"Auto 源码 → 原生执行"与 AAC 自举未完成**，由 A1/B/C 后续计划承接。

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
- 模块内函数名即 COFF 符号：重名在 backend 阶段拒绝；intrinsic 按声明 symbol 文本作 import。

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

- `auto-ac-prototype check <file>`：bind+verify，诊断 `file:line:col: error[stage/code]`；退出码 0/1/2
  （成功/流水线拒绝/用法错）。
- `auto-ac-prototype build <file> --entry <DefId> --output <exe> [--capability N]... [--support-lib L]...`：
  产物 `.obj`/`.exe`/`.ac-link.txt` 收据落 `--output` 目录；**失败构建不覆盖既有成功制品**：
  exe/obj/收据全部先以进程唯一暂存名构建，发布前备份既有制品，任一步失败整体回滚
  （还原备份、清除已放置文件；SENTINEL 前置失败与发布阶段失败测试双锁定）。
- 全部子进程（工具发现 rustc/reg、链接 lld、运行 exe）走带硬截止时间的执行器；
  stdout/stderr 并发读排空，防子进程填满管道阻塞。
- 一键验证：`scripts/verify-ac-741.ps1`（§6 全命令 + CLI 实机构建运行 + 主仓门禁；PS 5.1 兼容，
  异步管道读防死锁；含 `cargo check --all-targets` 零 warning 健康检查）。

## 边界与已知限制

- 不宣称：Auto 源码编译（旧 AST adapter 未接）、生产 ABI 冻结、DLL/热重载、Linux 发布验收、
  AAC 自举、性能承诺。
- 单模块 lowering；跨模块调用/动态分派不在本层。
- 工具链发现依赖本机状态（注册表/env），CI 化需先固化 SDK 路径策略。

## 相关 plan

见 [plans.md](plans.md)。
