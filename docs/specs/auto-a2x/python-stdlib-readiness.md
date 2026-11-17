# auto-a2x-python 标准库后端与运行端口规约（PLAN-1042）

> **Status**: active（M2-A 标准库 Python 目标基础设施就绪） | 依据：[auto-a2x project](project.md)、[architecture](architecture.md)、PLAN-1042

本规约记录经 Checked HIR → 共享 `a2x-core` → `ace-a2x-python`（`TargetProvider`）→ 版本化运行 helper（`a2x_py_runtime.py`）→ 真实 OS Python 执行的标准库前置基础设施契约与显式边界。

## 1. 身份与分层契约

- **Crate 与隔离**：`experimental/ace/crates/a2x-python/`（包名 `ace-a2x-python`），只依赖 `ace-a2x-core` 与 `ace-core`；`ace-a2x-store` 保持目标中立，不静态依赖 `ace-a2x-python`（由 `boundaries` 机械断言）。
- **Provider 与生产者身份**：
  - `PY_BACKEND = "auto-a2x-python"`，`PY_STANDARD = "python3.11+"`
  - `PY_PRODUCER = "ace-a2x-python/modules/1"`（注册于 `semantics-contract::assembly::PYTHON_PRODUCER`）
- **Store 分层锚**：`AXPY1`（Scalar）、`AXPY2`（RecordText）、`AXPY3`（Collections），与 `AXR1..3`/`AXC1..3` 及 `AXS1..3` 完全隔离；支持 `RecordText` 与 `Collections` 的 `anchors_family` 及失败发布隔离（失格编排/校验失败不覆盖既有 `manifest.json` / store 状态）。
- **TestCase 与 Worker 通道**：
  - Emit recipes（spawn-zero，`execution_claimed == false`）：`ac-a2py/hir-scalar-emit-v1`、`ac-a2py/hir-record-text-emit-v1`、`ac-a2py/hir-collections-emit-v1`（worker op: `daily-emit-py`）。
  - Execution recipe（真实 OS Python，`spawned_python == 1`，`execution_claimed == true`，必携 `python_identity`）：`ac-a2py/hir-stdlib-run-v1`（worker op: `run-py`，以 `python -I -B a2x_main.py` 在隔离模式运行）。

## 2. 运行时值、文本、错误与显式资源契约

- **数值与控制流语义**：标量算术显式按 i32 范围检查与 C 截断除法/取余（`a2x_check_i32`、`a2x_c_trunc_div`、`a2x_c_trunc_rem`）；短路逻辑按惰性求值序降维；除零与溢出稳定触发 `A2xTrap`。
- **UTF-8 / 内嵌 NUL 与编码**：字符串/文本端口通过 `_decode_utf8_strict` 严格校验 UTF-8（非法字节稳定抛出 `A2xTrap("InvalidScalar", ...)`，禁止静默 lossy 替换）；bytes 与文本保真支持多字节与内嵌 `\x00`（如 `b"ab\x00cd"`、`"A中😀"`）。
- **确定性显式清理（零 GC 兜底）**：`A2xOwnedStr` 与 `A2xResourceHandle` 使用 `__slots__` 且禁止依赖 `__del__` 释放；生成的 `finally:` 块与句柄计数显式追踪分配/释放，重复关闭触发 `DoubleClose`，关闭后读取触发 `ResourceClosed`。
- **标准操作端口绑定**：`ports.rs` 与 `stdlib/contracts/operations.toml` 对齐，调用点经 `a2x_port_call` 校验端口版本、参数方向与依赖 digest，helper 严禁硬编码被测结果。

## 3. 准入否定控制与未交域边界

- **严禁旧栈兜底**：`use_legacy_python_trans`（`crates/auto-lang-core/src/trans/python.rs`）、`use_vm`（AutoVM 回退）、`emit_only` 冒充执行、或向 worker 注入 `expected` 预言均被 worker 与 `ac_a2py_hir` 适配器显式拒绝。
- **基础设施缺失不可变绿**：缺 OS Python、错误 `producer`、或篡改 `manifest.json` / 文件 sha256 一律返回可控错误。
- **显式未交域与后继接口**：
  - 完整 Python 语言与存量语料扩面归 `PLAN-977/PY`（复用本 `ace-a2x-python` 与 `AXPY1..3`，不复制后端）。
  - `#in` 源码与标准库装载归 `PLAN-1043`；Python→Auto 嵌入归 `PLAN-1026`；C/Rust FFI provider 归 `PLAN-967`。
  - Linux 宿主机真实 Python runner 取证依赖 Linux 执行环境（当前仅于 Windows `Python 3.14.2` 实机取证）。

## 父侧有界捕获与响应分域（PLAN-1034）

- 两个 Py 父入口（emit/run）接同一 opt-in 有界捕获（`run_bounded`）：stdin 请求、exec 70s/
  单一 cleanup 3s、stderr 1MiB；emit 响应元数据 cap 4MiB，run-py 原始响应 cap 20MiB。
- run-py 响应分域（解码前拒超量）：每流 `stdout_hex`/`stderr_hex` 文本 ≤2MiB 且解码后每流
  ≤1MiB（原始 bytes 是 oracle，lossy 显示文本不是）；去除四个流字段后的同一 JSON 编码 ≤4MiB
  （非流元数据）。hex 2MiB 门在算术上蕴含解码 1MiB 门，双门均为合同。
- 显式执行与身份保持：run recipe 保 `spawned_python==1`、`execution_claimed==true`、完整
  `python_identity`；emit 保 spawn0/`execution_claimed==false`；原 30s 执行腿与程序流
  PY_STREAM_CAP=1MiB 不变。
- 版本探针有界：`resolve_python` 走受管 capped runner，全部候选共享一次 30s 探测期限、每流
  1MiB；解释器内容 hash 流式计算（不整缓存 binary），读取失败具名 Infra，不以路径字符串 hash
  兜底。平台准入不变（非 Windows 受管 runner 拒绝即拒绝）。

## PLAN-1035 增量：作为共享分类事实的 consumer（2026-11-16）

- Python 目标只消费 core verifier 的 Collections/资源事实与参数模式事实；资源 record
  闭包分类修正（List-only record 归资源）经 `ace-a2x` 组（含 `ace-a2x-python`）回归
  验证，未改变本文件的准入、句柄/`finally` 显式清理与 `A2xOwnedStr`/
  `A2xResourceHandle` 契约，也未引入 GC/`__del__` 兜底或复制 ownership 分析。
- 证据锚：`ace-a2x` 固定组（含 `ace-a2x-python`）在 PLAN-1035 冻结提交上的全绿结果。
