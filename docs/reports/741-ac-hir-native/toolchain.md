# PLAN-741 T-01 工具链探针收据（toolchain.md）

> 2026-10-04，worktree `D:/autostack/.wt/lang-741/auto-lang`，分支 `plan-741-dev`，
> 基线 commit `951b6c70ff596f79e464ba977139669f2d196821`（v0.6-dev 计划起草提交）。
> 本报告记录 T-01 实测结果；所有命令在实机执行，产物保留于
> `experimental/ac-core/probe-out/`（gitignore，不入库）。

## 1. Rust 工具链

- `rustc -Vv`：
  - rustc 1.98.1 (48a229cea 2026-09-01)
  - binary: rustc；host: **x86_64-pc-windows-msvc**
  - release: 1.98.1；LLVM version: 22.1.8
- `cargo -V`：cargo 1.98.1 (797e8a9bc 2026-08-05)
- sysroot：`C:\Users\zhaop\.rustup\toolchains\stable-x86_64-pc-windows-msvc`
- rust-lld（COFF 模式）：`C:\Users\zhaop\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\x86_64-pc-windows-msvc\bin\rust-lld.exe`
  （LLVM lld 家族，随 rustc 22.1.8 分发；`-flavor link` 进入 COFF 模式）
- 普通 PATH 无 lld-link；PATH 上的 `link.exe` 是 Git 的 coreutils link，
  **不是 MSVC 链接器**——脚本必须用绝对路径取 MSVC link.exe。

## 2. MSVC 与 Windows SDK

- VS2022 BuildTools（vswhere -latest 实测）：
  `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools`
- MSVC 工具集：`VC/Tools/MSVC/14.44.35207`
  - link.exe：`...\bin\Hostx64\x64\link.exe`，实测横幅 **14.44.35222.0**
  - lib/x64：`libcmt.lib`、`libvcruntime.lib` 在位
- Windows 10/11 SDK：注册表 `KitsRoot10 = D:\Windows Kits\10\`（**D 盘**，
  `C:\Program Files (x86)\Windows Kits` 仅有 8.1/NETFXSDK）
  - 版本 **10.0.26100.0**
  - `D:\Windows Kits\10\Lib\10.0.26100.0\um\x64\kernel32.lib` 在位
  - `D:\Windows Kits\10\Lib\10.0.26100.0\ucrt\x64\libucrt.lib` 在位
- rustc 自身的链接探测：`rustc probe741.rs && ./probe741.exe` 退出码 42 与源码一致
  ——MSVC+SDK 环境经 rustc 驱动真实可用。

## 3. 后端选型：Cranelift 0.126.2

- 依赖（experimental/ac-core/Cargo.toml，lock 已提交）：
  `cranelift-codegen/frontend/module/object 0.126.2`、`target-lexicon 0.13.5`、`anyhow 1.0.104`
- `cargo tree --locked` 全树仅 Cranelift 栈 + object 0.37.3 + 传递依赖；
  **无 auto-val / auto-lang / auto-atom / auto-man / 旧 UI 依赖**（AC-01 依赖面证据）。
- crates.io 访问经 rsproxy-sparse，fetch/lock 成功（网络面可用）。
- `cranelift-object` 的 `ObjectModule::emit()` 直接产出 COFF `.obj`（x86_64，
  WindowsFastcall 调用约定），满足“HIR → COFF”目标，不需要 LLVM 绑定。

## 4. 最小 COFF/link/call 探针（examples/toolchain_probe.rs）

探针生成两个真实编译函数：`probe_add(i32,i32)->i32`（iadd）与最小启动包装器
`ac_start()`（调用 probe_add(2,3) 后调 `ExitProcess`，import 自 kernel32），
`/entry:ac_start /subsystem:console` 链接为 PE 后运行。

| 链接器 | 命令要点 | 链接 | 运行退出码 |
|---|---|---|---|
| rust-lld（首选） | `rust-lld -flavor link /entry:ac_start /subsystem:console probe_min.obj /libpath:<SDK um x64> kernel32.lib` | OK | **5**（=2+3，PASS） |
| MSVC link.exe（备选） | 同参数，绝对路径调用 | OK | **5**（PASS） |

- 期望退出码 5 = probe_add(2,3) 经 ExitProcess 上行——证明 COFF 生成、
  导入解析、x64 调用约定与进程执行全链路真实工作。
- Git Bash 会把 `/nologo` 等改写成路径（LNK1181 假故障）；驱动脚本用
  PowerShell/直接 CreateProcess 传参不受影响（T-07 脚本已按 PowerShell 设计）。

## 5. 选型结论与计划约束对账

1. **Cranelift 能力成立**：Windows x64 COFF 发射 + rust-lld/link.exe 双链接 +
   原生执行均实测通过，无需修订合同（§3 允许的替代未触发）。
2. 首选链接驱动 = rust-lld（零 VS 环境初始化、路径随 sysroot 可预测）；
   MSVC link.exe 作为备选记录在案。
3. 溢出 trap 用 `ExitProcess(70)` 独立路径（T-04 实现），本探针不涉及。
4. 生产 ABI 不冻结：本探针的 `ac_start` 包装器是测试 profile 的最小启动形态，
   仅为 PLAN-741 §5.3 约定。
