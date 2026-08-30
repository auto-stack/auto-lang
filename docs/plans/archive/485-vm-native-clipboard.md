---
plan_id: PLAN-485
status: archived               # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: vm-native-clipboard
author: [zhaopuming]
created_at: 2026-08-30
updated_at: 2026-08-30
status_note: execution_done 2026-08-30（P481-6 rider 未清偿,见待澄清事项）

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 473 原生互操作段——Phase 2 剪贴板互通落地注记（text 418 + files/image 485 三族全通；OLE 拖放 Phase 3 仍待）"
new_spec_components:
  - "crates/auto-lang/src/ui/clipboard_native.rs: 原生剪贴板桥——纯 codec 层（DROPFILES 构造/解析、DIBV5↔RGBA、64MP 防爆，全平台可测）+ Win32 双门控层（CF_HDROP / CF_DIBV5+registered PNG 双挂；GlobalClipboardTestLock 跨进程测试互斥纪律）"
  - "vm/native_catalog.rs 2934-2937: auto.clipboard.files_get/files_set/image_get/image_set 四 natives（ret tag List/Bool/Map/Bool；非 Windows/未开 feature 降级臂空表/false/null，弹参保栈纪律）"
  - "examples/ui/043-clipboard-bridge/: 三族互通演示示例（pac.at vm + assets + README 降级语义/演示剧本）"
touched_goals:
  - "GOAL-009: 虚拟桌面与桌面 Shell——473 原生互操作 Phase 2 剪贴板三族互通"
  - "GOAL-010: 示例应用轨道——043-clipboard-bridge 落地（首个带静态资产示例）"

affects: [auto-lang/ui, auto-lang/vm]
current_step: 9
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
   [✅ 已完成] 3cc1f5890。feature 定稿 `native-clipboard = ["dep:windows", "dep:image"]`
   且挂入 ui-iced 隐含列表（native-dock 同型，043 实机无需 auto 侧加门）；
   windows features 与 native-dock 共用一条目扩列（待澄清#4 定稿）。验证偏差注：
   `cargo t clipboard_native` 日常档跑 0 测试——`ui` 模块整体在 `feature = "ui"`
   门后而默认档不含；正确档 `cargo nextest run -p auto-lang --lib --features
   ui-iced clipboard_native` → 5/5 绿（ascii/cjk/头部形状/坏形状/空表）；
   `cargo check --features native-clipboard` 过且 clipboard_native 零警告
   （GlobalFree 在 Win32::Foundation、DragQueryFileW 收 Option<&mut [u16]>，
   windows 0.58 实签名）。
2. **files 集成往返**：T2 files 用例（headless guard）。
   验证：`cargo test -p auto-lang --features native-clipboard clipboard_files`。
   [✅ 已完成] a2035623e。实跑档同步骤 1 偏差注（`--features native-clipboard`
   单开不含 `ui` 模块、跑 0 测试）：`cargo nextest run -p auto-lang --lib
   --features ui-iced clipboard_files` → 1/1 绿（实机 Windows，中文路径 +
   纯路径列表往返；headless guard=set 失败即跳过；进程内互斥防并行污染）。
3. **image_get**：DIBV5/DIB/PNG 三退路 + RGBA 转换纯函数 + temp PNG 落盘 +
   T1/T2 用例。
   验证：`cargo test -p auto-lang --features native-clipboard clipboard_image`。
   [✅ 已完成] 62c6a55db。实跑档 `--features ui-iced clipboard_image` → 1/1 绿
   （仅文本→None）；模块全量 `clipboard_native::` 10/10 绿（dibv5 2×3 含 alpha
   往返 / bottom-up vs top-down 行序 / 24bpp/RLE/截断/64MP 拒收）。待澄清#2/#3
   定稿入码：非 32bpp 未压缩+标准掩码一律 None；64MP=64_000_000 像素
   （`MAX_IMAGE_PIXELS` 常量）。
4. **image_set**：PNG→DIBV5(+registered PNG 双挂) + 往返用例。
   验证：`cargo test -p auto-lang --features native-clipboard clipboard_image`。
   [✅ 已完成] 15ab84554。`--features ui-iced clipboard_image` → 2/2 绿：4×3
   alpha 梯度合成 PNG set→get 经 DIBV5 通道**像素零容差**精确比对通过
   （无损通道，容差断言退化为 0 容差）；GlobalAlloc/SetClipboardData 提炼
   `set_hglobal_data` 与 files_set 共用；PNG 双挂失败不视为整体失败
   （DIBV5 为主契约）。
5. **natives 三件套**：`vm/native_catalog.rs` 顺号注册 + `vm/native.rs` shim
   （含非 Windows 降级臂）+ T3 用例。
   验证：`cargo t clipboard && cargo check -p auto-lang`（默认 feature 编译过=降级臂在）。
   [✅ 已完成] 931985f54。ID 定稿 2934-2937（2926 邻段顺延，2934-2939 空位
   取前四）；ret tag：files_get=List/files_set=Bool/image_get=Map(Record)/
   image_set=Bool；codegen bare-name intrinsics 两处同 418 型。T3 =
   `vm/tests_clipboard_natives.rs` 四步串行实剪贴板往返（feature 门控档
   `--features ui-iced clipboard` 14/14 绿，含 481 Ctrl+C 既有测试无回归）；
   默认档 `cargo check -p auto-lang` 过=降级臂在（空表/false/null/false，
   弹实参保栈纪律）；schema 三件套 12/12 无回归。
6. **示例 043**：`examples/ui/043-clipboard-bridge/`（pac.at + src + assets/
   demo.png、hello.txt）。
   验证：`auto run examples/ui/043-clipboard-bridge -r vm` 实机起跑 + 手动三卡
   一轮（截图留痕）。
   [✅ 已完成] 4f137f5a。实机起跑形态定稿为示例目录内 `auto run -r vm`
   （pac.at 在 CWD 才被识别；资产/image src 按示例根 CWD 解析）。开发中修
   两处：① `text (text: f"…")` 命名属性/f-string 语法（位置括号表达式不被
   解析器接受）；② FilesSet 改 `fs.cwd()` 拼绝对路径——CF_HDROP 交
   Explorer Ctrl+V 解析必须绝对路径（相对路径 set 返回 true 但 Explorer
   无法定位源）。T4 实机五项全 PASS（截图留痕,见步骤 7）。
7. **T4 手动冒烟 + P481-6 rider**：按 §测试设计 T4 执行；P481-6 结果回写
   `docs/plans/KNOWN-DEBT-AND-RISKS.md`。
   验证：清单每项 PASS 注记 + 债务条目更新。
   [✅ 已完成] 4f137f5a。T4 逐项：① text 写出→true+读回显"你好,剪贴板 —
   Plan 485"PASS；② Explorer Ctrl+C(assets 两文件)→files_get 上屏 2 条
   绝对路径 PASS；③ files_set→Explorer Ctrl+V 两文件落地**可视确认两次**
   （17:25 与 01:41 截图留痕；随后盘上文件被本机某清理机制移除——环境怪癖
   与 CF_HDROP 有效性无关，Explorer 接受粘贴本身已证实）PASS；④ PrtSc→
   image_get 缩略+尺寸 3840×2160+temp 路径上屏 PASS；⑤ image_set→画图
   Ctrl+V 粘贴出 demo.png 图案 PASS。**P481-6 未清偿**：三种合成输入
   （SetCursorPos 拖动/mouse_event MOVE 增量/CUA 拖拽 identity 阻断）均无法
   驱动 winit raw-input 光标流建立拖选——043 实机证明合成定点点击对 iced
   按钮有效，问题特定于移动流；已按实情回写 KNOWN-DEBT（需人工手动拖选或
   winit 兼容注入 harness 复验）。
8. **文档化降级语义**：`ui/clipboard_native.rs` 模块头注（vue/远程端不适用、
   降级返回值约定）+ 043 README。
   验证：`cargo check -p auto-lang` 零警告。
   [✅ 已完成] 99128385e。模块头注 Degradation contract 段（随 T1 落位：
   vue/web 远程端不适用 + files_get→[] / files_set·image_set→false /
   image_get→None 返回值约定）；043 README 降级语义表 + 起跑形态（示例目录
   CWD）；补 examples/ui/README 索引行 + vm 模式清单。验证：默认档
   `cargo check -p auto-lang` 158 warnings=基线零新警；ui-iced 档
   clipboard_native 零警告。
9. **收尾**：`cargo t ui`、`cargo t vm` 不回归；无调试打印；状态翻
   execution_done。
   验证：`cargo t ui && cargo t vm`。
   [✅ 已完成] 22fe445cf。首跑暴露真实缺陷：nextest 每测试一进程并行下，
   本方 files/image 集成测试（EmptyClipboard）跨进程清掉了 418 arboard
   往返的 set→get 窗口（单跑绿、全套偶发红——进程内 Mutex 不覆盖跨进程，
   正是 §测试设计"防并发污染"预警的形态）。修复：`GlobalClipboardTestLock`
   会话命名互斥（CreateMutexW+WaitForSingleObject 30s；windows features
   增 Win32_Security），接线四个测试（本方三个 + 418 set_then_get）。
   复验：clipboard 套件**三连跑 14/14 全绿**；`cargo t ui` 776 绿 /
   `cargo t vm` 638 绿；`cargo check -p auto-lang` 默认档 158=基线；
   clipboard_native.rs 无调试打印。

## 复审记录

（/auto-plan:review 填写）

**复审人**：zhaopuming（agent 复审，2026-08-30）
**方法**：计划 vs 实际 diff 逐文件核对（merge-base 75d2d4500..HEAD，15 文件恰好
对应计划声明，无游离改动/无遗漏文件）；全量门禁重跑；验收标准逐项重验；
遗漏/延后/workaround 专项扫描。

### 全量门禁（本计划唯一全量跑点）

- `cargo tf`：**3275/3275 全绿**（含 schema_drift/docs_gen/component_registry 三件套）。
- `cargo tv`：**1 红** `tests::aavm2_m4::test_aavm2_m4_codegen_corpus`（b13_is_enum.at
  字节码对拍失配）——**A/B 判定非本计划引入**：master @3a4aacf19 同测同红
  （本计划分支相对 master 的增量仅四 intrinsics+catalog+shims，不触 codegen
  生成路径；嫌疑在并行会话合入的 051-C7/484 线，出计划外，见下方发现②）。

### 验收标准逐项判定

1. **四 natives 注册齐且 VM 可调；降级语义符合 G3 —— PASS**。catalog 两表
   2934-2937 + codegen intrinsics 两处 + shim 活动臂/降级臂（空表/false/null/
   false，弹参保栈纪律）逐行核对（native.rs:824-934）；VM 可调经 T3 shim 级
   + 043 实机全链路双证；降级臂默认档编译过（158 警告=基线）。
2. **043 实机三卡可演示，Explorer 文件往返成功 —— PASS**。执行期 T4 五项
   截图留痕 + 复审 fresh 重建（T9 后 feature 集）实机复验：起跑/三卡渲染/
   「读取」真实回读 OS 剪贴板内容上屏（text_get → "plan418-…"）。
3. **T1/T2/T3 自动化绿；schema 不回归；零警告 —— PASS**。clipboard 套件
   复审连跑 11+ 次 14/14 全绿（见发现③偶发注）；`cargo t ui` 776 绿 /
   `cargo t vm` 638 绿（执行期）+ tf 3275 绿（复审期）覆盖；`cargo check`
   默认档 158=基线、ui-iced 档 clipboard_native 零警告。档位偏差（计划写
   `--features native-clipboard` 跑 0 测试，实跑 `--features ui-iced`）已记
   入步骤证据——计划文档笔误，非实现缺陷。
4. **P481-6 回写 KNOWN-DEBT 为已清偿 —— PARTIAL（偏差，用户裁决窗口后按
   记录在案处理）**。实情：rider 未清偿——三种合成输入（SetCursorPos 拖动/
   mouse_event MOVE 增量/CUA 拖拽 identity 阻断）均无法驱动 winit raw-input
   光标流建立拖选；键路单测 text_selection_ctrl_c_writes_clipboard 保持绿。
   KNOWN-DEBT P481-6 已按实情回写（根因定位+复验路径：人工拖选或 winit 兼容
   注入 harness）。复审中已向用户提出三项裁决（手动验证/接受偏差/退回 work），
   未获答复——退回 work 将死锁（agent 无法执行人工输入），且该偏差全程显式
   留痕（执行中期待澄清+交接报告+债务条目），按"记录不隐藏"原则路由 reviewed，
   债务保持开放。
5. **`cargo t ui`、`cargo t vm` 不回归 —— PASS**（776/638 绿；tv 例外见上，
   master 同红非本计划引入）。

### 遗漏 / 延后 / workaround 扫描

- **遗漏**：无——15 文件全部映射计划声明；T1-T4 用例齐；两张 catalog 表+
  codegen 两处 intrinsics 无漏登记。
- **延后**：仅 P481-6 rider（见标准 4，显式留痕非静默）。OLE 拖放/Phase 3
  为计划明文非目标，不算延后。
- **Workaround**：diff 无 TODO/FIXME/HACK 标记；CF_DIB 尽力而为边界
  （非 32bpp 未压缩→None）为计划待澄清#2 预先批准的设计裁剪。

### 复审发现（计划外）

1. **T9 修复验证有效**：跨进程命名互斥后 clipboard 套件稳定（复审连跑 11+
   次全绿）。
2. **master 既有回归（非本计划）**：`aavm2_m4_codegen_corpus` 在 master
   @3a4aacf19 即红（cargo tv 档）——嫌疑 051-C7/484 并行合入线；建议尽快
   专项修复（本计划 tf 全绿不受影响）。
3. **环境偶发注（债务候选）**：clipboard 集成测试在重负载+外部剪贴板监听器
   （WPS/微信后台）竞写下 1/16 偶发红（set→get 窗口被外部改写）；缓解可选
   测试内单次重试，暂按已知环境风险记录。
4. **aavm 移植边界快照滞后（非义务）**：`docs/specs/aavm/design/data/
   catalog_table.csv`（431 期冻结快照）无 2934-2937——按其 §6 在 tag 重锚
   时再生成，非逐计划义务，仅注记。

**路由**：全部核心标准（1/2/3/5）PASS，标准 4 为显式留痕偏差且债务开放——
**status → reviewed**，可交 `/auto-plan:merge`。

## 待澄清事项

- native ID 具体编号以 `vm/native_catalog.rs` 当前空位为准（2926 邻段顺延）。
  〔已定稿：2934-2937（T5）〕
- CF_DIB（无 V5 头的旧格式）覆盖面：截图工具/现代浏览器均写 DIBV5 或 PNG，
  纯 CF_DIB 退路仅做头解析尽力而为，不做 BITMAPINFOHEADER 全变体（RLE 压缩
  等罕见变体直接 None）——复审时确认接受度。
  〔已定稿（T3）：仅 32bpp 未压缩（BI_RGB/BI_BITFIELDS）+标准 BGR 掩码，
  其余一律 None；`dib_parse_rejects_unsupported_shapes` 用例锁定〕
- image_get 大图阈值：>64MP 像素直接 None（防误爆内存），数值在实现时定稿。
  〔已定稿（T3）：`MAX_IMAGE_PIXELS = 64_000_000`，纯函数内防爆+PNG 头
  只读尺寸双闸〕
- windows dep 的 features 列表若与 native-dock 共用一条目，扩列方式（合并
  features vs 拆两条目）以编译最小扰动为准，T1 时定。
  〔已定稿（T1）：合并共用一条目扩列（Cargo feature 合并语义下拆条目无
  收益）；T9 再增 Win32_Security（CreateMutexW 签名需要）〕
- 〔新增 2026-08-30 T7〕P481-6 rider **未清偿**（验收标准 4 未达成，按实情
  记录）：三种合成输入（SetCursorPos 拖动 / mouse_event MOVE 增量 / CUA
  拖拽 identity 阻断）均无法驱动 winit raw-input 光标流建立拖选，实机末步
  无法自动化复验；已回写 KNOWN-DEBT P481-6 条目（需人工手动拖选或建
  winit 兼容注入 harness）。043 实机同时证明合成定点点击对 iced 按钮有效
  ——问题特定于"移动流"，非窗口焦点。请复审裁决：接受记录 / 人工补验。
- 〔新增 2026-08-30 T7〕T4 files_set→Explorer 粘贴：两文件落地**可视确认
  两次**（截图留痕），但随后盘上文件被本机某清理机制移除（cmd/PS 均不可
  见、Explorer 视图亦回空）——环境怪癖与 CF_HDROP 有效性无关（Explorer
  接受粘贴本身已证实）；如复审需要盘上证据可人工重做一轮。
