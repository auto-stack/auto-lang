# ACE M2 Build — 工程增量多目标构建与预编译候选 SDK（PLAN-978）

状态：work 阶段交付稿（待独立 review 核验后定稿）。来源计划：
[978](../../plans/978-ace-m2-build.md)（r14 合同）。本文件是该包的规范落点
（SD-01/SD-PHASE/SD-HELLO/SD-NATIVE/SD-DETAIL/SD-ID-BOUNDARY/SD-PLATFORM-IO
共同目标；SD-LINUX-PHASE 见 §7）。

## 1. 能力边界（已实现并验证）

`experimental/ace/crates/m2-build`（包 `ace-m2-build`）+ CLI `ac-build`：

- 一套工程构建身份/缓存/装配/恢复层，五目标：`av | native | a2r | a2c | a2py`。
- 源码构建 lane：AV 进程内执行（值观测）；native 进程内 cranelift+rust-lld 真
  PE 构建+进程运行；a2r/a2c/a2py 文件集 emit（真编译属显式 build-lane：
  rustc/clang/cl 为构建期工具，禁入日常测试体——2026-11-14 测试裁定）。
- 预编译消费 lane（`ac-build run --sdk`）：先 selection 准入→runtime 兼容核对
  →再 dispatch；AV 进程内、native/a2r 走 SDK 内预编译 exe、a2py 经声明解释器
  执行生成的 `a2x_main.py` runner；缺预编译产物=具名拒绝
  `m2build.run.requires-build-lane`，不以 emit 冒 run。
- 候选 SDK：单根可重定位目录（`install-manifest.json` + `sources/` +
  `targets/` + `providers/` + 可选 `stdlib/auto/`），全部相对路径、逐工件
  sha256 校验、未声明文件/绝对路径/`..` 逃逸均具名拒绝。

## 2. 身份分层（ID-01..03）

| 层 | 身份 | owner |
|---|---|---|
| 语义键 | 既有 AXS*（不动） | a2x-store |
| 目标片段 | 既有 AXR*/AXC*/AXPY*（不动） | a2x-store |
| provider 构建 | 967 S967B `BuildIdentity`（原样消费） | ace-abi |
| 工程构建节点 | `ace-m2-build/project-build/1`（M978B 层，可再生缓存） | m2-build |

工程身份轴均为真实读：project/entry/caps/排序源(path,role,sha256)/stdlib 闭包/
grammar 轴（各 lane 自报）/工具轴（仅溯源，不作运行准入）/provider_build。
改 Auto body 只翻对应节点；未变 target 与 967 provider 缓存不受影响。

## 3. 运行兼容与选择（ID-02..04）

`RuntimeCompatibility`（`ace-m2-build/runtime-compat/1`）：consumer required 与
provider offer 在 OS/arch/env/位宽/端序/ABI域/接口schema/required 符号→签名哈希/
语义 owner/线程/运行依赖逐轴核对；**rustc 身份不参与**（不同 rustc 构建的兼容
provider 不误拒；同 rustc 标签不构成放行）。不兼容=dispatch 前具名拒绝
`m2build.compat.<axis>`。

`ProviderSelection`（`ace-m2-build/provider-selection/1`）：provider id/代次/
工件 sha 精确 pin/当前授权 grant。新工件哈希（required 子集完好）不自动升级
pin——须显式重 pin 后重新准入；授权逐次核验，恢复不继承历史授权。

## 4. 增量、CURRENT 与恢复

- 精确缓存：同身份命中=零生产零 spawn；身份漂移=按 lane 重产。
- CURRENT 发布：`replace_current_root` CAS；并发写败者具名 Boundary，无静默
  覆盖/丢更新。
- 失败保旧：坏收据行（外域 schema）具名拒绝 `m2build.store.poisoned-row`，
  旧有效身份继续命中；`restore` = 783 recover + M978B 可再生层 purge + CURRENT
  逐目标复验 live + 后续重建（恢复读回修复）。
- 进程 oracle：驱动 spawn seam 计数 + 归因进程快照（按父链归属本进程；
  cargo/rustc/bindgen/clang）。Auto body 编辑/重复运行的零工具进程断言用归因
  oracle，外部并发构建不误报。

## 5. 分发账目（ID-05..06，967→978→986）

recipe 即交接文档：每 target 一条 provenance 身份；构建工具只在
`build_inputs`（溯源，非运行需求）；运行闭包独立列（av-host/crt/解释器/
预编译程序）；provider 选择带 pin/授权映射。真实安装/升级/退役归 986/987。

## 6. 平台 I/O（Windows 腿）

native 产物 PE import 表由测试内 PE 头解析实测，所有导入 DLL 必须落在声明的
运行依赖族（kernel32/ucrt/vcruntime/api-ms-win-crt/msvcrt）。

## 7. Linux 移交（2026-11-15 裁定，r14）

本包 Linux 编译/运行/恢复验证腿全部 `deferred-required / NOT_RUN`，
owner=PLAN-1063；Windows 原 required 门不减；交叉编译成功或 Windows 仓外运行
不作为 Linux 支持证据。

## 8. 固定组

`ace-m2-build`（nextest `-p ace-m2-build --lib --tests`，
`scripts/test-groups.toml` 注册）；p978 永久用例分布于
`crates/m2-build/tests/{b0_closure,identity_cache,compat_selection,sdk_pack,restore_recovery,coverage_lanes,load_selection}.rs`。

## 附：PLAN-1107 追加（use.py 工程的 AV/A2PY 构建、缓存与分发）

> 本节追加到 `docs/specs/ace-migration/m2-build.md`，其他五目标的既有义务保持。

1. **车道选择**：项目源含 `use.py` 时，A2py 车道用 `cold_assemble_py`（Aggregates 单元 +
   中立 py 声明契约，自动补齐 `foreign.scalar`/`foreign.resource`，不留 retained env）；
   不含 `use.py` 的普通工程仍走冻结 Scalar cold 路径（无 py helper、无 ctypes provider）。
2. **构建计数（真实，非自报）**：冷构建 `rebuilt=true` 且 `fragments_produced>0`；
   同输入暖构建 `rebuilt=false`、`lane="cache-hit"`、零 `tool_spawns`；改 body 使
   `BuildIdentity` 变更并重产片段。
3. **身份轴**：`ProjectBuildIdentity` 覆盖 project 名、code caps、源/标准库内容、grammar
   identity、tool axes 与 participating provider artifact；provider 工件只在参与节点
   as-is 消费，不静默改写其他身份。
4. **运行与分发**：`build_sdk` 打包可重定位 SDK，`run_sdk` 用声明的 `python3 -B` 执行
   `a2x_main.py`；缺解释器 → `m2build.run.interpreter-failed`，缺声明包 →
   `m2build.run.runtime-dependency`（具名，不把 traceback 计为运行）；emit 从不执行程序。
5. **非 Python 回归**：既有五目标行为不变；A2py 的非 Python 工程不受 py 车道影响
   （`p1107_v08_…`）。证据：`crates/m2-build/tests/py_project_1107.rs`。
