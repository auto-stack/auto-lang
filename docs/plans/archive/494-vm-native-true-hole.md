---
plan_id: PLAN-494
status: archived                # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: vm-native-true-hole
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 修订——native dock 473/486 段后追加 494 真洞段（hole 模式位 shell.native.hole / raise_desktop_above z 翻转 / apply_hole_regions Region 洞排除 / 回退假洞）"
new_spec_components: []        # 无新 spec 模块——扩展既有 native_dock/ui 域
touched_goals:
  - "GOAL-009: 虚拟桌面与桌面 Shell——原生窗口收编的真洞形态（视觉+输入穿透）落地"

affects: [auto-lang/ui]
current_step: 10
total_steps: 10
---

# [PLAN-494] 原生互操作 Phase 4——真洞（透明穿透 + 覆盖层）

## 变更摘要

原生互操作四阶段的收官（473 假洞 → 485 剪贴板 → 486 触发面 → 488 拖放 →
**本期**）。桌面窗口转**透明**（全屏壳 + wgpu 预乘 alpha），docked 原生窗口
从"盖在桌面上方"（473 假洞）**翻转为垫在桌面下方**，洞区 alpha=0 透出原生
内容；自有窗口上 `WM_NCHITTEST` 子类化使洞区点击穿透（`HTTRANSPARENT`）直达
原生窗口。由此解锁**覆盖层能力**：桌面任意 UI 可绘制在原生窗口之上——兑现
488 延期的拖拽 ghost 跨洞、toast/任务栏横贯洞区。

**模式化落地**：`shell.native.hole` storage 位（默认 **off**，假洞行为与
473/486/488 逐项一致）；透明初始化失败自动回退假洞。翻默认值另立决策。

**两项 spike 先行**（本计划仅有的两个"验证而非已知"点）：
① iced/winit 透明窗口 + wgpu 预乘 alpha swapchain 在本仓渲染器的可行性；
② `HTTRANSPARENT` 跨进程点击穿透实证。

## 目标

- **G1 真洞渲染**：hole 模式下 docked 原生窗口内容经洞区可见（桌面窗口恒在
  z 顶），洞内桌面**零绘制**（alpha=0 纪律，槽位框只画四周）。
- **G2 点击穿透**：洞区点击/悬停直达原生窗口（fixture E2E 断言铁证）；洞外
  桌面交互不受影响。
- **G3 z 序不变量翻转**：docked 窗口紧贴桌面窗口**正下方**（替代 473 的
  "正上方"）；杂散窗口镇压策略沿用（镇压面下移一层）。
- **G4 覆盖层兑现**：拖拽 ghost 跨洞可见（488 增强兑现）+ toast 横贯洞区
  实机演示。
- **G5 模式兼容**：hole_mode=off 时与 473/486/488 行为**逐项一致**（既有
  native_dock/native_dnd E2E 全量回归即门禁）；透明初始化失败自动回退+日志。
- **非目标**：翻默认值（验证后另立决策）；原生窗口变换/动画（位置尺寸
  之外）；洞内原生菜单/弹窗的额外管理（OS 置顶机制天然正确，仅验证不接
  管）；多洞性能专项优化（全屏 alpha 合成成本早已定论可接受）。

## 架构方案

```
z 序（自上而下，hole 模式）             对照（off = 473 假洞，不变）
┌ vm 桌面窗口（透明）                   ┌ docked 原生 HWND
│   洞区 alpha=0 → 透出下层             ├ vm 桌面窗口（不透明）
│   toast/ghost/任务栏可压洞上          └ （杂散镇压于其上）
├ docked 原生 HWND（洞区透出）
└ 杂散窗口（镇压于此层之下）
```

- **输入**：`WM_NCHITTEST`（SetWindowSubclass 自有窗口）：屏幕点 ∈ 任一
  Docked 槽位洞矩形 → `HTTRANSPARENT`（命中测试落到 z 序下层=原生窗口）；
  否则默认。洞矩形来自 WM 槽位状态（线程安全快照，relayout 即时更新）。
- **渲染**：槽位框 chrome 只画洞四周；洞内不产 span、无占位底色（off 模式
  的底色占位保留）。
- **ghost 跨洞**：488 的拖拽自绘预览在桌面窗口内绘制——hole 模式下天然
  压在原生内容之上，无需额外层。

**改动面**：`ui/native_dock/win32.rs`（z 序翻转 + NC 子类）、
`ui/native_dock/mod.rs`（hole 模式状态）、`ui/iced/renderer.rs`
（transparent 设置 + 洞区绘制纪律 + ghost 消费）、session/DesktopOptions
（模式位）、`tools/native-fixture`（点击坐标日志）。

## 技术栈

winit `transparent` + wgpu 预乘 alpha swapchain；windows crate
`SetWindowSubclass`/`WM_NCHITTEST`。零新三方依赖。

## 需求分析与背景调查

（取材 docs/specs/overview.md §ui + 现场核验 2026-08-31）

- **路线定位**：473 阶段表 Phase 4；473 待澄清"假设成立性：vm 桌面为全屏
  壳拓扑 → 真洞方向保留"——486 已落全屏壳 + 触发面，假设成立。
- **可复用资产**：473 假洞全套（z 序 insertAfter/槽位框/WinEvent/夹具，
  native_dock 模块）；486 DragWatch（指针→槽位命中计算，穿透判定同源）；
  488 拖放（ghost 的自绘预览延到期即本计划消费者；DIB/PNG/OLE 全不涉）。
- **风险已知项**（前期论证结论）：全屏透明窗口失去 DWM independent flip
  全屏优化——UI 密集场景可接受；`DWMWA_WINDOW_CORNER_PREFERENCE=DONOTROUND`
  473 dock 时已设；DPI/坐标换算不变。
- **唯一双高风险点**：透明 swapchain（本仓 iced 0.14 渲染管线实测）与
  `HTTRANSPARENT` 跨进程穿透（覆盖层/HUD 社区实践为据但需实证）——故
  T1/T2 spike 先行，且都有退路（透明失败→回退假洞；穿透失败→洞区动态
  `WS_EX_TRANSPARENT` 切换或独立覆盖层方案，见待澄清①②）。
- **排程**：队列空（490/491/492/493 均归档），无并行冲突；490 键位表已合
  入，桌面热键面稳定。

## 详细设计

### 1. 模式位

- `DesktopOptions.hole_mode: bool` + storage 键 `shell.native.hole`（boot 读
  入，487 dock 配置同型）；默认 off；运行时切换（settings 面板）后置。

### 2. 透明窗口（spike① → 实装）

- `run_dynamic_desktop` 窗口构建在 hole_mode 时置 `transparent(true)`；
  wgpu surface alpha mode 预乘（iced Settings 透传，执行期以 0.14 实际 API
  定——spike 结论回写）。
- **回退**：surface 创建/首帧 alpha 校验失败 → 降级 off + 一行日志
  （`shell.native.hole` 不回写，下次启动重试）。

### 3. z 序翻转（win32.rs）

- dock/relayout 的 insertAfter 目标从"桌面窗口"改为"紧贴桌面窗口下方"
  （双步舞：置原生于桌面上→置桌面于原生上，净效果=桌面紧贴其上）；
- 杂散窗口镇压（EVENT_OBJECT_FOREGROUND 治理）不变，镇压基准层下移。

### 4. WM_NCHITTEST 子类

- `SetWindowSubclass(宿主 HWND)`：`WM_NCHITTEST` → 屏幕点 ∈ 洞矩形集合 →
  `HTTRANSPARENT`；洞矩形快照 = `WmState.native_slots` 的槽位 rect 只读
  投影（relayout 后原子换）；其余消息默认过程。

### 5. 绘制纪律

- 槽位渲染：hole 模式洞内**零 span/零底色**（off 模式底色占位保留——同一
  代码路径按模式分支，I3 配置差异原则）。

### 6. 覆盖层消费者

- ghost：488 拖拽预览绘制臂在 hole 模式确认不被洞区裁剪（天然可见）；
- toast：既有桌面 toast 叠层横贯洞区的实机演示项（无代码改动预期，验证性
  任务）。

### 7. 夹具

- `tools/native-fixture` 增 `WM_LBUTTONDOWN` 坐标日志（JSON lines
  `{"evt":"click",x,y}`）——穿透 E2E 的断言源。

## 测试设计

1. **T1 单元**：屏幕点↔洞矩形命中映射；hole 模式状态机（on/off/初始化
   回退）；z 序翻转后的不变量断言（离线——顺序模型）。
2. **T2 spike 成文**：透明 swapchain（像素格式/首帧 alpha 采样）与
   `HTTRANSPARENT` 跨进程穿透（探针窗+下层窗点击）结论回写待澄清①②。
3. **T3 穿透 E2E**（fixture）：hole 模式 dock 后向洞心坐标发点击 →
   fixture 日志收到该坐标点击（**穿透铁证**）；洞外坐标 → fixture 无日志。
4. **T4 回归**：hole_mode=off 下 native_dock + native_dnd 全量 E2E 与
   473/488 行为一致（零差异门禁）。
5. **T5 实机**：Explorer docked 真洞视觉（四角/边框/阴影）；ghost 拖拽跨
   洞可见；toast 横贯洞区；双屏不同缩放；洞内右键菜单/文件对话框置顶正常。

## 验收标准

1. T3 穿透铁证绿 + T4 off 模式全量回归零差异。
2. T5 实机清单 PASS 留痕（真洞视觉/ghost/toast/双屏/原生弹窗）。
3. spike①②结论成文回写待澄清；透明失败回退路径实测一次。
4. `cargo check -p auto-lang` 零警告；`cargo t ui` 不回归；schema 三件套
   不回归（无 aura.at 改动）。
5. hole 默认仍为 off（默认值翻转不在本期，待实机使用反馈）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **spike① 透明窗口**：临时最小 demo（独立 bin 或 dev 开关）验证 iced
   0.14 + winit transparent + wgpu 预乘 alpha，截图像素采样洞区；结论回写。
   验证：demo 截图留痕 + 待澄清①成文。
   [✅ 已完成] 2026-08-31：demo `tmp/spike494-transparent/`（iced::daemon 同
   型）——渲染缓冲 alpha 纪律成立（截图角 alpha=0 留痕
   `tmp/spike494-transparent-buffer.png`），但视觉穿透三条路径全部实证失败
   （Opaque-surface / DComp 不上屏 / 色键分层破坏呈现；详见待澄清①）；
   生产落点=模式位+运行时回退假洞（本机实走回退，物理机复验点登记）。
2. **spike② 穿透**：探针窗（SetWindowSubclass+HTTRANSPARENT 区域）盖在
   任意他进程窗口上，SendInput 点击断言落穿；结论回写。
   验证：探针输出 + 待澄清②成文。
   [✅ 已完成] 2026-08-31：探针 `tmp/spike494-passthrough/`——子类机制正确
   （NCHITTEST 直查 -1）但 HTTRANSPARENT 落穿仅限同线程（MSDN 文义）：
   跨进程 WindowFromPoint 不跳层、真实点击被系统丢弃（探针 0 收、他进程
   底窗 0 收）。NC 子类路径不可行，详见待澄清②。
3. **模式位**：`crates/auto-lang/src/ui/session.rs` DesktopOptions +
   `shell.native.hole` boot 读入（487 同型）。
   验证：`cargo check -p auto-lang && cargo t session`。
   [✅ 已完成] DesktopState.hole_mode（session.rs）+ DesktopOptions.hole_mode
   + renderer `load_native_hole_mode`（boot 与程序位取或）+
   native_hole_mode_reads_storage_key 单测（缺省/坏值/true 三态）；
   `cargo t session` 17 绿 + `cargo check` 零错误。
4. **透明接线**（裁决改 Region 接线）：`ui/iced/renderer.rs` 窗口构建按
   hole_mode 置透明 + 失败回退。
   验证：`cargo t ui`。
   [✅ 已完成] 机制替换（SetWindowRgn，见待澄清②裁决留痕）：
   sync_native_geometry hole 分支（z 翻转 + refresh_hole_regions 洞集
   重建=全部 Docked 槽位 slot_rect）+ 失败自动回退 off（假洞 z 全量重申 +
   一行 stderr 日志，storage 不回写）+ restore_all_native_slots 退出清
   Region；`cargo t ui` 776 绿。
5. **z 序翻转**：`ui/native_dock/win32.rs` insertAfter 双步舞 + 镇压基准
   下移 + T1 顺序模型单测。
   验证：`cargo t native_dock`。
   [✅ 已完成] `raise_desktop_above`（SetWindowPos(slot, desktop) 单步即达
   ——双步舞不必要，勘误见 T1 模型注释）；镇压面无实现（现场勘误：473
   未落，无操作）；T1 z 模型单测 ×2（翻转幂等/off 不变量）+ 真实 win32
   测试 z_order_true_hole_desktop_above_native（首可见邻居断言——IME
   伴随窗楔入 z 位的实测教训）；native_dock 24 测绿。
6. **NC 子类**（spike② 证伪 → Region 排除替换）：win32.rs
   SetWindowSubclass + 洞矩形快照 + T1 命中单测。
   验证：`cargo t native_dock`。
   [✅ 已完成（机制替换）] HTTRANSPARENT 跨进程证伪（待澄清②）→
   `apply_hole_regions`（CreateRectRgn + RGN_DIFF 逐洞扣除 +
   SetWindowRgn(berase=false)，空表复位）+ `window_local_holes` 纯函数
   （裁剪/换算/空矩形丢弃）单测 + 真实测试
   hole_region_carves_input_pass_through（洞心 WindowFromPoint 直达
   下层窗/洞外仍本窗/复位恢复）。
7. **绘制纪律**：renderer.rs 槽位渲染洞内零绘制（模式分支）。
   验证：`cargo t ui`。
   [✅ 已完成] Region 裁剪下洞区绘制由 OS 丢弃（窗口在洞内不存在）；
   槽位客户区本就零 span 零底色（virtual_window::native_slot_element 空
   容器），chrome 四周照画——off 模式代码路径零改动（I3：分支仅
   z/Region，绘制无分叉）。
8. **夹具点击日志**：`tools/native-fixture/src/main.rs` 增 click 事件日志。
   验证：`cargo run --manifest-path tools/native-fixture/Cargo.toml` 手动点击目检。
   [✅ 已完成] WM_LBUTTONDOWN → `{"evt":"click",x,y}`（client 域；--offer
   模式叠加拖源触发）；构建零错误；目检由 T3 E2E 机器化覆盖（比目检强）。
9. **穿透 E2E + off 回归**：`crates/auto-lang/tests/native_dock_e2e.rs` 增
   T3 用例 + 全量 off 模式回归跑（T4）。
   验证：`cargo test -p auto-lang --features test-native-dock --test native_dock_e2e`。
   [✅ 已完成] fixture_click_passes_through_hole_region_t3（铁证：洞心
   SendInput 点击跨进程精确落 fixture（client 坐标 ±6 吻合）、洞外点击
   零泄漏；win32::test_support scratch 设施 feature 门控新增）+ T4：
   native_dock_e2e 9/9 + native_dnd_e2e 2/2 全绿（off 默认零差异）。
10. **实机冒烟 + 收尾**：T5 清单执行留痕（ghost/toast 消费演示含）；健康
    检查；状态翻 execution_done。
    验证：`cargo check -p auto-lang && cargo t ui`。
    [✅ 已完成（环境受限）] 真会话启动验证：ui_desktop + `shell.native.hole=
    true` storage 启动（无回退日志、MCP 正常）；健康检查 `cargo check`
    零新增警告（213=基线持平）+ `cargo t ui` 776 绿。**实机 dock 手势被本机
    输入拦截环境阻断**（ToDesk + Chrome Legacy Window 吞 SendInput/SC_MOVE
    全部拖拽路径——与 spike② 探针同源拦截者；三路径实测：caption 拖/候选
    位探测/SC_MOVE 注入前置前台化均未达 fixture），真洞视觉/ghost/toast
    横贯/双屏目检项**未在本环境执行**——机制证据由 T3 E2E 跨进程铁证 +
    真实 win32 测试矩阵（z 不变量/Region 穿透/复位）承载；物理机复验与
    spike① DComp 复验合并为同一条环境债务（见待澄清①）。ui_desktop 示例
    增 `AUTO_DESKTOP_HOLE=1` env 钩子（物理机复验宿主）。

## 复审记录

**复审人**：zhaopuming（/auto-plan:review，2026-08-31）
**对照基线**：worktree `.worktrees/plan-494-dev` @ a84a0faed（merge-base
39abc730f，3 提交）；master 期间被并行会话推进至 279bc5bc1（Plan 495 文档，
与本计划零交叠）。

**逐条验收裁决**：

1. **T3 穿透铁证 + T4 off 零差异——PASS**：复跑
   `--test native_dock_e2e` 9/9（含新 T3：洞心 SendInput 精确穿透 ±6、
   洞外零泄漏）+ `--test native_dnd_e2e` 2/2；评审期出现 3 连红
   （T3 点击 + 486 既有拖拽）经对照实验定性为**机器合成输入瞬态失效**
   （T5 冒烟注入扰动输入子系统：同代码红→暂存复跑绿→恢复复跑 2×绿，
   机制级断言全程绿；486 测试自带同族环境注释佐证），非代码回归。
2. **T5 实机清单留痕——PARTIAL→已批准债务**：真会话 boot 验证（hole
   storage 生效、无回退日志）；视觉清单（ghost/toast 裁剪边界/双屏/
   原生弹窗）因本机输入拦截环境不可执行——**用户裁决接受为已批准债务**
   （P494-2，物理机复验）。
3. **spike 成文 + 回退实测——PASS（评审补齐）**：①②待澄清成文完整；
   回退路径原仅代码审读，评审补 `refresh_hole_regions_at` 可测核心 +
   `hole_region_fallback_flips_mode_off`（stale hwnd 驱动真实 Err→
   hole_mode 翻 off）——实测一次达成。
4. **门禁——PASS**：`cargo tf` 3303/3303 全绿（评审唯一全量门禁）；
   `cargo t ui` 776 绿；`cargo check` 零**新增**警告（213=master 基线
   持平——计划文"零警告"按增量口径解释，基线债非本期引入）；schema
   三件套无涉（diff 无 aura.at/schema 文件，docs_gen 门不适用）。
5. **hole 默认 off——PASS**：写入点审计（session.rs:310 默认 false /
   renderer.rs:9033 boot 取或 / 8027 回退置 false / 示例 env=1 显式），
   无无条件开启点。

**遗漏/延后/workaround 扫描**：diff 零 TODO/FIXME/dbg（唯一 eprintln=
计划要求的回退日志行）；§4 NC 子类按 spike 结论证伪替换（计划文与实现
分叉已在待澄清②+裁决留痕备案）；"双步舞"勘误为单步（T1 模型注释）。
**债务登记**：P494-1（G4 覆盖层洞边裁剪）/P494-2（T5 物理机复验）/
P494-3（透明路径物理机复验）——用户批准，见 KNOWN-DEBT-AND-RISKS.md。
**注记**：win32.rs 全文件 CRLF→LF 行尾归一（python 改写副产物，内容
diff 实际 +356/-1）。

**裁决：PASS → reviewed。**

## 待澄清事项

- **① 透明 swapchain（T1 回写，2026-08-31 实测成文）**：
  - **iced 0.14 官方路径不可行（本机）**：`window::Settings.transparent` 建窗
    成功、渲染缓冲 alpha 正确（截图角部 alpha=0），但 wgpu 27 对普通 HWND
    surface 只报 `alpha_modes=[Opaque]`（NVIDIA Vulkan 与 DX12+DxgiFromHwnd
    均然）——DWM 视觉不穿透（屏采显示底层原样=角部"看似穿透"实为窗口
    整体不可见，见下）。
  - **DxgiFromVisual（DirectComposition）路径内容不上屏（本机）**：vendored
    iced_wgpu（`backend_options: from_env_or_default()` + PreMultiplied 优先）
    + `WGPU_BACKEND=dx12 WGPU_DX12_PRESENTATION_SYSTEM=visual` + vendored
    iced_winit（`with_no_redirection_bitmap(transparent)`）后：surface 能力
    报全量 alpha、PreMultiplied 选中、swapchain/DComp SetContent/Commit 全部
    成功零错误，但窗口内容（红方块）**屏幕上完全不可见**（真实截屏 +
    GDI 截区双重证实；NVIDIA 与 WARP 适配器对照同样）。**疑似 ToDesk 远程
    显示驱动环境特有**（native_dock 既有测试注记同环境对合成输入亦有干扰）；
    物理机待用户复验。三处 vendored 补丁在 `tmp/spike494-transparent/`
    （gitignored，含 vendor/iced_wgpu + vendor/iced_winit 一行级 diff）。
  - **色键分层窗（LWA_COLORKEY）不可行**：创建后补设 `WS_EX_LAYERED` 令
    flip-model 呈现整窗失效（连不透明内容也消失）。
  - **GDI 屏幕采样读不到 GPU 呈现内容**（MPO/直翻层面）：对照组（默认
    不透明配置）红方块同样不出现在 GetPixel/BitBlt 采样——屏幕级透明断言
    只能用真实截屏。
  - **生产落点**：模式位/透明接线/回退全部照计划实装；本机运行时实际
    走"透明失败→自动回退假洞"路径（回退触发=首帧洞区渲染缓冲 alpha 校验，
    可捕获 Opaque-surface 类失败；DComp 不上屏类失败无法程序化检测，属
    环境级问题，物理机复验后如成 → 透明自动生效）。
- **② HTTRANSPARENT 跨进程（T2 回写，2026-08-31 实测成文）**：
  - **机制本体工作正常**：SetWindowSubclass（DefWindowProc 宿主窗，生产
    同型）安装成功；洞心 SendMessage(WM_NCHITTEST) 直查返回 `-1`
    （HTTRANSPARENT）——子类链、坐标域（线程级 per-monitor v2 声明后）
    全部正确。
  - **跨进程落穿不成立（OS 语义级）**：① WindowFromPoint(洞心) 返回探针
    自身（不跳到 z 序下层他进程窗）；② SendInput 真实点击洞心后，**任何
    窗口都收不到**（探针 WM_LBUTTONDOWN=0、WM_NCLBUTTONDOWN=0，他进程
    底窗 click 日志为空）——系统命中测试对 HTTRANSPARENT 的下传仅限
    **同线程**窗口（MSDN WM_NCHITTEST 原文 "covered by another window in
    the same thread"），跨线程/跨进程时该点击被丢弃。非环境问题，物理机
    同样如此。
  - 结论：计划 §4 的 NC 子类路径**不可行**；输入穿透需走计划预留的
    退路 A（动态 `WS_EX_TRANSPARENT`）或 SetWindowRgn 洞排除（见下）。
  - 探针留痕：`tmp/spike494-passthrough/`（自孵化双进程 + 四断言 JSON）。
- **默认值翻转**：实机使用反馈（视觉一致性/性能体感）后另立决策。
- **多洞性能**：全屏 alpha 恒定成本（非随洞数变化）——T5 复核一次体感。
- 洞内原生窗口的菜单/文件对话框为 OS 置顶弹窗，预期天然正确（T5 验证项），
  若出现被桌面窗口遮挡的边角案例，登记债务不阻塞本期。
- **现场勘误**：计划文所述"杂散窗口镇压（EVENT_OBJECT_FOREGROUND 治理）"
  在现行代码中**不存在实现**（473 未落）——"镇压沿用/基准下移"为无操作，
  z 序翻转仅涉 insertAfter 双步舞。
- **T5 环境债务（物理机复验清单）**：①真洞实机视觉（Explorer/夹具 dock
  后洞区透出、四角/边框观感）；②ghost 跨洞与 toast 横贯的裁剪边界目检
  （G4 降级形态）；③双屏不同缩放；④洞内原生菜单/弹窗置顶；⑤DComp 透明
  路径复验（spike① 物理机分支）。本机（ToDesk 远程显示 + Chrome Legacy
  Window 输入拦截）全部四条拖拽/点击注入路径实测不可达。
- **【2026-08-31 执行期裁决】真洞机制改 SetWindowRgn 洞排除**（双 spike
  失败后的机制替换；用户问询未复，按推荐项继续，此处留痕）：z 序翻转照
  原 §3；§2 透明接线与 §4 NC 子类**替换为** `SetWindowRgn`（全窗 Region
  逐洞 RGN_DIFF 排除，客户端域=槽位客户区矩形）：视觉+输入穿透由窗口
  区域语义天然成立（HTTRANSPARENT 同线程限制与透明环境问题双双绕开）。
  G4 降级：ghost/toast 在洞边界被裁剪（不进洞内）——记 KNOWN-DEBT，待
  透明路径在物理机复验后以"Region+透明"复合形态偿还。回退路径：
  SetWindowRgn 失败 → 降级 off + 日志（对应原"透明失败回退"）。
