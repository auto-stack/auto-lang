---
plan_id: PLAN-488
status: executing              # drafting → executing → execution_done → reviewed → archived
feature_name: vm-native-dragdrop
author: [zhaopuming]
created_at: 2026-08-30
updated_at: 2026-08-30

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-lang/vm]
current_step: 4
total_steps: 11
---

# [PLAN-488] 原生互操作 Phase 3——OLE 拖放双向 + 虚拟文件落地

## 变更摘要

473 原生互操作路线的 Phase 3（前置 485 剪贴板已合入），兑现用例矩阵的拖放族
（A1/A2/A5/A6/A8/D2–D4）：

1. **拖出（desktop → native）**：自实现 `IDataObject`（windows crate
   `implement` 宏），格式族 `CF_UNICODETEXT`（文本）/ `CF_HDROP`（真实文件）/
   `CFSTR_FILEDESCRIPTOR + CFSTR_FILECONTENTS`（**内存虚拟文件**——"拖出即落地"
   用例 A2：把桌面 App 的文章对象拖进 Explorer 变成真实文件）；`DoDragDrop`
   跑在专用 STA 线程（消息泵），发起走新 native `auto.dnd.start(payload)`
   （受理即返，完成事件异步回 App）。
2. **拖入（native → desktop）**：自实现 `IDropTarget` 挂宿主 HWND
   （`RegisterDragDrop`），DragEnter/Over/Leave/Drop → `DesktopMessage` →
   指针命中虚拟窗口 → **App 级 `on_native_drop` 事件**（`virtual_window`
   声明 events，走既有 handler 管线）。
3. **全局 Ctrl+V 粘贴路由**：兑现 485 非目标承诺——桌面级热键臂
   （renderer.rs:6353 段）读剪贴板（418 文本 + 485 文件/图片）注入焦点 App
   的 `on_native_paste` 事件。
4. **夹具扩展**：`tools/native-fixture` 落地 473 预留的 `--offer` 拖源与
   放置目标日志（main.rs:19 TODO）。

**开工前置 = 486 合入**（拖拽会话/手势触发面先就位，见待澄清⑤）。

## 目标

- **G1 拖出**：`auto.dnd.start({text?, files?, virtual_files?})` 发起系统拖拽；
  Explorer/notepad/Chrome 作为落点分别收到文件/文本/虚拟文件落地；完成效果
  （copy/move/cancel）以 `dnd_finished` 事件回 App。
- **G2 拖入**：从 Explorer 拖文件、从 notepad/浏览器拖文本/URL、从 Chrome 拖
  图片（CF_DIB/PNG，复用 485 转换）到桌面窗口 → 光标下虚拟窗口的 App 收到
  `on_native_drop`（payload 含 text/files/image  whichever 可用）。
- **G3 Ctrl+V 路由**：桌面任意处 Ctrl+V → 焦点 App 收 `on_native_paste`
  （text/files/image，来自 OS 剪贴板）。
- **G4 夹具**：`--offer text|files` 可编程拖源 + 全窗放置日志（JSON lines），
  使拖放双向可自动化断言。
- **G5 降级**：非 Windows / 未开 feature：`auto.dnd.start` 返 false、事件
  不触发，.at 代码零条件分支。
- **非目标**：自定义拖拽 ghost 覆盖层（= Phase 4 真洞的覆盖层基建）；真
  延迟回调虚拟文件（拖出时才向 VM 要内容——跨线程回调，v1 用内存字节，
  列增强）；外来虚拟文件拖入的 FILECONTENTS 拉流（v1 拒收，见待澄清②）；
  桌面内 AutoUI↔AutoUI 拖放（另行机制）；Excel CF_CSV 专用格式（A8 走文本
  通道覆盖）。

## 架构方案

```
拖出：.at App ──auto.dnd.start(payload)──▶ native shim ──▶ DnD STA 线程
        payload {text?, files?[path], virtual_files?[{name,bytes, mime}]}
        线程：OleInitialize → DoDragDrop(IDataObject+IDropSource) → 泵
        完成：DesktopMessage ──▶ App 事件 dnd_finished{effect}
拖入：OS ──OLE──▶ IDropTarget(宿主 HWND) ──▶ DesktopMessage::NativeDrop{formats, 屏幕位置}
        ──指针命中虚拟窗口──▶ (AppId) AppSession ──▶ on_native_drop(payload Record)
Ctrl+V：桌面级热键臂(renderer.rs:6353 段) ──读 clipboard(418/485 natives)──▶
        焦点 App on_native_paste(text/files/image Record)
```

- **模块**：新 `crates/auto-lang/src/ui/native_dnd.rs`；feature
  `native-dnd = ["dep:windows"]`，windows features 增 `Win32_System_Com` +
  `Win32_System_Ole`（共用条目，485 先例——Cargo.toml 186 段注释续写）。
- **协议**：拖放是数据/输入面，不加 `desktop.*` 动词——486/487 的 v1.3/v1.4
  升版不涉本期。
- **事件面**：`schema/aura.at` `virtual_window`（L5050，props 空）增
  `events: ["on_native_drop", "on_native_paste", "on_dnd_finished"]`（语法先例
  L526）；WidgetRegistry backend 映射同步。
- **winit 共存**：本仓未消费 winit 的 DroppedFile（grep 零业务命中）——
  OLE 目标是本仓首个拖入通道；与 winit 既有 DragAcceptFiles 路径的互斥表现
  由 T2 spike 定案（待澄清①）。

## 技术栈

windows crate COM（`implement` 宏：`IDataObject`/`IDropSource`/`IDropTarget`、
IStream/HGLOBAL）；复用 485 `clipboard_native.rs` 的 DIB/PNG 转换；复用 473
fixture。零新三方依赖。

## 需求分析与背景调查

（取材 docs/specs/overview.md §ui/vm + 现场核验 2026-08-30）

- **路线定位**：473 阶段表 Phase 3（OLE 拖放 + 虚拟文件落地）；Phase 2
  （485 剪贴板三族）已合入归档（natives `auto.clipboard.*`、
  GlobalClipboardTestLock、windows features 共用条目模式）。
- **473 资产复用**：native_dock 的 feature 门控模板；`tools/native-fixture`
  main.rs:19 预留 TODO 恰为本期拖源/落点；WinEventHook 线程模式（STA 线程+
  泵同型）。
- **事件管线**：aura.at events 语法先例（L526 `events: ["onmouseenter",
  "onmouseleave"]`）；`virtual_window` 声明 L5050（props 空、(AppId,event)
  路由语义）；AppSession 事件注入走既有 handler 管线（执行期对齐具体符号，
  见待澄清④）。
- **热键挂点**：renderer.rs:6353（463 T3/T6 桌面级热键订阅——单份、按宿主窗
  过滤），Ctrl+V 臂加在此。
- **拖出 UI 前置**：481 已落 selectable 文字（文本拖源可用）；028-launcher/
  009-article-feed 等卡片为天然拖源候选（实机冒烟用）。
- **排程**：486（触发面）执行中——**开工前置 = 486 合入**；487（settings）
  无交叠；P485-2 的 master tv 红不涉本面（aavm2）但复审门禁前应已分诊。

## 详细设计

### 1. ui/native_dnd.rs（新）

- `DndPayload`：`{ text: Option<String>, files: Vec<PathBuf>, virtual_files:
  Vec<VirtualFile{name, bytes, mime}> }`（组合，多格式同挂）。
- `#[implement(IDataObject)]`：`EnumFormatEtc`（顺序：文本→HDROP→
  FILEDESCRIPTOR/CONTENTS）；`QueryGetData`；`GetData` 按需产 HGLOBAL——
  FILEGROUPDESCRIPTORW 构造（FD_WRITESTREAM|FD_READURI 视接收方兼容）+
  FILECONTENTS 按 index 出字节块；`SetData`/通知系返回 E_NOTIMPL（源侧最小）。
- `#[implement(IDropSource)]`：标准最小实现（Esc 取消、拖出反馈走系统默认）。
- `#[implement(IDropTarget)]`：Enter/Over 用 `ModifierState`+命中窗口算
  效果（DROPEFFECT_COPY 为主）；Leave 清状态；Drop 枚举可用格式→取数据
  （HDROP→路径列表 / CF_UNICODETEXT→串 / CF_DIB·DIBV5·"PNG"→image 走 485
  转换产 temp PNG）→ 投递 `DesktopMessage::NativeDrop{…}`（含屏幕坐标）。
- **线程**：`dnd_sta_thread`（OleInitialize + DoDragDrop + 泵 + Marshal 结果
  回主循环 channel）；宿主侧 `OleInitialize` 幂等共存。

### 2. VM native（三件套）

- `auto.dnd.start(payload_json) -> Bool`：payload 解析→转 DndPayload→投 STA
  线程→受理返 true；完成经 `DesktopMessage::DndFinished{effect}` → App 事件
  `on_dnd_finished`。
- `native_catalog.rs` 顺号（1106/2926 邻段空位）+ `vm/native.rs` shim（非
  Windows 降级返 false）。

### 3. 拖入路由 + 事件注入

- `DesktopMessage::NativeDrop{app_id, text?, files?, image?, screen_pos}`：
  命中计算复用 486 DragWatch 的指针→虚拟窗口命中（或 layout bounds 直接
  命中，执行期取更简路径）；
- AppSession 事件注入：`on_native_drop` 收 Record（同型 472/479 事件注入
  管线）；`on_native_paste` 同管线（Ctrl+V 臂产 Record：text/files/image
  whichever）。

### 4. Ctrl+V 路由

renderer.rs:6353 桌面级热键订阅增 Ctrl+V 臂：读剪贴板优先级
text（418）→ files（485）→ image（485）；产 `on_native_paste` 注入焦点
App（`__wm_wins` focused 或宿主焦点 AppId，执行期对齐既有焦点事实源）。

### 5. 夹具扩展（tools/native-fixture/src/main.rs）

- `--offer text:<str>` / `--offer files:<p1;p2>`：窗口内按钮触发本进程
  DoDragDrop（自实现最小 IDataObject，或复用 native_dnd 导出——独立 bin 故
  内置最小实现）；
- IDropTarget 全窗挂载：drop 时日志 `{"evt":"drop","formats":[…],"text":…,
  "files":[…]}`（JSON lines，供 E2E 断言）。

### 6. schema/registry

`schema/aura.at` virtual_window 增 events 三枚；`ui_gen/widget/registry.rs`
virtual_window spec events 映射同步。

## 测试设计

1. **T1 COM 单元**（`--features native-dnd`）：IDataObject 格式枚举顺序/
   QueryGetData 命中/GetData 往返（文本串、HDROP 字节解析、
   FILEGROUPDESCRIPTORW 构造字节断言）——本进程直调 COM 方法，无需真拖。
2. **T2 路由单元**：NativeDrop→命中 AppId（注入布局）；payload Record 构造；
   热键臂→on_native_paste（注入剪贴板态）。
3. **T3 E2E 拖入**：fixture `--offer files` → SendInput 拖至桌面窗口 → 经
   AutoUI MCP 断言目标 App 收到 on_native_drop（files 列表一致）。
4. **T4 E2E 拖出**：App 调 `auto.dnd.start`（text+virtual_file）→ 拖到
   fixture → 断言 fixture drop 日志（格式+文本内容+虚拟文件落地字节比对）。
5. **T5 Ctrl+V**：预置剪贴板（485 natives）→ 注入 Ctrl+V → 断言焦点 App
   on_native_paste。
6. **T6 实机冒烟**：A1 Explorer→桌面（文件列表入 App 事件）；A2 桌面→
   Explorer 虚拟文件落地双击可开；A5 notepad 文本双向；A6 浏览器 URL 双向；
   D2/D3/D4 Chrome 拖出/拖入/图片（格式协商观察记录）。结果逐行记入 §验收
   标准下。

## 验收标准

1. T1–T5 自动化绿（`cargo t native_dnd` 档 + E2E feature 档）。
2. T6 六场景实机留痕（A1/A2/A5/A6/D2–D4）。
3. winit 共存 spike 结论回写待澄清①（RegisterDragDrop 与 winit 拖放路径
   互不干扰或明确取舍）。
4. schema 三件套（schema_drift/docs_gen/component_registry）绿（events 增量）；
   `cargo t ui`、`cargo t vm` 不回归；`cargo check -p auto-lang` 零警告。
5. 非 Windows 编译通过且 natives 降级语义符合 G5。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **feature + 骨架**：`crates/auto-lang/Cargo.toml` 增
   `native-dnd = ["dep:windows"]` + windows features 增
   `Win32_System_Com`/`Win32_System_Ole`；新建
   `crates/auto-lang/src/ui/native_dnd.rs`（DndPayload + 模块骨架 +
   `ui/mod.rs` 登记）。
   验证：`cargo check -p auto-lang --features native-dnd && cargo check -p auto-lang`。
   [✅ 已完成] worktree 提交 1d43a0da9：双档 check 绿，native_dnd/ui-mod 零警告（160 条为既有存量，与本改动无关，grep 实证）。
2. **共存 spike**：宿主 HWND `RegisterDragDrop` 试验（临时日志/按钮触发
   触发 `DragEnter` 探针），结论回写待澄清①。
   验证：`cargo t ui`（无回归）+ spike 记录。
   [✅ 已完成] worktree 提交 2bda07b90：运行时探针
   `spike_register_revoke_roundtrip` 绿（单 HWND 二次注册→
   DRAGDROP_E_ALREADYREGISTERED；Revoke→重注 S_OK）+ winit/iced 源码级
   取证；结论已回写待澄清①。验证补充：日常档 772 全绿；ui-iced 档唯一红
   `plan050_i18n_lookup` 为存量（stash 基线复现，与本改动无关）。
3. **IDataObject + IDropSource**：实现 + T1 单测。
   验证：`cargo test -p auto-lang --features native-dnd native_dnd`。
   [✅ 已完成] worktree 提交 04d962cab/700f85bb8：T1 九测全绿（枚举定序
   文本→HDROP→DESCRIPTOR→CONTENTS / QueryGetData 命中与未挂 / 文本与
   HDROP 往返 / FILEGROUPDESCRIPTORW 字节断言（cItems+flags+尺寸+名）/
   FILECONTENTS 按 index / 源侧最小 E_NOTIMPL / Esc-取消-落下）。验证命令
   修正：module 在 ui 门控下，实际跑
   `cargo test -p auto-lang --lib --features ui-iced,native-dnd ui::native_dnd`
   （裸 `--features native-dnd` 不编译 ui 模块——计划命令笔误，见步骤 1 注）。
   实现注记：FILEDESCRIPTORW 为 packed——cFileName 存取走 addr_of!；
   FD_WRITESTREAM bindings 缺失本地定义；FD_READURI 无 SDK 对应物 v1 不置。
4. **STA 线程 + native**：`dnd_sta_thread`（OleInitialize/DoDragDrop/泵）+
   `auto.dnd.start` 三件套（catalog 顺号 + shim 降级）+ `dnd_finished` 回注。
   验证：`cargo t native_dnd && cargo t vm`。
   [✅ 已完成] worktree 提交 6653b2803：STA 线程（受理即返/DRAG_ACTIVE 重入
   拒/完成通道 Mutex 装载）+ native_catalog 2938（顺号取当前空位，签名表
   Bool 同步）+ shim_dnd_start 真臂/降级臂（G5 false）+
   DesktopEvent::DndFinished + dnd_finished_subscription（16ms 轮询，
   非双门控档空订阅）+ renderer 注册与占位 update 臂（注入管线步骤 6 接线）。
   验证：native_dnd 15 测全绿（新增 payload JSON 解析×3 + 受理语义×2）；
   `cargo t vm` 638 绿；ui-iced 单档/default 档 check 零 native_dnd 警告。
5. **IDropTarget + 拖入路由**：实现 + `DesktopMessage::NativeDrop` + 命中→
   AppId + T2 路由单测。
   验证：`cargo t session && cargo t native_dnd`。
6. **事件面**：`schema/aura.at` virtual_window 增 events 三枚 +
   `ui_gen/widget/registry.rs` 映射 + AppSession 注入管线接线。
   验证：`cargo test -p auto-lang --test schema_drift && cargo test -p auto-lang --test docs_gen && cargo t ui`。
7. **Ctrl+V 路由**：renderer.rs:6353 段热键臂 + 剪贴板读取优先级 +
   on_native_paste 注入 + T2 对应单测。
   验证：`cargo t ui`。
8. **夹具扩展**：`tools/native-fixture/src/main.rs` `--offer` 拖源 + drop
   日志（清 main.rs:19 TODO）。
   验证：`cargo run --manifest-path tools/native-fixture/Cargo.toml -- --offer text:hi` 手动起拖冒烟。
9. **E2E T3/T4**：`crates/auto-lang/tests/native_dnd_e2e.rs`（feature 门控，
   拖拽模拟手段沿 486 待澄清③裁定）。
   验证：`cargo test -p auto-lang --features native-dnd --test native_dnd_e2e`。
10. **实机冒烟 + 收尾**：T6 六场景执行留痕；健康检查（零警告/无调试打印）；
    状态翻 execution_done。
    验证：`cargo check -p auto-lang && cargo t ui && cargo t vm`。
11. **骑手：P485-2 分诊（先分类、后处置，带逃生舱；2026-08-30 调度追加）**：
    本计划改动面含 `vm/`（natives），复审门禁跑 `cargo tv`——先在红
    `tests::aavm2_m4::test_aavm2_m4_codegen_corpus`（master @3a4aacf19 即红，
    KNOWN-DEBT P485-2）由本会话顺路分诊：
    - **T11a 分类**：复现锚定 → 判定"corpus 期望过期"（051-C7/484 语义变更
      未更新对拍）还是"codegen 真回归"（bisect 锚 3a4aacf19 邻域）。
    - **期望过期** → 更新 corpus 期望 + 成因注释一行，`cargo tv` 全绿，
      KNOWN-DEBT P485-2 标已清偿；本计划收尾门禁含 tv 绿。
    - **真回归** → **不在本计划修**（域不同、规模未知）：回写 P485-2 精确
      归属注记（根因 commit + 证据），复审按"先在红 + 精确归属 + 488 分支
      零 codegen 触碰（改动面=native_dnd/ui + vm natives）"放行，修复转
      独立专项（后续计划号）。
    验证：`cargo tv` 全绿（或逃生舱路径的 DEBT 注记 + 复审记录成文）。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **与 490（桌面热键清理）的热键域协调（2026-08-30 调度补记）**：T7 的
  Ctrl+V 臂与 490 G3 的键位表化（`shell.keys.*`）同域——后合者适配：490
  先合则 T7 挂 `shell.keys.paste` 键位表；488 先合则 490 收编该臂。另：
  486 已合入，本计划开工前置已满足，可领取。
- **① winit 共存**（T2 spike 回写）——**已定案（2026-08-30，步骤 2）**：
  winit 0.30.13 Windows 后端**本身就是 OLE 注册方**（窗口创建时
  `OleInitialize`+`RegisterDragDrop(hwnd, 自实现 IDropTarget)`，
  winit window.rs:1168/1194；teardown `RevokeDragDrop` 忽略返回值，
  event_loop.rs:1262）——不是计划假设的 WM_DROPFILES/DragAcceptFiles
  旧路径。其 Drop 只取 CF_HDROP → iced `FileDropped` 等，本仓零消费
  （grep=0）。运行时探针实证：单 HWND 二次注册返回
  DRAGDROP_E_ALREADYREGISTERED，Revoke 后重注 S_OK。**取舍**：步骤 5
  挂载宿主 HWND 时执行 `RevokeDragDrop(hwnd)`（撤销 winit 单格式目标，
  损失=0）→ `RegisterDragDrop(hwnd, 我方多格式目标)`；winit teardown 的
  Revoke 忽略返回值不会 panic。**残留**：iced/winit 重建宿主 HWND 时会
  重注自己的目标——步骤 5 挂载逻辑按 HWND 身份键控，检测变化时重跑
  Revoke+Register。
- **② 外来虚拟文件拖入**：FILECONTENTS 拉流（接收方视角）v1 拒收（格式
  枚举可见、内容不取）——真拉流与"拖出真延迟回调"同列增强；Chrome 图片
  拖入走 CF_DIB/"PNG" 通道（485 转换复用），不受此限。
- **③ 拖拽模拟手段**：SendInput 真拖优先、事件注入退路——与 486 同题，
  先合者定案后者沿用（避免两套模拟栈）。
- **④ 事件管线符号**：`on_native_drop`/`on_native_paste`/`on_dnd_finished`
  的 handler 触发路径以 472/479 既有事件注入管线现状对齐；命名若与约定
  冲突以管线现状为准（aura.at events 以最终定名回填）。
- **⑤ 开工前置 = 486 合入**（拖拽会话宿主/命中计算复用其 DragWatch 产出）；
  执行时行号漂移（renderer.rs:6353 / aura.at:5050/L526）以 grep 重定位。
- **⑥ natives 编号**：`auto.dnd.start` 顺号以 native_catalog 当前空位为准。
