---
plan_id: PLAN-494
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-native-true-hole
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
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
2. **spike② 穿透**：探针窗（SetWindowSubclass+HTTRANSPARENT 区域）盖在
   任意他进程窗口上，SendInput 点击断言落穿；结论回写。
   验证：探针输出 + 待澄清②成文。
3. **模式位**：`crates/auto-lang/src/ui/session.rs` DesktopOptions +
   `shell.native.hole` boot 读入（487 同型）。
   验证：`cargo check -p auto-lang && cargo t session`。
4. **透明接线**：`ui/iced/renderer.rs` run_dynamic_desktop 窗口构建按
   hole_mode 置透明 + 失败回退。
   验证：`cargo t ui`。
5. **z 序翻转**：`ui/native_dock/win32.rs` insertAfter 双步舞 + 镇压基准
   下移 + T1 顺序模型单测。
   验证：`cargo t native_dock`。
6. **NC 子类**：win32.rs SetWindowSubclass + 洞矩形快照 + T1 命中单测。
   验证：`cargo t native_dock`。
7. **绘制纪律**：renderer.rs 槽位渲染洞内零绘制（模式分支）。
   验证：`cargo t ui`。
8. **夹具点击日志**：`tools/native-fixture/src/main.rs` 增 click 事件日志。
   验证：`cargo run --manifest-path tools/native-fixture/Cargo.toml` 手动点击目检。
9. **穿透 E2E + off 回归**：`crates/auto-lang/tests/native_dock_e2e.rs` 增
   T3 用例 + 全量 off 模式回归跑（T4）。
   验证：`cargo test -p auto-lang --features test-native-dock --test native_dock_e2e`。
10. **实机冒烟 + 收尾**：T5 清单执行留痕（ghost/toast 消费演示含）；健康
    检查；状态翻 execution_done。
    验证：`cargo check -p auto-lang && cargo t ui`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **① 透明 swapchain（T1 回写）**：iced 0.14 预乘 alpha 的实际透传点
  （Settings vs surface API）；失败回退实测记录。
- **② HTTRANSPARENT 跨进程（T2 回写）**：若实证不可行——退路 A：洞区
  动态 `WS_EX_TRANSPARENT`（整窗切换粒度粗）；退路 B：放弃输入穿透、真洞
  仅作视觉层（输入仍走假洞语义=原生在上）——届时覆盖层价值减半，计划
  范围重估。
- **默认值翻转**：实机使用反馈（视觉一致性/性能体感）后另立决策。
- **多洞性能**：全屏 alpha 恒定成本（非随洞数变化）——T5 复核一次体感。
- 洞内原生窗口的菜单/文件对话框为 OS 置顶弹窗，预期天然正确（T5 验证项），
  若出现被桌面窗口遮挡的边角案例，登记债务不阻塞本期。
