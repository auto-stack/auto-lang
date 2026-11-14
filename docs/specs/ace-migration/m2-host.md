# ace-migration: m2-host — C/Rust FFI、自动binding与通用宿主资源

> Status: F0 早期切片已验（PLAN-967 T-F0-I/V/R，2026-11-14 落主）；完整 C/Rust importer、
> 四端/双向 ABI/异步端口、精确缓存与真实库（PLAN-967 T-09..15）**未交付**，本文件不把它们
> 标为 implemented。来源：[PLAN-967](../../plans/967-ace-m2-host.md) §5 F0 + T-F0-I 冻结决定、
> 审计 f5bf3cd50→d77e369b2（§8 T-F0-R）、Design56 §0.1。

## F0 已验接口（AV 真调 Rust 的通用切片）

单一导入输入 = 带版本化注解的 Rust provider 源文件本身；任何 per-function 手写 manifest
均不存在。

| 面 | 接口/事实 | 来源 |
|---|---|---|
| 导入器 | `auto_bindgen::rust_import::import/generate_shim/to_manual_manifest`（格式 `auto.bindgen.rust-provider/1`；注解 `auto-ffi provider/fn`；域外具名拒绝 `rust-import.*`） | `crates/auto-bindgen/src/rust_import.rs` |
| 冻结 provider | `f0_text`：`text_byte_len`/`text_is_ascii`/`text_first_byte`(fallible `empty-input:-1`)/`text_panic_probe` | `experimental/ace/crates/abi/rustside/f0_text.rs` |
| 构建单元 | provider+生成 shim 平铺一个 rustc `cdylib` 闭包（记录 rustc 身份）；`LoadLibraryW/GetProcAddress` 装载并解析导出 | `experimental/ace/crates/abi/src/rust_provider.rs` |
| 物理帧（版本化） | `extern "C" (data: *const u8, byte_len: u64) -> i32|u8`；fallible `+ out: *mut i32 -> i32`，`None`→最小命名负 status；bool=0/1 | 生成 shim（rust_import） |
| panic 通道 | shim 单元内 `catch_unwind` + 导出 `f0_panic_flag()`；宿主每调用后观察并清零→独立 host 失败（`foreign.panic`）；**任何 unwind 都不跨 C 边界**（跨 DLL unwind 宿主不可 catch，实测冻结） | shim 生成 + `av-core/foreign_exec.rs` |
| AV stub 表 | `ForeignStubTable`：拒绝序=身份(unknown-symbol/signature，含 form/ABI 形匹配)→generation→UTF-8→call-bytes(65536)→run-bytes(1048576，每 run 计费输入字节)→reentry，全部在 callee 前；`callee_count` 只计真实调用；`bump_generation` 全表失效 | `experimental/ace/crates/av-core/src/foreign_exec.rs` |
| AV 执行 | `av::execute_foreign`（每 run `authorize_module`+run_bytes 复位）；`abc_verify::verify_abc_foreign`（无表默认拒绝，938 语义不变）；fallible status≠0 → `Trap{ForeignStatus}`（命名表在 manifest error model） | `av-core/src/{av,abc_verify,contract}.rs` |
| 源前门 | `use.rs <provider>`（CST `use-rs-item`，仅 text-record profile；`source.use-rs.syntax/domain/duplicate/provider`；legacy oracle 一致） | `grammars/auto-source.at` + `core/src/source/{cst/project,parser,resolve}.rs` |
| 永久验证 | `ace-abi/tests/cases/rust_f0.rs`（5 E2E：真 cdylib、真 AV 调用、oracle、拒绝、预算、身份/重绑定）+ av-core foreign_exec 单测 + auto-bindgen 单测 | 冻结组 `cargo t` 受影响并集 |

边界（未交付，归原任务）：C importer/真实预处理（T-09/T-10）；Rust 完整公开元数据/泛型/
String/Vec/owned/future（T-11）；AC/A2R/A2C/A2PY 四端接入与 AXR0 腿（T-12）；双向 export/
callback 重入/generation+owner+签名授权账/异步完成端口（T-08/T-13，§0 的 GrantSet 维度）；
精确缓存/分发清单（T-14）；真实 SQLite/serde_json（T-15）；ProgramSnapshot/session 链接面
（969/E0）；跨 rustc/目标分发合同（B-02..06 的运行腿）。938/939 的 host 注入裸名调用道保持
（F0 前门为"存在即校验"的可用性前门，非强制门）。
