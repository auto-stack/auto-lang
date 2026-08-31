---
plan_id: PLAN-504
status: reviewed               # drafting → executing → execution_done → reviewed → archived
feature_name: calculator-fit-window-osconfig-stdlib
author: [kimi-code]
created_at: 2026-08-31
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "pac window:\"fit\"——首帧内容 shrink 测量自适应窗口（独立 VM 窗 + 桌面虚拟窗双路径）"
  - "stdlib Math.pow / Str.is_digit——静态分发（VM Rust shim + Vue ts_adapter 映射）"
  - "os-config per-app 配置约定——~/.config/autoos/apps/<app>/config.at，launch 时读注 theme/accent"
touched_goals:
  - "GOAL-009: 虚拟桌面与桌面 Shell——示例应用桌面化三件套：fit 窗口 / 设置上移 os-config / stdlib 静态分发"
  - "GOAL-010: 示例应用轨道——011-calculator 作为桌面移植范式样板"
  - "GOAL-007: AutoUI 跨端视觉一致——静态分发函数双端（Vue TS / VM Rust）同语义"

affects: [auto-lang/ui, auto-lang/vm]
current_step: 8
total_steps: 8
---

# [PLAN-504] 示例桌面化三件套：fit 自适应窗口 + title/settings 上移 os-config + stdlib 静态分发（011-calculator 落地）

## 变更摘要

以 011-calculator 为样板，为 examples/ui 应用移植虚拟桌面做三项改造：

1. **`window: "fit"` 自适应窗口**：pac.at `window` 字段在 `"WxH"` 之外新增
   `"fit"` 取值——VM 独立窗首帧按内容 shrink 测量后 resize；虚拟桌面窗
   用测量值替代"可用区 60%"的写死初始尺寸（session.rs:1716）。
2. **title/settings 上移**：应用内删除 `ExampleHeader`（标题栏 + 主题/accent
   设置面板）；标题由 pac.at `title:` + 桌面 chrome 提供；theme/accent 成为
   os-config 的 per-app 配置模块（`~/.config/autoos/apps/calculator.at`），
   launch 时读取注入（运行期热更新为非目标）。
3. **常用函数静态分发**：`pow` 收进 stdlib `math`（VM `#[vm]` + Rust `powf`；
   Vue 端 `math.*`→`Math.*` 通配 arm 已覆盖）；新增 str 谓词方法
   `is_digit()`（VM Rust shim + Vue ts_adapter 映射 `/^\d$/.test`）；
   应用内手写的 `is_digit`/`pow` 删除，`is_op`（应用专有运算符表）保留。

## 目标

- **G1 fit 窗口（双路径）**：`window: "fit"` 在独立 VM 窗（`auto run -r vm`）
  与虚拟桌面窗（desktop launch）都生效：窗口初始尺寸 = 内容 shrink 测量值
  （clamp 到 [200, 可用区]）。Vue 端语义 = 应用去掉 `min-h-screen` 居中外壳。
- **G2 title/settings 上移**：011 app.at 无 `ExampleHeader`、无 settings 状态；
  pac.at 补 `title: "Calculator"`；calculator 的 theme/accent 在 os-config
  注册为配置模块并可经其通用编辑器编辑；launch 时读回注入。
- **G3 静态分发**：`math.pow` 与 `str.is_digit()` 双端可用（VM 走
  `native_registry` 自动扫描 + Rust shim；Vue 走 ts_adapter 映射），
  011 app.at 改用之，删除本地 `is_digit`/`pow` 实现。
- **非目标**：主题运行时热更新（改配置需重启应用生效）；os-config 通用
  编辑器自身功能改动；`is_op` 等应用专有 helper 入 stdlib；其他示例的
  批量迁移（后续计划按本样板展开）。

## 架构方案

```
pac.at: window: "fit" ──auto run──▶ AUTO_VM_WINDOW=fit ──▶ renderer 首帧
                                                                │ shrink 测量
        桌面 launch（app_registry → LaunchSpec.fit）───────────┤
                                                                ▼
                                          pending_window_resize / 虚拟窗初始 rect

~/.config/autoos/apps/calculator.at ◀── os-config daemon(:17701) + 通用编辑器
        │ launch 时读取
        ▼
AUTO_UI_THEME / AUTO_UI_ACCENT 注入（沿用既有 env 注入惯用法）

app.at: ch.is_digit() ──┬─ VM: stdlib/auto/str.at ext str #[vm] → native_registry
                        │      自动扫描 → shim_str_is_digit (Rust)
                        └─ Vue: ts_adapter 方法映射 → /^\d$/.test(ch)
        math.pow(a,b) ──┬─ VM: math.vm.at #[vm] → shim_math_pow (f64::powf)
                        └─ Vue: ts_adapter "math" arm 通配 → Math.pow(a,b)
```

## 需求分析与背景调查

（2026-08-31 现场核验）

- **窗口现状**：pac.at `window: "WxH"`（Plan 411，`crates/auto-man/src/pac.rs:94`
  `Option<(f32,f32)>`；`crates/auto/src/main.rs:967` 注入 `AUTO_VM_WINDOW`；
  `renderer.rs:4560 startup_window_size()` 消费，缺省 1280x800）。无自适应
  模式。iced 0.14（`crates/auto-lang/Cargo.toml:164`）无内建 shrink-to-content，
  但基建齐全：`pending_window_resize`/`initial_resize_done`（session.rs）、
  布局测量（`ui/layout.rs`）、native_dock `observe_min_size_estimate`
  （`ui/native_dock/mod.rs:307`）先例。虚拟桌面窗初始尺寸写死可用区 60%
  （session.rs:1716）；`LaunchSpec`（session.rs:1256）当前无 size/fit 字段。
- **title/settings 现状**：pac.at 已有 `title:`（pac.rs:99）/`icon:`/
  `theme:`/`accent:`（Plan 458/463）；`LaunchSpec.title` 已驱动桌面 chrome
  标题。011 的 pac.at 已声明 icon/theme/accent 但无 `title:`。应用内
  `ExampleHeader`（examples/ui/common/src/front/header.at）自带标题栏 +
  主题/accent 设置面板，与宿主职责重复。os-config（相邻仓 auto-os-config）
  = 统一设置中心：daemon(:17701) 读写 `~/.config/autoos/*.at` + 形状驱动
  通用编辑器 + 模块注册表；桌面 shell 已接通（Plan 501 归档：daemon 懒起、
  外部仓 app 注册、settings 面板入口）。全局 theme/accent 状态已有
  （`ui/style/theme.rs`：`set_dark_mode`/`set_accent_name`，AUTO_UI_THEME
  env 惯用法见 `crates/auto-man/src/vue.rs:4705`）。
- **静态分发现状**：VM 端 `stdlib/auto/X.vm.at` 的 `#[vm]` 声明由
  `vm/native_registry.rs:498` 自动扫描注册（ext 块方法见 :445-464），Rust
  实现在 `vm/ffi/stdlib.rs`（`#[auto_macros::rust_fn("Char.is_digit")]`
  :1843 等）。Vue 端 `ui_gen/ts_adapter.rs`：`try_transpile_builtin_call`
  （:1904，模块方法 `math.*`→`Math.*` 通配 :2076）+ str 方法映射表
  （:1370-1488，`.len()`→`.length` 等）。stdlib `char.is_digit` 签名是
  int codepoint（stdlib/auto/char.at），与计算器的单字符 str 用法不贴，
  故采用 ext str 谓词路线。stdlib `math` 无 `pow`（math.at 仅
  abs/min/max/sqrt）。
- **前置**：Plan 411（window 字段）/458（theme/accent）/463（icon）/462+
  （桌面 WM）/501（os-config 集成）均 ✅ 归档。

## 详细设计

### 1. `window: "fit"`（双路径）

- **pac.rs**：`window` 解析支持 `"fit"` 关键字。结构侧新增
  `pub window_fit: bool`（`window: Option<(f32,f32)>`` 保持原义不变，
  两者互斥：解析到 `"fit"` 时 `window=None, window_fit=true`；非法值告警
  回退现状）。
- **auto/src/main.rs:967**：`window_fit` 时 `AUTO_VM_WINDOW=fit`。
- **renderer.rs**：`startup_window_size()` 遇 `fit` 回缺省尺寸 + 记录
  fit 待生效标记（线程局域或 storage publish，仿 PLAN-046-B
  `vm.window_inner_height` 模式）；首帧 view 构建后对根元素以
  `Length::Shrink` 在宽松 `Limits` 下测量得内容尺寸，clamp 到
  [200x200, 可用区]，经既有 `pending_window_resize` 通道 resize（一次性，
  `initial_resize_done` 同槽位标记）。
- **桌面路径**：`LaunchSpec` 增 `fit: bool`；`app_registry` 从 pac 条目
  透传；session.rs:1716 处分支：fit → 首帧测量值作为虚拟窗初始 rect
  （级联定位逻辑 `cascade_rect` 不变，仅 size 来源换）。
- **Vue**：无窗口控制；011 app.at 去掉 `min-h-screen`/`center` 外壳（随
  修改 2 一并改）。

### 2. title/settings 上移

- **011 app.at**：删 `use common.header` 与 `ExampleHeader` 调用；删
  settings 相关 msg/model（`ToggleSettings` 等不在本 app，ExampleHeader
  自含）；`dark_mode`/`accent_color` model 保留（样式分支仍消费），但不再
  有应用内切换入口——初值即 pac 声明值。布局外壳从 `min-h-screen + center`
  改为内容即页面（w-fit 语义）。
- **011 pac.at**：补 `title: "Calculator"`。
- **os-config 侧**（跨仓 `../auto-os-config`，按其仓 worktree 规则）：
  注册配置模块 `apps/calculator` → `~/.config/autoos/apps/calculator.at`
  （shape：`theme: "dark"|"light"`、`accent: 5 色`、`mode: "basic"|...`）。
  通用编辑器零手写获得设置 UI。
- **launch 读取**：桌面 launch / `auto run` 时若存在该配置文件则读值注入
  `AUTO_UI_THEME`/`AUTO_UI_ACCENT`（优先级：os-config 文件 > pac.at >
  内置缺省；`--theme` CLI 仍最高）。读取点选在既有 env 注入链路旁
  （auto/src/main.rs 与桌面 launch 臂各一处，薄读文件，不经 daemon——
  启动期 daemon 可能未起，直读文件零依赖）。

### 3. stdlib 静态分发

- **spike 先行**：验证 UI track 里 stdlib 模块调用的解析形态
  （`Math.pow` vs `math.pow`，use 别名在 vue codegen 的可见性），据此定
  呼叫语法并补 ts_adapter 匹配（如需大小写双匹配）。
- **math.pow**：`stdlib/auto/math.at` 增 `pub fn pow(base double, exp double) double;`；
  `math.vm.at` 增 `#[vm]` 声明；`vm/ffi/stdlib.rs` 增
  `#[auto_macros::rust_fn("Math.pow")] shim_math_pow`（`f64::powf`）。
  Vue 端 `"math"` arm 通配已覆盖（spike 确认大小写）。
- **str 谓词**：`stdlib/auto/str.at` 的 `ext str` 增
  `#[vm] fn is_digit() bool`（语义：单字符且 ASCII 数字；多字符恒 false）；
  `vm/ffi/stdlib.rs` 增 `Str.is_digit` shim；`ts_adapter.rs` 方法表增
  `"is_digit"` → `/^[0-9]$/.test(recv)`。
- **011 app.at**：删本地 `is_digit`（504-536 行的 if 链）与 `pow`
  （560-578 行），改用 `ch.is_digit()` / `math.pow`；`is_op` 保留；
  `eval_expr` 主体不变。

## 测试设计

- **单测**（cargo t 档）：
  - pac.rs：`window: "fit"` 解析（含与 `"WxH"` 互斥、非法值回退）；
  - ts_adapter：`is_digit` 映射快照（recv 转译 + 正则包裹）；
  - native_registry：`Math.pow`/`Str.is_digit` 自动扫描注册断言；
  - VM：`crates/auto-lang/test/vm/18_ffi/` 增 `pow`、`str_is_digit` 用例
    （仿 013_char_is_digit）。
- **fit 测量**：layout 层纯逻辑单测（测量值 clamp 边界：小于 200、大于
  可用区）；renderer 首帧 resize 走 `pending_window_resize` 的 session 级
  测试（仿 session.rs:2919 `window_registration_routes_and_transfers_pending_size`）。
- **011 双端**：更新 `tests/test_011_vm.py` / `test_011_vue.mjs`——移除
  settings 面板断言，新增：无 header 元素、窗口/视口尺寸贴合内容（VM 端
  断言 fit 生效）、`2*(3+4)=` 等表达式求值回归（覆盖 is_digit/pow 分发）。
  自动化走 `.agents/skills/autoui-verifier` 双端一致性。
- **门禁分级**：本计划改 Rust 源码（renderer/session/ts_adapter/stdlib）
  → Category B：`cargo check -p auto-lang` + 局部 `cargo t <module>`；
  合入前一次 `cargo tf`。

## 验收标准

1. 011 pac.at `window: "fit"` + `title: "Calculator"`；`auto run -r vm`
   独立窗与虚拟桌面 launch 均以内容尺寸开窗（无大片空白居中外壳）。
2. 011 app.at 无 ExampleHeader / settings UI；桌面 chrome 显示标题；
   theme/accent 可由 os-config 编辑 `apps/calculator.at` 并在下次 launch
   生效。
3. `ch.is_digit()` 与 `math.pow` 在 Vue/VM 双端同语义可用；011 求值
   回归双端绿（含科学模式 `2^10`、`sqrt` 路径）。
4. 复审门禁：`cargo tf` 全量绿；无新增编译警告。

## 执行步骤

1. **[S1] spike：UI track stdlib 调用形态**——最小 .at widget 调
   `math.sqrt`/`Char.is_digit`，双端编译，确认解析语法与 ts_adapter
   缺口；结论写入本文件「待澄清事项」关闭对应条目。
   验证：spike 用例双端跑通。
2. **[S2] stdlib 静态分发**——math.pow + str.is_digit 三处声明/实现 +
   ts_adapter 映射 + 18_ffi 用例。
   验证：`cargo t native_registry` + `cargo tv` 新用例 + ts_adapter 单测。
3. **[S3] pac.rs `window: "fit"`**——解析 + 互斥 + 单测；main.rs env
   注入。验证：`cargo t -p auto-man`（或对应包）pac 模块。
4. **[S4] VM 独立窗 fit**——renderer 首帧测量 + resize 通道 + clamp
   单测。验证：`cargo t` 相关模块 + 手动 `auto run -r vm` 011 目测。
5. **[S5] 桌面虚拟窗 fit**——LaunchSpec.fit + app_registry 透传 +
   session.rs:1716 分支 + session 级测试。验证：`cargo t session`。
6. **[S6] 011 应用改造**——删 header/settings、外壳改内容即页面、
   pac.at 补 title/fit、is_digit/pow 换静态分发；同步更新双端测试脚本。
   验证：autoui-verifier 双端绿。
7. **[S7] os-config 配置模块**（跨仓）——注册 `apps/calculator` +
   launch 读取注入（本仓 launch 臂）。验证：编辑配置 → 重启 app →
   主题生效；os-config 仓侧 e2e 不破。
8. **[S8] 收尾**——docs/specs 回写准备、复审记录、`cargo tf` 全量。

## 复审记录

（2026-09-01，/auto-plan:review）

**验收逐条核对**（verify, don't trust）：

1. ✅ pac.at `window: "fit"` + `title: "Calculator"` 已落（011 pac.at）。
   独立 VM 窗实机：窗口自 1293x836 收缩至 397x428（内容 w-96=384 +
   chrome：TITLEBAR_H 28 + BORDER×2），MCP 截图确认无居中外壳空白。
   桌面路径：session 级单测 `apply_fit_measured_shrinks_desktop_vwin`
   （386x422 = 384+2·BORDER / 392+28+2·BORDER）+ standalone 实机；
   桌面真实 launch 实况 e2e 受阻于 winit 合成输入（债项 P504-3）。
2. ✅ 011 app.at 无 ExampleHeader/settings UI（双端 e2e 含"无 Settings
   按钮"断言）。os-config 模块 `auto-calculator` 经 daemon（端口实测
   17901）热注册成功，`/api/config/auto-calculator` 返回
   `{"theme":"dark","accent":"indigo","mode":"basic"}`；launch 读注链
   实测打印 `UI theme: light (from os-config)` / `UI accent: coral
   (from os-config)`（临时改值验证后已恢复 dark/indigo）。
3. ✅ `ch.is_digit()` / `math.pow` 双端同语义：VM 文件测试
   056_math_pow/057_str_is_digit 过；Vue 端 `2^10=1024` e2e 过；
   ts_adapter 映射单测 `str_is_digit_maps_to_regex_test` 过。
4. ✅ `cargo tf` 全量 3326 过（含 1M churn 档）；ui-iced 档
   fit/launch/osconfig 相关 45 测试全过；无新增编译警告；新增代码
   rustfmt 偏差已清零（四改动文件 hunk 数与 master 对齐，新文件
   osconfig_apps.rs 零偏差）。

**遗漏/延后/Workaround 扫描**：见 KNOWN-DEBT-AND-RISKS.md P504-1~4
（18_ffi 存量格式化腐烂 / fit 一次性测量 Scientific 裁剪 / desktop
实况 e2e 输入通道 / boot 直开无 pac 语境）。无未批准的暗延后。

**与计划文本的偏差（已裁定）**：

- 配置实际路径为 `~/.config/autoos/apps/calculator/config.at`（目录
  形态），非计划文的 `apps/calculator.at` 单文件——循 musk 先例，
  待澄清③裁定以 os-config 仓现状为准。
- pac.at 取值名保留 `window: "fit"`（用户裁定，不改 `auto`）。
- 关键缺陷修复：`session.rs register_window` 同键覆盖（Opened 幂等
  兜底）会抹掉 boot 置位的 fit_pending——改为覆盖前保留旧值；此为
  standalone fit 最初不生效的根因，已补回归测试
  `register_window_overwrite_preserves_fit_pending`。

## 待澄清事项

1. ~~UI track 中 stdlib 模块调用的书写形态（`Math.pow` vs `math.pow`）与
   vue codegen 对无 body extern 声明的解析~~——**已关闭（S1 spike +
   S2 落地）**：`math.pow` 小写形态双端可用，ts_adapter `"math"` 通配
   arm 覆盖；str 谓词走方法表 `"is_digit"` arm。
2. 应用样式对 `.dark_mode` model 的依赖在"设置上移"后的长期形态
   （语义 token 化 vs 保留 model 初值）——本期保留 model 初值方案，
   token 化另立。
3. ~~os-config 仓侧改动量（注册表条目形态）~~——**已关闭（S7）**：
   循 musk 先例，模块注册 = `modules.d/auto-calculator.at` +
   配置 = `apps/calculator/config.at` 目录形态，零 daemon 契约改动。
