# PLAN-785 T-01 冻结矩阵（feature/调用/编码/ownership + 接线决策 + 反控输入）

日期：2026-10-16 | 基面：master `340ab95fd`（含 786 归档；786 仅动 `crates/auto-man/`，与本计划零重叠）
实测命令环境备注：本机 sccache 端口被 Hyper-V 保留段占用（os error 10013），feature 组合实测均以 `RUSTC_WRAPPER=` 绕行执行。

## 1. 调用入口与绑定 identity（改动前事实）

- catalog（`vm/native_catalog.rs:83-84`）：`(2925, NATIVE_CLIPBOARD_TEXT, shim_clipboard_text, "auto.clipboard.text")` / `(2926, NATIVE_CLIPBOARD_SET_TEXT, shim_clipboard_set_text, "auto.clipboard.set_text")`。**本计划不改 catalog**——名字/ID/contract/注册全保形。
- `.at` 调用形（实锚 `examples/capability-tests/clipboard-bridge/src/front/app.at:167,171`）：根作用域 `clipboard_text()` / `clipboard_set_text(text)`。a2r-std（`crates/a2r-std/src/`）无 clipboard 镜像声明——`.at` 面零触点。
- shim 实现位：`vm/native.rs:2044`（ui 臂 get）/ `:2053`（ui 臂 set）/ `:3052`、`:3058`（no-ui 错误臂）。

## 2. feature 可建性矩阵（今日实测，"原本可建"登记）

| 组合 | 原本可建 | 实测/证据 |
|---|---|---|
| default（`with-file-history`+`ui-iced` 超集，含 ui+ui-clipboard） | ✅ | 日常档全绿历史 |
| no-default（779 fixture consumer 形态，`default-features=false`） | ✅ | 779 门禁绿（`test/fixtures/modularity/native-provider-consumer`） |
| `{ui-clipboard}` 单开（无 ui） | ❌ | `cargo check --no-default-features --features ui-clipboard` → E0433 cannot find `ui` in `crate`（含两 clipboard shim 直引 `crate::ui::clipboard`） |
| `{ui, ui-clipboard}`（无 iced/interpreter） | ❌ | 8 个预存错（`ui::iced/session/dynamic` 缺失 + `log::set_boxed_logger` feature 缺）——**与 clipboard 无关**，非本计划引入 |

推论：clipboard 文本两函数"原本可建面"只有两端——ui-iced 全家桶（OS 语义）与 no-default（feature-disabled 错误）。不存在"ui-clipboard 局部 UI"形态；T-04"受支持宿主"按此边界登记，不外推。

## 3. 错误/值编码/生命周期契约（保持不变项）

- get（ui 臂）：`ui::clipboard::clipboard_get() -> Option<String>` → `unwrap_or_default()` → `vm.add_string(text.into_bytes())` + `vm.rc_push_str_idx(task, idx)`。String lane；OS 不可用=空串（非错误、非 raise）。
- set（ui 臂）：`pop_string_arg(task, vm)`（native.rs:1414——pop nv + StakeGuard + `get_string` + `from_utf8_lossy().into_owned()`，**owned String**，无 VM 索引外泄）→ `clipboard_set(&text) -> bool` → `push_nv(encode_bool)`。Bool lane。
- 缺席臂（no-ui build）：`VMError::RuntimeError("clipboard_text: the `ui-clipboard` feature is disabled")`（set 同形，函数名 `clipboard_set_text`）。错误文本**原样保留**——函数名即 2925/2926 定位面。
- 绑定 identity：provider 注入不 bump `binding_revision`、不撤 reference proof（779 纪律原文沿用）。

## 4. 781 热点重叠实测裁定（依赖重评）

计划 §4 原文："Rust生产接线等781最终验收合并、共享native/宿主热点释放后开始"。实测（2026-10-16）：

- plan-781-dev（未合入，HEAD 6f91f1809 + worktree 未提交 WIP：native_catalog/ffi::stdlib/plan781_budget_tests）在 `crates/` 的改动面：engine.rs（3 hunks：~L39 常量、~L3055 段循环 HTTP 取消探测、~L13024 intercept_error 帧语义）+ native_catalog.rs（1526/1527 追加表项）+ ffi/http_server.rs + ffi/http_transport.rs + ffi/stdlib.rs + ui/{aura_view_builder,iced/scroll_anchor,mcp_server,scroll} + ui_gen/vue.rs + trans/rust.rs + a2r-std/str.rs + 测试。
- 785 触面：`vm/host_capabilities.rs`、`vm/native.rs`、`vm/engine.rs`（仅 `AutoVM::new` ~L816 快照区 + ~L1240 实例 API 区）、`ui/clipboard_provider.rs`（新）、`ui/mod.rs`、`lib.rs`、`crates/auto/src/main.rs`。
- **重叠 = engine.rs 单文件，hunk 区域不相交**（781 触 39/3055/13024；785 触 816/1240）。native_catalog.rs 785 零改动。其余 785 文件 781 零触。
- 裁定：热点冲突风险实测为"同文件不相邻 hunk"（git 三方合并可自动解），非 776→781 当年"未提交改动正撞写面"形态。**准予开工**，登记残余义务：785 final review 前同步 master（若 781 已合入则吸收并复跑受影响门禁）；若届时 engine.rs 冲突，按 781 语义优先解冲突后复审受影响面。
- 783/786 与 785 触面零重叠（783 无 crates 改动；786 仅 auto-man）。

## 5. 接线决策（bounded wiring decision）

1. **trait 名**：`ClipboardTextProvider`（计划拟名核定，无冲突）。方法：`fn text(&self) -> Option<String>`（None=OS 不可用→空串）；`fn set_text(&self, text: &str) -> bool`。出参 owned String（V02"临时 owned String"）、入参 `&str` 纯 Rust 借用——接口零 VM 索引/零 UI 类型/零缓冲悬挂。
2. **装配**：完全镜像 779——`host_capabilities.rs` 增进程默认槽 4 API（set_/set_if_absent/process_default_/clear_）；`NativeInterface` 增实例槽（Clone 保留、new()=None）；`AutoVM::new` 构造期快照；`AutoVM::set_clipboard_text_provider`/`clipboard_text_provider`。
3. **shim 单体化**：删除 `#[cfg(feature="ui-clipboard")]`/`#[cfg(not(...))]` 双臂——单一函数体：provider 在→委托；缺席→**原 feature-disabled 错误文本**。效果：能力=注入。副作用登记：`{ui-clipboard}` 单开组合**新可建**（原 E0433 消失），语义=缺席错误（adapter 在 `mod ui` 内不可达），不宣称 OS 能力。
4. **UI adapter**：新文件 `ui/clipboard_provider.rs`（镜像 frame_provider.rs）：`ArboardClipboardProvider` 委托 `ui::clipboard::{clipboard_get, clipboard_set}` 现有实现（含既有测试不动）；`install_process_default()` = `set_..._if_absent`。
5. **宿主接线（T-04 全清单）**：779 frame 的 6 个入口全部加 clipboard 臂（`#[cfg(feature="ui-clipboard")]` 语句级门控）：lib.rs:453 统一 installer、lib.rs:5587（run_file_dynamic_ui_inner）、vm_bridge.rs:639/761、crates/auto/src/main.rs:963（原 `#[cfg(feature="ui-iced")]` 块内）、autovm_persistent.rs:89（经统一 installer 已覆盖）。
6. **不改动**：catalog/contract/names/IDs、`pop_string_arg`/`add_string`/`rc_push_str_idx`/`encode_bool` 编码路径、`ui/clipboard.rs` 本体、默认 feature 图。

## 6. 反控输入（review 共享重放集）

- R-A（串线反例）：两 VM 实例 A/B 各持不同 fake provider（"alpha-α🎉"/"beta-β"），`run_with_capture` 高层 + 低层编译链双活；set 侧 provider 记录收到的精确 UTF-8 参数并断言逐字节相等。
- R-B（ownership 反例）：相邻连续读写（set→get→再 set）、provider 返回临时构造 owned String、空串/中文/emoji 边界；VM 后续对返回文本的使用仍正确。
- R-C（缺席矩阵）：无 UI+无 provider→原 VMError 文本断言；provider 在但返回 None/false→空串/false；显式失败不计成功。
- R-D（优先矩阵）：显式默认不被兼容安装覆盖（if_absent）；空槽补缺；实例覆盖不污染进程默认；779 frame 历史控制全过。
- R-E（identity）：provider 注入前后 `binding_revision` 不变；NI Clone/merge 保留 provider；名字/ID 查找 2925/2926 与邻族（files 2934+/image 等）不撞。
