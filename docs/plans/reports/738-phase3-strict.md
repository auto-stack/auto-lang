# PLAN-738 Phase 3 / T-10 收据：Rust strict 门修复与合法适配链证明

- 任务：T-10（修复 Rust strict 门并证明合法适配链）
- worktree：`D:/autostack/.wt/lang-738/auto-lang`，基线 `2c1b4a763`，本任务提交见 §5。
- 触面：`stdlib_assembly/host.rs`（重写）、`stdlib_assembly/validate.rs`（`normalize_host_type` 升 pub(crate)）、`trans/rust.rs`（Await 臂验证接线）、`tests/plan724_http_client_tests.rs`（探针改判，见 §3）。

## 1. 修复的放行缺口（rv3 §4.3 登记 → 本轮闭合）

| 旧缺口 | 旧行为 | 新行为 |
|---|---|---|
| 包装形状 async 豁免 | `producer_async != awaited && !wrapped` —— 凡识别为包装块即跳过 async 双向核对 | 每条契约声明 awaited/producer_async，发射与 producer 两面各自对拍；不匹配= `SIGNATURE_DRIFT` |
| 元数不匹配 Resolved-only 放行 | `wrapped \|\| mode_variant` 时元数漂移返回 `Resolved` 证明且发射继续成功 | Resolved-only 放行臂删除；元数漂移= `SIGNATURE_DRIFT`，核心引用拒绝即转译失败（不产出成功产物/成功生成收据） |
| Await 臂绕过验证 | `fn expr` 的 `await http.post_sync(...)` 臂直接发射侧信道壳，不经 `call()` 验证钩子 | 该臂先收集发射文本，走同一 `verify_rust_reference` 门，证明入 `assembly_references` |
| 类型名模式当证明 | `AsyncHTTPStream` 类型名 pattern / `(int,str)` 元组 pattern 直接充当适配证据 | 返回结构映射只来自 `ADAPTER_RULES` 契约声明（`ProducerReturn::{Scalar,StatusBodyTuple,AuthTuple4,Facade}`）；producer 实形与契约对拍，不再以类型名猜测 |
| 公共 AST 复制风险 | 三参 post 的公共 2 参签名与发射 3 参无对拍关系 | `check_public_projection`：公共声明参数/返回与适配投影逐位对拍；历史扩展位（api_key）由契约 `extension_params` 单独冻结并记入 error_shape，不与公共签名混淆 |

## 2. 新验证模型（四源一致）

发射文本 → `observe_emission` 结构化观察（形状/callee/元数/await/cast 目标/块内结构逐项核对）→ 按 `(module, symbol, shape, callee, runtime)` 查 `ADAPTER_RULES` → 契约驱动四面校验：

1. **发射 ↔ 契约**：元数、await、cast 目标（必须 int 面）、块结构（侧信道三段结构：绑定主调用/set_last_status(r.0)/尾取 r.1；元数分派四元解构 + HttpResponse shorthand 构造）。
2. **producer ↔ 契约**：`resolve_runtime_producer`（Standalone=a2r-std crate / Embedded=内嵌镜像，含 `pub use` 再导出链跟随）后核对 async、参数数、返回结构（tuple 逐元素逻辑型）。
3. **参数双源**：producer 逻辑参数面 == 公共参数面 + 契约扩展位（逐位相等）。
4. **公共声明 ↔ 适配投影**：参数/返回/元数（公共元数只对 public_params 部分）。

**规则表 12 条**（http 模块；R3-02 勘误：初稿误记 14——c1579ed71 实测 12）：post_sync/post_bearer/post_bearer_sync/get_sync 侧信道族（sync+async 变体）、last_status 数值 cast、3 参 post 元数分派（**Embedded 限定**——a2r-std crate 无此 producer）、get_stream/post_stream/post_stream_with_headers async facade。无契约的非 Plain 形状 → `SIGNASSEMBLY.SIGNATURE_UNVERIFIED: no declared adapter contract ...`。

Plain 形状保留通用严格路径（callee 解析、async 双向、元数、`signature_matches` 公共对拍、AsRef 视图/parse Null-sentinel 记录），同样无任何 Resolved-only 通道。

## 3. plan724 探针改判说明

`plan724_probe_post_arity_dispatch_shapes` 旧断言「3 参 post 在 Standalone 发射 HttpResponse 认证壳文本」。该壳调用 `a2r_std::http::post` 3 参——**a2r-std crate 只有 2 参 `post`**（embedded 镜像才有 3 参 async 面），该发射在 Standalone 从未可编译，旧断言冻结的是一个不可用形状。新断言：Standalone 下转译诚实拒绝（`SIGNATURE_UNVERIFIED ... arity-dispatch ... standalone runtime`）；合法用法不变——Standalone 认证走 post_sync/post_sync_async 侧信道面（golden + e2e 在案），Embedded 元数分派完整证明由 host 测试 `arity_dispatch_post_proves_only_under_embedded` 与 qualified-facade e2e 承担。724 冻结的可编译能力全部保留。

## 4. 测试与门禁

| 命令 | 结果 |
|---|---|
| `cargo t plan738` | 62/62（含 plan738_host 新 5 测：侧信道契约正负例/无契约包装拒绝/元数分派 Embedded 限定+元数漂移/cast 非 int 面拒绝/facade 契约+缺 await 漂移） |
| `cargo t plan724` | 5/5（双 golden 含 post_sync_async 壳/get_stream_async facade/last_status cast 全部经新门验证） |
| `cargo test -p auto-man --lib api_gen -- --test-threads=1` | 43/43（Embedded 生成链消费新门） |
| `cargo test -p auto-lang --lib --features test-http-e2e http_e2e_plan724 -- --test-threads=1` | 3/3（**Standalone 真编实跑**：build_and_run 编 a2r-std crate 断言 auth status 200/body；**Embedded**：内嵌 http::post 3 参 + post_sync/post_bearer_sync 可观察返回/状态断言） |
| `cargo nextest run -p auto-lang --lib --features test-trans --no-fail-fast`（tt 全档） | 5532 绿/15 红全分诊：11 文档化 master 预存（musk p053/p054×6、desktop_protocol、iced×2、ash_leak、plan606_029）+ 4 负载 flake（plan484_024、plan502、plan707_client、plan730_commit_target_matrix——后者隔离复跑绿）。**零新增确定性红** |
| `cargo t use_semantics` / `cargo t module_cache` | 7/7、16/16 |
| `cargo check -p auto-lang` | 0 error；触面文件 rustfmt 干净 |

## 5. 提交

- worktree `plan-738-dev`：`c1579ed71`（`fix(stdlib): PLAN-738 T-10 Rust strict 门——适配契约注册表四源一致验证，删除包装豁免与 Resolved-only 放行 (Plan 738)`，4 files +1248/-211，基线 `2c1b4a763`）；本报告与计划勾选在主检出另行簿记提交。
- 映射：T-03/04/07、AC-02/03/05/07/08、SD-01/02/04/05/07。
