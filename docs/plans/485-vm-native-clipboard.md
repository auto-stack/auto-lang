---
plan_id: PLAN-485
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-native-clipboard
author: [zhaopuming]
created_at: 2026-08-30
updated_at: 2026-08-30

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-lang/vm]
current_step: 0
total_steps: 9
---

# [PLAN-485] 原生剪贴板互通 Phase 2——文件与图片（CF_HDROP / DIB）

## 变更摘要

补齐虚拟桌面 App ↔ OS 剪贴板的**文件与图片**两族格式互通（473 原生互操作路线
的 Phase 2）。文本双向已通（Plan 418 `auto.clipboard.get/set_text` + 481
selectable 文字 Ctrl+C + iced input 粘贴），缺口是：

- **文件（CF_HDROP）**：Explorer Ctrl+C → 桌面 App 粘贴取路径；桌面 App 复制
  文件路径 → Explorer Ctrl+V（用例 A3 文件复制半边）。
- **图片（CF_DIBV5/PNG）**：截图工具/浏览器复制 → 桌面 App 取图显示（A7 半边）；
  桌面 App 图片写回剪贴板。

实现：新 `ui/clipboard_native.rs`（`cfg(windows)` + 新 feature `native-clipboard`，
windows crate 访问 Win32 剪贴板），四个新 VM natives
`auto.clipboard.files_get/files_set/image_get/image_set`（418 三件套同型：
native_catalog + native.rs shim），新示例 `examples/ui/043-clipboard-bridge`
实机演示三族往返；**rider**：顺带清偿 P481-6（481 遗留的实机 Ctrl+C→系统
剪贴板末步复验）。拖放（A1/A2/A5）仍是 Phase 3，不在本期。

## 目标

- **G1**：`auto.clipboard.files_get() -> [string]`（Explorer 复制后取回路径列表）
  / `auto.clipboard.files_set(paths) -> bool`（Explorer Ctrl+V 落地）。
- **G2**：`auto.clipboard.image_get() -> {path,width,height}|None`（DIBV5→PNG 存
  temp 返回）/ `auto.clipboard.image_set(path) -> bool`。
- **G3**：非 Windows / 未开 feature 时优雅降级（files 空表、set/image 返回
  false/None），.at 代码零条件分支可写。
- **G4**：示例 `043-clipboard-bridge`：text/files/image 三卡 get 显示 + set 写回，
  VM 桌面实机可演示 Explorer 往返。
- **G5**：清偿 P481-6：001/004 实机 Ctrl+C→notepad 粘贴复验，结果回写
  KNOWN-DEBT。
- **非目标**：OLE 拖放与虚拟文件延迟渲染（Phase 3）；剪贴板监视/历史；桌面级
  全局 Ctrl+V 自动路由（App 以按钮/热键显式调 native）；vue/web 远程端（无原生
  剪贴板场景，降级语义文档化即止）；440 file-manager 的消费侧 UX（后续计划吃
  本期 natives）。

## 架构方案

```
.at App  ──auto.clipboard.files_get/files_set/image_get/image_set──▶  vm/native.rs shim
                                                                        │
                     ┌──────────────────────────────────────────────────┤
                     ▼                                                  ▼
        ui/clipboard.rs（既有，arboard 文本跨平台）        ui/clipboard_native.rs（新）
                                                           cfg(windows)+feature
                                                           native-clipboard
                                                           CF_HDROP / CF_DIBV5 / "PNG"
```

- **层次**：VM natives → shim → ui 层桥；Win32 只出现在 clipboard_native.rs
  （与 native_dock/win32.rs 同纪律）。
- **feature 门控**：`native-clipboard = ["dep:windows"]`（Cargo.toml，对齐 473
  的 `native-dock` 双门控形态：optional dep + cfg(windows)）。
- **图片形态**：进出统一走 PNG 临时文件（对齐 image widget 的 path 消费形态），
  DIB↔像素转换用既有 image crate（ui-iced feature 已带）。

## 技术栈

windows crate（Win32_System_DataExchange 剪贴板、Win32_UI_Shell 的
DragQueryFileW、GlobalAlloc/GlobalLock 内存块）、image crate（PNG↔BGRA）。
零新三方依赖。

## 需求分析与背景调查

（取材 docs/specs/overview.md §ui/vm + 现场核验 2026-08-30）

- **桌面线现状**：473（native dock P1）/479/480/481/482/483 全部复审合入归档；
  在途仅 484（charts 线，非桌面）。473 阶段表（§架构方案 Phase 路线）明确
  Phase 2 = 剪贴板双向（A3/A4/A7/D5 用例）——本期兑现其中文件/图片两族。
- **文本已通的证据**：Plan 418 `ui/clipboard.rs`（arboard，get/set_text）+
  `vm/native_catalog.rs:44`（`auto.clipboard.set_text` 2926 段）+ native.rs
  shim 先例（native.rs:768 起 code_editor clipboard 族，feature 双门控）；
  Plan 481 落了 selectable 文字与 Ctrl+C（P481-6 实机末步待复验=rider）。
- **Win32 门控先例**：473 落的 `native-dock` feature（crates/auto-lang/
  Cargo.toml:67 注释、186-188 target.'cfg(windows)' 段）——`native-clipboard`
  完全同型，windows crate 依赖复用不重引。
- **消费侧**：Plan 440（file-manager，已立项未开工）是 files natives 的天然
  大户；本期以 043 示例承担可见消费面，440 后续直接吃。
- **vue/远程端定位**：无原生窗口场景，natives 降级返回空值；不做 web polyfill
  （Design 23 Web 形态走 in-page，不涉及 OS 剪贴板桥）。
- **风险**：剪贴板为进程全局资源，集成测试须 set→get 即时往返并防并发污染
  （CI 无头环境 arboard/Win32 失败的 guard 先例 = 418 测试）。

## 详细设计

### 1. ui/clipboard_native.rs（新，cfg(windows) + feature native-clipboard）

- `clipboard_files_get() -> Vec<String>`：OpenClipboard → IsClipboardFormatAvailable
  (CF_HDROP) → GetClipboardData → GlobalLock → DragQueryFileW 计数+逐条取宽串；
  任一步失败返回空 Vec。
- `clipboard_files_set(paths: &[PathBuf]) -> bool`：GlobalAlloc(GMEM_MOVEABLE)
  构造 DROPFILES（pFiles 偏移+双 NUL 结尾宽串列表）→ SetClipboardData；
  空列表 false。
- `clipboard_image_get() -> Option<TempImage>`：格式优先级 CF_DIBV5 → CF_DIB →
  registered "PNG"；BITMAPV5HEADER 解析（自下而上行序、BGRA→RGBA、stride
  对齐）→ image crate 编 PNG 写 std::env::temp_dir()；PNG 格式直接落盘。
  返回 `{ path, width, height }`。
- `clipboard_image_set(png_path) -> bool`：image 解码 → 构造 BITMAPV5HEADER +
  自下而上 BGRA 像素 → SetClipboardData(CF_DIBV5)（并同时挂 "PNG" registered
  format，兼容只认 PNG 的接收方）。

### 2. feature 与 Cargo.toml

- `[features] native-clipboard = ["dep:windows"]`；windows features 增补
  Win32_System_DataExchange / Win32_UI_Shell / Win32_System_Memory（在 473 既有
  dep 条目上扩 features 列表，不新增依赖行——若 473 的 windows dep 条目 feature
  列表是共享的，扩列需注明两 feature 共用）。
- 非 Windows：模块整体 cfg 掉，shim 层提供降级实现（见 §3）。

### 3. VM natives 三件套

- `vm/native_catalog.rs`：按 2926 邻段顺号注册四个：
  `auto.clipboard.files_get` / `files_set` / `image_get` / `image_set`。
- `vm/native.rs` shim：字符串数组/路径/记录(Record) 进出（image_get 返回
  `{path,width,height}` Record——按 native 层既有 Record 构造先例）；
  `#[cfg(not(all(windows, feature = "native-clipboard")))]` 臂返回
  `[] / false / None / false`。

### 4. 示例 examples/ui/043-clipboard-bridge

- 三卡布局：文本（get/set，对照 418）/ 文件（get 列路径、set 从示例目录挑
  1-2 个真实文件写回）/ 图片（get 显示缩略+尺寸、set 把示例自带 PNG 写回）。
- 按钮显式触发（本期不做全局 Ctrl+V 路由，非目标已注明）。
- 自带资产：`assets/demo.png`、`assets/hello.txt`（示例可复制源）。

### 5. rider：P481-6 清偿

实机走 001-helloworld：拖选文字 → Ctrl+C → notepad Ctrl+V 出字；结果一行回写
`docs/plans/KNOWN-DEBT-AND-RISKS.md` P481-6 条目（✅ 已清偿 + 日期）。

## 测试设计

1. **T1 纯单元**（全平台）：DROPFILES blob 构造/解析往返（合成字节，含中文
  路径宽串）；BITMAPV5HEADER↔RGBA 转换（合成 2×3 像素含 alpha，行序/stride
  断言）。
2. **T2 Windows 集成**（feature 门控 + headless guard，418 同款）：files
  set→get 往返；image set→get 往返（像素容差断言）。
3. **T3 natives 层**：VM 侧调用四 natives（空剪贴板 → 空表/None/false；
  有剪贴板环境 → 往返），非 Windows 降级臂单测。
4. **T4 手动冒烟**：Explorer Ctrl+C → 043 files_get 显示；043 files_set →
  Explorer Ctrl+V 落地；截图工具 → image_get 显示；P481-6 复验。结果逐行
   记入 §验收标准下。

## 验收标准

1. 四 natives 注册齐且 VM 可调；非 Windows/未开 feature 降级语义符合 G3。
2. 043-clipboard-bridge 在 VM 桌面实机三卡可演示（T4 留痕），Explorer 文件
   往返成功。
3. T1/T2/T3 自动化绿（`cargo t clipboard` 档）；schema 三件套不回归（本期无
   schema 改动）；`cargo check -p auto-lang` 零警告。
4. P481-6 回写 KNOWN-DEBT 为已清偿。
5. `cargo t ui`、`cargo t vm` 不回归。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **feature 门控 + files 桥**：`crates/auto-lang/Cargo.toml` 增
   `native-clipboard = ["dep:windows"]`（windows dep features 扩 DataExchange/
   Shell/Memory）；新建 `crates/auto-lang/src/ui/clipboard_native.rs`：
   files_get/files_set + DROPFILES 构造/解析抽为纯函数。
   验证：`cargo check -p auto-lang --features native-clipboard && cargo t clipboard_native`。
2. **files 集成往返**：T2 files 用例（headless guard）。
   验证：`cargo test -p auto-lang --features native-clipboard clipboard_files`。
3. **image_get**：DIBV5/DIB/PNG 三退路 + RGBA 转换纯函数 + temp PNG 落盘 +
   T1/T2 用例。
   验证：`cargo test -p auto-lang --features native-clipboard clipboard_image`。
4. **image_set**：PNG→DIBV5(+registered PNG 双挂) + 往返用例。
   验证：`cargo test -p auto-lang --features native-clipboard clipboard_image`。
5. **natives 三件套**：`vm/native_catalog.rs` 顺号注册 + `vm/native.rs` shim
   （含非 Windows 降级臂）+ T3 用例。
   验证：`cargo t clipboard && cargo check -p auto-lang`（默认 feature 编译过=降级臂在）。
6. **示例 043**：`examples/ui/043-clipboard-bridge/`（pac.at + src + assets/
   demo.png、hello.txt）。
   验证：`auto run examples/ui/043-clipboard-bridge -r vm` 实机起跑 + 手动三卡
   一轮（截图留痕）。
7. **T4 手动冒烟 + P481-6 rider**：按 §测试设计 T4 执行；P481-6 结果回写
   `docs/plans/KNOWN-DEBT-AND-RISKS.md`。
   验证：清单每项 PASS 注记 + 债务条目更新。
8. **文档化降级语义**：`ui/clipboard_native.rs` 模块头注（vue/远程端不适用、
   降级返回值约定）+ 043 README。
   验证：`cargo check -p auto-lang` 零警告。
9. **收尾**：`cargo t ui`、`cargo t vm` 不回归；无调试打印；状态翻
   execution_done。
   验证：`cargo t ui && cargo t vm`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- native ID 具体编号以 `vm/native_catalog.rs` 当前空位为准（2926 邻段顺延）。
- CF_DIB（无 V5 头的旧格式）覆盖面：截图工具/现代浏览器均写 DIBV5 或 PNG，
  纯 CF_DIB 退路仅做头解析尽力而为，不做 BITMAPINFOHEADER 全变体（RLE 压缩
  等罕见变体直接 None）——复审时确认接受度。
- image_get 大图阈值：>64MP 像素直接 None（防误爆内存），数值在实现时定稿。
- windows dep 的 features 列表若与 native-dock 共用一条目，扩列方式（合并
  features vs 拆两条目）以编译最小扰动为准，T1 时定。
